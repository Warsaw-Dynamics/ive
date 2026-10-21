//! The Restart button: types `<script> --replace <pid>` into a terminal of its own, which later presses reuse. The
//! script builds IVE; after a good build it sends this IVE SIGUSR1, which quits leaving the terminals running, and
//! starts the new IVE. A failed build stays on screen in that terminal.

use std::path::{Path, PathBuf};

use gpui::{Context, PromptLevel, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::process::ProcessId;

/// Marks the hosted session the Restart button types into, so the restarted IVE reuses it.
const RESTART_LABEL: &str = "ive.restart";

impl Workspace {
    pub fn restart_script(&self) -> Option<&Path> {
        self.restart_script.as_deref()
    }

    /// The Restart button. An IVE that does not keep its terminals (another one does) asks first, since
    /// restarting ends them.
    pub(crate) fn restart(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.keeps_terminals() || self.processes.iter().next().is_none() {
            return self.run_restart_script(window, cx);
        }
        let detail = "This IVE does not keep its terminals; restarting ends them.";
        let answer = window.prompt(PromptLevel::Warning, "Restart IVE?", Some(detail), &["Restart", "Cancel"], cx);
        cx.spawn_in(window, async move |this, cx| {
            if answer.await == Ok(0) {
                this.update_in(cx, |workspace, window, cx| workspace.run_restart_script(window, cx)).ok();
            }
        })
        .detach();
    }

    #[cfg(target_os = "linux")]
    fn keeps_terminals(&self) -> bool {
        self.hosted.is_some()
    }

    /// `--restart` is refused outside Linux, so this is never asked there.
    #[cfg(not(target_os = "linux"))]
    fn keeps_terminals(&self) -> bool {
        false
    }

    /// Types the start script into the restart terminal, starting one when there is none. While a program runs
    /// there (the script building), a press only shows it.
    fn run_restart_script(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(script) = self.restart_script.clone() else {
            return;
        };
        let id = match self.restart_terminal.filter(|id| self.processes.get(*id).is_some()) {
            Some(id) if self.runs_program(id) => return self.show_process(id, window, cx),
            Some(id) => {
                self.show_process(id, window, cx);
                id
            }
            None => match self.open_restart_terminal(&script, window, cx) {
                Some(id) => id,
                None => return,
            },
        };
        if let Some(tab) = self.tabs.find(|tab| tab.shows_process(id)) {
            // the shell reads it once started; the PTY holds it until then
            let _ = self.send_text_to_process(tab, &replace_command(&script, std::process::id()));
        }
    }

    /// A shell in the script's folder; `None` when it could not start, which its failed tab explains.
    fn open_restart_terminal(&mut self, script: &Path, window: &mut Window, cx: &mut Context<Self>) -> Option<ProcessId> {
        let directory = script.parent().map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        let checkout = self.checkout_of_path(&directory);
        let labels = vec![(RESTART_LABEL.to_owned(), "1".to_owned())];
        let tab = self.open_terminal(checkout, &directory, labels, window, cx);
        let Some(Tab::Process(id)) = self.tabs.get(tab) else {
            return None;
        };
        self.restart_terminal = Some(*id);
        self.restart_terminal
    }

    #[cfg(unix)]
    fn runs_program(&self, id: ProcessId) -> bool {
        let probe = self.processes.get(id).and_then(|process| process.view.foreground());
        probe.is_some_and(|probe| probe.program_group().is_some())
    }

    /// Without Unix process groups IVE cannot tell; `--restart` is Linux only anyway.
    #[cfg(not(unix))]
    fn runs_program(&self, _id: ProcessId) -> bool {
        false
    }
}

/// Whether a hosted session's labels mark it as the restart terminal.
pub(super) fn is_restart_terminal(labels: &[(String, String)]) -> bool {
    labels.iter().any(|(key, _)| key == RESTART_LABEL)
}

/// The line typed into the restart terminal, Enter included. Single quotes keep any path one word in sh, bash, zsh
/// and fish alike.
fn replace_command(script: &Path, pid: u32) -> String {
    let quoted = script.display().to_string().replace('\'', r"'\''");
    format!("'{quoted}' --replace {pid}\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_type_the_script_with_this_ive_to_replace() {
        // given
        let script = Path::new("/home/me/ive/restart.sh");

        // when
        let command = replace_command(script, 4242);

        // then
        assert_eq!(command, "'/home/me/ive/restart.sh' --replace 4242\r");
    }

    #[test]
    fn should_quote_a_script_path_with_spaces_and_quotes() {
        // given
        let script = Path::new("/home/me/it's mine/restart.sh");

        // when
        let command = replace_command(script, 7);

        // then
        assert_eq!(command, "'/home/me/it'\\''s mine/restart.sh' --replace 7\r");
    }

    #[test]
    fn should_recognise_the_restart_terminal_by_its_label() {
        // given
        let restart = vec![("ive.project".to_owned(), "p1".to_owned()), (RESTART_LABEL.to_owned(), "1".to_owned())];
        let plain = vec![("ive.project".to_owned(), "p1".to_owned())];

        // when
        let marked = [is_restart_terminal(&restart), is_restart_terminal(&plain)];

        // then
        assert_eq!(marked, [true, false]);
    }
}
