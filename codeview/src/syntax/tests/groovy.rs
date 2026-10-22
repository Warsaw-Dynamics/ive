use super::keys_at;

#[test]
fn should_tell_groovy_method_declaration_from_instance_and_static_call() {
    // given
    let source = "class C {\n  def run(int p) { list.add(p); Math.max(1, 2) }\n}";

    // when
    let keys = keys_at("Groovy", source, ["run", "add", "max", "p)", "def"]);

    // then
    assert_eq!(
        keys,
        [
            Some("Groovy method declaration"),
            Some("Method call"),
            Some("Static method access"),
            Some("Groovy parameter"),
            Some("GROOVY_KEYWORD"),
        ]
    );
}

#[test]
fn should_tell_groovy_closure_braces_map_key_and_string_kinds() {
    // given
    let source = "def map = [key: 'single', other: \"double\\n\"]\nlist.each { item -> println item }";

    // when
    let keys = keys_at("Groovy", source, ["map", "key", "'single'", "\"double", "\\n", "{", "item"]);

    // then
    assert_eq!(
        keys,
        [
            Some("Groovy var"),
            Some("Map key"),
            Some("String"),
            Some("GString"),
            Some("Valid string escape"),
            Some("Closure braces"),
            Some("Groovy parameter"),
        ]
    );
}

#[test]
fn should_colour_gradle_build_script_calls_and_groovydoc() {
    // given
    let source = "/** doc */\ndependencies {\n    implementation 'org.x:y:1.0'\n}";

    // when
    let keys = keys_at("Groovy", source, ["doc", "dependencies", "implementation", "'org"]);

    // then
    assert_eq!(keys, [Some("Groovydoc comment"), Some("Method call"), Some("Method call"), Some("String")]);
}
