//! The rows the hub shows for a state: its sections, the matches highlighted, the numbers 1-9.

use std::ops::Range;

use super::corpus::{Corpus, Origin};
use super::matcher::{Query, lowered, split_line};
use super::model::{HubState, Mode, mode};
use super::search::{Found, RENDERED};
use super::sources::{FreeBranch, HubScope, Item, Place, ProjectInfo, Sources, StateIn, Target, WorktreeInfo};
use crate::icons;
use crate::project::{CheckoutRef, ProjectId};
use crate::scratch::ScratchRef;

/// Recent rows on an empty query, as the mock shows.
pub const RECENT_EMPTY: usize = 12;
pub const RECENT_WHILE_SEARCHING: usize = 5;
pub const LOADING_FILES: &str = "Loading the file lists…";
pub const LINE_NEEDS_TEXT: &str = "Line numbers go to a file shown before the hub";
/// New terminal mode: a project's newest branches without a worktree before typing, the best matches while typing.
pub const NEW_TERMINAL_BRANCHES: usize = 5;
pub const NEW_TERMINAL_MATCHES: usize = 20;
const NUMBERED: usize = 9;

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub sections: Vec<Section>,
    /// flat, as shown
    pub rows: Vec<Row>,
    /// files matched but not shown
    pub more: usize,
    /// `matched/total · ms`, while searching files
    pub counter: Option<String>,
    /// said instead of rows
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub title: String,
    pub rows: Range<usize>,
    pub note: Option<String>,
    /// a muted line under the rows: what is not listed
    pub more: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub item: Item,
    /// char positions in the title and the detail that matched
    pub title_hits: Vec<usize>,
    pub detail_hits: Vec<usize>,
    /// 1-9, for Alt+1…9
    pub number: Option<u8>,
    pub tag: Tag,
    /// 1 for a worktree under its project in the `:` list
    pub depth: u8,
}

/// Where a row's thing is, as its tag shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    None,
    Worktree(CheckoutRef),
    /// `N worktrees`
    Several(usize),
}

/// The list for `state`; `files` is the search's result for a files query.
pub fn list(state: &HubState, sources: &Sources, files: Option<(&Corpus, &Found)>, states: &dyn StateIn) -> List {
    let mut list = Builder::new(&state.scope);
    match mode(state) {
        Mode::Pick => list.section("Worktrees", pick_rows(state, sources, states)),
        Mode::Empty if state.changed_only => list.section("Changed", list.matching(&sources.changed, "")),
        Mode::Empty => {
            let recent: Vec<Row> = list.matching(&sources.recent, "").into_iter().take(RECENT_EMPTY).collect();
            list.section("Recent", recent);
            list.section("Changed", list.matching(&sources.changed, ""));
            list.section("Running", list.matching(&sources.running, ""));
        }
        Mode::Files { .. } if state.changed_only => list.section("Changed", list.matching(&sources.changed, split_line(&state.query).query)),
        Mode::Files { .. } => {
            list.section("Recent", recent_matches(state, sources));
            list.files(sources, files);
        }
        Mode::Actions => list.section("Actions", list.matching(&sources.actions, rest(&state.query))),
        Mode::Processes => {
            list.section("Processes", list.matching(&sources.processes, rest(&state.query)));
            list.section("Run configurations", list.matching(&sources.configs, rest(&state.query)));
        }
        Mode::Worktrees => {
            list.section("Worktrees", list.matching(&worktree_items(sources), rest(&state.query)));
            list.section("Branches", list.matching(&branch_items(sources), rest(&state.query)));
        }
        Mode::Scopes => list.section("Scope", scope_rows(sources, rest(&state.query))),
        Mode::Line(line) => match &sources.back {
            Some(back) if back.is_text => list.section("Line", vec![plain(Item::new(Target::Line(line), format!("Line {line}"), icons::FILE_TEXT))]),
            _ => list.message = Some(LINE_NEEDS_TEXT.to_owned()),
        },
        Mode::NewTerminal => list.new_terminal(&state.query, sources),
    }
    list.finish()
}

/// The Recent rows a files query matches, which the files under them leave out.
pub(super) fn recent_matches(state: &HubState, sources: &Sources) -> Vec<Row> {
    let builder = Builder::new(&state.scope);
    builder.matching(&sources.recent, split_line(&state.query).query).into_iter().take(RECENT_WHILE_SEARCHING).collect()
}

/// The text after a one-character prefix.
fn rest(query: &str) -> &str {
    query.get(1..).unwrap_or_default()
}

struct Builder<'a> {
    scope: &'a HubScope,
    sections: Vec<Section>,
    rows: Vec<Row>,
    more: usize,
    counter: Option<String>,
    message: Option<String>,
}

impl<'a> Builder<'a> {
    fn new(scope: &'a HubScope) -> Self {
        Self { scope, sections: Vec::new(), rows: Vec::new(), more: 0, counter: None, message: None }
    }

    /// Adds a section unless it has no rows; a long one shows its first rows only, as drawing thousands is slow.
    fn section(&mut self, title: &str, mut rows: Vec<Row>) {
        if rows.is_empty() {
            return;
        }
        let note = (rows.len() > RENDERED).then(|| format!("{RENDERED} of {} · type to narrow", thousands(rows.len())));
        rows.truncate(RENDERED);
        self.push(title, rows, note);
    }

    fn push(&mut self, title: &str, rows: Vec<Row>, note: Option<String>) {
        self.push_with_more(title, rows, note, None);
    }

    fn push_with_more(&mut self, title: &str, rows: Vec<Row>, note: Option<String>, more: Option<String>) {
        let start = self.rows.len();
        self.rows.extend(rows);
        self.sections.push(Section { title: title.to_owned(), rows: start..self.rows.len(), note, more });
    }

    /// Ctrl+Shift+T: before typing, a section per project with its places and newest branches; typing ranks every
    /// place of every project, the best first.
    fn new_terminal(&mut self, query: &str, sources: &Sources) {
        let places = new_terminal_places(sources);
        if Query::parse(query).is_empty() {
            for place in places {
                let hidden = place.branches.len().saturating_sub(NEW_TERMINAL_BRANCHES);
                let more = (hidden > 0).then(|| format!("… {hidden} more branches; type to search them"));
                let rows = place.checkouts.into_iter().chain(place.branches.into_iter().take(NEW_TERMINAL_BRANCHES)).map(plain).collect();
                self.push_with_more(&place.name, rows, None, more);
            }
            return;
        }
        let items: Vec<Item> = places.into_iter().flat_map(|place| place.checkouts.into_iter().chain(place.branches)).collect();
        let total = items.len();
        let mut found = matched(items.into_iter().map(|item| (item, 0)), query, true);
        self.counter = Some(format!("{}/{total}", found.len()));
        self.more = found.len().saturating_sub(NEW_TERMINAL_MATCHES);
        found.truncate(NEW_TERMINAL_MATCHES);
        self.push("New terminal in", found, None);
    }

    /// Files, always shown while searching, with the counter and what is still loading.
    fn files(&mut self, sources: &Sources, files: Option<(&Corpus, &Found)>) {
        let rows: Vec<Row> = files.map_or_else(Vec::new, |(corpus, found)| found.hits.iter().filter_map(|hit| file_row(corpus, hit, sources, self.scope)).collect());
        if let Some((_, found)) = files {
            self.more = found.matched.saturating_sub(rows.len());
            self.counter = Some(format!("{}/{} · {} ms", found.matched, found.total, found.millis));
        }
        let note = sources.loading.then(|| LOADING_FILES.to_owned());
        self.push("Files", rows, note);
    }

    /// The items in scope that `text` matches over "title detail", best first; all of them in order for no text.
    fn matching(&self, items: &[Item], text: &str) -> Vec<Row> {
        let scoped = items.iter().filter(|item| in_scope(item, self.scope));
        let rows = matched(scoped.map(|item| (item.clone(), 0)), text, true);
        rows.into_iter().map(|row| Row { tag: tag(&row.item.place, self.scope), ..row }).collect()
    }

    fn finish(mut self) -> List {
        for (index, row) in self.rows.iter_mut().enumerate().take(NUMBERED) {
            row.number = u8::try_from(index + 1).ok();
        }
        List { sections: self.sections, rows: self.rows, more: self.more, counter: self.counter, message: self.message }
    }
}

/// Matches `items` (with their depth) against `text`; `rank` sorts the hits best first, else they keep their order.
fn matched(items: impl Iterator<Item = (Item, u8)>, text: &str, rank: bool) -> Vec<Row> {
    let query = Query::parse(text);
    if query.is_empty() {
        return items.map(|(item, depth)| Row { depth, ..plain(item) }).collect();
    }
    let mut hits: Vec<(i32, Row)> = items
        .filter_map(|(item, depth)| {
            let hay = format!("{} {}", item.title, item.detail);
            let hit = query.matches(&hay, &lowered(&hay), None)?;
            let title_chars = item.title.chars().count();
            let title_hits = hit.positions.iter().copied().filter(|at| *at < title_chars).collect();
            let detail_hits = hit.positions.iter().filter(|at| **at > title_chars).map(|at| at - title_chars - 1).collect();
            Some((hit.score, Row { title_hits, detail_hits, depth, ..plain(item) }))
        })
        .collect();
    if rank {
        hits.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    }
    hits.into_iter().map(|(_, row)| row).collect()
}

fn plain(item: Item) -> Row {
    Row { item, title_hits: Vec::new(), detail_hits: Vec::new(), number: None, tag: Tag::None, depth: 0 }
}

/// A worktree scope shows that worktree's rows and the project's own; the other scopes show every row.
fn in_scope(item: &Item, scope: &HubScope) -> bool {
    let HubScope::Worktree(worktree) = scope else {
        return true;
    };
    match &item.place {
        Place::One(checkout) => checkout == worktree,
        Place::Several { checkouts, .. } => checkouts.contains(&worktree.path),
        Place::Nowhere => item.project.as_ref() == Some(&worktree.project),
    }
}

fn tag(place: &Place, scope: &HubScope) -> Tag {
    match (place, scope) {
        (Place::One(checkout), _) => Tag::Worktree(checkout.clone()),
        (Place::Several { checkouts, .. }, HubScope::Worktree(worktree)) if checkouts.contains(&worktree.path) => Tag::Worktree(worktree.clone()),
        (Place::Several { checkouts, .. }, _) => Tag::Several(checkouts.len()),
        (Place::Nowhere, _) => Tag::None,
    }
}

fn file_row(corpus: &Corpus, hit: &super::search::Hit, sources: &Sources, scope: &HubScope) -> Option<Row> {
    let entry = corpus.entries.get(hit.entry)?;
    let project = sources.projects.get(entry.project)?;
    let path = &*entry.path;
    let (folder, name) = path.rsplit_once('/').unwrap_or(("", path));
    let mut item = match entry.origin {
        Origin::Scratch => {
            let scratch = ScratchRef { project: project.id.clone(), name: name.to_owned() };
            Item::new(Target::Scratch(scratch), name, icons::file_icon(name))
        }
        Origin::File => {
            let checkouts: Vec<&CheckoutRef> =
                project.worktrees.iter().enumerate().filter(|(bit, _)| *bit < 64 && entry.checkouts & (1 << bit) != 0).map(|(_, worktree)| &worktree.checkout).collect();
            let place = match checkouts.as_slice() {
                [] => Place::Nowhere,
                [one] => Place::One((*one).clone()),
                several => Place::Several { project: project.id.clone(), checkouts: several.iter().map(|checkout| checkout.path.clone()).collect() },
            };
            Item { place, path: Some(path.to_owned()), ..Item::new(Target::File { project: project.id.clone(), path: path.to_owned() }, name, icons::file_icon(name)) }
        }
    };
    item.detail = folder.to_owned();
    item.project = Some(project.id.clone());
    item.class = entry.class;
    let name_at = folder.chars().count() + usize::from(!folder.is_empty());
    let title_hits = hit.positions.iter().filter(|at| **at >= name_at).map(|at| at - name_at).collect();
    let detail_hits = hit.positions.iter().copied().filter(|at| *at + 1 < name_at).collect();
    Some(Row { title_hits, detail_hits, tag: tag(&item.place, scope), ..plain(item) })
}

/// The worktree question: the row's worktrees, most recently used first, with the row's state in each.
fn pick_rows(state: &HubState, sources: &Sources, states: &dyn StateIn) -> Vec<Row> {
    let Some(pick) = &state.pick else {
        return Vec::new();
    };
    let items = sources.by_recency(&pick.project, Some(&pick.checkouts)).into_iter().map(|worktree| {
        let item = Item {
            detail: worktree.branch.clone().unwrap_or_default(),
            state: states.state_in(&pick.target, &worktree.checkout.path),
            place: Place::One(worktree.checkout.clone()),
            project: Some(pick.project.clone()),
            age: worktree.last_used,
            ..Item::new(pick.target.clone(), worktree.label.clone(), icons::BRANCH)
        };
        (item, 0)
    });
    matched(items, &pick.query, true).into_iter().map(|row| Row { tag: tag(&row.item.place, &HubScope::All), ..row }).collect()
}

/// Where a new terminal can go in one project: its checkouts (the main one first, the others most recently used
/// first), then its branches without a worktree, newest first.
struct TerminalPlaces {
    name: String,
    checkouts: Vec<Item>,
    branches: Vec<Item>,
}

fn new_terminal_places(sources: &Sources) -> Vec<TerminalPlaces> {
    sources.projects.iter().map(|project| TerminalPlaces { name: project.name.clone(), checkouts: checkout_places(project, sources), branches: branch_places(project, sources) }).collect()
}

fn checkout_places(project: &ProjectInfo, sources: &Sources) -> Vec<Item> {
    let Some(main) = project.worktrees.first() else {
        return Vec::new();
    };
    let running = |worktree: &WorktreeInfo| (worktree.running > 0).then(|| format!("{} running", worktree.running));
    let (title, detail, icon) = match project.git {
        true => (main.label.clone(), format!("{} · main checkout", project.name), icons::BRANCH),
        false => (project.name.clone(), format!("{} · no git", project.name), icons::PROJECT_FOLDER),
    };
    let main_item = Item { detail, state: running(main), project: Some(project.id.clone()), ..Item::new(Target::NewTerminal(main.checkout.clone()), title, icon) };
    let others = sources.by_recency(&project.id, None).into_iter().filter(|worktree| worktree.checkout != main.checkout).map(|worktree| {
        let branch = worktree.branch.as_ref().filter(|branch| **branch != worktree.label);
        Item {
            detail: branch.map_or_else(|| project.name.clone(), |branch| format!("{} · {branch}", project.name)),
            state: running(worktree),
            project: Some(project.id.clone()),
            age: worktree.last_used,
            ..Item::new(Target::NewTerminal(worktree.checkout.clone()), worktree.label.clone(), icons::BRANCH)
        }
    });
    std::iter::once(main_item).chain(others).collect()
}

fn branch_places(project: &ProjectInfo, sources: &Sources) -> Vec<Item> {
    let mine = sources.branches.iter().filter(|branch| branch.project == project.id);
    mine.map(|branch| {
        let target = Target::NewWorktreeTerminal { project: project.id.clone(), branch: branch.local.clone(), remote: branch.remote.clone() };
        Item {
            detail: format!("{} · {}", branch.remote.as_deref().unwrap_or("local"), project.name),
            faded: branch.remote.is_some(),
            project: Some(project.id.clone()),
            age: Some(branch.when),
            ..Item::new(target, branch.local.clone(), icons::BRANCH)
        }
    })
    .collect()
}

/// `#`: the branches without a worktree; Enter opens the New worktree card with one typed.
fn branch_items(sources: &Sources) -> Vec<Item> {
    let name = |project: &ProjectId| sources.projects.iter().find(|info| &info.id == project).map_or_else(|| project.to_string(), |info| info.name.clone());
    let item = |branch: &FreeBranch| {
        let target = Target::Branch { project: branch.project.clone(), name: branch.local.clone() };
        Item { detail: format!("{} · new worktree", name(&branch.project)), project: Some(branch.project.clone()), ..Item::new(target, branch.shown.clone(), icons::BRANCH) }
    };
    sources.branches.iter().map(item).collect()
}

/// `#`: every open worktree, most recently used first.
fn worktree_items(sources: &Sources) -> Vec<Item> {
    let mut worktrees: Vec<(&str, &WorktreeInfo)> =
        sources.projects.iter().flat_map(|project| project.worktrees.iter().map(move |worktree| (project.name.as_str(), worktree))).collect();
    worktrees.sort_by_key(|(_, worktree)| std::cmp::Reverse(worktree.last_used));
    let item = |(project, worktree): (&str, &WorktreeInfo)| {
        let detail = worktree.branch.as_ref().map_or_else(|| project.to_owned(), |branch| format!("{project} · {branch}"));
        Item {
            detail,
            place: Place::One(worktree.checkout.clone()),
            project: Some(worktree.checkout.project.clone()),
            age: worktree.last_used,
            ..Item::new(Target::Worktree(worktree.checkout.clone()), worktree.label.clone(), icons::BRANCH)
        }
    };
    worktrees.into_iter().map(item).collect()
}

/// `:`: all projects, then each project with its worktrees nested, most recently used first.
fn scope_rows(sources: &Sources, text: &str) -> Vec<Row> {
    let mut items = vec![(Item::new(Target::Scope(HubScope::All), "All projects", icons::PROJECT_FOLDER), 0)];
    for project in &sources.projects {
        let row = Item { detail: "project".to_owned(), project: Some(project.id.clone()), ..Item::new(Target::Scope(HubScope::Project(project.id.clone())), project.name.clone(), icons::PROJECT_FOLDER) };
        items.push((row, 0));
        for worktree in sources.by_recency(&project.id, None) {
            let row = Item {
                detail: worktree.branch.clone().unwrap_or_default(),
                state: (worktree.running > 0).then(|| format!("{} running", worktree.running)),
                age: worktree.last_used,
                project: Some(project.id.clone()),
                ..Item::new(Target::Scope(HubScope::Worktree(worktree.checkout.clone())), worktree.label.clone(), icons::BRANCH)
            };
            items.push((row, 1));
        }
    }
    matched(items.into_iter(), text, false)
}

/// `381877` as `381,877`.
pub fn thousands(count: usize) -> String {
    let digits = count.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}
