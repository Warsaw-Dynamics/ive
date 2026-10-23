// Development-only provenance audit; no filesystem/process code enters the engine.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const parserSources = JSON.parse(fs.readFileSync(path.join(root, 'provenance/plantuml-parser.json'))).authoredFiles;
const sourceRoot = '/mnt/data/Projects/plantuml-mit/src';
const artifact = '/mnt/data/Projects/plantuml-mit/plantuml-mit-1.2026.9-sources.jar';
const artifactHash = '88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3';
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const safe = value => typeof value === 'string' && !value.split('/').some(part => part === '..' || part === '.' || part === '');

export function validateManifest(manifest) {
  if (manifest.edition !== 'PlantUML MIT' || manifest.version !== '1.2026.9' || manifest.sourceRoot !== sourceRoot || manifest.artifact.path !== artifact || manifest.artifact.sha256 !== artifactHash) throw new Error('unauthorized PlantUML edition, source root or artifact');
  for (const file of manifest.sourceFiles) {
    if (!safe(file.path) || /\/(?:gen|h|smetana|elk)\//.test(file.path) || !/^net\/sourceforge\/plantuml\/(?:ebnf\/(?:EbnfExpression|PSystemEbnf|Symbol|CommandEBnfSingleLine|UBrexCommandEbnfMultilines|ETile|ETileEmpty|ETileBox|ETileConcatenation|ETileAlternation|ETileOptional2|ETileOneOrMore|ETileZeroOrMore|ETileWithCircles|ETileRegexGroup|ETileRegexGroupAllBut)\.java|regexdiagram\/(?:RegexExpression|PSystemRegex|ReTokenType|GroupSplitter)\.java|chart\/(?:ChartDiagramFactory|ChartDiagram|ChartAxis|ChartSeries|ChartRenderer|BarRenderer|LineRenderer|command\/CommandChart(?:HAxis|VAxis|Bar|Line|Legend))\.java|cheneer\/(?:ChenEerDiagram|command\/(?:CommandCreateEntity|CommandCreateAttribute|CommandAssociate|CommandEndGroup))\.java|svek\/image\/EntityImageChen(?:Entity|Relationship|Attribute)\.java|decoration\/(?:LinkType|LinkStyle|symbol\/(?:USymbols|USymbolRectangle))\.java|klimt\/color\/ColorTrieNode\.java|filesdiagram\/(?:FEntry|FilesDiagram|FilesListing)\.java|board\/(?:Activity|BNode|BoardDiagram|CommandBoardPlus|CardBox|PostIt)\.java|hcl\/(?:HclParser|HclSource)\.java|packetdiag\/(?:PacketDiagram|PacketBlock|PacketIndicator|command\/(?:CommandNumRange|CommandColWidth|CommandNodeHeight|CommandScaleInterval|CommandScaleDirection|CommandSameHeight|CommandPacketDiagStart|CommandPacketDiagEnd))\.java|nwdiag\/(?:CommandNwDiagInit|CommandNetwork|CommandElement|CommandProperty|CommandGroup|CommandEndSomething|NwDiagram|core\/(?:NServer|Network|NwGroup))\.java|BlockUmlBuilder\.java|PSystemBuilder\.java|salt\/(?:DataSourceImpl|Positionner2|Cell|PSystemSalt|factory\/ElementFactory(?:Pyramid|Border|Button|TextField|CheckboxOn|CheckboxOff|RadioOn|RadioOff|Tree|Tab|Menu|Droplist|Image|RetrieveFromDictonnary|Line|Scroll)|element\/(?:TableStrategy|ElementPyramid|ElementRadioCheckbox|ElementTextField|ElementButton|AbstractElementText))\.java|gantt\/(?:GanttDiagram|OpenClose|command\/NaturalGanttCommand|lang\/(?:SubjectTask|SubjectGantt|ComplementDuration|ComplementBeforeOrAfterOrAtTaskStartOrEnd|AbstractComplementTaskInstant|DayPattern|TimeResolution|Verbs)|core\/(?:TaskCode|TaskInstant|TaskImpl)|solver\/Solver|draw\/(?:TaskDrawDiamond|TaskDrawRegular))\.java|timingdiagram\/(?:TimingDiagramFactory|TimingDiagram|TimingRuler|AbstractStatePlayer|ChangeState|TimeConstraint|graphic\/(?:PanelsRobust|PanelsConcise|PanelsState)|command\/(?:CommandRobustConcise|CommandAtTime|CommandAtPlayer|CommandChangeState|CommandChangeStateByTime|CommandChangeStateByPlayerCode|CommandConstraint|CommandTimeMessage|TimeTickBuilder))\.java|tim\/(?:EaterAffectationDefine|TContext|VariableManager)\.java|TitledDiagram\.java|style\/(?:FromSkinparamToStyle|parser\/RawStyleParser|value\/(?:Specificity|ValueImpl))\.java|klimt\/creole\/Display\.java|utils\/(?:CharHidder|Direction)\.java|text\/TLineType\.java|(?:core|command|sequencediagram\/command|classdiagram|objectdiagram\/command|descdiagram|cucadiagram|statediagram|activitydiagram3|mindmap|wbs|json|jsondiagram|yaml(?:\/parser)?|klimt\/(?:font|creole\/(?:legacy|command)))\/[A-Za-z0-9_/]+\.java)$/.test(file.path)) throw new Error(`unauthorized source path: ${file.path}`);
    if (!/^[a-f0-9]{64}$/.test(file.sha256) || file.notice !== 'provenance/LICENSE-PlantUML-MIT.txt') throw new Error(`invalid source hash/notice: ${file.path}`);
    if (file.rust.some(target => !manifest.nativeModules.includes(target) && !parserSources.includes(target))) throw new Error(`unregistered Rust destination: ${file.path}`);
  }
  for (const file of manifest.fixtures) {
    if (!/^fixtures\/plantuml\/[a-z]+\.puml$/.test(file.path) || !/^[a-f0-9]{64}$/.test(file.sha256)) throw new Error(`invalid fixture: ${file.path}`);
  }
  if (manifest.nativeModules.some(target => !/^engine\/src\/plantuml\/[a-z]+\.rs$/.test(target))) throw new Error('invalid native module path');
}

export function audit(manifest) {
  validateManifest(manifest);
  auditParserInventory();
  if (sha256(fs.readFileSync(artifact)) !== artifactHash) throw new Error('source artifact checksum mismatch');
  for (const file of manifest.sourceFiles) {
    const extracted = fs.readFileSync(path.join(sourceRoot, file.path));
    const archived = execFileSync('unzip', ['-p', artifact, file.path], { timeout: 5000, maxBuffer: 2 * 1024 * 1024 });
    if (sha256(extracted) !== file.sha256 || !archived.equals(extracted)) throw new Error(`source/ZIP checksum mismatch: ${file.path}`);
    if (!extracted.toString().split('package ')[0].includes('under the terms of the MIT License.')) throw new Error(`MIT-edition grant missing: ${file.path}`);
    const credits = extracted.toString().split('package ')[0].split('\n').map(line => line.trim()).filter(line => /(?:Original\s+Author|Contribut(?:ion|or)s?)\s*:/i.test(line));
    for (const credit of credits) {
      if (!file.copyrights.includes(credit)) throw new Error(`missing source credit: ${file.path}: ${credit}`);
    }
    const copyrights = extracted.toString().split('package ')[0].split('\n').map(line => line.trim()).filter(line => /\bCopyright\s+(?:\(c\)\s+)?(?:19|20)\d{2}/i.test(line));
    for (const copyright of copyrights) {
      if (!file.copyrights.includes(copyright)) throw new Error(`missing source copyright: ${file.path}: ${copyright}`);
    }
    if (file.path.startsWith('net/sourceforge/plantuml/json/')) {
      const embeddedNotice = extracted.toString().match(/\/\*\*+\n \* Copyright \(c\) 2016 EclipseSource\.[\s\S]*?\*\//)?.[0];
      const normalizeWhitespace = value => value.split('\n').map(line => line.trimEnd()).join('\n');
      if (!embeddedNotice || !normalizeWhitespace(fs.readFileSync(path.join(root, file.notice), 'utf8')).includes(normalizeWhitespace(embeddedNotice))) throw new Error(`minimal-json MIT grant missing: ${file.path}`);
    }
  }
  for (const file of manifest.fixtures) {
    if (sha256(fs.readFileSync(path.join(root, file.path))) !== file.sha256) throw new Error(`fixture checksum mismatch: ${file.path}`);
  }
  const actual = fs.readdirSync(path.join(root, 'engine/src/plantuml')).filter(name => !['parser', 'generated'].includes(name)).map(name => `engine/src/plantuml/${name}`).sort();
  if (JSON.stringify(actual) !== JSON.stringify([...manifest.nativeModules].sort())) throw new Error('native module inventory changed');
  if (!fs.readFileSync(path.join(root, 'provenance/LICENSE-PlantUML-MIT.txt'), 'utf8').includes('Hisashi Miyashita')) throw new Error('contributor notice missing');
  for (const notice of ['provenance/LICENSE-PlantUML-MIT.txt', 'THIRD-PARTY-NOTICES.md']) {
    if (!fs.readFileSync(path.join(root, notice), 'utf8').includes('Serge Wenger')) throw new Error(`contributor notice missing: ${notice}: Serge Wenger`);
  }
  return `Verified ${manifest.sourceFiles.length} MIT-edition source/ZIP hashes, ${manifest.fixtures.length} fresh fixture hashes and ${actual.length} native modules.`;
}

export function auditParserInventory(directory = root) {
  const inventory = JSON.parse(fs.readFileSync(path.join(directory, 'provenance/plantuml-parser.json')));
  const actual = ['parser', 'generated'].flatMap(subdirectory => fs.readdirSync(path.join(directory, 'engine/src/plantuml', subdirectory), { recursive: true, withFileTypes: true })
    .filter(entry => !entry.isDirectory()).map(entry => path.relative(directory, path.join(entry.parentPath, entry.name)))).sort();
  const expected = [...inventory.authoredFiles, ...inventory.generatedFiles].sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error('authored/generated PlantUML parser inventory changed');
  const directories = fs.readdirSync(path.join(directory, 'engine/src/plantuml'), { withFileTypes: true }).filter(entry => entry.isDirectory()).map(entry => entry.name).sort();
  if (JSON.stringify(directories) !== JSON.stringify(['generated', 'parser'])) throw new Error('unexpected PlantUML parser directory');
  auditGenerated(directory, inventory);
  auditResponsibilities(directory, inventory);
}

function auditGenerated(directory, inventory) {
  const manifest = JSON.parse(fs.readFileSync(path.join(directory, inventory.generationInventory)));
  const grammars = inventory.authoredFiles.filter(file => file.endsWith('.lalrpop')).sort();
  const expected = grammars.map(grammar => ({ grammar, generated: grammar.replace('/parser/grammar/', '/generated/').replace('.lalrpop', '.rs') }));
  const pairs = manifest.grammars.map(({ grammar, generated }) => ({ grammar, generated }));
  if (JSON.stringify(pairs) !== JSON.stringify(expected)) throw new Error('generated grammar mapping inventory changed');
  const outputs = [...expected.map(record => record.generated), inventory.generationInventory].sort();
  if (JSON.stringify(outputs) !== JSON.stringify([...inventory.generatedFiles].sort())) throw new Error('generated output inventory changed');
  const registration = fs.readFileSync(path.join(directory, 'engine/src/plantuml/parser/mod.rs'), 'utf8');
  const registered = [...registration.matchAll(/#\[path = "\.\.\/(generated\/[a-z_]+\.rs)"\]/g)].map(match => `engine/src/plantuml/${match[1]}`).sort();
  if (JSON.stringify(registered) !== JSON.stringify(expected.map(record => record.generated).sort())) throw new Error('generated module registration inventory changed');
  for (const record of manifest.grammars) {
    for (const kind of ['grammar', 'generated']) {
      const file = path.join(directory, record[kind]);
      if (!fs.lstatSync(file).isFile() || sha256(fs.readFileSync(file)) !== record[`${kind}Sha256`]) throw new Error(`${kind} checksum mismatch: ${record[kind]}`);
    }
  }
}

function functionSymbols(source) {
  const seen = new Map();
  return [...source.matchAll(/^\s*(?:pub(?:\([^\n]*?\))?\s+)?(?:(?:async|const|unsafe)\s+)*(?:extern\s+"[^"\n]+"\s+)?fn\s+((?:r#)?[\p{ID_Start}_][\p{ID_Continue}]*)/gmu)].map(match => {
    const count = (seen.get(match[1]) ?? 0) + 1;
    seen.set(match[1], count);
    return `${match[1]}#${count}`;
  });
}

function auditResponsibilities(directory, inventory) {
  const provenance = JSON.parse(fs.readFileSync(path.join(directory, 'provenance/plantuml-mit-1.2026.9.json')));
  const ledger = JSON.parse(fs.readFileSync(path.join(directory, 'provenance/plantuml-parser-responsibilities.json')));
  const expectedFiles = [...provenance.nativeModules, ...inventory.authoredFiles.filter(file => file.endsWith('.rs'))].sort();
  const nativeFiles = fs.readdirSync(path.join(directory, 'engine/src/plantuml'), { withFileTypes: true }).filter(entry => !entry.isDirectory()).map(entry => `engine/src/plantuml/${entry.name}`).sort();
  if (JSON.stringify(nativeFiles) !== JSON.stringify([...provenance.nativeModules].sort())) throw new Error('native module inventory changed');
  if (JSON.stringify(ledger.modules.map(module => module.path).sort()) !== JSON.stringify(expectedFiles)) throw new Error('function responsibility file inventory changed');
  const roles = new Set(['generated-facade', 'lexical', 'semantic', 'layout', 'infrastructure', 'test-adapter']);
  for (const module of ledger.modules) {
    const file = path.join(directory, module.path);
    if (!fs.lstatSync(file).isFile()) throw new Error(`invalid responsibility source: ${module.path}`);
    const source = fs.readFileSync(file, 'utf8');
    if (/\b(?:struct|enum|type)\s+Line\b|\bsource::Line\b/.test(source)) throw new Error(`obsolete physical-line parser adapter: ${module.path}`);
    const actual = functionSymbols(source).sort();
    const classified = module.groups.flatMap(group => {
      if (!roles.has(group.role) || !group.reason?.trim()) throw new Error(`invalid function responsibility: ${module.path}`);
      return group.symbols;
    }).sort();
    if (JSON.stringify(actual) !== JSON.stringify(classified)) {
      const added = actual.filter(symbol => !classified.includes(symbol));
      const removed = classified.filter(symbol => !actual.includes(symbol));
      throw new Error(`unclassified function or stale responsibility: ${module.path}; added ${JSON.stringify(added)}, removed ${JSON.stringify(removed)} (duplicate classifications also fail)`);
    }
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(audit(JSON.parse(fs.readFileSync(path.join(root, 'provenance/plantuml-mit-1.2026.9.json')))));
}
