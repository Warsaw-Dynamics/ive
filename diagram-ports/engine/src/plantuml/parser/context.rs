use super::super::{
    budget::{Budget, bound},
    origin::SourceSpan,
};
use super::{definition::DefinitionSyntax, error::ParseFailure};

pub(in crate::plantuml) struct Context<'a> {
    pub budget: &'a mut Budget,
    pub authored: SourceSpan,
}

impl Context<'_> {
    pub fn finish_definition<'s>(
        &mut self,
        name: (usize, &'s str, usize),
        literal: (usize, &'s str, usize),
    ) -> Result<DefinitionSyntax<'s>, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if name.1.len() > 64 || name.1.as_bytes()[0].is_ascii_digit() {
            return Err(ParseFailure::syntax(
                name.0..name.2,
                "invalid literal definition name (ASCII identifier, at most 64 bytes)",
            ));
        }
        bound("definition literal bytes", literal.1.len(), 16 * 1024)
            .map_err(ParseFailure::terminal)?;
        let mut pending = 0;
        let mut trailing_slashes = 0;
        for ch in literal.1.chars() {
            pending += ch.len_utf8();
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
            if (ch.is_control() && ch != '\t') || matches!(ch, '\u{fffe}' | '\u{ffff}') {
                return Err(ParseFailure::unsupported(
                    literal.0..literal.2,
                    "literal definition control character",
                ));
            }
            trailing_slashes = if ch == '\\' { trailing_slashes + 1 } else { 0 };
        }
        self.budget
            .charge(pending)
            .map_err(ParseFailure::terminal)?;
        if trailing_slashes % 2 == 1 {
            return Err(ParseFailure::unsupported(
                literal.0..literal.2,
                "literal definition continuation",
            ));
        }
        Ok(DefinitionSyntax {
            name: name.1,
            literal: literal.1,
            name_span: self
                .authored
                .slice(name.0 - self.authored.start_byte..name.2 - self.authored.start_byte),
            literal_span: self
                .authored
                .slice(literal.0 - self.authored.start_byte..literal.2 - self.authored.start_byte),
        })
    }
}

pub(super) struct Admission {
    pub tokens: usize,
    pub shifts: usize,
    pub reductions: usize,
    pub stack_slots: usize,
    pub diagnostic_slots: usize,
    pub diagnostic_work: usize,
}

pub(super) const DEFINITION_ADMISSION: Admission = Admission {
    tokens: 5,
    shifts: 5,
    reductions: 2,
    stack_slots: 16,
    // Four expected terminals; simulated lookahead copies at most six states.
    diagnostic_slots: 6 + 4 + 4,
    diagnostic_work: 4 * (6 + 2),
};

impl Admission {
    pub fn admit(&self, count: usize, budget: &mut Budget) -> Result<(), crate::RenderError> {
        bound("definition parser tokens", count, self.tokens)?;
        bound(
            "definition parser stack slots",
            self.stack_slots + self.diagnostic_slots,
            budget.limits.layout_work,
        )?;
        // The fixed generated symbol enum is no larger than the sum of its
        // syntax/token payloads plus a tag. Include triples, state copies,
        // expected strings and the separately retained five-token buffer.
        let symbol_bytes = std::mem::size_of::<DefinitionSyntax<'_>>()
            + std::mem::size_of::<super::token::Token<'_>>()
            + 3 * std::mem::size_of::<usize>();
        let retained_bytes = 8 * symbol_bytes
            + 8
            + 6
            + 4 * (std::mem::size_of::<String>() + 16)
            + 5 * std::mem::size_of::<super::token::Spanned<'_>>();
        bound(
            "definition parser retained bytes",
            retained_bytes,
            budget.limits.source_bytes.saturating_mul(256),
        )?;
        budget.charge(
            self.shifts
                + self.reductions
                + self.stack_slots
                + self.diagnostic_slots
                + self.diagnostic_work,
        )
    }
}
