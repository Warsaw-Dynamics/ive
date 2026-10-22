//! A diff panel's texts and colours, driven through casts and frames as a host would.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Entity, IntoElement, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, Point, Styled, TestAppContext,
    VisualTestContext, div, point, px, size,
};

use super::*;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::host::{ClipboardTarget, Host, KeyOutcome};
use crate::diff::texts::Placed;
use crate::diff::{DiffBody, DiffLine, GitPath, Headers, Hunk, LineKind, Mark};
use crate::style::{Appearance, ViewStyle};

/// What a diff panel asked of its host.
#[derive(Default)]
struct Asked {
    texts: Vec<usize>,
    requests: Vec<TextsRequest>,
    parses: Vec<ParseJob>,
}

/// A window showing a panel; its rows are 20 px high.
fn window(cx: &mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Asked>>, &mut VisualTestContext) {
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
    });
    let cx = cx.add_empty_window();
    let panel = cx.new(Panel::new);
    let asked: Rc<RefCell<Asked>> = Rc::default();
    let seen = asked.clone();
    cx.update(|_window, cx| {
        cx.subscribe(&panel, move |_, event: &Event, _| match event {
            Event::NeedTexts(request) => {
                seen.borrow_mut().texts.push(request.file);
                seen.borrow_mut().requests.push(*request);
            }
            Event::NeedParse(job) => seen.borrow_mut().parses.push(job.clone()),
            _ => {}
        })
        .detach();
    });
    (panel, asked, cx)
}

fn open(panel: &Entity<Panel>, files: Vec<FileDiff>, headers: Headers, cx: &mut VisualTestContext) {
    show(panel, DiffInput { files, headers, layout: DiffLayout::Inline, editable: Editable::No }, cx);
}

fn show(panel: &Entity<Panel>, input: DiffInput, cx: &mut VisualTestContext) {
    panel.update(cx, |panel, cx| panel.handle(Cast::Open(Content::Diff(input)), cx));
}

/// Draws a frame `rows` rows high.
fn draw(panel: &Entity<Panel>, rows: f32, cx: &mut VisualTestContext) {
    let panel = panel.clone();
    cx.draw(point(px(0.), px(0.)), size(px(800.), px(20. * rows)), |_, _| div().size_full().flex().flex_col().child(panel));
}

fn file(name: &str, lines: Vec<DiffLine>) -> FileDiff {
    let hunk = Hunk { header: "@@ -1 +1 @@".to_owned(), lines };
    FileDiff { path: GitPath::from_bytes(name.as_bytes()), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks: vec![hunk], cut: None } }
}

fn line(kind: LineKind, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
    DiffLine { kind, old, new, text: text.to_owned() }
}

fn source(text: &str) -> Option<Source> {
    Some(Source::from_vec(text.as_bytes().to_vec()))
}

/// Runs the parses the panel asked for and answers them.
fn answer_parses(panel: &Entity<Panel>, asked: &RefCell<Asked>, cx: &mut VisualTestContext) {
    let jobs = std::mem::take(&mut asked.borrow_mut().parses);
    for job in jobs {
        let (target, revision, syntax) = (job.target(), job.revision(), job.run().unwrap());
        panel.update(cx, |panel, cx| panel.handle(Cast::Parsed { target, revision, syntax }, cx));
    }
}

/// The colours of row `row`, as (text, colour key) pairs.
fn colours_of(panel: &Entity<Panel>, row: usize, text: &str, cx: &mut VisualTestContext) -> Vec<(String, String)> {
    let placed: Rc<[Placed]> = cx.update(|_window, cx| match &panel.read(cx).body {
        Body::Diff(diff) => diff.read(cx).row_colours(row),
        Body::Empty | Body::View(_) | Body::Edit(_) => panic!("not a diff"),
    });
    placed.iter().map(|(range, key)| (text[range.clone()].to_owned(), key.id().to_owned())).collect()
}

#[gpui::test]
fn should_colour_a_hunk_line_inside_a_block_comment(cx: &mut TestAppContext) {
    // given
    let (panel, asked, cx) = window(cx);
    let lines = vec![line(LineKind::Removed, Some(3), None, "old words"), line(LineKind::Added, None, Some(3), "new words")];
    open(&panel, vec![file("src/a.rs", lines)], Headers::Hidden, cx);
    draw(&panel, 10., cx);
    let asked_texts = asked.borrow().texts.clone();

    // when
    let (old, new) = (source("fn a() {}\n/* start\nold words\nend */\n"), source("fn a() {}\n/* start\nnew words\nend */\n"));
    let generation = asked.borrow().requests[0].generation;
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, generation, old, new }, cx));
    answer_parses(&panel, &asked, cx);

    // then
    let comment = |text: &str| vec![(text.to_owned(), "RUST.BLOCK_COMMENT".to_owned())];
    assert_eq!(asked_texts, [0]);
    assert_eq!((colours_of(&panel, 1, "old words", cx), colours_of(&panel, 2, "new words", cx)), (comment("old words"), comment("new words")));
}

#[gpui::test]
fn should_ask_texts_only_for_files_on_screen(cx: &mut TestAppContext) {
    // given
    let (panel, asked, cx) = window(cx);
    let added = |n: u32| line(LineKind::Added, None, Some(n), "let a = 1;");
    // a header, a hunk header and three lines: two files fill ten rows
    let files = (0..50).map(|n| file(&format!("f{n}.rs"), vec![added(1), added(2), added(3)])).collect();
    open(&panel, files, Headers::Shown, cx);

    // when
    draw(&panel, 10., cx);
    draw(&panel, 10., cx);

    // then
    assert_eq!(asked.borrow().texts, [0, 1]);
}

#[gpui::test]
fn should_show_a_file_without_texts_uncoloured(cx: &mut TestAppContext) {
    // given
    let (panel, asked, cx) = window(cx);
    let first = file("a.rs", vec![line(LineKind::Added, None, Some(1), "let a = 1;")]);
    let second = file("b.rs", vec![line(LineKind::Added, None, Some(1), "let b = 2;")]);
    open(&panel, vec![first, second], Headers::Shown, cx);
    draw(&panel, 10., cx);

    // when
    let generation = asked.borrow().requests[0].generation;
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, generation, old: None, new: None }, cx));

    // then
    assert_eq!((asked.borrow().texts.clone(), asked.borrow().parses.len()), (vec![0, 1], 0));
    assert_eq!((colours_of(&panel, 2, "let a = 1;", cx), colours_of(&panel, 5, "let b = 2;", cx)), (vec![], vec![]));
}

#[gpui::test]
fn should_drop_texts_answering_an_older_ask(cx: &mut TestAppContext) {
    // given
    let (panel, asked, cx) = window(cx);
    open(&panel, vec![file("a.rs", vec![line(LineKind::Added, None, Some(1), "let a = 1;")])], Headers::Hidden, cx);
    draw(&panel, 10., cx);
    let changed = file("a.rs", vec![line(LineKind::Added, None, Some(1), "let b = 2;")]);
    panel.update(cx, |panel, cx| panel.handle(Cast::SetFiles(vec![changed]), cx));
    draw(&panel, 10., cx);
    let (older, newer) = (asked.borrow().requests[0].generation, asked.borrow().requests[1].generation);

    // when
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, generation: older, old: None, new: source("let a = 1;\n") }, cx));
    let after_older = asked.borrow().parses.len();
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, generation: newer, old: None, new: source("let b = 2;\n") }, cx));

    // then
    assert_eq!((after_older, asked.borrow().parses.len()), (0, 1));
}

/// What `panel` told its host besides asking for texts and parses: the layouts toggled, and how often its state
/// changed.
#[derive(Default)]
struct Told {
    toggled: Vec<DiffLayout>,
    state_changes: usize,
}

fn told_by(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Rc<RefCell<Told>> {
    let told: Rc<RefCell<Told>> = Rc::default();
    let seen = told.clone();
    cx.update(|_window, cx| {
        cx.subscribe(panel, move |_, event: &Event, _| match event {
            Event::LayoutToggled(layout) => seen.borrow_mut().toggled.push(*layout),
            Event::StateChanged => seen.borrow_mut().state_changes += 1,
            _ => {}
        })
        .detach();
    });
    told
}

#[gpui::test]
fn should_report_a_toggled_layout_and_restore_a_tabs_own_without_reporting_it(cx: &mut TestAppContext) {
    // given
    let (panel, _asked, cx) = window(cx);
    open(&panel, vec![file("a.rs", vec![line(LineKind::Added, None, Some(1), "a")])], Headers::Hidden, cx);
    let told = told_by(&panel, cx);

    // when
    panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::ToggleLayout), cx));
    cx.run_until_parked();
    let toggled = panel.read_with(cx, |panel, _| panel.state().layout);
    panel.update(cx, |panel, cx| panel.handle(Cast::Restore(PanelState { layout: Some(DiffLayout::Inline), ..PanelState::default() }), cx));
    cx.run_until_parked();

    // then
    let restored = panel.read_with(cx, |panel, cx| (panel.state().layout, panel.diff_layout(cx)));
    assert_eq!((toggled, restored), (Some(DiffLayout::SideBySide), (Some(DiffLayout::Inline), Some(DiffLayout::Inline))));
    assert_eq!((told.borrow().toggled.clone(), told.borrow().state_changes), (vec![DiffLayout::SideBySide], 2));
}

/// What the panel's view puts on the clipboard.
fn clipboard(cx: &mut VisualTestContext) -> Rc<RefCell<Vec<String>>> {
    let copied: Rc<RefCell<Vec<String>>> = Rc::default();
    let seen = copied.clone();
    cx.update(|_window, cx| {
        cx.set_global(Host {
            keys: Rc::new(|_, _, _| KeyOutcome::Unbound),
            clipboard: Rc::new(move |_, text, _| seen.borrow_mut().push(text)),
            paste: Rc::new(|_, _| None),
            diff_mark: Rc::new(|_, _| div().into_any_element()),
            click_target: Rc::new(|_, _, _| {}),
        });
    });
    copied
}

/// Drags the mouse from `from` to `to` over the panel's diff, as a left-button selection.
fn drag(panel: &Entity<Panel>, from: Point<Pixels>, to: Point<Pixels>, cx: &mut VisualTestContext) {
    let Body::Diff(diff) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not a diff") };
    let press = MouseDownEvent { button: MouseButton::Left, position: from, modifiers: Modifiers::default(), click_count: 1, first_mouse: false };
    let moved = MouseMoveEvent { position: to, pressed_button: Some(MouseButton::Left), modifiers: Modifiers::default() };
    cx.update(|window, cx| diff.update(cx, |diff, cx| diff.on_mouse_down(&press, window, cx)));
    diff.update(cx, |diff, cx| diff.on_window_mouse_move(&moved, cx));
}

#[gpui::test]
fn should_copy_only_the_left_side(cx: &mut TestAppContext) {
    // given
    let (panel, _asked, cx) = window(cx);
    let copied = clipboard(cx);
    let lines = vec![
        line(LineKind::Context, Some(1), Some(1), "keep"),
        line(LineKind::Removed, Some(2), None, "old a"),
        line(LineKind::Removed, Some(3), None, "old b"),
        line(LineKind::Added, None, Some(2), "new a"),
        line(LineKind::Context, Some(4), Some(3), "end"),
    ];
    show(&panel, DiffInput { files: vec![file("a.rs", lines)], headers: Headers::Hidden, layout: DiffLayout::SideBySide, editable: Editable::No }, cx);
    // 800 px wide: the old panel left of x 400, the new one right of it; row 0 is the hunk header
    draw(&panel, 10., cx);
    let copy = |cx: &mut VisualTestContext| panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Copy(ClipboardTarget::Both)), cx));

    // when
    drag(&panel, point(px(1.), px(30.)), point(px(399.), px(90.)), cx);
    copy(cx);
    drag(&panel, point(px(401.), px(30.)), point(px(799.), px(90.)), cx);
    copy(cx);

    // then
    assert_eq!(*copied.borrow(), ["keep\nold a\nold b\nend", "keep\nnew a\nend"]);
}

/// Presses the divider between the side-by-side panels, as the `clicks`th click in a row.
fn press_divider(panel: &Entity<Panel>, clicks: usize, cx: &mut VisualTestContext) {
    let Body::Diff(diff) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not a diff") };
    let press = MouseDownEvent { button: MouseButton::Left, position: point(px(400.), px(30.)), modifiers: Modifiers::default(), click_count: clicks, first_mouse: false };
    diff.update(cx, |diff, cx| diff.on_divider_down(&press, cx));
}

/// Moves the held mouse to window x `to` over the panel's diff.
fn move_held(panel: &Entity<Panel>, to: Pixels, cx: &mut VisualTestContext) {
    let Body::Diff(diff) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not a diff") };
    let moved = MouseMoveEvent { position: point(to, px(30.)), pressed_button: Some(MouseButton::Left), modifiers: Modifiers::default() };
    diff.update(cx, |diff, cx| diff.on_window_mouse_move(&moved, cx));
}

/// A side-by-side diff of one removed line, `old`, and one added, `new`, drawn 800 px wide; row 1 is at y 30.
fn side_by_side(cx: &mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Vec<String>>>, &mut VisualTestContext) {
    let (panel, _asked, cx) = window(cx);
    let copied = clipboard(cx);
    let lines = vec![line(LineKind::Removed, Some(1), None, "old"), line(LineKind::Added, None, Some(1), "new")];
    show(&panel, DiffInput { files: vec![file("a.rs", lines)], headers: Headers::Hidden, layout: DiffLayout::SideBySide, editable: Editable::No }, cx);
    draw(&panel, 10., cx);
    (panel, copied, cx)
}

/// Selects row 1 from window x `from`, past the end of its text, back to window x `to` and copies it.
fn copy_between(panel: &Entity<Panel>, from: Pixels, to: Pixels, cx: &mut VisualTestContext) {
    drag(panel, point(from, px(30.)), point(to, px(30.)), cx);
    panel.update(cx, |panel, cx| panel.handle(Cast::Command(Command::Copy(ClipboardTarget::Both)), cx));
}

#[gpui::test]
fn should_select_in_the_right_panel_left_of_the_middle_once_the_divider_is_dragged_left(cx: &mut TestAppContext) {
    // given
    let (panel, copied, cx) = side_by_side(cx);

    // when
    // the divider at x 240 leaves the old panel 30 % of the 800 px
    press_divider(&panel, 1, cx);
    move_held(&panel, px(240.), cx);
    draw(&panel, 10., cx);
    copy_between(&panel, px(300.), px(241.), cx);

    // then
    assert_eq!(*copied.borrow(), ["new"]);
}

#[gpui::test]
fn should_put_the_divider_back_in_the_middle_on_a_double_click(cx: &mut TestAppContext) {
    // given
    let (panel, copied, cx) = side_by_side(cx);
    press_divider(&panel, 1, cx);
    move_held(&panel, px(240.), cx);
    draw(&panel, 10., cx);

    // when
    press_divider(&panel, 2, cx);
    draw(&panel, 10., cx);
    copy_between(&panel, px(300.), px(1.), cx);

    // then
    assert_eq!(*copied.borrow(), ["old"]);
}
