import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { runCorpus, runChild, closeBrowser, main } from './oracles.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const svg = '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><text x="1" y="8">Signal</text></svg>';

async function fixture(context) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ive-oracle-tool-test-'));
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const input = path.join(root, 'input');
  await fs.mkdir(input);
  return { root, input, output: path.join(root, 'fresh-output') };
}

function backend(overrides = {}) {
  return { metadata: { browser: 'test-double', mermaid: 'test-double' }, render: async () => svg,
    inspect: async () => ({ bounds: { x: 0, y: 0, width: 20, height: 10 }, texts: [{ label: 'Signal' }] }),
    close: async () => {}, ...overrides };
}

test('should_write_one_coherent_run_to_explicit_fresh_output_without_changing_old_artifacts', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.input, 'same.mmd'), 'flowchart LR\n A[Signal]');
  await fs.writeFile(path.join(options.input, 'same.puml'), '@startuml\nAlice -> Bob: Signal\n@enduml');
  const old = path.join(options.root, 'corpus-mermaid');
  await fs.mkdir(old);
  await fs.writeFile(path.join(old, 'admission.json'), 'old evidence');
  let closed = 0;
  // when
  const manifest = await runCorpus(options, async () => backend({ close: async () => { closed++; } }));
  // then
  assert.equal(closed, 1);
  assert.equal(await fs.readFile(path.join(old, 'admission.json'), 'utf8'), 'old evidence');
  assert.equal(manifest.outputDirectory, options.output);
  assert.deepEqual(manifest.summary, { total: 2, rendered: 2 });
  assert.deepEqual(JSON.parse(await fs.readFile(path.join(options.output, 'admission.json'))), manifest);
  for (const result of manifest.results) {
    for (const artifact of [result.source, result.svg, result.metrics]) {
      const bytes = await fs.readFile(path.join(options.output, artifact.path));
      assert.equal(hash(bytes), artifact.sha256);
      assert.equal(bytes.length, artifact.bytes);
    }
    const metrics = JSON.parse(await fs.readFile(path.join(options.output, result.metrics.path)));
    assert.equal(metrics.svgSha256, result.svg.sha256);
  }
});

test('should_refuse_existing_output_even_when_empty_before_launching_renderer', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.input, 'one.mmd'), 'flowchart LR\n A-->B');
  await fs.mkdir(options.output);
  let opened = 0;
  // when
  const run = runCorpus(options, async () => { opened++; return backend(); });
  // then
  await assert.rejects(run, /EEXIST/);
  assert.equal(opened, 0);
  assert.deepEqual(await fs.readdir(options.output), []);
});

test('should_record_bounded_negative_artifact_and_close_renderer', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.input, 'bad.mmd'), 'broken');
  let closed = 0;
  // when
  const manifest = await runCorpus(options, async () => backend({ render: async () => { throw new Error('bad diagram'); }, close: async () => { closed++; } }));
  // then
  assert.equal(manifest.results[0].status, 'error');
  assert.equal(manifest.summary.rendered, 0);
  assert.equal(closed, 1);
  assert.equal(await fs.readFile(path.join(options.output, manifest.results[0].error.path), 'utf8'), 'bad diagram\n');
  assert.equal(manifest.results[0].svg, undefined);
});

test('should_reject_artifact_drift_before_publishing_manifest_and_close_renderer', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.input, 'one.mmd'), 'flowchart LR\n A-->B');
  let closed = 0;
  // when
  const run = runCorpus(options, async () => backend({ inspect: async () => { await fs.writeFile(path.join(options.output, 'one.mmd'), 'tampered'); return {}; }, close: async () => { closed++; } }));
  // then
  await assert.rejects(run, /artifact hash drift: one.mmd/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
  assert.equal(closed, 1);
});

test('should_bound_sources_and_reject_symlink_inputs_without_publishing_manifest', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.root, 'oversized'), Buffer.alloc(256 * 1024 + 1));
  await fs.symlink(path.join(options.root, 'oversized'), path.join(options.input, 'one.mmd'));
  let closed = 0;
  // when
  const run = runCorpus(options, async () => backend({ close: async () => { closed++; } }));
  // then
  await assert.rejects(run, /unsafe or oversized oracle input/);
  assert.equal(closed, 1);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_require_an_explicit_output_and_dependency_root', async () => {
  // given
  const argumentsWithoutOutput = ['--corpus', '/tmp/input'];
  // when
  const run = main(argumentsWithoutOutput);
  // then
  await assert.rejects(run, /usage:.*INPUT FRESH_OUTPUT.*LOCAL_DEPENDENCY_DIRECTORY/);
});

test('should_kill_and_reap_timed_out_child_that_ignores_termination', async context => {
  // given
  const options = await fixture(context);
  const pidFile = path.join(options.root, 'pid');
  const code = `require('fs').writeFileSync(process.argv[1],String(process.pid));process.on('SIGTERM',()=>{});setInterval(()=>{},1000)`;
  // when
  const run = runChild(process.execPath, ['-e', code, pidFile], '', 300);
  // then
  await assert.rejects(run, /child exceeded 300ms/);
  const pid = Number(await fs.readFile(pidFile, 'utf8'));
  assert.throws(() => process.kill(pid, 0), /ESRCH/);
});

test('should_kill_and_reap_browser_when_graceful_close_hangs', async () => {
  // given
  const child = spawn(process.execPath, ['-e', 'setInterval(()=>{},1000)'], { detached: true, stdio: 'ignore' });
  const browser = { process: () => child, close: () => new Promise(() => {}) };
  // when
  await closeBrowser(browser, 100);
  // then
  assert.equal(child.signalCode, 'SIGKILL');
  assert.throws(() => process.kill(child.pid, 0), /ESRCH/);
});
