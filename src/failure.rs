//! What failed and what pins it down: a summary, labelled details, and the output a program left, shown selectable in
//! failed tabs and notices, and copied or read over MCP as one plain-text report.

use gpui::{App, ClipboardItem, Div, FontWeight, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::selectable::selectable;
use crate::theme::{self, Theme};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    summary: String,
    details: Vec<(&'static str, String)>,
    /// the last lines a program printed, oldest first
    output: Vec<String>,
}

impl Failure {
    pub fn new(summary: impl Into<String>) -> Self {
        Self { summary: summary.into(), details: Vec::new(), output: Vec::new() }
    }

    /// Adds a labelled detail, e.g. `Folder: /home/me/p`.
    pub fn with(mut self, label: &'static str, value: impl Into<String>) -> Self {
        self.details.push((label, value.into()));
        self
    }

    pub fn with_output(self, output: Vec<String>) -> Self {
        Self { output, ..self }
    }

    /// The whole failure as text to paste into a search or hand to an agent: what failed, its details, the output,
    /// and which IVE on which OS.
    pub fn report(&self, title: &str) -> String {
        let mut report = format!("{title}: {}\n\n", self.summary);
        if !self.details.is_empty() {
            let details: Vec<String> = self.details.iter().map(|(label, value)| format!("{label}: {value}\n")).collect();
            report.push_str(&format!("{}\n", details.concat()));
        }
        if !self.output.is_empty() {
            report.push_str(&format!("Output (last {} lines):\n{}\n\n", self.output.len(), self.output.join("\n")));
        }
        report.push_str(&format!("{}\nOS: {}\n", crate::build_info::summary(), std::env::consts::OS));
        report
    }

    /// The failure as a page: the summary in the error colour, the details, the output, and Copy error (click target
    /// `copy-error`); every text selectable, as blocks named after `key`.
    pub fn render(&self, key: &str, title: &str, cx: &App) -> Div {
        let theme = *cx.global::<Theme>();
        let report = self.report(title);
        let copy = div()
            .id("copy-error")
            .flex_none()
            .text_color(rgb(theme.accent))
            .cursor_pointer()
            .hover(|style| style.underline())
            .child("Copy error")
            .on_click(target("copy-error".to_owned(), move |_event, _window, cx| cx.write_to_clipboard(ClipboardItem::new_string(report.clone())), cx));
        let summary = selectable(format!("failure-summary-{key}"), self.summary.clone(), cx).text_color(rgb(theme.error)).font_weight(FontWeight::SEMIBOLD);
        let details = self.details.iter().enumerate().map(|(index, (label, value))| {
            let value = selectable(format!("failure-detail-{key}-{index}"), value.clone(), cx).flex_1().min_w_0().font_family(theme::mono_font(cx));
            div().flex().gap_3().child(div().flex_none().w_24().text_color(rgb(theme.muted)).child(*label)).child(value)
        });
        let output = (!self.output.is_empty()).then(|| {
            let lines = selectable(format!("failure-output-{key}"), self.output.join("\n"), cx).font_family(theme::mono_font(cx));
            div().p_2().rounded_md().border_1().border_color(rgb(theme.border)).bg(rgb(theme.background)).child(lines)
        });
        let details: Vec<Div> = details.collect();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .max_w(gpui::px(960.))
            .child(div().flex().items_start().gap_4().child(summary.flex_1().min_w_0()).child(copy))
            .children((!details.is_empty()).then(|| div().flex().flex_col().gap_1().children(details)))
            .children(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_write_report_with_details_and_output() {
        // given
        let failure = Failure::new("The shell exited right after start").with("Folder", "/p").with("Exit", "SIGSEGV (11)").with_output(vec!["boom".to_owned()]);

        // when
        let report = failure.report("Terminal");

        // then
        let expected_start = "Terminal: The shell exited right after start\n\nFolder: /p\nExit: SIGSEGV (11)\n\nOutput (last 1 lines):\nboom\n\n";
        assert!(report.starts_with(expected_start), "{report}");
        assert!(report.contains(&crate::build_info::summary()), "{report}");
        assert!(report.ends_with(&format!("OS: {}\n", std::env::consts::OS)), "{report}");
    }

    #[test]
    fn should_write_report_without_output() {
        // given
        let failure = Failure::new("cannot read /a: gone");

        // when
        let report = failure.report("a");

        // then
        assert!(report.starts_with("a: cannot read /a: gone\n\nIVE "), "{report}");
        assert!(!report.contains("Output"), "{report}");
    }
}
