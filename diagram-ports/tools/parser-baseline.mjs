// Development-only driver; recording is deliberately write-once.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const destination = path.join(root, 'fixtures/plantuml/parser-baseline.json');
const configuration = Object.freeze({
  language: 'PlantUml', measurement: 'default deterministic', sourceBytes: 262144,
  svgBytes: 16777216, modelItems: 10000, nestingDepth: 64,
  layoutWork: 5000000, timeoutMilliseconds: 10000,
});
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

export function compare(currentDocument, baselineDocument) {
  assert.deepEqual(currentDocument.configuration, baselineDocument.configuration);
  assert.equal(currentDocument.fixtures.length, baselineDocument.fixtures.length);
  for (let index = 0; index < currentDocument.fixtures.length; index++) {
    const current = currentDocument.fixtures[index];
    const baseline = baselineDocument.fixtures[index];
    assert.equal(current.path, baseline.path);
    assert.equal(current.sourceSha256, baseline.sourceSha256);
    assert.equal(current.family, baseline.family);
    assert.equal(current.svgSha256, baseline.svgSha256);
  }
}

function collect() {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'provenance/plantuml-mit-1.2026.9.json')));
  const files = manifest.fixtures.map(file => file.path).sort();
  assert.ok(files.length > 0 && files.length <= 256, 'baseline fixture count exceeds bounds');
  const output = execFileSync('cargo', ['run', '--locked', '--offline', '-q', '-p', 'diagram-engine', '--example', 'parser_baseline', '--', ...files], {
    cwd: root, env: { ...process.env, CARGO_TARGET_DIR: '/mnt/data/Projects/ive/target' },
    maxBuffer: 32 * 1024 * 1024, timeout: 300000,
  }).toString();
  const records = output.trim().split('\n').map(line => JSON.parse(line));
  assert.equal(records.length, files.length);
  const fixtures = records.map((record, index) => {
    assert.equal(record.path, files[index]);
    const sourceSha256 = sha256(fs.readFileSync(path.join(root, record.path)));
    assert.equal(sourceSha256, manifest.fixtures.find(file => file.path === record.path).sha256);
    return { path: record.path, sourceSha256, family: record.family, svgSha256: sha256(record.svg) };
  });
  return { schema: 1, configuration, fixtures };
}

export function run(mode) {
  assert.ok(['record', 'check'].includes(mode), 'usage: parser-baseline.mjs record|check');
  if (mode === 'record' && fs.existsSync(destination)) throw new Error(`baseline already exists: ${destination}`);
  const current = collect();
  if (mode === 'record') fs.writeFileSync(destination, `${JSON.stringify(current, null, 2)}\n`, { flag: 'wx' });
  else compare(current, JSON.parse(fs.readFileSync(destination)));
  console.log(`Verified immutable parser baseline: ${current.fixtures.length} source/family/SVG hashes (${mode}).`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) run(process.argv[2]);
