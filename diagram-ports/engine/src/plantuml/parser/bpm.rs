//! Generated legacy BPM branch structure with borrowed row/event lexemes.
use super::{
    BodySource,
    context::{self, Checked},
    error::{ParseFailure, generated_failure},
};
use crate::{
    RenderError,
    plantuml::{
        bpm::{Bpm, Builder},
        budget::{Budget, bound},
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    New,
    Branch,
    Else,
    End,
    Colon,
    Semi,
    Label(&'s str),
    Newline,
}
type Spanned<'s> = (usize, Token<'s>, usize);
pub(in crate::plantuml) struct Storage {
    bytes: usize,
    limit: usize,
}
impl Storage {
    pub fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if values.len() == values.capacity() {
            let capacity = values.capacity().saturating_mul(2).max(4);
            let old = values.capacity() * std::mem::size_of::<T>();
            bound(
                "BPM retained arena bytes",
                self.bytes
                    .saturating_add(capacity.saturating_mul(std::mem::size_of::<T>())),
                self.limit,
            )?;
            budget.charge(values.len() + 1)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML BPM allocation failed: {error}"
                    )))
                })?;
            self.bytes = self
                .bytes
                .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()))
                .saturating_sub(old);
        }
        budget.charge(1)?;
        values.push(value);
        Ok(())
    }
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Bpm, RenderError> {
    let result = (|| {
        let (tokens, depth) = lex(&body, budget)?;
        let storage = admit((&tokens, tokens.capacity()), (&body, depth), budget)?;
        let iterator = Checked::new(tokens, budget.control.clone());
        let mut builder = Builder::new(storage, budget)?;
        let root = super::generated_bpm::DocumentParser::new()
            .parse(&mut builder, iterator)
            .map_err(|error| {
                generated_failure(
                    error,
                    body.range.end,
                    "invalid or incomplete legacy BPM event/branch/else/end branch",
                )
            })?;
        builder.finish(root)
    })();
    result.map_err(|error: ParseFailure| error.into_document(body.document, budget))
}
fn lex<'s>(
    body: &BodySource<'s>,
    budget: &mut Budget,
) -> Result<(Vec<Spanned<'s>>, usize), ParseFailure> {
    let mut tokens = Vec::new();
    let mut base = body.range.start;
    let mut depth = 0usize;
    let mut maximum = 0usize;
    for row in body.document.text[body.range.clone()].split_inclusive('\n') {
        for chunk in row.as_bytes().chunks(256) {
            budget.charge(chunk.len())?;
        }
        let text = row.trim();
        let start = base + row.len() - row.trim_start().len();
        if !text.is_empty() {
            if let Some(label) = text.strip_prefix(':') {
                context::push(&mut tokens, (start, Token::Colon, start + 1), budget)?;
                let value = label.strip_suffix(';').unwrap_or(label);
                if !value.is_empty() {
                    context::push(
                        &mut tokens,
                        (start + 1, Token::Label(value), start + 1 + value.len()),
                        budget,
                    )?;
                }
                if label.ends_with(';') {
                    context::push(
                        &mut tokens,
                        (start + text.len() - 1, Token::Semi, start + text.len()),
                        budget,
                    )?;
                }
            } else {
                let mut cursor = 0;
                for word in text.split_whitespace() {
                    let position = text[cursor..].find(word).unwrap() + cursor;
                    cursor = position + word.len();
                    let token = match word {
                        "new" => {
                            depth += 1;
                            budget.depth(depth)?;
                            maximum = maximum.max(depth);
                            Token::New
                        }
                        "branch" => Token::Branch,
                        "else" => Token::Else,
                        "end" => {
                            depth = depth.saturating_sub(1);
                            Token::End
                        }
                        _ => {
                            return Err(ParseFailure::unsupported(
                                start + position..start + cursor,
                                "legacy BPM named merge/goto/resume, modern activity or unknown command",
                            ));
                        }
                    };
                    context::push(
                        &mut tokens,
                        (start + position, token, start + cursor),
                        budget,
                    )?;
                }
            }
            context::push(
                &mut tokens,
                (start + text.len(), Token::Newline, base + row.len()),
                budget,
            )?;
        }
        base += row.len();
    }
    Ok((tokens, maximum))
}
fn admit(
    input: (&[Spanned<'_>], usize),
    view: (&BodySource<'_>, usize),
    budget: &mut Budget,
) -> Result<Storage, ParseFailure> {
    let (tokens, capacity) = input;
    let (body, depth) = view;
    // Complete generated viable-prefix closure: 21 states, 8 terminals, 7d+6 live,
    // at most five consecutive reductions, including invalid-token diagnostics.
    let live = 7 * depth + 6;
    let terminals = 8usize;
    let reductions = 5usize;
    let stack = live.next_power_of_two().max(8);
    let diagnostic = (live + 1).next_power_of_two().max(8);
    let symbol = std::mem::size_of::<Token<'_>>().max(std::mem::size_of::<Vec<usize>>()) + 8;
    let slots = 2 * (stack + stack / 2)
        + 3 * (diagnostic + diagnostic / 2)
        + 3 * terminals.next_power_of_two();
    let fixed = (stack + stack / 2) * (symbol + 18)
        + (diagnostic + diagnostic / 2) * 6
        + terminals.next_power_of_two() * 3 * std::mem::size_of::<String>()
        + terminals * 32;
    bound("BPM parser stack slots", slots, budget.limits.layout_work)?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(reductions + 1)
            .saturating_add(reductions + slots + terminals * (live + reductions)),
    )?;
    bound(
        "BPM parser fixed bytes",
        fixed,
        budget.limits.layout_work.saturating_mul(symbol + 24),
    )?;
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned<'_>>())
        .saturating_add(body.adapters)
        .saturating_add(budget.limits.source_bytes.saturating_mul(8));
    let limit = budget.limits.source_bytes.saturating_mul(256);
    bound("BPM parser retained bytes", bytes, limit)?;
    Ok(Storage { bytes, limit })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    #[test]
    fn should_reject_arena_growth_before_allocation_and_measure_actual_payloads() {
        // given
        let mut storage = Storage { bytes: 0, limit: 1 };
        let mut values = Vec::new();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let rejected = storage.push(&mut values, 0usize, &mut budget);
        // then
        assert!(rejected.is_err());
        assert_eq!(values.capacity(), 0);
        assert_eq!(std::mem::size_of::<Token<'_>>(), 24);
        assert_eq!(std::mem::size_of::<Spanned<'_>>(), 40);
    }
}
