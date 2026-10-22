//! A diff tab: one file's diff, frozen when it opened, with Reload. A diff of uncommitted changes edits its new
//! side, the file in the work tree, saved as a file tab saves it; its rows are always the diff of the very text
//! edited: as read to edit, or as saved.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, IntoElement, Render, Subscription, Task, Window, div, prelude::*, rgb};

use super::git_ui::{layout_switch, mark_label, muted, toggle_button};
use crate::click_targets::target;
use crate::codeview_host::{DiffOrigin, WantedTexts, diff_panel, send_texts, serve};
use crate::document::Document;
use crate::file_tab::{EditedFile, FileEvent, Shown, Shows};
use crate::file_write;
use codeview::diff::{DiffBody, Headers};
use codeview::document::{Reread, Stamp};
use codeview::panel::{Cast, EditBase, EditPolicy, Editable, Event, Panel, PanelState, TextPurpose, TextsRequest};
use codeview::syntax::MAX_HIGHLIGHT_BYTES;
use crate::git::base::{ForkPoint, fork_point};
use crate::git::diff::{self, EditedDiff, FileDiff, FilePaths, Sides, SinceRange, Uncommitted};
use crate::git::status::relative_path;
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

    /// The panel when the tab edits the new side: a diff of uncommitted changes.
    pub fn editing_panel(&self, cx: &App) -> Option<Entity<Panel>> {
        let body = self.body.read(cx);
        match body.new_side {
            NewSide::ReadOnly | NewSide::Unreadable(_) => None,
            NewSide::Unread | NewSide::Edited { .. } => Some(body.panel.clone()),
        }
    }

    /// The work tree's file the tab edits (a diff of uncommitted changes), which the workspace watches for it.
    pub fn edited_path(&self) -> Option<PathBuf> {
        match self.request.source {
            DiffSource::Uncommitted => Some(self.request.dir.join(relative_path(self.request.file.path.as_bytes()))),
            DiffSource::Since { .. } | DiffSource::Commit { .. } => None,
        }
    }

    /// What a re-read of the edited file compares it with, once the tab edits it.
    pub fn edited_stamp(&self, cx: &App) -> Option<Option<Stamp>> {
        match &self.body.read(cx).new_side {
            NewSide::Edited { file, .. } => Some(file.read(cx).stamp()),
            NewSide::ReadOnly | NewSide::Unread | NewSide::Unreadable(_) => None,
        }
    }

    /// A change of the edited file on disk: with nothing unsaved, the tab takes the file as it is now to edit, rows
    /// and all; with unsaved edits, the rows keep showing them, and the next save meets the change.
    pub fn take_reread(&self, reread: Reread, cx: &mut App) {
        let file = match &self.body.read(cx).new_side {
            NewSide::Edited { file, .. } => file.clone(),
            NewSide::ReadOnly | NewSide::Unread | NewSide::Unreadable(_) => return,
        };
        file.update(cx, |file, cx| file.take_reread(reread, cx));
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

/// The new side of the diff, which only a diff of uncommitted changes edits.
enum NewSide {
    ReadOnly,
    /// editable once its text is read to edit
    Unread,
    /// its text could not be read to edit, for this reason: read-only until Reload tries again
    Unreadable(String),
    /// the file behind it, read to edit; each save diffs the text saved again
    Edited { file: Entity<EditedFile>, _changes: Subscription },
}

/// What the panel saves with until it knows the file's size.
const FIRST_POLICY: EditPolicy = EditPolicy { save_delay: file_write::SMALL_FILE_SAVE_DELAY, limit: None };

/// A diff's new side read to edit: the file in the work tree with its stamp, HEAD's text of it to colour, and the
/// diff of the two, made from the very text read.
struct EditRead {
    opened: (Document, Option<Stamp>),
    head: EditedDiff,
}

pub struct DiffTabBody {
    request: DiffRequest,
    source: String,
    mark: Mark,
    panel: Entity<Panel>,
    state: BodyState,
    new_side: NewSide,
    _reload: Option<Task<()>>,
}

impl DiffTabBody {
    fn new(spec: &DiffTabSpec, cx: &mut Context<Self>) -> Self {
        let panel = diff_panel(vec![spec.file.clone()], Headers::Hidden, cx);
        serve(&panel, spec.request.file.path.display(), cx);
        cx.subscribe(&panel, |body: &mut Self, _panel, event, cx| {
            if let Event::NeedTexts(request) = event {
                body.send_texts(*request, cx);
            }
        })
        .detach();
        let new_side = new_side_of(&spec.request.source);
        if let NewSide::Unread = new_side {
            panel.update(cx, |panel, cx| panel.handle(Cast::SetEditable(Editable::NewSide { policy: FIRST_POLICY }), cx));
        }
        let source = source_text(&spec.request.source, &spec.label);
        Self { request: spec.request.clone(), source, mark: spec.file.mark, panel, state: BodyState::Shown, new_side, _reload: None }
    }

    /// Answers an ask for the texts, as the request stands now (a reload may move a Since diff's fork point); the
    /// new side to edit comes from the work tree, with the stamp the next save is checked against.
    fn send_texts(&mut self, request: TextsRequest, cx: &mut Context<Self>) {
        let origin = self.request.origin();
        if request.purpose == TextPurpose::Colours {
            return send_texts(&self.panel, WantedTexts { request, origin }, cx);
        }
        let file = self.request.file.clone();
        let reading = cx.background_executor().spawn(async move { read_to_edit(&origin.dir, &file) });
        cx.spawn(async move |body, cx| {
            let read = reading.await;
            body.update(cx, |body, cx| body.take_texts(request.generation, read, cx)).ok();
        })
        .detach();
    }

    /// Answers ask `generation` with the new side read to edit, while the panel waits for it: the text becomes the
    /// edited file's base and the panel's together, with the diff of that very text; one that cannot be read leaves
    /// the diff read-only, saying why.
    fn take_texts(&mut self, generation: u64, read: Result<EditRead, String>, cx: &mut Context<Self>) {
        if self.panel.read(cx).waits_to_edit(generation, cx) {
            let base = match read {
                Ok(read) => self.take_base(read, cx),
                Err(reason) => self.unreadable(reason),
            };
            self.panel.update(cx, |panel, cx| panel.handle(Cast::SetEditBase { generation, base }, cx));
            cx.notify();
        }
    }

    fn unreadable(&mut self, reason: String) -> EditBase {
        eprintln!("ive: the diff of {} cannot be edited: {reason}", self.request.file.path.display());
        self.new_side = NewSide::Unreadable(reason);
        EditBase::Unavailable
    }

    /// The file read to edit becomes the base of the file behind the new side, and the panel's.
    fn take_base(&mut self, read: EditRead, cx: &mut Context<Self>) -> EditBase {
        let EditRead { opened, head } = read;
        let new = match &self.new_side {
            NewSide::Edited { file, .. } => file.update(cx, |file, cx| file.adopt(opened, cx)),
            NewSide::ReadOnly | NewSide::Unread | NewSide::Unreadable(_) => self.edit_file(opened, cx),
        };
        let diff = self.shown_diff(head.body);
        EditBase::Text { old: head.old, new, diff }
    }

    /// Edits the new side as `opened`; its base, for the panel.
    fn edit_file(&mut self, opened: (Document, Option<Stamp>), cx: &mut Context<Self>) -> Arc<Document> {
        let shown = Shown { panel: self.panel.clone(), shows: Shows::DiffSide };
        let file = cx.new(|cx| EditedFile::new(opened, shown, cx));
        let base = file.read(cx).base();
        let changes = cx.subscribe(&file, |body, _file, event: &FileEvent, cx| match event {
            FileEvent::Saved => body.rediff(cx),
            FileEvent::Gone => body.reload_diff(cx),
        });
        self.new_side = NewSide::Edited { file, _changes: changes };
        base
    }

    /// `body` as the file's diff, which the header follows.
    fn shown_diff(&mut self, body: DiffBody) -> FileDiff {
        let diff = FileDiff { path: self.request.file.path.clone(), old_path: self.request.file.old_path.clone(), mark: self.mark, body };
        self.state = state_of(&diff);
        diff
    }

    /// The edited file was saved: the diff of the text saved becomes the rows' (a diff of the work tree, which may
    /// hold a later write already, never numbers them).
    fn rediff(&mut self, cx: &mut Context<Self>) {
        let NewSide::Edited { file, .. } = &self.new_side else {
            return;
        };
        let (base, dir, paths) = (file.read(cx).base(), self.request.dir.clone(), self.request.file.clone());
        let saved = base.clone();
        let diffing = cx.background_executor().spawn(async move { diff::edited_uncommitted(&dir, &paths, saved.bytes()) });
        cx.spawn(async move |body, cx| {
            let diffed = diffing.await;
            body.update(cx, |body, cx| body.rediffed(base, diffed, cx)).ok();
        })
        .detach();
    }

    /// The diff of `base`, a text saved; the panel drops it if its edits apply to another text by now.
    fn rediffed(&mut self, base: Arc<Document>, diffed: Result<EditedDiff, GitError>, cx: &mut Context<Self>) {
        let diff = match diffed {
            Ok(edited) => self.shown_diff(edited.body),
            Err(error) => {
                self.state = BodyState::Failed(error.to_string());
                return cx.notify();
            }
        };
        self.panel.update(cx, |panel, cx| panel.handle(Cast::SetBaseDiff { base, diff }, cx));
        cx.notify();
    }

    /// Reload: an edited new side with nothing unsaved is read again to edit, rows and all (with unsaved edits the
    /// rows keep showing them); one that could not be read is tried again; any other diff is read again.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let panel = self.panel.read(cx);
        let edited = panel.edits_new_side(cx);
        let unsaved = panel.has_unsaved(cx);
        match (&self.new_side, edited) {
            // the rows keep showing the edits; the next save meets the change
            (NewSide::Edited { .. }, true) if unsaved => {}
            (NewSide::Edited { file, .. }, true) => file.clone().update(cx, |file, cx| file.read_again(cx)),
            (NewSide::Unreadable(_), _) => {
                self.new_side = NewSide::Unread;
                self.panel.update(cx, |panel, cx| panel.handle(Cast::SetEditable(Editable::NewSide { policy: FIRST_POLICY }), cx));
            }
            _ => self.reload_diff(cx),
        }
        cx.notify();
    }

    /// Reads the diff of the work tree again, for a diff the panel does not edit.
    fn reload_diff(&mut self, cx: &mut Context<Self>) {
        self.state = BodyState::Reloading;
        let request = self.request.clone();
        self._reload = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { request.reload() }).await;
            this.update(cx, |body, cx| body.reloaded(result, cx)).ok();
        }));
        cx.notify();
    }

    fn reloaded(&mut self, result: Result<(DiffRequest, Option<FileDiff>), String>, cx: &mut Context<Self>) {
        // taken to edit meanwhile: the rows show the diff of the text taken
        if self.panel.read(cx).edits_new_side(cx) {
            self.state = BodyState::Shown;
            return cx.notify();
        }
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
        let reload = target("reload-diff".to_owned(), cx.listener(|body: &mut Self, _event, _window, cx| body.reload(cx)), cx);
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
            .children(self.edit_note(cx).map(|note| div().text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.error)).child(note)))
            .child(layout_switch(&self.panel, theme, cx))
            .child(toggle_button("reload-diff", icons::HISTORY, "Reload", theme).on_click(reload));
        div().flex().flex_col().flex_1().min_h_0().child(header).child(self.panel.clone())
    }
}

impl DiffTabBody {
    /// Why the new side is read-only for now: another tab holds unsaved edits to the file, or it could not be read
    /// to edit.
    fn edit_note(&self, cx: &App) -> Option<gpui::SharedString> {
        match &self.new_side {
            NewSide::Edited { file, .. } => file.read(cx).lock_note(),
            NewSide::Unreadable(reason) => Some(format!("read-only: {reason}").into()),
            NewSide::ReadOnly | NewSide::Unread => None,
        }
    }

    /// The note in the header saying why the new side is read-only, if it is.
    #[cfg(test)]
    fn note(&self, cx: &App) -> Option<String> {
        self.edit_note(cx).map(|note| note.to_string())
    }
}

/// Shown, or gone when git finds no change any more.
fn state_of(diff: &FileDiff) -> BodyState {
    match &diff.body {
        DiffBody::Text { hunks, .. } if hunks.is_empty() => BodyState::Gone,
        DiffBody::Text { .. } | DiffBody::Binary | DiffBody::TooLarge => BodyState::Shown,
    }
}

/// Only a diff of uncommitted changes edits its new side: every other one shows commits, which stay as they are.
fn new_side_of(source: &DiffSource) -> NewSide {
    match source {
        DiffSource::Uncommitted => NewSide::Unread,
        DiffSource::Since { .. } | DiffSource::Commit { .. } => NewSide::ReadOnly,
    }
}

/// The work tree's file of `file` in the repository at `dir`, read to edit, with its diff against HEAD.
fn read_to_edit(dir: &Path, file: &FilePaths) -> Result<EditRead, String> {
    let path = diff::work_tree_path(dir, &file.path).map_err(|error| error.to_string())?;
    let opened = editable_file(&path)?;
    let head = diff::edited_uncommitted(dir, file, opened.0.bytes()).map_err(|error| error.to_string())?;
    match head.body {
        DiffBody::Text { .. } => Ok(EditRead { opened, head }),
        DiffBody::Binary | DiffBody::TooLarge => Err(format!("{} is binary or too large in HEAD", file.path.display())),
    }
}

/// The file at `path` with the stamp it had when read, when a diff edits it: a regular file, not binary, small enough
/// to colour (so held in memory, never mapped).
fn editable_file(path: &Path) -> Result<(Document, Option<Stamp>), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    match (metadata.is_file(), metadata.len()) {
        (true, length) if length <= MAX_HIGHLIGHT_BYTES as u64 => {}
        (true, length) => return Err(format!("{} is too large to edit in a diff ({length} bytes)", path.display())),
        (false, _) => return Err(format!("{} is not a regular file", path.display())),
    }
    let opened = Document::open_stamped(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    match opened.0.byte_len() <= MAX_HIGHLIGHT_BYTES {
        true => Ok(opened),
        false => Err(format!("{} is too large to edit in a diff ({} bytes)", path.display(), opened.0.byte_len())),
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

#[cfg(test)]
mod editing_tests {
    use std::time::Duration;

    use codeview::colors::{Overrides, ResolvedScheme, Scheme};
    use codeview::diff::{DiffBody, LineKind};
    use codeview::host::Command;
    use gpui::{TestAppContext, VisualTestContext, point, px, size};

    use super::*;
    use crate::git::testing::TestRepo;
    use crate::project::ProjectId;
    use crate::theme::{Appearance, Theme, view_style};

    /// A repository whose `a.txt` was committed as "one", "two", "three" and changed since: "two" became "TWO".
    fn changed_repo() -> (TestRepo, Sha) {
        let repo = TestRepo::init();
        repo.write("a.txt", "one\ntwo\nthree\n");
        let commit = Sha::of(repo.commit("three lines"));
        repo.write("a.txt", "one\nTWO\nthree\n");
        (repo, commit)
    }

    /// A diff tab of `a.txt` in `repo` from `source`.
    fn diff_tab(repo: &TestRepo, source: DiffSource, cx: &mut App) -> DiffTab {
        let file = FilePaths { path: GitPath::from_bytes(b"a.txt"), old_path: None };
        let request = DiffRequest { dir: repo.path().to_path_buf(), file, source };
        let file = request.load().expect("load").expect("a.txt changed");
        let checkout = CheckoutRef { project: ProjectId::new("p"), path: repo.path().to_path_buf() };
        DiffTab::open(DiffTabSpec { checkout, label: "main".to_owned(), request, file }, cx)
    }

    fn styled(cx: &mut TestAppContext) {
        cx.update(|cx| {
            cx.set_global(view_style(&Theme::of(Appearance::Dark)));
            cx.set_global(ResolvedScheme::resolve(Scheme::builtin(codeview::style::Appearance::Dark), &Overrides::new()));
        });
    }

    #[gpui::test]
    fn should_not_allow_edits_in_commit_and_since_diffs(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, commit) = changed_repo();
        let base = BaseBranch { name: "main".to_owned(), reference: "refs/heads/main".to_owned() };
        let since = DiffSource::Since { base, fork: commit.clone(), uncommitted: Uncommitted::Included };
        let sources = [DiffSource::Uncommitted, since, DiffSource::Commit { sha: commit, subject: "three lines".to_owned() }];

        // when
        let tabs: Vec<DiffTab> = sources.into_iter().map(|source| cx.update(|cx| diff_tab(&repo, source, cx))).collect();

        // then
        let editable = |tab: &DiffTab, cx: &App| tab.editing_panel(cx).is_some_and(|panel| panel.read(cx).is_editable_diff(cx));
        let found: Vec<bool> = tabs.iter().map(|tab| cx.read(|cx| editable(tab, cx))).collect();
        assert_eq!(found, [true, false, false]);
    }

    /// Draws `panel` in a window 20 rows high, so it asks for its texts, and lets the answers land.
    fn draw(panel: &Entity<Panel>, cx: &mut VisualTestContext) {
        let panel = panel.clone();
        cx.draw(point(px(0.), px(0.)), size(px(800.), px(400.)), |_, _| div().size_full().flex().flex_col().child(panel));
        cx.run_until_parked();
    }

    /// The lines of the new side the diff shows.
    fn new_lines(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Vec<String> {
        panel.read_with(cx, |panel, cx| {
            let DiffBody::Text { hunks, .. } = &panel.diff_file(0, cx).expect("a file").body else { panic!("no text") };
            hunks.iter().flat_map(|hunk| &hunk.lines).filter(|line| line.kind != LineKind::Removed).map(|line| line.text.clone()).collect()
        })
    }

    #[gpui::test]
    fn should_save_an_edit_of_an_uncommitted_diff_to_its_file_and_read_the_diff_again(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        let cx = cx.add_empty_window();
        draw(&panel, cx);

        // when
        panel.update(cx, |panel, cx| panel.handle(Cast::Type("x".to_owned()), cx));
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Flush), cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(10));
        cx.run_until_parked();

        // then
        assert_eq!(std::fs::read_to_string(repo.path().join("a.txt")).unwrap(), "one\nxTWO\nthree\n");
        assert_eq!(new_lines(&panel, cx), ["one", "xTWO", "three"]);
        assert!(panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx)));
    }

    #[gpui::test]
    fn should_read_the_diff_again_after_a_save(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        let cx = cx.add_empty_window();
        draw(&panel, cx);

        // when
        // the caret starts on "TWO": typing "two" over it undoes the change
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Delete(codeview::text_editor::Unit::Word)), cx));
        panel.update(cx, |panel, cx| panel.handle(Cast::Type("two".to_owned()), cx));
        let before_save = new_lines(&panel, cx);
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Flush), cx));
        cx.run_until_parked();

        // then
        assert_eq!(std::fs::read_to_string(repo.path().join("a.txt")).unwrap(), "one\ntwo\nthree\n");
        assert_eq!(before_save, ["one", "two", "three"]);
        // git finds no change any more: the diff has no lines left
        assert_eq!(new_lines(&panel, cx), Vec::<String>::new());
    }

    /// Types `text` at the caret and lets the save land.
    fn type_and_save(panel: &Entity<Panel>, text: &str, cx: &mut VisualTestContext) {
        panel.update(cx, |panel, cx| panel.handle(Cast::Type(text.to_owned()), cx));
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Flush), cx));
        cx.run_until_parked();
    }

    fn on_disk(repo: &TestRepo) -> String {
        std::fs::read_to_string(repo.path().join("a.txt")).unwrap()
    }

    #[gpui::test]
    fn should_edit_the_line_shown_when_the_file_changed_before_the_tab_was_shown(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        // a restored tab not shown yet: its diff was read, then the file changed
        repo.write("a.txt", "zero\none\nTWO\nthree\n");
        let cx = cx.add_empty_window();

        // when
        draw(&panel, cx);
        let shown = new_lines(&panel, cx);
        // the caret starts on the first added line
        type_and_save(&panel, "x", cx);

        // then
        assert_eq!(shown, ["zero", "one", "TWO", "three"]);
        assert_eq!(on_disk(&repo), "xzero\none\nTWO\nthree\n");
        assert_eq!(new_lines(&panel, cx), ["xzero", "one", "TWO", "three"]);
    }

    #[gpui::test]
    fn should_take_the_file_again_to_edit_when_reloaded_after_an_outside_change(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        let cx = cx.add_empty_window();
        draw(&panel, cx);
        repo.write("a.txt", "one\nTWO\nthree\nfour\n");

        // when
        tab.body.update(cx, |body, cx| body.reload(cx));
        cx.run_until_parked();
        draw(&panel, cx);
        let shown = new_lines(&panel, cx);
        type_and_save(&panel, "x", cx);

        // then
        assert_eq!(shown, ["one", "TWO", "three", "four"]);
        // no conflict: the edit applies to the file as it is now
        assert_eq!(on_disk(&repo), "one\nxTWO\nthree\nfour\n");
    }

    #[gpui::test]
    fn should_take_an_outside_change_to_edit_only_while_nothing_is_unsaved(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        let cx = cx.add_empty_window();
        draw(&panel, cx);
        let path = repo.path().join("a.txt");
        let changed = |text: &str, cx: &mut VisualTestContext| {
            repo.write("a.txt", text);
            let stamp = cx.read(|cx| tab.edited_stamp(cx)).expect("the file is edited");
            cx.update(|_, cx| tab.take_reread(codeview::document::reread(&path, stamp), cx));
            cx.run_until_parked();
            draw(&panel, cx);
        };

        // when
        changed("one\nTWO\nthree\nfour\n", cx);
        let taken = new_lines(&panel, cx);
        panel.update(cx, |panel, cx| panel.handle(Cast::Type("x".to_owned()), cx));
        changed("theirs\n", cx);
        let kept = new_lines(&panel, cx);
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Flush), cx));
        cx.run_until_parked();

        // then
        assert_eq!(taken, ["one", "TWO", "three", "four"]);
        // with an unsaved edit the rows go on showing the text edited, and the save meets the change
        assert_eq!(kept, ["one", "xTWO", "three", "four"]);
        assert_eq!(on_disk(&repo), "theirs\n");
        assert!(panel.read_with(cx, |panel, cx| panel.has_unsaved(cx)));
    }

    #[gpui::test]
    fn should_leave_the_diff_read_only_with_a_note_when_its_file_cannot_be_read_to_edit(cx: &mut TestAppContext) {
        // given
        styled(cx);
        let (repo, _) = changed_repo();
        let tab = cx.update(|cx| diff_tab(&repo, DiffSource::Uncommitted, cx));
        let panel = cx.read(|cx| tab.editing_panel(cx)).expect("an uncommitted diff edits");
        // the repository went before the tab was shown
        std::fs::remove_dir_all(repo.path().join(".git")).unwrap();
        let cx = cx.add_empty_window();

        // when
        draw(&panel, cx);

        // then
        assert!(panel.read_with(cx, |panel, cx| !panel.is_editable_diff(cx)));
        let note = tab.body.read_with(cx, |body, cx| body.note(cx)).unwrap_or_default();
        assert!(note.starts_with("read-only: cannot open the repository"), "{note}");
    }
}
