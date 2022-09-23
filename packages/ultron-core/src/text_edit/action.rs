use crate::TextBuffer;
use nalgebra::Point2;

#[derive(Clone, Debug)]
pub enum Action {
    Insert(Point2<usize>, char),
    Delete(Point2<usize>, char),
    /// replace character at (cursor, old_char, new_char)
    Replace(Point2<usize>, char, char),
    BreakLine(Point2<usize>),
    JoinLine(Point2<usize>),
}

impl Action {
    /// return the location of this action
    pub fn location(&self) -> Point2<usize> {
        match self {
            Action::Insert(cursor, _ch) => *cursor,
            Action::Delete(cursor, _ch) => *cursor,
            Action::Replace(cursor, _old_ch, _new_ch) => *cursor,
            Action::BreakLine(cursor) => *cursor,
            Action::JoinLine(cursor) => *cursor,
        }
    }

    pub fn apply(&self, content: &mut TextBuffer) {
        match *self {
            Action::Insert(cursor, ch) => {
                content.insert_char(cursor.x, cursor.y, ch);
            }
            Action::Delete(cursor, _ch) => {
                content.delete_char(cursor.x, cursor.y);
            }
            Action::Replace(cursor, _old_ch, ch) => {
                content.replace_char(cursor.x, cursor.y, ch);
            }
            Action::BreakLine(loc) => {
                content.break_line(loc.x, loc.y);
            }
            Action::JoinLine(loc) => {
                content.join_line(loc.x, loc.y);
            }
        };
    }

    pub fn invert(&self) -> Action {
        match *self {
            Action::Insert(cursor, ch) => Action::Delete(cursor, ch),
            Action::Delete(cursor, ch) => Action::Insert(cursor, ch),
            Action::Replace(cursor, old_ch, ch) => {
                Action::Replace(cursor, ch, old_ch)
            }
            Action::BreakLine(loc) => Action::JoinLine(loc),
            Action::JoinLine(loc) => Action::BreakLine(loc),
        }
    }

    pub fn same_variant(&self, other: &Action) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
