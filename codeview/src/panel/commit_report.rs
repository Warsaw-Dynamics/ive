//! The bar saying what went wrong while committing a change, when it must stay in sight whatever HEAD does next
//! ([`super::Cast::ChangeCommitTrouble`]).

use gpui::{App, Div, prelude::*, div};

use super::saving::{Action, Button, bar_row, button};
use crate::style::ViewStyle;

/// The report of the last troubled commit; shown until dismissed or a commit goes through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CommitReport {
    Nothing,
    Shown(String),
}

/// The bar above the text while a report is shown: the report and a button dismissing it, click target
/// `commit-report-dismiss`.
pub(super) fn bar(report: &CommitReport, dismiss: Action, cx: &App) -> Option<Div> {
    let CommitReport::Shown(report) = report else {
        return None;
    };
    let style = ViewStyle::drawing(cx);
    let dismiss = Button { name: "commit-report-dismiss", label: "✕", action: dismiss };
    let text = div().flex_1().min_w_0().child(report.clone());
    Some(bar_row(style).child(text).child(button(dismiss, style, cx)))
}
