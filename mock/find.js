// IVE mock: find in the shown file, scratch, diff or commit (Ctrl+F), a proposal not in the app yet. A box floating at
// the top right of the code (as Chrome and VS Code) with the query, "3 of 12", ▲ ▼, match case,
// whole words, regex and ×; every match marked, the current one stronger, and where they are on a strip by the
// scrollbar. A side-by-side diff searches only the side last clicked. checkouts.js mixes `methods` and `state()`
// into its Alpine component.
(function () {
  'use strict';

  var KINDS = ['file', 'scratch', 'diff', 'commit'];
  var CLOSED = Object.freeze({ open: false, query: '', matchCase: false, word: false, regex: false, at: -1, count: 0, bad: false, marks: [], top: 0, height: 0, side: 'new' });
  // Ranges of the shown tab's matches: kept outside Alpine, whose proxies break DOM ranges.
  var found = [];
  var observer = null;
  var scheduled = false;

  function pattern(f) {
    if (f.query === '') return null;
    var source = f.regex ? f.query : f.query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    if (f.word) source = '\\b' + source + '\\b';
    try { return new RegExp(source, f.matchCase ? 'g' : 'gi'); } catch (e) { return undefined; }
  }

  // A Range over characters start..end of el's text, which its token spans split into several text nodes.
  function textRange(el, start, end) {
    var walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    var range = document.createRange();
    var at = 0;
    var node;
    while ((node = walker.nextNode())) {
      var next = at + node.length;
      if (start >= at && start <= next) range.setStart(node, start - at);
      if (end >= at && end <= next) { range.setEnd(node, end - at); return range; }
      at = next;
    }
    return range;
  }

  function matchesIn(lines, re) {
    var out = [];
    lines.forEach(function (el, line) {
      var text = el.textContent;
      Array.from(text.matchAll(re)).forEach(function (m) {
        if (m[0] !== '') out.push({ line: line, start: m.index, end: m.index + m[0].length, range: textRange(el, m.index, m.index + m[0].length) });
      });
    });
    return out;
  }

  function paint(at) {
    if (typeof CSS === 'undefined' || CSS.highlights === undefined) return;
    CSS.highlights.set('find-hit', new Highlight(...found.filter(function (m, i) { return i !== at; }).map(function (m) { return m.range; })));
    CSS.highlights.set('find-current', new Highlight(...(found[at] ? [found[at].range] : [])));
  }

  function clear() {
    found = [];
    paint(-1);
  }

  // Scrolls the match to a third of the way down, and sideways only when it is out of view.
  function reveal(scroller, range) {
    var r = range.getBoundingClientRect();
    var s = scroller.getBoundingClientRect();
    if (r.top < s.top + 40 || r.bottom > s.bottom) scroller.scrollTop += r.top - s.top - s.height / 3;
    if (r.left < s.left + 60 || r.right > s.right - 20) scroller.scrollLeft += r.left - s.left - s.width / 2;
  }

  // The first match at the top of the view or below it, so typing finds what is on screen first.
  function firstInView(scroller) {
    var top = scroller.getBoundingClientRect().top;
    var i = found.findIndex(function (m) { return m.range.getBoundingClientRect().top >= top; });
    return i < 0 ? 0 : i;
  }

  function wordAt(text, caret) {
    var before = /\w*$/.exec(text.slice(0, caret))[0];
    var after = /^\w*/.exec(text.slice(caret))[0];
    return before + after;
  }

  window.IVE_FIND = {
    methods: {
      findOf: function (tab) { return (tab && tab.find) || CLOSED; },
      // The tab's own find state, made on first use (findOf never writes, so drawing does not).
      ownFind: function (tab) {
        if (tab.find === undefined) tab.find = Object.assign({}, CLOSED, { marks: [] });
        return tab.find;
      },
      canFind: function (tab) { return tab !== null && KINDS.includes(tab.kind); },
      findScroller: function () { return document.querySelector('.pane [data-find-scroll]'); },
      // The searched lines: a file's, or a diff's code cells (side by side: the side last clicked only).
      findLines: function (tab, scroller) {
        if (tab.kind === 'file' || tab.kind === 'scratch') return Array.from(scroller.querySelectorAll('.editor__text'));
        if (this.layoutOf(tab) === 'side') {
          var cell = this.findOf(tab).side === 'old' ? 'first-child' : 'last-child';
          return Array.from(scroller.querySelectorAll('.sbs__cell:' + cell + ' .diff__code'));
        }
        return Array.from(scroller.querySelectorAll('.diff__row .diff__code'));
      },
      // Ctrl+F: the bar opens (or takes focus) with the selection, else the word at the caret, all of it selected.
      openFind: function () {
        var tab = this.currentTab();
        if (this.canFind(tab) === false) return false;
        this.ownFind(tab);
        var picked = this.pickedText(tab.find.open);
        if (picked !== '') tab.find.query = picked;
        tab.find.open = true;
        this.watchCode();
        var self = this;
        this.$nextTick(function () {
          var input = document.querySelector('.pane .findbar__input');
          if (input) { input.focus(); input.select(); }
          self.refind('jump');
        });
        return true;
      },
      pickedText: function (wasOpen) {
        var active = document.activeElement;
        if (active && active.classList.contains('editor__input')) {
          var selected = active.value.slice(active.selectionStart, active.selectionEnd);
          if (selected !== '') return selected.includes('\n') ? '' : selected;
          return wasOpen ? '' : wordAt(active.value, active.selectionStart);
        }
        var text = String(window.getSelection());
        return text.includes('\n') ? '' : text;
      },
      // Esc: the bar closes; in a file the caret is left on the current match, selected.
      closeFind: function (tab) {
        var f = this.findOf(tab);
        var match = found[f.at];
        tab.find.open = false;
        clear();
        var input = document.querySelector('.pane .editor__input');
        if (match === undefined || input === null) return;
        var offset = input.value.split('\n').slice(0, match.line).reduce(function (n, l) { return n + l.length + 1; }, 0) + match.start;
        input.focus({ preventScroll: true });
        input.setSelectionRange(offset, offset + match.end - match.start);
      },
      findStep: function (tab, step) {
        var f = this.findOf(tab);
        if (f.count === 0) return;
        f.at = (f.at + step + f.count) % f.count;
        paint(f.at);
        this.findMarks(tab);
        reveal(this.findScroller(), found[f.at].range);
      },
      // Searches the shown tab again. 'jump': from the view's top, scrolled to; otherwise the current match stays.
      refind: function (how) {
        var tab = this.currentTab();
        var scroller = this.findScroller();
        if (this.findOf(tab).open === false || scroller === null) { clear(); return; }
        var f = tab.find;
        var re = pattern(f);
        f.bad = re === undefined;
        found = re ? matchesIn(this.findLines(tab, scroller), re) : [];
        f.count = found.length;
        f.at = how === 'jump' ? firstInView(scroller) : Math.min(Math.max(f.at, 0), found.length - 1);
        if (found.length === 0) f.at = -1;
        paint(f.at);
        this.findMarks(tab);
        if (how === 'jump' && found[f.at]) reveal(scroller, found[f.at].range);
      },
      // The strip by the scrollbar: each match at its height in the whole text.
      findMarks: function (tab) {
        var scroller = this.findScroller();
        var f = tab.find;
        var top = scroller.getBoundingClientRect().top;
        f.top = scroller.offsetTop;
        f.height = scroller.clientHeight;
        f.marks = found.map(function (m, i) {
          var y = m.range.getBoundingClientRect().top - top + scroller.scrollTop;
          return { at: i, top: (100 * y / scroller.scrollHeight).toFixed(2) + '%' };
        });
      },
      findMarkClick: function (tab, at) {
        tab.find.at = at;
        paint(at);
        this.findMarks(tab);
        reveal(this.findScroller(), found[at].range);
      },
      findToggle: function (tab, option) {
        tab.find[option] = tab.find[option] === false;
        this.refind('jump');
      },
      findCount: function (tab) {
        var f = this.findOf(tab);
        if (f.bad) return 'Bad regex';
        if (f.query === '') return '';
        return f.count === 0 ? 'No results' : (f.at + 1) + ' of ' + f.count;
      },
      findSideText: function (tab) {
        if (tab.kind === 'file' || tab.kind === 'scratch' || this.layoutOf(tab) !== 'side') return '';
        return this.findOf(tab).side === 'old' ? 'left side' : 'right side';
      },
      // A click in a side-by-side diff picks the side Ctrl+F searches.
      findSideClick: function (event) {
        var cell = event.target.closest && event.target.closest('.sbs__cell');
        var tab = this.currentTab();
        if (cell === null || this.canFind(tab) === false) return;
        var side = cell === cell.parentElement.firstElementChild ? 'old' : 'new';
        if (this.findOf(tab).side === side) return;
        this.ownFind(tab).side = side;
        this.refind('keep');
      },
      // Edits, a layout switch or another tab change the code under the bar: search again, after the DOM settles.
      watchCode: function () {
        if (observer !== null) return;
        var self = this;
        observer = new MutationObserver(function (mutations) {
          var outside = mutations.some(function (m) {
            var el = m.target.nodeType === 1 ? m.target : m.target.parentElement;
            return el === null || el.closest('.find') === null;
          });
          if (outside === false || scheduled) return;
          scheduled = true;
          requestAnimationFrame(function () { scheduled = false; self.refind('keep'); });
        });
        observer.observe(document.querySelector('.pane'), { childList: true, subtree: true, characterData: true });
      },
      // Window keys: Ctrl+F (Cmd+F) opens, F3 and Shift+F3 step (and open with the last query).
      findKey: function (event) {
        var tab = this.currentTab();
        if (this.canFind(tab) === false) return;
        var modifier = event.ctrlKey || event.metaKey;
        if (modifier && event.code === 'KeyF' && event.shiftKey === false && event.altKey === false) {
          event.preventDefault();
          this.openFind();
          return;
        }
        if (event.key !== 'F3' || this.findOf(tab).query === '') return;
        event.preventDefault();
        if (tab.find.open === false) { tab.find.open = true; this.watchCode(); }
        var self = this;
        this.$nextTick(function () { self.refind('keep'); self.findStep(tab, event.shiftKey ? -1 : 1); });
      },
      // The query field's keys: Enter and Shift+Enter step, Esc closes, Alt+C, Alt+W, Alt+R toggle.
      findInputKey: function (tab, event) {
        if (event.key === 'Enter') { event.preventDefault(); this.findStep(tab, event.shiftKey ? -1 : 1); return; }
        if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); this.closeFind(tab); return; }
        var option = { KeyC: 'matchCase', KeyW: 'word', KeyR: 'regex' }[event.code];
        if (event.altKey && option) { event.preventDefault(); this.findToggle(tab, option); }
      }
    }
  };
})();
