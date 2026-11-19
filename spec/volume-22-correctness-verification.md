CEP:FILE: spec/volume-22-correctness-verification.md
CEP:WHY: design.md §29–§30 (lines 3362–3652) demands an explicit, layered correctness strategy and a dedicated formal-verification workstream; this volume is the keystone that maps the G1–G26 goals to concrete compiler mechanisms, owns the proof-obligation registry, the differential and fuzzing protocols, the TCB, and the bootstrap chain.
CEP:WHAT: Normative specification of compiler correctness and verification: conformance suite structure, per-pass test gates, IR verification, differential testing against the reference interpreter, grammar-aware fuzzing with an enumerated mutation-operator set, property-based testing, runtime validation, backend testing, the five-level verification ladder, formal language/IR semantics anchors, transformation-proof dossiers, translation-correctness chain, TCB enumeration, proof infrastructure, verified bootstrap, counterexample minimization (G17), independent verification (G18), and verified evolution (G26).
CEP:STATUS: complete
CEP:FAILURE: A claimed-verified property without a registry row and artifact is a lie (FV prime law, standards.md §54.1, Severity 0); a pass in the default pipeline without its §22.3 test gate is a Severity 0 defect; a bootstrap stage promoted without the §22.15 comparisons invalidates the trust chain.
CEP:ASSUMES: SPEC-INDEX.md §1 (method), §3 (settled decisions: TCB, bootstrap trust model, UB-1..UB-6, provenance), §4 (verification scheme), §5 (pinned kernel contracts); "verification standards.md" G1–G26; standards.md §3, §54; Volumes 02 (grammar), 04–10 (semantics obligations), 12 (limits), 13–15 (IRs), 16–18 (backend/runtime), 19 (stdlib), 20 (build determinism), 21 (tooling).
CEP:COST: Verification tooling is CEP-2 (offline, deterministic); the reference interpreter and fuzz harness are CEP-1/CEP-2; no runtime cost except the debug-mode checks owned by Volumes 05/18.
CEP:EVIDENCE: Verification matrix (OB-22-1..OB-22-28), the proof-obligation registry (§22.16), and the coverage ledger (bullets in scope: 166, defined: 166; plus 2 prose directives, defined) in this file.

---

## 22.0 Scope and traceability

This volume defines design.md **§29 Compiler correctness strategy** (lines 3362–3525, 102 bullets) and **§30 Formal verification** (lines 3526–3652, 64 bullets). Bullets in scope: 166, defined: 166; the two prose directives of §29.6 ("build an end-to-end program fuzzer") and §30.6 ("decide whether the language specification is a formal executable model …") are settled normatively in §22.5 and §22.14 respectively and ledgered as prose rows. The §29 prose claims "'All tests pass' is not a correctness argument" and "You need multiple layers of validation" are honored by the G1–G26 mechanism map below: tests are evidence, obligations are the claims, and the matrix binds them.

### 22.0.1 G1–G26 → concrete mechanism map

| Goal | Concrete mechanism in the compiler |
| --- | --- |
| G1 source→target equivalence | Differential testing §22.4 (interpreter vs C0 vs C1) + translation chain §22.12 + bootstrap equivalence §22.15 |
| G2 memory safety | Provenance/validity checks (Volumes 05/18) + sanitizers §22.7 + UB-1..UB-6 closure (§22.9) |
| G3 type soundness | Preservation/progress obligations anchored in §22.9; Z3 over bounded step relations |
| G4 pass correctness | Per-pass dossiers §22.11 + test gate §22.3 |
| G5 equivalence after every optimization | IR verifier §22.2 after every pass in verification mode + per-pass rows + pipeline differential |
| G6 performance non-regression | Volume 23 §23.6–§23.10 measurement and gates (referenced, not redefined) |
| G7 optimization effectiveness | Volume 23 §23.9 utility function U(T) |
| G8 pass ordering and interaction | Interaction tests §22.3 + pipeline order fixed in Volumes 13/14 |
| G9 code generation quality | Backend tests §22.8 + independent machine-code validation §22.18 |
| G10 target-specific cost models | Volume 23 §23.7 per-target cost models |
| G11 termination | Per-phase termination measures and limits (Volumes 12/13/14/16) + resolver bound (Volume 20) |
| G12 compiler memory safety | Rust stage 0 + limits (Volume 12) + fuzzing §22.5 over adversarial inputs |
| G13 resource safety | Volume 12 limits table enforced everywhere; harness bounds §22.5, §22.17 |
| G14 determinism | Volume 20 §20.4 contract + byte-identity obligations here (OB-22-6, OB-22-14) |
| G15 proof completeness | Registry §22.16 + matrix template §22.19; no claim without a row |
| G16 TCB minimization | §22.13 enumeration and trust argument |
| G17 counterexamples | Divergence reporting §22.4 + delta debugging §22.17 |
| G18 independent verification | Artifact replay, independent object reader, cross-validation §22.18 |
| G19 complete language semantics | §22.9 anchors to Volumes 01–10 + fragment gate for the executable model |
| G20 runtime correctness | Volume 18 obligations + runtime validation §22.7 |
| G21 concurrency correctness | Concurrency semantics §22.9 + stress protocol §22.7 |
| G22 diagnostics correctness | Compile-fail diagnostics testing §22.1/§22.2 + parity obligations (Volume 21 OB-21-12) |
| G23 incremental correctness | Volume 20 §20.5 OB-20-9 (full-build byte equivalence) |
| G24 linking correctness | §22.8 linker/relocation tests + Volume 17 obligations |
| G25 self-hosting correctness | §22.15 verified bootstrap chain |
| G26 verified evolution | §22.20 versioning, dependency tracking, invalidation, replay |

## 22.1 Conformance test suite (defines design.md 29.1 bullets B29-1..B29-18 and 29.2 bullets B29-19..B29-30)

**Suite structure.** The conformance suite lives in `tests/conformance/<area>/<case-id>.plk`. Every case carries directive comments (closed set): `// @expect: ok` (must compile), `// @expect-error: <CODE>` (must fail; code exact), `// @run` (compile and run; exit 0), `// @run-exit: <n>` (exit code n required), `// @stdout: <line>` (each directive line is one expected stdout line, in order), `// @trap` (exit 134 required), `// @target: <triple>` (restriction), `// @rule: <volume-§>` (traceability anchor; ≥ 1 mandatory per case). A case with no `@rule` is a suite defect. Suite execution is `plkc test --conformance`; all cases must pass in CI (must).

**Language conformance areas** (B29-1..B29-18). Each area is a directory with a defined minimum case count and required coverage: every MUST rule of the anchored volume that admits a source-level witness has ≥ 1 positive case; every defined rejection has ≥ 1 negative case asserting the diagnostic code.

| Area (bullet) | Anchored volume | Minimum cases |
| --- | --- | --- |
| B29-1 Lexical tests | Volume 02 §2.2 | 128 |
| B29-2 Parsing tests | Volume 02 §2.3 | 256 |
| B29-3 Grammar tests | Volume 02 §2 productions | 128 |
| B29-4 Name resolution tests | Volume 03 | 128 |
| B29-5 Type checking tests | Volume 04 | 256 |
| B29-6 Generic tests | Volume 04 §6 | 128 |
| B29-7 Trait resolution tests | Volume 04 §6 | 128 |
| B29-8 Inference tests | Volume 04 | 96 |
| B29-9 Ownership tests | Volume 06 | 192 |
| B29-10 Borrow checking tests | Volume 06 | 128 |
| B29-11 Lifetime tests | Volume 06 | 96 |
| B29-12 Effect tests | Volume 07 | 96 |
| B29-13 Pattern tests | Volume 05/07 | 96 |
| B29-14 Constant evaluation tests | Volume 09 | 64 |
| B29-15 Async tests | Volume 08 | 64 |
| B29-16 FFI tests | Volume 10 | 48 |
| B29-17 Unsafe tests | Volume 10 §16 | 64 |
| B29-18 ABI tests | Volume 10 §17 | 48 |

Minimum total = 2144 cases. Counts are floors; growth is append-only.

**Compile-pass and compile-fail canonical pairs** (B29-19..B29-30). Every semantic rule needs positive and negative examples; the following twelve canonical pairs are mandatory named cases (exact case ids in `tests/conformance/`), each with the stated expectation:

| Case id | Bullet | Expectation |
| --- | --- | --- |
| cf-move-ok | B29-19 Valid move | compiles; runtime moves value once |
| cf-double-move | B29-20 Invalid double move | `@expect-error` ownership rejection |
| cf-shared-borrows | B29-21 Valid shared borrows | compiles; concurrent reads permitted |
| cf-conflicting-mut-borrow | B29-22 Invalid conflicting mutable borrow | `@expect-error` |
| cf-partial-init-ok | B29-23 Valid partial initialization | compiles; reads of initialized fields only |
| cf-uninit-field-use | B29-24 Invalid use of uninitialized fields | `@expect-error` |
| cf-reborrow-ok | B29-25 Valid reborrow | compiles |
| cf-ref-escape | B29-26 Invalid reference escape | `@expect-error` |
| cf-effect-ok | B29-27 Valid effect contract | compiles; row satisfied |
| cf-effect-bad | B29-28 Invalid effect contract | `@expect-error` effect rejection |
| cf-trait-impl-ok | B29-29 Valid trait implementation | compiles; impl applies |
| cf-coherence-overlap | B29-30 Invalid coherence overlap | `@expect-error` overlap rejection |

**Diagnostics are tested, not just success** (design.md §29.2 prose). Every negative case asserts the exact diagnostic code; suite execution fails on: wrong code, missing diagnostic, extra unasserted diagnostic, or a compiling negative case. This implements G22 at the language level (transport-level G22 is Volume 21 OB-21-12).

## 22.2 IR verification tests (defines design.md 29.3, bullets B29-31..B29-40)

For every IR (IR 1, IR 2, Machine IR): the in-process **IR verifier** is an executable checker of that IR's valid-graph invariants (Volumes 13 §20, 14 §21, 15 §22). It runs after every pass when `-C verify-ir=on`; the default is on in the dev profile and in all CI verification jobs; release builds default off (cost). A verifier failure is a Severity 0 defect that halts the build and produces a counterexample (§22.17).

The ten test classes below are corpus directories `tests/ir/<ir>/<class>/` with a floor of 8 cases per class per IR (240 cases total across the three IRs):

| Bullet | Class | Pass criterion |
| --- | --- | --- |
| B29-31 Valid graph tests | valid-graph | verifier accepts every case |
| B29-32 Invalid graph tests | invalid-graph | verifier rejects with the named invariant id |
| B29-33 Type consistency tests | type-consistency | verifier's type check accepts; negative cases rejected |
| B29-34 Control-flow tests | control-flow | dominator/post-dominator facts match Volume 13 math |
| B29-35 Memory SSA tests | memory-ssa | memory SSA invariants (provenance, dominance) hold |
| B29-36 Effect dependency tests | effect-dep | effect rows consistent per Volume 07 rules |
| B29-37 Ownership metadata tests | ownership-meta | loan metadata consistent with Volume 06 states |
| B29-38 Representation consistency tests | repr-consistency | representation selections satisfy Volume 14 §21 rules |
| B29-39 Machine constraint tests | machine-constraint | MIR satisfies target constraints per Volume 15 §22 |
| B29-40 Serialization tests | serialization | serialize∘deserialize = identity on all IR corpora, and serialization is byte-deterministic (G14) |

Pipeline-preservation property: for the optimization corpus, verifier(input) passes ⇒ verifier(output) passes after every pass (OB-22-4), which also identifies the exact pass that introduced an invalid result (G5 acceptance criterion).

## 22.3 Optimization correctness tests (defines design.md 29.4, bullets B29-41..B29-55)

**Per-pass test gate.** Every optimization pass has the directory `tests/opt/<pass>/` containing exactly the seven named cases:

| Bullet | File | Property |
| --- | --- | --- |
| B29-41 | `precondition.plk` | A program violating the pass precondition is rejected before the pass runs (verifier/precondition check fires) |
| B29-42 | `applies.plk` | The pass performs its transformation on a program where its precondition holds (remark emitted, B28-60) |
| B29-43 | `must-not.plk` | A program where the pass must not optimize is left semantically and structurally unchanged |
| B29-44 | `interaction.plk` | Composition with the preceding and following pipeline pass is semantics-preserving (differential per §22.4) |
| B29-45 | `edge.plk` | The stated edge case (limits, zero-size, maximal values per the pass's domain) is handled per spec |
| B29-46 | `equivalence.plk` | Differential equivalence: interpreter = C0 = C1 on this program (§22.4 protocol) |
| B29-47 | `regression.plk` | The pinned historical bug's input is retained and must keep passing |

Gate: a pass may be added to the default pipeline only when all seven cases exist and pass and its dossier (§22.11) is registered. No unverified optimization silently enters the trusted compilation pipeline (G5).

**Allocation elimination named scenarios** (B29-48..B29-55). The allocation-elimination pass additionally carries the eight mandatory scenarios in `tests/opt/alloc-elim/`, each normative (must / must not):

| Bullet | Scenario | Normative rule |
| --- | --- | --- |
| B29-48 | Unobservable allocation | An allocation whose address never escapes and whose deallocation has no observable effect may be eliminated (must) |
| B29-49 | Address-observable allocation | Address compared, stored, or used in identity → elimination must not apply |
| B29-50 | Foreign pointer escape | Address passed across an FFI boundary → elimination must not apply |
| B29-51 | Custom allocator effects | Allocation through an allocator with observable effects → elimination must not apply |
| B29-52 | Allocation failure behavior | Programs where allocation failure is observable → elimination must preserve the defined failure behavior or be forbidden |
| B29-53 | Destructor side effects | Types with effect-carrying drop → the drop must not be eliminated |
| B29-54 | Object identity | Identity comparisons on the allocation (allocation-id semantics per SPEC-INDEX §3) → elimination must not apply |
| B29-55 | Pinned objects | Pinned allocations → elimination and movement must not apply |

## 22.4 Differential testing (defines design.md 29.5, bullets B29-56..B29-59)

**Reference implementations** (B29-56..B29-59), exactly four:

| Bullet | Implementation | Role |
| --- | --- | --- |
| B29-56 | Reference interpreter I | Executable operational semantics of the defined core fragment (§22.9); the oracle |
| B29-57 | Unoptimized compiler mode C0 | `-O0`, `verify-ir=on`; baseline translation |
| B29-58 | Optimized compiler C1 | `-O2` default pipeline |
| B29-59 | Separate reference backend | `tools/ref-backend/`: an independent MIR→artifact path used only for cross-validation of encodings (CEP-2) |

**Fragment gate.** Differential testing applies to programs satisfying all of: effect row ⊆ {`io`, `alloc`, `panic`}; no `unsafe`, FFI, or inline assembly; deterministic (no `nondet`, no `concurrency` atoms); within interpreter bounds (recursion depth ≤ 512, heap ≤ 64 MiB, execution steps ≤ 10^7). Programs outside the gate are excluded from the oracle comparison by a documented gate check; exclusion is recorded, never silent.

**Protocol (exact).** For each corpus program P and input vector v: run I(P, v), C0(P, v), C1(P, v). Observable behavior = the pair (outcome, stdout bytes), where outcome is the exact process exit code (0..=255; 134 = defined trap) and stdout is the exact byte sequence. Equivalence: I ≡ C0 ≡ C1 on the pair, pairwise for (I, C0), (I, C1), (C0, C1). stderr is diagnostics and is not compared. Any disagreement is a **divergence**: the harness records the first point of divergence (smallest common prefix of observable events that differ), the violated property (oracle equality), and produces a minimized counterexample (§22.17). Divergences are Severity 0 findings; a divergence class fixed once becomes a `regression.plk` case (B29-47). The corpus = the gated subset of the conformance suite plus generator output (§22.5, class `well-formed`), ≥ 10^4 programs at adoption. The interpreter is verification tooling (FV-2): a wrong interpreter can only produce false alarms (investigated divergences), never validate a wrong compiler, because no compiler claim rests on interpreter verdicts — compiler claims rest on the verification matrix.

## 22.5 Fuzzing (defines design.md 29.6, bullets B29-60..B29-74 and the prose directive B29-75)

**Fuzzer targets** (B29-60..B29-74): one harness per component, closed list: lexer, parser, AST construction, name resolution, type inference, trait solving, borrow checker, constant evaluator, macro expansion, IR construction, IR transformations, machine instruction selection, object emission, linker integration, debug information generation. Each harness feeds its component (directly or via the full pipeline up to that component) with generated inputs and checks the harness invariant.

**End-to-end program fuzzer** (B29-75, prose directive — settled): yes, it is built: harness `e2e` compiles and runs generated programs through C1 and applies the differential protocol of §22.4 against the interpreter on gated programs.

**Generator (grammar-aware, structure-aware).** The generator is grammar-directed over the Volume 02 productions: production choices from the pinned xorshift64* PRNG (SPEC-INDEX §5.2), depth limit 24, node budget 20000. Two corpus classes: `raw` (any tree; parser robustness) and `well-formed` (type-directed generation: identifiers collected before use, types inferred bottom-up; exercises semantic components). Structure-awareness = the generator maintains grammar and scope invariants instead of emitting flat byte strings; byte-level mutations are applied only by the operators below.

**Mutation operators (enumerated, closed set of 12):**

| Op | Mutation |
| --- | --- |
| M1 | token deletion (random token removed) |
| M2 | token duplication (random token duplicated adjacent) |
| M3 | token splice (insert a token from the retained corpus) |
| M4 | sibling shuffle (permute two sibling statements) |
| M5 | literal mutation (±1 or single bit flip on an integer literal) |
| M6 | identifier conflict (rename an identifier to another in-scope name) |
| M7 | bracket imbalance (delete one bracket token) |
| M8 | brace swap (exchange `{` and `}` at one site) |
| M9 | operator substitution (swap two same-arity operators) |
| M10 | attribute flip (drop or invert one `#[cfg]`/feature attribute) |
| M11 | unicode injection (insert one non-ASCII scalar into a string or comment position) |
| M12 | span truncation (cut the input at a random byte offset) |

**Harness invariant.** For every input, within the per-input limits (time 10 s, memory per Volume 12 limits): the component must not crash (no panic, no UB, no memory fault) and must terminate with either a defined diagnostic or a defined artifact. Any other outcome (crash, hang beyond limits, wrong-artifact signal) is a finding. Compiler memory safety under adversarial input is G12; this harness is its evidence engine.

**Corpus retention.** Seeds live in `fuzz/corpus/<component>/` (append-only; new corpus entries must increase coverage per the harness coverage counters). Findings are minimized (§22.17) and retained in `fuzz/crashes/<component>/<id>/` with the input, the invariant violated, and the first divergence data. Every retained input is replayed in CI and must pass (must; OB-22-9).

**Budgets.** Per CI run per component: 10^5 executions or 300 s wall, whichever first; CI seeds are fixed (`seed = 20260101`) for determinism; long-running campaigns run offline with rotated seeds and report only minimized findings.

## 22.6 Property-based testing (defines design.md 29.7, bullets B29-76..B29-83)

Each property is an obligation with a stated sampling domain and, per SPEC-INDEX §4.3, each PROP is paired with Z3 or EXH for the same property (pairing named in the row):

| Bullet | Property | Sampling domain | Pairing |
| --- | --- | --- | --- |
| B29-76 | Parsing and printing consistency: print(parse(x)) reparses to an AST equal to parse(x) | corpus + generated trees (depth ≤ 12) | EXH over corpus |
| B29-77 | Type substitution consistency: substitution preserves well-formedness and commutes with unification | generated type terms (depth ≤ 6) | Z3 bounded |
| B29-78 | Graph mutation consistency: every mutation of a corpus graph keeps the IR verifier's verdict | generated graphs (n ≤ 64) | EXH over small instances |
| B29-79 | IR verifier preservation: verifier(input) passes ⇒ verifier(pipeline output) passes | optimization corpus | EXH (OB-22-4) |
| B29-80 | Deterministic compilation: byte-identical artifacts for identical inputs | conformance corpus | EXH double-compile (OB-22-6) |
| B29-81 | Serialization round-trip: deserialize(serialize(v)) = v for all IR/lockfile/cache values | corpus values | EXH over corpus |
| B29-82 | Semantic equivalence of optimization passes: ⟦pass(x)⟧ = ⟦x⟧ | gated differential corpus | EXH/EXH (OB-22-5) |
| B29-83 | Validity of generated machine instructions: every emitted encoding satisfies the Volume 15 table | generated functions | EXH over instruction table (OB-22-11) |

## 22.7 Runtime validation (defines design.md 29.8, bullets B29-84..B29-93)

| Bullet | Mechanism (exact) |
| --- | --- |
| B29-84 Address sanitization | Runtime allocator mode `san=address` (Volume 18 §25): 32-byte redzones before and after every allocation; quarantine of the 256 most recent freed allocations; violation = defined trap (exit 134) |
| B29-85 Undefined-behavior detection | Debug runtime enforces the Volume 05 provenance/validity checks, converting detectable instances of UB-1..UB-6 into defined traps; detection coverage per UB class is documented in the Volume 18 review record (REVIEW) |
| B29-86 Thread sanitization | Mode `san=thread`: vector-clock happens-before tracking; a race against UB-3 = defined trap; plus the stress scheduler of B29-91 |
| B29-87 Memory leak detection | Allocator reports live allocations at exit; leaks are reported with site ids; in test runs (`--sanitizer`), a non-empty leak report fails the test (exit 1) |
| B29-88 Stack overflow testing | Guard page + stack probing per Volume 18; overflow = defined trap (exit 134), never silent corruption; corpus cases exercise 1-frame and N-frame overflow |
| B29-89 Allocation failure injection | Allocator fault-injection mode: the k-th allocation fails, k drawn from the seeded PRNG over 1..=10^4; programs must exhibit their defined OOM behavior (G13) |
| B29-90 Panic injection | Runtime hook `panic=inject(k)`: the k-th panic-capable point panics; unwinding and drop paths are exercised per SPEC-INDEX destruction model |
| B29-91 Concurrency stress tests | Stress scheduler: seeds 1..=1000, preemption at every scheduling point with probability 0.1; schedules are recorded and replayable (deterministic per seed, G14) |
| B29-92 FFI stress tests | FFI harness: 1000 iterations of callback re-entry and error-return loops across the C ABI boundary; contract per Volume 10 §17 |
| B29-93 Unwinding tests | Panic across 1..=64 frames with drop counters; drop order must be innermost-first (SPEC-INDEX §3); `panic = abort` builds must trap immediately (exit 134) |

## 22.8 Backend testing (defines design.md 29.9, bullets B29-94..B29-103)

| Bullet | Mechanism (exact) |
| --- | --- |
| B29-94 Instruction encoding tests | EXH over the full Volume 15 instruction table: every instruction × operand form, encoding checked byte-exactly against the table (OB-22-11) |
| B29-95 Register allocation tests | SPEC-INDEX §5.8 laws: each interval gets one location; interfering intervals disjoint; EXH over generated small instances (n ≤ 16 intervals) + Z3 bounded instances |
| B29-96 Calling convention tests | Parameter/return location vectors for the declared type set, cross-checked against a C reference compiled by the pinned system toolchain per target (toolchain version recorded in evidence) |
| B29-97 ABI conformance tests | Struct passing/return classification per Volume 10 §17; each class has ≥ 4 vectors per target |
| B29-98 Object relocation tests | planck-reloc-math certificates (SPEC-INDEX §5.9) + linked artifacts executed under the emulator of B29-101 |
| B29-99 Linker tests | Symbol resolution and final layout per Volume 17 §24; duplicate/missing symbol cases produce the defined diagnostics |
| B29-100 Disassembly comparisons | `plkc disasm` output vs the independent reader of §22.18 on the object corpus: identical instruction streams (parity, OB-22-13) |
| B29-101 Emulator tests | Artifacts run under qemu-user (version pinned per target in the evidence record); exit code and stdout must equal the hardware run on the corpus |
| B29-102 Hardware tests | Target lab matrix: one reference machine class per implemented target, recorded in each evidence artifact (CEP&CC §13.1 fields) |
| B29-103 Cross-target tests | Corpus compiled for both implemented targets; on the target-independent corpus subset (same generator gate as §22.4 plus no target-specific features), behavior must be equal |

## 22.9 Verification ladder and formal language semantics (defines design.md §30 intro and 30.1, bullets B30-1..B30-9)

**The five levels** (design.md §30 table) are the verification ladder. Settled commitments per level in edition P1:

| Level | Commitment | Assurance class |
| --- | --- | --- |
| Verified data structures | All `math/` kernels meet their SPEC-INDEX §5.1 certificates (EXH/Z3/PROP) | FV-0 |
| Verified analyses | Solver, ownership, effect, resolution obligations of Volumes 04/06/07 | FV-0/FV-1 |
| Verified transformations | Per-pass dossiers and rows (§22.3, §22.11) | FV-0/FV-1 |
| Verified backend | Encoding, regalloc, relocation, ABI rows (§22.8) | FV-0/FV-1 |
| Verified compiler (end-to-end) | Composition of the above + differential (§22.4) + bootstrap equivalence (§22.15). A single machine-checked end-to-end theorem is **declared-not-implemented** in edition P1; it is a named refinement gap whose runtime bridge is the differential protocol plus the IR-verifier chain. Stating this gap is mandated (never omitted). | FV-1 |

The levels are not interchangeable (design.md §30 prose): a pass proof never substitutes for the end-to-end differential, and the differential never substitutes for a pass proof.

**Formal language semantics** (B30-1..B30-9). The settled form of the language specification (§30.6 prose directive, settled here and in §22.14): Planck's specification is a conventional normative document (Volumes 01–10) accompanied by formal executable models for a defined fragment — not a fully executable specification. The executable model is the reference interpreter (§22.4) plus the Z3-checkable judgments of the pinned kernels. Anchors:

| Bullet | Home and mechanism |
| --- | --- |
| B30-1 Source-language operational semantics | Volume 07 §10 (big-step judgments); executable model = reference interpreter over the fragment gate |
| B30-2 Type safety properties | G3 preservation/progress; Volume 04 obligations; Z3 over bounded step relations + REVIEW of progress |
| B30-3 Memory safety properties | G2 under explicit assumptions (target, runtime, FFI); provenance model per SPEC-INDEX §3; Volume 05 obligations |
| B30-4 Ownership soundness | Accepted programs never reach a rejected ownership state; Volume 06; EXH over the 42-cell table + Z3 over composed transitions |
| B30-5 Borrow checker soundness | Loans never outlive place validity; Volume 06 obligations; Z3 bounded + REVIEW |
| B30-6 Effect soundness | Call rule and row algebra of SPEC-INDEX §5.3; laws EXH over the full 2^8 row lattice |
| B30-7 Concurrency safety properties | UB-3 exclusion; happens-before model Volume 08 §12; obligations + stress protocol §22.7 |
| B30-8 FFI boundary assumptions | Volume 10 §17: C ABI at boundaries; callee memory model, no unwinding across FFI, declared preconditions — the enumerated assumption list is part of the claim (G2 catch honored) |
| B30-9 Undefined behavior semantics | UB is closed: UB-1..UB-6 exactly as pinned in SPEC-INDEX §3; everything else is a defined value, defined trap (exit 134), or compile-time rejection; no open-ended UB |

## 22.10 IR semantics (defines design.md 30.2, bullets B30-10..B30-17)

For every IR (IR 1: Volume 13 §20; IR 2: Volume 14 §21; Machine IR: Volume 15 §22):

| Bullet | Definition |
| --- | --- |
| B30-10 Formal node semantics | Each node kind has a written small-step semantics section in its volume; judgments are ATP-cast (obligation rows in the owning volume) |
| B30-11 Graph execution semantics | Execution = a defined schedule over the graph (Volume 13/14) or instruction stream (Volume 15); termination per pass bounds (G11) |
| B30-12 Control-flow semantics | Block/branch semantics with the dominator facts of §22.2 control-flow tests |
| B30-13 Memory semantics | Memory SSA with provenance (SPEC-INDEX §3 pointer model); per-IR memory obligations |
| B30-14 Effect semantics | Effect rows ride on nodes/functions per Volume 07 §10 rows; verifier checks row consistency (§22.2 effect-dep) |
| B30-15 Type preservation requirements | Each pass must preserve the IR's type discipline (verifier-checked; per-pass rows) |
| B30-16 Execution equivalence | ⟦IR1⟧ = ⟦IR2⟧ = ⟦MIR⟧ = ⟦artifact⟧ on the gated differential corpus (OB-22-5); per-stage verifiers hold statically, differential holds dynamically |
| B30-17 Valid graph invariants | The invariants checked by the §22.2 verifier; each invariant carries an invariant id (id form `INV-<IR>-<NAME>`; canonical instance `INV-IR1-SSA-DOM`) that is a registry anchor for G26 (§22.20) |

## 22.11 Transformation proofs (defines design.md 30.3, bullets B30-18..B30-26)

For every optimizing pass, the **pass dossier** (registered per §22.16) contains exactly the design.md §40 10-row template plus the preservation decomposition:

| Bullet | Dossier element |
| --- | --- |
| B30-18 Preconditions | The exact input domain D_T (IR invariants + program facts the pass requires); checked at runtime by the verifier or an explicit precondition check |
| B30-19 Transformation definition | The pass defined as a total function on D_T (productions, not prose) |
| B30-20 Postconditions | The IR invariants + the pass-specific facts guaranteed on output |
| B30-21 Semantic preservation argument | The obligation ⟦T(x)⟧ = ⟦x⟧ for all x ∈ D_T, with method and artifact (Z3 script, EXH certificate, or REVIEW record) |
| B30-22 Memory preservation | Memory SSA dependencies and provenance preserved; killed loads/stores justified (sub-obligation of B30-21) |
| B30-23 Effect preservation | Effect rows unchanged or narrowed (row subtyping per SPEC-INDEX §5.3) |
| B30-24 Control-flow preservation | CFG behavior preserved: dominator relation preserved or transformed per the pass's defined rule |
| B30-25 Ownership preservation | Loan/place states preserved (Volume 06 metadata consistency) |
| B30-26 Proof or validation artifact | The registered artifact: `tools/fv/<pass>.smt2`, an EXH certificate, or a REVIEW record; an obligation without an artifact is invalid (G15) |

## 22.12 Translation correctness (defines design.md 30.4, bullets B30-27..B30-31)

The five transitions, each with a defined preservation statement and mechanism; composition gives end-to-end preservation (G5 compositional argument, REVIEW):

| Bullet | Transition | Preservation statement | Mechanism |
| --- | --- | --- | --- |
| B30-27 | Source semantics → frontend semantic representation | Resolution/type/borrow/effect checking accepts exactly the well-formed programs; the checked representation denotes the source program | Conformance suite §22.1 + printer round-trip (B29-76) + differential on the fragment |
| B30-28 | Frontend representation → IR 1 | ⟦IR1⟧ = ⟦source⟧ per function | Per-desugaring-rule Z3 obligations on bounded instances + differential |
| B30-29 | IR 1 → IR 2 | ⟦IR2⟧ = ⟦IR1⟧ | Pass rows (§22.11) + pipeline differential (B29-79) |
| B30-30 | IR 2 → Machine IR | ⟦MIR⟧ = ⟦IR2⟧ | Instruction-selection equivalence: EXH per instruction class (B29-94) + differential |
| B30-31 | Machine IR → emitted instructions | ⟦artifact⟧ = ⟦MIR⟧ | Encoder EXH (OB-22-11) + relocation certificates + independent reader (§22.18) |

## 22.13 Trusted computing base (defines design.md 30.5, bullets B30-32..B30-43)

Per SPEC-INDEX §3 the TCB is exactly: (T1) the Planck math kernel (`math/` crates, FV-checked), (T2) the Z3 scripts (`tools/fv/`) and the Z3 kernel at the pinned version, (T3) the review process (human judgment records), (T4) the stage-0 compiler (bootstrap trust model). Amendment pointer (CEP-DR-01, in force now; this volume's T1..T4 enumeration is revised at its next revision per DR-7.2 to fold in the layered TCB): the proof layer gains the Rocq kernel at its pinned version (constructive proofs, `rocq/`) and MATP pinned at 677bea9 (first-order obligations), and the extraction layer (Peregrine/ConCert Rust route) is NOT trusted until OB-DR-1..4 discharge; until that revision, CEP-DR-01 §4 is the normative layered-TCB statement and this enumeration remains true for the layers it names. The checker-design obligation OB-DR-5 (small trusted proof checker) lands here as a verification-matrix row at the same revision. The §30.5 candidate components are classified; a component is trusted iff it is in {T1..T4}; every other component is a verified artifact carrying obligations:

| Bullet | Component | Classification | Basis |
| --- | --- | --- | --- |
| B30-32 | Lexer and parser | Verified artifact | OB-02 family rows; fuzz harness §22.5 |
| B30-33 | Type checker | Verified artifact | Volume 04 rows |
| B30-34 | Trait solver | Verified artifact | Volume 04 rows |
| B30-35 | Borrow checker | Verified artifact | Volume 06 rows; EXH state machine |
| B30-36 | IR verifier | Verified artifact | §22.2; verifier-completeness REVIEW row |
| B30-37 | Optimization pass infrastructure | Verified artifact | Pipeline order fixed; per-pass rows |
| B30-38 | Target descriptions | Verified artifact | Volume 16 rows; encoding EXH |
| B30-39 | Machine instruction encoder | Verified artifact | OB-22-11 |
| B30-40 | Object writer | Verified artifact | Volume 17 rows; independent reader cross-check |
| B30-41 | Linker | Verified artifact | §22.8 linker tests; Volume 17 rows |
| B30-42 | Runtime | Verified artifact | Volume 18 rows; §22.7 validation |
| B30-43 | Formal proof checker | Trusted (T2) | Z3 kernel + scripts; pinned version; replayed (§22.18) |

Trust argument (G16): the end-to-end semantic-preservation claim rests on T1–T4 plus the verified artifacts above. The reference interpreter is **not** in the TCB: it is FV-2 tooling; a defect in it can create false divergences (investigated) but cannot validate an incorrect compiler, because no claim in the matrix is discharged by an interpreter verdict. Each TCB element carries an integrity anchor: math kernel crates are hash-pinned in the build (Volume 20 §20.4 fingerprint), Z3's version string is recorded in every script header and replayed, review records are named documents with reviewers and dates, and stage 0 is pinned per §22.15.

## 22.14 Proof infrastructure (defines design.md 30.6, bullets B30-44..B30-53 and prose B30-54)

| Bullet | Settled definition |
| --- | --- |
| B30-44 Specification language | This spec library: structured markdown volumes with obligation IDs, plus SMT-LIB 2.x scripts for machine-checked fragments; obligations are stated as judgments/invariants (ATP-cast per SPEC-INDEX §1.3) |
| B30-45 Proof assistant | Z3 (version pinned in every script header; strict timeout 10 s per SPEC-INDEX §4.2) for bounded SMT checks; MATP (the project ATP, SPEC-INDEX §3) for first-order obligations where its theories apply |
| B30-46 Formal models | The `math/` kernels, the ownership state machine (SPEC-INDEX §5.4), the effect algebra (§5.3), and the reference interpreter (§22.4) |
| B30-47 Verified algorithms | The kernel crates (graph, regalloc, reloc, …) with per-function certificates in the §5.1 equivalence-claim form |
| B30-48 Proof automation | Deterministic tactic scripts in `tools/fv/`: no randomness, no wall-clock reads, fixed 10 s timeout, documented in each script header |
| B30-49 Proof checking | CI replay per §22.18; a timeout or `unknown` marks the obligation unproven and fails the build (SPEC-INDEX §4.2) |
| B30-50 Proof artifact storage | `tools/fv/<obligation>.smt2` + recorded results in the registry (§22.16); archived per release tag |
| B30-51 Proof reproducibility | Pinned tool versions + fixed timeouts + deterministic scripts ⇒ identical replay results; historical replays must succeed (§22.20) |
| B30-52 Spec-to-implementation correspondence | Extraction-crate certificates: every public kernel function documents its definition anchor, domain, failure set, and EXH/Z3/PROP certificate (SPEC-INDEX §5.1) |
| B30-53 Proof regression testing | Full registry replay on every commit (OB-22-2) |
| B30-54 (prose) Executable model vs normative document | Settled: conventional normative document + formal executable models for the defined fragment (§22.9). Rationale recorded: full executability of a systems language with FFI/concurrency is not achievable at bounded cost; the fragment gate keeps the executable model exact where it exists (FV-1 with named gap and bridge) |

## 22.15 Self-hosting and bootstrapping (defines design.md 30.7, bullets B30-55..B30-65)

| Bullet | Settled definition |
| --- | --- |
| B30-55 Initial compiler implementation language | Rust for stage 0 (`plkc0`), with the verified math kernels extracted into it (SPEC-INDEX §3 bootstrap trust model) |
| B30-56 Bootstrap compiler | `plkc0`: the trusted stage-0 compiler; compiles the Planck source of the compiler (`compiler-src`) |
| B30-57 Stage-one compiler | `plkc1 = plkc0(compiler-src)`: a Planck-compiled Planck compiler |
| B30-58 Stage-two compiler | `plkc2 = plkc1(compiler-src)` |
| B30-59 Stage comparison | (a) fixed-point check: `plkc1` and `plkc2` must produce byte-identical artifacts when compiling the same inputs (OB-22-8); (b) equivalence check: outputs of `plkc0` vs `plkc1` compared on the conformance corpus by differential execution + certificate comparison (SPEC-INDEX §3) |
| B30-60 Reproducible bootstrap | The chain plkc0 → plkc1 → plkc2 reruns byte-identically under the Volume 20 §20.4 contract (fixed toolchain, SOURCE_DATE_EPOCH) |
| B30-61 Trusted bootstrap chain | The chain is documented in the Bootstrap Specification deliverable: trust plkc0; verify plkc1 against plkc0 (corpus differential); verify plkc2 against plkc1 (byte identity); each link records its evidence artifacts and dates |
| B30-62 Cross-compilation bootstrap | plkc0 cross-builds plkc1 for each implemented target (Volume 20 §20.3); per-target equivalence is checked on the target (hardware lab) or under the pinned emulator |
| B30-63 Compiler self-build tests | The compiler compiles its own source with `-C verify-ir=on` and the full optimization corpus; self-build failures are Severity 0 |
| B30-64 Compiler version transitions | For release N → N+1: run the differential protocol between the N and N+1 binaries on the conformance corpus; produce an artifact-hash diff report; unexplained behavioral diffs block release |
| B30-65 Compiler distribution integrity | Release artifacts carry BLAKE3-256 checksums and an Ed25519 signature over the canonical-JSON checksum list (algorithm per Volume 20 §20.2); verification procedure: check signature, then per-artifact checksum; release builds must satisfy the reproducibility contract (G14) |

A self-hosting compiler is not automatically a trustworthy compiler: trust enters only through the documented chain of B30-61, and each promotion step is gated by its comparison (OB-22-8, OB-22-9).

## 22.16 Proof-obligation registry (G15)

The registry is the single completeness instrument for all obligations of all volumes. It lives at `spec/obligations.jsonl`: one JSON object per line (JSON Lines, sorted by obligation id), fields exactly:

| Field | Content |
| --- | --- |
| `id` | `OB-<vol>-<n>` (globally unique) |
| `volume` | owning volume file |
| `property` | one-line exact statement (identical to the matrix row) |
| `method` | one of `Z3`, `EXH`, `PROP`, `REVIEW` (PROP rows name their pairing) |
| `class` | one of `FV-0`, `FV-1`, `FV-2` (FV-1 rows name the refinement gap and runtime bridge) |
| `artifact` | path to the script/certificate/record |
| `status` | one of `proven`, `gap` (documented), `unproven` |
| `g` | list of G-goal ids served |
| `deps` | list of anchors (volume-§ ids and invariant ids such as `INV-IR1-SSA-DOM`) the obligation depends on |
| `deps_hash` | BLAKE3-256 over the canonical JSON of `deps` |

Registry law: (R1) every verification-matrix row of every volume appears exactly once in the registry (totality; checked mechanically, OB-22-1). (R2) no obligation without an artifact. (R3) `unproven` obligations fail CI; only `proven` and `gap` (with documented bridge) are mergeable. (R4) the registry is regenerated from the volume matrices by `tools/fv/regen` and a diff is a review requirement (no hand-edits that diverge from matrices).

## 22.17 Counterexample minimization (G17)

Every failed verification obligation, differential divergence, or fuzz finding produces a counterexample containing: the violated property, the failing input (program/IR/bytes), the execution path or first divergence point, and the violated precondition/postcondition. Minimization is **delta debugging over the token sequence**, exactly:

```
ddmin(P):                        # P = token list of the failing input; test(x) = failure reproduces
  n ← 2
  loop:
    chunks ← split P into n contiguous chunks (sizes differ by ≤ 1)
    for each complement c_i (P minus chunk i, in index order):
      if test(c_i): P ← c_i; n ← 2; goto loop
    if n = |P|: return P
    n ← min(2n, |P|); goto loop
```

Determinism: partitions are taken in token order; equal-size chunk ties break by lowest index; evaluation is single-threaded. Bounds: at most 512 test executions and 10 s per execution; if the bound is exhausted, the current best is returned with the flag `unminimized`. Termination measure: either |P| strictly decreases (progress) or n strictly increases bounded by |P|; hence termination is total. Complexity: O(|P|²) executions worst case, capped at 512. The minimized input replaces the raw finding in `fuzz/crashes/` and in regression cases (B29-47).

## 22.18 Independent verification and proof-artifact replay (G18)

| Mechanism | Definition |
| --- | --- |
| Proof-artifact replay | The CI runner re-derives every registry row from the volume matrices, executes its artifact with the pinned tool version and 10 s timeout, and compares the result with the recorded `status`; any mismatch fails the build (OB-22-2). Replay results are byte-recorded per commit id |
| Independent machine-code validation | `tools/objcheck/`: an independent reader of the Volume 17 object format that does not share code with the writer; it re-parses every corpus object and validates: encodings against the Volume 15 table, relocation fields against planck-reloc-math, symbol-table consistency. A mismatch is Severity 0 (OB-22-13) |
| Cross-validation of semantic models | The differential protocol (§22.4) is the standing cross-validation between the executable semantics (interpreter) and the compiler; the reference-backend comparison (B29-59) cross-validates encodings |
| Independent result reproduction | Release-tag replays are executed twice on independent CI runners; results must agree byte-wise (OB-22-14) |
| Reproducible proof results | Tool versions and timeouts are pinned in artifacts; a replay on the pinned versions must reproduce the recorded verdict (must) |

## 22.19 Verification matrix template (G15)

Every volume's verification matrix uses exactly these columns and rules:

| Column | Rule |
| --- | --- |
| Obligation | `OB-<vol>-<n>`, unique per volume, registered |
| Property | Exact statement (no prose hedges); the input domain is part of the statement |
| Method | one of `Z3`, `EXH`, `PROP`, `REVIEW`; PROP must name its pairing (`Z3` or `EXH`) or an explicit refinement gap |
| Class | one of `FV-0` (machine-checked, zero gaps), `FV-1` (bounded, named gap + runtime bridge), `FV-2` (verification tooling) |
| Artifact | Existing path; no artifact ⇒ invalid row |
| G-mapping | The G-goals served (may list several) |

Status discipline: a row is `proven` when its artifact passes in CI; `gap` when FV-1 with a documented bridge; anything else is `unproven` and fails CI. This template is binding for Volumes 01–26 and is checked by the registry regeneration (R4).

## 22.20 Specification evolution (G26)

| Bullet-level requirement (design.md §30 prose + G26) | Settled mechanism |
| --- | --- |
| Specification versioning | The normative version is `SPEC-INDEX.md` version (edition P1 = 0.1); each volume file carries a `CEP:STATUS` and its git commit id is the revision anchor. Changes are classified: `clarification` (no obligation text changes) or `normative` (some obligation's property, method, class, or domain changes); every normative change names the affected obligation ids in its review record |
| Proof dependency tracking | Registry rows carry `deps` anchors; changing any file or invariant that an anchor points to changes that anchor's content hash. `tools/fv/affected <anchor>` lists every obligation whose `deps` contain the anchor (linear scan; deterministic output order = obligation id ascending). Changing an IR invariant (id form `INV-<IR>-<NAME>`; canonical instance `INV-IR1-SSA-DOM`) automatically identifies every theorem and pass row depending on it (design.md §30.6 prose requirement, mechanically satisfied) |
| Proof invalidation on assumption change | When a dep's content hash changes, the dependent obligations' status resets to `unproven` and CI fails until re-proven; no stale-proof merge (design.md §30 prose: "old proofs … no longer proving what you think they prove") |
| Re-verification of affected components | The CI affected-set gate: only obligations whose deps changed re-run on incremental runs; the full registry replays on release branches (must) |
| Historical proof reproducibility | Each release tag archives the registry, artifacts, and tool versions; replaying an archived tag on its pinned versions must succeed (must, OB-22-15) |
| Compatibility verification | Edition/IR-format changes bump format tags (Volume 20 §20.6 invalidation law) and re-run the affected-set gate before merge |

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-22-1 | Registry totality: registry rows = union of all volume matrix rows, ids unique, fields complete | EXH | FV-2 | `tools/fv/regen` diff check | G15 |
| OB-22-2 | Replay fidelity: every registry artifact replays with its recorded verdict under pinned versions and 10 s timeout | EXH | FV-2 | CI replay runner | G15, G18 |
| OB-22-3 | Suite directive conformance: every conformance case parses its directives and behaves exactly per them | EXH | FV-0 | `tools/fv/suite_directives.z3` | G22, G19 |
| OB-22-4 | Verifier preservation: verifier(input) passes ⇒ verifier(output) passes after every pipeline pass on the optimization corpus | EXH | FV-1 | CI verify-ir job | G5, G4 |
| OB-22-5 | Differential equivalence: I ≡ C0 ≡ C1 on (outcome, stdout) for every gated corpus program | EXH | FV-1 | CI differential job | G1, G5 |
| OB-22-6 | Compiler determinism: byte-identical artifacts on double compile, J ∈ {1,2,4,8} | EXH | FV-1 | CI double-compile job | G14 |
| OB-22-7 | Harness safety: no fuzz input crashes/hangs any component within limits (replay of all retained corpus + crashes) | EXH | FV-1 | CI fuzz replay | G12, G13 |
| OB-22-8 | Bootstrap fixed point: plkc1 and plkc2 produce byte-identical artifacts on the corpus | EXH | FV-1 | CI bootstrap job | G25, G14 |
| OB-22-9 | Retained-input regression: every retained fuzz/conformance/regression input passes in CI | EXH | FV-1 | CI regression job | G15 |
| OB-22-10 | Encoding validity: every emitted instruction encoding satisfies the Volume 15 table | EXH | FV-0 | `tools/fv/encoding.z3` | G9, G1 |
| OB-22-11 | Regalloc laws: one location per interval; interfering intervals disjoint (small instances) | EXH+Z3 | FV-0 | `tools/fv/regalloc.smt2` | G9 |
| OB-22-12 | Differential counterexample completeness: every divergence record contains property, input, first divergence, and minimized input | REVIEW | FV-2 | Divergence record schema check | G17 |
| OB-22-13 | Independent reader parity: objcheck parse = writer output on the object corpus | EXH | FV-1 | CI objcheck job | G18, G24 |
| OB-22-14 | Replay reproducibility: independent CI runners produce byte-identical replay records for release tags | EXH | FV-2 | CI double-runner job | G18 |
| OB-22-15 | Historical replay: archived release-tag registries replay successfully on pinned versions | EXH | FV-2 | Release replay job | G26 |
| OB-22-16 | Affected-set correctness: `tools/fv/affected` returns exactly the obligations whose deps contain the anchor | EXH | FV-2 | `tools/fv/affected` self-test | G26 |
| OB-22-17 | Deps-hash invalidation: changing a dep's content resets dependent obligations to unproven | EXH | FV-2 | CI gate test | G26 |
| OB-22-18 | Ownership machine totality: the 42-cell transition table is total and matches SPEC-INDEX §5.4 | EXH | FV-0 | `math/planck-ownership-math` certificate | G3, G19 |
| OB-22-19 | Effect algebra laws: join/meet/subtyping laws hold over the 2^8 row lattice | EXH | FV-0 | `math/planck-effect-math` certificate | G3, G21 |
| OB-22-20 | Serialization round-trip: serialize∘deserialize = identity on IR, lockfile, cache values | EXH | FV-1 | CI round-trip job | G23, G14 |
| OB-22-21 | Fragment-gate soundness: excluded programs are never silently compared; gate decisions are recorded | REVIEW | FV-2 | Differential gate record | G1 |
| OB-22-22 | ddmin termination and determinism: ≤ 512 executions, deterministic partitions, total termination | Z3 | FV-1 | `tools/fv/ddmin.smt2` | G17, G11 |
| OB-22-23 | Per-pass gate completeness: every default-pipeline pass has 7 test files + registered dossier | EXH | FV-2 | `tools/fv/gate_audit` | G4, G5 |
| OB-22-24 | Allocation-elimination scenarios: the 8 named scenarios behave per their must/must-not rules (differential) | EXH | FV-1 | `tests/opt/alloc-elim/` | G4, G2 |
| OB-22-25 | Runtime validation traps: sanitizer/probe/injection modes produce the defined traps and reports | EXH | FV-1 | `tests/runtime-validation/` | G2, G20 |
| OB-22-26 | Emulator parity: qemu-user runs match hardware runs (exit, stdout) on the corpus | EXH | FV-1 | CI emulator job | G9 |
| OB-22-27 | TCB closure: every component of §22.13 is classified and every trusted element has an integrity anchor | REVIEW | FV-1 | TCB review record | G16 |
| OB-22-28 | End-to-end gap statement: the L5 refinement gap and its bridge are stated and never omitted | REVIEW | FV-1 | §22.9 ladder record | G1, G16 |

## Coverage ledger

design.md §29 (lines 3362–3525) and §30 (lines 3526–3652). Bullets in scope: 166, defined: 166; prose directives ledgered: 2 (B29-75, B30-54), defined.

| # | Bullet (abbreviated) | Defined in |
| --- | --- | --- |
| B29-1 | Lexical tests | §22.1 area table |
| B29-2 | Parsing tests | §22.1 area table |
| B29-3 | Grammar tests | §22.1 area table |
| B29-4 | Name resolution tests | §22.1 area table |
| B29-5 | Type checking tests | §22.1 area table |
| B29-6 | Generic tests | §22.1 area table |
| B29-7 | Trait resolution tests | §22.1 area table |
| B29-8 | Inference tests | §22.1 area table |
| B29-9 | Ownership tests | §22.1 area table |
| B29-10 | Borrow checking tests | §22.1 area table |
| B29-11 | Lifetime tests | §22.1 area table |
| B29-12 | Effect tests | §22.1 area table |
| B29-13 | Pattern tests | §22.1 area table |
| B29-14 | Constant evaluation tests | §22.1 area table |
| B29-15 | Async tests | §22.1 area table |
| B29-16 | FFI tests | §22.1 area table |
| B29-17 | Unsafe tests | §22.1 area table |
| B29-18 | ABI tests | §22.1 area table |
| B29-19 | Valid move | §22.1 canonical pairs (cf-move-ok) |
| B29-20 | Invalid double move | §22.1 canonical pairs (cf-double-move) |
| B29-21 | Valid shared borrows | §22.1 canonical pairs (cf-shared-borrows) |
| B29-22 | Invalid conflicting mutable borrow | §22.1 canonical pairs (cf-conflicting-mut-borrow) |
| B29-23 | Valid partial initialization | §22.1 canonical pairs (cf-partial-init-ok) |
| B29-24 | Invalid use of uninitialized fields | §22.1 canonical pairs (cf-uninit-field-use) |
| B29-25 | Valid reborrow | §22.1 canonical pairs (cf-reborrow-ok) |
| B29-26 | Invalid reference escape | §22.1 canonical pairs (cf-ref-escape) |
| B29-27 | Valid effect contract | §22.1 canonical pairs (cf-effect-ok) |
| B29-28 | Invalid effect contract | §22.1 canonical pairs (cf-effect-bad) |
| B29-29 | Valid trait implementation | §22.1 canonical pairs (cf-trait-impl-ok) |
| B29-30 | Invalid coherence overlap | §22.1 canonical pairs (cf-coherence-overlap) |
| B29-31 | Valid graph tests | §22.2 class table |
| B29-32 | Invalid graph tests | §22.2 class table |
| B29-33 | Type consistency tests | §22.2 class table |
| B29-34 | Control-flow tests | §22.2 class table |
| B29-35 | Memory SSA tests | §22.2 class table |
| B29-36 | Effect dependency tests | §22.2 class table |
| B29-37 | Ownership metadata tests | §22.2 class table |
| B29-38 | Representation consistency tests | §22.2 class table |
| B29-39 | Machine constraint tests | §22.2 class table |
| B29-40 | Serialization tests | §22.2 class table |
| B29-41 | Precondition test | §22.3 per-pass gate (precondition.plk) |
| B29-42 | Successful optimization test | §22.3 per-pass gate (applies.plk) |
| B29-43 | Must-not-optimize case | §22.3 per-pass gate (must-not.plk) |
| B29-44 | Interaction test | §22.3 per-pass gate (interaction.plk) |
| B29-45 | Edge-case test | §22.3 per-pass gate (edge.plk) |
| B29-46 | Semantic equivalence test | §22.3 per-pass gate (equivalence.plk) |
| B29-47 | Regression test | §22.3 per-pass gate (regression.plk) |
| B29-48 | Unobservable allocation | §22.3 allocation-elimination scenarios |
| B29-49 | Address-observable allocation | §22.3 allocation-elimination scenarios |
| B29-50 | Foreign pointer escape | §22.3 allocation-elimination scenarios |
| B29-51 | Custom allocator effects | §22.3 allocation-elimination scenarios |
| B29-52 | Allocation failure behavior | §22.3 allocation-elimination scenarios |
| B29-53 | Destructor side effects | §22.3 allocation-elimination scenarios |
| B29-54 | Object identity | §22.3 allocation-elimination scenarios |
| B29-55 | Pinned objects | §22.3 allocation-elimination scenarios |
| B29-56 | Simple semantic interpreter | §22.4 reference implementations (I) |
| B29-57 | Unoptimized compiler mode | §22.4 reference implementations (C0) |
| B29-58 | Optimized compiler | §22.4 reference implementations (C1) |
| B29-59 | Separate reference backend | §22.4 reference implementations (ref-backend) |
| B29-60 | Fuzzer: lexer | §22.5 target list |
| B29-61 | Fuzzer: parser | §22.5 target list |
| B29-62 | Fuzzer: AST construction | §22.5 target list |
| B29-63 | Fuzzer: name resolution | §22.5 target list |
| B29-64 | Fuzzer: type inference | §22.5 target list |
| B29-65 | Fuzzer: trait solving | §22.5 target list |
| B29-66 | Fuzzer: borrow checker | §22.5 target list |
| B29-67 | Fuzzer: constant evaluator | §22.5 target list |
| B29-68 | Fuzzer: macro expansion | §22.5 target list |
| B29-69 | Fuzzer: IR construction | §22.5 target list |
| B29-70 | Fuzzer: IR transformations | §22.5 target list |
| B29-71 | Fuzzer: machine instruction selection | §22.5 target list |
| B29-72 | Fuzzer: object emission | §22.5 target list |
| B29-73 | Fuzzer: linker integration | §22.5 target list |
| B29-74 | Fuzzer: debug information generation | §22.5 target list |
| B29-75 | (prose) end-to-end program fuzzer | §22.5 End-to-end program fuzzer (settled: built) |
| B29-76 | Parsing/printing consistency | §22.6 property table |
| B29-77 | Type substitution consistency | §22.6 property table |
| B29-78 | Graph mutation consistency | §22.6 property table |
| B29-79 | IR verifier preservation | §22.6 property table |
| B29-80 | Deterministic compilation | §22.6 property table |
| B29-81 | Serialization/deserialization consistency | §22.6 property table |
| B29-82 | Semantic equivalence of optimization passes | §22.6 property table |
| B29-83 | Validity of generated machine instructions | §22.6 property table |
| B29-84 | Address sanitization | §22.7 mechanism table |
| B29-85 | Undefined-behavior detection | §22.7 mechanism table |
| B29-86 | Thread sanitization | §22.7 mechanism table |
| B29-87 | Memory leak detection | §22.7 mechanism table |
| B29-88 | Stack overflow testing | §22.7 mechanism table |
| B29-89 | Allocation failure injection | §22.7 mechanism table |
| B29-90 | Panic injection | §22.7 mechanism table |
| B29-91 | Concurrency stress tests | §22.7 mechanism table |
| B29-92 | FFI stress tests | §22.7 mechanism table |
| B29-93 | Unwinding tests | §22.7 mechanism table |
| B29-94 | Instruction encoding tests | §22.8 mechanism table |
| B29-95 | Register allocation tests | §22.8 mechanism table |
| B29-96 | Calling convention tests | §22.8 mechanism table |
| B29-97 | ABI conformance tests | §22.8 mechanism table |
| B29-98 | Object relocation tests | §22.8 mechanism table |
| B29-99 | Linker tests | §22.8 mechanism table |
| B29-100 | Disassembly comparisons | §22.8 mechanism table |
| B29-101 | Emulator tests | §22.8 mechanism table |
| B29-102 | Hardware tests | §22.8 mechanism table |
| B29-103 | Cross-target tests | §22.8 mechanism table |
| B30-1 | Source-language operational semantics | §22.9 anchor table |
| B30-2 | Type safety properties | §22.9 anchor table |
| B30-3 | Memory safety properties | §22.9 anchor table |
| B30-4 | Ownership soundness | §22.9 anchor table |
| B30-5 | Borrow checker soundness | §22.9 anchor table |
| B30-6 | Effect soundness | §22.9 anchor table |
| B30-7 | Concurrency safety properties | §22.9 anchor table |
| B30-8 | FFI boundary assumptions | §22.9 anchor table |
| B30-9 | Undefined behavior semantics | §22.9 anchor table |
| B30-10 | Formal node semantics | §22.10 table |
| B30-11 | Graph execution semantics | §22.10 table |
| B30-12 | Control-flow semantics | §22.10 table |
| B30-13 | Memory semantics | §22.10 table |
| B30-14 | Effect semantics | §22.10 table |
| B30-15 | Type preservation requirements | §22.10 table |
| B30-16 | Execution equivalence | §22.10 table |
| B30-17 | Valid graph invariants | §22.10 table |
| B30-18 | Preconditions | §22.11 dossier table |
| B30-19 | Transformation definition | §22.11 dossier table |
| B30-20 | Postconditions | §22.11 dossier table |
| B30-21 | Semantic preservation argument | §22.11 dossier table |
| B30-22 | Memory preservation | §22.11 dossier table |
| B30-23 | Effect preservation | §22.11 dossier table |
| B30-24 | Control-flow preservation | §22.11 dossier table |
| B30-25 | Ownership preservation | §22.11 dossier table |
| B30-26 | Proof or validation artifact | §22.11 dossier table |
| B30-27 | Source → frontend representation | §22.12 transition table |
| B30-28 | Frontend → IR 1 | §22.12 transition table |
| B30-29 | IR 1 → IR 2 | §22.12 transition table |
| B30-30 | IR 2 → Machine IR | §22.12 transition table |
| B30-31 | Machine IR → emitted instructions | §22.12 transition table |
| B30-32 | TCB: lexer and parser | §22.13 classification table |
| B30-33 | TCB: type checker | §22.13 classification table |
| B30-34 | TCB: trait solver | §22.13 classification table |
| B30-35 | TCB: borrow checker | §22.13 classification table |
| B30-36 | TCB: IR verifier | §22.13 classification table |
| B30-37 | TCB: optimization pass infrastructure | §22.13 classification table |
| B30-38 | TCB: target descriptions | §22.13 classification table |
| B30-39 | TCB: machine instruction encoder | §22.13 classification table |
| B30-40 | TCB: object writer | §22.13 classification table |
| B30-41 | TCB: linker | §22.13 classification table |
| B30-42 | TCB: runtime | §22.13 classification table |
| B30-43 | TCB: formal proof checker | §22.13 classification table (trusted T2) |
| B30-44 | Specification language | §22.14 table |
| B30-45 | Proof assistant | §22.14 table |
| B30-46 | Formal models | §22.14 table |
| B30-47 | Verified algorithms | §22.14 table |
| B30-48 | Proof automation | §22.14 table |
| B30-49 | Proof checking | §22.14 table |
| B30-50 | Proof artifact storage | §22.14 table |
| B30-51 | Proof reproducibility | §22.14 table |
| B30-52 | Spec-to-implementation correspondence | §22.14 table |
| B30-53 | Proof regression testing | §22.14 table |
| B30-54 | (prose) executable model vs normative document | §22.14 table (settled: document + fragment models) |
| B30-55 | Initial compiler implementation language | §22.15 table |
| B30-56 | Bootstrap compiler | §22.15 table |
| B30-57 | Stage-one compiler | §22.15 table |
| B30-58 | Stage-two compiler | §22.15 table |
| B30-59 | Stage comparison | §22.15 table |
| B30-60 | Reproducible bootstrap | §22.15 table |
| B30-61 | Trusted bootstrap chain | §22.15 table |
| B30-62 | Cross-compilation bootstrap | §22.15 table |
| B30-63 | Compiler self-build tests | §22.15 table |
| B30-64 | Compiler version transitions | §22.15 table |
| B30-65 | Compiler distribution integrity | §22.15 table |
