//! The change popup of an edited text, driven through casts, click targets and keys as a host and a user would.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gpui::{ClickEvent, Entity, Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, VisualTestContext, div, point, px};

use super::*;
use crate::changes::{ChangeDiffJob, ChangeHunk};
use crate::text_editor::Unit;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::document::TextSource;
use crate::host::{ClickListener, Host, KeyOutcome};
use crate::line_view::Metrics;
use crate::style::{Appearance, ViewStyle};

/// What the host saw: the click targets of the last frames by name, the clipboard, the diffs asked for, and the
/// change markers' diffs asked for.
#[derive(Default)]
struct Seen {
    targets: HashMap<String, ClickListener>,
    copied: Vec<String>,
    diffs: Vec<usize>,
    jobs: Vec<ChangeDiffJob>,
    commits: Vec<Commit>,
}

/// An [`Event::CommitChange`], its text as a string.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Commit {
    head_id: String,
    block: std::ops::Range<usize>,
    text: String,
    message: String,
}

/// A window showing only a panel that edits `text` against base `head`, with the markers' diff landed and drawn.
fn editing<'a>(text: &str, head: &str, cx: &'a mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Seen>>, &'a mut VisualTestContext) {
    let seen: Rc<RefCell<Seen>> = Rc::default();
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
        cx.set_global(host(&seen));
    });
    let (panel, cx) = cx.add_window_view(|_window, cx| Panel::new(cx));
    let heard = seen.clone();
    cx.update(|_window, cx| {
        cx.subscribe(&panel, move |_, event: &Event, _| match event {
            Event::NeedChangeDiff(job) => heard.borrow_mut().jobs.push(job.clone()),
            Event::OpenDiff { line } => heard.borrow_mut().diffs.push(*line),
            Event::CommitChange { head_id, block, text, message } => heard.borrow_mut().commits.push(Commit {
                head_id: head_id.clone(),
                block: block.clone(),
                text: String::from_utf8(text.clone()).unwrap(),
                message: message.clone(),
            }),
            _ => {}
        })
        .detach();
    });
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
    let content = Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit: Some(policy) };
    cast(&panel, Cast::Open(content), cx);
    let head = Source::from_vec(head.as_bytes().to_vec());
    cast(&panel, Cast::SetChangeBase { base: ChangeBase::Tracked { head, head_id: "head".to_owned() } }, cx);
    land(&panel, &seen, cx);
    (panel, seen, cx)
}

/// Lands the change markers' diff asked for last, then draws the next frame.
fn land(panel: &Entity<Panel>, seen: &RefCell<Seen>, cx: &mut VisualTestContext) {
    let job = seen.borrow().jobs.last().cloned().expect("a diff was asked for");
    cast(panel, Cast::ChangeDiff { revision: job.revision(), hunks: job.run() }, cx);
    draw(cx);
}

fn jobs_asked(seen: &RefCell<Seen>) -> usize {
    seen.borrow().jobs.len()
}

/// Whether the popup's buttons wait for the markers of the current text.
fn is_busy(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> bool {
    editor_of(panel, cx).read_with(cx, |editor, _| editor.waits_for_changes())
}

fn host(seen: &Rc<RefCell<Seen>>) -> Host {
    let (targets, copied) = (seen.clone(), seen.clone());
    Host {
        keys: Rc::new(|_, _, _| KeyOutcome::Unbound),
        clipboard: Rc::new(move |_, text, _| copied.borrow_mut().copied.push(text)),
        paste: Rc::new(|_, _| None),
        diff_mark: Rc::new(|_, _| div().into_any_element()),
        click_target: Rc::new(move |name, listener, _| drop(targets.borrow_mut().targets.insert(name, listener))),
        tooltip: Rc::new(|_, _, cx| cx.new(|_| gpui::EmptyView).into()),
    }
}

fn cast(panel: &Entity<Panel>, cast: Cast, cx: &mut VisualTestContext) {
    panel.update(cx, |panel, cx| panel.handle(cast, cx));
}

/// Draws the window's next frame; it registers the click targets shown.
fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// Clicks target `name` as IVE's MCP `click` does, then draws the next frame.
fn click(name: &str, seen: &RefCell<Seen>, cx: &mut VisualTestContext) {
    seen.borrow_mut().targets.clear();
    draw(cx);
    let listener = seen.borrow().targets.get(name).cloned().unwrap_or_else(|| panic!("no click target {name}"));
    cx.update(|window, cx| listener(&ClickEvent::default(), window, cx));
    draw(cx);
}

fn editor_of(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Entity<TextEditor> {
    let Body::Edit(editor) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not an editor") };
    editor
}

/// The change the popup shows, if open.
fn in_popup(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Option<ChangeHunk> {
    editor_of(panel, cx).read_with(cx, |editor, _| editor.change_in_popup().cloned())
}

fn text_of(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> String {
    editor_of(panel, cx).read_with(cx, |editor, _| editor.text())
}

fn caret_line(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> usize {
    editor_of(panel, cx).read_with(cx, |editor, _| editor.caret_position().line)
}

fn hunk(base: std::ops::Range<usize>, current: std::ops::Range<usize>) -> ChangeHunk {
    ChangeHunk { base, current }
}

const HEAD: &str = "one\ntwo\nthree\nfour\nfive\nsix\nseven\n";

#[gpui::test]
fn should_open_the_popup_for_the_clicked_marker(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    let before = in_popup(&panel, cx);

    let metrics = cx.update(|window, cx| Metrics::measure(window, cx, 8));
    let on_marker = point(metrics.gutter_width() - metrics.char_width / 2., metrics.line_height * 1.5);

    // when
    cx.simulate_click(on_marker, Modifiers::default());
    draw(cx);

    // then
    assert_eq!((before, in_popup(&panel, cx)), (None, Some(hunk(1..2, 1..2))));
    assert_eq!(caret_line(&panel, cx), 0, "the click does not move the caret");
    let targets = &seen.borrow().targets;
    assert!(["prev", "next", "rollback", "diff", "copy"].iter().all(|name| targets.contains_key(&format!("change-popup:{name}"))));
}

#[gpui::test]
fn should_roll_back_a_block_as_one_undoable_edit(cx: &mut TestAppContext) {
    // given
    let edited = "one\nTWO\nTHREE\nextra\nfour\nfive\nsix\nseven\n";
    let (panel, seen, cx) = editing(edited, HEAD, cx);
    click("change-marker:2", &seen, cx);

    // when
    click("change-popup:rollback", &seen, cx);
    let rolled_back = text_of(&panel, cx);
    cast(&panel, Cast::Command(Command::Undo), cx);

    // then
    assert_eq!(rolled_back, HEAD);
    assert_eq!(in_popup(&panel, cx), None);
    assert_eq!(text_of(&panel, cx), edited, "one undo brings the change back");
    assert!(panel.read_with(cx, |panel, cx| panel.has_unsaved(cx)), "the rollback is saved as an edit is");
}

#[gpui::test]
fn should_copy_the_previous_text(cx: &mut TestAppContext) {
    // given
    let (_panel, seen, cx) = editing("one\nTWO\nTHREE\nfour\nfive\nsix\nseven\nadded\n", HEAD, cx);
    click("change-marker:2", &seen, cx);

    // when
    click("change-popup:copy", &seen, cx);
    click("change-marker:8", &seen, cx);
    click("change-popup:copy", &seen, cx);

    // then
    assert_eq!(seen.borrow().copied, ["two\nthree\n"], "added lines have no previous text");
}

#[gpui::test]
fn should_move_to_the_next_and_previous_change(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfive\nsix\nSEVEN\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    let mut visited = Vec::new();

    // when
    for name in ["next", "next", "next", "prev"] {
        click(&format!("change-popup:{name}"), &seen, cx);
        visited.push((in_popup(&panel, cx).map(|shown| shown.current), caret_line(&panel, cx)));
    }

    // then
    assert_eq!(visited, [(Some(3..3), 3), (Some(5..6), 5), (Some(1..2), 1), (Some(5..6), 5)], "round from the last to the first");
}

#[gpui::test]
fn should_close_the_popup_on_escape_and_on_an_edit_elsewhere(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);

    // when
    cx.simulate_keystrokes("escape");
    let after_escape = in_popup(&panel, cx);
    click("change-marker:2", &seen, cx);
    cast(&panel, Cast::RevealLine(2), cx);
    cast(&panel, Cast::Type("!".to_owned()), cx);
    let after_typing_inside = in_popup(&panel, cx);
    cast(&panel, Cast::RevealLine(4), cx);
    cast(&panel, Cast::Type("!".to_owned()), cx);

    // then
    assert_eq!(after_escape, None);
    assert_eq!(after_typing_inside, Some(hunk(1..2, 1..2)));
    assert_eq!(in_popup(&panel, cx), None);
}

#[gpui::test]
fn should_ask_for_the_diff_at_the_change(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\ntwo\nthree\nfour\nFIVE\nsix\nseven\n", HEAD, cx);
    click("change-marker:5", &seen, cx);

    // when
    click("change-popup:diff", &seen, cx);

    // then
    assert_eq!((seen.borrow().diffs.clone(), in_popup(&panel, cx)), (vec![5], None));
}

#[gpui::test]
fn should_roll_back_to_head_after_an_edit_joined_the_next_line(cx: &mut TestAppContext) {
    // given
    let head = "a\nb\nc\nd\n";
    let (panel, seen, cx) = editing("a\nB1\nB2\nc\nd\n", head, cx);
    cast(&panel, Cast::RevealLine(4), cx);
    cast(&panel, Cast::Command(Command::Backspace(Unit::Char)), cx);
    click("change-marker:2", &seen, cx);
    let asked = jobs_asked(&seen);

    // when
    click("change-popup:rollback", &seen, cx);
    let waiting = (text_of(&panel, cx), is_busy(&panel, cx), jobs_asked(&seen));
    land(&panel, &seen, cx);

    // then
    assert_eq!(waiting, ("a\nB1\nB2c\nd\n".to_owned(), true, asked + 1), "waits for a diff asked for at once");
    assert_eq!(text_of(&panel, cx), head);
    assert_eq!(in_popup(&panel, cx), None);
}

#[gpui::test]
fn should_wait_for_current_markers_before_copying(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    cast(&panel, Cast::RevealLine(3), cx);
    cast(&panel, Cast::Type("x".to_owned()), cx);

    // when
    click("change-popup:copy", &seen, cx);
    let waiting = (seen.borrow().copied.clone(), is_busy(&panel, cx));
    land(&panel, &seen, cx);

    // then
    assert_eq!(waiting, (Vec::<String>::new(), true));
    assert_eq!(seen.borrow().copied, ["two\nthree\n"], "the change as the new diff found it");
    assert_eq!((in_popup(&panel, cx), is_busy(&panel, cx)), (Some(hunk(1..3, 1..3)), false));
}

/// The text after rolling back the change whose marker is on line `marker` (from 1) of `text`, diffed with `head`.
fn rolled_back(text: &str, head: &str, marker: usize, cx: &mut TestAppContext) -> String {
    let (panel, seen, cx) = editing(text, head, cx);
    click(&format!("change-marker:{marker}"), &seen, cx);
    click("change-popup:rollback", &seen, cx);
    text_of(&panel, cx)
}

#[gpui::test]
fn should_roll_back_deleted_lines(cx: &mut TestAppContext) {
    // given
    let text = "one\ntwo\nfour\nfive\nsix\nseven\n";

    // when
    let restored = rolled_back(text, HEAD, 3, cx);

    // then
    assert_eq!(restored, HEAD);
}

#[gpui::test]
fn should_roll_back_lines_of_a_crlf_file(cx: &mut TestAppContext) {
    // given
    let head = "one\r\ntwo\r\nthree\r\n";

    // when
    let restored = rolled_back("one\r\nTWO\r\nTHREE\r\n", head, 2, cx);

    // then
    assert_eq!(restored, head);
}

#[gpui::test]
fn should_roll_back_a_last_line_without_a_line_break(cx: &mut TestAppContext) {
    // given
    let head = "one\ntwo";

    // when
    let restored = rolled_back("one\nTWO", head, 2, cx);

    // then
    assert_eq!(restored, head);
}

#[gpui::test]
fn should_close_the_popup_on_a_click_outside_it_and_when_the_focus_leaves(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    let metrics = cx.update(|window, cx| Metrics::measure(window, cx, 8));
    let below_popup = point(metrics.gutter_width() + px(40.), metrics.line_height * 7.5);
    cx.update(|window, _| window.activate_window());
    click("change-marker:2", &seen, cx);

    // when
    cx.simulate_click(below_popup, Modifiers::default());
    draw(cx);
    let after_click = in_popup(&panel, cx);
    click("change-marker:2", &seen, cx);
    let reopened = in_popup(&panel, cx);
    cx.update(|window, cx| window.blur(cx));
    draw(cx);

    // then
    assert_eq!((after_click, reopened), (None, Some(hunk(1..2, 1..2))));
    assert_eq!(in_popup(&panel, cx), None);
}

#[gpui::test]
fn should_close_the_popup_once_its_change_is_scrolled_out_of_view(cx: &mut TestAppContext) {
    // given
    let text: String = (0..200).map(|line| if line == 1 { "CHANGED\n".to_owned() } else { format!("line {line}\n") }).collect();
    let head: String = (0..200).map(|line| format!("line {line}\n")).collect();
    let (panel, seen, cx) = editing(&text, &head, cx);
    let metrics = cx.update(|window, cx| Metrics::measure(window, cx, 200));
    let wheel = |lines: f32| ScrollWheelEvent {
        position: point(metrics.gutter_width() + px(40.), metrics.line_height * 0.5),
        delta: ScrollDelta::Lines(point(0., lines)),
        ..Default::default()
    };
    click("change-marker:2", &seen, cx);

    // when
    cx.simulate_event(wheel(-1.));
    draw(cx);
    let one_line_down = in_popup(&panel, cx);
    cx.simulate_event(wheel(-10.));
    draw(cx);

    // then
    assert_eq!(one_line_down, Some(hunk(1..2, 1..2)), "kept while its change shows");
    assert_eq!(in_popup(&panel, cx), None);
}

/// The commit message typed in the popup, and why its last commit failed, if it did.
fn commit_state(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> (String, Option<String>) {
    editor_of(panel, cx).read_with(cx, |editor, cx| (editor.commit_message(cx), editor.commit_failure()))
}

#[gpui::test]
fn should_send_the_change_with_its_unsaved_text_to_be_committed_on_enter(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nFIVE\nsix\nseven\n", HEAD, cx);
    cast(&panel, Cast::RevealLine(2), cx);
    cast(&panel, Cast::Type("X".to_owned()), cx);
    click("change-marker:2", &seen, cx);
    let edited = text_of(&panel, cx);

    // when
    click("change-popup:message", &seen, cx);
    cx.simulate_input("fix two");
    cx.simulate_keystrokes("enter");
    let waiting = seen.borrow().commits.len();
    land(&panel, &seen, cx);
    cx.simulate_keystrokes("enter");
    draw(cx);

    // then
    let sent = Commit { head_id: "head".to_owned(), block: 1..2, text: "one\nXTWO\nthree\nfour\nfive\nsix\nseven\n".to_owned(), message: "fix two".to_owned() };
    assert_eq!(waiting, 0, "it waits for the markers of the text typed");
    assert_eq!(seen.borrow().commits, [sent], "sent once: the second Enter came while it was being committed");
    assert_eq!(text_of(&panel, cx), edited, "the field took the typing, not the text");
    assert_eq!(in_popup(&panel, cx), Some(hunk(1..2, 1..2)));
}

#[gpui::test]
fn should_show_why_a_commit_failed_and_close_once_one_is_done(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_input("fix");
    click("change-popup:commit", &seen, cx);

    // when
    cast(&panel, Cast::ChangeCommitFailed("HEAD moved".to_owned()), cx);
    draw(cx);
    let failed = (in_popup(&panel, cx), commit_state(&panel, cx));
    cx.simulate_keystrokes("enter");
    cast(&panel, Cast::ChangeCommitted, cx);
    draw(cx);
    cx.simulate_input("!");

    // then
    assert_eq!(failed, (Some(hunk(1..2, 1..2)), ("fix".to_owned(), Some("HEAD moved".to_owned()))), "the message stays");
    assert_eq!(seen.borrow().commits.len(), 2);
    assert_eq!((in_popup(&panel, cx), commit_state(&panel, cx)), (None, (String::new(), None)));
    assert_eq!(text_of(&panel, cx), "!one\nTWO\nthree\nfour\nfive\nsix\nseven\n", "the text has the keyboard again");
}

#[gpui::test]
fn should_send_a_change_once_while_its_commit_is_in_flight_after_reopening(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_input("fix two");
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("escape");
    draw(cx);
    let closed = in_popup(&panel, cx);

    // when
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_keystrokes("enter");
    click("change-popup:commit", &seen, cx);
    cx.simulate_input("!");

    // then
    assert_eq!(closed, None);
    assert_eq!(seen.borrow().commits.len(), 1, "sent once");
    assert_eq!(commit_state(&panel, cx), ("fix two".to_owned(), None), "the field takes no typing meanwhile");
    assert_eq!(text_of(&panel, cx), "one\nTWO\nthree\nfour\nfive\nsix\nseven\n");
}

#[gpui::test]
fn should_show_a_failure_on_the_change_sent_when_its_popup_opens_again(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nFIVE\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_input("fix two");
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("escape");
    draw(cx);

    // when
    cast(&panel, Cast::ChangeCommitFailed("HEAD moved".to_owned()), cx);
    click("change-marker:5", &seen, cx);
    let other = commit_state(&panel, cx);
    cast(&panel, Cast::ChangeCommitted, cx);
    let after_stale_answer = in_popup(&panel, cx);
    click("change-marker:2", &seen, cx);

    // then
    assert_eq!(other, ("fix two".to_owned(), None), "another change's popup shows no failure");
    assert_eq!(after_stale_answer, Some(hunk(4..5, 4..5)), "an answer to no commit in flight does nothing");
    assert_eq!(commit_state(&panel, cx), ("fix two".to_owned(), Some("HEAD moved".to_owned())));
}

#[gpui::test]
fn should_keep_another_changes_popup_open_when_the_one_sent_is_committed(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nFIVE\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_input("fix two");
    cx.simulate_keystrokes("enter");
    click("change-popup:next", &seen, cx);

    // when
    cast(&panel, Cast::ChangeCommitted, cx);
    draw(cx);

    // then
    assert_eq!(in_popup(&panel, cx), Some(hunk(4..5, 4..5)));
    assert_eq!(commit_state(&panel, cx), (String::new(), None), "the message went with the commit");
}

#[gpui::test]
fn should_close_the_popup_on_escape_in_the_field_and_give_the_text_the_keyboard(cx: &mut TestAppContext) {
    // given
    let (panel, seen, cx) = editing("one\nTWO\nthree\nfour\nfive\nsix\nseven\n", HEAD, cx);
    click("change-marker:2", &seen, cx);
    click("change-popup:message", &seen, cx);
    cx.simulate_input("typed in the field");

    // when
    cx.simulate_keystrokes("escape");
    draw(cx);
    let after_escape = (in_popup(&panel, cx), text_of(&panel, cx));
    cx.simulate_input("!");

    // then
    assert_eq!(after_escape, (None, "one\nTWO\nthree\nfour\nfive\nsix\nseven\n".to_owned()), "the file is untouched");
    assert_eq!(seen.borrow().commits.len(), 0);
    assert_eq!(text_of(&panel, cx), "!one\nTWO\nthree\nfour\nfive\nsix\nseven\n", "the text has the keyboard again");
}
