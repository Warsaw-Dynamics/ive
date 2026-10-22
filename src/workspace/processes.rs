//! Starting, showing and ending processes, and telling terminals running Claude Code from plain ones.

#[cfg(unix)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alacritty_terminal::event::Event;
use gpui::{Context, SharedString, Task, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::process::{ProcessId, ProcessKind, RunLife, RunOrigin};
#[cfg(unix)]
use crate::process_directory::directory_of;
use crate::project::{CheckoutRef, ProjectId};
use crate::tabs::TabId;
use crate::progress_bar;
use crate::terminal::{Launch, Progress, Source, TerminalState, TerminalView};
#[cfg(unix)]
use crate::terminal::{DirectoryReport, DirectoryTracker, Sample};

/// Which adopted terminals get a tab.
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdoptedTabs {
    /// every one: no tabs were saved yet
    Each,
    /// those the saved tabs name, opened with them
    AsSaved,
}

/// How often the foreground program of each terminal is checked.
const FOREGROUND_POLL: Duration = Duration::from_secs(1);

/// How a process's Stop begins; either way SIGKILL follows while it is still there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramEnding {
    /// a hang-up and SIGTERM, then SIGKILL after [`STOP_GRACE`]
    Stop,
    /// SIGKILL at once
    Kill,
}

/// How long a stopped process has to go after its hang-up and SIGTERM before SIGKILL.
#[cfg(unix)]
const STOP_GRACE: Duration = Duration::from_secs(5);
/// How long a process may take to go after SIGKILL before IVE gives up on it.
#[cfg(unix)]
const KILL_GRACE: Duration = Duration::from_secs(2);
/// How often a stopping process is checked for being gone.
#[cfg(unix)]
const STOP_POLL: Duration = Duration::from_millis(100);
/// Session labels naming the checkout a hosted terminal belongs to.
const PROJECT_LABEL: &str = "ive.project";
const CHECKOUT_LABEL: &str = "ive.checkout";
/// Session labels naming the configuration a hosted run comes from.
const RUN_FOLDER_LABEL: &str = "ive.run.folder";
const RUN_KEY_LABEL: &str = "ive.run.key";
const RUN_NAME_LABEL: &str = "ive.run.name";
const RUN_URL_LABEL: &str = "ive.run.url";

impl Workspace {
    /// Starts a login shell in `checkout` (the home folder for `None`) and shows it in a new tab.
    pub fn open_terminal_in(&mut self, checkout: Option<CheckoutRef>, window: &mut Window, cx: &mut Context<Self>) -> TabId {
        let directory = checkout
            .as_ref()
            .map(|checkout| checkout.path.clone())
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("/"));
        self.open_terminal(checkout, &directory, Vec::new(), window, cx)
    }

    /// Starts a login shell in `directory`, listed under `checkout`, and shows it in a new tab; a hosted one also
    /// carries `labels`.
    pub(super) fn open_terminal(
        &mut self,
        checkout: Option<CheckoutRef>,
        directory: &Path,
        labels: Vec<(String, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> TabId {
        // hosted when this IVE owns the hosted terminals, so the shell outlives IVE
        #[cfg(unix)]
        let source = match &self.hosted {
            Some(terminals) => Source::Hosted { terminals, launch: Launch::shell(), labels: [session_labels(checkout.as_ref()), labels].concat() },
            None => Source::InProcess(Launch::shell()),
        };
        #[cfg(not(unix))]
        let source = {
            drop(labels);
            Source::InProcess(Launch::shell())
        };
        let started = self.processes.start(checkout, |id| TerminalView::open(id, directory, source, window, cx));
        let tab = match started {
            Ok(id) => {
                self.tree_changed(cx);
                Tab::Process(id)
            }
            Err(error) => Tab::Failed {
                title: "Terminal".to_owned(),
                error: format!("Cannot start a shell in {}: {error}", directory.display()).into(),
            },
        };
        self.open_tab(tab, window, cx)
    }

    /// Reattaches every hosted terminal still running, oldest first, under its checkout when its project is open
    /// (its main checkout when the checkout's folder is gone), else as a home terminal. Each gets a tab when no tabs were saved (a first start);
    /// otherwise the saved tabs say which do. Returns the processes by session id.
    #[cfg(unix)]
    pub(super) fn adopt_sessions(&mut self, tabs: AdoptedTabs, window: &mut Window, cx: &mut Context<Self>) -> HashMap<String, ProcessId> {
        let mut adopted = HashMap::new();
        let mut orphans = Vec::new();
        let Some(terminals) = self.hosted.take() else { return adopted };
        for session in terminals.live() {
            let checkout = checkout_from_labels(&session.labels, |project| self.project(project).map(|open| open.main_checkout().path.clone()));
            let source = Source::Adopt { terminals: &terminals, session: &session };
            let open = |id| TerminalView::open(id, &session.cwd, source, window, cx);
            let orphaned = orphaned_run(&session.labels, checkout.as_ref());
            let started = match run_origin_from_labels(&session.labels) {
                Some(origin) => self.processes.start_run(checkout, origin, |id| open(id).map(|mut view| {
                    view.keep_after_exit();
                    view
                })),
                None => self.processes.start(checkout, open),
            };
            match started {
                Ok(id) if orphaned => orphans.push(id),
                Ok(id) => {
                    if super::restart::is_restart_terminal(&session.labels) {
                        self.restart_terminal = Some(id);
                    }
                    adopted.insert(session.id.clone(), id);
                    if tabs == AdoptedTabs::Each {
                        self.open_tab(Tab::Process(id), window, cx);
                    }
                }
                Err(error) => eprintln!("ive: cannot reattach terminal {}: {error}", session.id),
            }
        }
        self.hosted = Some(terminals);
        self.end_orphaned_runs(orphans, cx);
        adopted
    }

    /// Processes whose shell outlives IVE and still runs, which quitting leaves running.
    pub fn hosted_process_count(&self) -> usize {
        self.processes.iter().filter(|process| process.is_live() && process.view.is_hosted()).count()
    }

    /// Ends runs whose worktree is gone: their program, row and tab.
    pub(super) fn end_orphaned_runs(&mut self, orphans: Vec<ProcessId>, cx: &mut Context<Self>) {
        for id in orphans {
            if let Some(process) = self.processes.close(id) {
                process.view.hang_up();
            }
            if let Some(tab) = self.tabs.find(|tab| tab.shows_process(id)) {
                self.tabs.close(tab);
            }
        }
        self.tree_changed(cx);
    }

    /// MCP `send_text`: types `text` into the tab, as on the keyboard. Parts add arms for their editable tabs.
    pub fn send_text_to_tab(&mut self, tab: TabId, text: &str, cx: &mut Context<Self>) -> Result<(), String> {
        match self.tabs.get(tab) {
            Some(Tab::Scratch(scratch)) => scratch.type_text(text, cx),
            Some(Tab::Document(file)) => {
                file.cast(codeview::panel::Cast::Type(text.to_owned()), cx);
                Ok(())
            }
            Some(Tab::Diff(diff)) => {
                let panel = diff.editing_panel(cx).ok_or_else(|| format!("tab {} shows a diff that cannot be edited: only a diff of uncommitted changes can", tab.value()))?;
                panel.update(cx, |panel, cx| panel.handle(codeview::panel::Cast::Type(text.to_owned()), cx));
                Ok(())
            }
            _ => self.send_text_to_process(tab, text),
        }
    }

    /// Types `text` into the process tab `tab` shows, as on the keyboard; a trailing `\r` presses Enter.
    pub fn send_text_to_process(&self, tab: TabId, text: &str) -> Result<(), String> {
        let number = tab.value();
        match self.tabs.get(tab) {
            Some(Tab::Process(id)) => {
                let process = self.processes.get(*id).ok_or_else(|| format!("tab {number} has ended"))?;
                process.view.send_text(text);
                Ok(())
            }
            Some(other) => Err(format!("tab {number} is a {}, not a process", other.kind())),
            None => Err(format!("no tab {number}; see list_tabs")),
        }
    }

    /// Shows the process's tab, opening one when it was closed.
    pub fn show_process(&mut self, id: ProcessId, window: &mut Window, cx: &mut Context<Self>) {
        match self.tabs.find(|tab| tab.shows_process(id)) {
            Some(tab) => self.activate_tab(tab, window, cx),
            None if self.processes.get(id).is_some() => {
                self.open_tab(Tab::Process(id), window, cx);
            }
            None => {}
        }
    }

    /// The one way IVE ends a process: hangs up its shell, hosted or not, then removes it. Dropping a hosted process
    /// without this leaves its shell running.
    pub fn end_process(&mut self, id: ProcessId, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(process) = self.processes.get(id) {
            process.view.hang_up();
        }
        self.remove_process(id, None, window, cx);
    }

    /// Stop (Ctrl+Shift+Q, a process tab's menu, a process row): ends process `id` for good. Its tab closes at once
    /// and its row reads "stopping"; the shell or program gets a hang-up and SIGTERM to its group and the foreground
    /// program's (Kill: SIGKILL at once), then SIGKILL after [`STOP_GRACE`]. Once it is gone its row goes; still
    /// there [`KILL_GRACE`] after SIGKILL, IVE forgets it and opens a tab saying so.
    pub fn stop_process(&mut self, id: ProcessId, ending: ProgramEnding, window: &mut Window, cx: &mut Context<Self>) {
        let Some(process) = self.processes.get(id).filter(|_| !self.processes.is_closing(id)) else {
            return;
        };
        process.view.hang_up();
        #[cfg(unix)]
        let targets = crate::process_stop::Targets::new(process.view.program_pid(), process.view.foreground().and_then(|probe| probe.program_group()));
        #[cfg(not(unix))]
        let targets: Option<()> = None;
        let Some(targets) = targets else {
            // nothing to wait for: the hang-up is all there is
            return self.remove_process(id, None, window, cx);
        };
        self.processes.mark_closing(id);
        if let Some(tab) = self.tabs.find(|tab| tab.shows_process(id)) {
            self.tabs.close(tab);
        }
        #[cfg(unix)]
        self.supervise_stop(id, targets, ending, window, cx);
        self.tree_changed(cx);
        self.tab_shown(window, cx);
    }

    /// Signals `targets` as `ending` says and waits for them to go: SIGKILL after [`STOP_GRACE`], then
    /// [`stop_finished`](Self::stop_finished).
    #[cfg(unix)]
    fn supervise_stop(&mut self, id: ProcessId, targets: crate::process_stop::Targets, ending: ProgramEnding, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.processes.get(id).map(|process| process.title()).unwrap_or_default();
        let grace = match ending {
            ProgramEnding::Stop => {
                targets.signal(libc::SIGTERM);
                STOP_GRACE
            }
            ProgramEnding::Kill => Duration::ZERO,
        };
        cx.spawn_in(window, async move |workspace, cx| {
            let mut gone = wait_gone(&targets, grace, cx).await;
            if !gone {
                targets.signal(libc::SIGKILL);
                gone = wait_gone(&targets, KILL_GRACE, cx).await;
            }
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.stop_finished(id, title, targets.pid(), gone, window, cx));
        })
        .detach();
    }

    /// The end of a Stop: the process is removed, as its exit would; one that outlived SIGKILL is forgotten, and a
    /// tab says so.
    #[cfg(unix)]
    fn stop_finished(&mut self, id: ProcessId, title: String, pid: libc::pid_t, gone: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.processes.get(id).is_some() {
            self.remove_process(id, None, window, cx);
        }
        if gone {
            return;
        }
        let error = format!("Process {pid} did not stop: it still ran {} s after SIGKILL. IVE no longer manages it.", KILL_GRACE.as_secs());
        self.open_tab(Tab::Failed { title, error: error.into() }, window, cx);
    }

    /// Removes the process; its tab closes, or, with `failure`, shows why the shell ended.
    fn remove_process(&mut self, id: ProcessId, failure: Option<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.processes.get(id).map(|process| process.title()).unwrap_or_default();
        let was_live_run = self.processes.get(id).is_some_and(|process| process.run.is_some() && process.is_live());
        self.processes.close(id);
        if was_live_run {
            self.run_removed(id, window, cx);
        }
        if let Some(tab) = self.tabs.find(|tab| tab.shows_process(id)) {
            match failure {
                Some(error) => {
                    self.tabs.replace(tab, Tab::Failed { title, error });
                }
                None => {
                    self.tabs.close(tab);
                }
            }
        }
        self.tree_changed(cx);
        self.settle_open_hub(cx);
        self.tab_shown(window, cx);
    }

    /// Applies events from process `id`'s terminal. Output in a background tab does not redraw the window; a changed
    /// title or progress does, since the tree and the tab row show them.
    pub fn handle_process_events(&mut self, id: ProcessId, events: Vec<Event>, window: &mut Window, cx: &mut Context<Self>) {
        let shown = self.tabs.active().is_some_and(|tab| tab.shows_process(id));
        let detached = self.tabs.find(|tab| tab.shows_process(id)).filter(|tab| self.tabs.is_detached(*tab));
        let Some(process) = self.processes.get_mut(id) else {
            return;
        };
        let state = process.view.handle_events(events, cx);
        let (title, progress) = (process.view.title(), process.view.progress());
        // a run keeps its configuration's name, whatever title its program sets
        let renamed = process.run.is_none() && self.processes.set_title(id, title);
        let progressed = self.processes.set_progress(id, progress);
        if progressed {
            self.tick_progress(cx);
        }
        if renamed {
            self.retitle_hub_preview(cx);
        }
        match state {
            TerminalState::Running if renamed || progressed => self.tree_changed(cx),
            TerminalState::Running if shown => cx.notify(),
            TerminalState::Running => detached.into_iter().for_each(|tab| self.redraw_tab_window(tab, cx)),
            TerminalState::Closed => self.remove_process(id, None, window, cx),
            TerminalState::Failed(error) => self.remove_process(id, Some(error), window, cx),
            TerminalState::Exited(code) => {
                self.run_exited(id, code, window, cx);
                // Stop ends a run for good: its row goes rather than stay greyed
                if self.processes.is_closing(id) {
                    self.remove_process(id, None, window, cx);
                }
            }
        }
    }

    /// Redraws every [`progress_bar::FRAME`] while some process reports indeterminate progress, so its bar moves.
    /// The ticker ends itself once none does, however the busy ones ended.
    fn tick_progress(&mut self, cx: &mut Context<Self>) {
        if self.progress_ticker.is_some() || !self.has_busy_process() {
            return;
        }
        self.progress_ticker = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(progress_bar::FRAME).await;
                let ticking = this.update(cx, |workspace, cx| {
                    let busy = workspace.has_busy_process();
                    if busy {
                        cx.notify();
                    } else {
                        workspace.progress_ticker = None;
                    }
                    busy
                });
                if !ticking.unwrap_or(false) {
                    return;
                }
            }
        }));
    }

    fn has_busy_process(&self) -> bool {
        self.processes.iter().any(|process| process.progress() == Progress::Indeterminate)
    }

    fn apply_kinds(&mut self, kinds: Vec<(ProcessId, ProcessKind)>, cx: &mut Context<Self>) {
        let mut changed = false;
        for (id, kind) in kinds {
            changed |= self.processes.set_kind(id, kind);
        }
        if changed {
            self.tree_changed(cx);
        }
    }
}

/// Once a second, reads each terminal's foreground program on the background executor: updates the kind of the
/// processes that started or ended Claude Code, and hands on the working directories that changed.
#[cfg(unix)]
pub(super) fn poll_foreground(cx: &mut Context<Workspace>) -> Option<Task<()>> {
    Some(cx.spawn(async move |this, cx| {
        let mut trackers: HashMap<ProcessId, DirectoryTracker> = HashMap::new();
        loop {
            cx.background_executor().timer(FOREGROUND_POLL).await;
            let probes = this.update(cx, |workspace, _cx| {
                let probe = |process: &crate::process::Process| process.view.foreground().map(|probe| (process.id, probe, process.view.directory()));
                workspace.processes.iter().filter_map(probe).collect::<Vec<_>>()
            });
            let Ok(probes) = probes else {
                return;
            };
            let (kinds, directories, kept) = cx
                .background_executor()
                .spawn(async move {
                    let kind = |probe: &crate::terminal::ForegroundProbe| {
                        if probe.runs_claude_code() { ProcessKind::ClaudeCode } else { ProcessKind::Terminal }
                    };
                    let kinds = probes.iter().map(|(id, probe, _)| (*id, kind(probe))).collect::<Vec<_>>();
                    let directories = observe_directories(&mut trackers, probes);
                    (kinds, directories, trackers)
                })
                .await;
            trackers = kept;
            let applied = this.update(cx, |workspace, cx| {
                workspace.apply_kinds(kinds, cx);
                workspace.apply_directories(directories, cx);
            });
            if applied.is_err() {
                return;
            }
        }
    }))
}

/// The processes whose working directory changed since the last poll, and where they work now; trackers of ended
/// processes are dropped. A directory that cannot be resolved (removed meanwhile) counts as no change.
#[cfg(unix)]
fn observe_directories(
    trackers: &mut HashMap<ProcessId, DirectoryTracker>,
    probes: Vec<(ProcessId, crate::terminal::ForegroundProbe, Option<DirectoryReport>)>,
) -> Vec<(ProcessId, PathBuf)> {
    trackers.retain(|id, _| probes.iter().any(|(probed, _, _)| probed == id));
    probes
        .into_iter()
        .filter_map(|(id, probe, report)| {
            let leader = probe.leader();
            let sample = Sample { leader, report, process_cwd: directory_of(leader) };
            let directory = trackers.entry(id).or_default().observe(sample)?;
            Some((id, directory.canonicalize().ok()?))
        })
        .collect()
}

/// Without Unix process groups IVE cannot tell what runs in a terminal: every process stays a terminal.
#[cfg(not(unix))]
pub(super) fn poll_foreground(_cx: &mut Context<Workspace>) -> Option<Task<()>> {
    None
}

/// Labels recording `checkout` in a hosted session, so the next IVE puts it back there.
pub(super) fn session_labels(checkout: Option<&CheckoutRef>) -> Vec<(String, String)> {
    checkout.map_or_else(Vec::new, |checkout| {
        vec![(PROJECT_LABEL.to_owned(), checkout.project.to_string()), (CHECKOUT_LABEL.to_owned(), checkout.path.display().to_string())]
    })
}

/// Labels recording which configuration a hosted run comes from, so the next IVE shows it as that run.
pub(super) fn run_labels(origin: &RunOrigin) -> Vec<(String, String)> {
    [
        (RUN_FOLDER_LABEL.to_owned(), origin.folder.display().to_string()),
        (RUN_KEY_LABEL.to_owned(), origin.key.clone()),
        (RUN_NAME_LABEL.to_owned(), origin.name.clone()),
    ]
    .into_iter()
    .chain(origin.url.iter().map(|url| (RUN_URL_LABEL.to_owned(), url.clone())))
    .collect()
}

/// A run whose recorded checkout is not the one it is put under: its worktree is gone (or its project closed), so
/// its folder is too.
pub(super) fn orphaned_run(labels: &[(String, String)], checkout: Option<&CheckoutRef>) -> bool {
    if run_origin_from_labels(labels).is_none() {
        return false;
    }
    let recorded = labels.iter().find(|(name, _)| name == CHECKOUT_LABEL).map(|(_, value)| PathBuf::from(value));
    checkout.is_none_or(|checkout| recorded.as_deref() != Some(checkout.path.as_path()))
}

/// The run a hosted session's labels record; `None` for a terminal.
pub(super) fn run_origin_from_labels(labels: &[(String, String)]) -> Option<RunOrigin> {
    let label = |key: &str| labels.iter().find(|(name, _)| name == key).map(|(_, value)| value.clone());
    Some(RunOrigin {
        folder: PathBuf::from(label(RUN_FOLDER_LABEL)?),
        key: label(RUN_KEY_LABEL)?,
        name: label(RUN_NAME_LABEL)?,
        url: label(RUN_URL_LABEL),
        life: RunLife::Running,
    })
}

/// The checkout a hosted session's labels record while its project is open: the recorded folder, or the project's
/// main checkout (`main_of`) once that folder is gone, as a refresh moves the terminals of a removed worktree.
fn checkout_from_labels(labels: &[(String, String)], main_of: impl Fn(&ProjectId) -> Option<PathBuf>) -> Option<CheckoutRef> {
    let label = |key: &str| labels.iter().find(|(name, _)| name == key).map(|(_, value)| value.as_str());
    let project = ProjectId::parse(label(PROJECT_LABEL)?).ok()?;
    let main = main_of(&project)?;
    let recorded = label(CHECKOUT_LABEL).map(PathBuf::from).filter(|path| path.is_dir());
    Some(CheckoutRef { project, path: recorded.unwrap_or(main) })
}

/// Waits up to `limit` for `targets` to be gone; whether they are.
#[cfg(unix)]
async fn wait_gone(targets: &crate::process_stop::Targets, limit: Duration, cx: &mut gpui::AsyncWindowContext) -> bool {
    let checks = limit.as_millis().div_ceil(STOP_POLL.as_millis()).max(1);
    for _ in 0..checks {
        if !targets.alive() {
            return true;
        }
        cx.background_executor().timer(STOP_POLL).await;
    }
    !targets.alive()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project::ProjectId;

    fn labels(project: &str, checkout: &str) -> Vec<(String, String)> {
        vec![(PROJECT_LABEL.to_owned(), project.to_owned()), (CHECKOUT_LABEL.to_owned(), checkout.to_owned())]
    }

    #[test]
    fn should_find_a_run_of_a_removed_worktree_orphaned() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().to_str().unwrap().to_owned();
        let removed = format!("{main}/.claude/worktrees/gone");
        let run_of = |checkout: &str| {
            let origin = RunOrigin { folder: PathBuf::from(format!("{checkout}/misc")), key: "app".to_owned(), name: "App".to_owned(), url: None, life: RunLife::Running };
            [labels("ive", checkout), run_labels(&origin)].concat()
        };
        let checkout_of = |labelled: &[(String, String)]| checkout_from_labels(labelled, |_| Some(dir.path().to_path_buf()));

        // when
        let orphaned = [run_of(&removed), run_of(&main), labels("ive", &removed)].map(|labelled| orphaned_run(&labelled, checkout_of(&labelled).as_ref()));

        // then
        assert_eq!(orphaned, [true, false, false]);
    }

    #[test]
    fn should_restore_run_origin_from_labels() {
        // given
        let origin = RunOrigin { folder: PathBuf::from("/work/misc"), key: "app".to_owned(), name: "MiscApplication".to_owned(), url: Some("http://127.0.0.1:4200/docs/".to_owned()), life: RunLife::Running };
        let labelled = [labels("ive", "/work"), run_labels(&origin)].concat();

        // when
        let restored = run_origin_from_labels(&labelled);

        // then
        assert_eq!(restored, Some(origin));
        assert_eq!(run_origin_from_labels(&labels("ive", "/work")), None);
    }

    #[test]
    fn should_label_session_with_project_and_checkout() {
        // given
        let checkout = CheckoutRef { project: ProjectId::new("ive"), path: PathBuf::from("/work/ive") };

        // when
        let labelled = session_labels(Some(&checkout));

        // then
        assert_eq!(labelled, labels("ive", "/work/ive"));
    }

    #[test]
    fn should_not_label_home_terminal() {
        // given
        let checkout = None;

        // when
        let labelled = session_labels(checkout);

        // then
        assert!(labelled.is_empty());
    }

    #[test]
    fn should_map_labels_of_open_project_to_checkout() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        // when
        let checkout = checkout_from_labels(&labels("ive", path), |_| Some(PathBuf::from("/work/ive")));

        // then
        assert_eq!(checkout, Some(CheckoutRef { project: ProjectId::new("ive"), path: dir.path().to_path_buf() }));
    }

    #[test]
    fn should_map_session_with_missing_checkout_folder_to_main_checkout() {
        // given
        let missing = labels("ive", "/nonexistent/worktree");

        // when
        let checkout = checkout_from_labels(&missing, |_| Some(PathBuf::from("/work/ive")));

        // then
        assert_eq!(checkout, Some(CheckoutRef { project: ProjectId::new("ive"), path: PathBuf::from("/work/ive") }));
    }

    #[test]
    fn should_map_session_of_closed_project_to_home() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let checkout = checkout_from_labels(&labels("ive", dir.path().to_str().unwrap()), |_| None);

        // then
        assert_eq!(checkout, None);
    }
}
