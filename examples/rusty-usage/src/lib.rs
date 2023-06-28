use ultron_web::{
    sauron::{html::attributes::*, html::*, *},
    ultron_editor,
};

enum Msg {}

struct App {
    content: String,
}

impl App {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        div(
            [class("container")],
            [ultron_editor([value(&self.content), syntax("rust")], [])],
        )
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        Cmd::none()
    }

    fn style(&self) -> String {
        "".to_string()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    Program::mount_to_body(App::new(include_str!("./lib.rs")));
}
