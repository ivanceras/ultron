#![allow(warnings)]
use app::App;
pub use sauron;
use sauron::prelude::*;
pub use ultron_core::{editor, nalgebra, Options, TextBuffer};
pub use web_editor::{Command, MouseCursor, WebEditor};

pub use ultron_core;

pub(crate) mod app;
pub(crate) mod util;
pub mod web_editor;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    log::trace!("starting ultron..");
    console_error_panic_hook::set_once();
    let app_container = sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(App::new(), &app_container);
}
