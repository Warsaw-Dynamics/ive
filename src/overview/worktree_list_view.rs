//! Draws the Worktrees list as the hub draws its list (`hub/view.rs`): the filter, sections of 24 px rows with the
//! selected one highlighted, the key strip. The workspace prepares the rows (`workspace/worktree_list.rs`).

use std::ops::Range;
use std::rc::Rc;

use gpui::{ClickEvent, Context, Div, Entity, FontWeight, SharedString, Stateful, div, img, prelude::*, px, rgb, uniform_list};

use super::git_ui::tag;
use super::worktree_list::{HOOKS_NOTE, WorktreeList};
use super::worktree_rows::{Group, GroupKind, Line, RowId};
use crate::click_targets::target;
use crate::icons;
use crate::paging::more_row;
use crate::project::ProjectId;
use crate::scrollbar::vertical_scrollbar;
use crate::theme::Theme;
use crate::tooltip::text_tooltip;
use crate::workspace::Workspace;

/// every line's: the list draws only the lines in view, so they share one height
const ROW_HEIGHT: f32 = 24.;
const FILTER_WIDTH: f32 = 420.;
const BAR_WIDTH: f32 = 3.;
const BAR_HEIGHT: f32 = 14.;
const ACTION_WIDTH: f32 = 84.;
const AGE_WIDTH: f32 = 64.;
/// a hidden worktree's row, until pointed at or selected
const HIDDEN_OPACITY: f32 = 0.55;
/// the icon of a branch only a remote has
const FADED_ICON: f32 = 0.45;

/// One row as drawn.
pub struct DrawnRow {
    pub id: RowId,
    /// the checkout's colour; none for a branch
    pub colour: Option<u32>,
    pub icon: &'static str,
    pub title: String,
    pub detail: String,
    pub tags: Vec<String>,
    /// `● N running`
    pub running: Option<String>,
    pub age: String,
    pub faded_icon: bool,
    pub hidden: bool,
    /// `Hide`, `Show`, `+ Worktree`; none for the main checkout
    pub action: Option<&'static str>,
}

/// A group's header and notes as drawn.
pub struct DrawnGroup {
    pub kind: GroupKind,
    pub title: String,
    pub hint: String,
    /// rows the filter matched, of which `shown` are shown
    pub total: usize,
    pub shown: usize,
    pub empty: Option<&'static str>,
}

pub struct Drawn {
    pub project: ProjectId,
    /// the rows of each group, drawn as they come into view
    pub groups: Vec<Group>,
    pub headers: Vec<DrawnGroup>,
    pub lines: Vec<Line>,
    pub selected: Option<RowId>,
    pub hidden_open: bool,
    /// a failed add or branch read, under the filter
    pub error: Option<String>,
    /// `(keys, what they do)`
    pub strip: Vec<(&'static str, &'static str)>,
}

/// The list; `list` is `None` for a plain folder, which has no filter. It fills the page and scrolls itself.
pub fn render_worktree_list(drawn: Drawn, list: Option<&Entity<WorktreeList>>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let filter = list.map(|list| div().w(px(FILTER_WIDTH)).child(list.read(cx).input().clone()));
    let error = drawn.error.clone().map(|error| div().text_xs().text_color(rgb(theme.error)).child(error));
    let strip = key_strip(&drawn.strip, theme);
    let count = drawn.lines.len();
    let (shown, entity) = (Rc::new(drawn), list.cloned());
    let rows = uniform_list("worktree-lines", count, cx.processor(move |workspace: &mut Workspace, range, _window, cx| render_lines(workspace, &shown, entity.as_ref(), range, theme, cx)))
        .size_full();
    let scrolled = list.map(|list| list.read(cx));
    let rows = match scrolled {
        Some(list) => rows.track_scroll(&list.scroll().list_handle()),
        None => rows,
    };
    let style = crate::theme::view_style(&theme);
    let scrollbar = scrolled.map(|list| vertical_scrollbar(list.scrollbar(), &style));
    div()
        .key_context("WorktreeList")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(px(6.))
        .text_sm()
        .children(filter)
        .children(error)
        .child(div().relative().flex_1().min_h_0().child(rows).children(scrollbar))
        .child(strip)
}

/// The lines in `range`: the rows among them are drawn now.
fn render_lines(workspace: &Workspace, drawn: &Drawn, list: Option<&Entity<WorktreeList>>, range: Range<usize>, theme: Theme, cx: &mut Context<Workspace>) -> Vec<Stateful<Div>> {
    let lines = &drawn.lines[range.start.min(drawn.lines.len())..range.end.min(drawn.lines.len())];
    let ids: Vec<&RowId> = lines.iter().filter_map(|line| match line {
        Line::Row { group, at } => Some(&drawn.groups[*group].rows[*at]),
        _ => None,
    }).collect();
    let mut rows = workspace.drawn_rows(&drawn.project, list.map(|list| list.read(cx)), &ids, theme).into_iter();
    lines
        .iter()
        .enumerate()
        .map(|(offset, line)| {
            let at = range.start + offset;
            let element = match line {
                Line::Header(group) => group_header(&drawn.headers[*group], drawn.hidden_open, list, theme, cx),
                Line::Empty(group) => empty_note(at, drawn.headers[*group].empty, theme),
                Line::Row { .. } => match rows.next().flatten() {
                    Some(row) => row_line(&row, drawn.selected.as_ref() == Some(&row.id), &drawn.project, theme, cx),
                    None => div().id(("worktree-gone", at)),
                },
                Line::More(group) => branch_more(at, list, &drawn.headers[*group], theme, cx),
            };
            element.flex_none().w_full().h(px(ROW_HEIGHT))
        })
        .collect()
}

fn empty_note(at: usize, empty: Option<&'static str>, theme: Theme) -> Stateful<Div> {
    div().id(("worktree-empty", at)).flex().items_center().pl(px(30.)).text_xs().text_color(rgb(theme.muted)).children(empty)
}

fn group_header(group: &DrawnGroup, hidden_open: bool, list: Option<&Entity<WorktreeList>>, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    // a row high, so the gap above a group is the row's room the text leaves
    let header = div()
        .id(SharedString::from(format!("worktree-group-{:?}", group.kind)))
        .flex()
        .items_end()
        .gap_2()
        .px_1()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(theme.muted));
    let title = div().child(group.title.clone());
    let hint = div().font_weight(FontWeight::NORMAL).child(group.hint.clone());
    match (group.kind, list) {
        (GroupKind::Hidden, Some(list)) => {
            let chevron = if hidden_open { icons::CHEVRON_DOWN } else { icons::CHEVRON_RIGHT };
            let list = list.clone();
            let toggle = target("worktree-hidden-group".to_owned(), move |_event: &ClickEvent, _window, cx| list.update(cx, |list, cx| list.toggle_hidden_group(cx)), cx);
            header.cursor_pointer().child(img(icons::embedded(theme.appearance, chevron)).flex_none().size(px(16.))).child(title).child(hint).on_click(toggle)
        }
        (GroupKind::Branches, _) => header.child(title).child(hint).tooltip(text_tooltip(HOOKS_NOTE)),
        _ => header.child(title).child(hint),
    }
}

fn row_line(row: &DrawnRow, selected: bool, project: &ProjectId, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let name = match &row.id {
        RowId::Checkout(path) => format!("worktree-row:{}", path.display()),
        RowId::Branch(branch) => format!("branch-row:{branch}"),
    };
    let group = SharedString::from(name.clone());
    let (opened, toggled) = ((project.clone(), row.id.clone()), (project.clone(), row.id.clone()));
    let open = target(name.clone(), cx.listener(move |workspace, _event, window, cx| workspace.worktree_row_clicked(&opened.0, &opened.1, window, cx)), cx);
    let action = row.action.map(|label| {
        let toggle = target(format!("worktree-toggle:{}", row_name(&row.id)), cx.listener(move |workspace, _event, _window, cx| {
            workspace.worktree_row_toggled(&toggled.0, &toggled.1, cx);
        }), cx);
        let button = div().id("worktree-action").px_1().rounded(px(4.)).text_color(rgb(theme.accent)).hover(|style| style.bg(rgb(theme.hover))).child(label);
        button.on_click(move |event, window, cx| {
            cx.stop_propagation();
            toggle(event, window, cx);
        })
    });
    let actions = div().flex().flex_none().justify_end().w(px(ACTION_WIDTH));
    let actions = if selected { actions } else { actions.invisible().group_hover(group.clone(), |style| style.visible()) };
    let content = div()
        .flex()
        .flex_1()
        .min_w_0()
        .items_center()
        .gap(px(6.))
        .child(img(icons::embedded(theme.appearance, row.icon)).flex_none().size(px(16.)).opacity(if row.faded_icon { FADED_ICON } else { 1. }))
        // a long title gives way before the tags at the row's end
        .child(div().flex_shrink_1().min_w_0().overflow_hidden().whitespace_nowrap().text_color(rgb(theme.text)).child(row.title.clone()))
        .child(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().text_xs().text_color(rgb(theme.muted)).child(row.detail.clone()))
        .children(row.tags.iter().map(|text| tag(text.clone(), theme.muted, theme)))
        .children(row.running.clone().map(|running| div().flex_none().text_xs().text_color(rgb(theme.mark_added)).child(running)))
        .child(div().flex_none().w(px(AGE_WIDTH)).text_right().whitespace_nowrap().text_xs().text_color(rgb(theme.muted)).child(row.age.clone()));
    let content = if row.hidden && !selected { content.opacity(HIDDEN_OPACITY).group_hover(group.clone(), |style| style.opacity(1.)) } else { content };
    let line = div()
        .id(SharedString::from(format!("worktree-line-{}", row_name(&row.id))))
        .group(group)
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .h(px(ROW_HEIGHT))
        .pl_1()
        .pr_1()
        .rounded(px(4.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(div().flex_none().w(px(BAR_WIDTH)).h(px(BAR_HEIGHT)).rounded(px(2.)).when_some(row.colour, |bar, colour| bar.bg(rgb(colour))))
        .child(content)
        .child(actions.children(action))
        .on_click(open);
    if selected { line.bg(rgb(theme.selected)) } else { line }
}

fn row_name(row: &RowId) -> String {
    match row {
        RowId::Checkout(path) => path.display().to_string(),
        RowId::Branch(branch) => branch.clone(),
    }
}

/// Show 10 more, Show all; an empty line for a plain folder, which pages nothing.
fn branch_more(at: usize, list: Option<&Entity<WorktreeList>>, group: &DrawnGroup, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let Some(list) = list else {
        return div().id(("branch-more-none", at));
    };
    let (more_list, all_list) = (list.clone(), list.clone());
    let more = target("branch-more".to_owned(), move |_event: &ClickEvent, _window, cx| more_list.update(cx, |list, cx| list.show_more(cx)), cx);
    let all = target("branch-all".to_owned(), move |_event: &ClickEvent, _window, cx| all_list.update(cx, |list, cx| list.show_all(cx)), cx);
    let row = more_row("branch-more", group.total, group.shown, theme, more, all).map(|row| row.pl(px(30.)).text_xs());
    row.unwrap_or_else(|| div().id(("branch-more-none", at)))
}

fn key_strip(strip: &[(&'static str, &'static str)], theme: Theme) -> Div {
    let entries = strip.iter().map(|(keys, does)| div().flex().flex_none().gap_1().child(div().text_color(rgb(theme.text)).child(*keys)).child(div().child(*does)));
    div().flex().flex_wrap().items_center().gap_x_3().gap_y_1().px_1().text_xs().text_color(rgb(theme.muted)).children(entries)
}
