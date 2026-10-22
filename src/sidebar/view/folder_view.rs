//! Draws a worktree's folders and what the git folders list, and the badges at a checkout row's end.

use std::rc::Rc;

use gpui::{Context, Div, Pixels, Stateful, div, prelude::*, px, rgb};

use super::{RowAction, RowLine, RowMouse, RunActionsIn, Shown, bind_row, chevron, count_badge, dot_note, keep_shown, row_action, row_actions, run_actions, run_label, secondary, shared};
use crate::click_targets::{Listener, target};
use crate::clipboard;
use crate::icons;
use crate::overview::git_ui::mark_label;
use crate::project::CheckoutRef;
use crate::script::keymap::ClipboardTarget;
use crate::sidebar::folders::{ChangeFolderRow, ChangeRow, CommitRow, CommitsMoreRow, FolderBadge, FolderKind, FolderRow, NoteRow};
use crate::sidebar::rows::{RunRow, target_name};
use crate::theme::Theme;
use crate::tooltip;
use crate::workspace::{Workspace, icon};

/// A folder: its chevron (an empty one keeps the slot), icon, name and count. An empty git folder is muted, says `· 0`
/// or `· no base`, and why in its tooltip. A click on Processes opens the checkout's Run configurations tab, its
/// chevron folds it, its + starts a terminal; other folders fold on both.
pub(super) fn folder_row(line: RowLine, row: &FolderRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let kind = row.kind;
    let processes = kind == FolderKind::Processes;
    let toggled = row.checkout.clone();
    let fold = cx.listener(move |workspace, _event, _window, cx| workspace.toggle_folder_folded(&toggled, kind, cx));
    // Processes' click opens its tab, so its chevron needs a target of its own; an empty one has no chevron
    let toggle: Listener = match (processes, row.empty) {
        (true, true) => Rc::new(fold),
        (true, false) => Rc::new(target(line.name("toggle"), fold, cx)),
        (false, _) => Rc::new(target(line.name("folder"), fold, cx)),
    };
    let new_terminal = processes.then(|| {
        let started = row.checkout.clone();
        let action = RowAction {
            id: "new-terminal",
            icon: icons::ADD,
            tip: "New terminal (type claude to start Claude Code)".to_owned(),
            on_click: Box::new(target(line.name("new-terminal"), cx.listener(move |workspace, _event, window, cx| {
                workspace.open_terminal_in(Some(started.clone()), window, cx);
            }), cx)),
        };
        row_action(action, &line)
    });
    let (activate, keep) = match processes {
        true => {
            let shown = row.checkout.clone();
            let show: Listener = Rc::new(target(line.name("folder"), cx.listener(move |workspace, _event, window, cx| workspace.show_run_configs(&shown, window, cx)), cx));
            (show, keep_shown(&line.select, cx))
        }
        false => (toggle.clone(), None),
    };
    let toggles = (!row.empty).then(|| toggle.clone());
    let mouse = RowMouse { select: line.select.clone(), name: line.name("folder"), checkout: Some(row.checkout.path.clone()), activate, keep, toggle: toggles };
    let disclosure = match row.empty {
        true => slot(theme).into_any_element(),
        false => chevron(row.expanded, theme, shared(&toggle)).into_any_element(),
    };
    let line = line
        .div
        .child(disclosure)
        .child(div().flex_none().when(row.nothing.is_some(), |slot| slot.opacity(MUTED_ICON)).child(icon(theme, kind.icon())))
        .child(kind.title())
        .children(row.count.filter(|count| *count > 0).map(|count| count_badge(count, theme)))
        .children(new_terminal);
    let line = match &row.nothing {
        Some(nothing) => line.text_color(rgb(secondary(theme))).child(dot_note(nothing.mark.to_owned(), theme)).tooltip(tooltip::text_tooltip(nothing.tip.clone())),
        None => line,
    };
    bind_row(line, mouse, cx)
}

/// An empty git folder's icon, as faded as its muted name.
const MUTED_ICON: f32 = 0.6;

/// A chevron's width of nothing, so icons line up with their siblings'.
pub(super) fn slot(theme: Theme) -> Div {
    div().flex_none().size(theme.sizes.icon.px())
}

/// A changed file: its name (struck through when deleted), its mark, its folder in the flat layout; a click opens its
/// diff.
pub(super) fn change_row(line: RowLine, row: &ChangeRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let (checkout, source, change) = (row.checkout.clone(), row.source.clone(), row.change.clone());
    let open: Listener = Rc::new(target(line.name("change"), cx.listener(move |workspace, _event, window, cx| {
        workspace.open_change(&checkout, &source, &change, window, cx);
    }), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("change"), checkout: Some(row.checkout.path.clone()), activate: open, keep: None, toggle: None };
    let name = row.change.path.file_name();
    let deleted = row.change.mark == crate::git::types::Mark::Deleted;
    let folder = row.folder.clone().map(|folder| div().text_color(rgb(secondary(theme))).child(folder));
    let line = line
        .div
        .child(slot(theme))
        .child(icon(theme, icons::file_icon(&name)))
        .child(div().when(deleted, |name| name.line_through()).child(name))
        .child(mark_label(row.change.mark, theme, cx))
        .children(folder)
        .tooltip(tooltip::text_tooltip(row.change.path.display()));
    bind_row(line, mouse, cx)
}

/// A folder of changed files in the tree layout, with how many it holds.
pub(super) fn change_folder_row(line: RowLine, row: &ChangeFolderRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let (checkout, source, path) = (row.checkout.clone(), row.source.clone(), row.path.clone());
    let toggle: Listener = Rc::new(target(line.name("change-folder"), cx.listener(move |workspace, _event, _window, cx| {
        workspace.toggle_change_folder(&checkout, source.clone(), path.clone(), cx);
    }), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("change-folder"), checkout: Some(row.checkout.path.clone()), activate: toggle.clone(), keep: None, toggle: Some(toggle.clone()) };
    let line = line
        .div
        .child(chevron(row.expanded, theme, shared(&toggle)))
        .child(icon(theme, icons::FOLDER))
        .child(row.label.clone())
        .children((row.files > 1).then(|| count_badge(row.files, theme)))
        .tooltip(tooltip::text_tooltip(row.path.clone()));
    bind_row(line, mouse, cx)
}

/// A commit: a click, or its chevron, folds its files out and in.
pub(super) fn commit_row(line: RowLine, row: &CommitRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let (toggled, sha) = (row.checkout.clone(), row.sha.clone());
    let toggle: Listener = Rc::new(target(line.name("commit"), cx.listener(move |workspace, _event, _window, cx| {
        workspace.toggle_commit(&toggled, &sha, cx);
    }), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("commit"), checkout: Some(row.checkout.path.clone()), activate: toggle.clone(), keep: None, toggle: Some(toggle.clone()) };
    let line = line
        .div
        .child(chevron(row.expanded, theme, shared(&toggle)))
        .child(icon(theme, icons::COMMIT))
        .child(row.label.clone())
        .child(div().text_color(rgb(secondary(theme))).child(row.meta.clone()))
        .tooltip(tooltip::text_tooltip(row.hint.clone()));
    bind_row(line, mouse, cx)
}

/// `Load 20 more · 10 of 32`: lists more commits in place.
pub(super) fn commits_more_row(line: RowLine, row: &CommitsMoreRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let checkout = row.checkout.clone();
    let more: Listener = Rc::new(target(line.name("commits-more"), cx.listener(move |workspace, _event, _window, cx| {
        workspace.show_more_commits(&checkout, cx);
    }), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("commits-more"), checkout: Some(row.checkout.path.clone()), activate: more, keep: None, toggle: None };
    let line = line.div.child(slot(theme)).child(div().text_color(rgb(theme.accent)).child(row.text.clone()));
    bind_row(line, mouse, cx)
}

/// Why an open commit's files could not be read.
pub(super) fn note_row(line: RowLine, row: &NoteRow) -> Stateful<Div> {
    let theme = line.theme;
    line.div.cursor_default().child(slot(theme)).child(div().text_color(rgb(theme.error)).child(row.text.clone()))
}

/// A live run in Processes: its name and state; a click shows it, Restart and Stop on hover.
pub(super) fn live_run_row(line: RowLine, run: &RunRow, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = line.theme;
    let id = run.id;
    let show: Listener = Rc::new(target(line.name("live-run"), cx.listener(move |workspace, _event, window, cx| workspace.show_run(id, window, cx)), cx));
    let mouse = RowMouse { select: line.select.clone(), name: line.name("live-run"), checkout: None, activate: show, keep: None, toggle: None };
    let mut actions = link_actions(run, &line, cx);
    actions.extend(run_actions(run, RunActionsIn::Tree, cx));
    let actions = row_actions(actions, theme, line.group.clone(), Shown::OnHover);
    let kind_icon = if run.serve { icons::FILE_HTML } else { icons::RUN_CONFIG };
    let line = line.div.child(slot(theme)).child(icon(theme, kind_icon)).child(run.name.clone()).child(run_label(run, theme)).child(actions);
    bind_row(line, mouse, cx)
}

/// A running static server's link, as the bar over its output has it: Open in the browser and Copy the link.
fn link_actions(run: &RunRow, line: &RowLine, cx: &mut Context<Workspace>) -> Vec<RowAction> {
    let Some(link) = run.link.clone() else {
        return Vec::new();
    };
    let opened = link.clone();
    let open = target(line.name("open-link"), move |_event, _window, cx| cx.open_url(&opened), cx);
    let copy = target(line.name("copy-link"), move |_event, _window, cx| clipboard::write(ClipboardTarget::Clipboard, link.clone(), cx), cx);
    vec![
        RowAction { id: "open-link", icon: icons::WEB, tip: "Open in the browser".to_owned(), on_click: Box::new(open) },
        RowAction { id: "copy-link", icon: icons::COPY, tip: "Copy the link".to_owned(), on_click: Box::new(copy) },
    ]
}

/// `Commits (5) · this worktree only`, then what a click and an Alt+click do.
fn badge_tip(badge: &FolderBadge) -> String {
    let count = badge.count.map_or_else(String::new, |count| format!(" ({count})"));
    let apart = if badge.apart { " · this worktree only" } else { "" };
    format!("{}{count}{apart}\nClick: this worktree · Alt+click: every worktree", badge.kind.title())
}

/// The mark of a badge set apart from every worktree's: a 4 px accent dot at its top right.
fn apart_dot(theme: Theme) -> Div {
    div().absolute().top(px(1.)).right(px(1.)).size(px(4.)).rounded_full().bg(rgb(theme.accent))
}

/// The badges at the end of a checkout's row (or a single checkout's project row): each its folder's icon and count,
/// highlighted while the folder shows; a click shows or hides it. Shown while the row is hovered, pinned like row
/// actions to the panel's visible right edge, `after` short of it, over a long name.
pub(super) fn folder_badges(badges: &[FolderBadge], checkout: &CheckoutRef, line: &RowLine, after: Pixels, cx: &mut Context<Workspace>) -> Div {
    let theme = line.theme;
    let sizes = &theme.sizes.tree;
    let height = theme.sizes.icon.0 + 2. * sizes.action_padding.0;
    let buttons = badges.iter().map(|badge| {
        let (kind, toggled) = (badge.kind, checkout.clone());
        // Alt (Option on macOS): every worktree
        let toggle = target(target_name("badge", &badge.target), cx.listener(move |workspace, event: &gpui::ClickEvent, _window, cx| {
            match event.modifiers().alt {
                true => workspace.toggle_folder_everywhere(kind, cx),
                false => workspace.toggle_folder_shown(&toggled, kind, cx),
            }
        }), cx);
        let (fill, text) = if badge.shown { (Some(theme.border), theme.text) } else { (None, secondary(theme)) };
        let tip = badge_tip(badge);
        let dot = badge.apart.then(|| apart_dot(theme));
        div()
            .id(("folder-badge", kind as usize))
            .flex()
            .flex_none()
            .items_center()
            .gap(sizes.tag_gap.px())
            .px(sizes.count_padding_x.px())
            .rounded(sizes.count_radius.px())
            .text_size(sizes.count_text_size.px())
            .text_color(rgb(text))
            .map(|button| match fill {
                Some(fill) => button.bg(rgb(fill)),
                None => button,
            })
            .hover(|style| style.bg(rgb(theme.border)))
            .relative()
            .children(dot)
            .child(icon(theme, kind.icon()))
            .children(badge.count.map(|count| count.to_string()))
            .tooltip(tooltip::text_tooltip(tip))
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                toggle(event, window, cx);
            })
    });
    div()
        .absolute()
        .top(px((sizes.row_height.0 - height) / 2.))
        .right(sizes.padding_right.px() + after)
        .h(px(height))
        .rounded(sizes.action_radius.px())
        .bg(rgb(theme.hover))
        .flex()
        .flex_none()
        .items_center()
        .gap_0p5()
        .invisible()
        .group_hover(line.group.clone(), |style| style.visible())
        .children(buttons)
}
