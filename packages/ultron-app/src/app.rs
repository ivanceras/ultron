#![allow(unused)]
use ultron_web::{
    editor, sauron,
    sauron::{html::attributes, jss_ns_pretty, prelude::*, wasm_bindgen::JsCast},
    web_editor, Options, SelectionMode, WebEditor, COMPONENT_NAME,
};
use web_sys::HtmlDocument;

pub enum Msg {
    WebEditorMsg(web_editor::Msg),
    Keydown(web_sys::KeyboardEvent),
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    web_editor: WebEditor<Msg>,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../test_data/hello.rs");
        let content = include_str!("../test_data/long.rs");
        //let content = include_str!("../test_data/svgbob.md");

        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            //theme_name: Some("gruvbox-dark".to_string()),
            use_syntax_highlighter: true,
            allow_text_selection: false,
            selection_mode: SelectionMode::Block,
            ..Default::default()
        };
        Self {
            web_editor: WebEditor::from_str(options, content),
        }
    }

    fn process_commands(
        &mut self,
        wcommands: impl IntoIterator<Item = impl Into<web_editor::Command>>,
    ) -> Vec<Msg> {
        self.web_editor
            .process_commands(wcommands.into_iter().map(|wcommand| wcommand.into()))
            .into_iter()
            .collect()
    }
}

impl Component<Msg, ()> for App {
    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::Keydown(ke) => {
                let effects = self.web_editor.update(web_editor::Msg::Keydown(ke));
                effects.localize(Msg::WebEditorMsg)
            }
            Msg::WebEditorMsg(emsg) => {
                let effects = self.web_editor.update(emsg);
                effects.localize(Msg::WebEditorMsg)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(
            [class_ns("app")],
            [self.web_editor.view().map_msg(Msg::WebEditorMsg)],
        )
    }

    fn style(&self) -> String {
        let css = jss_ns_pretty! {COMPONENT_NAME,
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
        };
        [css, self.web_editor.style()].join("\n")
    }
}

/// Auto implementation of Application trait for Component that
/// has no external MSG
impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::batch([Window::add_event_listeners(vec![
            on_mousemove(|me| Msg::WebEditorMsg(web_editor::Msg::Mousemove(me))),
            on_mousedown(|me| Msg::WebEditorMsg(web_editor::Msg::Mousedown(me))),
            on_mouseup(|me| Msg::WebEditorMsg(web_editor::Msg::Mouseup(me))),
        ])])
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        let effects = <Self as crate::Component<Msg, ()>>::update(self, msg);
        Cmd::from(effects)
    }

    fn view(&self) -> Node<Msg> {
        <Self as crate::Component<Msg, ()>>::view(self)
    }

    fn style(&self) -> String {
        <Self as crate::Component<Msg, ()>>::style(self)
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        log::info!("measurements in ultron app");
        Cmd::new(|program| {
            program.dispatch(Msg::WebEditorMsg(web_editor::Msg::Measurements(
                measurements,
            )))
        })
    }
}
