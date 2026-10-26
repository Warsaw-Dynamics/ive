//! Packetdiag range/config semantics translated from authorized PlantUML MIT packetdiag sources.
//! Copyright 2009-2024/2009-2026 Arnaud Roques; kolulu23, Duy Nguyen, The-Lum.
//! MIT grant and precise per-file credits/hashes retained in provenance/.
use super::{
    budget::{Budget, bound},
    parser::error::ParseFailure,
};
use crate::RenderError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Direction {
    Ltr,
    Rtl,
}
pub(super) struct Config {
    pub columns: usize,
    pub interval: usize,
    pub direction: Direction,
    pub node_height: usize,
    pub same_height: bool,
}
pub(super) struct Field {
    pub start: usize,
    pub end: usize,
    pub label: String,
    pub height: usize,
}
#[derive(Debug)]
pub(super) struct Fragment {
    pub field: usize,
    pub row: usize,
    pub offset: usize,
    pub width: usize,
}
pub(super) struct Packet {
    pub config: Config,
    pub fields: Vec<Field>,
    pub fragments: Vec<Fragment>,
    pub rows: usize,
}

#[derive(Clone, Copy)]
pub(in crate::plantuml) enum Range {
    Unit(usize),
    Inclusive(usize, usize),
    Auto,
}
#[derive(Default)]
pub(in crate::plantuml) struct Attributes {
    length: usize,
    height: usize,
}
pub(in crate::plantuml) enum Attribute {
    Length(usize),
    Height(usize),
}
pub(in crate::plantuml) enum Configuration {
    Columns(usize),
    Interval(usize),
    NodeHeight(usize),
    Direction(Direction),
    SameHeight(bool),
}
pub(in crate::plantuml) struct FieldInput {
    pub range: Range,
    pub label: String,
    pub attributes: Attributes,
}
pub(in crate::plantuml) struct Builder<'b> {
    budget: &'b mut Budget,
    config: Config,
    fields: Vec<Field>,
}

impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            config: Config {
                columns: 16,
                interval: 0,
                direction: Direction::Ltr,
                node_height: 32,
                same_height: false,
            },
            fields: Vec::new(),
        }
    }
    pub fn configure(
        &mut self,
        configuration: Configuration,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        match configuration {
            Configuration::Columns(value) => {
                self.config.columns = positive(value, &span)?;
                bound("packet colwidth", value, 999).map_err(ParseFailure::terminal)?;
            }
            Configuration::Interval(value) => self.config.interval = positive(value, &span)?,
            Configuration::NodeHeight(value) => {
                self.config.node_height = positive(value, &span)?;
                bound("packet node height", value, 999).map_err(ParseFailure::terminal)?;
            }
            Configuration::Direction(value) => self.config.direction = value,
            Configuration::SameHeight(value) => self.config.same_height = value,
        }
        Ok(())
    }
    pub fn attribute(
        &mut self,
        mut attributes: Attributes,
        attribute: (Attribute, std::ops::Range<usize>),
    ) -> Result<Attributes, ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        let (attribute, span) = attribute;
        match attribute {
            Attribute::Length(value) if attributes.length == 0 => {
                attributes.length = positive(value, &span)?
            }
            Attribute::Height(value) if attributes.height == 0 => {
                attributes.height = positive(value, &span)?
            }
            _ => {
                return Err(ParseFailure::unsupported(
                    span,
                    "duplicate packet len/height attribute",
                ));
            }
        }
        Ok(attributes)
    }
    pub fn field(
        &mut self,
        input: FieldInput,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        bound("label bytes", input.label.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        let (start, end) = self.resolve(input.range, input.attributes.length, span.clone())?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(input.label.len())
            .map_err(ParseFailure::terminal)?;
        super::parser::context::push(
            &mut self.fields,
            Field {
                start,
                end,
                label: input.label,
                height: input.attributes.height.max(1),
            },
            self.budget,
        )
    }
    fn resolve(
        &mut self,
        range: Range,
        length: usize,
        span: std::ops::Range<usize>,
    ) -> Result<(usize, usize), ParseFailure> {
        let (start, end) = match range {
            Range::Inclusive(start, end) => {
                if length != 0 {
                    return Err(ParseFailure::syntax(
                        span,
                        "len attribute cannot override explicit packet range",
                    ));
                }
                (start, end)
            }
            Range::Unit(start) => (
                start,
                start
                    .checked_add(length.max(1) - 1)
                    .ok_or_else(|| ParseFailure::syntax(span.clone(), "packet end overflow"))?,
            ),
            Range::Auto => {
                let start = self.fields.last().map_or(0, |field| field.end + 1);
                (
                    start,
                    start
                        .checked_add(length.max(1) - 1)
                        .ok_or_else(|| ParseFailure::syntax(span.clone(), "packet end overflow"))?,
                )
            }
        };
        if end < start || end > 9_999_999 {
            self.budget.text(128).map_err(ParseFailure::terminal)?;
            return Err(ParseFailure::owned_syntax(
                span,
                format!("invalid inclusive packet range {start}-{end}"),
            ));
        }
        Ok((start, end))
    }
    pub fn unknown(
        &mut self,
        name: String,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        Err(self.reject(name, span))
    }
    pub fn unknown_attribute(
        &mut self,
        name: String,
        span: std::ops::Range<usize>,
    ) -> Result<(Attribute, std::ops::Range<usize>), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        Err(self.reject(name, span))
    }
    fn reject(&mut self, name: String, span: std::ops::Range<usize>) -> ParseFailure {
        if let Err(error) = self
            .budget
            .text(name.len().saturating_mul(4).saturating_add(128))
        {
            return ParseFailure::terminal(error);
        }
        ParseFailure {
            span,
            kind: super::parser::error::FailureKind::Unsupported(
                format!("unsupported packet command or attribute {name:?}").into(),
            ),
        }
    }
    pub fn finish(mut self, span: std::ops::Range<usize>) -> Result<Packet, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if self.fields.is_empty() {
            return Err(ParseFailure::syntax(span, "packetdiag requires fields"));
        }
        if self.config.interval == 0 {
            self.config.interval = (self.config.columns / 2).max(1);
        }
        self.config.interval = self.config.interval.min(self.config.columns);
        let fragments =
            fragment(&self.fields, &self.config, self.budget).map_err(ParseFailure::terminal)?;
        let rows = fragments.last().map_or(0, |fragment| fragment.row + 1);
        Ok(Packet {
            config: self.config,
            fields: self.fields,
            fragments,
            rows,
        })
    }
}

fn positive(value: usize, span: &std::ops::Range<usize>) -> Result<usize, ParseFailure> {
    if value == 0 {
        return Err(ParseFailure::syntax(
            span.clone(),
            "packet value must be positive: 0",
        ));
    }
    Ok(value)
}

fn fragment(
    fields: &[Field],
    config: &Config,
    budget: &mut Budget,
) -> Result<Vec<Fragment>, RenderError> {
    let mut result = Vec::new();
    let mut row = 0;
    let mut offset = 0;
    for (id, field) in fields.iter().enumerate() {
        let mut remaining = field.end - field.start + 1;
        while remaining > 0 {
            budget.item()?;
            let width = remaining.min(config.columns - offset);
            result.push(Fragment {
                field: id,
                row,
                offset,
                width,
            });
            remaining -= width;
            offset += width;
            if offset == config.columns {
                row += 1;
                offset = 0;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_reject_oversized_packet_label_before_fragment_generation() {
        // given
        let lines = [format!("0: {}", "x".repeat(16_385))].join("\n");
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let result = crate::plantuml::parser::test_support::body(
            (&lines, "packetdiag"),
            &mut budget,
            crate::plantuml::parser::packet::parse,
        );
        // then
        assert!(
            result.is_err(),
            "oversized field retained before fragmentation"
        );
    }
}
