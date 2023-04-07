#![allow(unused)]
pub use crate::Selection;
use crate::{Options, TextBuffer, TextEdit};
use nalgebra::Point2;
use std::sync::Arc;
pub use ultron_syntaxes_themes::{Style, TextHighlighter};
use async_delay::Throttle;

/// An editor with core functionality platform specific UI
pub struct Editor<XMSG> {
    options: Options,
    text_edit: TextEdit,
    text_highlighter: TextHighlighter,
    /// lines of highlighted ranges
    highlighted_lines: Vec<Vec<(Style, String)>>,
    /// Other components can listen to the an event.
    /// When the content of the text editor changes, the change listener will be emitted
    change_listeners: Vec<Callback<String, XMSG>>,
    /// a cheaper listener which doesn't need to assemble the text content
    /// of the text editor everytime
    change_notify_listeners: Vec<Callback<(), XMSG>>,
    throttle: Throttle,
}


#[derive(Debug)]
pub enum Command {
    IndentForward,
    IndentBackward,
    BreakLine,
    DeleteBack,
    DeleteForward,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveLeftStart,
    MoveRight,
    MoveRightEnd,
    InsertChar(char),
    ReplaceChar(char),
    InsertText(String),
    PasteTextBlock(String),
    MergeText(String),
    /// set a new content to the editor, resetting to a new history for undo/redo
    SetContent(String),
    Undo,
    Redo,
    BumpHistory,
    SetSelection(Point2<i32>, Point2<i32>),
    SelectAll,
    ClearSelection,
    SetPosition(i32, i32),
}

pub struct Callback<IN, OUT> {
    func: Arc<dyn Fn(IN) -> OUT>,
}

impl<IN, F, OUT> From<F> for Callback<IN, OUT>
where
    F: Fn(IN) -> OUT+ 'static ,
{
    fn from(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<IN, OUT> Callback<IN, OUT> {
    /// This method calls the actual callback.
    pub fn emit(&self, input: IN) -> OUT {
        (self.func)(input)
    }
}

impl<XMSG> Editor<XMSG> {
    pub fn from_str(options: Options, content: &str) -> Self {
        let mut text_highlighter = TextHighlighter::default();
        if let Some(theme_name) = &options.theme_name {
            text_highlighter.select_theme(theme_name);
        }
        text_highlighter.set_syntax_token(&options.syntax_token);

        let text_edit = TextEdit::from_str(content);
        let highlighted_lines =
            Self::highlight_lines(&text_edit, &mut text_highlighter);

        Editor {
            options,
            text_edit,
            text_highlighter,
            highlighted_lines,
            change_listeners: vec![],
            change_notify_listeners: vec![],
            throttle: Throttle::from_interval(100),
        }
    }

    pub fn text_buffer(&self) -> &TextBuffer {
        &self.text_edit.text_buffer()
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.text_edit.set_selection(start, end);
    }

    pub fn selection(&self) -> &Selection {
        self.text_edit.selection()
    }

    pub fn set_selection_start(&mut self, start: Point2<i32>) {
        self.text_edit.set_selection_start(start);
    }

    pub fn set_selection_end(&mut self, end: Point2<i32>) {
        self.text_edit.set_selection_end(end);
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        self.text_edit.get_char(x, y)
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.text_edit.get_position()
    }

    pub fn get_content(&self) -> String {
        self.text_edit.get_content()
    }

    pub fn total_lines(&self) -> usize {
        self.text_edit.total_lines()
    }

    pub fn highlight_lines(
        text_edit: &TextEdit,
        text_highlighter: &mut TextHighlighter,
    ) -> Vec<Vec<(Style, String)>> {
        text_edit
            .lines()
            .iter()
            .map(|line| {
                text_highlighter
                    .highlight_line(line)
                    .expect("must highlight")
                    .into_iter()
                    .map(|(style, line)| (style, line.to_owned()))
                    .collect()
            })
            .collect()
    }

    /// rehighlight the texts
    pub fn rehighlight(&mut self) {
        self.text_highlighter.reset();
        self.highlighted_lines =
            Self::highlight_lines(&self.text_edit, &mut self.text_highlighter);
    }

    pub fn text_highlighter(&self) -> &TextHighlighter {
        &self.text_highlighter
    }
}

impl<XMSG> Editor<XMSG> {


    pub async fn process_commands(&mut self, commands: impl IntoIterator<Item = Command>) -> Vec<XMSG>{
        let results:Vec<bool> = commands.into_iter().map(|command|
            self.process_command(command)
        ).collect();
        if  results.into_iter().any(|v|v){
            self.content_has_changed().await
        }else{
            vec![]
        }
    }

    /// TODO: convert this into process_commands
    /// where each command marks whether the content has changed or not
    /// then once all of the commands have been executed,
    /// the emit and rehighlight will commence
    ///
    /// TODO option2: have a boolean flag, for each of the command to determine
    /// if the editor content changed or not
    ///
    pub fn process_command(&mut self, command: Command) -> bool {
        match command {
            Command::IndentForward => {
                let indent = "    ";
                self.text_edit.command_insert_text(indent);
                true
            }
            Command::IndentBackward => {
                true
            }
            Command::BreakLine => {
                self.text_edit.command_break_line();
                true
            }
            Command::DeleteBack => {
                self.text_edit.command_delete_back();
                true
            }
            Command::DeleteForward => {
                self.text_edit.command_delete_forward();
                true
            }
            Command::MoveUp => {
                self.command_move_up();
                false
            }
            Command::MoveDown => {
                self.command_move_down();
                false
            }
            Command::PasteTextBlock(text) => {
                self.text_edit.paste_text_block_mode(text);
                true
            }
            Command::MergeText(text) => {
                self.text_edit.merge_text(text);
                true
            }
            Command::MoveLeft => {
                self.text_edit.command_move_left();
                false
            }
            Command::MoveLeftStart => {
                self.text_edit.command_move_left_start();
                false
            }
            Command::MoveRightEnd => {
                self.text_edit.command_move_right_end();
                false
            }
            Command::MoveRight => {
                self.command_move_right();
                false
            }
            Command::InsertChar(c) => {
                self.text_edit.command_insert_char(c);
                true
            }
            Command::ReplaceChar(c) => {
                self.text_edit.command_replace_char(c);
                true
            }
            Command::InsertText(text) => {
                self.text_edit.command_insert_text(&text);
                true
            }
            Command::SetContent(content) => {
                self.text_edit = TextEdit::from_str(&content);
                true
            }
            Command::Undo => {
                self.text_edit.command_undo();
                true
            }
            Command::Redo => {
                self.text_edit.command_redo();
                true
            }
            Command::BumpHistory => {
                self.text_edit.bump_history();
                false
            }
            Command::SetSelection(start, end) => {
                self.text_edit.command_set_selection(start, end);
                false
            }
            Command::SelectAll => {
                self.text_edit.command_select_all();
                false
            }
            Command::ClearSelection => {
                self.text_edit.clear_selection();
                false
            }
            Command::SetPosition(x, y) => {
                self.command_set_position(x, y);
                false
            }
        }
    }

    fn command_move_up(&mut self) {
        if self.options.use_virtual_edit {
            self.text_edit.command_move_up();
        } else {
            self.text_edit.command_move_up_clamped();
        }
    }

    fn command_move_down(&mut self) {
        if self.options.use_virtual_edit {
            self.text_edit.command_move_down();
        } else {
            self.text_edit.command_move_down_clamped();
        }
    }

    fn command_move_left(&mut self) {
        self.text_edit.command_move_left();
    }

   fn command_move_right(&mut self) {
        if self.options.use_virtual_edit {
            self.text_edit.command_move_right();
        } else {
            self.text_edit.command_move_right_clamped();
        }
    }

   fn command_set_position(&mut self, cursor_x: i32, cursor_y: i32) {
        if self.options.use_virtual_edit {
            self.text_edit
                .command_set_position(cursor_x as usize, cursor_y as usize);
        } else {
            self.text_edit.command_set_position_clamped(
                cursor_x as usize,
                cursor_y as usize,
            );
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.text_edit.selected_text()
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.text_edit.cut_selected_text()
    }

    pub fn selected_text_block_mode(&self) -> Option<String> {
        self.text_edit.selected_text_block_mode()
    }

    pub fn cut_selected_text_block_mode(&mut self) -> Option<String> {
        self.text_edit.cut_selected_text_block_mode()
    }

    pub fn clear(&mut self) {
        self.text_edit.clear();
    }


    /// call this when a command changes the text_edit content
    /// This will rehighlight the content
    /// and emit the external XMSG in the event listeners
    ///
    /// TODO: create a flag to mark that the content has changed
    /// for each command, then finally it will be used to determine
    /// whether to execute rehilight and emit change listener
    pub async fn content_has_changed(&mut self) -> Vec<XMSG> {
        self.rehighlight_and_emit().await
        //self.throttled_content_has_changed().await
    }

    async fn rehighlight_and_emit(&mut self) -> Vec<XMSG>{
        if self.options.use_syntax_highlighter{
            self.rehighlight();
        }
        self.emit_on_change_listeners()
    }

    /// check last executed, if elapsed time is lesser than the allowed duration
    /// then make a delayed call to this function with a delayed of
    /// allowed_duration - (now - last);
    /// if time since last executed is greater than execute the function
    async fn throttled_content_has_changed(&mut self) -> Vec<XMSG> {
        if self.throttle.should_execute() {
            //log::info!("executing the content changed");
            self.throttle.set_executing(true);
            let msgs = self.rehighlight_and_emit().await;
            self.throttle.executed();
            msgs
        } else if self.throttle.is_dirty() {
            log::info!("content is dirty");
            let remaining = self.throttle.remaining_time()
                .expect("must have a remaining time");
            log::info!("remaining time for next execution: {}", remaining);
            async_delay::delay(remaining + 1).await;
            self.throttle.set_executing(true);
            let msgs = self.rehighlight_and_emit().await;
            self.throttle.executed();
            msgs
        } else {
            log::info!("no execution..");
            vec![]
        }
    }

    /// Attach a callback to this editor where it is invoked when the content is changed.
    ///
    /// Note:The content is extracted into string and used as a parameter to the function.
    /// This may be a costly operation when the editor has lot of text on it.
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_listeners.push(cb);
        self
    }

    pub fn add_on_change_listener<F>(&mut self, f: F)
    where
        F: Fn(String) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_listeners.push(cb);
    }

    /// Attach an callback to this editor where it is invoked when the content is changed.
    /// The callback function just notifies the parent component that uses the Editor component.
    /// It will be up to the parent component to extract the content of the editor manually.
    ///
    /// This is intended to be used in a debounced or throttled functionality where the component
    /// decides when to do an expensive operation based on time and recency.
    ///
    ///
    pub fn on_change_notify<F>(mut self, f: F) -> Self
    where
        F: Fn(()) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_notify_listeners.push(cb);
        self
    }

    pub fn add_on_change_notify<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_notify_listeners.push(cb);
    }

    fn emit_on_change_listeners(&self) -> Vec<XMSG> {
        let mut extern_msgs: Vec<XMSG> = vec![];
        if !self.change_listeners.is_empty() {
            let content = self.text_edit.get_content();
            let xmsgs: Vec<XMSG> = self
                .change_listeners
                .iter()
                .map(|listener| listener.emit(content.clone()))
                .collect();
            extern_msgs.extend(xmsgs);
        }

        if !self.change_notify_listeners.is_empty() {
            let xmsgs: Vec<XMSG> = self
                .change_notify_listeners
                .iter()
                .map(|notify| notify.emit(()))
                .collect();
            extern_msgs.extend(xmsgs);
        }

        extern_msgs
    }

    pub fn numberline_wide(&self) -> usize {
        self.text_edit.numberline_wide()
    }

    pub fn highlighted_lines(&self) -> &[Vec<(Style, String)>] {
        &self.highlighted_lines
    }
}
