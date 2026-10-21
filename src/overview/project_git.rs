//! The project overview's Git section: a summary with Open Git, the main checkout's last commits, the stashes.

use std::path::PathBuf;

use gpui::{App, ClickEvent, Context, Div, EventEmitter, IntoElement, Render, Stateful, Task, Window, div, prelude::*, rgb};

use super::git_event::{CommitLink, GitEvent};
use super::git_ui::{Picked, counted, error_with_retry, group_title, list_row, muted, tag, toggle_button};
use crate::age::Clock;
use crate::click_targets::target;
use crate::git::log::{self, CommitInfo, CommitRange, Page, Stash};
use crate::git::refs::{self, BranchInfo, Refs};
use crate::git::repo::GitError;
use crate::icons;
use crate::project::{Project, ProjectId};
use crate::theme::{self, Theme};
use crate::workspace::icon;

const LOG_SIZE: usize = 10;

/// What the project's git views need from the project model; replaced after each refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    pub project: ProjectId,
    pub name: String,
    pub main_dir: PathBuf,
    pub checkouts: Vec<CheckoutLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutLine {
    pub label: String,
    pub branch: Option<String>,
    pub path: PathBuf,
}

impl ProjectContext {
    pub fn of(project: &Project) -> Self {
        let checkouts = project
            .checkouts
            .iter()
            .map(|checkout| CheckoutLine { label: checkout.label.clone(), branch: checkout.branch.clone(), path: checkout.path.clone() })
            .collect();
        Self { project: project.id.clone(), name: project.name.clone(), main_dir: project.main_checkout().path.clone(), checkouts }
    }

    /// Branches some checkout has checked out.
    pub fn checked_out(&self) -> Vec<String> {
        self.checkouts.iter().filter_map(|checkout| checkout.branch.clone()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectGitData {
    pub log: Vec<CommitInfo>,
    /// branches and the names on each commit; the next load reuses them while no ref moved
    pub refs: Refs,
    pub stashes: Vec<Stash>,
    pub free_branches: Vec<BranchInfo>,
}

pub fn load_project(context: &ProjectContext, previous: Option<&Refs>) -> Result<ProjectGitData, GitError> {
    let dir = &context.main_dir;
    let refs = refs::read(dir, previous)?;
    let free_branches = refs::without_worktree(&refs.branches, &context.checked_out());
    Ok(ProjectGitData { log: log::commits(dir, &CommitRange::Head, Page { skip: 0, limit: LOG_SIZE })?, refs, stashes: log::stashes(dir)?, free_branches })
}

/// `10 recent commits · 1 stash · 3 branches without a worktree`.
pub fn summary(data: &ProjectGitData) -> String {
    format!(
        "{} · {} · {}",
        counted(data.log.len(), "recent commit", "recent commits"),
        counted(data.stashes.len(), "stash", "stashes"),
        counted(data.free_branches.len(), "branch without a worktree", "branches without a worktree")
    )
}

/// What one log row shows: the commit and the branch and tag names pointing at it.
pub struct LogRow<'a> {
    pub index: usize,
    pub commit: &'a CommitInfo,
    pub names: Option<&'a Vec<String>>,
}

/// A log row: sha, subject, names, author, age; `open` (target `log:<short sha>`) opens the commit's tab.
pub fn log_row(row: LogRow<'_>, open: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> Stateful<Div> {
    let theme = crate::theme::tabs(cx);
    let names = row.names.into_iter().flatten().map(|name| tag(format!("⎇ {name}"), theme.text, theme));
    let open = target(format!("log:{}", row.commit.sha.short()), open, cx);
    list_row(("log-row", row.index), Picked::No, theme)
        .child(icon(theme, icons::COMMIT))
        .child(div().flex_none().font_family(theme::mono_font(cx)).text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.accent)).child(row.commit.sha.short().to_owned()))
        .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().child(row.commit.subject.clone()))
        .children(names)
        .child(div().flex_none().max_w(theme.sizes.git.author_max_width.px()).overflow_hidden().text_ellipsis().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(row.commit.author.clone()))
        .child(div().flex_none().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(Clock::system().age(row.commit.when.seconds)))
        .on_click(open)
}

pub fn stash_row(index: usize, stash: &Stash, theme: Theme) -> Stateful<Div> {
    div()
        .id(("stash-row", index))
        .flex()
        .items_center()
        .gap(theme.sizes.git.row_gap.px())
        .h(theme.sizes.git.row_height.px())
        .px(theme.sizes.git.row_padding_x.px())
        .border_b(theme.sizes.git.row_border.px())
        .border_color(rgb(theme.border))
        .text_size(theme.sizes.git.text_size.px())
        .child(icon(theme, icons::HISTORY))
        .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().child(stash.message.clone()))
        .child(div().flex_none().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(Clock::system().age(stash.when.seconds)))
}

pub fn bordered(theme: Theme) -> Div {
    div().flex().flex_col().rounded(theme.sizes.git.box_radius.px()).border(theme.sizes.git.box_border.px()).border_color(rgb(theme.border)).overflow_hidden()
}

pub struct ProjectGit {
    context: ProjectContext,
    data: Option<Result<ProjectGitData, String>>,
    loading: bool,
    rerun: bool,
    _load: Option<Task<()>>,
}

impl EventEmitter<GitEvent> for ProjectGit {}

impl ProjectGit {
    /// Nothing is read until [`Self::load`]: the workspace loads when the section is shown.
    pub fn new(context: ProjectContext) -> Self {
        Self { context, data: None, loading: false, rerun: false, _load: None }
    }

    pub fn set_context(&mut self, context: ProjectContext) {
        self.context = context;
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            self.rerun = true;
            return;
        }
        self.loading = true;
        let context = self.context.clone();
        let previous = match &self.data {
            Some(Ok(data)) => Some(data.refs.clone()),
            _ => None,
        };
        self._load = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { load_project(&context, previous.as_ref()) }).await;
            this.update(cx, |git, cx| git.loaded(result, cx)).ok();
        }));
        cx.notify();
    }

    fn loaded(&mut self, result: Result<ProjectGitData, GitError>, cx: &mut Context<Self>) {
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

    fn render_data(&self, data: &ProjectGitData, theme: Theme, cx: &mut Context<Self>) -> Div {
        let project = self.context.project.clone();
        let open_git = target("open-git".to_owned(), cx.listener(move |_git, _event, _window, cx| cx.emit(GitEvent::OpenGitLog(project.clone()))), cx);
        let summary_row = div()
            .flex()
            .items_center()
            .gap(theme.sizes.git.summary_gap.px())
            .px(theme.sizes.git.summary_padding_x.px())
            .py(theme.sizes.git.summary_padding_y.px())
            .rounded(theme.sizes.git.box_radius.px())
            .border(theme.sizes.git.box_border.px())
            .border_color(rgb(theme.border))
            .child(icon(theme, icons::HISTORY))
            .child(div().flex_1().text_size(theme.sizes.git.text_size.px()).child(summary(data)))
            .child(toggle_button("open-git", icons::BRANCH, "Open Git", theme).on_click(open_git));
        let log_rows: Vec<_> = data
            .log
            .iter()
            .enumerate()
            .map(|(index, commit)| {
                let event = GitEvent::OpenCommit(CommitLink { project: self.context.project.clone(), dir: self.context.main_dir.clone(), commit: commit.clone() });
                let open = cx.listener(move |_git, _event, _window, cx| cx.emit(event.clone()));
                log_row(LogRow { index, commit, names: data.refs.names.get(&commit.sha) }, open, cx)
            })
            .collect();
        let section = div().flex().flex_col().gap(theme.sizes.git.section_gap.px()).child(summary_row).child(group_title("Log", None, theme)).child(bordered(theme).children(log_rows));
        if data.stashes.is_empty() {
            return section;
        }
        let stashes = data.stashes.iter().enumerate().map(|(index, stash)| stash_row(index, stash, theme));
        section.child(group_title("Stashes", Some("shared by all worktrees"), theme)).child(bordered(theme).children(stashes))
    }
}

impl Render for ProjectGit {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        let body = match &self.data {
            None => muted("Reading git…", theme).into_any_element(),
            Some(Err(message)) => error_with_retry(message.clone(), cx.listener(|git, _event, _window, cx| git.load(cx)), cx).into_any_element(),
            Some(Ok(data)) => self.render_data(data, theme, cx).into_any_element(),
        };
        div().flex().flex_col().font_weight(theme.sizes.git.body_weight.font_weight()).child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::age::When;
    use crate::git::testing::TestRepo;
    use crate::git::types::Sha;

    fn context(repo: &TestRepo, branches: &[&str]) -> ProjectContext {
        ProjectContext {
            project: ProjectId::new("test"),
            name: "test".to_owned(),
            main_dir: repo.path().to_path_buf(),
            checkouts: branches.iter().map(|branch| CheckoutLine { label: branch.to_string(), branch: Some(branch.to_string()), path: repo.path().to_path_buf() }).collect(),
        }
    }

    #[test]
    fn should_load_the_last_ten_commits_stashes_and_free_branches() {
        // given
        let mut repo = TestRepo::init();
        for n in 0..12 {
            repo.write("f.txt", &format!("{n}\n"));
            repo.commit(&format!("c{n}"));
        }
        repo.branch("topic");
        repo.write("f.txt", "dirty\n");
        repo.stash("wip");

        // when
        let data = load_project(&context(&repo, &["main"]), None).expect("load");

        // then
        let free: Vec<_> = data.free_branches.iter().map(|branch| branch.name.display()).collect();
        assert_eq!((data.log.len(), data.log[0].subject.as_str(), data.stashes.len(), free), (10, "c11", 1, vec!["topic".to_owned()]));
    }

    #[test]
    fn should_summarize_with_singular_and_plural() {
        // given
        let commit = CommitInfo { sha: Sha::of(git2::Oid::ZERO_SHA1), subject: "s".to_owned(), author: "a".to_owned(), when: When { seconds: 0, offset_minutes: 0 }, parents: Vec::new() };
        let data = ProjectGitData { log: vec![commit], refs: Refs::default(), stashes: Vec::new(), free_branches: Vec::new() };

        // when
        let text = summary(&data);

        // then
        assert_eq!(text, "1 recent commit · 0 stashes · 0 branches without a worktree");
    }
}
