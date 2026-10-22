//! What the last close or quit waits for: the saves of its tabs, then, for edits that could not be saved, the
//! user's "Close without saving". Going on working (editing a waiting tab, showing another tab) forgets it, and a
//! new close or quit replaces it, so a forgotten one never closes or quits later.

use gpui::EntityId;

use crate::project::ProjectId;
use crate::tabs::TabId;

/// One close or quit; only the latest is live, and only until it is forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket(u64);

/// Whether what the user did forgot the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forgot {
    Nothing,
    Request,
}

/// What closing a tab without saving lets go on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterDiscard {
    /// the tab was not one the live request waits for
    Nothing,
    /// the projects whose close waits may close once their tabs are gone
    CloseProjects,
    Quit(Ticket),
}

#[derive(Default)]
pub struct Waits {
    /// the panels the workspace listens to for edits and discards
    pub(super) watched: Vec<EntityId>,
    /// tickets handed out and requests forgotten; the request is live while its ticket is this one
    issued: u64,
    request: Request,
}

#[derive(Default)]
enum Request {
    #[default]
    Idle,
    /// closing `tabs`, and once they are gone, `projects`
    Close { tabs: Vec<TabId>, projects: Vec<ProjectId> },
    /// a quit; `tabs` hold the edits it stopped at
    Quit { tabs: Vec<TabId> },
}

impl Waits {
    /// A close of `tabs`: one of tabs the live close waits for goes on with that close, any other replaces it.
    pub fn close(&mut self, tabs: &[TabId]) -> Ticket {
        let continues = match &self.request {
            Request::Close { tabs: waiting, .. } => !tabs.is_empty() && tabs.iter().all(|tab| waiting.contains(tab)),
            Request::Idle | Request::Quit { .. } => false,
        };
        if continues {
            return Ticket(self.issued);
        }
        self.replace(Request::Close { tabs: tabs.to_vec(), projects: Vec::new() })
    }

    /// A quit; it replaces any other request.
    pub fn quit(&mut self) -> Ticket {
        self.replace(Request::Quit { tabs: Vec::new() })
    }

    pub fn is_live(&self, ticket: Ticket) -> bool {
        ticket.0 == self.issued && !matches!(self.request, Request::Idle)
    }

    /// Project `project` closes with close `ticket`, once no tab of it is left; nothing when `ticket` is not live.
    pub fn close_project(&mut self, ticket: Ticket, project: ProjectId) {
        let live = self.is_live(ticket);
        if let (true, Request::Close { projects, .. }) = (live, &mut self.request)
            && !projects.contains(&project)
        {
            projects.push(project);
        }
    }

    /// Quit `ticket` stopped at the edits of `tabs`; nothing when it is not live.
    pub fn quit_stopped(&mut self, ticket: Ticket, tabs: Vec<TabId>) {
        let live = self.is_live(ticket);
        if let (true, Request::Quit { tabs: waiting }) = (live, &mut self.request) {
            *waiting = tabs;
        }
    }

    /// Whether the live request waits for tab `tab`.
    pub fn waits_on(&self, tab: TabId) -> bool {
        match &self.request {
            Request::Close { tabs, .. } | Request::Quit { tabs } => tabs.contains(&tab),
            Request::Idle => false,
        }
    }

    /// The user showed tab `tab`: a tab the request does not wait for forgets it.
    pub fn activated(&mut self, tab: TabId) -> Forgot {
        if self.waits_on(tab) {
            return Forgot::Nothing;
        }
        self.forget()
    }

    /// The user edited tab `tab`: a tab the request waits for forgets it.
    pub fn edited(&mut self, tab: TabId) -> Forgot {
        if self.waits_on(tab) {
            return self.forget();
        }
        Forgot::Nothing
    }

    /// The user closed tab `tab` without saving.
    pub fn discarded(&self, tab: TabId) -> AfterDiscard {
        match &self.request {
            Request::Quit { tabs } if tabs.contains(&tab) => AfterDiscard::Quit(Ticket(self.issued)),
            Request::Close { tabs, .. } if tabs.contains(&tab) => AfterDiscard::CloseProjects,
            Request::Idle | Request::Close { .. } | Request::Quit { .. } => AfterDiscard::Nothing,
        }
    }

    /// The projects the live close closes once their tabs are gone.
    pub fn projects(&self) -> Vec<ProjectId> {
        match &self.request {
            Request::Close { projects, .. } => projects.clone(),
            Request::Idle | Request::Quit { .. } => Vec::new(),
        }
    }

    /// Project `project` closed: the close waits for it no more.
    pub fn project_closed(&mut self, project: &ProjectId) {
        if let Request::Close { projects, .. } = &mut self.request {
            projects.retain(|waiting| waiting != project);
        }
    }

    fn forget(&mut self) -> Forgot {
        if let Request::Idle = self.request {
            return Forgot::Nothing;
        }
        self.issued += 1;
        self.request = Request::Idle;
        Forgot::Request
    }

    fn replace(&mut self, request: Request) -> Ticket {
        self.issued += 1;
        self.request = request;
        Ticket(self.issued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tabs::Tabs;

    /// `count` distinct tab ids.
    fn tab_ids(count: usize) -> Vec<TabId> {
        let mut tabs = Tabs::default();
        (0..count).map(|_| tabs.open(())).collect()
    }

    #[test]
    fn should_forget_a_stopped_quit_once_another_tab_is_activated() {
        // given
        let [unsaved, other] = tab_ids(2)[..] else { unreachable!() };
        let mut waits = Waits::default();
        let quit = waits.quit();
        waits.quit_stopped(quit, vec![unsaved]);
        let shown_itself = waits.activated(unsaved);

        // when
        let forgot = waits.activated(other);

        // then
        assert_eq!((shown_itself, forgot, waits.is_live(quit), waits.discarded(unsaved)), (Forgot::Nothing, Forgot::Request, false, AfterDiscard::Nothing));
    }

    #[test]
    fn should_not_quit_when_a_later_unrelated_tab_is_closed_without_saving() {
        // given
        let [unsaved, later] = tab_ids(2)[..] else { unreachable!() };
        let mut waits = Waits::default();
        let quit = waits.quit();
        waits.quit_stopped(quit, vec![unsaved]);

        // when
        let close = waits.close(&[later]);

        // then
        assert_eq!((waits.is_live(quit), waits.is_live(close), waits.discarded(later)), (false, true, AfterDiscard::CloseProjects));
    }

    #[test]
    fn should_close_a_waiting_project_when_its_last_unsaved_tab_is_closed() {
        // given
        let [first, last] = tab_ids(2)[..] else { unreachable!() };
        let project = ProjectId::new("p");
        let mut waits = Waits::default();
        let close = waits.close(&[first, last]);
        waits.close_project(close, project.clone());

        // when
        let again = waits.close(&[last]);

        // then
        assert_eq!((again, waits.is_live(close), waits.projects()), (close, true, vec![project]));
    }

    #[test]
    fn should_forget_a_project_close_once_the_user_goes_on_working() {
        // given
        let [unsaved, other] = tab_ids(2)[..] else { unreachable!() };
        let mut waits = Waits::default();
        let edited = waits.close(&[unsaved]);
        waits.close_project(edited, ProjectId::new("p"));

        // when
        let unrelated_edit = waits.edited(other);
        let edit = waits.edited(unsaved);
        let after_edit = waits.projects();
        let shown = waits.close(&[unsaved]);
        waits.close_project(shown, ProjectId::new("p"));
        let activation = waits.activated(other);

        // then
        assert_eq!((unrelated_edit, edit, after_edit, waits.is_live(edited)), (Forgot::Nothing, Forgot::Request, Vec::new(), false));
        assert_eq!((activation, waits.projects(), waits.is_live(shown)), (Forgot::Request, Vec::new(), false));
    }
}
