//! Bounded subset of PlantUML MIT skinparam migration and plain style selectors.
//! Copyright 2009-2024 Arnaud Roques; source traces/notices in provenance manifest.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::{Line, Wrapper},
};
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
    fn parse(text: &str) -> Option<Self> {
        match text {
            "title" => Some(Self::Title),
            "legend" => Some(Self::Legend),
            "header" => Some(Self::Header),
            "footer" => Some(Self::Footer),
            _ => None,
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
    pub start: usize,
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
            start: 0,
        }
    }
}

pub(super) fn prepare(
    lines: &[Line],
    wrapper: Wrapper,
    budget: &mut Budget,
) -> Result<Preamble, RenderError> {
    let mut result = Preamble::default();
    let mut syntax = 0;
    let data = matches!(wrapper, Wrapper::Json | Wrapper::Yaml);
    while let Some(line) = lines.get(result.start) {
        budget.charge(line.text.len() + 1)?;
        let text = line.text.trim();
        if text.is_empty() {
            result.start += 1;
            continue;
        }
        if wrapper == Wrapper::Yaml && super::yaml::find_unquoted(text, ':').is_some() {
            break;
        }
        // Keyword-like sequence endpoints belong to their family grammar.
        if super::has_identifier_arrow(line) {
            break;
        }
        if text.eq_ignore_ascii_case("<style>") {
            if syntax == 1 {
                return Err(unsupported(
                    line.number,
                    "mixed skinparam/style declarations",
                ));
            }
            syntax = 2;
            sheet(lines, &mut result, budget)?;
        } else if text.to_ascii_lowercase().starts_with("skinparam") {
            if data || syntax == 2 {
                return Err(unsupported(line.number, text));
            }
            syntax = 1;
            skinparam(line, &mut result, budget)?;
            result.start += 1;
        } else if let Some(region) = region(line, wrapper, budget)? {
            result.regions.retain(|old| old.role != region.role);
            result.regions.push(region);
            result.start += 1;
        } else {
            break;
        }
    }
    if result.start == lines.len() {
        return Err(invalid(1, "document chrome requires a diagram body"));
    }
    Ok(result)
}

fn region(
    line: &Line,
    wrapper: Wrapper,
    budget: &mut Budget,
) -> Result<Option<Region>, RenderError> {
    let text = line.text.trim();
    let data = matches!(wrapper, Wrapper::Json | Wrapper::Yaml);
    // Data extraction deliberately recognizes only the upstream prefix form.
    if data && !text.starts_with("title ") {
        let lower = text.to_ascii_lowercase();
        let (first, tail) = word(&lower);
        let command = if matches!(first, "left" | "center" | "right") {
            word(tail.trim_start()).0
        } else {
            first
        };
        if matches!(command, "legend" | "header" | "footer") {
            return Err(unsupported(
                line.number,
                "data preamble supports only title and style",
            ));
        }
        return Ok(None);
    }
    let (head, mut rest) = word(text);
    let lower = head.to_ascii_lowercase();
    let (name, align) = match lower.as_str() {
        "left" | "center" | "right" => {
            let (name, tail) = word(rest.trim_start());
            if !name.eq_ignore_ascii_case("header") && !name.eq_ignore_ascii_case("footer") {
                return Ok(None);
            }
            rest = tail;
            (
                name.to_ascii_lowercase(),
                match lower.as_str() {
                    "left" => Alignment::Left,
                    "right" => Alignment::Right,
                    _ => Alignment::Center,
                },
            )
        }
        _ => (
            lower.clone(),
            if lower == "header" {
                Alignment::Right
            } else {
                Alignment::Center
            },
        ),
    };
    let Some(role) = Role::parse(&name) else {
        return Ok(None);
    };
    let payload = rest
        .trim_start()
        .strip_prefix(':')
        .unwrap_or(rest.trim_start())
        .trim_start();
    if payload.is_empty() {
        return Err(unsupported(line.number, "multiline/empty document region"));
    }
    bound("label bytes", payload.len(), 16 * 1024)?;
    let value = if data {
        payload
    } else if payload.starts_with('"') {
        payload
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .ok_or_else(|| invalid(line.number, "unclosed document label quote"))?
    } else {
        if !payload
            .chars()
            .any(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
        {
            return Err(invalid(
                line.number,
                "unquoted document label requires a letter, number, underscore or dot",
            ));
        }
        payload
    };
    if value.trim().is_empty() {
        return Err(invalid(line.number, "empty document region"));
    }
    budget.item()?;
    budget.text(value.len())?;
    Ok(Some(Region {
        role,
        text: value.into(),
        align,
    }))
}

fn word(text: &str) -> (&str, &str) {
    let end = text
        .find(|ch: char| ch.is_whitespace() || ch == ':')
        .unwrap_or(text.len());
    (&text[..end], &text[end..])
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Root,
    Element,
    Document,
    Region(Role),
}

fn skinparam(line: &Line, result: &mut Preamble, budget: &mut Budget) -> Result<(), RenderError> {
    let (command, rest) = word(&line.text);
    if !command.eq_ignore_ascii_case("skinparam") {
        return Err(unsupported(line.number, command));
    }
    let (key, value) = word(rest.trim_start());
    let key = key.to_ascii_lowercase();
    let (target, property) = match key.as_str() {
        "defaultfontname" => (Target::Root, "fontname"),
        "defaultfontcolor" => (Target::Root, "fontcolor"),
        "defaultfontsize" => (Target::Element, "fontsize"),
        "backgroundcolor" => (Target::Document, "backgroundcolor"),
        _ => {
            let Some(role) = [Role::Title, Role::Legend, Role::Header, Role::Footer]
                .into_iter()
                .find(|role| key.starts_with(role.name()))
            else {
                return Err(unsupported(line.number, &key));
            };
            (Target::Region(role), &key[role.name().len()..])
        }
    };
    declaration(
        result,
        (target, property, value.trim()),
        line.number,
        budget,
    )
}

fn sheet(lines: &[Line], result: &mut Preamble, budget: &mut Budget) -> Result<(), RenderError> {
    let mut stack: Vec<(Target, usize)> = Vec::new();
    let mut count = 0;
    result.start += 1;
    while let Some(line) = lines.get(result.start) {
        budget.charge(line.text.len() + 1)?;
        result.start += 1;
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }
        if text.eq_ignore_ascii_case("</style>") {
            return if stack.is_empty() && count > 0 {
                Ok(())
            } else {
                Err(invalid(line.number, "empty or unclosed style rules"))
            };
        }
        if text == "}" {
            let Some((_, start)) = stack.pop() else {
                return Err(invalid(line.number, "unmatched style closing brace"));
            };
            if start == count {
                return Err(invalid(line.number, "empty style rule"));
            }
        } else if let Some(selector) = text.strip_suffix('{') {
            budget.depth(stack.len() + 1)?;
            budget.item()?;
            let selector = selector.trim().to_ascii_lowercase();
            let target = match (stack.last().map(|s| s.0), selector.as_str()) {
                (None, "root") => Target::Root,
                (None, "element") => Target::Element,
                (None, "document") => Target::Document,
                (Some(Target::Document), value) => Role::parse(value)
                    .map(Target::Region)
                    .ok_or_else(|| unsupported(line.number, selector.as_str()))?,
                _ => return Err(unsupported(line.number, selector.as_str())),
            };
            stack.push((target, count));
        } else {
            let Some((target, _)) = stack.last() else {
                return Err(invalid(
                    line.number,
                    format!("style declaration outside selector: {text}"),
                ));
            };
            let (property, value) = word(text);
            let value = value
                .trim()
                .strip_suffix(';')
                .unwrap_or(value.trim())
                .trim_end();
            declaration(
                result,
                (*target, &property.to_ascii_lowercase(), value),
                line.number,
                budget,
            )?;
            count += 1;
        }
    }
    Err(invalid(1, "unterminated <style> block"))
}

fn declaration(
    result: &mut Preamble,
    declaration: (Target, &str, &str),
    line: usize,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (target, key, value) = declaration;
    budget.item()?;
    if target == Target::Document {
        if key != "backgroundcolor" {
            return Err(unsupported(line, key));
        }
        result.background = color(value, line)?;
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
            budget.charge(value.len() + 1)?;
            budget.text(value.len())?;
            property(&mut result.fonts[role.index()], (key, value), line)?;
        }
    }
    Ok(())
}

fn property(font: &mut Font, property: (&str, &str), line: usize) -> Result<(), RenderError> {
    let (key, value) = property;
    match key {
        "fontcolor" => font.color = color(value, line)?,
        "fontsize" => {
            if value.len() > 2 || !value.bytes().all(|c| c.is_ascii_digit()) {
                return Err(unsupported(line, value));
            }
            let size = value.parse::<u16>().map_err(|_| unsupported(line, value))?;
            if !(6..=96).contains(&size) {
                return Err(unsupported(line, value));
            }
            font.size = size;
        }
        "fontname" => {
            let value = value
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(value);
            if value.trim().is_empty()
                || value.len() > 128
                || !value
                    .chars()
                    .all(|ch| ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '_'))
            {
                return Err(unsupported(line, value));
            }
            font.family = value.into();
        }
        "fontstyle" => {
            (font.weight, font.italic) = match value.to_ascii_lowercase().as_str() {
                "plain" | "normal" => (400, false),
                "bold" => (700, false),
                "italic" => (400, true),
                _ => return Err(unsupported(line, value)),
            };
        }
        _ => return Err(unsupported(line, key)),
    }
    Ok(())
}

fn color(value: &str, line: usize) -> Result<Color, RenderError> {
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].bytes().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err(unsupported(line, value));
    }
    Ok(Color(
        u32::from_str_radix(&value[1..], 16).map_err(|_| unsupported(line, value))?,
    ))
}
