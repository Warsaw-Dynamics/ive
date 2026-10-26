import type { MainModule, CGraph as CGraphWasm, CSubgraph as CSubgraphWasm } from "../types/graphvizlib.js";
import type { NodeDotAttr, NodeAttrs, EdgeDotAttr, EdgeAttrs, ClusterDotAttr, ClusterAttrs, GraphDotAttr, GraphAttrs, Format, Engine } from "./types.ts";
type AttrValues<T> = Partial<{
    [K in keyof T]: NonNullable<T[K]>;
}>;
/**
 * An edge returned by graph-traversal methods.
 */
export interface EdgeInfo {
    /** The tail (source) node id. */
    tail: string;
    /** The head (target) node id. */
    head: string;
    /**
     * The cgraph edge key that distinguishes parallel edges.
     * Typically `""` for single (non-keyed) edges.
     */
    key: string;
}
/**
 * Example:  Passing a web hosted Image to GraphViz:
 * ```ts
 * import { Graphviz } from "@hpcc-js/wasm-graphviz";
 *
 * const graphviz = await Graphviz.load();
 * const svg = graphviz.layout('digraph { a[image="https://.../image.png"]; }', "svg", "dot", {
 *    images: [{
 *        path: "https://.../image.png",
 *            width: "272px",
 *            height: "92px"
 *    }]
 * });
 * document.getElementById("placeholder").innerHTML = svg;
 * ```
 */
export interface Image {
    /**
     * Full URL to image
     */
    path: string;
    width: string;
    height: string;
}
export interface File {
    path: string;
    data: string;
}
export interface Options {
    images?: Image[];
    files?: File[];
    yInvert?: boolean;
    nop?: number;
}
/**
 * The type of graph to create.
 * - `"directed"` – directed graph (`digraph`)
 * - `"undirected"` – undirected graph (`graph`)
 * - `"strict directed"` – directed graph that disallows parallel edges and self-loops
 * - `"strict undirected"` – undirected graph that disallows parallel edges and self-loops
 */
export type GraphType = "directed" | "undirected" | "strict directed" | "strict undirected";
export interface GraphInit {
    /** The graph id, used as the cgraph name and set as the `id` Graphviz attribute. */
    id?: string;
    type?: GraphType;
    attrs?: AttrValues<GraphAttrs>;
    htmlAttrs?: AttrValues<GraphAttrs>;
}
export interface NodeInit {
    /** The node id, used as the cgraph node name and set as the `id` Graphviz attribute. */
    id: string;
    attrs?: AttrValues<NodeAttrs>;
    htmlAttrs?: AttrValues<NodeAttrs>;
}
export interface EdgeInit {
    tail: string;
    head: string;
    key?: string;
    attrs?: AttrValues<EdgeAttrs>;
    htmlAttrs?: AttrValues<EdgeAttrs>;
}
export interface SubgraphInit {
    /**
     * The subgraph id.  The `cluster_` prefix is added automatically when
     * communicating with the underlying cgraph library; users never need to
     * include it.  The id is also set as the `id` Graphviz attribute on the
     * subgraph.
     */
    id: string;
    attrs?: AttrValues<ClusterAttrs>;
    htmlAttrs?: AttrValues<ClusterAttrs>;
}
export type ClusterInit = SubgraphInit;
/**
 * A subgraph (cluster) inside a {@link Graph}.
 *
 * Obtain via {@link Graph.addSubgraph} or {@link Subgraph.addSubgraph}.  All mutation methods return `this`
 * for chaining.  **Call {@link Subgraph.delete} (or use the `using` keyword)
 * when finished** to free the underlying WASM wrapper — the actual subgraph
 * data is owned by the parent {@link Graph} and is freed with it.
 *
 * Every subgraph is automatically rendered as a cluster (the `cluster_` prefix
 * is added to the id when communicating with the underlying cgraph library).
 * The `id` Graphviz attribute on the subgraph is set to the user-supplied id
 * (without the prefix):
 *
 * ```ts
 * using cluster = graph.addSubgraph("0");
 * cluster
 *   .setAttr("label", "My Cluster")
 *   .setAttr("style", "filled")
 *   .setAttr("color", "lightblue")
 *   .addEdge("a", "b");
 * ```
 */
export declare class Subgraph {
    private _sg;
    /** @internal */
    constructor(sg: CSubgraphWasm);
    /**
     * Create (or find) a node and add it to this subgraph.  The node's `id`
     * Graphviz attribute is automatically set to match `id`.
     */
    addNode(id: string, attrs?: AttrValues<NodeAttrs>, htmlAttrs?: AttrValues<NodeAttrs>): this;
    addNode(init: NodeInit): this;
    /** Alias for {@link addNode}. */
    addVertex(id: string, attrs?: AttrValues<NodeAttrs>, htmlAttrs?: AttrValues<NodeAttrs>): this;
    addVertex(init: NodeInit): this;
    /**
     * Create an edge inside this subgraph.  Both endpoints are created
        * automatically if they do not already exist.
     */
    addEdge(tail: string, head: string, attrs?: AttrValues<EdgeAttrs>, htmlAttrs?: AttrValues<EdgeAttrs>): this;
    addEdge(tail: string, head: string, key: string, attrs?: AttrValues<EdgeAttrs>, htmlAttrs?: AttrValues<EdgeAttrs>): this;
    addEdge(init: EdgeInit): this;
    /**
     * Create (or return an existing) subgraph under this subgraph.  The
     * `cluster_` prefix is added to `id` internally so that layout engines
     * render it as a bounded cluster.  The subgraph's `id` Graphviz attribute
     * is set to the user-supplied `id` (without the prefix).
     */
    addSubgraph(id: string, attrs?: AttrValues<ClusterAttrs>, htmlAttrs?: AttrValues<ClusterAttrs>): Subgraph;
    addSubgraph(init: SubgraphInit): Subgraph;
    applyInit(init: SubgraphInit): this;
    /**
     * Remove a node from this subgraph only.  The node (and any edges
     * connecting it) remains in the root graph and all other subgraphs.
     * No-op if the node is not in this subgraph.
     */
    removeNode(id: string): this;
    /** Alias for {@link removeNode}. */
    removeVertex(id: string): this;
    /**
     * Remove a single edge from this subgraph only.  The edge remains in the
     * root graph and all other subgraphs.  No-op if the edge is not present.
     */
    removeEdge(tail: string, head: string, key?: string): this;
    /**
     * Set an attribute on the subgraph itself (e.g. `"label"`, `"style"`,
     * `"color"`, `"bgcolor"`).  See the official Graphviz
     * [attribute reference](https://graphviz.org/docs/attrs/) for supported
     * attributes and values.
     *
     * When `attr` is a known cluster attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom or less-common attributes.  Omit `value` to reset
     * the attribute to Graphviz's default empty value.  `defaultValue` is
     * passed to Graphviz as the default for this attribute if it has not
     * already been declared.
     */
    setAttr<K extends ClusterDotAttr>(attr: K, value?: NonNullable<ClusterAttrs[K]>, defaultValue?: string | number | boolean): this;
    setAttr(attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setHtmlAttr<K extends ClusterDotAttr>(attr: K, value: NonNullable<ClusterAttrs[K]>, defaultValue?: string | number | boolean): this;
    setHtmlAttr(attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultAttr<K extends ClusterDotAttr>(attr: K, value: NonNullable<ClusterAttrs[K]>): this;
    setDefaultAttr(attr: string, value: string | number | boolean): this;
    setDefaultHtmlAttr<K extends ClusterDotAttr>(attr: K, value: NonNullable<ClusterAttrs[K]>): this;
    setDefaultHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Clear a subgraph-level attribute by resetting it to its default (empty)
     * value.  Equivalent to `setAttr(attr, "")`.
     */
    removeAttr(attr: string): this;
    /**
     * Set an attribute on a node inside this subgraph.
     *
     * When `attr` is a known node attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom attributes.  Omit `value` to reset the attribute to
     * Graphviz's default empty value.  `defaultValue` is passed to Graphviz as
     * the default for this attribute if it has not already been declared.
     */
    setNodeAttr<K extends NodeDotAttr>(node: string, attr: K, value?: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setNodeAttr(node: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    /** Alias for {@link setNodeAttr}. */
    setVertexAttr<K extends NodeDotAttr>(node: string, attr: K, value?: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setVertexAttr(node: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setNodeHtmlAttr<K extends NodeDotAttr>(node: string, attr: K, value: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setNodeHtmlAttr(node: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    /** Alias for {@link setNodeHtmlAttr}. */
    setVertexHtmlAttr<K extends NodeDotAttr>(node: string, attr: K, value: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setVertexHtmlAttr(node: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultNodeAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultNodeAttr(attr: string, value: string | number | boolean): this;
    /** Alias for {@link setDefaultNodeAttr}. */
    setDefaultVertexAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultVertexAttr(attr: string, value: string | number | boolean): this;
    setDefaultNodeHtmlAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultNodeHtmlAttr(attr: string, value: string | number | boolean): this;
    /** Alias for {@link setDefaultNodeHtmlAttr}. */
    setDefaultVertexHtmlAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultVertexHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Clear a node attribute inside this subgraph by resetting it to its
     * default (empty) value.  Equivalent to `setNodeAttr(node, attr, "")`.
     */
    removeNodeAttr(node: string, attr: string): this;
    /** Alias for {@link removeNodeAttr}. */
    removeVertexAttr(node: string, attr: string): this;
    /**
     * Set an attribute on an edge inside this subgraph (identified by
     * `tail`, `head`, and `key`).
     *
     * When `attr` is a known edge attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom attributes.  Omit `value` to reset the attribute to
     * Graphviz's default empty value.  `defaultValue` is passed to Graphviz as
     * the default for this attribute if it has not already been declared.
     */
    setEdgeAttr<K extends EdgeDotAttr>(tail: string, head: string, key: string, attr: K, value?: NonNullable<EdgeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setEdgeAttr(tail: string, head: string, key: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setEdgeHtmlAttr<K extends EdgeDotAttr>(tail: string, head: string, key: string, attr: K, value: NonNullable<EdgeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setEdgeHtmlAttr(tail: string, head: string, key: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultEdgeAttr<K extends EdgeDotAttr>(attr: K, value: NonNullable<EdgeAttrs[K]>): this;
    setDefaultEdgeAttr(attr: string, value: string | number | boolean): this;
    setDefaultEdgeHtmlAttr<K extends EdgeDotAttr>(attr: K, value: NonNullable<EdgeAttrs[K]>): this;
    setDefaultEdgeHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Clear an edge attribute inside this subgraph by resetting it to its
     * default (empty) value.  Equivalent to `setEdgeAttr(tail, head, key, attr, "")`.
     */
    removeEdgeAttr(tail: string, head: string, key: string, attr: string): this;
    /**
     * Returns `true` if a node with the given id exists in this subgraph.
     */
    hasNode(id: string): boolean;
    /** Alias for {@link hasNode}. */
    hasVertex(id: string): boolean;
    /**
     * Returns `true` if an edge from `tail` to `head` exists in this subgraph.
     * Pass `key` to check for a specific parallel edge; omit (or pass `""`) to
     * check for any edge between the two nodes.
     */
    hasEdge(tail: string, head: string, key?: string): boolean;
    /** Returns the number of nodes in this subgraph. */
    nodeCount(): number;
    /** Alias for {@link nodeCount}. */
    vertexCount(): number;
    /** Returns the number of edges in this subgraph. */
    edgeCount(): number;
    /** Returns the degree of the node with the given id in this subgraph. */
    nodeDegree(node: string, inDegree?: number, outDegree?: number): number;
    /** Alias for {@link nodeDegree}. */
    vertexDegree(node: string, inDegree?: number, outDegree?: number): number;
    /**
     * Returns the current value of a subgraph-level attribute, or `""` if
     * the attribute has not been set.
     */
    getAttr(attr: string): string;
    /**
     * Returns the current value of the named attribute on a node in this
     * subgraph, or `""` if the node or attribute does not exist.
     */
    getNodeAttr(node: string, attr: string): string;
    /** Alias for {@link getNodeAttr}. */
    getVertexAttr(node: string, attr: string): string;
    /**
     * Returns the current value of the named attribute on an edge in this
     * subgraph, or `""` if the edge or attribute does not exist.
     * Pass `key = ""` for the first (or only) edge between `tail` and `head`.
     */
    getEdgeAttr(tail: string, head: string, key: string, attr: string): string;
    /**
     * Returns the ids of all nodes in this subgraph (in internal iteration
     * order).
     */
    nodeNames(): string[];
    /** Alias for {@link nodeNames}. */
    vertexNames(): string[];
    /**
     * Returns all edges in this subgraph.  Each unique edge is listed exactly
     * once.
     */
    edges(): EdgeInfo[];
    /**
     * Returns the out-edges of the node with the given id in this subgraph.
     * Returns `[]` if the node does not exist.
     */
    outEdges(node: string): EdgeInfo[];
    /**
     * Returns the in-edges of the node with the given id in this subgraph.
     * Returns `[]` if the node does not exist.
     */
    inEdges(node: string): EdgeInfo[];
    /**
     * Returns all edges incident to the node with the given id in this
     * subgraph (both in and out).  Returns `[]` if the node does not exist.
     */
    nodeEdges(node: string): EdgeInfo[];
    /**
     * Release the WASM wrapper.  The subgraph data itself is freed when the
     * parent {@link Graph} is deleted.
     */
    delete(): void;
    /** @internal – supports the `using` keyword (explicit resource management). */
    [Symbol.dispose](): void;
}
/**
 * A programmatic graph builder backed by the cgraph library.
 *
 * Obtain an instance via {@link Graphviz.createGraph} and call
 * {@link Graph.toDot} to serialise to a DOT string that can be passed to
 * {@link Graphviz.layout} (or any of its convenience wrappers).
 *
 * **Always call {@link Graph.delete} (or use the `using` keyword) when
 * finished** to release the underlying WASM memory.
 *
 * ```ts
 * const graphviz = await Graphviz.load();
 *
 * using graph = graphviz.createGraph("G");
 * graph.addNode("a");
 * graph.addNode("b");
 * graph.addEdge("a", "b");
 * graph.setNodeAttr("a", "color", "red");
 * graph.setEdgeAttr("a", "b", "", "label", "hello");
 * graph.setGraphAttr("rankdir", "LR");
 *
 * const svg = graph.layout(); // render without a DOT round-trip
 * ```
 */
export declare class Graph {
    private _graph;
    private _module;
    /** @internal */
    constructor(g: CGraphWasm, module: MainModule);
    private ensureDefaultFonts;
    /**
     * Create a node with the given `id`.  If a node with this id already
     * exists it is returned unchanged.  The node's `id` Graphviz attribute is
     * automatically set to match.
     */
    addNode(id: string, attrs?: AttrValues<NodeAttrs>, htmlAttrs?: AttrValues<NodeAttrs>): this;
    addNode(init: NodeInit): this;
    /** Alias for {@link addNode}. */
    addVertex(id: string, attrs?: AttrValues<NodeAttrs>, htmlAttrs?: AttrValues<NodeAttrs>): this;
    addVertex(init: NodeInit): this;
    /**
     * Create an edge from `tail` to `head`.  Both nodes are created
        * automatically if they do not already exist.  `key` distinguishes
        * parallel edges between the same pair of nodes; omit (or pass `""`) for
        * an anonymous edge.
     */
    addEdge(tail: string, head: string, attrs?: AttrValues<EdgeAttrs>, htmlAttrs?: AttrValues<EdgeAttrs>): this;
    addEdge(tail: string, head: string, key: string, attrs?: AttrValues<EdgeAttrs>, htmlAttrs?: AttrValues<EdgeAttrs>): this;
    addEdge(init: EdgeInit): this;
    applyInit(init: GraphInit): this;
    /**
     * Replace this graph with one parsed from DOT source using Graphviz cgraph
     * reading support.
     */
    read(dotSource: string): this;
    /**
     * Set a graph-level attribute (e.g. `"rankdir"`, `"label"`, `"bgcolor"`).
     * See the official Graphviz [attribute reference](https://graphviz.org/docs/attrs/)
     * for supported attributes and values.
     *
     * When `attr` is a known graph attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom or less-common attributes.  Omit `value` to reset
     * the attribute to Graphviz's default empty value.  `defaultValue` is
     * passed to Graphviz as the default for this attribute if it has not
     * already been declared.
     */
    setGraphAttr<K extends GraphDotAttr>(attr: K, value?: NonNullable<GraphAttrs[K]>, defaultValue?: string | number | boolean): this;
    setGraphAttr(attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setGraphHtmlAttr<K extends GraphDotAttr>(attr: K, value: NonNullable<GraphAttrs[K]>, defaultValue?: string | number | boolean): this;
    setGraphHtmlAttr(attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultGraphAttr<K extends GraphDotAttr>(attr: K, value: NonNullable<GraphAttrs[K]>): this;
    setDefaultGraphAttr(attr: string, value: string | number | boolean): this;
    setDefaultGraphHtmlAttr<K extends GraphDotAttr>(attr: K, value: NonNullable<GraphAttrs[K]>): this;
    setDefaultGraphHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Set an attribute on a named node (e.g. `"color"`, `"label"`, `"shape"`).
     * The node must exist; call {@link addNode} first if needed.
     *
     * When `attr` is a known node attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom attributes.  Omit `value` to reset the attribute to
     * Graphviz's default empty value.  `defaultValue` is passed to Graphviz as
     * the default for this attribute if it has not already been declared.
     */
    setNodeAttr<K extends NodeDotAttr>(node: string, attr: K, value?: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setNodeAttr(node: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    /** Alias for {@link setNodeAttr}. */
    setVertexAttr<K extends NodeDotAttr>(node: string, attr: K, value?: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setVertexAttr(node: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setNodeHtmlAttr<K extends NodeDotAttr>(node: string, attr: K, value: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setNodeHtmlAttr(node: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    /** Alias for {@link setNodeHtmlAttr}. */
    setVertexHtmlAttr<K extends NodeDotAttr>(node: string, attr: K, value: NonNullable<NodeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setVertexHtmlAttr(node: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultNodeAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultNodeAttr(attr: string, value: string | number | boolean): this;
    /** Alias for {@link setDefaultNodeAttr}. */
    setDefaultVertexAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultVertexAttr(attr: string, value: string | number | boolean): this;
    setDefaultNodeHtmlAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultNodeHtmlAttr(attr: string, value: string | number | boolean): this;
    /** Alias for {@link setDefaultNodeHtmlAttr}. */
    setDefaultVertexHtmlAttr<K extends NodeDotAttr>(attr: K, value: NonNullable<NodeAttrs[K]>): this;
    setDefaultVertexHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Set an attribute on an edge identified by `(tail, head, key)`.
     * Use the same `key` that was passed to {@link addEdge}; pass `""` for
     * anonymous edges.
     *
     * When `attr` is a known edge attribute the value type is inferred
     * automatically.  A generic `string | number | boolean` fallback is
     * provided for custom attributes.  Omit `value` to reset the attribute to
     * Graphviz's default empty value.  `defaultValue` is passed to Graphviz as
     * the default for this attribute if it has not already been declared.
     */
    setEdgeAttr<K extends EdgeDotAttr>(tail: string, head: string, key: string, attr: K, value?: NonNullable<EdgeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setEdgeAttr(tail: string, head: string, key: string, attr: string, value?: string | number | boolean, defaultValue?: string | number | boolean): this;
    setEdgeHtmlAttr<K extends EdgeDotAttr>(tail: string, head: string, key: string, attr: K, value: NonNullable<EdgeAttrs[K]>, defaultValue?: string | number | boolean): this;
    setEdgeHtmlAttr(tail: string, head: string, key: string, attr: string, value: string | number | boolean, defaultValue?: string | number | boolean): this;
    setDefaultEdgeAttr<K extends EdgeDotAttr>(attr: K, value: NonNullable<EdgeAttrs[K]>): this;
    setDefaultEdgeAttr(attr: string, value: string | number | boolean): this;
    setDefaultEdgeHtmlAttr<K extends EdgeDotAttr>(attr: K, value: NonNullable<EdgeAttrs[K]>): this;
    setDefaultEdgeHtmlAttr(attr: string, value: string | number | boolean): this;
    /**
     * Remove a node and all its edges from the graph.  Removing from the root
     * graph also removes the node from every subgraph.
     * No-op if the node does not exist.
     */
    removeNode(id: string): this;
    /** Alias for {@link removeNode}. */
    removeVertex(id: string): this;
    /**
     * Remove a single edge identified by `(tail, head, key)`.
     * Pass `""` for `key` on anonymous edges.
     * No-op if the edge does not exist.
     */
    removeEdge(tail: string, head: string, key?: string): this;
    /**
     * Dissolve the cluster boundary for the subgraph with the given `id`.
     * Nodes and edges that belonged to the subgraph remain in the parent graph.
     * No-op if no subgraph with that id exists.
     */
    removeSubgraph(id: string): this;
    /**
     * Clear a graph-level attribute by resetting it to its default (empty)
     * value.  Equivalent to `setGraphAttr(attr, "")`.
     */
    removeGraphAttr(attr: string): this;
    /**
     * Clear a node attribute by resetting it to its default (empty) value.
     * Equivalent to `setNodeAttr(node, attr, "")`.
     */
    removeNodeAttr(node: string, attr: string): this;
    /** Alias for {@link removeNodeAttr}. */
    removeVertexAttr(node: string, attr: string): this;
    /**
     * Clear an edge attribute by resetting it to its default (empty) value.
     * Equivalent to `setEdgeAttr(tail, head, key, attr, "")`.
     */
    removeEdgeAttr(tail: string, head: string, key: string, attr: string): this;
    /**
     * Returns `true` if a node with the given id exists in the graph.
     */
    hasNode(id: string): boolean;
    /** Alias for {@link hasNode}. */
    hasVertex(id: string): boolean;
    /**
     * Returns `true` if an edge from `tail` to `head` exists in the graph.
     * Pass `key` to check for a specific parallel edge; omit (or pass `""`) to
     * check for any edge between the two nodes.
     */
    hasEdge(tail: string, head: string, key?: string): boolean;
    /**
     * Returns `true` if a subgraph with the given id exists.
     */
    hasSubgraph(id: string): boolean;
    /** Returns the number of nodes in the graph. */
    nodeCount(): number;
    /** Alias for {@link nodeCount}. */
    vertexCount(): number;
    /** Returns the number of edges in the graph. */
    edgeCount(): number;
    /** Returns the number of direct subgraphs of this graph. */
    subgraphCount(): number;
    /** Returns the degree of the node with the given id in the graph. */
    nodeDegree(node: string, inDegree?: number, outDegree?: number): number;
    /** Alias for {@link nodeDegree}. */
    vertexDegree(node: string, inDegree?: number, outDegree?: number): number;
    /**
     * Returns the current value of a graph-level attribute, or `""` if the
     * attribute has not been set.
     */
    getGraphAttr(attr: string): string;
    /**
     * Returns the current value of the named attribute on a node, or `""` if
     * the node or attribute does not exist.
     */
    getNodeAttr(node: string, attr: string): string;
    /** Alias for {@link getNodeAttr}. */
    getVertexAttr(node: string, attr: string): string;
    /**
     * Returns the current value of the named attribute on an edge, or `""` if
     * the edge or attribute does not exist.
     * Pass `key = ""` for the first (or only) edge between `tail` and `head`.
     */
    getEdgeAttr(tail: string, head: string, key: string, attr: string): string;
    /**
     * Returns the ids of all direct subgraphs (the `cluster_` prefix is
     * stripped from the underlying cgraph names).
     */
    subgraphNames(): string[];
    /**
     * Returns the ids of all nodes in the graph (in internal iteration order).
     */
    nodeNames(): string[];
    /** Alias for {@link nodeNames}. */
    vertexNames(): string[];
    /**
     * Returns all edges in the graph.  Each unique edge is listed exactly
     * once.
     */
    edges(): EdgeInfo[];
    /**
     * Returns the out-edges of the node with the given id.  Returns `[]` if
     * the node does not exist.
     */
    outEdges(node: string): EdgeInfo[];
    /** Alias for {@link outEdges}. */
    outEdgesFrom(node: string): EdgeInfo[];
    /**
     * Returns the in-edges of the node with the given id.  Returns `[]` if
     * the node does not exist.
     */
    inEdges(node: string): EdgeInfo[];
    /** Alias for {@link inEdges}. */
    inEdgesTo(node: string): EdgeInfo[];
    /**
     * Returns all edges incident to the node with the given id (both in and
     * out).  Returns `[]` if the node does not exist.
     */
    nodeEdges(node: string): EdgeInfo[];
    /** Alias for {@link nodeEdges}. */
    vertexEdges(node: string): EdgeInfo[];
    /**
     * Create (or return an existing) subgraph with the given `id`.  The
     * `cluster_` prefix is added to `id` internally so that layout engines
     * render it as a bounded cluster.  The subgraph's `id` Graphviz attribute
     * is set to the user-supplied `id` (without the prefix).
     *
     * Returns a {@link Subgraph} wrapper.  **Call {@link Subgraph.delete} (or
     * use the `using` keyword) when finished** to free the WASM wrapper.  The
     * subgraph data itself is owned by this graph.
     *
     * ```ts
     * using cluster = graph.addSubgraph("0");
     * cluster.setAttr("label", "My Cluster").addEdge("a", "b");
     * ```
     */
    addSubgraph(id: string, attrs?: AttrValues<ClusterAttrs>, htmlAttrs?: AttrValues<ClusterAttrs>): Subgraph;
    addSubgraph(init: SubgraphInit): Subgraph;
    /**
     * Look up an existing subgraph by id without creating a new one.
     * Returns a {@link Subgraph} wrapper if found, or `null` if it does not
     * exist.  **Call {@link Subgraph.delete} (or use the `using` keyword)
     * when finished** to free the WASM wrapper.
     *
     * ```ts
     * using sg = graph.getSubgraph("0");
     * if (sg) {
     *     console.log(sg.nodeNames());
     * }
     * ```
     */
    getSubgraph(id: string): Subgraph | null;
    /**
     * Render the graph directly to the specified format without first
     * serialising to DOT.  Equivalent to:
     * ```ts
     * graphviz.layout(graph.toDot(), outputFormat, layoutEngine, options)
     * ```
     * but avoids the DOT round-trip.
     *
     * @param outputFormat The output format (default `"svg"`).
     * @param layoutEngine The layout engine to use (default `"dot"`).
     * @param options       Optional images / extra files for the renderer.
     * @returns The rendered output as a string.
     */
    layout(outputFormat?: Format, layoutEngine?: Engine, options?: Options): string;
    /**
     * Serialise the graph to a DOT-language string.
     */
    write(): string;
    /**
     * Serialise the graph to a DOT-language string.
     */
    toDot(): string;
    /**
     * Release the underlying WASM object.  Must be called when the graph is
     * no longer needed (or use the `using` keyword with TypeScript ≥ 5.2).
     */
    delete(): void;
    /** @internal – supports the `using` keyword (explicit resource management). */
    [Symbol.dispose](): void;
}
/**
 * The Graphviz layout algorithms take descriptions of graphs in a simple text language, and make diagrams in useful formats, such as images and SVG for web pages or display in an interactive graph browser.
 *
 * Graphviz has many useful features for concrete diagrams, such as options for colors, fonts, tabular node layouts, line styles, hyperlinks, and custom shapes.
 *
 * See [graphviz.org](https://graphviz.org/) for more details.
 *
 * ### Rendering from a DOT string
 * ```ts
 * import { Graphviz } from "@hpcc-js/wasm/graphviz";
 *
 * const graphviz = await Graphviz.load();
 *
 * const dot = "digraph G { Hello -> World }";
 * const svg = graphviz.dot(dot);
 * ```
 *
 * ### Programmatic graph construction
 * ```ts
 * import { Graphviz } from "@hpcc-js/wasm/graphviz";
 *
 * const graphviz = await Graphviz.load();
 *
 * using graph = graphviz.createGraph("G");
 * graph
 *   .addNode("a")
 *   .addNode("b")
 *   .addEdge("a", "b")
 *   .setNodeAttr("a", "color", "red")
 *   .setGraphAttr("rankdir", "LR");
 *
 * const svg = graph.layout(); // render without a DOT round-trip
 * ```
 *
 * ### Online Demos
 * * https://raw.githack.com/hpcc-systems/hpcc-js-wasm/main/index.html
 * * https://observablehq.com/@gordonsmith/graphviz
 */
export declare class Graphviz {
    private _module;
    private constructor();
    /**
     * Compiles and instantiates the raw wasm.
     *
     * ::: info
     * In general WebAssembly compilation is disallowed on the main thread if the buffer size is larger than 4KB, hence forcing `load` to be asynchronous;
     * :::
     *
     * @returns A promise to an instance of the Graphviz class.
     */
    static load(): Promise<Graphviz>;
    /**
     * Unloades the compiled wasm instance.
     */
    static unload(): void;
    /**
     * Unloads the compiled wasm instance, equivalent to {@link Graphviz.unload}.
     * Disposing an instance that has already been unloaded is a no-op.
     */
    dispose(): void;
    [Symbol.dispose](): void;
    /**
     * @returns The Graphviz c++ version
     */
    version(): string;
    /**
     * Performs layout for the supplied _dotSource_, see [The DOT Language](https://graphviz.gitlab.io/doc/info/lang.html) for specification.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param layoutEngine The type of layout to perform.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    layout(dotSource: string, outputFormat?: Format, layoutEngine?: Engine, options?: Options): string;
    /**
      * acyclic is a filter that takes a directed graph as input and outputs a copy of the graph with sufficient edges reversed to make the graph acyclic. The reversed edge inherits all of the attributes of the original edge. The optional file argument specifies where the input graph is stored; by default.
      *
      * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
      * @param doWrite Enable output is produced, though the return value will indicate whether the graph is acyclic or not.
      * @param verbose Print information about whether the file is acyclic, has a cycle or is undirected.
      * @returns `{ acyclic: boolean, num_rev: number, outFile: string }` `acyclic` will be true if a cycle was found, `num_rev` will contain the number of reversed edges and `outFile` will (optionally) contain the output.
      */
    acyclic(dotSource: string, doWrite?: boolean, verbose?: boolean): {
        acyclic: boolean;
        num_rev: number;
        outFile: string;
    };
    /**
      * tred computes the transitive reduction of directed graphs, and prints the resulting graphs to standard output.  This removes edges implied by transitivity. Nodes and subgraphs are not otherwise affected. The ‘‘meaning’’ and validity of the reduced graphs is application dependent. tred is particularly useful as a preprocessor to dot to reduce clutter in dense layouts.  Undirected graphs are silently ignored.
      *
      * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
      * @param verbose Print additional information.
      * @param printRemovedEdges Print information about removed edges.
      * @returns `{ out: string, err: string }`.
      */
    tred(dotSource: string, verbose?: boolean, printRemovedEdges?: boolean): {
        out: string;
        err: string;
    };
    /**
     * unflatten is a preprocessor to dot that is used to improve the aspect ratio of graphs having many leaves or disconnected nodes. The usual layout for such a graph is generally very wide or tall. unflatten inserts invisible edges or adjusts the minlen on edges to improve layout compaction.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param maxMinlen The minimum length of leaf edges is staggered between 1 and len (a small integer).
     * @param do_fans Enables the staggering of the -maxMinlen option to fanout nodes whose indegree and outdegree are both 1.  This helps with structures such as a -> \{w x y \} -> b. This option only works if the -maxMinlen flag is set.
     * @param chainLimit Form disconnected nodes into chains of up to len nodes.
     * @returns A string containing the "unflattened" dotSource.
     */
    unflatten(dotSource: string, maxMinlen?: number, do_fans?: boolean, chainLimit?: number): string;
    /**
     * Convenience function that performs the **circo** layout, is equivalent to `layout(dotSource, outputFormat, "circo");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    circo(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **dot** layout, is equivalent to `layout(dotSource, outputFormat, "dot");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    dot(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **fdp** layout, is equivalent to `layout(dotSource, outputFormat, "fdp");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    fdp(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **sfdp** layout, is equivalent to `layout(dotSource, outputFormat, "sfdp");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    sfdp(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **neato** layout, is equivalent to `layout(dotSource, outputFormat, "neato");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    neato(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **osage** layout, is equivalent to `layout(dotSource, outputFormat, "osage");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    osage(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **patchwork** layout, is equivalent to `layout(dotSource, outputFormat, "patchwork");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    patchwork(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **twopi** layout, is equivalent to `layout(dotSource, outputFormat, "twopi");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @param outputFormat The format of the result.
     * @param options Advanced Options for images, files, yInvert and nop.
     * @returns A string containing the calculated layout in the format specified by `outputFormat`
     */
    twopi(dotSource: string, outputFormat?: Format, options?: Options): string;
    /**
     * Convenience function that performs the **nop** layout, is equivalent to `layout(dotSource, "dot", "nop");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @returns A string containing the "pretty printed" dotSource.
     */
    nop(dotSource: string): string;
    /**
     * Convenience function that performs the **nop2** layout, is equivalent to `layout(dotSource, "dot", "nop2");`.
     *
     * @param dotSource Required - graph definition in [DOT](https://graphviz.gitlab.io/doc/info/lang.html) language
     * @returns A string containing the "pretty printed" dotSource.
     */
    nop2(dotSource: string): string;
    /**
     * Programmatically create a graph using the cgraph library.
     *
     * Returns a {@link Graph} builder that lets you add nodes, edges, and
     * attributes without writing DOT source by hand.  The graph's `id`
     * Graphviz attribute is automatically set to match `id`.  Call
     * {@link Graph.layout} to render directly, or {@link Graph.toDot} to
     * serialise to DOT.
     *
     * **You must call {@link Graph.delete} when finished** to free the
     * underlying WASM memory, or use the `using` keyword (TypeScript ≥ 5.2).
     *
     * @param id    The graph id (default `"G"`).
     * @param type  The graph type (default `"directed"`).
     *
     * ```ts
     * using graph = graphviz.createGraph("G");
     * graph
     *   .addNode("a")
     *   .addNode("b")
     *   .addEdge("a", "b")
     *   .setNodeAttr("a", "color", "red")
     *   .setGraphAttr("rankdir", "LR");
     *
     * const svg = graph.layout(); // render without a DOT round-trip
     * ```
     */
    createGraph(id?: string, type?: GraphType): Graph;
    createGraph(init: GraphInit): Graph;
    /**
     * Parse DOT source into a mutable {@link Graph}.  The returned graph can
     * be queried, modified, rendered directly, or serialised back to DOT with
     * {@link Graph.toDot}.
     */
    read(dotSource: string): Graph;
}
export {};
