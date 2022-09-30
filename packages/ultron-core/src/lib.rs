#![deny(warnings)]
pub use editor::{Command, Editor, Options};
pub use nalgebra::Point2;
pub use text_buffer::TextBuffer;
pub use text_edit::{Selection, TextEdit};
pub use ultron_syntaxes_themes::Color;

pub use nalgebra;

pub mod editor;
mod text_buffer;
mod text_edit;
pub mod util;
