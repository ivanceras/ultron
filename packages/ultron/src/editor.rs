use crate::Options;
use crate::TextBuffer;
use crate::CH_HEIGHT;
use crate::CH_WIDTH;
use crate::COMPONENT_NAME;
use css_colors::Color;
use history::Recorded;
use nalgebra::Point2;
use sauron::html::attributes;
use sauron::html::units;
use sauron::jss::jss_ns;
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use sauron::Measurements;

pub(crate) mod action;
mod history;

#[derive(Clone, PartialEq, Debug)]
pub enum Msg {
    TextareaMounted(web_sys::Node),
    EditorMounted(web_sys::Node),
    /// keydown from window events
    WindowKeydown(web_sys::KeyboardEvent),
    /// Keydown from the hidden text area
    TextareaKeydown(web_sys::KeyboardEvent),
    MoveCursor(usize, usize),
    MoveCursorToLine(usize),
    StartSelection(usize, usize),
    EndSelection(usize, usize),
    StopSelection,
    ToSelection(usize, usize),
    Paste(String),
    CopiedSelected,
    Mouseup(i32, i32),
    Mousedown(i32, i32),
    Mousemove(i32, i32),
    SetMeasurement(Measurements),
    Scrolled((i32, i32)),
    WindowScrolled((i32, i32)),
    TextareaInput(String),
}

pub struct Editor<XMSG> {
    options: Options,
    text_buffer: TextBuffer,
    /// number of lines in a page, when paging up and down
    #[allow(unused)]
    page_size: usize,
    /// for undo and redo
    #[allow(unused)]
    recorded: Recorded,
    measurements: Option<Measurements>,
    scroll_top: f32,
    scroll_left: f32,
    window_scroll_top: f32,
    window_scroll_left: f32,
    /// Other components can listen to the an event.
    /// When the content of the text editor changes, the change listener will be emitted
    change_listeners: Vec<Callback<String, XMSG>>,
    change_notify_listeners: Vec<Callback<(), XMSG>>,
    hidden_textarea: Option<web_sys::HtmlTextAreaElement>,
    editor_element: Option<web_sys::Element>,
    composed_key: Option<char>,
    last_char_count: Option<usize>,
    is_selecting: bool,
    selection_start: Option<Point2<i32>>,
    selection_end: Option<Point2<i32>>,
}

impl<XMSG> Editor<XMSG> {
    pub fn from_str(options: Options, content: &str) -> Self {
        Editor {
            options: options.clone(),
            text_buffer: TextBuffer::from_str(options, content),
            page_size: 10,
            recorded: Recorded::new(),
            measurements: None,
            scroll_top: 0.0,
            scroll_left: 0.0,
            window_scroll_top: 0.0,
            window_scroll_left: 0.0,
            change_listeners: vec![],
            change_notify_listeners: vec![],
            hidden_textarea: None,
            editor_element: None,
            composed_key: None,
            last_char_count: None,
            is_selecting: false,
            selection_start: None,
            selection_end: None,
        }
    }
}

impl<XMSG> Component<Msg, XMSG> for Editor<XMSG> {
    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::EditorMounted(target_node) => {
                let element: &web_sys::Element = target_node.unchecked_ref();
                self.editor_element = Some(element.clone());
                Effects::none()
            }
            Msg::TextareaMounted(target_node) => {
                self.hidden_textarea = Some(target_node.unchecked_into());
                Effects::none()
            }
            Msg::WindowScrolled((scroll_top, scroll_left)) => {
                self.window_scroll_top = scroll_top as f32;
                self.window_scroll_left = scroll_left as f32;
                Effects::none()
            }
            Msg::Scrolled((scroll_top, scroll_left)) => {
                self.scroll_top = scroll_top as f32;
                self.scroll_left = scroll_left as f32;
                Effects::none()
            }
            Msg::TextareaInput(input) => {
                log::trace!("text are input: {:?}", input);
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

                if char_count == 1 && (was_cleared || char_count_decreased) {
                    let c = input.chars().next().expect("must be only 1 chr");
                    self.composed_key = Some(c);
                    if c == '\n' {
                        self.command_break_line();
                    } else {
                        self.command_insert_char(c);
                    }
                    self.clear_hidden_textarea();
                }
                self.last_char_count = Some(char_count);

                let extern_msgs = self.emit_on_change_listeners();
                Effects::with_external(extern_msgs).measure()
            }
            Msg::TextareaKeydown(ke) => {
                let is_ctrl = ke.ctrl_key();
                log::trace!("text area key down... {}", is_ctrl);
                // don't process key presses when
                // CTRL key is pressed.
                let key = ke.key();
                self.process_keypresses(&ke);
                if key.chars().count() == 1 {
                    let c = key.chars().next().expect("must be only 1 chr");
                    match c {
                        'c' if is_ctrl => {
                            let ret = self.command_copy();
                            log::trace!("copy works: {}", ret);
                        }
                        'x' if is_ctrl => {
                            let ret = self.command_cut();
                            log::trace!("cut works: {}", ret);
                        }
                        'v' if is_ctrl => {
                            log::trace!("pasting is handled");
                            self.clear_hidden_textarea();
                        }
                        _ => {
                            self.command_insert_char(c);
                            self.clear_hidden_textarea();
                        }
                    }
                }
                let extern_msgs = self.emit_on_change_listeners();
                Effects::with_external(extern_msgs).measure()
            }
            Msg::Mouseup(client_x, client_y) => {
                let cursor = self.client_to_cursor(client_x, client_y);
                self.command_set_position(cursor.x, cursor.y);
                self.selection_end = Some(cursor);
                if let (Some(start), Some(end)) =
                    (self.selection_start, self.selection_end)
                {
                    self.is_selecting = false;
                    self.command_set_selection(start, end);
                    // pre-emptively put the selection into the hidden textarea
                    self.set_hidden_textarea_with_selection();
                }

                if let Some(selected_text) = self.text_buffer.selected_text() {
                    log::trace!("selected text: \n{}", selected_text);
                }
                Effects::none().measure()
            }
            Msg::Mousedown(client_x, client_y) => {
                let cursor = self.client_to_cursor(client_x, client_y);
                self.is_selecting = true;
                self.selection_start = Some(cursor);
                self.command_set_position(cursor.x, cursor.y);
                Effects::none().measure()
            }
            Msg::Mousemove(client_x, client_y) => {
                if self.is_selecting {
                    let cursor = self.client_to_cursor(client_x, client_y);
                    self.selection_end = Some(cursor);

                    if let Some(start) = self.selection_start {
                        self.command_set_selection(start, cursor);
                    }
                    Effects::none().measure()
                } else {
                    Effects::none().no_render()
                }
            }
            Msg::Paste(text_content) => {
                self.command_insert_text(&text_content);
                let extern_msgs = self.emit_on_change_listeners();
                Effects::with_external(extern_msgs)
            }
            Msg::CopiedSelected => Effects::none(),
            Msg::MoveCursor(_line, _col) => Effects::none(),
            Msg::MoveCursorToLine(_line) => Effects::none(),
            Msg::StartSelection(_line, _col) => Effects::none(),
            Msg::ToSelection(_line, _col) => Effects::none(),
            Msg::EndSelection(_line, _col) => Effects::none(),
            Msg::StopSelection => Effects::none(),
            Msg::SetMeasurement(measurements) => {
                self.measurements = Some(measurements);
                Effects::none().no_render()
            }
            Msg::WindowKeydown(ke) => {
                let key = ke.key();
                self.process_keypresses(&ke);
                if key.chars().count() == 1 {
                    let c = key.chars().next().expect("must be only 1 chr");
                    self.text_buffer.command_insert_char(c);
                    self.text_buffer.rehighlight();
                }
                let extern_msgs = self.emit_on_change_listeners();
                Effects::with_external(extern_msgs)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        div(
            [
                class(COMPONENT_NAME),
                on_scroll(Msg::Scrolled),
                on_mount(|mount| Msg::EditorMounted(mount.target_node)),
            ],
            [
                self.view_hidden_textarea(),
                self.text_buffer.view(),
                view_if(self.options.show_status_line, self.view_status_line()),
                view_if(
                    self.options.show_cursor
                        && self.text_buffer.is_in_virtual_position(),
                    self.view_virtual_cursor(),
                ),
            ],
        )
    }

    fn style(&self) -> String {
        let css = jss_ns! {COMPONENT_NAME,
            ".": {
                position: "relative",
                font_size: px(14),
                cursor: "text",
                width: percent(100),
                height: percent(100),
                white_space: "normal",
            },

            "pre code":{
                white_space: "pre",
                word_spacing: "normal",
                word_break: "normal",
                word_wrap: "normal",
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

            ".hidden_textarea_wrapper": {
                overflow: "hidden",
                position: "relative",
                width: px(300),
                height: px(0),
            },

            ".status": {
                position: "sticky",
                bottom: 0,
                display: "flex",
                flex_direction: "flex-end",
                user_select: "none",
            },

        };

        [css, self.text_buffer.style()].join("\n")
    }
}

impl<XMSG> Editor<XMSG> {
    pub fn with_options(mut self, options: Options) -> Self {
        self.options = options.clone();
        self.text_buffer.set_options(options);
        self
    }

    fn command_insert_char(&mut self, ch: char) {
        self.text_buffer.command_insert_char(ch);
        self.text_buffer.rehighlight();
    }

    pub fn command_replace_char(&mut self, ch: char) {
        self.text_buffer.command_replace_char(ch);
        self.text_buffer.rehighlight();
    }

    fn command_break_line(&mut self) {
        self.text_buffer.command_break_line();
        self.text_buffer.rehighlight();
    }

    fn command_insert_text(&mut self, text: &str) {
        self.text_buffer.command_insert_text(text);
        self.text_buffer.rehighlight();
    }

    pub fn command_set_position(&mut self, cursor_x: i32, cursor_y: i32) {
        if self.text_buffer.in_bounds(Point2::new(cursor_x, cursor_y)) {
            self.text_buffer
                .set_position(cursor_x as usize, cursor_y as usize);
        }
    }

    fn command_set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        if self.text_buffer.in_bounds(start) && self.text_buffer.in_bounds(end)
        {
            let start = Point2::new(start.x as usize, start.y as usize);
            let end = Point2::new(end.x as usize, end.y as usize);
            self.text_buffer.set_selection(start, end);
        }
    }

    /// calls on 2 ways to copy
    /// either 1 should work
    /// returns true if it succeded
    fn command_copy(&self) -> bool {
        if self.textarea_exec_copy() {
            true
        } else {
            #[cfg(web_sys_unstable_apis)]
            #[cfg(feature = "with-navigator-clipboard")]
            self.copy_to_clipboard()
        }
    }

    /// try exec_cut, try cut to clipboard if the first fails
    /// This shouldn't execute both since cut is destructive.
    /// Returns true if it succeded
    fn command_cut(&mut self) -> bool {
        if self.textarea_exec_cut() {
            true
        } else {
            #[cfg(web_sys_unstable_apis)]
            #[cfg(feature = "with-navigator-clipboard")]
            self.cut_to_clipboard()
        }
    }

    /// set the content of the textarea to selection
    ///
    /// Note: This is necessary for webkit2.
    /// webkit2 doesn't seem to allow to fire the setting of textarea value, select and copy
    /// in the same animation frame.
    fn set_hidden_textarea_with_selection(&self) {
        if let Some(selected_text) = self.text_buffer.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                log::trace!("setting the value to textarea: {}", selected_text);
                hidden_textarea.set_value(&selected_text);
                log::trace!("textarea value: {}", hidden_textarea.value());
                hidden_textarea.select();
            }
        }
    }

    /// this is for newer browsers
    /// This doesn't work on webkit2
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn copy_to_clipboard(&self) -> bool {
        if let Some(selected_text) = self.text_buffer.selected_text() {
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

    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "with-navigator-clipboard")]
    fn cut_to_clipboard(&mut self) -> bool {
        if let Some(selected_text) = self.text_buffer.cut_selected_text() {
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

    /// execute copy on the selected textarea
    /// this works even on older browser
    fn textarea_exec_copy(&self) -> bool {
        use sauron::web_sys::HtmlDocument;

        if let Some(selected_text) = self.text_buffer.selected_text() {
            if let Some(ref hidden_textarea) = self.hidden_textarea {
                log::trace!("setting the value to textarea: {}", selected_text);
                hidden_textarea.set_value(&selected_text);
                log::trace!("textarea value: {}", hidden_textarea.value());
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
        use sauron::web_sys::HtmlDocument;

        if let Some(selected_text) = self.text_buffer.cut_selected_text() {
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

    fn process_keypresses(&mut self, ke: &web_sys::KeyboardEvent) {
        let key = ke.key();
        match &*key {
            "Tab" => {
                log::trace!("tab key is pressed");
                let tab = "    ";
                self.text_buffer.command_insert_text(tab);
                self.refocus_hidden_textarea();
            }
            "Enter" => {
                self.text_buffer.command_break_line();
            }
            "Backspace" => {
                self.text_buffer.command_delete_back();
            }
            "Delete" => {
                self.text_buffer.command_delete_forward();
            }
            "ArrowUp" => {
                self.text_buffer.move_up();
            }
            "ArrowDown" => {
                self.text_buffer.move_down();
            }
            "ArrowLeft" => {
                self.text_buffer.move_left();
            }
            "ArrowRight" => {
                self.text_buffer.move_right();
            }
            _ => (),
        }
    }

    /// Attach a callback to this editor where it is invoked when the content is changed.
    ///
    /// Note:The content is extracted into string and used as a parameter to the function.
    /// This may be a costly operation when the editor has lot of text on it.
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_listeners.push(cb);
        self
    }

    /// Attach an callback to this editor where it is invoked when the content is changed.
    /// The callback function just notifies the parent component that uses the Editor component.
    /// It will be up to the parent component to extract the content of the editor manually.
    ///
    /// This is intended to be used in a debounced or throttled functionality where the component
    /// decides when to do an expensive operation based on time and recency.
    ///
    ///
    pub fn on_change_notify<F>(mut self, f: F) -> Self
    where
        F: Fn(()) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.change_notify_listeners.push(cb);
        self
    }

    fn emit_on_change_listeners(&self) -> Vec<XMSG> {
        let mut extern_msgs: Vec<XMSG> = if !self.change_listeners.is_empty() {
            let content = self.text_buffer.to_string();
            self.change_listeners
                .iter()
                .map(|listener| listener.emit(content.clone()))
                .collect()
        } else {
            vec![]
        };

        let extern_notify_msgs: Vec<XMSG> =
            if !self.change_notify_listeners.is_empty() {
                self.change_notify_listeners
                    .iter()
                    .map(|notify| notify.emit(()))
                    .collect()
            } else {
                vec![]
            };
        extern_msgs.extend(extern_notify_msgs);
        extern_msgs
    }

    pub fn editor_offset(&self) -> Option<Point2<f32>> {
        if let Some(ref editor_element) = self.editor_element {
            log::trace!("calculating editor offset..");
            let rect = editor_element.get_bounding_client_rect();
            let editor_x = rect.x().round() as f32;
            let editor_y = rect.y().round() as f32;
            Some(Point2::new(editor_x, editor_y))
        } else {
            None
        }
    }

    /// convert screen coordinate to cursor position
    pub fn client_to_cursor(
        &self,
        client_x: i32,
        client_y: i32,
    ) -> Point2<i32> {
        let numberline_wide = self.text_buffer.get_numberline_wide() as f32;
        let editor = self.editor_offset().expect("must have an editor offset");
        let col =
            (client_x as f32 - editor.x) / CH_WIDTH as f32 - numberline_wide;
        let line = (client_y as f32 - editor.y) / CH_HEIGHT as f32;
        let x = col.floor() as i32;
        let y = line.floor() as i32;
        Point2::new(x, y)
    }

    /// convert current cursor position to client coordinate relative to the editor div
    pub fn cursor_to_client(&self) -> Point2<i32> {
        let numberline_wide = self.text_buffer.get_numberline_wide() as f32;
        let cursor = self.text_buffer.get_position();

        let top = cursor.y as i32 * CH_HEIGHT as i32;
        let left = (cursor.x as i32 + numberline_wide as i32) * CH_WIDTH as i32;

        Point2::new(left, top)
    }

    fn view_hidden_textarea(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.cursor_to_client();
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
                    on_keydown(Msg::TextareaKeydown),
                    focus(true),
                    autofocus(true),
                    attr("autocorrect", "off"),
                    autocapitalize("none"),
                    autocomplete("off"),
                    spellcheck("off"),
                    on_input(|input| Msg::TextareaInput(input.value)),
                ],
                [],
            )],
        )
    }

    fn refocus_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.focus().expect("must focus the textarea");
        }
    }

    fn clear_hidden_textarea(&self) {
        if let Some(element) = &self.hidden_textarea {
            element.set_value("");
        }
    }

    fn view_virtual_cursor(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.cursor_to_client();
        div(
            [
                class_ns("virtual_cursor"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                },
            ],
            [],
        )
    }

    fn view_status_line<Msg>(&self) -> Node<Msg> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let cursor = self.text_buffer.get_position();
        div(
            [
                class_ns("status"),
                if let Some(gutter_bg) = self.text_buffer.gutter_background() {
                    style! {
                        background_color: gutter_bg.to_css(),
                    }
                } else {
                    empty_attr()
                },
                if let Some(gutter_fg) = self.text_buffer.gutter_foreground() {
                    style! {
                        color: gutter_fg.to_css()
                    }
                } else {
                    empty_attr()
                },
            ],
            [
                text!("line: {}, col: {}  |", cursor.y + 1, cursor.x + 1),
                if let Some(measurements) = &self.measurements {
                    text!(
                        "patches: {} | nodes: {} | view time: {}ms | patch time: {}ms | update time: {}ms",
                        measurements.total_patches,
                        measurements.view_node_count,
                        measurements.build_view_took.round(),
                        measurements.dom_update_took.round(),
                        measurements.total_time.round()
                    )
                } else {
                    comment("")
                },
                if let (Some(start), Some(end)) =
                    (self.selection_start, self.selection_end)
                {
                    text!("| selection start: {} end: {}", start, end)
                } else {
                    text("| no selection")
                },
            ],
        )
    }
}
