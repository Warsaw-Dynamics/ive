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

test('should reject a pending handwritten syntax exemption in the completed migration', t => {
  // given
  const root = checkout(t);
  const file = path.join(root, 'provenance/plantuml-parser-responsibilities.json');
  const ledger = JSON.parse(fs.readFileSync(file));
  ledger.modules[0].groups[0].role = 'pending-syntax';
  fs.writeFileSync(file, JSON.stringify(ledger));
  // when / then
  assert.throws(() => auditParserInventory(root), /invalid function responsibility/);
});

test('should reject restoration of the obsolete physical-line input adapter', t => {
  // given
  const root = checkout(t);
  fs.appendFileSync(path.join(root, 'engine/src/plantuml/source.rs'), '\npub(super) struct Line { pub number: usize, pub text: String }\n');
  // when / then
  assert.throws(() => auditParserInventory(root), /obsolete physical-line parser adapter/);
});
