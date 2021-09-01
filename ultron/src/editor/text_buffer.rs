use crate::editor::text_highlight::TextHighlight;



use sauron::prelude::*;





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

pub struct TextBuffer {
    x_pos: usize,
    y_pos: usize,
    pub text_highlight: TextHighlight,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
}

impl TextBuffer {
    /// create a text buffer from string
    pub fn from_str(content: &str) -> Self {
        TextBuffer {
            x_pos: 0,
            y_pos: 0,
            text_highlight: TextHighlight::from_str(content),
            selection_start: None,
            selection_end: None,
        }
    }

    pub(crate) fn step(&mut self, mov: Movement) {
        match mov {
            Movement::Up => {}
            Movement::Down => {}
            Movement::PageUp(_up) => {}
            Movement::PageDown(_down) => {}
            Movement::Left => {}
            Movement::Right => {}
            Movement::LineStart => {}
            Movement::LineEnd => {}
        }
    }

    /// insert character at the left of cursor position
    pub(crate) fn insert(&mut self, _ch: char) {}

    pub fn view<MSG>(&self) -> Node<MSG> {
        self.text_highlight.view()
    }
}
