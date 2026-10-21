use gpui::{FontStyle, FontWeight, HighlightStyle, rgb};
use serde::{Deserialize, Serialize};

/// How text of a key looks; `None` colours mean the plain text colour and no background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextAttributes {
    pub foreground: Option<u32>,
    pub background: Option<u32>,
    pub bold: bool,
    pub italic: bool,
}

impl TextAttributes {
    pub fn highlight_style(self) -> HighlightStyle {
        HighlightStyle {
            color: self.foreground.map(|color| rgb(color).into()),
            background_color: self.background.map(|color| rgb(color).into()),
            font_weight: self.bold.then_some(FontWeight::BOLD),
            font_style: self.italic.then_some(FontStyle::Italic),
            ..HighlightStyle::default()
        }
    }
}

/// A scheme's or the user's setting for one key: inherit the parent's look, or own attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Inherit,
    Own(TextAttributes),
}

/// An entry as written in a scheme file or `settings.toml`, before validation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawEntry {
    #[serde(skip_serializing_if = "is_false")]
    pub inherit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub italic: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl RawEntry {
    /// `path` names the entry in error messages, e.g. `colors.dark."JSON.KEYWORD"`.
    pub fn to_entry(&self, path: &str) -> Result<Entry, String> {
        let owns = self.foreground.is_some() || self.background.is_some() || self.bold || self.italic;
        if self.inherit && owns {
            return Err(format!("{path} sets inherit = true and its own attributes; keep one"));
        }
        if self.inherit {
            return Ok(Entry::Inherit);
        }
        Ok(Entry::Own(TextAttributes {
            foreground: hex_field(path, "foreground", self.foreground.as_deref())?,
            background: hex_field(path, "background", self.background.as_deref())?,
            bold: self.bold,
            italic: self.italic,
        }))
    }
}

impl From<Entry> for RawEntry {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Inherit => Self { inherit: true, ..Self::default() },
            Entry::Own(attributes) => Self {
                inherit: false,
                foreground: attributes.foreground.map(|color| format!("{color:06X}")),
                background: attributes.background.map(|color| format!("{color:06X}")),
                bold: attributes.bold,
                italic: attributes.italic,
            },
        }
    }
}

fn hex_field(path: &str, field: &str, value: Option<&str>) -> Result<Option<u32>, String> {
    value
        .map(|text| parse_hex(text).ok_or_else(|| format!("{path}.{field} is \"{text}\", expected 6 hex digits")))
        .transpose()
}

/// `CF8E6D` → `0xCF8E6D`: exactly 6 hex digits, either case.
fn parse_hex(text: &str) -> Option<u32> {
    let valid = text.len() == 6 && text.bytes().all(|byte| byte.is_ascii_hexdigit());
    valid.then(|| u32::from_str_radix(text, 16).ok()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_six_hex_digits_only() {
        // given
        let texts = ["CF8E6D", "cf8e6d", "CF8E6", "#CF8E6D", "CF8E6G", "+F8E6D"];

        // when
        let parsed: Vec<_> = texts.iter().map(|text| parse_hex(text)).collect();

        // then
        assert_eq!(parsed, [Some(0xCF8E6D), Some(0xCF8E6D), None, None, None, None]);
    }

    #[test]
    fn should_name_key_and_value_of_bad_colour() {
        // given
        let raw = RawEntry { foreground: Some("CF8E6".to_owned()), ..RawEntry::default() };

        // when
        let error = raw.to_entry("colors.dark.\"JSON.KEYWORD\"").unwrap_err();

        // then
        assert_eq!(error, "colors.dark.\"JSON.KEYWORD\".foreground is \"CF8E6\", expected 6 hex digits");
    }

    #[test]
    fn should_reject_inherit_with_own_attributes() {
        // given
        let raw = RawEntry { inherit: true, bold: true, ..RawEntry::default() };

        // when
        let result = raw.to_entry("x");

        // then
        assert!(result.unwrap_err().contains("keep one"));
    }

    #[test]
    fn should_round_trip_entry_through_raw_form() {
        // given
        let entry = Entry::Own(TextAttributes { foreground: Some(0x0033B3), background: None, bold: true, italic: false });

        // when
        let back = RawEntry::from(entry).to_entry("x");

        // then
        assert_eq!(back, Ok(entry));
    }
}
