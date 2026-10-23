//! Salt tokens, widget dispatch and cell spans from PlantUML MIT 1.2026.9.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance manifest.
use super::{
    budget::{Budget, bound},
    parser::error::ParseFailure,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Border {
    None,
    Outer,
    Grid,
}
impl Border {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Outer => "outer",
            Self::Grid => "grid",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LeafKind {
    Text,
    Field,
    Button,
    CheckboxOn,
    CheckboxOff,
    RadioOn,
    RadioOff,
}
impl LeafKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "salt-text",
            Self::Field => "salt-field",
            Self::Button => "salt-button",
            Self::CheckboxOn => "salt-checkbox-on",
            Self::CheckboxOff => "salt-checkbox-off",
            Self::RadioOn => "salt-radio-on",
            Self::RadioOff => "salt-radio-off",
        }
    }
}
#[derive(Debug)]
pub(super) struct Leaf {
    pub kind: LeafKind,
    pub raw: String,
}
#[derive(Debug)]
pub(super) struct Cell {
    pub id: usize,
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
    pub widget: usize,
}
#[derive(Debug)]
pub(super) struct Table {
    pub border: Border,
    pub cells: Vec<Cell>,
    pub rows: usize,
    pub columns: usize,
}
#[derive(Debug)]
pub(super) enum Kind {
    Leaf(Leaf),
    Table(Table),
}
#[derive(Debug)]
pub(super) struct Widget {
    pub id: usize,
    pub kind: Kind,
}
#[derive(Debug)]
pub(super) struct Salt {
    // Children precede their parent, allowing one bottom-up measurement pass.
    pub widgets: Vec<Widget>,
    pub root: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum End {
    Column,
    Row,
}

struct Frame {
    id: usize,
    table: Table,
    row: usize,
    column: usize,
}

pub(in crate::plantuml) struct Builder<'b> {
    widgets: Vec<Widget>,
    frames: Vec<Frame>,
    next_widget: usize,
    next_cell: usize,
    budget: &'b mut Budget,
    retained: usize,
    limit: usize,
}

impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, retained: usize, limit: usize) -> Self {
        Self {
            widgets: Vec::new(),
            frames: Vec::new(),
            next_widget: 0,
            next_cell: 0,
            budget,
            retained,
            limit,
        }
    }

    pub fn control(&self) -> crate::RenderControl {
        self.budget.control.clone()
    }

    pub fn finish(&mut self, root: usize) -> Result<Salt, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        Ok(Salt {
            widgets: std::mem::take(&mut self.widgets),
            root,
        })
    }

    pub fn open(
        &mut self,
        border: Border,
        _span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.frames.len() + 1)
            .map_err(ParseFailure::terminal)?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        let frame = Frame {
            id: self.next_widget,
            table: Table {
                border,
                cells: Vec::new(),
                rows: 0,
                columns: 0,
            },
            row: 0,
            column: 0,
        };
        push(
            &mut self.frames,
            frame,
            (self.budget, &mut self.retained, self.limit),
        )?;
        self.next_widget += 1;
        Ok(())
    }

    pub fn close(
        &mut self,
        end: End,
        span: std::ops::Range<usize>,
    ) -> Result<(usize, End), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        let frame = self
            .frames
            .pop()
            .ok_or_else(|| ParseFailure::syntax(span.clone(), "Salt closing brace has no table"))?;
        if frame.table.cells.is_empty() {
            return Err(ParseFailure::syntax(span, "Salt table requires a widget"));
        }
        let index = self.widgets.len();
        push(
            &mut self.widgets,
            Widget {
                id: frame.id,
                kind: Kind::Table(frame.table),
            },
            (self.budget, &mut self.retained, self.limit),
        )?;
        Ok((index, end))
    }

    pub fn start_cell(&mut self) -> Result<usize, ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        let id = self.next_cell;
        self.next_cell += 1;
        Ok(id)
    }

    pub fn cell(&mut self, id: usize, widget: (usize, End)) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        let frame = self
            .frames
            .last_mut()
            .ok_or_else(|| ParseFailure::syntax(0..0, "Salt cell has no table"))?;
        let cell = Cell {
            id,
            row: frame.row,
            column: frame.column,
            row_span: 1,
            column_span: 1,
            widget: widget.0,
        };
        push(
            &mut frame.table.cells,
            cell,
            (self.budget, &mut self.retained, self.limit),
        )?;
        advance(frame, widget.1, self.budget)
    }

    pub fn span(&mut self, end: End, span: std::ops::Range<usize>) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        let frame = self
            .frames
            .last_mut()
            .ok_or_else(|| ParseFailure::syntax(span.clone(), "Salt span has no table"))?;
        let last = frame
            .table
            .cells
            .last_mut()
            .filter(|cell| cell.row == frame.row && cell.column + cell.column_span == frame.column)
            .ok_or_else(|| {
                ParseFailure::syntax(
                    span.clone(),
                    "orphan Salt span * requires a preceding cell in the same row",
                )
            })?;
        last.column_span = last
            .column_span
            .checked_add(1)
            .ok_or_else(|| ParseFailure::syntax(span, "Salt span overflow"))?;
        advance(frame, end, self.budget)
    }

    pub fn tree(
        &mut self,
        raw: String,
        span: std::ops::Range<usize>,
    ) -> Result<usize, ParseFailure> {
        if self
            .frames
            .last()
            .is_some_and(|frame| frame.table.cells.is_empty())
        {
            return Err(ParseFailure::unsupported(span, "Salt tree marker"));
        }
        self.leaf((LeafKind::Text, raw), span)
    }

    pub fn leaf(
        &mut self,
        value: (LeafKind, String),
        _span: std::ops::Range<usize>,
    ) -> Result<usize, ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(value.1.len())
            .map_err(ParseFailure::terminal)?;
        let index = self.widgets.len();
        let widget = Widget {
            id: self.next_widget,
            kind: Kind::Leaf(Leaf {
                kind: value.0,
                raw: value.1,
            }),
        };
        push(
            &mut self.widgets,
            widget,
            (self.budget, &mut self.retained, self.limit),
        )?;
        self.next_widget += 1;
        Ok(index)
    }
}

fn advance(frame: &mut Frame, end: End, budget: &mut Budget) -> Result<(), ParseFailure> {
    frame.table.rows = frame.table.rows.max(frame.row + 1);
    frame.table.columns = frame.table.columns.max(frame.column + 1);
    bound("Salt rows", frame.table.rows, budget.limits.model_items)
        .map_err(ParseFailure::terminal)?;
    bound(
        "Salt columns",
        frame.table.columns,
        budget.limits.model_items,
    )
    .map_err(ParseFailure::terminal)?;
    match end {
        End::Column => frame.column += 1,
        End::Row => {
            frame.row += 1;
            frame.column = 0;
        }
    }
    Ok(())
}

fn push<T>(
    values: &mut Vec<T>,
    value: T,
    accounting: (&mut Budget, &mut usize, usize),
) -> Result<(), ParseFailure> {
    let (budget, retained, limit) = accounting;
    budget.charge(1).map_err(ParseFailure::terminal)?;
    if values.len() == values.capacity() {
        let previous = values.capacity();
        let capacity = values.capacity().saturating_mul(2).max(1);
        retain(
            (retained, limit),
            capacity.saturating_mul(std::mem::size_of::<T>()),
        )?;
        budget
            .charge(values.len())
            .map_err(ParseFailure::terminal)?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                ParseFailure::terminal(crate::RenderError::Backend(format!(
                    "PlantUML Salt arena allocation for capacity {capacity} failed: {error}"
                )))
            })?;
        retain(
            (retained, limit),
            values
                .capacity()
                .saturating_sub(capacity)
                .saturating_mul(std::mem::size_of::<T>()),
        )?;
        *retained = retained.saturating_sub(previous.saturating_mul(std::mem::size_of::<T>()));
    }
    values.push(value);
    Ok(())
}

fn retain(accounting: (&mut usize, usize), bytes: usize) -> Result<(), ParseFailure> {
    let (retained, limit) = accounting;
    let next = retained.saturating_add(bytes);
    bound("Salt retained arena bytes", next, limit).map_err(ParseFailure::terminal)?;
    *retained = next;
    Ok(())
}

#[cfg(test)]
pub(super) fn parse(source: &str, budget: &mut Budget) -> Result<Salt, crate::RenderError> {
    let document = super::preprocess::prepare(source, budget)?;
    let mut prepared = super::parser::document::parse(&document, budget)?;
    super::parser::preamble::parse(&mut prepared, budget)?;
    super::parser::salt::parse(prepared.body, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_freeze_distinct_widget_ids_nested_ownership_and_positive_spans() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let source = "@startsalt\n{#\nA | {+\n[X] | (X)\nWide | *\n}\n}\n@endsalt";
        // when
        let model = parse(source, &mut budget).unwrap();
        // then
        let Kind::Table(root) = &model.widgets[model.root].kind else {
            panic!()
        };
        assert_eq!(model.widgets[model.root].id, 0);
        assert_eq!((root.rows, root.columns, root.cells.len()), (1, 2, 2));
        let Kind::Table(nested) = &model.widgets[root.cells[1].widget].kind else {
            panic!()
        };
        assert_eq!((nested.rows, nested.columns, nested.cells.len()), (2, 2, 3));
        assert_eq!(
            (
                nested.cells[2].row,
                nested.cells[2].column,
                nested.cells[2].row_span,
                nested.cells[2].column_span
            ),
            (1, 0, 1, 2)
        );
        let mut ids = model
            .widgets
            .iter()
            .map(|widget| widget.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, [0, 1, 2, 3, 4, 5]);
        for (index, widget) in model.widgets.iter().enumerate() {
            if let Kind::Table(table) = &widget.kind {
                for cell in &table.cells {
                    assert!(cell.widget < index && cell.row_span > 0 && cell.column_span > 0);
                }
            }
        }
    }

    #[test]
    fn should_bound_salt_model_items_and_depth_before_retaining_children() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 1;
        options.limits.nesting_depth = 1;
        let mut budget = Budget::new(&options);
        let mut builder = Builder::new(&mut budget, 0, options.limits.source_bytes * 256);
        builder.open(Border::None, 0..1).unwrap();
        // when
        let depth = builder
            .open(Border::Grid, 2..4)
            .unwrap_err()
            .into_line(1)
            .to_string();
        let items = builder.start_cell().unwrap_err().into_line(1).to_string();
        // then
        assert!(
            depth.contains("nesting depth value 2 exceeds limit 1"),
            "{depth}"
        );
        assert!(
            items.contains("model items value 2 exceeds limit 1"),
            "{items}"
        );
        assert_eq!(
            (
                builder.frames.len(),
                builder.next_widget,
                builder.widgets.len()
            ),
            (1, 1, 0)
        );
        assert!(builder.frames[0].table.cells.is_empty());
    }

    #[test]
    fn should_reject_salt_arena_growth_before_allocating_a_leaf_widget() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let limit = std::mem::size_of::<Frame>();
        let mut builder = Builder::new(&mut budget, 0, limit);
        builder.open(Border::None, 0..1).unwrap();
        // when
        let error = builder
            .leaf((LeafKind::Text, "value".into()), 2..7)
            .unwrap_err()
            .into_line(1)
            .to_string();
        // then
        assert!(error.contains("Salt retained arena bytes"), "{error}");
        assert_eq!(
            (
                builder.widgets.len(),
                builder.widgets.capacity(),
                builder.next_widget
            ),
            (0, 0, 1)
        );
        assert_eq!(builder.retained, limit);
    }
}
