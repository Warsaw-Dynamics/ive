//! Editing the new side of a diff, driven through casts, frames and the mouse as a host and a user would.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{Entity, Modifiers, MouseButton, MouseDownEvent, ParentElement, Point, Pixels, Styled, TestAppContext, VisualTestContext, div, point, px, size};

use super::*;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::diff::{DiffBody, DiffLine, GitPath, Headers, Hunk, LineKind, Mark};
use crate::style::{Appearance, ViewStyle};
use crate::document::TextSource;
use crate::text_editor::model::Position;

/// What an editable diff panel told its host.
#[derive(Default)]
struct Heard {
    texts: Vec<TextsRequest>,
    saves: Vec<SaveRequest>,
    edits: usize,
}

/// A window showing a panel; its rows are 20 px high and it is 800 px wide.
fn window(cx: &mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Heard>>, &mut VisualTestContext) {
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
    });
    let cx = cx.add_empty_window();
    let panel = cx.new(Panel::new);
    let heard: Rc<RefCell<Heard>> = Rc::default();
    let seen = heard.clone();
    cx.update(|_window, cx| {
        cx.subscribe(&panel, move |_, event: &Event, _| match event {
            Event::NeedTexts(request) => seen.borrow_mut().texts.push(*request),
            Event::SaveRequested(request) => seen.borrow_mut().saves.push(request.clone()),
            Event::Edited { .. } => seen.borrow_mut().edits += 1,
            _ => {}
        })
        .detach();
    });
    (panel, heard, cx)
}

fn line(kind: LineKind, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
    DiffLine { kind, old, new, text: text.to_owned() }
}

fn file(hunks: Vec<Hunk>) -> FileDiff {
    FileDiff { path: GitPath::from_bytes(b"notes.txt"), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks, cut: None } }
}

fn hunk(lines: Vec<DiffLine>) -> Hunk {
    Hunk { header: "@@ @@".to_owned(), lines }
}

const POLICY: EditPolicy = EditPolicy { save_delay: Duration::from_millis(500), limit: None };

fn cast(panel: &Entity<Panel>, cast: Cast, cx: &mut VisualTestContext) {
    panel.update(cx, |panel, cx| panel.handle(cast, cx));
}

/// Draws a frame 20 rows high.
fn draw(panel: &Entity<Panel>, cx: &mut VisualTestContext) {
    let panel = panel.clone();
    cx.draw(point(px(0.), px(0.)), size(px(800.), px(400.)), |_, _| div().size_full().flex().flex_col().child(panel));
}

/// Opens `files` as `editable` lays them out, draws them, and answers the ask for the texts with `old` and `new`.
fn opened(panel: &Entity<Panel>, input: DiffInput, texts: (&str, &str), cx: &mut VisualTestContext, heard: &RefCell<Heard>) {
    cast(panel, Cast::Open(Content::Diff(input)), cx);
    draw(panel, cx);
    answer_texts(panel, texts, heard, cx);
}

/// Answers the last ask for file 0's texts with `old` and `new`; one to edit with the diff the rows show now as the
/// diff of the two.
fn answer_texts(panel: &Entity<Panel>, (old, new): (&str, &str), heard: &RefCell<Heard>, cx: &mut VisualTestContext) {
    let diff = diff_of(panel, cx).read_with(cx, |diff, _| diff.file(0).cloned()).expect("one file");
    answer_to_edit(panel, (old, Arc::new(TextSource::new(new))), diff, heard, cx);
}

/// Answers the last ask for file 0's texts with `old` and `new`; one to edit with `diff` as their diff.
fn answer_to_edit(panel: &Entity<Panel>, (old, new): (&str, Arc<TextSource>), diff: FileDiff, heard: &RefCell<Heard>, cx: &mut VisualTestContext) {
    let request = *heard.borrow().texts.last().expect("the texts were asked for");
    let old = Some(Source::from_vec(old.as_bytes().to_vec()));
    let generation = request.generation;
    match request.purpose {
        TextPurpose::Colours => cast(panel, Cast::SetTexts { file: 0, generation, old, new: new.source() }, cx),
        TextPurpose::Editing => cast(panel, Cast::SetEditBase { generation, base: EditBase::Text { old, new, diff } }, cx),
    }
}

/// The text the edits apply to: as last taken to edit or saved.
fn edited_base(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Arc<dyn LineSource> {
    let diff = diff_of(panel, cx);
    diff.read_with(cx, |diff, _| diff.edited().expect("the new side is edited").table().base().clone())
}

fn editable(files: Vec<FileDiff>, layout: DiffLayout) -> DiffInput {
    DiffInput { files, headers: Headers::Hidden, layout, editable: Editable::NewSide { policy: POLICY } }
}

fn diff_of(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Entity<DiffView> {
    let Body::Diff(diff) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not a diff") };
    diff
}

/// The edited text's caret.
fn caret(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Position {
    let diff = diff_of(panel, cx);
    diff.read_with(cx, |diff, _| diff.edited().expect("the new side is edited").caret())
}

/// The lines the rows show, as (sign, old, new, text).
fn rows(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Vec<(char, Option<u32>, Option<u32>, String)> {
    let diff = diff_of(panel, cx);
    diff.read_with(cx, |diff, _| {
        let DiffBody::Text { hunks, .. } = &diff.file(0).expect("one file").body else { panic!("no text") };
        let sign = |kind| match kind {
            LineKind::Added => '+',
            LineKind::Removed => '-',
            LineKind::Context => ' ',
            LineKind::NoNewline => '\\',
        };
        hunks.iter().flat_map(|hunk| &hunk.lines).map(|line| (sign(line.kind), line.old, line.new, line.text.clone())).collect()
    })
}

fn press(panel: &Entity<Panel>, at: Point<Pixels>, cx: &mut VisualTestContext) {
    let diff = diff_of(panel, cx);
    let press = MouseDownEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 1, first_mouse: false };
    cx.update(|window, cx| diff.update(cx, |diff, cx| diff.on_mouse_down(&press, window, cx)));
}

fn typed(text: &str) -> Cast {
    Cast::Type(text.to_owned())
}

fn edit(bytes: std::ops::Range<u64>, text: &str) -> Edit {
    Edit { bytes, text: text.as_bytes().to_vec() }
}

/// "one", "two" changed to "TWO", "three".
fn changed_second_line() -> FileDiff {
    file(vec![hunk(vec![
        line(LineKind::Context, Some(1), Some(1), "one"),
        line(LineKind::Removed, Some(2), None, "two"),
        line(LineKind::Added, None, Some(2), "TWO"),
        line(LineKind::Context, Some(3), Some(3), "three"),
    ])])
}

#[gpui::test]
fn should_edit_only_the_new_side_of_an_uncommitted_diff(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    opened(&panel, editable(vec![changed_second_line()], DiffLayout::SideBySide), ("one\ntwo\nthree\n", "one\nTWO\nthree\n"), cx, &heard);
    let asked = heard.borrow().texts[0].purpose;
    // row 0 is the hunk header, row 3 shows "three" on both sides; the old panel is left of x 400
    let three = px(70.);

    // when
    press(&panel, point(px(1.), three), cx);
    cast(&panel, typed("A"), cx);
    press(&panel, point(px(401.), three), cx);
    cast(&panel, typed("B"), cx);
    cast(&panel, Cast::Command(Command::Flush), cx);

    // then
    assert_eq!(asked, TextPurpose::Editing);
    let edits = heard.borrow().saves.last().map(|save| save.edits.clone());
    // the press on the old side left the caret on the first added line
    assert_eq!(edits, Some(vec![edit(4..4, "A"), edit(8..8, "B")]));
    let old_side: Vec<String> = rows(&panel, cx).into_iter().filter(|row| row.0 != '+').map(|row| row.3).collect();
    assert_eq!(old_side, ["one", "two", "three"]);
    assert_eq!(heard.borrow().edits, 2);
}

#[gpui::test]
fn should_keep_the_caret_by_new_line_after_a_rediff(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    opened(&panel, editable(vec![changed_second_line()], DiffLayout::Inline), ("one\ntwo\nthree\n", "one\nTWO\nthree\n"), cx, &heard);
    // rows: the hunk header, "one", "two", "TWO", "three"; the text starts right of the gutter
    press(&panel, point(px(799.), px(90.)), cx);
    cast(&panel, typed("!"), cx);
    cast(&panel, Cast::Command(Command::Flush), cx);
    let saved_text: Arc<dyn LineSource> = Arc::new(TextSource::new("one\nTWO\nthree!\n"));
    cast(&panel, Cast::Saved { revision: 1, source: saved_text.clone() }, cx);
    let saved = rows(&panel, cx);

    // when
    let rediffed = file(vec![hunk(vec![
        line(LineKind::Context, Some(1), Some(1), "one"),
        line(LineKind::Removed, Some(2), None, "two"),
        line(LineKind::Removed, Some(3), None, "three"),
        line(LineKind::Added, None, Some(2), "TWO"),
        line(LineKind::Added, None, Some(3), "three!"),
    ])]);
    cast(&panel, Cast::SetBaseDiff { base: saved_text, diff: rediffed }, cx);
    let kept = caret(&panel, cx);
    cast(&panel, Cast::Command(Command::Undo), cx);

    // then
    assert_eq!(kept, Position::new(2, 6));
    assert_eq!(saved.last().map(|row| row.3.clone()), Some("three!".to_owned()));
    assert_eq!(rows(&panel, cx).last().map(|row| (row.0, row.3.clone())), Some(('+', "three".to_owned())));
    assert_eq!(caret(&panel, cx), Position::new(2, 5));
}

/// A diff of a 60-line text whose lines are their numbers, with lines `changed` (from 1) changed.
fn hunks_at(changed: &[u32]) -> FileDiff {
    let hunk_at = |changed: u32| {
        let context = |n: u32| line(LineKind::Context, Some(n), Some(n), &n.to_string());
        let before = (changed.saturating_sub(3).max(1)..changed).map(context);
        let after = (changed + 1..changed + 4).map(context);
        let change = [line(LineKind::Removed, Some(changed), None, "old"), line(LineKind::Added, None, Some(changed), &changed.to_string())];
        hunk(before.chain(change).chain(after).collect())
    };
    file(changed.iter().copied().map(hunk_at).collect())
}

fn numbers() -> String {
    (1..=60).map(|n| format!("{n}\n")).collect()
}

#[gpui::test]
fn should_move_the_caret_when_its_hunk_disappears(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    opened(&panel, editable(vec![hunks_at(&[2, 30])], DiffLayout::Inline), ("", &numbers()), cx, &heard);
    let started = caret(&panel, cx);
    let base = edited_base(&panel, cx);
    let rediffed = |changed: &[u32]| Cast::SetBaseDiff { base: base.clone(), diff: hunks_at(changed) };

    // when
    cast(&panel, rediffed(&[30]), cx);
    let moved = caret(&panel, cx);
    cast(&panel, rediffed(&[10, 30, 50]), cx);
    cast(&panel, rediffed(&[30, 50]), cx);

    // then
    // the caret started on the first added line, line 2; the nearest line left is the far hunk's first, 27
    assert_eq!((started, moved), (Position::new(1, 0), Position::new(26, 0)));
    // hunks appearing and going elsewhere leave it where it is
    assert_eq!(caret(&panel, cx), Position::new(26, 0));
}

#[gpui::test]
fn should_not_edit_a_read_only_diff(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    let input = DiffInput { files: vec![changed_second_line()], headers: Headers::Hidden, layout: DiffLayout::Inline, editable: Editable::No };
    cast(&panel, Cast::Open(Content::Diff(input)), cx);
    draw(&panel, cx);
    answer_texts(&panel, ("one\ntwo\nthree\n", "one\nTWO\nthree\n"), &heard, cx);

    // when
    press(&panel, point(px(799.), px(90.)), cx);
    cast(&panel, typed("x"), cx);
    cast(&panel, Cast::Command(Command::Flush), cx);

    // then
    assert_eq!((heard.borrow().texts[0].purpose, heard.borrow().edits, heard.borrow().saves.len()), (TextPurpose::Colours, 0, 0));
    assert_eq!(rows(&panel, cx), rows_of(&changed_second_line()));
    assert!(panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx)));
}

fn rows_of(diff: &FileDiff) -> Vec<(char, Option<u32>, Option<u32>, String)> {
    let DiffBody::Text { hunks, .. } = &diff.body else { panic!("no text") };
    let sign = |kind| match kind {
        LineKind::Added => '+',
        LineKind::Removed => '-',
        LineKind::Context => ' ',
        LineKind::NoNewline => '\\',
    };
    hunks.iter().flat_map(|hunk| &hunk.lines).map(|line| (sign(line.kind), line.old, line.new, line.text.clone())).collect()
}

#[gpui::test]
fn should_show_the_diff_of_the_text_taken_to_edit_not_the_one_opened(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(Content::Diff(editable(vec![changed_second_line()], DiffLayout::Inline))), cx);
    draw(&panel, cx);
    // the file changed after the diff opened: a line came at the top
    let now = file(vec![hunk(vec![
        line(LineKind::Added, None, Some(1), "zero"),
        line(LineKind::Context, Some(1), Some(2), "one"),
        line(LineKind::Removed, Some(2), None, "two"),
        line(LineKind::Added, None, Some(3), "TWO"),
        line(LineKind::Context, Some(3), Some(4), "three"),
    ])]);

    // when
    answer_to_edit(&panel, ("one\ntwo\nthree\n", Arc::new(TextSource::new("zero\none\nTWO\nthree\n"))), now, &heard, cx);
    // rows: the hunk header, "zero", "one", "two", "TWO", "three"; the text starts right of the gutter
    press(&panel, point(px(799.), px(90.)), cx);
    cast(&panel, typed("!"), cx);
    cast(&panel, Cast::Command(Command::Flush), cx);

    // then
    let edits = heard.borrow().saves.last().map(|save| save.edits.clone());
    assert_eq!(edits, Some(vec![edit(12..12, "!")]));
    let expected = [('-', Some(2), None, "two".to_owned()), ('+', None, Some(3), "TWO!".to_owned()), (' ', Some(3), Some(4), "three".to_owned())];
    assert_eq!(rows(&panel, cx)[2..], expected);
}

#[gpui::test]
fn should_keep_the_rows_of_the_edited_text_when_a_diff_of_another_text_comes(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    opened(&panel, editable(vec![changed_second_line()], DiffLayout::Inline), ("one\ntwo\nthree\n", "one\nTWO\nthree\n"), cx, &heard);
    cast(&panel, typed("x"), cx);
    // a save in flight: a diff read meanwhile may see the file half written
    cast(&panel, Cast::Command(Command::Flush), cx);
    let typed_rows = rows(&panel, cx);
    let written = file(vec![hunk(vec![line(LineKind::Added, None, Some(1), "garbled")])]);

    // when
    cast(&panel, Cast::SetFiles(vec![written.clone()]), cx);
    cast(&panel, Cast::SetBaseDiff { base: Arc::new(TextSource::new("garbled\n")), diff: written }, cx);

    // then
    assert_eq!(heard.borrow().saves.len(), 1);
    assert_eq!(rows(&panel, cx), typed_rows);
    assert_eq!(typed_rows[2], ('+', None, Some(2), "xTWO".to_owned()));
}

#[gpui::test]
fn should_leave_the_diff_read_only_when_its_new_side_cannot_be_read_to_edit(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(Content::Diff(editable(vec![changed_second_line()], DiffLayout::Inline))), cx);
    draw(&panel, cx);
    let generation = heard.borrow().texts[0].generation;

    // when
    cast(&panel, Cast::SetEditBase { generation, base: EditBase::Unavailable }, cx);
    draw(&panel, cx);

    // then
    assert!(panel.read_with(cx, |panel, cx| !panel.is_editable_diff(cx)));
    // asked for again, to colour the read-only diff
    assert_eq!(heard.borrow().texts.last().map(|request| request.purpose), Some(TextPurpose::Colours));
}
