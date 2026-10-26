//! How a file tree row is drawn besides its name: excluded, ignored, and its git status.

use std::path::Path;

use crate::git::status::TreeStatus;
use crate::git::types::Mark;
use crate::theme_config::{GitColors, Rgb, RowStyle, TreeStates};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    Plain,
    /// a gitignored folder, a folder named by an exclusion rule, or anything inside one
    Excluded,
    /// a gitignored file outside excluded folders
    Ignored,
}

/// The state of the entry at `relative` (to its checkout); `excluded` are folder names, as `config.excluded`.
pub fn entry_state(relative: &Path, is_dir: bool, status: &TreeStatus, excluded: &[String]) -> EntryState {
    let excluded_folder = |folder: &Path| {
        status.ignored.contains(folder) || folder.file_name().is_some_and(|name| excluded.iter().any(|rule| name == rule.as_str()))
    };
    // a folder counts itself; a file only its folders
    let mut folders = relative.ancestors().skip(usize::from(!is_dir)).filter(|folder| !folder.as_os_str().is_empty());
    if folders.any(excluded_folder) {
        return EntryState::Excluded;
    }
    if status.ignored.contains(relative) {
        return EntryState::Ignored;
    }
    EntryState::Plain
}

/// A file's git mark: its own, or unversioned inside a folder git does not track; folders get none.
pub fn entry_mark(relative: &Path, is_dir: bool, status: &TreeStatus) -> Option<Mark> {
    if is_dir {
        return None;
    }
    if let Some(mark) = status.marks.get(relative) {
        return Some(*mark);
    }
    relative.ancestors().skip(1).any(|folder| status.untracked.contains(folder)).then_some(Mark::Untracked)
}

/// What a row is besides its content: a file row's state and git mark, whether it is selected, and whether it
/// shows the active tab's file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Highlight {
    pub entry: Option<(EntryState, Option<Mark>)>,
    pub selected: bool,
    pub active: bool,
    /// what is dragged over it may be dropped on it
    pub drop: bool,
}

/// How a row is drawn when the mouse is not over it: the theme's styles layered (excluded or ignored, then active
/// unless selected, then selected, then a drop row's), and a git mark's colour on the text over all of them, as IntelliJ keeps it on
/// selected rows.
pub fn row_look(highlight: Highlight, states: &TreeStates, git: &GitColors) -> RowStyle {
    let state = highlight.entry.map(|(state, _)| state);
    let mut style = match state {
        Some(EntryState::Excluded) => states.excluded,
        Some(EntryState::Ignored) => states.ignored,
        Some(EntryState::Plain) | None => RowStyle::default(),
    };
    if highlight.active && !highlight.selected {
        style = layered(style, states.active);
    }
    if highlight.selected {
        style = layered(style, states.selected);
    }
    if highlight.drop {
        style = layered(style, states.drop);
    }
    if let Some((_, Some(mark))) = highlight.entry {
        style.text = Some(git_colour(mark, git));
    }
    style
}

/// `above`'s properties, and `below`'s where `above` leaves one unset.
fn layered(below: RowStyle, above: RowStyle) -> RowStyle {
    RowStyle {
        text: above.text.or(below.text),
        background: above.background.or(below.background),
        weight: above.weight.or(below.weight),
        italic: above.italic.or(below.italic),
    }
}

fn git_colour(mark: Mark, git: &GitColors) -> Rgb {
    match mark {
        Mark::Modified | Mark::TypeChange => git.modified,
        Mark::Added => git.added,
        Mark::Deleted => git.deleted,
        Mark::Untracked => git.unversioned,
        Mark::Renamed => git.renamed,
        Mark::Conflicted => git.conflicted,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use super::*;
    use crate::theme_config::Rgba;

    fn status(ignored: &[&str], marks: &[(&str, Mark)]) -> TreeStatus {
        TreeStatus {
            ignored: ignored.iter().map(PathBuf::from).collect::<HashSet<_>>(),
            marks: marks.iter().map(|(path, mark)| (PathBuf::from(path), *mark)).collect::<HashMap<_, _>>(),
            untracked: HashSet::new(),
        }
    }

    fn rules(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn should_exclude_folder_by_name_rule() {
        // given
        let (status, rules) = (status(&[], &[]), rules(&["node_modules"]));

        // when
        let state = entry_state(Path::new("web/node_modules"), true, &status, &rules);

        // then
        assert_eq!(state, EntryState::Excluded);
    }

    #[test]
    fn should_exclude_ignored_folder() {
        // given
        let status = status(&["dist"], &[]);

        // when
        let state = entry_state(Path::new("dist"), true, &status, &[]);

        // then
        assert_eq!(state, EntryState::Excluded);
    }

    #[test]
    fn should_mark_ignored_file() {
        // given
        let status = status(&["debug.log"], &[]);

        // when
        let state = entry_state(Path::new("debug.log"), false, &status, &[]);

        // then
        assert_eq!(state, EntryState::Ignored);
    }

    #[test]
    fn should_exclude_children_of_excluded_folder() {
        // given
        let (status, rules) = (status(&[], &[]), rules(&["target"]));

        // when
        let states = [
            entry_state(Path::new("target/a.txt"), false, &status, &rules),
            entry_state(Path::new("target/debug"), true, &status, &rules),
        ];

        // then
        assert_eq!(states, [EntryState::Excluded, EntryState::Excluded]);
    }

    #[test]
    fn should_not_exclude_file_named_like_a_rule() {
        // given
        let rules = rules(&["target"]);

        // when
        let state = entry_state(Path::new("docs/target"), false, &status(&[], &[]), &rules);

        // then
        assert_eq!(state, EntryState::Plain);
    }

    fn look(entry: Option<(EntryState, Option<Mark>)>, selected: bool, active: bool) -> RowStyle {
        let theme = crate::theme::Theme::of(crate::theme::Appearance::Dark);
        row_look(Highlight { entry, selected, active, drop: false }, &theme.tree_states, &theme.git)
    }

    #[test]
    fn should_style_active_row_only_when_not_selected() {
        // given
        let theme = crate::theme::Theme::of(crate::theme::Appearance::Dark);
        let states = TreeStates { active: RowStyle { background: Some(Rgba(0x11223344)), ..RowStyle::default() }, ..theme.tree_states };
        let look = |selected, active| row_look(Highlight { entry: None, selected, active, drop: false }, &states, &theme.git);

        // when
        let (active, both, plain) = (look(false, true), look(true, true), look(false, false));

        // then
        assert_eq!(active.background, Some(Rgba(0x11223344)));
        assert_eq!(both.background, states.selected.background);
        assert_eq!(plain, RowStyle::default());
    }

    #[test]
    fn should_draw_drop_row_over_selection() {
        // given
        let theme = crate::theme::Theme::of(crate::theme::Appearance::Dark);

        // when
        let style = row_look(Highlight { entry: None, selected: true, active: false, drop: true }, &theme.tree_states, &theme.git);

        // then
        assert_eq!(style.background, theme.tree_states.drop.background);
        assert!(theme.tree_states.drop.background.is_some());
    }

    #[test]
    fn should_layer_italic_like_weight() {
        // given
        let below = RowStyle { italic: Some(true), ..RowStyle::default() };
        let upright = RowStyle { italic: Some(false), ..RowStyle::default() };

        // when
        let (kept, overridden) = (layered(below, RowStyle::default()), layered(below, upright));

        // then
        assert_eq!((kept.italic, overridden.italic), (Some(true), Some(false)));
    }

    #[test]
    fn should_leave_active_row_plain_by_default() {
        // when
        let style = look(None, false, true);

        // then
        assert_eq!(style, RowStyle::default());
    }

    #[test]
    fn should_keep_git_colour_on_selected_excluded_row() {
        // given
        let theme = crate::theme::Theme::of(crate::theme::Appearance::Dark);

        // when
        let style = look(Some((EntryState::Excluded, Some(Mark::Modified))), true, false);

        // then
        assert_eq!(style.text, Some(theme.git.modified));
        assert_eq!(style.background, theme.tree_states.selected.background);
    }

    #[test]
    fn should_draw_excluded_row_with_its_own_colours() {
        // given
        let theme = crate::theme::Theme::of(crate::theme::Appearance::Dark);

        // when
        let style = look(Some((EntryState::Excluded, None)), false, false);

        // then
        assert_eq!(style, theme.tree_states.excluded);
    }

    #[test]
    fn should_mark_file_inside_untracked_folder_as_unversioned() {
        // given
        let status = TreeStatus { untracked: HashSet::from([PathBuf::from("vendor")]), ..TreeStatus::default() };

        // when
        let marks = [
            entry_mark(Path::new("vendor/lib/a.js"), false, &status),
            entry_mark(Path::new("vendor/lib"), true, &status),
            entry_mark(Path::new("other.js"), false, &status),
        ];

        // then
        assert_eq!(marks, [Some(Mark::Untracked), None, None]);
    }

    #[test]
    fn should_not_mark_folders() {
        // given
        let status = status(&[], &[("src/a.rs", Mark::Modified), (".claude/worktrees/topic", Mark::Untracked)]);

        // when
        let marks = [
            entry_mark(Path::new("src"), true, &status),
            entry_mark(Path::new(".claude/worktrees/topic"), true, &status),
            entry_mark(Path::new("src/a.rs"), false, &status),
        ];

        // then
        assert_eq!(marks, [None, None, Some(Mark::Modified)]);
    }
}
