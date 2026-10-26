// Developer-only original renderer evidence; dependencies remain outside the repository.
import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const SOURCE_LIMIT = 256 * 1024;
const OUTPUT_LIMIT = 16 * 1024 * 1024;
export const config = Object.freeze({
  startOnLoad: false, securityLevel: 'strict', theme: 'default', deterministicIds: true,
  deterministicIDSeed: 'ive-diagram-oracle-v1', look: 'classic', handDrawnSeed: 42,
  fontFamily: 'Liberation Sans', layout: 'dagre', htmlLabels: false,
  flowchart: { defaultRenderer: 'dagre-wrapper', htmlLabels: false, useMaxWidth: false },
  sequence: { useMaxWidth: false, actorFontFamily: 'Liberation Sans', noteFontFamily: 'Liberation Sans', messageFontFamily: 'Liberation Sans' },
  class: { useMaxWidth: false, htmlLabels: false }, themeVariables: { fontFamily: 'Liberation Sans' },
});

function fatal(message) { return Object.assign(new Error(message), { fatal: true }); }

async function bounded(operation, milliseconds = 30000) {
  let timer;
  try {
    return await Promise.race([operation, new Promise((_, reject) => {
      timer = setTimeout(() => reject(fatal(`oracle operation exceeded ${milliseconds}ms`)), milliseconds);
    })]);
  } finally { clearTimeout(timer); }
}

async function read(file, limit) {
  const stat = await fs.lstat(file);
  if (!stat.isFile() || stat.size > limit) throw new Error(`unsafe or oversized oracle input: ${file}`);
  const bytes = await fs.readFile(file);
  if (bytes.length > limit) throw new Error(`oracle input grew beyond limit: ${file}`);
  return bytes;
}

function kill(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  try { process.kill(-child.pid, 'SIGKILL'); }
  catch { child.kill('SIGKILL'); }
}

export async function runChild(command, args, input = '', timeout = 30000) {
  const child = spawn(command, args, { detached: process.platform !== 'win32', stdio: ['pipe', 'pipe', 'pipe'] });
  const stdout = [];
  const stderr = [];
  let size = 0;
  let failure;
  const timer = setTimeout(() => { failure = fatal(`oracle child exceeded ${timeout}ms: ${command}`); kill(child); }, timeout);
  const collect = output => bytes => {
    size += bytes.length;
    if (size > OUTPUT_LIMIT) { failure = fatal(`oracle child output exceeds ${OUTPUT_LIMIT} bytes: ${command}`); kill(child); }
    else output.push(bytes);
  };
  child.stdout.on('data', collect(stdout));
  child.stderr.on('data', collect(stderr));
  child.stdin.on('error', () => {});
  child.stdin.end(input);
  try {
    return await new Promise((resolve, reject) => {
      child.on('error', error => { failure = error; });
      child.on('close', code => {
        if (failure) reject(failure);
        else if (code !== 0) reject(new Error(`oracle child ${command} exited ${code}: ${Buffer.concat(stderr).toString('utf8').slice(0, 4096)}`));
        else resolve(Buffer.concat(stdout).toString('utf8'));
      });
    });
  } finally { clearTimeout(timer); kill(child); }
}

export async function closeBrowser(browser, milliseconds = 5000) {
  const child = browser.process();
  const exited = child && child.exitCode === null && child.signalCode === null
    ? new Promise(resolve => child.once('exit', resolve)) : Promise.resolve();
  try { await bounded(Promise.resolve().then(() => browser.close()), milliseconds); }
  catch (error) { if (!child) throw error; kill(child); }
  finally { if (child) kill(child); await bounded(exited, milliseconds); }
}

async function writeArtifact(directory, name, bytes) {
  if (Buffer.byteLength(bytes) > OUTPUT_LIMIT) throw fatal(`oracle artifact exceeds ${OUTPUT_LIMIT} bytes: ${name}`);
  await fs.writeFile(path.join(directory, name), bytes, { flag: 'wx' });
  return { path: name, sha256: hash(bytes), bytes: Buffer.byteLength(bytes) };
}

async function inputs(directory) {
  const handle = await fs.opendir(directory);
  const names = [];
  for await (const entry of handle) {
    if (names.length >= 256) throw new Error('oracle corpus exceeds256 directory entries');
    names.push(entry.name);
  }
  const files = names.filter(name => /\.(mmd|puml)$/.test(name)).sort();
  if (files.length === 0 || files.some(name => !/^[\w.-]+\.(mmd|puml)$/.test(name))) throw new Error(`oracle corpus has no safe source filenames: ${directory}`);
  return files;
}

async function renderCase(context, source, name) {
  const remaining = () => Math.max(1, Math.min(30000, context.deadline - Date.now()));
  const svg = await bounded(context.backend.render(source.toString('utf8'), name), remaining());
  if (Buffer.byteLength(svg) > OUTPUT_LIMIT) throw fatal(`oracle SVG exceeds ${OUTPUT_LIMIT} bytes: ${name}`);
  const metrics = await bounded(context.backend.inspect(svg), remaining());
  return { svg, metrics: { ...metrics, svgSha256: hash(svg) } };
}

async function recordCase(context, file) {
  const source = await read(path.join(context.input, file), SOURCE_LIMIT);
  const sourceArtifact = await writeArtifact(context.output, file, source);
  let rendered;
  try {
    rendered = await renderCase(context, source, file);
  } catch (error) {
    if (error.fatal) throw error;
    const failure = await writeArtifact(context.output, `${file}.error.txt`, `${String(error.message).slice(0, 16384)}\n`);
    return { file, status: 'error', source: sourceArtifact, error: failure };
  }
  const svgArtifact = await writeArtifact(context.output, `${file}.svg`, rendered.svg);
  const metricsArtifact = await writeArtifact(context.output, `${file}.json`, `${JSON.stringify(rendered.metrics, null, 2)}\n`);
  return { file, status: 'rendered', source: sourceArtifact, svg: svgArtifact, metrics: metricsArtifact };
}

async function verify(directory, results) {
  for (const result of results) for (const key of ['source', 'svg', 'metrics', 'error']) {
    const artifact = result[key];
    if (artifact && hash(await read(path.join(directory, artifact.path), OUTPUT_LIMIT)) !== artifact.sha256) throw fatal(`oracle artifact hash drift: ${artifact.path}`);
  }
}

export async function runCorpus(options, createBackend = openBackend) {
  const input = path.resolve(options.input);
  const output = path.resolve(options.output);
  const files = await inputs(input);
  await fs.mkdir(output); // Exclusive reservation: even an existing empty directory is refused.
  const backend = await createBackend(options);
  try {
    const results = [];
    const started = Date.now();
    let total = 0;
    for (const file of files) {
      if (Date.now() - started > 120000) throw fatal('oracle corpus exceeded120000ms');
      total += (await fs.lstat(path.join(input, file))).size;
      if (total > OUTPUT_LIMIT) throw fatal('oracle corpus source bytes exceed16MiB');
      results.push(await recordCase({ input, output, backend, deadline: started + 120000 }, file));
    }
    await verify(output, results);
    const manifest = { schema: 1, sourceDirectory: input, outputDirectory: output, node: process.version, platform: process.platform, architecture: process.arch, recordedAt: new Date().toISOString(),
      ...backend.metadata, config, results, summary: { total: results.length, rendered: results.filter(result => result.status === 'rendered').length },
      limitation: 'Original-only DOM geometry/text evidence; not native support, ink bounds or pixel parity. Dates, environment and renderer randomness may vary SVG bytes.' };
    await writeArtifact(output, 'admission.json', `${JSON.stringify(manifest, null, 2)}\n`);
    return manifest;
  } finally { await backend.close(); }
}

async function inspect(page, svg) {
  await page.evaluate(source => {
    const xml = new DOMParser().parseFromString(source, 'image/svg+xml');
    if (xml.querySelector('parsererror,script') || xml.documentElement.localName !== 'svg') throw new Error('oracle returned invalid or scripted SVG');
    if (xml.querySelectorAll('*').length > 50000) throw new Error('oracle SVG exceeds50000 nodes');
    for (const node of xml.querySelectorAll('*')) for (const attribute of node.attributes) {
      if (/^on/i.test(attribute.name)) throw new Error('oracle SVG has event handlers');
    }
  }, svg);
  await page.setContent(svg);
  await page.evaluate(() => document.fonts.ready);
  return page.evaluate(() => {
    const root = document.querySelector('svg');
    const box = node => {
      const value = node.getBBox();
      const result = { x: value.x, y: value.y, width: value.width, height: value.height };
      if (Object.values(result).some(value => !Number.isFinite(value) || Math.abs(value) > 1000000)) throw new Error('oracle SVG has unbounded geometry');
      return result;
    };
    const bounds = box(root);
    if (bounds.width <= 0 || bounds.height <= 0) throw new Error('oracle SVG has empty bounds');
    return { original: { width: root.getAttribute('width'), height: root.getAttribute('height'), viewBox: root.getAttribute('viewBox') }, bounds,
      texts: [...root.querySelectorAll('text')].map(node => ({ label: node.textContent, bounds: box(node), font: getComputedStyle(node).font })),
      geometryCount: root.querySelectorAll('path,rect,circle,ellipse,line,polygon,polyline').length,
      foreignObjectCount: root.querySelectorAll('foreignObject').length,
      externalReferences: [...root.querySelectorAll('[href],[xlink\\:href]')].flatMap(node => ['href', 'xlink:href'].map(key => node.getAttribute(key)).filter(value => value && !value.startsWith('#') && !value.startsWith('data:'))),
      note: 'Root getBBox may include paper; child boxes use local coordinates. Computed CSS font is not proof of the actual shaping face.' };
  });
}

async function openBackend(options) {
  if (!options.oracleRoot) throw new Error('--oracle-root must point to existing local developer dependencies');
  const directory = path.resolve(options.oracleRoot);
  const require = createRequire(path.join(directory, 'package.json'));
  const puppeteer = (await import(pathToFileURL(require.resolve('puppeteer-core')).href)).default;
  const browser = await puppeteer.launch({ executablePath: process.env.ORACLE_CHROMIUM ?? '/home/kb/.cache/ms-playwright/chromium-1243/chrome-linux64/chrome',
    headless: true, timeout: 30000, protocolTimeout: 30000, args: ['--no-sandbox', '--disable-dev-shm-usage', '--disable-background-networking'] });
  try {
    const renderer = await browser.newPage();
    const inspector = await browser.newPage();
    for (const page of [renderer, inspector]) { await page.setOfflineMode(true); await page.setViewport({ width: 1600, height: 1200, deviceScaleFactor: 1 }); }
    await renderer.addScriptTag({ path: path.join(directory, 'node_modules/mermaid/dist/mermaid.min.js') });
    const jar = path.join(directory, 'plantuml-mit-1.2026.9.jar');
    const jarSha256 = hash(await read(jar, 64 * 1024 * 1024));
    const mermaid = JSON.parse(await fs.readFile(path.join(directory, 'node_modules/mermaid/package.json'))).version;
    const puppeteerVersion = JSON.parse(await fs.readFile(path.join(directory, 'node_modules/puppeteer-core/package.json'))).version;
    return { metadata: { browser: await browser.version(), mermaid, puppeteer: puppeteerVersion, plantuml: { version: '1.2026.9', jarSha256, identity: 'local observed bytes, not external authentication' } },
      render: (source, file) => file.endsWith('.puml') ? runChild('java', ['-Djava.awt.headless=true', '-DPLANTUML_SECURITY_PROFILE=SANDBOX', '-jar', jar, '-charset', 'UTF-8', '-tsvg', '-pipe'], source)
        : renderer.evaluate(async ({ source, config, id }) => { document.body.innerHTML = ''; window.mermaid.initialize(config); return (await window.mermaid.render(id, source)).svg; }, { source, config, id: `oracle-${hash(file).slice(0, 12)}` }),
      inspect: svg => inspect(inspector, svg), close: () => closeBrowser(browser) };
  } catch (error) { await closeBrowser(browser); throw error; }
}

export async function main(args) {
  const [mode, input, output, flag, oracleRoot] = args;
  if (mode !== '--corpus' || !input || !output || flag !== '--oracle-root' || !oracleRoot || args.length !== 5) throw new Error('usage: oracles.mjs --corpus INPUT FRESH_OUTPUT --oracle-root LOCAL_DEPENDENCY_DIRECTORY');
  const manifest = await runCorpus({ input, output, oracleRoot });
  console.log(JSON.stringify({ outputDirectory: manifest.outputDirectory, ...manifest.summary }));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).catch(error => { console.error(error); process.exitCode = 1; });
