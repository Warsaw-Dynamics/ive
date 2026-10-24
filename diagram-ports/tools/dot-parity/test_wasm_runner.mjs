import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const runner = fileURLToPath(new URL("./run_wasm.mjs", import.meta.url));
const wasm = fileURLToPath(new URL("../../target/wasm32-unknown-unknown/release/diagram_web.wasm", import.meta.url));
const chain = fileURLToPath(new URL("./fixtures/chain.dot", import.meta.url));

function invoke(input, output) {
  return spawnSync(process.execPath, [runner, "--wasm", wasm, "--input", input, "--output", output],
    { encoding: "utf8", timeout: 10000 });
}

test("should_render_all_dot_formats_through_the_browser_wasm_abi", () => {
  // given
  const folder = fs.mkdtempSync(path.join(os.tmpdir(), "dot-wasm-test-"));
  try {
    // when
    const results = ["json", "svg", "png"].map((format) => {
      const output = path.join(folder, `chain.${format}`);
      return { format, output, result: invoke(chain, output) };
    });
    // then
    for (const { result } of results) assert.equal(result.status, 0, result.stderr);
    assert.ok(JSON.parse(fs.readFileSync(results[0].output, "utf8")).objects.length >= 3);
    assert.match(fs.readFileSync(results[1].output, "utf8"), /<svg\b/);
    assert.deepEqual(Array.from(fs.readFileSync(results[2].output).subarray(0, 8)),
      [137, 80, 78, 71, 13, 10, 26, 10]);
  } finally {
    fs.rmSync(folder, { recursive: true, force: true });
  }
});

test("should_reject_oversized_source_before_wasm_allocation", () => {
  // given
  const folder = fs.mkdtempSync(path.join(os.tmpdir(), "dot-wasm-test-"));
  try {
    const input = path.join(folder, "too-large.dot");
    const output = path.join(folder, "output.svg");
    fs.writeFileSync(input, Buffer.alloc(256 * 1024 + 1, 97));
    // when
    const result = invoke(input, output);
    // then
    assert.equal(result.status, 1);
    assert.match(result.stderr, /outside 1\.\.262144/);
    assert.equal(fs.existsSync(output), false);
  } finally {
    fs.rmSync(folder, { recursive: true, force: true });
  }
});
