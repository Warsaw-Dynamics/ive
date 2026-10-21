//! Moments parts 2 and 3 hook into: each part adds its calls to these bodies.

use gpui::{Context, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::overview::ProjectSection;
use crate::project::ProjectId;
use crate::tabs::TabId;

impl Workspace {
    /// An overview's selected section came on screen: its tab was shown, or the section was selected in it.
    pub fn section_shown(&mut self, tab: TabId, window: &mut Window, cx: &mut Context<Self>) {
        self.load_git_section(tab, cx);
        let project = match self.tabs.get(tab) {
            Some(Tab::ProjectOverview(overview)) if overview.section == ProjectSection::Scratches => {
                return self.scratches_shown(tab, window, cx);
            }
            Some(Tab::ProjectOverview(overview)) => overview.project.clone(),
            _ => return,
        };
        // the Worktrees list's filter has the keyboard while the section shows
        if self.tabs.active_id() == Some(tab) {
            self.focus_active_tab(window, cx);
        }
        // a section switch away from Scratches unregisters its folder
        self.sync_watches();
        self.refresh_project(&project, cx);
    }

    /// A refresh result replaced project `id`'s checkouts (not called after a failed refresh).
    pub fn project_refreshed(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        self.refresh_git_views(id, cx);
        self.index_project(id, cx);
    }

    /// Project `id` closed: its tabs are closed and its processes ended.
    pub fn project_closed(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        self.forget_git_views(id);
        self.forget_closed_checkouts(cx);
    }
}
