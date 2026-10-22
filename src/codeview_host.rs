//! IVE's side of `codeview::host`: its key bindings and its clipboard; and what every panel asks of IVE.

use std::path::PathBuf;
use std::rc::Rc;

use codeview::changes::ChangeDiffJob;
use codeview::diff::{FileDiff, Headers};
use codeview::host::{ClickListener, Command, Host, KeyOutcome, KeyScope};
use codeview::panel::{Cast, Content, DiffInput, DiffLayout, Editable, Event, Panel, ParseJob, TextsRequest};
use codeview::syntax::ParseError;
use gpui::{App, AppContext, BorrowAppContext, Entity, IntoElement, KeyDownEvent, WeakEntity};

use crate::click_targets::ClickTargets;
use crate::clipboard;
use crate::git::diff::{self, FilePaths, Sides};
use crate::overview::git_ui::mark_label;
use crate::script::keymap::{Action, Builtin, Place};
use crate::script::state::{bound_action, invoke};
use crate::settings::SettingsStore;

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
        tooltip: Rc::new(|text, window, cx| crate::tooltip::text_tooltip(text)(window, cx)),
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
        Action::Invoke { .. } | Action::Send(_) | Action::ShowTabMenu(_) | Action::ShowTreeMenu(_) | Action::Disable => {
            KeyOutcome::Foreign
        }
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
        Builtin::ToggleDiffLayout => Command::ToggleLayout,
        _ => return None,
    })
}

/// Answers what every panel asks of IVE, whichever tab holds it: parses and change diffs on the background
/// executor. `name` (a path) says on stderr which text a failed parse was for. Call it before the panel's first
/// [`Cast::Open`].
pub fn serve(panel: &Entity<Panel>, name: String, cx: &mut App) {
    cx.subscribe(panel, move |panel, event: &Event, cx| match event {
        Event::NeedParse(job) => parse_in_background(panel.downgrade(), job.clone(), &name, cx),
        Event::NeedChangeDiff(job) => diff_in_background(panel.downgrade(), job.clone(), cx),
        _ => {}
    })
    .detach();
}

/// Runs a change markers' line diff off the UI thread and casts what it found to `panel`.
fn diff_in_background(panel: WeakEntity<Panel>, job: ChangeDiffJob, cx: &mut App) {
    let revision = job.revision();
    let diffing = cx.background_executor().spawn(async move { job.run() });
    cx.spawn(async move |cx| {
        let hunks = diffing.await;
        panel.update(cx, |panel, cx| panel.handle(Cast::ChangeDiff { revision, hunks }, cx)).ok();
    })
    .detach();
}

/// Where a diff's texts are read: its repository, and the sides its files are diffed between.
#[derive(Debug, Clone)]
pub struct DiffOrigin {
    pub dir: PathBuf,
    pub sides: Sides,
}

/// The texts of one file of a diff panel, asked for by [`Event::NeedTexts`].
pub struct WantedTexts {
    pub request: TextsRequest,
    pub origin: DiffOrigin,
}

/// Answers a diff panel's [`Event::NeedTexts`] with the texts `origin` has, for a diff whose origin never changes.
pub fn serve_texts(panel: &Entity<Panel>, origin: DiffOrigin, cx: &mut App) {
    cx.subscribe(panel, move |panel, event: &Event, cx| {
        if let Event::NeedTexts(request) = event {
            send_texts(&panel, WantedTexts { request: *request, origin: origin.clone() }, cx);
        }
    })
    .detach();
}

/// Reads the two texts of the file `wanted` names off the UI thread and casts them to `panel`; a file that cannot be
/// read stays uncoloured, with the reason on stderr.
pub fn send_texts(panel: &Entity<Panel>, wanted: WantedTexts, cx: &mut App) {
    let Some(file) = panel.read(cx).diff_file(wanted.request.file, cx) else {
        return;
    };
    let paths = FilePaths { path: file.path.clone(), old_path: file.old_path.clone() };
    let (panel, WantedTexts { request: TextsRequest { file, generation, .. }, origin }) = (panel.downgrade(), wanted);
    let reading = cx.background_executor().spawn(async move { diff::texts(&origin.dir, &origin.sides, &paths) });
    cx.spawn(async move |cx| match reading.await {
        Ok((old, new)) => drop(panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file, generation, old, new }, cx))),
        Err(error) => eprintln!("ive: a diff file shown uncoloured: {error}"),
    })
    .detach();
}

/// A panel showing `files` read-only, in the layout the user last switched a diff to; switching it again is
/// remembered for the next diff.
pub fn diff_panel(files: Vec<FileDiff>, headers: Headers, cx: &mut App) -> Entity<Panel> {
    let layout = cx.try_global::<SettingsStore>().map(|store| store.current().diff_layout).unwrap_or_default();
    let input = DiffInput { files, headers, layout, editable: Editable::No };
    let panel = cx.new(|cx| {
        let mut panel = Panel::new(cx);
        panel.handle(Cast::Open(Content::Diff(input)), cx);
        panel
    });
    cx.subscribe(&panel, |_panel, event: &Event, cx| {
        if let Event::LayoutToggled(layout) = event {
            remember_layout(*layout, cx);
        }
    })
    .detach();
    panel
}

fn remember_layout(layout: DiffLayout, cx: &mut App) {
    if cx.try_global::<SettingsStore>().is_none() {
        return;
    }
    let stored = cx.update_global::<SettingsStore, _>(|store, _| store.replace(store.current().with_diff_layout(layout)));
    if let Err(error) = stored {
        eprintln!("ive: cannot save the diff layout: {error}");
    }
}

fn parse_in_background(panel: WeakEntity<Panel>, job: ParseJob, name: &str, cx: &mut App) {
    let (target, revision, stale) = (job.target(), job.revision(), job.cancel());
    let name = name.to_owned();
    let parsing = cx.background_executor().spawn(async move { job.run() });
    cx.spawn(async move |cx| {
        let result = parsing.await;
        // the text was replaced (or the panel went away) after the parse finished, before this update ran; edits
        // made meanwhile are no reason to drop it, the panel shows it moved by them
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
    use codeview::diff::{DiffBody, DiffLine, GitPath, Hunk, LineKind, Mark};
    use codeview::host::ClipboardTarget;
    use codeview::panel::PanelState;
    use gpui::TestAppContext;

    use super::*;
    use crate::settings::Settings;

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
    fn should_map_toggling_the_diff_layout_in_a_viewer_to_a_command() {
        // given
        let toggle = Action::Builtin(Builtin::ToggleDiffLayout);

        // when
        let decision = decide(Some(toggle), KeyScope::Viewer);

        // then
        assert_eq!(decision, Decision::Done(KeyOutcome::Command(Command::ToggleLayout)));
    }

    /// A diff panel showing one added line, as a diff tab opens it.
    fn opened_diff(cx: &mut TestAppContext) -> Entity<Panel> {
        let line = DiffLine { kind: LineKind::Added, old: None, new: Some(1), text: "a".to_owned() };
        let hunk = Hunk { header: "@@ -0,0 +1 @@".to_owned(), lines: vec![line] };
        let file = FileDiff { path: GitPath::from_bytes(b"a.rs"), old_path: None, mark: Mark::Added, body: DiffBody::Text { hunks: vec![hunk], cut: None } };
        cx.update(|cx| diff_panel(vec![file], Headers::Hidden, cx))
    }

    fn layout_of(panel: &Entity<Panel>, cx: &mut TestAppContext) -> Option<DiffLayout> {
        panel.read_with(cx, |panel, cx| panel.diff_layout(cx))
    }

    #[gpui::test]
    fn should_open_new_diffs_in_the_last_toggled_layout(cx: &mut TestAppContext) {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        cx.update(|cx| {
            cx.set_global(SettingsStore::new(Settings::default(), Some(path.clone())));
            cx.set_global(crate::theme::view_style(&crate::theme::Theme::of(crate::theme::Appearance::Dark)));
        });
        let first = opened_diff(cx);
        let restored = opened_diff(cx);
        let before = layout_of(&first, cx);

        // when
        first.update(cx, |panel, cx| panel.handle(Cast::Command(Command::ToggleLayout), cx));
        cx.run_until_parked();
        restored.update(cx, |panel, cx| panel.handle(Cast::Restore(PanelState { layout: Some(DiffLayout::Inline), ..PanelState::default() }), cx));
        let opened = opened_diff(cx);

        // then
        assert_eq!((before, layout_of(&opened, cx), layout_of(&restored, cx)), (Some(DiffLayout::Inline), Some(DiffLayout::SideBySide), Some(DiffLayout::Inline)));
        assert_eq!(Settings::load(&path).unwrap().diff_layout, DiffLayout::SideBySide);
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
