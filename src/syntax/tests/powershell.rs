use super::keys_at;

#[test]
fn should_tell_powershell_function_declaration_from_command_and_method_call() {
    // given
    let source = "function Get-Total([int] $Count) {\n  $list.Add($Count)\n  Write-Host -Object $list.Length\n}";

    // when
    let keys = keys_at("PowerShell", source, ["Get-Total", "int", "$Count", "Add", "Write-Host", "-Object", "Length"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PS.FUNCTION_DECLARATION"),
            Some("PS.TYPE"),
            Some("PS.PARAMETER"),
            Some("PS.METHOD_CALL"),
            Some("PS.COMMAND"),
            Some("PS.COMMAND_PARAMETER"),
            Some("PS.PROPERTY"),
        ]
    );
}

#[test]
fn should_mark_powershell_help_comment_automatic_variable_and_subexpression() {
    // given
    let source = "<#\n.SYNOPSIS\nAdds.\n#>\n<# note #>\n$name = \"n=$($this.Size)\" -f $null";

    // when
    let keys = keys_at("PowerShell", source, ["SYNOPSIS", "note", "$name", "$(", "$this", "-f"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PS.DOC_COMMENT"),
            Some("PS.BLOCK_COMMENT"),
            Some("PS.VARIABLE"),
            Some("PS.INTERPOLATION"),
            Some("PS.PREDEFINED"),
            Some("PS.OPERATION_SIGN"),
        ]
    );
}

#[test]
fn should_tell_powershell_environment_variable_from_local_and_here_string() {
    // given
    let source = "$path = $env:PATH\n$text = @\"\nin $path\n\"@\n:outer foreach ($x in $list) { break }";

    // when
    let keys = keys_at("PowerShell", source, ["$path", "$env:PATH", "@\"", ":outer", "foreach"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PS.VARIABLE"),
            Some("PS.ENVIRONMENT_VARIABLE"),
            Some("PS.STRING"),
            Some("PS.LABEL"),
            Some("PS.KEYWORD"),
        ]
    );
}

#[test]
fn should_mark_powershell_class_members_and_enum_members() {
    // given
    let source =
        "enum Color { Red }\nclass Box {\n  hidden [int] $Size = 1\n  [int] Area([int] $scale) { return $this.Size * $scale }\n}";

    // when
    let keys = keys_at("PowerShell", source, ["Red", "Box", "hidden", "$Size", "Area", "$scale"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PS.CONSTANT"),
            Some("PS.TYPE"),
            Some("PS.KEYWORD"),
            Some("PS.PROPERTY"),
            Some("PS.FUNCTION_DECLARATION"),
            Some("PS.PARAMETER"),
        ]
    );
}
