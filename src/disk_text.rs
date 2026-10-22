//! A file a panel shows as it is on disk: what IVE knows of it, and taking what re-reading it found.

use std::sync::Arc;

use codeview::document::{self, Reread, Stamp};
use codeview::panel::{Cast, Content, Panel};
use gpui::{AnyElement, App, Div, Entity, SharedString, div, prelude::*, rgb};

use crate::document::Document;
use crate::syntax::Language;
use crate::theme::Theme;

/// When IVE last read the file, and why a re-read failed.
pub struct DiskText {
    /// size and modification time when last read; `None` after the file went away or a re-read failed, so the
    /// next change reads it
    stamp: Option<Stamp>,
    /// why the last re-read failed, shown above the text until a re-read succeeds
    failure: Option<SharedString>,
}

impl DiskText {
    /// Shows `document` in `panel`.
    pub fn show(document: Document, panel: &Entity<Panel>, cx: &mut App) -> Self {
        let stamp = document::stamp(document.path());
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(viewed(document)), cx));
        Self { stamp, failure: None }
    }

    /// What a re-read compares the file with.
    pub fn stamp(&self) -> Option<Stamp> {
        self.stamp
    }

    /// New text goes to `panel`, keeping its scroll; a file that is gone or cannot be read keeps its old text,
    /// with a note saying why.
    pub fn take(&mut self, reread: Reread, panel: &Entity<Panel>, cx: &mut App) {
        let cast = match reread {
            Reread::Unchanged => return,
            Reread::Changed(document, stamp) => {
                self.stamp = stamp;
                self.failure = None;
                Cast::Open(viewed(document))
            }
            Reread::Gone => {
                self.stamp = None;
                Cast::MarkDeleted
            }
            Reread::Failed(reason) => {
                self.stamp = None;
                self.failure = Some(format!("cannot reload: {reason}").into());
                return;
            }
        };
        panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    /// The panel under the note about a failed re-read, if there is one.
    pub fn render(&self, panel: &Entity<Panel>, theme: Theme) -> AnyElement {
        let Some(failure) = self.failure.clone() else {
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
fn failure_note(failure: SharedString, theme: Theme) -> Div {
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
