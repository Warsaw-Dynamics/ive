#!/usr/bin/env node
// Developer-only dot-compatible wrapper for the pinned prebuilt Graphviz WASM package.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { pathToFileURL } = require("node:url");

const PACKAGE_VERSION = "1.30.0";
const GRAPHVIZ_VERSION = "16.1.0";
const BUNDLE_SHA256 = "77d45a274928484dd1daccd55ab3e41ddda5b4262e29240e205f7188839d9d5d";
const MAX_SOURCE = 256 * 1024;
const MAX_OUTPUT = 16 * 1024 * 1024;

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function packagePaths() {
  const root = process.env.DOT_WASM_PACKAGE_ROOT;
  if (!root) throw new Error("DOT_WASM_PACKAGE_ROOT is required");
  const metadata = path.join(root, "package.json");
  const bundle = path.join(root, "dist", "index.js");
  const info = JSON.parse(fs.readFileSync(metadata, "utf8"));
  if (info.name !== "@hpcc-js/wasm-graphviz" || info.version !== PACKAGE_VERSION) {
    throw new Error(`WASM package identity differs: ${info.name}@${info.version}`);
  }
  const digest = sha256(fs.readFileSync(bundle));
  if (digest !== BUNDLE_SHA256) throw new Error(`WASM bundle SHA-256 differs: ${digest}`);
  return { bundle };
}

function request(args) {
  if (args.length === 1 && args[0] === "-V") return { version: true };
  if (args.length !== 4 || !["-Tjson0", "-Tsvg"].includes(args[0]) || args[1] !== "-o") {
    throw new Error("usage: dot -V | dot -Tjson0|-Tsvg -o OUTPUT INPUT");
  }
  return { format: args[0].slice(2), output: args[2], input: args[3] };
}

async function main() {
  const call = request(process.argv.slice(2));
  const { Graphviz } = await import(pathToFileURL(packagePaths().bundle).href);
  const graphviz = await Graphviz.load();
  const version = graphviz.version();
  if (version !== GRAPHVIZ_VERSION) throw new Error(`embedded Graphviz version differs: ${version}`);
  if (call.version) {
    process.stdout.write(`dot - graphviz version ${version} (hpcc-js WASM ${PACKAGE_VERSION})\n`);
    return;
  }
  const size = fs.statSync(call.input).size;
  if (size < 1 || size > MAX_SOURCE) throw new Error(`DOT source ${call.input} is outside 1..${MAX_SOURCE} bytes`);
  const source = fs.readFileSync(call.input, "utf8");
  const rendered = graphviz.layout(source, call.format, "dot");
  if (typeof rendered !== "string" || Buffer.byteLength(rendered) > MAX_OUTPUT) {
    throw new Error(`Graphviz WASM ${call.format} output exceeds ${MAX_OUTPUT} bytes`);
  }
  fs.writeFileSync(call.output, rendered);
}

main().catch((error) => {
  process.stderr.write(`pinned Graphviz WASM oracle failed: ${error.message}\n`);
  process.exitCode = 1;
});
