#![deny(warnings)]
use ultron::{
    editor,
    editor::{
        Command,
        Editor,
    },
    sauron,
    sauron::{
        html::attributes,
        jss,
        prelude::*,
        wasm_bindgen::JsCast,
        web_sys::HtmlDocument,
        Window,
    },
    Options,
    COMPONENT_NAME,
};

#[derive(Debug, Clone)]
pub enum Msg {
    WindowScrolled((i32, i32)),
    WindowResized(i32, i32),
    EditorMsg(editor::Msg),
    Keydown(web_sys::KeyboardEvent),
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    TextareaKeydown(web_sys::KeyboardEvent),
    TextareaInput(String),
    Paste(String),
    TextareaMounted(web_sys::Node),
    NoOp,
}

pub struct App {
    editor: Editor<Msg>,
    hidden_textarea: Option<web_sys::HtmlTextAreaElement>,
    composed_key: Option<char>,
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
            ..Default::default()
        };
        App {
            editor: Editor::from_str(options, content),
            hidden_textarea: None,
            composed_key: None,
            last_char_count: None,
        }
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::batch([
            Window::on_resize(Msg::WindowResized),
            Window::add_event_listeners(vec![
                on_scroll(Msg::WindowScrolled),
                on_mousemove(|me| Msg::Mousemove(me.client_x(), me.client_y())),
                on_mousedown(|me| Msg::Mousedown(me.client_x(), me.client_y())),
                on_mouseup(|me| Msg::Mouseup(me.client_x(), me.client_y())),
                on_keydown(|ke| {
                    ke.prevent_default();
                    ke.stop_propagation();
                    Msg::Keydown(ke)
                }),
            ]),
        ])
    }

    fn style(&self) -> String {
        let lib_css = jss! {
            ".app": {
                display: "flex",
                flex: "none",
                width: percent(100),
                height: percent(100),
            },
            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".hidden_textarea": {
                resize: "none",
                height: 0,
                position: "absolute",
                padding: 0,
                width: px(300),
                height: px(0),
                border:format!("{} solid black",px(1)),
                bottom: units::em(-1),
                outline: "none",
            },
        };

        [lib_css, self.editor.style()].join("\n")
    }

    fn view(&self) -> Node<Msg> {
        div(
            vec![class("app")],
            vec![
                self.editor.view().map_msg(Msg::EditorMsg),
                //self.view_hidden_textarea(),
            ],
        )
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::WindowScrolled((_scroll_top, _scroll_left)) => Cmd::none(),
            Msg::WindowResized(_width, _height) => Cmd::none(),
            Msg::EditorMsg(emsg) => {
                let effects = self.editor.update(emsg);
                Cmd::from(effects.localize(Msg::EditorMsg))
            }
            Msg::Mouseup(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mouseup(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::Mousedown(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mousedown(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }
            Msg::Mousemove(client_x, client_y) => {
                let effects = self
                    .editor
                    .update(editor::Msg::Mousemove(client_x, client_y));
                Cmd::from(effects.localize(Msg::EditorMsg))
            }
            Msg::Keydown(ke) => self.process_keypress(&ke),
            Msg::TextareaKeydown(ke) => self.process_keypress(&ke),
            Msg::TextareaMounted(target_node) => {
                self.hidden_textarea = Some(target_node.unchecked_into());
                self.refocus_hidden_textarea();
                Cmd::none()
            }
            Msg::TextareaInput(input) => {
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

                let mut effects = Effects::none();
                if char_count == 1 && (was_cleared || char_count_decreased) {
                    self.clear_hidden_textarea();
                    log::trace!("in textarea input char_count == 1..");
                    let c = input.chars().next().expect("must be only 1 chr");
                    self.composed_key = Some(c);
                    effects = if c == '\n' {
                        self.editor.process_command(Command::BreakLine)
                    } else {
                        self.editor.process_command(Command::InsertChar(c))
                    };
                } else {
                    log::trace!("char is not inserted becase char_count: {}, was_cleared: {}, char_count_decreased: {}", char_count, was_cleared, char_count_decreased);
                }
                self.last_char_count = Some(char_count);
                log::trace!("extern messages");
                Cmd::from(effects.localize(Msg::EditorMsg)).measure()
            }

            Msg::Paste(text_content) => {
                let effects = self
                    .editor
                    .process_command(Command::InsertText(text_content));
                Cmd::from(effects.localize(Msg::EditorMsg))
            }
            Msg::NoOp => Cmd::none().no_render(),
        }
    }

    fn measurements(&self, measurements: Measurements) -> Cmd<Self, Msg> {
        Cmd::new(move |program| {
            program.dispatch(Msg::EditorMsg(editor::Msg::SetMeasurement(
                measurements,
            )))
        })
        .no_render()
    }
}

impl App {
    #[allow(unused)]
    fn view_hidden_textarea(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.editor.cursor_to_client();
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

    fn process_keypress(
        &mut self,
        ke: &web_sys::KeyboardEvent,
    ) -> Cmd<Self, Msg> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            log::trace!("inserting from window keydown event");
            let c = key.chars().next().expect("must be only 1 chr");
            match c {
                'c' if is_ctrl => {
                    self.command_copy();
                    Cmd::none()
                }
                'x' if is_ctrl => {
                    self.command_cut();
                    Cmd::none()
                }
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    self.clear_hidden_textarea();
                    Cmd::none()
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        self.editor.process_command(Command::Redo);
                    } else {
                        self.editor.process_command(Command::Undo);
                    }
                    Cmd::none()
                }
                'a' if is_ctrl => {
                    self.editor.process_command(Command::SelectAll);
                    Cmd::none()
                }
                _ => {
                    self.editor.process_command(Command::InsertChar(c));
                    Cmd::none()
                }
            }
        } else {
            let command = match &*key {
                "Tab" => Some(Command::IndentForward),
                "Enter" => Some(Command::BreakLine),
                "Backspace" => Some(Command::DeleteBack),
                "Delete" => Some(Command::DeleteForward),
                "ArrowUp" => Some(Command::MoveUp),
                "ArrowDown" => Some(Command::MoveDown),
                "ArrowLeft" => Some(Command::MoveLeft),
                "ArrowRight" => Some(Command::MoveRight),
                _ => None,
            };
            if let Some(command) = command {
                let effects = self.editor.process_command(command);
                Cmd::from(effects.localize(Msg::EditorMsg))
            } else {
                Cmd::none()
            }
        }
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
        if let Some(selected_text) = self.editor.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                hidden_textarea.set_value(&selected_text);
                hidden_textarea.select();
            }
        }
    }

    /// this is for newer browsers
    /// This doesn't work on webkit2
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn copy_to_clipboard(&self) -> bool {
        if let Some(selected_text) = self.editor.selected_text() {
            let navigator = sauron::window().navigator();
            if let Some(clipboard) = navigator.clipboard() {
                let _ = clipboard.write_text(&selected_text);
                return true;
            } else {
                log::warn!("no navigator clipboard");
            }
        }
        false
    }

    #[cfg(not(feature = "with-navigator-clipboard"))]
    fn copy_to_clipboard(&self) -> bool {
        false
    }

    /// execute copy on the selected textarea
    /// this works even on older browser
    fn textarea_exec_copy(&self) -> bool {
        if let Some(selected_text) = self.editor.selected_text() {
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
        if let Some(selected_text) = self.editor.cut_selected_text() {
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

    /// calls on 2 ways to copy
    /// either 1 should work
    /// returns true if it succeded
    fn command_copy(&self) {
        if self.copy_to_clipboard() {
            // do nothing
        } else {
            self.textarea_exec_copy();
        }
    }

    /// try exec_cut, try cut to clipboard if the first fails
    /// This shouldn't execute both since cut is destructive.
    /// Returns true if it succeded
    fn command_cut(&mut self) {
        if self.cut_to_clipboard() {
            // nothing
        } else {
            self.textarea_exec_cut();
        }
    }

    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn cut_to_clipboard(&mut self) -> bool {
        if let Some(selected_text) = self.editor.cut_selected_text() {
            let navigator = sauron::window().navigator();
            if let Some(clipboard) = navigator.clipboard() {
                let _ = clipboard.write_text(&selected_text);
                return true;
            } else {
                log::warn!("no navigator clipboard");
            }
        }
        false
    }

    #[cfg(not(feature = "with-navigator-clipboard"))]
    fn cut_to_clipboard(&mut self) -> bool {
        false
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    log::trace!("starting ultron..");
    console_error_panic_hook::set_once();
    let app_container = ultron::sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(App::new(), &app_container);
}
