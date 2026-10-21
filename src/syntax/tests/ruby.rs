use super::{keys_at, parsed};

#[test]
fn should_tell_ruby_method_declaration_from_calls_and_variables() {
    // given
    let source = "class Book\n  def add(order)\n    @items << order\n    @@count += 1\n    log(order.id)\n  end\nend";

    // when
    let keys = keys_at("Ruby", source, ["Book", "add", "(order", "@items", "@@count", "log", "id"]);

    // then
    assert_eq!(
        keys,
        [
            Some("RUBY.CLASS"),
            Some("RUBY.METHOD_DECLARATION"),
            Some("RUBY.PARENTHESES"),
            Some("RUBY.INSTANCE_VARIABLE"),
            Some("RUBY.CLASS_VARIABLE"),
            Some("RUBY.FUNCTION_CALL"),
            Some("RUBY.METHOD_CALL"),
        ]
    );
}

#[test]
fn should_mark_ruby_symbol_constant_global_and_interpolation() {
    // given
    let source = "LIMIT = 5\n$out.puts(:done, \"n=#{LIMIT}\")\ntext = <<~EOS\n  hi\nEOS";

    // when
    let keys = keys_at("Ruby", source, ["LIMIT", "$out", ":done", "#{", "text", "<<~EOS", "hi"]);

    // then
    assert_eq!(
        keys,
        [
            Some("RUBY.CONSTANT"),
            Some("RUBY.GLOBAL_VARIABLE"),
            Some("RUBY.SYMBOL"),
            Some("RUBY.INTERPOLATION"),
            Some("RUBY.LOCAL_VARIABLE"),
            Some("RUBY.HEREDOC_ID"),
            Some("RUBY.HEREDOC_CONTENT"),
        ]
    );
}

#[test]
fn should_parse_ruby_regex_literal_as_regular_expression() {
    // given
    let source = "ok = name =~ /\\d+[a-z]/";

    // when
    let syntax = parsed("Ruby", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Ruby", 0), ("RegExp", 1)]);
}
