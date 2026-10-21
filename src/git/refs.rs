use std::collections::{HashMap, HashSet};
use std::path::Path;

use git2::{BranchType, Direction, Reference, Remote, Repository};

use super::repo::{self, GitError};
use super::types::{Sha, failed, lossy};
use crate::age::When;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BranchName {
    Local(String),
    Remote { remote: String, branch: String },
}

impl BranchName {
    /// As git prints it: `feature/x`, `origin/feature/x`.
    pub fn display(&self) -> String {
        match self {
            Self::Local(name) => name.clone(),
            Self::Remote { remote, branch } => format!("{remote}/{branch}"),
        }
    }

    /// The local branch a worktree checks out: `feature/x` for both kinds.
    pub fn local_name(&self) -> &str {
        match self {
            Self::Local(name) => name,
            Self::Remote { branch, .. } => branch,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: BranchName,
    pub sha: Sha,
    /// of its last commit
    pub when: When,
}

/// What a repository's refs point at: equal keys mean [`read`] would find the same branches and names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefsKey(Vec<(Vec<u8>, Vec<u8>)>);

/// Branches and log decorations, read once per change of the refs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refs {
    key: RefsKey,
    /// local and remote branches, remote `HEAD` excluded, newest last commit first
    pub branches: Vec<BranchInfo>,
    /// branch, remote branch and tag names pointing at each commit, sorted
    pub names: HashMap<Sha, Vec<String>>,
}

/// Reads the branches and decorations, or hands back `previous` when no ref moved since it was read: the views
/// showing them reload every few seconds, and naming remote branches and peeling tags costs more than listing refs.
#[tracing::instrument(level = "debug", name = "read_refs", skip_all, fields(dir = %dir.display()))]
pub fn read(dir: &Path, previous: Option<&Refs>) -> Result<Refs, GitError> {
    let fail = failed("read the branches", dir);
    let repo = repo::open(dir)?;
    let key = key(&repo).map_err(&fail)?;
    if let Some(previous) = previous.filter(|previous| previous.key == key) {
        return Ok(previous.clone());
    }
    Ok(Refs { key, branches: branches(&repo).map_err(&fail)?, names: names_by_commit(&repo).map_err(&fail)? })
}

fn key(repo: &Repository) -> Result<RefsKey, git2::Error> {
    let mut targets = Vec::new();
    for reference in repo.references()? {
        let reference = reference?;
        let target = match reference.target() {
            Some(oid) => oid.as_bytes().to_vec(),
            None => reference.symbolic_target_bytes().unwrap_or_default().to_vec(),
        };
        targets.push((reference.name_bytes().to_vec(), target));
    }
    targets.sort();
    Ok(RefsKey(targets))
}

fn branches(repo: &Repository) -> Result<Vec<BranchInfo>, git2::Error> {
    let remotes = remotes(repo)?;
    let mut found = Vec::new();
    for entry in repo.branches(None)? {
        let (branch, kind) = entry?;
        let reference = branch.get();
        // `origin/HEAD` points at another remote branch; it is not a branch of its own
        if reference.symbolic_target_bytes().is_some() {
            continue;
        }
        let (Ok(commit), Some(name)) = (reference.peel_to_commit(), branch_name(&remotes, reference, kind)) else {
            continue;
        };
        let time = commit.time();
        found.push(BranchInfo { name, sha: Sha::of(commit.id()), when: When { seconds: time.seconds(), offset_minutes: time.offset_minutes() } });
    }
    found.sort_by(|a, b| b.when.seconds.cmp(&a.when.seconds).then_with(|| a.name.display().cmp(&b.name.display())));
    Ok(found)
}

fn names_by_commit(repo: &Repository) -> Result<HashMap<Sha, Vec<String>>, git2::Error> {
    let mut names: HashMap<Sha, Vec<String>> = HashMap::new();
    for reference in repo.references()? {
        let reference = reference?;
        let decorates = reference.is_branch() || reference.is_remote() || reference.is_tag();
        if decorates && reference.symbolic_target_bytes().is_none() && let Ok(commit) = reference.peel_to_commit() {
            names.entry(Sha::of(commit.id())).or_default().push(lossy(reference.shorthand_bytes()));
        }
    }
    names.values_mut().for_each(|list| list.sort());
    Ok(names)
}

/// Branches a new worktree can start from: none checked out, and a remote branch only when no local branch has
/// its name (picking it would use the local one).
pub fn without_worktree(branches: &[BranchInfo], checked_out: &[String]) -> Vec<BranchInfo> {
    let locals: HashSet<&str> = branches
        .iter()
        .filter_map(|branch| match &branch.name {
            BranchName::Local(name) => Some(name.as_str()),
            BranchName::Remote { .. } => None,
        })
        .collect();
    branches
        .iter()
        .filter(|branch| match &branch.name {
            BranchName::Local(name) => checked_out.iter().all(|taken| taken != name),
            BranchName::Remote { branch, .. } => !locals.contains(branch.as_str()),
        })
        .cloned()
        .collect()
}

/// Loaded once per read: `Repository::branch_remote_name` loads every remote again for each branch, millions of
/// allocations for a repository with thousands of remote branches.
fn remotes(repo: &Repository) -> Result<Vec<(String, Remote<'_>)>, git2::Error> {
    let names = repo.remotes()?;
    // a name that is not UTF-8 names no branch, as before
    let utf8 = names.iter().filter_map(|name| name.ok().flatten());
    utf8.map(|name| repo.find_remote(name).map(|remote| (name.to_owned(), remote))).collect()
}

/// A remote branch belongs to the one remote whose fetch refspec maps to it, as `git_branch_remote_name` decides;
/// none, or several, leave it out.
fn remote_of<'a>(remotes: &'a [(String, Remote<'_>)], reference: &str) -> Option<&'a str> {
    let fetches_into = |remote: &Remote<'_>| remote.refspecs().any(|spec| spec.direction() == Direction::Fetch && spec.dst_matches(reference));
    let mut matching = remotes.iter().filter(|(_, remote)| fetches_into(remote));
    match (matching.next(), matching.next()) {
        (Some((name, _)), None) => Some(name),
        _ => None,
    }
}

fn branch_name(remotes: &[(String, Remote<'_>)], reference: &Reference<'_>, kind: BranchType) -> Option<BranchName> {
    let short = lossy(reference.shorthand_bytes());
    match kind {
        BranchType::Local => Some(BranchName::Local(short)),
        BranchType::Remote => {
            let remote = remote_of(remotes, reference.name().ok()?)?.to_owned();
            let branch = short.strip_prefix(&format!("{remote}/"))?.to_owned();
            (branch != "HEAD").then_some(BranchName::Remote { remote, branch })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;

    fn info(name: BranchName) -> BranchInfo {
        BranchInfo { name, sha: Sha::of(git2::Oid::ZERO_SHA1), when: When { seconds: 0, offset_minutes: 0 } }
    }

    fn remote(branch: &str) -> BranchName {
        BranchName::Remote { remote: "origin".to_owned(), branch: branch.to_owned() }
    }

    #[test]
    fn should_list_local_and_remote_branches_without_remote_head() {
        // given
        let origin = TestRepo::init();
        origin.write("o.txt", "o\n");
        origin.commit("origin root");
        origin.branch("feature/x");
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("root");
        repo.fetch_from("origin", &origin);
        repo.repo.reference_symbolic("refs/remotes/origin/HEAD", "refs/remotes/origin/main", true, "test").expect("remote HEAD");

        // when
        let listed = read(repo.path(), None).expect("branches").branches;

        // then
        let mut names: Vec<_> = listed.iter().map(|branch| branch.name.display()).collect();
        names.sort();
        assert_eq!(names, ["main", "origin/feature/x", "origin/main"]);
    }

    #[test]
    fn should_name_each_remote_branch_after_the_remote_fetching_it() {
        // given
        let origin = TestRepo::init();
        origin.write("o.txt", "o\n");
        origin.commit("origin root");
        let fork = TestRepo::init();
        fork.write("f.txt", "f\n");
        fork.commit("fork root");
        fork.branch("topic");
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("root");
        repo.fetch_from("origin", &origin);
        repo.fetch_from("origin-fork", &fork);

        // when
        let listed = read(repo.path(), None).expect("branches").branches;

        // then
        let mut names: Vec<_> = listed.into_iter().map(|branch| branch.name).collect();
        names.sort_by_key(BranchName::display);
        let remote_of = |remote: &str, branch: &str| BranchName::Remote { remote: remote.to_owned(), branch: branch.to_owned() };
        assert_eq!(names, [BranchName::Local("main".to_owned()), remote_of("origin-fork", "main"), remote_of("origin-fork", "topic"), remote_of("origin", "main")]);
    }

    #[test]
    fn should_offer_only_branches_no_checkout_has_and_prefer_local_ones() {
        // given
        let listed = [info(BranchName::Local("main".to_owned())), info(BranchName::Local("topic".to_owned())), info(remote("topic")), info(remote("new"))];

        // when
        let offered = without_worktree(&listed, &["main".to_owned()]);

        // then
        let names: Vec<_> = offered.iter().map(|branch| branch.name.display()).collect();
        assert_eq!(names, ["topic", "origin/new"]);
    }

    #[test]
    fn should_decorate_commits_with_branch_and_tag_names() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        let head = Sha::of(repo.commit("root"));
        repo.branch("b1");
        repo.tag("v1");

        // when
        let names = read(repo.path(), None).expect("names").names;

        // then
        assert_eq!(names.get(&head).map(Vec::as_slice), Some(&["b1".to_owned(), "main".to_owned(), "v1".to_owned()][..]));
    }

    #[test]
    fn should_reuse_what_was_read_until_a_ref_moves() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("root");
        let mut stale = read(repo.path(), None).expect("refs");
        stale.branches.clear();

        // when
        let kept = read(repo.path(), Some(&stale)).expect("refs");
        repo.branch("topic");
        let reread = read(repo.path(), Some(&stale)).expect("refs");

        // then
        assert_eq!((kept.branches.len(), reread.branches.len()), (0, 2));
    }
}
