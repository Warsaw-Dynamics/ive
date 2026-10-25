//! Files and folders dragged inside IVE (from the project tree); a terminal they are dropped on types their paths.

use std::path::PathBuf;

use gpui::{App, Context, IntoElement, ParentElement, Render, SharedString, Styled, WeakEntity, Window, div, rgb};

use crate::sidebar::rows::TreeKey;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// A drag from the tree: the pressed row, and the workspace whose selection says what else goes with it. The paths
/// are worked out when the drag starts and when it is dropped, not while the tree is drawn.
#[derive(Clone)]
pub struct DraggedFiles {
    pub pressed: TreeKey,
    pub workspace: WeakEntity<Workspace>,
}

impl DraggedFiles {
    /// The pressed row's path, or the whole selection when the row is selected.
    pub fn paths(&self, cx: &App) -> Vec<PathBuf> {
        match self.workspace.upgrade() {
            Some(workspace) => workspace.read(cx).dragged_paths(&self.pressed),
            None => Vec::new(),
        }
    }

    /// What a drop on terminal `process` types: the paths relative to its directory when it works in their checkout.
    pub fn typed_into(&self, process: crate::process::ProcessId, cx: &App) -> String {
        let paths = self.paths(cx);
        match self.workspace.upgrade() {
            Some(workspace) => workspace.read(cx).typed_for_process(&paths, process),
            None => crate::path_text::typed_paths(&paths, crate::path_text::Base::Absolute),
        }
    }

    /// The chip under the pointer.
    pub fn chip(&self, cx: &App) -> DragChip {
        DragChip { label: label(&self.paths(cx)).into(), workspace: self.workspace.clone() }
    }
}

/// A drag of process rows (terminals, Claude Code, runs) from the tree: the pressed row, and the workspace whose
/// selection says what else goes with it.
#[derive(Clone)]
pub struct DraggedProcesses {
    pub pressed: TreeKey,
    pub title: String,
    pub workspace: WeakEntity<Workspace>,
}

impl DraggedProcesses {
    /// The chip under the pointer: the pressed row's title, `+N` for the others picked with it.
    pub fn chip(&self, cx: &App) -> DragChip {
        let count = self.workspace.upgrade().map_or(1, |workspace| workspace.read(cx).dragged_keys(&self.pressed).len());
        let label = match count {
            0 | 1 => self.title.clone(),
            count => format!("{} +{}", self.title, count - 1),
        };
        DragChip { label: label.into(), workspace: self.workspace.clone() }
    }
}

/// The first path's name, plus `+N` for the N others, e.g. `main.rs +2`.
fn label(paths: &[PathBuf]) -> String {
    let first = paths.first().map(|path| path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned()));
    match (first, paths.len()) {
        (Some(first), 1) => first,
        (Some(first), count) => format!("{first} +{}", count - 1),
        (None, _) => String::new(),
    }
}

/// Drawn like a dragged tab; over a tree row it says what a drop there does.
pub struct DragChip {
    label: SharedString,
    workspace: WeakEntity<Workspace>,
}

impl Render for DragChip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let sizes = &theme.sizes.tabs;
        let hint = self.workspace.upgrade().and_then(|workspace| workspace.read(cx).tree().shown_hint(cx).map(|hint| hint.text.clone()));
        let text: SharedString = hint.map_or_else(|| self.label.clone(), SharedString::from);
        div()
            .flex()
            .items_center()
            .h(sizes.tab_height.px())
            .px(sizes.tab_padding_left.px())
            .rounded(sizes.tab_radius.px())
            .border(sizes.tab_border.px())
            .border_color(rgb(theme.tab_active_border))
            .bg(rgb(theme.tab_active))
            .opacity(sizes.drag_chip_opacity.0)
            .text_size(sizes.tab_text_size.px())
            .text_color(rgb(theme.text))
            .max_w(sizes.title_max_width.px())
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_label_one_file_by_its_name() {
        // when
        let label = label(&[PathBuf::from("/a/b/c.rs")]);

        // then
        assert_eq!(label, "c.rs");
    }

    #[test]
    fn should_label_many_files_by_first_name_and_count() {
        // when
        let label = label(&["/a/b/c.rs", "/a/d", "/e.md"].map(PathBuf::from));

        // then
        assert_eq!(label, "c.rs +2");
    }

    #[test]
    fn should_label_root_by_its_path() {
        // when
        let label = label(&[PathBuf::from("/")]);

        // then
        assert_eq!(label, "/");
    }
}
