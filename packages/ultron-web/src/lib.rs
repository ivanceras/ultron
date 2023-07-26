#![deny(warnings)]
pub use font_loader::FontLoader;
pub use sauron;
use sauron::*;
pub use ultron_core::{base_editor, nalgebra, SelectionMode, TextBuffer};
#[cfg(feature = "custom_element")]
pub use web_editor::custom_element::{attributes, register, ultron_editor, WebEditorCustomElement};
pub use web_editor::{
    BaseOptions, Call, Command, FontSettings, MouseCursor, Options, WebEditor, COMPONENT_NAME,
};

pub use ultron_core;

pub(crate) mod context_menu;
pub mod font_loader;
pub(crate) mod util;
pub mod web_editor;
