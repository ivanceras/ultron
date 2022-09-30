#![deny(warnings)]
use crate::util;
use css_colors::{rgba, Color, RGBA};
use sauron::{
    html::attributes, jss_ns, jss_pretty, prelude::*, wasm_bindgen::JsCast,
};
pub use ultron_core;
use ultron_core::{editor, nalgebra::Point2, Editor, Options};

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 7;
pub const CH_HEIGHT: u32 = 16;

pub enum Command {
    EditorCommand(editor::Command),
    PasteText,
    CopyText,
    CutText,
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
}

/// rename this to WebEditor
pub struct EditorWeb {
    options: Options,
    editor: Editor<Msg>,
    editor_element: Option<web_sys::Element>,
    mouse_cursor: MouseCursor,
}

impl EditorWeb {
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
        EditorWeb {
            options,
            editor,
            editor_element: None,
            mouse_cursor: MouseCursor::default(),
        }
    }
}

impl Component<Msg, ()> for EditorWeb {
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
        div(vec![class("app")], vec![self.view_editor()])
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::EditorMounted(mount_event) => {
                let mount_element: web_sys::Element =
                    mount_event.target_node.unchecked_into();
                self.editor_element = Some(mount_element);
                Effects::none()
            }
            Msg::Mouseup(client_x, client_y) => {
                let cursor = self.client_to_cursor_clamped(client_x, client_y);
                self.editor.process_command(editor::Command::SetPosition(
                    cursor.x, cursor.y,
                ));
                self.editor.set_selection_end(cursor);
                let selection = self.editor.selection();
                if let (Some(start), Some(end)) =
                    (selection.start, selection.end)
                {
                    let msgs = self.editor.process_command(
                        editor::Command::SetSelection(start, end),
                    );
                    Effects::new(msgs, vec![])
                } else {
                    Effects::none()
                }
            }
            Msg::Mousedown(client_x, client_y) => {
                if self.in_bounds(client_x as f32, client_y as f32) {
                    let cursor =
                        self.client_to_cursor_clamped(client_x, client_y);
                    self.editor.set_selection_start(cursor);
                    let msgs = self.editor.process_command(
                        editor::Command::SetPosition(cursor.x, cursor.y),
                    );
                    Effects::new(msgs, vec![]).measure()
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
                        let msgs = self.editor.process_command(
                            editor::Command::SetSelection(start, cursor),
                        );
                        Effects::new(msgs, vec![]).measure()
                    } else {
                        Effects::none()
                    }
                } else {
                    Effects::none()
                }
            }
            Msg::Keydown(ke) => self.process_keypress(&ke),
        }
    }
}

impl EditorWeb {
    #[allow(unused)]
    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }

    pub fn keyevent_to_command(ke: &web_sys::KeyboardEvent) -> Option<Command> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            log::trace!("inserting from window keydown event");
            let c = key.chars().next().expect("must be only 1 chr");
            let command = match c {
                'c' if is_ctrl => Command::CopyText,
                'x' if is_ctrl => Command::CutText,
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    Command::PasteText
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        Command::EditorCommand(editor::Command::Redo)
                    } else {
                        Command::EditorCommand(editor::Command::Undo)
                    }
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
                _ => None,
            };
            editor_command.map(Command::EditorCommand)
        }
    }

    /// make this into keypress to command
    pub fn process_keypress(
        &mut self,
        ke: &web_sys::KeyboardEvent,
    ) -> Effects<Msg, ()> {
        if let Some(command) = Self::keyevent_to_command(ke) {
            let msgs = self.process_command(command);
            Effects::new(msgs, vec![])
        } else {
            Effects::none()
        }
    }

    pub fn process_command(&mut self, command: Command) -> Vec<Msg> {
        match command {
            Command::EditorCommand(ecommand) => {
                self.editor.process_command(ecommand)
            }
            Command::PasteText => todo!(),
            Command::CopyText => todo!(),
            Command::CutText => todo!(),
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.editor.selected_text()
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.editor.cut_selected_text()
    }

    /*
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
    */

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
