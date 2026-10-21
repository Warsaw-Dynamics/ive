//! The hub's state and what its keys do to it: the query, the scope, the worktree question, the selection.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::matcher::split_line;
use super::rows::List;
use super::rows::recent_matches;
use super::corpus::MAX_CHECKOUTS;
use super::search::Filter;
pub use super::sources::HubScope;
use super::sources::{Place, Sources, Target};
use crate::project::{CheckoutRef, ProjectId};
use crate::script::keymap::Builtin;
use crate::tabs::TabId;

/// Rows PgUp and PgDn move by.
pub const PAGE: usize = 10;

/// Everything the hub remembers between keys; also what `ive::open_hub` restores, as JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HubState {
    pub query: String,
    pub scope: HubScope,
    /// a scope picked with `:` that Tab does not reach otherwise
    pub extra_scope: Option<HubScope>,
    /// the worktree question
    pub pick: Option<Pick>,
    pub selected: usize,
    /// Ctrl+Shift+E again: only Changed
    pub changed_only: bool,
    /// excluded and ignored files searched too
    pub excluded: bool,
    /// the tab shown before the hub, for a hub that comes back (`opened_from_hub`)
    pub back: Option<TabId>,
    /// Ctrl+Shift+T: the hub only asks where a new terminal goes
    pub new_terminal: bool,
}

/// The worktree question for a row in several worktrees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pick {
    /// the row's title, shown as `<title> ›` in the query row
    pub title: String,
    pub target: Target,
    /// filters the worktrees
    pub query: String,
    pub selected: usize,
    pub behind: bool,
    pub line: Option<u32>,
    pub project: ProjectId,
    /// the checkouts that have the row's thing
    pub checkouts: Vec<PathBuf>,
}

/// What the query asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Empty,
    Files { line: Option<u32> },
    /// `>`
    Actions,
    /// `@`
    Processes,
    /// `#`
    Worktrees,
    /// `:`
    Scopes,
    /// `:42`
    Line(u32),
    Pick,
    /// projects, worktrees and branches a new terminal can go to; the query is plain text
    NewTerminal,
}

pub fn mode(state: &HubState) -> Mode {
    if state.pick.is_some() {
        return Mode::Pick;
    }
    if state.new_terminal {
        return Mode::NewTerminal;
    }
    let query = state.query.trim_start();
    match query.chars().next() {
        None => Mode::Empty,
        Some('>') => Mode::Actions,
        Some('@') => Mode::Processes,
        Some('#') => Mode::Worktrees,
        Some(':') => query[1..].parse::<u32>().ok().filter(|_| query[1..].bytes().all(|byte| byte.is_ascii_digit())).map_or(Mode::Scopes, Mode::Line),
        Some(_) if query.trim().is_empty() => Mode::Empty,
        Some(_) => Mode::Files { line: split_line(query).line },
    }
}

/// What the workspace does after a key; made once per key, so its size does not matter.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Nothing,
    /// the state changed: draw the list again
    Redraw,
    /// the query, scope or filter changed: search again
    Search,
    Open { target: Target, checkout: Option<CheckoutRef>, line: Option<u32>, behind: bool },
    /// back to the tab shown before the hub
    Leave,
    /// an absolute path, for the last terminal
    TypePath(String),
    /// a path, for the clipboard
    CopyPath(String),
    /// a process row, or a Recent row's open tab, whose process to stop
    Stop(Target),
}

impl HubState {
    /// The text the files are searched with, when the query searches files.
    pub fn search_text(&self) -> Option<&str> {
        match mode(self) {
            Mode::Files { .. } if !self.changed_only => Some(split_line(&self.query).query),
            _ => None,
        }
    }

    /// The query field's text changed.
    pub fn typed(&mut self, text: String) -> Effect {
        if let Some(pick) = &mut self.pick {
            pick.query = text;
            pick.selected = 0;
            return Effect::Redraw;
        }
        self.query = text;
        self.selected = 0;
        Effect::Search
    }

    pub fn act(&mut self, action: Builtin, list: &List, sources: &Sources) -> Effect {
        let last = list.rows.len().saturating_sub(1);
        match action {
            Builtin::HubDown => self.select(|selected| (selected + 1).min(last)),
            Builtin::HubUp => self.select(|selected| selected.saturating_sub(1)),
            Builtin::HubPageDown => self.select(|selected| (selected + PAGE).min(last)),
            Builtin::HubPageUp => self.select(|selected| selected.saturating_sub(PAGE)),
            Builtin::HubOpen => self.open(self.selected_row(), list, sources, false),
            Builtin::HubOpenBehind => self.open(self.selected_row(), list, sources, true),
            Builtin::HubOpenRow(number) => self.open(usize::from(number).saturating_sub(1), list, sources, false),
            Builtin::HubTypePath => self.row_path(list, sources).map_or(Effect::Nothing, |(absolute, _)| Effect::TypePath(absolute)),
            Builtin::HubCopyPath => self.row_path(list, sources).map_or(Effect::Nothing, |(_, relative)| Effect::CopyPath(relative)),
            Builtin::CopyFullPath => self.row_path(list, sources).map_or(Effect::Nothing, |(absolute, _)| Effect::CopyPath(absolute)),
            Builtin::HubStop => self.stop(list),
            Builtin::HubCycleScope if mode(self) == Mode::Scopes => self.open(self.selected_row(), list, sources, false),
            Builtin::HubCycleScope => self.cycle_scope(sources),
            Builtin::HubWiden if self.query.is_empty() && self.pick.is_none() && self.scope != HubScope::All => {
                self.scope = HubScope::All;
                Effect::Search
            }
            Builtin::HubToggleExcluded => {
                self.excluded = !self.excluded;
                Effect::Search
            }
            Builtin::HubToggleChanged if self.new_terminal => {
                self.new_terminal = false;
                self.selected = 0;
                Effect::Search
            }
            Builtin::HubToggleChanged => {
                self.changed_only = !self.changed_only;
                self.selected = 0;
                Effect::Redraw
            }
            Builtin::HubBack => self.back(),
            _ => Effect::Nothing,
        }
    }

    /// Ctrl+Shift+E on a hub tab that holds the new terminal picker: the hub itself, fresh; a hub stays as it is.
    pub fn show_as_hub(&mut self) {
        if self.new_terminal {
            *self = HubState { scope: self.scope.clone(), extra_scope: self.extra_scope.clone(), ..HubState::default() };
        }
    }

    /// The selection within `list`, after the rows changed.
    pub fn fit_selection(&mut self, list: &List) {
        let last = list.rows.len().saturating_sub(1);
        self.select(|selected| selected.min(last));
    }

    /// This state with what is no longer open dropped: a scope of a closed project or worktree counts as All, a
    /// question about one is gone.
    pub fn fitted(mut self, sources: &Sources) -> HubState {
        let open = |scope: &HubScope| match scope {
            HubScope::All => true,
            HubScope::Project(project) => sources.project_index(project).is_some(),
            HubScope::Worktree(checkout) => sources.worktree(checkout).is_some(),
        };
        if !open(&self.scope) {
            self.scope = HubScope::All;
        }
        self.extra_scope = self.extra_scope.filter(open);
        if let Some(pick) = &mut self.pick {
            let project = pick.project.clone();
            pick.checkouts.retain(|path| sources.worktree(&CheckoutRef { project: project.clone(), path: path.clone() }).is_some());
        }
        self.pick = self.pick.filter(|pick| !pick.checkouts.is_empty());
        self
    }

    /// What the files search looks at: the scope, the excluded switch, and the paths of the Recent rows above.
    pub fn filter(&self, sources: &Sources) -> Filter {
        let project = |id: &ProjectId| sources.project_index(id);
        let (project, checkout) = match &self.scope {
            HubScope::All => (None, None),
            HubScope::Project(id) => (project(id), None),
            HubScope::Worktree(checkout) => {
                let index = project(&checkout.project);
                let position = index.and_then(|index| sources.projects[index].worktrees.iter().position(|worktree| worktree.checkout == *checkout));
                let bit = position.filter(|position| *position < MAX_CHECKOUTS);
                (index, index.zip(bit).and_then(|(index, bit)| u32::try_from(bit).ok().map(|bit| (index, bit))))
            }
        };
        let skip: HashSet<(usize, Arc<str>)> = recent_matches(self, sources)
            .into_iter()
            .filter_map(|row| Some((sources.project_index(row.item.project.as_ref()?)?, Arc::from(row.item.path?.as_str()))))
            .collect();
        Filter { project, checkout, excluded: self.excluded, skip }
    }

    fn selected_row(&self) -> usize {
        self.pick.as_ref().map_or(self.selected, |pick| pick.selected)
    }

    fn select(&mut self, change: impl FnOnce(usize) -> usize) -> Effect {
        let selected = match &mut self.pick {
            Some(pick) => &mut pick.selected,
            None => &mut self.selected,
        };
        let new = change(*selected);
        if new == *selected {
            return Effect::Nothing;
        }
        *selected = new;
        Effect::Redraw
    }

    fn back(&mut self) -> Effect {
        if self.pick.take().is_some() {
            return Effect::Redraw;
        }
        if self.query.is_empty() {
            return Effect::Leave;
        }
        self.query.clear();
        self.selected = 0;
        Effect::Search
    }

    fn open(&mut self, index: usize, list: &List, sources: &Sources, behind: bool) -> Effect {
        let Some(row) = list.rows.get(index) else {
            return Effect::Nothing;
        };
        let target = row.item.target.clone();
        if let Target::Scope(scope) = target {
            return self.pick_scope(scope, sources);
        }
        if let Some(pick) = &self.pick {
            let checkout = match &row.item.place {
                Place::One(checkout) => Some(checkout.clone()),
                _ => None,
            };
            return Effect::Open { target: pick.target.clone(), checkout, line: pick.line, behind: pick.behind || behind };
        }
        let line = match mode(self) {
            Mode::Files { line } => line,
            Mode::Line(line) => Some(line),
            _ => None,
        };
        // nothing opens behind a new terminal's place
        let behind = behind && !self.new_terminal;
        match &row.item.place {
            Place::Nowhere => Effect::Open { target, checkout: None, line, behind },
            Place::One(checkout) => Effect::Open { target, checkout: Some(checkout.clone()), line, behind },
            Place::Several { project, checkouts } => match &self.scope {
                HubScope::Worktree(worktree) if checkouts.contains(&worktree.path) => Effect::Open { target, checkout: Some(worktree.clone()), line, behind },
                _ => {
                    let title = row.item.title.clone();
                    self.pick = Some(Pick { title, target, query: String::new(), selected: 0, behind, line, project: project.clone(), checkouts: checkouts.clone() });
                    Effect::Redraw
                }
            },
        }
    }

    /// The stops Tab goes through, as the scope chips show them.
    pub fn stops(&self, sources: &Sources) -> Vec<HubScope> {
        let mut stops = self.scope_stops(sources);
        stops.extend(self.extra_scope.clone().filter(|extra| !stops.contains(extra)));
        stops
    }

    /// The stops Tab goes through: All, the back tab's project, its worktree (its own, else the project's most
    /// recently used), then a scope picked with `:`.
    fn scope_stops(&self, sources: &Sources) -> Vec<HubScope> {
        let mut stops = vec![HubScope::All];
        if let Some(back) = &sources.back
            && let Some(project) = &back.project
        {
            stops.push(HubScope::Project(project.clone()));
            let worktree = back.checkout.clone().or_else(|| sources.by_recency(project, None).first().map(|worktree| worktree.checkout.clone()));
            stops.extend(worktree.map(HubScope::Worktree));
        }
        stops
    }

    fn cycle_scope(&mut self, sources: &Sources) -> Effect {
        let stops = self.stops(sources);
        let next = match stops.iter().position(|stop| *stop == self.scope) {
            Some(at) => stops[(at + 1) % stops.len()].clone(),
            None => HubScope::All,
        };
        if next == self.scope {
            return Effect::Nothing;
        }
        self.scope = next;
        Effect::Search
    }

    /// Makes `scope` the scope and clears the query; one Tab does not reach joins its stops.
    pub fn pick_scope(&mut self, scope: HubScope, sources: &Sources) -> Effect {
        if !self.scope_stops(sources).contains(&scope) {
            self.extra_scope = Some(scope.clone());
        }
        if mode(self) == Mode::Scopes {
            self.query.clear();
        }
        self.scope = scope;
        self.selected = 0;
        Effect::Search
    }

    fn stop(&self, list: &List) -> Effect {
        match list.rows.get(self.selected_row()).map(|row| &row.item.target) {
            Some(target @ (Target::Process(_) | Target::Tab(_))) if self.pick.is_none() => Effect::Stop(target.clone()),
            _ => Effect::Nothing,
        }
    }

    /// The selected row's path: absolute in its (most recently used) worktree, and project-relative.
    fn row_path(&self, list: &List, sources: &Sources) -> Option<(String, String)> {
        let row = list.rows.get(self.selected_row())?;
        let relative = row.item.path.clone()?;
        let checkout = match &row.item.place {
            Place::One(checkout) => checkout.path.clone(),
            Place::Several { project, checkouts } => sources.by_recency(project, Some(checkouts)).first()?.checkout.path.clone(),
            Place::Nowhere => return None,
        };
        Some((checkout.join(&relative).display().to_string(), relative))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::hub::corpus::{Corpus, ProjectInput, build};
    use crate::hub::files::{CheckoutFiles, Class};
    use crate::hub::rows::{LINE_NEEDS_TEXT, LOADING_FILES, Tag, list};
    use crate::hub::search::{RENDERED, search};
    use crate::hub::sources::{BackTab, FreeBranch, Item, NoStates, Place, ProjectInfo, Sources, Target, WorktreeInfo};
    use crate::process::ProcessId;
    use crate::project::{CheckoutRef, ProjectId};
    use crate::saved_tabs::{SavedTab, ScrollPosition};
    use crate::scratch::search::Generations;
    use crate::script::keymap::Builtin;
    use crate::tabs::Tabs;

    fn atlassian() -> ProjectId {
        ProjectId::parse("atlassian").unwrap()
    }

    fn notes() -> ProjectId {
        ProjectId::parse("notes").unwrap()
    }

    fn checkout(project: ProjectId, path: &str) -> CheckoutRef {
        CheckoutRef { project, path: PathBuf::from(path) }
    }

    fn dev() -> CheckoutRef {
        checkout(atlassian(), "/a/dev")
    }

    fn ive() -> CheckoutRef {
        checkout(atlassian(), "/a/ive")
    }

    fn old() -> CheckoutRef {
        checkout(atlassian(), "/a/old")
    }

    fn worktree(checkout: CheckoutRef, label: &str, last_used: Option<i64>) -> WorktreeInfo {
        WorktreeInfo { checkout, label: label.to_owned(), branch: Some(label.to_owned()), tags: Vec::new(), last_used, running: 0 }
    }

    fn file(path: &str) -> SavedTab {
        SavedTab::File { path: PathBuf::from(path), scroll: ScrollPosition { top_line: 0., left: 0. } }
    }

    fn sources() -> Sources {
        let projects = vec![
            ProjectInfo {
                id: atlassian(),
                name: "atlassian".to_owned(),
                git: true,
                worktrees: vec![worktree(dev(), "dev", Some(100)), worktree(ive(), "GEN-817-ive", Some(200)), worktree(old(), "old", None)],
            },
            ProjectInfo { id: notes(), name: "notes".to_owned(), git: false, worktrees: vec![worktree(checkout(notes(), "/n"), "notes", None)] },
        ];
        let recent_file = Item {
            detail: "src".to_owned(),
            place: Place::One(ive()),
            project: Some(atlassian()),
            path: Some("src/theme.rs".to_owned()),
            age: Some(150),
            ..Item::new(Target::Reopen(file("/a/ive/src/theme.rs")), "theme.rs", "file")
        };
        let settings = Item { greyed: true, ..Item::new(Target::Reopen(SavedTab::Settings { page: "appearance".to_owned() }), "Settings", "settings") };
        let changed = Item {
            place: Place::One(dev()),
            project: Some(atlassian()),
            path: Some("README.md".to_owned()),
            ..Item::new(Target::Change { checkout: dev(), path: "README.md".to_owned() }, "README.md", "file")
        };
        let running = Item {
            place: Place::One(ive()),
            project: Some(atlassian()),
            state: Some("running".to_owned()),
            ..Item::new(Target::Reopen(SavedTab::Terminal { session: "s".to_owned() }), "Terminal", "terminal")
        };
        Sources {
            projects,
            recent: vec![recent_file, settings],
            changed: vec![changed],
            running: vec![running],
            ..Sources::default()
        }
    }

    fn listing(paths: &[&str]) -> CheckoutFiles {
        CheckoutFiles { paths: paths.iter().map(|path| (path.to_string(), Class::Searched)).collect(), skipped: 0 }
    }

    fn corpus() -> Corpus {
        let (dev, ive, old, notes) = (listing(&["src/theme.rs"]), listing(&["src/new.rs", "src/theme.rs"]), listing(&["src/theme.rs"]), listing(&["notes.md"]));
        build(&[
            ProjectInput { checkouts: vec![Some(&dev), Some(&ive), Some(&old)], ignored: Vec::new(), scratches: &[] },
            ProjectInput { checkouts: vec![Some(&notes)], ignored: Vec::new(), scratches: &[] },
        ])
    }

    fn back_in_dev() -> BackTab {
        BackTab { tab: Tabs::default().open(()), project: Some(atlassian()), checkout: Some(dev()), is_text: true }
    }

    /// The list as the hub draws it: a search runs for a files query.
    fn shown(state: &HubState, sources: &Sources) -> List {
        let corpus = corpus();
        let found = state.search_text().map(|text| {
            let query = crate::hub::matcher::Query::parse(text);
            search(&corpus, &query, &state.filter(sources), &Generations::default().next(), |_| {}).unwrap()
        });
        list(state, sources, found.as_ref().map(|found| (&corpus, found)), &NoStates)
    }

    fn act(state: &mut HubState, action: Builtin, sources: &Sources) -> Effect {
        let list = shown(state, sources);
        state.act(action, &list, sources)
    }

    fn typed(state: &mut HubState, text: &str) {
        state.typed(text.to_owned());
    }

    fn titles(list: &List) -> Vec<&str> {
        list.rows.iter().map(|row| row.item.title.as_str()).collect()
    }

    fn sections(list: &List) -> Vec<&str> {
        list.sections.iter().map(|section| section.title.as_str()).collect()
    }

    fn opened_in(effect: &Effect) -> Option<&Path> {
        match effect {
            Effect::Open { checkout, .. } => checkout.as_ref().map(|checkout| checkout.path.as_path()),
            _ => None,
        }
    }

    #[test]
    fn should_list_recent_changed_and_running_on_an_empty_query() {
        // given
        let state = HubState::default();

        // when
        let list = shown(&state, &sources());

        // then
        assert_eq!(sections(&list), ["Recent", "Changed", "Running"]);
        assert_eq!(titles(&list), ["theme.rs", "Settings", "README.md", "Terminal"]);
        assert_eq!(list.rows.iter().map(|row| row.number).collect::<Vec<_>>(), [Some(1), Some(2), Some(3), Some(4)]);
    }

    #[test]
    fn should_put_matching_recent_rows_before_files() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "theme");

        // when
        let list = shown(&state, &sources());

        // then
        assert_eq!(sections(&list), ["Recent", "Files"]);
        assert_eq!(titles(&list), ["theme.rs"]);
        assert_eq!(list.counter.as_deref().map(|counter| counter.starts_with("0/2")), Some(true), "{:?}", list.counter);
    }

    #[test]
    fn should_ask_the_worktree_for_a_file_in_several() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "new");
        typed(&mut state, "srctheme");

        // when
        let effect = act(&mut state, Builtin::HubOpen, &Sources { recent: Vec::new(), ..sources() });

        // then
        assert_eq!(effect, Effect::Redraw);
        let list = shown(&state, &sources());
        let tags: Vec<_> = list.rows.iter().map(|row| row.tag.clone()).collect();
        assert_eq!(tags, [Tag::Worktree(ive()), Tag::Worktree(dev()), Tag::Worktree(old())]);
        assert_eq!(mode(&state), Mode::Pick);
    }

    #[test]
    fn should_open_in_the_most_recent_worktree_on_enter_enter() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, "srctheme");
        act(&mut state, Builtin::HubOpen, &sources);

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources);

        // then
        assert_eq!(opened_in(&effect), Some(Path::new("/a/ive")));
        assert!(matches!(effect, Effect::Open { target: Target::File { ref path, .. }, behind: false, .. } if path == "src/theme.rs"));
    }

    #[test]
    fn should_open_a_file_of_one_worktree_without_asking() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "srcnew");

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources());

        // then
        assert_eq!(opened_in(&effect), Some(Path::new("/a/ive")));
    }

    #[test]
    fn should_open_without_asking_in_a_worktree_scope() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState { scope: HubScope::Worktree(dev()), ..HubState::default() };
        typed(&mut state, "srctheme");

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources);

        // then
        assert_eq!(opened_in(&effect), Some(Path::new("/a/dev")));
    }

    #[test]
    fn should_carry_open_behind_into_the_question() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, "srctheme");
        act(&mut state, Builtin::HubOpenBehind, &sources);

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources);

        // then
        assert!(matches!(effect, Effect::Open { behind: true, .. }), "{effect:?}");
    }

    #[test]
    fn should_step_back_with_escape() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, "srctheme");
        act(&mut state, Builtin::HubOpen, &sources);
        typed(&mut state, "d");

        // when
        let effects = [
            act(&mut state, Builtin::HubBack, &sources),
            act(&mut state, Builtin::HubBack, &sources),
            act(&mut state, Builtin::HubBack, &sources),
        ];

        // then
        assert_eq!(effects, [Effect::Redraw, Effect::Search, Effect::Leave]);
        assert_eq!((state.query.as_str(), state.pick.is_none()), ("", true));
    }

    #[test]
    fn should_keep_the_query_when_leaving_the_question() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, "srctheme");
        act(&mut state, Builtin::HubOpen, &sources);

        // when
        act(&mut state, Builtin::HubBack, &sources);

        // then
        assert_eq!(state.query, "srctheme");
    }

    #[test]
    fn should_open_the_numbered_row() {
        // given
        let mut state = HubState::default();

        // when
        let second = act(&mut state, Builtin::HubOpenRow(2), &sources());
        let ninth = act(&mut state, Builtin::HubOpenRow(9), &sources());

        // then
        assert!(matches!(second, Effect::Open { target: Target::Reopen(SavedTab::Settings { .. }), .. }), "{second:?}");
        assert_eq!(ninth, Effect::Nothing);
    }

    #[test]
    fn should_move_the_selection_within_the_list() {
        // given
        let mut state = HubState::default();

        // when
        act(&mut state, Builtin::HubUp, &sources());
        let at_top = state.selected;
        for _ in 0..6 {
            act(&mut state, Builtin::HubDown, &sources());
        }

        // then
        assert_eq!((at_top, state.selected), (0, 3));
    }

    #[test]
    fn should_cycle_the_scope_from_the_back_tab() {
        // given
        let sources = Sources { back: Some(back_in_dev()), ..sources() };
        let mut state = HubState::default();

        // when
        let scopes: Vec<HubScope> = (0..3)
            .map(|_| {
                act(&mut state, Builtin::HubCycleScope, &sources);
                state.scope.clone()
            })
            .collect();

        // then
        assert_eq!(scopes, [HubScope::Project(atlassian()), HubScope::Worktree(dev()), HubScope::All]);
    }

    #[test]
    fn should_stay_on_all_without_a_back_tab() {
        // given
        let mut state = HubState::default();

        // when
        act(&mut state, Builtin::HubCycleScope, &sources());

        // then
        assert_eq!(state.scope, HubScope::All);
    }

    #[test]
    fn should_add_a_scope_picked_with_colon_as_the_last_stop() {
        // given
        let mut state = HubState::default();
        typed(&mut state, ":notes");
        let rows = titles(&shown(&state, &sources())).iter().map(|title| title.to_string()).collect::<Vec<_>>();

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources());
        let picked = state.scope.clone();
        act(&mut state, Builtin::HubCycleScope, &sources());
        let after = state.scope.clone();
        act(&mut state, Builtin::HubCycleScope, &sources());

        // then
        assert_eq!(rows[0], "notes");
        assert_eq!((effect, picked, state.query.as_str()), (Effect::Search, HubScope::Project(notes()), ""));
        assert_eq!((after, state.scope.clone()), (HubScope::All, HubScope::Project(notes())));
    }

    #[test]
    fn should_list_projects_with_their_worktrees_by_recency_for_colon() {
        // given
        let mut state = HubState::default();
        typed(&mut state, ":");

        // when
        let list = shown(&state, &sources());

        // then
        assert_eq!(titles(&list), ["All projects", "atlassian", "GEN-817-ive", "dev", "old", "notes", "notes"]);
        assert_eq!(list.rows.iter().map(|row| row.depth).collect::<Vec<_>>(), [0, 0, 1, 1, 1, 0, 1]);
    }

    #[test]
    fn should_widen_only_on_an_empty_query() {
        // given
        let mut state = HubState { scope: HubScope::Project(atlassian()), ..HubState::default() };
        typed(&mut state, "x");

        // when
        let with_query = act(&mut state, Builtin::HubWiden, &sources());
        typed(&mut state, "");
        let without = act(&mut state, Builtin::HubWiden, &sources());

        // then
        assert_eq!((with_query, without, state.scope), (Effect::Nothing, Effect::Search, HubScope::All));
    }

    #[test]
    fn should_go_to_a_line_of_the_back_tab() {
        // given
        let sources = Sources { back: Some(back_in_dev()), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, ":42");

        // when
        let list = shown(&state, &sources);
        let effect = act(&mut state, Builtin::HubOpen, &sources);

        // then
        assert_eq!(titles(&list), ["Line 42"]);
        assert!(matches!(effect, Effect::Open { target: Target::Line(42), line: Some(42), .. }), "{effect:?}");
    }

    #[test]
    fn should_say_why_a_line_needs_a_text_tab() {
        // given
        let sources = Sources { back: Some(BackTab { is_text: false, ..back_in_dev() }), ..sources() };
        let mut state = HubState::default();
        typed(&mut state, ":42");

        // when
        let list = shown(&state, &sources);

        // then
        assert!(list.rows.is_empty());
        assert_eq!(list.message.as_deref(), Some(LINE_NEEDS_TEXT));
        assert_eq!(LINE_NEEDS_TEXT, "Line numbers go to a file shown before the hub");
    }

    #[test]
    fn should_open_a_file_at_its_line() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "srcnew:7");

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources());

        // then
        assert!(matches!(effect, Effect::Open { line: Some(7), .. }), "{effect:?}");
    }

    #[test]
    fn should_list_changed_only_after_ctrl_shift_e() {
        // given
        let mut state = HubState::default();

        // when
        let effect = act(&mut state, Builtin::HubToggleChanged, &sources());
        let list = shown(&state, &sources());

        // then
        assert_eq!((effect, sections(&list), titles(&list)), (Effect::Redraw, vec!["Changed"], vec!["README.md"]));
    }

    #[test]
    fn should_show_only_the_first_changes_of_a_long_list() {
        // given
        let mut sources = sources();
        let change = sources.changed[0].clone();
        sources.changed = vec![change; 4871];

        // when
        let list = shown(&HubState::default(), &sources);

        // then
        let changed = list.sections.iter().find(|section| section.title == "Changed").unwrap();
        assert_eq!((changed.rows.len(), changed.note.as_deref()), (RENDERED, Some("80 of 4,871 · type to narrow")));
    }

    #[test]
    fn should_show_a_worktree_scope_rows_and_the_project_own() {
        // given
        let mut sources = sources();
        sources.recent.push(Item { project: Some(atlassian()), ..Item::new(Target::Reopen(SavedTab::GitLog { project: "atlassian".to_owned(), view: "log".to_owned() }), "Git log", "history") });
        let state = HubState { scope: HubScope::Worktree(dev()), ..HubState::default() };

        // when
        let list = shown(&state, &sources);

        // then
        assert_eq!(titles(&list), ["Git log", "README.md"]);
    }

    #[test]
    fn should_clamp_the_selection_when_rows_shrink() {
        // given
        let mut state = HubState { selected: 5, ..HubState::default() };

        // when
        let list = shown(&state, &sources());
        state.fit_selection(&list);

        // then
        assert_eq!(state.selected, 3);
    }

    #[test]
    fn should_note_loading_files() {
        // given
        let sources = Sources { loading: true, ..sources() };
        let mut state = HubState::default();
        typed(&mut state, "x");

        // when
        let list = shown(&state, &sources);

        // then
        let files = list.sections.iter().find(|section| section.title == "Files");
        assert_eq!(files.and_then(|section| section.note.as_deref()), Some(LOADING_FILES));
    }

    #[test]
    fn should_say_how_many_more_matched() {
        // given
        let names: Vec<String> = (0..100).map(|number| format!("f{number}.rs")).collect();
        let files = listing(&names.iter().map(String::as_str).collect::<Vec<_>>());
        let corpus = build(&[ProjectInput { checkouts: vec![Some(&files)], ignored: Vec::new(), scratches: &[] }]);
        let sources = Sources { projects: vec![sources().projects[1].clone()], ..Sources::default() };
        let mut state = HubState::default();
        typed(&mut state, "rs");
        let found = search(&corpus, &crate::hub::matcher::Query::parse("rs"), &state.filter(&sources), &Generations::default().next(), |_| {}).unwrap();

        // when
        let list = list(&state, &sources, Some((&corpus, &found)), &NoStates);

        // then
        assert_eq!((list.rows.len(), list.more), (80, 20));
    }

    #[test]
    fn should_fall_back_to_all_for_a_scope_no_longer_open() {
        // given
        let gone = ProjectId::parse("gone").unwrap();
        let pick = Pick {
            title: "x".to_owned(),
            target: Target::File { project: gone.clone(), path: "x".to_owned() },
            query: String::new(),
            selected: 0,
            behind: false,
            line: None,
            project: gone.clone(),
            checkouts: vec![PathBuf::from("/g")],
        };
        let state = HubState { scope: HubScope::Worktree(checkout(gone, "/g")), pick: Some(pick), ..HubState::default() };

        // when
        let fitted = state.fitted(&sources());

        // then
        assert_eq!((fitted.scope, fitted.pick), (HubScope::All, None));
    }

    #[test]
    fn should_type_and_copy_a_row_path() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "srcnew");

        // when
        let typed_path = act(&mut state, Builtin::HubTypePath, &sources());
        let copied = act(&mut state, Builtin::HubCopyPath, &sources());

        // then
        assert_eq!(typed_path, Effect::TypePath("/a/ive/src/new.rs".to_owned()));
        assert_eq!(copied, Effect::CopyPath("src/new.rs".to_owned()));
    }

    #[test]
    fn should_copy_the_full_path_of_a_row() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "srcnew");

        // when
        let copied = act(&mut state, Builtin::CopyFullPath, &sources());

        // then
        assert_eq!(copied, Effect::CopyPath("/a/ive/src/new.rs".to_owned()));
    }

    #[test]
    fn should_stop_the_process_of_a_process_row() {
        // given
        let process = Target::Process(ProcessId::for_tests(7));
        let sources = Sources { running: vec![Item::new(process.clone(), "cargo run", "terminal")], ..sources() };
        let mut state = HubState::default();
        state.selected = titles(&shown(&state, &sources)).iter().position(|title| *title == "cargo run").unwrap();

        // when
        let effect = act(&mut state, Builtin::HubStop, &sources);

        // then
        assert_eq!(effect, Effect::Stop(process));
    }

    #[test]
    fn should_stop_nothing_on_a_file_row() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "srcnew");

        // when
        let effect = act(&mut state, Builtin::HubStop, &sources());

        // then
        assert_eq!(effect, Effect::Nothing);
    }

    #[test]
    fn should_toggle_excluded_files() {
        // given
        let mut state = HubState::default();

        // when
        let effect = act(&mut state, Builtin::HubToggleExcluded, &sources());

        // then
        assert_eq!((effect, state.excluded, state.filter(&sources()).excluded), (Effect::Search, true, true));
    }

    #[test]
    fn should_round_trip_the_state_as_json() {
        // given
        let sources = Sources { recent: Vec::new(), ..sources() };
        let mut state = HubState { scope: HubScope::Project(atlassian()), ..HubState::default() };
        typed(&mut state, "srctheme");
        act(&mut state, Builtin::HubOpen, &sources);

        // when
        let json = serde_json::to_value(&state).unwrap();
        let back: HubState = serde_json::from_value(json).unwrap();

        // then
        assert_eq!(back, state);
    }

    #[test]
    fn should_read_an_empty_object_as_a_fresh_hub() {
        // when
        let state: HubState = serde_json::from_value(serde_json::json!({})).unwrap();

        // then
        assert_eq!(state, HubState::default());
    }

    #[test]
    fn should_scope_a_worktree_past_the_64th_to_its_project() {
        // given
        let project = atlassian();
        let worktrees = (0..65).map(|number| worktree(checkout(project.clone(), &format!("/w{number}")), &format!("w{number}"), None)).collect();
        let sources = Sources { projects: vec![ProjectInfo { id: project.clone(), name: "atlassian".to_owned(), git: true, worktrees }], ..Sources::default() };
        let state = HubState { scope: HubScope::Worktree(checkout(project, "/w64")), ..HubState::default() };

        // when
        let filter = state.filter(&sources);

        // then
        assert_eq!((filter.project, filter.checkout), (Some(0), None));
    }

    #[test]
    fn should_keep_the_query_when_a_scope_chip_is_clicked() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "theme");

        // when
        let effect = state.pick_scope(HubScope::Worktree(dev()), &sources());

        // then
        assert_eq!((effect, state.query.as_str(), state.scope.clone()), (Effect::Search, "theme", HubScope::Worktree(dev())));
    }

    fn branch(name: &str, remote: Option<&str>, when: i64) -> FreeBranch {
        let shown = remote.map_or_else(|| name.to_owned(), |remote| format!("{remote}/{name}"));
        FreeBranch { project: atlassian(), local: name.to_owned(), shown, remote: remote.map(str::to_owned), when }
    }

    /// atlassian with `count` branches without a worktree, b1 the newest.
    fn with_branches(count: i64) -> Sources {
        Sources { branches: (1..=count).map(|number| branch(&format!("b{number}"), None, 1000 - number)).collect(), ..sources() }
    }

    fn new_terminal() -> HubState {
        HubState { new_terminal: true, ..HubState::default() }
    }

    fn more_lines(list: &List) -> Vec<Option<&str>> {
        list.sections.iter().map(|section| section.more.as_deref()).collect()
    }

    #[test]
    fn should_list_projects_worktrees_and_five_newest_branches() {
        // given
        let sources = with_branches(7);

        // when
        let list = shown(&new_terminal(), &sources);

        // then
        assert_eq!(sections(&list), ["atlassian", "notes"]);
        assert_eq!(titles(&list), ["dev", "GEN-817-ive", "old", "b1", "b2", "b3", "b4", "b5", "notes"]);
        assert_eq!(more_lines(&list), [Some("… 2 more branches; type to search them"), None]);
    }

    #[test]
    fn should_describe_the_main_checkout_and_a_plain_folder() {
        // when
        let list = shown(&new_terminal(), &sources());

        // then
        let details: Vec<&str> = list.rows.iter().map(|row| row.item.detail.as_str()).collect();
        assert_eq!(details, ["atlassian · main checkout", "atlassian", "atlassian", "notes · no git"]);
    }

    #[test]
    fn should_mark_remote_only_branches() {
        // given
        let sources = Sources { branches: vec![branch("mine", None, 2), branch("theirs", Some("origin"), 1)], ..sources() };

        // when
        let list = shown(&new_terminal(), &sources);

        // then
        let rows: Vec<(&str, &str, bool)> = list.rows[3..5].iter().map(|row| (row.item.title.as_str(), row.item.detail.as_str(), row.item.faded)).collect();
        assert_eq!(rows, [("mine", "local · atlassian", false), ("theirs", "origin · atlassian", true)]);
    }

    #[test]
    fn should_rank_new_terminal_rows_when_typing() {
        // given
        let mut state = new_terminal();
        typed(&mut state, "817");

        // when
        let list = shown(&state, &with_branches(3));

        // then
        assert_eq!(sections(&list), ["New terminal in"]);
        assert_eq!(titles(&list), ["GEN-817-ive"]);
        assert_eq!(list.counter.as_deref(), Some("1/7"));
    }

    #[test]
    fn should_show_the_twenty_best_matches() {
        // given
        let mut state = new_terminal();
        typed(&mut state, "b");

        // when
        let list = shown(&state, &with_branches(30));

        // then
        assert_eq!((list.rows.len(), list.more), (20, 10));
    }

    #[test]
    fn should_treat_prefixes_as_text_in_new_terminal_mode() {
        // given
        let mut state = new_terminal();

        // when
        typed(&mut state, ":x");

        // then
        assert_eq!((mode(&state), state.search_text()), (Mode::NewTerminal, None));
    }

    #[test]
    fn should_leave_new_terminal_mode_for_the_hub_with_toggle_changed() {
        // given
        let mut state = new_terminal();

        // when
        let effect = act(&mut state, Builtin::HubToggleChanged, &sources());

        // then
        assert_eq!((effect, state.new_terminal, state.changed_only), (Effect::Search, false, false));
    }

    #[test]
    fn should_turn_the_picker_into_the_hub_for_ctrl_shift_e() {
        // given
        let mut state = new_terminal();
        typed(&mut state, "topic");
        state.selected = 2;

        // when
        state.show_as_hub();

        // then
        assert_eq!((state.new_terminal, state.query.as_str(), state.selected), (false, "", 0));
    }

    #[test]
    fn should_keep_the_hub_as_it_is_for_ctrl_shift_e() {
        // given
        let mut state = HubState::default();
        typed(&mut state, "theme");

        // when
        state.show_as_hub();

        // then
        assert_eq!(state.query, "theme");
    }

    #[test]
    fn should_open_a_terminal_in_the_picked_worktree() {
        // given
        let mut state = new_terminal();
        state.selected = 1;

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources());

        // then
        assert_eq!(effect, Effect::Open { target: Target::NewTerminal(ive()), checkout: None, line: None, behind: false });
    }

    #[test]
    fn should_add_a_worktree_for_a_picked_remote_branch() {
        // given
        let sources = Sources { branches: vec![branch("theirs", Some("origin"), 1)], ..sources() };
        let mut state = new_terminal();
        state.selected = 3;

        // when
        let effect = act(&mut state, Builtin::HubOpen, &sources);

        // then
        let target = Target::NewWorktreeTerminal { project: atlassian(), branch: "theirs".to_owned(), remote: Some("origin".to_owned()) };
        assert_eq!(effect, Effect::Open { target, checkout: None, line: None, behind: false });
    }
}
