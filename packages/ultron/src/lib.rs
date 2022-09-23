#![deny(warnings)]
pub use ultron_core::{
    TextBuffer,
    TextEdit,
};
pub use ultron_syntaxes_themes::TextHighlighter;

pub use nalgebra;
pub use sauron;

#[cfg(feature = "with-dom")]
pub mod editor;
#[cfg(feature = "with-dom")]
pub use editor::{
    Command,
    Editor,
    MouseCursor,
    Msg,
};

mod util;

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 7;
pub const CH_HEIGHT: u32 = 16;

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
