#![deny(warnings)]
pub use base_editor::{BaseCommand, BaseEditor};
pub use base_options::BaseOptions;
pub use nalgebra::Point2;
pub use text_buffer::{Ch, TextBuffer, BLANK_CH};
pub use text_edit::{Selection, SelectionMode, TextEdit};
pub use ultron_syntaxes_themes::{Color, Style, TextHighlighter};

pub use nalgebra;
pub use unicode_width;

pub mod base_editor;
mod base_options;
mod text_buffer;
mod text_edit;
pub mod util;
