//! Board stages/ancestry/DFS columns translated from authorized PlantUML MIT board sources.
//! Copyright 2009-2024 Arnaud Roques; source hashes and MIT grant in provenance/.
use super::{
    budget::{Budget, bound},
    parser::{error::ParseFailure, outline::Arena},
};
use std::ops::Range;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Parent {
    Root,
    Node(usize),
}
pub(super) struct Card {
    pub parent: Parent,
    pub stage: usize,
    pub name: String,
    pub children: Vec<usize>,
}
pub(super) struct Board {
    pub cards: Vec<Card>,
    pub activities: Vec<usize>,
}

pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    arena: Arena,
    model: Board,
    ancestors: Vec<usize>,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, arena: Arena) -> Self {
        Self {
            budget,
            arena,
            model: Board {
                cards: Vec::new(),
                activities: Vec::new(),
            },
            ancestors: Vec::new(),
        }
    }
    pub fn card(
        &mut self,
        stage: usize,
        name: String,
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget
            .depth(stage + 1)
            .map_err(ParseFailure::terminal)?;
        bound("label bytes", name.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        if name.is_empty() {
            return Err(ParseFailure::syntax(span, "empty board card"));
        }
        if stage == 0 {
            self.ancestors.clear();
        }
        while let Some(&id) = self.ancestors.last() {
            self.budget.charge(1).map_err(ParseFailure::terminal)?;
            if self.model.cards[id].stage < stage {
                break;
            }
            self.ancestors.pop();
        }
        let parent =
            if stage == 0 {
                Parent::Root
            } else {
                Parent::Node(*self.ancestors.last().ok_or_else(|| {
                    ParseFailure::syntax(span, "board release requires an activity")
                })?)
            };
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len())
            .map_err(ParseFailure::terminal)?;
        let id = self.model.cards.len();
        self.arena.push(
            &mut self.model.cards,
            Card {
                parent,
                stage,
                name,
                children: Vec::new(),
            },
            self.budget,
        )?;
        if let Parent::Node(parent) = parent {
            self.arena
                .push(&mut self.model.cards[parent].children, id, self.budget)?;
        } else {
            self.arena
                .push(&mut self.model.activities, id, self.budget)?;
        }
        self.arena.push(&mut self.ancestors, id, self.budget)
    }
    pub fn finish(self, span: Range<usize>) -> Result<Board, ParseFailure> {
        self.budget.checkpoint().map_err(ParseFailure::terminal)?;
        if self.model.cards.is_empty() {
            return Err(ParseFailure::syntax(span, "board requires an activity"));
        }
        Ok(self.model)
    }
}
#[cfg(test)]
use super::source::Line;
#[cfg(test)]
pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Board, crate::RenderError> {
    super::parser::outline::test_parse(lines, "board", budget, super::parser::board::parse)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_reject_oversized_board_label_during_model_construction() {
        // given
        let lines = vec![Line {
            number: 3,
            text: "x".repeat(16_385),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let result = parse(&lines, &mut budget);
        // then
        assert!(result.is_err(), "oversized card retained before layout");
    }
}
