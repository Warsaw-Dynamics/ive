import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { validateManifest, audit } from './audit-plantuml.mjs';

const manifest = () => JSON.parse(fs.readFileSync(new URL('../provenance/plantuml-mit-1.2026.9.json', import.meta.url)));

test('should admit registered parser and grammar destinations and reject unregistered destinations', () => {
  // given
  const source = manifest();
  source.sourceFiles[0].rust = ['engine/src/plantuml/parser/document.rs', 'engine/src/plantuml/parser/grammar/document.lalrpop'];
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
  for (const target of ['engine/src/plantuml/parser/not_registered.rs', 'engine/src/plantuml/generated/document.rs', '../outside.rs']) {
    source.sourceFiles[0].rust = [target];
    assert.throws(() => validateManifest(source), /unregistered Rust destination/);
  }
});

test('should admit exact Chen and icon-free rectangle semantics without subtype or asset paths', () => {
  // given
  const source = manifest();
  // when / then
  for (const path of ['cheneer/ChenEerDiagram.java', 'cheneer/command/CommandCreateAttribute.java', 'svek/image/EntityImageChenRelationship.java', 'decoration/LinkStyle.java', 'decoration/symbol/USymbolRectangle.java', 'klimt/color/ColorTrieNode.java']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/${path}`;
    assert.doesNotThrow(() => validateManifest(source));
  }
  for (const path of ['cheneer/command/CommandMultiSubclass.java', 'cheneer/command/CommandSimpleSubclass.java', 'svek/image/EntityImageChenCircle.java', 'decoration/symbol/USymbolCloud.java', 'klimt/color/OtherColors.java', 'smetana/SmetanaForJson.java', 'stdlib/archimate/sprite.java']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/${path}`;
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should admit exact inventory semantics and reject unselected source siblings', () => {
  // given
  const source = manifest();
  // when / then
  for (const path of ['filesdiagram/FEntry.java', 'board/BNode.java', 'hcl/HclParser.java', 'packetdiag/command/CommandSameHeight.java']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/${path}`;
    assert.doesNotThrow(() => validateManifest(source));
  }
  for (const path of ['filesdiagram/FilesDiagramFactory.java', 'board/BArray.java', 'hcl/OtherParser.java', 'packetdiag/PacketDiagramFactory.java']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/${path}`;
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should admit exact nwdiag semantics and reject unselected layout implementations', () => {
  // given
  const source = manifest();
  source.sourceFiles[0].path = 'net/sourceforge/plantuml/nwdiag/core/NServer.java';
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
  for (const path of ['net/sourceforge/plantuml/nwdiag/next/NPlayField.java', 'net/sourceforge/plantuml/nwdiag/core/GridTextBlockDecorated.java', 'net/sourceforge/plantuml/nwdiag/CommandLink.java']) {
    source.sourceFiles[0].path = path;
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should admit exact selected Salt paths and reject unselected rendering or resource code', () => {
  // given
  const source = manifest();
  source.sourceFiles[0].path = 'net/sourceforge/plantuml/salt/DataSourceImpl.java';
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
  for (const path of ['net/sourceforge/plantuml/salt/SaltDictionary.java', 'net/sourceforge/plantuml/salt/element/ElementImage.java', 'net/sourceforge/plantuml/salt/factory/OtherFactory.java']) {
    source.sourceFiles[0].path = path;
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should accept the exact authorized source and module inventory', () => {
  // given
  const source = manifest();
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
});

test('should admit selected categorical chart sources and reject unported chart siblings', () => {
  // given
  const source = manifest();
  // when / then
  for (const name of ['ChartAxis', 'ChartSeries', 'ChartRenderer', 'BarRenderer', 'LineRenderer', 'command/CommandChartBar', 'command/CommandChartLine']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/chart/${name}.java`;
    assert.doesNotThrow(() => validateManifest(source));
  }
  for (const name of ['AreaRenderer', 'ScatterRenderer', 'command/CommandChartAnnotation', 'command/CommandChartOrientation', 'command/CommandChartStackMode']) {
    source.sourceFiles[0].path = `net/sourceforge/plantuml/chart/${name}.java`;
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
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

test('should reject unselected timing waveform and resource implementation paths', () => {
  // given
  for (const path of ['net/sourceforge/plantuml/timingdiagram/command/CommandClock.java', 'net/sourceforge/plantuml/timingdiagram/PlayerAnalog.java', 'net/sourceforge/plantuml/timingdiagram/graphic/HexaShape.java']) {
    const source = manifest();
    source.sourceFiles[0].path = path;
    // when / then
    assert.throws(() => validateManifest(source), /unauthorized source path/);
  }
});

test('should admit selected Gantt semantics while rejecting calendar resource and layout implementations', () => {
  // given
  const source = manifest();
  source.sourceFiles[0].path = 'net/sourceforge/plantuml/gantt/lang/SubjectTask.java';
  // when / then
  assert.doesNotThrow(() => validateManifest(source));
  for (const path of ['net/sourceforge/plantuml/gantt/Resource.java', 'net/sourceforge/plantuml/gantt/ngm/NGMAllocation.java', 'net/sourceforge/plantuml/gantt/lang/SubjectResource.java', 'net/sourceforge/plantuml/gantt/draw/RectangleTask.java']) {
    source.sourceFiles[0].path = path;
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
