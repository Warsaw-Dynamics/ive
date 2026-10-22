use super::{keys_at, parsed};

#[test]
fn should_tell_svelte_block_keyword_directive_and_component_from_markup() {
    // given
    let source = "{#each rows as row}\n  <Card on:click={open} title=\"t\">&amp;</Card>\n  <p>x</p>\n{/each}\n{#snippet cell(v)}{/snippet}";

    // when
    let keys = keys_at("Svelte", source, ["{#", "each", "as", "Card", "p>", "on:click", "title", "\"t\"", "&amp;", "cell"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SVELTE.DELIMITER"),
            Some("SVELTE.BLOCK_KEYWORD"),
            Some("SVELTE.BLOCK_KEYWORD"),
            Some("SVELTE.COMPONENT_NAME"),
            Some("SVELTE.TAG_NAME"),
            Some("SVELTE.DIRECTIVE"),
            Some("SVELTE.ATTRIBUTE_NAME"),
            Some("SVELTE.ATTRIBUTE_VALUE"),
            Some("SVELTE.ENTITY_REFERENCE"),
            Some("SVELTE.SNIPPET_NAME"),
        ]
    );
}

#[test]
fn should_colour_svelte_script_style_and_expression_in_their_languages() {
    // given
    let source = "<script lang=\"ts\">\n  let count: number = 0;\n</script>\n<button on:click={() => count++}>{label(7)}</button>\n<style>\n  p { color: red; }\n</style>";

    // when
    let syntax = parsed("Svelte", source);
    let keys = keys_at("Svelte", source, ["let", "number", "=>", "label", "7", "color"]);

    // then
    let mut layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    layers.sort();
    assert_eq!(layers, [("CSS", 1), ("Svelte", 0), ("TypeScript", 1), ("TypeScript", 1), ("TypeScript", 1)]);
    assert_eq!(
        keys,
        [
            Some("JS.KEYWORD"),
            Some("TS.PRIMITIVE_TYPE"),
            Some("JS.OPERATION_SIGN"),
            Some("JS.FUNCTION_CALL"),
            Some("JS.NUMBER"),
            Some("CSS.PROPERTY_NAME"),
        ]
    );
}
