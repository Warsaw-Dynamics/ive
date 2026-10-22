import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { validateManifest } from './audit-plantuml.mjs';

const manifest = () => JSON.parse(fs.readFileSync(new URL('../provenance/plantuml-mit-1.2026.9.json', import.meta.url)));

test('should accept the exact authorized source and module inventory', () => {
  // given
  const source = manifest();
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
});

test('should reject source escape and restricted implementation paths before reads', () => {
  // given
  for (const path of ['../plantuml/Other.java', 'net/sourceforge/plantuml/../../Other.java', 'net/sourceforge/plantuml/smetana/Layout.java']) {
    const source = manifest();
    source.sourceFiles[0].path = path;
    // when / then
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should reject a different edition or artifact identity', () => {
  // given
  const source = manifest();
  source.artifact.sha256 = '0'.repeat(64);
  // when / then
  assert.throws(() => validateManifest(source), /unauthorized PlantUML/);
});
