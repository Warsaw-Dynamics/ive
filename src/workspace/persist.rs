//! Writes `state.toml` after changes: at most once per 500 ms, one write at a time, not when nothing changed,
//! and once more when IVE closes.

use std::path::PathBuf;
use std::time::Duration;

use codeview::panel::Event;
use gpui::{App, Context};

use super::Workspace;
use crate::tabs::TabId;
use crate::state::{SavedState, StateError, VERSION, Writer};

const SAVE_DELAY: Duration = Duration::from_millis(500);

/// When to write. A change starts a 500 ms wait unless a wait or a write is under way; then it joins the next
/// write. A state equal to the one last written is not written again.
#[derive(Debug, Default)]
struct Saves {
    /// a change not written yet
    pending: bool,
    /// a wait or a write is under way
    busy: bool,
    /// what the last write took, or what the file held at start
    written: Option<SavedState>,
    /// number of the last write taken; numbers keep an earlier write from replacing a later one
    taken: u64,
}

impl Saves {
    /// Something saved changed; true when a wait must start now.
    fn changed(&mut self) -> bool {
        self.pending = true;
        if self.busy {
            return false;
        }
        self.busy = true;
        true
    }

    /// A wait ended: the state to write, numbered, unless it equals the last one written.
    fn take(&mut self, state: SavedState) -> Option<(u64, SavedState)> {
        self.pending = false;
        if self.written.as_ref() == Some(&state) {
            return None;
        }
        self.written = Some(state.clone());
        self.taken += 1;
        Some((self.taken, state))
    }

    /// A wait or a write ended; true when changes came meanwhile, so another wait must start.
    fn finished(&mut self) -> bool {
        self.busy = self.pending;
        self.pending
    }

    /// A write failed: the next change writes again, even an unchanged state.
    fn failed(&mut self) {
        self.written = None;
    }

    /// IVE is closing: the state to write now while something may not be on disk, a change not taken yet or a
    /// write still running in the background, which may not finish before IVE exits.
    fn take_final(&mut self, state: SavedState) -> Option<(u64, SavedState)> {
        if self.pending || self.busy {
            self.pending = false;
            self.written = Some(state.clone());
            self.taken += 1;
            return Some((self.taken, state));
        }
        None
    }
}

/// Where state goes, when it is written, and the last write error.
pub(super) struct StateFile {
    path: Option<PathBuf>,
    saves: Saves,
    writer: Writer,
    /// the last write error printed, so a failing disk is reported once, not every 500 ms
    reported: Option<String>,
}

impl StateFile {
    /// `restored` is what the file held at start.
    pub(super) fn new(path: Option<PathBuf>, restored: SavedState) -> Self {
        Self { path, saves: Saves { written: Some(restored), ..Saves::default() }, writer: Writer::default(), reported: None }
    }
}

/// One write, ready to run on any thread.
struct Write {
    number: u64,
    state: SavedState,
    path: PathBuf,
    writer: Writer,
}

impl Write {
    fn run(&self) -> Result<(), StateError> {
        self.writer.write(self.number, &self.state, &self.path)
    }
}

impl Workspace {
    fn saved_state(&self, cx: &App) -> SavedState {
        let every = (self.shown_folders != crate::sidebar::folders::default_shown()).then(|| self.shown_folders.iter().copied().collect());
        SavedState {
            version: VERSION,
            shown_folders: every,
            hidden_projects: self.hidden_projects.iter().map(|id| id.as_str().to_owned()).collect(),
            projects: self.projects.iter().map(|open| open.saved()).collect(),
            recent: self.recent.clone(),
            tabs: self.saved_tabs(cx),
            window: self.window_place.saved(),
            history: self.history.saved(),
            worktree_use: self.worktree_use.saved(),
        }
    }

    /// Saves soon: changes within the next 500 ms, or made while a write runs, go into one write. Called after a
    /// change to the project list, Recent, an expansion or a last focused checkout, and on every redraw, which may
    /// have moved a tab's scroll or caret (an unchanged state is not written).
    pub(crate) fn schedule_save(&mut self, cx: &mut Context<Self>) {
        if self.state.path.is_some() && self.state.saves.changed() {
            self.spawn_save(cx);
        }
    }

    /// Saves soon whenever the panel of tab `id` (a file, diff or commit tab) scrolls or changes layout, so a crash
    /// does not lose it; the subscription ends with the panel.
    pub(super) fn save_on_state_change(&self, id: TabId, cx: &mut Context<Self>) {
        let Some(panel) = self.tabs.get(id).and_then(|tab| tab.saved_panel(cx)) else {
            return;
        };
        cx.subscribe(&panel, |workspace, _panel, event: &Event, cx| {
            if let Event::StateChanged = event {
                workspace.schedule_save(cx);
            }
        })
        .detach();
    }

    fn spawn_save(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let Ok(write) = this.update(cx, |workspace, cx| workspace.take_write(cx)) else {
                return;
            };
            if let Some(write) = write {
                let result = cx.background_executor().spawn(async move { write.run() }).await;
                if this.update(cx, |workspace, _cx| workspace.report_save(result)).is_err() {
                    return;
                }
            }
            this.update(cx, |workspace, cx| workspace.save_finished(cx)).ok();
        })
        .detach();
    }

    fn take_write(&mut self, cx: &App) -> Option<Write> {
        let path = self.state.path.clone()?;
        let (number, state) = self.state.saves.take(self.saved_state(cx))?;
        Some(Write { number, state, path, writer: self.state.writer.clone() })
    }

    fn save_finished(&mut self, cx: &mut Context<Self>) {
        if self.state.saves.finished() {
            self.spawn_save(cx);
        }
    }

    /// Writes now, on this thread, whatever may not be on disk yet: IVE is closing. A background write still
    /// running finishes first (the writer runs one at a time), and its earlier number keeps it from replacing
    /// this one.
    pub(super) fn flush_state(&mut self, cx: &App) {
        let Some(path) = self.state.path.clone() else {
            return;
        };
        if let Some((number, state)) = self.state.saves.take_final(self.saved_state(cx)) {
            let result = Write { number, state, path, writer: self.state.writer.clone() }.run();
            self.report_save(result);
        }
    }

    fn report_save(&mut self, result: Result<(), StateError>) {
        match result {
            Ok(()) => self.state.reported = None,
            Err(error) => {
                self.state.saves.failed();
                let message = error.to_string();
                if self.state.reported.as_ref() != Some(&message) {
                    eprintln!("ive: {message}");
                    self.state.reported = Some(message);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::RecentProject;

    fn state(recent: &str) -> SavedState {
        SavedState { recent: vec![RecentProject { path: PathBuf::from("/r"), id: recent.to_owned() }], ..SavedState::default() }
    }

    #[test]
    fn should_write_once_for_changes_made_while_waiting_and_skip_an_unchanged_state() {
        // given
        let mut saves = Saves::default();

        // when
        let waits = [saves.changed(), saves.changed(), saves.changed()];
        let first = saves.take(state("a"));
        let again = saves.finished();
        let second_wait = saves.changed();
        let unchanged = saves.take(state("a"));

        // then
        assert_eq!((waits, first, again), ([true, false, false], Some((1, state("a"))), false));
        assert_eq!((second_wait, unchanged), (true, None));
    }

    #[test]
    fn should_start_another_write_for_a_change_made_while_one_runs() {
        // given
        let mut saves = Saves::default();
        saves.changed();
        saves.take(state("a"));

        // when
        let wait_during_write = saves.changed();
        let another = saves.finished();
        let second = saves.take(state("b"));

        // then
        assert_eq!((wait_during_write, another, second), (false, true, Some((2, state("b")))));
    }

    #[test]
    fn should_write_at_quit_while_a_change_waits_or_a_write_runs_and_not_when_all_is_on_disk() {
        // given
        let (mut waiting, mut writing, mut done) = (Saves::default(), Saves::default(), Saves::default());
        waiting.changed();
        for saves in [&mut writing, &mut done] {
            saves.changed();
            saves.take(state("a"));
        }
        done.finished();

        // when
        let finals = [&mut waiting, &mut writing, &mut done].map(|saves| saves.take_final(state("a")));

        // then
        assert_eq!(finals, [Some((1, state("a"))), Some((2, state("a"))), None]);
    }

    #[test]
    fn should_write_an_unchanged_state_again_after_a_failed_write() {
        // given
        let mut saves = Saves { written: Some(state("a")), ..Saves::default() };
        saves.failed();
        saves.changed();

        // when
        let retried = saves.take(state("a"));

        // then
        assert_eq!(retried, Some((1, state("a"))));
    }
}
