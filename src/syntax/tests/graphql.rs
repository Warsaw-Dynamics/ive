use super::keys_at;

#[test]
fn should_tell_graphql_operation_name_from_field_alias_and_argument() {
    // given
    let source = "query Board($id: ID!) { total: issueCount(project: $id) }";

    // when
    let keys = keys_at("GraphQL", source, ["query", "Board", "$id", "ID", "total", "issueCount", "project"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GRAPHQL.KEYWORD"),
            Some("GRAPHQL.OPERATION_NAME"),
            Some("GRAPHQL.VARIABLE"),
            Some("GRAPHQL.BUILTIN_TYPE"),
            Some("GRAPHQL.ALIAS"),
            Some("GRAPHQL.FIELD"),
            Some("GRAPHQL.ARGUMENT"),
        ]
    );
}

#[test]
fn should_tell_graphql_fragment_definition_from_spread_and_directive() {
    // given
    let source = "{ issues { ...Card @include(if: true) } }\nfragment Card on Issue { key }";

    // when
    let keys = keys_at("GraphQL", source, ["Card", "@include", "fragment Card", "Issue"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GRAPHQL.FRAGMENT_SPREAD"),
            Some("GRAPHQL.DIRECTIVE"),
            Some("GRAPHQL.KEYWORD"),
            Some("GRAPHQL.TYPE"),
        ]
    );
}

#[test]
fn should_tell_graphql_description_from_comment_string_and_type_kinds() {
    // given
    let source = "# schema\n\"\"\"A person\"\"\"\ninterface Node { id: ID }\n\
                  type User implements Node @key(fields: \"id\") { status: Status }\nenum Status { OPEN }";

    // when
    let keys = keys_at("GraphQL", source, ["schema", "A person", "Node", "User", "\"id\"", "OPEN"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GRAPHQL.COMMENT"),
            Some("GRAPHQL.DESCRIPTION"),
            Some("GRAPHQL.INTERFACE"),
            Some("GRAPHQL.TYPE"),
            Some("GRAPHQL.STRING"),
            Some("GRAPHQL.ENUM_VALUE"),
        ]
    );
}
