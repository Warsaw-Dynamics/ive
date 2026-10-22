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
