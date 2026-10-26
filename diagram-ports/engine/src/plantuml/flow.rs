//! Immutable directional tiles/side paths, traced to MIT FlowDiagram/TilesField.
use super::{
    budget::{Budget, bound},
    parser::{
        error::{FailureKind, ParseFailure},
        flow::Storage,
    },
    style::{Color, Font, Preamble},
};
use std::{cmp::Ordering, ops::Range};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cardinal {
    North,
    South,
    East,
    West,
}
impl Cardinal {
    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::North => "n",
            Self::South => "s",
            Self::East => "e",
            Self::West => "w",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Cell {
    pub row: i64,
    pub column: i64,
}
impl Cell {
    fn moved(self, direction: Cardinal) -> Result<Self, ParseFailure> {
        let (row, column) = match direction {
            Cardinal::North => (self.row.checked_sub(1), Some(self.column)),
            Cardinal::South => (self.row.checked_add(1), Some(self.column)),
            Cardinal::East => (Some(self.row), self.column.checked_add(1)),
            Cardinal::West => (Some(self.row), self.column.checked_sub(1)),
        };
        let (Some(row), Some(column)) = (row, column) else {
            return Err(ParseFailure::syntax(0..0, "flow coordinate overflow"));
        };
        bound("flow grid row", row.unsigned_abs() as usize, 1_000_000)?;
        bound(
            "flow grid column",
            column.unsigned_abs() as usize,
            1_000_000,
        )?;
        Ok(Self { row, column })
    }
}
#[derive(Debug)]
pub(super) struct FlowTiles {
    pub tiles: Vec<Tile>,
    pub placements: Vec<Placement>,
    pub paths: Vec<Path>,
}
#[derive(Debug)]
pub(super) struct Tile {
    pub id: String,
    pub label: String,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) struct Placement {
    pub cell: Cell,
    pub declared: Cardinal,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Endpoint {
    pub tile: usize,
    pub side: Cardinal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PathKind {
    Implicit,
    Explicit,
}
#[derive(Debug)]
pub(super) struct Path {
    pub from: Endpoint,
    pub to: Endpoint,
    pub kind: PathKind,
    pub span: Range<usize>,
}
enum Cursor {
    Empty,
    Tile(usize),
}
enum Lookup {
    Found(usize),
    Vacant(usize),
}
pub(super) struct Builder<'a> {
    pub model: FlowTiles,
    pub budget: &'a mut Budget,
    pub storage: Storage,
    names: Vec<usize>,
    occupied: Vec<(Cell, usize)>,
    last: Cursor,
}
impl<'a> Builder<'a> {
    pub fn new(storage: Storage, budget: &'a mut Budget) -> Self {
        Self {
            model: FlowTiles {
                tiles: Vec::new(),
                placements: Vec::new(),
                paths: Vec::new(),
            },
            budget,
            storage,
            names: Vec::new(),
            occupied: Vec::new(),
            last: Cursor::Empty,
        }
    }
    pub fn direction(&mut self, value: &str, span: Range<usize>) -> Result<Cardinal, ParseFailure> {
        self.budget.charge(value.len())?;
        match value {
            "n" => Ok(Cardinal::North),
            "s" => Ok(Cardinal::South),
            "e" => Ok(Cardinal::East),
            "w" => Ok(Cardinal::West),
            _ => Err(unsupported(
                self.budget,
                span,
                ("flow direction must be n/s/e/w", value),
            )),
        }
    }
    fn locate_name(&mut self, name: &str) -> Result<Lookup, ParseFailure> {
        let (mut left, mut right) = (0, self.names.len());
        while left < right {
            let middle = left + (right - left) / 2;
            let existing = self.model.tiles[self.names[middle]].id.as_str();
            self.budget.charge(existing.len().min(name.len()) + 1)?;
            match existing.cmp(name) {
                Ordering::Less => left = middle + 1,
                Ordering::Greater => right = middle,
                Ordering::Equal => return Ok(Lookup::Found(self.names[middle])),
            }
        }
        Ok(Lookup::Vacant(left))
    }
    fn position(&mut self, cell: Cell) -> Result<Lookup, ParseFailure> {
        self.budget
            .charge(self.occupied.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        Ok(
            match self.occupied.binary_search_by_key(&cell, |entry| entry.0) {
                Ok(index) => Lookup::Found(self.occupied[index].1),
                Err(index) => Lookup::Vacant(index),
            },
        )
    }
    pub fn tile(
        &mut self,
        span: Range<usize>,
        direction: Cardinal,
        raw: (&str, &str),
    ) -> Result<(), ParseFailure> {
        let (name, label) = raw;
        let Lookup::Vacant(name_at) = self.locate_name(name)? else {
            return Err(unsupported(
                self.budget,
                span,
                ("duplicate flow tile ID", name),
            ));
        };
        let cell = match self.last {
            Cursor::Empty => Cell { row: 0, column: 0 },
            Cursor::Tile(id) => self.model.placements[id].cell.moved(direction)?,
        };
        let Lookup::Vacant(cell_at) = self.position(cell)? else {
            return Err(unsupported(
                self.budget,
                span,
                (
                    "flow placement collision requires unimplemented source east-shift semantics",
                    name,
                ),
            ));
        };
        super::text::validate_label(label, 1)?;
        self.budget.item()?;
        self.budget.text(name.len() + label.len())?;
        let id = self.model.tiles.len();
        self.storage.push(
            &mut self.model.tiles,
            Tile {
                id: name.to_owned(),
                label: label.to_owned(),
                span: span.clone(),
            },
            self.budget,
        )?;
        self.budget.item()?;
        self.storage.push(
            &mut self.model.placements,
            Placement {
                cell,
                declared: direction,
            },
            self.budget,
        )?;
        self.storage
            .insert(&mut self.names, name_at, id, self.budget)?;
        self.storage
            .insert(&mut self.occupied, cell_at, (cell, id), self.budget)?;
        if let Cursor::Tile(from) = self.last {
            self.path(span, direction, (from, id), PathKind::Implicit)?;
        }
        self.last = Cursor::Tile(id);
        Ok(())
    }
    pub fn link(
        &mut self,
        span: Range<usize>,
        direction: Cardinal,
        name: &str,
    ) -> Result<(), ParseFailure> {
        let Cursor::Tile(from) = self.last else {
            return Err(unsupported(
                self.budget,
                span,
                ("flow link requires a previously declared tile", name),
            ));
        };
        let Lookup::Found(to) = self.locate_name(name)? else {
            return Err(unsupported(
                self.budget,
                span,
                ("unknown flow link target ID", name),
            ));
        };
        self.path(span, direction, (from, to), PathKind::Explicit)
    }
    fn path(
        &mut self,
        span: Range<usize>,
        direction: Cardinal,
        ends: (usize, usize),
        kind: PathKind,
    ) -> Result<(), ParseFailure> {
        let (from, to) = ends;
        self.budget.charge(1)?;
        if from == to {
            return Err(unsupported(
                self.budget,
                span,
                (
                    "self flow link would cross its own tile",
                    self.model.tiles[to].id.as_str(),
                ),
            ));
        }
        let (a, b) = (
            self.model.placements[from].cell,
            self.model.placements[to].cell,
        );
        let adjoining = a.moved(direction)? == b;
        let west = a.row == b.row && b.column < a.column && direction == Cardinal::West;
        if !adjoining && !west {
            return Err(unsupported(
                self.budget,
                span,
                (
                    "unrouteable flow link: expected adjacent opposing ports or clear west-to-east sides",
                    self.model.tiles[to].id.as_str(),
                ),
            ));
        }
        if west {
            self.budget
                .charge(self.occupied.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
            let start = self.occupied.partition_point(|entry| entry.0 <= b);
            if let Some(&(cell, _)) = self.occupied.get(start) {
                self.budget.charge(1)?;
                if cell.row == a.row && cell.column < a.column {
                    return Err(unsupported(
                        self.budget,
                        span,
                        (
                            "flow link would cross an intervening tile",
                            self.model.tiles[to].id.as_str(),
                        ),
                    ));
                }
            }
        }
        self.budget.item()?;
        self.storage.push(
            &mut self.model.paths,
            Path {
                from: Endpoint {
                    tile: from,
                    side: direction,
                },
                to: Endpoint {
                    tile: to,
                    side: direction.opposite(),
                },
                kind,
                span,
            },
            self.budget,
        )
    }
    pub fn finish(self) -> Result<FlowTiles, ParseFailure> {
        if self.model.tiles.is_empty() {
            return Err(ParseFailure::syntax(
                0..0,
                "flow document requires a named quoted tile",
            ));
        }
        Ok(self.model)
    }
}
fn unsupported(budget: &mut Budget, span: Range<usize>, value: (&str, &str)) -> ParseFailure {
    let (detail, name) = value;
    if let Err(error) = budget.text(
        name.len()
            .saturating_mul(4)
            .saturating_add(detail.len())
            .saturating_add(64),
    ) {
        return ParseFailure::terminal(error);
    }
    ParseFailure {
        span,
        kind: FailureKind::Unsupported(format!("{detail}: {name:?}").into()),
    }
}
pub(super) fn fonts() -> [Font; 5] {
    let mut fonts = Preamble::default().fonts;
    fonts[0] = Font {
        family: "serif".into(),
        color: Color(0),
        ..Font::default()
    };
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser};
    fn parse(body: &str) -> FlowTiles {
        let mut budget = Budget::new(&RenderOptions::default());
        parser::test_support::body((body, "flow"), &mut budget, parser::flow::parse).unwrap()
    }
    #[test]
    fn should_preserve_cardinal_cells_ignored_root_direction_and_opposite_owned_ports() {
        // given
        let source = "n A \"A\"\ne B \"B\"\ns C \"C\"\nw D \"D\"\ns E \"E\"\nw F \"F\"\nn G \"G\"";
        // when
        let model = parse(source);
        // then
        assert_eq!(
            model
                .placements
                .iter()
                .map(|p| (p.cell.row, p.cell.column))
                .collect::<Vec<_>>(),
            [(0, 0), (0, 1), (1, 1), (1, 0), (2, 0), (2, -1), (1, -1)]
        );
        assert_eq!(model.placements[0].declared, Cardinal::North);
        for (i, path) in model.paths.iter().enumerate() {
            assert_eq!(path.from.tile, i);
            assert_eq!(path.to.tile, i + 1);
            assert_eq!(path.to.side, path.from.side.opposite());
            assert_eq!(path.kind, PathKind::Implicit);
        }
        assert!(
            model
                .tiles
                .windows(2)
                .all(|t| t[0].span.end <= t[1].span.start)
        );
    }
    #[test]
    fn should_preserve_original_probed_explicit_link_cursor_semantics() {
        // given
        let source = include_str!("../../../fixtures/plantuml/flowcursor.puml");
        // when
        let mut budget = Budget::new(&RenderOptions::default());
        let model = parser::test_support::parse(source, &mut budget, parser::flow::parse).unwrap();
        // then
        assert_eq!(
            model
                .placements
                .iter()
                .map(|p| (p.cell.row, p.cell.column))
                .collect::<Vec<_>>(),
            [(0, 0), (0, 1), (1, 1)]
        );
        assert_eq!(model.paths.len(), 3);
        assert_eq!(model.paths[1].kind, PathKind::Explicit);
        assert_eq!((model.paths[1].from.tile, model.paths[1].to.tile), (1, 0));
        assert_eq!((model.paths[2].from.tile, model.paths[2].to.tile), (1, 2));
    }
    #[test]
    fn should_reject_checked_coordinate_overflow_without_mutating_cells() {
        // given
        let cell = Cell {
            row: i64::MAX,
            column: 0,
        };
        // when
        let result = cell.moved(Cardinal::South);
        // then
        assert!(result.is_err());
        assert_eq!(cell.row, i64::MAX);
    }
}
