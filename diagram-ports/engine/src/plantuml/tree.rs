//! MIT PlantUML mindmap/ and wbs/: ordered marker levels, sides and parent policies.
use super::{
    budget::{Budget, invalid, unsupported},
    source::Line,
    text::validate_label,
};
use crate::RenderError;

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

struct Marker<'a> {
    prefix: &'a str,
    label: &'a str,
    side: Side,
    outline: Outline,
    alias: &'a str,
}

pub(super) fn parse_mindmap(lines: &[Line], budget: &mut Budget) -> Result<Mindmap, RenderError> {
    let mut nodes = Vec::new();
    let mut roots = Vec::new();
    let mut stacks = [Vec::new(), Vec::new()];
    let mut first = String::new();
    let mut multiplier = 0;
    let mut side = Side::Right;
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        let trimmed = line.text.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }
        match trimmed {
            "left side" | "top side" => {
                side = Side::Left;
                continue;
            }
            "right side" | "bottom side" => {
                side = Side::Right;
                continue;
            }
            _ => {}
        }
        let marker = mindmap_marker(line, side)?;
        let raw = if marker.prefix.contains(['+', '-']) {
            marker.prefix.len() - 1
        } else {
            smart_level(marker.prefix, &mut first, Smart::Mindmap, line.number)?
        };
        let level = if nodes.is_empty() { 0 } else { raw };
        let depth = if level == 0 {
            0
        } else {
            if multiplier == 0 {
                multiplier = level;
            }
            if level % multiplier != 0 {
                return Err(invalid(
                    line.number,
                    "mindmap Bad indentation: level is not a multiple of first child depth",
                ));
            }
            level / multiplier
        };
        budget.depth(depth + 1)?;
        let index = nodes.len();
        let branch = usize::from(marker.side == Side::Right);
        let parent = if depth == 0 {
            roots.push(index);
            stacks = [vec![index], vec![index]];
            multiplier = 0;
            Parent::Root
        } else {
            find_parent(&mut stacks[branch], depth, line.number)?
        };
        if depth > 0 {
            stacks[branch].push(index);
        }
        nodes.push(make_node(marker, parent, depth, line.number, budget)?);
    }
    if nodes.is_empty() {
        return Err(invalid(1, "mindmap has no root"));
    }
    Ok(Mindmap { nodes, roots })
}

pub(super) fn parse_wbs(lines: &[Line], budget: &mut Budget) -> Result<Wbs, RenderError> {
    let mut nodes = Vec::new();
    let mut stack = Vec::new();
    let mut first = String::new();
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        if line.text.trim().is_empty() || line.text.trim_start().starts_with('\'') {
            continue;
        }
        let marker = wbs_marker(line)?;
        let depth = if nodes.is_empty() {
            first = marker.prefix.to_owned();
            0
        } else {
            smart_level(marker.prefix, &mut first, Smart::Wbs, line.number)?
        };
        budget.depth(depth + 1)?;
        let parent = if nodes.is_empty() {
            Parent::Root
        } else {
            if depth == 0 {
                return Err(invalid(line.number, "WBS only accepts one root"));
            }
            find_parent(&mut stack, depth, line.number)?
        };
        stack.push(nodes.len());
        nodes.push(make_node(marker, parent, depth, line.number, budget)?);
    }
    if nodes.is_empty() {
        return Err(invalid(1, "WBS has no root"));
    }
    Ok(Wbs { nodes })
}

fn find_parent(stack: &mut Vec<usize>, depth: usize, line: usize) -> Result<Parent, RenderError> {
    if depth > stack.len() {
        return Err(invalid(line, format!("invalid tree depth jump to {depth}")));
    }
    stack.truncate(depth);
    stack
        .last()
        .copied()
        .map(Parent::Node)
        .ok_or_else(|| invalid(line, "tree parent is missing"))
}

fn make_node(
    marker: Marker<'_>,
    parent: Parent,
    depth: usize,
    line: usize,
    budget: &mut Budget,
) -> Result<Node, RenderError> {
    budget.item()?;
    budget.text(marker.label.len() + marker.alias.len())?;
    if marker.label.contains("<<") {
        return Err(unsupported(line, "tree stereotypes"));
    }
    validate_label(marker.label, line)?;
    Ok(Node {
        parent,
        depth,
        side: marker.side,
        outline: marker.outline,
        label: marker.label.to_owned(),
        alias: marker.alias.to_owned(),
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
    line: usize,
) -> Result<usize, RenderError> {
    if first.is_empty() {
        *first = prefix.to_owned();
    }
    let normalized = prefix.replace('\t', " ");
    let kind = if matches!(policy, Smart::Mindmap) && prefix.ends_with("**") {
        normalized.trim()
    } else {
        &normalized
    };
    if !kind.contains(' ') {
        return Ok(kind.len() - 1);
    }
    if kind.ends_with(first.as_str()) {
        return Ok(kind.len() - first.len());
    }
    if kind.trim().len() == 1 {
        return Ok(kind.len() - 1);
    }
    if kind.starts_with(first.as_str()) {
        return Ok(kind.len() - first.len());
    }
    Err(invalid(
        line,
        format!("unsupported tree indentation prefix {prefix:?} relative to {first:?}"),
    ))
}

fn mindmap_marker(line: &Line, default: Side) -> Result<Marker<'_>, RenderError> {
    let raw = &line.text;
    if let Some(label) = raw.strip_prefix('0') {
        return Ok(Marker {
            prefix: "0",
            label: label.trim_start(),
            side: Side::Right,
            outline: Outline::Box,
            alias: "",
        });
    }
    let text = raw.trim_start_matches([' ', '\t']);
    let count = text
        .bytes()
        .take_while(|ch| matches!(ch, b'*' | b'#'))
        .count();
    let (prefix, rest, side) = if count > 0 {
        let end = raw.len() - text.len() + count;
        (&raw[..end], &raw[end..], default)
    } else {
        let count = raw
            .bytes()
            .take_while(|ch| matches!(ch, b'+' | b'-'))
            .count();
        if count == 0 {
            return Err(unsupported(
                line.number,
                &format!("mindmap command {:?}", line.text),
            ));
        }
        (
            &raw[..count],
            &raw[count..],
            if raw[..count].contains('-') {
                Side::Left
            } else {
                Side::Right
            },
        )
    };
    if rest.starts_with('[') {
        return Err(unsupported(line.number, "mindmap colors"));
    }
    let (outline, rest) = if let Some(rest) = rest.strip_prefix('_') {
        (Outline::Boxless, rest)
    } else {
        (Outline::Box, rest)
    };
    if rest.starts_with(':') {
        return Err(unsupported(
            line.number,
            "mindmap multiline command variant",
        ));
    }
    Ok(Marker {
        prefix,
        label: rest.trim_start(),
        side,
        outline,
        alias: "",
    })
}

fn wbs_marker(line: &Line) -> Result<Marker<'_>, RenderError> {
    let raw = &line.text;
    let text = raw.trim_start_matches([' ', '\t']);
    let count = text
        .bytes()
        .take_while(|ch| matches!(ch, b'*' | b'+' | b'-'))
        .count();
    if count == 0 {
        return Err(unsupported(line.number, "WBS command or crosslink"));
    }
    let end = raw.len() - text.len() + count;
    let prefix = &raw[..end];
    let mut rest = &raw[end..];
    let mut outline = Outline::Box;
    let mut side = if prefix.contains('-') {
        Side::Left
    } else {
        Side::Right
    };
    let mut seen_shape = false;
    let mut seen_direction = false;
    while let Some(ch) = rest.chars().next().filter(|c| matches!(c, '_' | '<' | '>')) {
        match ch {
            '_' if !seen_shape => {
                outline = Outline::Boxless;
                seen_shape = true;
            }
            '<' | '>' if !seen_direction => {
                side = if ch == '<' { Side::Left } else { Side::Right };
                seen_direction = true;
            }
            _ => {
                return Err(invalid(
                    line.number,
                    "repeated WBS shape/direction modifier",
                ));
            }
        }
        rest = &rest[1..];
    }
    let mut alias = "";
    if rest.starts_with('(') {
        let close = rest
            .find(')')
            .ok_or_else(|| invalid(line.number, "unterminated WBS alias"))?;
        alias = &rest[1..close];
        if alias.is_empty() || !alias.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(invalid(line.number, format!("invalid WBS alias {alias:?}")));
        }
        rest = &rest[close + 1..];
    }
    if rest.starts_with('[') {
        return Err(unsupported(line.number, "WBS colors"));
    }
    if rest.starts_with(':') {
        return Err(unsupported(line.number, "WBS multiline command variant"));
    }
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return Err(invalid(
            line.number,
            "WBS label requires separating whitespace",
        ));
    }
    let mut label = rest.trim_start();
    if label.starts_with('"') {
        if let Some((display, code)) = label
            .strip_prefix('"')
            .and_then(|s| s.rsplit_once("\" as "))
        {
            if code.is_empty() || !code.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(invalid(line.number, "invalid WBS quoted-label alias"));
            }
            alias = code;
            label = display;
        } else {
            return Err(unsupported(line.number, "WBS quoted label without alias"));
        }
    }
    if label.is_empty() {
        if outline == Outline::Boxless {
            outline = Outline::Pseudo;
        } else {
            return Err(invalid(line.number, "missing WBS node label"));
        }
    }
    Ok(Marker {
        prefix,
        label,
        side,
        outline,
        alias,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    fn lines(source: &str) -> Vec<Line> {
        source
            .lines()
            .enumerate()
            .map(|(i, s)| Line {
                number: i + 2,
                text: s.into(),
            })
            .collect()
    }

    #[test]
    fn should_resolve_side_specific_mindmap_parents_and_depths() {
        // given
        let source = lines(
            "* Root\nleft side\n*** Left\nright side\n*** Right\nleft side\n***** Left child\nright side\n***** Right child\n* Next\n*** Next child",
        );
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = parse_mindmap(&source, &mut budget).unwrap();
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
        let model = parse_wbs(&source, &mut budget).unwrap();
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
        let mindmap = parse_mindmap(&source, &mut Budget::new(&RenderOptions::default()));
        let wbs = parse_wbs(&source, &mut Budget::new(&RenderOptions::default()));
        // then
        assert!(mindmap.is_ok());
        assert!(wbs.is_err());
    }
}
