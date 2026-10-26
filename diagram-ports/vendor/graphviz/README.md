# Bundled Graphviz source

`graphviz-16.1.0.tar.xz` is the unmodified [official Graphviz 16.1.0 source release](https://graphviz.org/download/source/), SHA-256 `0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3`. `COPYING` is copied byte-for-byte from its root and retains the Eclipse Public License 2.0. The archive also retains upstream copyright notices and component licenses.

`viewer/build.rs` verifies the archive hash, extracts it into Cargo's build directory, configures a static `dot_static` with optional image/font/plugin dependencies disabled, and embeds that helper in `diagram-viewer`. The helper still uses ordinary platform libraries such as libc, libstdc++ and zlib; no installed Graphviz executable or Graphviz shared library is used at runtime. The viewer writes the embedded helper to a private temporary directory, executes it as a separate process and removes the directory after the render.

Graphviz source and modifications to it remain available under EPL-2.0 regardless of IVE's separate source-available license. The archive is unmodified; the Rust adapter and packaging code are separate files. This build path is exercised on Linux. macOS native build and execution require validation, and Windows is not supported by this vendored Autotools build.
