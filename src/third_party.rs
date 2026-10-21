//! The third-party software IVE includes, as the Settings page Third-Party Software lists it: compiled in from
//! `assets/licenses/third-party.json`, which `tools/third-party-notices.sh` writes.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::assets::Assets;

const DATA: &str = "licenses/third-party.json";

static BUNDLED: LazyLock<ThirdParty> = LazyLock::new(|| {
    let file = Assets::get(DATA).unwrap_or_else(|| panic!("{DATA} is not compiled in: run tools/third-party-notices.sh"));
    let json = String::from_utf8_lossy(file);
    ThirdParty::parse(&json).unwrap_or_else(|error| panic!("{DATA} is broken: {error}; run tools/third-party-notices.sh"))
});

/// What IVE ships; read once.
pub fn bundled() -> &'static ThirdParty {
    &BUNDLED
}

#[derive(Debug, Deserialize)]
pub struct ThirdParty {
    /// each distinct licence text once
    licenses: Vec<String>,
    /// sorted by name
    software: Vec<Software>,
}

/// One row: a crate, font, icon set or grammar.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Software {
    pub name: String,
    pub version: String,
    /// its project page
    pub url: String,
    /// as an SPDX expression, e.g. `MIT OR Apache-2.0`
    pub license: String,
    /// positions in [`ThirdParty::licenses`]
    texts: Vec<usize>,
}

impl ThirdParty {
    /// Fails when a row names a licence text the file does not hold.
    pub fn parse(json: &str) -> Result<Self, String> {
        let parsed: Self = serde_json::from_str(json).map_err(|error| error.to_string())?;
        let count = parsed.licenses.len();
        match parsed.software.iter().find(|software| software.texts.iter().any(|&text| text >= count)) {
            Some(software) => Err(format!("{} {} names a licence text beyond the {count} held", software.name, software.version)),
            None => Ok(parsed),
        }
    }

    pub fn software(&self) -> &[Software] {
        &self.software
    }

    /// The licence texts that apply to `software`, in the order listed.
    pub fn texts_of(&self, software: &Software) -> Vec<&str> {
        software.texts.iter().map(|&text| self.licenses[text].as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "licenses": ["MIT text", "Apache text"],
        "software": [
            {"name": "anyhow", "version": "1.0.0", "url": "https://github.com/dtolnay/anyhow", "license": "MIT OR Apache-2.0", "texts": [1, 0]},
            {"name": "Inter", "version": "", "url": "https://github.com/rsms/inter", "license": "OFL-1.1", "texts": [0]}
        ]
    }"#;

    #[test]
    fn should_give_texts_of_software_in_listed_order() {
        // given
        let third_party = ThirdParty::parse(SAMPLE).unwrap();

        // when
        let texts = third_party.texts_of(&third_party.software()[0]);

        // then
        assert_eq!(texts, ["Apache text", "MIT text"]);
    }

    #[test]
    fn should_reject_text_position_beyond_texts() {
        // given
        let json = SAMPLE.replace("[1, 0]", "[2]");

        // when
        let error = ThirdParty::parse(&json).unwrap_err();

        // then
        assert_eq!(error, "anyhow 1.0.0 names a licence text beyond the 2 held");
    }

    #[test]
    fn should_bundle_crates_and_other_software_sorted_by_name() {
        // given
        let third_party = bundled();

        // when
        let names: Vec<&str> = third_party.software().iter().map(|software| software.name.as_str()).collect();

        // then
        assert!(["gpui", "libgit2", "Inter", "tree-sitter-vue"].iter().all(|name| names.contains(name)), "{names:?}");
        assert!(!names.contains(&"ive"), "IVE lists itself");
        assert!(names.windows(2).all(|pair| pair[0].to_lowercase() <= pair[1].to_lowercase()), "not sorted");
        assert!(third_party.software().iter().all(|software| !third_party.texts_of(software).is_empty()));
    }
}
