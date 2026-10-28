//! `config.excluded`: folder names whose content counts as excluded, besides gitignored folders: the tree draws
//! them as excluded.

use gpui::Global;
use serde_json::Value;

const MAX_NAMES: usize = 64;
/// The longest file name Linux and macOS allow.
const MAX_NAME_BYTES: usize = 255;

/// Folder names, matched against every folder of a path by its whole name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Excluded(Vec<String>);

impl Global for Excluded {}

impl Excluded {
    pub fn names(&self) -> &[String] {
        &self.0
    }
}

/// `value` as `config.excluded` left it; the error names the offending value.
pub fn parse(value: &rune::Value) -> Result<Excluded, String> {
    parse_names(value, "excluded", "folder names").map(Excluded)
}

/// `value` as `config.<key>` left it: a list of distinct `what`, each an entry's own name; the error names `key`
/// and the offending value.
pub(super) fn parse_names(value: &rune::Value, key: &str, what: &str) -> Result<Vec<String>, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("{key}: {error}"))?;
    let Value::Array(items) = json else {
        return Err(format!("{key} must be a list of {what} such as [\"target\"], not {json}"));
    };
    if items.len() > MAX_NAMES {
        return Err(format!("{key}: {} names, at most {MAX_NAMES}", items.len()));
    }
    let mut names = Vec::new();
    for item in items {
        let name = entry_name(&item, key)?;
        if names.contains(&name) {
            return Err(format!("{key}: {item} is listed twice"));
        }
        names.push(name);
    }
    Ok(names)
}

/// An entry's own name: not empty, without a path separator, not `.` or `..`.
fn entry_name(item: &Value, key: &str) -> Result<String, String> {
    let name = item.as_str().ok_or_else(|| format!("{key}: {item} is not a name in quotes"))?;
    let fits = !name.is_empty() && name.len() <= MAX_NAME_BYTES;
    if !fits || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(format!("{key}: {item} is not a name: 1 to {MAX_NAME_BYTES} bytes, no / or \\, not . or .."));
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_parse_folder_names() {
        // given
        let list = value(serde_json::json!(["target", "node_modules"]));

        // when
        let excluded = parse(&list).unwrap();

        // then
        assert_eq!(excluded.names(), ["target", "node_modules"]);
    }

    #[test]
    fn should_accept_an_empty_list() {
        // given
        let list = value(serde_json::json!([]));

        // when
        let excluded = parse(&list).unwrap();

        // then
        assert!(excluded.names().is_empty());
    }

    #[test]
    fn should_reject_a_path() {
        // given
        let list = value(serde_json::json!(["target/debug"]));

        // when
        let error = parse(&list).unwrap_err();

        // then
        assert_eq!(error, r#"excluded: "target/debug" is not a name: 1 to 255 bytes, no / or \, not . or .."#);
    }

    #[test]
    fn should_reject_dot_dot_and_empty_names() {
        // given
        let lists = [serde_json::json!([".."]), serde_json::json!(["."]), serde_json::json!([""])];

        // when
        let errors: Vec<String> = lists.into_iter().map(|list| parse(&value(list)).unwrap_err()).collect();

        // then
        assert!(errors.iter().all(|error| error.contains("is not a name")), "{errors:?}");
    }

    #[test]
    fn should_reject_a_name_listed_twice() {
        // given
        let list = value(serde_json::json!(["dist", "dist"]));

        // when
        let error = parse(&list).unwrap_err();

        // then
        assert_eq!(error, r#"excluded: "dist" is listed twice"#);
    }

    #[test]
    fn should_reject_a_non_list() {
        // given
        let text = value(serde_json::json!("target"));

        // when
        let error = parse(&text).unwrap_err();

        // then
        assert_eq!(error, r#"excluded must be a list of folder names such as ["target"], not "target""#);
    }

    #[test]
    fn should_reject_too_many_names() {
        // given
        let names: Vec<String> = (0..=MAX_NAMES).map(|n| format!("dir{n}")).collect();
        let list = value(serde_json::json!(names));

        // when
        let error = parse(&list).unwrap_err();

        // then
        assert_eq!(error, "excluded: 65 names, at most 64");
    }
}
