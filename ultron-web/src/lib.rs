//#![deny(warnings)]
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use ultron::editor;
use ultron::editor::Editor;

pub enum Msg {
    EditorMsg(editor::Msg),
    KeyDown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
}

pub struct App {
    editor: Editor,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../../ultron/src/editor.rs");
        // FIXME:
        // optimization: show only the lines
        // from the file that is viewable by the screen
        // for, now we will just use files with smaller size
        let content = include_str!("../../ultron/test_data/hello.rs");
        App {
            editor: Editor::from_str(&content),
        }
    }

    fn convert_mouse_to_line_col(client_x: i32, client_y: i32) -> (usize, usize) {
        let col = client_x as f32 / editor::CH_WIDTH as f32;
        let line = client_y as f32 / editor::CH_HEIGHT as f32;
        (line.round() as usize, col.round() as usize)
    }
}

impl Component<Msg> for App {
    fn init(&self) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            let window_elm = web_sys::window().expect("no global `window` exists");

            let program_clone = program.clone();
            let task_keydown: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |event: web_sys::Event| {
                    event.prevent_default();
                    event.stop_propagation();
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

            let program_clone = program.clone();
            let task_mousedown: Closure<dyn Fn(web_sys::Event)> =
                Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let me: MouseEvent = e.dyn_into().expect("unable to cast to mousevent");
                    program_clone.dispatch(Msg::Mousedown(me.client_x(), me.client_y()));
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
                    let me: MouseEvent = e.dyn_into().expect("unable to cast to mousevent");
                    program_clone.dispatch(Msg::Mousemove(me.client_x(), me.client_y()));
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
                let should_update = self.editor.update(emsg);
                Cmd::should_update_view(should_update)
            }
            Msg::Mouseup(client_x, client_y) => {
                let (line, col) = Self::convert_mouse_to_line_col(client_x, client_y);
                let should_update = self.editor.update(editor::Msg::EndSelection(line, col));
                if should_update {
                    Cmd::measure()
                } else {
                    Cmd::no_render()
                }
            }
            Msg::Mousedown(client_x, client_y) => {
                let (line, col) = Self::convert_mouse_to_line_col(client_x, client_y);
                let should_update = self.editor.update(editor::Msg::StartSelection(line, col));
                if should_update {
                    Cmd::measure()
                } else {
                    Cmd::no_render()
                }
            }
            Msg::Mousemove(client_x, client_y) => {
                let (line, col) = Self::convert_mouse_to_line_col(client_x, client_y);
                let should_update = self.editor.update(editor::Msg::ToSelection(line, col));
                if should_update {
                    Cmd::measure()
                } else {
                    Cmd::no_render()
                }
            }
            Msg::KeyDown(ke) => {
                let should_update = self.editor.update(editor::Msg::KeyDown(ke));
                if should_update {
                    Cmd::measure()
                } else {
                    Cmd::no_render()
                }
            }
        }
    }

    fn measurements(&mut self, measurements: Measurements) -> Cmd<Self, Msg> {
        self.editor.update_took = Some(measurements.total_time);
        Cmd::none()
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
