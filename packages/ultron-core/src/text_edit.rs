use crate::{util, TextBuffer};
pub use action::Action;
pub use history::Recorded;
use nalgebra::Point2;
use std::fmt;
use unicode_width::UnicodeWidthChar;

mod action;
mod history;

/// A struct with text_buffer, selection commands, and history recording for undo and redo editing
pub struct TextEdit {
    pub text_buffer: TextBuffer,
    /// for undo and redo
    recorded: Recorded,
    selection: Selection,
}

#[derive(Default)]
pub struct Selection {
    pub start: Option<Point2<i32>>,
    pub end: Option<Point2<i32>>,
    pub mode: SelectionMode,
}

impl fmt::Debug for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start = if let Some(start) = self.start {
            format!("({},{})", start.x, start.y)
        } else {
            format!("..")
        };
        let end = if let Some(end) = self.end {
            format!("({},{})", end.x, end.y)
        } else {
            format!("..")
        };

        write!(f, "{:?}: {} -> {}", self.mode, start, end)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SelectionMode {
    Linear,
    Block,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Linear
    }
}

impl TextEdit {
    pub fn from_str(content: &str) -> Self {
        let text_buffer = TextBuffer::from_str(content);
        TextEdit {
            text_buffer,
            recorded: Recorded::new(),
            selection: Selection::default(),
        }
    }

    pub fn text_buffer(&self) -> &TextBuffer {
        &self.text_buffer
    }

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.selection.start = Some(start);
        self.selection.end = Some(end);
    }

    pub fn set_selection_start(&mut self, start: Point2<i32>) {
        self.selection.start = Some(start);
    }

    pub fn clear(&mut self) {
        self.text_buffer.clear();
        self.clear_selection();
        self.recorded.clear();
    }

    pub fn set_selection_end(&mut self, end: Point2<i32>) {
        self.selection.end = Some(end);
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn command_insert_char(&mut self, ch: char) {
        let cursor = self.text_buffer.get_position();
        self.text_buffer.command_insert_char(ch);
        self.recorded.insert_char(cursor, ch);
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        self.text_buffer.get_char(x, y)
    }

    pub fn command_smart_replace_insert_char(&mut self, ch: char) {
        let cursor = self.text_buffer.get_position();
        let has_right_char = if let Some(ch) = self.get_char(cursor.x + 1, cursor.y) {
            !ch.is_whitespace()
        } else {
            false
        };
        if has_right_char {
            self.command_insert_char(ch);
        } else {
            self.command_replace_char(ch);
            self.command_move_right();
        }
    }

    pub fn command_replace_char(&mut self, ch: char) {
        let cursor = self.text_buffer.get_position();
        if let Some(old_ch) = self.text_buffer.command_replace_char(ch) {
            self.recorded.replace_char(cursor, old_ch, ch);
        }
    }

    pub fn command_delete_back(&mut self) {
        let ch = self.text_buffer.command_delete_back();
        let cursor = self.text_buffer.get_position();
        self.recorded.delete(cursor, ch);
    }

    pub fn command_delete_forward(&mut self) {
        let ch = self.text_buffer.command_delete_forward();
        let cursor = self.text_buffer.get_position();
        self.recorded.delete(cursor, ch);
    }

    pub fn command_move_up(&mut self) {
        self.text_buffer.move_up();
    }

    pub fn command_move_up_clamped(&mut self) {
        self.text_buffer.move_up_clamped();
    }

    pub fn command_move_down(&mut self) {
        self.text_buffer.move_down();
    }

    pub fn command_move_down_clamped(&mut self) {
        self.text_buffer.move_down_clamped();
    }

    pub fn command_move_left(&mut self) {
        self.text_buffer.move_left();
    }

    pub fn command_move_left_start(&mut self) {
        self.text_buffer.move_left_start();
    }

    pub fn command_move_right(&mut self) {
        self.text_buffer.move_right();
    }

    pub fn command_move_right_end(&mut self) {
        self.text_buffer.move_right_end();
    }

    pub fn command_move_right_clamped(&mut self) {
        self.text_buffer.move_right_clamped();
    }

    pub fn command_break_line(&mut self) {
        let pos = self.text_buffer.get_position();
        self.text_buffer.command_break_line(pos.x, pos.y);
        self.recorded.break_line(pos);
    }

    pub fn command_join_line(&mut self) {
        let pos = self.text_buffer.get_position();
        self.text_buffer.command_join_line(pos.x, pos.y);
        self.recorded.join_line(pos);
    }

    pub fn command_insert_text(&mut self, text: &str) {
        self.text_buffer.command_insert_text(text);
    }

    //TODO: use Point2<usize>
    pub fn command_set_position(&mut self, cursor_x: usize, cursor_y: usize) {
        self.text_buffer.set_position(cursor_x, cursor_y);
    }

    //TODO: use Point2<usize>
    pub fn command_set_position_clamped(&mut self, cursor_x: usize, cursor_y: usize) {
        self.text_buffer.set_position_clamped(cursor_x, cursor_y);
    }

    pub fn command_set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.set_selection(start, end)
    }

    pub fn command_select_all(&mut self) {
        let start = Point2::new(0, 0);
        let max = self.text_buffer.last_char_position();
        let end = Point2::new(max.x as i32, max.y as i32);
        self.set_selection(start, end);
    }

    pub fn command_select_all_block_mode(&mut self) {
        let start = Point2::new(0, 0);
        let max = self.text_buffer.max_position();
        let end = Point2::new(max.x as i32, max.y as i32);
        self.set_selection(start, end);
    }

    /// Make a history separator for the undo/redo
    /// This is used for breaking undo action list
    pub fn bump_history(&mut self) {
        self.recorded.bump_history();
    }

    pub fn command_undo(&mut self) {
        if let Some(location) = self.recorded.undo(&mut self.text_buffer) {
            self.text_buffer.set_position(location.x, location.y);
        }
    }

    pub fn command_redo(&mut self) {
        if let Some(location) = self.recorded.redo(&mut self.text_buffer) {
            self.text_buffer.set_position(location.x, location.y);
        }
    }

    pub fn set_selection_mode(&mut self, mode: SelectionMode) {
        self.selection.mode = mode;
    }

    pub fn is_selected(&self, loc: Point2<i32>) -> bool {
        match self.selection.mode {
            SelectionMode::Linear => self.is_selected_in_linear_mode(loc),
            SelectionMode::Block => self.is_selected_in_block_mode(loc),
        }
    }

    pub fn is_selected_in_linear_mode(&self, loc: Point2<i32>) -> bool {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => {
                let (start, end) = util::normalize_points(start, end);
                let only_one_line = start.y == end.y;
                let in_first_line = loc.y == start.y;
                let in_inner_line = loc.y > start.y && loc.y < end.y;
                let in_last_line = loc.y == end.y;
                if in_first_line {
                    if only_one_line {
                        loc.x >= start.x && loc.x <= end.x
                    } else {
                        loc.x >= start.x
                    }
                } else if in_inner_line {
                    true
                } else if in_last_line {
                    if only_one_line {
                        loc.x >= start.x && loc.x <= end.x
                    } else {
                        loc.x <= end.x
                    }
                } else {
                    // outside line
                    false
                }
            }
            _ => false,
        }
    }

    pub fn is_selected_in_block_mode(&self, loc: Point2<i32>) -> bool {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => {
                let (start, end) = util::normalize_points(start, end);
                loc.x >= start.x && loc.x <= end.x && loc.y >= start.y && loc.y <= end.y
            }
            _ => false,
        }
    }

    /// clear the text selection
    pub fn clear_selection(&mut self) {
        self.selection.start = None;
        self.selection.end = None;
    }

    pub fn selected_text(&self) -> Option<String> {
        match self.selection.mode {
            SelectionMode::Linear => self.selected_text_in_linear_mode(),
            SelectionMode::Block => self.selected_text_in_block_mode(),
        }
    }

    pub fn selected_text_in_linear_mode(&self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => Some(
                self.text_buffer
                    .get_text_in_linear_mode(util::cast_point(start), util::cast_point(end)),
            ),
            _ => None,
        }
    }

    pub fn selected_text_in_block_mode(&self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => Some(
                self.text_buffer
                    .get_text_in_block_mode(util::cast_point(start), util::cast_point(end)),
            ),
            _ => None,
        }
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        match self.selection.mode {
            SelectionMode::Linear => self.cut_selected_text_in_linear_mode(),
            SelectionMode::Block => self.cut_selected_text_in_block_mode(),
        }
    }

    pub fn cut_selected_text_in_linear_mode(&mut self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => {
                let start = util::cast_point(start);
                let end = util::cast_point(end);
                let cut_text = self.text_buffer.cut_text_in_linear_mode(start, end);
                if !cut_text.is_empty() {
                    self.record_deleted_text(start, end, &cut_text);
                }
                Some(cut_text)
            }
            _ => None,
        }
    }

    fn record_deleted_text(&mut self, _start: Point2<usize>, _end: Point2<usize>, cut_text: &str) {
        let lines = cut_text.lines();
        for (y, line) in lines.enumerate() {
            for (_x, ch) in line.chars().enumerate() {
                let position = Point2::new(0, y);
                self.recorded.delete(position, Some(ch));
            }
        }
    }

    pub fn cut_selected_text_in_block_mode(&mut self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => Some(
                self.text_buffer
                    .cut_text_in_block_mode(util::cast_point(start), util::cast_point(end)),
            ),
            _ => None,
        }
    }

    pub fn paste_text_in_block_mode(&mut self, text_block: String) {
        self.text_buffer.paste_text_in_block_mode(text_block);
    }

    /// paste the text block overlaying on the text content of the buffer
    /// excluding the whitespace
    pub fn command_merge_text(&mut self, text_block: String) {
        for (line_index, line) in text_block.lines().enumerate() {
            let mut width = 0;
            let y = line_index;
            for ch in line.chars() {
                if ch != crate::BLANK_CH {
                    let x = width;
                    self.command_set_position(x, y);
                    self.command_replace_char(ch);
                }
                width += ch.width().unwrap_or(0);
            }
        }
    }

    pub fn get_position(&self) -> Point2<usize> {
        self.text_buffer.get_position()
    }

    pub fn max_position(&self) -> Point2<usize> {
        self.text_buffer.max_position()
    }

    pub fn get_content(&self) -> String {
        self.text_buffer.to_string()
    }

    pub fn total_lines(&self) -> usize {
        self.text_buffer.total_lines()
    }

    pub fn lines(&self) -> Vec<String> {
        self.text_buffer.lines()
    }

    /// return the number of characters to represent the line number of the last line of the text
    /// buffer
    pub fn numberline_wide(&self) -> usize {
        self.text_buffer.numberline_wide()
    }
}
