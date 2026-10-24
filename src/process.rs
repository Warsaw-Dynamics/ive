//! Terminal sessions started in a checkout, owned here rather than by tabs: closing a tab keeps its process.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::icons;
use crate::project::{CheckoutRef, ProjectId};
use crate::terminal::{Progress, TerminalView};

/// Identity of a process; stays the same while its kind and title change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProcessId(u64);

impl ProcessId {
    /// For element ids and MCP answers.
    pub fn value(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub fn for_tests(value: u64) -> Self {
        Self(value)
    }
}

/// A run as the tree and the folder overview name it: a started process, or one still waiting for its `before`
/// steps, which has no process yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunId {
    Process(ProcessId),
    Pending(u64),
}

impl RunId {
    /// What click targets name it after: the process id MCP reports, or `pending-<n>`.
    pub fn target(self) -> String {
        match self {
            Self::Process(id) => id.value().to_string(),
            Self::Pending(number) => format!("pending-{number}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessKind {
    Terminal,
    /// a terminal whose foreground program is Claude Code
    ClaudeCode,
    /// a run configuration's program; kept after it exits
    Run,
}

/// Which configuration a run comes from, and whether its program still runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOrigin {
    /// the folder of its `.run.toml`
    pub folder: PathBuf,
    pub key: String,
    pub name: String,
    /// a static server's link (its `open` page); `None` for a command
    pub url: Option<String>,
    pub life: RunLife,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunLife {
    Running,
    /// `stopped` when IVE stopped it, rather than it ending by itself
    Exited { code: i32, stopped: bool },
}

impl ProcessKind {
    /// As MCP reports it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::ClaudeCode => "claude-code",
            Self::Run => "run",
        }
    }

    /// In the tree and on the tab.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Terminal => icons::TERMINAL,
            Self::ClaudeCode => icons::AGENT,
            Self::Run => icons::RUN,
        }
    }
}

/// Whether a process moved by hand stays where it was put.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pin {
    Free,
    /// moved before its directory was known: the first one reported is the one it had
    Next,
    At(PathBuf),
}

pub struct Process<V = TerminalView> {
    pub id: ProcessId,
    /// `None` for a shell started in the home folder, outside every project
    pub checkout: Option<CheckoutRef>,
    pub kind: ProcessKind,
    /// a run's configuration and life; `None` for terminals
    pub run: Option<RunOrigin>,
    /// what the program in the terminal set as its title, as tabs and tree rows show it
    title: String,
    progress: Progress,
    /// where its foreground program last worked, as the foreground poll saw it
    directory: Option<PathBuf>,
    /// moved to another checkout by hand: the directory it does not follow, only a later one
    pin: Pin,
    pub view: V,
}

impl<V> Process<V> {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// What the program last reported with `OSC 9;4`.
    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    pub fn pinned(&self) -> Option<&Path> {
        match &self.pin {
            Pin::At(directory) => Some(directory),
            Pin::Free | Pin::Next => None,
        }
    }

    /// Its program still runs: every terminal, and a run until it exits.
    pub fn is_live(&self) -> bool {
        self.run.as_ref().is_none_or(|origin| origin.life == RunLife::Running)
    }

    /// The link to a running server's page; `None` for anything else, and once the server has stopped.
    pub fn link(&self) -> Option<&str> {
        self.run.as_ref().filter(|origin| origin.life == RunLife::Running).and_then(|origin| origin.url.as_deref())
    }

    /// The icon of its tab and rows: a server's run shows as HTML.
    pub fn icon(&self) -> &'static str {
        match &self.run {
            Some(origin) if origin.url.is_some() => icons::FILE_HTML,
            _ => self.kind.icon(),
        }
    }
}

/// The processes of all checkouts, in start order.
/// Generic over the view so tests need no gpui window.
pub struct Processes<V = TerminalView> {
    entries: Vec<Process<V>>,
    next_id: u64,
    /// runs IVE asked to stop: their exit reads as stopped
    stopping: HashSet<ProcessId>,
    /// processes Stop ends for good: their tab closed, their row goes once they are gone
    closing: HashSet<ProcessId>,
}

impl<V> Default for Processes<V> {
    fn default() -> Self {
        Self { entries: Vec::new(), next_id: 0, stopping: HashSet::new(), closing: HashSet::new() }
    }
}

impl<V> Processes<V> {
    /// Adds the process `start(id)` creates; an error leaves the list as it was.
    pub fn start<E>(&mut self, checkout: Option<CheckoutRef>, start: impl FnOnce(ProcessId) -> Result<V, E>) -> Result<ProcessId, E> {
        let id = ProcessId(self.next_id);
        self.next_id += 1;
        let view = start(id)?;
        self.entries.push(Process { id, checkout, kind: ProcessKind::Terminal, run: None, title: "Terminal".to_owned(), progress: Progress::None, directory: None, pin: Pin::Free, view });
        Ok(id)
    }

    /// Adds the run `start(id)` creates, titled with its configuration's name; an error leaves the list as it was.
    pub fn start_run<E>(&mut self, checkout: Option<CheckoutRef>, origin: RunOrigin, start: impl FnOnce(ProcessId) -> Result<V, E>) -> Result<ProcessId, E> {
        let id = ProcessId(self.next_id);
        self.next_id += 1;
        let view = start(id)?;
        let title = origin.name.clone();
        self.entries.push(Process { id, checkout, kind: ProcessKind::Run, run: Some(origin), title, progress: Progress::None, directory: None, pin: Pin::Free, view });
        Ok(id)
    }

    /// The runs started from `folder` in the checkout at `checkout`, newest first.
    pub fn runs_in(&self, checkout: &Path, folder: &Path) -> Vec<&Process<V>> {
        let in_folder = |process: &&Process<V>| {
            process.run.as_ref().is_some_and(|origin| origin.folder == folder) && process.checkout.as_ref().is_some_and(|at| at.path == checkout)
        };
        self.entries.iter().rev().filter(in_folder).collect()
    }

    /// The runs started in `checkout`, from any folder, oldest first.
    pub fn runs_of(&self, checkout: &Path) -> Vec<&Process<V>> {
        let started_here = |process: &&Process<V>| process.run.is_some() && process.checkout.as_ref().is_some_and(|at| at.path == checkout);
        self.entries.iter().filter(started_here).collect()
    }

    /// Notes that IVE is stopping run `id`, so its exit counts as stopped.
    pub fn mark_stopping(&mut self, id: ProcessId) {
        self.stopping.insert(id);
    }

    /// Notes that Stop is ending process `id` for good; a run's exit then also reads as stopped.
    pub fn mark_closing(&mut self, id: ProcessId) {
        self.stopping.insert(id);
        self.closing.insert(id);
    }

    /// Whether IVE is ending process `id` itself, so a signal it dies of is no failure.
    pub fn is_stopping(&self, id: ProcessId) -> bool {
        self.stopping.contains(&id)
    }

    pub fn is_closing(&self, id: ProcessId) -> bool {
        self.closing.contains(&id)
    }

    /// Run `id`'s program exited with `code`.
    pub fn set_exited(&mut self, id: ProcessId, code: i32) {
        let stopped = self.stopping.remove(&id);
        if let Some(origin) = self.get_mut(id).and_then(|process| process.run.as_mut()) {
            origin.life = RunLife::Exited { code, stopped };
        }
    }

    /// The process whose id has this `value()`, as MCP names it.
    pub fn id_of(&self, value: u64) -> Option<ProcessId> {
        self.entries.iter().map(|process| process.id).find(|id| id.value() == value)
    }

    pub fn get(&self, id: ProcessId) -> Option<&Process<V>> {
        self.entries.iter().find(|process| process.id == id)
    }

    pub fn get_mut(&mut self, id: ProcessId) -> Option<&mut Process<V>> {
        self.entries.iter_mut().find(|process| process.id == id)
    }

    /// Removes the process; dropping what is returned hangs up its shell.
    pub fn close(&mut self, id: ProcessId) -> Option<Process<V>> {
        self.stopping.remove(&id);
        self.closing.remove(&id);
        let index = self.entries.iter().position(|process| process.id == id)?;
        Some(self.entries.remove(index))
    }

    /// The terminals and Claude Code sessions of a checkout; runs show under their folder instead.
    pub fn by_checkout(&self, path: &Path) -> Vec<&Process<V>> {
        let listed = |process: &&Process<V>| process.kind != ProcessKind::Run && process.checkout.as_ref().is_some_and(|checkout| checkout.path == path);
        self.entries.iter().filter(listed).collect()
    }

    pub fn in_project(&self, project: &ProjectId) -> Vec<&Process<V>> {
        self.entries.iter().filter(|process| process.checkout.as_ref().is_some_and(|checkout| &checkout.project == project)).collect()
    }

    /// Re-points the terminals of `main`'s project whose checkout `is_listed` rejects to `main`: a worktree removed,
    /// in IVE or outside it, leaves its terminals running, and the tree shows only listed checkouts. Its runs are
    /// handed back instead, for the caller to end: their folder is gone with the worktree.
    pub fn move_unlisted_to(&mut self, main: &CheckoutRef, is_listed: impl Fn(&Path) -> bool) -> Vec<ProcessId> {
        let mut orphans = Vec::new();
        for process in &mut self.entries {
            let Some(checkout) = process.checkout.as_mut().filter(|checkout| checkout.project == main.project && !is_listed(&checkout.path)) else {
                continue;
            };
            match process.kind {
                ProcessKind::Run => orphans.push(process.id),
                ProcessKind::Terminal | ProcessKind::ClaudeCode => *checkout = main.clone(),
            }
        }
        orphans
    }

    /// The processes of `project` whose program still runs: what closing it ends.
    pub fn live_in_project(&self, project: &ProjectId) -> usize {
        self.in_project(project).iter().filter(|process| process.is_live()).count()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Process<V>> {
        self.entries.iter()
    }

    /// Returns whether the title changed.
    pub fn set_title(&mut self, id: ProcessId, title: String) -> bool {
        match self.get_mut(id) {
            Some(process) if process.title != title => {
                process.title = title;
                true
            }
            _ => false,
        }
    }

    /// Returns whether the progress changed.
    pub fn set_progress(&mut self, id: ProcessId, progress: Progress) -> bool {
        match self.get_mut(id) {
            Some(process) if process.progress != progress => {
                process.progress = progress;
                true
            }
            _ => false,
        }
    }

    /// A directory other than the pinned one ends the pin.
    pub fn set_directory(&mut self, id: ProcessId, directory: PathBuf) {
        if let Some(process) = self.get_mut(id) {
            process.pin = match std::mem::replace(&mut process.pin, Pin::Free) {
                Pin::Next => Pin::At(directory.clone()),
                Pin::At(pinned) if pinned == directory => Pin::At(pinned),
                Pin::At(_) | Pin::Free => Pin::Free,
            };
            process.directory = Some(directory);
        }
    }

    /// Moves terminal or Claude Code session `id` to `checkout` by hand, pinned to its directory, so following
    /// directories leaves it there until that changes; returns whether it moved. A run stays under its folder.
    pub fn move_pinned(&mut self, id: ProcessId, checkout: CheckoutRef) -> bool {
        if !self.move_to(id, checkout) {
            return false;
        }
        if let Some(process) = self.get_mut(id) {
            process.pin = process.directory.clone().map_or(Pin::Next, Pin::At);
        }
        true
    }

    /// Restores a pin saved with the process's checkout.
    pub fn set_pinned(&mut self, id: ProcessId, directory: PathBuf) {
        if let Some(process) = self.get_mut(id) {
            process.pin = Pin::At(directory);
        }
    }

    /// Lists terminal or Claude Code session `id` under `checkout`; returns whether it moved. A run stays under its
    /// folder.
    pub fn move_to(&mut self, id: ProcessId, checkout: CheckoutRef) -> bool {
        match self.get_mut(id) {
            Some(process) if process.kind != ProcessKind::Run && process.checkout.as_ref() != Some(&checkout) => {
                process.checkout = Some(checkout);
                true
            }
            _ => false,
        }
    }

    /// Returns whether the kind changed. A run stays a run, whatever its foreground program.
    pub fn set_kind(&mut self, id: ProcessId, kind: ProcessKind) -> bool {
        match self.get_mut(id) {
            Some(process) if process.kind != kind && process.kind != ProcessKind::Run => {
                process.kind = kind;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn checkout(project: &str, path: &str) -> Option<CheckoutRef> {
        Some(CheckoutRef { project: ProjectId::new(project), path: PathBuf::from(path) })
    }

    #[test]
    fn should_list_processes_of_a_checkout_in_start_order() {
        // given
        let mut processes: Processes<&str> = Processes::default();
        let first = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>("one")).unwrap();
        processes.start(checkout("a", "/a/wt"), |_| Ok::<_, ()>("other")).unwrap();
        let third = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>("two")).unwrap();

        // when
        let ids: Vec<_> = processes.by_checkout(Path::new("/a")).iter().map(|process| process.id).collect();

        // then
        assert_eq!(ids, [first, third]);
        assert_eq!(processes.in_project(&ProjectId::new("a")).len(), 3);
    }

    #[test]
    fn should_move_processes_of_unlisted_checkout_to_main() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let removed = processes.start(checkout("a", "/a/removed"), |_| Ok::<_, ()>(())).unwrap();
        let kept = processes.start(checkout("a", "/a/kept"), |_| Ok::<_, ()>(())).unwrap();
        let other_project = processes.start(checkout("b", "/b/removed"), |_| Ok::<_, ()>(())).unwrap();
        let home = processes.start(None, |_| Ok::<_, ()>(())).unwrap();
        let main = checkout("a", "/a").unwrap();

        // when
        processes.move_unlisted_to(&main, |path| path == Path::new("/a") || path == Path::new("/a/kept"));

        // then
        let checkouts = [removed, kept, other_project, home].map(|id| processes.get(id).unwrap().checkout.clone());
        assert_eq!(checkouts, [checkout("a", "/a"), checkout("a", "/a/kept"), checkout("b", "/b/removed"), None]);
    }

    #[test]
    fn should_find_a_running_process_by_the_id_mcp_reports() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let first = processes.start(None, |_| Ok::<_, ()>(())).unwrap();
        let second = processes.start(None, |_| Ok::<_, ()>(())).unwrap();
        processes.close(first);

        // when
        let found = [first.value(), second.value(), 99].map(|value| processes.id_of(value));

        // then
        assert_eq!(found, [None, Some(second), None]);
    }

    #[test]
    fn should_keep_list_unchanged_when_start_fails() {
        // given
        let mut processes: Processes<&str> = Processes::default();

        // when
        let result = processes.start(checkout("a", "/a"), |_| Err("no shell"));

        // then
        assert_eq!((result, processes.iter().count()), (Err("no shell"), 0));
    }

    #[test]
    fn should_title_process_as_its_program_sets_whatever_the_kind() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(None, |_| Ok::<_, ()>(())).unwrap();
        processes.set_title(id, "zsh — ~/atlassian".to_owned());
        processes.set_kind(id, ProcessKind::ClaudeCode);

        // when
        let renamed = processes.set_title(id, "✳ Claude Code".to_owned());
        let renamed_again = processes.set_title(id, "✳ Claude Code".to_owned());

        // then
        assert_eq!((renamed, renamed_again, processes.get(id).unwrap().title()), (true, false, "✳ Claude Code".to_owned()));
    }

    fn origin(key: &str) -> RunOrigin {
        RunOrigin { folder: PathBuf::from("/a/misc"), key: key.to_owned(), name: key.to_owned(), url: None, life: RunLife::Running }
    }

    #[test]
    fn should_keep_runs_out_of_the_checkout_processes() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let terminal = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();
        let run = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let listed: Vec<_> = processes.by_checkout(Path::new("/a")).iter().map(|process| process.id).collect();

        // then
        assert_eq!(listed, [terminal]);
        assert_eq!(processes.in_project(&ProjectId::new("a")).len(), 2);
        assert_eq!((processes.get(run).unwrap().kind, processes.get(run).unwrap().title()), (ProcessKind::Run, "build".to_owned()));
    }

    #[test]
    fn should_list_runs_of_a_folder_newest_first() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let first = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();
        let second = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        processes.start_run(checkout("a", "/a/wt"), origin("app"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let runs: Vec<_> = processes.runs_in(Path::new("/a"), Path::new("/a/misc")).iter().map(|process| process.id).collect();

        // then
        assert_eq!(runs, [second, first]);
    }

    #[test]
    fn should_keep_an_exited_run_with_its_code() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();

        // when
        processes.set_exited(run, 1);

        // then
        let process = processes.get(run).unwrap();
        assert_eq!(process.run.as_ref().map(|origin| origin.life), Some(RunLife::Exited { code: 1, stopped: false }));
        assert!(!process.is_live());
    }

    #[test]
    fn should_offer_a_link_only_while_a_server_runs() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let server = RunOrigin { url: Some("http://127.0.0.1:4301/mock/".to_owned()), ..origin("mock") };
        let running = processes.start_run(checkout("a", "/a"), server.clone(), |_| Ok::<_, ()>(())).unwrap();
        let stopped = processes.start_run(checkout("a", "/a"), server, |_| Ok::<_, ()>(())).unwrap();
        let command = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();

        // when
        processes.set_exited(stopped, 143);

        // then
        let links = [running, stopped, command].map(|id| processes.get(id).unwrap().link().map(str::to_owned));
        assert_eq!(links, [Some("http://127.0.0.1:4301/mock/".to_owned()), None, None]);
    }

    #[test]
    fn should_count_only_live_processes_in_a_project() {
        // given
        let mut processes: Processes<()> = Processes::default();
        processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();
        processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        let build = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();
        processes.set_exited(build, 0);

        // when
        let live = processes.live_in_project(&ProjectId::new("a"));

        // then
        assert_eq!(live, 2);
    }

    #[test]
    fn should_hand_back_the_runs_of_an_unlisted_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let terminal = processes.start(checkout("a", "/a/removed"), |_| Ok::<_, ()>(())).unwrap();
        let run = processes.start_run(checkout("a", "/a/removed"), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        let kept = processes.start_run(checkout("a", "/a"), origin("build"), |_| Ok::<_, ()>(())).unwrap();
        let main = checkout("a", "/a").unwrap();

        // when
        let orphans = processes.move_unlisted_to(&main, |path| path == Path::new("/a"));

        // then
        assert_eq!(orphans, [run]);
        assert_eq!(processes.get(terminal).unwrap().checkout, Some(main));
        assert_eq!(processes.get(kept).unwrap().checkout, checkout("a", "/a"));
    }

    #[test]
    fn should_keep_a_run_a_run_whatever_runs_in_its_foreground() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let changed = [processes.set_kind(run, ProcessKind::Terminal), processes.set_kind(run, ProcessKind::ClaudeCode)];

        // then
        assert_eq!((changed, processes.get(run).unwrap().kind), ([false, false], ProcessKind::Run));
    }

    #[test]
    fn should_count_a_stopped_run_as_stopped() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        processes.mark_stopping(run);

        // when
        processes.set_exited(run, 143);

        // then
        assert_eq!(processes.get(run).unwrap().run.as_ref().map(|origin| origin.life), Some(RunLife::Exited { code: 143, stopped: true }));
    }

    #[test]
    fn should_count_a_closing_run_as_stopped() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        processes.mark_closing(run);

        // when
        processes.set_exited(run, 143);

        // then
        assert!(processes.is_closing(run));
        assert_eq!(processes.get(run).unwrap().run.as_ref().map(|origin| origin.life), Some(RunLife::Exited { code: 143, stopped: true }));
    }

    #[test]
    fn should_know_a_process_ive_is_stopping() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let (stopped, running) = (processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap(), processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap());

        // when
        processes.mark_closing(stopped);

        // then
        assert_eq!((processes.is_stopping(stopped), processes.is_stopping(running)), (true, false));
    }

    #[test]
    fn should_forget_closing_once_removed() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let terminal = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();
        processes.mark_closing(terminal);

        // when
        processes.close(terminal);

        // then
        assert!(!processes.is_closing(terminal));
    }

    #[test]
    fn should_move_terminal_to_another_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let moved = processes.move_to(id, checkout("a", "/a/wt").unwrap());

        // then
        assert_eq!((moved, processes.get(id).unwrap().checkout.clone()), (true, checkout("a", "/a/wt")));
    }

    #[test]
    fn should_adopt_home_terminal_into_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(None, |_| Ok::<_, ()>(())).unwrap();

        // when
        let moved = processes.move_to(id, checkout("b", "/b").unwrap());

        // then
        assert_eq!((moved, processes.get(id).unwrap().checkout.clone()), (true, checkout("b", "/b")));
    }

    #[test]
    fn should_never_move_a_run() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let moved = processes.move_to(run, checkout("a", "/a/wt").unwrap());

        // then
        assert_eq!((moved, processes.get(run).unwrap().checkout.clone()), (false, checkout("a", "/a")));
    }

    #[test]
    fn should_refuse_to_move_pin_a_run() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let run = processes.start_run(checkout("a", "/a"), origin("app"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let moved = processes.move_pinned(run, checkout("a", "/a/wt").unwrap());

        // then
        assert_eq!((moved, processes.get(run).unwrap().checkout.clone(), processes.get(run).unwrap().pinned()), (false, checkout("a", "/a"), None));
    }

    #[test]
    fn should_pin_moved_terminal_to_its_directory() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();
        processes.set_directory(id, PathBuf::from("/a/src"));

        // when
        let moved = processes.move_pinned(id, checkout("b", "/b").unwrap());

        // then
        assert_eq!((moved, processes.get(id).unwrap().pinned()), (true, Some(Path::new("/a/src"))));
    }

    #[test]
    fn should_report_no_move_for_same_checkout() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(checkout("a", "/a"), |_| Ok::<_, ()>(())).unwrap();

        // when
        let moved = processes.move_to(id, checkout("a", "/a").unwrap());

        // then
        assert!(!moved);
    }

    #[test]
    fn should_remember_last_directory() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(None, |_| Ok::<_, ()>(())).unwrap();
        processes.set_directory(id, PathBuf::from("/a"));

        // when
        processes.set_directory(id, PathBuf::from("/b/src"));

        // then
        assert_eq!(processes.get(id).unwrap().directory(), Some(Path::new("/b/src")));
    }

    #[test]
    fn should_forget_closed_process() {
        // given
        let mut processes: Processes<()> = Processes::default();
        let id = processes.start(None, |_| Ok::<_, ()>(())).unwrap();

        // when
        let closed = processes.close(id).map(|process| process.id);

        // then
        assert_eq!((closed, processes.get(id).is_none(), processes.set_kind(id, ProcessKind::ClaudeCode)), (Some(id), true, false));
    }
}
