//! The tabs shown, newest last, open or closed, and when each worktree was last used: the hub's Recent rows and
//! its worktree order.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::saved_tabs::SavedTab;
pub use crate::state::{MAX_HISTORY as HISTORY_LIMIT, MAX_WORKTREE_USE, SavedShown, SavedUse};

/// What a tab shows, apart from its id and scroll: the same identity is the same Recent row, open or closed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabIdentity {
    File(PathBuf),
    Scratch { project: String, name: String },
    /// a hosted terminal or Claude Code session, which outlives IVE
    Terminal { session: String },
    /// a process IVE does not host (a run's output), by its id; for this session only
    Process(u64),
    ProjectOverview(String),
    GitLog(String),
    Commit { project: String, sha: String },
    Diff { checkout: PathBuf, path: Vec<u8> },
    Settings,
    RunValues { checkout: PathBuf, folder: PathBuf, key: String },
    /// never listed: the hub's own Recent leaves it out
    Hub,
}

impl TabIdentity {
    /// `None` for an older IVE's worktree overview, which is gone.
    pub fn of_saved(saved: &SavedTab) -> Option<TabIdentity> {
        let identity = match saved {
            SavedTab::File { path, .. } => Self::File(path.clone()),
            SavedTab::Scratch { project, name, .. } => Self::Scratch { project: project.clone(), name: name.clone() },
            SavedTab::Terminal { session } => Self::Terminal { session: session.clone() },
            SavedTab::ProjectOverview { project, .. } => Self::ProjectOverview(project.clone()),
            SavedTab::WorktreeOverview { .. } => return None,
            SavedTab::GitLog { project, .. } => Self::GitLog(project.clone()),
            SavedTab::Commit { project, sha, .. } => Self::Commit { project: project.clone(), sha: sha.clone() },
            SavedTab::Diff { checkout, request, .. } => Self::Diff { checkout: checkout.clone(), path: request.path.clone() },
            SavedTab::Settings { .. } => Self::Settings,
            SavedTab::Hub => Self::Hub,
        };
        Some(identity)
    }
}

/// One tab as last shown.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    pub identity: TabIdentity,
    /// how to reopen it after a restart; `None` for kinds that are never saved
    pub saved: Option<SavedTab>,
    pub checkout: Option<PathBuf>,
    pub title: String,
    /// seconds since the epoch
    pub shown_at: i64,
}

/// Shown tabs, newest last, one entry per identity, at most [`HISTORY_LIMIT`].
#[derive(Debug, Clone, Default)]
pub struct History {
    entries: Vec<Shown>,
}

impl History {
    pub fn shown(&mut self, entry: Shown) {
        self.entries.retain(|shown| shown.identity != entry.identity);
        self.entries.push(entry);
        let extra = self.entries.len().saturating_sub(HISTORY_LIMIT);
        self.entries.drain(..extra);
    }

    pub fn newest_first(&self) -> impl Iterator<Item = &Shown> {
        self.entries.iter().rev()
    }

    /// The entries that can be reopened after a restart, oldest first.
    pub fn saved(&self) -> Vec<SavedShown> {
        let saved = self.entries.iter().filter_map(|shown| {
            let tab = shown.saved.clone()?;
            Some(SavedShown { shown_at: shown.shown_at, checkout: shown.checkout.clone(), tab })
        });
        saved.collect()
    }

    pub fn restore(saved: Vec<SavedShown>) -> History {
        let mut history = History::default();
        for SavedShown { shown_at, checkout, tab } in saved {
            let Some(identity) = TabIdentity::of_saved(&tab) else {
                continue;
            };
            let entry = Shown { identity, checkout, title: tab.title(), saved: Some(tab), shown_at };
            history.shown(entry);
        }
        history
    }

    /// When a tab of `checkout` was last shown.
    fn last_shown(&self, checkout: &Path) -> Option<i64> {
        self.newest_first().find(|shown| shown.checkout.as_deref() == Some(checkout)).map(|shown| shown.shown_at)
    }
}

/// The last input typed into a checkout's terminals or run started in it, at most [`MAX_WORKTREE_USE`] checkouts.
#[derive(Debug, Clone, Default)]
pub struct WorktreeUse {
    used: HashMap<PathBuf, i64>,
}

impl WorktreeUse {
    pub fn used(&mut self, checkout: &Path, at: i64) {
        self.used.insert(checkout.to_path_buf(), at);
        if self.used.len() > MAX_WORKTREE_USE
            && let Some(oldest) = self.used.iter().min_by_key(|(_, at)| **at).map(|(path, _)| path.clone())
        {
            self.used.remove(&oldest);
        }
    }

    /// The later of the last use and the last time one of its tabs was shown.
    pub fn last_used(&self, checkout: &Path, history: &History) -> Option<i64> {
        self.used.get(checkout).copied().max(history.last_shown(checkout))
    }

    /// Sorted by checkout, so an unchanged state saves the same text.
    pub fn saved(&self) -> Vec<SavedUse> {
        let mut saved: Vec<SavedUse> = self.used.iter().map(|(checkout, at)| SavedUse { checkout: checkout.clone(), used_at: *at }).collect();
        saved.sort_by(|a, b| a.checkout.cmp(&b.checkout));
        saved
    }

    pub fn restore(saved: Vec<SavedUse>) -> WorktreeUse {
        WorktreeUse { used: saved.into_iter().map(|saved| (saved.checkout, saved.used_at)).collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saved_tabs::PanelState;

    fn file(path: &str, shown_at: i64) -> Shown {
        let path = PathBuf::from(path);
        let saved = SavedTab::File { path: path.clone(), panel: PanelState::default() };
        Shown { identity: TabIdentity::File(path), saved: Some(saved), checkout: Some(PathBuf::from("/w")), title: String::new(), shown_at }
    }

    fn identities(history: &History) -> Vec<TabIdentity> {
        history.newest_first().map(|shown| shown.identity.clone()).collect()
    }

    #[test]
    fn should_move_a_reshown_tab_to_the_newest() {
        // given
        let mut history = History::default();

        // when
        for (path, at) in [("/a", 1), ("/b", 2), ("/a", 3)] {
            history.shown(file(path, at));
        }

        // then
        assert_eq!(identities(&history), [TabIdentity::File("/a".into()), TabIdentity::File("/b".into())]);
    }

    #[test]
    fn should_bound_the_history() {
        // given
        let mut history = History::default();

        // when
        for number in 0..=HISTORY_LIMIT {
            history.shown(file(&format!("/f{number}"), number as i64));
        }

        // then
        assert_eq!(history.newest_first().count(), HISTORY_LIMIT);
        assert_eq!(history.newest_first().last().map(|shown| shown.identity.clone()), Some(TabIdentity::File("/f1".into())));
    }

    #[test]
    fn should_save_only_entries_with_a_saved_form() {
        // given
        let mut history = History::default();
        history.shown(file("/a", 1));
        history.shown(Shown { identity: TabIdentity::Process(7), saved: None, checkout: None, title: "run".to_owned(), shown_at: 2 });

        // when
        let saved = history.saved();

        // then
        assert_eq!(saved.iter().map(|saved| saved.shown_at).collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn should_restore_saved_entries_in_order() {
        // given
        let mut history = History::default();
        history.shown(file("/a", 1));
        history.shown(file("/b", 2));

        // when
        let restored = History::restore(history.saved());

        // then
        assert_eq!(identities(&restored), identities(&history));
        assert_eq!(restored.newest_first().next().map(|shown| (shown.title.as_str(), shown.checkout.clone())), Some(("b", Some(PathBuf::from("/w")))));
    }

    #[test]
    fn should_drop_an_older_ive_s_worktree_overview_on_restore() {
        // given
        let mut saved = History::default();
        saved.shown(file("/a", 1));
        let mut entries = saved.saved();
        let overview = SavedTab::WorktreeOverview { checkout: PathBuf::from("/w"), section: "Git".to_owned() };
        entries.push(SavedShown { shown_at: 2, checkout: Some(PathBuf::from("/w")), tab: overview });

        // when
        let restored = History::restore(entries);

        // then
        assert_eq!(identities(&restored), identities(&saved));
    }

    #[test]
    fn should_take_the_later_of_use_and_history() {
        // given
        let mut history = History::default();
        history.shown(file("/a", 100));
        let mut early = WorktreeUse::default();
        early.used(Path::new("/w"), 50);
        let mut late = WorktreeUse::default();
        late.used(Path::new("/w"), 200);

        // when
        let times = [early.last_used(Path::new("/w"), &history), late.last_used(Path::new("/w"), &history), late.last_used(Path::new("/x"), &history)];

        // then
        assert_eq!(times, [Some(100), Some(200), None]);
    }

    #[test]
    fn should_bound_worktree_use() {
        // given
        let mut used = WorktreeUse::default();

        // when
        for number in 0..=MAX_WORKTREE_USE {
            used.used(&PathBuf::from(format!("/w{number}")), number as i64);
        }

        // then
        assert_eq!(used.saved().len(), MAX_WORKTREE_USE);
        assert!(used.saved().iter().all(|saved| saved.checkout != Path::new("/w0")));
    }
}
