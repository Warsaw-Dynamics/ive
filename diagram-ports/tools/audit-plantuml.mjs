// Development-only provenance audit; no filesystem/process code enters the engine.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = '/mnt/data/Projects/plantuml-mit/src';
const artifact = '/mnt/data/Projects/plantuml-mit/plantuml-mit-1.2026.9-sources.jar';
const artifactHash = '88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3';
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const safe = value => typeof value === 'string' && !value.split('/').some(part => part === '..' || part === '.' || part === '');

export function validateManifest(manifest) {
  if (manifest.edition !== 'PlantUML MIT' || manifest.version !== '1.2026.9' || manifest.sourceRoot !== sourceRoot || manifest.artifact.path !== artifact || manifest.artifact.sha256 !== artifactHash) throw new Error('unauthorized PlantUML edition, source root or artifact');
  for (const file of manifest.sourceFiles) {
    if (!safe(file.path) || !/^net\/sourceforge\/plantuml\/(?:BlockUmlBuilder\.java|PSystemBuilder\.java|utils\/CharHidder\.java|(?:core|command|sequencediagram\/command|classdiagram|objectdiagram\/command|descdiagram|cucadiagram|klimt\/(?:font|creole\/(?:legacy|command)))\/[A-Za-z0-9_/]+\.java)$/.test(file.path)) throw new Error(`unauthorized source path: ${file.path}`);
    if (!/^[a-f0-9]{64}$/.test(file.sha256) || file.notice !== 'provenance/LICENSE-PlantUML-MIT.txt') throw new Error(`invalid source hash/notice: ${file.path}`);
    if (file.rust.some(target => !manifest.nativeModules.includes(target))) throw new Error(`unregistered Rust destination: ${file.path}`);
  }
  for (const file of manifest.fixtures) {
    if (!/^fixtures\/plantuml\/[a-z]+\.puml$/.test(file.path) || !/^[a-f0-9]{64}$/.test(file.sha256)) throw new Error(`invalid fixture: ${file.path}`);
  }
  if (manifest.nativeModules.some(target => !/^engine\/src\/plantuml\/[a-z]+\.rs$/.test(target))) throw new Error('invalid native module path');
}

export function audit(manifest) {
  validateManifest(manifest);
  if (sha256(fs.readFileSync(artifact)) !== artifactHash) throw new Error('source artifact checksum mismatch');
  for (const file of manifest.sourceFiles) {
    const extracted = fs.readFileSync(path.join(sourceRoot, file.path));
    const archived = execFileSync('unzip', ['-p', artifact, file.path], { timeout: 5000, maxBuffer: 2 * 1024 * 1024 });
    if (sha256(extracted) !== file.sha256 || !archived.equals(extracted)) throw new Error(`source/ZIP checksum mismatch: ${file.path}`);
    if (!extracted.toString().split('package ')[0].includes('under the terms of the MIT License.')) throw new Error(`MIT-edition grant missing: ${file.path}`);
  }
  for (const file of manifest.fixtures) {
    if (sha256(fs.readFileSync(path.join(root, file.path))) !== file.sha256) throw new Error(`fixture checksum mismatch: ${file.path}`);
  }
  const actual = fs.readdirSync(path.join(root, 'engine/src/plantuml')).map(name => `engine/src/plantuml/${name}`).sort();
  if (JSON.stringify(actual) !== JSON.stringify([...manifest.nativeModules].sort())) throw new Error('native module inventory changed');
  if (!fs.readFileSync(path.join(root, 'provenance/LICENSE-PlantUML-MIT.txt'), 'utf8').includes('Hisashi Miyashita')) throw new Error('contributor notice missing');
  return `Verified ${manifest.sourceFiles.length} MIT-edition source/ZIP hashes, ${manifest.fixtures.length} fresh fixture hashes and ${actual.length} native modules.`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(audit(JSON.parse(fs.readFileSync(path.join(root, 'provenance/plantuml-mit-1.2026.9.json')))));
}
