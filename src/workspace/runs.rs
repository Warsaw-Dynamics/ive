//! Run configurations in the workspace: the runs started from `.run.toml` files: Run, Restart, Stop,
//! Remove, `before` chains, and the tab asking for missing values.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{Context, Window};

use super::Workspace;
use super::processes::{run_labels, session_labels};
use crate::editor::Tab;
use crate::file_tree::RunState;
use crate::tabs::TabId;
use crate::process::{Process, ProcessId, RunId, RunLife, RunOrigin};
use crate::profile;
use crate::project::CheckoutRef;
use crate::run::chain::{Chain, Next, Step};
use crate::run::file::{ConfigKind, RunFile, VarDecl};
use crate::run::serve::{free_port, is_free, port_of};
use crate::run::values::{Builtins, PreparedKind, Resolved, Sources, VarsFile, prepare, resolve, steps_of};
use crate::overview::file_list::FileLayout;
use crate::run::configs_tab::{RunConfigsTab, RunFiles};
use crate::run::values_tab::{RunRequest, RunValuesTab, ShowRun, empty_fields};
use crate::run::wallet::{OsWallet, Wallet};
use crate::sidebar::rows::{RunBadge, RunRow, RunsView};
use crate::terminal::{Launch, Source, TerminalView};

/// How long a restarted server waits for its old run to let go of its port before it takes another.
const PORT_RELEASE_WAIT: Duration = Duration::from_secs(2);
const PORT_RELEASE_POLL: Duration = Duration::from_millis(25);

/// Runs still waiting for their `before` steps, which have no process yet.
#[derive(Default)]
pub(super) struct PendingRuns {
    runs: Vec<PendingRun>,
    next: u64,
    /// configurations whose values are being read (checkout, folder, key): a second Run meanwhile does nothing
    resolving: HashSet<(PathBuf, PathBuf, String)>,
}

/// What reading a run's values takes, owned, so it runs off the UI thread: the wallet may block on D-Bus or wait
/// for its unlock prompt.
pub(super) struct RunInputs {
    folder: PathBuf,
    key: String,
    checkout: PathBuf,
    /// the project's main checkout, naming its `[repo]` table in vars.toml
    project: PathBuf,
    vars: Option<PathBuf>,
}

/// What a run starts with: its file read now, its values and the built-ins.
pub(super) type RunValues = (RunFile, Resolved, Builtins);

/// A run whose `before` steps run first; it starts once each exited 0, or stays, failed, until removed.
struct PendingRun {
    id: u64,
    request: RunRequest,
    name: String,
    serve: bool,
    chain: Chain,
    failed: Option<String>,
    /// what it starts with once the steps are done, read when Run was clicked
    file: Arc<RunFile>,
    resolved: Resolved,
    builtins: Builtins,
}

/// What starting one configuration takes, read when Run was clicked.
struct Start<'a> {
    request: &'a RunRequest,
    key: &'a str,
    file: &'a RunFile,
    resolved: &'a Resolved,
    builtins: &'a Builtins,
}

impl Workspace {
    /// Run: reads `.run.toml` and the values again; asks for missing values in a tab, else starts the `before`
    /// steps and then the configuration. A live or waiting run of it is shown instead.
    pub fn request_run(&mut self, request: RunRequest, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = blocking_run(&self.runs_in(&request.checkout.path, &request.folder), &request.key) {
            if request.show == ShowRun::Tab {
                self.show_run(id, window, cx);
            }
            return;
        }
        let resolving = (request.checkout.path.clone(), request.folder.clone(), request.key.clone());
        if self.pending.resolving.contains(&resolving) {
            return;
        }
        self.pending.resolving.insert(resolving.clone());
        // what runs is what the editors show
        self.flush_edits(cx);
        let inputs = self.run_inputs(&request);
        let profile = profile::current().name().to_owned();
        let read = cx.background_executor().spawn(async move { read_run_values(&inputs, &OsWallet::new(&profile)) });
        cx.spawn_in(window, async move |workspace, cx| {
            let read = read.await;
            super::saves::saves_landed(&workspace, cx).await;
            let _ = workspace.update_in(cx, |workspace, window, cx| {
                workspace.pending.resolving.remove(&resolving);
                workspace.continue_run(request, read, window, cx);
            });
        })
        .detach();
    }

    /// Run, once its values are read: asks for missing ones in a tab, else starts the steps and the configuration.
    pub(super) fn continue_run(&mut self, request: RunRequest, read: Result<RunValues, String>, window: &mut Window, cx: &mut Context<Self>) {
        match read {
            Ok((file, resolved, builtins)) if resolved.missing.is_empty() => self.start_chain(request, Arc::new(file), resolved, builtins, window, cx),
            Ok((file, resolved, _)) => self.ask_values(request, &file, &resolved.missing, window, cx),
            Err(error) => self.run_failed(&request, error, window, cx),
        }
    }

    /// What reading `request`'s values takes, for [`read_run_values`] off the UI thread.
    pub(super) fn run_inputs(&self, request: &RunRequest) -> RunInputs {
        let project = self.project(&request.checkout.project).map_or_else(|| request.checkout.path.clone(), |project| project.main_checkout().path.clone());
        RunInputs {
            folder: request.folder.clone(),
            key: request.key.clone(),
            checkout: request.checkout.path.clone(),
            project,
            vars: vars_file().ok().map(|vars| vars.path().to_path_buf()),
        }
    }

    /// Restart (and Start of an exited run): ends it and runs its configuration again, its tab brought to the front
    /// or not as `show` says. A server starts again once its port is free, up to [`PORT_RELEASE_WAIT`], so it keeps
    /// its link.
    pub fn restart_run_with(&mut self, id: RunId, show: ShowRun, window: &mut Window, cx: &mut Context<Self>) {
        let Some(request) = self.request_of(id).map(|request| RunRequest { show, ..request }) else { return };
        let port = self.serving_port(id);
        self.remove_run(id, window, cx);
        let Some(port) = port else { return self.request_run(request, window, cx) };
        cx.spawn_in(window, async move |workspace, cx| {
            let deadline = Instant::now() + PORT_RELEASE_WAIT;
            while !is_free(port) && Instant::now() < deadline {
                cx.background_executor().timer(PORT_RELEASE_POLL).await;
            }
            let _ = workspace.update_in(cx, |workspace, window, cx| workspace.request_run(request, window, cx));
        })
        .detach();
    }

    /// The port live server run `id` listens on.
    fn serving_port(&self, id: RunId) -> Option<u16> {
        let RunId::Process(process) = id else { return None };
        let process = self.processes.get(process).filter(|process| process.is_live())?;
        process.run.as_ref()?.url.as_deref().and_then(port_of)
    }

    /// Stop: SIGTERM to the program's group (SIGKILL 5 s later); a waiting run stops waiting and fails.
    pub fn stop_run(&mut self, id: RunId, cx: &mut Context<Self>) {
        match id {
            RunId::Process(process) => {
                let Some(pid) = self.processes.get(process).filter(|process| process.is_live()).map(|process| process.view.program_pid()) else {
                    return;
                };
                self.processes.mark_stopping(process);
                match pid {
                    #[cfg(unix)]
                    Some(pid) => crate::run::signal::stop_group(pid),
                    _ => self.processes.get(process).iter().for_each(|process| {
                        process.view.hang_up();
                        #[cfg(windows)]
                        process.view.job().into_iter().for_each(crate::run::signal::stop_job);
                    }),
                }
            }
            RunId::Pending(pending) => {
                if let Some(run) = self.pending.runs.iter_mut().find(|run| run.id == pending) {
                    run.chain.cancel();
                    run.failed = Some("stopped".to_owned());
                }
            }
        }
        self.tree_changed(cx);
    }

    /// Remove: ends the run, its row and its tab.
    pub fn remove_run(&mut self, id: RunId, window: &mut Window, cx: &mut Context<Self>) {
        match id {
            RunId::Process(process) => self.end_process(process, window, cx),
            RunId::Pending(pending) => {
                self.pending.runs.retain(|run| run.id != pending);
                self.tree_changed(cx);
            }
        }
    }

    /// Shows run `id`: its tab, or the step a waiting run waits for.
    pub fn show_run(&mut self, id: RunId, window: &mut Window, cx: &mut Context<Self>) {
        let process = match id {
            RunId::Process(process) => Some(process),
            RunId::Pending(pending) => {
                let waiting = self.pending.runs.iter().find(|run| run.id == pending).and_then(|run| run.chain.waiting_on());
                waiting.and_then(|value| self.processes.id_of(value))
            }
        };
        let Some(process) = process else { return };
        if let Some(checkout) = self.processes.get(process).and_then(|process| process.checkout.clone()) {
            self.reveal_process_of(&checkout, cx);
        }
        self.show_process(process, window, cx);
    }

    /// Shows the checkout's Run configurations tab, opening it when needed; its run files are read again when due.
    pub fn show_run_configs(&mut self, checkout: &CheckoutRef, window: &mut Window, cx: &mut Context<Self>) {
        self.index_checkout(checkout, cx);
        match self.tabs.find(|tab| matches!(tab, Tab::RunConfigs(open) if open.checkout == *checkout)) {
            Some(open) => self.activate_tab(open, window, cx),
            None => {
                let tab = RunConfigsTab::new(checkout.clone(), window, cx);
                self.open_tab(Tab::RunConfigs(tab), window, cx);
            }
        }
    }

    pub fn set_run_configs_layout(&mut self, id: TabId, layout: FileLayout, cx: &mut Context<Self>) {
        if let Some(Tab::RunConfigs(tab)) = self.tabs.get_mut(id) {
            tab.layout = layout;
            cx.notify();
        }
    }

    /// Folds or unfolds the tree folder at `path` (checkout-relative) of Run configurations tab `id`.
    pub fn toggle_run_configs_folder(&mut self, id: TabId, path: String, cx: &mut Context<Self>) {
        if let Some(Tab::RunConfigs(tab)) = self.tabs.get_mut(id) {
            if !tab.folded.remove(&path) {
                tab.folded.insert(path);
            }
            cx.notify();
        }
    }

    /// The run files the hub index read in `checkout`, by checkout-relative folder; `None` before its first read.
    pub fn run_files(&self, checkout: &Path) -> Option<RunFiles> {
        self.hub_index.files(checkout).map(|read| read.configs.clone())
    }

    /// Why the hub index could not read `checkout`, while its last read failed.
    pub fn run_files_failure(&self, checkout: &Path) -> Option<String> {
        self.hub_index.failure(checkout).map(str::to_owned)
    }

    /// `folder`'s `.run.toml` as the tree read it; `None` while it is missing, broken or not read.
    pub fn run_file_of(&self, checkout: &CheckoutRef, folder: &Path) -> Option<Arc<RunFile>> {
        let files = self.open_project(&checkout.project)?.files(&checkout.path)?;
        match &files.entry(folder)?.run {
            RunState::Loaded(file) => Some(file.clone()),
            RunState::None | RunState::Broken(_) => None,
        }
    }

    /// Each configuration's command with built-ins and plain values filled in; secrets stay `${name}`, and the
    /// wallet is not asked.
    pub(super) fn shown_commands(&self, checkout: &CheckoutRef, folder: &Path) -> HashMap<String, String> {
        let Some(file) = self.run_file_of(checkout, folder) else { return HashMap::new() };
        let request = RunRequest { checkout: checkout.clone(), folder: folder.to_path_buf(), key: String::new(), show: ShowRun::Tab };
        let builtins = builtins(&request);
        let doc = vars_file().and_then(|vars| vars.load()).unwrap_or_default();
        let project = self.project(&checkout.project).map_or_else(|| checkout.path.clone(), |project| project.main_checkout().path.clone());
        let sources = Sources { doc: &doc, wallet: &NoWallet, project: &project };
        let shown = |key: &str| {
            let resolved = resolve(&file, key, &sources, &builtins).ok()?;
            prepare(&file, key, &resolved, &builtins).ok().map(|prepared| prepared.shown)
        };
        file.configs.iter().filter_map(|config| shown(&config.key).map(|text| (config.key.clone(), text))).collect()
    }

    /// The values tab's Run: saves each value (a secret to the wallet), closes the tab and runs.
    pub fn submit_run_values(&mut self, request: &RunRequest, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.find(|tab| matches!(tab, Tab::RunValues(values) if values.shows(request))) else { return };
        let values: Vec<(VarDecl, String)> = match self.tabs.get(tab) {
            Some(Tab::RunValues(values)) => values.fields.iter().map(|field| (field.var.clone(), field.input.read(cx).text().to_owned())).collect(),
            _ => return,
        };
        let empty = empty_fields(&values);
        if !empty.is_empty() {
            return self.show_values_error(tab, format!("Fill in {}: an empty value would be remembered", empty.join(", ")), cx);
        }
        let profile = profile::current().name().to_owned();
        let vars = vars_file();
        let save = cx.background_executor().spawn(async move { save_values(vars?, &values, &OsWallet::new(&profile)) });
        let request = request.after_values();
        cx.spawn_in(window, async move |workspace, cx| {
            let saved = save.await;
            let _ = workspace.update_in(cx, |workspace, window, cx| match saved {
                Ok(()) => {
                    workspace.close_tab(tab, window, cx);
                    workspace.request_run(request, window, cx);
                }
                Err(error) => workspace.show_values_error(tab, error, cx),
            });
        })
        .detach();
    }

    fn show_values_error(&mut self, tab: TabId, error: String, cx: &mut Context<Self>) {
        self.update_tab(tab, |tab, cx| {
            if let Tab::RunValues(values) = tab {
                values.error = Some(error);
                cx.notify();
            }
        }, cx);
    }

    /// Run `id`'s program exited with `code`: the run stays, greyed, with its output; a chain waiting for it moves on.
    pub(super) fn run_exited(&mut self, id: ProcessId, code: i32, window: &mut Window, cx: &mut Context<Self>) {
        self.processes.set_exited(id, code);
        let stopped = self.processes.get(id).and_then(|process| process.run.as_ref()).is_some_and(|origin| matches!(origin.life, RunLife::Exited { stopped: true, .. }));
        let waiting: Vec<u64> = self.pending.runs.iter().filter(|run| run.chain.waiting_on() == Some(id.value())).map(|run| run.id).collect();
        for pending in waiting {
            let next = self.pending.runs.iter_mut().find(|run| run.id == pending).and_then(|run| run.chain.on_exit(id.value(), code, stopped));
            if let Some(next) = next {
                self.advance(pending, next, window, cx);
            }
        }
        self.tree_changed(cx);
    }

    /// Process `id` was removed before it exited: a chain waiting for it fails, rather than wait forever.
    pub(super) fn run_removed(&mut self, id: ProcessId, window: &mut Window, cx: &mut Context<Self>) {
        let waiting: Vec<(u64, Next)> = self.pending.runs.iter_mut().filter_map(|run| run.chain.on_removed(id.value()).map(|next| (run.id, next))).collect();
        for (pending, next) in waiting {
            self.advance(pending, next, window, cx);
        }
    }

    /// Opens (or shows) the tab asking for `missing`'s values.
    fn ask_values(&mut self, request: RunRequest, file: &RunFile, missing: &[String], window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.find(|tab| matches!(tab, Tab::RunValues(values) if values.shows(&request))) {
            return self.activate_tab(tab, window, cx);
        }
        let name = file.configs.iter().find(|config| config.key == request.key).map_or_else(|| request.key.clone(), |config| config.name.clone());
        let vars = missing.iter().filter_map(|name| file.vars.iter().find(|var| &var.name == name).cloned()).collect();
        let tab = RunValuesTab::new(request, name, vars, window, cx);
        self.open_tab(Tab::RunValues(tab), window, cx);
    }

    /// Starts `request`'s `before` steps, or the configuration at once without any.
    fn start_chain(&mut self, request: RunRequest, file: Arc<RunFile>, resolved: Resolved, builtins: Builtins, window: &mut Window, cx: &mut Context<Self>) {
        self.remove_exited(&request, &request.key, window, cx);
        let Some(config) = file.configs.iter().find(|config| config.key == request.key) else {
            return self.run_failed(&request, format!("no configuration {} in .run.toml", request.key), window, cx);
        };
        let steps: Vec<Step> = steps_of(&file, &request.key)
            .iter()
            .filter_map(|key| file.configs.iter().find(|config| &config.key == key))
            .map(|config| Step { key: config.key.clone(), name: config.name.clone() })
            .collect();
        if steps.is_empty() {
            let start = Start { request: &request, key: &request.key, file: &file, resolved: &resolved, builtins: &builtins };
            match self.launch(&start, window, cx) {
                Ok(id) => self.launched(&request, id, window, cx),
                Err(error) => self.run_failed(&request, error, window, cx),
            }
            return;
        }
        let id = self.pending.next;
        self.pending.next += 1;
        let (name, serve) = (config.name.clone(), matches!(config.kind, ConfigKind::Serve(_)));
        let mut chain = Chain::new(steps);
        let next = chain.next();
        self.pending.runs.push(PendingRun { id, request, name, serve, chain, failed: None, file, resolved, builtins });
        self.advance(id, next, window, cx);
    }

    /// Does what a waiting run's chain says next: start (or wait for) a step, start the run, or fail.
    fn advance(&mut self, pending: u64, next: Next, window: &mut Window, cx: &mut Context<Self>) {
        // a stopped or failed run starts nothing more
        let Some(index) = self.pending.runs.iter().position(|run| run.id == pending && run.failed.is_none()) else { return };
        match next {
            Next::Start(step) => {
                let run = &self.pending.runs[index];
                let request = run.request.clone();
                let (file, resolved, builtins) = (run.file.clone(), run.resolved.clone(), run.builtins.clone());
                let live = self.processes.runs_in(&request.checkout.path, &request.folder).into_iter().find(|process| process.is_live() && process.run.as_ref().is_some_and(|origin| origin.key == step.key)).map(|process| process.id);
                let started = match live {
                    Some(id) => Ok(id),
                    None => {
                        self.remove_exited(&request, &step.key, window, cx);
                        let start = Start { request: &request, key: &step.key, file: &file, resolved: &resolved, builtins: &builtins };
                        // a before step shows only as its row
                        self.launch(&start, window, cx)
                    }
                };
                let Some(run) = self.pending.runs.iter_mut().find(|run| run.id == pending) else { return };
                match started {
                    Ok(id) => run.chain.wait_for(id.value()),
                    Err(error) => run.failed = Some(format!("before: {} cannot start: {error}", step.name)),
                }
            }
            Next::Launch => {
                let run = self.pending.runs.remove(index);
                let start = Start { request: &run.request, key: &run.request.key, file: &run.file, resolved: &run.resolved, builtins: &run.builtins };
                match self.launch(&start, window, cx) {
                    Ok(id) => self.launched(&run.request, id, window, cx),
                    Err(error) => self.run_failed(&run.request, error, window, cx),
                }
            }
            Next::Fail(reason) => self.pending.runs[index].failed = Some(reason),
        }
        self.tree_changed(cx);
    }

    /// Shows the new run's tab, unless the request asked to stay where it is.
    fn launched(&mut self, request: &RunRequest, id: ProcessId, window: &mut Window, cx: &mut Context<Self>) {
        match request.show {
            ShowRun::Tab => self.show_run(RunId::Process(id), window, cx),
            ShowRun::Stay => self.tree_changed(cx),
        }
    }

    /// Starts one configuration's program as a run.
    fn launch(&mut self, start: &Start, window: &mut Window, cx: &mut Context<Self>) -> Result<ProcessId, String> {
        let prepared = prepare(start.file, start.key, start.resolved, start.builtins)?;
        if !prepared.cwd.is_dir() {
            return Err(format!("{} is not a folder", prepared.cwd.display()));
        }
        let config = start.file.configs.iter().find(|config| config.key == start.key).ok_or_else(|| format!("no configuration {}", start.key))?;
        let (launch, url) = match prepared.kind {
            PreparedKind::Command(command) => (Launch::command(&command, prepared.launch_env), None),
            PreparedKind::Serve { root, port, spec } => {
                let port = free_port(port)?;
                (crate::run::serve::launch(&root, port, &spec, &start.request.folder)?, Some(crate::run::serve::address(port, &spec.open)))
            }
        };
        let origin = RunOrigin { folder: start.request.folder.clone(), key: start.key.to_owned(), name: config.name.clone(), url, life: RunLife::Running };
        let checkout = start.request.checkout.clone();
        self.worktree_use.used(&checkout.path, crate::age::Clock::system().now());
        let source = match &self.hosted {
            Some(terminals) => Source::Hosted { terminals, launch, labels: [session_labels(Some(&checkout)), run_labels(&origin)].concat() },
            None => Source::InProcess(launch),
        };
        let cwd = prepared.cwd;
        let id = self
            .processes
            .start_run(Some(checkout), origin, |id| {
                TerminalView::open(id, &cwd, source, window, cx).map(|mut view| {
                    view.keep_after_exit();
                    view
                })
            })
            .map_err(|error| format!("cannot start {} ({} in {}): {error}", config.name, written_command(config), cwd.display()))?;
        self.tree_changed(cx);
        Ok(id)
    }

    /// Removes the exited runs of `key` in `request`'s folder: one row per configuration.
    fn remove_exited(&mut self, request: &RunRequest, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        for id in exited_runs_of(&self.runs_in(&request.checkout.path, &request.folder), key) {
            self.remove_run(id, window, cx);
        }
    }

    /// A run that cannot start says why in a tab of its own.
    fn run_failed(&mut self, request: &RunRequest, error: String, window: &mut Window, cx: &mut Context<Self>) {
        self.open_tab(Tab::Failed { title: format!("Run {}", request.key), failure: run_failure(request, error) }, window, cx);
    }

    /// The request that runs `id`'s configuration again.
    fn request_of(&self, id: RunId) -> Option<RunRequest> {
        match id {
            RunId::Process(process) => {
                let process = self.processes.get(process)?;
                let origin = process.run.as_ref()?;
                Some(RunRequest { checkout: process.checkout.clone()?, folder: origin.folder.clone(), key: origin.key.clone(), show: ShowRun::Tab })
            }
            RunId::Pending(pending) => self.pending.runs.iter().find(|run| run.id == pending).map(|run| run.request.clone()),
        }
    }

    /// Shows the project, checkout and Processes folder that list the run.
    fn reveal_process_of(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        let Some(open) = self.open_project_mut(&checkout.project) else { return };
        open.reveal(&checkout.path);
        // runs follow the paged processes, so no page needs to grow
        open.reveal_process(&checkout.path, 0);
        let folders = open.folders_to_read();
        self.read_folders(folders, cx);
    }
}

impl RunsView for Workspace {
    fn runs_in(&self, checkout: &Path, folder: &Path) -> Vec<RunRow> {
        let pending = self.pending.runs.iter().rev().filter(|run| run.request.checkout.path == checkout && run.request.folder == folder).map(pending_row);
        pending.chain(self.processes.runs_in(checkout, folder).into_iter().filter_map(run_row)).collect()
    }

    fn runs_of(&self, checkout: &Path) -> Vec<RunRow> {
        let started = self.processes.runs_of(checkout).into_iter().filter_map(run_row);
        let pending = self.pending.runs.iter().filter(|run| run.request.checkout.path == checkout).map(pending_row);
        started.chain(pending).collect()
    }
}

/// A wallet that has nothing: what rows show never needs a secret.
struct NoWallet;

impl Wallet for NoWallet {
    fn get(&self, _id: &str) -> Result<Option<String>, String> {
        Ok(None)
    }

    fn set(&self, _id: &str, _secret: &str) -> Result<(), String> {
        Err("rows never store secrets".to_owned())
    }
}

/// The `.run.toml` of `folder`, read now.
pub(super) fn load_for_run(folder: &Path) -> Result<RunFile, String> {
    match crate::run::file::load(folder) {
        Some(Ok(file)) => Ok(file),
        Some(Err(error)) => Err(error.to_string()),
        None => Err(format!("{} has no .run.toml", folder.display())),
    }
}

/// The live (or waiting) run of `key` among `rows`: Run shows it rather than starting another.
pub(super) fn blocking_run(rows: &[RunRow], key: &str) -> Option<RunId> {
    rows.iter().find(|row| row.live && row.key == key).map(|row| row.id)
}

/// What `config` runs as its `.run.toml` writes it, `${name}` for values: never a secret, so an error can show it.
fn written_command(config: &crate::run::file::Config) -> String {
    match &config.kind {
        crate::run::file::ConfigKind::Run(run) => format!("`{}`", run.command),
        crate::run::file::ConfigKind::Serve(serve) => format!("a static server of {} on :{}", serve.root, serve.port),
    }
}

/// Why `request` could not run, with which configuration of which `.run.toml` and in which worktree.
fn run_failure(request: &RunRequest, error: String) -> crate::failure::Failure {
    let folder = crate::run::values_tab::relative(&request.folder, &request.checkout.path);
    let file = match folder.is_empty() {
        true => ".run.toml".to_owned(),
        false => format!("{folder}/.run.toml"),
    };
    crate::failure::Failure::new(error).with("Configuration", format!("{} in {file}", request.key)).with("Worktree", request.checkout.path.display().to_string())
}

/// The runs of `key` among `rows` that ended: a new run of it replaces them.
fn exited_runs_of(rows: &[RunRow], key: &str) -> Vec<RunId> {
    rows.iter().filter(|row| !row.live && row.key == key).map(|row| row.id).collect()
}

fn vars_file() -> Result<VarsFile, String> {
    let dir = profile::current().config_dir().ok_or_else(|| "no config folder for vars.toml".to_owned())?;
    Ok(VarsFile::new(dir.join("vars.toml")))
}

fn builtins(request: &RunRequest) -> Builtins {
    builtins_at(&request.checkout.path, &request.folder)
}

fn builtins_at(checkout: &Path, folder: &Path) -> Builtins {
    Builtins {
        user: std::env::var("USER").or_else(|_| std::env::var("LOGNAME")).or_else(|_| std::env::var("USERNAME")).unwrap_or_default(),
        home: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        repo: checkout.to_path_buf(),
        dir: folder.to_path_buf(),
    }
}

/// A run's `.run.toml` and values, read now; the wallet may block, so this runs on the background executor.
pub(super) fn read_run_values(inputs: &RunInputs, wallet: &dyn Wallet) -> Result<RunValues, String> {
    let file = load_for_run(&inputs.folder)?;
    let builtins = builtins_at(&inputs.checkout, &inputs.folder);
    let vars = inputs.vars.clone().ok_or_else(|| "no config folder for vars.toml".to_owned())?;
    let doc = VarsFile::new(vars).load()?;
    let sources = Sources { doc: &doc, wallet, project: &inputs.project };
    let resolved = resolve(&file, &inputs.key, &sources, &builtins)?;
    Ok((file, resolved, builtins))
}

/// Saves what the values tab got: plain values to vars.toml, secrets to the wallet with a reference in vars.toml.
fn save_values(vars: VarsFile, values: &[(VarDecl, String)], wallet: &dyn Wallet) -> Result<(), String> {
    values.iter().try_for_each(|(var, value)| match var.secret {
        true => vars.save_secret(&var.name, value, wallet),
        false => vars.save_plain(&var.name, value),
    })
}

/// A waiting run as the tree shows it.
fn pending_row(run: &PendingRun) -> RunRow {
    let (state, status) = match (&run.failed, run.chain.waiting_name()) {
        (Some(reason), _) => (RunBadge::Failed, reason.clone()),
        (None, Some(step)) => (RunBadge::Waiting, format!("before: {step}")),
        (None, None) => (RunBadge::Waiting, "starting".to_owned()),
    };
    let label = pending_label(run.failed.as_ref(), run.chain.waiting_name());
    RunRow { id: RunId::Pending(run.id), key: run.request.key.clone(), name: run.name.clone(), serve: run.serve, state, status, label, live: run.failed.is_none(), link: None }
}

/// What a waiting run's configuration row says: the step it waits for, or why its chain failed.
fn pending_label(failed: Option<&String>, waiting_on: Option<&str>) -> String {
    match (failed, waiting_on) {
        (Some(reason), _) => format!("✕ {reason}"),
        (None, Some(step)) => format!("… waiting for {step}"),
        (None, None) => "… starting".to_owned(),
    }
}

/// What a started run's configuration row says; a running server adds its port.
fn life_label(life: RunLife, url: Option<&str>) -> String {
    match life {
        RunLife::Running => {
            url.and_then(port_of).map_or_else(|| "● running".to_owned(), |port| format!("● running · :{port}"))
        }
        RunLife::Exited { stopped: true, .. } => "■ stopped".to_owned(),
        RunLife::Exited { code: 0, .. } => "exit 0".to_owned(),
        // 128 + n is how shells report a program killed by signal n
        RunLife::Exited { code, .. } => match crate::terminal::exit::signal_short_name(code - 128) {
            Some(signal) => format!("✕ exit {code} ({signal})"),
            None => format!("✕ exit {code}"),
        },
    }
}

/// A started run as the tree shows it.
fn run_row(process: &Process) -> Option<RunRow> {
    let origin = process.run.as_ref()?;
    let (state, text) = match origin.life {
        RunLife::Running => (RunBadge::Running, "running".to_owned()),
        RunLife::Exited { stopped: true, .. } => (RunBadge::Stopped, "stopped".to_owned()),
        RunLife::Exited { code, .. } => (RunBadge::Exited(code), format!("exited {code}")),
    };
    Some(RunRow {
        id: RunId::Process(process.id),
        key: origin.key.clone(),
        name: origin.name.clone(),
        serve: origin.url.is_some(),
        state,
        status: text,
        label: life_label(origin.life, origin.url.as_deref()),
        live: origin.life == RunLife::Running,
        link: process.link().map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(key: &str, id: RunId, live: bool) -> RunRow {
        let state = if live { RunBadge::Running } else { RunBadge::Exited(0) };
        RunRow { id, key: key.to_owned(), name: key.to_owned(), serve: false, state, status: String::new(), label: String::new(), live, link: None }
    }

    fn request(folder: &str) -> RunRequest {
        let checkout = crate::project::CheckoutRef { project: crate::project::ProjectId::new("p"), path: PathBuf::from("/r") };
        RunRequest { checkout, folder: PathBuf::from(folder), key: "web".to_owned(), show: ShowRun::Tab }
    }

    #[test]
    fn should_name_configuration_and_file_of_failed_run() {
        // given
        let request = request("/r/apps/site");

        // when
        let report = run_failure(&request, "cannot start web (`npm run dev` in /r/apps/site): No such file or directory".to_owned()).report("Run web");

        // then
        assert!(report.contains("Configuration: web in apps/site/.run.toml\nWorktree: /r\n"), "{report}");
    }

    #[test]
    fn should_name_root_run_file() {
        // given
        let request = request("/r");

        // when
        let report = run_failure(&request, "no configuration web".to_owned()).report("Run web");

        // then
        assert!(report.contains("Configuration: web in .run.toml\n"), "{report}");
    }

    #[cfg(unix)]
    #[test]
    fn should_name_signal_of_run_exit() {
        // given
        let killed = RunLife::Exited { code: 137, stopped: false };

        // when
        let label = life_label(killed, None);

        // then
        assert_eq!(label, "✕ exit 137 (SIGKILL)");
    }

    #[test]
    fn should_label_a_started_run_as_its_configurations_row_shows_it() {
        // given
        let lives = [
            (RunLife::Running, Some("http://127.0.0.1:4301/mock/")),
            (RunLife::Running, None),
            (RunLife::Exited { code: 0, stopped: false }, None),
            (RunLife::Exited { code: 2, stopped: false }, None),
            (RunLife::Exited { code: 143, stopped: true }, None),
        ];

        // when
        let labels = lives.map(|(life, url)| life_label(life, url));

        // then
        assert_eq!(labels, ["● running · :4301", "● running", "exit 0", "✕ exit 2", "■ stopped"].map(str::to_owned));
    }

    #[test]
    fn should_label_a_waiting_run_with_its_step_or_why_it_failed() {
        // given
        let failed = "before: Build misc failed (exit 1)".to_owned();

        // when
        let labels = [pending_label(None, Some("Build misc")), pending_label(None, None), pending_label(Some(&failed), Some("Build misc"))];

        // then
        assert_eq!(labels, ["… waiting for Build misc", "… starting", "✕ before: Build misc failed (exit 1)"].map(str::to_owned));
    }

    #[test]
    fn should_read_values_without_the_workspace() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("misc");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(".run.toml"), "[vars.token]\nsecret = true\n\n[vars.port]\n\n[run.app]\ncommand = \"run ${port}\"\nenv = { T = \"${token}\" }\n").unwrap();
        std::fs::write(dir.path().join("vars.toml"), "token = { wallet = \"t\" }\n").unwrap();
        let wallet = crate::run::wallet::MemoryWallet::default();
        wallet.set("t", "s3cret").unwrap();
        let inputs = RunInputs {
            folder: folder.clone(),
            key: "app".to_owned(),
            checkout: dir.path().to_path_buf(),
            project: dir.path().to_path_buf(),
            vars: Some(dir.path().join("vars.toml")),
        };

        // when
        let (_, resolved, builtins) = read_run_values(&inputs, &wallet).unwrap();

        // then
        assert_eq!((resolved.values.get("token").map(String::as_str), resolved.missing), (Some("s3cret"), vec!["port".to_owned()]));
        assert_eq!(builtins.dir, folder);
    }

    #[test]
    fn should_read_the_file_again_on_run() {
        // given
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".run.toml"), "[run.build]\ncommand = \"make\"\n").unwrap();
        let first = load_for_run(dir.path()).unwrap();
        std::fs::write(dir.path().join(".run.toml"), "[run.build]\ncommand = \"make all\"\n").unwrap();

        // when
        let second = load_for_run(dir.path()).unwrap();

        // then
        assert_ne!(first, second);
        assert_eq!(load_for_run(&dir.path().join("gone")).unwrap_err(), format!("{} has no .run.toml", dir.path().join("gone").display()));
    }

    #[test]
    fn should_not_start_a_second_run_while_one_is_pending() {
        // given
        let rows = [row("app", RunId::Pending(1), true), row("build", RunId::Pending(2), false)];

        // when
        let blocked = [blocking_run(&rows, "app"), blocking_run(&rows, "build"), blocking_run(&rows, "other")];

        // then
        assert_eq!(blocked, [Some(RunId::Pending(1)), None, None]);
    }

    #[test]
    fn should_remove_the_exited_run_before_starting_again() {
        // given
        let rows = [row("app", RunId::Pending(1), false), row("build", RunId::Pending(2), false), row("app", RunId::Pending(3), true)];

        // when
        let removed = exited_runs_of(&rows, "app");

        // then
        assert_eq!(removed, [RunId::Pending(1)]);
    }
}
