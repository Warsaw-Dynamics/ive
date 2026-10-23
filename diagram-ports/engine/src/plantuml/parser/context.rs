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
    pub shifts: usize,
    pub reductions: usize,
    pub stack_slots: usize,
    pub diagnostic_slots: usize,
    pub diagnostic_work: usize,
}

pub(super) const DEFINITION_ADMISSION: Admission = Admission {
    shifts: 5,
    reductions: 2,
    stack_slots: 16,
    // Five expected terminals; simulated lookahead copies at most six states.
    diagnostic_slots: 6 + 8 + 5,
    diagnostic_work: 5 * (6 + 2),
};

impl Admission {
    pub fn admit(
        &self,
        token_capacity: usize,
        budget: &mut Budget,
    ) -> Result<(), crate::RenderError> {
        bound(
            "definition parser stack slots",
            self.stack_slots + self.diagnostic_slots,
            budget.limits.layout_work,
        )?;
        // The fixed generated symbol enum is no larger than the sum of its
        // syntax/token payloads plus a tag. Include triples, state copies,
        // expected strings and the actual separately retained token capacity.
        let symbol_bytes = std::mem::size_of::<DefinitionSyntax<'_>>()
            + std::mem::size_of::<super::token::Token<'_>>()
            + 3 * std::mem::size_of::<usize>();
        let retained_bytes = 8 * symbol_bytes
            + 8
            + 6
            + 8 * std::mem::size_of::<String>()
            + 5 * 16
            + token_capacity * std::mem::size_of::<super::token::Spanned<'_>>();
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

pub(in crate::plantuml) fn push<T>(
    tokens: &mut Vec<T>,
    token: T,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    bound(
        "front-end buffered tokens",
        tokens.len() + 1,
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget.charge(1).map_err(ParseFailure::terminal)?;
    if tokens.len() == tokens.capacity() {
        let capacity = tokens.capacity().saturating_mul(2).max(4);
        bound(
            "front-end buffered bytes",
            capacity
                .saturating_add(tokens.capacity())
                .saturating_mul(std::mem::size_of::<T>()),
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
        budget
            .charge(tokens.len())
            .map_err(ParseFailure::terminal)?;
        tokens
            .try_reserve_exact(capacity - tokens.len())
            .map_err(|error| {
                ParseFailure::terminal(crate::RenderError::Backend(format!(
                    "PlantUML front-end token allocation for capacity {capacity} failed: {error}"
                )))
            })?;
        bound(
            "front-end actual buffer bytes",
            tokens.capacity().saturating_mul(std::mem::size_of::<T>()),
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
    }
    tokens.push(token);
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) enum Frontend {
    Document(usize),
    Header,
}

pub(super) fn admit_frontend(
    kind: Frontend,
    count: usize,
    capacity: usize,
    token_bytes: usize,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (terminals, symbol_slots, live_states, reductions, symbol_bytes) = match kind {
        Frontend::Document(_) => (
            47usize,
            8usize,
            7usize,
            4usize,
            std::mem::size_of::<super::document::Bounds>()
                + std::mem::size_of::<super::document::Token>(),
        ),
        Frontend::Header => (
            25,
            4,
            4,
            4,
            std::mem::size_of::<super::header::Tail>()
                + std::mem::size_of::<super::header::Head>()
                + std::mem::size_of::<super::header::Evidence>()
                + std::mem::size_of::<super::header::Atom>(),
        ),
    };
    let diagnostic_capacity = terminals.next_power_of_two().max(4);
    let shared_bytes = match kind {
        Frontend::Document(bytes) => bytes,
        Frontend::Header => 0,
    };
    let retained = capacity
        .saturating_mul(token_bytes)
        .saturating_add(shared_bytes)
        .saturating_add(symbol_slots * (symbol_bytes + 3 * std::mem::size_of::<usize>()))
        // Include old+new vectors during diagnostic growth and generated stack growth.
        .saturating_add(40 + symbol_slots / 2 * (symbol_bytes + 3 * std::mem::size_of::<usize>()))
        .saturating_add(diagnostic_capacity * 2 * std::mem::size_of::<String>())
        .saturating_add(terminals * 32);
    bound(
        "front-end parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = 8 + symbol_slots + 16 + diagnostic_capacity * 2;
    bound(
        "front-end parser stack slots",
        slots,
        budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(
            count
                .saturating_mul(reductions + 1)
                .saturating_add(reductions + slots + terminals * (live_states + reductions)),
        )
        .map_err(ParseFailure::terminal)
}

pub(super) struct Checked<T> {
    tokens: std::vec::IntoIter<T>,
    control: crate::RenderControl,
}

impl<T> Checked<T> {
    pub fn new(tokens: Vec<T>, control: crate::RenderControl) -> Self {
        Self {
            tokens: tokens.into_iter(),
            control,
        }
    }
}

impl<T> Iterator for Checked<T> {
    type Item = Result<T, ParseFailure>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.control.checkpoint() {
            Ok(()) => self.tokens.next().map(Ok),
            Err(error) => Some(Err(ParseFailure::terminal(crate::RenderError::Cancelled(
                error.to_string(),
            )))),
        }
    }
}
