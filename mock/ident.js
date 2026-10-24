// IVE mock-only, a proposal (brainstorm 2026-10-23): where a tab or a hub row belongs. A project's `.favicon.*` on its
// default branch is its icon; a folder with a `.favicon.*` of its own is a module of a monorepo, and the files under it
// carry that icon. A worktree has no picture of its own, so it keeps a colour; the main checkout has none.
// The mock bar switches how the owner and the worktree show, to compare variants; nothing here is in the app.
(function () {
  'use strict';

  var FAVICONS = 'favicons/';
  // the folders with a .favicon.* per project; a path belongs to the deepest one above it
  var MODULES = {
    atlassian: ['internal-apps/ive', 'internal-apps/communication/idp', 'internal-apps/communication/paas', 'external-share-jira',
      'external-share-confluence', 'approval-path', 'macro-pack', 'document-sign', 'homepage', 'timelog2']
  };
  // the projects with a .favicon.* on their default branch; the others show their initials
  var PROJECT_ICONS = ['atlassian', 'forge-installer', 'platform'];
  var OPTIONS = {
    // badge: the favicon small on the file-type icon's lower right; one beside it (two icons in a tab) was dropped
    owner: ['none', 'badge'],
    worktree: ['today', 'none', 'underline', 'tint', 'dot', 'label'],
    // groups: a group per worktree; projects: a group per project
    layout: ['flat', 'groups', 'projects'],
    // the project tree's worktree rows: tint (the row's background), underline (its name), dot, or rail (a bar at
    // the row's left)
    tree: ['none', 'tint', 'underline', 'dot', 'rail'],
    when: ['always', 'spread', 'clash']
  };
  var TITLES = { owner: 'Project or module', worktree: 'Worktree', layout: 'Tabs', tree: 'Worktree in the tree', when: 'Show' };
  var TODAY = { owner: 'none', worktree: 'today', layout: 'flat', tree: 'none', when: 'always' };
  // name, values, what it shows
  var PRESETS = [
    ['Today', TODAY, 'as the app'],
    // the direction picked (brainstorm 2026-10-24)
    ['Chosen', { owner: 'badge', worktree: 'tint', layout: 'projects', tree: 'rail', when: 'spread' }, 'project groups, tint, rail'],
    ['A Badge, tint', { owner: 'badge', worktree: 'tint', layout: 'flat', when: 'spread' }, 'badge, tinted tab'],
    ['B Groups', { owner: 'badge', worktree: 'tint', layout: 'groups', when: 'spread' }, 'a group per worktree'],
    ['C Clashes', { owner: 'badge', worktree: 'label', layout: 'flat', when: 'clash' }, 'only same names'],
    ['D Badge, underline', { owner: 'badge', worktree: 'underline', layout: 'flat', when: 'spread' }, 'badge, underline'],
    ['E Project groups', { owner: 'badge', worktree: 'tint', layout: 'projects', when: 'spread' }, 'a group per project']
  ];

  // pixels the pointer moves with a menu row pressed before it drags, as the tab row's
  var DRAG_THRESHOLD = 4;
  // byWorktree's ranks: worktrees never dragged come after the dragged ones, the tabs of no worktree last
  var UNORDERED = 1000;
  var LAST = 1000000;

  function trimmed(path) { return (path || '').replace(/^\//, ''); }

  // `keys` with drop.key moved before or after drop.target.
  function placed(keys, drop) {
    var rest = keys.filter(function (k) { return k !== drop.key; });
    var at = rest.indexOf(drop.target) + (drop.side === 'after' ? 1 : 0);
    return rest.slice(0, at).concat(drop.key, rest.slice(at));
  }

  function moduleOf(project, path) {
    return (MODULES[project] || []).filter(function (m) { return path === m || path.startsWith(m + '/'); })
      .reduce(function (best, m) { return best === null || m.length > best.length ? m : best; }, null);
  }

  function distinct(values) { return new Set(values.filter(function (v) { return v !== null; })).size; }

  window.IVE_IDENT = {
    OPTIONS: OPTIONS,
    TITLES: TITLES,
    PRESETS: PRESETS,
    // identFolded: the keys of the tab groups folded to their head
    // identGroupMenu: the group whose menu is open, and where ({ key, x, y }), while menu is 'tabGroup'
    // identMenuDrag: a drag in that menu (pressGroupMenu); identWorktreeOrder: per project, its worktrees' order as
    // dragged in the menu
    state: function () {
      return { ident: Object.assign({}, TODAY), identFolded: [], identGroupMenu: null, identMenuDrag: null, identMenuDragged: false, identWorktreeOrder: {} };
    },

    methods: {
      // A preset sets the parts it names; the tree's keeps its pick, except Today's.
      identPreset: function (values) { this.ident = Object.assign({}, this.ident, values); },
      isIdentPreset: function (values) {
        var self = this;
        return Object.keys(values).every(function (k) { return self.ident[k] === values[k]; });
      },
      // #ident=owner,worktree,layout,when
      identParam: function () {
        if (this.isIdentPreset(TODAY)) return null;
        var self = this;
        return Object.keys(OPTIONS).map(function (k) { return self.ident[k]; }).join(',');
      },
      // An older link's `icon` owner reads as badge.
      restoreIdent: function (value) {
        var parts = value.replace(/^icon,/, 'badge,').split(',');
        var self = this;
        Object.keys(OPTIONS).forEach(function (k, i) { if (OPTIONS[k].includes(parts[i])) self.ident[k] = parts[i]; });
      },

      // ---- owners: { key, name, project, module, icon, initials } ----
      ownerOf: function (project, path) {
        if (project === null || project === undefined) return null;
        var m = moduleOf(project, trimmed(path));
        var name = m === null ? project : m.split('/').pop();
        var icon = m !== null ? name : PROJECT_ICONS.includes(project) ? project : null;
        return { key: project + '/' + (m || ''), name: name, project: project, module: m, icon: icon && FAVICONS + icon + '.svg', initials: this.initials(name) };
      },
      projectOwner: function (project) { return this.ownerOf(project, ''); },
      tabProject: function (tab) { return tab.project || (tab.co ? this.checkout(tab.co).project : null); },
      tabOwner: function (tab) {
        var path = tab.kind === 'file' || tab.kind === 'diff' ? tab.path : tab.kind === 'console' ? (this.consoleOf(tab) || {}).folder : '';
        return this.ownerOf(this.tabProject(tab), path);
      },
      // The main checkout has no colour: only the other worktrees need telling apart.
      worktreeColour: function (co) {
        var checkout = co ? this.checkout(co) : null;
        return checkout === null || checkout.main ? null : checkout.colour;
      },

      // Whether `subject` gets its cue among `peers`: always; when the peers span more than one (spread); or only
      // when a peer of the same title differs (clash). key: what the cue tells apart.
      identWanted: function (subject, peers, key, title) {
        if (this.ident.when === 'always') return key(subject) !== null;
        if (this.ident.when === 'spread') return distinct(peers.map(key)) > 1;
        return peers.some(function (p) { return p !== subject && title(p) === title(subject) && key(p) !== key(subject); });
      },

      // ---- the tree: a project row's favicon, a module folder's; null keeps the badge or the folder icon ----
      projectFavicon: function (project) {
        return this.ident.owner === 'none' ? null : this.projectOwner(project).icon;
      },
      treeFavicon: function (node) {
        if (this.ident.owner === 'none') return null;
        if (node.project) return this.projectOwner(node.project.name).icon;
        var path = this.treeFolderPath(node);
        if (path === null) return null;
        var owner = this.ownerOf(node.key.slice(node.key.indexOf(':') + 1, node.key.indexOf('@')), path);
        return owner.module === trimmed(path) ? owner.icon : null;
      },
      // A folder row's path in its checkout: Files rows carry their entry, the changes' folders their path as hint.
      treeFolderPath: function (node) {
        if (node.children === undefined) return null;
        if (node.entry) return node.entry.folder ? node.entry.path : null;
        return node.key.startsWith('cg:') ? node.hint : null;
      },

      // The tree's rows, each a worktree's own row (head) or under one, with that worktree's colour: { node, depth, wt }.
      // The main checkout's rows and the rows outside a worktree have none.
      identTreeRows: function (rows) {
        if (this.ident.tree === 'none') return rows;
        var self = this;
        var at = null;
        return rows.map(function (row) {
          if (at !== null && row.depth <= at.depth) at = null;
          var head = row.node.key.startsWith('w:');
          if (head) at = { depth: row.depth, colour: self.worktreeColour(row.node.key.slice(2)) };
          return at === null || at.colour === null ? row : Object.assign({}, row, { wt: { colour: at.colour, head: head, depth: at.depth } });
        });
      },

      // ---- tabs ----
      // The owner the tab shows, in either form; a badge needs a favicon, its initials would not read at that size.
      tabOwnerShown: function (tab) {
        if (this.ident.owner === 'none') return null;
        var self = this;
        var owner = this.tabOwner(tab);
        if (owner === null) return null;
        // a group's head shows the project, so its tabs show only a module's icon
        if (this.ident.layout !== 'flat' && owner.module === null && this.tabGroups().length > 1) return null;
        var key = function (t) { var o = self.tabOwner(t); return o && o.key; };
        return this.identWanted(tab, this.tabs, key, this.tabTitle.bind(this)) ? owner : null;
      },
      // A group's menu: the group's head names the project, so a tab's badge is its module's, whenever it has one.
      tabModuleBadge: function (tab) {
        var owner = this.ident.owner === 'badge' ? this.tabOwner(tab) : null;
        return owner && owner.module && owner.icon ? owner : null;
      },
      tabOwnerIcon: function (tab) { return this.ident.owner === 'icon' ? this.tabOwnerShown(tab) : null; },
      tabOwnerBadge: function (tab) {
        var owner = this.ident.owner === 'badge' ? this.tabOwnerShown(tab) : null;
        return owner && owner.icon ? owner : null;
      },
      // { mode, colour, label } or null; 'today' is the app's diff underline (tabUnderline).
      // The tab's worktree cue; a project group's tab out of its folded group names its worktree, as nothing beside
      // it does.
      tabWorktreeCue: function (tab) {
        var mode = this.isInFoldedProject(tab) ? 'label' : this.ident.worktree;
        if (mode === 'today' || mode === 'none' || tab.co === null || tab.co === undefined) return null;
        // in groups the group carries it
        if (this.ident.layout === 'groups') return null;
        var self = this;
        var key = function (t) { return t.co || null; };
        if (this.identWanted(tab, this.tabs, key, this.tabTitle.bind(this)) === false) return null;
        var colour = this.worktreeColour(tab.co);
        if (mode === 'label') return { mode: mode, colour: colour, label: this.label(tab.co) };
        return colour === null ? null : { mode: mode, colour: colour, label: self.label(tab.co) };
      },
      // Tabs in their order, or grouped in the order each group first shows: by worktree (a project's own tabs,
      // scratches, Git log, commits, join its main checkout), or by project (no colour: each tab keeps its
      // worktree's cue). { key, head: { owner, label, sub } | null, colour, tabs }
      tabGroups: function () {
        if (this.ident.layout === 'flat') return [{ key: 'all', head: null, colour: null, tabs: this.tabs, shown: this.tabs, folded: false }];
        var self = this;
        var groups = this.ident.layout === 'projects' ? this.projectTabGroups() : this.worktreeTabGroups();
        return groups.map(function (g) { return self.foldedGroup(g); });
      },
      // A folded group shows its head, its count and its own current tab: the one of its tabs shown last.
      foldedGroup: function (group) {
        var folded = group.head !== null && this.identFolded.includes(group.key);
        var current = this.groupCurrentTab(group.tabs);
        return Object.assign(group, { folded: folded, current: current, shown: folded ? [current] : group.tabs });
      },
      groupCurrentTab: function (tabs) {
        var keys = tabs.map(function (t) { return t.key; });
        var last = this.history.filter(function (h) { return keys.includes(h.key); }).pop();
        return last ? tabs[keys.indexOf(last.key)] : tabs[0];
      },
      isInFoldedProject: function (tab) {
        return this.ident.layout === 'projects' && this.identFolded.includes('p:' + (this.tabProject(tab) || ''));
      },
      toggleTabGroup: function (key) {
        this.identFolded = this.identFolded.includes(key) ? this.identFolded.filter(function (k) { return k !== key; }) : this.identFolded.concat(key);
      },
      // A click on the head's name: the group's tabs in a menu under it (fixed, the tab row scrolls and would clip it).
      openTabGroupMenu: function (key, event) {
        var box = event.currentTarget.getBoundingClientRect();
        this.identGroupMenu = { key: key, x: box.left, y: box.bottom + 4 };
        this.menu = 'tabGroup';
      },
      tabGroupMenu: function () {
        var at = this.menu === 'tabGroup' ? this.identGroupMenu : null;
        var group = at && this.tabGroups().find(function (g) { return g.key === at.key; });
        return group ? Object.assign({ group: group }, at) : null;
      },
      // A group's menu in sections: a project group's by worktree (the main checkout first, the project's own tabs
      // last), a worktree group's in one, without a head. { key, label, colour, tabs }
      tabGroupSections: function (group) {
        if (this.ident.layout !== 'projects') return [{ key: 'all', label: null, colour: null, tabs: group.tabs }];
        var self = this;
        return this.byWorktree(group.tabs).map(function (run) {
          var co = run[0].co || null;
          // scratches, Git log and commits are the project's, in no worktree
          return { key: co || 'project', label: co ? self.label(co) : 'Whole project', colour: self.worktreeColour(co), tabs: run };
        });
      },
      // `tabs` (of one project) in runs of one worktree each: in the order the group's menu was dragged to, else the
      // main checkout's, then the others' in the order each first shows; the tabs of no worktree last.
      byWorktree: function (tabs) {
        var self = this;
        var order = tabs.length ? this.identWorktreeOrder[this.tabProject(tabs[0])] || [] : [];
        var rank = function (run) {
          var co = run[0].co;
          if (!co) return LAST;
          var at = order.indexOf(co);
          return at !== -1 ? at : self.checkout(co).main ? UNORDERED : UNORDERED + 1;
        };
        var runs = [];
        tabs.forEach(function (tab) {
          var run = runs.find(function (r) { return (r[0].co || null) === (tab.co || null); });
          if (run) run.push(tab); else runs.push([tab]);
        });
        return runs.sort(function (a, b) { return rank(a) - rank(b); });
      },

      // ---- the group's menu: a tab dragged within its worktree, a worktree's head among the project's ----
      // drag: { kind: 'tab' | 'section', key, section }; the drop marks { target, side: 'before' | 'after' }.
      pressGroupMenu: function (drag, event) {
        // no text selection starts from the press, so a drag does not select the rows it passes
        event.preventDefault();
        var self = this;
        var start = { x: event.clientX, y: event.clientY };
        var move = function (e) {
          if (self.identMenuDrag === null && Math.hypot(e.clientX - start.x, e.clientY - start.y) < DRAG_THRESHOLD) return;
          self.identMenuDrag = Object.assign({}, drag, self.groupMenuDrop(drag, e));
        };
        var up = function () {
          window.removeEventListener('mousemove', move);
          window.removeEventListener('mouseup', up);
          var done = self.identMenuDrag;
          self.identMenuDrag = null;
          if (done === null) return;
          // the release's click is the drag's end, not a pick
          self.identMenuDragged = true;
          setTimeout(function () { self.identMenuDragged = false; });
          if (done.target !== null) self.dropGroupMenu(done);
        };
        window.addEventListener('mousemove', move);
        window.addEventListener('mouseup', up);
      },
      // The row under the pointer a drag may land by: a tab of the same worktree, or another worktree's head (the
      // project's own section stays last). { target, side }, target null for nowhere.
      groupMenuDrop: function (drag, event) {
        var under = document.elementFromPoint(event.clientX, event.clientY);
        var row = under && under.closest(drag.kind === 'tab' ? '[data-menu-tab]' : '[data-menu-section]');
        var target = row && (drag.kind === 'tab' ? row.dataset.menuTab : row.dataset.menuSection);
        var fits = drag.kind === 'tab' ? row && row.dataset.section === drag.section : target !== 'project';
        if (row === null || target === drag.key || fits === false) return { target: null, side: null };
        var box = row.getBoundingClientRect();
        return { target: target, side: event.clientY < box.top + box.height / 2 ? 'before' : 'after' };
      },
      dropGroupMenu: function (drop) {
        if (drop.kind === 'tab') { this.moveTabBeside(drop); return; }
        var group = this.tabGroupMenu().group;
        var order = this.tabGroupSections(group).map(function (s) { return s.key; }).filter(function (k) { return k !== 'project'; });
        this.identWorktreeOrder = Object.assign({}, this.identWorktreeOrder, { [group.project]: placed(order, drop) });
      },
      moveTabBeside: function (drop) {
        var keys = placed(this.tabs.map(function (t) { return t.key; }), drop);
        var byKey = new Map(this.tabs.map(function (t) { return [t.key, t]; }));
        this.tabs = keys.map(function (k) { return byKey.get(k); });
      },
      groupMenuDropSide: function (key) {
        var drag = this.identMenuDrag;
        return drag !== null && drag.target === key ? drag.side : null;
      },
      worktreeTabGroups: function () {
        var self = this;
        var groups = [];
        this.tabs.forEach(function (tab) {
          var project = self.tabProject(tab);
          var co = tab.co || (project ? self.project(project).checkouts[0].id : null);
          var key = co || '';
          var group = groups.find(function (g) { return g.key === key; });
          if (group === undefined) {
            group = { key: key, co: co, project: project, tabs: [] };
            groups.push(group);
          }
          group.tabs.push(tab);
        });
        var heads = groups.filter(function (g) { return g.co !== null; }).length > 1 || this.ident.when === 'always';
        return groups.map(function (g) {
          var head = g.co === null || heads === false ? null : {
            owner: self.projectOwner(g.project),
            label: self.checkout(g.co).main ? g.project : self.label(g.co),
            sub: self.checkout(g.co).main ? self.checkout(g.co).branch : null
          };
          return { key: g.key, head: head, colour: self.worktreeColour(g.co), tabs: g.tabs };
        });
      },
      projectTabGroups: function () {
        var self = this;
        var groups = [];
        this.tabs.forEach(function (tab) {
          var project = self.tabProject(tab);
          var group = groups.find(function (g) { return g.project === project; });
          if (group === undefined) {
            group = { key: 'p:' + (project || ''), project: project, tabs: [] };
            groups.push(group);
          }
          group.tabs.push(tab);
        });
        var heads = groups.filter(function (g) { return g.project !== null; }).length > 1 || this.ident.when === 'always';
        return groups.map(function (g) {
          var head = g.project === null || heads === false ? null : { owner: self.projectOwner(g.project), label: g.project, sub: null };
          // a worktree's tabs side by side, so their tint reads as one run
          return { key: g.key, project: g.project, head: head, colour: null, tabs: self.byWorktree(g.tabs).flat() };
        });
      },

      // ---- hub rows ----
      hubOwner: function (item) {
        var project = item.project || (item.co ? this.checkout(item.co).project : null);
        return this.ownerOf(project, item.ownerPath || item.path || '');
      },
      hubOwnerShown: function (item, items) {
        if (this.ident.owner === 'none') return null;
        var self = this;
        var owner = this.hubOwner(item);
        if (owner === null) return null;
        if (this.ident.layout !== 'flat' && item.inGroup) return null;
        var key = function (i) { var o = self.hubOwner(i); return o && o.key; };
        return this.identWanted(item, items, key, function (i) { return i.title; }) ? owner : null;
      },
      // With project groups (the direction picked) the hub stays as the app's but for this badge, on every row whose
      // module or project has a favicon.
      hubOwnerBadge: function (item, items) {
        if (this.ident.owner !== 'badge') return null;
        var owner = this.ident.layout === 'projects' ? this.hubOwner(item) : this.hubOwnerShown(item, items);
        return owner && owner.icon ? owner : null;
      },
      // How the row's worktree tag draws: 'today' (the app's), or the variant's mode with the tag's colour.
      hubCoCue: function (item) {
        var mode = this.ident.layout === 'projects' ? 'today' : this.ident.worktree;
        if (mode === 'today') return { mode: 'today', colour: this.checkout(item.co).colour };
        return { mode: mode, colour: this.worktreeColour(item.co) };
      },
      // groups: a section's rows by owner, the groups in the order of their best row; a head over each when there
      // are several.
      identGroupSections: function (sections) {
        if (this.ident.layout !== 'groups') return sections;
        var self = this;
        sections.forEach(function (s) {
          var groups = [];
          s.items.forEach(function (item) {
            var owner = self.hubOwner(item);
            var key = owner ? owner.key : '';
            var group = groups.find(function (g) { return g.key === key; });
            if (group === undefined) { group = { key: key, owner: owner, items: [] }; groups.push(group); }
            group.items.push(item);
          });
          var heads = groups.length > 1 || self.ident.when === 'always';
          s.items = groups.flatMap(function (g) {
            g.items.forEach(function (item, i) {
              item.inGroup = heads && g.owner !== null;
              item.groupHead = item.inGroup && i === 0 ? { owner: g.owner, label: g.owner.module ? g.owner.project + ' › ' + g.owner.name : g.owner.name } : null;
            });
            return g.items;
          });
        });
        return sections;
      },

      // Mock-only: a busy day's tabs, the same names in several worktrees, projects and modules.
      openIdentDemo: function () {
        var self = this;
        var dev = 'atlassian@dev';
        var ive = 'atlassian@GEN-817-ive';
        var idp = 'atlassian@IDP-30-Reserved-subdomain-returns-404';
        this.tabs = [];
        this.temporary = null;
        [[ive, '/internal-apps/ive/src/theme.rs'], [ive, '/internal-apps/ive/Cargo.toml'], [dev, '/internal-apps/ive/src/theme.rs'],
          [idp, '/internal-apps/communication/idp/pom.xml'], [dev, '/internal-apps/misc/pom.xml'], [dev, '/approval-path/pom.xml'],
          ['forge-installer@main', '/lib/installer.ex'], ['forge-installer@main', '/README.md'], ['platform@main', '/README.md'], ['notes@', '/todo.md']]
          .forEach(function (f) { self.openFile(f[0], f[1]); });
        this.consoles.filter(function (c) { return c.co === ive; }).forEach(function (c) { self.openConsole(c); });
        this.newTerminal(dev);
        this.newTerminal(idp);
        this.openScratch(this.scratch('idp-users.sql'));
        var co = this.checkout(ive);
        this.openDiff(co, co.changes[3], 'work');
        this.focus = this.tabs[0].key;
      }
    }
  };
})();
