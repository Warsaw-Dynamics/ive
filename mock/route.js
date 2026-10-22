// IVE mock-only: the mock's state in the URL hash, so a refresh (or a shared link) shows the same screen. Every change
// rewrites the hash (replaceState, so no hashchange and no history entries); loading a hash with `t` restores it.
// Older screen links (#tab=…) still work: showScreen applies them, then the hash is rewritten in this form.
//
//   t       a tab, by its key, repeated in order (files, scratches, overviews, git log, commits, diffs, Settings, hub,
//           the two processes the mock starts with); at: the shown tab's key; temp: the temporary one's
//   tree    a tree node whose open state differs from its default, repeated
//   o, g    an overview's section (<tab key>~<section>), a worktree's git view (<checkout>~<view>), repeated
//   rlayout the run configurations tab's layout
//   dl      the diff layout last toggled, side (inline when absent); save, closing: mock-only, read but not written
//   theme, side, page, layout, sort, gap, radius, q, changed, excl, scope, nt: theme, tree width, Settings page, git
//           layout and order, panel gap and radius, hub query, "changed only", "with excluded", scope and whether
//           the hub asks where a new terminal goes (Ctrl+Shift+T)
//
// Not kept: processes started later, runs, created and hidden worktrees and scratches, the hub's history, picks in lists.
(function () {
  'use strict';

  var SEPARATOR = '~';

  function pairs(map) { return Object.keys(map).map(function (k) { return k + SEPARATOR + map[k]; }); }

  function unpair(value) {
    var at = value.lastIndexOf(SEPARATOR);
    return [value.slice(0, at), value.slice(at + 1)];
  }

  // "<checkout id><path>": checkout ids hold no '/', paths start with one.
  function splitCheckout(rest) {
    var at = rest.indexOf('/');
    return at === -1 ? [rest, ''] : [rest.slice(0, at), rest.slice(at)];
  }

  window.IVE_ROUTE = {
    state: function () {
      return { panelGap: null, panelRadius: null };
    },

    methods: {
      routeInit: function () {
        var write = function (hash) { if (location.hash.slice(1) !== hash) history.replaceState(null, '', '#' + hash); };
        write(this.routeHash());
        this.$watch('routeHash()', write);
      },
      routeHash: function () {
        var p = new URLSearchParams();
        var add = function (name, value) { if (value !== null && value !== undefined && value !== '') p.append(name, value); };
        this.tabs.filter(this.isRoutable, this).forEach(function (t) { add('t', t.key); });
        add('at', this.focus);
        add('temp', this.temporary);
        this.toggled.forEach(function (k) { add('tree', k); });
        pairs(this.overviewTabs).forEach(function (v) { add('o', v); });
        add('theme', this.themePreference === 'system' ? null : this.themePreference);
        add('side', this.sidebarWidth === this.SIDEBAR_DEFAULT_WIDTH ? null : this.sidebarWidth);
        add('page', this.settingsPage === 'Appearance' ? null : this.settingsPage);
        add('layout', this.gitLayout === 'flat' ? null : this.gitLayout);
        add('rlayout', this.runLayout === 'flat' ? null : this.runLayout);
        add('gap', this.panelGap);
        add('radius', this.panelRadius);
        add('dl', this.diffLayout === 'inline' ? null : this.diffLayout);
        add('counts', this.countStyle === 'dot' ? null : this.countStyle);
        if (this.tabs.some(function (t) { return t.key === 'hub'; })) {
          add('q', this.hub.query);
          add('excl', this.hub.withExcluded ? '1' : null);
          add('scope', this.hub.scope === 'all' ? null : this.hub.scope);
          add('changed', this.hub.changedOnly ? '1' : null);
          add('nt', this.hub.newTerminal ? '1' : null);
        }
        // keys hold '/', '@' and ':', which are fine in a hash; keep them readable
        return p.toString().replace(/%2F/g, '/').replace(/%40/g, '@').replace(/%3A/g, ':').replace(/%7E/g, '~');
      },
      // Tabs a key brings back: run variable prompts and processes started after load cannot be rebuilt.
      isRoutable: function (tab) { return tab.kind !== 'vars'; },

      isRouteHash: function (p) { return p.has('t') || p.has('at') || p.has('tree'); },
      restoreRoute: function (p) {
        var self = this;
        this.tabs = [];
        this.temporary = null;
        this.focus = null;
        // the hub tab reopens asking where a new terminal goes, before its query comes back
        this.diffLayout = p.get('dl') === 'side' ? 'side' : 'inline';
        var newTerminal = p.get('nt') === '1';
        p.getAll('t').forEach(function (key) {
          if (key === 'hub' && newTerminal) { self.openNewTerminalHub(); self.temporary = null; self.focus = 'hub'; return; }
          self.reopen(key);
          self.temporary = null;
          self.focus = self.tabs.length ? self.tabs[self.tabs.length - 1].key : null;
        });
        if (this.tabs.some(function (t) { return t.key === p.get('at'); })) this.focus = p.get('at');
        // only ever the shown tab is temporary
        if (p.get('temp') === this.focus) this.temporary = this.focus;
        // mock-only, not written back: #save=disk|big|error (&closing=1) puts the shown tab in that save state
        if (p.has('save')) { this.mockSave(p.get('save')); if (p.get('closing') === '1' && this.currentTab()) this.currentTab().closing = true; }
        if (p.has('tree')) this.toggled = p.getAll('tree');
        p.getAll('o').forEach(function (v) { var kv = unpair(v); self.overviewTabs[kv[0]] = kv[1]; });
        if (p.has('q')) this.hub.query = p.get('q');
        this.hub.changedOnly = p.get('changed') === '1';
        this.hub.withExcluded = p.get('excl') === '1';
        this.hub.scope = p.get('scope') || 'all';
      },
      // Opens the tab `key` names, if what it shows still exists.
      reopen: function (key) {
        var colon = key.indexOf(':');
        var kind = colon === -1 ? key : key.slice(0, colon);
        var rest = key.slice(colon + 1);
        var openers = {
          settings: function (self) { self.openSettings(); },
          hub: function (self) { self.openHub(); },
          f: function (self) { var cp = splitCheckout(rest); if (self.hasCheckout(cp[0])) self.openFile(cp[0], cp[1]); },
          s: function (self) { var s = self.scratch(rest); if (s) self.openScratch(s); },
          c: function (self) { var c = self.consoles.find(function (x) { return String(x.id) === rest; }); if (c) self.openConsole(c); },
          rcs: function (self) { if (self.hasCheckout(rest)) self.openRunConfigs(self.checkout(rest)); },
          pp: function (self) { var project = self.project(rest); if (project) self.openProjectPage(project); },
          git: function (self) { var project = self.project(rest); if (project) self.openGitLog(project); },
          cm: function (self) {
            var ps = rest.split('/');
            var project = self.project(ps[0]);
            var commit = project && project.log.find(function (c) { return c.sha === ps[1]; });
            if (commit) self.openCommit(project, commit);
          },
          df: function (self) {
            var cp = splitCheckout(rest);
            var at = cp[1].indexOf(':');
            if (self.hasCheckout(cp[0]) === false || at === -1) return;
            var co = self.checkout(cp[0]);
            var source = cp[1].slice(1, at);
            var x = self.diffEntry(co, source, cp[1].slice(at + 1));
            if (x) self.openDiff(co, x, source);
          }
        };
        if (openers[kind]) openers[kind](this);
      },
      hasCheckout: function (id) {
        return this.projects.some(function (p) { return p.checkouts.some(function (c) { return c.id === id; }); });
      }
    }
  };
})();
