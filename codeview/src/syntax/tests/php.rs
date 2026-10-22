use super::{keys_at, parsed};

#[test]
fn should_tell_php_function_declaration_from_calls_and_variable_from_property() {
    // given
    let source = "<?php\nfunction total($order) {\n    $sum = $order->price;\n    return round($sum) + Tax::rate();\n}";

    // when
    let keys = keys_at("PHP", source, ["total", "$order", "$sum", "price", "round", "Tax", "rate"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PHP.FUNCTION_DECLARATION"),
            Some("PHP.PARAMETER"),
            Some("PHP.VARIABLE"),
            Some("PHP.PROPERTY"),
            Some("PHP.FUNCTION_CALL"),
            Some("PHP.CLASS"),
            Some("PHP.STATIC_METHOD_CALL"),
        ]
    );
}

#[test]
fn should_mark_php_doc_comment_method_and_constant() {
    // given
    let source = "<?php\nclass Book {\n    /** Adds. */\n    public function add() { $this->log(self::MAX); }\n}";

    // when
    let keys = keys_at("PHP", source, ["/**", "add", "$this", "log", "MAX", "<?php"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PHP.DOC_COMMENT"),
            Some("PHP.METHOD_DECLARATION"),
            Some("PHP.PREDEFINED"),
            Some("PHP.METHOD_CALL"),
            Some("PHP.CONSTANT"),
            Some("PHP.TAG"),
        ]
    );
}

#[test]
fn should_colour_html_around_php_as_html() {
    // given
    let source = "<ul>\n<?php foreach ($items as $item): ?>\n  <li class=\"x\"><?= $item ?></li>\n<?php endforeach ?>";

    // when
    let keys = keys_at("PHP", source, ["ul", "class", "$item", "foreach"]);

    // then
    assert_eq!(keys, [Some("HTML_TAG_NAME"), Some("HTML_ATTRIBUTE_NAME"), Some("PHP.VARIABLE"), Some("PHP.KEYWORD")]);
}

#[test]
fn should_parse_all_html_parts_of_php_as_one_document() {
    // given
    let source = "<div>\n<?php echo 1; ?>\n<p>a</p>\n<?php echo 2; ?>\n</div>";

    // when
    let syntax = parsed("PHP", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("PHP", 0), ("HTML", 1)]);
}
