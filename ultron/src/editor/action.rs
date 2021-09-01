use super::TextBuffer;

#[derive(Clone, Debug)]
pub enum Action {
    Insert(String),
    InsertForward(String),
    Delete(String),
    Move(isize),
    DeleteForward(String),
    DeleteSelectedForward(usize, usize, String),
    InsertStringForward(usize, usize, String),
}

impl Action {
    pub fn apply(&self, content: &mut TextBuffer) {
        match *self {
            Action::Insert(ref s) => {}
            Action::Delete(ref s) => {}
            Action::Move(rel) => {}
            Action::DeleteForward(ref s) => {}
            Action::InsertForward(ref s) => {}
            Action::DeleteSelectedForward(start_pos, end_pos, ref _s) => {}
            Action::InsertStringForward(start_pos, end_pos, ref s) => {}
        };
    }

    pub fn invert(&self) -> Action {
        match *self {
            Action::Insert(ref s) => Action::Delete(s.clone()),
            Action::Delete(ref s) => Action::Insert(s.clone()),
            Action::Move(ref rel) => Action::Move(-rel),
            Action::InsertForward(ref s) => Action::DeleteForward(s.clone()),
            Action::DeleteForward(ref s) => Action::InsertForward(s.clone()),
            Action::DeleteSelectedForward(start_pos, end_pos, ref s) => {
                Action::InsertStringForward(start_pos, end_pos, s.clone())
            }
            Action::InsertStringForward(start_pos, end_pos, ref s) => {
                Action::DeleteSelectedForward(start_pos, end_pos, s.clone())
            }
        }
    }

    pub fn join(&mut self, act: Action) {
        assert!(self.same_variant(&act));
        match *self {
            Action::Insert(ref mut s) => {
                let act_string = match act {
                    Action::Insert(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(&act_string);
            }
            Action::InsertForward(ref mut s) => {
                let act_string = match act {
                    Action::InsertForward(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(&act_string);
            }
            Action::Delete(ref mut s) => {
                let mut act_string = match act {
                    Action::Delete(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                act_string.push_str(s);
                *s = act_string;
            }
            Action::DeleteForward(ref mut s) => {
                let mut act_string = match act {
                    Action::DeleteForward(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                act_string.push_str(s);
                *s = act_string;
            }
            Action::Move(ref mut rel) => {
                let act_rel = match act {
                    Action::Move(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                *rel += act_rel;
            }
            Action::DeleteSelectedForward(..) => (),
            Action::InsertStringForward(..) => (),
        }
    }

    pub fn same_variant(&self, other: &Action) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
