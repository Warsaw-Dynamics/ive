import { Graphviz } from "./pkg/graphviz.js";

const decoder = new TextDecoder("utf-8", { fatal: true });
const encoder = new TextEncoder();
const MAX_SOURCE_BYTES = 256 * 1024;
const MAX_SVG_BYTES = 16 * 1024 * 1024;

let wasm;
let graphviz;

function dotLayout(pointer, length) {
  try {
    if (length > 2 * 1024 * 1024) {
      throw new Error(`DOT input has ${length} bytes, exceeding 2 MiB`);
    }
    const source = decoder.decode(new Uint8Array(wasm.memory.buffer, pointer, length));
    const json = graphviz.layout(source, "json0", "dot");
    return dotResult(json, false);
  } catch (error) {
    return dotResult(String(error?.message ?? error).slice(0, 64 * 1024), true);
  }
}

function dotResult(text, error) {
  const bytes = encoder.encode(text);
  if (bytes.length === 0 || bytes.length > 8 * 1024 * 1024) {
    return 0;
  }
  const handle = wasm.diagram_output_alloc(bytes.length, error);
  if (handle === 0) return 0;
  const pointer = wasm.diagram_output_data(handle);
  new Uint8Array(wasm.memory.buffer, pointer, bytes.length).set(bytes);
  return handle;
}

async function initialize(url) {
  graphviz = await Graphviz.load();
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`cannot fetch diagram WASM ${url}: HTTP ${response.status}`);
  }
  const bytes = await response.arrayBuffer();
  const module = await WebAssembly.instantiate(bytes, {
    env: { diagram_dot_layout: dotLayout },
  });
  wasm = module.instance.exports;
  if (!(wasm.memory instanceof WebAssembly.Memory)) {
    throw new Error("diagram WASM does not export memory");
  }
}

function renderDiagram(source, format) {
  if (typeof source !== "string") {
    throw new TypeError("diagram source must be a string");
  }
  const input = encoder.encode(source);
  if (input.length === 0 || input.length > MAX_SOURCE_BYTES) {
    throw new Error(`diagram source must have 1..${MAX_SOURCE_BYTES} UTF-8 bytes`);
  }
  const pointer = wasm.diagram_alloc(input.length);
  if (pointer === 0) {
    throw new Error(`cannot allocate ${input.length} diagram source bytes`);
  }
  let output = 0;
  try {
    new Uint8Array(wasm.memory.buffer, pointer, input.length).set(input);
    output = format === "png"
      ? wasm.diagram_render_png(pointer, input.length)
      : wasm.diagram_render_svg(pointer, input.length);
    if (output === 0) {
      throw new Error("diagram WASM returned no result");
    }
    const length = wasm.diagram_output_len(output);
    if (length > MAX_SVG_BYTES) {
      throw new Error(`diagram WASM returned ${length} bytes, exceeding ${MAX_SVG_BYTES}`);
    }
    const data = new Uint8Array(
      wasm.memory.buffer,
      wasm.diagram_output_data(output),
      length,
    ).slice();
    if (wasm.diagram_output_is_error(output)) {
      throw new Error(decoder.decode(data));
    }
    return format === "png" ? data : decoder.decode(data);
  } finally {
    if (output !== 0) wasm.diagram_output_free(output);
    wasm.diagram_free(pointer, input.length);
  }
}

self.onmessage = async ({ data }) => {
  const { id, type } = data;
  try {
    if (type === "init") {
      await initialize(data.url);
      self.postMessage({ id, ok: true });
      return;
    }
    if (type !== "render" || wasm === undefined) {
      throw new Error("diagram worker is not initialized");
    }
    if (data.format !== "svg" && data.format !== "png") {
      throw new Error(`unsupported diagram format ${JSON.stringify(data.format)}`);
    }
    const result = renderDiagram(data.source, data.format);
    if (data.format === "svg") {
      self.postMessage({ id, ok: true, format: "svg", data: result });
      return;
    }
    self.postMessage({ id, ok: true, format: "png", data: result }, [result.buffer]);
  } catch (error) {
    self.postMessage({ id, ok: false, error: String(error?.message ?? error) });
  }
};

