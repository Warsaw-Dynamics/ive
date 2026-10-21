// IVE mock: monorepo-scale fake data for checkouts.js — 30 branches without a worktree and 600 scratches over two
// years. An ES module so it runs before DOMContentLoaded, which is when include.js starts Alpine. Seeded, so every
// load is the same.
import { faker } from './vendor/faker/locale/en.js';

faker.seed(817);

var TODAY = new Date('2026-10-16T12:00:00Z');
var DAY = 24 * 3600 * 1000;
var pick = faker.helpers.arrayElement;
function between(min, max) { return faker.number.int({ min: min, max: max }); }

var WORDS = ['orgs', 'users', 'tokens', 'quota', 'invoices', 'waf', 'deploy', 'backup', 'restore', 'region', 'brain', 'registry',
  'webhook', 'retry', 'migration', 'slow-query', 'repro', 'notes', 'plan', 'spike', 'payload', 'curl', 'oidc', 'csrf', 'jira', 'export'];
function branchName() { return pick(['GEN', 'OHQ', 'AP', 'ESFJ']) + '-' + between(100, 990) + '-' + faker.hacker.noun().replace(/\s+/g, '-'); }
// Branches in the repo without a worktree, newest first; every third only on origin (remote: the remote's name; null
// for a local branch).
var LIVE = Array.from(new Set(faker.helpers.multiple(branchName, { count: 30 })))
  .map(function (name, i) { return { name: name, remote: i % 3 === 1 ? 'origin' : null, modified: new Date(TODAY - Math.floor(i * i * 1.7) * DAY) }; });

var TEXT = {
  sql: function () { return 'SELECT ' + faker.database.column() + ', ' + faker.database.column() + '\nFROM ' + pick(WORDS).replace('-', '_') + '\nWHERE ' + faker.database.column() + ' = \'' + faker.string.alphanumeric(6) + '\';'; },
  http: function () { return faker.internet.httpMethod() + ' https://idp.localhost/api/' + faker.word.noun() + '\nAccept: application/json\nAuthorization: Bearer {{token}}'; },
  md: function () { return '# ' + faker.hacker.phrase() + '\n\n- ' + faker.lorem.sentence() + '\n- ' + faker.lorem.sentence(); },
  txt: function () { return faker.lorem.paragraph(); },
  json: function () { return JSON.stringify({ id: faker.string.uuid(), email: faker.internet.email(), plan: faker.commerce.productName() }, null, 2); },
  sh: function () { return '#!/bin/sh\ncurl -s ' + faker.internet.url() + ' | jq .'; }
};
var EXT = ['sql', 'sql', 'sql', 'http', 'http', 'md', 'md', 'txt', 'json', 'sh'];

var scratches = [];
var names = new Set();
for (var s = 0; s < 595; s++) {
  var ext = pick(EXT);
  var base = pick(WORDS) + '-' + faker.word.noun().toLowerCase().replace(/[^a-z]+/g, '-');
  var name = base + '.' + ext;
  for (var k = 2; names.has(name); k++) name = base + '-' + k + '.' + ext;
  names.add(name);
  // skewed towards recent: most scratches are young, a long tail goes back two years
  var age = Math.floor(Math.pow(faker.number.float(), 2) * 700);
  scratches.push({ name: name, text: TEXT[ext](), project: 'atlassian', modified: new Date(TODAY - age * DAY - between(1, 10) * 3600 * 1000) });
}

// The platform project: exactly RUN_CONFIGS run configurations over ~140 folders with a .run.toml (services/<domain>/<name>,
// apps/<name>, tools/<name>, infra/<env>), each folder's stack picking a few of its usual configurations.
var RUN_CONFIGS = 1000;
var BUILD_OK = ['Compiling…', 'Build finished.'];
function cfg(key, name, command, more) { return Object.assign({ type: 'run', key: key, name: name, command: command, output: ['…'] }, more || {}); }
var STACKS = {
  java: { file: ['pom.xml', '<project/>'], configs: function (n) { return [
    cfg('build', n + ' - Build', 'mvn -q package -DskipTests', { output: BUILD_OK, exits: 0 }),
    cfg('test', n + ' - Test', 'mvn test', { output: ['Tests run: ' + between(20, 400) + ', Failures: 0'], exits: 0 }),
    cfg('app', n, 'mvn spring-boot:run -Dspring-boot.run.profiles=dev', { before: ['build'], output: ['Tomcat started on port ' + between(8080, 8999)] }),
    cfg('migrate', n + ' - DB migrate', 'mvn liquibase:update', { output: ['Update has been successful.'], exits: 0 }),
    cfg('it', n + ' - Integration tests', 'mvn verify -Pit', { exits: 0 }),
    cfg('image', n + ' - Docker image', 'mvn jib:dockerBuild', { exits: 0 }),
    cfg('deps', n + ' - Dependency tree', 'mvn dependency:tree', { exits: 0 })]; } },
  node: { file: ['package.json', '{}'], configs: function (n) { return [
    cfg('dev', n + ' - Dev server', 'npm run dev', { output: ['ready on http://localhost:' + between(3000, 3999)] }),
    cfg('build', n + ' - Build', 'npm run build', { output: BUILD_OK, exits: 0 }),
    cfg('test', n + ' - Test', 'npm test', { exits: 0 }),
    cfg('lint', n + ' - Lint', 'npm run lint', { exits: 0 }),
    cfg('e2e', n + ' - E2E', 'npx playwright test', { exits: 0 }),
    { type: 'serve', key: 'storybook', name: n + ' - Storybook', live_reload: true, root: 'storybook-static', port: String(between(6000, 6999)), output: ['…'] }]; } },
  go: { file: ['go.mod', 'module example.com/svc'], configs: function (n) { return [
    cfg('build', n + ' - Build', 'go build ./...', { output: BUILD_OK, exits: 0 }),
    cfg('test', n + ' - Test', 'go test ./...', { exits: 0 }),
    cfg('run', n, 'go run ./cmd/server', { output: ['listening on :' + between(9000, 9999)] }),
    cfg('lint', n + ' - Lint', 'golangci-lint run', { exits: 0 }),
    cfg('gen', n + ' - Generate', 'go generate ./...', { exits: 0 })]; } },
  rust: { file: ['Cargo.toml', '[package]'], configs: function (n) { return [
    cfg('build', n + ' - Build', 'cargo build --release', { output: BUILD_OK, exits: 0 }),
    cfg('test', n + ' - Test', 'cargo test', { exits: 0 }),
    cfg('run', n, 'cargo run', { output: ['Running `target/debug/' + n.toLowerCase().replace(/\W+/g, '-') + '`'] }),
    cfg('clippy', n + ' - Clippy', 'cargo clippy --all-targets', { exits: 0 }),
    cfg('bench', n + ' - Bench', 'cargo bench', { exits: 0 })]; } },
  infra: { file: ['compose.yaml', 'services: {}'], configs: function (n) { return [
    cfg('up', n + ' - Compose up', 'docker compose up', { output: ['Container started'] }),
    cfg('down', n + ' - Compose down', 'docker compose down', { exits: 0 }),
    cfg('plan', n + ' - Terraform plan', 'terraform plan -var-file=${env}.tfvars', { exits: 0 }),
    cfg('apply', n + ' - Terraform apply', 'terraform apply -var-file=${env}.tfvars', { env: { TF_TOKEN: '${tf_token}' }, exits: 0 }),
    cfg('smoke', n + ' - Smoke test', './smoke.sh ${env}', { exits: 0 })]; } }
};
var DOMAINS = ['billing', 'identity', 'search', 'catalog', 'orders', 'payments', 'notifications', 'reporting', 'analytics', 'gateway', 'inventory', 'shipping'];
function title(slug) { return slug.split('-').map(function (w) { return w[0].toUpperCase() + w.slice(1); }).join(' '); }
function slug() { return faker.word.noun().toLowerCase().replace(/[^a-z]+/g, '-') + pick(['', '-api', '-worker', '-sync', '-service']); }
// A folder's configurations: the stack's first two always, then a random few more, in the stack's order.
function folderConfigs(stack, name) {
  var all = STACKS[stack].configs(name);
  return all.slice(0, 2).concat(all.slice(2).filter(function () { return faker.datatype.boolean(); }));
}
var runFolders = {};
var stacksOf = {};
var count = 0;
function addFolder(path, stack, name) {
  if (runFolders[path] || count >= RUN_CONFIGS) return;
  var configs = folderConfigs(stack, name).slice(0, RUN_CONFIGS - count);
  // apply's ${tf_token} and the env default make infra folders exercise variables
  var vars = stack === 'infra' ? { env: { description: 'Environment', default: path.split('/').pop() }, tf_token: { description: 'Terraform Cloud token', secret: true } } : {};
  runFolders[path] = { vars: vars, configs: configs };
  stacksOf[path] = stack;
  count += configs.length;
}
['dev', 'qa', 'staging', 'prod'].forEach(function (env) { addFolder('/infra/' + env, 'infra', title(env)); });
while (count < RUN_CONFIGS) {
  var roll = faker.number.float();
  if (roll < 0.6) { var s = slug(); addFolder('/services/' + pick(DOMAINS) + '/' + s, pick(['java', 'java', 'go']), title(s)); }
  else if (roll < 0.85) { var a = slug(); addFolder('/apps/' + a, 'node', title(a)); }
  else { var t = slug(); addFolder('/tools/' + t, pick(['rust', 'go']), title(t)); }
}

window.IVE_SCALE = { today: TODAY, scratches: scratches, branches: LIVE, runFiles: runFolders,
  stackFile: function (folder) { return STACKS[stacksOf[folder]].file; } };
