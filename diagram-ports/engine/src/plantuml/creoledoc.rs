//! Standalone line collection from MIT PSystemCreole/PSystemCreoleFactory.
//! Copyright 2009-2024 Arnaud Roques; exact sources and grant in provenance.
use super::{
    budget::{Budget, bound},
    parser::{context, error::ParseFailure},
};
use std::ops::Range;

pub(super) struct Document<'s> {
    pub source: &'s str,
    pub lines: Vec<DocLine>,
    pub paragraphs: Vec<Paragraph>,
}
pub(in crate::plantuml) struct DocLine {
    pub span: Range<usize>,
}
pub(in crate::plantuml) enum Paragraph {
    Text(Range<usize>),
    Blank(usize),
}
pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    lines: Vec<DocLine>,
    paragraphs: Vec<Paragraph>,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            lines: Vec::new(),
            paragraphs: Vec::new(),
        }
    }
    pub fn line(&mut self, span: Range<usize>) -> Result<usize, ParseFailure> {
        self.budget.item()?;
        bound("Creole document line bytes", span.len(), 16 * 1024)?;
        let index = self.lines.len();
        context::push(&mut self.lines, DocLine { span }, self.budget)?;
        Ok(index)
    }
    pub fn extend(
        &mut self,
        lines: Range<usize>,
        next: usize,
    ) -> Result<Range<usize>, ParseFailure> {
        self.budget.charge(1)?;
        bound(
            "Creole paragraph bytes",
            self.lines[next].span.end - self.lines[lines.start].span.start,
            16 * 1024,
        )?;
        Ok(lines.start..next + 1)
    }
    pub fn paragraph(&mut self, paragraph: Paragraph) -> Result<(), ParseFailure> {
        self.budget.item()?;
        context::push(&mut self.paragraphs, paragraph, self.budget)
    }
    pub fn finish(self, source: &str) -> Document<'_> {
        Document {
            source,
            lines: self.lines,
            paragraphs: self.paragraphs,
        }
    }
}
