//! A panel breaking an edited text's long lines into rows: switched before or after the text opens, told to the
//! host, and moving and scrolling by rows.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{AppContext, ClickEvent, Entity, IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext, div, point, px, size};

use super::*;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::document::TextSource;
use crate::position::CaretPosition;
use crate::style::{Appearance, ViewStyle};
use crate::text_editor::{Motion, Selecting};

/// The events a panel sent that these tests look at, and the click targets it drew.
#[derive(Default)]
struct Heard {
    toggled: Vec<SoftWrap>,
    gutter_menus: usize,
    targets: std::collections::HashMap<String, crate::host::ClickListener>,
}

/// A window 800 by 400 px with an empty panel in it, and what the panel says.
fn window(cx: &mut TestAppContext) -> (Entity<Panel>, Rc<RefCell<Heard>>, &mut VisualTestContext) {
    let heard: Rc<RefCell<Heard>> = Rc::default();
    let targets = heard.clone();
    let host = crate::host::Host {
        keys: Rc::new(|_, _, _| crate::host::KeyOutcome::Unbound),
        clipboard: Rc::new(|_, _, _| {}),
        paste: Rc::new(|_, _| None),
        diff_mark: Rc::new(|_, _| div().into_any_element()),
        click_target: Rc::new(move |name, listener, _| drop(targets.borrow_mut().targets.insert(name, listener))),
        tooltip: Rc::new(|_, _, cx| cx.new(|_| gpui::EmptyView).into()),
    };
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
            Event::SoftWrapToggled(soft_wrap) => seen.borrow_mut().toggled.push(*soft_wrap),
            Event::GutterMenu => seen.borrow_mut().gutter_menus += 1,
            _ => {}
        })
        .detach();
    });
    (panel, heard, cx)
}

/// `text` to edit.
fn edited(text: &str) -> Content {
    let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
    Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit: Some(policy) }
}

/// `count` lines of 300 characters, each wrapping into several rows of an 800 px wide panel.
fn long_lines(count: usize) -> String {
    (0..count).map(|line| format!("{line} {}\n", "word ".repeat(60))).collect()
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

/// Moves the caret as its key would, then draws twice: the first frame lays out, the second shows the caret.
fn press(panel: &Entity<Panel>, motion: Motion, cx: &mut VisualTestContext) {
    cast(panel, Cast::Command(Command::Move(motion, Selecting::Move)), cx);
    draw(panel, cx);
}

#[gpui::test]
fn should_open_an_edited_text_wrapped_as_set_before(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);

    // when
    cast(&panel, Cast::SetSoftWrap(SoftWrap::On), cx);
    cast(&panel, Cast::Open(edited("text\n")), cx);

    // then
    let wraps = editor(&panel, cx).read_with(cx, |editor, _| editor.soft_wrap());
    assert_eq!((wraps, panel.read_with(cx, |panel, _| panel.edited_soft_wrap())), (SoftWrap::On, Some(SoftWrap::On)));
    assert!(heard.borrow().toggled.is_empty(), "a host's setting is not a switch to tell it of");
}

#[gpui::test]
fn should_switch_and_tell_the_host(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(edited("text\n")), cx);

    // when
    cast(&panel, Cast::Command(Command::ToggleSoftWrap), cx);
    let switched_on = panel.read_with(cx, |panel, _| panel.edited_soft_wrap());
    cast(&panel, Cast::Command(Command::ToggleSoftWrap), cx);

    // then
    assert_eq!(switched_on, Some(SoftWrap::On));
    assert_eq!(heard.borrow().toggled, [SoftWrap::On, SoftWrap::Off]);
}

#[gpui::test]
fn should_not_wrap_a_text_only_viewed(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(Content::Lines { source: Arc::new(TextSource::new("text\n")), language: None, edit: None }), cx);

    // when
    cast(&panel, Cast::Command(Command::ToggleSoftWrap), cx);

    // then
    assert_eq!(panel.read_with(cx, |panel, _| panel.edited_soft_wrap()), None);
    assert!(heard.borrow().toggled.is_empty());
}

#[gpui::test]
fn should_move_down_a_row_of_the_same_line(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    cast(&panel, Cast::SetSoftWrap(SoftWrap::On), cx);
    cast(&panel, Cast::Open(edited(&long_lines(3))), cx);
    // the first frame measures the width the rows wrap at
    draw(&panel, cx);

    // when
    press(&panel, Motion::Down, cx);

    // then
    let moved = caret(&panel, cx);
    assert_eq!(moved.line, 0, "the caret stays on the wrapped line");
    assert!(moved.column > 0, "the caret went down a row: {moved:?}");
}

#[gpui::test]
fn should_scroll_wrapped_lines_down_to_their_last_row(cx: &mut TestAppContext) {
    // given
    let (panel, _, cx) = window(cx);
    let count = 40;
    cast(&panel, Cast::Open(edited(&long_lines(count))), cx);
    press(&panel, Motion::DocumentEnd, cx);
    let unwrapped = top_line(&panel, cx);

    // when
    cast(&panel, Cast::Command(Command::ToggleSoftWrap), cx);
    draw(&panel, cx);

    // then: the rows of the last lines fill the viewport, so the top is past where single rows put it
    let wrapped = top_line(&panel, cx);
    assert!(wrapped > unwrapped && wrapped < count as f64, "top line {wrapped}, unwrapped {unwrapped}");
}

#[gpui::test]
fn should_ask_for_the_gutter_menu_by_click_target(cx: &mut TestAppContext) {
    // given
    let (panel, heard, cx) = window(cx);
    cast(&panel, Cast::Open(edited("text\n")), cx);
    let listener = heard.borrow().targets.get("gutter-menu").cloned().expect("the gutter's target is drawn");

    // when
    cx.update(|window, cx| listener(&ClickEvent::default(), window, cx));

    // then
    assert_eq!(heard.borrow().gutter_menus, 1);
}
