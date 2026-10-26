// Development-only source-integrity audit; never linked into the native engine.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(root, 'vendor/IMPORTS.json');
const patchesPath = path.join(root, 'vendor/PATCHES.json');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const read = relative => fs.readFileSync(path.join(root, 'vendor', relative));
const MAX_BYTES = 64 * 1024 * 1024;
const MAX_FILES = 4096;

function boundedRead(file) {
  const descriptor = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
  try {
    const stat = fs.fstatSync(descriptor);
    if (!stat.isFile() || stat.size > MAX_BYTES) throw new Error(`unsafe or oversized source file: ${file}`);
    const bytes = Buffer.alloc(stat.size);
    let offset = 0;
    while (offset < bytes.length) {
      const count = fs.readSync(descriptor, bytes, offset, bytes.length - offset, offset);
      if (count === 0) throw new Error(`source file shrank while reading: ${file}`);
      offset += count;
    }
    if (fs.readSync(descriptor, Buffer.alloc(1), 0, 1, offset)) throw new Error(`source file grew while reading: ${file}`);
    return bytes;
  } finally { fs.closeSync(descriptor); }
}

function safeMember(name, directory) {
  const parts = name.split('/');
  if (Buffer.byteLength(name) > 512 || parts.length > 32 || parts[0] !== directory
    || parts.some(part => ['', '.', '..'].includes(part)) || /[\\:\x00-\x1f\x7f]/.test(name)) throw new Error(`unsafe archive member: ${name}`);
  return name;
}

function tarNumber(header, start, length) {
  const raw = header.subarray(start, start + length).toString('ascii').replace(/\0.*$/s, '').trim();
  if (!/^[0-7]+$/.test(raw)) throw new Error(`invalid archive numeric field at ${start}`);
  return Number.parseInt(raw, 8);
}

function tarName(bytes) {
  const end = bytes.indexOf(0);
  return new TextDecoder('utf-8', { fatal: true }).decode(end < 0 ? bytes : bytes.subarray(0, end));
}

function archiveMembers(archive, directory) {
  const bytes = gunzipSync(archive, { maxOutputLength: MAX_BYTES });
  const members = new Map();
  let offset = 0;
  let headers = 0;
  let longName = '';
  while (offset + 512 <= bytes.length) {
    const header = bytes.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) {
      if (longName || bytes.length - offset < 1024 || bytes.subarray(offset).some(byte => byte !== 0)) throw new Error('invalid archive terminator');
      return members;
    }
    if (++headers > MAX_FILES) throw new Error('archive entry count exceeds limit4096');
    const checksum = header.reduce((sum, byte, index) => sum + (index >= 148 && index < 156 ? 32 : byte), 0);
    if (checksum !== tarNumber(header, 148, 8)) throw new Error('archive header checksum mismatch');
    const size = tarNumber(header, 124, 12);
    const end = offset + 512 + size;
    if (size > 16 * 1024 * 1024 || end > bytes.length) throw new Error('archive member size exceeds bounds');
    const content = bytes.subarray(offset + 512, end);
    offset += 512 + Math.ceil(size / 512) * 512;
    const kind = header[156];
    if (kind === 76) {
      if (longName || size > 513) throw new Error('invalid archive long name');
      longName = tarName(content);
      continue;
    }
    const prefix = header.subarray(257, 263).toString('ascii') === 'ustar\0' ? tarName(header.subarray(345, 500)) : '';
    const name = safeMember((longName || [prefix, tarName(header.subarray(0, 100))].filter(Boolean).join('/')).replace(/\/$/, ''), directory);
    longName = '';
    if (kind === 53 && size === 0) continue;
    if (![0, 48].includes(kind)) throw new Error(`unsupported archive member type: ${name}`);
    if (members.has(name)) throw new Error(`duplicate archive member: ${name}`);
    members.set(name, content);
  }
  throw new Error('truncated archive');
}

function importedFiles(directory, members) {
  const location = path.join(root, 'vendor', directory);
  if (!fs.lstatSync(location).isDirectory()) throw new Error(`unsafe extraction directory: ${directory}`);
  const expected = [...members.keys()].filter(name => name !== `${directory}/Cargo.lock` && !name.startsWith(`${directory}/examples/`)).sort();
  const actual = files(location).map(file => safeMember(`${directory}/${file}`, directory));
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error(`archive extraction inventory mismatch: ${directory}`);
  return actual.map(relative => {
    const extracted = boundedRead(path.join(root, 'vendor', relative));
    const original = members.get(relative);
    if (!original.equals(extracted)) throw new Error(`archive member mismatch: ${relative}`);
    return { path: relative, upstreamSha256: hash(original) };
  });
}

function files(directory, prefix = '', budget = { entries: 0 }) {
  if (prefix.split('/').length > 32) throw new Error(`source inventory depth exceeds limit32: ${prefix}`);
  const handle = fs.opendirSync(directory);
  const result = [];
  try {
    for (let entry; (entry = handle.readSync()) !== null;) {
      if (++budget.entries > MAX_FILES) throw new Error(`source inventory exceeds limit4096: ${prefix}`);
      const relative = path.posix.join(prefix, entry.name);
      if (entry.isDirectory()) result.push(...files(path.join(directory, entry.name), relative, budget));
      else if (entry.isFile()) result.push(relative);
      else throw new Error(`unexpected source entry: ${relative}`);
    }
  } finally { handle.closeSync(); }
  return result.sort();
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
    const archive = boundedRead(path.join(archiveDirectory, `${directory}.crate`));
    if (hash(archive) !== crate.archiveSha256) throw new Error(`archive checksum mismatch: ${directory}`);
    return { ...crate, directory, omitted: ['Cargo.lock', 'examples/**'], files: importedFiles(directory, archiveMembers(archive, directory)) };
  });
  fs.writeFileSync(manifestPath, `${JSON.stringify({ format: 1, registry: 'https://crates.io', crates }, null, 2)}\n`, { flag: 'wx' });
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
