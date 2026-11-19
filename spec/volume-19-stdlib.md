<!-- CEP:FILE: spec/volume-19-stdlib.md
CEP:WHAT: Volume 19 of the Planck Master Specification Library: the standard library specification — the closed module inventory, per-module API contracts with exact Rust-like signatures and effect rows, core primitives (Option/Result/never semantics exact), collections with exact per-operation complexity guarantees and exact resize constants, the closed deterministic hasher with defined iteration order, IO with static dispatch (no trait objects per standards §25.6), OS interfaces over the Volume 18 closed syscall subset, concurrency primitives aligned with Volume 08 (SC-Per-Location model), text, numeric, networking, testing, cryptography interfaces, and the per-module guarantee table.
CEP:WHY: design.md §26 (lines 2955–3139) requires the deliverable "Standard Library API and Behavioral Specification" and states that the standard library determines whether ordinary software is pleasant to write; all 133 bullets of §26.1–§26.12 receive exactly one normative definition here, consistent with Volume 07 (effects, panics, traps), Volume 08 (memory model, sync, executors), Volume 10 (intrinsics, SIMD, FFI), and Volume 18 (runtime modes, closed OS subset, blocking pool).
CEP:STATUS: complete
CEP:FAILURE: This volume must be rejected if any design.md §26 bullet lacks a coverage-ledger row, if any named constant lacks an exact value, if any public signature is not exactly stated, if the API surface uses dynamic dispatch (dyn), or if any contract relies on an unspecified hasher, an unspecified resize policy, or a nondeterministic iteration order.
CEP:ASSUMES: SPEC-INDEX §1 (definition method), §3 (settled decisions), §5.3 (effect atoms); Volume 04 (types, traits, variance); Volume 05 (layout); Volume 06 (ownership, Pin §6.10); Volume 07 (effects, panic conditions §7.6 d8, trap reasons §7.11 d12); Volume 08 (SC-Per-Location model §8.1, sync §8.4, atomics §8.5, futures/executors §8.6/§8.8); Volume 10 (intrinsics §10.5, SIMD §10.7, FFI §10.8); Volume 12 (limits); Volume 17 (linkage); Volume 18 (modes §18.1, OS subset §18.8, blocking pool); standards.md §3 (normative verbs), §5.1 (CEP-0), §25.6 (trait-object policy); verification standards.md G2, G3, G11, G13, G14, G15, G16, G21, G22.
CEP:COST: compile-time: none beyond monomorphization (Volume 12 limits). Runtime: allocation occurs only at the documented sites of §19.1.5 (Law 1); the default hasher is unsalted (no per-process seed); collection resize uses the exact factors of §19.4; the async IO pool is bounded at BLOCKING_POOL_MAX = 16 threads (Volume 18 OB-18-13). No hidden cost beyond these documented sites.
CEP:EVIDENCE: verification matrix §19.14 (OB-19-1..OB-19-26); coverage ledger §19.15 (bullets in scope: 133, defined: 133).
-->

# Volume 19 — Standard Library

## 19.0 Scope and traceability

This volume defines design.md §26 "Standard library specification" (lines 2955–3139). §26 contains 12 subsections (§26.1–§26.12) and **133 bullets**; every bullet receives exactly one normative definition below. Section mapping: §19.2 ↔ design.md §26.1, §19.3 ↔ §26.2, §19.4 ↔ §26.3, §19.5 ↔ §26.4, §19.6 ↔ §26.5, §19.7 ↔ §26.6, §19.8 ↔ §26.7, §19.9 ↔ §26.8, §19.10 ↔ §26.9, §19.11 ↔ §26.10, §19.12 ↔ §26.11, §19.13 ↔ §26.12. The §26 deliverable ("Standard Library API and Behavioral Specification") is delivered by §§19.1–19.13 as a whole; §26.11's preamble sentence (provide a maintained library or specify ecosystem interfaces) is settled normatively in §19.12.

Normative verbs have the meaning of standards.md §3. Effect rows use the pinned atom set {io, alloc, panic, unsafe, async, concurrency, ffi, nondet} (Volume 07 §7.1); every contract states its row. Traps are the 11 closed reasons of Volume 07 §7.11 d12; panics are defined unwinding to the enclosing catch boundary (Volume 07 §7.11). Compile-time rejection codes introduced here: C19-E001..C19-E004 (std build gates, §19.13 g12/g13), P19-FMT-1..3 (format strings, §19.2 d8), P19-ALN, P19-RNG (const-generic rejections, §19.4 d17/d13). All named constants have exact values.

## 19.1 Library structure, inventory, and global policies

**19.1.1 Module inventory (closed list).** The standard library is one crate, `std`, with exactly 11 top-level modules; adding, removing, or renaming a top-level module is an edition change. CEP class "0/1" means: the module's non-allocating, non-IO entry points are CEP-0; all other entry points are CEP-1; formatting-capable functions are always CEP-1 (standards §5.1 bans formatting in CEP-0).

| Module | Defines design.md | CEP class | Linked in mode M2 (freestanding) |
| ------ | ----------------- | --------- | -------------------------------- |
| std::core | §26.1 | 0/1 | yes |
| std::mem | §26.2 | 0/1 | yes |
| std::coll | §26.3 | 0/1 | yes |
| std::text | §26.4 | 0/1 | yes |
| std::num | §26.5 | 0 | yes |
| std::io | §26.6 | 1 | no |
| std::os | §26.7 | 1 | no |
| std::sync | §26.8 | 0/1 | no |
| std::net | §26.9 | 1 | no |
| std::test | §26.10 | 2 | no |
| std::crypto | §26.11 | 1 | yes, except crypto::OsRng |

Mode M2 links exactly the five marked modules (plus the OsRng-excluded crypto); mode M1 links none of std; modes M3/M4 link all of std (Volume 18 §18.1).

**19.1.2 Prelude.** The prelude, auto-imported into every module without declaration, contains exactly these 14 names: Option, Result, Some, None, Ok, Err, Vec, String, Ordering, PartialEq, Eq, Drop, Iterator, Error. The list is closed.

**19.1.3 No dynamic dispatch in std.** No public std type, function, or trait uses a trait object: `dyn` is banned in the entire std source and API surface (strictest application of standards §25.6, because std must be usable from CEP-0 programs). All polymorphism is generics (monomorphized per Volume 12), closed enums, or function-pointer tables where measured. Violation is std build gate rejection C19-E001.

**19.1.4 Error and failure discipline.** Every fallible std function returns `Result<T, E>` with a complete error enum; the error enums of this volume are closed: IoError (§19.7.0), TextError (§19.5 d1), ParseError (§19.5 d6), ConversionError (§19.6 d5), AddressError (§19.10 d4), DnsError (§19.10 d5), CryptoError (§19.12 d3), AllocError (§19.3 d8), TryLockError (§19.9 d2), JoinError (thread: {Panic}, §19.9 d1; async: {Panic, Cancelled}, §19.9 d10). Safe std functions must not panic except at the declared sites of §19.13 g7; every declared panic site is listed in the item's CEP:FAILURE field. Operations that trap are noted per contract; traps are not panics.

**19.1.5 Allocation policy.** Heap allocation occurs only at these documented sites: collection growth (§19.4 growth rules), String/StringBuilder/ByteBuffer growth, Box/Rc/Arc/SecretBox/Waker (waker_fn) construction, channel message queueing (per message), tree node insertion, executor task state blocks, async IO pool submissions (per call), and the std::test harness (CEP-2). Every allocating function carries the alloc atom in its row. Allocation failure at the Volume 12 limit is the defined trap reason 5 (Volume 07 §7.11 d12); std never returns a null allocation to safe code.

**19.1.6 Hasher policy (closed deterministic hasher).** The default hasher `std::coll::hash::PlanckHasher` is exactly: (1) FNV-1a over the hashed byte string: h ← 0xcbf2_9ce4_8422_2325; for each byte b: h ← (h ⊕ b) × 0x0000_0100_0000_01b3 (u64, two's-complement wrapping); (2) splitmix64 finalizer: z ← h + 0x9e37_79b9_7f4a_7c15; z ← (z ⊕ (z >> 30)) × 0xbf58_476d_1ce4_e5b9; z ← (z ⊕ (z >> 27)) × 0x94d0_49bb_1331_11eb; result ← z ⊕ (z >> 31). All constants exact; all arithmetic u64 wrapping. The hasher has no seed, no per-process randomness, no time or address input: it is a pure function of the byte string (G14). The salted constructor `PlanckHasher::new(k0: u64, k1: u64)` mixes k0 and k1 into h₀ (h₀ ⊕ k0, × prime, ⊕ k1, × prime) and is Cold-only for CEP-0 code (hash-flooding mitigation for adversarial keys, Volume 24 interface); HashMap and HashSet must use the unsalted default unless constructed with an explicit hasher. The byte string fed by a Hash impl is fixed by the type (§19.2 d7); a type feeding different byte strings for equal values is std build gate rejection C19-E002.

**19.1.7 Iteration-order determinism.** No std container has time-, thread-, or address-dependent iteration order (CEP-0 rule; standards §5.1). HashMap/HashSet iterate buckets in ascending bucket index; BTreeMap/BTreeSet iterate keys in Ord ascending order; all other containers iterate in structural order (front-to-back, insertion order, or document order). Two executions of the same program with the same insertion sequences produce the same iteration order (G14; OB-19-7).

**19.1.8 Naming and ABI.** std functions use the Planck fast convention internally (Volume 10 §10.8); no std function is C-ABI-callable unless declared `#[ffi]`. The only std-provided C-ABI symbols are the ABI4 allocator wrappers (Volume 18 §18.7 ABI4: planck_alloc, planck_dealloc, planck_realloc, planck_alloc_zeroed). std links as a static library selected per runtime mode (Volume 17); internal signatures may change between minor versions (§19.13 g9).

## 19.2 Core (design.md §26.1)

**d1 — Primitive traits; Copy.** `pub trait Copy: Clone {}` is a marker trait: a type is Copy iff it has no drop glue and all fields are Copy. Copy types are duplicated instead of moved on assignment (Volume 06 §6.4). The std Copy impl set is closed: all integer and float primitives, bool, char, Ordering, `&T` (shared references are Copy; `&mut T` is not), raw pointers, NonNull, PhantomData, arrays [T; N] over Copy, tuples of Copy types, Option/Result over Copy components, Range types. Row ∅.

**d2 — Clone.** `pub trait Clone { fn clone(&self) -> Self; }` must produce a value equal to self under the type's Eq, or a deep copy of its owned parts. #[derive(Clone)] requires every field Clone; #[derive(Copy)] requires every field Copy. Rows: Rc::clone is ∅ (non-atomic increment); Arc::clone is {concurrency} (atomic RMW); all other std clones that allocate carry {alloc}. Violation of the equality law is checked by the certificate tests of OB-19-3's domain.

**d3 — Drop.** `pub trait Drop { fn drop(&mut self); }` runs at scope exit on every path, innermost-first (Volume 06 §6.7); row = the type's declared drop row (Volume 07 §7.4 d10). Drop::drop must not move out of self (checker-enforced). Drop glue order: the type's Drop::drop first, then fields in reverse declaration order (Volume 05 §7). A panic inside drop while unwinding is trap reason 11 (Volume 07 §7.11 d6).

**d4 — Default.** `pub trait Default { fn default() -> Self; }`. Exact std values: numeric types 0; bool false; char U+0000; Option → None; Result → Ok(T::default()) where T: Default; Vec/String/HashMap/HashSet/BTreeMap/BTreeSet/VecDeque/LinkedList/BinaryHeap/BitVec/ByteBuffer/StringBuilder → empty with no allocation; Box has no Default in edition P1; references have no Default. #[derive(Default)] requires every field Default. Row ∅ unless the impl allocates.

**d5 — Equality.** `pub trait PartialEq<Rhs = Self> { fn eq(&self, other: &Rhs) -> bool; }`; `pub trait Eq: PartialEq<Self> {}`. Laws (OB-19-3): Eq implementations must be reflexive, symmetric, transitive; PartialEq without Eq may be partial. The only std partial-equality family is floating point (IEEE-754-2019: NaN ≠ NaN, +0 == −0). Other std impls: structural for products, elementwise for slices/arrays/Vec/String, identity (address + allocation ID, SPEC-INDEX §3) for references, value equality for primitives and Ordering.

**d6 — Ordering.** `pub enum Ordering { Less, Equal, Greater }`; `pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs> { fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>; }`; `pub trait Ord: Eq + PartialOrd<Self> { fn cmp(&self, other: &Self) -> Ordering; fn max(self, other: Self) -> Self; fn min(self, other: Self) -> Self; }`. Laws (OB-19-2): cmp is total, antisymmetric, transitive; max(a, b) = a iff cmp(a, b) ≠ Less, else b; min symmetric. Float types have no Ord in edition P1; total ordering over floats is num::total_cmp (§19.6 d2, IEEE-754 totalOrder). #[derive(Ord)] compares fields lexicographically in declaration order and requires every field Ord.

**d7 — Hashing.** `pub trait Hash { fn hash<H: Hasher>(&self, state: &mut H); }`; `pub trait Hasher { fn write(&mut self, bytes: &[u8]); fn finish(&self) -> u64; }`. Contract: the byte sequence a type's Hash impl feeds is a pure function of the value; equal values (Eq) must feed equal byte strings (C19-E002; OB-19-3 domain). Derived Hash feeds fields in declaration order; char feeds its UTF-8 encoding; integers feed their little-endian byte representation. The default Hasher is §19.1.6; HashMap::with_hasher accepts any Hasher.

**d8 — Formatting.** `pub trait Format { fn format(&self, f: &mut Formatter) -> Result<(), FormatError>; }`. Formatter writes into a caller-provided sink (StringBuilder or a fixed [u8] buffer) and never allocates itself. Format-string grammar, exact productions: fmtstring ::= (lit | spec)*; lit ::= "{{" | "}}" | any char except "{" or "}"; spec ::= "{" [name] [":" fmtspec] "}"; fmtspec ::= [[fill] align] [sign] ["#"] [width] ["." precision] [type]; align ∈ {<, >, ^}; sign ∈ {+, −}; width and precision are u16 decimal literals; type ∈ {empty, b, o, x, X, e, E, ?} (empty = Display decimal; ? = DebugFormat). Semantics: fill defaults to space; align defaults to < for numeric types and to none for strings; width = minimum field width; precision on numbers = exactly that many digits after the point (round-half-even on the exact decimal expansion); precision on strings = maximum length; "#" prefixes 0b/0o/0x for base types. The format string is checked at the call site: P19-FMT-1 unbalanced brace, P19-FMT-2 unknown argument name, P19-FMT-3 invalid fmtspec — all compile-time rejections. FormatError = {WriteFailed, NoSpace} (NoSpace arises only for fixed sinks). Rows: ∅ into fixed sinks; {alloc} when the sink grows. Cost: formatting is CEP-1; per written character O(1).

**d9 — Debug formatting.** `pub trait DebugFormat { fn debug_format(&self, f: &mut Formatter) -> Result<(), FormatError>; }`, the "{" ? "}" type. Exact std conventions: structs `Name { field: value, … }`; tuples `(v1, v2)`; enums `Variant(value)`; sequences `[v1, v2]`; strings quoted with escapes \n \t \\ \" and \u{hex}; chars 'c'; u8 slices as hex lists. DebugFormat is required by assert_eq and by panic payload rendering (Volume 18 §18.4).

**d10 — Conversion.** `pub trait From<T> { fn from(value: T) -> Self; }` with blanket `impl<T, U: From<T>> Into<T> for U { fn into(self) -> T }`; `pub trait TryFrom<T>: Sized { type Error; fn try_from(value: T) -> Result<Self, Self::Error>; }`. The `?` operator uses From::from on the error (Volume 07 §7.11 d3). Laws: From impls are total and lossless; TryFrom error sets are complete. Closed std impl set: identity conversions; widening integer conversions; String → Vec<u8> (UTF-8 bytes); u64/i64 → f64/f32 via From (round-to-nearest-ties-even; precision loss documented, not an error); Vec<u8> → String via TryFrom (validation, TextError); every conversion with a possible range failure is TryFrom (§19.6 d5). From/TryFrom impls must not overlap ambiguously (Volume 04 coherence).

**d11 — Iterator interfaces.** `pub trait Iterator { type Item; fn next(&mut self) -> Option<Self::Item>; fn size_hint(&self) -> (usize, Option<usize>) { (0, None) } }`; `pub trait DoubleEndedIterator: Iterator { fn next_back(&mut self) -> Option<Self::Item>; }`; `pub trait ExactSizeIterator: Iterator { fn len(&self) -> usize; }` with invariant len = remaining items exactly; `pub trait FromIterator<A> { fn from_iter(iter: impl Iterator<Item = A>) -> Self; }`. Consumer set (closed, std::core::iter): fold, for_each, count, nth, last, collect, position, find, all, any, sum (T: Zero + Add), min, max (T: Ord). Combinator set (closed): map, filter, filter_map, enumerate, rev (DoubleEndedIterator only), take, skip, take_while, skip_while, chain, zip. Laziness law: combinators run no user code at construction; take(n) consumes exactly n + 1 next calls at most. Adapters allocate nothing (row ∅); collect allocates only into an allocating container. Non-terminating iterators are defined divergence (Volume 07 §7.6 d9).

**d12 — Numeric traits.** Closed trait set: `Zero { fn zero() -> Self; fn is_zero(&self) -> bool; }`; `One { fn one() -> Self; }`; `Signed: Num { fn abs(&self) -> Self; fn is_negative(&self) -> bool; }`; `Num: PartialEq + Zero + One` (marker; arithmetic comes from the language operator traits, Volume 02 §2.4); `Int: Num` carrying the associated consts BITS: u32, MIN: Self, MAX: Self and the method set of §19.6 d1; `Float: Num` carrying MAX, MIN, MIN_POSITIVE, EPSILON, INFINITY, NEG_INFINITY, NAN as associated consts and the method set of §19.6 d2. Float constant values are the exact IEEE-754-2019 binary32/binary64 constants (EPSILON = 2^−23 for f32, 2^−52 for f64). Implemented by exactly the 12 integer and 2 float primitives (matrix completeness, OB-19-15).

**d13 — Error interfaces.** `pub trait Error: DebugFormat { fn code(&self) -> u32; fn message(&self) -> &'static str; }`. code is stable within the defining type, assigned at definition, and listed in the type's CEP:FAILURE block; codes are unique within the defining type. message is a static string; no allocation. Every std error enum implements Error and is Send + Sync + 'static (so errors cross threads and satisfy 'static spawn bounds). Errors are values, never panics (§19.1.4).

**d14 — Option.** `pub enum Option<T> { None, Some(T) }`, declared `#[repr(niche)]`. Exact size rules, first match wins: (a) T has a null niche (reference, Box, Rc, Arc, fn pointer, raw pointer, NonNull) → size_of(Option<T>) = size_of(T), align_of = align_of(T); (b) T has an unused all-zero value in its domain (NonZeroU8..NonZeroU64, NonNull, the uninhabited !) → same rule; (c) otherwise size_of(Option<T>) = size_of(T) plus the smallest unsigned integer able to hold one discriminant bit, padded to align_of(T); discriminant storage reuses layout padding where available (layout math Volume 05 §7). Option<T> is Copy iff T: Copy. Closed method set: is_some, is_none, unwrap (panic site; payload "unwrap on None"), expect(msg: &str) (panic payload msg), unwrap_or, unwrap_or_else, map, map_or, and_then, or_else, ok_or(err) → Result, as_ref, as_mut, take, replace, iter (yields 0 or 1 item). Option<!> is uninhabited with size 0 (rule (b): ! contributes no values).

**d15 — Result and the never type.** `pub enum Result<T, E> { Ok(T), Err(E) }`, declared `#[repr(P)]` (no niche guarantee in edition P1): size_of = max(size_of(T), size_of(E)) plus discriminant, padded to max alignment. Closed method set: is_ok, is_err, unwrap (panic site; payload "unwrap on Err" plus e's DebugFormat), unwrap_err (panic site "unwrap_err on Ok"), expect, ok, err, map, map_err, and_then, or_else, unwrap_or, unwrap_or_else, as_ref, as_mut, iter. The never type `!`: uninhabited; size_of::<!>() = 0, align_of::<!>() = 1; a value of type ! can exist only as a diverging expression (Volume 07 §7.6 d9: loop without break, non-returning calls, panic/trap paths); ! coerces to every type (defined coercion, Volume 04 §4.3); a match on a !-typed value requires zero arms; Result<T, !> has size_of(T) and proves infallibility; functions returning ! carry no panic atom from the return itself.

**d16 — Range types.** The complete set: `Range<Idx> { start, end }` (half-open [start, end)), `RangeInclusive<Idx> { start, end }`, `RangeFrom<Idx> { start }`, `RangeTo<Idx> { end }`, `RangeToInclusive<Idx> { end }`, `RangeFull`. Range is empty iff start ≥ end; RangeInclusive is empty iff start > end. Slice indexing with ranges requires i ≤ j ≤ len; a reversed range or j > len at runtime is a panic condition per Volume 07 §7.6 d8 (trap reason 4/3 in abort builds). Range<Idx> is an iterator where Idx: Step; the closed Step impl set is {u8..u64, i8..i64, usize, isize, char}. RangeFrom iterates without bound: defined divergence. Rows ∅.

**d17 — Phantom types.** `pub struct PhantomData<T: ?Sized>;` — zero-sized (size 0, align 1, no drop glue); participates in variance and auto-trait derivation exactly as if it held T (Volume 04 §4.8, Volume 08 §8.3 d2). PhantomPinned is PhantomData<*mut ()> with the Unpin opt-out (Volume 06 §6.10 d3). Uses: typestate encoding and lifetime binding. Row ∅.

**d18 — Manually managed storage.** `pub struct ManuallyDrop<T>` — #[repr(transparent)]: layout identical to T; the value's drop glue does not run when the wrapper drops. Methods: new(v) (row = the move's row); Deref/DerefMut to T; unsafe fn take(&mut self) -> T (moves the value out; precondition: take is called at most once per wrapper; violation = UB-1 reading the vacated slot); unsafe fn drop(this: ManuallyDrop<T>) (runs the inner drop glue exactly once). Ownership: the wrapper owns the value; the borrow checker sees one owned place.

## 19.3 Memory (design.md §26.2)

**d1 — Box-like owned pointers.** `pub struct Box<T: ?Sized>` (edition P1 exposes Box<T> with the global allocator only; a parameterized Box<T, A: Allocator> exists for custom allocators). Box::new(v): allocates size_of(T) at align_of(T) via ABI4 planck_alloc, moves v in; row {alloc}. Box is Send + Sync iff T: Send + Sync. Box::into_raw / unsafe Box::from_raw (from_raw precondition: the pointer came from into_raw or Box::new with matching layout; violation = UB-2). Box::pin(v) is sanctioned pinning (Volume 06 §6.10 d2). Deref/DerefMut to T. Drop deallocates exactly one block through planck_dealloc(size_of(T), align_of(T)).

**d2 — Reference-counted pointers.** `pub struct Rc<T: ?Sized>` — non-atomic strong count u64, weak count u64, T. Rc::new(v) {alloc}; Rc::clone: non-atomic increment, row ∅; count overflow at 2^64 − 1 is trap reason 9 (Volume 06 §6.2 d3). Rc is neither Send nor Sync (Volume 08 §8.3 d2). API: strong_count, weak_count, downgrade → Weak<T>, Weak::upgrade → Option<Rc<T>> (None iff strong = 0), get_mut → Option<&mut T> (Some iff strong = 1 ∧ weak = 0), Rc::pin (sanctioned, Volume 06 §6.10 d2). Deallocation: T's drops run when strong reaches 0; the weak allocation frees when both counts are 0.

**d3 — Thread-safe reference counting.** `pub struct Arc<T: ?Sized>` — same shape with atomic counts: clone = fetch_add(Relaxed); strong-drop = fetch_sub(Release) with an Acquire fence on reaching 0 (release/acquire pair = sw source per Volume 08 §8.1.2 SW-R). Arc is Send + Sync iff T: Send + Sync (Volume 08 §8.3 d2). Count overflow = trap reason 9. Arc::clone row {concurrency}. Arc::make_mut(&mut self) → &mut T: if strong > 1 or weak > 0, clones T (requires T: Clone, {alloc}) and rebinds self.

**d4 — Non-null pointers.** `pub struct NonNull<T: ?Sized>` — wraps *mut T with the niche "never 0"; neither Send nor Sync (Volume 08 §8.3 d2). unsafe fn new_unchecked(p) (precondition p ≠ 0); as_ptr; unsafe deref/deref_mut. Option<NonNull<T>> is pointer-sized (§19.2 d14 rule (b)). No allocation.

**d5 — Maybe-uninitialized storage.** `pub union MaybeUninit<T>` — closed method set: uninit() (no read), new(v) (initialized), write(v) (overwrites without dropping the old value; the caller tracks initialization), assume_init (unsafe; precondition: initialized; violation = UB-1), assume_init_ref, assume_init_mut (both unsafe), zeroed() (safe; produces all-zero storage). Reading zeroed storage through assume_init is safe only for T: ValidZero (auto trait over integer, float, bool, raw-pointer types and arrays/tuples of them); otherwise UB-1. MaybeUninit is never Copy.

**d6 — Manually dropped values.** `pub fn mem::drop<T>(v: T)` takes ownership and runs drop glue immediately (row = T's drop row); `pub fn mem::forget<T>(v: T)` takes ownership and runs nothing (leaks are defined behavior; memory reclaims at process exit; leak accounting is FV-2 tooling, Volume 22). Also: mem::size_of<T>() -> usize, mem::align_of<T>() -> usize, mem::size_of_val(&T) -> usize (const-evaluable for sized T, Volume 07 §7.6 d10).

**d7 — Pinning.** `pub struct Pin<P: Deref>` — semantics exactly Volume 06 §6.10: Pin::new (requires T: Unpin), unsafe Pin::new_unchecked, get_ref, unsafe get_mut (T: Unpin, or the caller guarantees no move), unsafe into_inner_unchecked. DerefMut for Pin<P> exists only when P::Target: Unpin (§6.10 d4). Sanctioned storages: Box::pin, pin!, Rc::pin, Arc::pin (d1, d2, d3). Stdlib provides no generic projection helper in edition P1.

**d8 — Allocator interfaces.** `pub struct Layout { size: usize, align: usize }` with align a power of two in 1..=4096 (Layout::from_size_align returns Err otherwise); `pub enum AllocError { OutOfMemory, InvalidLayout }`; `pub unsafe trait Allocator { fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>; fn deallocate(&self, ptr: NonNull<u8>, layout: Layout); fn grow(&self, ptr: NonNull<u8>, old: Layout, new: Layout) -> Result<NonNull<[u8]>, AllocError>; }` — unsafe because implementors must guarantee validity, non-aliasing, and size preservation. `pub struct Global` implements Allocator by forwarding to the GlobalAllocator in effect (Volume 18 MEM1/MEM7) through ABI4. allocate(0) returns a valid empty slice (dangling non-null); contents are uninitialized (reads = UB-1 until written).

**d9 — Arena interfaces.** `pub struct Arena<T>` — bump allocator over chained blocks of ARENA_BLOCK_BYTES = 65,536 bytes (exact). alloc(v: T) → &mut T places T at the bump pointer and advances by size_of(T) rounded up to align_of(T); amortized O(1), never moves existing elements (a new block is started when the current is full). No individual deallocation; all element drops run at Arena::drop in reverse allocation order. total memory = ⌈Σ rounded sizes⌉ rounded up to whole blocks. Arena is Send iff T: Send, never Sync.

**d10 — Region interfaces.** `pub struct Region` — scoped resettable memory: Region::alloc(v) → &mut T valid for the region's scope (region-lexical binding per Volume 06 region rules; the Region must not outlive its scope); Region::reset() runs all element drops in reverse allocation order and resets the bump pointer, reusing the current block (O(n) in elements). Uses ARENA_BLOCK_BYTES blocks. Distinction from Arena: Arena grows until container drop; Region is resettable in place. Row {alloc} on first block acquisition.

**d11 — Raw allocation APIs.** `pub mod mem::raw`, all unsafe, all forwarding to ABI4 (Volume 18 §18.7), rows {alloc, unsafe}: alloc(layout) -> *mut u8 (null on failure — the raw form does not trap; callers must check); dealloc(ptr, layout); realloc(ptr, old: Layout, new_size: usize) -> *mut u8 (may move; contents preserved up to min(old.size, new_size)); alloc_zeroed(layout) -> *mut u8. Preconditions: layout valid; ptr valid with matching layout for dealloc/realloc. Violation = UB-2. These are the only raw entry points; safe std code must not call ABI4 symbols directly.

## 19.4 Collections (design.md §26.3)

Complexity notation: n = current element count; m = argument length; all bounds are worst case unless "amortized"; amortization is per the stated resize policy.

**d1 — Dynamic arrays.** `pub struct Vec<T>` — contiguous. Growth exact: on push when len = cap, new cap = max(4, 2 × cap) elements; cap never shrinks automatically; shrink_to_fit() reallocates to len (no-op when cap = len). Ops: push amortized O(1); pop O(1); insert(i, v) and remove(i) O(n); get(i) → Option<&T> O(1); index operator O(1) (out of bounds = panic condition per Volume 07 §7.6 d8; trap reason 3 in abort builds); len O(1); as_slice; iter, iter_mut; extend_from_slice O(m); truncate(n); clear (drops all, keeps cap); reserve(additional) grows to ≥ len + additional by the doubling formula; swap_remove(i) O(1). Row {alloc} on growth ops. Vec is Send + Sync iff T: Send + Sync.

**d2 — Fixed arrays.** [T; N] is a language primitive (Volume 05). Std additions: map (elementwise, row ∅), as_slice, as_mut_slice, TryFrom<&[T]> (Err(ArrayLengthError) iff lengths differ; ArrayLengthError is a complete one-variant enum), Copy iff T: Copy. Row ∅.

**d3 — Slices.** &[T], &mut [T], [T] are language primitives. Closed std method set: len, is_empty, first, last, get(i), get_mut(i), split_at(mid) → (&[T], &[T]) (panic site "split_at out of bounds" when mid > len), chunks(n) (panic site "chunks: zero chunk size" when n = 0), windows(n), contains, binary_search(&x) → Result<usize, usize> (T: Ord; O(log n); Ok = index of an equal element (the leftmost index on the search path), Err = insertion point), sort (stable merge sort, O(n log n) worst case, allocates ⌊n/2⌋ elements of temporary storage {alloc}; stable = equal elements keep order, G14), reverse O(n), fill(v) (T: Clone), copy_from_slice (panic site "copy_from_slice length mismatch"), swap(i, j) (out of bounds = panic condition).

**d4 — Strings.** `pub struct String` — owned growable UTF-8 byte buffer; invariant: contents valid UTF-8 at every safe-surface point. Growth = the Vec rule (d1). Ops: push_str(&str) O(m); push(char) O(1..4); insert(i, char) O(n) with panic site "insert not on char boundary" (checked form try_insert(i, char) → Result<(), TextError>); truncate(i) (panic site on non-boundary); pop → Option<char>; clear; capacity; as_str; into_bytes. Row {alloc} on growth. Send + Sync.

**d5 — Byte buffers.** `pub struct ByteBuffer` — Vec<u8> with byte-oriented additions: extend_from_slice, Write impl (§19.7 d9), into_vec, as_slice. Growth = the Vec rule. Row {alloc} on growth.

**d6 — Hash maps.** `pub struct HashMap<K, V, H = PlanckHasher>` — open addressing, linear probing, power-of-two capacity ≥ 8, load bound 7/8 (resize when occupied × 8 > capacity × 7): on crossing, new cap = 2 × capacity and all entries reinsert (O(n), amortized into insert). Bucket = hash & (cap − 1). Ops: insert → Option<V> (old), get/get_mut expected amortized O(1), worst case O(n) (long probe chains; refinement gap named FV-1, OB-19-6 bounds termination), remove O(cap) worst case (backward-shift deletion keeps probe chains valid), len, contains_key, entry(k) → Entry {Occupied, Vacant} with or_insert, iter (ascending bucket index), keys, values, capacity, with_hasher. No automatic shrink. K: Eq + Hash. Row {alloc} on growth. Determinism per §19.1.7 (OB-19-7).

**d7 — Ordered maps.** `pub struct BTreeMap<K: Ord, V>` — B-tree with exact parameters: ORDER_KEYS = 15 keys per node maximum, ORDER_KIDS = 16 children maximum, split on inserting into a full node moves the median key up; root may hold 1 key; removal rebalances by sibling borrow or merge. Ops: get/get_mut/insert/remove O(log n) (depth ⌈log₁₆ n⌉); first/last O(log n); len; range(bounds: impl RangeBounds<K>) → iterator over the sub-range, O(log n + m); iter ascending (in-order). Structure is a pure function of the operation sequence (no randomness; OB-19-11).

**d8 — Hash sets.** `pub struct HashSet<T, H = PlanckHasher>` = HashMap<T, ()>: insert → bool (true iff absent), remove → bool, contains → bool, iter, len, capacity, with_hasher. No set-algebra combinators in edition P1 (settlement; ecosystem interface). Same complexity, resize, and hasher contract as d6.

**d9 — Ordered sets.** `pub struct BTreeSet<T: Ord>` = BTreeMap<T, ()>: insert → bool, remove → bool, contains, iter ascending, range, first, last, len. Same contract as d7.

**d10 — Linked lists.** `pub struct LinkedList<T>` — doubly-linked heap nodes. Ops: push_front, push_back, pop_front, pop_back, front, back O(1); len O(1); iter, iter_mut O(n) total; cursor_front() with move_next, current → Option<&mut T>, remove_current → Option<T> O(1) at the cursor. Node allocation per push: {alloc}. Not cache-friendly (documented; complexity honest). Send + Sync iff T: Send + Sync.

**d11 — Intrusive lists.** `pub struct Links<T> { next: Option<NonNull<T>>, prev: Option<NonNull<T>> }` embedded in the element type; `pub struct IntrusiveList<T>` holds head/tail as Option<NonNull<T>>. All methods are unsafe (they operate on caller-stable storage): push_front(&mut T), push_back(&mut T), pop_front → Option<&mut T>, pop_back → Option<&mut T>, remove(elem: &mut T). All O(1), no allocation (row ∅ + unsafe). Documented invariant: an element is in at most one intrusive list at a time and stays address-stable while linked; violation = UB-2. No safe wrapper in edition P1.

**d12 — Deques.** `pub struct VecDeque<T>` — single ring buffer; head/tail indices; index arithmetic idx(i) = (head + i) mod cap (OB-19-8); growth = the Vec rule (d1). Ops: push_front, push_back, pop_front, pop_back amortized O(1); front, back, get O(1); insert(i, v), remove(i) O(min(i, n − i)); rotate_right(n) O(n); len; as_slices; iter, iter_mut. Row {alloc} on growth.

**d13 — Ring buffers.** `pub struct RingBuffer<T, const CAP: usize>` — fixed capacity; CAP ≥ 1 required (CAP = 0 is compile-time rejection P19-RNG). Count-based occupancy. Ops: push(v) → Result<(), BufferFull> O(1); pop → Option<T> O(1); peek → Option<&T>; len; is_full; is_empty. Wrap arithmetic (head + i) mod CAP. No allocation ever (row ∅). Send + Sync iff T: Send + Sync.

**d14 — Binary heaps.** `pub struct BinaryHeap<T: Ord>` — max-heap in a Vec; children of i at 2i + 1 and 2i + 2; invariant parent ≥ children under cmp (OB-19-10). Ops: push O(log n) (sift-up); pop → Option<T> O(log n) (sift-down swaps with the larger child); peek O(1); len; into_sorted_vec → Vec<T> ascending, O(n log n), {alloc}. Not stable (documented). Row {alloc} on growth.

**d15 — Trees.** `pub struct Tree<T>` over `Arena<Node<T>>` (§19.3 d9): node = {value: T, parent: Option<NodeId>, children: Vec<NodeId>}; NodeId = opaque u32 arena index. Ops: new; root → Option<NodeId>; insert_child(parent, v) → NodeId amortized O(1) (Vec push); remove_subtree(node) O(subtree size) (drops values; freed arena slots are not reused until Tree::drop — documented); children(node) → &[NodeId] O(1); value(node) → &T (panic site "invalid node id" on a removed or never-issued NodeId); iter_preorder O(n). No ordering contract on children (general unsorted trees).

**d16 — Bitsets.** `pub struct BitVec` — growable bit vector; word = u64; bit i lives in word i / 64 at bit position i % 64 (little-endian bit order within a word, exact). Ops: set(i, bool), get(i) O(1) (out of range = panic site "bit index out of range"); push(bool) amortized O(1); len (bits); count_ones O(n / 64); bitwise and/or/xor/not with equal-length operands (panic site "BitVec length mismatch"); capacity growth = the Vec rule at word granularity (max(4, 2 × words)). Row {alloc} on growth.

**d17 — Specialized numeric containers.** `pub struct AlignedVec<T, const A: usize>` — Vec with allocation alignment A; A ∈ {1, 2, 4, 8, 16, 32, 64} and A ≥ align_of::<T>() required, else compile-time rejection P19-ALN; serves simd_load_aligned (Volume 10 §10.7). `pub struct SmallVec<T, const N: usize>` — inline storage [MaybeUninit<T>; N]; spills once to a heap buffer when len > N (all elements move once, {alloc}); push/pop amortized O(1); iteration identical to Vec. Both Send + Sync iff T: Send + Sync.

## 19.5 Text and formatting (design.md §26.4)

**d1 — UTF-8 strings.** `pub struct str` (unsized) — a UTF-8 byte sequence; invariant: valid UTF-8 (scalar values U+0000..=U+10FFFF excluding surrogates U+D800..=U+DFFF, per Volume 02 §2.2). len = byte length. is_char_boundary(i): i = 0, or i = len, or byte[i] is not a UTF-8 continuation byte (0b10xxxxxx). Slicing s[i..j] requires is_char_boundary(i) ∧ is_char_boundary(j) ∧ i ≤ j ≤ len; the checked forms slice_checked(i, j) and get_checked(i, j) → Result<&str, TextError>; the unchecked operator forms are panic conditions per Volume 07 §7.6 d8 (payloads "char boundary violated", "index out of bounds"; trap reasons 4/3 in abort builds). Closed method set: chars() (O(1) per char, incremental decode), char_indices, bytes, split_whitespace (exactly U+0009, U+000A, U+000D, U+0020), lines (splits on \n, strips one trailing \r), starts_with, ends_with, contains, find, replace(&str, &str) → String ({alloc}), trim, trim_start, trim_end, split(sep). TextError = {InvalidUtf8 { offset: usize }, NotCharBoundary { index: usize }} — complete (allocation-limit failures are trap reason 5, not TextError).

**d2 — String builders.** `pub struct StringBuilder` — String with explicit build API: append(&str), append_char(char), append_fmt (§19.2 d8 sink), finish() → String (moves out). Growth = the Vec rule; no intermediate copies beyond growth. Row {alloc} on growth.

**d3 — Character operations.** char methods (closed set): is_ascii and the exact ASCII predicate set of SPEC-INDEX §5.10 (is_ascii_digit, is_ascii_hexdigit, is_ascii_whitespace over the four exact chars, is_ascii_alphabetic, is_ascii_alphanumeric, is_ascii_lowercase, is_ascii_uppercase, is_ascii_punctuation, is_ascii_graphic, is_ascii_control); to_ascii_lowercase, to_ascii_uppercase (ASCII-only mapping; full Unicode case mapping is banned in P1, d4); as_u32; from_u32(u32) → Option<char> (None for surrogates and > U+10FFFF); len_utf8() ∈ {1, 2, 3, 4}; encode_utf8(&mut [u8; 4]) → &str (the encoded sub-slice). char is Copy, Eq, Ord (by scalar value), Hash (feeds the UTF-8 encoding).

**d4 — Unicode operations.** Edition P1 provides exactly the closed set: char_count(&str) → usize (O(n)); decode_next(bytes: &[u8], i: usize) → Result<(char, usize), TextError> (scalar + bytes consumed); encode_scalar(char, out: &mut [u8; 4]) → usize (bytes written); is_scalar(u32) → bool; next_boundary(bytes: &[u8], i: usize) → Option<usize>; prev_boundary(bytes: &[u8], i: usize) → Option<usize>. Collation, normalization (NFC/NFD), case mapping, grapheme clustering, and width tables are banned from std in edition P1 and are the ecosystem interface against this closed set (normative scope settlement; adding one is an edition change with a Volume 24 security review). Scalar semantics exactly Volume 02 §2.2.

**d5 — Formatting.** The Format trait, grammar, and Formatter of §19.2 d8, applied to text: the entry point is `text::format(sink: &mut Formatter, fmt: &str, args: &[FormatArg]) → Result<(), FormatError>`, where the format! macro form (Volume 09) desugars to a call whose string is checked at compile time (P19-FMT-1..3) and whose runtime work is Formatter writes only. Cost documented: CEP-1 (banned in CEP-0, standards §5.1); O(1) per written character; allocation only when the sink grows.

**d6 — Parsing.** `pub trait FromStr: Sized { type Err; fn parse(s: &str) -> Result<Self, Self::Err>; }`. Std impls exactly: all integer types (radix 10; optional +/−; ParseError = {Invalid, Overflow, Empty}; digit grammar per Volume 02 §2.4 without radix prefixes); bool ("true"/"false" only, else Invalid); char (exactly one scalar, else Invalid); String (identity, infallible); f32/f64 (decimal grammar of Volume 02 §2.4 float literals including exponent; overflow to ±infinity is Ok per IEEE round-to-nearest; NaN spellings and empty input rejected → Invalid/Empty). Parsing never allocates (row ∅). from_str_radix(s, radix: u32) for integers accepts radix 2..=36 (digit letters case-insensitive), else Invalid.

**d7 — Numeric formatting.** Integers: to_str_radix(radix: u32) → String for radix ∈ {2, 8, 10, 16} ({alloc}); ASCII '-' for negatives; no grouping; prefixes only via the "#" spec of §19.2 d8 (0b/0o/0x). Floats: the empty-type (Display) form is the shortest round-trip decimal: the shortest digit string d (decimal point and optional e±dd exponent) such that parsing d as binary64 yields exactly the value; tie-break 1: the candidate nearest the true binary value; tie-break 2: last digit even. The precision spec .N = exactly N digits after the point (exact decimal expansion, round-half-even); type e/E without precision = 6 fractional digits; NaN prints "NaN"; infinities "inf"/"-inf". Round-trip correctness = OB-19-16.

**d8 — Text encoding.** Encoding (text → bytes) in edition P1 is UTF-8 only: str is UTF-8 by invariant, so `text::encode_utf8(s: &str) → &[u8]` is a borrow (row ∅) and String::into_bytes() → Vec<u8> moves the buffer ({alloc}-free move). Other encodings (Latin-1, UTF-16, and all others) are banned from std in P1 and are the ecosystem interface (same settlement as d4). Encoding a single char: char::encode_utf8 (d3).

**d9 — Text decoding.** `text::decode_utf8(bytes: &[u8]) → Result<&str, TextError>` — validates the whole buffer; InvalidUtf8 { offset } reports the first ill-formed byte index. The validator is total, O(n), and rejects exactly: overlong encodings, surrogates, values > U+10FFFF, and truncated sequences (OB-19-12). There is no lossy decoding in P1 (errors are explicit). String::from_utf8(Vec<u8>) → Result<String, FromUtf8Error>; FromUtf8Error returns the original Vec<u8> and the TextError.

**d10 — Byte-oriented APIs.** `pub mod text::bytes` (closed set): read_u8, read_u16le, read_u16be, read_u32le, read_u32be, read_u64le, read_u64be(slice: &[u8], i: usize) → Option<T> (None iff i + width > len); hex_encode(bytes: &[u8]) → String ({alloc}, lowercase); hex_decode(s: &str) → Option<ByteBuffer> (None on odd length or non-hex digit). ByteBuffer and the §19.4 d5 additions complete the byte surface.

## 19.6 Numeric library (design.md §26.5)

**d1 — Integer utilities.** Exact method set on all 12 integer primitives: {count_ones, count_zeros, leading_zeros, trailing_zeros, leading_ones, trailing_ones, rotate_left(u32), rotate_right(u32), swap_bytes, from_str_radix, div_euclid, rem_euclid, checked_add, checked_sub, checked_mul, checked_div, checked_rem, checked_shl, checked_shr, checked_neg, checked_abs, wrapping_add, wrapping_sub, wrapping_mul, wrapping_div, wrapping_rem, wrapping_shl, wrapping_shr, wrapping_neg, wrapping_abs, saturating_add, saturating_sub, saturating_mul, saturating_div, saturating_shl, saturating_shr, saturating_neg, saturating_abs, unchecked_add, unchecked_sub, unchecked_mul (all three unsafe intrinsics, UB-6 precondition "no overflow", Volume 07 §7.9 d1)}. div/rem by zero and MIN / −1 are panic conditions (Volume 07 §7.9 d14). All are const-evaluable within the const limits (Volume 07 §7.6 d10). Rows ∅; panic conditions add {panic} in unwind builds automatically (Volume 07 §7.2.1).

**d2 — Floating-point utilities.** Exact method set on f32/f64: {is_nan, is_infinite, is_finite, is_normal, is_subnormal, classify → FpCategory {Nan, Inf, Zero, Subnormal, Normal}, signum, copysign, abs, floor, ceil, trunc, round (half away from zero, exact), fract, total_cmp (IEEE-754-2019 §3.5 totalOrder: −NaN < −inf < … < −0 < +0 < … < +inf < +NaN; the unique total order), min, max (IEEE minNum/maxNum: a NaN operand yields the other operand), mul_add (fused, one rounding), recip, to_degrees, to_radians (factor DEG_PER_RAD = 57.29577951308232 (f64) and its f32-nearest value, computed in the operand's precision), next_up, next_down (IEEE nextUp/nextDown), to_bits, from_bits (bit-preserving; NaN payloads preserved)}. All follow IEEE-754-2019 binary32/binary64, round-to-nearest-ties-even default (Volume 07 §7.9 d15). −0.0: 0.0 == −0.0 is true; the sign is observable through copysign/to_bits.

**d3 — Mathematical functions.** Closed list with accuracy contracts (same names for f32/f64; ulp relative to the correctly rounded ideal): sqrt ≤ 0.5 ulp (IEEE-correctly-rounded); cbrt, exp, exp2, ln, log2, log10, hypot (no overflow for finite inputs) ≤ 1 ulp; powi(n: i32) exact (binary exponentiation, left-to-right multiplication order — defined); powf, sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh ≤ 2 ulp (asin/acos domain [−1, 1], else NaN). Implementations link libm through FFI (row {ffi}); the libm version is pinned at build (Volume 20 build config); results may differ between targets (platform row, §19.13 g10). Verification: EXH on argument boundary sets + PROP on sampled domains (OB-19-14 scope excludes these; see OB-19-17 companion in the std test suite), ulp measured against the reference implementation.

**d4 — SIMD abstractions.** Closed type set for both initial targets: {f32x4, f64x2, i8x16, i16x8, i32x4, i64x2, u8x16, u16x8, u32x4, u64x2} (x86-64 baseline SSE2; AArch64 baseline NEON). Closed operation set: splat(v), load(&[lane; N]), load_aligned (alignment precondition = size; violation = UB-6, Volume 10 §10.7), store, store_aligned, add, sub, mul (per-lane; float lanes per IEEE), and, or, xor (integer lanes, bitwise), extract<const LANE>(). Edition P1 provides no runtime feature dispatch in std (settles Volume 08 §8.5 d9's "may offer": P1 does not; both initial targets always have their baseline features; other targets are declared, not implemented — SPEC-INDEX §3). SIMD beyond this set: #[repr(simd)] structs per Volume 10 §10.7. Aligned storage: AlignedVec (§19.4 d17).

**d5 — Numeric conversions.** The TryFrom matrix, exact behavior table: int → wider int: Ok, lossless. int → narrower-or-equal int: range check; out of range → Err(ConversionError::Overflow); else Ok. int → float: always Ok; rounds to nearest-ties-even (precision loss documented, not an error). float → int: NaN or ±inf → Err(ConversionError::NanToInteger); truncated value out of range → Err(ConversionError::Overflow); else Ok (fraction discarded toward zero). f64 → f32: |x| > f32::MAX → Err(ConversionError::Overflow); NaN → Ok(NaN); else IEEE round-to-nearest. f32 → f64: always Ok, exact. ConversionError = {Overflow, NanToInteger} — complete. The as-cast operator keeps its language semantics (Volume 07 §7.9); these TryFrom impls are the checked surface. Matrix completeness and behavior = OB-19-15.

**d6 — Saturating arithmetic.** saturating_* family of d1 with exact laws: integer overflow clamps to [MIN, MAX] (signed: positive overflow → MAX, negative → MIN; unsigned: overflow → MAX, underflow → 0); saturating_div(MIN, −1) = MAX (defined); saturating_shl/shr with n ≥ BITS yield 0. Laws are OB-19-18's L3, verified over the stated domain.

**d7 — Wrapping arithmetic.** wrapping_* family of d1: the result is the mathematical result mod 2^N in two's complement; wrapping_shl/shr mask the shift amount n mod BITS. Law L2 of OB-19-18.

**d8 — Checked arithmetic.** checked_* family of d1: checked_add/sub/mul/neg/abs return Some(mathematical result) iff representable, None otherwise; checked_div/checked_rem return None iff the divisor is 0 or (MIN, −1); checked_shl/shr return None iff n ≥ BITS. Law L1 of OB-19-18. The three families together apply to all 12 integer primitives × the op names of d1 (matrix completeness in OB-19-15's domain statement). Floats have none of the three families (IEEE semantics govern).

**d9 — Big integers.** Edition P1: std::num provides no big-integer type — the bullet's "if provided" is settled: not provided. Big-integer computation is the ecosystem interface over the std numeric primitives; a future edition that adds std big integers must satisfy the full §19.13 guarantee table and enter by edition change. This row is the complete definition of the bullet.

## 19.7 IO (design.md §26.6)

All IO is available in hosted modes M3/M4 (Volume 18 §18.1); IO rows carry {io}. Static-dispatch policy (§19.1.3): every IO interface is a generic over the traits of d9; no dyn, no type erasure (standards §25.6).

**19.7.0 IoError.** `pub enum IoError` — complete variant set: {NotFound, PermissionDenied, ConnectionRefused, ConnectionReset, ConnectionAborted, NotConnected, AddrInUse, AddrNotAvailable, BrokenPipe, AlreadyExists, WouldBlock, InvalidInput, InvalidData, TimedOut, WriteZero, Interrupted, Unsupported, UnexpectedEof, DirNotEmpty, IsDir, NotADirectory, StorageFull, OutOfMemory, Unknown(u32)}. Exact errno mapping: ENOENT → NotFound; EACCES, EPERM → PermissionDenied; ECONNREFUSED → ConnectionRefused; ECONNRESET → ConnectionReset; ECONNABORTED → ConnectionAborted; ENOTCONN → NotConnected; EADDRINUSE → AddrInUse; EADDRNOTAVAIL → AddrNotAvailable; EPIPE → BrokenPipe; EEXIST → AlreadyExists; EAGAIN, EWOULDBLOCK, EINPROGRESS → WouldBlock; EINVAL → InvalidInput; EILSEQ → InvalidData; EIO → InvalidData; ETIMEDOUT → TimedOut; EINTR → Interrupted; EOPNOTSUPP, ENOSYS → Unsupported; ENOSPC, EFBIG → StorageFull; EOVERFLOW, ERANGE → StorageFull; ENOMEM → OutOfMemory; ENOTEMPTY → DirNotEmpty; EISDIR → IsDir; ENOTDIR → NotADirectory; every other errno value e → Unknown(e). A write call returning 0 on a non-empty buffer maps to WriteZero (library-detected). The mapping is total over the failure set of the Volume 18 §18.8 subset (OB-19-19).

**d1 — Standard input.** `pub fn io::stdin() → Stdin` — fd 0 (Volume 18 I6). Stdin is internally buffered (BufReader over StdinRaw, BUF_CAP = 8192) and Send + Sync (an internal Mutex guards the fd; {concurrency}). Methods: read, read_exact, read_line(&mut String) → Result<usize, IoError> ({alloc} on growth), bytes()? no — the closed set is {read, read_exact, read_line, lock() → StdinLock (MutexGuard-like)}. Unbuffered access: io::stdin_raw() → StdinRaw (d6).

**d2 — Standard output.** `pub fn io::stdout() → Stdout` — fd 1; buffered (BufWriter, BUF_CAP = 8192); no auto-flush on newline (explicit flush; process exit flushes per Volume 18 I9). Implements Write.

**d3 — Standard error.** `pub fn io::stderr() → Stderr` — fd 2; unbuffered (direct write syscalls per call). Implements Write.

**d4 — File operations.** `pub struct File`. Constructors: File::open(path: &str) → Result<File, IoError> (read-only); File::create(path) (write-only, create, truncate; mode bits 0o666 with umask applied by the OS). Methods: Read/Write impls (unbuffered, direct syscalls), Seek impl, metadata() → Result<Metadata, IoError> (fstat) with `Metadata { size: u64, kind: FileKind {File, Dir, Other}, permissions: u32, modified_unix_ns: i64 }`. Not provided in P1: fsync, ftruncate, fd duplication (absent from the Volume 18 subset; #[ffi] escape per §19.8 d10). Rows {io}.

**d5 — Buffered IO.** `pub struct BufReader<R: Read>` and `pub struct BufWriter<W: Write>` — default capacity BUF_CAP = 8192 bytes (exact); BufReader::with_capacity(cap: usize ≥ 1, inner) (cap = 0 → IoError::InvalidInput). BufReader drains its buffer before the inner read; read_exact fills across refills. BufWriter::flush empties the buffer; the Drop impl attempts flush and discards errors (documented: drops cannot report; use explicit flush when errors matter). Both Send iff the inner type is Send.

**d6 — Unbuffered IO.** The unbuffered surface is exactly: File (d4); StdinRaw/StdoutRaw/StderrRaw via io::stdin_raw()/stdout_raw()/stderr_raw() (zero-buffer Read/Write over the fd); and unsafe fn io::from_raw_fd(fd: i32) → File with precondition "the fd is owned exclusively by the caller" (double-close is a defined misuse producing EBADF errors, not UB). Rows {io}.

**d7 — Filesystem operations.** `pub mod io::fs` (closed set): metadata(path) → Result<Metadata, IoError> (stat); rename(from, to) → Result<(), IoError>; remove_file(path) (unlink); create_dir(path) (mkdir, mode bits 0o777 with umask; exists → AlreadyExists); remove_dir(path) (rmdir; non-empty → IoError::DirNotEmpty); current_dir() → Result<String, IoError> (getcwd; buffer grows by doubling from 256 bytes; ERANGE → IoError::StorageFull). No chdir in P1 (not in the subset; chdir is process-global — settlement). Rows {io}, {alloc} for current_dir's path buffer.

**d8 — Directory operations.** Edition P1 provides exactly create_dir, remove_dir, current_dir (d7). Directory listing is not provided: the Volume 18 §18.8 closed OS subset contains no directory-read entry point; adding one is an edition change. A program needing listing declares the entry itself via #[ffi] (Volume 10 §10.8) — this is the normative settlement of the bullet.

**d9 — Streams.** The stream traits, exact signatures (static dispatch only):

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>; // 0 = EOF
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError>;
    fn read_to_end(&mut self, out: &mut ByteBuffer) -> Result<usize, IoError>; // {alloc}
}
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError>;
    fn flush(&mut self) -> Result<(), IoError>;
}
pub trait Seek { fn seek(&mut self, pos: SeekFrom) -> Result<u64, IoError>; }
pub enum SeekFrom { Start(u64), Current(i64), End(i64) }
pub fn copy<R: Read, W: Write>(r: &mut R, w: &mut W) -> Result<u64, IoError>; // 8192-byte stack buffer, no alloc
```

Laws: read returns 0 ≤ n ≤ buf.len(), 0 only at EOF; write may write fewer bytes (callers use write_all); read_exact errors with UnexpectedEof when EOF precedes a full buffer; Seek resolution exact: Start(p) → p; Current(d) → cur + d (i64 arithmetic; overflow or negative → IoError::InvalidInput); End(d) → size + d (same rule). Implementors: File (Seek via lseek); the three raw std streams (no Seek); BufReader::seek discards the buffer and delegates; BufWriter::seek flushes first, then delegates. read_exact/read_to_end/write_all/flush are default methods over read/write (no dyn anywhere).

**d10 — Sockets.** Defined in §19.10 (the net module owns the socket surface); TcpStream implements Read and Write. This bullet is satisfied by §19.10 d1–d3.

**d11 — Asynchronous IO interfaces.** `pub mod io::async_io` — closed type set: {AsyncFile, AsyncStdin, AsyncStdout, AsyncStderr, AsyncTcpListener, AsyncTcpStream, AsyncUdpSocket}. Each exposes async fns 1:1 with its blocking counterpart, same name and signature; the read entry is AsyncFile::read(&mut self, buf: &mut [u8]) → Result<usize, IoError> as an async fn; rows {async, io, alloc}. Mechanism: each call submits the blocking operation to the runtime blocking pool (Volume 18, BLOCKING_POOL_MAX = 16 threads, OB-18-13) and awaits completion; pool exhaustion blocks the calling task at the await until a slot frees (defined; {nondet}); completion wakes the task through its Waker (Volume 08 §8.6 d7). There is no reactive/epoll layer in edition P1 (settlement: the pool model is the P1 contract; evented IO requires an edition change).

## 19.8 Operating system (design.md §26.7)

Every entry in §19.8 calls only the Volume 18 §18.8 closed OS subset; rows carry {io} or {concurrency} per entry; every syscall failure maps through IoError (§19.7.0).

**d1 — Processes.** `pub mod os::process`: `Command::new(argv0: &str)`; .arg(s: &str) accumulates argv; .spawn() → Result<Child, IoError> = fork + execve (argv from the accumulated args; execve only — no PATH search, argv[0] must be a path, documented); child environment is inherited (no env modification in P1, d2). Child = {pid: i32}; Child::wait(&mut self) → Result<ExitStatus, IoError> (waitpid; blocks; rows {io, concurrency}); `ExitStatus { exit_code: i32 }` with the convention: a signal-killed child yields exit_code = 128 + signal number (documented; siginfo is outside the subset). exec failure in the child = the child exits with code 127 (documented). Between fork and execve the child may perform only async-signal-safe operations; calling std allocation there is banned (review invariant). Not provided in P1: kill, pipes, fd redirection (absent from the subset; #[ffi] escape).

**d2 — Environment variables.** `pub mod os::env` (read-only in P1): get(name: &str) → Option<String> ({alloc} for the value copy; linear scan over the envp walk of Volume 18 I4); vars() → iterator of (String, String) ({alloc} per pair; order = envp order, deterministic, G14). Set and unset are banned in P1 (setenv/unsetenv not in the subset; the only P1 use case is child inheritance, which is inherent). This read-only settlement is normative.

**d3 — Filesystem.** Satisfied by §19.7 d7/d8 (io::fs); the bullet's content (metadata, rename, unlink, mkdir, rmdir, getcwd) is exactly the closed set there.

**d4 — Memory mapping.** `pub mod os::mmap`: map(len: u64, prot: Prot, flags: MapFlags) → Result<NonNull<u8>, IoError> — `pub struct Prot(u8)` with constants Prot::R, Prot::W, Prot::X, or-combinable; `pub struct MapFlags(u8)` with constants PRIVATE, SHARED, ANONYMOUS, or-combinable; the kernel chooses the address (no hint parameter in P1). unmap(ptr, len) → Result<(), IoError> (munmap; requires exactly the returned base and length, else InvalidInput); protect(ptr, len, prot) → Result<(), IoError> (mprotect; same exactness rule). Executable mappings are allowed; std enforces no W^X policy (Volume 24 governs). Rows {io}.

**d5 — Signals.** Edition P1 provides no user-facing signal API: the sigaction entry of the OS subset is runtime-internal (Volume 18 INS1); the runtime may install fatal-signal handlers solely to emit the `trap: <reason>` line and re-raise; user signal handlers are banned in P1; adding a signal API is an edition change. Signal-caused process death is a defined abnormal termination under d1's exit-status convention.

**d6 — Threads.** Satisfied by std::thread of §19.9 d1/d8 (thread abstraction and scoped threads); the OS-level contract (pthread_create/join, stack sizes, futex) is Volume 08 §8.2 and Volume 18 §18.8.

**d7 — System information.** `pub mod os::sys` (closed set): page_size() → u64 (sysconf _SC_PAGESIZE); clock_ticks_per_second() → u64 (_SC_CLK_TCK); available_parallelism() → usize (_SC_NPROCESSORS_ONLN; consumed by the executor clamp of Volume 08 §8.8 d3); pointer_width() → u64 (compile-time constant 64 on both initial targets). First call carries {io}; values are cached per process; later calls are row ∅.

**d8 — Timers.** `pub mod time`: `pub struct Duration { secs: u64, nanos: u32 }` with invariant nanos < 1_000_000_000 (OB-19-24); constructors from_secs, from_millis, from_nanos (exact split); ops checked_add, checked_sub (None on overflow or a < b). `pub struct Instant` — a CLOCK_MONOTONIC reading held as u64 nanoseconds; now() rows {io, nondet}; elapsed() → Duration; duration_since(earlier) → Option<Duration>; Instant is Copy and Ord by internal value; monotonicity of the comparison is the contract (OB-19-24). `pub fn thread::sleep(d: Duration)` — blocks the calling thread ≥ d; rows {io, concurrency, nondet}; implemented over the futex entry of the OS subset with timeout (FUTEX_WAIT with the deadline from CLOCK_MONOTONIC; the value-changed path loops until the deadline) — the implementation contract is REVIEW-checked to use no syscall outside the subset (OB-19-25). The permitted clock IDs are exactly {CLOCK_MONOTONIC}; there is no wall-clock API in P1 (settlement: wall time is nondeterministic and out of scope).

**d9 — Synchronization.** Satisfied by std::sync of §19.9 d2–d7 (mutex, read-write lock, condition variable, semaphore, barrier, atomics, channels); the memory-ordering contract is Volume 08 §8.1 (SC-Per-Location + release/acquire + seq-cst fences).

**d10 — Platform APIs.** The total platform surface of edition P1 is the Volume 18 §18.8 closed subset plus the bindings of this volume. Every other OS facility requires an explicit #[ffi] declaration by the program (Volume 10 §10.8; Volume 18 rule (e)); std provides no additional bindings and no catch-all platform module. This closed-surface rule is the normative settlement of the bullet.

## 19.9 Concurrency (design.md §26.8)

API surface of Volume 08's semantics; the memory model is Volume 08 §8.1 (SC-Per-Location + release/acquire + seq-cst fences); blocking primitives carry {concurrency} and {nondet} for timing.

**d1 — Thread abstraction.** `pub mod thread`: spawn<F, T>(f: F) → JoinHandle<T> where F: FnOnce() → T + Send + 'static, T: Send (row {concurrency}; default stack 2 MiB = 2,097,152 bytes; minimum 65,536 bytes; below minimum = C08-E001, Volume 08 §8.2 d9). current() → ThreadId (u64, from 1, monotonic, never reused, §8.2 d5). `JoinHandle<T>`: join(self) → Result<T, JoinError> with `pub enum JoinError { Panic }` (the OS-thread enum, §8.2 d6); is_finished() → bool; dropping the handle detaches (§8.2 d7). There is no thread::yield_now in P1 (no sched_yield in the OS subset; settlement — cooperative yielding exists only at the task level, d10).

**d2 — Mutex.** `pub struct Mutex<T>` — semantics exactly Volume 08 §8.4 d1 (poisoning: a panic unwinding while a guard lives poisons the mutex; every later lock returns Err(PoisonError<()>)). API: new(v); lock() → LockResult<MutexGuard<T>> with LockResult<T> = Result<T, PoisonError<()>>; try_lock() → Result<Option<MutexGuard<T>>, TryLockError> with TryLockError = {Poisoned} (Ok(Some(g)) acquired, Ok(None) contended, Err = poisoned); into_inner(self) → Result<T, PoisonError<T>>; get_mut(&mut self) → Result<&mut T, PoisonError<&mut T>> (no lock required under &mut exclusivity). MutexGuard<T>: Deref/DerefMut, unlock on drop, !Send. Implementation: the 3-state futex protocol (Volume 18 OB-18-5); acquisition is an sw source (§8.4 d1). Mutex<T> is Send + Sync iff T: Send.

**d3 — Read-write lock.** `pub struct RwLock<T>` — Volume 08 §8.4 d2: read() → LockResult<RwLockReadGuard<T>> (concurrent readers), write() → LockResult<RwLockWriteGuard<T>> (exclusive); try_read/try_write → Result<Option<…>, TryLockError {Poisoned}>; poisoning occurs only on write-guard panic (read-guard panics do not poison — normative per §8.4 d2); no fairness guarantee ({nondet}); reader count bounded only by the Volume 18 implementation limit. Guards mirror MutexGuard.

**d4 — Condition variable.** `pub struct Condvar` — Volume 08 §8.4 d3: wait<T>(guard: MutexGuard<T>) → LockResult<MutexGuard<T>> (atomically releases, blocks, reacquires); wait_timeout<T>(guard, d: Duration) → (LockResult<MutexGuard<T>>, bool) (bool = the timeout expired); notify_one(), notify_all() (rows {concurrency, nondet}). Spurious wakeups are permitted; callers must recheck predicates (normative). Implemented over futex; wait_timeout's deadline comes from CLOCK_MONOTONIC.

**d5 — Semaphore and Barrier.** `pub struct Semaphore` — Volume 08 §8.4 d4: new(n: u64); acquire() → Permit (blocks at count 0); try_acquire() → Option<Permit>; Permit is Send (release may happen on another thread); the count invariant 0 ≤ count ≤ n holds at every step (OB-19-20). `pub struct Barrier` — Volume 08 §8.4 d5: new(n: usize ≥ 1); wait(&self) → BarrierWaitResult { is_leader: bool } (the n-th arrival is the leader); reusable indefinitely. Implemented over Mutex + Condvar ({concurrency}; {alloc} at construction).

**d6 — Atomic types.** `pub mod sync::atomic` — the complete type set of Volume 08 §8.5 d2: AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicIsize, AtomicUsize, AtomicPtr<T> (AtomicIsize/AtomicUsize are 8 bytes on both initial targets; AtomicI128/AtomicU128 only on x86-64 with cmpxchg16b, C08-E002 elsewhere). Method set per §8.5 d3: load, store, swap, compare_exchange, compare_exchange_weak, fetch_add/sub/and/or/xor/max/min (integers; AtomicBool: fetch_and/or/xor logical). Ordering enum exactly {Relaxed, Acquire, Release, AcqRel, SeqCst} (§8.4 d11); failure-ordering rule §8.5 d4 (C08-E006); alignment = size (§8.5 d8). fence(order) is re-exposed over the Volume 10 intrinsic. Rows {concurrency}; compare_exchange_weak adds {nondet} (spurious failure).

**d7 — Channels.** `pub mod sync::mpsc` — Volume 08 §8.4 d6: channel<T>() → (Sender<T>, Receiver<T>) unbounded (send never blocks; {alloc} per message); sync_channel<T>(cap: u64) → (SyncSender<T>, Receiver<T>) bounded (send blocks while len = cap). Sender<T>: Clone (T: Send); SyncSender<T>: !Clone (settlement); send(v: T) → Result<(), SendError<T>> (SendError returns the value — no loss, §8.4 d6); Receiver<T>: recv() → Result<T, RecvError>, try_recv() → Result<T, TryRecvError {Empty, Disconnected}>; the channel closes when all senders drop and the queue drains; recv then returns Err(RecvError); Receiver implements Iterator<Item = T> (ends on Disconnected). Message handoff is a release/acquire sw pair (§8.4 d6). Receiver is !Send? No — Receiver: Send (single consumer may live on another thread); both handle types require T: Send.

**d8 — Scoped threads.** thread::scope<'env, F, R>(f: F) → R where F: FnOnce(&Scope<'env>) → R — Volume 08 §8.2 d1/d10: Scope::spawn(f) requires f: FnOnce() → T + Send + 'scope (borrows of 'env permitted); the scope joins every handle before returning (join order = spawn order, deterministic, G14); a scoped thread's panic is surfaced: if any join is Err(JoinError::Panic), scope() panics with payload "scoped thread panicked" after all joins complete (never swallowed, §8.2 d10).

**d9 — Task interfaces.** `pub mod task`: `pub trait Future { type Output; fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output>; }` — the exact Volume 08 §8.6 d1 shape; `pub enum Poll<T> { Ready(T), Pending }`; `pub struct Context<'a> { waker: &'a Waker }`. `pub struct Waker` with wake(self), wake_by_ref(&self); Waker: Send + Sync (§8.6 d7); wake runs no user code beyond the registered action; waking after completion is a no-op. Construction: Waker::noop() (no-op waker for infrastructure and tests) and waker_fn<F: Fn() + Send + Sync + 'static>(f: F) → Waker ({alloc} at construction; each wake call runs f exactly once — coalescing is the executor's business, §8.8 d7). No user-visible vtable form in P1 (keeps the trait-object ban, §19.1.3).

**d10 — Executor interfaces.** `pub mod exec`, exact Volume 08 §8.8 d1 shape:

```rust
pub trait Executor {
    fn spawn<F>(&self, fut: F) -> exec::JoinHandle<F::Output>
      where F: Future + Send + 'static, F::Output: Send + 'static;
    fn block_on<F: Future>(&self, fut: F) -> F::Output;
}
```

exec::JoinHandle<T> implements Future<Output = Result<T, JoinError>> with the async `pub enum JoinError { Panic, Cancelled }` (distinct type from thread::JoinError, §8.8 d1); dropping a JoinHandle requests cancellation (§8.6 d11). spawn uses default priority 128; spawn_with(fut, priority: u8) sets it (0 = highest, §8.8 d9). Std executors, closed set of exactly two: `exec::CurrentThread` (one worker; ready queue FIFO by (priority, sequence); priorities honored exactly; rows {alloc}; §8.8 d2) and `exec::MultiThread` (W = clamp(available_parallelism, 1, 256), or with_workers(n: usize) → clamp(n, 1, 256); work stealing: steal ⌈len/2⌉ FIFO from a victim; local deque bound 1024 with global overflow, §8.8 d4/d5 and the Volume 12 limits table; rows {concurrency, alloc, nondet}). Combinators (§8.6 d9, poll-only): exec::join(f1, f2) → Join2<F1, F2> (polls children to completion; cancels losers only on parent cancellation; child wakes schedule the parent) and exec::race(f1, f2) → Race2<F1, F2> (first Ready wins; losers dropped = cooperative cancellation, §8.6 d11). Arity 2 only in P1 (closed; no join_all — settlement, keeps static dispatch).

## 19.10 Networking (design.md §26.9)

The net group links only with the net feature (Volume 18 rule (b); Volume 17 L14). Edition P1 supports IPv4 only (settlement; every entry point rejects IPv6 with IoError::Unsupported).

**d1 — TCP.** `pub mod net::tcp`: TcpListener::bind(addr: SocketAddr) → Result<TcpListener, IoError> (socket, bind, listen; backlog LISTEN_BACKLOG = 128, exact); accept() → Result<(TcpStream, SocketAddr), IoError> (rows {io, nondet}); TcpStream::connect(addr) → Result<TcpStream, IoError> (blocking connect). TcpStream implements Read, Write, and shutdown(Shutdown {Read, Write, Both}) → Result<(), IoError>; options: set_nodelay(bool)/nodelay() over setsockopt/getsockopt. The closed std option set is exactly {TCP_NODELAY}; other options require #[ffi] (settlement). Rows {io, nondet}.

**d2 — UDP.** `pub struct UdpSocket`: bind(addr) → Result<UdpSocket, IoError>; send_to(buf: &[u8], addr) → Result<usize, IoError> (sendto); recv_from(buf: &mut [u8]) → Result<(usize, SocketAddr), IoError> (recvfrom; bytes beyond buf.len() are discarded per POSIX — documented); connect(addr) restricts the peer; send/recv in connected mode; send_to after connect overrides the peer per datagram (documented). Rows {io, nondet}.

**d3 — Socket interfaces.** The socket surface is exactly {TcpListener, TcpStream, UdpSocket} plus the option methods of d1/d2. Raw sockets, Unix-domain sockets, and all other families are not in P1 (#[ffi] escape per §19.8 d10 — settlement). All three types are Send + Sync (fd-level sharing is race-free under the type's internal locking where required).

**d4 — Address parsing.** `pub struct SocketAddr { ip: Ipv4Addr, port: u16 }`; `pub struct Ipv4Addr([u8; 4])`. FromStr grammar, exact: dotted quad a.b.c.d, each octet 1..3 decimal digits ∈ 0..=255, leading zeros rejected; `pub enum AddressError { InvalidOctet, TooFewParts, TooManyParts, NonNumeric, PortOutOfRange }` — complete. SocketAddr parses "a.b.c.d:port" (port 0..=65535). Display: dotted quad, no leading zeros, then ":port".

**d5 — DNS integration.** `pub fn net::dns::resolve(host: &str) → Result<Ipv4Addr, DnsError>` — getaddrinfo (AF_INET, SOCK_STREAM hints, NULL service) + freeaddrinfo; takes the first returned address (getaddrinfo order; the OS resolver may be nondeterministic, rows {io, nondet}); `pub enum DnsError { TryAgain, NoData, Fail, InvalidName, UnsupportedFamily }` (EAI_AGAIN → TryAgain, EAI_NODATA → NoData, EAI_FAIL → Fail, EAI_NONAME → InvalidName, EAI_FAMILY → UnsupportedFamily). No caching in std: every resolve is a fresh lookup (settlement).

**d6 — Networking protocols.** Edition P1 std provides no protocol libraries (no HTTP, TLS, WebSocket, or DNS wire protocol beyond getaddrinfo); protocols are ecosystem crates over the d1–d5 surface. Adding a protocol to std requires an edition change and a Volume 24 security review. This settlement is the complete definition of the bullet.

**d7 — Async networking.** The wrappers AsyncTcpListener, AsyncTcpStream, AsyncUdpSocket (§19.7 d11) expose async accept, connect, send, recv, send_to, recv_from 1:1 with d1/d2 over the blocking pool (BLOCKING_POOL_MAX = 16, OB-18-13). Rows {async, io, alloc, nondet}.

**d8 — Buffer management.** Net APIs allocate no internal buffers: send/recv take caller buffers and pass them to the syscalls (Law 1: one copy at the syscall boundary only). The documented exceptions: the async wrappers copy the caller buffer into a pool-owned allocation of exactly buf.len() bytes per call ({alloc}; one extra copy per async operation — documented cost, Law 1 compliant); the listener backlog is kernel-owned. No other net-side storage exists.

## 19.11 Testing and diagnostics (design.md §26.10) — CEP-2

**d1 — Assertions.** `pub mod test`: assert(cond: bool) — panic site, payload "assertion failed" (row {panic}); assert_msg(cond: bool, msg: &str) (payload msg); assert_eq<T: PartialEq + DebugFormat>(a, b) and assert_ne (payloads include both values' DebugFormat); the language's assert! macro (Volume 09) desugars to test::assert_msg with the stringified expression when the compiler supplies expression text, else "assertion failed". Every assertion payload carries Location { file: &'static str, line: u32, col: u32 } from the call site (compiler-provided, Volume 13 metadata).

**d2 — Unit testing.** #[test] functions (attribute per Volume 10 §10.3; signatures fn name() or fn name() → Result<(), E: Error>). `plkc --test` compiles a harness binary that runs every #[test] sequentially in registration order (link order; deterministic, G14); each test runs inside a panic catch boundary (Volume 07 §7.11 d5): Ok = pass; Err(e) or a panic = fail with the payload formatted. Test code may allocate ({alloc}); class CEP-2.

**d3 — Test discovery.** Discovery is compile-time: the compiler collects the #[test] set during resolution (Volume 10 §10.4) into a generated table (name, function pointer, Location); no filesystem scanning and no runtime discovery; #[test] functions are unreachable outside the harness (dead-code elimination applies in non-test builds).

**d4 — Test reporting.** Output format exact: one line per test, `ok <name>` or `FAIL <name>: <payload>`; final line `passed: <p>, failed: <f>, total: <n>`; exit code 0 iff f = 0, else 1. All lines go to stdout, flushed per line (single stream, deterministic order, G14); stderr is reserved for harness-internal errors. Payload rendering uses DebugFormat (§19.2 d9) with the Location prefix "file:line:col: ".

**d5 — Benchmarking.** #[bench] fn name(b: &mut Bencher); b.iter(body: impl FnMut()). Protocol exact: 10 unmeasured warmup iterations, then 100 measured iterations timed by Instant (CLOCK_MONOTONIC); the report line `bench <name>: mean=<n>s min=<n>s max=<n>s stddev=<n>s` (nanoseconds, integers) joins the test-report stream. No auto-scaling in P1 (fixed counts — settlement). Rows {io, nondet}; class CEP-2.

**d6 — Property-based testing.** `pub mod test::prop`: the PRNG is the pinned xorshift64* of SPEC-INDEX §5.2 (state u64; seed 0 → panic site "prop seed must be nonzero"). prop::for_all(name: &str, cases: u32, f: Fn(&mut Rng) → bool): runs f exactly cases times (the harness default is PROP_CASES = 256), stops at the first false, and reports the failing inputs (DebugFormat) plus the seed (same seed + same cases = same inputs, G14). Generator set (closed): u8..u64, i8..i64, f64, bool, char (ASCII), Vec<T> with len 0..=64, String with len 0..=64 (ASCII). Shrinking: generators shrink toward zero (integers and lengths) by binary bisection of the last generated value; at most SHRINK_STEPS = 16 shrink attempts are reported with the final shrunk counterexample.

**d7 — Fuzzing integration.** #[fuzz] functions have signature fn name(data: &[u8]). `plkc --fuzz` compiles a target binary: it reads one input (stdin when piped, else the file named by argv[1]), runs every #[fuzz] function in registration order on that input, and exits 0 normally; a panic exits 1; a trap exits 134 (the external fuzz engine detects failures by exit status). Mutation, corpus management, and coverage guidance are external tooling (Volume 22); std defines only this target contract (settlement).

**d8 — Diagnostic formatting.** All test and failure diagnostics use the d4 format: Location prefix, payload via DebugFormat, one line per diagnostic, stdout as the single stream. Formatting cost is unconstrained (CEP-2); determinism of the stream order is a G14 obligation.

## 19.12 Cryptography and security (design.md §26.11)

Settlement of the §26.11 preamble ("provide a carefully maintained library or specify supported external ecosystem interfaces"): edition P1 does both, split by contract. std::crypto defines exactly the interface contracts below (traits, OsRng, constant-time operations, secret storage, clearing) and ships no algorithm implementations (no AES, SHA, AEAD, RSA, or any other algorithm in std); the maintained external library (`planck-crypto`, an ecosystem crate per Volume 20) implements the traits. Substitutability is the contract: any crate implementing these traits is usable wherever the traits appear. Review criteria: Volume 24 interface.

**d1 — Secure random generation.** `pub struct crypto::OsRng` — fill(&mut [u8]) → Result<(), CryptoError> from the OS entropy source (getrandom, Volume 18 entropy group); rows {io, nondet}. No PRNG, no seeding, no caching in std. OsRng is Send + Sync. The test harness never draws from OsRng implicitly (determinism, G14); test seeds are explicit (§19.11 d6).

**d2 — Hash functions.** `pub trait Digest { const OUTPUT_SIZE: usize; fn new() -> Self; fn update(&mut self, data: &[u8]); fn finalize(self) -> [u8; Self::OUTPUT_SIZE]; fn reset(&mut self); }` — the closed interface surface; std ships no impls (preamble settlement). A compliant impl must: be O(n), never panic (the API is infallible by contract), and document OUTPUT_SIZE. Data-independent memory access on secrets is a review criterion (d7).

**d3 — Cryptographic primitives.** Closed trait surfaces: `pub trait Aead { const KEY_SIZE: usize; const NONCE_SIZE: usize; const TAG_SIZE: usize; fn encrypt(&self, key: &[u8; Self::KEY_SIZE], nonce: &[u8; Self::NONCE_SIZE], aad: &[u8], plaintext: &[u8]) -> Result<ByteBuffer, CryptoError>; fn decrypt(&self, key: &[u8; Self::KEY_SIZE], nonce: &[u8; Self::NONCE_SIZE], aad: &[u8], ciphertext: &[u8]) -> Result<ByteBuffer, CryptoError>; }` (decrypt failure returns Err, never panics; tag mismatch → AuthenticationFailed); `pub trait Kdf { fn derive(&self, ikm: &[u8], salt: &[u8], info: &[u8], out: &mut [u8]) -> Result<(), CryptoError>; }`. `pub enum CryptoError { Unavailable, SourceFailure(u32), AuthenticationFailed, InvalidKeyLength, InvalidNonceLength }` — complete (SourceFailure carries the errno). Rows: {alloc} where an output ByteBuffer is produced, else ∅ plus the impl's declared row.

**d4 — Constant-time operations.** `pub mod crypto::ct` (closed set of exactly three): ct_eq(a: &[u8], b: &[u8]) → bool (length mismatch → false immediately — length is not secret by contract; equal length: one pass, XOR-accumulate all byte differences, no early exit; O(n)); ct_zero32(x: u32) → u32 (0xFFFF_FFFF iff x = 0, else 0, branch-free); ct_select_u32(mask: u32, a: u32, b: u32) → u32 (all-ones mask → a, else 0 → b, branch-free). Invariant for all three: control flow is independent of data content (OB-19-23). Rows ∅.

**d5 — Secret storage.** `pub struct SecretBox<T>` — owns T on the heap ({alloc} at construction); on drop, zeroizes the T storage byte-wise via clear (d6) before deallocation. SecretBox implements neither Format nor DebugFormat (compile-time absence prevents accidental secret printing); !Copy and !Clone; expose(&self) → &T (the name makes exposure explicit; documented risk); into_inner(self) → T. Send + Sync iff T: Send + Sync.

**d6 — Secure memory clearing.** `pub fn crypto::clear(buf: &mut [u8])` — writes 0x00 to every byte via the volatile-store intrinsic (Volume 10 §10.5 store_volatile, ascending byte order); the stores must not be elided, merged, or dead-store-eliminated (intrinsic contract). Postcondition: every byte of buf reads 0 (OB-19-22). Safe (takes &mut [u8]); rows ∅.

**d7 — Cryptographic API contracts.** Every crate implementing the §19.12 traits must satisfy, normatively (review criteria, Volume 24 interface): (a) constant-time handling of all secret-derived values (d4 discipline); (b) no panic paths (Result everywhere; no unwrap); (c) no nondet except documented OsRng use; (d) no hidden allocation beyond documented output buffers; (e) version stability per the crate's semver (Volume 20); (f) per-primitive side-channel documentation (timing and cache). Violations are review rejections of the ecosystem crate, not compile errors: the traits enforce signatures; criteria (a)–(f) are REVIEW obligations (OB-19-23 companion).

## 19.13 Standard library guarantees (design.md §26.12)

The 13 bullets of §26.12 are the per-module guarantee columns; rows g1–g13 state the normative mechanism for every module.

| # | Guarantee | Normative contract |
| --- | --------- | ------------------ |
| g1 | Public API | The item lists of §§19.2–19.12 are the complete public API of edition P1; the inventory is closed (§19.1.1); adding a public item is an edition change. |
| g2 | Type signatures | Every public item has the exact signature of this volume; no overloads, no default arguments beyond those stated. |
| g3 | Effects | Every public function states its row over the pinned atom set (Volume 07 §7.1); std rows are checked by the effect checker like user rows (Volume 07 §7.3.9). |
| g4 | Ownership behavior | Signatures carry complete ownership/borrow information (Volume 06); no hidden ownership transfer (Law 1). |
| g5 | Allocation behavior | Allocation sites are exactly §19.1.5; every allocating function carries {alloc}; non-allocating functions must not allocate (checked by the std test suite's allocation counter, CEP-2 tooling). |
| g6 | Thread safety | Send/Sync per Volume 08 §8.3, stated per type in this volume; unsafe impls carry documented invariants (§8.3 d3). |
| g7 | Panic behavior | Safe std functions must not panic except the declared sites, exactly: the Option/Result unwrap/expect family (§19.2 d14/d15); String::insert/truncate boundary panics (§19.4 d4); slice split_at/chunks/copy_from_slice/swap panics (§19.4 d3); BitVec index and length panics (§19.4 d16); Tree NodeId panics (§19.4 d15); test::assert family (§19.11 d1); prop seed panic (§19.11 d6). In panic = abort builds these are trap reason 10 (Volume 07 §7.11 d12). |
| g8 | Performance expectations | The complexity tables of §19.4 and the accuracy contracts of §19.6 are implementation obligations, not aspirations (OB-19 rows). |
| g9 | ABI guarantees | No std function has a guaranteed ABI except the ABI4 wrappers (§19.1.8); internal conventions may change between minor versions; #[ffi] items keep the Volume 10 contract. |
| g10 | Platform support | Modes M3/M4: full std. Mode M2: exactly {core, mem, coll, text, num} plus crypto without OsRng. Mode M1: none. Targets: x86-64 SysV and AArch64 AAPCS64 only (SPEC-INDEX §3); other targets are declared, not implemented. |
| g11 | Version stability | Edition P1 = std 0.1.x; the signatures of this volume are the 0.1 contract; breaking changes require an edition bump and a Volume 25 governance review; fixes keep 0.1 signatures. |
| g12 | Documentation | Every public item carries its CEP block (standards §10.2) and a doc comment; the std build rejects a public item lacking either (gate C19-E003, FV-2 tooling). |
| g13 | Tests | Every std module ships its certificate tests (the EXH domains of §19.14) in-crate per SPEC-INDEX §5.2; the std build gate runs them (C19-E004 on failure). |

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-19-1 | Option niche rules: for the closed candidate list (references, Box, Rc, Arc, fn pointer, raw pointer, NonNull, NonZeroU8..U64), size_of(Option<T>) = size_of(T) and align_of matches, over the candidate type list on both targets | EXH (12 candidate types × 2 targets) | FV-0 | tools/fv/ob19-1-niche | G3 |
| OB-19-2 | Ordering laws: cmp is total, antisymmetric, transitive for the primitive impls; max/min agree with cmp; domain = all ordered pairs of i8 (65,536 pairs) | EXH | FV-0 | tools/fv/ob19-2-order | G3 |
| OB-19-3 | Eq laws (reflexivity, symmetry, transitivity) and Hash/Eq consistency (equal values feed equal byte strings) over the std impl set on sampled small domains (i8, u8, char ASCII, 2-tuples; ≤ 10^6 cases) | EXH | FV-0 | tools/fv/ob19-3-eq-hash | G3 |
| OB-19-4 | PlanckHasher equals the §19.1.6 definition for every byte string of length ≤ 2 (65,793 cases including empty), and the implementation contains no seed, time, or address input | EXH + REVIEW (seed-freedom invariant) | FV-0 | tools/fv/ob19-4-hasher | G14 |
| OB-19-5 | Vec growth: after any sequence of ≤ 12 push ops, cap ≥ len and cap ∈ {4 × 2^k}; cumulative element copies ≤ 2 × final len | EXH (all op counts ≤ 12) + Z3 (≤ 24 ops symbolic) | FV-1 | tools/fv/ob19-5-vec.smt2 | G13 |
| OB-19-6 | HashMap probe termination: linear-probing lookup with backward-shift deletion terminates within cap probes for any table with load ≤ 7/8 and cap ≤ 16 | Z3 (bounded table model) | FV-1 | tools/fv/ob19-6-probe.smt2 | G11 |
| OB-19-7 | HashMap/HashSet iteration order is a function of (insertion sequence, capacity) only — no time, thread, or address input; same inputs → same order across runs | REVIEW (invariant: order derived from bucket indices) + PROP (paired) | FV-2 | review/ob19-7-iter-order.md | G14 |
| OB-19-8 | VecDeque index arithmetic idx(i) = (head + i) mod cap is in bounds and unique for head ∈ 0..cap, i ∈ 0..len, cap ∈ {4, 8} | EXH (576 cases) | FV-0 | tools/fv/ob19-8-deque | G2 |
| OB-19-9 | RingBuffer: for op sequences ≤ 10 over CAP = 4, count stays in 0..=CAP, pop order is FIFO, and full/push errors are exact | EXH | FV-0 | tools/fv/ob19-9-ring | G2 |
| OB-19-10 | BinaryHeap invariant: parent ≥ children under cmp holds after ≤ 10 mixed push/pop ops over value domain 0..=15 | EXH | FV-0 | tools/fv/ob19-10-heap | G2 |
| OB-19-11 | BTreeMap: in-order iteration yields ascending keys after ≤ 10 mixed insert/remove ops over key domain 0..=15; node key counts stay ≤ 15 (root ≥ 1) | EXH | FV-0 | tools/fv/ob19-11-btree | G3 |
| OB-19-12 | UTF-8 validation rejects exactly the ill-formed forms (overlong, surrogates, > U+10FFFF, truncated) for all 1- and 2-byte sequences (65,792) plus the enumerated 3/4-byte boundary set (256 values) | EXH | FV-0 | tools/fv/ob19-12-utf8 | G2 |
| OB-19-13 | Char-boundary slicing: slice_checked returns Err(NotCharBoundary) exactly when the index is not a boundary, over all (corpus, index) pairs of a fixed 6-corpus set (≤ 10^4 pairs) | EXH | FV-0 | tools/fv/ob19-13-slice | G2 |
| OB-19-14 | Arithmetic laws L1–L3 (checked = Some(math) iff representable; wrapping = math mod 2^N with shift mask; saturating clamps to [MIN, MAX]) over the full i8 domain per op (binary ops 65,536 pairs; shift ops 256 × 41 values) + Z3 for i16/i32 bounded instances | EXH + Z3 | FV-0 | tools/fv/ob19-14-arith.smt2 | G3 |
| OB-19-15 | Conversion and trait-matrix completeness: every (from, to) pair of the 13 numeric primitives has exactly one behavior row matching §19.6 d5; Int/Float carry exactly the d1/d2 method sets | EXH (matrix completeness, 13 × 13) | FV-0 | tools/fv/ob19-15-conv | G15 |
| OB-19-16 | Float Display round-trip: parse(format(x)) = x for the boundary set {0, ±0, ±MIN_POSITIVE, ±MAX, ±EPSILON, powers of 2 in range, 10^k boundaries, 2,048 sampled mantissas} | EXH + PROP (sampled, paired) | FV-1 | tools/fv/ob19-16-ryu | G15 |
| OB-19-17 | Math-function accuracy: sqrt ≤ 0.5 ulp and the ≤ 1/≤ 2 ulp bounds of §19.6 d3 hold on the argument boundary sets against the reference implementation | EXH (boundary sets) | FV-1 | tests/ob19-17-libm | G15 |
| OB-19-18 | IoError mapping totality: every errno in the failure set of the Volume 18 §18.8 subset maps to exactly one IoError variant; the table is total and injective per errno | EXH (enumerated errno artifact) | FV-1 | tools/fv/ob19-18-errno | G15 |
| OB-19-19 | Seek arithmetic: SeekFrom resolution follows the §19.7 d9 formulas and returns InvalidInput rather than overflowing, over the boundary value set {0, ±1, i64::MIN/MAX, u64::MIN/MAX} | Z3 (bounded u64/i64 model) | FV-1 | tools/fv/ob19-19-seek.smt2 | G3 |
| OB-19-20 | Channel state machine: for interleavings ≤ 6 of send/recv/drop over 1 sender + 1 receiver, no message is lost or duplicated; close occurs only after all senders drop and the queue drains; recv after close = RecvError | EXH | FV-0 | tools/fv/ob19-20-channel | G21 |
| OB-19-21 | Semaphore/Barrier invariants: count ∈ [0, n] at every step for interleavings ≤ 6; Barrier releases exactly on the n-th arrival and the leader is that arrival | EXH | FV-0 | tools/fv/ob19-21-sem | G21 |
| OB-19-22 | crypto::clear postcondition: after clear(buf), all bytes read 0, for buffer lengths 0..=64, and the volatile stores are not elided (intrinsic contract, Volume 10 §10.5) | EXH + REVIEW (elision invariant) | FV-0 | tools/fv/ob19-22-clear | G2 |
| OB-19-23 | Constant-time data independence: ct_eq, ct_zero32, ct_select_u32 introduce no branch on data content (MIR branch conditions are not functions of input bytes); ct_zero32/ct_select_u32 formulas correct over a 2^20 boundary set | REVIEW (straight-line invariant) + Z3 (bitvector) | FV-1 | tools/fv/ob19-23-ct.smt2 | G16 |
| OB-19-24 | Duration/Instant laws: nanos < 1_000_000_000 invariant holds through all constructors and ops; duration_since returns None exactly when the argument is later; comparison monotonicity holds on the bounded op domain (≤ 10^6 cases) | EXH | FV-0 | tools/fv/ob19-24-time | G3 |
| OB-19-25 | thread::sleep implementation uses only the futex entry of the OS subset (no clock_nanosleep or other syscall), and the deadline arithmetic never overflows u64 nanoseconds | REVIEW (syscall audit) + Z3 (deadline arithmetic) | FV-1 | review/ob19-25-sleep.md | G13, G21 |
| OB-19-26 | Coverage completeness: every design.md §26 bullet has exactly one coverage-ledger row naming one definition anchor in this volume | REVIEW (ledger audit) | FV-2 | review/ob19-26-coverage.md | G15 |

## Coverage ledger

Bullets in scope: 133, defined: 133.

| design.md § | Bullet (abbreviated) | Defined in |
| ----------- | -------------------- | ---------- |
| 26.1 | Primitive traits | §19.2 d1 |
| 26.1 | Copy | §19.2 d1 |
| 26.1 | Clone | §19.2 d2 |
| 26.1 | Drop | §19.2 d3 |
| 26.1 | Default | §19.2 d4 |
| 26.1 | Equality | §19.2 d5 |
| 26.1 | Ordering | §19.2 d6 |
| 26.1 | Hashing | §19.2 d7 |
| 26.1 | Formatting | §19.2 d8 |
| 26.1 | Debug formatting | §19.2 d9 |
| 26.1 | Conversion | §19.2 d10 |
| 26.1 | Iterator interfaces | §19.2 d11 |
| 26.1 | Numeric traits | §19.2 d12 |
| 26.1 | Error interfaces | §19.2 d13 |
| 26.1 | Option | §19.2 d14 |
| 26.1 | Result | §19.2 d15 (never in d15) |
| 26.1 | Range types | §19.2 d16 |
| 26.1 | Phantom types | §19.2 d17 |
| 26.1 | Manually managed storage | §19.2 d18 |
| 26.2 | Box-like owned pointers | §19.3 d1 |
| 26.2 | Reference-counted pointers | §19.3 d2 |
| 26.2 | Thread-safe reference counting | §19.3 d3 |
| 26.2 | Non-null pointers | §19.3 d4 |
| 26.2 | Maybe-uninitialized storage | §19.3 d5 |
| 26.2 | Manually dropped values | §19.3 d6 |
| 26.2 | Pinning | §19.3 d7 |
| 26.2 | Allocator interfaces | §19.3 d8 |
| 26.2 | Arena interfaces | §19.3 d9 |
| 26.2 | Region interfaces | §19.3 d10 |
| 26.2 | Raw allocation APIs | §19.3 d11 |
| 26.3 | Dynamic arrays | §19.4 d1 |
| 26.3 | Fixed arrays | §19.4 d2 |
| 26.3 | Slices | §19.4 d3 |
| 26.3 | Strings | §19.4 d4 |
| 26.3 | Byte buffers | §19.4 d5 |
| 26.3 | Hash maps | §19.4 d6 |
| 26.3 | Ordered maps | §19.4 d7 |
| 26.3 | Hash sets | §19.4 d8 |
| 26.3 | Ordered sets | §19.4 d9 |
| 26.3 | Linked lists | §19.4 d10 |
| 26.3 | Intrusive lists | §19.4 d11 |
| 26.3 | Deques | §19.4 d12 |
| 26.3 | Ring buffers | §19.4 d13 |
| 26.3 | Binary heaps | §19.4 d14 |
| 26.3 | Trees | §19.4 d15 |
| 26.3 | Bitsets | §19.4 d16 |
| 26.3 | Specialized numeric containers | §19.4 d17 |
| 26.4 | UTF-8 strings | §19.5 d1 |
| 26.4 | String builders | §19.5 d2 |
| 26.4 | Character operations | §19.5 d3 |
| 26.4 | Unicode operations | §19.5 d4 |
| 26.4 | Formatting | §19.5 d5 (trait/grammar §19.2 d8) |
| 26.4 | Parsing | §19.5 d6 |
| 26.4 | Numeric formatting | §19.5 d7 |
| 26.4 | Text encoding | §19.5 d8 |
| 26.4 | Text decoding | §19.5 d9 |
| 26.4 | Byte-oriented APIs | §19.5 d10 |
| 26.5 | Integer utilities | §19.6 d1 |
| 26.5 | Floating-point utilities | §19.6 d2 |
| 26.5 | Mathematical functions | §19.6 d3 |
| 26.5 | SIMD abstractions | §19.6 d4 |
| 26.5 | Numeric conversions | §19.6 d5 |
| 26.5 | Saturating arithmetic | §19.6 d6 |
| 26.5 | Wrapping arithmetic | §19.6 d7 |
| 26.5 | Checked arithmetic | §19.6 d8 |
| 26.5 | Big integers, if provided | §19.6 d9 (settled: not provided in P1) |
| 26.6 | Standard input | §19.7 d1 |
| 26.6 | Standard output | §19.7 d2 |
| 26.6 | Standard error | §19.7 d3 |
| 26.6 | File operations | §19.7 d4 |
| 26.6 | Buffered IO | §19.7 d5 |
| 26.6 | Unbuffered IO | §19.7 d6 |
| 26.6 | Filesystem operations | §19.7 d7 |
| 26.6 | Directory operations | §19.7 d8 |
| 26.6 | Streams | §19.7 d9 |
| 26.6 | Sockets | §19.7 d10 (surface in §19.10 d1–d3) |
| 26.6 | Asynchronous IO interfaces | §19.7 d11 |
| 26.7 | Processes | §19.8 d1 |
| 26.7 | Environment variables | §19.8 d2 |
| 26.7 | Filesystem | §19.8 d3 (set in §19.7 d7/d8) |
| 26.7 | Memory mapping | §19.8 d4 |
| 26.7 | Signals | §19.8 d5 |
| 26.7 | Threads | §19.8 d6 (set in §19.9 d1/d8) |
| 26.7 | System information | §19.8 d7 |
| 26.7 | Timers | §19.8 d8 |
| 26.7 | Synchronization | §19.8 d9 (set in §19.9 d2–d7) |
| 26.7 | Platform APIs | §19.8 d10 |
| 26.8 | Thread abstraction | §19.9 d1 |
| 26.8 | Mutex | §19.9 d2 |
| 26.8 | Read-write lock | §19.9 d3 |
| 26.8 | Condition variable | §19.9 d4 |
| 26.8 | Semaphore | §19.9 d5 |
| 26.8 | Atomic types | §19.9 d6 |
| 26.8 | Channels | §19.9 d7 |
| 26.8 | Scoped threads | §19.9 d8 |
| 26.8 | Task interfaces | §19.9 d9 |
| 26.8 | Executor interfaces | §19.9 d10 |
| 26.9 | TCP | §19.10 d1 |
| 26.9 | UDP | §19.10 d2 |
| 26.9 | Socket interfaces | §19.10 d3 |
| 26.9 | Address parsing | §19.10 d4 |
| 26.9 | DNS integration | §19.10 d5 |
| 26.9 | Networking protocols | §19.10 d6 (settled: none in P1) |
| 26.9 | Async networking | §19.10 d7 |
| 26.9 | Buffer management | §19.10 d8 |
| 26.10 | Assertions | §19.11 d1 |
| 26.10 | Unit testing | §19.11 d2 |
| 26.10 | Test discovery | §19.11 d3 |
| 26.10 | Test reporting | §19.11 d4 |
| 26.10 | Benchmarking | §19.11 d5 |
| 26.10 | Property-based testing | §19.11 d6 |
| 26.10 | Fuzzing integration | §19.11 d7 |
| 26.10 | Diagnostic formatting | §19.11 d8 |
| 26.11 | (preamble) maintained library or ecosystem interfaces | §19.12 preamble (settled: both, split by contract) |
| 26.11 | Secure random generation | §19.12 d1 |
| 26.11 | Hash functions | §19.12 d2 |
| 26.11 | Cryptographic primitives | §19.12 d3 |
| 26.11 | Constant-time operations | §19.12 d4 |
| 26.11 | Secret storage | §19.12 d5 |
| 26.11 | Secure memory clearing | §19.12 d6 |
| 26.11 | Cryptographic API contracts | §19.12 d7 |
| 26.12 | Public API | §19.13 g1 |
| 26.12 | Type signatures | §19.13 g2 |
| 26.12 | Effects | §19.13 g3 |
| 26.12 | Ownership behavior | §19.13 g4 |
| 26.12 | Allocation behavior | §19.13 g5 |
| 26.12 | Thread safety | §19.13 g6 |
| 26.12 | Panic behavior | §19.13 g7 |
| 26.12 | Performance expectations | §19.13 g8 |
| 26.12 | ABI guarantees | §19.13 g9 |
| 26.12 | Platform support | §19.13 g10 |
| 26.12 | Version stability | §19.13 g11 |
| 26.12 | Documentation | §19.13 g12 |
| 26.12 | Tests | §19.13 g13 |

Ledger note: design.md §26.1–§26.12 contain 133 "- " bullets (§26.11's preamble sentence is prose, addressed in §19.12 and ledgered as a 134th informational row above, marked as preamble). Bullets in scope: 133, defined: 133.
