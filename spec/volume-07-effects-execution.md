<!-- CEP:FILE: spec/volume-07-effects-execution.md
CEP:WHAT: Volume 07 of the Planck Master Specification Library: the effect system (categories, semantics, function effects, optimization semantics) and the evaluation and control-flow semantics (expression evaluation, control flow, pattern matching, operator semantics, function-call semantics, panic and error handling, trap semantics). Embeds the pinned effect algebra of SPEC-INDEX §5.3 verbatim.
CEP:WHY: design.md §10 (lines 917–976) and §11 (lines 977–1085) require a complete effect system and operational semantics with every expression and control construct defined; this volume is the single normative definition home for all 127 bullets of design.md §10–§11 and settles every open policy question they pose (strict evaluation order, default overflow policy, labeled control flow, yield, elision of unknown effects, tail calls, variadics).
CEP:STATUS: complete
CEP:FAILURE: This volume must be rejected if any design.md §10–§11 bullet lacks a coverage-ledger row, if the pinned effect algebra of SPEC-INDEX §5.3 is altered, or if any rule here lacks an exact trigger, an exact code, an exact constant, or a complete enumeration.
CEP:ASSUMES: SPEC-INDEX §3 (UB model; panic/abort and unwinding model; effect-polymorphism decision), §5.3 (pinned effect algebra, embedded verbatim in §7.1), §5.5 (operator table columns), Volume 02 (full operator table, literal math), Volume 04 (types, traits, `dyn Any`), Volume 05 (layout, drop glue), Volume 06 (ownership actions referenced by evaluation), Volume 08 (async/atomics semantics referenced by effects), Volume 10 (ABI, intrinsics), Volume 12 (limits), standards.md §3 (normative verbs), verification standards.md G1, G3, G13, G14, G15, G17, G22.
CEP:COST: compile-time effect checking by row unification, worst case O(|C|·V) with V row variables (§7.3.9); no runtime cost: effect rows are erased after type checking (§7.3.13).
CEP:EVIDENCE: verification matrix §7.12 (OB-07-1..OB-07-11); coverage ledger §7.13 (bullets in scope: 127, defined: 127).
-->

# Volume 07 — Effects and Execution Semantics

## 7.0 Scope and traceability

This volume defines design.md §10 "Effect system" (lines 917–976) and design.md §11 "Evaluation and control-flow semantics" (lines 977–1085). §10 contains 4 subsections (§10.1–§10.4) and §11 contains 6 subsections (§11.1–§11.6), together **127 bullets**, each of which receives exactly one normative definition below. Section mapping: §7.2 ↔ design.md §10.1, §7.3 ↔ §10.2, §7.4 ↔ §10.3, §7.5 ↔ §10.4, §7.6 ↔ §11.1, §7.7 ↔ §11.2, §7.8 ↔ §11.3, §7.9 ↔ §11.4, §7.10 ↔ §11.5, §7.11 ↔ §11.6. The four deliverables named at design.md lines 1082 (Effect System Specification, Operational Semantics Specification, Control-Flow Specification, Error and Panic Specification) are delivered by §7.2–§7.5, §7.6/§7.9/§7.10, §7.7/§7.8, and §7.11 respectively.

Embedded pinned contract: SPEC-INDEX §5.3 is embedded verbatim in §7.1 and elaborated in §7.2–§7.4. Where this volume and SPEC-INDEX §5.3 could be read to disagree, §5.3 wins (SPEC-INDEX §0.5).

Normative verbs have the meaning of standards.md §3. Compile-time rejection codes are `P07-C<nnn>`; the set is closed (§7.3.14, §7.11.10). All named constants have exact values.

## 7.1 Pinned effect algebra (SPEC-INDEX §5.3, verbatim)

### 7.1.1 Embedded contract

The following text is copied byte-for-byte from SPEC-INDEX §5.3 and is normative without modification:

- Effect atoms: `io`, `alloc`, `panic`, `unsafe`, `async`, `concurrency`, `ffi`, `nondet`. Atom set is closed; adding an atom is an edition change.
- A row is a finite subset of the atom set, plus a row variable flag (open rows for effect polymorphism). Canonical form: sorted, deduplicated.
- `join(r1, r2) = r1 ∪ r2` (set union). `meet(r1, r2) = r1 ∩ r2`. Row subtyping for effect *requirement* positions: `r1 ≤ r2 ⟺ r1 ⊆ r2` (demanding fewer effects is allowed where a row is required).
- Function call rule: calling `f : (row ρ) → τ` requires the caller's row to include ρ.
- Laws (each an obligation): join is associative, commutative, idempotent, with identity ∅; meet distributes over join; subtyping is a partial order (reflexive, antisymmetric, transitive). All laws EXH over the full 2^8 atom lattice (256 rows; pairwise checks bounded per-law domain, stated per obligation).

### 7.1.2 Canonical order

The canonical sorted order for canonical form is the fixed alphabetical order on atom names: `alloc < async < concurrency < ffi < io < nondet < panic < unsafe`. Two rows are equal iff their canonical forms are equal.

### 7.1.3 Row representation

A row is exactly one of: (a) a closed row, a subset of the atom set A = {io, alloc, panic, unsafe, async, concurrency, ffi, nondet}, 256 possible values; (b) an open row, a single row variable ρ drawn from a per-item namespace. There are no tail variables and no open rows with fixed atom parts: if an item needs "at least io", it is written as a closed row containing io, or as a row variable constrained by an outlives-style constraint (§7.3.8). This settles the design of open rows for edition P1.

## 7.2 Effect categories (design.md §10.1)

### 7.2.1 Mapping of design categories to the pinned atoms

design.md §10.1 lists effect categories that must be defined. The pinned atom set (§7.1.1) is the authoritative vocabulary; the mapping below is normative and total:

| design.md category | Atom(s) | Exact meaning |
| ------------------ | ------- | ------------- |
| Read | (no atom) | Reading owned memory carries no atom; it is not observable outside the allocation (§7.3.3) |
| Write | (no atom) | Writing owned memory carries no atom, same rule |
| Allocation | `alloc` | Any heap allocation (Box, Rc/Arc clone, Vec growth, future construction with captures) |
| Deallocation | `alloc` | Drop glue and free are memory effects and carry `alloc` |
| IO | `io` | File, socket, console, and OS-handle operations |
| Atomic | `concurrency` | All atomic operations (Volume 08 §8.5) carry `concurrency` |
| Synchronization | `concurrency` | Locks, barriers, channels, fences, thread spawn/join (Volume 08 §8.2–§8.4) |
| Panic | `panic` | `panic!` and every operation whose default failure mode is a panic (§7.9 d13); absent in `panic = abort` builds, where failure is the trap instead |
| Suspension | `async` | Every `.await` (§7.4 d8) and every generator `yield` (§7.7 d11) |
| FFI | `ffi` | Every call to an `extern` function |
| Any additional required external effects | (banned) | The atom set is closed (§7.1.1); edition P1 defines no additional external effect categories; a requirement for a new category is settled by an edition change, not by a per-project atom |

The nondeterminism sources of edition P1 (thread scheduling of multi-threaded executors, spurious wakeups, lock acquisition order, weak-memory value choice, OS scheduling visible through clocks) are covered by `nondet`; an operation carries `nondet` iff its result may vary between two executions under identical inputs and program state (§7.3.3). Pure reads/writes of owned memory carry no atom because they are neither externally observable nor nondeterministic; cross-allocation ordering guarantees are optimization-semantics concerns (§7.5), not row content.

## 7.3 Effect semantics (design.md §10.2)

### 7.3.1 Observable versus non-observable effects

An atom is **observable** iff a two-run differential test with identical inputs can distinguish its presence from its absence through the program's own outputs. The exact classification: `io`, `nondet` are observable; `concurrency` is observable through `nondet` only when the code is multi-threaded — an atomic operation on single-threaded code is non-observable, on multi-threaded code it is observable through interleaving, so `concurrency` is classified as conditionally observable; `alloc` is non-observable except through allocation-failure traps at the compiler limit (Volume 12); `panic` is observable at the process boundary (exit status) only in `panic = abort` builds; `unsafe`, `async`, `ffi` are not themselves observable — they mark obligation categories whose carried operations are observable per the rules above. Optimizations may eliminate an operation only when every atom in its row is non-observable under this classification (§7.5 d2).

### 7.3.2 Memory-region effects

Every memory-touching operation is qualified by a region set: `regions(op) ⊆ {allocation IDs}` of the allocations it may touch. Region sets are computed in IR 1 (Volume 13) from provenance (SPEC-INDEX §3) and are used for dependency partitioning and reordering (§7.5 d5, d6); they are not part of effect rows and must not appear in source signatures.

### 7.3.3 Effect identities

Each atom is an identity with a fixed extension, exactly: `io` = operations transferring data across the process boundary through OS interfaces; `alloc` = operations changing the heap allocation state; `panic` = operations that can start unwinding (or trap in abort builds); `unsafe` = operations requiring unsafe permission (Volume 06 §6.9 d1); `async` = operations suspending or resuming a future or generator; `concurrency` = operations synchronizing threads or performing atomics; `ffi` = calls crossing the ABI boundary to non-Planck code; `nondet` = operations whose result may vary between identical executions. The extensions are disjoint classifications: one operation may carry several atoms (e.g. an FFI call to a function that allocates and can block carries {ffi, alloc, concurrency} per its declared row, §7.4 d7).

### 7.3.4 Effect dependencies

Effect dependencies are the optimizer's barrier relation B (§7.5 d1), not a row implication: the presence of atom a never forces the presence of atom b in a row. The exact barrier relation is defined in §7.5 d1. For diagnostics, the presentation order of atoms in a row follows §7.1.2.

### 7.3.5 Effect composition

Composition is join (§7.1.1): the row of a sequential composition is the union of operand rows; the row of a call is the union of argument rows and the callee's row. There are no higher-order effect combinators in edition P1.

### 7.3.6 Effect inference

Inference is bottom-up over the typed AST: the row of every expression is computed by the judgment system of §7.3.9; for functions with an explicit annotation, the inferred row r_inf must satisfy `r_inf ⊆ r_decl` (the annotation is an upper bound the implementation must not exceed); for functions without annotation, the declared row is the inferred row. Inferring `unsafe`: the `unsafe` atom is required by listed operations (§7.2.1) outside an `unsafe` block and is stripped by the block (§7.3.9 rule 7). An inferred row containing `unsafe` where no enclosing `unsafe` block covers it is a compile-time rejection P07-C02.

### 7.3.7 Effect propagation

Propagation rules, exact: (i) function row propagates to its uses through the function type; (ii) trait method row propagates through dyn dispatch and generics as the trait-declared row; (iii) closures propagate the join of capture rows and body row (§7.4 d3); (iv) macro expansion propagates the rows of the expanded statements (no special rule); (v) `async fn` propagates its latent row at await, not at call (§7.4 d8).

### 7.3.8 Effect constraints

Constraints are generated exactly by: (a) call rule (§7.1.1): ambient row must include the callee row — as a unification constraint `r_ambient = r_ambient ∪ r_callee`; (b) annotation constraint `r_inf ⊆ r_decl` (§7.3.6); (c) impl-conformance constraint: impl method row ⊆ trait method row (§7.4 d5); (d) row-variable instantiation constraints at monomorphization (§7.3.12). Constraint solving is row unification (§7.3.9).

### 7.3.9 Row unification algorithm and effect-checking judgments

The checking judgment is `Γ ⊢ e : τ ! r` (expression e has type τ, effect row r under context Γ). The core rules, exact:

1. Literal, variable read, pure arithmetic on `Copy` operands without overflow potential (compile-known non-overflowing): r = ∅.
2. Operator (§7.9): r = join(r_left, r_right) ∪ ({panic} if the operator has a panic condition and the operands do not prove it impossible, §7.9 d13) — in unwind builds; in `panic = abort` builds the `panic` atom is not added (SPEC-INDEX §3: panics carry the atom unless `panic = abort`), and the condition's failure mode is the trap (§7.11 d12).
3. Call: r = join(r_args) ∪ r_callee, where r_callee is the callee's declared or instantiated row; the enclosing function's ambient constraint additionally requires r_callee ⊆ r_decl of the enclosing function — violation is P07-C03.
4. Assignment `p = e`: r = join(r_p, r_e); compound assignment adds {panic} per §7.9 d6.
5. Borrow `&p` / `&mut p`: r = r_p (no added atom; ownership actions are checked by Volume 06).
6. Deref `*p` on references: r = r_p; on raw pointers: r = r_p ∪ {unsafe}.
7. `unsafe { e }`: check e under permission granted; result row = row(e) \ {unsafe}. Emitting an `unsafe` block containing no operation that needs it is diagnostic W07-U01 (allowed, non-failing).
8. `panic!(…)`: r = {panic} in unwind builds; in abort builds the panic is the trap and the row contribution at this rule is ∅ (the trap model of §7.11 d12 governs).
9. `e?` (§7.11 d3): r = r_e.
10. `.await` (§7.4 d8): r = r_inner ∪ {async} ∪ r_latent(callee).
11. `yield` (§7.7 d11): r = r_inner ∪ {async} — generators carry the async atom for suspension mechanics; the pure synchronous generator case is distinguished by its type (Volume 08 §8.9), not by a different atom.
12. Block: r = join of statement rows; scope-exit drops add the join of the drop rows of may-initialized locals (Volume 05 drop glue rows), so a block containing a local whose `Drop` impl is declared `!{io}` checks with `io` included.

Row unification operates on the constraint set C generated by §7.3.8 (a)–(d) together with the block rule 12 of this section. Exact algorithm:

1. Normalize both operands to canonical form (§7.1.2).
2. Case analysis over (row₁, row₂):
   a. (closed, closed): succeed iff equal; else fail with `UnifyError { only_in_left: row₁ \ row₂, only_in_right: row₂ \ row₁ }`.
   b. (var ρ, closed r): occurs check (ρ must not appear, transitively, in the current binding of ρ); bind σ(ρ) := r.
   c. (closed r, var ρ): symmetric to (b).
   d. (var ρ₁, var ρ₂): if ρ₁ = ρ₂ succeed; else bind σ(ρ_max) := ρ_min where min/max are by variable creation index (determinism, G14).
3. Apply bindings with path compression; re-check deferred constraints involving bound variables; loop until no deferred constraints remain or a failure occurs.

Termination measure: the pair (number of unbound row variables, number of unprocessed constraints) decreases lexicographically at every step — bindings decrease the first component, non-binding steps decrease the second and never increase the first. Total steps ≤ |C| + V + Σ bindings, worst-case complexity O(|C|·V) with path compression. Termination is obligation OB-07-6. A failure is compile-time rejection P07-C01 (effect mismatch), reporting both rows in canonical form.

### 7.3.10 Effect subtyping

Row subtyping in requirement positions is exactly the pinned relation `r1 ≤ r2 ⟺ r1 ⊆ r2` (§7.1.1). Positions: (i) impl methods against trait methods (§7.4 d5); (ii) function arguments accepting a callee row; (iii) re-exports and type aliases of function types. Non-requirement positions (the row a body produces) use equality/join, not subtyping.

### 7.3.11 Effect polymorphism

Effect polymorphism is supported exactly through row variables (§7.1.3): a generic function may abstract over the row of a function-valued parameter; the variable is universally quantified at the definition and instantiated per call site (§7.3.12). There is no bounded effect polymorphism (no constraint language over rows beyond ⊆ via §7.3.10) in edition P1.

### 7.3.12 Effect specialization

Monomorphization instantiates each row variable with the concrete closed row of the call site's callee (via §7.3.9 case (b)/(c)). The specialized instance's semantics must equal the generic's semantics with the row substituted (SPEC-INDEX §3 specialization law; obligation family OB-04 of Volume 04 applies). Specialized rows are erased at code generation (§7.3.13).

### 7.3.13 Effect erasure

All effect rows and row variables are erased after type checking: no runtime representation exists, no metadata section carries them, and no codegen decision may depend on a row at runtime. The only persistent artifacts are the compiler's own effect summaries (§7.5 d4) in its incremental cache.

### 7.3.14 Unknown effect handling

The atom set is closed (§7.1.1): an atom name outside the 8 pinned names is a parse/check rejection P07-C01. Opaque callees receive default rows exactly: `extern` function without annotation: `{ffi}`; `extern` function with attribute-annotated row: the annotated row joined with `{ffi}`; function pointer without row annotation: `{}` (the row is part of the pointer type, §7.4 d2); trait-object method: the trait-declared row. There is no "unknown effect" runtime value; nothing is deferred to runtime.

## 7.4 Function effects (design.md §10.3)

**d1 — Function declarations.** Syntax, exact productions:

```
Signature  ::= "fn" IDENT GenParams? Params ("->" Type)? EffectAnno?
EffectAnno ::= "!" "{" AtomList? "}" | "!" Atom | "!" ROWVAR
AtomList   ::= Atom ("," Atom)*
Atom       ::= "io" | "alloc" | "panic" | "unsafe" | "async" | "concurrency" | "ffi" | "nondet"
```

Omitted EffectAnno means the closed row ∅. Atoms must be unique within a list (duplicate is P07-C01). `async fn` is sugar for a function returning a future type with latent row = annotated row (d8).

**d2 — Function types.** The function type carries the row: `fn(T₁, …, Tₙ) -> U ! r`. Function pointer types are the same form with `fn` (no lifetime polymorphism beyond Volume 06 regions). The row is part of type equality: two function types are equal iff parameters, return, and rows are equal (rows in canonical form; open rows equal by variable identity after unification).

**d3 — Closure effect types.** A closure's row is `join(rows of captures-as-accessed) ∪ row(body)` per §7.3.9 rule 12; the closure type implements exactly one of the function traits `Fn`, `FnMut`, `FnOnce` (Volume 04) with the row as part of the trait's method signature. Capture access rows: shared capture read contributes ∅; exclusive capture use contributes ∅ (ownership, not effects); move capture contributes the capture's construction row? No: move capture contributes ∅ at call time — the move happened at closure construction, whose row includes the moved values' rows. Exact: closure-construction row = join of rows of moved captures' evaluation; closure-call row = body row.

**d4 — Generic effect contracts.** A generic function may abstract over a callee row: `fn call<F, T, U>(f: F, x: T) -> U !ρ where F: Fn(T) -> U !ρ` — ρ is a row variable universally quantified at the definition. Instantiation per call site binds ρ by unification (§7.3.9). The caller of `call` must include the instantiated row (pinned call rule).

**d5 — Trait method effects.** A trait declares each method's row; an impl's method row must satisfy `r_impl ≤ r_trait`, i.e. `r_impl ⊆ r_trait` (§7.3.10): an impl may demand fewer effects than the trait declares, never more (violation P07-C04). Callers through the trait use the trait row; callers through the concrete type use the impl row.

**d6 — Dynamic dispatch effect contracts.** A vtable slot stores the code address only; the effect contract at a dyn call site is statically the trait-declared row (erased at runtime per §7.3.13). The compiler must not assume a stronger row than the trait's for any dyn dispatch (obligation OB-07-7).

**d7 — Foreign function effects.** An `extern "ABI"` declaration's row is `{ffi}` joined with its annotation (§7.3.14). Foreign functions must not carry `panic` in unwind builds (§7.11 d9). A foreign function declared with `nondet` documents its nondeterminism sources; `nondet` without `ffi` cannot arise for foreign functions.

**d8 — Async latent effects.** Calling an `async fn` creates its future: that call's row is ∅ plus the rows of the argument expressions (construction is pure except for captures requiring allocation, which contribute `alloc` when the state machine heap-allocates — the compiler must state which, per target, in Volume 08 §8.7; by contract here: construction row = join(r_args) ∪ {alloc} iff the future is heap-boxed, else join(r_args)). Executing the body happens at poll/await: the awaiting context's row includes `{async} ∪ r_latent` where r_latent is the async fn's annotated row (§7.4 d1). The pinned call rule applies at the await site against r_latent.

**d9 — Constructor effects.** Constructors are ordinary functions: struct/tuple/enum literal construction carries ∅ plus operand rows; heap-boxing adds `alloc`; `Rc`/`Arc`/`Vec` constructors are stdlib functions whose declared rows include `alloc` (Volume 19). There is no special language-level constructor mechanism.

**d10 — Destructor effects.** A `Drop` impl's `drop(&mut self)` method has a declared row r_drop; every scope whose locals may run this drop glue carries r_drop in its block row (§7.3.9 rule 12). In `panic = abort` builds, drop rows still count (drops run before abort paths that are not traps). Unwinding drops carry the same rows (§7.11 d6).

**d11 — Intrinsic effects.** Each compiler intrinsic has a fixed declared row, tabulated in Volume 10 §15 (the table is normative there); the mechanism is defined here: intrinsic rows are closed, never open, and are checked like ordinary callee rows. The memory-model intrinsics (atomics, fences) carry `{concurrency}` (Volume 08 §8.5); the trap intrinsic carries `{panic}` in unwind builds and is the trap primitive (§7.11.9); alloc/dealloc intrinsics carry `{alloc}` (Volume 06 §6.9 d5, d6).

## 7.5 Optimization semantics (design.md §10.4)

**d1 — Reordering.** Two operations a, b in one thread may be reordered by the optimizer iff one of: (i) both are pure (row ∅) and their region sets (§7.3.2) are disjoint or one is read-only on a region the other does not write; (ii) `row(a) ∪ row(b) ⊆ {alloc}` and their region sets are disjoint; (iii) `row(a) ∪ row(b) ⊆ {unsafe}` and their region sets are disjoint. Otherwise (a, b) is a barrier pair. Traps and panics are barriers in both directions (a trap must not be moved past an operation with any atom, and vice versa, unless the moved-across operation is pure and its result is dead — in which case it may be deleted, not moved). This ordering contract is a G21 obligation (Volume 08 §8.10 covers the concurrent case).

**d2 — Dead-effect elimination.** An operation may be deleted iff its result is unused and every atom of its row is non-observable per §7.3.1 (i.e. row ⊆ {alloc, unsafe} and, for `alloc`, the allocation is provably dead — not read, not aliased into a live place, not dropped-with-side-effect). An operation with row containing io, concurrency, nondet, ffi, panic, or async must never be deleted. Deletion of an allocation whose drop glue has a non-∅ row is banned.

**d3 — Effect-preserving inlining.** Inlining a callee with row r_c into a caller whose ambient row r_a satisfies `r_c ⊆ r_a` is always permitted (the join cannot grow). Inlining is permitted even when `¬(r_c ⊆ r_a)` only if the caller's declared row is then widened to include r_c — but widening a declared row after checking is banned in edition P1: inlining is therefore permitted iff `r_c ⊆ r_a` at the IR level, where r_a is the IR-1 function's accumulated row (which may exceed the source annotation only by compiler-introduced drop rows, §7.4 d10).

**d4 — Call summaries.** Per monomorphized function, the compiler must produce the summary `Σ(f) = (row: closed row, may_read: region set, may_write: region set, may_trap: bool, may_panic: bool)`. Summaries are computed bottom-up, cached (incremental), and used for reordering (d1), dead-effect elimination (d2), and dependency partitioning (d5). Summary content is part of the IR 1 specification (Volume 13) and its equivalence to actual behavior is a G1 obligation.

**d5 — Memory dependency partitioning.** Memory operations are partitioned into dependency classes by (allocation ID, offset-range) from §7.3.2; operations in disjoint classes do not depend, except through barrier pairs (d1). Volatile-equivalent operations (Volume 10 intrinsics) are their own class and never merge.

**d6 — Effect independence.** Two effects are independent iff no barrier pair relates them (d1) and their region sets are disjoint. Independence licenses reordering and fusion; it must never license deletion (d2 governs deletion).

**d7 — Unknown effect barriers.** An operation whose summary is unavailable (opaque FFI, unresolved indirect call at analysis time) must be treated as row `{io, alloc, panic, concurrency, nondet}` for optimization purposes: it is a barrier against everything except pure operations with disjoint regions. This conservative default may not be widened to UB or narrowed below the declared row.

**d8 — Atomic ordering.** Atomic operations (Volume 08 §8.5) carry their ordering as IR facts; the optimizer must preserve per-ordering guarantees exactly: it may not merge, duplicate, split, or reorder atomic RMW operations; it may not weaken an ordering; it may strengthen only when the target's lowerings coincide (d1 of Volume 08 §8.5). Fence operations are barriers against all memory operations of weaker-or-unrelated region per the Volume 08 fence rules.

**d9 — IO ordering.** Operations with `io` are barriers against each other and against `nondet` and `concurrency` operations (d1); their relative order must be preserved exactly as in the source program's evaluation order (§7.6 d1), including across inlining.

## 7.6 Expression evaluation (design.md §11.1)

The operational semantics uses the judgment `⟨e, σ⟩ ⇓ v` (terminating value), `⟨e, σ⟩ ⇑` (divergence), or `⟨e, σ⟩ ⇓ trap(134)` (defined trap, §7.11.9). σ is the store: allocations with contents, plus the frame stack. Evaluation of a well-typed program is deterministic except at operations carrying `nondet` or `concurrency` (Volume 08); within a single thread, the order below is exact.

**d1 — Evaluation order.** Evaluation is strict. The order of subexpression evaluation is left-to-right for every construct, with exactly these exceptions: `&&` and `||` (right operand conditionally evaluated, §7.9 d3), `?` (right side is the implicit early return, §7.11 d3), and `yield`/`.await` (suspend/resume mechanics, §7.7 d11, §7.4 d8). Function arguments: left-to-right, then the call (§7.10 d1). Assignment and compound assignment: left place expression first, then right value expression (§7.9 d5, d6).

**d2 — Operand evaluation order.** For every binary operator and every n-ary construct (array literals, struct literals, tuples, calls, indexing `a[i]`), operands evaluate in source order left-to-right, one operand fully (including nested effects, panics, traps) before the next begins. Partially-evaluated operands that panic leave earlier operands' side effects performed (this is observable and normative).

**d3 — Result type.** Every expression has exactly one type, computed by Volume 04's type system; this volume adds only the evaluation rules. The never type `!` types all divergent expressions (§7.7 d14).

**d4 — Result category.** Value categories are exactly: place expression (denotes a place; may appear on the left of `=`, may be borrowed), value expression (denotes a value). Every place expression can be used as a value expression by a read; the reverse is banned. The category grammar follows Volume 04 §6; evaluation rules here are stated against both.

**d5 — Ownership behavior.** By-value use of a non-`Copy` value performs `MoveOut` on the source place and `Write` on the destination (Volume 06 §6.2 d8, §6.4 d1); `Copy` types read. Temporary values created by value expressions are stored in compiler-allocated temporaries with the lifetimes of Volume 06 §6.5 d12.

**d6 — Borrowing behavior.** `&e` / `&mut e` on a place expression create loans per Volume 06 §6.5 d3; on a value expression (rvalue) the compiler materializes a temporary first and borrows that temporary. Two-phase behavior is Volume 06 §6.7.3.

**d7 — Effects.** The row of an expression is computed by §7.3.9; the runtime effect of each atom is the operation class of §7.3.3. There is no runtime effect representation (§7.3.13).

**d8 — Panic conditions.** An operation has a panic condition iff its row includes `panic` (§7.2.1) — in unwind builds; in `panic = abort` builds the same conditions are traps (§7.11 d12) and no atom is added (SPEC-INDEX §3). The complete list of operations with panic conditions in edition P1: integer arithmetic overflow (+, −, *, /, % on ints, and shift amount invalid), array/slice index out of bounds, explicit `panic!`, slice construction with `end > len` or reversed range at runtime, allocation exceeding the compiler limit table (Volume 12), `Option`/`Result` unwrapping methods declared panicking (Volume 19), calling a function whose body panics, and drop glue declared with `panic` in unwind builds. Each panic carries the payload per §7.11 d4.

**d9 — Divergence conditions.** An expression diverges iff its execution never reaches a value, trap, or panic: possible exactly in `loop` without break (§7.7 d8), recursion without a base case (§7.10 d5), and non-terminating iterators (Volume 19). Divergence is defined behavior, must not be UB, and must not be prevented by the compiler. Stack exhaustion during divergence is the defined trap (§7.10 d5). A diverging expression has type `!`.

**d10 — Constant-evaluation rules.** A const context (const items, array lengths, enum discriminants, const generics) evaluates a subset of expressions: those whose row is ⊆ {alloc} and which do not read mutable statics, call non-const functions, or take raw pointers of runtime allocations. Const evaluation is an interpreter with exact limits: `MAX_CONST_EVAL_DEPTH = 512` (call depth), `MAX_CONST_EVAL_STEPS = 10_000_000` (interpreter steps). Exceeding a limit, performing a panic condition, or violating a place rule is a compile-time rejection P07-C05 — never a runtime trap, never UB, never a silent wrong value. Floating-point const evaluation uses the exact IEEE-754 semantics of §7.9 d15, evaluated at compile time on the host with round-to-nearest-ties-even and canonical NaN; the host result must equal the target result (both implement IEEE-754 exactly per this volume; obligation OB-07-8 domain includes float ops).

## 7.7 Control flow (design.md §11.2)

Every construct lowers to the CFG of Volume 06 §6.7.1; the semantics here is the normative source of that lowering. Effect rows accumulate per §7.3.9 rule 12.

**d1 — Blocks.** `{ s₁ … sₙ e? }`: statements in order; the block's value is e, or unit if absent; the block's row is the join of all statement rows and the drop rows of its locals (§7.3.9 rule 12). Scope exit runs drops per Volume 06 §6.7.6 on every path (including break/continue/return/panic paths).

**d2 — If expressions.** `if c { a } else { b }`: c must be `bool` (no truthiness coercion); c evaluates once; then exactly one branch evaluates. Both branches must have the same type for the `if` to be used as a value; otherwise unit. Row = r_c ∪ r_a ∪ r_b.

**d3 — If statements.** Same as d2 in statement position; `else if` chains are nested d2; a missing else with non-unit branch type is a type error (Volume 04), not an evaluation rule.

**d4 — Match expressions.** `match e { arms }`: e evaluates once, left-to-right over its subexpressions; arms are tested top-down; the first arm whose pattern matches (and whose guard, if present, evaluates true) is taken; guards may not diverge the match (guard row must not contain `async`); non-matching arms have no effect. Exhaustiveness per §7.8 d15. Pattern-matching effects: matching itself is a read of e's place (no atom); guard rows join the match row; binding moves follow Volume 06 §6.4.

**d5 — Match statements.** Same evaluation as d4; arms may have unit type; the match statement does not require exhaustiveness only when the scrutinee type is uninhabited or the match is in a diverging position (Volume 04); otherwise exhaustiveness is mandatory (P07-C08).

**d6 — While loops.** `while c { body }`: c evaluated at loop entry and before each repetition; if true, body evaluates and control returns to c; if false, the loop yields unit. Row = r_c ∪ r_body. `break`/`continue` per d9/d10.

**d7 — For loops.** `for pat in e { body }`: e must implement `IntoIterator` (Volume 04/19); exact protocol: `into_iter()` called once, then `next()` repeatedly; `Some(v)` binds pat (per §7.8) and evaluates body; `None` ends the loop and drops the iterator; the iterator is dropped on `break` and on panic paths too (drop at the exit point, Volume 06 §6.7.6). Row = r_e ∪ r_next ∪ r_body where r_next is the iterator's `next` row (from its trait declaration).

**d8 — Infinite loops.** `loop { body }`: body evaluates repeatedly; the loop's value is the value of a `break e` (d9), else its type is `!` (divergent). A `loop` without any `break` diverges (§7.6 d9).

**d9 — Break.** `break` exits the innermost enclosing loop (d6/d7/d8) or labeled construct (d13); `break e` supplies the loop's value (only in `loop`); break across a `match` or block is banned (it exits loops and labeled blocks only). Break runs scope-exit drops of the scopes it exits (Volume 06 §6.7.6).

**d10 — Continue.** `continue` skips to the next iteration of the innermost (or labeled, d13) loop: for `while`, jumps to the condition; for `for`, jumps to `next()`; for `loop`, jumps to the body start. Scope-exit drops of the skipped remainder run (Volume 06 §6.7.6).

**d11 — Yield.** `yield e` is legal only inside a generator body (Volume 08 §8.9): e evaluates, its value is delivered to the resumer as `Yielded(e)`, and the generator suspends; the value of the `yield` expression itself is the resume argument R on resumption. Row = r_e ∪ {async}. `yield` outside a generator is P07-C06. A `yield` inside `async fn` is banned (Volume 08 §8.9 d8, P07-C06).

**d12 — Early exits.** The early-exit constructs are exactly: `return`, `break`, `continue`, `?` (§7.11 d3), and panic/unwind (§7.11). Each runs the drop sequence of every scope it exits (Volume 06 §6.7.6) before transferring control. No other construct exits early.

**d13 — Labeled control flow.** Supported: `'a: loop { … }`, `'a: while … { … }`, `'a: for … { … }`, and `'a: { … }` (labeled block). `break 'a` exits the labeled construct; `continue 'a` targets labeled loops only. Labels are function-local, must be unique per nesting path (shadowing an outer label by an inner identical name is banned, P07-C09), and are erased at codegen.

**d14 — Unreachable paths.** The type `!` marks divergence (§7.6 d9). Statements after a diverging expression are permitted but the compiler must report them as unreachable (diagnostic W07-U02, non-failing) and must not generate code for them. Executing genuinely unreachable generated code would violate G1; the compiler must prove divergence for `!`-typed positions (Volume 04) or the position is rejected (P07-C10). `unreachable!()`-style constructs map to `panic!` with a fixed message; they are not compiler-assertion primitives (standards.md §25.3.1 governs compiler-internal use).

## 7.8 Pattern matching (design.md §11.3)

A pattern is a tree over the pattern grammar of Volume 02 §2; this section defines matching semantics and compilation.

**d1 — Literal patterns.** Integer, char, bool, and string-literal patterns match by value equality using the literal math of Volume 02 (SPEC-INDEX §5.6); integer literal patterns must fit the scrutinee type (else P07-C11); float patterns are banned in edition P1 (float equality is not exhaustiveness-friendly; the ban is normative and complete — there is no float pattern form).

**d2 — Identifier patterns.** `x` binds the matched place by the binding mode (d13); a second binding of the same name in one pattern is banned (P07-C12).

**d3 — Wildcards.** `_` matches everything and binds nothing; `_` may appear any number of times.

**d4 — Reference patterns.** `&p` / `&mut p` match a reference by dereferencing into p with binding mode shared/mut respectively; matching `&mut` against a shared binding is a mode error (P07-C12).

**d5 — Tuple patterns.** `(p₀, …, pₙ₋₁)` match tuples componentwise, left-to-right.

**d6 — Struct patterns.** `S { f₀: p₀, … }` match by field names; omitted fields must be covered by `..` (d9); field order in the pattern is irrelevant; duplicate field is P07-C12.

**d7 — Enum patterns.** `V(a, b)` / `V { f: p }` match the variant V after a discriminant test (Volume 06 §6.3 d9); matching is exclusive per variant.

**d8 — Slice patterns.** `[p₀, …, pₙ]` match fixed-size arrays of length n exactly; `[p₀, .., pₖ]` match slices/arrays with a rest segment (`..` binds a slice of the remainder); fixed patterns against slices are banned (length unknown).

**d9 — Or patterns.** `p | q` match if either matches; both sides must bind the same name set with the same modes; or-patterns may nest arbitrarily (depth ≤ MAX_PLACE_DEPTH = 64).

**d10 — Binding patterns.** `mut x` declares a mutable binding; `ref x` / `ref mut x` force shared/exclusive binding modes (d13); binding requires the scrutinee place to permit the corresponding action (Volume 06).

**d11 — Guards.** `p if g`: g is a bool expression evaluated after p matches, before binding finalization; a guard failure falls through to later arms; guards may not mutate the scrutinee place (borrow-checked as a shared borrow of the match scrutinee) and may not contain `?` or `yield`.

**d12 — Move bindings.** A by-value binding of non-`Copy` type performs `MoveOut` on the matched subplace (Volume 06 §6.4 d1); matching does not move unless the binding mode is by-value.

**d13 — Copy bindings.** Bindings of `Copy` types copy (Volume 06 §6.4 d1); the source place remains owned.

**d14 — Reference bindings.** Default binding modes: matching a non-reference scrutinee against a non-reference pattern binds by value/copy; matching a reference scrutinee against a non-reference pattern shifts the default mode (shared after `&`, mut after `&mut`) — the exact ergonomics rule: the default mode is the mode required to make the pattern's top level match the scrutinee's reference structure, applied recursively. `ref`/`ref mut` override explicitly (d10).

**d15 — Exhaustiveness.** The checker computes, for each match, whether the union of arm patterns covers the scrutinee type, using the usefulness algorithm: `useful(M, p)` decides whether pattern p matches a value not matched by matrix M, computed by specialization on constructors (enum variants, tuple/struct fields, slice lengths, literals, or-pattern splitting); it terminates because each recursive call strictly reduces (constructor count + pattern depth). A non-exhaustive match is P07-C08 with a witness value constructed from the un-covered space (G17 data). Wildcard-carrying rests (`..`) make the check trivially pass for the covered extent.

**d16 — Reachability.** An arm is reachable iff `useful(arms above it, its pattern)` is true; an unreachable arm is diagnostic W07-U03 (non-failing, must be reported).

**d17 — Irrefutability.** A pattern is irrefutable iff `useful(∅, p)` holds against the full type (matches every value); `let` and `for` bindings require irrefutable patterns (P07-C13 otherwise); `if let` / `while let` accept refutable patterns.

**d18 — Pattern compilation.** Compilation lowers a match to a decision tree: interior nodes test discriminants, field values, slice lengths, or literal equality; leaves perform bindings in source order of the matched arm. Compilation must preserve: first-match-wins semantics, guard placement after structural match and before binding finalization, and the binding move order left-to-right (d5, d6). Tree size is bounded by `MAX_PATTERN_TREE_NODES = 65536` per match; exceeding is P07-C11. Or-patterns compile to shared subtrees by deduplication of equal binding structure; sharing must not duplicate side effects of guards (guards compile after the structural decision, once per arm taken).

## 7.9 Operator semantics (design.md §11.4)

The operator table (tokens, precedence, associativity, arity, overloadability, overflow class) is Volume 02 §2.4 (pinned columns in SPEC-INDEX §5.5); this section defines the evaluation semantics of each class. All operators evaluate operands per §7.6 d2 except where d3 states otherwise.

**d1 — Arithmetic.** `+ − *` on integers: mathematical result; on overflow, the default is the panic condition (§7.6 d8, trap-equivalent panic; the `panic` atom applies). The named method forms are exact: `wrapping_*` (two's-complement wrap), `checked_*` (returns `Option`), `saturating_*` (clamps to min/max), `unchecked_*` (unsafe; declared precondition "no overflow"; violation of the precondition is UB-6 per SPEC-INDEX §3, because `unchecked_*` are intrinsics). `/` and `%` on integers: division/remainder truncating toward zero; `MIN / −1` and `MIN % −1` are panic conditions (no wrap); division by zero is a panic condition (§7.9 d14). On floats: IEEE-754-2019 binary32/binary64 operations, round-to-nearest-ties-even (§7.9 d15).

**d2 — Comparisons.** `== != < <= > >=` produce `bool`. Integers: mathematical comparison. Floats: IEEE-754 comparisons: any comparison with NaN is false except `!=` which is true; `+0.0 == −0.0` is true. Chars/bools/enum-without-data: by discriminant/value order. References: compare referents. Overloadable via `PartialEq`/`PartialOrd` (d11).

**d3 — Logical operations.** `&&` and `||` on `bool` short-circuit, exactly: `a && b` evaluates a; if false, result false and b is not evaluated; else evaluates b. `a || b` symmetric. Not overloadable. Their row is r_a ∪ (r_b if b may evaluate — the checker conservatively uses r_a ∪ r_b).

**d4 — Bitwise operations.** `& | ^ !` on integers/bools: bitwise. `<< >>` on integers: shift amount must satisfy `0 ≤ amount < bit_width(T)`; violation is a panic condition (§7.9 d13); shift semantics: logical shift for unsigned, arithmetic (sign-preserving) for signed; shifts of negative amounts or ≥ width by the `<<`/`>>` operators are panics; the `wrapping_shl/shr` methods mask the amount modulo width (exact).

**d5 — Assignment.** `p = e`: evaluate p to a place, evaluate e, perform the write (`Write` action, Volume 06). `Copy` types write by copy; non-`Copy` types move (§7.6 d5). The expression's value is unit. Destructuring assignment `p₁ = p₂ = e`? Banned in edition P1 (single-assignment only, exact).

**d6 — Compound assignment.** `p op= e` for op ∈ {+, −, *, /, %, &, |, ^, <<, >>}: exactly the desugar `p = p op e` with p's place expression evaluated exactly once (address taken first, then e, then the operation; a panic during e leaves p unmodified — normative). Overloadable via the `*Assign` traits (d11). Value: unit.

**d7 — Ranges.** Range expressions: `a..b` (half-open, `Range`), `a..=b` (closed, `RangeInclusive`), `a..` (`RangeFrom`), `..b` (`RangeTo`), `..` (`RangeFull`). Their `contains` is exact integer comparison per the bounds' inclusivity. Iteration semantics: for `a..b` on integer T, yields a, a+1, … while `< b` (empty iff a ≥ b); for `a..=b`, yields exactly `b − a + 1` values when a ≤ b (implementation must not overflow at T::MAX — normative obligation on the iterator's internal counter, using a widened counter or the wrapping-proof structure); open-ended ranges iterate to T::MAX then stop. Range expressions evaluate a and b left-to-right.

**d8 — Casting.** `e as T`, exact rules per source/target family: int→int: truncate to the target width and reinterpret (two's complement; defined, never a panic); int→float: nearest value, round-to-nearest-ties-even (large ints may lose precision — defined); float→float: IEEE conversion binary64→binary32 (or identity); float→int: saturation (NaN → 0; below `T::MIN` → `T::MIN`; above `T::MAX` → `T::MAX` — exact, never UB, never a panic); bool→int: 0/1; char→u32: scalar value; enum→int: allowed only for field-less enums, yields the discriminant (Volume 04 repr); usize↔pointer: only via unsafe intrinsics (Volume 10), not via `as`. Any other pair: compile-time rejection P07-C14.

**d9 — Dereferencing.** `*p`: on references, reads/writes the referent place (Volume 06 rules; row = r_p); on raw pointers, requires `unsafe` and checks provenance at the memory model level (UB-2 on violation); on `Deref`/`DerefMut` types, calls the trait method (row joins the method's declared row).

**d10 — Borrow operators.** `&` and `&mut` per Volume 06 §6.5; not overloadable in edition P1 (`Deref`-based smart pointers provide the only indirection customization).

**d11 — Operator overloading.** Overloadable operators and their traits, complete list: `+ − * / %` via `Add Sub Mul Div Rem`; unary `−` `!` via `Neg Not`; `& | ^` via `BitAnd BitOr BitXor`; `<< >>` via `Shl Shr`; the ten `op=` forms via `AddAssign SubAssign MulAssign DivAssign RemAssign BitAndAssign BitOrAssign BitXorAssign ShlAssign ShrAssign`; `== !=` via `PartialEq`; `< <= > >=` via `PartialOrd`; indexing via `Index`/`IndexMut`; deref via `Deref`/`DerefMut`. Overloaded operator evaluation: operands evaluate per §7.6 d2, then the trait method call occurs; the row is `join(r_operands) ∪ r_method`. Overloading must not change precedence, associativity, or arity (Volume 02 §2.4 columns are fixed).

**d12 — Short-circuiting.** Exactly `&&` and `||` (d3). No other operator short-circuits; in particular overloaded operators, indexing, and range endpoints are strict.

**d13 — Overflow behavior.** The overflow class column of Volume 02 §2.4 is exact: integer `+ − *` and shifts have class Panic by default; `wrapping_*` class Wrap; `checked_*` class N/A (returns Option); `saturating_*` class Saturate; `unchecked_*` class Unchecked (UB-6 on violation, §7.9 d1). There is no build-mode flag that changes these classes: debug and release builds have identical overflow behavior (determinism, G13); the only difference permitted between profiles is diagnostic verbosity, never semantics. Float overflow produces ±infinity per IEEE (defined, not a panic).

**d14 — Division-by-zero behavior.** Integer `/` and `%` by zero: panic condition (§7.6 d8). Float `/` by zero: IEEE-754 result (±inf or NaN per operand signs; 0/0 → canonical NaN) — defined, not a panic.

**d15 — Floating-point edge cases.** Exact contract: binary32 and binary64 per IEEE-754-2019; every arithmetic, conversion, and comparison rounds to nearest, ties to even; NaN results of arithmetic are the canonical quiet NaN (sign bit 0, payload 0) — payload propagation is not preserved and never observable as a distinct value; `+0.0` and `−0.0` are distinct bit patterns, equal under `==`, and `−0.0 + 0.0 = +0.0`; `min`/`max` functions (Volume 19) return the non-NaN operand when one operand is NaN, and `+0.0` over `−0.0` on ties; `sqrt(−0.0) = −0.0`; overflow → ±inf; underflow → nearest denormal or ±0 with the inexact flag semantics not exposed. FMA contraction is banned unless the target has a fused instruction and the source used the `fma` method explicitly (no implicit contraction; determinism G14).

## 7.10 Function-call semantics (design.md §11.5)

**d1 — Argument evaluation.** Arguments evaluate left-to-right (§7.6 d2), fully (values materialized into argument temporaries), then the callee begins. An argument panic/trap aborts the call before the callee starts; earlier arguments' effects remain performed.

**d2 — Argument ownership transfer.** By-value arguments move (Volume 06 §6.4 d10); by-reference arguments loan (Volume 06 §6.5); `Copy` arguments copy. The callee never drops a caller-owned allocation it did not receive by value.

**d3 — Parameter initialization.** Parameters are locals of the callee, initialized in declaration order at frame entry (state `Owned` per Volume 06 §6.3 d3); they drop at frame exit in reverse declaration order, interleaved with other locals per Volume 06 §6.7.6 scope rules.

**d4 — Return-value semantics.** The callee returns exactly one value into a caller-provided return slot; `return e` / the final expression perform `MoveOut` on e's place and `Write` into the slot; `Copy` types copy. `return` runs scope-exit drops to the function boundary first (§7.7 d12). The caller then drops nothing extra; the return slot is a value in the caller's frame.

**d5 — Stack frame semantics.** A frame holds parameters, locals, temporaries, and the return slot; layout is target ABI (Volume 10); semantically the frame is destroyed at return (or at unwinding, §7.11 d5, or at trap — trap skips drops, §7.11.9). Frame size is bounded by the limits table (Volume 12); exceeding it at compile time is P07-C15; exceeding it at runtime is stack exhaustion → trap 134.

**d6 — Recursive calls.** Recursion is permitted without depth limit in the semantics; termination is the program's concern (§7.6 d9); stack exhaustion is the defined trap (§7.11.9); the compiler may not transform recursion in a way that changes observable behavior (G1/G4 obligations in Volumes 13–16).

**d7 — Tail calls.** Edition P1 defines no tail-call guarantee and no `become` keyword; a tail position is an ordinary call. Tail-call optimization is permitted as an optimization only where it preserves observable behavior including stack-exhaustion timing? Settle exactly: TCO is permitted only when the callee's frame drops nothing (no drop glue on exit path) and the transformation preserves all observable behavior except stack depth; stack-depth-dependent behavior (exhaustion point) is therefore not portable across optimization levels and programs must not rely on it (normative ban on relying on stack depth).

**d8 — Indirect calls.** Calls through function pointers and closures: the pointer's row is part of its type (§7.4 d2); the call applies the pinned call rule with that row. A null or dangling function pointer call is UB-4? Settle exactly: calling through a function pointer whose value is not the address of a function of the exact type (same ABI, same signature) is UB-4 (SPEC-INDEX §3: "calling a function through a wrong-ABI function pointer"); null is a wrong-ABI value (no function has a null address) hence UB-4.

**d9 — Generic calls.** Calls to generic functions monomorphize per SPEC-INDEX §3: depth limit 32, total-instance limit per Volume 12; the instance's semantics equal the generic's (obligation family OB-04); effect rows instantiate per §7.3.12.

**d10 — Dynamic calls.** Calls through trait objects: vtable slot per method (Volume 04 §6 layout); the static effect contract is the trait-declared row (§7.4 d6); the dynamic target must implement the trait (guaranteed by construction of the trait object).

**d11 — Variadic calls.** Variadics exist only in `extern "C"` declarations: `extern "C" fn f(x: T, args: …)`. Calling them requires `unsafe` (the `unsafe` atom). Post-fixed arguments must be C-ABI-compatible scalars: integer types of width ≤ 64, `f64`, `bool`, `char`, or pointers; passing any other type is P07-C16. Planck-ABI functions must not be variadic (P07-C16).

**d12 — Function pointer calls.** Same as d8 for `fn` pointers; the row travels with the type; a `fn` pointer to a function with a stronger row than the pointer type declares is a type error (Volume 04) — the pointer type's row is an upper bound at every call.

**d13 — Panic propagation.** A panic in the callee unwinds the callee frame and continues unwinding into the caller per §7.11 d5; the caller's row must include `panic` (call rule, §7.3.9 rule 3 — callee row includes panic when its body can panic; a callee whose body provably cannot panic has no `panic` atom and the caller needs none).

## 7.11 Panic and error handling (design.md §11.6)

**d1 — Result values.** `Result<T, E>` is the core enum `Ok(T) | Err(E)` (Volume 04); its control role is the target of `?` (d3) and `match`. No special evaluation rule beyond d3.

**d2 — Optional values.** `Option<T>` is `Some(T) | None`; same rule as d1; unwrapping methods are stdlib (Volume 19) with declared `panic` rows where documented.

**d3 — Propagation.** `e?` desugars exactly: for `Result<T, E>` in a function returning `Result<T', E'>`: `match e { Ok(v) => v, Err(err) => return Err(From::from(err)) }`; for `Option<T>` returning `Option<T'>`: `match e { Some(v) => v, None => return None }`. The `From` conversion must be total (it is a function call; its row joins). `?` in a function whose return type is neither the operand's type family nor `Result`/`Option` is P07-C17. `?` performs no other effect; its row is r_e (§7.3.9 rule 9).

**d4 — Panic.** `panic!("…")` and `panic!(expr)` start unwinding with a payload: the payload type is `Box<dyn Any>`-equivalent (Volume 04 `dyn Any`); string panics box a string slice value. The `panic` atom applies (§7.2.1). Panic starts at the panic site and unwinds (d5) unless the build is `panic = abort` (d5) or a hook/catch intervenes (d5, d7).

**d5 — Unwind.** Unwinding is the defined propagation of a panic to the enclosing catch boundary: it runs the drop sequence of every scope it exits (Volume 06 §6.7.6, §7.7 d12), in innermost-first order, then transfers control to the boundary. Catch boundaries in edition P1 are exactly: the thread root (panic terminates the thread, §8.2 Volume 08) and the unsafe intrinsic `catch_unwind`-equivalent (Volume 10) which returns `Result<T, Payload>`. Unwinding cannot be observed mid-flight except through drop execution order (normative, testable).

**d6 — Destructor behavior during unwinding.** Drops run during unwinding per d5. A panic raised inside a drop while unwinding is in progress must abort immediately: trap 134, no further drops, no second unwind. This is the "double panic" rule and it is normative for all builds (in abort builds every panic is already a trap, so the rule collapses to the same outcome).

**d7 — Panic hooks.** There is exactly one hook slot per process. `set_hook(f)` replaces the current hook (initial: the default hook, which writes `panic at <location>: <message>` to stderr); `take_hook()` restores the default. The hook runs once per panic before unwinding begins. Installing hook f requires the installer's ambient row to include f's declared row (call rule). Hooks must not panic (a panicking hook is the double-panic rule, d6).

**d8 — Error conversion.** Conversion is via `From` (d3); `From::from` must be total and pure relative to its declared row. No implicit conversions exist beyond d3's desugar; `From` impls are library-level (Volume 19).

**d9 — Panic boundaries.** In `panic = abort` builds, every panic is the trap (d4, §7.11.9); there is no unwinding and `catch_unwind` is a compile-time rejection P07-C18. In unwind builds, the boundaries are d5's list. Unwinding across an FFI boundary is banned: a Planck `extern "ABI"` function whose body contains a panic path must be rejected at compile time P07-C07 unless its body provably catches at the boundary (d5 intrinsic) — the check is the body's row: `panic ∈ row(body)` and no catch boundary intervenes before the ABI exit → P07-C07. In abort builds this check is vacuous (traps, not unwinds).

**d10 — Error traits.** `trait Error: Display + Debug { fn source(&self) -> Option<&dyn Error> { None } }` is the complete error trait in edition P1 (core method set exactly: `source`; `Display`/`Debug` supertraits provide rendering). Library error enums should implement it; the language attaches no semantics to it beyond trait checking.

**d11 — FFI unwind behavior.** Unwinding into foreign code is banned (d9). The `"C-unwind"` ABI is banned in edition P1 (declaring it is P07-C19). Foreign code calling into Planck through a declared callback that panics: the Planck side must install a catch boundary at the callback entry (the compiler inserts it for `extern "C"` callbacks declared with the `#[planck_callback]` attribute, Volume 10; without the attribute, a panic path in a callback body is P07-C07).

**d12 — Trap semantics (closed section).** A trap is a defined immediate process termination: exit code 134; all threads terminate; no destructors run; no unwinding; stderr receives one line `trap: <reason>` (exact format, deterministic, G14). The complete trap-reason list in edition P1, exactly 11 reasons: (1) division/remainder by zero; (2) invalid shift amount; (3) index out of bounds; (4) slice range reversal; (5) allocation limit exceeded; (6) stack exhaustion; (7) misaligned atomic access (Volume 08 §8.5); (8) atomic width unsupported at runtime dispatch (Volume 08 §8.5); (9) shared-counter overflow (Volume 06 §6.2 d3); (10) panic in `panic = abort` mode; (11) double panic in unwind builds (d6). Every trap site must exit 134 — uniformity obligation OB-07-9. Traps are not UB (SPEC-INDEX §3) and must not be recoverable.

## 7.12 Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-07-1 | join is associative: ∀ r₁,r₂,r₃ ∈ D₃: join(join(r₁,r₂),r₃) = join(r₁,join(r₂,r₃)), D₃ = {∅} ∪ singletons ∪ 2-subsets ∪ {A} (\|D₃\| = 38, triples = 54,872) | EXH (54,872 cases) | FV-0 | `math/crates/planck-effect-math` `tests/join_assoc.rs` | G15 |
| OB-07-2 | join is commutative: ∀ r₁,r₂ ∈ 2^A (256 rows): join(r₁,r₂) = join(r₂,r₁) | EXH (65,536 pairs) | FV-0 | same crate | G15 |
| OB-07-3 | join idempotent (∀r: join(r,r)=r) and identity ∅ (∀r: join(r,∅)=r) | EXH (256 + 256 cases) | FV-0 | same crate | G15 |
| OB-07-4 | meet distributes over join: ∀ r₁,r₂,r₃ ∈ D₃: meet(r₁, join(r₂,r₃)) = join(meet(r₁,r₂), meet(r₁,r₃)) | EXH (54,872 triples) | FV-0 | same crate | G15 |
| OB-07-5 | `≤` is a partial order: reflexive (256), antisymmetric (all 65,536 pairs: r₁ ≤ r₂ ∧ r₂ ≤ r₁ ⟹ r₁ = r₂), transitive (D₃ triples: 54,872) | EXH (256 + 65,536 + 54,872 cases) | FV-0 | same crate | G15 |
| OB-07-6 | Row unification terminates: the measure (unbound variables, unprocessed constraints) of §7.3.9 decreases lexicographically at every step | REVIEW (invariant named: case analysis over §7.3.9 steps 2a–2d shows each step either binds (first component −1) or consumes a constraint (second component −1); reason not machine-checkable: the argument is over arbitrary constraint graphs, not a finite domain; bounded instances are EXH-checked in the paired artifact) | FV-1 | `tools/fv/ob07_06.md` + `planck-effect-math` unification property tests (PROP paired with EXH over generated constraint sets ≤ 8 variables) | G11, G15 |
| OB-07-7 | Effect-checking soundness: for every accepted program, every call site's ambient row includes the callee's row at runtime execution paths, dyn dispatch never uses a row stronger than the trait's declaration, and every accepted body's runtime effect set is a subset of its checked row | REVIEW (invariants named: (i) call rule checked at every site (§7.3.9 rule 3); (ii) impl-conformance ⊆ (§7.4 d5); (iii) body-row ⊆ declared-row (§7.3.6); reason not machine-checkable: quantifies over all programs; runtime bridge: mismatch manifests only as a trap/panic already counted in the trap model, never as UB outside SPEC-INDEX §3's list) | FV-1 | `tools/fv/ob07_07.md` | G3, G15 |
| OB-07-8 | Evaluation-order determinism: for every generated expression over {binary ops, calls, assignments} with depth ≤ 3, the side-effect order of any conforming implementation equals the left-to-right order of §7.6 d1–d2, including panics in earlier operands | EXH (generated universe: all depth-≤ 3 trees over the 6-node grammar {literal, read, +, =, call, index} = 6 + 36 + 216 = 258 expressions; differential run against the reference interpreter) paired with PROP (deterministic xorshift64* PRNG, 10,000 samples, depth ≤ 5) | FV-0 | `tools/fv/ob07_08.py` + differential test corpus | G1, G13, G14 |
| OB-07-9 | Trap uniformity: every trap reason in the closed list of §7.11 d12 (11 reasons) exits with code 134 and writes exactly one `trap: <reason>` line | EXH (one program per trap reason; 11 programs) + REVIEW (invariant named: all trap sites route through the single runtime primitive `plkc_trap(reason)`; reason not machine-checkable at spec level: the runtime is Volume 18's implementation) | FV-1 | `tests/traps/` corpus (11 programs) + `tools/fv/ob07_09.md` | G13, G22 |
| OB-07-10 | Exhaustiveness checker correctness: the usefulness algorithm accepts a match iff the arm patterns cover the scrutinee type; no false rejection and no false acceptance | EXH (bounded universe: enums with ≤ 4 variants, ≤ 2 levels of nesting, patterns of depth ≤ 3, ≤ 6 arms = 4,096 generated matches, cross-checked against a brute-force witness search over the finite value space) + REVIEW (invariant named: usefulness recursion strictly reduces (constructor count, pattern depth), hence terminates and agrees with semantic coverage; reason: full type universes are infinite) | FV-1 | `tools/fv/ob07_10.py` + `tools/fv/ob07_10.md` | G3, G15, G17 |
| OB-07-11 | Const-evaluation totality: every const context either produces a value within MAX_CONST_EVAL_STEPS = 10,000,000 and MAX_CONST_EVAL_DEPTH = 512, or is rejected with P07-C05; const evaluation never emits a trap, never reads mutable statics, never UB | EXH (interpreter run against the const-test corpus: 500 programs covering limits, recursion, floats, panics-in-const) + REVIEW (invariant: the interpreter is a total function from (program, budget) to value-or-rejection; reason: corpus-bound, not universe-bound) | FV-1 | `tools/fv/ob07_11.py` + corpus | G13, G15 |

## 7.13 Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| ------------------------------ | ---------- |
| 10.1 Read | §7.2.1 |
| 10.1 Write | §7.2.1 |
| 10.1 Allocation | §7.2.1 |
| 10.1 Deallocation | §7.2.1 |
| 10.1 IO | §7.2.1 |
| 10.1 Atomic | §7.2.1 |
| 10.1 Synchronization | §7.2.1 |
| 10.1 Panic | §7.2.1 |
| 10.1 Suspension | §7.2.1 |
| 10.1 FFI | §7.2.1 |
| 10.1 Any additional required external effects | §7.2.1 (banned; atom set closed) |
| 10.2 Observable versus non-observable effects | §7.3.1 |
| 10.2 Memory-region effects | §7.3.2 |
| 10.2 Effect identities | §7.3.3 |
| 10.2 Effect dependencies | §7.3.4 |
| 10.2 Effect composition | §7.3.5 |
| 10.2 Effect inference | §7.3.6 |
| 10.2 Effect propagation | §7.3.7 |
| 10.2 Effect constraints | §7.3.8 |
| 10.2 Effect subtyping | §7.3.10 |
| 10.2 Effect polymorphism, if supported | §7.1.3, §7.3.11 (supported, via row variables) |
| 10.2 Effect specialization | §7.3.12 |
| 10.2 Effect erasure | §7.3.13 |
| 10.2 Unknown effect handling | §7.3.14 |
| 10.3 Function declarations | §7.4 d1 |
| 10.3 Function types | §7.4 d2 |
| 10.3 Closure effect types | §7.4 d3 |
| 10.3 Generic effect contracts | §7.4 d4 |
| 10.3 Trait method effects | §7.4 d5 |
| 10.3 Dynamic dispatch effect contracts | §7.4 d6 |
| 10.3 Foreign function effects | §7.4 d7 |
| 10.3 Async latent effects | §7.4 d8 |
| 10.3 Constructor effects | §7.4 d9 |
| 10.3 Destructor effects | §7.4 d10 |
| 10.3 Intrinsic effects | §7.4 d11 |
| 10.4 Reordering | §7.5 d1 |
| 10.4 Dead-effect elimination | §7.5 d2 |
| 10.4 Effect-preserving inlining | §7.5 d3 |
| 10.4 Call summaries | §7.5 d4 |
| 10.4 Memory dependency partitioning | §7.5 d5 |
| 10.4 Effect independence | §7.5 d6 |
| 10.4 Unknown effect barriers | §7.5 d7 |
| 10.4 Atomic ordering | §7.5 d8 |
| 10.4 IO ordering | §7.5 d9 |
| 11.1 Evaluation order | §7.6 d1 |
| 11.1 Operand evaluation order | §7.6 d2 |
| 11.1 Result type | §7.6 d3 |
| 11.1 Result category | §7.6 d4 |
| 11.1 Ownership behavior | §7.6 d5 |
| 11.1 Borrowing behavior | §7.6 d6 |
| 11.1 Effects | §7.6 d7 |
| 11.1 Panic conditions | §7.6 d8 |
| 11.1 Divergence conditions | §7.6 d9 |
| 11.1 Constant-evaluation rules | §7.6 d10 |
| 11.2 Blocks | §7.7 d1 |
| 11.2 If expressions | §7.7 d2 |
| 11.2 If statements | §7.7 d3 |
| 11.2 Match expressions | §7.7 d4 |
| 11.2 Match statements | §7.7 d5 |
| 11.2 While loops | §7.7 d6 |
| 11.2 For loops | §7.7 d7 |
| 11.2 Infinite loops | §7.7 d8 |
| 11.2 Break | §7.7 d9 |
| 11.2 Continue | §7.7 d10 |
| 11.2 Return | §7.7 d12, §7.10 d4 |
| 11.2 Yield | §7.7 d11 |
| 11.2 Early exits | §7.7 d12 |
| 11.2 Labeled control flow, if supported | §7.7 d13 (supported) |
| 11.2 Unreachable paths | §7.7 d14 |
| 11.3 Literal patterns | §7.8 d1 |
| 11.3 Identifier patterns | §7.8 d2 |
| 11.3 Wildcards | §7.8 d3 |
| 11.3 Reference patterns | §7.8 d4 |
| 11.3 Tuple patterns | §7.8 d5 |
| 11.3 Struct patterns | §7.8 d6 |
| 11.3 Enum patterns | §7.8 d7 |
| 11.3 Slice patterns | §7.8 d8 |
| 11.3 Or patterns | §7.8 d9 |
| 11.3 Binding patterns | §7.8 d10 |
| 11.3 Guards | §7.8 d11 |
| 11.3 Move bindings | §7.8 d12 |
| 11.3 Copy bindings | §7.8 d13 |
| 11.3 Reference bindings | §7.8 d14 |
| 11.3 Exhaustiveness | §7.8 d15 |
| 11.3 Reachability | §7.8 d16 |
| 11.3 Irrefutability | §7.8 d17 |
| 11.3 Pattern compilation | §7.8 d18 |
| 11.4 Arithmetic | §7.9 d1 |
| 11.4 Comparisons | §7.9 d2 |
| 11.4 Logical operations | §7.9 d3 |
| 11.4 Bitwise operations | §7.9 d4 |
| 11.4 Assignment | §7.9 d5 |
| 11.4 Compound assignment | §7.9 d6 |
| 11.4 Ranges | §7.9 d7 |
| 11.4 Casting | §7.9 d8 |
| 11.4 Dereferencing | §7.9 d9 |
| 11.4 Borrow operators | §7.9 d10 |
| 11.4 Operator overloading | §7.9 d11 |
| 11.4 Short-circuiting | §7.9 d3, d12 |
| 11.4 Overflow behavior | §7.9 d13 |
| 11.4 Division-by-zero behavior | §7.9 d14 |
| 11.4 Floating-point edge cases | §7.9 d15 |
| 11.5 Argument evaluation | §7.10 d1 |
| 11.5 Argument ownership transfer | §7.10 d2 |
| 11.5 Parameter initialization | §7.10 d3 |
| 11.5 Return-value semantics | §7.10 d4 |
| 11.5 Stack frame semantics | §7.10 d5 |
| 11.5 Recursive calls | §7.10 d6 |
| 11.5 Tail calls | §7.10 d7 (no guarantee in P1; settled) |
| 11.5 Indirect calls | §7.10 d8 |
| 11.5 Generic calls | §7.10 d9 |
| 11.5 Dynamic calls | §7.10 d10 |
| 11.5 Variadic calls | §7.10 d11 |
| 11.5 Function pointer calls | §7.10 d12 |
| 11.5 Panic propagation | §7.10 d13 |
| 11.6 Result values | §7.11 d1 |
| 11.6 Optional values | §7.11 d2 |
| 11.6 Propagation | §7.11 d3 |
| 11.6 Panic | §7.11 d4 |
| 11.6 Unwind | §7.11 d5 |
| 11.6 Abort | §7.11 d5, d9, d12 |
| 11.6 Panic hooks | §7.11 d7 |
| 11.6 Destructor behavior during unwinding | §7.11 d6 |
| 11.6 Error conversion | §7.11 d8 |
| 11.6 Error traits | §7.11 d10 |
| 11.6 Panic boundaries | §7.11 d9 |
| 11.6 FFI unwind behavior | §7.11 d11 |

bullets in scope: 127, defined: 127
