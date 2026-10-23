//! Shared bounded physical-row lexing and arena admission for outline grammars.
use super::{BodySource, context, error::ParseFailure, ownership::Row};
use crate::{
    RenderError,
    plantuml::budget::{Budget, bound},
};
use std::ops::Range;

pub(in crate::plantuml) type Spanned<T> = (usize, T, usize);

pub(in crate::plantuml) struct Lexer<'b, T> {
    pub tokens: Vec<Spanned<T>>,
    pub budget: &'b mut Budget,
    literals: usize,
}

impl<T> Lexer<'_, T> {
    pub fn push(&mut self, token: T, span: Range<usize>) -> Result<(), ParseFailure> {
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }

    pub fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.literals = self.literals.saturating_add(value.len());
        bound(
            "outline syntax literal bytes",
            self.literals,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        for chunk in value.as_bytes().chunks(256) {
            self.budget
                .charge(chunk.len())
                .map_err(ParseFailure::terminal)?;
        }
        Ok(value.to_owned())
    }
}

pub(in crate::plantuml) fn lex<T>(
    body: &BodySource<'_>,
    budget: &mut Budget,
    mut tokenize: impl FnMut(&Row, usize, &mut Lexer<'_, T>) -> Result<(), ParseFailure>,
) -> Result<Vec<Spanned<T>>, ParseFailure> {
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut ownership = body.ownership;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        budget,
        literals: 0,
    };
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        tokenize(&row, base, &mut lexer)?;
        base += physical.len();
    }
    ownership.finish(base)?;
    Ok(lexer.tokens)
}

pub(in crate::plantuml) struct Arena {
    bytes: usize,
}

impl Arena {
    #[cfg(test)]
    pub fn for_test() -> Self {
        Self { bytes: 0 }
    }
    #[cfg(test)]
    pub fn for_stream<T>(
        input: (&[Spanned<T>], usize),
        budget: &mut Budget,
        syntax: Syntax,
    ) -> Result<Self, ParseFailure> {
        let body = BodySource {
            document: &crate::plantuml::origin::EMPTY,
            range: 0..0,
            ownership: super::ownership::Ownership::default(),
            adapters: 0,
        };
        Self::admit(input, &body, budget, syntax)
    }
    pub fn admit<T>(
        input: (&[Spanned<T>], usize),
        body: &BodySource<'_>,
        budget: &mut Budget,
        syntax: Syntax,
    ) -> Result<Self, ParseFailure> {
        let (tokens, token_capacity) = input;
        // Exhaustive table witnesses: fixed stacks even for arbitrarily many rows.
        let (live, terminals, symbol, builder) = match syntax {
            Syntax::Mindmap => (
                6usize,
                15usize,
                std::mem::size_of::<super::tree::Marker>()
                    + std::mem::size_of::<T>()
                    + 4 * std::mem::size_of::<usize>(),
                std::mem::size_of::<crate::plantuml::tree::MindmapBuilder<'_>>(),
            ),
            Syntax::Wbs => (
                13,
                15,
                std::mem::size_of::<super::tree::Marker>()
                    + std::mem::size_of::<T>()
                    + 4 * std::mem::size_of::<usize>(),
                std::mem::size_of::<crate::plantuml::tree::WbsBuilder<'_>>(),
            ),
            Syntax::Files => (
                6,
                6,
                std::mem::size_of::<T>() + 5 * std::mem::size_of::<usize>(),
                std::mem::size_of::<crate::plantuml::files::Builder<'_>>(),
            ),
            Syntax::Board => (
                5,
                3,
                std::mem::size_of::<T>() + 4 * std::mem::size_of::<usize>(),
                std::mem::size_of::<crate::plantuml::board::Builder<'_>>(),
            ),
        };
        let reductions = 4;
        let capacity = live.next_power_of_two().max(8);
        let slots =
            (capacity + capacity / 2) * 2 + (live + 1) * 3 + terminals.next_power_of_two() * 3;
        bound(
            "outline parser stack slots",
            slots,
            budget.limits.layout_work,
        )
        .map_err(ParseFailure::terminal)?;
        budget
            .charge(
                tokens
                    .len()
                    .saturating_mul(reductions + 1)
                    .saturating_add(reductions + slots + terminals * (live + reductions)),
            )
            .map_err(ParseFailure::terminal)?;
        let bytes = token_capacity
            .saturating_mul(std::mem::size_of::<Spanned<T>>())
            .saturating_add(body.adapters)
            .saturating_add((capacity + capacity / 2) * (symbol + 2))
            .saturating_add((live + 1) * 3 * std::mem::size_of::<usize>())
            .saturating_add(
                terminals.next_power_of_two() * 3 * std::mem::size_of::<String>() + terminals * 32,
            )
            // Twice the existing semantic text quota plus all lexical literal capacities.
            .saturating_add(budget.limits.source_bytes.saturating_mul(20))
            .saturating_add(builder);
        bound(
            "outline parser retained bytes",
            bytes,
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
        Ok(Self { bytes })
    }

    pub fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        budget.charge(1).map_err(ParseFailure::terminal)?;
        if values.len() == values.capacity() {
            let capacity = values.capacity().saturating_mul(2).max(4);
            let growth = capacity.saturating_mul(std::mem::size_of::<T>());
            bound(
                "outline retained arena bytes",
                self.bytes.saturating_add(growth),
                budget.limits.source_bytes.saturating_mul(256),
            )
            .map_err(ParseFailure::terminal)?;
            budget
                .charge(values.len())
                .map_err(ParseFailure::terminal)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML outline arena allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.bytes = self
                .bytes
                .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()));
            bound(
                "outline actual retained arena bytes",
                self.bytes,
                budget.limits.source_bytes.saturating_mul(256),
            )
            .map_err(ParseFailure::terminal)?;
        }
        values.push(value);
        Ok(())
    }
}

pub(in crate::plantuml) enum Syntax {
    Mindmap,
    Wbs,
    Files,
    Board,
}

#[cfg(test)]
pub(in crate::plantuml) fn test_parse<T>(
    lines: &[crate::plantuml::source::Line],
    wrapper: &str,
    budget: &mut Budget,
    parse: impl FnOnce(BodySource<'_>, &mut Budget) -> Result<T, RenderError>,
) -> Result<T, RenderError> {
    let body = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!("@start{wrapper}\n{body}\n@end{wrapper}");
    let document = crate::plantuml::preprocess::prepare(&source, budget)?;
    let mut prepared = super::document::parse(&document, budget)?;
    super::preamble::parse(&mut prepared, budget)?;
    parse(prepared.body, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    #[test]
    fn should_admit_actual_generated_stacks_and_arenas_before_allocation() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.layout_work = 1;
        let tokens = vec![(0, super::super::board::Token::Newline, 0)];
        // when
        let result = Arena::for_stream((&tokens, tokens.capacity()), &mut budget, Syntax::Board);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Backend(_)),
                ..
            })
        ));
        let mut budget = Budget::new(&RenderOptions::default());
        let limit = budget.limits.source_bytes * 256;
        let mut arena = Arena { bytes: limit };
        let mut values = Vec::<usize>::new();
        assert!(arena.push(&mut values, 1, &mut budget).is_err());
        assert_eq!(values.capacity(), 0);
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn should_cancel_shared_row_lexing_before_owning_long_literals() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let source = format!("@startfiles\n/{}\n@endfiles", "界".repeat(3000));
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        budget.control.cancel_after_checkpoints(4);
        // when
        let result = super::super::files::parse(prepared.body, &mut budget);
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }
}
