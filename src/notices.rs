//! Failures away from any tab: a notice at the bottom right of the main window until closed, a line in `ive.log`,
//! and stderr. Before this, many of them went to stderr alone, which nobody sees when IVE starts from the desktop.

use std::path::PathBuf;
use std::sync::OnceLock;

use gpui::{App, ClipboardItem, Div, Global, div, prelude::*, px, rgb};

use crate::age::{self, Clock};
use crate::click_targets::target;
use crate::script::log::{MAX_LOG_BYTES, ScriptLog};
use crate::selectable::selectable;
use crate::theme::Theme;

/// More would cover the window; the oldest goes first, and `ive.log` keeps them all.
const MAX_NOTICES: usize = 5;

/// The app log for failures raised where no window can be reached.
static APP_LOG: OnceLock<ScriptLog> = OnceLock::new();

/// `ive.log` next to `scripts.log`.
pub fn default_log_path() -> PathBuf {
    crate::script::log::default_path().with_file_name("ive.log")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// when it happened, local time
    pub time: String,
    pub text: String,
}

/// The open notices, oldest first, and the log every notice is also written to.
pub struct Notices {
    list: Vec<Notice>,
    log: ScriptLog,
}

impl Global for Notices {}

impl Notices {
    pub fn new(log: ScriptLog) -> Self {
        Self { list: Vec::new(), log }
    }

    /// Keeps `text` as a notice, dropping the oldest past five, and writes it to the log.
    pub fn add(&mut self, text: String) {
        self.log.append(&text);
        self.list.push(Notice { time: age::date_time(Clock::system().when()), text });
        let excess = self.list.len().saturating_sub(MAX_NOTICES);
        self.list.drain(..excess);
    }

    /// Closes the `index`th notice, counted newest first as they are shown.
    pub fn dismiss(&mut self, index: usize) {
        if let Some(position) = self.list.len().checked_sub(index + 1) {
            self.list.remove(position);
        }
    }

    pub fn newest_first(&self) -> impl Iterator<Item = &Notice> {
        self.list.iter().rev()
    }

    /// The `notices` MCP tool's answer: the open notices, newest first, and the log's path.
    pub fn json(&self) -> serde_json::Value {
        let notices: Vec<serde_json::Value> = self.newest_first().map(|notice| serde_json::json!({ "time": notice.time, "text": notice.text })).collect();
        serde_json::json!({ "notices": notices, "log_file": self.log.path() })
    }
}

/// Makes the notices global and the app log reachable from any thread.
pub fn init(cx: &mut App) {
    let log = ScriptLog::new(default_log_path(), MAX_LOG_BYTES);
    let _ = APP_LOG.set(log.clone());
    cx.set_global(Notices::new(log));
}

/// Shows `text` as a notice, writes it to `ive.log` and to stderr; only logs it before [`init`].
pub fn notify(cx: &mut App, text: impl Into<String>) {
    let text = text.into();
    if cx.has_global::<Notices>() {
        eprintln!("ive: {text}");
        cx.update_global::<Notices, _>(|notices, _| notices.add(text));
        return cx.refresh_windows();
    }
    log_only(&text);
}

/// Writes `text` to `ive.log` and stderr, for a failure raised where no window can be reached.
pub fn log_only(text: &str) {
    eprintln!("ive: {text}");
    if let Some(log) = APP_LOG.get() {
        log.append(text);
    }
}

/// The open notices, stacked at the bottom right; none when there are none. Each has its time, its text
/// (selectable), Copy (click target `notice-copy:<n>`) and × (`notice-close:<n>`), counted newest first.
pub fn render(cx: &App) -> Option<Div> {
    let notices: Vec<Notice> = cx.try_global::<Notices>()?.newest_first().cloned().collect();
    if notices.is_empty() {
        return None;
    }
    let theme = *cx.global::<Theme>();
    let panels: Vec<Div> = notices.into_iter().enumerate().map(|(index, notice)| render_notice(index, notice, theme, cx)).collect();
    Some(div().absolute().bottom(px(16.)).right(px(16.)).w(px(420.)).flex().flex_col_reverse().gap_2().children(panels))
}

fn render_notice(index: usize, notice: Notice, theme: Theme, cx: &App) -> Div {
    let copied = format!("{} {}", notice.time, notice.text);
    let copy = div()
        .id(("notice-copy", index))
        .cursor_pointer()
        .text_color(rgb(theme.accent))
        .hover(|style| style.underline())
        .child("Copy")
        .on_click(target(format!("notice-copy:{index}"), move |_event, _window, cx| cx.write_to_clipboard(ClipboardItem::new_string(copied.clone())), cx));
    let close = div()
        .id(("notice-close", index))
        .cursor_pointer()
        .text_color(rgb(theme.muted))
        .hover(|style| style.text_color(rgb(theme.text)))
        .child("×")
        .on_click(target(format!("notice-close:{index}"), move |_event, window, cx| {
            cx.update_global::<Notices, _>(|notices, _| notices.dismiss(index));
            window.refresh();
        }, cx));
    let head = div().flex().items_center().gap_3().text_xs().child(div().flex_1().text_color(rgb(theme.muted)).child(notice.time.clone())).child(copy).child(close);
    let text = selectable(format!("notice-{}-{}", notice.time, notice.text.len()), notice.text, cx).text_color(rgb(theme.error));
    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(theme.error))
        .bg(rgb(theme.sidebar))
        .shadow_md()
        .text_sm()
        .child(head)
        .child(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notices_in(dir: &std::path::Path) -> Notices {
        Notices::new(ScriptLog::new(dir.join("ive.log"), 64 * 1024))
    }

    #[test]
    fn should_keep_five_newest_notices() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut notices = notices_in(dir.path());

        // when
        for index in 0..6 {
            notices.add(format!("failure {index}"));
        }

        // then
        let texts: Vec<&str> = notices.newest_first().map(|notice| notice.text.as_str()).collect();
        assert_eq!(texts, ["failure 5", "failure 4", "failure 3", "failure 2", "failure 1"]);
    }

    #[test]
    fn should_append_notice_to_log() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut notices = notices_in(dir.path());

        // when
        notices.add("cannot reattach terminal t1: gone".to_owned());

        // then
        let log = std::fs::read_to_string(dir.path().join("ive.log")).unwrap();
        assert!(log.trim_end().ends_with("cannot reattach terminal t1: gone"), "{log}");
    }

    #[test]
    fn should_describe_notices_for_mcp() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut notices = notices_in(dir.path());
        notices.add("first".to_owned());
        notices.add("second".to_owned());

        // when
        let described = notices.json();

        // then
        let texts: Vec<&str> = described["notices"].as_array().unwrap().iter().map(|notice| notice["text"].as_str().unwrap()).collect();
        assert_eq!(texts, ["second", "first"]);
        assert!(described["log_file"].as_str().unwrap().ends_with("ive.log"));
    }

    #[test]
    fn should_dismiss_notice() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut notices = notices_in(dir.path());
        notices.add("first".to_owned());
        notices.add("second".to_owned());

        // when
        notices.dismiss(0);

        // then
        let texts: Vec<&str> = notices.newest_first().map(|notice| notice.text.as_str()).collect();
        assert_eq!(texts, ["first"]);
    }
}
