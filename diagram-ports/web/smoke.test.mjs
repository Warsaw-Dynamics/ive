import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { inflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const module = fs.readFileSync(new URL("./pkg/diagram_web.wasm", import.meta.url));
let wasm;

wasm = (await WebAssembly.instantiate(module)).instance.exports;

function render(source, format, language = "plantuml") {
  const bytes = encoder.encode(source);
  const pointer = wasm.diagram_alloc(bytes.length);
  assert.notEqual(pointer, 0);
  let handle = 0;
  try {
    new Uint8Array(wasm.memory.buffer, pointer, bytes.length).set(bytes);
    const renderInput = language === "dot"
      ? { svg: wasm.diagram_render_dot_svg, png: wasm.diagram_render_dot_png, json: wasm.diagram_render_dot_json }[format]
      : { svg: wasm.diagram_render_svg, png: wasm.diagram_render_png }[format];
    handle = renderInput(pointer, bytes.length);
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

function pngRgb(data) {
  assert.deepEqual(Array.from(data.subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10]);
  let width = 0;
  let height = 0;
  let color = 0;
  let bitDepth = 0;
  let palette;
  const compressed = [];
  for (let offset = 8; offset + 12 <= data.length;) {
    const length = data.readUInt32BE(offset);
    const type = data.toString("ascii", offset + 4, offset + 8);
    const chunk = data.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") {
      width = chunk.readUInt32BE(0);
      height = chunk.readUInt32BE(4);
      bitDepth = chunk[8];
      color = chunk[9];
      assert.ok(color === 2 && bitDepth === 8 || color === 3 && [1, 2, 4, 8].includes(bitDepth));
    }
    if (type === "PLTE") palette = chunk;
    if (type === "IDAT") compressed.push(chunk);
    offset += length + 12;
    if (type === "IEND") break;
  }
  assert.ok(width > 0 && height > 0);
  const channels = color === 2 ? 3 : 1;
  const stride = color === 2 ? width * 3 : Math.ceil(width * bitDepth / 8);
  const raw = inflateSync(Buffer.concat(compressed));
  const rgb = Buffer.alloc(width * height * 3);
  let previous = Buffer.alloc(stride);
  let offset = 0;
  for (let y = 0; y < height; y++) {
    const filter = raw[offset++];
    const row = Buffer.alloc(stride);
    for (let x = 0; x < stride; x++) {
      const left = x >= channels ? row[x - channels] : 0;
      const above = previous[x];
      const upperLeft = x >= channels ? previous[x - channels] : 0;
      const prediction = left + above - upperLeft;
      const distances = [Math.abs(prediction - left), Math.abs(prediction - above), Math.abs(prediction - upperLeft)];
      const paeth = distances[0] <= distances[1] && distances[0] <= distances[2] ? left
        : distances[1] <= distances[2] ? above : upperLeft;
      const predictor = [0, left, above, Math.floor((left + above) / 2), paeth][filter];
      assert.notEqual(predictor, undefined);
      row[x] = (raw[offset++] + predictor) & 255;
    }
    for (let x = 0; x < width; x++) {
      const target = (y * width + x) * 3;
      if (color === 2) row.copy(rgb, target, x * 3, x * 3 + 3);
      else {
        const bit = x * bitDepth;
        const shift = 8 - bitDepth - bit % 8;
        const index = row[Math.floor(bit / 8)] >> shift & ((1 << bitDepth) - 1);
        palette.copy(rgb, target, index * 3, index * 3 + 3);
      }
    }
    previous = row;
  }
  return { width, height, rgb };
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

test("should_render_standalone_dot_to_svg_png_and_json", () => {
  // given
  const source = "digraph G { a -> b }";

  // when
  const svg = render(source, "svg", "dot");
  const png = render(source, "png", "dot");
  const geometry = render(source, "json", "dot");

  // then
  assert.equal(svg.error, false, decoder.decode(svg.data));
  assert.match(decoder.decode(svg.data), /<svg\b/);
  assert.equal(png.error, false, decoder.decode(png.data));
  assert.deepEqual(Array.from(png.data.subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10]);
  assert.equal(geometry.error, false, decoder.decode(geometry.data));
  assert.ok(JSON.parse(decoder.decode(geometry.data)).objects.length >= 2);
});

test("should_match_native_graphviz_gd_chain_pixels_in_wasm", () => {
  // given
  const source = fs.readFileSync(new URL("../tools/dot-parity/fixtures/chain.dot", import.meta.url), "utf8");
  const oracle = fs.readFileSync(new URL("../engine/src/dot_runtime/goldens/chain-gd.png", import.meta.url));

  // when
  const rendered = render(source, "png", "dot");

  // then
  assert.equal(rendered.error, false, decoder.decode(rendered.data));
  const expected = pngRgb(oracle);
  const actual = pngRgb(Buffer.from(rendered.data));
  assert.deepEqual(actual, expected);
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

test("should_render_rust_graph_to_png_and_report_invalid_source", () => {
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
