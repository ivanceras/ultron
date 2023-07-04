#![allow(unused)]
use crate::wasm_bindgen_futures::JsFuture;
use ultron_web::{
    editor, sauron,
    sauron::{
        dom::{Measurements, Window},
        html::attributes::*,
        html::events::*,
        html::*,
        jss_ns_pretty,
        wasm_bindgen::JsCast,
        *,
    },
    web_editor, Options, SelectionMode, WebEditor, COMPONENT_NAME,
};
use web_sys::HtmlDocument;
use web_sys::FontFace;
use crate::wasm_bindgen_futures::spawn_local;
use ultron_web::web_editor::{FONT_NAME,FONT_URL,FONT_SIZE};


pub enum Msg {
    WebEditorMsg(web_editor::Msg),
    Keydown(web_sys::KeyboardEvent),
    FontsLoaded,
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    web_editor: WebEditor<Msg>,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../test_data/hello.rs");
        //let content = include_str!("../test_data/long.rs");
        let content = include_str!("../../ultron-web/src/web_editor.rs");
        //let content = include_str!("../test_data/svgbob.md");

        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            //theme_name: Some("gruvbox-dark".to_string()),
            use_syntax_highlighter: true,
            allow_text_selection: false,
            selection_mode: SelectionMode::Linear,
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
            Msg::FontsLoaded => {
                let effects = self.web_editor.update(web_editor::Msg::FontsLoaded);
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
        Cmd::batch([
            Window::add_event_listeners(vec![
                on_mousemove(|me| Msg::WebEditorMsg(web_editor::Msg::Mousemove(me))),
                on_mousedown(|me| Msg::WebEditorMsg(web_editor::Msg::Mousedown(me))),
                on_mouseup(|me| Msg::WebEditorMsg(web_editor::Msg::Mouseup(me))),
            ]),
            Cmd::new(|program| {
                spawn_local(async move{
                    let font_set = document().fonts();
                    let font_face = FontFace::new_with_str(FONT_NAME, FONT_URL)
                        .expect("font face");
                    font_set.add(&font_face);
                    // Note: the 14px in-front of the font family is needed for this to work
                    // properly
                    JsFuture::from(font_set.load(&format!("{FONT_SIZE}px {FONT_NAME}"))).await;
                    program.dispatch(Msg::FontsLoaded);
                })
            }),
        ])
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
        Cmd::new(|program| {
            program.dispatch(Msg::WebEditorMsg(web_editor::Msg::Measurements(
                measurements,
            )))
        })
    }
}
