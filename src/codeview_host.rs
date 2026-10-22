//! IVE's side of `codeview::host`: its key bindings and its clipboard; and what every panel asks of IVE.

use std::rc::Rc;

use codeview::diff::{FileDiff, Headers};
use codeview::host::{ClickListener, Command, Host, KeyOutcome, KeyScope};
use codeview::panel::{Cast, Content, DiffInput, Editable, Event, Panel, ParseJob};
use codeview::syntax::ParseError;
use gpui::{App, AppContext, Entity, IntoElement, KeyDownEvent, WeakEntity};

use crate::click_targets::ClickTargets;
use crate::clipboard;
use crate::overview::git_ui::mark_label;
use crate::script::keymap::{Action, Builtin, Place};
use crate::script::state::{bound_action, invoke};

pub fn install(cx: &mut App) {
    cx.set_global(Host {
        keys: Rc::new(key_outcome),
        clipboard: Rc::new(|target, text, cx| clipboard::write(target, text, cx)),
        paste: Rc::new(|source, cx| clipboard::read(source, cx)),
        diff_mark: Rc::new(|mark, cx| mark_label(mark, crate::theme::tabs(cx), cx).into_any_element()),
        click_target: Rc::new(|name, listener: ClickListener, cx| {
            if let Some(targets) = cx.try_global::<ClickTargets>() {
                targets.register(name, listener);
            }
        }),
    });
}

/// What a codeview panel does with `event`: its bound action, if any.
fn key_outcome(event: &KeyDownEvent, scope: KeyScope, cx: &App) -> KeyOutcome {
    match decide(bound_action(event, Place::Editor, cx), scope) {
        Decision::Done(outcome) => outcome,
        Decision::Invoke { generation, function } => {
            invoke(generation, function, None, cx);
            KeyOutcome::Handled
        }
    }
}

/// What to do about a key, before anything has run.
#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Done(KeyOutcome),
    /// run the script function, then the key is handled
    Invoke { generation: u64, function: usize },
}

/// Maps the action bound to a key to what the panel does, without side effects. A viewer takes only its commands:
/// everything else propagates to the window, which looks up its own bindings.
fn decide(action: Option<Action>, scope: KeyScope) -> Decision {
    let Some(action) = action else {
        return Decision::Done(KeyOutcome::Unbound);
    };
    Decision::Done(match action {
        Action::Builtin(builtin) => match command_of(builtin) {
            Some(command) => KeyOutcome::Command(command),
            // the shown tab is an editor, not a terminal: it does nothing
            None if builtin == Builtin::Nop && scope == KeyScope::Editor => KeyOutcome::Handled,
            None => KeyOutcome::Foreign,
        },
        Action::Invoke { generation, function } if scope == KeyScope::Editor => {
            return Decision::Invoke { generation, function };
        }
        // the shown tab is an editor, not a terminal, so a bound `SendString` has nowhere to go
        Action::Send(_) if scope == KeyScope::Editor => KeyOutcome::Handled,
        // a window action (Settings, a new terminal) is the workspace's to run, and so is all a viewer leaves
        Action::Invoke { .. } | Action::Send(_) | Action::ShowTabMenu(_) | Action::Disable => KeyOutcome::Foreign,
    })
}

/// The editor's own built-ins; the rest are the workspace's.
fn command_of(builtin: Builtin) -> Option<Command> {
    Some(match builtin {
        Builtin::SelectAll => Command::SelectAll,
        Builtin::CopyTo(target) => Command::Copy(target),
        Builtin::Cut => Command::Cut,
        Builtin::PasteFrom(source) => Command::Paste(source),
        Builtin::Undo => Command::Undo,
        Builtin::Redo => Command::Redo,
        _ => return None,
    })
}

/// Answers what every panel asks of IVE, whichever tab holds it: parses on the background executor. `name` (a
/// path) says on stderr which text a failed parse was for. Call it before the panel's first [`Cast::Open`].
pub fn serve(panel: &Entity<Panel>, name: String, cx: &mut App) {
    cx.subscribe(panel, move |panel, event: &Event, cx| {
        if let Event::NeedParse(job) = event {
            parse_in_background(panel.downgrade(), job.clone(), &name, cx);
        }
    })
    .detach();
}

/// A panel showing `files` read-only.
pub fn diff_panel(files: Vec<FileDiff>, headers: Headers, cx: &mut App) -> Entity<Panel> {
    let input = DiffInput { files, headers, editable: Editable::No };
    cx.new(|cx| {
        let mut panel = Panel::new(cx);
        panel.handle(Cast::Open(Content::Diff(input)), cx);
        panel
    })
}

fn parse_in_background(panel: WeakEntity<Panel>, job: ParseJob, name: &str, cx: &mut App) {
    let (target, revision, stale) = (job.target(), job.revision(), job.cancel());
    let name = name.to_owned();
    let parsing = cx.background_executor().spawn(async move { job.run() });
    cx.spawn(async move |cx| {
        let result = parsing.await;
        // the text changed (or the panel went away) after the parse finished, before this update ran
        if stale.is_cancelled() {
            return;
        }
        match result {
            Ok(syntax) => {
                panel.update(cx, |panel, cx| panel.handle(Cast::Parsed { target, revision, syntax }, cx)).ok();
            }
            Err(ParseError::Cancelled) => {}
            Err(error) => eprintln!("ive: {name} shown without highlighting: {error}"),
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use codeview::host::ClipboardTarget;

    use super::*;

    #[test]
    fn should_map_editor_builtins_to_commands() {
        // given
        let builtins = [Builtin::Undo, Builtin::CopyTo(ClipboardTarget::Both), Builtin::SelectAll, Builtin::OpenHub];

        // when
        let commands = builtins.map(command_of);

        // then
        assert_eq!(commands, [Some(Command::Undo), Some(Command::Copy(ClipboardTarget::Both)), Some(Command::SelectAll), None]);
    }

    #[test]
    fn should_let_script_bindings_propagate_from_a_viewer() {
        // given
        let invoke = Action::Invoke { generation: 1, function: 2 };
        let actions = [invoke, Action::Send("x".into()), Action::Builtin(Builtin::Nop), Action::Builtin(Builtin::OpenHub)];

        // when
        let decisions = actions.map(|action| decide(Some(action), KeyScope::Viewer));

        // then
        assert_eq!(decisions, [(); 4].map(|_| Decision::Done(KeyOutcome::Foreign)));
        assert_eq!(decide(None, KeyScope::Viewer), Decision::Done(KeyOutcome::Unbound));
    }

    #[test]
    fn should_run_script_bindings_in_the_editor() {
        // given
        let invoke = Action::Invoke { generation: 1, function: 2 };

        // when
        let invoked = decide(Some(invoke), KeyScope::Editor);
        let sent = decide(Some(Action::Send("x".into())), KeyScope::Editor);

        // then
        assert_eq!(invoked, Decision::Invoke { generation: 1, function: 2 });
        assert_eq!(sent, Decision::Done(KeyOutcome::Handled));
    }

    #[test]
    fn should_map_copy_in_a_viewer_to_a_command() {
        // given
        let copy = Action::Builtin(Builtin::CopyTo(ClipboardTarget::Both));

        // when
        let decision = decide(Some(copy), KeyScope::Viewer);

        // then
        assert_eq!(decision, Decision::Done(KeyOutcome::Command(Command::Copy(ClipboardTarget::Both))));
    }
}
