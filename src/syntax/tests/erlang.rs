use super::keys_at;

#[test]
fn should_tell_erlang_function_clause_from_local_and_remote_calls() {
    // given
    let source = "-module(cart).\ntotal(Items, Rate) ->\n    Sum = lists:sum(Items),\n    round(Sum * Rate).";

    // when
    let keys = keys_at("Erlang", source, ["-module", "cart", "total", "Items,", "Sum =", "lists", "sum(", "round"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ERLANG.ATTRIBUTE"),
            Some("ERLANG.MODULE"),
            Some("ERLANG.FUNCTION_DECLARATION"),
            Some("ERLANG.PARAMETER"),
            Some("ERLANG.VARIABLE"),
            Some("ERLANG.MODULE"),
            Some("ERLANG.REMOTE_CALL"),
            Some("ERLANG.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_mark_erlang_records_macros_and_atoms() {
    // given
    let source = "-record(user, {name}).\nname(#user{name = N}) -> {?MODULE, ok, N}.";

    // when
    let keys = keys_at("Erlang", source, ["user,", "name}", "#user", "?MODULE", "ok"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ERLANG.RECORD"),
            Some("ERLANG.RECORD_FIELD"),
            Some("ERLANG.RECORD"),
            Some("ERLANG.MACRO"),
            Some("ERLANG.ATOM"),
        ]
    );
}

#[test]
fn should_tell_erlang_spec_types_from_doc_and_comment() {
    // given
    let source = "% plain\n-doc \"Counts.\".\n-spec count(list()) -> integer().\ncount(L) -> length(L).";

    // when
    let keys = keys_at("Erlang", source, ["plain", "Counts", "count(list", "list()", "length"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ERLANG.LINE_COMMENT"),
            Some("ERLANG.DOC_COMMENT"),
            Some("ERLANG.FUNCTION_DECLARATION"),
            Some("ERLANG.TYPE"),
            Some("ERLANG.FUNCTION_CALL"),
        ]
    );
}
