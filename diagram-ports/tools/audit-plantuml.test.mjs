import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { validateManifest, audit } from './audit-plantuml.mjs';

const manifest = () => JSON.parse(fs.readFileSync(new URL('../provenance/plantuml-mit-1.2026.9.json', import.meta.url)));

test('should accept the exact authorized source and module inventory', () => {
  // given
  const source = manifest();
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
});

test('should reject source escape and restricted implementation paths before reads', () => {
  // given
  for (const path of ['../plantuml/Other.java', 'net/sourceforge/plantuml/../../Other.java', 'net/sourceforge/plantuml/smetana/Layout.java', 'net/sourceforge/plantuml/statediagram/smetana/Layout.java', 'net/sourceforge/plantuml/activitydiagram3/elk/Layout.java', 'net/sourceforge/plantuml/style/StyleLoader.java', 'net/sourceforge/plantuml/style/parser/OtherParser.java']) {
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

test('should reject untranslated TIM runtime files', () => {
  // given
  for (const path of ['net/sourceforge/plantuml/tim/TMemoryGlobal.java', 'net/sourceforge/plantuml/tim/FunctionsSet.java', 'net/sourceforge/plantuml/tim/EaterLegacyDefine.java']) {
    const source = manifest();
    source.sourceFiles[0].path = path;
    // when / then
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should reject missing exact contributor credits despite colon spacing', () => {
  // given
  const source = manifest();
  const state = source.sourceFiles.find(file => file.path.endsWith('/statediagram/StateDiagram.java'));
  state.copyrights = state.copyrights.filter(line => !line.includes('Serge Wenger'));
  // when / then
  assert.throws(() => audit(source), /missing source credit/);
});

test('should reject removed minimal-json copyright before accepting translated semantics', () => {
  // given
  const source = manifest();
  const json = source.sourceFiles.find(file => file.path.endsWith('/json/JsonParser.java'));
  json.copyrights = json.copyrights.filter(line => !line.includes('EclipseSource'));
  // when / then
  assert.throws(() => audit(source), /missing source copyright/);
});
