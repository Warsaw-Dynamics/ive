//! Draws the sidebar: header with the Open project menu, the tree rows, the resize handle.

use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    App, ClickEvent, Context, CursorStyle, Div, DragMoveEvent, ElementId, FocusHandle, ListHorizontalSizingBehavior, MouseButton, MouseDownEvent,
    SharedString, Stateful, StyleRefinement, WeakEntity, Window, WindowControlArea, deferred, div, prelude::*, px, rgb, rgba, uniform_list,
};

mod folder_view;
mod header_menus;

use super::rows::{
    CheckoutRow, EntryRow, MoreRow, ProcessRow, ProjectRow, ProjectTag, RunBadge, RunRow, TreeItem, TreeKey, TreeRow,
    initials, target_name,
};
use super::drop_place::DropHint;
use super::row_icons::{FolderIcon, RowMark, folder_icon, row_mark};
use super::selection::{Listed, PickGroup, Selection, pick_group};
use crate::attention_dot::with_dot;
use crate::dragged_files::{DraggedFiles, DraggedProcesses};
use crate::favicon_badge::{Badge, BadgeImage, badged, favicon_or};
use crate::file_tree::{EntryKind, HIDDEN_GROUP};
use super::states::{EntryState, Highlight, row_look};
use crate::click_targets::{Listener, target};
use crate::icons;
use crate::mouse_binding;
use crate::script::keymap::{Action, Builtin, Scope};
use crate::script::state::{bound_mouse, invoke};
use crate::paging::more_row;
use crate::progress_bar::progress_bar;
use crate::scrollbar::{ListScroll, ScrollState, horizontal_scrollbar, vertical_scrollbar, widest_row};
use crate::process::RunId;
use crate::run::values_tab::ShowRun;
use crate::state::RecentProject;
use crate::theme::Theme;
use crate::theme_config::RowStyle;
use crate::tooltip;
use crate::window_controls::TRAFFIC_LIGHTS_WIDTH;
use crate::window_drag::WindowDrag;
use crate::workspace::{Dragging, ProgramEnding, Workspace, icon};

/// One chevron slot: a child's first icon lines up under its parent's icon.
pub(crate) fn tree_indent(theme: Theme) -> f32 {
    theme.sizes.icon.0 + theme.sizes.tree.icon_gap.0
}

/// The tree's rows (rebuilt by the workspace whenever what they show changes), the selected one, the active tab's,
/// and scrolling.
pub struct Tree {
    pub rows: Vec<TreeRow>,
    pub selection: Selection,
    pub active: Option<TreeKey>,
    pub list: ListScroll,
    pub scroll: ScrollState,
    /// what a drop on the row under the pointer would do, while something is dragged over the tree
    pub drop_hint: Option<DropHint>,
    /// the line under the tree saying how a copy or move goes
    pub note: Option<TreeNote>,
}

/// The line under the tree: what a copy or move did, until `until` (`None`: until clicked).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNote {
    pub text: String,
    pub until: Option<std::time::Instant>,
}

impl Tree {
    pub fn new() -> Self {
        let list = ListScroll::new();
        let scroll = ScrollState::for_list(&list);
        Self { rows: Vec::new(), selection: Selection::default(), active: None, list, scroll, drop_hint: None, note: None }
    }

    /// Marks `shown`, the shown tab's row; the selection stays, as in IntelliJ, until the tree's own clicks change it.
    pub fn show_active(&mut self, shown: Option<TreeKey>) {
        self.active = shown;
    }

    /// Whether the row of `key` is drawn as where the drag would drop; a hint left from a drag that ended elsewhere
    /// shows nothing.
    pub fn is_drop_row(&self, key: &TreeKey, cx: &App) -> bool {
        cx.has_active_drag() && self.drop_hint.as_ref().is_some_and(|hint| hint.allowed && hint.row == *key)
    }

    /// The hint of the drag going on, if it is over a row.
    pub fn shown_hint(&self, cx: &App) -> Option<&DropHint> {
        self.drop_hint.as_ref().filter(|_| cx.has_active_drag())
    }
}

pub use header_menus::{FoldersMenu, PickedProject, PickerTitle, ProjectsMenu, menu_keys, picker_title};

/// The header's menus; one is open at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMenu {
    OpenProject,
    Folders,
    Projects,
    /// the tab row's bell: the processes that wait for the user
    Waiting,
}

/// What the sidebar draws besides the tree.
pub struct Sidebar<'a> {
    pub tree: &'a Tree,
    pub recent: &'a [RecentProject],
    pub menu: Option<HeaderMenu>,
    /// while the folders menu is open
    pub folders: Option<FoldersMenu>,
    /// the title's picker
    pub picker: PickerTitle,
    /// while the picker's menu is open
    pub projects: Option<ProjectsMenu>,
    pub drag: &'a WindowDrag,
    pub width: f32,
    pub resizing: bool,
    /// paths the OS refused to watch, one line each
    pub watch_failures: Vec<String>,
}

pub fn render(sidebar: Sidebar<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let tree = sidebar.tree;
    // proportional font, so this only approximates the widest visible row: an icon slot counts as 3 characters
    let widest = widest_row(tree.list.visible_rows(tree.rows.len()), |index| row_width(&tree.rows[index]));
    // only the visible rows are built, so deep or wide trees stay cheap to scroll
    let rows = uniform_list(
        "project-tree",
        tree.rows.len(),
        cx.processor(|workspace: &mut Workspace, range, window, cx| {
            let theme = crate::theme::for_panel(crate::theme::Panel::Sidebar, window, cx);
            render_rows(workspace, range, theme, cx)
        }),
    )
    .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
    .with_width_from_item(widest)
    .track_scroll(&tree.list.list_handle())
    .size_full();
    let sidebar_focus = cx.global::<crate::theme::SidebarFocus>().0.clone();
    let body = div()
        .relative()
        .flex_1()
        .min_h_0()
        .child(div().size_full().capture_any_mouse_down(take_keys(sidebar_focus.clone())).child(rows))
        .child(vertical_scrollbar(&tree.scroll, &crate::theme::view_style(&theme)))
        .child(horizontal_scrollbar(&tree.scroll, &crate::theme::view_style(&theme)))
        .children(tree.note.as_ref().map(|note| render_note(note, theme, cx)));
    let header = render_header(&sidebar, theme, cx).capture_any_mouse_down(take_keys(sidebar_focus.clone()));
    let panel = div().relative().flex().flex_col().flex_none().w(px(sidebar.width)).h_full();
    let panel = match theme.panels.shown() {
        // the header sits on the frame; the tree below it is the panel
        true => panel.child(header).child(body.bg(rgb(theme.tree)).rounded_b(theme.panels.radius.px()).overflow_hidden()),
        false => panel
            .bg(rgb(theme.tree))
            .border_r(theme.sizes.tree.border.px())
            .border_color(rgb(if sidebar.resizing { theme.accent } else { theme.border }))
            .child(header)
            .child(body),
    };
    panel.track_focus(&sidebar_focus).child(render_resize_handle(theme, cx))
}

/// A left press on the header or the rows gives the sidebar the keyboard; captured, as rows stop their presses. A
/// field that takes the keys afterwards keeps them; the scrollbars and the resize handle leave them where they are.
fn take_keys(sidebar_focus: FocusHandle) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    move |event, window, cx| {
        if event.button == MouseButton::Left && !sidebar_focus.contains_focused(window, cx) {
            window.focus(&sidebar_focus, cx);
        }
    }
}

fn row_width(row: &TreeRow) -> usize {
    let (slots, text) = match &row.item {
        TreeItem::Project(project) => (2, project.name.chars().count() + 12 + said_width(&waiting_text(project.waiting))),
        TreeItem::Checkout(checkout) => (2, checkout.label.chars().count() + said_width(&waiting_text(checkout.waiting))),
        TreeItem::Folder(folder) => {
            let nothing = folder.nothing.as_ref().map_or(0, |nothing| nothing.mark.chars().count());
            (2, folder.kind.title().chars().count() + 4 + nothing + said_width(&waiting_text(folder.waiting)))
        }
        TreeItem::Process(process) => (2, process.title.chars().count() + said_width(&process.attention.as_ref().map(|text| format!("● {text}")))),
        TreeItem::LiveRun(run) => (2, run.name.chars().count() + run.label.chars().count() + 1),
        TreeItem::Change(change) => (2, change.change.path.file_name().chars().count() + 2 + change.folder.as_ref().map_or(0, |folder| folder.chars().count() + 1)),
        TreeItem::ChangeFolder(folder) => (2, folder.label.chars().count() + 4),
        TreeItem::Commit(commit) => (2, commit.label.chars().count() + commit.meta.chars().count() + 1),
        TreeItem::CommitsMore(more) => (1, more.text.chars().count()),
        TreeItem::Note(note) => (1, note.text.chars().count()),
        TreeItem::Entry(row) => (2, row.entry.name.chars().count() + 4),
        TreeItem::More(_) => (0, 30),
    };
    (row.depth + slots) * 3 + text
}

/// The project picker (All projects, or the one shown), then the folders menu, Select Opened File and the Open
/// project button at the right; empty space moves the window, as
/// the tab row's does.
fn render_header(sidebar: &Sidebar<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let locate = header_button("select-opened", icons::LOCATE, theme)
        .tooltip(tooltip::text_tooltip("Select opened file"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("select-opened".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.select_opened(cx)), cx));
    let open = header_button_press(header_button("open-project", icons::ADD, theme), cx)
        .tooltip(tooltip::text_tooltip("Open project"))
        .on_click(target("open-project".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.toggle_menu(HeaderMenu::OpenProject, window, cx)), cx));
    let folders = header_button_press(header_button("folders-menu", icons::PROJECT_FOLDER, theme), cx)
        .tooltip(tooltip::text_tooltip("Folders in every worktree"))
        .on_click(target("folders-menu".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.toggle_menu(HeaderMenu::Folders, window, cx)), cx));
    let folders_menu = sidebar.folders.as_ref().map(|menu| deferred(header_menus::folders_menu(menu, theme, cx)).with_priority(1));
    let projects_menu = sidebar.projects.as_ref().map(|menu| deferred(header_menus::projects_menu(menu, theme, cx)).with_priority(1));
    sidebar
        .drag
        .apply(div().id("sidebar-header"))
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_between()
        .h(sizes.header_height.px())
        .pr(sizes.header_padding_right.px())
        .map(|header| if cfg!(target_os = "macos") { header.pl(px(TRAFFIC_LIGHTS_WIDTH)) } else { header.pl(sizes.header_padding_left.px()) })
        .border_b(sizes.border.px())
        .border_color(rgb(theme.border))
        .window_control_area(WindowControlArea::Drag)
        .child(header_menus::picker_button(&sidebar.picker, theme, cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap(sizes.header_tools_gap.px())
                .children(watch_warning(&sidebar.watch_failures, theme))
                .child(folders)
                .child(locate)
                .child(open),
        )
        .when(sidebar.menu == Some(HeaderMenu::OpenProject), |header| header.child(deferred(render_menu(sidebar.recent, theme, cx)).with_priority(1)))
        .children(folders_menu)
        .children(projects_menu)
}

/// A header button's press: noted before the open menu's press-outside closes it (the capture phase comes first),
/// and kept from moving the window.
pub(crate) fn header_button_press(button: Stateful<Div>, cx: &mut Context<Workspace>) -> Stateful<Div> {
    button
        .capture_any_mouse_down(cx.listener(|workspace, _event, _window, _cx| workspace.note_menu_press()))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
}

fn header_button(id: &'static str, name: &str, theme: Theme) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    div()
        .id(id)
        .p(sizes.header_button_padding.px())
        .rounded(sizes.header_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(crate::workspace::mask_icon(theme, name))
}

fn watch_warning(failures: &[String], theme: Theme) -> Option<Stateful<Div>> {
    let text = failures.join("\n");
    failures.first().map(|_| div().id("watch-warning").flex_none().child(icon(theme, icons::WARNING)).tooltip(tooltip::text_tooltip(text)))
}

/// Up to 10 recent projects, newest first, then Open Folder…; a press outside closes it.
fn render_menu(recent: &[RecentProject], theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let items = recent.iter().enumerate().map(|(index, project)| {
        let path = project.path.clone();
        let name = crate::project::folder_name(&project.path);
        let open = target(format!("recent:{}", project.id), cx.listener(move |workspace, _event, window, cx| workspace.open_recent(path.clone(), window, cx)), cx);
        menu_item(("recent-project", index), theme)
            .child(badge(&initials(&name), theme.hover, theme.text, theme))
            .child(div().flex_1().overflow_hidden().text_ellipsis().child(name))
            .tooltip(tooltip::text_tooltip(project.path.display().to_string()))
            .on_click(open)
    });
    let section = (!recent.is_empty()).then(|| menu_section("Recent", theme));
    let menu = menu_panel("open-project-menu", theme)
        .right(sizes.menu_offset.px())
        .w(sizes.menu_width.px())
        .children(section)
        .children(items)
        .child(
            menu_item("open-folder", theme)
                .child(icon(theme, icons::PROJECT_FOLDER))
                .child("Open Folder…")
                .on_click(target("open-folder".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.prompt_for_folder(window, cx)), cx)),
        );
    closing_outside(menu, cx)
}

/// A header menu's box under the header, before its side, width and [`closing_outside`] are set.
pub(crate) fn menu_panel(id: &'static str, theme: Theme) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    div()
        .id(id)
        .absolute()
        .top(px(sizes.header_height.0 + sizes.menu_offset.0))
        .p(sizes.menu_padding.px())
        .rounded(sizes.menu_radius.px())
        .border(sizes.border.px())
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .shadow_lg()
        .occlude()
        .text_size(sizes.menu_text_size.px())
        .font_weight(sizes.menu_weight.font_weight())
}

/// A header menu that a press outside closes.
pub(crate) fn closing_outside(menu: Stateful<Div>, cx: &mut Context<Workspace>) -> Stateful<Div> {
    menu.on_mouse_down_out(cx.listener(|workspace, _event, window, cx| workspace.close_menu(window, cx)))
}

pub(crate) fn menu_section(title: &'static str, theme: Theme) -> Div {
    let sizes = &theme.sizes.tree;
    div()
        .px(sizes.menu_section_padding_x.px())
        .pt(sizes.menu_section_padding_top.px())
        .pb(sizes.menu_section_padding_bottom.px())
        .text_size(sizes.menu_section_text_size.px())
        .text_color(rgb(theme.muted))
        .child(title)
}

pub(crate) fn menu_item(id: impl Into<ElementId>, theme: Theme) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(sizes.menu_item_gap.px())
        .h(sizes.menu_item_height.px())
        .px(sizes.menu_item_padding_x.px())
        .rounded(sizes.menu_item_radius.px())
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
}

/// The initials badge of a project, as today's title bar drew it.
pub fn badge(text: &str, background: u32, color: u32, theme: Theme) -> Div {
    let sizes = &theme.sizes.tree;
    div()
        .flex()
        .flex_none()
        .justify_center()
        .items_center()
        .size(sizes.badge_size.px())
        .rounded(sizes.badge_radius.px())
        .bg(rgb(background))
        .text_color(rgb(color))
        .text_size(sizes.badge_text_size.px())
        .font_weight(sizes.badge_weight.font_weight())
        .child(text.to_owned())
}

fn render_rows(workspace: &Workspace, range: Range<usize>, theme: Theme, cx: &mut Context<Workspace>) -> Vec<Stateful<Div>> {
    range.map(|index| render_row(workspace, index, theme, cx)).collect()
}

/// Row `index` of the tree, drawn by its state, git mark, selection and whether it is the active tab's.
fn render_row(workspace: &Workspace, index: usize, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let tree = workspace.tree();
    let row = &tree.rows[index];
    let key = row.key();
    let entry = match &row.item {
        TreeItem::Entry(entry) => Some((entry.state, entry.mark)),
        _ => None,
    };
    let highlight = Highlight { entry, selected: tree.selection.contains(&key), active: tree.active.as_ref() == Some(&key), drop: tree.is_drop_row(&key, cx) };
    let dragged = dragged_processes(row, &key, cx);
    let drop_key = key.clone();
    let select = RowSelect { key, pick: pick_group(row), workspace: cx.entity().downgrade() };
    let line = RowLine { mark: row_mark(row, workspace, theme), ..RowLine::new(row, index, theme, row_look(highlight, &theme.tree_states, &theme.git), select) };
    let drawn = match &row.item {
        TreeItem::Project(project) => project_row(line, project, cx),
        TreeItem::Checkout(checkout) => checkout_row(line, checkout, cx),
        TreeItem::Folder(folder) => folder_view::folder_row(line, folder, cx),
        TreeItem::Process(process) => process_row(line, process, cx),
        TreeItem::LiveRun(run) => folder_view::live_run_row(line, run, cx),
        TreeItem::Change(change) => folder_view::change_row(line, change, cx),
        TreeItem::ChangeFolder(folder) => folder_view::change_folder_row(line, folder, cx),
        TreeItem::Commit(commit) => folder_view::commit_row(line, commit, cx),
        TreeItem::CommitsMore(more) => folder_view::commits_more_row(line, more, cx),
        TreeItem::Note(note) => folder_view::note_row(line, note),
        TreeItem::Entry(entry) => entry_row(line, entry, cx),
        TreeItem::More(more) => more_line(line, more, cx),
    };
    let drawn = match dragged {
        Some(dragged) => drawn.on_drag(dragged, |dragged, _offset, _window, cx| cx.new(|cx| dragged.chip(cx))),
        None => drawn,
    };
    drop_target(drawn, drop_key, cx)
}

/// What dragging a process or run row carries; `None` for other rows, and for a run still waiting for its `before`
/// steps, which has no process yet.
fn dragged_processes(row: &TreeRow, key: &TreeKey, cx: &Context<Workspace>) -> Option<DraggedProcesses> {
    let title = match &row.item {
        TreeItem::Process(process) => process.title.clone(),
        TreeItem::LiveRun(run) if matches!(run.id, RunId::Process(_)) => run.name.clone(),
        _ => return None,
    };
    Some(DraggedProcesses { pressed: key.clone(), title, workspace: cx.entity().downgrade() })
}

/// A row that tells the workspace what a drag over it would do, and takes the drop.
fn drop_target(row: Stateful<Div>, key: TreeKey, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let (hovered, dropped, files_hovered, files_dropped) = (key.clone(), key.clone(), key.clone(), key);
    row.on_drag_move(cx.listener(move |workspace, event: &DragMoveEvent<DraggedProcesses>, window, cx| {
        if event.bounds.contains(&event.event.position) {
            let pressed = event.drag(cx).pressed.clone();
            workspace.hover_drop(Dragging::Processes(pressed), hovered.clone(), window, cx);
        }
    }))
    .on_drag_move(cx.listener(move |workspace, event: &DragMoveEvent<DraggedFiles>, window, cx| {
        if event.bounds.contains(&event.event.position) {
            let pressed = event.drag(cx).pressed.clone();
            workspace.hover_drop(Dragging::Files(pressed), files_hovered.clone(), window, cx);
        }
    }))
    .on_drop(cx.listener(move |workspace, dragged: &DraggedProcesses, window, cx| {
        workspace.drop_processes((&dragged.pressed, dropped.clone()), &window.modifiers(), window, cx);
    }))
    .on_drop(cx.listener(move |workspace, dragged: &DraggedFiles, window, cx| {
        workspace.drop_files((&dragged.pressed, files_dropped.clone()), &window.modifiers(), window, cx);
    }))
}

/// The line under the tree; a click on it lets it go.
fn render_note(note: &TreeNote, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let dismiss = target("tree-note".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.dismiss_note(cx)), cx);
    let sizes = &theme.sizes.tree;
    div()
        .id("tree-note")
        .absolute()
        .left(sizes.padding_left.px())
        .right(sizes.padding_right.px())
        .bottom(sizes.padding_right.px())
        .px(sizes.padding_left.px())
        .py(sizes.icon_gap.px())
        .rounded(theme.sizes.tabs.tab_radius.px())
        .border(theme.sizes.tree.border.px())
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .text_size(sizes.text_size.px())
        .text_color(rgb(theme.text))
        .cursor_pointer()
        .child(note.text.clone())
        .on_click(dismiss)
}

/// `element` drawn with what `style` sets.
fn styled<E: Styled>(element: E, style: RowStyle) -> E {
    let element = match style.background {
        Some(background) => element.bg(rgba(background.0)),
        None => element,
    };
    let element = match style.text {
        Some(text) => element.text_color(rgb(text.0)),
        None => element,
    };
    let element = match style.weight {
        Some(weight) => element.font_weight(weight.font_weight()),
        None => element,
    };
    match style.italic {
        Some(true) => element.italic(),
        Some(false) | None => element,
    }
}

/// A highlighted row's background as a rounded box inset from the panel's sides, which the hover background
/// fills while the mouse is over the row.
fn highlight_box(look: RowStyle, group: SharedString, theme: Theme) -> Div {
    let sizes = &theme.sizes.tree;
    let inset = sizes.selection_inset.px();
    let hover = RowStyle { text: None, weight: None, ..theme.tree_states.hover };
    let shape = div().absolute().top_0().bottom_0().left(inset).right(inset).rounded(sizes.selection_radius.px());
    styled(shape, RowStyle { text: None, weight: None, ..look }).group_hover(group, move |style| hovered(style, hover))
}

/// The hover style of a row: what the theme's `hover` row style sets.
pub(crate) fn hovered(style: StyleRefinement, hover: RowStyle) -> StyleRefinement {
    styled(style, hover)
}

/// The grey of extra text such as a project's tags: the theme's `secondary` row style, else the muted colour.
pub(crate) fn secondary(theme: Theme) -> u32 {
    theme.tree_states.secondary.text.map_or(theme.muted, |text| text.0)
}

/// The line of a tree row before its content, the hover group its row actions show with, and what its click
/// targets are named after.
struct RowLine {
    div: Stateful<Div>,
    group: SharedString,
    index: usize,
    theme: Theme,
    target: String,
    select: RowSelect,
    /// its project's or module's favicon, a worktree's rail
    mark: RowMark,
    /// its content's distance from the row's left edge, by its depth
    indent: f32,
}

impl RowLine {
    /// The row's line drawn with `look`: its background fills the row, or sits in a rounded box inset from the
    /// panel's sides when the theme asks for one, behind the row's content.
    fn new(row: &TreeRow, index: usize, theme: Theme, look: RowStyle, select: RowSelect) -> Self {
        let sizes = &theme.sizes.tree;
        let group = SharedString::from(format!("tree-row-{index}"));
        let indent = sizes.padding_left.0 + row.depth as f32 * tree_indent(theme);
        let div = div()
            .id(("tree-row", index))
            .group(group.clone())
            // uniform_list lays a row out as wide as the viewport plus the scroll, so it reaches the visible right
            // edge however far the tree is scrolled: the highlight spans it and row actions pin to it
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap(sizes.icon_gap.px())
            .h(sizes.row_height.px())
            .pl(px(indent))
            .pr(sizes.padding_right.px())
            .text_size(sizes.text_size.px())
            .whitespace_nowrap()
            .cursor_pointer();
        let hover = theme.tree_states.hover;
        let div = match sizes.flat_selection() {
            true => styled(div.hover(move |style| hovered(style, hover)), look),
            false => {
                let foreground = RowStyle { background: None, ..look };
                let row_hover = RowStyle { background: None, ..hover };
                styled(div.hover(move |style| hovered(style, row_hover)), foreground).child(highlight_box(look, group.clone(), theme))
            }
        };
        Self { div, group, index, theme, target: row.target.clone(), select, mark: RowMark::default(), indent }
    }

    /// Click target `<verb>:<row's target>`, as `TreeRow::target_names` lists it.
    fn name(&self, verb: &str) -> String {
        target_name(verb, &self.target)
    }
}

/// A worktree's colour, as a bar left of its row's chevron.
fn worktree_rail(colour: u32, indent: f32, theme: Theme) -> Div {
    let sizes = &theme.sizes.tabs;
    div().absolute().left(px(indent - 5.)).top(sizes.rail_inset.px()).bottom(sizes.rail_inset.px()).w(sizes.rail_width.px()).rounded(px(1.)).bg(rgb(colour))
}

/// A folder row's icon: its module's favicon, else `embedded`.
fn folder_mark(embedded: &'static str, mark: &RowMark, theme: Theme) -> gpui::AnyElement {
    match folder_icon(embedded, mark.favicon.clone()) {
        FolderIcon::Favicon(path) => favicon_or(&path, theme.sizes.icon.px(), move || icon(theme, embedded).into_any_element()).into_any_element(),
        FolderIcon::Embedded(name) => icon(theme, name).into_any_element(),
    }
}

/// A worktree row's branch icon; the main checkout's carries the star.
fn branch_icon(mark: &RowMark, theme: Theme) -> Div {
    let star = mark.star.then_some(Badge { image: BadgeImage::Embedded(icons::STAR), ring: theme.tree });
    badged(icon(theme, icons::BRANCH), star, theme)
}

fn checkout_row(line: RowLine, row: &CheckoutRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let toggled = row.checkout.clone();
    let toggle: Listener = Rc::new(target(line.name("toggle"), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_checkout(&toggled, cx)), cx));
    let checkout = Some(row.checkout.path.clone());
    let mouse = RowMouse { select: line.select.clone(), name: line.name("toggle"), checkout, activate: toggle.clone(), keep: None, toggle: Some(toggle.clone()) };
    let hide = (!row.is_main).then(|| {
        let hidden = row.checkout.clone();
        let on_click = target(line.name("hide-worktree"), cx.listener(move |workspace, _event, _window, cx| workspace.hide_worktree(&hidden, cx)), cx);
        let action = RowAction { id: "hide-worktree", icon: icons::HIDE, tip: "Hide worktree (the project page shows it again)".to_owned(), on_click: Box::new(on_click) };
        row_action(action, &line)
    });
    let rail = line.mark.rail.map(|colour| worktree_rail(colour, line.indent, theme));
    let line = line
        .div
        .children(rail)
        .child(chevron(row.expanded, theme, shared(&toggle)))
        .child(branch_icon(&line.mark, theme))
        .child(row.label.clone())
        .children(waiting_tag(row.waiting, theme))
        .children(hide);
    bind_row(line, mouse, cx)
}

fn process_row(line: RowLine, row: &ProcessRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let id = row.id;
    // Stop ends any process for good, as Ctrl+Shift+Q does
    let stop = cx.listener(move |workspace, _event, window, cx| workspace.stop_process(id, ProgramEnding::Stop, window, cx));
    let action = RowAction { id: "stop-process", icon: icons::STOP, tip: "Stop".to_owned(), on_click: Box::new(target(line.name("stop-process"), stop, cx)) };
    let action = row_action(action, &line);
    let show: Listener = Rc::new(target(line.name("process"), cx.listener(move |workspace, _event, window, cx| workspace.show_process(id, window, cx)), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("process"), checkout: None, activate: show, keep: None, toggle: None };
    let theme = line.theme;
    let kind_icon = match &row.attention {
        Some(_) => with_dot(icon(theme, row.kind.icon()), theme, theme.tree).into_any_element(),
        None => icon(theme, row.kind.icon()).into_any_element(),
    };
    let said = row.attention.as_ref().map(|text| attention_text(format!("● {text}"), theme));
    let row_div = line.div.child(folder_view::slot(theme)).child(kind_icon).children(progress_bar(row.progress, theme)).child(row.title.clone()).children(said).child(action);
    bind_row(row_div, mouse, cx)
}

/// What a process said when it called for the user, after its title, in the attention colour.
fn attention_text(text: String, theme: Theme) -> Div {
    div().flex_none().text_size(theme.sizes.tree.tag_text_size.px()).text_color(rgb(theme.attention)).child(text)
}

/// `● n waiting` on a row with processes below it that called for the user and that its open rows do not show.
pub(super) fn waiting_tag(waiting: usize, theme: Theme) -> Option<Div> {
    waiting_text(waiting).map(|text| attention_text(text, theme))
}

fn waiting_text(waiting: usize) -> Option<String> {
    (waiting > 0).then(|| format!("● {waiting} waiting"))
}

/// The characters a row's attention text adds, with the gap before it, so the tree scrolls far enough to show it.
fn said_width(text: &Option<String>) -> usize {
    text.as_ref().map_or(0, |text| text.chars().count() + 1)
}

/// A file or folder row; dragged onto a terminal, it types its path, or the selection's when it is selected.
fn entry_row(line: RowLine, row: &EntryRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let entry = &row.entry;
    let file_icon = match (entry.expands(), row.run_file, row.state) {
        _ if entry.kind == EntryKind::HiddenGroup => icons::FOLDER_EXCLUDED,
        (true, true, _) => icons::FOLDER_RUN,
        (true, false, EntryState::Excluded) => icons::FOLDER_EXCLUDED,
        (true, false, _) => icons::FOLDER,
        (false, _, _) => icons::file_icon(&entry.name),
    };
    let chevron_icon = if row.expanded { icons::CHEVRON_DOWN } else { icons::CHEVRON_RIGHT };
    let (checkout, activated) = (row.checkout.clone(), entry.clone());
    let toggled: Listener = Rc::new(target(line.name("entry"), cx.listener(move |workspace, _event, window, cx| {
        workspace.activate_entry(&checkout, &activated, window, cx);
    }), cx));
    let toggle = entry.expands().then(|| toggled.clone());
    let mouse = RowMouse { select: line.select.clone(), name: line.name("entry"), checkout: Some(row.checkout.path.clone()), activate: toggled, keep: None, toggle };
    // a file keeps the chevron's slot, so its icon lines up with its sibling folders'
    let disclosure = match entry.expands() {
        true => icon(theme, chevron_icon).into_any_element(),
        false => div().flex_none().size(theme.sizes.icon.px()).into_any_element(),
    };
    let warning = row.warning.clone().map(|error| div().id("run-file-warning").flex_none().child(icon(theme, icons::WARNING)).tooltip(tooltip::text_tooltip(error)));
    let pickable = line.select.pick == Some(PickGroup::Files);
    let name = match entry.kind {
        EntryKind::HiddenGroup => virtual_name(HIDDEN_GROUP, theme.virtual_folders.hidden, theme).into_any_element(),
        _ => entry.name.clone().into_any_element(),
    };
    let file_icon = if entry.expands() { folder_mark(file_icon, &line.mark, theme) } else { icon(theme, file_icon).into_any_element() };
    let line = line.div.child(disclosure).child(file_icon).child(name).children(warning);
    if pickable {
        let dragged = DraggedFiles { pressed: entry.path.clone(), workspace: cx.entity().downgrade() };
        return bind_row(line.on_drag(dragged, |dragged, _offset, _window, cx| cx.new(|cx| dragged.chip(cx))), mouse, cx);
    }
    bind_row(line, mouse, cx)
}

/// Where a run's actions are drawn. Each place names its own click targets: a run shown in both would otherwise
/// register one name twice, and the first registration in a frame (the tree's) would take every MCP click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunActionsIn {
    /// Processes in the tree, `restart-process:<n>`: Restart and Start bring the new run's tab to the front
    Tree,
    /// the Run configurations tab, `run-configs-restart:<n>`: the list stays shown
    RunConfigs,
}

impl RunActionsIn {
    fn show(self) -> ShowRun {
        match self {
            Self::Tree => ShowRun::Tab,
            Self::RunConfigs => ShowRun::Stay,
        }
    }

    /// The target of action `verb` (`restart`, `stop`, `start`, `remove`) on `run`.
    pub(crate) fn target_name(self, verb: &str, run: RunId) -> String {
        let verb = match self {
            Self::Tree => format!("{verb}-process"),
            Self::RunConfigs => format!("run-configs-{verb}"),
        };
        target_name(&verb, &run.target())
    }
}

/// A run's actions: Restart and Stop while it runs or waits, Start and Remove once it exited.
pub(crate) fn run_actions(run: &RunRow, place: RunActionsIn, cx: &mut Context<Workspace>) -> Vec<RowAction> {
    let (id, show) = (run.id, place.show());
    let named = |verb: &str| place.target_name(verb, id);
    if run.live {
        return vec![
            RowAction {
                id: "restart-run",
                icon: icons::RESTART,
                tip: "Restart".to_owned(),
                on_click: Box::new(target(named("restart"), cx.listener(move |workspace, _event, window, cx| workspace.restart_run_with(id, show, window, cx)), cx)),
            },
            RowAction {
                id: "stop-run",
                icon: icons::STOP,
                tip: "Stop".to_owned(),
                on_click: Box::new(target(named("stop"), cx.listener(move |workspace, _event, _window, cx| workspace.stop_run(id, cx)), cx)),
            },
        ];
    }
    vec![
        RowAction {
            id: "start-run",
            icon: icons::RUN,
            tip: "Start".to_owned(),
            on_click: Box::new(target(named("start"), cx.listener(move |workspace, _event, window, cx| workspace.restart_run_with(id, show, window, cx)), cx)),
        },
        RowAction {
            id: "remove-run",
            icon: icons::CLOSE,
            tip: "Remove".to_owned(),
            on_click: Box::new(target(named("remove"), cx.listener(move |workspace, _event, window, cx| workspace.remove_run(id, window, cx)), cx)),
        },
    ]
}

/// A run's state in words after its configuration's name: green while it runs, red for a failure, muted otherwise.
pub(crate) fn run_label(run: &RunRow, theme: Theme) -> Div {
    let color = match run.state {
        RunBadge::Running => theme.mark_added,
        RunBadge::Failed | RunBadge::Exited(1..) | RunBadge::Exited(..0) => theme.error,
        RunBadge::Waiting | RunBadge::Stopped | RunBadge::Exited(0) => theme.muted,
    };
    div().flex_none().text_xs().text_color(rgb(color)).child(run.label.clone())
}

/// The paging row under a long list: Show 10 more, Show all.
fn more_line(line: RowLine, row: &MoreRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let (more_project, more_list, all_project, all_list) = (row.project.clone(), row.list.clone(), row.project.clone(), row.list.clone());
    let more = target(line.name("more"), cx.listener(move |workspace, _event, _window, cx| workspace.show_more(&more_project, &more_list, cx)), cx);
    let all = target(line.name("all"), cx.listener(move |workspace, _event, _window, cx| workspace.show_all(&all_project, &all_list, cx)), cx);
    let footer = more_row(("tree-more", line.index), row.total, row.shown, line.theme, more, all);
    line.div.cursor_default().children(footer)
}

fn project_row(line: RowLine, project: &ProjectRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let select = line.select.clone();
    let (toggled, shown, closed) = (project.id.clone(), project.id.clone(), project.id.clone());
    let present = project.tag != Some(ProjectTag::Missing);
    let name_color = if present { theme.text } else { theme.muted };
    let sizes = &theme.sizes.tree;
    let tag_text = sizes.tag_text_size.px();
    let tag = project.tag.as_ref().map(|tag| match tag {
        ProjectTag::Branch(branch) => div()
            .flex()
            .items_center()
            .gap(sizes.tag_gap.px())
            .px(sizes.tag_padding_x.px())
            .rounded(sizes.tag_radius.px())
            .bg(rgb(theme.hover))
            .text_size(tag_text)
            .child(icon(theme, icons::BRANCH))
            .child(branch.clone()),
        ProjectTag::NoGit => div().text_size(tag_text).text_color(rgb(secondary(theme))).child("no git"),
        ProjectTag::Missing => div().text_size(tag_text).text_color(rgb(theme.error)).child("missing"),
    });
    let running = (project.running > 0).then(|| div().text_size(tag_text).text_color(rgb(secondary(theme))).child(format!("● {} running", project.running)));
    let warning = project.error.clone().map(|error| {
        div().id("project-warning").flex_none().child(icon(theme, icons::WARNING)).tooltip(tooltip::text_tooltip(error))
    });
    let close = RowAction {
        id: "close-project",
        icon: icons::CLOSE,
        tip: "Close project".to_owned(),
        on_click: Box::new(target(line.name("close-project"), cx.listener(move |workspace, _event, window, cx| {
            workspace.close_project(&closed, window, cx);
        }), cx)),
    };
    let close = row_action(close, &line);
    // a missing project has nothing to expand or show: Close is all it offers
    let toggle: Option<Listener> = present.then(|| {
        let toggle = cx.listener(move |workspace, _event, _window, cx| workspace.toggle_project(&toggled, cx));
        Rc::new(target(line.name("toggle"), toggle, cx)) as Listener
    });
    let chevron = toggle.as_ref().map(|toggle| chevron(project.expanded, theme, shared(toggle)));
    let project_target = line.name("project");
    let (badge_fill, badge_text) = if present { (theme.accent, theme.on_accent) } else { (theme.hover, theme.muted) };
    // its favicon, else its initials on a badge
    let mark = match line.mark.favicon.clone() {
        Some(path) => {
            let initials = project.initials.clone();
            favicon_or(&path, theme.sizes.icon.px(), move || badge(&initials, badge_fill, badge_text, theme).into_any_element()).into_any_element()
        }
        None => badge(&project.initials, badge_fill, badge_text, theme).into_any_element(),
    };
    let line = line
        .div
        .children(chevron)
        .child(mark)
        .child(div().font_weight(sizes.project_name_weight.font_weight()).text_color(rgb(name_color)).child(project.name.clone()))
        .children(tag)
        .children(running)
        .children(waiting_tag(project.waiting, theme))
        .children(warning)
        .child(close);
    if present {
        let show: Listener = Rc::new(target(project_target.clone(), cx.listener(move |workspace, _event, window, cx| {
            workspace.show_project_overview(&shown, window, cx);
        }), cx));
        let keep = keep_shown(&select, cx);
        return bind_row(line, RowMouse { select, name: project_target, checkout: None, activate: show, keep, toggle }, cx);
    }
    line
}

/// What a tree row's mouse bindings run: `ActivateRow` its click target (what MCP `click` runs too),
/// `ActivateRowToKeep` that and then `keep` on a row whose tab may open temporary, `ToggleRow` its chevron's toggle,
/// and a bound function gets `{ row, checkout }`.
struct RowMouse {
    select: RowSelect,
    name: String,
    checkout: Option<PathBuf>,
    activate: Listener,
    keep: Option<Listener>,
    toggle: Option<Listener>,
}

/// Makes the tab the row just showed stay, when it is the shown one.
fn keep_shown(select: &RowSelect, cx: &mut Context<Workspace>) -> Option<Listener> {
    let row = select.key.clone();
    Some(Rc::new(cx.listener(move |workspace, _event, _window, cx| workspace.keep_shown_tab_of(&row, cx))))
}

fn bind_row(line: Stateful<Div>, row: RowMouse, cx: &App) -> Stateful<Div> {
    let RowMouse { select, name, checkout, activate, keep, toggle } = row;
    let context = serde_json::json!({ "row": name, "checkout": checkout });
    let menu_row = name.clone();
    // for MCP's click: opens the row's default menu, as a right press does, at the pointer
    let (opened, opened_row) = (select.clone(), name.clone());
    drop(target(format!("tree-menu:{name}"), move |_event, window, cx| open_menu(&opened, &opened_row, crate::workspace::DEFAULT_MENU, window, cx), cx));
    let selected = select.clone();
    let click = Rc::new(move |window: &mut Window, cx: &mut App| {
        selected.update(cx, |selection, key, _listed| selection.select_only(key));
        activate(&ClickEvent::default(), window, cx);
    });
    // for MCP's click: a Ctrl+click and a Shift+click on the row, which MCP's click cannot hold keys for
    if select.pick.is_some() {
        let (toggled, extended, fallback) = (select.clone(), select.clone(), click.clone());
        drop(target(format!("tree-pick:{name}"), move |_event, _window, cx| toggled.update(cx, |selection, key, listed| selection.toggle(key, listed)), cx));
        drop(target(format!("tree-pick-range:{name}"), move |_event, window, cx| {
            if !extended.update(cx, |selection, key, listed| selection.extend(&key, listed)) {
                fallback(window, cx);
            }
        }, cx));
    }
    mouse_binding::on_mouse_triggers(line, format!("tree:{name}"), move |trigger, window, cx| match bound_mouse(Scope::Tree, &trigger, cx) {
        Some(Action::Builtin(Builtin::ActivateRow)) => click(window, cx),
        Some(Action::Builtin(Builtin::ActivateRowToKeep)) => {
            click(window, cx);
            if let Some(keep) = &keep {
                keep(&ClickEvent::default(), window, cx);
            }
        }
        Some(Action::Builtin(Builtin::ToggleRowSelection)) if select.pick.is_some() => select.update(cx, |selection, key, listed| selection.toggle(key, listed)),
        Some(Action::Builtin(Builtin::ExtendRowSelection)) if select.pick.is_some() => {
            if !select.update(cx, |selection, key, listed| selection.extend(&key, listed)) {
                click(window, cx);
            }
        }
        // a row that cannot be picked with others takes a Ctrl+ or Shift+click as a click
        Some(Action::Builtin(Builtin::ToggleRowSelection | Builtin::ExtendRowSelection)) => click(window, cx),
        Some(Action::Builtin(Builtin::ToggleRow)) => {
            if let Some(toggle) = &toggle {
                toggle(&ClickEvent::default(), window, cx);
            }
        }
        Some(Action::Invoke { generation, function }) => invoke(generation, function, Some(context.clone()), cx),
        Some(Action::ShowTreeMenu(menu)) => open_menu(&select, &menu_row, &menu, window, cx),
        Some(Action::Builtin(builtin)) if crate::workspace::is_path_action(builtin) => {
            let key = select.key.clone();
            let _ = select.workspace.update(cx, |workspace, cx| workspace.run_row_path_action(key, builtin, window, cx));
        }
        // the keymap allows no other action in the tree scope
        _ => {}
    })
}

/// Opens tree menu `menu` on the row of `select`, named `row` for MCP.
fn open_menu(select: &RowSelect, row: &str, menu: &str, window: &mut Window, cx: &mut App) {
    let key = select.key.clone();
    // the workspace is gone only while the window closes
    let _ = select.workspace.update(cx, |workspace, cx| workspace.open_tree_menu(key, row.to_owned(), menu, window, cx));
}

/// What a row's selection clicks change: the workspace's tree selection, by the row's key.
#[derive(Clone)]
struct RowSelect {
    key: TreeKey,
    /// what it can be picked with; `None`: alone
    pick: Option<PickGroup>,
    workspace: WeakEntity<Workspace>,
}

impl RowSelect {
    /// Runs `change` on the tree's selection with this row's key and the listed rows, and redraws; what `change`
    /// returned, false when the workspace is gone.
    fn update<R: Default>(&self, cx: &mut App, change: impl FnOnce(&mut Selection, TreeKey, &[Listed]) -> R) -> R {
        let key = self.key.clone();
        self.workspace
            .update(cx, |workspace, cx| {
                let changed = workspace.change_selection(|selection, listed| change(selection, key, listed));
                cx.notify();
                changed
            })
            .unwrap_or_default()
    }
}

/// A shared listener as a handler of its own, for an element that takes one by value.
fn shared(listener: &Listener) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let listener = listener.clone();
    move |event, window, cx| listener(event, window, cx)
}

/// Expands or collapses its row without the row's own click (opening an overview) firing.
pub(crate) fn chevron(expanded: bool, theme: Theme, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Stateful<Div> {
    div()
        .id("chevron")
        .flex_none()
        .rounded(theme.sizes.tree.chevron_radius.px())
        .hover(|style| style.bg(rgb(theme.border)))
        .child(icon(theme, if expanded { icons::CHEVRON_DOWN } else { icons::CHEVRON_RIGHT }))
        .on_click(move |event, window, cx| {
            cx.stop_propagation();
            on_click(event, window, cx);
        })
}

/// The name of a row that is not on disk (`@hidden`) on a chip drawn with `style`.
fn virtual_name(name: &'static str, style: RowStyle, theme: Theme) -> Div {
    let sizes = &theme.sizes.tree;
    styled(div().px(sizes.chip_padding_x.px()).rounded(sizes.chip_radius.px()), style).child(name)
}

/// Keeps a folder's count a step behind its label, which has the same colour on secondary rows.
const COUNT_OPACITY: f32 = 0.75;

/// A folder's count after its label: `· 4`.
fn count_badge(count: usize, theme: Theme) -> Div {
    dot_note(count.to_string(), theme)
}

/// `· <text>` after a folder's label: its count, or why it is empty.
fn dot_note(text: String, theme: Theme) -> Div {
    let sizes = &theme.sizes.tree;
    div()
        .flex_none()
        .px(sizes.folder_count_padding_x.px())
        .text_size(sizes.folder_count_text_size.px())
        .text_color(rgb(secondary(theme)))
        .opacity(COUNT_OPACITY)
        .child(format!("· {text}"))
}

pub(crate) type OnClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A button at a row's right end: Close project, New terminal, Close, Stop.
pub(crate) struct RowAction {
    pub id: &'static str,
    pub icon: &'static str,
    pub tip: String,
    pub on_click: OnClick,
}

/// When a row's buttons show.
pub(crate) enum Shown {
    /// a configuration's ▷ Run, before it ever ran
    Always,
    OnHover,
}

/// `actions` at the right end of a row of hover group `group`, side by side, over a long name.
pub(crate) fn row_actions(actions: Vec<RowAction>, theme: Theme, group: SharedString, shown: Shown) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let action_size = theme.sizes.icon.0 + 2. * sizes.action_padding.0;
    let buttons = actions.into_iter().map(|action| {
        let on_click = action.on_click;
        div()
            .id(action.id)
            .p(sizes.action_padding.px())
            .rounded(sizes.action_radius.px())
            .hover(|style| style.bg(rgb(theme.border)))
            .child(icon(theme, action.icon))
            .tooltip(tooltip::text_tooltip(action.tip))
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                on_click(event, window, cx);
            })
    });
    let hover_group = group;
    let group = div()
        .id("row-actions")
        .absolute()
        .top(px((sizes.row_height.0 - action_size) / 2.))
        .right(sizes.padding_right.px())
        .flex()
        .gap_0p5()
        .rounded_sm()
        .children(buttons);
    match shown {
        Shown::Always => group,
        Shown::OnHover => group.bg(rgb(theme.hover)).invisible().group_hover(hover_group, |style| style.visible()),
    }
}

/// `action` at the right end of `line`, shown while the mouse is over the row.
fn row_action(action: RowAction, line: &RowLine) -> Stateful<Div> {
    let theme = line.theme;
    let sizes = &theme.sizes.tree;
    let action_size = theme.sizes.icon.0 + 2. * sizes.action_padding.0;
    let on_click = action.on_click;
    div()
        .id(action.id)
        // over a long name rather than after it; only shown on hover, so it covers the text with the hover colour
        .absolute()
        .top(px((sizes.row_height.0 - action_size) / 2.))
        .right(sizes.padding_right.px())
        .p(sizes.action_padding.px())
        .rounded(sizes.action_radius.px())
        .bg(rgb(theme.hover))
        .invisible()
        .group_hover(line.group.clone(), |style| style.visible())
        .hover(|style| style.bg(rgb(theme.border)))
        .child(icon(theme, action.icon))
        .tooltip(tooltip::text_tooltip(action.tip))
        .on_click(move |event, window, cx| {
            cx.stop_propagation();
            on_click(event, window, cx);
        })
}

/// A strip along the sidebar's right edge, wide enough to grab without hunting for the one-pixel border: drag it to
/// resize, double-click it for the default width.
fn render_resize_handle(theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("sidebar-resize")
        .absolute()
        .top_0()
        .bottom_0()
        .right_0()
        .w(theme.sizes.tree.resize_handle_width.px())
        .cursor(CursorStyle::ResizeLeftRight)
        .on_mouse_down(MouseButton::Left, cx.listener(|workspace, event: &MouseDownEvent, _window, cx| {
            cx.stop_propagation();
            if event.click_count == 2 {
                workspace.reset_sidebar_width(cx);
                return;
            }
            workspace.start_sidebar_resize(f32::from(event.position.x));
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_widen_a_row_by_what_its_process_said() {
        // given
        let process = |attention: Option<&str>| TreeRow {
            depth: 0,
            item: TreeItem::Process(ProcessRow {
                id: crate::process::ProcessId::for_tests(1),
                kind: crate::process::ProcessKind::Terminal,
                title: "zsh".to_owned(),
                progress: crate::terminal::Progress::None,
                attention: attention.map(str::to_owned),
            }),
            target: String::new(),
        };

        // when
        let widths = [row_width(&process(None)), row_width(&process(Some("Claude needs you")))];

        // then
        assert_eq!(widths[1] - widths[0], "● Claude needs you".chars().count() + 1);
    }

    #[test]
    fn should_keep_tree_selection_when_shown_tab_changes() {
        // given
        let (selected, shown) = (TreeKey::Entry(PathBuf::from("/p/a")), TreeKey::Entry(PathBuf::from("/p/b")));
        let mut tree = Tree::new();
        tree.selection.select_only(selected.clone());

        // when
        tree.show_active(Some(shown.clone()));

        // then
        assert_eq!((tree.selection.contains(&selected), tree.selection.contains(&shown)), (true, false));
        assert_eq!(tree.active, Some(shown));
    }

    #[test]
    fn should_name_run_actions_after_where_they_are_drawn() {
        // given
        let (live, waiting) = (RunId::Process(crate::process::ProcessId::for_tests(7)), RunId::Pending(2));

        // when
        let tree = [RunActionsIn::Tree.target_name("restart", live), RunActionsIn::Tree.target_name("start", waiting)];
        let tab = ["restart", "stop", "start", "remove"].map(|verb| RunActionsIn::RunConfigs.target_name(verb, live));

        // then
        assert_eq!(tree, ["restart-process:7", "start-process:pending-2"]);
        assert_eq!(tab, ["run-configs-restart:7", "run-configs-stop:7", "run-configs-start:7", "run-configs-remove:7"]);
    }
}
