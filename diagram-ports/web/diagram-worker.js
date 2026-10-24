const decoder = new TextDecoder("utf-8", { fatal: true });
const encoder = new TextEncoder();
const MAX_SOURCE_BYTES = 256 * 1024;
const MAX_SVG_BYTES = 16 * 1024 * 1024;

let wasm;

async function initialize(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`cannot fetch diagram WASM ${url}: HTTP ${response.status}`);
  }
  const bytes = await response.arrayBuffer();
  const module = await WebAssembly.instantiate(bytes);
  wasm = module.instance.exports;
  if (!(wasm.memory instanceof WebAssembly.Memory)) {
    throw new Error("diagram WASM does not export memory");
  }
}

function renderDiagram(source, format, language) {
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
    const render = language === "dot"
      ? { svg: wasm.diagram_render_dot_svg, png: wasm.diagram_render_dot_png, json: wasm.diagram_render_dot_json }[format]
      : { svg: wasm.diagram_render_svg, png: wasm.diagram_render_png }[format];
    output = render(pointer, input.length);
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
    if (data.format !== "svg" && data.format !== "png" && data.format !== "json") {
      throw new Error(`unsupported diagram format ${JSON.stringify(data.format)}`);
    }
    if (data.language !== "dot" && data.language !== "plantuml") {
      throw new Error(`unsupported diagram language ${JSON.stringify(data.language)}`);
    }
    if (data.format === "json" && data.language !== "dot") {
      throw new Error("JSON geometry is available for DOT sources only");
    }
    const result = renderDiagram(data.source, data.format, data.language);
    if (data.format !== "png") {
      self.postMessage({ id, ok: true, format: data.format, data: result });
      return;
    }
    self.postMessage({ id, ok: true, format: "png", data: result }, [result.buffer]);
  } catch (error) {
    self.postMessage({ id, ok: false, error: String(error?.message ?? error) });
  }
};

