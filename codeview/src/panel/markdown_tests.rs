//! A panel switching a Markdown text between Text and Rendered: what is offered, what the switch keeps, find in the
//! view shown, and the view saved with the tab.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{AppContext, ClickEvent, Entity, IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext, div, point, px, size};

use super::*;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::diff::Headers;
use crate::document::TextSource;
use crate::markdown::RenderedView;
use crate::position::{CaretPosition, ScrollPosition};
use crate::style::{Appearance, ViewStyle};
use crate::syntax::{Language, MAX_HIGHLIGHT_BYTES};

/// The events a panel sent that these tests look at, and the click targets it drew.
#[derive(Default)]
struct Heard {
    toggled: Vec<MarkdownView>,
    states: usize,
    targets: std::collections::HashMap<String, crate::host::ClickListener>,
}

/// A host that keeps the click targets drawn.
fn host(heard: &Rc<RefCell<Heard>>) -> crate::host::Host {
    let targets = heard.clone();
    crate::host::Host {
        keys: Rc::new(|_, _, _| crate::host::KeyOutcome::Unbound),
        clipboard: Rc::new(|_, _, _| {}),
        paste: Rc::new(|_, _| None),
        diff_mark: Rc::new(|_, _| div().into_any_element()),
        click_target: Rc::new(move |name, listener, _| drop(targets.borrow_mut().targets.insert(name, listener))),
        tooltip: Rc::new(|_, _, cx| cx.new(|_| gpui::EmptyView).into()),
    }
}

/// A window 800 by 400 px with an empty panel in it, and what the panel says.
fn window(cx: &mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Heard>>, &mut VisualTestContext) {
    let heard: Rc<RefCell<Heard>> = Rc::default();
    let host = host(&heard);
    cx.update(|cx| {
        ViewStyle::set_for_test(cx);
        cx.set_global(ResolvedScheme::resolve(Scheme::builtin(Appearance::Dark), &Overrides::new()));
        cx.set_global(host);
    });
    let cx = cx.add_empty_window();
    let panel = cx.new(Panel::new);
    let seen = heard.clone();
    cx.update(|_window, cx| {
        cx.subscribe(&panel, move |_, event: &Event, _| match event {
            Event::MarkdownViewToggled(view) => seen.borrow_mut().toggled.push(*view),
            Event::StateChanged => seen.borrow_mut().states += 1,
            _ => {}
        })
        .detach();
    });
    (panel, heard, cx)
}

fn markdown() -> Option<Language> {
    Language::for_alias("markdown")
}

/// `text` to edit, in `language`.
fn lines(text: &str, language: Option<Language>) -> Content {
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
    Content::Lines { source: Arc::new(TextSource::new(text)), language, edit: Some(policy) }
}

fn cast(panel: &Entity<Panel>, cast: Cast, cx: &mut VisualTestContext) {
    panel.update(cx, |panel, cx| panel.handle(cast, cx));
    draw(panel, cx);
}

/// Draws the panel, after anything running in the background has landed.
fn draw(panel: &Entity<Panel>, cx: &mut VisualTestContext) {
    cx.run_until_parked();
    let shown = panel.clone();
    cx.draw(point(px(0.), px(0.)), size(px(800.), px(400.)), |_, _| div().size_full().flex().flex_col().child(shown));
    cx.run_until_parked();
}

fn toggle(panel: &Entity<Panel>, cx: &mut VisualTestContext) {
    cast(panel, Cast::Command(Command::ToggleMarkdownView), cx);
}

fn shown(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Option<MarkdownView> {
    panel.read_with(cx, |panel, _| panel.markdown_view())
}

fn rendered(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Entity<RenderedView> {
    panel.read_with(cx, |panel, _| panel.rendered_view()).expect("Rendered is shown")
}

fn editor(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> Entity<TextEditor> {
    let Body::Edit(editor) = panel.read_with(cx, |panel, _| panel.body.clone()) else { panic!("not an editor") };
    editor
}

fn caret(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> CaretPosition {
    editor(panel, cx).read_with(cx, |editor, _| editor.caret_position())
}

fn top_line(panel: &Entity<Panel>, cx: &mut VisualTestContext) -> f64 {
    editor(panel, cx).read_with(cx, |editor, _| editor.scroll_position().top_line)
}

/// `count` paragraphs `para N`, a blank line apart: paragraph N is on source line 2N.
fn paragraphs(count: usize) -> String {
    (0..count).map(|index| format!("para {index}\n\n")).collect()
}

fn find(panel: &Entity<Panel>, query: &str, cx: &mut VisualTestContext) -> String {
    cast(panel, Cast::Command(Command::Find), cx);
    panel.update(cx, |panel, cx| {
        panel.select_find_query(cx);
        panel.type_find_query(query, cx);
    });
    draw(panel, cx);
    panel.read_with(cx, |panel, cx| panel.find_view(cx).count)
}

#[gpui::test]
fn should_offer_markdown_only_for_markdown_lines(cx: &mut TestAppContext) {
    // given
    let (md, _, cx) = window(cx);
    let rust = cx.new(Panel::new);
    let preview = cx.new(|cx| Panel::new(cx).unselectable());
    let diff = cx.new(Panel::new);

    // when
    cast(&md, Cast::Open(lines("# A\n", markdown())), cx);
    cast(&rust, Cast::Open(lines("fn a() {}\n", Language::for_alias("rust"))), cx);
    cast(&preview, Cast::Open(lines("# A\n", markdown())), cx);
    cast(&diff, Cast::Open(Content::Diff(DiffInput { files: Vec::new(), headers: Headers::Hidden, layout: DiffLayout::Inline, editable: Editable::No })), cx);

    // then
    let views: Vec<Option<MarkdownView>> = [&md, &rust, &preview, &diff].into_iter().map(|panel| shown(panel, cx)).collect();
    assert_eq!(views, vec![Some(MarkdownView::Text), None, None, None]);
}

#[gpui::test]
fn should_toggle_and_emit_once(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(lines("# A\n", markdown())), cx);
    let states = heard.borrow().states;

    // when
    toggle(&panel, cx);

    // then
    assert_eq!(shown(&panel, cx), Some(MarkdownView::Rendered));
    assert_eq!(heard.borrow().toggled, vec![MarkdownView::Rendered]);
    assert!(heard.borrow().states > states, "the state saved with the tab changed");
    assert_eq!(rendered(&panel, cx).read_with(cx, |view, _| view.doc().text.clone()), "A");
}

#[gpui::test]
fn should_render_the_unsaved_edits(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("old\n", markdown())), cx);
    cast(&panel, Cast::Type("# New\r".to_owned()), cx);

    // when
    toggle(&panel, cx);

    // then
    let text = rendered(&panel, cx).read_with(cx, |view, _| view.doc().text.clone());
    assert_eq!(text, "New\nold");
}

#[gpui::test]
fn should_keep_undo_and_caret_across_switches(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("text\n", markdown())), cx);
    cast(&panel, Cast::Type("x".to_owned()), cx);

    // when
    toggle(&panel, cx);
    toggle(&panel, cx);
    let kept = caret(&panel, cx);
    cast(&panel, Cast::Command(Command::Undo), cx);

    // then
    assert_eq!(kept, CaretPosition { line: 0, column: 1 });
    assert_eq!(editor(&panel, cx).read_with(cx, |editor, _| editor.text()), "text\n");
}

#[gpui::test]
fn should_put_the_caret_on_screen_after_reading_elsewhere(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines(&paragraphs(200), markdown())), cx);
    toggle(&panel, cx);
    rendered(&panel, cx).update(cx, |view, cx| view.reveal_source_line(300, cx));
    draw(&panel, cx);

    // when
    toggle(&panel, cx);

    // then
    assert_eq!(top_line(&panel, cx), 300.);
    assert_eq!(caret(&panel, cx), CaretPosition { line: 300, column: 0 });
}

#[gpui::test]
fn should_keep_the_place_both_ways(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines(&paragraphs(200), markdown())), cx);
    cast(&panel, Cast::Restore(PanelState { scroll: ScrollPosition { top_line: 101., left: 0. }, ..PanelState::default() }), cx);

    // when
    toggle(&panel, cx);
    let rendered_top = rendered(&panel, cx).read_with(cx, |view, _| view.top_line());
    toggle(&panel, cx);

    // then
    assert_eq!((rendered_top, top_line(&panel, cx)), (100, 100.));
}

#[gpui::test]
fn should_rerender_on_new_text_keeping_the_top_block(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    let text = paragraphs(200);
    cast(&panel, Cast::Open(lines(&text, markdown())), cx);
    toggle(&panel, cx);
    rendered(&panel, cx).update(cx, |view, cx| view.reveal_source_line(100, cx));
    draw(&panel, cx);

    // when
    cast(&panel, Cast::Open(lines(&format!("{text}one more\n"), markdown())), cx);

    // then
    let view = rendered(&panel, cx);
    assert_eq!(view.read_with(cx, |view, _| (view.top_block(), view.doc().blocks.len())), (50, 201));
}

#[gpui::test]
fn should_save_and_restore_the_view(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("# A\n", markdown())), cx);
    toggle(&panel, cx);
    let saved = panel.read_with(cx, |panel, _| panel.state());
    let (restored, defaulted) = (cx.new(Panel::new), cx.new(Panel::new));

    // when
    cast(&restored, Cast::Restore(saved.clone()), cx);
    cast(&restored, Cast::Open(lines("# A\n", markdown())), cx);
    cast(&defaulted, Cast::SetMarkdownView(MarkdownView::Rendered), cx);
    cast(&defaulted, Cast::Restore(PanelState { markdown: None, ..saved.clone() }), cx);
    cast(&defaulted, Cast::Open(lines("# A\n", markdown())), cx);

    // then
    assert_eq!(saved.markdown, Some(MarkdownView::Rendered));
    assert_eq!((shown(&restored, cx), shown(&defaulted, cx)), (Some(MarkdownView::Rendered), Some(MarkdownView::Rendered)));
    assert!(restored.read_with(cx, |panel, _| panel.rendered_view()).is_some());
}

#[gpui::test]
fn should_find_in_the_view_shown(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("a **b** c\n", markdown())), cx);
    toggle(&panel, cx);

    // when
    let in_rendered = find(&panel, "a b c", cx);
    toggle(&panel, cx);
    let in_text = panel.read_with(cx, |panel, cx| panel.find_view(cx));

    // then
    assert_eq!(in_rendered, "1 of 1");
    assert_eq!((in_text.open, in_text.query.as_str(), in_text.count.as_str()), (true, "a b c", "No results"));
}

#[gpui::test]
fn should_close_find_in_rendered_without_moving_the_caret(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("one two\n", markdown())), cx);
    toggle(&panel, cx);
    find(&panel, "two", cx);

    // when
    panel.update(cx, |panel, cx| panel.close_find(cx));
    toggle(&panel, cx);

    // then
    assert_eq!(caret(&panel, cx), CaretPosition { line: 0, column: 0 });
    assert_eq!(editor(&panel, cx).read_with(cx, |editor, _| editor.selected_text()), None);
}

#[gpui::test]
fn should_refuse_rendered_over_the_limit(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(lines(&"a".repeat(MAX_HIGHLIGHT_BYTES + 1), markdown())), cx);

    // when
    toggle(&panel, cx);

    // then
    assert_eq!(panel.read_with(cx, |panel, _| panel.markdown_offered()), Offer::TooLarge);
    assert_eq!(shown(&panel, cx), Some(MarkdownView::Text));
    assert!(heard.borrow().toggled.is_empty());
}

/// Clicks target `name` as IVE's MCP `click` does, after drawing the panel again.
fn click(panel: &Entity<Panel>, name: &str, heard: &RefCell<Heard>, cx: &mut VisualTestContext) {
    heard.borrow_mut().targets.clear();
    draw(panel, cx);
    let listener = heard.borrow().targets.get(name).cloned().unwrap_or_else(|| panic!("no click target {name}"));
    cx.update(|window, cx| listener(&ClickEvent::default(), window, cx));
    draw(panel, cx);
}

fn has_pill(cx: &mut VisualTestContext) -> bool {
    cx.debug_bounds("markdown-pill").is_some()
}

#[gpui::test]
fn should_show_the_pill_until_find_opens(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::Open(lines("# A\n", markdown())), cx);
    let before = has_pill(cx);

    // when
    cast(&panel, Cast::Command(Command::Find), cx);
    let finding = has_pill(cx);
    panel.update(cx, |panel, cx| panel.close_find(cx));
    draw(&panel, cx);

    // then
    assert_eq!((before, finding, has_pill(cx)), (true, false, true));
}

#[gpui::test]
fn should_draw_no_pill_for_other_texts(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);

    // when
    cast(&panel, Cast::Open(lines("fn a() {}\n", Language::for_alias("rust"))), cx);

    // then
    assert!(!has_pill(cx));
}

#[gpui::test]
fn should_switch_by_click_target(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(lines("# A\n", markdown())), cx);

    // when
    click(&panel, "markdown-view:rendered", &heard, cx);
    let rendered = shown(&panel, cx);
    click(&panel, "markdown-view:text", &heard, cx);
    click(&panel, "markdown-view:text", &heard, cx);

    // then
    assert_eq!((rendered, shown(&panel, cx)), (Some(MarkdownView::Rendered), Some(MarkdownView::Text)));
    assert_eq!(heard.borrow().toggled, vec![MarkdownView::Rendered, MarkdownView::Text]);
}

#[gpui::test]
fn should_grey_rendered_over_the_limit(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);

    // when
    cast(&panel, Cast::Open(lines(&"a".repeat(MAX_HIGHLIGHT_BYTES + 1), markdown())), cx);

    // then
    assert!(has_pill(cx));
    assert!(heard.borrow().targets.contains_key("markdown-view:text"));
    assert!(!heard.borrow().targets.contains_key("markdown-view:rendered"));
}
