use ultron_web::{
    attributes::{syntax, theme},
    sauron::{html::attributes::*, html::events::*, html::*, *},
    ultron_editor,
};

enum Msg {
    ContentChanged(String),
}

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
    fn init(&mut self) -> Vec<Cmd<Self, Msg>> {
        vec![]
    }

    fn view(&self) -> Node<Msg> {
        div(
            [class("container")],
            [ultron_editor(
                [
                    syntax("rust"),
                    theme("solarized-light"),
                    value(&self.content),
                    on_input(|input| Msg::ContentChanged(input.value)),
                ],
                [],
            )],
        )
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::ContentChanged(new_content) => {
                log::info!("Content has been changed to: \n{new_content}");
                Cmd::none()
            }
        }
    }

    fn style(&self) -> Vec<String> {
        vec![jss_pretty! {
            body: {
                margin: 0,
            }
        }]
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    ultron_web::register();
    Program::mount_to_body(App::new(include_str!("./lib.rs")));
}
