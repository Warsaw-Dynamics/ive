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
    if (!safe(file.path) || /\/(?:gen|h|smetana|elk)\//.test(file.path) || !/^net\/sourceforge\/plantuml\/(?:cheneer\/(?:ChenEerDiagram|command\/(?:CommandCreateEntity|CommandCreateAttribute|CommandAssociate|CommandEndGroup))\.java|svek\/image\/EntityImageChen(?:Entity|Relationship|Attribute)\.java|decoration\/(?:LinkType|LinkStyle|symbol\/(?:USymbols|USymbolRectangle))\.java|klimt\/color\/ColorTrieNode\.java|filesdiagram\/(?:FEntry|FilesDiagram|FilesListing)\.java|board\/(?:Activity|BNode|BoardDiagram|CommandBoardPlus|CardBox|PostIt)\.java|hcl\/(?:HclParser|HclSource)\.java|packetdiag\/(?:PacketDiagram|PacketBlock|PacketIndicator|command\/(?:CommandNumRange|CommandColWidth|CommandNodeHeight|CommandScaleInterval|CommandScaleDirection|CommandSameHeight|CommandPacketDiagStart|CommandPacketDiagEnd))\.java|nwdiag\/(?:CommandNwDiagInit|CommandNetwork|CommandElement|CommandProperty|CommandGroup|CommandEndSomething|NwDiagram|core\/(?:NServer|Network|NwGroup))\.java|BlockUmlBuilder\.java|PSystemBuilder\.java|salt\/(?:DataSourceImpl|Positionner2|Cell|PSystemSalt|factory\/ElementFactory(?:Pyramid|Border|Button|TextField|CheckboxOn|CheckboxOff|RadioOn|RadioOff|Tree|Tab|Menu|Droplist|Image|RetrieveFromDictonnary|Line|Scroll)|element\/(?:TableStrategy|ElementPyramid|ElementRadioCheckbox|ElementTextField|ElementButton|AbstractElementText))\.java|gantt\/(?:GanttDiagram|OpenClose|command\/NaturalGanttCommand|lang\/(?:SubjectTask|SubjectGantt|ComplementDuration|ComplementBeforeOrAfterOrAtTaskStartOrEnd|AbstractComplementTaskInstant|DayPattern|TimeResolution|Verbs)|core\/(?:TaskCode|TaskInstant|TaskImpl)|solver\/Solver|draw\/(?:TaskDrawDiamond|TaskDrawRegular))\.java|timingdiagram\/(?:TimingDiagramFactory|TimingDiagram|TimingRuler|AbstractStatePlayer|ChangeState|TimeConstraint|graphic\/(?:PanelsRobust|PanelsConcise|PanelsState)|command\/(?:CommandRobustConcise|CommandAtTime|CommandAtPlayer|CommandChangeState|CommandChangeStateByTime|CommandChangeStateByPlayerCode|CommandConstraint|CommandTimeMessage|TimeTickBuilder))\.java|tim\/(?:EaterAffectationDefine|TContext|VariableManager)\.java|TitledDiagram\.java|style\/(?:FromSkinparamToStyle|parser\/RawStyleParser|value\/(?:Specificity|ValueImpl))\.java|klimt\/creole\/Display\.java|utils\/(?:CharHidder|Direction)\.java|text\/TLineType\.java|(?:core|command|sequencediagram\/command|classdiagram|objectdiagram\/command|descdiagram|cucadiagram|statediagram|activitydiagram3|mindmap|wbs|json|jsondiagram|yaml(?:\/parser)?|klimt\/(?:font|creole\/(?:legacy|command)))\/[A-Za-z0-9_/]+\.java)$/.test(file.path)) throw new Error(`unauthorized source path: ${file.path}`);
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
  const actual = fs.readdirSync(path.join(root, 'engine/src/plantuml')).map(name => `engine/src/plantuml/${name}`).sort();
  if (JSON.stringify(actual) !== JSON.stringify([...manifest.nativeModules].sort())) throw new Error('native module inventory changed');
  if (!fs.readFileSync(path.join(root, 'provenance/LICENSE-PlantUML-MIT.txt'), 'utf8').includes('Hisashi Miyashita')) throw new Error('contributor notice missing');
  for (const notice of ['provenance/LICENSE-PlantUML-MIT.txt', 'THIRD-PARTY-NOTICES.md']) {
    if (!fs.readFileSync(path.join(root, notice), 'utf8').includes('Serge Wenger')) throw new Error(`contributor notice missing: ${notice}: Serge Wenger`);
  }
  return `Verified ${manifest.sourceFiles.length} MIT-edition source/ZIP hashes, ${manifest.fixtures.length} fresh fixture hashes and ${actual.length} native modules.`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(audit(JSON.parse(fs.readFileSync(path.join(root, 'provenance/plantuml-mit-1.2026.9.json')))));
}
