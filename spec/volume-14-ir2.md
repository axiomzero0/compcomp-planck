<!-- CEP:FILE: spec/volume-14-ir2.md
CEP:WHY: design.md §21 (lines 2159–2460) requires a complete normative definition of IR 2, the heterogeneous Sea of Nodes: representation system, representation selection, aggregate/object/allocation optimization, specialization, escape analysis, vectorization, the pass catalog on the 10-row template of design.md §40, and the IR 2 verifier. This volume is that definition; every §21 bullet has exactly one home here.
CEP:STATUS: complete
CEP:FAILURE: An IR 2 graph that fails verifier check W1–W12 of §14.15 is rejected; a pass that emits an illegal representation, breaks an observable identity, or drops a destruction obligation is non-conforming and must be corrected, not the rule bent. An omitted §21 bullet in the coverage ledger is a missing definition.
CEP:ASSUMES: SPEC-INDEX.md §1 (method), §3 (settled decisions: monomorphization-only with depth 32, provenance model, identity = allocation ID + offset, effect atoms), §5.7 (pinned graph math, embedded in Volume 13 §13.2 and normative here); Volume 13 (IR 1 invariants I1–I12, metadata, pass costs); design.md §40 (10-row pass template); standards.md §3; verification standards.md G4/G5/G11/G13/G14/G15; Volumes 04 (layouts), 05 (value/memory semantics, layouts), 08 (concurrency), 10 (ABI), 12 (limits), 15 (Machine IR boundary), 16 (vtables, target features).
CEP:COST: document. Fixes compile-time budgets (§14.16) charged against the Volume 12 limits table and per-pass costs in §14.14.
CEP:EVIDENCE: Verification matrix (OB-14-1..OB-14-16) and coverage ledger (bullets in scope: 212) at the end of this file.
CEP:FILE: spec/volume-14-ir2.md -->

# Volume 14 — IR 2: Heterogeneous Sea of Nodes

## 14.0 Scope and traceability

This volume defines design.md §21 "IR 2 complete specification", lines 2159–2460, including the header prose (2159–2168) and the deliverables line (2457). It contains 212 bullets over subsections §21.1–§21.16. Mapping:

| design.md subsection | Lines | Bullets | Defined in |
| --- | --- | --- | --- |
| 21.1 IR 2 purpose and boundaries | 2169–2180 | 8 | §14.1 |
| 21.2 Heterogeneous graph model | 2182–2195 | 12 | §14.2 |
| 21.3 Representation type system | 2197–2228 | 25 (16 reps + 9 facets) | §14.3 |
| 21.4 Representation selection | 2230–2258 | 17 (12 inputs + 5 outputs) + 2 prose requirements | §14.4 |
| 21.5 Aggregate optimization | 2259–2272 | 12 | §14.5 |
| 21.6 Object optimization | 2274–2288 | 13 | §14.6 |
| 21.7 Allocation optimization | 2290–2304 | 13 | §14.7 |
| 21.8 Type representation optimization | 2306–2320 | 13 | §14.8 |
| 21.9 Generic specialization | 2322–2337 | 14 | §14.9 |
| 21.10 Closure optimization | 2339–2351 | 11 | §14.10 |
| 21.11 Dynamic dispatch optimization | 2353–2362 | 8 | §14.11 |
| 21.12 Escape and lifetime optimization | 2364–2375 | 10 | §14.12 |
| 21.13 Vectorization and data layout | 2377–2389 | 11 | §14.13 |
| 21.14 IR 2 optimization passes | 2391–2415 | 21 | §14.14 |
| 21.15 IR 2 verification | 2417–2434 | 12 | §14.15 |
| 21.16 Optimization profitability | 2436–2456 | 12 + 2 prose requirements | §14.16 |

Bullets in scope: 212, defined: 212 (ledger at end of file; the ledger additionally maps 3 prose requirements — l2257 selection cost models, l2453 removal-is-not-goodness, l2455 inlining example — and the deliverables line l2457, mapped below). Deliverables of line 2457 map to: IR 2 Semantics Specification = §14.1; Representation System Specification = §14.2, §14.3; Specialization Specification = §14.9; Object Optimization Specification = §14.5–§14.7, §14.10–§14.12; Pass Catalog = §14.14; Verifier = §14.15.

## 14.1 IR 2 purpose and boundaries (§21.1)

| Bullet | Normative definition |
| --- | --- |
| Semantic information inherited from IR 1 | IR 2 inherits, and must preserve verbatim unless a pass row licenses a change: semantic types, effect rows and the single effect chain (Volume 13 §13.7), memory partitions and MemState SSA (Volume 13 §13.6), provenance metadata, ownership metadata and Drop obligations (Volume 13 §13.10), initialization facts, identity and pinning facts, debug spans, and the handoff axioms of Volume 11 §18.14. |
| Representation information introduced in IR 2 | IR 2 introduces: the representation types of §14.3, layout decisions (field offsets, sizes, alignments, discriminant encodings), virtual objects, materialization operations, and conversion plans. Every introduced fact must be derivable from inherited facts plus target data; inventing facts is banned. |
| Target-independent versus target-aware decisions | Target-independent in IR 2: all layout and representation decisions whose inputs are sizes/alignments from the target data layout table only (the decisions are made once per target, not per function); niche and enum layout selection; specialization. Target-aware inputs permitted in IR 2: the target's register counts, vector register widths, and the cost weights of §14.16. Banned in IR 2: instruction selection, scheduling, register assignment — these begin in Volume 15/16. IR 2 must not emit target opcodes. |
| Permitted representation transformations | Exactly the passes of §14.14; any other representation change is banned. A transformation is permitted only if it preserves semantics per G5 and every check of §14.15 holds on output. |
| Semantic preservation requirements | For every pass T and input x: ⟦T(x)⟧ = ⟦x⟧ over the observable behavior of Volume 13 §13.1 (returned value, live memory, I/O, panic/unwind, traps), modulo the resource-freedom assumption of §14.7 (pinned here: executions that exhaust stack or heap are outside the equivalence domain; this is the RF domain and Volume 05/18 must conform). Identity observations and pinning must be preserved where observable. |
| When layout decisions become fixed | Layout fixation is the state after the pass LayoutOptimization (§14.14) has run for a function. Before fixation, layout may be re-decided by re-selection (§14.4). After fixation, any decision embedded in a materialization, field access, or discriminant encoding must not change; re-selection may only move values toward materialization (toward-physical monotonicity). |
| When physical storage becomes mandatory | A value must have physical storage exactly when: (a) its address is observable (AddrObserved), (b) its identity is observable, (c) it is pinned, (d) it crosses an ABI boundary (call argument or return of a type the ABI does not pass in registers), (e) it is accessed by a volatile or atomic operation, (f) it is captured by an escaping closure or a Suspend, (g) it has drop glue and its Drop survives optimization, or (h) it reaches the IR 2 → MIR boundary (W12). |
| Which invariants must survive lowering | The following must hold on every IR 2 graph handed to MIR: I1–I12 of Volume 13 §13.11 (adapted: IR 2 keeps one Start, one terminal, chain discipline, dominance); W1–W12 of §14.15; every value physical per the rule above; provenance metadata still attached to every pointer; no VirtualObject remains. |

## 14.2 Heterogeneous graph model (§21.2)

IR 2 retains the Sea-of-Nodes backbone of Volume 13 §13.2 (ownership, identity, edge classes, ordering, entry/exit, regions) and adds representation typing:

| Bullet | Normative definition |
| --- | --- |
| Multiple value representations | Each value edge carries a representation tag R ∈ Repr (§14.3). The same semantic type may appear under different representations at different edges; the representation is metadata, never a new semantic type. |
| Multiple aggregate representations | An aggregate value may exist simultaneously as MultiScalarAgg, RegisterAgg, Vector (lanes), VirtualObject, or materialized Stack/HeapObject, with conversion nodes between (§14.3 conversions). |
| Different memory representations | A partition's storage representation is chosen by §14.4: scalar-field storage (per-field partitions after scalar replacement), aggregate storage, or packed storage. Loads/stores must use the partition's selected representation. |
| Virtual objects | A VirtualObject is a compiler-tracked object with no storage: it has a semantic type, a creation site (Alloc, AggNew, or Param), field slots holding values or virtual sub-objects, and an identity record. Virtual objects obey: they must be materialized before any use that requires storage (§14.1 mandatory-physical rule); their field slots may hold only values whose representations are feasible without storage. |
| Materialized objects | A materialized object has a storage location (stack slot or heap allocation) and address; accesses compile to Load/Store. Materialization is the node `Materialize(obj)` producing the address; dematerialization (`Virtualize`) is legal only before layout fixation and only if no mandatory-physical use exists. |
| Scalar values | Representation ImmScalar or PackedScalar (§14.3); all arithmetic operates on them. |
| Vector values | Representation Vector; lane operations only; no field access except lane extract/insert. |
| Address-bearing values | Representations Pointer, Slice, TraitObject, StackObject, HeapObject, AddressStableObject, ForeignLayoutObject; every address-bearing value carries provenance metadata (Volume 13 §13.5) unchanged. |
| Representation conversions | `RepConvert(src, R2)` converts a value from R1 to R2; legal iff (R1→R2) is in the conversion table of §14.3; each conversion must preserve values exactly (W2); conversion cost is in the §14.16 cost model. |
| Representation-specific operations | Each representation lists its Supported operations (§14.3); applying an unsupported operation is a verifier error `IllegalOp`. |
| Representation compatibility | Two representations are compatible iff a conversion path exists in the conversion table with cost ≤ CONV_COST_MAX = 8 units; incompatible pairs cannot meet at a Phi (W-welding rule below). |
| Representation merging at control-flow joins | At a Region/Loop join, `RepPhi` requires all inputs to carry the same representation. If inputs differ, the merge plan converts every input to the representation selected for the Phi's semantic type by §14.4; if that representation is infeasible for some input (no conversion path), the fallback representation of the semantic type (first entry of its §14.3 materialization chain) is used for all inputs. The plan is deterministic: one chosen representation per Phi, conversions inserted in input order. |

## 14.3 Representation type system (§21.3)

The representation set Repr is closed with exactly 16 members. The nine §21.3 facets are the columns. Size/Alignment are in bytes; target-dependent values are marked ⌐t and come from the target data layout table (Volume 16); the columns Copy/Move/Destruction describe behavior of the representation, not new semantics.

| Representation | Supported operations | Size | Align | Field access | Copy | Move | Destruction | Conversion rules | Materialization requirements |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ImmScalar | arith, bit, cmp, conv (Vol 13 §13.4) | 1,2,4,8,16 ⌐t | = size | none (is a leaf) | bitwise | bitwise | none | ↔ Packed, Vector lanes, Pointer via unsafe casts per Vol 05 | none (already physical) |
| MultiScalarAgg | AggNew/Get/Set, MemCopy/MemMove | Σ fields padded ⌐t | max field align | by field index, offsets from layout decision | fieldwise | fieldwise, transfers ownership | drop glue, reverse declaration order | ↔ RegisterAgg, ↔ VirtualObject, ↔ Stack/Heap materialized | Materialize before address-taking |
| PackedScalar | extract, pack, scalar ops after extract | ≤ 8 ⌐t | 1 | none | bitwise | bitwise | none | ↔ ImmScalar (unpack/pack) | unpack at use |
| RegisterAgg | AggGet/Set by slot, direct move | ≤ 16 ⌐t (ABI) | natural | by slot | bitwise pair | pair move | none (POD only; types with drop glue are banned from RegisterAgg) | ↔ MultiScalarAgg | spill to memory at ABI boundary |
| Vector | SIMD ops per Volume 16 feature table, lane extract/insert | 16 or 32 ⌐t | 16 or 32 ⌐t | lanes only | bitwise | bitwise | none | ↔ ImmScalar lanes (splat/extract) | store to aligned memory |
| TaggedValue | tag test, discriminant, payload access after test | payload + tag, padded ⌐t | max(payload, tag) | payload fields after tag check | per payload | per payload | tag dispatch to payload drop | ↔ NicheEncoded, ↔ untagged when proven (§14.8) | materialize full layout |
| NicheEncoded | as TaggedValue; discriminant decodes niche bits | payload size ⌐t | payload align | as TaggedValue | per payload | per payload | as TaggedValue | ↔ TaggedValue | materialize full layout |
| Pointer | PtrOffset/PtrCmp/Load/Store (Vol 13 §13.5) | 8 ⌐t | 8 ⌐t | via Load/Store | bitwise, provenance preserved | transfer | none (raw) | ↔ ImmScalar via unsafe per Vol 05 | none |
| Slice | SliceLen/SliceGet, Load/Store of (ptr,len) | 16 ⌐t | 8 ⌐t | ptr@0, len@8 | shallow | transfers element ownership | element drops + dealloc if owned | split to (Pointer, ImmScalar) | materialize pair |
| TraitObject | vtable dispatch Call, data access via vtable | 16 ⌐t | 8 ⌐t | data@0, vtable@8 | shallow | transfer | vtable drop | ↔ concrete after devirtualization (§14.11) | materialize fat pointer |
| ClosureEnv | Call via env, capture access | Σ captures ⌐t | max capture align | capture i | per captures | per captures, reverse order | per captures | decompose to captures (§14.10) | materialize env object |
| VirtualObject | AggGet/AggSet on virtual fields only | none (no storage) | none | virtual fields | metadata only | metadata only | must be eliminated or materialized before any surviving Drop | materialize (one-way after fixation) | required before MIR (W12) |
| StackObject | Load/Store, AddrOf | exact ⌐t | exact ⌐t | offsets | MemCopy | MemMove | Drop at scope exit | ↔ VirtualObject (virtualization, pre-fixation only) | already physical |
| HeapObject | all memory ops, AddrOf | exact ⌐t | exact ⌐t | offsets | shallow | pointer move | Drop + Dealloc obligation | none upward | already physical |
| AddressStableObject | as HeapObject, minus object moves/copies | exact ⌐t | exact ⌐t | offsets | banned for the object | banned | Drop only | none | already physical; address must not change (Vol 13 §13.10 Pinning) |
| ForeignLayoutObject | Load/Store with exact foreign offsets | per foreign layout ⌐t | per foreign layout | exact offsets | MemCopy | MemMove | no Planck drop glue across FFI (Vol 05) | ↔ Planck layout via checked boundary cast | always physical |

Materialization chains (fallback order used by §14.2 merging and §14.4): for every semantic type the chain is (best first): ImmScalar/Packed → RegisterAgg (if POD and ABI-eligible) → MultiScalarAgg → Vector (if lane type and width fits) → StackObject → HeapObject. The chain is per type, fixed at construction, deterministic.

## 14.4 Representation selection (§21.4)

**Homogeneity judgment (exact).** A subgraph S of the IR 2 value graph is homogeneous iff all four hold: (H1) every value edge inside S carries the same representation R; (H2) no RepConvert node lies inside S; (H3) every value entering S from outside is already in R or is converted at the boundary of S by the merge plan of §14.2; (H4) every memory access in S uses the single storage representation selected for S's partition. Selection proceeds per SSA value v; after selection, the value-flow graph of each Region is scanned: maximal subgraphs whose edges share one representation are homogeneous; remaining conversions sit at boundaries. The selection result is the input to the RepresentationSelection pass (§14.14); homogeneity of each Region's subgraph is checked by W-welding (§14.15).

**Selection inputs (the 12 §21.4 bullets, exact semantics):**

| Input | Definition and source |
| --- | --- |
| Semantic type | TypeId from IR 1 (inherited); fixes the feasible representation set via §14.3 operation coverage. |
| Target architecture | Target data layout table (sizes, alignments, vector widths, register counts); fixed per compilation. |
| Usage patterns | The operation multiset applied to v: from the use list; each op must be in the candidate's Supported set. |
| Escape information | Summary of v's underlying object from §14.6; L0 enables ImmScalar/decomposed reps; forcing address observability forces materialized reps. |
| Address observability | AO ∈ {NotObserved, Virtualizable, Forcing} (§14.6); Forcing forbids non-address representations. |
| Object identity | IdentityObserved flag (Volume 13 §13.10); true forces a materialized object with a stable address. |
| Layout requirements | Explicit repr attributes (repr(P/C/packed/transparent/niche), Volume 05); repr(C)/packed/transparent forbid representation selection for storage (ForeignLayoutObject) and fix layout. |
| Register pressure estimates | RP estimate per Region (live values); candidates increasing RP are penalized by the §14.16 register model. |
| Allocation behavior | Whether v is the direct result of an Alloc and whether that Alloc is eliminated by §14.7; eliminated allocations force non-address reps. |
| ABI restrictions | Call/return positions must use the ABI's representation for the type (RegisterAgg eligibility per Volume 10); ABI positions override all other inputs. |
| Alignment | Required alignment of candidate R must be ≤ the target's natural alignment for the type; unaligned access is allowed only for PackedScalar at cost +1 per access. |
| Access frequency | Static frequency estimates per §14.16; hot values prefer ImmScalar/Vector. |

**Selection outputs (the 5 §21.4 bullets, exact):** Selected representation: the candidate R minimizing cost(R) = Σ_{op u of v} cost(u in R) + Σ conversions inserted, with cost from §14.16; ties broken by smaller size(R), then by earlier position in the materialization chain of §14.3. Materialization requirements: the list of uses requiring storage (from §14.1 mandatory-physical rule) that force R to be address-bearing. Conversion plan: the ordered list of RepConvert nodes to insert at v's uses. Layout decision: field offsets/size/align/discriminant encoding for v's type, per §14.8 algorithms, marked fixed or revisable (fixation, §14.1). Specialization decision: whether v's type triggers a type/layout-specialized instance (§14.9).

**Selection cost model and revision (the §21.4 prose requirement).** The cost model is §14.16; it is the only permitted profitability input. Revision: re-selection runs when an analysis invalidation event fires (escape summary changed, allocation eliminated, alias fact added, fixation not yet reached). Revision rules: (R1) before layout fixation, any revision is allowed; (R2) after fixation, revisions may only move toward materialization (a value may gain storage, never lose it); (R3) a revision must re-run §14.2 merge planning on every Region the value reaches; (R4) at most MAX_RESELECTION_ROUNDS = 8 revision rounds per function, then the current selection is frozen (determinism, G14).

## 14.5 Aggregate optimization (§21.5)

Each bullet is a pass or analysis in §14.14; this subsection fixes the exact legality judgments the passes use. Judgment premises: the object o under optimization is a VirtualObject or MultiScalarAgg with fields f1..fk; views(o, fi) are the pointer-derived field accesses.

| Bullet | Exact legality judgment |
| --- | --- |
| Scalar replacement | Legal for object o iff: summary(o) = L0 and thread=false (§14.6); every access to o is a constant-path field access AggGet/AggSet(o, fi) or a whole-object AggNew/AggCopy; no address of o or of any field is taken beyond those accesses (AO ≠ Forcing); o has no drop glue or all Drops are removable. Effect: replace o by k scalar values; MemStates of o's partition are dropped with the object. |
| Aggregate decomposition | Split MultiScalarAgg value into per-field values across control joins; legal iff every use of the aggregate consumes at most one field or reconstructs it (AggNew); decomposition is undone by Aggregate reconstruction without semantic change. |
| Field promotion | Promote a frequently accessed field to an ImmScalar across its region; legal iff the field's offsets are never observed and no aliasing write can reach the field between its loads (partition analysis). |
| Field elimination | Remove a field whose values are never observable (never loaded, never in a materialized copy, not in the constructor of an escaping value); legal iff o's type layout is not repr(C)/packed/foreign and o does not escape. |
| Field splitting | Split array-of-structs field into k independent scalar arrays (SoA, §14.13); legal iff all accesses are fieldwise with constant field index, no whole-struct address escapes, no identity observation. |
| Aggregate reconstruction | Rebuild an aggregate from decomposed fields at a materialization point; must reproduce the exact fieldwise semantics of AggNew; no observable difference permitted. |
| Aggregate propagation | Forward AggNew field values to AggGet uses (same value flow as store-to-load forwarding on virtual fields); legal iff no intervening write to the field in the partition model. |
| Aggregate merging, when legal | Merge two aggregates with identical layouts and disjoint live ranges into one storage; legal iff their live ranges do not overlap (interval overlap per SPEC-INDEX §5.8 semantics) and neither is pinned nor identity-observed. |
| Aggregate materialization | Insert Materialize per §14.1 mandatory-physical rule; the materialized layout is the current layout decision; after fixation the layout is frozen. |
| Aggregate virtualization | Replace a StackObject whose address is only used for fieldwise access by a VirtualObject; legal only pre-fixation, only if AO = Virtualizable, and the object's storage is then dead. |
| Aggregate copy elimination | Remove AggCopy/MemCopy of an aggregate whose source and target are both virtual (fieldwise copy of values) or whose copy is unobservable (target never read before death); never remove a copy into or out of a ForeignLayoutObject. |
| Partial aggregate optimization | Optimize a strict subset of fields (promote some, keep others stored); legal iff kept fields keep exact storage semantics and the mixed object's layout decision records the split; must materialize to the full layout at ABI/escape boundaries. |

## 14.6 Object optimization (§21.6)

**Escape analysis as a summary-based algorithm (exact).**

State space. For each abstract object o (an Alloc, an AggNew of object type, or a pointer-typed Param) the analysis computes a summary S(o) = (level, thread, addr) with:

level ∈ {L0 NoEscape, L1 ArgEscape, L2 ReturnEscape, L3 HeapEscape, L4 ForeignEscape}, ordered L0 < L1 < L2 < L3 < L4; thread ∈ {false, true}; addr ∈ {NotObserved, Virtualizable, Forcing} ordered NotObserved < Virtualizable < Forcing.

Join (exact): S(o) ⊔ S′(o) = (max(level), thread ∨ thread′, max(addr)). Order on summaries is componentwise; the lattice is finite (5 × 2 × 3 = 30 elements per object), so every monotone fixpoint terminates.

Transfer functions (exact, per IR 2 node touching o or a view of o): Alloc → new o with (L0, false, NotObserved). PtrOffset, AggGet-field, ProvenanceCheck → view of o: the view's summary is o's summary, except PtrOffset's store-capability narrowing (Volume 13 §13.5) applies to provenance, not to the summary. AggSet(o.field ← v) → no change to o; v's summary updated by v's own transfer. Store(q ← p) where p is a view of o → o.level := max(o.level, L3). Return p (view of o) → o.level := max(o.level, L2). Call: actual p (view of o) bound to formal i of callee c with callee summary Σc(i) → o.level := max(o.level, Σc(i)); if c is unknown or c has the `ffi` atom → o.level := L4. ThreadSpawn/AsyncSpawn carrying p → o.thread := true. AddrOf(o) whose result is stored, compared (other than Vol 13 §13.5 PtrCmp identity use), volatile/atomic-accessed, or passed to unknown code → o.addr := max(o.addr, Forcing); AddrOf(o) used only for constant-path field access → o.addr := max(o.addr, Virtualizable); PtrCmp identity use → identity flag (below), no addr change. ForeignCall receiving p → o.level := L4. Drop(o) → no escape; obligation recorded. Suspend capturing p (live across the await) → o.level := max(o.level, L1) and o.thread := true (the async frame may resume elsewhere; Volume 08).

Callee summaries Σc: map from formal index to level, computed per function. Interprocedural algorithm: build the call graph; compute SCCs with `planck-graph-math` (§5.7); process SCCs in reverse topological order (callees first); inside an SCC iterate: initialize Σc(i) = L0; apply all callers' transfer functions; join; repeat until no Σc changes or ROUNDS = |SCC| + 2 rounds elapse; then widen every remaining changing Σc(i) to L4 (monotone, sound over-approximation; summaries may only increase during the fixpoint). Termination: each Σc(i) is one of 5 levels; monotone increase; bounded rounds. Complexity: O(Σ_c N_c × P_c) per round, N_c = nodes of c, P_c = pointer params; total O(rounds × Σ N_c P_c). Determinism: SCC order and join order are fixed by ascending function ID (G14).

The remaining §21.6 bullets:

| Bullet | Exact definition |
| --- | --- |
| Object lifetime analysis | The live range of o = from creation to its last use or surviving Drop; computed from use lists; lifetime shortening (§14.12) may shrink it under the Drop rules of Volume 13 §13.13. |
| Object identity analysis | IdentityObserved(o) = true iff Vol 13 §13.10's identity facts fire: o's address compared by PtrCmp/identity against another pointer into o, or stored where it may be so compared; identity-observed objects are never eliminated, never merged, never re-allocated. |
| Address observability analysis | Computes AO per §14.6 transfer functions; Forcing values are mandatory-physical (§14.1). |
| Virtual object construction | An object becomes virtual iff created by Alloc/AggNew and summary L0 and AO ≠ Forcing; virtualization removes the Alloc and its MemStates; construction is the Virtualize operation of §14.2. |
| Virtual object mutation | AggSet on virtual fields updates the field slot directly; no memory operation is emitted; mutation history must remain consistent with init facts (W5). |
| Virtual object merging, if legal | Two virtual objects may merge iff same type, disjoint live ranges, both L0, neither identity-observed nor pinned; merging joins field slots at joins with RepPhi per §14.2. |
| Object materialization | Materialize inserts storage per §14.1 rule and §14.3 materialization chain; materialization of a virtual object at a join materializes on every incoming path; Drop glue runs at the materialization point. |
| Object elimination | Delete object o iff: S(o) = (L0, false, NotObserved); identity not observed; AO ≠ Forcing or all Forcing uses are constant-path field accesses rewritten away; no UnknownEscapeEdge (§14.7 exact definition); drop glue absent or all Drops removed by the same proof; the Alloc, its failure edge (RF domain), and its MemStates are deleted together. |
| Object replacement | Replace a virtual object by its field values (scalar replacement judgment §14.5) or by a cheaper representation per §14.4; replacement must keep all identity/pinning/provenance facts. |
| Identity preservation | Every pass must preserve IdentityObserved facts: no transformation may introduce a second allocation for an identity-observed object, compare-rewrite, or coalesce two identity-distinct objects. W6 checks. |
| Address-stability handling | AddressStableObject (pinned): no move, no copy, no storage relocation; virtualization banned; the address in use must be the same allocation from creation to death (W6). |
| Pin-aware optimization | For pinned objects, passes must skip: allocation elimination, stack promotion, lifetime shortening across pin scope, field elimination, and any transform changing the address; the pin scope is the metadata region of Volume 13 §13.10. |

## 14.7 Allocation optimization (§21.7)

**Allocation elimination preconditions (exact, for deleting an Alloc a of size s, alignment al):** all six must hold: (A1) S(a) = (L0, false, NotObserved) per §14.6; (A2) no UnknownEscapeEdge exists, where UnknownEscapeEdge is exactly one of: use of a view of a as a Call/Invoke argument with callee summary ≥ L1 or Unknown; use at a ForeignCall; a Store/RawStore storing the view into any memory location; use of the view in a volatile or atomic operation; use of the view across a Suspend; a return of the view (level would be ≥ L2, so A1 already covers it; listed for completeness); (A3) IdentityObserved(a) = false; (A4) AO(a) ≠ Forcing, or every Forcing use is a constant-path field access rewritten by scalar replacement in the same pass; (A5) a's type has no drop glue, or every Drop of a is deleted by this pass under the side-effect-free proof obligation (drop glue with observable behavior bans elimination); (A6) the allocation failure edge is deleted with a; behavior is compared in the RF domain (pinned below). All eliminations are recorded as one transformation; partial elimination (deleting the Alloc but keeping a use) is banned.

**Resource-freedom (RF) domain (pinned):** defined semantics of a program is its behavior on executions that do not exhaust stack or heap; resource-exhaustion events (OOM, stack overflow) are outside the equivalence domain of G5. Allocation failure behavior is owned by Volume 18; IR 2 preserves the failure edge of every allocation that is not eliminated, and replaces it by the stack-limit event on stack promotion; both events are defined non-catchable terminations in edition P1, and the difference is confined to the RF domain. Volumes 05, 07, 18 must conform to this allocation-failure model; any divergence is non-conforming per SPEC-INDEX §0.5.

**Remaining §21.7 bullets (exact):**

| Bullet | Exact rule |
| --- | --- |
| Stack promotion | Replace heap Alloc by stack storage iff: s is a compile-time constant ≤ STACK_PROMO_MAX_BYTES = 16384; Σ promoted sizes in the function ≤ 32768 bytes; S(a) level ≤ L1 and thread=false (ArgEscape is acceptable: borrows end before frame death per Volume 06); AO ≠ Forcing or Forcing uses are fieldwise; a is not live across a Suspend; not pinned; no drop-glue obligation moves out of frame (drop must run in-frame). The OOM failure edge is replaced by the stack-limit event (RF domain). |
| Stack allocation | IR 2 emits stack objects for promoted and for by-value locals without heap allocation; stack object addresses are provenance-checked like any address; frame size per function ≤ STACK_FRAME_LIMIT = 8 MiB (limits table). |
| Arena allocation | Arena objects (allocator type from Volume 19) are recognized by their intrinsic summary: objects allocated in an arena may be bulk-deallocated at the arena's drop point; individual Dealloc of arena objects is removed iff the allocator summary declares arena semantics; otherwise Dealloc obligations are preserved exactly. |
| Allocation sinking | Move an Alloc from a dominating region to the arm(s) where the object is used, iff the allocation is pure (no aliasing write between the original site and uses) and the summary is unchanged; failure edge moves with it. |
| Allocation hoisting | Move an Alloc out of a loop to the preheader iff the allocation is loop-invariant (operands constant), S(a) allows one instance (no per-iteration identity requirement: IdentityObserved per-iteration objects ban hoisting), and drop timing moves with it (drop glue must not accumulate: only allowed if drop is side-effect-free or the object's lifetime is loop-independent). |
| Allocation coalescing | Merge n ≤ 4 same-size, same-align, same-partition allocations with disjoint live ranges into one allocation with offsets; banned if any is identity-observed or pinned; requires interval overlap check per SPEC-INDEX §5.8. |
| Reallocation elimination | Grow/realloc pattern (Alloc a2 after Dealloc of a1 with copy) replaced by in-place growth iff allocator summary declares in-place growth success semantics otherwise; if not provable, the pattern is preserved (never invent realloc semantics). |
| Allocation size specialization | Specialize a dynamic-size allocation to a constant size iff the size expression folds to a constant on every path reaching the Alloc; per-path constants ≤ 4 distinct values create ≤ 4 specialized sites, else keep dynamic. |
| Dead allocation elimination | Alloc whose object has no uses at all (not even Drop): delete Alloc, failure edge, and MemStates; this is allocation elimination with vacuous A2–A5. |
| Lifetime shortening | Per §14.12. |
| Allocator-specific optimization | Transformations beyond the above are banned in IR 2; allocator-specific behavior lives in Volume 18 and the allocator type's intrinsic summaries; IR 2 consumes summaries, never allocator internals. |
| Allocation failure preservation | For every allocation that survives, its failure edge must survive unmodified (same unwind target, same payload path); W8 checks. Eliminations and promotions follow the RF domain rule above. |

## 14.8 Type representation optimization (§21.8)

| Bullet | Exact rule |
| --- | --- |
| Niche optimization | niche(F) for payload field type F is computed exactly as: {null} if F is a reference/pointer/trait-object type; {0} if F is a NonZero integer type; the unused encodings if F is an enum encoded in d bits with k < 2^d variants; {false} or {true} if F is bool and one value is unreachable for F's constructors; ∅ otherwise. An enum uses niche encoding iff every variant has no payload or a payload of one common type F with niche(F) ≠ ∅; then tagless variants take niche values and the payload variant takes non-niche values. |
| Enum layout selection | Algorithms in fixed preference order: (1) niche-encoded (rule above); (2) untagged iff the variants' payload value spaces are provably disjoint — proof = pairwise range disjointness from range metadata or type disjointness (Volume 04), variants ≤ 8; (3) tagged with discriminant width d = min byte width ≥ ⌈log2 k⌉ from {1,2,4,8}; repr(C) fixes the C layout per Volume 05 and skips (1)–(2). |
| Tagged representation | Tag placed after the payload, payload first, tag aligned to d, total size padded to max(payload align, d); tag values 0..k−1. |
| Untagged representation where proven valid | Exactly rule (2) above; the disjointness proof is a per-instance obligation checked at layout decision time; unproven → tagged. |
| Integer representation | Integer values keep their exact width; bool is 1 byte with values {0,1} in memory; no width change is permitted (Volume 04/05 fidelity). |
| Pointer representation | 8 bytes on both initial targets; provenance metadata rides along (non-physical); thin pointers for sized types, fat for slices/trait objects. |
| Optional value representation | Option<T>: niche-encoded if niche(T) ≠ ∅; else tagged with 1-byte tag; Option<&T> is always niche (null); Option<NonZeroInt> always niche (0). |
| Boolean representation | 1 byte in memory restricted to {0,1}; as ImmScalar i1/i8 in registers; a bool loaded from memory with value ∉ {0,1} in safe code is unreachable by construction (frontend guarantees); in unsafe raw reads the value is used as-is per UB-1's absence — a raw byte is not a bool value until converted. |
| String and slice representations | &[T]: (ptr, len), 16 bytes; String: (ptr, len, cap), 24 bytes; str payloads are unsized and always behind pointers. |
| Trait object representation | (data ptr, vtable ptr), 16 bytes; vtable layout and slot order owned by Volume 16; IR 2 consumes slot IDs. |
| Closure representation | Per §14.10: empty captures → fn pointer; non-escaping → decomposed captures (no object); escaping → (fn ptr, env ptr). |
| Generic representation | Post-monomorphization, a generic type's representation is the representation of its substitution; no shared representation exists for an open generic. |
| Packed representations | repr(packed(N)), N ∈ {1,2,4,8}: align = 1, fields at accumulating offsets, no padding; accesses that would be misaligned compile to byte-wise loads or unaligned ops at cost +1; repr(packed) without N = packed(1). |

## 14.9 Generic specialization (§21.9)

| Bullet | Exact rule |
| --- | --- |
| Monomorphization | Per SPEC-INDEX §3: monomorphization only; every demanded instance is created before IR 1 of that instance; MONO_DEPTH_MAX = 32 (pinned); instance demand discovered by the demand-driven walker over the call graph. |
| Partial monomorphization | An item whose generic parameters are used only in effect rows, lifetimes, or never is emitted once, with erased metadata (see below) — no per-substitution instance. This is the only sharing mechanism. |
| Hybrid monomorphization | Per-parameter decision inside one item: type parameters are always monomorphized (layout depends on them); const parameters are specialized per §14.9 constant-specialization rule; lifetime and effect parameters are erased. A single item may thus have both concrete instances (per type) and shared code (per effect/lifetime substitution). |
| Erased polymorphism | The shared body form of partial monomorphization: dynamic metadata (vtable-like) is inserted only where the body uses the parameter's layout-dependent operations; if any such use exists, the item must be monomorphized instead (erasure is banned when layout matters). |
| Constant specialization | Fires at a call site iff the const argument is compile-time known AND (a) it enables folding a branch on that value in the callee, or (b) the estimated size reduction ≥ 10 cost units. Cap: ≤ 16 specializations per (item, call-site class); beyond that, share one instance. |
| Type specialization | Monomorphization of type parameters; identical to Monomorphization above; no separate mechanism exists. |
| Layout specialization | Types differing only in layout-relevant parameters (repr attributes, packed N) receive distinct instances; their layouts follow §14.8; the instances never share code. |
| Function specialization | A function instance is created per demanded (type substitution, const substitution, effect-row substitution) tuple; the tuple is the instance key. |
| Call-site specialization | Clone the callee for a call site when the clone lets a parameter become constant (devirtualization or constant specialization at that site); ≤ MAX_FN_CLONES = 8 clones per callee (from Volume 13 call graph optimization); clones are private symbols. |
| Specialization caching | Cache key = canonical (ItemId, ordered substitution); the cache is content-addressed by the canonical key; two compilations of the same key must yield byte-identical instances (determinism, G14); duplicate instantiation of one key in one compilation is a verifier error. |
| Specialization recursion | Depth ≤ MONO_DEPTH_MAX = 32, counted per instance chain; violation = compile-time error `MONO_DEPTH_EXCEEDED` (never a truncation or silent stop). |
| Code growth management | Budget per crate: Σ instance sizes (cost units, §14.16 node costs) ≤ 4 × Σ generic item sizes; when exceeded, stop creating new non-demanded-by-ABI instances in descending-profit order (largest profit first, ties by ascending ItemId) and emit warning `SPEC_GROWTH_EXCEEDED` with the instance count; demanded-by-ABI instances are always created. |
| Specialization profitability | profit(instance) = Σ call-site savings (per §14.16 runtime model) − instance size; instances with profit ≤ 0 are created only when demanded by ABI or by an exported symbol. |
| ABI interaction | Specialization must not change the ABI of any symbol in the export table: exported generic instantiations get explicit mangled names (Volume 12 mangling) and their C-ABI boundaries follow Volume 10; W9 checks at the IR 2 → MIR boundary. |

## 14.10 Closure optimization (§21.10)

| Bullet | Exact rule |
| --- | --- |
| Capture analysis | Capture set = free variables of the closure body. Mode per capture, in precedence order: ByValue(Move) if the body moves the variable in; ByRef(Mut) if the body mutates through it and the borrow checker permits a mutable borrow of that lifetime; ByValue(Copy) if the type is Copy and the body does not mutate through it; ByRef(Shared) otherwise (read-only borrow). The capture list is sorted by ascending variable ID (determinism). |
| Environment construction | Env layout = captures in capture-list order; built by AggNew of the capture tuple; the env's provenance and ownership metadata ride on the captures. |
| Environment decomposition | If the closure is non-escaping (S(env) level = L0) and every call is direct (callee known), the env object is removed and captures are passed as extra arguments at each call; decomposition applies when capture count ≤ 6; with > 6 captures the env object is kept. |
| Environment lifetime | The env must outlive every borrow it contains; borrow regions (Volume 13 §13.10) must nest the env's lifetime; a capture by reference may not outlive the env; W5 checks. |
| Capture ownership | ByValue(Move) captures transfer ownership into the env; the env's Drop runs capture drop glue in reverse capture-list order; ownership metadata transfers with the capture. |
| Capture borrowing | ByRef captures are AddrOf of the variable plus borrow-origin metadata; the borrow's region is the closure's use region. |
| Closure representation | Empty captures → FnValue (plain function pointer). Non-escaping, directly called → decomposed: the closure type erases to the function-with-captures signature (no object). Escaping → (fn ptr, env ptr) pair, 16 bytes. |
| Closure allocation elimination | The env is a VirtualObject candidate: scalar replacement (§14.5) or decomposition (above) eliminates its allocation under §14.7 preconditions; the closure's identity is not observable unless IdentityObserved, in which case elimination is banned. |
| Closure specialization | Closure call with constant callee (ClosureValue with known fn and known captures) rewrites to Direct(fn, captures-as-args) then follows the inlining pipeline of Volume 13 §13.12. |
| Closure conversion | The ClosureConversion pass runs before the IR 2 → MIR boundary and converts every closure to one of the three forms above; after it, no Closure-typed call may remain (W11). |
| Function pointer conversion | The empty-capture rule: closure with capture set ∅ converts to FnValue of the erased signature; the closure type then behaves exactly as the fn type (call ABI per Volume 10). |

## 14.11 Dynamic dispatch optimization (§21.11)

| Bullet | Exact rule |
| --- | --- |
| Devirtualization | Trait call → Direct iff the receiver's concrete type set is a singleton; sources of the set: constructor/allocation-site analysis, discriminant guard tests, handoff facts (Volume 11 §18.14); the rewrite must preserve the vtable-slot semantics of the call (Direct target = the vtable slot's function for that type). |
| Vtable constant propagation | If the vtable pointer is a known constant (materialized trait object of known type), each vtable slot load folds to its function value; slot → function mapping comes from Volume 16's vtable tables; IR 2 must not invent slot contents. |
| Trait object simplification | A trait object whose concrete type is proven singleton may replace the fat pointer by the thin data pointer plus Direct calls; the vtable field is dead after simplification and removed; banned if the object's identity includes the vtable (identity-observed). |
| Dispatch elimination | The combination: devirtualize + inline (Volume 13 §13.12); a dispatch is eliminated only when both rewrite; partial rewrites are fine, full elimination is the goal metric. |
| Interface specialization | Clone the calling function per receiver type, guarded by discriminant tests inserted at the clone site's callers; ≤ MAX_IFACE_CLONES = 4 clones per call site; beyond that, dispatch is preserved. |
| Dynamic dispatch preservation | If the receiver set has > 1 element or is unknown, the dynamic dispatch must remain verbatim; dropping a dispatch on a non-singleton set is a verifier error (W7). |
| Indirect call optimization | Indirect calls through fn pointers: rewrite to Direct iff the pointer is a compile-time-known constant; otherwise preserved; speculative/guarded indirect-call elimination is banned in edition P1 (no speculative execution modeling in the cost model). |
| Object safety constraints | Trait objects exist only for object-safe traits (Volume 04 rule); vtable slot indices are assigned by Volume 16; IR 2 consumes slot IDs from the trait table and must not renumber them; a trait call to a non-object-safe trait is a frontend rejection, never an IR 2 rewrite. |

## 14.12 Escape and lifetime optimization (§21.12)

| Bullet | Exact rule |
| --- | --- |
| Local escape analysis | The §14.6 algorithm run on one function with all callees summarized (Unknown summary → L4). |
| Interprocedural escape analysis | The §14.6 SCC fixpoint; summaries are over-approximations; the summary table is the only permitted cross-function escape input. |
| Return escape | L2: the object (or view) is returned, or stored into a location that is returned; permits stack promotion only if the return is a borrow that ends per Volume 06 (checked: return type is a reference with lifetime ≤ the promoted frame — else banned). |
| Heap escape | L3: stored into another heap/stack object; the stored pointer's lifetime is untracked locally. |
| Thread escape | thread flag: sent to another thread (spawn, channel, async spawn per Volume 08); thread-escaping objects are never stack-promoted and their allocations are never eliminated. |
| Foreign escape | L4: reaches FFI or unknown code; no elimination, no promotion, no representation change to storage. |
| Address escape | AO = Forcing: the address is observable; forces materialized storage; address escape with constant-path-only uses is Virtualizable (§14.6). |
| Lifetime shortening | Move a Drop earlier only if drop glue is side-effect-free (Volume 13 §13.13); otherwise Drop may only move later; live range = creation → last use or surviving Drop; shortening must not cross a pin scope or a borrow region. |
| Ownership transfer tracking | Move/MemMove metadata (Volume 13 §13.10) must be preserved through every pass: a value moved must not be dropped twice or used after move; W5 checks init/ownership consistency. |
| Reference analysis | Borrow regions must not extend: no transformation may lengthen the region in which a borrow is used, move a use outside its borrow region, or merge two borrow regions into one wider region. W6 checks. |

## 14.13 Vectorization and data layout (§21.13)

| Bullet | Exact rule |
| --- | --- |
| Automatic vectorization | A loop is vectorizable iff: it carries the `Vectorizable` fact of Volume 13 §13.12 (re-verified here); trip count is a compile-time constant or a runtime value with an emitted guard; all memory accesses use pairwise-disjoint partitions, or same-partition accesses are proven ≥ VF apart in index distance; body ops ∈ {arith, bit, cmp, load, store, select} plus reductions from {add, mul, and, or, xor, min, max} on integers only (float reductions are banned: reassociation would change IEEE semantics); branches inside are uniform (condition loop-invariant); no Suspend, no panic-raising op, no volatile/atomic in the body. |
| Explicit SIMD lowering | SIMD intrinsic calls (Volume 10 intrinsic table) lower to the target's vector ops per the Volume 16 mapping; the mapping is total over the intrinsic table for each enabled feature set. |
| Data layout specialization | Selection of SoA vs AoS per §14.13 rules below, applied to whole-storage layouts, recorded in the layout decision. |
| Structure-of-arrays transformation | Legal iff: all accesses to the struct array are fieldwise with constant field index; no address of a whole element escapes (AO rule); no identity observation of elements; the region is closed (transformation applies to the whole program region using the array). Result: per-field arrays with element-wise correspondence preserved. |
| Array-of-structures transformation | Legal iff: fields of the SoA form are always accessed together (co-access ratio ≥ 0.7 per §14.16 locality model) and the transform reduces estimated dynamic cost; inverse of SoA with the same escape conditions. |
| Loop vectorization | Emits: prologue guard (trip count ≥ VF or alignment guard), vector loop body with vector memory ops, epilogue scalar loop for the remainder (no speculative masked ops in edition P1); VF = min(target VR width / element width, 16), exact. |
| Vector memory operations | Use the aligned form iff the base alignment fact ≥ VF × element width, else the unaligned form at cost +1 per access; alignment facts come from §14.16's alignment analysis. |
| Alignment analysis | Computes per-pointer alignment facts: from Alloc alignment, static item alignment, stack promotion alignment, and divisibility of offsets; facts must be under-approximations (never claim more alignment than provable). |
| Alias restrictions | Vectorized accesses must not conflict: for equal partitions, require either disjoint objects or a proven index-distance ≥ VF between potentially conflicting accesses; unproven → no vectorization (conservative). |
| Vector legality | The opcode whitelist and the bans of the first bullet; additionally the loop must not contain RepConvert to a non-lane representation. |
| Target feature specialization | One instance per compile-time enabled feature set; runtime dispatch is banned in edition P1 (pinned); `#[target_feature]` instances (Volume 10) get separate symbols and guarded direct calls emitted by Volume 16's target setup; IR 2 creates the instances and records the feature set per instance. |

## 14.14 IR 2 optimization passes (§21.14)

The 21-pass catalog. Every pass uses the 10-row template of design.md §40; rows: Pre / Analyses / Algorithm / Preserves / Invalid-if / Complexity / Profit / Verifier / Tests / Proof. Passes run in the pipeline order listed; each emits a graph satisfying Volume 13 I1–I12 (adapted) and W1–W12 (§14.15); the verifier runs after every pass (G5). Node cost units as Volume 13 §13.12.

**P1 Representation selection.** Pre: verified IR 2 input graph; layout fixation state recorded. Analyses: §14.4 inputs (escape, alias, pressure, frequency). Algorithm: per SSA value compute candidate set, cost per §14.4, choose min, emit RepConvert/merge plan; then re-run per Region until homogeneous or MAX_RESELECTION_ROUNDS = 8. Preserves: S,E,M,O,P,D; identities; pinning. Invalid-if: candidate set empty for any value (type unsupported on target → compile-time rejection, not a pass failure). Complexity: O(n × candidates), candidates ≤ 16. Profit: §14.4 cost function; runs once, then on invalidation. Verifier: W1, W-welding. Tests: repr-selection corpus (each §14.3 rep exercised). Proof: REVIEW of cost-model monotonicity + differential corpus.

**P2 Scalar replacement.** Pre: §14.5 judgment premises hold. Analyses: §14.6 summary, AO, identity, partitions. Algorithm: per object o meeting the judgment, replace by k field values; rewrite accesses; delete Alloc, MemStates, failure edge; rewires Drop if drop glue absent. Preserves: S,E,P,D; O: ownership transitions of fields retained; M: partition of o removed with it. Invalid-if: any condition of the §14.5 judgment fails mid-pass. Complexity: O(size of o's subgraph). Profit: −Alloc(10) − field loads(3 each) + materialization risk 0 (judgment excludes it). Verifier: W1–W6. Tests: scalar-replacement corpus (escape variants ×). Proof: REVIEW per G4 dead-code row + differential.

**P3 Aggregate decomposition.** Pre: §14.5 decomposition judgment. Analyses: use shapes. Algorithm: split aggregate value into per-field values; insert RepConvert at boundaries; merge planning at joins per §14.2. Preserves: all. Invalid-if: a use consumes the whole aggregate in a non-reconstructing way. Complexity: O(n). Profit: enables P2/P5; fires when it removes ≥ 1 aggregate op. Verifier: W1–W3. Tests: decomposition corpus. Proof: REVIEW (fieldwise equivalence is definitional).

**P4 Aggregate reconstruction.** Pre: a decomposed value must be rebuilt. Analyses: none beyond uses. Algorithm: AggNew from decomposed fields at the materialization point; exact field order. Preserves: all. Invalid-if: field set incomplete. Complexity: O(k). Profit: required for correctness of materialization, not optional. Verifier: W1–W3. Tests: shared with P3. Proof: REVIEW.

**P5 Escape analysis.** Pre: verified graph; callee summary table available or crate-internal. Analyses: none required as input (this pass computes the summary table). Algorithm: §14.6 algorithm verbatim (local + SCC fixpoint + widening), rounds ≤ SCC size + 2. Preserves: nothing changes — analysis pass; outputs summaries. Invalid-if: not applicable (analysis). Complexity: O(rounds × Σ N_c P_c), rounds ≤ SCC size + 2. Profit: enables P2, P6, P7. Verifier: summary soundness spot-checks (W-checks on consumers). Tests: escape corpus (each lattice level exercised). Proof: monotonicity argument REVIEW + Z3 on the lattice join laws (OB-14-5).

**P6 Allocation elimination.** Pre: §14.7 preconditions A1–A6 per allocation. Analyses: §14.6 summaries, identity, AO, partitions. Algorithm: per qualifying Alloc: delete Alloc, MemStates, failure edge; rewrite uses per A4; delete Drops per A5; record the transformation as one unit. Preserves: S in RF domain, E (minus deleted alloc atom), P, D; O per A5. Invalid-if: any A1–A6 fails for that allocation (skip it, never partially eliminate). Complexity: O(n). Profit: −Alloc(10) − Dealloc(5) − failure edge. Verifier: W1–W6, W8. Tests: allocation-elimination corpus (each precondition violation exercised as negative test). Proof: REVIEW per G4 dead-code row; RF domain pinned in §14.7.

**P7 Stack promotion.** Pre: §14.7 stack-promotion rule (size ≤ 16384 const; frame budget; level ≤ L1; thread=false; no Suspend liveness; not pinned). Analyses: summaries; frame size accounting. Algorithm: replace heap Alloc by StackObject in the function frame; adjust AddrOf/Load/Store unchanged (same partition); failure edge → stack-limit event (RF domain). Preserves: S in RF domain; O (drop in frame); identity: promoted objects keep a stable intra-frame address. Invalid-if: any precondition fails. Complexity: O(n). Profit: −Alloc(10) − Dealloc(5) + stack bytes (charged to frame budget). Verifier: W1, W8, frame budget check. Tests: stack-promotion corpus. Proof: REVIEW + RF-domain argument (pinned §14.7).

**P8 Layout optimization.** Pre: graph verified; no fixation yet. Analyses: field access patterns, identity, pinning, repr attributes. Algorithm: per type, run §14.8 algorithms (niche, enum layout, packing decisions, SoA/AoS per §14.13); fix the layout decision; mark fixation. Preserves: all semantics; layout changes must respect repr attributes exactly (repr(C)/packed/transparent fixed per Volume 05). Invalid-if: a repr attribute constrains the layout (skip re-decision for that type). Complexity: O(types × fields). Profit: size reduction ≥ 8 bytes per object or access-cost reduction ≥ 2 units. Verifier: W4 (layout contracts). Tests: layout corpus per §14.8 rule. Proof: REVIEW against Volume 05 layout math.

**P9 Niche selection.** Pre: type has niche per §14.8. Analyses: niche computation. Algorithm: apply the §14.8 niche encoding; rewrite discriminant extract/construct to decode/encode niche bits. Preserves: S (semantic discriminant unchanged); D (spans). Invalid-if: niche(F) = ∅ or mixed payload types. Complexity: O(variants). Profit: −tag bytes, −1 discriminant op per construct. Verifier: W4. Tests: niche corpus (Option<&T>, Option<NonZero>, enum-with-spare-encodings). Proof: Z3 on the niche bijection (encoding/decoding round-trip, OB-14-6).

**P10 Boxing elimination.** Pre: heap object of box-like type (single-field allocator wrapper from Volume 19) with summary ≤ L1 and the §14.7 preconditions. Analyses: §14.6. Algorithm: remove the box indirection: replace Load(box.field) by the field value, replace the box object by the field (scalar replacement at box granularity); identity of the box is banned from being observable (frontend: Box identity is not a language guarantee; if IdentityObserved, skip). Preserves: S, O (drop glue of the field retained). Invalid-if: §14.7 preconditions fail. Complexity: O(n). Profit: −Alloc − deref(3). Verifier: W1–W6. Tests: boxing corpus. Proof: REVIEW per G4.

**P11 Unboxing.** Pre: inverse direction: a value wrapped to cross a generic/ABI boundary that no longer needs the wrapper after specialization. Analyses: specialization results (P14). Algorithm: remove the wrapper construct/extract pair when the wrapped value's representation is feasible at the boundary (RegisterAgg eligibility per Volume 10); ABI-eligible only. Preserves: S, ABI. Invalid-if: boundary requires the boxed form (ABI restriction input of §14.4). Complexity: O(n). Profit: −2 ops per unwrap. Verifier: W3, W9. Tests: unboxing corpus. Proof: REVIEW.

**P12 Closure conversion.** Pre: verified graph; capture analysis fresh. Analyses: §14.10 capture analysis, escape summary. Algorithm: per §14.10: decompose non-escaping; build (fn,env) pairs for escaping; FnValue for empty captures; rewrite calls per §14.10 specialization. Preserves: all. Invalid-if: capture mode conflicts with borrow regions (borrow checker error upstream; pass must not fire). Complexity: O(closures × captures). Profit: −env Alloc for decomposed closures. Verifier: W1–W7. Tests: closure corpus (each capture mode). Proof: REVIEW per G4 inlining-adjacent argument.

**P13 Closure specialization.** Pre: constant callee closure call sites. Analyses: constant analysis. Algorithm: per §14.10 specialization rule; then mark sites for inlining. Preserves: S,E,M,O,P,D. Invalid-if: callee unknown. Complexity: O(sites). Profit: indirect(10) → direct. Verifier: W7. Tests: closure corpus. Proof: REVIEW.

**P14 Generic specialization.** Pre: instance demand set computed. Analyses: call graph; demand walker. Algorithm: §14.9 rules verbatim: create instances per key, caching per canonical key, depth ≤ 32, growth budget management. Preserves: S (per SPEC-INDEX §3: semantics of a specialized instance equal the semantics of the generic — OB-04 family obligation); ABI per §14.9 ABI interaction. Invalid-if: depth > 32 → error MONO_DEPTH_EXCEEDED (compile-time rejection). Complexity: O(instances × instance size). Profit: §14.9 profitability rule. Verifier: W9, W11. Tests: specialization corpus (depth limits, growth budget, ABI cases). Proof: REVIEW + OB-04 family (per-instance equivalence differential).

**P15 Devirtualization.** Pre: receiver type set singleton (§14.11 sources). Analyses: class/constructor analysis, guard tests. Algorithm: rewrite Trait → Direct per §14.11; then vtable constant propagation per §14.11. Preserves: S,E,M,O,P,D. Invalid-if: set > 1 or unknown. Complexity: O(n). Profit: indirect(10) → direct(10) − dispatch overhead(4). Verifier: W7. Tests: devirt corpus. Proof: REVIEW.

**P16 Constant specialization.** Pre: §14.9 constant-specialization firing rule. Analyses: constant propagation results. Algorithm: create the ≤ 16 per-class instances; redirect call sites. Preserves: S (instance semantics = generic semantics). Invalid-if: cap exceeded (share one instance). Complexity: O(instances). Profit: per §14.9 rule (a) or (b). Verifier: W9. Tests: const-spec corpus. Proof: REVIEW + differential.

**P17 Object virtualization.** Pre: §14.5 aggregate virtualization judgment (pre-fixation, AO = Virtualizable). Analyses: AO, escapes. Algorithm: replace StackObject/HeapObject by VirtualObject; delete storage, MemStates; keep field values. Preserves: S in RF domain; O (drop obligations still resolvable). Invalid-if: any address use beyond constant-path field access. Complexity: O(object subgraph). Profit: −Alloc − loads. Verifier: W1–W6. Tests: virtualization corpus. Proof: REVIEW.

**P18 Object materialization.** Pre: §14.1 mandatory-physical rule fires for some object. Analyses: §14.4 outputs. Algorithm: insert Materialize per §14.3 materialization chain; drop virtual field slots; wire Drop glue. Preserves: all. Invalid-if: no chain entry feasible (compile-time rejection). Complexity: O(objects). Profit: correctness requirement, not optional. Verifier: W2, W4, W12. Tests: materialization corpus. Proof: REVIEW.

**P19 SIMD transformation.** Pre: §14.13 automatic-vectorization conditions. Analyses: alignment, aliasing, trip counts, feature sets. Algorithm: per §14.13 loop vectorization: guard, vector body, scalar epilogue; VF rule; emit fact per instance feature set. Preserves: S (lane semantics, memory behavior, ordering per G4 vectorization row); E (no new effects except the guard's memory reads); M (partitions unchanged). Invalid-if: any condition of §14.13 fails (skip loop, keep scalar). Complexity: O(body × VF). Profit: est. dynamic cost ÷ VF ≥ 2. Verifier: W1–W4, W8. Tests: vectorization corpus (each legality condition as negative test). Proof: REVIEW per G4 vectorization row + differential on integer kernels.

**P20 Loop transformation.** Pre: loop regions from Volume 13 §13.2; facts from §14.13. Analyses: trip counts, dependencies. Algorithm: unrolling (factor 4, or 8 when body cost ≤ 12 and trip constant multiple; full unroll only if trip is a compile-time constant ≤ 16), unswitching (≤ 4 clones), fusion (equal trips, non-conflicting partitions) — the same legality as Volume 13 §13.12 loop rows, applied to IR 2 graphs; bound MAX_UNROLL_BYTES = 4096 cost units. Preserves: S, iteration counts per Volume 13 §13.13 termination implications. Invalid-if: per-pass conditions. Complexity: O(body × factor). Profit: est. dynamic cost reduction ≥ 1 unit per iteration. Verifier: W1–W4. Tests: loop corpus. Proof: REVIEW per G4 loop unrolling row.

**P21 Interprocedural specialization.** Pre: crate call graph; summaries available. Analyses: call graph SCCs, summaries, constant demand. Algorithm: call-site cloning (§14.9 call-site specialization, ≤ 8 per callee) + parameter-constant propagation across the clone; SCC order per §5.7 math. Preserves: S, ABI (clones are private symbols). Invalid-if: clone would alter an exported symbol's ABI. Complexity: O(clones × size). Profit: per §14.16 runtime model, clone fires when savings ≥ 10 units. Verifier: W9, W11. Tests: interprocedural corpus. Proof: REVIEW + differential.

## 14.15 IR 2 verification (§21.15)

Verifier checks W1–W12 (each §21.15 bullet is one check), run after every pass and at the IR 2 → MIR boundary:

| Check | Property (exact) |
| --- | --- |
| W1 | Every representation is legal for its semantic type: each value's R supports all operations applied to it (§14.3 Supported set) and lies in the type's feasible set. |
| W2 | Representation conversions preserve values: every RepConvert(R1→R2) is in the §14.3 conversion table and its round-trip semantics per Volume 05 hold (no value change beyond the declared conversion semantics). |
| W3 | Ownership remains valid: ownership transitions, moves, and borrows of Volume 13 §13.10 are consistent; no use after move; no double drop. |
| W4 | Destruction remains correct: every surviving Drop obligation has exactly one Drop on every live path; drop order is reverse declaration order within aggregates; niche/tagged encodings decode to the semantic discriminant. |
| W5 | Object identity is preserved where observable: IdentityObserved facts hold; no second allocation, no coalescing, no compare-rewrite of identity-observed objects; address-stable objects keep one address. |
| W6 | Address observability requirements are respected: AO = Forcing values are materialized; borrow regions do not extend; pinned objects keep their address. |
| W7 | Dynamic dispatch preservation: every Trait call with non-singleton receiver set remains a dispatch; vtable slot IDs unchanged. |
| W8 | Memory dependencies remain correct: partitions, MemState SSA (Volume 13 §13.6 invariants), failure edges of surviving allocations, and volatile/atomic orderings are unchanged. |
| W9 | ABI requirements remain satisfied: call/return representations match Volume 10 per boundary; exported symbols unchanged per §14.9 ABI interaction. |
| W10 | Layout contracts are satisfied: every materialized access offset equals the current layout decision; repr(C)/packed/transparent layouts match Volume 05 exactly. |
| W11 | No unconverted construct remains: no Closure-typed call, no VirtualObject, no IR 1-only opcode at the boundary. |
| W12 | No illegal representation is passed to Machine IR: every value at the boundary is physical per §14.1; effect chain and dominance invariants of Volume 13 §13.11 still hold. |

Error enum `Ir2VerifyError`: IllegalRepr, IllegalOp, IllegalConvert, OwnershipViolation, DestructionViolation, IdentityViolation, AddressObsViolation, DispatchDropped, MemDepViolation, AbiViolation, LayoutViolation, UnconvertedConstruct, IllegalBoundaryRepr, LimitExceeded. Each variant names the check above that produces it.

## 14.16 Optimization profitability (§21.16)

The cost model is the single permitted profitability input (§14.4); the compiler must not assume a transformation is good merely because it removes an operation (binding design.md's principle here).

| Bullet | Exact model |
| --- | --- |
| Code growth | growth = Σ size(output graph) − Σ size(input graph) in cost units; per-function budget 256 cold / 1024 hot; crate-level specialization budget 4× per §14.9. |
| Compilation time | Per-function optimization budget COMPILE_BUDGET_OPS = 10^8 abstract operations (limits table); each pass charges its Complexity column in ops; exceeding the budget stops further passes in pipeline order (deterministic), emits warning `OPT_BUDGET_EXCEEDED`. |
| Runtime performance | est. dynamic cost = Σ freq(node) × cost(node); freq from PGO when available, else the static model: loop body multiplied by trip estimate 8, branch cost = 2 + 15 × (1 − p)/100 with p the branch probability in percent (default 50). |
| Register pressure | RP = max simultaneous live values per Region; a transformation may not push RP above the target's allocatable register count R_GPR (Volume 15 §15.2: 14 on x86-64 SysV, 28 on AArch64) without spilling; spilling cost 3 per spilled live range is charged to the transformation. |
| Instruction-cache pressure | ICP = bytes of hot code (loops × body bytes, 4 bytes/instruction estimate); a transformation may increase hot code by at most max(25%, 512 bytes) unless PGO shows a win; exceeding that bans the transformation. |
| Allocation reduction | Metric: count of Alloc nodes removed; reported, never used alone to justify a transformation (the removal rule of §14.7 is the gate). |
| Memory footprint | Σ object sizes; stack promotion moves bytes from heap to frame (frame budget 8 MiB, §14.7); heap delta reported. |
| Cache locality | locality score = Σ access freq × 1/(1 + reuse distance); used by SoA/AoS (ratio ≥ 0.7 fires SoA, §14.13). |
| Branch behavior | Transforms may not add branches with unknown probability on hot paths unless removing ≥ 2; expected branch cost per the runtime model above. |
| Specialization count | ≤ 65536 instances per crate (limits table), depth ≤ 32; growth budget per §14.9. |
| Inlining | Thresholds pinned in Volume 13 §13.12 (INLINE_BUDGET = 256 cold / hot multiplier 4; MAX_INLINE_DEPTH = 5; callee size cap 4096 units); IR 2 uses the same table. |
| Loop transformations | Unroll factor 4 (8 under the §14.14 P20 condition); full unroll ≤ 16 iterations; unswitch ≤ 4 clones; fusion requires equal trip counts; each fires only when est. dynamic cost decreases (≥ 1 unit per iteration). |

Pipeline composition (G5): the full pass list P1–P21 runs with W1–W12 verification after every pass; a failing pass aborts compilation with the failing obligation's counterexample (G17); no unverified pass enters the pipeline.

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-14-1 | Representation closure: every value edge's R ∈ the 16-member Repr set; ops applied are in R's Supported set | EXH over Repr × opcode table | FV-0 | `tools/fv/ir2_repr.rs` | G14, G15 |
| OB-14-2 | Conversion legality: every RepConvert is in the §14.3 conversion table and value-preserving per Volume 05 | Z3 over the conversion table (bounded value domains per rep) + EXH table check | FV-0 | `tools/fv/ir2_convert.smt2` | G4, G5 |
| OB-14-3 | Escape lattice laws: join is associative, commutative, idempotent, identity L0; order is total on levels | Z3/EXH over the 5 × 2 × 3 = 30-element summary lattice (pairwise, 30² = 900 pairs) | FV-0 | `tools/fv/ir2_escape_lattice.smt2` | G4 |
| OB-14-4 | Escape analysis soundness: computed level ≥ true escape level on the differential corpus; summaries monotone under widening | PROP (differential, deterministic PRNG) + REVIEW of transfer functions | FV-1 (gap: true-escape oracle is corpus-based; bridge: conservative widening to L4 on uncertainty) | `tools/fv/ir2_escape_diff.rs` | G4, G5 |
| OB-14-5 | Escape fixpoint termination: SCC fixpoint terminates in ≤ (SCC size + 2) rounds; lattice finite | EXH over generated call graphs (n ≤ 8 functions) recording rounds + REVIEW of monotonicity | FV-0 | `tools/fv/ir2_escape_term.rs` | G11 |
| OB-14-6 | Niche encoding bijection: encode(discriminant, payload) then decode yields the semantic discriminant and payload exactly | Z3 over the niche rules (bounded discriminant domains k ≤ 256) | FV-0 | `tools/fv/ir2_niche.smt2` | G4 |
| OB-14-7 | Allocation elimination preconditions: elimination fires iff A1–A6 all hold; negative tests for each violated precondition | EXH over precondition-combination corpus (2^6 combinations × type classes) | FV-0 | `tools/fv/ir2_allocelim.rs` | G4, G5 |
| OB-14-8 | Stack promotion bounds: promoted size ≤ 16384, frame ≤ 32768 promoted bytes per function, level ≤ L1, thread=false | EXH over bound-boundary cases + Z3 bound arithmetic | FV-0 | `tools/fv/ir2_stackpromo.smt2` | G13, G4 |
| OB-14-9 | Specialization limits: depth ≤ 32 enforced with error MONO_DEPTH_EXCEEDED; instances ≤ 65536; growth ≤ 4× | EXH over depth/instance boundary cases + Z3 budget arithmetic | FV-0 | `tools/fv/ir2_specialize.smt2` | G13, G14 |
| OB-14-10 | Identity preservation: no pass creates a second allocation or merges identity-observed objects (W5 fires on generated cases) | PROP over generated object graphs paired with EXH negative corpus | FV-1 (gap: generated-graph coverage bounded at n ≤ 20; bridge: W5 runs on every real graph) | `tools/fv/ir2_identity.rs` | G4, G5 |
| OB-14-11 | Vectorization legality: each §14.13 condition enforced; vector output preserves lane semantics and memory ordering on integer kernels | PROP (differential on generated integer loops) + EXH negative corpus per condition | FV-1 (gap: float kernels excluded by rule, not by proof; bridge: float reductions banned by §14.13) | `tools/fv/ir2_vector_diff.rs` | G4, G5 |
| OB-14-12 | IR 2 verifier completeness: each W1–W12 implemented, exercised, mapped to its §21.15 bullet | REVIEW with checklist | FV-2 | verifier source review | G15 |
| OB-14-13 | Boundary soundness: every graph at the IR 2 → MIR boundary passes W11, W12 (all values physical, no residual constructs) | EXH over boundary corpus + REVIEW | FV-1 (gap: corpus-bounded; bridge: W12 is a hard gate in the pipeline) | `tools/fv/ir2_boundary.rs` | G4, G15 |
| OB-14-14 | Pass pipeline equivalence: every pass P1–P21 preserves semantics per G5 on the differential corpus; verifier runs after each pass | PROP (differential) paired with Z3 obligations OB-14-2/6/8/9 | FV-1 (gap: sampling; bridge: W1–W12 after every pass in production) | `tools/fv/ir2_pass_diff.rs` | G4, G5 |
| OB-14-15 | Cost model determinism: same input graph + target → identical selection, layout, and pass decisions (G14) | PROP (byte-hash of decisions over corpus) + REVIEW of tie-break rules | FV-1 (gap: property sampling; bridge: all tie-breaks pinned in §14.4/§14.8/§14.9) | `tools/fv/ir2_determinism.rs` | G14 |
| OB-14-16 | Coverage: every §21.14 pass has the 10-row template filled and a regression class; every §21.16 model has a pinned value | REVIEW against §14.14, §14.16 | FV-2 | coverage ledger below | G15 |

## Coverage ledger

| design.md bullet (line, abbreviated) | Defined in |
| --- | --- |
| l2173 Semantic information inherited from IR 1 | §14.1 |
| l2174 Representation information introduced in IR 2 | §14.1 |
| l2175 Target-independent versus target-aware decisions | §14.1 |
| l2176 Permitted representation transformations | §14.1 |
| l2177 Semantic preservation requirements | §14.1 |
| l2178 When layout decisions become fixed | §14.1 |
| l2179 When physical storage becomes mandatory | §14.1 |
| l2180 Which invariants must survive lowering | §14.1 |
| l2184 Multiple value representations | §14.2 |
| l2185 Multiple aggregate representations | §14.2 |
| l2186 Different memory representations | §14.2 |
| l2187 Virtual objects | §14.2 |
| l2188 Materialized objects | §14.2 |
| l2189 Scalar values | §14.2 |
| l2190 Vector values | §14.2 |
| l2191 Address-bearing values | §14.2 |
| l2192 Representation conversions | §14.2 |
| l2193 Representation-specific operations | §14.2 |
| l2194 Representation compatibility | §14.2 |
| l2195 Representation merging at control-flow joins | §14.2 |
| l2201 Immediate scalar | §14.3 |
| l2202 Multi-scalar aggregate | §14.3 |
| l2203 Packed scalar | §14.3 |
| l2204 Register aggregate | §14.3 |
| l2205 Vector | §14.3 |
| l2206 Tagged value | §14.3 |
| l2207 Niche-encoded value | §14.3 |
| l2208 Pointer representation | §14.3 |
| l2209 Slice representation | §14.3 |
| l2210 Trait-object representation | §14.3 |
| l2211 Closure environment | §14.3 |
| l2212 Virtual object | §14.3 |
| l2213 Stack object | §14.3 |
| l2214 Heap object | §14.3 |
| l2215 Address-stable object | §14.3 |
| l2216 Foreign-layout object | §14.3 |
| l2220 Supported operations (per rep) | §14.3 col. Supported operations |
| l2221 Size (per rep) | §14.3 col. Size |
| l2222 Alignment (per rep) | §14.3 col. Align |
| l2223 Field access (per rep) | §14.3 col. Field access |
| l2224 Copy behavior (per rep) | §14.3 col. Copy |
| l2225 Move behavior (per rep) | §14.3 col. Move |
| l2226 Destruction behavior (per rep) | §14.3 col. Destruction |
| l2227 Conversion rules (per rep) | §14.3 col. Conversion rules |
| l2228 Materialization requirements (per rep) | §14.3 col. Materialization |
| l2236 Semantic type (sel. input) | §14.4 |
| l2237 Target architecture (sel. input) | §14.4 |
| l2238 Usage patterns (sel. input) | §14.4 |
| l2239 Escape information (sel. input) | §14.4 |
| l2240 Address observability (sel. input) | §14.4 |
| l2241 Object identity (sel. input) | §14.4 |
| l2242 Layout requirements (sel. input) | §14.4 |
| l2243 Register pressure estimates (sel. input) | §14.4 |
| l2244 Allocation behavior (sel. input) | §14.4 |
| l2245 ABI restrictions (sel. input) | §14.4 |
| l2246 Alignment (sel. input) | §14.4 |
| l2247 Access frequency (sel. input) | §14.4 |
| l2251 Selected representation (sel. output) | §14.4 |
| l2252 Materialization requirements (sel. output) | §14.4 |
| l2253 Conversion plan (sel. output) | §14.4 |
| l2254 Layout decision (sel. output) | §14.4 |
| l2255 Specialization decision (sel. output) | §14.4 |
| l2257 Selection cost models and revision | §14.4 |
| l2261 Scalar replacement | §14.5 |
| l2262 Aggregate decomposition | §14.5 |
| l2263 Field promotion | §14.5 |
| l2264 Field elimination | §14.5 |
| l2265 Field splitting | §14.5 |
| l2266 Aggregate reconstruction | §14.5 |
| l2267 Aggregate propagation | §14.5 |
| l2268 Aggregate merging, when legal | §14.5 |
| l2269 Aggregate materialization | §14.5 |
| l2270 Aggregate virtualization | §14.5 |
| l2271 Aggregate copy elimination | §14.5 |
| l2272 Partial aggregate optimization | §14.5 |
| l2276 Escape analysis | §14.6 |
| l2277 Object lifetime analysis | §14.6 |
| l2278 Object identity analysis | §14.6 |
| l2279 Address observability analysis | §14.6 |
| l2280 Virtual object construction | §14.6 |
| l2281 Virtual object mutation | §14.6 |
| l2282 Virtual object merging, if legal | §14.6 |
| l2283 Object materialization | §14.6 |
| l2284 Object elimination | §14.6 |
| l2285 Object replacement | §14.6 |
| l2286 Identity preservation | §14.6 |
| l2287 Address-stability handling | §14.6 |
| l2288 Pin-aware optimization | §14.6 |
| l2292 Allocation elimination | §14.7 |
| l2293 Stack promotion | §14.7 |
| l2294 Stack allocation | §14.7 |
| l2295 Arena allocation | §14.7 |
| l2296 Allocation sinking | §14.7 |
| l2297 Allocation hoisting | §14.7 |
| l2298 Allocation coalescing | §14.7 |
| l2299 Reallocation elimination | §14.7 |
| l2300 Allocation size specialization | §14.7 |
| l2301 Dead allocation elimination | §14.7 |
| l2302 Lifetime shortening | §14.12 |
| l2303 Allocator-specific optimization | §14.7 |
| l2304 Allocation failure preservation | §14.7 |
| l2308 Niche optimization | §14.8 |
| l2309 Enum layout selection | §14.8 |
| l2310 Tagged representation | §14.8 |
| l2311 Untagged representation where proven valid | §14.8 |
| l2312 Integer representation | §14.8 |
| l2313 Pointer representation | §14.8 |
| l2314 Optional value representation | §14.8 |
| l2315 Boolean representation | §14.8 |
| l2316 String and slice representations | §14.8 |
| l2317 Trait object representation | §14.8 |
| l2318 Closure representation | §14.8 |
| l2319 Generic representation | §14.8 |
| l2320 Packed representations | §14.8 |
| l2324 Monomorphization | §14.9 |
| l2325 Partial monomorphization | §14.9 |
| l2326 Hybrid monomorphization | §14.9 |
| l2327 Erased polymorphism | §14.9 |
| l2328 Constant specialization | §14.9 |
| l2329 Type specialization | §14.9 |
| l2330 Layout specialization | §14.9 |
| l2331 Function specialization | §14.9 |
| l2332 Call-site specialization | §14.9 |
| l2333 Specialization caching | §14.9 |
| l2334 Specialization recursion | §14.9 |
| l2335 Code growth management | §14.9 |
| l2336 Specialization profitability | §14.9 |
| l2337 ABI interaction | §14.9 |
| l2341 Capture analysis | §14.10 |
| l2342 Environment construction | §14.10 |
| l2343 Environment decomposition | §14.10 |
| l2344 Environment lifetime | §14.10 |
| l2345 Capture ownership | §14.10 |
| l2346 Capture borrowing | §14.10 |
| l2347 Closure representation | §14.10 |
| l2348 Closure allocation elimination | §14.10 |
| l2349 Closure specialization | §14.10 |
| l2350 Closure conversion | §14.10 |
| l2351 Function pointer conversion | §14.10 |
| l2355 Devirtualization | §14.11 |
| l2356 Vtable constant propagation | §14.11 |
| l2357 Trait object simplification | §14.11 |
| l2358 Dispatch elimination | §14.11 |
| l2359 Interface specialization | §14.11 |
| l2360 Dynamic dispatch preservation | §14.11 |
| l2361 Indirect call optimization | §14.11 |
| l2362 Object safety constraints | §14.11 |
| l2366 Local escape analysis | §14.12 |
| l2367 Interprocedural escape analysis | §14.12 |
| l2368 Return escape | §14.12 |
| l2369 Heap escape | §14.12 |
| l2370 Thread escape | §14.12 |
| l2371 Foreign escape | §14.12 |
| l2372 Address escape | §14.12 |
| l2373 Lifetime shortening | §14.12 |
| l2374 Ownership transfer tracking | §14.12 |
| l2375 Reference analysis | §14.12 |
| l2379 Automatic vectorization | §14.13 |
| l2380 Explicit SIMD lowering | §14.13 |
| l2381 Data layout specialization | §14.13 |
| l2382 Structure-of-arrays transformation | §14.13 |
| l2383 Array-of-structures transformation | §14.13 |
| l2384 Loop vectorization | §14.13 |
| l2385 Vector memory operations | §14.13 |
| l2386 Alignment analysis | §14.13 |
| l2387 Alias restrictions | §14.13 |
| l2388 Vector legality | §14.13 |
| l2389 Target feature specialization | §14.13 |
| l2395 Representation selection (pass) | §14.14 P1 |
| l2396 Scalar replacement (pass) | §14.14 P2 |
| l2397 Aggregate decomposition (pass) | §14.14 P3 |
| l2398 Aggregate reconstruction (pass) | §14.14 P4 |
| l2399 Escape analysis (pass) | §14.14 P5 |
| l2400 Allocation elimination (pass) | §14.14 P6 |
| l2401 Stack promotion (pass) | §14.14 P7 |
| l2402 Layout optimization (pass) | §14.14 P8 |
| l2403 Niche selection (pass) | §14.14 P9 |
| l2404 Boxing elimination (pass) | §14.14 P10 |
| l2405 Unboxing (pass) | §14.14 P11 |
| l2406 Closure conversion (pass) | §14.14 P12 |
| l2407 Closure specialization (pass) | §14.14 P13 |
| l2408 Generic specialization (pass) | §14.14 P14 |
| l2409 Devirtualization (pass) | §14.14 P15 |
| l2410 Constant specialization (pass) | §14.14 P16 |
| l2411 Object virtualization (pass) | §14.14 P17 |
| l2412 Object materialization (pass) | §14.14 P18 |
| l2413 SIMD transformation (pass) | §14.14 P19 |
| l2414 Loop transformation (pass) | §14.14 P20 |
| l2415 Interprocedural specialization (pass) | §14.14 P21 |
| l2423 Every representation is legal for its semantic type | §14.15 W1 |
| l2424 Representation conversions preserve values | §14.15 W2 |
| l2425 Ownership remains valid | §14.15 W3 |
| l2426 Destruction remains correct | §14.15 W4 |
| l2427 Object identity is preserved where observable | §14.15 W5 |
| l2428 Address observability requirements are respected | §14.15 W6 |
| l2429 Pinning restrictions are preserved | §14.15 W6 |
| l2430 Layout contracts are satisfied | §14.15 W10 |
| l2431 Memory dependencies remain correct | §14.15 W8 |
| l2432 Effect dependencies remain correct | §14.15 W8, W12 |
| l2433 ABI requirements remain satisfied | §14.15 W9 |
| l2434 No illegal representation is passed to Machine IR | §14.15 W12 |
| l2440 Code growth (cost model) | §14.16 |
| l2441 Compilation time (cost model) | §14.16 |
| l2442 Runtime performance (cost model) | §14.16 |
| l2443 Register pressure (cost model) | §14.16 |
| l2444 Instruction-cache pressure (cost model) | §14.16 |
| l2445 Allocation reduction (cost model) | §14.16 |
| l2446 Memory footprint (cost model) | §14.16 |
| l2447 Cache locality (cost model) | §14.16 |
| l2448 Branch behavior (cost model) | §14.16 |
| l2449 Specialization count (cost model) | §14.16 |
| l2450 Inlining (cost model) | §14.16 |
| l2451 Loop transformations (cost model) | §14.16 |
| l2453 No transformation is good merely for removing an operation | §14.16 |
| l2455 Inlining example (i-cache pressure) | §14.16 |

Bullets in scope: 212, defined: 212; prose rows: 3 (l2257, l2453, l2455).
