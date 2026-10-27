//! The rows the hub shows for a state: its sections, the matches highlighted, the numbers 1-9.

use std::ops::Range;
use std::path::Path;

use super::content::{ContentFound, ContentHit, MAX_MATCHES, MIN_QUERY_CHARS};
use super::corpus::{Corpus, Origin};
use super::matcher::{Query, lowered, split_line};
use super::model::{HubState, Mode, mode};
use super::output::OutputHit;
use super::search::{Found, RENDERED};
use super::sources::{FreeBranch, HubScope, Item, Place, ProjectInfo, Sources, StateIn, Target, WorktreeInfo};
use crate::icons;
use crate::process::ProcessId;
use crate::project::{CheckoutRef, ProjectId};
use crate::scratch::ScratchRef;

/// Recent rows on an empty query, as the mock shows.
pub const RECENT_EMPTY: usize = 12;
pub const RECENT_WHILE_SEARCHING: usize = 5;
/// Changed rows beside Recent and Running, the last written; the changed-only list shows them all.
pub const CHANGED_EMPTY: usize = 10;
pub const LOADING_FILES: &str = "Loading the file lists…";
pub const LINE_NEEDS_TEXT: &str = "Line numbers go to a file shown before the hub";
pub const CONTENT_NEEDS_TEXT: &str = "Type 2 characters or more to search the files' contents";
pub const SEARCHING_CONTENTS: &str = "Searching…";
pub const NO_CONTENT_MATCHES: &str = "No line matches";
/// New terminal mode: the best matches while typing.
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
        Mode::Empty if state.changed_only => list.section_of("Changed", list.matching(&sources.changed, "", RENDERED)),
        Mode::Empty => {
            list.section_of("Waiting", list.matching(&sources.waiting, "", RENDERED));
            let recent = list.matching(&sources.recent, "", RECENT_EMPTY).rows;
            list.section("Recent", recent);
            list.latest("Changed", list.matching(&sources.changed, "", CHANGED_EMPTY));
            list.section_of("Running", list.matching(&sources.running, "", RENDERED));
        }
        Mode::Files { .. } if state.changed_only => list.section_of("Changed", list.matching(&sources.changed, split_line(&state.query).query, RENDERED)),
        Mode::Files { .. } => {
            list.section("Recent", recent_matches(state, sources));
            list.files(sources, files);
        }
        Mode::Actions => list.section_of("Actions", list.matching(&sources.actions, rest(&state.query), RENDERED)),
        Mode::Processes => {
            list.section_of("Processes", list.matching(&sources.processes, rest(&state.query), RENDERED));
            list.section_of("Run configurations", list.matching(&sources.configs, rest(&state.query), RENDERED));
        }
        Mode::Worktrees => {
            list.section_of("Worktrees", list.matching(&worktree_items(sources), rest(&state.query), RENDERED));
            list.section_of("Branches", list.matching(&branch_items(sources), rest(&state.query), RENDERED));
        }
        Mode::Scopes => list.section("Scope", scope_rows(sources, rest(&state.query))),
        Mode::Line(line) => match &sources.back {
            Some(back) if back.is_text => list.section("Line", vec![plain(Item::new(Target::Line(line), format!("Line {line}"), icons::FILE_TEXT))]),
            _ => list.message = Some(LINE_NEEDS_TEXT.to_owned()),
        },
        Mode::NewTerminal => list.new_terminal(&state.query, sources),
        // the workspace lists the contents search's lines itself (`content_list`)
        Mode::Content => {}
    }
    list.finish()
}

/// Where a contents search's file is, as rows show it.
pub struct Located {
    pub place: Place,
    pub project: Option<ProjectId>,
    /// relative to its checkout, or absolute outside every checkout
    pub path: String,
}

/// Where the rows of a contents search are: each file in its checkout, each process in the checkout it works in.
pub struct Places<'a> {
    pub file: &'a dyn Fn(&Path) -> Located,
    pub process: &'a dyn Fn(ProcessId) -> Option<CheckoutRef>,
}

/// The rows of a contents search: one per matching line, `path:line` and the line with its matches, then the lines of
/// processes' output as `title:line`.
pub fn content_list(state: &HubState, found: Option<&ContentFound>, places: &Places) -> List {
    let mut list = Builder::new(&state.scope);
    let title = match &state.paths[..] {
        [] => "Contents".to_owned(),
        [only] => format!("Contents of {}", file_name(only)),
        [first, rest @ ..] => format!("Contents of {} +{}", file_name(first), rest.len()),
    };
    if let Some(Err(error)) = state.content_query().map(|query| query.compile()) {
        list.message = Some(error);
        return list.finish();
    }
    let Some(found) = found.filter(|_| state.query.chars().count() >= MIN_QUERY_CHARS) else {
        list.message = Some(if state.query.chars().count() < MIN_QUERY_CHARS { CONTENT_NEEDS_TEXT } else { SEARCHING_CONTENTS }.to_owned());
        return list.finish();
    };
    // up to 2,000 hits: only the shown ones are made rows
    let rows = found.hits.iter().take(RENDERED).map(|hit| hit_row(hit, (places.file)(&hit.path))).collect();
    let output = found.output.iter().take(RENDERED).map(|hit| output_row(hit, (places.process)(hit.process))).collect();
    if found.hits.is_empty() && found.output.is_empty() && found.done {
        list.message = Some(NO_CONTENT_MATCHES.to_owned());
    }
    list.section_of(&title, Matched { rows, total: found.hits.len() });
    list.section_of("Output", Matched { rows: output, total: found.output.len() });
    if found.limited
        && let Some(section) = list.sections.last_mut()
    {
        section.more = Some(format!("the first {} matches", MAX_MATCHES));
    }
    let searching = if found.done { "" } else { " …" };
    let mut list = list.finish();
    let counted = |count: usize, one: &str, many: &str| format!("{count} {}", if count == 1 { one } else { many });
    let mut processes: Vec<ProcessId> = found.output.iter().map(|hit| hit.process).collect();
    processes.dedup();
    let matches = counted(found.hits.len() + found.output.len(), "match", "matches");
    let places = match processes.len() {
        0 => counted(found.files, "file", "files"),
        count => format!("{} and {}", counted(found.files, "file", "files"), counted(count, "process", "processes")),
    };
    list.counter = Some(format!("{matches} in {places} · {} ms{searching}", found.millis));
    list
}

fn hit_row(hit: &ContentHit, located: Located) -> Row {
    let mut item = Item::new(Target::Hit { path: hit.path.clone(), line: hit.line, bytes: hit.bytes.clone() }, format!("{}:{}", located.path, hit.line), icons::file_icon(&located.path));
    item.detail = hit.text.clone();
    item.place = located.place;
    item.project = located.project;
    item.path = Some(located.path);
    // the matches' bytes as char positions, as rows mark them
    let detail_hits = hit.text.char_indices().enumerate().filter(|(_, (at, _))| hit.ranges.iter().any(|range| range.contains(at))).map(|(index, _)| index).collect();
    Row { item, title_hits: Vec::new(), detail_hits, number: None, tag: Tag::None, depth: 0 }
}

fn output_row(hit: &OutputHit, checkout: Option<CheckoutRef>) -> Row {
    let mut item = Item::new(Target::Output(Box::new(hit.clone())), format!("{}:{}", hit.title, hit.line), icons::TERMINAL);
    item.detail = hit.text.clone();
    item.project = checkout.as_ref().map(|checkout| checkout.project.clone());
    item.place = checkout.map_or(Place::Nowhere, Place::One);
    let detail_hits = hit.text.char_indices().enumerate().filter(|(_, (at, _))| hit.ranges.iter().any(|range| range.contains(at))).map(|(index, _)| index).collect();
    Row { item, title_hits: Vec::new(), detail_hits, number: None, tag: Tag::None, depth: 0 }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

/// The Recent rows a files query matches, which the files under them leave out.
pub(super) fn recent_matches(state: &HubState, sources: &Sources) -> Vec<Row> {
    let builder = Builder::new(&state.scope);
    builder.matching(&sources.recent, split_line(&state.query).query, RECENT_WHILE_SEARCHING).rows
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
    fn section(&mut self, title: &str, rows: Vec<Row>) {
        let total = rows.len();
        self.section_of(title, Matched { rows, total });
    }

    /// Adds a section of the first of `matched`'s rows, saying how many there are when some are left out.
    fn section_of(&mut self, title: &str, matched: Matched) {
        let Matched { mut rows, total } = matched;
        if total == 0 {
            return;
        }
        let note = (total > RENDERED).then(|| format!("{RENDERED} of {} · type to narrow", thousands(total)));
        rows.truncate(RENDERED);
        self.push(title, rows, note);
    }

    /// A section of the first rows of `matched`, saying how many there are when they are not all shown.
    fn latest(&mut self, title: &str, matched: Matched) {
        let Matched { rows, total } = matched;
        if total == 0 {
            return;
        }
        let note = (total > rows.len()).then(|| format!("last {} of {}", rows.len(), thousands(total)));
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

    /// Ctrl+Shift+T: before typing, a section per project with its checkouts; typing ranks every checkout and branch
    /// of every project, the best first, as `project.name` so that typing the project finds them all.
    fn new_terminal(&mut self, query: &str, sources: &Sources) {
        let places = new_terminal_places(sources);
        if Query::parse(query).is_empty() {
            for place in places {
                let branches = place.branches.len();
                let more = (branches > 0).then(|| format!("… {branches} branches; type to search them"));
                self.push_with_more(&place.name, place.checkouts.into_iter().map(plain).collect(), None, more);
            }
            return;
        }
        let items: Vec<(String, Item)> = places.into_iter().flat_map(|place| place.checkouts.into_iter().chain(place.branches).map(move |item| (place.name.clone(), item))).collect();
        let found = matched_in_projects(&items, query, NEW_TERMINAL_MATCHES);
        self.counter = Some(format!("{}/{}", found.total, items.len()));
        self.more = found.total.saturating_sub(NEW_TERMINAL_MATCHES);
        self.push("New terminal in", found.rows, None);
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

    /// The first `limit` items in scope that `text` matches over "title detail", best first; all of them in order for
    /// no text.
    fn matching(&self, items: &[Item], text: &str, limit: usize) -> Matched {
        let scoped = items.iter().filter(|item| in_scope(item, self.scope));
        let found = matched(scoped.map(|item| (item, 0)), text, Order::Ranked, limit);
        let rows = found.rows.into_iter().map(|row| Row { tag: tag(&row.item.place, self.scope), ..row }).collect();
        Matched { rows, total: found.total }
    }

    fn finish(mut self) -> List {
        for (index, row) in self.rows.iter_mut().enumerate().take(NUMBERED) {
            row.number = u8::try_from(index + 1).ok();
        }
        List { sections: self.sections, rows: self.rows, more: self.more, counter: self.counter, message: self.message }
    }
}

/// Rows matched, the first of them: `total` counts all.
struct Matched {
    rows: Vec<Row>,
    total: usize,
}

/// How matches are ordered: best first, or as the items come.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Order {
    Ranked,
    Listed,
}

/// Matches `items` (with their depth) against `text` and makes rows of the first `limit` hits only: the hub matches
/// thousands of items every frame. A stable sort keeps hits of equal score in their order.
fn matched<'a>(items: impl Iterator<Item = (&'a Item, u8)>, text: &str, order: Order, limit: usize) -> Matched {
    let query = Query::parse(text);
    if query.is_empty() {
        let mut rows = Vec::new();
        let mut total = 0;
        for (item, depth) in items {
            if total < limit {
                rows.push(Row { depth, ..plain(item.clone()) });
            }
            total += 1;
        }
        return Matched { rows, total };
    }
    let mut hits: Vec<(i32, &Item, u8, Vec<usize>)> = items
        .filter_map(|(item, depth)| {
            let hay = format!("{} {}", item.title, item.detail);
            let hit = query.matches(&hay, &lowered(&hay), None)?;
            Some((hit.score, item, depth, hit.positions))
        })
        .collect();
    if order == Order::Ranked {
        hits.sort_by_key(|(score, ..)| std::cmp::Reverse(*score));
    }
    let total = hits.len();
    let rows = hits.into_iter().take(limit).map(|(_, item, depth, positions)| hit_row_of(item, depth, &positions)).collect();
    Matched { rows, total }
}

/// [`matched`] for items of named projects, best first: each is matched as "project.title detail", and the project's
/// part highlights nothing.
fn matched_in_projects(items: &[(String, Item)], text: &str, limit: usize) -> Matched {
    let query = Query::parse(text);
    let mut hits: Vec<(i32, &Item, Vec<usize>)> = items
        .iter()
        .filter_map(|(project, item)| {
            let hay = format!("{project}.{} {}", item.title, item.detail);
            let hit = query.matches(&hay, &lowered(&hay), None)?;
            let prefix = project.chars().count() + 1;
            let positions = hit.positions.iter().filter_map(|at| at.checked_sub(prefix)).collect();
            Some((hit.score, item, positions))
        })
        .collect();
    hits.sort_by_key(|(score, ..)| std::cmp::Reverse(*score));
    let total = hits.len();
    let rows = hits.into_iter().take(limit).map(|(_, item, positions)| hit_row_of(item, 0, &positions)).collect();
    Matched { rows, total }
}

fn hit_row_of(item: &Item, depth: u8, positions: &[usize]) -> Row {
    let title_chars = item.title.chars().count();
    let title_hits = positions.iter().copied().filter(|at| *at < title_chars).collect();
    let detail_hits = positions.iter().filter(|at| **at > title_chars).map(|at| at - title_chars - 1).collect();
    Row { title_hits, detail_hits, depth, ..plain(item.clone()) }
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
    let items: Vec<(Item, u8)> = items.collect();
    let found = matched(items.iter().map(|(item, depth)| (item, *depth)), &pick.query, Order::Ranked, usize::MAX);
    found.rows.into_iter().map(|row| Row { tag: tag(&row.item.place, &HubScope::All), ..row }).collect()
}

/// Where a new terminal can go in one project: its checkouts (the main one first, the others most recently used
/// first), then its branches without a worktree, newest first.
struct TerminalPlaces {
    name: String,
    checkouts: Vec<Item>,
    branches: Vec<Item>,
}

fn new_terminal_places(sources: &Sources) -> Vec<TerminalPlaces> {
    sources.shown_projects().map(|project| TerminalPlaces { name: project.name.clone(), checkouts: checkout_places(project, sources), branches: branch_places(project, sources) }).collect()
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
        sources.shown_projects().flat_map(|project| project.worktrees.iter().map(move |worktree| (project.name.as_str(), worktree))).collect();
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

/// `:`: all projects listed, then each with its worktrees nested, most recently used first; with one project listed,
/// All is titled with it and its own row is left out.
fn scope_rows(sources: &Sources, text: &str) -> Vec<Row> {
    let mut items = vec![(Item::new(Target::Scope(HubScope::All), sources.all_label(), icons::PROJECT_FOLDER), 0)];
    let only = sources.only_shown().map(|one| one.id.clone());
    for project in sources.shown_projects() {
        let row = Item { detail: "project".to_owned(), project: Some(project.id.clone()), ..Item::new(Target::Scope(HubScope::Project(project.id.clone())), project.name.clone(), icons::PROJECT_FOLDER) };
        if only.as_ref() != Some(&project.id) {
            items.push((row, 0));
        }
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
    matched(items.iter().map(|(item, depth)| (item, *depth)), text, Order::Listed, usize::MAX).rows
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::hub::content::Matching;

    fn nowhere(path: &Path) -> Located {
        Located { place: Place::Nowhere, project: None, path: path.display().to_string() }
    }

    #[test]
    fn should_say_why_a_regex_does_not_compile() {
        // given
        let state = HubState { content: true, matching: Matching::Regex, query: "foo(".to_owned(), ..HubState::default() };
        let old = ContentFound { hits: vec![ContentHit { path: PathBuf::from("/a"), line: 1, text: "foo".to_owned(), ranges: std::iter::once(0..3).collect(), bytes: 0..3 }], files: 1, done: true, ..ContentFound::default() };

        // when
        let list = content_list(&state, Some(&old), &Places { file: &nowhere, process: &no_process });

        // then
        assert!(list.rows.is_empty(), "{:?}", list.rows);
        assert!(list.message.as_deref().is_some_and(|message| message.contains("foo(")), "{:?}", list.message);
    }

    fn no_process(_process: crate::process::ProcessId) -> Option<crate::project::CheckoutRef> {
        None
    }

    fn found_in_both() -> ContentFound {
        let hit = ContentHit { path: PathBuf::from("/a"), line: 1, text: "foo".to_owned(), ranges: std::iter::once(0..3).collect(), bytes: 0..3 };
        let output = OutputHit {
            process: crate::process::ProcessId::for_tests(1),
            title: "cargo".to_owned(),
            line: 3,
            top: "a foo".to_owned(),
            whole: "a foo".to_owned(),
            text: "a foo".to_owned(),
            ranges: std::iter::once(2..5).collect(),
            bytes: 2..5,
        };
        ContentFound { hits: vec![hit], output: vec![output], files: 1, done: true, ..ContentFound::default() }
    }

    #[test]
    fn should_list_output_hits_after_contents() {
        // given
        let state = HubState { content: true, query: "foo".to_owned(), ..HubState::default() };

        // when
        let list = content_list(&state, Some(&found_in_both()), &Places { file: &nowhere, process: &no_process });

        // then
        let sections: Vec<&str> = list.sections.iter().map(|section| section.title.as_str()).collect();
        let titles: Vec<&str> = list.rows.iter().map(|row| row.item.title.as_str()).collect();
        assert_eq!((sections, titles), (vec!["Contents", "Output"], vec!["/a:1", "cargo:3"]));
    }

    #[test]
    fn should_count_output_and_file_matches_together() {
        // given
        let state = HubState { content: true, query: "foo".to_owned(), ..HubState::default() };

        // when
        let list = content_list(&state, Some(&found_in_both()), &Places { file: &nowhere, process: &no_process });

        // then
        let counter = list.counter.unwrap_or_default();
        assert!(counter.starts_with("2 matches in 1 file and 1 process"), "{counter}");
    }

    #[test]
    fn should_count_every_match_but_make_rows_of_the_first_only() {
        // given
        let items: Vec<Item> = (0..300).map(|n| Item::new(Target::Line(n), format!("branch-{n}"), icons::BRANCH)).collect();
        let scoped = || items.iter().map(|item| (item, 0));
        let query = Query::parse("branch-1");
        let matching = items.iter().filter(|item| query.matches(&format!("{} ", item.title), &format!("{} ", item.title), None).is_some()).count();

        // when
        let typed = matched(scoped(), "branch-1", Order::Ranked, 5);
        let empty = matched(scoped(), "", Order::Listed, 5);

        // then
        assert!(matching > 100, "{matching}");
        assert_eq!((typed.rows.len(), typed.total), (5, matching));
        assert_eq!((empty.rows.len(), empty.total, empty.rows[4].item.title.as_str()), (5, 300, "branch-4"));
    }
}
