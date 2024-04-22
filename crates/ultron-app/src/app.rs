use ultron_web::{
    sauron, sauron::prelude::*, web_editor, BaseOptions, Options, SelectionMode, WebEditor,
};

pub enum Msg {
    WebEditorMsg(web_editor::Msg),
    EditorReady,
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    web_editor: WebEditor<Msg>,
}

impl App {
    pub fn new() -> Self {
        let content = include_str!("../../ultron-web/src/web_editor.rs");
        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            use_syntax_highlighter: false,
            allow_text_selection: true,
            base_options: BaseOptions {
                selection_mode: SelectionMode::Linear,
                ..Default::default()
            },
            ch_width: None,
            ch_height: None,
            ..Default::default()
        };
        let mut web_editor: WebEditor<Msg> = WebEditor::from_str(&options, content);
        web_editor.on_ready(|| Msg::EditorReady);
        Self { web_editor }
    }
}

impl Application for App {
    type MSG = Msg;

    fn init(&mut self) -> Cmd<Msg> {
        Cmd::batch([
            Window::on_mousemove(|me| Msg::WebEditorMsg(web_editor::Msg::Mousemove(me))),
            Window::on_mousedown(|me| Msg::WebEditorMsg(web_editor::Msg::Mousedown(me))),
            Window::on_mouseup(|me| Msg::WebEditorMsg(web_editor::Msg::Mouseup(me))),
            Document::on_selectionchange(|selection| {
                Msg::WebEditorMsg(web_editor::Msg::Selection(selection))
            }),
            Cmd::from(self.web_editor.init().localize(Msg::WebEditorMsg)),
        ])
    }

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::EditorReady => {
                log::info!("Editor is now ready..");
                Cmd::none()
            }
            Msg::WebEditorMsg(emsg) => {
                let effects = self.web_editor.update(emsg).localize(Msg::WebEditorMsg);
                Cmd::from(effects)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            [class("app")],
            [self.web_editor.view().map_msg(Msg::WebEditorMsg)],
        )
    }

    fn stylesheet() -> Vec<String> {
        let css = jss! {
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
        };
        [vec![css], WebEditor::<Msg>::stylesheet()].concat()
    }

    fn style(&self) -> Vec<String> {
        self.web_editor.style()
    }

    fn measurements(&mut self, measurements: Measurements) {
        log::info!("got some measurements..");
        self.web_editor
            .update(web_editor::Msg::Measurements(measurements));
    }
}
