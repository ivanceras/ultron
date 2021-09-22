#![allow(unused)]
use super::TextBuffer;
use nalgebra::Point2;

#[derive(Clone, Debug)]
pub enum Action {
    Insert(String),
    InsertForward(String),
    Delete(String),
    Move(i32, i32),
    DeleteForward(String),
    DeleteSelectedForward(Point2<usize>, Point2<usize>, String),
    InsertStringForward(Point2<usize>, Point2<usize>, String),
    BreakLine(usize, usize),
    JoinLine(usize, usize),
}

impl Action {
    pub fn apply(&self, content: &mut TextBuffer) {
        match *self {
            Action::Insert(ref s) => {
                for c in s.chars() {
                    content.command_insert_char(c);
                }
            }
            Action::Delete(ref s) => {
                for _ in s.chars() {
                    content.command_delete_back();
                }
            }
            Action::Move(x, y) => {
                let cursor = content.get_position();
                let cursor_x = cursor.x as i32;
                let cursor_y = cursor.y as i32;
                content.set_position(
                    (cursor_x + x) as usize,
                    (cursor_y + y) as usize,
                );
            }
            Action::DeleteForward(ref s) => {
                for _ in s.chars() {
                    content.command_delete_forward();
                }
            }
            Action::InsertForward(ref s) => {
                for c in s.chars() {
                    content.command_insert_forward_char(c);
                }
            }
            Action::DeleteSelectedForward(start_pos, end_pos, ref _s) => {
                content.move_to(start_pos);
                content.set_selection(start_pos, end_pos);
                content.command_delete_selected_forward();
                content.clear_selection();
            }
            Action::InsertStringForward(start_pos, end_pos, ref s) => {
                content.move_to(start_pos);
                content.command_insert_text(s);
                content.set_selection(start_pos, end_pos);
            }
            Action::BreakLine(x, y) => {
                content.command_break_line(x, y);
            }
            Action::JoinLine(x, y) => {
                content.command_join_line(x, y);
            }
        };
    }

    pub fn invert(&self) -> Action {
        match *self {
            Action::Insert(ref s) => Action::Delete(s.clone()),
            Action::Delete(ref s) => Action::Insert(s.clone()),
            Action::Move(ref x, ref y) => Action::Move(-x, -y),
            Action::InsertForward(ref s) => Action::DeleteForward(s.clone()),
            Action::DeleteForward(ref s) => Action::InsertForward(s.clone()),
            Action::DeleteSelectedForward(start_pos, end_pos, ref s) => {
                Action::InsertStringForward(start_pos, end_pos, s.clone())
            }
            Action::InsertStringForward(start_pos, end_pos, ref s) => {
                Action::DeleteSelectedForward(start_pos, end_pos, s.clone())
            }
            Action::BreakLine(x, y) => Action::JoinLine(x, y),
            Action::JoinLine(x, y) => Action::BreakLine(x, y),
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
            Action::Move(ref mut rel_x, ref mut rel_y) => {
                let (act_rel_x, act_rel_y) = match act {
                    Action::Move(x, y) => (x, y),
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                *rel_x += act_rel_x;
                *rel_y += act_rel_y;
            }
            Action::DeleteSelectedForward(..) => (),
            Action::InsertStringForward(..) => (),
            Action::BreakLine(..) => (),
            Action::JoinLine(..) => (),
        }
    }

    pub fn same_variant(&self, other: &Action) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
