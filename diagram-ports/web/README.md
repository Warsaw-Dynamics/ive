# Diagram web renderer

`diagram-web` compiles the native diagram engine to browser WASM. `diagram.js` loads it in a dedicated Web Worker. It accepts PlantUML or standalone DOT and returns SVG text or PNG bytes; DOT can also return JSON geometry.

```sh
rustup target add wasm32-unknown-unknown
./web/build.sh
cd web && python3 -m http.server 8000
```

Open `http://localhost:8000/example.html`. The `web/` directory alone contains the runtime files after the build. Serve the files over HTTP; `file://` cannot load module workers and WASM reliably.

In IVE, run **Build PlantUML web renderer** once, then **PlantUML web example** from this worktree's run configurations. The static server opens `/example.html` on port 8765.

Run `node --test web/smoke.test.mjs` from the workspace root after building to check the WASM ABI, all 58 PlantUML fixtures, PNG export and invalid-input reporting.

```js
import { createDiagramRenderer } from "./diagram.js";

const renderer = await createDiagramRenderer();
const source = "@startuml\nAlice -> Bob : hello\n@enduml";
const svg = await renderer.render(source, { format: "svg" }); // string
const png = await renderer.render(source, { format: "png" }); // Uint8Array
const dotSvg = await renderer.render("digraph G { a -> b }", { language: "dot" });
const dotGeometry = await renderer.render("digraph G { a -> b }", { language: "dot", format: "json" });
renderer.close();
```

`createDiagramRenderer({ wasmUrl })` accepts a URL for the built `diagram_web.wasm`; by default it loads `web/pkg/diagram_web.wasm`. Each renderer owns one worker. The source limit is 256 KiB, the SVG limit is 16 MiB, and PNG images are bounded to 4096 pixels per axis and 4,194,304 total pixels. Render requests have a 10-second deadline; a timed-out worker is terminated and the renderer must be created again. Errors reject the Promise. The caller can put SVG text into a Blob with `image/svg+xml`, or PNG bytes into a Blob with `image/png`.

The worker uses the same in-process Rust layered graph layout as the desktop viewer. PNG is rasterized by resvg inside Rust WASM with the bundled OFL Liberation fonts, then encoded as PNG bytes; browser fonts do not affect PNG pixels. SVG text remains text and uses the consumer's fonts when displayed. This requires a browser with module workers and WebAssembly. A worker keeps rendering off the main UI thread; it is not a separate OS process or memory sandbox.
