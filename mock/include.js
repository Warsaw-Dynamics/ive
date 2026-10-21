/**
 * x-include — HTML partials with slots, for pawbooking mocks.
 *
 * Splits a mock screen into reusable files. Alpine does the reactivity; this
 * adds the two things Alpine has no answer for — loading a partial from a file,
 * and slots.
 *
 *
 * SETUP
 *
 * One tag in <head>. It loads Alpine for you, so do not add Alpine yourself.
 * Do not put `defer` on it.
 *
 *   <script src="./include.js"></script>
 *
 * Serve over HTTP — fetch is blocked on file:// URLs:
 *
 *   python3 -m http.server
 *
 *
 * USING A PARTIAL
 *
 * `x-include` says which file; the enclosing `x-data` is what it can see.
 *
 *   <div x-include="components/property-card.html"
 *        x-data="{ hotel: someHotel, nights: 2 }"></div>
 *
 * Inside the partial, use those values like any other Alpine scope:
 *
 *   <h4 x-text="hotel.name"></h4>
 *   <span x-text="(hotel.price * nights) + ' zł'"></span>
 *
 * Change `nights` on the page and every card re-prices itself. Nothing is
 * re-rendered and nothing inside the partial is lost — an open dropdown stays
 * open, a focused field stays focused.
 *
 * Paths are relative to the PAGE, not to the partial doing the including. A
 * partial inside components/ still writes "components/header.html".
 *
 *
 * WRITING A PARTIAL
 *
 * Markup, plus an optional <script> if it needs behaviour of its own. One file:
 *
 *   <!-- components/property-card.html -->
 *   <article class="property-card" x-data="propertyCard(hotel)">
 *     <button @click="toggleSaved()" :aria-pressed="saved">♥</button>
 *     <h4 x-text="hotel.name"></h4>
 *     <button @click="$dispatch('select', { id: hotel.id })">See availability</button>
 *   </article>
 *
 *   <script>
 *     document.addEventListener('alpine:init', function () {
 *       Alpine.data('propertyCard', function (hotel) {
 *         return {
 *           saved: !!hotel.saved,
 *           toggleSaved: function () { this.saved = !this.saved; }
 *         };
 *       });
 *     });
 *   </script>
 *
 * The script runs once no matter how many cards are on the page; the factory
 * runs once per card, each with its own `saved`.
 *
 * Watch out for `:disabled="hotel.soldOut"` when a record simply omits the key.
 * Alpine treats undefined as truthy for boolean attributes, and every button
 * ends up disabled. Write `:disabled="!!hotel.soldOut"`.
 *
 *
 * GETTING SOMETHING BACK OUT
 *
 * A partial reports events with `$dispatch`; the page listens with `@`:
 *
 *   <div x-include="components/property-card.html" x-data="{ hotel: h }"
 *        @select="open($event.detail.id)"></div>
 *
 * Events bubble, so one listener on a container catches every card in it.
 *
 *
 * SLOTS
 *
 * A partial marks holes with <slot>; anything inside <slot> is the fallback
 * shown when the page supplies nothing:
 *
 *   <!-- components/page.html -->
 *   <div class="shell">
 *     <header><slot name="lede"><p>A pawbooking mock screen.</p></slot></header>
 *     <slot></slot>
 *   </div>
 *
 * The page fills them with <template>. `slot="name"` matches a named hole; a
 * plain <template> fills the unnamed one:
 *
 *   <div x-include="components/page.html" x-data="{ title: 'Results' }">
 *     <template slot="lede"><p class="shell__lede">Stays in Warszawa.</p></template>
 *     <template>
 *       <section class="section">…the page…</section>
 *     </template>
 *   </div>
 *
 * Slot content MUST be wrapped in <template>. Bare children are ignored, with a
 * warning in the console.
 *
 *
 * LISTS
 *
 * Ordinary Alpine. Use <template x-for>, which leaves no wrapper element behind,
 * so CSS like `.list-group__item:last-child` keeps working:
 *
 *   <template x-for="hotel in visible" :key="hotel.id">
 *     <div x-include="components/property-card.html"></div>
 *   </template>
 *
 * Filtering the list adds and removes cards; each new one renders on its own.
 *
 *
 * WHEN SOMETHING IS WRONG
 *
 * A missing file or an include cycle prints a red strip in the page itself, not
 * only in the console — naming the file, and for a cycle the full trail. Opened
 * over file://, the page says so instead of coming up blank.
 *
 * If a partial is only ever reached through a computed path, load it up front so
 * its Alpine.data() is registered in time:
 *
 *   Include.preload('components/property-card.html');
 *
 *
 * See sandbox.html for all of the above on one page, and selftest.html for the
 * behaviour asserted as tests.
 */
(function () {
  'use strict';

  var ALPINE_SRC = './vendor/alpine.min.js';
  var REVEAL_TIMEOUT = 3000;

  var templates = new Map(); // src -> <template>, scripts already stripped and run
  var pending = new Map();   // src -> in-flight Promise
  var errors = new Map();    // src -> message
  var hoisted = new Set();   // <script src> URLs already added to <head>

  // Hosts are hidden until filled so a partial cannot flash in half-rendered.
  var style = document.createElement('style');
  style.textContent =
    '[x-include]:not([data-include-ready]) { visibility: hidden; }\n' +
    '.include-error { display: block; margin: 4px 0; padding: 8px 12px; ' +
    'border: 1px solid #c1121f; border-radius: 6px; background: #fff0f1; ' +
    'color: #7a0b16; font: 500 13px/1.4 ui-monospace, monospace; }';
  document.head.appendChild(style);

  /**
   * A quoted string or a bare path is resolved up front, which is what lets the
   * pre-pass walk the whole template graph before Alpine exists. Anything else
   * is a live expression and can only be evaluated at directive time.
   */
  function staticSrc(expression) {
    if (!expression) return null;
    var quoted = /^\s*(['"])(.*)\1\s*$/.exec(expression);
    if (quoted) return quoted[2];
    if (/^[\w./-]+$/.test(expression.trim())) return expression.trim();
    return null;
  }

  function fail(src, message) {
    errors.set(src, message);
    console.error('x-include: ' + src + ' — ' + message);
  }

  function runScripts(tpl, src) {
    tpl.content.querySelectorAll('script').forEach(function (script) {
      if (script.src) {
        var url = new URL(script.getAttribute('src'), location.href).href;
        if (!hoisted.has(url)) {
          hoisted.add(url);
          var tag = document.createElement('script');
          tag.src = url;
          document.head.appendChild(tag);
        }
      } else {
        try {
          // Indirect eval: global scope, so `Alpine.data(…)` registrations and
          // any helper functions the partial declares are visible everywhere.
          // sourceURL keeps the partial debuggable by name in devtools.
          (0, eval)(script.textContent + '\n//# sourceURL=' + src);
        } catch (e) {
          console.error('x-include: script error in ' + src, e);
        }
      }
      script.remove();
    });
  }

  /**
   * Cannot recurse forever: each template is cached BEFORE its own nested
   * includes are walked, so a cycle terminates here on the second visit. That
   * is also why the cycle guard lives at render time instead — see render().
   */
  function load(src) {
    if (templates.has(src) || errors.has(src)) return Promise.resolve();
    if (pending.has(src)) return pending.get(src);

    var job = (function () {
      return fetch(src)
        .then(function (resp) {
          if (!resp.ok) throw new Error(resp.status + ' ' + resp.statusText);
          return resp.text();
        })
        .then(function (html) {
          var tpl = document.createElement('template');
          tpl.innerHTML = html;
          runScripts(tpl, src);
          templates.set(src, tpl);
          return Promise.all(nestedSources(tpl.content).map(load));
        })
        .catch(function (e) {
          fail(src, e.message);
        });
    })();

    pending.set(src, job);
    return job;
  }

  /**
   * Recurses into <template> content on purpose: a card used inside
   * `<template x-for>` is invisible to a plain querySelectorAll, and missing it
   * would mean its `Alpine.data()` is unregistered when the list first renders.
   */
  function nestedSources(root) {
    var found = [];
    collectSources(root, found);
    return found;
  }

  function collectSources(root, found) {
    root.querySelectorAll('[x-include]').forEach(function (el) {
      var src = staticSrc(el.getAttribute('x-include'));
      if (src) found.push(src);
    });
    root.querySelectorAll('template').forEach(function (tpl) {
      collectSources(tpl.content, found);
    });
  }

  function projectSlots(fragment, host, src) {
    var named = new Map();
    var fallback = null;

    Array.prototype.forEach.call(host.children, function (child) {
      if (child.tagName !== 'TEMPLATE') {
        console.warn('x-include: ' + src + ' — slot content must be wrapped in ' +
                     '<template>; ignoring', child);
        return;
      }
      var name = child.getAttribute('slot');
      if (name) {
        named.set(name, child.content);
        return;
      }
      fallback = child.content;
    });

    fragment.querySelectorAll('slot').forEach(function (slot) {
      var name = slot.getAttribute('name');
      var content = name ? named.get(name) : fallback;
      if (content) {
        slot.replaceWith(content.cloneNode(true));
        return;
      }
      // Nothing supplied — whatever the partial put inside <slot> stands.
      slot.replaceWith.apply(slot, Array.prototype.slice.call(slot.childNodes));
    });
  }

  function showError(host, message) {
    var box = document.createElement('span');
    box.className = 'include-error';
    box.textContent = 'x-include: ' + message;
    host.replaceChildren(box);
    host.setAttribute('data-include-ready', '');
  }

  function insert(host, src) {
    if (errors.has(src)) {
      showError(host, src + ' — ' + errors.get(src));
      return;
    }

    var tpl = templates.get(src);
    if (!tpl) return;

    var Alpine = window.Alpine;
    var fragment = tpl.content.cloneNode(true);
    projectSlots(fragment, host, src);

    // mutateDom keeps Alpine's own observer from initialising these nodes a
    // second time. Insert first, then initTree — the scope is resolved by
    // walking up from the node, so it has to be in place already.
    Alpine.mutateDom(function () {
      var nodes = Array.prototype.slice.call(fragment.childNodes);
      host.replaceChildren(fragment);
      host.setAttribute('data-include-src', src);
      host.setAttribute('data-include-ready', '');
      nodes.forEach(function (node) {
        if (node.nodeType === Node.ELEMENT_NODE) Alpine.initTree(node);
      });
    });
  }

  /**
   * Walks up the rendered tree collecting the partials this host sits inside.
   * This is where cycles actually bite: every inserted partial re-runs the
   * directive, so a → b → a expands until the tab dies. The loading pass cannot
   * see it, because it stops on its own cache long before the DOM grows.
   */
  function includeAncestry(host) {
    var chain = [];
    for (var node = host.parentElement; node; node = node.parentElement) {
      var src = node.getAttribute('data-include-src');
      if (src) chain.push(src);
    }
    return chain;
  }

  function render(host, src) {
    var chain = includeAncestry(host);
    if (chain.indexOf(src) !== -1) {
      // Not recorded in `errors` — the cycle is a property of this position in
      // the tree, not of the partial, which may be fine everywhere else.
      var trail = chain.slice().reverse().concat(src).join(' → ');
      console.error('x-include: include cycle: ' + trail);
      showError(host, 'include cycle: ' + trail);
      return;
    }

    if (templates.has(src) || errors.has(src)) {
      insert(host, src);
      return;
    }
    // Only reached by a partial that no static scan could find — a runtime
    // `x-include` built from a live expression.
    load(src).then(function () {
      insert(host, src);
    });
  }

  document.addEventListener('alpine:init', function () {
    window.Alpine.directive('include', function (el, meta, utils) {
      var src = staticSrc(meta.expression);
      if (src === null) src = String(utils.evaluate(meta.expression));
      render(el, src);
    });
  });

  function banner(message) {
    var box = document.createElement('div');
    box.className = 'include-error';
    box.style.margin = '16px';
    box.textContent = message;
    document.body.prepend(box);
  }

  function start() {
    if (location.protocol === 'file:') {
      banner('x-include needs HTTP — fetch is blocked on file:// URLs. ' +
             'Run  python3 -m http.server  in this directory and reload.');
      document.querySelectorAll('[x-include]').forEach(function (el) {
        el.setAttribute('data-include-ready', '');
      });
      return;
    }

    // A broken partial must never leave the page invisible.
    setTimeout(function () {
      document.querySelectorAll('[x-include]:not([data-include-ready])')
        .forEach(function (el) { el.setAttribute('data-include-ready', ''); });
    }, REVEAL_TIMEOUT);

    Promise.all(nestedSources(document).map(function (src) {
      return load(src);
    })).then(function () {
      var tag = document.createElement('script');
      tag.src = ALPINE_SRC;
      document.head.appendChild(tag);
    });
  }

  // Public surface: preload a partial only reachable from a live expression, so
  // that `x-data` referring to its component registration resolves in time.
  window.Include = {
    preload: function (src) { return load(src); },
    templates: templates,
    errors: errors
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
