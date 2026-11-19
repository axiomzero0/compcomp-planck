<!-- CEP:FILE: spec/volume-24-security-reliability.md -->
<!-- CEP:WHAT: Volume 24 of the Planck master specification library: the Compiler Security and Reliability Specification. Defines the compiler threat model (adversaries A1-A8), the untrusted-input contract and resource limits (Table 24.1), build execution sandboxing, supply-chain integrity, the compiler reliability and failure taxonomy, and runtime hardening for compiled programs. -->
<!-- CEP:WHY: design.md §32 (lines 3784-3855) enumerates 47 security and reliability items as bare bullets. Without exact definitions, malformed input could corrupt the compiler or its outputs instead of producing a defined compilation failure, violating G12, G13, G22 and standards.md §22. -->
<!-- CEP:STATUS: complete -->
<!-- CEP:FAILURE: A limit outside Table 24.1; an input for which the compiler judgment is undefined; a failure path without an exit code and diagnostic from the taxonomy of §24.2.11; a sandbox capability outside §24.3; a release artifact without signature and checksum; a runtime trap outside the §24.6 trap set. Each of these is a Severity 0 defect per standards.md §22. -->
<!-- CEP:ASSUMES: SPEC-INDEX.md §3 (settled decisions: UB model, traps exit 134, provenance model, bootstrap trust) and §5 (pinned kernel contracts); standards.md CEP&CC 0.4 (§3 normative verbs, §17 review checklist, §22 security, Laws 6 and 7); verification standards.md G11, G12, G13, G14, G16, G20, G22; doc standards.md; Volumes 02, 05, 07, 10, 12, 17, 18, 19, 20, 21, 22, 23. -->
<!-- CEP:COST: document. Compile-time cost of the security checks of this volume is O(1) per checked item (one counter comparison or one validation pass over the validated object). Runtime cost of §24.6 hardening is stated per option in §24.6.3 and §24.6.4. -->
<!-- CEP:EVIDENCE: the verification matrix (OB-24-01 through OB-24-22) and the coverage ledger (47 design.md bullets) at the end of this file. -->

# Volume 24: Compiler security and reliability

## 24.0 Scope and traceability

This volume defines design.md §32, lines 3784-3855 ("Compiler security"), and is the deliverable named at line 3852: the Compiler Security and Reliability Specification.

Bullet counts, verified by enumeration of design.md lines 3784-3855: §32.1 has 11 bullets, §32.2 has 10, §32.3 has 8, §32.4 has 10, §32.5 has 8. Bullets in scope: 47, defined: 47. The prose lines 3786-3788 ("A production compiler processes untrusted source code, dependencies, build scripts, and potentially compile-time programs. It therefore needs its own security model.") are settled by the threat model of §24.1.

Precedence. Where this volume fixes a value that another volume also fixes, the values are identical by construction: the Volume 12 limits table incorporates Table 24.1 by reference and must not state a wider value for any quantity of Table 24.1; a narrower value in Volume 12 governs. Volumes 05, 06, 07, 18 govern language and runtime semantics; this volume adds the security envelope around them and never redefines a semantic rule.

Code classes. The compiler, the package manager, and the release tooling are CEP-2 tooling per standards.md §3; the math kernels they call are CEP-0 per SPEC-INDEX §5.2; the runtime code specified in §24.6 is CEP-0 or CEP-1 as stated per option. CEP-2 status does not waive determinism obligations where G14 applies: compilation output is deterministic per G14 even though tooling code may use the platform allocator and libc.

## 24.1 Threat model

### 24.1.1 Trusted and untrusted input inventory

Trusted input: the compiler release artifacts after successful signature and checksum verification (§24.4), the math kernels and FV scripts (SPEC-INDEX §3 TCB), the pinned stage-0 Rust toolchain recorded by hash (§24.4.7), and the operating system primitives the compiler calls (filesystem, process, memory).

Untrusted input: every byte of source text, every package tarball and manifest fetched from a registry or mirror, every build script and procedural macro (their code and their behavior at execution time), every compile-time program executed during const evaluation, every cache entry read from disk, every environment variable not on the allowlist of §24.3.7, and every value arriving through FFI at runtime.

### 24.1.2 Adversaries

| ID | Adversary | Capabilities | Trust boundary | Goal | Forbidden outcome | Acceptable outcome |
| --- | --- | --- | --- | --- | --- | --- |
| A1 | Untrusted source author | Supplies any byte stream as source, up to LIM-INPUT-TOTAL; constructs pathological but in-limit programs | Source text | Crash or corrupt the compiler; make the compiler emit wrong code; leak secrets | Compiler-side memory corruption; silent wrong output | Defined rejection, exit 1, diagnostics |
| A2 | Malicious dependency author | Fully controls package content, build script, proc macro behavior within the sandbox | Registry fetch; process sandbox | Escape the sandbox; read or write host files; reach the network; exfiltrate environment | Sandbox escape; secret exfiltration; network reach from sandbox | E-SANDBOX termination, exit 4 |
| A3 | Compromised registry or mirror | Serves arbitrary bytes for any path; replays old manifests and tarballs; cannot forge Ed25519 signatures | Signature and checksum verification | Substitute package or compiler content | Acceptance of unsigned or hash-mismatched content | Defined fetch rejection |
| A4 | Build-environment attacker | Controls environment variables, PATH, working directory contents, TMPDIR | Process spawn (allowlist env, fixed PATH, §24.3.7) | Influence build semantics through undeclared state | Build result depends on undeclared environment | Undeclared variables are not passed; nothing to exploit |
| A5 | Network attacker | Passive and active man-in-the-middle on fetches | TLS 1.3 plus signatures plus checksums | Substitute content in transit | Content substitution accepted | Fetch rejection |
| A6 | Compromised release CI | Arbitrary code execution in CI; holds CI secrets; no access to offline signing keys | Offline signing host (§24.4.1) | Publish a malicious signed release | Signed malicious artifact | Signing impossible without the offline key; reproducibility check detects divergence |
| A7 | End-user attacker on the target machine | Runs the produced binary; tampers with the installed toolchain | Runtime hardening (§24.6); plkc doctor --verify (§24.4.5) | Memory corruption beyond the UB model; silent tampering | Memory corruption outside UB-1..UB-6; silent acceptance of tampering | Defined trap, exit 134; E-TAMPER |
| A8 | Co-resident process | Consumes CPU, memory, disk, network; no privileges over the build | Resource limits (Table 24.1) | Denial of service against a build | Compiler corruption under pressure | Defined resource failure, exit 2 or 3 |

### 24.1.3 Trusted actors and assets

Trusted actors: T1, the holders of the release root key (exactly 2 named maintainers, key kept offline); T2, the signing host (never networked during signing); T3, the math kernels and Z3 scripts per SPEC-INDEX §3; T4, the pinned Rust toolchain that builds stage 0, identified by hash (§24.4.7).

Assets: integrity of the compiler process; integrity of build outputs; confidentiality of user source and user secrets; secrecy and integrity of signing keys; integrity of the registry index; integrity of the build cache.

### 24.1.4 Standards.md §22.2 questions, answered

What input is trusted and untrusted: §24.1.1. What boundaries exist: source text boundary, package fetch boundary, sandbox boundary, environment boundary, cache load boundary, release verification boundary, FFI boundary at runtime. What privileges the compiler holds: read of declared inputs, write of the target directory, cache directory, and diagnostic files, spawn of the §24.3.6 process list, no network. What secrets the compiler touches: none; the compiler must not read, log, or transmit secret material; `#[sensitive]` values are a runtime concern (§24.6.8). What resources can be exhausted: memory, disk, CPU time, wall clock, process count; each has a limit in Table 24.1. What failure mode is acceptable under attack: a defined compilation failure from the taxonomy of §24.2.11. What failure mode is forbidden: memory corruption, secret leakage, signature bypass, silent output corruption, unbounded failure (Law 6).

Rationale: the security objective is the standards.md §22.1 objective, restated with the enumerated adversaries above.

## 24.2 Compiler input security (design.md §32.1)

### 24.2.0 Untrusted input contract and the limits table

The compiler computes, for every input, exactly one judgment: Accept, or Reject with a reason code. A Reject is a **defined compilation failure**: the compiler emits the diagnostic named by the reason code, writes no further output artifacts, and exits with the code of the taxonomy in §24.2.11. There is no input for which the judgment is undefined. This contract is the malformed-input rule of G12 and G13: malformed input is a defined compilation failure, never corruption, never a crash, never silent acceptance.

Every counter over untrusted input is compared against Table 24.1 before the resource is consumed. Values are exact; units are stated.

| ID | Quantity | Exact value | On exceed |
| --- | --- | --- | --- |
| LIM-FILE-SIZE | Source file size | 16777216 bytes (16 MiB) | Reject LIM-FILE-SIZE, exit 1 |
| LIM-INPUT-TOTAL | Total input bytes per invocation | 1073741824 (1 GiB) | Reject LIM-INPUT-TOTAL, exit 1 |
| LIM-TOKENS | Tokens per file | 16777216 (2^24) | Reject LIM-TOKENS, exit 1 |
| LIM-AST-FILE | AST nodes per file | 16777216 (2^24) | Reject LIM-AST-FILE, exit 1 |
| LIM-AST-CRATE | AST nodes per crate | 67108864 (2^26) | Reject LIM-AST-CRATE, exit 1 |
| LIM-NEST | Syntactic nesting depth | 256 | Reject LIM-NEST, exit 1 |
| LIM-SEM-DEPTH | Semantic nesting depth (explicit-stack phases) | 1024 | Reject LIM-SEM-DEPTH, exit 1 |
| LIM-TRAIT-DEPTH | Trait solver recursion depth | 64 | Reject LIM-TRAIT-DEPTH, exit 1 |
| LIM-TRAIT-STEPS | Trait solver steps per crate | 1048576 (2^20) | Reject LIM-TRAIT-STEPS, exit 1 |
| LIM-INST-DEPTH | Monomorphization depth | 32 (pinned by SPEC-INDEX §3) | Reject LIM-INST-DEPTH, exit 1 |
| LIM-INST-TOTAL | Total generic instances per crate | 1048576 (2^20) | Reject LIM-INST-TOTAL, exit 1 |
| LIM-CONST-STEPS | Const-eval steps per const item | 10000000 (10^7) | Reject LIM-CONST-STEPS, exit 1 |
| LIM-CONST-DEPTH | Const-eval recursion depth | 256 | Reject LIM-CONST-DEPTH, exit 1 |
| LIM-CONST-MEM | Const value footprint | 1048576 bytes (1 MiB) | Reject LIM-CONST-MEM, exit 1 |
| LIM-MACRO-DEPTH | Macro expansion nesting depth | 128 | Reject LIM-MACRO-DEPTH, exit 1 |
| LIM-MACRO-TOKENS | Tokens emitted per single macro expansion | 1048576 (2^20) | Reject LIM-MACRO-TOKENS, exit 1 |
| LIM-MACRO-STEPS | Total macro expansion steps per crate | 1000000 (10^6) | Reject LIM-MACRO-STEPS, exit 1 |
| LIM-IR-FN | IR nodes per function | 16777216 (2^24) | Reject LIM-IR-FN, exit 1 |
| LIM-IR-CRATE | IR nodes per crate | 67108864 (2^26) | Reject LIM-IR-CRATE, exit 1 |
| LIM-BB | Basic blocks per function | 1048576 (2^20) | Reject LIM-BB, exit 1 |
| LIM-DIAG | Diagnostics per compilation | 100 | Stop after diagnostic 100, emit E-DIAG-LIMIT with the total count, exit 1 |
| LIM-RECOVER | Recovered errors per compiler phase | 64 | Abort the phase with E-DIAG-LIMIT, exit 1 |
| LIM-MEM | Compiler memory budget | 8589934592 bytes (8 GiB) default; --mem-budget accepts multiples of 268435456 (256 MiB), minimum 1073741824 (1 GiB) | E-OOM, exit 2 (§24.2.9) |
| LIM-PROC-TIME | Build script or proc-macro process wall clock | 60 seconds default; --build-timeout accepts 1 to 3600 seconds | Kill the process, E-SANDBOX-LIMIT, exit 4 |
| LIM-PROC-MEM | Build script or proc-macro address space | 2147483648 bytes (2 GiB) | Kill the process, E-SANDBOX-LIMIT, exit 4 |
| LIM-PROC-WRITE | Bytes written to OUT_DIR per build-script process | 268435456 (256 MiB) | Kill the process, E-SANDBOX-LIMIT, exit 4 |
| LIM-MACRO-TIME | One procedural macro expansion wall clock | 10 seconds | Kill the host, E-SANDBOX-LIMIT, exit 4 |
| LIM-MACRO-OUT | Macro output pipe capacity per expansion | 67108864 bytes (64 MiB) | Kill the host, E-SANDBOX-LIMIT, exit 4 |

### 24.2.1 Malformed source handling

Source text is a sequence of bytes. The lexer maps every byte value 0 through 255 to exactly one of: token, whitespace, comment content, or lexer error. There is no byte value for which the lexer is undefined (G12). The byte 0x00 is not whitespace (SPEC-INDEX §5.10); it produces the lexer error E-NUL. A UTF-8 BOM (EF BB BF) is permitted only at byte offset 0 and is skipped; a BOM elsewhere produces E-ENC. An invalid UTF-8 byte produces diagnostic E-ENC at the byte offset, is replaced by U+FFFD for the remainder of the pass, and marks the file rejected; lexing continues so that one pass reports every encoding error in the file. A file with any lexer error is rejected at end of file with exit 1; its AST is never lowered further (per-file gating, §24.5.6).

Malformed source is therefore always a defined compilation failure: at most LIM-DIAG diagnostics are emitted, the exit code is 1, and no object file is written. The compiler must not read past the end of any buffer derived from input, must not index by an unchecked length, and must not deserialize any input into a structure without a total validity judgment (G12; the cache loader of §24.3.10 is the one deserializer, and it is total).

### 24.2.2 Parser resource limits

The parser enforces LIM-FILE-SIZE, LIM-TOKENS, LIM-AST-FILE, LIM-AST-CRATE, LIM-NEST, LIM-DIAG, and LIM-RECOVER from Table 24.1. Counters are exact integers incremented at allocation sites; each counter is checked before allocation, so an exceed is observed before memory for the exceeded item is committed. Termination of the parser is guaranteed by LIM-TOKENS and LIM-NEST alone, independent of input size (G11): the parser consumes at most LIM-TOKENS tokens and recurses at most LIM-NEST deep.

### 24.2.3 Recursion limits

Syntactic recursion is bounded by LIM-NEST (256). Semantic recursion in lowering, type checking, and borrow checking is executed on explicit stacks, bounded by LIM-SEM-DEPTH (1024); the bound is enforced by stack length, not by native stack depth, so the compiler process cannot overflow its native stack from input depth alone. Native stack use per explicit-stack frame is bounded by the implementation contract in Volume 12; the compiler must not recurse natively on input-derived depth.

### 24.2.4 Generic instantiation limits

Monomorphization is bounded by LIM-INST-DEPTH (32, pinned by SPEC-INDEX §3) and LIM-INST-TOTAL (2^20 per crate). The instance counter counts distinct instances; re-instantiating an existing instance is free and does not consume budget. On exceed, the compiler emits the rejection with the instantiation chain (up to 32 entries, innermost last) so the user can locate the cycle. Termination of monomorphization follows from LIM-INST-TOTAL (G11).

### 24.2.5 Trait solver limits

The trait solver is bounded by LIM-TRAIT-DEPTH (64) and LIM-TRAIT-STEPS (2^20 per crate). A step is one goal reduction. The solver caches proven goals; the cache holds at most 2^24 entries, and insertion beyond the cache bound evicts in insertion order (FIFO). On exceed of either bound, the solver returns the rejection LIM-TRAIT-STEPS or LIM-TRAIT-DEPTH with the goal stack; the rejected goal is reported as a failed constraint with an explanation per G22 ("useful explanations of failed constraints": the explanation names the goal chain). Termination follows from the step bound (G11).

### 24.2.6 Constant evaluation limits

Const evaluation is bounded by LIM-CONST-STEPS (10^7 per const item), LIM-CONST-DEPTH (256), and LIM-CONST-MEM (1 MiB footprint of the value under construction). A step is one evaluation of one const expression node. On exceed, the const item is rejected with the corresponding code and the const item path; evaluation never loops without consuming steps (G11), and overflow inside const evaluation is a compile-time rejection per SPEC-INDEX §5.6, never a runtime event.

### 24.2.7 Macro expansion limits

Macro expansion is bounded by LIM-MACRO-DEPTH (128 nesting levels), LIM-MACRO-TOKENS (2^20 tokens per single expansion), and LIM-MACRO-STEPS (10^6 expansion steps per crate). A step is one invocation of one macro or one pass of its output through expansion. For procedural macros the additional wall-clock and pipe limits LIM-MACRO-TIME and LIM-MACRO-OUT apply (§24.3.2). On exceed, the expansion site is rejected with the code and the macro path chain.

### 24.2.8 Graph size limits

The IR graph of a function is bounded by LIM-IR-FN (2^24 nodes), LIM-BB (2^20 basic blocks), and a CFG edge bound of 2^24 edges per function; the crate graph is bounded by LIM-IR-CRATE (2^26 nodes). These are the security-side values of the graph bound referenced by SPEC-INDEX §5.7; the Volume 12 limits table states the same values. Lowering and every pass must check node and block counters before insertion. On exceed, lowering stops and the function is rejected with LIM-IR-FN, LIM-BB, or LIM-IR-CRATE.

### 24.2.9 Memory exhaustion behavior

The compiler allocates through tracked arenas with one process-wide budget LIM-MEM. When an allocation is refused, either by the budget or by the operating system, the compiler: stops the current compilation unit, emits E-OOM naming the phase and the budget value, releases the arena of the failed unit, and exits 2. The compiler must not continue in a degraded mode after a memory failure. The failure path itself allocates nothing beyond one pre-reserved 64 KiB diagnostic buffer, so out-of-memory handling cannot itself run out of memory. A8 pressure therefore yields exit 2 with a defined message, never corruption and never a hang.

### 24.2.10 Integer overflow in compiler internals

All compiler arithmetic over input-derived quantities (sizes, counts, offsets, lengths, indices) is checked arithmetic; on overflow the compiler emits E-INTERNAL-OVERFLOW, which is an ICE-class failure per §24.5.2, and exits 101. The compiler is built with overflow checks enabled in all profiles; this is a build setting, verifiable from the build script of Volume 12. Arithmetic over compiler-internal quantities not derived from input (for instance, the step counter of a fixed-size ring buffer with a proven bound) must carry a proof or documented bound per Law 6. Unchecked wrapping arithmetic is banned on input-derived quantities.

### 24.2.11 Compiler crash handling and the failure taxonomy

Every compiler termination is in exactly one class of this taxonomy. Exit codes are a complete enumeration; no other exit code is emitted by compilation commands (subcommands declare their own exhaustive sets where stated: §24.4.8 and Volume 25 §25.1.10).

| Exit | Class | Meaning |
| --- | --- | --- |
| 0 | success | Compilation succeeded |
| 1 | defined compilation failure | Diagnostic rejection C0, limit rejection C1, semantic rejection C2 |
| 2 | resource failure | E-OOM, E-DISK-FULL after one retry (§24.5.9) |
| 3 | environment failure | Missing or unreadable input, missing external tool, E-TAMPER, unusable cache directory |
| 4 | build-execution failure | E-SANDBOX, E-SANDBOX-LIMIT, build script or proc macro failure |
| 101 | internal compiler error | Panic, assertion failure, E-INTERNAL-OVERFLOW, fault signal |
| 130 | interrupted | SIGINT, after cleanup (§24.5.8) |
| 143 | interrupted | SIGTERM, after cleanup (§24.5.8) |

A crash of the compiler (panic, assertion, SIGSEGV, SIGBUS, SIGILL, E-INTERNAL-OVERFLOW) is an **internal compiler error (ICE)**. On ICE the compiler: writes the ICE report of §24.5.2, prints one stderr line `internal compiler error: <phase> (<class>), report: <path>`, and exits 101. Signal handlers for SIGSEGV, SIGBUS, and SIGILL write the report using only a pre-allocated async-signal-safe buffer of 4096 bytes with the fields marked "signal-safe subset" below; exit is by `_exit(101)`. Exit code 134 is reserved to compiled-program traps per SPEC-INDEX §3 and must not be used by the compiler.

## 24.3 Build execution security (design.md §32.2)

### 24.3.1 Build script sandboxing

Build scripts execute out-of-process under the runner `plkc-run` with a default-deny capability set. Granted capabilities: read of the package source directory, read of declared dependency source directories, write of the single directory PLANCK_OUT_DIR, and the environment of §24.3.7. All other filesystem access is denied; network access is denied (§24.3.5); process spawning is denied (§24.3.6). Enforcement uses the platform mechanism exactly: Linux seccomp-BPF filter plus Landlock rules; macOS sandbox-exec with the compiled profile stored in the toolchain; Windows AppContainer. Where the platform mechanism is unavailable, the build fails with E-SANDBOX unless the user passes `--allow-unsafe-build`, which records `"sandbox": "absent"` in the build manifest and makes `--frozen` builds refuse the package. Sandbox violations detected by the mechanism terminate the process; `plkc-run` reports E-SANDBOX and plkc exits 4.

### 24.3.2 Procedural macro sandboxing

Procedural macros execute out-of-process by default in the same sandbox as build scripts, with a reduced capability set: read of the package source directory, no write capability anywhere, no network, no process spawn. Communication with the compiler uses a request-response pipe with per-expansion limits LIM-MACRO-TIME (10 s) and LIM-MACRO-OUT (64 MiB); exceeding either kills the macro host and fails the build with E-SANDBOX-LIMIT, exit 4. In-process execution is permitted only under `--unsafe-macros-inproc`, which the compiler records in the build manifest; `--frozen` builds reject crates whose macros ran in-process. Determinism: on the first build of a crate, every macro expansion is executed twice and the two outputs are compared byte-wise; a difference fails the crate with E-MACRO-NONDET, exit 1, since expansion output feeds compilation (G14).

### 24.3.3 Compile-time execution restrictions

Compile-time execution (const evaluation, inline const, const generics evaluation) is interpreted, never native-compiled, and runs inside the compiler process under the step, depth, and memory limits of §24.2.6. The effect row available at compile time is exactly {panic}; the atoms io, alloc, unsafe, async, concurrency, ffi, nondet are compile-time rejections in const context per Volume 09. Compile-time execution cannot open files, cannot spawn, and cannot touch the network because the effect atoms that would grant those capabilities are not part of the compile-time row and the interpreter implements no such operations.

### 24.3.4 Filesystem access controls

The compiler process reads exactly: explicit source paths from the command line or manifest, dependency checkouts under the package cache, the cache directory, the sysroot, and target-directory entries it wrote itself. It writes exactly: the target directory, the cache directory, and diagnostic files (ICE reports under the cache directory). Paths arriving from untrusted inputs (package manifests, lockfiles) are validated by the total predicate `valid_path(s)`: s is non-empty; s is at most 4096 bytes; s contains no NUL; s is relative; s has no ".." component; s has no empty component other than a trailing one. A path failing `valid_path` yields E-PATH, exit 1. Symlinks: source inputs inside a package root must not escape the root through symlinks; the resolver resolves each component and rejects any resolution outside the root with E-PATH. Cache directory symlinks are rejected with E-PATH (§24.3.10).

### 24.3.5 Network access controls

The compiler proper (plkc and plkc-run) performs no network I/O; this is a property of its code, checked by the sandbox filters of §24.3.1 which deny socket syscalls outright. All fetching belongs to the package manager (Volume 20), which must use TLS 1.3 exactly for registry fetches and must verify checksums and signatures per §24.4. Build scripts and proc macros have network denied by the sandbox. Under `--offline` the package manager additionally denies all socket syscalls by sandbox filter; a fetch attempt then fails with E-OFFLINE, exit 3.

### 24.3.6 Process execution controls

The compiler and build system spawn exactly this enumerated set of processes, and no other spawn is permitted: (1) build scripts, via plkc-run; (2) proc-macro hosts, via plkc-run; (3) the linker named by the target configuration; (4) the C compiler for C dependencies, named by the target configuration; (5) the archiver; (6) debugger and profiling tools invoked by explicit user commands of Volume 21. Arguments are passed as argv arrays, never through a shell; a spawn request containing shell metacharacter interpretation is a construction error at the call site. Compiler-internal tools are invoked by absolute path recorded at install time; PATH is consulted only for the user-named external tools (3), (4), (5), (6).

### 24.3.7 Environment isolation

plkc reads exactly these environment variables and no others: PLANCK_TOOLCHAIN, PLANCK_CACHE_DIR, PLANCK_TARGET, PLANCK_OFFLINE, TMPDIR, HOME, PATH. All other variables are ignored; reading any other variable in compiler code is a review-blocking defect per standards.md §22.3.

Build scripts and proc-macro processes receive exactly this environment: PLANCK_OUT_DIR, PLANCK_PACKAGE_NAME, PLANCK_PACKAGE_VERSION, PLANCK_TARGET_TRIPLE, PLANCK_PROFILE, PLANCK_BUILD_ID, PLANCK_SYSROOT, PATH with the fixed value `<sysroot>/bin`, and TMPDIR set to a fresh empty temporary directory created for the invocation. No other variable is passed, so A4 has no channel into the build. The parent process environment is never forwarded.

### 24.3.8 Resource limits

Build-script and proc-macro processes run under LIM-PROC-TIME (wall clock, default 60 s), LIM-PROC-MEM (address space, 2 GiB), LIM-PROC-WRITE (OUT_DIR bytes, 256 MiB), and a thread limit of 4 threads per process. Enforcement is by OS mechanism where available (rlimit on POSIX, Job Objects on Windows) and by the sandbox otherwise. On exceed, the process is killed and the build fails with E-SANDBOX-LIMIT, exit 4. The compiler process itself is bounded by LIM-MEM and, when invoked under the test harness of Volume 22, by the compile-time limits of Volume 23 §31.1.

### 24.3.9 Dependency trust

Workspace members are trusted code. Dependencies are untrusted: their source compiles normally, their build scripts and proc macros run only inside the §24.3 sandbox. A workspace manifest may set per-package `trust = "sandboxed"` (default) or `trust = "none"`; `trust = "none"` refuses to build any build script or proc macro of that package, failing with E-DEP-SCRIPT, exit 1, listing the offending packages. `--no-build-scripts` denies build scripts and proc macros of all dependencies the same way. A dependency tarball is accepted for extraction only after checksum and signature verification per §24.4.2 and §24.4.3, and extraction applies the path rules of §24.3.4.

### 24.3.10 Build cache trust

A cache entry is the byte sequence: header, then payload. The header is a fixed 128-byte structure:

```text
offset 0   magic[10]   = "PLCKCACHE1"
offset 10  schema: u32 = 1
offset 14  key: [u8; 32]      (SHA-256 content key)
offset 46  created_unix: u64
offset 54  comp_ver: [u8; 16] (first 16 bytes of SHA-256 over the compiler version string)
offset 70  payload_len: u64
offset 78  payload_sha256: [u8; 32]
offset 110 reserved: 18 bytes, zero   (header total: 128 bytes)
```

The loader is total: it reads the header, checks magic, schema, key match, compiler version match, payload length, and recomputes payload_sha256 over the payload. Any failure, including short files, I/O errors, and hash mismatch, classifies the entry as corrupt: the entry is deleted and its value is recomputed by normal compilation. A corrupt entry is never used and never causes a diagnostic beyond one W-CACHE-CORRUPT warning per invocation. Cross-version reuse is impossible because comp_ver differs per compiler version; a rollback (Volume 25 §25.1.13) therefore recomputes, never reuses. A symlinked or otherwise non-regular cache entry is rejected with E-PATH and recomputed. This loader is the single deserializer of untrusted bytes in the compiler (G12: no invalid deserialization).

## 24.4 Supply-chain security (design.md §32.3)

### 24.4.1 Signed compiler releases

Every release artifact set is described by a release manifest, signed with Ed25519 by the release root key held offline by T1 on host T2. The signature file is minisign-format: one untrusted-comment line, one base64 signature line over the manifest bytes. `plkc verify-release --manifest M --sig S` verifies and exits 0 on success, 1 on failure; the root public key is published in the repository file `release-keys.md` with its 64-hex-character public key and its fingerprint, and the installer prints the fingerprint before verification (Volume 25 §25.1.10). Key rotation: a new root key must be signed by the previous root key (continuity signature) and published at least 30 days before the previous key is withdrawn; verifiers accept either key during the overlap and only the continuity chain after.

### 24.4.2 Signed package manifests

The registry index entry for a package version (name, version, dependency list, yank status, checksum list) is signed with Ed25519 by the registry signing key. The client verifies the signature before using an index entry and refuses unsigned or invalid entries with a defined fetch rejection. The tarball checksum recorded in the entry is the SHA-256 of the tarball bytes; a mismatch rejects the fetch and marks the cached index entry bad.

### 24.4.3 Dependency provenance

Every published package artifact carries a provenance manifest in the text format `planck-provenance/1`:

```text
subject: <name>@<version>
source: <vcs-url>
revision: <40 lowercase hex commit>
builder: plkc <version> channel <channel>
inputs: <sha256>  <relative-path>     (one line per input)
artifacts: <sha256>  <relative-path>  (one line per artifact)
timestamp: <unix-seconds u64>
```

The file is signed with Ed25519 by the builder key of the publishing CI. Verification obligation: for every published artifact, a provenance manifest exists, its artifact lines match the published artifact hashes, and its input lines match the lockfile-resolved dependency hashes. The provenance chain is: source commit, tarball hash, build input hashes, artifact hashes; each link is a hash comparison, checkable offline.

### 24.4.4 Build artifact integrity

Build outputs under the target directory are content-addressed: the build manifest records each artifact with its SHA-256 and its content key. Reuse of an artifact across builds requires hash equality (§24.3.10); a mismatch recomputes. The build manifest itself is written atomically (§24.5.8) and lists: artifacts with hashes, compiler version, flags, environment values from the §24.3.7 allowlist, and the sandbox status per package.

### 24.4.5 Compiler checksum validation

Every installed toolchain writes `<sysroot>/manifest.sha256` listing the SHA-256 of every binary and data file it ships, in the format `<64 lowercase hex>  <relative path>`. `plkc doctor --verify` recomputes all hashes and compares; any mismatch is E-TAMPER, exit 3, and `--frozen` builds refuse to run against a tampered toolchain. The installer runs the same verification at install time by default (Volume 25 §25.1.10).

### 24.4.6 Reproducible releases

A release is reproducible when two independent builds from identical inputs produce byte-identical artifacts (G14, and the reproducibility success criterion of verification standards.md §9). Inputs are pinned and recorded in the release manifest: source tarball SHA-256, stage-0 toolchain hash, target list, and the exact build command line. Nondeterminism sources are banned: embedded timestamps are the constant 0 (SOURCE_DATE_EPOCH is honored and defaults to 0); paths in debug info are relative, and any unavoidable absolute value is hashed before embedding; the build uses no randomness (any PRNG is seeded with the build ID). The CI gate `repro-check` builds the release twice and compares the artifact hash set; inequality fails the release. `plkc verify-rebuild` reproduces one artifact locally from the recorded inputs and compares hashes.

### 24.4.7 Bootstrap integrity

Stage 0 is the pinned Rust toolchain recorded in `bootstrap/pinned-stage0.txt` (version string and SHA-256 of the toolchain archive); T4 is trusted for it. Stage 1 is Planck-compiled Planck. The release publishes stage-1 outputs only when the stage-1/stage-2 equality check of Volume 22 (design.md §30.7, G25) passed on CI: for every program P of the conformance corpus, hash(stage1_output(P)) = hash(stage2_output(P)). The release manifest records both the stage-0 hash and the equality-check verdict. Bootstrap replacement attacks are bounded by A3: stage-0 archives are fetched with checksum verification against the pinned value.

### 24.4.8 Dependency vulnerability handling

The advisory database uses the text format `planck-advisory/1`: fields id (PLK-YYYY-NNNN), package, affected (a set of comparators, each `op version` with op in {<, <=, >, >=, =}, matched by semver precedence; the set matches if any comparator matches), patched (version list), severity (one of critical, high, moderate, low), title, description of at most 2000 characters, references (URL list). Advisory files are signed with the registry key. `plkc audit` compares the lockfile against the advisory database; exit codes for this subcommand are exhaustive: 0 clean, 1 findings at or above the threshold (default: moderate; `--audit-threshold` accepts one of the four severities), 3 database unreadable, 4 fetch failure. A package version with an unpatched critical advisory must not be added as a new dependency when the workspace lint `deny-critical-advisories = true` (default) is set: resolution fails with E-ADVISORY, exit 1, naming the advisory. `plkc publish` refuses to publish a new version of a package whose declared dependency set matches an unpatched critical advisory, exit 1.

## 24.5 Compiler reliability (design.md §32.4)

### 24.5.1 Graceful error handling

Every compiler phase returns an explicit Result; every error enum is complete and listed in its CEP:FAILURE (Law 6). Phases continue after recoverable errors to report further diagnostics, up to LIM-RECOVER (64 recovered errors per phase) and LIM-DIAG (100 diagnostics); past either bound the phase aborts with E-DIAG-LIMIT and the summary count. When a phase produced any error, downstream phases run in validation-only mode: they consume the artifacts of the failed phase, report their own errors against them, and write no output artifacts. Final exit is 1 whenever any error was reported. Panic on user input is banned: a panic reachable from input handling is an ICE (§24.5.2) and a Severity 0 defect.

### 24.5.2 Internal compiler error reports

The ICE report is a JSON document of at most 1 MiB written to `<cache-dir>/ice/plkc-ice-<unix-seconds>-<pid>.json`, with the printed path in the stderr line of §24.2.11. Fields, exactly: schema (literal "planck-ice/1"); report_id (32 lowercase hex, random per report); timestamp_unix (u64); compiler_version (the §25 version string); channel; target_triple; host_triple; phase (one of lexer, parser, resolve, typecheck, borrowck, consteval, macroexp, lower-ir1, ir2, sel, regalloc, asm, obj, link); class (one of panic, assert, overflow, signal); message (at most 512 bytes, control bytes 0x00-0x1F and 0x7F removed); backtrace (10 frames: instruction pointer, symbol, file and line when available); argv; input_paths (paths only); env_allowlist (the values of §24.3.7); reduceable (bool). The signal-safe subset written from the SIGSEGV/SIGBUS/SIGILL handler is: schema, report_id, timestamp, phase, class "signal", message, argv. The report must not contain source text, file contents, environment values outside the allowlist, or memory dumps (§22.1 secret-leakage prohibition; obligation OB-24-05).

### 24.5.3 Crash reproduction

`plkc ice-repro <report.json>` attempts to reproduce and minimize an ICE: (1) re-run the stored argv against the stored input paths; (2) if the ICE reproduces and `reduceable` is true, apply delta debugging over the input files at line granularity, then token granularity, with at most 10^4 test executions and a 10-second timeout per execution; (3) write the minimized input set and invocation to `ice-repro/` next to the report. Termination measure: the test-execution counter bounded by 10^4. If the ICE does not reproduce deterministically, the tool records class "nondet" on the report; a nondeterministic ICE is Severity 0 and takes fix priority over all non-S0 work. Reproduction of an input-dependent ICE succeeds by construction of (1); the obligation is the termination bound and the report correctness, not reproducibility of nondeterministic faults.

### 24.5.4 Compiler state dumps

With `--ice-dump` (default off; enabled by the environment variable PLANCK_ICE_DUMP with value 1), an ICE additionally writes a state dump next to the report: the IR 1 graph of the function under compilation in the Volume 13 text format, the phase name, and the phase-local counters. The dump is at most 256 MiB; truncation appends the marker line `--- truncated ---`. The dump is not written by the signal handler.

### 24.5.5 Diagnostic quality

Diagnostics satisfy G22 by the following rules. R1 location: the primary span covers exactly the tokens that caused the failure. R2 classification: each diagnostic is one of error, warning, note, and carries a stable code (E-numbered, W-numbered, LIM-numbered). R3 no misleading diagnostics: a diagnostic must not name a location that did not contribute to the failure; conformance is checked by the diagnostic corpus, in which every case pins the expected code and span. R4 ordering: diagnostics sort by (file path, byte offset, code); the sort is total and stable. R5 determinism: rendering contains no timestamps, no addresses, no pointer values, no thread identifiers. R6 explanations: every code has an entry in the diagnostics reference, reachable as `plkc explain <code>`, containing one paragraph of definition and the conditions that produce it. R7 no silently ignored errors: every Result in compiler code is consumed; the compiler build denies the unused-must-use lint class. R8 volume: at most LIM-DIAG diagnostics, then E-DIAG-LIMIT. Columns are 1-based byte columns; output is UTF-8.

### 24.5.6 Recovery from malformed source

Recovery is defined, per file: an invalid byte is replaced (§24.2.1); a syntax error triggers the recovery productions of Volume 02 (skip to the next statement boundary, or balance delimiters up to depth 16), after which parsing continues; a file with any lexer or parser error is never lowered to IR. Cross-file: other files continue to be checked so their diagnostics are reported in the same invocation. Invariants: after a malformed input the compiler emits only defined diagnostics, exits 1, and writes no object file; the set of diagnostics is deterministic per G14.

### 24.5.7 Corrupt cache recovery

Cache loading uses the total loader of §24.3.10: corruption yields deletion and recompute, never use of the corrupt value and never a failure beyond the single W-CACHE-CORRUPT warning. A cache directory that cannot be read or written (I/O error, wrong permissions) switches the invocation to an in-memory cache and emits W-CACHE-OFFLINE once. The result of a compilation with a cold, corrupt, or offline cache is byte-identical to the result with a warm cache (obligation OB-24-16 ties this to G23).

### 24.5.8 Interrupted build recovery

All artifact writes use the write-then-rename pattern: write to `<path>.tmp`, flush, rename to the final path. On SIGINT the compiler stops spawning, lets the current write-rename pair finish or removes its `.tmp` file, and exits 130; on SIGTERM likewise with exit 143. No partial artifact exists after an interrupted build. The build lock file `<target>/.plkc-lock` carries a heartbeat timestamp updated every 5 seconds; a lock older than 30 seconds without heartbeat is broken automatically with warning W-LOCK-STALE. At startup, `.tmp` files from a previous run are deleted.

### 24.5.9 Disk exhaustion handling

Before writing outputs, the build system estimates required space as 3 times the total input source size plus 536870912 bytes (512 MiB); if free space is below the estimate it emits warning W-DISK-LOW and continues. A failed write is never retried silently: the build evicts cache entries (LRU per Volume 20), retries the failed write once, and on second failure emits E-DISK-FULL and exits 2. Atomic rename (§24.5.8) guarantees that a failed write leaves no partial artifact.

### 24.5.10 Out-of-memory behavior

Compiler-side behavior is §24.2.9: budget LIM-MEM, E-OOM, exit 2, no degraded continuation. Runtime-side behavior of compiled programs is defined in Volume 18 (§25.3 of design.md); this volume adds the toolchain-level rule: the compiler does not promise to compile programs whose compilation exceeds LIM-MEM, and programs whose object size would exceed the Volume 12 object-size limit are rejected with the corresponding LIM code before code generation. The two rules together give a defined failure for every exhaustion point in the toolchain.

## 24.6 Runtime security (design.md §32.5)

The runtime security surface is the compiled program's; the semantics it protects are Volume 18's and the UB model of SPEC-INDEX §3. All mechanisms here are opt-in or platform-conditional as stated, and every mechanism maps failure to the trap set of §24.6.1 or to a compile-time rejection.

### 24.6.1 Trap set

The runtime security traps are exactly: stack-overflow, stack-canary, cfi-violation, entropy-unavailable. Each is a defined termination: abort with exit code 134 per SPEC-INDEX §3, with the trap name printed to stderr as the single line `trap: <name>`. The JIT product adds its own traps (Volume 26 §26.2), disjoint from this set.

### 24.6.2 Stack protection

The runtime maps a stack-violation fault inside the guard region to the trap stack-overflow. Guard pages are the platform page size, probed at startup, never assumed (Law 7). Stack size limits: main thread default 8 MiB; spawned tasks default 2 MiB; per-function override by the attribute `#[stack_size(N)]` where N is a multiple of 65536 bytes and at most 1073741824 (1 GiB); larger or unaligned N is a compile-time rejection. A stack overflow during unwinding is an immediate abort: drop glue runs at most once per frame, and a second fault inside unwinding terminates without further unwinding.

### 24.6.3 Memory protection and hardened allocation options

The `hardened` profile flag enables, exactly: (a) freed memory is filled with 0xA5 and freshly allocated memory is filled with 0x5A before use; (b) quarantine: freed blocks of at most 1 MiB each are held unreusable in a FIFO quarantine with total capacity 67108864 bytes (64 MiB); (c) deterministic reuse (no randomized placement); (d) types marked `#[sensitive]` are filled with 0x00 on drop. Cost statement per Law 1: hardened allocation is CEP-1; it is restricted to the profiles debug and releng-hardened and is banned in the default release profile and in all CEP-0 code. Deallocated memory is never observable through safe code regardless of profile (Volume 05); the patterns above are defense in depth against unsafe code and FFI errors. Generated code pages are mapped read-execute after load; no page is writable and executable at the same time in an AOT build.

### 24.6.4 Control-flow protection

Forward edge: with `hardened = true`, every indirect call site emits a tag check. The tag of a callee is the low 32 bits of FNV-1a over its mangled name (FNV-1a 32-bit: basis 2166136261, prime 16777619, per byte: xor then multiply). The check compares the tag stored with the target against the expected constant; mismatch is the trap cfi-violation, exit 134. Backward edge: platform capability table, exact: x86-64 with CET present uses the shadow stack; AArch64 with PAC present uses pointer authentication on returns; on platforms without the mechanism, backward-edge protection is absent and this absence is stated in the target capability table of Volume 16 (it is never silently claimed). Return-address corruption without CET/PAC remains in the UB model as UB-2/UB-4 exposure through unsafe code only; safe code cannot perform it (G2).

### 24.6.5 Platform security features

Per build: position-independent executables are the default on every host target; full RELRO on ELF targets in the release and hardened profiles; the ELF stack is marked non-executable; stack canaries are emitted, in the release and hardened profiles, for every function containing a local array of at least 64 bytes or any address-taken local; the canary value is 8 bytes from the runtime CSPRNG per thread, checked on epilogue, mismatch is the trap stack-canary, exit 134. Target capability entries (used if present, declared-not-implemented otherwise, stated in Volume 16): x86-64 CET shadow stack used if present, CET Indirect Branch Tracking declared-not-implemented in edition P1; AArch64 PAC used if present, BTI emitted when supported, MTE declared-not-implemented.

### 24.6.6 Sanitizer support

Two sanitizer builds are supported, exactly: `--sanitize=address` (address sanitizer: detects out-of-bounds, use-after-free, double-free in the compiled program at runtime) and `--sanitize=thread` (thread sanitizer: detects data races against UB-3). Both link the platform sanitizer runtime and emit the required instrumentation; each has a conformance suite of 24 known-bug programs, each of which must be reported or rejected by the sanitizer build. The memory sanitizer is declared-not-implemented in edition P1 (it requires a fully instrumented support library; this is a named limitation, not a silent gap). Sanitizer builds are permitted with any profile; combining a sanitizer with the hardened allocator is a compile-time rejection E-SAN-CONFLICT, exit 1, because both replace the allocator.

### 24.6.7 Secure FFI boundaries

FFI rules of Volume 10, hardened here by four requirements. (1) Every pointer parameter of an extern declaration is annotated `#[ffi_owned]` or `#[ffi_borrowed]`; the annotation states who allocates and who frees; an unannotated pointer parameter is a compile-time rejection. (2) Unwinding across FFI is a compile-time rejection per SPEC-INDEX §3. (3) Textual parameters crossing FFI are pointer-plus-length pairs; NUL-terminated assumptions in FFI signatures are rejected at type level (the byte-string type carries its length). (4) Re-entering the Planck runtime from a foreign thread requires the attach call of Volume 18 (`plk_runtime_attach`); an unattached callback invocation is the defined trap `runtime-not-attached`, which joins the trap set of §24.6.1 as its fifth member. FFI calls carry the ffi effect atom (Volume 07); the runtime never passes values through environment variables to foreign code.

### 24.6.8 Secure cryptographic facilities

The cryptography library is defined in Volume 19 (§26.11); this volume pins its security contract. CSPRNG: the OS entropy source per target, exactly: getrandom on Linux, getentropy on macOS, BCryptGenRandom on Windows; entropy-source failure is the trap entropy-unavailable, exit 134; a userland fallback or weak default is banned. Secret comparison: `secret_eq` performs a fixed-time comparison over padded length max(len1, len2) with an accumulate-then-test reduction, no branch on secret bytes; the obligation is that its memory access pattern is independent of secret content (OB-24-19). Zeroization: `#[sensitive]` types zeroize on drop with 0x00 and are banned from Debug and formatting derives (compile-time rejection). No cryptographic primitive is implemented inside the compiler. Secrets must not appear in logs, diagnostics, or ICE reports; the ICE field rules of §24.5.2 enforce this on the report side.

## Verification matrix

| ID | Source | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- | --- |
| OB-24-01 | §32.1 | Table 24.1 defines an entry for each of the 11 limit classes FILE, INPUT, TOKENS, AST-FILE, AST-CRATE, NEST, SEM, TRAIT, CONST, MACRO, IR, DIAG, PROC, MEM, and each value is in the stated range | EXH over the 28 table rows | FV-0 | tools/fv/ob-24-01.smt2 | G13, G15 |
| OB-24-02 | §32.1.1 | For every byte value b in 0..256 the lexer judgment maps b to exactly one of {token, whitespace, comment, error} | EXH over 256 values | FV-0 | tools/fv/ob-24-02.smt2 | G12 |
| OB-24-03 | §32.1.2-3 | Parser and lowering terminate on every input: token budget 2^24 and depth budget 256/1024 strictly decrease the remaining budget per step | Z3, bound 10^6 steps unrolled | FV-1 (gap: full unrolling; bridge: runtime counter checks) | tools/fv/ob-24-03.smt2 | G11, G13 |
| OB-24-04 | §32.1.11 | The exit-code function is total over the failure taxonomy {0, 1, 2, 3, 4, 101, 130, 143} and every failure enum variant maps to exactly one code | EXH over taxonomy variants | FV-0 | tools/fv/ob-24-04.smt2 | G13, G22 |
| OB-24-05 | §32.4.2 | Every ICE report field value is drawn from the field allowlist of §24.5.2; no field path can contain source text or non-allowlist environment values | REVIEW of the report builder against the field list | FV-2 | review notes in volume-24 review record | G16, G22 |
| OB-24-06 | §32.2.10 | The cache loader never returns accept for a mutated entry: for all single-byte mutations of a 128-byte header the verdict is delete-and-recompute | EXH over 128 x 256 = 32768 mutations | FV-0 | math/../tests or tools/fv/ob-24-06 harness | G12, G23 |
| OB-24-07 | §32.2.1 | The sandbox capability set is exactly {read package source, read dep sources, write OUT_DIR, allowlisted env} for build scripts and {read package source, allowlisted env} for proc macros | REVIEW of plkc-run capability tables per platform | FV-2 | plkc-run source review record | G16 |
| OB-24-08 | §32.2.4 | valid_path(s) accepts exactly the relative, traversal-free, NUL-free paths and rejects all others | EXH over all strings of length at most 6 over the alphabet {a, ".", "/", "\0", "-", " "} (93408 cases) | FV-0 | tools/fv/ob-24-08.smt2 | G12 |
| OB-24-09 | §32.3.1 | Signature verification accepts exactly the Ed25519-valid (manifest, signature, key) triples and rejects all others | PROP over generated pairs, paired with REVIEW of the Ed25519 library trust assumption | FV-1 (gap: external crypto library; bridge: pinned library version) | tools/fv/ob-24-09 harness | G16 |
| OB-24-10 | §32.3.3 | For every published artifact the provenance chain (commit, tarball hash, inputs, artifacts) is closed under hash equality | EXH over the release corpus of Volume 25 §25.1.6 (7 artifact classes per release) | FV-0 | release CI job provenance-check | G14 |
| OB-24-11 | §32.3.6 | Two builds from the pinned inputs produce byte-identical artifact hash sets | EXH over the release corpus: 2 builds x full artifact set, hash comparison | FV-0 | CI job repro-check | G14 |
| OB-24-12 | §32.3.7 | For every conformance corpus program P: hash(stage1_output(P)) = hash(stage2_output(P)) | EXH over the full conformance corpus of Volume 22 §29.1 | FV-0 | CI job bootstrap-equality | G25 |
| OB-24-13 | §32.4.8 | The advisory match predicate is total and exact: v matches set S iff some comparator (op, w) in S satisfies compare(v, w); domain v in {0..2}^3, w in {0..2}^3, op in 5 comparators (3645 cases) | EXH over 3645 cases | FV-0 | math/../tests (audit crate) | G26 |
| OB-24-14 | §32.4.5 | For every case of the diagnostic corpus, the rendered diagnostic is byte-identical across two runs and matches the pinned code and span | EXH over the diagnostic corpus of Volume 21 §28.7 | FV-0 | CI job diag-stability | G22, G14 |
| OB-24-15 | §32.4.3 | ice-repro terminates: its test-execution counter is bounded by 10^4 and each test by 10 seconds | REVIEW of the counter structure (each loop decrements one bounded counter) | FV-2 | review record of the ice-repro tool | G11 |
| OB-24-16 | §32.4.7 | Cached and recomputed outputs are byte-identical: for every cacheable unit, reuse verdict accept implies hash equality with recompute | PROP over the incremental corpus, paired with OB-24-06 EXH | FV-1 (gap: corpus sampling; bridge: hash check on every reuse) | CI job incremental-equality | G23 |
| OB-24-17 | §32.4.8-9 | Atomicity of artifact writes: after any interleaving of write, rename, kill, the target path holds either the old or the new complete artifact, never a partial one | PROP with deterministic fault schedule over write sequences of length at most 8 | FV-1 (gap: schedule sampling; bridge: write-then-rename pattern enforced by review) | tools/fv/ob-24-17 harness | G13 |
| OB-24-18 | §32.5.3 | Hardened allocator invariants: every freed block is 0xA5-filled before reuse or enters quarantine; every allocation is 0x5A-filled; quarantine capacity is exactly 64 MiB FIFO | EXH over allocation sequences of length at most 8 with sizes in {64, 256, 4096} (6561 cases) | FV-0 | math/../tests (hardened allocator crate) | G2, G20 |
| OB-24-19 | §32.5.4, §32.5.8 | CFI tags are pairwise distinct over the FFI corpus, and secret_eq's access pattern is independent of secret content | EXH over the FFI surface corpus of Volume 10 for tags; REVIEW for secret_eq (branch-free construction argued line by line) | FV-1 (gap: secret_eq timing beyond access pattern; bridge: branch-free construction and constant padding) | tools/fv/ob-24-19 harness; review record | G16, G20 |
| OB-24-20 | §32.5.1 | Every runtime security trap is in the set {stack-overflow, stack-canary, cfi-violation, entropy-unavailable, runtime-not-attached}, exits 134, and prints exactly one `trap: <name>` line | EXH over the 5-member set | FV-0 | tools/fv/ob-24-20.smt2 | G20 |
| OB-24-21 | §32.5.8 | Each target maps entropy failure to the trap entropy-unavailable; no target defines a userland fallback | REVIEW of the three per-target entropy bindings (OS behavior is not machine-checkable) | FV-1 (gap: OS entropy semantics; bridge: defined trap on failure) | review record per target | G20 |
| OB-24-22 | §32.1.0 | The untrusted-input contract is closed: every input handler's failure set is a subset of the taxonomy of §24.2.11 | REVIEW over the phase Result types (one phase table in Volume 12) | FV-2 | Volume 12 phase table review | G12, G13, G22 |

## Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| --- | --- |
| §32.1 Malformed source handling | §24.2.1 |
| §32.1 Parser resource limits | §24.2.2 |
| §32.1 Recursion limits | §24.2.3 |
| §32.1 Generic instantiation limits | §24.2.4 |
| §32.1 Trait solver limits | §24.2.5 |
| §32.1 Constant evaluation limits | §24.2.6 |
| §32.1 Macro expansion limits | §24.2.7 |
| §32.1 Graph size limits | §24.2.8 |
| §32.1 Memory exhaustion behavior | §24.2.9, §24.5.10 |
| §32.1 Integer overflow in compiler internals | §24.2.10 |
| §32.1 Compiler crash handling | §24.2.11, §24.5.2 |
| §32.2 Build script sandboxing | §24.3.1 |
| §32.2 Procedural macro sandboxing | §24.3.2 |
| §32.2 Compile-time execution restrictions | §24.3.3 |
| §32.2 Filesystem access controls | §24.3.4 |
| §32.2 Network access controls | §24.3.5 |
| §32.2 Process execution controls | §24.3.6 |
| §32.2 Environment isolation | §24.3.7 |
| §32.2 Resource limits | §24.3.8, Table 24.1 |
| §32.2 Dependency trust | §24.3.9 |
| §32.2 Build cache trust | §24.3.10 |
| §32.3 Signed compiler releases | §24.4.1 |
| §32.3 Signed package manifests | §24.4.2 |
| §32.3 Dependency provenance | §24.4.3 |
| §32.3 Build artifact integrity | §24.4.4 |
| §32.3 Compiler checksum validation | §24.4.5 |
| §32.3 Reproducible releases | §24.4.6 |
| §32.3 Bootstrap integrity | §24.4.7 |
| §32.3 Dependency vulnerability handling | §24.4.8 |
| §32.4 Graceful error handling | §24.5.1 |
| §32.4 Internal compiler error reports | §24.5.2 |
| §32.4 Crash reproduction | §24.5.3 |
| §32.4 Compiler state dumps | §24.5.4 |
| §32.4 Diagnostic quality | §24.5.5 |
| §32.4 Recovery from malformed source | §24.5.6 |
| §32.4 Corrupt cache recovery | §24.5.7, §24.3.10 |
| §32.4 Interrupted build recovery | §24.5.8 |
| §32.4 Disk exhaustion handling | §24.5.9 |
| §32.4 Out-of-memory behavior | §24.5.10, §24.2.9 |
| §32.5 Stack protection | §24.6.2 |
| §32.5 Memory protection | §24.6.3 |
| §32.5 Hardened allocation options | §24.6.3 |
| §32.5 Control-flow protection | §24.6.4 |
| §32.5 Platform security features | §24.6.5 |
| §32.5 Sanitizer support | §24.6.6 |
| §32.5 Secure FFI boundaries | §24.6.7 |
| §32.5 Secure cryptographic facilities | §24.6.8 |

Bullets in scope: 47, defined: 47.
