use crate::util;
use css_colors::{rgba, Color, RGBA};
use sauron::{
    html::attributes, jss_ns_pretty, prelude::*, wasm_bindgen::JsCast,
    Measurements,
    wasm_bindgen_futures::JsFuture,
};
pub use ultron_core;
use ultron_core::{editor, nalgebra::Point2, Editor, Options};
use async_trait::async_trait;

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 7;
pub const CH_HEIGHT: u32 = 16;

#[derive(Debug)]
pub enum Command {
    EditorCommand(editor::Command),
    /// execute paste text
    PasteTextBlock(String),
    MergeText(String),
    /// execute copy text
    CopyText,
    /// execute cut text
    CutText,
}

impl From<editor::Command> for Command {
    fn from(ecommand: editor::Command) -> Self {
        Self::EditorCommand(ecommand)
    }
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

#[derive(Debug, Clone)]
pub enum Msg {
    EditorMounted(MountEvent),
    //TODO: Turn this into a generic keyboard event
    Keydown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    Measurements(Measurements),
}

/// rename this to WebEditor
pub struct WebEditor<XMSG> {
    options: Options,
    editor: Editor<XMSG>,
    editor_element: Option<web_sys::Element>,
    mouse_cursor: MouseCursor,
    measure: Measure,
}

#[derive(Default)]
struct Measure{
    average_dispatch: Option<f64>,
}

impl<XMSG> WebEditor<XMSG> {
    pub fn from_str(options: Options, content: &str) -> Self {
        let editor = Editor::from_str(options.clone(), content);
        WebEditor {
            options,
            editor,
            editor_element: None,
            mouse_cursor: MouseCursor::default(),
            measure: Measure::default(),
        }
    }

    pub fn add_on_change_listener<F>(&mut self, f: F)
    where
        F: Fn(String) -> XMSG + 'static,
    {
        self.editor.add_on_change_listener(f);
    }

    pub fn add_on_change_notify<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        self.editor.add_on_change_notify(f);
    }

    pub fn get_content(&self) -> String {
        self.editor.get_content()
    }
}

#[async_trait(?Send)]
impl<XMSG> Component<Msg, XMSG> for WebEditor<XMSG> {
    fn style(&self) -> String {
        let user_select = if self.options.allow_text_selection {
            "text"
        } else {
            "none"
        };
        jss_ns_pretty! {COMPONENT_NAME,
            ".": {
                position: "relative",
                font_size: px(14),
                white_space: "normal",
                user_select: user_select,
                "-webkit-user-select": user_select,
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
                user_select: user_select,
                "-webkit-user-select": user_select,
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
                background_color: "#ddd",
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
                user_select: user_select,
                "-webkit-user-select": user_select,
            },

            ".line span::selection": {
                background_color: self.selection_background().to_css(),
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
                border_color: self.cursor_border().to_css(),
                opacity: 1,
                border_style: "solid",
            },

            ".cursor_center":{
                width: percent(100),
                height: percent(100),
                background_color: self.cursor_color().to_css(),
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

    fn view(&self) -> Node<Msg> {
        div(
            [
                class(COMPONENT_NAME),
                classes_flag_namespaced(
                    COMPONENT_NAME,
                    [("occupy_container", self.options.occupy_container)],
                ),
                on_mount(|mount_event| Msg::EditorMounted(mount_event)),
                style! {
                    cursor: self.mouse_cursor.to_str(),
                },
            ],
            [
                if self.options.use_syntax_highlighter {
                    self.view_highlighted_lines(self.editor.highlighted_lines())
                } else {
                    self.plain_view()
                },
                view_if(self.options.show_status_line, self.view_status_line()),
                view_if(self.options.show_cursor, self.view_virtual_cursor()),
            ],
        )
    }

    async fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::EditorMounted(mount_event) => {
                let mount_element: web_sys::Element =
                    mount_event.target_node.unchecked_into();
                self.editor_element = Some(mount_element);
                Effects::none()
            }
            Msg::Mouseup(client_x, client_y) => {
                let cursor = self.client_to_cursor_clamped(client_x, client_y);
                self.editor.process_commands([editor::Command::SetPosition(
                    cursor.x, cursor.y,
                )]).await;
                self.editor.set_selection_end(cursor);
                let selection = self.editor.selection();
                if let (Some(start), Some(end)) =
                    (selection.start, selection.end)
                {
                    let msgs = self.editor.process_commands(
                        [editor::Command::SetSelection(start, end)],
                    ).await;
                    Effects::new(vec![], msgs)
                } else {
                    Effects::none()
                }
            }
            Msg::Mousedown(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.editor.set_selection_start(cursor);
                    let msgs = self.editor.process_commands(
                        [editor::Command::SetPosition(cursor.x, cursor.y)],
                    ).await;
                    Effects::new(vec![], msgs).measure()
                } else {
                    Effects::none()
                }
            }
            Msg::Mousemove(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.editor.set_selection_end(cursor);

                    let selection = self.editor.selection();
                    if let Some(start) = selection.start {
                        let msgs = self.editor.process_commands(
                            [editor::Command::SetSelection(start, cursor)],
                        ).await;
                        Effects::new(vec![], msgs).measure()
                    } else {
                        Effects::none()
                    }
                } else {
                    Effects::none()
                }
            }
            Msg::Keydown(ke) => self.process_keypress(&ke).await,
            Msg::Measurements(measure) => {
                self.update_measure(measure);
                Effects::none()
            }
        }
    }
}

impl<XMSG> WebEditor<XMSG> {

    fn update_measure(&mut self, measure: Measurements){
        if let Some(average_dispatch) = self.measure.average_dispatch.as_mut(){
            *average_dispatch = (*average_dispatch + measure.total_time) / 2.0;
        }else{
            self.measure.average_dispatch = Some(measure.total_time);
        }
    }
    #[allow(unused)]
    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        self.editor.get_char(x, y)
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.editor.get_position()
    }

    pub fn rehighlight(&mut self) {
        self.editor.rehighlight()
    }

    pub fn keyevent_to_command(ke: &web_sys::KeyboardEvent) -> Option<Command> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            let c = key.chars().next().expect("must be only 1 chr");
            let command = match c {
                'c' if is_ctrl => Command::CopyText,
                'x' if is_ctrl => Command::CutText,
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    Command::PasteTextBlock(String::new())
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        Command::EditorCommand(editor::Command::Redo)
                    } else {
                        Command::EditorCommand(editor::Command::Undo)
                    }
                }
                'r' if is_ctrl => {
                    Command::EditorCommand(editor::Command::Redo)
                }
                'a' if is_ctrl => {
                    Command::EditorCommand(editor::Command::SelectAll)
                }
                _ => Command::EditorCommand(editor::Command::InsertChar(c)),
            };

            Some(command)
        } else {
            let editor_command = match &*key {
                "Tab" => Some(editor::Command::IndentForward),
                "Enter" => Some(editor::Command::BreakLine),
                "Backspace" => Some(editor::Command::DeleteBack),
                "Delete" => Some(editor::Command::DeleteForward),
                "ArrowUp" => Some(editor::Command::MoveUp),
                "ArrowDown" => Some(editor::Command::MoveDown),
                "ArrowLeft" => Some(editor::Command::MoveLeft),
                "ArrowRight" => Some(editor::Command::MoveRight),
                "Home" => Some(editor::Command::MoveLeftStart),
                "End" => Some(editor::Command::MoveRightEnd),
                _ => None,
            };
            editor_command.map(Command::EditorCommand)
        }
    }

    /// make this into keypress to command
    pub async fn process_keypress(
        &mut self,
        ke: &web_sys::KeyboardEvent,
    ) -> Effects<Msg, XMSG> {
        if let Some(command) = Self::keyevent_to_command(ke) {
            let msgs = self.process_commands([command]).await;
            Effects::new(vec![], msgs).measure()
        } else {
            Effects::none()
        }
    }

    pub async fn process_commands(&mut self, commands: impl IntoIterator<Item = Command>) -> Vec<XMSG>{
        let results:Vec<bool> =
            commands.into_iter().map(|command|
                self.process_command(command)
            ).collect();
        if results.into_iter().any(|v|v){
            self.editor.content_has_changed().await
        }else{
            vec![]
        }
    }

    pub fn process_command(&mut self, command: Command) -> bool {
        log::info!("Processing command: {:?}", command);
        match command {
            Command::EditorCommand(ecommand) => {
                self.editor.process_command(ecommand)
            }
            Command::PasteTextBlock(text_block) => self
                .editor
                .process_command(editor::Command::PasteTextBlock(text_block)),
            Command::MergeText(text_block) => self
                .editor
                .process_command(editor::Command::MergeText(text_block)),
            Command::CopyText => self.copy_selected_text_to_clipboard(),
            Command::CutText => self.cut_selected_text_to_clipboard(),
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.editor.selected_text()
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.editor.cut_selected_text()
    }

    pub fn selected_text_block_mode(&self) -> Option<String> {
        self.editor.selected_text_block_mode()
    }

    pub fn cut_selected_text_block_mode(&mut self) -> Option<String> {
        self.editor.cut_selected_text_block_mode()
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.editor.set_selection(start, end);
    }

    pub fn copy_selected_text_to_clipboard(&self) -> bool{
        log::warn!("Copying text to clipboard..");
        if let Some(clipboard) = window().navigator().clipboard(){
            if let Some(selected_text) = self.selected_text(){
                let fut = JsFuture::from(clipboard.write_text(&selected_text));
                spawn_local(async move{fut.await.expect("must not error");});
                return true;
            }
        }else{
            log::error!("Clipboard is not supported");
        }
        false
    }

    pub fn cut_selected_text_to_clipboard(&mut self) -> bool{
        log::warn!("Cutting text to clipboard");
        let ret = self.copy_selected_text_to_clipboard();
        self.cut_selected_text_block_mode();
        ret
    }

    /// calculate the bounding rect of the editor using a DOM call [getBoundingClientRect](https://developer.mozilla.org/en-US/docs/Web/API/Element/getBoundingClientRect)
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

    #[allow(unused)]
    /// calculate the points relative to the editor bounding box
    pub fn relative_client(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let editor = self.editor_offset().expect("must have an editor offset");
        let x = client_x as f32 - editor.x;
        let y = client_y as f32 - editor.y;
        Point2::new(x.round() as i32, y.round() as i32)
    }

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    fn theme_background(&self) -> RGBA {
        let default = rgba(255, 255, 255, 1.0);
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .background
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn gutter_background(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .gutter
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn gutter_foreground(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn selection_background(&self) -> RGBA {
        let default = rgba(0, 0, 255, 1.0);
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .selection
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn cursor_color(&self) -> RGBA {
        rgba(0, 0, 0, 1.0)
    }

    fn cursor_border(&self) -> RGBA {
        rgba(0, 0, 0, 1.0)
    }

    /// how wide the numberline based on the character lengths of the number
    fn numberline_wide_with_padding(&self) -> usize {
        if self.options.show_line_numbers {
            self.editor.total_lines().to_string().len()
                + self.numberline_padding_wide()
        } else {
            0
        }
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
        let cursor = self.editor.get_position();
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

    /// the view for the status line
    pub fn view_status_line<Msg>(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.editor.get_position();

        div(
            [
                class_ns("status"),
                style! {
                    background_color: self.gutter_background().to_css(),
                    color: self.gutter_foreground().to_css(),
                    height: px(self.status_line_height()),
                },
            ],
            [
                text!("line: {}, col: {} ", cursor.y + 1, cursor.x + 1),
                text!("| version:{}", env!("CARGO_PKG_VERSION")),
                text!("| lines: {}", self.editor.total_lines()),
                if let Some(average_dispatch) = self.measure.average_dispatch{
                    text!("| average dispatch: {}ms", average_dispatch.round())
                }else{
                    text!("| ..")
                },
            ],
        )
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
                    style! {
                        background_color: self.gutter_background().to_css(),
                        color: self.gutter_foreground().to_css(),
                    },
                ],
                [text(line_number)],
            ),
        )
    }

    // highlighted view
    pub fn view_highlighted_lines<MSG>(
        &self,
        highlighted_lines: &[Vec<(editor::Style, String)>],
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.editor.numberline_wide());

        let code_attributes = [
            class_ns("code"),
            class_ns(&class_number_wide),
            style! {background: self.theme_background().to_css()},
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
        view_text_buffer(self.editor.text_buffer(), &self.options)
    }

    /// height of the status line which displays editor infor such as cursor location
    pub fn status_line_height(&self) -> i32 {
        30
    }
}

pub fn view_text_buffer<MSG>(
    text_buffer: &crate::TextBuffer,
    options: &Options,
) -> Node<MSG> {
    let class_ns =
        |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

    let class_number_wide =
        format!("number_wide{}", text_buffer.numberline_wide());

    let code_attributes = [class_ns("code"), class_ns(&class_number_wide)];
    let rendered_lines = text_buffer.lines().into_iter().enumerate().map(
        |(line_index, line)| {
            let line_number = line_index + 1;
            div(
                [class_ns("line")],
                [
                    view_if(
                        options.show_line_numbers,
                        span([class_ns("number")], [text(line_number)]),
                    ),
                    // Note: this is important since text node with empty
                    // content seems to cause error when finding the dom in rust
                    span([],[text(line)])
                ],
            )
        },
    );

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
