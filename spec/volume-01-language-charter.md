<!--
CEP:FILE: spec/volume-01-language-charter.md
CEP:WHAT: Volume 01 of the Planck Master Specification Library: the Language Charter and Implementation Requirements Specification for Planck 0.1, edition P1. Defines design.md §1 (lines 39-117): language identity, performance contract, safety contract, platform contract, compatibility policy.
CEP:WHY: design.md §1 requires the product contract to be settled before any language feature is defined; every later volume cites the settled values here (UB set, overflow classes, platform matrix, compatibility rules) instead of re-deciding them.
CEP:STATUS: complete
CEP:FAILURE: A volume or crate that contradicts a normative statement here is non-conforming and must be corrected. A missing row in the coverage ledger (§1.8) is a missing definition. Any normative statement containing "appropriate", "reasonable", "as needed", or "TBD" in this file is a defect.
CEP:ASSUMES: SPEC-INDEX.md §0-§5 (authority chain, method, pinned decisions, pinned contracts); standards.md §3-§4 (normative verbs, core laws); verification standards.md §2 (G1-G5); doc standards.md (structure and style); design.md lines 39-117.
CEP:COST: document only; no runtime cost. The performance numbers herein become measurement obligations on the artifacts named in §1.2 and §1.7.
CEP:EVIDENCE: the Verification matrix (§1.7) and Coverage ledger (§1.8) in this file; measurement artifacts bench/kernels-v1 and bench/corpus-v1 (defined in Volume 23).
-->

# Volume 01 — Language charter

**Abstract.** This volume defines the product contract of the Planck language, version 0.1, edition P1: identity, performance objectives, safety guarantees, supported platforms, and compatibility policy. It defines every bullet of design.md §1 (lines 39-117) normatively. It constrains all later volumes; it defines no syntax itself.

## 1.0 Scope and traceability

| Field | Value |
| ----- | ----- |
| design.md sections defined | §1.1-§1.5 (lines 39-117) |
| Bullets in scope | 54 (§1.1: 9, §1.2: 11, §1.3: 12, §1.4: 12, §1.5: 10) |
| Prose requirements in scope | 2 (design.md line 71 "Do not promise that every abstraction is free ...", line 116 deliverable) |
| Deliverable satisfied | "Language Charter and Implementation Requirements Specification" (this file) |
| Normative verbs | must / must not / may per standards.md §3 |

The language is **Planck**, version **0.1**, edition **P1** (SPEC-INDEX §0). The compiler is the **Planck compiler**, working title `plkc`.

## 1.1 Language identity

### 1.1.1 Language name and versioning (design.md L47)

The language name is `Planck`. The version is `0.1`. The edition is `P1`. Canonical version string: `Planck 0.1 (edition P1)`. `plkc --version` must print exactly `plkc 0.1.0 (edition P1)` followed by U+000A. Editions are named `P1`, `P2`, ...; the version pair (0.1, P1) is fixed for this specification library; a change of either is an edition change (§1.5.8).

### 1.1.2 Language design principles (design.md L48)

The principles, in priority order, are:

| # | Principle | Exact statement |
| - | --------- | --------------- |
| P1 | No hidden cost | standards.md Law 1; the only permitted cost-introducing constructs are §1.2.6 A1-A3, each documented. |
| P2 | Closed UB | The only undefined behaviors are UB-1..UB-6 (§1.3.9); everything else is a defined value, a defined trap, or a compile-time rejection. |
| P3 | Explicitness | No implicit conversion outside the exact coercion list of Volume 04 §6.3; widening that list is an edition change. |
| P4 | Determinism | Identical (inputs, configuration, target, toolchain version) must produce byte-identical outputs (G14). |
| P5 | Verifiability | Every semantic rule must be stated as a judgment, invariant, or obligation with an OB-<vol>-<n> ID and a verification row. |

Where principles conflict, the lower number wins.

### 1.1.3 Intended programming domains (design.md L49)

The intended domains are exactly: (D1) operating-system kernels and drivers, (D2) embedded firmware (freestanding, §1.4.12), (D3) high-performance network and storage services, (D4) compilers, interpreters, and verification tooling, (D5) performance-critical library infrastructure. Domains outside this list are permitted uses but are not design targets.

### 1.1.4 Supported programming paradigms (design.md L50)

Supported paradigms: (1) imperative, expression-oriented structured programming; (2) parametric polymorphism via generics and traits (Volumes 04, 06); (3) modular programming via the module tree and visibility (Volume 03); (4) structured concurrency via async/await with the pinned task-tree model (SPEC-INDEX §3, Volume 08); (5) functional fragments (closures, iterators) under the cost rules of §1.2. Excluded paradigms: inheritance-based object orientation (no subtype inheritance exists in Planck), dynamic duck typing (no unchecked dynamic dispatch outside `dyn`, which requires object safety, Volume 04), reflection beyond compile-time evaluation (Volume 09), garbage-collected managed style (no tracing collector exists).

### 1.1.5 Primary implementation goals (design.md L51)

The primary implementation goals, in priority order: (I1) G1 semantic preservation for every accepted program; (I2) G2 memory safety per §1.3.1; (I3) G3 type soundness per §1.3.2; (I4) G4/G5 pass and optimization equivalence (Volume 22); (I5) G14 reproducibility per §1.2.9; (I6) the measurable objectives of §1.2.1-§1.2.5; (I7) self-hosting: stage 1 (`plkc` compiled by `plkc`) verified equivalent to stage 0 (Volume 22). Conflicts are resolved in the listed order.

### 1.1.6 Non-goals (design.md L52)

The following are non-goals for edition P1 and must not be implemented without an edition change: (N1) an interpreter or JIT in the baseline product (Volume 26 defines optional execution products outside the P1 contract); (N2) class inheritance or subtype polymorphism; (N3) exception handling other than `panic` (Volume 07); (N4) implicit conversions beyond Volume 04 §6.3; (N5) procedural, attribute, or derive macros (§2.5.20 of Volume 02 admits declarative macros only); (N6) dynamic linking as a requirement for deployed binaries (static linking is the default; Volume 17 defines the object contract); (N7) non-IEEE floating-point modes (no fast-math); (N8) implementation of any target in the declared-not-implemented list of §1.4.1.

### 1.1.7 Compatibility philosophy (design.md L53)

The compatibility contract is this specification library, not documentation or tests. Rules: (C1) breaking changes are made only through editions; (C2) within an edition, source compatibility is governed by §1.5.1; (C3) a feature is stabilized only by the procedure of §1.5.9; (C4) deprecation follows §1.5.7; (C5) a volume that contradicts design.md is corrected in the volume, and design.md is retained unchanged (SPEC-INDEX §0.4).

### 1.1.8 Relationship between language and standard library (design.md L54)

The language defines: primitive types and their layouts (Volume 04), operators (Volume 02 §2.4), control flow, patterns, ownership and effects (Volumes 05-08), modules (Volume 03). Everything else is the standard library (Volume 19), split into `core` (no allocation, no OS) and `std` (allocation, OS, I/O). The compiler mandates exactly two runtime facilities: (R1) the trap path, which terminates the process with exit code 134 (pinned, SPEC-INDEX §3); (R2) unwinding tables for `panic` builds, absent in `panic = abort` builds. A freestanding crate (inner attribute `#![no_std]`) must compile with no standard-library linkage; the language itself requires none. The edition P1 prelude is the empty set: no name is in scope without an explicit `use` (Volume 03 §3.1.16).

### 1.1.9 Relationship between language and compiler implementation (design.md L55)

`plkc` must accept exactly the programs admitted by the judgments of Volumes 02-10 and must reject exactly their complements. The compiler must not add language surface outside the attribute set enumerated in Volume 10. The compiler itself must satisfy standards.md (classes CEP-0/1/2 per component) and G3 (compiler self-correctness). Compiler release N must accept every program accepted by release N-1 under the same edition, restricted only by §1.5.4. The CLI surface is defined in Volume 21.

**Integration points.** Depends on: SPEC-INDEX §0-§5. Depended on by: every volume; Volume 19 (library split), Volume 21 (CLI), Volume 22 (bootstrap), Volume 23 (measurement of §1.2).

## 1.2 Performance contract

### 1.2.1 Compilation-time objectives (design.md L59)

Reference hardware: one x86-64 Linux machine, 8 cores at 3.5 GHz, 32 GiB RAM, NVMe SSD. Reference workload: the corpus `bench/corpus-v1`, defined in Volume 23 §31.1 (50 crates, 1200 files, 250000 source lines). Objectives, exact:

| ID | Objective | Value |
| -- | --------- | ----- |
| T1 | Frontend throughput (lex, parse, resolve, typecheck, lower to IR 1) | ≥ 100000 source lines per second per worker thread |
| T2 | Full optimized build of the corpus | ≤ 120 s wall clock on 8 workers |
| T3 | Incremental rebuild after editing one function body, including code generation for affected items | ≤ 2 s wall clock |
| T4 | Constraint on all of T1-T3 | No objective may be met by nondeterministic caching or by reordering output (G14) |

### 1.2.2 Runtime performance objectives (design.md L60)

The benchmark suite `bench/kernels-v1` (defined in Volume 23 §31.2) consists of exactly 10 kernels: daxpy (n = 1048576), dgemm (512 × 512, row-major), matvec (4096 × 4096), memcpy (67108864 bytes), memset (67108864 bytes), quicksort (10000000 `u64` keys), binary_search (10000000 sorted `u64` keys, 10000000 probes), sha256 (16777216 bytes), crc32 (16777216 bytes), string_search (67108864-byte haystack, 1000000 probes). Objective: each kernel compiled with `-O2` must execute within 1.10× the wall-clock time of the C reference implementation compiled with clang at `-O2` on the reference hardware, measured by the protocol of Volume 23 §31.3.

### 1.2.3 Memory consumption objectives (design.md L61)

Compiler: peak resident set size over the corpus build must be ≤ 4096 bytes per source line. Runtime: the Planck allocator (Volume 18) must add at most 16 bytes of overhead per allocation, including alignment padding charged to the allocator, not to type layout (Volume 05). No garbage collector exists; deallocation is exactly the destruction model pinned in SPEC-INDEX §3.

### 1.2.4 Code-size objectives (design.md L62)

With `-Os`: the generated machine code (text + rodata) of each `bench/kernels-v1` kernel must be ≤ 1.5× the size of the C reference at `-Os`. A freestanding program whose `main` returns immediately must have text + rodata ≤ 2048 bytes.

### 1.2.5 Startup-time objectives (design.md L63)

For a static freestanding binary of ≤ 2048 bytes text+rodata on the reference hardware, the number of cycles from process entry to the first instruction of `main` must be ≤ 3000000, measured with `perf stat` per the Volume 23 protocol.

### 1.2.6 Allocation behavior (design.md L64)

The exact set of constructs that may allocate heap memory:

| ID | Construct | Volume defining it |
| -- | --------- | ------------------ |
| A1 | Calls to allocator functions of the Volume 18 API | 18 |
| A2 | Operations on growable collection types enumerated in Volume 19 §26.2 | 19 |
| A3 | Async task spawn (task frames), per the pinned task-tree model | 08 |

Every other construct must not allocate. Closure environments that escape analysis (Volume 14) proves non-escaping must be stack-allocated; escaping closures allocate under A3. `dyn` trait object creation allocates nothing: the vtable is placed in static rodata (Volume 05). Hidden allocation outside A1-A3 violates Law 1 and is a defect.

### 1.2.7 Zero-cost abstraction definition (design.md L65)

An abstraction is **zero-cost** if and only if, for every program P using the abstraction, both hold: (Z1) semantic equivalence: the IR 1 of `P` equals the IR 1 of the manual expansion `M(P)` up to the exact rewrite below, machine-checked on bounded instances (G4/G5); (Z2) cost parity: on the reference hardware with `-O2`, the cycle count of `P` equals that of `M(P)` within the measurement tolerance of 0 cycles (equal medians over 30 runs). The manual expansions are exact: generics monomorphize to the hand-specialized source; closures compile to a struct holding the captures with a call operator; an iterator chain of `map`/`filter`/`sum` compiles to the equivalent `while` loop over the same element sequence; `for x in a..b` compiles to a counted loop over `[a, b)`. Abstractions outside this list are not claimed zero-cost; their costs are those of their expansion rules in their defining volumes.

### 1.2.8 Runtime overhead policy (design.md L66)

The Planck runtime (Volume 18) consists of exactly: the task scheduler (CEP-0), the allocator (CEP-0), the panic and unwinding machinery (cold-only; absent entirely in `panic = abort` builds), and formatting/I/O support (cold-only). Each hot path must be cycle-exact class CEP-0 per standards.md §5.1. Every mechanism on the standards.md Law 1 hidden-cost list that Planck admits must appear in §1.2.6; all others are banned in shipped code.

### 1.2.9 Performance predictability (design.md L67)

**Predictable** means all five hold for a program: (1) no allocation outside A1-A3; (2) no dynamic dispatch outside `dyn` at an explicit object type; (3) no unbounded hidden loop (all loops are source-visible); (4) every standard-library hot function documents its worst-case asymptotic complexity in its reference entry (Volume 19); (5) `panic = abort` builds contain no unwinding tables. Enforcement: lint gates per standards.md §2.3 (warnings, CI checks, benchmarks), exact rules in Volume 21 §28.4.

### 1.2.10 Compile-time versus runtime computation (design.md L68)

The const contexts are exactly: `const` items, `static` initializers, array lengths, enum discriminants, and calls appearing inside those contexts that resolve to `const fn` (Volume 09). Evaluation there happens at compile time under the Volume 09 interpreter contract (total, deterministic, step-limited). Everything else evaluates at runtime. No other construct is implicitly evaluated at compile time; const generic parameters do not exist in edition P1 (Volume 02 §2.5.4 admits lifetime and type parameters only).

### 1.2.11 Optimization guarantees versus optimization hints (design.md L69)

Guaranteed transformations (the compiler must apply or prove inapplicable): (O1) literal folding; (O2) constant folding of `const` items into immediates; (O3) removal of provably unreachable basic blocks; (O4) register allocation satisfying the coloring law of SPEC-INDEX §5.8. Hints: `#[inline]`, `#[inline(never)]`, `#[cold]` (semantics in Volume 10). A hint must never change observable behavior (G1, G4); a guarantee must never change observable behavior (G1). Everything else in the pipeline is opportunistic and must preserve semantics per G5.

### 1.2.12 Guaranteed costs versus optimization-dependent costs (design.md line 71)

design.md line 71 requires that the language not promise every abstraction is free. The exact split:

| Cost | Status | Definition |
| ---- | ------ | ---------- |
| One compare-and-branch per checked arithmetic operation | Guaranteed | Overflow class Panic (Volume 02 §2.4); the check cannot be removed unless range is proven (G5 allows proof-based removal). |
| One compare-and-branch per slice/array index unless bounds are proven | Guaranteed | Bounds check emitted by Volume 02 §2.4 lowering; provably-in-bounds indices may drop it (G5). |
| Move of a value: memcpy of the value's layout size | Guaranteed | Volume 05 move semantics. |
| Monomorphization of a generic instantiation | Guaranteed compile-time cost | SPEC-INDEX §3: depth limit 32; Volume 12 limits table. |
| Inlining, unrolling, vectorization, escape-driven stack promotion | Not guaranteed (opportunistic) | Volume 14 passes; must preserve semantics (G5); may be absent. |

**Integration points.** Depends on: Volumes 02 (operators), 05 (moves/layout), 08 (tasks), 09 (const), 12 (limits), 14 (passes), 18 (runtime), 19 (collections), 23 (measurement). Depended on by: Volumes 04-22 cite §1.2 numbers instead of restating them.

## 1.3 Safety contract

### 1.3.1 Memory safety (design.md L75)

For every well-typed program P and every valid input I, the generated program must satisfy `Safe(C(P), I)` with the 13-point definition of verification standards.md §G2 (no out-of-bounds access, no use-after-free, no double-free, no invalid deallocation, no prohibited uninitialized read, no invalid dereference, no out-of-bounds pointer arithmetic, no data race, no invalid lifetime transition, no alignment violation, no overflow-driven invalid memory operation, no invalid stack or heap access, no aliasing or provenance violation). The assumptions on target ISA, runtime, allocator, foreign code, and OS are exactly those of verification standards.md §G2. Enforcement: the ownership and borrow checker (Volume 06) for safe code; unsafe code is bounded by §1.3.10.

### 1.3.2 Type safety (design.md L76)

The type system must satisfy preservation and progress (G3) in the form of Volume 04 §6.9: every well-typed, non-diverging program step preserves typing or reaches a permitted terminal state (value, defined trap, or defined termination). The rejection set of the type checker (Volume 04 §6.10) must be exactly the complement of its acceptance judgments; no accepted program may reach an untyped state.

### 1.3.3 Initialization safety (design.md L77)

Every place must be in state `Owned` or otherwise initialized before a `Read` action. Reading a place in state `Uninitialized` is UB-1. The checker must prove definite initialization for every safe-code read using the state machine of SPEC-INDEX §5.4 (Uninitialized → Owned via Write only). The exact algorithm is Volume 06 §9.4; the pinned transition table is normative here by reference.

### 1.3.4 Ownership safety (design.md L78)

Every value has exactly one owner at every program point. `MoveOut` transfers ownership and sets the source place to `MovedFrom`; reading or moving from `MovedFrom` is a compile-time rejection in safe code. The transition table of SPEC-INDEX §5.4 is normative; its totality (42 cells) is obligation OB-01-3.

### 1.3.5 Borrowing safety (design.md L79)

At every program point, for every place, exactly one of the following holds: the place is un-borrowed (`Owned`, `MovedFrom`, or `Uninitialized`); the place carries n ≥ 1 shared loans and no mutable loan (`Shared(n)` or `Frozen(n)`); the place carries exactly one mutable loan and no other loan (`MutableBorrowed`). The conflict predicate is derived exactly from the rejection entries of the pinned table (SPEC-INDEX §5.4). Loans end only by `EndBorrow` at their lexical scope end.

### 1.3.6 Data-race freedom (design.md L80)

Safe code must not be able to cause UB-3 (two unsynchronized conflicting accesses, at least one non-atomic). The static proof obligation (send/sync analysis) is Volume 08 §12.3; the goal is pinned here: a program accepted by the safe fragment must have no UB-3 execution under the memory model of Volume 08.

### 1.3.7 Pointer validity (design.md L81)

Provenance is allocation ID + offset + capability set {load, store, compute}, per the pinned model (SPEC-INDEX §3). Creating a reference outside its validity bounds is UB-2. Provenance narrows on arithmetic, is preserved by copies, and is checked at dereference. Safe code cannot form a pointer with an invalid provenance; raw-pointer construction and dereference are restricted to `unsafe` blocks (§1.3.10).

### 1.3.8 Integer overflow behavior (design.md L82)

Overflow is a defined event, never undefined behavior. Default builds: an out-of-range arithmetic result raises a panic with the exact message of Volume 02 §2.4.4; in `panic = abort` builds the panic is an immediate trap (exit code 134). Wrapping and unchecked arithmetic exist only as named intrinsics (Volume 10) carrying the `Wrap` and `Unchecked` overflow classes; `Unchecked` is restricted to `unsafe` code. Overflow in a const context is a compile-time rejection (Volume 09). The exact per-operator classification is the table of Volume 02 §2.4.

### 1.3.9 Undefined behavior categories (design.md L83)

The complete set of undefined behaviors, pinned (SPEC-INDEX §3): (UB-1) reading uninitialized memory through a typed place; (UB-2) creating a reference outside its validity bounds; (UB-3) violating data-race freedom (two unsynchronized conflicting accesses, at least one non-atomic); (UB-4) calling a function through a wrong-ABI function pointer; (UB-5) inline-assembly blocks violating their declared clobber/condition lists; (UB-6) violating an intrinsic's declared precondition. Everything else is a defined value, a defined trap, or a compile-time rejection. Traps are defined terminations: the process aborts with exit code 134. This enumeration is closed; adding a category is an edition change (G26).

### 1.3.10 Unsafe-code responsibilities (design.md L84)

Inside an `unsafe` block the programmer must uphold, for each operation used, its declared precondition: raw-pointer dereference validity (UB-2), union field-activity validity (UB-1, Volume 05 §8.6), foreign-function contracts (UB-4, §1.3.11), intrinsic preconditions (UB-6), and inline-assembly clobber/condition lists (UB-5). The compiler must reject every unsafe operation outside an `unsafe` block (diagnostic catalog Volume 21 §28.6). Each `unsafe` block must carry a comment whose first line begins with the exact byte string `// SAFETY:` stating the upheld preconditions; a missing comment is lint failure `E-LINT-SAFETY` (Volume 21). The compiler's obligations do not change inside unsafe blocks: it still rejects malformed programs and still guarantees G1 for accepted programs.

### 1.3.11 FFI safety boundaries (design.md L85)

Crossing FFI must use `extern "C"` declarations (Volume 03 §3.3.16; calling convention per §1.4.5). Foreign functions carry the `ffi` effect atom (SPEC-INDEX §5.3). Unwinding across an FFI boundary is a compile-time rejection: a Planck function whose effect row contains `panic` must not be exposed to foreign code without the wrapper defined in Volume 10 §16.1. Planck does not deallocate foreign allocations and must not hand foreign code a pointer it did not receive or one allocated by the Planck allocator without an explicit documented transfer (Volume 18 §18.4). Foreign code is an assumption of G2, not a proven component.

### 1.3.12 Compiler trust boundaries (design.md L86)

The trusted computing base is exactly (pinned, SPEC-INDEX §3): the math kernel `math/` (FV-checked per §54), the Z3 scripts `tools/fv/` (timeouts documented), the review process, and the stage 0 compiler written in Rust. Stage 1 is verified against stage 0 by differential equivalence (Volume 22). Nothing outside this list may be trusted without a documented refinement gap and a runtime bridge (standards.md §54.2.2).

### 1.3.13 Feature template: checked integer arithmetic

The 16-row template of design.md §40 applied to the charter-level feature "checked integer arithmetic" (the safety-critical core of §1.3.8):

| Row | Definition |
| --- | ---------- |
| 1. Syntax | Operators `+ - * / % << >>` and unary `-` on integer types (Volume 02 §2.4 table). |
| 2. Static semantics | Operand types must be identical integer types; unsigned unary `-` is rejected; mixing widths is rejected (Volume 04). |
| 3. Type semantics | Result type equals operand type; no promotion exists. |
| 4. Value semantics | In-range mathematical result; overflow is not a value event. |
| 5. Evaluation semantics | Check then operation; panic on violation with exact message (Volume 02 §2.4.4). |
| 6. Ownership | Operands are read in left-to-right order; no moves beyond operand evaluation (Volume 06). |
| 7. Effects | Effect row {panic} on integer operands regardless of build mode; `panic = abort` changes the termination mechanism, not the row (SPEC-INDEX §3). |
| 8. Control flow | Panic unwinds to the enclosing catch boundary, or aborts (exit 134) in abort builds. |
| 9. Compile-time behavior | In const contexts, out-of-range results are compile-time rejections (Volume 09). |
| 10. Memory behavior | None: no memory is accessed beyond operand reads. |
| 11. ABI behavior | Implemented with target add/sub/mul/div instructions plus branch; no ABI-visible difference. |
| 12. IR 1 mapping | Check node followed by Bin node (Volume 02 §2.4.6; Volume 13 node semantics). |
| 13. IR 2 mapping | Check may merge with proven range information; removal only under proof (Volume 14). |
| 14. Machine IR mapping | Target conditional-branch sequences per Volume 15 instruction selection. |
| 15. Diagnostics | Runtime: panic message; compile-time: const-context rejection (Volume 09 catalog). |
| 16. Validation | EXH over {i8, u8} × 7 operators × 65536 pairs (OB-01-2); Z3 for wider types (FV-1 bridge). |

**Integration points.** Depends on: Volumes 02 (operator table), 05-08 (semantics), 10 (intrinsics), 13-15 (lowering). Depended on by: Volume 22 (proof obligations), Volume 24 (security review of unsafe surface).

## 1.4 Platform contract

### 1.4.1 Supported architectures (design.md L90)

Implemented: x86-64 (baseline: x86-64 with SSE2; optional features `avx2`, `avx512f`, `cmpxchg16b`) and AArch64 (AArch64 A-profile, ARMv8.0 baseline; optional feature `lse`). Declared but not implemented in edition P1: RISC-V RV64GC (`riscv64gc-unknown-linux-gnu`), i686 (`i686-unknown-linux-gnu`), ARMv7-M (`thumbv7m-none-eabi`). Implementing a declared target requires no edition change; adding an unlisted target is a specification change (G26).

### 1.4.2 Supported operating systems (design.md L91)

Implemented: Linux (kernel ≥ 5.10; libc glibc ≥ 2.31 or musl 1.2.x), macOS ≥ 12.0, Windows ≥ 10 (UCRT). No other OS is implemented; none is declared.

### 1.4.3 Supported object formats (design.md L92)

ELF64 (Linux targets), Mach-O (macOS targets), PE/COFF (Windows target). Exact container rules are Volume 17 §24.2.

### 1.4.4 Supported executable formats (design.md L93)

ELF `ET_DYN` (PIE is the default; `ET_EXEC` opt-in), Mach-O `MH_EXECUTE`, PE32+. Entry sequences per Volume 17 §24.5.

### 1.4.5 Supported ABIs (design.md L94)

Pinned (SPEC-INDEX §3): x86-64 System V ABI and AArch64 AAPCS64. The Planck default convention is the C ABI at FFI boundaries and the Planck fast convention internally (exact definition: Volume 10). The Windows x64 calling convention applies to the Windows target's FFI boundary. No other ABI is implemented.

### 1.4.6 Cross-compilation (design.md L95)

Targets are selected by `--target <triple>`. The triple grammar is `<arch>-<vendor>-<os>-<abi>` with the exact implemented triples: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`, `x86_64-apple-macos`, `aarch64-apple-macos`, `x86_64-pc-windows-msvc`. Default target is the host triple. Compiling for target T must not read host headers or host libraries; all target facts come from the target description (Volume 16). A triple outside the list is rejected at argument parse time with the exact diagnostic of Volume 21 §28.6.

### 1.4.7 Endianness (design.md L96)

Both implemented architectures are little-endian. The compiler must not assume host endianness for object emission or constant encoding: byte order is an explicit per-target parameter (pinned, SPEC-INDEX §5.9). Big-endian targets would be declared targets; none is declared in P1.

### 1.4.8 Pointer widths (design.md L97)

| Target | Pointer width | `usize` | `isize` |
| ------ | ------------- | ------- | ------- |
| x86-64 targets | 64 bits | 64-bit | 64-bit |
| AArch64 targets | 64 bits | 64-bit | 64-bit |
| i686 (declared) | 32 bits | 32-bit | 32-bit |

### 1.4.9 Atomic capabilities (design.md L98)

x86-64: lock-prefixed read-modify-write for 8/16/32/64-bit widths; 128-bit operations only via `lock cmpxchg16b`, gated on target feature `cmpxchg16b`. AArch64: LSE instructions when feature `lse` is enabled; baseline uses LL/SC sequences. Misaligned atomic access is a defined trap (exit code 134), not UB. The memory model all atomics map to is Volume 08 §12.2.

### 1.4.10 SIMD capabilities (design.md L99)

x86-64 baseline: SSE2; optional `avx2`, `avx512f` (+ declared subfeatures in Volume 16 §23.4). AArch64 baseline: NEON; `sve` is declared, not implemented. SIMD types (Volume 04 §6.1) are usable only when the selected target features enable them; otherwise the use is a compile-time rejection (Volume 04 diagnostics).

### 1.4.11 Operating-system interfaces (design.md L100)

Edition P1 standard library calls libc on Linux and macOS (glibc/musl/libSystem per §1.4.2) and UCRT on Windows; no direct syscall surface exists in the baseline. Freestanding targets have no OS interface; `std`-level I/O items are unavailable there (Volume 19 §26.5 lists the freestanding subset exactly).

### 1.4.12 Hosted and freestanding environments (design.md L101)

Hosted: an environment providing the §1.4.2 OS contract; entry is `fn main()` (exit code 0) or `fn main() -> i32` (exit code is the low 8 bits of the value, two's complement). Freestanding: an environment with no OS contract; the crate must carry `#![no_std]` and define `#[entry] fn name() -> !`. Both environments are defined by their exact entry signatures here; the start code that calls them is Volume 17 §24.5.

**Integration points.** Depends on: Volumes 15-17 (backend, object, linking), 16 (target descriptions), 18 (runtime), 19 (freestanding subset). Depended on by: Volume 16 (target tables must match §1.4), Volume 21 (CLI target validation).

## 1.5 Compatibility policy

### 1.5.1 Source compatibility (design.md L105)

A change is **source-compatible** if and only if every program accepted by compiler version V under edition E compiles under V' ≥ V in edition E with the same observable behavior (G1's observable set). Carve-outs, which are compatible: improved diagnostics, faster compilation, new warnings, and keyword additions (which occur only in new editions, §1.5.8). A change failing this definition is breaking and must follow §1.5.10.

### 1.5.2 Binary compatibility (design.md L106)

Planck guarantees binary compatibility only for the C ABI surface: `extern "C"` function signatures compiled with the same target triple are link-compatible across compiler versions. The Planck fast convention is version-pinned per compiler build and is not stable. Object files produced by different compiler versions have no other compatibility guarantee.

### 1.5.3 Standard library compatibility (design.md L107)

The standard library follows package compatibility (§1.5.5): additions are minor releases; removals and behavior changes are major releases. Layouts of library types follow the pinned default `#[repr(P)]` rules (declaration order, max-field alignment, padded size, no reordering, no niche packing without `#[repr(niche)]`), so a minor release must not change the layout of an existing public type.

### 1.5.4 Compiler version compatibility (design.md L108)

Within an edition, compiler release N must accept every program accepted by release N-1 (§1.5.1) and must produce semantically equivalent output. Two compiler releases may differ in diagnostics, object layout of internal symbols, and optimization decisions. The exact CLI compatibility surface (flags and their stability tiers) is Volume 21 §28.7.

### 1.5.5 Package compatibility (design.md L109)

Package versions and requirements follow SemVer 2.0.0; the exact grammar, comparison, and requirement-operator semantics are Volume 03 §4.14. The policy pinned here: a minor release must not remove or change the behavior of any public item; a major release may; the resolver selects the highest version satisfying all constraints, deterministically (G14).

### 1.5.6 ABI stability (design.md L110)

Pinned: only the C ABI at FFI boundaries is stable in edition P1 (§1.5.2). The default `#[repr(P)]` layout rules are stable across compiler versions for the same edition; `#[repr(C)]`, `#[repr(packed)]`, `#[repr(transparent)]`, and `#[repr(niche)]` layouts are defined in Volume 05 and are stable per edition.

### 1.5.7 Deprecation rules (design.md L111)

A deprecated item carries `#[deprecated(since = "X.Y.Z", note = "...")]` (exact attribute form, Volume 10). Each use produces warning `W-DEP-01` naming the item, the `since` version, and the note. Removal requires: the item was deprecated for at least one minor release cycle (a tagged release of the package), the removal lands in a major release or an edition change, and the migration is expressible by `plkc fix` (Volume 21 §28.8). A deprecation cycle that skips these steps is a compatibility defect.

### 1.5.8 Edition changes (design.md L112)

Editions are named `P1`, `P2`, ...; each edition is identified by an exact delta list relative to the previous edition, published as a change to this library. A crate is compiled entirely in one edition (`--edition`, Volume 21); mixing editions inside a crate is rejected. Editions may: add keywords (§1.5.1 carve-out), change defaults, remove deprecated surface. Editions must not change the semantics of programs that are valid in both editions unless the delta list names the change and `plkc fix` can migrate it (§1.5.10).

### 1.5.9 Language feature stabilization (design.md L113)

A feature is **stabilized** if and only if all four hold: (S1) normative sections exist in this library covering every row of the design.md §40 template for it; (S2) every verification row for it is FV-0, or FV-1 with a named gap and runtime bridge; (S3) conformance tests exist in the Volume 22 suite covering its acceptance and rejection sets; (S4) the review vote is recorded in `spec/VOTE-RECORD.md`. Edition P1 has no unstable surface: a construct is either in the P1 specification or absent from the compiler; there is no nightly flag.

### 1.5.10 Breaking-change policy (design.md L114)

A breaking change requires all three: (B1) a new edition; (B2) a `plkc fix` migration producing the exact rewritten source or, when no rewrite exists, the exact diagnostic naming the incompatibility; (B3) a VOTE-RECORD entry citing the delta list. Any other breaking change is banned. Diagnostics for constructs removed by an edition use the code family `E-EDITION-*` (Volume 21 catalog).

**Integration points.** Depends on: Volumes 03 (SemVer), 10 (attributes), 21 (fix, CLI, diagnostics), 22 (conformance), VOTE-RECORD. Depended on by: Volume 19 (library releases), Volume 20 (package resolution policy).

## 1.6 Known limitations of the charter

The performance numbers of §1.2 are measurement obligations (class FV-2), not machine-checked proofs; the zero-cost cost-parity clause (Z2) is measured, not proven. The UB-closure property (OB-01-1) is review-checked, not mechanically enumerated across volumes. The declared targets of §1.4.1 are not implemented in edition P1. The prelude is empty by design; ergonomics of imports are a Volume 21 tooling matter, not a language one.

## 1.7 Verification matrix

| Obligation ID | Property (exact statement) | Method | Class | Artifact | G-mapping |
| ------------- | -------------------------- | ------ | ----- | -------- | --------- |
| OB-01-1 | Every language operation is classified into exactly one of: defined value, defined trap (exit 134), compile-time rejection, UB-1..UB-6; no operation is unclassified. | REVIEW | FV-1 | This file §1.3.9 + Volumes 04-10 judgment tables | G1, G16 |
| OB-01-2 | For T in {i8, u8}, op in {+,-,*,/,%,<<,>>}, all 65536 operand pairs: panic occurs iff the mathematical result is outside T's range, or divisor is 0, or shift amount ≥ width; else the exact mathematical result is produced. | EXH | FV-0 | math/planck-operator-math tests | G1 |
| OB-01-3 | The ownership transition table is total (6 states × 7 actions) and cell-equal to SPEC-INDEX §5.4. | EXH | FV-0 | math/planck-ownership-math tests | G2 |
| OB-01-4 | Effect lattice: join is associative, commutative, idempotent with identity ∅; meet distributes over join; row subtyping is a partial order; all over the 256-row lattice. | EXH | FV-0 | math/planck-effect-math tests | G1 |
| OB-01-5 | A typed integer literal is well-formed iff min(T) ≤ v ≤ max(T), checked at the 4 boundary values for each of the 16 integer types. | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-01-6 | A char literal denotes exactly one Unicode scalar value; the 8 boundary code points (0, D7FF, D800, DFFF, E000, 10FFFF, 110000, 0) and all escape forms are classified exactly. | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-01-7 | For each abstraction of §1.2.7 and bounded instances (≤ 64 IR 1 nodes), the IR 1 node multiset of the abstraction equals that of its manual expansion. | Z3 + PROP | FV-1 | tools/fv/zerocost_v1.smt2 | G4, G5 |
| OB-01-8 | Each of the 10 kernels of `bench/kernels-v1` executes within 1.10× of its C reference at -O2 on the reference hardware. | REVIEW | FV-2 | bench/kernels-v1 result tables | G13 |
| OB-01-9 | Two builds of `bench/corpus-v1` with identical inputs, configuration, target, and toolchain version produce byte-identical object files for every crate. | EXH | FV-2 | tools/ reproducibility runner | G14 |
| OB-01-10 | The 30 cells of the §1.4 platform tables (6 triples × 5 attributes) agree with the Volume 15/16 target capability tables. | EXH | FV-2 | tools/fv platform-matrix check | G15 |
| OB-01-11 | The impl-legality predicate accepts exactly the headers matching the pinned orphan rule over all 32 header shapes (trait local/foreign × ≤ 4 parameters local/foreign). | EXH | FV-0 | tools/fv orphan check | G3 |
| OB-01-12 | SemVer requirement membership is exact on the grid: versions {0,1,2}^3 × operators {=,^,~,*,*} × {with, without} prerelease (216 cases). | EXH | FV-0 | math/ or tools/fv semver check | G14 |
| OB-01-13 | Startup ≤ 3000000 cycles and minimal-binary size ≤ 2048 bytes on the reference hardware. | REVIEW | FV-2 | bench startup and size reports | G13 |
| OB-01-14 | For every construct outside A1-A3, generated code performs zero allocator calls (bounded instances; PROP paired with Z3). | Z3 + PROP | FV-1 | tools/fv/noalloc.smt2 | G1, G2 |
| OB-01-15 | No edition P1 grammar production forms a detached task: every spawn is lexically inside a task scope. | REVIEW | FV-1 | Volume 02 grammar + Volume 08 | G21 |

## 1.8 Coverage ledger

bullets in scope: 54, defined: 54. Prose requirements in scope: 2, defined: 2.

| design.md bullet | Defined in |
| ---------------- | ---------- |
| L47 Language name and versioning | §1.1.1 |
| L48 Language design principles | §1.1.2 |
| L49 Intended programming domains | §1.1.3 |
| L50 Supported programming paradigms | §1.1.4 |
| L51 Primary implementation goals | §1.1.5 |
| L52 Non-goals | §1.1.6 |
| L53 Compatibility philosophy | §1.1.7 |
| L54 Relationship between language and standard library | §1.1.8 |
| L55 Relationship between language and compiler implementation | §1.1.9 |
| L59 Compilation-time objectives | §1.2.1 |
| L60 Runtime performance objectives | §1.2.2 |
| L61 Memory consumption objectives | §1.2.3 |
| L62 Code-size objectives | §1.2.4 |
| L63 Startup-time objectives | §1.2.5 |
| L64 Allocation behavior | §1.2.6 |
| L65 Zero-cost abstraction definition | §1.2.7 |
| L66 Runtime overhead policy | §1.2.8 |
| L67 Performance predictability | §1.2.9 |
| L68 Compile-time versus runtime computation | §1.2.10 |
| L69 Optimization guarantees versus optimization hints | §1.2.11 |
| L71 (prose) Do not promise that every abstraction is free; define guaranteed vs optimization-dependent costs | §1.2.12 |
| L75 Memory safety | §1.3.1 |
| L76 Type safety | §1.3.2 |
| L77 Initialization safety | §1.3.3 |
| L78 Ownership safety | §1.3.4 |
| L79 Borrowing safety | §1.3.5 |
| L80 Data-race freedom | §1.3.6 |
| L81 Pointer validity | §1.3.7 |
| L82 Integer overflow behavior | §1.3.8, §1.3.13 |
| L83 Undefined behavior categories | §1.3.9 |
| L84 Unsafe-code responsibilities | §1.3.10 |
| L85 FFI safety boundaries | §1.3.11 |
| L86 Compiler trust boundaries | §1.3.12 |
| L90 Supported architectures | §1.4.1 |
| L91 Supported operating systems | §1.4.2 |
| L92 Supported object formats | §1.4.3 |
| L93 Supported executable formats | §1.4.4 |
| L94 Supported ABIs | §1.4.5 |
| L95 Cross-compilation | §1.4.6 |
| L96 Endianness | §1.4.7 |
| L97 Pointer widths | §1.4.8 |
| L98 Atomic capabilities | §1.4.9 |
| L99 SIMD capabilities | §1.4.10 |
| L100 Operating-system interfaces | §1.4.11 |
| L101 Hosted and freestanding environments | §1.4.12 |
| L105 Source compatibility | §1.5.1 |
| L106 Binary compatibility | §1.5.2 |
| L107 Standard library compatibility | §1.5.3 |
| L108 Compiler version compatibility | §1.5.4 |
| L109 Package compatibility | §1.5.5 |
| L110 ABI stability | §1.5.6 |
| L111 Deprecation rules | §1.5.7 |
| L112 Edition changes | §1.5.8 |
| L113 Language feature stabilization | §1.5.9 |
| L114 Breaking-change policy | §1.5.10 |
| L116 (prose) Deliverable: Language Charter and Implementation Requirements Specification | §1.0, this file |
