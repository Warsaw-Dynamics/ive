import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { runCorpus, verifyCorpus, main } from './native-corpus.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 20"><text x="1" y="15">Signal</text></svg>';

async function fixture(context, behavior = '') {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ive-native-corpus-test-'));
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const fixtures = path.join(root, 'fixtures');
  for (const language of ['mermaid', 'plantuml']) await fs.mkdir(path.join(fixtures, language), { recursive: true });
  const viewer = path.join(root, 'viewer');
  const code = `#!${process.execPath}\nconst fs=require('fs');const args=process.argv.slice(2);const source=args[args.indexOf('--render')+1];const output=args[args.indexOf('--output')+1];${behavior}\nfs.writeFileSync(output,${JSON.stringify(svg)});console.log(JSON.stringify({args,path:process.env.PATH,display:process.env.DISPLAY,wayland:process.env.WAYLAND_DISPLAY}));`;
  await fs.writeFile(viewer, code, { mode: 0o700 });
  return { root, fixtures, viewer, output: path.join(root, 'fresh') };
}

async function source(options, name = 'one.mmd', bytes = 'flowchart LR\n A[Signal]') {
  const language = name.endsWith('.puml') ? 'plantuml' : 'mermaid';
  await fs.writeFile(path.join(options.fixtures, language, name), bytes);
}

test('should_enumerate_both_languages_and_hash_exact_coherent_artifacts', async context => {
  // given
  const options = await fixture(context);
  await source(options, 'same.mmd');
  await source(options, 'same.puml', '@startuml\nAlice -> Bob: Signal\n@enduml');
  await fs.writeFile(path.join(options.fixtures, 'mermaid', 'README.md'), 'ignored metadata');
  // when
  const manifest = await runCorpus(options);
  // then
  assert.deepEqual(manifest.summary, { total: 2, rendered: 2, failed: 0 });
  assert.equal(manifest.binary.sha256, hash(await fs.readFile(options.viewer)));
  assert.deepEqual(manifest.results.map(result => result.file), ['mermaid/same.mmd', 'plantuml/same.puml']);
  for (const artifact of manifest.artifacts) {
    const bytes = await fs.readFile(path.join(options.output, artifact.path));
    assert.equal(bytes.length, artifact.bytes);
    assert.equal(hash(bytes), artifact.sha256);
  }
  for (const result of manifest.results) {
    const output = JSON.parse(await fs.readFile(path.join(options.output, result.stdout), 'utf8'));
    assert.equal(output.path, '');
    assert.equal(output.display, undefined);
    assert.equal(output.wayland, undefined);
    assert.equal(output.args[1], result.language);
    assert.equal(result.native, 'rendered');
    assert.equal(result.original, 'not-observed');
  }
  assert.deepEqual(await verifyCorpus(options.output), manifest);
});

test('should_refuse_existing_even_empty_output_before_spawning_child', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(source+".spawned","yes");');
  await source(options);
  await fs.mkdir(options.output);
  // when
  const run = runCorpus(options);
  // then
  await assert.rejects(run, /EEXIST/);
  assert.deepEqual(await fs.readdir(options.output), []);
  assert.deepEqual(await fs.readdir(path.join(options.fixtures, 'mermaid')), ['one.mmd']);
});

test('should_record_failed_native_case_and_continue_without_treating_original_error_as_success', async context => {
  // given
  const options = await fixture(context, 'if(source.endsWith("bad.mmd")){console.error("native parse failed");process.exit(7)}');
  await source(options, 'bad.mmd', 'bad');
  await source(options, 'good.mmd');
  const original = { results: [{ file: 'bad.mmd', status: 'error', source: { sha256: hash('bad') } }] };
  options.original = path.join(options.root, 'original.json');
  await fs.writeFile(options.original, JSON.stringify(original));
  // when
  const manifest = await runCorpus(options);
  // then
  assert.deepEqual(manifest.summary, { total: 2, rendered: 1, failed: 1 });
  assert.equal(manifest.results[0].original, 'error');
  assert.equal(manifest.results[0].native, 'failed');
  assert.equal(manifest.results[1].native, 'rendered');
  assert.equal(JSON.parse(await fs.readFile(path.join(options.output, manifest.results[0].status))).exitCode, 7);
  await verifyCorpus(options.output);
});

test('should_reject_artifact_tamper_and_extra_unrecorded_files', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  const manifest = await runCorpus(options);
  const svgPath = path.join(options.output, manifest.results[0].svg);
  const before = await fs.readFile(svgPath);
  // when
  await fs.writeFile(svgPath, 'tamper');
  // then
  await assert.rejects(verifyCorpus(options.output), /artifact (hash|size) drift/);
  await fs.writeFile(svgPath, before);
  await fs.writeFile(path.join(options.output, 'unrecorded'), 'extra');
  await assert.rejects(verifyCorpus(options.output), /artifact inventory/);
});

test('should_kill_and_reap_timed_out_child_and_keep_failure_evidence', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(require("path").join(require("path").dirname(require("path").dirname(source)),"pid"),String(process.pid));process.on("SIGTERM",()=>{});setInterval(()=>{},1000);return;');
  await source(options);
  // when
  const manifest = await runCorpus({ ...options, limits: { caseMilliseconds: 300 } });
  // then
  assert.equal(manifest.results[0].native, 'timeout');
  const pid = Number(await fs.readFile(path.join(options.root, 'pid'), 'utf8'));
  assert.throws(() => process.kill(pid, 0), /ESRCH/);
});

test('should_refuse_oversized_or_symlink_sources_before_child_launch', async context => {
  // given
  const options = await fixture(context);
  await fs.writeFile(path.join(options.root, 'large'), Buffer.alloc(256 * 1024 + 1));
  await fs.symlink(path.join(options.root, 'large'), path.join(options.fixtures, 'mermaid', 'one.mmd'));
  // when
  const run = runCorpus(options);
  // then
  await assert.rejects(run, /unsafe|source.*bound|regular/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_require_explicit_absolute_fixture_output_and_viewer_arguments', async () => {
  // given
  const args = ['export', '--fixtures', '/tmp/fixtures'];
  // when
  const run = main(args);
  // then
  await assert.rejects(run, /usage/);
});

test('should_detect_source_tamper_before_publishing_manifest', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(source,"changed after snapshot");');
  await source(options);
  // when
  const run = runCorpus(options);
  // then
  await assert.rejects(run, /artifact (hash|size) drift/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_validate_exact_source_and_aggregate_bounds_before_launch', async context => {
  // given
  const options = await fixture(context);
  const bytes = 'flowchart LR\n A[Signal]';
  await source(options, 'one.mmd', bytes);
  // when
  const equal = await runCorpus({ ...options, limits: { sourceBytes: bytes.length, sourceTotalBytes: bytes.length } });
  // then
  assert.equal(equal.summary.rendered, 1);
  await assert.rejects(runCorpus({ ...options, output: path.join(options.root, 'too-small'), limits: { sourceBytes: bytes.length - 1 } }), /oversized/);
  await source(options, 'two.mmd', bytes);
  await assert.rejects(runCorpus({ ...options, output: path.join(options.root, 'aggregate'), limits: { sourceTotalBytes: bytes.length } }), /oversized/);
  await assert.rejects(fs.stat(path.join(options.root, 'aggregate')), /ENOENT/);
});

test('should_bound_actual_case_count_and_refuse_widened_configuration', async context => {
  // given
  const options = await fixture(context);
  await source(options, 'one.mmd');
  await source(options, 'two.mmd');
  // when
  const run = runCorpus({ ...options, limits: { cases: 1 } });
  // then
  await assert.rejects(run, /excessive native corpus cases/);
  await assert.rejects(runCorpus({ ...options, limits: { cases: 257 } }), /invalid native corpus bound/);
  await assert.rejects(runCorpus({ ...options, limits: { unknown: 1 } }), /invalid native corpus bound/);
  await assert.rejects(fs.stat(options.output), /ENOENT/);
});

test('should_keep_partial_svg_on_nonzero_exit_without_positive_admission', async context => {
  // given
  const options = await fixture(context, `fs.writeFileSync(output,${JSON.stringify(svg)});console.error("partial output");process.exit(2);`);
  await source(options);
  // when
  const manifest = await runCorpus(options);
  // then
  assert.equal(manifest.results[0].native, 'failed');
  assert.equal(await fs.readFile(path.join(options.output, manifest.results[0].svg), 'utf8'), svg);
  assert.equal(manifest.summary.rendered, 0);
  await verifyCorpus(options.output);
});

test('should_reject_missing_and_empty_svg_as_failed_exports', async context => {
  // given
  const options = await fixture(context, 'if(source.endsWith("empty.mmd"))fs.writeFileSync(output,"");process.exit(0);');
  await source(options, 'empty.mmd');
  await source(options, 'missing.mmd');
  // when
  const manifest = await runCorpus(options);
  // then
  assert.equal(manifest.summary.failed, 2);
  assert.equal(manifest.results[0].native, 'failed');
  assert.equal(manifest.results[1].native, 'failed');
  await verifyCorpus(options.output);
});

test('should_bound_child_stdout_and_reap_the_flooding_process', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(require("path").join(require("path").dirname(require("path").dirname(source)),"pid"),String(process.pid));setInterval(()=>process.stdout.write("x".repeat(4096)),1);return;');
  await source(options);
  // when
  const manifest = await runCorpus({ ...options, limits: { logBytes: 512 } });
  // then
  assert.equal(manifest.results[0].native, 'output-limit');
  assert.equal(manifest.summary.failed, 1);
  const pid = Number(await fs.readFile(path.join(options.root, 'pid'), 'utf8'));
  assert.throws(() => process.kill(pid, 0), /ESRCH/);
  await verifyCorpus(options.output);
});

test('should_discard_oversized_svg_and_record_bounded_failure_status', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(output,"x".repeat(4096));process.exit(0);');
  await source(options);
  // when
  const manifest = await runCorpus({ ...options, limits: { svgBytes: 128 } });
  // then
  assert.equal(manifest.results[0].native, 'output-limit');
  assert.equal(manifest.results[0].svg, undefined);
  assert.equal(JSON.parse(await fs.readFile(path.join(options.output, manifest.results[0].status))).discardedSvgBytes, 4096);
  await verifyCorpus(options.output);
});

test('should_require_manifest_and_artifacts_to_fit_aggregate_budget', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  // when
  const run = runCorpus({ ...options, limits: { artifactBytes: 1024 } });
  // then
  await assert.rejects(run, /aggregate.*bound/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_refuse_original_manifest_source_hash_drift', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  options.original = path.join(options.root, 'original.json');
  await fs.writeFile(options.original, JSON.stringify({ results: [{ file: 'one.mmd', status: 'error', source: { sha256: hash('different input') } }] }));
  // when
  const run = runCorpus(options);
  // then
  await assert.rejects(run, /original source\/status mismatch/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_record_spawn_failure_with_status_and_hash_closure', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  await fs.chmod(options.viewer, 0o600);
  // when
  const manifest = await runCorpus(options);
  // then
  assert.equal(manifest.results[0].native, 'spawn-error');
  assert.equal(manifest.summary.failed, 1);
  await verifyCorpus(options.output);
});

test('should_reject_manifest_status_or_path_forgery_before_reading_external_paths', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  const manifest = await runCorpus(options);
  const file = path.join(options.output, 'admission.json');
  // when
  await fs.writeFile(file, JSON.stringify({ ...manifest, results: [{ ...manifest.results[0], native: 'failed' }] }));
  // then
  await assert.rejects(verifyCorpus(options.output), /status mismatch/);
  await fs.writeFile(file, JSON.stringify({ ...manifest, artifacts: [{ ...manifest.artifacts[0], path: '../escape' }] }));
  await assert.rejects(verifyCorpus(options.output), /invalid native corpus artifact/);
});

test('should_bound_aggregate_render_deadline_without_publishing_incomplete_manifest', async context => {
  // given
  const options = await fixture(context, 'setInterval(()=>{},1000);return;');
  await source(options, 'one.mmd');
  await source(options, 'two.mmd');
  // when
  const run = runCorpus({ ...options, limits: { corpusMilliseconds: 30 } });
  // then
  await assert.rejects(run, /aggregate deadline/);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_reject_expired_final_phase_after_successful_child_and_artifact_verification', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  const ticks = [0, 0, 0];
  const clock = { now: () => ticks.shift() ?? 300001 };
  // when
  const run = runCorpus(options, clock);
  // then
  await assert.rejects(run, /aggregate deadline/);
  const status = JSON.parse(await fs.readFile(path.join(options.output, 'mermaid--one.mmd.status.json')));
  assert.equal(status.outcome, 'rendered');
  assert.equal(await fs.readFile(path.join(options.output, 'mermaid--one.mmd.svg'), 'utf8'), svg);
  await assert.rejects(fs.stat(path.join(options.output, 'admission.json')), /ENOENT/);
});

test('should_exit_nonzero_for_complete_failed_native_run_in_export_and_check_modes', async context => {
  // given
  const options = await fixture(context, 'console.error("bad source");process.exit(3);');
  await source(options);
  const tool = new URL('./native-corpus.mjs', import.meta.url);
  const run = args => new Promise(resolve => {
    const child = spawn(process.execPath, [tool.pathname, ...args], { stdio: 'ignore' });
    child.on('close', resolve);
  });
  // when
  const exported = await run(['export', '--fixtures', options.fixtures, '--output', options.output, '--viewer', options.viewer]);
  const checked = await run(['check', options.output]);
  // then
  assert.equal(exported, 1);
  assert.equal(checked, 1);
  assert.equal((await verifyCorpus(options.output)).summary.failed, 1);
});

test('should_preserve_existing_evidence_without_launch_or_overwrite', async context => {
  // given
  const options = await fixture(context, 'fs.writeFileSync(source+".spawned","yes");');
  await source(options);
  await fs.mkdir(options.output);
  const retained = path.join(options.output, 'admission.json');
  await fs.writeFile(retained, 'retained previous evidence');
  const before = hash(await fs.readFile(retained));
  // when
  const run = runCorpus(options);
  // then
  await assert.rejects(run, /EEXIST/);
  assert.equal(hash(await fs.readFile(retained)), before);
  assert.deepEqual(await fs.readdir(options.output), ['admission.json']);
});

test('should_preserve_positive_native_export_when_original_manifest_records_error', async context => {
  // given
  const options = await fixture(context);
  await source(options, 'plugin.mmd', 'plugin example');
  options.original = path.join(options.root, 'original.json');
  await fs.writeFile(options.original, JSON.stringify({ results: [{ file: 'plugin.mmd', status: 'error', source: { sha256: hash('plugin example') } }] }));
  // when
  const manifest = await runCorpus(options);
  // then
  assert.equal(manifest.results[0].native, 'rendered');
  assert.equal(manifest.results[0].original, 'error');
  assert.equal(manifest.summary.failed, 0);
  await verifyCorpus(options.output);
});

test('should_reject_artifact_symlink_even_when_destination_bytes_match', async context => {
  // given
  const options = await fixture(context);
  await source(options);
  const manifest = await runCorpus(options);
  const artifact = path.join(options.output, manifest.results[0].svg);
  const outside = path.join(options.root, 'same-svg');
  await fs.writeFile(outside, await fs.readFile(artifact));
  // when
  await fs.unlink(artifact);
  await fs.symlink(outside, artifact);
  // then
  await assert.rejects(verifyCorpus(options.output), /artifact size drift or unsafe file/);
});
