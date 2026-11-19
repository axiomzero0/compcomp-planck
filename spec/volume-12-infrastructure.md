CEP:FILE: spec/volume-12-infrastructure.md
CEP:WHAT: Normative specification of Planck compiler-wide infrastructure: type and symbol representation, graph infrastructure, memory management, diagnostics, pass infrastructure, metadata, internal APIs, the compiler limits table, the incremental cache contract, and the determinism contract (design.md §19).
CEP:WHY: design.md §19 lists 88 items describing systems that are painful to retrofit; without exact interning math, a limits table, a fixed-point termination contract, and cache invalidation rules the compiler has no G11/G13/G14/G23 guarantees.
CEP:STATUS: complete
CEP:FAILURE: An unspecified interner invariant, a missing limit value, a fixed-point pass without a termination measure, or an incomplete invalidation rule is a conformance failure of this volume.
CEP:ASSUMES: SPEC-INDEX.md §1 method, §3 settled decisions (monomorphization depth 32, instance limit defined here), §5 pinned contracts (graph math, effect row encoding); Volumes 01–11 for consumed data; Volumes 13–17 for IR and object formats.
CEP:COST: document; the limits table is binding on every compiler component (G13).
CEP:EVIDENCE: verification matrix OB-12-1..OB-12-14; coverage ledger (bullets in scope: 88, defined: 88).

---

## 12.0 Scope, components, and traceability

This volume defines design.md §19 "Core compiler infrastructure" (lines 1723–1841). Bullet census: §19.1 = 11, §19.2 = 8, §19.3 = 16, §19.4 = 8, §19.5 = 13, §19.6 = 12, §19.7 = 10, §19.8 = 10; total 88. Section mapping: 12.n ↔ §19.n for n = 1..8. Sections 12.9–12.11 define the limits table (G13), the incremental cache contract (G23), and the determinism contract (G14); they are the definition homes for the resource, caching, and determinism requirements that design.md places on this and every other volume.

Component CEP classes (standards.md §5): interners, the graph kernel, arenas, and the ownership/effect transition cores are **CEP-0**; the pass manager, diagnostics emission, metadata read/write, and driver plumbing are **CEP-1**; DOT export, profiling output, and coverage tooling are **CEP-2**.

Diagnostic identifiers use block PLC-0700..PLC-0799 (registry: §12.5).

## 12.1 Type representation infrastructure (design §19.1)

**Interning math** (normative, CEP-0): the interner is a hash-consing table over canonical serializations. Hash = FNV-1a-64: `h = 0xcbf29ce484222325`; per byte `h ^= b; h *= 0x100000001b3` (mod 2^64). Table: open addressing, linear probing, capacity a power of two, slot = h & (cap − 1); rehash when count > 3·cap/4 (load factor ≤ 0.75), doubling capacity. Interner uniqueness: `intern(x) = intern(y) ⟺ x = y` (OB-12-1). The same hash and table law governs the symbol interner (§12.2).

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Type interning | Every type is interned by its canonical serialization; TypeId = u32, dense per workspace; TypeId 0 is the recovery/ poison type (Volume 11 §11.8). |
| Type IDs | TypeId space = 2^32; exhaustion is PLC-0711 (never reachable before limit L12 binds). |
| Type canonicalization | Canonical form: substitute all inference variables with their solutions (poison stays 0), normalize associated types through solved obligations, order effect rows canonically (sorted atom bitmask, §12.1 effect encoding), intern. Canonicalization is idempotent. |
| Generic substitution | Structural fold over the type tree; substitution depth ≤ 32 (pinned monomorphization depth, SPEC-INDEX §3); deeper is PLC-0656 (Volume 11 §11.9). Substitution results are cached per (TypeId, substitution). |
| Type normalization | Associated-type projections are normalized exactly once, via the solver result recorded in the handoff (Volume 11 §11.14); normalized results are interned and cached. |
| Type equality | Equality is TypeId equality after canonicalization — O(1); no structural comparison occurs after interning. |
| Type hashing | Hash = FNV-1a-64 over the canonical serialization (same law as interning); used only for cache keys, never for output ordering (G14, §12.11). |
| Recursive type handling | Types reference their components by TypeId; cycles are represented by indirection and are rejected at ADT definition time by the sizedness rules of Volume 04; the interner itself accepts no cyclic serializations. |
| Lifetime representation | RegionId = u32; 0 = `'static`; regions are assigned by the frontend (Volume 11 §11.13); limit L17 = 65536 per function. |
| Effect representation | A row is u8 bitmask over the 8 pinned atoms in fixed bit order: bit0 io, bit1 alloc, bit2 panic, bit3 unsafe, bit4 async, bit5 concurrency, bit6 ffi, bit7 nondet; plus one open-row flag bit stored alongside (SPEC-INDEX §5.3; the atom set is closed). |
| Type serialization | Canonical byte format: one tag byte per constructor (tag table in the crate `plkc_types`), uLEB128 lengths, fixed atom bit order; round-trip `read(write(x)) = x` (OB-12-14). |

## 12.2 Symbol infrastructure (design §19.2)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Symbol IDs | SymId = u32, dense per workspace, assigned in discovery order; limit L26 = 2^32 − 1 symbols. |
| Symbol tables | Table maps SymId → (name string, kind ∈ {function, static, type, field, macro, label}, visibility, linkage, DefId); the table is arena-backed and frozen after resolution. |
| Mangling | Exact grammar below; version tag `1`; mangling is injective over canonical paths (OB-12-3). |
| Demangling | A total parser over the same grammar; `demangle(mangle(x)) = x` for every canonical path (OB-12-4); invalid input is a defined error value, not a panic. |
| Symbol visibility | Per Volume 10 §10.8.9: default hidden; exported iff `pub` and crate-root-reachable or `#[no_mangle]`/`#[export_name]`. |
| Linkage names | `#[export_name]` overrides verbatim; `#[no_mangle]` = source name; otherwise mangled name; a collision between an explicit name and another symbol's name is a defined rejection PLC-0704 at link setup. |
| Generic instance symbols | A monomorphized instance's symbol = mangled name of the canonical path with substituted arguments (generic productions of the grammar); instance symbols are created per instantiation, at most L11 = 65536 per crate. |
| Incremental symbol identity | SymId of an instance = low 32 bits of FNV-1a-64 over its mangled name; stable across incremental sessions; collisions resolved by appending the discovery ordinal (deterministic order, §12.11). |

**Mangling grammar** (exact; identifiers are length-prefixed ASCII):

```
MangledName ::= "_P" "1" "_" Ident EncodedPath
EncodedPath ::= Segment ( "." Segment )*
Segment     ::= Ident [ "<" ArgList ">" ]
Ident       ::= Count Char+                 -- Count = decimal byte length
ArgList     ::= Arg ( "," Arg )*
Arg         ::= Type | ConstArg
Type        ::= Ident                       -- nominal type path
              | "u" Width | "i" Width       -- Width ∈ {8,16,32,64}
              | "f" Width                   -- Width ∈ {32,64}
              | "R" Type | "M" Type         -- shared / mutable reference
              | "P" Type | "Q" Type         -- const / mut raw pointer
              | "A" Count "_" Type          -- array
              | "S" Type                    -- slice
              | "T" ArgList                 -- tuple
              | "F" Count "_" ArgList       -- fn pointer
              | "C" Ident                   -- closure instance
              | "E"                         -- never
ConstArg    ::= "V" Count "_" Char+         -- decimal value text
```

Lifetime arguments are erased (not mangled); generic depth ≤ 32 (L10); nested generic arguments are allowed.

## 12.3 Graph infrastructure (design §19.3)

The graph infrastructure serves both Sea-of-Nodes IRs (Volumes 13–14) and any pass-local graph; its math kernel is `planck-graph-math` (SPEC-INDEX §5.7).

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Node IDs | NodeId = u32, dense per graph, assigned in creation order; limit L13 = 2^24 nodes per function, L12 = 2^30 per crate. |
| Edge representation | A node stores its inputs as an array of (NodeId, slot); edges are stored once, use lists are the derived view (no dual storage to desynchronize). |
| Input slots | Arity is fixed per node kind; slot i's semantics are defined by the owning IR volume; arity mismatch is a verifier failure. |
| Output values | Exactly one value per node; multi-result operations are represented by distinct projection nodes. |
| Use lists | `uses(def) -> &[(NodeId, slot)]`, maintained incrementally on replace; invariant: for every (u, s) in uses(d), inputs(u)[s] = d. |
| Def-use relationships | The def-use view is exactly the use-list mapping; no third representation exists. |
| Graph ownership | One arena per graph (per function), owned by the pass pipeline (§12.4). |
| Graph mutation | Mutations: append (new NodeId = count), replace, delete; any other structural change is an ICE. |
| Node replacement | `replace(n, m)` rewires every use of n to m in one pass over uses(n); n becomes unreferenced. |
| Node deletion | Deletion is permitted only for unreferenced nodes; deleting a referenced node is a verifier failure PLC-0713; unreferenced nodes are reclaimed at function end. |
| Graph cloning | Subgraph copy with an old→new remap table built in traversal order (deterministic, G14). |
| Graph traversal | Iterative DFS, pre-order default, children ordered by ascending slot then ascending NodeId; post-order on request; no recursion (CEP-0, bounded stack). |
| Graph serialization | Text format, one line per node in ID order: `n<id> <kind> [<slot:nodeid>...]`; the format is the IR dump format referenced by §12.6. |
| Graph verification | The verifier asserts: every input ID exists; arity matches the kind; use lists are consistent; kind-specific acyclicity holds where required. Failure = PLC-0713 (ICE under `--verify-passes`, OB-12-12). |
| Graph visualization | DOT export via `plkc ir-dump --dot` (CEP-2); node order and colors are fixed by kind table. |
| Graph hashing | Hash = FNV-1a-64 over the canonical serialization (§12.3 serialization order). |

## 12.4 Memory infrastructure (design §19.4)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Arena allocation | Bump arenas: initial chunk 65536 bytes, doubling to a maximum chunk of 16777216 bytes (16 MiB); allocations aligned to ≤ 64 bytes; no per-object free. |
| Interning storage | Each interner owns a private arena; the arena is frozen (immutable) when the interner freezes; interned data never moves. |
| Temporary allocation | One scratch arena per pass; reset to empty at pass end; scratch data never escapes the pass. |
| Memory reclamation | Reclamation happens only at phase boundaries by dropping whole arenas; there is no garbage collector and no incremental free. |
| Compilation-unit ownership | Arena tree: crate arena → per-item arenas → per-function graph arenas; a child arena dies before its parent; cross-arena references are by ID, never by pointer. |
| Parallel-safe allocation | Parallel passes use per-thread arenas; shared results are merged in deterministic order (ascending thread index, then task id, §12.11). |
| Memory profiling | `--profile-mem` prints per-arena peak bytes to stderr in fixed order (CEP-2 output). |
| Peak-memory tracking | High-water mark = sum of live chunk bytes, updated on every chunk allocation, read at phase end; global peak compared against limit L19. |

## 12.5 Diagnostics infrastructure (design §19.5)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Diagnostic IDs | Form `PLC-NNNN` (u16 numeric part). Block allocation (closed registry): 0000–0099 this volume; 0100–0199 Volumes 01–02; 0200–0399 Volumes 03–08; 0400–0499 Volume 09; 0500–0599 Volume 10; 0600–0699 Volume 11; 0700–0799 this volume; 0800–0899 Volumes 13–17; 0900–0999 Volumes 18–19; 1000–1099 Volumes 20–26. Unassigned blocks are reserved; allocation outside the table is an edition-review event. This volume's named codes: PLC-0700 limit exceeded (message names the limit row); PLC-0701 fixed-point pass non-convergence (warning); PLC-0702 incremental cache corruption (cache dropped, full rebuild); PLC-0703 internal compiler error; PLC-0704 linkage-name collision; PLC-0710 arena exhaustion; PLC-0711 interner ID space exhausted; PLC-0712 metadata round-trip failure; PLC-0713 graph verification failure. |
| Severity | Closed: {error, warning, note, help, ice}; only error, warning, and ice are emitted standalone. |
| Source spans | Every diagnostic has exactly one primary span; spans are Volume 11 §11.2 Spans. |
| Secondary labels | ≤ 8 per diagnostic (limit L23); each label = (span, text). |
| Notes | ≤ 16 per diagnostic (limit L23); notes never carry fix-its. |
| Suggestions | ≤ 4 per diagnostic (limit L23); a suggestion is text plus optional fix-its. |
| Fix-its | ≤ 4 per diagnostic (limit L24); each = (span, replacement text, applicability ∈ {suggested, machine}); only `machine` fix-its may be auto-applied by tools. |
| Diagnostic deduplication | Key = (code, primary span, FNV-1a-64 of message); exact duplicates are dropped; the total dropped count is reported once at compilation end. |
| Diagnostic ordering | Emission order is the sort key (source id, start offset, code) ascending — deterministic (G14, G22); the cap is limit L20 (1024 total, L21 = 256 errors before stopping). |
| Macro expansion tracebacks | The expansion chain (invocation spans, outermost first) is printed after the primary message; depth ≤ 128 (limit L25). |
| Internal compiler errors | Code PLC-0703, exit code 101; message `internal compiler error: <component>: <detail>` plus bug-report template (version, target triple, flags, panic location); the compiler never continues after an ICE. |
| Machine-readable diagnostic output | JSON with fixed field order: `{"code","level","message","spans":[{"file","line_start","col_start","line_end","col_end","label"}],"notes":[],"children":[],"fixits":[{"snippet","applicability"}],"traceback":[]}`; selected by `--error-format=json` (Volume 11 §11.1). |
| Localization, if required | Not required in edition P1: all messages are ASCII English; no translation keys, no locale dependence (Law 1: locale is hidden cost). |

## 12.6 Pass infrastructure (design §19.6)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Pass registration | Passes register in a compile-time static registry; a descriptor is {id, name, reads, writes, invalidates, options, is_fixed_point, measure, max_iterations}; no runtime registration exists. |
| Pass dependencies | Each pass declares the analysis sets it reads, writes, and invalidates; undeclared use of an analysis is an ICE. |
| Pass ordering | Execution order = topological order over the read/write dependency DAG, computed by Kahn's algorithm (planck-graph-math) with tie-break by ascending pass id (deterministic, G14); a registration cycle is an ICE (registration is a build-time bug). |
| Analysis invalidation | Invalidating analysis A marks A and all transitive dependents dirty; dirty analyses are recomputed on next request; invalidation is transitive by construction. |
| Cached analyses | Cache key = (analysis id, input fingerprint); fingerprints per §12.10; a dirty key never serves a cached value. |
| Pass configuration | Per-pass options bind to `-C` flags listed in the descriptor; an unknown flag is a usage error (exit 2). |
| Pass instrumentation | `--times` prints per-pass wall time in execution order (deterministic output, G14). |
| Pass verification | `--verify-passes` runs every pass's self-check hook (invariant assertions per G4, Volume 22); a failed invariant is an ICE (exit 101). |
| Pass profiling | `--pass-stats` prints per-pass node/edge/allocation counters in execution order. |
| Pass debugging | `-Zdump-<stage>` writes the IR text dump of that stage to the output directory (formats: Volumes 13–15). |
| Pass serialization | Pass state is never persisted except through the incremental cache (§12.10); dumps are text only. |
| Failure reporting | A pass failure emits a diagnostic and aborts compilation with exit 1; a pass panic is an ICE (exit 101); no pass may swallow an error. |

**Fixed-point contract (G11).** Every fixed-point pass must declare (a) a termination measure that strictly decreases on every iteration, (b) max_iterations (default 100 = limit L28), and (c) a defined fallback. On reaching max_iterations without convergence the pass emits warning PLC-0701 naming the pass and returns its last iterate; correctness is preserved because every iterate of a semantics-preserving pass is semantics-preserving (G4 obligation per pass, verified in Volume 22). No pass may iterate unboundedly; "no infinite optimization loops" is enforced by this contract (OB-12-6).

## 12.7 Metadata infrastructure (design §19.7)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Debug metadata | DWARF 5 on both initial targets; emitted per `--debug-info` (Volume 11 §11.1); format details Volume 17. |
| Profiling metadata | Emitted only under `--emit=prof-data`; off by default; contains per-function entry/exit instrumentation tables. |
| Source locations | A span table per function is always present in rlib metadata (needed for diagnostics of downstream crates). |
| Optimization annotations | Emitted only under `--opt-record`; per-function decision records (inlining, unrolling); CEP-2 consumption. |
| Safety metadata | Emitted only under `--emit=safety-report`; list of unsafe operations with spans and authority sites. |
| Effect summaries | Canonical effect row per function, always present in rlib metadata (needed for cross-crate effect checking, Volume 07). |
| Ownership metadata | Not emitted in edition P1 (banned): ownership facts live in the frontend handoff (Volume 11 §11.14) and never escape the crate. |
| Target metadata | Triple, feature set, pointer width, endianness in the rlib header (Volume 16 defines the parameter values). |
| ABI metadata | Calling convention per function and extern signatures (Volume 10 §10.8); consumed at link time. |
| Coverage metadata | Counter tables under `--emit=coverage` (Volume 26); off by default. |

## 12.8 Compiler internal APIs (design §19.8)

One crate per API; all are internal (unstable across minor versions). The stable interfaces in edition P1 are exactly: the CLI (Volume 11 §11.1), the JSON diagnostic schema (§12.5), and the mangling grammar (§12.2).

| API | Crate | Core signatures (exact) |
| --- | ----- | ----------------------- |
| AST API | `plkc_ast` | `fn node(&self, id: NodeId) -> &Node`; `fn span(&self, id: NodeId) -> Span`; `struct Visitor` with pre-order `visit(&Node)`; arena-backed, no parent pointers. |
| Semantic type API | `plkc_types` | `fn type_of(&self, id: NodeId) -> TypeId`; `fn canonical(&self, t: TypeId) -> TypeId`; `fn subst(&self, t: TypeId, s: &Subst) -> Result<TypeId, SubstError>` (depth ≤ 32). |
| Frontend analysis API | `plkc_analysis` | `fn fcfg(&self, def: DefId) -> &Fcfg`; `fn ownership(&self, def: DefId) -> &OwnershipFacts`; `fn regions(&self, def: DefId) -> &RegionMap`. |
| IR 1 API | `plkc_ir1` | `fn node(&self, id: NodeId) -> &Ir1Node`; `fn replace(&mut self, n: NodeId, m: NodeId)`; `fn verify(&self) -> Result<(), GraphError>`. |
| IR 2 API | `plkc_ir2` | Same shape as IR 1 with IR 2 node kinds. |
| Machine IR API | `plkc_mir` | `fn insts(&self, block: BlockId) -> &[Inst]`; `fn reg_class(&self, r: PhysReg) -> RegClass`; `fn verify(&self) -> Result<(), MirError>`. |
| Target API | `plkc_target` | `fn from_triple(t: &str) -> Result<Target, TargetError>`; `Target` exposes pointer width, endianness, feature set, ABI tables (Volume 10 §10.8), layout parameters (Volume 05). |
| Pass API | `plkc_pass` | `trait Pass { fn descriptor(&self) -> &Descriptor; fn run(&mut self, ctx: &mut Ctx) -> Result<(), PassError>; fn verify(&self, ctx: &Ctx) -> Result<(), PassError>; }`. |
| Diagnostic API | `plkc_diag` | `fn emit(&mut self, d: Diagnostic)`; `Diagnostic` carries code, level, spans, notes, suggestions, fix-its per §12.5; ordering applied at drain time. |
| Metadata API | `plkc_meta` | `fn read(&self, kind: RecordKind) -> Result<Record, MetaError>`; `fn write(&mut self, r: Record) -> Result<(), MetaError>`; round-trip guaranteed (OB-12-14). |

## 12.9 Compiler limits table (G13)

Every limit is exact; exceeding any limit is a defined compilation failure — diagnostic PLC-0700 naming the limit row, exit code 1 — never a crash (G13; resource exhaustion on untrusted input is handled, not uncontrolled).

| Row | Limit | Exact value |
| --- | ----- | ----------- |
| L01 | Max source files per crate | 65536 |
| L02 | Max source file size | 268435456 bytes (256 MiB) |
| L03 | Max AST depth | 256 |
| L04 | Max parser backtrack lookahead | 8 tokens |
| L05 | Max macro expansion depth | 128 |
| L06 | Max macro tokens emitted per crate | 1048576 (2^20) |
| L07 | Max const-eval steps per request | 10^7 |
| L08 | Max const-eval call depth | 512 |
| L09 | Max const-eval memory per request | 2^20 words (8 MiB) |
| L10 | Max generic instantiation depth | 32 |
| L11 | Max monomorphized instances per crate | 65536 |
| L12 | Max IR nodes per crate (IR 1 + IR 2 + MIR summed) | 1073741824 (2^30) |
| L13 | Max IR nodes per function | 16777216 (2^24) |
| L14 | Max compiler-internal recursion depth | 4096 frames |
| L15 | Max trait-solver steps per goal | 10^6 |
| L16 | Max trait-solver depth | 128 |
| L17 | Max regions per function | 65536 |
| L18 | Max compile time, wall clock, per crate | 600 s |
| L19 | Max peak compiler memory | 16 GiB |
| L20 | Max diagnostics per compilation | 1024 |
| L21 | Max errors before stopping | 256 |
| L22 | Max diagnostic message length | 4096 bytes |
| L23 | Max labels / notes / suggestions per diagnostic | 8 / 16 / 4 |
| L24 | Max fix-its per diagnostic | 4 |
| L25 | Max macro expansion traceback depth | 128 |
| L26 | Max symbols per workspace | 4294967295 (2^32 − 1) |
| L27 | Max promoted const value size | 65536 bytes |
| L28 | Max fixed-point iterations per pass | 100 |
| L29 | Max type-inference steps per function body | 1048576 (2^20) |
| L30 | Max const-eval arena per crate | 268435456 bytes (256 MiB) |

Wall-clock enforcement: the 600 s budget (L18) is checked at stage boundaries; a stage that overshoots is interrupted at its next internal checkpoint and reported as PLC-0700. Memory enforcement: arenas check L19 at chunk allocation.

## 12.10 Incremental compilation cache invalidation contract (G23)

The unit of caching is the **query** (a named computation of any stage: lexing, parsing, resolution, type checking, trait solving, const evaluation, pass analyses). For query q, the fingerprint is `f(q) = FNV-1a-64(query kind, canonical key, sorted list of f(input))` over q's transitive inputs. The cache stores (f(q), result bytes) with a content checksum.

Rules (all normative):

1. On rebuild, f(q) is recomputed: if it equals the stored fingerprint the entry is green and its result is reused; otherwise the query is red, is recomputed, and its dependents are marked red (transitively).
2. Input kinds and their fingerprints: source files = content hash (§12.1 hash law); compiler flags = canonical flag string; environment records = the env-record of Volume 09 §9.1.13; impl sets = the impl-set fingerprint of Volume 11 §11.9; dependency metadata = metadata content hash (§12.7).
3. Whole-cache invalidation events: compiler version change, target triple change, any flag change, change of the cache directory. After any of these the cache is dropped and a full compilation runs.
4. A checksum mismatch on read is PLC-0702: the cache is dropped, a full compilation runs, and the run's output is still the contract output (rule 5).
5. **Correctness contract**: the output of an incremental compilation is byte-identical to the output of the corresponding full compilation (OB-12-8, G23). No query may be reused across a changed input; "cache validity" and "dependency invalidation correctness" of G23 are exactly rules 1–4.

## 12.11 Determinism contract (G14)

Given identical (P, Configuration, Target, Toolchain), the compiler produces byte-identical outputs. Rules (all normative; each is enforceable by inspection or test):

| Rule | Requirement |
| ---- | ----------- |
| R1 | No output-affecting hash iteration: any hash map that feeds an output decision is iterated in sorted-key order; unsorted iteration is banned in CEP-0/CEP-1 code. |
| R2 | Every topological or graph order tie-breaks by ascending ID (pass ids, symbol ids, block ids). |
| R3 | Parallel merges concatenate in ascending (thread index, task id) order before any output-affecting use. |
| R4 | The only PRNG in the compiler is a pinned xorshift64* with seed 0x9E3779B97F4A7C15, used exclusively by CEP-2 tooling; it never affects output bytes. |
| R5 | Objects contain no timestamps, no absolute paths, no hostnames; build IDs are content hashes; DWARF `comp_dir` is omitted unless `--record-paths` is given. |
| R6 | Diagnostics are emitted in the fixed order of §12.5 (also G22). |

Determinism is verified by double-compilation byte comparison over the test corpus (OB-12-9). Any profile-guided or search-based feature must seed from R4's PRNG and record the seed, or it is banned; in edition P1 no such feature exists.

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-12-1 | Interner uniqueness: intern(x) = intern(y) ⟺ x = y over a domain of 10^6 canonical serializations. | EXH | FV-0 | tests/vol12/interner | G14 |
| OB-12-2 | Interning table law: rehash at count > 3·cap/4, slot = h & (cap − 1), no entry lost across rehashes for insertion sequences to 10^6. | EXH | FV-0 | tests/vol12/intern-math | G12 |
| OB-12-3 | Mangling injectivity: distinct canonical paths of ≤ 6 segments and ≤ 32 generic depth map to distinct mangled names. | Z3 (structural) + EXH over generated paths | FV-0 | tools/fv/ob12-03.z3 | G24 |
| OB-12-4 | Demangler totality: demangle(mangle(x)) = x for every canonical path in the EXH domain; malformed inputs return errors, never panic. | EXH | FV-0 | tests/vol12/demangle | G24 |
| OB-12-5 | The pass dependency DAG is acyclic and its topological order is the unique Kahn order with ascending-id tie-break. | Z3 + EXH over generated DAGs | FV-0 | tools/fv/ob12-05.z3 | G11, G14 |
| OB-12-6 | Every fixed-point pass declares a strictly decreasing measure and max_iterations; non-convergence yields PLC-0701 and a semantics-preserving iterate. | REVIEW of descriptors + Z3 (measure model) | FV-1 | tools/fv/ob12-06.z3 | G11, G4 |
| OB-12-7 | Exceeding any limit L01–L30 produces PLC-0700 with exit 1 and never a crash or hang. | EXH (boundary test per limit) | FV-0 | tests/vol12/limits | G13 |
| OB-12-8 | Incremental cache reuse occurs iff the fingerprint is unchanged, and incremental output is byte-identical to full output. | EXH (small) + PROP (paired) | FV-0 | tests/vol12/incremental | G23 |
| OB-12-9 | Identical (P, Configuration, Target, Toolchain) produce byte-identical outputs across the corpus. | EXH (double-compile byte compare) | FV-0 | tests/vol12/determinism | G14 |
| OB-12-10 | Diagnostic emission order equals the (source id, start offset, code) sort order for every diagnostic set. | EXH | FV-0 | tests/vol12/diag-order | G22 |
| OB-12-11 | Arena contracts hold: chunk growth ≤ 16 MiB, no per-object free, peak accounting equals live chunk bytes at every phase end. | REVIEW + EXH (instrumented arenas) | FV-1 | tests/vol12/arenas | G12 |
| OB-12-12 | The graph verifier detects every violation of the §12.3 invariant list (dangling input, arity mismatch, inconsistent use list, forbidden cycle). | EXH over systematically mutated graphs | FV-0 | tests/vol12/graph-verify | G4 |
| OB-12-13 | Malformed and adversarial inputs never corrupt compiler memory (no out-of-bounds, no use-after-free, no overflow-induced allocation). | PROP (deterministic fuzzing) paired with REVIEW | FV-1 | tests/vol12/fuzz | G12, G17 |
| OB-12-14 | Metadata round-trip: read(write(x)) = x for every record kind of §12.7 over the corpus. | EXH + PROP (paired) | FV-0 | tests/vol12/meta-roundtrip | G23 |

## Coverage ledger

| Bullet (design.md §19, abbreviated) | Defined in |
| ----------------------------------- | ---------- |
| Type interning | §12.1 |
| Type IDs | §12.1 |
| Type canonicalization | §12.1 |
| Generic substitution | §12.1 |
| Type normalization | §12.1 |
| Type equality | §12.1 |
| Type hashing | §12.1 |
| Recursive type handling | §12.1 |
| Lifetime representation | §12.1 |
| Effect representation | §12.1 |
| Type serialization | §12.1 |
| Symbol IDs | §12.2 |
| Symbol tables | §12.2 |
| Mangling | §12.2 |
| Demangling | §12.2 |
| Symbol visibility | §12.2 |
| Linkage names | §12.2 |
| Generic instance symbols | §12.2 |
| Incremental symbol identity | §12.2 |
| Node IDs (graph) | §12.3 |
| Edge representation | §12.3 |
| Input slots | §12.3 |
| Output values | §12.3 |
| Use lists | §12.3 |
| Def-use relationships | §12.3 |
| Graph ownership | §12.3 |
| Graph mutation | §12.3 |
| Node replacement | §12.3 |
| Node deletion | §12.3 |
| Graph cloning | §12.3 |
| Graph traversal | §12.3 |
| Graph serialization | §12.3 |
| Graph verification | §12.3 |
| Graph visualization | §12.3 |
| Graph hashing | §12.3 |
| Arena allocation | §12.4 |
| Interning storage | §12.4 |
| Temporary allocation | §12.4 |
| Memory reclamation | §12.4 |
| Compilation-unit ownership | §12.4 |
| Parallel-safe allocation | §12.4 |
| Memory profiling | §12.4 |
| Peak-memory tracking | §12.4 |
| Diagnostic IDs | §12.5 |
| Severity | §12.5 |
| Source spans (diagnostics) | §12.5 |
| Secondary labels | §12.5 |
| Notes | §12.5 |
| Suggestions | §12.5 |
| Fix-its | §12.5 |
| Diagnostic deduplication | §12.5 |
| Diagnostic ordering | §12.5 |
| Macro expansion tracebacks | §12.5 |
| Internal compiler errors | §12.5 |
| Machine-readable diagnostic output | §12.5 |
| Localization, if required | §12.5 (not required in P1) |
| Pass registration | §12.6 |
| Pass dependencies | §12.6 |
| Pass ordering | §12.6 |
| Analysis invalidation | §12.6 |
| Cached analyses | §12.6 |
| Pass configuration | §12.6 |
| Pass instrumentation | §12.6 |
| Pass verification | §12.6 |
| Pass profiling | §12.6 |
| Pass debugging | §12.6 |
| Pass serialization | §12.6 |
| Failure reporting | §12.6 |
| Debug metadata | §12.7 |
| Profiling metadata | §12.7 |
| Source locations (metadata) | §12.7 |
| Optimization annotations | §12.7 |
| Safety metadata | §12.7 |
| Effect summaries | §12.7 |
| Ownership metadata | §12.7 |
| Target metadata | §12.7 |
| ABI metadata | §12.7 |
| Coverage metadata | §12.7 |
| AST API | §12.8 |
| Semantic type API | §12.8 |
| Frontend analysis API | §12.8 |
| IR 1 API | §12.8 |
| IR 2 API | §12.8 |
| Machine IR API | §12.8 |
| Target API | §12.8 |
| Pass API | §12.8 |
| Diagnostic API | §12.8 |
| Metadata API | §12.8 |

bullets in scope: 88, defined: 88
