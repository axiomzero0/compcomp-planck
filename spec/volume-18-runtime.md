<!--
CEP:FILE: spec/volume-18-runtime.md
CEP:WHY: design.md §25 (lines 2860–2953) states 60 items on runtime modes, initialization, memory, panic/unwinding, concurrency, instrumentation, and the runtime ABI that each need an explicit definition; this volume settles the open policy questions (DWARF EH frames for panic unwinding; POSIX subset = mmap/brk/pthread/futex plus the closed extension list) and defines every item normatively.
CEP:WHAT: Normative specification of Planck Volume 18 — runtime and platform support: runtime modes, startup contract with exact per-target entry/stack/argc-argv rules, allocator contract with max_align_t = 16, panic and unwinding via DWARF EH frames, thread management and FFI boundaries, the closed POSIX OS-interface subset, freestanding no_std mode, instrumentation, and the Runtime ABI with its closed entry-point list.
CEP:STATUS: complete
CEP:FAILURE: A design.md §25 bullet without a coverage-ledger row here is an undefined runtime behavior; an implementation contradicting a rule here is non-conforming and must be corrected.
CEP:ASSUMES: SPEC-INDEX.md §3 (UB model: traps are abort with exit code 134; destruction/unwinding model; async model); Volume 16 §16.6–§16.7 (frame, prologue, unwind metadata emission); Volume 17 (program headers, TLS init, exit codes); Volume 19 (std streams, thread-safe types); verification standards.md G11, G13, G15, G21, G22.
CEP:COST: document; the pinned contracts bound runtime work (allocator O(1) class lookup, O(log classes) free; unwind lookup O(log FDEs) via binary search); no cost of its own.
CEP:EVIDENCE: verification matrix ("Verification matrix", OB-18-1 … OB-18-13) and coverage ledger ("Coverage ledger") at the end of this file.
-->

# Volume 18 — Runtime and platform support

## 18.0 Scope and traceability

This volume defines design.md §25 "Runtime architecture", lines 2860–2953, subsections 25.1–25.7.

Bullets in scope: 60, distributed as follows.

| design.md subsection | Bullets | Defined in |
| -------------------- | ------- | ---------- |
| 25.1 Runtime modes | 5 | §18.1 (M1–M5) |
| 25.2 Runtime initialization | 9 | §18.2 (I1–I9) |
| 25.3 Memory runtime | 10 | §18.3 (MEM1–MEM10) |
| 25.4 Panic and unwinding runtime | 11 | §18.4 (P1–P11) |
| 25.5 Concurrency runtime | 9 | §18.5 (C1–C9) |
| 25.6 Runtime instrumentation | 8 | §18.6 (INS1–INS8) |
| 25.7 Runtime ABI | 8 | §18.7 (ABI1–ABI8) |

Upstream inputs: emitted unwind metadata and frames from Volume 16; executable layout from Volume 17. Downstream consumers: Volume 19 (stdlib built on this runtime) and Volume 10 (FFI). The non-bullet sentence "Specify which language features require which runtime components" is defined normatively in §18.1 (the feature/component matrix).

## 18.1 Runtime modes (defines 25.1)

There are exactly five runtime modes. The mode is a whole-program link-time property; mixing modes across one link is a link-time rejection E-RT-01.

M1 No-runtime mode. Nothing from `libplanck_rt.a` is linked: no allocator, no panic unwinding, no threads, no IO. `panic` is banned in this mode at compile time unless `panic = abort` with a user-supplied `#[panic_handler]`; UB-6-class intrinsics requiring runtime fallbacks are rejected. Language features permitted: core primitives (Volume 19 §19.1), atomics (target-provided), statics, FFI.

M2 Freestanding mode. The runtime ships the `core` subset only (the no_std subset list): modules `core`, `num`, and the fixed-size parts of `collections` that do not allocate; no allocator by default (`GlobalAllocator` may be supplied by the program); `panic = abort` only; no IO, threads, filesystem, sockets, or time APIs (their import is a compile-time rejection E-RT-02 naming the mode). OS-interface calls are banned in this mode (§18.8 list is unavailable).

M3 Minimal hosted mode. Links: allocator (MEM), panic/unwinding (P), std streams (Volume 19 I1–I3), time. Excluded: threads, sockets, filesystem, processes. Features requiring excluded components are compile-time rejections E-RT-03.

M4 Full hosted mode. All runtime components: allocator, panic/unwinding, threads, TLS, std streams, filesystem, processes, sockets, time, instrumentation hooks. This is the default mode for hosted targets.

M5 Custom runtime mode. The program supplies its own runtime crate implementing the Runtime ABI (§18.7 ABI1 entry-point list and ABI2–ABI7 contracts); the linker links it in place of `libplanck_rt.a`. A custom runtime missing any required entry point is a link error E-LNK-07 naming the missing symbol.

Feature/component matrix (each cell exactly one of: allowed / requires-component / rejected). The matrix is closed; adding a feature or component is an edition change.

| Feature | No-runtime | Freestanding | Minimal hosted | Full hosted |
| ------- | ---------- | ------------ | -------------- | ----------- |
| Core primitives, atomics | allowed | allowed | allowed | allowed |
| Heap allocation | rejected | requires GlobalAllocator | allowed (MEM) | allowed (MEM) |
| panic (unwind) | rejected | rejected (abort only) | allowed (P) | allowed (P) |
| panic (abort) | requires handler | allowed | allowed | allowed |
| std streams | rejected | rejected | allowed | allowed |
| Threads, TLS, sync | rejected | rejected | rejected | allowed (C) |
| Filesystem, processes | rejected | rejected | rejected | allowed |
| Sockets, DNS | rejected | rejected | rejected | allowed (net archive) |
| Time | rejected | rejected | allowed | allowed |

## 18.2 Runtime initialization (defines 25.2)

I1 Program entry. The executable entry point is the symbol `_start` (Volume 17 EX1), provided by the runtime startup object. Per-target exact contracts at the moment `_start` begins executing: X86_64: `RSP` → argc, stack `[argc][argv… NULL][envp… NULL][auxv]`, `RSP ≡ 0 (mod 16)`. AArch64: `SP` → argc with the same layout, `SP ≡ 0 (mod 16)`, X29/X30 contents undefined. Stack setup performed by `_start`: X86_64: `xor ebp, ebp; mov rdi, [rsp]; lea rsi, [rsp+8]; scan argv for its NULL terminator to compute envp into rdx; and rsp, -16; call planck_start`. AArch64: `mov x29, #0; ldr x0, [sp]; add x1, sp, #8; scan argv for its NULL terminator to compute envp into x2; bic sp, sp, #15; b planck_start`. The argv scan is bounded by MAX_ARGC = 1048576; exceeding it aborts with exit code 126.

I2 Static initialization. Language statics with non-constant initializers are run from `.init_array` in the order of Volume 17 EX4 (ascending symbol ID). The runtime provides no ordering beyond this rule; duplicate initialization is impossible because each static has exactly one entry.

I3 TLS initialization. The kernel/libc copies the `.tdata` image and zeroes the `.tbss` portion for the main thread before `_start`; the runtime performs no additional main-thread TLS setup. The TP-relative offset formula is in ABI7. Dynamic TLS for spawned threads is ABI7.

I4 Argument handling. `planck_start(argc: i32, argv: *const *const u8, envp: *const *const u8) -> !` receives argc and the NULL-terminated argv/envp arrays exactly as laid out in I1; it passes them to `main(argc: i32, argv: *const *const u8) -> i32`. The language-level `main` signatures accepted by the compiler: `fn main()`, `fn main() -> i32`, `fn main(argc: i32, argv: *const *const u8) -> i32`; other signatures are compile-time rejections E-RT-04.

I5 Environment access. The environment block is snapshotted into an owned structure during `planck_rt_init` (before any user code): a sorted (ascending byte order) array of `key=value` strings; later mutation via `std::os::setenv` mutates both the snapshot and the libc environment (§18.8). Reads after startup are O(log n) binary search over the sorted snapshot.

I6 Standard streams. The runtime does not open, close, or redirect fds 0/1/2; it wraps them as the std stream handles of Volume 19 I1–I3. Write buffering and flush-at-exit are Volume 19 contracts; the runtime guarantees only that the descriptors exist and that `planck_exit` flushes before `exit_group`.

I7 Runtime startup. Exact sequence inside `planck_start`: (1) initialize the allocator (MEM4); (2) load `planck_canary` from the first 8 bytes of the `AT_RANDOM` auxv entry (Volume 16 FR11); (3) snapshot the environment (I5); (4) run `.init_array` (I2); (5) call `main`; (6) on return, `planck_exit(main_return)` (I9). Any failure in steps 1–4 terminates with exit code 126 before `main` runs.

I8 Runtime shutdown. `planck_exit(code)`: run exit handlers (registered by `std::os::at_exit`) in reverse registration order; run `.fini_array` in reverse link order; flush std streams (I6); free nothing (the allocator is not shut down; the process image is discarded); call `exit_group(2)` with the final exit code. `planck_exit` never returns.

I9 Exit processing. Exit codes follow Volume 17 EX8 exactly (0 success; `main_return & 0xFF`; 134 trap/abort; 126 init failure). Exit handlers registered during exit processing run in the same reverse order; a panic inside an exit handler is a double-panic case of P9 (abort 134, unwinding does not continue).

## 18.3 Memory runtime (defines 25.3)

MEM1 Default allocator. The default allocator is `planck-alloc`, bundled in `libplanck_rt.a`. Exact design: 38 size classes: sizes 16, 32, 48, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 640, 768, 896, 1024, 1280, 1536, 1792, 2048, 2560, 3072, 3584, 4096 (28 classes), then 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304 (10 classes); requests > 4194304 bytes are served by direct `mmap(2)` rounded up to the 4096-byte page size. Class lookup is a table index by the size bucket (O(1)); free lists are LIFO per class; slab growth acquires 1 MiB (1048576-byte) slabs via `mmap(2)` and carves them into max-size-class blocks. There are no per-thread caches in P1 (stated); the shared structure is protected by a `pthread_mutex`. Complexity: alloc O(1) amortized (slab carve is O(1) per allocation within a slab); free O(1).

MEM2 Allocation interfaces. The global allocator interface (unsafe because callers must uphold the Layout contract):

```rust
pub struct Layout { pub size: usize, pub align: usize }  // align is a power of two, 1 ≤ align
pub unsafe trait GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;            // null on failure
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, new_size: usize) -> *mut u8;
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8;     // null on failure
}
```

`realloc` semantics: grow in place when the class permits, else allocate-copy-free; on failure returns null and leaves the old block valid. Violating the Layout contract (wrong size/align on dealloc, or use after free) is UB-2 per SPEC-INDEX §3.

MEM3 Deallocation interfaces. `dealloc(ptr, layout)` requires exactly the layout used at allocation; the allocator validates that ptr is non-null and 16-aligned as a cheap contract check and traps (abort 134) on a null or misaligned pointer (defined trap, not UB); deeper contract violations are UB-2.

MEM4 Allocator initialization. The allocator initializes lazily at first use inside `planck_rt_init` (I7 step 1); initialization is idempotent and thread-safe (performed before any user thread can exist, since threads are a Full-hosted feature started only after init). There is no allocator shutdown (I8).

MEM5 Allocation failure behavior. Allocation failure returns null (or `AllocError` at the stdlib layer); it never aborts, never panics, and never retries unboundedly. Stdlib functions convert null into `AllocError` per Volume 19. The out-of-memory path of a `Vec` growth is therefore a `Result`, not a crash.

MEM6 Memory alignment. Guarantees: every allocation is aligned to `max(align, 16)`; `max_align_t = 16` is the P1 default guarantee for C-ABI interop (the largest alignment the C ABI guarantees); allocations with `align > 16` take the aligned path (direct `mmap` with page-aligned base) and remain supported. `align_up(x, a) = (x + a − 1) & !(a − 1)` in checked arithmetic (overflow → AllocError).

MEM7 Custom allocators. A program replaces the global allocator by defining `planck_allocator_override` (a WEAK-symbol override, Volume 17 F10) implementing the `GlobalAllocator` ABI of ABI4; the runtime calls it through that symbol. Supplying an override with a wrong ABI is a link-time signature check failure (E-LNK-08).

MEM8 Arena interfaces. The runtime provides the bump-arena primitive used by Volume 19 M9: fixed 65536-byte chunks acquired via `mmap(2)`, bump pointer allocation, O(1) `reset` per chunk; chunk lists are freed in O(#chunks). The arena is not thread-safe (documented); concurrent arenas per thread are the supported pattern.

MEM9 Stack allocation. Stack allocation (`alloca`) is a compiler contract (Volume 16 FR9); the runtime supplies the stack-probe scratch conventions and the ALLOCA_MAX = 2^31 bound check (defined trap abort 134 on violation). The runtime itself performs no dynamic stack allocation.

MEM10 Runtime allocation diagnostics. Under the `alloc-trace` feature, the allocator maintains exact counters: `alloc_count: u64`, `dealloc_count: u64`, `bytes_live: i64`, `bytes_peak: u64`, read through `std::os::alloc_stats()`. Counters are updated on every operation (no sampling); the feature adds one atomic add per operation and is off by default.

## 18.4 Panic and unwinding runtime (defines 25.4)

Settled decision: panic unwinding uses DWARF EH frames (`.eh_frame`/`.eh_frame_hdr` + LSDA) emitted per Volume 16 FR12, with a two-level (search, then cleanup) personality implemented in the runtime.

P1 Panic entry. `planck_panic(payload: &PanicPayload, location: &SourceLocation) -> !`; `SourceLocation = { file: &str, line: u32, column: u32 }`. Panics start unwinding; with `panic = abort` the same entry calls the abort path (P8) instead of unwinding.

P2 Panic payload. Exact type: `enum PanicPayload { Static(&'static str), Formatted }` where `Formatted` reads from a fixed 4096-byte panic message buffer in TLS, filled by `planck_panic_fmt(fmt_args)` before `planck_panic` is called. The buffer is allocated once per thread, never reallocated during unwind (its address stability is a P invariant), and is cleared after the catch boundary runs.

P3 Panic reporting. The default handler writes exactly `"panicked at <file>:<line>:<col>: <message>\n"` to fd 2 (unbuffered write(2), one call when ≤ 4096 bytes) and then unwinds. A `planck_panic_hook_override` WEAK symbol (Volume 17 F10) may replace the reporter; the hook receives `(&PanicPayload, &SourceLocation)` and must not unwind.

P4 Unwinding mechanism. Two-level DWARF unwind: Phase 1 walks frames via `.eh_frame_hdr` binary search calling the personality in search mode to locate a landing pad; Phase 2 walks again in cleanup mode executing landing pads (destructors) at each frame. The personality is `planck_eh_personality` (ABI1); `planck_unwind_resume` resumes Phase 2 after a landing pad returns.

P5 Exception metadata. Per-frame LSDA (language-specific data area) referenced from the FDE augmentation; entries are (landing-pad offset u32, filter value u32) pairs sorted ascending; Planck uses cleanup entries only (no type table) in P1: filter value 0. CIE contract: version 3, augmentation `"zR"`, code alignment factor 1, data alignment factor −8, return-address register 16 (X86_64) / 30 (AArch64), FDE/LSDA encoding 0x1B (sdata4 pcrel).

P6 Stack unwinding. Frame walk algorithm: read the return address from the CFA per the FDE CFI; look up the next FDE by binary search over the sorted `.eh_frame_hdr` table (O(log FDEs), exact algorithm: standard binary search over the u32-sorted initial-location array); terminate at a frame with no FDE. Unwinding is bounded: the walk stops at the catch boundary or at the outermost frame; an outermost-frame miss (no catch boundary) terminates the process (abort 134) with the P3 report first.

P7 Destructor execution. Landing pads run destructors innermost-first per the SPEC-INDEX destruction model; a landing pad never observes the panic payload through its own frame's drops (payload access happens only at the catch boundary). Drop code runs with unwinding in progress; a panic inside a drop is the double-panic case P9.

P8 Abort behavior. The abort path: write the P3 report (if reporting is reachable), then `raise(SIGABRT)`; if the signal is unhandled, `exit_group(2)` with code 134. Machine-level trap sequences used by the compiler for unreachable code: X86_64 `ud2`, AArch64 `brk #0x1`; reaching one triggers the same SIGILL/SIGTRAP → abort 134 path via the runtime's signal handling (INS1). Every trap is a defined termination with exit 134, never UB (SPEC-INDEX §3).

P9 Double panic. A panic while unwinding (panic inside a drop, a landing pad, or an exit handler) writes `"panic during panic; aborting"` to fd 2 and immediately aborts (P8, exit 134). No further unwinding occurs; the second payload is not reported. The state space is finite (unwinding → panic → abort) and terminates by construction.

P10 Foreign unwind boundaries. Unwinding across FFI boundaries is a compile-time rejection (SPEC-INDEX §3): a function with the `ffi` effect atom may not contain a panic that crosses its boundary. At runtime, if Phase 1 reaches a frame without Planck LSDA data (a foreign frame), the runtime aborts with exit 134 and the message `"unwind across foreign frame"`. Foreign code catching via its own mechanism is outside the Planck contract.

P11 Runtime configuration. Panic behavior is configured at compile time only: `panic = unwind` (default) or `panic = abort` (build flag). There are no runtime configuration knobs for panicking in P1 (no environment variables, no config files); the hook override of P3 is the only runtime-extensible point.

## 18.5 Concurrency runtime (defines 25.5)

C1 Thread support. 1:1 threads over `pthread_create(2)`. `sync::Thread::spawn` (Volume 19 X1) allocates a `ThreadPacket { entry: fn(*mut u8), arg: *mut u8, result_slot: u64-slot, join_state: u32 }` and starts the trampoline `planck_thread_start(packet) -> !`. Default thread stack = 8388608 bytes (8 MiB); requested sizes below MIN_THREAD_STACK = 16384 bytes are a defined rejection at spawn time (spawn returns `JoinError::StackTooSmall`; not a trap). Thread handles are joinable only (no detach in P1, per the SPEC-INDEX no-detached-tasks model).

C2 Thread-local storage. `#[thread_local]` statics use the TLS ABI (ABI7): the main thread's image is kernel-provided (I3); spawned threads receive a dynamically allocated TLS block (image copy + zeroed `.tbss` portion) allocated from the default allocator before the thread entry runs. TLS variables are zero/null-initialized (no dynamic initializers in P1; a dynamic initializer requirement is a compile-time rejection E-RT-05).

C3 Synchronization primitives. The runtime provides the futex-based primitives of Volume 19 X2–X5 with these exact word layouts: Mutex state u32: 0 = unlocked, 1 = locked-no-waiters, 2 = locked-with-waiters (3-state futex protocol; acquire = CAS 0→1, contended path CAS/`futex(FUTEX_WAIT)` loop, release = 2→0 or 1→0 with `futex(FUTEX_WAKE, 1)`). RwLock state u32: bits 0–30 reader count, bit 31 writer flag; writer-preferring (new readers blocked while a writer waits). Condvar: u32 sequence counter; wait = unlock + `futex(FUTEX_WAIT, seq)` + relock; notify = increment + `FUTEX_WAKE` (1 or ∞). Spurious wakeups are permitted by contract; callers must use predicate loops (Volume 19 X4).

C4 Atomic support. Compiler-level mapping is Volume 16 SEL8/TO7; the runtime supplies no atomic slow paths for sizes in T13 (all supported sizes are native). Atomic orderings are the Volume 07 memory model values {Relaxed, Acquire, Release, AcqRel, SeqCst}; the runtime must not reorder atomic operations across its own lock operations in ways visible to the memory model (instrumentation code is exception-documented).

C5 Scheduler support. P1 has no user-space scheduler: threads are OS-scheduled. The runtime's only scheduling action is the blocking pool of C8 (for async IO), which is a work distribution mechanism, not a scheduler of user continuations.

C6 Async executor interfaces. The default executor is `sync::block_on` (single-threaded, blocking, Volume 19 X10) plus structured scopes (`scope.spawn(future)`); tasks form a tree with join at scope end per the SPEC-INDEX async model. The executor calls `poll` on `Pin<&mut Future>` with a `Context` carrying the `Waker` of X9. Cancellation is cooperative via cancel checks at await points; dropping a scope sets the cancel flags of its tasks before their join points complete.

C7 Task allocation. A task = a heap-allocated future plus its waker state, allocated from the default allocator (MEM) at spawn; the allocation is freed when the task completes or its scope is dropped. Task state per task: `cancel_flag: AtomicBool`, `waker_data: *mut u8`, `poll_fn: fn(Pin<&mut dyn-free Future>, &mut Context) -> Poll` — represented concretely as a monomorphized fn pointer pair (no trait objects; the function-pointer table exemption of standards.md §25.6 applies and is measured under Volume 23).

C8 Wake mechanisms. Waking = `futex(FUTEX_WAKE, 1)` on the woken task's 4-byte wait word after storing a flag; `Waker::wake` is idempotent (a second wake on a non-waiting task sets the flag observed by the next poll). Blocking-pool IO (Volume 19 I11/W7) uses at most 16 OS threads (exact bound BLOCKING_POOL_MAX = 16); wake from the pool uses the same mechanism.

C9 Cancellation. Exactly the structured model: cancelling a parent scope sets the cancel flags of all child tasks, then joins them; every task must reach its scope exit after cancellation (await points poll the cancel flag and return a defined `Cancelled` result); no detached tasks, no forced preemption, no cancellation-safe-drop gaps: the drop of a cancelled task's future happens after its last poll returns.

## 18.6 Runtime instrumentation (defines 25.6)

INS1 Stack overflow detection. Hosted targets with guard pages: the runtime maps an 8-byte guard region below each thread stack (including the main stack when the OS provides one) and installs a SIGSEGV handler via `sigaction(2)`; on a fault inside a registered stack range it writes `"stack overflow\n"` to fd 2 and aborts with exit code 134. On targets without guard-page support, stack overflow is not detected (documented limitation; the Volume 12 stack-size limits still apply statically).

INS2 Profiling hooks. `#[profile]` functions call `planck_profile_enter(id: u32)` / `planck_profile_exit(id: u32)`; timestamps use the monotonic clock (O8) when enabled, and the calls compile to nothing when the feature is off (zero cost, verified by the emission checker). The runtime aggregates per-id totals into a fixed 4096-entry table; overflow of the table is a compile-time limit rejection (Volume 12).

INS3 Allocation tracking. Exact counters of MEM10; no sampling, no tracing beyond the counters in P1 (allocation call-stack capture is declared, not implemented).

INS4 Sanitizer hooks. P1 sanitizers are compiler-inserted checks (bounds, alignment per Volume 16 emission) that call `planck_rt_violation(kind: u32, location: &SourceLocation) -> !`, which reports and aborts 134. There is no ASan/TSan linkage in P1; external sanitizer integration is declared.

INS5 Coverage instrumentation. Under the `coverage` feature, each basic block increments a u64 counter stored in a `.planck.cov` section (one counter per block, ascending block ID); the runtime exposes `std::os::coverage_map()` reading the counters; costs are one increment per block execution, off by default.

INS6 Runtime assertions. `debug_assert!` compiles to a branch plus a call to `planck_rt_violation` in debug builds and to nothing in release builds; assertions never allocate and never format unless they fire.

INS7 Backtrace support. `std::os::backtrace()` walks the current stack using the P6 FDE walk (bounded by the same termination rules), printing `"<symbol>+<offset>"` lines resolved from `.symtab` when present, or raw addresses otherwise; the walk allocates nothing (fixed 256-frame limit; deeper stacks truncate with a `"…"` line).

INS8 Debugger integration. DWARF 4 emission (Volume 16 EM10) is the debugger contract; the runtime adds no debug protocol in P1. Prologue CFI correctness (Volume 16 FR12) is the invariant debuggers rely on for stack walking.

## 18.7 Runtime ABI (defines 25.7)

ABI1 Runtime entry points. The closed list of runtime ABI symbols (all follow the C calling convention of Volume 10 unless marked Planck-fast): `planck_start`, `planck_exit`, `planck_panic`, `planck_panic_fmt`, `planck_eh_personality`, `planck_unwind_resume`, `planck_alloc`, `planck_dealloc`, `planck_realloc`, `planck_alloc_zeroed`, `planck_thread_start`, `planck_stack_chk_fail`, `planck_stack_overflow_report`, `planck_rt_violation`, `planck_backtrace`, `planck_profile_enter`, `planck_profile_exit`. Generated code may reference only these symbols (plus user `#[ffi]` symbols and stdlib symbols); any other unresolved runtime reference is a link error. M1/M2 modes require a subset: exactly {`planck_panic` or a user `#[panic_handler]`, `planck_rt_violation`}; the subset check is part of the E-RT mode validation.

ABI2 Runtime data structures. The heap chunk header is 16 bytes, exact layout: `magic: u64 = 0x504C4E4B48454150` ("PLNKHEAP"), `size_class: u32`, `flags: u32` (bit 0 = mmap-backed). The header precedes the user pointer (user ptr = header ptr + 16); the magic is checked on `dealloc` (mismatch = defined trap abort 134, catching double-free of foreign pointers cheaply). `ThreadPacket` and `PanicPayload` layouts are as defined in C1/P2.

ABI3 Unwind ABI. Two-phase protocol of P4–P6 with these exact structures: saved-register frame = the callee-saved GPRs of the target as a u64 array (X86_64: 6 entries {RBX, RBP, R12, R13, R14, R15} + RA; AArch64: 12 entries {X19–X28, FP, LR}); `UnwindContext` = { CFA: u64, regs: [u64; N], lsda: u32, personality_result: u32 }. The personality returns one of {no-action, install-context, handler-found}; the CIE/LSDA encodings are P5.

ABI4 Allocator ABI. C-ABI wrappers: `planck_alloc(size: u64, align: u64) -> *mut u8` (null on failure), `planck_dealloc(ptr: *mut u8, size: u64, align: u64)`, `planck_realloc(ptr: *mut u8, old_size: u64, old_align: u64, new_size: u64) -> *mut u8`, `planck_alloc_zeroed(size: u64, align: u64) -> *mut u8`. These forward to the `GlobalAllocator` in effect (MEM1 or MEM7 override). The wrapper layer is C-ABI so custom allocators and FFI allocators interoperate.

ABI5 Panic ABI. `PanicPayload` layout (ABI2/P2): 8-byte tag (0 = Static, 1 = Formatted) + 8-byte `&'static str` pair (ptr, len) for Static; `SourceLocation` = { ptr: *const u8, file_len: u32, line: u32, column: u32 } (16 bytes). `planck_panic` never returns (C ABIs see it as `-> !`).

ABI6 Thread ABI. `planck_thread_start(packet: *mut ThreadPacket) -> !` is the pthread entry; it installs the thread TLS block (C2), runs the entry, stores the result in the packet's result slot, sets the join state with release ordering, and calls `planck_exit(0)` for the thread. `ThreadPacket` layout is C1. Join reads the result slot with acquire ordering.

ABI7 TLS ABI. TP-relative offset formula (exact): for a TLS symbol x with image offset `o(x)` in the TLS block of total image size `T` (tdata bytes + tbss bytes): `tpoff(x) = o(x) − T`. The TLS block is placed after the thread pointer (variant 2 layout), so all `tpoff` values are ≤ 0 and the variable resides at `tp + tpoff(x)`. Main thread: image provided by kernel/libc (I3); spawned threads: block allocated by the runtime with `T` computed from the executable's `PT_TLS` (Volume 17 F11) and initialized per C2. The linker computes the same formula at static-link time (Volume 17 OB-17-11).

ABI8 Version compatibility. The runtime ABI version is a u32 stamped in `.note.planck` (Volume 17 F14); P1 value = 1. Link-time rule: all objects and the runtime archive in one link must carry the same ABI version, else E-LNK-06. There is no backward compatibility shim in P1: a version change is an edition-level event.

## 18.8 FFI boundaries and OS interface subset (integration contracts)

FFI boundaries. The FFI surface (attribute syntax, type mapping, convention selection) is Volume 10; the runtime's obligations at the boundary are: (a) unwinding across FFI is rejected (P10); (b) `#[ffi]` functions receive/return values per the Volume 10 tables with no runtime mediation; (c) the allocator ABI (ABI4) is the only supported way for foreign code to obtain Planck-heap memory; (d) foreign threads calling back into Planck code must first call `planck_thread_start`-compatible registration via `std::os::register_foreign_thread()`, which allocates the TLS block (C2); skipping registration and touching TLS is UB-2.

OS interface subset (settled for edition P1). The runtime and stdlib call exactly this closed list of OS entry points, grouped; anything not listed requires an explicit `#[ffi]` declaration by the program and is not part of the runtime contract:

| Group | Entry points (exact) |
| ----- | -------------------- |
| Memory | mmap, munmap, mprotect, brk |
| Threads and sync | pthread_create, pthread_join, pthread_mutex_init/lock/unlock/destroy, pthread_exit, futex |
| Process | fork, execve, waitpid, exit_group |
| File IO | open, read, write, close, lseek |
| Filesystem metadata | stat, fstat, rename, unlink, mkdir, rmdir, getcwd |
| Time and info | clock_gettime, sysconf |
| Signals | sigaction (runtime-internal use only: INS1) |
| Entropy | getrandom |
| Networking | socket, bind, listen, accept, connect, send, recv, sendto, recvfrom, shutdown, setsockopt, getsockopt, getaddrinfo, freeaddrinfo |

Rules: (a) the memory group is available in all hosted modes; (b) the networking group is linked only when the `net` feature is used (Volume 17 L14); (c) the entropy group backs `crypto::OsRng` (Volume 19 H1); (d) syscall failures map to `IoError` variants per Volume 19; (e) the list is closed — adding an entry is an edition change with a security review (Volume 24 interface).

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-18-1 | Startup layout: the argv/envp/auxv walk in I1 yields argc, argv, envp exactly matching the kernel contract for every argc ∈ 0..=8 and env count ∈ 0..=4 (auxv ignored except AT_RANDOM) | EXH (9 × 5 layouts with generated argv/env arrays) + Z3 (argc ≤ 16, env ≤ 8 symbolic walk) | FV-1 | tools/fv/ob18-1-startup.smt2 | G9, G15 |
| OB-18-2 | SP alignment: for every frame size F ∈ {0, 8, …, 65536} the `_start` + prologue sequences keep SP ≡ 0 (mod 16) at calls on both targets (joint with OB-16-11) | EXH (8192 × 2) | FV-0 | tools/fv/ob18-2-align | G9 |
| OB-18-3 | Allocator invariants: after any sequence of ≤ 10 alloc/dealloc ops with sizes in {16, 32, 48, 256, 4096, 8192}, no two live blocks overlap and every live block lies inside some acquired slab or mmap region | EXH (all op sequences over the 6 sizes, ≤ 10 ops, ≤ 3 distinct) + Z3 (bounded heap model, ≤ 12 ops) | FV-1 | tools/fv/ob18-3-alloc.smt2 | G2, G15 |
| OB-18-4 | Alignment guarantee: every allocation of size s ∈ 1..512 with align a ∈ {1,2,4,8,16,32} returns a pointer ≡ 0 (mod max(a, 16)) | EXH (512 × 6) | FV-0 | tools/fv/ob18-4-align | G2 |
| OB-18-5 | Mutex state machine legality: the 3-state futex protocol performs only legal transitions and no sequence of ≤ 6 interleaved lock/unlock ops on 2 threads deadlocks or corrupts state | EXH (full interleaving domain for 6 ops × 2 threads) | FV-0 (bounded stated) | tools/fv/ob18-5-mutex | G21 |
| OB-18-6 | Unwind lookup: binary search over a sorted `.eh_frame_hdr` table returns the unique FDE covering the PC or "none", for tables of ≤ 16 FDEs and PCs at range boundaries | Z3 (≤ 16 entries) + EXH (all boundary PCs for 4-entry tables) | FV-1 | tools/fv/ob18-6-unwind.smt2 | G9, G22 |
| OB-18-7 | Mode/feature matrix completeness: every (mode × feature) cell of §18.1 is assigned exactly one of {allowed, requires-component, rejected} and no cell is empty | EXH (5 × 9 matrix) | FV-0 | tools/fv/ob18-7-modes | G15 |
| OB-18-8 | Panic payload stability: the Formatted payload buffer address is fixed from first panic to catch-boundary completion (no reallocation path exists in P2) | REVIEW (invariant: single TLS allocation, no resize code path) | FV-2 | review/ob18-8-payload.md | G15 |
| OB-18-9 | Double panic terminates: from any state, the machine (unwinding, panicking, aborting) reaches abort 134 within ≤ 2 further panic events | REVIEW (finite state argument over P9) | FV-2 | review/ob18-9-double-panic.md | G11, G22 |
| OB-18-10 | TLS formula: tpoff(x) = o(x) − T yields values ≤ 0 and the variable resolves at tp + tpoff for all TLS symbol sets of ≤ 8 symbols, sizes ≤ 2^12, with tbss padding rules applied | Z3 (bounded instance set) | FV-1 | tools/fv/ob18-10-tls.smt2 | G9 |
| OB-18-11 | Runtime ABI closedness: every runtime symbol referenced by the codegen corpus is in the ABI1 list, and every ABI1 symbol required by a mode's subset is defined by that mode's runtime | EXH (corpus × subset checks) | FV-1 | tools/fv/ob18-11-abi | G15, G24 |
| OB-18-12 | Exit processing: exit handlers run in reverse registration order, fini_array in reverse link order, streams flush before exit_group; verified over registration sequences ≤ 6 handlers | EXH (all 6! orderings of a 3-handler set with nesting) | FV-0 (bounded stated) | tools/fv/ob18-12-exit | G15 |
| OB-18-13 | Blocking pool bound: at most BLOCKING_POOL_MAX = 16 threads are ever started by async IO wrappers | PROP (paired with REVIEW of the pool acquire path invariant) | FV-2 | review/ob18-13-pool.md | G13, G21 |

## Coverage ledger

Bullets in scope: 60, defined: 60.

| design.md bullet (§) | Abbreviated bullet | Defined in |
| -------------------- | ------------------ | ---------- |
| 25.1 | No-runtime mode | §18.1 M1 |
| 25.1 | Freestanding mode | §18.1 M2 |
| 25.1 | Minimal hosted mode | §18.1 M3 |
| 25.1 | Full hosted mode | §18.1 M4 |
| 25.1 | Custom runtime mode | §18.1 M5 |
| 25.2 | Program entry | §18.2 I1 |
| 25.2 | Static initialization | §18.2 I2 |
| 25.2 | TLS initialization | §18.2 I3 |
| 25.2 | Argument handling | §18.2 I4 |
| 25.2 | Environment access | §18.2 I5 |
| 25.2 | Standard streams | §18.2 I6 |
| 25.2 | Runtime startup | §18.2 I7 |
| 25.2 | Runtime shutdown | §18.2 I8 |
| 25.2 | Exit processing | §18.2 I9 |
| 25.3 | Default allocator | §18.3 MEM1 |
| 25.3 | Allocation interfaces | §18.3 MEM2 |
| 25.3 | Deallocation interfaces | §18.3 MEM3 |
| 25.3 | Allocator initialization | §18.3 MEM4 |
| 25.3 | Allocation failure behavior | §18.3 MEM5 |
| 25.3 | Memory alignment | §18.3 MEM6 |
| 25.3 | Custom allocators | §18.3 MEM7 |
| 25.3 | Arena interfaces | §18.3 MEM8 |
| 25.3 | Stack allocation | §18.3 MEM9 |
| 25.3 | Runtime allocation diagnostics | §18.3 MEM10 |
| 25.4 | Panic entry | §18.4 P1 |
| 25.4 | Panic payload | §18.4 P2 |
| 25.4 | Panic reporting | §18.4 P3 |
| 25.4 | Unwinding mechanism | §18.4 P4 |
| 25.4 | Exception metadata | §18.4 P5 |
| 25.4 | Stack unwinding | §18.4 P6 |
| 25.4 | Destructor execution | §18.4 P7 |
| 25.4 | Abort behavior | §18.4 P8 |
| 25.4 | Double panic | §18.4 P9 |
| 25.4 | Foreign unwind boundaries | §18.4 P10 |
| 25.4 | Runtime configuration | §18.4 P11 |
| 25.5 | Thread support | §18.5 C1 |
| 25.5 | Thread-local storage | §18.5 C2 |
| 25.5 | Synchronization primitives | §18.5 C3 |
| 25.5 | Atomic support | §18.5 C4 |
| 25.5 | Scheduler support | §18.5 C5 |
| 25.5 | Async executor interfaces | §18.5 C6 |
| 25.5 | Task allocation | §18.5 C7 |
| 25.5 | Wake mechanisms | §18.5 C8 |
| 25.5 | Cancellation | §18.5 C9 |
| 25.6 | Stack overflow detection | §18.6 INS1 |
| 25.6 | Profiling hooks | §18.6 INS2 |
| 25.6 | Allocation tracking | §18.6 INS3 |
| 25.6 | Sanitizer hooks | §18.6 INS4 |
| 25.6 | Coverage instrumentation | §18.6 INS5 |
| 25.6 | Runtime assertions | §18.6 INS6 |
| 25.6 | Backtrace support | §18.6 INS7 |
| 25.6 | Debugger integration | §18.6 INS8 |
| 25.7 | Runtime entry points | §18.7 ABI1 |
| 25.7 | Runtime data structures | §18.7 ABI2 |
| 25.7 | Unwind ABI | §18.7 ABI3 |
| 25.7 | Allocator ABI | §18.7 ABI4 |
| 25.7 | Panic ABI | §18.7 ABI5 |
| 25.7 | Thread ABI | §18.7 ABI6 |
| 25.7 | TLS ABI | §18.7 ABI7 |
| 25.7 | Version compatibility | §18.7 ABI8 |
