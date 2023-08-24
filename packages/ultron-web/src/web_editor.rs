#![allow(unused)]
use crate::context_menu::{self, Menu};
use crate::util;
use css_colors::{rgba, Color, RGBA};
use sauron::prelude::*;
use sauron::html::node_list;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use ultron_core::{
    base_editor::Callback, nalgebra::Point2, BaseEditor, Ch, Style, TextBuffer,
    TextEdit, TextHighlighter,
};
use crate::Spinner;
use sauron::dom::{IdleCallbackHandle, IdleDeadline, request_idle_callback};
use web_sys::HtmlElement;

pub use crate::context_menu::MenuAction;
pub use crate::font_loader::FontSettings;
use crate::wasm_bindgen::JsCast;
use crate::{font_loader, FontLoader};
pub use mouse_cursor::MouseCursor;
pub use options::Options;
pub use ultron_core;
pub use ultron_core::{BaseOptions, Command};

mod mouse_cursor;

#[cfg(feature = "custom_element")]
pub mod custom_element;
mod options;

#[derive(Debug)]
pub enum Msg {
    EditorMounted(MountEvent),
    FontReady,
    FontLoaderMsg(font_loader::Msg),
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
pub enum Call {
    Command(Command),
    /// execute paste text
    PasteTextBlock(String),
    MergeText(String),
    /// execute copy text
    CopyText,
    /// execute cut text
    CutText,
    SelectAll,
    ClearSelection,
}

/// rename this to WebEditor
pub struct WebEditor<XMSG> {
    pub options: Options,
    font_loader: FontLoader<Msg>,
    pub base_editor: BaseEditor<XMSG>,
    editor_element: Option<web_sys::Element>,
    /// the host element the web editor is mounted to, when mounted as a custom web component
    host_element: Option<web_sys::Element>,
    cursor_element: Option<web_sys::Element>,
    mouse_cursor: MouseCursor,
    measure: Measure,
    is_selecting: bool,
    text_highlighter: Rc<RefCell<TextHighlighter>>,
    /// lines of highlighted ranges
    #[allow(clippy::type_complexity)]
    highlighted_lines: Rc<RefCell<Vec<Vec<(Style, Vec<Ch>)>>>>,
    highlight_task_handles: Vec<IdleCallbackHandle>,
    background_task_handles: Vec<IdleCallbackHandle>,
    pub is_focused: bool,
    context_menu: Menu<Msg>,
    show_context_menu: bool,
    is_fonts_ready: bool,
    /// emitted when the editor is ready
    /// meaning the fonts has been loaded and the editor has been mounted
    ready_listener: Vec<Callback<(), XMSG>>,
    is_background_highlighting_ongoing: Rc<AtomicBool>,
}

impl<XMSG> Default for WebEditor<XMSG> {
    fn default() -> Self {
        let options = Options::default();
        let mut text_highlighter = TextHighlighter::default();
        text_highlighter.set_syntax_token(&options.syntax_token);

        let mut font_loader = FontLoader::default();
        font_loader.on_fonts_ready(|| Msg::FontReady);

        Self {
            options,
            font_loader,
            base_editor: BaseEditor::default(),
            editor_element: None,
            host_element: None,
            cursor_element: None,
            mouse_cursor: MouseCursor::default(),
            measure: Measure::default(),
            is_selecting: false,
            text_highlighter: Rc::new(RefCell::new(text_highlighter)),
            highlighted_lines: Rc::new(RefCell::new(vec![])),
            highlight_task_handles: vec![],
            background_task_handles: vec![],
            is_focused: false,
            context_menu: Menu::new(),
            show_context_menu: false,
            is_fonts_ready: false,
            ready_listener: vec![],
            is_background_highlighting_ongoing: Rc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Default, Clone)]
struct Measure {
    average_dispatch: Option<f64>,
    last_dispatch: Option<f64>,
    detail: Option<Measurements>,
}

impl<XMSG> WebEditor<XMSG>
where
    XMSG: 'static,
{
    pub fn from_str(options: &Options, content: &str) -> Self {
        let base_editor = BaseEditor::from_str(&options.base_options, content);
        let mut text_highlighter = TextHighlighter::default();
        if let Some(theme_name) = &options.theme_name {
            text_highlighter.select_theme(theme_name);
        }
        text_highlighter.set_syntax_token(&options.syntax_token);
        let highlighted_lines = Rc::new(RefCell::new(Self::highlight_lines(
            base_editor.as_ref(),
            &mut text_highlighter,
        )));

        let mut font_loader = if let Some(font_settings) = &options.font_settings {
            FontLoader::new(font_settings)
        } else {
            // if no font settings is loaded, we use the iosevka font
            FontLoader::default()
        };
        font_loader.on_fonts_ready(|| Msg::FontReady);

        WebEditor {
            options: options.clone(),
            base_editor,
            font_loader,
            text_highlighter: Rc::new(RefCell::new(text_highlighter)),
            highlighted_lines,
            context_menu: Menu::new().on_activate(Msg::MenuAction),
            show_context_menu: false,
            ..Default::default()
        }
    }

}

impl<XMSG> Component<Msg, XMSG> for WebEditor<XMSG>
where
    XMSG: 'static,
{
    fn init(&mut self) -> Effects<Msg, XMSG> {
        self.font_loader.init().localize(Msg::FontLoaderMsg)
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::EditorMounted(mount_event) => {
                log::info!("Web editor is mounted..");
                let mount_element: web_sys::Element = mount_event.target_node.unchecked_into();
                mount_element.unchecked_ref::<HtmlElement>().focus().expect("mount_node should focus");
                let root_node = mount_element.get_root_node();
                if let Some(shadow_root) = root_node.dyn_ref::<web_sys::ShadowRoot>() {
                    let host_element = shadow_root.host();
                    self.host_element = Some(host_element);
                }
                self.editor_element = Some(mount_element);
                let xmsgs = self.try_ready_listener();
                Effects::new([], xmsgs)
            }
            Msg::FontReady => {
                log::info!("Fonts is ready in Web editor..");
                let ch_width = self.font_loader.ch_width;
                let ch_height = self.font_loader.ch_height;
                self.options.ch_width = ch_width;
                self.options.ch_height = ch_height;
                self.is_fonts_ready = true;
                let xmsgs = self.try_ready_listener();
                Effects::new([], xmsgs)
            }
            Msg::FontLoaderMsg(fmsg) => self.font_loader.update(fmsg).localize(Msg::FontLoaderMsg),
            Msg::ChangeValue(content) => {
                self.process_calls_with_effects([Call::Command(Command::SetContent(content))]);
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
                if self.is_ready() {
                    let client_x = me.client_x();
                    let client_y = me.client_y();
                    let cursor = self.client_to_grid_clamped(client_x, client_y);
                    let msgs = self
                        .base_editor
                        .process_commands([Command::SetPosition(cursor)]);
                    Effects::new(vec![], msgs)
                } else {
                    Effects::none()
                }
            }
            Msg::Mousedown(me) => {
                if self.is_ready() && MouseButton::is_primary(&me) {
                    log::info!("mouse down event in ultron..");
                    let client_x = me.client_x();
                    let client_y = me.client_y();
                    //self.base_editor.clear_selection();
                    if self.options.allow_text_selection {
                        self.is_selecting = true;
                    }
                    let cursor = self.client_to_grid_clamped(client_x, client_y);
                    if self.options.allow_text_selection
                        && self.is_selecting
                        && !self.show_context_menu
                    {
                        self.base_editor.set_selection_start(cursor);
                    }
                    let msgs = self
                        .base_editor
                        .process_commands([Command::SetPosition(cursor)]);
                    Effects::new(vec![], msgs)
                } else {
                    Effects::none()
                }
            }
            Msg::Mousemove(me) => {
                if self.is_ready() {
                    let client_x = me.client_x();
                    let client_y = me.client_y();
                    let cursor = self.client_to_grid_clamped(client_x, client_y);
                    if self.options.allow_text_selection
                        && self.is_selecting
                        && !self.show_context_menu
                    {
                        let selection = self.base_editor.selection();
                        if let Some(start) = selection.start {
                            self.base_editor.set_selection_end(cursor);
                            let msgs = self
                                .base_editor
                                .process_commands([Command::SetSelection(start, cursor)]);
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
            Msg::Mouseup(me) => {
                if self.is_ready() && MouseButton::is_primary(&me){
                    let client_x = me.client_x();
                    let client_y = me.client_y();
                    let is_primary_btn = me.button() == 0;
                    if is_primary_btn {
                        let cursor = self.client_to_grid_clamped(client_x, client_y);
                        self.base_editor
                            .process_commands([Command::SetPosition(cursor)]);

                        if self.is_selecting {
                            self.is_selecting = false;
                            self.base_editor.set_selection_end(cursor);
                            let selection = self.base_editor.selection();
                            if let (Some(start), Some(end)) = (selection.start, selection.end) {
                                let msgs = self
                                    .base_editor
                                    .process_commands([Command::SetSelection(start, end)]);
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
                } else {
                    Effects::none()
                }
            }
            Msg::Keydown(ke) => {
                self.process_keypress(&ke)
            }
            Msg::Measurements(measure) => {
                self.update_measure(measure);
                Effects::none()
            }
            Msg::Focused(_fe) => {
                self.is_focused = true;
                log::info!("in Msg::Focused: {}", self.is_focused);
                log::info!("show cursor: {}", self.options.show_cursor);
                Effects::none()
            }
            Msg::SetFocus => {
                self.is_focused = true;
                log::info!("ultron editor is focused: {}", self.is_focused);
                if let Some(editor_element) = &self.editor_element {
                    let html_elm: &web_sys::HtmlElement = editor_element.unchecked_ref();
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
                self.context_menu
                    .update(context_menu::Msg::ShowAt(Point2::new(x, y)))
                    .localize(Msg::ContextMenuMsg)
            }
            Msg::ContextMenuMsg(cm_msg) => self
                .context_menu
                .update(cm_msg)
                .localize(Msg::ContextMenuMsg),
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
                        self.process_calls_with_effects([Call::Command(Command::Undo)]);
                    }
                    MenuAction::Redo => {
                        self.process_calls_with_effects([Call::Command(Command::Redo)]);
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
                        self.process_calls_with_effects([Call::SelectAll]);
                    }
                }
                Effects::none()
            }
            Msg::NoOp => Effects::none(),
        }
    }

    fn view(&self) -> Node<Msg> {
        if self.is_fonts_ready {
            self.view_web_editor()
        } else {
            self.font_loader.view().map_msg(Msg::FontLoaderMsg)
        }
    }

    fn stylesheet() -> Vec<String> {
        let main = jss! {
            Self::selector_ns(""): {
                position: "relative",
                white_space: "normal",
            },

            Self::selector_ns("occupy_container"): {
                width: percent(100),
                height: "auto",
            },

            "code":{
                white_space: "pre",
                caret_color: "transparent",
            },

            Self::selector_ns("code_wrapper"): {
                margin: 0,
            },

            Self::selector_ns("code"): {
                position: "relative",
                display: "block",
                // to make the background color extend to the longest line, otherwise only the
                // longest lines has a background-color leaving the shorter lines ugly
                min_width: "max-content",
            },



            // numbers
            Self::selector_ns("number"): {
                flex: "none", // dont compress the numbers
                text_align: "right",
                background_color: "#ddd",
                display: "inline-block",
                user_select: "none",
                "-webkit-user-select": "none",
            },

            // numbers when selected programatically
            // using Range::select_node_contents, the numbers are visually highlighted in the browser
            // but not included when copy pasted to a simple text editor.
            // To visually represent that the number are not included in the selection
            // set the background-color to transparent
            Self::selector_ns("number") + "::selection": {
                background_color: "transparent",
            },

            // line content
            Self::selector_ns("line"): {
                flex: "none", // dont compress lines
                display: "block",
            },

            "font_measure": {
                bottom: px(30),
                display: "inline-block",
            },

            Self::selector_ns("status"): {
                position: "fixed",
                bottom: 0,
                display: "flex",
                flex_direction: "row",
                user_select: "none",
                "-webkit-user-select": "none",
            },

            // set background color to transparent when programatically selected
            Self::selector_ns("status") + "::selection": {
                background_color: "transparent",
            },

            Self::selector_ns("virtual_cursor"): {
                position: "absolute",
                border_width: px(1),
                opacity: 1,
                border_style: "solid",
            },

            Self::selector_ns("cursor_center"):{
                width: percent(100),
                height: percent(100),
                opacity: percent(50),
                animation: "cursor_blink-anim 1000ms step-end infinite",
            },
        };

        let media_css = jss_with_media!{
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

        [vec![main, media_css], FontLoader::<Msg>::stylesheet(), Menu::<Msg>::stylesheet()].concat()
    }

    fn style(&self) -> Vec<String> {
        let font_family = &self.font_loader.settings.font_family;
        let font_size = self.font_loader.settings.font_size;

        vec![jss! {

            Self::selector_ns("code"): {
                font_family: font_family.to_owned(),
                font_size: px(font_size),
            },

            Self::selector_ns("line") + " .selected": {
               background_color: self.selection_background().to_css(),
            },

            Self::selector_ns("status"): {
                font_family: font_family.to_owned(),
            },

            Self::selector_ns("virtual_cursor"): {
                border_color: self.cursor_border().to_css(),
            },

            Self::selector_ns("cursor_center"):{
                background_color: self.cursor_color().to_css(),
            },

        }]
    }
}

impl<XMSG> WebEditor<XMSG>
where
    XMSG: 'static,
{

    pub fn on_ready<F>(&mut self, f: F)
    where
        F: Fn() -> XMSG + 'static,
    {
        self.ready_listener.push(Callback::from(move |_| f()));
    }


    pub fn set_syntax_token(&mut self, syntax_token: &str) {
        self.text_highlighter
            .borrow_mut()
            .set_syntax_token(syntax_token);
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
        self.base_editor.add_on_change_listener(f);
    }

    pub fn add_on_change_notify<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        self.base_editor.add_on_change_notify(f);
    }

    pub fn get_content(&self) -> String {
        self.base_editor.get_content()
    }

    pub fn text_buffer(&self) -> &TextBuffer {
        self.base_editor.text_buffer()
    }

    pub fn command_set_position(&mut self, position: Point2<usize>) {
        self.base_editor.as_mut().command_set_position(position)
    }

    /// returns true if the editor is ready
    fn is_ready(&self) -> bool {
        self.is_fonts_ready && self.editor_element.is_some()
    }

    fn try_ready_listener(&self) -> Vec<XMSG> {
        if self.is_ready() {
            log::info!("emitting the ready listener..");
            self.ready_listener.iter().map(|c| c.emit(())).collect()
        } else {
            vec![]
        }
    }

    fn browser_clear_selection(&self) -> bool {
        let selection = window().get_selection().ok().flatten().expect("must have selection");
        selection.remove_all_ranges();
        false
    }

    fn browser_select_all(&self) -> bool{
        log::info!("browser select all.");
        let selection = window().get_selection().ok().flatten().expect("must have selection");
        selection.remove_all_ranges();
        let editor_element = self.editor_element.as_ref().expect("expecting editor element");
        let editor_node: &web_sys::Node = editor_element.unchecked_ref();
        selection.select_all_children(editor_node);
        false
    }

    fn view_web_editor(&self) -> Node<Msg> {
        let enable_context_menu = self.options.enable_context_menu;
        let enable_keypresses = self.options.enable_keypresses;
        let enable_click = self.options.enable_click;
        div(
            [
                Self::class_ns(""),
                key("editor-main"),
                if self.options.occupy_container{
                    Self::class_ns("occupy_container")
                }else{
                    empty_attr()
                },
                on_mount(Msg::EditorMounted),
                on_keydown(move|ke|{
                    ke.prevent_default();
                    ke.stop_propagation();
                    if enable_keypresses {
                        Msg::Keydown(ke)
                    } else {
                        Msg::NoOp
                    }
                }),
                on_click(move |me| {
                    if enable_click {
                        Msg::Click(me)
                    } else {
                        Msg::NoOp
                    }
                }),
                spellcheck(false),
                tabindex(0),
                contenteditable(true),
                attr("role","textbox"),
                on_focus(Msg::Focused),
                on_blur(Msg::Blur),
                on_contextmenu(move |me| {
                    if enable_context_menu {
                        me.prevent_default();
                        me.stop_propagation();
                        Msg::ContextMenu(me)
                    } else {
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
                    self.view_text_edit()
                },
                lazy_view_if(
                    self.is_focused && self.options.show_cursor,
                    ||self.view_cursor(),
                ),
                lazy_view_if(self.options.show_status_line, ||self.view_status_line()),
                lazy_view_if(
                    self.is_focused && self.show_context_menu,
                    ||self.context_menu.view().map_msg(Msg::ContextMenuMsg),
                ),
            ],
        )
    }
    pub fn ch_width(&self) -> f32 {
        self.options.ch_width.expect("must have already measured")
    }
    #[track_caller]
    pub fn ch_height(&self) -> f32 {
        self.options.ch_height.expect("must have already measured")
    }

    fn update_measure(&mut self, measurements: Measurements) {
        match &*measurements.name {
            "keypress" => {
                if let Some(average_dispatch) = self.measure.average_dispatch.as_mut() {
                    *average_dispatch = (*average_dispatch + measurements.total_time) / 2.0;
                } else {
                    self.measure.average_dispatch = Some(measurements.total_time);
                }
                self.measure.last_dispatch = Some(measurements.total_time);
            }
            _ => {
                log::trace!("unexpected measurement name from: {measurements:?}");
            }
        }

        self.measure.detail = Some(measurements);
    }

    pub fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.mouse_cursor = mouse_cursor;
    }

    pub fn get_char(&self, loc: Point2<usize>) -> Option<char> {
        self.base_editor.get_char(loc)
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.base_editor.get_position()
    }

    fn rehighlight_all(&mut self) {
        self.text_highlighter.borrow_mut().reset();
        *self.highlighted_lines.borrow_mut() = Self::highlight_lines(
            self.base_editor.as_ref(),
            &mut self.text_highlighter.borrow_mut(),
        );
    }

    /// rehighlight from 0 to the end of the visible lines
    pub fn rehighlight_visible_lines(&mut self) {
        if let Some((_top, end)) = self.visible_lines() {
            let text_highlighter = self.text_highlighter.clone();
            let highlighted_lines = self.highlighted_lines.clone();
            let lines = self.base_editor.as_ref().lines();
            for handle in self.highlight_task_handles.drain(..) {
                //cancel the old ones, dropping the handle will call on the cancel_animation_frame
                //for this handle
                drop(handle);
            }

            let closure = move |deadline: IdleDeadline| {
                let mut text_highlighter = text_highlighter.borrow_mut();
                text_highlighter.reset();
                let mut did_complete = true;
                let mut new_highlighted_lines = Vec::with_capacity(end);
                for line in lines[..end].iter(){
                    new_highlighted_lines.push(Self::highlight_line(line, &mut text_highlighter));
                    if deadline.did_timeout(){
                        log::warn!("No more time highlighting visible lines");
                        did_complete = false;
                        break;
                    }
                }
                
                for (hl_line, new_hl_line) in highlighted_lines.borrow_mut().iter_mut().zip(new_highlighted_lines){
                    *hl_line = new_hl_line;
                }
                if did_complete{
                    log::info!("Succeeded highlighting all visible lines..");
                }
                else{
                    log::warn!("The highlighting job did not complete...");
                }
            };

            let handle = request_idle_callback(closure).expect("must have a handle");

            self.highlight_task_handles.push(handle);
        } else {
            self.rehighlight_all();
        }
    }

    /// rehighlight the rest of the lines that are not visible
    pub fn rehighlight_non_visible_lines_in_background(&mut self) -> Effects<Msg, XMSG> {
        if let Some((_top, end)) = self.visible_lines() {
            for handle in self.background_task_handles.drain(..) {
                drop(handle);
            }
            let text_highlighter = self.text_highlighter.clone();
            let highlighted_lines = self.highlighted_lines.clone();
            let lines = self.base_editor.as_ref().lines();
            let is_background_highlighting_ongoing =
                self.is_background_highlighting_ongoing.clone();

            let closure = move |deadline:IdleDeadline| {
                is_background_highlighting_ongoing.store(true, Ordering::Relaxed);
                let mut text_highlighter = text_highlighter.borrow_mut();
                let mut did_complete = true;
                let mut new_highlighted_lines = Vec::with_capacity(lines.len() - end);
                for line in lines[end..].iter(){
                    new_highlighted_lines.push(Self::highlight_line(line, &mut text_highlighter));
                    if deadline.did_timeout(){
                        log::warn!("---> No more time background highlighting...");
                        did_complete = false;
                        break;
                    }
                }

                for (hl_line, new_hl_line) in highlighted_lines.borrow_mut().iter_mut().skip(end).zip(new_highlighted_lines){
                    *hl_line = new_hl_line;
                }

                if did_complete{
                    log::info!("Succeeded background highlighting...");
                }else{
                    log::error!("Background highlighting did not complete...");
                }
            };

            let handle =
                sauron::dom::request_idle_callback(closure).expect("timeout handle");
            self.background_task_handles.push(handle);
        } else {
            self.rehighlight_all();
        }
        Effects::none()
    }

    pub fn keyevent_to_call(ke: &web_sys::KeyboardEvent) -> Option<Call> {
        let is_ctrl = ke.ctrl_key();
        let is_shift = ke.shift_key();
        let key = ke.key();
        if key.chars().count() == 1 {
            let c = key.chars().next().expect("must be only 1 chr");
            let command = match c {
                'c' if is_ctrl => Call::CopyText,
                'x' if is_ctrl => Call::CutText,
                'v' if is_ctrl => {
                    log::trace!("pasting is handled");
                    Call::PasteTextBlock(String::new())
                }
                'z' | 'Z' if is_ctrl => {
                    if is_shift {
                        Call::Command(Command::Redo)
                    } else {
                        Call::Command(Command::Undo)
                    }
                }
                'r' if is_ctrl => Call::Command(Command::Redo),
                'a' if is_ctrl => {
                    Call::SelectAll
                }
                _ => Call::Command(Command::InsertChar(c)),
            };

            Some(command)
        } else {
            let editor_command = match &*key {
                "Tab" => Some(Command::IndentForward),
                "Enter" => Some(Command::BreakLine),
                "Backspace" => Some(Command::DeleteBack),
                "Delete" => Some(Command::DeleteForward),
                "ArrowUp" => Some(Command::MoveUp),
                "ArrowDown" => Some(Command::MoveDown),
                "ArrowLeft" => Some(Command::MoveLeft),
                "ArrowRight" => Some(Command::MoveRight),
                "Home" => Some(Command::MoveLeftStart),
                "End" => Some(Command::MoveRightEnd),
                _ => None,
            };
            editor_command.map(Call::Command)
        }
    }

    /// make this into keypress to command
    pub fn process_keypress(&mut self, ke: &web_sys::KeyboardEvent) -> Effects<Msg, XMSG> {
        if let Some(command) = Self::keyevent_to_call(ke) {
            let effects = self
                .process_calls_with_effects([command])
                .measure_with_name("keypress");
            effects.append_local([Msg::ScrollCursorIntoView])
        } else {
            Effects::none()
        }
    }

    /// process the calls and dispatch effects events when applicable
    pub fn process_calls_with_effects(
        &mut self,
        commands: impl IntoIterator<Item = Call>,
    ) -> Effects<Msg, XMSG> {
        let results: Vec<bool> = commands
            .into_iter()
            .map(|command| self.process_call(command))
            .collect();
        let is_content_changed = results.into_iter().any(|v| v);
        if is_content_changed {
            let xmsgs = self.base_editor.emit_on_change_listeners();
            let mut all_effects = vec![Effects::new([], xmsgs)];
            if self.options.use_syntax_highlighter {
                self.rehighlight_visible_lines();
                let effects = self.rehighlight_non_visible_lines_in_background();
                all_effects.push(effects);
            }
            if let Some(host_element) = self.host_element.as_ref() {
                host_element
                    .set_attribute("content", &self.get_content())
                    .expect("set attr content");
                host_element
                    .dispatch_event(&InputEvent::create_web_event_composed())
                    .expect("dispatch event");
            }
            Effects::batch(all_effects)
        } else {
            Effects::none()
        }
    }

    fn highlight_line(line: &str, text_highlighter: &mut TextHighlighter) -> Vec<(Style, Vec<Ch>)> {
        let h_ranges = text_highlighter
            .highlight_line(line)
            .expect("must highlight");
        h_ranges
            .into_iter()
            .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
            .collect()
    }

    pub fn highlight_lines(
        text_edit: &TextEdit,
        text_highlighter: &mut TextHighlighter,
    ) -> Vec<Vec<(Style, Vec<Ch>)>> {
        text_edit
            .lines()
            .iter()
            .map(|line| {
                Self::highlight_line(line, text_highlighter)
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

    /// process a call and return true of the content has changed
    /// Note: this does not trigger an effect
    pub fn process_call(&mut self, call: Call) -> bool {
        match call {
            Call::Command(command) => match command {
                Command::InsertChar(ch) => {
                    self.insert_to_highlighted_line(ch);
                    self.base_editor.process_command(command)
                }
                _ => self.base_editor.process_command(command),
            },
            Call::PasteTextBlock(text_block) => self
                .base_editor
                .process_command(Command::PasteTextBlock(text_block)),
            Call::MergeText(text_block) => self
                .base_editor
                .process_command(Command::MergeText(text_block)),
            Call::CopyText => self.copy_selected_text_to_clipboard(),
            Call::CutText => self.cut_selected_text_to_clipboard(),
            Call::SelectAll => self.browser_select_all(),
            Call::ClearSelection => self.browser_clear_selection(),
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.base_editor.selected_text()
    }

    pub fn is_selected(&self, loc: Point2<i32>) -> bool {
        self.base_editor.is_selected(loc)
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        self.base_editor.cut_selected_text()
    }

    pub fn clear(&mut self) {
        self.base_editor.clear()
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.base_editor.set_selection(start, end);
    }

    pub fn copy_selected_text_to_clipboard(&self) -> bool {
        log::warn!("Copying text to clipboard..");
        #[cfg(web_sys_unstable_apis)]
        if let Some(clipboard) = window().navigator().clipboard() {
            if let Some(selected_text) = self.selected_text() {
                log::info!("selected text: {selected_text}");
                let fut = crate::wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&selected_text));
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

    /// calculate the bounding rect of the base_editor using a DOM call [getBoundingClientRect](https://developer.mozilla.org/en-US/docs/Web/API/Element/getBoundingClientRect)
    pub fn bounding_rect(&self) -> Option<(Point2<f32>, Point2<f32>)> {
        if let Some(ref editor_element) = self.editor_element {
            let rect = editor_element.get_bounding_client_rect();
            let editor_x = rect.x() as f32;
            let editor_y = rect.y() as f32;
            let bottom = rect.bottom() as f32;
            let right = rect.right() as f32;
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

    /// calculate screen client coordinates to coordinates relative to the editor's bounding box
    pub fn client_to_relative(&self, client_x: i32, client_y: i32) -> Point2<f32> {
        let editor = self.editor_offset().expect("must have an editor offset");
        let x = client_x as f32 - editor.x;
        let y = client_y as f32 - editor.y;
        Point2::new(x, y)
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
            self.base_editor.total_lines().to_string().len() + self.numberline_padding_wide()
        } else {
            0
        }
    }

    pub fn total_lines(&self) -> usize {
        self.base_editor.total_lines()
    }

    /// convert screen coordinate to grid coordinate taking into account the editor element
    pub fn client_to_grid(&self, client_x: i32, client_y: i32) -> Point2<i32> {
        let rel = self.client_to_relative(client_x, client_y);
        self.relative_to_cursor(rel.x, rel.y)
    }

    /// calculate this position, which relative to the editor's bound box into grid coordinate
    pub fn relative_to_cursor(&self, rel_x: f32, rel_y: f32) -> Point2<i32> {
        let numberline_wide_with_padding = self.numberline_wide_with_padding() as f32;
        let ch_width = self.ch_width();
        let ch_height = self.ch_height();
        assert!(ch_width > 0.);
        assert!(ch_height > 0.);
        let col = rel_x / ch_width - numberline_wide_with_padding;
        let line = rel_y / ch_height;
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
        let cursor = self.base_editor.get_position();
        Point2::new(
            (cursor.x + self.numberline_wide_with_padding()) as f32 * self.ch_width(),
            cursor.y as f32 * self.ch_height(),
        )
    }

    /// calculate the width of the numberline including the padding
    fn number_line_with_padding_width(&self) -> f32 {
        self.numberline_wide_with_padding() as f32 * self.ch_width()
    }

    fn view_cursor(&self) -> Node<Msg> {
        let cursor = self.cursor_to_client();
        div(
            [
                Self::class_ns("virtual_cursor"),
                style! {
                    top: px(cursor.y),
                    left: px(cursor.x),
                    width: px(self.ch_width()),
                    height: px(self.ch_height()),
                },
                on_mount(Msg::CursorMounted),
            ],
            [div([Self::class_ns("cursor_center")], [])],
        )
    }

    /// the view for the status line
    pub fn view_status_line(&self) -> Node<Msg>{
        let cursor = self.base_editor.get_position();

        div(
            [
                Self::class_ns("status"),
                style! {
                    background_color: self.gutter_background().to_css(),
                    color: self.gutter_foreground().to_css(),
                    height: px(Self::status_line_height()),
                    left: px(self.number_line_with_padding_width()),
                    font_size: px(self.font_loader.settings.font_size),
                },
            ],
            [
                text!(" |> line: {}, col: {} ", cursor.y + 1, cursor.x + 1),
                text!(" |> version:{}", env!("CARGO_PKG_VERSION")),
                text!(" |> lines: {}", self.base_editor.total_lines()),
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
                text!(" |> selection: {:?}", self.base_editor.selection()),
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
                if let Some(detail) = &self.measure.detail{
                    node_list([
                        text!(" |> count: {}", detail.strong_count),
                        text!(" |> weak: {}", detail.weak_count),
                    ])
                }else{
                    text!("")
                },
                if self
                    .is_background_highlighting_ongoing
                    .load(Ordering::Relaxed)
                {
                    Spinner::new(20).view()
                } else {
                    text!("")
                },
            ],
        )
    }

    fn view_line_number(&self, line_number: usize) -> Node<Msg> {
        lazy_view_if(
            self.options.show_line_numbers,
            ||span(
                [
                    Self::class_ns("number"),
                    style! {
                        background_color: self.gutter_background().to_css(),
                        color: self.gutter_foreground().to_css(),
                        width: px(self.ch_width() * self.base_editor.numberline_wide() as f32),
                        height: px(self.ch_height()),
                        padding_right: px(self.ch_width() * self.numberline_padding_wide() as f32),
                        user_select: "none",
                        "-webkit-user-select": "none",
                    },
                ],
                [text(line_number)],
            ),
        )
    }

    /// calculate the maximum number of visible lines
    fn max_visible_lines(&self) -> Option<usize> {
        if let Some((start, end)) = self.bounding_rect() {
            Some(((end.y - start.y) / self.ch_height()) as usize)
        } else {
            None
        }
    }

    /// calculate which lines are visible in the editor
    fn visible_lines(&self) -> Option<(usize, usize)> {
        if let Some((start, end)) = self.bounding_rect() {
            let ch_height = self.ch_height();
            let top = ((0.0 - start.y) / ch_height) as usize;
            let bottom = ((end.y - 2.0 * start.y) / ch_height) as usize;
            Some((top, bottom))
        } else {
            None
        }
    }

    fn view_highlighted_line(
        &self,
        line: &[(Style, Vec<Ch>)],
    ) -> Vec<Node<Msg>> {
        if line.is_empty(){
            // added here to have a newline for empty lines when copied using native browser
            // selection and copy command
            vec![text("\n")]
        }else{
            line.iter()
                .map(|(style, range)| {
                    let foreground = util::to_rgba(style.foreground).to_css();
                    let range_str = String::from_iter(range.iter().map(|ch| ch.ch));
                    span([style! { color: foreground }], [
                        text(range_str)
                    ])
                })
                .collect()
            }
    }

    fn user_select(&self) -> &'static str{
        if self.options.allow_text_selection {
            "text"
        } else {
            "none"
        }
    }

    // highlighted view
    pub fn view_highlighted_lines(&self) -> Node<Msg> {
        let code_attributes = [
            Self::class_ns("code"),
            style! {
                background: self.theme_background().to_css(),
            },
        ];

        let highlighted_lines = self.highlighted_lines.borrow();
        let rendered_lines = highlighted_lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                div(
                    [
                        Self::class_ns("line"),
                        // needed to put the height here, since for some reason it add 1px to the
                        // parent div, not a margin, not border,
                        style! {
                            height: px(self.ch_height()),
                        },
                    ],
                    {
                        [self.view_line_number(line_index + 1)]
                            .into_iter()
                            .chain(self.view_highlighted_line(line))
                            .collect::<Vec<_>>()
                    },
                )
            });

            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            code(code_attributes, rendered_lines)
    }

    /// height of the status line which displays editor infor such as cursor location
    pub fn status_line_height() -> i32 {
        50
    }

    pub fn view_text_edit(&self) -> Node<Msg> {
        let text_edit = &self.base_editor.as_ref();

        let code_attributes = [Self::class_ns("code")];
        let rendered_lines = text_edit
            .lines()
            .into_iter()
            .enumerate()
            .map(|(line_index, line)| {
                let line_number = line_index + 1;
                div(
                    [
                        Self::class_ns("line"),
                        // Important! This is needed to render blank lines with same height as the
                        // non blank ones
                        style! {
                            height: px(self.ch_height()),
                            user_select: self.user_select(),
                            "-webkit-user-select": self.user_select(),
                        },
                    ],
                    [
                        self.view_line_number(line_number),
                        span([],[text(line)])
                    ],
                )
            });

            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                [Self::class_ns("code_wrapper")],
                [code(code_attributes, rendered_lines)],
            )
    }

    pub fn view_text_buffer(text_buffer: &TextBuffer, options: &Options) -> Node<Msg> {

        let ch_height = options
            .ch_height
            .expect("error1: must have a ch_height in the options");

        let rendered_lines = text_buffer
            .lines()
            .into_iter()
            .enumerate()
            .map(|(line_index, line)| {
                let line_number = line_index + 1;
                div(
                    [
                        Self::class_ns("line"),
                        class("simple"),
                        // Important! This is needed to render blank lines with same height as the
                        // non blank ones
                        style! {height: px(ch_height)},
                    ],
                    [
                        lazy_view_if(
                            options.show_line_numbers,
                            ||span([Self::class_ns("number")], [text(line_number)]),
                        ),
                        // Note: this is important since text node with empty
                        // content seems to cause error when finding the dom in rust
                        span([], [text(line)]),
                    ],
                )
            });

            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre([Self::class_ns("code_wrapper")], [code(vec![Self::class_ns("code")], rendered_lines)])
    }
}

