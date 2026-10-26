# Browser Graphviz package

This directory is the unmodified npm package `@hpcc-js/wasm-graphviz` version 1.30.0, fetched from `https://registry.npmjs.org/@hpcc-js/wasm-graphviz/-/wasm-graphviz-1.30.0.tgz` (tarball SHA-256 `0105defc8548e241a08a22e40ee0499a851402c345e38745a5b4d521d776d57f`). Its bundled Graphviz reports version 16.1.0. `dist/index.js` has SHA-256 `77d45a274928484dd1daccd55ab3e41ddda5b4262e29240e205f7188839d9d5d` and embeds its WASM bytes. The package's TypeScript source, source map, notices and Apache-2.0 `LICENSE` are retained. Graphviz's EPL-2.0 source and license are retained separately in `vendor/graphviz/`.

`web/diagram-worker.js` imports `dist/index.js`, awaits `Graphviz.load()`, and calls `layout(dot, "json0", "dot")` synchronously when the Rust engine asks for graph positions. The package files are not modified.
