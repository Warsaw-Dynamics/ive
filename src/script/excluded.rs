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
    let json = serde_json::to_value(value).map_err(|error| format!("excluded: {error}"))?;
    let Value::Array(items) = json else {
        return Err(format!("excluded must be a list of folder names such as [\"target\"], not {json}"));
    };
    if items.len() > MAX_NAMES {
        return Err(format!("excluded: {} names, at most {MAX_NAMES}", items.len()));
    }
    let mut names = Vec::new();
    for item in items {
        let name = folder_name(&item)?;
        if names.contains(&name) {
            return Err(format!("excluded: {item} is listed twice"));
        }
        names.push(name);
    }
    Ok(Excluded(names))
}

/// A folder's own name: not empty, without a path separator, not `.` or `..`.
fn folder_name(item: &Value) -> Result<String, String> {
    let name = item.as_str().ok_or_else(|| format!("excluded: {item} is not a folder name in quotes"))?;
    let fits = !name.is_empty() && name.len() <= MAX_NAME_BYTES;
    if !fits || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(format!("excluded: {item} is not a folder name: 1 to {MAX_NAME_BYTES} bytes, no / or \\, not . or .."));
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
        assert_eq!(error, r#"excluded: "target/debug" is not a folder name: 1 to 255 bytes, no / or \, not . or .."#);
    }

    #[test]
    fn should_reject_dot_dot_and_empty_names() {
        // given
        let lists = [serde_json::json!([".."]), serde_json::json!(["."]), serde_json::json!([""])];

        // when
        let errors: Vec<String> = lists.into_iter().map(|list| parse(&value(list)).unwrap_err()).collect();

        // then
        assert!(errors.iter().all(|error| error.contains("is not a folder name")), "{errors:?}");
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
