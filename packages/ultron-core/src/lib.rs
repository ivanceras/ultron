#![deny(warnings)]
pub use editor::{Command, Editor};
pub use nalgebra::Point2;
pub use options::Options;
pub use text_buffer::{Ch, TextBuffer};
pub use text_edit::{Selection, TextEdit};
pub use ultron_syntaxes_themes::Color;

pub use nalgebra;

pub mod editor;
mod options;
mod text_buffer;
mod text_edit;
pub mod util;
