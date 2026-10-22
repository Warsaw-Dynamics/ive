//! A commit tab: author, date, message, parents, then each changed file's diff against the first parent.

use std::path::PathBuf;

use gpui::{App, AppContext, Context, Entity, IntoElement, Render, Task, Window, div, prelude::*, rgb};

use super::git_event::CommitLink;
use super::git_ui::{error_with_retry, layout_switch, muted};
use crate::age::date_time;
use crate::codeview_host::{DiffOrigin, diff_panel, serve, serve_texts};
use crate::git::diff::Sides;
use codeview::diff::Headers;
use codeview::panel::{Cast, Panel, PanelState};
use crate::git::log::{self, CommitDetail, CommitInfo};
use crate::git::repo::GitError;
use crate::git::types::Sha;
use crate::project::ProjectId;
use crate::theme;

pub struct CommitTab {
    pub project: ProjectId,
    pub sha: Sha,
    pub title: String,
    pub body: Entity<CommitView>,
}

impl CommitTab {
    /// A commit tab restored at start, titled as it was saved.
    pub fn reopen(project: ProjectId, dir: PathBuf, sha: Sha, title: String, cx: &mut App) -> Self {
        let body = cx.new(|cx| CommitView::new(dir, sha.clone(), cx));
        Self { project, sha, title, body }
    }

    pub fn panel_state(&self, cx: &App) -> PanelState {
        self.body.read(cx).files.read(cx).state()
    }

    /// Scrolls the files and collapses them as a saved tab had them, once they are read.
    pub fn restore(&self, state: PanelState, cx: &mut App) {
        let files = self.body.read(cx).files.clone();
        files.update(cx, |panel, cx| panel.handle(Cast::Restore(state), cx));
    }

    pub fn open(link: &CommitLink, cx: &mut App) -> Self {
        let sha = link.commit.sha.clone();
        let body = cx.new(|cx| CommitView::new(link.dir.clone(), sha.clone(), cx));
        Self { project: link.project.clone(), title: format!("{} {}", sha.short(), link.commit.subject), sha, body }
    }
}

/// The commit without its files, which live in the diff view.
struct Header {
    info: CommitInfo,
    message: String,
}

pub struct CommitView {
    dir: PathBuf,
    sha: Sha,
    header: Option<Result<Header, String>>,
    files: Entity<Panel>,
    /// read once, when first shown: a commit never changes
    started: bool,
    _load: Option<Task<()>>,
}

impl CommitView {
    /// Reads git once it is first shown.
    fn new(dir: PathBuf, sha: Sha, cx: &mut Context<Self>) -> Self {
        let files = diff_panel(Vec::new(), Headers::Shown, cx);
        serve(&files, format!("commit {}", sha.short()), cx);
        serve_texts(&files, DiffOrigin { dir: dir.clone(), sides: Sides::Commit(sha.clone()) }, cx);
        Self { dir, sha, header: None, files, started: false, _load: None }
    }

    /// Reads the commit the first time the tab is shown; after that only Retry reads it again.
    pub fn load_once(&mut self, cx: &mut Context<Self>) {
        if self.started {
            return;
        }
        self.load(cx);
    }

    /// Reads the commit; collapsed files and the scroll stay as they were.
    fn load(&mut self, cx: &mut Context<Self>) {
        self.started = true;
        let (dir, sha) = (self.dir.clone(), self.sha.clone());
        self._load = Some(cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { log::commit(&dir, &sha) }).await;
            this.update(cx, |view, cx| view.loaded(result, cx)).ok();
        }));
    }

    fn loaded(&mut self, result: Result<CommitDetail, GitError>, cx: &mut Context<Self>) {
        self.header = Some(match result {
            Ok(CommitDetail { info, message, files }) => {
                self.files.update(cx, |panel, cx| panel.handle(Cast::SetFiles(files), cx));
                Ok(Header { info, message })
            }
            Err(error) => {
                eprintln!("ive: {error}");
                Err(error.to_string())
            }
        });
        cx.notify();
    }
}

impl Render for CommitView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        let header = match &self.header {
            None => return div().p(theme.sizes.git.commit_message_padding.px()).child(muted("Reading the commit…", theme)),
            Some(Err(message)) => return div().p(theme.sizes.git.commit_message_padding.px()).child(error_with_retry(message.clone(), cx.listener(|view, _event, _window, cx| view.load(cx)), cx)),
            Some(Ok(header)) => header,
        };
        let parents = match header.info.parents.as_slice() {
            [] => "root commit".to_owned(),
            parents => format!("parents {}", parents.iter().map(Sha::short).collect::<Vec<_>>().join(", ")),
        };
        let top = div()
            .flex()
            .flex_none()
            .flex_col()
            .gap(theme.sizes.git.commit_header_gap.px())
            .px(theme.sizes.git.commit_header_padding_x.px())
            .py(theme.sizes.git.commit_header_padding_y.px())
            .border_b(theme.sizes.git.commit_header_border.px())
            .border_color(rgb(theme.border))
            .child(div().font_family(theme::mono_font(cx)).text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(header.info.sha.as_str().to_owned()))
            .child(div().text_size(theme.sizes.git.text_size.px()).child(format!("{} · {}", header.info.author, date_time(header.info.when))))
            .child(div().pt(theme.sizes.git.commit_subject_padding_top.px()).font_weight(theme.sizes.git.commit_subject_weight.font_weight()).child(header.message.clone()))
            .child(div().text_size(theme.sizes.git.detail_text_size.px()).text_color(rgb(theme.muted)).child(parents))
            .child(div().pt(theme.sizes.git.commit_subject_padding_top.px()).flex().child(layout_switch(&self.files, theme, cx)));
        div().flex().flex_col().flex_1().min_h_0().child(top).child(self.files.clone())
    }
}
