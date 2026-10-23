//! Commit/ancestor semantics from MIT GitTextArea, Commit and CursorPosition.
//! Copyright 2009-2024 Arnaud Roques; grant and exact source traces in provenance.
use super::{
    budget::{Budget, bound},
    parser::error::ParseFailure,
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct GitHistory {
    pub commits: Vec<Commit>,
    pub parents: Vec<Parent>,
    pub source_lanes: Vec<usize>,
}
#[derive(Debug)]
pub(super) struct Commit {
    pub name: String,
    pub comment: Comment,
    pub position: Position,
}
#[derive(Debug)]
pub(super) struct Parent {
    pub child: usize,
    pub parent: usize,
    pub path: Vec<Position>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plantuml) struct Position {
    pub row: usize,
    pub column: usize,
}
impl Position {
    fn shift(self, delta: (isize, isize)) -> Option<Self> {
        Some(Self {
            row: self.row.checked_add_signed(delta.0)?,
            column: self.column.checked_add_signed(delta.1)?,
        })
    }
}
#[derive(Debug)]
pub(in crate::plantuml) enum Comment {
    Absent,
    Text(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plantuml) enum Kind {
    Pipe,
    Slash,
    Backslash,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) struct Cell {
    pub column: usize,
    pub kind: Kind,
    used: bool,
}
impl Cell {
    pub fn new(column: usize, kind: Kind) -> Self {
        Self {
            column,
            kind,
            used: false,
        }
    }
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Name {
    pub text: String,
    pub column: usize,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) struct RowInfo {
    pub base: usize,
    pub width: usize,
}
pub(in crate::plantuml) struct CommitRow {
    pub before: Vec<Cell>,
    pub after: Vec<Cell>,
    pub column: usize,
    pub name: Name,
    pub comment: Comment,
}
enum Marker {
    Absent,
    Commit {
        id: usize,
        column: usize,
        end: usize,
    },
}
struct GridRow {
    cells: Vec<Cell>,
    marker: Marker,
    info: RowInfo,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum At {
    Outside,
    Blank,
    Commit(usize),
    Connector(Kind),
}

pub(in crate::plantuml) struct Storage {
    bytes: usize,
}
impl Storage {
    pub fn new(bytes: usize) -> Self {
        Self { bytes }
    }
    fn admit(&self, additional: usize, budget: &mut Budget) -> Result<(), ParseFailure> {
        bound(
            "Git retained bytes",
            self.bytes.saturating_add(additional),
            budget.limits.source_bytes.saturating_mul(256),
        )?;
        budget.charge(1)?;
        Ok(())
    }
    fn retain(&mut self, additional: usize, budget: &mut Budget) -> Result<(), ParseFailure> {
        self.admit(additional, budget)?;
        self.bytes = self.bytes.saturating_add(additional);
        Ok(())
    }
    pub fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if values.len() == values.capacity() {
            let old = values.capacity();
            let capacity = old.saturating_mul(2).max(4);
            self.admit(capacity.saturating_mul(std::mem::size_of::<T>()), budget)?;
            budget.charge(values.len())?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(crate::RenderError::Backend(format!(
                        "Git allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.retain(
                (values.capacity() - old).saturating_mul(std::mem::size_of::<T>()),
                budget,
            )?;
        }
        budget.charge(1)?;
        values.push(value);
        Ok(())
    }
}

pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    storage: Storage,
    rows: Vec<GridRow>,
    commits: Vec<Commit>,
    names: BTreeMap<String, usize>,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, retained: usize) -> Self {
        Self {
            budget,
            storage: Storage::new(retained),
            rows: Vec::new(),
            commits: Vec::new(),
            names: BTreeMap::new(),
        }
    }
    pub fn cell(&mut self, mut cells: Vec<Cell>, cell: Cell) -> Result<Vec<Cell>, ParseFailure> {
        self.budget.item()?;
        self.storage.push(&mut cells, cell, self.budget)?;
        Ok(cells)
    }
    pub fn blank(&mut self, cells: Vec<Cell>, info: RowInfo) -> Result<(), ParseFailure> {
        self.budget.item()?;
        self.storage.push(
            &mut self.rows,
            GridRow {
                cells,
                marker: Marker::Absent,
                info,
            },
            self.budget,
        )
    }
    pub fn commit(&mut self, row: CommitRow, info: RowInfo) -> Result<(), ParseFailure> {
        let span = info.base..info.base + info.width;
        self.budget.charge(
            row.name
                .text
                .len()
                .saturating_mul(self.names.len().checked_ilog2().unwrap_or(0) as usize + 1)
                .saturating_mul(32),
        )?;
        if self.names.contains_key(&row.name.text) {
            return Err(ParseFailure::owned_syntax(
                span,
                format!("duplicate Git commit name {:?}", row.name.text),
            ));
        }
        self.budget.item()?;
        self.storage
            .retain(row.name.text.len().saturating_add(512), self.budget)?;
        let key = super::parser::git::copy(&row.name.text, span.clone(), self.budget)?;
        let id = self.commits.len();
        self.names.insert(key, id);
        let mut cells = row.before;
        for cell in row.after {
            self.storage.push(&mut cells, cell, self.budget)?;
        }
        let marker = Marker::Commit {
            id,
            column: row.column,
            end: row.name.column,
        };
        self.storage.push(
            &mut self.commits,
            Commit {
                name: row.name.text,
                comment: row.comment,
                position: Position {
                    row: self.rows.len(),
                    column: row.column,
                },
            },
            self.budget,
        )?;
        self.budget.item()?;
        self.storage.push(
            &mut self.rows,
            GridRow {
                cells,
                marker,
                info,
            },
            self.budget,
        )
    }
    fn at(&mut self, position: Position) -> Result<At, ParseFailure> {
        self.budget.charge(1)?;
        let Some(row) = self.rows.get(position.row) else {
            return Ok(At::Outside);
        };
        if let Marker::Commit { id, column, end } = row.marker {
            if position.column == column {
                return Ok(At::Commit(id));
            }
            if position.column >= end {
                return Ok(At::Outside);
            }
        }
        if position.column > row.info.width {
            return Ok(At::Outside);
        }
        self.budget
            .charge(row.cells.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        Ok(
            match row
                .cells
                .binary_search_by_key(&position.column, |cell| cell.column)
            {
                Ok(index) => At::Connector(row.cells[index].kind),
                Err(_) => At::Blank,
            },
        )
    }
    fn candidates(&mut self, position: Position) -> Result<Vec<Position>, ParseFailure> {
        let mut candidates = Vec::new();
        for (delta, expected) in [
            ((1, 0), Kind::Pipe),
            ((1, -1), Kind::Slash),
            ((1, 1), Kind::Backslash),
        ] {
            let Some(next) = position.shift(delta) else {
                continue;
            };
            let at = self.at(next)?;
            if at == At::Connector(expected) || (delta.1 == 0 && matches!(at, At::Commit(_))) {
                self.storage.push(&mut candidates, next, self.budget)?;
            }
        }
        if candidates.len() > 2 {
            return Err(self.failure(position, "octopus Git ancestry is unsupported"));
        }
        Ok(candidates)
    }
    fn next(&mut self, position: Position, kind: Kind) -> Result<Position, ParseFailure> {
        let right = match position.shift((0, 1)) {
            Some(next) => self.at(next)?,
            None => At::Outside,
        };
        if kind == Kind::Backslash && right != At::Blank {
            return Err(self.failure(position, "unsupported Git connector crossing"));
        }
        let mut found = None;
        for delta in match kind {
            Kind::Pipe => &[(1, 0), (1, -1), (1, 1)][..],
            Kind::Slash => &[(1, -1)][..],
            Kind::Backslash => &[(1, 1)][..],
        } {
            let Some(next) = position.shift(*delta) else {
                continue;
            };
            let at = self.at(next)?;
            let accepted = match (kind, delta.1, at) {
                (Kind::Pipe, 0, At::Commit(_) | At::Connector(Kind::Pipe)) => true,
                (_, -1, At::Connector(Kind::Slash)) | (_, 1, At::Connector(Kind::Backslash)) => {
                    right == At::Blank
                }
                (Kind::Slash, -1, At::Commit(_) | At::Connector(Kind::Pipe))
                | (Kind::Backslash, 1, At::Commit(_) | At::Connector(Kind::Pipe)) => true,
                _ => false,
            };
            if accepted {
                if found.is_some() {
                    return Err(self.failure(position, "ambiguous Git connector continuation"));
                }
                found = Some(next);
            }
        }
        found.ok_or_else(|| {
            self.failure(
                position,
                "dangling or unsupported Git connector continuation",
            )
        })
    }
    fn trace(&mut self, child: usize, start: Position) -> Result<Parent, ParseFailure> {
        let mut path = Vec::new();
        let mut visited = Vec::new();
        self.budget.item()?;
        self.storage
            .push(&mut path, self.commits[child].position, self.budget)?;
        let mut position = start;
        loop {
            self.budget.item()?;
            self.storage.push(&mut path, position, self.budget)?;
            match self.at(position)? {
                At::Commit(parent) => {
                    return Ok(Parent {
                        child,
                        parent,
                        path,
                    });
                }
                At::Connector(kind) => {
                    self.budget
                        .charge(visited.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
                    let state = (position, kind);
                    // Every admitted direction advances one row, so append preserves search order.
                    if visited.binary_search(&state).is_ok() {
                        return Err(self.failure(position, "cyclic Git connector walk"));
                    }
                    self.budget.item()?;
                    self.storage.push(&mut visited, state, self.budget)?;
                    let row = &mut self.rows[position.row];
                    let index = row
                        .cells
                        .binary_search_by_key(&position.column, |cell| cell.column)
                        .unwrap();
                    row.cells[index].used = true;
                    position = self.next(position, kind)?;
                }
                _ => {
                    return Err(
                        self.failure(position, "Git connector does not reach an older commit")
                    );
                }
            }
        }
    }
    fn failure(&self, position: Position, detail: &'static str) -> ParseFailure {
        let base = self.rows.get(position.row).map_or(0, |row| row.info.base);
        ParseFailure::syntax(
            base.saturating_add(position.column)..base.saturating_add(position.column + 1),
            detail,
        )
    }
    pub fn finish(mut self) -> Result<GitHistory, ParseFailure> {
        if self.commits.is_empty() {
            self.budget.charge(self.rows.len())?;
            let span = self
                .rows
                .iter()
                .find_map(|row| {
                    row.cells
                        .first()
                        .map(|cell| row.info.base + cell.column..row.info.base + cell.column + 1)
                })
                .unwrap_or_else(|| {
                    self.rows
                        .first()
                        .map_or(0..0, |row| row.info.base..row.info.base + row.info.width)
                });
            return Err(ParseFailure::syntax(
                span,
                "Git history requires at least one named commit",
            ));
        }
        let mut parents = Vec::new();
        for child in 0..self.commits.len() {
            let mut previous = None;
            for candidate in self.candidates(self.commits[child].position)? {
                let edge = self.trace(child, candidate)?;
                if previous == Some(edge.parent) {
                    return Err(self.failure(
                        candidate,
                        "duplicate Git parent through separate connectors",
                    ));
                }
                previous = Some(edge.parent);
                self.budget.item()?;
                self.storage.push(&mut parents, edge, self.budget)?;
            }
        }
        for row in &self.rows {
            for cell in &row.cells {
                self.budget.charge(1)?;
                if cell.used {
                    continue;
                }
                return Err(ParseFailure::syntax(
                    row.info.base + cell.column..row.info.base + cell.column + 1,
                    "unowned or dangling Git connector",
                ));
            }
        }
        let mut source_lanes = Vec::new();
        for commit in &self.commits {
            self.storage
                .push(&mut source_lanes, commit.position.column, self.budget)?;
        }
        self.budget.charge(
            source_lanes
                .len()
                .saturating_mul(source_lanes.len().checked_ilog2().unwrap_or(0) as usize + 1)
                .saturating_mul(16),
        )?;
        source_lanes.sort_unstable();
        source_lanes.dedup();
        Ok(GitHistory {
            commits: self.commits,
            parents,
            source_lanes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::parser::{error::FailureKind, test_support},
    };

    #[test]
    fn should_preserve_raw_rows_columns_comments_and_typed_older_ancestry() {
        // given
        let source = "@startgit\n\n  * top (release note)\n  |\n  * base\n@endgit";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model =
            test_support::parse(source, &mut budget, crate::plantuml::parser::git::parse).unwrap();
        // then
        assert_eq!(model.source_lanes, [2]);
        assert_eq!(model.commits[0].position, Position { row: 1, column: 2 });
        assert_eq!(model.commits[1].position, Position { row: 3, column: 2 });
        assert!(matches!(&model.commits[0].comment, Comment::Text(text) if text == "release note"));
        assert_eq!((model.parents[0].child, model.parents[0].parent), (0, 1));
        assert_eq!(
            model.parents[0].path,
            [
                Position { row: 1, column: 2 },
                Position { row: 2, column: 2 },
                Position { row: 3, column: 2 }
            ]
        );
    }

    #[test]
    fn should_store_sparse_long_rows_without_a_width_times_height_grid() {
        // given
        let source = format!(
            "@startgit\n{}* tip\n{}* base\n@endgit",
            " ".repeat(8192),
            "|\n".repeat(2000)
        );
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let result = test_support::parse(&source, &mut budget, crate::plantuml::parser::git::parse);
        // then
        assert!(
            matches!(result, Err(crate::RenderError::InvalidSource(ref detail)) if detail.contains("unowned or dangling Git connector"))
        );
        assert!(
            budget.work < source.len() * 128,
            "{} for {} bytes",
            budget.work,
            source.len()
        );
    }

    #[test]
    fn should_cancel_inside_sparse_parent_traversal_before_emitting_any_model() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut builder = Builder::new(&mut budget, 0);
        builder
            .commit(
                CommitRow {
                    before: Vec::new(),
                    after: Vec::new(),
                    column: 0,
                    name: Name {
                        text: "top".into(),
                        column: 2,
                    },
                    comment: Comment::Absent,
                },
                RowInfo { base: 0, width: 5 },
            )
            .unwrap();
        for row in 1..100 {
            builder
                .blank(
                    vec![Cell::new(0, Kind::Pipe)],
                    RowInfo {
                        base: row * 6,
                        width: 1,
                    },
                )
                .unwrap();
        }
        builder
            .commit(
                CommitRow {
                    before: Vec::new(),
                    after: Vec::new(),
                    column: 0,
                    name: Name {
                        text: "base".into(),
                        column: 2,
                    },
                    comment: Comment::Absent,
                },
                RowInfo {
                    base: 600,
                    width: 6,
                },
            )
            .unwrap();
        builder.budget.control.cancel_after_checkpoints(20);
        // when
        let result = builder.finish();
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: FailureKind::Terminal(crate::RenderError::Cancelled(_)),
                ..
            })
        ));
    }
}
