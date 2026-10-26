//! Hierarchical wire semantics from authorized MIT WBlock/WireDiagram and finite arrow types.
//! Copyright 2009–2024 Arnaud Roques; MIT grant and exact source traces in provenance/.
use super::{
    budget::{Budget, bound},
    parser::{context, error::ParseFailure},
    style::Color,
};
use crate::RenderError;
use std::{collections::BTreeMap, ops::Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Orientation {
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    None,
    Forward,
    Reverse,
    Both,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum LinkType {
    Normal,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Style {
    pub orientation: Orientation,
    pub direction: Direction,
    pub kind: LinkType,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum WireColor {
    Default,
    Rgb(Color),
}
impl WireColor {
    pub fn value(self) -> Color {
        match self {
            Self::Default => Color(0x334155),
            Self::Rgb(c) => c,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Size {
    Auto,
    Forced(usize, usize),
}
pub(super) enum Placement {
    Child(usize),
    Column,
}
pub(super) struct Block {
    pub parent: usize,
    pub depth: usize,
    pub local: String,
    pub key: String,
    pub size: Size,
    pub color: WireColor,
    pub placements: Vec<Placement>,
    pub children: BTreeMap<String, usize>,
    pub span: Range<usize>,
}
pub(super) enum Anchor {
    Ordered(usize),
}
pub(super) struct Link {
    pub from: usize,
    pub to: usize,
    pub anchor: Anchor,
    pub style: Style,
    pub label: String,
    pub color: WireColor,
    pub span: Range<usize>,
}
pub(super) struct Wire {
    pub blocks: Vec<Block>,
    pub links: Vec<Link>,
}
pub(in crate::plantuml) struct Header {
    pub name: String,
    pub size: Size,
    pub color: WireColor,
    pub span: Range<usize>,
}
pub(in crate::plantuml) struct LinkInput {
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub style: Style,
    pub label: String,
    pub color: WireColor,
    pub span: Range<usize>,
}
pub(in crate::plantuml) struct Scope {
    parent: usize,
}
pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    blocks: Vec<Block>,
    pending: Vec<LinkInput>,
    owner: usize,
    last: usize,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, span: Range<usize>) -> Result<Self, ParseFailure> {
        budget.item()?;
        let mut blocks = Vec::new();
        context::push(
            &mut blocks,
            Block {
                parent: 0,
                depth: 0,
                local: String::new(),
                key: String::new(),
                size: Size::Auto,
                color: WireColor::Default,
                placements: Vec::new(),
                children: BTreeMap::new(),
                span,
            },
            budget,
        )?;
        Ok(Self {
            budget,
            blocks,
            pending: Vec::new(),
            owner: 0,
            last: 0,
        })
    }
    pub fn block(&mut self, input: Header) -> Result<(), ParseFailure> {
        self.budget.depth(self.blocks[self.owner].depth + 1)?;
        self.budget.charge(input.name.len().saturating_mul(
            8 * (self.blocks[self.owner].children.len() + 1).ilog2() as usize + 8,
        ))?;
        if self.blocks[self.owner].children.contains_key(&input.name) {
            return Err(ParseFailure::unsupported(
                input.span,
                "duplicate wire sibling name",
            ));
        }
        let parent = &self.blocks[self.owner];
        self.budget
            .text(parent.key.len() + input.name.len() * 2 + 2)?;
        let key = if parent.key.is_empty() {
            input.name.clone()
        } else {
            format!("{}.{}", parent.key, input.name)
        };
        let id = self.blocks.len();
        self.budget.item()?;
        let node = Block {
            parent: self.owner,
            depth: parent.depth + 1,
            local: input.name.clone(),
            key,
            size: input.size,
            color: input.color,
            placements: Vec::new(),
            children: BTreeMap::new(),
            span: input.span,
        };
        context::push(&mut self.blocks, node, self.budget)?;
        self.blocks[self.owner].children.insert(input.name, id);
        context::push(
            &mut self.blocks[self.owner].placements,
            Placement::Child(id),
            self.budget,
        )?;
        self.last = id;
        Ok(())
    }
    pub fn enter(&mut self, span: Range<usize>) -> Result<Scope, ParseFailure> {
        self.budget.charge(1)?;
        if self.last == 0 || self.blocks[self.last].parent != self.owner {
            return Err(ParseFailure::unsupported(
                span,
                "wire indentation requires a preceding block",
            ));
        }
        self.budget.depth(self.blocks[self.last].depth + 1)?;
        let scope = Scope { parent: self.owner };
        self.owner = self.last;
        Ok(scope)
    }
    pub fn leave(&mut self, scope: Scope) -> Result<(), ParseFailure> {
        self.budget.charge(1)?;
        self.owner = scope.parent;
        self.last = 0;
        Ok(())
    }
    pub fn column(&mut self) -> Result<(), ParseFailure> {
        self.budget.item()?;
        context::push(
            &mut self.blocks[self.owner].placements,
            Placement::Column,
            self.budget,
        )?;
        self.last = 0;
        Ok(())
    }
    pub fn path(
        &mut self,
        mut parts: Vec<String>,
        part: String,
    ) -> Result<Vec<String>, ParseFailure> {
        self.budget.item()?;
        bound(
            "wire endpoint segments",
            parts.len() + 1,
            self.budget.limits.nesting_depth,
        )?;
        context::push(&mut parts, part, self.budget)?;
        Ok(parts)
    }
    pub fn link(&mut self, input: LinkInput) -> Result<(), ParseFailure> {
        if self.owner != 0 {
            return Err(ParseFailure::unsupported(
                input.span,
                "wire links must use root-level dotted endpoints",
            ));
        }
        self.budget.item()?;
        context::push(&mut self.pending, input, self.budget)?;
        self.last = 0;
        Ok(())
    }
    pub fn finish(self, span: Range<usize>) -> Result<Wire, ParseFailure> {
        self.budget.charge(1)?;
        if self.blocks.len() == 1 {
            return Err(ParseFailure::unsupported(
                span,
                "wire requires at least one block",
            ));
        }
        let mut horizontal = Vec::new();
        let mut vertical = Vec::new();
        for _ in &self.blocks {
            context::push(&mut horizontal, 0usize, self.budget)?;
            context::push(&mut vertical, 0usize, self.budget)?;
        }
        let mut links = Vec::new();
        for input in self.pending {
            let from = lookup(&self.blocks, &input.from, (&input.span, self.budget))?;
            let to = lookup(&self.blocks, &input.to, (&input.span, self.budget))?;
            let slots = if input.style.orientation == Orientation::Horizontal {
                &mut horizontal
            } else {
                &mut vertical
            };
            let slot = slots[from];
            slots[from] += 1;
            context::push(
                &mut links,
                Link {
                    from,
                    to,
                    anchor: Anchor::Ordered(slot),
                    style: input.style,
                    label: input.label,
                    color: input.color,
                    span: input.span,
                },
                self.budget,
            )?;
        }
        Ok(Wire {
            blocks: self.blocks,
            links,
        })
    }
}
fn lookup(
    blocks: &[Block],
    path: &[String],
    site: (&Range<usize>, &mut Budget),
) -> Result<usize, ParseFailure> {
    let (span, budget) = site;
    let mut owner = 0;
    for part in path {
        budget.charge(
            part.len()
                .saturating_mul(8 * (blocks[owner].children.len() + 1).ilog2() as usize + 8),
        )?;
        owner = blocks[owner]
            .children
            .get(part.as_str())
            .copied()
            .ok_or_else(|| {
                ParseFailure::unsupported(span.clone(), "unknown wire dotted endpoint")
            })?;
    }
    Ok(owner)
}
pub(in crate::plantuml) fn size(
    value: (usize, usize),
    span: Range<usize>,
) -> Result<Size, ParseFailure> {
    if value.0 == 0 || value.1 == 0 || value.0 > 100000 || value.1 > 100000 {
        return Err(ParseFailure::unsupported(
            span,
            "wire forced dimensions require 1..100000",
        ));
    }
    Ok(Size::Forced(value.0, value.1))
}
pub(super) fn failure(span: Range<usize>, message: String) -> ParseFailure {
    ParseFailure {
        span,
        kind: super::parser::error::FailureKind::Unsupported(message.into()),
    }
}
pub(super) fn overflow() -> RenderError {
    RenderError::Backend("wire retained storage arithmetic overflow".into())
}
