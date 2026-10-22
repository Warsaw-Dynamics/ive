use super::keys_at;

#[test]
fn should_tell_ini_section_from_key_and_value_kinds() {
    // given
    let source = "; editor\n[editor]\nfont = Mono\nsize = 13\nwrap = off\n";

    // when
    let keys = keys_at("INI", source, ["editor\n", "[editor]", "font", "Mono", "13", "off", "="]);

    // then
    assert_eq!(
        keys,
        [
            Some("INI.COMMENT"),
            Some("INI.BRACKETS"),
            Some("INI.KEY"),
            Some("INI.VALUE"),
            Some("INI.NUMBER"),
            Some("INI.BOOLEAN"),
            Some("INI.OPERATION_SIGN"),
        ]
    );
}

#[test]
fn should_colour_ini_section_name_inside_brackets() {
    // given
    let source = "[project \"ive\"]\nroot = ~\n";

    // when
    let keys = keys_at("INI", source, ["project", "ive"]);

    // then
    assert_eq!(keys, [Some("INI.SECTION"), Some("INI.SECTION")]);
}
