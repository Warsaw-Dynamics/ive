//! Salt tokens, widget dispatch and cell spans from PlantUML MIT 1.2026.9.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance manifest.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

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
enum End {
    Column,
    Row,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Syntax {
    Leaf,
    Open,
    Close,
}
#[derive(Debug)]
struct Token {
    text: String,
    line: usize,
    end: End,
    syntax: Syntax,
}
struct Parser<'a> {
    tokens: Vec<Token>,
    cursor: usize,
    next_widget: usize,
    next_cell: usize,
    widgets: Vec<Widget>,
    budget: &'a mut Budget,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Salt, RenderError> {
    let tokens = tokenize(lines, budget)?;
    let mut parser = Parser {
        tokens,
        cursor: 0,
        next_widget: 0,
        next_cell: 0,
        widgets: Vec::new(),
        budget,
    };
    let (root, _) = parser.table(1)?;
    if let Some(token) = parser.tokens.get(parser.cursor) {
        return Err(invalid(
            token.line,
            format!("unconsumed Salt token {:?}", token.text),
        ));
    }
    Ok(Salt {
        widgets: parser.widgets,
        root,
    })
}

fn tokenize(lines: &[Line], budget: &mut Budget) -> Result<Vec<Token>, RenderError> {
    let mut tokens = Vec::new();
    for line in lines {
        let mut rest = line.text.as_str();
        let mut text = String::new();
        while let Some(ch) = rest.chars().next() {
            budget.charge(ch.len_utf8() + 1)?;
            if ch == '\\'
                && let Some(next @ ('|' | '{' | '}')) = rest[1..].chars().next()
            {
                bound(
                    "Salt token bytes",
                    text.len().saturating_add(next.len_utf8()),
                    16 * 1024 + 2,
                )?;
                budget.text(next.len_utf8())?;
                text.push(next);
                rest = &rest[1 + next.len_utf8()..];
                continue;
            }
            if matches!(ch, '|' | '{' | '}') {
                retain_token(&mut tokens, &mut text, line.number, budget)?;
                rest = &rest[1..];
                if ch == '|' {
                    continue;
                }
                budget.text(1)?;
                text.push(ch);
                if ch == '{'
                    && let Some(marker) = rest.chars().next().filter(|c| {
                        matches!(c, '+' | '#' | '-' | '!' | '^' | '*' | '/' | 'S' | 'T')
                    })
                {
                    budget.text(marker.len_utf8())?;
                    text.push(marker);
                    rest = &rest[marker.len_utf8()..];
                }
                retain_token(&mut tokens, &mut text, line.number, budget)?;
                tokens.last_mut().unwrap().syntax = if ch == '{' {
                    Syntax::Open
                } else {
                    Syntax::Close
                };
            } else {
                bound(
                    "Salt token bytes",
                    text.len().saturating_add(ch.len_utf8()),
                    16 * 1024 + 2,
                )?;
                budget.text(ch.len_utf8())?;
                text.push(ch);
                rest = &rest[ch.len_utf8()..];
            }
        }
        retain_token(&mut tokens, &mut text, line.number, budget)?;
        if let Some(last) = tokens.last_mut().filter(|t| t.line == line.number) {
            let tail = line.text.trim_end();
            last.end = if tail.ends_with('|') && !tail.ends_with("\\|") {
                End::Column
            } else {
                End::Row
            };
        }
    }
    Ok(tokens)
}

fn retain_token(
    tokens: &mut Vec<Token>,
    text: &mut String,
    line: usize,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.charge(text.len() + 1)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        text.clear();
        return Ok(());
    }
    budget.item()?;
    budget.text(trimmed.len())?;
    tokens.push(Token {
        text: trimmed.to_owned(),
        line,
        end: End::Column,
        syntax: Syntax::Leaf,
    });
    text.clear();
    Ok(())
}

impl Parser<'_> {
    fn table(&mut self, depth: usize) -> Result<(usize, End), RenderError> {
        self.budget.depth(depth)?;
        let token = self
            .tokens
            .get(self.cursor)
            .ok_or_else(|| invalid(1, "expected Salt table opener"))?;
        if token.syntax != Syntax::Open {
            return Err(invalid(
                token.line,
                format!("expected Salt table opener, found {:?}", token.text),
            ));
        }
        let border = match token.text.as_str() {
            "{" => Border::None,
            "{+" => Border::Outer,
            "{#" => Border::Grid,
            other => {
                return Err(unsupported(
                    token.line,
                    &format!("Salt table opener {other:?}"),
                ));
            }
        };
        self.budget.item()?;
        let id = self.next_widget;
        self.next_widget += 1;
        self.cursor += 1;
        if let Some(first) = self.tokens.get(self.cursor)
            && is_tree_marker(&first.text)
        {
            return Err(unsupported(
                first.line,
                &format!("Salt tree marker {:?}", first.text),
            ));
        }
        let mut table = Table {
            border,
            cells: Vec::new(),
            rows: 0,
            columns: 0,
        };
        let (mut row, mut column) = (0, 0);
        loop {
            self.budget.charge(1)?;
            let token = self
                .tokens
                .get(self.cursor)
                .ok_or_else(|| invalid(1, "unclosed Salt table; expected }"))?;
            if token.syntax == Syntax::Close {
                if table.cells.is_empty() {
                    return Err(invalid(token.line, "Salt table requires a widget"));
                }
                let end = token.end;
                self.cursor += 1;
                self.budget.charge(1)?;
                let index = self.widgets.len();
                self.widgets.push(Widget {
                    id,
                    kind: Kind::Table(table),
                });
                return Ok((index, end));
            }
            let end = if token.text == "*" {
                self.budget.item()?;
                let last = table
                    .cells
                    .last_mut()
                    .filter(|c| c.row == row && c.column + c.column_span == column)
                    .ok_or_else(|| {
                        invalid(
                            token.line,
                            "orphan Salt span * requires a preceding cell in the same row",
                        )
                    })?;
                last.column_span = last
                    .column_span
                    .checked_add(1)
                    .ok_or_else(|| invalid(token.line, "Salt span overflow"))?;
                self.cursor += 1;
                token.end
            } else {
                self.budget.item()?;
                let cell_id = self.next_cell;
                self.next_cell += 1;
                let (widget, end) = if token.syntax == Syntax::Open {
                    self.table(depth + 1)?
                } else {
                    self.leaf()?
                };
                table.cells.push(Cell {
                    id: cell_id,
                    row,
                    column,
                    row_span: 1,
                    column_span: 1,
                    widget,
                });
                end
            };
            table.rows = table.rows.max(row + 1);
            table.columns = table.columns.max(column + 1);
            bound("Salt rows", table.rows, self.budget.limits.model_items)?;
            bound(
                "Salt columns",
                table.columns,
                self.budget.limits.model_items,
            )?;
            match end {
                End::Column => column += 1,
                End::Row => {
                    row += 1;
                    column = 0;
                }
            }
        }
    }

    fn leaf(&mut self) -> Result<(usize, End), RenderError> {
        let token = &self.tokens[self.cursor];
        self.budget.charge(token.text.len() + 1)?;
        let (kind, raw) = classify(&token.text, token.line)?;
        self.budget.item()?;
        self.budget.text(raw.len())?;
        let index = self.widgets.len();
        self.widgets.push(Widget {
            id: self.next_widget,
            kind: Kind::Leaf(Leaf {
                kind,
                raw: raw.to_owned(),
            }),
        });
        self.next_widget += 1;
        self.cursor += 1;
        Ok((index, token.end))
    }
}

fn classify(text: &str, line: usize) -> Result<(LeafKind, &str), RenderError> {
    if matches!(
        text.split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "skinparam" | "<style>" | "</style>" | "title" | "legend" | "header" | "footer"
    ) {
        return Err(unsupported(
            line,
            &format!("Salt styles/chrome must precede the table: {text:?}"),
        ));
    }
    if text.starts_with("[[")
        || text.starts_with("<<")
        || (text.starts_with('^') && text.ends_with('^'))
        || matches!(text, "N=" | "S=" | "E=" | "W=" | "C=")
        || ["--", "==", "~~", ".."]
            .iter()
            .any(|mark| text.starts_with(mark) && text.ends_with(mark))
    {
        return Err(unsupported(
            line,
            &format!("Salt resource, dropdown or border-region token {text:?}"),
        ));
    }
    if let Some(raw) = text.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Ok((LeafKind::Field, raw));
    }
    for (marker, kind) in [
        ("[X]", LeafKind::CheckboxOn),
        ("[]", LeafKind::CheckboxOff),
        ("[ ]", LeafKind::CheckboxOff),
    ] {
        if let Some(raw) = text.strip_prefix(marker) {
            return Ok((kind, raw.trim()));
        }
    }
    if let Some(raw) = text.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return Ok((LeafKind::Button, raw));
    }
    for (marker, kind) in [
        ("(X)", LeafKind::RadioOn),
        ("()", LeafKind::RadioOff),
        ("( )", LeafKind::RadioOff),
    ] {
        if let Some(raw) = text.strip_prefix(marker) {
            return Ok((kind, raw.trim()));
        }
    }
    if text.starts_with(['"', '[']) {
        return Err(invalid(line, format!("malformed Salt widget {text:?}")));
    }
    Ok((LeafKind::Text, text))
}

fn is_tree_marker(text: &str) -> bool {
    matches!(text, "T" | "T+" | "T#" | "T-" | "T!" | "T^")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn lines(source: &str) -> Vec<Line> {
        source
            .lines()
            .enumerate()
            .map(|(i, text)| Line {
                number: i + 1,
                text: text.into(),
            })
            .collect()
    }

    #[test]
    fn should_freeze_distinct_widget_ids_nested_ownership_and_positive_spans() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let source = lines("{#\nA | {+\n[X] | (X)\nWide | *\n}\n}");
        // when
        let model = parse(&source, &mut budget).unwrap();
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
        let mut ids = model.widgets.iter().map(|w| w.id).collect::<Vec<_>>();
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
    fn should_keep_escaped_braces_lexically_distinct_from_structural_tokens() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let source = lines("{ \\{ | \\} | \\| | \u{e000}\u{e001}\u{e002} }");
        // when
        let tokens = tokenize(&source, &mut budget).unwrap();
        // then
        assert_eq!(
            tokens
                .iter()
                .map(|t| (t.text.as_str(), t.syntax, t.end))
                .collect::<Vec<_>>(),
            [
                ("{", Syntax::Open, End::Column),
                ("{", Syntax::Leaf, End::Column),
                ("}", Syntax::Leaf, End::Column),
                ("|", Syntax::Leaf, End::Column),
                ("\u{e000}\u{e001}\u{e002}", Syntax::Leaf, End::Column),
                ("}", Syntax::Close, End::Row)
            ]
        );
    }

    #[test]
    fn should_bound_escape_accumulation_and_lexical_work_before_retention() {
        // given
        let source = lines(&format!("{{\n{}\n}}", "\\|".repeat(16 * 1024 + 3)));
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let error = tokenize(&source, &mut budget).unwrap_err().to_string();
        // then
        assert!(error.contains("Salt token bytes"), "{error}");
        let mut config = RenderOptions::default();
        config.limits.layout_work = 5;
        let mut budget = Budget::new(&config);
        assert!(tokenize(&lines("{ [Large token] }"), &mut budget).is_err());
        assert!(budget.work <= 5);
    }
}
