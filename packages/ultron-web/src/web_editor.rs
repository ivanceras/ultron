use crate::context_menu::{self, Menu, MenuAction};
use crate::util;
use css_colors::{rgba, Color, RGBA};
use sauron::{
    dom::Measurements, html::attributes::*, html::events::*, html::*, jss_ns_pretty,
    wasm_bindgen::JsCast, wasm_bindgen_futures::JsFuture, *,
    web_sys::HtmlElement,
};
use std::cell::RefCell;
use std::rc::Rc;
pub use ultron_core;
use ultron_core::{
    editor, nalgebra::Point2, Ch, Editor, Options, SelectionMode, Style, TextBuffer, TextEdit,
    TextHighlighter,
};
use selection::SelectionSplits;
pub use mouse_cursor::MouseCursor;

mod selection;
mod mouse_cursor;
pub mod custom_element;

pub const COMPONENT_NAME: &str = "ultron";
pub const CH_WIDTH: u32 = 7;
pub const CH_HEIGHT: u32 = 16;

#[derive(Debug, Clone)]
pub enum Msg {
    EditorMounted(MountEvent),
    /// Discard current editor content if any, and use this new value
    /// This is triggered from the top-level DOM of this component
    ChangeValue(String),
    /// Syntax token is changed
    ChangeSyntax(String),
    /// Change the theme of the editor
    ChangeTheme(String),
    CursorMounted(MountEvent),
    Keydown(web_sys::KeyboardEvent),
    Mouseup(web_sys::MouseEvent),
    Click(web_sys::MouseEvent),
    Mousedown(web_sys::MouseEvent),
    Mousemove(web_sys::MouseEvent),
    Measurements(Measurements),
    Focused(web_sys::FocusEvent),
    Blur(web_sys::FocusEvent),
    ContextMenu(web_sys::MouseEvent),
    ContextMenuMsg(context_menu::Msg),
    ScrollCursorIntoView,
    MenuAction(MenuAction),
    /// set focus to the editor
    SetFocus,
    NoOp,
}

#[derive(Debug)]
pub enum Command {
    EditorCommand(editor::Command),
    /// execute paste text
    PasteTextBlock(String),
    MergeText(String),
    /// execute copy text
    CopyText,
    /// execute cut text
    CutText,
}

/// rename this to WebEditor
pub struct WebEditor<XMSG> {
    options: Options,
    pub editor: Editor<XMSG>,
    editor_element: Option<web_sys::Element>,
    /// the host element the web editor is mounted to, when mounted as a custom web component
    host_element: Option<web_sys::Element>,
    cursor_element: Option<web_sys::Element>,
    mouse_cursor: MouseCursor,
    measure: Measure,
    is_selecting: bool,
    text_highlighter: Rc<RefCell<TextHighlighter>>,
    /// lines of highlighted ranges
    highlighted_lines: Rc<RefCell<Vec<Vec<(Style, Vec<Ch>)>>>>,
    animation_frame_handles: Vec<i32>,
    background_task_handles: Vec<i32>,
    pub is_focused: bool,
    context_menu: Menu<Msg>,
    show_context_menu: bool,
}

impl<XMSG> Default for WebEditor<XMSG>{
    fn default() -> Self {
        Self::from_str(Options::default(), "")
    }
}

impl From<editor::Command> for Command {
    fn from(ecommand: editor::Command) -> Self {
        Self::EditorCommand(ecommand)
    }
}


#[derive(Default)]
struct Measure {
    average_dispatch: Option<f64>,
    last_dispatch: Option<f64>,
}

impl<XMSG> WebEditor<XMSG> {
    pub fn from_str(options: Options, content: &str) -> Self {
        let editor = Editor::from_str(options.clone(), content);
        let mut text_highlighter = TextHighlighter::default();
        if let Some(theme_name) = &options.theme_name {
            text_highlighter.select_theme(theme_name);
        }
        text_highlighter.set_syntax_token(&options.syntax_token);
        let highlighted_lines = Rc::new(RefCell::new(Self::highlight_lines(
            &editor.text_edit,
            &mut text_highlighter,
        )));
        WebEditor {
            options,
            editor,
            editor_element: None,
            host_element: None,
            cursor_element: None,
            mouse_cursor: MouseCursor::default(),
            measure: Measure::default(),
            is_selecting: false,
            text_highlighter: Rc::new(RefCell::new(text_highlighter)),
            highlighted_lines,
            animation_frame_handles: vec![],
            background_task_handles: vec![],
            is_focused: false,
            context_menu: Menu::new().on_activate(Msg::MenuAction),
            show_context_menu: false,
        }
    }

    pub fn set_syntax_token(&mut self, syntax_token: &str){
        self.text_highlighter.borrow_mut().set_syntax_token(syntax_token);
        self.rehighlight_all();
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        self.text_highlighter.borrow_mut().select_theme(theme_name);
        self.rehighlight_all();
    }

    pub fn add_on_change_listener<F>(&mut self, f: F)
    where
        F: Fn(String) -> XMSG + 'static,
    {
        self.editor.add_on_change_listener(f);
    }

    pub fn add_on_change_notify<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        self.editor.add_on_change_notify(f);
    }

    pub fn get_content(&self) -> String {
        self.editor.get_content()
    }
}

impl<XMSG> Component<Msg, XMSG> for WebEditor<XMSG> {

    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::EditorMounted(mount_event) => {
                log::info!("Web editor is mounted..");
                let mount_element: web_sys::Element = mount_event.target_node.unchecked_into();
                let root_node = mount_element.get_root_node();
                if let Some(shadow_root) = root_node.dyn_ref::<web_sys::ShadowRoot>(){
                    let host_element = shadow_root.host();
                    self.host_element = Some(host_element);
                }
                self.editor_element = Some(mount_element);
                Effects::none()
            }
            Msg::ChangeValue(content) => {
                self.process_commands([editor::Command::SetContent(content).into()]);
                Effects::none()
            }
            Msg::ChangeSyntax(syntax_token) => {
                self.set_syntax_token(&syntax_token);
                Effects::none()
            }
            Msg::ChangeTheme(theme_name) => {
                self.set_theme(&theme_name);
                Effects::none()
            }
            Msg::CursorMounted(mount_event) => {
                let cursor_element: web_sys::Element = mount_event.target_node.unchecked_into();
                self.cursor_element = Some(cursor_element);
                Effects::none()
            }
            Msg::Click(me) => {
                let client_x = me.client_x();
                let client_y = me.client_y();
                let cursor = self.client_to_grid_clamped(client_x, client_y);
                let msgs = self.editor.process_commands([editor::Command::SetPosition(cursor)]);
                Effects::new(vec![], msgs)
            }
            Msg::Mousedown(me) => {
                log::info!("mouse down event in ultron..");
                let client_x = me.client_x();
                let client_y = me.client_y();
                let is_primary_btn = me.button() == 0;
                if is_primary_btn {
                    //self.editor.clear_selection();
                    self.is_selecting = true;
                    let cursor = self.client_to_grid_clamped(client_x, client_y);
                    if self.is_selecting && !self.show_context_menu {
                        self.editor.set_selection_start(cursor);
                    }
                    let msgs = self
                        .editor
                        .process_commands([editor::Command::SetPosition(cursor)]);
                    Effects::new(vec![], msgs).measure()
                } else {
                    Effects::none()
                }
            }
            Msg::Mousemove(me) => {
                let client_x = me.client_x();
                let client_y = me.client_y();
                let cursor = self.client_to_grid_clamped(client_x, client_y);
                if self.is_selecting && !self.show_context_menu {
                    let selection = self.editor.selection();
                    if let Some(start) = selection.start {
                        self.editor.set_selection_end(cursor);
                        let msgs = self
                            .editor
                            .process_commands([editor::Command::SetSelection(start, cursor)]);
                        Effects::new(vec![], msgs).measure()
                    } else {
                        Effects::none()
                    }
                } else {
                    Effects::none()
                }
            }
            Msg::Mouseup(me) => {
                let client_x = me.client_x();
                let client_y = me.client_y();
                let is_primary_btn = me.button() == 0;
                if is_primary_btn {
                    let cursor = self.client_to_grid_clamped(client_x, client_y);
                    self.editor
                        .process_commands([editor::Command::SetPosition(cursor)]);

                    if self.is_selecting {
                        self.is_selecting = false;
                        self.editor.set_selection_end(cursor);
                        let selection = self.editor.selection();
                        if let (Some(start), Some(end)) = (selection.start, selection.end) {
                            let msgs = self
                                .editor
                                .process_commands([editor::Command::SetSelection(start, end)]);
                            Effects::new(vec![], msgs)
                        } else {
                            Effects::none()
                        }
                    } else {
                        Effects::none()
                    }
                } else {
                    Effects::none()
                }
            }
            Msg::Keydown(ke) => self.process_keypress(&ke),
            Msg::Measurements(measure) => {
                self.update_measure(measure);
                Effects::none()
            }
            Msg::Focused(_fe) => {
                self.is_focused = true;
                Effects::none()
            }
            Msg::SetFocus => {
                self.is_focused = true;
                if let Some(editor_element) = &self.editor_element{
                    let html_elm: &HtmlElement = editor_element.unchecked_ref();
                    html_elm.focus().expect("element must focus");
                }
                Effects::none()
            }
            Msg::Blur(_fe) => {
                self.is_focused = false;
                Effects::none()
            }
            Msg::ContextMenu(me) => {
                self.show_context_menu = true;
                let (start, _end) = self.bounding_rect().expect("must have a bounding rect");
                let x = me.client_x() - start.x as i32;
                let y = me.client_y() - start.y as i32;
                let (msgs, _) = self
                    .context_menu
                    .update(context_menu::Msg::ShowAt(Point2::new(x, y)))
                    .map_msg(Msg::ContextMenuMsg)
                    .unzip();
                Effects::new(msgs, [])
            }
            Msg::ContextMenuMsg(cm_msg) => {
                let (msgs, xmsg) = self.context_menu.update(cm_msg).unzip();
                Effects::new(
                    xmsg.into_iter()
                        .chain(msgs.into_iter().map(Msg::ContextMenuMsg)),
                    [],
                )
            }
            Msg::ScrollCursorIntoView => {
                if self.options.scroll_cursor_into_view {
                    let cursor_element = self.cursor_element.as_ref().unwrap();
                    let mut options = web_sys::ScrollIntoViewOptions::new();
                    options.behavior(web_sys::ScrollBehavior::Smooth);
                    options.block(web_sys::ScrollLogicalPosition::Center);
                    options.inline(web_sys::ScrollLogicalPosition::Center);
                    cursor_element.scroll_into_view_with_scroll_into_view_options(&options);
                }
                Effects::none()
            }
            Msg::MenuAction(menu_action) => {
                self.show_context_menu = false;
                match menu_action {
                    MenuAction::Undo => {
                        self.process_command(Command::EditorCommand(editor::Command::Undo));
                    }
                    MenuAction::Redo => {
                        self.process_command(Command::EditorCommand(editor::Command::Redo));
                    }
                    MenuAction::Cut => {
                        self.cut_selected_text_to_clipboard();
                    }
                    MenuAction::Copy => {
                        self.copy_selected_text_to_clipboard();
                    }
                    MenuAction::Paste => todo!(),
                    MenuAction::Delete => todo!(),
                    MenuAction::SelectAll => {
                        self.process_command(Command::EditorCommand(editor::Command::SelectAll));
                        log::info!("selected text: {:?}", self.selected_text());
                    }
                }
                Effects::none()
            }
            Msg::NoOp => Effects::none()
        }
    }

    fn view(&self) -> Node<Msg> {
        let enable_context_menu = self.options.enable_context_menu;
        let enable_keypresses = self.options.enable_keypresses;
        let enable_click = self.options.enable_click;
        div(
            [
                class(COMPONENT_NAME),
                classes_flag_namespaced(
                    COMPONENT_NAME,
                    [("occupy_container", self.options.occupy_container)],
                ),
                on_mount(Msg::EditorMounted),
                attributes::tabindex(1),
                on_keydown(move|ke| {
                    if enable_keypresses{
                        ke.prevent_default();
                        ke.stop_propagation();
                        Msg::Keydown(ke)
                    }else{
                        Msg::NoOp
                    }
                }),
                on_click(move|me|{
                    if enable_click{
                        Msg::Click(me)
                    }else{
                        Msg::NoOp
                    }
                }),
                tabindex(0),
                on_focus(Msg::Focused),
                on_blur(Msg::Blur),
                on_contextmenu(move|me| {
                    if enable_context_menu{
                        me.prevent_default();
                        me.stop_propagation();
                        Msg::ContextMenu(me)
                    }else{
                        Msg::NoOp
                    }
                }),
                style! {
                    cursor: self.mouse_cursor.to_str(),
                },
            ],
            [
                if self.options.use_syntax_highlighter {
                    self.view_highlighted_lines()
                } else {
                    self.plain_view()
                },
                view_if(self.options.show_status_line, self.view_status_line()),
                view_if(
                    self.is_focused && self.options.show_cursor,
                    self.view_cursor(),
                ),
                view_if(
                    self.is_focused && self.show_context_menu,
                    self.context_menu.view().map_msg(Msg::ContextMenuMsg),
                ),
            ],
        )
    }


    fn style(&self) -> String {
        let user_select = if self.options.allow_text_selection {
            "text"
        } else {
            "none"
        };
        let main = jss_ns_pretty! {COMPONENT_NAME,
            ".": {
                position: "relative",
                font_size: px(14),
                white_space: "normal",
                user_select: user_select,
                "-webkit-user-select": user_select,
            },

            ".occupy_container": {
                width: percent(100),
                height: "auto",
            },

            "pre code":{
                white_space: "pre",
                word_spacing: "normal",
                word_break: "normal",
                word_wrap: "normal",
            },

            ".code_wrapper": {
                margin: 0,
            },

            ".code": {
                position: "relative",
                font_size: px(14),
                display: "block",
                // to make the background color extend to the longest line, otherwise only the
                // longest lines has a background-color leaving the shorter lines ugly
                min_width: "max-content",
                user_select: user_select,
                "-webkit-user-select": user_select,
                font_family: "Iosevka Fixed",
            },

            ".line_block": {
                display: "block",
                height: px(CH_HEIGHT),
            },

            // number and line
            ".number__line": {
                display: "flex",
                height: px(CH_HEIGHT),
            },

            // numbers
            ".number": {
                flex: "none", // dont compress the numbers
                text_align: "right",
                background_color: "#ddd",
                padding_right: px(CH_WIDTH as f32 * self.numberline_padding_wide() as f32),
                height: px(CH_HEIGHT),
                display: "inline-block",
                user_select: "none",
                "-webkit-user-select": "none",
            },
            ".number_wide1 .number": {
                width: px(CH_WIDTH),
            },
            // when line number is in between: 10 - 99
            ".number_wide2 .number": {
                width: px(2 * CH_WIDTH),
            },
            // when total lines is in between 100 - 999
            ".number_wide3 .number": {
                width: px(3 * CH_WIDTH),
            },
            // when total lines is in between 1000 - 9000
            ".number_wide4 .number": {
                width: px(4 * CH_WIDTH),
            },
            // 10000 - 90000
            ".number_wide5 .number": {
                width: px(5 * CH_WIDTH),
            },

            // line content
            ".line": {
                flex: "none", // dont compress lines
                height: px(CH_HEIGHT),
                display: "block",
                user_select: user_select,
                "-webkit-user-select": user_select,
            },

            ".line span::selection": {
                background_color: self.selection_background().to_css(),
            },

            ".line .selected": {
               background_color: self.selection_background().to_css(),
               //background_color: rgba(221, 72, 20, 1.0).to_css(),
            },

            ".status": {
                position: "fixed",
                bottom: 0,
                display: "flex",
                flex_direction: "row",
                user_select: "none",
                font_family: "Iosevka Fixed",
            },

            ".virtual_cursor": {
                position: "absolute",
                width: px(CH_WIDTH),
                height: px(CH_HEIGHT),
                border_width: px(1),
                border_color: self.cursor_border().to_css(),
                opacity: 1,
                border_style: "solid",
            },

            ".cursor_center":{
                width: percent(100),
                height: percent(100),
                background_color: self.cursor_color().to_css(),
                opacity: percent(50),
                animation: "cursor_blink-anim 1000ms step-end infinite",
            },

            "@keyframes cursor_blink-anim": {
              "0%": {
                opacity: percent(0),
              },
              "25%": {
                opacity: percent(25)
              },
              "50%": {
                opacity: percent(100),
              },
              "75%": {
                opacity: percent(75)
              },
              "100%": {
                opacity: percent(0),
              },
            },
        };

        [main, self.context_menu.style()].join("\n")
    }
}

impl<XMSG> WebEditor<XMSG> {
    fn update_measure(&mut self, measure: Measurements) {
        if let Some(average_dispatch) = self.measure.average_dispatch.as_mut() {
            *average_dispatch = (*average_dispatch + measure.total_time) / 2.0;
        } else {
            self.measure.average_dispatch = Some(measure.total_time);
        }
        self.measure.last_dispatch = Some(measure.total_time);
    }

    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }

    pub fn get_char(&self,loc: Point2<usize>) -> Option<char> {
        self.editor.get_char(loc)
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.editor.get_position()
    }


  fn rehighlight_all(&mut self) {
        self.text_highlighter.borrow_mut().reset();
        *self.highlighted_lines.borrow_mut() = Self::highlight_lines(
            &self.editor.text_edit,
            &mut self.text_highlighter.borrow_mut(),
        );
    }

    /// rehighlight from 0 to the end of the visible lines
    pub fn rehighlight_visible_lines(&mut self) {
        if let Some((_top, end)) = self.visible_lines(){
            let text_highlighter = self.text_highlighter.clone();
            let highlighted_lines = self.highlighted_lines.clone();
            let lines = self.editor.text_edit.lines();
            for handle in self.animation_frame_handles.drain(..) {
                //cancel the old ones
                sauron::dom::util::cancel_animation_frame(handle).expect("must cancel");
            }
            let closure = move || {
                let mut text_highlighter = text_highlighter.borrow_mut();
                text_highlighter.reset();
                // Note: we are just starting from the very top.
                // The alternative would be to save parse state, and start to the line
                // where the parse state didn't change at that location, but that would be a much
                // complex code
                let start = 0;
                let new_highlighted_lines = lines.iter().skip(start).take(end - start).map(|line| {
                    text_highlighter
                        .highlight_line(line)
                        .expect("must highlight")
                        .into_iter()
                        .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
                        .collect()
                });

                for (line, new_highlight) in highlighted_lines
                    .borrow_mut()
                    .iter_mut()
                    .skip(start)
                    .zip(new_highlighted_lines)
                {
                    *line = new_highlight;
                }
            };

            let handle =
                sauron::dom::util::request_animation_frame(closure).expect("must have a handle");

            self.animation_frame_handles.push(handle);
        }else{
            self.rehighlight_all();
        }
    }

    /// rehighlight the rest of the lines that are not visible
    pub fn rehighlight_non_visible_lines_in_background(&mut self) {
        if let Some((_top, end)) = self.visible_lines(){
            for handle in self.background_task_handles.drain(..) {
                sauron::dom::util::cancel_timeout_callback(handle).expect("cancel timeout");
            }
            let text_highlighter = self.text_highlighter.clone();
            let highlighted_lines = self.highlighted_lines.clone();
            let lines = self.editor.text_edit.lines();
            let closure = move || {
                let mut text_highlighter = text_highlighter.borrow_mut();

                let new_highlighted_lines = lines.iter().skip(end).map(|line| {
                    text_highlighter
                        .highlight_line(line)
                        .expect("must highlight")
                        .into_iter()
                        .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
                        .collect()
                });

                // TODO: there could be a bug here, what if the new and old highlighted lines have
                // different length, it will only iterate to which ever is the shorted length.
                for (line, new_highlight) in highlighted_lines
                    .borrow_mut()
                    .iter_mut()
                    .skip(end)
                    .zip(new_highlighted_lines)
                {
                    *line = new_highlight;
                }
            };

            let handle =
                sauron::dom::util::request_timeout_callback(closure, 1_000).expect("timeout handle");
            self.background_task_handles.push(handle);
        }else{
            self.rehighlight_all();
        }
    }

    pub fn keyevent_to_command(ke: &web_sys::KeyboardEvent) -> Option<Command> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            let c = key.chars().next().expect("must be only 1 chr");
            let command = match c {
                'c' if is_ctrl => Command::CopyText,
                'x' if is_ctrl => Command::CutText,
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    Command::PasteTextBlock(String::new())
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        Command::EditorCommand(editor::Command::Redo)
                    } else {
                        Command::EditorCommand(editor::Command::Undo)
                    }
                }
                'r' if is_ctrl => Command::EditorCommand(editor::Command::Redo),
                'a' if is_ctrl => Command::EditorCommand(editor::Command::SelectAll),
                _ => Command::EditorCommand(editor::Command::InsertChar(c)),
            };

            Some(command)
        } else {
            let editor_command = match &*key {
                "Tab" => Some(editor::Command::IndentForward),
                "Enter" => Some(editor::Command::BreakLine),
                "Backspace" => Some(editor::Command::DeleteBack),
                "Delete" => Some(editor::Command::DeleteForward),
                "ArrowUp" => Some(editor::Command::MoveUp),
                "ArrowDown" => Some(editor::Command::MoveDown),
                "ArrowLeft" => Some(editor::Command::MoveLeft),
                "ArrowRight" => Some(editor::Command::MoveRight),
                "Home" => Some(editor::Command::MoveLeftStart),
                "End" => Some(editor::Command::MoveRightEnd),
                _ => None,
            };
            editor_command.map(Command::EditorCommand)
        }
    }

    /// make this into keypress to command
    pub fn process_keypress(&mut self, ke: &web_sys::KeyboardEvent) -> Effects<Msg, XMSG> {
        if let Some(command) = Self::keyevent_to_command(ke) {
            let msgs = self.process_commands([command]);
            Effects::new(vec![Msg::ScrollCursorIntoView], msgs).measure()
        } else {
            Effects::none()
        }
    }

    pub fn process_commands(&mut self, commands: impl IntoIterator<Item = Command>) -> Vec<XMSG> {
        let results: Vec<bool> = commands
            .into_iter()
            .map(|command| self.process_command(command))
            .collect();
        if results.into_iter().any(|v| v) {
            let xmsgs = self.editor.emit_on_change_listeners();
            if self.options.use_syntax_highlighter{
                self.rehighlight_visible_lines();
                self.rehighlight_non_visible_lines_in_background();
            }
            if let Some(host_element) = self.host_element.as_ref(){
                host_element.set_attribute("content", &self.get_content()).expect("set attr content");
                host_element.dispatch_event(&InputEvent::create_web_event_composed()).expect("dispatch event");
            }
            xmsgs
        } else {
            vec![]
        }
    }

    pub fn highlight_lines(
        text_edit: &TextEdit,
        text_highlighter: &mut TextHighlighter,
    ) -> Vec<Vec<(Style, Vec<Ch>)>> {
        text_edit
            .lines()
            .iter()
            .map(|line| {
                text_highlighter
                    .highlight_line(line)
                    .expect("must highlight")
                    .into_iter()
                    .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
                    .collect()
            })
            .collect()
    }

    /// insert the newly typed character to the highlighted line
    /// Note: This is a hacky way to have a visual feedback for the users to see
    /// the typed letter, the highlighter will eventually color it when it is done running
    fn insert_to_highlighted_line(&mut self, ch: char) {
        let cursor = self.get_position();
        let line = cursor.y;
        let column = cursor.x;
        if let Some(line) = self.highlighted_lines.borrow_mut().get_mut(line) {
            let mut width: usize = 0;
            for (_style, ref mut range) in line.iter_mut() {
                let range_width = range.iter().map(|range| range.width).sum::<usize>();
                if column > width && column <= width + range_width {
                    let diff = column - width;
                    range.insert(diff, Ch::new(ch));
                }
                width += range_width;
            }
        }
    }

    pub fn process_command(&mut self, command: Command) -> bool {
        match command {
            Command::EditorCommand(ecommand) => match ecommand {
                editor::Command::InsertChar(ch) => {
                    self.insert_to_highlighted_line(ch);
                    self.editor.process_command(ecommand)
                }
                _ => self.editor.process_command(ecommand),
            },
            Command::PasteTextBlock(text_block) => self
                .editor
                .process_command(editor::Command::PasteTextBlock(text_block)),
            Command::MergeText(text_block) => self
                .editor
                .process_command(editor::Command::MergeText(text_block)),
            Command::CopyText => self.copy_selected_text_to_clipboard(),
            Command::CutText => self.cut_selected_text_to_clipboard(),
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.editor.selected_text()
    }

    pub fn is_selected(&self, loc: Point2<i32>) -> bool {
        self.editor.is_selected(loc)
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.editor.cut_selected_text()
    }

    pub fn clear(&mut self) {
        self.editor.clear()
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.editor.set_selection(start, end);
    }

    pub fn copy_selected_text_to_clipboard(&self) -> bool {
        log::warn!("Copying text to clipboard..");
        if let Some(clipboard) = window().navigator().clipboard() {
            if let Some(selected_text) = self.selected_text() {
                log::info!("selected text: {selected_text}");
                let fut = JsFuture::from(clipboard.write_text(&selected_text));
                sauron::dom::spawn_local(async move {
                    fut.await.expect("must not error");
                });
                return true;
            } else {
                log::warn!("No selected text..")
            }
        } else {
            log::error!("Clipboard is not supported");
        }
        false
    }

    pub fn cut_selected_text_to_clipboard(&mut self) -> bool {
        log::warn!("Cutting text to clipboard");
        let ret = self.copy_selected_text_to_clipboard();
        self.cut_selected_text();
        ret
    }

    /// calculate the bounding rect of the editor using a DOM call [getBoundingClientRect](https://developer.mozilla.org/en-US/docs/Web/API/Element/getBoundingClientRect)
    pub fn bounding_rect(&self) -> Option<(Point2<f32>, Point2<f32>)> {
        if let Some(ref editor_element) = self.editor_element {
            let rect = editor_element.get_bounding_client_rect();
            let editor_x = rect.x().round() as f32;
            let editor_y = rect.y().round() as f32;
            let bottom = rect.bottom().round() as f32;
            let right = rect.right().round() as f32;
            Some((Point2::new(editor_x, editor_y), Point2::new(right, bottom)))
        } else {
            None
        }
    }

    /// check if this mouse client x and y is inside the editor bounds
    pub fn in_bounds(&self, client_x: f32, client_y: f32) -> bool {
        if let Some((start, end)) = self.bounding_rect() {
            client_x >= start.x && client_x <= end.x && client_y >= start.y && client_y <= end.y
        } else {
            false
        }
    }

    pub fn editor_offset(&self) -> Option<Point2<f32>> {
        if let Some((start, _end)) = self.bounding_rect() {
            Some(start)
        } else {
            None
        }
    }

    /// calculate the points relative to the editor bounding box
    pub fn relative_client(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let editor = self.editor_offset().expect("must have an editor offset");
        let x = client_x as f32 - editor.x;
        let y = client_y as f32 - editor.y;
        Point2::new(x.round() as i32, y.round() as i32)
    }

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    fn theme_background(&self) -> RGBA {
        let default = rgba(255, 255, 255, 1.0);
        self.text_highlighter
            .borrow()
            .active_theme()
            .settings
            .background
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn gutter_background(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.text_highlighter
            .borrow()
            .active_theme()
            .settings
            .gutter
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn gutter_foreground(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.text_highlighter
            .borrow()
            .active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn selection_background(&self) -> RGBA {
        let default = rgba(0, 0, 255, 1.0);
        self.text_highlighter
            .borrow()
            .active_theme()
            .settings
            .selection
            .map(util::to_rgba)
            .unwrap_or(default)
    }

    fn cursor_color(&self) -> RGBA {
        rgba(0, 0, 0, 1.0)
    }

    fn cursor_border(&self) -> RGBA {
        rgba(0, 0, 0, 1.0)
    }

    /// how wide the numberline based on the character lengths of the number
    fn numberline_wide_with_padding(&self) -> usize {
        if self.options.show_line_numbers {
            self.editor.total_lines().to_string().len() + self.numberline_padding_wide()
        } else {
            0
        }
    }

    pub fn total_lines(&self) -> usize {
        self.editor.total_lines()
    }

    /// convert screen coordinate to grid coordinate taking into account the editor element
    pub fn client_to_grid(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let numberline_wide_with_padding = self.numberline_wide_with_padding() as f32;
        let editor = self.editor_offset().expect("must have an editor offset");
        let col = (client_x as f32 - editor.x) / CH_WIDTH as f32 - numberline_wide_with_padding;
        let line = (client_y as f32 - editor.y) / CH_HEIGHT as f32;
        let x = col.floor() as i32;
        let y = line.floor() as i32;
        Point2::new(x, y)
    }

    /// convert screen coordinate to grid coordinate
    /// clamped negative values due to padding in the line number
    pub fn client_to_grid_clamped(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let cursor = self.client_to_grid(client_x, client_y);
        util::clamp_to_edge(cursor)
    }

    /// convert current cursor position to client coordinate relative to the editor div
    pub fn cursor_to_client(&self) -> Point2<f32> {
        let cursor = self.editor.get_position();
        Point2::new(
            (cursor.x + self.numberline_wide_with_padding()) as f32 * CH_WIDTH as f32,
            cursor.y as f32 * CH_HEIGHT as f32,
        )
    }

    /// calculate the width of the numberline including the padding
    fn number_line_with_padding_width(&self) -> f32 {
        self.numberline_wide_with_padding() as f32 * CH_WIDTH as f32
    }

    fn view_cursor(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let cursor = self.cursor_to_client();
        div(
            [
                class_ns("virtual_cursor"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                },
                on_mount(Msg::CursorMounted),
            ],
            [div([class_ns("cursor_center")], [])],
        )
    }

    /// the view for the status line
    pub fn view_status_line<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let cursor = self.editor.get_position();

        div(
            [
                class_ns("status"),
                style! {
                    background_color: self.gutter_background().to_css(),
                    color: self.gutter_foreground().to_css(),
                    height: px(self.status_line_height()),
                    left: px(self.number_line_with_padding_width())
                },
            ],
            [
                text!(" |> line: {}, col: {} ", cursor.y + 1, cursor.x + 1),
                text!(" |> version:{}", env!("CARGO_PKG_VERSION")),
                text!(" |> lines: {}", self.editor.total_lines()),
                if let Some((start, end)) = self.bounding_rect() {
                    text!(" |> bounding rect: {}->{}", start, end)
                } else {
                    text!("")
                },
                if let Some(visible_lines) = self.max_visible_lines() {
                    text!(" |> visible lines: {}", visible_lines)
                } else {
                    text!("")
                },
                if let Some((start, end)) = self.visible_lines() {
                    text!(" |> lines: ({},{})", start, end)
                } else {
                    text!("")
                },
                text!(" |> selection: {:?}", self.editor.selection()),
                if let Some(average_dispatch) = self.measure.average_dispatch {
                    text!(" |> average dispatch: {}ms", average_dispatch.round())
                } else {
                    text!("")
                },
                if let Some(last_dispatch) = self.measure.last_dispatch {
                    text!(" |> latest: {}ms", last_dispatch.round())
                } else {
                    text!("")
                },
            ],
        )
    }

    fn view_line_number<MSG>(&self, line_number: usize) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        view_if(
            self.options.show_line_numbers,
            span(
                [
                    class_ns("number"),
                    style! {
                        background_color: self.gutter_background().to_css(),
                        color: self.gutter_foreground().to_css(),
                    },
                ],
                [text(line_number)],
            ),
        )
    }

    /// calculate the maximum number of visible lines
    fn max_visible_lines(&self) -> Option<usize> {
        if let Some((start, end)) = self.bounding_rect() {
            Some(((end.y - start.y) / CH_HEIGHT as f32).round() as usize)
        } else {
            None
        }
    }

    /// calculate which lines are visible in the editor
    fn visible_lines(&self) -> Option<(usize, usize)> {
        if let Some((start, end)) = self.bounding_rect() {
            let ch_height = CH_HEIGHT as f32;
            let top = ((0.0 - start.y) / ch_height) as usize;
            let bottom = ((end.y - 2.0 * start.y) / ch_height) as usize;
            Some((top, bottom))
        } else {
            None
        }
    }

    fn view_highlighted_line<MSG>(
        &self,
        line_index: usize,
        line: &[(Style, Vec<Ch>)],
    ) -> Vec<Node<MSG>> {
        //let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let mut range_x: usize = 0;
        line.iter()
            .map(|(style, range)| {
                let range_str = String::from_iter(range.iter().map(|ch| ch.ch));

                let range_start = Point2::new(range_x, line_index);
                range_x += range.iter().map(|ch| ch.width).sum::<usize>();
                let range_end = Point2::new(range_x, line_index);

                let foreground = util::to_rgba(style.foreground).to_css();

                let selection_splits = match self.editor.text_edit.selection_reorder_casted() {
                    Some((start, end)) => {
                        // selection end points is only on the same line
                        let selection_in_same_line = start.y == end.y;
                        // this line is on the first line of selection
                        let selection_start_within_first_line = line_index == start.y;
                        // this line is on the last line of selection
                        let selection_end_within_last_line = line_index == end.y;
                        // this line is in between the selection end points
                        let line_within_selection = line_index > start.y && line_index < end.y;
                        let line_outside_selection = line_index < start.y || line_index > end.y;

                        // the start selection is within this range  location
                        let selection_start_within_range_start = start.x >= range_start.x;
                        // the end selection is within this range location
                        let selection_end_within_range_end = end.x <= range_end.x;
                        // both selection endpoints is inside this range
                        let selection_within_range =
                            start.x >= range_start.x && end.x <= range_end.x;

                        // range is in the right side of selection start
                        let range_in_right_of_selection_start =
                            range_start.x >= start.x && range_end.x >= start.x;
                        let range_in_left_of_selection_end =
                            range_start.x <= end.x && range_end.x <= end.x;
                        let range_in_right_of_selection_end =
                            range_start.x > end.x && range_end.x > end.x;

                        let text_buffer = TextBuffer::from_ch(&[range]);

                        if line_within_selection {
                            SelectionSplits::SelectAll(range_str)
                        } else if line_outside_selection {
                            SelectionSplits::NotSelected(range_str)
                        } else if selection_in_same_line {
                            let range_within_selection =
                                range_start.x >= start.x && range_end.x <= end.x;
                            if range_within_selection {
                                SelectionSplits::SelectAll(range_str)
                            } else if selection_within_range {
                                // the first is plain
                                // the second is selected
                                // the third is plain
                                let break1 = Point2::new(start.x - range_start.x, 0);
                                let break1 = text_buffer.clamp_position(break1);
                                let break2 = Point2::new(end.x - range_start.x, 0);
                                let break2 = text_buffer.clamp_position(break2);
                                let (first, second, third) =
                                    text_buffer.split_line_at_2_points(break1, break2);
                                SelectionSplits::SelectMiddle(first, second, third)
                            } else if selection_start_within_range_start {
                                let break1 = Point2::new(start.x - range_start.x, 0);
                                let break1 = text_buffer.clamp_position(break1);
                                let (first, second) = text_buffer.split_line_at_point(break1);
                                SelectionSplits::SelectRight(first, second)
                            } else if range_in_right_of_selection_end {
                                SelectionSplits::NotSelected(range_str)
                            } else if selection_end_within_range_end {
                                // the first is selected
                                // the second is plain
                                let break1 = Point2::new(end.x - range_start.x, 0);
                                let break1 = text_buffer.clamp_position(break1);
                                let (first, second) = text_buffer.split_line_at_point(break1);
                                SelectionSplits::SelectLeft(first, second)
                            } else {
                                SelectionSplits::NotSelected(range_str)
                            }
                        } else if selection_start_within_first_line {
                            if range_in_right_of_selection_start {
                                SelectionSplits::SelectAll(range_str)
                            } else if selection_start_within_range_start {
                                let break1 = Point2::new(start.x - range_start.x, 0);
                                let break1 = text_buffer.clamp_position(break1);
                                let (first, second) = text_buffer.split_line_at_point(break1);
                                SelectionSplits::SelectRight(first, second)
                            } else {
                                SelectionSplits::NotSelected(range_str)
                            }
                        } else if selection_end_within_last_line {
                            if range_in_left_of_selection_end {
                                SelectionSplits::SelectAll(range_str)
                            } else if range_in_right_of_selection_end {
                                SelectionSplits::NotSelected(range_str)
                            } else if selection_end_within_range_end {
                                // the first is selected
                                // the second is plain
                                let break1 = Point2::new(end.x - range_start.x, 0);
                                let break1 = text_buffer.clamp_position(break1);
                                let (first, second) = text_buffer.split_line_at_point(break1);
                                SelectionSplits::SelectLeft(first, second)
                            } else {
                                SelectionSplits::NotSelected(range_str)
                            }
                        } else {
                            SelectionSplits::NotSelected(range_str)
                        }
                    }
                    None => SelectionSplits::NotSelected(range_str),
                };
                selection_splits.view_with_style(style! { color: foreground })
            })
            .collect()
    }

    // highlighted view
    pub fn view_highlighted_lines<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let class_number_wide = format!("number_wide{}", self.editor.numberline_wide());

        let code_attributes = [
            class_ns("code"),
            class_ns(&class_number_wide),
            style! {background: self.theme_background().to_css()},
        ];

        let highlighted_lines = self.highlighted_lines.borrow();
        let rendered_lines = highlighted_lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                div([class_ns("line")], {
                    [self.view_line_number(line_index + 1)]
                        .into_iter()
                        .chain(self.view_highlighted_line(line_index, line))
                        .collect::<Vec<_>>()
                })
            });

        if self.options.use_for_ssg {
            // using div works well when select-copying for both chrome and firefox
            // this is ideal for statis site generation highlighting
            div(code_attributes, rendered_lines)
        } else {
            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                [class_ns("code_wrapper")],
                [code(code_attributes, rendered_lines)],
            )
        }
    }

    pub fn plain_view<MSG>(&self) -> Node<MSG> {
        self.view_text_edit()
    }

    /// height of the status line which displays editor infor such as cursor location
    pub fn status_line_height(&self) -> i32 {
        30
    }

    fn view_line_with_linear_selection<MSG>(&self, line_index: usize, line: String) -> Node<MSG> {
        let line_width = self.editor.text_edit.text_buffer.line_width(line_index);
        let line_end = Point2::new(line_width, line_index);

        let selection_splits = match self.editor.text_edit.selection_reorder_casted() {
            Some((start, end)) => {
                // this line is in between the selection end points
                let in_inner_line = line_index > start.y && line_index < end.y;

                if in_inner_line {
                    SelectionSplits::SelectAll(line)
                } else {
                    // selection end points is only on the same line
                    let in_same_line = start.y == end.y;
                    // this line is on the first line of selection
                    let in_first_line = line_index == start.y;
                    // this line is on the last line of selection
                    let in_last_line = line_index == end.y;
                    let text_buffer = &self.editor.text_edit.text_buffer;
                    if in_first_line {
                        // the first part is the plain
                        // the second part is the highlighted
                        let break1 = Point2::new(start.x, line_index);
                        let break1 = text_buffer.clamp_position(break1);
                        let (first, second) = text_buffer.split_line_at_point(break1);
                        if in_same_line {
                            // the third part will be in plain
                            let break2 = Point2::new(end.x, line_end.y);
                            let break2 = text_buffer.clamp_position(break2);
                            let (first, second, third) =
                                text_buffer.split_line_at_2_points(break1, break2);
                            SelectionSplits::SelectMiddle(first, second, third)
                        } else {
                            SelectionSplits::SelectRight(first, second)
                        }
                    } else if in_last_line {
                        // the first part is the highlighted
                        // the second part is plain
                        let break1 = Point2::new(end.x, line_index);
                        let break1 = text_buffer.clamp_position(break1);
                        let (first, second) = text_buffer.split_line_at_point(break1);
                        SelectionSplits::SelectLeft(first, second)
                    } else {
                        SelectionSplits::NotSelected(line)
                    }
                }
            }
            None => SelectionSplits::NotSelected(line),
        };
        selection_splits.view()
    }

    //TODO: this needs fixing, as we are accessing characters that may not not in the right index
    fn view_line_with_block_selection<MSG>(&self, line_index: usize, line: String) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

        let default_view = span([], [text(&line)]);
        match self.editor.text_edit.selection_normalized_casted() {
            Some((start, end)) => {
                let text_buffer = &self.editor.text_edit.text_buffer;

                // there will be 3 parts
                // the first one is plain
                // the second one is highlighted
                // the third one is plain
                let break1 = Point2::new(start.x, line_index);
                let break1 = text_buffer.clamp_position(break1);

                let break2 = Point2::new(end.x, line_index);
                let break2 = text_buffer.clamp_position(break2);
                let (first, second, third) = text_buffer.split_line_at_2_points(break1, break2);

                if line_index >= start.y && line_index <= end.y {
                    span(
                        [],
                        [
                            span([], [text(first)]),
                            span([class_ns("selected")], [text(second)]),
                            span([], [text(third)]),
                        ],
                    )
                } else {
                    default_view
                }
            }
            _ => default_view,
        }
    }

    pub fn view_text_edit<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let text_edit = &self.editor.text_edit;

        let class_number_wide = format!("number_wide{}", text_edit.numberline_wide());

        let code_attributes = [class_ns("code"), class_ns(&class_number_wide)];
        let rendered_lines = text_edit
            .lines()
            .into_iter()
            .enumerate()
            .map(|(line_index, line)| {
                let line_number = line_index + 1;
                div(
                    [class_ns("line")],
                    [
                        view_if(
                            self.options.show_line_numbers,
                            span([class_ns("number")], [text(line_number)]),
                        ),
                        match self.options.selection_mode {
                            SelectionMode::Linear => {
                                self.view_line_with_linear_selection(line_index, line)
                            }
                            SelectionMode::Block => {
                                self.view_line_with_block_selection(line_index, line)
                            }
                        },
                    ],
                )
            });

        if self.options.use_for_ssg {
            // using div works well when select-copying for both chrome and firefox
            // this is ideal for static site generation highlighting
            div(code_attributes, rendered_lines)
        } else {
            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                [class_ns("code_wrapper")],
                [code(code_attributes, rendered_lines)],
            )
        }
    }
}

pub fn view_text_buffer<MSG>(text_buffer: &TextBuffer, options: &Options) -> Node<MSG> {
    let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

    let class_number_wide = format!("number_wide{}", text_buffer.numberline_wide());

    let code_attributes = [class_ns("code"), class_ns(&class_number_wide)];
    let rendered_lines = text_buffer
        .lines()
        .into_iter()
        .enumerate()
        .map(|(line_index, line)| {
            let line_number = line_index + 1;
            div(
                [class_ns("line")],
                [
                    view_if(
                        options.show_line_numbers,
                        span([class_ns("number")], [text(line_number)]),
                    ),
                    // Note: this is important since text node with empty
                    // content seems to cause error when finding the dom in rust
                    span([], [text(line)]),
                ],
            )
        });

    if options.use_for_ssg {
        // using div works well when select-copying for both chrome and firefox
        // this is ideal for static site generation highlighting
        div(code_attributes, rendered_lines)
    } else {
        // using <pre><code> works well when copying in chrome
        // but in firefox, it creates a double line when select-copying the text
        // whe need to use <pre><code> in order for typing whitespace works.
        pre(
            [class_ns("code_wrapper")],
            [code(code_attributes, rendered_lines)],
        )
    }
}



