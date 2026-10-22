//! A file a panel shows as it is on disk: what IVE knows of it, and taking what re-reading it found.

use std::sync::Arc;

use codeview::document::{self, Reread, Stamp};
use codeview::panel::{Cast, Content, Panel};
use gpui::{AnyElement, App, Div, Entity, SharedString, div, prelude::*, rgb};

use crate::document::Document;
use crate::syntax::Language;
use crate::theme::Theme;

/// What IVE knows of a file after its last read or re-read.
#[derive(Debug, PartialEq)]
pub(crate) enum Disk {
    /// read, with its size and modification time then
    Read(Stamp),
    /// read, but its stamp could not be taken, so the next change reads it again
    Unstamped,
    Gone,
    /// the re-read failed with this note, so the next change reads it again
    Failed(SharedString),
}

impl Disk {
    /// What `reread` makes of the file's state; `None` when it found nothing new.
    pub(crate) fn after(reread: &Reread) -> Option<Self> {
        match reread {
            Reread::Unchanged => None,
            Reread::Changed(_, stamp) => Some(stamp.map_or(Self::Unstamped, Self::Read)),
            Reread::Gone => Some(Self::Gone),
            Reread::Failed(reason) => Some(Self::Failed(format!("cannot reload: {reason}").into())),
        }
    }

    pub(crate) fn stamp(&self) -> Option<Stamp> {
        match self {
            Self::Read(stamp) => Some(*stamp),
            Self::Unstamped | Self::Gone | Self::Failed(_) => None,
        }
    }

    /// The note about a failed re-read, shown until a re-read finds something else.
    pub(crate) fn failure(&self) -> Option<&SharedString> {
        match self {
            Self::Failed(note) => Some(note),
            Self::Read(_) | Self::Unstamped | Self::Gone => None,
        }
    }
}

/// When IVE last read the file, and why a re-read failed.
pub struct DiskText {
    disk: Disk,
}

impl DiskText {
    /// Shows `document` in `panel`.
    pub fn show(document: Document, panel: &Entity<Panel>, cx: &mut App) -> Self {
        let disk = document::stamp(document.path()).map_or(Disk::Unstamped, Disk::Read);
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(viewed(document)), cx));
        Self { disk }
    }

    /// What a re-read compares the file with; `None` after the file went away or a re-read failed, so the next
    /// change reads it.
    pub fn stamp(&self) -> Option<Stamp> {
        self.disk.stamp()
    }

    /// New text goes to `panel`, keeping its scroll; a file that is gone or cannot be read keeps its old text,
    /// with a note saying why.
    pub fn take(&mut self, reread: Reread, panel: &Entity<Panel>, cx: &mut App) {
        let Some(disk) = Disk::after(&reread) else {
            return;
        };
        self.disk = disk;
        let cast = match reread {
            Reread::Changed(document, _) => Cast::Open(viewed(document)),
            Reread::Gone => Cast::MarkDeleted,
            Reread::Unchanged | Reread::Failed(_) => return,
        };
        panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    /// The panel under the note about a failed re-read, if there is one.
    pub fn render(&self, panel: &Entity<Panel>, theme: Theme) -> AnyElement {
        let Some(failure) = self.disk.failure().cloned() else {
            return panel.clone().into_any_element();
        };
        div().flex().flex_col().flex_1().min_h_0().child(failure_note(failure, theme)).child(panel.clone()).into_any_element()
    }
}

/// `document` to view, coloured as its name says.
fn viewed(document: Document) -> Content {
    let language = Language::for_path(document.path(), document.bytes());
    Content::Lines { source: Arc::new(document), language, edit: None }
}

/// Drawn as the panel draws its "deleted on disk" note.
pub(crate) fn failure_note(failure: SharedString, theme: Theme) -> Div {
    let sizes = &theme.sizes.editor;
    div()
        .flex_none()
        .px(sizes.notice_padding_x.px())
        .py(sizes.notice_padding_y.px())
        .bg(rgb(theme.hover))
        .text_size(sizes.notice_text_size.px())
        .text_color(rgb(theme.error))
        .child(failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_no_stale_failure_once_the_file_is_gone() {
        // given
        let failed = Disk::after(&Reread::Failed("permission denied".to_owned())).unwrap();

        // when
        let gone = Disk::after(&Reread::Gone).unwrap();

        // then
        assert_eq!(failed.failure().map(|note| note.as_ref()), Some("cannot reload: permission denied"));
        assert_eq!(gone.failure(), None);
        assert_eq!(gone.stamp(), None);
    }

    #[test]
    fn should_keep_what_is_known_when_nothing_changed() {
        // given, when
        let next = Disk::after(&Reread::Unchanged);

        // then
        assert_eq!(next, None);
    }
}
