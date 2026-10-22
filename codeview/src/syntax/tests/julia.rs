use super::keys_at;

#[test]
fn should_tell_julia_function_declaration_from_calls_and_parameter() {
    // given
    let source = "function total(book::Book; tax=0.2)\n    net = sum(book.prices)\n    apply(net)\nend";

    // when
    let keys = keys_at("Julia", source, ["total", "book", "Book", "tax", "net", "sum", "prices", "apply"]);

    // then
    assert_eq!(
        keys,
        [
            Some("JULIA.FUNCTION_DECLARATION"),
            Some("JULIA.PARAMETER"),
            Some("JULIA.TYPE"),
            Some("JULIA.PARAMETER"),
            Some("JULIA.LOCAL_VARIABLE"),
            Some("JULIA.PREDEFINED"),
            Some("JULIA.FIELD"),
            Some("JULIA.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_mark_julia_docstring_macro_symbol_and_interpolation() {
    // given
    let source = "\"\"\"Docs.\"\"\"\nf(x) = @show x\ns = :done\nprintln(\"n=$x\")";

    // when
    let keys = keys_at("Julia", source, ["Docs", "f(", "@show", ":done", "$x"]);

    // then
    assert_eq!(
        keys,
        [
            Some("JULIA.DOC_COMMENT"),
            Some("JULIA.FUNCTION_DECLARATION"),
            Some("JULIA.MACRO"),
            Some("JULIA.SYMBOL"),
            Some("JULIA.INTERPOLATION"),
        ]
    );
}
