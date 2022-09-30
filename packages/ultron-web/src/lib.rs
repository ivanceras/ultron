#![deny(warnings)]
use editor_web::Msg;
pub use editor_web::{EditorWeb, MouseCursor};
use sauron::prelude::*;

pub use ultron_core;

mod editor_web;
pub(crate) mod util;

struct EditorApp {
    editor_web: EditorWeb,
}

impl EditorApp {
    pub fn new() -> Self {
        Self {
            editor_web: EditorWeb::new(),
        }
    }
}

impl Application<Msg> for EditorApp {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::batch([
            Window::on_resize(Msg::WindowResized),
            Window::add_event_listeners(vec![
                on_scroll(Msg::WindowScrolled),
                on_mousemove(|me| Msg::Mousemove(me.client_x(), me.client_y())),
                on_mousedown(|me| Msg::Mousedown(me.client_x(), me.client_y())),
                on_mouseup(|me| Msg::Mouseup(me.client_x(), me.client_y())),
                on_keydown(|ke| {
                    ke.prevent_default();
                    ke.stop_propagation();
                    Msg::Keydown(ke)
                }),
            ]),
        ])
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        let effects = self.editor_web.update(msg);
        Cmd::from(effects)
    }

    fn view(&self) -> Node<Msg> {
        self.editor_web.view()
    }

    fn style(&self) -> String {
        self.editor_web.style()
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        log::info!("measurements: {:?}", measurements);
        Cmd::none()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    log::trace!("starting ultron..");
    console_error_panic_hook::set_once();
    let app_container = sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(EditorApp::new(), &app_container);
}
