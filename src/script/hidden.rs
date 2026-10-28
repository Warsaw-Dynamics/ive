//! `config.hidden`: file and folder names the tree shows under `@hidden`, besides those starting with `.`.

use gpui::Global;

use super::excluded::parse_names;

/// Entry names, matched against an entry's whole name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hidden(Vec<String>);

impl Global for Hidden {}

impl Hidden {
    /// Whether the tree shows an entry named `name` under `@hidden`: a dot file, or one of these names.
    pub fn hides(&self, name: &str) -> bool {
        name.starts_with('.') || self.0.iter().any(|hidden| hidden == name)
    }
}

/// `value` as `config.hidden` left it; the error names the offending value.
pub fn parse(value: &rune::Value) -> Result<Hidden, String> {
    parse_names(value, "hidden", "file or folder names").map(Hidden)
}

#[cfg(test)]
pub fn of(names: &[&str]) -> Hidden {
    Hidden(names.iter().map(|name| (*name).to_owned()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_hide_listed_names_and_dot_files() {
        // given
        let hidden = parse(&value(serde_json::json!(["target"]))).unwrap();

        // when
        let hides = ["target", ".git", "src", "target2"].map(|name| hidden.hides(name));

        // then
        assert_eq!(hides, [true, true, false, false]);
    }

    #[test]
    fn should_reject_a_path() {
        // given
        let list = value(serde_json::json!(["target/debug"]));

        // when
        let error = parse(&list).unwrap_err();

        // then
        assert_eq!(error, r#"hidden: "target/debug" is not a name: 1 to 255 bytes, no / or \, not . or .."#);
    }

    #[test]
    fn should_reject_a_non_list() {
        // given
        let text = value(serde_json::json!("target"));

        // when
        let error = parse(&text).unwrap_err();

        // then
        assert_eq!(error, r#"hidden must be a list of file or folder names such as ["target"], not "target""#);
    }
}
