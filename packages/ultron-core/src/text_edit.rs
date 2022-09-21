use crate::util;
use crate::TextBuffer;
pub use action::Action;
pub use history::Recorded;
use nalgebra::Point2;

mod action;
mod history;

/// A struct with text_buffer, selection commands, and history recording for undo and redo editing
pub struct TextEdit {
    text_buffer: TextBuffer,
    /// for undo and redo
    recorded: Recorded,
    selection: Selection,
}

#[derive(Default)]
pub struct Selection {
    pub start: Option<Point2<i32>>,
    pub end: Option<Point2<i32>>,
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

    pub fn set_selection(&mut self, start: Point2<i32>, end: Point2<i32>) {
        self.selection.start = Some(start);
        self.selection.end = Some(end);
    }

    pub fn set_selection_start(&mut self, start: Point2<i32>) {
        self.selection.start = Some(start);
    }

    pub fn set_selection_end(&mut self, end: Point2<i32>) {
        self.selection.end = Some(end);
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn command_insert_char(&mut self, ch: char) {
        let cursor = self.text_buffer.get_position();
        log::trace!("inserting char: {}, at cursor: {}", ch, cursor);
        self.text_buffer.command_insert_char(ch);
        self.recorded.insert_char(cursor, ch);
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        self.text_buffer.get_char(x, y)
    }

    pub fn command_smart_replace_insert_char(&mut self, ch: char) {
        let cursor = self.text_buffer.get_position();
        let has_right_char =
            if let Some(ch) = self.get_char(cursor.x + 1, cursor.y) {
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
        let _ch = self.text_buffer.command_delete_forward();
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

    pub fn command_move_right(&mut self) {
        self.text_buffer.move_right();
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
    pub fn command_set_position_clamped(
        &mut self,
        cursor_x: usize,
        cursor_y: usize,
    ) {
        self.text_buffer.set_position_clamped(cursor_x, cursor_y);
    }

    pub fn command_set_selection(
        &mut self,
        start: Point2<i32>,
        end: Point2<i32>,
    ) {
        self.set_selection(start, end)
    }

    pub fn command_select_all(&mut self) {
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

    pub fn undo(&mut self) {
        if let Some(location) = self.recorded.undo(&mut self.text_buffer) {
            self.text_buffer.set_position(location.x, location.y);
        }
    }

    pub fn redo(&mut self) {
        if let Some(location) = self.recorded.redo(&mut self.text_buffer) {
            self.text_buffer.set_position(location.x, location.y);
        }
    }

    /// clear the text selection
    pub fn clear_selection(&mut self) {
        self.selection.start = None;
        self.selection.end = None;
    }

    pub fn selected_text(&self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => Some(
                self.text_buffer
                    .get_text(util::cast_point(start), util::cast_point(end)),
            ),
            _ => None,
        }
    }

    pub fn cut_selected_text(&mut self) -> Option<String> {
        match (self.selection.start, self.selection.end) {
            (Some(start), Some(end)) => Some(
                self.text_buffer
                    .cut_text(util::cast_point(start), util::cast_point(end)),
            ),
            _ => None,
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
}
