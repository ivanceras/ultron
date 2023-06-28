#![deny(warnings)]
pub use sauron;
use sauron::*;
pub use ultron_core::{editor, nalgebra, Options, SelectionMode, TextBuffer};
pub use web_editor::custom_element::{attributes, register, ultron_editor, WebEditorCustomElement};
pub use web_editor::{Command, MouseCursor, WebEditor, COMPONENT_NAME};

pub use ultron_core;

pub(crate) mod context_menu;
pub(crate) mod util;
pub mod web_editor;
