pub use crate::Selection;
use crate::{BaseOptions, SelectionMode, TextBuffer, TextEdit};
use nalgebra::Point2;
use std::marker::PhantomData;
use std::sync::Arc;
pub use ultron_syntaxes_themes::{Style, TextHighlighter};

/// An editor with core functionality platform specific UI
pub struct BaseEditor<XMSG> {
    options: BaseOptions,
    text_edit: TextEdit,
    /// Other components can listen to the an event.
    /// When the content of the text editor changes, the change listener will be emitted
    #[cfg(feature = "callback")]
    change_listeners: Vec<Callback<String, XMSG>>,
    /// a cheaper listener which doesn't need to assemble the text content
    /// of the text editor everytime
    #[cfg(feature = "callback")]
    change_notify_listeners: Vec<Callback<(), XMSG>>,
    _phantom: PhantomData<XMSG>,
}

impl<XMSG> AsRef<TextEdit> for BaseEditor<XMSG> {
    fn as_ref(&self) -> &TextEdit {
        &self.text_edit
    }
}

impl<XMSG> Default for BaseEditor<XMSG> {
    fn default() -> Self {
        Self {
            options: BaseOptions::default(),
            text_edit: TextEdit::default(),
            #[cfg(feature = "callback")]
            change_listeners: vec![],
            #[cfg(feature = "callback")]
            change_notify_listeners: vec![],
            _phantom: PhantomData,
        }
    }
}

impl<XMSG> Clone for BaseEditor<XMSG> {
    fn clone(&self) -> Self {
        Self {
            options: self.options.clone(),
            text_edit: self.text_edit.clone(),
            #[cfg(feature = "callback")]
            change_listeners: self.change_listeners.clone(),
            #[cfg(feature = "callback")]
            change_notify_listeners: self.change_notify_listeners.clone(),
            _phantom: self._phantom.clone(),
        }
    }
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
    SetPosition(Point2<i32>),
}

pub struct Callback<IN, OUT> {
    func: Arc<dyn Fn(IN) -> OUT>,
}

impl<IN, F, OUT> From<F> for Callback<IN, OUT>
where
    F: Fn(IN) -> OUT + 'static,
{
    fn from(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<IN, OUT> Clone for Callback<IN, OUT> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

impl<IN, OUT> Callback<IN, OUT> {
    /// This method calls the actual callback.
    pub fn emit(&self, input: IN) -> OUT {
        (self.func)(input)
    }
}

impl<XMSG> BaseEditor<XMSG> {
    pub fn from_str(options: &BaseOptions, content: &str) -> Self {
        let text_edit = TextEdit::from_str(content);

        BaseEditor {
            options: options.clone(),
            text_edit,
            #[cfg(feature = "callback")]
            change_listeners: vec![],
            #[cfg(feature = "callback")]
            change_notify_listeners: vec![],
            _phantom: PhantomData,
        }
    }

    pub fn text_buffer(&self) -> &TextBuffer {
        self.text_edit.text_buffer()
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.text_edit.set_selection(start, end);
    }

    pub fn selection(&self) -> &Selection {
        self.text_edit.selection()
    }

    pub fn selected_text(&self) -> Option<String> {
        match self.options.selection_mode {
            SelectionMode::Linear => self.text_edit.selected_text_in_linear_mode(),
            SelectionMode::Block => self.text_edit.selected_text_in_block_mode(),
        }
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        match self.options.selection_mode {
            SelectionMode::Linear => self.text_edit.cut_selected_text_in_linear_mode(),
            SelectionMode::Block => self.text_edit.cut_selected_text_in_block_mode(),
        }
    }

    pub fn is_selected(&self, loc: Point2<i32>) -> bool {
        match self.options.selection_mode {
            SelectionMode::Linear => self.text_edit.is_selected_in_linear_mode(loc),
            SelectionMode::Block => self.text_edit.is_selected_in_block_mode(loc),
        }
    }

    pub fn clear_selection(&mut self) {
        self.text_edit.clear_selection()
    }

    pub fn set_selection_start(&mut self, start: Point2<i32>) {
        self.text_edit.set_selection_start(start);
    }

    pub fn set_selection_end(&mut self, end: Point2<i32>) {
        self.text_edit.set_selection_end(end);
    }

    pub fn get_char(&self, loc: Point2<usize>) -> Option<char> {
        self.text_edit.get_char(loc)
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
}

impl<XMSG> BaseEditor<XMSG> {
    pub fn process_commands(&mut self, commands: impl IntoIterator<Item = Command>) -> Vec<XMSG> {
        let results: Vec<bool> = commands
            .into_iter()
            .map(|command| self.process_command(command))
            .collect();

        #[cfg(feature = "callback")]
        if results.into_iter().any(|v| v) {
            self.emit_on_change_listeners()
        } else {
            vec![]
        }
        #[cfg(not(feature = "callback"))]
        vec![]
    }

    /// process the supplied command to text_edit
    pub fn process_command(&mut self, command: Command) -> bool {
        match command {
            Command::IndentForward => {
                let indent = "    ";
                self.text_edit.command_insert_text(indent);
                true
            }
            Command::IndentBackward => true,
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
                self.text_edit.paste_text_in_block_mode(text);
                true
            }
            Command::MergeText(text) => {
                self.text_edit.command_merge_text(text);
                true
            }
            Command::MoveLeft => {
                self.command_move_left();
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
            Command::SetPosition(pos) => {
                self.command_set_position(pos);
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

    fn command_set_position(&mut self, loc: Point2<i32>) {
        let cursor = Point2::new(loc.x as usize, loc.y as usize);
        if self.options.use_virtual_edit {
            self.text_edit.command_set_position(cursor);
        } else {
            self.text_edit.command_set_position_clamped(cursor);
        }
    }

    pub fn clear(&mut self) {
        self.text_edit.clear();
    }

    /// Attach a callback to this editor where it is invoked when the content is changed.
    ///
    /// Note:The content is extracted into string and used as a parameter to the function.
    /// This may be a costly operation when the editor has lot of text on it.
    #[cfg(feature = "callback")]
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_listeners.push(cb);
        self
    }

    #[cfg(feature = "callback")]
    pub fn add_on_change_listener<F>(&mut self, f: F)
    where
        F: Fn(String) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_listeners.push(cb);
    }

    /// Attach an callback to this editor where it is invoked when the content is changed.
    /// The callback function just notifies the parent component that uses the BaseEditor component.
    /// It will be up to the parent component to extract the content of the editor manually.
    ///
    /// This is intended to be used in a debounced or throttled functionality where the component
    /// decides when to do an expensive operation based on time and recency.
    ///
    ///
    #[cfg(feature = "callback")]
    pub fn on_change_notify<F>(mut self, f: F) -> Self
    where
        F: Fn(()) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_notify_listeners.push(cb);
        self
    }

    #[cfg(feature = "callback")]
    pub fn add_on_change_notify<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_notify_listeners.push(cb);
    }

    #[cfg(feature = "callback")]
    pub fn emit_on_change_listeners(&self) -> Vec<XMSG> {
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
}
