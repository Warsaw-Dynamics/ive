import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

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
