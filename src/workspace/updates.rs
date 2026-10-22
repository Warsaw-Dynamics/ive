//! Updates in the running IVE: checking now and then, the Restart to update button, restarting when idle, and
//! swapping the staged release in on quit. The work itself is in `crate::update`.

use std::ffi::OsString;
use std::time::{Duration, Instant};

use gpui::{AppContext as _, Context, PromptLevel, Task, Window};

use super::Workspace;
use super::quit::QuitEvent;
use super::saves::saves_landed;
use crate::settings::{AutoUpdate, SettingsStore};
use crate::update::install::Install;
use crate::update::restart::{Focus, RestartConditions, RestartVerdict, button_verdict, idle_verdict, relaunch, relaunch_args};
use crate::update::staging::{Running, Staged, others_running, own_protocol};
use crate::update::updater::{Applied, Check, CheckOutcome, FEED, UpdateState, apply_staged, check_and_stage, discard};
use crate::update::{Platform, RELEASE_TAG};

/// The first check waits for IVE to settle after starting.
const FIRST_CHECK: Duration = Duration::from_secs(60);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const IDLE_TICK: Duration = Duration::from_secs(60);
const IDLE_LIMIT: Duration = Duration::from_secs(30 * 60);
/// Overrides [`FEED`], for trying updates without GitHub.
const FEED_ENV: &str = "IVE_UPDATE_FEED";
/// Overrides [`IDLE_LIMIT`], in seconds.
const IDLE_ENV: &str = "IVE_UPDATE_IDLE_SECONDS";

/// Updates as this build has them.
pub(super) enum Updates {
    /// built from source: it updates by rebuilding
    SourceBuild,
    /// a release that cannot update itself, and why
    Unavailable(String),
    Release(Box<ReleaseUpdates>),
}

pub(super) struct ReleaseUpdates {
    tag: &'static str,
    platform: Platform,
    install: Install,
    feed: String,
    idle_limit: Duration,
    state: UpdateState,
    last_activity: Instant,
    _running: Running,
    _checks: Task<()>,
    _idle: Task<()>,
}

/// Updates for this build, with their timers started for `workspace`.
pub(super) fn start(window: &mut Window, cx: &mut Context<Workspace>) -> Updates {
    let Some(tag) = RELEASE_TAG else {
        return Updates::SourceBuild;
    };
    match release_install() {
        Ok((platform, install, running)) => Updates::Release(Box::new(ReleaseUpdates {
            tag,
            platform,
            install,
            feed: std::env::var(FEED_ENV).unwrap_or_else(|_| FEED.to_owned()),
            idle_limit: idle_limit(std::env::var(IDLE_ENV).ok().as_deref()),
            state: UpdateState::NotChecked,
            last_activity: Instant::now(),
            _running: running,
            _checks: check_now_and_then(window, cx),
            _idle: watch_idle(window, cx),
        })),
        Err(reason) => {
            eprintln!("ive: update: {reason}");
            Updates::Unavailable(reason)
        }
    }
}

fn release_install() -> Result<(Platform, Install, Running), String> {
    let platform = Platform::current().ok_or_else(|| format!("no releases are built for {} {}", std::env::consts::OS, std::env::consts::ARCH))?;
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).map_err(|error| format!("cannot tell where IVE is installed: {error}"))?;
    let install = Install::of_exe(&exe);
    install.check_writable()?;
    let dir = install.staging_dir();
    std::fs::create_dir_all(&dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let running = Running::register(&dir)?;
    Ok((platform, install, running))
}

/// `IVE_UPDATE_IDLE_SECONDS`, when it is a whole number of seconds from 10 to a day; 30 minutes otherwise.
fn idle_limit(value: Option<&str>) -> Duration {
    let seconds = value.and_then(|value| value.trim().parse::<u64>().ok()).filter(|seconds| (10..=86_400).contains(seconds));
    seconds.map_or(IDLE_LIMIT, Duration::from_secs)
}

fn check_now_and_then(window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        cx.background_executor().timer(FIRST_CHECK).await;
        while this.update(cx, |workspace, cx| workspace.check_for_update(cx)).is_ok() {
            cx.background_executor().timer(CHECK_INTERVAL).await;
        }
    })
}

fn watch_idle(window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        loop {
            cx.background_executor().timer(IDLE_TICK).await;
            if this.update_in(cx, |workspace, window, cx| workspace.idle_tick(window, cx)).is_err() {
                return;
            }
        }
    })
}

fn is_on(cx: &gpui::App) -> bool {
    cx.global::<SettingsStore>().current().auto_update.is_on()
}

impl Workspace {
    /// A key, a click or an MCP call: IVE is not idle.
    pub(super) fn note_activity(&mut self) {
        if let Updates::Release(updates) = &mut self.updates {
            updates.last_activity = Instant::now();
        }
    }

    /// What the Settings tab says about updates.
    pub fn update_status(&self) -> String {
        match &self.updates {
            Updates::SourceBuild => "Built from source: it updates by rebuilding".to_owned(),
            Updates::Unavailable(reason) => format!("Cannot update: {reason}"),
            Updates::Release(updates) => updates.state.describe(),
        }
    }

    /// The staged release Restart to update switches to: one whose terminals carry over.
    pub fn update_to_restart_onto(&self) -> Option<&str> {
        let Updates::Release(updates) = &self.updates else {
            return None;
        };
        match &updates.state {
            UpdateState::Ready(staged) if staged.protocol == own_protocol() => Some(&staged.tag),
            _ => None,
        }
    }

    pub(super) fn check_for_update(&mut self, cx: &mut Context<Self>) {
        let on = is_on(cx);
        let Updates::Release(updates) = &mut self.updates else {
            return;
        };
        if !on {
            updates.state = UpdateState::Off;
            return cx.notify();
        }
        if updates.state == UpdateState::Checking {
            return;
        }
        updates.state = UpdateState::Checking;
        cx.notify();
        let (install, tag, feed, platform) = (updates.install.clone(), updates.tag, updates.feed.clone(), updates.platform);
        cx.spawn(async move |this, cx| {
            let checked = cx.background_spawn(async move { check_and_stage(&Check { install: &install, own_tag: tag, feed: &feed, platform }) }).await;
            let _ = this.update(cx, |workspace, cx| workspace.checked(checked, cx));
        })
        .detach();
    }

    fn checked(&mut self, checked: Result<CheckOutcome, String>, cx: &mut Context<Self>) {
        let Updates::Release(updates) = &mut self.updates else {
            return;
        };
        updates.state = match checked {
            Ok(CheckOutcome::UpToDate) => UpdateState::UpToDate,
            Ok(CheckOutcome::Staged(staged)) => UpdateState::Ready(staged),
            // another IVE on this install is staging; what it stages shows at the next check
            Ok(CheckOutcome::Busy) => Staged::load(&updates.install.staging_dir()).map_or(UpdateState::NotChecked, UpdateState::Ready),
            Err(reason) => {
                eprintln!("ive: update: {reason}");
                UpdateState::Failed(reason)
            }
        };
        cx.notify();
    }

    fn restart_conditions(&self, updates: &ReleaseUpdates, cx: &gpui::App) -> RestartConditions {
        let staged = match &updates.state {
            UpdateState::Ready(staged) => Some(staged),
            _ => None,
        };
        RestartConditions {
            staged: staged.is_some() && is_on(cx),
            idle: updates.last_activity.elapsed(),
            idle_limit: updates.idle_limit,
            others_running: others_running(&updates.install.staging_dir()),
            protocol_matches: staged.is_some_and(|staged| staged.protocol == own_protocol()),
            unsaved: self.unsaved_tab(cx).is_some(),
            prompt_shown: self.quit_prompt.is_some(),
            terminals_at_risk: !self.keeps_terminals() && self.processes.iter().next().is_some(),
        }
    }

    /// Once a minute: restarts onto a staged update after long enough without input.
    fn idle_tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Updates::Release(updates) = &self.updates else {
            return;
        };
        let conditions = self.restart_conditions(updates, cx);
        match idle_verdict(&conditions) {
            RestartVerdict::Restart => self.restart_onto_update(Focus::Keep, window, cx),
            // edits a failed or pending save left; saving them lets the next tick restart
            RestartVerdict::Wait("unsaved edits") if conditions.idle >= conditions.idle_limit => self.flush_edits(cx),
            RestartVerdict::Wait(_) => {}
        }
    }

    /// Restart to update: saves edits first, then restarts onto the staged release, or says why it cannot.
    pub fn restart_to_update(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_edits(cx);
        cx.spawn_in(window, async move |this, cx| {
            saves_landed(&this, cx).await;
            let _ = this.update_in(cx, |workspace, window, cx| workspace.restart_when_saved(window, cx));
        })
        .detach();
    }

    fn restart_when_saved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.unsaved_tab(cx) {
            return self.activate_tab(tab, window, cx);
        }
        let Updates::Release(updates) = &self.updates else {
            return;
        };
        match button_verdict(&self.restart_conditions(updates, cx)) {
            RestartVerdict::Restart => self.restart_onto_update(Focus::Take, window, cx),
            RestartVerdict::Wait(reason) => drop(window.prompt(PromptLevel::Info, "Cannot restart to update now", Some(reason), &["OK"], cx)),
        }
    }

    /// Swaps the staged release in, starts it in place of this IVE, and quits leaving the terminals running.
    fn restart_onto_update(&mut self, focus: Focus, window: &mut Window, cx: &mut Context<Self>) {
        let Updates::Release(updates) = &mut self.updates else {
            return;
        };
        match apply_staged(&updates.install) {
            Ok(Applied::Swapped(_)) => {}
            Ok(Applied::NothingStaged | Applied::NotNow(_)) => return,
            Err(reason) => return self.update_failed(reason, cx),
        }
        let original: Vec<OsString> = std::env::args_os().skip(1).collect();
        let cwd = std::env::current_dir().unwrap_or_default();
        if let Err(reason) = relaunch(&updates.install, &relaunch_args(&original, &cwd, std::process::id(), focus)) {
            // the install is the new release already: the next start runs it
            return self.update_failed(reason, cx);
        }
        self.quit_event(QuitEvent::Replaced, window, cx);
    }

    fn update_failed(&mut self, reason: String, cx: &mut Context<Self>) {
        eprintln!("ive: update: {reason}");
        if let Updates::Release(updates) = &mut self.updates {
            updates.state = UpdateState::Failed(reason);
        }
        cx.notify();
    }

    /// On quit: the staged release replaces the install, so the next start runs it.
    pub(super) fn apply_update_on_quit(&self, cx: &gpui::App) {
        let Updates::Release(updates) = &self.updates else {
            return;
        };
        if !is_on(cx) {
            return;
        }
        match apply_staged(&updates.install) {
            Ok(Applied::Swapped(tag)) => eprintln!("ive: updated to {tag}; it starts next time"),
            Ok(Applied::NotNow(reason)) => eprintln!("ive: update: not applied on quit: {reason}"),
            Ok(Applied::NothingStaged) => {}
            Err(reason) => eprintln!("ive: update: {reason}"),
        }
    }

    /// Settings → Updates: turning updates off drops what is staged; turning them on checks at once.
    pub(super) fn auto_update_changed(&mut self, auto_update: AutoUpdate, cx: &mut Context<Self>) {
        let Updates::Release(updates) = &mut self.updates else {
            return;
        };
        match auto_update {
            AutoUpdate::Off => {
                discard(&updates.install.staging_dir());
                updates.state = UpdateState::Off;
                cx.notify();
            }
            AutoUpdate::On => {
                updates.state = UpdateState::NotChecked;
                self.check_for_update(cx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_take_the_idle_limit_from_the_environment_within_bounds() {
        // given
        let values = [None, Some("90"), Some("5"), Some("ten"), Some("100000")];

        // when
        let limits = values.map(idle_limit);

        // then
        assert_eq!(limits, [IDLE_LIMIT, Duration::from_secs(90), IDLE_LIMIT, IDLE_LIMIT, IDLE_LIMIT]);
    }
}
