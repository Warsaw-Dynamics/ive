//! Bounded borrowed data lexemes and generated semantic-vector admission.
use super::{BodySource, error::ParseFailure};
use crate::{
    RenderError,
    plantuml::budget::{Budget, bound},
};

pub(in crate::plantuml) struct Cursor<'s, 'b> {
    pub input: &'s str,
    pub offset: usize,
    pub end: usize,
    pub budget: &'b mut Budget,
}

impl<'s, 'b> Cursor<'s, 'b> {
    pub fn new(body: &BodySource<'s>, budget: &'b mut Budget) -> Self {
        Self {
            input: &body.document.text,
            offset: body.range.start,
            end: body.range.end,
            budget,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.input[self.offset..self.end].chars().next()
    }

    pub fn take(&mut self) -> Result<char, ParseFailure> {
        let ch = self
            .peek()
            .ok_or_else(|| ParseFailure::syntax(self.offset..self.offset, "unexpected data end"))?;
        self.budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        self.offset += ch.len_utf8();
        Ok(ch)
    }
}

pub(in crate::plantuml) enum Syntax {
    Json,
    Hcl,
}

pub(in crate::plantuml) trait DepthToken {
    fn depth_delta(&self) -> isize;
}

pub(in crate::plantuml) struct Arena {
    bytes: usize,
    limit: usize,
}

fn measure_symbol<T>(syntax: &Syntax) -> usize {
    let payloads = match syntax {
        Syntax::Json => [
            std::mem::size_of::<T>(),
            std::mem::size_of::<crate::plantuml::data::Value>(),
            std::mem::size_of::<(String, crate::plantuml::data::Value)>(),
            std::mem::size_of::<Vec<crate::plantuml::data::Value>>(),
            std::mem::size_of::<String>(),
            std::mem::size_of::<usize>(),
        ],
        Syntax::Hcl => [
            std::mem::size_of::<T>(),
            std::mem::size_of::<crate::plantuml::hcl::Field>(),
            std::mem::size_of::<crate::plantuml::hcl::Block>(),
            std::mem::size_of::<crate::plantuml::hcl::HclValue>(),
            std::mem::size_of::<super::hcl::Word>(),
            std::mem::size_of::<super::hcl::Header>(),
        ],
    };
    // Generated enum tags need at most usize; every concrete payload is usize-aligned.
    payloads.into_iter().max().unwrap_or(0) + std::mem::size_of::<usize>()
}

impl Arena {
    pub fn admit<T: DepthToken>(
        input: (&[(usize, T, usize)], usize),
        body: &BodySource<'_>,
        budget: &mut Budget,
        syntax: Syntax,
    ) -> Result<Self, ParseFailure> {
        let (tokens, capacity) = input;
        let mut depth = 0usize;
        let mut maximum = 0usize;
        for chunk in tokens.chunks(256) {
            budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
            for (_, token, _) in chunk {
                depth = depth.saturating_add_signed(token.depth_delta());
                maximum = maximum.max(depth);
            }
        }
        // Renewed against each exact generated table; see the data admission witness.
        let symbol = measure_symbol::<T>(&syntax);
        let (live, terminals, reductions) = match syntax {
            Syntax::Json => (5 * maximum + 8, 16usize, 8usize),
            Syntax::Hcl => ((4 * maximum + 4).max(6), 10, 5),
        };
        let stack_capacity = live.next_power_of_two().max(8);
        let diagnostic_capacity = (live + 1).next_power_of_two().max(8);
        let slots = (stack_capacity + stack_capacity / 2) * 2
            + (diagnostic_capacity + diagnostic_capacity / 2) * 3
            + terminals.next_power_of_two() * 3;
        bound("data parser stack slots", slots, budget.limits.layout_work)
            .map_err(ParseFailure::terminal)?;
        budget
            .charge(
                tokens
                    .len()
                    .saturating_mul(reductions + 1)
                    .saturating_add(reductions + slots + terminals * (live + reductions)),
            )
            .map_err(ParseFailure::terminal)?;
        let fixed_bytes = ((stack_capacity + stack_capacity / 2)
            * (symbol + 2 * std::mem::size_of::<usize>() + 2))
            .saturating_add((diagnostic_capacity + diagnostic_capacity / 2) * 3 * 2)
            .saturating_add(
                terminals.next_power_of_two() * 3 * std::mem::size_of::<String>() + terminals * 32,
            );
        // Stack storage is bounded by observed nesting; source-backed arenas have their own quota.
        let limit = budget
            .limits
            .source_bytes
            .saturating_mul(256)
            .saturating_add(fixed_bytes);
        let bytes = capacity
            .saturating_mul(std::mem::size_of::<(usize, T, usize)>())
            .saturating_add(body.adapters)
            .saturating_add(fixed_bytes)
            .saturating_add(budget.limits.source_bytes.saturating_mul(8));
        bound("data parser retained bytes", bytes, limit).map_err(ParseFailure::terminal)?;
        Ok(Self { bytes, limit })
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
                "data retained arena bytes",
                self.bytes.saturating_add(growth),
                self.limit,
            )
            .map_err(ParseFailure::terminal)?;
            budget
                .charge(values.len())
                .map_err(ParseFailure::terminal)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML data arena allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.bytes = self
                .bytes
                .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()));
            bound("data actual retained arena bytes", self.bytes, self.limit)
                .map_err(ParseFailure::terminal)?;
        }
        values.push(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::parser::{hcl, json},
    };

    #[test]
    fn should_admit_data_storage_and_work_before_generated_entry_or_vector_growth() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.layout_work = 1;
        let source = BodySource {
            document: &crate::plantuml::origin::EMPTY,
            range: 0..0,
            ownership: Default::default(),
            adapters: 0,
        };
        let tokens = vec![(0, json::Token::Null, 0)];
        // when
        let result = Arena::admit(
            (&tokens, tokens.capacity()),
            &source,
            &mut budget,
            Syntax::Json,
        );
        // then
        assert!(result.is_err());
        let mut budget = Budget::new(&RenderOptions::default());
        let mut arena = Arena {
            bytes: budget.limits.source_bytes * 256,
            limit: budget.limits.source_bytes * 256,
        };
        let mut values = Vec::<usize>::new();
        assert!(arena.push(&mut values, 1, &mut budget).is_err());
        assert_eq!(values.capacity(), 0);
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn should_measure_data_payloads_without_enlarging_the_common_failure_storage() {
        // given
        let json_symbol = measure_symbol::<json::Token<'_>>(&Syntax::Json);
        let hcl_symbol = measure_symbol::<hcl::Token<'_>>(&Syntax::Hcl);
        // when
        let failure = std::mem::size_of::<ParseFailure>();
        // then
        assert_eq!(json_symbol, 64);
        assert_eq!(hcl_symbol, 64);
        assert_eq!(failure, 48);
        assert_eq!(std::mem::size_of::<super::super::error::FailureKind>(), 32);
    }

    #[test]
    fn should_reject_the_first_byte_above_the_retained_source_arena_limit() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = 32;
        let mut source = BodySource {
            document: &crate::plantuml::origin::EMPTY,
            range: 0..0,
            ownership: Default::default(),
            adapters: 0,
        };
        let tokens = vec![(0, json::Token::Null, 0)];
        let arena = Arena::admit(
            (&tokens, tokens.capacity()),
            &source,
            &mut budget,
            Syntax::Json,
        )
        .unwrap();
        source.adapters = arena.limit - arena.bytes;
        // when
        let admitted = Arena::admit(
            (&tokens, tokens.capacity()),
            &source,
            &mut budget,
            Syntax::Json,
        )
        .unwrap();
        source.adapters += 1;
        let denied = Arena::admit(
            (&tokens, tokens.capacity()),
            &source,
            &mut budget,
            Syntax::Json,
        );
        // then
        assert_eq!(admitted.bytes, admitted.limit);
        assert!(matches!(
            denied,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Backend(_)),
                ..
            })
        ));
    }
}
