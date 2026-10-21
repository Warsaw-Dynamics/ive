use super::keys_at;

#[test]
fn should_tell_nix_attribute_binding_from_selection_parameter_and_call() {
    // given
    let source = "{ pkgs, ... }:\n{ shell = pkgs.mkShell { name = pkgs.hello.pname; }; }\n";

    // when
    let keys = keys_at("Nix", source, ["pkgs,", "shell", "mkShell", "name", "pname"]);

    // then
    assert_eq!(
        keys,
        [
            Some("NIX.PARAMETER"),
            Some("NIX.ATTRIBUTE_BINDING"),
            Some("NIX.FUNCTION_CALL"),
            Some("NIX.ATTRIBUTE_BINDING"),
            Some("NIX.ATTRIBUTE_SELECTION"),
        ]
    );
}

#[test]
fn should_tell_nix_builtin_path_uri_and_interpolation() {
    // given
    let source = "let src = import ./src; url = https://nixos.org; in \"v${toString src}\"\n";

    // when
    let keys = keys_at("Nix", source, ["import", "./src", "https", "${", "\"v", "let"]);

    // then
    assert_eq!(
        keys,
        [
            Some("NIX.BUILTIN"),
            Some("NIX.PATH"),
            Some("NIX.URI"),
            Some("NIX.INTERPOLATION"),
            Some("NIX.STRING"),
            Some("NIX.KEYWORD"),
        ]
    );
}

#[test]
fn should_tell_nix_doc_comment_from_block_and_line_comment() {
    // given
    let source = "# line\n/* block */\n/** doc */\nx: x\n";

    // when
    let keys = keys_at("Nix", source, ["line", "block", "doc"]);

    // then
    assert_eq!(keys, [Some("NIX.LINE_COMMENT"), Some("NIX.BLOCK_COMMENT"), Some("NIX.DOC_COMMENT")]);
}
