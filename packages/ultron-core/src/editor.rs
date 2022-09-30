pub use crate::Selection;
use crate::TextEdit;
use nalgebra::Point2;
use std::rc::Rc;
pub use ultron_syntaxes_themes::{Style, TextHighlighter};

#[derive(Clone, Debug)]
pub struct Options {
    /// block mode is when the selection is rectangular
    pub use_block_mode: bool,
    /// allow the click outside of the bounds of the text content editor
    pub use_virtual_edit: bool,
    /// allow the editor to show or hide pages for optimization
    /// Note: set this to false when using the editor as a headless buffer
    pub use_paging_optimization: bool,
    pub show_line_numbers: bool,
    pub show_status_line: bool,
    pub show_cursor: bool,
    /// use spans instead of div when rendering ranges
    /// and characters
    /// this is used when doing a static site rendering
    pub use_spans: bool,
    /// when used for ssg, whitespace will be rendered as &nbsp;
    pub use_for_ssg: bool,
    /// apply background on the characters from syntax highlighter
    pub use_background: bool,
    pub theme_name: Option<String>,
    pub syntax_token: String,
    /// whether or not the editor occupy the container element
    /// false means the editor only expands to the number of lines in the code
    pub occupy_container: bool,
    /// number of lines in a page, when paging up and down
    pub page_size: usize,
    /// a flag to use syntax highlighting or not
    pub use_syntax_highlighter: bool,
    /// a flag to do replace mode when there is no characters to the right
    /// and switch to insert mode when there is characters to the right
    pub use_smart_replace_insert: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            use_block_mode: false,
            use_virtual_edit: false,
            use_paging_optimization: true,
            show_line_numbers: true,
            show_status_line: true,
            show_cursor: true,
            use_spans: true,
            use_for_ssg: false,
            use_background: true,
            theme_name: None,
            syntax_token: "txt".to_string(),
            occupy_container: true,
            page_size: 20,
            use_syntax_highlighter: true,
            use_smart_replace_insert: false,
        }
    }
}

pub enum Command {
    IndentForward,
    IndentBackward,
    BreakLine,
    DeleteBack,
    DeleteForward,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    InsertChar(char),
    ReplaceChar(char),
    InsertText(String),
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

#[derive(Clone, PartialEq, Debug)]
pub enum Msg {}

pub struct Callback<IN, OUT> {
    func: Rc<dyn Fn(IN) -> OUT>,
}

impl<IN, F, OUT> From<F> for Callback<IN, OUT>
where
    F: Fn(IN) -> OUT + 'static,
{
    fn from(func: F) -> Self {
        Self {
            func: Rc::new(func),
        }
    }
}

impl<IN, OUT> Callback<IN, OUT> {
    /// This method calls the actual callback.
    pub fn emit(&self, input: IN) -> OUT {
        (self.func)(input)
    }
}

///TODO: abstract more this editor
/// There should be no browser specific types
/// here, the points should be converted to cursor coordinate rather than pixels
///
/// This should be akin to headless editor
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
        }
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
    pub fn process_command(&mut self, command: Command) -> Vec<XMSG> {
        match command {
            Command::IndentForward => {
                let indent = "    ";
                self.command_insert_text(indent)
            }
            Command::IndentBackward => {
                todo!()
            }
            Command::BreakLine => self.command_break_line(),
            Command::DeleteBack => self.command_delete_back(),
            Command::DeleteForward => self.command_delete_forward(),
            Command::MoveUp => {
                self.command_move_up();
                vec![]
            }
            Command::MoveDown => {
                self.command_move_down();
                vec![]
            }
            Command::MoveLeft => {
                self.command_move_left();
                vec![]
            }
            Command::MoveRight => {
                self.command_move_right();
                vec![]
            }
            Command::InsertChar(c) => self.command_insert_char(c),
            Command::ReplaceChar(c) => self.command_replace_char(c),
            Command::InsertText(text) => self.command_insert_text(&text),
            Command::SetContent(content) => self.command_set_content(&content),
            Command::Undo => self.command_undo(),
            Command::Redo => self.command_redo(),
            Command::BumpHistory => {
                self.bump_history();
                vec![]
            }
            Command::SetSelection(start, end) => {
                self.command_set_selection(start, end);
                vec![]
            }
            Command::SelectAll => {
                self.command_select_all();
                vec![]
            }
            Command::ClearSelection => {
                self.clear_selection();
                vec![]
            }
            Command::SetPosition(x, y) => {
                self.command_set_position(x, y);
                vec![]
            }
        }
    }

    /// recreate the text edit, this also clears the history
    pub fn command_set_content(&mut self, content: &str) -> Vec<XMSG> {
        self.text_edit = TextEdit::from_str(content);
        self.content_has_changed()
    }

    fn command_insert_char(&mut self, ch: char) -> Vec<XMSG> {
        self.text_edit.command_insert_char(ch);
        self.content_has_changed()
    }

    fn command_replace_char(&mut self, ch: char) -> Vec<XMSG> {
        self.text_edit.command_replace_char(ch);
        self.content_has_changed()
    }

    fn command_delete_back(&mut self) -> Vec<XMSG> {
        self.text_edit.command_delete_back();
        self.content_has_changed()
    }

    fn command_delete_forward(&mut self) -> Vec<XMSG> {
        let _ch = self.text_edit.command_delete_forward();
        self.content_has_changed()
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

    fn command_break_line(&mut self) -> Vec<XMSG> {
        self.text_edit.command_break_line();
        self.content_has_changed()
    }

    #[allow(unused)]
    fn command_join_line(&mut self) -> Vec<XMSG> {
        self.text_edit.command_join_line();
        self.content_has_changed()
    }

    fn command_insert_text(&mut self, text: &str) -> Vec<XMSG> {
        self.text_edit.command_insert_text(text);
        self.content_has_changed()
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

    fn command_set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.set_selection(start, end)
    }

    fn command_select_all(&mut self) {
        let start = Point2::new(0, 0);
        let max = self.text_edit.max_position();
        let end = Point2::new(max.x as i32, max.y as i32);
        self.set_selection(start, end);
    }

    /// Make a history separator for the undo/redo
    /// This is used for breaking undo action list
    fn bump_history(&mut self) {
        self.text_edit.bump_history();
    }

    fn command_undo(&mut self) -> Vec<XMSG> {
        self.text_edit.command_undo();
        self.content_has_changed()
    }

    fn command_redo(&mut self) -> Vec<XMSG> {
        self.text_edit.command_redo();
        self.content_has_changed()
    }

    /// call this when a command changes the text_edit content
    /// This will rehighlight the content
    /// and emit the external XMSG in the event listeners
    fn content_has_changed(&mut self) -> Vec<XMSG> {
        self.rehighlight();
        self.emit_on_change_listeners()
    }

    /// clear the text selection
    fn clear_selection(&mut self) {
        self.text_edit.clear_selection()
    }

    pub fn selected_text(&self) -> Option<String> {
        self.text_edit.selected_text()
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.text_edit.cut_selected_text()
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
