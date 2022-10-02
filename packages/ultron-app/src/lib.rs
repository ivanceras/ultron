#![deny(warnings)]
use app::App;
use ultron_web::sauron;
use ultron_web::sauron::prelude::*;

mod app;

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
