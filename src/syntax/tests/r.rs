use super::keys_at;

#[test]
fn should_tell_r_function_declaration_from_calls_and_parameter() {
    // given
    let source = "total <- function(x, tax = 0.2) {\n  net <- sum(x$price)\n  apply_tax(net, rate = tax)\n}";

    // when
    let keys = keys_at("R", source, ["total", "x,", "net", "sum", "price", "apply_tax", "rate"]);

    // then
    assert_eq!(
        keys,
        [
            Some("R.FUNCTION_DECLARATION"),
            Some("R.PARAMETER"),
            Some("R.LOCAL_VARIABLE"),
            Some("R.PREDEFINED"),
            Some("R.FIELD"),
            Some("R.FUNCTION_CALL"),
            Some("R.NAMED_ARGUMENT"),
        ]
    );
}

#[test]
fn should_mark_r_roxygen_comment_namespace_and_constant() {
    // given
    let source = "#' Doc\n# plain\nx <- stats::median(c(1, NA))";

    // when
    let keys = keys_at("R", source, ["Doc", "plain", "stats", "median", "NA", "<-"]);

    // then
    assert_eq!(
        keys,
        [
            Some("R.DOC_COMMENT"),
            Some("R.LINE_COMMENT"),
            Some("R.NAMESPACE"),
            Some("R.FUNCTION_CALL"),
            Some("R.CONSTANT"),
            Some("R.ASSIGNMENT"),
        ]
    );
}
