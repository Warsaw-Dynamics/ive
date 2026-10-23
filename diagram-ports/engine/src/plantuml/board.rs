//! Board stages/ancestry/DFS columns translated from authorized PlantUML MIT board sources.
//! Copyright 2009-2024 Arnaud Roques; source hashes and MIT grant in provenance/.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

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

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Board, RenderError> {
    let mut model = Board {
        cards: Vec::new(),
        activities: Vec::new(),
    };
    let mut ancestors: Vec<usize> = Vec::new();
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }
        if super::style::is_preamble(text)
            || matches!(
                text.split_whitespace().next(),
                Some("newpage" | "page" | "sprite" | "hide" | "show" | "scale" | "url")
            )
            || text.starts_with(['!', '@'])
        {
            return Err(unsupported(line.number, text));
        }
        let stage = text.bytes().take_while(|&ch| ch == b'+').count();
        budget.depth(stage + 1)?;
        let name = text[stage..].trim_start();
        bound("label bytes", name.len(), 16 * 1024)?;
        if name.is_empty() {
            return Err(invalid(line.number, format!("empty board card {text:?}")));
        }
        if stage == 0 {
            ancestors.clear();
        }
        while let Some(&id) = ancestors.last() {
            budget.charge(1)?;
            if model.cards[id].stage < stage {
                break;
            }
            ancestors.pop();
        }
        let parent = if stage == 0 {
            Parent::Root
        } else {
            Parent::Node(*ancestors.last().ok_or_else(|| {
                invalid(
                    line.number,
                    format!("board release requires an activity: {text:?}"),
                )
            })?)
        };
        budget.item()?;
        budget.text(name.len())?;
        let id = model.cards.len();
        model.cards.push(Card {
            parent,
            stage,
            name: name.into(),
            children: Vec::new(),
        });
        if let Parent::Node(parent) = parent {
            model.cards[parent].children.push(id);
        } else {
            model.activities.push(id);
        }
        ancestors.push(id);
    }
    if model.cards.is_empty() {
        return Err(invalid(1, "board requires an activity"));
    }
    Ok(model)
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
