#!/usr/bin/env node
/** Export one standalone DOT artifact through the browser WASM ABI in Node. */

import fs from "node:fs";
import path from "node:path";

const MAX_SOURCE = 256 * 1024;
const MAX_WASM = 64 * 1024 * 1024;
const MAX_OUTPUT = 16 * 1024 * 1024;
const MAX_PIXELS = 4_194_304;
const PNG_MAGIC = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

function argumentsForRun(argv) {
  if (argv.length !== 6) throw new Error("expected --wasm FILE --input FILE --output FILE");
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    if (!["--wasm", "--input", "--output"].includes(option) || values.has(option)) {
      throw new Error(`unknown or repeated option ${option}`);
    }
    values.set(option, argv[index + 1]);
  }
  if (values.size !== 3) throw new Error("--wasm, --input and --output are required");
  const format = path.extname(values.get("--output")).slice(1).toLowerCase();
  if (!["json", "svg", "png"].includes(format)) throw new Error(`unsupported WASM output format ${format}`);
  return { wasm: values.get("--wasm"), input: values.get("--input"), output: values.get("--output"), format };
}

function readBounded(file, maximum) {
  const size = fs.statSync(file).size;
  if (size < 1 || size > maximum) throw new Error(`${file} size ${size} is outside 1..${maximum}`);
  const data = fs.readFileSync(file);
  if (data.length !== size || data.length > maximum) throw new Error(`${file} changed size while being read`);
  return data;
}

function checkRange(memory, pointer, length, label) {
  if (!Number.isSafeInteger(pointer) || !Number.isSafeInteger(length) ||
      pointer <= 0 || length < 0 || length > MAX_OUTPUT || pointer + length > memory.buffer.byteLength) {
    throw new Error(`${label} pointer ${pointer} and length ${length} exceed WASM memory bounds`);
  }
}

function checkPng(data) {
  if (data.length < 24 || !data.subarray(0, 8).equals(PNG_MAGIC)) throw new Error("WASM output is not PNG");
  const width = data.readUInt32BE(16);
  const height = data.readUInt32BE(20);
  if (width < 1 || height < 1 || width > 4096 || height > 4096 || width * height > MAX_PIXELS) {
    throw new Error(`WASM PNG dimensions ${width}x${height} exceed raster bounds`);
  }
}

async function run(options) {
  const source = readBounded(options.input, MAX_SOURCE);
  const module = readBounded(options.wasm, MAX_WASM);
  const exports = (await WebAssembly.instantiate(module)).instance.exports;
  if (!(exports.memory instanceof WebAssembly.Memory)) throw new Error("browser WASM does not export memory");
  const render = {
    json: exports.diagram_render_dot_json,
    svg: exports.diagram_render_dot_svg,
    png: exports.diagram_render_dot_png,
  }[options.format];
  if (typeof render !== "function") throw new Error(`browser WASM lacks DOT ${options.format} export`);
  const pointer = exports.diagram_alloc(source.length);
  if (pointer === 0) throw new Error(`browser WASM refused ${source.length} source bytes`);
  let handle = 0;
  try {
    checkRange(exports.memory, pointer, source.length, "source");
    new Uint8Array(exports.memory.buffer, pointer, source.length).set(source);
    handle = render(pointer, source.length);
    if (handle === 0) throw new Error("browser WASM returned a null result handle");
    const length = exports.diagram_output_len(handle);
    const dataPointer = exports.diagram_output_data(handle);
    checkRange(exports.memory, dataPointer, length, "result");
    const data = Buffer.from(new Uint8Array(exports.memory.buffer, dataPointer, length));
    if (exports.diagram_output_is_error(handle)) throw new Error(`browser WASM DOT error: ${data.toString("utf8").slice(0, 1000)}`);
    if (options.format === "png") checkPng(data);
    if (options.format === "json") JSON.parse(data.toString("utf8"));
    fs.writeFileSync(options.output, data);
  } finally {
    if (handle !== 0) exports.diagram_output_free(handle);
    exports.diagram_free(pointer, source.length);
  }
}

run(argumentsForRun(process.argv.slice(2))).catch((error) => {
  process.stderr.write(`run_wasm: ${error.message}\n`);
  process.exitCode = 1;
});
