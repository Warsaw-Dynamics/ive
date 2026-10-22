//! `config.tab_menus` and `config.tree_menus`: named menus that `act::ShowTabMenu(name)` opens on a tab and
//! `act::ShowTreeMenu(name)` on a tree row. Read from Rune values, not JSON, so items can name functions.

use std::collections::HashMap;

use gpui::Global;
use rune::Value;
use rune::runtime::Function;

use super::declare::{self, Functions};
use super::keymap::{Action, Builtin};

const SEPARATOR: &str = "Separator";
const MAX_MENUS: usize = 32;
const MAX_ITEMS: usize = 32;
const MAX_NAME_CHARS: usize = 32;
const MAX_LABEL_CHARS: usize = 64;
/// Submenus in submenus, at most.
const MAX_DEPTH: usize = 4;
const ITEM_FIELDS: [&str; 4] = ["label", "action", "items", "when"];
const WHEN_FIELDS: [&str; 5] = ["kind", "name", "path", "count", "test"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Tab,
    Tree,
}

impl MenuKind {
    fn field(self) -> &'static str {
        match self {
            Self::Tab => "tab_menus",
            Self::Tree => "tree_menus",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAction {
    Builtin(Builtin),
    /// function `function` of script load `generation`, run on the script thread
    Invoke { generation: u64, function: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Label {
    /// the built-in action's own
    Default,
    Custom(String),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KindIs {
    #[default]
    Any,
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CountIs {
    #[default]
    Any,
    /// exactly one path
    One,
    /// two or more
    Many,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Test {
    #[default]
    Absent,
    /// a predicate: function `function` of script load `generation`
    Function { generation: u64, function: usize },
}

/// When an item shows: every field given must match; empty `name` and `path` match anything.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct When {
    pub kind: KindIs,
    /// globs against the file or folder name, any one
    pub name: Vec<String>,
    /// globs against the path relative to its checkout, any one
    pub path: Vec<String>,
    pub count: CountIs,
    pub test: Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Action { label: Label, action: ItemAction, when: When },
    Submenu { label: String, items: Vec<MenuItem>, when: When },
    Separator,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Menus {
    tab: Vec<(String, Vec<MenuItem>)>,
    tree: Vec<(String, Vec<MenuItem>)>,
}

impl Global for Menus {}

impl Menus {
    pub fn new(tab: Vec<(String, Vec<MenuItem>)>, tree: Vec<(String, Vec<MenuItem>)>) -> Self {
        Self { tab, tree }
    }

    pub fn get(&self, kind: MenuKind, name: &str) -> Option<&[MenuItem]> {
        let menus = match kind {
            MenuKind::Tab => &self.tab,
            MenuKind::Tree => &self.tree,
        };
        menus.iter().find(|(known, _)| known == name).map(|(_, items)| items.as_slice())
    }

    /// What names function `function` in messages: the label of the item it runs, or the condition it decides.
    pub fn function_label(&self, function: usize) -> Option<String> {
        let mut all = self.tab.iter().chain(&self.tree).flat_map(|(_, items)| items);
        all.find_map(|item| label_of(item, function))
    }
}

fn label_of(item: &MenuItem, function: usize) -> Option<String> {
    let (label, when) = match item {
        MenuItem::Separator => return None,
        MenuItem::Submenu { label, items, when } => match items.iter().find_map(|item| label_of(item, function)) {
            Some(found) => return Some(found),
            None => (label.clone(), when),
        },
        MenuItem::Action { label, action, when } => {
            if matches!(action, ItemAction::Invoke { function: bound, .. } if *bound == function) {
                return Some(described(label, action));
            }
            (described(label, action), when)
        }
    };
    matches!(when.test, Test::Function { function: bound, .. } if bound == function).then(|| format!("the condition of {label}"))
}

fn described(label: &Label, action: &ItemAction) -> String {
    match (label, action) {
        (Label::Custom(label), _) => label.clone(),
        (Label::Default, ItemAction::Builtin(builtin)) => format!("{builtin:?}"),
        (Label::Default, ItemAction::Invoke { .. }) => "a menu item".to_owned(),
    }
}

/// `value` as `config.<kind>_menus` left it, over `defaults` (what `ive.rn` set) per menu name; functions go to
/// `functions`. The error names the menu, the item's place and the offending value.
pub fn parse(kind: MenuKind, defaults: &Value, value: &Value, functions: &mut Functions) -> Result<Vec<(String, Vec<MenuItem>)>, String> {
    let field = kind.field();
    let mut merged = object(defaults, field)?;
    merged.extend(object(value, field)?);
    if merged.len() > MAX_MENUS {
        return Err(format!("{field}: {} menus declared, at most {MAX_MENUS}", merged.len()));
    }
    let mut names: Vec<String> = merged.keys().cloned().collect();
    names.sort();
    let mut menus = Vec::new();
    for name in names {
        let count = name.chars().count();
        if count == 0 || count > MAX_NAME_CHARS {
            return Err(format!("{field}: menu name {name:?} must be 1 to {MAX_NAME_CHARS} characters"));
        }
        let reader = Reader { kind, functions: &mut *functions };
        let items = reader.items(&merged[&name], &format!("{field}.{name}"), 0)?;
        menus.push((name, items));
    }
    Ok(menus)
}

/// Fails on the first menu of `names` that `menus` does not declare as a `kind` menu.
pub fn check_shown<'a>(menus: &Menus, kind: MenuKind, mut names: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let action = match kind {
        MenuKind::Tab => "ShowTabMenu",
        MenuKind::Tree => "ShowTreeMenu",
    };
    match names.find(|name| menus.get(kind, name).is_none()) {
        Some(name) => Err(format!("act::{action}({name:?}): config.{} declares no menu {name:?}", kind.field())),
        None => Ok(()),
    }
}

fn object(value: &Value, field: &str) -> Result<HashMap<String, Value>, String> {
    if rune::from_value::<()>(value.clone()).is_ok() {
        return Ok(HashMap::new());
    }
    fields(value).map_err(|_| format!("{field} must be an object such as #{{ main: [act::CloseTab] }}"))
}

/// Reads the items of one menu of `kind`, putting the functions they name in `functions`.
struct Reader<'a, 'b> {
    kind: MenuKind,
    functions: &'a mut Functions<'b>,
}

impl Reader<'_, '_> {
    fn items(mut self, value: &Value, place: &str, depth: usize) -> Result<Vec<MenuItem>, String> {
        self.list(value, place, depth)
    }

    fn list(&mut self, value: &Value, place: &str, depth: usize) -> Result<Vec<MenuItem>, String> {
        if depth > MAX_DEPTH {
            return Err(format!("{place}: submenus nested more than {MAX_DEPTH} deep"));
        }
        let items = values(value).map_err(|_| format!("{place} must be a list such as [act::CloseTab]"))?;
        if items.len() > MAX_ITEMS {
            return Err(format!("{place}: {} items, at most {MAX_ITEMS}", items.len()));
        }
        let read = items.iter().enumerate().map(|(index, item)| self.item(item, &format!("{place}[{index}]"), depth));
        read.collect()
    }

    fn item(&mut self, value: &Value, place: &str, depth: usize) -> Result<MenuItem, String> {
        if let Some(name) = declare::string(value) {
            return self.named(&name).map_err(|problem| format!("{place}: {problem}"));
        }
        let fields = fields(value).map_err(|_| format!("{place}: {} is not a menu item", declare_describe(value)))?;
        if let Some(unknown) = fields.keys().find(|field| !ITEM_FIELDS.contains(&field.as_str())) {
            return Err(format!("{place}: unknown field {unknown:?} (fields: {})", ITEM_FIELDS.join(", ")));
        }
        let when = match fields.get("when") {
            Some(when) => self.when(when).map_err(|problem| format!("{place}.when: {problem}"))?,
            None => When::default(),
        };
        match (fields.get("items"), fields.get("action")) {
            (Some(items), None) => {
                let label = label(fields.get("label")).map_err(|problem| format!("{place}: {problem}"))?.ok_or_else(|| format!("{place}: a submenu needs a label"))?;
                let items = self.list(items, &format!("{place}.items"), depth + 1)?;
                Ok(MenuItem::Submenu { label, items, when })
            }
            (None, Some(action)) => {
                let label = label(fields.get("label")).map_err(|problem| format!("{place}: {problem}"))?.map_or(Label::Default, Label::Custom);
                let action = self.action(action).map_err(|problem| format!("{place}: {problem}"))?;
                if matches!((&label, action), (Label::Default, ItemAction::Invoke { .. })) {
                    return Err(format!("{place}: an item running a function needs a label"));
                }
                Ok(MenuItem::Action { label, action, when })
            }
            _ => Err(format!("{place}: an item has either action or items")),
        }
    }

    /// `"Separator"` or a unit action's name.
    fn named(&self, name: &str) -> Result<MenuItem, String> {
        if name == SEPARATOR {
            return Ok(MenuItem::Separator);
        }
        let action = self.builtin(name)?;
        Ok(MenuItem::Action { label: Label::Default, action, when: When::default() })
    }

    fn builtin(&self, name: &str) -> Result<ItemAction, String> {
        match declare::unit_action(name) {
            Ok(Action::Builtin(builtin)) if allowed(self.kind, builtin) => Ok(ItemAction::Builtin(builtin)),
            _ => Err(format!("{name} is not an action a {} menu can hold", self.kind.field().trim_end_matches("_menus"))),
        }
    }

    fn action(&mut self, value: &Value) -> Result<ItemAction, String> {
        if let Some(function) = self.function(value) {
            return Ok(function);
        }
        match declare::string(value) {
            Some(name) => self.builtin(&name),
            None => Err("action must be an act:: action or a function".to_owned()),
        }
    }

    fn function(&mut self, value: &Value) -> Option<ItemAction> {
        let function = rune::from_value::<Function>(value.clone()).ok()?;
        self.functions.bound.push(function);
        Some(ItemAction::Invoke { generation: self.functions.generation, function: self.functions.bound.len() - 1 })
    }

    fn when(&mut self, value: &Value) -> Result<When, String> {
        if let Some(ItemAction::Invoke { generation, function }) = self.function(value) {
            return Ok(When { test: Test::Function { generation, function }, ..When::default() });
        }
        let fields = fields(value).map_err(|_| "must be an object such as #{ kind: \"file\" } or a function".to_owned())?;
        if let Some(unknown) = fields.keys().find(|field| !WHEN_FIELDS.contains(&field.as_str())) {
            return Err(format!("unknown field {unknown:?} (fields: {})", WHEN_FIELDS.join(", ")));
        }
        let test = match fields.get("test").map(|test| self.function(test)) {
            None => Test::Absent,
            Some(Some(ItemAction::Invoke { generation, function })) => Test::Function { generation, function },
            Some(_) => return Err("test must be a function".to_owned()),
        };
        Ok(When { kind: kind(fields.get("kind"))?, name: globs(fields.get("name"), "name")?, path: globs(fields.get("path"), "path")?, count: count(fields.get("count"))?, test })
    }
}

/// Which built-ins a menu of `kind` takes.
fn allowed(kind: MenuKind, builtin: Builtin) -> bool {
    builtin.menu_kinds().contains(&kind)
}

fn label(value: Option<&Value>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let label = declare::string(value).filter(|label| (1..=MAX_LABEL_CHARS).contains(&label.chars().count()));
    label.map(Some).ok_or_else(|| format!("label must be a string of 1 to {MAX_LABEL_CHARS} characters"))
}

fn kind(value: Option<&Value>) -> Result<KindIs, String> {
    match value.map(declare::string) {
        None => Ok(KindIs::Any),
        Some(Some(kind)) if kind == "file" => Ok(KindIs::File),
        Some(Some(kind)) if kind == "folder" => Ok(KindIs::Folder),
        Some(other) => Err(format!("kind must be \"file\" or \"folder\", not {other:?}")),
    }
}

fn count(value: Option<&Value>) -> Result<CountIs, String> {
    match value.map(declare::string) {
        None => Ok(CountIs::Any),
        Some(Some(count)) if count == "one" => Ok(CountIs::One),
        Some(Some(count)) if count == "many" => Ok(CountIs::Many),
        Some(other) => Err(format!("count must be \"one\" or \"many\", not {other:?}")),
    }
}

/// A glob or a list of globs.
fn globs(value: Option<&Value>, field: &str) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if let Some(glob) = declare::string(value) {
        crate::menu::when::check_globs(std::slice::from_ref(&glob)).map_err(|problem| format!("{field}: {problem}"))?;
        return Ok(vec![glob]);
    }
    let list = values(value).map_err(|_| format!("{field} must be a glob or a list of globs"))?;
    let globs = list.iter().map(|glob| declare::string(glob).ok_or_else(|| format!("{field} must be a glob or a list of globs"))).collect::<Result<Vec<_>, _>>()?;
    crate::menu::when::check_globs(&globs).map_err(|problem| format!("{field}: {problem}"))?;
    Ok(globs)
}

/// An object's fields, borrowed: a menu `ive.rn` declared is shared with `init.rn`'s `config`, so it cannot be taken.
fn fields(value: &Value) -> Result<HashMap<String, Value>, ()> {
    let object = value.borrow_ref::<rune::runtime::Object>().map_err(drop)?;
    Ok(object.iter().map(|(key, value)| (key.to_string(), value.clone())).collect())
}

/// A list's values, borrowed as [`fields`] borrows.
fn values(value: &Value) -> Result<Vec<Value>, ()> {
    let list = value.borrow_ref::<rune::runtime::Vec>().map_err(drop)?;
    Ok(list.iter().cloned().collect())
}

fn declare_describe(value: &Value) -> String {
    super::engine::describe(value.clone())
}
