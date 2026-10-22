use super::keys_at;

#[test]
fn should_colour_jsdoc_tag_type_and_parameter_inside_javascript_doc_comment() {
    // given
    let source = "/**\n * Greets, see {@link Counter}.\n * @param {string} name - who\n */\nfunction greet(name) {}";

    // when
    let keys = keys_at("JavaScript", source, ["@param", "string", "name -", "Greets", "{@link", "function"]);

    // then
    assert_eq!(
        keys,
        [
            Some("JSDOC.TAG"),
            Some("JSDOC.TYPE"),
            Some("JSDOC.PARAMETER"),
            Some("JS.DOC_COMMENT"),
            Some("JSDOC.INLINE_TAG"),
            Some("JS.KEYWORD"),
        ]
    );
}

#[test]
fn should_leave_plain_block_comment_without_jsdoc() {
    // given
    let source = "/* @param {string} name */\nlet n: number = 1;";

    // when
    let keys = keys_at("TypeScript", source, ["@param", "string"]);

    // then
    assert_eq!(keys, [Some("JS.BLOCK_COMMENT"), Some("JS.BLOCK_COMMENT")]);
}

#[test]
fn should_colour_jsdoc_in_typescript_doc_comment() {
    // given
    let source = "/** @returns {number} */\nfunction one(): number { return 1; }";

    // when
    let keys = keys_at("TypeScript", source, ["@returns", "number}"]);

    // then
    assert_eq!(keys, [Some("JSDOC.TAG"), Some("JSDOC.TYPE")]);
}
