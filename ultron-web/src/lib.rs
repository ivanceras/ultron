//#![deny(warnings)]
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use ultron::editor;
use ultron::editor::Editor;

pub enum Msg {
    EditorMsg(editor::Msg),
    KeyDown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
}

pub struct App {
    editor: Editor,
}

impl App {
    pub fn new() -> Self {
        let content = include_str!("../../ultron/test_data/hello.rs");
        App {
            editor: Editor::from_str(&content),
        }
    }
}

impl Component<Msg> for App {
    fn init(&self) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            let window_elm = web_sys::window().expect("no global `window` exists");

            let program_clone = program.clone();
            let task_keydown: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let ke: KeyboardEvent =
                        event.dyn_into().expect("unable to cast to keyboard event");
                    #[cfg(feature = "with-debug")]
                    log::trace!("keydown got: {:?}", ke.code());
                    program_clone.dispatch(Msg::KeyDown(ke));
                }));
            window_elm
                .add_event_listener_with_callback("keydown", task_keydown.as_ref().unchecked_ref())
                .expect("Unable to attached event listener");
            task_keydown.forget();

            let program_clone = program.clone();
            let task_mouseup: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let me: MouseEvent = e.dyn_into().expect("unable to cast to mousevent");
                    program_clone.dispatch(Msg::Mouseup(me.client_x(), me.client_y()));
                }));
            window_elm
                .add_event_listener_with_callback("mouseup", task_mouseup.as_ref().unchecked_ref())
                .expect("Unable to attached event listener");
            task_mouseup.forget();
        })
    }
    fn style(&self) -> Vec<String> {
        let lib_css = jss!({
            ".app": {
                "display": "flex",
                "flex": "none",
                "width": percent(100),
                "height": percent(100),
            },
        });

        vec![lib_css, self.editor.style().join("\n")]
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
                let should_update_view = self.editor.update(emsg);
                Cmd::should_update_view(should_update_view)
            }
            Msg::Mouseup(_client_x, _client_y) => {
                let should_update_view = self.editor.update(editor::Msg::StopSelection);
                Cmd::should_update_view(should_update_view)
            }
            Msg::KeyDown(ke) => {
                let should_update_view = self.editor.update(editor::Msg::KeyDown(ke));
                Cmd::should_update_view(should_update_view)
            }
        }
    }
}

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    log::trace!("starting ultron..");
    Program::mount_to_body(App::new());
}
