use super::keys_at;

#[test]
fn should_tell_cpp_method_declaration_from_function_and_method_call() {
    // given
    let source = "class Line { void add(int n); };\nvoid Line::add(int n) { kinds.push_back(n); }\nint main() { run(); }";

    // when
    let keys = keys_at("C++", source, ["add", "push_back", "main", "run"]);

    // then
    assert_eq!(
        keys,
        [Some("CPP.METHOD_DECLARATION"), Some("CPP.METHOD_CALL"), Some("CPP.FUNCTION_DECLARATION"), Some("CPP.FUNCTION_CALL")]
    );
}

#[test]
fn should_mark_cpp_namespace_type_parameter_field_and_overloaded_operator() {
    // given
    let source = "namespace ive { template <typename T> struct Box { T value; bool operator==(const Box &) const; }; }";

    // when
    let keys = keys_at("C++", source, ["ive", "T>", "value", "operator=="]);

    // then
    assert_eq!(
        keys,
        [Some("CPP.NAMESPACE"), Some("CPP.TYPE_PARAMETER"), Some("CPP.FIELD"), Some("CPP.OVERLOADED_OPERATOR")]
    );
}

#[test]
fn should_mark_cpp_attribute_local_parameter_and_constant() {
    // given
    let source = "[[nodiscard]] int f(int count) { auto total = count * LIMIT; return total; }";

    // when
    let keys = keys_at("C++", source, ["nodiscard", "count", "total", "LIMIT"]);

    // then
    assert_eq!(keys, [Some("CPP.ATTRIBUTE"), Some("CPP.PARAMETER"), Some("CPP.LOCAL_VARIABLE"), Some("CPP.CONSTANT")]);
}
