//! A diff panel's texts and colours, driven through casts and frames as a host would.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Entity, ParentElement, Styled, TestAppContext, VisualTestContext, div, point, px, size};

use super::*;
use crate::colors::{Overrides, ResolvedScheme, Scheme};
use crate::diff::texts::Placed;
use crate::diff::{DiffBody, DiffLine, GitPath, Headers, Hunk, LineKind, Mark};
use crate::style::{Appearance, ViewStyle};

/// What a diff panel asked of its host.
#[derive(Default)]
struct Asked {
    texts: Vec<usize>,
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
            Event::NeedTexts { file } => seen.borrow_mut().texts.push(*file),
            Event::NeedParse(job) => seen.borrow_mut().parses.push(job.clone()),
            _ => {}
        })
        .detach();
    });
    (panel, asked, cx)
}

fn open(panel: &Entity<Panel>, files: Vec<FileDiff>, headers: Headers, cx: &mut VisualTestContext) {
    let content = Content::Diff(DiffInput { files, headers, editable: Editable::No });
    panel.update(cx, |panel, cx| panel.handle(Cast::Open(content), cx));
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
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, old, new }, cx));
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
    panel.update(cx, |panel, cx| panel.handle(Cast::SetTexts { file: 0, old: None, new: None }, cx));

    // then
    assert_eq!((asked.borrow().texts.clone(), asked.borrow().parses.len()), (vec![0, 1], 0));
    assert_eq!((colours_of(&panel, 2, "let a = 1;", cx), colours_of(&panel, 5, "let b = 2;", cx)), (vec![], vec![]));
}
