//! The open tabs as `state.toml` keeps them: what each needs to open again, in tab order, and which one is shown.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const MAX_TABS: usize = 200;

pub use codeview::panel::PanelState;
pub use codeview::position::{CaretPosition, ScrollPosition};

/// One tab, by what opens it again. Sections, views and pages are their labels as shown; `panel` is where a code
/// panel was scrolled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", from = "ReadTab")]
pub enum SavedTab {
    File { path: PathBuf, panel: PanelState },
    Scratch { project: String, name: String, panel: PanelState },
    /// a hosted terminal, by its session id
    Terminal { session: String },
    ProjectOverview { project: String, section: String },
    /// an older IVE's worktree overview: still read, so its `state.toml` stays usable, and dropped on restore
    WorktreeOverview { checkout: PathBuf, section: String },
    GitLog { project: String, view: String },
    /// `title` as the tab showed it, since the subject is only known once git is read
    Commit { project: String, sha: String, title: String, panel: PanelState },
    Diff { checkout: PathBuf, request: SavedDiffRequest, panel: PanelState },
    Settings { page: String },
    /// a kept hub, reopened with an empty query
    Hub,
}

/// A tab as `state.toml` may hold it: as written now, or by an IVE from before `panel`, with its own `scroll` and
/// `caret` fields.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ReadTab {
    File { path: PathBuf, panel: Option<PanelState>, scroll: Option<ScrollPosition> },
    Scratch { project: String, name: String, panel: Option<PanelState>, caret: Option<CaretPosition>, scroll: Option<ScrollPosition> },
    Terminal { session: String },
    ProjectOverview { project: String, section: String },
    WorktreeOverview { checkout: PathBuf, section: String },
    GitLog { project: String, view: String },
    Commit { project: String, sha: String, title: String, panel: Option<PanelState> },
    Diff { checkout: PathBuf, request: SavedDiffRequest, panel: Option<PanelState> },
    Settings { page: String },
    Hub,
}

impl From<ReadTab> for SavedTab {
    fn from(tab: ReadTab) -> Self {
        let panel = |panel: Option<PanelState>, scroll: Option<ScrollPosition>, caret| {
            panel.unwrap_or_else(|| PanelState { scroll: scroll.unwrap_or_default(), caret, ..PanelState::default() })
        };
        match tab {
            ReadTab::File { path, panel: saved, scroll } => Self::File { path, panel: panel(saved, scroll, None) },
            ReadTab::Scratch { project, name, panel: saved, caret, scroll } => Self::Scratch { project, name, panel: panel(saved, scroll, caret) },
            ReadTab::Terminal { session } => Self::Terminal { session },
            ReadTab::ProjectOverview { project, section } => Self::ProjectOverview { project, section },
            ReadTab::WorktreeOverview { checkout, section } => Self::WorktreeOverview { checkout, section },
            ReadTab::GitLog { project, view } => Self::GitLog { project, view },
            ReadTab::Commit { project, sha, title, panel: saved } => Self::Commit { project, sha, title, panel: saved.unwrap_or_default() },
            ReadTab::Diff { checkout, request, panel: saved } => Self::Diff { checkout, request, panel: saved.unwrap_or_default() },
            ReadTab::Settings { page } => Self::Settings { page },
            ReadTab::Hub => Self::Hub,
        }
    }
}

/// A diff tab's request; paths are git's bytes, which need not be UTF-8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedDiffRequest {
    pub dir: PathBuf,
    pub path: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_path: Option<Vec<u8>>,
    pub source: SavedDiffSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SavedDiffSource {
    Uncommitted,
    Since { base: String, reference: String, fork: String, with_uncommitted: bool },
    Commit { sha: String, subject: String },
    /// Compare Files: two files anywhere
    Files { left: PathBuf, right: PathBuf },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SavedTabs {
    pub tabs: Vec<SavedTab>,
    /// index of the shown tab in `tabs`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<usize>,
}

impl SavedTabs {
    pub fn validate(&self) -> Result<(), String> {
        if self.tabs.len() > MAX_TABS {
            return Err(format!("{} tabs, the limit is {MAX_TABS}", self.tabs.len()));
        }
        if let Some(active) = self.active.filter(|active| *active >= self.tabs.len()) {
            return Err(format!("shown tab {active} is past the last of {} tabs", self.tabs.len()));
        }
        match self.tabs.iter().flat_map(SavedTab::paths).find(|path| path.is_relative()) {
            Some(relative) => Err(format!("tab path {} is not absolute", relative.display())),
            None => Ok(()),
        }
    }
}

impl SavedTab {
    /// As MCP's `list_tabs` reports the tab once it is loaded.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::File { .. } => "document",
            Self::Scratch { .. } => "scratch",
            Self::Terminal { .. } => "process",
            Self::ProjectOverview { .. } => "project-overview",
            Self::WorktreeOverview { .. } => "worktree-overview",
            Self::GitLog { .. } => "git-log",
            Self::Commit { .. } => "commit",
            Self::Diff { .. } => "diff",
            Self::Settings { .. } => "settings",
            Self::Hub => "hub",
        }
    }

    /// The chip's title before the tab loads, close to what it shows after.
    pub fn title(&self) -> String {
        let file_name = |path: &Path| path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned());
        match self {
            Self::File { path, .. } | Self::WorktreeOverview { checkout: path, .. } => file_name(path),
            Self::Scratch { name, .. } => name.clone(),
            Self::Terminal { .. } => "Terminal".to_owned(),
            Self::ProjectOverview { project, .. } => project.clone(),
            Self::GitLog { .. } => "Git log".to_owned(),
            Self::Commit { title, .. } => title.clone(),
            Self::Diff { request, .. } => file_name(Path::new(&*String::from_utf8_lossy(&request.path))),
            Self::Settings { .. } => "Settings".to_owned(),
            Self::Hub => "Hub".to_owned(),
        }
    }

    /// The path whose module owns the tab before it loads, as `Tab::owner_path` gives it after: a file's, a diff's
    /// file; `None` for the rest.
    pub fn owner_path(&self) -> Option<PathBuf> {
        match self {
            Self::File { path, .. } => Some(path.clone()),
            Self::Diff { request, .. } => Some(request.dir.join(&*String::from_utf8_lossy(&request.path))),
            _ => None,
        }
    }

    pub fn paths(&self) -> Vec<&PathBuf> {
        match self {
            Self::File { path, .. } | Self::WorktreeOverview { checkout: path, .. } => vec![path],
            Self::Diff { checkout, request, .. } => vec![checkout, &request.dir],
            Self::Scratch { .. } | Self::Terminal { .. } | Self::ProjectOverview { .. } | Self::GitLog { .. } | Self::Commit { .. } | Self::Settings { .. } | Self::Hub => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeview::panel::{DiffLayout, MarkdownView};

    #[test]
    fn should_own_a_restored_diff_by_its_file_as_a_loaded_one() {
        // given
        let request = SavedDiffRequest { dir: PathBuf::from("/r"), path: b"svc/A.java".to_vec(), old_path: None, source: SavedDiffSource::Uncommitted };
        let diff = SavedTab::Diff { checkout: PathBuf::from("/r"), request, panel: PanelState::default() };
        let file = SavedTab::File { path: PathBuf::from("/r/lib/x.rs"), panel: PanelState::default() };

        // when
        let owners = [diff.owner_path(), file.owner_path(), SavedTab::Hub.owner_path()];

        // then
        assert_eq!(owners, [Some(PathBuf::from("/r/svc/A.java")), Some(PathBuf::from("/r/lib/x.rs")), None]);
    }

    fn scroll() -> ScrollPosition {
        ScrollPosition { top_line: 12.5, left: 30. }
    }

    fn scrolled() -> PanelState {
        PanelState { scroll: scroll(), ..PanelState::default() }
    }

    fn every_kind() -> Vec<SavedTab> {
        vec![
            SavedTab::File { path: PathBuf::from("/p/src/main.rs"), panel: scrolled() },
            SavedTab::Scratch {
                project: "p".to_owned(),
                name: "notes.md".to_owned(),
                panel: PanelState { caret: Some(CaretPosition { line: 3, column: 7 }), ..scrolled() },
            },
            SavedTab::Terminal { session: "s-1".to_owned() },
            SavedTab::ProjectOverview { project: "p".to_owned(), section: "Scratches".to_owned() },
            SavedTab::WorktreeOverview { checkout: PathBuf::from("/p/wt"), section: "Processes".to_owned() },
            SavedTab::GitLog { project: "p".to_owned(), view: "Stashes".to_owned() },
            SavedTab::Commit { project: "p".to_owned(), sha: "0123abcd".to_owned(), title: "0123abc Fix".to_owned(), panel: PanelState { collapsed: vec!["src/a.rs".to_owned()], ..scrolled() } },
            SavedTab::Diff {
                checkout: PathBuf::from("/p/wt"),
                request: SavedDiffRequest {
                    dir: PathBuf::from("/p/wt"),
                    path: b"src/a.rs".to_vec(),
                    old_path: Some(b"src/b.rs".to_vec()),
                    source: SavedDiffSource::Since { base: "main".to_owned(), reference: "refs/heads/main".to_owned(), fork: "0123abcd".to_owned(), with_uncommitted: true },
                },
                panel: PanelState { layout: Some(DiffLayout::SideBySide), ..scrolled() },
            },
            SavedTab::Settings { page: "Appearance".to_owned() },
        ]
    }

    #[test]
    fn should_title_an_unloaded_tab_by_its_file() {
        // given
        let tabs = every_kind();

        // when
        let titles: Vec<String> = tabs.iter().map(SavedTab::title).collect();

        // then
        assert_eq!(titles, ["main.rs", "notes.md", "Terminal", "p", "wt", "Git log", "0123abc Fix", "a.rs", "Settings"]);
    }

    #[test]
    fn should_round_trip_every_tab_kind_through_toml() {
        // given
        let saved = SavedTabs { tabs: every_kind(), active: Some(2) };

        // when
        let text = toml::to_string(&saved).unwrap();
        let read: SavedTabs = toml::from_str(&text).unwrap();

        // then
        assert_eq!(read, saved);
    }

    #[test]
    fn should_reject_an_active_index_past_the_last_tab() {
        // given
        let saved = SavedTabs { tabs: every_kind(), active: Some(9) };

        // when
        let checked = saved.validate();

        // then
        assert_eq!(checked, Err("shown tab 9 is past the last of 9 tabs".to_owned()));
    }

    #[test]
    fn should_reject_relative_paths_in_tabs() {
        // given
        let saved = SavedTabs { tabs: vec![SavedTab::File { path: PathBuf::from("src/main.rs"), panel: scrolled() }], active: None };

        // when
        let checked = saved.validate();

        // then
        assert_eq!(checked, Err("tab path src/main.rs is not absolute".to_owned()));
    }

    #[test]
    fn should_reject_more_than_200_tabs() {
        // given
        let saved = SavedTabs { tabs: vec![SavedTab::Terminal { session: "s".to_owned() }; 201], active: None };

        // when
        let checked = saved.validate();

        // then
        assert_eq!(checked, Err("201 tabs, the limit is 200".to_owned()));
    }

    #[test]
    fn should_parse_a_saved_hub() {
        // given
        let text = "kind = \"hub\"\n";

        // when
        let tab: SavedTab = toml::from_str(text).unwrap();

        // then
        assert_eq!((tab.kind(), tab.title()), ("hub", "Hub".to_owned()));
    }

    #[test]
    fn should_restore_a_tab_saved_without_a_markdown_view() {
        // given
        let before = "kind = \"file\"\npath = \"/p/README.md\"\n";
        let rendered = "kind = \"file\"\npath = \"/p/README.md\"\n[panel]\nmarkdown = \"rendered\"\n";

        // when
        let tabs: Vec<SavedTab> = [before, rendered].into_iter().map(|text| toml::from_str(text).unwrap()).collect();

        // then
        let views: Vec<Option<MarkdownView>> = tabs.iter().map(|tab| match tab {
            SavedTab::File { panel, .. } => panel.markdown,
            other => panic!("read as {other:?}"),
        }).collect();
        assert_eq!(views, vec![None, Some(MarkdownView::Rendered)]);
        let written = toml::to_string(&tabs[1]).unwrap();
        assert!(written.contains("markdown = \"rendered\""), "{written}");
    }

    #[test]
    fn should_read_old_scroll_and_caret_into_panel_state() {
        // given
        let text = "kind = \"scratch\"\nproject = \"p\"\nname = \"notes.md\"\n\
                    scroll = { top_line = 12.5, left = 30.0 }\ncaret = { line = 3, column = 7 }\n";

        // when
        let tab: SavedTab = toml::from_str(text).unwrap();

        // then
        let SavedTab::Scratch { panel, .. } = tab else {
            panic!("read as {tab:?}");
        };
        assert_eq!((panel.scroll.top_line, panel.caret), (12.5, Some(CaretPosition { line: 3, column: 7 })));
    }

    #[test]
    fn should_write_only_the_panel_state() {
        // given
        let tab = SavedTab::File { path: PathBuf::from("/p/a.rs"), panel: scrolled() };

        // when
        let text = toml::to_string(&tab).unwrap();

        // then
        assert_eq!(text, "kind = \"file\"\npath = \"/p/a.rs\"\n\n[panel.scroll]\ntop_line = 12.5\nleft = 30.0\n");
    }
}
