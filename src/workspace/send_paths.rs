//! Send to Terminal: paths typed into a terminal picked from a list of the terminals, most recently used first;
//! relative to the terminal's directory when it works in their checkout, else absolute.

use std::path::PathBuf;

use gpui::{Context, Window};

use super::Workspace;
use crate::menu::resolve::{Applies, Choice};
use crate::menu::{Home, Subject};
use crate::path_text::typed_into;
use crate::process::{ProcessId, ProcessKind};
use crate::terminal_use::{Candidate, Place};

/// The choice of Send to Terminal that opens a new terminal first.
const NEW_TERMINAL: &str = "new";

impl Workspace {
    /// Send to Terminal's submenu for `subject`: its terminals, then a new one in its checkout.
    pub(super) fn terminal_choices(&self, subject: &Subject) -> Applies {
        if subject.paths.is_empty() {
            return Applies::Hidden;
        }
        let terminals = self.terminal_use.by_recency(self.processes.iter().map(|process| Candidate { id: process.id, place: place(process.kind, process.is_live()) }));
        let mut choices: Vec<Choice> = terminals.iter().filter_map(|id| self.terminal_choice(*id)).collect();
        if let Home::Checkout(path) = &subject.home {
            let label = self.checkout_ref_of(path).map_or_else(|| path.display().to_string(), |checkout| self.checkout_label(&checkout));
            choices.push(Choice::Separator);
            choices.push(Choice::Item { label: format!("New Terminal in {label}"), choice: NEW_TERMINAL.to_owned(), enabled: true });
        }
        if choices.is_empty() {
            choices.push(Choice::Item { label: "No terminals".to_owned(), choice: String::new(), enabled: false });
        }
        Applies::Choices(choices)
    }

    fn terminal_choice(&self, id: ProcessId) -> Option<Choice> {
        let process = self.processes.get(id)?;
        let place = process.checkout.as_ref().map_or_else(|| "home".to_owned(), |checkout| self.checkout_label(checkout));
        Some(Choice::Item { label: format!("{} · {place}", process.title()), choice: id.value().to_string(), enabled: true })
    }

    /// Runs Send to Terminal's `choice` for `subject`: a terminal by its process id, or a new one.
    pub(super) fn send_paths_to(&mut self, choice: &str, subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = subject.paths.iter().map(|path| path.path.clone()).collect();
        if choice == NEW_TERMINAL {
            return self.send_paths_to_new_terminal(&paths, subject, window, cx);
        }
        let Some(id) = choice.parse::<u64>().ok().and_then(|value| self.processes.iter().map(|process| process.id).find(|id| id.value() == value)) else {
            return;
        };
        self.send_paths(&paths, id, window, cx);
    }

    /// Pastes `paths` into terminal `id` and shows it; one that ended in the meantime gets nothing.
    pub(super) fn send_paths(&mut self, paths: &[PathBuf], id: ProcessId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(process) = self.processes.get(id).filter(|process| process.is_live()) else {
            return;
        };
        process.view.paste(&self.typed_for_process(paths, id));
        self.show_process(id, window, cx);
    }

    fn send_paths_to_new_terminal(&mut self, paths: &[PathBuf], subject: &Subject, window: &mut Window, cx: &mut Context<Self>) {
        let Home::Checkout(root) = &subject.home else {
            return;
        };
        let checkout = self.checkout_ref_of(root);
        let tab = self.open_terminal_in(checkout, window, cx);
        let typed = typed_into(paths, &self.all_checkouts(), (Some(root.as_path()), Some(root.as_path())));
        // the shell reads it once it starts, as a scratch's run command
        if let Err(error) = self.send_text_to_process(tab, &typed) {
            eprintln!("ive: cannot type the paths into the new terminal: {error}");
        }
    }

    /// What typing `paths` into terminal `id` writes: relative to its directory when it works in their checkout.
    pub(crate) fn typed_for_process(&self, paths: &[PathBuf], id: ProcessId) -> String {
        let process = self.processes.get(id);
        let checkout = process.and_then(|process| process.checkout.as_ref()).map(|checkout| checkout.path.as_path());
        let directory = process.and_then(|process| process.directory());
        typed_into(paths, &self.all_checkouts(), (checkout, directory))
    }

    /// The most recently used live terminal, for the hub's Alt+Enter.
    pub(super) fn last_terminal(&self) -> Option<ProcessId> {
        let candidates = self.processes.iter().map(|process| Candidate { id: process.id, place: place(process.kind, process.is_live()) });
        self.terminal_use.by_recency(candidates).first().copied()
    }
}

fn place(kind: ProcessKind, live: bool) -> Place {
    match kind {
        ProcessKind::Terminal | ProcessKind::ClaudeCode if live => Place::Terminal,
        _ => Place::Run,
    }
}
