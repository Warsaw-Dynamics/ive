/**
 * @file Rune (https://rune-rs.github.io): tree-sitter-rust plus the syntax Rune adds to Rust.
 */

const rust = require('./rust');

module.exports = grammar(rust, {
  name: 'rune',

  rules: {
    _expression_except_range: ($, original) => choice(
      original,
      $.object_expression,
      $.template_string,
      $.is_expression,
      $.select_expression,
    ),

    // macro arguments are token trees, which a template string may appear in: assert_eq!(`${a}`, "x")
    _tokens: ($, original) => choice(original, $.template_string),
    _delim_tokens: ($, original) => choice(original, $.template_string),

    // Rune has no types: fn init(config) takes a pattern, where Rust would read an anonymous parameter's type
    parameters: $ => seq(
      '(',
      sepBy(',', seq(
        optional($.attribute_item),
        choice($.self_parameter, $._pattern),
      )),
      optional(','),
      ')',
    ),

    // #{ "a": 1, b: 2 }
    object_expression: $ => seq('#{', sepBy(',', $.object_entry), optional(','), '}'),

    object_entry: $ => seq(
      field('key', choice($.identifier, $.string_literal)),
      ':',
      field('value', $._expression),
    ),

    // `hello ${name}`
    template_string: $ => seq(
      '`',
      repeat(choice($.template_content, $.escape_sequence, $.template_substitution)),
      token.immediate('`'),
    ),

    template_content: _ => token.immediate(/([^`$\\]|\$[^{`\\])+/),

    template_substitution: $ => seq(token.immediate('${'), $._expression, '}'),

    // value is Type, value is not Type
    is_expression: $ => prec.left(4, seq(
      field('left', $._expression),
      choice('is', seq('is', 'not')),
      field('right', $._expression),
    )),

    // select { value = future => body, }
    select_expression: $ => seq('select', '{', sepBy(',', $.select_arm), optional(','), '}'),

    select_arm: $ => choice(
      seq(field('pattern', $._pattern), '=', field('future', $._expression), '=>', field('body', $._expression)),
      seq('default', '=>', field('body', $._expression)),
    ),

    _pattern: ($, original) => choice(original, $.object_pattern),

    // #{ a, "b": 2, .. }
    object_pattern: $ => seq(
      '#{',
      sepBy(',', choice($.identifier, $.object_pattern_entry, $.remaining_field_pattern)),
      optional(','),
      '}',
    ),

    object_pattern_entry: $ => seq(
      field('key', choice($.identifier, $.string_literal)),
      ':',
      field('pattern', $._pattern),
    ),

    // Rune has no types: struct fields and constants come without them
    field_declaration: $ => seq(
      optional($.visibility_modifier),
      field('name', $._field_identifier),
      optional(seq(':', field('type', $._type))),
    ),

    const_item: $ => seq(
      optional($.visibility_modifier),
      'const',
      field('name', $.identifier),
      optional(seq(':', field('type', $._type))),
      optional(seq('=', field('value', $._expression))),
      ';',
    ),
  },
});

function sepBy(sep, rule) {
  return optional(seq(rule, repeat(seq(sep, rule))));
}
