//! Generated cardinal flow tile/link syntax with borrowed primitive tokens.
use super::{
    BodySource,
    context::{self, Checked},
    error::{ParseFailure, generated_failure},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        flow::{Builder, FlowTiles},
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Word(&'s str),
    Quote,
    Text(&'s str),
    Gap,
    Newline,
}
type Spanned<'s> = (usize, Token<'s>, usize);
pub(in crate::plantuml) struct Storage {
    bytes: usize,
    limit: usize,
}
impl Storage {
    pub fn insert<T>(
        &mut self,
        values: &mut Vec<T>,
        index: usize,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if values.len() == values.capacity() {
            let capacity = values.capacity().saturating_mul(2).max(4);
            let old = values.capacity() * std::mem::size_of::<T>();
            bound(
                "flow retained arena bytes",
                self.bytes
                    .saturating_add(capacity.saturating_mul(std::mem::size_of::<T>())),
                self.limit,
            )?;
            budget.charge(values.len())?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML flow allocation failed: {error}"
                    )))
                })?;
            self.bytes = self
                .bytes
                .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()))
                .saturating_sub(old);
        }
        budget.charge(values.len() - index + 1)?;
        values.insert(index, value);
        Ok(())
    }
    pub fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        self.insert(values, values.len(), value, budget)
    }
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<FlowTiles, RenderError> {
    let result = (|| {
        let tokens = lex(&body, budget)?;
        let storage = admit((&tokens, tokens.capacity()), &body, budget)?;
        let iterator = Checked::new(tokens, budget.control.clone());
        let mut builder = Builder::new(storage, budget);
        super::generated_flow::DocumentParser::new()
            .parse(&mut builder, iterator)
            .map_err(|error| {
                generated_failure(
                    error,
                    body.range.end,
                    "invalid or incomplete flow direction, ID, quoted label or line ending",
                )
            })?;
        builder.finish()
    })();
    result.map_err(|error: ParseFailure| error.into_document(body.document, budget))
}
fn lex<'s>(body: &BodySource<'s>, budget: &mut Budget) -> Result<Vec<Spanned<'s>>, ParseFailure> {
    let mut tokens = Vec::new();
    let mut base = body.range.start;
    for row in body.document.text[body.range.clone()].split_inclusive('\n') {
        for chunk in row.as_bytes().chunks(256) {
            budget.charge(chunk.len())?;
        }
        let text = row.trim();
        let start = base + row.len() - row.trim_start().len();
        let mut offset = 0;
        while offset < text.len() {
            let mut from = offset;
            let ch = text[offset..].chars().next().unwrap();
            let token = if ch.is_whitespace() {
                while offset < text.len() && text[offset..].chars().next().unwrap().is_whitespace()
                {
                    if (offset - from) % 256 == 0 {
                        budget.checkpoint()?;
                    }
                    offset += text[offset..].chars().next().unwrap().len_utf8();
                }
                Token::Gap
            } else if ch == '"' {
                context::push(
                    &mut tokens,
                    (start + offset, Token::Quote, start + offset + 1),
                    budget,
                )?;
                offset += 1;
                let close = text[offset..]
                    .rfind('"')
                    .map(|i| offset + i)
                    .unwrap_or(text.len());
                if close > offset {
                    context::push(
                        &mut tokens,
                        (
                            start + offset,
                            Token::Text(&text[offset..close]),
                            start + close,
                        ),
                        budget,
                    )?;
                }
                offset = close;
                if close == text.len() {
                    continue;
                }
                from = close;
                offset += 1;
                Token::Quote
            } else {
                while offset < text.len()
                    && (text.as_bytes()[offset].is_ascii_alphanumeric()
                        || text.as_bytes()[offset] == b'_')
                {
                    if (offset - from) % 256 == 0 {
                        budget.checkpoint()?;
                    }
                    offset += 1;
                }
                if offset == from {
                    return Err(ParseFailure::unsupported(
                        start + from..start + from + ch.len_utf8(),
                        "flow IDs are ASCII word tokens; unsupported command or suffix",
                    ));
                }
                Token::Word(&text[from..offset])
            };
            context::push(&mut tokens, (start + from, token, start + offset), budget)?;
        }
        if !text.is_empty() {
            context::push(
                &mut tokens,
                (start + text.len(), Token::Newline, base + row.len()),
                budget,
            )?;
        }
        base += row.len();
    }
    Ok(tokens)
}
fn admit(
    input: (&[Spanned<'_>], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<Storage, ParseFailure> {
    let (tokens, capacity) = input;
    // Complete closure of the actual generated table: 19 states, 5 terminals,
    // fixed 9 live states and at most 4 consecutive reductions/diagnostic steps.
    let (live, terminals, reductions) = (9usize, 5usize, 4usize);
    let stack = live.next_power_of_two().max(8);
    let diagnostic = (live + 1).next_power_of_two().max(8);
    let symbol = std::mem::size_of::<Token<'_>>() + 8;
    let slots = 2 * (stack + stack / 2)
        + 3 * (diagnostic + diagnostic / 2)
        + 3 * terminals.next_power_of_two();
    let fixed = (stack + stack / 2) * (symbol + 18)
        + (diagnostic + diagnostic / 2) * 6
        + terminals.next_power_of_two() * 3 * std::mem::size_of::<String>()
        + terminals * 32;
    bound("flow parser stack slots", slots, budget.limits.layout_work)?;
    bound(
        "flow parser fixed bytes",
        fixed,
        budget.limits.layout_work.saturating_mul(symbol + 24),
    )?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(reductions + 1)
            .saturating_add(reductions + slots + terminals * (live + reductions)),
    )?;
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned<'_>>())
        .saturating_add(body.adapters)
        .saturating_add(budget.limits.source_bytes.saturating_mul(8));
    let limit = budget.limits.source_bytes.saturating_mul(256);
    bound("flow parser retained bytes", bytes, limit)?;
    Ok(Storage { bytes, limit })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    #[test]
    fn should_reject_index_arena_growth_before_allocation_and_measure_symbol_payloads() {
        // given
        let mut storage = Storage { bytes: 0, limit: 1 };
        let mut values = Vec::new();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let result = storage.insert(&mut values, 0, 0usize, &mut budget);
        // then
        assert!(result.is_err());
        assert_eq!(values.capacity(), 0);
        assert_eq!(std::mem::size_of::<Token<'_>>(), 24);
        assert_eq!(std::mem::size_of::<Spanned<'_>>(), 40);
    }
}
