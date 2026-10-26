//! The Run configurations tab of a checkout: every configuration its `.run.toml` files declare, from the hub
//! index, flat or as a folder tree, filtered, each row with its run's state and actions.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, Context, Div, Entity, ScrollStrategy, SharedString, Stateful, Window, div, prelude::*, px, rgb,
    uniform_list,
};

use super::configs_list::{self, ConfigEntry, ListItem, ListRow};
use super::file::{ConfigKind, RunFile};
use super::values_tab::{RunRequest, ShowRun};
use crate::click_targets::{Listener, target};
use crate::selectable::selectable;
use crate::icons;
use crate::overview::file_list::FileLayout;
use crate::overview::git_ui::{Picked, muted, segment};
use crate::project::CheckoutRef;
use crate::scrollbar::{ListScroll, ScrollState, vertical_scrollbar};
use crate::sidebar::rows::{RunRow, RunsView};
use crate::sidebar::view::{RowAction, RunActionsIn, Shown, chevron, hovered, row_actions, run_actions, run_label, secondary, tree_indent};
use crate::tabs::TabId;
use crate::text_input::{TextInput, TextInputEvent};
use crate::theme::Theme;
use crate::tooltip;
use crate::workspace::{Workspace, icon};

/// As wide as the mock's page: wider, a name and its actions drift apart.
const MAX_WIDTH: f32 = 600.;
/// Room for the whole placeholder.
const FILTER_WIDTH: f32 = 260.;
const FILTER_PLACEHOLDER: &str = "Filter by name, folder or command";

/// The run files of the checkout, as the hub index read them.
pub type RunFiles = Arc<Vec<(String, Arc<RunFile>)>>;

pub struct RunConfigsTab {
    pub checkout: CheckoutRef,
    pub filter: Entity<TextInput>,
    /// per tab, not saved: Flat when it opens
    pub layout: FileLayout,
    /// the tree's folded folders, by checkout-relative path
    pub folded: HashSet<String>,
    pub list: ListScroll,
    pub scroll: ScrollState,
    /// the entries of the last read and filter, so a redraw does not clone every configuration again
    entries: RefCell<Option<Filtered>>,
}

struct Filtered {
    files: RunFiles,
    filter: String,
    entries: Rc<Vec<ConfigEntry>>,
}

impl RunConfigsTab {
    /// Typing in the filter redraws the workspace, which draws this tab, from the top of the list.
    pub fn new(checkout: CheckoutRef, window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        let filter = cx.new(|cx| {
            let mut input = TextInput::new(cx);
            input.placeholder(FILTER_PLACEHOLDER);
            input.set_target("run-configs-filter");
            input
        });
        let list = ListScroll::new();
        let scroll = ScrollState::for_list(&list);
        let handle = list.list_handle();
        cx.subscribe_in(&filter, window, move |_workspace, _input, event: &TextInputEvent, _window, cx| {
            if matches!(event, TextInputEvent::Changed(_)) {
                // the old offset would leave a shorter list scrolled past its first matches
                handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
        })
        .detach();
        Self { checkout, filter, layout: FileLayout::Flat, folded: HashSet::new(), list, scroll, entries: RefCell::new(None) }
    }

    fn entries(&self, files: &RunFiles, filter: &str) -> Rc<Vec<ConfigEntry>> {
        let mut cached = self.entries.borrow_mut();
        if let Some(hit) = cached.as_ref().filter(|hit| Arc::ptr_eq(&hit.files, files) && hit.filter == filter) {
            return hit.entries.clone();
        }
        let entries = Rc::new(configs_list::entries(files, filter));
        *cached = Some(Filtered { files: files.clone(), filter: filter.to_owned(), entries: entries.clone() });
        entries
    }
}

/// What the list's rows are drawn from, shared with the list's row builder.
struct Listing {
    tab: TabId,
    checkout: CheckoutRef,
    layout: FileLayout,
    entries: Rc<Vec<ConfigEntry>>,
    rows: Vec<ListRow>,
}

/// The bar (filter, Flat | Tree, where), the summary, then the rows; `Reading run configurations…` until the hub
/// index has read the checkout, or why it could not.
pub fn render_run_configs_tab(id: TabId, tab: &RunConfigsTab, workspace: &Workspace, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let page = div().id("run-configs").flex().flex_col().size_full().max_w(px(MAX_WIDTH)).px(px(16.)).pt(px(12.)).gap_2();
    let page = page.child(bar(id, tab, workspace, theme, cx));
    let checkout = tab.checkout.path.clone();
    let Some(files) = workspace.run_files(&checkout) else {
        return page.child(unread(workspace.run_files_failure(&checkout), theme)).into_any_element();
    };
    if !configs_list::lists_any(&files) {
        return page.child(muted("No run configurations in this worktree", theme)).into_any_element();
    }
    let runs = runs_by_folder(workspace, &checkout, &files);
    let running = configs_list::running(&runs);
    let filter = tab.filter.read(cx).text().to_owned();
    let entries = tab.entries(&files, &filter);
    let run_of = |folder: &str, key: &str| runs.get(folder).and_then(|runs| runs.iter().find(|run| run.key == key)).cloned();
    let rows = configs_list::rows(&entries, &configs_list::run_folders(&files), tab.layout, &tab.folded, &run_of);
    let page = page.child(muted(configs_list::summary(&files, running), theme));
    if rows.is_empty() {
        return page.child(muted("No configuration matches", theme)).into_any_element();
    }
    let listing = Rc::new(Listing { tab: id, checkout: tab.checkout.clone(), layout: tab.layout, entries, rows });
    page.child(list(listing, tab, theme, cx)).into_any_element()
}

/// Before the checkout's first read: reading it, or why the last read failed.
fn unread(failure: Option<String>, theme: Theme) -> Div {
    match failure {
        Some(error) => div().text_color(rgb(theme.error)).child(format!("Cannot read run configurations: {error}")),
        None => muted("Reading run configurations…", theme),
    }
}

/// The filter, Flat | Tree, and the worktree's label and folder.
fn bar(id: TabId, tab: &RunConfigsTab, workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let layouts = [(FileLayout::Flat, "flat", "Flat"), (FileLayout::Tree, "tree", "Tree")].map(|(layout, name, label)| {
        let pick = target(format!("run-configs-layout:{name}"), cx.listener(move |workspace, _event, _window, cx| workspace.set_run_configs_layout(id, layout, cx)), cx);
        segment(("run-configs-layout", layout as usize), label, Picked::from(tab.layout == layout), theme).on_click(pick)
    });
    let main = workspace.project(&tab.checkout.project).map(|project| project.main_checkout().path.clone());
    let folder = crate::paths::card_folder(&tab.checkout.path, main.as_deref().unwrap_or(&tab.checkout.path), dirs::home_dir().as_deref());
    let place = format!("{} · {folder}", workspace.checkout_label(&tab.checkout));
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_2()
        .child(div().flex_none().w(px(FILTER_WIDTH)).overflow_hidden().child(tab.filter.clone()))
        .child(div().flex().flex_none().gap_1().children(layouts))
        .child(selectable(format!("run-configs-place-{}", tab.checkout.path.display()), place, cx).text_color(rgb(theme.muted)).flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap())
}

/// Every listed folder's runs, read once per draw rather than once per row.
fn runs_by_folder(workspace: &Workspace, checkout: &Path, files: &RunFiles) -> HashMap<String, Vec<RunRow>> {
    let folders = files.iter().map(|(folder, _)| folder).filter(|folder| !folder.is_empty());
    folders.map(|folder| (folder.clone(), workspace.runs_in(checkout, &checkout.join(folder)))).filter(|(_, runs)| !runs.is_empty()).collect()
}

fn list(listing: Rc<Listing>, tab: &RunConfigsTab, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let count = listing.rows.len();
    // only the visible rows are built, so a thousand configurations stay cheap to scroll
    let rows = uniform_list(
        "run-configs-list",
        count,
        cx.processor(move |_workspace: &mut Workspace, range: std::ops::Range<usize>, _window, cx| {
            let theme = crate::theme::tabs(cx);
            range.map(|index| list_row(&listing, index, theme, cx)).collect::<Vec<_>>()
        }),
    )
    .track_scroll(&tab.list.list_handle())
    .size_full();
    div().relative().flex_1().min_h_0().child(rows).child(vertical_scrollbar(&tab.scroll, &crate::theme::view_style(&theme)))
}

fn list_row(listing: &Listing, index: usize, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let row = &listing.rows[index];
    let line = RowLine::new(index, row.depth, theme);
    match &row.item {
        ListItem::Folder { path, label, run_file, live, open } => {
            folder_row(line, FolderLine { path, label, run_file: *run_file, live: *live, open: *open }, listing, cx)
        }
        ListItem::Config { entry, run } => config_row(line, &listing.entries[*entry], run.as_ref(), listing, cx),
    }
}

/// A row's line as the tree draws one (height, text, indent, hover), and the hover group its actions show with.
struct RowLine {
    div: Stateful<Div>,
    group: SharedString,
    theme: Theme,
}

impl RowLine {
    fn new(index: usize, depth: usize, theme: Theme) -> Self {
        let sizes = &theme.sizes.tree;
        let group = SharedString::from(format!("run-configs-row-{index}"));
        let hover = theme.tree_states.hover;
        let div = div()
            .id(("run-configs-row", index))
            .group(group.clone())
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap(sizes.icon_gap.px())
            .h(sizes.row_height.px())
            .pl(px(sizes.padding_left.0 + depth as f32 * tree_indent(theme)))
            .pr(sizes.padding_right.px())
            .rounded(sizes.selection_radius.px())
            .text_size(sizes.text_size.px())
            .whitespace_nowrap()
            .overflow_hidden()
            .cursor_pointer()
            .hover(move |style| hovered(style, hover));
        Self { div, group, theme }
    }
}

struct FolderLine<'a> {
    path: &'a str,
    label: &'a str,
    run_file: bool,
    live: usize,
    open: bool,
}

/// A tree folder: chevron, icon (with a run badge when it has a `.run.toml`, which opens on hover), name, `● n`
/// live runs below it; a click folds it.
fn folder_row(line: RowLine, folder: FolderLine, listing: &Listing, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let (tab, path) = (listing.tab, folder.path.to_owned());
    let toggle: Listener = Rc::new(target(format!("run-configs-folder:{path}"), cx.listener(move |workspace, _event, _window, cx| {
        workspace.toggle_run_configs_folder(tab, path.clone(), cx);
    }), cx));
    let open_file = folder.run_file.then(|| {
        let file = listing.checkout.path.join(folder.path).join(super::file::RUN_FILE);
        let action = RowAction {
            id: "open-run-file",
            icon: icons::file_icon(super::file::RUN_FILE),
            tip: format!("Open {}/{}", folder.path, super::file::RUN_FILE),
            on_click: Box::new(target(format!("open-run-file:{}", folder.path), cx.listener(move |workspace, _event, window, cx| {
                workspace.open_document(&file, window, cx);
            }), cx)),
        };
        row_actions(vec![action], theme, line.group.clone(), Shown::OnHover)
    });
    let live = (folder.live > 0).then(|| div().flex_none().text_color(rgb(secondary(theme))).child(format!("● {}", folder.live)));
    let chevron_toggle = toggle.clone();
    line.div
        .child(chevron(folder.open, theme, move |event: &ClickEvent, window: &mut Window, cx: &mut gpui::App| chevron_toggle(event, window, cx)))
        .child(icon(theme, if folder.run_file { icons::FOLDER_RUN } else { icons::FOLDER }))
        .child(folder.label.to_owned())
        .children(live)
        .children(open_file)
        .on_click(move |event, window, cx| toggle(event, window, cx))
}

/// A configuration: its icon, name, folder (flat), its run's state and actions; ▷ Run until it first runs. A click
/// shows its run's tab, when it has a run.
fn config_row(line: RowLine, entry: &ConfigEntry, run: Option<&RunRow>, listing: &Listing, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let config = &entry.config;
    let name = format!("{}#{}", entry.folder, config.key);
    let kind_icon = match config.kind {
        ConfigKind::Serve(_) => icons::FILE_HTML,
        ConfigKind::Run(_) => icons::RUN_CONFIG,
    };
    let shown = run.map(|run| run.id);
    let show = target(format!("run-config:{name}"), cx.listener(move |workspace, _event, window, cx| {
        if let Some(id) = shown {
            workspace.show_run(id, window, cx);
        }
    }), cx);
    let (actions, row) = match run {
        Some(run) => (row_actions(run_actions(run, RunActionsIn::RunConfigs, cx), theme, line.group.clone(), Shown::OnHover), line.div),
        // ▷ Run is always drawn, so the text keeps clear of it rather than running under it
        None => (row_actions(vec![run_action(&name, request(listing, entry), cx)], theme, line.group.clone(), Shown::Always), line.div.pr(always_shown_room(theme))),
    };
    let folder = (listing.layout == FileLayout::Flat).then(|| clipped(entry.folder.clone()).text_color(rgb(secondary(theme))));
    let tip = match run {
        Some(run) => format!("{}\n{}", entry.hint, run.status),
        None => entry.hint.clone(),
    };
    row.child(div().flex_none().size(theme.sizes.icon.px()))
        .child(icon(theme, kind_icon))
        .child(clipped(config.name.clone()))
        .children(folder)
        .children(run.map(|run| run_label(run, theme)))
        .child(actions)
        .tooltip(tooltip::text_tooltip(tip))
        .on_click(show)
}

/// The row's right padding plus one action button.
fn always_shown_room(theme: Theme) -> gpui::Pixels {
    let sizes = &theme.sizes.tree;
    px(sizes.padding_right.0 + theme.sizes.icon.0 + 2. * sizes.action_padding.0)
}

/// Text that shrinks to an ellipsis when the row is too narrow.
fn clipped(text: String) -> Div {
    div().min_w_0().overflow_hidden().text_ellipsis().child(text)
}

fn request(listing: &Listing, entry: &ConfigEntry) -> RunRequest {
    let folder: PathBuf = listing.checkout.path.join(&entry.folder);
    RunRequest { checkout: listing.checkout.clone(), folder, key: entry.config.key.clone(), show: ShowRun::Stay }
}

/// ▷ Run: starts it where it is, the list staying shown.
fn run_action(name: &str, request: RunRequest, cx: &mut Context<Workspace>) -> RowAction {
    RowAction {
        id: "run-config",
        icon: icons::RUN,
        tip: "Run".to_owned(),
        on_click: Box::new(target(format!("run:{name}"), cx.listener(move |workspace, _event, window, cx| workspace.request_run(request.clone(), window, cx)), cx)),
    }
}
