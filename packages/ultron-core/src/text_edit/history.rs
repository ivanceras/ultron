use super::Action;
use crate::TextBuffer;
use nalgebra::Point2;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = 100;
const UNDO_SIZE: usize = 100;

#[derive(Debug)]
pub struct Recorded {
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
            history: VecDeque::new(),
            undone: VecDeque::new(),
        }
    }

    /// This ensures that the succeeding action to be recorded will not be merged in the last
    /// action list
    pub fn bump_history(&mut self) {
        // pushing an empty new action list, will ensure that the next action
        // will in a separate action list from the previous ones
        self.history.push_front(ActionList::from(vec![]));
    }

    /// if the last action list is empty, add it there
    /// if it has content, merge if same variant
    ///
    /// returns true if the action `act` is added to history.
    fn try_merge(&mut self, act: Action) -> Result<(), bool> {
        if let Some(a) = self.history.front_mut() {
            if a.actions.is_empty() {
                a.actions.push(act);
                return Ok(());
            } else if a.same_variant_to_last(&act) {
                a.actions.push(act);
                return Ok(());
            }
        }
        Err(false)
    }

    /// create a new action list
    fn record_new(&mut self, act: Action) {
        self.history.push_front(ActionList::from(vec![act]));
    }

    /// will try to merge the action to the last action list if possible
    /// otherwise record it as a new action list
    fn record(&mut self, act: Action) {
        self.undone.clear(); // we are branching to a new sequence of events
        if self.try_merge(act.clone()).is_err() {
            self.record_new(act);
        }
        self.freeup_history();
    }

    /// free up some history if there are more than allowable length
    fn freeup_history(&mut self) {
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    /// undo the history and return the location of the last occurence
    pub(crate) fn undo(
        &mut self,
        text_buffer: &mut TextBuffer,
    ) -> Option<Point2<usize>> {
        let mut last_location = None;
        if let Some(to_undo) = self.history.pop_front() {
            self.undone.push_front(to_undo.clone());
            self.freeup_undone();

            to_undo.actions.iter().rev().for_each(|tu| {
                let inverted = tu.invert();
                inverted.apply(text_buffer);
                last_location = Some(inverted.location());
            });
        }
        last_location
    }

    fn freeup_undone(&mut self) {
        while self.undone.len() > UNDO_SIZE {
            self.undone.pop_back();
        }
    }

    pub(crate) fn redo(
        &mut self,
        text_buffer: &mut TextBuffer,
    ) -> Option<Point2<usize>> {
        let mut last_location = None;
        if let Some(to_redo) = self.undone.pop_front() {
            to_redo.actions.iter().for_each(|tr| {
                tr.apply(text_buffer);
                last_location = Some(tr.location());
            });
            self.history.push_front(to_redo);
        }
        last_location
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
