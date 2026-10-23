//! The project's Git log tab: Log · Stashes · Worktrees · Branches.

use std::ops::Range;
use std::path::PathBuf;

use gpui::{
    AnyElement, Context, Div, Entity, EventEmitter, IntoElement, ListSizingBehavior, Render, Stateful, Subscription, Task, Window, div,
    prelude::*, rgb, uniform_list,
};

use super::git_event::{CommitLink, GitEvent};
use super::git_ui::{Picked, error_with_retry, list_row, muted, segment, toggle_button};
use super::project_git::{CheckoutLine, LogRow, ProjectContext, bordered, log_row, stash_row};
use crate::age::Clock;
use crate::click_targets::target;
use crate::git::log::{self, CommitInfo, CommitRange, Page, Stash};
use crate::git::refs::{self, BranchInfo, Refs};
use crate::git::repo::GitError;
use crate::icons;
use crate::project::ProjectId;
use crate::scrollbar::{ListScroll, ScrollState, vertical_scrollbar};
use crate::theme::Theme;
use crate::workspace::icon;

const LOG_PAGE: usize = 100;

pub struct GitLogTab {
    pub project: ProjectId,
    pub of: LogOf,
    pub body: Entity<GitLogView>,
    _events: Subscription,
}

impl GitLogTab {
    pub fn new(project: ProjectId, of: LogOf, body: Entity<GitLogView>, events: Subscription) -> Self {
        Self { project, of, body, _events: events }
    }
}

/// Whose commits a Git log lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogOf {
    /// every branch and tag of the project
    Project,
    /// those of checkout `dir`'s HEAD that changed `path`, relative to it
    Path { dir: PathBuf, path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct GitLogData {
    pub log: Vec<CommitInfo>,
    /// branches and the names on each commit; the next load reuses them while no ref moved
    pub refs: Refs,
    pub stashes: Vec<Stash>,
    /// branches without a worktree, newest first
    pub branches: Vec<BranchInfo>,
}

pub fn load_log(context: &ProjectContext, of: &LogOf, limit: usize, previous: Option<&Refs>) -> Result<GitLogData, GitError> {
    let dir = &context.main_dir;
    let refs = refs::read(dir, previous)?;
    let branches = refs::without_worktree(&refs.branches, &context.checked_out());
    let page = Page { skip: 0, limit };
    let log = match of {
        LogOf::Project => log::commits(dir, &CommitRange::AllRefs, page)?,
        LogOf::Path { dir, path } => log::path_history(dir, path, page)?,
    };
    Ok(GitLogData { log, refs, stashes: log::stashes(dir)?, branches })
}

/// A full page means the walk may have more.
pub fn has_more(data: &GitLogData, limit: usize) -> bool {
    data.log.len() == limit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Log,
    Stashes,
    Worktrees,
    Branches,
}

impl Section {
    const ALL: [Self; 4] = [Self::Log, Self::Stashes, Self::Worktrees, Self::Branches];

    fn name(self) -> &'static str {
        match self {
            Self::Log => "Log",
            Self::Stashes => "Stashes",
            Self::Worktrees => "Worktrees",
            Self::Branches => "Branches",
        }
    }
}

pub struct GitLogView {
    context: ProjectContext,
    of: LogOf,
    section: Section,
    limit: usize,
    data: Option<Result<GitLogData, String>>,
    loading: bool,
    rerun: bool,
    /// git changed since the last load; read again when next shown
    stale: bool,
    /// the Log draws only the rows in view: it grows by 100 per "Load 100 more"
    log_scroll: ListScroll,
    log_scrollbar: ScrollState,
    /// Stashes, Worktrees and Branches, which also draw only the rows in view
    rows_scroll: ListScroll,
    rows_scrollbar: ScrollState,
    _load: Option<Task<()>>,
}

impl EventEmitter<GitEvent> for GitLogView {}

impl GitLogView {
    /// Shows the view named `name`; an unknown name leaves the shown one.
    pub fn show_named(&mut self, name: &str, cx: &mut Context<Self>) {
        if let Some(section) = Section::ALL.into_iter().find(|section| section.name() == name) {
            self.select(section, cx);
        }
    }

    /// The shown view's name, as on its button.
    pub fn section_name(&self) -> &'static str {
        self.section.name()
    }

    /// Reads git once it is first shown.
    pub fn new(context: ProjectContext, of: LogOf) -> Self {
        let log_scroll = ListScroll::new();
        let log_scrollbar = ScrollState::for_list(&log_scroll);
        let rows_scroll = ListScroll::new();
        let rows_scrollbar = ScrollState::for_list(&rows_scroll);
        Self {
            context,
            of,
            section: Section::Log,
            limit: LOG_PAGE,
            data: None,
            loading: false,
            rerun: false,
            stale: true,
            log_scroll,
            log_scrollbar,
            rows_scroll,
            rows_scrollbar,
            _load: None,
        }
    }

    /// Checkouts added or gone change which branches have no worktree.
    pub fn set_context(&mut self, context: ProjectContext) {
        if context != self.context {
            self.context = context;
            self.stale = true;
        }
    }

    /// Git changed: the next [`Self::load_if_stale`] reads it again.
    pub fn invalidate(&mut self) {
        self.stale = true;
    }

    pub fn load_if_stale(&mut self, cx: &mut Context<Self>) {
        if self.stale {
            self.load(cx);
        }
    }

    /// Reads the log again, keeping the last one on screen (and its scroll) meanwhile.
    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        if self.loading {
            self.rerun = true;
            return;
        }
        self.loading = true;
        let (context, of, limit) = (self.context.clone(), self.of.clone(), self.limit);
        let previous = match &self.data {
            Some(Ok(data)) => Some(data.refs.clone()),
            _ => None,
        };
        self._load = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { load_log(&context, &of, limit, previous.as_ref()) }).await;
            this.update(cx, |view, cx| view.loaded(result, cx)).ok();
        }));
        cx.notify();
    }

    fn loaded(&mut self, result: Result<GitLogData, GitError>, cx: &mut Context<Self>) {
        self.loading = false;
        self.data = Some(result.map_err(|error| {
            eprintln!("ive: {error}");
            error.to_string()
        }));
        if std::mem::take(&mut self.rerun) {
            self.load(cx);
        }
        cx.notify();
    }

    fn load_more(&mut self, cx: &mut Context<Self>) {
        self.limit += LOG_PAGE;
        self.load(cx);
    }

    fn select(&mut self, section: Section, cx: &mut Context<Self>) {
        if section != self.section {
            // the sections share one list: another section starts at its top
            self.rows_scroll.set_offset(gpui::Point::default());
        }
        self.section = section;
        cx.notify();
    }

    fn render_section(&self, data: &GitLogData, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let empty = match self.section {
            Section::Log => return self.render_log(theme, cx).into_any_element(),
            Section::Stashes if data.stashes.is_empty() => "No stashes.",
            Section::Branches if data.branches.is_empty() => "Every branch has a worktree.",
            Section::Stashes | Section::Worktrees | Section::Branches => return self.render_rows(self.count(data, self.section), theme, cx).into_any_element(),
        };
        div().p(theme.sizes.git.message_padding.px()).child(muted(empty, theme)).into_any_element()
    }

    /// Stashes, worktrees and branches: a repository can have thousands, so only the rows in view are built; a
    /// short list's box ends under its last row.
    fn render_rows(&self, count: usize, theme: Theme, cx: &mut Context<Self>) -> Div {
        let rows = uniform_list(("git-log-section-rows", cx.entity_id().as_u64()), count, cx.processor(|view: &mut GitLogView, range, _window, cx| view.render_section_rows(range, cx)))
            .with_sizing_behavior(ListSizingBehavior::Infer)
            .track_scroll(&self.rows_scroll.list_handle())
            .w_full();
        bordered(theme).relative().flex_shrink_1().min_h_0().mb(theme.sizes.git.log_padding_bottom.px()).child(rows).child(vertical_scrollbar(&self.rows_scrollbar, &crate::theme::view_style(&theme)))
    }

    fn render_section_rows(&self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        let Some(Ok(data)) = &self.data else {
            return Vec::new();
        };
        let (theme, clock) = (crate::theme::tabs(cx), Clock::system());
        let row = |index: usize| match self.section {
            Section::Stashes => data.stashes.get(index).map(|stash| stash_row(index, stash, theme)),
            Section::Worktrees => self.context.checkouts.get(index).map(|checkout| worktree_row(index, checkout, theme)),
            Section::Branches => data.branches.get(index).map(|branch| branch_row(index, branch, &clock, theme)),
            Section::Log => None,
        };
        range.filter_map(row).map(|row| row.w_full()).collect()
    }

    fn count(&self, data: &GitLogData, section: Section) -> usize {
        match section {
            Section::Log => data.log.len(),
            Section::Stashes => data.stashes.len(),
            Section::Worktrees => self.context.checkouts.len(),
            Section::Branches => data.branches.len(),
        }
    }

    /// The Log fills the tab and scrolls itself; the last row loads 100 more while the walk may have more.
    fn render_log(&self, theme: Theme, cx: &mut Context<Self>) -> Div {
        let count = match &self.data {
            Some(Ok(data)) => data.log.len() + usize::from(has_more(data, self.limit)),
            _ => 0,
        };
        let rows = uniform_list(("git-log-rows", cx.entity_id().as_u64()), count, cx.processor(|view: &mut GitLogView, range, _window, cx| view.render_log_rows(range, cx)))
            .track_scroll(&self.log_scroll.list_handle())
            .size_full();
        bordered(theme).relative().flex_1().min_h_0().mb(theme.sizes.git.log_margin_bottom.px()).child(rows).child(vertical_scrollbar(&self.log_scrollbar, &crate::theme::view_style(&theme)))
    }

    fn render_log_rows(&self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        let Some(Ok(data)) = &self.data else {
            return Vec::new();
        };
        range.map(|index| self.render_log_row(index, data, cx).w_full()).collect()
    }

    fn render_log_row(&self, index: usize, data: &GitLogData, cx: &mut Context<Self>) -> Stateful<Div> {
        let Some(commit) = data.log.get(index) else {
            let theme = crate::theme::tabs(cx);
            let more = target("load-more".to_owned(), cx.listener(|view, _event, _window, cx| view.load_more(cx)), cx);
            let button = toggle_button("git-log-more", icons::HISTORY, format!("Load {LOG_PAGE} more"), theme).on_click(more);
            return div().id(("log-row", index)).flex().items_center().h(theme.sizes.git.row_height.px()).px(theme.sizes.git.log_more_padding_x.px()).child(button);
        };
        let event = GitEvent::OpenCommit(CommitLink { project: self.context.project.clone(), dir: self.context.main_dir.clone(), commit: commit.clone() });
        let open = cx.listener(move |_view, _event, _window, cx| cx.emit(event.clone()));
        log_row(LogRow { index, commit, names: data.refs.names.get(&commit.sha) }, open, cx)
    }

    fn section_label(&self, section: Section) -> String {
        match &self.data {
            Some(Ok(data)) => format!("{} ({})", section.name(), self.count(data, section)),
            _ => section.name().to_owned(),
        }
    }
}

impl Render for GitLogView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        let buttons: Vec<_> = Section::ALL
            .into_iter()
            .enumerate()
            .map(|(index, section)| {
                let select = target(format!("git-log:{}", section.name()), cx.listener(move |view, _event, _window, cx| view.select(section, cx)), cx);
                segment(("git-log-section", index), self.section_label(section), Picked::from(section == self.section), theme).on_click(select)
            })
            .collect();
        let body = match &self.data {
            None => muted("Reading git…", theme).into_any_element(),
            Some(Err(message)) => error_with_retry(message.clone(), cx.listener(|view, _event, _window, cx| view.load(cx)), cx).into_any_element(),
            Some(Ok(data)) => self.render_section(data, theme, cx),
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .gap(theme.sizes.git.log_toolbar_gap.px())
            .px(theme.sizes.git.log_toolbar_padding_x.px())
            .pt(theme.sizes.git.log_toolbar_padding_top.px())
            .child(div().flex().flex_none().items_center().gap(theme.sizes.git.log_buttons_gap.px()).children(buttons))
            .child(body)
    }
}

fn worktree_row(index: usize, checkout: &CheckoutLine, theme: Theme) -> Stateful<Div> {
    list_row(("git-log-worktree", index), Picked::No, theme)
        .cursor_default()
        .child(icon(theme, icons::BRANCH))
        .child(div().flex_none().child(checkout.label.clone()))
        .child(div().flex_none().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(checkout.branch.clone().unwrap_or_else(|| "detached".to_owned())))
        .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(checkout.path.display().to_string()))
}

fn branch_row(index: usize, branch: &BranchInfo, clock: &Clock, theme: Theme) -> Stateful<Div> {
    list_row(("git-log-branch", index), Picked::No, theme)
        .cursor_default()
        .child(icon(theme, icons::BRANCH))
        .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().child(branch.name.display()))
        .child(div().flex_none().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(clock.age(branch.when.seconds)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;
    use crate::overview::project_git::CheckoutLine;

    #[test]
    fn should_page_the_log_of_all_refs_and_list_branches_newest_first() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        repo.commit("c0");
        repo.branch("old");
        repo.write("f.txt", "1\n");
        repo.commit("c1");
        repo.branch("new");
        repo.write("f.txt", "2\n");
        repo.commit("c2");
        let context = ProjectContext {
            project: ProjectId::new("test"),
            name: "test".to_owned(),
            main_dir: repo.path().to_path_buf(),
            checkouts: vec![CheckoutLine { label: "main".to_owned(), branch: Some("main".to_owned()), path: repo.path().to_path_buf() }],
        };

        // when
        let data = load_log(&context, &LogOf::Project, 2, None).expect("load");

        // then
        let branches: Vec<_> = data.branches.iter().map(|branch| branch.name.display()).collect();
        assert_eq!((data.log.len(), has_more(&data, 2), branches), (2, true, vec!["new".to_owned(), "old".to_owned()]));
    }
}
