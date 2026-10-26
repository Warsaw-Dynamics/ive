//! The find box of a panel: opening, typing, stepping and closing it over each kind of body.

use std::sync::Arc;
use std::time::Duration;

use gpui::{AppContext, Entity, TestAppContext};

use super::*;
use crate::diff::{DiffBody, DiffLine, GitPath, Headers, Hunk, LineKind, Mark, side_by_side::Side};
use crate::document::TextSource;
use crate::find::{FindOption, FindText};
use crate::position::CaretPosition;

/// A panel editing `text`.
fn editing(text: &str, cx: &mut TestAppContext) -> Entity<Panel> {
    let panel = cx.new(Panel::new);
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
    let content = Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit: Some(policy) };
    cast(&panel, Cast::Open(content), cx);
    panel
}

fn cast(panel: &Entity<Panel>, cast: Cast, cx: &mut TestAppContext) {
    panel.update(cx, |panel, cx| panel.handle(cast, cx));
    cx.run_until_parked();
}

fn command(panel: &Entity<Panel>, command: Command, cx: &mut TestAppContext) {
    cast(panel, Cast::Command(command), cx);
}

/// A panel showing `text`, read-only.
fn viewing(text: &str, cx: &mut TestAppContext) -> Entity<Panel> {
    let panel = cx.new(Panel::new);
    cast(&panel, Cast::Open(Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit: None }), cx);
    panel
}

/// Types `text` over the find box's query, as the keyboard would with all of it selected.
fn type_query(panel: &Entity<Panel>, text: &str, cx: &mut TestAppContext) {
    panel.update(cx, |panel, cx| {
        panel.select_find_query(cx);
        panel.type_find_query(text, cx);
    });
    cx.run_until_parked();
}

fn toggle(panel: &Entity<Panel>, option: FindOption, cx: &mut TestAppContext) {
    panel.update(cx, |panel, cx| panel.toggle_find_option(option, cx));
    cx.run_until_parked();
}

fn view(panel: &Entity<Panel>, cx: &mut TestAppContext) -> FindView {
    panel.read_with(cx, |panel, cx| panel.find_view(cx))
}

fn caret_at(line: usize, column: usize) -> Cast {
    Cast::Restore(PanelState { caret: Some(CaretPosition { line, column }), ..PanelState::default() })
}

#[gpui::test]
fn should_open_with_the_word_at_the_caret_and_select_the_first_match_from_it(cx: &mut TestAppContext) {
    // given
    let panel = editing("one two\ntwo one\n", cx);
    cast(&panel, caret_at(0, 5), cx);

    // when
    command(&panel, Command::Find, cx);

    // then
    let view = view(&panel, cx);
    assert_eq!((view.open, view.query.as_str(), view.count.as_str()), (true, "two", "1 of 2"));
    assert_eq!(view.selected.as_deref(), Some("two"));
    assert_eq!(panel.read_with(cx, |panel, cx| panel.state_now(cx).caret), Some(CaretPosition { line: 0, column: 7 }));
}

#[gpui::test]
fn should_step_round_both_ways(cx: &mut TestAppContext) {
    // given
    let panel = editing("o o o", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "o", cx);

    // when
    let counts: Vec<String> = [Command::FindNext, Command::FindNext, Command::FindNext, Command::FindPrevious]
        .into_iter()
        .map(|step| {
            command(&panel, step, cx);
            view(&panel, cx).count
        })
        .collect();

    // then
    assert_eq!(counts, ["2 of 3", "3 of 3", "1 of 3", "3 of 3"]);
}

#[gpui::test]
fn should_say_no_results_and_bad_regex(cx: &mut TestAppContext) {
    // given
    let panel = editing("text", cx);
    command(&panel, Command::Find, cx);

    // when
    type_query(&panel, "zzz", cx);
    let missing = view(&panel, cx).count;
    toggle(&panel, FindOption::Regex, cx);
    type_query(&panel, "(", cx);

    // then
    assert_eq!((missing.as_str(), view(&panel, cx).count.as_str()), ("No results", "Bad regex"));
}

#[gpui::test]
fn should_keep_the_caret_where_the_user_types_when_an_edit_searches_again(cx: &mut TestAppContext) {
    // given
    let panel = editing("a b\n", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "a", cx);
    cast(&panel, caret_at(0, 3), cx);

    // when
    cast(&panel, Cast::Type(" a".to_owned()), cx);

    // then
    assert_eq!(view(&panel, cx).count, "1 of 2");
    assert_eq!(panel.read_with(cx, |panel, cx| panel.state_now(cx).caret), Some(CaretPosition { line: 0, column: 5 }));
}

#[gpui::test]
fn should_leave_the_match_selected_after_closing(cx: &mut TestAppContext) {
    // given
    let panel = editing("one two two", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "two", cx);
    command(&panel, Command::FindNext, cx);

    // when
    panel.update(cx, |panel, cx| panel.close_find(cx));

    // then
    let view = view(&panel, cx);
    assert_eq!((view.open, view.selected.as_deref(), view.marked), (false, Some("two"), false));
    assert_eq!(panel.read_with(cx, |panel, cx| panel.state_now(cx).caret), Some(CaretPosition { line: 0, column: 11 }));
}

#[gpui::test]
fn should_reopen_with_the_last_query_and_step_on_find_next(cx: &mut TestAppContext) {
    // given
    let panel = editing("two one two", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "two", cx);
    panel.update(cx, |panel, cx| panel.close_find(cx));

    // when
    command(&panel, Command::FindNext, cx);

    // then
    let view = view(&panel, cx);
    assert_eq!((view.open, view.query.as_str(), view.count.as_str()), (true, "two", "2 of 2"));
}

#[gpui::test]
fn should_show_only_the_result_of_the_last_query(cx: &mut TestAppContext) {
    // given
    let panel = editing("on o on", cx);
    command(&panel, Command::Find, cx);

    // when
    panel.update(cx, |panel, cx| {
        panel.type_find_query("o", cx);
        panel.type_find_query("n", cx);
    });
    cx.run_until_parked();

    // then
    assert_eq!(view(&panel, cx).count, "1 of 2");
}

#[gpui::test]
fn should_remember_the_query_and_options_for_the_next_panel(cx: &mut TestAppContext) {
    // given
    let first = editing("Two", cx);
    command(&first, Command::Find, cx);
    toggle(&first, FindOption::MatchCase, cx);
    type_query(&first, "Two", cx);
    // the caret on a space: no word to take
    let second = editing(" two Two", cx);

    // when
    command(&second, Command::Find, cx);

    // then
    let view = view(&second, cx);
    assert_eq!((view.query.as_str(), view.count.as_str(), view.options.match_case), ("Two", "1 of 1", true));
}

#[gpui::test]
fn should_type_into_the_query_while_it_has_the_keyboard(cx: &mut TestAppContext) {
    // given
    let panel = editing("abc", cx);
    command(&panel, Command::Find, cx);

    // when
    let takes = panel.update(cx, |panel, cx| {
        panel.find_takes_typing_for_tests();
        panel.takes_typing(cx)
    });
    cast(&panel, Cast::Type("bc".to_owned()), cx);

    // then
    assert!(takes);
    assert_eq!((view(&panel, cx).query.as_str(), view(&panel, cx).count.as_str()), ("bc", "1 of 1"));
}

#[gpui::test]
fn should_select_a_match_across_a_line_break(cx: &mut TestAppContext) {
    // given
    let panel = editing("ab\ncd", cx);
    command(&panel, Command::Find, cx);
    toggle(&panel, FindOption::Regex, cx);

    // when
    type_query(&panel, r"b\nc", cx);

    // then
    assert_eq!((view(&panel, cx).count.as_str(), view(&panel, cx).selected.as_deref()), ("1 of 1", Some("b\nc")));
}

#[gpui::test]
fn should_find_and_select_in_a_viewed_file(cx: &mut TestAppContext) {
    // given
    let panel = viewing("one\ttwo\ntwo\n", cx);
    command(&panel, Command::Find, cx);

    // when
    type_query(&panel, "two", cx);
    command(&panel, Command::FindNext, cx);

    // then
    let view = view(&panel, cx);
    assert_eq!((view.count.as_str(), view.selected.as_deref(), view.marked), ("2 of 2", Some("two"), true));
}

fn file(name: &str, lines: &[(LineKind, &str)]) -> FileDiff {
    let numbered = lines.iter().enumerate().map(|(at, (kind, text))| {
        let number = Some(at as u32 + 1);
        let (old, new) = match kind {
            LineKind::Removed => (number, None),
            LineKind::Added => (None, number),
            LineKind::Context | LineKind::NoNewline => (number, number),
        };
        DiffLine { kind: *kind, old, new, text: (*text).to_owned() }
    });
    let hunk = Hunk { header: "@@ -1 +1 @@".to_owned(), lines: numbered.collect() };
    FileDiff { path: GitPath::from_bytes(name.as_bytes()), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks: vec![hunk], cut: None } }
}

fn diffing(files: Vec<FileDiff>, shape: (Headers, DiffLayout), cx: &mut TestAppContext) -> Entity<Panel> {
    cx.update(crate::style::ViewStyle::set_for_test);
    let panel = cx.new(Panel::new);
    let input = DiffInput { files, headers: shape.0, layout: shape.1, editable: Editable::No };
    cast(&panel, Cast::Open(Content::Diff(input)), cx);
    panel
}

#[gpui::test]
fn should_search_every_line_of_an_inline_diff(cx: &mut TestAppContext) {
    // given
    let panel = diffing(vec![file("a.txt", &[(LineKind::Removed, "old x"), (LineKind::Added, "new x")])], (Headers::Hidden, DiffLayout::Inline), cx);
    command(&panel, Command::Find, cx);

    // when
    type_query(&panel, "x", cx);
    command(&panel, Command::FindNext, cx);

    // then
    let view = view(&panel, cx);
    assert_eq!((view.count.as_str(), view.selected.as_deref(), view.marked), ("2 of 2", Some("x"), true));
}

#[gpui::test]
fn should_search_only_the_side_clicked_side_by_side(cx: &mut TestAppContext) {
    // given
    let lines = [(LineKind::Removed, "x x"), (LineKind::Added, "x")];
    let panel = diffing(vec![file("a.txt", &lines)], (Headers::Hidden, DiffLayout::SideBySide), cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "x", cx);
    let new_side = view(&panel, cx).count;

    // when
    panel.update(cx, |panel, cx| panel.search_diff_side_for_tests(Side::Old, cx));
    cx.run_until_parked();

    // then
    assert_eq!((new_side.as_str(), view(&panel, cx).count.as_str()), ("1 of 1", "1 of 2"));
}

#[gpui::test]
fn should_count_a_collapsed_file_and_expand_it_on_reaching_it(cx: &mut TestAppContext) {
    // given
    let files = vec![file("a.txt", &[(LineKind::Added, "x")]), file("b.txt", &[(LineKind::Added, "x")])];
    let panel = diffing(files, (Headers::Shown, DiffLayout::Inline), cx);
    cast(&panel, Cast::Restore(PanelState { collapsed: vec!["a.txt".to_owned()], ..PanelState::default() }), cx);
    command(&panel, Command::Find, cx);

    // when
    type_query(&panel, "x", cx);

    // then
    assert_eq!(view(&panel, cx).count, "1 of 2");
    assert_eq!(panel.read_with(cx, |panel, cx| panel.state_now(cx).collapsed), Vec::<String>::new());
}

#[gpui::test]
fn should_not_select_again_when_the_files_are_replaced(cx: &mut TestAppContext) {
    // given
    let panel = diffing(vec![file("a.txt", &[(LineKind::Added, "x"), (LineKind::Added, "y")])], (Headers::Hidden, DiffLayout::Inline), cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "y", cx);

    // when
    cast(&panel, Cast::SetFiles(vec![file("a.txt", &[(LineKind::Added, "x"), (LineKind::Added, "y"), (LineKind::Added, "y")])]), cx);

    // then: new rows drop the rows' selection, as before, and the search does not select again
    let view = view(&panel, cx);
    assert_eq!((view.count.as_str(), view.selected.as_deref()), ("1 of 2", None));
}

#[gpui::test]
fn should_keep_a_regex_query_when_find_is_pressed_again(cx: &mut TestAppContext) {
    // given
    let panel = editing("o1 o2", cx);
    command(&panel, Command::Find, cx);
    toggle(&panel, FindOption::Regex, cx);
    type_query(&panel, r"o\d", cx);

    // when
    command(&panel, Command::Find, cx);

    // then
    assert_eq!((view(&panel, cx).query.as_str(), view(&panel, cx).count.as_str()), (r"o\d", "1 of 2"));
}

#[gpui::test]
fn should_step_once_the_search_lands_when_asked_before(cx: &mut TestAppContext) {
    // given
    let panel = editing("a a a", cx);
    command(&panel, Command::Find, cx);

    // when
    // the search runs on the background executor, which a test runs only once parked
    panel.update(cx, |panel, cx| panel.type_find_query("a", cx));
    panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::FindNext), cx));
    cx.run_until_parked();

    // then
    assert_eq!(view(&panel, cx).count, "2 of 3");
}

#[gpui::test]
fn should_start_from_the_top_on_the_other_side(cx: &mut TestAppContext) {
    // given
    let lines = [(LineKind::Removed, "x x x"), (LineKind::Added, "x x x")];
    let panel = diffing(vec![file("a.txt", &lines)], (Headers::Hidden, DiffLayout::SideBySide), cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "x", cx);
    command(&panel, Command::FindPrevious, cx);
    let on_new_side = view(&panel, cx).count;

    // when
    panel.update(cx, |panel, cx| panel.search_diff_side_for_tests(Side::Old, cx));
    cx.run_until_parked();

    // then
    assert_eq!((on_new_side.as_str(), view(&panel, cx).count.as_str()), ("3 of 3", "1 of 3"));
}

#[gpui::test]
fn should_search_one_copy_of_an_edited_text_until_it_changes(cx: &mut TestAppContext) {
    // given
    let panel = editing("abc", cx);
    let bytes = |panel: &Entity<Panel>, cx: &mut TestAppContext| match panel.read_with(cx, |panel, cx| panel.body_find_text(cx).1) {
        FindText::Owned(bytes) => bytes,
        FindText::Shared(_) | FindText::Parts { .. } => panic!("an edited text is copied whole"),
    };
    let first = bytes(&panel, cx);

    // when
    let again = bytes(&panel, cx);
    cast(&panel, Cast::Type("d".to_owned()), cx);
    let edited = bytes(&panel, cx);

    // then
    assert!(std::sync::Arc::ptr_eq(&first, &again));
    assert_eq!(&edited[..], b"dabc");
}

#[gpui::test]
fn should_not_match_across_two_files_of_a_diff(cx: &mut TestAppContext) {
    // given
    let files = vec![file("a.txt", &[(LineKind::Added, "end")]), file("b.txt", &[(LineKind::Added, "start")])];
    let panel = diffing(files, (Headers::Shown, DiffLayout::Inline), cx);
    command(&panel, Command::Find, cx);
    toggle(&panel, FindOption::Regex, cx);

    // when
    type_query(&panel, r"end\nstart", cx);

    // then
    assert_eq!(view(&panel, cx).count, "No results");
}

#[gpui::test]
fn should_have_no_find_in_a_panel_that_takes_no_keyboard(cx: &mut TestAppContext) {
    // given
    let panel = cx.new(|cx| Panel::new(cx).unselectable());
    cast(&panel, Cast::Open(Content::Lines { source: Arc::new(TextSource::new("text")), language: None, edit: None }), cx);

    // when
    command(&panel, Command::Find, cx);
    command(&panel, Command::FindNext, cx);

    // then
    assert!(!view(&panel, cx).open);
}

#[gpui::test]
fn should_not_take_a_selection_of_invalid_utf8_as_the_query(cx: &mut TestAppContext) {
    // given
    let panel = cx.new(Panel::new);
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
    let source = Arc::new(TextSource::from_bytes(b"ab\xe9cd ab"));
    cast(&panel, Cast::Open(Content::Lines { source, language: None, edit: Some(policy) }), cx);
    cast(&panel, Cast::Command(Command::SelectAll), cx);

    // when
    command(&panel, Command::Find, cx);

    // then
    assert_eq!(view(&panel, cx).query, "");
}

#[gpui::test]
fn should_keep_the_last_matches_shown_until_a_search_of_the_same_text_lands(cx: &mut TestAppContext) {
    // given
    let panel = editing("ab ab ab", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "a", cx);

    // when: the next search has not run yet
    panel.update(cx, |panel, cx| panel.type_find_query("b", cx));
    let while_searching = view(&panel, cx);
    panel.update(cx, |panel, cx| panel.handle(Cast::Type("x".to_owned()), cx));
    let after_an_edit = view(&panel, cx);
    cx.run_until_parked();

    // then: an edit moves the text under them, so they go until the search lands
    assert_eq!((while_searching.count.as_str(), while_searching.marked), ("1 of 3", true));
    assert_eq!((after_an_edit.count.as_str(), after_an_edit.marked), ("", false));
    assert_eq!(view(&panel, cx).count, "1 of 2", "the x typed over the selected match leaves two");
}

#[gpui::test]
fn should_tick_a_collapsed_file_s_matches_at_its_header(cx: &mut TestAppContext) {
    // given
    let files = vec![file("a.txt", &[(LineKind::Added, "x")]), file("b.txt", &[(LineKind::Added, "x")])];
    let panel = diffing(files, (Headers::Shown, DiffLayout::Inline), cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "x", cx);

    // when: rows File(a), File(b), Hunk, Line
    cast(&panel, Cast::Restore(PanelState { collapsed: vec!["a.txt".to_owned()], ..PanelState::default() }), cx);
    let ticks = panel.read_with(cx, |panel, cx| panel.diff_ticks_for_tests(cx));

    // then
    let at: Vec<f32> = ticks.iter().map(|tick| tick.at).collect();
    assert_eq!(at, [0., 0.75]);
}

#[gpui::test]
fn should_search_the_reloaded_text(cx: &mut TestAppContext) {
    // given
    let panel = editing("old one\n", cx);
    command(&panel, Command::Find, cx);
    type_query(&panel, "new", cx);
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };

    // when
    cast(&panel, Cast::Open(Content::Lines { source: Arc::new(TextSource::new("new two\n")), language: None, edit: Some(policy) }), cx);

    // then
    assert_eq!(view(&panel, cx).count, "1 of 1");
}
