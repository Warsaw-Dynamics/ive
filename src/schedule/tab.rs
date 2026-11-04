//! The Scheduled commands tab (what came due, to pick and run) and Settings › Scheduled (everything, and what
//! became of it). Nothing here runs a command: the workspace does, on Run selected.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use gpui::{AnyElement, Context, Div, FontWeight, SharedString, Stateful, div, prelude::*, px, rgb};

use super::target::Plan;
use super::{Entry, Schedule, State, clock_text};
use crate::click_targets::target;
use crate::selectable::selectable;
use crate::theme::Theme;
use crate::workspace::schedules::{StartedHere, TabPlace, where_text};
use crate::workspace::Workspace;

const MAX_WIDTH: f32 = 1100.;
const CHECKBOX_SIZE: f32 = 14.;
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

/// What the tab and the page draw from.
pub struct View<'a> {
    pub schedule: &'a Schedule,
    pub plans: &'a HashMap<u64, Plan>,
    pub starting: &'a HashSet<u64>,
    pub picked: &'a BTreeSet<u64>,
    pub always: &'a BTreeSet<u64>,
    pub started: &'a [StartedHere],
    pub now: i64,
    pub path: Option<&'a Path>,
}

impl View<'_> {
    fn is_runnable(&self, entry: &Entry) -> bool {
        self.plans.get(&entry.id).is_none_or(Plan::is_runnable)
    }

    /// Due commands, those that can run first, each part oldest first.
    fn due(&self) -> Vec<&Entry> {
        let mut due = self.schedule.due();
        due.sort_by_key(|entry| !self.is_runnable(entry));
        due
    }
}

/// The tab: the title, each due command with its checkbox, Run selected and Skip selected, then what started.
pub fn render_tab(view: View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let due = view.due();
    let title = match due.len() {
        0 => "Nothing is due".to_owned(),
        1 => "A scheduled command is due".to_owned(),
        n => format!("{n} scheduled commands are due"),
    };
    let hint = "Scheduled over MCP. Nothing runs until you tick it and press Run selected · each runs in a new terminal";
    let rows: Vec<AnyElement> = due.iter().map(|entry| due_row(entry, &view, theme, cx)).collect();
    div()
        .id("scheduled")
        .flex()
        .flex_col()
        .size_full()
        .max_w(px(MAX_WIDTH))
        .px(px(16.))
        .pt(px(12.))
        .pb(px(24.))
        .gap_2()
        .overflow_y_scroll()
        .text_size(theme.sizes.tree.text_size.px())
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(muted(hint, theme))
        .when(!due.is_empty(), |tab| tab.child(all_row(&view, &due, theme, cx)).children(rows).child(buttons(&view, theme, cx)))
        .children(started(&view, theme, cx))
        .child(footer(&view, theme, cx))
        .into_any_element()
}

fn all_row(view: &View<'_>, due: &[&Entry], theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let runnable: Vec<u64> = due.iter().filter(|entry| view.is_runnable(entry)).map(|entry| entry.id).collect();
    let all = !runnable.is_empty() && runnable.iter().all(|id| view.picked.contains(id));
    let pick = target("scheduled-all".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.pick_all_scheduled(cx)), cx);
    div().id("scheduled-all").flex().items_center().gap_2().cursor_pointer().on_click(pick).child(checkbox(Tick::from(all), theme)).child("All")
}

fn due_row(entry: &Entry, view: &View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let id = entry.id;
    let starting = view.starting.contains(&id);
    let runnable = view.is_runnable(entry) && !starting;
    let picked = view.picked.contains(&id);
    let pick = target(format!("scheduled-pick:{id}"), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_scheduled_pick(id, cx)), cx);
    let tick = div().id(("scheduled-pick", id)).flex_none().pt(px(3.)).child(checkbox(if runnable { Tick::from(picked) } else { Tick::Disabled }, theme)).when(runnable, |tick| tick.cursor_pointer().on_click(pick));
    let body = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap_1()
        .child(head(entry, view.now, theme))
        .when(!entry.description.is_empty(), |body| body.child(selectable(format!("scheduled-description-{id}"), entry.description.clone(), cx)))
        .child(where_line(entry, view, starting, theme))
        .child(command_box(entry, theme, cx))
        .child(muted(format!("scheduled {} ({})", relative(entry.created, view.now), clock_text(entry.created)), theme))
        .when(entry.repeat.is_some() && runnable, |body| body.child(always_box(id, view.always.contains(&id), theme, cx)));
    let remove = (!view.is_runnable(entry)).then(|| {
        let click = target(format!("scheduled-remove:{id}"), cx.listener(move |workspace, _event, _window, cx| workspace.remove_failed_scheduled(id, cx)), cx);
        plain_button(("scheduled-remove", id), "Remove", theme).on_click(click)
    });
    div()
        .flex()
        .items_start()
        .gap_3()
        .px_2()
        .py_2()
        .border_t_1()
        .border_color(rgb(theme.border))
        .when(picked, |row| row.bg(rgb(theme.hover)))
        .child(tick)
        .child(body)
        .children(remove)
        .into_any_element()
}

/// The name, `↻ cron` and when it was due.
fn head(entry: &Entry, now: i64, theme: Theme) -> Div {
    let mut due = format!("due {} ({})", relative(entry.at, now), clock_text(entry.at));
    if entry.closed {
        due.push_str(" · IVE was closed then");
    }
    if entry.missed > 0 {
        due.push_str(&format!(" · missed {} more {} while waiting", entry.missed, if entry.missed == 1 { "time" } else { "times" }));
    }
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_2()
        .child(div().font_weight(FontWeight::SEMIBOLD).child(entry.name.clone()))
        .children(entry.repeat.as_ref().map(|repeat| chip(format!("↻ {repeat}"), theme.text, theme)))
        .child(muted(due, theme))
}

/// `project · ⎇ branch · in <path>`, or what Run does first, or why it cannot run, or a start's error.
fn where_line(entry: &Entry, view: &View<'_>, starting: bool, theme: Theme) -> Div {
    let (text, colour) = match (starting, &entry.error, view.plans.get(&entry.id)) {
        (true, _, _) => ("starting…".to_owned(), theme.muted),
        (false, Some(error), _) => (error.clone(), theme.error),
        (false, None, Some(plan)) if plan.is_runnable() => (plan.text(&entry.target.branch), theme.muted),
        (false, None, Some(plan)) => (plan.text(&entry.target.branch), theme.error),
        (false, None, None) => ("reading where it runs…".to_owned(), theme.muted),
    };
    // the plan on a line of its own: a path wraps only at its spaces
    div().flex().flex_col().min_w_0().text_size(theme.sizes.git.text_size.px()).child(where_text(&entry.target)).child(div().w_full().min_w_0().text_color(rgb(colour)).child(text))
}

/// The whole command, wrapped, never cut: what the user agrees to run.
fn command_box(entry: &Entry, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(rgb(theme.border))
        .bg(rgb(theme.terminal.background))
        .text_color(rgb(theme.terminal.foreground))
        .font_family(crate::theme::mono_font(cx))
        .text_size(theme.sizes.git.text_size.px())
        .whitespace_normal()
        .child(selectable(format!("scheduled-command-{}", entry.id), entry.command.clone(), cx))
}

fn always_box(id: u64, ticked: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let toggle = target(format!("scheduled-always:{id}"), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_scheduled_always(id, cx)), cx);
    div()
        .id(("scheduled-always", id))
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .on_click(toggle)
        .child(checkbox(Tick::from(ticked), theme))
        .child(muted("Always run this one: its later times start without this tab while the command stays exactly this", theme))
}

fn buttons(view: &View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let count = view.picked.len();
    let ready = count > 0;
    let label = if ready { format!("Run selected ({count})") } else { "Run selected".to_owned() };
    let run = target("scheduled-run".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.run_picked_scheduled(window, cx)), cx);
    let skip = target("scheduled-skip".to_owned(), cx.listener(|workspace, _event, _window, cx| workspace.skip_picked_scheduled(cx)), cx);
    let (background, colour) = if ready { (theme.accent, theme.on_accent) } else { (theme.hover, theme.muted) };
    let run_button = div().id("scheduled-run").px_3().py_1().rounded_md().bg(rgb(background)).text_color(rgb(colour)).child(label).when(ready, |button| button.cursor_pointer().on_click(run));
    let skip_button = plain_button("scheduled-skip", "Skip selected", theme).map(|button| match ready {
        true => button.on_click(skip),
        false => button.cursor_default().text_color(rgb(theme.muted)),
    });
    div().flex().items_center().gap_2().pt_3().child(run_button).child(skip_button).child(muted("Unticked ones stay due", theme))
}

fn started(view: &View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Option<Div> {
    if view.started.is_empty() {
        return None;
    }
    let rows = view.started.iter().map(|started| {
        let process = started.process.value();
        let show = target(format!("scheduled-started:{process}"), cx.listener(move |workspace, _event, window, cx| workspace.show_scheduled_process(process, window, cx)), cx);
        div()
            .id(("scheduled-started", process))
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .h(theme.sizes.tree.row_height.px())
            .rounded_md()
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.hover)))
            .on_click(show)
            .child(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().text_ellipsis().child(format!("{} · {}", started.name, started.where_)))
            .child(muted("show terminal", theme))
    });
    Some(div().flex().flex_col().pt_4().child(muted("Started", theme)).children(rows))
}

fn footer(view: &View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let open = target("scheduled-settings".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.open_settings_page(crate::settings_page::SettingsPage::Scheduled, window, cx)), cx);
    let link = div().id("scheduled-settings").cursor_pointer().text_color(rgb(theme.accent)).child("Settings › Scheduled").on_click(open);
    div().flex().items_center().gap_1().pt_4().child(muted(format!("{} more scheduled ·", view.schedule.pending().len()), theme)).child(link)
}

/// Settings › Scheduled: Due (with Review), Coming (with Cancel and Ask again), and the last days.
pub fn render_settings(view: View<'_>, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    let intro = "Commands agents scheduled over MCP (schedule_command). When one comes due IVE opens the Scheduled commands tab; nothing runs until you tick it there, unless you allowed that exact command to run unasked. Missed while IVE was closed: due when it starts.";
    let due = view.due();
    let pending = view.schedule.pending();
    let history = view.schedule.history(view.now);
    let review = target("scheduled-review".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.open_scheduled_tab(TabPlace::Shown, window, cx)), cx);
    let due_section = (!due.is_empty()).then(|| {
        let link = div().id("scheduled-review").cursor_pointer().text_color(rgb(theme.accent)).child("Review").on_click(review);
        let rows = due.iter().map(|entry| summary_row(entry, format!("{} · due {}", where_text(&entry.target), relative(entry.at, view.now)), theme, cx));
        div().flex().flex_col().child(section_head(format!("Due ({})", due.len()), theme).child(link)).children(rows)
    });
    let pending_rows: Vec<AnyElement> = pending.iter().map(|entry| coming_row(entry, view.now, theme, cx)).collect();
    let history_rows: Vec<AnyElement> = history.iter().map(|entry| history_row(entry, view.now, theme, cx)).collect();
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_size(theme.sizes.settings.title_text_size.px()).font_weight(theme.sizes.settings.title_weight.font_weight()).child("Scheduled"))
        .child(div().text_color(rgb(theme.muted)).child(selectable("scheduled-intro", intro, cx)))
        .children(due_section)
        .child(section_head(format!("Coming ({})", pending.len()), theme))
        .when(pending.is_empty(), |page| page.child(muted("Nothing scheduled.", theme)))
        .children(pending_rows)
        .child(section_head("Last 7 days".to_owned(), theme))
        .when(history.is_empty(), |page| page.child(muted("Nothing yet.", theme)))
        .children(history_rows)
}

/// Where Settings › Scheduled says the schedule is saved, in place of the settings file.
pub fn saved_to(view: &View<'_>) -> Option<String> {
    view.path.map(|path| format!("Saved to {}", path.display()))
}

fn section_head(text: String, theme: Theme) -> Div {
    div().flex().items_center().gap_3().pt_4().pb_1().font_weight(FontWeight::SEMIBOLD).text_size(theme.sizes.settings.text_size.px()).child(text)
}

fn summary_row(entry: &Entry, detail: String, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .py_1()
        .border_t_1()
        .border_color(rgb(theme.border))
        .child(div().flex().flex_wrap().items_center().gap_2().child(div().font_weight(FontWeight::SEMIBOLD).child(entry.name.clone())).child(muted(detail, theme)))
        .child(one_line_command(entry, theme, cx))
        .into_any_element()
}

fn coming_row(entry: &Entry, now: i64, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let id = entry.id;
    let cancel = target(format!("scheduled-cancel:{id}"), cx.listener(move |workspace, _event, _window, cx| {
        let _ = workspace.cancel_scheduled(id, cx);
    }), cx);
    let cancel_link = div().id(("scheduled-cancel", id)).ml_auto().cursor_pointer().text_color(rgb(theme.accent)).child("Cancel").on_click(cancel);
    let unasked = entry.runs_unasked().then(|| {
        let again = target(format!("scheduled-ask-again:{id}"), cx.listener(move |workspace, _event, _window, cx| workspace.ask_again_scheduled(id, cx)), cx);
        let link = div().id(("scheduled-ask-again", id)).cursor_pointer().text_color(rgb(theme.accent)).child("Ask again").on_click(again);
        div().flex().items_center().gap_2().child(chip("runs unasked".to_owned(), theme.attention, theme)).child(link)
    });
    let when = format!("{} · {} ({})", where_text(&entry.target), clock_text(entry.at), relative(entry.at, now));
    let head = div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_2()
        .child(div().font_weight(FontWeight::SEMIBOLD).child(entry.name.clone()))
        .children(entry.repeat.as_ref().map(|repeat| chip(format!("↻ {repeat}"), theme.text, theme)))
        .children(unasked)
        .child(muted(when, theme))
        .child(cancel_link);
    div()
        .flex()
        .flex_col()
        .gap_1()
        .py_1()
        .border_t_1()
        .border_color(rgb(theme.border))
        .child(head)
        .when(!entry.description.is_empty(), |row| row.child(div().child(entry.description.clone())))
        .child(one_line_command(entry, theme, cx))
        .into_any_element()
}

fn history_row(entry: &Entry, now: i64, theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let state = match (entry.state, entry.unasked) {
        (State::Started, true) => "started, unasked".to_owned(),
        (state, _) => state.label().to_owned(),
    };
    let colour = match entry.state {
        State::Started => theme.terminal.ansi[2],
        State::Failed => theme.error,
        _ => theme.muted,
    };
    let mut text = format!("{} · {}", entry.name, where_text(&entry.target));
    if let Some(error) = &entry.error {
        text.push_str(&format!(" · {error}"));
    }
    let done = entry.done.map_or_else(String::new, |done| relative(done, now));
    let row = div()
        .id(("scheduled-history", entry.id))
        .flex()
        .items_center()
        .gap_3()
        .px_1()
        .h(theme.sizes.tree.row_height.px())
        .child(div().flex_none().w(px(150.)).whitespace_nowrap().text_color(rgb(colour)).child(state))
        .child(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().text_ellipsis().child(text))
        .child(muted(done, theme));
    let Some(process) = entry.process else {
        return row.into_any_element();
    };
    let show = target(format!("scheduled-history:{}", entry.id), cx.listener(move |workspace, _event, window, cx| workspace.show_scheduled_process(process, window, cx)), cx);
    row.cursor_pointer().rounded_md().hover(|style| style.bg(rgb(theme.hover))).on_click(show).into_any_element()
}

fn one_line_command(entry: &Entry, theme: Theme, cx: &mut Context<Workspace>) -> Div {
    div().overflow_hidden().whitespace_nowrap().text_ellipsis().font_family(crate::theme::mono_font(cx)).text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.muted)).child(entry.command.clone())
}

/// A checkbox's look; only the row around it takes the click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tick {
    On,
    Off,
    Disabled,
}

impl From<bool> for Tick {
    fn from(on: bool) -> Self {
        if on {
            return Self::On;
        }
        Self::Off
    }
}

fn checkbox(tick: Tick, theme: Theme) -> Div {
    let size = px(CHECKBOX_SIZE);
    let base = div().flex().flex_none().items_center().justify_center().size(size).rounded(px(4.)).border_1().text_size(px(11.));
    match tick {
        Tick::On => base.bg(rgb(theme.accent)).border_color(rgb(theme.accent)).text_color(rgb(theme.on_accent)).child("✓"),
        Tick::Off => base.bg(rgb(theme.background)).border_color(rgb(theme.muted)),
        Tick::Disabled => base.bg(rgb(theme.hover)).border_color(rgb(theme.border)),
    }
}

fn chip(text: String, colour: u32, theme: Theme) -> Div {
    div().px_1p5().rounded(px(4.)).border_1().border_color(rgb(colour)).text_color(rgb(colour)).text_size(theme.sizes.git.text_size.px()).child(text)
}

fn plain_button(id: impl Into<gpui::ElementId>, label: &'static str, theme: Theme) -> Stateful<Div> {
    div().id(id).flex_none().px_3().py_1().rounded_md().cursor_pointer().bg(rgb(theme.hover)).text_color(rgb(theme.text)).child(label)
}

fn muted(text: impl Into<SharedString>, theme: Theme) -> Div {
    div().text_size(theme.sizes.git.text_size.px()).text_color(rgb(theme.muted)).child(text.into())
}

/// `in 2 h 5 min`, `30 min ago`, `just now`, `in 3 d 4 h`.
pub fn relative(seconds: i64, now: i64) -> String {
    let delta = seconds - now;
    let span = delta.abs();
    if span < MINUTE {
        return "just now".to_owned();
    }
    let text = if span < HOUR {
        format!("{} min", span / MINUTE)
    } else if span < DAY {
        joined(span / HOUR, "h", span % HOUR / MINUTE, "min")
    } else {
        joined(span / DAY, "d", span % DAY / HOUR, "h")
    };
    if delta > 0 {
        return format!("in {text}");
    }
    format!("{text} ago")
}

fn joined(major: i64, major_unit: &str, minor: i64, minor_unit: &str) -> String {
    if minor == 0 {
        return format!("{major} {major_unit}");
    }
    format!("{major} {major_unit} {minor} {minor_unit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_say_how_far_a_time_is() {
        // given
        let now = 1_000_000;
        // when
        let texts = [relative(now + 20, now), relative(now - 30 * MINUTE, now), relative(now + 2 * HOUR + 5 * MINUTE, now), relative(now - 3 * DAY, now), relative(now + DAY + 4 * HOUR, now)];
        // then
        assert_eq!(texts, ["just now", "30 min ago", "in 2 h 5 min", "3 d ago", "in 1 d 4 h"]);
    }
}
