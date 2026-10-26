import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';

const workspace = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function fixture(context) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ive-merman-audit-test-'));
  context.after(() => fs.rmSync(root, { recursive: true }));
  fs.cpSync(path.join(workspace, 'vendor'), path.join(root, 'vendor'), { recursive: true });
  fs.mkdirSync(path.join(root, 'tools'));
  fs.copyFileSync(path.join(workspace, 'tools/audit-merman.mjs'), path.join(root, 'tools/audit-merman.mjs'));
  return root;
}

function audit(root) {
  return spawnSync(process.execPath, [path.join(root, 'tools/audit-merman.mjs')], { encoding: 'utf8', timeout: 5000 });
}

test('should_accept_the_original_imports_with_only_recorded_patches', context => {
  // given
  const root = fixture(context);
  // when
  const result = audit(root);
  // then
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /656 imported files; 4 explicitly recorded source patches/);
});

test('should_reject_an_unrecorded_change_to_an_imported_source', context => {
  // given
  const root = fixture(context);
  const source = path.join(root, 'vendor/merman-core-0.9.2/src/baseline.rs');
  fs.appendFileSync(source, '\n// Unrecorded change in a disposable test copy.\n');
  // when
  const result = audit(root);
  // then
  assert.equal(result.status, 1);
  assert.match(result.stderr, /unrecorded source change: merman-core-0\.9\.2\/src\/baseline\.rs/);
});

test('should_reject_an_added_unimported_source_file', context => {
  // given
  const root = fixture(context);
  fs.writeFileSync(path.join(root, 'vendor/merman-core-0.9.2/src/unimported.rs'), '// Unimported file.\n');
  // when
  const result = audit(root);
  // then
  assert.equal(result.status, 1);
  assert.match(result.stderr, /source file inventory changed: merman-core-0\.9\.2/);
});

function archive(entries) {
  const blocks = entries.flatMap(([name, bytes, kind = 48]) => {
    const content = Buffer.from(bytes);
    const header = Buffer.alloc(512);
    header.write(name);
    header.write(`${content.length.toString(8).padStart(11, '0')}\0`, 124);
    header.fill(32, 148, 156);
    header[156] = kind;
    const sum = header.reduce((sum, byte) => sum + byte, 0);
    header.write(`${sum.toString(8).padStart(6, '0')}\0 `, 148);
    return [header, content, Buffer.alloc((512 - content.length % 512) % 512)];
  });
  return gzipSync(Buffer.concat([...blocks, Buffer.alloc(1024)]));
}

function importFixture(context, entries = []) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ive-merman-import-test-'));
  context.after(() => fs.rmSync(root, { recursive: true }));
  fs.mkdirSync(path.join(root, 'tools'));
  fs.mkdirSync(path.join(root, 'archives'));
  let script = fs.readFileSync(path.join(workspace, 'tools/audit-merman.mjs'), 'utf8');
  for (const name of ['merman-core', 'merman-render']) {
    const directory = `${name}-0.9.2`;
    const bytes = archive([[`${directory}/src/lib.rs`, '// genuine archive member\n'], ...entries.map(([name, ...rest]) => [name.replace('$crate', directory), ...rest])]);
    fs.writeFileSync(path.join(root, 'archives', `${directory}.crate`), bytes);
    script = script.replace(new RegExp(`(name: '${name}', archiveSha256: ')[a-f0-9]+`), `$1${createHash('sha256').update(bytes).digest('hex')}`);
    fs.mkdirSync(path.join(root, 'vendor', directory, 'src'), { recursive: true });
    fs.writeFileSync(path.join(root, 'vendor', directory, 'src/lib.rs'), '// genuine archive member\n');
  }
  fs.writeFileSync(path.join(root, 'tools/audit-merman.mjs'), script);
  return root;
}

function recordImport(root) {
  return spawnSync(process.execPath, [path.join(root, 'tools/audit-merman.mjs'), '--record-import', path.join(root, 'archives')], { encoding: 'utf8', timeout: 5000 });
}

test('should_reject_tampered_extraction_before_recording_authenticated_import', context => {
  // given
  const root = importFixture(context);
  fs.writeFileSync(path.join(root, 'vendor/merman-core-0.9.2/src/lib.rs'), '// independent tampered extraction\n');
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stderr, /archive member mismatch.*merman-core-0.9.2\/src\/lib.rs/);
  assert.equal(fs.existsSync(path.join(root, 'vendor/IMPORTS.json')), false);
});

test('should_record_only_matching_authenticated_archive_members', context => {
  // given
  const root = importFixture(context);
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'vendor/IMPORTS.json')));
  assert.equal(manifest.crates.length, 2);
  assert.equal(manifest.crates[0].files[0].upstreamSha256, createHash('sha256').update('// genuine archive member\n').digest('hex'));
});

test('should_reject_second_crate_mismatch_without_partially_recording_first_crate', context => {
  // given
  const root = importFixture(context);
  fs.writeFileSync(path.join(root, 'vendor/merman-render-0.9.2/src/lib.rs'), '// different bytes\n');
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 1);
  assert.match(result.stderr, /archive member mismatch.*merman-render/);
  assert.equal(fs.existsSync(path.join(root, 'vendor/IMPORTS.json')), false);
});

for (const [name, entries, message] of [
  ['missing extracted archive member', [['$crate/src/missing.rs', '// missing']], /archive extraction inventory mismatch/],
  ['traversal archive path', [['$crate/../escape', 'bad']], /unsafe archive member/],
  ['duplicate archive path', [['$crate/src/lib.rs', 'duplicate']], /duplicate archive member/],
  ['link archive entry', [['$crate/src/link.rs', '', 50]], /unsupported archive member type/],
]) test(`should_reject_${name.replaceAll(' ', '_')}_before_recording`, context => {
  // given
  const root = importFixture(context, entries);
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 1);
  assert.match(result.stderr, message);
  assert.equal(fs.existsSync(path.join(root, 'vendor/IMPORTS.json')), false);
});

test('should_accept_only_explicitly_omitted_archive_lock_and_examples', context => {
  // given
  const root = importFixture(context, [['$crate/Cargo.lock', '# omitted lock'], ['$crate/examples/demo.rs', '// omitted example']]);
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'vendor/IMPORTS.json')));
  assert.deepEqual(manifest.crates.map(crate => crate.files.length), [1, 1]);
});

test('should_reject_symlink_extraction_before_recording', context => {
  // given
  const root = importFixture(context);
  const file = path.join(root, 'vendor/merman-core-0.9.2/src/lib.rs');
  fs.renameSync(file, path.join(root, 'genuine.rs'));
  fs.symlinkSync(path.join(root, 'genuine.rs'), file);
  // when
  const result = recordImport(root);
  // then
  assert.equal(result.status, 1);
  assert.match(result.stderr, /unexpected source entry/);
  assert.equal(fs.existsSync(path.join(root, 'vendor/IMPORTS.json')), false);
});
