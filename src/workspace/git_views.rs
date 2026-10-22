//! The workspace side of the project overview's git views: the entities it owns, when they load, and what their
//! events ask for.

use std::collections::HashMap;

use gpui::{AppContext, Context, Entity, Subscription, Window};

use super::{AfterOpen, Workspace};
use crate::editor::Tab;

use crate::overview::commit_tab::CommitTab;
use crate::overview::diff_tab::{DiffTab, DiffTabSpec};
use crate::overview::git_event::{CommitLink, GitEvent};
use crate::overview::git_log_tab::{GitLogTab, GitLogView, LogOf};
use crate::project::CheckoutRef;
use std::path::PathBuf;
use crate::overview::worktree_list::WorktreeList;
use crate::overview::project_git::{ProjectContext, ProjectGit};
use crate::overview::{ProjectGitViews, ProjectOverview, ProjectSection};
use crate::project::{ProjectId, ProjectKind};
use crate::tabs::TabId;

/// The git views of every project shown so far; dropped when their project closes.
#[derive(Default)]
pub(super) struct GitViews {
    projects: HashMap<ProjectId, ProjectEntry>,
}

struct ProjectEntry {
    views: ProjectGitViews,
    _events: [Subscription; 3],
}

impl Workspace {
    /// The Git section and the New worktree card of project `id`, made once and kept; `None` for a plain folder.
    pub fn project_views(&mut self, id: &ProjectId, window: &mut Window, cx: &mut Context<Self>) -> Option<ProjectGitViews> {
        if let Some(entry) = self.git.projects.get(id) {
            return Some(entry.views.clone());
        }
        let project = self.project(id).filter(|project| project.kind == ProjectKind::Git)?;
        let context = ProjectContext::of(project);
        let card_context = context.clone();
        let section = cx.new(|_| ProjectGit::new(context));
        let list = cx.new(|cx| WorktreeList::new(card_context, cx));
        let events = [
            cx.subscribe_in(&section, window, |workspace, _git, event: &GitEvent, window, cx| workspace.on_git_event(event, window, cx)),
            cx.subscribe_in(&list, window, |workspace, _list, event: &GitEvent, window, cx| workspace.on_git_event(event, window, cx)),
            // the workspace draws the list: it draws again when the list changes
            cx.observe(&list, |_workspace, _list, cx| cx.notify()),
        ];
        let views = ProjectGitViews { section, list };
        self.git.projects.insert(id.clone(), ProjectEntry { views: views.clone(), _events: events });
        Some(views)
    }

    /// A tab came on screen, or an overview switched sections: what it shows of git is read again (the Git log only
    /// when git changed meanwhile, a commit tab only the first time).
    pub(super) fn load_git_section(&mut self, tab: TabId, cx: &mut Context<Self>) {
        match self.tabs.get(tab) {
            Some(Tab::ProjectOverview(ProjectOverview { section: ProjectSection::Git, git: Some(views), .. })) => {
                views.section.update(cx, |git, cx| git.load(cx));
            }
            Some(Tab::ProjectOverview(ProjectOverview { section: ProjectSection::Worktrees, git: Some(views), .. })) => {
                views.list.update(cx, |list, cx| list.load(cx));
            }
            Some(Tab::GitLog(log)) => log.body.update(cx, |view, cx| view.load_if_stale(cx)),
            Some(Tab::Commit(commit)) => commit.body.update(cx, |view, cx| view.load_once(cx)),
            _ => {}
        }
    }

    /// After a refresh of project `id` (a change on disk, a window activation): the views take the new checkouts,
    /// and those on screen read git again.
    pub(super) fn refresh_git_views(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        if let Some(context) = self.project(id).map(ProjectContext::of) {
            self.set_project_context(id, &context, cx);
        }
        let on_screen: Vec<TabId> = self.tabs.on_screen().filter(|(_, tab)| tab.project(self).as_ref() == Some(id)).map(|(tab, _)| tab).collect();
        for tab in on_screen {
            self.load_git_section(tab, cx);
        }
    }

    fn set_project_context(&mut self, id: &ProjectId, context: &ProjectContext, cx: &mut Context<Self>) {
        if let Some(entry) = self.git.projects.get(id) {
            entry.views.section.update(cx, |git, _| git.set_context(context.clone()));
            entry.views.list.update(cx, |list, _| list.set_context(context.clone()));
        }
        for (_, tab) in self.tabs.iter() {
            if let Tab::GitLog(log) = tab
                && log.project == *id
            {
                log.body.update(cx, |view, _| view.set_context(context.clone()));
            }
        }
    }

    /// Something changed in project `id`'s git dir (a commit, a branch, a fetch): its Git log tab reads git again,
    /// now when shown, else when next shown. Commit tabs stay: a commit does not change.
    pub(super) fn git_changed(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        for (_, tab) in self.tabs.iter() {
            if let Tab::GitLog(log) = tab
                && log.project == *id
            {
                log.body.update(cx, |view, _| view.invalidate());
            }
        }
        let on_screen: Vec<TabId> = self.tabs.on_screen().map(|(tab, _)| tab).collect();
        for tab in on_screen {
            self.load_git_section(tab, cx);
        }
    }

    /// The Worktrees list of git project `id`, once its views were made.
    pub(super) fn worktree_list_of(&self, id: &ProjectId) -> Option<Entity<WorktreeList>> {
        self.git.projects.get(id).map(|entry| entry.views.list.clone())
    }

    /// Project `id` closed: its git views go.
    pub(super) fn forget_git_views(&mut self, id: &ProjectId) {
        self.git.projects.remove(id);
    }

    /// Does what a git view asked for.
    pub fn on_git_event(&mut self, event: &GitEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            GitEvent::OpenCommit(link) => self.open_commit_tab(link, window, cx),
            GitEvent::OpenGitLog(project) => self.open_git_log_tab(project, window, cx),
            GitEvent::WorktreeAdded { project, path } => {
                // the new checkout shows collapsed under its project; the refresh brings the views the new checkout
                self.open_folder(path.clone(), AfterOpen::ListCheckout, window, cx);
                self.refresh_project(project, cx);
            }
        }
    }

    /// Shows the tab of this diff when one is open, else opens one; returns the tab.
    pub(super) fn open_diff_tab(&mut self, spec: &DiffTabSpec, window: &mut Window, cx: &mut Context<Self>) -> TabId {
        let open = self.tabs.find(|tab| matches!(tab, Tab::Diff(diff) if diff.checkout == spec.checkout && diff.request.same_diff(&spec.request)));
        if let Some(id) = open {
            self.activate_tab(id, window, cx);
            return id;
        }
        let tab = DiffTab::open(spec.clone(), cx);
        self.open_tab(Tab::Diff(tab), window, cx)
    }

    fn open_commit_tab(&mut self, link: &CommitLink, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.tabs.find(|tab| tab.is_commit(&link.project, link.commit.sha.as_str()));
        if let Some(id) = open {
            return self.activate_tab(id, window, cx);
        }
        let tab = CommitTab::open(link, cx);
        self.open_tab(Tab::Commit(tab), window, cx);
    }

    /// One Git log tab per project.
    pub(super) fn open_git_log_tab(&mut self, project: &ProjectId, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.tabs.find(|tab| tab.is_git_log(project));
        if let Some(id) = open {
            return self.activate_tab(id, window, cx);
        }
        if let Some(tab) = self.git_log_tab_of(project, LogOf::Project, window, cx) {
            self.open_tab(Tab::GitLog(tab), window, cx);
        }
    }

    /// Show History: a Git log of the commits of `checkout` that changed `path`, relative to it; one tab per path.
    pub(super) fn open_path_history(&mut self, checkout: &CheckoutRef, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let of = LogOf::Path { dir: checkout.path.clone(), path };
        let open = self.tabs.find(|tab| matches!(tab, Tab::GitLog(log) if log.of == of));
        if let Some(id) = open {
            return self.activate_tab(id, window, cx);
        }
        if let Some(tab) = self.git_log_tab_of(&checkout.project, of, window, cx) {
            self.open_tab(Tab::GitLog(tab), window, cx);
        }
    }

    /// A Git log tab for `project`, not shown yet; `None` when the project is not open.
    pub(super) fn git_log_tab_of(&mut self, project: &ProjectId, of: LogOf, window: &mut Window, cx: &mut Context<Self>) -> Option<GitLogTab> {
        let context = self.project(project).map(ProjectContext::of)?;
        let body = cx.new(|_| GitLogView::new(context, of.clone()));
        let events = cx.subscribe_in(&body, window, |workspace, _view, event: &GitEvent, window, cx| workspace.on_git_event(event, window, cx));
        Some(GitLogTab::new(project.clone(), of, body, events))
    }
}
