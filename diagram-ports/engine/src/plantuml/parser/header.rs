use super::{context, error::ParseFailure};
use crate::{RenderError, plantuml::budget::Budget};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::plantuml) enum Selection {
    #[default]
    Sequence,
    Uml,
    State,
    Activity,
    Timing,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum Atom {
    Word(Word),
    Quote,
    Arrow,
    Initial,
    Colon,
    ColonPair,
    Semi,
    OpenParen,
    Bracket,
    At,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum Word {
    Graph,
    State,
    Timing,
    Activity,
    IfWhile,
    Note,
    NoteSide,
    Beside,
    Over,
    Sequence,
    Group,
    ActorDatabase,
    Left,
    LeftTop,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) struct Tail {
    pub first: Atom,
    pub last_head: Atom,
    pub count: usize,
    pub colon: bool,
    pub arrow: bool,
}

impl Tail {
    pub fn empty() -> Self {
        Self {
            first: Atom::Other,
            last_head: Atom::Other,
            count: 0,
            colon: false,
            arrow: false,
        }
    }

    pub fn append(mut self, atom: Atom, budget: &mut Budget) -> Result<Self, ParseFailure> {
        budget.charge(1).map_err(ParseFailure::terminal)?;
        if self.count == 0 {
            self.first = atom;
        }
        if !self.colon && atom != Atom::Colon {
            self.last_head = atom;
            self.arrow |= atom == Atom::Arrow;
        }
        self.colon |= atom == Atom::Colon;
        self.count += 1;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) struct Evidence {
    pub identifier_arrow: bool,
    pub selection: Selection,
    pub candidate: Selection,
    pub graph: bool,
    pub graph_declaration: bool,
    pub note_body: bool,
    pub at: bool,
    pub timing: bool,
    pub graph_stop: bool,
}

impl Evidence {
    pub fn empty() -> Self {
        Self {
            identifier_arrow: false,
            selection: Selection::Pending,
            candidate: Selection::Pending,
            graph: false,
            graph_declaration: false,
            note_body: false,
            at: false,
            timing: false,
            graph_stop: false,
        }
    }
}

pub(in crate::plantuml) struct Head {
    pub atom: Atom,
    pub selection: Selection,
    pub graph: bool,
    pub declaration: bool,
}

pub(in crate::plantuml) fn finish(
    head: Head,
    tail: Tail,
    budget: &mut Budget,
) -> Result<Evidence, ParseFailure> {
    budget.charge(1).map_err(ParseFailure::terminal)?;
    let first = head.atom;
    let identifier_arrow = first != Atom::Initial
        && first != Atom::Bracket
        && tail.first == Atom::Arrow
        && tail.last_head != Atom::Initial;
    let word = match first {
        Atom::Word(word) => word,
        _ => Word::Other,
    };
    let graph_declaration = !identifier_arrow && head.declaration;
    let note_body = word == Word::Note
        && matches!(
            tail.first,
            Atom::Word(Word::Over | Word::NoteSide | Word::Left | Word::Beside)
        )
        && !tail.colon;
    let selection = if identifier_arrow {
        Selection::Pending
    } else {
        match (first, tail.first) {
            (Atom::Word(Word::IfWhile), Atom::OpenParen) => Selection::Activity,
            (Atom::Word(Word::Note), Atom::Quote) => Selection::Uml,
            (Atom::Word(Word::Note | Word::Sequence), _) => Selection::Sequence,
            (Atom::Initial, _) | (_, Atom::Initial) => Selection::State,
            _ if tail.last_head == Atom::Initial => Selection::State,
            _ => head.selection,
        }
    };
    let candidate = if tail.colon && !tail.arrow && word != Word::ActorDatabase && !identifier_arrow
    {
        Selection::State
    } else {
        Selection::Pending
    };
    let graph = !identifier_arrow && head.graph;
    let graph_stop = !identifier_arrow && word == Word::Group;
    Ok(Evidence {
        identifier_arrow,
        selection,
        candidate,
        graph,
        graph_declaration,
        note_body,
        at: first == Atom::At,
        timing: word == Word::Timing && !identifier_arrow,
        graph_stop,
    })
}

fn word(text: &str) -> Word {
    match text {
        "class" | "abstract" | "object" | "interface" | "enum" | "annotation" | "component"
        | "node" | "rectangle" | "package" | "namespace" | "usecase" | "archimate" => Word::Graph,
        "state" => Word::State,
        "robust" | "concise" => Word::Timing,
        "start" | "stop" | "repeat" | "fork" | "partition" => Word::Activity,
        "if" | "while" => Word::IfWhile,
        "note" => Word::Note,
        "over" => Word::Over,
        "right" => Word::NoteSide,
        "left" => Word::Left,
        "beside" => Word::Beside,
        "participant" | "activate" | "deactivate" | "alt" | "loop" | "opt" | "create"
        | "destroy" => Word::Sequence,
        "group" => Word::Group,
        "actor" | "database" => Word::ActorDatabase,
        "top" => Word::LeftTop,
        _ => Word::Other,
    }
}

pub(super) fn parse(
    text: &str,
    base: usize,
    budget: &mut Budget,
) -> Result<Evidence, ParseFailure> {
    crate::plantuml::budget::bound(
        "header source bytes",
        text.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    if base.checked_add(text.len()).is_none() {
        return Err(ParseFailure::syntax(
            base..base,
            "family header byte range overflow",
        ));
    }
    budget.checkpoint().map_err(ParseFailure::terminal)?;
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let rest = &text[cursor..];
        let ch = rest.chars().next().unwrap();
        let (atom, bytes) = primitive(rest, budget)?;
        if !ch.is_whitespace() {
            context::push(
                &mut tokens,
                (base + cursor, atom, base + cursor + bytes),
                budget,
            )?;
        }
        cursor += bytes;
    }
    context::admit_frontend(
        context::Frontend::Header,
        tokens.len(),
        tokens.capacity(),
        std::mem::size_of::<(usize, Atom, usize)>(),
        budget,
    )?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    super::generated_header::HeaderParser::new()
        .parse(budget, iterator)
        .map_err(|error| {
            super::error::generated_failure(error, base + text.len(), "invalid family header")
        })
}

fn primitive(rest: &str, budget: &mut Budget) -> Result<(Atom, usize), ParseFailure> {
    if rest.starts_with('"') {
        return quote(rest, budget);
    }
    if starts_arrow(rest) {
        let bytes = scan_arrow(rest, budget)?;
        return Ok((Atom::Arrow, bytes));
    }
    let bytes = scan_word(rest, budget)?;
    if bytes > 0 {
        return Ok((Atom::Word(word(&rest[..bytes])), bytes));
    }
    let (atom, bytes) = punctuation(rest);
    budget.charge(bytes).map_err(ParseFailure::terminal)?;
    Ok((atom, bytes))
}

fn starts_arrow(rest: &str) -> bool {
    matches!(
        rest.as_bytes(),
        [b'-', b'-' | b'>', ..]
            | [b'.', b'.', ..]
            | [b'<', b'-' | b'|' | b'.', ..]
            | [b'o' | b'*', b'-' | b'.', ..]
    )
}

fn scan_arrow(rest: &str, budget: &mut Budget) -> Result<usize, ParseFailure> {
    let mut length = 0;
    let mut pending = 0;
    for _ in rest
        .bytes()
        .take_while(|byte| matches!(byte, b'-' | b'.' | b'<' | b'>' | b'|' | b'o' | b'*'))
    {
        length += 1;
        pending += 1;
        if pending == 256 {
            budget.charge(pending).map_err(ParseFailure::terminal)?;
            pending = 0;
        }
    }
    budget.charge(pending).map_err(ParseFailure::terminal)?;
    Ok(length)
}

fn scan_word(rest: &str, budget: &mut Budget) -> Result<usize, ParseFailure> {
    let mut length = 0;
    let mut pending = 0;
    for (offset, ch) in rest.char_indices() {
        if (offset == 0 && ch == '@')
            || starts_arrow(&rest[offset..])
            || !(ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@'))
        {
            break;
        }
        length += ch.len_utf8();
        pending += ch.len_utf8();
        if pending >= 256 {
            budget.charge(pending).map_err(ParseFailure::terminal)?;
            pending = 0;
        }
    }
    budget.charge(pending).map_err(ParseFailure::terminal)?;
    Ok(length)
}

fn quote(rest: &str, budget: &mut Budget) -> Result<(Atom, usize), ParseFailure> {
    let mut escaped = false;
    let mut pending = 1;
    for (offset, ch) in rest.char_indices().skip(1) {
        pending += ch.len_utf8();
        if pending >= 256 {
            budget.charge(pending).map_err(ParseFailure::terminal)?;
            pending = 0;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            budget.charge(pending).map_err(ParseFailure::terminal)?;
            return Ok((Atom::Quote, offset + 1));
        }
    }
    budget.charge(pending).map_err(ParseFailure::terminal)?;
    Ok((Atom::Quote, rest.len()))
}

fn punctuation(rest: &str) -> (Atom, usize) {
    if rest.starts_with("[*]") {
        return (Atom::Initial, 3);
    }
    if rest.starts_with("::") {
        return (Atom::ColonPair, 2);
    }
    let ch = rest.chars().next().unwrap();
    (
        match ch {
            ':' => Atom::Colon,
            ';' => Atom::Semi,
            '(' => Atom::OpenParen,
            '[' => Atom::Bracket,
            '@' => Atom::At,
            _ => Atom::Other,
        },
        ch.len_utf8(),
    )
}

pub(in crate::plantuml) fn select(
    body: &super::BodySource<'_>,
    budget: &mut Budget,
) -> Result<Selection, RenderError> {
    crate::plantuml::budget::bound(
        "family body bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )?;
    let mut candidate = Selection::Sequence;
    let mut graph = None;
    let mut ownership = body.ownership;
    let mut offset = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let range = offset..offset + physical.len();
        offset = range.end;
        let raw = physical.strip_suffix('\n').unwrap_or(physical);
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let evidence = (|| {
            let row = ownership.classify(raw, budget)?;
            if row.opening_action {
                return parse(row.text.trim_start(), range.start, budget);
            }
            ownership.advance(&row, &row.text, budget)
        })()
        .map_err(|mut error: ParseFailure| {
            error.span = range;
            error.into_document(body.document, budget)
        })?;
        if evidence.selection != Selection::Pending {
            return Ok(evidence.selection);
        }
        if evidence.candidate == Selection::State {
            candidate = Selection::State;
        }
        if graph.is_none() {
            if evidence.graph {
                graph = Some(true);
            }
            if evidence.graph_stop {
                graph = Some(false);
            }
        }
    }
    Ok(if candidate == Selection::State {
        candidate
    } else if graph == Some(true) {
        Selection::Uml
    } else {
        Selection::Sequence
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_freeze_state_header_facts_before_literal_label_payload() {
        // given
        for text in [
            "Idle : waiting -> ready",
            "Idle : waiting --> [*]",
            "Idle : waiting o-- ready",
            "Idle : \"waiting -> ready\"",
            "\"Idle -> Ready\" : waiting",
            "Idle::entry : waiting -> ready",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let evidence = parse(text, 0, &mut budget).unwrap();
            // then
            assert_eq!(evidence.candidate, Selection::State, "{text}");
        }
        // given
        for (text, expected) in [
            ("state Idle", Selection::State),
            ("Idle --> Ready : waiting -> ready", Selection::Pending),
            ("class A : waiting -> ready", Selection::Uml),
            ("participant \"arrow -> label\" as A", Selection::Sequence),
            ("robust \"arrow -> track\" as T", Selection::Timing),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let evidence = parse(text, 0, &mut budget).unwrap();
            // then
            assert_eq!(evidence.selection, expected, "{text}");
        }
    }

    #[test]
    fn should_preserve_complete_keyword_prefixed_endpoint_names_in_header_evidence() {
        // given
        for endpoint in [
            "class.foo",
            "class@foo",
            "state.foo",
            "state@foo",
            "start.foo",
            "note@foo",
            "robust.foo",
            "concise@foo",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let evidence = parse(&format!("{endpoint} -> B : ping"), 0, &mut budget).unwrap();
            // then
            assert_eq!(evidence.selection, Selection::Pending, "{endpoint}");
            assert_eq!(evidence.candidate, Selection::Pending, "{endpoint}");
            assert!(
                !evidence.graph_declaration && !evidence.timing && !evidence.note_body,
                "{endpoint}"
            );
        }
        // given
        for (text, expected) in [
            ("class class.foo", Selection::Uml),
            ("state state@foo", Selection::State),
            ("robust \"track\" as T", Selection::Timing),
            ("note over note@foo", Selection::Sequence),
            ("class..>B : ping", Selection::Pending),
            ("class.foo-->B : ping", Selection::Pending),
            ("state@foo<--B : ping", Selection::Pending),
            ("if(ready) then (yes)", Selection::Activity),
            ("\"A -> B\" -> C : ping", Selection::Pending),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let evidence = parse(text, 0, &mut budget).unwrap();
            // then
            assert_eq!(evidence.selection, expected, "{text}");
        }
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let evidence = parse("@42", 0, &mut budget).unwrap();
        // then
        assert!(evidence.at);
        assert!(!evidence.timing);
    }

    #[test]
    fn should_admit_header_source_and_interrupt_primitive_scans_before_retention() {
        // given
        let text = format!("\"{}\"", "Ω".repeat(4096));
        let mut options = RenderOptions::default();
        options.limits.source_bytes = 8;
        let mut budget = Budget::new(&options);
        // when
        let limited = parse(&text, 0, &mut budget);
        // then
        assert!(limited.is_err());
        assert_eq!(budget.work, 0);
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(3);
        // when
        let error = parse(&text, 0, &mut budget).unwrap_err();
        // then
        assert!(
            matches!(
                error.kind,
                super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
            ),
            "{error:?}"
        );
        assert!(budget.work > 0 && budget.work < text.len());
    }

    #[test]
    fn should_derive_typed_headers_without_confusing_keyword_endpoints_or_empty_rows() {
        // given
        for (text, expected) in [
            ("", Selection::Pending),
            ("  ", Selection::Pending),
            ("class -> state : label", Selection::Pending),
            ("robust -> concise : label", Selection::Pending),
            ("actor A", Selection::Pending),
            ("database D", Selection::Pending),
            ("note over A", Selection::Sequence),
            ("note \"label\" as N", Selection::Uml),
            ("[*] --> A", Selection::State),
            ("A --> [*] : final", Selection::State),
            ("class A", Selection::Uml),
            ("state S", Selection::State),
            (":action;", Selection::Activity),
            ("robust \"track\" as T", Selection::Timing),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = parse(text, 0, &mut budget).unwrap();
            // then
            assert_eq!(result.selection, expected, "{text:?}");
        }
    }
}
