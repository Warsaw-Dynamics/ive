use super::keys_at;

#[test]
fn should_tell_glsl_builtins_from_user_functions_and_variables() {
    // given
    let source = "float shade(vec3 n) { return n.x; }\nvoid main() { float k = shade(normalize(gl_Position.xyz)); }";

    // when
    let keys = keys_at("GLSL", source, ["shade", "shade(normalize", "normalize", "gl_Position", "k ="]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLSL.FUNCTION_DECLARATION"),
            Some("GLSL.FUNCTION_CALL"),
            Some("GLSL.BUILTIN_FUNCTION"),
            Some("GLSL.BUILTIN_VARIABLE"),
            Some("GLSL.LOCAL_VARIABLE")
        ]
    );
}

#[test]
fn should_mark_glsl_qualifiers_layout_and_types() {
    // given
    let source = "layout(location = 0) in vec3 position;\nuniform sampler2D albedo;\nout Light light;";

    // when
    let keys = keys_at("GLSL", source, ["layout", "location", "in ", "uniform", "sampler2D", "Light", "albedo"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLSL.QUALIFIER"),
            Some("GLSL.LAYOUT_QUALIFIER"),
            Some("GLSL.QUALIFIER"),
            Some("GLSL.QUALIFIER"),
            Some("GLSL.PRIMITIVE_TYPE"),
            Some("GLSL.TYPE"),
            Some("GLSL.GLOBAL_VARIABLE")
        ]
    );
}

#[test]
fn should_mark_glsl_preprocessor_macro_and_constant() {
    // given
    let source = "#version 450\n#extension GL_EXT_foo : require\n#define COUNT 4\nfloat v[COUNT];";

    // when
    let keys = keys_at("GLSL", source, ["#version", "GL_EXT_foo", "require", "#define", "COUNT", "COUNT]"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLSL.PREPROCESSOR_DIRECTIVE"),
            Some("GLSL.MACRO"),
            Some("GLSL.KEYWORD"),
            Some("GLSL.PREPROCESSOR_DIRECTIVE"),
            Some("GLSL.MACRO"),
            Some("GLSL.CONSTANT")
        ]
    );
}
