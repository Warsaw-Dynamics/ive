//! Ordered symbolic EBNF rules; source traces are recorded in provenance.
use super::{
    budget::Budget,
    parser::{error::ParseFailure, railroad::Storage},
    railroad::{Diagram, Tile},
};
use std::ops::Range;

#[derive(Debug)]
pub(super) struct EbnfDocument {
    pub rules: Vec<EbnfRule>,
    pub exprs: Vec<EbnfExpr>,
}
#[derive(Debug)]
pub(super) struct EbnfRule {
    pub name: String,
    pub expr: usize,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) struct EbnfExpr {
    pub kind: Kind,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) enum Kind {
    Empty,
    Terminal(String, char),
    Reference(String),
    Sequence(usize, usize),
    Choice(usize, usize),
    Group(usize),
    Optional(usize),
    Repeat(usize, usize),
}

pub(super) struct Builder<'a> {
    pub storage: Storage,
    pub document: EbnfDocument,
    pub budget: &'a mut Budget,
}
impl Builder<'_> {
    pub fn node(&mut self, span: Range<usize>, kind: Kind) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        let id = self.document.exprs.len();
        self.storage.push(
            &mut self.document.exprs,
            EbnfExpr { kind, span },
            self.budget,
        )?;
        Ok(id)
    }
    pub fn text(&mut self, text: &str) -> Result<String, ParseFailure> {
        self.budget.item()?;
        super::text::validate_label(text, 1)?;
        self.budget.text(text.len())?;
        Ok(text.to_owned())
    }
    pub fn rule(
        &mut self,
        span: Range<usize>,
        name: &str,
        expr: usize,
    ) -> Result<(), ParseFailure> {
        self.budget.item()?;
        let name = self.text(name)?;
        self.storage.push(
            &mut self.document.rules,
            EbnfRule { name, expr, span },
            self.budget,
        )
    }
}
impl Diagram for EbnfDocument {
    fn span(&self, id: usize) -> Range<usize> {
        self.exprs[id].span.clone()
    }
    fn rule_span(&self, index: usize) -> Range<usize> {
        self.rules[index].span.clone()
    }
    fn len(&self) -> usize {
        self.exprs.len()
    }
    fn root_count(&self) -> usize {
        self.rules.len()
    }
    fn root(&self, index: usize) -> (&str, usize) {
        (&self.rules[index].name, self.rules[index].expr)
    }
    fn tile(&self, id: usize) -> Tile<'_> {
        match &self.exprs[id].kind {
            Kind::Empty => Tile::Empty,
            Kind::Terminal(text, quote) => {
                debug_assert!(matches!(quote, '\'' | '"'));
                Tile::Terminal(text)
            }
            Kind::Reference(text) => Tile::Reference(text),
            Kind::Sequence(a, b) => Tile::Sequence(*a, *b),
            Kind::Choice(a, b) => Tile::Choice(*a, *b),
            Kind::Group(child) => Tile::Group(*child, ""),
            Kind::Optional(child) => Tile::Optional(*child),
            Kind::Repeat(child, min) => Tile::Repeat(*child, *min, ""),
        }
    }
}
