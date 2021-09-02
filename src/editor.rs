use crate::util;
use css_colors::rgba;
use css_colors::Color;
use css_colors::RGBA;
use history::Recorded;
use sauron::jss::jss_ns;
use sauron::prelude::*;
use sauron::Measurements;
use syntect::highlighting::Theme;
pub use text_buffer::TextBuffer;

pub const CH_WIDTH: u32 = 8;
pub const CH_HEIGHT: u32 = 16;

pub(crate) mod action;
mod history;
mod text_buffer;

#[derive(Clone, PartialEq)]
pub enum Msg {
    KeyDown(web_sys::KeyboardEvent),
    MoveCursor(usize, usize),
    MoveCursorToLine(usize),
    StartSelection(usize, usize),
    EndSelection(usize, usize),
    StopSelection,
    ToSelection(usize, usize),
    Paste(String),
    CopiedSelected,
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    SetMeasurement(Measurements),
    Scrolled((i32, i32)),
}

pub const COMPONENT_NAME: &str = "ultron";

pub struct Editor {
    text_buffer: TextBuffer,
    use_block_cursor: bool,
    /// number of lines in a page, when paging up and down
    page_size: usize,
    /// for undo and redo
    recorded: Recorded,
    pub measurements: Option<Measurements>,
    scroll_top: f32,
    scroll_left: f32,
}

impl Component<Msg, ()> for Editor {
    /// returns bool indicating whether the view should be updated or not
    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::Scrolled((scroll_top, scroll_left)) => {
                self.scroll_top = scroll_top as f32;
                self.scroll_left = scroll_left as f32;
                Effects::none()
            }
            Msg::Mouseup(_client_x, _client_y) => Effects::none(),
            Msg::Mousedown(client_x, client_y) => {
                let (x, y) = self.calculate_position(client_x, client_y);
                self.text_buffer.set_position(x, y);
                Effects::none()
            }
            Msg::Mousemove(_client_x, _client_y) => Effects::none(),
            Msg::Paste(_text_content) => Effects::none(),
            Msg::CopiedSelected => Effects::none(),
            Msg::MoveCursor(_line, _col) => Effects::none(),
            Msg::MoveCursorToLine(_line) => Effects::none(),
            Msg::StartSelection(_line, _col) => Effects::none(),
            Msg::ToSelection(_line, _col) => Effects::none(),
            Msg::EndSelection(_line, _col) => Effects::none(),
            Msg::StopSelection => Effects::none(),
            Msg::SetMeasurement(measurements) => {
                self.measurements = Some(measurements);
                Effects::none()
            }
            Msg::KeyDown(ke) => {
                let key = ke.key();
                if key.chars().count() == 1 {
                    let c = key.chars().next().expect("must be only 1 chr");
                    self.text_buffer.command_insert_char(c);
                }
                Effects::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            vec![class(COMPONENT_NAME), on_scroll(Msg::Scrolled)],
            vec![self.text_buffer.view()],
        )
    }

    fn style(&self) -> String {
        let selection_bg = self
            .selection_background()
            .unwrap_or(rgba(100, 100, 100, 0.5));

        let cursor_color = self.cursor_color().unwrap_or(rgba(255, 0, 0, 1.0));
        let theme_background = self.theme_background().unwrap_or(rgba(0, 0, 255, 1.0));

        jss_ns! {COMPONENT_NAME,
            ".": {
                user_select: "none",
                "-webkit-user-select": "none",
                position: "relative",
                font_size: px(14),
                cursor: "text",
                width: percent(100),
                height: percent(100),
                overflow: "auto",
            },

            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".paste_area": {
                resize: "none",
                height: 0,
                position: "sticky",
                top: 0,
                left: 0,
                padding: 0,
                border:0,
            },

            ".code": {
                position: "relative",
                background: theme_background.to_css(),
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
                background_color: "cyan",
                padding_right: px(CH_WIDTH * self.text_buffer.numberline_padding_wide() as u32),
                height: px(CH_HEIGHT),
            },
            ".number_wide1 .number": {
                width: px(1 * CH_WIDTH),
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
                display: "flex",
                flex: "none", // dont compress lines
                height: px(CH_HEIGHT),
                overflow: "hidden",
            },

            ".filler": {
                width: percent(100),
            },

            ".line_focused": {
            },

            ".range": {
                display: "flex",
                flex: "none",
                height: px(CH_HEIGHT),
                overflow: "hidden",
            },

            ".line .ch": {
                width: px(CH_WIDTH),
                height: px(CH_HEIGHT),
                font_family: "monospace",
                font_stretch: "ultra-condensed",
                font_variant_numeric: "slashed-zero",
                font_kerning: "none",
                font_size_adjust: "none",
                font_optical_sizing: "none",
                position: "relative",
                overflow: "hidden",
                align_items: "center",
            },

            ".ch.selected": {
                background_color:selection_bg.to_css(),
            },

            ".ch .cursor": {
               position: "absolute",
               left: 0,
               width : px(CH_WIDTH),
               height: px(CH_HEIGHT),
               background_color: cursor_color.to_css(),
               display: "inline",
               animation: "cursor_blink-anim 1000ms step-end infinite",
            },

            ".ch.wide2 .cursor": {
                width: px(2 * CH_WIDTH),
            },
            ".ch.wide3 .cursor": {
                width: px(3 * CH_WIDTH),
            },

            // i-beam cursor
            ".thin_cursor .cursor": {
                width: px(2),
            },

            ".block_cursor .cursor": {
                width: px(CH_WIDTH),
            },

            ".line .ch.wide2": {
                width: px(2 * CH_WIDTH),
                font_size: px(12),
            },

            ".line .ch.wide3": {
                width: px(3 * CH_WIDTH),
                font_size: px(12),
            },


            ".status": {
                background_color: "blue",
                position: "sticky",
                bottom: 0,
                display: "flex",
                flex_direction: "flex-end",
            },

            "@keyframes cursor_blink-anim": {
              "50%": {
                background_color: "transparent",
                border_color: "transparent",
              }
            },

        }
    }
}

impl Editor {
    pub fn from_str(content: &str) -> Self {
        let editor = Editor {
            text_buffer: TextBuffer::from_str(content),
            use_block_cursor: true,
            page_size: 10,
            recorded: Recorded::new(),
            measurements: None,
            scroll_top: 0.0,
            scroll_left: 0.0,
        };
        editor
    }

    fn calculate_position(&self, client_x: i32, client_y: i32) -> (usize, usize) {
        let numberline_wide = self.text_buffer.get_numberline_wide() as f32;
        let col = (client_x as f32 + self.scroll_left) / CH_WIDTH as f32 - numberline_wide;
        let line = (client_y as f32 + self.scroll_top) / CH_HEIGHT as f32 - 1.0;
        let x = col.round() as usize;
        let y = line.round() as usize;
        log::trace!(
            "client_x_y:({},{}) col_line: ({},{}), x_y: ({},{})",
            client_x,
            client_y,
            col,
            line,
            x,
            y
        );
        (x, y)
    }

    fn active_theme(&self) -> &Theme {
        &self.text_buffer.active_theme()
    }

    fn theme_background(&self) -> Option<RGBA> {
        self.active_theme().settings.background.map(util::to_rgba)
    }

    fn gutter_background(&self) -> Option<RGBA> {
        self.active_theme().settings.gutter.map(util::to_rgba)
    }

    fn gutter_foreground(&self) -> Option<RGBA> {
        self.active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
    }

    #[allow(unused)]
    fn accent_color(&self) -> Option<RGBA> {
        self.active_theme().settings.accent.map(util::to_rgba)
    }

    fn selection_background(&self) -> Option<RGBA> {
        self.active_theme().settings.selection.map(util::to_rgba)
    }

    fn cursor_color(&self) -> Option<RGBA> {
        self.active_theme().settings.caret.map(util::to_rgba)
    }
}