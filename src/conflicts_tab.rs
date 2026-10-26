//! A copy or move with files in the way asks here first: one row per conflict with Replace, Skip or Keep both;
//! nothing is written until Copy (or Move) is pressed, and closing the tab writes nothing.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use gpui::{AnyElement, Context, Div, IntoElement, ParentElement, SharedString, Stateful, Styled, div, prelude::*, px, rgb};

use crate::click_targets::target;
use crate::colors::checkout_colour;
use crate::icons;
use crate::overview::git_ui::{Picked, muted, segment};
use crate::project::CheckoutRef;
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::transfer::Verb;
use crate::transfer::plan::{Action, Conflict, Plan, counted, offered};
use crate::workspace::{Workspace, icon};

/// The tab's widest content; wider windows leave the rest empty.
const MAX_WIDTH: f32 = 1200.;
/// where a row's second line starts: under its name, past the chevron and the icon
const DETAIL_INDENT: f32 = 40.;
const STRIPE_WIDTH: f32 = 3.;
const PREVIEW_HEIGHT: f32 = 220.;
/// conflict rows drawn; All reaches the others too
const MAX_SHOWN_ROWS: usize = 500;


/// What each conflicting row's two files are, by row: `12 lines · 189 B`, read with the plan.
pub type Details = HashMap<usize, (String, String)>;

pub struct ConflictsTab {
    pub plan: Plan,
    pub details: Details,
    unfolded: BTreeSet<usize>,
    /// both files' first lines, read when a row first unfolds
    previews: HashMap<usize, (String, String)>,
    pub clean_unfolded: bool,
    /// the names Keep both gives, worked out again whenever an action changes
    kept: HashMap<usize, PathBuf>,
}

impl ConflictsTab {
    pub fn new(plan: Plan, details: Details) -> Self {
        let kept = plan.kept_names(exists);
        Self { plan, details, unfolded: BTreeSet::new(), previews: HashMap::new(), clean_unfolded: false, kept }
    }

    pub fn title(&self) -> String {
        if self.plan.operation.verb == Verb::Move { "Move conflicts".to_owned() } else { "Copy conflicts".to_owned() }
    }

    /// The tab's two lines on top: what is copied where, and how much is in the way.
    pub fn header(&self, label: impl Fn(&CheckoutRef) -> String) -> (String, String) {
        let button = if self.plan.operation.verb == Verb::Move { "Move" } else { "Copy" };
        let in_the_way = counted(self.plan.conflicts().count(), "file is", "files are");
        let hint = format!("{in_the_way} in the way · {} · nothing is written until you press {button}", self.plan.layout_text());
        (self.plan.title(label), hint)
    }

    pub fn offered(&self, index: usize) -> &'static [Action] {
        self.plan.rows[index].conflict.as_ref().map_or(&[], offered)
    }

    /// Row `index` gets `action`, where it is offered.
    pub fn set_action(&mut self, index: usize, action: Action) {
        if self.offered(index).contains(&action) {
            self.plan.rows[index].action = Some(action);
            self.kept = self.plan.kept_names(exists);
        }
    }

    /// Every conflict that offers `action` gets it.
    pub fn set_all(&mut self, action: Action) {
        self.plan.set_all(action);
        self.kept = self.plan.kept_names(exists);
    }

    /// The name Keep both gives row `index`, when it is chosen there.
    pub fn kept_name(&self, index: usize) -> Option<&Path> {
        self.kept.get(&index).map(PathBuf::as_path)
    }

    pub fn can_apply(&self) -> bool {
        self.plan.undecided() == 0
    }

    pub fn toggle_row(&mut self, index: usize) {
        if !self.unfolded.remove(&index) {
            self.unfolded.insert(index);
        }
    }

    pub fn is_unfolded(&self, index: usize) -> bool {
        self.unfolded.contains(&index)
    }

    /// Both files' first lines for row `index`, read once.
    pub fn read_preview(&mut self, index: usize) {
        if self.previews.contains_key(&index) {
            return;
        }
        let row = &self.plan.rows[index];
        let there = match &row.conflict {
            Some(Conflict::Twice { first }) => format!("(also incoming, from {})", first.display()),
            Some(Conflict::FolderThere) => "(a folder)".to_owned(),
            _ => crate::transfer::scan::preview(&row.destination),
        };
        self.previews.insert(index, (there, crate::transfer::scan::preview(&row.incoming.path)));
    }

    /// Why row `index` is in the way: `differs: +1 −0 lines`, `same content`, `also coming from dev · a.css`.
    pub fn why(&self, index: usize, label: impl Fn(&CheckoutRef) -> String) -> String {
        match &self.plan.rows[index].conflict {
            Some(Conflict::Differs { added, removed }) => format!("differs: +{added} −{removed} lines"),
            Some(Conflict::DiffersBinary) | Some(Conflict::Exists) => "differs".to_owned(),
            Some(Conflict::Same) => "same content".to_owned(),
            Some(Conflict::Twice { first }) => {
                let source = self.plan.rows.iter().find(|row| row.incoming.path == *first).map(|row| &self.plan.operation.sources[row.incoming.top].checkout);
                let shown = source.and_then(|checkout| first.strip_prefix(&checkout.path).ok()).unwrap_or(first);
                let from = source.map(&label).unwrap_or_default();
                format!("also coming from {from} · {}", shown.display())
            }
            Some(Conflict::FolderThere) => "a folder of that name is there".to_owned(),
            Some(Conflict::FileThere) => "a file of that name is there".to_owned(),
            None => String::new(),
        }
    }
}

fn exists(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn action_name(action: Action) -> (&'static str, &'static str) {
    match action {
        Action::Replace => ("replace", "Replace"),
        Action::Skip => ("skip", "Skip"),
        Action::KeepBoth => ("both", "Keep both"),
    }
}

pub fn render_conflicts_tab(id: TabId, tab: &ConflictsTab, workspace: &Workspace, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let label = |checkout: &CheckoutRef| workspace.checkout_label(checkout);
    let (title, hint) = tab.header(label);
    let conflicts: Vec<usize> = tab.plan.rows.iter().enumerate().filter(|(_, row)| row.conflict.is_some()).map(|(index, _)| index).collect();
    let rows: Vec<AnyElement> = conflicts.iter().take(MAX_SHOWN_ROWS).map(|index| conflict_row(Row { id, index: *index, tab, workspace }, theme, cx)).collect();
    let hidden = conflicts.len().saturating_sub(MAX_SHOWN_ROWS);
    let more = (hidden > 0).then(|| muted(format!("{} more not shown; All sets them too", counted(hidden, "conflict", "conflicts")), theme));
    div()
        .id("conflicts")
        .flex()
        .flex_col()
        .size_full()
        .max_w(px(MAX_WIDTH))
        .px(px(16.))
        .pt(px(12.))
        .gap_2()
        .overflow_y_scroll()
        .text_size(theme.sizes.tree.text_size.px())
        .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(title))
        .child(muted(hint, theme))
        .child(all_row(id, theme, cx))
        .children(rows)
        .children(more)
        .children(clean_list(id, tab, workspace, theme, cx))
        .child(buttons(id, tab, theme, cx))
        .into_any_element()
}

fn all_row(id: TabId, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let all = [Action::Replace, Action::Skip, Action::KeepBoth].map(|action| {
        let (name, label) = action_name(action);
        let set = target(format!("conflicts-all:{name}"), cx.listener(move |workspace, _event, _window, cx| workspace.set_all_conflicts(id, action, cx)), cx);
        segment(SharedString::from(format!("conflicts-all-{name}")), label, Picked::No, theme).on_click(set)
    });
    div().flex().items_center().gap_1().child(muted("All", theme)).children(all)
}

struct Row<'a> {
    id: TabId,
    index: usize,
    tab: &'a ConflictsTab,
    workspace: &'a Workspace,
}

fn conflict_row(row: Row<'_>, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let Row { id, index, tab, workspace } = row;
    let planned = &tab.plan.rows[index];
    let target_folder = &tab.plan.operation.target.folder;
    let shown_name = planned.destination.strip_prefix(target_folder).unwrap_or(&planned.destination).display().to_string();
    let source = &tab.plan.operation.sources[planned.incoming.top];
    let from_path = planned.incoming.path.strip_prefix(&source.checkout.path).unwrap_or(&planned.incoming.path).display().to_string();
    let stripe = workspace.checkout(&source.checkout).map_or(theme.accent, |checkout| checkout_colour(checkout, theme));
    let (incoming, there) = tab.details.get(&index).cloned().unwrap_or_default();
    let why = tab.why(index, |checkout| workspace.checkout_label(checkout));
    let why_colour = if planned.conflict == Some(Conflict::Same) { theme.muted } else { theme.mark_modified };
    let unfolded = tab.is_unfolded(index);
    let toggle = target(format!("conflict-row:{index}"), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_conflict_row(id, index, cx)), cx);
    let chevron = if unfolded { icons::CHEVRON_DOWN } else { icons::CHEVRON_RIGHT };
    // the file and what to do with it; under it, where it comes from, both sizes and why it is in the way
    let first = div()
        .flex()
        .items_center()
        .gap_2()
        .h(theme.sizes.tree.row_height.px())
        .child(div().flex().flex_1().min_w_0().gap_1().items_center().child(icon(theme, chevron)).child(icon(theme, icons::file_icon(&shown_name))).child(clipped(shown_name)))
        .child(actions(Row { id, index, tab, workspace }, theme, cx));
    let detail = format!("from {} · {from_path} · {incoming} → {there} · ", workspace.checkout_label(&source.checkout));
    let second = div()
        .flex()
        .items_center()
        .gap_1()
        .pl(px(DETAIL_INDENT))
        .pb_1()
        .min_w_0()
        .child(div().flex_none().w(px(STRIPE_WIDTH)).h(px(12.)).rounded_sm().bg(rgb(stripe)))
        .child(div().flex().min_w_0().overflow_hidden().whitespace_nowrap().text_color(rgb(theme.muted)).child(detail).child(div().text_color(rgb(why_colour)).child(why)));
    let line = div().id(("conflict", index)).flex().flex_col().cursor_pointer().rounded_md().hover(|style| style.bg(rgb(theme.hover))).on_click(toggle).child(first).child(second);
    let preview = unfolded.then(|| preview(tab, index, theme));
    div().flex().flex_col().child(line).children(preview).into_any_element()
}

fn clipped(text: String) -> Div {
    div().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().child(text)
}

/// Replace | Skip | Keep both, as offered; Keep both says the name it gives.
fn actions(row: Row<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let Row { id, index, tab, .. } = row;
    let chosen = tab.plan.rows[index].action;
    let buttons = tab.offered(index).iter().map(|action| {
        let (name, label) = action_name(*action);
        let action = *action;
        let set = target(format!("conflict:{index}:{name}"), cx.listener(move |workspace, _event, _window, cx| workspace.set_conflict(id, (index, action), cx)), cx);
        segment(SharedString::from(format!("conflict-{index}-{name}")), label, Picked::from(chosen == Some(action)), theme).on_click(set)
    });
    let kept = tab.kept_name(index).map(|name| muted(format!("as {}", name.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()), theme));
    let undecided = chosen.is_none();
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_1()
        .when(undecided, |cell| cell.child(div().size(px(6.)).rounded_full().bg(rgb(theme.accent))))
        .children(buttons)
        .children(kept)
}

fn preview(tab: &ConflictsTab, index: usize, theme: Theme) -> Div {
    let (there, incoming) = tab.previews.get(&index).cloned().unwrap_or_default();
    let side = |heading: &str, text: String| {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap_1()
            .child(muted(heading.to_owned(), theme))
            .child(div().max_h(px(PREVIEW_HEIGHT)).overflow_hidden().p_2().rounded_md().border_1().border_color(rgb(theme.border)).bg(rgb(theme.sidebar)).font_family("monospace").whitespace_normal().children(text.lines().map(|line| div().child(line.to_owned())).collect::<Vec<_>>()))
    };
    div().flex().gap_2().pl(px(28.)).pb_2().child(side("there now", there)).child(side("incoming", incoming))
}

/// `4 files go in without a conflict`, folded; unfolded, each with where it comes from.
fn clean_list(id: TabId, tab: &ConflictsTab, workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Option<Stateful<Div>> {
    let clean: Vec<_> = tab.plan.clean().collect();
    if clean.is_empty() {
        return None;
    }
    let toggle = target("conflicts-clean".to_owned(), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_conflicts_clean(id, cx)), cx);
    let chevron = if tab.clean_unfolded { icons::CHEVRON_DOWN } else { icons::CHEVRON_RIGHT };
    let heading = div().flex().items_center().gap_1().child(icon(theme, chevron)).child(muted(counted(clean.len(), "file goes", "files go") + " in without a conflict", theme));
    let target_folder = &tab.plan.operation.target.folder;
    let lines = tab.clean_unfolded.then(|| {
        clean.iter().map(|row| {
            let source = &tab.plan.operation.sources[row.incoming.top];
            let to = row.destination.strip_prefix(target_folder).unwrap_or(&row.destination).display().to_string();
            let from = row.incoming.path.strip_prefix(&source.checkout.path).unwrap_or(&row.incoming.path).display().to_string();
            muted(format!("{to}    from {} · {from}", workspace.checkout_label(&source.checkout)), theme).pl(px(20.))
        }).collect::<Vec<_>>()
    });
    Some(div().id("conflicts-clean").flex().flex_col().gap_1().pt_2().cursor_pointer().on_click(toggle).child(heading).children(lines.into_iter().flatten()))
}

/// Copy (or Move), greyed until every conflict has an action, Cancel, and what pressing it does.
fn buttons(id: TabId, tab: &ConflictsTab, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let label = if tab.plan.operation.verb == Verb::Move { "Move" } else { "Copy" };
    let ready = tab.can_apply();
    let apply = target("conflicts-apply".to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.apply_conflicts(id, window, cx)), cx);
    let cancel = target("conflicts-cancel".to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.close_tab(id, window, cx)), cx);
    let (background, colour) = if ready { (theme.accent, theme.on_accent) } else { (theme.hover, theme.muted) };
    let apply_button = div().id("conflicts-apply").px_3().py_1().rounded_md().bg(rgb(background)).text_color(rgb(colour)).child(label).when(ready, |button| button.cursor_pointer().on_click(apply));
    let cancel_button = div().id("conflicts-cancel").px_3().py_1().rounded_md().cursor_pointer().bg(rgb(theme.hover)).text_color(rgb(theme.text)).child("Cancel").on_click(cancel);
    div().flex().items_center().gap_2().pt_3().pb_4().child(apply_button).child(cancel_button).child(muted(tab.plan.summary(), theme))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use super::*;
    use crate::project::{CheckoutRef, ProjectId};
    use crate::sidebar::drop_place::DropPlace;
    use crate::transfer::Verb;
    use crate::transfer::plan::{Existing, Incoming, IncomingKind, Layout, Operation, Source, SourceKind, plan};

    const GEN: &str = "/a/.claude/worktrees/GEN-817-ive";
    const IDP: &str = "/a-wt/IDP-30";

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("atlassian"), path: PathBuf::from(path) }
    }

    fn label(checkout: &CheckoutRef) -> String {
        match checkout.path.to_str() {
            Some(GEN) => "GEN-817-ive".to_owned(),
            Some(IDP) => "IDP-30-reserved-subdomain".to_owned(),
            _ => "dev".to_owned(),
        }
    }

    /// GEN-817's mock.css (differs), index.html (same) and IDP-30's mock.css (twice), plus 4 files going in plainly.
    fn tab() -> ConflictsTab {
        let file = |at: &str, relative: &str| Source { path: Path::new(at).join(relative), checkout: checkout(at), kind: SourceKind::File };
        let sources = vec![
            file(GEN, "mock/mock.css"),
            file(GEN, "mock/index.html"),
            file(IDP, "mock/mock.css"),
            file(GEN, "run.sh"),
            file(GEN, "Cargo.toml"),
            file(GEN, "main.rs"),
            file(GEN, "theme.rs"),
        ];
        let incoming = sources.iter().enumerate().map(|(top, source)| Incoming { top, path: source.path.clone(), relative: PathBuf::new(), kind: IncomingKind::File, size: 1 }).collect();
        let target = DropPlace { checkout: checkout("/a"), folder: PathBuf::from("/a/internal-apps/ive/mock") };
        let taken = |path: &Path| ["mock.css", "index.html"].iter().any(|name| path == Path::new("/a/internal-apps/ive/mock").join(name));
        let existing = move |path: &Path| taken(path).then_some(Existing::File { size: 1, modified: SystemTime::UNIX_EPOCH });
        let mut planned = plan(Operation { verb: Verb::Copy, sources, target, layout: Layout::Flat }, incoming, existing);
        planned.rows[0].conflict = Some(Conflict::Differs { added: 1, removed: 0 });
        planned.rows[1].conflict = Some(Conflict::Same);
        planned.rows[1].action = Some(Action::Skip);
        ConflictsTab::new(planned, Default::default())
    }

    #[test]
    fn should_offer_only_skip_and_keep_both_for_twice() {
        // given
        let mut tab = tab();

        // when
        tab.set_action(2, Action::Replace);

        // then
        assert_eq!(tab.plan.rows[2].action, None);
        assert_eq!(tab.offered(2), [Action::Skip, Action::KeepBoth]);
    }

    #[test]
    fn should_disable_apply_until_every_conflict_decided() {
        // given
        let mut tab = tab();
        let before = tab.can_apply();

        // when
        tab.set_action(0, Action::Replace);
        let between = tab.can_apply();
        tab.set_action(2, Action::KeepBoth);

        // then
        assert_eq!((before, between, tab.can_apply()), (false, false, true));
    }

    #[test]
    fn should_name_header_as_spec() {
        // when
        let (title, hint) = tab().header(label);

        // then
        assert_eq!(title, "Copy 7 files from GEN-817-ive, IDP-30-reserved-subdomain to dev · internal-apps/ive/mock");
        assert_eq!(hint, "3 files are in the way · flat: every file straight into mock/ · nothing is written until you press Copy");
    }

    #[test]
    fn should_say_why_each_conflict() {
        // given
        let tab = tab();

        // when
        let why: Vec<String> = [0, 1, 2].iter().map(|index| tab.why(*index, label)).collect();

        // then
        assert_eq!(why, ["differs: +1 −0 lines", "same content", "also coming from GEN-817-ive · mock/mock.css"]);
    }

    #[test]
    fn should_toggle_unfolded_row() {
        // given
        let mut tab = tab();

        // when
        tab.toggle_row(0);
        let open = tab.is_unfolded(0);
        tab.toggle_row(0);

        // then
        assert_eq!((open, tab.is_unfolded(0)), (true, false));
    }
}
