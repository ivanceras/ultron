use crate::editor::COMPONENT_NAME;
use crate::util;
use css_colors::rgba;
use css_colors::Color;
use css_colors::RGBA;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::highlighting::Theme;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use unicode_width::UnicodeWidthChar;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    lines: Vec<Line>,
    highlighter: Highlighter,
    x_pos: usize,
    y_pos: usize,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
}

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

#[derive(Debug)]
pub struct Line {
    ranges: Vec<Range>,
    /// total width of this line
    width: usize,
}

#[derive(Debug)]
pub struct Range {
    cells: Vec<Cell>,
    width: usize,
    style: Style,
}

#[derive(Debug)]
pub struct Cell {
    ch: char,
    /// width of this character
    width: usize,
}

#[derive(Clone, Copy, Debug)]
struct FocusCell {
    line_index: usize,
    range_index: usize,
    cell_index: usize,
}

impl TextBuffer {
    pub fn from_str(content: &str) -> Self {
        let highlighter = Highlighter::default();
        let lines = content
            .lines()
            .map(|line| {
                let line_str = String::from_iter(line.chars());
                let style_range: Vec<(Style, &str)> = highlighter.highlight(&line_str);

                let ranges: Vec<Range> = style_range
                    .into_iter()
                    .map(|(style, range_str)| {
                        let cells = range_str.chars().map(Cell::from_char).collect();
                        Range::from_cells(cells, style)
                    })
                    .collect();

                Line::from_ranges(ranges)
            })
            .collect();
        Self {
            lines,
            highlighter,
            x_pos: 0,
            y_pos: 0,
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn active_theme(&self) -> &Theme {
        self.highlighter.active_theme()
    }

    fn gutter_background(&self) -> Option<RGBA> {
        self.active_theme().settings.gutter.map(util::to_rgba)
    }

    fn gutter_foreground(&self) -> Option<RGBA> {
        self.active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
    }

    /// how wide the numberline based on the character lengths of the number
    fn numberline_wide(&self) -> usize {
        self.lines.len().to_string().len()
    }

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    /// This is the total width of the number line
    pub(crate) fn get_numberline_wide(&self) -> usize {
        self.numberline_wide() + self.numberline_padding_wide()
    }

    pub fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let class_number_wide = format!("number_wide{}", self.numberline_wide());
        let focus_cell = self.get_focus_cell();
        div(
            vec![class_ns("code"), class_ns(&class_number_wide)],
            self.lines
                .iter()
                .enumerate()
                .map(|(line_index, line)| {
                    line.view_line(
                        &self,
                        focus_cell,
                        line_index,
                        line_index == focus_cell.line_index,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }
}

/// text manipulation
impl TextBuffer {
    /// the total number of lines of this text canvas
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the width of the line at line `n`
    pub fn line_width(&self, n: usize) -> Option<usize> {
        self.lines.get(n).map(|l| l.width)
    }

    /// calcultate which column position for this x relative to the widths
    fn calc_range_col_insert_position(line: &Line, x: usize) -> (usize, usize) {
        println!("calculating range col where x is: {}", x);
        let mut col_width = 0;
        let mut index = 0;
        for (i, range) in line.ranges.iter().enumerate() {
            for (j, cell) in range.cells.iter().enumerate() {
                if col_width >= x {
                    return (i, j);
                }
                col_width += cell.width;
                index += 1;
            }
        }
        let line_ranges_len = line.ranges.len();
        let last = if line_ranges_len > 0 {
            line_ranges_len - 1
        } else {
            0
        };

        (
            last,
            line.ranges
                .last()
                .map(|ranges| ranges.cells.len())
                .unwrap_or(0),
        )
    }

    fn get_focus_cell(&self) -> FocusCell {
        let line_index = self.y_pos;
        let line = &self.lines[line_index];
        let (range_index, cell_index) = Self::calc_range_col_insert_position(line, self.x_pos);
        FocusCell {
            line_index,
            range_index,
            cell_index,
        }
    }

    /// add more lines, used internally
    fn add_lines(&mut self, n: usize) {
        for _i in 0..n {
            println!("Adding line...{}", _i);
            self.lines.push(Line::default());
        }
    }

    /// fill columns at line y putting a space in each of the cells
    fn add_col(&mut self, y: usize, n: usize) {
        let ch = ' ';
        for _i in 0..n {
            println!("adding to column {}: {:?}", y, ch);
            self.lines[y].push_char(ch);
        }
    }

    /// break at line y and put the characters after x on the next line
    pub fn break_line(&mut self, x: usize, y: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            let (range_index, col) = Self::calc_range_col_insert_position(line, x);
            if let Some(range_bound) = line.ranges.get_mut(range_index) {
                let other = range_bound.split_at(col);
                let mut rest = line.ranges.drain(range_index + 1..).collect::<Vec<_>>();
                rest.insert(0, other);
                self.lines.insert(y + 1, Line::from_ranges(rest));
            }
        }
    }

    /// insert a character at this x and y and move cells after it to the right
    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.add_char(false, x, y, ch);
    }

    /// replace the character at this location
    pub fn replace_char(&mut self, x: usize, y: usize, ch: char) {
        self.add_char(true, x, y, ch);
    }

    /// delete character at this position
    pub fn delete_char(&mut self, x: usize, y: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            let (range_index, col) = Self::calc_range_col_insert_position(line, x);
            if let Some(mut range) = line.ranges.get_mut(range_index) {
                if range.cells.get(col).is_some() {
                    range.cells.remove(col);
                }
            }
        }
    }

    fn add_char(&mut self, is_replace: bool, x: usize, y: usize, ch: char) {
        assert!(
            ch != '\n',
            "line breaks should have been pre-processed before this point"
        );
        assert!(
            ch != '\t',
            "tabs should have been pre-processed before this point"
        );
        let line_gap = if y > self.total_lines() {
            y - self.total_lines()
        } else {
            0
        };

        if self.total_lines() == 0 {
            self.add_lines(1);
        }
        if line_gap > 0 {
            self.add_lines(line_gap);
        }
        let line = &self.lines[y];
        let col_diff = if x > line.width { x - line.width } else { 0 };
        if col_diff > 0 {
            self.add_col(y, col_diff);
        }

        let ch_width = ch.width().expect("must have a unicode width");
        let cell = Cell {
            ch,
            width: ch_width,
        };

        let (range_index, char_index) = Self::calc_range_col_insert_position(&self.lines[y], x);

        if is_replace {
            self.lines[y].ranges[range_index].cells[char_index] = cell
        } else {
            self.lines[y].ranges[range_index]
                .cells
                .insert(char_index, cell);
        }
    }

    fn rehighlight_line(&mut self, y: usize) {
        if let Some(mut line) = self.lines.get_mut(y) {
            line.rehighlight(&self.highlighter);
        }
    }
}

impl TextBuffer {
    pub(crate) fn command_insert_char(&mut self, ch: char) {
        self.insert_char(self.x_pos, self.y_pos, ch);
    }
    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.x_pos = x;
        self.y_pos = y;
    }
}

impl Line {
    fn push_char(&mut self, ch: char) {
        let cell = Cell::from_char(ch);
        let range = Range::from_cells(vec![cell], Style::default());
        self.ranges.push(range);
    }

    /// rehighlight this line
    fn rehighlight(&mut self, highlighter: &Highlighter) {
        let line_str = self.text();
        let style_range: Vec<(Style, &str)> = highlighter.highlight(&line_str);
        self.ranges = style_range
            .into_iter()
            .map(|(style, range_str)| {
                let cells = range_str.chars().map(Cell::from_char).collect();
                Range::from_cells(cells, style)
            })
            .collect();
    }

    /// get the text content of this line
    fn text(&self) -> String {
        String::from_iter(
            self.ranges
                .iter()
                .flat_map(|range| range.cells.iter().map(|cell| cell.ch)),
        )
    }

    fn from_ranges(ranges: Vec<Range>) -> Self {
        Self {
            width: ranges.iter().map(|range| range.width).sum(),
            ranges,
        }
    }

    fn view_line<MSG>(
        &self,
        text_buffer: &TextBuffer,
        focus_cell: FocusCell,
        line_index: usize,
        is_focused: bool,
    ) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let classes_ns_flag =
            |class_name_flags| classes_flag_namespaced(COMPONENT_NAME, class_name_flags);
        div(
            vec![
                class_ns("number__line"),
                classes_ns_flag([("line_focused", is_focused)]),
            ],
            vec![
                div(
                    vec![
                        class_ns("number"),
                        if let Some(gutter_bg) = text_buffer.gutter_background() {
                            style! {
                                background_color: gutter_bg.to_css(),
                            }
                        } else {
                            empty_attr()
                        },
                        if let Some(gutter_fg) = text_buffer.gutter_foreground() {
                            style! {
                                color: gutter_fg.to_css(),
                            }
                        } else {
                            empty_attr()
                        },
                    ],
                    vec![text(line_index + 1)],
                ),
                div(
                    vec![class_ns("line")],
                    self.ranges
                        .iter()
                        .enumerate()
                        .map(|(range_index, range)| {
                            range.view_range(
                                focus_cell,
                                is_focused && focus_cell.range_index == range_index,
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            ],
        )
    }
}

impl Default for Line {
    fn default() -> Self {
        Self {
            ranges: vec![Range::default()],
            width: 0,
        }
    }
}

impl Range {
    fn from_cells(cells: Vec<Cell>, style: Style) -> Self {
        Self {
            width: cells.iter().map(|cell| cell.width).sum(),
            cells,
            style,
        }
    }

    fn split_at(&mut self, cell_index: usize) -> Self {
        let other = self.cells.split_off(cell_index);
        Self::from_cells(other, self.style)
    }

    fn view_range<MSG>(&self, focus_cell: FocusCell, is_focused: bool) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let background = util::to_rgba(self.style.background);
        let foreground = util::to_rgba(self.style.foreground);
        div(
            vec![
                class_ns("range"),
                style! {
                    color: foreground.to_css(),
                    background_color: background.to_css(),
                },
            ],
            self.cells
                .iter()
                .enumerate()
                .map(|(cell_index, cell)| {
                    cell.view_cell(is_focused && cell_index == focus_cell.cell_index)
                })
                .collect::<Vec<_>>(),
        )
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            cells: vec![],
            width: 0,
            style: Style::default(),
        }
    }
}

impl ToString for TextBuffer {
    fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Cell {
    fn from_char(ch: char) -> Self {
        Self {
            width: ch.width().expect("must have a unicode width"),
            ch,
        }
    }

    fn view_cell<MSG>(&self, is_focused: bool) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };
        div(
            vec![
                class_ns("ch"),
                classes_ns_flag([("ch_focused", is_focused)]),
                classes_ns_flag([(&format!("wide{}", self.width), self.width > 1)]),
            ],
            if is_focused {
                vec![div(vec![class_ns("cursor")], vec![text(self.ch)])]
            } else {
                vec![text(self.ch)]
            },
        )
    }
}

impl Highlighter {
    fn highlight<'b>(&self, line: &'b str) -> Vec<(Style, &'b str)> {
        let syntax: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .expect("unable to find rust syntax reference");
        let mut highlight_line = HighlightLines::new(syntax, self.active_theme());
        highlight_line.highlight(line, &self.syntax_set)
    }
    fn active_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.theme_name]
    }
}

impl Highlighter {
    fn default() -> Self {
        let syntax_set: SyntaxSet = SyntaxSet::load_defaults_newlines();
        let theme_set: ThemeSet = ThemeSet::load_defaults();
        let theme_name = "base16-eighties.dark".to_string();
        let _active_theme = &theme_set.themes[&theme_name];

        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }
}