use crate::{
    util,
    Options,
    TextBuffer,
    TextEdit,
    TextHighlighter,
    CH_HEIGHT,
    CH_WIDTH,
    COMPONENT_NAME,
};
use css_colors::{
    rgba,
    Color,
    RGBA,
};
use nalgebra::Point2;
use sauron::{
    html::attributes,
    jss_ns,
    prelude::*,
    wasm_bindgen::JsCast,
    Measurements,
};
use ultron_syntaxes_themes::Style;

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
    Undo,
    Redo,
    BumpHistory,
    SelectAll,
    ClearSelection,
    SetPosition(i32, i32),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Msg {
    EditorMounted(web_sys::Node),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    SetMeasurement(Measurements),
}

///TODO: abstract more this editor
/// There should be no browser specific types
/// here, the points should be converted to cursor coordinate rather than pixels
pub struct Editor<XMSG> {
    options: Options,
    text_edit: TextEdit,
    text_highlighter: TextHighlighter,
    /// lines of highlighted ranges
    highlighted_lines: Vec<Vec<(Style, String)>>,
    measurements: Option<Measurements>,
    average_update_time: Option<f64>,
    /// Other components can listen to the an event.
    /// When the content of the text editor changes, the change listener will be emitted
    change_listeners: Vec<Callback<String, XMSG>>,
    /// a cheaper listener which doesn't need to assemble the text content
    /// of the text editor everytime
    change_notify_listeners: Vec<Callback<(), XMSG>>,
    editor_element: Option<web_sys::Element>,
    mouse_cursor: MouseCursor,
}

pub enum MouseCursor {
    Text,
    Move,
    Pointer,
    CrossHair,
}

impl Default for MouseCursor {
    fn default() -> Self {
        Self::Text
    }
}

impl MouseCursor {
    fn to_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Move => "move",
            Self::Pointer => "default",
            Self::CrossHair => "crosshair",
        }
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
            highlight_lines(&text_edit, &mut text_highlighter);

        Editor {
            options,
            text_edit,
            text_highlighter,
            highlighted_lines,
            measurements: None,
            average_update_time: None,
            change_listeners: vec![],
            change_notify_listeners: vec![],
            editor_element: None,
            mouse_cursor: MouseCursor::default(),
        }
    }

    /// rehighlight the texts
    pub fn rehighlight(&mut self) {
        self.text_highlighter.reset();
        self.highlighted_lines =
            highlight_lines(&self.text_edit, &mut self.text_highlighter);
    }

    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.text_edit.set_selection(start, end);
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

    fn theme_background(&self) -> Option<RGBA> {
        self.text_highlighter
            .active_theme()
            .settings
            .background
            .map(util::to_rgba)
    }

    pub(crate) fn cursor_color(&self) -> Option<RGBA> {
        Some(rgba(0, 0, 0, 1.0))
    }

    fn gutter_background(&self) -> Option<RGBA> {
        self.text_highlighter
            .active_theme()
            .settings
            .gutter
            .map(util::to_rgba)
    }

    fn gutter_foreground(&self) -> Option<RGBA> {
        self.text_highlighter
            .active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.text_edit.get_position()
    }

    pub fn get_content(&self) -> String {
        self.text_edit.get_content()
    }
}

impl<XMSG> Component<Msg, XMSG> for Editor<XMSG> {
    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::EditorMounted(target_node) => {
                let element: &web_sys::Element = target_node.unchecked_ref();
                self.editor_element = Some(element.clone());
                Effects::none()
            }
            Msg::Mouseup(client_x, client_y) => {
                let cursor = self.client_to_cursor_clamped(client_x, client_y);
                self.command_set_position(cursor.x, cursor.y);
                self.text_edit.set_selection_end(cursor);
                let selection = self.text_edit.selection();
                if let (Some(start), Some(end)) =
                    (selection.start, selection.end)
                {
                    self.command_set_selection(start, end);
                }
                Effects::none().measure()
            }
            Msg::Mousedown(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.text_edit.set_selection_start(cursor);
                    self.command_set_position(cursor.x, cursor.y);
                }
                Effects::none().measure()
            }
            Msg::Mousemove(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.text_edit.set_selection_end(cursor);

                    let selection = self.text_edit.selection();
                    if let Some(start) = selection.start {
                        self.command_set_selection(start, cursor);
                    }
                    Effects::none().measure()
                } else {
                    Effects::none().no_render()
                }
            }
            Msg::SetMeasurement(measurements) => {
                match self.average_update_time {
                    Some(average_update_time) => {
                        self.average_update_time = Some(
                            (average_update_time + measurements.total_time)
                                / 2.0,
                        );
                    }
                    None => {
                        self.average_update_time = Some(measurements.total_time)
                    }
                }
                self.measurements = Some(measurements);
                Effects::none().no_render()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            [
                class(COMPONENT_NAME),
                classes_flag_namespaced(
                    COMPONENT_NAME,
                    [("occupy_container", self.options.occupy_container)],
                ),
                on_mount(|mount| Msg::EditorMounted(mount.target_node)),
                style! {
                    cursor: self.mouse_cursor.to_str(),
                },
            ],
            [
                if self.options.use_syntax_highlighter {
                    self.view_highlighted_lines(
                        &self.highlighted_lines,
                        self.theme_background(),
                    )
                } else {
                    self.plain_view()
                },
                view_if(self.options.show_status_line, self.view_status_line()),
                view_if(self.options.show_cursor, self.view_virtual_cursor()),
            ],
        )
    }

    fn style(&self) -> String {
        let cursor_color = self.cursor_color().unwrap_or(rgba(0, 0, 0, 1.0));
        let border_color = rgba(0, 0, 0, 1.0);

        jss_ns! {COMPONENT_NAME,
            ".": {
                position: "relative",
                font_size: px(14),
                white_space: "normal",
                user_select: "none",
                "-webkit-user-select": "none",
            },

            ".occupy_container": {
                width: percent(100),
                height: percent(100),
            },

            "pre code":{
                white_space: "pre",
                word_spacing: "normal",
                word_break: "normal",
                word_wrap: "normal",
            },


            ".hidden_textarea_wrapper": {
                overflow: "hidden",
                position: "relative",
                width: px(300),
                height: px(0),
            },

            ".code_wrapper": {
                margin: 0,
            },

            ".code": {
                position: "relative",
                font_size: px(14),
                display: "block",
                // to make the background color extend to the longest line, otherwise only the
                // longest lines has a background-color leaving the shorter lines ugly
                min_width: "max-content",
                user_select: "none",
                "-webkit-user-select": "none",
            },

            ".line_block": {
                display: "block",
                height: px(CH_HEIGHT),
            },

            // number and line
            ".number__line": {
                display: "flex",
                height: px(CH_HEIGHT),
            },

            // numbers
            ".number": {
                flex: "none", // dont compress the numbers
                text_align: "right",
                background_color: "#002b36",
                padding_right: px(CH_WIDTH as f32 * self.numberline_padding_wide() as f32),
                height: px(CH_HEIGHT),
                display: "inline-block",
                user_select: "none",
                "-webkit-user-select": "none",
            },
            ".number_wide1 .number": {
                width: px(CH_WIDTH),
            },
            // when line number is in between: 10 - 99
            ".number_wide2 .number": {
                width: px(2 * CH_WIDTH),
            },
            // when total lines is in between 100 - 999
            ".number_wide3 .number": {
                width: px(3 * CH_WIDTH),
            },
            // when total lines is in between 1000 - 9000
            ".number_wide4 .number": {
                width: px(4 * CH_WIDTH),
            },
            // 10000 - 90000
            ".number_wide5 .number": {
                width: px(5 * CH_WIDTH),
            },

            // line content
            ".line": {
                flex: "none", // dont compress lines
                height: px(CH_HEIGHT),
                display: "block",
                user_select: "none",
                "-webkit-user-select": "none",
            },


            ".status": {
                position: "sticky",
                bottom: 0,
                display: "flex",
                flex_direction: "flex-end",
                user_select: "none",
            },

            ".virtual_cursor": {
                position: "absolute",
                width: px(CH_WIDTH),
                height: px(CH_HEIGHT),
                border_width: px(1),
                border_color: border_color.to_css(),
                opacity: 1,
                border_style: "solid",
            },

            ".cursor_center":{
                width: percent(100),
                height: percent(100),
                background_color: cursor_color.to_css(),
                opacity: percent(50),
                animation: "cursor_blink-anim 1000ms step-end infinite",
            },

            "@keyframes cursor_blink-anim": {
              "0%": {
                opacity: percent(0),
              },
              "25%": {
                opacity: percent(25)
              },
              "50%": {
                opacity: percent(100),
              },
              "75%": {
                opacity: percent(75)
              },
              "100%": {
                opacity: percent(0),
              },
            },

        }
    }
}

impl<XMSG> Editor<XMSG> {
    pub fn process_command(&mut self, command: Command) -> Effects<Msg, XMSG> {
        match command {
            //TODO: make a command indent forward and backward in text_buffer
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
                Effects::none()
            }
            Command::MoveDown => {
                self.command_move_down();
                Effects::none()
            }
            Command::MoveLeft => {
                self.command_move_left();
                Effects::none()
            }
            Command::MoveRight => {
                self.command_move_right();
                Effects::none()
            }
            Command::InsertChar(c) => self.command_insert_char(c),
            Command::ReplaceChar(c) => self.command_replace_char(c),
            Command::InsertText(text) => self.command_insert_text(&text),
            Command::Undo => self.command_undo(),
            Command::Redo => self.command_redo(),
            Command::BumpHistory => {
                self.bump_history();
                Effects::none()
            }
            Command::SelectAll => {
                self.command_select_all();
                Effects::none()
            }
            Command::ClearSelection => {
                self.clear_selection();
                Effects::none()
            }
            Command::SetPosition(x, y) => {
                self.command_set_position(x, y);
                Effects::none()
            }
        }
    }

    fn command_insert_char(&mut self, ch: char) -> Effects<Msg, XMSG> {
        self.text_edit.command_insert_char(ch);
        self.content_has_changed()
    }

    fn command_replace_char(&mut self, ch: char) -> Effects<Msg, XMSG> {
        self.text_edit.command_replace_char(ch);
        self.content_has_changed()
    }

    fn command_delete_back(&mut self) -> Effects<Msg, XMSG> {
        self.text_edit.command_delete_back();
        self.content_has_changed()
    }

    fn command_delete_forward(&mut self) -> Effects<Msg, XMSG> {
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

    fn command_break_line(&mut self) -> Effects<Msg, XMSG> {
        self.text_edit.command_break_line();
        self.content_has_changed()
    }

    #[allow(unused)]
    fn command_join_line(&mut self) -> Effects<Msg, XMSG> {
        self.text_edit.command_join_line();
        self.content_has_changed()
    }

    fn command_insert_text(&mut self, text: &str) -> Effects<Msg, XMSG> {
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

    fn command_undo(&mut self) -> Effects<Msg, XMSG> {
        self.text_edit.command_undo();
        self.content_has_changed()
    }

    fn command_redo(&mut self) -> Effects<Msg, XMSG> {
        self.text_edit.command_redo();
        self.content_has_changed()
    }

    /// call this when a command changes the text_edit content
    /// This will rehighlight the content
    /// and emit the external XMSG in the event listeners
    fn content_has_changed(&mut self) -> Effects<Msg, XMSG> {
        self.rehighlight();
        let extern_msgs = self.emit_on_change_listeners();
        if !extern_msgs.is_empty() {
            Effects::with_external(extern_msgs).measure()
        } else {
            Effects::none()
        }
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
        let mut extern_msgs: Vec<XMSG> = if !self.change_listeners.is_empty() {
            let content = self.text_edit.get_content();
            self.change_listeners
                .iter()
                .map(|listener| listener.emit(content.clone()))
                .collect()
        } else {
            vec![]
        };

        let extern_notify_msgs: Vec<XMSG> =
            if !self.change_notify_listeners.is_empty() {
                self.change_notify_listeners
                    .iter()
                    .map(|notify| notify.emit(()))
                    .collect()
            } else {
                vec![]
            };
        extern_msgs.extend(extern_notify_msgs);
        extern_msgs
    }

    pub fn bounding_rect(&self) -> Option<(Point2<f32>, Point2<f32>)> {
        if let Some(ref editor_element) = self.editor_element {
            let rect = editor_element.get_bounding_client_rect();
            let editor_x = rect.x().round() as f32;
            let editor_y = rect.y().round() as f32;
            let bottom = rect.bottom().round() as f32;
            let right = rect.right().round() as f32;
            Some((Point2::new(editor_x, editor_y), Point2::new(right, bottom)))
        } else {
            None
        }
    }

    /// check if this mouse client x and y is inside the editor bounds
    pub fn in_bounds(&self, client_x: f32, client_y: f32) -> bool {
        if let Some((start, end)) = self.bounding_rect() {
            client_x >= start.x
                && client_x <= end.x
                && client_y >= start.y
                && client_y <= end.y
        } else {
            false
        }
    }

    pub fn editor_offset(&self) -> Option<Point2<f32>> {
        if let Some((start, _end)) = self.bounding_rect() {
            Some(start)
        } else {
            None
        }
    }

    pub fn relative_client(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let editor = self.editor_offset().expect("must have an editor offset");
        let x = client_x as f32 - editor.x;
        let y = client_y as f32 - editor.y;
        Point2::new(x.round() as i32, y.round() as i32)
    }

    /// convert screen coordinate to cursor position
    pub fn client_to_cursor(
        &self,
        client_x: i32,
        client_y: i32,
    ) -> Point2<i32> {
        let numberline_wide_with_padding =
            self.numberline_wide_with_padding() as f32;
        let editor = self.editor_offset().expect("must have an editor offset");
        let col = (client_x as f32 - editor.x) / CH_WIDTH as f32
            - numberline_wide_with_padding;
        let line = (client_y as f32 - editor.y) / CH_HEIGHT as f32;
        let x = col.floor() as i32;
        let y = line.floor() as i32;
        Point2::new(x, y)
    }

    /// clamped negative cursor values due to padding in the line number
    pub fn client_to_cursor_clamped(
        &self,
        client_x: i32,
        client_y: i32,
    ) -> Point2<i32> {
        let cursor = self.client_to_cursor(client_x, client_y);
        util::clamp_to_edge(cursor)
    }

    /// convert current cursor position to client coordinate relative to the editor div
    pub fn cursor_to_client(&self) -> Point2<f32> {
        let cursor = self.text_edit.get_position();
        Point2::new(
            (cursor.x + self.numberline_wide_with_padding()) as f32
                * CH_WIDTH as f32,
            cursor.y as f32 * CH_HEIGHT as f32,
        )
    }

    fn view_virtual_cursor(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.cursor_to_client();
        div(
            [
                class_ns("virtual_cursor"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                },
            ],
            [div([class_ns("cursor_center")], [])],
        )
    }

    /// height of the status line which displays editor infor such as cursor location
    pub fn status_line_height(&self) -> i32 {
        30
    }

    /// the number of page of the editor based on the number of lines
    #[allow(unused)]
    fn pages(&self) -> i32 {
        let n_lines = self.text_edit.total_lines() as i32;
        (n_lines - 1) / self.options.page_size as i32 + 1
    }

    /// the view for the status line
    pub fn view_status_line<Msg>(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.text_edit.get_position();
        div(
            [
                class_ns("status"),
                if let Some(gutter_bg) = self.gutter_background() {
                    style! {
                        background_color: gutter_bg.to_css(),
                    }
                } else {
                    empty_attr()
                },
                if let Some(gutter_fg) = self.gutter_foreground() {
                    style! {
                        color: gutter_fg.to_css()
                    }
                } else {
                    empty_attr()
                },
                style! {height: px(self.status_line_height()) },
            ],
            [
                text!("line: {}, col: {} ", cursor.y + 1, cursor.x + 1),
                if let Some(measurements) = &self.measurements {
                    text!(
                        "| msgs: {} | patches: {} | nodes: {} | view time: {}ms | patch time: {}ms | update time: {}ms",
                        measurements.msg_count,
                        measurements.total_patches,
                        measurements.view_node_count,
                        measurements.build_view_took.round(),
                        measurements.dom_update_took.round(),
                        measurements.total_time.round(),
                    )
                } else {
                    comment("")
                },
                if let Some(average_update_time) = self.average_update_time {
                    text!("| average time: {:.2}ms", average_update_time)
                } else {
                    comment("")
                },
                text!("| version:{}", env!("CARGO_PKG_VERSION")),
                text!("| lines: {}", self.text_edit.total_lines()),
            ],
        )
    }

    fn numberline_wide(&self) -> usize {
        self.text_edit.numberline_wide()
    }

    /// how wide the numberline based on the character lengths of the number
    fn numberline_wide_with_padding(&self) -> usize {
        if self.options.show_line_numbers {
            self.text_edit.total_lines().to_string().len()
                + self.numberline_padding_wide()
        } else {
            0
        }
    }

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    fn view_line_number<MSG>(&self, line_number: usize) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        view_if(
            self.options.show_line_numbers,
            span(
                [
                    class_ns("number"),
                    if let Some(gutter_bg) = self.gutter_background() {
                        style! {
                            background_color: gutter_bg.to_css(),
                        }
                    } else {
                        empty_attr()
                    },
                    if let Some(gutter_fg) = self.gutter_foreground() {
                        style! {
                            color: gutter_fg.to_css(),
                        }
                    } else {
                        empty_attr()
                    },
                ],
                [text(line_number)],
            ),
        )
    }

    // highlighted view
    pub fn view_highlighted_lines<MSG>(
        &self,
        highlighted_lines: &[Vec<(Style, String)>],
        theme_background: Option<RGBA>,
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.numberline_wide());

        let code_attributes = [
            class_ns("code"),
            class_ns(&class_number_wide),
            if let Some(theme_background) = theme_background {
                style! {background: theme_background.to_css()}
            } else {
                empty_attr()
            },
        ];

        let rendered_lines =
            highlighted_lines
                .iter()
                .enumerate()
                .map(|(line_index, line)| {
                    div([class_ns("line")], {
                        [self.view_line_number(line_index + 1)]
                            .into_iter()
                            .chain(line.iter().map(|(style, range)| {
                                let background =
                                    util::to_rgba(style.background).to_css();
                                let foreground =
                                    util::to_rgba(style.foreground).to_css();
                                span(
                                    [style! {
                                        color: foreground,
                                        background_color: background,
                                    }],
                                    [text(range)],
                                )
                            }))
                            .collect::<Vec<_>>()
                    })
                });

        if self.options.use_for_ssg {
            // using div works well when select-copying for both chrome and firefox
            // this is ideal for statis site generation highlighting
            div(code_attributes, rendered_lines)
        } else {
            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                [class_ns("code_wrapper")],
                [code(code_attributes, rendered_lines)],
            )
        }
    }

    pub fn plain_view<MSG>(&self) -> Node<MSG> {
        text_buffer_view(self.text_edit.text_buffer(), &self.options)
    }
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

pub fn text_buffer_view<MSG>(
    text_buffer: &TextBuffer,
    options: &Options,
) -> Node<MSG> {
    let class_ns =
        |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

    let class_number_wide =
        format!("number_wide{}", text_buffer.numberline_wide());

    let code_attributes = [class_ns("code"), class_ns(&class_number_wide)];
    let rendered_lines = text_buffer
        .lines()
        .into_iter()
        .map(|line| div([class_ns("line")], [text(line)]));

    if options.use_for_ssg {
        // using div works well when select-copying for both chrome and firefox
        // this is ideal for static site generation highlighting
        div(code_attributes, rendered_lines)
    } else {
        // using <pre><code> works well when copying in chrome
        // but in firefox, it creates a double line when select-copying the text
        // whe need to use <pre><code> in order for typing whitespace works.
        pre(
            [class_ns("code_wrapper")],
            [code(code_attributes, rendered_lines)],
        )
    }
}
