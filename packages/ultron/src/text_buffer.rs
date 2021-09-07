use crate::util;
use crate::Options;
use crate::CH_HEIGHT;
use crate::CH_WIDTH;
use crate::COMPONENT_NAME;
use cell::Cell;
use css_colors::rgba;
use css_colors::Color;
use css_colors::RGBA;
use line::Line;
use range::Range;
use sauron::html::attributes;
use sauron::jss::jss_ns;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
use syntect::highlighting::Style;
use syntect::highlighting::Theme;
use text_highlighter::TextHighlighter;
#[allow(unused)]
use unicode_width::UnicodeWidthChar;

mod cell;
mod line;
mod range;
mod text_highlighter;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    options: Options,
    lines: Vec<Line>,
    text_highlighter: TextHighlighter,
    x_pos: usize,
    y_pos: usize,
    #[allow(unused)]
    selection_start: Option<(usize, usize)>,
    #[allow(unused)]
    selection_end: Option<(usize, usize)>,
    focused_cell: Option<FocusCell>,
    /// the language to be used for highlighting the content
    #[allow(unused)]
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
    pub fn from_str(
        options: Options,
        content: &str,
        syntax_token: &str,
    ) -> Self {
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
            options,
            syntax_token: syntax_token.to_string(),
        };

        this.calculate_focused_cell();
        this
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = options;
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

    pub(crate) fn active_theme(&self) -> &Theme {
        self.text_highlighter.active_theme()
    }

    pub(crate) fn gutter_background(&self) -> Option<RGBA> {
        self.active_theme().settings.gutter.map(util::to_rgba)
    }

    pub(crate) fn gutter_foreground(&self) -> Option<RGBA> {
        self.active_theme()
            .settings
            .gutter_foreground
            .map(util::to_rgba)
    }

    pub(crate) fn theme_background(&self) -> Option<RGBA> {
        self.active_theme().settings.background.map(util::to_rgba)
    }

    pub(crate) fn selection_background(&self) -> Option<RGBA> {
        self.active_theme().settings.selection.map(util::to_rgba)
    }

    #[allow(unused)]
    pub(crate) fn selection_foreground(&self) -> Option<RGBA> {
        self.active_theme()
            .settings
            .selection_foreground
            .map(util::to_rgba)
    }

    pub(crate) fn cursor_color(&self) -> Option<RGBA> {
        self.active_theme().settings.caret.map(util::to_rgba)
    }

    /// how wide the numberline based on the character lengths of the number
    fn numberline_wide(&self) -> usize {
        if self.options.show_line_numbers {
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
    #[allow(unused)]
    pub(crate) fn get_numberline_wide(&self) -> usize {
        if self.options.show_line_numbers {
            self.numberline_wide() + self.numberline_padding_wide()
        } else {
            0
        }
    }

    pub fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.numberline_wide());

        let code_attributes =
            vec![class_ns("code"), class_ns(&class_number_wide)];

        let rendered_lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| line.view_line(&self, line_index))
            .collect::<Vec<_>>();

        if self.options.use_for_ssg {
            // using div works well when select-copying for both chrome and firefox
            // this is ideal for statis site generation highlighting
            div(code_attributes, rendered_lines)
        } else {
            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                vec![class_ns("code_wrapper")],
                vec![code(code_attributes, rendered_lines)],
            )
        }
    }

    pub fn style(&self) -> String {
        let selection_bg = self
            .selection_background()
            .unwrap_or(rgba(100, 100, 100, 0.5));

        let cursor_color = self.cursor_color().unwrap_or(rgba(255, 0, 0, 1.0));
        let theme_background =
            self.theme_background().unwrap_or(rgba(0, 0, 255, 1.0));

        jss_ns! {COMPONENT_NAME,
            ".code_wrapper": {
                margin: 0,
            },

            ".code": {
                position: "relative",
                background: theme_background.to_css(),
                font_size: px(14),
                cursor: "text",
                display: "block",
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
                background_color: "#002b36",
                padding_right: px(CH_WIDTH * self.numberline_padding_wide() as u32),
                height: px(CH_HEIGHT),
                user_select: "none",
            },
            ".number_wide1 .number": {
                width: px(1 * CH_WIDTH),
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
                overflow: "hidden",
                display: "inline-block",
            },

            ".filler": {
                width: percent(100),
            },

            ".line_focused": {
            },

            ".range": {
                flex: "none",
                height: px(CH_HEIGHT),
                overflow: "hidden",
                display: "inline-block",
            },

            ".line .ch": {
                width: px(CH_WIDTH),
                height: px(CH_HEIGHT),
                font_stretch: "ultra-condensed",
                font_variant_numeric: "slashed-zero",
                font_kerning: "none",
                font_size_adjust: "none",
                font_optical_sizing: "none",
                position: "relative",
                overflow: "hidden",
                align_items: "center",
                line_height: 1,
                display: "inline-block",
            },

            ".line .ch::selection": {
                "background-color": selection_bg.to_css(),
            },

            ".ch.selected": {
                background_color:selection_bg.to_css(),
            },

            ".virtual_cursor": {
                position: "absolute",
                width: px(CH_WIDTH),
                height: px(CH_HEIGHT),
                background_color: cursor_color.to_css(),
            },

            ".ch .cursor": {
                position: "absolute",
                left: 0,
                width : px(CH_WIDTH),
                height: px(CH_HEIGHT),
                background_color: cursor_color.to_css(),
                display: "inline",
                animation: "cursor_blink-anim 1000ms step-end infinite",
            },

            ".ch.wide2 .cursor": {
                width: px(2 * CH_WIDTH),
            },
            ".ch.wide3 .cursor": {
                width: px(3 * CH_WIDTH),
            },

            // i-beam cursor
            ".thin_cursor .cursor": {
                width: px(2),
            },

            ".block_cursor .cursor": {
                width: px(CH_WIDTH),
            },


            ".line .ch.wide2": {
                width: px(2 * CH_WIDTH),
                font_size: px(13),
            },

            ".line .ch.wide3": {
                width: px(3 * CH_WIDTH),
                font_size: px(13),
            },

            "@keyframes cursor_blink-anim": {
              "50%": {
                background_color: "transparent",
                border_color: "transparent",
              },

              "100%": {
                background_color: cursor_color.to_css(),
                border_color: "transparent",
              },
            },
        }
    }
}

/// text manipulation
#[cfg(feature = "with-dom")]
impl TextBuffer {
    /// the total number of lines of this text canvas
    pub(crate) fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the cursor is in virtual position when the position
    /// has no character in it.
    pub(crate) fn is_in_virtual_position(&self) -> bool {
        self.focused_cell.is_none()
    }

    /// rerun highlighter on the content
    pub(crate) fn rehighlight(&mut self) {
        self.lines = Self::highlight_content(
            &self.to_string(),
            &self.text_highlighter,
            &self.syntax_token,
        );
    }

    /// the width of the line at line `n`
    #[allow(unused)]
    pub(crate) fn line_width(&self, n: usize) -> Option<usize> {
        self.lines.get(n).map(|l| l.width)
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

    pub(crate) fn insert_text(&mut self, x: usize, y: usize, text: &str) {
        let mut new_line = y;
        let mut new_col = x;
        let lines: Vec<&str> = text.lines().collect();
        let is_multi_line = lines.len() > 1;
        if is_multi_line {
            self.break_line(x, y);
        }
        for line in lines {
            for ch in line.chars() {
                let width = ch.width().unwrap_or_else(|| {
                    panic!("must have a unicode width for {:?}", ch)
                });
                self.insert_char(new_col, new_line, ch);
                new_col += width;
            }
            new_col = 0;
            new_line += 1;
            self.break_line(new_col, new_line);
            self.y_pos = new_line;
        }
    }

    /// replace the character at this location
    #[allow(unused)]
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

#[cfg(feature = "with-dom")]
/// Command implementation here
impl TextBuffer {
    pub(crate) fn command_insert_char(&mut self, ch: char) {
        self.insert_char(self.x_pos, self.y_pos, ch);
        let width = ch.width().expect("must have a unicode width");
        self.move_x(width);
    }
    pub(crate) fn command_insert_text(&mut self, text: &str) {
        use unicode_width::UnicodeWidthStr;
        self.insert_text(self.x_pos, self.y_pos, text);
        self.move_x(text.width());
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
    pub(crate) fn move_x(&mut self, x: usize) {
        self.x_pos = self.x_pos.saturating_add(x);
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
        let mut buffer = TextBuffer::from_str(Options::default(), "", "txt");
        buffer.ensure_line_exist(10);
        assert!(buffer.lines.get(10).is_some());
        assert_eq!(buffer.total_lines(), 11);
    }
}
