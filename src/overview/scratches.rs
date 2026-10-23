//! The project overview's Scratches section: search, date groups, paging, New scratch, rename and delete.
//! Every button and row is a click target (README, "MCP"), so agents can drive it.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::time::Duration;

use gpui::{
    AnyElement, ClickEvent, Context, Div, Entity, Focusable, KeyDownEvent, PromptLevel, SharedString,
    ScrollStrategy, Stateful, Subscription, Task, WeakEntity, Window, div, img, prelude::*, rgb, uniform_list,
};

use crate::age::{Clock, unix_seconds};
use crate::click_targets::target;
use crate::selectable::selectable;
use crate::icons;
use crate::paging::{self, Paging};
use crate::paths::tilde_home;
use crate::project::ProjectId;
use crate::scratch::groups::{self, Group};
use crate::scratch::icon::scratch_icon;
use crate::scratch::search::{self, Answer, Found, Generations, Ticket};
use crate::scratch::store::{self, ScratchError, ScratchInfo, ScratchStore};
use crate::scratch::{ScratchRef, log_error};
use crate::scrollbar::{ListScroll, ScrollState, vertical_scrollbar};
use crate::text_input::{TextInput, TextInputEvent};
use crate::theme::Theme;
use crate::workspace::Workspace;

const CLOSED: &str = "the IVE window is closed";
/// Typing in the search field searches this long after the last key.
const SEARCH_DELAY: Duration = Duration::from_millis(150);
/// the open name input's click target, New scratch's or Rename's (one is open at a time)
const NAME_TARGET: &str = "scratch-name";

pub struct ScratchesTarget {
    pub workspace: WeakEntity<Workspace>,
    pub project: ProjectId,
    pub store: Result<ScratchStore, String>,
}

#[derive(PartialEq)]
enum Listing {
    Loading,
    Loaded(Vec<ScratchInfo>),
    Failed(String),
}

enum InlineMode {
    New,
    Rename(String),
}

/// The name input of New scratch, or of Rename in place of a row.
struct Inline {
    mode: InlineMode,
    input: Entity<TextInput>,
    error: Option<String>,
    _events: Subscription,
}

impl Inline {
    fn renames(&self, name: &str) -> bool {
        matches!(&self.mode, InlineMode::Rename(old) if old == name)
    }
}

/// A line of the listing.
enum Line<'a> {
    /// a group and how many scratches it has
    Header(Group, usize),
    Row(&'a ScratchInfo),
    /// Show 10 more, Show all: some of the group's `total` are not shown
    More { group: Group, total: usize, shown: usize },
}

/// Why the listing has no lines: it is read, found nothing, or failed.
enum NoLines {
    Muted(String),
    Failed(String),
}

impl NoLines {
    fn into_any_element(self, theme: Theme) -> AnyElement {
        match self {
            Self::Muted(message) => muted(message, theme),
            Self::Failed(reason) => div().text_color(rgb(theme.error)).child(reason).into_any_element(),
        }
    }
}

/// What every row of one frame shares.
#[derive(Clone, Copy)]
struct Frame {
    clock: Clock,
    theme: Theme,
}

pub struct ScratchesSection {
    target: ScratchesTarget,
    listing: Listing,
    query: String,
    generations: Generations,
    /// the content search's answers for `query`
    found: Found,
    search_input: Entity<TextInput>,
    inline: Option<Inline>,
    collapsed: HashSet<Group>,
    /// collapsed while searching; a new search opens every group, so no match is hidden
    collapsed_in_search: HashSet<Group>,
    paging: HashMap<Group, Paging>,
    /// the listing draws only the lines in view
    list: ListScroll,
    scrollbar: ScrollState,
    error: Option<String>,
    list_task: Option<Task<()>>,
    search_task: Option<Task<()>>,
    _search_events: Subscription,
}

/// Makes the section on first show and re-reads its folder on every later one. The re-read is deferred: the
/// show may come from inside one of the section's own handlers (a delete closing the shown scratch tab).
pub fn shown(slot: &mut Option<Entity<ScratchesSection>>, make: impl FnOnce() -> ScratchesTarget, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(section) = slot.clone() {
        cx.defer(move |cx| section.update(cx, |section, cx| section.reload(cx)));
        return;
    }
    let target = make();
    *slot = Some(cx.new(|cx| ScratchesSection::new(target, window, cx)));
}

impl ScratchesSection {
    pub fn new(target: ScratchesTarget, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(TextInput::new);
        search_input.update(cx, |input, _cx| {
            input.placeholder("Search names and contents…");
            input.set_target("scratch-search");
        });
        let search_events = cx.subscribe_in(&search_input, window, |section, _input, event: &TextInputEvent, _window, cx| {
            if let TextInputEvent::Changed(query) = event {
                section.search_for(query.clone(), cx);
            }
        });
        let list = ListScroll::new();
        let scrollbar = ScrollState::for_list(&list);
        let mut section = Self {
            target,
            listing: Listing::Loading,
            query: String::new(),
            generations: Generations::default(),
            found: Found::default(),
            search_input,
            inline: None,
            collapsed: HashSet::from([Group::Older]),
            collapsed_in_search: HashSet::new(),
            paging: HashMap::new(),
            list,
            scrollbar,
            error: None,
            list_task: None,
            search_task: None,
            _search_events: search_events,
        };
        section.reload(cx);
        section
    }

    /// Re-reads the folder on the background executor.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let store = match &self.target.store {
            Ok(store) => store.clone(),
            Err(reason) => {
                self.listing = Listing::Failed(reason.clone());
                return;
            }
        };
        self.list_task = Some(cx.spawn(async move |section, cx| {
            let listing = cx.background_executor().spawn(async move { store.list() }).await;
            section.update(cx, |section, cx| section.listed(listing, cx)).ok();
        }));
    }

    /// A listing like the shown one changes nothing; a new one searches only what the answers lack.
    fn listed(&mut self, listing: Result<Vec<ScratchInfo>, ScratchError>, cx: &mut Context<Self>) {
        let listing = match listing {
            Ok(scratches) => Listing::Loaded(scratches),
            Err(error) => {
                log_error(&error);
                Listing::Failed(error.to_string())
            }
        };
        if listing == self.listing {
            return;
        }
        self.listing = listing;
        self.start_search(Duration::ZERO, cx);
        cx.notify();
    }

    fn search_for(&mut self, query: String, cx: &mut Context<Self>) {
        if self.query.is_empty() {
            self.collapsed_in_search.clear();
        }
        self.query = query;
        self.found = Found::default();
        self.paging.clear();
        self.start_search(SEARCH_DELAY, cx);
        cx.notify();
    }

    fn is_searching(&self) -> bool {
        !self.query.is_empty()
    }

    /// Reads the scratches the answers lack, after `delay`. A newer search stops this one, also mid-way through
    /// the files, and keeps its late result from applying.
    fn start_search(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let ticket = self.generations.next();
        self.search_task = None;
        let (Listing::Loaded(scratches), Ok(store)) = (&self.listing, &self.target.store) else {
            return;
        };
        let stale = self.found.stale(scratches);
        if stale.is_empty() || self.query.is_empty() {
            return;
        }
        let (dir, query, applies) = (store.dir().to_path_buf(), self.query.clone(), ticket.clone());
        self.search_task = Some(cx.spawn(async move |section, cx| {
            cx.background_executor().timer(delay).await;
            let answers = cx.background_executor().spawn(async move { search::search(&dir, &stale, &query, &ticket) }).await;
            if let Some(answers) = answers {
                section.update(cx, |section, cx| section.searched(&applies, answers, cx)).ok();
            }
        }));
    }

    fn searched(&mut self, ticket: &Ticket, answers: Vec<Answer>, cx: &mut Context<Self>) {
        if ticket.is_stale() {
            return;
        }
        self.found.record(answers);
        cx.notify();
    }

    fn is_collapsed(&self, group: Group) -> bool {
        if self.is_searching() { self.collapsed_in_search.contains(&group) } else { self.collapsed.contains(&group) }
    }

    fn free_name(&self) -> String {
        let taken: Vec<&str> = match &self.listing {
            Listing::Loaded(scratches) => scratches.iter().map(|scratch| scratch.name.as_str()).collect(),
            _ => Vec::new(),
        };
        store::free_name(&taken)
    }

    fn start_inline(&mut self, mode: InlineMode, window: &mut Window, cx: &mut Context<Self>) {
        let name = match &mode {
            InlineMode::New => self.free_name(),
            InlineMode::Rename(old) => old.clone(),
        };
        let input = cx.new(TextInput::new);
        input.update(cx, |input, cx| {
            input.set_text(name.clone(), cx);
            input.select_range(0..store::stem_len(&name));
            input.set_target(NAME_TARGET);
        });
        window.focus(&input.focus_handle(cx), cx);
        let events = cx.subscribe_in(&input, window, |section, _input, event: &TextInputEvent, window, cx| match event {
            TextInputEvent::Changed(_) => section.clear_inline_error(cx),
            TextInputEvent::Submitted => section.submit_inline(window, cx),
        });
        self.inline = Some(Inline { mode, input, error: None, _events: events });
        cx.notify();
    }

    fn clear_inline_error(&mut self, cx: &mut Context<Self>) {
        if let Some(inline) = &mut self.inline {
            inline.error = None;
            cx.notify();
        }
    }

    fn cancel_inline(&mut self, cx: &mut Context<Self>) {
        self.inline = None;
        cx.notify();
    }

    fn submit_inline(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inline) = &self.inline else {
            return;
        };
        let name = inline.input.read(cx).text().trim().to_owned();
        let result = match &inline.mode {
            InlineMode::New => self.create(&name, window, cx),
            InlineMode::Rename(old) if *old == name => Ok(()),
            InlineMode::Rename(old) => self.rename(old, &name, cx),
        };
        match result {
            Ok(()) => {
                self.inline = None;
                self.reload(cx);
            }
            Err(error) => self.inline.iter_mut().for_each(|inline| inline.error = Some(error.clone())),
        }
        cx.notify();
    }

    fn scratch(&self, name: &str) -> ScratchRef {
        ScratchRef { project: self.target.project.clone(), name: name.to_owned() }
    }

    fn create(&self, name: &str, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        let scratch = self.scratch(name);
        self.target
            .workspace
            .update(cx, |workspace, cx| workspace.create_scratch(&scratch, window, cx).map(|_| ()))
            .unwrap_or_else(|_| Err(CLOSED.to_owned()))
    }

    fn rename(&self, from: &str, to: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let scratch = self.scratch(from);
        self.target
            .workspace
            .update(cx, |workspace, cx| workspace.rename_scratch(&scratch, to, cx))
            .unwrap_or_else(|_| Err(CLOSED.to_owned()))
    }

    fn open(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let scratch = self.scratch(name);
        let opened = self.target.workspace.update(cx, |workspace, cx| workspace.open_scratch(&scratch, window, cx));
        self.error = opened.unwrap_or_else(|_| Err(CLOSED.to_owned())).err();
        cx.notify();
    }

    fn confirm_delete(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let message = format!("Delete scratch `{name}`? It cannot be restored.");
        let answer = window.prompt(PromptLevel::Warning, &message, None, &["Delete", "Cancel"], cx);
        let scratch = self.scratch(&name);
        cx.spawn_in(window, async move |section, cx| {
            if matches!(answer.await, Ok(0)) {
                section.update_in(cx, |section, window, cx| section.delete(&scratch, window, cx)).ok();
            }
        })
        .detach();
    }

    fn delete(&mut self, scratch: &ScratchRef, window: &mut Window, cx: &mut Context<Self>) {
        let deleted = self.target.workspace.update(cx, |workspace, cx| workspace.delete_scratch(scratch, window, cx));
        self.error = deleted.unwrap_or_else(|_| Err(CLOSED.to_owned())).err();
        self.reload(cx);
    }

    fn toggle(&mut self, group: Group, cx: &mut Context<Self>) {
        let collapsed = if self.is_searching() { &mut self.collapsed_in_search } else { &mut self.collapsed };
        if collapsed.contains(&group) {
            collapsed.remove(&group);
        } else {
            collapsed.insert(group);
        }
        cx.notify();
    }

    fn show_more(&mut self, group: Group, cx: &mut Context<Self>) {
        self.paging.entry(group).or_default().more();
        cx.notify();
    }

    fn show_all(&mut self, group: Group, cx: &mut Context<Self>) {
        self.paging.entry(group).or_default().all();
        cx.notify();
    }

    /// Target `new-scratch`; the search input is target `scratch-search`.
    fn render_toolbar(&self, theme: Theme, cx: &mut Context<Self>) -> Div {
        let hint = match &self.target.store {
            Ok(store) => format!("Project scratches in {} · outside the repo, kept when worktrees are removed", tilde_home(store.dir())),
            Err(reason) => reason.clone(),
        };
        let new = cx.listener(|section, _: &ClickEvent, window, cx| section.start_inline(InlineMode::New, window, cx));
        let sizes = &theme.sizes.overview;
        let search = div().w(sizes.scratches_search_width.px()).child(self.search_input.clone());
        div()
            .flex()
            .flex_col()
            .gap(sizes.scratches_gap.px())
            .child(div().text_size(sizes.scratches_hint_text_size.px()).text_color(rgb(theme.muted)).child(selectable("scratches-hint", hint, cx)))
            .child(div().flex().items_center().gap(sizes.scratches_toolbar_gap.px()).child(search).child(link_button("new-scratch", "+ New scratch", theme).on_click(target("new-scratch".to_owned(), new, cx))))
    }

    /// The groups and their rows, or why there are none; a long listing scrolls itself and draws only the lines in
    /// view, as "Show all" may show thousands.
    fn render_listing(&self, frame: Frame, cx: &mut Context<Self>) -> AnyElement {
        let count = match self.lines(frame.clock) {
            Ok(lines) => lines.len(),
            Err(message) => return message.into_any_element(frame.theme),
        };
        let rows = uniform_list("scratch-lines", count, cx.processor(|section: &mut Self, range, _window, cx| section.render_lines(range, cx)))
            .track_scroll(&self.list.list_handle())
            .size_full();
        div().relative().flex_1().min_h_0().child(rows).child(vertical_scrollbar(&self.scrollbar, &crate::theme::view_style(&frame.theme))).into_any_element()
    }

    /// Scrolls the listing to the row of click target `name` (`scratch:`, `rename-scratch:` or `delete-scratch:`
    /// and a scratch's name), which only rows in view register; says whether it has that row.
    pub fn scroll_to_target(&self, name: &str) -> bool {
        let scratch = ["scratch:", "rename-scratch:", "delete-scratch:"].iter().find_map(|prefix| name.strip_prefix(prefix));
        let (Some(scratch), Ok(lines)) = (scratch, self.lines(Clock::system())) else {
            return false;
        };
        let Some(line) = lines.iter().position(|line| matches!(line, Line::Row(row) if row.name == scratch)) else {
            return false;
        };
        self.list.list_handle().scroll_to_item(line, ScrollStrategy::Center);
        true
    }

    /// The listing's lines: each group's header, then its shown rows and paging unless it is collapsed.
    fn lines(&self, clock: Clock) -> Result<Vec<Line<'_>>, NoLines> {
        let scratches = match &self.listing {
            Listing::Loading => return Err(NoLines::Muted("Reading scratches…".to_owned())),
            Listing::Failed(reason) => return Err(NoLines::Failed(reason.clone())),
            Listing::Loaded(scratches) => scratches,
        };
        let names = self.is_searching().then(|| self.found.names(scratches, &self.query));
        let visible = groups::visible(scratches, &self.query, names.as_deref());
        if visible.is_empty() {
            let message = if self.is_searching() { format!("No scratch matches “{}”", self.query) } else { "No scratches yet".to_owned() };
            return Err(NoLines::Muted(message));
        }
        let mut lines = Vec::new();
        for (group, members) in groups::grouped(visible, clock) {
            lines.push(Line::Header(group, members.len()));
            if self.is_collapsed(group) {
                continue;
            }
            let shown = self.paging.get(&group).copied().unwrap_or_default().visible(members.len());
            lines.extend(members[..shown].iter().map(|scratch| Line::Row(scratch)));
            if shown < members.len() {
                lines.push(Line::More { group, total: members.len(), shown });
            }
        }
        Ok(lines)
    }

    /// The lines in `range`, each a row high (Rename's input a little more).
    fn render_lines(&self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let frame = Frame { clock: Clock::system(), theme: *cx.global::<Theme>() };
        let Ok(lines) = self.lines(frame.clock) else {
            return Vec::new();
        };
        let height = frame.theme.sizes.overview.scratches_row_height.px();
        let lines = &lines[range.start.min(lines.len())..range.end.min(lines.len())];
        lines
            .iter()
            .enumerate()
            .map(|(offset, line)| {
                let at = range.start + offset;
                let drawn = match line {
                    Line::Header(group, count) => self.render_group_header(*group, *count, cx),
                    Line::Row(scratch) => self.render_row(scratch, frame, cx),
                    Line::More { group, total, shown } => self.render_more(*group, *total, *shown, cx).unwrap_or_else(|| div().id(("scratch-more-none", at))),
                };
                drawn.flex_none().w_full().h(height).into_any_element()
            })
            .collect()
    }

    /// "Show 10 more · N of M" and "Show all" (targets `scratch-more:<group>`, `scratch-all:<group>`) while rows
    /// are hidden, with "or refine the search" while searching.
    fn render_more(&self, group: Group, total: usize, shown: usize, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        paging::more_label(total, shown)?;
        let theme = crate::theme::tabs(cx);
        let on_more = target(format!("scratch-more:{}", group.title()), cx.listener(move |section, _: &ClickEvent, _window, cx| section.show_more(group, cx)), cx);
        let on_all = target(format!("scratch-all:{}", group.title()), cx.listener(move |section, _: &ClickEvent, _window, cx| section.show_all(group, cx)), cx);
        let refine = self.is_searching().then(|| div().text_color(rgb(theme.muted)).child("· or refine the search"));
        let id = SharedString::from(format!("scratch-more-{}", group.title()));
        paging::more_row(id, total, shown, theme, on_more, on_all).map(|row| row.children(refine))
    }

    /// Target `scratch-group:<group>`: expands or collapses it.
    fn render_group_header(&self, group: Group, count: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = crate::theme::tabs(cx);
        let chevron = if self.is_collapsed(group) { icons::CHEVRON_RIGHT } else { icons::CHEVRON_DOWN };
        let toggle = cx.listener(move |section, _: &ClickEvent, _window, cx| section.toggle(group, cx));
        let sizes = &theme.sizes.overview;
        div()
            .id(SharedString::from(format!("scratch-group-{}", group.title())))
            .flex()
            .items_center()
            .gap(sizes.scratches_group_gap.px())
            .h(sizes.scratches_row_height.px())
            .cursor_pointer()
            .font_weight(sizes.scratches_group_weight.font_weight())
            .child(img(icons::embedded(theme.appearance, chevron)).size(theme.sizes.icon.px()))
            .child(group.title())
            .child(div().font_weight(sizes.scratches_count_weight.font_weight()).text_color(rgb(theme.muted)).child(count.to_string()))
            .on_click(target(format!("scratch-group:{}", group.title()), toggle, cx))
    }

    /// Target `scratch:<name>`: opens it.
    fn render_row(&self, scratch: &ScratchInfo, frame: Frame, cx: &mut Context<Self>) -> Stateful<Div> {
        if let Some(inline) = self.inline.as_ref().filter(|inline| inline.renames(&scratch.name)) {
            return render_name_input(inline, frame.theme, cx);
        }
        let hover_group = SharedString::from(format!("scratch-row:{}", scratch.name));
        let name = scratch.name.clone();
        let open = cx.listener(move |section, _: &ClickEvent, window, cx| section.open(&name, window, cx));
        let muted_color = rgb(frame.theme.muted);
        let sizes = &frame.theme.sizes.overview;
        div()
            .id(SharedString::from(format!("scratch:{}", scratch.name)))
            .group(hover_group.clone())
            .flex()
            .items_center()
            .gap(sizes.scratches_gap.px())
            .h(sizes.scratches_row_height.px())
            .px(sizes.scratches_row_padding_x.px())
            .rounded(sizes.scratches_row_radius.px())
            .cursor_pointer()
            .hover(|style| style.bg(rgb(frame.theme.hover)))
            .child(scratch_icon(frame.theme, &scratch.name))
            .child(div().flex_none().child(scratch.name.clone()))
            .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().text_color(muted_color).child(scratch.first_line.clone()))
            .child(div().flex_none().text_color(muted_color).child(frame.clock.age(unix_seconds(scratch.modified))))
            .child(self.render_row_actions(&scratch.name, hover_group, frame.theme, cx))
            .on_click(target(format!("scratch:{}", scratch.name), open, cx))
    }

    /// Targets `rename-scratch:<name>` and `delete-scratch:<name>`; drawn (and registered) while hidden.
    fn render_row_actions(&self, name: &str, hover_group: SharedString, theme: Theme, cx: &mut Context<Self>) -> Div {
        let (rename, delete) = (name.to_owned(), name.to_owned());
        let on_rename = cx.listener(move |section, _: &ClickEvent, window, cx| {
            cx.stop_propagation();
            section.start_inline(InlineMode::Rename(rename.clone()), window, cx);
        });
        let on_delete = cx.listener(move |section, _: &ClickEvent, window, cx| {
            cx.stop_propagation();
            section.confirm_delete(delete.clone(), window, cx);
        });
        div()
            .flex()
            .flex_none()
            .gap(theme.sizes.overview.scratches_gap.px())
            .invisible()
            .group_hover(hover_group, |style| style.visible())
            .child(link_button(format!("rename:{name}"), "Rename", theme).on_click(target(format!("rename-scratch:{name}"), on_rename, cx)))
            .child(link_button(format!("delete:{name}"), "Delete", theme).on_click(target(format!("delete-scratch:{name}"), on_delete, cx)))
    }
}

impl Render for ScratchesSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame = Frame { clock: Clock::system(), theme: *cx.global::<Theme>() };
        let new_row = self.inline.as_ref().filter(|inline| matches!(inline.mode, InlineMode::New)).map(|inline| render_name_input(inline, frame.theme, cx));
        let sizes = &frame.theme.sizes.overview;
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .gap(sizes.scratches_gap.px())
            .text_size(sizes.scratches_text_size.px())
            .child(self.render_toolbar(frame.theme, cx))
            .children(new_row)
            .children(self.error.clone().map(|error| div().text_color(rgb(frame.theme.error)).child(error)))
            .child(self.render_listing(frame, cx))
    }
}

/// Escape cancels; the text input lets it through.
fn render_name_input(inline: &Inline, theme: Theme, cx: &mut Context<ScratchesSection>) -> Stateful<Div> {
    let sizes = &theme.sizes.overview;
    div()
        .id("scratch-name")
        .flex()
        .items_center()
        .gap(sizes.scratches_gap.px())
        .h(sizes.scratches_input_row_height.px())
        .px(sizes.scratches_row_padding_x.px())
        .on_key_down(cx.listener(|section, event: &KeyDownEvent, _window, cx| {
            if event.keystroke.key == "escape" {
                section.cancel_inline(cx);
            }
        }))
        .child(div().w(sizes.scratches_name_input_width.px()).child(inline.input.clone()))
        .children(inline.error.clone().map(|error| div().text_size(sizes.scratches_error_text_size.px()).text_color(rgb(theme.error)).child(error)))
}

fn link_button(id: impl Into<SharedString>, label: &'static str, theme: Theme) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex_none()
        .px(theme.sizes.overview.link_padding_x.px())
        .rounded(theme.sizes.overview.link_radius.px())
        .cursor_pointer()
        .text_color(rgb(theme.accent))
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(label)
}

fn muted(text: impl Into<SharedString>, theme: Theme) -> AnyElement {
    div().text_color(rgb(theme.muted)).child(text.into()).into_any_element()
}
