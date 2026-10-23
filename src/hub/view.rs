//! The hub tab: the query row with its scope chips and Excluded switch, the list, the preview and the key strip,
//! sized as `mock/hub.css`.

use std::ops::Range;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, HighlightStyle, MouseButton, ScrollHandle, SharedString,
    Stateful, StyledText, Subscription, Task, div, img, prelude::*, px, relative, rgb,
};

use super::content::ContentFound;
use super::corpus::Corpus;
use super::files::Class;
use super::model::{HubState, mode, Mode};
use super::output::Taken;
use super::rows::{List, Row, Tag};
use super::search::Found;
use super::sources::HubScope;
use crate::age::Clock;
use crate::click_targets::target;
use crate::colors::checkout_colour;
use codeview::panel::Panel;
use crate::icons;
use crate::overview::git_ui::{layout_switch, mark_label, tag};
use crate::scratch::search::Generations;
use crate::tabs::TabId;
use crate::terminal::{Snapshot, snapshot_element};
use crate::text_input::TextInput;
use crate::theme::Theme;
use crate::workspace::Workspace;

const QUERY_ROW_HEIGHT: f32 = 44.;
const ROW_HEIGHT: f32 = 24.;
const STRIP_HEIGHT: f32 = 28.;
/// the preview's share of the hub's width
const PREVIEW_WIDTH: f32 = 0.44;
/// the opacity of a faded row icon: a branch only a remote has
const FADED_ICON: f32 = 0.45;

pub struct HubTab {
    pub state: HubState,
    /// the tab shown before the hub, which Escape goes back to
    pub back: Option<TabId>,
    pub input: Entity<TextInput>,
    /// the last search's result, over `searched`
    pub found: Option<Found>,
    /// the output the last contents search read, searched again while fresh
    pub read_output: ReadOutput,
    /// the last contents search's lines
    pub content: Option<ContentFound>,
    pub searched: Arc<Corpus>,
    /// the running search; a new one replaces it
    pub search: Option<Task<()>>,
    pub generations: Generations,
    pub list_scroll: ScrollHandle,
    /// what the preview shows, for the row `preview_key` names
    pub preview: Preview,
    pub preview_key: Option<String>,
    /// the preview being read; a new selection replaces it
    pub preview_task: Option<Task<()>>,
    /// said above the list: a worktree being created for a new terminal, or why it could not be
    pub notice: Option<String>,
    pub _input_events: Subscription,
}

/// The processes' output a hub's contents searches read.
pub enum ReadOutput {
    Unread,
    Read(Taken),
}

/// The preview right of the list, as `fzf --preview`.
pub enum Preview {
    None,
    Loading,
    /// a file in the editor's view, read-only
    File { title: String, panel: Entity<Panel> },
    /// lines of a process's terminal, drawn as it draws them
    Terminal { title: String, snapshot: Snapshot },
    Diff { title: String, panel: Entity<Panel> },
    /// a process's last lines, a run configuration's command, a worktree's summary
    Lines { title: String, lines: Vec<String>, mono: bool },
    Message(String),
}

impl HubTab {
    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }

    /// The query field shows the list's query, or the worktree question's.
    pub fn sync_input(&self, cx: &mut App) {
        let text = self.state.pick.as_ref().map_or(&self.state.query, |pick| &pick.query).clone();
        self.input.update(cx, |input, cx| {
            if input.text() != text {
                input.set_text(text, cx);
            }
        });
    }
}

/// What the workspace hands the view besides the tab: the list and the key strip's entries.
pub struct Drawn {
    pub list: List,
    /// `(keys, what they do)`
    pub strip: Vec<(String, &'static str)>,
    /// the scope chips: label, scope, shown
    pub chips: Vec<(String, HubScope)>,
    /// `N excluded not searched` while the switch is off
    pub excluded_note: Option<String>,
}

pub fn render_hub(id: TabId, hub: &HubTab, drawn: Drawn, workspace: &Workspace, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let input = hub.input.clone();
    div()
        .id("hub")
        .key_context("Hub")
        .flex()
        .flex_col()
        .size_full()
        .bg(rgb(theme.background))
        .text_sm()
        // the query keeps the keyboard: a click anywhere in the hub gives it back
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            let handle = input.focus_handle(cx);
            window.defer(cx, move |window, cx| window.focus(&handle, cx));
        })
        .child(query_row(id, hub, &drawn, theme, cx))
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(list_pane(id, hub, &drawn.list, workspace, theme, cx))
                .child(preview_pane(&hub.preview, theme, cx)),
        )
        .child(key_strip(&drawn, theme))
        .into_any_element()
}

fn query_row(id: TabId, hub: &HubTab, drawn: &Drawn, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let pick = hub.state.pick.as_ref().map(|pick| div().flex_none().text_color(rgb(theme.muted)).child(format!("{} ›", pick.title)));
    let counter = drawn.list.counter.clone().map(|counter| div().flex_none().text_xs().text_color(rgb(theme.muted)).child(counter));
    // new terminal mode lists every project and searches no files: no scope, no Excluded switch
    let searching = mode(&hub.state) != Mode::NewTerminal;
    let chips: Vec<Stateful<Div>> = match searching {
        true => drawn.chips.iter().map(|(label, scope)| scope_chip(id, label, scope, *scope == hub.state.scope, theme, cx)).collect(),
        false => Vec::new(),
    };
    let switches = searching.then(|| search_switches(id, hub, theme, cx));
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_2()
        .h(px(QUERY_ROW_HEIGHT))
        .px_3()
        .border_b_1()
        .border_color(rgb(theme.border))
        .child(img(icons::embedded(theme.appearance, icons::LOCATE)).flex_none().size(px(16.)))
        .children(pick)
        .child(div().flex_1().min_w_0().child(hub.input.clone()))
        .children(counter)
        .children(chips)
        .children(switches.into_iter().flatten())
}

/// The `:…` scope picker, the Excluded switch and, for a contents search, what it covers.
fn search_switches(id: TabId, hub: &HubTab, theme: Theme, cx: &mut Context<Workspace>) -> Vec<Stateful<Div>> {
    let picker = target("hub-scope-picker".to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.hub_typed(id, ":".to_owned(), window, cx)), cx);
    let excluded = target("hub-excluded".to_owned(), cx.listener(move |workspace, _event, window, cx| {
        workspace.hub_action(id, crate::script::keymap::Builtin::HubToggleExcluded, window, cx);
    }), cx);
    let switch_colour = if hub.state.excluded { theme.accent } else { theme.muted };
    let mut switches = vec![
        chip_box("hub-scope-picker", ":…", false, theme).on_click(picker),
        chip_box("hub-excluded", "Excluded", hub.state.excluded, theme).text_color(rgb(switch_colour)).on_click(excluded),
    ];
    if mode(&hub.state) == Mode::Content {
        let search_in = target("hub-search-in".to_owned(), cx.listener(move |workspace, _event, window, cx| {
            workspace.hub_action(id, crate::script::keymap::Builtin::HubCycleSearchIn, window, cx);
        }), cx);
        switches.push(chip_box("hub-search-in", hub.state.search_in.label(), false, theme).on_click(search_in));
    }
    switches
}

fn scope_chip(id: TabId, label: &str, scope: &HubScope, shown: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let name = match scope {
        HubScope::All => "all".to_owned(),
        HubScope::Project(project) => project.as_str().to_owned(),
        HubScope::Worktree(checkout) => checkout.path.display().to_string(),
    };
    let picked = scope.clone();
    let listener = target(format!("hub-scope:{name}"), cx.listener(move |workspace, _event, window, cx| workspace.hub_pick_scope(id, picked.clone(), window, cx)), cx);
    chip_box(SharedString::from(format!("hub-scope-{name}")), label, shown, theme).on_click(listener)
}

fn chip_box(id: impl Into<gpui::ElementId>, label: impl Into<SharedString>, shown: bool, theme: Theme) -> Stateful<Div> {
    let chip = div().id(id).flex_none().px_2().rounded(px(4.)).cursor_pointer().text_xs().child(label.into());
    match shown {
        true => chip.bg(rgb(theme.selected)).text_color(rgb(theme.text)),
        false => chip.bg(rgb(theme.hover)).text_color(rgb(theme.muted)),
    }
}

fn list_pane(id: TabId, hub: &HubTab, list: &List, workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let selected = hub.state.pick.as_ref().map_or(hub.state.selected, |pick| pick.selected);
    let mut pane = div().id("hub-list").flex().flex_col().flex_1().min_w_0().overflow_y_scroll().track_scroll(&hub.list_scroll).py_1();
    if let Some(message) = &list.message {
        pane = pane.child(div().px_3().py_2().text_color(rgb(theme.muted)).child(message.clone()));
    }
    for section in &list.sections {
        pane = pane.child(section_header(&section.title, section.note.as_deref(), theme));
        for index in section.rows.clone() {
            let look = RowLook { selected: index == selected, question: hub.state.pick.is_some() };
            pane = pane.child(row(id, index, &list.rows[index], look, workspace, theme, cx));
        }
        if let Some(more) = &section.more {
            pane = pane.child(div().px_3().py_1().text_xs().text_color(rgb(theme.muted)).child(more.clone()));
        }
    }
    if list.more > 0 {
        pane = pane.child(div().px_3().py_1().text_xs().text_color(rgb(theme.muted)).child(format!("… {} more; type more to narrow", list.more)));
    }
    pane
}

fn preview_pane(preview: &Preview, theme: Theme, cx: &App) -> Div {
    let pane = div().flex().flex_col().flex_none().w(relative(PREVIEW_WIDTH)).h_full().min_h_0().overflow_hidden().border_l_1().border_color(rgb(theme.border));
    let title = |title: &str| div().flex_none().px_3().py_1().text_xs().text_color(rgb(theme.muted)).overflow_hidden().whitespace_nowrap().child(title.to_owned());
    let mono = crate::theme::mono_font(cx);
    match preview {
        Preview::None => pane,
        Preview::Loading => pane.child(title("…")),
        Preview::Message(message) => pane.child(div().px_3().py_2().text_color(rgb(theme.muted)).child(message.clone())),
        Preview::Diff { title: name, panel } => {
            let head = div().flex().flex_none().items_center().pr_3().child(title(name).flex_1().min_w_0()).child(layout_switch(panel, theme, cx));
            pane.child(head).child(div().flex().flex_1().min_h_0().child(panel.clone()))
        }
        Preview::Lines { title: name, lines, mono: is_mono } => {
            let body = div().flex().flex_col().px_3().text_xs().overflow_hidden();
            let body = if *is_mono { body.font_family(mono) } else { body };
            pane.child(title(name)).child(body.children(lines.iter().map(|line| div().whitespace_nowrap().child(line.clone()))))
        }
        Preview::Terminal { title: name, snapshot } => pane.child(title(name)).child(snapshot_element(snapshot.clone(), theme.terminal)),
        Preview::File { title: name, panel } => pane.child(title(name)).child(div().flex().flex_1().min_h_0().child(panel.clone())),
    }
}

fn section_header(title: &str, note: Option<&str>, theme: Theme) -> Div {
    div()
        .flex()
        .gap_2()
        .px_3()
        .pt_2()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(theme.muted))
        .child(title.to_owned())
        .children(note.map(|note| div().font_weight(FontWeight::NORMAL).child(note.to_owned())))
}

/// How a row is drawn and named: selected, and whether it is one of the worktree question's.
#[derive(Clone, Copy)]
struct RowLook {
    selected: bool,
    question: bool,
}

fn row(id: TabId, index: usize, row: &Row, look: RowLook, workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let item = &row.item;
    // the worktree question's rows are named by their checkout, the list's by their number
    let name = match (&row.tag, look.question) {
        (Tag::Worktree(checkout), true) => format!("hub-worktree:{}", checkout.path.display()),
        _ => format!("hub-row:{}", index + 1),
    };
    let open = target(name, cx.listener(move |workspace, _event, window, cx| workspace.hub_open_row(id, index, window, cx)), cx);
    let title_colour = if item.greyed { theme.muted } else { theme.text };
    let class = match item.class {
        Class::Searched => None,
        Class::Excluded => Some("excluded"),
        Class::Ignored => Some("ignored"),
    };
    let class_style = match item.class {
        Class::Ignored => theme.tree_states.ignored,
        _ => theme.tree_states.excluded,
    };
    let class_tag = class.map(|class| tag(class, class_style.text.map_or(theme.muted, |text| text.0), theme));
    let mut line = div()
        .id(SharedString::from(format!("hub-row-{index}")))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .h(px(ROW_HEIGHT))
        .pl(px(12. + f32::from(row.depth) * 16.))
        .pr_3()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(div().flex_none().w(px(12.)).text_xs().text_color(rgb(theme.muted)).child(row.number.map(|number| number.to_string()).unwrap_or_default()))
        .child(img(icons::embedded(theme.appearance, item.icon)).flex_none().size(px(16.)).opacity(if item.faded { FADED_ICON } else { 1. }))
        // a long title gives way before the tags at the row's end
        .child(div().flex_shrink_1().min_w_0().overflow_hidden().whitespace_nowrap().text_color(rgb(title_colour)).child(highlighted(&item.title, &row.title_hits, theme)))
        .child(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().text_xs().text_color(rgb(theme.muted)).child(highlighted(&item.detail, &row.detail_hits, theme)))
        .children(item.mark.map(|mark| mark_label(mark, theme, cx)))
        .children(class_tag)
        .children(item.state.clone().map(|state| div().flex_none().text_xs().text_color(rgb(theme.muted)).child(state)))
        .children(worktree_tag(&row.tag, workspace, theme))
        .children(item.age.map(|age| div().flex_none().text_xs().text_color(rgb(theme.muted)).child(Clock::system().age(age))))
        .on_click(open);
    if look.selected {
        line = line.bg(rgb(theme.selected));
    }
    line
}

fn worktree_tag(row_tag: &Tag, workspace: &Workspace, theme: Theme) -> Option<Div> {
    match row_tag {
        Tag::None => None,
        Tag::Several(count) => Some(tag(format!("{count} worktrees"), theme.muted, theme)),
        Tag::Worktree(checkout) => {
            let colour = workspace.checkout(checkout).map_or(theme.muted, |found| checkout_colour(found, theme));
            Some(tag(workspace.checkout_label(checkout), colour, theme))
        }
    }
}

/// `text` with the matched characters in the accent colour.
fn highlighted(text: &str, hits: &[usize], theme: Theme) -> StyledText {
    let style = HighlightStyle { color: Some(rgb(theme.accent).into()), font_weight: Some(FontWeight::SEMIBOLD), ..HighlightStyle::default() };
    StyledText::new(text.to_owned()).with_highlights(char_ranges(text, hits).into_iter().map(|range| (range, style)))
}

/// The byte ranges of runs of the chars at `hits`.
fn char_ranges(text: &str, hits: &[usize]) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for (index, (at, letter)) in text.char_indices().enumerate() {
        if !hits.contains(&index) {
            continue;
        }
        let end = at + letter.len_utf8();
        match ranges.last_mut() {
            Some(last) if last.end == at => last.end = end,
            _ => ranges.push(at..end),
        }
    }
    ranges
}

fn key_strip(drawn: &Drawn, theme: Theme) -> Div {
    let entries = drawn.strip.iter().map(|(keys, does)| {
        div().flex().flex_none().gap_1().child(div().text_color(rgb(theme.text)).child(keys.clone())).child(div().child(*does))
    });
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_3()
        .h(px(STRIP_HEIGHT))
        .px_3()
        .border_t_1()
        .border_color(rgb(theme.border))
        .text_xs()
        .text_color(rgb(theme.muted))
        .overflow_hidden()
        .children(entries)
        .child(div().flex_1())
        .children(drawn.excluded_note.clone())
}

/// Whether Escape clears the query or leaves the hub, for the key strip.
pub fn escape_does(state: &HubState) -> &'static str {
    match mode(state) {
        Mode::Pick => "back",
        Mode::Empty => "back",
        Mode::NewTerminal if state.query.is_empty() => "back",
        _ => "clear",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_go_back_with_escape_on_an_empty_new_terminal_query() {
        // given
        let empty = HubState { new_terminal: true, ..HubState::default() };
        let typed = HubState { query: "topic".to_owned(), ..empty.clone() };

        // when
        let does = (escape_does(&empty), escape_does(&typed));

        // then
        assert_eq!(does, ("back", "clear"));
    }

    #[test]
    fn should_join_hits_into_byte_ranges() {
        // given
        let text = "żab.rs";

        // when
        let ranges = char_ranges(text, &[0, 1, 4]);

        // then
        assert_eq!(ranges, [0..3, 5..6]);
    }
}
