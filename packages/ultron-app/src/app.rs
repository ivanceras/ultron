#![allow(unused)]
use crate::wasm_bindgen_futures::spawn_local;
use crate::wasm_bindgen_futures::JsFuture;
use ultron_web::font_loader::{self, FontLoader};
use ultron_web::web_editor::{FONT_NAME, FONT_SIZE, FONT_URL};
use ultron_web::{
    base_editor, sauron,
    sauron::{
        dom::{self, Measurements, Window},
        html::attributes::*,
        html::events::*,
        html::*,
        jss_ns_pretty,
        wasm_bindgen::JsCast,
        *,
    },
    web_editor, Options, SelectionMode, WebEditor, COMPONENT_NAME,
};
use web_sys::FontFace;
use web_sys::HtmlDocument;

pub enum Msg {
    WebEditorMsg(web_editor::Msg),
    Keydown(web_sys::KeyboardEvent),
    FontLoaderMsg(font_loader::Msg),
    /// when the font is ready
    FontReady,
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    font_loader: FontLoader<Msg>,
    web_editor: Option<WebEditor<Msg>>,
}

impl App {
    pub fn create_web_editor(&mut self, ch_width: Option<f32>, ch_height: Option<f32>) {
        let content = include_str!("../../ultron-web/src/web_editor.rs");
        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            use_syntax_highlighter: true,
            allow_text_selection: false,
            selection_mode: SelectionMode::Linear,
            ch_width,
            ch_height,
            ..Default::default()
        };
        let web_editor: WebEditor<Msg> = WebEditor::from_str(options, content);
        dom::inject_style(&web_editor.style());
        self.web_editor = Some(web_editor);
    }
    pub fn new() -> Self {
        let mut font_loader = FontLoader::new(FONT_SIZE as f32, &FONT_NAME, &FONT_URL);
        font_loader.on_fonts_ready(|_| Msg::FontReady);

        Self {
            web_editor: None,
            font_loader,
        }
    }

    fn process_commands(
        &mut self,
        wcommands: impl IntoIterator<Item = impl Into<web_editor::Command>>,
    ) -> Vec<Msg> {
        if let Some(web_editor) = self.web_editor.as_mut() {
            web_editor
                .process_commands(wcommands.into_iter().map(|wcommand| wcommand.into()))
                .into_iter()
                .collect()
        } else {
            vec![]
        }
    }
}

impl Component<Msg, ()> for App {
    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::Keydown(ke) => {
                if let Some(web_editor) = self.web_editor.as_mut() {
                    let effects = web_editor.update(web_editor::Msg::Keydown(ke));
                    effects.localize(Msg::WebEditorMsg)
                } else {
                    Effects::none()
                }
            }
            Msg::WebEditorMsg(emsg) => {
                if let Some(web_editor) = self.web_editor.as_mut() {
                    let effects = web_editor.update(emsg);
                    effects.localize(Msg::WebEditorMsg)
                } else {
                    Effects::none()
                }
            }
            Msg::FontLoaderMsg(fmsg) => {
                let effects = self.font_loader.update(fmsg);
                effects.localize(Msg::FontLoaderMsg)
            }
            Msg::FontReady => {
                let ch_width = self.font_loader.ch_width;
                let ch_height = self.font_loader.ch_height;
                self.create_web_editor(ch_width, ch_height);
                Effects::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        if let Some(web_editor) = self.web_editor.as_ref() {
            div(
                [class_ns("app")],
                [web_editor.view().map_msg(Msg::WebEditorMsg)],
            )
        } else {
            div(
                [],
                [
                    text("Loading fonts...."),
                    self.font_loader.view().map_msg(Msg::FontLoaderMsg),
                ],
            )
        }
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
        [css].join("\n")
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
            Cmd::from(self.font_loader.init().map_msg(Msg::FontLoaderMsg)),
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
