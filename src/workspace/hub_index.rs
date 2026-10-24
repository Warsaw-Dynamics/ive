//! Keeps the hub's file index: a project's checkouts are read when it opens, read again when the hub opens after
//! a change or a while, and merged into one corpus off the UI thread.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gpui::Context;

use super::Workspace;
use crate::git::{refs, status};
use crate::hub::corpus::{self, Corpus, ProjectInput};
use crate::hub::files::{self, CheckoutFiles};
use crate::hub::index::{Branch, Read};
use crate::hub::model::Effect;
use crate::open_project::OpenProject;
use crate::run::file::{self as run_file, RUN_FILE, RunFile};
use crate::editor::Tab;
use crate::favicon::{Favicons, MAX_FAVICON_BYTES};
use crate::project::{CheckoutRef, ProjectId, ProjectKind};
use crate::script::excluded::Excluded;

/// One corpus build at a time; a change meanwhile builds again after it.
#[derive(Debug, Default)]
pub(super) struct CorpusBuild {
    building: bool,
    again: bool,
}

/// A project's listings, handed to the background build.
struct ProjectListing {
    id: ProjectId,
    checkouts: Vec<Option<Arc<CheckoutFiles>>>,
    ignored: Vec<Option<Arc<CheckoutFiles>>>,
}

impl Workspace {
    /// Project `id` was read: its checkouts never indexed are read now, so the hub's first search finds them.
    pub(super) fn index_project(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        let Some(open) = self.open_project(id) else {
            return;
        };
        let paths = indexed_paths(open);
        let unread = self.hub_index.unread(&paths);
        self.read_hub_checkouts(id, unread, cx);
    }

    /// The hub opens or a checkout changed while it is open: every checkout due is read again.
    pub(super) fn index_due_checkouts(&mut self, cx: &mut Context<Self>) {
        let projects: Vec<(ProjectId, Vec<PathBuf>)> =
            self.projects.iter().map(|open| (open.project.id.clone(), indexed_paths(open))).collect();
        let now = Instant::now();
        for (id, paths) in projects {
            let due = self.hub_index.due(&paths, now);
            self.read_hub_checkouts(&id, due, cx);
        }
    }

    /// Files or git changed in `checkout`: read again when the hub next needs it, at once while it, or the
    /// checkout's Run configurations tab, is shown.
    pub(super) fn hub_checkout_changed(&mut self, checkout: &Path, cx: &mut Context<Self>) {
        self.hub_index.mark_dirty(checkout);
        if self.hub_tab_id().is_some() {
            self.index_due_checkouts(cx);
        }
        let shown = match self.tabs.active() {
            Some(Tab::RunConfigs(tab)) if tab.checkout.path == checkout => Some(tab.checkout.clone()),
            _ => None,
        };
        if let Some(shown) = shown {
            self.index_checkout(&shown, cx);
        }
    }

    /// Reads `checkout` when it was never read, changed since or is stale.
    pub(super) fn index_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        let due = self.hub_index.due(std::slice::from_ref(&checkout.path), Instant::now());
        self.read_hub_checkouts(&checkout.project, due, cx);
    }

    /// The repository's git dir changed (an index write, a commit): every checkout of its project is due.
    pub(super) fn hub_git_changed(&mut self, dir: &Path, cx: &mut Context<Self>) {
        let checkouts: Vec<PathBuf> = self
            .projects
            .iter()
            .filter(|open| open.project.git_dir.as_ref().is_some_and(|git_dir| dir.starts_with(git_dir)))
            .flat_map(indexed_paths)
            .collect();
        for checkout in checkouts {
            self.hub_index.mark_dirty(&checkout);
        }
        if self.hub_tab_id().is_some() {
            self.index_due_checkouts(cx);
        }
    }

    /// The Excluded switch went on: ignored files are read from now on.
    pub(super) fn index_ignored_files(&mut self, cx: &mut Context<Self>) {
        if !self.hub_index.wants_ignored() {
            self.hub_index.want_ignored();
            self.index_due_checkouts(cx);
        }
    }

    /// A project closed or a worktree was hidden: its files leave the index.
    pub(super) fn forget_closed_checkouts(&mut self, cx: &mut Context<Self>) {
        let open: Vec<PathBuf> = self.projects.iter().flat_map(indexed_paths).collect();
        self.hub_index.retain(&open);
        self.rebuild_corpus(cx);
    }

    fn read_hub_checkouts(&mut self, project: &ProjectId, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let Some(project) = self.project(project) else {
            return;
        };
        let kind = project.kind;
        // the main checkout reads the branches; those checked out in a worktree are left out
        let main = (kind == ProjectKind::Git).then(|| project.main_checkout().path.clone());
        let checked_out: Vec<String> = project.checkouts.iter().filter_map(|checkout| checkout.branch.clone()).collect();
        let excluded: Vec<String> = cx.try_global::<Excluded>().map_or_else(Vec::new, |excluded| excluded.names().to_vec());
        let ignored = self.hub_index.wants_ignored();
        let since = self.status_since();
        for path in paths {
            self.hub_index.started(&path);
            let branches = (main.as_ref() == Some(&path)).then(|| checked_out.clone());
            let wanted = Wanted { kind, excluded: excluded.clone(), ignored, branches, since };
            let dir = path.clone();
            let reading = cx.background_executor().spawn(async move { read_checkout(&dir, &wanted) });
            cx.spawn(async move |this, cx| {
                let result = reading.await;
                this.update(cx, |workspace, cx| workspace.checkout_read(&path, result, cx)).ok();
            })
            .detach();
        }
    }

    fn checkout_read(&mut self, checkout: &Path, result: Result<Read, String>, cx: &mut Context<Self>) {
        match result {
            Ok(read) => {
                if read.files.skipped > 0 {
                    eprintln!("ive: the hub leaves out {} paths of {} that are not UTF-8", read.files.skipped, checkout.display());
                }
                self.hub_index.read(checkout, read, Instant::now());
                // a shown Run configurations tab lists the run files just read
                cx.notify();
            }
            Err(error) => {
                eprintln!("ive: the hub cannot list the files of {}: {error}", checkout.display());
                self.hub_index.failed(checkout, error);
                // a shown Run configurations tab says why it lists nothing
                cx.notify();
            }
        }
        self.rebuild_corpus(cx);
    }

    /// Merges every open checkout's listing into a new corpus on the background executor; the hub searches again
    /// once it is there.
    pub(super) fn rebuild_corpus(&mut self, cx: &mut Context<Self>) {
        if self.corpus_build.building {
            self.corpus_build.again = true;
            return;
        }
        self.corpus_build.building = true;
        let listings: Vec<ProjectListing> = self
            .projects
            .iter()
            .map(|open| {
                let reads = open.listed_checkouts().map(|checkout| self.hub_index.files(&checkout.path));
                let (checkouts, ignored) = reads.map(|read| (read.map(|read| read.files.clone()), read.and_then(|read| read.ignored.clone()))).unzip();
                ProjectListing { id: open.project.id.clone(), checkouts, ignored }
            })
            .collect();
        let building = cx.background_executor().spawn(async move { build_corpus(listings) });
        cx.spawn(async move |this, cx| {
            let (ids, corpus) = building.await;
            this.update(cx, |workspace, cx| workspace.corpus_built(ids, corpus, cx)).ok();
        })
        .detach();
    }

    fn corpus_built(&mut self, ids: Vec<ProjectId>, corpus: Corpus, cx: &mut Context<Self>) {
        self.corpus = Arc::new(corpus);
        self.corpus_projects = ids;
        self.corpus_build.building = false;
        if std::mem::take(&mut self.corpus_build.again) {
            self.rebuild_corpus(cx);
        }
        let searched = self.hub_tab_id().and_then(|id| match self.tabs.get(id) {
            Some(Tab::Hub(hub)) => Some((id, hub.state.index_changed())),
            _ => None,
        });
        if let Some((id, Effect::Search)) = searched {
            self.start_hub_search(id, cx);
        }
        cx.notify();
    }
}

/// The checkouts the hub reads of a project: the listed ones, never a hidden worktree.
fn indexed_paths(open: &OpenProject) -> Vec<PathBuf> {
    open.listed_checkouts().map(|checkout| checkout.path.clone()).collect()
}

/// What a read of one checkout takes in.
struct Wanted {
    kind: ProjectKind,
    excluded: Vec<String>,
    ignored: bool,
    /// for the main checkout: the branches checked out in a worktree, which the branch list leaves out
    branches: Option<Vec<String>>,
    /// the oldest status read to reuse
    since: Instant,
}

/// One checkout's files, changes and run configurations, its ignored files too once asked for, and for the main
/// checkout the branches without a worktree. Reads the disk and git.
#[tracing::instrument(level = "debug", name = "index_checkout", skip_all, fields(dir = %dir.display()))]
fn read_checkout(dir: &Path, wanted: &Wanted) -> Result<Read, String> {
    let files = files::read_checkout(dir, wanted.kind, &wanted.excluded)?;
    let changes = match wanted.kind {
        ProjectKind::Git => status::shared(dir, wanted.since).map_err(|error| error.to_string())?.changes,
        ProjectKind::Plain => Arc::default(),
    };
    let ignored = wanted.ignored.then(|| files::read_ignored(dir, wanted.kind)).transpose()?.map(Arc::new);
    let configs = run_files(dir, &files);
    let too_big = |file: &str| std::fs::metadata(dir.join(file)).map_or(true, |metadata| metadata.len() > MAX_FAVICON_BYTES);
    let favicons = Favicons::from_paths(files.paths.iter().map(|(path, _)| path.as_str())).without(too_big);
    let branches = wanted.branches.as_deref().map_or_else(Vec::new, |checked_out| branches_without_worktree(dir, checked_out));
    Ok(Read { files: Arc::new(files), ignored, changes, configs: Arc::new(configs), favicons: Arc::new(favicons), branches: Arc::new(branches) })
}

/// The `.run.toml` files among `files` that parse, by checkout-relative folder; a broken one shows in its folder
/// overview, not in the hub.
fn run_files(dir: &Path, files: &CheckoutFiles) -> Vec<(String, Arc<RunFile>)> {
    let folders = files.paths.iter().filter_map(|(path, _)| match path.strip_suffix(RUN_FILE) {
        Some("") => Some(String::new()),
        Some(folder) => folder.strip_suffix('/').map(str::to_owned),
        None => None,
    });
    folders.filter_map(|folder| match run_file::load(&dir.join(&folder)) {
        Some(Ok(file)) => Some((folder, Arc::new(file))),
        _ => None,
    })
    .collect()
}

/// A failed read lists no branches: the `#` list then shows the worktrees alone.
fn branches_without_worktree(dir: &Path, checked_out: &[String]) -> Vec<Branch> {
    let Ok(refs) = refs::read(dir, None) else {
        return Vec::new();
    };
    let free = refs::without_worktree(&refs.branches, checked_out);
    free.into_iter()
        .map(|branch| {
            let remote = match &branch.name {
                refs::BranchName::Local(_) => None,
                refs::BranchName::Remote { remote, .. } => Some(remote.clone()),
            };
            Branch { shown: branch.name.display(), local: branch.name.local_name().to_owned(), remote, when: branch.when.seconds }
        })
        .collect()
}

/// The corpus of `listings`, each project's scratches read from its store.
fn build_corpus(listings: Vec<ProjectListing>) -> (Vec<ProjectId>, Corpus) {
    let scratches: Vec<Vec<String>> = listings
        .iter()
        .map(|listing| {
            let names = crate::scratch::store_of(&listing.id).ok().and_then(|store| store.list().ok());
            names.map_or_else(Vec::new, |names| names.into_iter().map(|info| info.name).collect())
        })
        .collect();
    let inputs: Vec<ProjectInput<'_>> = listings
        .iter()
        .zip(&scratches)
        .map(|(listing, scratches)| ProjectInput {
            checkouts: listing.checkouts.iter().map(Option::as_deref).collect(),
            ignored: listing.ignored.iter().map(Option::as_deref).collect(),
            scratches,
        })
        .collect();
    let corpus = corpus::build(&inputs);
    (listings.into_iter().map(|listing| listing.id).collect(), corpus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Checkout, ProjectScan};

    fn project(worktrees: &[&str]) -> OpenProject {
        let checkout = |path: &str, is_main: bool| Checkout { path: PathBuf::from(path), label: path.to_owned(), branch: None, head: None, is_main, status: None };
        let checkouts = std::iter::once(checkout("/p", true)).chain(worktrees.iter().map(|path| checkout(path, false))).collect();
        let mut open = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        open.apply(1, Ok(ProjectScan { root: PathBuf::from("/p"), kind: ProjectKind::Git, checkouts, error: None, git_dir: None }));
        open
    }

    #[test]
    fn should_index_the_listed_checkouts_only() {
        // given
        let mut open = project(&["/p/a", "/p/b"]);
        open.hide(Path::new("/p/a"));

        // when
        let paths = indexed_paths(&open);

        // then
        assert_eq!(paths, [PathBuf::from("/p"), PathBuf::from("/p/b")]);
    }
}
