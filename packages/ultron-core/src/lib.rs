#![deny(warnings)]
pub use base_editor::{BaseEditor, Command};
pub use nalgebra::Point2;
pub use options::Options;
pub use text_buffer::{Ch, TextBuffer, BLANK_CH};
pub use text_edit::{Selection, SelectionMode, TextEdit};
pub use ultron_syntaxes_themes::{Color, Style, TextHighlighter};

pub use nalgebra;
pub use unicode_width;

pub mod base_editor;
mod options;
mod text_buffer;
mod text_edit;
pub mod util;
