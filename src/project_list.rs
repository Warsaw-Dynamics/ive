//! The workspace's open and recent projects, without gpui: which open project a folder belongs to, adding a
//! project (the limit, its id) and closing one (Recent), and where Alt+F12 opens a terminal.

use std::collections::BTreeSet;
use std::path::Path;

use crate::open_project::OpenProject;
use crate::project::{self, CheckoutRef, ProjectError, ProjectId, ProjectKind, ProjectScan};
use crate::state::{MAX_PROJECTS, MAX_RECENT, RecentProject};

/// The open project a scanned folder belongs to: the one with the same root, or, for a plain folder, the deepest
/// one whose checkout holds it.
pub fn holding<'a>(projects: &'a [OpenProject], folder: &Path, scan: &ProjectScan) -> Option<&'a OpenProject> {
    let same_root = projects.iter().find(|open| open.project.root == scan.root);
    let holding = || {
        let containing = projects.iter().filter(|open| open.project.checkout_containing(folder).is_some());
        containing.max_by_key(|open| open.project.root.components().count())
    };
    match scan.kind {
        ProjectKind::Git => same_root,
        ProjectKind::Plain => same_root.or_else(holding),
    }
}

/// A project about to be added: its first scan and the number that scan was started as.
pub struct NewProject {
    pub scan: ProjectScan,
    pub number: u64,
    /// ids that still own data on disk (part 3's scratches), from `state::ids_with_data`
    pub ids_on_disk: Vec<String>,
}

/// Adds a project at the end of the tree. A recent one gets its old id back and leaves Recent; a new one gets a
/// slug of its folder name that no open, recent or on-disk id has. Refuses more than 100 projects, and a path
/// `state.toml` could not hold.
pub fn add(projects: &mut Vec<OpenProject>, recent: &mut Vec<RecentProject>, new: NewProject) -> Result<ProjectId, ProjectError> {
    if projects.len() >= MAX_PROJECTS {
        return Err(ProjectError::TooMany(MAX_PROJECTS));
    }
    if new.scan.root.to_str().is_none() {
        return Err(ProjectError::NotUtf8(new.scan.root));
    }
    let reopened = recent.iter().position(|closed| closed.path == new.scan.root).map(|index| recent.remove(index));
    let id = match reopened.and_then(|closed| ProjectId::parse(&closed.id).ok()) {
        Some(id) => id,
        None => new_id(&new.scan.root, projects, recent, &new.ids_on_disk),
    };
    let mut open = OpenProject::placeholder(id.clone(), new.scan.root.clone());
    open.apply(new.number, Ok(new.scan));
    projects.push(open);
    Ok(id)
}

/// The projects the tree shows: all but the `hidden` ones; all when that would leave none.
pub fn shown<'a>(projects: &'a [OpenProject], hidden: &'a BTreeSet<ProjectId>) -> impl Iterator<Item = &'a OpenProject> {
    let none_left = projects.iter().all(|open| hidden.contains(&open.project.id));
    projects.iter().filter(move |open| none_left || !hidden.contains(&open.project.id))
}

/// A slug of `root`'s folder name that no open, recent or on-disk id has.
fn new_id(root: &Path, projects: &[OpenProject], recent: &[RecentProject], on_disk: &[String]) -> ProjectId {
    let others = recent.iter().map(|closed| closed.id.as_str()).chain(on_disk.iter().map(String::as_str));
    let mut taken: Vec<ProjectId> = projects.iter().map(|open| open.project.id.clone()).collect();
    taken.extend(others.filter_map(|id| ProjectId::parse(id).ok()));
    ProjectId::slug(&project::folder_name(root), &taken.iter().collect::<Vec<_>>())
}

/// Closes project `id`: it leaves the tree and goes first in Recent, once; Recent keeps the newest 10.
pub fn close(projects: &mut Vec<OpenProject>, recent: &mut Vec<RecentProject>, id: &ProjectId) -> Option<OpenProject> {
    let index = projects.iter().position(|open| &open.project.id == id)?;
    let closed = projects.remove(index);
    recent.retain(|earlier| earlier.path != closed.project.root);
    recent.insert(0, RecentProject { path: closed.project.root.clone(), id: closed.project.id.as_str().to_owned() });
    recent.truncate(MAX_RECENT);
    Some(closed)
}

/// The checkout whose folder holds `path`, across all projects; the deepest wins.
pub fn checkout_of_path(projects: &[OpenProject], path: &Path) -> Option<CheckoutRef> {
    projects
        .iter()
        .filter_map(|open| open.project.checkout_containing(path).map(|checkout| open.project.checkout_ref(checkout)))
        .max_by_key(|checkout| checkout.path.components().count())
}

/// Where Alt+F12 opens a terminal: the shown tab's checkout, else the selected tree row's, else the main checkout
/// of the first project whose folder is there; `None` means the home folder.
pub fn current_checkout(shown: Option<CheckoutRef>, selected: Option<CheckoutRef>, projects: &[OpenProject]) -> Option<CheckoutRef> {
    shown.or(selected).or_else(|| {
        let first = projects.iter().find(|open| !open.missing)?;
        Some(first.project.checkout_ref(first.project.main_checkout()))
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn opened(id: &str, root: &str) -> OpenProject {
        let mut open = OpenProject::placeholder(ProjectId::new(id), PathBuf::from(root));
        open.apply(1, Ok(ProjectScan::plain(PathBuf::from(root))));
        open
    }

    fn new_project(root: &str) -> NewProject {
        NewProject { scan: ProjectScan::plain(PathBuf::from(root)), number: 1, ids_on_disk: Vec::new() }
    }

    fn recent(id: &str, root: &str) -> RecentProject {
        RecentProject { path: PathBuf::from(root), id: id.to_owned() }
    }

    fn checkout(project: &str, path: &str) -> Option<CheckoutRef> {
        Some(CheckoutRef { project: ProjectId::new(project), path: PathBuf::from(path) })
    }

    #[test]
    fn should_build_the_tree_of_the_projects_not_hidden() {
        // given
        let projects = [opened("a", "/a"), opened("b", "/b"), opened("c", "/c")];
        let hidden = |ids: &[&str]| ids.iter().map(|id| ProjectId::new(*id)).collect::<BTreeSet<_>>();
        let ids = |hidden: BTreeSet<ProjectId>| shown(&projects, &hidden).map(|open| open.project.id.as_str().to_owned()).collect::<Vec<_>>();

        // when
        let some = ids(hidden(&["a", "gone"]));
        let none = ids(hidden(&[]));
        let every = ids(hidden(&["a", "b", "c"]));

        // then
        assert_eq!(some, ["b", "c"]);
        assert_eq!(none, ["a", "b", "c"]);
        assert_eq!(every, ["a", "b", "c"], "the tree never shows no project");
    }

    #[test]
    fn should_find_the_project_of_the_same_root_or_the_plain_one_holding_the_folder() {
        // given
        let projects = [opened("a", "/a"), opened("notes", "/a/notes")];
        let plain = |root: &str| ProjectScan::plain(PathBuf::from(root));
        let inside_repository = ProjectScan { kind: ProjectKind::Git, ..plain("/a/notes/repo") };

        // when
        let found = [
            holding(&projects, Path::new("/a"), &plain("/a")),
            holding(&projects, Path::new("/a/notes/deep"), &plain("/a/notes/deep")),
            holding(&projects, Path::new("/a/notes/repo"), &inside_repository),
            holding(&projects, Path::new("/b"), &plain("/b")),
        ]
        .map(|open| open.map(|open| open.project.id.as_str()));

        // then
        assert_eq!(found, [Some("a"), Some("notes"), None, None]);
    }

    #[test]
    fn should_give_a_reopened_project_its_old_id_and_new_ones_an_unused_slug() {
        // given
        let mut projects = vec![opened("notes", "/x/notes")];
        let mut closed = vec![recent("notes-2", "/y/notes"), recent("notes-3", "/z/notes")];
        let on_disk = NewProject { ids_on_disk: vec!["notes-4".to_owned()], ..new_project("/w/notes") };

        // when
        let reopened = add(&mut projects, &mut closed, new_project("/y/notes")).unwrap();
        let fresh = add(&mut projects, &mut closed, on_disk).unwrap();

        // then
        assert_eq!((reopened.as_str(), fresh.as_str()), ("notes-2", "notes-5"));
        assert_eq!(closed, [recent("notes-3", "/z/notes")]);
    }

    #[test]
    fn should_refuse_the_101st_project_and_a_path_state_toml_cannot_hold() {
        // given
        let mut projects: Vec<OpenProject> = (0..MAX_PROJECTS).map(|n| opened(&format!("p{n}"), &format!("/p/{n}"))).collect();
        let mut closed = Vec::new();

        // when
        let refused = add(&mut projects, &mut closed, new_project("/q")).unwrap_err().to_string();

        // then
        assert_eq!(refused, "cannot open another project: 100 are open, the limit; close one first");
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            projects.pop();
            let odd = PathBuf::from(std::ffi::OsStr::from_bytes(b"/q/\xff"));
            let scan = ProjectScan::plain(odd);
            let error = add(&mut projects, &mut closed, NewProject { scan, number: 1, ids_on_disk: Vec::new() }).unwrap_err();
            assert!(matches!(error, ProjectError::NotUtf8(_)), "{error}");
        }
    }

    #[test]
    fn should_put_a_closed_project_first_in_recent_once_and_keep_ten() {
        // given
        let mut projects = vec![opened("a", "/a")];
        let mut closed: Vec<RecentProject> = (0..MAX_RECENT).map(|n| recent(&format!("r{n}"), &format!("/r/{n}"))).collect();
        closed.insert(3, recent("a", "/a"));

        // when
        let gone = close(&mut projects, &mut closed, &ProjectId::new("a"));

        // then
        assert!(gone.is_some() && projects.is_empty());
        assert_eq!(closed.len(), MAX_RECENT);
        assert_eq!((closed[0].id.as_str(), closed[9].id.as_str()), ("a", "r8"));
        assert_eq!(closed.iter().filter(|project| project.id == "a").count(), 1);
    }

    #[test]
    fn should_open_terminals_in_the_shown_tabs_checkout_then_the_selected_rows_then_the_first_projects() {
        // given
        let mut gone = opened("gone", "/gone");
        gone.missing = true;
        let projects = [gone, opened("b", "/b")];

        // when
        let picked = [
            current_checkout(checkout("b", "/b/tab"), checkout("b", "/b/row"), &projects),
            current_checkout(None, checkout("b", "/b/row"), &projects),
            current_checkout(None, None, &projects),
            current_checkout(None, None, &[]),
        ];

        // then
        assert_eq!(picked, [checkout("b", "/b/tab"), checkout("b", "/b/row"), checkout("b", "/b"), None]);
    }
}
