<!-- CEP:FILE: spec/volume-13-ir1.md
CEP:WHY: design.md §20 (lines 1845–2158) requires a complete normative definition of IR 1, the effect-typed Sea of Nodes: node inventory, graph representation, memory SSA, effect tokens, type metadata, verifier, and the pass catalog with per-pass legality. This volume is that definition; every §20 bullet has exactly one home here.
CEP:STATUS: complete
CEP:FAILURE: An IR 1 graph that fails any verifier check V1–V18 of §13.11 is rejected; a pass that emits a graph violating invariant I1–I12 is non-conforming and must be corrected, not the invariant bent. An omitted §20 bullet in the coverage ledger is a missing definition.
CEP:ASSUMES: SPEC-INDEX.md §1 (method), §3 (settled decisions: UB enumeration, provenance model, effect atoms), §5.3 (effect algebra), §5.7 (pinned graph math, embedded verbatim in §13.2); design.md §19 (graph infrastructure) and §40 (10-row pass template); standards.md §3 normative verbs; verification standards.md G1/G2/G4/G5/G11/G13/G14/G15/G21; Volumes 04 (types), 05 (value/memory semantics), 07 (effects), 10 (ABI/unsafe), 11 (semantic handoff), 12 (limits, interning).
CEP:COST: document. It fixes compile-time costs: verifier O(n·(1+p)+m) per graph (§13.11), each pass costed in §13.12; node/edge limits charged against the Volume 12 limits table.
CEP:EVIDENCE: Verification matrix (OB-13-1..OB-13-18) and coverage ledger (bullets in scope: 216) at the end of this file.
CEP:FILE: spec/volume-13-ir1.md -->

# Volume 13 — IR 1: Effect-typed Sea of Nodes

## 13.0 Scope and traceability

This volume defines design.md §20 "IR 1 complete specification", lines 1845–2158 of design.md, including the section header prose (lines 1845–1852) and the deliverables line (line 2155). It contains 216 bullets, distributed over subsections §20.1–§20.13. Mapping:

| design.md subsection | Lines | Bullets | Defined in |
| --- | --- | --- | --- |
| 20.1 IR 1 philosophy | 1853–1868 | 12 | §13.1 |
| 20.2 Graph representation | 1870–1886 | 15 | §13.2 |
| 20.3 Control nodes | 1888–1916 | 22 (16 nodes + 6 per-node facets) | §13.3 |
| 20.4 Value nodes | 1918–1937 | 18 | §13.4 |
| 20.5 Memory nodes | 1939–1960 | 20 | §13.5 |
| 20.6 Memory SSA | 1962–1981 | 14 | §13.6 |
| 20.7 Effect nodes | 1983–2001 | 14 | §13.7 |
| 20.8 Type representation | 2002–2014 | 9 | §13.8 |
| 20.9 Call nodes | 2016–2028 | 11 | §13.9 |
| 20.10 Ownership information | 2030–2044 | 9 | §13.10 |
| 20.11 IR 1 verification | 2046–2064 | 15 | §13.11 |
| 20.12 IR 1 transformations | 2066–2136 | 45 | §13.12 |
| 20.13 IR 1 optimization legality | 2138–2153 | 12 | §13.13 |

Bullets in scope: 216, defined: 216 (ledger at end of file; the ledger additionally maps 1 prose requirement — l2000 effect-token propagation, combination, removal). The deliverable line of §20 (line 2155) names six deliverables; each maps to a section of this volume: IR 1 Semantics Specification = §13.1, §13.8, §13.9; Node Schema = §13.3–§13.5; Memory SSA Specification = §13.6; Effect Graph Specification = §13.7; Pass Catalog = §13.12; Verifier Specification = §13.11.

## 13.1 IR 1 philosophy (§20.1)

| Bullet | Normative definition |
| --- | --- |
| Purpose | IR 1 is the first optimizing representation of the Planck compiler. It must preserve source-language semantics exactly as defined by Volumes 04–07 and expose data, control, memory, and effect dependencies as explicit graph edges. It must not make representation decisions: no layout, no register mapping, no target opcodes. |
| Semantic guarantees | An IR 1 graph G for function f must satisfy: for every execution path P of f that terminates without resource exhaustion, the observable behavior of P under Volume 05/07 semantics must equal the observable behavior of the corresponding graph execution. Observable behavior = returned value, memory contents of live allocations at exit, I/O effects, panic/unwind events, and traps. UB domain (SPEC-INDEX §3 UB-1..UB-6) must be preserved: a graph execution must not move a program out of the UB set nor into it. |
| Supported operations | The closed node inventory of §13.3–§13.5 and §13.7. Every source-language operation of Volumes 01–10 must lower to a finite composition of these nodes; the lowering table is Volume 11 §18.14's handoff consumed by the constructor (§13.2). Any operation not representable is a frontend or volume bug, never an IR extension done ad hoc; extending the inventory is an edition change with a verifier update. |
| Type preservation | Every value-producing node carries a semantic type (§13.8) and must satisfy the type schema of its opcode (check V2/V3). IR 1 must never erase a type distinction that Volume 04 makes; it must never introduce a distinction Volume 04 does not make. |
| Graph model | A directed multigraph with typed edges (§13.2): Value, Control, Effect, Memory edge classes, Sea-of-Nodes style (operations float free of control except where a class of edge pins them). |
| Control model | Explicit control subgraph rooted at Start and ending at terminal nodes (§13.3). Control flow is a subgraph, not block structure; there are no basic blocks in IR 1. |
| Memory model | Memory SSA over the effect chain and per-partition memory states (§13.6). Provenance (allocation ID + offset + capability set) is tracked on every pointer value per SPEC-INDEX §3. |
| Effect model | One effect token chain per function (§13.7); effect rows are sets of effect atoms per SPEC-INDEX §5.3; each effectful node carries its row; ordering is the chain. |
| Ownership metadata | Ownership facts are metadata annotations on nodes and edges, not extra executable nodes, except the Drop node and ProvenanceCheck node (§13.10). This settles design.md's observation that "static metadata and graph annotations may be more appropriate": it is normative here. |
| Transformation permissions | A transformation may be applied to a graph G iff it preserves observable behavior (G5), preserves the effect row of every executed path, preserves panic and trap events, preserves ownership obligations, preserves debug mappings (§13.13), and yields a graph that passes V1–V18. |
| Transformation prohibitions | The following are banned in IR 1: target-specific reasoning of any kind; layout or representation decisions; reordering of effectful operations across each other except as licensed by §13.12 pass rows; weakening a provenance capability set on a value that reaches a dereference without a ProvenanceCheck; introducing undefined behavior that the input did not contain; dropping a Drop obligation; changing a call's effect summary to a strictly smaller row (strictly larger is permitted). |
| Target independence | IR 1 graphs must be target-independent: the same source function must produce the same graph modulo node IDs for every target. Target-specific decisions begin in IR 2 (Volume 14). Node IDs must be assigned by a deterministic construction order (§13.2), so IR 1 is bit-identical across hosts (G14). |

## 13.2 Graph representation (§20.2)

**Ownership and structure.** Graph ownership: a `Graph` is owned by exactly one `Function` instance; the function arena owns all nodes. Node ownership: nodes are owned by the graph arena; edges are stored as node references inside fixed input slots of the consumer node; no shared ownership, no reference counting. Node identity: `NodeId` is a `u32`; ID 0 is the null sentinel; IDs are assigned monotonically in construction order starting at 1 and must not be reused while the graph exists. Node ID width is 32 bits; a graph must not exceed `MAX_NODES_IR1 = 2^20` nodes and `MAX_EDGES_IR1 = 2^22` edges (limits recorded in the Volume 12 limits table; exceeding them is error `LimitExceeded`).

**Edge classes.** Input edges: an ordered, fixed-arity list of input slots per node, each slot typed by the node's schema (§13.3–§13.5). Output edges: there is no stored output-edge list; a node's outputs are its use-list entries, maintained by the graph infrastructure (design.md §19.3): every input slot insert registers a use record `(consumer, slot_index)` on the producer. Value edges: edges of class `Value` in Value slots; they carry typed data and must form a DAG when combined with Effect and Memory edges (I2, check V1). Control edges: edges of class `Control` in Control slots; they form the control subgraph, which may contain cycles only through Loop nodes (V4). Effect edges: edges of class `Effect` carrying the effect token (§13.7). Memory edges: edges of class `Memory` carrying `MemState` values (§13.6); they are the memory-SSA def-use edges and are distinct from Effect edges; a node that is both effectful and memory-writing consumes the Effect token and produces both a new Effect token and new MemState values for each written partition.

**Ordering, entry, exit.** Dependency ordering: all Value, Effect, and Memory edges point from producer to consumer and, excluding Phi backedge inputs (§13.3, §13.6, §13.7), must be acyclic (check V1). The canonical total order over nodes is the topological order of SPEC-INDEX §5.7 (Kahn's algorithm with stable ascending-NodeId tie-break) over this acyclic data subgraph; the scheduler (§13.12) refines it with control anchors. Graph entry: exactly one `Start` node per graph. Graph exit: exactly one terminal node per graph, which must be `Return`, `Trap`, or `PanicExit` (I1). Dead nodes: a node is dead iff it is not transitively value-, effect-, memory-, or control-reachable from a live root, where the live roots are the graph terminal node, all `Param` nodes, and all nodes with metadata `MustKeep`. Unreachable nodes: a node is control-unreachable iff there is no directed Control path from `Start` to a Region containing it as anchored, or for floating nodes, if its anchor is unreachable. A verified graph must contain no dead and no unreachable nodes (checks V14/V15); passes that would create them must remove them before verification (DCE, §13.12). Graph regions: IR 1 has no nested-region container; a region is an analysis product: given the dominator tree, the region builder must produce (a) loop regions: for each Loop header h, the set of nodes dominated by h that are not dominated by any exit projection of h; and (b) linear regions: maximal single-entry subgraphs between loop regions. Regions are recomputed after control-changing passes; they are never stored as ownership containers.

**Pinned graph math.** The following contract of SPEC-INDEX §5.7 is embedded verbatim and is normative for IR 1 and for the `planck-graph-math` kernel crate:

> - Graphs are static arrays: nodes `0..n`, edges as CSR pairs; `n` bounded by the compiler limits table (Volume 12).
> - Topological order: Kahn's algorithm with stable tie-break by ascending node ID (determinism, G14); failure on cycle = error, never a partial order.
> - Dominators: iterative dataflow over reverse postorder; entry dominates itself; result unique for reducible and irreducible graphs; termination measure = ≤ n passes with early exit (G11 obligation).
> - Cycle detection: three-color DFS, iterative, bounded stack; reports one cycle as a node list.

IR 1 must use this math through `planck-graph-math` only; re-implementing topological order, dominators, or cycle detection inside the IR is banned (G16, TCB discipline). The extraction certificate of `planck-graph-math` (EXH/Z3 per §5.1) is the evidence for OB-13-3..OB-13-6.

## 13.3 Control nodes (§20.3)

The node inventory of IR 1 is closed. The complete inventory is: control nodes of the table below, value nodes of §13.4, memory nodes of §13.5, and effect nodes `EffectChain`, `EffectPhi`, `MemPhi`, `Drop` (§13.5–§13.7), plus `ProvenanceCheck` (§13.5). The 16 control entries of design.md map onto the table; "Merge" is defined as a synonym of `Region` (the graph stores `Region` only); "Loop header" is the `Loop` node itself; "Loop backedge" and "Loop exit" are the `Loop` node's second control input and its `ExitProj` outputs. `Suspend` (§13.7), `LandingPad`, `PanicExit`, and `Trap` complete the inventory. For every node below, the columns Inputs, Outputs, Semantics, Dominance behavior, Control projection behavior, Legal graph transformations are the six §20.3 facets; a facet not stated for a node is "none".

| Node | Inputs | Outputs | Semantics | Dominance | Projections | Legal transformations |
| --- | --- | --- | --- | --- | --- | --- |
| Start | none | Ctrl out; one Value `Param` per parameter; Effect token out; MemState out per incoming partition | Function entry; parameters become Param nodes (§13.4) | Dominates all reachable nodes | none | May gain outputs only at construction; never removed |
| If | Ctrl in; Value cond: bool | 2 Ctrl outs (TrueProj, FalseProj) | Branches on cond | Dominates both successors | TrueProj, FalseProj by slot index | If cond is constant c, fold: replace If by the taken projection wired to c's branch (V13-branch) |
| TrueProj | Ctrl (from If) | Ctrl out | True arm control | Dominates the true arm | none | Becomes the sole input of the true-arm Region on folding |
| FalseProj | Ctrl (from If) | Ctrl out | False arm control | Dominates the false arm | none | Symmetric to TrueProj |
| Switch | Ctrl in; Value index: integer | k+1 Ctrl outs: arm 0..k−1 plus DefaultProj | k must satisfy `2 ≤ k ≤ MAX_SWITCH_ARMS = 2^16`; arms are compile-time constants of the index type, pairwise distinct, no gaps | Dominates each arm | ArmProj(i), DefaultProj | Constant index folds to the taken arm; arm set may be renumbered preserving semantics |
| Region | ≥ 1 Ctrl ins | Ctrl out; Phi inputs attach here | Join point of control paths; input i corresponds to predecessor i (order fixed by slot index) | Dominated by all preds; dominates all users of its outputs | none | Single-pred Region must be eliminated by rewiring (V6); pred deletion rewires phis |
| Phi (at Region/Loop) | Ctrl (implicit via Region); ≥ 2 Value/MemState/Effect ins, one per pred | 1 output of the phi type | Selects input i on entry from pred i | Dominated by Region; all phi inputs' defs must dominate their corresponding pred's end (V5/V6) | none | Identity phi (all inputs same producer) must be replaced by the producer |
| Loop | 2 Ctrl ins: entry, backedge | Ctrl out; ExitProjs (0..n); phi inputs attach | Loop header: executes entry path then backedge path repeatedly; a Loop without ExitProj is legal only if the function returns the never type or every reachable body path ends in Trap/PanicExit/Unreachable (V15) | Dominates the loop body = all nodes dominated by Loop except those dominated by an ExitProj | ExitProj(i) | Peeling/unrolling/unswitching must keep header arity 2 and phi correspondence (V6) |
| Loop backedge | (role of Loop input 2) | — | Source must be a control node dominated by the Loop header (V4) | — | — | Rotation preserves the entry/backedge correspondence of attached phis |
| Loop exit | (role of Loop ExitProj) | Ctrl out | Exit path control | Dominates the exit path | none | May be guarded by If when unswitching |
| Return | Ctrl in; Effect in; MemState in per live partition; Value ins = return tuple | terminal | Returns values; graph exit for returning functions | Terminates a path | none | Exactly one per graph (I1) |
| Trap | Ctrl in; Effect in | terminal | Defined termination: abort with exit code 134 (SPEC-INDEX §3); emitted for UB-4/UB-5/UB-6 sites reached in unsafe code and for resource-exhaustion traps | Terminates a path | none | Must not be removed; may not be reordered across effects |
| Panic | Ctrl in; Effect in; Value payload | Ctrl out (to PanicExit or enclosing LandingPad region) | Raises panic with payload; unwinds to the enclosing catch boundary (Volume 07) | Dominates its unwind target region | none | Panic with constant payload may be canonicalized; must not be deleted unless the path is dead |
| PanicExit | Ctrl in (from one or more Panic or UnwindProj) | terminal | Function-level exceptional exit for panics not caught in-function | Terminates a path | none | Exactly one per graph when any Panic or unwinding Call exists (I1) |
| LandingPad | Ctrl in (unwind edges only) | Ctrl out; Value payload out | Entry of a catch region; binds the panic payload | Dominates the catch body | none | Only reachable via unwind edges (V4) |
| Invoke | Ctrl in; Value callee; args; Effect in; MemState in | Ctrl outs NormalProj, UnwindProj | A Call (§13.9) that can unwind to a catch boundary inside this function; must be used exactly then, otherwise plain Call is required (V4) | Dominates both projections | NormalProj, UnwindProj | Devirtualized/inlined Invoke keeps both projections until catch-free proof passes replace it by Call |
| Unreachable | Ctrl in | terminal | Marks a path proven impossible by the frontend; execution reaching it is a compiler bug, not a defined event | Terminates a path | none | DCE must remove Unreachable nodes whose inputs are also dead |
| Suspend | Ctrl in; Effect in; MemState in | Ctrl out; Effect out; MemState out | Await suspension point (Volume 08): saves state, yields control; resume continues at the following region | Dominates the resume region | none | Must not be moved across memory or effect operations |

Per-node facet rule (§20.3's six bullets): the table above is the complete per-node definition of Inputs, Outputs, Semantics, Dominance behavior, Control projection behavior, and Legal graph transformations for every control node; no other control node may exist (V1).

## 13.4 Value nodes (§20.4)

Value nodes produce data. The §20.4 bullets map to the following closed opcode set. Tuple operations, array operations, and range operations are the aggregate opcodes applied to tuple/array/range types; they are listed explicitly so the inventory stays closed.

| Opcode(s) | Bullet | Inputs | Output | Semantics |
| --- | --- | --- | --- | --- |
| `Const(T, v)` | Constants | none | type T | Compile-time constant of scalar type T; v in the value space of T (Volume 02 §2; overflow at literal level is a frontend rejection). |
| `Param(i)` | Parameters | none | type of param i | i-th function parameter; exists only as Start outputs. |
| `Add,Sub,Mul,Div,Rem,Neg,AddW,SubW,MulW` (per width 8/16/32/64/128, signed/unsigned) | Arithmetic | 1–2 values | same width | Wrapped forms (`*W`) wrap per Volume 05; non-W forms panic on overflow (atom `panic`). Div/Rem by zero: defined trap for integers. |
| `Cmp{Eq,Ne,Lt,Le,Gt,Ge}` | Comparisons | 2 values of one type | bool | Per Volume 05 ordering; float comparisons per IEEE-754 totalOrder-free semantics with NaN ≠ NaN. |
| `And,Or,Xor,Shl,ShrL,ShrA,RotL,RotR,Not,PopCnt,Clz,Ctz,ByteSwap,BitRev` | Bitwise operations | 1–2 integers | same width | Exact bitwise semantics per Volume 05; shift ≥ width is a defined trap. |
| `Sext,Zext,Trunc,F2I,I2F,F2F,Int2Int,Ptr2Int,Int2Ptr,Transmute` | Conversions | 1 value | target type | Per Volume 05 conversion rules; `Transmute` requires equal size and is `unsafe` (atom `unsafe`). |
| `AggNew(T)` | Aggregate construction | field values | T | Constructs a struct/array/tuple value of semantic type T with fields in declaration order (layout not decided). |
| `AggGet(T, f)` | Aggregate extraction; Tuple operations; Array operations; Range operations | 1 value of T | field type | Extracts field f by name/index; for arrays f is an index with a bounds fact (V13); for tuples f is the slot; for ranges f ∈ {start, end}. |
| `AggSet(T, f)` | Aggregate insertion; Tuple operations; Array operations | value of T, field value | T | Functional update producing a new T (IR 1 values are immutable; in-place update is a memory operation). |
| `EnumNew(T, variant)` | Enum construction | payload values | T | Constructs variant `variant` of enum T. |
| `Discriminant(T)` | Discriminant extraction | 1 value of T | discriminant type | Produces the semantic discriminant (an abstract value; representation encoding is an IR 2 decision). |
| `RangeNew,RangeGet` | Range operations | — | — | Covered by AggNew/AggGet on range types; listed for closure. |
| `ArrayLen` | Array operations | 1 array value | usize | Length of a fixed-size array (constant); slice length uses `SliceLen` = AggGet on the slice pair. |
| `FnValue(sym)` | Function values | none | fn type | Reference to function symbol sym; `sym` is a SymId (Volume 12). |
| `ClosureValue(fn, env)` | Closure values | fn value, env aggregate | closure type | Closure pairing function and environment; env may be the empty aggregate. |
| `Call` | Function calls | callee + args + Effect + MemState | return values | Defined fully in §13.9. |
| `IntrinsicCall(id)` | Intrinsics | per-intrinsic schema | per-intrinsic | Planck intrinsic id from Volume 10's intrinsic table; carries its declared precondition and effect row (UB-6 if violated). |
| `ProvenanceCheck(p, caps)` | (§13.5 support) | pointer p, capability demand | p | Emits a runtime provenance check for unsafe-derived pointers; output has the demanded capabilities. |

Type schema rule: every opcode above has a fixed input/output type schema in the node schema table of the implementation; V2/V3 check each instance against it after monomorphization substitution (§13.8).

## 13.5 Memory nodes (§20.5)

Memory nodes read or write memory, form addresses, or manage allocations. Columns: memory behavior (Effect and MemState handling), provenance rule (per SPEC-INDEX §3), effect atoms.

| Opcode | Bullet | Memory/state behavior | Provenance rule | Effect atoms |
| --- | --- | --- | --- | --- |
| `Load(place, mem)` | Load | Consumes MemState of place's partition; produces loaded value; does not consume the Effect token (pure read) | Address must carry `load` capability, else verifier error `ProvenanceViolation` unless guarded by ProvenanceCheck | ∅ (row may include `nondet` for racing loads) |
| `Store(val, place, mem, chain)` | Store | Consumes Effect token; produces new token and new MemState for the written partition | Address must carry `store` capability | {alloc} if val contains a new allocation, else ∅ |
| `RawLoad/RawStore` | Raw load / Raw store | As Load/Store but on raw bytes with explicit width w ∈ {1,2,4,8,16} and no type-based aliasing; partition = unknown | Capability demand is `compute`+`load`/`store`; raw access is `unsafe` | {unsafe} |
| `Init(place, mem, chain)` | Initialization | Transitions a place from uninitialized to initialized in the init-fact metadata (§13.6); writes the value | as Store | ∅ |
| `MemCopy(dst, src, len, mem, chain)` | Copy memory | Copies len bytes src→dst; produces MemState updates for both partitions; src/dst may overlap only if declared `CopyNonOverlapping` is absent — overlapping is UB-1-adjacent and rejected in safe lowering | Both addresses need load/store capabilities | {alloc} if len is dynamic |
| `MemMove(...)` | Move memory | As MemCopy with ownership-transfer metadata (source becomes MovedFrom) | as MemCopy | ∅ |
| `MemClear(place, len, mem, chain)` | Memory clear | Zero-fills; new MemState | store capability | ∅ |
| `Alloc(kind, size, align, chain)` | Memory allocation | Produces pointer value with fresh allocation ID; produces MemState for its partition; consumes Effect token | Provenance = (fresh alloc id, offset 0, {load, store, compute}) | {alloc} |
| `Dealloc(ptr, chain)` | Memory deallocation | Produces MemState marking partition dead; consumes Effect token | Requires compute capability; pointer capability set narrowed to {compute} afterwards | {alloc} |
| `AddrOf(place, mem)` | Address formation | Pure address computation; yields pointer to place with provenance derived from the place's base | (base alloc id, offset of place, {load, store, compute}) | ∅ |
| `PtrOffset(p, d)` | Pointer offset | Pure arithmetic on provenance | (alloc id(p), offset(p) + d, caps(p) ∩ {load, compute, store·in-bounds}): store capability is kept iff d is a compile-time-known constant with 0 ≤ offset+d ≤ size; otherwise dropped (narrowing on arithmetic) | ∅ |
| `PtrCmp(a, b, op)` | Pointer comparison | Pure | Compares (alloc id, offset) pairs; equality across different alloc ids is false, never UB | ∅ |
| `VolatileLoad/VolatileStore` | Volatile load / Volatile store | As Load/Store but marked volatile: no elimination, no reordering, width w ∈ {1,2,4,8} | capability demanded | {io} |
| `AtomicLoad(op, ord)` / `AtomicStore(op, ord)` | Atomic load / Atomic store | ord ∈ {Relaxed, Acquire, Release, SeqCst}; participate in the concurrency memory model (Volume 08) | capability demanded | {concurrency} |
| `AtomicRmw(op, ord)` | Atomic read-modify-write | Reads, computes op ∈ {Add,Sub,And,Or,Xor,Exchange,Min,Max}, writes; produces old value | capability demanded | {concurrency} |
| `CmpXchg(cmp, swap, ord)` | Compare-and-swap | Produces (old, bool success) | capability demanded | {concurrency} |
| `Fence(ord)` | Memory fence | No value; consumes and produces Effect token | none | {concurrency} |

Provenance narrowing law (normative): for any pointer value p and offset chain, `alloc_id(PtrOffset(p,d)) = alloc_id(p)`; `caps(PtrOffset(p,d)) ⊆ caps(p)`; `caps(p) ⊆ {load, store, compute}`; copies preserve provenance exactly; a dereference node must have `load` (Load, VolatileLoad, AtomicLoad, RawLoad, MemCopy src) or `store` (Store, VolatileStore, AtomicStore, RawStore, CmpXchg, AtomicRmw, MemCopy dst, MemClear) in the address capability set or be guarded by a `ProvenanceCheck` node demanding that capability. A pointer produced by `Int2Ptr` has alloc id `ForeignAlloc` and caps {compute}; it must be guarded before any dereference.

## 13.6 Memory SSA (§20.6)

**Memory state.** A `MemState` is an SSA value of class Memory describing the contents of one memory partition at a program point. The distinguished state `Unknown` represents memory whose contents the compiler cannot track (§13.6 below). **Memory definitions:** the producers of MemState values are exactly: Start (one state per incoming partition), Store, RawStore, Init, MemCopy, MemMove, MemClear, Alloc (new partition), Dealloc, VolatileStore, AtomicStore, AtomicRmw, CmpXchg, every Call/Invoke with a summary declaring writes (§13.9), Drop (§13.10), and MemPhi. **Memory uses:** the consumers are exactly: Load, RawLoad, VolatileLoad, AtomicLoad, AtomicRmw, CmpXchg, MemCopy, MemMove, MemClear, Dealloc (must observe the partition alive), Drop, and Call (states for partitions the callee may touch). **Memory merge nodes:** `MemPhi` exists only at Region and Loop nodes; it has one MemState input per control predecessor (Region) or per entry/backedge path (Loop); V6 checks arity and predecessor correspondence. **Memory loop-carried states:** the MemPhi at a Loop header implements the loop-carried memory state; its input 0 is the state on loop entry, input 1 is the backedge state; the backedge input may be the MemPhi itself. **Memory partitions:** a partition is an equivalence class of memory locations the compiler tracks together; `MAX_MEM_PARTITIONS = 65536` per function, partition id is `u16`. Partition assignment algorithm (exact): (1) each `Alloc` node starts a fresh partition; (2) each distinct static item and each `Param` of pointer type with a handoff no-alias fact starts a fresh partition; (3) `AddrOf`/`PtrOffset`/`ProvenanceCheck` preserve the partition of their base; (4) all pointer-typed Params without a separating handoff fact, all Call results of pointer type with Unknown summary, all RawLoad results, and all `Int2Ptr` results join the single partition `P_UNKNOWN`; (5) union-find merges partitions when an alias fact proves sharing. **Alias classes:** an alias class is a partition plus a kind tag {typed, raw}; `alias(x, y) = (partition(x) = partition(y)) ∧ (kind(x) = kind(y) ∨ kind(x) = raw ∨ kind(y) = raw)`. **Read dependencies:** for a Load of partition P consuming MemState s, its read dependency set is the set of memory definitions d on the def-chain from s backward (excluding Phi merges) with `d` writing P or writing P_UNKNOWN. **Write dependencies:** symmetric: for a Store to P, the set of reads and writes of P or P_UNKNOWN that its new MemState chain covers. **Unknown memory state:** `P_UNKNOWN`'s initial state at Start is `Unknown`; any definition whose summary is Unknown produces `Unknown` for every partition it may touch. **Memory state splitting:** at an If/Switch, each arm continues from the incoming state; the states are split structurally (no node); rejoining uses MemPhi. **Memory state merging:** MemPhi as above; `join` of two states where at least one is `Unknown` is `Unknown`. **Memory dependency pruning:** the optimization that narrows a Load's MemState input from a coarser state to the nearest definition that writes its partition (pass M-13.12-mem-5, legality: the pruned-to definition must dominate the load and every intervening definition on the removed chain must not write the load's partition nor P_UNKNOWN). **Memory state verification:** the verifier checks: each MemState value has exactly one definition per partition per program point (SSA), all MemPhi inputs correspond to Region predecessors, no use of a MemState from a Dealloc'd partition except through that Dealloc's chain, and the state graph (excluding Loop backedge inputs) is acyclic (V7).

## 13.7 Effect nodes (§20.7)

**Effect definitions:** a node defines (produces) an effect token iff its schema has an Effect output; the producers are exactly: EffectChain (start), Store, RawStore, MemCopy, MemMove, MemClear, Alloc, Dealloc, VolatileLoad, VolatileStore, AtomicLoad, AtomicStore, AtomicRmw, CmpXchg, Fence, Drop, Call/Invoke with non-empty row, Suspend, Panic, and Trap. **Effect uses:** a node uses (consumes) a token iff its schema has an Effect input; the consumers are exactly the producers other than EffectChain, plus every Call/Invoke regardless of row (calls are ordered by the chain even when pure, to preserve ABI observability of side-effect order). **Effect merges:** `EffectPhi` exists only at Region and Loop nodes with one token input per predecessor; identical discipline to MemPhi (V6, V8). **Effect partitions:** an effect partition is the classification of each chain segment by its effect row, computed as an analysis over the chain (a segment is maximal between two multi-input merges); partitions license reordering only as stated in §13.12 pass rows; they never split the chain: IR 1 maintains exactly one chain. **Effect dependencies:** the token edges; two effectful nodes are ordered iff one is on the chain path to the other's input; reordering beyond chain adjacency is banned except by named passes with explicit legality (§13.12). **IO:** effect atom `io` (SPEC-INDEX §5.3); produced by VolatileLoad/VolatileStore, intrinsic calls marked io, and Foreign calls marked io. **Allocation:** atom `alloc`; produced by Alloc, MemCopy/MemMove/MemClear with dynamic length, Dealloc, and calls whose summary includes alloc. **Deallocation:** Dealloc; its obligation must be preserved: a Drop obligation for an owned allocation may be removed only by the pass that removes the allocation itself. **Synchronization:** atom `concurrency`; produced by atomic nodes and Fence. **Atomic operations:** as §13.5; they consume the token so that they cannot be reordered relative to each other or to I/O. **Panic:** atom `panic`; Panic nodes and unwinding Calls carry it; when the build is `panic = abort`, lowering replaces Panic by Trap (recorded as a build-mode transformation, not an IR 1 pass). **Suspension:** atom `async`; produced by Suspend; the Suspend node must be positioned exactly at the frontend's await points (Volume 08); it must not be deleted or created. **Foreign calls:** atom `ffi`; Foreign calls also carry every atom their ABI declares observable. **Unknown effects:** a call with Unknown summary (§13.9) carries the full atom set `{io, alloc, panic, unsafe, async, concurrency, ffi, nondet}` and writes P_UNKNOWN. **Token propagation, combination, removal (exact):** propagation: each producer consumes exactly one token input and produces exactly one token output; the chain must form a path from EffectChain through every effectful node to the terminal node's effect input, with EffectPhi at every multi-predecessor Region/Loop on that path. Combination: EffectPhi merges tokens from predecessors; the merged token is the join point. Removal: when a pass deletes a node with an Effect input and output, it must rewire the token producer directly to the deleted node's token consumer; when it deletes an EffectPhi (region gone), it must rewire each predecessor's token to the surviving successor path. The chain graph with Loop backedge inputs removed must be acyclic (V8); it must be connected: every effectful node lies on exactly one chain path (V8).

## 13.8 Type representation (§20.8)

Every value-producing node carries the following metadata, all checked or propagated by the verifier and preserved by passes unless a pass row says otherwise:

| Bullet | Definition |
| --- | --- |
| Semantic type | A `TypeId` interned per Volume 12 §19.1; every Value output slot and every Value input slot carries one; V2/V3 enforce the opcode schema. |
| Generic substitution | IR 1 graphs are built per monomorphized instance (SPEC-INDEX §3); all TypeIds are post-substitution concrete types; the instance's substitution map is graph metadata used for debug info and is not consulted by passes. |
| Known value constraints | `ValueConstraint ∈ {Exact(Const v), NotIn(set), Bool(true|false), NonNull}`; attached to value nodes; produced by constant folding and branch facts; consumed by VCCP (§13.12) and folding. Constraint sets combine by intersection; `Exact` conflicts with `NotIn` containing it iff they make the type empty, which is a verifier error only if the node is reachable on a path where the branch fact was derived (check is local: a node may carry conflicting constraints only if unreachable, which V14 already rejects). |
| Integer range information | `Range(lo: i128, hi: i128)` with lo ≤ hi, clamped to the value's integer type bounds; attached where available; must never claim a range wider than the type's value space; folding must respect overflow semantics per §13.4. |
| Nullability information | `Nullable ∈ {AlwaysNull, NeverNull, MaybeNull}` for reference and pointer types; NeverNull derives from Volume 04 non-nullable references and NonZero types; used to fold null checks (banned on raw pointers from unsafe sources, which stay MaybeNull). |
| Enum variant information | `VariantInfo ∈ {AllVariants, OneVariant(id), VariantSet(set)}`; produced by Discriminant comparisons on branch facts; OneVariant licenses AggGet on the variant payload directly (frontend already checked, fact preserved). |
| Ownership category | `Own | Copy | MoveOnly | BorrowShared | BorrowMut | Raw`; from Volume 06; drives Drop obligations (§13.10) and pass legality (ownership preservation, §13.13). |
| Relevant lifetime metadata | `LifetimeRegionId` references to the frontend's lifetime regions (Volume 06/11), attached to borrow-typed values; passes must not extend the region in which a borrow is used (V12). |
| Representation-independent metadata | IR 1 must not contain field offsets, sizes, alignments, register assignments, or any layout fact; those begin in IR 2 (Volume 14 §21.3). The verifier checks their absence (V13). |

## 13.9 Call nodes (§20.9)

`Call` is one opcode with a `CallKind` discriminator; all bullets are properties of this node.

| Bullet | Definition |
| --- | --- |
| Direct calls | `CallKind::Direct(SymId)`; callee resolved; subject to inlining (§13.12). |
| Indirect calls | `CallKind::Indirect`; callee operand is a fn-pointer value with provenance-checked ABI (UB-4 judgment deferred to runtime; the graph must keep the callee operand exactly). |
| Generic calls | `CallKind::Generic` exists only transiently during construction; monomorphization (SPEC-INDEX §3) must resolve every generic call to Direct before verification; V10 rejects any remaining Generic. |
| Intrinsics | `CallKind::Intrinsic(IntrinsicsId)`; schema, preconditions (UB-6), and effect row from the Volume 10 intrinsic table. |
| Foreign calls | `CallKind::Foreign(abi)`; abi is the Volume 10 convention; carries atom `ffi` plus the ABI's declared observable atoms; must consume the Effect token. |
| Trait calls | `CallKind::Trait(vtable_slot)` or `Direct` after devirtualization; the vtable operand is the trait object's fat pointer second half; IR 1 keeps the dynamic form (devirtualization is a §13.12 pass operating on derived facts). |
| Closure calls | `CallKind::Closure`; operands are the ClosureValue and call arguments; lowering of env access is by AggGet on the env. |
| Recursive calls | `CallKind::Direct(s)` with `s` = this function's own symbol; flagged `is_recursive`; inlining must respect the recursion guard (§13.12). |
| Calls with panic edges | If the callee summary includes `panic` and the build is not `panic = abort`: a Call gets one Ctrl output routed to the function's PanicExit; an Invoke (§13.3) is required instead exactly when a catch boundary is reachable in-function; with `panic = abort`, PanicExit is replaced by Trap and no unwind edge exists. |
| Calls with allocation effects | Summary includes `alloc`: the call consumes the token and produces MemState updates for P_UNKNOWN and for any partition the summary declares returned-pointer-into. |
| Calls with unknown effect summaries | `CallSummary::Unknown`: row = full atom set, writes P_UNKNOWN with `Unknown` state, reads all partitions. `CallSummary::Known { row, reads: set<Partition>, writes: set<Partition>, returns_ptr_to: Option<Partition>, may_unwind: bool }` is used when the callee's summary is available from the interprocedural summary table (§13.12 function passes). |

Call signature rule (V9): argument count, argument types, and return types must equal the callee signature's post-substitution values; the demanded row must be a subset of the caller's available row per SPEC-INDEX §5.3 (`r1 ≤ r2 ⟺ r1 ⊆ r2` at requirement positions).

## 13.10 Ownership information (§20.10)

IR 1 retains the frontend's semantic guarantees as metadata; the following are the complete metadata kinds, attached to nodes and checked by V12:

| Bullet | Definition |
| --- | --- |
| Ownership transitions | `OwnershipTransition { place, from, to }` metadata on Store/Init/Move (MemMove)/Drop nodes; `from`/`to` are place states of SPEC-INDEX §5.4; every transition recorded by the borrow checker must appear exactly once. |
| Move origins | `MoveOrigin { source_span }` metadata on MemMove nodes; preserved through passes for diagnostics (G22). |
| Borrow origins | `BorrowOrigin { span, kind: Shared|Mut }` metadata on AddrOf/AggGet-of-borrow nodes; the borrow's end is marked by an `EndBorrow` metadata record on the node where the loan dies. |
| Lifetime regions | `LifetimeRegionId` on borrow-typed values (§13.8); the set of regions forms the frontend's region forest; passes must keep borrows within their regions. |
| Initialization states | Per-partition init facts carried on the MemState chain: `InitFacts(place) ∈ {Uninit, Init, MaybeUninit}`; Init/Store make places Init; V11 checks that safe-code loads read only Init facts. |
| Destruction obligations | A `Drop` node: Inputs: Ctrl, Effect, MemState, the value; Outputs: new Effect and MemState; Semantics: runs the drop glue of the value's type at this point; every `Own` value with drop glue must have exactly one Drop on every path where it is live (V12); Drop obligations may move only to strictly later program points on the same path (dropping earlier is banned unless the pass proves the drop is side-effect-free). |
| Pinning constraints | `Pinned` metadata: the place's address must not change and the object must not move; passes must not relocate, copy-eliminate, or virtualize pinned objects (Volume 14 §21.6 honors this via `Pin-aware optimization`). |
| Address observability facts | `AddrObserved` metadata on places whose address is taken and observable (stored, compared, passed to unknown code); set by AddrOf whose result reaches an unknown use; blocks elimination of the place's storage (§13.12 memory passes). |
| Object identity facts | `IdentityObserved` metadata on allocations whose identity is observable (PtrCmp with another pointer to the same partition, address stored); blocks allocation elimination and identity-breaking rewrites (Volume 14 §21.6 Identity preservation). |

## 13.11 IR 1 verification (§20.11)

**Invariants I1–I12.** I1: exactly one Start and exactly one terminal node per graph, the terminal being Return, Trap, or PanicExit; PanicExit exists iff any Panic or unwinding Call exists. I2: every edge points producer→consumer; Value+Effect+Memory edges, excluding Phi backedge inputs, form a DAG. I3: exactly one EffectChain; the effect subgraph is connected and acyclic after removing Loop backedge inputs; EffectPhi appears at every multi-predecessor Region/Loop on the chain. I4: for every Value/Memory/Effect edge def→use, `anchor(def)` dominates `anchor(use)` in the dominator tree of the control subgraph, where `anchor(n)` is n's control anchor (Region containing it, assigned by the scheduler; Start-anchored nodes dominate all). I5: Phi, EffectPhi, MemPhi exist only at Region and Loop nodes, with input count equal to predecessor count and input i corresponding to predecessor i. I6: MemState single-assignment per partition per program point; MemPhi discipline of I5; state graph acyclic after backedge removal. I7: every pointer-typed value carries provenance metadata; every dereference obeys §13.5's provenance rule. I8: every node instance satisfies its opcode's type schema (V2/V3). I9: no dead and no unreachable nodes. I10: every Call satisfies V9 signatures and row demands. I11: every node opcode is in the closed inventory; no target-specific or layout-bearing metadata exists. I12: every Loop has exactly 2 control inputs; every reachable path from Start ends in a terminal node; every loop is exited on every path that reaches a terminal other than through diverging bodies (V15).

**Verifier checks.** The verifier for the §20.11 bullets is check set V1–V18: V1 graph structure (closed inventory I11, slot arity, I1, I2, node/edge limits); V2 node input types (I8); V3 node output types (I8); V4 control-flow validity (Region predecessor validity, Loop arity I12, Switch arm coverage, unwind edges target LandingPad or PanicExit, every path reaches a terminal); V5 dominance (I4); V6 merge validity (I5, phi/Region correspondence); V7 memory SSA consistency (I6); V8 effect SSA consistency (I3); V9 call signatures (I10); V10 return types (Return value types = signature return tuple; no Generic calls remain); V11 initialization constraints (loads in safe code read Init facts); V12 ownership constraints (transitions match borrow-checker records, Drop obligations on every live path, borrow regions respected); V13 safety metadata (atoms `unsafe`/`ffi` present where required; provenance rule I7; no layout metadata I11); V14 unreachable control (I9: no unreachable nodes; error `UnreachableNode`); V15 graph termination rules (I1, I12 path completeness; `LoopMalformed` otherwise); V16 node count and edge count within limits; V17 effect row demands satisfiable at every call (row subset rule); V18 metadata consistency (constraints not contradictory on reachable nodes; ownership categories consistent with types).

**Algorithm.** Input: graph G with n nodes, m edges. Step 1: build CSR arrays for the data subgraph (Value+Effect+Memory edges, excluding Phi backedge inputs) and the control subgraph. Step 2: run three-color iterative cycle detection on the control subgraph and on the data subgraph; a cycle in the data subgraph, or in the effect/memory subgraphs after backedge removal, is error `EffectCycle`/`MemCycle`/`DataCycle` with the node list reported (G17). Step 3: compute the topological order of the data subgraph by Kahn's algorithm with ascending-ID tie-break; failure is an error (never a partial order). Step 4: compute reverse postorder of the control subgraph from Start and run iterative dominators (≤ n passes, early exit) to obtain the dominator tree. Step 5: compute anchors by scheduling floating nodes into regions (each floating node is placed in the Region of its highest-anchored use; ties broken by ascending ID). Step 6: traverse nodes in topological order; per node execute the schema check (V2/V3, O(deg)), the dominance check (V5, O(1) per edge via the dominator tree), and the node-family checks V4, V6–V13, V17, V18 (each O(deg) with per-partition state maps). Step 7: reachability sweep from Start and from live roots computes V14 and dead-node detection; the sweep is a bounded DFS. Output: `Ok(())` or the first error with its check ID and node list. Termination: every step is a bounded traversal; the only loop with a data-dependent bound is the dominator fixpoint, bounded by n passes with early exit (pinned §5.7). Complexity: O(n·p + m) with p ≤ n the dominator pass count (measured p ≤ 4 on the compiler's own bootstrap corpus; the bound is the obligation, not the measurement). Errors: the complete enum `Ir1VerifyError` has exactly the variants named in CEP:FAILURE below.

## 13.12 IR 1 transformations (§20.12)

Every pass below is specified by the 10-row template of design.md §40; the columns are: Pre (exact preconditions), Analyses (required analyses), Algorithm, Preserves (what must not change: semantics S, effects E, memory M, ownership O, panics P, debug D), Invalid-if (conditions making the transformation invalid), Complexity, Profit (profitability model), Verifier (beyond V1–V18 on output), Tests (regression class), Proof (correctness strategy per G4/G5). All passes run on graphs satisfying I1–I12 and must emit graphs satisfying I1–I12 (G5: the verifier runs after every pass; a failing pass is a hard error with counterexample). Node costs for profit models: const 0, arith/bit/conv 1, cmp 1, load 3, store 3, call 10, branch 2, phi 1.

**Simplification.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Constant folding | I1–I12 | constraint/range metadata (§13.8) | For each pure arith/bit/cmp/conv node with all inputs `Exact`, compute per Volume 05 semantics incl. overflow and traps; replace by Const or Trap/panic path; repeat to fixpoint ≤ 3 sweeps | S,E,M,O,P,D | Node effectful; overflow semantics would differ from Vol 05; float folding only for non-NaN inputs | O(n) per sweep | removes 1 node per fold | folded Const value in type range | folding-of-overflow corpus | Z3 over bounded integer domains + EXH boundary cases (OB-13-13) |
| Constant propagation | I1–I12 | def-use; constraints | Replace uses of values with `Exact` constraints by the Const; then rerun folding | S,E,P,D | Use is in a Phi at a Region not dominated by the fact's branch (V5 would fail) | O(n·d) | −1 use per propagation | — | SCCP corpus | REVIEW + Z3 small cases |
| Identity elimination | I1–I12 | none | Replace `Add x 0`, `Sub x 0`, `Mul x 1`, `And x −1`, `Or x 0`, `Xor x 0`, `Shl x 0`, double negation, `zext` of bool, identity phis, by the surviving operand | S,E,M,O,P,D | Width-0 edge cases; signedness changes the identity's validity | O(n) | −1 node per match | — | algebraic corpus | REVIEW; identities table checked EXH |
| Algebraic simplification | I1–I12 | none | Apply rule table R-ALG, a closed table of exactly 47 entries (the table is normative data; OB-13-13's EXH artifact enumerates and checks all 47); representative members: `x·0→0 (non-float)`, `x·2^k→shl`, `x−x→0`, `¬¬x→x`, `(x^a)^b→x^(a^b)`, `a+(b+c)→(a+b)+c by ID order` | S,E,P,D | Rule application would introduce a trap the input did not contain: `0/x→0` is banned (division by zero is a defined trap); `x−x→0` is allowed | O(n·r), r = rules per opcode ≤ 6 | −1 node per match | — | algebraic corpus | EXH over rule table on 16-bit bounded domain |
| Canonicalization | I1–I12 | none | Normalize operand order by ascending NodeId for commutative ops; normalize comparisons to Lt/Le/Gt/Ge form; hoist constants to RHS | S,E,P,D | none (purely syntactic, both forms equivalent) | O(n) | enables CSE matches | — | canon corpus | REVIEW |
| Redundant conversion elimination | I1–I12 | type lattice | Collapse conv chains zext(zext)→zext, sext(sext)→sext, trunc(trunc)→trunc, ext-then-trunc to same width→identity, when the outer conv is value-preserving on the inner's range (checked via range metadata) | S,E,P,D | Range metadata absent and outer conv not total | O(n) | −1 node per match | — | conv corpus | Z3 on width lattice (widths ≤ 128) |

**Value optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Common subexpression elimination | I1–I12 | GVN table (below) | Hash-cons pure value nodes by (opcode, normalized inputs, type); replace later duplicates | S,E,M,O,P,D | Node effectful or memory-reading (loads excluded here) | O(n) expected, O(n·k) worst k = chain collisions | −1 node per dup | — | CSE corpus | REVIEW + GVN equivalence |
| Global value numbering | I1–I12 | dominators; the same table | Partition pure nodes into congruence classes over the dominator tree; iterate class merge to fixpoint ≤ n rounds | S,E,M,O,P,D | same | O(n·rounds), rounds ≤ n | input to CSE | — | GVN corpus | REVIEW; soundness of congruence argued per G4 |
| Conditional constant propagation | I1–I12 | branch facts; constraints | Propagate `Exact` facts along control paths guarded by branch conditions; re-derive dead arms | S,E,M,O,P,D | fact uses not dominated by branch (V5) | O(n) | enables folding | — | SCCP corpus | Z3 on small CFG lattice |
| Redundant computation elimination | I1–I12 | GVN + schedule | After GVN, delete nodes whose class leader dominates all their uses and which are pure | S,E,M,O,P,D | leader doesn't dominate a use (V5) | O(n) | −1 node per deletion | — | shared with GVN | as GVN |
| Value numbering through control flow | I1–I12 | dominators; Region phi structure | Extend GVN across Regions: value v on branch i is congruent iff all arm values are congruent; then insert/normalize Phi of leaders | S,E,M,O,P,D | arm counts differ from Region preds | O(n·arms) | merges arm computations | phi arity (V6) | CFN corpus | REVIEW |

**Control optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Branch simplification | I1–I12 | constraints | If cond has `Exact(true/false)`, rewire the taken projection's users to the surviving arm; delete If and dead arm | S,E,M,O,P,D | cond not exact | O(n) | −1 If per fold | V4 | branch corpus | Z3 |
| Branch elimination | I1–I12 | constraints; dominance | Both projections feeding one Region with identical non-Phi content: replace Region by the common predecessor and delete If | S,E,M,O,P,D | Region carries Phis with differing inputs | O(n) | −1 If, −1 Region | V4/V6 | branch corpus | REVIEW |
| Unreachable block elimination | I1–I12 | reachability | Delete control-unreachable subgraphs and their nodes (dead nodes too, §13.2); rewire phis losing predecessors | S,E,M,O,P,D | would leave a Phi with fewer inputs than Region preds (fix by rewiring in same pass) | O(n+m) | −dead nodes | V4/V6/V14 | unreach corpus | REVIEW |
| Merge simplification | I1–I12 | none | Single-pred Region: rewire users of phis to the pred values, users of the Region to the pred; delete Region | S,E,M,O,P,D | none | O(n) | −1 Region | V6 | merge corpus | REVIEW |
| Loop simplification | I1–I12 | dominators; loop regions | Rotate loops to canonical do-while form: guard becomes exit test at bottom; header keeps entry+backedge arity | S,E,M,O,P,D | loop contains Suspend or unwinding Call in preheader-guarded position | O(loop size) | enables unrolling | V4/V6 | loop corpus | REVIEW per G4 |
| Control dependency reduction | I1–I12 | dominators (post-dominance on control) | Pure node anchored under a control region where all arms compute the same pure value: hoist to the join and delete arm copies ( If conversion support) | S,E,M,O,P,D | hoist crosses an effectful op (E) | O(n) | −dup nodes | V5 | cdep corpus | REVIEW |
| If conversion, when legal | I1–I12 | post-dominance; effect chain | Both arms of an If are straight-line, side-effect-free except one Store each to the same partition: replace branch by `Select(c, a_val, b_val)` + single Store | S,E,M,O,P,D | arms contain calls, atomics, volatile, Suspend, or writes to different partitions | O(arm size) | −2 blocks worth of control | V4 | ifconv corpus | REVIEW + differential tests |

**Memory optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Load elimination | I1–I12 | partitions (§13.6); def chains | Load from partition P reading state s: walk the def chain; if the nearest dominating def is a Store to P with the same address and no intervening unknown write, replace Load by the stored value | S,E,M,O,P,D | intervening def writes P or P_UNKNOWN; address mismatch; volatile/atomic load | O(chain) per load | load(3)→0 | V7 | loadopt corpus | REVIEW + differential |
| Store elimination | I1–I12 | partitions; liveness of MemStates | Store to P whose new MemState has no uses other than overwritten states and whose value is not observable (no later read of P before another def) is deleted | S,E,M,O,P,D | state value escapes into a Phi consumed by a read; volatile/atomic store; store to IdentityObserved place | O(uses) | −store(3) | V7 | deadstore corpus | REVIEW |
| Store-to-load forwarding | I1–I12 | partitions; address equivalence | Same-address Store dominates Load with no intervening def of P or P_UNKNOWN: forward value | S,E,M,O,P,D | intervening unknown; address not provably equal; volatile/atomic | O(chain) | load(3)→0 | V7 | forwarding corpus | REVIEW + Z3 on partition model |
| Dead store elimination | I1–I12 | liveness | Stores killed by a later Store to the same address before any read: delete earlier store | S,E,M,O,P,D | volatile/atomic; IdentityObserved; Pinned place | O(n·d) | −store | V7 | dse corpus | REVIEW |
| Memory dependence refinement | I1–I12 | partitions | Narrow each memory node's MemState input to the nearest def of its partition (pruning rule §13.6) | S,E,M,O,P,D | nearest def does not dominate the use (V5) | O(n·p̄) p̄ = mean chain length | enables forwarding/DCE | V5/V7 | refine corpus | REVIEW |
| Alias analysis | (analysis, not a transform) | — | Computes/refreshes partitions and alias classes per §13.6; required by the passes above | — | — | O(n·α) α = union-find | — | partition axioms vs handoff facts | alias corpus | EXH on generated access sets ≤ 8 sites |
| Memory state simplification | I1–I12 | partitions | Identity MemPhi (all inputs same state) → replace by that state; unused states → drop definitions | S,E,M,O,P,D | none | O(n) | −phi nodes | V6/V7 | memssa corpus | REVIEW |
| Redundant initialization elimination | I1–I12 | InitFacts | Init of a place immediately followed by a full Store to the same place with no intervening read: delete the Init | S,E,M,O,P,D | intervening read or partial write; place Pinned | O(n) | −init store | V7/V11 | init corpus | REVIEW |

**Effect optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Effect dependency simplification | I1–I12 | chain analysis | Merge adjacent token pass-throughs; remove dead branches' EffectPhi inputs with pred rewiring | S,E,M,O,P,D | would disconnect a live effectful node from the chain | O(n) | −phis | V8 | eff corpus | REVIEW |
| Dead effect elimination | I1–I12 | chain; use lists | Delete effectful nodes with no observable behavior (no memory write, no io/panic/async/concurrency atom) by chain rewiring (§13.7 removal rule) | S,E,M,O,P,D | node writes memory, has io/panic/ffi/async atom, or is MustKeep | O(n) | −nodes | V8 | dead eff corpus | REVIEW |
| Effect-aware code motion | I1–I12 | partitions; chain; dominance | Move a pure node's anchor from a cold arm to the join if it is computed on all arms; never move across a token edge | S,E,M,O,P,D | motion crosses Effect/Store/Fence | O(n) | −dup computation | V5 | motion corpus | REVIEW |
| Call effect propagation | I1–I12 | interprocedural summary table | Replace a Call's `Unknown` summary by the callee's `Known` summary when available (callee verified) | S,E,M,O,P,D | callee's summary unverified or recursive cycle unresolved | O(calls) | enables memory passes | V17 | summary corpus | REVIEW; summaries proven at callee |
| Effect summary refinement | I1–I12 | summary table | Narrow a Known summary's `reads/writes` sets by the actual partition sets at this call site (intersection with reachable partitions) | S,E,M,O,P,D | narrowing drops a partition the callee can reach via P_UNKNOWN | O(calls·parts) | enables pruning | V17 | summary corpus | REVIEW |

**Loop optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Loop invariant code motion | I1–I12 | loop regions; dominance; effects | Node in loop body is hoistable iff pure, no effectful operand defined in loop, and its block dominates all loop exits; hoist to preheader | S,E,M,O,P,D | invariant only conditionally executed (would add a computation on non-taken paths: allowed only if pure and trap-free) | O(n·loops) | −body node per iteration | V5 | licm corpus | REVIEW + differential |
| Induction-variable analysis | (analysis) | — | Classify IVs: base + stride per iteration; closed-form detection for affine IVs | — | — | O(n) | feeds passes | — | iv corpus | Z3 on affine systems |
| Strength reduction | I1–I12 | IV analysis | Replace IV multiplications by Adds of accumulated stride (mul x·c → add chain) when c constant and target-independent win (cost mul 3 vs add 1) | S,E,M,O,P,D | non-affine; overflow semantics differ (must use wrapping form) | O(n) | −2 cost per repl | — | strred corpus | Z3 on modular arithmetic, width ≤ 64 |
| Loop peeling | I1–I12 | regions | Peel first iteration k=1: copy body, guard entry; enables constant propagation of boundary conditions | S,E,M,O,P,D | body contains Suspend | O(body) copy | +code for −branch/iteration | V4/V6 | peel corpus | REVIEW per G4 loop table |
| Loop unrolling | I1–I12 | regions; trip analysis | Unroll by factor u: u = 4 default, 8 if body cost ≤ 12 and trip count constant multiple; full unroll only if trip count is a compile-time constant ≤ 16 | S,E,M,O,P,D | body contains Call with `panic` unwind edge into the loop's PanicExit (must keep unwind structure); body size·u > MAX_UNROLL_BYTES = 4096 cost units | O(body·u) | trade code for branches | V4/V6 | unroll corpus | REVIEW per G4 |
| Loop unswitching | I1–I12 | regions; constraints | Loop-invariant If inside body: clone loop per arm, hoist the If | S,E,M,O,P,D | clone count would exceed MAX_LOOP_CLONES = 4 per loop | O(body·2) | removes per-iteration branch | V4/V6 | unswitch corpus | REVIEW |
| Loop fusion, if legal | I1–I12 | regions; partitions; trip equality | Merge adjacent loops with equal trip counts, no loop-carried conflict between first loop's writes and second's reads (checked on partitions), no effectful op between them | S,E,M,O,P,D | trip counts unequal; conflicting partitions; atomics/volatile in body | O(bodies) | −loop overhead | V4/V6/V7 | fusion corpus | REVIEW + differential |
| Loop vectorization preparation | I1–IR2 handoff | IV; aliasing; alignment facts | Annotate loops satisfying: affine IV, unit or constant stride access, non-conflicting partitions, body in the vector-legal opcode set {arith, bit, cmp, load, store, select}; emit `Vectorizable` fact consumed by Volume 14 §21.13 | S,E,M,O,P,D | fact wrong (IR 2 must re-verify) | O(n) | fact only | — | vecprep corpus | REVIEW |

**Function optimization.**

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier+ | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Inlining | I1–I12; callee graph verified; non-recursive callee or recursion depth ≤ MAX_INLINE_DEPTH = 5 | cost model (below); summaries | Splice callee graph at call site: rename NodeIds, merge Effect chains, MemStates, Region joins for both projections; delete Call | S,E,M,O,P,D | callee contains Suspend, inline asm equivalent, or `ffi` atom and caller is in a transactional effect context (Volume 07); splicing breaks phi correspondence | O(callee size) | per §13.13 cost model | V4–V13 | inline corpus | REVIEW per G4 inline row: call semantics, stack behavior, exceptions, effects preserved |
| Inlining cost models | — | — | The cost function `size(G) = Σ node costs`; `inline_profit = CALL_COST(10) + est(callee dynamic savings) − size(callee)·growth_weight`; growth_weight = 1 cold, 0.25 hot (hot = site dominated by a Loop or profile frequency ≥ 0.9); inline iff inline_profit > 0 and caller growth ≤ INLINE_BUDGET = 256 cost units per function | — | — | O(callee) | the model itself | — | cost corpus | REVIEW; thresholds pinned |
| Devirtualization | I1–I12; class analysis | receiver class set | CallKind::Trait with singleton receiver set (from handoff facts, guard tests, or allocation-site analysis): rewrite to Direct | S,E,M,O,P,D | receiver set has > 1 element or is unknown | O(n) | indirect(10)→direct | V9 | devirt corpus | REVIEW |
| Tail-call recognition | I1–I12 | call positions | Mark Direct calls whose result is returned unchanged on all paths from the call as `TailCall`; IR 2/15 lower per ABI | S,E,M,O,P,D | arguments used after call; call in loop needing frame | O(n) | fact only | — | tailcall corpus | REVIEW |
| Call graph optimization | crate level | call graph (SCC via §5.7 math) | Compute SCCs; order bottom-up for summaries; prune unreachable functions; clone functions by context when a summary makes a parameter constant (clone ≤ MAX_FN_CLONES = 8) | S,E,M,O,P,D | clone would break exported ABI symbol | O(cg) | enables specialization | V9 per clone | cg corpus | REVIEW |
| Interprocedural propagation | crate level | summaries (§13.9) | Compute fixed-point of Known summaries over SCC reverse topological order; terminate when no summary changes or after MAX_SUMMARY_ROUNDS = n_scc + 2 rounds | S,E,M,O,P,D | summary over-approximates unsafely (must remain an over-approximation: sound direction is widen-only) | O(scc·n) | enables passes above | V17 | ipa corpus | REVIEW; monotonicity argued |

## 13.13 IR 1 optimization legality (§20.13)

Every pass of §13.12 must satisfy all twelve of the following; the pass table columns cite them. This subsection is the legality framework; §13.12 rows are the per-pass instantiations.

| Bullet | Normative rule |
| --- | --- |
| Preconditions | A pass runs only on a graph satisfying I1–I12 and its row's Pre column; running on an unverified graph is banned. |
| Required analyses | Exactly the Analyses column; an analysis must be freshly computed or proven valid for the current graph (invalidation per design.md §19.6); stale analysis use is banned. |
| Semantic preservation conditions | For all x in the pass domain, ⟦T(x)⟧ = ⟦x⟧ per G4; the domain is the row's Pre; preservation is checked by the verifier plus the pass's Proof column (G5: every optimization has formal pre/postconditions, a preservation strategy, explicit exceptional cases, and no unverified pass in the pipeline). |
| Effect preservation | The effect row of every executed path must be preserved; effectful reordering is banned except where a row's Algorithm licenses it; EffectPhi/chain discipline (§13.7) must hold on output. |
| Memory preservation | Partition semantics must be preserved: a load's value must equal the most recent write in the input semantics; P_UNKNOWN conservatively blocks all reuse. |
| Ownership preservation | Every Drop obligation must survive; borrow regions must not extend; ownership transitions must keep the SPEC-INDEX §5.4 state machine consistency. |
| Panic preservation | Panic and trap events and their payloads must be preserved on every path where the input's semantics produces them; `panic = abort` builds may replace Panic by Trap build-wide, not per-pass. |
| Termination implications | A pass must not turn a terminating program into a non-terminating one or vice versa except by removing provably dead paths; loop transforms must preserve trip counts (peeling/unrolling included) or prove the change is equivalence-preserving on the iteration domain. |
| Debug information preservation | Every node must keep its source-span metadata; deleted nodes transfer spans to surviving replacements; span loss is a verifier warning `DebugGap` in debug builds (G22). |
| Expected benefit | The Profit column states the metric; a pass must be skipped when its model shows no benefit; passes must not fire "because they can" (design.md §21.16's principle, binding here). |
| Cost model | Node cost table of §13.12; compile-time cost is the Complexity column; both are pinned; changing them is a spec change. |
| Verification strategy | The Proof column names the strategy: Z3 script in `tools/fv/` (10 s timeout), EXH over a stated finite domain, or REVIEW naming the invariant; the verifier V1–V18 runs after every pass; pipeline composition is checked by running the full pass list with verification after each (G5 acceptance criteria). |

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-13-1 | Node inventory closure: for every constructed node, opcode ∈ the closed set of §13.3–§13.5, §13.7; slot arity and classes match the schema table | EXH over the full opcode × slot schema table | FV-0 | `tools/fv/ir1_schema.rs` test enumeration | G14, G15 |
| OB-13-2 | Data subgraph acyclicity: for every verified graph, Value+Effect+Memory edges minus Phi backedge inputs are acyclic; a cycle is reported with a node list | Z3 on generated graphs n ≤ 32 + EXH over generated small graphs (all DAG/edge subsets for n ≤ 6) | FV-0 | `tools/fv/ir1_acyclic.smt2`, `tools/fv/ir1_acyclic_exh.rs` | G11, G17 |
| OB-13-3 | Topological order correctness: Kahn output is a permutation of IDs, respects every data edge, and tie-breaks by ascending ID; cycle input yields error, never a partial order | EXH over all directed acyclic graphs on n ≤ 6 labeled nodes (generated) + Z3 for order axioms on n ≤ 64 | FV-0 | `math/crates/planck-graph-math` certificate + `tools/fv/graph_topo.smt2` | G11, G14 |
| OB-13-4 | Dominator correctness: iterative RPO dominators equal the unique dominator relation; entry dominates itself; result correct for reducible and irreducible graphs | EXH over all single-entry digraphs on n ≤ 5 nodes + Z3 dominator axioms on generated graphs n ≤ 48 | FV-0 | `planck-graph-math` certificate + `tools/fv/graph_dom.smt2` | G11, G14 |
| OB-13-5 | Dominator termination: fixpoint terminates within ≤ n passes with early exit | EXH over the same generated domain, recording pass counts | FV-0 | `planck-graph-math` certificate | G11 |
| OB-13-6 | Cycle detection: three-color iterative DFS reports a cycle iff one exists; reported node list is a cycle | EXH over all digraphs on n ≤ 6 nodes | FV-0 | `planck-graph-math` certificate | G11, G17 |
| OB-13-7 | Dominance invariant I4: every def→use edge has anchor(def) dom anchor(use) on generated graphs after scheduling | EXH on generated graphs n ≤ 8 with random anchors; PROP paired with OB-13-4 | FV-1 (gap: anchors on n > 8 untested exhaustively; bridge: verifier V5 runs on every real graph) | `tools/fv/ir1_dominance.rs` | G1 |
| OB-13-8 | Phi discipline I5: phi input count = Region/Loop predecessor count, correspondence exact | EXH over generated Region/phi structures | FV-0 | `tools/fv/ir1_phi.rs` | G1 |
| OB-13-9 | Effect chain I3: single chain, connected, acyclic after backedge removal, EffectPhi at every multi-pred region on the chain | EXH on generated chain structures ≤ 12 nodes | FV-0 | `tools/fv/ir1_effect.rs` | G1, G21 |
| OB-13-10 | Memory SSA I6: single assignment per partition per point; MemPhi correspondence; no use after Dealloc | Z3 on bounded partition/state models + EXH generated cases ≤ 4 partitions | FV-0 | `tools/fv/ir1_memssa.smt2` | G2 |
| OB-13-11 | Provenance law: alloc_id preserved and caps ⊆ caps under PtrOffset; narrow∘narrow = narrow; dereference demand satisfied or guarded | Z3 over the 8-element capability lattice × offset lattice (bounded) | FV-0 | `tools/fv/ir1_provenance.smt2` | G2 |
| OB-13-12 | Verifier termination and complexity: O(n·p + m), all loops bounded; every error variant of Ir1VerifyError reachable and complete | EXH over generated malformed graphs (each error variant triggered ≥ 1 case) + REVIEW of the termination argument | FV-1 (gap: complexity is measured, not proven, above n ≤ 64; bridge: limits table caps n) | `tools/fv/ir1_verifier_exh.rs` | G11, G15, G22 |
| OB-13-13 | Constant folding correctness: folded result equals Volume 05 semantics including overflow (panic) and trap cases, for all opcodes and widths | Z3 over bounded integer domains (8/16-bit full; 32/64-bit boundary classes) + EXH boundary sets | FV-0 | `tools/fv/ir1_fold.smt2` | G4, G5 |
| OB-13-14 | Pass pipeline equivalence: for every pass P and generated graph pair, verifier accepts output and differential execution matches on the test corpus; each pass has formal pre/postconditions | PROP (differential, deterministic PRNG) paired with Z3 fold/rewrite obligations | FV-1 (gap: generated-graph coverage bounded at n ≤ 24; bridge: G5 verifier after every pass in production) | `tools/fv/ir1_pass_diff.rs` | G4, G5 |
| OB-13-15 | Effect preservation: no pass reorders two effectful operations whose observable order differs in input semantics | Z3 on the chain-order model + REVIEW of each pass row's preservation column | FV-1 (gap: REVIEW component; bridge: chain discipline enforced by V8) | `tools/fv/ir1_efforder.smt2` | G1, G21 |
| OB-13-16 | Verifier completeness: each of V1–V18 implemented, exercised, and mapped to its §20.11 bullet | REVIEW with checklist against §13.11 | FV-2 | `spec/volume-13-ir1.md` §13.11 + verifier source review | G15 |
| OB-13-17 | Determinism: identical input graph + pass list produce byte-identical serialized output (G14) | PROP over corpus (byte-hash comparison) + REVIEW of ID assignment rules | FV-1 (gap: property sampling; bridge: deterministic construction order pinned in §13.2) | `tools/fv/ir1_determinism.rs` | G14 |
| OB-13-18 | Coverage: every §20.12 pass has the 10-row template filled and a regression class | REVIEW against §13.12 tables | FV-2 | coverage ledger below | G15 |

## Coverage ledger

| design.md bullet (line, abbreviated) | Defined in |
| --- | --- |
| l1857 Purpose | §13.1 |
| l1858 Semantic guarantees | §13.1 |
| l1859 Supported operations | §13.1 |
| l1860 Type preservation | §13.1 |
| l1861 Graph model | §13.1 |
| l1862 Control model | §13.1 |
| l1863 Memory model | §13.1 |
| l1864 Effect model | §13.1 |
| l1865 Ownership metadata | §13.1 |
| l1866 Transformation permissions | §13.1 |
| l1867 Transformation prohibitions | §13.1 |
| l1868 Target independence | §13.1 |
| l1872 Graph ownership | §13.2 |
| l1873 Node ownership | §13.2 |
| l1874 Node identity | §13.2 |
| l1875 Input edges | §13.2 |
| l1876 Output edges | §13.2 |
| l1877 Value edges | §13.2 |
| l1878 Control edges | §13.2 |
| l1879 Effect edges | §13.2 |
| l1880 Memory edges | §13.2 |
| l1881 Dependency ordering | §13.2 |
| l1882 Graph entry | §13.2 |
| l1883 Graph exit | §13.2 |
| l1884 Dead nodes | §13.2 |
| l1885 Unreachable nodes | §13.2 |
| l1886 Graph regions | §13.2 |
| l1892 Start | §13.3 |
| l1893 If | §13.3 |
| l1894 True projection | §13.3 |
| l1895 False projection | §13.3 |
| l1896 Switch | §13.3 |
| l1897 Merge | §13.3 |
| l1898 Region | §13.3 |
| l1899 Loop | §13.3 |
| l1900 Loop header | §13.3 |
| l1901 Loop backedge | §13.3 |
| l1902 Loop exit | §13.3 |
| l1903 Return | §13.3 |
| l1904 Unreachable | §13.3 |
| l1905 Panic | §13.3 |
| l1906 Exceptional exit | §13.3 |
| l1907 Invoke, if needed | §13.3 |
| l1911 Inputs (per node) | §13.3 table col. Inputs |
| l1912 Outputs (per node) | §13.3 table col. Outputs |
| l1913 Semantics (per node) | §13.3 table col. Semantics |
| l1914 Dominance behavior (per node) | §13.3 table col. Dominance |
| l1915 Control projection behavior (per node) | §13.3 table col. Projections |
| l1916 Legal graph transformations (per node) | §13.3 table col. Legal transformations |
| l1920 Constants | §13.4 |
| l1921 Parameters | §13.4 |
| l1922 Arithmetic | §13.4 |
| l1923 Comparisons | §13.4 |
| l1924 Bitwise operations | §13.4 |
| l1925 Conversions | §13.4 |
| l1926 Aggregate construction | §13.4 |
| l1927 Aggregate extraction | §13.4 |
| l1928 Aggregate insertion | §13.4 |
| l1929 Enum construction | §13.4 |
| l1930 Discriminant extraction | §13.4 |
| l1931 Tuple operations | §13.4 |
| l1932 Array operations | §13.4 |
| l1933 Range operations | §13.4 |
| l1934 Function values | §13.4 |
| l1935 Closure values | §13.4 |
| l1936 Function calls | §13.4, §13.9 |
| l1937 Intrinsics | §13.4, §13.9 |
| l1941 Load | §13.5 |
| l1942 Store | §13.5 |
| l1943 Raw load | §13.5 |
| l1944 Raw store | §13.5 |
| l1945 Initialization | §13.5 |
| l1946 Copy memory | §13.5 |
| l1947 Move memory | §13.5 |
| l1948 Memory clear | §13.5 |
| l1949 Memory allocation | §13.5 |
| l1950 Memory deallocation | §13.5 |
| l1951 Address formation | §13.5 |
| l1952 Pointer offset | §13.5 |
| l1953 Pointer comparison | §13.5 |
| l1954 Volatile load | §13.5 |
| l1955 Volatile store | §13.5 |
| l1956 Atomic load | §13.5 |
| l1957 Atomic store | §13.5 |
| l1958 Atomic read-modify-write | §13.5 |
| l1959 Compare-and-swap | §13.5 |
| l1960 Memory fence | §13.5 |
| l1968 Memory state | §13.6 |
| l1969 Memory definitions | §13.6 |
| l1970 Memory uses | §13.6 |
| l1971 Memory merge nodes | §13.6 |
| l1972 Memory loop-carried states | §13.6 |
| l1973 Memory partitions | §13.6 |
| l1974 Alias classes | §13.6 |
| l1975 Read dependencies | §13.6 |
| l1976 Write dependencies | §13.6 |
| l1977 Unknown memory state | §13.6 |
| l1978 Memory state splitting | §13.6 |
| l1979 Memory state merging | §13.6 |
| l1980 Memory dependency pruning | §13.6 |
| l1981 Memory state verification | §13.6 |
| l1985 Effect definitions | §13.7 |
| l1986 Effect uses | §13.7 |
| l1987 Effect merges | §13.7 |
| l1988 Effect partitions | §13.7 |
| l1989 Effect dependencies | §13.7 |
| l1990 IO | §13.7 |
| l1991 Allocation | §13.7 |
| l1992 Deallocation | §13.7 |
| l1993 Synchronization | §13.7 |
| l1994 Atomic operations | §13.7 |
| l1995 Panic | §13.7 |
| l1996 Suspension | §13.7 |
| l1997 Foreign calls | §13.7 |
| l1998 Unknown effects | §13.7 |
| l2000 Effect token propagation, combination, removal | §13.7 |
| l2006 Semantic type | §13.8 |
| l2007 Generic substitution | §13.8 |
| l2008 Known value constraints | §13.8 |
| l2009 Integer range information | §13.8 |
| l2010 Nullability information | §13.8 |
| l2011 Enum variant information | §13.8 |
| l2012 Ownership category | §13.8 |
| l2013 Relevant lifetime metadata | §13.8 |
| l2014 Representation-independent metadata | §13.8 |
| l2018 Direct calls | §13.9 |
| l2019 Indirect calls | §13.9 |
| l2020 Generic calls | §13.9 |
| l2021 Intrinsics (calls) | §13.9 |
| l2022 Foreign calls | §13.9 |
| l2023 Trait calls | §13.9 |
| l2024 Closure calls | §13.9 |
| l2025 Recursive calls | §13.9 |
| l2026 Calls with panic edges | §13.9 |
| l2027 Calls with allocation effects | §13.9 |
| l2028 Calls with unknown effect summaries | §13.9 |
| l2034 Ownership transitions | §13.10 |
| l2035 Move origins | §13.10 |
| l2036 Borrow origins | §13.10 |
| l2037 Lifetime regions | §13.10 |
| l2038 Initialization states | §13.10 |
| l2039 Destruction obligations | §13.10 |
| l2040 Pinning constraints | §13.10 |
| l2041 Address observability facts | §13.10 |
| l2042 Object identity facts | §13.10 |
| l2050 Graph structure (verify) | §13.11 V1 |
| l2051 Node input types (verify) | §13.11 V2 |
| l2052 Node output types (verify) | §13.11 V3 |
| l2053 Control-flow validity (verify) | §13.11 V4 |
| l2054 Dominance (verify) | §13.11 V5 |
| l2055 Merge validity (verify) | §13.11 V6 |
| l2056 Memory SSA consistency (verify) | §13.11 V7 |
| l2057 Effect SSA consistency (verify) | §13.11 V8 |
| l2058 Call signatures (verify) | §13.11 V9 |
| l2059 Return types (verify) | §13.11 V10 |
| l2060 Initialization constraints (verify) | §13.11 V11 |
| l2061 Ownership constraints (verify) | §13.11 V12 |
| l2062 Safety metadata (verify) | §13.11 V13 |
| l2063 Unreachable control (verify) | §13.11 V14 |
| l2064 Graph termination rules (verify) | §13.11 V15 |
| l2074 Constant folding | §13.12 Simplification |
| l2075 Constant propagation | §13.12 Simplification |
| l2076 Identity elimination | §13.12 Simplification |
| l2077 Algebraic simplification | §13.12 Simplification |
| l2078 Canonicalization | §13.12 Simplification |
| l2079 Redundant conversion elimination | §13.12 Simplification |
| l2083 Common subexpression elimination | §13.12 Value optimization |
| l2084 Global value numbering | §13.12 Value optimization |
| l2085 Conditional constant propagation | §13.12 Value optimization |
| l2086 Redundant computation elimination | §13.12 Value optimization |
| l2087 Value numbering through control flow | §13.12 Value optimization |
| l2091 Branch simplification | §13.12 Control optimization |
| l2092 Branch elimination | §13.12 Control optimization |
| l2093 Unreachable block elimination | §13.12 Control optimization |
| l2094 Merge simplification | §13.12 Control optimization |
| l2095 Loop simplification | §13.12 Control optimization |
| l2096 Control dependency reduction | §13.12 Control optimization |
| l2097 If conversion, when legal | §13.12 Control optimization |
| l2101 Load elimination | §13.12 Memory optimization |
| l2102 Store elimination | §13.12 Memory optimization |
| l2103 Store-to-load forwarding | §13.12 Memory optimization |
| l2104 Dead store elimination | §13.12 Memory optimization |
| l2105 Memory dependence refinement | §13.12 Memory optimization |
| l2106 Alias analysis | §13.12 Memory optimization |
| l2107 Memory state simplification | §13.12 Memory optimization |
| l2108 Redundant initialization elimination | §13.12 Memory optimization |
| l2112 Effect dependency simplification | §13.12 Effect optimization |
| l2113 Dead effect elimination | §13.12 Effect optimization |
| l2114 Effect-aware code motion | §13.12 Effect optimization |
| l2115 Call effect propagation | §13.12 Effect optimization |
| l2116 Effect summary refinement | §13.12 Effect optimization |
| l2120 Loop invariant code motion | §13.12 Loop optimization |
| l2121 Induction-variable analysis | §13.12 Loop optimization |
| l2122 Strength reduction | §13.12 Loop optimization |
| l2123 Loop peeling | §13.12 Loop optimization |
| l2124 Loop unrolling | §13.12 Loop optimization |
| l2125 Loop unswitching | §13.12 Loop optimization |
| l2126 Loop fusion, if legal | §13.12 Loop optimization |
| l2127 Loop vectorization preparation | §13.12 Loop optimization |
| l2131 Inlining | §13.12 Function optimization |
| l2132 Inlining cost models | §13.12 Function optimization |
| l2133 Devirtualization | §13.12 Function optimization |
| l2134 Tail-call recognition | §13.12 Function optimization |
| l2135 Call graph optimization | §13.12 Function optimization |
| l2136 Interprocedural propagation | §13.12 Function optimization |
| l2142 Preconditions | §13.13 |
| l2143 Required analyses | §13.13 |
| l2144 Semantic preservation conditions | §13.13 |
| l2145 Effect preservation | §13.13 |
| l2146 Memory preservation | §13.13 |
| l2147 Ownership preservation | §13.13 |
| l2148 Panic preservation | §13.13 |
| l2149 Termination implications | §13.13 |
| l2150 Debug information preservation | §13.13 |
| l2151 Expected benefit | §13.13 |
| l2152 Cost model | §13.13 |
| l2153 Verification strategy | §13.13 |

Bullets in scope: 216, defined: 216; prose rows: 1 (l2000).
