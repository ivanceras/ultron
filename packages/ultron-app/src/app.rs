use ultron_web::{
    sauron::{
        dom::{Measurements, Task},
        html::events::*,
        html::*,
        jss_ns_pretty, *,
    },
    web_editor, BaseOptions, Options, SelectionMode, WebEditor, COMPONENT_NAME,
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
            use_syntax_highlighter: true,
            allow_text_selection: false,
            base_options: BaseOptions {
                selection_mode: SelectionMode::Linear,
                ..Default::default()
            },
            ch_width: None,
            ch_height: None,
            ..Default::default()
        };
        let mut web_editor: WebEditor<Msg> = WebEditor::from_str(&options, content);
        web_editor.on_ready(|_| Msg::EditorReady);
        Self { web_editor }
    }
}

impl Component<Msg, ()> for App {
    fn init(&mut self) -> Vec<Task<Msg>> {
        self.web_editor
            .init()
            .into_iter()
            .map(|task| task.map_msg(Msg::WebEditorMsg))
            .collect()
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::EditorReady => {
                log::info!("Editor is now ready..");
                Effects::none()
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

    fn style(&self) -> Vec<String> {
        let css = jss_ns_pretty! {COMPONENT_NAME,
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
        };
        [vec![css], self.web_editor.style()].concat()
    }
}

/// Auto implementation of Application trait for Component that
/// has no external MSG
impl Application<Msg> for App {
    fn init(&mut self) -> Vec<Cmd<Self, Msg>> {
        vec![
            Cmd::new(|program| {
                program.add_window_event_listeners(vec![
                    on_mousemove(|me| Msg::WebEditorMsg(web_editor::Msg::Mousemove(me))),
                    on_mousedown(|me| Msg::WebEditorMsg(web_editor::Msg::Mousedown(me))),
                    on_mouseup(|me| Msg::WebEditorMsg(web_editor::Msg::Mouseup(me))),
                ])
            }),
            Cmd::batch(
                <Self as crate::Component<Msg, ()>>::init(self)
                    .into_iter()
                    .map(Cmd::from),
            ),
        ]
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        let effects = <Self as crate::Component<Msg, ()>>::update(self, msg);
        Cmd::from(effects)
    }

    fn view(&self) -> Node<Msg> {
        <Self as crate::Component<Msg, ()>>::view(self)
    }

    fn style(&self) -> Vec<String> {
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
