//#![deny(warnings)]
pub use editor::Editor;
use sauron::jss::jss;
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;

pub mod editor;
mod util;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub show_line_numbers: bool,
    pub show_status_line: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_status_line: true,
        }
    }
}

pub enum Msg {
    EditorMsg(editor::Msg),
    KeyDown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
}

pub struct App {
    editor: Editor<Msg>,
}

impl App {
    pub fn new() -> Self {
        let content = include_str!("../test_data/hello.rs");
        //let content = include_str!("../test_data/svgbob.md");
        App {
            editor: Editor::from_str(&content, "rust"),
        }
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            let window_elm =
                web_sys::window().expect("no global `window` exists");

            let program_clone = program.clone();
            let task_keydown: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |event: web_sys::Event| {
                    event.prevent_default();
                    event.stop_propagation();
                    let ke: KeyboardEvent = event
                        .dyn_into()
                        .expect("unable to cast to keyboard event");
                    program_clone.dispatch(Msg::KeyDown(ke));
                }));
            window_elm
                .add_event_listener_with_callback(
                    "keydown",
                    task_keydown.as_ref().unchecked_ref(),
                )
                .expect("Unable to attached event listener");
            task_keydown.forget();

            let program_clone = program.clone();
            let task_mouseup: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let me: MouseEvent =
                        e.dyn_into().expect("unable to cast to mousevent");
                    program_clone
                        .dispatch(Msg::Mouseup(me.client_x(), me.client_y()));
                }));
            window_elm
                .add_event_listener_with_callback(
                    "mouseup",
                    task_mouseup.as_ref().unchecked_ref(),
                )
                .expect("Unable to attached event listener");
            task_mouseup.forget();

            let program_clone = program.clone();
            let task_mousedown: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let me: MouseEvent =
                        e.dyn_into().expect("unable to cast to mousevent");
                    program_clone
                        .dispatch(Msg::Mousedown(me.client_x(), me.client_y()));
                }));
            window_elm
                .add_event_listener_with_callback(
                    "mousedown",
                    task_mousedown.as_ref().unchecked_ref(),
                )
                .expect("Unable to attached event listener");
            task_mousedown.forget();

            let program_clone = program.clone();
            let task_mousemove: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let me: MouseEvent =
                        e.dyn_into().expect("unable to cast to mousevent");
                    program_clone
                        .dispatch(Msg::Mousemove(me.client_x(), me.client_y()));
                }));
            window_elm
                .add_event_listener_with_callback(
                    "mousemove",
                    task_mousemove.as_ref().unchecked_ref(),
                )
                .expect("Unable to attached event listener");
            task_mousemove.forget();
        })
    }
    fn style(&self) -> String {
        let lib_css = jss! {
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
        };

        [lib_css, self.editor.style()].join("\n")
    }
    fn view(&self) -> Node<Msg> {
        div(
            vec![class("app")],
            vec![self.editor.view().map_msg(Msg::EditorMsg)],
        )
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::EditorMsg(emsg) => {
                self.editor.update(emsg);
                Cmd::none()
            }
            Msg::Mouseup(client_x, client_y) => {
                self.editor.update(editor::Msg::Mouseup(client_x, client_y));
                Cmd::none().measure()
            }
            Msg::Mousedown(client_x, client_y) => {
                self.editor
                    .update(editor::Msg::Mousedown(client_x, client_y));
                Cmd::none().measure()
            }
            Msg::Mousemove(client_x, client_y) => {
                self.editor
                    .update(editor::Msg::Mousemove(client_x, client_y));
                Cmd::none().no_render()
            }
            Msg::KeyDown(ke) => {
                self.editor.update(editor::Msg::KeyDown(ke));
                Cmd::none().measure()
            }
        }
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            program.dispatch(Msg::EditorMsg(editor::Msg::SetMeasurement(
                measurements.clone(),
            )))
        })
        .no_render()
    }
}

#[cfg(test)]
mod unit_tests;

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "standalone")]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    log::trace!("starting ultron..");
    Program::mount_to_body(App::new());
}
