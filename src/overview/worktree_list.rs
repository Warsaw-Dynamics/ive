//! The Worktrees section's state: its filter field, the branches without a worktree, paging, the selection, and a
//! worktree being added. The workspace draws it with the project's checkouts (`workspace/worktree_list.rs`).

use std::path::PathBuf;

use gpui::{App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Subscription, Task};

use super::git_event::GitEvent;
use super::project_git::ProjectContext;
use super::worktree_rows::{Group, GroupKind, RowId, Selection, resolved, selection_of};
use crate::git::refs::{self, BranchInfo, Refs, without_worktree};
use crate::git::repo::GitError;
use crate::git::worktree_add::add_worktree;
use crate::paging::Paging;
use crate::project::ProjectId;
use crate::text_input::{TextInput, TextInputEvent};

pub const HOOKS_NOTE: &str = "libgit2 checks out the files; it runs no git hooks (post-checkout) and ignores sparse checkout";
pub const PLACEHOLDER: &str = "Filter worktrees and branches…";
/// The filter field's click target.
const INPUT_TARGET: &str = "worktree-filter";

pub struct WorktreeList {
    context: ProjectContext,
    /// as last read; the next read reuses them while no ref moved
    refs: Option<Refs>,
    /// branches without a worktree, newest first
    branches: Vec<BranchInfo>,
    /// where new worktrees go, as `.ive.toml` says
    worktrees_folder: Option<String>,
    load_error: Option<String>,
    input: Entity<TextInput>,
    paging: Paging,
    selection: Option<Selection>,
    hidden_open: bool,
    /// the branch whose worktree is being added
    adding: Option<String>,
    /// why the last add failed
    error: Option<String>,
    _input_events: Subscription,
    _load: Option<Task<()>>,
    _add: Option<Task<()>>,
}

impl EventEmitter<GitEvent> for WorktreeList {}

impl WorktreeList {
    pub fn new(context: ProjectContext, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            let mut input = TextInput::new(cx);
            input.set_target(INPUT_TARGET);
            input.placeholder(PLACEHOLDER);
            input
        });
        let events = cx.subscribe(&input, |list, _input, event: &TextInputEvent, cx| {
            if let TextInputEvent::Changed(_) = event {
                list.filter_changed(cx);
            }
        });
        let mut list = Self {
            context,
            refs: None,
            branches: Vec::new(),
            worktrees_folder: None,
            load_error: None,
            input,
            paging: Paging::default(),
            selection: None,
            hidden_open: true,
            adding: None,
            error: None,
            _input_events: events,
            _load: None,
            _add: None,
        };
        list.load(cx);
        list
    }

    pub fn set_context(&mut self, context: ProjectContext) {
        self.context = context;
    }

    pub fn input(&self) -> &Entity<TextInput> {
        &self.input
    }

    pub fn query(&self, cx: &App) -> String {
        self.input.read(cx).text().to_owned()
    }

    pub fn branches(&self) -> &[BranchInfo] {
        &self.branches
    }

    pub fn branch(&self, shown: &str) -> Option<&BranchInfo> {
        self.branches.iter().find(|branch| branch.name.display() == shown)
    }

    pub fn worktrees_folder(&self) -> Option<&str> {
        self.worktrees_folder.as_deref()
    }

    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn adding(&self) -> Option<&str> {
        self.adding.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn hidden_open(&self) -> bool {
        self.hidden_open
    }

    /// How many matching branches the paging shows.
    pub fn branches_visible(&self) -> usize {
        self.paging.visible(usize::MAX)
    }

    /// The selected row in `groups`, where a row that left its group leaves the selection at its place.
    pub fn selected(&self, groups: &[Group]) -> Option<RowId> {
        self.selection.as_ref().and_then(|selection| resolved(selection, groups, self.hidden_open))
    }

    pub fn select(&mut self, row: Option<&RowId>, groups: &[Group], cx: &mut Context<Self>) {
        self.selection = row.and_then(|row| selection_of(groups, row));
        cx.notify();
    }

    pub fn clear_filter(&mut self, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_text("", cx));
        self.filter_changed(cx);
    }

    pub fn toggle_hidden_group(&mut self, cx: &mut Context<Self>) {
        self.hidden_open = !self.hidden_open;
        cx.notify();
    }

    pub fn show_more(&mut self, cx: &mut Context<Self>) {
        self.paging.more();
        cx.notify();
    }

    pub fn show_all(&mut self, cx: &mut Context<Self>) {
        self.paging.all();
        cx.notify();
    }

    /// The hub's `#` row of a branch without a worktree: the branch typed in the filter, its row selected.
    pub fn start_with(&mut self, branch: &str, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_text(branch, cx));
        self.filter_changed(cx);
        // the filter holds the branch's name: its row is the first branch
        self.selection = Some(Selection { row: RowId::Branch(branch.to_owned()), kind: GroupKind::Branches, at: 0 });
    }

    fn filter_changed(&mut self, cx: &mut Context<Self>) {
        self.paging = Paging::default();
        self.selection = None;
        self.error = None;
        cx.notify();
    }

    /// Reads the branches again; the Worktrees section asks every few seconds, so unchanged refs are not read twice.
    pub fn load(&mut self, cx: &mut Context<Self>) {
        let (dir, previous) = (self.context.main_dir.clone(), self.refs.clone());
        self._load = Some(cx.spawn(async move |this, cx| {
            let read = cx.background_executor().spawn(async move {
                let folder = crate::ive_toml::worktrees_folder(&dir).ok().map(|folder| folder.display().to_string());
                (refs::read(&dir, previous.as_ref()), folder)
            });
            let (result, folder) = read.await;
            this.update(cx, |list, cx| list.loaded(result, folder, cx)).ok();
        }));
    }

    fn loaded(&mut self, result: Result<Refs, GitError>, folder: Option<String>, cx: &mut Context<Self>) {
        match result {
            Ok(refs) => {
                let mut branches = without_worktree(&refs.branches, &self.context.checked_out());
                branches.sort_by_key(|branch| std::cmp::Reverse(branch.when.seconds));
                self.branches = branches;
                self.refs = Some(refs);
                self.load_error = None;
            }
            Err(error) => {
                eprintln!("ive: {error}");
                self.load_error = Some(error.to_string());
            }
        }
        self.worktrees_folder = folder;
        cx.notify();
    }

    /// Adds `branch`'s worktree, one at a time; `keep_filter` leaves the typed filter (Space), else it is cleared.
    pub fn add(&mut self, branch: BranchInfo, keep_filter: bool, cx: &mut Context<Self>) {
        if self.adding.is_some() {
            return;
        }
        if !keep_filter {
            self.clear_filter(cx);
        }
        self.adding = Some(branch.name.display());
        self.error = None;
        let (main_dir, project) = (self.context.main_dir.clone(), self.context.project.clone());
        self._add = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { add_worktree(&main_dir, &branch.name) }).await;
            this.update(cx, |list, cx| list.added(project, result, cx)).ok();
        }));
        cx.notify();
    }

    fn added(&mut self, project: ProjectId, result: Result<PathBuf, GitError>, cx: &mut Context<Self>) {
        self.adding = None;
        match result {
            Ok(path) => {
                cx.emit(GitEvent::WorktreeAdded { project, path });
                self.load(cx);
            }
            Err(error) => {
                eprintln!("ive: {error}");
                self.error = Some(error.to_string());
            }
        }
        cx.notify();
    }
}

impl Focusable for WorktreeList {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
