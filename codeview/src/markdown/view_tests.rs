//! The Rendered view: drawing, scrolling by block, answering find, copying code blocks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{AppContext, ClickEvent, Entity, IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext, div, point, px, size};

use super::view::RenderedView;
use super::{Doc, parse};
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::find::{Compiled, FindOptions, FindQuery, FindText};
use crate::host::{ClickListener, Host, KeyOutcome};
use crate::style::{Appearance, ViewStyle};

/// What the view asked of its host.
#[derive(Default)]
pub(super) struct Seen {
    pub copied: Vec<String>,
    pub targets: HashMap<String, ClickListener>,
}

pub(super) fn host(seen: &Rc<RefCell<Seen>>) -> Host {
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

/// A window showing `source` rendered, 800 by 400 px, drawn once.
pub(super) fn shown<'a>(source: &str, cx: &'a mut TestAppContext) -> (Entity<RenderedView>, Rc<RefCell<Seen>>, &'a mut VisualTestContext) {
    let seen: Rc<RefCell<Seen>> = Rc::default();
    let host = host(&seen);
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
        cx.set_global(host);
    });
    let cx = cx.add_empty_window();
    let doc = Arc::new(parse(source));
    let view = cx.new(|cx| RenderedView::new(doc, cx));
    draw(&view, cx);
    (view, seen, cx)
}

pub(super) fn draw(view: &Entity<RenderedView>, cx: &mut VisualTestContext) {
    let view = view.clone();
    cx.draw(point(px(0.), px(0.)), size(px(800.), px(400.)), |_, _| div().size_full().flex().flex_col().child(view));
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
    draw(&view, cx);

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
    draw(&view, cx);

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
    draw(&view, cx);

    // when
    let longer = Arc::new(parse(&format!("{source}one more\n")));
    view.update(cx, |view, cx| view.set_doc(longer, cx));
    draw(&view, cx);

    // then
    assert_eq!(top_block(&view, cx), 50);
    assert_eq!(view.read_with(cx, |view, _| view.doc_revision()), 1);
}

#[gpui::test]
fn should_keep_the_top_block_inside_a_shorter_doc(cx: &mut TestAppContext) {
    // given
    let (view, _, cx) = shown(&paragraphs(200), cx);
    view.update(cx, |view, cx| view.reveal_source_line(300, cx));
    draw(&view, cx);

    // when
    view.update(cx, |view, cx| view.set_doc(Arc::new(parse(&paragraphs(3))), cx));
    draw(&view, cx);

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
    draw(&view, cx);
    let label = view.read_with(cx, |view, _| view.copy_label(1));
    cx.executor().advance_clock(Duration::from_millis(1500));
    draw(&view, cx);

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
