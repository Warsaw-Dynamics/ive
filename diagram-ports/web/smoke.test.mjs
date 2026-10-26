import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { Graphviz } from "./pkg/graphviz.js";

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const graphviz = await Graphviz.load();
const module = fs.readFileSync(new URL("./pkg/diagram_web.wasm", import.meta.url));
let wasm;

function dotLayout(pointer, length) {
  let text;
  let error = false;
  try {
    const source = decoder.decode(new Uint8Array(wasm.memory.buffer, pointer, length));
    text = graphviz.layout(source, "json0", "dot");
  } catch (cause) {
    text = String(cause);
    error = true;
  }
  const bytes = encoder.encode(text);
  const handle = wasm.diagram_output_alloc(bytes.length, error);
  assert.notEqual(handle, 0);
  new Uint8Array(wasm.memory.buffer, wasm.diagram_output_data(handle), bytes.length).set(bytes);
  return handle;
}

wasm = (await WebAssembly.instantiate(module, {
  env: { diagram_dot_layout: dotLayout },
})).instance.exports;

function render(source, format) {
  const bytes = encoder.encode(source);
  const pointer = wasm.diagram_alloc(bytes.length);
  assert.notEqual(pointer, 0);
  let handle = 0;
  try {
    new Uint8Array(wasm.memory.buffer, pointer, bytes.length).set(bytes);
    handle = format === "png"
      ? wasm.diagram_render_png(pointer, bytes.length)
      : wasm.diagram_render_svg(pointer, bytes.length);
    assert.notEqual(handle, 0);
    const result = new Uint8Array(
      wasm.memory.buffer,
      wasm.diagram_output_data(handle),
      wasm.diagram_output_len(handle),
    ).slice();
    return { data: result, error: Boolean(wasm.diagram_output_is_error(handle)) };
  } finally {
    if (handle !== 0) wasm.diagram_output_free(handle);
    wasm.diagram_free(pointer, bytes.length);
  }
}

test("should_release_source_buffers_at_different_sizes", () => {
  // given
  const lengths = [1, 2, 3, 16, 20, 32, 33, 34, 35, 100];

  // when
  for (const length of lengths) {
    const pointer = wasm.diagram_alloc(length);
    assert.notEqual(pointer, 0);
    wasm.diagram_free(pointer, length);
  }

  // then
  assert.ok(wasm.diagram_alloc(256 * 1024 + 1) === 0);
});

test("should_render_every_plantuml_fixture_to_svg", () => {
  // given
  const directory = fileURLToPath(new URL("../fixtures/plantuml/", import.meta.url));
  const fixtures = fs.readdirSync(directory).filter((name) => name.endsWith(".puml"));
  assert.equal(fixtures.length, 58);

  // when
  const results = fixtures.map((name) => [name, render(fs.readFileSync(path.join(directory, name), "utf8"), "svg")]);

  // then
  for (const [name, result] of results) {
    assert.equal(result.error, false, `${name}: ${decoder.decode(result.data)}`);
    assert.match(decoder.decode(result.data), /<svg\b/, name);
  }
});

test("should_render_dot_graph_to_png_and_report_invalid_source", () => {
  // given
  const valid = "@startuml\nclass Customer\nclass Order\nCustomer --> Order\n@enduml";
  const invalid = "@startuml\nAlice ->\n@enduml";

  // when
  const png = render(valid, "png");
  const failed = render(invalid, "svg");

  // then
  assert.equal(png.error, false);
  assert.deepEqual(Array.from(png.data.subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10]);
  assert.equal(failed.error, true);
  assert.match(decoder.decode(failed.data), /invalid diagram source/);
});
