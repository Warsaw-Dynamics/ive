// Developer-only native CLI evidence; never linked into either runtime crate.
import fs from 'node:fs/promises';
import { constants } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const defaults = Object.freeze({ cases: 256, directoryEntries: 512, sourceBytes: 256 * 1024,
  sourceTotalBytes: 16 * 1024 * 1024, svgBytes: 16 * 1024 * 1024, logBytes: 1024 * 1024,
  artifactBytes: 64 * 1024 * 1024, binaryBytes: 256 * 1024 * 1024,
  manifestBytes: 4 * 1024 * 1024, caseMilliseconds: 15000, corpusMilliseconds: 300000 });
const safeName = name => /^[\w.-]+$/.test(name) && name !== '.' && name !== '..' && Buffer.byteLength(name) <= 240;
const json = value => `${JSON.stringify(value, null, 2)}\n`;

function limits(overrides = {}) {
  for (const [key, value] of Object.entries(overrides)) {
    if (!(key in defaults) || !Number.isSafeInteger(value) || value <= 0 || value > defaults[key]) throw new Error(`invalid native corpus bound ${key}: ${value}`);
  }
  return Object.freeze({ ...defaults, ...overrides });
}

function absolute(value, name) {
  if (typeof value !== 'string' || !path.isAbsolute(value)) throw new Error(`native corpus ${name} must be an absolute path: ${value}`);
  return path.resolve(value);
}

async function scan(file, maximum, consume) {
  const handle = await fs.open(file, constants.O_RDONLY | constants.O_NOFOLLOW).catch(error => {
    if (error.code === 'ELOOP') throw new Error(`unsafe symbolic-link input: ${file}`);
    throw error;
  });
  try {
    const stat = await handle.stat();
    if (!stat.isFile() || stat.size > maximum) throw new Error(`unsafe or oversized regular file: ${file}`);
    const chunk = Buffer.alloc(Math.min(65536, maximum + 1));
    let size = 0;
    for (;;) {
      const { bytesRead } = await handle.read(chunk, 0, Math.min(chunk.length, maximum - size + 1));
      if (bytesRead === 0) return size;
      size += bytesRead;
      if (size > maximum) throw new Error(`file grew beyond native corpus bound: ${file}`);
      consume(chunk.subarray(0, bytesRead));
    }
  } finally { await handle.close(); }
}

async function read(file, maximum) {
  const chunks = [];
  await scan(file, maximum, chunk => chunks.push(Buffer.from(chunk)));
  return Buffer.concat(chunks);
}

async function identify(file, maximum) {
  const digest = createHash('sha256');
  const bytes = await scan(file, maximum, chunk => digest.update(chunk));
  return { path: file, bytes, sha256: digest.digest('hex') };
}

async function names(directory, maximum) {
  const result = [];
  const handle = await fs.opendir(directory);
  for await (const entry of handle) {
    if (result.length >= maximum) throw new Error(`native corpus directory exceeds ${maximum} entries: ${directory}`);
    result.push(entry.name);
  }
  return result.sort();
}

async function inputs(options, bound) {
  const files = [];
  let total = 0;
  for (const [language, extension] of [['mermaid', '.mmd'], ['plantuml', '.puml']]) {
    const directory = path.join(options.fixtures, language);
    for (const name of await names(directory, bound.directoryEntries)) {
      if (!name.endsWith(extension)) continue;
      if (!safeName(name) || files.length >= bound.cases) throw new Error(`unsafe filename or excessive native corpus cases: ${name}`);
      const source = await read(path.join(directory, name), Math.min(bound.sourceBytes, bound.sourceTotalBytes - total));
      total += source.length;
      files.push({ file: `${language}/${name}`, language, name, source });
    }
  }
  if (files.length === 0) throw new Error(`native corpus has no fixture sources: ${options.fixtures}`);
  return files;
}

function kill(child) {
  // Kill the group even if its leader exited while a descendant holds the pipes open.
  if (child.pid === undefined) return;
  try { process.kill(-child.pid, 'SIGKILL'); }
  catch { if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL'); }
}

function childEnvironment() {
  const env = { ...process.env, PATH: '' };
  delete env.DISPLAY;
  delete env.WAYLAND_DISPLAY;
  return env;
}

async function runExport(context, paths) {
  const started = performance.now();
  const args = ['--language', paths.language, '--render', paths.source, '--output', paths.svg];
  const child = spawn(context.viewer, args, { detached: process.platform !== 'win32', env: childEnvironment(), stdio: ['ignore', 'pipe', 'pipe'] });
  const streams = { stdout: [], stderr: [] };
  const state = { size: 0, failure: '', checking: false };
  const remaining = Math.min(context.bound.caseMilliseconds, Math.max(1, context.deadline - context.clock.now()));
  const fail = reason => { if (state.failure === '') state.failure = reason; kill(child); };
  const timer = setTimeout(() => fail('timeout'), remaining);
  const watcher = setInterval(() => checkOutput(paths.svg, context, state, fail), 20);
  for (const key of ['stdout', 'stderr']) child[key].on('data', chunk => {
    state.size += chunk.length;
    if (state.size > context.bound.logBytes) fail('output-limit');
    else streams[key].push(chunk);
  });
  try {
    const exit = await new Promise(resolve => {
      child.on('error', error => { state.failure = 'spawn-error'; state.detail = error.message; });
      child.on('close', (code, signal) => resolve({ ...(code === null ? {} : { exitCode: code }), ...(signal === null ? {} : { signal }) }));
    });
    return { stdout: Buffer.concat(streams.stdout), stderr: Buffer.concat(streams.stderr),
      status: { ...exit, outcome: state.failure || (exit.exitCode === 0 ? 'rendered' : 'failed'), ...(state.detail ? { detail: state.detail } : {}), milliseconds: Math.ceil(performance.now() - started) } };
  } finally { clearTimeout(timer); clearInterval(watcher); kill(child); }
}

async function checkOutput(file, context, state, fail) {
  if (state.checking) return;
  state.checking = true;
  try {
    const stat = await fs.lstat(file);
    if (!stat.isFile() || stat.size > Math.min(context.bound.svgBytes, context.bound.artifactBytes - context.total)) fail('output-limit');
  } catch (error) { if (error.code !== 'ENOENT') fail(`output-error: ${error.message}`); }
  finally { state.checking = false; }
}

async function artifact(context, name, bytes) {
  if (!safeName(name) || context.total + bytes.length > context.bound.artifactBytes) throw new Error(`native corpus aggregate artifact bound exceeded: ${name}`);
  await fs.writeFile(path.join(context.output, name), bytes, { flag: 'wx' });
  context.total += bytes.length;
  context.artifacts.push({ path: name, bytes: bytes.length, sha256: hash(bytes) });
  return name;
}

async function original(context, file, source) {
  const matches = context.original.results.filter(row => row.file === file);
  if (matches.length === 0) return 'not-observed';
  if (matches.length !== 1 || matches[0].source?.sha256 !== hash(source) || !['rendered', 'error'].includes(matches[0].status)) throw new Error(`original source/status mismatch: ${file}`);
  return matches[0].status;
}

async function takeSvg(context, name, status) {
  const file = path.join(context.output, name);
  try {
    const stat = await fs.lstat(file);
    if (stat.isFile() && stat.size > Math.min(context.bound.svgBytes, context.bound.artifactBytes - context.total)) {
      await fs.unlink(file);
      return { ...status, outcome: 'output-limit', discardedSvgBytes: stat.size };
    }
    const bytes = await read(file, Math.min(context.bound.svgBytes, context.bound.artifactBytes - context.total));
    context.total += bytes.length;
    context.artifacts.push({ path: name, bytes: bytes.length, sha256: hash(bytes) });
    return bytes.length === 0 ? { ...status, outcome: 'failed', detail: 'native CLI emitted an empty SVG' } : status;
  } catch (error) {
    if (error.code === 'ENOENT') return status.outcome === 'rendered' ? { ...status, outcome: 'failed', detail: 'native CLI emitted no SVG' } : status;
    throw error;
  }
}

async function recordCase(context, input) {
  if (context.clock.now() >= context.deadline) throw new Error('native corpus aggregate deadline exceeded');
  const prefix = `${input.language}--${input.name}`;
  const source = await artifact(context, prefix, input.source);
  const paths = { language: input.language, source: path.join(context.output, source), svg: path.join(context.output, `${prefix}.svg`) };
  const reference = await original(context, input.name, input.source);
  const exported = await runExport(context, paths);
  const status = await takeSvg(context, `${prefix}.svg`, exported.status);
  const stdout = await artifact(context, `${prefix}.stdout.log`, exported.stdout);
  const stderr = await artifact(context, `${prefix}.stderr.log`, exported.stderr);
  const statusFile = await artifact(context, `${prefix}.status.json`, Buffer.from(json(status)));
  const output = context.artifacts.some(entry => entry.path === `${prefix}.svg`) ? { svg: `${prefix}.svg` } : {};
  return { file: input.file, language: input.language, native: status.outcome, original: reference, source, stdout, stderr, status: statusFile, ...output };
}

function summary(results) {
  const rendered = results.filter(row => row.native === 'rendered').length;
  return { total: results.length, rendered, failed: results.length - rendered };
}

async function loadOriginal(context, file) {
  if (file === undefined) return { results: [] };
  const bytes = await read(absolute(file, 'original manifest'), context.bound.manifestBytes);
  const parsed = JSON.parse(bytes);
  if (!Array.isArray(parsed.results) || parsed.results.length > context.bound.cases) throw new Error(`invalid original manifest results: ${file}`);
  await artifact(context, 'original-admission.json', bytes);
  return parsed;
}

export async function runCorpus(options, clock = performance) {
  const bound = limits(options.limits);
  const fixtures = absolute(options.fixtures, 'fixtures');
  const output = absolute(options.output, 'output');
  const viewer = absolute(options.viewer, 'viewer');
  if (output === fixtures || output.startsWith(`${fixtures}${path.sep}`)) throw new Error('native corpus output must be outside fixture inputs');
  const sources = await inputs({ fixtures }, bound);
  await fs.mkdir(output); // Exclusive even when a preexisting directory is empty.
  const binary = await identify(viewer, bound.binaryBytes);
  const context = { output, viewer, bound, clock, artifacts: [], total: 0, deadline: clock.now() + bound.corpusMilliseconds };
  context.original = await loadOriginal(context, options.original);
  const results = [];
  const startedAt = new Date().toISOString();
  for (const input of sources) results.push(await recordCase(context, input));
  const after = await identify(viewer, bound.binaryBytes);
  if (after.sha256 !== binary.sha256) throw new Error('native viewer binary changed during corpus run');
  const manifest = { schema: 1, fixtureRoot: fixtures, outputDirectory: output, binary, limits: bound, startedAt, finishedAt: new Date().toISOString(),
    environment: { node: process.version, platform: process.platform, architecture: process.arch, childPath: '', childDisplay: 'removed' },
    expectation: 'Every fixture must render natively; original error status never authorizes native failure.',
    limitation: 'Native deterministic CLI exit status/nonempty export only. Independent SVG safety/usvg and semantic checks remain required. No actual family/measurement telemetry, native UI capture, topology, geometry or pixel comparison is inferred. Original manifest is status provenance only; original SVG/metrics closure is verified separately.',
    artifacts: context.artifacts, results, summary: summary(results) };
  const artifacts = await verifyArtifacts(output, manifest);
  verifyRows(manifest, artifacts);
  const bytes = Buffer.from(json(manifest));
  if (bytes.length > bound.manifestBytes || context.total + bytes.length > bound.artifactBytes) throw new Error('native corpus manifest exceeds aggregate bound');
  if (clock.now() >= context.deadline) throw new Error('native corpus aggregate deadline exceeded before manifest publication');
  await fs.writeFile(path.join(output, 'admission.json'), bytes, { flag: 'wx' });
  return manifest;
}

async function verifyArtifacts(directory, manifest) {
  const registered = new Map();
  const bound = limits(manifest.limits);
  let total = 0;
  for (const artifact of manifest.artifacts) {
    if (!safeName(artifact.path) || registered.has(artifact.path) || !Number.isSafeInteger(artifact.bytes) || artifact.bytes < 0 || !/^[a-f0-9]{64}$/.test(artifact.sha256)) throw new Error(`invalid native corpus artifact: ${artifact.path}`);
    total += artifact.bytes;
    if (total > bound.artifactBytes) throw new Error('native corpus aggregate artifact bound exceeded');
    const bytes = await read(path.join(directory, artifact.path), artifact.bytes).catch(error => { throw new Error(`artifact size drift or unsafe file: ${artifact.path}: ${error.message}`); });
    if (bytes.length !== artifact.bytes) throw new Error(`artifact size drift: ${artifact.path}`);
    if (hash(bytes) !== artifact.sha256) throw new Error(`artifact hash drift: ${artifact.path}`);
    registered.set(artifact.path, bytes);
  }
  const actual = (await names(directory, defaults.cases * 6 + 2)).filter(name => name !== 'admission.json');
  if (JSON.stringify(actual) !== JSON.stringify([...registered.keys()].sort())) throw new Error('native corpus artifact inventory mismatch');
  return registered;
}

function verifyRows(manifest, artifacts) {
  if (!Array.isArray(manifest.results) || manifest.results.length === 0 || manifest.results.length > defaults.cases) throw new Error('invalid native corpus results');
  const files = new Set();
  const referenced = new Set(artifacts.has('original-admission.json') ? ['original-admission.json'] : []);
  const originals = artifacts.has('original-admission.json') ? JSON.parse(artifacts.get('original-admission.json')).results : [];
  if (!Array.isArray(originals) || originals.length > defaults.cases) throw new Error('invalid original manifest results');
  for (const row of manifest.results) {
    const prefix = `${row.language}--${row.file?.split('/')[1]}`;
    if (!['mermaid', 'plantuml'].includes(row.language) || row.file !== `${row.language}/${row.file?.split('/')[1]}` || files.has(row.file)) throw new Error(`invalid native corpus case: ${row.file}`);
    files.add(row.file);
    for (const [key, suffix] of [['source', ''], ['stdout', '.stdout.log'], ['stderr', '.stderr.log'], ['status', '.status.json'], ...(row.svg ? [['svg', '.svg']] : [])]) {
      if (row[key] !== `${prefix}${suffix}` || !artifacts.has(row[key])) throw new Error(`native case artifact mismatch: ${row.file}/${key}`);
      referenced.add(row[key]);
    }
    const status = JSON.parse(artifacts.get(row.status));
    if (!['rendered', 'failed', 'timeout', 'output-limit', 'spawn-error'].includes(row.native) || status.outcome !== row.native || (row.native === 'rendered' && (status.exitCode !== 0 || !row.svg || artifacts.get(row.svg).length === 0))) throw new Error(`native case status mismatch: ${row.file}`);
    const original = originals.filter(entry => entry.file === row.file.split('/')[1]);
    if (original.length > 1 || row.original !== (original[0]?.status ?? 'not-observed') || (original.length && original[0].source?.sha256 !== hash(artifacts.get(row.source)))) throw new Error(`original source/status mismatch: ${row.file}`);
  }
  if (referenced.size !== artifacts.size || JSON.stringify(manifest.summary) !== JSON.stringify(summary(manifest.results))) throw new Error('native corpus result/artifact summary mismatch');
}

export async function verifyCorpus(directory) {
  const output = absolute(directory, 'output');
  const bytes = await read(path.join(output, 'admission.json'), defaults.manifestBytes);
  const manifest = JSON.parse(bytes);
  if (manifest.schema !== 1 || !Array.isArray(manifest.artifacts) || manifest.artifacts.length > defaults.cases * 6 + 1) throw new Error('invalid native corpus manifest');
  const artifacts = await verifyArtifacts(output, manifest);
  if (bytes.length + manifest.artifacts.reduce((sum, artifact) => sum + artifact.bytes, 0) > limits(manifest.limits).artifactBytes) throw new Error('native corpus manifest exceeds aggregate bound');
  verifyRows(manifest, artifacts);
  return manifest;
}

export async function main(args) {
  if (args[0] === 'check' && args.length === 2) {
    const manifest = await verifyCorpus(args[1]);
    return { ...manifest.summary, verified: true };
  }
  const pairs = new Map();
  for (let index = 1; index + 1 < args.length; index += 2) {
    if (pairs.has(args[index])) throw new Error(`duplicate native corpus argument: ${args[index]}`);
    pairs.set(args[index], args[index + 1]);
  }
  if (args[0] !== 'export' || args.length % 2 !== 1 || [...pairs.keys()].some(key => !['--fixtures', '--output', '--viewer', '--original'].includes(key)) || ['--fixtures', '--output', '--viewer'].some(key => !pairs.has(key))) throw new Error('usage: native-corpus.mjs export --fixtures ABSOLUTE_FIXTURE_ROOT --output FRESH_ABSOLUTE_OUTPUT --viewer ABSOLUTE_VIEWER [--original ABSOLUTE_ADMISSION_JSON]; or check ABSOLUTE_OUTPUT');
  const manifest = await runCorpus({ fixtures: pairs.get('--fixtures'), output: pairs.get('--output'), viewer: pairs.get('--viewer'), ...(pairs.has('--original') ? { original: pairs.get('--original') } : {}) });
  return { ...manifest.summary, verified: true };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(result => {
  console.log(JSON.stringify(result));
  if (result.failed > 0) process.exitCode = 1;
}).catch(error => { console.error(error); process.exitCode = 1; });
