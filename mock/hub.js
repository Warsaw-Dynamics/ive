// IVE mock: the hub (Ctrl+E), a proposal not in the app yet. One temporary tab for everything recent and for finding
// any file in all open projects: recent items (whatever can be a tab, open or closed), uncommitted changes in every
// worktree, live processes; typing searches the projects' files with fzf's matching and extended syntax. Driven by
// keys. checkouts.js mixes `methods` and `state()` into its Alpine component.
(function () {
  'use strict';

  var DAY = 24 * 3600 * 1000;
  // Rows rendered per section: fzf shows what fits; the counter tells how many more matched.
  var RENDERED = 80;
  // hub/content.rs: lines listed at most; a preview starts this many lines above its line (preview.rs PREVIEW_LEAD)
  var CONTENT_LIMIT = 2000;
  var CONTENT_LEAD = 8;
  var RECENT = 12;
  var RECENT_WHILE_SEARCHING = 5;
  // New terminal: a project's newest branches without a worktree listed before typing; the best matches while typing
  var NEW_TERMINAL_BRANCHES = 5;
  var NEW_TERMINAL_MATCHES = 20;
  // fzf's algo.go constants (scoreMatch, scoreGapStart, scoreGapExtension, bonus*), path scheme: '/' is a boundary.
  var SCORE_MATCH = 16;
  var GAP_START = -3;
  var GAP_EXTENSION = -1;
  var BONUS_BOUNDARY = 8;
  var BONUS_SLASH = 10;
  var BONUS_CAMEL = 7;
  var BONUS_CONSECUTIVE = 4;
  var FIRST_CHAR_MULTIPLIER = 2;
  // Not fzf: a term matched inside the file name beats one spread over folders ("theme" → theme.rs first).
  var BONUS_FILE_NAME = 12;

  // { project, path, lower, nameAt, excluded, checkouts }: kept outside Alpine, whose proxies would make a 22k-entry
  // scan slow. excluded: null, 'excluded' (below a folder named in config.excluded) or 'ignored' (gitignored);
  // checkouts: the checkout ids that have the path, null for all of the project's.
  var corpus = [];
  // config.excluded in ive.rn: folder names drawn as excluded, besides gitignored ones
  var EXCLUDED_NAMES = ['target', 'node_modules'];
  // per project, how many excluded files there are (the mock holds a sample of them)
  var excludedTotals = {};
  // the ignored sample was listed in this worktree; ignored files are local to a checkout
  var IGNORED_CHECKOUT = 'atlassian@GEN-817-ive';
  var NOTES = ['2026/october.md', 'todo.md'];
  var searchMemo = { key: null, value: null };

  function load(done) {
    fetch('data/files.gz').then(function (r) { return r.arrayBuffer(); }).then(function (buffer) {
      var bytes = new Uint8Array(buffer);
      // a server may already have decoded it (Content-Encoding); gzip starts with 1f 8b
      if (bytes[0] !== 0x1f || bytes[1] !== 0x8b) return new TextDecoder().decode(bytes);
      return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).text();
    }).then(function (text) {
      parse(text);
      NOTES.forEach(function (path) { corpus.push(entry('notes', path)); });
      done(corpus.filter(function (e) { return e.excluded === null; }).length);
    });
  }

  // tools/hub-files.sh: "#project" or "#project excluded <total>", then front-coded paths "<shared prefix length> <rest>".
  function parse(text) {
    var project = null;
    var ignored = false;
    var previous = '';
    text.split('\n').forEach(function (line) {
      if (line === '') return;
      if (line[0] === '#') {
        var head = line.slice(1).split(' ');
        project = head[0];
        ignored = head[1] === 'excluded';
        if (ignored) excludedTotals[project] = Number(head[2]);
        previous = '';
        return;
      }
      var space = line.indexOf(' ');
      var path = previous.slice(0, Number(line.slice(0, space))) + line.slice(space + 1);
      var e = entry(project, path);
      if (ignored) { e.excluded = e.excluded || 'ignored'; e.checkouts = [IGNORED_CHECKOUT]; }
      corpus.push(e);
      previous = path;
    });
  }

  function entry(project, path) {
    var named = path.split('/').slice(0, -1).some(function (folder) { return EXCLUDED_NAMES.includes(folder); });
    return { project: project, path: path, lower: path.toLowerCase(), nameAt: path.lastIndexOf('/') + 1, excluded: named ? 'excluded' : null, checkouts: null };
  }

  // ---- fzf extended search: space = AND, | = OR, 'exact, ^prefix, suffix$, !not; smart case ----
  function parseQuery(query) {
    var groups = [];
    var joinNext = false;
    query.trim().split(/\s+/).filter(Boolean).forEach(function (token) {
      if (token === '|') { joinNext = groups.length > 0; return; }
      var term = parseTerm(token);
      if (joinNext) groups[groups.length - 1].push(term); else groups.push([term]);
      joinNext = false;
    });
    return groups;
  }

  function parseTerm(token) {
    var term = { kind: 'fuzzy', inverse: false, text: token };
    if (term.text[0] === '!') { term.inverse = true; term.kind = 'exact'; term.text = term.text.slice(1); }
    if (term.text[0] === '\'') { term.kind = 'exact'; term.text = term.text.slice(1); }
    if (term.text[0] === '^') { term.kind = 'prefix'; term.text = term.text.slice(1); }
    if (term.text.length > 1 && term.text[term.text.length - 1] === '$') {
      term.kind = term.kind === 'prefix' ? 'equal' : 'suffix';
      term.text = term.text.slice(0, -1);
    }
    term.caseSensitive = /[A-Z]/.test(term.text);
    if (term.caseSensitive === false) term.text = term.text.toLowerCase();
    return term;
  }

  function bonusAt(text, i) {
    if (i === 0) return BONUS_SLASH;
    var prev = text[i - 1];
    var c = text[i];
    if (prev === '/') return BONUS_SLASH;
    if ('-_. '.includes(prev)) return BONUS_BOUNDARY;
    if (/[a-z0-9]/.test(prev) && /[A-Z]/.test(c)) return BONUS_CAMEL;
    return 0;
  }

  // fzf v1: the leftmost match, its start pulled right as far as the pattern allows, scored along the window.
  function fuzzy(text, hay, pattern) {
    var p = 0;
    var start = -1;
    var end = -1;
    for (var i = 0; i < hay.length && end === -1; i++) {
      if (hay[i] !== pattern[p]) continue;
      if (start === -1) start = i;
      if (++p === pattern.length) end = i;
    }
    if (end === -1) return null;
    p = pattern.length - 1;
    for (var j = end; j >= start; j--) {
      if (hay[j] === pattern[p] && --p < 0) { start = j; break; }
    }
    return scoreWindow(text, hay, pattern, start, end);
  }

  function scoreWindow(text, hay, pattern, start, end) {
    var score = 0;
    var positions = [];
    var p = 0;
    var run = 0;
    var gap = false;
    var firstBonus = 0;
    for (var i = start; i <= end; i++) {
      if (p < pattern.length && hay[i] === pattern[p]) {
        var bonus = bonusAt(text, i);
        if (run === 0) firstBonus = bonus; else bonus = Math.max(bonus, firstBonus, BONUS_CONSECUTIVE);
        score += SCORE_MATCH + (p === 0 ? bonus * FIRST_CHAR_MULTIPLIER : bonus);
        positions.push(i);
        p++;
        run++;
        gap = false;
      } else {
        score += gap ? GAP_EXTENSION : GAP_START;
        gap = true;
        run = 0;
      }
    }
    return { score: score, positions: positions };
  }

  function exact(text, hay, term) {
    var at;
    if (term.kind === 'prefix') at = hay.startsWith(term.text) ? 0 : -1;
    else if (term.kind === 'suffix') at = hay.endsWith(term.text) ? hay.length - term.text.length : -1;
    else if (term.kind === 'equal') at = hay === term.text ? 0 : -1;
    else at = hay.indexOf(term.text);
    if (at === -1) return null;
    return scoreWindow(text, hay, term.text, at, at + term.text.length - 1);
  }

  function matchTerm(text, lower, term) {
    var hay = term.caseSensitive ? text : lower;
    if (term.text === '') return { score: 0, positions: [] };
    var hit = term.kind === 'fuzzy' ? fuzzy(text, hay, term.text) : exact(text, hay, term);
    if (term.inverse) return hit === null ? { score: 0, positions: [] } : null;
    return hit;
  }

  // The match of every group (best of its alternatives), or null; positions are sorted, for highlighting.
  function match(text, lower, groups, nameAt) {
    var score = 0;
    var positions = [];
    for (var g = 0; g < groups.length; g++) {
      var best = null;
      for (var t = 0; t < groups[g].length; t++) {
        var hit = matchTerm(text, lower, groups[g][t]);
        if (hit && nameAt !== undefined && hit.positions.length && hit.positions[0] >= nameAt) hit.score += BONUS_FILE_NAME;
        if (hit && (best === null || hit.score > best.score)) best = hit;
      }
      if (best === null) return null;
      score += best.score;
      positions = positions.concat(best.positions);
    }
    return { score: score, positions: positions.sort(function (a, b) { return a - b; }) };
  }

  // Runs of matched and unmatched characters of text[from, to), for the template.
  function spans(text, positions, from, to) {
    var out = [];
    var hits = new Set(positions);
    for (var i = from; i < to; i++) {
      var on = hits.has(i);
      if (out.length && out[out.length - 1].on === on) out[out.length - 1].t += text[i];
      else out.push({ t: text[i], on: on });
    }
    return out;
  }

  // `file.rs:12` opens at line 12: the line is cut off before matching.
  function splitLine(query) {
    var m = /^(.*\S):(\d+)$/.exec(query.trim());
    return m ? { query: m[1], line: Number(m[2]) } : { query: query, line: null };
  }

  function placeholderLines(path) {
    var name = path.split('/').pop();
    var ext = name.split('.').pop();
    if (ext === 'java') {
      var pkg = (/src\/(?:main|test)\/java\/(.*)\//.exec(path) || [null, 'com/example'])[1].replace(/\//g, '.');
      return ['package ' + pkg + ';', '', 'import java.util.List;', '', 'public final class ' + name.replace('.java', '') + ' {', '', '    // (mock: the hub searches real paths; contents are made up)', '', '}'];
    }
    if (ext === 'md') return ['# ' + name.replace('.md', ''), '', '(mock: the hub searches real paths; contents are made up)'];
    return ['// ' + path, '', '// (mock: the hub searches real paths; contents are made up)'];
  }

  // The mock's clock: the fixed "today" of the fake data, moving on with real time since the page loaded, so what is
  // used now sorts above what was used a minute ago.
  var loaded = Date.now();
  function now() { return new Date(window.IVE_SCALE.today.getTime() + Date.now() - loaded); }

  function textOf(line) { return typeof line === 'object' ? line.prompt.user + ':' + line.prompt.dir + '$ ' + line.command : line; }

  window.IVE_HUB = {
    placeholderLines: placeholderLines,

    state: function () {
      return {
        // query; selected row; scope: 'all', 'p:<project>' or 'w:<checkout id>' (Tab cycles, : picks any); withExcluded: Alt+I, search excluded and
        // gitignored files too; changedOnly: Ctrl+E again, as IntelliJ's
        // "show changed only"; back: the tab Escape returns to; pick: the worktree question after Enter on a row that
        // is in several ({ item, behind, query, selected, back (the row to return to) }), else null; newTerminal:
        // opened with Ctrl+Shift+T, the hub only asks where a new terminal goes.
        // content: Ctrl+Shift+F, the query searches the files' lines (regex: Alt+R, word: Alt+W); paths: the files and
        // folders Find in Files was picked on ({ co, path }), else the scope.
        hub: { query: '', selected: 0, scope: 'all', withExcluded: false, changedOnly: false, back: null, pick: null, loaded: 0, note: '', newTerminal: false,
          content: false, regex: false, word: false, paths: [] },
        // Every tab shown, newest last: { key, tab, at }. Closed tabs stay, so the hub reopens them.
        history: [],
        // Per checkout id, the last input typed into one of its terminals or Claude Code, or run started in it.
        worktreeUse: {}
      };
    },

    methods: {
      hubInit: function () {
        var self = this;
        load(function (count) { self.hub.loaded = count; });
      },
      remember: function (tab) {
        if (tab === null || tab.kind === 'hub') return;
        this.history = this.history.filter(function (h) { return h.key !== tab.key; }).concat({ key: tab.key, tab: tab, at: now() });
      },
      // Mock-only: a few days of history, some of it closed tabs.
      seedHistory: function () {
        var self = this;
        var atl = this.project('atlassian');
        var ive = this.checkout('atlassian@GEN-817-ive');
        var dev = this.checkout('atlassian@dev');
        var at = function (days, hours) { return new Date(window.IVE_SCALE.today - days * DAY - hours * 3600 * 1000); };
        var file = function (co, path) { return { key: 'f:' + co + path, kind: 'file', title: path.split('/').pop(), co: co, project: co.split('@')[0], path: path }; };
        var seeds = [
          [0, 0.4, { key: 's:idp-users.sql', kind: 'scratch', title: 'idp-users.sql', co: null, project: 'atlassian' }],
          [0, 0.9, file(dev.id, '/internal-apps/communication/idp/src/main/java/ovh/atlasinc/idp/api/grants/ApiTokenService.java')],
          [0, 2, { key: 'cm:atlassian/' + atl.log[1].sha, kind: 'commit', title: atl.log[1].sha + ' ' + atl.log[1].subject, co: null, project: 'atlassian', sha: atl.log[1].sha }],
          [1, 1, { key: 'df:' + ive.id + '/work:' + ive.changes[3].path, kind: 'diff', title: 'theme.rs', co: ive.id, project: 'atlassian', path: ive.changes[3].path, source: 'work', mark: 'M' }],
          [1, 4, file(ive.id, '/internal-apps/ive/src/workspace/tab_menu.rs')],
          [2, 0, { key: 'git:atlassian', kind: 'gitLog', title: 'Git log', co: null, project: 'atlassian' }],
          [3, 0, file('forge-installer@main', '/lib/installer.ex')],
          [3, 2, file('atlassian@IDP-30-Reserved-subdomain-returns-404', '/internal-apps/communication/idp/src/main/resources/jte/error.jte')],
          [4, 0, { key: 'settings', kind: 'settings', title: 'Settings', co: null, project: null }],
          [5, 0, file('notes@', '/todo.md')]
        ];
        // restored tabs were shown in the last few minutes, left to right
        this.tabs.forEach(function (t, i) { seeds.unshift([0, (self.tabs.length - i) / 20, t]); });
        var older = seeds.reverse().map(function (s) { return { key: s[2].key, tab: s[2], at: at(s[0], s[1]) }; });
        var keys = new Set(older.map(function (h) { return h.key; }));
        this.history = older.concat(this.history.filter(function (h) { return keys.has(h.key) === false; }));
      },

      // Ctrl+E: opens the hub (temporary); in the hub, toggles "changed only".
      openHub: function () {
        var current = this.currentTab();
        if (current && current.kind === 'hub' && this.hub.newTerminal === false) { this.hub.changedOnly = !this.hub.changedOnly; this.hub.selected = 0; return; }
        this.showHub(false);
      },
      // Ctrl+Shift+T: the hub asking where a new terminal goes.
      openNewTerminalHub: function () { this.showHub(true); },
      showHub: function (newTerminal) {
        var current = this.currentTab();
        if (current === null || current.kind !== 'hub') this.hub.back = current ? current.key : null;
        this.hub.query = '';
        this.hub.selected = 0;
        this.hub.changedOnly = false;
        this.hub.pick = null;
        this.hub.newTerminal = newTerminal;
        this.hub.content = false;
        this.hub.paths = [];
        this.menu = null;
        var title = newTerminal ? 'New terminal' : 'Hub';
        var open = this.tabs.find(function (t) { return t.key === 'hub'; });
        if (open) open.title = title;
        this.openTab({ key: 'hub', kind: 'hub', title: title, co: null, project: null });
      },
      // Find in Files (Ctrl+Shift+F, or a menu's Find in Files… on `paths`): the hub searching the files' lines.
      openContentHub: function (paths) {
        this.showHub(false);
        this.hub.content = true;
        this.hub.paths = paths || [];
      },
      // The window's keys the hub adds: Ctrl+Shift+T, Ctrl+Shift+F (Firefox keeps Ctrl+Shift+T for itself, so the mock
      // takes Alt+Shift+T too).
      newTerminalKey: function (event) {
        var modifier = event.ctrlKey || event.altKey;
        if (modifier && event.shiftKey && event.code === 'KeyT' && event.metaKey === false) {
          event.preventDefault();
          this.openNewTerminalHub();
        }
        var current = this.currentTab();
        if (event.ctrlKey && event.shiftKey && event.code === 'KeyF' && (current === null || current.kind !== 'hub')) {
          event.preventDefault();
          this.openContentHub([]);
        }
      },
      // Escape: back to the tab shown before; the hub, temporary, goes.
      leaveHub: function () {
        var back = this.hub.back;
        var tab = this.tabs.find(function (t) { return t.key === back; });
        if (tab) { this.showTab(tab); return; }
        var hub = this.tabs.find(function (t) { return t.key === 'hub'; });
        if (hub) this.closeTab(hub);
      },
      hubProject: function () {
        var back = this.hub.back;
        var tab = this.tabs.find(function (t) { return t.key === back; });
        return tab && tab.project ? tab.project : 'atlassian';
      },
      // ---- scope: all projects, one project (p:<name>) or one worktree (w:<checkout id>) ----
      // The scope as set, or 'all' when what it names is no longer open or out of the tree (its header shows one project).
      hubScope: function () {
        var scope = this.hub.scope;
        var shown = this.treeProjectNames();
        if (scope.startsWith('p:') && shown.includes(scope.slice(2))) return scope;
        if (scope.startsWith('w:') && this.hasCheckout(scope.slice(2)) && shown.includes(scope.slice(2).split('@')[0])) return scope;
        return 'all';
      },
      hubScopeProject: function () {
        var scope = this.hubScope();
        if (scope === 'all') return null;
        return scope.startsWith('p:') ? scope.slice(2) : scope.slice(2).split('@')[0];
      },
      hubScopeCheckout: function () { var scope = this.hubScope(); return scope.startsWith('w:') ? scope.slice(2) : null; },
      // Tab's stops: all, the back tab's project, its worktree (the tab's own, else the most recently used), and a
      // scope picked with : that is none of these.
      hubScopeStops: function () {
        var back = this.tabs.find(function (t) { return t.key === this.hub.back; }, this);
        var project = back && back.project && this.treeProjectNames().includes(back.project) ? back.project : null;
        var stops = ['all'];
        // with one project in the tree, all is that project
        if (project && project !== this.onlyShown()) stops.push('p:' + project);
        if (project) stops.push('w:' + (back.co || this.byRecency(this.checkoutIds(project))[0]));
        var scope = this.hubScope();
        if (stops.includes(scope) === false) stops.push(scope);
        return stops;
      },
      hubScopeLabel: function (scope) {
        if (scope === 'all') return this.allLabel();
        return scope.startsWith('p:') ? scope.slice(2) : this.label(scope.slice(2));
      },
      // Tab: the next stop; from a picked scope (the last stop) back to all.
      hubCycleScope: function () {
        var stops = this.hubScopeStops();
        var at = stops.indexOf(this.hubScope());
        this.setHubScope(stops[(at + 1) % stops.length]);
      },
      setHubScope: function (scope) {
        this.hub.scope = scope;
        this.hub.selected = 0;
      },
      // A row in the scope: any in all or a project's; in a worktree's, that worktree's rows and the project's own
      // (scratches, commits, Git log), which belong to no worktree.
      hubInScope: function (item) {
        if (item.project && this.treeProjectNames().includes(item.project) === false) return false;
        var co = this.hubScopeCheckout();
        if (co === null) return true;
        return item.co === co || (item.co === null && item.project === this.hubScopeProject());
      },
      hubScoped: function (items) { return items.filter(this.hubInScope, this); },
      useWorktree: function (id) { this.worktreeUse[id] = now(); },
      // A worktree's recency: the last time it was used: one of its tabs shown (file, process, run output, overview,
      // diff; from the history of shown tabs), input typed into one of its terminals, or a run started in it; null
      // when never.
      worktreeLastUsed: function (id) {
        var used = this.worktreeUse[id] || null;
        return this.history.reduce(function (last, h) { return h.tab.co === id && (last === null || h.at > last) ? h.at : last; }, used);
      },
      // The ids of `ids`, most recently shown first; never shown ones after, in the project's order.
      byRecency: function (ids) {
        var self = this;
        var shown = function (id) { var at = self.worktreeLastUsed(id); return at === null ? -1 : at.getTime(); };
        return ids.slice().sort(function (a, b) { return shown(b) - shown(a) || ids.indexOf(a) - ids.indexOf(b); });
      },
      // Hidden worktrees are left out: the hub and New terminal never list them.
      checkoutIds: function (projectName) { return this.shownCheckouts(this.project(projectName)).map(function (c) { return c.id; }); },
      // Where a row opens without asking: its only worktree, else the most recently shown one.
      hubFirstChoice: function (item) { return item.co || (item.choices ? this.byRecency(item.choices)[0] : null); },
      hubMode: function () {
        if (this.hub.pick !== null) return 'pick';
        if (this.hub.newTerminal) return 'terminal';
        if (this.hub.content) return 'content';
        var q = this.hub.query.trimStart();
        if (q[0] === '>') return 'actions';
        if (q[0] === '@') return 'processes';
        if (q[0] === '#') return 'branches';
        // ":42" is a line of the tab the hub came from, as VS Code's Ctrl+P; any other ":" text picks a scope
        if (/^:\d+$/.test(q)) return 'line';
        if (q[0] === ':') return 'scope';
        return this.hub.changedOnly ? 'changed' : 'files';
      },
      hubText: function () {
        var mode = this.hubMode();
        if (mode === 'pick') return this.hub.pick.query;
        if (mode === 'scope') return this.hub.query.trimStart().slice(1);
        if (mode === 'line') return '';
        if (mode === 'content') return '';
        return mode === 'files' || mode === 'changed' || mode === 'terminal' ? this.hub.query : this.hub.query.trimStart().slice(1);
      },
      // The query box edits the search, or the worktree filter while asking.
      hubInputValue: function () { return this.hub.pick === null ? this.hub.query : this.hub.pick.query; },
      hubInput: function (value) {
        if (this.hub.pick === null) { this.hub.query = value; this.hub.selected = 0; return; }
        this.hub.pick.query = value;
        this.hub.pick.selected = 0;
      },

      // ---- what the hub lists ----
      // { sections: [{ key, title, hint, items, more }], items (flat, in order), matched, total, ms }
      hubView: function () {
        var mode = this.hubMode();
        var split = splitLine(this.hubText());
        var groups = parseQuery(split.query);
        var sections = this.identGroupSections(this['hub_' + mode](groups, split));
        var items = [];
        sections.forEach(function (s) { s.items.forEach(function (item) { item.index = items.length; items.push(item); }); });
        var stats = searchMemo.value && mode === 'files' && groups.length ? searchMemo.value : null;
        return { mode: mode, line: split.line, sections: sections.filter(function (s) { return s.items.length || s.empty; }), items: items, stats: stats };
      },
      hub_files: function (groups, split) {
        if (groups.length === 0) {
          return [
            { key: 'waiting', title: 'Waiting', hint: 'processes that called, longest waiting first', items: this.hubScoped(this.hubWaiting()) },
            { key: 'recent', title: 'Recent', hint: 'shown tabs, newest first; closed ones reopen', items: this.hubScoped(this.hubRecent([])).slice(0, RECENT) },
            { key: 'changed', title: 'Changed', hint: this.hubScopeCheckout() ? 'uncommitted' : 'uncommitted, every worktree', items: this.hubScoped(this.hubChanged([])) },
            { key: 'running', title: 'Running', hint: 'processes and runs', items: this.hubScoped(this.hubRunning()) }
          ];
        }
        var recent = this.hubScoped(this.hubRecent(groups, split.line)).slice(0, RECENT_WHILE_SEARCHING);
        var listed = new Set(recent.map(function (item) { return item.pathKey; }));
        var found = this.hubSearch(groups, split);
        return [
          { key: 'recent', title: 'Recent', items: recent },
          { key: 'files', title: 'Files', hint: this.hubScopeLabel(this.hubScope()),
            items: found.items.filter(function (item) { return listed.has(item.pathKey) === false; }), more: found.more,
            empty: this.hub.loaded === 0 ? 'Loading the file lists…' : 'No file matches' }
        ];
      },
      hub_changed: function (groups) {
        return [{ key: 'changed', title: 'Changed only', hint: 'Ctrl+E for everything', items: this.hubScoped(this.hubChanged(groups)), empty: 'Nothing uncommitted' }];
      },
      // Contents (hub/content.rs): one row per matching line, `path:line` and the line with its matches; plain text from
      // 2 characters, smart case, Alt+R regex, Alt+W whole words; the list stops at CONTENT_LIMIT lines.
      hub_content: function () {
        var self = this;
        var paths = this.hub.paths;
        var title = paths.length === 0 ? 'Contents' : 'Contents of ' + this.subjectName(paths[0]) + (paths.length > 1 ? ' +' + (paths.length - 1) : '');
        var hint = paths.length ? 'Backspace on an empty query searches the scope' : this.hubScopeLabel(this.hubScope());
        var section = function (items, empty, more) { return [{ key: 'content', title: title, hint: hint, items: items, empty: empty, more: more, moreText: 'lines; the first ' + CONTENT_LIMIT + ' are listed' }]; };
        if (this.hub.query.length < 2) return section([], "Type 2 characters or more to search the files' contents");
        var regex = this.contentRegex();
        if (typeof regex === 'string') return section([], regex);
        var items = [];
        var total = 0;
        this.contentFiles().forEach(function (file) {
          file.text.split('\n').forEach(function (line, at) {
            regex.lastIndex = 0;
            var spans = [];
            var from = 0;
            var m;
            while ((m = regex.exec(line)) !== null && m[0] !== '') {
              spans.push({ t: line.slice(from, m.index), on: false }, { t: m[0], on: true });
              from = m.index + m[0].length;
            }
            if (spans.length === 0) return;
            total += 1;
            if (items.length >= CONTENT_LIMIT) return;
            spans.push({ t: line.slice(from), on: false });
            items.push(self.contentRow(file, at + 1, spans));
          });
        });
        return section(items, 'No line matches', Math.max(0, total - items.length));
      },
      contentRow: function (file, line, spans) {
        var self = this;
        var lines = file.text.split('\n');
        var first = Math.max(1, line - CONTENT_LEAD);
        return this.hubItem({
          id: 'ct:' + file.key + ':' + line, icon: this.fileIcon(file.name), title: file.title + ':' + line, detail: spans.map(function (s) { return s.t; }).join(''),
          detailSpans: spans, co: file.co, path: file.co ? file.path.slice(1) : undefined,
          open: function () {
            file.open();
            var tab = self.currentTab();
            if (tab) self.goToLine(tab, line);
          },
          preview: function () { return { title: file.title, sub: file.sub, lines: lines.slice(first - 1), first: first, line: line, numbered: true }; }
        });
      },
      // The query as a RegExp (smart case: a capital of its own, not a regex escape's, makes it exact), or why it is not one.
      contentRegex: function () {
        var text = this.hub.query;
        var source = this.hub.regex ? text : text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        if (this.hub.word) source = '\\b(?:' + source + ')\\b';
        var own = this.hub.regex ? text.replace(/\\./g, '') : text;
        try {
          return new RegExp(source, own !== own.toLowerCase() ? 'g' : 'gi');
        } catch (error) {
          return JSON.stringify(text) + ' is not a regex: ' + error.message;
        }
      },
      // The files searched: those under the picked paths, else the scope's worktrees and, for a project or all, its
      // scratches. { key, co, path, name, title, sub, text, open }
      contentFiles: function () {
        var self = this;
        var out = [];
        var picked = this.hub.paths;
        var inPicked = function (co, path) {
          return picked.length === 0 || picked.some(function (p) { return p.co === co.id && (p.path === '' || path === p.path || path.startsWith(p.path + '/')); });
        };
        var scoped = this.openProjects().flatMap(function (p) { return self.shownCheckouts(p); }).filter(function (co) {
          return picked.length ? picked.some(function (p) { return p.co === co.id; }) : self.hubInScope({ co: co.id, project: co.project });
        });
        scoped.forEach(function (co) {
          (function walk(entries) {
            entries.forEach(function (e) {
              if (e.children) { walk(e.children); return; }
              if (typeof e.text !== 'string' || inPicked(co, e.path) === false) return;
              out.push({ key: co.id + e.path, co: co.id, path: e.path, name: e.name, title: e.path.slice(1), sub: self.label(co.id), text: e.text,
                open: function () { self.openFile(co.id, e.path); } });
            });
          })(co.tree);
        });
        if (picked.length === 0 && this.hubScopeCheckout() === null) {
          this.scratches.filter(function (s) { return self.hubInScope({ co: null, project: s.project }); }).forEach(function (s) {
            out.push({ key: 's:' + s.name, co: null, path: s.name, name: s.name, title: 'scratches/' + s.name, sub: 'scratch · ' + s.project, text: s.text,
              open: function () { self.openScratch(s); } });
          });
        }
        return out;
      },
      hub_actions: function (groups) {
        return [{ key: 'actions', title: 'Actions', items: this.hubMatching(this.hubActions(), groups), empty: 'No action matches' }];
      },
      hub_processes: function (groups) {
        return [
          { key: 'running', title: 'Processes', items: this.hubMatching(this.hubProcesses(), groups) },
          { key: 'configs', title: 'Run configurations', hint: 'Enter starts, or shows the live run', items: this.hubMatching(this.hubConfigs(), groups) }
        ];
      },
      // : — the open projects with their worktrees under them, most recently used first; Enter or Tab makes one the
      // scope.
      hub_scope: function (groups) {
        var self = this;
        var running = function (ids) { return self.consoles.filter(function (c) { return ids.includes(c.co) && self.isLive(c); }).length; };
        var items = [this.hubItem({ id: 'sc:all', icon: 'project-folder.svg', title: this.hubScopeLabel('all'), detail: this.allProjectsShown() ? 'every open project' : 'the projects the tree shows', scope: 'all' })];
        this.treeProjects().forEach(function (p) {
          var ids = self.checkoutIds(p.name);
          var n = running(ids);
          // with one project in the tree, the first row (all) is that project; its worktrees follow
          if (p.name !== self.onlyShown()) items.push(self.hubItem({ id: 'sc:p:' + p.name, icon: 'project-folder.svg', title: p.name, detail: ids.length > 1 ? ids.length + ' worktrees' : (p.git ? 'one checkout' : 'no git'),
            state: n ? '● ' + n + ' running' : null, scope: 'p:' + p.name }));
          if (ids.length < 2) return;
          self.byRecency(ids).forEach(function (id) {
            var co = self.checkout(id);
            var used = self.worktreeLastUsed(id);
            var k = running([id]);
            items.push(self.hubItem({ id: 'sc:w:' + id, icon: 'branch.svg', title: co.label, detail: p.name + (co.branch && co.branch !== co.label ? ' · ' + co.branch : ''),
              indent: true, age: used === null ? 'never' : self.age(used), state: k ? '● ' + k + ' running' : null, scope: 'w:' + id }));
          });
        });
        items.forEach(function (item) {
          item.open = function () { self.setHubScope(item.scope); self.hub.query = ''; };
          item.preview = function () { return { title: self.hubScopeLabel(item.scope), sub: 'search scope', lines: ['Enter or Tab searches ' + (item.scope === 'all' ? 'all open projects' : item.title) + '.'] }; };
        });
        return [{ key: 'scope', title: 'Search in', hint: 'projects, their worktrees most recently used first', items: this.hubMatching(items, groups), empty: 'No project or worktree matches' }];
      },
      // Ctrl+Shift+T — where a new terminal goes: each open project (its main checkout), its worktrees most recently
      // used first, then its newest branches without a worktree; typing ranks all of them fzf-style, the best first.
      // A remote branch whose name a local one has is left out (the worktree would check out the local one). Enter on
      // a branch adds its worktree first.
      hub_terminal: function (groups) {
        var items = this.newTerminalTargets();
        var counter = function (n) { return n.toLocaleString('en') + '/' + items.length.toLocaleString('en'); };
        if (groups.length) {
          var found = this.hubMatching(items, groups);
          return [{ key: 'terminal', title: 'New terminal in', hint: 'best matches first', items: found.slice(0, NEW_TERMINAL_MATCHES), counter: counter(found.length),
            more: Math.max(0, found.length - NEW_TERMINAL_MATCHES), empty: 'No project, worktree or branch matches' }];
        }
        // before typing: a section per project, its newest branches only; the rest typing reaches
        return this.treeProjects().map(function (p) {
          var mine = items.filter(function (item) { return item.project === p.name; });
          var branches = mine.filter(function (item) { return item.branch; });
          return { key: 'terminal:' + p.name, title: p.name, hint: branches.length ? 'worktrees by last use, then the newest branches without one' : '',
            items: mine.filter(function (item) { return item.branch !== true; }).concat(branches.slice(0, NEW_TERMINAL_BRANCHES)), counter: counter(items.length),
            more: Math.max(0, branches.length - NEW_TERMINAL_BRANCHES), moreText: 'branches; type to search them' };
        });
      },
      // Every place a new terminal can go: projects, worktrees, branches without a worktree (project: the project's name).
      newTerminalTargets: function () {
        var self = this;
        var running = function (ids) { return self.consoles.filter(function (c) { return ids.includes(c.co) && self.isLive(c); }).length; };
        var items = [];
        this.treeProjects().forEach(function (p) {
          var main = p.checkouts[0];
          var ids = self.checkoutIds(p.name);
          var n = running([main.id]);
          items.push(self.hubItem({ id: 'nt:p:' + p.name, icon: p.git ? 'branch.svg' : 'project-folder.svg', title: p.git ? main.label : p.name,
            detail: p.name + (p.git ? ' · main checkout' : ' · no git'), project: p.name,
            state: n ? '● ' + n + ' running' : null, open: function () { self.newTerminalIn(main); }, preview: function () { return self.newTerminalPreview(main); } }));
          self.byRecency(ids.slice(1)).forEach(function (id) {
            var co = self.checkout(id);
            var used = self.worktreeLastUsed(id);
            var k = running([id]);
            items.push(self.hubItem({ id: 'nt:w:' + id, icon: 'branch.svg', title: co.label, detail: p.name + (co.branch && co.branch !== co.label ? ' · ' + co.branch : ''),
              project: p.name, age: used === null ? 'never' : self.age(used), state: k ? '● ' + k + ' running' : null,
              open: function () { self.newTerminalIn(co); }, preview: function () { return self.newTerminalPreview(co); } }));
          });
          p.branches.forEach(function (b) {
            items.push(self.hubItem({ id: 'nt:b:' + p.name + ':' + b.name, icon: 'branch.svg', title: b.name, detail: (b.remote || 'local') + ' · ' + p.name,
              project: p.name, weak: b.remote !== null, branch: true, age: self.age(b.modified),
              open: function () { self.newTerminalOnBranch(p, b); }, preview: function () { return self.newWorktreePreview(p, b); } }));
          });
        });
        return items;
      },
      // A terminal in `co`, its worktree shown expanded in the tree.
      newTerminalIn: function (co) {
        this.revealWorktree(co);
        this.newTerminal(co.id);
      },
      newTerminalPreview: function (co) {
        var k = this.consoles.filter(function (c) { return c.co === co.id && this.isLive(c); }, this);
        return { title: 'New terminal in ' + co.label, sub: co.path, lines: k.length ? ['Running there:'].concat(k.map(function (c) { return '  ' + c.title; })) : ['Nothing running there yet.'] };
      },
      newWorktreePreview: function (p, b) {
        var path = this.worktreePath(p, b);
        var steps = b.remote ? ['Only on ' + b.remote + '. Enter:', '1. a local ' + b.name, '   tracking ' + b.remote + '/' + b.name] : ['Enter:'];
        var n = b.remote ? 2 : 1;
        var from = p.worktrees === '.worktree' ? '(the default .worktree)' : '(worktrees = "' + p.worktrees + '" in .ive.toml)';
        return { title: b.name, sub: p.name + ' · no worktree yet', mono: true,
          lines: steps.concat([n + '. a worktree in', '   ' + path, '   ' + from, (n + 1) + '. a terminal there']) };
      },
      // :42 — line 42 of the back tab's file or scratch.
      hub_line: function () {
        var self = this;
        var line = Number(this.hub.query.trim().slice(1));
        var back = this.tabs.find(function (t) { return t.key === self.hub.back; });
        var text = back && (back.kind === 'file' || back.kind === 'scratch');
        var items = text ? [this.hubItem({
          id: 'ln:' + back.key, icon: this.tabIcon(back), title: 'Line ' + line, detail: this.tabTitle(back), co: back.co, project: back.project,
          open: function () { self.goToLine(back, line); },
          preview: function () { return Object.assign(self.hubTabPreview(back), { line: line }); }
        })] : [];
        return [{ key: 'line', title: 'Go to line', hint: text ? 'in the tab shown before the hub' : '', items: items,
          empty: 'The tab shown before the hub is not a file: :' + line + ' goes to a line of one' }];
      },
      // Shows `tab` scrolled to `line` (from 1), marked until the next jump.
      goToLine: function (tab, line) {
        tab.line = line;
        this.showTab(tab);
      },
      // After Enter on a row in several worktrees: which one, most recently used first; Enter on the first is the default.
      hub_pick: function (groups) {
        var self = this;
        var pick = this.hub.pick;
        var items = this.byRecency(pick.item.choices).map(function (id) {
          var co = self.checkout(id);
          var shown = self.worktreeLastUsed(id);
          return self.hubItem({
            id: 'pk:' + id, icon: 'branch.svg', title: co.label, detail: (co.branch && co.branch !== co.label ? co.branch + ' · ' : '') + self.cardTags(co).join(' · '),
            age: shown === null ? 'never' : self.age(shown), state: pick.item.stateIn ? pick.item.stateIn(id) : null,
            open: function () { pick.item.openIn(id); },
            preview: function () { return pick.item.previewIn(id); }
          });
        });
        return [{ key: 'pick', title: 'Open ' + pick.item.title + ' in', hint: 'worktrees, most recently used first', items: this.hubMatching(items, groups), empty: 'No worktree matches' }];
      },
      hub_branches: function (groups) {
        return [
          { key: 'worktrees', title: 'Worktrees', items: this.hubMatching(this.hubWorktrees(), groups) },
          { key: 'branches', title: 'Branches without a worktree', hint: 'Enter creates one', items: this.hubMatching(this.hubBranches(), groups).slice(0, RENDERED) }
        ];
      },

      // Items: { id, icon, badge, title, detail, titleSpans, detailSpans, co, state, age, open (Enter), preview }.
      // Matching runs over "title detail"; positions past the title highlight the detail.
      hubMatching: function (items, groups) {
        if (groups.length === 0) return items;
        return items.map(function (item) {
          var text = item.title + ' ' + (item.detail || '');
          var hit = match(text, text.toLowerCase(), groups);
          if (hit === null) return null;
          item.score = hit.score;
          item.titleSpans = spans(text, hit.positions, 0, item.title.length);
          item.detailSpans = spans(text, hit.positions, item.title.length + 1, text.length);
          return item;
        }).filter(Boolean).sort(function (a, b) { return b.score - a.score; });
      },
      hubItem: function (fields) {
        var item = Object.assign({ badge: null, detail: '', co: null, choices: null, state: null, mark: null, age: '', pathKey: null, excluded: null }, fields);
        item.titleSpans = item.titleSpans || [{ t: item.title, on: false }];
        item.detailSpans = item.detailSpans || [{ t: item.detail, on: false }];
        return item;
      },
      // `line`: from a `file:line` query, where a file or scratch row opens.
      hubRecent: function (groups, line) {
        var self = this;
        var items = this.history.slice().reverse().filter(function (h) { return h.tab.kind !== 'hub'; }).map(function (h) {
          var tab = h.tab;
          var open = self.tabs.some(function (t) { return t.key === tab.key; });
          // a process that ended has no tab to come back to
          if (tab.kind === 'console' && self.consoleOf(tab) === null) return null;
          // a recent tab is in one worktree: it reopens there
          return self.hubItem({
            id: 'h:' + tab.key, icon: self.tabIcon(tab), badge: tab.kind === 'scratch' ? 'scratch.svg' : null,
            title: self.tabTitle(tab), detail: self.hubWhere(tab), co: tab.co, age: self.age(h.at),
            project: tab.project, path: tab.kind === 'file' ? tab.path.slice(1) : undefined, pathKey: tab.kind === 'file' ? tab.project + ':' + tab.path.slice(1) : null,
            state: open ? 'open' : 'closed',
            open: function () {
              self.openTab(Object.assign({}, tab));
              if (line && (tab.kind === 'file' || tab.kind === 'scratch')) self.goToLine(self.currentTab(), line);
            },
            preview: function () { return Object.assign(self.hubTabPreview(tab), line ? { line: line } : {}); }
          });
        }).filter(Boolean);
        return this.hubMatching(items, groups);
      },
      hubWhere: function (tab) {
        if (tab.kind === 'file') return tab.path.slice(1, tab.path.lastIndexOf('/') + 1);
        if (tab.kind === 'diff') return tab.path + ' · ' + this.sourceText(this.checkout(tab.co), tab.source);
        if (tab.kind === 'console') return this.consoleKind(this.consoleOf(tab));
        if (tab.kind === 'scratch') return 'scratch · ' + tab.project;
        if (tab.kind === 'commit') return 'commit · ' + tab.project;
        if (tab.kind === 'projectPage') return 'project overview';
        if (tab.kind === 'gitLog') return tab.project;
        return '';
      },
      hubChanged: function (groups) {
        var self = this;
        var items = [];
        this.treeProjects().forEach(function (p) {
          p.checkouts.forEach(function (co) {
            co.changes.forEach(function (x) {
              var at = x.path.lastIndexOf('/') + 1;
              // a change is in one worktree
              items.push(self.hubItem({
                id: 'ch:' + co.id + ':' + x.path, icon: self.fileIcon(x.path), title: x.path.slice(at), detail: x.path.slice(0, at), co: co.id, mark: x.mark,
                project: p.name, path: x.path,
                open: function () { self.openFile(co.id, '/' + x.path); },
                preview: function () { return { title: x.path, sub: self.label(co.id) + ' · uncommitted', diff: self.diffRows(x.diff) }; }
              }));
            });
          });
        });
        return this.hubMatching(items, groups);
      },
      hubWaiting: function () {
        var self = this;
        return this.waitingConsoles().map(function (c) { return self.hubConsole(c); });
      },
      hubRunning: function () {
        var self = this;
        return this.consoles.filter(function (c) { return self.isLive(c); }).map(function (c) { return self.hubConsole(c); });
      },
      hubProcesses: function () {
        var self = this;
        return this.consoles.map(function (c) { return self.hubConsole(c); });
      },
      hubConsole: function (c) {
        var self = this;
        var state = c.kind === 'run' ? this.runStatus(c) : c.kind === 'agent' ? 'running' : 'idle';
        return this.hubItem({
          id: 'c:' + c.id, project: c.co.split('@')[0], ownerPath: c.folder, icon: this.consoleIcon(c), title: c.title, detail: this.consoleKind(c) + (c.folder ? ' · ' + c.folder.slice(1) : ''), co: c.co,
          state: c.attention !== null ? '● ' + c.attention.text : state, attention: c.attention !== null,
          open: function () { self.openConsole(c); },
          preview: function () { return { title: c.title, sub: self.label(c.co) + ' · ' + self.consoleKind(c), lines: c.lines.slice(-30).map(textOf), mono: true }; }
        });
      },
      // Every checkout of atlassian has the same .run.toml files (ids[0]'s): Enter asks where to run, unless one runs already.
      hubConfigs: function () {
        var self = this;
        var ids = this.hubScopeCheckout() ? [this.hubScopeCheckout()] : this.checkoutIds('atlassian');
        return this.runFolders(ids[0]).flatMap(function (folder) {
          return self.runFile(folder).configs.map(function (cfg) {
            var liveIn = function (id) { return self.liveRun(id, folder, cfg.key); };
            var running = ids.filter(liveIn);
            return self.hubItem(self.withChoices({
              id: 'rc:' + folder + ':' + cfg.key, icon: cfg.type === 'serve' ? 'file-html.svg' : 'run-config.svg', title: cfg.name, detail: folder.slice(1),
              state: running.length ? self.runStatus(liveIn(running[0])) + ' in ' + running.map(function (id) { return self.label(id); }).join(', ') : null,
              stateIn: function (id) { var live = liveIn(id); return live ? self.runStatus(live) : null; },
              openIn: function (id) { var live = liveIn(id); if (live) self.openConsole(live); else self.requestRun(id, folder, cfg.key, 'tab'); },
              previewIn: function (id) {
                var co = self.checkout(id);
                var live = liveIn(id);
                return { title: cfg.name, sub: self.label(id) + ' · ' + folder.slice(1) + '/.run.toml', mono: true,
                  lines: live ? live.lines.slice(-30).map(textOf) : [self.shownCommand(co, folder, cfg)].concat(cfg.before ? ['before: ' + cfg.before.join(', ')] : []) };
              }
            }, ids));
          });
        });
      },
      // A row in `ids` worktrees: with one, it is marked with it and opens there; with more, Enter asks.
      withChoices: function (fields, ids) {
        var self = this;
        var first = function () { return self.byRecency(ids)[0]; };
        return Object.assign(fields, {
          co: ids.length === 1 ? ids[0] : null, choices: ids,
          open: function () { fields.openIn(first()); },
          preview: function () { return fields.previewIn(first()); }
        });
      },
      hubWorktrees: function () {
        var self = this;
        return this.treeProjects().flatMap(function (p) {
          return p.checkouts.filter(function (co) { return co.branch !== null; }).map(function (co) {
            return self.hubItem({
              id: 'w:' + co.id, icon: 'branch.svg', title: co.branch, detail: p.name + ' · ' + self.cardTags(co).join(' · '), co: co.id,
              open: function () { self.revealWorktree(co); },
              preview: function () { return { title: co.branch, sub: co.path, lines: ['base ' + co.base].concat(self.cardTags(co), co.changes.map(function (x) { return x.mark + ' ' + x.path; })) }; }
            });
          });
        });
      },
      hubBranches: function () {
        var self = this;
        return this.treeProjects().flatMap(function (p) {
          return p.branches.map(function (b) {
            return self.hubItem({
              id: 'b:' + p.name + ':' + b.name, icon: 'branch.svg', title: b.name, detail: p.name + ' · no worktree', age: self.age(b.modified),
              open: function () { self.createWorktree(p, b); },
              preview: function () { return { title: b.name, sub: p.name, lines: ['No worktree yet.', 'Enter: New worktree in ' + self.worktreePath(p, b)] }; }
            });
          });
        });
      },
      hubActions: function () {
        var self = this;
        var back = this.tabs.find(function (t) { return t.key === self.hub.back; }) || null;
        var co = back && back.co ? back.co : this.byRecency(this.checkoutIds('atlassian'))[0];
        var action = function (title, detail, run) {
          return self.hubItem({ id: 'a:' + title, icon: 'run.svg', title: title, detail: detail, open: run,
            preview: function () { return { title: title, sub: detail, lines: ['(an action: Enter runs it)'] }; } });
        };
        return [
          action('New terminal', this.label(co), function () { self.newTerminal(co); }),
          action('New scratch', this.hubProject(), function () { self.newScratch(self.project(self.hubProject())); }),
          action('Open Git log', this.hubProject(), function () { self.openGitLog(self.project(self.hubProject())); }),
          action('Open project overview', this.hubProject(), function () { self.openProjectPage(self.project(self.hubProject())); }),
          action('Close Other Tabs', back ? 'all but ' + this.tabTitle(back) : 'nothing shown', function () { if (back) { self.showTab(back); self.closeTabs(back, self.tabsBeside(back, 'other')); } }),
          action('Settings', 'Ctrl+,', function () { self.openSettings(); }),
          action('Theme: Dark', 'Appearance', function () { self.themePreference = 'dark'; self.leaveHub(); }),
          action('Theme: Light', 'Appearance', function () { self.themePreference = 'light'; self.leaveHub(); }),
          action('Theme: System', 'Appearance', function () { self.themePreference = 'system'; self.leaveHub(); })
        ];
      },

      // The files of every open project (or the shown tab's), fzf-ranked; scratches too. Memoized per query.
      hubSearch: function (groups, split) {
        var scope = { project: this.hubScopeProject(), checkout: this.hubScopeCheckout() };
        var open = this.treeProjectNames();
        var key = JSON.stringify([this.hubText(), scope, open, this.hub.withExcluded, this.hub.loaded, this.scratches.length]);
        if (searchMemo.key !== key) searchMemo = { key: key, value: this.hubScan(groups, scope, open, this.hub.withExcluded) };
        var self = this;
        var hits = searchMemo.value.hits;
        var items = hits.slice(0, RENDERED).map(function (h) { return self.hubFileItem(h, split.line); });
        return { items: items, more: Math.max(0, hits.length - RENDERED) };
      },
      hubScan: function (groups, scope, open, withExcluded) {
        var started = performance.now();
        var hits = [];
        var total = 0;
        var inScope = function (project) { return open.includes(project) && (scope.project === null || project === scope.project); };
        for (var i = 0; i < corpus.length; i++) {
          var e = corpus[i];
          if (inScope(e.project) === false) continue;
          // a worktree scope leaves out paths that worktree lacks
          if (scope.checkout !== null && e.checkouts !== null && e.checkouts.includes(scope.checkout) === false) continue;
          if (e.excluded !== null && withExcluded === false) continue;
          total++;
          var hit = match(e.path, e.lower, groups, e.nameAt);
          if (hit) hits.push({ e: e, score: hit.score, positions: hit.positions });
        }
        this.scratches.forEach(function (s) {
          if (inScope(s.project) === false) return;
          total++;
          var path = 'scratches/' + s.name;
          var hit = match(path, path.toLowerCase(), groups, 10);
          if (hit) hits.push({ e: { project: s.project, path: path, nameAt: 10, scratch: s }, score: hit.score, positions: hit.positions });
        });
        // fzf --scheme=path tiebreak: score, then the shorter path
        hits.sort(function (a, b) { return b.score - a.score || a.e.path.length - b.e.path.length || (a.e.path < b.e.path ? -1 : 1); });
        return { hits: hits, total: total, ms: performance.now() - started, hidden: withExcluded ? 0 : this.hubExcludedCount(inScope) };
      },
      // Excluded and gitignored files of the projects in scope: the real count, though the mock searches a sample.
      hubExcludedCount: function (inScope) {
        var named = corpus.filter(function (e) { return e.excluded === 'excluded' && e.checkouts === null && inScope(e.project); }).length;
        return Object.keys(excludedTotals).filter(inScope).reduce(function (n, p) { return n + excludedTotals[p]; }, named);
      },
      hubFileItem: function (h, line) {
        var self = this;
        var e = h.e;
        var fields = {
          id: 'fi:' + e.project + ':' + e.path, pathKey: e.project + ':' + e.path, icon: this.fileIcon(e.path), badge: e.scratch ? 'scratch.svg' : null,
          title: e.path.slice(e.nameAt), detail: e.path.slice(0, e.nameAt) || (e.project + '/'),
          titleSpans: spans(e.path, h.positions, e.nameAt, e.path.length), detailSpans: spans(e.path, h.positions, 0, e.nameAt),
          project: e.project, path: e.path, excluded: e.excluded || null
        };
        if (e.scratch) {
          // a scratch belongs to the project, not a worktree
          return this.hubItem(Object.assign(fields, {
            state: this.tabs.some(function (t) { return t.key === 's:' + e.scratch.name; }) ? 'open' : null,
            open: function () { self.openScratch(e.scratch); if (line) self.goToLine(self.currentTab(), line); },
            preview: function () { return { title: e.path, sub: 'scratch · ' + e.project, lines: e.scratch.text.split('\n'), numbered: true, line: line }; }
          }));
        }
        // in a worktree scope a file opens there without asking
        var ids = this.hubScopeCheckout() ? [this.hubScopeCheckout()] : e.checkouts || this.checkoutIds(e.project);
        return this.hubItem(this.withChoices(Object.assign(fields, {
          state: this.tabs.some(function (t) { return t.kind === 'file' && t.project === e.project && t.path === '/' + e.path; }) ? 'open' : null,
          stateIn: function (id) { return self.tabs.some(function (t) { return t.key === 'f:' + id + '/' + e.path; }) ? 'open' : null; },
          openIn: function (id) { self.openFile(id, '/' + e.path); if (line) self.goToLine(self.currentTab(), line); },
          previewIn: function (id) {
            var co = self.checkout(id);
            return { title: e.path, sub: self.label(id) + ' · ' + co.path, lines: self.fileLines(co, '/' + e.path), numbered: true, line: line };
          }
        }), ids));
      },
      fileLines: function (co, path) {
        var hit = this.findFile(co.id, path);
        return hit ? hit.text.split('\n') : placeholderLines(path);
      },
      hubTabPreview: function (tab) {
        if (tab.kind === 'file') return { title: tab.path.slice(1), sub: this.label(tab.co), lines: this.fileLines(this.checkout(tab.co), tab.path), numbered: true };
        if (tab.kind === 'scratch') return { title: tab.title, sub: 'scratch · ' + tab.project, lines: this.scratch(tab.title).text.split('\n'), numbered: true };
        if (tab.kind === 'diff') { var d = this.diffOfTab(tab); return { title: tab.path, sub: this.label(tab.co), diff: d ? this.diffRows(d.diff) : [] }; }
        if (tab.kind === 'console') { var c = this.consoleOf(tab); return { title: c.title, sub: this.consoleKind(c), lines: c.lines.slice(-30).map(textOf), mono: true }; }
        if (tab.kind === 'commit') { var cm = this.commitTab(tab); return { title: cm.message, sub: cm.author + ' · ' + cm.date, diff: this.diffRows(cm.files[0].diff) }; }
        return { title: this.tabTitle(tab), sub: this.tabTooltip(tab), lines: ['Enter reopens the ' + this.hubWhere(tab) + '.'] };
      },

      hubSelectedIndex: function () { return this.hub.pick === null ? this.hub.selected : this.hub.pick.selected; },
      hubSelected: function (view) { return view.items[Math.min(this.hubSelectedIndex(), view.items.length - 1)] || null; },
      hubSelect: function (index) { if (this.hub.pick === null) this.hub.selected = index; else this.hub.pick.selected = index; },
      hubPreview: function () {
        var item = this.hubSelected(this.hubView());
        return item ? item.preview() : null;
      },
      hubStats: function () {
        // searchMemo is not reactive: reading the view's inputs makes the counter follow them
        var view = this.hubView();
        var stats = view.stats;
        if (view.mode === 'terminal') return view.sections.length ? view.sections[0].counter : '';
        if (view.mode === 'content') {
          var files = new Set(view.items.map(function (item) { return item.id.slice(0, item.id.lastIndexOf(':')); })).size;
          var lines = view.items.length + (view.sections.length && view.sections[0].more ? view.sections[0].more : 0);
          return lines === 0 ? '' : lines.toLocaleString('en') + (lines === 1 ? ' match' : ' matches') + ' in ' + files + (files === 1 ? ' file' : ' files');
        }
        if (stats === null) {
          return this.hub.loaded ? this.hub.loaded.toLocaleString('en') + ' files' : 'loading…';
        }
        return stats.hits.length.toLocaleString('en') + '/' + stats.total.toLocaleString('en') + ' · ' + stats.ms.toFixed(1) + ' ms';
      },
      // "+ 381,877 excluded" while they are left out, so the user knows there is more to search
      hubHiddenNote: function () {
        var stats = this.hubView().stats;
        return stats && stats.hidden ? stats.hidden.toLocaleString('en') + ' excluded not searched' : '';
      },
      toggleExcluded: function () {
        this.hub.withExcluded = !this.hub.withExcluded;
        this.hub.selected = 0;
      },

      // ---- keys ----
      hubKey: function (event) {
        var view = this.hubView();
        var count = view.items.length;
        var move = { ArrowDown: 1, ArrowUp: -1, PageDown: 10, PageUp: -10 }[event.key];
        if (event.ctrlKey && 'jn'.includes(event.key)) move = 1;
        if (event.ctrlKey && 'kp'.includes(event.key)) move = -1;
        if (move !== undefined && count) { event.preventDefault(); this.hubSelect(Math.max(0, Math.min(count - 1, this.hubSelectedIndex() + move))); return; }
        var item = this.hubSelected(view);
        if (event.altKey && /^[1-9]$/.test(event.key)) { event.preventDefault(); this.hubOpen(view.items[Number(event.key) - 1], event); return; }
        if (event.key === 'Enter' && item) { event.preventDefault(); this.hubOpen(item, event); return; }
        if (event.altKey && event.key.toLowerCase() === 'i' && this.hub.pick === null) { event.preventDefault(); this.toggleExcluded(); return; }
        // Ctrl+Shift+F: file names or contents; Alt+R, Alt+W: a contents query as a regex, whole words
        if (event.ctrlKey && event.shiftKey && event.code === 'KeyF' && this.hub.pick === null && this.hub.newTerminal === false) {
          event.preventDefault(); this.hub.content = !this.hub.content; this.hub.selected = 0; return;
        }
        if (event.altKey && event.key.toLowerCase() === 'r' && this.hub.content) { event.preventDefault(); this.hub.regex = !this.hub.regex; this.hub.selected = 0; return; }
        if (event.altKey && event.key.toLowerCase() === 'w' && this.hub.content) { event.preventDefault(); this.hub.word = !this.hub.word; this.hub.selected = 0; return; }
        // Backspace on an empty query: first the picked files and folders go, then the scope widens
        if (event.key === 'Backspace' && this.hub.query === '' && this.hub.pick === null && this.hub.paths.length) { event.preventDefault(); this.hub.paths = []; return; }
        if (event.key === 'Tab' && view.mode === 'scope') { event.preventDefault(); if (item) item.open(); return; }
        if (event.key === 'Tab' && this.hub.pick === null) { event.preventDefault(); this.hubCycleScope(); return; }
        // Backspace on an empty query widens the scope back to all projects
        if (event.key === 'Backspace' && this.hub.query === '' && this.hub.pick === null && this.hubScope() !== 'all') { event.preventDefault(); this.setHubScope('all'); return; }
        if (event.ctrlKey && event.key === 'y' && item) { event.preventDefault(); this.hubCopyPath(item); return; }
        if (event.key === 'Escape') { event.preventDefault(); this.hubEscape(); }
      },
      // Escape steps back: from the worktree question to the list, then clears the search, then leaves the hub.
      hubEscape: function () {
        if (this.hub.pick !== null) { this.hub.pick = null; return; }
        if (this.hub.query !== '') { this.hub.query = ''; this.hub.selected = 0; return; }
        this.leaveHub();
      },
      // Enter opens (the hub goes); Shift+Enter opens behind and stays; Alt+Enter types the path into the last terminal.
      // A row in several worktrees asks first (the question remembers Shift).
      hubOpen: function (item, event) {
        if (item === undefined || item === null) return;
        if (event && event.altKey && event.key === 'Enter') { this.hubTypePath(item); return; }
        var behind = Boolean(event && event.shiftKey);
        if (this.hub.pick === null && item.choices && item.choices.length > 1) { this.hub.pick = { item: item, behind: behind, query: '', selected: 0 }; return; }
        if (this.hub.pick !== null) behind = this.hub.pick.behind || behind;
        this.hub.pick = null;
        if (behind) { this.hubOpenBehind(item); return; }
        item.open();
      },
      hubOpenBehind: function (item) {
        this.temporary = null;
        item.open();
        this.remember(this.currentTab());
        this.focus = 'hub';
        this.temporary = 'hub';
        this.hubNote('Opened ' + item.title + ' behind');
      },
      hubTypePath: function (item) {
        var c = this.consoles.slice().reverse().find(function (x) { return x.kind === 'shell'; });
        var co = this.hubFirstChoice(item);
        if (c === undefined || item.path === undefined || co === null) { this.hubNote('No terminal to type into'); return; }
        c.input = (c.input ? c.input + ' ' : '') + "'" + this.checkout(co).path + '/' + item.path + "'";
        this.hubNote('Typed into ' + c.title);
      },
      hubCopyPath: function (item) {
        var text = item.path || item.title;
        try { navigator.clipboard.writeText(text); } catch (e) { /* mock: clipboard may be blocked */ }
        this.hubNote('Copied ' + text);
      },
      hubNote: function (text) {
        var self = this;
        this.hub.note = text;
        clearTimeout(this.hubNoteTimer);
        this.hubNoteTimer = setTimeout(function () { self.hub.note = ''; }, 2500);
      },
      hubKeys: function (mode) {
        if (mode === 'pick') return [['↑↓', 'move'], ['Enter', 'open here'], ['type', 'filter'], ['Esc', 'back to the list']];
        if (mode === 'scope') return [['↑↓', 'move'], ['Enter', 'search there'], ['Tab', 'search there'], ['Esc', 'clear']];
        if (mode === 'line') return [['Enter', 'go to the line'], ['Esc', 'clear']];
        if (mode === 'terminal') return [['↑↓', 'move'], ['Enter', 'new terminal there'], ['type', 'filter'], ['Esc', this.hub.query === '' ? 'back' : 'clear'], ['Ctrl+E', 'hub']];
        if (mode === 'content') {
          return [['↑↓', 'move'], ['Enter', 'open at the line'], ['Shift+Enter', 'behind'], ['Alt+Enter', 'to terminal'], ['Ctrl+Y', 'copy'],
            ['Alt+R', this.hub.regex ? 'regex on' : 'regex'], ['Alt+W', this.hub.word ? 'words on' : 'words'], ['Alt+I', this.hub.withExcluded ? 'excluded on' : 'excluded'],
            ['Ctrl+Shift+F', 'names'], ['Esc', this.hub.query === '' ? 'back' : 'clear']];
        }
        var common = [['↑↓', 'move'], ['Enter', 'open'], ['Shift+Enter', 'behind'], ['Esc', this.hub.query === '' ? 'back' : 'clear']];
        if (mode === 'files' || mode === 'changed') {
          return common.concat([['Alt+Enter', 'to terminal'], ['Ctrl+Y', 'copy'], ['Tab', 'scope'], [':', 'any scope'], [':42', 'line'], ['Alt+I', this.hub.withExcluded ? 'hide excluded' : 'excluded'],
            ['Ctrl+E', mode === 'changed' ? 'all' : 'changed']]);
        }
        return common;
      }
    }
  };
})();
