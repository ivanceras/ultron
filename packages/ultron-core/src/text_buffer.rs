use nalgebra::Point2;
use std::iter::FromIterator;
use unicode_width::UnicodeWidthChar;

/// A text buffer where characters are manipulated visually with
/// consideration on the unicode width of characters.
/// Characters can span more than 1 cell, therefore
/// visually manipulating text in a 2-dimensional way should consider using the unicode width.
#[derive(Clone)]
pub struct TextBuffer {
    chars: Vec<Vec<Ch>>,
    cursor: Point2<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Ch {
    pub ch: char,
    pub width: usize,
}

impl Ch {
    fn new(ch: char) -> Self {
        Self {
            width: ch.width().unwrap_or(0),
            ch,
        }
    }
}

impl TextBuffer {
    pub fn from_str(content: &str) -> Self {
        Self {
            chars: content
                .lines()
                .map(|line| line.chars().map(Ch::new).collect())
                .collect(),
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

    /// Remove the text within the start and end position then return the deleted text
    pub fn cut_text(
        &mut self,
        start: Point2<usize>,
        end: Point2<usize>,
    ) -> String {
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        let is_one_line = start.y == end.y;
        if is_one_line {
            let selection: Vec<Ch> =
                self.chars[start.y].drain(start.x..=end.x).collect();
            String::from_iter(selection.iter().map(|ch| ch.ch))
        } else {
            let end_text: Vec<Ch> =
                self.chars[end.y].drain(0..=end.x).collect();

            let mid_text_range = start.y + 1..end.y;
            let mid_text: Option<Vec<Vec<Ch>>> = if !mid_text_range.is_empty() {
                Some(self.chars.drain(mid_text_range).collect())
            } else {
                None
            };
            let start_text: Vec<Ch> =
                self.chars[start.y].drain(start.x..).collect();

            let start_text_str: String =
                String::from_iter(start_text.iter().map(|ch| ch.ch));

            let end_text_str: String =
                String::from_iter(end_text.iter().map(|ch| ch.ch));

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

    pub fn get_text(&self, start: Point2<usize>, end: Point2<usize>) -> String {
        println!("original : {}, {}", start, end);
        let start = self.point_to_index(start);
        let end = self.point_to_index(end);
        println!("corrected : {}, {}", start, end);
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
            let start_text_str: String =
                String::from_iter(start_text.iter().map(|ch| ch.ch));

            let end_text_str: String =
                String::from_iter(end_text.iter().map(|ch| ch.ch));

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
}

/// text manipulation
/// This are purely manipulating text into the text buffer.
/// The cursor shouldn't be move here, since it is done by the commands functions
impl TextBuffer {
    /// the total number of lines of this text canvas
    pub fn total_lines(&self) -> usize {
        self.chars.len()
    }

    pub fn lines(&self) -> Vec<String> {
        self.chars
            .iter()
            .map(|line| String::from_iter(line.iter().map(|ch| ch.ch)))
            .collect()
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

    /// break at line y and put the characters after x on the next line
    pub fn break_line(&mut self, x: usize, y: usize) {
        self.ensure_before_cell_exist(x, y);
        let line = &self.chars[y];
        if let Some(break_point) = self.column_index(x, y) {
            let (break1, break2): (Vec<_>, Vec<_>) = line
                .iter()
                .enumerate()
                .partition(|(i, _ch)| *i < break_point);

            let break1: Vec<Ch> =
                break1.into_iter().map(|(_, ch)| *ch).collect();
            let break2: Vec<Ch> =
                break2.into_iter().map(|(_, ch)| *ch).collect();
            self.chars.remove(y);
            self.chars.insert(y, break2);
            self.chars.insert(y, break1);
        } else {
            self.chars.insert(y + 1, vec![]);
        }
    }

    pub fn join_line(&mut self, _x: usize, y: usize) {
        let next_line_index = y.saturating_add(1);
        let mut next_line = self.chars.remove(next_line_index);
        self.chars[y].append(&mut next_line);
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
    pub fn ensure_cell_exist(&mut self, x: usize, y: usize) {
        self.ensure_line_exist(y);
        let line_width = self.line_width(y);
        let diff = x.saturating_add(1).saturating_sub(line_width);
        for _ in 0..diff {
            self.chars[y].push(Ch::new(' '));
        }
    }

    pub fn ensure_before_cell_exist(&mut self, x: usize, y: usize) {
        self.ensure_line_exist(y);
        if x > 0 {
            self.ensure_cell_exist(x.saturating_sub(1), y);
        }
    }

    /// calculate the column index base on position of x and y
    /// and considering the unicode width of the characters
    fn column_index(&self, x: usize, y: usize) -> Option<usize> {
        if let Some(line) = self.chars.get(y) {
            let mut width_sum = 0;
            for (i, ch) in line.iter().enumerate() {
                if width_sum == x {
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
        let x = point.x;
        let y = point.y;
        let column_x = self.column_index(x, y).unwrap_or(x);
        Point2::new(column_x, y)
    }

    /// insert a character at this x and y and move cells after it to the right
    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.ensure_before_cell_exist(x, y);
        let new_ch = Ch::new(ch);
        if let Some(column_index) = self.column_index(x, y) {
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

    pub fn insert_text(&mut self, x: usize, y: usize, text: &str) {
        let mut start = x;
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                self.chars.insert(y + 1, vec![]);
            }
            self.insert_line_text(start, y + i, line);
            start = 0;
        }
    }

    /// replace the character at this location
    pub fn replace_char(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
    ) -> Option<char> {
        self.ensure_cell_exist(x, y);
        let column_index =
            self.column_index(x, y).expect("must have a column index");
        let ex_ch = self.chars[y].remove(column_index);
        self.chars[y].insert(column_index, Ch::new(ch));
        Some(ex_ch.ch)
    }

    /// get the character at this cursor position
    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        if let Some(line) = self.chars.get(y) {
            let column_index = self.column_index(x, y);
            column_index.and_then(|col| line.get(col).map(|ch| ch.ch))
        } else {
            None
        }
    }

    /// delete character at this position
    pub fn delete_char(&mut self, x: usize, y: usize) -> Option<char> {
        if let Some(column_index) = self.column_index(x, y) {
            let ex_ch = self.chars[y].remove(column_index);
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
        self.insert_char(self.cursor.x, self.cursor.y, ch);
        let width = ch.width().expect("must have a unicode width");
        self.move_x(width);
    }

    pub fn command_replace_char(&mut self, ch: char) -> Option<char> {
        self.replace_char(self.cursor.x, self.cursor.y, ch)
    }

    pub fn command_insert_text(&mut self, text: &str) {
        self.insert_text(self.cursor.x, self.cursor.y, text);
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
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.cursor.x = x;
        self.cursor.y = y;
    }

    /// set the position to the max_column of the line if it is out of
    /// bounds
    pub fn set_position_clamped(&mut self, mut x: usize, mut y: usize) {
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

    pub fn command_break_line(&mut self, x: usize, y: usize) {
        self.break_line(x, y);
        self.move_left_start();
        self.move_down();
    }

    pub fn command_join_line(&mut self, x: usize, y: usize) {
        self.join_line(x, y);
        self.set_position(x, y);
    }

    pub fn command_delete_back(&mut self) -> Option<char> {
        if self.cursor.x > 0 {
            let c = self
                .delete_char(self.cursor.x.saturating_sub(1), self.cursor.y);
            self.move_left();
            c
        } else {
            None
        }
    }
    pub fn command_delete_forward(&mut self) -> Option<char> {
        self.delete_char(self.cursor.x, self.cursor.y)
    }

    /// move the cursor to position
    pub fn move_to(&mut self, pos: Point2<usize>) {
        self.set_position(pos.x, pos.y);
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
