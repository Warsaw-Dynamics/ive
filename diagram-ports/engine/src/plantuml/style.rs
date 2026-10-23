//! Bounded subset of PlantUML MIT skinparam migration and plain style selectors.
//! Copyright 2009-2024 Arnaud Roques; source traces/notices in provenance manifest.
use super::budget::{Budget, unsupported};
use crate::RenderError;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct Color(pub u32);
impl Color {
    pub fn svg(self) -> String {
        format!("#{:06x}", self.0)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct Font {
    pub family: Arc<str>,
    pub size: u16,
    pub weight: u16,
    pub italic: bool,
    pub color: Color,
}
impl Default for Font {
    fn default() -> Self {
        Self {
            family: Arc::from(""),
            size: 14,
            weight: 400,
            italic: false,
            color: Color(0x172033),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Element,
    Title,
    Legend,
    Header,
    Footer,
}
impl Role {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Element => "element",
            Self::Title => "title",
            Self::Legend => "legend",
            Self::Header => "header",
            Self::Footer => "footer",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Alignment {
    Left,
    Center,
    Right,
}
#[derive(Clone, Debug)]
pub(super) struct Region {
    pub role: Role,
    pub text: String,
    pub align: Alignment,
}

pub(super) struct Preamble {
    pub fonts: [Font; 5],
    pub background: Color,
    pub regions: Vec<Region>,
}
impl Default for Preamble {
    fn default() -> Self {
        let mut fonts = std::array::from_fn(|_| Font::default());
        fonts[Role::Title.index()].weight = 700;
        for role in [Role::Header, Role::Footer] {
            fonts[role.index()].size = 10;
            fonts[role.index()].color = Color(0x888888);
        }
        Self {
            fonts,
            background: Color(0xffffff),
            regions: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Root,
    Element,
    Document,
    Region(Role),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PropertyKey {
    FontName,
    FontColor,
    FontSize,
    FontStyle,
    BackgroundColor,
}

pub(super) enum Property<'s> {
    FontName(&'s str),
    FontColor(Color),
    FontSize(u16),
    FontStyle(u16, bool),
    BackgroundColor(Color),
}

pub(super) fn decode_property<'s>(
    key: PropertyKey,
    value: &'s str,
) -> Result<Property<'s>, &'static str> {
    match key {
        PropertyKey::FontColor => decode_color(value).map(Property::FontColor),
        PropertyKey::BackgroundColor => decode_color(value).map(Property::BackgroundColor),
        PropertyKey::FontSize => {
            if value.len() > 2 || !value.bytes().all(|ch| ch.is_ascii_digit()) {
                return Err("font size requires an integer from 6 through 96");
            }
            match value.parse::<u16>() {
                Ok(size @ 6..=96) => Ok(Property::FontSize(size)),
                _ => Err("font size requires an integer from 6 through 96"),
            }
        }
        PropertyKey::FontName => {
            let value = value
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(value);
            if value.len() > 128
                || value.trim().is_empty()
                || !value
                    .chars()
                    .all(|ch| ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '_'))
            {
                return Err(
                    "font name requires 1..128 literal letters, digits, spaces, hyphens or underscores",
                );
            }
            Ok(Property::FontName(value))
        }
        PropertyKey::FontStyle => {
            for (name, weight, italic) in [
                ("plain", 400, false),
                ("normal", 400, false),
                ("bold", 700, false),
                ("italic", 400, true),
            ] {
                if value.eq_ignore_ascii_case(name) {
                    return Ok(Property::FontStyle(weight, italic));
                }
            }
            Err("font style requires plain, normal, bold or italic")
        }
    }
}

impl Preamble {
    pub fn apply(
        &mut self,
        target: Target,
        property: Property<'_>,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        budget.item()?;
        if let Property::BackgroundColor(color) = property {
            self.background = color;
            return Ok(());
        }
        for role in [
            Role::Element,
            Role::Title,
            Role::Legend,
            Role::Header,
            Role::Footer,
        ] {
            if target == Target::Root
                || target == Target::Region(role)
                || (target == Target::Element && role == Role::Element)
            {
                let font = &mut self.fonts[role.index()];
                match property {
                    Property::FontName(value) => font.family = value.into(),
                    Property::FontColor(value) => font.color = value,
                    Property::FontSize(value) => font.size = value,
                    Property::FontStyle(weight, italic) => {
                        font.weight = weight;
                        font.italic = italic;
                    }
                    Property::BackgroundColor(_) => unreachable!(),
                }
            }
        }
        Ok(())
    }
}

fn decode_color(value: &str) -> Result<Color, &'static str> {
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].bytes().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err("color requires exactly six hexadecimal digits after #");
    }
    u32::from_str_radix(&value[1..], 16)
        .map(Color)
        .map_err(|_| "invalid hexadecimal color")
}

pub(super) fn color(value: &str, line: usize) -> Result<Color, RenderError> {
    decode_color(value).map_err(|_| unsupported(line, value))
}
