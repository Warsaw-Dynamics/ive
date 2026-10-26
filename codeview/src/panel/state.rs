//! What a panel keeps across a restart of its host.

use serde::{Deserialize, Serialize};

use super::protocol::{DiffLayout, MarkdownView};
use crate::position::{CaretPosition, ScrollPosition};

/// Where a panel was scrolled and what else the user set in it; the host saves it with its tabs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelState {
    pub scroll: ScrollPosition,
    /// in Edit mode and editable diffs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caret: Option<CaretPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<DiffLayout>,
    /// a commit tab's collapsed files, by path
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collapsed: Vec<String>,
    /// a Markdown text's view; none for any other text, or a tab saved before Markdown had views
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<MarkdownView>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use gpui::{AppContext, Entity, TestAppContext};

    use super::*;
    use crate::diff::rows::COLLAPSE_LINES;
    use crate::diff::{DiffBody, DiffLine, FileDiff, GitPath, Headers, Hunk, LineKind, Mark};
    use crate::document::TextSource;
    use crate::style::ViewStyle;
    use crate::panel::{Cast, Content, DiffInput, Editable, EditPolicy, Panel};

    /// `count` numbered lines, the last without a line break.
    fn numbered(count: usize) -> Arc<TextSource> {
        let text: Vec<String> = (1..=count).map(|line| format!("line {line}")).collect();
        Arc::new(TextSource::new(&text.join("\n")))
    }

    fn viewed(count: usize) -> Content {
        Content::Lines { source: numbered(count), language: None, edit: None }
    }

    fn edited(count: usize) -> Content {
        let policy = EditPolicy { save_delay: Duration::from_millis(500), limit: None };
        Content::Lines { source: numbered(count), language: None, edit: Some(policy) }
    }

    fn scrolled_to(top_line: f64) -> PanelState {
        PanelState { scroll: ScrollPosition { top_line, left: 0. }, ..PanelState::default() }
    }

    fn cast(panel: &Entity<Panel>, casts: Vec<Cast>, cx: &mut TestAppContext) -> PanelState {
        panel.update(cx, |panel, cx| {
            for cast in casts {
                panel.handle(cast, cx);
            }
            panel.state()
        })
    }

    #[gpui::test]
    fn should_restore_after_open_or_before_it(cx: &mut TestAppContext) {
        // given
        let (before, after) = (cx.new(Panel::new), cx.new(Panel::new));

        // when
        let restored_first = cast(&before, vec![Cast::Restore(scrolled_to(40.)), Cast::Open(viewed(100))], cx);
        let opened_first = cast(&after, vec![Cast::Open(viewed(100)), Cast::Restore(scrolled_to(40.))], cx);

        // then
        assert_eq!((restored_first.scroll.top_line, opened_first.scroll.top_line), (40., 40.));
    }

    #[gpui::test]
    fn should_clamp_a_restored_position_past_the_end(cx: &mut TestAppContext) {
        // given
        let panel = cx.new(Panel::new);
        let past_the_end = PanelState { caret: Some(CaretPosition { line: 500, column: 0 }), ..scrolled_to(500.) };

        // when
        let state = cast(&panel, vec![Cast::Open(edited(10)), Cast::Restore(past_the_end)], cx);

        // then
        assert!(state.scroll.top_line <= 9., "top {}", state.scroll.top_line);
        assert_eq!(state.caret, Some(CaretPosition { line: 9, column: 0 }));
    }

    /// `text` viewed, or with `edit`, edited.
    fn lines_of(text: &str, edit: Option<EditPolicy>) -> Content {
        Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit }
    }

    #[gpui::test]
    fn should_clamp_a_caret_on_the_editors_empty_last_line_to_each_mode(cx: &mut TestAppContext) {
        // given
        let panel = cx.new(Panel::new);
        let policy = EditPolicy { save_delay: Duration::from_millis(500), limit: None };
        // the editor shows "a\nb\n" as three lines, the last one empty; the viewer as two
        let on_last_line = PanelState { caret: Some(CaretPosition { line: 2, column: 0 }), ..scrolled_to(2.) };

        // when
        let viewed = cast(&panel, vec![Cast::Open(lines_of("a\nb\n", None)), Cast::Restore(on_last_line.clone())], cx);
        let to_editor = cast(&panel, vec![Cast::Open(lines_of("a\nb\n", Some(policy)))], cx);
        let restored = cast(&panel, vec![Cast::Restore(on_last_line.clone())], cx);
        let to_viewer = cast(&panel, vec![Cast::Open(lines_of("a\nb\n", None))], cx);

        // then
        assert_eq!(viewed, scrolled_to(1.));
        assert_eq!(to_editor, PanelState { caret: Some(CaretPosition::default()), ..scrolled_to(1.) });
        assert_eq!(restored, on_last_line);
        assert_eq!(to_viewer, scrolled_to(1.));
    }

    #[gpui::test]
    fn should_hold_a_restore_while_there_is_nothing_to_apply_it_to(cx: &mut TestAppContext) {
        // given
        let panel = cx.new(Panel::new);

        // when
        let state = cast(&panel, vec![Cast::Restore(scrolled_to(7.))], cx);

        // then
        assert_eq!(state, scrolled_to(7.));
    }

    fn diff_file(name: &str, lines: usize) -> FileDiff {
        let lines = (0..lines).map(|n| DiffLine { kind: LineKind::Added, old: None, new: Some(n as u32 + 1), text: n.to_string() }).collect();
        let hunk = Hunk { header: "@@ -0,0 +1 @@".to_owned(), lines };
        FileDiff { path: GitPath::from_bytes(name.as_bytes()), old_path: None, mark: Mark::Added, body: DiffBody::Text { hunks: vec![hunk], cut: None } }
    }

    fn commit_without_files() -> Content {
        Content::Diff(DiffInput { files: Vec::new(), headers: Headers::Shown, layout: DiffLayout::Inline, editable: Editable::No })
    }

    #[gpui::test]
    fn should_keep_the_default_collapse_when_restoring_a_state_without_a_collapsed_list(cx: &mut TestAppContext) {
        // given
        cx.update(ViewStyle::set_for_test);
        let panel = cx.new(Panel::new);
        let files = vec![diff_file("big.rs", COLLAPSE_LINES + 1), diff_file("small.rs", 3)];

        // when
        let state = cast(&panel, vec![Cast::Open(commit_without_files()), Cast::Restore(PanelState::default()), Cast::SetFiles(files)], cx);

        // then
        assert_eq!(state.collapsed, ["big.rs"]);
    }

    #[gpui::test]
    fn should_report_a_restored_collapsed_list_while_the_files_are_not_there_yet(cx: &mut TestAppContext) {
        // given
        cx.update(ViewStyle::set_for_test);
        let panel = cx.new(Panel::new);
        let restored = PanelState { collapsed: vec!["a.rs".to_owned()], ..scrolled_to(5.) };

        // when
        let state = cast(&panel, vec![Cast::Open(commit_without_files()), Cast::Restore(restored.clone())], cx);

        // then
        assert_eq!(state, restored);
    }
}
