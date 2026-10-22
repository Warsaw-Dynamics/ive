use super::keys_at;

#[test]
fn should_tell_lua_function_declaration_from_calls_and_field() {
    // given
    let source = "function Book:add(order)\n  local n = self.size\n  log(n)\n  self:flush()\n  print(n)\nend";

    // when
    let keys = keys_at("Lua", source, ["add", "order", "n =", "size", "log", "flush", "print"]);

    // then
    assert_eq!(
        keys,
        [
            Some("LUA.FUNCTION_DECLARATION"),
            Some("LUA.PARAMETER"),
            Some("LUA.LOCAL_VARIABLE"),
            Some("LUA.FIELD"),
            Some("LUA.FUNCTION_CALL"),
            Some("LUA.METHOD_CALL"),
            Some("LUA.PREDEFINED"),
        ]
    );
}

#[test]
fn should_tell_lua_doc_comment_from_line_and_block_comment() {
    // given
    let source = "--- doc\n-- line\n--[[ block ]]\nlocal MAX <const> = 1";

    // when
    let keys = keys_at("Lua", source, ["doc", "line", "block", "MAX", "const"]);

    // then
    assert_eq!(
        keys,
        [
            Some("LUA.DOC_COMMENT"),
            Some("LUA.LINE_COMMENT"),
            Some("LUA.BLOCK_COMMENT"),
            Some("LUA.CONSTANT"),
            Some("LUA.ATTRIBUTE"),
        ]
    );
}
