#![deny(warnings)]
use ultron::{editor, editor::Editor};
use ultron::{
    sauron::{jss, prelude::*, Window},
    Options,
};

#[derive(Debug, Clone)]
pub enum Msg {
    WindowScrolled((i32, i32)),
    WindowResized(i32, i32),
    EditorMsg(editor::Msg),
    Keydown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    NoOp,
}

pub struct App {
    editor: Editor<Msg>,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../test_data/hello.rs");
        let content = include_str!("../test_data/long.rs");
        //let content = include_str!("../test_data/svgbob.md");
        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            ..Default::default()
        };
        App {
            editor: Editor::from_str(options, content),
        }
    }
}

impl Application<Msg> for App {
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
            Msg::WindowScrolled((scroll_top, scroll_left)) => {
                log::trace!("scrolled: {},{}", scroll_top, scroll_left);
                let effects = self.editor.update(editor::Msg::WindowScrolled(
                    (scroll_top, scroll_left),
                ));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::WindowResized(width, height) => {
                self.editor
                    .update(editor::Msg::WindowResized { width, height });
                Cmd::none()
            }
            Msg::EditorMsg(emsg) => {
                let effects = self.editor.update(emsg);
                Cmd::from(effects.localize(Msg::EditorMsg))
            }
            Msg::Mouseup(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mouseup(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::Mousedown(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mousedown(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::Mousemove(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mousemove(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg))
            }
            Msg::Keydown(ke) => {
                let effects =
                    self.editor.update(editor::Msg::WindowKeydown(ke));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::NoOp => Cmd::none().no_render(),
        }
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            program.dispatch(Msg::EditorMsg(editor::Msg::SetMeasurement(
                measurements,
            )))
        })
        .no_render()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    log::trace!("starting ultron..");
    console_error_panic_hook::set_once();
    let app_container = ultron::sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(App::new(), &app_container);
}
