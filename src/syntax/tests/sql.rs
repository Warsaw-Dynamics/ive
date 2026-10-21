use super::keys_at;

#[test]
fn should_tell_sql_table_from_column_alias_and_schema() {
    // given
    let source = "SELECT u.email AS contact FROM auth.users AS u WHERE u.id = 1;";

    // when
    let keys = keys_at("SQL", source, ["SELECT", "email", "contact", "auth", "users", "u WHERE", "1;"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SQL.KEYWORD"),
            Some("SQL.COLUMN"),
            Some("SQL.ALIAS"),
            Some("SQL.SCHEMA"),
            Some("SQL.TABLE"),
            Some("SQL.ALIAS"),
            Some("SQL.NUMBER"),
        ]
    );
}

#[test]
fn should_tell_sql_builtin_type_from_custom_type_and_column_definition() {
    // given
    let source = "CREATE TABLE ticket (\n  title VARCHAR(80) NOT NULL,\n  level priority DEFAULT 'low'\n);";

    // when
    let keys = keys_at("SQL", source, ["title", "VARCHAR", "priority", "'low'"]);

    // then
    assert_eq!(keys, [Some("SQL.COLUMN"), Some("SQL.BUILTIN_TYPE"), Some("SQL.TYPE"), Some("SQL.STRING")]);
}

#[test]
fn should_tell_sql_function_declaration_from_call_builtin_and_parameter() {
    // given
    let source = "CREATE FUNCTION age(since DATE) RETURNS INT AS $$ SELECT 1 $$ LANGUAGE sql;\n\
                  SELECT age(created), count(*) FROM t WHERE id = $1 AND name = @name;";

    // when
    let keys = keys_at("SQL", source, ["age", "since", "age(created", "count", "$1", "@name"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SQL.FUNCTION_DECLARATION"),
            Some("SQL.PARAMETER"),
            Some("SQL.FUNCTION_CALL"),
            Some("SQL.BUILTIN_FUNCTION"),
            Some("SQL.PARAMETER"),
            Some("SQL.PARAMETER"),
        ]
    );
}

#[test]
fn should_tell_sql_line_comment_from_block_comment() {
    // given
    let source = "-- nightly\n/* cut-off */\nSELECT 'a' FROM t;";

    // when
    let keys = keys_at("SQL", source, ["nightly", "cut-off", "'a'"]);

    // then
    assert_eq!(keys, [Some("SQL.LINE_COMMENT"), Some("SQL.BLOCK_COMMENT"), Some("SQL.STRING")]);
}
