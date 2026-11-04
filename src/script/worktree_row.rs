//! `config.worktree_row`: a function that says how a worktree's tree row reads, built from the worktree's data.
//!
//! ```rune
//! config.worktree_row = |row| [row.folder, #{ text: row.branch, style: "grey" }];
//! ```

use rune::runtime::Function;
use serde_json::{Map, Value};


/// Parts a row may have; more is a script gone wrong.
const MAX_PARTS: usize = 8;
/// Characters a part's text or tooltip may have.
const MAX_TEXT: usize = 200;

/// That load `generation` has a `config.worktree_row` function; the script thread keeps the function itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowFunction {
    pub generation: u64,
}

/// What the function is called with: a checkout as the tree knows it. `branch`, `head` and `base` are `()` when
/// HEAD is detached, before the first commit, or without a base.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct RowInput {
    /// the project's name and its main checkout's folder
    pub project: String,
    pub project_dir: String,
    /// the checkout's folder and the folder's name
    pub path: String,
    pub folder: String,
    /// how IVE names the row by itself: the branch for the main checkout, the folder for a worktree
    pub label: String,
    pub branch: Option<String>,
    pub main: bool,
    /// HEAD's short sha
    pub head: Option<String>,
    pub base: Option<String>,
}

/// How a checkout's row reads after its icon: one part or more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowLook {
    pub parts: Vec<RowPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowPart {
    pub text: String,
    pub style: PartStyle,
    pub icon: Option<PartIcon>,
    pub tip: Option<String>,
}

/// `style`: `"name"` (the row's own text), `"grey"` (smaller and grey, as a project's tags) or `"pill"` (on a chip).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartStyle {
    Name,
    Grey,
    Pill,
}

/// `icon`, drawn before the part's text: `"branch"`, `"folder"` or `"project"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartIcon {
    Branch,
    Folder,
    Project,
}

const STYLES: [(&str, PartStyle); 3] = [("name", PartStyle::Name), ("grey", PartStyle::Grey), ("pill", PartStyle::Pill)];
const ICONS: [(&str, PartIcon); 3] = [("branch", PartIcon::Branch), ("folder", PartIcon::Folder), ("project", PartIcon::Project)];

impl RowPart {
    fn named(text: String) -> Self {
        Self { text, style: PartStyle::Name, icon: None, tip: None }
    }
}

impl RowLook {
    /// The row's text as one line, its parts joined by spaces: what the tree's width and tests go by.
    pub fn text(&self) -> String {
        self.parts.iter().map(|part| part.text.as_str()).collect::<Vec<_>>().join(" ")
    }
}

/// IVE's own look, also `ive.rn`'s: the label, and a worktree's branch after it in grey when its folder is named
/// otherwise.
pub fn default_look(input: &RowInput) -> RowLook {
    let other = input.branch.as_ref().filter(|branch| !input.main && **branch != input.label);
    let Some(branch) = other else {
        return RowLook { parts: vec![RowPart::named(input.label.clone())] };
    };
    let name = RowPart { tip: Some(format!("{} on branch {branch}", input.label)), ..RowPart::named(input.label.clone()) };
    RowLook { parts: vec![name, RowPart { style: PartStyle::Grey, ..RowPart::named(branch.clone()) }] }
}

/// `config.worktree_row` as a script left it: `()` for IVE's own look, else a function.
pub fn bind(value: &rune::Value) -> Result<Option<Function>, String> {
    if rune::from_value::<()>(value.clone()).is_ok() {
        return Ok(None);
    }
    let function = rune::from_value::<Function>(value.clone()).map_err(|_| "worktree_row must be a function, like |row| [row.label]".to_owned())?;
    Ok(Some(function))
}

/// What the function returned: a text, or a list of parts, each a text or `#{ text, style, icon, tip }`.
pub fn parse(returned: &Value) -> Result<RowLook, String> {
    let parts = match returned {
        Value::String(text) => vec![RowPart::named(checked_text(text, "text")?)],
        Value::Array(items) if (1..=MAX_PARTS).contains(&items.len()) => items.iter().map(parse_part).collect::<Result<_, _>>()?,
        Value::Array(items) => return Err(format!("worktree_row must return 1 to {MAX_PARTS} parts, not {}", items.len())),
        other => return Err(format!("worktree_row must return a text or a list of parts, not {other}")),
    };
    Ok(RowLook { parts })
}

fn parse_part(item: &Value) -> Result<RowPart, String> {
    match item {
        Value::String(text) => Ok(RowPart::named(checked_text(text, "text")?)),
        Value::Object(fields) => parse_fields(fields),
        other => Err(format!("a worktree_row part must be a text or #{{ text, style, icon, tip }}, not {other}")),
    }
}

fn parse_fields(fields: &Map<String, Value>) -> Result<RowPart, String> {
    if let Some(unknown) = fields.keys().find(|key| !["text", "style", "icon", "tip"].contains(&key.as_str())) {
        return Err(format!("a worktree_row part has text, style, icon and tip, not {unknown:?}"));
    }
    let text = match fields.get("text") {
        Some(Value::String(text)) => checked_text(text, "text")?,
        other => return Err(format!("a worktree_row part's text must be a text, not {}", other.unwrap_or(&Value::Null))),
    };
    let style = match fields.get("style") {
        None => PartStyle::Name,
        Some(style) => named(style, &STYLES, "style")?,
    };
    let icon = fields.get("icon").map(|icon| named(icon, &ICONS, "icon")).transpose()?;
    let tip = fields.get("tip").map(|tip| tip.as_str().ok_or_else(|| format!("a worktree_row tip must be a text, not {tip}")).and_then(|tip| checked_text(tip, "tip"))).transpose()?;
    Ok(RowPart { text, style, icon, tip })
}

/// The value `names` gives `value`; the error lists them.
fn named<T: Copy>(value: &Value, names: &[(&str, T)], field: &str) -> Result<T, String> {
    let found = names.iter().find(|(name, _)| value.as_str() == Some(name));
    let listed = names.iter().map(|(name, _)| format!("{name:?}")).collect::<Vec<_>>().join(", ");
    found.map(|(_, named)| *named).ok_or_else(|| format!("a worktree_row {field} must be one of {listed}, not {value}"))
}

fn checked_text(text: &str, field: &str) -> Result<String, String> {
    let length = text.chars().count();
    if length > MAX_TEXT {
        return Err(format!("a worktree_row {field} has at most {MAX_TEXT} characters, not {length}: {:?}…", text.chars().take(20).collect::<String>()));
    }
    Ok(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(label: &str, branch: Option<&str>, main: bool) -> RowInput {
        RowInput {
            project: "p".to_owned(),
            project_dir: "/p".to_owned(),
            path: format!("/p/{label}"),
            folder: label.to_owned(),
            label: label.to_owned(),
            branch: branch.map(str::to_owned),
            main,
            head: None,
            base: None,
        }
    }

    #[test]
    fn should_add_the_branch_in_grey_only_to_a_worktree_named_otherwise() {
        // given
        let inputs = [input("dev", Some("dev"), true), input("same", Some("same"), false), input("wt", Some("GEN-1"), false), input("detached", None, false)];

        // when
        let looks = inputs.each_ref().map(|input| default_look(input).text());

        // then
        assert_eq!(looks, ["dev", "same", "wt GEN-1", "detached"]);
        assert_eq!(default_look(&inputs[2]).parts[1].style, PartStyle::Grey);
    }

    #[test]
    fn should_read_a_text_and_parts() {
        // when
        let looks = [json!("wt"), json!(["wt", { "text": "GEN-1", "style": "pill", "icon": "branch", "tip": "on GEN-1" }])].map(|value| parse(&value));

        // then
        let pill = RowPart { text: "GEN-1".to_owned(), style: PartStyle::Pill, icon: Some(PartIcon::Branch), tip: Some("on GEN-1".to_owned()) };
        assert_eq!(looks, [Ok(RowLook { parts: vec![RowPart::named("wt".to_owned())] }), Ok(RowLook { parts: vec![RowPart::named("wt".to_owned()), pill] })]);
    }

    #[test]
    fn should_refuse_what_is_not_a_row() {
        // when
        let errors = [json!([]), json!(3), json!([{ "text": "a", "colour": "red" }]), json!([{ "text": "a", "style": "bold" }]), json!([{ "style": "grey" }])]
            .map(|value| parse(&value).unwrap_err());

        // then
        assert_eq!(errors, [
            "worktree_row must return 1 to 8 parts, not 0".to_owned(),
            "worktree_row must return a text or a list of parts, not 3".to_owned(),
            "a worktree_row part has text, style, icon and tip, not \"colour\"".to_owned(),
            "a worktree_row style must be one of \"name\", \"grey\", \"pill\", not \"bold\"".to_owned(),
            "a worktree_row part's text must be a text, not null".to_owned(),
        ]);
    }

    #[test]
    fn should_refuse_a_text_over_200_characters() {
        // when
        let refused = parse(&json!("x".repeat(201)));

        // then
        assert_eq!(refused, Err(format!("a worktree_row text has at most 200 characters, not 201: {:?}…", "x".repeat(20))));
    }
}
