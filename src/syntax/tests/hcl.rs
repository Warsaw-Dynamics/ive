use super::keys_at;

#[test]
fn should_tell_hcl_block_type_from_labels_and_attribute() {
    // given
    let source = "resource \"aws_instance\" \"web\" {\n  ami = \"ami-1\"\n  ebs { size = 8 }\n}\n";

    // when
    let keys = keys_at("HCL", source, ["resource", "aws_instance", "web", "ami ", "ebs", "8"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HCL.BLOCK_TYPE"),
            Some("HCL.BLOCK_LABEL"),
            Some("HCL.BLOCK_LABEL"),
            Some("HCL.ATTRIBUTE"),
            Some("HCL.BLOCK_TYPE"),
            Some("HCL.NUMBER"),
        ]
    );
}

#[test]
fn should_tell_hcl_input_variable_from_reference_property_and_function_call() {
    // given
    let source = "x = max(var.size, aws_disk.data.size)\n";

    // when
    let keys = keys_at("HCL", source, ["max", "var", "size,", "aws_disk", "data.size"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HCL.FUNCTION_CALL"),
            Some("HCL.PREDEFINED"),
            Some("HCL.VARIABLE"),
            Some("HCL.LOCAL_VARIABLE"),
            Some("HCL.PROPERTY"),
        ]
    );
}

#[test]
fn should_mark_hcl_interpolation_and_heredoc_inside_strings() {
    // given
    let source = "name = \"web-${local.env}\"\ndata = <<EOT\n%{ if on }up%{ endif }\nEOT\n";

    // when
    let keys = keys_at("HCL", source, ["web-", "${", "env", "<<EOT", "if", "up"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HCL.STRING"),
            Some("HCL.INTERPOLATION"),
            Some("HCL.VARIABLE"),
            Some("HCL.HEREDOC_DELIMITER"),
            Some("HCL.KEYWORD"),
            Some("HCL.STRING"),
        ]
    );
}

#[test]
fn should_mark_hcl_object_key_loop_variable_and_comments() {
    // given
    let source = "# ids\n/* all */\ntags = { Name = [for s in list : s.id] }\n";

    // when
    let keys = keys_at("HCL", source, ["ids", "all", "Name", "s in", "for"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HCL.LINE_COMMENT"),
            Some("HCL.BLOCK_COMMENT"),
            Some("HCL.ATTRIBUTE"),
            Some("HCL.LOCAL_VARIABLE"),
            Some("HCL.KEYWORD"),
        ]
    );
}
