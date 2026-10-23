//! Generated UML/description syntax and bounded lexical primitives.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        arrow::{self, Arrow},
        budget::{Budget, bound},
        lexer::Token as Name,
        scene::Decoration,
        style,
        text::validate_label,
        uml::{Builder, Direction, Fill, Kind, Member, Uml},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Keyword {
    pub kind: Kind,
    pub name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum WordEnding {
    Space,
    End,
    Other,
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Word {
    pub name: Name,
    ending: WordEnding,
}

#[derive(Debug)]
pub(in crate::plantuml) struct Declaration {
    pub kind: Kind,
    pub alias: String,
    pub label: String,
    pub fill: Fill,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(in crate::plantuml) struct Link {
    pub from: String,
    pub to: String,
    pub arrow: Arrow,
    pub label: String,
    pub left: String,
    pub right: String,
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Class(Name),
    QuotedClass(Name),
    QuotedArchimate(Name),
    QuotedNote(Name),
    QuotedAbstract(Name),
    QuotedAs(Name),
    QuotedName(Name),
    QuotedMemberKind(Keyword),
    QuotedGroupKind(Keyword),
    QuotedLeft(Word),
    QuotedRight(Word),
    QuotedTop(Word),
    QuotedBottom(Word),
    QuotedTo(Word),
    QuotedDirection(Word),
    MemberKind(Keyword),
    GroupKind(Keyword),
    Archimate(Name),
    Note(Name),
    Abstract(Name),
    As(Name),
    Name(Name),
    Color(Name),
    Arrow(Arrow),
    Label(String),
    Member(Member),
    Left(Word),
    Right(Word),
    Top(Word),
    Bottom(Word),
    To(Word),
    Direction(Word),
    Open,
    Close,
    Colon,
    Newline,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn single(name: Name) -> (String, String) {
    (name.text.clone(), name.text)
}
pub(in crate::plantuml) fn direction(
    value: (Direction, [Word; 4]),
    span: Range<usize>,
) -> Result<Direction, ParseFailure> {
    let (direction, words) = value;
    if words.iter().any(|word| word.name.quoted)
        || words[..3]
            .iter()
            .any(|word| word.ending != WordEnding::Space)
        || words[3].ending != WordEnding::End
    {
        return Err(ParseFailure::unsupported(
            span,
            "direction requires its exact unquoted spelling and single spaces",
        ));
    }
    Ok(direction)
}
pub(in crate::plantuml) fn declaration(
    kind: Kind,
    name: (String, String),
    span: Range<usize>,
) -> Result<Declaration, ParseFailure> {
    if name.0.contains("::") || name.0.contains(['<', '>', '#', '$']) {
        return Err(ParseFailure::unsupported(
            span,
            "entity alias or attached syntax",
        ));
    }
    Ok(Declaration {
        kind,
        alias: name.0,
        label: name.1,
        fill: Fill::Default,
        span,
    })
}
pub(in crate::plantuml) fn archimate(
    color: Name,
    name: (String, String),
    span: Range<usize>,
) -> Result<Declaration, ParseFailure> {
    if color.quoted {
        return Err(ParseFailure::unsupported(
            span,
            "archimate requires an unquoted safe fill",
        ));
    }
    let fill = match color.text.to_ascii_lowercase().as_str() {
        "#lightblue" => style::Color(0xadd8e6),
        "#lightgreen" => style::Color(0x90ee90),
        _ => style::color(&color.text, 1).map_err(|error| {
            primitive_failure(error, span.clone(), "unsupported ArchiMate fill")
        })?,
    };
    if name.0.is_empty()
        || name
            .0
            .chars()
            .any(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@')))
    {
        return Err(ParseFailure::unsupported(
            span,
            "archimate alias must be an identifier",
        ));
    }
    let mut declaration = declaration(Kind::Archimate, name, span)?;
    declaration.fill = Fill::Authored(fill);
    Ok(declaration)
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Uml, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Uml, ParseFailure> {
    bound(
        "UML source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        budget,
        members: false,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        lexer.row(&row, base)?;
        base += physical.len();
    }
    ownership.finish(base)?;
    admit(
        (&lexer.tokens, lexer.tokens.capacity()),
        body.adapters,
        lexer.budget,
    )?;
    let iterator = context::Checked::new(lexer.tokens, lexer.budget.control.clone());
    let mut builder = Builder::new(lexer.budget);
    super::generated_uml::UmlParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(error, body.range.end, "invalid or incomplete UML syntax")
        })?;
    builder.finish(body.range.end..body.range.end)
}

fn primitive_failure(error: RenderError, span: Range<usize>, detail: &'static str) -> ParseFailure {
    match error {
        RenderError::InvalidSource(_) => ParseFailure::syntax(span, detail),
        RenderError::UnsupportedFeature(_) => ParseFailure::unsupported(span, detail),
        error => ParseFailure::terminal(error),
    }
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    budget: &'b mut Budget,
    members: bool,
}
impl Lexer<'_> {
    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }
    fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.budget
            .text(value.len())
            .map_err(ParseFailure::terminal)?;
        for chunk in value.as_bytes().chunks(256) {
            self.budget
                .charge(chunk.len())
                .map_err(ParseFailure::terminal)?;
        }
        Ok(value.into())
    }
    fn row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        let text = row.text.trim();
        if text.is_empty() {
            return Ok(());
        }
        let offset = row.text.len() - row.text.trim_start().len();
        let span = row.span(base, offset..offset + text.len());
        if self.members && text != "}" {
            let plain = text
                .strip_prefix("{static}")
                .or_else(|| text.strip_prefix("{abstract}"))
                .unwrap_or(text);
            if plain.contains(['{', '}']) {
                return Err(ParseFailure::unsupported(
                    span,
                    "member braces or attached syntax",
                ));
            }
            let member = if matches!(text, "--" | ".." | "==" | "__") {
                Member::Separator
            } else {
                validate_label(text, 1).map_err(|error| {
                    primitive_failure(error, span.clone(), "unsupported member label")
                })?;
                let (value, decoration) = if let Some(value) = text.strip_prefix("{static}") {
                    (value.trim(), Decoration::Underlined)
                } else if let Some(value) = text.strip_prefix("{abstract}") {
                    (value.trim(), Decoration::Italic)
                } else {
                    (text, Decoration::Plain)
                };
                Member::Text(self.copy(value)?, decoration)
            };
            self.push(Token::Member(member), span)?;
        } else {
            self.members = false;
            let start_index = self.tokens.len();
            self.primitives(row, (base, offset))?;
            let first = self.tokens.get(start_index).map(|(_, token, _)| token);
            self.members = matches!(
                first,
                Some(
                    Token::Class(_)
                        | Token::MemberKind(_)
                        | Token::Abstract(_)
                        | Token::QuotedClass(_)
                        | Token::QuotedMemberKind(_)
                        | Token::QuotedAbstract(_)
                )
            ) && matches!(self.tokens.last(), Some((_, Token::Open, _)));
        }
        self.push(
            Token::Newline,
            row.span(base, row.text.len()..row.text.len()),
        )
    }

    fn primitives(&mut self, row: &Row, location: (usize, usize)) -> Result<(), ParseFailure> {
        let (base, mut cursor) = location;
        while cursor < row.text.len() {
            let rest = &row.text[cursor..];
            let ch = rest.chars().next().unwrap();
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            if ch.is_whitespace() {
                cursor += ch.len_utf8();
                continue;
            }
            let mut used = ch.len_utf8();
            let token = match ch {
                '{' => Token::Open,
                '}' => Token::Close,
                ':' => {
                    if rest.starts_with("::") {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor..cursor + 2),
                            "qualified port or attached syntax",
                        ));
                    }
                    self.push(Token::Colon, row.span(base, cursor..cursor + 1))?;
                    let value = self.copy(rest[1..].trim())?;
                    validate_label(&value, 1).map_err(|error| {
                        primitive_failure(
                            error,
                            row.span(base, cursor..row.text.len()),
                            "unsupported relation label",
                        )
                    })?;
                    return self.push(
                        Token::Label(value),
                        row.span(base, cursor + 1..row.text.len()),
                    );
                }
                '"' => {
                    let (name, bytes) = self.quoted(row, (base, cursor))?;
                    used = bytes;
                    classify(name, ending(&row.text[cursor + bytes..]))
                }
                _ => {
                    let bytes = if let Some(arrow) = crate::plantuml::lexer::arrow_at(rest) {
                        arrow.len()
                    } else {
                        let mut end = ch.len_utf8();
                        for (index, next) in rest.char_indices().skip(1) {
                            self.budget
                                .charge(next.len_utf8())
                                .map_err(ParseFailure::terminal)?;
                            if next.is_whitespace()
                                || matches!(next, '{' | '}' | ':' | '"')
                                || crate::plantuml::lexer::arrow_at(&rest[index..]).is_some()
                            {
                                break;
                            }
                            end = index + next.len_utf8();
                        }
                        end
                    };
                    used = bytes;
                    let span = row.span(base, cursor..cursor + used);
                    if crate::plantuml::lexer::arrow_at(rest).is_some() {
                        Token::Arrow(arrow::parse(&rest[..bytes], 1).map_err(|error| {
                            primitive_failure(error, span, "unsupported arrow spelling")
                        })?)
                    } else {
                        let value = self.copy(&rest[..bytes])?;
                        if value.starts_with('#') {
                            Token::Color(Name {
                                text: value,
                                quoted: false,
                            })
                        } else {
                            let name = Name {
                                text: value,
                                quoted: false,
                            };
                            validate_name(&name, span)?;
                            classify(name, ending(&row.text[cursor + bytes..]))
                        }
                    }
                }
            };
            self.push(token, row.span(base, cursor..cursor + used))?;
            cursor += used;
        }
        Ok(())
    }
    fn quoted(
        &mut self,
        row: &Row,
        location: (usize, usize),
    ) -> Result<(Name, usize), ParseFailure> {
        let (base, cursor) = location;
        let mut value = String::new();
        let mut escaped = false;
        for (index, ch) in row.text[cursor..].char_indices().skip(1) {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            bound("label bytes", value.len() + ch.len_utf8(), 16 * 1024 + 1)
                .map_err(ParseFailure::terminal)?;
            if value.len() + ch.len_utf8() + 1 > value.capacity() {
                self.budget
                    .charge(value.len())
                    .map_err(ParseFailure::terminal)?;
            }
            if escaped {
                match ch {
                    'n' | '\\' | '~' => {
                        value.push('\\');
                        value.push(ch);
                    }
                    '"' => value.push(ch),
                    _ => {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor + index..cursor + index + ch.len_utf8()),
                            "quoted escape",
                        ));
                    }
                }
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' => {
                        let name = Name {
                            text: self.copy(&value)?,
                            quoted: true,
                        };
                        validate_name(&name, row.span(base, cursor..cursor + index + 1))?;
                        return Ok((name, index + 1));
                    }
                    _ => value.push(ch),
                }
            }
        }
        Err(ParseFailure::syntax(
            row.span(base, cursor..row.text.len()),
            "unterminated quoted identifier",
        ))
    }
}
fn validate_name(name: &Name, span: Range<usize>) -> Result<(), ParseFailure> {
    if !name.quoted
        && (name.text.is_empty()
            || name
                .text
                .chars()
                .any(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@'))))
    {
        return Err(ParseFailure::unsupported(
            span,
            "identifier or attached syntax",
        ));
    }
    validate_label(&name.text, 1)
        .map_err(|error| primitive_failure(error, span, "unsupported name label"))
}
fn ending(tail: &str) -> WordEnding {
    if tail.trim().is_empty() {
        return WordEnding::End;
    }
    let gap = &tail[..tail.len() - tail.trim_start().len()];
    if gap == " " {
        return WordEnding::Space;
    }
    WordEnding::Other
}
fn classify(name: Name, ending: WordEnding) -> Token {
    let quoted = name.quoted;
    let token = classify_word(name, ending);
    if quoted {
        return match token {
            Token::MemberKind(value) => Token::QuotedMemberKind(value),
            Token::GroupKind(value) => Token::QuotedGroupKind(value),
            Token::Class(value) => Token::QuotedClass(value),
            Token::Archimate(value) => Token::QuotedArchimate(value),
            Token::Note(value) => Token::QuotedNote(value),
            Token::Abstract(value) => Token::QuotedAbstract(value),
            Token::As(value) => Token::QuotedAs(value),
            Token::Name(value) => Token::QuotedName(value),
            Token::Left(value) => Token::QuotedLeft(value),
            Token::Right(value) => Token::QuotedRight(value),
            Token::Top(value) => Token::QuotedTop(value),
            Token::Bottom(value) => Token::QuotedBottom(value),
            Token::To(value) => Token::QuotedTo(value),
            Token::Direction(value) => Token::QuotedDirection(value),
            token => token,
        };
    }
    token
}
fn classify_word(name: Name, ending: WordEnding) -> Token {
    let kind = match name.text.as_str() {
        "class" => return Token::Class(name),
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        "annotation" => Kind::Annotation,
        "object" => Kind::Object,
        "package" => Kind::Package,
        "namespace" => Kind::Namespace,
        "component" => Kind::Component,
        "node" => Kind::Node,
        "database" => Kind::Database,
        "actor" => Kind::Actor,
        "usecase" => Kind::Usecase,
        "rectangle" => Kind::Rectangle,
        "archimate" => return Token::Archimate(name),
        "note" => return Token::Note(name),
        "abstract" => return Token::Abstract(name),
        "as" => return Token::As(name),
        "left" => return Token::Left(Word { name, ending }),
        "right" => return Token::Right(Word { name, ending }),
        "top" => return Token::Top(Word { name, ending }),
        "bottom" => return Token::Bottom(Word { name, ending }),
        "to" => return Token::To(Word { name, ending }),
        "direction" => return Token::Direction(Word { name, ending }),
        _ => return Token::Name(name),
    };
    if kind.members() {
        Token::MemberKind(Keyword { kind, name })
    } else {
        Token::GroupKind(Keyword { kind, name })
    }
}

fn admit(
    input: (&[Spanned], usize),
    adapters: usize,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    let mut depth = 0usize;
    let mut maximum = 0usize;
    let mut literals = 0usize;
    let mut model_items = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1).map_err(ParseFailure::terminal)?;
        model_items = model_items.saturating_add(match token {
            Token::Class(_)
            | Token::QuotedClass(_)
            | Token::MemberKind(_)
            | Token::QuotedMemberKind(_)
            | Token::GroupKind(_)
            | Token::QuotedGroupKind(_)
            | Token::Archimate(_)
            | Token::QuotedArchimate(_)
            | Token::Note(_)
            | Token::QuotedNote(_)
            | Token::Member(_) => 1,
            Token::Arrow(_) => 3,
            _ => 0,
        });
        match token {
            Token::Open => {
                depth += 1;
                maximum = maximum.max(depth);
                budget.depth(depth).map_err(ParseFailure::terminal)?;
            }
            Token::Close => depth = depth.saturating_sub(1),
            Token::MemberKind(key)
            | Token::GroupKind(key)
            | Token::QuotedMemberKind(key)
            | Token::QuotedGroupKind(key) => literals += key.name.text.capacity(),
            Token::QuotedClass(name)
            | Token::QuotedArchimate(name)
            | Token::QuotedNote(name)
            | Token::QuotedAbstract(name)
            | Token::QuotedAs(name)
            | Token::QuotedName(name)
            | Token::Class(name)
            | Token::Archimate(name)
            | Token::Note(name)
            | Token::Abstract(name)
            | Token::As(name)
            | Token::Name(name)
            | Token::Color(name) => literals += name.text.capacity(),
            Token::QuotedLeft(word)
            | Token::QuotedRight(word)
            | Token::QuotedTop(word)
            | Token::QuotedBottom(word)
            | Token::QuotedTo(word)
            | Token::QuotedDirection(word)
            | Token::Left(word)
            | Token::Right(word)
            | Token::Top(word)
            | Token::Bottom(word)
            | Token::To(word)
            | Token::Direction(word) => literals += word.name.text.capacity(),
            Token::Label(value) | Token::Member(Member::Text(value, _)) => {
                literals += value.capacity()
            }
            _ => {}
        }
    }
    // Actual 379-state/36-terminal tables: 2d+8 live states, four reductions
    // between deliveries, and at most 30 expected terminals. Leave twelve
    // extra states; include both old/new buffers during geometric growth.
    let live = maximum.saturating_mul(2).saturating_add(20);
    let stack = live.next_power_of_two().max(8);
    let growing = stack + stack / 2;
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Declaration>()
        + std::mem::size_of::<Link>()
        + 4 * std::mem::size_of::<usize>();
    let diagnostics = 3 * (live + 1) + 64;
    let retained = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals)
        .saturating_add(adapters)
        .saturating_add(growing.saturating_mul(symbol + 1))
        .saturating_add(diagnostics * std::mem::size_of::<String>() + 36 * 32)
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(Builder::retained_bytes(
            budget.limits.model_items.min(model_items),
        ))
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "UML parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    bound(
        "UML parser stack slots",
        growing * 2 + diagnostics,
        budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(
            tokens
                .len()
                .saturating_mul(12)
                .saturating_add(growing * 2 + diagnostics + 36 * (live + 12)),
        )
        .map_err(ParseFailure::terminal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn model(body: &str) -> Result<Uml, RenderError> {
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(
            &format!("@startuml\n{body}\n@enduml"),
            &mut budget,
        )?;
        let mut parsed = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut parsed, &mut budget)?;
        parse(parsed.body, &mut budget)
    }

    #[test]
    fn should_build_generated_scopes_members_aliases_and_relation_roles() {
        // given
        let body = "package P {\nclass \"Alpha\" as A {\n{static} count\n--\n{abstract} act()\n}\nobject B\nA \"one\" --> \"many\" B : calls\n}";
        // when
        let result = model(body).unwrap();
        // then
        assert_eq!(
            result
                .entities
                .iter()
                .map(|entity| entity.key.as_str())
                .collect::<Vec<_>>(),
            ["P", "P.A", "P.B"]
        );
        assert_eq!(result.entities[1].members.len(), 3);
        assert_eq!((result.relations[0].from, result.relations[0].to), (1, 2));
        assert_eq!(result.relations[0].left, "one");
        assert_eq!(result.relations[0].right, "many");
    }

    #[test]
    fn should_keep_archimate_contextual_names_and_quoted_arrow_labels() {
        // given
        let body = "rectangle archimate\narchimate #LightBlue \"Order --> service\" as A\nnote \"plain\" as N\narchimate --> A : reads --> literal\nA --> N";
        // when
        let result = model(body).unwrap();
        // then
        assert_eq!(result.entities[0].alias, "archimate");
        assert_eq!(result.entities[1].label, "Order --> service");
        assert_eq!(result.relations.len(), 2);
    }

    #[test]
    fn should_keep_direction_spacing_and_quotes_exact() {
        // given
        for direction in [
            "left\tto right direction",
            "left  to right direction",
            "\"left\" to right direction",
        ] {
            // when
            let result = model(&format!("class A\n{direction}"));
            // then
            assert!(result.is_err(), "accepted {direction}: {result:?}");
        }
    }

    #[test]
    fn should_reject_generated_suffixes_and_unclosed_scopes_at_eof() {
        // given
        let streams = [
            vec![(0, Token::Close, 1), (1, Token::Newline, 1)],
            vec![
                (
                    0,
                    Token::MemberKind(Keyword {
                        kind: Kind::Class,
                        name: Name {
                            text: "class".into(),
                            quoted: false,
                        },
                    }),
                    5,
                ),
                (
                    6,
                    Token::Name(Name {
                        text: "A".into(),
                        quoted: false,
                    }),
                    7,
                ),
                (7, Token::Open, 8),
                (8, Token::Newline, 9),
            ],
            vec![
                (
                    0,
                    Token::GroupKind(Keyword {
                        kind: Kind::Package,
                        name: Name {
                            text: "package".into(),
                            quoted: false,
                        },
                    }),
                    7,
                ),
                (
                    8,
                    Token::Name(Name {
                        text: "P".into(),
                        quoted: false,
                    }),
                    9,
                ),
                (9, Token::Open, 10),
                (10, Token::Newline, 11),
            ],
        ];
        for tokens in streams {
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_uml::UmlParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for body in [
            "class A : suffix",
            "class A extra",
            "class A\nA --> B extra",
            "rectangle A\nclass B",
            "archimate #123456 A {\nrectangle B\n}",
        ] {
            assert!(model(body).is_err(), "accepted {body}");
        }
    }

    #[test]
    fn should_keep_every_safe_archimate_fill_and_reject_icons_in_every_position() {
        // given
        for (fill, expected) in [
            ("#LightBlue", 0xadd8e6),
            ("#lightgreen", 0x90ee90),
            ("#123456", 0x123456),
            ("#aAbBcC", 0xaabbcc),
        ] {
            // when
            let result = model(&format!("archimate {fill} A")).unwrap();
            // then
            assert!(
                matches!(result.entities[0].fill, Fill::Authored(style::Color(value)) if value == expected)
            );
        }
        for body in [
            "archimate #123456 A <<business>>",
            "archimate #123456 \"A\" <<business>> as A",
            "archimate #123456 A <<business>> as \"A\"",
            "archimate #123456 A as \"A\" <<business>>",
            "archimate #url(x) A",
            "archimate \"#123456\" A",
        ] {
            assert!(model(body).is_err(), "accepted {body}");
        }
    }

    #[test]
    fn should_map_generated_and_semantic_failures_once_through_expansion_origins() {
        // given
        for body in [
            "!define BAD class A extra\nBAD",
            "!define BAD class A\nclass A\nBAD",
        ] {
            // when
            let error = model(body).unwrap_err().to_string();
            // then
            assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
            assert!(error.contains("used at line"), "{error}");
        }
    }

    #[test]
    fn should_bound_syntax_tokens_depth_work_and_cancel_before_generated_delivery() {
        // given
        for resource in ["tokens", "depth", "work", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                "@startuml\npackage P {\npackage Q {\nclass A\n}\n}\n@enduml",
                &mut budget,
            )
            .unwrap();
            let mut parsed = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut parsed, &mut budget).unwrap();
            match resource {
                "tokens" => budget.limits.source_bytes = 4,
                "depth" => budget.limits.nesting_depth = 1,
                "work" => budget.limits.layout_work = budget.work + 1,
                _ => budget.control.cancel_after_checkpoints(30),
            }
            // when
            let error = parse(parsed.body, &mut budget).unwrap_err();
            // then
            if resource == "cancel" {
                assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
            } else {
                assert!(
                    matches!(error, RenderError::Backend(_))
                        && error.to_string().contains("exceeds limit"),
                    "{resource}: {error}"
                );
            }
        }
    }

    #[test]
    fn should_keep_shared_comment_ownership_at_the_family_boundary() {
        // given
        let body = "title Ω /' comment\n'/  package P {\nclass A\n}\nclass B\nP.A --> B";
        // when
        let result = model(body).unwrap();
        // then
        assert_eq!(result.entities[1].key, "P.A");
        assert_eq!((result.relations[0].from, result.relations[0].to), (1, 2));
    }

    #[test]
    fn should_cancel_generated_delivery_and_model_actions() {
        // given
        for checkpoint in 1..=8 {
            let tokens = vec![
                (
                    0,
                    Token::Class(Name {
                        text: "class".into(),
                        quoted: false,
                    }),
                    5,
                ),
                (
                    6,
                    Token::Name(Name {
                        text: "A".into(),
                        quoted: false,
                    }),
                    7,
                ),
                (7, Token::Newline, 8),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_uml::UmlParser::new().parse(&mut builder, iterator);
            // then
            assert!(
                matches!(
                    result,
                    Err(lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    })
                ),
                "checkpoint {checkpoint}: {result:?}"
            );
        }
    }
}
