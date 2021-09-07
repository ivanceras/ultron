#![deny(warnings)]
pub use text_buffer::TextBuffer;

pub use sauron;

#[cfg(feature = "with-dom")]
pub mod editor;
mod text_buffer;
mod util;

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 8;
pub const CH_HEIGHT: u32 = 16;

#[derive(Clone, Debug)]
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
    pub theme_name: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_status_line: true,
            show_cursor: true,
            use_spans: true,
            use_for_ssg: false,
            theme_name: None,
        }
    }
}

#[cfg(test)]
mod unit_tests;
