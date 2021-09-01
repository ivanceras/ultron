use history::Recorded;
use sauron::html::attributes::class_namespaced;
use sauron::html::attributes::classes_flag_namespaced;
use sauron::jss;
use sauron::jss::jss_ns;
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use sauron::web_sys::HtmlTextAreaElement;
use sauron::Measurements;
use std::iter::FromIterator;
use syntect::easy::HighlightLines;
use syntect::highlighting::Color;
use syntect::highlighting::Theme;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use text_buffer::Movement;
use text_buffer::TextBuffer;
use unicode_width::UnicodeWidthChar;

pub const CH_WIDTH: u32 = 8;
pub const CH_HEIGHT: u32 = 16;

pub(crate) mod action;
mod history;
mod text_buffer;
mod text_highlight;

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

const COMPONENT_NAME: &str = "ultron";

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
            Msg::Scrolled((scroll_top, scroll_left)) => {}
            Msg::Mouseup(client_x, client_y) => {}
            Msg::Mousedown(client_x, client_y) => {}
            Msg::Mousemove(client_x, client_y) => {}
            Msg::Paste(text_content) => {}
            Msg::CopiedSelected => {}
            Msg::MoveCursor(line, col) => {}
            Msg::MoveCursorToLine(line) => {}
            Msg::StartSelection(line, col) => {}
            Msg::ToSelection(line, col) => {}
            Msg::EndSelection(line, col) => {}
            Msg::StopSelection => {}
            Msg::SetMeasurement(measurements) => {
                self.measurements = Some(measurements);
            }
            Msg::KeyDown(ke) => {}
        }
        Effects::none()
    }

    fn view(&self) -> Node<Msg> {
        div(vec![], vec![text("hello")])
    }
}

impl Editor {
    pub fn from_str(content: &str) -> Self {
        let mut editor = Editor {
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

    fn active_theme(&self) -> &Theme {
        &self.text_buffer.text_highlight.active_theme()
    }

    fn theme_background(&self) -> Option<Color> {
        self.active_theme().settings.background
    }

    fn gutter_background(&self) -> Option<Color> {
        self.active_theme().settings.gutter
    }

    fn gutter_foreground(&self) -> Option<Color> {
        self.active_theme().settings.gutter_foreground
    }

    #[allow(unused)]
    fn accent_color(&self) -> Option<Color> {
        self.active_theme().settings.accent
    }

    fn selection_background(&self) -> Option<Color> {
        self.active_theme().settings.selection
    }

    fn cursor_color(&self) -> Option<Color> {
        self.active_theme().settings.caret
    }

    fn convert_rgba(c: Color) -> String {
        format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 * 255.0)
    }

    fn style(&self) -> String {
        let selection_bg = if let Some(selection_bg) = self.selection_background() {
            selection_bg
        } else {
            Color {
                r: 100,
                g: 100,
                b: 100,
                a: 100,
            }
        };

        let cursor_color = if let Some(cursor_color) = self.cursor_color() {
            cursor_color
        } else {
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        };

        jss_ns! {COMPONENT_NAME,
            ".": {
                "user-select": "none",
                "-webkit-user-select": "none",
                "position": "relative",
                "font-size": px(14),
                "cursor": "text",
                "overflow": "auto",
            },

            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".paste_area": {
                "resize": "none",
                //"width": 0, //if width is 0, it will not work in chrome
                "height": 0,
                "position": "sticky",
                "top": 0,
                "left": 0,
                "padding": 0,
                "border":0,
            },

            ".code": {
                "position": "relative",
            },

            ".line_block": {
                "display": "block",
                "height": px(CH_HEIGHT),
            },

            // number and line
            ".number__line": {
                "display": "flex",
                "height": px(CH_HEIGHT),
            },

            // numbers
            ".number": {
                "flex": "none", // dont compress the numbers
                "text-align": "right",
                "background-color": "cyan",
                "padding-right": ex(1),
                "height": px(CH_HEIGHT),
            },
            ".number_wide1 .number": {
                "width": px(1 * CH_WIDTH),
            },
            // when line number is in between: 10 - 99
            ".number_wide2 .number": {
                "width": px(2 * CH_WIDTH),
            },
            // when total lines is in between 100 - 999
            ".number_wide3 .number": {
                "width": px(3 * CH_WIDTH),
            },
            // when total lines is in between 1000 - 9000
            // we don't support beyond this
            ".number_wide4 .number": {
                "width": px(4 * CH_WIDTH),
            },

            // line content
            ".line": {
                "display": "flex",
                "flex": "none", // dont compress lines
                "height": px(CH_HEIGHT),
                "overflow": "hidden",
            },

            ".filler": {
                //"background-color": "#eee",
                "width": percent(100),
            },

            ".line_focused": {
                "background-color": "pink",
            },

            ".range": {
                "display": "flex",
                "flex": "none",
                "height": px(CH_HEIGHT),
                "overflow": "hidden",
            },

            ".line .ch": {
                "width": px(CH_WIDTH),
                "height": px(CH_HEIGHT),
                "font-family": "monospace",
                "font-stretch": "ultra-condensed",
                "font-variant-numeric": "slashed-zero",
                "font-kerning": "none",
                "font-size-adjust": "none",
                "font-optical-sizing": "none",
                "position": "relative",
                "overflow": "hidden",
                "align-items": "center",
            },

            ".ch.selected": {
                "background-color": Self::convert_rgba(selection_bg),
            },

            ".ch .cursor": {
               "position": "absolute",
               "left": 0,
               "width" : px(CH_WIDTH),
               "height": px(CH_HEIGHT),
               "background-color": Self::convert_rgba(cursor_color),
               //"border": "1px solid red",
               //"margin": "-1px",
               "display": "inline",
               "animation": "cursor_blink-anim 1000ms step-end infinite",
               //"z-index": 1,
            },

            ".ch.wide2 .cursor": {
                "width": px(2 * CH_WIDTH),
            },

            // i-beam cursor
            ".thin_cursor .cursor": {
                "width": px(2),
            },

            ".thin_cursor .wide2 .cursor": {
                "width": px(2 * CH_WIDTH),
            },

            ".block_cursor .cursor": {
                "width": px(CH_WIDTH),
            },

            ".line .ch.wide2": {
                "width": px(2 * CH_WIDTH),
                "font-size": px(12),
            },


            ".status": {
                "background-color": "blue",
                "position": "sticky",
                "bottom": 0,
                "display": "flex",
                "flex-direction": "flex-end",
            },

            "@keyframes cursor_blink-anim": {
              "50%": {
                "background-color": "transparent",
                "border-color": "transparent",
              }
            },

        }
    }
}
