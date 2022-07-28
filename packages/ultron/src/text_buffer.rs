#![allow(unused)]

use crate::{util, Options, CH_HEIGHT, CH_WIDTH, COMPONENT_NAME};
use css_colors::{rgba, Color, RGBA};
use nalgebra::Point2;
use parry2d::bounding_volume::{BoundingVolume, AABB};
use sauron::{html::attributes, jss_ns, prelude::*, Node};
use std::{collections::HashMap, iter::FromIterator};
use ultron_syntaxes_themes::{Style, TextHighlighter, Theme};
use unicode_width::UnicodeWidthChar;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextBuffer {
    options: Options,
    chars: Vec<Vec<Ch>>,
    cursor: Point2<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Ch {
    ch: char,
    width: usize,
}

impl Ch {
    fn new(ch: char) -> Self {
        Self {
            width: ch.width().unwrap_or(0),
            ch,
        }
    }
}

#[derive(Copy, Clone, Default)]
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

impl TextBuffer {
    pub fn from_str(options: Options, context: Context, content: &str) -> Self {
        Self {
            options,
            chars: content
                .lines()
                .map(|line| line.chars().map(|ch| Ch::new(ch)).collect())
                .collect(),
            cursor: Point2::new(0, 0),
        }
    }

    pub(crate) fn calculate_cursor_location(&self) -> Point2<f32> {
        Point2::new(
            self.cursor.x as f32 * CH_WIDTH as f32,
            self.cursor.y as f32 * CH_HEIGHT as f32,
        )
    }
    pub fn set_selection(&mut self, start: Point2<usize>, end: Point2<usize>) {}
    /// clear the text selection
    pub fn clear_selection(&mut self) {}
    pub fn select_all(&mut self) {}
    /// return the min and max selection bound
    pub fn normalize_selection(
        &self,
    ) -> Option<(Point2<usize>, Point2<usize>)> {
        None
    }
    /// Remove the text within the start and end position then return the deleted text
    pub(crate) fn cut_text(
        &mut self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        "".to_string()
    }

    pub(crate) fn get_text(
        &self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        "".to_string()
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        None
    }
    pub(crate) fn cut_selected_text(&mut self) -> Option<String> {
        None
    }
    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }
    fn calculate_focused_cell(&mut self) {}
    pub(crate) fn cursor_color(&self) -> Option<RGBA> {
        None
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
    pub(crate) fn get_numberline_wide(&self) -> usize {
        0
    }

    pub fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let class_number_wide =
            format!("number_wide{}", self.numberline_wide());

        /*
        let theme_background =
            self.theme_background().unwrap_or(rgba(0, 0, 255, 1.0));
        */

        let code_attributes = [
            class_ns("code"),
            class_ns(&class_number_wide),
            /*
            if self.options.use_background {
                style! {background: theme_background.to_css()}
            } else {
                empty_attr()
            },
            */
        ];

        let rendered_pages =
            self.chars.iter().enumerate().map(|(number, line)| {
                div(
                    [class_ns("line")],
                    line.iter().map(|ch| div([class_ns("ch")], [text(ch.ch)])),
                )
            });

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
        /*
        let selection_bg = self
            .selection_background()
            .unwrap_or(rgba(100, 100, 100, 0.5));
        */

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
                display: "block",
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

            /*
            ".line .ch::selection": {
                "background-color": selection_bg.to_css(),
            },

            ".ch.selected": {
                background_color:selection_bg.to_css(),
            },
            */

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
        self.chars.len()
    }

    fn lines(&self) -> Vec<String> {
        self.chars
            .iter()
            .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
            .collect()
    }

    /// the width of the line at line `n`
    pub(crate) fn line_width(&self, n: usize) -> usize {
        self.chars
            .get(n)
            .map(|line| line.iter().map(|ch| ch.width).sum())
            .unwrap_or(0)
    }

    /// fill columns at line y putting a space in each of the cells
    fn add_cell(&mut self, y: usize, n: usize) {}

    /// break at line y and put the characters after x on the next line
    pub(crate) fn break_line(&mut self, x: usize, y: usize) {
        println!("lines before breaking line: {:#?}", self.lines());
        self.ensure_before_cell_exist(x, y);
        let line = &self.chars[y];
        if let Some(break_point) = self.column_index(x, y) {
            let (break1, break2): (Vec<_>, Vec<_>) = line
                .iter()
                .enumerate()
                .partition(|(i, ch)| *i < break_point);

            let break1: Vec<Ch> =
                break1.into_iter().map(|(_, ch)| *ch).collect();
            let break2: Vec<Ch> =
                break2.into_iter().map(|(_, ch)| *ch).collect();
            dbg!(&break1);
            dbg!(&break2);
            println!("lines before removing: {:#?}", self.lines());
            self.chars.remove(y);
            println!("lines after removing: {:#?}", self.lines());
            self.chars.insert(y, break2);
            println!("lines after inserting break2: {:#?}", self.lines());
            self.chars.insert(y, break1);
            println!("lines after inserting break1: {:#?}", self.lines());
        } else {
            println!(
                "no column index.. inserting a blank line to the next line"
            );
            self.chars.insert(y + 1, vec![]);
        }
    }

    pub(crate) fn join_line(&mut self, x: usize, y: usize) {}

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

    /// ensure line at index y exist
    pub fn ensure_line_exist(&mut self, y: usize) {
        println!("ensuring line {} exist", y);
        let total_lines = self.total_lines();
        let diff = y.saturating_add(1).saturating_sub(total_lines);
        println!("adding {} lines", diff);
        for _ in 0..diff {
            self.chars.push(vec![]);
        }
    }

    pub fn ensure_before_line_exist(&mut self, y: usize) {
        println!("ensuring before line {} exist", y);
        if y > 0 {
            self.ensure_line_exist(y.saturating_sub(1));
        }
    }

    /// ensure line in index y exist and the cell at index x
    pub fn ensure_cell_exist(&mut self, x: usize, y: usize) {
        println!("ensuring {},{} exist", x, y);
        self.ensure_line_exist(y);
        let line_width = self.line_width(y);
        let diff = x.saturating_add(1).saturating_sub(line_width);
        println!("adding {} columns to line {}", diff, y);
        for _ in 0..diff {
            self.chars[y].push(Ch::new(' '));
        }
    }

    pub fn ensure_before_cell_exist(&mut self, x: usize, y: usize) {
        println!("ensuring before {},{} exist", x, y);
        self.ensure_line_exist(y);
        if x > 0 {
            self.ensure_cell_exist(x.saturating_sub(1), y);
        }
    }

    /// calculate the column index base on position of x and y
    fn column_index(&self, x: usize, y: usize) -> Option<usize> {
        if let Some(line) = self.chars.get(y) {
            let mut width_sum = 0;
            for (i, ch) in line.iter().enumerate() {
                if width_sum == x {
                    return Some(i);
                }
                width_sum += ch.width;
            }
            println!("reach the end of the loop for column_index");
            None
        } else {
            println!("no line for this column_index");
            None
        }
    }

    /// insert a character at this x and y and move cells after it to the right
    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.ensure_before_cell_exist(x, y);
        let new_ch = Ch::new(ch);
        let line_width = self.line_width(y);
        if let Some(column_index) = self.column_index(x, y) {
            let diff =
                x.saturating_sub(column_index).saturating_sub(new_ch.width);
            let insert_index = column_index;
            self.chars[y].insert(insert_index, new_ch);
        } else {
            self.chars[y].push(new_ch);
        }
    }

    /// insert a text, must not contain a \n
    fn insert_line_text(&mut self, x: usize, y: usize, text: &str) {
        let mut width_inc = 0;
        for ch in text.chars() {
            let new_ch = Ch::new(ch);
            self.insert_char(x + width_inc, y, new_ch.ch);
            width_inc += new_ch.width;
        }
    }

    pub(crate) fn insert_text(&mut self, x: usize, y: usize, text: &str) {
        println!("before inserting text: {:#?}", self.lines());
        let mut start = x;
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                self.chars.insert(y + 1, vec![]);
            }
            self.insert_line_text(start, y + i, line);
            start = 0;
        }
        println!("after inserting text: {:#?}", self.lines());
    }

    /// replace the character at this location
    pub fn replace_char(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
    ) -> Option<char> {
        self.ensure_cell_exist(x, y);
        let new_ch = Ch::new(ch);
        let column_index =
            self.column_index(x, y).expect("must have a column index");
        let ex_ch = self.chars[y].remove(column_index);
        self.chars[y].insert(column_index, Ch::new(ch));
        Some(ex_ch.ch)
    }

    /// delete character at this position
    pub(crate) fn delete_char(&mut self, x: usize, y: usize) -> Option<char> {
        if let Some(column_index) = self.column_index(x, y) {
            let ex_ch = self.chars[y].remove(column_index);
            Some(ex_ch.ch)
        } else {
            None
        }
    }

    /// return the position of the cursor
    pub(crate) fn get_position(&self) -> Point2<usize> {
        self.cursor
    }

    fn calculate_offset(&self, text: &str) -> (usize, usize) {
        (0, 0)
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
        let line_column = self
            .chars
            .get(self.cursor.y)
            .map(|line| line.len())
            .unwrap_or(0);
        if self.cursor.x < line_column {
            self.move_right();
        }
    }

    pub(crate) fn move_right_end(&mut self) {
        //let line_width = self.focused_line().map(|l| l.width).unwrap_or(0);
        //self.cursor.x += line_width;
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
            y = total_lines.saturating_sub(1);
        }
        let line_width = self.line_width(y);
        if x > line_width {
            x = line_width.saturating_sub(1);
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

    pub fn clear(&mut self) {
        self.chars.clear();
    }
}

impl ToString for TextBuffer {
    fn to_string(&self) -> String {
        self.chars
            .iter()
            .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
