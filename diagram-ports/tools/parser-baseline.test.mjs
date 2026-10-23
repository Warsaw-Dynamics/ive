import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { compare, decodeExport } from './parser-baseline.mjs';

test('should compare actual exporter configuration and reject actual default drift', () => {
  // given
  const baseline = JSON.parse(fs.readFileSync(new URL('../fixtures/plantuml/parser-baseline.json', import.meta.url)));
  const files = ['fixtures/example.puml'];
  const output = configuration => `${JSON.stringify({ configuration })}\n${JSON.stringify({ path: files[0], family: 'sequence', svg: '<svg/>' })}\n`;
  // when
  const exact = decodeExport(output(baseline.configuration), files);
  const changed = decodeExport(output({ ...baseline.configuration, sourceBytes: baseline.configuration.sourceBytes + 1 }), files);
  // then
  assert.deepEqual(exact.configuration, baseline.configuration);
  assert.equal(changed.configuration.sourceBytes, baseline.configuration.sourceBytes + 1);
  assert.throws(() => compare({ ...baseline, configuration: changed.configuration }, baseline));
  assert.throws(() => decodeExport(`${JSON.stringify(exact.records[0])}\n`, files), /configuration/);
});

test('should detect source family SVG and configuration drift', () => {
  // given
  const baseline = { configuration: { language: 'PlantUml' }, fixtures: [{ path: 'fixture.puml', sourceSha256: 'source', family: 'sequence', svgSha256: 'svg' }] };
  for (const key of ['sourceSha256', 'family', 'svgSha256', 'path']) {
    const current = structuredClone(baseline);
    current.fixtures[0][key] = 'changed';
    // when / then
    assert.throws(() => compare(current, baseline));
  }
  assert.throws(() => compare({ ...baseline, configuration: {} }, baseline));
  assert.throws(() => compare({ ...baseline, fixtures: [] }, baseline));
});

test('should refuse a second recording without modifying the immutable baseline', () => {
  // given
  const path = new URL('../fixtures/plantuml/parser-baseline.json', import.meta.url);
  const before = fs.readFileSync(path);
  // when
  const result = spawnSync(process.execPath, [new URL('./parser-baseline.mjs', import.meta.url).pathname, 'record'], { encoding: 'utf8' });
  // then
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /baseline already exists/);
  assert.deepEqual(fs.readFileSync(path), before);
});
