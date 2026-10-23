//! MIT PlantUML mindmap/ and wbs/: ordered marker levels, sides and parent policies.
use super::{
    budget::Budget,
    parser::{error::ParseFailure, outline::Arena, tree::Marker},
};
use std::ops::Range;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Parent {
    Root,
    Node(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outline {
    Box,
    Boxless,
    Pseudo,
}
#[derive(Debug)]
pub(super) struct Node {
    pub parent: Parent,
    pub side: Side,
    pub depth: usize,
    pub label: String,
    pub alias: String,
    pub outline: Outline,
}
pub(super) struct Mindmap {
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
}
pub(super) struct Wbs {
    pub nodes: Vec<Node>,
}

pub(in crate::plantuml) struct MindmapBuilder<'b> {
    pub budget: &'b mut Budget,
    arena: Arena,
    nodes: Vec<Node>,
    roots: Vec<usize>,
    stacks: [Vec<usize>; 2],
    first: String,
    multiplier: usize,
    pub side: Side,
}
pub(in crate::plantuml) struct WbsBuilder<'b> {
    pub budget: &'b mut Budget,
    arena: Arena,
    nodes: Vec<Node>,
    stack: Vec<usize>,
    first: String,
}
impl<'b> MindmapBuilder<'b> {
    pub fn new(budget: &'b mut Budget, arena: Arena) -> Self {
        Self {
            budget,
            arena,
            nodes: Vec::new(),
            roots: Vec::new(),
            stacks: [Vec::new(), Vec::new()],
            first: String::new(),
            multiplier: 0,
            side: Side::Right,
        }
    }
    pub fn set_side(&mut self, side: Side) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.side = side;
        Ok(())
    }
    pub fn node(&mut self, mut marker: Marker, span: Range<usize>) -> Result<(), ParseFailure> {
        let raw = if marker.prefix.contains(['+', '-']) {
            marker.side = if marker.prefix.contains('-') {
                Side::Left
            } else {
                Side::Right
            };
            marker.prefix.len() - 1
        } else {
            marker.side = if marker.prefix == "0" {
                Side::Right
            } else {
                self.side
            };
            smart_level(
                &marker.prefix,
                &mut self.first,
                Smart::Mindmap,
                span.clone(),
                self.budget,
            )?
        };
        let level = if self.nodes.is_empty() { 0 } else { raw };
        let depth = if level == 0 {
            0
        } else {
            if self.multiplier == 0 {
                self.multiplier = level;
            }
            if level % self.multiplier != 0 {
                return Err(ParseFailure::syntax(
                    span,
                    "mindmap Bad indentation: level is not a multiple of first child depth",
                ));
            }
            level / self.multiplier
        };
        self.budget
            .depth(depth + 1)
            .map_err(ParseFailure::terminal)?;
        let index = self.nodes.len();
        let branch = usize::from(marker.side == Side::Right);
        let parent = if depth == 0 {
            self.arena.push(&mut self.roots, index, self.budget)?;
            self.stacks[0].clear();
            self.stacks[1].clear();
            self.arena.push(&mut self.stacks[0], index, self.budget)?;
            self.arena.push(&mut self.stacks[1], index, self.budget)?;
            self.multiplier = 0;
            Parent::Root
        } else {
            find_parent(&mut self.stacks[branch], depth, span.clone())?
        };
        if depth > 0 {
            self.arena
                .push(&mut self.stacks[branch], index, self.budget)?;
        }
        let node = make_node(marker, (parent, depth), span, self.budget)?;
        self.arena.push(&mut self.nodes, node, self.budget)
    }
    pub fn finish(self, span: Range<usize>) -> Result<Mindmap, ParseFailure> {
        self.budget.checkpoint().map_err(ParseFailure::terminal)?;
        if self.nodes.is_empty() {
            return Err(ParseFailure::syntax(span, "mindmap has no root"));
        }
        Ok(Mindmap {
            nodes: self.nodes,
            roots: self.roots,
        })
    }
}
impl<'b> WbsBuilder<'b> {
    pub fn new(budget: &'b mut Budget, arena: Arena) -> Self {
        Self {
            budget,
            arena,
            nodes: Vec::new(),
            stack: Vec::new(),
            first: String::new(),
        }
    }
    pub fn node(&mut self, mut marker: Marker, span: Range<usize>) -> Result<(), ParseFailure> {
        let depth = if self.nodes.is_empty() {
            self.budget
                .charge(marker.prefix.len())
                .map_err(ParseFailure::terminal)?;
            self.first = marker.prefix.clone();
            0
        } else {
            smart_level(
                &marker.prefix,
                &mut self.first,
                Smart::Wbs,
                span.clone(),
                self.budget,
            )?
        };
        self.budget
            .depth(depth + 1)
            .map_err(ParseFailure::terminal)?;
        let parent = if self.nodes.is_empty() {
            Parent::Root
        } else {
            if depth == 0 {
                return Err(ParseFailure::syntax(span, "WBS only accepts one root"));
            }
            find_parent(&mut self.stack, depth, span.clone())?
        };
        if marker.label.is_empty() {
            if marker.outline == Outline::Boxless {
                marker.outline = Outline::Pseudo;
            } else {
                return Err(ParseFailure::syntax(span, "missing WBS node label"));
            }
        }
        let node = make_node(marker, (parent, depth), span, self.budget)?;
        self.arena
            .push(&mut self.stack, self.nodes.len(), self.budget)?;
        self.arena.push(&mut self.nodes, node, self.budget)
    }
    pub fn finish(self, span: Range<usize>) -> Result<Wbs, ParseFailure> {
        self.budget.checkpoint().map_err(ParseFailure::terminal)?;
        if self.nodes.is_empty() {
            return Err(ParseFailure::syntax(span, "WBS has no root"));
        }
        Ok(Wbs { nodes: self.nodes })
    }
}
fn find_parent(
    stack: &mut Vec<usize>,
    depth: usize,
    span: Range<usize>,
) -> Result<Parent, ParseFailure> {
    if depth > stack.len() {
        return Err(ParseFailure::syntax(span, "invalid tree depth jump"));
    }
    stack.truncate(depth);
    stack
        .last()
        .copied()
        .map(Parent::Node)
        .ok_or_else(|| ParseFailure::syntax(span, "tree parent is missing"))
}
fn make_node(
    marker: Marker,
    position: (Parent, usize),
    span: Range<usize>,
    budget: &mut Budget,
) -> Result<Node, ParseFailure> {
    budget.item().map_err(ParseFailure::terminal)?;
    budget
        .text(marker.label.len() + marker.alias.len())
        .map_err(ParseFailure::terminal)?;
    if marker.label.contains("<<") {
        return Err(ParseFailure::unsupported(span, "tree stereotypes"));
    }
    // Shared label capability validation remains separate from marker syntax.
    super::text::validate_label(&marker.label, 1).map_err(ParseFailure::terminal)?;
    Ok(Node {
        parent: position.0,
        depth: position.1,
        side: marker.side,
        outline: marker.outline,
        label: marker.label,
        alias: marker.alias,
    })
}
enum Smart {
    Mindmap,
    Wbs,
}
fn smart_level(
    prefix: &str,
    first: &mut String,
    policy: Smart,
    span: Range<usize>,
    budget: &mut Budget,
) -> Result<usize, ParseFailure> {
    budget
        .charge(prefix.len())
        .map_err(ParseFailure::terminal)?;
    if first.is_empty() {
        *first = prefix.to_owned();
    }
    let normalized = prefix.replace('\t', " ");
    let kind = if matches!(policy, Smart::Mindmap) && prefix.ends_with("**") {
        normalized.trim()
    } else {
        &normalized
    };
    let Some(_) = kind.split_once(' ') else {
        return Ok(kind.len() - 1);
    };
    if kind.ends_with(first.as_str()) {
        return Ok(kind.len() - first.len());
    }
    if kind.trim().len() == 1 {
        return Ok(kind.len() - 1);
    }
    if kind.starts_with(first.as_str()) {
        return Ok(kind.len() - first.len());
    }
    Err(ParseFailure::syntax(
        span,
        "unsupported tree indentation prefix relative to first node",
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    fn lines(source: &str) -> String {
        source.to_owned()
    }

    #[test]
    fn should_resolve_side_specific_mindmap_parents_and_depths() {
        // given
        let source = lines(
            "* Root\nleft side\n*** Left\nright side\n*** Right\nleft side\n***** Left child\nright side\n***** Right child\n* Next\n*** Next child",
        );
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = crate::plantuml::parser::test_support::body(
            (&source, "mindmap"),
            &mut budget,
            crate::plantuml::parser::tree::parse_mindmap,
        )
        .unwrap();
        // then
        assert_eq!(model.roots, [0, 5]);
        assert_eq!(
            model.nodes.iter().map(|n| n.parent).collect::<Vec<_>>(),
            [
                Parent::Root,
                Parent::Node(0),
                Parent::Node(0),
                Parent::Node(1),
                Parent::Node(2),
                Parent::Root,
                Parent::Node(5)
            ]
        );
        assert_eq!(
            model.nodes.iter().map(|n| n.depth).collect::<Vec<_>>(),
            [0, 1, 1, 2, 2, 0, 1]
        );
    }

    #[test]
    fn should_use_wbs_global_chain_across_sides_and_retain_aliases() {
        // given
        let source = lines(
            "*** Root\n**(left) Left\n---> Right child\n***< Left sibling\n****_\n***** Leaf\n** \"Other display\" as Other",
        );
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = crate::plantuml::parser::test_support::body(
            (&source, "wbs"),
            &mut budget,
            crate::plantuml::parser::tree::parse_wbs,
        )
        .unwrap();
        // then
        assert_eq!(
            model.nodes.iter().map(|n| n.parent).collect::<Vec<_>>(),
            [
                Parent::Root,
                Parent::Node(0),
                Parent::Node(1),
                Parent::Node(1),
                Parent::Node(3),
                Parent::Node(4),
                Parent::Node(0)
            ]
        );
        assert_eq!(model.nodes[2].side, Side::Right);
        assert_eq!(model.nodes[3].side, Side::Left);
        assert_eq!(model.nodes[4].outline, Outline::Pseudo);
        assert_eq!(model.nodes[1].alias, "left");
        assert_eq!(model.nodes[6].alias, "Other");
        assert_eq!(model.nodes[6].label, "Other display");
    }

    #[test]
    fn should_reject_wbs_smart_depth_that_mindmap_normalizes() {
        // given
        let source = lines("* Root\n*** Child\n***** Grandchild");
        // when
        let mindmap = crate::plantuml::parser::test_support::body(
            (&source, "mindmap"),
            &mut Budget::new(&RenderOptions::default()),
            crate::plantuml::parser::tree::parse_mindmap,
        );
        let wbs = crate::plantuml::parser::test_support::body(
            (&source, "wbs"),
            &mut Budget::new(&RenderOptions::default()),
            crate::plantuml::parser::tree::parse_wbs,
        );
        // then
        assert!(mindmap.is_ok());
        assert!(wbs.is_err());
    }
}
