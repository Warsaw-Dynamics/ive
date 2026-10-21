// IVE mock: the current app as plain HTML + Alpine. Fake data, no file system.
(function () {
  'use strict';

  var ICONS = '../assets/icons/';
  var LINE_HEIGHT = 21; // 14 px font × 1.5, as in editor.rs
  var TREE_PADDING = 8;
  var TREE_INDENT = 20; // icon 16 + gap 4, as in workspace.rs

  var lines = function (count, make) {
    return { lineCount: count, line: make };
  };
  var fixed = function (text) {
    var all = text.split('\n');
    return lines(all.length, function (i) { return all[i]; });
  };

  var SAMPLE = {
    name: 'ive-sample',
    children: [
      { name: 'a', children: [
        { name: 'b', children: [
          { name: 'c', children: [
            { name: 'deep.md', content: fixed('# Deep\n\nFour folders down.') }
          ] },
          { name: 'Dockerfile', content: fixed('FROM debian:13\nCOPY ive /usr/local/bin/\nCMD ["ive"]') }
        ] },
        { name: 'config.toml', content: fixed('[editor]\ntheme = "system"') },
        { name: 'Main.java', content: fixed('class Main {\n    public static void main(String[] args) {\n        System.out.println("IVE");\n    }\n}') }
      ] },
      { name: 'image.png', failed: 'image.png looks like a binary file' },
      { name: 'long-line.txt', content: lines(200, function (i) {
        return i === 149 ? 'x'.repeat(10000) + ' …' : 'short line ' + i;
      }) },
      { name: 'ten-million.log', content: lines(10000000, function (i) {
        return '2026-10-15T12:00:00.000Z INFO  [worker-' + (i % 8) + '] request ' + i + ' handled in ' + (i % 97) + ' ms';
      }) }
    ]
  };

  // Same table as icons.rs file_icon().
  var BY_NAME = { Dockerfile: 'file-docker.svg', Containerfile: 'file-docker.svg', BUILD: 'file-bazel.svg', WORKSPACE: 'file-bazel.svg', 'MODULE.bazel': 'file-bazel.svg' };
  var BY_EXTENSION = {
    java: 'java', js: 'javaScript', mjs: 'javaScript', cjs: 'javaScript', jsx: 'javaScript',
    ts: 'typeScript', mts: 'typeScript', cts: 'typeScript', tsx: 'typeScript', json: 'json',
    xml: 'xml', pom: 'xml', xsd: 'xml', svg: 'xml', yml: 'yaml', yaml: 'yaml', toml: 'toml',
    md: 'markdown', markdown: 'markdown', html: 'html', htm: 'html', jte: 'html', css: 'css', scss: 'css', less: 'css',
    sh: 'shell', bash: 'shell', zsh: 'shell', sql: 'sql', properties: 'properties', env: 'properties',
    png: 'image', jpg: 'image', jpeg: 'image', gif: 'image', webp: 'image', ico: 'image', bmp: 'image',
    zip: 'archive', jar: 'archive', war: 'archive', tar: 'archive', gz: 'archive', tgz: 'archive', '7z': 'archive', rar: 'archive',
    csv: 'csv', tsv: 'csv', ini: 'config', conf: 'config', cfg: 'config', editorconfig: 'config',
    gradle: 'gradle', kts: 'gradle', dockerfile: 'docker', tf: 'terraform', tfvars: 'terraform', hcl: 'terraform',
    graphql: 'graphql', gql: 'graphql', graphqls: 'graphql', groovy: 'groovy', gvy: 'groovy', ipynb: 'jupyter',
    bat: 'microsoftWindows', cmd: 'microsoftWindows', ps1: 'microsoftWindows', exe: 'microsoftWindows', dll: 'microsoftWindows',
    pl: 'perl', pm: 'perl', swift: 'swiftLang', cs: 'Csharp', c: 'c', cpp: 'cpp', cc: 'cpp', cxx: 'cpp', hpp: 'cpp', hh: 'cpp',
    h: 'h', bazel: 'bazel', bzl: 'bazel'
  };

  function withPaths(entry, parent) {
    entry.path = parent + '/' + entry.name;
    (entry.children || []).forEach(function (child) { withPaths(child, entry.path); });
    return entry;
  }

  document.addEventListener('alpine:init', function () {
    var dark = window.matchMedia('(prefers-color-scheme: dark)');

    Alpine.data('ive', function () {
      return {
        project: null,
        welcomeError: '',
        expanded: [],
        selected: null,
        pane: { kind: 'empty' },
        top: 0,
        maximized: false,
        osDark: dark.matches,
        themePreference: 'system',
        themeOptions: [
          { value: 'system', label: 'Same as OS' },
          { value: 'dark', label: 'Dark' },
          { value: 'light', label: 'Light' }
        ],

        init: function () {
          var self = this;
          dark.addEventListener('change', function (event) { self.osDark = event.matches; });
          this.$watch('appearance', function (value) { document.documentElement.dataset.theme = value; });
          document.documentElement.dataset.theme = this.appearance;
          this.openProject();
          this.open(this.findEntry('long-line.txt'));
        },

        get appearance() {
          if (this.themePreference === 'system') return this.osDark ? 'dark' : 'light';
          return this.themePreference;
        },

        icon: function (name) { return ICONS + this.appearance + '/' + name; },

        fileIcon: function (entry) {
          if (entry.children) return 'folder.svg';
          if (BY_NAME[entry.name]) return BY_NAME[entry.name];
          var dot = entry.name.lastIndexOf('.');
          var kind = dot > 0 ? BY_EXTENSION[entry.name.slice(dot + 1).toLowerCase()] : null;
          return 'file-' + (kind || 'text') + '.svg';
        },

        initials: function (name) {
          var words = name.split(/[^\p{L}\p{N}]+/u).filter(Boolean).slice(0, 2);
          return words.map(function (word) { return word[0].toUpperCase(); }).join('') || '?';
        },

        indent: function (depth) { return (TREE_PADDING + depth * TREE_INDENT) + 'px'; },

        rows: function () {
          var self = this;
          var out = [];
          (function walk(entries, depth) {
            entries.forEach(function (entry) {
              out.push({ entry: entry, depth: depth });
              if (entry.children && self.expanded.includes(entry.path)) walk(entry.children, depth + 1);
            });
          })(this.project.children, 0);
          return out;
        },

        findEntry: function (name) {
          return this.project.children.find(function (entry) { return entry.name === name; });
        },

        openProject: function () {
          this.project = withPaths(SAMPLE, '');
          this.expanded = [];
          this.selected = null;
          this.pane = { kind: 'empty' };
          document.title = this.project.name + ' — IVE';
        },

        closeProject: function () {
          this.project = null;
          this.pane = { kind: 'empty' };
          document.title = 'IVE';
        },

        activate: function (entry) {
          this.selected = entry.path;
          if (entry.children) {
            var at = this.expanded.indexOf(entry.path);
            if (at === -1) this.expanded.push(entry.path); else this.expanded.splice(at, 1);
            return;
          }
          this.open(entry);
        },

        open: function (entry) {
          this.selected = entry.path;
          this.top = 0;
          if (entry.failed) {
            this.pane = { kind: 'failed', error: entry.failed };
            return;
          }
          this.pane = { kind: 'document', entry: entry, lineCount: entry.content.lineCount };
        },

        openFailed: function () { this.open(this.findEntry('image.png')); },
        openSettings: function () { this.pane = { kind: 'settings' }; },
        closePane: function () { this.pane = { kind: 'empty' }; },
        setTheme: function (value) { this.themePreference = value; },
        notify: function (what) { console.info('IVE mock: ' + what + ' is handled by the window manager'); },

        viewportLines: function () { return Math.ceil(window.innerHeight / LINE_HEIGHT); },

        maxTop: function () { return Math.max(0, this.pane.lineCount - this.viewportLines() + 2); },

        scroll: function (event) {
          this.top = Math.min(this.maxTop(), Math.max(0, this.top + event.deltaY / LINE_HEIGHT));
        },

        visibleLines: function () {
          var first = Math.floor(this.top);
          var last = Math.min(this.pane.lineCount, first + this.viewportLines());
          var content = this.pane.entry.content;
          var out = [];
          for (var i = first; i < last; i++) out.push({ number: i + 1, text: content.line(i) });
          return out;
        },

        thumbStyle: function () {
          var height = 'max(24px, ' + Math.min(100, this.viewportLines() / this.pane.lineCount * 100) + '%)';
          var at = this.maxTop() === 0 ? 0 : this.top / this.maxTop();
          return { height: height, top: 'calc((100% - ' + height + ') * ' + at + ')' };
        }
      };
    });
  });
})();
