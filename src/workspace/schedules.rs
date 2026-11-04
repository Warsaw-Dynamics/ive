//! Scheduled commands in the workspace: the clock that makes them due, the Scheduled commands tab and Settings page
//! they show in, starting one in a new terminal, and the MCP tools that schedule them.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Local;
use gpui::{Context, Task, Window};
use serde_json::json;

use super::processes::BehindStart;
use super::{AfterOpen, Workspace};
use crate::api::ScheduleArgs;
use crate::desktop_notice::NoticeKey;
use crate::editor::Tab;
use crate::process::ProcessId;
use crate::schedule::target::{self, Plan};
use crate::schedule::when::{Timing, first_time};
use crate::schedule::{Check, NewEntry, Outcome, Schedule, Target, file, local_text, utc_text};

/// How often the clock is read; a command starts at most this late.
const CHECK_EVERY: Duration = Duration::from_secs(30);
const CLOSED: &str = "the IVE window is closed";

/// Who starts a command: the user from the tab, or the clock for a command allowed to run unasked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartBy {
    Tab,
    Unasked,
}

/// Where the Scheduled commands tab goes when commands come due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabPlace {
    Shown,
    Behind,
}

/// A command started from the tab since it opened: the tab lists it under Started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedHere {
    pub name: String,
    pub where_: String,
    pub process: ProcessId,
}

/// The schedule as last read or written, and what the tab shows about it.
#[derive(Default)]
pub(crate) struct Schedules {
    schedule: Schedule,
    path: Option<PathBuf>,
    /// each due command's plan, read off the UI thread
    plans: HashMap<u64, Plan>,
    starting: HashSet<u64>,
    picked: BTreeSet<u64>,
    /// rows whose Always run this one is ticked
    always: BTreeSet<u64>,
    started_here: Vec<StartedHere>,
    _clock: Option<Task<()>>,
}

impl Schedules {
    /// The profile's `schedule.toml`; why it was set aside, when it could not be used.
    pub(crate) fn load() -> (Self, Option<String>) {
        let path = crate::profile::current().config_dir().map(|dir| dir.join("schedule.toml"));
        let (schedule, problem) = path.as_deref().map_or((Schedule::default(), None), file::load);
        (Self { schedule, path, ..Self::default() }, problem)
    }

    /// What the tab and the Settings page draw from.
    pub(crate) fn view(&self) -> crate::schedule::tab::View<'_> {
        crate::schedule::tab::View {
            schedule: &self.schedule,
            plans: &self.plans,
            starting: &self.starting,
            picked: &self.picked,
            always: &self.always,
            started: &self.started_here,
            now: Local::now().timestamp(),
            path: self.path.as_deref(),
        }
    }

    /// Another IVE of the profile may have written since: the file is the truth.
    fn reload(&mut self) {
        let Some(path) = &self.path else {
            return;
        };
        let (schedule, problem) = file::load(path);
        if let Some(problem) = problem {
            crate::notices::log_only(&problem);
        }
        self.schedule = schedule;
    }

    fn save(&mut self) {
        self.schedule.prune(Local::now().timestamp());
        let Some(path) = &self.path else {
            return;
        };
        if let Err(error) = file::save(&self.schedule, path) {
            crate::notices::log_only(&error);
        }
    }

    /// Reads the file, changes the schedule with `change`, and writes it.
    fn change<T>(&mut self, change: impl FnOnce(&mut Schedule) -> T) -> T {
        self.reload();
        let result = change(&mut self.schedule);
        self.save();
        result
    }
}

impl Workspace {
    /// Starts the clock: commands that passed while IVE was closed come due now, then every [`CHECK_EVERY`]. Only
    /// the IVE owning the profile's terminals runs it; another one only schedules.
    pub(super) fn start_schedule_clock(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let task = cx.spawn_in(window, async move |this, cx| {
            let mut check = Check::Startup;
            loop {
                if this.update_in(cx, |workspace, window, cx| workspace.check_schedule(check, window, cx)).is_err() {
                    return;
                }
                check = Check::Running;
                cx.background_executor().timer(CHECK_EVERY).await;
            }
        });
        self.schedules._clock = Some(task);
    }

    fn check_schedule(&mut self, check: Check, window: &mut Window, cx: &mut Context<Self>) {
        let came = self.schedules.change(|schedule| schedule.come_due(Local::now(), check));
        for id in came.unasked {
            self.start_scheduled(id, StartBy::Unasked, window, cx);
        }
        if came.due.is_empty() {
            return;
        }
        let place = if check == Check::Startup { TabPlace::Shown } else { TabPlace::Behind };
        self.open_scheduled_tab(place, window, cx);
        self.notify_due_on_desktop();
    }

    /// With no IVE window active, one desktop notification says what is due; a click shows the tab.
    fn notify_due_on_desktop(&mut self) {
        if self.active_window.is_any() {
            return;
        }
        let due = self.schedules.schedule.due();
        let title = match due.len() {
            1 => "A scheduled command is due".to_owned(),
            n => format!("{n} scheduled commands are due"),
        };
        let names: Vec<&str> = due.iter().map(|entry| entry.name.as_str()).collect();
        let body = format!("{} · nothing runs until you pick it", names.join(", "));
        self.desktop_notices.send(NoticeKey::Scheduled, &crate::terminal::one_line(&title), &crate::terminal::one_line(&body));
    }

    /// Opens the Scheduled commands tab, or shows it; reads the plans of what is due.
    pub(crate) fn open_scheduled_tab(&mut self, place: TabPlace, window: &mut Window, cx: &mut Context<Self>) {
        match (self.tabs.find(Tab::is_scheduled), place) {
            (Some(open), TabPlace::Shown) => self.activate_tab(open, window, cx),
            (Some(_), TabPlace::Behind) => {}
            (None, TabPlace::Shown) => {
                self.schedules.started_here.clear();
                self.open_tab(Tab::Scheduled, window, cx);
            }
            (None, TabPlace::Behind) => {
                self.schedules.started_here.clear();
                self.tabs.open_behind(Tab::Scheduled);
            }
        }
        self.read_schedule_plans(cx);
        cx.notify();
    }

    /// A click on the desktop notification: the main window in front, the tab shown.
    pub(crate) fn show_scheduled_from_notice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_menu(window, cx);
        if window.window_handle() == self.main_window {
            window.activate_window();
        } else {
            let _ = self.main_window.update(cx, |_, main, _| main.activate_window());
        }
        self.open_scheduled_tab(TabPlace::Shown, window, cx);
    }

    /// Reads where each due command would run, off the UI thread.
    fn read_schedule_plans(&mut self, cx: &mut Context<Self>) {
        let targets: Vec<(u64, Target)> = self.schedules.schedule.due().iter().map(|entry| (entry.id, entry.target.clone())).collect();
        if targets.is_empty() {
            return;
        }
        cx.spawn(async move |this, cx| {
            let plans = cx.background_executor().spawn(async move { targets.iter().map(|(id, target)| (*id, target::plan(target))).collect::<HashMap<u64, Plan>>() }).await;
            let _ = this.update(cx, |workspace, cx| {
                // a row that cannot run now cannot be ticked either
                workspace.schedules.picked.retain(|id| plans.get(id).is_none_or(Plan::is_runnable));
                workspace.schedules.plans = plans;
                cx.notify();
            });
        })
        .detach();
    }

    /// The Scheduled commands tab's dot and tooltip line, while something is due.
    pub(crate) fn scheduled_attention(&self) -> Option<String> {
        match self.schedules.schedule.due().len() {
            0 => None,
            1 => Some("1 command is due".to_owned()),
            n => Some(format!("{n} commands are due")),
        }
    }

    pub(crate) fn schedules(&self) -> &Schedules {
        &self.schedules
    }

    // ---- starting ----

    /// Starts command `id`: its project opened when it is not, its worktree made when missing (off the UI thread),
    /// then a terminal behind the shown tab running it.
    fn start_scheduled(&mut self, id: u64, by: StartBy, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.schedules.schedule.get(id).cloned() else {
            return;
        };
        if !self.schedules.starting.insert(id) {
            return;
        }
        let wanted = entry.target.clone();
        let project_open = self.checkout_of_path(&wanted.project).is_some();
        cx.spawn_in(window, async move |this, cx| {
            let prepared = async {
                if !project_open {
                    let opening = this.update_in(cx, |workspace, window, cx| workspace.open_folder_task(wanted.project.clone(), AfterOpen::ListCheckout, window, cx));
                    opening.map_err(|_| CLOSED.to_owned())?.await?;
                }
                let target = wanted.clone();
                let made = cx.background_executor().spawn(async move { prepare_checkout(&target) }).await?;
                if made.is_new {
                    // the project reads its new worktree
                    let opening = this.update_in(cx, |workspace, window, cx| workspace.open_folder_task(made.path.clone(), AfterOpen::ListCheckout, window, cx));
                    opening.map_err(|_| CLOSED.to_owned())?.await?;
                }
                Ok::<PathBuf, String>(made.path)
            }
            .await;
            let _ = this.update_in(cx, |workspace, window, cx| workspace.scheduled_prepared(id, by, prepared, window, cx));
        })
        .detach();
        cx.notify();
    }

    fn scheduled_prepared(&mut self, id: u64, by: StartBy, prepared: Result<PathBuf, String>, window: &mut Window, cx: &mut Context<Self>) {
        self.schedules.starting.remove(&id);
        match prepared.and_then(|path| self.open_scheduled_terminal(id, &path, window, cx)) {
            Ok(process) => self.scheduled_started(id, by, process, cx),
            Err(error) => self.scheduled_failed(id, by, error, window, cx),
        }
        self.read_schedule_plans(cx);
        cx.notify();
    }

    fn open_scheduled_terminal(&mut self, id: u64, path: &Path, window: &mut Window, cx: &mut Context<Self>) -> Result<ProcessId, String> {
        let entry = self.schedules.schedule.get(id).cloned().ok_or_else(|| format!("scheduled command {id} is gone"))?;
        let checkout = self.checkout_of_path(path).ok_or_else(|| format!("{} is not in an open project", path.display()))?;
        self.show_worktree(&checkout, cx);
        let env = vec![
            ("IVE_SCHEDULE_ID".to_owned(), id.to_string()),
            ("IVE_SCHEDULE_NAME".to_owned(), entry.name.clone()),
            ("IVE_SCHEDULE_DESCRIPTION".to_owned(), entry.description.clone()),
        ];
        self.open_terminal_behind(checkout, BehindStart { env, title: entry.name.clone(), command: entry.command.clone() }, window, cx)
    }

    fn scheduled_started(&mut self, id: u64, by: StartBy, process: ProcessId, cx: &mut Context<Self>) {
        let always = self.schedules.always.remove(&id);
        self.schedules.picked.remove(&id);
        let unasked = by == StartBy::Unasked;
        let entry = self.schedules.change(|schedule| {
            if always {
                schedule.allow(id);
            }
            let entry = schedule.get(id).cloned();
            schedule.finish(id, Outcome::Started { process: process.value(), unasked }, Local::now());
            entry
        });
        let Some(entry) = entry else {
            return;
        };
        match by {
            StartBy::Tab => self.schedules.started_here.push(StartedHere { name: entry.name, where_: where_text(&entry.target), process }),
            StartBy::Unasked => self.call_for_user(process, "Scheduled: started unasked".to_owned(), cx),
        }
    }

    /// A start that failed: the command is due with the reason; one the clock started comes to the tab.
    fn scheduled_failed(&mut self, id: u64, by: StartBy, error: String, window: &mut Window, cx: &mut Context<Self>) {
        crate::notices::log_only(&format!("cannot start scheduled command {id}: {error}"));
        self.schedules.change(|schedule| schedule.fail_start(id, error));
        if by == StartBy::Unasked {
            self.open_scheduled_tab(TabPlace::Behind, window, cx);
            self.notify_due_on_desktop();
        }
    }

    // ---- the tab and the Settings page ----

    pub(crate) fn toggle_scheduled_pick(&mut self, id: u64, cx: &mut Context<Self>) {
        if !self.schedules.picked.remove(&id) && self.is_schedule_runnable(id) {
            self.schedules.picked.insert(id);
        }
        cx.notify();
    }

    /// All: ticks every row that can run, or none when they all are.
    pub(crate) fn pick_all_scheduled(&mut self, cx: &mut Context<Self>) {
        let runnable: BTreeSet<u64> = self.schedules.schedule.due().iter().map(|entry| entry.id).filter(|id| self.is_schedule_runnable(*id)).collect();
        self.schedules.picked = if runnable.is_subset(&self.schedules.picked) { BTreeSet::new() } else { runnable };
        cx.notify();
    }

    /// Always run this one: ticking it ticks the row too.
    pub(crate) fn toggle_scheduled_always(&mut self, id: u64, cx: &mut Context<Self>) {
        if !self.schedules.always.remove(&id) {
            self.schedules.always.insert(id);
            self.schedules.picked.insert(id);
        }
        cx.notify();
    }

    fn is_schedule_runnable(&self, id: u64) -> bool {
        !self.schedules.starting.contains(&id) && self.schedules.plans.get(&id).is_none_or(Plan::is_runnable)
    }

    pub(crate) fn run_picked_scheduled(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked: Vec<u64> = std::mem::take(&mut self.schedules.picked).into_iter().collect();
        for id in picked {
            self.start_scheduled(id, StartBy::Tab, window, cx);
        }
    }

    pub(crate) fn skip_picked_scheduled(&mut self, cx: &mut Context<Self>) {
        let picked = std::mem::take(&mut self.schedules.picked);
        self.schedules.change(|schedule| picked.iter().for_each(|id| schedule.finish(*id, Outcome::Skipped, Local::now())));
        cx.notify();
    }

    /// Remove on a row that cannot run: into the history as failed, saying why.
    pub(crate) fn remove_failed_scheduled(&mut self, id: u64, cx: &mut Context<Self>) {
        let reason = self.schedules.plans.get(&id).map_or_else(|| "removed".to_owned(), |plan| plan.text(""));
        self.schedules.change(|schedule| schedule.finish(id, Outcome::Failed(reason), Local::now()));
        cx.notify();
    }

    pub(crate) fn cancel_scheduled(&mut self, id: u64, cx: &mut Context<Self>) -> Result<(), String> {
        let cancelled = self.schedules.change(|schedule| schedule.cancel(id, Local::now().timestamp()));
        cx.notify();
        cancelled
    }

    pub(crate) fn ask_again_scheduled(&mut self, id: u64, cx: &mut Context<Self>) {
        self.schedules.change(|schedule| schedule.ask_again(id));
        cx.notify();
    }

    /// A started command's terminal, while it lives.
    pub(crate) fn show_scheduled_process(&mut self, process: u64, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.processes.id_of(process) {
            self.show_process(id, window, cx);
        }
    }

    // ---- MCP ----

    /// `schedule_command`: checks, resolves the target and the first time, and writes the command.
    pub(super) fn schedule_command(&mut self, args: ScheduleArgs, cx: &mut Context<Self>) -> Result<serde_json::Value, String> {
        let target = self.schedule_target(&args)?;
        let timing = Timing { at: args.at, delay: args.delay, repeat: args.repeat };
        let first = first_time(&timing, &Local::now())?;
        let new = NewEntry { name: args.name, description: args.description.unwrap_or_default(), command: args.command, target, at: first.at, repeat: first.repeat };
        let added = self.schedules.change(|schedule| schedule.add(new, Local::now().timestamp()))?;
        cx.notify();
        Ok(json!({ "id": added.id, "at": utc_text(first.at), "at_local": local_text(first.at), "replaced": added.replaced }))
    }

    pub(super) fn list_scheduled(&mut self) -> serde_json::Value {
        self.schedules.reload();
        self.schedules.schedule.json(Local::now().timestamp())
    }

    /// The main checkout and branch `args` name: `project` and `branch`, or the checkout holding `directory`.
    fn schedule_target(&self, args: &ScheduleArgs) -> Result<Target, String> {
        let (project, branch) = match (&args.directory, &args.project) {
            (Some(_), Some(_)) => return Err("give `project` or `directory`, not both".to_owned()),
            (Some(directory), None) => self.directory_target(directory, args.branch.clone())?,
            (None, Some(project)) => (main_checkout(project)?, args.branch.clone().ok_or("`branch` is required with `project`")?),
            (None, None) => return Err("say where: `project` and `branch`, or `directory`".to_owned()),
        };
        Ok(Target { project, branch, worktree: args.worktree.as_deref().map(PathBuf::from), source: args.source_branch.clone() })
    }

    fn directory_target(&self, directory: &str, branch: Option<String>) -> Result<(PathBuf, String), String> {
        let path = self.resolve(Path::new(directory));
        let path = dunce::canonicalize(&path).map_err(|error| format!("cannot use {}: {error}", path.display()))?;
        let checkout = self.checkout_of_path(&path).ok_or_else(|| format!("no open project contains {}", path.display()))?;
        let main = self.project(&checkout.project).map(|project| project.main_checkout().path.clone()).ok_or("its project is closed")?;
        let branch = branch.or_else(|| self.checkout(&checkout).and_then(|open| open.branch.clone()));
        Ok((main, branch.ok_or_else(|| format!("{} has no branch checked out (detached HEAD)", checkout.path.display()))?))
    }
}

/// The checkout folder made or found for `target`, and whether IVE made it.
struct Made {
    path: PathBuf,
    is_new: bool,
}

fn prepare_checkout(target: &Target) -> Result<Made, String> {
    let plan = target::plan(target);
    let is_new = !matches!(plan, Plan::In { .. });
    target::prepare(&plan, target).map(|path| Made { path, is_new })
}

/// The repository's main checkout for a folder in it, canonical; refused when it is not in one.
fn main_checkout(project: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(project);
    if path.is_relative() {
        return Err(format!("project {project:?} must be an absolute path"));
    }
    let path = dunce::canonicalize(&path).map_err(|error| format!("cannot use {project}: {error}"))?;
    crate::git::worktrees::main_checkout_of(&path)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("{} is not in a git repository", path.display()))
}

/// `<project folder name> · ⎇ <branch>`.
pub fn where_text(target: &Target) -> String {
    let project = target.project.file_name().map_or_else(|| target.project.display().to_string(), |name| name.to_string_lossy().into_owned());
    format!("{project} · ⎇ {}", target.branch)
}
