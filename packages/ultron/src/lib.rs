//#![deny(warnings)]
pub use editor::Editor;
use sauron::jss::jss;
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use sauron::Window;
pub use text_buffer::TextBuffer;

pub use sauron;

pub mod editor;
mod text_buffer;
mod util;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub show_line_numbers: bool,
    pub show_status_line: bool,
    pub show_cursor: bool,
    /// use spans instead of div when rendering ranges
    /// and characters
    /// this is used when doing a static site rendering
    pub use_spans: bool,
    /// when used for ssg, whitespace will be rendered as &nbsp;
    pub use_for_ssg: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_status_line: true,
            show_cursor: true,
            use_spans: false,
            use_for_ssg: false,
        }
    }
}

#[cfg(test)]
mod unit_tests;
