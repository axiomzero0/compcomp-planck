<!-- CEP:FILE: spec/volume-08-concurrency-async.md
CEP:WHAT: Volume 08 of the Planck Master Specification Library: the concurrency and asynchronous-execution specification — the memory consistency model (SC-Per-Location + release/acquire + seq-cst fences), the atomic model with exact per-ordering guarantees, the data-race definition (UB-3), the thread model, Send/Sync capabilities, synchronization primitives, the future model, the async state-machine lowering contract, executors, and generators.
CEP:WHY: design.md §12 (lines 1086–1140) and §13 (lines 1141–1199) require a complete concurrency memory model and async semantics ("For a general-purpose language, this deserves its own specification", verification standards.md G21); this volume is the single normative definition home for all 80 bullets of design.md §12–§13 and settles the pinned async/cancellation model of SPEC-INDEX §3 (structured concurrency, cooperative cancellation, no detached tasks).
CEP:STATUS: complete
CEP:FAILURE: This volume must be rejected if any design.md §12–§13 bullet lacks a coverage-ledger row, if the memory model deviates from the pinned form (SC-Per-Location + release/acquire + seq-cst fences with happens-before = transitive closure of sequenced-before ∪ synchronizes-with), or if any rule lacks an exact trigger, code, constant, or complete enumeration.
CEP:ASSUMES: SPEC-INDEX §3 (UB-3 data-race category; async/cancellation model; no detached tasks in edition P1), Volume 06 (ownership, Send/Sync-enforcing region system), Volume 07 (effect atoms io/alloc/panic/unsafe/async/concurrency/ffi/nondet; trap reasons; `?` semantics), Volume 10 (intrinsics, ABI), Volume 12 (limits), Volume 18 (runtime), standards.md §3 (normative verbs), verification standards.md G2, G13, G14, G15, G21.
CEP:COST: compile-time: Send/Sync and atomic static checks are linear in type/operation counts. Runtime: atomics map to native instructions (§8.5.8); executors allocate per task (task state block, exact size Volume 18); no hidden cost beyond those two (Law 1 compliance).
CEP:EVIDENCE: verification matrix §8.11 (OB-08-1..OB-08-10); coverage ledger §8.12 (bullets in scope: 80, defined: 80).
-->

# Volume 08 — Concurrency and Async

## 8.0 Scope and traceability

This volume defines design.md §12 "Concurrency semantics" (lines 1086–1140) and design.md §13 "Async system" (lines 1141–1199). §12 contains 4 subsections (§12.1–§12.4) and §13 contains 4 subsections (§13.1–§13.4), together **80 bullets**, each of which receives exactly one normative definition below. Section mapping: §8.2 ↔ design.md §12.1, §8.3 ↔ §12.2, §8.4 ↔ §12.3, §8.5 ↔ §12.4, §8.6 ↔ §13.1, §8.7 ↔ §13.2, §8.8 ↔ §13.3, §8.9 ↔ §13.4. The three deliverables named at design.md line 1196 (Concurrency Memory Model, Async Semantics Specification, Executor Interface Specification) are delivered by §8.1/§8.4/§8.5, §8.6/§8.7/§8.9, and §8.8 respectively.

The memory consistency model of §8.1 is the shared contract referenced by §8.4 and §8.5 and by Volume 07 §7.5 (optimization barriers). Normative verbs have the meaning of standards.md §3. Compile-time rejection codes are `C08-E<nnn>` (closed set: C08-E001..C08-E006, enumerated in §8.9 d8). All named constants have exact values.

## 8.1 Memory consistency model

### 8.1.1 Executions and events

An execution is a set of events. Each memory event is the tuple `e = (tid, kind, loc, ord, val)` where: `tid ∈ ThreadId`; `kind ∈ {load, store, rmw, fence}`; `loc` is a byte range `[a, a+w)` with base address a and width w ∈ {1, 2, 4, 8, 16}; `ord` is the ordering, one of the exact enum `{Relaxed, Acquire, Release, AcqRel, SeqCst}` for accesses; fences carry fence-order ∈ `{Acquire, Release, SeqCst}` in `ord`; `val` is the value read or written. Non-atomic accesses have `ord = Relaxed` and are marked `nonatomic: true`. Events also include thread-management events (spawn, join) and executor wake events (§8.8).

### 8.1.2 Derived relations

- **Sequenced-before (sb):** the program order of each thread; a strict total order over the events of one thread.
- **Reads-from (rf):** `rf(r) = w` iff load r reads the value written by store w, with identical width and overlapping `loc`.
- **Synchronizes-with (sw):** exactly three rules, complete:
  - SW-R: W is a store with `ord ∈ {Release, AcqRel, SeqCst}`, R is a load with `ord ∈ {Acquire, AcqRel, SeqCst}`, and `rf(R) = W` → `W sw R`.
  - SW-F: F₁ is a fence with fence-order ∈ {Release, SeqCst}; W is any atomic store; F₁ sb W; R is any atomic load; `rf(R) = W`; R sb F₂ where F₂ is a fence with fence-order ∈ {Acquire, SeqCst} → `F₁ sw F₂`.
  - SW-S: S₁ is a SeqCst fence; W any atomic store with S₁ sb W; R any atomic load with `rf(R) = W`; R sb S₂ where S₂ is a SeqCst fence → `S₁ sw S₂`.
- **Happens-before (hb):** the transitive closure of `sb ∪ sw`, exactly: `hb = (sb ∪ sw)⁺`. This is the pinned definition. hb is a strict partial order in every consistent execution (OB-08-2).
- **Modification order (mo):** for each location l (a maximal byte range accessed by aligned atomic operations of one width), `mo_l` is a strict total order over all writes to l.

### 8.1.3 Consistency axioms

An execution is consistent iff all of:

- C1 (atomicity): every load reads from exactly one write of the same width and overlapping range; no torn reads.
- C2 (no future reads): `rf(r) = w` → ¬(r hb w).
- C3 (write-read coherence): `rf(r) = w` → ¬∃ write w′ to l: `w <_{mo_l} w′` ∧ w′ hb r.
- C4 (read-read coherence): r₁, r₂ loads of l with r₁ hb r₂ → ¬(`rf(r₂) <_{mo_l} rf(r₁)`).
- C5 (RMW atomicity): an RMW r of l reads the `mo_l`-maximal write w such that ¬∃ write w′: `w <_{mo_l} w′` ∧ w′ hb r, and r's own write immediately follows w in `mo_l`.
- C6 (SC, exactly SC-Per-Location + fences): there exists a strict total order S over all SeqCst accesses and all SeqCst fences such that (i) S contains sb restricted to those events; (ii) S contains sw restricted to those events; (iii) for every location l, the restriction of S to the SeqCst writes to l equals the restriction of `mo_l` to the same writes; (iv) a SeqCst load r of l reads from a write w permitted by C2/C3 such that no SeqCst write w′ to l satisfies `w <_S w′` ∧ w′ hb r.

The model is exactly: SC-Per-Location + release/acquire (SW-R, SW-F) + seq-cst fences (SW-S, C6). Weak-memory behavior is fully defined by C1–C6: any value choice permitted by a consistent execution is a defined value; there is no out-of-thin-air axiom beyond C2–C5 in edition P1, and this boundary is normative (programs must not rely on narrower behaviors than the axioms permit).

### 8.1.4 Ordering guarantees (exact, per ordering)

| Ordering | Applicable to | Exact guarantee |
| -------- | ------------- | --------------- |
| Relaxed | all atomics | Atomicity (C1), modification-order coherence (C2–C4); no synchronization, no hb edges |
| Acquire | loads, acquire fences | SW-R with the release store it reads from (SW-F for fences); prior writes of the releasing thread become visible (hb) |
| Release | stores, release fences | Enables SW-R for acquire loads reading from it (SW-F for fences); all prior writes in sb are released |
| AcqRel | RMW only | Acquire and Release simultaneously on the single RMW event |
| SeqCst | all atomics and fences | Release+Acquire semantics plus C6 (global order S over SeqCst events and fences) |

Compiler constraints per ordering (G21): an Acquire load must not be reordered with subsequent memory operations of the same thread; a Release store must not be reordered with prior memory operations; SeqCst operations must not be reordered with other SeqCst operations or fences; RMW operations must not be merged, duplicated, or split (Volume 07 §7.5 d8).

### 8.1.5 Data race definition (UB-3)

Two events e₁ ≠ e₂ form a **data race** iff all of: (i) their `loc` byte ranges overlap; (ii) they conflict: at least one is a store (or RMW); (iii) they are unordered by hb: ¬(e₁ hb e₂) ∧ ¬(e₂ hb e₁); (iv) they are not both atomic accesses of the same width. Race existence in a consistent execution is UB-3 (SPEC-INDEX §3): the program's subsequent behavior is undefined. Cases covered exactly: non-atomic vs non-atomic; non-atomic vs atomic (any width); atomic vs atomic of different widths. Same-width atomic pairs are never races (the model defines their values).

### 8.1.6 Scope of the guarantee

Safe programs (no `unsafe` blocks, no unsafe-reaching abstractions violated) have no UB-3 races: this is the data-race-freedom soundness statement, obligation OB-08-1 (REVIEW, invariants named there). Unsafe code assumes the obligation; the runtime bridge is the dynamic detector `plkc-racecheck` (FV-2 tooling, Volume 22).

### 8.1.7 Compiler reordering constraints (concurrent case)

A program transformation is concurrency-preserving iff for every consistent execution of the original program there is a consistent execution of the transformed program with the same observable behaviors (values of `io` and `nondet` operations, traps, panics). The optimizer must satisfy this for every pass that touches memory operations (G21 obligation; Volume 14/16 pass matrices reference OB-08-4). Transformations banned outright: introducing a read not in the source (speculative loads of possibly-racing locations), merging two atomic RMWs, weakening an ordering (§8.1.4).

## 8.2 Thread model (design.md §12.1)

**d1 — Thread creation.** `thread::spawn(f)` requires `f: FnOnce() -> T + Send + 'static` and `T: Send`; spawn moves f into the new thread (ownership transfer per Volume 06). `thread::scope(|s| { … })` allows non-`'static` borrows: `s.spawn(f)` requires `f: FnOnce() -> T + Send + 'scope`; the scope's region (Volume 06 §6.6) ends only after every spawned thread is joined, which the scope implementation must guarantee before returning (structured join). Both spawn forms carry the effect row `{concurrency}`.

**d2 — Thread termination.** A thread terminates when its closure returns (state Finished) or its panic unwinds to the thread root (d10). Termination releases the thread's storage; thread-locals are destroyed first (d4), then the closure's drops run.

**d3 — Thread ownership.** The closure owns everything it captured (moved at spawn); the JoinHandle owns the join obligation and the produced value. `JoinHandle<T>: Send` iff `T: Send`. Dropping a JoinHandle detaches (d7); `join()` consumes the handle.

**d4 — Thread-local data.** Per Volume 06 §6.2 d5: lazy per-thread initialization, destruction at thread exit in reverse initialization order, cross-thread access rejected at compile time (P06-E016). The effect of a thread-local access is that of the accessor body; initialization rows join the first accessor's ambient row.

**d5 — Thread migration.** Edition P1 threads are OS threads; there is no user-level migration of OS threads. `ThreadId` is a `u64`, assigned from 1 (main thread) monotonically, never reused. Task migration between worker threads of an executor is executor-level (§8.8 d4) and does not move OS threads; it is visible only through `nondet`.

**d6 — Thread joining.** `handle.join()` blocks until the thread is Finished, then returns `Result<T, JoinError>`; join consumes the handle; `JoinError` is the exact enum `{Panic}` (edition P1 has no thread cancellation, hence no other variant). A thread joining itself is a defined deadlock (non-termination, not UB, not a trap).

**d7 — Thread detachment.** A thread whose handle is dropped without join is detached: it continues running. At process exit (main returns, `exit()` called, or trap), detached threads are terminated at an arbitrary interrupt point: defined behavior, must not be UB; no memory events occur after the exit point, so no race arises. Detached threads must not be relied upon to complete (normative ban on relying on detached-thread completion at exit).

**d8 — Thread lifecycle.** The exact state set is `{New, Running, Finished, Joined}` with transitions: New → Running (scheduled), Running → Finished (return or panic unwind), Finished → Joined (join). No other transitions exist; the state machine is total (OB-08-6 domain).

**d9 — Stack allocation.** Default thread stack size is 2 MiB = 2,097,152 bytes. The minimum is 65,536 bytes; a requested size below the minimum is C08-E001. Requested sizes round up to the target page size (4096 bytes on both initial targets). Stack overflow at runtime is stack exhaustion → trap (Volume 07 §7.11 d12 reason 6). Main-thread stack size is the OS default and is not settable by the language.

**d10 — Thread failure.** A panic in a thread unwinds to the thread root (a catch boundary, Volume 07 §7.11 d5); the thread becomes Finished with join result `Err(JoinError::Panic)`; the payload is dropped, not delivered; the process continues. In `panic = abort` builds, the panic is the trap (exit 134, whole process). A panic in a scoped thread must be surfaced by the scope: every scoped spawn is joined before the scope returns, and the panic is delivered through the scope's join result (Volume 19 API); it must not be swallowed.

## 8.3 Send and Sync (design.md §12.2)

**d1 — Capability definitions.** `Send` and `Sync` are unsafe auto traits (Volume 04 §6). Exact defining equations: `T: Send` ⟺ a value of T may be transferred to another thread (by move into a spawn closure, channel send, or Arc transfer) without UB-3 risk; `T: Sync` ⟺ `&T: Send` ⟺ all shared access to T from any thread is race-free under the safe API of T. `Send` and `Sync` are independent capabilities; neither implies the other except via `&T`.

**d2 — Automatic derivation.** Derivation is structural and exact: a struct, enum, or tuple is `Send`/`Sync` iff every field/component is. The exact non-structural baseline: `&T` is Send iff `T: Sync`; `&mut T` is Send iff `T: Send`; `*const T`/`*mut T` are neither Send nor Sync; `Cell<T>` is Send iff `T: Send`, never Sync; `RefCell<T>` same; `Rc<T>` is neither; `Arc<T>` is Send+Sync iff `T: Send + Sync`; `Mutex<T>`/`RwLock<T>` are Send+Sync iff `T: Send`; `Box<T>` inherits T; function pointers `fn(…) !ρ` are always Send and Sync (they own no data); closures are Send+Sync iff their captures are, with the additional rule that an `Fn` closure used from multiple threads requires `Sync` captures (shared access across threads must be race-free).

**d3 — Explicit implementations.** `unsafe impl Send for X` / `unsafe impl Sync for X` override the structural derivation; they must be accompanied by a documented invariant per Volume 06 §6.9 d10 (preconditions, preserved properties, UB categories at violation). The invariant is review-checked (OB-08-1 relies on it for foreign and hand-written types).

**d4 — Generic propagation.** Bounds propagate: `struct S<T> { f: T }` is Send/Sync iff `T: Send`/`T: Sync`; where clauses may require the capabilities; monomorphization substitutes them exactly (Volume 04 instance law).

**d5 — Unsafe implementations.** Permitted, Restricted to documented invariants (d3). A safe (non-`unsafe`) impl of Send or Sync is banned outright: both are unsafe traits, so the grammar rejects the impl without the `unsafe` keyword; no keyword-independent pathway exists.

**d6 — Trait object behavior.** `dyn Trait` is Send/Sync only when declared `dyn Trait + Send + Sync`; the marker set is part of the object type; vtables carry no capability data (erasure per Volume 07 §7.3.13); a `dyn Trait + Send` value may cross threads but shared access still requires `Sync` for the concrete type — guaranteed by construction because forming `dyn Trait + Send` from `T` requires `T: Send` (upcast check, Volume 04).

**d7 — Foreign type behavior.** Types introduced by FFI default to ¬Send ∧ ¬Sync. Opt-in via `unsafe impl` with invariant (d3). There is no blanket escape: a foreign type used across threads without an impl is a compile-time rejection (C08-E003).

**d8 — Thread-transfer validation.** The validation points are exactly: `spawn`/`scope.spawn` bounds (d1); channel `send` requires `T: Send`; `Arc` cross-thread use requires `T: Send + Sync` (checked by Arc's impls); wake calls require `Waker: Send + Sync` (§8.6 d7). Violations are compile-time rejections C08-E003. Validation is static; there is no runtime capability check.

## 8.4 Synchronization (design.md §12.3)

All primitives in this section carry the effect atom `concurrency` (Volume 07 §7.2.1); blocking primitives additionally carry `nondet` (result timing varies). All are stdlib types (Volume 19 holds the API surface); this section fixes the semantics.

**d1 — Mutexes.** `Mutex<T>`: `lock() -> LockResult<MutexGuard<T>>` blocks until acquired; the guard gives `Deref`/`DerefMut` and unlocks on drop (drop order per Volume 06 §6.7.6). Poisoning, exact: if a panic unwinds while a guard is alive, the mutex is poisoned; every later `lock()` returns `Err(PoisonError<()>)` carrying no payload; `PoisonError` exposes the guard for recovery. `try_lock() -> Result<Option<Guard>, TryLockError>` never blocks. Mutex acquisition is a sw source: unlock (release) synchronizes-with the next successful lock (acquire) — normative, mapping to §8.1.2 SW-R.

**d2 — Read-write locks.** `RwLock<T>`: `read()` returns a shared guard (multiple concurrent readers), `write()` an exclusive guard; no fairness guarantee (nondet); poisoning on write-guard panic (same mechanism as d1); reader count is unbounded by the language (implementation limit documented in Volume 18). Unlock/lock pairs are sw sources per d1.

**d3 — Condition variables.** `Condvar::wait(guard)`: atomically releases the associated mutex, blocks, reacquires on wake; spurious wakeups are permitted (nondet) and callers must recheck predicates; `wait_timeout(guard, d) -> (LockResult<Guard>, bool)` where the bool is true iff the timeout d expired; `notify_one()`/`notify_all()` wake waiters (nondet which). notify is a sw source per d1's rule through the internal mutex handoff.

**d4 — Semaphores.** `Semaphore(n)` with count `n: u64`, n ≥ 0: `acquire() -> Permit` blocks at count 0 and decrements; `try_acquire() -> Option<Permit>`; releasing drops the Permit (increment). Permit is `!Send`? Settle: Permit is Send (it is the right to release from another thread); count never exceeds the initial n (release beyond n is impossible by construction: releases are Permit drops and Permits only come from acquisitions).

**d5 — Barriers.** `Barrier(n)`, n ≥ 1: `wait() -> BarrierWaitResult` blocks until n threads of the current generation have waited, then releases all; the generation increments; exactly one waiter per generation receives `is_leader == true` (the n-th arrival); barriers are reusable indefinitely.

**d6 — Channels.** `mpsc::channel<T>()` unbounded: `send` never blocks (allocates per message, `alloc` atom), queue capacity unbounded. `mpsc::sync_channel<T>(cap)` bounded, cap ≥ 0 (`u64`): `send` blocks while the queue holds cap items. `recv() -> Result<T, RecvError>` blocks until a message or close; close happens when all senders drop and the queue drains; `recv` then returns `Err(RecvError)`. `send` on a dropped receiver returns `Err(SendError(t))` returning the value (no loss). Channel send/receive are sw sources (message handoff = release/acquire pair, SW-R).

**d7 — Atomics.** §8.5 (the complete atomic model).

**d8 — Fences.** `fence(order)` intrinsics with fence-order ∈ {Acquire, Release, SeqCst} and exactly the SW-F/SW-S rules of §8.1.2; lowerings per §8.5.8. A fence is not tied to a location.

**d9 — Lock-free structures.** A stdlib structure claiming lock-freedom must: (a) use only §8.5 atomics; (b) state, per operation, the progress condition (lock-free: some operation completes in a bounded number of steps system-wide; the bound is stated per structure in Volume 19); (c) avoid ABA hazards without UB — retire/reclaim schemes must be stated in the structure's invariant (Volume 06 §6.9 d10 discipline). The language provides no hazard-pointer runtime in edition P1.

**d10 — Wait-free structures.** Same constraints as d9 with the stronger condition: every operation completes in a bounded number of steps independent of other threads' progress; the bound is a named constant per structure (Volume 19).

**d11 — Memory ordering.** The complete ordering semantics is §8.1.4; primitive APIs expose exactly the ordering enum {Relaxed, Acquire, Release, AcqRel, SeqCst}; no primitive exposes weaker or stronger notions (no Consume, no custom orderings) — closed enumeration.

**d12 — Deadlock considerations.** Deadlock is a defined liveness failure: program state where threads block cyclically; it is never UB, never a trap, and the language provides no detection or prevention. Tooling `plkc-lockview` (FV-2) reports static lock-order cycles as diagnostics; it must not gate compilation. Liveness obligations on primitives: none beyond d1–d6 blocking semantics (documented liveness gaps are FV-1 refinement gaps named per primitive in Volume 19).

## 8.5 Atomic model (design.md §12.4)

**d1 — Supported atomic widths.** Exact per-target table:

| Target | Widths (bytes) | Operations | Feature requirement |
| ------ | -------------- | ---------- | ------------------- |
| x86-64 SysV | 1, 2, 4, 8 | all (d3) | none |
| x86-64 SysV | 16 | load, store, compare_exchange (strong), SeqCst only | `cmpxchg16b` |
| AArch64 AAPCS64 | 1, 2, 4, 8 | all (d3) | none (RMW form per d8) |
| AArch64 AAPCS64 | 16 | — | compile-time rejection C08-E002 in edition P1 |

No other widths are supported; there are no bit-level atomics in edition P1.

**d2 — Atomic types.** `AtomicBool`, `AtomicI8`, `AtomicI16`, `AtomicI32`, `AtomicI64`, `AtomicU8`, `AtomicU16`, `AtomicU32`, `AtomicU64`, `AtomicIsize`, `AtomicUsize`, `AtomicPtr<T>` — the complete type set; `AtomicI128`/`AtomicU128` exist only on x86-64 with `cmpxchg16b` and are a compile-time rejection elsewhere (C08-E002). `AtomicIsize`/`AtomicUsize` are 8 bytes on both initial targets.

**d3 — Atomic operations.** Complete method set, exact: `load(ord)`, `store(v, ord)`, `swap(v, ord)` (exchange), `compare_exchange(current, new, success: ord, failure: ord)`, `compare_exchange_weak(current, new, success, failure)` (may fail spuriously, nondet), and the fetch family `fetch_add`, `fetch_sub`, `fetch_and`, `fetch_or`, `fetch_xor`, `fetch_max`, `fetch_min` (integers only; two's-complement wrap on overflow — defined, never a trap), plus `fetch_and`/`fetch_or`/`fetch_xor` for `AtomicBool` (logical). Pointers support load/store/swap/compare_exchange only. Every operation carries the `concurrency` atom (Volume 07 §7.2.1).

**d4 — Compare-and-swap.** `compare_exchange(current, new, success, failure)`: if the loaded value equals `current`, writes `new` and returns `Ok(previous)`; else returns `Err(actual)`. Constraint, exact: `failure ∈ {Relaxed, Acquire, SeqCst}` and `strength(failure) ≤ strength(success)` where strength order is `Relaxed < Acquire = Release < AcqRel = SeqCst`; violation is a compile-time rejection C08-E006. `compare_exchange_weak` may fail spuriously (nondet atom) and is the LL/SC-native form.

**d5 — Exchange.** `swap(v, ord)`: atomically writes v and returns the previous value; the RMW rules C5 apply.

**d6 — Fetch operations.** The fetch family computes the arithmetic/logic result and writes it back atomically in one RMW (C5); the returned value is the pre-operation value; integer overflow wraps (two's complement) — defined, not a panic.

**d7 — Memory ordering.** Per §8.1.4; the ordering argument is part of each call, checked statically to be a member of the closed enum.

**d8 — Atomic alignment.** The alignment of every atomic type equals its size (1, 2, 4, 8, or 16 bytes). A misaligned atomic access (address not divisible by the width) is the defined trap, Volume 07 §7.11 d12 reason 7 — not UB. Layout attributes (Volume 05) must not reduce atomic alignment; `#[repr(packed)]` on a struct containing an atomic field is a compile-time rejection C08-E004.

**d9 — Target feature requirements.** Feature use is static: the compiler enables `cmpxchg16b` codegen only when the target feature set includes it (compile command or target spec, Volume 16); `lse`/`rcpc` on AArch64 likewise. No runtime CPU dispatch is performed by the language; stdlib may offer runtime-dispatched wrappers (Volume 19) which trap (Volume 07 §7.11 d12 reason 8) when the feature is absent.

**d10 — Unsupported atomic behavior.** An atomic operation whose (width, ordering, op) combination is outside the d1 table for the target is a compile-time rejection C08-E002; there is no silent fallback, no emulation, no trap path for statically-known-unsupported forms (traps apply only to runtime-dispatched wrappers per d9).

**d11 — Compiler lowering.** Exact per target, normative:

| Operation | x86-64 | AArch64 (no LSE / with LSE) |
| --------- | ------ | ---------------------------- |
| Relaxed load | `mov` | `ldr` / `ldr` |
| Acquire load | `mov` (TSO) | `ldar` / `ldar` |
| SeqCst load | `mov` (TSO) | `ldar` / `ldar` |
| Relaxed store | `mov` | `str` / `str` |
| Release store | `mov` (TSO) | `stlr` / `stlr` |
| SeqCst store | `xchg` | `stlr` / `stlr` |
| Acquire fence | `lfence` | `dmb ishld` / `dmb ishld` |
| Release fence | `sfence` | `dmb ishst` / `dmb ishst` |
| SeqCst fence | `mfence` | `dmb ish` / `dmb ish` |
| RMW / CAS (1–8 B) | `lock`-prefixed (`lock xadd`, `lock cmpxchg`, `lock and/or/xor`) | `ldxr/stxr` loop / `ldadd*`, `cas*`, `swp` (LSE) |
| 16-byte (x86-64, `cmpxchg16b`) | `cmpxchg16b` sequences (CAS loop for load/store) | — (rejected) |

Lowerings must preserve the guarantees of §8.1.4 on the target model (x86-64 TSO; AArch64 other-multicopy-atomic): obligation OB-08-5. The `lock` prefix on x86-64 is SeqCst; RMW with weaker requested orderings still lower to lock-prefixed forms (defined: the implementation may be stronger than requested, never weaker).

## 8.6 Future model (design.md §13.1)

**d1 — Future type.** `trait Future { type Output; fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output>; }` with `enum Poll<T> { Ready(T), Pending }` — the exact trait shape (Volume 04 holds the trait-object form `dyn Future<Output = T>`). `Context` carries the `Waker` (d7).

**d2 — Future construction.** Calling an `async fn` or evaluating an `async { }` block constructs the state-machine object (§8.7) by moving the captures; no body code runs (d3); construction row per Volume 07 §7.4 d8.

**d3 — Lazy execution.** No statement of an async body executes before the first `poll`. This is observable and normative: construction must be side-effect-free except argument/capture evaluation and any `alloc` for heap-boxed futures.

**d4 — Polling.** `poll` drives the body from the current resume point until (a) completion: returns `Ready(output)` after running all drops of the body's frame; or (b) suspension at an `.await`: stores state, returns `Pending`. One poll may execute an unbounded amount of body work up to the next suspension (executors must not assume bounded polls; normative).

**d5 — Completion.** Once `Ready` is returned, the output is fixed and the future is complete. Polling a complete future must return `Ready` again with the same output (contract on all implementors; generated futures satisfy it by construction — the `Done` state, §8.7 d5). Output mutation after Ready is banned.

**d6 — Pending state.** A future returning `Pending` must have arranged resumption: it holds, or has handed the executor, a `Waker` for its task. A future that returns `Pending` without arranging a wake may never resume: defined liveness gap (programmer contract), not UB, not a trap.

**d7 — Waking.** `Waker::wake()` (and `wake_by_ref()`) marks the owning task Notified and schedules it (§8.8 d7 state machine); wake runs no user code; wake is permitted from any thread (`Waker: Send + Sync`, d8 of §8.3); waking after completion is a no-op.

**d8 — Scheduling.** Scheduling policy is executor-defined (§8.8); wake only enqueues. Deterministic executors (§8.8 d2) must define wake → enqueue → poll order exactly (FIFO by (priority, sequence)).

**d9 — Future composition.** Language-level composition is `.await` (Volume 07 §7.4 d8). Library combinators (Volume 19) must be implemented over `poll` only; their exact semantics: `join` = poll children to completion, cancelling losers only on parent cancellation; `race` = first `Ready` wins, losers dropped (dropping = cooperative cancellation, d11); combinators must propagate wakes (a child's wake must schedule the parent task).

**d10 — Future destruction.** Dropping a future runs the state-machine drop paths (§8.7 d6): live locals of the current state in reverse creation order, then captures in reverse declaration order. Dropping must not run any remaining body statements.

**d11 — Cancellation.** The cooperative model, per SPEC-INDEX §3, exactly: cancellation of a task = (i) the cancel flag is set (§8.8 d8); (ii) at the task's next await point — which is the defined cancel check: every `poll` entry consults the flag before dispatching to the future — the future is dropped (d10); (iii) the task completes as `Cancelled` and reaches its scope exit. Dropping a `JoinHandle` requests cancellation of its child (there are no detached tasks: every handle is tied to a scope). Cancellation of a parent cancels children first and the parent's join point completes only after all children complete (§8.8 d8). Await points are the only cancel checks; code between awaits is atomic with respect to cancellation (normative).

## 8.7 Async lowering (design.md §13.2)

The async state-machine lowering contract, exact:

**d1 — State-machine construction.** One generated type per `async fn` / async block; it implements `Future`; it is `!Unpin` iff any capture or any local live across an await is `!Unpin`, else `Unpin` (Volume 06 §6.10 d9). The type's fields: captures (moved in at construction), the per-state storage (d4), and the state discriminant (d2).

**d2 — Suspension points.** Each `.await` is a suspension point, numbered 0..n−1 in source order. The discriminant enumerates exactly {Start, Await₀, …, Awaitₙ₋₁, Done}; state Awaitᵢ means "resuming at the point immediately after await i". Discriminant width: the smallest unsigned integer type with width ≥ ⌈log₂(n+2)⌉ bits, rounded up to 1, 2, 4, or 8 bytes.

**d3 — Captured variables.** Captures are the async fn's parameters plus any borrowed names from the enclosing scope (closure-style capture per Volume 06 §6.4 d9); captures are moved into the object at construction; their regions (Volume 06 §6.6 d13) must cover every poll site of the owning task, enforced by the region solver.

**d4 — State fields.** Storage is the union over states i ∈ {Start, Await₀..Awaitₙ₋₁} of the locals live across await i at that state, laid out per Volume 05 (overlapping per-state extents allowed; exact layout math Volume 05 §7; the layout must be stated in the generated type's metadata for G18 reproducibility). Locals not live across any await are stack-allocated in the poll frame, not stored in the state machine.

**d5 — Resume dispatch.** `poll` dispatches on the discriminant: Start → begin the body; Awaitᵢ → jump to the resume point after await i; Done → return `Ready(output)` (the stored output; contract d5 of §8.6). Dispatch is a computed jump table (target lowering per Volume 16); semantics is the switch, not the table.

**d6 — Drop paths.** For each state i, the initialized set Iᵢ is the set of locals initialized at that suspension (computed by Volume 06 §6.7.2 `Init` facts at the await point). Dropping the future at state i runs drops for Iᵢ in reverse creation order, then the captures in reverse declaration order, then the discriminant becomes Done-without-output (drop-only). Every exit path (return, `?`, panic, cancellation drop) uses the same Iᵢ sets; no local is dropped twice and none is skipped: obligation OB-08-7.

**d7 — Error paths.** `?` in an async body (Volume 07 §7.11 d3): evaluate operand; on `Err(e)`/`None`, run the current-state drops (d6) and return `Ready(Err(From::from(e)))` (or `Ready(None)` for Option); the future is complete after this.

**d8 — Panic paths.** A panic inside `poll` unwinds: the current-state drops (d6) run during unwinding, then the panic propagates out of `poll` to the executor; the executor marks the task panicked (§8.8 d7) and its join yields `Err(JoinError::Panic)` (d2 of §8.2 mechanism). Double-panic inside poll follows Volume 07 §7.11 d6 (trap).

**d9 — Lifetime storage.** The generated type is generic over the capture lifetimes; `for<'a>`-style higher-ranked regions appear on the future type when the async fn is higher-ranked (Volume 04). The region solver (Volume 06 §6.6 d2) constrains capture regions to cover all poll sites; escape past the future's drop is P06-E009.

**d10 — Pin requirements.** `poll` takes `Pin<&mut Self>`; the executor must poll at a stable address from the first poll until drop (address stability, Volume 06 §6.10 d4); moving the future before the first poll is allowed; moving after the first poll is banned on safe paths and is UB-2-category under unsafe violation (Volume 06 §6.10 d11). Stack pinning (`pin!`) and heap pinning (`Box::pin`) are the sanctioned storages (Volume 06 §6.10 d2).

## 8.8 Executors (design.md §13.3)

**d1 — Executor interfaces.** The exact trait shape: `trait Executor { fn spawn<F>(&self, fut: F) -> JoinHandle<F::Output> where F: Future + Send + 'static, F::Output: Send + 'static; fn block_on<F: Future>(&self, fut: F) -> F::Output; }`. `JoinHandle<T>` implements `Future<Output = Result<T, JoinError>>` with `JoinError ∈ {Panic, Cancelled}` — the async task's exact enum (distinct from the OS-thread `JoinError ∈ {Panic}`, §8.2 d6). Dropping a `JoinHandle` requests cancellation of the child (§8.6 d11); there is no detach.

**d2 — Single-threaded executors.** One worker; ready queue is a priority queue ordered by (priority, sequence number), exact FIFO within equal priority; identical inputs produce identical schedules (G14 determinism; obligation OB-08-10); effect row `{alloc}` (no `concurrency`/`nondet`). `block_on` runs until the root future completes; tasks spawned during `block_on` must be driven to completion before `block_on` returns (structured join at the block_on boundary).

**d3 — Multi-threaded executors.** W worker threads with W = clamp(available_parallelism, 1, 256) (exact clamp); each worker owns a local deque; effect row `{concurrency, nondet}`; schedules are not deterministic (nondet atom); the executor must still satisfy §8.6/§8.1 semantics — nondeterminism is confined to interleaving, never to value validity.

**d4 — Work stealing.** Steal policy, exact: when a worker's local deque is empty it steals ⌈len/2⌉ items FIFO from a victim's deque (victim chosen nondeterministically), retrying across victims; if the system-wide queues are empty the worker parks (with wake-up on notify). Deque operations use §8.5 atomics (Chase-Lev shape; the exact algorithm is Volume 18's implementation obligation, semantics: LIFO local pop/push, FIFO steal, no lost items under C1–C6).

**d5 — Task queues.** Two queues per executor: the global inject queue (spawn from outside workers) and per-worker local deques; spawn from a worker pushes to its own local deque; the scheduler drains local before stealing. Queue bounds: global queue unbounded (alloc per entry); local deques bounded at 1024 entries with overflow to the global queue (exact constant, Volume 12 limits table).

**d6 — Scheduling policy.** The policy is per-executor and must be documented: stdlib single-threaded = FIFO (d2); stdlib multi-threaded = LIFO-local + half-steal (d4). Priorities are honored exactly on single-threaded executors and best-effort (nondet) on multi-threaded ones.

**d7 — Wake queues.** The task state machine, exact: states `{Idle, Notified, Running, Complete}`; transitions: wake on Idle → Notified + enqueue; wake on Notified → no-op (coalescing); wake on Running → set pending flag, consumed at poll exit (poll exit re-enqueues iff pending was set); poll entry: Notified → Running; poll exit with Pending → Idle (future returned Pending, no re-wake); task completion → Complete (wake no-ops thereafter). The state machine is total over (state × event {wake, poll-entry, poll-exit, complete}) — OB-08-6 domain.

**d8 — Cancellation propagation.** Exact algorithm per SPEC-INDEX §3: cancel(T) sets T's flag; if T is parked it is also marked Notified; at T's next poll entry the flag is consulted: if set, the executor drops T's future (§8.7 d6) and completes T as `Cancelled`. cancel(parent) must first cancel(parent.children) recursively and wait for their Complete states, then complete the parent: children are always cancelled before the parent's join point completes. Scope exit (`block_on` end or scope end) joins all children: the join point of a parent completes only after every child reaches Complete (OB-08-8).

**d9 — Task priority.** Supported: priority is a `u8`, 0 = highest, default 128; single-threaded executors honor it exactly (d2); multi-threaded executors treat it as best-effort (nondet).

**d10 — Executor-independent futures.** Futures must not reference executor internals; the only coupling mechanism is `Waker` (§8.6 d7). Stdlib futures must be pollable to completion on any executor conforming to d1 (conformance criterion; Volume 22 conformance suite). A future that stores an executor reference or assumes a specific scheduler is non-conforming and must be rejected in stdlib review (normative for the library, not a language rule).

## 8.9 Generators and yield (design.md §13.4)

Edition P1 settles the design.md conditional: `yield` exists, in generator bodies (`gen fn`).

**d1 — Generator values.** `trait Generator<R> { type Yield; type Return; fn resume(self: Pin<&mut Self>, arg: R) -> GeneratorState<Self::Yield, Self::Return>; }` with `enum GeneratorState<Y, R> { Yielded(Y), Complete(R) }` — the exact trait shape.

**d2 — Generator state.** Generated exactly like async state machines (§8.7 d1–d5) with yield points in place of awaits; the discriminant enumerates {Start, Yield₀..Yieldₙ₋₁, Done}; the resume argument is stored in the state storage and delivered as the value of the `yield` expression.

**d3 — Yielded values.** `yield e`: e evaluates (row = r_e ∪ {async}, Volume 07 §7.3.9 rule 11), its value is returned as `Yielded(e)`, and the generator suspends at the next state.

**d4 — Resumption.** `resume(arg)` drives from the current state to the next yield or completion; `resume` after `Complete` is a contract violation: the generated implementation panics with the fixed message "generator resumed after completion" (a defined panic, `panic` atom, not UB); custom implementations must obey the same contract (documented per-impl invariant).

**d5 — Generator destruction.** Drop paths per §8.7 d6 with the current yield state's Iᵢ sets; drops run reverse creation order; no body statements run at drop.

**d6 — Generator borrowing.** A borrow of a generator-local that lives across a `yield` is a compile-time rejection C08-E005. Borrows of captured data across yield are allowed with regions spanning the resumption sites (region solver, Volume 06 §6.6).

**d7 — Generator pinning.** Same Pin contract as futures: `resume` takes `Pin<&mut Self>`; stable address from first `resume` until drop; `Unpin` iff no `!Unpin` locals/captures live across yields (Volume 06 §6.10 d3/d9).

**d8 — Async-generator interaction.** `yield` inside `async fn` and `.await` inside `gen fn` are compile-time rejections C08-E005 (the generator-structure rejection code, shared with d6). The complete closed rejection set of this volume is: C08-E001 (stack size below minimum), C08-E002 (unsupported atomic width or missing target feature), C08-E003 (thread-transfer validation failure), C08-E004 (packed layout containing an atomic field), C08-E005 (generator-structure rejections), C08-E006 (CAS failure-ordering violation). Edition P1 has no async generators; adding them is an edition change, not a per-crate extension.

## 8.10 Integration points, invariants, and known limitations

### 8.10.1 Integration points

The exact interfaces between this volume and the rest of the library:

| Counterpart | Contract |
| ----------- | -------- |
| Volume 05 | Layout math of atomic types (alignment = size, §8.5 d8), of generated future/generator types (§8.7 d4), and of thread stacks (§8.2 d9) |
| Volume 06 | Loan/region enforcement that makes OB-08-1 hold for references (§8.1.6); pinning contract for futures (§8.7 d10); region solver over poll/resume sites (§8.7 d9, §8.9 d6) |
| Volume 07 | Effect atoms (`concurrency`, `nondet`, `async`) attached in §8.2–§8.9; trap reasons 7 and 8 (§8.5 d8, d9); `?` and panic paths of the lowering (§8.7 d7, d8) |
| Volume 10 | Intrinsic declarations for fences, atomics, `catch_unwind`-equivalent (§8.4 d8); ABI for thread spawn and callbacks |
| Volume 13 | IR 1 memory SSA carries provenance and event orderings for C1–C6 checking and for the G21 pass-equivalence obligations |
| Volume 16 | Instruction selection implements the §8.5 d11 lowerings; pass matrices must not violate §8.1.7 |
| Volume 18 | Runtime executor implementation (deque algorithm, task blocks, parking), thread stacks, and the exit behavior of detached threads (§8.2 d7) |
| Volume 19 | The API surface of §8.4 primitives, `JoinHandle`, `Waker`, combinators, and runtime-dispatched atomic wrappers |
| Volume 22 | OB-08 artifacts, the `plkc-racecheck` bridge, executor conformance suite (§8.8 d10) |

### 8.10.2 Named invariants

The soundness claims of this volume rest on exactly these invariants, each with its obligation: (I1) one live exclusive loan per place (Volume 06) → no safe conflicting access, OB-08-1; (I2) Send/Sync gates at every transfer point (§8.3 d8) → no cross-thread loan escape, OB-08-1; (I3) every Consistent execution satisfies C1–C6 with hb irreflexive, OB-08-2/3/4; (I4) lowerings preserve ordering guarantees, OB-08-5; (I5) generated state machines drop every live local exactly once, OB-08-7; (I6) cancellation reaches scope exit with children first, OB-08-8; (I7) single-threaded schedules are deterministic, OB-08-10.

### 8.10.3 Known limitations (exact, settled for edition P1)

Each limitation is a named boundary, not a deferral: (L1) no async generators (§8.9 d8, C08-E005); (L2) no 16-byte atomics on AArch64 (§8.5 d1, C08-E002); (L3) no thread cancellation and no async `JoinError` other than {Panic, Cancelled} (§8.2 d6, §8.8 d1); (L4) no detached async tasks — dropping a handle cancels (§8.8 d1); (L5) no fairness guarantee on locks or multi-threaded scheduling (§8.4 d2, §8.8 d6, `nondet`); (L6) no out-of-thin-air axiom beyond C2–C5 (§8.1.3) — programs must not rely on narrower behaviors; (L7) deadlock is undetected by the language (§8.4 d12); (L8) a `Pending` future that never arranges a wake never resumes (§8.6 d6, programmer contract).

## 8.11 Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-08-1 | Data-race-freedom soundness: every program accepted without `unsafe` has no UB-3 race on any execution and any input (G2 item 8) | REVIEW (invariants named: (i) Volume 06 grants at most one live exclusive loan per place and shared loans only for read access, so unsynchronized conflicting access through references is impossible; (ii) thread transfer requires Send/'static or scoped regions (§8.2 d1), so no reference outlives its owner across threads; (iii) shared mutation requires Sync types whose safe APIs perform release/acquire handoffs (§8.4) or atomics (§8.5); (iv) raw-pointer and foreign access exist only under `unsafe`, which this obligation excludes by definition; reason not machine-checkable: quantifies over all programs and all interleavings — an infinite domain; runtime bridge: `plkc-racecheck` dynamic detector (FV-2, Volume 22)) | FV-1 | `tools/fv/ob08_01.md` | G2, G21 |
| OB-08-2 | hb is a strict partial order on consistent executions: transitive by construction ((sb ∪ sw)⁺) and irreflexive (no event happens-before itself in a consistent execution) | Z3 (bounded executions: ≤ 3 threads × ≤ 4 events, all orderings of §8.1.4, enumerating candidate (sb, sw, rf) structures; check irreflexivity of closure) | FV-0 | `tools/fv/ob08_02.smt2` (timeout 10 s) | G21, G15 |
| OB-08-3 | SC-per-location consistency: for every consistent candidate execution there exists a total order S satisfying C6(i)–(iv), and for executions violating C6 no S exists | Z3 (same bounded domain as OB-08-02; existence encoding of S over SeqCst events) | FV-0 | `tools/fv/ob08_03.smt2` (timeout 10 s) | G21, G15 |
| OB-08-4 | Release/acquire synchronization: in every consistent execution, R rf W with W release and R acquire implies W hb R; fence rules SW-F/SW-S likewise; and lock/unlock handoffs of §8.4 d1–d3 map onto SW-R | Z3 (bounded executions with fences and lock handoff events) | FV-0 | `tools/fv/ob08_04.smt2` (timeout 10 s) | G21 |
| OB-08-5 | Lowering preserves orderings: the §8.5 d11 lowerings satisfy §8.1.4 per-ordering guarantees on the target models (x86-64 TSO; AArch64 other-multicopy-atomic); verified against 20 canonical litmus tests per target (SB, LB, IRIW, CoRR, CoWW, WRC, MP + fence variants) | EXH (20 litmus programs × 2 targets, all final-value observations checked against the permitted sets of the axioms) + REVIEW (invariant: every lowered sequence's allowed observations ⊆ axiom-permitted observations; reason: full target models are external standards, not finite domains) | FV-1 | `tests/litmus/` (40 programs) + `tools/fv/ob08_05.md` | G21, G16 |
| OB-08-6 | Task/executor state machines are total and stuck-free: every (state, event) pair of §8.8 d7 and §8.2 d8 has exactly one defined outcome | EXH (§8.8 d7: 4 states × 4 events = 16 cells; §8.2 d8: 4 states × transitions) | FV-0 | `math/` executor-model test `tests/task_state_machine.rs`; `tools/fv/ob08_06.py` | G15, G21 |
| OB-08-7 | Async drop coverage: for every generated async fn with ≤ 3 awaits and ≤ 4 locals (generated universe: 512 programs), every local initialized before a suspension is dropped exactly once on every exit path (return, error, panic, cancellation drop), and no local is dropped twice | EXH (512 generated programs × 4 exit paths, cross-checked against the reference interpreter's drop log) | FV-0 | `tools/fv/ob08_07.py` + interpreter corpus | G2, G15 |
| OB-08-8 | Cancellation law: on the reference single-threaded executor, for every rooted task tree of depth ≤ 3 with ≤ 2 children per node (15 tree shapes), cancelling any node results in all descendants completing as `Cancelled` before the cancelled node's join point completes, and every cancelled task reaches its scope exit with drops run | EXH (15 shapes × cancellation injected at each of the ≤ 7 nodes = 105 generated cases, schedule-verified) | FV-0 | `tools/fv/ob08_08.py` + reference executor | G21, G15 |
| OB-08-9 | Static atomic checks are complete: for every (width ∈ {1,2,4,8,16}, ordering ∈ 5, op ∈ 10) combination per target, the checker accepts exactly the d1/d4-legal combinations and rejects the rest with C08-E002/C08-E004/C08-E006 | EXH (5 × 5 × 10 = 250 combinations × 2 targets = 500 cases) | FV-0 | `tools/fv/ob08_09.py` + checker unit tests | G3, G15 |
| OB-08-10 | Single-threaded executor determinism: identical programs produce identical schedules (task completion order and outputs) across runs | EXH (bounded task graphs ≤ 6 tasks, ≤ 3 awaits each: 4,096 programs, two runs each, byte-identical logs) paired with PROP (deterministic xorshift64* PRNG, 10,000 samples over larger graphs) | FV-0 | `tools/fv/ob08_10.py` | G14, G21 |

## 8.12 Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| ------------------------------ | ---------- |
| 12.1 Thread creation | §8.2 d1 |
| 12.1 Thread termination | §8.2 d2 |
| 12.1 Thread ownership | §8.2 d3 |
| 12.1 Thread-local data | §8.2 d4 |
| 12.1 Thread migration | §8.2 d5 |
| 12.1 Thread joining | §8.2 d6 |
| 12.1 Thread detachment | §8.2 d7 |
| 12.1 Thread lifecycle | §8.2 d8 |
| 12.1 Stack allocation | §8.2 d9 |
| 12.1 Thread failure | §8.2 d10 |
| 12.2 Capability definitions | §8.3 d1 |
| 12.2 Automatic derivation | §8.3 d2 |
| 12.2 Explicit implementations | §8.3 d3 |
| 12.2 Generic propagation | §8.3 d4 |
| 12.2 Unsafe implementations | §8.3 d5 |
| 12.2 Trait object behavior | §8.3 d6 |
| 12.2 Foreign type behavior | §8.3 d7 |
| 12.2 Thread-transfer validation | §8.3 d8 |
| 12.3 Mutexes | §8.4 d1 |
| 12.3 Read-write locks | §8.4 d2 |
| 12.3 Condition variables | §8.4 d3 |
| 12.3 Semaphores | §8.4 d4 |
| 12.3 Barriers | §8.4 d5 |
| 12.3 Channels | §8.4 d6 |
| 12.3 Atomics | §8.4 d7, §8.5 |
| 12.3 Fences | §8.4 d8 |
| 12.3 Lock-free structures | §8.4 d9 |
| 12.3 Wait-free structures | §8.4 d10 |
| 12.3 Memory ordering | §8.4 d11, §8.1.4 |
| 12.3 Deadlock considerations | §8.4 d12 |
| 12.4 Supported atomic widths | §8.5 d1 |
| 12.4 Atomic types | §8.5 d2 |
| 12.4 Atomic operations | §8.5 d3 |
| 12.4 Compare-and-swap | §8.5 d4 |
| 12.4 Exchange | §8.5 d5 |
| 12.4 Fetch operations | §8.5 d6 |
| 12.4 Memory ordering | §8.5 d7, §8.1.4 |
| 12.4 Atomic alignment | §8.5 d8 |
| 12.4 Target feature requirements | §8.5 d9 |
| 12.4 Unsupported atomic behavior | §8.5 d10 |
| 12.4 Compiler lowering | §8.5 d11 |
| 13.1 Future type | §8.6 d1 |
| 13.1 Future construction | §8.6 d2 |
| 13.1 Lazy execution | §8.6 d3 |
| 13.1 Polling | §8.6 d4 |
| 13.1 Completion | §8.6 d5 |
| 13.1 Pending state | §8.6 d6 |
| 13.1 Waking | §8.6 d7 |
| 13.1 Scheduling | §8.6 d8 |
| 13.1 Future composition | §8.6 d9 |
| 13.1 Future destruction | §8.6 d10 |
| 13.1 Cancellation | §8.6 d11 |
| 13.2 State-machine construction | §8.7 d1 |
| 13.2 Suspension points | §8.7 d2 |
| 13.2 Captured variables | §8.7 d3 |
| 13.2 State fields | §8.7 d4 |
| 13.2 Resume dispatch | §8.7 d5 |
| 13.2 Drop paths | §8.7 d6 |
| 13.2 Error paths | §8.7 d7 |
| 13.2 Panic paths | §8.7 d8 |
| 13.2 Lifetime storage | §8.7 d9 |
| 13.2 Pin requirements | §8.7 d10 |
| 13.3 Executor interfaces | §8.8 d1 |
| 13.3 Single-threaded executors | §8.8 d2 |
| 13.3 Multi-threaded executors | §8.8 d3 |
| 13.3 Work stealing | §8.8 d4 |
| 13.3 Task queues | §8.8 d5 |
| 13.3 Scheduling policy | §8.8 d6 |
| 13.3 Wake queues | §8.8 d7 |
| 13.3 Cancellation propagation | §8.8 d8 |
| 13.3 Task priority, if supported | §8.8 d9 (supported) |
| 13.3 Executor-independent futures | §8.8 d10 |
| 13.4 Generator values | §8.9 d1 |
| 13.4 Generator state | §8.9 d2 |
| 13.4 Yielded values | §8.9 d3 |
| 13.4 Resumption | §8.9 d4 |
| 13.4 Generator destruction | §8.9 d5 |
| 13.4 Generator borrowing | §8.9 d6 |
| 13.4 Generator pinning | §8.9 d7 |
| 13.4 Async-generator interaction | §8.9 d8 (rejected in P1) |

bullets in scope: 80, defined: 80
