//! Quitting IVE: with terminals that outlive it running, a prompt asks whether to stop them, and counts down to
//! leaving them running.

use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{ClickEvent, Context, Div, FocusHandle, KeyDownEvent, Task, Window, div, prelude::*};
#[cfg(target_os = "macos")]
use gpui::{App, KeyBinding, Menu, MenuItem};

use super::Workspace;
use crate::click_targets::target;
use crate::prompt;
use crate::script::quit_countdown::QuitCountdown;
use crate::theme::Theme;

/// Seconds the prompt waits before IVE quits and leaves the processes running, until `ive.rn` has loaded.
pub const COUNTDOWN_SECONDS: u8 = 10;

#[cfg(target_os = "macos")]
gpui::actions!(ive, [Quit]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitEvent {
    /// the window's ×, or the window manager closing the window
    CloseRequested,
    /// SIGTERM or SIGINT
    Signal,
    /// SIGUSR1 (Windows: the event `restart.ps1` sets): a restart script built the new IVE and starts it once this
    /// one quits
    Replaced,
    StopAll,
    KeepRunning,
    Cancel,
    /// a second of the countdown passed
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitStep {
    /// quit; hosted processes keep running
    QuitNow,
    ShowPrompt,
    StopAllThenQuit,
    /// show this many seconds left
    Countdown(u8),
    /// close the prompt and stay
    Dismiss,
    Ignore,
}

/// What quitting does with hosted processes still running.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OnQuit {
    /// show the quit prompt
    #[default]
    Ask,
    /// `--stop-processes-on-quit`: stop them all without asking
    StopProcesses,
}

impl OnQuit {
    /// `step` under this choice: where the prompt would show, the processes stop and IVE quits.
    pub fn apply(self, step: QuitStep) -> QuitStep {
        match (self, step) {
            (Self::StopProcesses, QuitStep::ShowPrompt) => QuitStep::StopAllThenQuit,
            _ => step,
        }
    }
}

/// What `event` does with `running` hosted processes, while the prompt shows `prompt` seconds left (or no prompt).
/// A second signal while the prompt is up quits at once, keeping the processes, like the countdown's end; being
/// replaced quits at once, keeping them, the new IVE adopts them.
pub fn next_step(running: usize, prompt: Option<u8>, event: QuitEvent) -> QuitStep {
    match (prompt, event) {
        (_, QuitEvent::Replaced) => QuitStep::QuitNow,
        (None, QuitEvent::CloseRequested | QuitEvent::Signal) if running == 0 => QuitStep::QuitNow,
        (None, QuitEvent::CloseRequested | QuitEvent::Signal) => QuitStep::ShowPrompt,
        (None, _) => QuitStep::Ignore,
        (Some(seconds), QuitEvent::Tick) if seconds > 1 => QuitStep::Countdown(seconds - 1),
        (Some(_), QuitEvent::Tick | QuitEvent::Signal | QuitEvent::KeepRunning) => QuitStep::QuitNow,
        (Some(_), QuitEvent::StopAll) => QuitStep::StopAllThenQuit,
        (Some(_), QuitEvent::Cancel) => QuitStep::Dismiss,
        (Some(_), QuitEvent::CloseRequested) => QuitStep::Ignore,
    }
}

/// The prompt while it shows: seconds left before IVE quits keeping the processes.
pub struct QuitPrompt {
    seconds: u8,
    focus: FocusHandle,
    _tick: Task<()>,
}

const DETAIL: &str = "They keep running and come back next time IVE starts.";

#[cfg(target_os = "macos")]
pub fn add_app_menu(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    cx.set_menus([Menu::new("IVE").items([MenuItem::action("Quit IVE", Quit)])]);
    cx.on_action(|_: &Quit, cx| cx.defer(request_quit_of_workspace));
}

#[cfg(target_os = "macos")]
fn request_quit_of_workspace(cx: &mut App) {
    match main_window(cx) {
        Some(workspace) => drop(workspace.update(cx, |workspace, window, cx| workspace.request_quit(window, cx))),
        None => cx.quit(),
    }
}

#[cfg(target_os = "macos")]
pub fn show_main_window(cx: &mut App) {
    if let Some(workspace) = main_window(cx) {
        drop(workspace.update(cx, |_, window, _| window.activate_window()));
    }
}

#[cfg(target_os = "macos")]
fn main_window(cx: &App) -> Option<gpui::WindowHandle<Workspace>> {
    cx.windows().into_iter().find_map(|window| window.downcast::<Workspace>())
}

impl Workspace {
    /// The window's × or the window manager asked to close: quit, or ask first while hosted processes run.
    pub fn request_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.quit_event(QuitEvent::CloseRequested, window, cx);
    }

    #[cfg(target_os = "macos")]
    pub(super) fn quit_requested_by_system(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let seconds = self.quit_prompt.as_ref().map(|prompt| prompt.seconds);
        // unsaved edits quit through the workspace, which saves them first
        let quits = self.on_quit.apply(next_step(self.hosted_process_count(), seconds, QuitEvent::CloseRequested)) == QuitStep::QuitNow;
        if quits && self.unsaved_tab(cx).is_none() {
            return true;
        }
        self.request_quit(window, cx);
        false
    }

    pub(super) fn quit_event(&mut self, event: QuitEvent, window: &mut Window, cx: &mut Context<Self>) {
        let seconds = self.quit_prompt.as_ref().map(|prompt| prompt.seconds);
        match self.on_quit.apply(next_step(self.hosted_process_count(), seconds, event)) {
            // dropping the hosted sessions detaches them; their shells keep running
            QuitStep::QuitNow => self.quit_when_saved(window, cx),
            QuitStep::ShowPrompt => self.show_quit_prompt(window, cx),
            QuitStep::StopAllThenQuit => {
                self.stop_all_processes(window, cx);
                self.quit_when_saved(window, cx);
            }
            QuitStep::Countdown(left) => {
                if let Some(prompt) = self.quit_prompt.as_mut() {
                    prompt.seconds = left;
                }
                cx.notify();
            }
            QuitStep::Dismiss => {
                self.quit_prompt = None;
                cx.cancel_quit();
                window.focus(&self.focus_handle, cx);
                cx.notify();
            }
            QuitStep::Ignore => {}
        }
    }

    fn show_quit_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.activate_window();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let tick = cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this.update_in(cx, |workspace, window, cx| workspace.quit_event(QuitEvent::Tick, window, cx)).is_err() {
                    return;
                }
            }
        });
        let seconds = cx.try_global::<QuitCountdown>().map_or(COUNTDOWN_SECONDS, |countdown| countdown.seconds());
        self.quit_prompt = Some(QuitPrompt { seconds, focus, _tick: tick });
        cx.notify();
    }

    fn stop_all_processes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<_> = self.processes.iter().map(|process| process.id).collect();
        for id in ids {
            self.end_process(id, window, cx);
        }
    }

    /// The quit prompt over the whole window while it shows. Keep running is the primary button, as the countdown
    /// ends there; Enter presses it, Escape cancels.
    pub(super) fn render_quit_prompt(&self, theme: Theme, cx: &mut Context<Self>) -> Option<Div> {
        let prompt = self.quit_prompt.as_ref()?;
        let answer = |index: usize, name: &str, label: String, event: QuitEvent| {
            let listener = cx.listener(move |workspace, _: &ClickEvent, window, cx| workspace.quit_event(event, window, cx));
            prompt::button(index, label.into(), theme).on_click(target(format!("prompt:{name}"), listener, cx))
        };
        let buttons = vec![
            answer(0, "Keep running", format!("Keep running ({})", prompt.seconds), QuitEvent::KeepRunning),
            answer(1, "Stop all", "Stop all".to_owned(), QuitEvent::StopAll),
            answer(2, "Cancel", "Cancel".to_owned(), QuitEvent::Cancel),
        ];
        let panel = prompt::dialog(theme, prompt_message(self.hosted_process_count()), Some(DETAIL.to_owned()), buttons)
            .track_focus(&prompt.focus)
            .on_key_down(cx.listener(|workspace, event: &KeyDownEvent, window, cx| {
                let quit_event = match event.keystroke.key.as_str() {
                    "enter" => QuitEvent::KeepRunning,
                    "escape" => QuitEvent::Cancel,
                    _ => return,
                };
                cx.stop_propagation();
                workspace.quit_event(quit_event, window, cx);
            }));
        Some(div().absolute().top_0().left_0().size_full().child(prompt::scrim(panel, theme)))
    }
}

/// Forwards SIGTERM, SIGINT and SIGUSR1 to the quit flow for as long as the workspace lives.
pub(super) fn forward_signals(mut signals: UnboundedReceiver<i32>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        while let Some(signal) = signals.next().await {
            let event = quit_event_of(signal);
            if this.update_in(cx, |workspace, window, cx| workspace.quit_event(event, window, cx)).is_err() {
                return;
            }
        }
    })
}

#[cfg(unix)]
fn quit_event_of(signal: i32) -> QuitEvent {
    match signal {
        libc::SIGUSR1 => QuitEvent::Replaced,
        _ => QuitEvent::Signal,
    }
}

/// Signals are caught on Unix only.
#[cfg(windows)]
fn quit_event_of(signal: i32) -> QuitEvent {
    match signal {
        crate::signals::REPLACED => QuitEvent::Replaced,
        _ => QuitEvent::Signal,
    }
}

pub fn prompt_message(running: usize) -> String {
    match running {
        1 => "Stop 1 running process?".to_owned(),
        count => format!("Stop {count} running processes?"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_quit_at_once_without_hosted_processes() {
        // given
        let running = 0;

        // when
        let steps = [QuitEvent::CloseRequested, QuitEvent::Signal].map(|event| next_step(running, None, event));

        // then
        assert_eq!(steps, [QuitStep::QuitNow, QuitStep::QuitNow]);
    }

    #[test]
    fn should_show_prompt_with_hosted_processes() {
        // given
        let running = 2;

        // when
        let steps = [QuitEvent::CloseRequested, QuitEvent::Signal].map(|event| next_step(running, None, event));

        // then
        assert_eq!(steps, [QuitStep::ShowPrompt, QuitStep::ShowPrompt]);
    }

    #[test]
    fn should_count_down_then_quit() {
        // given
        let running = 1;

        // when
        let steps = [Some(4), Some(2), Some(1)].map(|seconds| next_step(running, seconds, QuitEvent::Tick));

        // then
        assert_eq!(steps, [QuitStep::Countdown(3), QuitStep::Countdown(1), QuitStep::QuitNow]);
    }

    #[test]
    fn should_quit_keeping_processes_on_second_signal() {
        // given
        let prompt = Some(3);

        // when
        let step = next_step(1, prompt, QuitEvent::Signal);

        // then
        assert_eq!(step, QuitStep::QuitNow);
    }

    #[test]
    fn should_quit_keeping_processes_when_replaced() {
        // given
        let prompts = [None, Some(3)];

        // when
        let steps = prompts.map(|prompt| next_step(2, prompt, QuitEvent::Replaced));

        // then
        assert_eq!(steps, [QuitStep::QuitNow, QuitStep::QuitNow]);
    }

    #[cfg(unix)]
    #[test]
    fn should_take_sigusr1_for_being_replaced() {
        // given
        let signals = [libc::SIGUSR1, libc::SIGTERM, libc::SIGINT];

        // when
        let events = signals.map(quit_event_of);

        // then
        assert_eq!(events, [QuitEvent::Replaced, QuitEvent::Signal, QuitEvent::Signal]);
    }

    #[test]
    fn should_answer_the_prompt_buttons() {
        // given
        let prompt = Some(3);

        // when
        let steps = [QuitEvent::StopAll, QuitEvent::KeepRunning, QuitEvent::Cancel].map(|event| next_step(1, prompt, event));

        // then
        assert_eq!(steps, [QuitStep::StopAllThenQuit, QuitStep::QuitNow, QuitStep::Dismiss]);
    }

    #[test]
    fn should_ignore_repeated_close_and_stray_answers() {
        // given
        let shown = Some(3);

        // when
        let while_shown = next_step(1, shown, QuitEvent::CloseRequested);
        let without_prompt = [QuitEvent::Tick, QuitEvent::StopAll, QuitEvent::KeepRunning, QuitEvent::Cancel].map(|event| next_step(1, None, event));

        // then
        assert_eq!(while_shown, QuitStep::Ignore);
        assert_eq!(without_prompt, [QuitStep::Ignore; 4]);
    }

    #[test]
    fn should_stop_processes_instead_of_prompting_when_asked_to() {
        // given
        let steps = [QuitStep::ShowPrompt, QuitStep::QuitNow, QuitStep::Countdown(3), QuitStep::Dismiss];

        // when
        let stopping = steps.map(|step| OnQuit::StopProcesses.apply(step));
        let asking = steps.map(|step| OnQuit::Ask.apply(step));

        // then
        assert_eq!(stopping, [QuitStep::StopAllThenQuit, QuitStep::QuitNow, QuitStep::Countdown(3), QuitStep::Dismiss]);
        assert_eq!(asking, steps);
    }

    #[test]
    fn should_word_the_prompt_by_count() {
        // given
        let counts = [1, 3];

        // when
        let messages = counts.map(prompt_message);

        // then
        assert_eq!(messages, ["Stop 1 running process?".to_owned(), "Stop 3 running processes?".to_owned()]);
    }
}
