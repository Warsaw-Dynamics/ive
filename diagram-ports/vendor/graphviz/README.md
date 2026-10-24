# Graphviz reference source

`graphviz-16.1.0.tar.xz` is the unmodified [official Graphviz 16.1.0 source release](https://graphviz.org/download/source/), SHA-256 `0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3`. `COPYING` is copied byte-for-byte from its root and retains the Eclipse Public License 2.0. The archive also retains upstream copyright notices and component licenses.

This archive is a developer reference for the in-process Rust DOT parser, layered layout, and differential tests. It is not a Cargo dependency, build input, or runtime asset. The Rust parser and layout are original implementations; they do not link Graphviz code. Exact geometry and pixel parity are tracked in `../../../dot-progress.md`.
