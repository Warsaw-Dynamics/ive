import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { auditParserInventory } from './audit-plantuml.mjs';

const source = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
function checkout(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'plantuml-inventory-test-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, 'engine/src'), { recursive: true });
  fs.cpSync(path.join(source, 'engine/src/plantuml'), path.join(root, 'engine/src/plantuml'), { recursive: true });
  fs.cpSync(path.join(source, 'provenance'), path.join(root, 'provenance'), { recursive: true });
  return root;
}

test('should admit the current explicitly classified parser inventory', () => {
  // given / when / then
  assert.doesNotThrow(() => auditParserInventory(source));
});

for (const file of ['parser/grammar/unknown.lalrpop', 'parser/unknown.rs', 'generated/unknown.rs']) {
  test(`should reject unregistered input or output ${file}`, t => {
    // given
    const root = checkout(t);
    fs.writeFileSync(path.join(root, 'engine/src/plantuml', file), 'unregistered');
    // when / then
    assert.throws(() => auditParserInventory(root), /inventory changed/);
  });
}

for (const [file, diagnostic] of [
  ['engine/src/plantuml/parser/grammar/salt.lalrpop', /grammar checksum mismatch/],
  ['engine/src/plantuml/generated/salt.rs', /generated checksum mismatch/],
]) {
  test(`should reject stale registered bytes in ${file}`, t => {
    // given
    const root = checkout(t);
    fs.appendFileSync(path.join(root, file), '\n// drift\n');
    // when / then
    assert.throws(() => auditParserInventory(root), diagnostic);
  });
}

test('should reject an unclassified function in an existing native module', t => {
  // given
  const root = checkout(t);
  fs.appendFileSync(path.join(root, 'engine/src/plantuml/lexer.rs'), '\nfn hidden_new_syntax() {}\n');
  // when / then
  assert.throws(() => auditParserInventory(root), /unclassified function/);
});
