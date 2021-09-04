pub use super::TextHighlighter;
use crate::editor::COMPONENT_NAME;
use crate::util;
use cell::Cell;
use css_colors::RGBA;
use line::Line;
use range::Range;
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

mod cell;
mod line;
mod range;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    lines: Vec<Line>,
    text_highlighter: TextHighlighter,
    x_pos: usize,
    y_pos: usize,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    focused_cell: Option<FocusCell>,
    show_line_numbers: bool,
    /// the language to be used for highlighting the content
    syntax_token: String,
}

#[derive(Clone, Copy, Debug)]
struct FocusCell {
    line_index: usize,
    range_index: usize,
    cell_index: usize,
    cell: Option<Cell>,
}

impl TextBuffer {
    pub fn from_str(content: &str, syntax_token: &str) -> Self {
        let text_highlighter = TextHighlighter::default();
        let mut this = Self {
            lines: Self::highlight_content(
                content,
                &text_highlighter,
                syntax_token,
            ),
            text_highlighter,
            x_pos: 0,
            y_pos: 0,
            selection_start: None,
            selection_end: None,
            focused_cell: None,
            show_line_numbers: true,
            syntax_token: syntax_token.to_string(),
        };

        this.calculate_focused_cell();
        this
    }

    pub fn show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// rerun highlighter on the content
    pub(crate) fn rehighlight(&mut self) {
        self.lines = Self::highlight_content(
            &self.to_string(),
            &self.text_highlighter,
            &self.syntax_token,
        );
    }

    fn highlight_content(
        content: &str,
        text_highlighter: &TextHighlighter,
        syntax_token: &str,
    ) -> Vec<Line> {
        let (mut line_highlighter, syntax_set) =
            text_highlighter.get_line_highlighter(syntax_token);

        content
            .lines()
            .map(|line| {
                let line_str = String::from_iter(line.chars());
                let style_range: Vec<(Style, &str)> =
                    line_highlighter.highlight(&line_str, syntax_set);

                let ranges: Vec<Range> = style_range
                    .into_iter()
                    .map(|(style, range_str)| {
                        let cells =
                            range_str.chars().map(Cell::from_char).collect();
                        Range::from_cells(cells, style)
                    })
                    .collect();

                Line::from_ranges(ranges)
            })
            .collect()
    }

    fn calculate_focused_cell(&mut self) {
        self.focused_cell = self.find_focused_cell();
    }

    fn is_focused_line(&self, line_index: usize) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched_line(line_index)
        } else {
            false
        }
    }

    fn is_focused_range(&self, line_index: usize, range_index: usize) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched_range(line_index, range_index)
        } else {
            false
        }
    }

    fn is_focused_cell(
        &self,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched(line_index, range_index, cell_index)
        } else {
            false
        }
    }

    /// the cursor is in virtual position when the position
    /// has no character in it.
    pub(crate) fn is_in_virtual_position(&self) -> bool {
        self.focused_cell.is_none()
    }

    pub(crate) fn active_theme(&self) -> &Theme {
        self.text_highlighter.active_theme()
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
        if self.show_line_numbers {
            self.lines.len().to_string().len()
        } else {
            0
        }
    }

    /// the padding of the number line width
    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }

    /// This is the total width of the number line
    pub(crate) fn get_numberline_wide(&self) -> usize {
        if self.show_line_numbers {
            self.numberline_wide() + self.numberline_padding_wide()
        } else {
            0
        }
    }

    pub(crate) fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.numberline_wide());
        div(
            vec![class_ns("code"), class_ns(&class_number_wide)],
            self.lines
                .iter()
                .enumerate()
                .map(|(line_index, line)| {
                    line.view_line(&self, line_index, self.show_line_numbers)
                })
                .collect::<Vec<_>>(),
        )
    }
}

/// text manipulation
impl TextBuffer {
    /// the total number of lines of this text canvas
    pub(crate) fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the width of the line at line `n`
    pub(crate) fn line_width(&self, n: usize) -> Option<usize> {
        self.lines.get(n).map(|l| l.width)
    }

    fn find_focused_cell(&self) -> Option<FocusCell> {
        let line_index = self.y_pos;
        if let Some(line) = self.lines.get(line_index) {
            if let Some((range_index, cell_index)) =
                line.calc_range_cell_index_position(self.x_pos)
            {
                if let Some(range) = line.ranges.get(range_index) {
                    return Some(FocusCell {
                        line_index,
                        range_index,
                        cell_index,
                        cell: range.cells.get(cell_index).cloned(),
                    });
                }
            }
        }
        return None;
    }

    /// add more lines, used internally
    fn add_lines(&mut self, n: usize) {
        for _i in 0..n {
            self.lines.push(Line::default());
        }
    }

    /// fill columns at line y putting a space in each of the cells
    fn add_cell(&mut self, y: usize, n: usize) {
        let ch = ' ';
        for _i in 0..n {
            self.lines[y].push_char(ch);
        }
    }

    /// break at line y and put the characters after x on the next line
    pub(crate) fn break_line(&mut self, x: usize, y: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            let (range_index, col) = line
                .calc_range_cell_index_position(x)
                .unwrap_or(line.last_range_cell_index());
            if let Some(range_bound) = line.ranges.get_mut(range_index) {
                range_bound.recalc_width();
                let mut other = range_bound.split_at(col);
                other.recalc_width();
                let mut rest =
                    line.ranges.drain(range_index + 1..).collect::<Vec<_>>();
                rest.insert(0, other);
                self.insert_line(y + 1, Line::from_ranges(rest));
            } else {
                self.insert_line(y, Line::default());
            }
        }
    }

    fn assert_chars(&self, ch: char) {
        assert!(
            ch != '\n',
            "line breaks should have been pre-processed before this point"
        );
        assert!(
            ch != '\t',
            "tabs should have been pre-processed before this point"
        );
    }

    /// insert a character at this x and y and move cells after it to the right
    pub(crate) fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.assert_chars(ch);
        self.ensure_cell_exist(x, y);

        let (range_index, cell_index) = self.lines[y]
            .calc_range_cell_index_position(x)
            .unwrap_or(self.lines[y].last_range_cell_index());

        self.lines[y].insert_char(range_index, cell_index, ch);
    }

    /// replace the character at this location
    pub(crate) fn replace_char(&mut self, x: usize, y: usize, ch: char) {
        self.assert_chars(ch);
        self.ensure_cell_exist(x, y);

        let (range_index, cell_index) = self.lines[y]
            .calc_range_cell_index_position(x)
            .expect("the range_index and cell_index must have existed at this point");

        self.lines[y].replace_char(range_index, cell_index, ch);
    }

    /// delete character at this position
    pub(crate) fn delete_char(&mut self, x: usize, y: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            if let Some((range_index, col)) =
                line.calc_range_cell_index_position(x)
            {
                if let Some(range) = line.ranges.get_mut(range_index) {
                    if range.cells.get(col).is_some() {
                        range.cells.remove(col);
                    }
                }
            }
        }
    }

    fn ensure_cell_exist(&mut self, x: usize, y: usize) {
        self.ensure_line_exist(y);
        let cell_gap = x.saturating_sub(self.lines[y].width);
        if cell_gap > 0 {
            self.add_cell(y, cell_gap);
        }
    }

    fn ensure_line_exist(&mut self, y: usize) {
        let line_gap = y.saturating_add(1).saturating_sub(self.total_lines());
        if line_gap > 0 {
            self.add_lines(line_gap);
        }
    }

    fn insert_line(&mut self, line_index: usize, line: Line) {
        self.ensure_line_exist(line_index.saturating_sub(1));
        self.lines.insert(line_index, line);
    }

    pub(crate) fn get_position(&self) -> (usize, usize) {
        (self.x_pos, self.y_pos)
    }
}

/// Command implementation here
impl TextBuffer {
    pub(crate) fn command_insert_char(&mut self, ch: char) {
        self.insert_char(self.x_pos, self.y_pos, ch);
        self.move_right();
    }
    pub(crate) fn move_left(&mut self) {
        self.x_pos = self.x_pos.saturating_sub(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_left_start(&mut self) {
        self.x_pos = 0;
        self.calculate_focused_cell();
    }
    pub(crate) fn move_right(&mut self) {
        self.x_pos = self.x_pos.saturating_add(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_up(&mut self) {
        self.y_pos = self.y_pos.saturating_sub(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_down(&mut self) {
        self.y_pos = self.y_pos.saturating_add(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.x_pos = x;
        self.y_pos = y;
        self.calculate_focused_cell();
    }
    pub(crate) fn command_break_line(&mut self) {
        self.break_line(self.x_pos, self.y_pos);
        self.move_left_start();
        self.move_down();
    }
    pub(crate) fn command_delete_back(&mut self) {
        if self.x_pos > 0 {
            self.delete_char(self.x_pos.saturating_sub(1), self.y_pos);
            self.move_left();
        }
    }
    pub(crate) fn command_delete_forward(&mut self) {
        self.delete_char(self.x_pos, self.y_pos);
        self.calculate_focused_cell();
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

impl FocusCell {
    fn matched(
        &self,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        self.line_index == line_index
            && self.range_index == range_index
            && self.cell_index == cell_index
    }
    fn matched_line(&self, line_index: usize) -> bool {
        self.line_index == line_index
    }
    fn matched_range(&self, line_index: usize, range_index: usize) -> bool {
        self.line_index == line_index && self.range_index == range_index
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ensure_line_exist() {
        let mut buffer = TextBuffer::from_str("");
        buffer.ensure_line_exist(10);
        assert!(buffer.lines.get(10).is_some());
        assert_eq!(buffer.total_lines(), 11);
    }
}
