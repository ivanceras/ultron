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
use page::Page;
use parry2d::bounding_volume::BoundingVolume;
use parry2d::bounding_volume::AABB;
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
mod page;
mod range;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    options: Options,
    pages: Vec<Page>,
    text_highlighter: TextHighlighter,
    cursor: Point2<usize>,
    selection_start: Option<Point2<usize>>,
    selection_end: Option<Point2<usize>>,
    focused_cell: Option<FocusCell>,
    context: Context,
}

#[derive(Clone, Default)]
pub struct Context {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub viewport_scroll_top: f32,
    pub viewport_scroll_left: f32,
}

impl Context {
    fn viewport_box(&self) -> AABB {
        AABB::new(
            Point2::new(0.0, 0.0),
            Point2::new(self.viewport_width, self.viewport_height),
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct FocusCell {
    page_index: usize,
    line_index: usize,
    range_index: usize,
    cell_index: usize,
    cell: Option<Cell>,
}

impl TextBuffer {
    pub fn from_str(options: Options, context: Context, content: &str) -> Self {
        let mut text_highlighter = TextHighlighter::default();
        if let Some(theme_name) = &options.theme_name {
            text_highlighter.select_theme(theme_name);
        }
        let lines = Self::highlight_content(
            content,
            &text_highlighter,
            &options.syntax_token,
        );
        let mut this = Self {
            context,
            pages: Page::from_lines(options.page_size, lines),
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

    pub(crate) fn update_context(&mut self, context: Context) {
        self.context = context;
        self.update_page_visibility();
    }

    fn update_page_visibility(&mut self) {
        let page_visibility: Vec<bool> = self
            .pages
            .iter()
            .enumerate()
            .map(|(page_index, _page)| self.is_page_visible(page_index))
            .collect();

        for (visible, page) in page_visibility.iter().zip(self.pages.iter_mut())
        {
            page.set_visible(*visible);
        }
    }

    fn calc_page_line_index(&self, line_index: usize) -> (usize, usize) {
        let page = line_index / self.options.page_size;
        let index = line_index % self.options.page_size;
        (page, index)
    }

    /// check if page at page_index is visible or not
    ///
    /// The page is visible if either the page_top >
    fn is_page_visible(&self, page_index: usize) -> bool {
        let page_box = self.page_box(page_index);
        let viewport_box = self.context.viewport_box();
        viewport_box.intersects(&page_box)
    }

    fn page_box(&self, page_index: usize) -> AABB {
        let page_top: u32 = self.pages[0..page_index]
            .iter()
            .map(|page| page.page_height())
            .sum();

        // distance of the page's top to the viewport top
        let distance_top = page_top as f32 - self.context.viewport_scroll_top;

        let page_height = self.pages[page_index].page_height() as f32;
        let page_width = self.pages[page_index].page_width() as f32;
        AABB::new(
            Point2::new(0.0, distance_top),
            Point2::new(page_width, distance_top + page_height),
        )
    }

    /// delete the lines starting from start_line
    fn delete_lines_to_end(&mut self, start_line: usize) {
        let (page, line_index) = self.calc_page_line_index(start_line);
        self.pages[page].delete_lines_to_end(line_index);
        self.pages.drain(page + 1..);
    }

    fn delete_lines(&mut self, start_line: usize, end_line: usize) {
        let (start_page, start_line_index) =
            self.calc_page_line_index(start_line);
        let (end_page, end_line_index) = self.calc_page_line_index(end_line);
        if start_page == end_page {
            self.pages[start_page]
                .delete_lines_exclusive(start_line_index, end_line_index);
        } else {
            self.pages[end_page].delete_lines(0, end_line_index);
            self.pages[start_page].delete_lines_to_end(start_line_index);
            self.pages.drain(start_page..end_page);
        }
    }

    fn get_text_in_lines(&self, start_line: usize, end_line: usize) -> String {
        let (start_page, start_line_index) =
            self.calc_page_line_index(start_line);
        let (end_page, end_line_index) = self.calc_page_line_index(end_line);
        if start_page == end_page {
            self.pages[start_page]
                .get_text_in_lines_exclusive(start_line_index, end_line_index)
        } else {
            let end_text =
                self.pages[end_page].get_text_in_lines(0, end_line_index);
            let start_text =
                self.pages[start_page].get_text_lines_to_end(start_line_index);

            let mid_text = self.pages[start_page..end_page]
                .iter()
                .map(|page| page.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            if mid_text.is_empty() {
                [start_text, end_text].join("\n")
            } else {
                [start_text, mid_text, end_text].join("\n")
            }
        }
    }

    pub fn clear(&mut self) {
        self.pages.clear();
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
        (page_index, line_index, range_index, cell_index): (
            usize,
            usize,
            usize,
            usize,
        ),
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> bool {
        let x = self.pages[page_index].lines[line_index]
            .calc_range_cell_index_to_x(range_index, cell_index);
        let y = self.options.page_size * page_index + line_index;

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
        page_index: usize,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        if let Some((start, end)) = self.normalize_selection() {
            self.is_within_position(
                (page_index, line_index, range_index, cell_index),
                start,
                end,
            )
        } else {
            false
        }
    }

    fn delete_cells_in_line(
        &mut self,
        line: usize,
        start_x: usize,
        end_x: usize,
    ) {
        let (page, line_index) = self.calc_page_line_index(line);
        self.pages[page].delete_cells_in_line(line_index, start_x, end_x);
    }

    fn get_text_in_line(
        &self,
        line: usize,
        start_x: usize,
        end_x: usize,
    ) -> Option<String> {
        let (page, line_index) = self.calc_page_line_index(line);
        self.pages[page].get_text_in_line(line_index, start_x, end_x)
    }

    /// delete cells in line `line` starting from cell `start_x` to end of the line
    fn delete_cells_to_end(&mut self, line: usize, start_x: usize) {
        let (page, line_index) = self.calc_page_line_index(line);
        self.pages[page].delete_cells_to_end(line_index, start_x);
    }

    fn get_text_to_end(&self, line: usize, start_x: usize) -> Option<String> {
        let (page, line_index) = self.calc_page_line_index(line);
        self.pages[page].get_text_to_end(line_index, start_x)
    }

    /// Remove the text within the start and end position then return the deleted text
    pub(crate) fn cut_text(
        &mut self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        let deleted_text = self.get_text(start, end);
        if self.options.use_block_mode {
            for line_index in start.y..=end.y {
                println!("deleting cells in line: {}", line_index);
                self.delete_cells_in_line(line_index, start.x, end.x);
            }
        } else {
            let is_one_line = start.y == end.y;
            //delete the lines in between
            if is_one_line {
                self.delete_cells_in_line(start.y, start.x, end.x);
            } else {
                // at the last line of selection: delete the cells from 0 to end.x
                self.delete_cells_in_line(end.y, 0, end.x);
                // drain the lines in between
                self.delete_lines(start.y + 1, end.y);
                // at the first line of selection: delete the cells from start.x to end
                self.delete_cells_to_end(start.y, start.x);
            }
        }
        deleted_text
    }

    pub(crate) fn get_text(
        &self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        if self.options.use_block_mode {
            (start.y..=end.y)
                .map(|line_index| {
                    self.get_text_in_line(line_index, start.x, end.x)
                        .unwrap_or(String::from(""))
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            let is_one_line = start.y == end.y;
            //delete the lines in between
            if is_one_line {
                self.get_text_in_line(start.y, start.x, end.x)
                    .unwrap_or(String::from(""))
            } else {
                let end_text = self
                    .get_text_in_line(end.y, 0, end.x)
                    .unwrap_or(String::from(""));

                let mid_text = self.get_text_in_lines(start.y + 1, end.y);

                let start_text = self
                    .get_text_to_end(start.y, start.x)
                    .unwrap_or(String::from(""));
                if mid_text.is_empty() {
                    [start_text, end_text].join("\n")
                } else {
                    [start_text, mid_text, end_text].join("\n")
                }
            }
        }
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
        let total_lines = self.total_lines() as i32;
        let max_column = self.max_column() as i32;
        Point2::new(max_column, total_lines)
    }

    pub fn max_column(&self) -> usize {
        self.pages
            .iter()
            .map(|page| page.max_column())
            .max()
            .unwrap_or(0)
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

    fn get_line(&self, line: usize) -> Option<&Line> {
        let (page_index, line_index) = self.calc_page_line_index(line);
        self.pages
            .get(page_index)
            .map(|page| page.lines.get(line_index))
            .flatten()
    }

    fn get_line_mut(&mut self, line: usize) -> Option<&mut Line> {
        let (page, line_index) = self.calc_page_line_index(line);
        self.pages
            .get_mut(page)
            .map(|page| page.lines.get_mut(line_index))
            .flatten()
    }

    fn find_focused_cell(&self) -> Option<FocusCell> {
        let line = self.cursor.y;
        let (page_index, line_index) = self.calc_page_line_index(line);
        if let Some(line) = self.get_line(line) {
            if let Some((range_index, cell_index)) =
                line.calc_range_cell_index_position(self.cursor.x)
            {
                if let Some(range) = line.ranges.get(range_index) {
                    return Some(FocusCell {
                        page_index,
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

    fn is_focused_line(&self, page_index: usize, line_index: usize) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched_line(page_index, line_index)
        } else {
            false
        }
    }

    fn is_focused_range(
        &self,
        page_index: usize,
        line_index: usize,
        range_index: usize,
    ) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched_range(page_index, line_index, range_index)
        } else {
            false
        }
    }

    fn is_focused_cell(
        &self,
        page_index: usize,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        if let Some(focused_cell) = self.focused_cell {
            focused_cell.matched(
                page_index,
                line_index,
                range_index,
                cell_index,
            )
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
            self.total_lines().to_string().len()
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

        let rendered_pages = self
            .pages
            .iter()
            .enumerate()
            .map(|(page_index, page)| page.view_page(&self, page_index));

        if self.options.use_for_ssg {
            // using div works well when select-copying for both chrome and firefox
            // this is ideal for statis site generation highlighting
            div(code_attributes, rendered_pages)
        } else {
            // using <pre><code> works well when copying in chrome
            // but in firefox, it creates a double line when select-copying the text
            // whe need to use <pre><code> in order for typing whitespace works.
            pre(
                [class_ns("code_wrapper")],
                [code(code_attributes, rendered_pages)],
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
        self.pages.iter().map(|page| page.total_lines()).sum()
    }

    /// the cursor is in virtual position when the position
    /// has no character in it.
    pub(crate) fn is_in_virtual_position(&self) -> bool {
        self.focused_cell.is_none()
    }

    /// rerun highlighter on the content
    pub(crate) fn rehighlight(&mut self) {
        let lines = Self::highlight_content(
            &self.to_string(),
            &self.text_highlighter,
            &self.options.syntax_token,
        );
        self.pages = Page::from_lines(self.options.page_size, lines);
        self.calculate_focused_cell();
    }

    /// the width of the line at line `n`
    pub(crate) fn line_width(&self, n: usize) -> Option<usize> {
        let (page, line_index) = self.calc_page_line_index(n);
        self.pages[page].line_width(line_index)
    }

    /// add more lines, used internally
    fn add_lines(&mut self, n: usize) {
        if let Some(last) = self.pages.last_mut() {
            let last_total_lines = last.total_lines();
            if last_total_lines < self.options.page_size {
                let capacity =
                    self.options.page_size as i32 - last_total_lines as i32;
                let to_add = n.min(capacity as usize);
                last.add_lines(to_add);
                let excess = n as i32 - capacity as i32;
                if excess > 0 {
                    self.add_lines(excess as usize)
                }
            } else {
                self.add_page(1);
                self.add_lines(n);
            }
        } else {
            self.add_page(1);
            self.add_lines(n);
        }
    }

    /// fill columns at line y putting a space in each of the cells
    fn add_cell(&mut self, y: usize, n: usize) {
        let (page, line_index) = self.calc_page_line_index(y);
        self.pages[page].add_cell(line_index, n);
    }

    /// break at line y and put the characters after x on the next line
    pub(crate) fn break_line(&mut self, x: usize, y: usize) {
        if let Some(line) = self.get_line_mut(y) {
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
                //self.insert_line(y, Line::default());
                line.push_range(Range::default());
                self.insert_line(y + 1, Line::default());
            }
        } else {
            log::error!("There is no line {}", y);
        }
    }

    pub(crate) fn join_line(&mut self, x: usize, y: usize) {
        let (page, line_index) = self.calc_page_line_index(y);
        self.pages[page].join_line(x, line_index);
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
        let (page, line_index) = self.calc_page_line_index(y);
        self.pages[page].insert_char_to_line(line_index, x, ch);
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

        let (page, line_index) = self.calc_page_line_index(y);
        self.pages[page].replace_char_to_line(line_index, x, ch)
    }

    //TODO: delegrate the deletion of the char to the line and range
    /// delete character at this position
    pub(crate) fn delete_char(&mut self, x: usize, y: usize) -> Option<char> {
        let (page, line_index) = self.calc_page_line_index(y);
        let c = self.pages[page].delete_char_to_line(line_index, x);
        self.calculate_focused_cell();
        c
    }

    /// return true if the the cell already exist, false if the cell doesn't exist and needs to add
    /// more char
    fn ensure_cell_exist(&mut self, x: usize, y: usize) {
        self.ensure_line_exist(y);
        let cell_gap = x.saturating_sub(self.get_line(y).unwrap().width);
        self.add_cell(y, cell_gap);
    }

    fn cell_exist(&self, x: usize, y: usize) -> bool {
        if let Some(line) = self.get_line(y) {
            line.width >= x
        } else {
            false
        }
    }

    fn ensure_line_exist(&mut self, y: usize) {
        let (page, line_index) = self.calc_page_line_index(y);
        self.ensure_page_exist(page);
        let line_gap = line_index
            .saturating_add(1)
            .saturating_sub(self.total_lines());
        if line_gap > 0 {
            self.add_lines(line_gap);
        }
    }

    fn total_pages(&self) -> usize {
        self.pages.len()
    }

    fn ensure_page_exist(&mut self, page_index: usize) {
        let page_gap = page_index
            .saturating_add(1)
            .saturating_sub(self.total_pages());
        if page_gap > 0 {
            self.add_page(page_gap);
        }
    }

    fn add_page(&mut self, n_pages: usize) {
        for _ in 0..n_pages {
            self.pages.push(Page::default());
        }
    }

    /// insert a line at this line: y
    fn insert_line(&mut self, y: usize, line: Line) {
        self.ensure_line_exist(y.saturating_sub(1));
        let (page, line_index) = self.calc_page_line_index(y);
        self.pages[page].insert_line(line_index, line);
    }

    /// return the line where the cursor is located
    fn focused_line(&self) -> Option<&Line> {
        self.get_line(self.cursor.y)
    }

    /// return the position of the cursor
    pub(crate) fn get_position(&self) -> Point2<usize> {
        self.cursor
    }

    /// the last line and last char of the text buffer
    fn max_position(&self) -> Point2<usize> {
        let last_y = self.total_lines().saturating_sub(1);

        // if in block mode use the longest line
        let last_x = if self.options.use_block_mode {
            self.pages
                .iter()
                .map(|page| page.page_width().saturating_sub(1))
                .max()
                .unwrap_or(0)
        } else {
            // else use the width of the last line
            if let Some(last_line) = self.get_line(last_y) {
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
    pub(crate) fn move_right_clamped(&mut self) {
        if self.cursor.x < self.max_column() {
            self.move_right();
        }
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
    pub(crate) fn move_down_clamped(&mut self) {
        if self.cursor.y + 1 < self.total_lines() {
            self.move_down()
        }
    }
    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.cursor.x = x;
        self.cursor.y = y;
        self.calculate_focused_cell();
    }

    /// set the position to the max_column of the line if it is out of
    /// bounds
    pub(crate) fn set_position_clamped(&mut self, mut x: usize, mut y: usize) {
        let total_lines = self.total_lines();
        if y > total_lines {
            y = total_lines - 1;
        }
        let (page, line_index) = self.calc_page_line_index(y);
        let line_width = self.pages[page].line_width(line_index).unwrap();
        if x > line_width {
            x = line_width - 1;
        }
        self.set_position(x, y)
    }

    pub(crate) fn command_break_line(&mut self, x: usize, y: usize) {
        self.break_line(x, y);
        self.move_left_start();
        self.move_down();
    }

    pub(crate) fn command_join_line(&mut self, x: usize, y: usize) {
        self.join_line(x, y);
        self.set_position(x, y);
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
        self.set_position(pos.x, pos.y);
    }
}

impl ToString for TextBuffer {
    fn to_string(&self) -> String {
        self.pages
            .iter()
            .map(|page| page.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl FocusCell {
    fn matched(
        &self,
        page_index: usize,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
    ) -> bool {
        self.page_index == page_index
            && self.line_index == line_index
            && self.range_index == range_index
            && self.cell_index == cell_index
    }
    fn matched_page(&self, page_index: usize) -> bool {
        self.page_index == page_index
    }
    fn matched_line(&self, page_index: usize, line_index: usize) -> bool {
        self.matched_page(page_index) && self.line_index == line_index
    }
    fn matched_range(
        &self,
        page_index: usize,
        line_index: usize,
        range_index: usize,
    ) -> bool {
        self.matched_page(page_index)
            && self.matched_line(page_index, line_index)
            && self.range_index == range_index
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ensure_line_exist() {
        let mut buffer =
            TextBuffer::from_str(Options::default(), Context::default(), "");
        buffer.ensure_line_exist(10);
        assert!(buffer.pages[0].lines.get(10).is_some());
        assert_eq!(buffer.total_lines(), 11);
    }
}
