use crate::editor_web;
use crate::editor_web::EditorWeb;
use crate::editor_web::COMPONENT_NAME;
use crate::ultron_core::editor;
use sauron::{html::attributes, prelude::*, wasm_bindgen::JsCast};
pub use ultron_core;
use web_sys::HtmlDocument;

pub enum Msg {
    TextareaMounted(web_sys::Node),
    TextareaInput(String),
    TextareaKeydown(web_sys::KeyboardEvent),
    Paste(String),
    EditorWebMsg(editor_web::Msg),
    Keydown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
}

/// The web editor with text area hacks for listening to typing events
pub struct App {
    editor_web: EditorWeb,
    hidden_textarea: Option<web_sys::HtmlTextAreaElement>,
}

impl App {
    pub fn new() -> Self {
        Self {
            editor_web: EditorWeb::new(),
            hidden_textarea: None,
        }
    }

    fn view_hidden_textarea(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.editor_web.cursor_to_client();
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
                        log::trace!(
                            "paste triggered from textarea: {}",
                            pasted_text
                        );
                        Msg::Paste(pasted_text)
                    }),
                    // for listening to CTRL+C, CTRL+V, CTRL+X
                    on_keydown(Msg::TextareaKeydown),
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
        if let Some(selected_text) = self.editor_web.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
            }
        }
    }

    /// execute copy on the selected textarea
    /// this works even on older browser
    fn textarea_exec_copy(&self) -> bool {
        if let Some(selected_text) = self.editor_web.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
                let html_document: HtmlDocument =
                    sauron::document().unchecked_into();
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
        if let Some(selected_text) = self.editor_web.cut_selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                log::trace!("setting the value to textarea: {}", selected_text);
                hidden_textarea.set_value(&selected_text);

                hidden_textarea.select();
                let html_document: HtmlDocument =
                    sauron::document().unchecked_into();
                if let Ok(ret) = html_document.exec_command("cut") {
                    hidden_textarea.set_value("");
                    return ret;
                }
            }
        }
        false
    }
}

impl Component<Msg, ()> for App {
    fn update(&mut self, msg: Msg) -> Effects<Msg, ()> {
        match msg {
            Msg::TextareaKeydown(ke) => {
                /*
                let effects = self.editor_web.process_keypress(&ke);
                effects.map_msg(Msg::EditorWebMsg)
                */
                Effects::none()
            }
            Msg::TextareaMounted(target_node) => {
                self.hidden_textarea = Some(target_node.unchecked_into());
                self.refocus_hidden_textarea();
                Effects::none()
            }
            Msg::TextareaInput(input) => {
                /*
                let char_count = input.chars().count();
                // for chrome:
                // detect if the typed in character was a composed and becomes 1 unicode character
                let char_count_decreased =
                    if let Some(last_char_count) = self.last_char_count {
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
                    self.composed_key = Some(c);
                    let more_msgs = if c == '\n' {
                        self.editor_web.process_command(Command::BreakLine)
                    } else {
                        self.editor_web.process_command(Command::InsertChar(c))
                    };
                    msgs.extend(more_msgs);
                } else {
                    log::trace!("char is not inserted becase char_count: {}, was_cleared: {}, char_count_decreased: {}", char_count, was_cleared, char_count_decreased);
                }
                self.last_char_count = Some(char_count);
                log::trace!("extern messages");
                Effects::new(msgs, vec![]).measure()
                */
                Effects::none()
            }
            Msg::Paste(text_content) => {
                let msgs = self.editor_web.process_command(
                    editor_web::Command::EditorCommand(
                        editor::Command::InsertText(text_content),
                    ),
                );
                Effects::new(msgs.into_iter().map(Msg::EditorWebMsg), vec![])
            }
            Msg::Keydown(key_event) => {
                let effects =
                    self.editor_web.update(editor_web::Msg::Keydown(key_event));
                effects.map_msg(Msg::EditorWebMsg)
            }
            Msg::Mouseup(client_x, client_y) => {
                let effects = self
                    .editor_web
                    .update(editor_web::Msg::Mouseup(client_x, client_y));
                effects.map_msg(Msg::EditorWebMsg)
            }
            Msg::Mousedown(client_x, client_y) => {
                let effects = self
                    .editor_web
                    .update(editor_web::Msg::Mousedown(client_x, client_y));
                effects.map_msg(Msg::EditorWebMsg)
            }
            Msg::Mousemove(client_x, client_y) => {
                let effects = self
                    .editor_web
                    .update(editor_web::Msg::Mousemove(client_x, client_y));
                effects.map_msg(Msg::EditorWebMsg)
            }
            Msg::EditorWebMsg(emsg) => {
                let effects = self.editor_web.update(emsg);
                effects.map_msg(Msg::EditorWebMsg)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            [],
            [
                self.editor_web.view().map_msg(Msg::EditorWebMsg),
                //self.view_hidden_textarea(),
            ],
        )
    }

    fn style(&self) -> String {
        self.editor_web.style()
    }
}

/// Auto implementation of Application trait for Component that
/// has no external MSG
impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::batch([Window::add_event_listeners(vec![
            on_mousemove(|me| Msg::Mousemove(me.client_x(), me.client_y())),
            on_mousedown(|me| Msg::Mousedown(me.client_x(), me.client_y())),
            on_mouseup(|me| Msg::Mouseup(me.client_x(), me.client_y())),
            on_keydown(|ke| {
                ke.prevent_default();
                ke.stop_propagation();
                Msg::Keydown(ke)
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
}
