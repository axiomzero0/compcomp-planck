<!-- CEP:FILE: spec/volume-06-ownership.md
CEP:WHAT: Volume 06 of the Planck Master Specification Library: the ownership system and borrow checker — ownership kinds, place semantics, move analysis, borrow semantics, the lifetime/region system, the flow-sensitive borrow-checking algorithm, corner cases, unsafe integration, and pinning. Embeds the pinned ownership state machine of SPEC-INDEX §5.4 verbatim.
CEP:WHY: design.md §9 (lines 765–916) demands "both semantic rules and an explicit algorithm for enforcing them" and states that saying "use NLL" is not a complete borrow-checker design; this volume is the single normative definition home for all 117 bullets of design.md §9 and settles every open policy question they pose.
CEP:STATUS: complete
CEP:FAILURE: This volume must be rejected if any design.md §9 bullet lacks a coverage-ledger row, if the pinned transition table or conflict predicate of SPEC-INDEX §5.4 is altered, or if any rule here lacks an exact trigger, an exact code, or an exact constant.
CEP:ASSUMES: SPEC-INDEX §3 (UB-1..UB-6 model; destruction model: scoped, innermost-first, every path; identity = allocation ID + offset), §5.4 (pinned state machine, embedded verbatim in §6.1), Volume 05 (layout math, drop glue), Volume 07 (effect atoms used by unsafe rules), Volume 12 (limits table), standards.md §3 (normative verbs), verification standards.md G2, G15, G17, G18.
CEP:COST: compile-time analysis. Per-function borrow checking is a dataflow fixpoint bounded by B·(P+1) transfer-function applications, B = block count ≤ 65536, P = tracked place-path count ≤ 65536 (§6.7.1). No runtime cost in safe code; all ownership metadata is erased before IR 2.
CEP:EVIDENCE: verification matrix §6.12 (OB-06-1..OB-06-10); coverage ledger §6.13 (bullets in scope: 117, defined: 117).
-->

# Volume 06 — Ownership and Borrow Checker

## 6.0 Scope and traceability

This volume defines design.md §9 "Ownership system" (lines 765–916). It contains 9 subsections (§9.1–§9.9) comprising **117 bullets**, each of which receives exactly one normative definition below. Section mapping: §6.2 ↔ design.md §9.1, §6.3 ↔ §9.2, §6.4 ↔ §9.3, §6.5 ↔ §9.4, §6.6 ↔ §9.5, §6.7 ↔ §9.6, §6.8 ↔ §9.7, §6.9 ↔ §9.8, §6.10 ↔ §9.9. design.md §9's deliverable statement (lines 911–913) is addressed in §6.11.

Embedded pinned contract: SPEC-INDEX §5.4 is embedded verbatim in §6.1 and elaborated there and in §6.5–§6.7. Where this volume and SPEC-INDEX §5.4 could be read to disagree, §5.4 wins (SPEC-INDEX §0.5); the elaborations in §6.1.2–§6.1.4 are compatible readings, not modifications.

Normative verbs have the meaning of standards.md §3. All rejection codes defined here are of the form `P06-E<nnn>`; the set is closed (§6.7.5). All named constants have exact values (§6.7.6).

## 6.1 Pinned ownership state machine (SPEC-INDEX §5.4, verbatim)

### 6.1.1 Embedded contract

The following text is copied byte-for-byte from SPEC-INDEX §5.4 and is normative without modification:

- Place states: `Uninitialized`, `Owned`, `MovedFrom`, `Shared { loans: u16 }`, `MutableBorrowed`, `Frozen { loans: u16 }`.
- Actions on a place: `Read`, `Write`, `MoveOut`, `BorrowShared`, `BorrowMut`, `Drop`, `EndBorrow`.
- Transition table (total function `trans : State × Action → State ⊔ {Rejection}`) is exactly:

| State \ Action | Read | Write | MoveOut | BorrowShared | BorrowMut | Drop | EndBorrow |
| -------------- | ---- | ----- | ------- | ------------ | --------- | ---- | --------- |
| Uninitialized | Rej | Owned | Rej | Rej | Rej | Uninit | Rej |
| Owned | Owned | Owned | MovedFrom | Shared(1) | MutableBorrowed | Uninit | Rej |
| MovedFrom | Rej | Owned | Rej | Rej | Rej | Uninit | Rej |
| Shared(n) | Shared(n) | Rej | Rej | Shared(n+1 if n<65535 else Rej) | Rej | Rej | n≥1 → Shared(n−1); n=0 → Owned |
| MutableBorrowed | Rej | Rej | Rej | Rej | Rej | Rej | Owned |
| Frozen(n) | Frozen(n) | Rej | Rej | Frozen(n+1 if n<65535 else Rej) | Rej | Rej | n≥1 → Shared(n−1); n=0 → Owned |

- `Frozen` is the shared-borrowed state reached when a mutable borrow is requested on a place with outstanding shared loans; `Shared` and `Frozen` differ in that `Frozen` records that a mutable borrow was attempted. `EndBorrow` from a state with n = 0 returns the place to `Owned`.
- Conflict predicate: `conflict(a, b)` holds iff two actions on the same place cannot both be permitted in the same state; derived exactly from the rejection entries above.
- Laws: the table is total (every cell defined); EndBorrow never yields `Rej` from a borrowed state; `Shared(0)` and `Frozen(0)` normalize to `Owned` at EndBorrow. All EXH (6 states × 7 actions = 42 cells).

### 6.1.2 Rejection semantics

A rejected action (`trans` result `Rej`) must not change the state of the place. The state after a rejected action equals the state before it. A rejected action must produce exactly one diagnostic with a code from §6.7.5, determined by (state, action).

### 6.1.3 Derived predicates (exact, from the table only)

For an action `a`, `permits(a) = { s ∈ State : trans(s, a) ≠ Rej }`. The exact sets are:

| Action | permits(a) |
| ------ | ---------- |
| Read | {Owned, Shared(n) for all n, Frozen(n) for all n} |
| Write | {Uninitialized, Owned, MovedFrom} |
| MoveOut | {Owned} |
| BorrowShared | {Owned, Shared(n) for all n, Frozen(n) for all n} |
| BorrowMut | {Owned} |
| Drop | {Uninitialized, Owned, MovedFrom} |
| EndBorrow | {Shared(n) for all n, Frozen(n) for all n, MutableBorrowed} |

Two derived conflict notions, both derived exactly from the rejection entries:

1. `conflictAt(s, a) ⟺ trans(s, a) = Rej`. This is the per-state conflict used by the checker at each program point (§6.7).
2. `incompatible(a, b) ⟺ permits(a) ∩ permits(b) = ∅`. This is the action-pair incompatibility named by the pinned contract: two actions cannot both be permitted in the same state. The exact incompatible pairs over the 7 actions are: (Write, EndBorrow), (MoveOut, MoveOut), (MoveOut, BorrowMut), (MoveOut, EndBorrow), (BorrowMut, BorrowMut), (BorrowMut, EndBorrow), (Drop, EndBorrow). All other pairs are compatible by this definition. Both predicates are exhaustive-checked (OB-06-6).

### 6.1.4 Frozen: role and reachability (elaboration, not modification)

Per the pinned table, the only transitions into `Frozen(n)` are `BorrowShared` on a place already in `Frozen(k)`; `BorrowMut` on a place with outstanding shared loans is `Rej` in the table. The normative reading, consistent with both the table and the pinned note, is:

1. `Frozen(n)` is the analyzer's diagnostic-time representation of a shared-borrowed place against which a mutable borrow was attempted. When `BorrowMut` is rejected on `Shared(n)`, the diagnostic must report the place as `Frozen(n)` (code P06-E007, §6.7.5) and the persistent state must remain `Shared(n)` (§6.1.2).
2. `Frozen(n)` is a first-class state of the table: the analysis may hold a place in `Frozen(n)` whenever the loan graph records a mutable-borrow attempt against a shared-borrowed place. `Frozen(n)` and `Shared(n)` are behaviorally identical on every action: for all n and all a, `trans(Shared(n), a) = trans(Frozen(n), a)`. They differ only in recorded history (OB-06-5).
3. Reachability from `Owned` through permitted transitions covers `Uninitialized`, `Owned`, `MovedFrom`, `Shared(0..65535)`, `MutableBorrowed`; `Frozen(n)` is entered only through the diagnostic path of rule 1 or through `BorrowShared` on a `Frozen(k)` place.

### 6.1.5 State machine application domain

The table governs one machine per **tracked place** (§6.3). Places owned by foreign or manually-managed ownership classes (§6.2 d6, d7) are not tracked: no machine exists for them and every action on them is checked only by the unsafe rules (§6.9). The state of a composite place (struct, tuple, array) is tracked per leaf path; the machine runs per leaf path, not per allocation (§6.3.1).

## 6.2 Ownership model (design.md §9.1)

**d1 — Unique ownership.** Every tracked place has exactly one owner obligation at each program point, represented by state `Owned` (§6.1.1). The owner obligation must be discharged exactly once, by `Drop` (state → `Uninit`) or by `MoveOut` (state → `MovedFrom`, obligation transfers to the destination). Two `MoveOut` actions from the same place are `incompatible` (§6.1.3); the second must be rejected (P06-E005 or P06-E007 per state).

**d2 — Borrowed ownership.** Borrowed ownership is temporary access mediated by a loan (§6.5.3); the owner obligation is retained by the source place. A borrowed place must not be written, moved out of, or dropped while the loan is live: the pinned table rejects `Write`, `MoveOut`, `Drop` in `Shared(n)`, `Frozen(n)`, `MutableBorrowed`. Loans must terminate (§6.5.5) before the owner discharges its obligation.

**d3 — Shared ownership.** Shared ownership is reference counting over the types `Rc<T>` and `Arc<T>`. Exact contract: the control block holds `strong: u64` and `weak: u64`. `clone` on `Rc`/`Arc` must increment `strong`; increment when `strong = 2^64 − 1` is a defined trap, exit code 134. `Drop` of a shared owner decrements `strong`; when it reaches 0, the owned value's drop glue runs and `weak` is decremented; when `weak` reaches 0, the control block is deallocated. `Weak::upgrade` returns `Some(Rc/Arc)` iff `strong > 0`. The borrow checker does not track places behind `Rc`/`Arc` dereferences as loans of the allocation: each `Rc`/`Arc` handle is itself uniquely owned, and contents access goes through interior-mutability rules (§6.8 d15) or shared references with untracked loans (§6.5.14). Reference cycles produce leaks; a leak is defined behavior, must not be UB, and must not be a compile error; cycle reporting is tooling only (§6.9 d11).

**d4 — Static ownership.** A `static` item is owned by the program for the whole execution. Its address is stable for the entire execution. Statics must not run drop glue at any point: destruction at process exit is skipped (defined, not UB). Shared borrows of statics have region `'static` (§6.6 d1). `static mut` is Restricted: every read, write, or borrow of a `static mut` requires an enclosing `unsafe` block and carries the `unsafe` atom (Volume 07 §7.2); loans on `static mut` are tracked, and concurrent unsynchronized access is UB-3 (Volume 08 §8.1.6).

**d5 — Thread-local ownership.** A thread-local item (`thread_local! { static X: T = init; }`) is owned by the thread that initializes it. Initialization is lazy at first access per thread. Destruction must run at thread exit, in reverse order of initialization within that thread. Access from another thread is a compile-time rejection (P06-E016). Cross-thread sharing must go through `Send`/`Sync` (Volume 08 §8.3).

**d6 — Foreign ownership.** An allocation created outside the Planck allocator (via FFI) has allocation ID `0` (the reserved foreign ID) and is never owned by Planck. The Planck runtime must not deallocate it. Deriving a Planck reference from a foreign pointer is Restricted: it requires `unsafe` (§6.9) and produces an **untracked loan** (§6.5.14). Violating the validity of foreign data at dereference is UB-2, not a compile-time event.

**d7 — Manually managed ownership.** Allocation and deallocation through the unsafe intrinsics (alloc/dealloc, Volume 10) create allocations with fresh allocation IDs but no state machine (§6.1.5). The programmer owns the safety obligation; reading before initialization is UB-1, dereferencing outside validity is UB-2, double deallocation is UB-2 via provenance invalidation after the first dealloc.

**d8 — Ownership transfer.** Transfer is the `MoveOut` action (§6.1.1) applied at: by-value assignment (source `MoveOut`, destination `Write`), by-value function argument passing, `return` of a value, move bindings in patterns (§6.4 d10), and move-capture into closures (§6.4 d9). `Copy` types perform `Read`+`Write` instead of `MoveOut` and never enter `MovedFrom`.

**d9 — Ownership boundaries.** The checked boundaries are exactly: (1) function entry/exit — parameters arrive `Owned`, the return value leaves `Owned`, loans must not escape (§6.6 d10); (2) FFI boundary — no loan crosses in either direction, tracked places passed by value transfer ownership and are not dropped by the callee (P06-E017 at compile time if a tracked type crosses an `extern` boundary by reference or with drop glue reachable from the callee side); (3) async boundary — a future owns its captures by move; loans crossing an await point require the region system of §6.6 d12; (4) thread boundary — values crossing must satisfy `Send` (Volume 08 §8.3); (5) module/package boundary — public items with elided output lifetimes are rejected by §6.6 d2 rule 4.

**d10 — Ownership graphs.** The static ownership graph G = (V, E) has V = allocation IDs plus one root per ownership scope, and E = edges owner → owned established at construction or transfer. Invariants: (a) every node has at most one incoming owner edge from unique ownership at any program point; (b) transfer removes the old edge and adds the new one in one step, so cycles over unique edges are impossible by construction; (c) shared ownership (d3) contributes in-degree equal to the strong count; cycles through shared edges are permitted and leak (d3). The graph is a compile-time data structure; it must be acyclic except through shared edges. Leak reporting is a diagnostic under `plkc --warn-leaks`, FV-2 tooling; it must not gate compilation.

## 6.3 Place semantics (design.md §9.2)

The **place tree model**: the set of places of a function forms a forest. Each node is a `Place = (base: Base, path: Vec<Proj>)` with `|path| ≤ MAX_PLACE_DEPTH = 64`. Edges are projections; the root of each tree is a base place; the allocation ID of a place is inherited from its base. Every access in the source language maps to exactly one place.

**d1 — Place definition.** A place is the pair above; `Proj` is exactly the enum `Proj ∈ {Field(u16), Tuple(u8), ArrayIdx(u64 const), Index(place), Deref, Downcast(u16)}`. Two places are equal iff same base and same path. The leaf places of a type are computed from Volume 05 layout math; a place's address is `base_address + offset(path)`.

**d2 — Base places.** `Base ∈ {Local(id), Param(id), Temp(id), Static(path), ThreadLocal(path)}`. Each base names exactly one allocation: locals and temporaries get compiler-assigned IDs; statics and thread-locals are the global allocation of the named item. A `Deref` of a pointer at the head of a path is not a base: dereference is a projection (d7), and the base remains the place holding the pointer.

**d3 — Local places.** Locals are `let` bindings, pattern bindings, and compiler-introduced temporaries promoted to locals. Each local has one state machine (§6.1.5). Locals must be initialized before any action other than `Write` and `Drop` (§6.1.1 row `Uninitialized`).

**d4 — Field projections.** `Field(i)` projects the i-th declared field of a struct (index = declaration order, 0-based). Validity: i < field count; else compile-time rejection P06-E019. Overlap and offsets come from Volume 05 §7 layout math.

**d5 — Tuple projections.** `Tuple(i)` projects the i-th component (0-based). Same validity and overlap rules as d4.

**d6 — Array projections.** `ArrayIdx(k)` projects the k-th element of a fixed-size array `[T; N]`; validity: k < N, else P06-E019. `ArrayIdx` with compile-known k produces a disjoint leaf place per k.

**d7 — Dereference projections.** `Deref` projects through a reference, raw pointer, or smart pointer implementing `Deref`/`DerefMut`. For references and raw pointers the projected allocation is the pointee's allocation ID recorded in provenance (SPEC-INDEX §3); dereference of a raw pointer requires `unsafe` (§6.9 d2) and is checked against UB-2 at the provenance level. For `Deref`-implementing smart pointers the projection is transparent to the borrow checker: loans attach to the dereferenced place, guarded by the pointer's own state.

**d8 — Index projections.** `Index(p)` projects `base[p]` with p a runtime place. Bounds must be checked at runtime: `0 ≤ p < len` else the defined trap exit code 134 (Volume 07 §7.6). A loan taken through `Index` is an **unknown projection** (d13).

**d9 — Downcasts.** `Downcast(v)` projects the v-th variant of an enum after a discriminant test. It must be dominated by a test proving the discriminant equals v (the match compiler guarantees this, Volume 07 §7.8); the downcast itself carries no state change.

**d10 — Dynamic projections.** Dynamic projections are projections whose extent is not compile-known: `Deref` of an unsized fat pointer (slice or trait object) and `Index` on slices. Only these two forms exist. A loan through a dynamic projection must conservatively cover the whole unsized extent.

**d11 — Place overlap.** `overlap(p1, p2) ⟺ alloc(p1) = alloc(p2) ∧ [addr(p1), addr(p1)+size(p1)) ∩ [addr(p2), addr(p2)+size(p2)) ≠ ∅`, with addresses and sizes from Volume 05 layout math. Overlap is exact for compile-known paths.

**d12 — Place disjointness.** `disjoint(p1, p2) ⟺ ¬overlap(p1, p2)`. Disjoint leaf places may be borrowed, moved, and written independently (borrow splitting, §6.5 d8). A place is disjoint from every place with a different allocation ID, always.

**d13 — Unknown projections.** A projection is unknown iff it is `Index` with a non-constant index or a dynamic projection (d10). A loan L through an unknown projection of base B must be treated as overlapping every place under B. Consequences: while L is live, every conflicting action (per §6.1.3) on any place under B must be rejected (P06-E001..P06-E009 per the table row), and conversely any active loan on any place under B rejects a borrow conflicting with L.

## 6.4 Move analysis (design.md §9.3)

**d1 — Move creation.** A move is the `MoveOut` action on a leaf place (§6.1.1), generated by: by-value use in an expression, by-value argument, `return` of the value, by-value pattern binding, move capture (d9). A move of type `T` is a copy instead iff `T: Copy`; copies never enter `MovedFrom`.

**d2 — Move legality.** `MoveOut` is legal on place p iff `trans(state(p), MoveOut) ≠ Rej`, i.e. `state(p) = Owned`, and every ancestor place of p in the place tree is `Owned` or partially owned (d5), and p is not pinned-with-non-`Unpin` type (d12), and no live loan on p or an overlapping place exists (§6.7.4). Violations yield P06-E005 (borrowed), P06-E001 (moved), P06-E002 (uninitialized), P06-E010 (destructor, d11), P06-E012 (pinned, d12).

**d3 — Source invalidation.** After a legal `MoveOut`, the source place enters `MovedFrom`. Any action on a `MovedFrom` place other than `Write` and `Drop` must be rejected: `Read` → P06-E001, `BorrowShared`/`BorrowMut` → P06-E008, `MoveOut` → P06-E001 (second move).

**d4 — Destination initialization.** The destination of a move receives the `Write` action; on `Uninitialized` or `MovedFrom` this yields `Owned` (§6.1.1). Initialization is tracked per leaf path (d5).

**d5 — Partial moves.** A move of a proper subplace is a partial move of the parent. The parent is tracked as a partial map over leaf paths: `moved(path) ∈ {yes, no}` per leaf. Actions on the parent `Read`/`BorrowShared`/`BorrowMut` are permitted on a partially-moved parent only for subplaces with all-descendant-leaves unmoved; otherwise P06-E003. `Drop` of a partially-moved parent drops exactly the unmoved leaves (§6.7.6).

**d6 — Reinitialization.** `Write` to a `MovedFrom` or `Uninitialized` place restores `Owned` (§6.1.1). Reinitialization must re-establish the full leaf path: writing a partially-moved parent assigns the written subplace only; the partial map updates the corresponding leaves.

**d7 — Move through branches.** Moved-ness is a may-fact: at a CFG join, a place is moved iff it is moved on every incoming edge (§6.7.2). A use permitted only when the place is certainly not moved: `Read` on a place moved on some path to the join is P06-E001. Initialization is the dual must-fact: a place is certainly initialized iff initialized on every path.

**d8 — Move through loops.** Moves into, out of, and inside loops are resolved by the dataflow fixpoint of §6.7.2; the fixpoint is monotone over the finite fact lattice and must terminate in at most `B·(P+1)` transfer applications per fact family (termination obligation OB-06-8). A move out of a loop body makes the source `MovedFrom` at the loop header join.

**d9 — Move through closures.** Capture analysis (§6.7.7) decides, per captured place, the minimal capture mode: `Copy` if every use is a copy-read, `Move` if any use is a by-value use, `Shared` if only shared borrows, `Exclusive` if any exclusive borrow. A move capture transfers ownership at closure construction; the source is `MovedFrom` afterward (unless `Copy`). A closure body using a moved-out captured place is P06-E001 inside the closure body.

**d10 — Move through function calls.** By-value arguments are `MoveOut` at the call site (evaluation order per Volume 07 §7.10 d1); parameters arrive `Owned` at callee entry. The return value is moved into its destination at the caller. Calls never move places behind references: only the values named by the argument list.

**d11 — Move involving destructors.** A type whose drop glue is non-trivial (it or any component implements `Drop`, per Volume 05 §7) must not be partially moved: `MoveOut` of a proper subplace of a destructor-bearing type must be rejected with P06-E010. Whole-value moves of destructor-bearing types are permitted; the destination discharges the drop obligation.

**d12 — Move involving pinned data.** A place whose type is `!Unpin` and which is pinned (§6.10) must not be moved out of: `MoveOut` on it or on any subplace must be rejected with P06-E012, for as long as the pin is live. `Unpin` types are exempt. Moves into pinned storage before pinning are ordinary moves.

## 6.5 Borrow semantics (design.md §9.4)

**d1 — Shared references.** `&'a T` grants read-only access for region `'a`. Creation applies `BorrowShared` (§6.1.1). Content writes through `&T` are possible only via interior mutability (§6.8 d15); the checker models interior-mutability writes as writes to the interior-mutability subplace, which is not covered by shared loans of the parent (§6.8 d15).

**d2 — Exclusive references.** `&'a mut T` grants read-write access for region `'a`. Creation applies `BorrowMut`. While exclusive, every conflicting action on the place or an overlapping place must be rejected (§6.1.3 pairs with BorrowMut: MoveOut, BorrowMut, Write, EndBorrow-context Write/MoveOut/Drop per table rows `MutableBorrowed`).

**d3 — Loan creation.** A loan is the record `Loan = (id: LoanId, kind ∈ {Shared, Exclusive}, base: Place, path-prefix, region: RegionVar, phase)`. `BorrowShared` increments the loan counter (u16, saturating rejection at 65535, P06-E020); `BorrowMut` enters `MutableBorrowed` (no counter). Loan IDs are function-local, bounded by `MAX_LOANS_PER_FUNCTION = 65536`.

**d4 — Loan uses.** A use of loan L is any action through L's place path: reads, writes (exclusive only), moves out (exclusive only), and further borrows (d6). Uses must occur while L is live; a use at a program point outside L's region is P06-E008 or P06-E011 per context.

**d5 — Loan termination.** A loan terminates at the edge following its last use (edge-sensitive, §6.7.4 d5): the analyzer applies `EndBorrow` on that edge (counter decrement for Shared; `Owned` restoration for Exclusive). Termination must precede any conflicting action on the place; a conflicting action while any loan is live is rejected per the table (P06-E003..P06-E009).

**d6 — Reborrowing.** Borrowing through an exclusive loan L (`&mut` applied to `*r` where `r: &mut T`) creates a child loan L′ with parent L; L is blocked (not terminated) until L′ terminates. The place state during a child loan remains `MutableBorrowed` (the parent's state); the parent's `EndBorrow` must be applied only after all children terminated, in innermost-first order. Conflicts among sibling child loans follow the pinned table on disjoint paths (d8).

**d7 — Nested borrowing.** Borrowing through a shared loan creates a shared child; through an exclusive loan either kind. Depth is bounded by `MAX_PLACE_DEPTH = 64`; deeper nesting is P06-E019.

**d8 — Borrow splitting.** Disjoint subplaces of one owner may hold simultaneous distinct loans: `&mut p.f0` and `&mut p.f1` are both legal because the leaf places are disjoint (§6.3 d12) and the pinned table is applied per leaf path (§6.1.5). Splitting through an unknown projection is banned (§6.3 d13): all elements count as overlapping.

**d9 — Field-sensitive borrowing.** Loans are keyed by (base allocation, exact path prefix). Conflict checks are field-sensitive: a loan on `p.f0` rejects actions on `p.f0` and overlapping paths, and permits disjoint paths. Loans through unknown projections degrade to whole-base (§6.3 d13).

**d10 — Array borrowing.** `BorrowShared`/`BorrowMut` on an array place with compile-known index k loans exactly the leaf `base[k]`. On an unknown index or a whole-array borrow, the loan covers the entire array. `&mut a[i]` and `&mut a[j]` with distinct compile-known i ≠ j are disjoint and simultaneously legal.

**d11 — Slice borrowing.** Slices (`&[T]`, `&mut [T]`) are fat pointers; all element places are unknown projections of the slice extent. A loan on a slice or any element covers the whole extent. Splitting a `&mut [T]` requires explicit subslicing (`&mut s[a..b]`) with compile-known or runtime-checked disjoint ranges: two subslice loans must be checked for overlap at compile time when both ranges are compile-known; if either is runtime-dependent, the checker must reject simultaneous exclusive subslice loans (P06-E014) unless they are provably disjoint by the region system.

**d12 — Borrowing temporaries.** A temporary's default lifetime is the enclosing statement. A temporary bound by a `let` reference pattern or by `&` in a `let` initializer is extended to the enclosing block. Borrow of a temporary otherwise ends at the statement boundary; uses after it are P06-E011.

**d13 — Borrowing statics.** Shared borrows of `static` items are permitted with region `'static`. Borrows of `static mut` are Restricted to `unsafe` context (§6.2 d4); they are tracked loans on the static's leaf paths, and the programmer holds the thread-safety obligation (UB-3, Volume 08 §8.1.6).

**d14 — Borrowing foreign data.** Loans on places rooted in allocation ID 0 (foreign) are **untracked**: no state machine exists (§6.1.5), no conflict checking applies, and the region must be written by the programmer via the reference-formation intrinsic. The compile-time system enforces only that formation happens in `unsafe`. Dereference outside validity is UB-2.

**d15 — Escaping references.** A reference escapes iff its region must include a program point at which the loaned place's owner obligation is discharged or the owner's scope has exited. Escape is rejected with P06-E011 (return/assignment to longer-lived storage) or P06-E009 (loan outlives owner scope). The precise check is the region constraint system of §6.6.

## 6.6 Lifetime system (design.md §9.5)

**d1 — Lifetime regions.** A region `'r` is a set of CFG program points (§6.7.1). Regions are finite: `|regions('r)| ≤ MAX_REGION_POINTS = 65536` per function; the per-function region-variable budget is `MAX_REGIONS_PER_FUNCTION = 65536`. A loan's region is the set of points where the loan may still be used (§6.7.4).

**d2 — Lifetime inference.** Inference solves constraints over regions: each region variable starts at ∅ and accumulates the points where the corresponding reference is live. Constraints are (i) use constraints `p ∈ 'r` for each use of a reference with region 'r, (ii) outlives constraints `'r1: 'r2`, (iii) signature constraints from the callee's declared regions. The solver is a worklist fixpoint over the CFG: termination measure = total accumulated points, strictly increasing per successful propagation and bounded by `MAX_REGIONS_PER_FUNCTION × MAX_REGION_POINTS`; exceeding `MAX_REGION_INFERENCE_STEPS = 1000000` propagation steps is P06-E015.

**d3 — Lifetime annotations.** Syntax: `'a` on references (`&'a T`, `&'a mut T`) and on type parameters (`<T: 'a>`). Elision rules, exact: (1) each elided lifetime in parameter position is a distinct fresh parameter; (2) if exactly one input lifetime exists, it is assigned to every elided output lifetime; (3) if the method receiver is `&self` or `&mut self`, its lifetime is assigned to every elided output lifetime; (4) otherwise an elided output lifetime is a compile-time rejection P06-E018.

**d4 — Lifetime parameters.** `'a` may be a generic parameter of functions, structs, enums, impls. Every lifetime parameter must be used in at least one input position of the item's signature; an unused lifetime parameter is P06-E019.

**d5 — Lifetime bounds.** `'a: 'b` constrains `'a ⊆ 'b` as point sets (d8). Bounds may appear on generic parameters, where clauses, and impl headers.

**d6 — Lifetime subtyping.** `'a: 'b` is the subtyping relation on regions; a reference `&'a T` is a subtype of `&'b T` iff `'a: 'b` and T is covariantly compatible (d7). In requirement positions (function parameter of type `&'b T` receiving `&'a T`), `'a: 'b` must hold; violation is P06-E015.

**d7 — Lifetime variance.** Variance is computed structurally over types: `&'a T` is covariant in `'a` and in T; `&'a mut T` is invariant in `'a` and T; `fn(A) -> B` is contravariant in A and covariant in B; a struct field of type F inherits F's variance in each of F's parameters; interior-mutability wrappers (`Cell`, `UnsafeCell`) are invariant in their content parameter. Composition: Cov∘Cov = Cov; Cov∘Contra = Contra; Inv∘anything = Inv. Variance of raw pointers `*const T` = Cov in T, `*mut T` = Inv in T. The computed variance table must be part of the type-checking output and is checked by the region solver.

**d8 — Region constraints.** A constraint is one of `p ∈ 'r` (use), `'r1 ⊆ 'r2` (outlives), `'r = ∅` (dead). The solver (d2) computes the least solution; leastness guarantees loans end as early as possible (edge-sensitive termination, §6.7.4 d5).

**d9 — Lifetime containment.** `'a` is contained in `'b` iff `points('a) ⊆ points('b)`; this is the exact meaning of `'a: 'b`. Containment is a partial order (reflexive, antisymmetric, transitive); the transitivity is inherited from set inclusion and is a stated obligation (OB-06-9).

**d10 — Lifetime escape.** Escape is defined in §6.5 d15. The checker must reject escape statically; the guarantee is: no accepted program returns or stores a reference whose region includes a point where the referent is not valid (soundness statement OB-06-9).

**d11 — Return lifetime relationships.** For a function returning a reference, the returned region must be a subset of the union of the regions of the by-reference parameters (after elision, d3) or `'static`; a returned reference tied to no parameter region is rejected P06-E011. This is enforced as an inference constraint `out ⊆ p1 ∪ … ∪ pn`.

**d12 — Closure lifetime capture.** A closure capturing by reference with region `'r` requires `'r` to cover every point where the closure may be called, and the capture region must be a parameter of the closure's type (`for<'r>`-style higher-ranked or a named parameter). The minimal region is inferred (d2). Async closures follow d13.

**d13 — Async lifetime capture.** A borrow live across an await point (§6.7.4) must have its region include every poll site of the owning future (Volume 08 §8.7). The region system treats poll sites as ordinary CFG points of the enclosing function for the purpose of use constraints; escape across the future's drop is P06-E009.

**d14 — Lifetime termination.** All regions are finite point sets (d1); the solver terminates (d2, OB-06-8); at function exit every loan is required to have terminated: the exit point must not be in any loan's region, else P06-E009.

## 6.7 Borrow checker analysis (design.md §9.6)

### 6.7.1 Control-flow graph construction

The analysis operates on the mid-level IR of Volume 13 §20 (statement-level, place-based). The CFG has blocks `b_0..b_{B−1}`, B ≤ 65536 (Volume 12 limits), a single entry, and edges from terminator statements. Program points are (block, statement-index) pairs. Every construct of Volume 07 §7.7 lowers to this CFG before borrow checking; lowering is semantics-preserving (G1 obligation of Volume 13).

### 6.7.2 Dataflow facts (flow sensitivity)

The analysis is a forward may/must dataflow. The fact set at a program point q is `Facts(q) = (Init, Moved, Loans, Resv)` where:

- `Init` : leaf-path → may-initialized (join = union over predecessors).
- `Moved` : leaf-path → must-moved (join = intersection over predecessors).
- `Loans` : set of live loans, each `(LoanId, kind, base, path, region, parent)` (join = union).
- `Resv` : set of active two-phase reservations (§6.7.3).

Transfer functions per statement kind are total functions on this fact lattice; the lattice is finite (bounded by the limits of §6.7.6), monotone, and the fixpoint terminates in at most `B·(P+1)` applications per fact family where P = number of tracked leaf paths (≤ 65536) — obligation OB-06-8. At joins, `Init` and `Loans` and `Resv` use union, `Moved` uses intersection.

### 6.7.3 Loan generation and two-phase borrow math

A borrow statement generates a loan record (§6.5 d3) at its program point. For `BorrowMut` intended to be used as an operand of a call or an assignment receiver (syntactic two-phase positions, exactly: the receiver place of a method call and the destination place of `=`), the generation is two-phase:

1. **Reservation** at the borrow point: add `(LoanId, phase = Reserved)` to `Resv`; the place state is not changed (the pinned table's `BorrowMut` transition is not applied yet).
2. **Permitted during reservation** on the reserved place and any overlapping place: exactly `Read` and `BorrowShared` (and actions on disjoint places of other owners). Rejected during reservation on the reserved place or any overlapping place: `Write`, `MoveOut`, `BorrowMut`, `Drop` — code P06-E014.
3. **Activation** at the first use of the reserved loan: apply the pinned `BorrowMut` transition (`Owned → MutableBorrowed`); move `(LoanId, phase = Active)` from `Resv` to `Loans`.
4. **Expiry**: if the statement ends with no use, remove the reservation; no transition is applied.

The pinned table therefore remains the single authority for activated states; the reservation phase is analyzer-side state that only suppresses conflicts that the table would not yet see. The suppression set is exactly {Read, BorrowShared} — obligation OB-06-7. Rejection inside `unsafe` is severity "unsafe-notice" and must not fail compilation (§6.9 d9).

### 6.7.4 Loan liveness and edge-sensitive termination

A loan L is live at point q iff there is a CFG path from q to a use of L. The use set of L is computed in one backward pass per loan (bounded by the loan budget). Loan generation adds L to `Loans` from the borrow point until the last use; on the CFG edge out of the last use, the analyzer applies `EndBorrow` (edge-sensitive termination). EndBorrow on a shared loan decrements the counter per the pinned table; exclusive loans restore `Owned` only when no child loan (§6.5 d6) is live. Opaque callees interact with liveness exactly as specified in §6.7.7 (call-boundary handling).

### 6.7.5 Conflict detection, overlap, and diagnostics

At each point q where statement s performs action a on place p: compute every live loan L ∈ `Loans(q)` with `overlap(L.place, p)` (unknown projections per §6.3 d13); the effective state of p is the pinned-table state updated by L (a live exclusive loan forces `MutableBorrowed`, a live shared loan forces at least `Shared(n)`); then evaluate `trans(effectiveState(p), a)`. Rejection produces exactly one diagnostic from this closed table:

| Code | Category | Exact trigger (state, action) |
| ---- | -------- | ----------------------------- |
| P06-E001 | use-after-move | `Read`/`MoveOut`/`BorrowShared`/`BorrowMut` on `MovedFrom` |
| P06-E002 | use of uninitialized | any action but `Write`/`Drop` on `Uninitialized` |
| P06-E003 | write while shared | `Write` on `Shared(n)` or `Frozen(n)` |
| P06-E004 | write while exclusively borrowed | `Write` on `MutableBorrowed` |
| P06-E005 | move while borrowed | `MoveOut` on `Shared(n)`, `Frozen(n)`, or `MutableBorrowed` |
| P06-E006 | shared borrow while exclusive | `BorrowShared` on `MutableBorrowed` |
| P06-E007 | mutable borrow while shared | `BorrowMut` on `Shared(n)` or `Frozen(n)`; diagnostic reports the place as `Frozen(n)` (§6.1.4) |
| P06-E008 | borrow while moved or uninitialized | `BorrowShared`/`BorrowMut` on `MovedFrom` or `Uninitialized` |
| P06-E009 | loan outlives owner scope | exit-point region membership (§6.6 d14) or drop-with-live-loan (§6.7.6) |
| P06-E010 | partial move of destructor type | §6.4 d11 |
| P06-E011 | lifetime escape | §6.6 d10, d11, §6.5 d12 |
| P06-E012 | move of pinned data | §6.4 d12 |
| P06-E013 | closure capture conflict | §6.7.7: capture mode inconsistent with body uses |
| P06-E014 | reservation conflict | §6.7.3 step 2; also runtime-dependent disjoint subslices (§6.5 d11) |
| P06-E015 | region constraint failure | §6.6 d2 step budget exceeded or unsatisfiable outlives constraint |
| P06-E016 | thread-local cross-thread access | §6.2 d5 |
| P06-E017 | loan or tracked drop across FFI boundary | §6.2 d9 boundary 2 |
| P06-E018 | elision failure | §6.6 d3 rule 4 |
| P06-E019 | structural rejection | invalid projection index, depth exceedance, unused lifetime parameter |
| P06-E020 | shared-loan counter saturation | `BorrowShared` at n = 65535 (pinned table) |
| P06-E021 | unsafe call outside unsafe | call of an `unsafe fn`, `extern` function, or intrinsic requiring permission without an enclosing `unsafe` block (§6.9 d8) |

Every diagnostic must carry: the code, the place path, the state before the action, the conflicting loan ID and its creation point when applicable (G17 counterexample data).

### 6.7.6 Destruction analysis and drop interplay

Destruction follows SPEC-INDEX §3: scoped, innermost-first, every path. Per function, at each scope exit the analyzer inserts drop statements for locals of that scope, in reverse declaration order; temporaries of the scope follow in reverse creation order (§6.5 d12). A drop statement for leaf-path set S applies `Drop` to exactly the leaves that are may-initialized and not must-moved at the drop point (`Init` ∧ ¬`Moved` from §6.7.2); dropped leaves become `Uninit` per the pinned table. `Drop` on a place with any live loan is rejected by the pinned table (row `Shared`/`Frozen`/`MutableBorrowed`, column `Drop`) with P06-E009. Drop glue order within a value is Volume 05 §7 (struct's `Drop::drop` first, then fields in reverse declaration order). Drop of a pinned value runs at its pinned address (§6.10 d10).

### 6.7.7 Closures, calls, recursion

**Closures** (design.md "Closures", "Captured references"): capture analysis (§6.4 d9) runs before borrow checking; captures become implicit locals of the closure body checked with the same algorithm; capture conflicts with the enclosing body's actions on the same place produce P06-E013 at the capture point.

**Call-boundary handling**: for a callee visible to the analysis, loans on argument places must end before the call (by-value arguments move them; by-reference arguments constrain the callee's declared parameter regions via §6.6 d2 signature constraints). For an opaque callee (foreign function, function pointer, trait-object dispatch), the exact rule is: exclusive loans on places reachable from the arguments must be terminated before the call (P06-E005 otherwise); shared loans may survive only if the argument's provenance capability set excludes `store`; otherwise they must also terminate. Shared loans on places not reachable from the arguments always survive.

**Recursive function handling**: the analysis is intraprocedural. For a recursive function f, each parameter reference region is abstract (a fresh region variable) and the body is checked once against the signature constraints; recursion adds no iteration to the fixpoint. Mutual recursion is the same, checked per function against declared signatures.

**Escape analysis** (design.md "Escape analysis"): the escape check is the region solver of §6.6 d2 applied to reference-valued flows: every store of a reference into a place with region `'p` generates `'r ⊆ 'p`; every call passing a reference generates the signature constraint. "Escape analysis" in the optimizer sense (allocation removal) is Volume 14's pass and must not affect borrow-checking results.

**Loop handling**: fixpoint iteration (§6.7.2); loans live around a back-edge are killed only at their last use, which may be in a later iteration — the fixpoint computes exactly this.

### 6.7.8 Initialization dataflow, move dataflow

`Init`/`Moved` are as in §6.7.2. A `Read`/`Borrow*`/`MoveOut` on a leaf not certainly initialized is P06-E002 (or P06-E001 if must-moved). Conditional initialization (§6.8 d3) is exactly the must-fact at joins; partial initialization (§6.8 d1) is the per-leaf map. This pair of facts is the complete answer to design.md bullets "Initialization dataflow" and "Move dataflow".

## 6.8 Borrow checking corner cases (design.md §9.7)

**d1 — Partial initialization.** Initialization is per leaf path (§6.7.2 `Init`); struct literals initialize leaves in declaration order; a read of any not-certainly-initialized leaf is P06-E002 with the leaf named.

**d2 — Partial moves.** As §6.4 d5: permitted on destructor-free, unpinned types; per-leaf tracking; parents report the moved leaves in diagnostics.

**d3 — Conditional initialization.** `if` branches initializing different leaves join by intersection (must); the drop analysis of §6.7.6 drops exactly may-initialized ∧ ¬must-moved leaves, so conditional paths never double-drop and never leak.

**d4 — Loops with moves.** Fixpoint (§6.7.2): moving out inside a loop invalidates the source for subsequent iterations unless reassigned on every path into the next iteration; the must-facts encode this exactly.

**d5 — Multiple exits.** Scopes with `return`, `break`, `?`, or panic paths insert drop statements on every exit edge (§6.7.6); every exit receives the same destruction treatment; no exit may skip drops of may-initialized locals.

**d6 — Reborrowing.** As §6.5 d6: parent blocked, children checked independently on disjoint paths, innermost-first termination order.

**d7 — Closures.** As §6.7.7: capture modes minimal per place; conflicts at capture points (P06-E013).

**d8 — Captured references.** A closure capturing `&'r T` requires `'r` as a closure-type parameter covering all call sites (§6.6 d12); the captured reference is an ordinary local of the closure body.

**d9 — Recursive aggregates.** Types containing themselves through pointers (`Rc`, boxes) are checked per-component: each pointer dereference introduces a fresh untracked or tracked loan at the dereference site; recursion in types never recurses in the checker (no type-level cycles are dereferenced implicitly).

**d10 — Mutable slices.** As §6.5 d11: extent-wide loans; subslice splitting requires compile-known disjointness or rejection P06-E014.

**d11 — Self-referential structures.** A type holding a reference into itself cannot be constructed in safe code: safe construction requires an interior-mutability or raw-pointer encoding (§6.8 d15, §6.9); the checker needs no special rule because the construction path is unsafe or address-independent by construction. Pinning (§6.10) is the sanctioned mechanism for address-dependent data.

**d12 — Pinning.** §6.10; the interaction with the checker is: `MoveOut`/`DerefMut`-derived `Write` on `!Unpin` pinned places rejected (P06-E012) while pinned.

**d13 — Async suspension.** Loans live across await points span the poll sites (§6.6 d13); the state machine's drop paths (Volume 08 §8.7) are generated from the same `Init` facts (§6.7.2), so the async drop paths cannot double-drop or leak: this is obligation OB-08-7 in Volume 08, fed by this volume's facts.

**d14 — Foreign pointers.** §6.5 d14, §6.2 d6: untracked, unsafe-only formation, UB-2 at misuse.

**d15 — Interior mutability.** `Cell<T>`, `UnsafeCell<T>` (and wrappers built on them) exempt their contents from loan tracking: borrows of a `Cell` place are loans on the cell, not the content; the cell's safe methods implement content access through `unsafe` internals whose obligation is the type invariant (documented per impl, §6.9 d10). The checker must not generate loans on the content of an interior-mutability place, and shared loans of the parent do not conflict with interior-mutability writes.

## 6.9 Unsafe integration (design.md §9.8)

**d1 — Unsafe authority.** `unsafe` is a permission, not a mode change: it releases exactly the checks enumerated in this volume and Volume 07 §7.2, and nothing else. Type checking, region well-formedness of annotated lifetimes, and effect checking still apply inside `unsafe`. The `unsafe` atom (Volume 07 §7.2) marks operations requiring permission: raw-pointer dereference, `unsafe fn` call, `extern` call, `static mut` access, union field access, reference formation from raw pointers.

**d2 — Raw-pointer access.** `*const T`/`*mut T` dereference requires `unsafe`. Dereference checks provenance (SPEC-INDEX §3): allocation ID valid, capability `load`/`store` present, offset within allocation bounds; violation is UB-2. Pointer arithmetic narrows capabilities (SPEC-INDEX §3); out-of-bounds arithmetic itself is defined (a pointer value), only dereference is checked.

**d3 — Manual initialization.** `ptr::write`-equivalent intrinsics initialize a place without reading it; after a manual write, the place is `Owned` (the analyzer models the intrinsic as `Write`). Reading before initialization is UB-1.

**d4 — Manual destruction.** `ptr::drop_in_place`-equivalent runs the drop glue without deallocating; the analyzer models it as `Drop` (state → `Uninit`). Running it twice on the same place without reinitialization is UB-2 (provenance invalidated) and must not be reachable in accepted safe programs.

**d5 — Manual allocation.** The alloc intrinsic returns a pointer with fresh allocation ID and capability set `{load, store, compute}`; layout must have `size > 0` for the non-zero-sized entry points; zero-sized allocations return a dangling-but-valid-for-ZST provenance (exact: allocation ID 0, offset 0, capabilities `{load, store, compute}`, size 0 — no dereference beyond size 0 is possible).

**d6 — Manual deallocation.** The dealloc intrinsic requires: allocation ID valid, currently allocated, offset 0, capability `store` on the provenance. Violation is UB-2. After deallocation the allocation ID is invalid; all pointers into it lose validity (SPEC-INDEX §3 identity rule).

**d7 — Unsafe trait obligations.** An `unsafe impl` must document its safety invariant in the item's CEP block (standards.md §25.3.1 discipline); the invariant is a named precondition on every public method. The compiler does not verify invariants; the obligation is the reviewer's (OB-06-10 covers the diagnostic side).

**d8 — Unsafe function contracts.** An `unsafe fn` must state its safety preconditions in its documentation; calling it from safe code outside `unsafe` is a compile-time rejection with exact code P06-E021 (§6.7.5). Inside `unsafe`, the caller takes over the preconditions.

**d9 — Borrow checker bypass boundaries.** Inside `unsafe`, exactly the state-machine conflict checks are downgraded: diagnostics P06-E001..P06-E009 and P06-E014 raised inside an `unsafe` block are emitted with severity "unsafe-notice" and must not fail compilation; static rejections P06-E010, P06-E011, P06-E012, P06-E013, P06-E015..P06-E021 keep compile-time rejection severity inside `unsafe`, because type structure, regions, and pinning obligations are not released by the permission. The downgraded checks exist to protect the *enclosing safe* abstractions. A safe abstraction must not expose behavior violating the checked rules on any input (G2 obligation OB-06-9).

**d10 — Safety invariants.** Every public unsafe-reaching abstraction must carry an invariant statement of the form: preconditions, preserved properties, and the UB categories (from SPEC-INDEX §3) that violation would trigger. This is a documentation norm enforced by review; the machine-checkable part is the safe surface's behavior (OB-06-9).

**d11 — Unsafe diagnostics and auditing.** `plkc --audit-unsafe` must emit, per crate: count and locations of unsafe blocks, unsafe fns, unsafe impls, and the P06-E-code notices raised inside them. Output is deterministic (G14). Auditing is FV-2 tooling; it must not change semantics.

## 6.10 Pinning (design.md §9.9)

**d1 — Pin semantics.** `Pin<P>` wraps a pointer P and asserts: the value behind P will not move (its address will not change) until the pin's region ends. Pinning is an obligation on all code, enforced on safe paths by the rules of this section; violation through unsafe is UB-2-category behavior defined per contract (d11).

**d2 — Pinned storage.** Sanctioned storage: `Box::pin(v)` (heap, address stable for the box's life), `pin!(e)` (stack pin for the remainder of the enclosing block; exact: introduces a local, moves e into it, returns `Pin<&mut T>` with region = block remainder), and `Rc::pin`/`Arc::pin` (shared, d3 of §6.2). Any other creation of a `Pin` over a non-`Unpin` value requires `unsafe` (d11).

**d3 — `Unpin`.** `Unpin` is an auto trait: implemented for every type whose fields are all `Unpin`, except types that opt out (`PhantomPinned` or a manually declared opt-out). `T: Unpin` means moves of T cannot invalidate self-references because T may not contain address-dependent data. `Pin::new` on `&mut T` requires `T: Unpin`; `Pin::new_unchecked` requires `unsafe` (d11).

**d4 — Address stability.** After a value is pinned at address A, every safe access must go through `Pin`-projected pointers (d7). Safe code must not obtain `&mut T` (non-`Unpin`) to a pinned value: `DerefMut` for `Pin<P>` is implemented only when `P::Target: Unpin`. A move of the *container* (e.g. the `Box`) does not move the value: heap containers satisfy stability by indirection.

**d5 — Pinned references.** `Pin<&'a T>` and `Pin<&'a mut T>` are references carrying the pin obligation for region `'a`. Their borrow interactions are ordinary loans (§6.5) on the pinned place; additionally the checker must reject `MoveOut` and `Write`-via-`DerefMut` on `!Unpin` pinned places (§6.4 d12, P06-E012).

**d6 — Pinned ownership.** The owner of the storage (box, stack local, shared handle) discharges the drop obligation; the `Pin` is a view, not an owner. Dropping the storage while a `Pin` view exists is prevented by ordinary loan rules (§6.7.5: `Drop` with live loans is rejected).

**d7 — Pin projection.** A projection function `fn project(self: Pin<&mut Self>) -> Pin<&mut F>` for field F is a safe abstraction iff F is structurally pinned, i.e., F's type may be `!Unpin` and the projection preserves the no-move obligation. Structurally-pinned fields must be declared in the type's documented invariant (§6.9 d10); non-structural projections must return `&mut F` and require `F: Unpin` or `unsafe`. The compiler provides `#[pin(project)]`-style derived projections exactly for structurally-pinned field lists; misuse (projecting a non-listed field as pinned) is P06-E019.

**d8 — Self-referential objects.** Constructible only as `!Unpin` types initialized before pinning, with all self-references formed after the value's final address is fixed (inside methods taking `Pin<&mut Self>`). The checker enforces the safe subset: no safe code may form an internal reference before the address is stable; forming it inside `Pin<&mut Self>` methods is permitted because the borrow of the pin keeps the address stable for the reference's region.

**d9 — Async state machines.** Generated future types (Volume 08 §8.7) are `!Unpin` iff they hold any `!Unpin` value or any reference live across an await; otherwise `Unpin`. They must be pinned before polling; executors must poll at a stable address (Volume 08 §8.8 obligation).

**d10 — Destruction of pinned values.** A pinned value is dropped through its storage: `Box::pin` drops in place at deallocation; stack pins drop at block exit in §6.7.6 order; shared pins drop per §6.2 d3 counters. Drop glue must run at the pinned address; moving a pinned value's bytes before drop is banned on safe paths and UB-2-category under unsafe violation (d11).

**d11 — Unsafe pinning contracts.** `Pin::new_unchecked` and raw-pointer pin intrinsics carry the exact obligation: the caller must guarantee no safe or unsafe code moves the value before the pin's region ends. Violation is UB-2 (reference outside validity bounds). The obligation is a named safety invariant per §6.9 d10.

## 6.11 Deliverable statement

design.md lines 911–913 require "both semantic rules and an explicit algorithm for enforcing them". The semantic rules are §6.1–§6.6 and §6.8–§6.10; the explicit algorithm is §6.7 (CFG construction, fact lattice with exact transfer functions, fixpoint with stated termination bound, loan generation with two-phase math, edge-sensitive termination, closed diagnostic table). No step of the algorithm is delegated to an external checker or named after an existing implementation.

## 6.12 Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-06-1 | Transition-table totality: all 6×7 = 42 cells of §6.1.1 yield a defined result in `State ⊔ {Rejection}`; no cell undefined | EXH (domain: 42 cells) | FV-0 | `math/crates/planck-ownership-math` (in-crate `#[cfg(test)]`: `exh_trans_pinned_cells_42_families`, `exh_trans_full_domain_131076x7`); `tools/fv/matp/problems/own_table_cells.p` + artifact `tools/fv/matp/artifacts/own_table_cells.result` | G15, G2 |
| OB-06-2 | EndBorrow never yields `Rej` from a borrowed state; borrowed states are exactly {Shared(n) : n ≥ 0, n ≤ 65535, Frozen(n) : same, MutableBorrowed} | EXH (domain: 3 borrowed state families × EndBorrow, 65536 + 65536 + 1 cases sampled at boundaries {0, 1, 65534, 65535} plus full u16 sweep 0..=65535 = 131072 cases) | FV-0 | same crate (in-crate `#[cfg(test)]`: `exh_endborrow_law_full_u16_sweep`, `exh_loan_saturation_boundaries_65534_65535`); MATP attempts on the same law: `tools/fv/matp/problems/own_endborrow_zero.p` (Theorem, pass) and `own_endborrow_borrowed_sh.p`/`own_endborrow_borrowed_mut.p`/`own_endborrow_borrowed_fr.p` (Saturated without refutation within the bound — recorded honestly as fail; discharged by the crate's full-domain EXH per SPEC-INDEX §4) | G15 |
| OB-06-3 | Normalization: `EndBorrow` from `Shared(0)` and `Frozen(0)` yields `Owned` | EXH (domain: 2 cells) | FV-0 | same crate | G15 |
| OB-06-4 | Counter law: for all n ∈ 0..=65535, `trans(Shared(n), BorrowShared)` is `Shared(n+1)` iff n < 65535, else `Rej`; same for `Frozen`; no transition produces a counter outside 0..=65535 | EXH (domain: 2 × 65536 cases) | FV-0 | same crate | G15, G13 |
| OB-06-5 | Behavioral identity of Shared and Frozen: for all n ∈ 0..=65535 and all 7 actions, `trans(Shared(n), a) = trans(Frozen(n), a)` | EXH (domain: 65536 × 7 × 2 = 917504 cases) | FV-0 | same crate | G15 |
| OB-06-6 | Conflict derivation: `conflictAt(s, a) ⟺ trans(s, a) = Rej`; `incompatible(a, b) ⟺ permits(a) ∩ permits(b) = ∅`; the incompatible-pair list of §6.1.3 is exactly the set of pairs with disjoint permit sets | EXH (domain: 6×7 = 42 conflictAt cells; 7×7 = 49 incompatibility pairs) | FV-0 | same crate | G15 |
| OB-06-7 | Two-phase suppression set is exactly {Read, BorrowShared}: for every (state, action) pair, the reservation phase rejects a iff a ∉ {Read, BorrowShared} and the place overlaps the reservation | EXH (domain: 6 states × 7 actions = 42) | FV-0 | `planck-ownership-math` in-crate `#[cfg(test)]` (reservation-phase EXH over the 42 cells; the two-phase checker is checker-side per the pinned §5.4 scope "state machine and conflict predicate", so the crate pins the action set and the checker consumes it) | G15 |
| OB-06-8 | Dataflow termination: the fact lattice of §6.7.2 is finite and every transfer function is monotone; the fixpoint reaches stability in ≤ B·(P+1) applications per fact family with B ≤ 65536, P ≤ 65536 | REVIEW (invariant named: monotone transfer over finite lattice of height ≤ 3·P per family; measure = accumulated fact count, strictly increasing, bounded) | FV-1 | in-crate PROP property tests (paired with EXH over generated ≤ 6-block CFGs per SPEC-INDEX §4.3); the monotonicity argument is reviewer-checkable and stated here in the row + `planck-ownership-math` monotonicity property tests (PROP, paired with EXH over generated ≤ 6-block CFGs) | G11, G15 |
| OB-06-9 | Borrow-checking soundness: every program accepted by the checker of §6.7 satisfies, on all executions and inputs: no UB-1 through a typed place, no UB-2 through a reference or loan formed by safe code, no double drop of any leaf, and no data race through safe shared mutable access (G2 items 2, 3, 5, 8, 9) | REVIEW (invariants named: (i) every accepted use point has the place effectively Owned per §6.7.5; (ii) loans are live only within their regions (§6.6 d2 least solution); (iii) drop applies only to may-init ∧ ¬must-moved leaves (§6.7.6); (iv) exclusive-loan exclusivity plus Send/Sync (Volume 08 §8.3) excludes UB-3; reason not machine-checkable: soundness is over all executions of all programs, not a finite domain) | FV-1 | this row's named invariants are the artifact (REVIEW method); runtime bridge: debug-mode drop-flag assertions (Volume 18) trap on double-drop (exit 134) | G2, G3 |
| OB-06-10 | Diagnostics completeness: every rejection cell reachable in §6.7.5 maps to exactly one code of the closed table P06-E001..P06-E021, and every code has a triggering (state, action) or exact static condition | EXH (domain: 42 cells × diagnostic mapping; 21 codes) | FV-0 | `planck-ownership-math` in-crate `#[cfg(test)]` diagnostics table EXH (rejection cells × closed code table P06-E001..P06-E021) | G17, G22 |

## 6.13 Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| ------------------------------ | ---------- |
| 9.1 Unique ownership | §6.2 d1 |
| 9.1 Borrowed ownership | §6.2 d2 |
| 9.1 Shared ownership | §6.2 d3 |
| 9.1 Static ownership | §6.2 d4 |
| 9.1 Thread-local ownership | §6.2 d5 |
| 9.1 Foreign ownership | §6.2 d6 |
| 9.1 Manually managed ownership | §6.2 d7 |
| 9.1 Ownership transfer | §6.2 d8 |
| 9.1 Ownership boundaries | §6.2 d9 |
| 9.1 Ownership graphs | §6.2 d10 |
| 9.2 Place definition | §6.3 intro, d1 |
| 9.2 Base places | §6.3 d2 |
| 9.2 Local places | §6.3 d3 |
| 9.2 Field projections | §6.3 d4 |
| 9.2 Tuple projections | §6.3 d5 |
| 9.2 Array projections | §6.3 d6 |
| 9.2 Index projections | §6.3 d8 |
| 9.2 Dereference projections | §6.3 d7 |
| 9.2 Downcasts | §6.3 d9 |
| 9.2 Dynamic projections | §6.3 d10 |
| 9.2 Place overlap | §6.3 d11 |
| 9.2 Place disjointness | §6.3 d12 |
| 9.2 Unknown projections | §6.3 d13 |
| 9.3 Move creation | §6.4 d1 |
| 9.3 Move legality | §6.4 d2 |
| 9.3 Source invalidation | §6.4 d3 |
| 9.3 Destination initialization | §6.4 d4 |
| 9.3 Partial moves | §6.4 d5 |
| 9.3 Reinitialization | §6.4 d6 |
| 9.3 Move through branches | §6.4 d7 |
| 9.3 Move through loops | §6.4 d8 |
| 9.3 Move through closures | §6.4 d9 |
| 9.3 Move through function calls | §6.4 d10 |
| 9.3 Move through pattern matching | §6.4 d1 (by-value binding), §6.7.7 |
| 9.3 Move involving destructors | §6.4 d11 |
| 9.3 Move involving pinned data | §6.4 d12 |
| 9.4 Shared references | §6.5 d1 |
| 9.4 Exclusive references | §6.5 d2 |
| 9.4 Loan creation | §6.5 d3 |
| 9.4 Loan uses | §6.5 d4 |
| 9.4 Loan termination | §6.5 d5, §6.7.4 |
| 9.4 Reborrowing | §6.5 d6 |
| 9.4 Nested borrowing | §6.5 d7 |
| 9.4 Borrow splitting | §6.5 d8 |
| 9.4 Field-sensitive borrowing | §6.5 d9 |
| 9.4 Array borrowing | §6.5 d10 |
| 9.4 Slice borrowing | §6.5 d11 |
| 9.4 Borrowing temporaries | §6.5 d12 |
| 9.4 Borrowing statics | §6.5 d13 |
| 9.4 Borrowing foreign data | §6.5 d14 |
| 9.4 Escaping references | §6.5 d15 |
| 9.5 Lifetime regions | §6.6 d1 |
| 9.5 Lifetime inference | §6.6 d2 |
| 9.5 Lifetime annotations | §6.6 d3 |
| 9.5 Lifetime parameters | §6.6 d4 |
| 9.5 Lifetime bounds | §6.6 d5 |
| 9.5 Lifetime subtyping | §6.6 d6 |
| 9.5 Lifetime variance | §6.6 d7 |
| 9.5 Region constraints | §6.6 d8 |
| 9.5 Lifetime containment | §6.6 d9 |
| 9.5 Lifetime escape | §6.6 d10 |
| 9.5 Return lifetime relationships | §6.6 d11 |
| 9.5 Closure lifetime capture | §6.6 d12 |
| 9.5 Async lifetime capture | §6.6 d13 |
| 9.5 Lifetime termination | §6.6 d14 |
| 9.6 Control-flow graph construction | §6.7.1 |
| 9.6 Initialization dataflow | §6.7.2, §6.7.8 |
| 9.6 Move dataflow | §6.7.2, §6.7.8 |
| 9.6 Loan generation | §6.7.3 |
| 9.6 Loan liveness | §6.7.4 |
| 9.6 Region inference | §6.6 d2, §6.7.4 |
| 9.6 Conflict detection | §6.1.3, §6.7.5 |
| 9.6 Place overlap analysis | §6.3 d11, §6.7.5 |
| 9.6 Escape analysis | §6.6 d10, §6.7.7 |
| 9.6 Destruction analysis | §6.7.6 |
| 9.6 Edge-sensitive loan termination | §6.7.4 |
| 9.6 Loop handling | §6.4 d8, §6.7.7 |
| 9.6 Call-boundary handling | §6.7.7 |
| 9.6 Recursive function handling | §6.7.7 |
| 9.6 Diagnostics | §6.7.5 |
| 9.7 Partial initialization | §6.8 d1 |
| 9.7 Partial moves | §6.8 d2 |
| 9.7 Conditional initialization | §6.8 d3 |
| 9.7 Loops with moves | §6.8 d4 |
| 9.7 Multiple exits | §6.8 d5 |
| 9.7 Reborrowing | §6.8 d6 |
| 9.7 Closures | §6.8 d7 |
| 9.7 Captured references | §6.8 d8 |
| 9.7 Recursive aggregates | §6.8 d9 |
| 9.7 Mutable slices | §6.8 d10 |
| 9.7 Self-referential structures | §6.8 d11 |
| 9.7 Pinning | §6.8 d12 |
| 9.7 Async suspension | §6.8 d13 |
| 9.7 Foreign pointers | §6.8 d14 |
| 9.7 Interior mutability | §6.8 d15 |
| 9.8 Unsafe authority | §6.9 d1 |
| 9.8 Raw-pointer access | §6.9 d2 |
| 9.8 Manual initialization | §6.9 d3 |
| 9.8 Manual destruction | §6.9 d4 |
| 9.8 Manual allocation | §6.9 d5 |
| 9.8 Manual deallocation | §6.9 d6 |
| 9.8 Unsafe trait obligations | §6.9 d7 |
| 9.8 Unsafe function contracts | §6.9 d8 |
| 9.8 Borrow checker bypass boundaries | §6.9 d9 |
| 9.8 Safety invariants | §6.9 d10 |
| 9.8 Unsafe diagnostics and auditing | §6.9 d11 |
| 9.9 Pin semantics | §6.10 d1 |
| 9.9 Pinned storage | §6.10 d2 |
| 9.9 `Unpin` | §6.10 d3 |
| 9.9 Address stability | §6.10 d4 |
| 9.9 Pinned references | §6.10 d5 |
| 9.9 Pinned ownership | §6.10 d6 |
| 9.9 Pin projection | §6.10 d7 |
| 9.9 Self-referential objects | §6.10 d8 |
| 9.9 Async state machines | §6.10 d9 |
| 9.9 Destruction of pinned values | §6.10 d10 |
| 9.9 Unsafe pinning contracts | §6.10 d11 |
| (deliverable, lines 911–913) | §6.11 |

bullets in scope: 117, defined: 117
