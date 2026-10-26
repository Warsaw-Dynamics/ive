// TypeScript bindings for emscripten-generated code.  Automatically generated at compile time.
declare var RuntimeExports: {
    /**
     * Given a pointer 'ptr' to a null-terminated UTF8-encoded string in the
     * emscripten HEAP, returns a copy of that string as a Javascript String object.
     *
     * @param {number} ptr
     * @param {number=} maxBytesToRead - An optional length that specifies the
     *   maximum number of bytes to read. You can omit this parameter to scan the
     *   string until the first 0 byte. If maxBytesToRead is passed, and the string
     *   at [ptr, ptr+maxBytesToReadr[ contains a null byte in the middle, then the
     *   string will cut short at that byte index.
     * @param {boolean=} ignoreNul - If true, the function will not stop on a NUL character.
     * @return {string}
     */
    UTF8ToString: (ptr: number, maxBytesToRead?: number | undefined, ignoreNul?: boolean | undefined) => string;
    lengthBytesUTF8: (str: any) => number;
    stringToUTF8: (str: any, outPtr: any, maxBytesToWrite: any) => any;
    /** @type {!Uint8Array} */
    HEAPU8: Uint8Array;
};
interface WasmModule {
  _malloc(_0: number): number;
  _free(_0: number): void;
}

type EmbindString = ArrayBuffer|Uint8Array|Uint8ClampedArray|Int8Array|string;
export interface ClassHandle {
  isAliasOf(other: ClassHandle): boolean;
  delete(): void;
  deleteLater(): this;
  isDeleted(): boolean;
  // @ts-ignore - If targeting lower than ESNext, this symbol might not exist.
  [Symbol.dispose](): void;
  clone(): this;
}
export interface CGraphviz extends ClassHandle {
  acyclic_num_rev: number;
  get layout_result(): string;
  set layout_result(value: EmbindString);
  get acyclic_outFile(): string;
  set acyclic_outFile(value: EmbindString);
  get tred_out(): string;
  set tred_out(value: EmbindString);
  get tred_err(): string;
  set tred_err(value: EmbindString);
  get unflatten_out(): string;
  set unflatten_out(value: EmbindString);
  createFile(_0: EmbindString, _1: EmbindString): void;
  layout(_0: EmbindString, _1: EmbindString, _2: EmbindString): string;
  acyclic(_0: EmbindString): boolean;
  acyclic(_0: EmbindString, _1: boolean): boolean;
  acyclic(_0: EmbindString, _1: boolean, _2: boolean): boolean;
  tred(_0: EmbindString): void;
  tred(_0: EmbindString, _1: boolean): void;
  tred(_0: EmbindString, _1: boolean, _2: boolean): void;
  unflatten(_0: EmbindString): string;
  unflatten(_0: EmbindString, _1: number): string;
  unflatten(_0: EmbindString, _1: number, _2: boolean): string;
  unflatten(_0: EmbindString, _1: number, _2: boolean, _3: number): string;
}

export interface CGraph extends ClassHandle {
  get dot_out(): string;
  set dot_out(value: EmbindString);
  get layout_result(): string;
  set layout_result(value: EmbindString);
  nodeCount(): number;
  edgeCount(): number;
  subgraphCount(): number;
  addNode(_0: EmbindString): void;
  addEdge(_0: EmbindString, _1: EmbindString): void;
  addEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setGraphAttr(_0: EmbindString, _1: EmbindString): void;
  setGraphAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setGraphHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setGraphHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): void;
  setNodeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): void;
  setEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString): void;
  setEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString, _5: EmbindString): void;
  setEdgeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString): void;
  setEdgeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString, _5: EmbindString): void;
  setDefaultGraphAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultGraphHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultNodeAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultNodeHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultEdgeAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultEdgeHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  read(_0: EmbindString): boolean;
  addSubgraph(_0: EmbindString): CSubgraph | null;
  getSubgraph(_0: EmbindString): CSubgraph | null;
  removeNode(_0: EmbindString): void;
  removeEdge(_0: EmbindString, _1: EmbindString): void;
  removeEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  removeSubgraph(_0: EmbindString): void;
  hasNode(_0: EmbindString): boolean;
  hasEdge(_0: EmbindString, _1: EmbindString): boolean;
  hasEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): boolean;
  hasSubgraph(_0: EmbindString): boolean;
  nodeDegree(_0: EmbindString): number;
  nodeDegree(_0: EmbindString, _1: number): number;
  nodeDegree(_0: EmbindString, _1: number, _2: number): number;
  getGraphAttr(_0: EmbindString): string;
  getNodeAttr(_0: EmbindString, _1: EmbindString): string;
  getEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): string;
  nodeNames(): string;
  subgraphNames(): string;
  edges(): string;
  outEdges(_0: EmbindString): string;
  inEdges(_0: EmbindString): string;
  nodeEdges(_0: EmbindString): string;
  write(): string;
  toDot(): string;
  layout(_0: EmbindString, _1: EmbindString): string;
}

export interface CSubgraph extends ClassHandle {
  nodeCount(): number;
  edgeCount(): number;
  addNode(_0: EmbindString): void;
  addEdge(_0: EmbindString, _1: EmbindString): void;
  addEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  addSubgraph(_0: EmbindString): CSubgraph | null;
  setAttr(_0: EmbindString, _1: EmbindString): void;
  setAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): void;
  setNodeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  setNodeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): void;
  setEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString): void;
  setEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString, _5: EmbindString): void;
  setEdgeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString): void;
  setEdgeHtmlAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString, _4: EmbindString, _5: EmbindString): void;
  setDefaultAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultNodeAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultNodeHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultEdgeAttr(_0: EmbindString, _1: EmbindString): void;
  setDefaultEdgeHtmlAttr(_0: EmbindString, _1: EmbindString): void;
  removeNode(_0: EmbindString): void;
  removeEdge(_0: EmbindString, _1: EmbindString): void;
  removeEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): void;
  hasNode(_0: EmbindString): boolean;
  hasEdge(_0: EmbindString, _1: EmbindString): boolean;
  hasEdge(_0: EmbindString, _1: EmbindString, _2: EmbindString): boolean;
  nodeDegree(_0: EmbindString): number;
  nodeDegree(_0: EmbindString, _1: number): number;
  nodeDegree(_0: EmbindString, _1: number, _2: number): number;
  getAttr(_0: EmbindString): string;
  getNodeAttr(_0: EmbindString, _1: EmbindString): string;
  getEdgeAttr(_0: EmbindString, _1: EmbindString, _2: EmbindString, _3: EmbindString): string;
  nodeNames(): string;
  edges(): string;
  outEdges(_0: EmbindString): string;
  inEdges(_0: EmbindString): string;
  nodeEdges(_0: EmbindString): string;
}

interface EmbindModule {
  CGraphviz: {
    new(): CGraphviz;
    new(_0: number): CGraphviz;
    new(_0: number, _1: number): CGraphviz;
    version(): string;
    lastError(): string;
  };
  CGraph: {
    new(): CGraph;
    new(_0: EmbindString): CGraph;
    new(_0: EmbindString, _1: number): CGraph;
    new(_0: EmbindString, _1: number, _2: number): CGraph;
  };
  CSubgraph: {};
}

export type MainModule = WasmModule & typeof RuntimeExports & EmbindModule;
export default function MainModuleFactory (options?: unknown): Promise<MainModule>;
