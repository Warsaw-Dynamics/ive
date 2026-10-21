use super::Row;

pub const ROWS: &[Row] = &[
    ("DOCKER.KEYWORD", "Instruction", Some("DEFAULT_KEYWORD")),
    ("DOCKER.STRING", "String", Some("DEFAULT_STRING")),
    ("DOCKER.ESCAPE", "Escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("DOCKER.NUMBER", "Port", Some("DEFAULT_NUMBER")),
    ("DOCKER.LINE_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("DOCKER.PARSER_DIRECTIVE", "Parser directive", Some("DEFAULT_METADATA")),
    ("DOCKER.OPERATOR", "Operator", Some("DEFAULT_OPERATION_SIGN")),
    ("DOCKER.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("DOCKER.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("DOCKER.IMAGE_NAME", "Image name", Some("DEFAULT_CLASS_NAME")),
    ("DOCKER.IMAGE_TAG", "Image tag or digest", Some("DEFAULT_CONSTANT")),
    ("DOCKER.STAGE_NAME", "Build stage name", Some("DEFAULT_LABEL")),
    ("DOCKER.OPTION", "Option", Some("DEFAULT_PARAMETER")),
    ("DOCKER.VARIABLE_DECLARATION", "ARG or ENV name", Some("DEFAULT_LOCAL_VARIABLE")),
    ("DOCKER.VARIABLE", "Variable expansion", Some("DEFAULT_LOCAL_VARIABLE")),
    ("DOCKER.LABEL_KEY", "Label key", Some("DEFAULT_INSTANCE_FIELD")),
    ("DOCKER.HEREDOC", "Here document", Some("DEFAULT_STRING")),
    ("DOCKER.HEREDOC_MARKER", "Here document marker", Some("DEFAULT_KEYWORD")),
];
