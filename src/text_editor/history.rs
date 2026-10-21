//! Undo history in groups: a run of typed characters is one group; any other edit starts a new one.

use std::collections::VecDeque;

use super::model::{EditKind, Position};

pub const MAX_GROUPS: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub caret: Position,
    pub anchor: Option<Position>,
}

/// `removed` was at `start`, and `inserted` took its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub start: Position,
    pub removed: String,
    pub inserted: String,
}

/// A change with the cursors around it.
pub struct Edit {
    pub change: Change,
    pub kind: EditKind,
    pub before: Cursor,
    pub after: Cursor,
}

#[derive(Debug)]
pub struct EditGroup {
    pub changes: Vec<Change>,
    pub before: Cursor,
    pub after: Cursor,
    kind: EditKind,
}

#[derive(Debug, Default)]
pub struct History {
    undo: VecDeque<EditGroup>,
    redo: Vec<EditGroup>,
    /// whether typing may still join the newest group
    open: bool,
}

impl History {
    pub fn record(&mut self, edit: Edit) {
        self.redo.clear();
        let typing = edit.kind == EditKind::Typing;
        if let Some(group) = self.undo.back_mut().filter(|group| self.open && typing && group.kind == EditKind::Typing) {
            group.changes.push(edit.change);
            group.after = edit.after;
            return;
        }
        self.undo.push_back(EditGroup { changes: vec![edit.change], before: edit.before, after: edit.after, kind: edit.kind });
        self.trim();
        self.open = typing;
    }

    /// Ends the open typing group, as a caret move does.
    pub fn seal(&mut self) {
        self.open = false;
    }

    pub fn pop_undo(&mut self) -> Option<EditGroup> {
        self.open = false;
        self.undo.pop_back()
    }

    pub fn push_redo(&mut self, group: EditGroup) {
        self.redo.push(group);
    }

    pub fn pop_redo(&mut self) -> Option<EditGroup> {
        self.open = false;
        self.redo.pop()
    }

    /// A redone group goes back on the undo stack without clearing the rest of the redo stack.
    pub fn push_undone(&mut self, group: EditGroup) {
        self.undo.push_back(group);
        self.trim();
    }

    fn trim(&mut self) {
        while self.undo.len() > MAX_GROUPS {
            self.undo.pop_front();
        }
    }
}
