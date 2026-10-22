//! Commits in real temp repositories through the `git` command; skipped when there is none.

use std::fs;
use std::path::PathBuf;

use super::*;

/// A temp repository, its `git` kept off the user's own config.
struct Repo {
    _dir: tempfile::TempDir,
    top: PathBuf,
    git: Git,
}

/// A repository with `f` committed as `text`; `None` (the test is skipped) without a `git` command.
fn repo_with(text: &str) -> Option<Repo> {
    let git = Git::with_env(vec![("GIT_CONFIG_GLOBAL".to_owned(), "/dev/null".to_owned()), ("GIT_CONFIG_NOSYSTEM".to_owned(), "1".to_owned())]);
    let dir = tempfile::tempdir().unwrap();
    if git.output(dir.path(), Call::of(&["--version"])).is_err() {
        eprintln!("skipped: no `git` command on PATH");
        return None;
    }
    let top = dir.path().canonicalize().unwrap();
    let repo = Repo { _dir: dir, top, git };
    repo.run(&["init", "-q", "-b", "main"]);
    repo.run(&["config", "user.name", "Test"]);
    repo.run(&["config", "user.email", "test@example.com"]);
    repo.write("f", text);
    repo.run(&["add", "-A"]);
    repo.run(&["commit", "-q", "-m", "first"]);
    Some(repo)
}

impl Repo {
    fn run(&self, args: &[&str]) -> String {
        self.git.text(&self.top, Call::of(args), "run a test command").unwrap_or_else(|refusal| panic!("git {args:?}: {refusal}"))
    }

    fn write(&self, name: &str, text: &str) {
        fs::write(self.top.join(name), text).unwrap();
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.top.join(name)).unwrap()
    }

    fn head(&self) -> String {
        self.run(&["rev-parse", "HEAD"]).trim().to_owned()
    }

    /// Commits `text` as `f`'s content on HEAD `head_id`.
    fn commit(&self, head_id: &str, text: &str, message: &str) -> Result<(), Refusal> {
        let commit = BlockCommit { path: self.top.join("f"), head_id: head_id.to_owned(), text: text.as_bytes().to_vec(), message: message.to_owned() };
        commit_block(&self.git, &commit)
    }
}

const HEAD_TEXT: &str = "a\nb\nc\nd\n";

#[test]
fn should_commit_only_the_block() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let before = repo.head();
    repo.write("f", "a\nB\nc\nD\n");

    // when
    let committed = repo.commit(&before, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["show", "HEAD:f"]), "a\nB\nc\nd\n");
    assert_eq!(repo.run(&["log", "-1", "--format=%P %s"]), format!("{before} change b\n"));
    assert_eq!(repo.read("f"), "a\nB\nc\nD\n", "the work tree stays");
    assert_eq!(repo.run(&["diff", "HEAD", "--numstat"]), "1\t1\tf\n", "only the other change is left");
    let temp_files = fs::read_dir(repo.top.join(".git")).unwrap().filter(|entry| entry.as_ref().unwrap().file_name().to_string_lossy().starts_with("ive-commit-index")).count();
    assert_eq!(temp_files, 0, "the temporary index is gone");
}

#[test]
fn should_leave_other_staged_files_staged() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    repo.write("g", "staged\n");
    repo.run(&["add", "g"]);
    let head = repo.head();

    // when
    let committed = repo.commit(&head, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["show", "--format=", "--name-only", "HEAD"]), "f\n");
    assert_eq!(repo.run(&["diff", "--cached", "--name-status"]), "A\tg\n");
}

#[test]
fn should_refuse_when_the_file_has_staged_changes() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    repo.write("f", "a\nb\nc\nD\n");
    repo.run(&["add", "f"]);
    let head = repo.head();

    // when
    let committed = repo.commit(&head, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(committed, Err(Refusal::StagedChanges));
    assert_eq!(repo.head(), head);
}

#[test]
fn should_refuse_during_a_merge() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    repo.write("g", "base\n");
    repo.run(&["add", "g"]);
    repo.run(&["commit", "-q", "-m", "g"]);
    repo.run(&["checkout", "-q", "-b", "side"]);
    repo.write("g", "side\n");
    repo.run(&["commit", "-q", "-am", "side"]);
    repo.run(&["checkout", "-q", "main"]);
    repo.write("g", "main\n");
    repo.run(&["commit", "-q", "-am", "main"]);
    let conflicted = repo.git.output(&repo.top, Call::of(&["merge", "side"])).unwrap();
    let head = repo.head();

    // when
    let committed = repo.commit(&head, "a\nB\nc\nd\n", "change b");

    // then
    assert!(!conflicted.status.success(), "the merge stops on its conflict");
    assert_eq!(committed, Err(Refusal::InProgress("merge")));
}

#[test]
fn should_refuse_when_head_moved() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let seen = repo.head();
    repo.write("g", "later\n");
    repo.run(&["add", "g"]);
    repo.run(&["commit", "-q", "-m", "later"]);

    // when
    let committed = repo.commit(&seen, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(committed, Err(Refusal::HeadMoved));
}

#[test]
fn should_commit_unsaved_block_text() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let head = repo.head();

    // when
    let committed = repo.commit(&head, "a\nunsaved\nc\nd\n", "typed");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["show", "HEAD:f"]), "a\nunsaved\nc\nd\n");
    assert_eq!(repo.read("f"), HEAD_TEXT, "the file on disk is not written");
}

#[test]
fn should_run_clean_filters() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    repo.write(".gitattributes", "f text eol=crlf\n");
    repo.run(&["add", ".gitattributes"]);
    repo.run(&["commit", "-q", "-m", "crlf"]);
    let head = repo.head();

    // when
    let committed = repo.commit(&head, "a\r\nB\r\nc\r\nd\r\n", "change b");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["cat-file", "blob", "HEAD:f"]), "a\nB\nc\nd\n");
}

#[test]
fn should_update_the_index_entry_when_nothing_was_staged() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let head = repo.head();
    repo.write("f", "a\nB\nc\nd\n");

    // when
    let committed = repo.commit(&head, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["diff", "--cached", "--name-only"]), "", "the commit does not show as staged backwards");
    assert_eq!(repo.run(&["status", "--porcelain"]), "");
}

#[test]
fn should_refuse_an_empty_message_and_a_file_staged_as_new() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let head = repo.head();
    repo.write("new", "x\n");
    repo.run(&["add", "new"]);
    let new = BlockCommit { path: repo.top.join("new"), head_id: head.clone(), text: b"y\n".to_vec(), message: "new".to_owned() };

    // when
    let empty = repo.commit(&head, "a\nB\nc\nd\n", " \n");
    let staged_new = commit_block(&repo.git, &new);

    // then
    assert_eq!((empty, staged_new), (Err(Refusal::EmptyMessage), Err(Refusal::StagedNew)));
    assert_eq!(repo.head(), head);
}

/// Commits `f`'s HEAD text with `b` changed on the current HEAD.
fn commit_b(repo: &Repo) -> Result<(), Refusal> {
    repo.commit(&repo.head(), "a\nB\nc\nd\n", "change b")
}

/// Makes `script` the repository's hook `name`.
fn hook(repo: &Repo, name: &str, script: &str) {
    let path = repo.top.join(".git/hooks").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, format!("#!/bin/sh\nset -e\n{script}\n")).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&path, permissions).unwrap();
}

/// The repository's commits, newest first: `format` of each.
fn log(repo: &Repo, format: &str) -> Vec<String> {
    repo.run(&["log", &format!("--format={format}")]).lines().map(str::to_owned).collect()
}

#[test]
fn should_undo_its_commit_when_another_lands_before_it() {
    // given
    let Some(mut repo) = repo_with(HEAD_TEXT) else { return };
    let seen = repo.head();
    let top = repo.top.clone();
    let other = Git::with_env(repo.git.env.clone());
    repo.git.before_commit = Box::new(move || {
        fs::write(top.join("g"), "concurrent\n").unwrap();
        other.text(&top, Call::of(&["add", "g"]), "stage g").unwrap();
        other.text(&top, Call::of(&["commit", "-q", "-m", "concurrent"]), "commit g").unwrap();
    });

    // when
    let committed = repo.commit(&seen, "a\nB\nc\nd\n", "change b");

    // then
    let Err(Refusal::TakenOff { commit, parent }) = &committed else { panic!("not taken off: {committed:?}") };
    assert_eq!(parent, &repo.head());
    assert!(committed.as_ref().unwrap_err().to_string().contains(commit.as_str()), "names the commit taken off");
    assert_eq!(log(&repo, "%s"), ["concurrent", "first"], "the other commit stays, ours goes");
    assert_eq!(repo.run(&["show", "HEAD:g"]), "concurrent\n");
    assert_eq!(repo.run(&["show", "HEAD:f"]), HEAD_TEXT);
    assert_eq!(repo.read("f"), HEAD_TEXT, "the work tree stays");
    assert_eq!(repo.run(&["log", "-g", "-1", "--format=%gs", "HEAD"]), "ive: undo commit on moved HEAD\n");
}

#[test]
fn should_say_how_to_undo_its_commit_when_head_moves_on_again() {
    // given
    let Some(mut repo) = repo_with(HEAD_TEXT) else { return };
    let seen = repo.head();
    let top = repo.top.clone();
    let other = Git::with_env(repo.git.env.clone());
    repo.git.before_commit = Box::new(move || {
        other.text(&top, Call::of(&["-c", "core.hooksPath=/dev/null", "commit", "-q", "--allow-empty", "-m", "before"]), "commit before").unwrap();
    });
    hook(&repo, "post-commit", "unset GIT_INDEX_FILE\ngit -c core.hooksPath=/dev/null commit -q --allow-empty -m after");

    // when
    let committed = repo.commit(&seen, "a\nB\nc\nd\n", "change b");

    // then
    assert_eq!(log(&repo, "%s")[..3], ["after", "change b", "before"], "nothing is lost");
    let ids = log(&repo, "%H");
    let (ours, before) = (&ids[1], &ids[2]);
    let Err(Refusal::UndoFailed { commit, parent, .. }) = &committed else { panic!("not refused as stranded: {committed:?}") };
    assert_eq!((commit, parent), (ours, before));
    let said = committed.unwrap_err().to_string();
    assert!(said.contains(ours.as_str()) && said.contains(&format!("git reset --soft {before}")) && said.contains(&format!("git revert {ours}")), "says what happened and how to undo it: {said}");
}

#[test]
fn should_move_the_index_to_the_entry_a_hook_committed() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    hook(&repo, "pre-commit", "blob=$(printf 'a\\nHOOKED\\nc\\nd\\n' | git hash-object -w --stdin)\ngit update-index --cacheinfo 100644,$blob,f");

    // when
    let committed = commit_b(&repo);

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["show", "HEAD:f"]), "a\nHOOKED\nc\nd\n");
    assert_eq!(repo.run(&["diff", "--cached", "--name-only"]), "", "the index holds what was committed");
}

#[test]
fn should_not_pass_on_the_git_locations_ive_was_started_with() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };

    // when
    let command = repo.git.command(&repo.top, &Call::of(&["status"]));

    // then
    let removed: Vec<String> = command.get_envs().filter(|(_, value)| value.is_none()).map(|(name, _)| name.to_string_lossy().into_owned()).collect();
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_COMMON_DIR"] {
        assert!(removed.iter().any(|removed| removed == name), "{name} is not removed: {removed:?}");
    }
}

#[test]
fn should_refuse_a_file_git_filters_or_reencodes() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    repo.write("g", "g\n");
    repo.run(&["add", "g"]);
    repo.write(".gitattributes", "f filter=upper\ng working-tree-encoding=UTF-16LE\n");
    repo.run(&["add", ".gitattributes"]);
    repo.run(&["commit", "-q", "-m", "attributes"]);
    let head = repo.head();
    let g = BlockCommit { path: repo.top.join("g"), head_id: head.clone(), text: b"G\n".to_vec(), message: "g".to_owned() };

    // when
    let filtered = commit_b(&repo);
    let encoded = commit_block(&repo.git, &g);

    // then
    assert_eq!((filtered, encoded), (Err(Refusal::Filtered), Err(Refusal::Filtered)));
    assert_eq!(repo.head(), head);
}

#[test]
fn should_commit_a_file_in_a_folder_with_spaces_and_accents_keeping_its_mode() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let name = "dir with space/żółw é.sh";
    fs::create_dir_all(repo.top.join("dir with space")).unwrap();
    repo.write(name, "old\n");
    repo.run(&["add", "-A"]);
    repo.run(&["update-index", "--chmod=+x", "--", name]);
    repo.run(&["commit", "-q", "-m", "script"]);
    let commit = BlockCommit { path: repo.top.join(name), head_id: repo.head(), text: b"new\n".to_vec(), message: "-m starts with a dash".to_owned() };

    // when
    let committed = commit_block(&repo.git, &commit);

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["show", &format!("HEAD:{name}")]), "new\n");
    assert!(repo.run(&["ls-tree", "HEAD", "--", name]).starts_with("100755 blob "), "the mode stays");
    assert_eq!(log(&repo, "%s")[0], "-m starts with a dash");
    assert!(repo.run(&["log", "-g", "-1", "--format=%gs", "HEAD"]).starts_with("commit (ive "), "the reflog names the commit IVE's");
}

#[test]
fn should_commit_in_a_linked_worktree_on_its_branch() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let linked = tempfile::tempdir().unwrap();
    let place = linked.path().canonicalize().unwrap().join("linked");
    repo.run(&["worktree", "add", "-q", "-b", "side", &place.to_string_lossy()]);
    let head = repo.head();
    let commit = BlockCommit { path: place.join("f"), head_id: head.clone(), text: b"a\nB\nc\nd\n".to_vec(), message: "on side".to_owned() };
    fs::write(place.join("f"), "a\nB\nc\nd\n").unwrap();

    // when
    let committed = commit_block(&repo.git, &commit);

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(repo.run(&["log", "-1", "--format=%s", "side"]), "on side\n");
    assert_eq!(repo.head(), head, "the main work tree's branch stays");
    assert_eq!(repo.run(&["status", "--porcelain"]), "");
    assert_eq!(repo.git.text(&place, Call::of(&["status", "--porcelain"]), "read the linked status").unwrap(), "", "the linked index follows");
}

/// Commits `g` on HEAD, so HEAD has a parent; HEAD's id.
fn second_commit(repo: &Repo) -> String {
    repo.write("g", "g\n");
    repo.run(&["add", "g"]);
    repo.run(&["commit", "-q", "-m", "second"]);
    repo.head()
}

#[test]
fn should_find_its_commit_when_a_pre_commit_hook_resets_head() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let before = second_commit(&repo);
    hook(&repo, "pre-commit", "git reset -q --soft HEAD");
    repo.write("f", "a\nB\nc\nd\n");

    // when
    let committed = commit_b(&repo);

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(log(&repo, "%s %P")[..2], [format!("change b {before}"), format!("second {}", log(&repo, "%H")[2])]);
    assert_eq!(repo.run(&["show", "HEAD:f"]), "a\nB\nc\nd\n");
    assert_eq!(repo.run(&["status", "--porcelain"]), "", "the index follows the commit");
}

#[test]
fn should_find_its_commit_when_a_pre_commit_hook_checks_out_the_branch() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let before = second_commit(&repo);
    hook(&repo, "pre-commit", "git checkout -q main");
    repo.write("f", "a\nB\nc\nd\n");

    // when
    let committed = commit_b(&repo);

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(log(&repo, "%s %P")[0], format!("change b {before}"));
    assert_eq!(repo.run(&["status", "--porcelain"]), "", "the index follows the commit");
}

#[test]
fn should_find_its_commit_by_the_reflogs_first_line_of_a_message_with_odd_spacing() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let before = repo.head();

    // when
    let committed = repo.commit(&before, "a\nB\nc\nd\n", "\n  change \t  b \nwrapped\n\nbody\n");

    // then
    assert_eq!(committed, Ok(()));
    assert_eq!(log(&repo, "%P")[0], before);
}

#[test]
fn should_say_it_committed_head_when_the_reflog_lost_its_entry() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let before = repo.head();
    hook(&repo, "post-commit", "git reflog delete HEAD@{0}");
    repo.write("f", "a\nB\nc\nd\n");

    // when
    let committed = commit_b(&repo);

    // then
    let head = repo.head();
    assert_eq!(committed, Err(Refusal::MadeUnconfirmed { commit: head.clone() }));
    assert_eq!(log(&repo, "%s %P")[0], format!("change b {before}"), "nothing is undone");
    assert_eq!(repo.run(&["status", "--porcelain"]), "", "the index follows the commit");
    assert!(committed.unwrap_err().to_string().contains(&head));
}

#[test]
fn should_undo_nothing_when_it_cannot_identify_its_commit() {
    // given
    let Some(repo) = repo_with(HEAD_TEXT) else { return };
    let after = "git reflog delete HEAD@{0}\nunset GIT_INDEX_FILE\nGIT_REFLOG_ACTION=other git -c core.hooksPath=/dev/null commit -q --allow-empty -m after";
    hook(&repo, "post-commit", after);

    // when
    let committed = commit_b(&repo);

    // then
    assert_eq!(committed, Err(Refusal::MadeUnknown));
    assert_eq!(log(&repo, "%s"), ["after", "change b", "first"], "nothing is undone");
    assert!(committed.unwrap_err().to_string().contains("git reflog"));
}

#[test]
fn should_not_pass_on_config_or_repository_limits_ive_was_started_with() {
    // given
    let names = ["PATH", "GIT_CONFIG_KEY_0", "GIT_CONFIG_VALUE_0", "GIT_CONFIG_KEY_12", "GIT_CONFIG_GLOBAL"].map(str::to_owned);

    // when
    let removed = inherited(names.into_iter());

    // then
    let expected = ["GIT_CONFIG_PARAMETERS", "GIT_CONFIG_COUNT", "GIT_NAMESPACE", "GIT_CEILING_DIRECTORIES", "GIT_CONFIG_KEY_0", "GIT_CONFIG_VALUE_0", "GIT_CONFIG_KEY_12"];
    for name in expected {
        assert!(removed.iter().any(|removed| removed == name), "{name} is not removed: {removed:?}");
    }
    assert!(!removed.iter().any(|removed| removed == "PATH" || removed == "GIT_CONFIG_GLOBAL"), "{removed:?}");
}
