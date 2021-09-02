use super::action::Action;
use super::TextBuffer;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = usize::MAX;
const UNDO_SIZE: usize = usize::MAX;

#[derive(Debug)]
pub struct Recorded {
    history: VecDeque<Action>,
    undone: VecDeque<Action>,
}

impl Recorded {
    pub fn new() -> Self {
        Recorded {
            history: VecDeque::new(),
            undone: VecDeque::new(),
        }
    }
    fn record(&mut self, act: Action) {
        self.undone.clear(); // we are branching to a new sequence of events
        if let Some(a) = self.history.front_mut() {
            if a.same_variant(&act) {
                // join similar actions together
                a.join(act);
                return;
            }
        }
        self.history.push_front(act);
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    pub(crate) fn undo(&mut self, text_buffer: &mut TextBuffer) {
        let to_undo = match self.history.pop_front() {
            None => return,
            Some(a) => a,
        };
        self.undone.push_front(to_undo.clone());
        while self.undone.len() > UNDO_SIZE {
            self.undone.pop_back();
        }
        to_undo.invert().apply(text_buffer);
    }

    pub(crate) fn redo(&mut self, text_buffer: &mut TextBuffer) {
        let to_redo = match self.undone.pop_front() {
            None => return,
            Some(a) => a,
        };
        to_redo.apply(text_buffer);
        self.history.push_front(to_redo);
    }

    #[allow(unused)]
    fn history_len(&self) -> usize {
        self.history.len()
    }

    pub(crate) fn move_cursor(&mut self, dist: isize) {
        self.record(Action::Move(dist));
    }

    pub(crate) fn insert(&mut self, c: char) {
        self.record(Action::Insert(c.to_string()));
    }

    pub(crate) fn delete(&mut self, c: Option<char>) -> Option<char> {
        if let Some(c) = c {
            self.record(Action::Delete(c.to_string()));
        }
        c
    }

    pub(crate) fn delete_selected_forward(
        &mut self,
        start_pos: usize,
        end_pos: usize,
        s: &str,
    ) {
        if !s.is_empty() {
            self.record(Action::DeleteSelectedForward(
                start_pos,
                end_pos,
                s.to_string(),
            ));
        }
    }

    pub(crate) fn insert_string(
        &mut self,
        start_pos: usize,
        end_pos: usize,
        s: &str,
    ) {
        if !s.is_empty() {
            self.record(Action::InsertStringForward(
                start_pos,
                end_pos,
                s.to_string(),
            ))
        }
    }

    pub(crate) fn delete_forward(&mut self, c: Option<char>) -> Option<char> {
        if let Some(c) = c {
            self.record(Action::DeleteForward(c.to_string()))
        }
        c
    }
}
