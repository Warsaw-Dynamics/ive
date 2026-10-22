//! A diff tab: one file's diff, frozen when it opened, with Reload.

use std::path::PathBuf;

use gpui::{App, Context, Entity, IntoElement, Render, Task, Window, div, prelude::*, rgb};

use super::git_ui::{layout_switch, mark_label, muted, toggle_button};
use crate::click_targets::target;
use crate::codeview_host::{DiffOrigin, WantedTexts, diff_panel, send_texts, serve};
use codeview::diff::Headers;
use codeview::panel::{Cast, Event, Panel, PanelState};
use crate::git::base::{ForkPoint, fork_point};
use crate::git::diff::{self, FileDiff, FilePaths, Sides, SinceRange, Uncommitted};
use crate::git::repo::GitError;
use crate::git::types::{GitPath, Mark, Sha};
use crate::git::worktrees::BaseBranch;
use crate::icons;
use crate::project::CheckoutRef;
use crate::saved_tabs::{SavedDiffRequest, SavedDiffSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffSource {
    Uncommitted,
    Since { base: BaseBranch, fork: Sha, uncommitted: Uncommitted },
    Commit { sha: Sha, subject: String },
}

/// Everything needed to read one file's diff again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRequest {
    pub dir: PathBuf,
    pub file: FilePaths,
    pub source: DiffSource,
}

impl DiffRequest {
    /// The request `state.toml` kept; `None` when a sha in it is not one.
    pub fn from_saved(saved: &SavedDiffRequest) -> Option<Self> {
        let source = match &saved.source {
            SavedDiffSource::Uncommitted => DiffSource::Uncommitted,
            SavedDiffSource::Since { base, reference, fork, with_uncommitted } => DiffSource::Since {
                base: BaseBranch { name: base.clone(), reference: reference.clone() },
                fork: Sha::parse(fork)?,
                uncommitted: if *with_uncommitted { Uncommitted::Included } else { Uncommitted::Excluded },
            },
            SavedDiffSource::Commit { sha, subject } => DiffSource::Commit { sha: Sha::parse(sha)?, subject: subject.clone() },
        };
        let file = FilePaths { path: GitPath::from_bytes(&saved.path), old_path: saved.old_path.as_deref().map(GitPath::from_bytes) };
        Some(Self { dir: saved.dir.clone(), file, source })
    }

    /// What `state.toml` keeps of this request.
    pub fn saved(&self) -> SavedDiffRequest {
        let source = match &self.source {
            DiffSource::Uncommitted => SavedDiffSource::Uncommitted,
            DiffSource::Since { base, fork, uncommitted } => SavedDiffSource::Since {
                base: base.name.clone(),
                reference: base.reference.clone(),
                fork: fork.as_str().to_owned(),
                with_uncommitted: *uncommitted == Uncommitted::Included,
            },
            DiffSource::Commit { sha, subject } => SavedDiffSource::Commit { sha: sha.as_str().to_owned(), subject: subject.clone() },
        };
        SavedDiffRequest {
            dir: self.dir.clone(),
            path: self.file.path.as_bytes().to_vec(),
            old_path: self.file.old_path.as_ref().map(|path| path.as_bytes().to_vec()),
            source,
        }
    }

    /// `None` when the file has no changes any more.
    pub fn load(&self) -> Result<Option<FileDiff>, GitError> {
        match &self.source {
            DiffSource::Uncommitted => diff::uncommitted(&self.dir, &self.file),
            DiffSource::Since { fork, uncommitted, .. } => {
                diff::since_file(&self.dir, &SinceRange { fork: fork.clone(), uncommitted: *uncommitted }, &self.file)
            }
            DiffSource::Commit { sha, .. } => diff::commit_file(&self.dir, sha, &self.file),
        }
    }

    /// Where its texts are read, as the request stands now.
    pub fn origin(&self) -> DiffOrigin {
        let sides = match &self.source {
            DiffSource::Uncommitted => Sides::Uncommitted,
            DiffSource::Since { fork, uncommitted, .. } => Sides::Since(SinceRange { fork: fork.clone(), uncommitted: *uncommitted }),
            DiffSource::Commit { sha, .. } => Sides::Commit(sha.clone()),
        };
        DiffOrigin { dir: self.dir.clone(), sides }
    }

    /// A diff tab's Reload: the request as it stands now (a Since diff starts from where the branch leaves its base
    /// now, which moves when the base is merged in) and its diff.
    pub fn reload(&self) -> Result<(DiffRequest, Option<FileDiff>), String> {
        let request = self.with_current_fork()?;
        let file = request.load().map_err(|error| error.to_string())?;
        Ok((request, file))
    }

    /// The same file from the same source, whatever fork point either was read from.
    pub fn same_diff(&self, other: &DiffRequest) -> bool {
        let same_source = match (&self.source, &other.source) {
            (DiffSource::Since { base, uncommitted, .. }, DiffSource::Since { base: other_base, uncommitted: other_uncommitted, .. }) => {
                base == other_base && uncommitted == other_uncommitted
            }
            (source, other_source) => source == other_source,
        };
        same_source && self.dir == other.dir && self.file == other.file
    }

    fn with_current_fork(&self) -> Result<DiffRequest, String> {
        let DiffSource::Since { base, uncommitted, .. } = &self.source else {
            return Ok(self.clone());
        };
        match fork_point(&self.dir, base, None).map_err(|error| error.to_string())? {
            ForkPoint::At { fork, .. } => Ok(DiffRequest { source: DiffSource::Since { base: base.clone(), fork, uncommitted: *uncommitted }, ..self.clone() }),
            ForkPoint::Missing(reason) => Err(format!("no base branch: {reason}")),
        }
    }
}

/// `uncommitted`; `dev … GEN-817-ive + uncommitted`; `commit 4371d9d · Fix the thing`.
pub fn source_text(source: &DiffSource, label: &str) -> String {
    match source {
        DiffSource::Uncommitted => "uncommitted".to_owned(),
        DiffSource::Since { base, uncommitted: Uncommitted::Excluded, .. } => format!("{} … {label}", base.name),
        DiffSource::Since { base, uncommitted: Uncommitted::Included, .. } => format!("{} … {label} + uncommitted", base.name),
        DiffSource::Commit { sha, subject } => format!("commit {} · {subject}", sha.short()),
    }
}

/// What opening a diff tab takes: where the diff comes from and the diff as shown now.
#[derive(Debug, Clone)]
pub struct DiffTabSpec {
    pub checkout: CheckoutRef,
    /// the checkout's label when the tab opened
    pub label: String,
    pub request: DiffRequest,
    pub file: FileDiff,
}

pub struct DiffTab {
    pub checkout: CheckoutRef,
    pub request: DiffRequest,
    pub title: String,
    pub tooltip: String,
    pub body: Entity<DiffTabBody>,
}

impl DiffTab {
    pub fn open(spec: DiffTabSpec, cx: &mut App) -> Self {
        let source = source_text(&spec.request.source, &spec.label);
        let tooltip = format!("{} — {} ({source})", spec.label, spec.request.file.path.display());
        let title = spec.request.file.path.file_name();
        let body = cx.new(|cx| DiffTabBody::new(&spec, cx));
        Self { checkout: spec.checkout, request: spec.request, title, tooltip, body }
    }

    pub fn panel_state(&self, cx: &App) -> PanelState {
        self.body.read(cx).panel.read(cx).state()
    }

    /// Scrolls the diff back to where a saved tab had it.
    pub fn restore(&self, state: PanelState, cx: &mut App) {
        let panel = self.body.read(cx).panel.clone();
        panel.update(cx, |panel, cx| panel.handle(Cast::Restore(state), cx));
    }
}

enum BodyState {
    Shown,
    Reloading,
    /// the file has no changes any more
    Gone,
    Failed(String),
}

pub struct DiffTabBody {
    request: DiffRequest,
    source: String,
    mark: Mark,
    panel: Entity<Panel>,
    state: BodyState,
    _reload: Option<Task<()>>,
}

impl DiffTabBody {
    fn new(spec: &DiffTabSpec, cx: &mut Context<Self>) -> Self {
        let panel = diff_panel(vec![spec.file.clone()], Headers::Hidden, cx);
        serve(&panel, spec.request.file.path.display(), cx);
        // the texts as the request stands when they are asked for: a reload may move a Since diff's fork point
        cx.subscribe(&panel, |body: &mut Self, panel, event, cx| {
            if let Event::NeedTexts(request) = event {
                send_texts(&panel, WantedTexts { request: *request, origin: body.request.origin() }, cx);
            }
        })
        .detach();
        let source = source_text(&spec.request.source, &spec.label);
        Self { request: spec.request.clone(), source, mark: spec.file.mark, panel, state: BodyState::Shown, _reload: None }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.state = BodyState::Reloading;
        let request = self.request.clone();
        self._reload = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { request.reload() }).await;
            this.update(cx, |body, cx| body.reloaded(result, cx)).ok();
        }));
        cx.notify();
    }

    fn reloaded(&mut self, result: Result<(DiffRequest, Option<FileDiff>), String>, cx: &mut Context<Self>) {
        let files = match result {
            Ok((request, Some(file))) => {
                self.request = request;
                self.mark = file.mark;
                self.state = BodyState::Shown;
                vec![file]
            }
            Ok((request, None)) => {
                self.request = request;
                self.state = BodyState::Gone;
                Vec::new()
            }
            Err(message) => {
                eprintln!("ive: {message}");
                self.state = BodyState::Failed(message);
                return cx.notify();
            }
        };
        self.panel.update(cx, |panel, cx| panel.handle(Cast::SetFiles(files), cx));
        cx.notify();
    }
}

impl Render for DiffTabBody {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        let reload = target("reload-diff".to_owned(), cx.listener(|body, _event, _window, cx| body.reload(cx)), cx);
        let status = match &self.state {
            BodyState::Shown => None,
            BodyState::Reloading => Some(muted("reloading…", theme)),
            BodyState::Gone => Some(muted(format!("No changes to {} any more.", self.request.file.path.display()), theme)),
            BodyState::Failed(message) => Some(div().text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.error)).child(message.clone())),
        };
        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(theme.sizes.git.diff_header_gap.px())
            .px(theme.sizes.git.diff_header_padding_x.px())
            .py(theme.sizes.git.diff_header_padding_y.px())
            .bg(rgb(theme.sidebar))
            .border_b(theme.sizes.git.diff_header_border.px())
            .border_color(rgb(theme.border))
            .child(mark_label(self.mark, theme, cx))
            .child(div().flex_none().font_weight(theme.sizes.git.diff_header_weight.font_weight()).text_size(theme.sizes.git.text_size.px()).child(self.request.file.path.display()))
            .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(self.source.clone()))
            .children(status)
            .child(layout_switch(&self.panel, theme, cx))
            .child(toggle_button("reload-diff", icons::HISTORY, "Reload", theme).on_click(reload));
        div().flex().flex_col().flex_1().min_h_0().child(header).child(self.panel.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;
    use crate::git::types::{GitPath, Sha};

    fn file(text: &str) -> FilePaths {
        FilePaths { path: GitPath::from_bytes(text.as_bytes()), old_path: None }
    }

    fn branch(name: &str) -> BaseBranch {
        BaseBranch { name: name.to_owned(), reference: format!("refs/heads/{name}") }
    }

    #[test]
    fn should_describe_where_a_diff_comes_from() {
        // given
        let fork = Sha::of(git2::Oid::ZERO_SHA1);
        let sources = [
            DiffSource::Uncommitted,
            DiffSource::Since { base: branch("dev"), fork: fork.clone(), uncommitted: Uncommitted::Excluded },
            DiffSource::Since { base: branch("dev"), fork: fork.clone(), uncommitted: Uncommitted::Included },
            DiffSource::Commit { sha: fork, subject: "Fix it".to_owned() },
        ];

        // when
        let texts: Vec<_> = sources.iter().map(|source| source_text(source, "GEN-817-ive")).collect();

        // then
        assert_eq!(texts, ["uncommitted", "dev … GEN-817-ive", "dev … GEN-817-ive + uncommitted", "commit 0000000 · Fix it"]);
    }

    #[test]
    fn should_reload_each_source_and_report_a_file_without_changes() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "1\n");
        let fork = Sha::of(repo.commit("base"));
        repo.write("a.txt", "2\n");
        let commit = Sha::of(repo.commit("change"));
        repo.write("a.txt", "3\n");
        let request = |source| DiffRequest { dir: repo.path().to_path_buf(), file: file("a.txt"), source };
        let sources = [
            DiffSource::Uncommitted,
            DiffSource::Since { base: branch("main"), fork, uncommitted: Uncommitted::Included },
            DiffSource::Commit { sha: commit, subject: "change".to_owned() },
        ];

        // when
        let found: Vec<bool> = sources.into_iter().map(|source| request(source).load().expect("load").is_some()).collect();
        repo.write("a.txt", "2\n");
        let gone = request(DiffSource::Uncommitted).load().expect("load");

        // then
        assert_eq!((found, gone), (vec![true, true, true], None));
    }

    #[test]
    fn should_start_a_reloaded_since_diff_from_where_the_branch_leaves_its_base_now() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "1\n");
        let opened_at = Sha::of(repo.commit("base"));
        repo.branch("topic");
        repo.write("b.txt", "on main\n");
        let merged = repo.commit("on main");
        repo.checkout("topic");
        repo.write("a.txt", "2\n");
        repo.commit("on topic");
        repo.write("b.txt", "on main\n");
        repo.merge("merge main", merged);
        let since = DiffSource::Since { base: branch("main"), fork: opened_at, uncommitted: Uncommitted::Excluded };
        let stale = DiffRequest { dir: repo.path().to_path_buf(), file: file("a.txt"), source: since };

        // when
        let (request, diff) = stale.reload().expect("reload");

        // then
        let DiffSource::Since { fork, .. } = &request.source else { panic!("not a since diff") };
        assert_eq!((fork, diff.is_some(), request.same_diff(&stale)), (&Sha::of(merged), true, true));
    }
}
