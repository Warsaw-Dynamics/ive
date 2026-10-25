// The menus of tree rows and tabs (workspace/menu.rs, ive.rn's config.tree_menus.main and config.tab_menus.main):
// Send to Terminal ▸, Open in Terminal, Run, Find in Files…, Copy Path ▸, Open In ▸, Git ▸. Items that do not apply
// to what was clicked are left out, then dangling separators and empty submenus. A menu item is { title, enabled, run }
// or a submenu { title, items }; null is a separator.
(function () {
  'use strict';

  var SHELL_FILE = /\.(sh|bash|zsh)$/;
  var WEB_FILE = /\.(html?|svg)$/i;

  // Without items that are not there, separators at either end or doubled, and submenus left empty.
  function pruned(items) {
    var kept = items.filter(function (item) { return item !== false && item !== undefined; }).map(function (item) {
      return item && item.items ? Object.assign({}, item, { items: pruned(item.items) }) : item;
    }).filter(function (item) { return item === null || item.items === undefined || item.items.length > 0; });
    var out = [];
    kept.forEach(function (item) {
      if (item === null && (out.length === 0 || out[out.length - 1] === null)) return;
      out.push(item);
    });
    if (out[out.length - 1] === null) out.pop();
    return out;
  }

  // pasted.rs shell_word: plain words as they are, others single-quoted.
  function shellWord(text) {
    return /^[A-Za-z0-9/._\-+,:@%=]+$/.test(text) ? text : "'" + text.replace(/'/g, "'\\''") + "'";
  }

  window.IVE_MENUS = {
    pruned: pruned,

    state: function () {
      return {
        // the tree row a menu was opened on (menu === 'tree'), where, and which submenu is open (its index, or null)
        treeMenu: { node: null, x: 0, y: 0 },
        menuSub: null,
        // console ids, the most recently used last: shown, or Enter pressed in it (terminal_use.rs)
        consoleUse: []
      };
    },

    methods: {
      // ---- what a menu applies to: { co (checkout id), path ('/a/b', '' for the root), folder } or null ----
      nodeSubject: function (node) { return node.entry || null; },
      tabSubject: function (tab) {
        if (tab.kind === 'file') return { co: tab.co, path: tab.path, folder: false };
        if (tab.kind === 'diff') return { co: tab.co, path: '/' + tab.path, folder: false };
        return null;
      },
      subjectName: function (s) { return s.path === '' ? this.label(s.co) : s.path.split('/').pop(); },
      relativePath: function (s) { return s.path === '' ? '.' : s.path.slice(1); },
      absolutePath: function (s) { return this.checkout(s.co).path + s.path; },

      // A right press on a tree row (sidebar/view.rs ShowTreeMenu): its menu at the pointer.
      openTreeMenu: function (node, x, y) {
        if (this.treeMenuItems(node).length === 0) return;
        this.treeMenu = { node: node, x: x, y: y };
        this.menuSub = null;
        this.menu = 'tree';
        var self = this;
        this.$nextTick(function () { self.keepMenuInWindow(document.querySelector('.menu--at'), self.treeMenu); });
      },
      keepMenuInWindow: function (menu, at) {
        if (menu === null) return;
        var box = menu.getBoundingClientRect();
        at.x = Math.max(0, Math.min(at.x, window.innerWidth - box.width));
        at.y = Math.max(0, Math.min(at.y, window.innerHeight - box.height));
      },
      // The open menu's items and where it is, for either kind.
      contextMenu: function () {
        if (this.menu === 'tab' && this.menuTab()) return { at: this.tabMenu, items: this.tabMenuItems(this.menuTab()) };
        if (this.menu === 'tree' && this.treeMenu.node) return { at: this.treeMenu, items: this.treeMenuItems(this.treeMenu.node) };
        if (this.menu === 'drop') return this.dropMenu;
        return null;
      },
      runMenuItem: function (item) {
        if (item.items) { this.menuSub = item; return; }
        if (item.enabled === false) return;
        this.menu = null;
        this.menuSub = null;
        item.run();
      },

      // ive.rn's config.tree_menus.main
      treeMenuItems: function (node) {
        var s = this.nodeSubject(node);
        if (s === null) return [];
        return pruned([
          this.sendToTerminalItem(s), this.openTerminalItem(s), this.runItem(s), null,
          this.findInFilesItem(s), null,
          this.copyItem(node), this.pasteItem(node), this.renameItem(node), this.deleteItem(node), null,
          this.copyPathItem(s), this.openInItem(s), null,
          this.gitItem(s, true)
        ]);
      },
      // What a file's tab adds to its menu, between Stop and Close (ive.rn's config.tab_menus.main).
      tabFileItems: function (tab) {
        var s = this.tabSubject(tab);
        if (s === null) return [];
        return pruned([this.sendToTerminalItem(s), this.findInFilesItem(s), this.copyPathItem(s), this.openInItem(s), this.gitItem(s, false)]);
      },

      // Send to Terminal ▸: the terminals, most recently used first, then a new one in the worktree.
      sendToTerminalItem: function (s) {
        var self = this;
        var terminals = this.terminalsByUse().map(function (c) {
          return { title: c.title + ' · ' + self.label(c.co), enabled: true, run: function () { self.sendPaths(c, s); } };
        });
        var fresh = { title: 'New Terminal in ' + this.label(s.co), enabled: true, run: function () {
          var c = self.addConsole(s.co, 'shell', []);
          self.sendPaths(c, s);
        } };
        return { title: 'Send to Terminal', items: terminals.concat([null, fresh]) };
      },
      terminalsByUse: function () {
        var use = this.consoleUse;
        var live = this.consoles.filter(function (c) { return c.kind !== 'run'; });
        return live.slice().sort(function (a, b) { return use.indexOf(b.id) - use.indexOf(a.id) || b.id - a.id; });
      },
      usedConsole: function (c) { this.consoleUse = this.consoleUse.filter(function (id) { return id !== c.id; }).concat(c.id); },
      // path_text.rs: relative to the terminal's folder (the mock's terminals stay in their worktree's root) when it
      // works in the path's worktree, else absolute; then the terminal is shown, nothing pressed.
      sendPaths: function (c, s) {
        var text = c.co === s.co ? this.relativePath(s) : this.absolutePath(s);
        c.input = (c.input && c.input.endsWith(' ') === false ? c.input + ' ' : c.input || '') + shellWord(text) + ' ';
        this.openConsole(c);
      },
      openTerminalItem: function (s) {
        var self = this;
        return { title: 'Open in Terminal', enabled: true, run: function () {
          var c = self.addConsole(s.co, 'shell', []);
          var folder = s.folder ? s.path : s.path.slice(0, s.path.lastIndexOf('/'));
          c.title = c.title + folder;
          self.openConsole(c);
        } };
      },
      // Run: a shell file as a shell scratch runs; a folder's .run.toml configurations as a submenu.
      runItem: function (s) {
        var self = this;
        if (s.folder === false && SHELL_FILE.test(s.path)) {
          return { title: 'Run', enabled: true, run: function () {
            var shell = s.path.match(SHELL_FILE)[1];
            var c = self.addConsole(s.co, 'shell', [{ prompt: self.prompt(s.co), command: shell + ' ' + shellWord(self.absolutePath(s)) }, '(mock) ran ' + s.path.slice(1)]);
            self.openConsole(c);
          } };
        }
        var file = s.folder ? this.runFile(s.path) : null;
        if (!file || file.error !== undefined) return false;
        return { title: 'Run', items: file.configs.map(function (cfg) {
          return { title: cfg.name, enabled: true, run: function () { self.requestRun(self.checkout(s.co), s.path, cfg.key, 'tree'); } };
        }) };
      },
      findInFilesItem: function (s) {
        var self = this;
        return { title: 'Find in Files…', enabled: true, run: function () { self.openContentHub([{ co: s.co, path: s.path }]); } };
      },
      copyPathItem: function (s) {
        var self = this;
        var copy = function (text) { return function () { if (navigator.clipboard) navigator.clipboard.writeText(text); }; };
        return { title: 'Copy Path', items: [
          { title: 'Copy Absolute Path', enabled: true, run: copy(this.absolutePath(s)) },
          { title: 'Copy Relative Path', enabled: true, run: copy(this.relativePath(s)) },
          { title: 'Copy File Name', enabled: true, run: copy(this.subjectName(s)) }
        ] };
      },
      // The OS's file manager, default application and browser: nothing happens in the mock.
      openInItem: function (s) {
        var nothing = function () {};
        return { title: 'Open In', items: [
          { title: 'Reveal in Files', enabled: true, run: nothing },
          s.folder === false && { title: 'Open in Default App', enabled: true, run: nothing },
          s.folder === false && WEB_FILE.test(s.path) && { title: 'Open in Browser', enabled: true, run: nothing },
          s.folder === false && { title: 'Open in New Window', enabled: true, run: nothing }
        ] };
      },
      // Git ▸: Show Diff (uncommitted), Compare with Base Branch (since base), Show History, Add to .gitignore (tree).
      gitItem: function (s, withIgnore) {
        var self = this;
        var co = this.checkout(s.co);
        if (this.project(co.project).git === false) return false;
        var relative = s.path.slice(1);
        var uncommitted = s.folder ? undefined : co.changes.find(function (x) { return x.path === relative; });
        var since = s.folder || co.base === undefined || co.base === null ? undefined : this.branchFiles(co).find(function (x) { return x.path === relative; });
        return { title: 'Git', items: [
          uncommitted && { title: 'Show Diff', enabled: true, run: function () { self.openDiff(co, uncommitted, 'work'); } },
          since && { title: 'Compare with Base Branch', enabled: true, run: function () { self.openDiff(co, since, 'branch'); } },
          { title: 'Show History', enabled: true, run: function () { self.openPathHistory(co, s); } },
          withIgnore && s.path !== '' && null,
          withIgnore && s.path !== '' && { title: 'Add to .gitignore', enabled: true, run: function () {} }
        ] };
      },
      // A Git log tab of the commits that changed the path (the mock lists the project's log).
      openPathHistory: function (co, s) {
        this.openTab({ key: 'git:' + co.project + ':' + co.id + s.path, kind: 'gitLog', title: 'History · ' + this.subjectName(s), co: null, project: co.project });
      }
    }
  };
})();
