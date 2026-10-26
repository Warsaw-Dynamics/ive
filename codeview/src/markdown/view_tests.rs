//! The Rendered view: drawing, scrolling by block, answering find, copying code blocks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AppContext, Bounds, ClickEvent, Entity, IntoElement, KeyDownEvent, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, Styled, TestAppContext, VisualTestContext, Window, div, point, px, size,
};

use super::view::RenderedView;
use super::{Doc, parse};
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::find::{Compiled, FindOptions, FindQuery, FindText};
use crate::host::{ClickListener, ClipboardTarget, Command, Host, KeyOutcome};
use crate::style::{Appearance, ViewStyle};

/// What the view asked of its host.
#[derive(Default)]
pub(super) struct Seen {
    pub copied: Vec<String>,
    pub targets: HashMap<String, ClickListener>,
}

/// Ctrl+C copies and Ctrl+A selects all, as `ive.rn` binds them; no other key is bound.
fn keys_of(event: &KeyDownEvent) -> KeyOutcome {
    let keystroke = &event.keystroke;
    match (keystroke.modifiers.control, keystroke.key.as_str()) {
        (true, "c") => KeyOutcome::Command(Command::Copy(ClipboardTarget::Clipboard)),
        (true, "a") => KeyOutcome::Command(Command::SelectAll),
        _ => KeyOutcome::Unbound,
    }
}

pub(super) fn host(seen: &Rc<RefCell<Seen>>) -> Host {
    let (targets, copied) = (seen.clone(), seen.clone());
    Host {
        keys: Rc::new(|event, _, _| keys_of(event)),
        clipboard: Rc::new(move |_, text, _| copied.borrow_mut().copied.push(text)),
        paste: Rc::new(|_, _| None),
        diff_mark: Rc::new(|_, _| div().into_any_element()),
        click_target: Rc::new(move |name, listener, _| drop(targets.borrow_mut().targets.insert(name, listener))),
        tooltip: Rc::new(|_, _, cx| cx.new(|_| gpui::EmptyView).into()),
    }
}

/// A window showing `source` rendered, 800 by 400 px, drawn once.
pub(super) fn shown<'a>(source: &str, cx: &'a mut TestAppContext) -> (Entity<RenderedView>, Rc<RefCell<Seen>>, &'a mut VisualTestContext) {
    shown_doc(parse(source), cx)
}

/// A window showing `doc`, 800 by 400 px, drawn once.
pub(super) fn shown_doc(doc: Doc, cx: &mut TestAppContext) -> (Entity<RenderedView>, Rc<RefCell<Seen>>, &mut VisualTestContext) {
    let seen: Rc<RefCell<Seen>> = Rc::default();
    let host = host(&seen);
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
        cx.set_global(host);
    });
    let doc = Arc::new(doc);
    let (root, cx) = cx.add_window_view(|_window, cx| Page { view: cx.new(|cx| RenderedView::new(doc, cx)) });
    cx.simulate_resize(size(px(800.), px(400.)));
    let view = root.read_with(cx, |root, _| root.view.clone());
    draw(cx);
    (view, seen, cx)
}

/// The window's root: the view filling it.
struct Page {
    view: Entity<RenderedView>,
}

impl Render for Page {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().child(self.view.clone())
    }
}

pub(super) fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// `count` paragraphs `para N`, a blank line apart: paragraph N is block N, on source line 2N.
pub(super) fn paragraphs(count: usize) -> String {
    (0..count).map(|index| format!("para {index}\n\n")).collect()
}

fn matches_of(text: &FindText, query: &str) -> usize {
    let query = FindQuery { text: query.to_owned(), options: FindOptions { whole_word: true, ..FindOptions::default() } };
    let Compiled::Ready(regex) = query.compile() else { panic!("{query:?} does not compile") };
    text.search(&regex).ranges.len()
}

fn top_block(view: &Entity<RenderedView>, cx: &mut VisualTestContext) -> usize {
    view.read_with(cx, |view, _| view.top_block())
}

#[gpui::test]
fn should_render_an_empty_document(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown("", cx);

    // when
    let text = view.read_with(cx, |view, _| view.find_text());

    // then
    assert_eq!(matches_of(&text, "x"), 0);
    assert_eq!(view.read_with(cx, |view, _| view.top_line()), 0);
}

#[gpui::test]
fn should_search_pieces_apart(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown("| App | Stack |\n|---|---|\n| ive | Rust |\n", cx);

    // when
    let text = view.read_with(cx, |view, _| view.find_text());

    // then
    assert_eq!((matches_of(&text, "App"), matches_of(&text, "pStack"), matches_of(&text, "Rust")), (1, 0, 1));
}

#[gpui::test]
fn should_scroll_a_match_into_view(cx: &mut TestAppContext) {
    // given
    let source = paragraphs(200);
    let (view, _, cx) = shown(&source, cx);
    let at = view.read_with(cx, |view, _| view.doc().blocks[150].pieces[0].text.clone());

    // when
    view.update(cx, |view, cx| view.show_match(at, cx));
    draw(cx);

    // then
    let shown = view.read_with(cx, |view, _| view.shown_blocks());
    assert!(shown.contains(&150), "block 150 not in {shown:?}");
}

#[gpui::test]
fn should_reveal_a_source_line_at_the_top(cx: &mut TestAppContext) {
    // given
    let source = paragraphs(200);
    let (view, _, cx) = shown(&source, cx);

    // when
    view.update(cx, |view, cx| view.reveal_source_line(101, cx));
    draw(cx);

    // then
    assert_eq!(top_block(&view, cx), 50);
    assert_eq!(view.read_with(cx, |view, _| view.top_line()), 100);
}

#[gpui::test]
fn should_keep_the_top_block_on_a_new_doc(cx: &mut TestAppContext) {
    // given
    let source = paragraphs(200);
    let (view, _, cx) = shown(&source, cx);
    view.update(cx, |view, cx| view.reveal_source_line(100, cx));
    draw(cx);

    // when
    let longer = Arc::new(parse(&format!("{source}one more\n")));
    view.update(cx, |view, cx| view.set_doc(longer, cx));
    draw(cx);

    // then
    assert_eq!(top_block(&view, cx), 50);
    assert_eq!(view.read_with(cx, |view, _| view.doc_revision()), 1);
}

#[gpui::test]
fn should_keep_the_top_block_inside_a_shorter_doc(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(&paragraphs(200), cx);
    view.update(cx, |view, cx| view.reveal_source_line(300, cx));
    draw(cx);

    // when
    view.update(cx, |view, cx| view.set_doc(Arc::new(parse(&paragraphs(3))), cx));
    draw(cx);

    // then
    assert!(top_block(&view, cx) <= 2);
}

#[gpui::test]
fn should_show_a_link_target_as_its_tooltip(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown("see [the docs](https://example.com/docs) here\n", cx);

    // when
    let targets = view.read_with(cx, |view, _| (view.link_at(6), view.link_at(1)));

    // then
    assert_eq!(targets, (Some("https://example.com/docs".into()), None));
}

#[gpui::test]
fn should_copy_a_code_block_and_say_copied(cx: &mut TestAppContext) {
    // given
    let (view, seen, cx) = shown("para\n\n```\nx\n  y\n```\n", cx);
    let listener = seen.borrow().targets.get("markdown-copy:1").cloned().expect("a Copy button for the first code block");

    // when
    cx.update(|window, cx| listener(&ClickEvent::default(), window, cx));
    draw(cx);
    let label = view.read_with(cx, |view, _| view.copy_label(1));
    cx.executor().advance_clock(Duration::from_millis(1500));
    draw(cx);

    // then
    assert_eq!(seen.borrow().copied, vec!["x\n  y".to_owned()]);
    assert_eq!((label, view.read_with(cx, |view, _| view.copy_label(1))), ("Copied", "Copy"));
}

#[gpui::test]
fn should_draw_every_kind_of_block(cx: &mut TestAppContext) {
    // given
    let source = "# H\n\n> q\n\n- [x] t\n  1. n\n\n| a | b |\n|---|--:|\n| c |\n\n---\n\n<div>h</div>\n\n```rust\nfn a() {}\n```\n";

    // when
    let (view, _, cx) = shown(source, cx);

    // then
    let doc: Arc<Doc> = view.read_with(cx, |view, _| view.doc().clone());
    assert_eq!(doc.blocks.len(), 8);
    assert_eq!(view.read_with(cx, |view, _| view.shown_blocks()), (0..8).collect::<Vec<_>>());
}

fn bounds_of(view: &Entity<RenderedView>, piece_start: usize, cx: &mut VisualTestContext) -> Bounds<Pixels> {
    view.read_with(cx, |view, _| view.piece_bounds(piece_start)).unwrap_or_else(|| panic!("no piece drawn at {piece_start}"))
}

/// Just inside the left end of a piece's first line.
fn left_of(bounds: Bounds<Pixels>) -> Point<Pixels> {
    point(bounds.left() + px(1.), bounds.top() + px(4.))
}

/// Past the end of a piece's first line.
fn right_of(bounds: Bounds<Pixels>) -> Point<Pixels> {
    point(bounds.right() - px(1.), bounds.top() + px(4.))
}

/// A left press and release at `at`; the view is drawn again after each, as the window would.
fn press(at: Point<Pixels>, clicks: (usize, Modifiers), cx: &mut VisualTestContext) {
    let (click_count, modifiers) = clicks;
    cx.simulate_event(MouseDownEvent { button: MouseButton::Left, position: at, modifiers, click_count, first_mouse: false });
    draw(cx);
    cx.simulate_event(MouseUpEvent { button: MouseButton::Left, position: at, modifiers, click_count });
    draw(cx);
}

fn drag(from: Point<Pixels>, to: Point<Pixels>, cx: &mut VisualTestContext) {
    let modifiers = Modifiers::none();
    cx.simulate_event(MouseDownEvent { button: MouseButton::Left, position: from, modifiers, click_count: 1, first_mouse: false });
    draw(cx);
    cx.simulate_event(MouseMoveEvent { position: to, pressed_button: Some(MouseButton::Left), modifiers });
    draw(cx);
    cx.simulate_event(MouseUpEvent { button: MouseButton::Left, position: to, modifiers, click_count: 1 });
    draw(cx);
}

fn selected(view: &Entity<RenderedView>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| view.selected_text())
}

const THREE: &str = "first para\n\nsecond\n\nthird para\n";

#[gpui::test]
fn should_select_across_blocks_by_dragging(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(THREE, cx);
    let (first, third) = (bounds_of(&view, 0, cx), bounds_of(&view, 18, cx));

    // when
    drag(left_of(first), right_of(third), cx);

    // then
    assert_eq!(selected(&view, cx).as_deref(), Some("first para\n\nsecond\n\nthird para"));
}

#[gpui::test]
fn should_select_a_word_on_double_click(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(THREE, cx);
    let second = bounds_of(&view, 11, cx);

    // when
    press(left_of(second), (2, Modifiers::none()), cx);

    // then
    assert_eq!(selected(&view, cx).as_deref(), Some("second"));
}

#[gpui::test]
fn should_select_the_piece_on_triple_click(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(THREE, cx);
    let first = bounds_of(&view, 0, cx);

    // when
    press(left_of(first), (3, Modifiers::none()), cx);

    // then
    assert_eq!(selected(&view, cx).as_deref(), Some("first para"));
}

#[gpui::test]
fn should_extend_with_shift_click(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(THREE, cx);
    let (first, second) = (bounds_of(&view, 0, cx), bounds_of(&view, 11, cx));
    press(left_of(first), (1, Modifiers::none()), cx);

    // when
    press(right_of(second), (1, Modifiers::shift()), cx);

    // then
    assert_eq!(selected(&view, cx).as_deref(), Some("first para\n\nsecond"));
}

#[gpui::test]
fn should_select_all_and_copy(cx: &mut TestAppContext) {
    // given
    let (view, seen, cx) = shown("- a\n- b\n\n| x | y |\n|---|---|\n", cx);
    let first = bounds_of(&view, 0, cx);
    press(left_of(first), (1, Modifiers::none()), cx);

    // when
    cx.simulate_keystrokes("ctrl-a");
    draw(cx);
    cx.simulate_keystrokes("ctrl-c");

    // then
    assert_eq!(seen.borrow().copied, vec!["- a\n- b\n\nx\ty".to_owned()]);
}

#[gpui::test]
fn should_show_the_copy_button_on_hover_and_not_select_its_label(cx: &mut TestAppContext) {
    // given
    let (view, seen, cx) = shown("```\ncode\n```\n", cx);
    let button = cx.debug_bounds("markdown-copy-1").expect("the Copy button is drawn");
    press(button.center(), (1, Modifiers::none()), cx);
    let hidden = seen.borrow().copied.is_empty();

    // when
    cx.simulate_event(MouseMoveEvent { position: button.center(), pressed_button: None, modifiers: Modifiers::none() });
    draw(cx);
    press(button.center(), (1, Modifiers::none()), cx);

    // then
    assert_eq!(selected(&view, cx), None);
    assert!(hidden, "the button takes no click before the mouse is over the block");
    assert_eq!(seen.borrow().copied, vec!["code".to_owned()], "the press reached the button");
}

#[gpui::test]
fn should_prefill_find_with_a_one_line_selection(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(THREE, cx);
    let second = bounds_of(&view, 11, cx);
    press(left_of(second), (2, Modifiers::none()), cx);

    // when
    let prefill = view.read_with(cx, |view, _| (view.find_prefill(), view.find_anchor()));

    // then
    assert_eq!(prefill, (Some(("second".to_owned(), 11)), 11..17));
}

#[test]
fn should_not_hit_text_that_was_measured_but_never_drawn() {
    // given
    let measured_only = vec![super::view::Placed::new(0, gpui::TextLayout::default())];

    // when
    let hit = super::view::hit_in(&measured_only, point(px(10.), px(10.)));

    // then
    assert_eq!(hit, None);
}

#[gpui::test]
fn should_scroll_to_a_match_deep_in_a_tall_block(cx: &mut TestAppContext) {
    // given
    let code: String = (0..120).map(|line| if line == 100 { "needle\n".to_owned() } else { format!("line {line}\n") }).collect();
    let (view, _, cx) = shown(&format!("intro\n\n```\n{code}```\n"), cx);
    let at = view.read_with(cx, |view, _| view.doc().text.find("needle").expect("the code holds it"));

    // when
    view.update(cx, |view, cx| view.show_match(at..at + 6, cx));
    draw(cx);
    draw(cx);

    // then
    let y = view.read_with(cx, |view, _| view.position_of(at)).expect("the match is drawn");
    assert!(y > px(0.) && y < px(400.), "the match is at {y:?}, out of the 400 px view");
}

#[gpui::test]
fn should_mark_many_matches_without_slowing_frames(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(&paragraphs(20_000), cx);
    let length = view.read_with(cx, |view, _| view.doc().text.len());
    let ranges: Vec<std::ops::Range<usize>> = (0..length).step_by(2).take(100_000).map(|at| at..at + 1).collect();
    let lines: Vec<usize> = vec![0; ranges.len()];
    let marks = crate::find::marks::FindMarks { ranges: Arc::from(ranges), lines: Arc::from(lines), current: Some(0) };

    // when
    let started = std::time::Instant::now();
    view.update(cx, |view, cx| view.set_find_marks(Some(marks), cx));
    for _ in 0..3 {
        draw(cx);
    }

    // then
    assert!(started.elapsed() < Duration::from_secs(1), "took {:?}", started.elapsed());
}

#[gpui::test]
fn should_copy_a_long_code_block_whole_by_its_one_button(cx: &mut TestAppContext) {
    // given
    let code: String = (0..250).map(|line| format!("line {line}\n")).collect();
    let doc = super::build(format!("```\n{code}```\n").as_bytes(), &crate::syntax::Cancel::default());
    let (view, seen, cx) = shown_doc(doc, cx);
    view.update(cx, |view, cx| view.reveal_source_line(230, cx));
    seen.borrow_mut().targets.clear();
    draw(cx);
    let listener = seen.borrow().targets.get("markdown-copy:1").cloned().expect("the last part's Copy button, named for its block");

    // when
    cx.update(|window, cx| listener(&ClickEvent::default(), window, cx));
    draw(cx);

    // then
    assert_eq!(seen.borrow().copied, vec![code.trim_end().to_owned()]);
    let names: Vec<String> = seen.borrow().targets.keys().filter(|name| name.starts_with("markdown-copy")).cloned().collect();
    assert_eq!(names, vec!["markdown-copy:1".to_owned()], "one target for the block, not one per part");
}

#[gpui::test]
fn should_scroll_by_page_and_to_the_ends_from_the_keyboard(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(&paragraphs(200), cx);
    let first = bounds_of(&view, 0, cx);
    press(left_of(first), (1, Modifiers::none()), cx);

    // when
    cx.simulate_keystrokes("pagedown");
    draw(cx);
    let paged = top_block(&view, cx);
    cx.simulate_keystrokes("end");
    draw(cx);
    let ended = top_block(&view, cx);
    cx.simulate_keystrokes("home");
    draw(cx);

    // then
    assert!(paged > 3, "a page down scrolls a view's worth, got to block {paged}");
    assert!(ended > 180, "End goes to the bottom, got to block {ended}");
    assert_eq!(top_block(&view, cx), 0);
}
