#![allow(unused)]
use super::action::Action;
use super::TextBuffer;
use nalgebra::Point2;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = usize::MAX;
const UNDO_SIZE: usize = usize::MAX;

#[derive(Debug)]
pub struct Recorded {
    /// a flag whether to continue to last action list or
    /// create a new one
    use_new: bool,
    history: VecDeque<ActionList>,
    undone: VecDeque<ActionList>,
}

#[derive(Debug, Clone)]
pub struct ActionList {
    actions: Vec<Action>,
}

impl From<Vec<Action>> for ActionList {
    fn from(actions: Vec<Action>) -> Self {
        Self { actions }
    }
}

impl ActionList {
    fn same_variant_to_last(&self, action: &Action) -> bool {
        if let Some(last) = self.actions.last() {
            last.same_variant(action)
        } else {
            false
        }
    }
}

impl Recorded {
    pub fn new() -> Self {
        Recorded {
            use_new: false,
            history: VecDeque::new(),
            undone: VecDeque::new(),
        }
    }
    /// Close the current history, such that undo and redo will
    /// be split in this bump point
    pub fn bump_history(&mut self) {
        self.use_new = true;
        log::trace!("bumping history..");
    }

    fn record(&mut self, act: Action) {
        log::trace!("recording: {:?}", act);
        self.undone.clear(); // we are branching to a new sequence of events
        if !self.use_new {
            if let Some(a) = self.history.front_mut() {
                if a.same_variant_to_last(&act) {
                    // join similar actions together
                    a.actions.push(act);
                    return;
                }
            }
        }
        self.history.push_front(ActionList::from(vec![act]));
        self.use_new = false;
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    pub(crate) fn undo(&mut self, text_buffer: &mut TextBuffer) {
        log::trace!("undoing...");
        let to_undo = match self.history.pop_front() {
            None => return,
            Some(a) => {
                log::trace!("undoing: {:?}", a);
                a
            }
        };
        self.undone.push_front(to_undo.clone());
        while self.undone.len() > UNDO_SIZE {
            self.undone.pop_back();
        }
        to_undo.actions.iter().rev().for_each(|tu| {
            let inverted = tu.invert();
            log::trace!("inverted: {:?}", inverted);
            inverted.apply(text_buffer);
        });
    }

    pub(crate) fn redo(&mut self, text_buffer: &mut TextBuffer) {
        let to_redo = match self.undone.pop_front() {
            None => return,
            Some(a) => a,
        };
        to_redo.actions.iter().for_each(|tr| tr.apply(text_buffer));
        self.history.push_front(to_redo);
    }

    #[allow(unused)]
    fn history_len(&self) -> usize {
        self.history.len()
    }

    pub(crate) fn move_cursor(&mut self, cursor: Point2<usize>) {
        self.record(Action::Move(cursor));
    }

    pub(crate) fn insert_char(&mut self, cursor: Point2<usize>, ch: char) {
        self.record(Action::Insert(cursor, ch));
    }

    pub(crate) fn replace_char(
        &mut self,
        cursor: Point2<usize>,
        old_ch: char,
        ch: char,
    ) {
        self.record(Action::Replace(cursor, old_ch, ch));
    }

    pub(crate) fn delete(
        &mut self,
        cursor: Point2<usize>,
        ch: Option<char>,
    ) -> Option<char> {
        if let Some(ch) = ch {
            self.record(Action::Delete(cursor, ch));
        }
        ch
    }

    pub(crate) fn break_line(&mut self, loc: Point2<usize>) {
        self.record(Action::BreakLine(loc));
    }

    pub(crate) fn join_line(&mut self, loc: Point2<usize>) {
        self.record(Action::JoinLine(loc));
    }
}
