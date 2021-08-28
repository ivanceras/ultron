use history::Recorded;
use sauron::html::attributes::class_namespaced;
use sauron::html::attributes::classes_flag_namespaced;
use sauron::jss;
use sauron::prelude::*;
use sauron::wasm_bindgen::JsCast;
use sauron::web_sys::HtmlTextAreaElement;
use sauron::Measurements;
use std::iter::FromIterator;
use syntect::easy::HighlightLines;
use syntect::highlighting::Color;
use syntect::highlighting::Theme;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use text_buffer::Movement;
use text_buffer::TextBuffer;
use unicode_width::UnicodeWidthChar;

pub const CH_WIDTH: u32 = 8;
pub const CH_HEIGHT: u32 = 16;

pub(crate) mod action;
mod history;
mod style;
mod text_buffer;

#[derive(Clone, PartialEq)]
pub enum Msg {
    KeyDown(web_sys::KeyboardEvent),
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
    Mounted(web_sys::Node),
    SetMeasurement(Measurements),
    Scrolled((i32, i32)),
}

const COMPONENT_NAME: &str = "ultron";

pub struct Editor {
    text_buffer: TextBuffer,
    /// current line, computed every keypressed
    current_line: usize,
    /// current column, computed every keypressed
    current_col: usize,
    /// line count, computed every keypressed
    line_count: usize,
    /// the number of digits of line count,
    /// line number padding is derive from this.
    number_wide: usize,
    /// the calculated lines from the text_buffer
    lines: Vec<Line>,
    /// flag to use block cursor or i-type cursor
    use_block_cursor: bool,
    /// number of lines in a page, when paging up and down
    page_size: usize,
    /// for undo and redo
    recorded: Recorded,
    ///
    is_selecting: bool,
    pub(crate) browser_size: (i32, i32),
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
    pub measurements: Option<Measurements>,
    editor_x: f32,
    editor_y: f32,
    scroll_top: f32,
    scroll_left: f32,
}

struct Line {
    line_pos: usize,
    /// hash of the line content
    /// total sum of the character width
    line_width: usize,
    highlight_ranges: Vec<HighlightRange>,
    last_col: usize,
}

/// highlight range, which contains a vector of individual char
struct HighlightRange {
    style: Style,
    chars: Vec<Ch>,
}

impl HighlightRange {
    fn unicode_width(&self) -> usize {
        self.chars.iter().map(|ch| ch.unicode_width).sum()
    }
}

/// each individual character
struct Ch {
    /// position of char relative to the whole text buffer
    position: usize,
    /// position of char relative to the line it is in.
    col_pos: usize,
    unicode_width: usize,
    ch: char,
}

impl Editor {
    /// returns bool indicating whether the view should be updated or not
    pub fn update(&mut self, msg: Msg) -> bool {
        let should_update_view = match msg {
            Msg::Mounted(target_node) => {
                let element: &web_sys::Element = target_node.unchecked_ref();
                let rect = element.get_bounding_client_rect();
                self.editor_x = rect.x().round() as f32;
                self.editor_y = rect.y().round() as f32;
                true
            }
            Msg::Scrolled((scroll_top, scroll_left)) => {
                self.scroll_top = scroll_top as f32;
                self.scroll_left = scroll_left as f32;
                log::trace!("scrolled to: {},{}", scroll_top, scroll_left);
                true
            }
            Msg::Mouseup(client_x, client_y) => {
                let (line, col) = self.convert_mouse_to_line_col(client_x, client_y);
                self.update(Msg::EndSelection(line, col))
            }
            Msg::Mousedown(client_x, client_y) => {
                //log::trace!("Mouse is down at {},{}", client_x, client_y);
                let (line, col) = self.convert_mouse_to_line_col(client_x, client_y);
                //log::trace!("selection should start at {:?}", (line, col));
                self.update(Msg::StartSelection(line, col))
            }
            Msg::Mousemove(client_x, client_y) => {
                let (line, col) = self.convert_mouse_to_line_col(client_x, client_y);
                self.update(Msg::ToSelection(line, col))
            }
            Msg::Paste(text_content) => {
                log::trace!("pasted text: {}", text_content);
                self.insert_string(&text_content);
                true
            }
            Msg::CopiedSelected => {
                log::info!("copying works?..");
                true
            }
            Msg::MoveCursor(line, col) => {
                self.move_at(line, col);
                true
            }
            Msg::MoveCursorToLine(line) => {
                self.move_to_line(line);
                true
            }
            Msg::StartSelection(line, col) => {
                if self.is_selecting {
                    self.is_selecting = false;
                    self.text_buffer.selection = None;
                    true
                } else {
                    self.is_selecting = true;
                    let start_pos = self.text_buffer.line_col_to_pos(line, col);
                    self.text_buffer.selection = Some((start_pos, None));
                    self.reposition_cursor_to_selection();
                    true
                }
            }
            Msg::ToSelection(line, col) => {
                if self.is_selecting {
                    let end_pos = self.text_buffer.line_col_to_pos(line, col);
                    self.text_buffer
                        .selection
                        .as_mut()
                        .map(|(_from, to)| *to = Some(end_pos));
                    self.reposition_cursor_to_selection();
                    true
                } else {
                    true
                }
            }
            Msg::EndSelection(line, col) => {
                if self.is_selecting {
                    self.is_selecting = false;
                    let end_pos = self.text_buffer.line_col_to_pos(line, col);
                    self.text_buffer
                        .selection
                        .as_mut()
                        .map(|(_from, to)| *to = Some(end_pos));
                    self.reposition_cursor_to_selection();
                    true
                } else {
                    true
                }
            }
            Msg::StopSelection => {
                if self.is_selecting {
                    self.is_selecting = false;
                    true
                } else {
                    true
                }
            }
            Msg::SetMeasurement(measurements) => {
                self.measurements = Some(measurements);
                false
            }
            Msg::KeyDown(ke) => {
                let key = ke.key();
                let ctrl = ke.ctrl_key();
                match &*key {
                    "Enter" => {
                        self.insert('\n');
                    }
                    "Backspace" => {
                        self.delete();
                    }
                    "Delete" => {
                        self.delete_forward();
                    }
                    "Tab" => {
                        // tab is 4 spaces
                        self.insert_string(&" ".repeat(4));
                    }
                    "ArrowUp" => {
                        self.step(Movement::Up);
                    }
                    "ArrowDown" => {
                        self.step(Movement::Down);
                    }
                    "ArrowLeft" => {
                        self.step(Movement::Left);
                    }
                    "ArrowRight" => {
                        self.step(Movement::Right);
                    }
                    "End" => {
                        self.step(Movement::LineEnd);
                    }
                    "Home" => {
                        self.step(Movement::LineStart);
                    }
                    "PageDown" => {
                        self.step(Movement::PageDown(self.page_size));
                    }
                    "PageUp" => {
                        self.step(Movement::PageUp(self.page_size));
                    }
                    _ => {
                        if key.chars().count() == 1 {
                            let c = key.chars().next().expect("must be only 1 chr");
                            match c {
                                'z' if ctrl => {
                                    self.undo();
                                }
                                'y' if ctrl => {
                                    self.redo();
                                }
                                'c' if ctrl => {
                                    self.copy();
                                }
                                'x' if ctrl => {
                                    self.cut();
                                }
                                'v' if ctrl => {
                                    //do nothing
                                    //as paste in the textarea is handled
                                    //by the on_paste event mapped to Msg::Paste
                                }
                                _ => {
                                    self.insert(c);
                                }
                            }
                        }
                    }
                }
                true
            }
        };
        self.recompute_lines();
        self.recompute_meta();
        should_update_view
    }

    pub fn style(&self) -> Vec<String> {
        vec![self.generate_style()]
    }

    pub fn view(&self) -> Node<Msg> {
        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);
        let class_number_wide = format!("number_wide{}", self.number_wide);
        div(
            vec![
                class(COMPONENT_NAME),
                on_mount(|me| Msg::Mounted(me.target_node)),
                on_scroll(Msg::Scrolled),
            ],
            vec![
                textarea(
                    vec![
                        class_ns("paste_area"),
                        if let Some(text_selection) = self.text_selection() {
                            value(text_selection)
                        } else {
                            empty_attr()
                        },
                        // focus the textarea at all times
                        focus(true),
                        on_paste(|ce| {
                            let pasted_text = ce
                                .clipboard_data()
                                .expect("must have data transfer")
                                .get_data("text/plain")
                                .expect("must be text data");

                            let target = ce.target().expect("expecting a target");
                            let text_area: &HtmlTextAreaElement = target.unchecked_ref();
                            text_area.set_value("");
                            Msg::Paste(pasted_text)
                        }),
                    ],
                    vec![],
                ),
                div(
                    vec![
                        class_ns("code"),
                        class_ns(&class_number_wide),
                        on_scroll(Msg::Scrolled),
                    ],
                    self.lines
                        .iter()
                        .map(|line| self.view_line(line))
                        .collect::<Vec<_>>(),
                ),
                div(
                    vec![
                        class_ns("status"),
                        if let Some(gutter_bg) = self.gutter_background() {
                            style! {
                                "background-color": Self::convert_rgba(gutter_bg),
                            }
                        } else {
                            empty_attr()
                        },
                        if let Some(gutter_fg) = self.gutter_foreground() {
                            style! {
                                "color": Self::convert_rgba(gutter_fg)
                            }
                        } else {
                            empty_attr()
                        },
                    ],
                    vec![text(format!(
                        "line: {}, column: {}, {}",
                        self.current_line + 1,
                        self.current_col + 1,
                        if let Some(measurements) = &self.measurements {
                            format!(
                                "{} patches on {} total nodes took {}ms",
                                measurements.total_patches,
                                measurements.view_node_count,
                                measurements.total_time
                            )
                        } else {
                            "".to_string()
                        }
                    ))],
                ),
            ],
        )
    }
}

impl Editor {
    pub fn from_str(content: &str) -> Self {
        let syntax_set: SyntaxSet = SyntaxSet::load_defaults_newlines();
        let theme_set: ThemeSet = ThemeSet::load_defaults();
        //let theme_name = "Solarized (dark)".to_string();
        //let theme_name = "Solarized (light)".to_string();
        let theme_name = "base16-eighties.dark".to_string();
        //let theme_name = "base16-ocean.dark".to_string();
        //let theme_name = "base16-mocha.dark".to_string();
        //let theme_name = "base16-ocean.light".to_string();

        let mut editor = Editor {
            text_buffer: TextBuffer::from_str(content),
            current_line: 0,
            current_col: 0,
            line_count: 0,
            number_wide: 0,
            lines: vec![],
            use_block_cursor: true,
            page_size: 10,
            recorded: Recorded::new(),
            is_selecting: false,
            browser_size: Window::get_size(),
            syntax_set,
            theme_set,
            theme_name,
            measurements: None,
            editor_x: f32::NAN,
            editor_y: f32::NAN,
            scroll_top: 0.0,
            scroll_left: 0.0,
        };
        editor.recompute_lines();
        editor.recompute_meta();
        editor
    }

    /// this just converts mouse into line col calculation, it still needs to be corrected
    /// by actual lines and cols
    fn convert_mouse_to_line_col(&self, client_x: i32, client_y: i32) -> (usize, usize) {
        let number_line_offset = self.number_wide as f32 * CH_WIDTH as f32;
        let col = (client_x as f32 - self.editor_x - number_line_offset + self.scroll_left)
            / CH_WIDTH as f32
            - 1.0;
        let line = (client_y as f32 - self.editor_y + self.scroll_top) / CH_HEIGHT as f32 - 1.0;
        let max_line = (self.lines.len() - 1) as f32;
        let actual_line = line.clamp(0.0, max_line).round() as usize;
        let last_col = self.lines[actual_line].last_col as f32;
        let actual_col = col.clamp(0.0, last_col).round() as usize;
        (actual_line, actual_col)
    }

    pub fn set_browser_size(&mut self, width: i32, height: i32) {
        self.browser_size = (width, height);
    }

    /// return the string content of the buffer
    pub fn buffer_content(&self) -> String {
        self.text_buffer.buffer_content()
    }

    /// returns true if the editor was modified
    pub fn was_modified(&self) -> bool {
        self.text_buffer.was_modified()
    }

    fn maybe_recompute_lines(&mut self) {
        if self.text_buffer.was_modified() {
            let t1 = sauron::now();
            self.recompute_lines();
            let t2 = sauron::now();
            log::warn!("recompute lines took: {}ms", t2 - t1);
        }
    }

    fn active_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.theme_name]
    }

    fn theme_background(&self) -> Option<Color> {
        self.active_theme().settings.background
    }

    fn gutter_background(&self) -> Option<Color> {
        self.active_theme().settings.gutter
    }

    fn gutter_foreground(&self) -> Option<Color> {
        self.active_theme().settings.gutter_foreground
    }

    #[allow(unused)]
    fn accent_color(&self) -> Option<Color> {
        self.active_theme().settings.accent
    }

    fn selection_background(&self) -> Option<Color> {
        self.active_theme().settings.selection
    }

    fn cursor_color(&self) -> Option<Color> {
        self.active_theme().settings.caret
    }

    fn convert_rgba(c: Color) -> String {
        format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 * 255.0)
    }

    fn recompute_lines(&mut self) {
        let syntax: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .expect("unable to find rust syntax reference");
        let mut syntax_highlighter = HighlightLines::new(syntax, self.active_theme());

        self.lines = self
            .text_buffer
            .lines()
            .enumerate()
            .map(|(n_line, line)| {
                let line_str = String::from_iter(line.chars());
                let ranges: Vec<(Style, &str)> =
                    syntax_highlighter.highlight(&line_str, &self.syntax_set);

                let mut range_col = 0;
                let highlight_ranges: Vec<HighlightRange> = ranges
                    .into_iter()
                    .map(|(style, range_str)| {
                        let width_chars: Vec<Ch> = range_str
                            .chars()
                            .enumerate()
                            .map(|(n_col, ch)| {
                                let col_pos = range_col + n_col;
                                Ch {
                                    unicode_width: ch.width().unwrap_or(1),
                                    position: self.text_buffer.line_col_to_pos(n_line, col_pos),
                                    col_pos,
                                    ch,
                                }
                            })
                            .collect();
                        range_col += range_str.chars().count();

                        HighlightRange {
                            style,
                            chars: width_chars,
                        }
                    })
                    .collect();

                let line_width = highlight_ranges.iter().map(|r| r.unicode_width()).sum();

                let last_col: usize = self.text_buffer.last_line_col(n_line);
                Line {
                    line_pos: n_line,
                    line_width,
                    highlight_ranges,
                    last_col,
                }
            })
            .collect::<Vec<_>>();
    }

    fn recompute_meta(&mut self) {
        self.current_line = self.text_buffer.line();
        self.current_col = self.text_buffer.col();
        self.line_count = self.lines.len();
        self.number_wide = self.line_count.to_string().len();
    }

    /// insert char into the text buffer
    /// and at the same time record it for undo/redo later
    fn insert(&mut self, c: char) {
        self.text_buffer.insert(c);
        self.recorded.insert(c);
    }

    /// TODO: record this
    fn insert_string(&mut self, s: &str) {
        let start_pos = self.text_buffer.pos();
        self.text_buffer.insert_string(s);
        let end_pos = start_pos + s.chars().count();
        self.recorded.insert_string(start_pos, end_pos, s);
    }

    /// move the cursor in the text buffer
    /// and record that same movement to for undo/redo usage in the future
    fn step(&mut self, movement: Movement) {
        let from = self.text_buffer.pos();
        self.text_buffer.step(movement);
        let to = self.text_buffer.pos();
        self.recorded.move_cursor(to as isize - from as isize);
    }

    /// delete the character in the cursor and record it for undo/redo later
    /// triggered when pressing backspace
    fn delete(&mut self) {
        let c = self.text_buffer.delete();
        self.recorded.delete(c);
    }

    /// triggered when pressing the delete key
    fn delete_forward(&mut self) {
        // delete the selected text if there is,
        // otherwise delete the character in the cursor
        if let Some((start_pos, Some(end_pos))) = self.text_buffer.normalized_selection() {
            let deleted_text = self.text_buffer.cut_text(start_pos, end_pos);
            //log::trace!("deleted: {}", deleted_text);
            self.recorded
                .delete_selected_forward(start_pos, end_pos, &deleted_text);
            self.text_buffer.selection = None;
        } else {
            let c = self.text_buffer.delete_forward();
            self.recorded.delete_forward(c);
        }
    }

    fn cut(&mut self) {
        self.copy();
        if let Some((start_pos, Some(end_pos))) = self.text_buffer.normalized_selection() {
            let deleted_text = self.text_buffer.cut_text(start_pos, end_pos);
            log::trace!("cut: {}, at {}:{}", deleted_text, start_pos, end_pos);
            self.recorded
                .delete_selected_forward(start_pos, end_pos, &deleted_text);
            self.text_buffer.selection = None;
        }
    }

    fn move_at(&mut self, line: usize, col: usize) {
        let from = self.text_buffer.pos();
        self.text_buffer.move_at(line, col);
        let to = self.text_buffer.pos();
        self.recorded.move_cursor(to as isize - from as isize);
    }

    fn move_to_line(&mut self, line: usize) {
        let from = self.text_buffer.pos();
        self.text_buffer.move_to_line_end(line);
        let to = self.text_buffer.pos();
        self.recorded.move_cursor(to as isize - from as isize);
    }

    fn undo(&mut self) {
        log::info!("UNDOING...");
        self.recorded.undo(&mut self.text_buffer);
    }

    fn redo(&mut self) {
        log::info!("ReDOING...");
        self.recorded.redo(&mut self.text_buffer);
    }

    fn copy(&self) {
        #[cfg(feature = "with-navigator-clipboard")]
        self.copy_to_clipboard();
        #[cfg(not(feature = "with-navigator-clipboard"))]
        self.textarea_exec_copy();
    }

    /// copy the text selection using the navigator clipboard interface
    /// Note: This doesn't work on older browser such as webkit2-gtk
    fn copy_to_clipboard(&self) {
        if let Some((start_pos, Some(end_pos))) = self.text_buffer.normalized_selection() {
            let navigator = web_sys::window()
                .expect("no global `window` exists")
                .navigator();
            if let Some(clipboard) = navigator.clipboard() {
                let text_selection = self.text_buffer.get_text(start_pos, end_pos);
                let _ = clipboard.write_text(&text_selection);
            }
        }
    }

    /// execute copy on the selected textarea
    /// this works even on older browser
    #[cfg(not(feature = "with-navigator-clipboard"))]
    fn textarea_exec_copy(&self) {
        use sauron::web_sys::HtmlDocument;

        let document = sauron::document();
        let textarea_elm = document
            .query_selector(&self.text_area_class())
            .expect("must not error")
            .expect("must have the text area");
        let text_area: HtmlTextAreaElement = textarea_elm.unchecked_into();
        text_area.select();
        let html_document: HtmlDocument = document.unchecked_into();
        html_document.exec_command("copy").expect("must copy");
    }

    /// return the selected text
    fn text_selection(&self) -> Option<String> {
        if let Some((start_pos, Some(end_pos))) = self.text_buffer.normalized_selection() {
            let text_selection = self.text_buffer.get_text(start_pos, end_pos);
            Some(text_selection)
        } else {
            None
        }
    }

    fn move_to(&mut self, pos: usize) {
        self.text_buffer.move_to(pos);
    }

    fn reposition_cursor_to_selection(&mut self) {
        if let Some((_start_pos, Some(end_pos))) = self.text_buffer.selection {
            self.move_to(end_pos);
        }
    }

    fn text_area_class(&self) -> String {
        format!(".{}__paste_area", COMPONENT_NAME)
    }

    fn select_textarea(&self) {
        let textarea_elm = sauron::document()
            .query_selector(&self.text_area_class())
            .expect("must not error")
            .expect("must have the text area");
        let text_area: HtmlTextAreaElement = textarea_elm.unchecked_into();
        text_area.select();
    }

    fn view_line(&self, line: &Line) -> Node<Msg> {
        let is_current_line = self.current_line == line.line_pos;
        let line_pos = line.line_pos;

        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);

        let classes_ns_flag =
            |class_name_flags| classes_flag_namespaced(COMPONENT_NAME, class_name_flags);

        let filler_width = self.browser_size.0 as usize - line.line_width;
        let line_last_col = line.last_col;
        div(
            vec![
                class_ns("line_block"),
                classes_ns_flag([("block_cursor", self.use_block_cursor)]),
                classes_ns_flag([("thin_cursor", !self.use_block_cursor)]),
            ],
            vec![div(
                vec![
                    class_ns("number__line"),
                    classes_ns_flag([("line_focused", is_current_line)]),
                ],
                vec![
                    div(
                        vec![
                            class_ns("number"),
                            if let Some(gutter_bg) = self.gutter_background() {
                                style! {
                                    "background-color": Self::convert_rgba(gutter_bg),
                                }
                            } else {
                                empty_attr()
                            },
                            if let Some(gutter_fg) = self.gutter_foreground() {
                                style! {
                                    "color": Self::convert_rgba(gutter_fg)
                                }
                            } else {
                                empty_attr()
                            },
                        ],
                        vec![text(line.line_pos + 1)],
                    ),
                    div(vec![class_ns("line")], {
                        line.highlight_ranges
                            .iter()
                            .map(|range| self.view_range(line.line_pos, range))
                            .collect::<Vec<_>>()
                    }),
                    div(
                        vec![
                            class_ns("filler"),
                            style! {
                                "width": ex(filler_width),
                            },
                            if let Some(theme_bg) = self.theme_background() {
                                style! { "background-color" : Self::convert_rgba(theme_bg)}
                            } else {
                                empty_attr()
                            },
                        ],
                        vec![],
                    ),
                ],
            )],
        )
    }

    fn view_range(&self, line_pos: usize, range: &HighlightRange) -> Node<Msg> {
        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);
        let background = range.style.background;
        let foreground = range.style.foreground;
        div(
            vec![
                class_ns("range"),
                style! {
                    "color": format!("rgba({},{},{},{})", foreground.r,foreground.g, foreground.b, (foreground.a as f32/ 255.0)),
                    "background-color": format!("rgba({},{},{},{})", background.r,background.g, background.b, (background.a as f32/ 255.0)),
                },
            ],
            range
                .chars
                .iter()
                .map(|ch| self.view_char(line_pos, ch))
                .collect::<Vec<_>>(),
        )
    }

    fn view_char(&self, line_pos: usize, ch: &Ch) -> Node<Msg> {
        let col_pos = ch.col_pos;
        let is_current_line = self.current_line == line_pos;
        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);

        let classes_ns_flag =
            |class_name_flags| classes_flag_namespaced(COMPONENT_NAME, class_name_flags);
        {
            let class_wide = match ch.unicode_width {
                1 => "wide1",
                2 => "wide2",
                _ => unreachable!("only supporting characters with width 1 and 2"),
            };
            let is_current_col = ch.col_pos == self.current_col;
            let is_char_focused = is_current_col && is_current_line;

            let is_selected =
                if let Some((start_pos, Some(end_pos))) = self.text_buffer.normalized_selection() {
                    ch.position >= start_pos && ch.position < end_pos
                } else {
                    false
                };

            div(
                vec![
                    class_ns("ch"),
                    attr("pos", ch.position),
                    key(format!("{}_{}", line_pos, ch.position)),
                    classes_ns_flag([("ch_focused", is_char_focused)]),
                    classes_ns_flag([("selected", is_selected)]),
                    classes_ns_flag([(class_wide, ch.unicode_width > 1)]),
                ],
                if is_char_focused {
                    vec![div(vec![class_ns("cursor")], vec![text(ch.ch)])]
                } else {
                    vec![text(ch.ch)]
                },
            )
        }
    }
}
