/** Starts a diagram renderer in a dedicated Web Worker. */
export async function createDiagramRenderer(options = {}) {
  const wasmUrl = new URL(options.wasmUrl ?? "./pkg/diagram_web.wasm", import.meta.url);
  const worker = new Worker(new URL("./diagram-worker.js", import.meta.url), { type: "module" });
  const pending = new Map();
  let nextId = 0;
  let closed = false;

  function rejectAll(error) {
    for (const request of pending.values()) {
      clearTimeout(request.timer);
      request.reject(error);
    }
    pending.clear();
  }

  worker.onmessage = ({ data }) => {
    const request = pending.get(data.id);
    if (request === undefined) return;
    pending.delete(data.id);
    clearTimeout(request.timer);
    if (data.ok) request.resolve(data);
    else request.reject(new Error(data.error));
  };
  worker.onerror = (event) => {
    rejectAll(new Error(`diagram worker failed: ${event.message}`));
    worker.terminate();
    closed = true;
  };

  function send(message) {
    if (closed) return Promise.reject(new Error("diagram renderer is closed"));
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        closed = true;
        worker.terminate();
        rejectAll(new Error(`diagram worker exceeded its ${message.type === "init" ? 30 : 10}-second deadline`));
      }, message.type === "init" ? 30_000 : 10_000);
      pending.set(id, { resolve, reject, timer });
      worker.postMessage({ id, ...message });
    });
  }

  try {
    await send({ type: "init", url: wasmUrl.href });
  } catch (error) {
    worker.terminate();
    throw error;
  }
  return Object.freeze({
    /** Returns SVG, PNG bytes, or DOT geometry JSON. */
    async render(source, { format = "svg", language = "plantuml" } = {}) {
      if (format !== "svg" && format !== "png" && format !== "json") {
        throw new Error(`unsupported diagram format ${JSON.stringify(format)}`);
      }
      if (language !== "plantuml" && language !== "dot") {
        throw new Error(`unsupported diagram language ${JSON.stringify(language)}`);
      }
      if (format === "json" && language !== "dot") {
        throw new Error("JSON geometry is available for DOT sources only");
      }
      const result = await send({ type: "render", source, format, language });
      return result.data;
    },
    close() {
      if (closed) return;
      closed = true;
      worker.terminate();
      rejectAll(new Error("diagram renderer is closed"));
    },
  });
}
