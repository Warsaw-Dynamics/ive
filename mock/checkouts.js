// IVE mock: projects and worktrees in one sidebar, as the app draws them. Processes (terminals, Claude Code) live
// under their worktree; scratches are project-level. Clicking a project or worktree opens its overview as a tab;
// git is read-only. Fake data.
(function () {
  'use strict';

  var ICONS = '../assets/icons/';
  // sidebar/view.rs: TREE_PADDING, TREE_INDENT (one icon slot plus its gap)
  var TREE_PADDING = 8;
  var TREE_INDENT = 20;
  // colors/checkout.rs: HUES_DARK / HUES_LIGHT, eight each (--co-0 … --co-7 in checkouts.css)
  var CHECKOUT_COLOURS = 8;
  var USER = 'kb@ive';
  var CONFIG = '~/.config/ive/default/';
  // sidebar/mod.rs: DEFAULT_WIDTH, MIN_WIDTH, MIN_EDITOR_WIDTH
  var SIDEBAR_DEFAULT_WIDTH = 288;
  var SIDEBAR_MIN_WIDTH = 160;
  var MIN_EDITOR_WIDTH = 320;
  // paging.rs: lists show 10, then 10 more
  var PAGE = 10;
  // worktree_git.rs: indents of the git list
  var DAY = 24 * 3600 * 1000;
  // script/temporary_tabs.rs TabKind (ive.rn's default kinds): a click opens these temporary
  var TEMPORARY_KINDS = ['projectPage', 'settings', 'hub', 'runConfigs'];
  // Commits lists this many at first, and this many more on each Load more.
  var COMMITS_MORE = 20;

  function file(name, text) { return { name: name, text: text }; }
  function dir(name, children) { return { name: name, children: children }; }

  // Run configurations (spec 2026-10-18-ive-run-configurations-design.md): one .run.toml per folder, never at a
  // checkout's root. Keyed by folder; each checkout has the same files here. A config's output and exit code are
  // mock-only: what its run prints, and whether it ends (exits) or keeps running.
  function runCfg(key, name, command, more) { return Object.assign({ type: 'run', key: key, name: name, command: command }, more || {}); }
  function serveCfg(key, name, more) { return Object.assign({ type: 'serve', key: key, name: name, live_reload: true }, more); }
  var MVN_BUILD = ['[INFO] Scanning for projects...', '[INFO] Building misc 1.0-SNAPSHOT', '[INFO] BUILD SUCCESS', '[INFO] Total time:  14.212 s'];
  var RUN_FILES = {
    '/run': {
      vars: { jira_token: { description: 'Jira API token for the dependency report', secret: true } },
      configs: [
        runCfg('build-all', 'Build all (skip tests)', 'mvn -q install -DskipTests', { cwd: '..', output: MVN_BUILD, exits: 0 }),
        runCfg('dep-report', 'Dep Check jira report', 'node extra/dependency-analyser.js', { cwd: '${REPO}', env: { JIRA_API_TOKEN: '${jira_token}', JIRA_USER: 'automation@example.com' },
          output: ['Reading 212 pom.xml files…', 'Posting the report to GEN-12…', 'Done: 4 outdated, 0 vulnerable.'], exits: 0 })
      ]
    },
    '/internal-apps/misc': {
      vars: {
        zoho_secret: { description: 'Zoho OAuth client secret', secret: true },
        ngrok_subdomain: { description: 'ngrok subdomain for misc', default: 'misc-${USER}' }
      },
      configs: [
        runCfg('build', 'Build misc', 'mvn -q package -DskipTests', { output: MVN_BUILD, exits: 0 }),
        runCfg('migrate', 'Misc DB migrate', 'mvn liquibase:update', { output: ['[INFO] Liquibase: Update has been successful. Rows affected: 0'], exits: 0 }),
        runCfg('app', 'MiscApplication', 'mvn spring-boot:run -Dspring-boot.run.profiles=dev', { before: ['build', 'migrate'], env: { ZOHO_SECRET: '${zoho_secret}' },
          output: ['  .   ____          _', ' /\\\\ / ___\'_ __ _ _(_)_ __  __ _', '( ( )\\___ | \'_ | \'_| | \'_ \\/ _` |', ' =========|_|==============|___/=/_/_/_/',
            'Started MiscApplication in 4.817 seconds (process running for 5.2)', 'Tomcat started on port 8080 (http) with context path \'/\''] }),
        runCfg('ngrok', 'Misc - Ngrok', 'ngrok http 8080 --subdomain ${ngrok_subdomain}', { output: ['Session Status  online', 'Forwarding      https://kb-misc.eu.ngrok.io -> http://localhost:8080'] })
      ]
    },
    '/internal-apps/ive': {
      vars: { mock_port: { description: 'Port for the HTML mock', default: '8767' } },
      configs: [
        runCfg('ive', 'IVE (profile kb)', './run.sh -- --profile kb .', { output: ['   Compiling ive v0.1.0', '    Finished `dev` profile in 9.31s', '     Running `target/debug/ive --profile kb .`', 'ive: MCP on http://127.0.0.1:41873/mcp'] }),
        serveCfg('mock', 'IVE mock', { root: '.', port: '${mock_port}', fallback: 'index.html', rewrites: [{ from: '^/checkouts$', to: '/checkouts.html' }], script: 'mock/serve.rn',
          output: ['GET /mock/checkouts.html 200 3 ms', 'GET /mock/checkouts.css 200 1 ms', 'GET /assets/icons/dark/run.svg 200 0 ms', 'GET /__ive/live 200 (live reload)', 'mock/checkouts.css changed: stylesheet swapped'] })
      ]
    },
    // A broken file: its overview shows the error and no configurations; other folders are unaffected.
    '/lib': { vars: {}, configs: [], error: '.run.toml:4:10: before names "compile", which this file does not define' }
  };
  // the platform project's 1000 configurations (checkouts-data.js); its folders do not clash with atlassian's
  Object.assign(RUN_FILES, window.IVE_SCALE.runFiles);
  // The tree's virtual rows under a folder with a .run.toml.
  var HIDDEN_NODE = '@hidden';
  // A worktree's folders (DESIGN.md "Worktree folders"), in tree order; each shows while the folders menu has it on
  // (shown: the default).
  var WORKTREE_FOLDERS = [
    { key: 'processes', title: 'Processes', icon: 'terminal.svg', shown: true, digit: '1' },
    { key: 'uncommitted', title: 'Uncommitted', icon: 'changes.svg', shown: false, digit: '2' },
    { key: 'sinceBase', title: 'Since base', icon: 'history.svg', shown: false, digit: '3' },
    { key: 'commits', title: 'Commits', icon: 'commit.svg', shown: false, digit: '4' },
    { key: 'files', title: 'Files', icon: 'project-folder.svg', shown: true, digit: '5' }
  ];
  var LIB_RUN_TOML = '[run.test]\nname = "Test lib"\ncommand = "npm test"\nbefore = ["compile"]\n';

  function tomlString(value) { return JSON.stringify(value); }
  // The .run.toml a RUN_FILES entry stands for.
  function tomlOf(folder) {
    var f = RUN_FILES[folder];
    var out = [];
    Object.keys(f.vars).forEach(function (name) {
      var v = f.vars[name];
      out.push('[vars.' + name + ']', 'description = ' + tomlString(v.description));
      if (v.secret) out.push('secret = true');
      if (v.default) out.push('default = ' + tomlString(v.default));
      out.push('');
    });
    f.configs.forEach(function (c) {
      out.push('[' + c.type + '.' + c.key + ']', 'name = ' + tomlString(c.name));
      ['command', 'cwd', 'root', 'port', 'fallback', 'script'].forEach(function (k) { if (c[k]) out.push(k + ' = ' + tomlString(c[k])); });
      if (c.type === 'serve') out.push('live_reload = ' + c.live_reload);
      if (c.before) out.push('before = [' + c.before.map(tomlString).join(', ') + ']');
      if (c.env) out.push('env = { ' + Object.keys(c.env).map(function (k) { return k + ' = ' + tomlString(c.env[k]); }).join(', ') + ' }');
      if (c.rewrites) out.push('rewrites = [' + c.rewrites.map(function (r) { return '{ from = ' + tomlString(r.from) + ', to = ' + tomlString(r.to) + ' }'; }).join(', ') + ']');
      out.push('');
    });
    return out.join('\n');
  }

  function css(branch) {
    var base = ['/* IVE mock styles */', ':root {', '  --background: #1e1f22;', '  --sidebar: #2b2d30;', '  --accent: #3574f0;', '}',
      '', '.sidebar {', '  width: 288px;', '  border-right: 1px solid var(--border);', '}'];
    if (branch === 'GEN-817-ive') {
      base.splice(4, 0, '  --tree: #191a1c;');
    }
    return base.join('\n');
  }

  function atlassian(branch) {
    return [
      dir('.claude', [file('settings.json', '{\n  "permissions": {}\n}')]),
      file('.editorconfig', 'root = true\n\n[*]\nindent_style = space'),
      file('.gitignore', 'target/\nnode_modules/'),
      dir('internal-apps', [
        dir('ive', [
          dir('mock', [file('index.html', '<!doctype html>\n<title>IVE mock</title>'), file('mock.css', css(branch))]),
          dir('src', [file('theme.rs', 'pub struct Theme {\n    pub background: u32,\n    pub sidebar: u32,\n    pub tree: u32,\n}'),
            file('main.rs', 'fn main() {\n    ive::run();\n}')]),
          file('.run.toml', tomlOf('/internal-apps/ive')),
          file('Cargo.toml', '[package]\nname = "ive"\nversion = "0.1.0"\nedition = "2024"'),
          file('run.sh', '#!/bin/sh\nexec cargo run "$@"')
        ]),
        dir('misc', [file('.run.toml', tomlOf('/internal-apps/misc')), file('pom.xml', '<project>\n  <artifactId>misc</artifactId>\n</project>')]),
        dir('paas', [file('pom.xml', '<project>\n  <artifactId>paas</artifactId>\n</project>')])
      ]),
      dir('lib', [file('.run.toml', LIB_RUN_TOML), file('package.json', '{\n  "name": "lib"\n}')]),
      dir('run', [file('.run.toml', tomlOf('/run'))]),
      file('CLAUDE.md', '# Java Guidelines\n- Use Java 26 features (records, var, etc.)\n- Avoid null where possible.'),
      file('README.md', '# atlassian\n\nWarsaw Dynamics monorepo, branch ' + branch + '.'),
      file('pom.xml', '<project>\n  <artifactId>atlassian</artifactId>\n</project>')
    ];
  }

  // The platform project: its generated .run.toml folders nested as a tree, each with its stack's build file.
  function platform() {
    var root = [file('README.md', '# platform\n\n1000 run configurations, for the run configurations tab at scale.')];
    Object.keys(window.IVE_SCALE.runFiles).forEach(function (folder) {
      var level = root;
      folder.split('/').filter(Boolean).forEach(function (name) {
        var next = level.find(function (e) { return e.name === name && e.children; });
        if (next === undefined) { next = dir(name, []); level.push(next); }
        level = next.children;
      });
      var stack = window.IVE_SCALE.stackFile(folder);
      level.push(file('.run.toml', tomlOf(folder)), file(stack[0], stack[1]));
    });
    return root;
  }

  function forge() {
    return [dir('lib', [file('installer.ex', 'defmodule Installer do\nend')]), file('mix.exs', 'defmodule Installer.MixProject do\nend')];
  }

  function change(mark, path, diff) { return { mark: mark, path: path, diff: diff }; }
  function commitOf(sha, when, message, files) { return { sha: sha, when: when, message: message, files: files }; }

  var THEME_DIFF = ['@@ -33,6 +33,8 @@ pub struct Theme {', '     pub appearance: Appearance,', '     pub background: u32,', '     pub sidebar: u32,',
    '+    /// the projects panel, as IntelliJ\'s project tool window', '+    pub tree: u32,', '     pub border: u32,', '     pub text: u32,'];
  var VIEW_DIFF = ['@@ -27,7 +27,10 @@ use crate::workspace::{Workspace, icon};', ' pub const HEADER_HEIGHT: f32 = 36.;',
    '+/// IntelliJ\'s project view: Inter 13 px on 24 px rows.', ' const ROW_HEIGHT: f32 = 24.;', '+/// 13 px at the default 16 px rem.',
    '+const TREE_TEXT_REMS: f32 = 0.8125;', ' const ICON_SIZE: f32 = 16.;', '@@ -96,7 +99,7 @@ pub fn render(sidebar: Sidebar<\'_>, theme: Theme) -> Div {',
    '         .w(px(sidebar.width))', '         .h_full()', '-        .bg(rgb(theme.sidebar))', '+        .bg(rgb(theme.tree))', '         .border_r_1()'];
  var EDITOR_DIFF = ['@@ -981,8 +981,6 @@ pub fn render_detached(workspace: &Workspace, id: TabId) {', '         .rounded(px(6.))', '         .border_1()',
    '-        .bg(rgb(theme.tab_active))', '-        .border_color(rgb(theme.tab_active_border))', '         .child(tab.icon(workspace, theme))'];

  // A worktree's commits since its base, newest first.
  var IVE_COMMITS = [
    commitOf('f39a013', '35 min', '[GEN-817] IVE - Integrated Vibe Editor - dragged tab previews its window', [
      change('A', 'internal-apps/ive/src/workspace/drag_preview.rs', ['@@ -0,0 +1,4 @@', '+//! A small picture of a dragged tab.', '+pub struct DragPreview {', '+    pub title: SharedString,', '+}']),
      change('M', 'internal-apps/ive/src/editor.rs', ['@@ -41,6 +41,7 @@ use crate::tabs::Tabs;', ' use crate::theme::Theme;', '+use crate::workspace::drag_preview::DragPreview;', ' use crate::tooltip;'])
    ]),
    commitOf('d5f20d5', '2 h', '[GEN-817] IVE - Integrated Vibe Editor - ive MCP server for the repo', [
      change('A', '.mcp.json', ['@@ -0,0 +1,5 @@', '+{', '+  "mcpServers": {', '+    "ive": { "type": "http", "url": "${IVE_MCP_URL}" }', '+  }', '+}'])
    ]),
    commitOf('968134f', '3 h', '[GEN-817] IVE - Integrated Vibe Editor - terminals know their IVE\'s MCP', [
      change('M', 'internal-apps/ive/src/terminal/session.rs', ['@@ -88,4 +88,6 @@ fn environment(checkout: &Path) -> Vec<(String, String)> {', '     vec![', '+        ("IVE_MCP_URL".into(), mcp.url()),', '+        ("IVE_MCP_KEY".into(), mcp.key()),', '     ]']),
      change('M', 'internal-apps/ive/README.md', ['@@ -12,3 +12,4 @@', ' | `--background` | open the window without taking focus |', '+| `--restart SCRIPT` | show a Restart button left of Settings |'])
    ]),
    commitOf('12c0f68', '3 h', '[GEN-817] IVE - Integrated Vibe Editor - no resize in fullscreen, drag leaves it', [
      change('M', 'internal-apps/ive/src/frame.rs', ['@@ -13,4 +13,5 @@', ' /// compositor): shadow, border, rounded top corners and resize handles.', '-pub fn frame(window: &Window) -> Div {', '+pub fn frame(window: &Window, tiling: Tiling) -> Div {'])
    ]),
    commitOf('91e0c39', '5 d', '[GEN-817] IVE - initial Rust app, icons and HTML mock', [
      change('A', 'internal-apps/ive/Cargo.toml', ['@@ -0,0 +1,4 @@', '+[package]', '+name = "ive"', '+version = "0.1.0"', '+edition = "2024"']),
      change('A', 'internal-apps/ive/src/main.rs', ['@@ -0,0 +1,3 @@', '+fn main() {', '+    Application::new().run(|cx| Workspace::open(cx));', '+}'])
    ])
  ];

  function logCommit(sha, subject, author, hours, names) { return { sha: sha, subject: subject, author: author, hours: hours, names: names || [] }; }
  var AUTHORS = ['Piotr Haraburda', 'Michał Błaszczykowski', 'Jakub Myśliński', 'Aleksandra Szymańska', 'Krzysztof Bogdan'];

  // name, path (as tilde_home shows it), git; checkouts: the first is the main one. label: the branch for the main
  // checkout, else its folder's name (project.rs). worktrees: where new worktrees go, relative to the main checkout
  // (`worktrees` in its .ive.toml, else `.worktree`); atlassian's .ive.toml keeps them in .claude/worktrees.
  var PROJECTS = [
    { name: 'atlassian', path: '~/Projects/atlassian', git: true, treeOf: atlassian, worktrees: '.claude/worktrees',
      checkouts: [
        { branch: 'dev', base: 'origin/dev', path: '~/Projects/atlassian', tree: atlassian('dev'), ahead: 0, behind: 0, changes: [], commits: [] },
        { branch: 'GEN-817-ive', base: 'dev', commits: IVE_COMMITS, path: '~/Projects/atlassian/.claude/worktrees/GEN-817-ive', tree: atlassian('GEN-817-ive'), ahead: 189, behind: 28,
          changes: [
            change('M', 'internal-apps/ive/src/editor.rs', EDITOR_DIFF),
            change('M', 'internal-apps/ive/src/sidebar/view.rs', VIEW_DIFF),
            change('M', 'internal-apps/ive/src/tab_window.rs', ['@@ -28,6 +28,8 @@ impl TabWindow {', '         let subscriptions = vec![cx.observe_window_activation(window, |tab_window, window, cx| {', '+            // the tab chip shows whether this window is active', '+            cx.notify();', '             if window.is_window_active() {']),
            change('M', 'internal-apps/ive/src/theme.rs', THEME_DIFF),
            change('A', 'internal-apps/ive/src/sidebar/tree_view.rs', ['@@ -0,0 +1,6 @@', '+/// What the tree shows under a worktree.', '+pub enum TreeView {', '+    Files,', '+    Changes,', '+    Processes,', '+}']),
            change('D', 'internal-apps/ive/mock/old-sidebar.html', ['@@ -1,2 +0,0 @@', '-<!-- the first sidebar, before worktrees -->', '-<aside class="sidebar"></aside>'])
          ] },
        { branch: 'IDP-30-Reserved-subdomain-returns-404', base: 'dev', path: '~/Projects/atlassian-wt/IDP-30-reserved-subdomain', tree: atlassian('IDP-30'), ahead: 0, behind: 104, commits: [],
          changes: [change('M', 'idp/src/main/java/com/warsawdynamics/idp/org/SubdomainController.java', ['@@ -41,3 +41,4 @@', '     if (reserved.contains(slug)) {', '-        return ResponseEntity.notFound().build();', '+        return ResponseEntity.status(HttpStatus.CONFLICT).body("reserved: " + slug);', '     }'])] },
        { branch: 'GEN-811-paas-infrastructure-reference', base: 'dev', path: '~/Projects/atlassian-wt/GEN-811-paas-reference', tree: atlassian('GEN-811'), ahead: 48, behind: 1204, changes: [],
          commits: [commitOf('b7d01e2', '3 d', '[GEN-811] paas: infrastructure reference', [change('A', 'internal-apps/paas/docs/reference.md', ['@@ -0,0 +1,2 @@', '+# Infrastructure reference', '+'])])] }
      ],
      log: [
        logCommit('d0f0bd8', 'Merge remote-tracking branch \'origin/dev\' into dev', AUTHORS[4], 2, ['dev', 'origin/dev']),
        logCommit('4eea9e0', 'Merged in HF-13_reopening_the_editor_reuses_the_spaces_types (pull request #5900)', AUTHORS[0], 50),
        logCommit('609381f', 'Merged in AP-2486_JQL_search-send_list_filters_in_a_POST (pull request #5897)', AUTHORS[1], 51),
        logCommit('56a1e6b', '[HF-13] Reopening the editor reuses the spaces, types and link types it read the first time', AUTHORS[0], 52, ['origin/HF-13_reopening_the_editor']),
        logCommit('3e32595', '[HF-8] Answering in the queue should not raise a success flag', AUTHORS[0], 53, ['origin/HF-8_answering_in_the_queue']),
        logCommit('43b0177', 'Merge remote-tracking branch \'origin/dev\' into HF-8_answering_in_the_queue', AUTHORS[0], 54),
        logCommit('7a54c8c', '[HF-8] Answering in the queue should not raise a success flag', AUTHORS[0], 55),
        logCommit('685df41', 'Merged in ESFC-2281_Webhook_job_queue_fails (pull request #5890)', AUTHORS[2], 56),
        logCommit('77a0697', 'Merged in ESFJ-2577-E2E-Test-fix-dashboard-timezone (pull request #5888)', AUTHORS[3], 57),
        logCommit('65e9262', '[ESFJ-2577] E2E test: fix dashboard timezone', AUTHORS[3], 58, ['origin/ESFJ-2577-E2E-Test-fix-dashboard-timezone'])
      ],
      stashes: [
        { message: 'On dev: Meta AM stats - unique index', days: 650 },
        { message: 'On ESFJ-1168-priv-escalation: 1168', days: 750 },
        { message: 'WIP on GEN-313-security-atlassian: 2718e641c [GEN-313] Fix Atlassian reporter', days: 1120 }
      ],
      branches: window.IVE_SCALE.branches },
    { name: 'forge-installer', path: '~/Projects/forge-installer', git: true, treeOf: forge, worktrees: '.worktree',
      checkouts: [{ branch: 'main', base: 'origin/main', path: '~/Projects/forge-installer', tree: forge(), ahead: 1, behind: 0,
        commits: [commitOf('a1b2c3d', 'yesterday', 'LiveView shell', [change('M', 'mix.exs', ['@@ -8,3 +8,4 @@', '   defp deps do', '+    {:phoenix_live_view, "~> 1.0"},', '   end'])])],
        changes: [change('M', 'lib/installer.ex', ['@@ -1,2 +1,3 @@', ' defmodule Installer do', '+  use Phoenix.LiveView', ' end'])] }],
      log: [logCommit('a1b2c3d', 'LiveView shell', AUTHORS[4], 30, ['main'])], stashes: [],
      branches: [{ name: 'live-view', remote: null, modified: new Date(window.IVE_SCALE.today - 9 * DAY) },
        { name: 'feature/oidc-login', remote: 'origin', modified: new Date(window.IVE_SCALE.today - 12 * DAY) }] },
    { name: 'platform', path: '~/Projects/platform', git: true, treeOf: platform,
      checkouts: [{ branch: 'main', base: 'origin/main', path: '~/Projects/platform', tree: platform(), ahead: 0, behind: 0, commits: [], changes: [] }],
      log: [], stashes: [], branches: [] },
    { name: 'notes', path: '~/Documents/notes', git: false,
      checkouts: [{ branch: null, path: '~/Documents/notes', changes: [], commits: [],
        tree: [dir('2026', [file('october.md', '# October\n\n- IVE: projects and worktrees UI')]), file('todo.md', '- [ ] pick a checkout layout')] }],
      log: [], stashes: [], branches: [] }
  ];

  // Scratches belong to a project, not a worktree: they live outside the repository and stay when worktrees go.
  function ago(days, hours) { return new Date(window.IVE_SCALE.today - days * DAY - (hours || 0) * 3600 * 1000); }
  var SCRATCHES = [
    { name: 'regex-tries.txt', text: '^(?<key>[A-Z]+-\\d+)\\s', project: 'atlassian', modified: ago(3) },
    { name: 'idp-users.sql', text: 'SELECT email, org\nFROM users\nWHERE org = \'wd\';', project: 'atlassian', modified: ago(0, 2) },
    { name: 'worktree-ui.md', text: '# Worktrees in IVE\n\n- processes follow the worktree they run in', project: 'atlassian', modified: ago(0, 5) },
    { name: 'deploy-idp.sh', text: '#!/bin/sh\nohq deploy idp --env qa', project: 'atlassian', modified: ago(1) },
    { name: 'try-waf.http', text: 'GET https://idp.localhost/%73ecret\nAccept: application/json', project: 'atlassian', modified: ago(12) },
    { name: 'backup-spike.md', text: '# Backup snapshot spike\n\n- freeze + reflink clone works\n- live tar corrupts SQLite', project: 'atlassian', modified: ago(200) }
  ].concat(window.IVE_SCALE.scratches);

  var AGENT_LINES = [
    '✻ Welcome to Claude Code', '',
    '> sync the html mock with the app',
    '● Read internal-apps/ive/src/sidebar/view.rs',
    '● Edited internal-apps/ive/mock/checkouts.css (+40 −12)',
    '● The tree is 13 px on 24 px rows, on the new tree colour.'
  ];

  var THIRD_PARTY = [
    { name: 'gpui', license: 'Apache-2.0' }, { name: 'alacritty_terminal', license: 'Apache-2.0' }, { name: 'tree-sitter', license: 'MIT' },
    { name: 'git2', license: 'MIT OR Apache-2.0' }, { name: 'rune', license: 'MIT OR Apache-2.0' }, { name: 'Inter', license: 'OFL-1.1' },
    { name: 'JetBrains Mono', license: 'OFL-1.1' }
  ];
  // Settings › Scripts: settings_page/scripts.rs sections, [heading ('' for none), text], paths of the default profile.
  var SCRIPT_SECTIONS = [
    ['', 'All your changes go in /home/me/.config/ive/default. IVE writes two reference files there at every start. They are read-only, and editing them changes nothing:\n- ive.rn: every default key binding, mouse binding and menu\n- theme.toml: every default colour and size, for both the dark and light looks'],
    ['To change keys, mouse clicks, menus or excluded folders:', 'write init.rn in that folder. IVE runs it after ive.rn. An entry with the same scope and key replaces the default one. Look up the entry to override in ive.rn. IVE reloads it on save.'],
    ['To make a theme:', 'create themes/<name>.toml in that folder, with the same sections as theme.toml ([sizes], [dark], [light]…) but only the values you change. Everything else stays at the default. Pick it under Settings › Appearance › Theme. IVE reloads it on save.'],
    ['To adjust any theme from a script:', 'set config.theme.<path> in init.rn. This applies on top of the picked theme, for example config.theme.dark.tree = "#202124"; for [dark] tree.'],
    ['Checking it worked:', 'load errors, with file, line or key, and the bad value, go to /home/me/.local/state/ive/default/scripts.log. IVE opens the log when a load fails. Agents connected over MCP can read it with script_status. A failed load keeps the previous settings.']
  ];
  // Settings › Appearance › Theme: the files in themes/ (theme_config/user.rs names, sorted).
  var USER_THEMES = ['nord', 'violet'];

  function withPaths(entry, parent) {
    entry.path = parent + '/' + entry.name;
    (entry.children || []).forEach(function (child) { withPaths(child, entry.path); });
  }

  // 64-bit FNV-1a of the path picks the colour, as colors/checkout.rs does.
  function fnv1a(text) {
    var hash = 0xcbf29ce484222325n;
    new TextEncoder().encode(text).forEach(function (byte) { hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n); });
    return hash;
  }

  function prepare(projects) {
    projects.forEach(function (project) {
      project.checkouts.forEach(function (checkout, i) {
        checkout.project = project.name;
        checkout.main = i === 0;
        checkout.id = project.name + '@' + (checkout.branch || '');
        checkout.label = checkout.main ? (checkout.branch || project.name) : checkout.path.split('/').pop();
        checkout.colour = checkout.main ? 'var(--accent)' : 'var(--co-' + Number(fnv1a(checkout.path) % BigInt(CHECKOUT_COLOURS)) + ')';
        checkout.tree.forEach(function (entry) { withPaths(entry, ''); });
      });
    });
    return projects;
  }

  function find(entries, path) {
    for (var i = 0; i < entries.length; i++) {
      if (entries[i].path === path) return entries[i];
      var hit = entries[i].children && find(entries[i].children, path);
      if (hit) return hit;
    }
    return null;
  }

  // ---- syntax colours (codeview/src/syntax.rs; the mock tokenises with regular expressions) ----
  // Files over 8 MiB (MAX_HIGHLIGHT_BYTES) are shown without colours.
  var C_KEYWORDS = 'as async await break case class const continue crate default def do else elif enum export extends fn for from function if impl import in let loop match mod move mut new null pub ref return self static struct super switch trait true false type typeof use var where while yield defmodule defp'.split(' ');
  var LANGUAGES = {
    rs: { line: '//', block: true }, js: { line: '//', block: true }, ts: { line: '//', block: true }, java: { line: '//', block: true },
    ex: { line: '#' }, py: { line: '#' }, sh: { line: '#' }, bash: { line: '#' }, toml: { line: '#', keywords: false }, yml: { line: '#', keywords: false },
    yaml: { line: '#', keywords: false }, json: { line: null, keywords: false }
  };
  var TOKEN = /(\/\/.*|#.*)|("(?:[^"\\]|\\.)*"?|'(?:[^'\\]|\\.){0,2}'?)|(\b\d[\d_.]*\b)|(\b[A-Z][A-Za-z0-9_]*\b)|(\b[a-z_][A-Za-z0-9_]*\b)/y;

  function languageOf(path) {
    var m = /\.([A-Za-z0-9]+)$/.exec(path || '');
    return m ? LANGUAGES[m[1].toLowerCase()] || null : null;
  }

  // Tokens of one line: [{ t: text, k: '' | 'kw' | 'str' | 'com' | 'type' | 'num' }]. `state.block`: inside /* … */ from an earlier line.
  function highlightLine(text, lang, state) {
    if (lang === null) return [{ t: text, k: '' }];
    var out = [];
    var plain = '';
    var flush = function () { if (plain !== '') out.push({ t: plain, k: '' }); plain = ''; };
    var at = 0;
    while (at < text.length) {
      if (state.block) {
        var end = text.indexOf('*/', at);
        var stop = end === -1 ? text.length : end + 2;
        flush(); out.push({ t: text.slice(at, stop), k: 'com' });
        if (end !== -1) state.block = false;
        at = stop;
        continue;
      }
      if (lang.block && text.startsWith('/*', at)) { state.block = true; continue; }
      TOKEN.lastIndex = at;
      var m = TOKEN.exec(text);
      if (m === null) { plain += text[at]; at += 1; continue; }
      var kind = m[1] !== undefined ? (lang.line !== null && text.startsWith(lang.line, at) ? 'com' : '') : m[2] !== undefined ? 'str' : m[3] !== undefined ? 'num'
        : m[4] !== undefined ? (lang.keywords === false ? '' : 'type') : (lang.keywords !== false && C_KEYWORDS.includes(m[5]) ? 'kw' : '');
      if (kind === '') plain += m[0]; else { flush(); out.push({ t: m[0], k: kind }); }
      at += m[0].length;
    }
    flush();
    return out.length ? out : [{ t: '', k: '' }];
  }

  // A text's lines as token lists; nothing is coloured when `coloured` is false (a file over 8 MiB).
  function highlightLines(lines, path, coloured) {
    var lang = coloured ? languageOf(path) : null;
    var state = { block: false };
    return lines.map(function (text) { return highlightLine(text, lang, state); });
  }

  // diff_view.rs: hunk headers set the old and new line numbers; each line counts on its side(s).
  // Colours: the old side's lines (context and removed) are tokenised in order on their own, the new side's likewise,
  // so a removed row has the old file's colours and an added or context row the new file's.
  function diffRows(lines, path) {
    var old = 0;
    var now = 0;
    var lang = languageOf(path);
    var before = { block: false };
    var after = { block: false };
    return lines.map(function (text) {
      var hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)/.exec(text);
      if (hunk) { old = Number(hunk[1]); now = Number(hunk[2]); return { kind: 'hunk', text: text }; }
      var sign = text[0];
      var row = { kind: sign === '+' ? 'add' : sign === '-' ? 'del' : 'ctx', sign: sign === '+' || sign === '-' ? sign : ' ', text: text.slice(1) };
      row.old = sign === '+' ? '' : old++;
      row.now = sign === '-' ? '' : now++;
      if (sign === '-') row.tokens = highlightLine(row.text, lang, before);
      else if (sign === '+') row.tokens = highlightLine(row.text, lang, after);
      else { row.tokens = highlightLine(row.text, lang, after); highlightLine(row.text, lang, before); }
      return row;
    });
  }

  // Side by side: one row per pair of cells. Context rows show on both sides; a run of removed lines is paired line by
  // line with the added lines that follow it, the shorter side getting empty filler cells. Hunk headers span both sides.
  function sideRows(rows) {
    var out = [];
    var at = 0;
    var cell = function (r, side) { return { kind: r.kind, num: side === 'old' ? r.old : r.now, tokens: r.tokens, text: r.text }; };
    var filler = { kind: 'fill', num: '', tokens: [], text: '' };
    while (at < rows.length) {
      var r = rows[at];
      if (r.kind === 'hunk') { out.push({ kind: 'hunk', text: r.text }); at += 1; continue; }
      if (r.kind === 'ctx') { out.push({ kind: 'pair', left: cell(r, 'old'), right: cell(r, 'new') }); at += 1; continue; }
      var dels = [];
      var adds = [];
      while (at < rows.length && rows[at].kind === 'del') dels.push(rows[at++]);
      while (at < rows.length && rows[at].kind === 'add') adds.push(rows[at++]);
      for (var i = 0; i < Math.max(dels.length, adds.length); i++) {
        out.push({ kind: 'pair', left: i < dels.length ? cell(dels[i], 'old') : filler, right: i < adds.length ? cell(adds[i], 'new') : filler });
      }
    }
    return out;
  }

  // age.rs: now, N min, N h today; yesterday; 2–6 d; then the date.
  function age(date) {
    var today = window.IVE_SCALE.today;
    var midnight = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    if (date >= midnight) {
      var minutes = Math.floor((today - date) / 60000);
      if (minutes < 1) return 'now';
      return minutes < 60 ? minutes + ' min' : Math.floor(minutes / 60) + ' h';
    }
    var days = Math.floor((midnight - date - 1) / DAY) + 1;
    if (days === 1) return 'yesterday';
    if (days <= 6) return days + ' d';
    return date.toISOString().slice(0, 10);
  }

  function counted(count, one, many) { return count === 1 ? '1 ' + one : count + ' ' + many; }

  // project.rs distance_tags: ↑ and ↓, each only when not zero.
  function distanceTags(co) {
    return [co.ahead ? '↑' + co.ahead : null, co.behind ? '↓' + co.behind : null].filter(Boolean);
  }

  // The prefix most of a worktree's commits share, up to a ' - ' ("[GEN-817] IVE - Integrated Vibe Editor - "): the
  // tree drops it so the commits tell apart; empty when fewer than two share one.
  function sharedSubject(commits) {
    var counts = {};
    commits.forEach(function (c) {
      for (var at = c.message.indexOf(' - '); at !== -1; at = c.message.indexOf(' - ', at + 1)) {
        var prefix = c.message.slice(0, at + 3);
        counts[prefix] = (counts[prefix] || 0) + 1;
      }
    });
    return Object.keys(counts).filter(function (p) { return counts[p] >= 2 && counts[p] * 2 >= commits.length; })
      .reduce(function (best, p) { return p.length > best.length ? p : best; }, '');
  }
  // A commit's tree label: without the shared prefix, else without its Jira key.
  function commitLabel(message, shared) {
    if (shared && message.startsWith(shared) && message.length > shared.length) return message.slice(shared.length);
    return message.replace(/^\[[A-Z]+-\d+\] /, '');
  }

  // folders.rs CHANGE_ROWS, file_list.rs collapsed_within: folders open shallowest first, in list order, each while
  // the list still shows at most this many rows (a click on a folder flips it, as in the app).
  var CHANGE_ROWS = 20;
  function openWithin(nodes, budget) {
    var shown = nodes.length;
    var level = nodes;
    while (level.length) {
      var next = [];
      level.forEach(function (node) {
        if (!node.children) return;
        node.open = shown + node.children.length <= budget;
        if (node.open) { shown += node.children.length; next = next.concat(node.children); }
      });
      level = next;
    }
    return nodes;
  }

  // Folders as the git list's Tree layout groups them: a chain of single folders shows as one row.
  function fileTree(files) {
    var root = { folders: {}, files: [] };
    files.forEach(function (x) {
      var parts = x.path.split('/');
      var node = root;
      parts.slice(0, -1).forEach(function (part) { node = node.folders[part] || (node.folders[part] = { folders: {}, files: [] }); });
      node.files.push(x);
    });
    function count(node) { return node.files.length + Object.keys(node.folders).reduce(function (n, k) { return n + count(node.folders[k]); }, 0); }
    function walk(node, path, depth, out) {
      Object.keys(node.folders).sort().forEach(function (name) {
        var label = name;
        var child = node.folders[name];
        while (child.files.length === 0 && Object.keys(child.folders).length === 1) {
          var only = Object.keys(child.folders)[0];
          label += '/' + only;
          child = child.folders[only];
        }
        var full = path ? path + '/' + label : label;
        out.push({ type: 'folder', path: full, label: label, depth: depth, files: count(child), node: child });
      });
      node.files.forEach(function (x) { out.push({ type: 'file', file: x, depth: depth }); });
      return out;
    }
    return { walk: walk, root: root };
  }

  document.addEventListener('alpine:init', function () {
    var dark = window.matchMedia('(prefers-color-scheme: dark)');

    Alpine.data('checkouts', function () {
      // the hub (hub.js, a proposal) and the hash route (route.js, mock-only) bring their own state and methods
      return Object.assign({
        projects: prepare(PROJECTS),
        scratches: SCRATCHES,
        scriptSections: SCRIPT_SECTIONS,
        userThemes: USER_THEMES,
        thirdParty: THIRD_PARTY,
        open: ['atlassian', 'forge-installer', 'platform', 'notes'],
        recent: ['paas-ownhq', 'forge-ari'],
        consoles: [],
        consoleCount: 0,
        tabs: [],
        focus: null,
        // Mock-only: the diff layout last toggled (inline | side); a diff or commit tab opened later starts with it.
        diffLayout: 'inline',
        // Mock-only: the pause timers of "saving…"; a tab carries `saving`, `conflict` ('disk' | 'big'), `error`, `closing`.
        savingTimers: {},
        // tabs.rs Tabs::temporary: the key of the tab that goes once another is shown; only ever the shown one.
        temporary: null,
        menu: null,
        // The tab context menu's tab and where it opened (menu === 'tab').
        tabMenu: { key: null, x: 0, y: 0 },
        // Node keys whose open state differs from the node's default.
        toggled: [],
        query: { scratch: '', branch: '', run: '' },
        expandedGroups: [],
        // How many rows each long list shows; lists start at PAGE and grow a page at a time.
        shown: {},
        // Overviews: the shown section per overview; changed files flat or grouped (Alt+Shift+G).
        overviewTabs: {},
        // the mock starts grouped, as the menu's Tree; the app's default is flat
        gitLayout: 'tree',
        // The run configurations tab: flat (by name) or tree (by folder).
        runLayout: 'flat',
        collapsedFiles: [],
        WORKTREE_FOLDERS: WORKTREE_FOLDERS,
        // DESIGN.md "Projects in the tree": the open projects the tree and the hub leave out; never all of them.
        hiddenProjects: [],
        // A proposal (DESIGN.md "Folders for every worktree"): the folders every worktree shows; the header's folders
        // menu and Ctrl+Alt+1 to 5 set it. The mock starts with them all on; a new
        // profile in the app with what WORKTREE_FOLDERS says (Processes and Files).
        everyFolder: WORKTREE_FOLDERS.reduce(function (on, f) { on[f.key] = true; return on; }, {}),
        gitLogSection: 'log',
        // Settings: theme preference (settings.rs ThemePreference) and the shown page.
        themePreference: 'system',
        settingsPage: 'Appearance',
        // Settings › Appearance › Theme (settings.rs ThemeChoice), and which Copy just said Copied.
        userTheme: 'Default',
        copied: null,
        // Last focused checkout per project: where project-level things run when no worktree tab is focused.
        lastCheckout: {},
        // Hidden worktrees (a proposal, DESIGN.md "Hidden worktrees"), by checkout id: out of the tree, the hub and New
        // terminal; only the project page lists them. Their tabs stay open. Never the main checkout.
        hidden: ['atlassian@GEN-811-paas-infrastructure-reference'],
        // The project page's selected row per project, by row id (it follows a worktree between Worktrees and Hidden);
        // none while typing in the filter.
        pageSelected: {},
        osDark: dark.matches,
        // ~/.config/ive/<profile>/vars.toml: a string, or { wallet: id }; the OS wallet holds the secrets themselves.
        vars: { ngrok_subdomain: 'kb-misc', jira_token: { wallet: 'jira-token' } },
        wallet: { 'jira-token': 'ATATT…' },
        SIDEBAR_DEFAULT_WIDTH: SIDEBAR_DEFAULT_WIDTH,
        sidebarWidth: SIDEBAR_DEFAULT_WIDTH,

        init: function () {
          var self = this;
          dark.addEventListener('change', function (event) { self.osDark = event.matches; });
          this.$watch('appearance', function (value) { document.documentElement.dataset.theme = value; });
          document.documentElement.dataset.theme = this.appearance;
          this.$watch('focus', function () {
            var tab = self.currentTab();
            if (tab && tab.co) self.lastCheckout[tab.project] = tab.co;
            self.remember(tab);
          });
          this.toggled.push('p:forge-installer', 'p:platform', 'p:notes', 'w:atlassian@dev', 'w:atlassian@IDP-30-Reserved-subdomain-returns-404', 'w:atlassian@GEN-811-paas-infrastructure-reference');
          this.addConsole('atlassian@GEN-817-ive', 'agent', AGENT_LINES.slice());
          this.addConsole('atlassian@GEN-817-ive', 'shell', []);
          // restored tabs (saved_tabs.rs) are never temporary
          this.openProjectPage(this.project('atlassian'));
          this.keepTab(this.currentTab());
          this.openFile('atlassian@GEN-817-ive', '/internal-apps/ive/src/theme.rs');
          this.hubInit();
          this.seedHistory();
          this.$nextTick(function () { self.showScreen(location.hash); self.routeInit(); });
          window.addEventListener('hashchange', function () { self.showScreen(location.hash); });
        },

        // Mock-only: #theme=dark&tab=project|file|scratch|terminal|agent|settings|gitlog|commit|diff|run|runs|hub&section=…&menu=add|branches|tab&gap=px&radius=px
        // opens a screen, for screenshots against the app; a hash with t/at/tree is the whole state (route.js).
        showScreen: function (hash) {
          var p = new URLSearchParams(hash.replace(/^#/, ''));
          if (p.get('theme')) this.themePreference = p.get('theme');
          if (p.get('side')) this.sidebarWidth = Number(p.get('side'));
          if (p.has('gap')) {
            var gap = Number(p.get('gap'));
            this.panelGap = gap;
            document.documentElement.style.setProperty('--panel-gap', gap + 'px');
            document.documentElement.toggleAttribute('data-flat', gap === 0);
          }
          if (p.has('radius')) { this.panelRadius = Number(p.get('radius')); document.documentElement.style.setProperty('--panel-radius', this.panelRadius + 'px'); }
          if (p.get('page')) this.settingsPage = p.get('page');
          if (p.get('layout')) this.gitLayout = p.get('layout');
          if (p.get('hidden')) this.hiddenProjects = p.get('hidden').split(',');
          if (p.get('rlayout')) this.runLayout = p.get('rlayout');
          // route.js: the whole state, as the mock last wrote it
          if (this.isRouteHash(p)) { this.restoreRoute(p); return; }
          var atl = this.project('atlassian');
          var ive = this.checkout('atlassian@GEN-817-ive');
          var screens = {
            project: function (self) { self.openProjectPage(atl); },
            file: function (self) { self.openFile(ive.id, '/internal-apps/ive/src/theme.rs'); },
            scratch: function (self) { self.openScratch(self.scratch('deploy-idp.sh')); },
            terminal: function (self) { self.openConsole(self.consoles.find(function (c) { return c.kind === 'shell'; })); },
            agent: function (self) { self.openConsole(self.consoles.find(function (c) { return c.kind === 'agent'; })); },
            settings: function (self) { self.openSettings(); },
            gitlog: function (self) { self.openGitLog(atl); },
            commit: function (self) { self.openCommit(atl, atl.log[1]); },
            diff: function (self) { self.openDiff(ive, ive.changes[3], 'work'); },
            run: function (self) { self.toggled.push('f:' + ive.id + '/internal-apps', 'f:' + ive.id + '/internal-apps/misc'); },
            // ive's and misc's runs in Processes: a server, a long-running command and a finished build
            runs: function (self) {
              self.toggled.push('f:' + ive.id + '/internal-apps', 'f:' + ive.id + '/internal-apps/ive', 'f:' + ive.id + '/internal-apps/misc');
              ['mock', 'ive'].forEach(function (key) { self.startChain({ co: ive.id, folder: '/internal-apps/ive', key: key, where: 'tab' }); });
              self.startChain({ co: ive.id, folder: '/internal-apps/misc', key: 'build', where: 'tab' });
            },
            // the Processes row's run configurations tab, with the runs above going
            runconfigs: function (self) { screens.runs(self); self.openRunConfigs(ive); },
            // the platform project's tab: 1000 configurations in ~140 folders
            runconfigs1k: function (self) { self.openRunConfigs(self.project('platform').checkouts[0]); },
            // a proposal (dnd.js): a flat copy into dev with three conflicts
            conflicts: function (self) { self.mockConflicts(); },
            newterminal: function (self) { self.openNewTerminalHub(); self.hub.query = p.get('q') || ''; },
            hub: function (self) { self.openHub(); self.hub.query = p.get('q') || ''; self.hub.changedOnly = p.get('changed') === '1'; self.hub.scope = p.get('scope') || 'all'; }
          };
          if (screens[p.get('tab')]) screens[p.get('tab')](this);
          var tab = this.currentTab();
          if (tab && p.get('section') && tab.kind === 'projectPage') this.setOverviewTab('pp:' + tab.project, p.get('section'));
          if (p.get('menu') === 'tab') this.$nextTick(function () { this.openTabMenuOnShown(); }.bind(this));
          else if (p.get('menu')) this.menu = p.get('menu');
          if (p.get('expand')) this.toggled = this.toggled.filter(function (k) { return k !== p.get('expand'); }).concat(p.get('expand'));
        },

        get appearance() {
          if (this.themePreference === 'system') return this.osDark ? 'dark' : 'light';
          return this.themePreference;
        },
        icon: function (name) { return ICONS + this.appearance + '/' + name; },
        indent: function (depth) { return (TREE_PADDING + depth * TREE_INDENT) + 'px'; },
        initials: function (name) {
          return name.split(/[^\p{L}\p{N}]+/u).filter(Boolean).slice(0, 2)
            .map(function (word) { return word[0].toUpperCase(); }).join('');
        },
        // icons.rs file_icon, the extensions this mock's files use
        fileIcon: function (name) {
          var ext = name.split('.').pop().toLowerCase();
          var map = { css: 'css', html: 'html', md: 'markdown', toml: 'toml', xml: 'xml', svg: 'xml', sql: 'sql', js: 'javaScript', json: 'json',
            sh: 'shell', yml: 'yaml', yaml: 'yaml', java: 'java', ts: 'typeScript' };
          return 'file-' + (map[ext] || 'text') + '.svg';
        },
        age: age,
        counted: counted,
        distanceTags: distanceTags,
        diffRows: diffRows,
        sideRows: sideRows,
        highlightLines: highlightLines,

        // ---- model ----
        project: function (name) { return this.projects.find(function (p) { return p.name === name; }); },
        checkout: function (id) {
          for (var i = 0; i < this.projects.length; i++) {
            var hit = this.projects[i].checkouts.find(function (c) { return c.id === id; });
            if (hit) return hit;
          }
          return null;
        },
        label: function (id) { return this.checkout(id).label; },
        openProjects: function () {
          var self = this;
          return this.projects.filter(function (p) { return self.open.includes(p.name); });
        },
        // A project with one shown checkout, or without git, hides the worktree level.
        hasWorktreeLevel: function (project) { return project.git && this.shownCheckouts(project).length > 1; },
        isHidden: function (co) { return this.hidden.includes(co.id); },
        shownCheckouts: function (project) { var self = this; return project.checkouts.filter(function (co) { return self.isHidden(co) === false; }); },
        hiddenCheckouts: function (project) { return project.checkouts.filter(this.isHidden, this); },
        // A worktree's terminals and Claude Code (its Processes folder adds every run).
        consolesOf: function (id) { return this.consoles.filter(function (c) { return c.co === id && c.kind !== 'run'; }); },
        // What is running in a checkout: its processes and its live runs (quit and Close project count both).
        isLive: function (c) { return c.kind !== 'run' || c.state !== 'exited'; },
        runningCount: function (id) { var self = this; return this.consoles.filter(function (c) { return c.co === id && self.isLive(c); }).length; },
        projectConsoles: function (project) {
          var self = this;
          return this.consoles.filter(function (c) { return c.co.split('@')[0] === project.name && self.isLive(c); });
        },
        // Where scratches run: the current tab's checkout, else the project's last focused worktree, else its main one.
        targetCheckout: function (project) {
          var current = this.currentTab();
          if (current && current.co && current.co.split('@')[0] === project.name) return current.co;
          return this.lastCheckout[project.name] || project.checkouts[0].id;
        },
        // paths.rs card_folder: relative to the main checkout's parent when below it.
        cardFolder: function (co) {
          var main = this.project(co.project).checkouts[0].path;
          var parent = main.split('/').slice(0, -1).join('/') + '/';
          return co.path.startsWith(parent) ? co.path.slice(parent.length) : co.path;
        },
        // project.rs card_tags
        cardTags: function (co) {
          var tags = co.branch === null ? ['no git'] : [co.changes.length === 0 ? 'clean' : counted(co.changes.length, 'change', 'changes')];
          var distance = co.branch === null ? [] : distanceTags(co);
          if (distance.length) tags.push(distance.join(' '));
          if (this.runningCount(co.id)) tags.push('● ' + this.runningCount(co.id) + ' running');
          return tags;
        },

        // ---- actions ----
        hideWorktree: function (co) { if (co.main === false && this.isHidden(co) === false) this.hidden.push(co.id); },
        showWorktree: function (co) { this.hidden = this.hidden.filter(function (id) { return id !== co.id; }); },
        openProject: function (name) {
          if (this.open.includes(name) === false) this.open.push(name);
          this.recent = this.recent.filter(function (r) { return r !== name; });
          // a project opened shows, though the picker left it out
          this.hiddenProjects = this.hiddenProjects.filter(function (h) { return h !== name; });
          this.menu = null;
        },
        closeProject: function (project) {
          this.open = this.open.filter(function (o) { return o !== project.name; });
          this.recent.unshift(project.name);
          this.hiddenProjects = this.hiddenProjects.filter(function (h) { return h !== project.name; });
          this.tabs = this.tabs.filter(function (t) { return t.project !== project.name; });
          if (this.tabs.some(function (t) { return t.key === this.temporary; }, this) === false) this.temporary = null;
          this.consoles = this.consoles.filter(function (c) { return c.co.split('@')[0] !== project.name; });
          if (this.currentTab() === null) this.focusLastTab();
        },
        // worktree_add.rs: the branch checked out in worktreePath; a remote branch first gets a local one tracking it.
        // The tree shows it under its open project, collapsed.
        createWorktree: function (project, branch) {
          project.branches = project.branches.filter(function (b) { return b !== branch; });
          var checkout = { branch: branch.name, base: project.checkouts[0].branch, path: this.worktreePath(project, branch), tree: project.treeOf(branch.name), ahead: 0, behind: 0, changes: [], commits: [] };
          project.checkouts.push(checkout);
          prepare([{ name: project.name, checkouts: project.checkouts }]);
          this.toggled = this.toggled.filter(function (key) { return key !== 'p:' + project.name; }).concat('w:' + checkout.id);
          return checkout;
        },
        // <main checkout>/<.ive.toml worktrees>/<branch with / as ->
        worktreePath: function (project, branch) { return project.path + '/' + project.worktrees + '/' + branch.name.replace(/\//g, '-'); },
        // Ctrl+Shift+T on a branch: its worktree, then a terminal there; the tree shows the worktree expanded.
        newTerminalOnBranch: function (project, branch) { this.newTerminalIn(this.createWorktree(project, branch)); },
        // The worktree's row expanded in the tree (a project overview card, New worktree): there is no worktree tab.
        revealWorktree: function (co) {
          var hidden = ['p:' + co.project, 'w:' + co.id];
          this.toggled = this.toggled.filter(function (key) { return hidden.includes(key) === false; });
        },
        toggleMenu: function (name) { this.menu = this.menu === name ? null : name; },
        // sidebar/mod.rs fitted_width: at least 160 px, and the editor keeps 320 px of the window.
        startSidebarResize: function (event) {
          var self = this;
          var startX = event.clientX;
          var startWidth = this.sidebarWidth;
          var move = function (e) {
            var widest = Math.max(window.innerWidth - MIN_EDITOR_WIDTH, SIDEBAR_MIN_WIDTH);
            self.sidebarWidth = Math.min(Math.max(startWidth + e.clientX - startX, SIDEBAR_MIN_WIDTH), widest);
          };
          var up = function () {
            window.removeEventListener('mousemove', move);
            window.removeEventListener('mouseup', up);
            document.body.style.cursor = '';
          };
          document.body.style.cursor = 'col-resize';
          window.addEventListener('mousemove', move);
          window.addEventListener('mouseup', up);
        },

        // ---- tabs ----
        // A tab: { key, kind, title, co (checkout id, or null for project-level things), project, ... }.
        // tabs.rs: an open tab is shown; a new one opens right of the shown one, or in the temporary tab's place.
        // Overviews and Settings open temporary.
        openTab: function (tab) {
          var open = this.tabs.find(function (t) { return t.key === tab.key; });
          if (open) { this.showTab(open); return; }
          if ((tab.kind === 'diff' || tab.kind === 'commit') && tab.layout === undefined) tab.layout = this.diffLayout;
          this.tabs.splice(this.insertionIndex(), 0, tab);
          this.focus = tab.key;
          if (TEMPORARY_KINDS.includes(tab.kind)) this.temporary = tab.key;
        },
        // Shows `tab`; a temporary tab shown before goes.
        showTab: function (tab) {
          if (this.focus === tab.key) return;
          this.dropTemporary();
          this.focus = tab.key;
        },
        insertionIndex: function () {
          var at = this.dropTemporary();
          if (at !== -1) return at;
          var self = this;
          var shown = this.tabs.findIndex(function (t) { return t.key === self.focus; });
          return shown === -1 ? this.tabs.length : shown + 1;
        },
        // Removes the temporary tab; the index it had, or -1.
        dropTemporary: function () {
          var self = this;
          var at = this.tabs.findIndex(function (t) { return t.key === self.temporary; });
          this.temporary = null;
          if (at !== -1) this.tabs.splice(at, 1);
          return at;
        },
        isTemporary: function (tab) { return tab.key === this.temporary; },
        // Double click on the tab, middle click on its tree row, or Keep Tab: it stays when another tab is shown.
        keepTab: function (tab) { if (tab && this.isTemporary(tab)) this.temporary = null; },
        openFile: function (id, path) {
          this.openTab({ key: 'f:' + id + path, kind: 'file', title: path.split('/').pop(), co: id, project: this.checkout(id).project, path: path });
        },
        // glow.rs: IntelliJ's project-colour glow at the top left, from the active tab's project;
        // the colour is one of IntelliJ's nine RecentProject MainToolbarGradientStart colours, picked by name hash.
        projectGlow: function () {
          var tab = this.currentTab();
          if (!tab || !tab.project) return 'var(--frame)';
          return 'var(--glow-' + Number(fnv1a(tab.project) % 9n) + ')';
        },
        currentTab: function () {
          var self = this;
          return this.tabs.find(function (t) { return t.key === self.focus; }) || null;
        },
        // tabs.rs close: when it was shown, its right neighbour shows, or the left one at the end.
        closeTab: function (tab) {
          // A tab whose save failed or conflicts keeps its edits: the bar first offers "Close without saving".
          if ((tab.conflict || tab.error) && tab.closing !== true) { tab.closing = true; this.showTab(tab); return; }
          var at = this.tabs.indexOf(tab);
          this.keepTab(tab);
          this.tabs = this.tabs.filter(function (t) { return t !== tab; });
          if (this.focus !== tab.key) return;
          var neighbour = this.tabs[at] || this.tabs[at - 1];
          this.focus = neighbour ? neighbour.key : null;
        },
        // Close Other Tabs, to the Left, to the Right: `tab` shows when the shown one closed.
        closeTabs: function (tab, closing) {
          var keys = closing.map(function (t) { return t.key; });
          if (keys.includes(this.temporary)) this.temporary = null;
          this.tabs = this.tabs.filter(function (t) { return keys.includes(t.key) === false; });
          if (keys.includes(this.focus)) this.focus = tab.key;
        },
        // The tab context menu: a right press opens it at the pointer without showing the tab.
        openTabMenu: function (tab, x, y) {
          this.tabMenu = { key: tab.key, x: x, y: y };
          this.menuSub = null;
          this.menu = 'tab';
          var self = this;
          this.$nextTick(function () { self.keepTabMenuInWindow(document.querySelector('.menu--at')); });
        },
        // Kept inside the window; the menu's own x-init calls it too, when it renders after the tick above.
        keepTabMenuInWindow: function (menu) {
          if (menu === null) return;
          var box = menu.getBoundingClientRect();
          this.tabMenu.x = Math.max(0, Math.min(this.tabMenu.x, window.innerWidth - box.width));
          this.tabMenu.y = Math.max(0, Math.min(this.tabMenu.y, window.innerHeight - box.height));
        },
        // Mock-only (#menu=tab): the menu on the shown tab, as a right press on its title would open it.
        openTabMenuOnShown: function () {
          var chip = document.querySelector('.tabs__tab--current');
          if (chip === null) return;
          var box = chip.getBoundingClientRect();
          this.openTabMenu(this.currentTab(), box.left + 24, box.top + box.height / 2);
        },
        menuTab: function () {
          var key = this.tabMenu.key;
          return this.tabs.find(function (t) { return t.key === key; }) || null;
        },
        // The menu's items, top to bottom (ive.rn's config.tab_menus.main): Keep Tab, Stop, Kill, then a file's items
        // (menus.js), then the Close items; null is a separator, dropped where it would dangle.
        tabMenuItems: function (tab) {
          var self = this;
          var c = tab.kind === 'console' ? this.consoleOf(tab) : null;
          var busy = c !== null && this.hasForeground(c);
          var top = [
            this.isTemporary(tab) && { title: 'Keep Tab', enabled: true, run: function () { self.keepTab(tab); } },
            c && { title: 'Stop', enabled: busy, run: function () { self.signal(c, 'TERM'); } },
            c && { title: 'Kill', enabled: busy, run: function () { self.signal(c, 'KILL'); } }
          ].filter(Boolean);
          var close = [
            { title: 'Close', enabled: true, run: function () { self.closeTab(tab); } },
            { title: 'Close Other Tabs', enabled: this.tabsBeside(tab, 'other').length > 0, run: function () { self.closeTabs(tab, self.tabsBeside(tab, 'other')); } },
            { title: 'Close Tabs to the Left', enabled: this.tabsBeside(tab, 'left').length > 0, run: function () { self.closeTabs(tab, self.tabsBeside(tab, 'left')); } },
            { title: 'Close Tabs to the Right', enabled: this.tabsBeside(tab, 'right').length > 0, run: function () { self.closeTabs(tab, self.tabsBeside(tab, 'right')); } }
          ];
          return window.IVE_MENUS.pruned(top.concat([null], this.tabFileItems(tab), [null], close));
        },
        runTabMenuItem: function (item) {
          this.menu = null;
          item.run();
        },
        tabsBeside: function (tab, side) {
          var at = this.tabs.indexOf(tab);
          if (side === 'left') return this.tabs.slice(0, at);
          if (side === 'right') return this.tabs.slice(at + 1);
          return this.tabs.filter(function (t) { return t !== tab; });
        },
        focusLastTab: function () {
          var last = this.tabs[this.tabs.length - 1];
          this.focus = last ? last.key : null;
        },
        // editor.rs Tab::icon_name
        tabIcon: function (tab) {
          if (tab.kind === 'console') return this.consoleIcon(this.consoleOf(tab));
          var icons = { projectPage: 'project-folder.svg', vars: 'run-config.svg', runConfigs: 'run-config.svg', diff: 'changes.svg', commit: 'commit.svg', gitLog: 'history.svg', settings: 'settings.svg', hub: 'locate.svg', conflicts: 'warning.svg' };
          return icons[tab.kind] || this.fileIcon(tab.title);
        },
        tabTitle: function (tab) {
          if (tab.kind === 'console') return this.consoleOf(tab).title;
          return tab.title;
        },
        // editor.rs Tab::tooltip: `<owner> — <title>`; a file's title is its path.
        tabTooltip: function (tab) {
          var title = tab.kind === 'file' ? this.checkout(tab.co).path + tab.path : this.tabTitle(tab);
          if (tab.kind === 'diff') return this.label(tab.co) + ' — ' + tab.path + ' (' + this.sourceText(this.checkout(tab.co), tab.source) + ')';
          if (tab.kind === 'projectPage' || tab.kind === 'settings') return title;
          if (tab.co) return this.label(tab.co) + ' — ' + title;
          return tab.project + ' — ' + title;
        },
        // editor.rs Tab::underline: a diff tab's checkout colour.
        tabUnderline: function (tab) {
          return tab.kind === 'diff' ? this.checkout(tab.co).colour : null;
        },
        // ---- diff layout, editing and saving (codeview design: "Saving: IntelliJ-style, no Ctrl+S") ----
        layoutOf: function (tab) { return tab.layout || this.diffLayout; },
        setLayout: function (tab, layout) { tab.layout = layout; this.diffLayout = layout; },
        // Only a single-file diff of uncommitted changes edits (its new side); commit tabs, since-base diffs and the hub preview are read-only.
        isEditable: function (tab) {
          if (tab.kind === 'diff') return tab.source === 'work';
          return tab.kind === 'file' || tab.kind === 'scratch';
        },
        // Files over 8 MiB show without colours, and a conflict on them only offers Reload.
        isBig: function (tab) { return tab.big === true; },
        codeLines: function (tab) {
          var lines = this.lines(tab);
          return this.highlightLines(lines, tab.kind === 'scratch' ? tab.title : tab.path, this.isBig(tab) === false && lines.length < 5000);
        },
        languageName: function (tab) {
          var m = /\.([A-Za-z0-9]+)$/.exec(tab.kind === 'scratch' ? tab.title : tab.path || '');
          var names = { rs: 'Rust', js: 'JavaScript', ts: 'TypeScript', java: 'Java', ex: 'Elixir', py: 'Python', sh: 'Shell', bash: 'Shell', toml: 'TOML', yml: 'YAML', yaml: 'YAML', json: 'JSON', md: 'Markdown' };
          return (m && names[m[1].toLowerCase()]) || 'Plain text';
        },
        // The text typed in an editor is kept at once; "saving…" shows until half a second after the last key.
        edited: function (tab, value) {
          if (tab.kind === 'scratch') this.scratch(tab.title).text = value;
          else { var e = this.findFile(tab.co, tab.path); if (e) e.text = value; else tab.draft = value; }
          this.touched(tab);
        },
        touched: function (tab) {
          if (tab.conflict) return;
          tab.saving = true;
          clearTimeout(this.savingTimers[tab.key]);
          this.savingTimers[tab.key] = setTimeout(function () { tab.saving = false; }, 500);
        },
        editorWidth: function (tab) {
          var longest = this.lines(tab).reduce(function (n, l) { return Math.max(n, l.length); }, 0);
          return 'calc(' + (longest + 4) + 'ch + var(--gutter-w))';
        },
        // Mock bar: the shown tab gets a conflict ('disk', or 'big': the file is over 8 MiB), a save error, or both clear.
        mockSave: function (what) {
          var tab = this.currentTab();
          if (tab === null || this.isEditable(tab) === false) return;
          tab.conflict = what === 'disk' || what === 'big' ? what : null;
          tab.big = what === 'big';
          tab.error = what === 'error';
          tab.closing = false;
          tab.saving = false;
        },
        // Reload takes the disk's text, Overwrite keeps the editor's; both end the conflict.
        resolveConflict: function (tab) { tab.conflict = null; tab.closing = false; this.touched(tab); },
        saveBarText: function (tab) {
          var name = tab.kind === 'scratch' ? tab.title : tab.path.split('/').pop();
          if (tab.conflict) return name + ' changed on disk';
          if (tab.error) return 'cannot save ' + name + ': Permission denied (your edits are kept; saving is retried after the next pause)';
          return '';
        },
        closeWithoutSaving: function (tab) { tab.conflict = null; tab.error = false; this.closeTab(tab); },
        lines: function (tab) {
          if (tab.kind === 'file' && tab.draft !== undefined) return tab.draft.split('\n');
          if (tab.kind === 'file') { var e = this.findFile(tab.co, tab.path); return e ? e.text.split('\n') : window.IVE_HUB.placeholderLines(tab.path); }
          if (tab.kind === 'scratch') return this.scratch(tab.title).text.split('\n');
          return [];
        },
        findFile: function (id, path) { return find(this.checkout(id).tree, path); },
        openSettings: function () { this.openTab({ key: 'settings', kind: 'settings', title: 'Settings', co: null, project: null }); },
        // Settings › Scripts: the instructions as plain text, as scripts.rs instructions() writes them.
        scriptInstructions: function () {
          var paragraphs = SCRIPT_SECTIONS.map(function (s) { return s[0] ? s[0] + ' ' + s[1] : s[1]; });
          return 'Customising IVE\n\n' + paragraphs.join('\n\n') + '\n\nFull reference: "Scripting" and "Theme" in IVE\'s README: https://github.com/Warsaw-Dynamics/ive#scripting\n';
        },
        copyText: function (id, text) {
          var self = this;
          try { navigator.clipboard.writeText(text); } catch (e) { /* mock: clipboard may be blocked */ }
          this.copied = id;
          clearTimeout(this.copiedTimer);
          this.copiedTimer = setTimeout(function () { self.copied = null; }, 1500);
        },

        // ---- scratches ----
        scratch: function (name) { return this.scratches.find(function (s) { return s.name === name; }); },
        openScratch: function (s) {
          this.openTab({ key: 's:' + s.name, kind: 'scratch', title: s.name, co: null, project: s.project });
        },
        // scratch/tab.rs render_header
        scratchHeader: function (tab) {
          var s = this.scratch(tab.title);
          return 'scratch of ' + s.project + ' · ' + CONFIG + 'scratches/' + s.project + '/' + s.name + ' · runs in ⎇ ' + this.label(this.targetCheckout(this.project(s.project)));
        },
        isShellScratch: function (name) { return /\.(sh|bash|zsh)$/.test(name); },
        projectScratches: function (project) {
          return this.scratches.filter(function (s) { return s.project === project.name; });
        },
        // scratch/groups.rs: Today, This week (7 days), This month (30), Older; Older starts collapsed.
        scratchBuckets: function (project, query) {
          var self = this;
          var today = window.IVE_SCALE.today;
          var midnight = new Date(today.getFullYear(), today.getMonth(), today.getDate());
          var list = this.projectScratches(project).filter(function (s) { return self.matches(s.name + ' ' + s.text, query); })
            .sort(function (a, b) { return b.modified - a.modified; });
          var buckets = [
            { key: 'today', title: 'Today', test: function (s) { return s.modified >= midnight; } },
            { key: 'week', title: 'This week', test: function (s) { return s.modified >= midnight - 7 * DAY; } },
            { key: 'month', title: 'This month', test: function (s) { return s.modified >= midnight - 30 * DAY; } },
            { key: 'older', title: 'Older', test: function () { return true; } }
          ];
          var placed = new Set();
          return buckets.map(function (b) {
            var items = list.filter(function (s) { return placed.has(s) === false && b.test(s); });
            items.forEach(function (s) { placed.add(s); });
            return { key: b.key, title: b.title, items: items };
          }).filter(function (b) { return b.items.length; });
        },
        newScratch: function (project) {
          var s = { name: 'scratch-' + (this.projectScratches(project).length + 1) + '.md', text: '', project: project.name, modified: window.IVE_SCALE.today };
          this.scratches.unshift(s);
          this.openScratch(s);
        },
        deleteScratch: function (s) { this.scratches = this.scratches.filter(function (x) { return x !== s; }); },

        // ---- processes ----
        // kind: shell (a terminal) | agent (Claude Code, which a terminal turns into while `claude` runs).
        addConsole: function (co, kind, lines) {
          var c = { id: ++this.consoleCount, co: co, kind: kind, title: '', lines: lines, input: '' };
          c.title = kind === 'agent' ? 'Claude Code' : this.shellTitle(co);
          this.consoles.push(c);
          return c;
        },
        // The shell sets the title (OSC 0): user@host: dir.
        shellTitle: function (co) { return USER + ': ' + this.checkout(co).path; },
        consoleOf: function (tab) { return this.consoles.find(function (c) { return 'c:' + c.id === tab.key; }) || null; },
        consoleIcon: function (c) {
          if (c !== null && c.kind === 'run') return c.serve ? 'file-html.svg' : 'run.svg';
          return c !== null && c.kind === 'agent' ? 'agent.svg' : 'terminal.svg';
        },
        // process.rs description
        consoleKind: function (c) {
          if (c.kind === 'run') return c.serve ? 'static server' : 'run';
          return c.kind === 'agent' ? 'Claude Code session' : 'terminal';
        },
        openConsole: function (c) {
          this.usedConsole(c);
          this.openTab({ key: 'c:' + c.id, kind: 'console', title: c.title, co: c.co, project: c.co.split('@')[0] });
        },
        newTerminal: function (co) { this.openConsole(this.addConsole(co, 'shell', [])); },
        becomeAgent: function (c) {
          c.kind = 'agent';
          c.title = 'Claude Code';
          c.lines.push('✻ Welcome to Claude Code', '', '  cwd: ' + this.checkout(c.co).path);
        },
        becomeShell: function (c) {
          c.kind = 'shell';
          c.title = this.shellTitle(c.co);
        },
        // Alt+F12: a new terminal in the current tab's checkout.
        openTerminal: function () {
          var current = this.currentTab();
          this.newTerminal(current && current.co ? current.co : this.targetCheckout(this.treeProjects()[0]));
        },
        // Stop: SIGTERM to Claude Code, which leaves the shell; Close ends a terminal and its tab.
        stop: function (c) { c.lines.push('', '(stopped)'); this.becomeShell(c); },
        // Whether a program runs in the foreground: Claude Code, or a live run; an idle shell has none.
        hasForeground: function (c) { return c.kind === 'agent' || (c.kind === 'run' && c.state !== 'exited'); },
        // The tab menu's Stop (TERM; a run's program gets KILL 5 s later) and Kill (KILL at once), to the foreground program.
        signal: function (c, name) {
          if (c.kind === 'run') { this.stopRun(c, name); return; }
          c.lines.push('', name === 'KILL' ? '(killed)' : '(stopped)');
          this.becomeShell(c);
        },
        closeConsole: function (c) {
          this.consoles = this.consoles.filter(function (x) { return x !== c; });
          var tab = this.tabs.find(function (t) { return t.key === 'c:' + c.id; });
          if (tab) this.closeTab(tab);
        },
        prompt: function (id) {
          var co = this.checkout(id);
          return { user: USER, dir: co.path, branch: co.branch };
        },
        submit: function (c) {
          var command = c.input.trim();
          c.input = '';
          this.useWorktree(c.co);
          this.usedConsole(c);
          if (c.kind === 'agent' && command === '/exit') { c.lines.push('> /exit', ''); this.becomeShell(c); return; }
          if (c.kind === 'agent') {
            if (command) c.lines.push('> ' + command, '● (mock) Claude Code works on it in ' + this.checkout(c.co).path);
            return;
          }
          if (command === 'claude') { c.lines.push({ prompt: this.prompt(c.co), command: command }); this.becomeAgent(c); return; }
          if (command === 'clear') { c.lines = []; return; }
          if (command === 'exit') { this.closeConsole(c); return; }
          c.lines.push({ prompt: this.prompt(c.co), command: command });
          this.output(c, command).forEach(function (line) { c.lines.push(line); });
          this.$nextTick(function () {
            var screen = document.querySelector('.terminal');
            if (screen) screen.scrollTop = screen.scrollHeight;
          });
        },
        // Canned answers; the real tab runs $SHELL through alacritty_terminal.
        output: function (c, command) {
          var co = this.checkout(c.co);
          var project = this.project(co.project);
          if (command === '') return [];
          if (command === 'pwd') return [co.path];
          if (command === 'ls') return [co.tree.map(function (e) { return e.name + (e.children ? '/' : ''); }).join('  ')];
          if (command.startsWith('git') && project.git === false) return ['fatal: not a git repository (or any of the parent directories): .git'];
          if (command === 'git status') {
            return ['On branch ' + co.branch].concat(co.changes.length
              ? ['Changes not staged for commit:'].concat(co.changes.map(function (x) { return '\t' + (x.mark === 'A' ? 'new file' : 'modified') + ':   ' + x.path; }))
              : ['nothing to commit, working tree clean']);
          }
          if (command === 'git worktree list') return project.checkouts.map(function (x) { return x.path + '  [' + x.branch + ']'; });
          return ['bash: ' + command.split(' ')[0] + ': command not found', 'try: pwd, ls, git status, git worktree list, claude, clear, exit'];
        },
        isPromptLine: function (line) { return typeof line === 'object'; },

        // ---- tree nodes (sidebar/rows.rs) ----
        // A node: { key, label, icon?, children?, open (default), click?, tab?, project?, pill?, count?, meta?, strong?,
        // slot? (a file's empty chevron slot), action? ({ icon, title, run }) }.
        isNodeOpen: function (node) { return node.open !== this.toggled.includes(node.key); },
        toggleNode: function (node) {
          var at = this.toggled.indexOf(node.key);
          if (at === -1) this.toggled.push(node.key); else this.toggled.splice(at, 1);
        },
        activateNode: function (node) {
          if (node.click) { node.click(); return; }
          if (node.children) this.toggleNode(node);
        },
        // Middle click: as a click, then the row's tab stays (sidebar/view.rs ActivateRowToKeep).
        activateNodeToKeep: function (node) {
          this.activateNode(node);
          var tab = this.currentTab();
          if (tab && tab.key === node.tab) this.keepTab(tab);
        },
        treeRows: function () {
          var self = this;
          var out = [];
          (function walk(list, depth) {
            list.forEach(function (node) {
              out.push({ node: node, depth: depth });
              if (node.children && self.isNodeOpen(node)) walk(node.children, depth + 1);
            });
          })(this.treeProjects().map(function (p) { return self.projectNode(p); }), 0);
          return out;
        },
        // The projects the tree shows: the open ones the header's picker did not leave out, all when it would leave
        // none. The hub and New terminal list these alone too.
        treeProjects: function () {
          var hidden = this.hiddenProjects;
          var shown = this.openProjects().filter(function (p) { return hidden.includes(p.name) === false; });
          return shown.length ? shown : this.openProjects();
        },
        treeProjectNames: function () { return this.treeProjects().map(function (p) { return p.name; }); },
        isProjectShown: function (name) { return this.treeProjectNames().includes(name); },
        allProjectsShown: function () { return this.treeProjects().length === this.openProjects().length; },
        // The one project shown while others are left out, else null.
        onlyShown: function () { return this.allProjectsShown() ? null : (this.treeProjects().length === 1 ? this.treeProjects()[0].name : null); },
        // The header's title, and the hub's All: All projects, the one shown, or 2 of 4 projects.
        allLabel: function () {
          if (this.allProjectsShown()) return 'All projects';
          return this.onlyShown() || this.treeProjects().length + ' of ' + this.openProjects().length + ' projects';
        },
        // A project's row in the picker: shows it or leaves it out; the last one shown stays. The menu stays open.
        toggleProjectShown: function (name) {
          if (this.hiddenProjects.includes(name)) {
            this.hiddenProjects = this.hiddenProjects.filter(function (h) { return h !== name; });
            return;
          }
          if (this.isProjectShown(name) && this.treeProjects().length === 1) return;
          this.hiddenProjects = this.hiddenProjects.concat([name]);
        },
        showAllProjects: function () { this.hiddenProjects = []; },
        // The row the shown tab belongs to is selected.
        isRowSelected: function (node) {
          var tab = this.currentTab();
          return tab !== null && node.tab !== undefined && node.tab === tab.key;
        },
        fileNodes: function (co) {
          var self = this;
          var build = function (entry) {
            var key = 'f:' + co.id + entry.path;
            // A folder with a .run.toml: the folder icon with a run badge (as IntelliJ marks modules), nothing else:
            // its runs are in the worktree's Processes. A broken .run.toml leaves a plain folder: only the file's own row warns.
            // what the row's menu applies to (menus.js)
            var subject = { co: co.id, path: entry.path, folder: Boolean(entry.children) };
            if (entry.children && self.hasRunFile(entry) && RUN_FILES[entry.path].error === undefined) {
              return { key: key, entry: subject, label: entry.name, icon: 'folder-run.svg', open: false, children: self.sortedNodes(co, entry.path, entry.children, build) };
            }
            if (entry.children) return { key: key, entry: subject, label: entry.name, icon: 'folder.svg', open: false, children: self.sortedNodes(co, entry.path, entry.children, build) };
            // a broken .run.toml: a warning after its name, the error as its tooltip
            var folder = entry.path.slice(0, entry.path.lastIndexOf('/'));
            var error = entry.name === '.run.toml' && RUN_FILES[folder] ? RUN_FILES[folder].error : undefined;
            return { key: key, tab: key, entry: subject, label: entry.name, icon: self.fileIcon(entry.name), slot: true, warning: error,
              click: function () { self.openFile(co.id, entry.path); } };
          };
          return this.sortedNodes(co, '', co.tree, build);
        },
        // file_tree.rs: two or more hidden entries go under @hidden, first; then folders, then files, each by name.
        sortedNodes: function (co, folder, entries, build) {
          var sorted = entries.slice().sort(function (a, b) { return (b.children ? 1 : 0) - (a.children ? 1 : 0) || a.name.localeCompare(b.name); });
          var hidden = sorted.filter(function (e) { return e.name.startsWith('.') && e.name !== '.run.toml'; });
          if (hidden.length < 2) return sorted.map(build);
          var group = { key: 'h:' + co.id + folder, label: HIDDEN_NODE, icon: 'folder-excluded.svg', open: false, special: true, weak: true, children: hidden.map(build) };
          return [group].concat(sorted.filter(function (e) { return hidden.includes(e) === false; }).map(build));
        },
        consoleNode: function (c) {
          var self = this;
          // a leaf: the empty chevron slot keeps its icon in the files' column
          return { key: 'c:' + c.id, tab: 'c:' + c.id, label: c.title, icon: this.consoleIcon(c), slot: true, click: function () { self.openConsole(c); },
            action: c.kind === 'agent'
              ? { icon: 'stop.svg', title: 'Stop', run: function () { self.stop(c); } }
              : { icon: 'close.svg', title: 'Close', run: function () { self.closeConsole(c); } } };
        },
        // What a checkout holds: the folders switched on (Processes by default), then Files.
        checkoutChildren: function (co) {
          var self = this;
          return this.worktreeFolders(co).filter(function (n) { return self.isFolderShown(n); });
        },
        // Every folder of a worktree, empty or not, so each can be switched on (Processes' + starts a terminal).
        worktreeFolders: function (co) {
          var self = this;
          // terminals and Claude Code, then every run of the worktree: running, waiting, exited, stopped, failed
          var all = this.consoles.filter(function (c) { return c.co === co.id; });
          var processes = all.filter(function (c) { return c.kind !== 'run'; }).concat(all.filter(function (c) { return c.kind === 'run'; }));
          // the Git overview's Since <base>: its commits' files, and the uncommitted ones while "with uncommitted" is on
          var sinceBase = this.branchFiles(co);
          // folders.rs Nothing: an empty git folder stays, muted, no chevron, `· 0` (or `· no base`), why in its tooltip
          var nothing = function (tip, mark) { return { slot: true, empty: true, count: mark || '0', hint: tip }; };
          var noBase = co.base ? null : nothing('No base branch to compare with', 'no base');
          var content = {
            // empty: no chevron, the slot keeps its icon in line
            processes: { children: processes.length ? processes.map(function (c) { return self.processNode(c); }) : undefined, slot: processes.length === 0,
              count: processes.length || null, action: this.newTerminalAction(co),
              tab: 'rcs:' + co.id, click: function () { self.openRunConfigs(co); } },
            uncommitted: co.changes.length ? { children: this.changedFileNodes(co, co.changes, 'work'), count: co.changes.length } : nothing('Nothing uncommitted'),
            sinceBase: noBase || (sinceBase.length ? { children: this.changedFileNodes(co, sinceBase, 'branch'), count: sinceBase.length, hint: 'Since ' + co.base }
              : nothing('No changes since ' + co.base)),
            commits: noBase || (co.commits.length ? { children: this.commitNodes(co), count: co.commits.length } : nothing('No commits since ' + co.base)),
            files: { hint: co.path, children: this.fileNodes(co) }
          };
          return WORKTREE_FOLDERS.map(function (f) {
            return Object.assign({ key: 'wf:' + co.id + '/' + f.key, folder: f, label: f.title, icon: f.icon, open: true }, content[f.key]);
          });
        },
        // A process row; a run carries its state (● running · :8767, … waiting for build, exit 1) and, on hover,
        // Restart and Stop while it is live, Start and Remove once it exited (sidebar/view.rs).
        processNode: function (c) {
          var self = this;
          var node = this.consoleNode(c);
          if (c.kind !== 'run') return node;
          node.state = this.runLabel(c);
          node.action = undefined;
          node.actions = c.state !== 'exited'
            ? [{ icon: 'restart.svg', title: 'Restart', run: function () { self.restartRun(c, 'tab'); } },
               { icon: 'stop.svg', title: 'Stop', run: function () { self.stopRun(c); } }]
            : [{ icon: 'run.svg', title: 'Start', run: function () { self.restartRun(c, 'tab'); } },
               { icon: 'close.svg', title: 'Remove', run: function () { self.closeRun(c); } }];
          return node;
        },
        // A worktree follows everyFolder but for the folders it set apart earlier (until Reset); the folder still
        // folds on its own.
        isFolderShown: function (node) { return this.everyFolder[node.folder.key] !== this.ownsFolder(node); },
        ownsFolder: function (node) { return this.toggled.includes('fs:' + node.key); },
        toggleFolderShown: function (node, event) {
          if (event && event.altKey) return this.showFolderEverywhere(node.folder.key, this.isFolderShown(node) === false);
          var key = 'fs:' + node.key;
          var at = this.toggled.indexOf(key);
          if (at === -1) this.toggled.push(key); else this.toggled.splice(at, 1);
        },
        // Every worktree shows folder `key` or not; the worktrees that had switched it themselves follow too.
        showFolderEverywhere: function (key, on) {
          this.everyFolder[key] = on;
          this.toggled = this.toggled.filter(function (k) { return (k.startsWith('fs:wf:') && k.endsWith('/' + key)) === false; });
        },
        // The worktrees whose folders differ from everyFolder (the Projects header menu's Reset).
        foldersOwned: function () {
          var ids = this.toggled.filter(function (k) { return k.startsWith('fs:wf:'); }).map(function (k) { return k.slice('fs:wf:'.length, k.lastIndexOf('/')); });
          return Array.from(new Set(ids));
        },
        resetOwnFolders: function () { this.toggled = this.toggled.filter(function (k) { return k.startsWith('fs:wf:') === false; }); },
        // Alt+Shift+G: changed files flat or grouped by folder, here and in the Git overview.
        groupKey: function (event) {
          if (event.altKey && event.shiftKey && event.code === 'KeyG' && event.ctrlKey === false && event.metaKey === false) {
            event.preventDefault();
            this.toggleGitLayout();
          }
        },
        toggleGitLayout: function () { this.gitLayout = this.gitLayout === 'flat' ? 'tree' : 'flat'; },
        // Ctrl+Alt+1 to 5 (ive.rn's act::ToggleFolderEverywhere keys): a worktree folder shown or hidden in every
        // worktree, as the header menu's ticks; without Shift a digit is a digit on every layout.
        folderKey: function (event) {
          var folder = WORKTREE_FOLDERS.find(function (f) { return 'Digit' + f.digit === event.code; });
          if (folder && event.ctrlKey && event.altKey && event.shiftKey === false && event.metaKey === false) {
            event.preventDefault();
            this.showFolderEverywhere(folder.key, this.everyFolder[folder.key] === false);
          }
        },
        // Processes' + on hover: a terminal in the worktree.
        newTerminalAction: function (co) {
          var self = this;
          return { icon: 'add.svg', title: 'New terminal in ' + co.label + ' (type claude to start Claude Code)', run: function () { self.newTerminal(co.id); } };
        },
        // Commits: a worktree's commits since its base, newest first: the first PAGE, then Load 20 more adds
        // COMMITS_MORE each click. A click folds a commit's files (with their marks) out and in; a file opens its diff.
        commitNodes: function (co) {
          var self = this;
          var shared = sharedSubject(co.commits);
          var key = 'commits:' + co.id;
          var shown = Math.min(co.commits.length, this.shown[key] || PAGE);
          var nodes = co.commits.slice(0, shown).map(function (c) {
            return { key: 'cm:' + co.id + '/' + c.sha, label: commitLabel(c.message, shared), icon: 'commit.svg', open: false, meta: c.sha + ' · ' + c.when, hint: c.sha + ' ' + c.message,
              children: self.changedFileNodes(co, c.files, c.sha) };
          });
          var rest = co.commits.length - shown;
          if (rest > 0) nodes.push({ key: 'cm:' + co.id + '/more', label: 'Load ' + Math.min(rest, COMMITS_MORE) + ' more · ' + shown + ' of ' + co.commits.length, link: true, slot: true,
            click: function () { self.shown[key] = shown + COMMITS_MORE; } });
          return nodes;
        },
        // Changed files of a source (work, or a commit's sha) in the git list's layout (gitLayout, shared with the Git
        // overview): Flat, a row per file with its folder after it; Tree, folders as its Tree layout groups them (a
        // chain of single folders is one row), open, each with its file count. A click opens the file's diff.
        changedFileNodes: function (co, files, source) {
          var self = this;
          var fileNode = function (x) {
            var at = x.path.lastIndexOf('/');
            return { key: 'cg:' + co.id + '/' + source + ':' + x.path, tab: 'df:' + co.id + '/' + source + ':' + x.path, entry: { co: co.id, path: '/' + x.path, folder: false }, label: x.path.slice(at + 1),
              icon: self.fileIcon(x.path), slot: true, mark: x.mark, struck: x.mark === 'D', hint: x.path,
              meta: self.gitLayout === 'flat' && at !== -1 ? x.path.slice(0, at) : null, click: function () { self.openDiff(co, x, source); } };
          };
          if (this.gitLayout === 'flat') return files.slice().sort(function (a, b) { return a.path.localeCompare(b.path); }).map(fileNode);
          var tree = fileTree(files);
          var build = function (node, path, depth) {
            return tree.walk(node, path, depth, []).map(function (row) {
              if (row.type === 'file') return fileNode(row.file);
              // a folder holding one file shows no count: the file is the next row
              return { key: 'cg:' + co.id + '/' + source + ':' + row.path + '/', label: row.label, icon: 'folder.svg', open: false, count: row.files > 1 ? row.files : null,
                hint: row.path, children: build(row.node, row.path, depth + 1) };
            });
          };
          return openWithin(build(tree.root, '', 0), CHANGE_ROWS);
        },
        worktreeNode: function (co) {
          var self = this;
          return { key: 'w:' + co.id, entry: { co: co.id, path: '', folder: true }, label: co.label, icon: 'branch.svg', open: true,
            children: this.checkoutChildren(co),
            action: co.main ? null : { icon: 'hide.svg', title: 'Hide worktree (the project page shows it again)', run: function () { self.hideWorktree(co); } } };
        },
        // Project children: worktrees, or the single checkout's content.
        projectNode: function (project) {
          var self = this;
          var single = this.hasWorktreeLevel(project) === false;
          var body = single ? this.checkoutChildren(project.checkouts[0]) : this.shownCheckouts(project).map(function (co) { return self.worktreeNode(co); });
          var running = this.projectConsoles(project).length;
          return { tab: 'pp:' + project.name, click: function () { self.openProjectPage(project); }, key: 'p:' + project.name, label: project.name,
            entry: { co: project.checkouts[0].id, path: '', folder: true },
            project: project, strong: true, open: true, children: body,
            pill: single && project.git ? project.checkouts[0].label : null,
            meta: project.git ? null : 'no git', running: running ? '● ' + running + ' running' : null,
            action: { icon: 'close.svg', title: 'Close project', run: function () { self.closeProject(project); } } };
        },

        // ---- run configurations (a folder with a .run.toml; its runs are in Processes, its configurations in the tab) ----
        hasRunFile: function (entry) { return entry.children.some(function (c) { return c.name === '.run.toml'; }); },
        // A run's state as words on its row, coloured by runTone: ● running · :port, … waiting for <step>,
        // exit <code>, ■ stopped, ✕ <why the chain failed>.
        runLabel: function (c) {
          var cfg = this.config(c.folder, c.key);
          var tone = this.runTone(c);
          if (c.state === 'running') return { text: '● running' + (cfg && cfg.type === 'serve' ? ' · :' + this.servePort(c) : ''), tone: tone };
          if (c.state === 'waiting') return { text: '… waiting for ' + c.waitingOn, tone: tone };
          if (c.failed) return { text: '✕ ' + c.failed, tone: tone };
          return { text: c.stopped ? '■ stopped' : (c.code === 0 ? '' : '✕ ') + 'exit ' + c.code, tone: tone };
        },
        servePort: function (c) {
          var co = this.checkout(c.co);
          var shown = this.shownCommand(co, c.folder, this.config(c.folder, c.key));
          return shown.split(':').pop();
        },
        // A configuration row of the run configurations tab, its run's state on it. A click shows the run's output (nothing without a run:
        // ▷ runs it); hover: Restart and Stop while it runs, Start and Remove once exited; ▷ Run always shown before.
        // where: 'tab' (the tree) shows the run's tab once it starts; 'stay' (the run configurations tab) keeps the list shown.
        configNode: function (co, folder, cfg, where) {
          var self = this;
          var run = this.runsOf(co.id, folder).find(function (c) { return c.key === cfg.key; }) || null;
          var live = run !== null && run.state !== 'exited';
          var start = function () { self.requestRun(co.id, folder, cfg.key, where || 'tab'); };
          var actions = run === null ? [{ icon: 'run.svg', title: 'Run', run: start }]
            : live ? [{ icon: 'restart.svg', title: 'Restart', run: function () { self.restartRun(run, where || 'tab'); } },
                      { icon: 'stop.svg', title: 'Stop', run: function () { self.stopRun(run); } }]
            : [{ icon: 'run.svg', title: 'Start', run: function () { self.restartRun(run, where || 'tab'); } },
               { icon: 'close.svg', title: 'Remove', run: function () { self.closeRun(run); } }];
          return { key: 'rc:' + co.id + folder + '#' + cfg.key, tab: run ? 'c:' + run.id : undefined, label: cfg.name,
            icon: cfg.type === 'serve' ? 'file-html.svg' : 'run-config.svg', slot: true,
            state: run ? this.runLabel(run) : null,
            hint: cfg.name + ' (' + cfg.type + '.' + cfg.key + '): ' + this.shownCommand(co, folder, cfg) + (run ? ' · ' + this.runStatus(run) + ' · started ' + age(run.started) : ''),
            click: run ? function () { self.openConsole(run); } : undefined,
            pinned: run === null, actions: actions };
        },
        runFile: function (folder) { return RUN_FILES[folder]; },
        // Mock-only (dnd.js): a copied or moved .run.toml reads as the one it came from.
        adoptRunFile: function (from, to) { if (RUN_FILES[from] !== undefined) RUN_FILES[to] = RUN_FILES[from]; },

        // ---- the run configurations tab (a click on a worktree's Processes row) ----
        openRunConfigs: function (co) {
          this.openTab({ key: 'rcs:' + co.id, kind: 'runConfigs', title: 'Run configurations', co: co.id, project: co.project });
        },
        // Every configuration of every .run.toml in the checkout that the filter matches (name, folder, command).
        runConfigEntries: function (co) {
          var self = this;
          return this.runFolders(co.id).flatMap(function (folder) {
            return RUN_FILES[folder].configs.map(function (cfg) {
              return { folder: folder, command: self.shownCommand(co, folder, cfg), node: self.configNode(co, folder, cfg, 'stay') };
            });
          }).filter(function (e) { return self.matches(e.node.label + ' ' + e.folder + ' ' + e.command, self.query.run); });
        },
        // 8 configurations in 3 files · 2 running · 1 broken file
        runConfigSummary: function (co) {
          var folders = this.runFolders(co.id);
          var count = folders.reduce(function (n, folder) { return n + RUN_FILES[folder].configs.length; }, 0);
          var live = this.consoles.filter(function (c) { return c.kind === 'run' && c.co === co.id && c.state !== 'exited'; }).length;
          var broken = this.brokenRunFolders(co.id).length;
          return [counted(count, 'configuration', 'configurations') + ' in ' + counted(folders.length, 'file', 'files'),
            live ? live + ' running' : null, broken ? counted(broken, 'broken file', 'broken files') : null].filter(Boolean).join(' · ');
        },
        // The tab's rows, as the tree's ({ node, depth }, drawn by rows.html): flat, each configuration by name with its
        // folder after it; tree, its folders. A .run.toml that does not parse is a row with a warning (last in flat).
        runConfigRows: function (co) {
          var self = this;
          var entries = this.runConfigEntries(co);
          var broken = this.query.run ? [] : this.brokenRunFolders(co.id);
          if (this.runLayout === 'tree') return this.runConfigTreeRows(co, entries, broken);
          var rows = entries.slice().sort(function (a, b) { return a.node.label.localeCompare(b.node.label); })
            .map(function (e) { return { node: Object.assign({}, e.node, { meta: e.folder.slice(1) }), depth: 0 }; });
          return rows.concat(broken.map(function (folder) { return { node: self.brokenRunFileNode(co, folder, folder.slice(1) + '/.run.toml'), depth: 0 }; }));
        },
        brokenRunFileNode: function (co, folder, label) {
          var self = this;
          return { key: 'rcb:' + co.id + folder, label: label, icon: 'file-toml.svg', slot: true, warning: RUN_FILES[folder].error,
            click: function () { self.openFile(co.id, folder + '/.run.toml'); } };
        },
        // Folders as the Files tree nests them, a chain of single folders as one row (internal-apps/misc when alone);
        // a folder's subfolders first, then its configurations in file order. A folder with a .run.toml opens it on hover.
        runConfigTreeRows: function (co, entries, broken) {
          var self = this;
          var root = { path: '', folders: {}, entries: [], broken: false };
          var place = function (path) {
            var node = root;
            path.split('/').filter(Boolean).forEach(function (name) {
              node.folders[name] = node.folders[name] || { path: node.path + '/' + name, folders: {}, entries: [], broken: false };
              node = node.folders[name];
            });
            return node;
          };
          entries.forEach(function (e) { place(e.folder).entries.push(e); });
          broken.forEach(function (folder) { place(folder).broken = true; });
          var out = [];
          var walk = function (node, label, depth) {
            var names = Object.keys(node.folders);
            if (node.entries.length === 0 && node.broken === false && names.length === 1) {
              walk(node.folders[names[0]], label + '/' + names[0], depth);
              return;
            }
            var file = RUN_FILES[node.path] !== undefined && RUN_FILES[node.path].error === undefined;
            var fold = { key: 'rcf:' + co.id + node.path, label: label, icon: file ? 'folder-run.svg' : 'folder.svg', open: true, children: [],
              runs: true, live: self.runConfigLive(node), hint: node.path.slice(1),
              action: file ? { icon: 'file-toml.svg', title: 'Open ' + node.path.slice(1) + '/.run.toml', run: function () { self.openFile(co.id, node.path + '/.run.toml'); } } : undefined };
            out.push({ node: fold, depth: depth });
            if (self.isNodeOpen(fold) === false) return;
            names.sort().forEach(function (name) { walk(node.folders[name], name, depth + 1); });
            if (node.broken) out.push({ node: self.brokenRunFileNode(co, node.path, '.run.toml'), depth: depth + 1 });
            node.entries.forEach(function (e) { out.push({ node: e.node, depth: depth + 1 }); });
          };
          Object.keys(root.folders).sort().forEach(function (name) { walk(root.folders[name], name, 0); });
          return out;
        },
        runConfigLive: function (node) {
          var self = this;
          return node.entries.filter(function (e) { return e.node.state && e.node.state.tone === 'running'; }).length
            + Object.values(node.folders).reduce(function (n, f) { return n + self.runConfigLive(f); }, 0);
        },
        // The folders of checkout `id` with a .run.toml that parses.
        runFolders: function (id) { return this.runFilesIn(id).filter(function (folder) { return RUN_FILES[folder].error === undefined; }); },
        brokenRunFolders: function (id) { return this.runFilesIn(id).filter(function (folder) { return RUN_FILES[folder].error !== undefined; }); },
        runFilesIn: function (id) { var self = this; return Object.keys(RUN_FILES).filter(function (folder) { return self.findFile(id, folder + '/.run.toml') !== null; }); },
        config: function (folder, key) { return RUN_FILES[folder].configs.find(function (c) { return c.key === key; }); },
        builtins: function (co, folder) { return { USER: 'kb', HOME: '~', REPO: co.path, DIR: co.path + folder }; },
        // A variable's value: vars.toml (a string, or a wallet entry that must exist), else its default; undefined when
        // it has none.
        varValue: function (folder, name) {
          var set = this.vars[name];
          if (typeof set === 'string') return set;
          if (set && this.wallet[set.wallet] !== undefined) return this.wallet[set.wallet];
          if (set) return undefined;
          return RUN_FILES[folder].vars[name].default;
        },
        // Where a config uses ${name}: command, cwd, env values, root, port.
        referenced: function (cfg) {
          var text = [cfg.command, cfg.cwd, cfg.root, cfg.port].concat(Object.values(cfg.env || {})).filter(Boolean).join(' ');
          return Array.from(text.matchAll(/(?<!\$)\$\{([a-z0-9_-]+)\}/gi), function (m) { return m[1]; });
        },
        // The variables a run of `key` needs a value for: its own and its before steps', only those without one.
        missingVars: function (folder, key) {
          var self = this;
          var file = RUN_FILES[folder];
          var cfg = this.config(folder, key);
          var names = (cfg.before || []).concat(key).flatMap(function (k) { return self.referenced(self.config(folder, k)); });
          return Array.from(new Set(names)).filter(function (n) { return file.vars[n] && self.varValue(folder, n) === undefined; });
        },
        // The command as the row shows it: built-ins and plain values filled in, secrets left as ${name}.
        shownCommand: function (co, folder, cfg) {
          var self = this;
          var file = RUN_FILES[folder];
          var builtins = this.builtins(co, folder);
          var fill = function (text) {
            return String(text).replace(/(\$?)\$\{([a-z0-9_-]+)\}/gi, function (all, escape, name) {
              if (escape) return '${' + name + '}';
              if (builtins[name] !== undefined) return builtins[name];
              var v = file.vars[name];
              if (v === undefined || v.secret) return '${' + name + '}';
              var value = self.varValue(folder, name);
              return value === undefined ? '${' + name + '}' : fill(value);
            });
          };
          if (cfg.type === 'serve') return 'serve ' + (cfg.root === '.' ? folder.slice(1) : cfg.root) + ' → :' + fill(cfg.port);
          return fill(cfg.command);
        },
        runsOf: function (co, folder) {
          return this.consoles.filter(function (c) { return c.kind === 'run' && c.co === co && c.folder === folder; }).reverse();
        },
        liveRun: function (co, folder, key) {
          return this.consoles.find(function (c) { return c.kind === 'run' && c.co === co && c.folder === folder && c.key === key && c.state !== 'exited'; }) || null;
        },
        // process row status: running, before: <step>, exited <code>, stopped, or why the chain failed.
        runStatus: function (c) {
          if (c.state === 'waiting') return 'before: ' + c.waitingOn;
          if (c.state === 'running') return 'running';
          if (c.failed) return c.failed;
          return c.stopped ? 'stopped' : 'exited ' + c.code;
        },
        // sidebar/view.rs run badge colour: running green; a failed chain or a non-zero exit red; else muted.
        runTone: function (c) {
          if (c.state === 'running') return 'running';
          if (c.failed || (c.state === 'exited' && c.stopped === false && c.code !== 0)) return 'failed';
          return 'muted';
        },
        // Run (or Restart): asks for missing values first, then runs the before steps and the config.
        // where: 'tab': the run's tab shows.
        requestRun: function (co, folder, key, where) {
          var request = { co: co, folder: folder, key: key, where: where };
          var names = this.missingVars(folder, key);
          if (names.length) { this.openVarsTab(request, names); return; }
          this.startChain(request);
        },
        // The missing values are asked in a tab of their own: one per configuration and checkout.
        openVarsTab: function (request, names) {
          var cfg = this.config(request.folder, request.key);
          this.openTab({ key: 'vars:' + request.co + request.folder + '#' + request.key, kind: 'vars', title: 'Run ' + cfg.name, co: request.co,
            project: this.checkout(request.co).project, request: request, names: names, values: {} });
        },
        // Plain values go to vars.toml; secrets to the wallet, with a reference in vars.toml. The tab closes, the run starts.
        submitVars: function (tab) {
          var self = this;
          tab.names.forEach(function (name) {
            var value = tab.values[name] || '';
            if (RUN_FILES[tab.request.folder].vars[name].secret) { self.wallet[name] = value; self.vars[name] = { wallet: name }; }
            else self.vars[name] = value;
          });
          this.closeTab(tab);
          this.startChain(tab.request);
        },
        newRun: function (co, folder, cfg) {
          var c = { id: ++this.consoleCount, co: co, kind: 'run', folder: folder, key: cfg.key, serve: cfg.type === 'serve', title: cfg.name,
            lines: [], input: '', state: 'waiting', waitingOn: null, code: null, stopped: false, failed: null, started: new Date() };
          this.consoles.push(c);
          // Alpine's proxy of c, so the timers' changes re-render
          return this.consoles[this.consoles.length - 1];
        },
        // Starting a configuration again removes its exited run (row and tab); the new run goes on top.
        runFor: function (co, folder, cfg) {
          var self = this;
          this.consoles.filter(function (c) { return c.kind === 'run' && c.co === co && c.folder === folder && c.key === cfg.key && c.state === 'exited'; })
            .forEach(function (c) { self.closeRun(c); });
          return this.newRun(co, folder, cfg);
        },
        // Mock-only timing: a before step ends 1.5 s after it starts; the chain polls for it.
        startChain: function (request) {
          var self = this;
          var co = request.co;
          var folder = request.folder;
          var cfg = this.config(folder, request.key);
          var main = this.runFor(co, folder, cfg);
          this.showRun(main, request.where);
          var steps = cfg.before || [];
          var next = function (i) {
            if (main.state !== 'waiting') return;
            if (i === steps.length) { self.launch(main, cfg, co, folder); return; }
            var step = self.config(folder, steps[i]);
            main.waitingOn = step.name;
            main.lines.push('Waiting for ' + step.name + '…');
            // a step already running (started by hand or by another chain) is waited for, not started again
            var run = self.liveRun(co, folder, step.key);
            if (run === null) { run = self.runFor(co, folder, step); self.launch(run, step, co, folder); }
            var poll = setInterval(function () {
              if (run.state !== 'exited') return;
              clearInterval(poll);
              if (run.code === 0) { next(i + 1); return; }
              main.state = 'exited';
              main.failed = 'before: ' + step.name + ' failed (' + (run.stopped ? 'stopped' : 'exit ' + run.code) + ')';
              main.lines.push(main.failed);
            }, 300);
          };
          next(0);
        },
        launch: function (run, cfg, co, folder) {
          var self = this;
          run.state = 'running';
          run.started = new Date();
          this.useWorktree(co);
          var shown = this.shownCommand(co, folder, cfg);
          run.lines.push(cfg.type === 'serve' ? shown.replace(/^serve (.*) → :(\d+)$/, 'Serving $1 at http://127.0.0.1:$2/') : '$ ' + shown);
          (cfg.output || []).forEach(function (line) { run.lines.push(line); });
          if (cfg.exits === undefined) return;
          setTimeout(function () {
            if (run.state !== 'running') return;
            run.state = 'exited';
            run.code = cfg.exits;
            run.lines.push('', 'Process finished with exit code ' + cfg.exits);
          }, 1500);
        },
        // Stop: SIGTERM to the process group (SIGKILL 5 s later); Kill: SIGKILL at once; a waiting chain just ends.
        stopRun: function (c, name) {
          if (c.state === 'exited') return;
          var killed = name === 'KILL';
          c.state = 'exited';
          c.stopped = true;
          c.code = killed ? 137 : 143;
          c.lines.push('', 'Process finished with exit code ' + c.code + (killed ? ' (killed)' : ' (stopped)'));
        },
        // Restart, and Start of an exited run: stops it; the new run replaces it.
        restartRun: function (c, where) {
          this.stopRun(c);
          this.requestRun(c.co, c.folder, c.key, where);
        },
        closeRun: function (c) { this.closeConsole(c); },
        // The run's tab, with its worktree's Processes shown and unfolded so its row is in sight; started from the run
        // configurations tab, that tab stays.
        showRun: function (c, where) {
          if (where === 'stay') return;
          var folder = 'wf:' + c.co + '/processes';
          this.toggled = this.toggled.filter(function (key) { return key !== folder && key !== 'fs:' + folder; });
          this.openConsole(c);
        },

        // ---- paging and search ----
        matches: function (text, query) {
          var hay = text.toLowerCase();
          return query.toLowerCase().split(/\s+/).filter(Boolean).every(function (word) { return hay.includes(word); });
        },
        isGroupOpen: function (key, open) { return open !== this.expandedGroups.includes(key); },
        toggleGroup: function (key) {
          var at = this.expandedGroups.indexOf(key);
          if (at === -1) this.expandedGroups.push(key); else this.expandedGroups.splice(at, 1);
        },
        // Long lists show PAGE rows, then grow a page at a time or all at once; nothing renders a thousand rows.
        limited: function (list, key) { return list.slice(0, this.shown[key] || PAGE); },
        showMore: function (key) { this.shown[key] = (this.shown[key] || PAGE) + PAGE; },
        showAll: function (key, total) { this.shown[key] = total; },
        // paging.rs more_label
        moreLabel: function (total, shown) { return 'Show ' + Math.min(PAGE, total - shown) + ' more · ' + shown + ' of ' + total; },

        // ---- overviews ----
        openProjectPage: function (project) {
          this.openTab({ key: 'pp:' + project.name, kind: 'projectPage', title: project.name, co: null, project: project.name });
        },
        overviewTab: function (key, fallback) { return this.overviewTabs[key] || fallback; },
        setOverviewTab: function (key, name) { this.overviewTabs[key] = name; },
        // overview/mod.rs: ProjectSection::ALL (Git only with git)
        projectTabs: function (p) {
          return [{ key: 'worktrees', title: 'Worktrees' }, p.git && { key: 'git', title: 'Git' }, { key: 'scratches', title: 'Scratches' }].filter(Boolean);
        },
        // The Worktrees section's rows: the main checkout, the shown worktrees by last use, then the hidden ones; each
        // matching the filter.
        worktreeRows: function (p, query) {
          var self = this;
          var shown = this.byRecency(this.shownCheckouts(p).slice(1).map(function (co) { return co.id; })).map(this.checkout, this);
          return [p.checkouts[0]].concat(shown).filter(function (co) { return self.matches(co.label + ' ' + (co.branch || '') + ' ' + co.path, query); });
        },
        hiddenRows: function (p, query) {
          var self = this;
          return this.hiddenCheckouts(p).filter(function (co) { return self.matches(co.label + ' ' + co.branch + ' ' + co.path, query); });
        },
        branchRows: function (p, query) {
          var self = this;
          return p.branches.filter(function (b) { return self.matches(b.name + ' ' + (b.remote || 'local'), query); });
        },
        // The Worktrees section's groups, as the hub's sections: each row { id, co | branch, open, toggle (null: none) }.
        pageSections: function (p, query) {
          var self = this;
          // toggle: Space on the row; a worktree hides or shows, a branch gets its worktree
          var worktree = function (co) {
            return { id: 'w:' + co.id, co: co, open: function () { self.revealWorktree(co); },
              toggle: co.main ? null : function () { if (self.isHidden(co)) self.showWorktree(co); else self.hideWorktree(co); } };
          };
          var sections = [{ key: 'worktrees', title: 'Worktrees', hint: 'the main checkout, then by last use', rows: this.worktreeRows(p, query).map(worktree),
            empty: 'No worktree matches' }];
          var hidden = this.hiddenRows(p, query);
          if (hidden.length) sections.push({ key: 'hidden', title: 'Hidden', hint: 'not in the tree, the hub or New terminal; their tabs stay open', rows: hidden.map(worktree) });
          if (p.git) {
            var found = this.branchRows(p, query);
            sections.push({ key: 'branches', title: 'Branches without a worktree', hint: 'newest first; Enter adds its worktree in ' + p.worktrees, found: found,
              rows: this.limited(found, 'branches').map(function (b) {
                return { id: 'b:' + b.name, branch: b, open: function () { self.query.branch = ''; self.createWorktree(p, b); },
                  toggle: function () { self.createWorktree(p, b); } };
              }), empty: 'No branch matches' });
          }
          return sections;
        },
        // The groups ↑↓ walk: a collapsed Hidden group's rows are skipped.
        openPageSections: function (p, query) {
          var self = this;
          return this.pageSections(p, query).filter(function (s) { return s.key !== 'hidden' || self.isGroupOpen('hidden:' + p.name, true); });
        },
        pageRows: function (p, query) { return this.openPageSections(p, query).flatMap(function (s) { return s.rows; }); },
        isPageSelected: function (p, row) { return this.pageSelected[p.name] === row.id; },
        selectPageRow: function (p, row) { this.pageSelected[p.name] = row.id; },
        // Space: the row's toggle; the row moves to another group, and the selection stays in place: the row now at the
        // same position in its group, else the row after the group.
        togglePageRow: function (p, row) {
          // rows are built anew on each call: matched by id
          var same = function (r) { return r.id === row.id; };
          var group = this.openPageSections(p, this.query.branch).find(function (s) { return s.rows.some(same); });
          var at = group.rows.findIndex(same);
          row.toggle();
          var sections = this.openPageSections(p, this.query.branch);
          var kept = sections.find(function (s) { return s.key === group.key; });
          if (kept && at < kept.rows.length) { this.selectPageRow(p, kept.rows[at]); return; }
          var order = ['worktrees', 'hidden', 'branches'];
          var through = sections.filter(function (s) { return order.indexOf(s.key) <= order.indexOf(group.key); });
          var rows = sections.flatMap(function (s) { return s.rows; });
          var following = rows[Math.min(through.reduce(function (n, s) { return n + s.rows.length; }, 0), rows.length - 1)];
          if (following) this.selectPageRow(p, following);
        },
        // The filter keeps the keyboard, as the hub's query does: ↑↓ select, Enter opens, Space hides or shows the
        // selected worktree or adds the selected branch's worktree (typing clears the selection, so Space types
        // again), Escape drops the selection, then clears the filter.
        pageKey: function (p, event) {
          var rows = this.pageRows(p, this.query.branch);
          var at = rows.findIndex(function (r) { return this.isPageSelected(p, r); }, this);
          var row = rows[at] || null;
          if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            var next = event.key === 'ArrowDown' ? Math.min(at + 1, rows.length - 1) : Math.max(at - 1, 0);
            if (rows.length) this.selectPageRow(p, rows[next]);
            return;
          }
          if (event.key === 'Enter' && row) { event.preventDefault(); row.open(); return; }
          if (event.key === ' ' && row) { event.preventDefault(); if (row.toggle) this.togglePageRow(p, row); return; }
          if (event.key === 'Escape') { event.stopPropagation(); this.pageEscape(p); }
        },
        pageEscape: function (p) {
          if (this.pageSelected[p.name]) { delete this.pageSelected[p.name]; return; }
          this.query.branch = '';
        },
        pageInput: function (p, value) {
          this.query.branch = value;
          delete this.pageSelected[p.name];
        },
        pageKeys: function (p) {
          return [['↑↓', 'move'], ['Enter', 'show in the tree · add the branch\'s worktree'], ['Space', 'hide or show · add the worktree, staying put'], ['type', 'filter'],
            ['Esc', this.pageSelected[p.name] ? 'back to typing' : 'clear']];
        },
        // When the worktree was last used, as New terminal shows it.
        lastUsedAge: function (co) { var at = this.worktreeLastUsed(co.id); return at === null ? 'never' : this.age(at); },
        // project_git.rs summary
        gitSummary: function (p) {
          return counted(p.log.length, 'recent commit', 'recent commits') + ' · ' + counted(p.stashes.length, 'stash', 'stashes') + ' · ' +
            counted(p.branches.length, 'branch without a worktree', 'branches without a worktree');
        },
        logAge: function (c) { return age(new Date(window.IVE_SCALE.today - c.hours * 3600 * 1000)); },
        stashAge: function (s) { return age(new Date(window.IVE_SCALE.today - s.days * DAY)); },
        openGitLog: function (project) {
          this.openTab({ key: 'git:' + project.name, kind: 'gitLog', title: 'Git log', co: null, project: project.name });
        },
        gitLogSections: function (p) {
          return [{ key: 'log', title: 'Log', count: p.log.length }, { key: 'stashes', title: 'Stashes', count: p.stashes.length },
            { key: 'worktrees', title: 'Worktrees', count: p.checkouts.length }, { key: 'branches', title: 'Branches', count: p.branches.length }];
        },
        openCommit: function (project, c) {
          this.openTab({ key: 'cm:' + project.name + '/' + c.sha, kind: 'commit', title: c.sha + ' ' + c.subject, co: null, project: project.name, sha: c.sha });
        },
        // What a commit tab shows (commit_tab.rs): the header and its files' diffs.
        commitTab: function (tab) {
          var p = this.project(tab.project);
          var log = p.log.find(function (c) { return c.sha === tab.sha; });
          var files = [change('M', 'internal-apps/handoff/README.md', ['@@ -107,6 +107,9 @@ comment points at **Handoff → My queue**', ' included. The panel is the convenient surface,', ' administrator the queue is the *usual* one.', '', '+## An incomplete definition', '+', '+The editor saves what it has.', ' ## Switching a definition off'])];
          return { sha: log.sha + '0da16312fd283f2b9420645ddddf1d4aa'.slice(0, 33), author: log.author, date: '2026-10-16 11:14 +0000', message: log.subject, parents: 'parents 609381f, 56a1e6b', files: files };
        },
        toggleFile: function (key) {
          var at = this.collapsedFiles.indexOf(key);
          if (at === -1) this.collapsedFiles.push(key); else this.collapsedFiles.splice(at, 1);
        },
        openDiff: function (co, x, source) {
          this.openTab({ key: 'df:' + co.id + '/' + source + ':' + x.path, kind: 'diff', title: x.path.split('/').pop(), co: co.id, project: co.project, path: x.path, source: source, mark: x.mark });
        },
        diffOfTab: function (tab) { return this.diffEntry(this.checkout(tab.co), tab.source, tab.path); },

        // ---- a worktree's changes (git/worktree_data.rs) ----
        // Every file the branch touched since its base, commits oldest first, then the working tree.
        branchFiles: function (co) {
          var files = {};
          var add = function (x, uncommitted) {
            var f = files[x.path] || (files[x.path] = { path: x.path, mark: x.mark, diff: [], uncommitted: false, commits: 0 });
            f.diff = f.diff.concat(x.diff);
            if (uncommitted) f.uncommitted = true; else f.commits += 1;
          };
          co.commits.slice().reverse().forEach(function (c) { c.files.forEach(function (x) { add(x, false); }); });
          co.changes.forEach(function (x) { add(x, true); });
          return Object.keys(files).sort().map(function (k) { return files[k]; });
        },
        diffFiles: function (co, source) {
          if (source === 'work') return co.changes;
          if (source === 'branch') return this.branchFiles(co);
          var c = co.commits.find(function (x) { return x.sha === source; });
          return c ? c.files : [];
        },
        diffEntry: function (co, source, path) { return this.diffFiles(co, source).find(function (x) { return x.path === path; }) || null; },
        // diff_tab.rs source_text
        sourceText: function (co, source) {
          if (source === 'work') return 'uncommitted';
          if (source === 'branch') return co.base + ' … ' + co.label + ' + uncommitted';
          return 'commit ' + source + ' · ' + co.commits.find(function (c) { return c.sha === source; }).message;
        }
      }, window.IVE_HUB.state(), window.IVE_HUB.methods, window.IVE_ROUTE.state(), window.IVE_ROUTE.methods, window.IVE_MENUS.state(), window.IVE_MENUS.methods, window.IVE_DND.state(), window.IVE_DND.methods);
    });
  });
})();
