#![allow(unused)]
use ultron_web::{
    editor, sauron,
    sauron::{html::attributes, jss_ns_pretty, prelude::*, wasm_bindgen::JsCast},
    web_editor, Options, WebEditor, COMPONENT_NAME,
};
use web_sys::HtmlDocument;

pub enum Msg {
    TextareaMounted(web_sys::Node),
    TextareaInput(String),
    Paste(String),
    WebEditorMsg(web_editor::Msg),
    Keydown(web_sys::KeyboardEvent),
    ContextMenu(web_sys::MouseEvent),
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    web_editor: WebEditor<Msg>,
    hidden_textarea: Option<web_sys::HtmlTextAreaElement>,
    last_char_count: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        //let content = include_str!("../test_data/hello.rs");
        let content = include_str!("../test_data/long.rs");
        //let content = include_str!("../test_data/svgbob.md");

        let options = Options {
            syntax_token: "rust".to_string(),
            theme_name: Some("solarized-light".to_string()),
            use_syntax_highlighter: false,
            allow_text_selection: false,
            ..Default::default()
        };
        Self {
            web_editor: WebEditor::from_str(options, content),
            hidden_textarea: None,
            last_char_count: None,
        }
    }

    fn view_hidden_textarea(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let cursor = self.web_editor.cursor_to_client();
        div(
            [
                class_ns("hidden_textarea_wrapper"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                    z_index: 99,
                },
            ],
            [textarea(
                [
                    class_ns("hidden_textarea"),
                    on_mount(|mount| Msg::TextareaMounted(mount.target_node)),
                    #[cfg(web_sys_unstable_apis)]
                    on_paste(|ce| {
                        let pasted_text = ce
                            .clipboard_data()
                            .expect("must have data transfer")
                            .get_data("text/plain")
                            .expect("must be text data");
                        log::trace!("paste triggered from textarea: {}", pasted_text);
                        Msg::Paste(pasted_text)
                    }),
                    // for listening to CTRL+C, CTRL+V, CTRL+X
                    //on_keydown(Msg::TextareaKeydown),
                    focus(true),
                    autofocus(true),
                    attr("autocorrect", "off"),
                    autocapitalize("none"),
                    autocomplete("off"),
                    spellcheck("off"),
                    // for processing unicode characters typed via: CTRL+U<unicode number> (linux),
                    on_input(|input| Msg::TextareaInput(input.value)),
                ],
                [],
            )],
        )
    }

    fn clear_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.set_value("");
        } else {
            panic!("there should always be hidden textarea");
        }
    }

    fn refocus_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.focus().expect("must focus the textarea");
        }
    }

    /// set the content of the textarea to selection
    ///
    /// Note: This is necessary for webkit2.
    /// webkit2 doesn't seem to allow to fire the setting of textarea value, select and copy
    /// in the same animation frame.
    #[allow(unused)]
    fn set_hidden_textarea_with_selection(&self) {
        if let Some(selected_text) = self.web_editor.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
            }
        }
    }

    /// execute copy on the selected textarea
    /// this works even on older browser
    fn textarea_exec_copy(&self) -> bool {
        if let Some(selected_text) = self.web_editor.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
                let html_document: HtmlDocument = sauron::document().unchecked_into();
                if let Ok(ret) = html_document.exec_command("copy") {
                    hidden_textarea.set_value("");
                    log::trace!("exec_copy ret: {}", ret);
                    return ret;
                }
            }
        }
        false
    }

    /// returns true if the command succeeded
    fn textarea_exec_cut(&mut self) -> bool {
        if let Some(selected_text) = self.web_editor.cut_selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                log::trace!("setting the value to textarea: {}", selected_text);
                hidden_textarea.set_value(&selected_text);

                hidden_textarea.select();
                let html_document: HtmlDocument = sauron::document().unchecked_into();
                if let Ok(ret) = html_document.exec_command("cut") {
                    hidden_textarea.set_value("");
                    return ret;
                }
            }
        }
        false
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
            Msg::TextareaMounted(target_node) => {
                self.hidden_textarea = Some(target_node.unchecked_into());
                self.refocus_hidden_textarea();
                Effects::none()
            }
            Msg::TextareaInput(input) => {
                let char_count = input.chars().count();
                // for chrome:
                // detect if the typed in character was a composed and becomes 1 unicode character
                let char_count_decreased = if let Some(last_char_count) = self.last_char_count {
                    last_char_count > 1
                } else {
                    false
                };
                // firefox doesn't register compose key strokes as input
                // if there were 1 char then it was cleared
                let was_cleared = self.last_char_count == Some(0);

                let mut msgs = vec![];
                if char_count == 1 && (was_cleared || char_count_decreased) {
                    self.clear_hidden_textarea();
                    log::trace!("in textarea input char_count == 1..");
                    let c = input.chars().next().expect("must be only 1 chr");
                    let more_msgs = if c == '\n' {
                        self.process_commands([editor::Command::BreakLine])
                    } else {
                        self.process_commands([editor::Command::InsertChar(c)])
                    };
                    msgs.extend(more_msgs);
                } else {
                    log::trace!("char is not inserted becase char_count: {}, was_cleared: {}, char_count_decreased: {}", char_count, was_cleared, char_count_decreased);
                }
                self.last_char_count = Some(char_count);
                log::trace!("extern messages");
                Effects::new(msgs, vec![]).measure()
            }
            Msg::Paste(text_content) => {
                let msgs = self.process_commands([editor::Command::InsertText(text_content)]);
                Effects::new(msgs, vec![])
            }
            Msg::WebEditorMsg(emsg) => {
                let effects = self.web_editor.update(emsg);
                effects.localize(Msg::WebEditorMsg)
            }
            Msg::ContextMenu(me) => {
                log::debug!("Right clicked!");
                Effects::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            [class("app")],
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

            ".hidden_textarea_wrapper": {
                overflow: "hidden",
                position: "relative",
                width: px(300),
                height: px(0),
            },
            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".hidden_textarea": {
                resize: "none",
                position: "absolute",
                padding: 0,
                width: px(300),
                height: px(10),
                border:format!("{} solid black",px(1)),
                bottom: units::em(-1),
                outline: "none",
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
            on_keydown(|ke| {
                ke.prevent_default();
                ke.stop_propagation();
                Msg::Keydown(ke)
            }),
            on_contextmenu(|me| {
                me.prevent_default();
                me.stop_propagation();
                Msg::ContextMenu(me)
            }),
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
