//! When a panel in Edit mode asks its host to save, what it waits for, and the bar saying why a save failed.

use std::time::Duration;

use gpui::{App, ClickEvent, Div, SharedString, Stateful, Subscription, Task, Window, div, prelude::*, px, rgb};

use super::protocol::{Edit, Resolve, SaveFailure, SaveKind, SaveRequest};
use crate::host::Host;
use crate::piece_table::WrongBase;
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
    close: Close,
}

/// Whether the host waits for the edits to close the tab or quit, and the user gave them up.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Close {
    Unasked,
    /// the host would close but for the unsaved edits: a bar offers closing without them
    Asked,
    /// the user chose to lose the edits: they count as saved, and nothing is written
    Discarded,
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
    /// the file changed on disk: the user reloads it, or may overwrite it
    Conflict(Resolve),
}

/// The buttons the bar offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Choices {
    Nothing,
    Reload,
    ReloadOrOverwrite,
}

/// Whether the edits were given up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Discarded {
    Done,
    Refused,
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
    /// the base is the saved text now; `then` says whether to save again now
    Saved { then: Then },
    Failed,
    /// for an older save, or after a reload
    Stale,
}

impl Saving {
    /// Nothing unsaved in revision `saved`.
    pub(super) fn new(delay: Duration, saved: u64) -> Self {
        Self { delay, saved, flight: Flight::Idle, pause: Pause::Idle, problem: Problem::Nothing, close: Close::Unasked }
    }

    pub(super) fn has_unsaved(&self, revision: u64) -> bool {
        revision != self.saved && self.close != Close::Discarded
    }

    /// The host waits for the edits to close the tab or quit; ends with a save.
    pub(super) fn ask_close(&mut self) {
        self.close = Close::Asked;
    }

    /// The bar offers closing without saving: a close waits, and the last save failed or met a conflict, with no
    /// save in flight.
    pub(super) fn offers_discard(&self) -> bool {
        self.close == Close::Asked && !self.is_in_flight() && self.problem != Problem::Nothing
    }

    /// The user chose closing without saving; no-op unless [`Self::offers_discard`]. Returns what happened.
    pub(super) fn discard(&mut self) -> Discarded {
        if !self.offers_discard() {
            return Discarded::Refused;
        }
        self.close = Close::Discarded;
        self.pause = Pause::Idle;
        Discarded::Done
    }

    pub(super) fn is_in_flight(&self) -> bool {
        matches!(self.flight, Flight::Sent { .. })
    }

    pub(super) fn problem(&self) -> &Problem {
        &self.problem
    }

    /// What the user may do about a conflict: nothing while a save is in flight, whose answer changes the text.
    pub(super) fn choices(&self) -> Choices {
        match (&self.problem, &self.flight) {
            (Problem::Conflict(_), Flight::Sent { .. }) | (Problem::Nothing | Problem::Failed(_), _) => Choices::Nothing,
            (Problem::Conflict(Resolve::Reload), Flight::Idle) => Choices::Reload,
            (Problem::Conflict(Resolve::ReloadOrOverwrite), Flight::Idle) => Choices::ReloadOrOverwrite,
        }
    }

    /// An edit restarts the pause; `pause` is the timer that ends it.
    pub(super) fn restart_pause(&mut self, pause: Task<()>) {
        if self.close == Close::Discarded {
            self.close = Close::Unasked;
        }
        self.pause = Pause::Running { _timer: pause };
    }

    /// What to do now about the text of `revision`: `edits` gives its changes since the saved text. An ordinary
    /// save waits while a conflict is unanswered, an overwrite needs a conflict that allows it; one in flight makes
    /// this one follow its answer.
    pub(super) fn next(&mut self, kind: SaveKind, revision: u64, edits: impl FnOnce() -> Vec<Edit>) -> Next {
        self.pause = Pause::Idle;
        if self.close == Close::Discarded || self.is_held(kind) {
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

    /// Whether a save of `kind` waits for the user's answer to a conflict.
    fn is_held(&self, kind: SaveKind) -> bool {
        match (kind, &self.problem) {
            (SaveKind::Checked, Problem::Conflict(_)) => true,
            (SaveKind::Checked, Problem::Nothing | Problem::Failed(_)) => false,
            (SaveKind::Overwrite, Problem::Conflict(Resolve::ReloadOrOverwrite)) => false,
            (SaveKind::Overwrite, Problem::Conflict(Resolve::Reload) | Problem::Nothing | Problem::Failed(_)) => true,
        }
    }

    /// The host wrote revision `revision`; `rebase` moves the text onto the saved one, given the edits that made it.
    /// A saved text that cannot be the edited one (another program changed the file right after the write) keeps
    /// the edits, and only a reload resolves that: the host's text and the panel's differ.
    pub(super) fn saved(&mut self, revision: u64, rebase: impl FnOnce(&[Edit]) -> Result<(), WrongBase>) -> Answer {
        let Flight::Sent { edits, then, .. } = self.take_flight(revision) else {
            return Answer::Stale;
        };
        if rebase(&edits).is_err() {
            self.problem = Problem::Conflict(Resolve::Reload);
            return Answer::Failed;
        }
        self.saved = revision;
        self.problem = Problem::Nothing;
        self.close = Close::Unasked;
        Answer::Saved { then }
    }

    /// The host could not write revision `revision`; the edits stay, and the next pause or flush tries again.
    pub(super) fn failed(&mut self, revision: u64, reason: SaveFailure) -> Answer {
        let Flight::Sent { .. } = self.take_flight(revision) else {
            return Answer::Stale;
        };
        self.problem = match reason {
            SaveFailure::Conflict(resolve) => Problem::Conflict(resolve),
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
    pub(super) discard: Action,
}

/// The bar above the text while a save failed: the reason, or for a conflict the [`Saving::choices`]. Click targets
/// `save-reload` and `save-overwrite`, and while a close waits, `save-discard`.
pub(super) fn bar(saving: &Saving, actions: BarActions, cx: &App) -> Option<Div> {
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
    let row = match saving.problem() {
        Problem::Nothing => return None,
        Problem::Failed(reason) => row.child(format!("not saved: {reason}")),
        Problem::Conflict(_) => row.child("changed on disk; not saved"),
    };
    let reload = Button { name: "save-reload", label: "Reload", action: actions.reload };
    let overwrite = Button { name: "save-overwrite", label: "Overwrite", action: actions.overwrite };
    let row = match saving.choices() {
        Choices::Nothing => row,
        Choices::Reload => row.child(button(reload, style, cx)),
        Choices::ReloadOrOverwrite => row.child(button(reload, style, cx)).child(button(overwrite, style, cx)),
    };
    if saving.offers_discard() {
        let discard = Button { name: "save-discard", label: "Close without saving (edits are lost)", action: actions.discard };
        return Some(row.child(button(discard, style, cx)));
    }
    Some(row)
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
