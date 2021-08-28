use ropey::iter::Chars;
use ropey::iter::Lines;
use ropey::Rope;
use std::cmp;

#[derive(Clone)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineStart,
    LineEnd,
    PageUp(usize),
    PageDown(usize),
}

#[derive(Debug)]
pub struct TextBuffer {
    pos: usize,
    text: Rope,
    name: String,
    modified: bool,
    pub(crate) selection: Option<(usize, Option<usize>)>,
}

impl TextBuffer {
    /// create an empty text_buffer
    #[allow(unused)]
    pub fn empty() -> Self {
        TextBuffer {
            pos: 0,
            text: Rope::from_str("\n"),
            name: String::new(),
            modified: false,
            selection: None,
        }
    }

    /// create a text buffer from string
    pub fn from_str(content: &str) -> Self {
        TextBuffer {
            pos: 0,
            text: Rope::from_str(content),
            name: String::new(),
            modified: false,
            selection: None,
        }
    }

    /// return the entire content of the text buffer
    pub(crate) fn buffer_content(&self) -> String {
        self.text.to_string()
    }

    /// get the name of this buffer
    #[allow(unused)]
    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    /// set the name of this text_buffer
    #[allow(unused)]
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// indicates if the text_buffer is dirty
    pub(crate) fn was_modified(&self) -> bool {
        self.modified
    }

    pub(crate) fn step(&mut self, mov: Movement) {
        match mov {
            Movement::Up => {
                if self.line() > 0 {
                    let prev_line = self.text.line_to_char(self.line() - 1);
                    let prev_line_size =
                        self.text.lines().nth(self.line() - 1).unwrap().len_chars();
                    self.pos = prev_line + cmp::min(self.col(), prev_line_size - 1);
                }
            }
            Movement::Down => {
                if self.line() < self.line_count() - 1 {
                    let next_line = self.text.line_to_char(self.line() + 1);
                    let next_line_size =
                        self.text.lines().nth(self.line() + 1).unwrap().len_chars();
                    self.pos = next_line + cmp::min(self.col(), next_line_size - 1);
                }
            }
            Movement::PageUp(up) => {
                let target_line = if self.line() < up {
                    0
                } else {
                    self.line() - up
                };
                self.pos = self.text.line_to_char(target_line);
            }
            Movement::PageDown(down) => {
                let target_line = if self.line_count() - self.line() < down {
                    self.line_count() - 1
                } else {
                    self.line() + down
                };
                self.pos = self.text.line_to_char(target_line);
            }
            Movement::Left => {
                if self.pos > 0 {
                    self.pos -= 1;
                }
            }
            Movement::Right => {
                if self.pos < self.text.len_chars() - 1 {
                    self.pos += 1;
                }
            }
            Movement::LineStart => {
                let curr_line = self.text.line_to_char(self.line());

                self.pos = curr_line;
            }
            Movement::LineEnd => {
                let curr_line = self.text.line_to_char(self.line());
                let curr_line_size = self.text.lines().nth(self.line()).unwrap().len_chars();
                self.pos = curr_line + curr_line_size - 1;
            }
        }
    }

    /// insert character at the left of cursor position
    pub(crate) fn insert(&mut self, c: char) {
        self.modified = true;
        self.text.insert(self.pos, &format!("{}", c));
        self.pos += 1;
    }

    /// insert a string to the  cursor position
    pub(crate) fn insert_string(&mut self, s: &str) {
        self.modified = true;
        self.text.insert(self.pos, s);
        self.pos += s.chars().count();
    }

    /// insert character to the right of the cusor
    pub(crate) fn insert_forward(&mut self, c: char) {
        self.modified = true;
        self.text.insert(self.pos, &format!("{}", c));
    }

    /// delete character from the left, (ie: backspace key is pressed)
    pub(crate) fn delete(&mut self) -> Option<char> {
        self.modified = true;
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            let ch = self.text.char(self.pos);
            self.text.remove(self.pos..=self.pos);
            Some(ch)
        }
    }

    /// delete character from the right(ie: delete key is pressed)
    pub(crate) fn delete_forward(&mut self) -> Option<char> {
        self.modified = true;
        if self.pos < self.len() - 1 {
            let ch = self.text.char(self.pos);
            self.text.remove(self.pos..=self.pos);
            Some(ch)
        } else {
            None
        }
    }

    /// move the cursor to this position
    pub(crate) fn move_to(&mut self, pos: usize) {
        let pos = if pos >= self.text.len_chars() {
            self.text.len_chars() - 1
        } else {
            pos
        };
        assert!(pos < self.text.len_chars());
        self.pos = pos;
    }

    /// convert line and column to text buffer position
    pub(crate) fn line_col_to_pos(&self, line: usize, col: usize) -> usize {
        self.text.line_to_char(line) + col
    }

    /// move the cursor to this line and column
    pub(crate) fn move_at(&mut self, line: usize, col: usize) {
        let line = cmp::min(line, self.line_count() - 1);
        let col = cmp::min(col, self.text.lines().nth(line).unwrap().len_chars() - 1);
        self.pos = self.text.line_to_char(line) + col;
    }

    /// get the last col of this line
    pub(crate) fn last_line_col(&self, line: usize) -> usize {
        self.text.lines().nth(line).unwrap().len_chars() - 1
    }

    /// move to the end of a line
    pub(crate) fn move_to_line_end(&mut self, line: usize) {
        let line = cmp::min(line, self.line_count() - 1);
        let col = self.text.lines().nth(line).unwrap().len_chars() - 1;
        self.pos = self.text.line_to_char(line) + col;
    }

    /// current cursor position
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    /// return the line number of pos
    pub(crate) fn line(&self) -> usize {
        self.text.char_to_line(self.pos)
    }

    /// return the column number of position
    pub(crate) fn col(&self) -> usize {
        self.pos - self.text.line_to_char(self.line())
    }

    /// the number of lines in this buffer
    pub(crate) fn line_count(&self) -> usize {
        self.text.len_lines() - 1
    }

    /// the number of characters in this buffer
    pub(crate) fn len(&self) -> usize {
        self.text.len_chars()
    }

    /// all the characters
    #[allow(unused)]
    pub(crate) fn iter(&self) -> Chars {
        self.text.chars()
    }

    /// the lines
    pub(crate) fn lines(&self) -> Lines {
        self.text.lines()
    }

    /// characters of this line
    #[allow(unused)]
    pub(crate) fn iter_line(&self, line: usize) -> Chars {
        self.text.line(line).chars()
    }

    /// get the character index for this line
    #[allow(unused)]
    pub(crate) fn line_index_to_char_index(&self, line: usize) -> usize {
        self.text.line_to_char(line)
    }

    /// get the text content at this range
    pub(crate) fn get_text(&self, start_pos: usize, end_pos: usize) -> String {
        self.text.slice(start_pos..end_pos).to_string()
    }

    /// get the text at this range and delete it from the buffer
    pub(crate) fn cut_text(&mut self, start_pos: usize, end_pos: usize) -> String {
        self.modified = true;
        let text_content = self.get_text(start_pos, end_pos);
        self.text.remove(start_pos..end_pos);
        text_content
    }

    /// get the selection of the text_buffer and normalize the start and end
    pub(crate) fn normalized_selection(&self) -> Option<(usize, Option<usize>)> {
        if let Some((from, Some(to))) = self.selection {
            if from > to {
                Some((to, Some(from)))
            } else {
                Some((from, Some(to)))
            }
        } else {
            None
        }
    }

    pub(crate) fn delete_selected_forward(&mut self) -> Option<String> {
        if let Some((start, Some(end))) = self.normalized_selection() {
            let deleted_text = self.cut_text(start, end);
            self.move_to(start);
            self.selection = None;
            log::info!("deleted: {}", deleted_text);
            Some(deleted_text)
        } else {
            log::error!("non is deleted");
            None
        }
    }
}
