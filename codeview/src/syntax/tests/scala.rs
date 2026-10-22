use super::keys_at;

#[test]
fn should_tell_scala_method_declaration_from_function_and_method_call() {
    // given
    let source = "object O {\n  def run(p: Int): Unit = { start(p); list.add(1) }\n}";

    // when
    let keys = keys_at("Scala", source, ["run", "start", "add", "p:", "O"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SCALA.FUNCTION_DECLARATION"),
            Some("SCALA.FUNCTION_CALL"),
            Some("SCALA.METHOD_CALL"),
            Some("SCALA.PARAMETER"),
            Some("SCALA.OBJECT"),
        ]
    );
}

#[test]
fn should_tell_scala_field_from_local_value_and_constant() {
    // given
    let source = "class A {\n  val size = 1\n  final val MAX_SIZE = 2\n  def f() = { val local = size }\n}";

    // when
    let keys = keys_at("Scala", source, ["size", "MAX_SIZE", "local"]);

    // then
    assert_eq!(keys, [Some("SCALA.FIELD"), Some("SCALA.CONSTANT"), Some("SCALA.LOCAL_VARIABLE")]);
}

#[test]
fn should_keep_scala_interpolated_code_out_of_string_colour() {
    // given
    let source = "val s = s\"name $name and ${count + 1}\\n\"";

    // when
    let keys = keys_at("Scala", source, ["name ", "$name", "count", "\\n"]);

    // then
    assert_eq!(keys, [Some("SCALA.STRING"), Some("SCALA.INTERPOLATION"), Some("TEXT"), Some("SCALA.ESCAPE")]);
}

#[test]
fn should_tell_scaladoc_from_block_comment_and_trait_from_class() {
    // given
    let source = "/** doc */\ntrait T\n/* block */\n@deprecated class C extends T";

    // when
    let keys = keys_at("Scala", source, ["doc", "T\n", "block", "deprecated", "C "]);

    // then
    assert_eq!(
        keys,
        [
            Some("SCALA.DOC_COMMENT"),
            Some("SCALA.TRAIT"),
            Some("SCALA.BLOCK_COMMENT"),
            Some("SCALA.ANNOTATION"),
            Some("SCALA.TYPE"),
        ]
    );
}
