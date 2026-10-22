// Development-only source-integrity audit; never linked into the native engine.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(root, 'vendor/IMPORTS.json');
const patchesPath = path.join(root, 'vendor/PATCHES.json');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const read = relative => fs.readFileSync(path.join(root, 'vendor', relative));

function files(directory, prefix = '') {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const relative = path.posix.join(prefix, entry.name);
    if (entry.isDirectory()) return files(path.join(directory, entry.name), relative);
    if (entry.isFile()) return [relative];
    throw new Error(`unexpected source entry: ${relative}`);
  }).sort();
}

const imports = [
  { name: 'merman-core', archiveSha256: 'f1d2afe2084c0223b772756489a5da51ad0ecbd195aef90268f3037648738796' },
  { name: 'merman-render', archiveSha256: '321c4ca6f834e07b01790c1bc2591dc3a51f450c56c4c71a8111000a50efec0f' },
].map(crate => ({ ...crate, version: '0.9.2', sourceCommit: '8e303a43b548c9e40e73f0bc657925d66e7d4fb5' }));

if (process.argv[2] === '--record-import') {
  const archiveDirectory = process.argv[3];
  if (!archiveDirectory) throw new Error('record-import requires the original registry archive directory');
  if (fs.existsSync(manifestPath)) throw new Error('refusing to overwrite original import manifest');
  const crates = imports.map(crate => {
    const directory = `${crate.name}-${crate.version}`;
    const archive = fs.readFileSync(path.join(archiveDirectory, `${directory}.crate`));
    if (hash(archive) !== crate.archiveSha256) throw new Error(`archive checksum mismatch: ${directory}`);
    return { ...crate, directory, omitted: ['Cargo.lock', 'examples/**'], files: files(path.join(root, 'vendor', directory)).map(file => {
      const relative = `${directory}/${file}`;
      return { path: relative, upstreamSha256: hash(read(relative)) };
    }) };
  });
  fs.writeFileSync(manifestPath, `${JSON.stringify({ format: 1, registry: 'https://crates.io', crates }, null, 2)}\n`);
  console.log('Recorded original imported-source hashes; do not regenerate after patching.');
} else if (process.argv[2] === '--record-patches') {
  if (fs.existsSync(patchesPath)) throw new Error('refusing to overwrite patch ledger');
  const reasons = new Map([
    ['merman-core-0.9.2/src/diagrams/mindmap/db.rs', 'Controlled structural-depth section assignment; retain branch defaults, eleven colors, decorations and child-edge projection.'],
    ['merman-core-0.9.2/src/diagrams/mindmap/parse.rs', 'Read effective mindmap.colorBy before constructing consumed typed nodes/edges.'],
    ['merman-render-0.9.2/src/usecase/measure.rs', 'Measure generic SVG usecase labels with their inherited global font, preserving actor/HTML and node-style overrides.'],
    ['merman-render-0.9.2/src/svg/parity/tree_view/render.rs', 'Emit the same validated effective family list used by tree layout.'],
  ]);
  const manifest = JSON.parse(fs.readFileSync(manifestPath));
  const changed = manifest.crates.flatMap(crate => crate.files).flatMap(file => {
    const patchedSha256 = hash(read(file.path));
    if (patchedSha256 === file.upstreamSha256) return [];
    if (!reasons.has(file.path)) throw new Error(`unexpected source patch: ${file.path}`);
    return [{ ...file, patchedSha256, reason: reasons.get(file.path) }];
  });
  if (changed.length !== reasons.size) throw new Error('expected exactly the four scoped source patches');
  fs.writeFileSync(patchesPath, `${JSON.stringify({ format: 1, files: changed }, null, 2)}\n`);
  console.log(`Recorded ${changed.length} explicit source patches.`);
} else {
  const manifest = JSON.parse(fs.readFileSync(manifestPath));
  const patches = JSON.parse(fs.readFileSync(patchesPath));
  const patched = new Map(patches.files.map(file => [file.path, file]));
  let count = 0;
  for (const crate of manifest.crates) {
    const expected = imports.find(candidate => candidate.name === crate.name);
    if (!expected || ['version', 'archiveSha256', 'sourceCommit'].some(key => expected[key] !== crate[key])) throw new Error(`unexpected import provenance: ${crate.name}`);
    const actual = files(path.join(root, 'vendor', crate.directory)).map(file => `${crate.directory}/${file}`);
    if (JSON.stringify(actual) !== JSON.stringify(crate.files.map(file => file.path))) throw new Error(`source file inventory changed: ${crate.directory}`);
    for (const file of crate.files) {
      const patch = patched.get(file.path);
      if (patch && patch.upstreamSha256 !== file.upstreamSha256) throw new Error(`patch baseline mismatch: ${file.path}`);
      const expected = patch?.patchedSha256 ?? file.upstreamSha256;
      if (hash(read(file.path)) !== expected) throw new Error(`unrecorded source change: ${file.path}`);
      patched.delete(file.path);
      count += 1;
    }
  }
  if (patched.size) throw new Error(`patch refers to unimported files: ${[...patched.keys()]}`);
  console.log(`Verified ${count} imported files; ${patches.files.length} explicitly recorded source patches.`);
}
