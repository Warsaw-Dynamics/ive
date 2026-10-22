use super::{keys_at, parsed};

#[test]
fn should_tell_regexp_character_from_class_quantifier_and_meta() {
    // given
    let source = r"^ab\d{2,4}(?<year>x)|[^c-e].\.";

    // when
    let keys = keys_at("RegExp", source, ["ab", r"\d", "2", "year", "|", "c-e", ".", r"\."]);

    // then
    assert_eq!(
        keys,
        [
            Some("REGEXP.CHARACTER"),
            Some("REGEXP.CHAR_CLASS"),
            Some("REGEXP.QUANTIFIER"),
            Some("REGEXP.NAME"),
            Some("REGEXP.META"),
            Some("REGEXP.CHARACTER"),
            Some("REGEXP.DOT"),
            Some("REGEXP.ESC_CHARACTER"),
        ]
    );
}

#[test]
fn should_colour_javascript_regex_literal_as_regexp() {
    // given
    let source = "const r = /^[a-z]+\\d$/gi;";

    // when
    let keys = keys_at("JavaScript", source, ["^", "[", "+", "\\d", "/gi", "const"]);

    // then
    assert_eq!(
        keys,
        [
            Some("REGEXP.META"),
            Some("REGEXP.BRACKETS"),
            Some("REGEXP.QUANTIFIER"),
            Some("REGEXP.CHAR_CLASS"),
            Some("JS.REGEXP"),
            Some("JS.KEYWORD"),
        ]
    );
}

#[test]
fn should_colour_typescript_and_tsx_regex_literals_as_regexp() {
    // given
    let source = "const r: RegExp = /a+/;";

    // when
    let layers = ["TypeScript", "TSX"].map(|language| {
        parsed(language, source).layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect::<Vec<_>>()
    });

    // then
    assert_eq!(layers, [vec![("TypeScript", 0), ("RegExp", 1)], vec![("TSX", 0), ("RegExp", 1)]]);
}
