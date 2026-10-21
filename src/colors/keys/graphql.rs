use super::Row;

/// No GraphQL keys in intellij-community (the GraphQL plugin is separate); ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("GRAPHQL.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("GRAPHQL.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("GRAPHQL.STRING", "String", Some("DEFAULT_STRING")),
    ("GRAPHQL.DESCRIPTION", "Description", Some("DEFAULT_DOC_COMMENT")),
    ("GRAPHQL.COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("GRAPHQL.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("GRAPHQL.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("GRAPHQL.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("GRAPHQL.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("GRAPHQL.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("GRAPHQL.OPERATION_NAME", "Operation name", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("GRAPHQL.FRAGMENT_DEFINITION", "Fragment definition", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("GRAPHQL.FRAGMENT_SPREAD", "Fragment spread", Some("DEFAULT_FUNCTION_CALL")),
    ("GRAPHQL.FIELD", "Field", Some("DEFAULT_INSTANCE_FIELD")),
    ("GRAPHQL.ALIAS", "Field alias", Some("DEFAULT_LABEL")),
    ("GRAPHQL.ARGUMENT", "Argument", Some("DEFAULT_PARAMETER")),
    ("GRAPHQL.VARIABLE", "Variable", Some("DEFAULT_LOCAL_VARIABLE")),
    ("GRAPHQL.TYPE", "Type", Some("DEFAULT_CLASS_NAME")),
    ("GRAPHQL.INTERFACE", "Interface declaration", Some("DEFAULT_INTERFACE_NAME")),
    ("GRAPHQL.BUILTIN_TYPE", "Builtin scalar", Some("DEFAULT_KEYWORD")),
    ("GRAPHQL.ENUM_VALUE", "Enum value", Some("DEFAULT_CONSTANT")),
    ("GRAPHQL.DIRECTIVE", "Directive", Some("DEFAULT_METADATA")),
];
