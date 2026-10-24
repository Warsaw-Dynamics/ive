import assert from "node:assert/strict";
import test from "node:test";
import { createDiagramRenderer } from "./diagram.js";

test("should_start_render_deadline_when_queued_request_reaches_worker", async () => {
  // given
  const originalWorker = globalThis.Worker;
  const originalSetTimeout = globalThis.setTimeout;
  const originalClearTimeout = globalThis.clearTimeout;
  const timers = new Map();
  let worker;
  let renderer;
  class FakeWorker {
    constructor() {
      this.messages = [];
      worker = this;
    }
    postMessage(message) {
      this.messages.push(message);
      if (message.type === "init") {
        queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
      }
    }
    respond(message, data) {
      this.onmessage({ data: { id: message.id, ok: true, data } });
    }
    terminate() {}
  }
  globalThis.Worker = FakeWorker;
  globalThis.setTimeout = (callback, duration) => {
    const id = Symbol();
    timers.set(id, { callback, duration });
    return id;
  };
  globalThis.clearTimeout = (id) => timers.delete(id);
  try {
    renderer = await createDiagramRenderer();
    // when
    const first = renderer.render("first", { language: "dot" });
    const second = renderer.render("second", { language: "dot" });
    await Promise.resolve();
    let renders = worker.messages.filter((message) => message.type === "render");
    // then
    assert.equal(renders.length, 1);
    assert.deepEqual(Array.from(timers.values()).map((timer) => timer.duration), [10_000]);

    // when
    worker.respond(renders[0], "first result");
    assert.equal(await first, "first result");
    await Promise.resolve();
    renders = worker.messages.filter((message) => message.type === "render");
    // then
    assert.equal(renders.length, 2);
    assert.deepEqual(Array.from(timers.values()).map((timer) => timer.duration), [10_000]);
    worker.respond(renders[1], "second result");
    assert.equal(await second, "second result");
    assert.equal(timers.size, 0);
  } finally {
    renderer?.close();
    globalThis.Worker = originalWorker;
    globalThis.setTimeout = originalSetTimeout;
    globalThis.clearTimeout = originalClearTimeout;
  }
});
