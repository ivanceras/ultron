use nalgebra::Point2;
use std::iter::FromIterator;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const BLANK_CH: char = ' ';

/// A text buffer where characters are manipulated visually with
/// consideration on the unicode width of characters.
/// Characters can span more than 1 cell, therefore
/// visually manipulating text in a 2-dimensional way should consider using the unicode width.
#[derive(Clone)]
pub struct TextBuffer {
    chars: Vec<Vec<Ch>>,
    cursor: Point2<usize>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            chars: vec![],
            cursor: Point2::new(0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ch {
    /// the char
    pub ch: char,
    /// the unicode width of the character
    pub width: usize,
}

impl Ch {
    pub fn new(ch: char) -> Self {
        Self {
            width: ch.width().unwrap_or(0),
            ch,
        }
    }
}

impl TextBuffer {
    pub fn new_from_str(content: &str) -> Self {
        Self {
            chars: content
                .lines()
                .map(|line| line.chars().map(Ch::new).collect())
                .collect(),
            cursor: Point2::new(0, 0),
        }
    }

    pub fn from_ch(chars: &[&[Ch]]) -> Self {
        Self {
            chars: chars.iter().map(|inner| inner.to_vec()).collect(),
            cursor: Point2::new(0, 0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn chars(&self) -> &[Vec<Ch>] {
        &self.chars
    }

    /// return the total number of characters
    /// excluding new lines
    pub fn total_chars(&self) -> usize {
        self.chars.iter().map(|line| line.len()).sum()
    }

    pub fn split_line_at_point(&self, loc: Point2<usize>) -> (String, String) {
        let loc = self.point_to_index(loc);
        let first = &self.chars[loc.y][0..loc.x];
        let first_str = String::from_iter(first.iter().map(|ch| ch.ch));
        let second = &self.chars[loc.y][loc.x..];
        let second_str = String::from_iter(second.iter().map(|ch| ch.ch));
        (first_str, second_str)
    }

    pub fn split_line_at_2_points(
        &self,
        loc: Point2<usize>,
        loc2: Point2<usize>,
    ) -> (String, String, String) {
        let loc = self.point_to_index(loc);
        let loc2 = self.point_to_index(loc2);

        let first = &self.chars[loc.y][0..loc.x];
        let first_str = String::from_iter(first.iter().map(|ch| ch.ch));
        let second = &self.chars[loc.y][loc.x..loc2.x];
        let second_str = String::from_iter(second.iter().map(|ch| ch.ch));
        let third = &self.chars[loc.y][loc2.x..];
        let third_str = String::from_iter(third.iter().map(|ch| ch.ch));

        (first_str, second_str, third_str)
    }

    /// Remove the text within the start and end position then return the deleted text
    pub fn cut_text_in_linear_mode(&mut self, start: Point2<usize>, end: Point2<usize>) -> String {
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        let is_one_line = start.y == end.y;
        if is_one_line {
            let selection: Vec<Ch> = self.chars[start.y].drain(start.x..=end.x).collect();
            String::from_iter(selection.iter().map(|ch| ch.ch))
        } else {
            let end_text: Vec<Ch> = self.chars[end.y].drain(0..=end.x).collect();

            let mid_text_range = start.y + 1..end.y;
            let mid_text: Option<Vec<Vec<Ch>>> = if !mid_text_range.is_empty() {
                Some(self.chars.drain(mid_text_range).collect())
            } else {
                None
            };
            let start_text: Vec<Ch> = self.chars[start.y].drain(start.x..).collect();

            let start_text_str: String = String::from_iter(start_text.iter().map(|ch| ch.ch));

            let end_text_str: String = String::from_iter(end_text.iter().map(|ch| ch.ch));

            if let Some(mid_text) = mid_text {
                let mid_text_str: String = mid_text
                    .iter()
                    .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
                    .collect::<Vec<_>>()
                    .join("\n");

                [start_text_str, mid_text_str, end_text_str].join("\n")
            } else {
                [start_text_str, end_text_str].join("\n")
            }
        }
    }

    /// get the text in between start and end if selected in linear mode
    pub fn get_text_in_linear_mode(&self, start: Point2<usize>, end: Point2<usize>) -> String {
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        let is_one_line = start.y == end.y;
        if is_one_line {
            let selection: &[Ch] = &self.chars[start.y][start.x..=end.x];
            String::from_iter(selection.iter().map(|ch| ch.ch))
        } else {
            let start_text: &[Ch] = &self.chars[start.y][start.x..];

            let mid_text_range = start.y + 1..end.y;
            let mid_text: Option<&[Vec<Ch>]> = if !mid_text_range.is_empty() {
                Some(&self.chars[mid_text_range])
            } else {
                None
            };

            let end_text: &[Ch] = &self.chars[end.y][0..=end.x];
            let start_text_str: String = String::from_iter(start_text.iter().map(|ch| ch.ch));

            let end_text_str: String = String::from_iter(end_text.iter().map(|ch| ch.ch));

            if let Some(mid_text) = mid_text {
                let mid_text_str: String = mid_text
                    .iter()
                    .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
                    .collect::<Vec<_>>()
                    .join("\n");

                [start_text_str, mid_text_str, end_text_str].join("\n")
            } else {
                [start_text_str, end_text_str].join("\n")
            }
        }
    }

    /// get the text in between start and end if selected in block mode
    pub fn get_text_in_block_mode(&self, start: Point2<usize>, end: Point2<usize>) -> String {
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        log::info!("here: {} {}", start, end);
        (start.y..=end.y)
            .map(|y| {
                if let Some(chars) = &self.chars.get(y) {
                    let text =
                        (start.x..=end.x).map(|x| chars.get(x).map(|ch| ch.ch).unwrap_or(BLANK_CH));
                    String::from_iter(text)
                } else {
                    String::new()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn cut_text_in_block_mode(&mut self, start: Point2<usize>, end: Point2<usize>) -> String {
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        (start.y..=end.y)
            .map(|y| {
                let text = self.chars[y].drain(start.x..=end.x);
                String::from_iter(text.map(|ch| ch.ch))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// paste the text block in the cursor location
    pub fn paste_text_in_block_mode(&mut self, text_block: String) {
        for (line_index, line) in text_block.lines().enumerate() {
            let mut width = 0;
            let y = self.cursor.y + line_index;
            for ch in line.chars() {
                let x = self.cursor.x + width;
                self.replace_char(Point2::new(x, y), ch);
                width += ch.width().unwrap_or(0);
            }
        }
    }
}

/// text manipulation
/// This are purely manipulating text into the text buffer.
/// The cursor shouldn't be move here, since it is done by the commands functions
impl TextBuffer {
    /// the total number of lines of this text canvas
    pub fn total_lines(&self) -> usize {
        self.chars.len()
    }

    /// return the number of characters to represent the line number of the last line of
    /// this text buffer
    pub fn numberline_wide(&self) -> usize {
        self.total_lines().to_string().len()
    }

    pub fn lines(&self) -> Vec<String> {
        self.chars
            .iter()
            .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
            .collect()
    }

    /// return the first non blank line
    pub fn first_non_blank_line(&self) -> Option<usize> {
        self.chars
            .iter()
            .enumerate()
            .find_map(|(line_index, line)| {
                if line.iter().any(|ch| ch.ch != BLANK_CH) {
                    Some(line_index)
                } else {
                    None
                }
            })
    }

    /// return the last non blank line
    pub fn last_non_blank_line(&self) -> Option<usize> {
        self.chars
            .iter()
            .enumerate()
            .rev()
            .find_map(|(line_index, line)| {
                if line.iter().any(|ch| ch.ch != BLANK_CH) {
                    Some(line_index)
                } else {
                    None
                }
            })
    }

    /// the width of the line at line `n`
    pub fn line_width(&self, n: usize) -> usize {
        self.chars
            .get(n)
            .map(|line| line.iter().map(|ch| ch.width).sum())
            .unwrap_or(0)
    }

    /// get the length of the widest line
    pub fn max_column_width(&self) -> usize {
        self.chars
            .iter()
            .map(|line| line.iter().map(|ch| ch.width).sum())
            .max()
            .unwrap_or(0)
    }

    /// return rectangular position starting from 0,0 to contain all
    /// the text
    pub fn max_position(&self) -> Point2<usize> {
        let last_line = self.total_lines().saturating_sub(1);
        let max_column = self.max_column_width();
        Point2::new(max_column, last_line)
    }

    pub fn last_char_position(&self) -> Point2<usize> {
        let last_line = self.total_lines().saturating_sub(1);
        let bottom_last_x = self.line_width(last_line).saturating_sub(1);
        Point2::new(bottom_last_x, last_line)
    }

    /// break at line y and put the characters after x on the next line
    pub fn break_line(&mut self, loc: Point2<usize>) {
        self.ensure_before_cell_exist(loc);
        if let Some(break_point) = self.column_index(loc) {
            let break2 = self.cut_to_end_of_line(loc.y, break_point);
            self.insert_line(loc.y + 1, break2);
        } else {
            self.insert_line(loc.y + 1, vec![]);
        }
    }

    /// insert a line at this location
    pub fn insert_line(&mut self, loc_y: usize, text: Vec<Ch>) {
        self.chars.insert(loc_y, text);
    }

    pub fn remove_line(&mut self, loc_y: usize) -> Vec<Ch> {
        self.chars.remove(loc_y)
    }

    pub fn cut_to_end_of_line(&mut self, loc_y: usize, char_index: usize) -> Vec<Ch> {
        self.chars[loc_y].drain(char_index..).collect()
    }

    pub fn append_to_line(&mut self, loc_y: usize, text: Vec<Ch>) {
        self.chars[loc_y].extend(text);
    }

    pub fn join_line(&mut self, loc: Point2<usize>) {
        let next_line_index = loc.y.saturating_add(1);
        let next_line = self.remove_line(next_line_index);
        self.append_to_line(loc.y, next_line);
    }

    /// ensure line at index y exist
    pub fn ensure_line_exist(&mut self, y: usize) {
        let total_lines = self.total_lines();
        let diff = y.saturating_add(1).saturating_sub(total_lines);
        for _ in 0..diff {
            self.chars.push(vec![]);
        }
    }

    pub fn ensure_before_line_exist(&mut self, y: usize) {
        if y > 0 {
            self.ensure_line_exist(y.saturating_sub(1));
        }
    }

    /// ensure line in index y exist and the cell at index x
    pub fn ensure_cell_exist(&mut self, loc: Point2<usize>) {
        self.ensure_line_exist(loc.y);
        let line_width = self.line_width(loc.y);
        let diff = loc.x.saturating_add(1).saturating_sub(line_width);
        for _ in 0..diff {
            self.chars[loc.y].push(Ch::new(BLANK_CH));
        }
    }

    pub fn ensure_before_cell_exist(&mut self, loc: Point2<usize>) {
        self.ensure_line_exist(loc.y);
        if loc.x > 0 {
            self.ensure_cell_exist(Point2::new(loc.x.saturating_sub(1), loc.y));
        }
    }

    /// calculate the column index base on position of x and y
    /// and considering the unicode width of the characters
    fn column_index(&self, loc: Point2<usize>) -> Option<usize> {
        if let Some(line) = self.chars.get(loc.y) {
            let mut width_sum = 0;
            for (i, ch) in line.iter().enumerate() {
                if width_sum == loc.x {
                    return Some(i);
                }
                width_sum += ch.width;
            }
            None
        } else {
            None
        }
    }

    /// translate this point into the correct index position
    /// considering the character widths
    fn point_to_index(&self, point: Point2<usize>) -> Point2<usize> {
        let column_x = self.column_index(point).unwrap_or(point.x);
        Point2::new(column_x, point.y)
    }

    /// insert a character at this x and y and move cells after it to the right
    pub fn insert_char(&mut self, loc: Point2<usize>, ch: char) {
        self.ensure_before_cell_exist(loc);
        let new_ch = Ch::new(ch);
        if let Some(column_index) = self.column_index(loc) {
            let insert_index = column_index;
            self.chars[loc.y].insert(insert_index, new_ch);
        } else {
            self.chars[loc.y].push(new_ch);
        }
    }

    /// insert a text, must not contain a \n
    fn insert_line_text(&mut self, loc: Point2<usize>, text: &str) {
        let mut width_inc = 0;
        for ch in text.chars() {
            let new_ch = Ch::new(ch);
            self.insert_char(Point2::new(loc.x + width_inc, loc.y), new_ch.ch);
            width_inc += new_ch.width;
        }
    }

    pub fn insert_text(&mut self, loc: Point2<usize>, text: &str) {
        let mut start = loc.x;
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                self.insert_line(loc.y + 1, vec![]);
            }
            self.insert_line_text(Point2::new(start, loc.y + i), line);
            start = 0;
        }
    }

    /// replace the character at this location
    pub fn replace_char(&mut self, loc: Point2<usize>, ch: char) -> Option<char> {
        self.ensure_cell_exist(loc);
        let column_index = self.column_index(loc).expect("must have a column index");
        let ex_ch = self.chars[loc.y].remove(column_index);
        self.chars[loc.y].insert(column_index, Ch::new(ch));
        Some(ex_ch.ch)
    }

    /// get the character at this cursor position
    pub fn get_char(&self, loc: Point2<usize>) -> Option<char> {
        if let Some(line) = self.chars.get(loc.y) {
            let column_index = self.column_index(loc);
            column_index.and_then(|col| line.get(col).map(|ch| ch.ch))
        } else {
            None
        }
    }

    /// delete character at this position
    pub fn delete_char(&mut self, loc: Point2<usize>) -> Option<char> {
        if let Some(column_index) = self.column_index(loc) {
            let ex_ch = self.chars[loc.y].remove(column_index);
            Some(ex_ch.ch)
        } else {
            None
        }
    }

    /// return the position of the cursor
    pub fn get_position(&self) -> Point2<usize> {
        self.cursor
    }
}

/// Command implementation here
///
/// functions that are preceeded with command also moves the
/// cursor and highlight the texts
///
/// Note: methods that are preceeded with `command` such as `command_insert_char` are high level methods
/// which has consequences in the text buffer such as moving the cursor.
/// While there corresponding more primitive counter parts such as `insert_char` are low level
/// commands, which doesn't move the cursor location
impl TextBuffer {
    pub fn command_insert_char(&mut self, ch: char) {
        self.insert_char(self.cursor, ch);
        let width = ch.width().expect("must have a unicode width");
        self.move_x(width);
    }

    pub fn command_replace_char(&mut self, ch: char) -> Option<char> {
        self.replace_char(self.cursor, ch)
    }

    pub fn command_insert_text(&mut self, text: &str) {
        let width = text.width();
        self.insert_text(self.cursor, text);
        self.move_x(width);
    }

    pub fn move_left(&mut self) {
        self.cursor.x = self.cursor.x.saturating_sub(1);
    }

    pub fn move_left_start(&mut self) {
        self.cursor.x = 0;
    }

    pub fn move_right(&mut self) {
        self.cursor.x = self.cursor.x.saturating_add(1);
    }

    fn line_max_column(&self, line: usize) -> usize {
        self.chars.get(line).map(|line| line.len()).unwrap_or(0)
    }

    fn current_line_max_column(&self) -> usize {
        self.line_max_column(self.cursor.y)
    }

    pub fn move_right_clamped(&mut self) {
        if self.cursor.x < self.current_line_max_column() {
            self.move_right();
        }
    }

    pub fn move_right_end(&mut self) {
        self.cursor.x = self.current_line_max_column();
    }

    pub fn move_x(&mut self, x: usize) {
        self.cursor.x = self.cursor.x.saturating_add(x);
    }

    pub fn move_y(&mut self, y: usize) {
        self.cursor.y = self.cursor.y.saturating_add(y);
    }

    pub fn move_up(&mut self) {
        self.cursor.y = self.cursor.y.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        self.cursor.y = self.cursor.y.saturating_add(1);
    }

    pub fn move_up_clamped(&mut self) {
        let target_line = self.cursor.y.saturating_sub(1);
        let target_line_max_column = self.line_max_column(target_line);
        if target_line < self.total_lines() {
            if self.cursor.x > target_line_max_column {
                self.cursor.x = target_line_max_column;
            }
            self.move_up()
        }
    }

    pub fn clamp_position(&self, loc: Point2<usize>) -> Point2<usize> {
        let line = loc.y;
        let line_max_column = self.line_max_column(line);
        let loc_x = if loc.x > line_max_column {
            line_max_column
        } else {
            loc.x
        };
        Point2::new(loc_x, loc.y)
    }

    pub fn move_down_clamped(&mut self) {
        let target_line = self.cursor.y.saturating_add(1);
        let target_line_max_column = self.line_max_column(target_line);
        if target_line < self.total_lines() {
            if self.cursor.x > target_line_max_column {
                self.cursor.x = target_line_max_column;
            }
            self.move_down()
        }
    }

    pub fn set_position(&mut self, pos: Point2<usize>) {
        self.cursor = pos;
    }

    /// set the position to the max_column of the line if it is out of
    /// bounds
    pub fn set_position_clamped(&mut self, pos: Point2<usize>) {
        let total_lines = self.total_lines();
        let mut y = pos.y;
        if y > total_lines {
            y = total_lines.saturating_sub(1);
        }
        let line_width = self.line_width(y);
        let mut x = pos.x;
        if x > line_width {
            x = line_width.saturating_sub(1);
        }
        self.set_position(Point2::new(x, y))
    }

    pub fn command_break_line(&mut self, loc: Point2<usize>) {
        self.break_line(loc);
        self.move_left_start();
        self.move_down();
    }

    pub fn command_join_line(&mut self, loc: Point2<usize>) {
        self.join_line(loc);
        self.set_position(loc);
    }

    pub fn command_delete_back(&mut self) -> Option<char> {
        if self.cursor.x > 0 {
            let c = self.delete_char(Point2::new(self.cursor.x.saturating_sub(1), self.cursor.y));
            self.move_left();
            c
        } else {
            None
        }
    }

    pub fn command_delete_forward(&mut self) -> Option<char> {
        self.delete_char(self.cursor)
    }

    /// move the cursor to position
    pub fn move_to(&mut self, pos: Point2<usize>) {
        self.set_position(pos);
    }

    /// clear the contents of this text buffer
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
