//! The rows of the run configurations tab: every `.run.toml` of the checkout, flat or as a folder tree, narrowed by
//! a filter. Pure: the tab feeds it the hub index's run files and draws what it returns.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::file::{Config, ConfigKind, RunFile};
use crate::overview::file_list::FileLayout;
use crate::sidebar::rows::RunRow;

/// A configuration with the folder of its `.run.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    /// checkout-relative, `/`-separated
    pub folder: String,
    pub config: Config,
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListItem {
    Folder {
        /// the fold key: the folder's full checkout-relative path
        path: String,
        /// the folder's name, or a chain of single-subfolder names (`internal-apps/misc`)
        label: String,
        /// the folder itself has a `.run.toml`
        run_file: bool,
        /// live runs of the configurations below
        live: usize,
        open: bool,
    },
    Config {
        /// index into the entries the rows were made from
        entry: usize,
        run: Option<RunRow>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRow {
    pub depth: usize,
    pub item: ListItem,
}

/// `<name> (<run|serve>.<key>): <command as written>`; variables stay `${name}`, so no secret shows.
pub fn config_hint(config: &Config) -> String {
    let table = match &config.kind {
        ConfigKind::Run(_) => "run",
        ConfigKind::Serve(_) => "serve",
    };
    format!("{} ({table}.{}): {}", config.name, config.key, command_text(config))
}

fn command_text(config: &Config) -> String {
    match &config.kind {
        ConfigKind::Run(run) => run.command.clone(),
        ConfigKind::Serve(serve) => format!("serve {} → :{}", serve.root, serve.port),
    }
}

/// The configurations of `files` below the checkout root whose name, folder and command contain every word of `filter`.
pub fn entries(files: &[(String, Arc<RunFile>)], filter: &str) -> Vec<ConfigEntry> {
    let words: Vec<String> = filter.split_whitespace().map(str::to_lowercase).collect();
    files
        .iter()
        .filter(|(folder, _)| !folder.is_empty())
        .flat_map(|(folder, file)| file.configs.iter().map(move |config| (folder, config)))
        .map(|(folder, config)| ConfigEntry { folder: folder.clone(), config: config.clone(), hint: config_hint(config) })
        .filter(|entry| matches_words(entry, &words))
        .collect()
}

fn matches_words(entry: &ConfigEntry, words: &[String]) -> bool {
    // not the hint: its `run.<key>` table name would make the word "run" match every configuration
    let haystack = format!("{}\n{}\n{}", entry.config.name, entry.folder, command_text(&entry.config)).to_lowercase();
    words.iter().all(|word| haystack.contains(word))
}

/// The checkout-relative folders with a `.run.toml`, the root left out.
pub fn run_folders(files: &[(String, Arc<RunFile>)]) -> HashSet<String> {
    files.iter().map(|(folder, _)| folder.clone()).filter(|folder| !folder.is_empty()).collect()
}

/// `run_files` are the folders with a `.run.toml` whatever the filter; `runs(folder, key)` is the run of a
/// configuration, if it has one.
pub fn rows(
    entries: &[ConfigEntry],
    run_files: &HashSet<String>,
    layout: FileLayout,
    folded: &HashSet<String>,
    runs: &dyn Fn(&str, &str) -> Option<RunRow>,
) -> Vec<ListRow> {
    let run_of = |index: usize| runs(&entries[index].folder, &entries[index].config.key);
    match layout {
        FileLayout::Flat => flat_rows(entries, &run_of),
        FileLayout::Tree => {
            let mut root = Node::default();
            for (index, entry) in entries.iter().enumerate() {
                root.insert(&entry.folder, index);
            }
            let context = FolderContext { run_files, folded, run_of: &run_of };
            let mut out = Vec::new();
            for (name, child) in root.sorted_children() {
                push_folder(&mut out, &context, name.to_owned(), child, 0);
            }
            out
        }
    }
}

fn flat_rows(entries: &[ConfigEntry], run_of: &dyn Fn(usize) -> Option<RunRow>) -> Vec<ListRow> {
    let mut order: Vec<usize> = (0..entries.len()).collect();
    // stable, so equal names keep file order
    order.sort_by_cached_key(|index| entries[*index].config.name.to_lowercase());
    order.into_iter().map(|entry| ListRow { depth: 0, item: ListItem::Config { entry, run: run_of(entry) } }).collect()
}

#[derive(Default)]
struct Node {
    path: String,
    own: Vec<usize>,
    children: Vec<(String, Node)>,
}

impl Node {
    fn insert(&mut self, folder: &str, index: usize) {
        let mut node = self;
        let mut path = String::new();
        for name in folder.split('/').filter(|name| !name.is_empty()) {
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(name);
            let at = match node.children.iter().position(|(existing, _)| existing == name) {
                Some(at) => at,
                None => {
                    node.children.push((name.to_owned(), Node { path: path.clone(), ..Node::default() }));
                    node.children.len() - 1
                }
            };
            node = &mut node.children[at].1;
        }
        node.own.push(index);
    }

    fn sorted_children(&self) -> Vec<(&str, &Node)> {
        let mut children: Vec<(&str, &Node)> = self.children.iter().map(|(name, node)| (name.as_str(), node)).collect();
        children.sort_by_cached_key(|(name, _)| name.to_lowercase());
        children
    }

    /// How many configurations in this folder and below `live` says are running.
    fn live(&self, live: &dyn Fn(usize) -> bool) -> usize {
        let own = self.own.iter().filter(|index| live(**index)).count();
        own + self.children.iter().map(|(_, child)| child.live(live)).sum::<usize>()
    }
}

struct FolderContext<'a> {
    run_files: &'a HashSet<String>,
    folded: &'a HashSet<String>,
    run_of: &'a dyn Fn(usize) -> Option<RunRow>,
}

fn push_folder(out: &mut Vec<ListRow>, context: &FolderContext, label: String, node: &Node, depth: usize) {
    let FolderContext { run_files, folded, run_of } = context;
    // a folder with nothing of its own and one subfolder reads as one row, unless it has its own run file to open
    if let [(name, only)] = node.children.as_slice()
        && node.own.is_empty()
        && !run_files.contains(&node.path)
    {
        push_folder(out, context, format!("{label}/{name}"), only, depth);
        return;
    }
    let live = node.live(&|index| run_of(index).is_some_and(|run| run.live));
    let open = !folded.contains(&node.path);
    out.push(ListRow { depth, item: ListItem::Folder { path: node.path.clone(), label, run_file: run_files.contains(&node.path), live, open } });
    if !open {
        return;
    }
    for (name, child) in node.sorted_children() {
        push_folder(out, context, name.to_owned(), child, depth + 1);
    }
    for index in &node.own {
        out.push(ListRow { depth: depth + 1, item: ListItem::Config { entry: *index, run: run_of(*index) } });
    }
}

/// `3 configurations in 2 files · 1 running`; the running part is left out at none.
pub fn summary(files: &[(String, Arc<RunFile>)], running: usize) -> String {
    // like `entries`: the root file is not listed, so it is not counted
    let listed: Vec<_> = files.iter().filter(|(folder, _)| !folder.is_empty()).collect();
    let configs: usize = listed.iter().map(|(_, file)| file.configs.len()).sum();
    let mut text = format!("{configs} {} in {} {}", plural(configs, "configuration"), listed.len(), plural(listed.len(), "file"));
    if running > 0 {
        text.push_str(&format!(" · {running} running"));
    }
    text
}

/// Whether the checkout has a `.run.toml` the list shows: the root file is not listed.
pub fn lists_any(files: &[(String, Arc<RunFile>)]) -> bool {
    files.iter().any(|(folder, _)| !folder.is_empty())
}

/// The live runs of `runs` (by checkout-relative folder) that the list shows: the root's are left out like its file.
pub fn running(runs: &HashMap<String, Vec<RunRow>>) -> usize {
    runs.iter().filter(|(folder, _)| !folder.is_empty()).flat_map(|(_, runs)| runs).filter(|run| run.live).count()
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 { word.to_owned() } else { format!("{word}s") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::RunId;
    use crate::sidebar::rows::RunBadge;

    fn file(configs: &[(&str, &str, &str)]) -> Arc<RunFile> {
        let text: String = configs.iter().map(|(key, name, command)| format!("[run.{key}]\nname = \"{name}\"\ncommand = \"{command}\"\n\n")).collect();
        Arc::new(crate::run::file::parse(&text).expect("valid run file"))
    }

    type Cfg = (&'static str, &'static str, &'static str);

    fn files(folders: &[(&str, &[Cfg])]) -> Vec<(String, Arc<RunFile>)> {
        folders.iter().map(|(folder, configs)| ((*folder).to_owned(), file(configs))).collect()
    }

    fn run(key: &str, live: bool) -> RunRow {
        let state = if live { RunBadge::Running } else { RunBadge::Stopped };
        RunRow { id: RunId::Pending(1), key: key.to_owned(), name: key.to_owned(), serve: false, state, status: String::new(), label: String::new(), live, link: None }
    }

    fn none() -> HashSet<String> {
        HashSet::new()
    }

    fn no_runs(_: &str, _: &str) -> Option<RunRow> {
        None
    }

    /// `label/@depth` for folders, `name@depth` for configurations.
    fn shown(entries: &[ConfigEntry], rows: &[ListRow]) -> Vec<String> {
        rows.iter()
            .map(|row| match &row.item {
                ListItem::Folder { label, .. } => format!("{label}/@{}", row.depth),
                ListItem::Config { entry, .. } => format!("{}@{}", entries[*entry].config.name, row.depth),
            })
            .collect()
    }

    fn sample() -> Vec<(String, Arc<RunFile>)> {
        files(&[
            ("internal-apps/misc", &[("build", "Build misc", "mvn package"), ("app", "app", "mvn run")]),
            ("internal-apps/ive", &[("mock", "IVE mock", "cargo run")]),
            ("run", &[("x", "Run x", "make")]),
        ])
    }

    #[test]
    fn should_sort_flat_rows_by_name() {
        // given
        let all = files(&[("misc", &[("build", "Build misc", "mvn"), ("app", "app", "mvn")]), ("ive", &[("mock", "IVE mock", "cargo")])]);
        let found = entries(&all, "");

        // when
        let listed = rows(&found, &none(), FileLayout::Flat, &HashSet::new(), &no_runs);

        // then
        assert_eq!(shown(&found, &listed), ["app@0", "Build misc@0", "IVE mock@0"]);
    }

    #[test]
    fn should_nest_tree_rows_and_collapse_single_folder_chains() {
        // given
        let found = entries(&sample(), "");
        let only_misc = entries(&files(&[("internal-apps/misc", &[("app", "app", "mvn")])]), "");

        // when
        let listed = rows(&found, &none(), FileLayout::Tree, &HashSet::new(), &no_runs);
        let merged = rows(&only_misc, &none(), FileLayout::Tree, &HashSet::new(), &no_runs);

        // then
        assert_eq!(
            shown(&found, &listed),
            ["internal-apps/@0", "ive/@1", "IVE mock@2", "misc/@1", "Build misc@2", "app@2", "run/@0", "Run x@1"]
        );
        assert_eq!(shown(&only_misc, &merged), ["internal-apps/misc/@0", "app@1"]);
        assert!(matches!(&merged[0].item, ListItem::Folder { path, .. } if path == "internal-apps/misc"));
    }

    #[test]
    fn should_list_subfolders_before_configs_in_file_order() {
        // given
        let all = files(&[("a", &[("z", "Zed", "z"), ("b", "Bee", "b")]), ("a/sub", &[("s", "Sub", "s")])]);
        let found = entries(&all, "");

        // when
        let listed = rows(&found, &none(), FileLayout::Tree, &HashSet::new(), &no_runs);

        // then
        assert_eq!(shown(&found, &listed), ["a/@0", "sub/@1", "Sub@2", "Zed@1", "Bee@1"]);
    }

    #[test]
    fn should_hide_a_folded_folders_rows() {
        // given
        let found = entries(&sample(), "");
        let folded = HashSet::from(["internal-apps".to_owned(), "run".to_owned()]);

        // when
        let listed = rows(&found, &none(), FileLayout::Tree, &folded, &no_runs);

        // then
        assert_eq!(listed.len(), 2);
        assert!(matches!(&listed[0].item, ListItem::Folder { open: false, .. }));
    }

    #[test]
    fn should_count_live_runs_below_a_folder() {
        // given
        let found = entries(&sample(), "");
        let run_files = run_folders(&sample());
        let runs = |folder: &str, key: &str| match (folder, key) {
            ("internal-apps/ive", "mock") | ("internal-apps/misc", "app") => Some(run(key, true)),
            ("internal-apps/misc", "build") => Some(run(key, false)),
            _ => None,
        };

        // when
        let listed = rows(&found, &run_files, FileLayout::Tree, &HashSet::new(), &runs);

        // then
        assert!(matches!(&listed[0].item, ListItem::Folder { live: 2, run_file: false, .. }));
        assert!(matches!(&listed[3].item, ListItem::Folder { live: 1, run_file: true, .. }));
    }

    #[test]
    fn should_filter_by_every_word() {
        // given
        let all = files(&[("misc", &[("build", "Build misc", "MVN package"), ("app", "app", "java")]), ("ive", &[("mock", "IVE mock", "cargo")])]);

        // when
        let both = entries(&all, "misc build");
        let by_command = entries(&all, "mvn");
        let by_folder = entries(&all, "IVE");

        // then
        assert_eq!(both.iter().map(|e| e.config.name.as_str()).collect::<Vec<_>>(), ["Build misc"]);
        assert_eq!(by_command.len(), 1);
        assert_eq!(by_folder[0].config.name, "IVE mock");
    }

    #[test]
    fn should_not_match_the_table_name() {
        // given
        let all = files(&[("misc", &[("build", "Build", "mvn package")])]);

        // when
        let by_table = entries(&all, "run");
        let by_command = entries(&all, "package");

        // then
        assert!(by_table.is_empty());
        assert_eq!(by_command.len(), 1);
    }

    #[test]
    fn should_keep_run_file_on_a_folder_whose_configs_are_filtered_out() {
        // given
        let all = files(&[("internal-apps", &[("a", "Alpha", "x")]), ("internal-apps/ive", &[("m", "Mock", "cargo")])]);
        let found = entries(&all, "mock");

        // when
        let listed = rows(&found, &run_folders(&all), FileLayout::Tree, &HashSet::new(), &no_runs);

        // then
        assert_eq!(shown(&found, &listed), ["internal-apps/@0", "ive/@1", "Mock@2"]);
        assert!(matches!(&listed[0].item, ListItem::Folder { run_file: true, .. }));
    }

    #[test]
    fn should_leave_out_the_root_run_file() {
        // given
        let all = files(&[("", &[("a", "Root one", "x")]), ("misc", &[("b", "Misc one", "y")])]);

        // when
        let found = entries(&all, "");

        // then
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].folder, "misc");
    }

    #[test]
    fn should_show_run_only_before_any_run() {
        // given
        let found = entries(&files(&[("misc", &[("app", "app", "mvn")])]), "");
        let started = |_: &str, key: &str| Some(run(key, true));

        // when
        let before = rows(&found, &none(), FileLayout::Flat, &HashSet::new(), &no_runs);
        let after = rows(&found, &none(), FileLayout::Flat, &HashSet::new(), &started);

        // then
        assert!(matches!(&before[0].item, ListItem::Config { run: None, .. }));
        assert!(matches!(&after[0].item, ListItem::Config { run: Some(_), .. }));
    }

    #[test]
    fn should_summarise_files_and_running() {
        // given
        let three = files(&[("", &[("r", "R", "r")]), ("a", &[("x", "X", "x"), ("y", "Y", "y")]), ("b", &[("z", "Z", "z")])]);
        let one = files(&[("a", &[("x", "X", "x")])]);

        // when
        let many = summary(&three, 1);
        let single = summary(&one, 0);

        // then
        assert_eq!(many, "3 configurations in 2 files · 1 running");
        assert_eq!(single, "1 configuration in 1 file");
    }

    #[test]
    fn should_list_nothing_with_only_a_root_run_file() {
        // given
        let root = files(&[("", &[("r", "R", "r")])]);
        let nested = files(&[("", &[("r", "R", "r")]), ("a", &[("x", "X", "x")])]);

        // when
        let listed = [lists_any(&[]), lists_any(&root), lists_any(&nested)];

        // then
        assert_eq!(listed, [false, false, true]);
    }

    #[test]
    fn should_count_only_live_runs_of_listed_folders() {
        // given
        let runs = HashMap::from([
            (String::new(), vec![run("r", true)]),
            ("a".to_owned(), vec![run("x", true), run("y", false)]),
            ("b".to_owned(), vec![run("z", true)]),
        ]);

        // when
        let count = running(&runs);

        // then
        assert_eq!(count, 2);
    }
}
