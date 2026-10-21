use super::{keys_at, parsed};

#[test]
fn should_tell_vue_component_and_directive_parts_from_attributes() {
    // given
    let source = "<template>\n  <MyList v-if=\"ok\" class=\"x\" :title=\"t\" @click.stop=\"go()\" v-bind:[name]=\"v\">\n    <li>a</li>\n  </MyList>\n</template>";

    // when
    let keys = keys_at("Vue", source, ["MyList", "li>", "v-if", "class", "\"x\"", "title", "click", "stop", "name"]);

    // then
    assert_eq!(
        keys,
        [
            Some("VUE.COMPONENT_NAME"),
            Some("VUE.TAG_NAME"),
            Some("VUE.DIRECTIVE"),
            Some("VUE.ATTRIBUTE_NAME"),
            Some("VUE.ATTRIBUTE_VALUE"),
            Some("VUE.DIRECTIVE_ARGUMENT"),
            Some("VUE.DIRECTIVE_ARGUMENT"),
            Some("VUE.DIRECTIVE_MODIFIER"),
            Some("VUE.DYNAMIC_ARGUMENT"),
        ]
    );
}

#[test]
fn should_colour_vue_typescript_script_in_typescript() {
    // given
    let source = "<script setup lang=\"ts\">\ninterface Item { id: number }\nconst count: number = 1;\n</script>";

    // when
    let syntax = parsed("Vue", source);
    let keys = keys_at("Vue", source, ["Item", "number", "const", "1;"]);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Vue", 0), ("TypeScript", 1)]);
    assert_eq!(keys, [Some("TS.INTERFACE"), Some("TS.PRIMITIVE_TYPE"), Some("JS.KEYWORD"), Some("JS.NUMBER")]);
}

#[test]
fn should_colour_vue_template_expressions_as_code_and_style_in_scss() {
    // given
    let source = "<template><p :class=\"active\" @click=\"count++\">{{ total(2) }}</p></template>\n<style lang=\"scss\">$gap: 4px;</style>";

    // when
    let keys = keys_at("Vue", source, ["active", "++", "{{", "total", "2", "$gap"]);

    // then
    assert_eq!(
        keys,
        [
            Some("TEXT"),
            Some("JS.OPERATION_SIGN"),
            Some("VUE.INTERPOLATION"),
            Some("JS.FUNCTION_CALL"),
            Some("JS.NUMBER"),
            Some("SASS_VARIABLE"),
        ]
    );
}
