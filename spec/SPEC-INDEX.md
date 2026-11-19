# Planck — Master Specification Library Index

CEP:FILE: spec/SPEC-INDEX.md
CEP:WHAT: Index, method, and normative contracts for the 26-volume Master Specification Library of the Planck compiler; pins the architectural decisions and the shared math-kernel contracts that every volume and every extraction crate must obey.
CEP:WHY: design.md names 26 volumes as the consolidated deliverable and requires its section 43 decisions to be settled before the individual specifications are written; this file settles them and defines the definition method so all volumes are mutually consistent.
CEP:STATUS: complete
CEP:FAILURE: A volume that contradicts a pinned decision or contract here is non-conforming and must be corrected, not the contract silently bent.
CEP:ASSUMES: standards.md (CEP&CC 0.4) is the process/class standard; "verification standards.md" (G1–G26) is the verification standard; "doc standards.md" is the documentation standard; design.md is the roadmap whose every bullet must be defined.
CEP:COST: document; no runtime cost.
CEP:EVIDENCE: coverage ledgers in each volume file; verification matrix rows in each volume file.

---

## 0. Authority chain (read first)

1. `standards.md` (CEP&CC 0.4) governs classes, comments, naming, and failure discipline. Normative verbs ("must", "must not", "should", "may", "banned", "restricted") have the exact meaning of standards.md §3.
2. `verification standards.md` governs correctness: G1–G26. The verification matrix of every volume references these goals by ID.
3. `doc standards.md` governs document structure and prohibits marketing language, subjective adjectives, and undefined terms.
4. `design.md` is the roadmap. Every bullet in design.md must have exactly one normative definition home in this library. design.md itself is retained byte-for-byte; nothing is removed from it.
5. Where volumes and design.md disagree, the volume (this library) is the definition; design.md retains the requirement that motivated it. Where volumes and standards.md disagree, standards.md wins.

**Language identity (settled):** the language is **Planck**, version 0.1, edition P1. The compiler is the **Planck compiler** (working title `plkc`). This settles design.md §1.1 bullet 1.

## 1. Definition method (binding for every volume)

Every definition in this library satisfies all of the following:

1. **Exactness.** Constants have exact values with units. Enumerations enumerate every member. Grammars are given as productions, not prose. Algorithms state input domain, output, termination measure, and worst-case complexity. No "appropriate", "reasonable", "as needed", "e.g." governing normative content, no TBD, no "implementation-defined" without an exact contract.
2. **Normative verbs.** Per standards.md §3. A definition uses must / must not / may; rationale sentences are separated from rules.
3. **ATP-cast.** Every semantic rule is stated as a judgment, an invariant, or an obligation that a theorem prover or a total finite-domain test can check. Each obligation has an ID of the form `OB-<vol>-<n>` and a verification row in the volume's verification matrix.
4. **Fidelity (standards.md §54.4).** Models are bounded exactly as the implementation is bounded: machine integers are modeled with their exact widths and overflow behavior, memory with addresses and aliasing, concurrency with the memory model. No unbounded-math shortcuts. Where a proof is bounded, the bound is part of the obligation.
5. **No sorry analogues.** No `todo!()`, `unimplemented!()`, `unreachable!()` (unless proven impossible and documented per standards.md §25.3.1), no admitted obligations, no obligation rows marked "later". If a property cannot be discharged, it is stated as a named refinement gap with a runtime bridge, per §54.2.2 (class FV-1) — never omitted.
6. **Coverage ledger.** Every volume file ends with a coverage ledger: one row per design.md bullet in its scope, quoting the bullet (abbreviated) and naming the section in the volume where it is defined. The ledger is the completeness instrument; an absent row is a missing definition.
7. **Feature template.** Language features use the 16-row template of design.md §40. Optimization passes use the 10-row template of design.md §40. A row that does not apply states "not applicable because …"; it is never silently omitted.
8. **Doc structure.** Component-definition subsections follow `doc standards.md` §3: Abstract, Formal Definitions (with Rust signatures), Algorithm (with complexity), Invariants, Integration Points, Known Limitations. Structure correspondence (fixed by vote round 1, V2 finding 6): the volumes realize this structure as — Abstract ≙ the volume's `CEP:WHAT` + section overview; Formal Definitions ≙ the definition bullets `d1..dn` of each component section, with the Rust signatures carried by the kernel extraction home named in the volume's kernel-extraction table (`math/crates/...`), which is where §3.3's signature requirement is discharged; Algorithm ≙ the section's algorithm subsection with stated complexity; Invariants ≙ the invariant list (e.g. Volume 13 I1–I12); Integration Points ≙ the section's cross-volume reference block; Known Limitations ≙ the volume's stated-limitations rows and refinement gaps. Every element of doc standards §3 therefore has exactly one realized home in each volume; an element with no realized home is a doc-structure defect.
9. **No deletion.** Definitions extend and refine design.md; they never drop a design.md item. A design bullet judged wrong is still defined, with the definition and the consequence stated normatively.

## 2. The 26 volumes

Master library per design.md "Master specification library". Status: defined = volume file exists with complete coverage ledger.

| Vol | Title | File | Defines design.md | Status |
| --- | ----- | ---- | ----------------- | ------ |
| 01 | Language charter | `spec/volume-01-language-charter.md` | §1 | defined |
| 02 | Lexical and syntactic specification | `spec/volume-02-lexical-syntactic.md` | §2 | defined |
| 03 | Names and modules | `spec/volume-03-names-modules.md` | §3–§5 | defined |
| 04 | Type system | `spec/volume-04-type-system.md` | §6 | defined |
| 05 | Value and memory semantics | `spec/volume-05-value-memory.md` | §7–§8 | defined |
| 06 | Ownership and borrow checker | `spec/volume-06-ownership.md` | §9 | defined |
| 07 | Effects and execution semantics | `spec/volume-07-effects-execution.md` | §10–§11 | defined |
| 08 | Concurrency and async | `spec/volume-08-concurrency-async.md` | §12–§13 | defined |
| 09 | Compile-time programming | `spec/volume-09-compile-time.md` | §14 | defined |
| 10 | Attributes and low-level interfaces | `spec/volume-10-attributes-lowlevel.md` | §15–§17 | defined |
| 11 | Frontend architecture | `spec/volume-11-frontend.md` | §18 | defined |
| 12 | Compiler infrastructure | `spec/volume-12-infrastructure.md` | §19 | defined |
| 13 | IR 1 — Effect-typed Sea of Nodes | `spec/volume-13-ir1.md` | §20 | defined |
| 14 | IR 2 — Heterogeneous Sea of Nodes | `spec/volume-14-ir2.md` | §21 | defined |
| 15 | Machine IR | `spec/volume-15-machine-ir.md` | §22 | defined |
| 16 | Backend and targets | `spec/volume-16-backend-targets.md` | §23 | defined |
| 17 | Object generation and linking | `spec/volume-17-object-linking.md` | §24 | defined |
| 18 | Runtime | `spec/volume-18-runtime.md` | §25 | defined |
| 19 | Standard library | `spec/volume-19-stdlib.md` | §26 | defined |
| 20 | Packages and build system | `spec/volume-20-packages-build.md` | §27 | defined |
| 21 | Developer tooling | `spec/volume-21-tooling.md` | §28 | defined |
| 22 | Correctness and verification | `spec/volume-22-correctness-verification.md` | §29–§30 | defined |
| 23 | Performance engineering | `spec/volume-23-performance.md` | §31 | defined |
| 24 | Security and reliability | `spec/volume-24-security-reliability.md` | §32 | defined |
| 25 | Production and governance | `spec/volume-25-production-governance.md` | §33–§36 | defined |
| 26 | Optional execution products | `spec/volume-26-optional-products.md` | §37–§39 | defined |

Reviewed: 26 / 26.

## 3. Settled architectural decisions (design.md §43 — settled here, as required)

| Decision | Settled value |
| -------- | ------------- |
| Exact undefined behavior model | UB is enumerated, not open-ended: the only UB categories are (UB-1) reading uninitialized memory through a typed place, (UB-2) creating a reference outside its validity bounds, (UB-3) violating data-race freedom (two unsynchronized conflicting accesses, at least one non-atomic), (UB-4) calling a function through a wrong-ABI function pointer, (UB-5) inline-assembly blocks violating their declared clobber/condition lists, (UB-6) violating an intrinsic's declared precondition. Everything else is a defined value, a defined trap, or a compile-time rejection. Traps are defined terminations (abort with exit code 134), not UB. |
| Pointer provenance model | Provenance is an allocation ID + offset + capability set {load, store, compute} per pointer (CHERI-inspired, untagged). Provenance narrows on arithmetic, is preserved by copies, and is checked at dereference. Provenance is tracked in IR 1 memory SSA and is part of the UB-2 judgment. |
| Object and storage identity | Identity is address + allocation ID. Two objects are the same object iff same allocation ID and same offset. Distinct allocations never compare equal by identity even if addresses are reused after free (freed addresses are not observable through safe code). |
| Default layout guarantees | `#[repr(P)]` default: fields in declaration order, alignment = max field alignment, size = padded to that alignment, no reordering, no niche packing unless `#[repr(niche)]`. `#[repr(C)]`, `#[repr(packed)]`, `#[repr(transparent)]` are the only other layouts. Exact layout math is defined in Volume 05. |
| Destruction and unwinding model | Destruction runs scoped, innermost-first, at scope exit on every path. Unwinding exists only through `panic` (defined unwinding to the enclosing catch boundary) and `?`/`try` (defined value propagation). Panics start unwinding; `panic = abort` builds make the trap immediate. Drop cannot observe the panic payload. Unwinding across FFI is a compile-time rejection. |
| Trait coherence rules | Orphan rule: an impl is legal iff the trait or the self type is local to the crate, or at least one type parameter is a local type that appears before any foreign type parameter. No fundamental-type escape hatch in edition P1. Overlap rejection is semantic (two impls that could both apply to some instantiation are rejected). |
| Effect polymorphism | Effect rows are sets of effect atoms (algebra pinned in §5.3 below). Functions are effect-polymorphic over row variables with exact row unification. `async` is the effect atom `async`; `unsafe` operations carry the `unsafe` atom; panics carry the `panic` atom unless `panic = abort`. |
| Async and cancellation model | Structured concurrency: tasks form a tree; cancellation is cooperative via cancel checks at defined await points; a cancelled task must reach its scope exit; cancellation of a parent cancels children before the parent's join point completes. No detached tasks in edition P1. |
| Generic specialization strategy | Monomorphization only, with a monomorphization depth limit of 32 and a total-instance limit defined in Volume 12 (limits table). Specialization is by substitution; semantics of a specialized instance equal the semantics of the generic (OB-04 family obligation). |
| ABI and calling convention policy | Initial targets: x86-64 SysV ABI and AArch64 (AAPCS64), both with the Planck default convention = C ABI at FFI boundaries and the Planck fast convention internally (defined in Volume 10). All other targets are declared, not implemented, in edition P1. |
| Compiler verification target | Every compiler stage carries verification rows (G15 matrix). The trusted computing base is: the Planck math kernel (`math/`), the Z3 scripts (`tools/fv/`), and the review process; the kernel and scripts are FV-checked per §54. MATP (github.com/compilecomp/MATP) is the project ATP used for search and for discharging first-order obligations where its theory applies. **Amended by CEP-DR-01:** Rocq is the primary proof assistant (constructive layer, `rocq/`), Peregrine is the first proof-to-Rust extraction route (untrusted until OB-DR-1..4 discharge), MATP is retained as complementary FO prover; see `spec/CEP-DR-01-rocq-peregrine-toolchain.md` for the layered TCB and the five qualification obligations. |
| Bootstrap trust model | Stage 0 compiler (written in Rust, verified kernel math) is trusted; stage 1 (Planck-compiled Planck) is verified against stage 0 outputs by semantic equivalence checking (differential + certificate comparison); the chain is documented in Volume 22. |

## 4. Verification scheme (verification standards.md G1–G26 + standards.md §54)

1. Every volume has a **verification matrix**: columns = Obligation ID, Property (exact statement), Source bullets, Method, Class, Artifact, Status.
2. Method is one of:
   - `Z3`: bounded SMT check, script in `tools/fv/`, strict timeout 10 s, documented in the script header; a timeout or `unknown` marks the obligation unproven (build fails).
   - `EXH`: exhaustive test over the full finite input domain stated in the obligation (test-as-proof; the domain statement is part of the obligation).
   - `PROP`: property test with deterministic PRNG over a stated sampling domain; must be paired with either Z3 or EXH for the same property, or an explicit refinement gap.
   - `REVIEW`: reviewer-checkable argument for properties outside mechanical reach; must name the invariant and the reason it is not machine-checkable.
   - `ROCQ` (added by CEP-DR-01 rule DR-1.5): constructive proof kernel-checked by the pinned Rocq version; the row must name the exact version, the file under `rocq/`, and the compiled `.vo` check command.
   - `DIF` (added by CEP-DR-01 rule DR-1.5): differential equivalence — execution of the artifact under test against the hand-written `math/` kernel reference over a stated domain, paired with EXH or Z3 per the pairing rule above; used for untrusted extraction output (CEP-DR-01 §4 layer 2). Instrument (CEP-DR-01 §8, DR-8.3): `tools/extractor dif` — the oracle is the Coq kernel itself (`vm_compute` ground truth over the complete finite domain via the generated `rocq/Effect/DifGroundTruth.v`), the artifact under test is the mechanically extracted Rust under `extract/`, and evidence lands in `tools/fv/extract/artifacts/`.
3. Class per §54: FV-0 (machine-checked, zero gaps), FV-1 (bounded with documented gap + runtime bridge), FV-2 (verification tooling).
4. `math/` crates carry `CEP:FV-*` comment fields exactly per §54.7.1 and the `CEP:` block per §10.2.
5. G-mapping: each verification matrix names the G-goals it serves (G1 semantic preservation, G2 memory safety, G3 type soundness, G4/G5 pass equivalence, G11 termination, G13/G14 determinism and limits, G15 matrix completeness, G16 TCB, G17 counterexamples, G18 independence, G21 concurrency, G22 diagnostics, G26 evolution).

## 5. Pinned math-kernel contracts (authoritative for `math/` crates and for volumes)

These contracts are normative for both the specification volumes and the Rust extraction. A volume that redefines them divergently is non-conforming.

### 5.1 Kernel crates and equivalence claim

| Crate | Math it extracts | Volumes |
| ----- | ---------------- | ------- |
| `planck-charclass` | Source character classification predicates (exact ASCII classes; Unicode policy from Volume 02) | 02 |
| `planck-token-math` | Identifier grammar, integer/float literal value spaces, radix parsing with range checks | 02 |
| `planck-operator-math` | Operator table: precedence, associativity, arity, overload class | 02 |
| `planck-effect-math` | Effect algebra: atoms, rows, join, meet, row subtyping, unification | 07 |
| `planck-ownership-math` | Ownership/loan state machine and conflict predicate | 06 |
| `planck-graph-math` | Graph math: topological order, dominator computation, cycle detection | 13, 16 |
| `planck-regalloc-math` | Live-interval interference, coloring bound, spill priority | 16 |
| `planck-reloc-math` | Relocation field extraction/patching with exact overflow checks | 17 |

**Equivalence claim form (per crate, per function).** For every public function `f`, the crate documents: the mathematical definition it refines (volume + section anchor), the input domain (exact, with bounds), the failure set, and the certificate:
- `EXH`: an exhaustive test enumerating the entire domain (stated size ≤ 10^6 cases), witnessing equivalence of implementation and definition on the whole domain. This is a complete proof for that domain.
- `Z3`: a bounded machine check with the bound stated; equivalence outside the bound is a named refinement gap with a runtime bridge (domain check before use).
- `PROP`: deterministic property test; always paired per §4.3.
No function claims unbounded equivalence. "Mathematically equivalent under all workloads" is therefore exact: equivalent over the whole defined input domain, and every workload is inside a defined domain because limits are part of the specification (G13).

### 5.2 Kernel implementation law (CEP&CC by the letter)

- Hot classes: every kernel function is CEP-0 (pure math, no allocation after init). Required: `core::` only, no `std` in crate code (`#![no_std]` with `core` imports; test builds may use `std`), no `Box`/`Vec`/`String`/`format!`/`panic!`/`unwrap`/`expect`/`todo!`/`unimplemented!`/dynamic dispatch/threads in non-test code.
- Every failure returns an explicit error enum; every error enum is complete; `CEP:FAILURE` lists every variant.
- Every public item carries the CEP block (§10.2) and, where verified, the FV block (§54.7.1).
- Constants are named, never magic literals (Law 7); bounds are named constants with exact values.
- Tests live in the same crate (`#[cfg(test)]`) and under `tests/`; tests are CEP-2 and may allocate.
- Zero external dependencies (offline build); test PRNG is a pinned xorshift64* implementation in-crate.

### 5.3 Effect algebra (pinned; Volume 07 elaborates)

- Effect atoms: `io`, `alloc`, `panic`, `unsafe`, `async`, `concurrency`, `ffi`, `nondet`. Atom set is closed; adding an atom is an edition change.
- A row is a finite subset of the atom set, plus a row variable flag (open rows for effect polymorphism). Canonical form: sorted, deduplicated.
- `join(r1, r2) = r1 ∪ r2` (set union). `meet(r1, r2) = r1 ∩ r2`. Row subtyping for effect *requirement* positions: `r1 ≤ r2 ⟺ r1 ⊆ r2` (demanding fewer effects is allowed where a row is required).
- Function call rule: calling `f : (row ρ) → τ` requires the caller's row to include ρ.
- Laws (each an obligation): join is associative, commutative, idempotent, with identity ∅; meet distributes over join; subtyping is a partial order (reflexive, antisymmetric, transitive). All laws EXH over the full 2^8 atom lattice (256 rows; pairwise checks bounded per-law domain, stated per obligation).

### 5.4 Ownership state machine (pinned; Volume 06 elaborates)

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

### 5.5 Operator table (pinned columns; Volume 02 gives the full table)

Columns: operator token, precedence level (integer, higher binds tighter), associativity (Left, Right, None), arity (1, 2), overloadable (bool), overflow behavior class (Panic, Wrap, Unchecked, N/A). The full table with exact rows is Volume 02 §2.4; `planck-operator-math` embeds exactly that table as a const array and exposes lookup with binary search; equivalence EXH over the table domain.

### 5.6 Literal math (pinned; Volume 02 elaborates)

- Integer literals: arbitrary-precision value `v`, radix `r ∈ {2, 8, 10, 16}`, digit string `d` with digits valid for `r`; value = fold over digits; typed literal is well-formed iff `min(T) ≤ v ≤ max(T)` for the inferred integer type `T`; overflow at literal level is a compile-time rejection (never a runtime event).
- Float literals: decimal strings map to the nearest IEEE-754 binary64/binary32 value, round-to-nearest, ties-to-even; NaN payloads and infinities are named forms, not literal spellings (spelled via constants), per Volume 02.
- Char literals: exactly one Unicode scalar value; surrogates rejected.
- All parsing functions are total: `(input) → Result<Value, LiteralError>`; error enum complete and exhaustive-tested over invalid digit sets and range boundaries.

### 5.7 Graph math (pinned; Volumes 13 and 16 elaborate)

- Graphs are static arrays: nodes `0..n`, edges as CSR pairs; `n` bounded by the compiler limits table (Volume 12).
- Topological order: Kahn's algorithm with stable tie-break by ascending node ID (determinism, G14); failure on cycle = error, never a partial order.
- Dominators: iterative dataflow over reverse postorder; entry dominates itself; result unique for reducible and irreducible graphs; termination measure = ≤ n passes with early exit (G11 obligation).
- Cycle detection: three-color DFS, iterative, bounded stack; reports one cycle as a node list.

### 5.8 Register allocation math (pinned; Volume 16 elaborates)

- Live intervals: `(v, start, end)` with `start ≤ end`, half-open `[start, end)`.
- Interference: two intervals interfere iff they overlap and at least one is not coalescible with the other (coalescibility given by a caller-provided predicate table).
- Coloring: degree-ordered allocation with First-Fit over the register class order; register pressure = max simultaneous live count; spill priority = `uses × 10^3 + loop_depth × 10^6 − size_cost`, exact integer math with checked multiply (u64); spills chosen in ascending benefit order until pressure ≤ available registers.
- Laws: allocation assigns each interval exactly one location; no two interfering intervals share a location (Z3 obligation on bounded instances + EXH on generated small instances).

### 5.9 Relocation math (pinned; Volume 17 elaborates)

- Relocation entry: `(offset: u64, kind: RelocKind, symbol: SymId, addend: i64)`.
- Kinds (initial set, exact): `Abs64`, `Abs32`, `PcRel32`, `PcRel16`, `ArmAdrp`, `ArmBranch26`, `X86RipRel32`.
- Patch math: field extraction by `(offset, width, bit_lower)` with mask math in checked arithmetic; overflow or range violation returns `RelocError` (complete enum); PC-relative range checks are exact (e.g., `PcRel32` requires `−2^31 ≤ target − site ≤ 2^31 − 1` after addend).
- Endianness: explicit per-target parameter; no host-endian assumptions (Law 7).

### 5.10 Character classes (pinned; Volume 02 elaborates)

Exact ASCII predicates (total, `u8 → bool`): whitespace (U+0009, U+000A, U+000D, U+0020), decimal digit, hexadecimal digit, identifier-start (ASCII letters plus `_`), identifier-continue (ASCII alnum plus `_`), line-comment introducer (`//`), block-comment introducer (`/*`). Non-ASCII policy is exactly Volume 02 §2.2 (edition P1: non-ASCII permitted in string/char literals and comments only; identifiers are ASCII in edition P1).

## 6. Repository layout this pass creates

```
spec/                       26 volume files + this index
math/                       Rust workspace (no_std kernels)
  Cargo.toml                workspace manifest, members, profile
  crates/<crate>/           one crate per §5.1 row
tools/fv/                   Z3 obligation scripts (FV-1/FV-0 artifacts);
                            extract/ artifacts record the DIF runs
                            (tools/fv/extract/artifacts/, CEP-DR-01 §8)
rocq/                       Rocq foundation layer (CEP-DR-01 DR-7.4):
                            proofs, pinned-version file, build.sh;
                            subset-report and rustc-assumptions land here
                            as OB-DR-2/OB-DR-4 artifacts
tools/extractor/            planck-extractor: mechanical Gallina-subset ->
                            Rust extractor (FV-2, CEP-DR-01 §8 DR-8.1);
                            emits extract/ and the DIF kernel oracle
extract/                    SEPARATED mechanically extracted Rust tree
                            (CEP-DR-01 §8 DR-8.2); generated only,
                            hand edits are non-conforming; carries the
                            kernel ground truth it was verified against
```

design.md is not modified by this pass (nothing may be removed from it). The review counter it contains ("0 / 26 reviewed") is superseded by the table in §2 above.

## 7. Implementation dependency order and completion gates (defines design.md §41–§42)

Added by the vote-round-1 completeness fix (V1 findings 1–2): design.md §41 and §42 must have a definition home. This section is it. design.md is retained byte-for-byte; the quoted rows below are defined, never replaced.

### 7.1 Implementation dependency order (defines design.md §41)

design.md §41 recommends writing the specification in dependency order, "not implementation convenience", and fixes an 8-step development sequence. The normative definition:

- DO-1 (definition home): the Master Specification Library is written and revised in exactly this order: (1) foundation and semantic definitions = Volumes 01–10; (2) frontend and compiler infrastructure = Volumes 11–12; (3) first IR and verifier = Volume 13; (4) optimization architecture = Volume 14; (5) machine backend = Volumes 15–17; (6) runtime and standard library = Volumes 18–19; (7) production toolchain = Volumes 20–21; (8) correctness and production maturity = Volumes 22–26.
- DO-2 (verification obligation, G15): the coverage ledger of each volume must cite the DO-1 phase it belongs to; a volume revision that moves a definition out of its phase order must state the dependency it breaks.
- DO-3 (scope boundary, quoting design.md §41 by the letter): this is "the dependency order for specifications, not a rule that all implementation must occur sequentially. Some implementation work can proceed in parallel once its interfaces and invariants have been defined."

### 7.2 Completion gates (defines design.md §42)

design.md §42 fixes 11 production-completion gates. Each gate is a milestone with a required result; the gate is open until the result is machine-checkable or review-checkable per the volume that owns it, and the owning volume's verification matrix must carry one row per gate it owns (G15). Gate ownership and required results:

| Gate | Required result (design.md §42, verbatim) | Owner volume | Gate evidence |
| ---- | ----------------------------------------- | ------------ | ------------- |
| Language definition complete | Every stabilized construct has normative semantics | 01–10 | coverage ledgers complete for the stabilized surface (G15, G19) |
| Frontend complete | Valid programs receive complete semantic validation | 11–12 | Volume 11/12 verification matrices |
| Borrow checker complete | Ownership and lifetime invariants enforced | 06 | Volume 06 matrix + `planck-ownership-math` certificates |
| IR 1 complete | Typed, effect-aware graph construction and verification | 13 | Volume 13 matrix + `planck-graph-math` certificates |
| IR 2 complete | Representation transformations verified | 14 | Volume 14 matrix (G4/G5) |
| Backend complete | Correct executable generation for supported targets | 15–17 | Volume 15–17 matrices + `planck-regalloc-math`/`planck-reloc-math` certificates |
| Runtime complete | Hosted and/or freestanding execution contracts work | 18 | Volume 18 matrix (G20) |
| Standard library complete | Foundational APIs are usable and documented | 19 | Volume 19 matrix |
| Toolchain complete | Build, debug, test, format, and package workflows work | 20–21 | Volume 20/21 matrices |
| Validation complete | Conformance, fuzzing, regression, and differential suites established | 22 | Volume 22 matrix |
| Production ready | Release, security, compatibility, and maintenance processes established | 24–25 | Volume 24/25 matrices |

- CG-1 (gate discipline, quoting design.md §42 by the letter): "A compiler can be usable before every milestone is complete. But it should not claim support for a feature until its semantics and implementation are actually covered." A feature claim without its gate evidence is non-conforming.
- CG-2 (bootstrap relation): the stage-0/stage-1 bootstrap chain of §3 (bootstrap trust model) is evidence for the "Backend complete" and "Production ready" gates (G25).

## 8. Review status


- Definition pass: 26 / 26 volumes defined (see §2).
- Math extraction: 8 crates in `math/`, certificates per §5.1.
- FV artifacts: `tools/fv/` scripts with documented timeouts.
- Toolchain decision: CEP-DR-01 (Rocq + Peregrine + MATP) recorded and integrated; OB-DR-1..5 all open.
- Five-agent vote: **5/5 COMPLETE_AND_OPTIMAL** (round 3, unanimous; rounds 1-2 drove the fixes). Scope: 26 volumes + SPEC-INDEX + CEP-DR-01 + math crates + FV artifacts + Rocq scaffold. Record: `spec/VOTE-RECORD.md`.
