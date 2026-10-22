//! The open tabs as `state.toml` keeps them, and bringing them back at start.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use codeview::panel::Cast;
use gpui::{App, Context, Window};

use super::Workspace;
use crate::document::Document;
use crate::editor::{FileTab, Tab};
use crate::git::types::Sha;
use crate::hub::model::HubState;
use crate::overview::commit_tab::CommitTab;
use crate::overview::diff_tab::{DiffRequest, DiffTab, DiffTabSpec};
use crate::overview::{ProjectOverview, ProjectSection};
use crate::process::ProcessId;
use crate::project::{self, CheckoutRef, ProjectId};
use crate::saved_tabs::{MAX_TABS, PanelState, SavedTab, SavedTabs};
use crate::scratch::ScratchRef;
use crate::settings_page::{SettingsPage, SettingsView, THIRD_PARTY_TITLE};
use crate::tabs::{TabId, Tabs};

impl Workspace {
    /// Opens the saved tabs whose terminal, project or checkout is still there, in order, and shows the saved
    /// shown one. Terminals are attached already (`sessions`); every other tab loads when first shown.
    pub(super) fn restore_tabs(&mut self, saved: SavedTabs, sessions: &HashMap<String, ProcessId>, window: &mut Window, cx: &mut Context<Self>) {
        let live: HashSet<String> = sessions.keys().cloned().collect();
        let projects = self.projects.iter().map(|open| (open.project.id.as_str().to_owned(), open.project.root.clone())).collect();
        let open = Open { sessions: &live, projects };
        let kept = kept_tabs(saved, |tab| open.keeps(tab));
        let ids: Vec<TabId> = kept
            .tabs
            .into_iter()
            .map(|tab| match tab {
                SavedTab::Terminal { session } => Tab::Process(sessions[&session]),
                other => Tab::Unloaded(other),
            })
            .map(|tab| self.tabs.open(tab))
            .collect();
        if let Some(&shown) = kept.active.and_then(|index| ids.get(index)) {
            self.activate_tab(shown, window, cx);
        }
    }

    /// Loads the shown tab after this frame when it is a restored one whose project has been read.
    pub(super) fn load_shown_tab_later(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((id, Tab::Unloaded(saved))) = self.tabs.active_id().zip(self.tabs.active()) else {
            return;
        };
        if self.can_load(saved) {
            cx.defer_in(window, move |workspace, window, cx| workspace.load_tab(id, window, cx));
        }
    }

    /// Tabs of a project's git or checkouts wait for its first scan, which tells what it is and which worktrees
    /// it has, or that its folder is gone.
    fn can_load(&self, saved: &SavedTab) -> bool {
        let loaded = |id: &str| self.projects.iter().any(|open| open.project.id.as_str() == id && open.is_read());
        let loaded_at = |path: &Path| self.projects.iter().any(|open| path.starts_with(&open.project.root) && open.is_read());
        match saved {
            SavedTab::ProjectOverview { project, .. } | SavedTab::GitLog { project, .. } | SavedTab::Commit { project, .. } => loaded(project),
            SavedTab::WorktreeOverview { checkout, .. } | SavedTab::Diff { checkout, .. } => loaded_at(checkout),
            SavedTab::File { .. } | SavedTab::Scratch { .. } | SavedTab::Terminal { .. } | SavedTab::Settings { .. } | SavedTab::Hub => true,
        }
    }

    /// Replaces restored tab `id` with the tab it stands for; one whose project, checkout or scratch is gone
    /// closes, a file that cannot be read shows why.
    fn load_tab(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tab::Unloaded(saved)) = self.tabs.get(id) else {
            return;
        };
        let saved = saved.clone();
        let Some(tab) = self.loaded_tab(&saved, window, cx) else {
            self.tabs.drop_tab(id);
            return self.tab_shown(window, cx);
        };
        self.tabs.replace(id, tab);
        self.save_on_state_change(id, cx);
        self.open_diff_on_request(id, window, cx);
        if let SavedTab::Settings { page } = &saved {
            self.show_settings_page(settings_page_named(page), cx);
        }
        self.tab_shown(window, cx);
    }

    pub(super) fn loaded_tab(&mut self, saved: &SavedTab, window: &mut Window, cx: &mut Context<Self>) -> Option<Tab> {
        let tab = match saved {
            SavedTab::File { path, panel } => file_tab(path, panel.clone(), cx),
            SavedTab::Scratch { project, name, panel } => {
                let scratch = ScratchRef { project: ProjectId::parse(project).ok()?, name: name.clone() };
                let tab = self.scratch_tab_of(&scratch, cx).ok()?;
                tab.restore(panel.clone(), cx);
                Tab::Scratch(tab)
            }
            SavedTab::Terminal { .. } | SavedTab::WorktreeOverview { .. } => return None,
            SavedTab::ProjectOverview { project, section } => {
                let id = ProjectId::parse(project).ok()?;
                let git = self.project_views(&id, window, cx);
                let section = ProjectSection::from_label(section).filter(|section| git.is_some() || *section != ProjectSection::Git);
                Tab::ProjectOverview(ProjectOverview { section: section.unwrap_or_default(), ..ProjectOverview::new(id, git) })
            }
            SavedTab::GitLog { project, view } => {
                let tab = self.git_log_tab_of(&ProjectId::parse(project).ok()?, window, cx)?;
                tab.body.update(cx, |body, cx| body.show_named(view, cx));
                Tab::GitLog(tab)
            }
            SavedTab::Commit { project, sha, title, panel } => {
                let id = ProjectId::parse(project).ok()?;
                let dir = self.project(&id)?.root.clone();
                let tab = CommitTab::reopen(id, dir, Sha::parse(sha)?, title.clone(), cx);
                tab.restore(panel.clone(), cx);
                Tab::Commit(tab)
            }
            SavedTab::Diff { checkout, request, panel } => {
                let checkout = self.listed_checkout(checkout)?;
                let request = DiffRequest::from_saved(request)?;
                // a diff that has no changes any more is not shown again
                let file = request.load().ok()??;
                let label = self.checkout_label(&checkout);
                let tab = DiffTab::open(DiffTabSpec { checkout, label, request, file }, cx);
                tab.restore(panel.clone(), cx);
                Tab::Diff(tab)
            }
            SavedTab::Settings { .. } => {
                let view = SettingsView::new(cx);
                Tab::Settings(view)
            }
            SavedTab::Hub => Tab::Hub(Box::new(self.new_hub(HubState::default(), None, window, cx))),
        };
        Some(tab)
    }

    /// The checkout at `path` while its project lists it.
    fn listed_checkout(&self, path: &Path) -> Option<CheckoutRef> {
        self.checkout_of_path(path).filter(|checkout| checkout.path == path)
    }

    /// The open tabs to save, in order, with the shown one; an IVE that does not own the profile keeps the
    /// owner's.
    pub(super) fn saved_tabs(&self, cx: &App) -> Option<SavedTabs> {
        self.tab_ownership.tabs_to_save(|| tabs_to_save(&self.tabs, |tab| tab.saved(self, cx)))
    }
}

/// The tabs of `tabs` to save, each reopened by `saved`. A temporary tab is not saved; when it is shown, the tab
/// closing it would show is saved as the shown one.
fn tabs_to_save<T>(tabs: &Tabs<T>, saved: impl Fn(&T) -> Option<SavedTab>) -> SavedTabs {
    let shown = tabs.settled_active();
    saved_tab_list(tabs.iter().map(|(id, tab)| (Some(id) == shown, saved(tab).filter(|_| !tabs.is_temporary(id)))))
}

/// The tabs to save, from each tab in order: whether it is shown, and what reopens it (`None` for a tab that is
/// not saved, such as a failed one). The shown tab is remembered only when it is saved itself. Tabs past the
/// 200th are left out, since a longer list would not load.
fn saved_tab_list(tabs: impl Iterator<Item = (bool, Option<SavedTab>)>) -> SavedTabs {
    let mut saved = SavedTabs::default();
    for (shown, tab) in tabs {
        let Some(tab) = tab.filter(|_| saved.tabs.len() < MAX_TABS) else {
            continue;
        };
        if shown {
            saved.active = Some(saved.tabs.len());
        }
        saved.tabs.push(tab);
    }
    saved
}

/// Whose tabs `state.toml` keeps: the IVE that owns the profile's terminals. A second IVE in the same profile
/// (say, `ive FILE` while IVE runs) neither restores them nor saves its own over them.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum TabOwnership {
    Owner,
    /// the tabs the file held at start, written back unchanged
    Guest(Option<SavedTabs>),
}

impl TabOwnership {
    fn tabs_to_save(&self, own: impl FnOnce() -> SavedTabs) -> Option<SavedTabs> {
        match self {
            Self::Owner => Some(own()),
            Self::Guest(saved) => saved.clone(),
        }
    }
}

fn file_tab(path: &Path, panel: PanelState, cx: &mut App) -> Tab {
    match Document::open_stamped(path) {
        Ok(opened) => {
            let file = FileTab::new(opened, cx);
            file.cast(Cast::Restore(panel), cx);
            Tab::Document(file)
        }
        Err(error) => Tab::Failed { title: project::folder_name(path), error: error.to_string().into() },
    }
}

/// Appearance, Scripts or Third-Party Software by its name; Appearance for a name no page has.
fn settings_page_named(name: &str) -> SettingsPage {
    if name == "Scripts" {
        return SettingsPage::Scripts;
    }
    if name == THIRD_PARTY_TITLE {
        return SettingsPage::ThirdParty;
    }
    SettingsPage::Appearance
}

/// What is open at start, which decides the saved tabs that come back.
struct Open<'a> {
    /// ids of the hosted sessions adopted
    sessions: &'a HashSet<String>,
    /// open projects: id and root
    projects: Vec<(String, PathBuf)>,
}

impl Open<'_> {
    /// A tab comes back when what it shows is still there: its terminal's session, its project, a checkout inside
    /// an open project. Files and Settings always come back; a file that is gone shows why when it loads.
    fn keeps(&self, tab: &SavedTab) -> bool {
        let project_open = |id: &str| self.projects.iter().any(|(open, _)| open == id);
        let inside_project = |path: &Path| self.projects.iter().any(|(_, root)| path.starts_with(root));
        match tab {
            SavedTab::Terminal { session } => self.sessions.contains(session),
            SavedTab::Scratch { project, .. } | SavedTab::ProjectOverview { project, .. } | SavedTab::GitLog { project, .. } | SavedTab::Commit { project, .. } => {
                project_open(project)
            }
            SavedTab::Diff { checkout, .. } => inside_project(checkout),
            SavedTab::File { .. } | SavedTab::Settings { .. } | SavedTab::Hub => true,
            // the worktree overview is gone: its git and processes are in the tree
            SavedTab::WorktreeOverview { .. } => false,
        }
    }
}

/// The saved tabs `keep` accepts, in order. The shown one stays shown; when it is dropped, the nearest kept tab on
/// its left is shown, else the first.
fn kept_tabs(saved: SavedTabs, keep: impl Fn(&SavedTab) -> bool) -> SavedTabs {
    let mut kept = SavedTabs::default();
    for (index, tab) in saved.tabs.into_iter().enumerate() {
        let shown = saved.active.is_some_and(|active| index <= active);
        if keep(&tab) {
            if shown {
                kept.active = Some(kept.tabs.len());
            }
            kept.tabs.push(tab);
        }
    }
    if kept.active.is_none() && !kept.tabs.is_empty() && saved.active.is_some() {
        kept.active = Some(0);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(page: &str) -> SavedTab {
        SavedTab::Settings { page: page.to_owned() }
    }

    fn file(name: &str) -> SavedTab {
        SavedTab::File { path: PathBuf::from("/p").join(name), panel: PanelState::default() }
    }

    fn terminal(session: &str) -> SavedTab {
        SavedTab::Terminal { session: session.to_owned() }
    }

    #[test]
    fn should_restore_the_scripts_page_by_its_name() {
        // given
        let name = "Scripts";

        // when
        let page = settings_page_named(name);

        // then
        assert_eq!(page, SettingsPage::Scripts);
    }

    #[test]
    fn should_restore_the_third_party_page_by_its_name() {
        // given
        let name = "Third-Party Software";

        // when
        let page = settings_page_named(name);

        // then
        assert_eq!(page, SettingsPage::ThirdParty);
    }

    #[test]
    fn should_show_the_nearest_kept_tab_left_of_a_dropped_active_one() {
        // given
        let saved = SavedTabs { tabs: vec![file("a"), terminal("gone"), file("c")], active: Some(1) };

        // when
        let kept = kept_tabs(saved, |tab| tab != &terminal("gone"));

        // then
        assert_eq!(kept, SavedTabs { tabs: vec![file("a"), file("c")], active: Some(0) });
    }

    #[test]
    fn should_show_the_first_tab_when_nothing_left_of_the_active_one_is_kept() {
        // given
        let saved = SavedTabs { tabs: vec![terminal("gone"), file("b"), file("c")], active: Some(0) };

        // when
        let kept = kept_tabs(saved, |tab| tab != &terminal("gone"));

        // then
        assert_eq!(kept.active, Some(0));
        assert_eq!(kept.tabs, [file("b"), file("c")]);
    }

    #[test]
    fn should_drop_terminal_tabs_whose_session_ended() {
        // given
        let sessions = HashSet::from(["live".to_owned()]);
        let open = Open { sessions: &sessions, projects: vec![("p".to_owned(), PathBuf::from("/p"))] };

        // when
        let keeps = [open.keeps(&terminal("live")), open.keeps(&terminal("ended"))];

        // then
        assert_eq!(keeps, [true, false]);
    }

    #[test]
    fn should_drop_tabs_of_projects_that_are_no_longer_open() {
        // given
        let sessions = HashSet::new();
        let open = Open { sessions: &sessions, projects: vec![("p".to_owned(), PathBuf::from("/p"))] };
        let overview = |project: &str| SavedTab::ProjectOverview { project: project.to_owned(), section: "Git".to_owned() };
        let worktree = |path: &str| SavedTab::WorktreeOverview { checkout: PathBuf::from(path), section: "Git".to_owned() };

        // when
        let keeps = [open.keeps(&overview("p")), open.keeps(&overview("q")), open.keeps(&worktree("/p/wt")), open.keeps(&worktree("/q"))];

        // then
        assert_eq!(keeps, [true, false, false, false]);
    }

    #[test]
    fn should_keep_files_outside_open_projects() {
        // given
        let sessions = HashSet::new();
        let open = Open { sessions: &sessions, projects: Vec::new() };

        // when
        let keeps = open.keeps(&file("notes.txt"));

        // then
        assert!(keeps);
    }

    #[test]
    fn should_save_tabs_in_order_with_the_shown_one() {
        // given
        let tabs = [(false, Some(settings("Appearance"))), (false, None), (true, Some(settings("Scripts")))];

        // when
        let saved = saved_tab_list(tabs.into_iter());

        // then
        assert_eq!(saved, SavedTabs { tabs: vec![settings("Appearance"), settings("Scripts")], active: Some(1) });
    }

    #[test]
    fn should_not_save_a_temporary_tab() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open(file("a.txt"));
        tabs.open(file("b.txt"));
        tabs.activate(a);
        tabs.open_temporary(settings("Appearance"));

        // when
        let saved = tabs_to_save(&tabs, |tab| Some(tab.clone()));

        // then
        assert_eq!(saved, SavedTabs { tabs: vec![file("a.txt"), file("b.txt")], active: Some(1) });
    }

    #[test]
    fn should_save_no_more_tabs_than_a_load_accepts() {
        // given
        let tabs = (0..MAX_TABS + 5).map(|index| (index == MAX_TABS + 2, Some(file(&format!("{index}.txt")))));

        // when
        let saved = saved_tab_list(tabs);

        // then
        assert_eq!(saved.tabs.len(), MAX_TABS);
        assert_eq!(saved.active, None);
        assert_eq!(saved.validate(), Ok(()));
    }

    #[test]
    fn should_save_its_own_tabs_when_it_owns_the_profile() {
        // given
        let ownership = TabOwnership::Owner;

        // when
        let saved = ownership.tabs_to_save(|| SavedTabs { tabs: vec![file("mine")], active: Some(0) });

        // then
        assert_eq!(saved, Some(SavedTabs { tabs: vec![file("mine")], active: Some(0) }));
    }

    #[test]
    fn should_keep_the_owners_tabs_when_another_ive_owns_the_profile() {
        // given
        let owners = SavedTabs { tabs: vec![file("theirs")], active: Some(0) };
        let ownership = TabOwnership::Guest(Some(owners.clone()));

        // when
        let saved = ownership.tabs_to_save(|| SavedTabs { tabs: vec![file("mine")], active: Some(0) });

        // then
        assert_eq!(saved, Some(owners));
    }

    #[test]
    fn should_save_no_shown_tab_when_the_shown_one_is_not_saved() {
        // given
        let tabs = [(false, Some(settings("Appearance"))), (true, None)];

        // when
        let saved = saved_tab_list(tabs.into_iter());

        // then
        assert_eq!(saved.active, None);
    }
}
