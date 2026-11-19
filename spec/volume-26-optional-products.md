<!-- CEP:FILE: spec/volume-26-optional-products.md -->
<!-- CEP:WHAT: Volume 26 of the Planck master specification library: the Optional Execution Products Specification. Defines, with exact contracts, the reference interpreter plki (the differential-testing oracle for G1, environment-passing interpretation with a step budget), the JIT compilation product, and the runtime specialization product. The interpreter is required for the verification program; the JIT and runtime specialization are specified-but-not-built in edition P1, with their extra correctness problems stated exactly. -->
<!-- CEP:WHY: design.md §37-§39 (lines 3942-4006) enumerates 42 items for three execution products and states that the JIT and adaptive optimization are not in the initial compiler. Nothing may be dropped: each product gets an exact contract now, so a future implementation cannot redefine semantics silently (G26), and the interpreter can serve as the G1 oracle from the first release. -->
<!-- CEP:STATUS: complete -->
<!-- CEP:FAILURE: An interpreter observable behavior outside the oracle set O of §26.1.0; an interpreter inconsistency (IVE) that is not treated as Severity 0; a JIT page that is writable and executable at the same time; a specialization whose observable behavior depends on which version executed; a guard or limit constant without the exact value stated here; a reserved flag or attribute silently accepted in edition P1. Each of these is a Severity 0 defect. -->
<!-- CEP:ASSUMES: SPEC-INDEX.md §3 (UB model, traps exit 134, provenance and identity model, monomorphization limits, effect atoms) and §5.9 (relocation math); standards.md CEP&CC 0.4 (§3 verbs, Law 1, Law 6); verification standards.md G1, G11, G14, G20, G21, G26; doc standards.md; Volumes 05, 07, 08, 10, 13, 14, 17, 18, 21, 22, 24, 25. -->
<!-- CEP:COST: document for the JIT and runtime specialization products (no runtime cost in edition P1, since they are not built). The interpreter is CEP-2 verification tooling: O(1) amortized per interpretation step, O(log n) worst case for environment operations, with the cost stated per mechanism in §26.1. -->
<!-- CEP:EVIDENCE: the verification matrix (OB-26-01 through OB-26-22) and the coverage ledger (42 design.md bullets) at the end of this file. -->

# Volume 26: Optional execution products

## 26.0 Scope and traceability

This volume defines design.md §37 through §39, lines 3942-4006.

Bullet counts, verified by enumeration of design.md lines 3942-4006: §37 has 14 bullets, §38 has 16, §39 has 12. Bullets in scope: 42, defined: 42. The prose lines are settled as follows: line 3944 (recommendation of a deliberately simple interpreter) is settled by the product contract of §26.1.0 with the clarity law; line 3963 (primary purpose is semantic validation and differential testing) is settled by the oracle contract of §26.1.0; line 3940 (these products are not mandatory for the first production release, but the architecture must leave room) is settled by the status table and the reserved-name mechanism below; lines 3967, 3990, and 4005 (runtime code generation and adaptive optimization, and their correctness cost) are settled by the declared status of §26.2 and §26.3 and by the exact statement of their extra correctness problems.

Implementation status per product, complete:

| Product | Design section | Status in edition P1 | Rationale, exact |
| --- | --- | --- | --- |
| Reference interpreter plki | §37 | Required-for-verification; ships in the toolchain (Volume 25 §25.1.7); not part of the production release criteria | G1 differential testing (Volume 22 §29.5) consumes it as the oracle; its absence blocks the verification program, not the artifact set |
| JIT compilation | §38 | Declared-not-implemented | design.md line 3967: only necessary for runtime code generation; edition P1 does not ship runtime code generation |
| Runtime specialization | §39 | Declared-not-implemented | design.md line 4005: not in the initial compiler; introduces another substantial correctness problem, stated exactly in §26.3.0 |

The "leave room" obligation of line 3940 is discharged by exactly these mechanisms: (1) plki ships in the release artifact set (Volume 25 §25.1.7); (2) the flags `--jit` and `--emit-deopt-metadata` and the attribute `#[adaptive]` are reserved names: in edition P1 they parse and are rejected with E-RESERVED-FLAG or E-RESERVED-ATTR, exit 1, with the message "declared product, not implemented in edition P1"; (3) the Planck fast calling convention of Volume 10 is already the contract the JIT would use (§26.2.7); (4) the debug information format of Volume 17 and the profile format of Volume 21 §28.5 are the formats the JIT would emit (§26.2.12, §26.2.13); (5) the sandbox, cache, and limits contracts of Volume 24 apply to any future JIT background compiler unchanged.

## 26.1 Reference interpreter (design.md §37)

### 26.1.0 Product identity and oracle contract

The reference interpreter is the binary `plki` (Volume 25 §25.1.7). Purpose, normative: plki is the semantic-validation and differential-testing oracle of the compiler program; its observable behavior on a program is the definition that compiled execution is tested against (G1, Volume 22 §29.5). Clarity law: plki must not contain any optimization pass; no transformation may exist in plki whose purpose is to reduce work rather than to clarify semantics; a violation is a Severity 0 defect because it makes the oracle a second compiler with its own correctness problem.

**Observable behavior** is the set O, exactly: (1) the sequence of bytes written to stdout; (2) the sequence of bytes written to stderr; (3) the process exit code; (4) the ordered sequence of file-system effects performed through the standard library io interface (operation, path, bytes); (5) panic message lines (exact text and location, §26.1.9); (6) trap terminations (exit 134 and the trap name line, Volume 24 §24.6.1). Timing, memory use, and the step budget of §26.1.14 are not observable. Oracle boundary, exact: differential testing applies to programs whose effect row is a subset of {io, alloc, panic}; a program whose row contains concurrency, async, ffi, or nondet is tested by the concurrency and simulation suites of Volumes 08 and 22, not by byte-level differential testing against compiled execution.

Interpreter determinism: for the same program, arguments, stdin bytes, and nondet seed (unused in the differential mode), plki produces byte-identical output in O across runs (G14). An **interpreter inconsistency (IVE)** is a detected state that the semantics forbid on a fully accepted program (for instance reading a Moved cell): plki prints `interpreter inconsistency: <kind>` and exits 125. An IVE on a program accepted by the full compiler pipeline is a Severity 0 defect in the interpreter or the compiler; it is never counted as a differential failure of the program under test.

### 26.1.1 Value representation

```rust
enum IVal {
    Unit,
    Bool(bool),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    F32(f32), F64(f64),
    Char(char),                       // one Unicode scalar value, surrogates impossible
    Str(Vec<u8>),                     // UTF-8 bytes with enforced validity
    Array { elem_ty: TyId, items: Vec<IVal> },
    Struct(StructId, Vec<IVal>),      // fields in declaration order
    Enum(EnumId, VariantId, Vec<IVal>),
    Tuple(Vec<IVal>),
    Ref { alloc: AllocId, offset: u64, pointee_ty: TyId },
    RawPtr { addr: u64, prov: Provenance },
    Fn(FnId, SubstId),                // SubstId: the monomorphized instance
}
```

All integer and float widths are the exact machine widths of Volume 05 with their exact overflow and NaN behavior (fidelity per SPEC-INDEX §1.4); no arbitrary-precision value exists at interpreter runtime. `Provenance` is the SPEC-INDEX §3 model: allocation id, offset, capability set. plki does not evaluate const items; it consumes their already-evaluated values from the compiler (const evaluation is Volume 09).

### 26.1.2 Execution state

plki evaluates the typed, borrow-checked AST of the program; it does not interpret IR. The state is σ = (heap, frames, out, counters). A frame is {fn: FnId, subst: SubstId, env: Env, resume: NodeId}, where Env maps every in-scope place to its location (AllocId, offset). Evaluation is environment-passing: every expression is evaluated by a total function eval(node, σ, env) → IVal or effect; no global mutable evaluation stack beyond frames exists. Interpretation advances one AST node per step; the step is the unit of the budget (§26.1.14) and of the trace (§26.1.13).

### 26.1.3 Memory model

The heap is a vector of allocations: allocation i has {align: u32, size: u64, cells: Vec<Cell>}, and a cell is {state: Full(IVal) | Moved | Uninit}. AllocId starts at 1 and increases monotonically; identity is (AllocId, offset) per SPEC-INDEX §3, and pointer equality and comparison are equality and comparison of (AllocId, offset), never of addresses. Every access is bounds-checked and state-checked: reading a Moved or Uninit cell, or any out-of-bounds offset, is an IVE with kind bounds or state, because the borrow checker and type checker must have rejected such programs; the interpreter therefore holds as an invariant that it never performs an undefined access on an accepted program (OB-26-07). Dangling provenance is impossible for the same reason: a freed allocation is marked dead and any Ref into it is an IVE kind freed.

### 26.1.4 Ownership behavior

Moves and drops execute exactly per Volume 06 and Volume 05: MoveOut sets the source cell to Moved and initializes the destination cell; borrows are already checked statically and execute as copies of locations; drop glue runs scoped, innermost-first, at scope exit on every path, with user Drop first and fields in declaration order per Volume 05. The differential corpus contains drop-order witnesses (programs whose stdout prints a marker per drop), and plki must match compiled execution on them (OB-26-06).

### 26.1.5 Evaluation order

Operand order is exactly the Volume 07 order: operands left-to-right, receiver before arguments, struct fields in declaration order, array elements in index order, block statements in order; `&&` and `||` short-circuit; `match` takes the first matching arm in source order. The differential corpus contains order witnesses (programs whose stdout prints an evaluation marker per side effect), and plki must match compiled execution on them (OB-26-05).

### 26.1.6 Control-flow behavior

If, loop, while, match, break, continue, and return execute per the Volume 07 rules with source-level control flow; break and continue target the innermost matching construct in source text. No control-flow transformation exists in plki (clarity law): the interpreter executes the source-level graph, so control-flow behavior is trivially the specified one, and the differential corpus checks it against compiled execution.

### 26.1.7 Function calls

A call: evaluates arguments left-to-right, pushes a frame, moves each argument into the parameter place (MoveOut from the argument, Full into the parameter), and executes the body. Tail-call optimization is banned in plki: it changes stack-depth behavior, and stack-depth behavior is observable through the recursion limit of §26.1.14. Return: the returned value is moved out, the frame is dropped (drop glue innermost-first), and evaluation resumes at the caller.

### 26.1.8 Generic execution

Generics execute by substitution: plki monomorphizes on demand, with the monomorphization depth limit 32 and the total-instance limit 2^20 of Table 24.1 (SPEC-INDEX §3 pins the depth), and it shares the compiler's instance cache so that a differential test compares compiled and interpreted execution of the same instance. Exceeding either limit is the defined interpreter rejection IE-LIMIT-INST, exit 1; the compiler rejects the same program with the same limit code, and the match of rejection behavior is part of the oracle contract (OB-26-04).

### 26.1.9 Panic behavior

Panic is the defined unwinding of SPEC-INDEX §3: plki unwinds frames innermost-first, runs drop glue at each frame, and delivers the payload at the enclosing catch boundary. With `panic = abort`, the panic terminates immediately. An uncaught panic prints exactly one stderr line `panic: <message> at <file>:<line>:<col>` and exits 101; an abort-mode uncaught panic terminates with exit 134. These two outcomes are part of O and are pinned for both plki and compiled execution (Volumes 07 and 18 state the same values; this volume is the differential pin).

### 26.1.10 Effects

The effect row of the program determines what plki executes: io is executed for real through the same standard library interface as compiled code, and its byte and ordering effects are part of O; alloc operates on the §26.1.3 heap; panic per §26.1.9. Concurrency, when present, executes deterministically: threads are serialized with a round-robin schedule, one atomic action per thread per scheduling step, in spawn order (thread with the lowest id first), until all threads finish; this schedule is deterministic and part of the interpreter contract, and differential tests for concurrent programs compare against this schedule rather than against compiled scheduling (OB-26-09). The nondet atom, when present in differential mode, is a defined interpreter rejection IE-NONDET before execution, exit 1, because the oracle boundary excludes nondet (§26.1.0).

### 26.1.11 Async execution, if supported

Async is supported, deterministically, as follows: plki runs a single-threaded executor over the task tree of Volume 08 (structured concurrency); scheduling is depth-first over the task tree in creation order; one scheduling step is one poll of one task up to its next await point; cancellation is cooperative per Volume 08 (a cancelled parent cancels children before its join point completes); wakers are internal objects with no external visibility. The executor consumes steps from the same budget as synchronous execution, so async programs cannot escape the §26.1.14 limits.

### 26.1.12 Foreign function simulation

plki never executes real foreign code: the oracle must be self-contained. Every extern declaration used by a program run under plki must be bound to a simulation function by the attribute `#[ffi_sim(name = "<extern-name>")]` on a Planck function whose signature matches the declaration. The simulation registry is resolved before execution; an unbound extern is the defined rejection E-SIM-MISSING, exit 1, naming the declaration; a runtime fault inside foreign code is impossible by construction. Simulation functions are ordinary Planck functions: they consume steps, allocate on the interpreter heap, and may raise panics; their effects are part of O.

### 26.1.13 Debugging

Three facilities, exact: `--trace` prints one line per step (step number, node kind, source location, at most 80 characters); `--break <file>:<line>` stops before evaluating the first node on that line and enters a command loop with the commands step, locals, heap <n> (print allocation n), continue, quit; `--trace-io` prints each io operation with its arguments before performing it. Debugging must not change O: with or without these flags, the program's observable behavior is identical (OB-26-08); the trace and the command loop are on stderr and are excluded from O by the oracle contract's byte rule only when these flags are active, which differential tests never set.

### 26.1.14 Execution limits

| Limit | Default | Override | On exceed |
| --- | --- | --- | --- |
| Steps | 10^9 | `--steps N`, 1 ≤ N ≤ 10^12 | IE-LIMIT-STEPS, exit 1, with the step count |
| Heap cells | 10^8 | none | IE-LIMIT-CELLS, exit 1 |
| Allocations | 10^8 | none | IE-LIMIT-ALLOCS, exit 1 |
| Frame depth | 10^6 | none | IE-LIMIT-DEPTH, exit 1 |
| Wall clock | 600 seconds | `--timeout S`, 1 ≤ S ≤ 86400 | IE-LIMIT-TIME, exit 1 |

A limit trip is a defined interpreter failure: the differential harness records it as a test failure to investigate, never as a pass. Termination of plki follows from the step budget (G11): every loop of the interpreter consumes at least one step per iteration, so execution ends within the budget. Cost per Law 1: each step is O(1) amortized; environment operations are O(log n) worst case and O(1) amortized (persistent map); there is no hidden cost because every mechanism above states its cost class (CEP-2 tooling).

## 26.2 JIT compilation (design.md §38)

### 26.2.0 Status and the extra correctness problems

Status: Declared-not-implemented in edition P1 (§26.0). The contract is fixed now; an implementation must satisfy it without renegotiation. A JIT is a compiler whose compilation unit, compilation time, and execution time all interleave with the program's own execution; its extra correctness problems, beyond the ordinary compiler's, are stated exactly here and each is an obligation row: (P1) semantic equivalence with the interpreter oracle under mid-execution code replacement; (P2) exact state reconstruction at deoptimization; (P3) two-phase memory-permission transitions with concurrent readers, no torn code; (P4) instruction-cache coherence across cores after code writes; (P5) relocation range failures possible only at runtime (no static bound check possible for dynamic symbols); (P6) in-flight invocation safety during code invalidation (use-after-free of code memory); (P7) memory ordering between the compiling thread and executing threads; (P8) guard soundness: specialized code must not execute outside its guard domain. All eight are discharged by the matrix rows OB-26-10 through OB-26-17.

### 26.2.1 JIT compilation API

The API is a future crate `plk-jit` with exactly this surface:

```rust
fn jit_create(cfg: JitConfig) -> Result<JitHandle, JitError>;
fn jit_compile(h: JitHandle, unit: JitUnit) -> Result<Vec<JitFn>, JitError>;
fn jit_invoke(h: JitHandle, f: JitFn, args: &[JitValue]) -> Result<JitValue, JitError>;
fn jit_invalidate(h: JitHandle, f: JitFn) -> Result<(), JitError>;
fn jit_destroy(h: JitHandle) -> Result<(), JitError>;
```

JitConfig fields, exact: target triple; opt level, one of O0, O1, O2; code cache size in bytes, default 67108864 (64 MiB); deoptimization enabled, bool. JitValue is the IVal set of §26.1.1, all variants. JitError variants, exactly 8 and complete: OutOfCodeMemory, RelocRange, SymbolMissing, PermissionFailed, UnsupportedTarget, InvalidUnit, ConventionMismatch, StepBudget. Semantics contract: jit_invoke(f, args) computes the same result and the same O-effects as executing f by plki (§26.1) on the same arguments; this is the JIT correctness theorem (P1), the G1 oracle relation ported to runtime codegen.

### 26.2.2 Executable memory allocation

The code cache is a set of slabs of 16777216 bytes (16 MiB) each; allocation within a slab is bump-pointer; the total cache is capped at the configured size (default 64 MiB) and OutOfCodeMemory is the defined failure when the cap is exceeded; the cache never grows silently beyond the cap. Function entry alignment is 16 bytes on both implemented targets. Memory is obtained by mmap with protections read-write initially; the permission transition is §26.2.3. Allocation behavior is deterministic per compilation sequence (addresses are not part of O).

### 26.2.3 Memory permission transitions

A page is never writable and executable at the same time. The protocol, exact: (1) write the new code with the target pages read-write; (2) quiesce all writer threads (the install barrier of §26.2.4); (3) transition the pages read-write to read-execute in one mprotect call per page range; (4) perform the icache synchronization of §26.2.11. Patching live code is banned: a patch goes through the per-slab shadow copy (a read-write copy of the slab maintained off to the side), then a copy-back plus flip cycle through (2) to (4). A violation of the protocol is a bug in the JIT itself and is the defined trap jit-wxor-violation, exit 134, in the host process (P3, OB-26-11).

### 26.2.4 Code installation

Installation of a compilation batch: (1) resolve all relocations of the batch (§26.2.8); (2) place the functions at slab offsets; (3) publish the batch into the function table: the table is a snapshot structure, readers fetch an immutable snapshot, installers append the new entries and flip the published pointer with a release store (P7); (4) run the §26.2.3 permission transition and §26.2.11 synchronization; (5) the previous versions remain valid until invalidated (§26.2.9). Atomicity: an invocation sees either the old or the new version of a function, never a mix (OB-26-12).

### 26.2.5 Symbol resolution

Symbol resolution order, exact: (1) JIT-local symbols of the same batch; (2) symbols exported by the host program's loaded modules; (3) the runtime's registered JIT symbol table (entries added by the host through the registration function of the API); (4) failure SymbolMissing. Names are mangled per Volume 10. dlopen at runtime is banned: the JIT never loads shared libraries; SymbolMissing is the defined failure for every unresolved symbol.

### 26.2.6 Runtime linking

Relocations use the reloc kinds and patch math of SPEC-INDEX §5.9 (kinds Abs64, Abs32, PcRel32, PcRel16, ArmAdrp, ArmBranch26, X86RipRel32; checked field extraction and patching; PcRel32 range −2^31 ≤ target − site + addend ≤ 2^31 − 1). A range violation at install time is JitError::RelocRange (P5: dynamic symbols have no static bound; the check happens at install, where it is exact). Lazy binding is banned: every relocation is resolved at install; a reference to a function compiled later is not expressible because a JitUnit is a closed set of functions resolved within the batch; a reference that escapes the batch and the resolution order is SymbolMissing.

### 26.2.7 Calling conventions

JIT-compiled functions use the Planck fast convention of Volume 10, exactly; a JIT function invoked from compiled code must have been compiled with the Planck fast convention; a callback from JIT code into host code that crosses an FFI boundary uses the C ABI per Volume 10. Stack layout and callee-saved discipline are Volume 15's. A JitFn carries its convention; invoking with a mismatched convention is JitError::ConventionMismatch at the API level; bypassing the API to call with a wrong convention is UB-4 per SPEC-INDEX §3.

### 26.2.8 Code relocation

Relocation entries are computed at install time per §26.2.6; there is no relocation work at invocation time. A relocation referencing a symbol resolved later in the same batch is resolved within the batch in dependency order (the batch is a closed set, §26.2.6); the dependency order is a topological order per SPEC-INDEX §5.7 (stable tie-break by ascending node id, cycle = SymbolMissing for the cyclic entries).

### 26.2.9 Code invalidation

jit_invalidate(h, f): (1) mark the function table entry dead in a new snapshot; new invocations of f execute the baseline definition (the original non-JIT body, always retained); (2) in-flight invocations hold a per-function reference count; the memory of the old version returns to the slab free list only when that count reaches zero (P6: no use-after-free of code memory, OB-26-13); (3) reused memory is rewritten only through the §26.2.3 protocol, so stale instruction bytes are never executed. Demotion is atomic with the same snapshot rule as installation.

### 26.2.10 Thread safety

jit_create and jit_destroy are exclusive: they must not run concurrently with any other call on the same handle; violating this precondition is the defined trap jit-handle-misuse, exit 134, in the host process. jit_compile, jit_invoke, and jit_invalidate are safe to call concurrently. Memory ordering, exact: the function table flip uses release on the writer and acquire on the reader; profiling counters use relaxed ordering (their values are advisory, §26.2.13); the patch shadow copy is protected by a per-slab lock. This discharges P7 at the contract level (OB-26-12 covers the protocol).

### 26.2.11 Instruction-cache synchronization

Per target, after writing code bytes and before publishing, the installer executes exactly this sequence: AArch64: `dc cvau` on every 64-byte line covering the written range, then `ic ivau` on the same lines, then `dsb ish`, then `isb`. x86-64: `mfence` on the writer after the final byte store; readers acquire the table snapshot and execute `lfence` before entering newly installed code (cross-modifying code protocol). The per-target sequence is a two-row table in the implementation, checked by review (OB-26-14) because ISA behavior is not machine-checkable.

### 26.2.12 Debug information

Each installed function records: a line table of (code offset, source span) pairs in the Volume 17 format; a frame-size record (frame size and callee-saved register list) sufficient for the unwinder to walk JIT frames; and its mangled name for the symbolizer. The debugger of Volume 21 §28.4 consumes the line tables; stack traces of JIT frames interleave with compiled frames using the same unwinder. Deoptimization metadata (§26.2.16) is recorded per version when deopt is enabled.

### 26.2.13 Profiling

Per-function counters: invocation_count (u64, exact, increments per jit_invoke) and self_time sampling (advisory, sampled by the host profiler, not part of O). Output uses the profile format of Volume 21 §28.5. Only invocation_count is exact; sampling values are advisory and excluded from the correctness contract (Law 1: the cost is stated, the counter overhead is one relaxed increment per invocation).

### 26.2.14 JIT security

Rules, exact: (1) the JIT accepts only compiler-emitted unit IR: a typed IR 2 subset with a total validity checker run before compilation; a malformed unit is JitError::InvalidUnit; the checker is part of the TCB for the JIT product (G16 note: it must be listed in the TCB when the product ships). (2) No API accepts machine bytes: generating code from raw bytes provided at runtime is banned. (3) W^X per §26.2.3. (4) Capability: a JitConfig flag io_allowed, default false; when false, JIT code cannot perform io because the io operations are not linked into the unit; a unit that references io while io_allowed is false is JitError::InvalidUnit. (5) Units containing values marked `#[sensitive]` are rejected with JitError::InvalidUnit: secrets must not be JIT-compiled (Volume 24 §24.6.8). (6) JIT code runs with the host's privileges; there is no JIT-side sandbox, and this limitation is stated, not hidden.

### 26.2.15 Dynamic specialization

Dynamic specialization in the JIT is recompilation of a function with constants substituted from runtime observations; it exists only together with the guard and deoptimization contracts of §26.3 (which define guards, versions, and dispatch). The specialization correctness obligation: for every input x satisfying the guard set G of version v, the observable behavior of v on x equals the observable behavior of the baseline on x (P8; OB-26-16).

### 26.2.16 Deoptimization, if supported

Settled: deoptimization is required whenever dynamic specialization exists; it is the mechanism that preserves soundness when a guard fails. Contract, exact: on guard failure, execution transfers to plki's execution model (§26.1) at the recorded resume point; each version records deopt metadata: for every live JIT value, its interpreter place (AllocId, offset) per the §26.1.2 environment model, and the resume NodeId. Obligations: (a) the reconstructed interpreter state is observationally equivalent to never having left the interpreter: same O, modulo the step budget (P2, OB-26-15); (b) deoptimization is O(live values) time and allocates at most a 4096-byte scratch buffer; (c) after 3 guard failures of any guard of a version, the version is demoted permanently (§26.3.8).

## 26.3 Runtime specialization (design.md §39)

### 26.3.0 Status and the named correctness problems

Status: Declared-not-implemented in edition P1 (§26.0). design.md line 4005 names "another substantial correctness problem"; it is stated exactly here: (R1) the guard soundness theorem: a specialized version must compute the same observable behavior as the baseline for every input satisfying its guards; (R2) deoptimization state reconstruction (§26.2.16); (R3) profile-adaptive nondeterminism: which version executes is timing-dependent, so observable behavior must be version-independent for every input; this is the core theorem of the product and the reason the whole design is guard-based. Every obligation below serves R1 through R3.

### 26.3.1 Runtime profiling

Profile points: loop back-edges and function entries of functions marked `#[adaptive]` (the attribute is reserved in edition P1, §26.0). Each site has one u64 counter, saturating at 18446744073709551615 (2^64 − 1), never wrapping (OB-26-18); increments are relaxed-atomic. The profile is advisory: it selects which versions get built, never what they compute; observable behavior is governed by the guard contract alone (R3).

### 26.3.2 Value profiling

A value profile records, per site, a table of at most 4 observed values. Eligible value classes, exactly: integers of width at most 64 bits, bool, char, enum discriminants. Aggregates, floats, and references are not eligible. Ties in frequency are broken by first-seen order (deterministic, G14). The table stores exact values, not hashes.

### 26.3.3 Type profiling

A type profile records, per cast or check site, a table of at most 8 observed dynamic type identifiers (enum variant ids, trait-object vtable ids). Replacement policy: evict the least-frequent entry; ties by oldest observation (deterministic). The table stores exact identifiers.

### 26.3.4 Hot-path detection

A site is hot when its counter is at least 10000 (10^4). A function is hot when its entry counter is at least 100000 (10^5). These thresholds are named constants; only they and the §26.3.5 thresholds trigger compilation, and nothing else does.

### 26.3.5 Compilation thresholds

Specialization compilation for a site triggers exactly when: the site is hot (§26.3.4), and the profile has at least 100 samples at the site, and the most frequent value or type accounts for at least 80 of those 100 samples. Compilation runs on the background compilation thread (§26.3.12: exactly one thread, pool size 1) with a soft budget of 100 milliseconds; exceeding the budget abandons the attempt, leaves the profile running, and retries when the site counter doubles since the attempt.

### 26.3.6 Specialized code versions

A version is the triple (function id, guard set, specialized body). Versions are immutable after install. Version numbering: a per-function u32 counter starting at 0; the baseline is version 0 and is never replaced. A function has at most 8 versions (baseline included); a compilation request beyond the eighth is dropped and the profile continues. A version is installed only after its guard soundness check (§26.3.7) passes at compile time: every guard must be checkable at runtime for the values it claims to fix.

### 26.3.7 Guard construction

Guards are exactly 5 forms: value-eq (place holds the recorded constant), type-eq (dynamic type equals the recorded id), shape-eq (array length equals n), global-not-mutated (no write to global g since version install), callee-version (each callee still at the version expected at compile time). Guards are checked in the recorded order at version entry; all must pass for the specialized body to execute. Cost contract per Law 1: a guard compiles to a straight-line comparison sequence of at most 10 machine instructions, with no call and no allocation (OB-26-17); this makes guard checks CEP-0 eligible hot code.

### 26.3.8 Guard failure

On the first failing guard: transfer to deoptimization per §26.2.16; record one failure in the site's failure counter; the failing input is observable only through timing, never through O (R3: the deoptimized continuation computes the same O as the baseline, OB-26-16). After 3 failures of any guard of version v, v is demoted: removed from dispatch, invocations execute the baseline; demotion uses the snapshot protocol of §26.2.4.

### 26.3.9 Deoptimization

The deoptimization contract is §26.2.16 in full: metadata per version (live value to interpreter place, resume node), observational equivalence of the reconstructed state (P2/R2), O(live values) time, at most a 4096-byte scratch buffer, and demotion after 3 failures. Runtime specialization inherits the contract by reference; it adds nothing to it.

### 26.3.10 Recompilation

After demotion of version v, a recompilation for the same site may trigger when the site counter doubles relative to the last attempt; the new version's guard set is the old guard set minus the value-eq entries whose values failed (type-eq, shape-eq, global-not-mutated, and callee-version guards are retained or recomputed from the fresh profile). At most 3 recompilation attempts per site per process; after the third, the site stays on the baseline permanently.

### 26.3.11 Code version management

Dispatch per adaptive function: fetch the current version-table snapshot; check the guards of versions in descending install order (newest first); execute the first version whose guards all pass; if none pass, execute the baseline. The check order is a named constant of the design (newest-first), deterministic per snapshot (G14). Install and demote are snapshot flips per §26.2.4; versions share the code cache and the §26.2.2 and §26.2.9 contracts; all versions are discarded at process exit (nothing persists).

### 26.3.12 Concurrent compilation

Hazards, exactly 5, each with its resolution: (H1) the compilation thread mutates the version table while executors dispatch: resolved by the snapshot protocol (immutable snapshots, release-store flip, §26.2.4); (H2) the compiler reads profile counters while executors increment them: counters are relaxed-atomic u64; a stale read causes at most a stale compilation decision, never unsound code, because guards re-check everything at runtime (R1); (H3) two compilations of the same function: excluded by construction, since there is exactly one background compilation thread (pool size 1); (H4) deoptimization while another thread compiles: independent, because deopt touches only the current thread's state; (H5) invalidation during deoptimization: no interaction, because the deopt target is the interpreter, never JIT code. Each hazard is an obligation row (OB-26-21 reviews the table against this list).

## Verification matrix

| ID | Source | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- | --- |
| OB-26-01 | §37 | For every program P of the differential corpus: plki and compiled execution agree on all of O | EXH over the differential corpus of Volume 22 §29.5 | FV-0 | CI job diff-test | G1, G20 |
| OB-26-02 | §37.0 | Every differential corpus program has an effect row that is a subset of {io, alloc, panic} | EXH over the corpus programs (row check per program) | FV-0 | CI job diff-test | G1 |
| OB-26-03 | §37.2, §37.14 | Every interpreter loop consumes at least one step per iteration, so execution ends within the step budget | REVIEW of the three interpretation loops (step counter is the termination measure) | FV-2 | review record of plki | G11 |
| OB-26-04 | §37.8 | Compiler and interpreter agree on generic-limit rejections: for generated programs with monomorphization depth 1..40, both reject exactly those with depth > 32 | EXH over 40 generated programs | FV-0 | CI job limit-parity | G1, G13 |
| OB-26-05 | §37.5 | plki matches compiled execution on every evaluation-order witness of the Volume 07 order corpus | EXH over the order corpus | FV-0 | CI job diff-test | G1 |
| OB-26-06 | §37.4 | plki matches compiled execution on every drop-order witness of the Volume 05 drop corpus | EXH over the drop corpus | FV-0 | CI job diff-test | G1 |
| OB-26-07 | §37.3 | No IVE occurs on any program of the accepted corpus (no bounds, state, or freed access) | EXH over the differential corpus | FV-0 | CI job diff-test | G1, G12 |
| OB-26-08 | §37.13 | Debugging does not change O: for every corpus program, O is identical with and without --trace, --break, --trace-io | EXH over the corpus x 4 flag combinations | FV-0 | CI job interp-debug-neutrality | G1, G14 |
| OB-26-09 | §37.10 | Concurrent programs under plki produce the output of the round-robin schedule (spawn order, quantum 1) exactly | EXH over the Volume 08 concurrency corpus (serialized comparison) | FV-0 | CI job interp-concurrency | G21, G14 |
| OB-26-10 | §38.1 | The JitError enum is complete: every failure cause of §38 maps to exactly one of the 8 variants | EXH over the 8 variants x the failure-cause list | FV-0 | tools/fv/ob-26-10.smt2 | G20 |
| OB-26-11 | §38.3 | The page state machine admits only {rw, rx, none}: no transition produces rwx; domain: 3 states x 5 events {alloc, seal, patch-begin, patch-end, free} | EXH over the 15-cell transition table | FV-0 | tools/fv/ob-26-11.smt2 | G20 |
| OB-26-12 | §38.4, §38.10 | Installation and invalidation are atomic per snapshot: an invocation sees exactly one version; readers never observe a partial table | REVIEW of the snapshot protocol (release/acquire argument) plus PROP over interleavings of 2 installers x 2 readers x 8 operations | FV-1 (gap: interleaving sampling; bridge: snapshot immutability argued structurally) | tools/fv/ob-26-12 harness | G21, G20 |
| OB-26-13 | §38.9 | Invalidation never frees in-use code: old-version memory is reused only when its reference count is zero | REVIEW of the reference-count protocol | FV-2 | review record of plk-jit | G20 |
| OB-26-14 | §38.11 | The icache sync sequence per target matches §26.2.11 exactly (AArch64: dc cvau, ic ivau, dsb ish, isb; x86-64: mfence writer, lfence reader) | REVIEW against the ISA manuals (ISA behavior is not machine-checkable) | FV-1 (gap: ISA semantics; bridge: exact instruction sequence per target) | review record per target | G20 |
| OB-26-15 | §38.16 | Deoptimization reconstructs an observationally equivalent interpreter state: for the deopt corpus, O after deopt equals O of pure interpretation | PROP over the deopt replay corpus, paired with REVIEW of the metadata completeness argument | FV-1 (gap: corpus sampling; bridge: metadata covers every live value by construction) | CI job deopt-replay | G1, G20 |
| OB-26-16 | §38.15, §39.8 | Version equivalence: for every version v and input x satisfying the guards of v, the observable behavior of v equals the baseline | Z3 on a bounded model (2 versions, 2 guard forms, 4 input classes), plus PROP over the specialization corpus | FV-1 (gap: bounded model plus sampling; bridge: guard checking at every entry) | tools/fv/ob-26-16.smt2 | G1, G20 |
| OB-26-17 | §39.7 | Every guard form compiles to a straight-line sequence of at most 10 instructions with no call and no allocation | REVIEW of the emitted sequence per guard form (5 forms) | FV-2 | review record per form | G20, Law 1 |
| OB-26-18 | §39.1 | Profile counters saturate at 2^64 − 1 and never wrap | EXH over the 2 boundary behaviors (increment at max, increment below max) | FV-0 | tools/fv/ob-26-18.smt2 | G20 |
| OB-26-19 | §39.11 | Dispatch is deterministic per snapshot: newest-first order over at most 8 versions with first-passing-guard selection; domain: 8 versions x 2^8 guard outcomes (2048 cases) | EXH over 2048 cases | FV-0 | tools/fv/ob-26-19.smt2 | G14, G20 |
| OB-26-20 | §39.4-6 | Threshold constants are exact and exhaustive: site hot 10^4, function hot 10^5, samples 100, share 80, demote 3, retries 3, versions 8 | EXH over the 7 constants x their boundary values | FV-0 | tools/fv/ob-26-20.smt2 | G13, G20 |
| OB-26-21 | §39.12 | The hazard table is complete and each resolution is sound: H1 snapshot protocol, H2 relaxed counters plus runtime guards, H3 single compiler thread, H4 thread-local deopt, H5 interpreter target | REVIEW of the hazard table against §26.3.12 | FV-2 | review record of plk-adaptive | G21 |
| OB-26-22 | §37.12 | Every extern declaration of a plki-run program is bound by #[ffi_sim] before execution; unbound externs are rejected with E-SIM-MISSING | EXH over the corpus crates with extern declarations (bound and unbound cases enumerated) | FV-0 | CI job sim-registry | G1, G22 |

## Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| --- | --- |
| §37 Value representation | §26.1.1 |
| §37 Execution state | §26.1.2 |
| §37 Memory model | §26.1.3 |
| §37 Ownership behavior | §26.1.4 |
| §37 Evaluation order | §26.1.5 |
| §37 Control-flow behavior | §26.1.6 |
| §37 Function calls | §26.1.7 |
| §37 Generic execution | §26.1.8 |
| §37 Panic behavior | §26.1.9 |
| §37 Effects | §26.1.10 |
| §37 Async execution, if supported | §26.1.11 |
| §37 Foreign function simulation | §26.1.12 |
| §37 Debugging | §26.1.13 |
| §37 Execution limits | §26.1.14 |
| §38 JIT compilation API | §26.2.1 |
| §38 Executable memory allocation | §26.2.2 |
| §38 Memory permission transitions | §26.2.3 |
| §38 Code installation | §26.2.4 |
| §38 Symbol resolution | §26.2.5 |
| §38 Runtime linking | §26.2.6 |
| §38 Calling conventions | §26.2.7 |
| §38 Code relocation | §26.2.8 |
| §38 Code invalidation | §26.2.9 |
| §38 Thread safety | §26.2.10 |
| §38 Instruction-cache synchronization | §26.2.11 |
| §38 Debug information | §26.2.12 |
| §38 Profiling | §26.2.13 |
| §38 JIT security | §26.2.14 |
| §38 Dynamic specialization | §26.2.15, §26.3 |
| §38 Deoptimization, if supported | §26.2.16, §26.3.9 |
| §39 Runtime profiling | §26.3.1 |
| §39 Value profiling | §26.3.2 |
| §39 Type profiling | §26.3.3 |
| §39 Hot-path detection | §26.3.4 |
| §39 Compilation thresholds | §26.3.5 |
| §39 Specialized code versions | §26.3.6 |
| §39 Guard construction | §26.3.7 |
| §39 Guard failure | §26.3.8 |
| §39 Deoptimization | §26.3.9, §26.2.16 |
| §39 Recompilation | §26.3.10 |
| §39 Code version management | §26.3.11 |
| §39 Concurrent compilation | §26.3.12 |

Bullets in scope: 42, defined: 42.
