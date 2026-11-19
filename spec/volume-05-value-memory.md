# Planck — Volume 05: Value, Memory, and Object Semantics

CEP:FILE: spec/volume-05-value-memory.md
CEP:WHY: design.md §7 (lines 525–630) and §8 (lines 631–764) require the Value Semantics Specification, Object and Layout Specification, Memory Model Specification, and UB Catalog (line 761); without them neither the type system (Volume 04) nor the ownership checker (Volume 06) nor any backend pass has a defined notion of value, object, allocation, or undefined behavior.
CEP:WHAT: Complete normative value/object semantics, initialization dataflow, move/copy/drop semantics, exact layout arithmetic, identity and optimization legality, memory regions, pointer provenance, memory access, aliasing, allocation, the UB catalog (UB-1..UB-6 plus defined traps, exit code 134), and memory ordering, with obligations OB-05-*.
CEP:STATUS: complete
CEP:FAILURE: A semantic rule here that admits reading uninitialized typed storage, invalid references, data races, wrong-ABI calls, asm-contract violations, or intrinsic-precondition violations outside the exact UB-1..UB-6 predicates is a Severity 0 soundness defect; an obligation row that cannot be discharged blocks release.
CEP:ASSUMES: SPEC-INDEX §1 (method), §3 (settled decisions: UB-1..UB-6, provenance = allocation ID + offset + capability set {load, store, compute}, identity = address + allocation ID, repr(P) default layout, destruction innermost-first, orphan rule, effect atoms), §5.3 (effect algebra), §5.4 (ownership state machine), standards.md §3 (normative verbs), verification standards.md G1–G26, Volume 02 (literals, operators), Volume 03 (places, DefId), Volume 04 (types, layout reflection), Volume 06 (borrow checker, regions, pinning), Volume 07 (panic/unwinding execution), Volume 08 (threads, atomics model), Volume 10 (intrinsics, FFI/ABI), Volume 12 (limits table), Volume 13–15 (IR mappings), Volume 16 (target memory models), Volume 17 (symbols), Volume 18 (runtime, allocator, fault handler).
CEP:COST: compile-time analysis costs stated per algorithm (init dataflow O(|places| × |CFG edges|)); runtime cost is zero in the abstract machine; concrete lowering may insert trap checks only where this volume defines them, elidable exactly when the compiler proves the check cannot fail.
CEP:EVIDENCE: verification matrix (## Verification matrix), obligations OB-05-1..OB-05-18; coverage ledger (## Coverage ledger): one row per design.md §7 and §8 bullet.

---

## 5.0 Scope and traceability

This volume is the definition home for design.md **§7 "Value semantics and object identity" (lines 525–630)** and **§8 "Memory model" (lines 631–764)**, including the deliverables at line 761 (Value Semantics Specification, Object and Layout Specification, Memory Model Specification, UB Catalog). Volume numbering: Volumes V–VI of design.md's prose map to library volumes 05 and 06; design §7 maps to §5.1–§5.6 here, design §8 maps to §5.7–§5.13 here.

| design.md subsection | design.md lines | This volume | Bullets |
| -------------------- | --------------- | ----------- | ------- |
| 7.1 Values and objects | 531–545 | §5.1 | 13 |
| 7.2 Initialization | 547–561 | §5.2 | 13 |
| 7.3 Copy and move | 563–577 | §5.3 | 13 |
| 7.4 Destruction | 579–595 | §5.4 | 15 |
| 7.5 Layout | 597–614 | §5.5 | 16 |
| 7.6 Identity and optimization | 616–629 | §5.6 | 12 |
| 8.1 Memory regions | 635–645 | §5.7 | 9 |
| 8.2 Pointers | 647–667 | §5.8 | 19 |
| 8.3 Memory access | 669–684 | §5.9 | 14 |
| 8.4 Aliasing model | 686–697 | §5.10 | 10 |
| 8.5 Allocation | 699–713 | §5.11 | 13 |
| 8.6 Undefined behavior | 715–747 | §5.12 | 18 |
| 8.7 Memory ordering | 748–759 | §5.13 | 10 |

bullets in scope: 175, defined: 175 (ledger in ## Coverage ledger).

The 16-row feature template of design.md §40 is applied to move semantics (§5.14.1) and slice indexing (§5.14.2). Settled constants defined here: TRAP_EXIT_CODE = 134; MAX_ALIGNMENT_BYTES = 536870912 (2^29); MAX_ALLOC_SIZE = 281474976710656 (2^48); STACK_LIMIT_MAIN = 8388608; STACK_LIMIT_SPAWNED = 2097152; DROP_FLAG_BITS_LIMIT = 65536; EXPOSE_TABLE_LIMIT = 1048576 (2^20); MAX_ATOMIC_WIDTH_BYTES = 16; PACKED_ALIGN_LIMIT = 64.

### 5.0.1 The abstract memory machine

Semantics are defined over an abstract machine state M = (Allocations, Provenances, Threads, Effect budget). An allocation is a record { id: u64 ≥ 1, base: address, size: u64, align: u64, kind: {Stack, Heap, Static, ThreadLocal, Foreign, Mmio}, live: bool }. Allocation IDs are assigned monotonically per execution (deterministic per execution path, G14); ID 0 is the null provenance; ID 1 is reserved for zero-sized allocation returns (§5.11.8); ID 2 with offset = address is the foreign class (§5.11.11). A pointer value is (address: u64, provenance: Provenance) where Provenance = { alloc_id: u64, offset: u64, caps: 2^{load, store, compute} } per SPEC-INDEX §3. Object identity is (alloc_id, offset) — pinned: two objects are the same object iff same allocation ID and same offset; distinct allocations never compare equal by identity.

### 5.0.2 Traps

Traps are defined terminations, never UB. Every trap terminates the process by abort with TRAP_EXIT_CODE = 134 (diagnostic line printed to stderr, ordering per Volume 18 §25).

T05-1 (complete trap taxonomy):

| Trap | Trigger (exact) | Raised by |
| ---- | --------------- | --------- |
| AddressTrap | dereference with provenance alloc_id = 0 (null) or address outside the target's mapped space | §5.8.7, §5.9.10 |
| AlignmentTrap | typed raw access at address p with p mod align(T) ≠ 0; atomic op with width w and p mod w ≠ 0 | §5.9.11, §5.13.4 |
| BoundsTrap | slice/array index i with i ≥ len or (element stride overflows the provenance range) | §5.9.2, §5.14.2 |
| ProvenanceTrap | raw dereference of a dead allocation; access with missing capability; ptr_offset_from across different allocations | §5.8.9, §5.8.17, §5.11.6 |
| CallTrap | indirect call through null fn pointer or through an address that is not a live function entry of the expected ABI | §5.8.16, §5.9.10 |
| ReprTrap | typed read of bool/char/enum storage violating its representation invariant (Volume 04 §4.1.4, §4.1.5, §4.1.12) | §5.9.1 |
| StackTrap | stack usage exceeds STACK_LIMIT_MAIN (main thread) or STACK_LIMIT_SPAWNED (spawned threads) | §5.7.1 |
| MemoryFaultTrap | fault in Foreign or Mmio storage regions | §5.7.5, §5.7.6 |

A compiler must emit a trap check at every point this table raises a trap unless it proves statically that the trigger condition cannot hold (proof obligations OB-05-6, OB-05-8). An elided check whose trigger fires is a Severity 0 defect.

---

## 5.1 Values and objects

### 5.1.1 Definition of value

A value of type T is a pair v = (b, P) where b ∈ {0..255}^{size(T)} is the byte pattern and P is the provenance context mapping each pointer-typed byte range of T to a Provenance record (§5.0.1). v is a valid value of T iff valid(T, b, P) holds, defined inductively: integers: all patterns valid (two's complement); f32/f64: all patterns valid; bool: b ∈ {0x00, 0x01}; char: b is a little-endian scalar in [0x000000, 0xD7FF] ∪ [0xE000, 0x10FFFF]; unions: all patterns valid; enums: tag pattern equals some declared discriminant; references and pointers: P valid per §5.10.1 at the moment of use (validity is checked at use, not at formation); aggregates: every field's bytes valid; ! : no valid values. A typed read of invalid storage raises ReprTrap for bool/char/enum (T05-1) and UB-2 for references (§5.12).

### 5.1.2 Definition of object

An object is a triple (allocation A, offset o, type T) with size(T) ≤ A.size − o, existing from its creation event (§5.1.10) to its destruction event (§5.1.11). An object occupies the byte range [A.base + o, A.base + o + size(T)).

### 5.1.3 Semantic identity

Pinned (SPEC-INDEX §3): identity is address + allocation ID. Two expressions denote the same object iff they denote the same (alloc_id, offset) pair. Compilation must not change the answer of any identity observation (obligation OB-05-15).

### 5.1.4 Object identity

Object identity is the pair (alloc_id, offset) plus the object's type interpretation T: the same storage pair with a different type is a different object (created by a re-interpretation event, §5.6.5). Pointer values referring to the same (alloc_id, offset) refer to the same object.

### 5.1.5 Storage identity

Storage identity is the pair (alloc_id, byte range [o, o + size)). Two storages are the same storage iff same alloc_id and identical ranges. Storage is reused after death (§5.6.5); reuse creates a new allocation identity only when a new allocation is created; frame-slot reuse within one allocation keeps alloc_id but must obey §5.6.5 rules.

### 5.1.6 Value equivalence

v1 ≡ v2 (both of type T) iff their byte patterns are equal and for every pointer field, their provenances are equal as records (alloc_id, offset, caps). Value equivalence ignores object identity of non-pointer content. This is the equivalence used by the compiler for commoning constants and by move/copy semantics.

### 5.1.7 Equality semantics

The == operator is defined exactly for: integer types (mathematical comparison in T04-2 range); f32/f64 (IEEE-754: NaN ≠ anything, +0 = −0); bool; char (scalar equality); () (always true); references and raw pointers (§5.8.10 provenance-pair equality, not address equality); fn pointers (address equality with provenance-alloc equality); SIMD (lane-wise per element type). For all other types == is the PartialEq trait (Volume 19); the compiler must not assume == is reflexive, symmetric, or consistent with ≡ for trait-implemented types (no exploitable equality assumption, per §5.6.12).

### 5.1.8 Address observability

Safe code observes an address only through: (1) forming a reference or raw pointer to it; (2) comparing or ordering pointers derived from it (§5.8.10); (3) unsafe ptr-to-integer exposure (§5.8.11); (4) passing pointers across FFI. The compiler must not introduce address observations that a source program cannot make; the runtime must not reorder allocations in ways that change observable addresses after they are observed.

### 5.1.9 Address stability

The address of an object is constant from its creation to its destruction. A move does not destabilize: the source object is destroyed and a new object is created at the destination (old pointers into the source become invalid per §5.5.3 of §5.3.10, not "moved addresses"). Heap realloc (§5.11.7) destroys the old object and creates a new one. For !Unpin types pinned through Pin, no event may relocate the object until its destructor completes (Volume 06 §9.9); violation is a compiler/runtime contract breach, prevented by the Pin API typing.

### 5.1.10 Object creation

Creation events, exact: (1) binding initialization; (2) temporary construction; (3) aggregate sub-object initialization during construction; (4) allocation success (heap); (5) static initialization at program load. A created object starts fully initialized for types whose construction produces a valid value (aggregates: per §5.2.4).

### 5.1.11 Object destruction

Destruction events: scope-exit drop glue (§5.4), explicit assignment replacement (§5.6.7), deallocation (§5.11.6), frame exit. After destruction the object does not exist; its (alloc_id, offset) pair may be reused (§5.6.5).

### 5.1.12 Virtual objects

A virtual object is an object the compiler keeps in registers or SSA values without materialized storage. The abstract machine semantics are defined as if materialized; virtualization is legal iff it preserves all observable behavior of §5.6.12's identity-sensitive set (obligation OB-05-16).

### 5.1.13 Materialized objects

Materialization triggers, exact list: (1) address escape (reference/pointer formation to the object or any interior place); (2) drop glue must run (Drop impl present, or a field with glue); (3) interior-mutable access (UnsafeCell projection); (4) volatile or atomic operation on the object; (5) FFI pass. Outside these triggers the compiler may keep the object virtual for the object's whole lifetime.

---

## 5.2 Initialization

### 5.2.1 Uninitialized storage

Fresh storage (frame slots, allocated heap, statics before init) starts uninitialized: every byte's init state is U (§5.2.12). Reading uninitialized storage through a typed place is UB-1 (§5.12). Reading uninitialized storage through raw byte reads is a defined value: each byte yields a non-deterministic value in [0, 255] chosen independently; the operation carries the nondet effect atom (SPEC-INDEX §5.3). Copying uninitialized bytes (raw copies, padding) is defined and propagates the uninit state.

### 5.2.2 Initialization

An initialization event at place p of type T writes a valid value of T and sets the init state of p's bytes to I. Initialization is the only transition from U to I (dataflow §5.2.12, transfer T-Write). Assignment to an initialized place first runs the destruction path (§5.6.7), then initializes.

### 5.2.3 Full initialization

A place of type T is fully initialized iff every byte of its range has state I and the stored bytes satisfy valid(T, b, P) for the current provenance context. Only fully initialized places may be read through typed places (UB-1 otherwise), moved out of (E-P05-001 otherwise at compile time in safe code; unsafe moves of partially initialized storage are rejected by §5.3.4 rules), or dropped (§5.4.2).

### 5.2.4 Partial initialization

A place is partially initialized iff some bytes are I and some are U or moved-out. Partial initialization states exist for: aggregates after partial moves (§5.3.7), arrays during element-wise construction, and MaybeInit dataflow states (§5.2.12). Each partially initialized place carries a runtime drop-flag set at every scope-exit drop point (§5.4.10).

### 5.2.5 Reinitialization

Writing to a moved-out or uninitialized place reinitializes it: MovedFrom/U → I (state machine SPEC-INDEX §5.4: Write from Uninitialized or MovedFrom → Owned). Reinitialization is always defined; no shadow tracking of old values exists; pointers to the place remain valid (same alloc_id and offset ⇒ same object, §5.6.7).

### 5.2.6 Initialization through references

*r = v (r: &mut T) initializes the referent place: requires valid_ref(r) including caps.store (UB-2 otherwise) and executes the init event at the referent. The referent's prior value is dropped first per §5.6.7.

### 5.2.7 Initialization through pointers

Raw pointer write ptr.write(v) (unsafe) requires caps.store, live allocation, range, alignment: failure raises AddressTrap/ProvenanceTrap/AlignmentTrap (T05-1), never UB. The init event fires at the target place. Successful raw writes through provenances not derived from an exclusive reference's provenance trigger alias invalidation per §5.10.6.

### 5.2.8 Initialization of arrays

Array construction forms: [a1, …, aN] (element-wise, left to right); [v; N] with v: T requiring T: Copy or v a const expression (evaluated once, then bitwise-copied N times, per Volume 04 §4.4); iterated construction by a loop writing each element. Per-element init states are tracked by §5.2.12; a partially constructed array at a panic point drops its initialized elements in ascending index order before unwinding (§5.4.3, construction-failure rule).

### 5.2.9 Initialization of enums

Constructing variant vi with field values initializes the whole enum storage to I and writes tag = d(vi) plus the variant's fields (§5.5.3 offsets). Enums have no partial states: construction is atomic; a panic during field evaluation destroys already-constructed fields of that variant in reverse construction order (§5.4.3).

### 5.2.10 Initialization of unions

Assigning union field f writes size(f) bytes at offset 0 and marks the union's full storage I (defined regardless of field size differences). Reading any field is a typed read requiring I (UB-1 if not) and applies the representation rules of Volume 04 §4.1.13 (ReprTrap on invalid bool/char/enum bytes). Unions never have partial states.

### 5.2.11 Initialization of dynamically sized types

A DST value exists only behind a pointer. Initialization of a DST binding initializes the fat pointer (data pointer, metadata). Initialization of the pointee storage is a separate event: for heap DSTs, the allocation + element-wise writes constitute the init; the owning fat pointer's init state is independent (the fat pointer is Sized-sized: 16 bytes).

### 5.2.12 Initialization tracking

Definite-assignment is a forward must-dataflow over the control-flow graph with the lattice below. Inputs: CFG of a function body, set of places P (locals and static projections, bounded by the Volume 12 limits table), statement effects. Output: per-place init state at every program point; rejection set (E-P05-002 uses of not-definitely-initialized typed places; E-P05-005 drops of moved/uninit places); drop-flag insertion plan (§5.4.10).

Lattice per scalar place (height 4):

| State | Meaning |
| ----- | ------- |
| Unreachable | no execution reaches this point (absorbing for the analysis: excluded from meets) |
| Uninit (U) | definitely not initialized |
| MaybeInit (M) | initialized on some incoming paths only |
| Init (I) | definitely initialized |

Order: U ⊑ M ⊑ I; Unreachable ⊑ everything; meet(x, y) over reachable predecessors = greatest lower bound in this chain (meet(I, U) = M; meet(U, M) = U; meet(U, U) = U; meet(M, M) = M). Transfer functions, total:

| Event | Transfer |
| ----- | -------- |
| write/init p | I |
| move-out of p | U (SPEC-INDEX §5.4 MoveOut: Owned → MovedFrom; dataflow records U) |
| partial move of field f | field f → U; parent stays tracked per-field |
| drop of p | p → U (drop consumes init) |
| scope-end of borrowed p | unchanged (Vol 06 owns loan ends) |

Aggregate places: a struct place is the product lattice over its fields; an array place tracks: (a) exact per-element states for constant indices (element map bounded to 64 exactly-tracked constant indices); (b) an All-element state for whole-array events ([v; N], full move); (c) Indet for non-constant-index events, treated as M for reads and as flag-per-element at drop points (runtime bitmap, 1 bit per element, allocated in the frame).

Algorithm: worklist over CFG in reverse postorder; on each edge, propagate transfer(state) and meet into the successor; loop headers iterate to fixpoint. Termination (G11, obligation OB-05-7): lattice height 4 per place, transfers monotone, each worklist update strictly lowers a state in the finite product lattice; measure = Σ over places of height; maximum iterations ≤ 4 × |P| × |E|; complexity O(|P| × |E| × cost(transfer)); |P| and |E| bounded by the Volume 12 limits table. Determinism: worklist order is reverse postorder with ascending node ID (G14).

---

## 5.3 Copy and move

### 5.3.1 Copy semantics

A copy of a value of Copy type T duplicates the bytes and provenance context bitwise: source and destination are independent values after the copy. Copy never runs user code, never changes init states of the source (reads of Copy places do not consume: SPEC-INDEX §5.4 Read from Owned → Owned), and carries effect row ∅.

### 5.3.2 Copy capability

Copy is the capability of Volume 04 §4.7 (T04-9). Types without Copy are moved, never implicitly copied. Reading a non-Copy place in a value context is a move (§5.3.5); reading it in a borrow context is a loan (Volume 06).

### 5.3.3 Implicit copying

Implicit copies happen exactly at: reads of Copy-typed rvalues (operands, arguments, returns); aggregate construction of Copy fields; [v; N] repeat expansion (Volume 04 §4.4 array rule); pattern matching on Copy values. No implicit copy of non-Copy types exists in edition P1.

### 5.3.4 Explicit cloning

Clone::clone(&self) -> Self produces an independent value; it is a method call carrying the Clone impl's effect row (at least alloc for heap-owning types). Clone is never implicit. A Clone impl must not alias the source's owned resources (obligation on the impl author; enforced by review and the aliasing rules of §5.10, not by UB).

### 5.3.5 Move semantics

A move from source place s to destination place d of type T executes: (1) require init(s) = I (compile-time rejection E-P05-001 in safe code for reads of moved-from/uninit places); (2) transfer bytes and provenance context of size(T) from s to d (memcpy semantics or in-place elision per §5.3.12); (3) set state(s) → U/MovedFrom and state(d) → I (SPEC-INDEX §5.4: MoveOut then Write). A move carries effect row ∅ and never runs user code.

### 5.3.6 Move source invalidation

After a move, the source place is invalidated: reads, borrows, moves, and drops of the source place are rejected at compile time in safe code (E-P05-001; state MovedFrom per SPEC-INDEX §5.4: Read/MoveOut/BorrowShared/BorrowMut from MovedFrom → Rej). Reinitialization restores it (§5.2.5).

### 5.3.7 Partial moves

Moving one field f of an aggregate place p: permitted for non-Copy aggregates whose other fields remain accessible; sets field f → U/MovedFrom and records the parent as partially moved. The parent may not be moved or dropped as a whole while any field is moved-from, except through the drop-flag path (§5.4.10) that drops exactly the initialized fields. Partial moves out of Copy types are plain copies. Partial moves through references or into panics-crossing positions follow the same rules as whole moves (§5.3.9).

### 5.3.8 Aggregate moves

Moving an aggregate moves all initialized fields and the metadata in one transfer; the whole-aggregate state becomes U/MovedFrom. For aggregates containing DST tails, the move transfers the fat pointer (§5.2.11).

### 5.3.9 Array element moves

Moving element i of an array place moves exactly that element (per-element states §5.2.12). Elements of a Copy array are copied. Moving out of an array through a non-constant index in safe code requires the element type to be Copy; non-Copy element moves through dynamic indices are rejected (E-P05-001) because per-element tracking cannot name the index at compile time.

### 5.3.10 Moves through references

Moving out of *r (r: &mut T) is a move of the referent place: valid_ref(r) with caps.store is required (UB-2 otherwise); the referent state → U; the reference remains valid as a value (reborrow rules Volume 06 §9.4 govern further uses). Writing through the reference afterwards reinitializes. Moving a value into *r = v is initialization through references (§5.2.6).

### 5.3.11 Moves involving pinned objects

For a place p whose type T is !Unpin and which is pinned (reachable only through Pin): moving p out requires unsafe projection per Volume 06 §9.9 (Pin::get_unchecked_mut + move); safe code cannot move !Unpin values out of a Pin. Unpinned !Unpin values move freely. The address-stability guarantee of §5.1.9 applies from pinning until the destructor completes.

### 5.3.12 Relocation

A move is a relocation of the value's bytes and provenance context. Implementation contract: the compiler may implement a move as memcpy(size(T)) or elide it (in-place construction), provided the identity rules of §5.6 hold: elision is legal iff no identity-sensitive operation (§5.6.12) can distinguish the elided move from the copying move; pointers derived into the source place are invalidated in both cases (§5.3.13).

### 5.3.13 Ownership transfer

A move transfers the ownership duty (final destruction) from source to destination: exactly one destruction obligation exists for every created object (§5.5.2 of design §7.4); moves reassign it, never duplicate or cancel it. Leaking cancels the obligation by an explicit defined operation (§5.4.13), never silently.

---

## 5.4 Destruction

### 5.4.1 Destructor invocation

Destruction of a place p of type T at a scope exit or unwind point executes drop glue: (1) if T has a Drop impl: call T::drop(&mut p) — user code with the impl's effect row (§5.4.6); (2) then destroy the components: struct/tuple fields in declaration order; enum: the active variant's fields in declaration order; array elements in ascending index order 0, 1, …, N−1; closure captures in first-capture order; union: no component glue (Volume 04 §4.1.13). ZSTs and ! have no glue. This order is pinned by SPEC-INDEX §3 (scoped, innermost-first) and made exact by the table above; obligation OB-05-9 checks it exhaustively.

### 5.4.2 Destructor eligibility

A place is drop-eligible at a drop point iff its dataflow init state is I, or M with drop flags materialized (§5.4.10), or partially moved (drop only the initialized components). States U (uninitialized) and MovedFrom are not drop-eligible: no glue runs, no flag is read. The compiler must not emit glue for ineligible places; a scope-exit analysis that cannot decide emits flags (this is the definition of M).

### 5.4.3 Drop order

T05-2 (exact drop order, all paths):

| Event | Order |
| ----- | ----- |
| scope exit, normal path | local bindings in reverse declaration order; within each binding: glue per §5.4.1 |
| scope exit, unwind path | same as normal path, executed from the panic point outward to the catch boundary (innermost scope first) |
| statement temporaries | destroyed at the end of the enclosing statement, reverse creation order |
| expression temporaries (Rust-style extension) | destroyed at the end of the full statement (P1 has no shorter temporary scopes) |
| aggregate construction failure | already-constructed fields destroyed in reverse construction order, then unwinding proceeds |
| assignment replacement | old value destroyed (glue) before the new value is written (§5.6.7) |
| frame exit | all remaining live bindings, reverse declaration order |
| program exit | statics destroyed in reverse declaration order; thread-locals at thread exit, reverse declaration order per thread |

Drop order is deterministic (G14) and observable (drop impls run user code); the compiler may not reorder drops except where it proves no observable difference (no glue, no effects, no identity observation), obligation OB-05-15.

### 5.4.4 Field destruction

Field destruction is step (2) of §5.4.1, in the stated orders. Field destruction happens after the owner's Drop::drop; a Drop impl that moves its own fields out (via unsafe) suppresses their glue only through the manual-drop path (§5.4.11), tracked by the same init dataflow applied to the drop body (obligation OB-05-8 corpus covers it).

### 5.4.5 Partial destruction

Partial destruction is destruction of exactly the initialized components of a partially moved/MaybeInit place, driven by drop flags (§5.4.10). Each flag gates exactly one component's glue; flag values are maintained by the init/move events of §5.2.12.

### 5.4.6 Destructor effects

The Drop impl of T carries a declared effect row ρ_drop (SPEC-INDEX §5.3). Any context that may drop a value of T requires ρ_drop ⊆ its row; the requirement is generated at every scope that can drop T (including unwind paths). Default (no Drop impl): ρ_drop = ∅. A Drop impl containing io/alloc/panic atoms therefore constrains every caller that lets such values go out of scope; the constraint is checked by Volume 07's effect checker.

### 5.4.7 Destructor panic

A panic raised inside Drop::drop starts unwinding per §5.4.8. The destructor completes its own frame's remaining glue only per the unwind rules (innermost-first from the panic point); components of the dropped value after the panicking destructor are still destroyed if the panic is caught below their scope, else they leak by defined behavior (no UB).

### 5.4.8 Unwinding

Unwinding is defined unwinding to the enclosing catch boundary (pinned): panic starts unwinding; every scope between the panic point and the catch boundary executes its §5.4.3 unwind-path order; at the catch boundary the payload is delivered (Volume 07 §11.6). Drop code cannot observe the panic payload (pinned: no API exposes the payload during unwinding). `?`/try propagation is value propagation, not unwinding (Volume 07).

### 5.4.9 Abort behavior

Panic = abort builds (build-mode flag, Volume 12) replace unwinding with the immediate trap: the process aborts with TRAP_EXIT_CODE = 134 at the panic point, with no glue executed. Traps (T05-1) always abort with exit code 134 in every build mode.

### 5.4.10 Double panic

A panic raised while unwinding (in a destructor or drop glue during an active unwind) is a double panic: the process aborts immediately with TRAP_EXIT_CODE = 134. No glue runs after the double panic; no catch boundary is consulted. Detection is exact: the runtime unwinder maintains an in-unwind flag (Volume 18 §25); a second unwind start while the flag is set aborts.

### 5.4.11 Drop flags

Drop flags are runtime booleans in the frame, one per sub-place whose compile-time state is M at some drop point (§5.2.12 Indet array elements use a 1-bit-per-element bitmap). Flag updates are emitted exactly at the corresponding init/move events. Total flag bits per function must not exceed DROP_FLAG_BITS_LIMIT = 65536; exceeding ⇒ compile-time rejection E-P05-004 (report suggests refactoring). Flags are dead after their scope; flag memory is frame storage (§5.7.1).

### 5.4.12 Manual destruction

core::intrinsics::drop_in_place<T>(ptr: *mut T) is the only manual destruction primitive (unsafe). Declared precondition (Volume 10 §16.1): ptr's provenance denotes a place holding a valid value of T that has not been destroyed and is not borrowed elsewhere. Violation = UB-6. drop_in_place runs exactly the glue of §5.4.1 and sets the place state to U in the abstract machine (in unsafe contexts the dataflow is not compiler-enforced; the obligation is on the programmer, checked only where the borrow checker can see it, Volume 06 §9.8).

### 5.4.13 Explicit resource release and deliberate leaking

Explicit resource release is user-defined API (close/run/deallocate methods); the compiler enforces nothing beyond the borrow rules; double-release is defined by the type's API contract (typically an error return), never UB. Deliberate leaking: core::mem::forget(v) and Box::leak are defined operations: they suppress the destruction obligation (no glue runs) and the bytes remain counted against heap limits (§5.11.5). Leaking is a defined value operation with effect row {alloc}; it can exhaust the heap budget → defined AllocError at later allocations, never UB.

### 5.4.14 Interaction with FFI

Unwinding across an FFI boundary is a compile-time rejection (pinned): a function value with the panic atom in its row must not be converted to a foreign-ABI fn pointer nor declared with a foreign ABI (E-P05-007). Foreign code never executes Planck drop glue; values passed to foreign code by value are passed per Volume 10 §17.1 ABI and the caller keeps the destruction obligation; values whose ownership is transferred to foreign code must be explicitly leaked or reclaimed through the documented foreign contract (Volume 10 §17.3).

---

## 5.5 Layout

### 5.5.1 Size, alignment, field offsets, padding

All layout math is over u64 byte quantities. Definitions, exact:

- size(T): byte count of a value of T; size(T) ≤ MAX_ALLOC_SIZE = 2^48.
- align(T): power of two, 1 ≤ align(T) ≤ MAX_ALIGNMENT_BYTES = 2^29.
- align_up(x, a) = (x + a − 1) & !(a − 1), for a = 2^k, k ∈ [0, 29]; defined for x + a − 1 < 2^64 (layout computations that overflow are compile-time rejections E-P05-008).
- Field offsets for a repr(P) aggregate: align ← 1; for each field f in declaration order: align ← max(align, align(f)); off(f) ← align_up(off, align(f)); off ← off + size(f). Final: size = align_up(off, align); padding = bytes in [0, size) not covered by any field range; padding bytes are uninitialized on aggregate creation (§5.2.4), never read by typed field access, and copied as raw bytes by aggregate copies and moves.

repr(P) is the default for all user aggregates (pinned): fields in declaration order, alignment = max field alignment, size = padded to that alignment, no reordering, no niche packing unless #[repr(niche)].

### 5.5.2 Enum discriminants

Variant count N; explicit discriminants must be distinct (Volume 04 §4.1.12). Tag width W for repr(P) and repr(C) enums: N ≤ 2^8 → 1 byte; N ≤ 2^16 → 2; N ≤ 2^32 → 4; else 8. Tag placement: offset 0, align W. Layout: payload_align = max field align over all variants; payload_base = align_up(W, payload_align); each variant's fields laid sequentially from payload_base by the §5.5.1 rules; P_max = max variant payload size (0 if all ZST); enum align = max(W, payload_align); size = align_up(payload_base + P_max, align). A variant-less (uninhabited) enum: size 0, align 1. repr(Int) enums: tag = that integer type (W = its width), no payload. Typed reads of tag patterns that match no discriminant raise ReprTrap (T05-1), not UB.

### 5.5.3 Niche representations

#[repr(niche)] encodes the discriminant in a validity hole instead of a separate tag. Hole search, exact: scan variants in declaration order; within a variant, scan fields in declaration order; the first field whose type has a validity hole H is selected. Hole capacities (exact): bool in 1 byte: 254 hole values (0x02..=0xFF); char in 4 bytes: 2^32 − 1112064 hole values (all non-scalar encodings); &T / non-null pointer types: 1 hole value (all-zero address); nested #[repr(niche)] enums: their hole. Discriminants are encoded into H's hole values in variant order (first hole value = first variant). Size = the layout of the variant payloads as a union with the tag embedded; align = max field align. If no hole exists, #[repr(niche)] falls back to §5.5.2 tag layout (defined, no diagnostic). Soundness of hole counting is obligation OB-05-5.

### 5.5.4 Zero-sized types

size = 0 types: no storage, no padding; align = max field align (possibly 1); offsets of ZST fields are the running offset (they consume 0); pointers to ZST places: valid iff offset ≤ allocation size (§5.10.1 clause for ZSTs); passing a ZST by value consumes no registers/stack slots (Volume 10 §17.1). ZST drops are no-ops (§5.4.1).

### 5.5.5 Aggregate packing

"Aggregate packing" in P1 is exactly #[repr(packed)] and #[repr(packed(N))]: with packed: every field's effective align = 1; offsets sequential without padding; aggregate align = 1. With packed(N), N ∈ {1,2,4,8,16,32,64} (PACKED_ALIGN_LIMIT = 64): effective field align = min(align(f), N); aggregate align = min(max field align, N). Rules: taking a reference to a field of a packed struct is rejected (E-P05-006, because the referent would violate alignment validity UB-2 conditions); field reads/writes of packed aggregates are performed as unaligned accesses (defined values, §5.9.11 unaligned path); packed is the user's explicit request, so no trap is raised for aligned-requirement violations inside packed aggregates.

### 5.5.6 Explicit alignment

#[repr(align = A)]: A ∈ {1, 2, 4, …, 2^29}. Aggregate align = max(natural align, A); offsets recomputed under the raised align (§5.5.1 rules); size padded to the raised align. A > 2^29 or non-power-of-two A ⇒ E-P05-009. Explicit alignment may only raise, never lower (packed handles lowering).

### 5.5.7 Packed structures

Covered exactly by §5.5.5; the bullet "Packed structures" and "Aggregate packing" are two design bullets sharing one definition home (both ledger rows anchor §5.5.5).

### 5.5.8 Transparent structures

#[repr(transparent)]: the aggregate must contain exactly one non-ZST field (plus any number of ZST fields); size, align, and ABI class equal the non-ZST field's; a transparent aggregate is FFI-usable (§5.5.10). Violation of the single-non-ZST rule ⇒ E-P05-010.

### 5.5.9 C layout

#[repr(C)] computes by the same arithmetic as §5.5.1 (structs) and §5.5.2 (enums: tag-first), with these additional contracts: it is the layout guaranteed across editions for FFI compatibility; it is the only aggregate layout (with repr(transparent), primitives, SIMD, and repr(C)-foreign types) permitted at FFI boundaries (Volume 10 §17.1); repr(P) aggregates crossing an FFI boundary are rejected (E-P04-014, Volume 04 §4.1.11). #[repr(C)] does not reorder fields and does not niche-pack.

### 5.5.10 SIMD layout

SIMD types (Volume 04 §4.1.22): size = width/8 bytes (≤ 64), align = size. ABI class Vector; passed in vector registers per target ABI (Volume 10 §17.1, Volume 16 §23). Element stride = element size; lane order is element index order (lane 0 = lowest bytes).

### 5.5.11 Foreign layout

Foreign types (Volume 04 §4.1.23): #[repr(C)] foreign declarations compute by §5.5.9; `extern type` declarations are opaque: size unknown (usable only behind pointers), align 1, no Copy/drop glue unless declared. Foreign layout overrides are trusted declarations: misdeclaring is a foreign-contract violation (Volume 10 §17.2), not Planck UB.

### 5.5.12 Layout reflection and layout compatibility

Layout reflection: core::meta::layout_of::<T>() returns (size(T), align(T)) as a const fn for T: Sized (Volume 04 §4.10.4); results must equal §5.5.1–§5.5.11 arithmetic (obligation OB-05-3). Layout compatibility: layouts L1, L2 are compatible iff size(L1) = size(L2) ∧ align(L1) = align(L2) ∧ ABI-class(L1) = ABI-class(L2), where ABI class ∈ {Scalar(int/float/bool/char), Vector, Aggregate-C, Aggregate-P, FatPtr(len), FatPtr(vtable), Fn, ZST, Opaque-Foreign} is derived from T04-1. Compatibility is the relation used by FFI checks and by transmute-style intrinsics (Volume 10): incompatible transmute = intrinsic precondition violation = UB-6.

---

## 5.6 Identity and optimization

### 5.6.1 Identity preservation

Compilation must preserve the answers of all identity observations (§5.6.12): the compiler must not merge, split, or relocate objects in ways that change pointer comparisons, exposure results, or drop-time addresses that the source can observe. Obligation OB-05-15 (REVIEW, exact invariant stated there).

### 5.6.2 Address escape

Address escape = any event that makes an address observable outside the place's own evaluation: reference/pointer formation, pointer stored into memory or passed to a call, ptr-to-int expose. Escape materializes virtual objects (§5.1.13) and forbids storage reuse of the escaped range while any live provenance to it exists (§5.6.5).

### 5.6.3 Pointer identity

Pointer identity is the provenance pair: two pointer values are the same pointer iff same alloc_id, same offset, and equal capability relevance at use (caps may differ; caps affect access legality, not identity). Pointer identity is preserved by copies (pinned: provenance preserved by copies) and narrowed by arithmetic (§5.8.9: offset changes; caps preserved; validity range narrows).

### 5.6.4 Pointer comparison

Comparisons ==, ≠, <, ≤, >, ≥ on references and raw pointers are defined on the (alloc_id, offset) pair: equality holds iff both components are equal; ordering is lexicographic on (alloc_id, offset). Null (alloc_id 0, offset 0) is less than every non-null pointer. Two pointers to the same address from different allocations are not equal and not observable as equal (pinned). Comparing pointers of different fat kinds (len vs vtable metadata) compares the data words and is defined; metadata comparison is rejected at compile time (E-P05-011).

### 5.6.5 Storage reuse

A dead storage range (object destroyed) may be reused for a new object iff no live provenance with load/store caps can reach it: reachability = provenance recorded in any live pointer value in the machine state with matching alloc_id and overlapping offset. Reuse without this condition would make old pointers observe the new object, breaking §5.6.1; it is forbidden (compiler-side check obligation OB-05-16). Frame slots: reuse is decided per-slot liveness by the dataflow of §5.2.12 extended with pointer liveness (Volume 13 §20 memory SSA records provenances).

### 5.6.6 Storage lifetime

Storage lifetime of each region is §5.7 (per-region table). A pointer whose allocation dies holds a dangling provenance (§5.8.17): raw dereference → ProvenanceTrap; reference formation → UB-2 (§5.10.1).

### 5.6.7 Object replacement

Assignment p = v on an initialized place destroys the old object (glue per §5.4.1) and creates a new object at the same (alloc_id, offset): object identity is unchanged (same pair), the value is new. Pointers to p stay valid across replacement; pointers derived from interior addresses of value contents (e.g., inside a moved-in heap buffer) are unaffected because they point into their own allocations.

### 5.6.8 Aggregate virtualization

An aggregate may be virtualized (kept in SSA parts) iff none of the §5.1.13 triggers fires for it or any interior place. Legality predicate, exact: no reference/pointer formation to the aggregate or any interior place appears in the aggregate's live range; no Drop glue needed (no glue at all); no UnsafeCell projection; no volatile/atomic op; no FFI pass. Virtualization is obligation-checked by OB-05-16.

### 5.6.9 Scalar replacement

Scalar replacement decomposes an aggregate into scalars, each stored independently, legal iff the §5.6.8 predicate holds and the aggregate's value equivalence (§5.1.6) is preserved at every observation point (reads, copies out, drops of initialized parts). This is the same legality condition as aggregate virtualization specialized to full decomposition.

### 5.6.10 Object merging legality

Merging two objects (commoning identical values) is legal iff: both are of the same type, value-equivalent, and no identity-sensitive operation (§5.6.12) can distinguish them. Exact conservative check: merge permitted when neither object's address is escaped (no live provenance to either), or when all observations are through value-equality only. When in doubt, do not merge (conservative legality is the normative rule; obligation OB-05-17 covers the predicate's conservativity by construction).

### 5.6.11 Allocation elimination legality

Eliminating an allocation (heap or frame) is legal iff: (1) no live provenance to the allocation can be formed or observed after the elimination point (no escape §5.6.2), (2) the allocation's effect on observable behavior is limited to values that survive in SSA form, (3) drop glue of the object is either absent or rewritten onto SSA values. Heap allocation elimination must additionally preserve allocation-count observability: allocation counts are observable only through foreign allocators and the Volume 12 profiling hooks, which are declared observability points (allocation elimination across them is legal because counts are not part of source semantics; this is a settled definition).

### 5.6.12 Identity-sensitive operations

The exact set of operations that observe identity (the compiler's no-observe obligation list): (1) pointer equality/ordering comparisons (§5.6.4); (2) ptr-to-integer exposure and int-to-pointer reconstruction (§5.8.11–§5.8.12); (3) passing pointers across FFI; (4) Drop::drop observing self's address through unsafe; (5) volatile/atomic operations on addresses; (6) allocation-identity-dependent behavior of foreign allocators. For every transformation, the invariant is: the set of possible results of each identity-sensitive operation in the compiled program equals the set in the source semantics (obligation OB-05-15).

---

## 5.7 Memory regions

T05-3 (regions, one row per §8.1 bullet):

| Region | Created by | Lifetime | Allocation IDs | Access contract |
| ------ | ---------- | -------- | -------------- | ---------------- |
| Stack | frame entry: one slot per binding, temporary, flag, spill | from slot init to scope exit / frame exit; frame lifetime bounded by STACK_LIMIT_MAIN = 8388608 bytes (main), STACK_LIMIT_SPAWNED = 2097152 (spawned); exceed = StackTrap | frame-local sub-IDs (alloc_id = frame ID, offset = slot offset) | all accesses; provenance dies at frame exit (§5.8.17) |
| Heap | allocator calls (§5.11.1) | from alloc success to dealloc or program end (leak) | monotonic global IDs | all accesses through provenance (§5.9) |
| Static | program image / static items / promoted constants | program lifetime | assigned at load, deterministic ascending order (G14) | all accesses; shared across threads (sync per §5.13) |
| Thread-local | thread-local items, one instance per thread | thread lifetime | one ID per thread instance | thread-private; cross-thread access only via API (Vol 08) |
| Foreign | FFI-obtained pointers (malloc, foreign APIs) | foreign contract (Volume 10 §17.3) | class ID 2, offset = address | access through foreign provenance; faults = MemoryFaultTrap; no Planck drop glue |
| Memory-mapped | unsafe mmio API (Volume 10 §16.2, Volume 18) | mapping contract | class ID 3 per mapping | volatile/atomic access only (§5.9.12); faults = MemoryFaultTrap |
| Allocator-managed | the global allocator provider (§5.11.1) | per allocation | monotonic global IDs | provider contract = unsafe trait with UB-6 obligations per method |
| Temporary storage | expression temporaries | end of enclosing statement (§5.4.3) | frame-local sub-IDs | as stack |
| Storage lifetime | this row is the rule set itself: a region's lifetime is the row above; a place is dereferenceable exactly within its region's lifetime | — | — | validity predicate §5.10.1 clause (a) |

---

## 5.8 Pointers

### 5.8.1 Pointer types

*const T (shared raw), *mut T (exclusive raw), fn pointers (Volume 04 §4.1.16), fat variants for DST targets (metadata: len: usize or vtable pointer). Pointer size: 8 (thin) / 16 (fat); align 8.

### 5.8.2 Pointer formation

Formation events: &e and &mut e (from places, Volume 04 §4.1.14); ptr::addr_of!(place) / addr_of_mut!(place) (raw formation without creating a reference: no UB-2 at formation, validity checked at use); decay coercions (Volume 04 §4.3.3); allocation returns (§5.11.1); int-to-pointer (§5.8.12). Formation from a place computes provenance = (alloc_id of the enclosing allocation, offset of the place, caps: {load, compute} for shared forms, {load, store, compute} for exclusive forms).

### 5.8.3 Null pointers

The null pointer is the value with alloc_id = 0, offset = 0 (any address bits are ignored for semantics: the null provenance discards them). Null is a defined value of every pointer type. Dereferencing null: raw → AddressTrap; reference creation → UB-2 (invalid reference). fn-pointer null (bit pattern 0): call → CallTrap (Volume 04 §4.1.16).

### 5.8.4 Non-null pointers

NonNull<T> is a library wrapper (Volume 19) typed to exclude null by construction; its safety contract is the wrapper's invariant (offset/alloc valid), enforced by the wrapper's unsafe constructors. The compiler gives NonNull no special semantics beyond its type.

### 5.8.5 Pointer provenance

Provenance is the pinned record Provenance { alloc_id: u64, offset: u64, caps: ⊆ {load, store, compute} }. Caps restrict access: load = readable, store = writable, compute = arithmetic/comparison-capable. Every pointer value carries one provenance; copies preserve it bit-exactly (pinned); IR 1 memory SSA tracks provenance per SPEC-INDEX §3; provenance is part of the UB-2 judgment (§5.10.1).

### 5.8.6 Provenance creation

Creation events (exact): allocation success (full caps, offset 0); reference/pointer formation from a place (§5.8.2); int-to-pointer reconstruction within an exposure window (§5.8.12, caps {compute} only unless reconstructed through the alloc-provenance path, which grants {load, compute}; store is granted only by reconstruction from a pointer that had store). No other event creates provenance.

### 5.8.7 Provenance preservation and exposure

Copies (moves of pointer values) preserve provenance bit-exactly (pinned). Arithmetic narrows: see §5.8.9. Exposure: casting a pointer to usize records (alloc_id, [base + offset, base + offset + size(T)]) into the execution's exposure set E (bounded by EXPOSE_TABLE_LIMIT = 2^20 entries; overflow ⇒ the oldest entry is evicted, defined FIFO behavior); int-to-pointer consults E (§5.8.12). Releasing exposure: there is no unexpose primitive in P1 (settlement: exposure is monotone within an execution).

### 5.8.8 Pointer arithmetic

p.add(n) (n: usize, scaled by stride = size(T)): new provenance = (same alloc_id, offset + n × stride, same caps); offset arithmetic is checked u64 math; a result with offset > allocation size is a defined value (provenance valid range narrows to empty: every later dereference traps); p.add with offset + n × stride > 2^64 saturates to a defined "out-of-range" provenance (offset = 2^64 − 1 marker) — no wrap-around. p.sub(n) symmetric. p.wrapping_offset(k) (i64): defined without validity considerations: offset = offset + k × stride in 2^64 wrap-around arithmetic, provenance unchanged; dereference still checked. Arithmetic on fat pointers scales by the element/tail stride and preserves metadata unchanged. Arithmetic requires caps.compute.

### 5.8.9 Pointer subtraction

ptr_a.offset_from(ptr_b) (i64 result): requires same alloc_id and both non-null (else ProvenanceTrap); result = (offset_a − offset_b) / stride, truncated toward zero; exact division remainder is defined as discarded. Subtraction of pointers with different alloc_ids is a defined trap (ProvenanceTrap), not UB. ptr_diff via usize casting is outside this rule (it is the exposure path, §5.8.11).

### 5.8.10 Pointer comparison

Defined on (alloc_id, offset) lexicographically (§5.6.4). == between a pointer and null tests alloc_id = 0. Ordering between different allocations is by alloc_id (defined, stable per execution, G14). Comparisons require caps.compute.

### 5.8.11 Pointer-to-integer conversion

p as usize: yields base + offset of the provenance if alloc_id ≥ 1 (live or dead: addresses of dead allocations are the recorded base; for freed allocations the recorded base remains defined); null provenance yields the address bits (which may be 0). The conversion records an exposure entry (§5.8.7). The result is a defined usize; the nondet atom is not carried (addresses are stable per §5.1.9 within lifetimes).

### 5.8.12 Integer-to-pointer conversion

addr as *const T / *mut T: consult the exposure set E: if the integer lies within an exposed range of some allocation a, the result is provenance (a.id, integer − base(a), caps {load, compute} if the exposure's source caps included load, plus store if included store; caps = intersection of the exposed source pointer's caps with {load, store, compute} minus nothing — reconstruction never widens beyond the exposed source's caps); if the integer lies in no exposed range, the result is null provenance (alloc_id 0) with offset = integer. Reconstruction from an allocation that has died yields a dangling provenance (§5.8.17). Reconstruction never grants store unless the exposed pointer had it.

### 5.8.13 One-past pointers

The provenance range of an allocation is [0, size]: offset = size (one-past-the-end) is a valid provenance offset for arithmetic, comparison, and formation of pointers; dereferencing at one-past is out of range: raw → ProvenanceTrap; reference creation → UB-2. Zero-sized targets: any offset ≤ size is valid to form; all dereference attempts trap.

### 5.8.14 Dangling pointers

A provenance is dangling iff its allocation is dead (freed, frame exited) or its valid range is empty (§5.8.8). Dangling raw dereference → ProvenanceTrap; creating a reference from dangling → UB-2; comparing/arithmetic on dangling → defined. Dangling is observable only through these defined behaviors, never through safe code (pinned: freed addresses are not observable through safe code).

### 5.8.15 Pointer invalidation

Invalidation events (exact): deallocation of the allocation (§5.11.6); frame exit (stack provenances); realloc of the allocation (old provenances die, §5.11.7); move of a value whose interior addresses were escaped (pointers into a moved aggregate's storage die at the move: the source frame slot's sub-range is dead after the move; the destination is a new object, §5.3.12); thread exit for thread-locals. After invalidation: caps remain recorded but the valid range is empty; uses per §5.8.14.

### 5.8.16 Pointer equality

== on (alloc_id, offset) (§5.6.4). Two distinct allocations with reused addresses (freed then reallocated at the same base) are never equal (pinned). Equality is deterministic per execution (G14) and must be preserved by compilation (OB-05-15).

### 5.8.17 Pointer ordering

<, ≤, >, ≥ lexicographic on (alloc_id, offset) (§5.6.4). Ordering across regions is defined (no restriction to same-allocation ordering; C-style same-buffer ordering is a consequence of same alloc_id). Ordering of fat pointers compares data words; metadata comparison banned (E-P05-011).

### 5.8.18 Pointer reconstruction

Reconstruction = int-to-pointer within exposure (§5.8.12) — the only defined reconstruction path in P1. Reconstruction outside exposure yields null provenance (defined value; later use → AddressTrap). The exposure set is bounded (2^20 entries, FIFO eviction). This settles "Pointer reconstruction" as the expose/recreate model, consistent with the pinned provenance decision.

---

## 5.9 Memory access

### 5.9.1 Read access

Typed place read of T at place p: requires full initialization (UB-1 if not, §5.12) and the representation invariant (ReprTrap for bool/char/enum violations at the read; UB-2 for invalid reference fields at their use). The read produces the value per §5.1.1. Raw byte reads (unsafe): produce bytes with uninit bytes as non-deterministic values (§5.2.1) and require caps.load + live allocation + range (trap otherwise).

### 5.9.2 Write access

Typed place write: destruction-first (§5.6.7), then init event. Raw write: caps.store + live + range + alignment (traps otherwise, never UB). Writes of uninit-marked bytes are defined (writing a partially initialized value writes all bytes of the written type).

### 5.9.3 Initialization requirements

Typed reads require I; typed writes require only addressability; raw accesses require nothing about init (defined values, §5.2.1). UB-1 fires exactly on typed reads (including reads through references, moves, drops of fields, and aggregate copies of typed content) of storage whose init state does not cover the read range.

### 5.9.4 Alignment requirements

Typed place access requires address ≡ 0 (mod align(T)); misaligned typed access through a place whose alignment the compiler cannot prove raises AlignmentTrap (inserted check); misaligned reference creation is UB-2 (validity clause (c)). read_unaligned/write_unaligned intrinsics perform unaligned typed access as defined values (no trap) with caps requirements unchanged. Packed aggregate fields use the unaligned path by construction (§5.5.5).

### 5.9.5 Access width

Access width = size(T) bytes for typed access; raw accesses declare width in the intrinsic (read_bytes(w), w ∈ [1, MAX_ALLOC_SIZE] and within the provenance range). Widths must not span allocation end (trap: ProvenanceTrap).

### 5.9.6 Valid memory ranges

The valid range of a provenance is [offset, alloc.size] (inclusive one-past, §5.8.13) intersected with [0, alloc.size] and emptiness rules of §5.8.8. Accesses outside the valid range: raw → ProvenanceTrap; typed through reference → UB-2.

### 5.9.7 Mutability

Store access requires caps.store. A pointer lacking store that is written through: raw → ProvenanceTrap (capability check at dereference, pinned); through reference → UB-2 (validity clause (d)). Interior mutability (UnsafeCell) is the only safe-code write path through shared provenance (§5.10.5).

### 5.9.8 Aliasing

Accesses alias iff their byte ranges overlap. Alias legality is governed by §5.10 (reference contracts) and the borrow checker (Volume 06) for safe code; unsafe code follows the reference validity contracts only (raw pointers impose no aliasing obligations on each other, §5.10.2).

### 5.9.9 Overlapping accesses

Typed overlapping accesses are regulated by the borrow checker in safe code. Raw copy intrinsics copy_nonoverlapping(src, dst, n): declared precondition "ranges disjoint" — violation = UB-6 (intrinsic precondition, Volume 10 §16.1). The overlapping raw copy intrinsic (copy_overlapping) is defined: copies byte-wise in ascending order, overlap tolerated. memcpy-style ops on non-Copy typed content must not be used to duplicate ownership (that is a Volume 06 unsafe-integration obligation, not UB).

### 5.9.10 Invalid accesses

Invalid access outcomes, exact mapping (each row also appears in the §5.12 catalog):

| Condition | Raw path | Reference path |
| --------- | -------- | -------------- |
| null provenance | AddressTrap | UB-2 |
| dead allocation | ProvenanceTrap | UB-2 |
| out of range | ProvenanceTrap | UB-2 |
| misaligned | AlignmentTrap | UB-2 |
| missing caps | ProvenanceTrap | UB-2 |
| index out of bounds | BoundsTrap (checked indexing) | — |
| null fn pointer call | CallTrap | — |
| wrong ABI/signature call | UB-4 | UB-4 |

### 5.9.11 Unaligned access

Unaligned typed raw access without the unaligned intrinsic: AlignmentTrap (§5.9.4). With read_unaligned/write_unaligned: defined values, byte-wise composed per little-endian target layout, caps and range checks unchanged. Unaligned atomic access: not provided; atomic ops on misaligned addresses raise AlignmentTrap (§5.13.4).

### 5.9.12 Volatile access

Volatile typed reads/writes (intrinsics read_volatile/write_volatile, Volume 10 §16.1): same validity/caps/range checks as raw typed access (traps, never UB); additionally: not reordered with respect to other volatile accesses to overlapping or identical ranges, not eliminated, not duplicated, not commoned. Volatile does not imply atomicity or cross-thread ordering (§5.13.7). Required for Mmio region access (§5.7.6): non-volatile access to Mmio is a compile-time rejection (E-P05-012).

### 5.9.13 Atomic access

Atomic ops (Volume 08 §12.4 pins the full model; this volume pins the access contract): supported widths {1, 2, 4, 8, 16} bytes (MAX_ATOMIC_WIDTH_BYTES = 16), alignment = width (misaligned → AlignmentTrap); caps per direction (load/store); the accessed range must lie in the valid range (trap otherwise). Atomic ops are always defined operations (never UB except the race condition UB-3 which arises from the pairing of accesses, §5.13.9).

### 5.9.14 Foreign memory access

Access through foreign provenance (class ID 2): caps as granted by the FFI contract declaration (Volume 10 §17.2); faults (unmapped, protection) → MemoryFaultTrap; no initialization tracking exists for foreign storage (typed reads of foreign memory apply the representation rules: invalid bool/char/enum bytes → ReprTrap; UB-1 does not fire because uninit tracking is not defined for foreign storage — the read yields defined bytes).

---

## 5.10 Aliasing model

### 5.10.1 Reference aliasing and the validity predicate

valid_ref(r) for reference r to type T with provenance (a, o, caps) at use time t:

(a) allocation a is live at t;
(b) o + size(T) ≤ alloc.size, for T with size(T) > 0; for ZSTs, o ≤ alloc.size;
(c) o mod align(T) = 0;
(d) caps include the access capability of the use (load for shared reads; store for exclusive writes);
(e) exclusivity: for &mut T, during r's lifetime no write to any byte of [o, o + size(T)) via a provenance p with ¬derived(p, r.prov) occurred, except writes through r-derived provenances;
(f) shared-frozen: for &T with T not Freeze (Volume 04 §4.7), during r's lifetime no write to bytes of [o, o + size(T)) outside UnsafeCell sub-ranges via a provenance p with ¬derived(p, r.prov).

Creating a reference (from a place, from a raw pointer, by reborrow) or using an existing reference with ¬valid_ref(r) is UB-2 (the single UB category for reference validity; SPEC-INDEX §3). derived(p, q): q's provenance is obtained from p's by a chain of {pointer copies, pointer arithmetic, ptr-to-ptr casts, int-to-pointer reconstruction from an exposure created by p (or by a pointer derived from p)}.

### 5.10.2 Raw pointer aliasing

Raw pointers impose no aliasing restrictions on each other: two raw pointers may alias freely; all raw accesses are defined operations subject to the trap checks (T05-1). Raw pointers alias references only in the direction of the contracts (e)/(f): a raw write into an exclusive reference's range is exactly the (e) violation.

### 5.10.3 Mutable alias restrictions and shared alias rules

Mutable (&mut) restriction = contract (e). Shared rules = contract (f) plus: shared references may alias each other freely; reads never invalidate. These two bullets share one definition home (the two contract clauses); both ledger rows anchor §5.10.1.

### 5.10.4 Interior mutability

UnsafeCell<T> is the interior-mutability marker type (Volume 04 §4.7 Freeze row; Volume 19 for the API). Semantics: bytes inside UnsafeCell sub-ranges are exempt from contract (f) (writes through any provenance to those bytes are permitted and do not invalidate shared references); UnsafeCell<T> is Sync iff T: Send and never Freeze. Accessing UnsafeCell content requires unsafe; the borrow checker does not track loans into UnsafeCell interiors (Volume 06 §9.8).

### 5.10.5 Provenance-based alias analysis

The compiler's alias analysis operates on provenances in IR 1 memory SSA (SPEC-INDEX §3): two memory operations may be reordered/removed as conflicting-free iff their provenance alloc_ids differ, or their ranges are disjoint, or both are reads, or both go through UnsafeCell sub-ranges. Conservative default when provenance is unknown: treat as possibly-aliasing (no transformation). This is the definition home of the "provenance-based alias analysis" bullet.

### 5.10.6 Alias invalidation

A write violating (e)/(f) invalidates the affected references: every reference whose validity depends on the violated clause becomes invalid at that moment; its subsequent creation/use is UB-2. Invalidation is a semantic event (the abstract machine records it); the compiler does not need to detect it (it is UB, obligation OB-05-13 lists it as outside the compiler's checked domain).

### 5.10.7 No-alias guarantees

The compiler may assume, for each exclusive reference r live at program point t: no write via provenance ¬derived-from-r touches r's range during r's lifetime (clause (e) holds because violations are UB-2). Formal guarantee statement (used by Volumes 14/16): if the program is UB-free and type-safe, every such assumption holds; transformations may exploit it unconditionally on well-typed programs. The guarantee is not extended to raw pointers (§5.10.2).

### 5.10.8 Unknown alias regions

Regions with unknown provenance (opaque FFI calls, inline assembly with memory clobbers, foreign threads): the checker treats all provenances not passed as arguments as possibly-written during the opaque event. For safe code this is invisible: safe references passed as arguments retain validity because a violation by the callee would itself be UB-2/UB-4 inside the callee's frame (foreign code cannot hold Planck provenances, §5.4.14). For unsafe code, the rule is exact: after an opaque event, unsafe-held raw pointers to memory the event could touch are not guaranteed dereferenceable unless rederived (documented contract, Volume 10 §17.3).

### 5.10.9 Compiler alias assumptions

The complete list of assumptions the compiler may exploit on well-typed, UB-free programs: (1) clause (e) no-alias for exclusive references (§5.10.7); (2) clause (f) frozen-shared for non-Freeze-free shared references (reads are stable); (3) UnsafeCell-exemption only within UnsafeCell ranges; (4) allocation liveness per §5.7 lifetimes; (5) init states per §5.2 for typed reads (UB-1 excluded); (6) alignment per type layout (UB-2 clause (c) excluded). Every assumption names the UB category that excludes its violation; that is the exact bridge between assumptions and the UB catalog (obligation OB-05-13).

---

## 5.11 Allocation

### 5.11.1 Allocation APIs

The exact core API surface (core::alloc, backed by the global allocator provider; the provider is an unsafe trait, Volume 04 §4.6.12):

```
fn alloc(layout: Layout) -> Result<NonNull<u8>, AllocError>;
fn alloc_zeroed(layout: Layout) -> Result<NonNull<u8>, AllocError>;
unsafe fn dealloc(ptr: NonNull<u8>, layout: Layout);
unsafe fn realloc(ptr: NonNull<u8>, layout: Layout, new_size: u64) -> Result<NonNull<u8>, AllocError>;
```

struct Layout { size: u64, align: u64 } with the invariants size ≤ MAX_ALLOC_SIZE, align = 2^k, 1 ≤ align ≤ 2^29 (violating constructors are compile-time rejections E-P05-008/E-P05-009).

### 5.11.2 Allocation layouts

Each allocation records its exact Layout at creation. alloc_zeroed initializes all bytes to 0x00 and the init state to I. alloc leaves bytes uninitialized (state U, §5.2.1). The recorded layout is the dealloc/realloc precondition operand (§5.11.6).

### 5.11.3 Alignment

The returned pointer's provenance offset is 0 and the allocation's base address satisfies base mod align = 0 (provider obligation; a provider returning unaligned bases violates its unsafe-trait contract = UB-6 at the provider call). Alignment values above 2^29 are unrepresentable (E-P05-009).

### 5.11.4 Allocator identity

Exactly one global allocator provider is linked per program (Volume 18 §25 configures it); it is a named unsafe-trait impl. Additional named allocator instances (pools) are library types (Volume 19) with the same contract shape. Pointer provenances do not record the provider; dealloc must use the provider that created the allocation (mixing providers violates the dealloc precondition = UB-6).

### 5.11.5 Allocation failure and limits

Allocation failure returns Err(AllocError) with enum AllocError { OutOfMemory, LayoutTooLarge, LimitExceeded } (complete; CEP:FAILURE discipline applies to the extraction). Failure never aborts and never traps. Limits: per-allocation size ≤ MAX_ALLOC_SIZE = 2^48; per-process heap byte budget HEAP_BUDGET and allocation-count budget defined in the Volume 12 limits table (the definition home of compiler/runtime limits); exceeding → AllocError::LimitExceeded (extraction obligation OB-05-10 checks the error mapping exhaustively).

### 5.11.6 Deallocation

dealloc(ptr, layout): declared precondition (Volume 10 §16.1): ptr's provenance has alloc_id ≥ 2 (a real Planck allocation), the allocation is live, and layout equals the allocation's recorded Layout exactly. Violation of any clause = UB-6 (this is the catalog's mapping for double-free and invalid free). On success: the allocation dies (§5.8.15 invalidation), the object at offset 0 is destroyed implicitly (its glue must have been run or suppressed by the caller first — the caller's obligation, checked by Volume 06 where visible); the allocation ID is never reused within the execution (monotonic IDs make address-reuse identity-safe, §5.6.4).

### 5.11.7 Reallocation

realloc(ptr, layout, new_size): precondition as dealloc plus new_size ≤ MAX_ALLOC_SIZE. On success: a new allocation is created with the same provider; min(old size, new size) bytes are copied; the old allocation dies; old provenances become dangling (§5.8.15). The returned pointer has provenance (new alloc_id, 0, {load, store, compute}). On failure: AllocError, the old allocation remains live and unchanged (strong guarantee, defined).

### 5.11.8 Placement construction

Placement construction is the two-step defined sequence: alloc(layout) then ptr.write(value) (§5.2.7). Edition P1 has no dedicated placement syntax (settlement: `placement-in` syntax does not exist in P1; Volume 02 grammar has no production for it). Failure between the steps leaves the allocation live (caller deallocs or leaks, both defined).

### 5.11.9 Zero-sized allocations

alloc with size 0: no allocation is created; the result is Ok(ptr) with provenance (alloc_id = 1, offset = layout.align, caps {compute}); dereference → AddressTrap; dealloc with size 0 is a no-op (precondition: layout.size = 0). This makes ZST containers total and safe without heap churn (settled behavior).

### 5.11.10 Allocation lifetime

An allocation lives from alloc success to dealloc, realloc (as the old allocation), or program end (leak, defined §5.4.13). Heap storage is never reclaimed by scope exit; the destruction obligation of §5.3.13 is a type-level duty, not a storage-level one.

### 5.11.11 Foreign allocation

malloc/free and foreign allocator APIs produce foreign-class provenance (alloc_id = 2, offset = address, caps per the FFI declaration). Foreign allocations are outside HEAP_BUDGET accounting; dealloc must use the matching foreign free (calling Planck dealloc on foreign memory violates the dealloc precondition = UB-6; calling foreign free on Planck memory violates the foreign contract = UB-6 on the foreign-call precondition, Volume 10 §17.3).

### 5.11.12 Allocation ownership

Ownership duty per §5.3.13: the type system assigns each heap allocation an owner (Box/library types, Volume 19) whose drop glue deallocs with the recorded layout; unsafe manual code assumes the duty explicitly (documented per Volume 06 §9.8). Double ownership is prevented by the move discipline; violations in unsafe code manifest as UB-6 at the second dealloc.

---

## 5.12 Undefined behavior

### 5.12.1 The UB model (pinned)

The only undefined behavior categories are UB-1..UB-6 (SPEC-INDEX §3, exact):

| ID | Category | Exact trigger predicate |
| -- | -------- | ----------------------- |
| UB-1 | reading uninitialized memory through a typed place | a typed read (§5.9.1) whose read range includes bytes with init state ≠ I (§5.2.12); includes moves, drops of fields, and typed aggregate copies of such bytes |
| UB-2 | creating a reference outside its validity bounds | ¬valid_ref(r) at creation or use (§5.10.1 clauses (a)–(f)); includes dangling, out-of-range, misaligned, capability-lacking, and aliasing-contract-violating references |
| UB-3 | violating data-race freedom | two accesses (§5.13.9) to overlapping byte ranges on different threads, not ordered by happens-before, at least one a write, at least one non-atomic |
| UB-4 | calling a function through a wrong-ABI function pointer | indirect call where callee ABI ≠ pointer ABI or callee signature ≠ pointer signature (Volume 04 §4.1.16) |
| UB-5 | inline-assembly violating declared clobber/condition lists | the executed asm block writes a register/memory location not in its declared clobber list, or leaves declared condition outputs inconsistent with the declared set (Volume 10 §16.2) |
| UB-6 | violating an intrinsic's declared precondition | any intrinsic/API call whose Volume 10 §16.1-declared precondition fails; the preconditions exercised in this volume: dealloc (§5.11.6), realloc (§5.11.7), copy_nonoverlapping (§5.9.9), drop_in_place (§5.4.12), provider contract (§5.11.3), foreign free contract (§5.11.11), transmute layout compatibility (§5.5.12) |

Everything else is a defined value, a defined trap (T05-1, exit code 134), or a compile-time rejection. This catalog is closed: no additional UB category may be introduced without an edition change.

### 5.12.2 UB catalog (design.md §8.6 list, complete)

Each §8.6 bullet with its five documentation fields (triggering condition; static detectability; unsafe required; compiler assumptions permitted; diagnostic behavior):

| §8.6 item | Settlement | Triggering condition | Statically detectable | Unsafe needed | Compiler assumptions permitted | Diagnostic |
| --------- | ---------- | -------------------- | --------------------- | ------------- | ------------------------------ | ---------- |
| Reading uninitialized values | UB-1 | typed read of non-I bytes (§5.12.1) | partly: dataflow proves safe cases; dynamic cases are UB | no: safe code cannot (init tracking rejects); unsafe can | typed reads assume I (§5.10.9 item 5) | compile: E-P05-002; dynamic: not diagnosed (UB) |
| Invalid references | UB-2 | ¬valid_ref at creation/use | partly: borrow checker proves safe cases | yes for raw-to-ref formation | validity clauses (§5.10.9 items 1–3, 6) | compile: E-P04-013/E-P05-005 where visible; dynamic: not diagnosed (UB) |
| Invalid pointer dereferences | defined trap (AddressTrap/ProvenanceTrap) | raw deref with null/dead/out-of-range provenance | no (dynamic by nature) | yes | none beyond live-allocation assumption after checks are elided only with proof (OB-05-6) | runtime: trap, exit 134 |
| Misaligned accesses | AlignmentTrap (raw) / UB-2 (reference creation) | typed access with address mod align ≠ 0 | partly: layout known, address often dynamic | yes (safe code cannot form misaligned places) | alignment per layout (§5.10.9 item 6) | runtime: trap, exit 134 |
| Out-of-bounds access | BoundsTrap (indexing) / ProvenanceTrap (raw) / UB-2 (reference) | index ≥ len; provenance range exceeded | partly: constant indices proven | yes for raw paths | bounds after checks proven (OB-05-8) | runtime: trap, exit 134 |
| Invalid enum representations | defined ReprTrap | typed read of tag ∉ discriminant set | no | yes (safe construction cannot produce them) | tags valid for typed reads of safe-constructed values | runtime: trap, exit 134 |
| Invalid Boolean representations | defined ReprTrap | typed read of bool byte ∉ {0,1} | no | yes | bool bytes valid on safe paths | runtime: trap, exit 134 |
| Use-after-free | UB-2 (reference) / ProvenanceTrap (raw) | access after deallocation/frame exit | partly: borrow checker for safe code | yes for raw | liveness per §5.7 for checked paths | compile: E-P05-005/E-P04-013 where visible; raw: trap 134 |
| Double-free | UB-6 | dealloc on dead allocation (§5.11.6) | partly: ownership analysis for safe code | yes | single dealloc per allocation on checked paths | compile: E-P05-005 where visible; dynamic: UB |
| Violated aliasing rules | UB-2 | clause (e)/(f) violation making a reference invalid | no | yes (safe code prevented by borrow checker) | no-alias guarantees (§5.10.7–§5.10.9) | dynamic: UB; compile: borrow errors where visible |
| Data races | UB-3 | §5.13.9 predicate holds | no (dynamic) | no (safe concurrent code can race only via unsafe leaks; Volume 08 API prevents) | race-freedom on Send/Sync-typed programs (G2 statement) | dynamic: UB; static tools: Volume 08 §12 |
| Invalid function calls | CallTrap (null/invalid entry) | call through null or non-entry address | no | yes | entry validity after elision proofs only | runtime: trap, exit 134 |
| Invalid calling conventions | UB-4 | §5.12.1 UB-4 predicate | partly: typed pointers make safe code correct | yes (transmute to fn ptr) | ABI agreement on typed calls | compile: E-P04-024 (cast banned); dynamic: UB |
| Invalid inline assembly contracts | UB-5 | §5.12.1 UB-5 predicate | no | yes | declared clobber lists (Volume 16 verifier re-checks, OB-05-13) | runtime: UB; asm-side verification in Volume 10 |
| Invalid union access | defined values + ReprTrap | reinterpretation per Volume 04 §4.1.13 | no | yes (union reads are unsafe) | none beyond representation checks | runtime: ReprTrap, exit 134, where representation violated |
| Invalid destruction | UB-6 | drop_in_place precondition violated (§5.4.12) | partly: init dataflow for visible places | yes | glue runs on valid values only on checked paths | compile: E-P05-005 where visible; dynamic: UB |
| Invalid atomic operations | defined traps | misaligned/wrong-width/invalid-range atomic op (§5.9.13) | partly: width/align static | yes (atomics are unsafe-adjacent API) | widths/aligns per §5.9.13 on checked paths | runtime: AlignmentTrap/ProvenanceTrap, exit 134 |
| Invalid provenance operations | defined values + traps | int-to-ptr outside exposure; arithmetic past 2^64 marker (§5.8.8, §5.8.12) | no | yes (exposure paths are unsafe) | exposure-set consistency within an execution | runtime: AddressTrap/ProvenanceTrap on use, exit 134 |

Settlement rationale (non-normative, separated from rules): the pinned UB list UB-1..UB-6 exhausts undefined behavior; the design.md §8.6 items map onto those six categories, defined traps, or compile-time rejections, so every item of the design list remains defined — none dropped.

---

## 5.13 Memory ordering

### 5.13.1 Ordinary reads and writes

Within one thread, ordinary (non-atomic, non-volatile) accesses are sequenced by program order (the evaluation order of Volume 07 §11.1). Observable effect order (G1): I/O, volatile, atomic, and allocation effects are observed in program order relative to each other within a thread; the compiler must not reorder two externally visible effects of different kinds (obligation OB-05-14 row 1).

### 5.13.2 Compiler reordering

The compiler may reorder/eliminate ordinary accesses across each other subject to: data dependence, the alias rules (§5.10.5), init-state semantics (§5.2), drop order (§5.4.3, only with the no-observation proof), and the no-reorder envelope of §5.13.1. Reordering must not introduce a behavior outside the source's behavior set (G1; obligation OB-05-14).

### 5.13.3 Hardware reordering

Hardware reordering is defined by the target memory model (Volume 16 §23 per target). Source-level guarantees exist only through atomics and fences (§5.13.4–§5.13.6); a source program without them has no cross-thread ordinary-access guarantees beyond the data-race rule (UB-3 fires on races, so UB-free programs never rely on unspecified reorderings).

### 5.13.4 Atomic access

Atomic operations (load, store, swap, compare-exchange, fetch-op family; exact per-op semantics Volume 08 §12.4) carry an ordering ∈ {Relaxed, Acquire, Release, AcqRel, SeqCst} (closed enum). Widths {1,2,4,8,16} bytes; alignment = width (else AlignmentTrap). Atomicity: each atomic access is indivisible; per-location coherence: all threads observe the writes to one location in one total order consistent with happens-before.

### 5.13.5 Fences

core::sync::fence(o): Acquire fence: no later access in this thread may be reordered before it (compiler + hardware, per target model); Release fence: no earlier access may be reordered after it; SeqCst fence: both plus a single total order over all SeqCst fences and SeqCst operations. Fence semantics are the standard release/acquire synchronizes-with edges (Volume 08 §12.3 elaborates the formal hb construction).

### 5.13.6 Synchronization and thread communication

happens-before (hb): (1) program order within a thread; (2) release-write W synchronizes-with acquire-read R reading from W or a later release-chain write to the same location; (3) fence-based edges per §5.13.5; (4) thread spawn/join edges (spawn: everything before spawn hb everything in the child; join: everything in the child hb everything after join); (5) library synchronization (mutexes, channels) maps to (2)/(3) via internal atomics (Volume 08 §12.3, Volume 19). Everything hb a memory access is visible to it (reads see hb-latest write or a racing write only when atomic).

### 5.13.7 Volatile

Volatile operations (§5.9.12): not reordered relative to each other or to externally visible effects; not eliminated, duplicated, or commoned; no atomicity; no cross-thread hb edges. Volatile exists for Mmio and foreign contracts, not for synchronization (using volatile for synchronization is a program error, not UB; its semantics are simply the above, which do not provide sync).

### 5.13.8 Data-race definition

Data race (UB-3) exact predicate: two accesses a1, a2 such that: (1) both access at least one common byte; (2) at least one is a write; (3) they execute on different threads; (4) neither hb the other; (5) at least one is non-atomic. All five conjuncts are part of the predicate; atomic-atomic pairs never race (coherence per §5.13.4 governs them). Volatile accesses participate as non-atomic accesses (they race). Obligation OB-05-12 machine-checks the predicate on bounded access sets.

### 5.13.9 Memory consistency guarantees

T05-4 (exact per-ordering guarantee table):

| Ordering | Compiler guarantee | Hardware guarantee (target model, Vol 16) |
| -------- | ------------------ | ------------------------------------------ |
| Relaxed | atomicity + per-location coherence only | indivisible access; coherence order per location |
| Acquire | no later memory access in this thread reordered before the acquire load | load-acquire: subsequent accesses observe hb-prior writes |
| Release | no earlier memory access reordered after the release store | store-release: prior writes visible to acquire readers |
| AcqRel | both of the above for the read-modify-write op | both |
| SeqCst | single total order over all SeqCst ops + fences | total order respected by all cores (target-dependent mechanism) |

Guarantees compose with hb (§5.13.6). Programs relying on any guarantee outside this table are outside the semantics (their behavior is the UB-3 race if a race exists, else defined-but-unspecified interleave of §5.13.3).

---

## 5.14 Feature template applications

### 5.14.1 Move (design.md §40 16-row template)

| Specification requirement | What must be established |
| ------------------------- | ------------------------ |
| 1. Syntax | move occurs at: let binding, assignment, function argument pass, return, tuple/struct construction, match binding; no explicit `move` keyword in P1 (closures use `move` per Volume 02 §2.5) |
| 2. Static semantics | source init = I (E-P05-001 otherwise); borrow conflicts rejected (Volume 06 §9.3) |
| 3. Type semantics | applies to every Sized type; DSTs move as fat pointers (§5.3.8) |
| 4. Value semantics | destination receives bytes + provenance (§5.1.6 equivalence); source invalid (§5.3.6) |
| 5. Evaluation semantics | transfer per §5.3.5; no user code; effect row ∅ |
| 6. Ownership | duty transferred (§5.3.13); SPEC-INDEX §5.4 transitions MoveOut + Write |
| 7. Effects | ∅ (no alloc, no io, no panic) |
| 8. Control flow | no paths; panics during destination evaluation precede the transfer (construction-failure rule §5.4.3) |
| 9. Compile-time behavior | move elision legal per §5.3.12 identity rules |
| 10. Memory behavior | source storage dead for value purposes; pointers into source die (§5.8.15) |
| 11. ABI behavior | by-value arguments moved into callee frame per Volume 10 §17.1 |
| 12. IR 1 mapping | copy + kill of source provenance (Volume 13 §20 memory SSA) |
| 13. IR 2 mapping | elision/in-place construction (Volume 14 §21) |
| 14. Machine IR mapping | register/stack moves, no memcpy unless lowered (Volume 15 §22) |
| 15. Diagnostics | E-P05-001 (use of moved value), E-P05-005 (drop of moved place) |
| 16. Validation | OB-05-8 (EXH corpus covers move/init interleavings), OB-05-9 (drop order EXH) |

### 5.14.2 Slice indexing e[i] (design.md §40 16-row template)

| Specification requirement | What must be established |
| ------------------------- | ------------------------ |
| 1. Syntax | postfix [expr]; grammar Volume 02 §2.5 |
| 2. Static semantics | e: [T] or [T; N]; i: usize (Volume 04 §4.0.1 T-Index) |
| 3. Type semantics | result type T |
| 4. Value semantics | element at data + i × stride(T) (§5.1.1 value at that place) |
| 5. Evaluation semantics | bounds check i < len, else BoundsTrap; then place read (UB-1 rules apply to the element) |
| 6. Ownership | read of Copy element copies; non-Copy element read in value position is a move (§5.3.9) |
| 7. Effects | ∅ (the trap path aborts) |
| 8. Control flow | normal path; trap path exits 134 |
| 9. Compile-time behavior | check elision iff i ∈ [0, len) proven statically (obligation OB-05-8) |
| 10. Memory behavior | element range within slice provenance range (§5.9.6) |
| 11. ABI behavior | not applicable because indexing produces a place, not a call |
| 12. IR 1 mapping | place projection with check node (Volume 13 §20) |
| 13. IR 2 mapping | check hoisting/licm under UB-free proof (Volume 14 §21) |
| 14. Machine IR mapping | compare + conditional trap branch (Volume 15 §22) |
| 15. Diagnostics | runtime: BoundsTrap (exit 134); compile-time index-type errors: E-P04-001 |
| 16. Validation | OB-05-8 (EXH over generated index programs), OB-05-6 (trap emission completeness) |

---

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G |
| -- | ---------------- | ------ | ----- | -------- | - |
| OB-05-1 | Preservation: every abstract-machine step from a well-formed state M (every live pointer's provenance resolves to a live-or-dead allocation; init states consistent with events) reaches a well-formed state; trap and panic steps are the defined terminal transitions | REVIEW | FV-1 | §5.0.1 invariant statement; differential corpus tools/fv/mem/ | G1, G2 |
| OB-05-2 | Progress: every non-terminal well-formed machine state either takes a defined step, raises a defined panic (unwinding), or raises a defined trap (T05-1, exit 134); no undefined transition exists | REVIEW | FV-1 | §5.0.2 + §5.12.1 closure argument | G2 |
| OB-05-3 | Layout math: for all layouts generated over ≤ 6 fields with sizes ∈ [0, 2^6] and aligns ∈ {1,2,4,8,16,32,64} (domain ≤ 10^6), computed offsets/size/align match the §5.5.1 formulas and every field satisfies off mod align(field) = 0 | Z3 | FV-0 | tools/fv/layout.z3 | G2, G3 |
| OB-05-4 | align_up arithmetic is overflow-free and correct for x + a − 1 < 2^64 and a = 2^k, k ∈ [0, 29] (bit-vector proof) | Z3 | FV-0 | tools/fv/align-up.z3 | G2 |
| OB-05-5 | Niche soundness: hole counts (bool: 254; char: 2^32 − 1112064; pointer: 1) cover N ≤ 2^32 discriminants without overlap with valid representations | Z3 | FV-0 | tools/fv/niche.z3 | G3 |
| OB-05-6 | Trap completeness: every condition of T05-1 has an emitted check in the naive lowering; elision occurs only with a discharged proof (checked by trap-auditing pass over the test corpus) | EXH | FV-0 | tools/fv/trap-audit/ | G2, G22 |
| OB-05-7 | Initialization dataflow terminates on every CFG accepted by the frontend: measure Σ lattice heights decreases monotonically; bound 4 × \|P\| × \|E\| iterations | REVIEW | FV-1 | §5.2.12 (exact measure and monotonicity argument) | G11 |
| OB-05-8 | Init dataflow soundness: over all generated functions (≤ 6 blocks, ≤ 3 places, statement set {write, move, drop, if, loop} — domain ≤ 10^6), no accepted program reads a not-definitely-initialized typed place | EXH | FV-0 | tools/fv/init-flow/ | G2 |
| OB-05-9 | Drop order: for all generated drop scenarios (nesting depth ≤ 4, ≤ 6 objects, domain ≤ 10^6), destruction sequence equals T05-2 | EXH | FV-0 | tools/fv/drop-order/ | G1 |
| OB-05-10 | Allocation API contracts: for all (op, layout, ptr-state) combinations over Layout{size ∈ {0,1,8,2^48}, align ∈ {1,8,2^29}, states {fresh, live, dead, foreign}} (domain ≤ 10^6), outcomes match §5.11 exactly (values, AllocError variants, UB-6 flags) | EXH | FV-0 | tools/fv/alloc-api/ | G2 |
| OB-05-11 | Pointer arithmetic laws: narrowing monotonicity (arithmetic never widens the valid range), copy preservation (bit-exact provenance), saturating offset behavior at 2^64 — over bounded offset/stride domains | Z3 | FV-0 | tools/fv/provenance.z3 | G2, G14 |
| OB-05-12 | Data-race predicate (UB-3): five-conjunct predicate matches its definition on all access pairs over ≤ 3 threads × ≤ 4 accesses (domain ≤ 10^6) | Z3 | FV-0 | tools/fv/race.z3 | G21 |
| OB-05-13 | UB catalog closure: every operation of §5.9 and §5.11 has an outcome in {defined value, trap T05-1, UB-1..UB-6, compile-time rejection}; the mapping table §5.12.2 is complete (reviewer-verified enumeration) | REVIEW | FV-1 | §5.12.2 enumeration | G2, G16 |
| OB-05-14 | Effect ordering preservation: compilations do not reorder externally visible effects across kinds (§5.13.1); checked by differential observation on the concurrency corpus | PROP | FV-1 | tools/fv/order-diff/ (paired with REVIEW of the reorder envelope) | G1, G21 |
| OB-05-15 | Identity preservation: for every transformation applied in Volumes 13–15, the result set of each §5.6.12 identity-sensitive operation is unchanged (reviewer invariant named: observability closure over (alloc_id, offset) observations) | REVIEW | FV-1 | §5.6.12 invariant; per-pass obligations in Volumes 13–15 | G1 |
| OB-05-16 | Virtualization/scalar-replacement legality predicate (§5.6.8) is conservative: on all generated object graphs (≤ 8 objects, escape/projection patterns, domain ≤ 10^6), no legal-virtualization verdict coexists with an escape trigger | EXH | FV-0 | tools/fv/virt-legality/ | G1 |
| OB-05-17 | Object-merging predicate (§5.6.10) never merges objects distinguishable by §5.6.12 operations (same corpus generator as OB-05-16) | EXH | FV-0 | tools/fv/merge-legality/ | G1 |
| OB-05-18 | Trap uniformity: all traps terminate with exit code 134 in every build mode; abort-mode panics also exit 134 (checked over the T05-1 × {debug, release, panic=abort} grid) | EXH | FV-0 | tools/fv/trap-exit/ | G22 |

G-mappings used: G1 semantic preservation, G2 memory safety, G3 type soundness (layout), G11 termination, G14 determinism, G16 TCB (UB closure), G21 concurrency, G22 diagnostics. REVIEW rows name their exact invariant in the Property column; FV-1 gap = no machine check for the stated domain, bridged by the paired artifact listed.

---

## Coverage ledger

Ledger of design.md §7 (lines 525–630) and §8 (lines 631–764). One row per bullet.

### §7.1 Values and objects (13 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Definition of value | §5.1.1 |
| Definition of object | §5.1.2 |
| Semantic identity | §5.1.3 |
| Object identity | §5.1.4 |
| Storage identity | §5.1.5 |
| Value equivalence | §5.1.6 |
| Equality semantics | §5.1.7 |
| Address observability | §5.1.8 |
| Address stability | §5.1.9 |
| Object creation | §5.1.10 |
| Object destruction | §5.1.11 |
| Virtual objects | §5.1.12 |
| Materialized objects | §5.1.13 |

### §7.2 Initialization (13 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Uninitialized storage | §5.2.1 |
| Initialization | §5.2.2 |
| Full initialization | §5.2.3 |
| Partial initialization | §5.2.4 |
| Reinitialization | §5.2.5 |
| Initialization through references | §5.2.6 |
| Initialization through pointers | §5.2.7 |
| Initialization of arrays | §5.2.8 |
| Initialization of enums | §5.2.9 |
| Initialization of unions | §5.2.10 |
| Initialization of dynamically sized types | §5.2.11 |
| Initialization failure | §5.4.3 (construction-failure row); §5.2.8–§5.2.9 |
| Initialization tracking | §5.2.12 |

### §7.3 Copy and move (13 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Copy semantics | §5.3.1 |
| Copy capability | §5.3.2; Volume 04 §4.7 T04-9 |
| Implicit copying | §5.3.3 |
| Explicit cloning | §5.3.4 |
| Move semantics | §5.3.5; §5.14.1 |
| Move source invalidation | §5.3.6 |
| Partial moves | §5.3.7 |
| Aggregate moves | §5.3.8 |
| Array element moves | §5.3.9 |
| Moves through references | §5.3.10 |
| Moves involving pinned objects | §5.3.11 |
| Relocation | §5.3.12 |
| Ownership transfer | §5.3.13 |

### §7.4 Destruction (15 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Destructor invocation | §5.4.1 |
| Destructor eligibility | §5.4.2 |
| Drop order | §5.4.3 T05-2 |
| Field destruction | §5.4.4 |
| Partial destruction | §5.4.5 |
| Destructor effects | §5.4.6 |
| Destructor panic | §5.4.7 |
| Unwinding | §5.4.8 |
| Abort behavior | §5.4.9 |
| Double panic | §5.4.10 |
| Drop flags | §5.4.11 |
| Manual destruction | §5.4.12 |
| Explicit resource release | §5.4.13 |
| Deliberate leaking | §5.4.13 |
| Interaction with FFI | §5.4.14 |

### §7.5 Layout (16 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Size | §5.5.1 |
| Alignment | §5.5.1 |
| Field offsets | §5.5.1 |
| Padding | §5.5.1 |
| Enum discriminants | §5.5.2 |
| Niche representations | §5.5.3 |
| Zero-sized types | §5.5.4 |
| Aggregate packing | §5.5.5 |
| Explicit alignment | §5.5.6 |
| Packed structures | §5.5.5; §5.5.7 |
| Transparent structures | §5.5.8 |
| C layout | §5.5.9 |
| SIMD layout | §5.5.10 |
| Foreign layout | §5.5.11 |
| Layout reflection | §5.5.12; Volume 04 §4.10.4 |
| Layout compatibility | §5.5.12 |

### §7.6 Identity and optimization (12 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Identity preservation | §5.6.1; OB-05-15 |
| Address escape | §5.6.2 |
| Pointer identity | §5.6.3 |
| Pointer comparison | §5.6.4; §5.8.16 |
| Storage reuse | §5.6.5 |
| Storage lifetime | §5.6.6; §5.7 T05-3 |
| Object replacement | §5.6.7 |
| Aggregate virtualization | §5.6.8 |
| Scalar replacement | §5.6.9 |
| Object merging legality | §5.6.10 |
| Allocation elimination legality | §5.6.11 |
| Identity-sensitive operations | §5.6.12 |

### §8.1 Memory regions (9 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Stack storage | §5.7 T05-3 row Stack |
| Heap storage | §5.7 T05-3 row Heap |
| Static storage | §5.7 T05-3 row Static |
| Thread-local storage | §5.7 T05-3 row Thread-local |
| Foreign storage | §5.7 T05-3 row Foreign |
| Memory-mapped storage | §5.7 T05-3 row Memory-mapped |
| Allocator-managed storage | §5.7 T05-3 row Allocator-managed; §5.11.1 |
| Temporary storage | §5.7 T05-3 row Temporary |
| Storage lifetime | §5.7 T05-3 row Storage lifetime; §5.6.6 |

### §8.2 Pointers (19 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Pointer types | §5.8.1 |
| Pointer formation | §5.8.2 |
| Null pointers | §5.8.3 |
| Non-null pointers | §5.8.4 |
| Pointer provenance | §5.8.5 |
| Provenance creation | §5.8.6 |
| Provenance preservation | §5.8.7 |
| Provenance exposure | §5.8.7; §5.8.11 |
| Pointer arithmetic | §5.8.8 |
| Pointer subtraction | §5.8.9 |
| Pointer comparison | §5.8.10; §5.6.4 |
| Pointer-to-integer conversion | §5.8.11 |
| Integer-to-pointer conversion | §5.8.12 |
| One-past pointers | §5.8.13 |
| Dangling pointers | §5.8.14 |
| Pointer invalidation | §5.8.15 |
| Pointer equality | §5.8.16 |
| Pointer ordering | §5.8.17 |
| Pointer reconstruction | §5.8.18 |

### §8.3 Memory access (14 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Read access | §5.9.1 |
| Write access | §5.9.2 |
| Initialization requirements | §5.9.3 |
| Alignment requirements | §5.9.4 |
| Access width | §5.9.5 |
| Valid memory ranges | §5.9.6 |
| Mutability | §5.9.7 |
| Aliasing | §5.9.8 |
| Overlapping accesses | §5.9.9 |
| Invalid accesses | §5.9.10 |
| Unaligned access | §5.9.11 |
| Volatile access | §5.9.12 |
| Atomic access | §5.9.13 |
| Foreign memory access | §5.9.14 |

### §8.4 Aliasing model (10 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Reference aliasing | §5.10.1 |
| Raw pointer aliasing | §5.10.2 |
| Mutable alias restrictions | §5.10.1 clause (e); §5.10.3 |
| Shared alias rules | §5.10.1 clause (f); §5.10.3 |
| Interior mutability | §5.10.4 |
| Provenance-based alias analysis | §5.10.5 |
| No-alias guarantees | §5.10.7 |
| Alias invalidation | §5.10.6 |
| Unknown alias regions | §5.10.8 |
| Compiler alias assumptions | §5.10.9 |

### §8.5 Allocation (13 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Allocation APIs | §5.11.1 |
| Allocation layouts | §5.11.2 |
| Alignment | §5.11.3 |
| Allocator identity | §5.11.4 |
| Allocation failure | §5.11.5 |
| Allocation limits | §5.11.5 |
| Deallocation | §5.11.6 |
| Reallocation | §5.11.7 |
| Placement construction | §5.11.8 |
| Zero-sized allocations | §5.11.9 |
| Allocation lifetime | §5.11.10 |
| Foreign allocation | §5.11.11 |
| Allocation ownership | §5.11.12 |

### §8.6 Undefined behavior (18 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Reading uninitialized values | §5.12.2 row 1 (UB-1) |
| Invalid references | §5.12.2 row 2 (UB-2) |
| Invalid pointer dereferences | §5.12.2 row 3 (trap) |
| Misaligned accesses | §5.12.2 row 4 |
| Out-of-bounds access | §5.12.2 row 5 |
| Invalid enum representations | §5.12.2 row 6 (ReprTrap) |
| Invalid Boolean representations | §5.12.2 row 7 (ReprTrap) |
| Use-after-free | §5.12.2 row 8 |
| Double-free | §5.12.2 row 9 (UB-6) |
| Violated aliasing rules | §5.12.2 row 10 (UB-2) |
| Data races | §5.12.2 row 11 (UB-3); §5.13.8 |
| Invalid function calls | §5.12.2 row 12 (CallTrap) |
| Invalid calling conventions | §5.12.2 row 13 (UB-4) |
| Invalid inline assembly contracts | §5.12.2 row 14 (UB-5) |
| Invalid union access | §5.12.2 row 15 |
| Invalid destruction | §5.12.2 row 16 (UB-6) |
| Invalid atomic operations | §5.12.2 row 17 (traps) |
| Invalid provenance operations | §5.12.2 row 18 (values + traps) |

### §8.7 Memory ordering (10 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Ordinary reads and writes | §5.13.1 |
| Compiler reordering | §5.13.2 |
| Hardware reordering | §5.13.3 |
| Atomic access | §5.13.4 |
| Fences | §5.13.5 |
| Synchronization | §5.13.6 |
| Volatile | §5.13.7 |
| Thread communication | §5.13.6 |
| Data-race definition | §5.13.8 |
| Memory consistency guarantees | §5.13.9 T05-4 |

bullets in scope: 175, defined: 175.
