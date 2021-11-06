#![allow(unused)]

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
use nalgebra::Point2;
use range::Range;
use sauron::html::attributes;
use sauron::jss::jss_ns;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
use ultron_syntaxes_themes::TextHighlighter;
use ultron_syntaxes_themes::{Style, Theme};
#[allow(unused)]
use unicode_width::UnicodeWidthChar;

mod cell;
mod line;
mod range;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    options: Options,
    lines: Vec<Line>,
    text_highlighter: TextHighlighter,
    cursor: Point2<usize>,
    #[allow(unused)]
    selection_start: Option<Point2<usize>>,
    #[allow(unused)]
    selection_end: Option<Point2<usize>>,
    focused_cell: Option<FocusCell>,
}

#[derive(Clone, Copy, Debug)]
struct FocusCell {
    line_index: usize,
    range_index: usize,
    cell_index: usize,
    cell: Option<Cell>,
}

impl TextBuffer {
    pub fn from_str(options: Options, content: &str) -> Self {
        let mut text_highlighter = TextHighlighter::default();
        if let Some(theme_name) = &options.theme_name {
            log::trace!("Selecting theme: {}", theme_name);
            text_highlighter.select_theme(theme_name);
        }
        let mut this = Self {
            lines: Self::highlight_content(
                content,
                &text_highlighter,
                &options.syntax_token,
            ),
            text_highlighter,
            cursor: Point2::new(0, 0),
            selection_start: None,
            selection_end: None,
            focused_cell: None,
            options,
        };

        this.calculate_focused_cell();
        this
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn set_selection(&mut self, start: Point2<usize>, end: Point2<usize>) {
        self.selection_start = Some(start);
        self.selection_end = Some(end);
    }

    /// clear the text selection
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(Point2::new(0, 0));
        self.selection_end = Some(self.max_position());
    }

    /// return the min and max selection bound
    pub fn normalize_selection(
        &self,
    ) -> Option<(Point2<usize>, Point2<usize>)> {
        if let (Some(start), Some(end)) =
            (self.selection_start, self.selection_end)
        {
            Some(util::normalize_points(start, end))
        } else {
            None
        }
    }

    fn is_within_position(
        &self,
        (line_index, range_index, cell_index): (usize, usize, usize),
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> bool {
        let x = self.lines[line_index]
            .calc_range_cell_index_to_x(range_index, cell_index);
        let y = line_index;

        if self.options.use_block_mode {
            x >= start.x && x <= end.x && y >= start.y && y <= end.y
        } else {
            if y > start.y && y < end.y {
                true
            } else {
                let same_start_line = y == start.y;
                let same_end_line = y == end.y;

                if same_start_line && same_end_line {
                    x >= start.x && x <= end.x
                } else if same_start_line {
                    x >= start.x
                } else if same_end_line {
                    x <= end.x
                } else {
                    false
                }
            }
        }
    }

    /// check if this cell is within the selection of the textarea
    pub(crate) fn in_selection(
        &self,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        if let Some((start, end)) = self.normalize_selection() {
            self.is_within_position(
                (line_index, range_index, cell_index),
                start,
                end,
            )
        } else {
            false
        }
    }

    /// TODO: build the text using the technique used in cut_text, which is more efficient way and
    /// less code
    pub(crate) fn get_text(
        &self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        let (start, end) = util::normalize_points(start, end);
        let mut buffer = TextBuffer::from_str(Options::default(), "");
        for (line_index, line) in self.lines.iter().enumerate() {
            let y = line_index;
            for (range_index, range) in line.ranges.iter().enumerate() {
                for (cell_index, cell) in range.cells.iter().enumerate() {
                    let x = line
                        .calc_range_cell_index_to_x(range_index, cell_index);
                    if self.is_within_position(
                        (line_index, range_index, cell_index),
                        start,
                        end,
                    ) {
                        if self.options.use_block_mode {
                            buffer.insert_char(
                                x - start.x,
                                y - start.y,
                                cell.ch,
                            );
                        } else {
                            //if its not in block mode, we only deduct x if it is on the first line
                            let in_start_selection_line = y == start.y;
                            let new_x = if in_start_selection_line {
                                x - start.x
                            } else {
                                x
                            };
                            buffer.insert_char(new_x, y - start.y, cell.ch);
                        }
                    }
                }
            }
        }
        buffer.to_string()
    }

    /// Remove the text within the start and end position then return the deleted text
    pub(crate) fn cut_text(
        &mut self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        log::trace!("cutting from {} to {}", start, end);
        let deleted_text = self.get_text(start, end);
        if self.options.use_block_mode {
            for line_index in start.y..=end.y {
                println!("deleting cells in line: {}", line_index);
                self.lines[line_index].delete_cells(start.x, end.x);
            }
        } else {
            let is_one_line = start.y == end.y;
            //delete the lines in between
            if is_one_line {
                self.lines[start.y].delete_cells(start.x, end.x);
            } else {
                // at the last line of selection: delete the cells from 0 to end.x
                self.lines[end.y].delete_cells_from_start(end.x);
                // drain the lines in between
                self.lines.drain(start.y + 1..end.y);
                // at the first line of selection: delete the cells from start.x to end
                self.lines[start.y].delete_cells_to_end(start.x);
            }
        }
        deleted_text
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        if let (Some(start), Some(end)) =
            (self.selection_start, self.selection_end)
        {
            Some(self.get_text(start, end))
        } else {
            None
        }
    }

    pub(crate) fn cut_selected_text(&mut self) -> Option<String> {
        if let (Some(start), Some(end)) =
            (self.selection_start, self.selection_end)
        {
            Some(self.cut_text(start, end))
        } else {
            None
        }
    }

    /// calculate the bounds of the text_buffer
    pub fn bounds(&self) -> Point2<i32> {
        let total_lines = self.lines.len() as i32;
        let max_column =
            self.lines.iter().map(|line| line.width).max().unwrap_or(0) as i32;
        Point2::new(max_column, total_lines)
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
        let line_index = self.cursor.y;
        if let Some(line) = self.lines.get(line_index) {
            if let Some((range_index, cell_index)) =
                line.calc_range_cell_index_position(self.cursor.x)
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

        let theme_background =
            self.theme_background().unwrap_or(rgba(0, 0, 255, 1.0));

        let code_attributes = [
            class_ns("code"),
            class_ns(&class_number_wide),
            if self.options.use_background {
                style! {background: theme_background.to_css()}
            } else {
                empty_attr()
            },
        ];

        let rendered_lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| line.view_line(&self, line_index));

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

    pub fn style(&self) -> String {
        let selection_bg = self
            .selection_background()
            .unwrap_or(rgba(100, 100, 100, 0.5));

        let cursor_color = self.cursor_color().unwrap_or(rgba(255, 0, 0, 1.0));

        jss_ns! {COMPONENT_NAME,
            ".code_wrapper": {
                margin: 0,
            },

            ".code": {
                position: "relative",
                font_size: px(14),
                cursor: "text",
                display: "block",
                // to make the background color extend to the longest line, otherwise only the
                // longest lines has a background-color leaving the shorter lines ugly
                min_width: "max-content",
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
/// This are purely manipulating text into the text buffer.
/// The cursor shouldn't be move here, since it is done by the commands functions
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
            &self.options.syntax_token,
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
                .unwrap_or(line.range_cell_next());
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

    pub(crate) fn join_line(&mut self, x: usize, y: usize) {
        if self.lines.get(y + 1).is_some() {
            let next_line = self.lines.remove(y + 1);
            self.lines[y].push_ranges(next_line.ranges);
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
    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.assert_chars(ch);
        self.ensure_cell_exist(x, y);

        let (range_index, cell_index) = self.lines[y]
            .calc_range_cell_index_position(x)
            .unwrap_or(self.lines[y].range_cell_next());

        self.lines[y].insert_char(range_index, cell_index, ch);
    }

    fn insert_line_text(&mut self, x: usize, y: usize, text: &str) {
        let mut new_col = x;
        for ch in text.chars() {
            let width = ch.width().unwrap_or_else(|| {
                panic!("must have a unicode width for {:?}", ch)
            });
            self.insert_char(new_col, y, ch);
            new_col += width;
        }
    }

    pub(crate) fn insert_text(&mut self, x: usize, y: usize, text: &str) {
        self.ensure_cell_exist(x, y);
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() == 1 {
            self.insert_line_text(x, y, lines[0]);
        } else {
            dbg!(&self.lines);
            let mut new_col = x;
            let mut new_line = y;
            for (line_index, line) in lines.iter().enumerate() {
                println!("inserting {} at {},{}", line, new_col, new_line);
                if line_index + 1 < lines.len() {
                    self.break_line(new_col, new_line);
                }
                self.insert_line_text(new_col, new_line, line);
                new_col = 0;
                new_line += 1;
            }
        }
    }

    /// replace the character at this location
    pub fn replace_char(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
    ) -> Option<char> {
        self.assert_chars(ch);
        self.ensure_cell_exist(x + 1, y);

        let (range_index, cell_index) = self.lines[y]
            .calc_range_cell_index_position(x)
            .expect("the range_index and cell_index must have existed at this point");
        self.lines[y].replace_char(range_index, cell_index, ch)
    }

    //TODO: delegrate the deletion of the char to the line and range
    /// delete character at this position
    pub(crate) fn delete_char(&mut self, x: usize, y: usize) -> Option<char> {
        if let Some(line) = self.lines.get_mut(y) {
            if let Some((range_index, col)) =
                line.calc_range_cell_index_position(x)
            {
                if let Some(range) = line.ranges.get_mut(range_index) {
                    if range.cells.get(col).is_some() {
                        let cell = range.cells.remove(col);
                        return Some(cell.ch);
                    }
                }
            }
        }
        None
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

    /// return the line where the cursor is located
    fn focused_line(&self) -> Option<&Line> {
        self.lines.get(self.cursor.y)
    }

    /// return the position of the cursor
    pub(crate) fn get_position(&self) -> Point2<usize> {
        self.cursor
    }

    /// the last line and last char of the text buffer
    fn max_position(&self) -> Point2<usize> {
        let last_y = self.lines.len().saturating_sub(1);

        // if in block mode use the longest line
        let last_x = if self.options.use_block_mode {
            self.lines
                .iter()
                .map(|line| line.width.saturating_sub(1))
                .max()
                .unwrap_or(0)
        } else {
            // else use the width of the last line
            if let Some(last_line) = self.lines.get(last_y) {
                last_line.width.saturating_sub(1)
            } else {
                0
            }
        };
        Point2::new(last_x, last_y)
    }

    fn calculate_offset(&self, text: &str) -> (usize, usize) {
        let lines: Vec<&str> = text.lines().collect();
        let cols = if let Some(last_line) = lines.last() {
            last_line
                .chars()
                .map(|ch| ch.width().expect("chars must have a width"))
                .sum()
        } else {
            0
        };
        (cols, lines.len().saturating_sub(1))
    }
}

/// Command implementation here
///
/// functions that are preceeded with command also moves the
/// cursor and highlight the texts
impl TextBuffer {
    pub(crate) fn command_insert_char(&mut self, ch: char) {
        self.insert_char(self.cursor.x, self.cursor.y, ch);
        let width = ch.width().expect("must have a unicode width");
        self.move_x(width);
    }

    /// insert the character but don't move to the right
    pub(crate) fn command_insert_forward_char(&mut self, ch: char) {
        self.insert_char(self.cursor.x, self.cursor.y, ch);
    }

    pub(crate) fn command_replace_char(&mut self, ch: char) -> Option<char> {
        self.replace_char(self.cursor.x, self.cursor.y, ch)
    }

    pub(crate) fn command_insert_text(&mut self, text: &str) {
        use unicode_width::UnicodeWidthStr;
        self.insert_text(self.cursor.x, self.cursor.y, text);
        let (x, y) = self.calculate_offset(text);
        self.move_y(y);
        self.move_x(x);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_left(&mut self) {
        self.cursor.x = self.cursor.x.saturating_sub(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_left_start(&mut self) {
        self.cursor.x = 0;
        self.calculate_focused_cell();
    }

    pub(crate) fn move_right(&mut self) {
        self.cursor.x = self.cursor.x.saturating_add(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_right_end(&mut self) {
        let line_width = self.focused_line().map(|l| l.width).unwrap_or(0);
        self.cursor.x += line_width;
        self.calculate_focused_cell();
    }

    pub(crate) fn move_x(&mut self, x: usize) {
        self.cursor.x = self.cursor.x.saturating_add(x);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_y(&mut self, y: usize) {
        self.cursor.y = self.cursor.y.saturating_add(y);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_up(&mut self) {
        self.cursor.y = self.cursor.y.saturating_sub(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn move_down(&mut self) {
        self.cursor.y = self.cursor.y.saturating_add(1);
        self.calculate_focused_cell();
    }
    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.cursor.x = x;
        self.cursor.y = y;
        self.calculate_focused_cell();
    }
    pub(crate) fn command_break_line(&mut self, x: usize, y: usize) {
        self.break_line(x, y);
        self.move_left_start();
        self.move_down();
        self.calculate_focused_cell();
    }

    pub(crate) fn command_join_line(&mut self, x: usize, y: usize) {
        self.join_line(x, y);
        self.set_position(x, y);
        self.calculate_focused_cell();
    }

    pub(crate) fn command_delete_back(&mut self) -> Option<char> {
        if self.cursor.x > 0 {
            let c = self
                .delete_char(self.cursor.x.saturating_sub(1), self.cursor.y);
            self.move_left();
            c
        } else {
            None
        }
    }
    pub(crate) fn command_delete_forward(&mut self) -> Option<char> {
        let c = self.delete_char(self.cursor.x, self.cursor.y);
        self.calculate_focused_cell();
        c
    }
    pub(crate) fn command_delete_selected_forward(&mut self) -> Option<String> {
        if let Some((start, end)) = self.normalize_selection() {
            let deleted_text = self.cut_text(start, end);
            self.move_to(start);
            Some(deleted_text)
        } else {
            None
        }
    }
    pub(crate) fn move_to(&mut self, pos: Point2<usize>) {
        self.cursor.x = pos.x;
        self.cursor.y = pos.y;
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
        let mut buffer = TextBuffer::from_str(Options::default(), "");
        buffer.ensure_line_exist(10);
        assert!(buffer.lines.get(10).is_some());
        assert_eq!(buffer.total_lines(), 11);
    }
}
