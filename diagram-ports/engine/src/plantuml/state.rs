//! MIT PlantUML state declarations, scoped pseudostates and transition semantics.
//! Exact source hashes/notices are recorded in the provenance manifest.
use super::{
    budget::{Budget, invalid, unsupported},
    lexer,
    source::Line,
    text,
};
use crate::RenderError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Scope {
    Root,
    State(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    State,
    Composite,
    Initial,
    Final,
}
#[derive(Debug)]
pub(super) struct Node {
    line: usize,
    pub name: String,
    pub label: String,
    pub descriptions: Vec<String>,
    pub scope: Scope,
    pub kind: Kind,
}
#[derive(Debug)]
pub(super) struct Transition {
    pub from: usize,
    pub to: usize,
    pub label: String,
}
#[derive(Debug)]
pub(super) struct State {
    pub nodes: Vec<Node>,
    pub transitions: Vec<Transition>,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<State, RenderError> {
    let mut model = State {
        nodes: Vec::new(),
        transitions: Vec::new(),
    };
    let mut stack = vec![Scope::Root];
    let mut pending = Vec::new();
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        let scope = *stack.last().unwrap();
        match line.text.as_str() {
            "}" | "end state" | "endstate" => {
                if stack.len() == 1 {
                    return Err(invalid(line.number, "state closer without composite"));
                }
                stack.pop();
            }
            value if value.starts_with("state ") => {
                let id = declare(&mut model, (line, scope), budget)?;
                if lexer::split_label(line).0 != line.text {
                    budget.item()?;
                    pending.push((line, scope));
                }
                if model.nodes[id].kind == Kind::Composite {
                    budget.depth(stack.len())?;
                    stack.push(Scope::State(id));
                }
            }
            _ => {
                budget.item()?;
                pending.push((line, scope));
            }
        }
    }
    if stack.len() != 1 {
        return Err(invalid(
            lines.last().unwrap().number,
            "unclosed composite state",
        ));
    }
    for node in &model.nodes {
        if find_ancestor(&model, (&node.name, node.scope), budget)?.is_some() {
            return Err(unsupported(node.line, "cross-ownership state declaration"));
        }
    }
    for (line, scope) in pending {
        statement(&mut model, (line, scope), budget)?;
    }
    Ok(model)
}

fn declare(
    model: &mut State,
    input: (&Line, Scope),
    budget: &mut Budget,
) -> Result<usize, RenderError> {
    let (line, scope) = input;
    let (head, description) = lexer::split_label(line);
    let (head, kind) = match head
        .strip_suffix('{')
        .or_else(|| head.strip_suffix(" begin"))
    {
        Some(head) => (head.trim_end(), Kind::Composite),
        None => (head, Kind::State),
    };
    let command = Line {
        number: line.number,
        text: head[6..].to_owned(),
    };
    let tokens = lexer::tokens(&command, budget)?;
    let (name, label) = lexer::name(&tokens, line.number)?;
    if name.contains('.')
        || (tokens.len() == 3 && tokens[0].quoted == tokens[2].quoted)
        || (kind == Kind::Composite && tokens.len() == 1 && tokens[0].quoted)
    {
        return Err(unsupported(
            line.number,
            "qualified state declaration or unquoted alias display",
        ));
    }
    if tokens
        .iter()
        .any(|t| !t.quoted && t.text != "as" && !is_name(&t.text))
    {
        return Err(unsupported(line.number, "state identifier"));
    }
    budget.charge(model.nodes.len() + 1)?;
    if model
        .nodes
        .iter()
        .any(|node| node.scope == scope && node.name == name)
    {
        return Err(unsupported(line.number, "duplicate state declaration"));
    }
    budget.item()?;
    budget.text(name.len() + label.len())?;
    text::validate_label(description, line.number)?;
    let id = model.nodes.len();
    model.nodes.push(Node {
        line: line.number,
        name,
        label,
        descriptions: vec![],
        scope,
        kind,
    });
    Ok(id)
}

fn statement(
    model: &mut State,
    input: (&Line, Scope),
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (line, scope) = input;
    let (head, description) = lexer::split_label(line);
    let declaration = head.starts_with("state ");
    if !declaration && let Some(index) = head.find(['-', '<']) {
        return transition(model, (line, scope, index), budget);
    }
    if head == line.text {
        return Err(unsupported(line.number, &line.text));
    }
    let head = head.strip_prefix("state ").unwrap_or(head);
    let tokens = lexer::tokens(
        &Line {
            number: line.number,
            text: head.into(),
        },
        budget,
    )?;
    let (name, _) = lexer::name(&tokens, line.number)?;
    if !declaration && tokens.len() != 1 {
        return Err(unsupported(line.number, &line.text));
    }
    text::validate_label(description, line.number)?;
    let id = resolve(model, (&name, scope, Kind::State, line.number), budget)?;
    budget.text(description.len())?;
    model.nodes[id].descriptions.push(description.into());
    Ok(())
}

fn transition(
    model: &mut State,
    input: (&Line, Scope, usize),
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (line, scope, index) = input;
    let left = line.text[..index].trim();
    let rest = &line.text[index..];
    let end = rest
        .find(|ch: char| !matches!(ch, '-' | '<' | '>'))
        .unwrap_or(rest.len());
    let arrow = &rest[..end];
    let (reverse, body) = if let Some(body) = arrow.strip_prefix('<') {
        (true, body)
    } else if let Some(body) = arrow.strip_suffix('>') {
        (false, body)
    } else {
        return Err(unsupported(line.number, "state transition arrow"));
    };
    if body.is_empty() || body.chars().any(|ch| ch != '-') {
        return Err(unsupported(line.number, "state transition decoration"));
    }
    let right = rest[end..].trim_start();
    let target_end = right.find(char::is_whitespace).unwrap_or(right.len());
    let target = &right[..target_end];
    let tail = right[target_end..].trim();
    let label = match tail.strip_prefix(':') {
        Some(label) if !label.trim().is_empty() => label.trim(),
        None if tail.is_empty() => "",
        _ => {
            return Err(unsupported(
                line.number,
                "state transition suffix or empty label",
            ));
        }
    };
    let (from, to) = if reverse {
        (target, left)
    } else {
        (left, target)
    };
    let from = resolve(model, (from, scope, Kind::Initial, line.number), budget)?;
    let to = resolve(model, (to, scope, Kind::Final, line.number), budget)?;
    text::validate_label(label, line.number)?;
    budget.item()?;
    budget.text(label.len())?;
    model.transitions.push(Transition {
        from,
        to,
        label: label.into(),
    });
    Ok(())
}

fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | ':'))
}

fn resolve(
    model: &mut State,
    input: (&str, Scope, Kind, usize),
    budget: &mut Budget,
) -> Result<usize, RenderError> {
    let (name, scope, role, line) = input;
    let kind = if name == "[*]" { role } else { Kind::State };
    if name != "[*]" && !is_name(name) {
        return Err(unsupported(
            line,
            &format!("state endpoint {name:?}; history/concurrency/decorations are unsupported"),
        ));
    }
    budget.charge(model.nodes.len() + 1)?;
    if let Some(id) = find(model, (name, scope, kind)) {
        return Ok(id);
    }
    if kind == Kind::State && find_ancestor(model, (name, scope), budget)?.is_some() {
        return Err(unsupported(line, "cross-ownership state endpoint"));
    }
    if name.contains('.') || name.contains(':') {
        return Err(unsupported(line, "unresolved qualified state endpoint"));
    }
    budget.item()?;
    budget.text(name.len() * 2)?;
    let id = model.nodes.len();
    model.nodes.push(Node {
        line,
        name: name.into(),
        label: if kind == Kind::State {
            name.into()
        } else {
            String::new()
        },
        descriptions: vec![],
        scope,
        kind,
    });
    Ok(id)
}

fn find_ancestor(
    model: &State,
    input: (&str, Scope),
    budget: &mut Budget,
) -> Result<Option<usize>, RenderError> {
    let (name, mut scope) = input;
    while let Scope::State(id) = scope {
        scope = model.nodes[id].scope;
        budget.charge(model.nodes.len() + 1)?;
        if let Some(id) = find(model, (name, scope, Kind::State)) {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn find(model: &State, input: (&str, Scope, Kind)) -> Option<usize> {
    let (name, scope, kind) = input;
    if matches!(kind, Kind::Initial | Kind::Final) {
        return model
            .nodes
            .iter()
            .position(|n| n.scope == scope && n.kind == kind);
    }
    if let Scope::State(id) = scope
        && model.nodes[id].name == name
    {
        return Some(id);
    }
    model
        .nodes
        .iter()
        .position(|n| n.scope == scope && n.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_leave_state_identity_unchanged_when_rejecting_an_ancestor_endpoint() {
        // given
        let mut budget = Budget::new(&crate::RenderOptions::default());
        let lines = super::super::source::prepare(
            "@startuml\nstate A\nstate Outer {\nstate B\n}\n@enduml",
            &mut budget,
        )
        .unwrap();
        let mut model = parse(&lines, &mut budget).unwrap();
        // when
        let result = resolve(
            &mut model,
            ("A", Scope::State(1), Kind::State, 5),
            &mut budget,
        );
        // then
        assert!(result.unwrap_err().to_string().contains("cross-ownership"));
        assert_eq!(model.nodes.len(), 3);
        assert_eq!(
            model.nodes.iter().filter(|node| node.name == "A").count(),
            1
        );
        assert_eq!(model.nodes[0].scope, Scope::Root);
    }
    #[test]
    fn should_charge_declaration_identity_search_before_scanning() {
        // given
        let source = format!(
            "@startuml\n{}@enduml",
            (0..1000)
                .map(|i| format!("state S{i}\n"))
                .collect::<String>()
        );
        let mut options = crate::RenderOptions::default();
        options.limits.layout_work = 100_000;
        let mut budget = Budget::new(&options);
        let lines = super::super::source::prepare(&source, &mut budget).unwrap();
        // when
        let result = parse(&lines, &mut budget);
        // then
        assert!(
            result.is_err(),
            "identity comparisons escaped the work budget"
        );
    }
}
