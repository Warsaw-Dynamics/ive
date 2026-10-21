use super::keys_at;

#[test]
fn should_tell_proto_field_name_from_types_and_field_number() {
    // given
    let source = "syntax = \"proto3\";\nmessage Tab {\n  string title = 7;\n  repeated Cursor cursors = 8;\n}\n";

    // when
    let keys = keys_at("Protocol Buffers", source, ["Tab", "string", "title", "7", "Cursor", "cursors"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PROTO.MESSAGE"),
            Some("PROTO.SCALAR_TYPE"),
            Some("PROTO.FIELD"),
            Some("PROTO.FIELD_NUMBER"),
            Some("PROTO.TYPE"),
            Some("PROTO.FIELD"),
        ]
    );
}

#[test]
fn should_tell_proto_service_rpc_enum_value_and_option() {
    // given
    let source = "enum Mode { MODE_FAST = 1 [deprecated = true]; }\nservice Editor {\n  rpc Save(Tab) returns (Tab);\n}\n";

    // when
    let keys = keys_at("Protocol Buffers", source, ["Mode", "MODE_FAST", "1", "deprecated", "Editor", "Save", "returns"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PROTO.ENUM"),
            Some("PROTO.ENUM_VALUE"),
            Some("PROTO.NUMBER"),
            Some("PROTO.OPTION"),
            Some("PROTO.SERVICE"),
            Some("PROTO.RPC"),
            Some("PROTO.KEYWORD"),
        ]
    );
}
