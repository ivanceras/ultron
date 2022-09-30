#![deny(warnings)]
use css_colors::{rgba, Color, RGBA};
use sauron::{
    html::attributes, jss_ns, jss_pretty, prelude::*, wasm_bindgen::JsCast,
    web_sys::HtmlDocument, Window,
};
pub use ultron_core;
use ultron_core::{editor, nalgebra::Point2, Command, Editor, Options};

mod util;

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 7;
pub const CH_HEIGHT: u32 = 16;

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
    WindowScrolled((i32, i32)),
    WindowResized(i32, i32),
    Keydown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    TextareaKeydown(web_sys::KeyboardEvent),
    TextareaInput(String),
    Paste(String),
    TextareaMounted(web_sys::Node),
    NoOp,
}

/// rename this to WebEditor
pub struct App {
    options: Options,
    editor: Editor<Msg>,
    hidden_textarea: Option<web_sys::HtmlTextAreaElement>,
    composed_key: Option<char>,
    last_char_count: Option<usize>,
    editor_element: Option<web_sys::Element>,
    measurements: Option<Measurements>,
    average_update_time: Option<f64>,
    mouse_cursor: MouseCursor,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../test_data/hello.rs");
        let content = include_str!("../test_data/long.rs");
        //let content = include_str!("../test_data/svgbob.md");
        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            ..Default::default()
        };
        let editor = Editor::from_str(options.clone(), content);
        App {
            options,
            editor,
            hidden_textarea: None,
            composed_key: None,
            last_char_count: None,
            editor_element: None,
            measurements: None,
            average_update_time: None,
            mouse_cursor: MouseCursor::default(),
        }
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::batch([
            Window::on_resize(Msg::WindowResized),
            Window::add_event_listeners(vec![
                on_scroll(Msg::WindowScrolled),
                on_mousemove(|me| Msg::Mousemove(me.client_x(), me.client_y())),
                on_mousedown(|me| Msg::Mousedown(me.client_x(), me.client_y())),
                on_mouseup(|me| Msg::Mouseup(me.client_x(), me.client_y())),
                on_keydown(|ke| {
                    ke.prevent_default();
                    ke.stop_propagation();
                    Msg::Keydown(ke)
                }),
            ]),
        ])
    }

    fn style(&self) -> String {
        let cursor_color = rgba(0, 0, 0, 1.0);
        let border_color = rgba(0, 0, 0, 1.0);

        let editor_css = jss_ns! {COMPONENT_NAME,
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

        };

        let lib_css = jss_pretty! {
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".hidden_textarea": {
                resize: "none",
                height: 0,
                position: "absolute",
                padding: 0,
                width: px(300),
                height: px(0),
                border:format!("{} solid black",px(1)),
                bottom: units::em(-1),
                outline: "none",
            },
        };

        [lib_css, editor_css].join("\n")
    }

    fn view(&self) -> Node<Msg> {
        div(
            vec![class("app")],
            vec![
                self.view_editor(),
                //self.view_hidden_textarea(),
            ],
        )
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::WindowScrolled((_scroll_top, _scroll_left)) => Cmd::none(),
            Msg::WindowResized(_width, _height) => Cmd::none(),
            Msg::Mouseup(client_x, client_y) => {
                let cursor = self.client_to_cursor_clamped(client_x, client_y);
                self.editor
                    .process_command(Command::SetPosition(cursor.x, cursor.y));
                self.editor.set_selection_end(cursor);
                let selection = self.editor.selection();
                if let (Some(start), Some(end)) =
                    (selection.start, selection.end)
                {
                    let msgs = self
                        .editor
                        .process_command(Command::SetSelection(start, end));
                    Cmd::from(Effects::new(msgs, vec![]))
                } else {
                    Cmd::none()
                }
            }
            Msg::EditorMounted(mount_event) => {
                let mount_element: web_sys::Element =
                    mount_event.target_node.unchecked_into();
                self.editor_element = Some(mount_element);
                Cmd::none()
            }
            Msg::Mousedown(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.editor.set_selection_start(cursor);
                    let msgs = self.editor.process_command(
                        Command::SetPosition(cursor.x, cursor.y),
                    );
                    Cmd::from(Effects::new(msgs, vec![])).measure()
                } else {
                    Cmd::none()
                }
            }
            Msg::Mousemove(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.editor.set_selection_end(cursor);

                    let selection = self.editor.selection();
                    if let Some(start) = selection.start {
                        let msgs = self.editor.process_command(
                            Command::SetSelection(start, cursor),
                        );
                        Cmd::from(Effects::new(msgs, vec![])).measure()
                    } else {
                        Cmd::none()
                    }
                } else {
                    Cmd::none()
                }
            }
            Msg::Keydown(ke) => self.process_keypress(&ke),
            Msg::TextareaKeydown(ke) => self.process_keypress(&ke),
            Msg::TextareaMounted(target_node) => {
                self.hidden_textarea = Some(target_node.unchecked_into());
                self.refocus_hidden_textarea();
                Cmd::none()
            }
            Msg::TextareaInput(input) => {
                let char_count = input.chars().count();
                // for chrome:
                // detect if the typed in character was a composed and becomes 1 unicode character
                let char_count_decreased =
                    if let Some(last_char_count) = self.last_char_count {
                        last_char_count > 1
                    } else {
                        false
                    };
                // firefox doesn't register compose key strokes as input
                // if there were 1 char then it was cleared
                let was_cleared = self.last_char_count == Some(0);

                let mut msgs = vec![];
                if char_count == 1 && (was_cleared || char_count_decreased) {
                    self.clear_hidden_textarea();
                    log::trace!("in textarea input char_count == 1..");
                    let c = input.chars().next().expect("must be only 1 chr");
                    self.composed_key = Some(c);
                    let more_msgs = if c == '\n' {
                        self.editor.process_command(Command::BreakLine)
                    } else {
                        self.editor.process_command(Command::InsertChar(c))
                    };
                    msgs.extend(more_msgs);
                } else {
                    log::trace!("char is not inserted becase char_count: {}, was_cleared: {}, char_count_decreased: {}", char_count, was_cleared, char_count_decreased);
                }
                self.last_char_count = Some(char_count);
                log::trace!("extern messages");
                Cmd::from(Effects::new(msgs, vec![])).measure()
            }

            Msg::Paste(text_content) => {
                let msgs = self
                    .editor
                    .process_command(Command::InsertText(text_content));
                Cmd::from(Effects::new(msgs, vec![]))
            }
            Msg::NoOp => Cmd::none().no_render(),
        }
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        log::info!("measurements: {:?}", measurements);
        Cmd::none()
    }
}

impl App {
    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }
    #[allow(unused)]
    fn view_hidden_textarea(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.cursor_to_client();
        div(
            [
                class_ns("hidden_textarea_wrapper"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                    z_index: 99,
                },
            ],
            [textarea(
                [
                    class_ns("hidden_textarea"),
                    on_mount(|mount| Msg::TextareaMounted(mount.target_node)),
                    #[cfg(web_sys_unstable_apis)]
                    on_paste(|ce| {
                        let pasted_text = ce
                            .clipboard_data()
                            .expect("must have data transfer")
                            .get_data("text/plain")
                            .expect("must be text data");
                        log::trace!(
                            "paste triggered from textarea: {}",
                            pasted_text
                        );
                        Msg::Paste(pasted_text)
                    }),
                    // for listening to CTRL+C, CTRL+V, CTRL+X
                    on_keydown(Msg::TextareaKeydown),
                    focus(true),
                    autofocus(true),
                    attr("autocorrect", "off"),
                    autocapitalize("none"),
                    autocomplete("off"),
                    spellcheck("off"),
                    // for processing unicode characters typed via: CTRL+U<unicode number> (linux),
                    on_input(|input| Msg::TextareaInput(input.value)),
                ],
                [],
            )],
        )
    }

    /// make this into keypress to command
    fn process_keypress(
        &mut self,
        ke: &web_sys::KeyboardEvent,
    ) -> Cmd<Self, Msg> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            log::trace!("inserting from window keydown event");
            let c = key.chars().next().expect("must be only 1 chr");
            match c {
                'c' if is_ctrl => {
                    self.command_copy();
                    Cmd::none()
                }
                'x' if is_ctrl => {
                    self.command_cut();
                    Cmd::none()
                }
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    self.clear_hidden_textarea();
                    Cmd::none()
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        self.editor.process_command(Command::Redo);
                    } else {
                        self.editor.process_command(Command::Undo);
                    }
                    Cmd::none()
                }
                'a' if is_ctrl => {
                    self.editor.process_command(Command::SelectAll);
                    Cmd::none()
                }
                _ => {
                    self.editor.process_command(Command::InsertChar(c));
                    Cmd::none()
                }
            }
        } else {
            let command = match &*key {
                "Tab" => Some(Command::IndentForward),
                "Enter" => Some(Command::BreakLine),
                "Backspace" => Some(Command::DeleteBack),
                "Delete" => Some(Command::DeleteForward),
                "ArrowUp" => Some(Command::MoveUp),
                "ArrowDown" => Some(Command::MoveDown),
                "ArrowLeft" => Some(Command::MoveLeft),
                "ArrowRight" => Some(Command::MoveRight),
                _ => None,
            };
            if let Some(command) = command {
                let msgs = self.editor.process_command(command);
                Cmd::from(Effects::new(msgs, vec![]))
            } else {
                Cmd::none()
            }
        }
    }

    fn clear_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.set_value("");
        } else {
            panic!("there should always be hidden textarea");
        }
    }

    fn refocus_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.focus().expect("must focus the textarea");
        }
    }

    /// set the content of the textarea to selection
    ///
    /// Note: This is necessary for webkit2.
    /// webkit2 doesn't seem to allow to fire the setting of textarea value, select and copy
    /// in the same animation frame.
    #[allow(unused)]
    fn set_hidden_textarea_with_selection(&self) {
        if let Some(selected_text) = self.editor.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
            }
        }
    }

    /// this is for newer browsers
    /// This doesn't work on webkit2
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn copy_to_clipboard(&self) -> bool {
        if let Some(selected_text) = self.editor.selected_text() {
            let navigator = sauron::window().navigator();
            if let Some(clipboard) = navigator.clipboard() {
                let _ = clipboard.write_text(&selected_text);
                return true;
            } else {
                log::warn!("no navigator clipboard");
            }
        }
        false
    }

    #[cfg(not(feature = "with-navigator-clipboard"))]
    fn copy_to_clipboard(&self) -> bool {
        false
    }

    /// execute copy on the selected textarea
    /// this works even on older browser
    fn textarea_exec_copy(&self) -> bool {
        if let Some(selected_text) = self.editor.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
                let html_document: HtmlDocument =
                    sauron::document().unchecked_into();
                if let Ok(ret) = html_document.exec_command("copy") {
                    hidden_textarea.set_value("");
                    log::trace!("exec_copy ret: {}", ret);
                    return ret;
                }
            }
        }
        false
    }

    /// returns true if the command succeeded
    fn textarea_exec_cut(&mut self) -> bool {
        if let Some(selected_text) = self.editor.cut_selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                log::trace!("setting the value to textarea: {}", selected_text);
                hidden_textarea.set_value(&selected_text);

                hidden_textarea.select();
                let html_document: HtmlDocument =
                    sauron::document().unchecked_into();
                if let Ok(ret) = html_document.exec_command("cut") {
                    hidden_textarea.set_value("");
                    return ret;
                }
            }
        }
        false
    }

    /// calls on 2 ways to copy
    /// either 1 should work
    /// returns true if it succeded
    fn command_copy(&self) {
        if self.copy_to_clipboard() {
            // do nothing
        } else {
            self.textarea_exec_copy();
        }
    }

    /// try exec_cut, try cut to clipboard if the first fails
    /// This shouldn't execute both since cut is destructive.
    /// Returns true if it succeded
    fn command_cut(&mut self) {
        if self.cut_to_clipboard() {
            // nothing
        } else {
            self.textarea_exec_cut();
        }
    }

    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn cut_to_clipboard(&mut self) -> bool {
        if let Some(selected_text) = self.editor.cut_selected_text() {
            let navigator = sauron::window().navigator();
            if let Some(clipboard) = navigator.clipboard() {
                let _ = clipboard.write_text(&selected_text);
                return true;
            } else {
                log::warn!("no navigator clipboard");
            }
        }
        false
    }

    #[cfg(not(feature = "with-navigator-clipboard"))]
    fn cut_to_clipboard(&mut self) -> bool {
        false
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

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    fn theme_background(&self) -> Option<RGBA> {
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .background
            .map(util::to_rgba)
    }

    fn gutter_background(&self) -> Option<RGBA> {
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .gutter
            .map(util::to_rgba)
    }

    fn gutter_foreground(&self) -> Option<RGBA> {
        self.editor
            .text_highlighter()
            .active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
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
                text!("| lines: {}", self.editor.total_lines()),
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
        highlighted_lines: &[Vec<(editor::Style, String)>],
        theme_background: Option<RGBA>,
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.editor.numberline_wide());

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

    /*
    pub fn plain_view<MSG>(&self) -> Node<MSG> {
        text_buffer_view(self.text_edit.text_buffer(), &self.options)
    }
    */

    fn view_editor(&self) -> Node<Msg> {
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
                    self.view_highlighted_lines(
                        self.editor.highlighted_lines(),
                        self.theme_background(),
                    )
                } else {
                    //self.plain_view()
                    span([], [])
                },
                view_if(self.options.show_status_line, self.view_status_line()),
                view_if(self.options.show_cursor, self.view_virtual_cursor()),
            ],
        )
    }

    /// height of the status line which displays editor infor such as cursor location
    pub fn status_line_height(&self) -> i32 {
        30
    }
}

/*
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
*/

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    log::trace!("starting ultron..");
    console_error_panic_hook::set_once();
    let app_container = sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(App::new(), &app_container);
}
