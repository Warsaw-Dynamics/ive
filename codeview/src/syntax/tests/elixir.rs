use super::{keys_at, parsed};

#[test]
fn should_tell_elixir_function_declaration_from_calls_and_parameter() {
    // given
    let source = "defmodule Shop.Cart do\n  def total(items, rate \\\\ 1) do\n    items |> Enum.map(&price/1) |> sum()\n  end\nend";

    // when
    let keys = keys_at("Elixir", source, ["defmodule", "Shop", "total", "items,", "rate", "|>", "map", "price", "sum"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELIXIR.KEYWORD"),
            Some("ELIXIR.MODULE"),
            Some("ELIXIR.FUNCTION_DECLARATION"),
            Some("ELIXIR.PARAMETER"),
            Some("ELIXIR.PARAMETER"),
            Some("ELIXIR.PIPE"),
            Some("ELIXIR.REMOTE_CALL"),
            Some("ELIXIR.FUNCTION_CALL"),
            Some("ELIXIR.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_elixir_doc_string_from_module_attribute_atom_and_field() {
    // given
    let source = "@doc \"Adds one.\"\n@limit 5\ndef add(user) when is_map(user), do: {:ok, user.age + @limit}";

    // when
    let keys = keys_at("Elixir", source, ["Adds", "@limit", ":ok", "age", "user.", "do:", "when"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELIXIR.DOC_COMMENT"),
            Some("ELIXIR.MODULE_ATTRIBUTE"),
            Some("ELIXIR.ATOM"),
            Some("ELIXIR.FIELD"),
            Some("ELIXIR.VARIABLE"),
            Some("ELIXIR.KEYWORD"),
            Some("ELIXIR.KEYWORD"),
        ]
    );
}

#[test]
fn should_mark_elixir_sigil_interpolation_and_escape() {
    // given
    let source = "words = ~w(a b)\nlabel = \"#{words}\\n\"";

    // when
    let keys = keys_at("Elixir", source, ["~w", "a b", "#{", "words}", "\\n"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELIXIR.SIGIL"),
            Some("ELIXIR.SIGIL"),
            Some("ELIXIR.INTERPOLATION"),
            Some("ELIXIR.VARIABLE"),
            Some("ELIXIR.ESCAPE"),
        ]
    );
}

#[test]
fn should_colour_heex_inside_an_elixir_h_sigil_and_elixir_inside_it() {
    // given
    let source =
        "def card(assigns) do\n  ~H\"\"\"\n  <section class=\"card\">{format(@title)}</section>\n  \"\"\"\nend";

    // when
    let keys = keys_at("Elixir", source, ["~H", "section", "class", "format", "@title"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELIXIR.SIGIL"),
            Some("HEEX.TAG_NAME"),
            Some("HEEX.ATTRIBUTE_NAME"),
            Some("ELIXIR.FUNCTION_CALL"),
            Some("ELIXIR.MODULE_ATTRIBUTE"),
        ]
    );
}

#[test]
fn should_nest_elixir_inside_heex_inside_elixir() {
    // given
    let source = "def card(assigns) do\n  ~H\"\"\"\n  <p>{@title}</p>\n  \"\"\"\nend";

    // when
    let syntax = parsed("Elixir", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Elixir", 0), ("HEEx", 1), ("Elixir", 2)]);
}

#[test]
fn should_colour_an_elixir_r_sigil_as_one_regular_expression() {
    // given
    let source = "@email ~r/^\\w+$/";

    // when
    let layers: Vec<_> =
        parsed("Elixir", source).layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    let keys = keys_at("Elixir", source, ["~r", "^", "\\w"]);

    // then
    assert_eq!(layers, [("Elixir", 0), ("RegExp", 1)]);
    assert_eq!(keys, [Some("ELIXIR.SIGIL"), Some("REGEXP.META"), Some("REGEXP.CHAR_CLASS")]);
}
