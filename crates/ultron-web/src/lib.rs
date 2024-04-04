#![deny(warnings)]
pub use font_loader::FontLoader;
pub use sauron;
use sauron::*;
pub use spinner::Spinner;
pub use ultron_core::{base_editor, nalgebra, SelectionMode, TextBuffer};
#[cfg(feature = "custom_element")]
pub use web_editor::custom_element::{attributes, register, ultron_editor};
pub use web_editor::{BaseOptions, Call, Command, FontSettings, MouseCursor, Options, WebEditor};

pub use ultron_core;

pub(crate) mod context_menu;
pub mod font_loader;
pub mod spinner;
pub(crate) mod util;
pub mod web_editor;
