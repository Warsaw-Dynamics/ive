//! Commands agents schedule over MCP for later. A command whose time comes is due: it waits in the Scheduled
//! commands tab until the user picks it, unless the user allowed that exact command to run unasked.

pub mod cron;
pub mod file;
pub mod tab;
pub mod target;
pub mod when;

use std::path::PathBuf;

use chrono::{DateTime, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use self::cron::Cron;

/// Pending and due commands at most; an agent scheduling more is most likely stuck in a loop.
pub const MAX_WAITING: usize = 100;
pub const MAX_NAME_CHARS: usize = 80;
pub const MAX_DESCRIPTION_CHARS: usize = 2000;
pub const MAX_COMMAND_BYTES: usize = 8000;
/// How long started, skipped, failed and cancelled commands stay listed.
pub const HISTORY_DAYS: i64 = 7;
const DAY_SECONDS: i64 = 24 * 3600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Pending,
    Due,
    Started,
    Skipped,
    Failed,
    Cancelled,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Due => "due",
            Self::Started => "started",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn is_waiting(self) -> bool {
        matches!(self, Self::Pending | Self::Due)
    }
}

/// Where a command runs: the project's main checkout, the branch, and how to make its worktree when missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub project: PathBuf,
    pub branch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// One scheduled command; times in seconds since the epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub command: String,
    pub target: Target,
    pub at: i64,
    /// a cron expression, for a repeating command
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<String>,
    pub state: State,
    /// the times a due repeating command passed while it waited
    #[serde(default)]
    pub missed: u32,
    /// the command text the user allowed to run unasked
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted: Option<String>,
    pub created: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<i64>,
    /// its terminal's process id, once started
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// started without the user picking it
    #[serde(default)]
    pub unasked: bool,
    /// came due when IVE started: it passed while IVE was closed
    #[serde(default)]
    pub closed: bool,
}

impl Entry {
    /// Whether it starts unasked: allowed for exactly this command.
    pub fn runs_unasked(&self) -> bool {
        self.trusted.as_deref() == Some(self.command.as_str())
    }

    pub fn cron(&self) -> Option<Cron> {
        self.repeat.as_deref().and_then(|text| Cron::parse(text).ok())
    }

    pub fn json(&self) -> serde_json::Value {
        json!({
            "id": self.id, "name": self.name, "description": self.description, "command": self.command,
            "project": self.target.project, "branch": self.target.branch, "worktree": self.target.worktree,
            "source_branch": self.target.source, "at": utc_text(self.at), "at_local": local_text(self.at),
            "repeat": self.repeat, "state": self.state.label(), "runs_unasked": self.runs_unasked(), "missed": self.missed,
            "process": self.process, "error": self.error, "unasked": self.unasked,
        })
    }
}

/// A command an agent asks for, checked and with its first time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub name: String,
    pub description: String,
    pub command: String,
    pub target: Target,
    pub at: i64,
    pub repeat: Option<Cron>,
}

impl NewEntry {
    /// Refuses an empty or multi-line name and texts over their bounds.
    pub fn check(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() || name.contains(['\n', '\r']) {
            return Err(format!("name {:?} must be one line, not empty", self.name));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(format!("name is over {MAX_NAME_CHARS} characters"));
        }
        if self.description.chars().count() > MAX_DESCRIPTION_CHARS {
            return Err(format!("description is over {MAX_DESCRIPTION_CHARS} characters"));
        }
        if self.command.trim().is_empty() || self.command.len() > MAX_COMMAND_BYTES {
            return Err(format!("command must be 1 to {MAX_COMMAND_BYTES} bytes"));
        }
        if self.target.branch.trim().is_empty() {
            return Err("branch is empty".to_owned());
        }
        Ok(())
    }
}

/// What [`Schedule::add`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Added {
    pub id: u64,
    pub replaced: bool,
}

/// What [`Schedule::come_due`] found: the commands to start unasked, and those that became due.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CameDue {
    pub unasked: Vec<u64>,
    pub due: Vec<u64>,
}

/// How a due command ended this time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Started { process: u64, unasked: bool },
    Skipped,
    Failed(String),
}

/// When [`Schedule::come_due`] runs: at start, what came due passed while IVE was closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    Startup,
    Running,
}

/// Every scheduled command and the history, as `schedule.toml` keeps them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    #[serde(default)]
    next_id: u64,
    #[serde(default, rename = "command")]
    entries: Vec<Entry>,
}

impl Schedule {
    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|entry| entry.id == id)
    }

    /// Adds `new`, or replaces the waiting command of the same name in the same project, keeping its id; a changed
    /// command loses the permission to run unasked.
    pub fn add(&mut self, new: NewEntry, now: i64) -> Result<Added, String> {
        new.check()?;
        let same = |entry: &&mut Entry| entry.state.is_waiting() && entry.name == new.name.trim() && entry.target.project == new.target.project;
        if let Some(entry) = self.entries.iter_mut().find(same) {
            let trusted = entry.trusted.take().filter(|trusted| *trusted == new.command);
            *entry = fresh(entry.id, new, now);
            entry.trusted = trusted;
            return Ok(Added { id: entry.id, replaced: true });
        }
        if self.waiting().count() >= MAX_WAITING {
            return Err(format!("{MAX_WAITING} commands are scheduled already; cancel some first"));
        }
        let id = self.take_id();
        self.entries.push(fresh(id, new, now));
        Ok(Added { id, replaced: false })
    }

    fn take_id(&mut self) -> u64 {
        let used = self.entries.iter().map(|entry| entry.id).max().unwrap_or(0);
        self.next_id = self.next_id.max(used) + 1;
        self.next_id
    }

    /// The pending commands whose time came by `now`: those allowed to run unasked are returned to start; the rest
    /// become due. A due repeating command counts the times that passed while it waited.
    pub fn come_due(&mut self, now: DateTime<Local>, check: Check) -> CameDue {
        let seconds = now.timestamp();
        let mut came = CameDue::default();
        for entry in self.entries.iter_mut() {
            match entry.state {
                State::Pending if entry.at <= seconds && entry.runs_unasked() => came.unasked.push(entry.id),
                State::Pending if entry.at <= seconds => {
                    entry.state = State::Due;
                    entry.closed = check == Check::Startup;
                    entry.error = None;
                    count_missed(entry, &now);
                    came.due.push(entry.id);
                }
                State::Due => count_missed(entry, &now),
                _ => {}
            }
        }
        came
    }

    /// Ends this time of a pending or due command: a repeating one leaves a history entry and waits for its next
    /// time after `now`; a one-off one becomes that entry.
    pub fn finish(&mut self, id: u64, outcome: Outcome, now: DateTime<Local>) {
        let Some(entry) = self.get(id).cloned() else {
            return;
        };
        let cron = entry.cron();
        let record = record(entry, outcome, now.timestamp());
        let Some(cron) = cron else {
            if let Some(slot) = self.get_mut(id) {
                *slot = record;
            }
            return;
        };
        let history_id = self.take_id();
        self.entries.push(Entry { id: history_id, trusted: None, ..record });
        if let Some(entry) = self.get_mut(id) {
            entry.at = cron.next_after(&now).map_or(i64::MAX, |time| time.timestamp());
            entry.state = State::Pending;
            entry.missed = 0;
            entry.error = None;
            entry.closed = false;
        }
    }

    /// A start that failed: the command is due, saying why.
    pub fn fail_start(&mut self, id: u64, error: String) {
        if let Some(entry) = self.get_mut(id) {
            entry.state = State::Due;
            entry.error = Some(error);
        }
    }

    /// Cancels a pending or due command (a repeating one with every later time); why not, otherwise.
    pub fn cancel(&mut self, id: u64, now: i64) -> Result<(), String> {
        let entry = self.get_mut(id).ok_or_else(|| format!("no scheduled command {id}; see list_scheduled"))?;
        if !entry.state.is_waiting() {
            return Err(format!("scheduled command {id} is {} already", entry.state.label()));
        }
        entry.state = State::Cancelled;
        entry.done = Some(now);
        Ok(())
    }

    /// Allows the command as it is now to run unasked.
    pub fn allow(&mut self, id: u64) {
        if let Some(entry) = self.get_mut(id) {
            entry.trusted = Some(entry.command.clone());
        }
    }

    pub fn ask_again(&mut self, id: u64) {
        if let Some(entry) = self.get_mut(id) {
            entry.trusted = None;
        }
    }

    fn waiting(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|entry| entry.state.is_waiting())
    }

    /// Due commands, oldest first.
    pub fn due(&self) -> Vec<&Entry> {
        sorted_by_time(self.entries.iter().filter(|entry| entry.state == State::Due))
    }

    /// Pending commands, soonest first.
    pub fn pending(&self) -> Vec<&Entry> {
        sorted_by_time(self.entries.iter().filter(|entry| entry.state == State::Pending))
    }

    /// What ended in the last [`HISTORY_DAYS`] days, newest first.
    pub fn history(&self, now: i64) -> Vec<&Entry> {
        let since = now - HISTORY_DAYS * DAY_SECONDS;
        let mut ended: Vec<&Entry> = self.entries.iter().filter(|entry| entry.done.is_some_and(|done| done >= since)).collect();
        ended.sort_by_key(|entry| (std::cmp::Reverse(entry.done), std::cmp::Reverse(entry.id)));
        ended
    }

    /// Drops what ended before the history's days.
    pub fn prune(&mut self, now: i64) {
        let since = now - HISTORY_DAYS * DAY_SECONDS;
        self.entries.retain(|entry| entry.state.is_waiting() || entry.done.is_some_and(|done| done >= since));
    }

    /// `list_scheduled`: pending and due, then the history.
    pub fn json(&self, now: i64) -> serde_json::Value {
        let waiting: Vec<serde_json::Value> = self.due().into_iter().chain(self.pending()).map(Entry::json).collect();
        let history: Vec<serde_json::Value> = self.history(now).into_iter().map(Entry::json).collect();
        json!({ "scheduled": waiting, "history": history })
    }
}

fn fresh(id: u64, new: NewEntry, now: i64) -> Entry {
    Entry {
        id,
        name: new.name.trim().to_owned(),
        description: new.description,
        command: new.command,
        target: new.target,
        at: new.at,
        repeat: new.repeat.map(|cron| cron.text().to_owned()),
        state: State::Pending,
        missed: 0,
        trusted: None,
        created: now,
        done: None,
        process: None,
        error: None,
        unasked: false,
        closed: false,
    }
}

/// `entry` as the history keeps this time of it.
fn record(entry: Entry, outcome: Outcome, now: i64) -> Entry {
    let (state, process, unasked, error) = match outcome {
        Outcome::Started { process, unasked } => (State::Started, Some(process), unasked, None),
        Outcome::Skipped => (State::Skipped, None, false, None),
        Outcome::Failed(error) => (State::Failed, None, false, Some(error)),
    };
    Entry { state, process, unasked, error, done: Some(now), ..entry }
}

/// A due repeating command's times that passed after its own: counted, and its time moved to the latest.
fn count_missed(entry: &mut Entry, now: &DateTime<Local>) {
    let Some(cron) = entry.cron() else {
        return;
    };
    let Some(mut last) = Local.timestamp_opt(entry.at, 0).single() else {
        return;
    };
    while let Some(next) = cron.next_after(&last).filter(|next| next <= now) {
        entry.missed += 1;
        last = next;
    }
    entry.at = last.timestamp();
}

fn sorted_by_time<'a>(entries: impl Iterator<Item = &'a Entry>) -> Vec<&'a Entry> {
    let mut list: Vec<&Entry> = entries.collect();
    list.sort_by_key(|entry| (entry.at, entry.id));
    list
}

pub fn utc_text(seconds: i64) -> String {
    Utc.timestamp_opt(seconds, 0).single().map_or_else(String::new, |time| time.to_rfc3339())
}

pub fn local_text(seconds: i64) -> String {
    Local.timestamp_opt(seconds, 0).single().map_or_else(String::new, |time| time.to_rfc3339())
}

/// `Fri 16 Oct, 14:05`, in local time.
pub fn clock_text(seconds: i64) -> String {
    Local.timestamp_opt(seconds, 0).single().map_or_else(String::new, |time| time.format("%a %-d %b, %H:%M").to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 10, 16, 14, 0, 0).unwrap()
    }

    fn new_entry(name: &str, at: i64, repeat: Option<&str>) -> NewEntry {
        NewEntry {
            name: name.to_owned(),
            description: String::new(),
            command: format!("echo {name}"),
            target: Target { project: PathBuf::from("/p"), branch: "main".to_owned(), worktree: None, source: None },
            at,
            repeat: repeat.map(|text| Cron::parse(text).unwrap()),
        }
    }

    fn hours(n: i64) -> i64 {
        now().timestamp() + n * 3600
    }

    #[test]
    fn should_replace_a_waiting_command_of_the_same_name_and_keep_its_id() {
        // given
        let mut schedule = Schedule::default();
        let first = schedule.add(new_entry("check", hours(1), None), hours(0)).unwrap();
        schedule.allow(first.id);
        // when
        let mut changed = new_entry("check", hours(2), None);
        changed.command = "echo other".to_owned();
        let second = schedule.add(changed, hours(0)).unwrap();
        // then
        assert_eq!(second, Added { id: first.id, replaced: true });
        let entry = schedule.get(first.id).unwrap();
        assert_eq!((entry.at, entry.trusted.clone()), (hours(2), None));
    }

    #[test]
    fn should_keep_the_permission_when_the_command_stays() {
        // given
        let mut schedule = Schedule::default();
        let added = schedule.add(new_entry("report", hours(1), Some("0 7 * * *")), hours(0)).unwrap();
        schedule.allow(added.id);
        // when
        schedule.add(new_entry("report", hours(3), Some("0 7 * * *")), hours(0)).unwrap();
        // then
        assert!(schedule.get(added.id).unwrap().runs_unasked());
    }

    #[test]
    fn should_make_due_what_passed_and_return_what_runs_unasked() {
        // given
        let mut schedule = Schedule::default();
        let past = schedule.add(new_entry("past", hours(-1), None), hours(-2)).unwrap();
        let later = schedule.add(new_entry("later", hours(1), None), hours(-2)).unwrap();
        let allowed = schedule.add(new_entry("allowed", hours(-1), Some("0 * * * *")), hours(-2)).unwrap();
        schedule.allow(allowed.id);
        // when
        let came = schedule.come_due(now(), Check::Startup);
        // then
        assert_eq!(came, CameDue { unasked: vec![allowed.id], due: vec![past.id] });
        assert_eq!(schedule.get(past.id).map(|entry| (entry.state, entry.closed)), Some((State::Due, true)));
        assert_eq!(schedule.get(later.id).map(|entry| entry.state), Some(State::Pending));
    }

    #[test]
    fn should_count_the_times_a_due_repeating_command_missed() {
        // given: hourly, due since 11:00
        let mut schedule = Schedule::default();
        let added = schedule.add(new_entry("hourly", hours(-3), Some("0 * * * *")), hours(-4)).unwrap();
        // when
        schedule.come_due(now(), Check::Running);
        // then: 12:00, 13:00 and 14:00 passed after it
        let entry = schedule.get(added.id).unwrap();
        assert_eq!((entry.missed, entry.at), (3, hours(0)));
    }

    #[test]
    fn should_move_a_repeating_command_to_its_next_time_and_keep_a_record() {
        // given
        let mut schedule = Schedule::default();
        let added = schedule.add(new_entry("daily", hours(-1), Some("0 7 * * *")), hours(-2)).unwrap();
        schedule.come_due(now(), Check::Running);
        // when
        schedule.finish(added.id, Outcome::Started { process: 4, unasked: false }, now());
        // then
        let entry = schedule.get(added.id).unwrap();
        assert_eq!(entry.state, State::Pending);
        assert_eq!(entry.at, (now() + Duration::hours(17)).timestamp());
        let history: Vec<(State, Option<u64>)> = schedule.history(now().timestamp()).iter().map(|entry| (entry.state, entry.process)).collect();
        assert_eq!(history, [(State::Started, Some(4))]);
    }

    #[test]
    fn should_end_a_one_off_command_with_its_outcome() {
        // given
        let mut schedule = Schedule::default();
        let added = schedule.add(new_entry("once", hours(-1), None), hours(-2)).unwrap();
        schedule.come_due(now(), Check::Running);
        // when
        schedule.finish(added.id, Outcome::Failed("gone".to_owned()), now());
        // then
        let entry = schedule.get(added.id).unwrap();
        assert_eq!((entry.state, entry.error.as_deref(), entry.done), (State::Failed, Some("gone"), Some(now().timestamp())));
        assert!(schedule.due().is_empty());
    }

    #[test]
    fn should_refuse_cancelling_what_ended() {
        // given
        let mut schedule = Schedule::default();
        let added = schedule.add(new_entry("once", hours(1), None), hours(0)).unwrap();
        // when
        let first = schedule.cancel(added.id, hours(0));
        let again = schedule.cancel(added.id, hours(0));
        // then
        assert_eq!(first, Ok(()));
        assert!(again.unwrap_err().contains("cancelled already"));
    }

    #[test]
    fn should_refuse_more_than_the_waiting_cap_and_bad_names() {
        // given
        let mut schedule = Schedule::default();
        for n in 0..MAX_WAITING {
            schedule.add(new_entry(&format!("c{n}"), hours(1), None), hours(0)).unwrap();
        }
        // when
        let over = schedule.add(new_entry("one more", hours(1), None), hours(0));
        let two_lines = Schedule::default().add(new_entry("a\nb", hours(1), None), hours(0));
        // then
        assert!(over.unwrap_err().contains("scheduled already"));
        assert!(two_lines.unwrap_err().contains("one line"));
    }

    #[test]
    fn should_prune_history_older_than_a_week() {
        // given
        let mut schedule = Schedule::default();
        let old = schedule.add(new_entry("old", hours(-200), None), hours(-300)).unwrap();
        schedule.cancel(old.id, hours(-200)).unwrap();
        let recent = schedule.add(new_entry("recent", hours(-2), None), hours(-3)).unwrap();
        schedule.cancel(recent.id, hours(-1)).unwrap();
        // when
        schedule.prune(hours(0));
        // then
        assert_eq!((schedule.get(old.id).is_none(), schedule.get(recent.id).is_some()), (true, true));
    }
}
