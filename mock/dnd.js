// A proposal (DESIGN.md "Drag and drop"): rows are picked with Ctrl+click and Shift+click; files and folders drag onto
// a folder, worktree or project of any project (same worktree: move, elsewhere: copy; Ctrl copies, Shift moves);
// processes drag onto a worktree, which starts the same thing there. Ctrl+C / Ctrl+V copy picked files keeping their
// folders. A copy that would overwrite something opens a Conflicts tab; nothing is written before it is answered.
(function () {
  'use strict';

  // a conflict's choices; Replace is not offered where there is nothing to replace with one file
  var ACTIONS = [['replace', 'Replace'], ['skip', 'Skip'], ['both', 'Keep both']];
  var NOTE_MS = 5000;

  function parentOf(path) { return path.slice(0, path.lastIndexOf('/')); }
  function nameOf(path) { return path.slice(path.lastIndexOf('/') + 1); }
  function withPaths(entry, parent) {
    entry.path = parent + '/' + entry.name;
    (entry.children || []).forEach(function (child) { withPaths(child, entry.path); });
  }
  function isWithin(path, folder) { return path === folder || path.startsWith(folder + '/'); }
  // The deepest folder holding every path ('' for the root).
  function commonFolder(paths) {
    var parts = paths.map(function (p) { return parentOf(p).split('/'); });
    var common = parts.reduce(function (out, p) {
      var i = 0;
      while (i < out.length && i < p.length && out[i] === p[i]) i++;
      return out.slice(0, i);
    });
    return common.join('/');
  }
  // An entry's files, with their paths under it ('' for the entry itself); an empty folder is one row.
  function leavesOf(entry, rel) {
    if (entry.children === undefined) return [{ rel: rel, entry: entry }];
    if (entry.children.length === 0) return [{ rel: rel, entry: entry }];
    return entry.children.flatMap(function (child) { return leavesOf(child, rel + '/' + child.name); });
  }
  // Lines only in `a`, as a multiset difference.
  function linesOnlyIn(a, b) {
    var rest = b.slice();
    return a.filter(function (line) {
      var at = rest.indexOf(line);
      if (at === -1) return true;
      rest.splice(at, 1);
      return false;
    }).length;
  }
  // name.ext → name (2).ext, the first that is free.
  function freeName(path, taken) {
    var name = nameOf(path);
    var dot = name.lastIndexOf('.');
    var stem = dot > 0 ? name.slice(0, dot) : name;
    var ext = dot > 0 ? name.slice(dot) : '';
    for (var n = 2; ; n++) {
      var candidate = parentOf(path) + '/' + stem + ' (' + n + ')' + ext;
      if (taken(candidate) === false) return candidate;
    }
  }

  window.IVE_DND = {
    state: function () {
      return {
        // picked tree rows, by node key, and the row Shift+click ranges from
        picked: [],
        pickAnchor: null,
        // the drag going on: { kind: 'files' | 'processes', sources, over (node key), target, verb, hint, x, y, ok }
        drag: null,
        // the files Ctrl+C copied: { sources }
        fileClip: null,
        // the menu a drop asks with (menu === 'drop'): { at, items }
        dropMenu: { at: { x: 0, y: 0 }, items: [] },
        // a line under the tree after a drop or paste
        dndNote: null,
        conflictCount: 0
      };
    },

    methods: {
      // ---- picking rows ----
      isPickable: function (node) { return (node.entry !== undefined && node.entry.path !== '') || node.key.startsWith('c:'); },
      isPicked: function (node) { return this.picked.includes(node.key); },
      // Ctrl+click adds or removes a row, Shift+click picks the rows from the last one; a plain click picks the row
      // and does what it did before.
      clickRow: function (node, event) {
        if (event.ctrlKey || event.metaKey) { this.togglePick(node); return; }
        if (event.shiftKey && this.pickAnchor !== null) { this.pickRange(node); return; }
        this.picked = this.isPickable(node) ? [node.key] : [];
        this.pickAnchor = node.key;
        this.activateNode(node);
      },
      togglePick: function (node) {
        if (this.isPickable(node) === false) return;
        this.pickAnchor = node.key;
        if (this.isPicked(node)) { this.picked = this.picked.filter(function (k) { return k !== node.key; }); return; }
        this.picked = this.picked.concat(node.key);
      },
      pickRange: function (node) {
        var keys = this.treeRows().map(function (r) { return r.node; }).filter(this.isPickable, this).map(function (n) { return n.key; });
        var from = keys.indexOf(this.pickAnchor);
        var to = keys.indexOf(node.key);
        if (from === -1 || to === -1) return;
        this.picked = keys.slice(Math.min(from, to), Math.max(from, to) + 1);
      },
      pickedNodes: function () {
        var picked = this.picked;
        return this.treeRows().map(function (r) { return r.node; }).filter(function (n) { return picked.includes(n.key); });
      },
      // The files and folders a drag or copy of `node` carries: the picked ones when it is picked, else itself;
      // without those inside another one carried.
      fileSources: function (node) {
        var nodes = this.isPicked(node) ? this.pickedNodes() : [node];
        var all = nodes.filter(function (n) { return n.entry !== undefined && n.entry.path !== ''; }).map(function (n) { return n.entry; });
        return all.filter(function (s) {
          return all.some(function (o) { return o !== s && o.co === s.co && o.folder && isWithin(s.path, o.path) && o.path !== s.path; }) === false;
        });
      },
      processSources: function (node) {
        var nodes = this.isPicked(node) ? this.pickedNodes() : [node];
        var ids = nodes.filter(function (n) { return n.key.startsWith('c:'); }).map(function (n) { return Number(n.key.slice(2)); });
        return this.consoles.filter(function (c) { return ids.includes(c.id); });
      },

      // ---- dragging ----
      isDraggable: function (node) { return this.isPickable(node); },
      dragStart: function (node, event) {
        if (this.isPicked(node) === false) { this.picked = [node.key]; this.pickAnchor = node.key; }
        var files = node.key.startsWith('c:') === false;
        this.drag = { kind: files ? 'files' : 'processes', sources: files ? this.fileSources(node) : this.processSources(node),
          over: null, target: null, verb: null, hint: '', ok: false, x: 0, y: 0 };
        event.dataTransfer.effectAllowed = 'copyMove';
        event.dataTransfer.setData('text/plain', files ? this.drag.sources.map(this.absolutePath, this).join('\n') : '');
        // the hint under the pointer is the drag picture
        var blank = document.createElement('canvas');
        blank.width = blank.height = 1;
        event.dataTransfer.setDragImage(blank, 0, 0);
      },
      dragOver: function (node, event) {
        if (this.drag === null) return;
        var d = this.drag;
        d.over = node.key;
        d.x = event.clientX;
        d.y = event.clientY;
        if (d.kind === 'files') this.aimFiles(node, event); else this.aimProcesses(node, event);
        if (d.ok) { event.preventDefault(); event.dataTransfer.dropEffect = d.verb === 'move' ? 'move' : 'copy'; }
      },
      dragEnd: function () { this.drag = null; },
      isDropRow: function (node) { return this.drag !== null && this.drag.over === node.key; },
      // Where files dropped on `node` go: a folder, a worktree's or project's root, the Files folder; a file's folder.
      fileTargetOf: function (node) {
        if (node.key.startsWith('wf:') && node.key.endsWith('/files')) return { co: node.key.slice(3, node.key.lastIndexOf('/')), path: '' };
        if (node.entry === undefined) return null;
        return { co: node.entry.co, path: node.entry.folder ? node.entry.path : parentOf(node.entry.path) };
      },
      aimFiles: function (node, event) {
        var d = this.drag;
        var to = this.fileTargetOf(node);
        d.target = to;
        d.ok = false;
        if (to === null) { d.hint = 'Drop on a folder, a worktree or a project'; return; }
        var into = d.sources.find(function (s) { return s.co === to.co && s.folder && isWithin(to.path, s.path); });
        if (into) { d.hint = 'Cannot put ' + nameOf(into.path) + ' inside itself'; return; }
        var staying = d.sources.every(function (s) { return s.co === to.co && parentOf(s.path) === to.path; });
        d.verb = this.dropVerb(d.sources, to, event);
        if (staying && d.verb === 'move') { d.hint = 'Already here; Ctrl+drop copies'; return; }
        d.ok = true;
        d.hint = (d.verb === 'move' ? 'Move ' : 'Copy ') + this.itemCount(d.sources) + ' to ' + this.placeName(to) + this.layoutNote(d.sources);
      },
      // The same worktree moves, another copies; Ctrl copies, Shift moves.
      dropVerb: function (sources, to, event) {
        if (event.ctrlKey) return 'copy';
        if (event.shiftKey) return 'move';
        return sources.every(function (s) { return s.co === to.co; }) ? 'move' : 'copy';
      },
      layoutNote: function (sources) {
        if (this.sourceCheckouts(sources).length > 1) return ' · flat: from ' + this.sourceCheckouts(sources).length + ' worktrees';
        return this.hasMixedFolders(sources) ? ' · asks: keep folders or flat' : '';
      },
      sourceCheckouts: function (sources) { return Array.from(new Set(sources.map(function (s) { return s.co; }))); },
      hasMixedFolders: function (sources) { return new Set(sources.map(function (s) { return parentOf(s.path); })).size > 1; },
      itemCount: function (sources) {
        if (sources.length === 1) return nameOf(sources[0].path);
        return this.counted(sources.length, 'item', 'items');
      },
      placeName: function (to) { return this.label(to.co) + ' · ' + (to.path === '' ? '/' : to.path.slice(1)); },
      // Processes go to a worktree: its row, its Processes folder, or a single-checkout project's row.
      // A drop moves terminals and Claude Code there (the shell is not touched; it follows its directory again once
      // that changes); runs stay with their .run.toml. Ctrl+drop starts the same things there instead.
      aimProcesses: function (node, event) {
        var d = this.drag;
        var co = this.processTargetOf(node);
        d.target = co;
        d.ok = false;
        if (co === null) { d.hint = 'Drop on a worktree or a project'; return; }
        var others = d.sources.filter(function (c) { return c.co !== co; });
        if (others.length === 0) { d.hint = 'Already in ' + this.label(co); return; }
        if (event.ctrlKey) { this.aimStart(others, co); return; }
        var movable = others.filter(function (c) { return c.kind !== 'run'; });
        var runs = others.length - movable.length;
        var runNote = runs ? ' · ' + this.counted(runs, 'run stays', 'runs stay') + ' (Ctrl+drop starts ' + (runs === 1 ? 'its' : 'their') + ' copy there)' : '';
        if (movable.length === 0) { d.hint = 'A run stays in its worktree; Ctrl+drop starts its copy in ' + this.label(co); return; }
        d.ok = true;
        d.verb = 'move';
        d.hint = 'Move ' + this.processesName(movable) + ' to ' + this.label(co) + runNote;
      },
      aimStart: function (others, co) {
        var d = this.drag;
        var missing = others.filter(function (c) { return this.isRunMissingIn(c, co); }, this);
        d.ok = missing.length < others.length;
        d.verb = 'start';
        d.hint = 'Start ' + this.processesName(others) + ' in ' + this.label(co) +
          (missing.length ? ' · ' + missing.map(this.processName, this).join(', ') + ': no such configuration there' : '') + ' · the originals keep running';
      },
      processesName: function (list) { return list.length === 1 ? this.processName(list[0]) : this.counted(list.length, 'process', 'processes'); },
      processTargetOf: function (node) {
        if (node.key.startsWith('w:')) return node.key.slice(2);
        if (node.key.startsWith('wf:') && node.key.endsWith('/processes')) return node.key.slice(3, node.key.lastIndexOf('/'));
        if (node.project) return node.project.checkouts[0].id;
        return null;
      },
      processName: function (c) { return c.kind === 'run' ? this.config(c.folder, c.key).name : c.title.split(':')[0] === c.title ? c.title : this.consoleKind(c); },
      isRunMissingIn: function (c, co) { return c.kind === 'run' && this.findFile(co, c.folder + '/.run.toml') === null; },

      drop: function (node, event) {
        var d = this.drag;
        this.drag = null;
        if (d === null || d.ok === false) return;
        event.preventDefault();
        if (d.kind === 'processes' && d.verb === 'move') { this.moveProcessesTo(d.target, d.sources); return; }
        if (d.kind === 'processes') { this.startProcessesIn(d.target, d.sources); return; }
        this.dropFiles({ verb: d.verb, to: d.target, sources: d.sources }, { x: event.clientX, y: event.clientY });
      },
      // From several worktrees: flat. From one, with files in different folders: a menu asks.
      dropFiles: function (op, at) {
        if (this.sourceCheckouts(op.sources).length > 1 || this.hasMixedFolders(op.sources) === false) {
          this.transfer(Object.assign({ layout: 'flat' }, op));
          return;
        }
        var self = this;
        var base = commonFolder(op.sources.map(function (s) { return s.path; }));
        this.dropMenu = { at: at, items: [
          { title: 'Keep folders, from ' + (base === '' ? '/' : base.slice(1) + '/'), enabled: true, run: function () { self.transfer(Object.assign({ layout: 'tree' }, op)); } },
          { title: 'Flat, all into ' + (op.to.path === '' ? '/' : nameOf(op.to.path) + '/'), enabled: true, run: function () { self.transfer(Object.assign({ layout: 'flat' }, op)); } },
          null,
          { title: 'Cancel', enabled: true, run: function () {} }
        ] };
        this.menuSub = null;
        this.menu = 'drop';
      },

      // ---- copy and paste ----
      copyFiles: function (node) {
        var sources = this.fileSources(node);
        if (sources.length === 0) return;
        this.fileClip = { sources: sources };
        this.note('Copied ' + this.itemCount(sources) + '; Ctrl+V on a folder pastes, keeping their folders');
      },
      // Keeps the folders under the sources' common one; from several worktrees, flat.
      pasteFiles: function (node) {
        var to = node && this.fileTargetOf(node);
        if (this.fileClip === null || to === null || to === undefined) return;
        var sources = this.fileClip.sources;
        var layout = this.sourceCheckouts(sources).length > 1 ? 'flat' : 'tree';
        this.transfer({ verb: 'copy', to: to, sources: sources, layout: layout, paste: true });
      },
      // Ctrl+C, Ctrl+V and Escape over the tree's picks (not while typing).
      clipKey: function (event) {
        if (event.target.closest && event.target.closest('input, textarea')) return;
        if (event.key === 'Escape') { this.picked = []; return; }
        if ((event.ctrlKey || event.metaKey) === false || event.shiftKey || event.altKey) return;
        var nodes = this.pickedNodes();
        if (nodes.length === 0) return;
        if (event.code === 'KeyC') { event.preventDefault(); this.copyFiles(nodes[0]); }
        if (event.code === 'KeyV') { event.preventDefault(); this.pasteFiles(nodes[nodes.length - 1]); }
      },
      copyItem: function (node) {
        var self = this;
        if (node.entry === undefined || node.entry.path === '') return false;
        var n = this.fileSources(node).length;
        return { title: n > 1 ? 'Copy ' + n + ' Items' : 'Copy', enabled: true, run: function () { self.copyFiles(node); } };
      },
      // Rename… and Delete as the app offers them (a file, folder or changed file, not a checkout's root); the mock
      // leaves the files as they are: the app's rename field and trash are in the README
      renameItem: function (node) {
        if (node.entry === undefined || node.entry.path === '' || this.fileSources(node).length !== 1) return false;
        return { title: 'Rename…', enabled: true, run: function () {} };
      },
      deleteItem: function (node) {
        if (node.entry === undefined || node.entry.path === '') return false;
        return { title: 'Delete', enabled: true, run: function () {} };
      },
      pasteItem: function (node) {
        var self = this;
        if (this.fileTargetOf(node) === null) return false;
        return { title: this.fileClip ? 'Paste ' + this.itemCount(this.fileClip.sources) : 'Paste', enabled: this.fileClip !== null, run: function () { self.pasteFiles(node); } };
      },

      // ---- the plan, and what it writes ----
      // Every file the operation writes, where, and what is already there.
      // op: { verb, to: { co, path }, sources: [{ co, path, folder }], layout: 'tree' | 'flat', paste? }
      planTransfer: function (op) {
        var self = this;
        var base = op.layout === 'tree' ? commonFolder(op.sources.map(function (s) { return s.path; })) : null;
        var planned = {};
        var rows = op.sources.flatMap(function (s) {
          var top = op.to.path + (base === null ? '/' + nameOf(s.path) : s.path.slice(base.length));
          var entry = self.findFile(s.co, s.path);
          return leavesOf(entry, '').map(function (leaf) {
            return self.planRow(op, { co: s.co, path: s.path + leaf.rel }, top + leaf.rel, leaf.entry, planned);
          });
        });
        return { op: op, rows: rows, base: base };
      },
      planRow: function (op, src, dest, entry, planned) {
        var row = { id: src.co + src.path, src: src, dest: dest, entry: entry, there: null, reason: null, action: null, first: null, open: false };
        if (planned[dest]) {
          row.reason = 'twice';
          row.first = planned[dest];
          return row;
        }
        planned[dest] = src;
        var there = this.findFile(op.to.co, dest);
        // pasting onto itself makes a copy beside it, without asking
        if (src.co === op.to.co && src.path === dest) { row.action = 'both'; return row; }
        if (there === null) return row;
        row.there = there;
        if (there.children !== undefined || entry.children !== undefined) row.reason = 'kind';
        else if (there.text === entry.text) { row.reason = 'same'; row.action = 'skip'; }
        else row.reason = 'differs';
        return row;
      },
      transfer: function (op) {
        var plan = this.planTransfer(op);
        if (plan.rows.some(function (r) { return r.reason !== null; })) { this.openConflicts(plan); return; }
        this.applyPlan(plan);
      },
      applyPlan: function (plan) {
        var self = this;
        var op = plan.op;
        var written = [];
        plan.rows.forEach(function (row) {
          if (row.action === 'skip') return;
          var dest = row.action === 'both' ? freeName(row.dest, function (p) { return self.findFile(op.to.co, p) !== null; }) : row.dest;
          if (nameOf(dest) === '.run.toml') self.adoptRunFile(parentOf(row.src.path), parentOf(dest));
          self.writeEntry(op.to.co, dest, row.entry);
          written.push(dest);
          if (op.verb === 'move') self.removeEntry(row.src.co, row.src.path);
        });
        if (op.verb === 'move') op.sources.forEach(function (s) { self.pruneEmpty(s.co, s.path); });
        this.revealWritten(op.to.co, written);
        var skipped = plan.rows.length - written.length;
        this.note((op.verb === 'move' ? 'Moved ' : 'Copied ') + this.counted(written.length, 'file', 'files') + ' to ' + this.placeName(op.to) +
          (skipped ? ' · skipped ' + skipped : ''));
      },
      // Writes a file (or an empty folder) at `path`, making its folders.
      writeEntry: function (coId, path, entry) {
        var folder = this.ensureFolder(coId, parentOf(path));
        var name = nameOf(path);
        var at = folder.findIndex(function (e) { return e.name === name; });
        var copy = entry.children !== undefined ? { name: name, children: [] } : { name: name, text: entry.text };
        withPaths(copy, parentOf(path));
        if (at === -1) folder.push(copy); else folder.splice(at, 1, copy);
      },
      ensureFolder: function (coId, path) {
        var list = this.checkout(coId).tree;
        var walked = '';
        path.split('/').filter(Boolean).forEach(function (name) {
          walked += '/' + name;
          var next = list.find(function (e) { return e.name === name && e.children; });
          if (next === undefined) { next = { name: name, children: [] }; withPaths(next, parentOf(walked)); list.push(next); }
          list = next.children;
        });
        return list;
      },
      removeEntry: function (coId, path) {
        var folder = this.ensureFolder(coId, parentOf(path));
        var at = folder.findIndex(function (e) { return e.path === path; });
        if (at !== -1) folder.splice(at, 1);
      },
      // After a move: the moved folders that are left empty go (what was skipped keeps its folders).
      pruneEmpty: function (coId, path) {
        var entry = this.findFile(coId, path);
        if (entry === null || entry.children === undefined) return;
        entry.children.slice().forEach(function (child) { this.pruneEmpty(coId, child.path); }, this);
        if (entry.children.length === 0) this.removeEntry(coId, path);
      },
      // Unfolds the target's folders and picks what was written.
      revealWritten: function (coId, paths) {
        var self = this;
        var co = this.checkout(coId);
        ['p:' + co.project, 'w:' + coId, 'wf:' + coId + '/files'].forEach(function (k) { self.toggled = self.toggled.filter(function (t) { return t !== k; }); });
        paths.forEach(function (p) {
          var folder = '';
          parentOf(p).split('/').filter(Boolean).forEach(function (name) {
            folder += '/' + name;
            var key = 'f:' + coId + folder;
            if (self.toggled.includes(key) === false) self.toggled.push(key);
          });
        });
        this.picked = paths.map(function (p) { return 'f:' + coId + p; });
      },
      note: function (text) {
        var self = this;
        this.dndNote = text;
        clearTimeout(this.noteTimer);
        this.noteTimer = setTimeout(function () { self.dndNote = null; }, NOTE_MS);
      },

      // ---- processes ----
      // Terminals and Claude Code are filed under `co`; runs stay. The shell keeps its directory.
      moveProcessesTo: function (co, sources) {
        var moved = sources.filter(function (c) { return c.co !== co && c.kind !== 'run'; });
        moved.forEach(function (c) { c.co = co; });
        this.toggled = this.toggled.filter(function (t) { return t !== 'w:' + co && t !== 'wf:' + co + '/processes'; });
        this.picked = moved.map(function (c) { return 'c:' + c.id; });
        this.note('Moved ' + this.processesName(moved) + ' to ' + this.label(co) + (moved.length === 1 ? '; it follows its directory' : '; they follow their directories') + ' again once that changes');
      },
      // The same thing again in `co`: a terminal, Claude Code, or the worktree's own copy of the configuration.
      startProcessesIn: function (co, sources) {
        var self = this;
        var started = sources.filter(function (c) { return c.co !== co && self.isRunMissingIn(c, co) === false; }).map(function (c) {
          if (c.kind === 'run') { self.requestRun(co, c.folder, c.key, 'tree'); return c; }
          var fresh = self.addConsole(co, c.kind === 'agent' ? 'agent' : 'shell', []);
          if (c.kind === 'agent') fresh.lines.push('✻ Welcome to Claude Code', '', '  cwd: ' + self.checkout(co).path);
          self.openConsole(fresh);
          return c;
        });
        this.toggled = this.toggled.filter(function (t) { return t !== 'w:' + co && t !== 'wf:' + co + '/processes'; });
        this.note('Started ' + this.counted(started.length, 'process', 'processes') + ' in ' + this.label(co) + '; the originals keep running');
      },

      // ---- the Conflicts tab ----
      openConflicts: function (plan) {
        var co = this.checkout(plan.op.to.co);
        this.openTab({ key: 'cf:' + (++this.conflictCount), kind: 'conflicts', title: (plan.op.verb === 'move' ? 'Move' : 'Copy') + ' conflicts',
          co: co.id, project: co.project, plan: plan });
      },
      conflictRows: function (plan) { return plan.rows.filter(function (r) { return r.reason !== null; }); },
      cleanRows: function (plan) { return plan.rows.filter(function (r) { return r.reason === null; }); },
      conflictActions: function (row) {
        if (row.reason === 'twice' || row.reason === 'kind') return ACTIONS.filter(function (a) { return a[0] !== 'replace'; });
        return ACTIONS;
      },
      conflictReason: function (row) {
        if (row.reason === 'twice') return 'also coming from ' + this.label(row.first.co) + ' · ' + row.first.path.slice(1);
        if (row.reason === 'kind') return row.there.children !== undefined ? 'a folder of that name is there' : 'a file of that name is there';
        if (row.reason === 'same') return 'same content';
        var incoming = row.entry.text.split('\n');
        var there = row.there.text.split('\n');
        return 'differs: +' + linesOnlyIn(incoming, there) + ' −' + linesOnlyIn(there, incoming) + ' lines';
      },
      entrySize: function (entry) {
        if (entry === null) return '';
        if (entry.children !== undefined) return 'folder';
        return this.counted(entry.text.split('\n').length, 'line', 'lines') + ' · ' + new TextEncoder().encode(entry.text).length + ' B';
      },
      // Sets every conflict to `action`, where it is offered.
      setAllConflicts: function (plan, action) {
        this.conflictRows(plan).forEach(function (row) {
          if (this.conflictActions(row).some(function (a) { return a[0] === action; })) row.action = action;
        }, this);
      },
      undecided: function (plan) { return this.conflictRows(plan).filter(function (r) { return r.action === null; }).length; },
      // The name Keep both gives, after the rows above it took theirs.
      keptName: function (plan, row) {
        var self = this;
        var taken = [];
        var taker = function (p) { return taken.includes(p) || self.findFile(plan.op.to.co, p) !== null; };
        plan.rows.slice(0, plan.rows.indexOf(row)).filter(function (r) { return r.action === 'both'; }).forEach(function (r) { taken.push(freeName(r.dest, taker)); });
        return nameOf(freeName(row.dest, taker));
      },
      conflictSummary: function (plan) {
        var rows = plan.rows;
        var count = function (test) { return rows.filter(test).length; };
        var parts = [
          [count(function (r) { return r.reason === null; }), plan.op.verb === 'move' ? 'moves' : 'copies'],
          [count(function (r) { return r.action === 'replace'; }), 'replaces'],
          [count(function (r) { return r.action === 'both' && r.reason !== null; }), 'keeps both of'],
          [count(function (r) { return r.action === 'skip'; }), 'skips']
        ].filter(function (p) { return p[0] > 0; }).map(function (p) { return p[1] + ' ' + p[0]; });
        var undecided = this.undecided(plan);
        return (parts.join(', ') || 'nothing') + (undecided ? ' · ' + undecided + ' to choose' : '');
      },
      conflictTitle: function (plan) {
        var op = plan.op;
        var from = this.sourceCheckouts(op.sources).map(this.label, this).join(', ');
        return (op.verb === 'move' ? 'Move ' : 'Copy ') + this.counted(plan.rows.length, 'file', 'files') + ' from ' + from + ' to ' + this.placeName(op.to);
      },
      conflictLayout: function (plan) {
        if (plan.op.layout === 'flat') return 'flat: every file straight into ' + (plan.op.to.path === '' ? '/' : nameOf(plan.op.to.path) + '/');
        return 'keeping folders, from ' + (plan.base === '' ? '/' : plan.base.slice(1) + '/');
      },
      applyConflicts: function (tab) {
        if (this.undecided(tab.plan) > 0) return;
        this.closeTab(tab);
        this.applyPlan(tab.plan);
      },

      // Mock-only (#tab=conflicts): GEN-817-ive's mock.css and index.html, IDP-30's mock.css, and two files and a
      // folder that go in cleanly, copied flat into dev's internal-apps/ive/mock.
      mockConflicts: function () {
        var gen = 'atlassian@GEN-817-ive';
        var idp = 'atlassian@IDP-30-Reserved-subdomain-returns-404';
        var at = function (co, path, folder) { return { co: co, path: path, folder: folder === true }; };
        this.transfer({ verb: 'copy', layout: 'flat', to: at('atlassian@dev', '/internal-apps/ive/mock', true), sources: [
          at(gen, '/internal-apps/ive/mock/mock.css'), at(gen, '/internal-apps/ive/mock/index.html'), at(idp, '/internal-apps/ive/mock/mock.css'),
          at(gen, '/internal-apps/ive/run.sh'), at(gen, '/internal-apps/ive/Cargo.toml'), at(gen, '/internal-apps/ive/src', true)
        ] });
      }
    }
  };
})();
