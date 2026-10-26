//! Regex railroad syntax only; expressions are never executed or expanded.
use super::{
    budget::{Budget, bound},
    parser::{error::ParseFailure, railroad::Storage},
    railroad::{Diagram, Tile},
};
use std::ops::Range;

#[derive(Debug)]
pub(super) struct RegexDocument {
    pub exprs: Vec<RegexExpr>,
    pub root: usize,
}
#[derive(Debug)]
pub(super) struct RegexExpr {
    pub kind: Kind,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) enum Member {
    Scalar(char),
    Range(char, char),
    Escape(String),
}
#[derive(Debug)]
pub(super) enum Upper {
    Finite(usize),
    Unbounded,
}
#[derive(Debug)]
pub(super) enum Kind {
    Literal(String),
    Dot,
    Escape(String),
    Class {
        negated: bool,
        members: Vec<Member>,
        display: String,
    },
    Sequence(usize, usize),
    Choice(usize, usize),
    Group(usize, &'static str),
    Repeat {
        child: usize,
        min: usize,
        max: Upper,
        display: String,
    },
}
pub(super) struct Builder<'a> {
    pub storage: Storage,
    pub document: RegexDocument,
    pub budget: &'a mut Budget,
}
impl Builder<'_> {
    pub fn bounds(
        &mut self,
        min: usize,
        max: Upper,
    ) -> Result<(usize, Upper, String), ParseFailure> {
        self.budget.text(32)?;
        let display = match max {
            Upper::Finite(n) if min == n => format!("{{{min}}}"),
            Upper::Finite(n) => format!("{{{min},{n}}}"),
            Upper::Unbounded => format!("{{{min},}}"),
        };
        Ok((min, max, display))
    }
    pub fn node(&mut self, span: Range<usize>, kind: Kind) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        let id = self.document.exprs.len();
        self.storage.push(
            &mut self.document.exprs,
            RegexExpr { kind, span },
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
    pub fn member(
        &mut self,
        members: &mut Vec<Member>,
        member: Member,
    ) -> Result<(), ParseFailure> {
        self.budget.item()?;
        self.storage.push(members, member, self.budget)
    }
    pub fn range(&mut self, span: Range<usize>, a: char, b: char) -> Result<Member, ParseFailure> {
        if a > b {
            return Err(ParseFailure::syntax(
                span,
                "descending regex character range",
            ));
        }
        Ok(Member::Range(a, b))
    }
    pub fn class(
        &mut self,
        span: Range<usize>,
        negated: bool,
        members: Vec<Member>,
    ) -> Result<usize, ParseFailure> {
        let mut display = if negated {
            "not:".to_owned()
        } else {
            "one of:".to_owned()
        };
        for member in &members {
            self.budget.charge(1)?;
            let size = match member {
                Member::Scalar(c) => c.len_utf8() + 1,
                Member::Range(a, b) => a.len_utf8() + b.len_utf8() + 2,
                Member::Escape(s) => s.len() + 1,
            };
            bound("regex class label bytes", display.len() + size, 16 * 1024)?;
            self.budget.text(size)?;
            display.push('\n');
            match member {
                Member::Scalar(c) => display.push(*c),
                Member::Range(a, b) => {
                    display.push(*a);
                    display.push('-');
                    display.push(*b);
                }
                Member::Escape(s) => display.push_str(s),
            }
        }
        self.node(
            span,
            Kind::Class {
                negated,
                members,
                display,
            },
        )
    }
    pub fn repeat(
        &mut self,
        span: Range<usize>,
        child: usize,
        bounds: (usize, Upper, String),
    ) -> Result<usize, ParseFailure> {
        let (min, max, display) = bounds;
        if let Upper::Finite(end) = max {
            if min > end {
                return Err(ParseFailure::syntax(
                    span,
                    "regex repeat minimum exceeds maximum",
                ));
            }
            if end == 0 {
                return Err(ParseFailure::unsupported(
                    span,
                    "zero-only regex repetition {0} or {0,0}",
                ));
            }
        }
        self.node(
            span,
            Kind::Repeat {
                child,
                min,
                max,
                display,
            },
        )
    }
}
impl Diagram for RegexDocument {
    fn span(&self, id: usize) -> Range<usize> {
        self.exprs[id].span.clone()
    }
    fn rule_span(&self, _: usize) -> Range<usize> {
        self.exprs[self.root].span.clone()
    }
    fn len(&self) -> usize {
        self.exprs.len()
    }
    fn root_count(&self) -> usize {
        1
    }
    fn root(&self, _: usize) -> (&str, usize) {
        ("", self.root)
    }
    fn tile(&self, id: usize) -> Tile<'_> {
        match &self.exprs[id].kind {
            Kind::Literal(text) => Tile::Terminal(text),
            Kind::Dot => Tile::Reference("any character"),
            Kind::Escape(text) => Tile::Reference(text),
            Kind::Class {
                display,
                negated,
                members,
            } => {
                debug_assert!(!members.is_empty());
                debug_assert_eq!(*negated, display.starts_with("not:"));
                Tile::Class(display)
            }
            Kind::Sequence(a, b) => Tile::Sequence(*a, *b),
            Kind::Choice(a, b) => Tile::Choice(*a, *b),
            Kind::Group(child, kind) => Tile::Group(*child, kind),
            Kind::Repeat {
                child,
                min,
                max,
                display,
            } => match (min, max) {
                (0, Upper::Finite(1)) => Tile::Optional(*child),
                _ => Tile::Repeat(*child, *min, display),
            },
        }
    }
}
