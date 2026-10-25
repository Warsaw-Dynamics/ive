// IVE mock: where a tab or a hub row belongs, as the app does it (DESIGN.md "Where a tab belongs"). A project's
// `.favicon.*` on its default branch is its icon; a folder with a `.favicon.*` of its own is a module of a monorepo, and
// the files under it carry that icon. A worktree has no picture of its own, so it keeps a colour; the main checkout has
// none. Tabs group by project; a group folds to its head and its current tab.
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

  // Whether `peers` span more than one `key`: a cue shows only where it tells something apart.
  function isSpread(peers, key) {
    return new Set(peers.map(key).filter(function (v) { return v !== null; })).size > 1;
  }

  window.IVE_IDENT = {
    // identFolded: the keys of the tab groups folded to their head
    // identGroupMenu: the group whose menu is open, and where ({ key, x, y }), while menu is 'tabGroup'
    // identMenuDrag: a drag in that menu (pressGroupMenu); identWorktreeOrder: per project, its worktrees' order as
    // dragged in the menu
    state: function () {
      return { identFolded: [], identGroupMenu: null, identMenuDrag: null, identMenuDragged: false, identWorktreeOrder: {} };
    },

    methods: {
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

      // ---- the tree: a project row's favicon, a module folder's; null keeps the badge or the folder icon ----
      projectFavicon: function (project) { return this.projectOwner(project).icon; },
      treeFavicon: function (node) {
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
      // The tree's rows; a worktree's own row (head) carries its colour for the rail: { node, depth, wt }. The main
      // checkout's rows and the rows outside a worktree have none.
      identTreeRows: function (rows) {
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
      // The badge on a tab's icon: its owner's favicon while the row has more than one owner; under a head only a
      // module's, as the head names the project.
      tabOwnerBadge: function (tab) {
        var self = this;
        var owner = this.tabOwner(tab);
        if (owner === null || owner.icon === null) return null;
        if (owner.module === null && this.tabGroups().length > 1) return null;
        return isSpread(this.tabs, function (t) { var o = self.tabOwner(t); return o && o.key; }) ? owner : null;
      },
      // A group's menu: the group's head names the project, so a tab's badge is its module's, whenever it has one.
      tabModuleBadge: function (tab) {
        var owner = this.tabOwner(tab);
        return owner && owner.module && owner.icon ? owner : null;
      },
      // { mode: 'tint' | 'label', colour, label } or null, while the row spans more than one worktree. A folded
      // group's tab names its worktree (muted for the main checkout), as nothing beside it does; the others are tinted
      // off the main checkout.
      tabWorktreeCue: function (tab) {
        if (tab.co === null || tab.co === undefined) return null;
        if (isSpread(this.tabs, function (t) { return t.co || null; }) === false) return null;
        var colour = this.worktreeColour(tab.co);
        if (this.isInFoldedGroup(tab)) return { mode: 'label', colour: colour, label: this.label(tab.co) };
        return colour === null ? null : { mode: 'tint', colour: colour, label: this.label(tab.co) };
      },
      // Tabs grouped by project in the order each project's first tab shows; a worktree's tabs side by side. A
      // project's own tabs (scratches, Git log, commits) join it; tabs of no project form a group without a head, and
      // with one project open there are no heads. { key, project, head: { owner, label } | null, tabs, shown, folded,
      // current }
      tabGroups: function () {
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
        var heads = groups.filter(function (g) { return g.project !== null; }).length > 1;
        return groups.map(function (g) {
          var head = g.project === null || heads === false ? null : { owner: self.projectOwner(g.project), label: g.project };
          return self.foldedGroup({ key: g.key, project: g.project, head: head, tabs: self.byWorktree(g.tabs).flat() });
        });
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
      isInFoldedGroup: function (tab) {
        return this.identFolded.includes('p:' + (this.tabProject(tab) || ''));
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
      // A group's menu by worktree: the main checkout first, the project's own tabs last. { key, label, colour, tabs }
      tabGroupSections: function (group) {
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

      // ---- the URL hash (route.js): fold and gc (a folded group's current tab), wto (a project's worktree order) ----
      tabGroupsRoute: function (add) {
        var self = this;
        this.tabGroups().filter(function (g) { return g.folded; }).forEach(function (g) {
          add('fold', g.key);
          if (g.current.key !== self.focus) add('gc', g.current.key);
        });
        var order = this.identWorktreeOrder;
        Object.keys(order).forEach(function (project) { add('wto', project + '~' + order[project].join(',')); });
      },
      // After the tabs are back: a folded group's current tab is the one of its tabs shown last, so each gc tab is
      // remembered as shown, then the shown tab after them.
      restoreTabGroups: function (p) {
        var self = this;
        this.identFolded = p.getAll('fold');
        this.identWorktreeOrder = {};
        p.getAll('wto').forEach(function (v) {
          var at = v.indexOf('~');
          if (at !== -1) self.identWorktreeOrder[v.slice(0, at)] = v.slice(at + 1).split(',');
        });
        var byKey = function (key) { return self.tabs.find(function (t) { return t.key === key; }) || null; };
        p.getAll('gc').forEach(function (key) { self.remember(byKey(key)); });
        this.remember(byKey(this.focus));
      },

      // ---- hub rows: the badge on each row's icon, its module's favicon, else its project's ----
      hubOwnerBadge: function (item) {
        var project = item.project || (item.co ? this.checkout(item.co).project : null);
        var owner = this.ownerOf(project, item.ownerPath || item.path || '');
        return owner && owner.icon ? owner : null;
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
