//! When a panel in Edit mode asks its host to save, what it waits for, and the bar saying why a save failed.

use std::time::Duration;

use gpui::{App, ClickEvent, Div, SharedString, Stateful, Subscription, Task, Window, div, prelude::*, px, rgb};

use super::protocol::{Edit, SaveFailure, SaveKind, SaveRequest};
use crate::host::Host;
use crate::style::ViewStyle;

/// The save state of the text being edited; see the spec's "Saving".
pub(super) struct Saving {
    /// the pause after the last edit before a save
    pub(super) delay: Duration,
    /// the revision the host has on disk
    saved: u64,
    flight: Flight,
    pause: Pause,
    problem: Problem,
}

/// The one save the host is writing.
enum Flight {
    Idle,
    Sent { revision: u64, edits: Vec<Edit>, then: Then },
}

/// What follows the answer to the save in flight.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Then {
    Wait,
    /// a pause ended or a flush came meanwhile: save what was typed since
    SaveAgain,
}

enum Pause {
    Idle,
    /// ends with a save; replaced by each edit
    Running { _timer: Task<()> },
}

/// Why the last save did not go through; shown in a bar until a save does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Problem {
    Nothing,
    Failed(String),
    /// the file changed on disk: the user reloads it or overwrites it
    Conflict,
}

/// What the panel does about a save request.
pub(super) enum Next {
    Nothing,
    /// the text is as saved again (edits undone): nothing to write
    AlreadySaved,
    Send(SaveRequest),
}

/// The answer to a save, for the save in flight.
pub(super) enum Answer {
    /// the base becomes the saved text, which `edits` made of the old one; `then` says whether to save again now
    Saved { edits: Vec<Edit>, then: Then },
    Failed,
    /// for an older save, or after a reload
    Stale,
}

impl Saving {
    /// Nothing unsaved in revision `saved`.
    pub(super) fn new(delay: Duration, saved: u64) -> Self {
        Self { delay, saved, flight: Flight::Idle, pause: Pause::Idle, problem: Problem::Nothing }
    }

    pub(super) fn has_unsaved(&self, revision: u64) -> bool {
        revision != self.saved
    }

    pub(super) fn is_in_flight(&self) -> bool {
        matches!(self.flight, Flight::Sent { .. })
    }

    pub(super) fn problem(&self) -> &Problem {
        &self.problem
    }

    /// An edit restarts the pause; `pause` is the timer that ends it.
    pub(super) fn restart_pause(&mut self, pause: Task<()>) {
        self.pause = Pause::Running { _timer: pause };
    }

    /// What to do now about the text of `revision`: `edits` gives its changes since the saved text. An ordinary
    /// save waits while a conflict is unanswered; one in flight makes this one follow its answer.
    pub(super) fn next(&mut self, kind: SaveKind, revision: u64, edits: impl FnOnce() -> Vec<Edit>) -> Next {
        self.pause = Pause::Idle;
        if kind == SaveKind::Checked && self.problem == Problem::Conflict {
            return Next::Nothing;
        }
        if let Flight::Sent { then, .. } = &mut self.flight {
            *then = Then::SaveAgain;
            return Next::Nothing;
        }
        if revision == self.saved {
            return Next::Nothing;
        }
        let edits = edits();
        if edits.is_empty() {
            self.saved = revision;
            return Next::AlreadySaved;
        }
        self.flight = Flight::Sent { revision, edits: edits.clone(), then: Then::Wait };
        Next::Send(SaveRequest { base: self.saved, revision, edits, kind })
    }

    /// The host wrote revision `revision`.
    pub(super) fn saved(&mut self, revision: u64) -> Answer {
        let Flight::Sent { edits, then, .. } = self.take_flight(revision) else {
            return Answer::Stale;
        };
        self.saved = revision;
        self.problem = Problem::Nothing;
        Answer::Saved { edits, then }
    }

    /// The host could not write revision `revision`; the edits stay, and the next pause or flush tries again.
    pub(super) fn failed(&mut self, revision: u64, reason: SaveFailure) -> Answer {
        let Flight::Sent { .. } = self.take_flight(revision) else {
            return Answer::Stale;
        };
        self.problem = match reason {
            SaveFailure::Conflict => Problem::Conflict,
            SaveFailure::Io(reason) => Problem::Failed(reason),
        };
        Answer::Failed
    }

    /// The save in flight when it is the one for `revision`, which is then answered; else [`Flight::Idle`].
    fn take_flight(&mut self, revision: u64) -> Flight {
        match &self.flight {
            Flight::Sent { revision: sent, .. } if *sent == revision => std::mem::replace(&mut self.flight, Flight::Idle),
            Flight::Sent { .. } | Flight::Idle => Flight::Idle,
        }
    }
}

/// What a button of the bar does.
pub(super) type Action = Box<dyn Fn(&mut Window, &mut App)>;

/// What the bar's buttons do.
pub(super) struct BarActions {
    pub(super) reload: Action,
    pub(super) overwrite: Action,
}

/// The bar above the text while a save failed: the reason, or for a conflict Reload and Overwrite. Click targets
/// `save-reload` and `save-overwrite`.
pub(super) fn bar(problem: &Problem, actions: BarActions, cx: &App) -> Option<Div> {
    let style = ViewStyle::drawing(cx);
    let row = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(style.editor_notice_padding_x))
        .px(px(style.editor_notice_padding_x))
        .py(px(style.editor_notice_padding_y))
        .bg(rgb(style.hover))
        .text_size(px(style.editor_notice_text_size))
        .text_color(rgb(style.error));
    match problem {
        Problem::Nothing => None,
        Problem::Failed(reason) => Some(row.child(format!("not saved: {reason}"))),
        Problem::Conflict => Some(
            row.child("changed on disk; not saved")
                .child(button(Button { name: "save-reload", label: "Reload", action: actions.reload }, style, cx))
                .child(button(Button { name: "save-overwrite", label: "Overwrite", action: actions.overwrite }, style, cx)),
        ),
    }
}

/// A button of the bar; `name` is its element id and its click target.
struct Button {
    name: &'static str,
    label: &'static str,
    action: Action,
}

fn button(button: Button, style: &ViewStyle, cx: &App) -> Stateful<Div> {
    let Button { name, label, action } = button;
    let click = Host::click_target(name.to_owned(), move |_: &ClickEvent, window, cx| action(window, cx), cx);
    div()
        .id(SharedString::from(name))
        .px(px(style.editor_notice_padding_x))
        .rounded(px(style.editor_notice_padding_y))
        .border_1()
        .border_color(rgb(style.error))
        .cursor_pointer()
        .child(label)
        .on_click(click)
}

/// Watches the editor's focus to save when it leaves.
pub(super) enum Blur {
    Unwatched,
    Watched { _subscription: Subscription },
}
