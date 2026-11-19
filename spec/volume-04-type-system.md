# Planck — Volume 04: Type System

CEP:FILE: spec/volume-04-type-system.md
CEP:WHAT: Complete normative type-system specification of Planck 0.1 edition P1: type categories, type identity, compatibility and coercion, inference, generics, traits, capabilities, variance, dynamic types, and type-level programming, with typing judgments, constraint rules, and obligations OB-04-*.
CEP:WHY: design.md §6 (lines 323–524) requires "Complete Type System Specification, including typing and constraint rules for every construct"; without this volume no construct can be statically accepted or rejected by a defined rule, and Volumes 06, 07, 09, 13 have no type layer to build on.
CEP:STATUS: complete
CEP:FAILURE: A construct accepted or rejected outside the rules of this volume is a Severity 0 soundness defect (miscompilation risk) per standards.md §34; an obligation row in the verification matrix that cannot be discharged blocks release.
CEP:ASSUMES: SPEC-INDEX §1 (definition method), §3 (settled decisions: UB-1..UB-6, provenance, identity, repr(P), destruction, orphan rule, effect polymorphism, monomorphization depth 32), §5.3 (effect algebra), §5.4 (ownership state machine), §5.5 (operator table), standards.md §3 (normative verbs), verification standards.md G1–G26, Volume 02 (grammar, literals, operators), Volume 03 (name resolution, DefId), Volume 05 (layout, values, memory), Volume 06 (ownership, regions), Volume 07 (effects, execution), Volume 08 (Send/Sync use), Volume 09 (const evaluation), Volume 10 (FFI/ABI, intrinsics), Volume 12 (compiler limits table), Volume 13–15 (IR mappings).
CEP:COST: compile-time only; type checking is O(|program types|) amortized with the bounds stated per algorithm; no runtime cost is added by this volume.
CEP:EVIDENCE: verification matrix (## Verification matrix) with obligation IDs OB-04-1..OB-04-16, methods Z3/EXH/PROP/REVIEW, classes FV-0/FV-1/FV-2, G-mappings; coverage ledger (## Coverage ledger) with one row per design.md §6 bullet.

---

## 4.0 Scope and traceability

This volume is the definition home for design.md section **6 "Type system specification" (lines 323–524)**, including its deliverable statement at line 521 ("Complete Type System Specification, including typing and constraint rules for every construct").

| design.md subsection | design.md lines | This volume | Bullets in subsection |
| -------------------- | --------------- | ----------- | --------------------- |
| 6.1 Type categories | 327–353 | §4.1 | 23 |
| 6.2 Type identity | 355–367 | §4.2 | 11 |
| 6.3 Type compatibility | 369–385 | §4.3 | 15 |
| 6.4 Type inference | 387–405 | §4.4 | 15 |
| 6.5 Generic system | 407–429 | §4.5 | 21 |
| 6.6 Trait system | 431–455 | §4.6 | 23 |
| 6.7 Capabilities | 457–479 | §4.7 | 16 |
| 6.8 Variance | 481–492 | §4.8 | 10 |
| 6.9 Dynamic types | 494–506 | §4.9 | 11 |
| 6.10 Type-level programming | 508–519 | §4.10 | 10 |

bullets in scope: 155, defined: 155 (ledger in ## Coverage ledger).

The 16-row feature template of design.md §40 (lines 4125–4152) is applied to the language features dyn Trait (§4.11.1) and closures (§4.11.2). Where a template row is owned by another volume, the row names the owning volume and section; where a row does not apply, it states "not applicable because …".

### 4.0.1 Judgment forms

All rules in this volume use the judgment forms below. A rule is written as an inference rule: premises above the horizontal line, conclusion below, side conditions after the rule. Γ is a typing context (variable bindings with types, lifetime constraints, and trait obligations); Σ is the trait environment (impls in scope); Δ is the const-generic environment.

| Judgment | Meaning |
| -------- | ------- |
| Γ ⊢ τ wf | τ is a well-formed type in Γ |
| Γ ⊢ e : τ | expression e has type τ |
| Γ ⊢ p : τ | pattern p accepts values of type τ |
| Γ ⊢ τ1 <: τ2 | τ1 is a subtype of τ2 (§4.3.9) |
| Γ ⊢ τ1 ⇝ τ2 | τ1 coerces to τ2 by the coercion table (§4.3.3) |
| Σ ⊢ τ : Trait⟨...⟩ | trait obligation holds (§4.6.9) |
| Γ ⊢ ρ ok | effect row ρ is well-formed (§5.3 of SPEC-INDEX; rows are finite subsets of the closed atom set) |
| Γ ⊢ unif(τ1, τ2) = σ | unification of τ1 and τ2 yields substitution σ (§4.4.2) |

Core expression rules:

```
T-Var    x : τ ∈ Γ
         ─────────────
         Γ ⊢ x : τ

T-Let    Γ ⊢ e1 : τ1    Γ, x : τ1 ⊢ e2 : τ2
         ───────────────────────────────────
         Γ ⊢ let x = e1; e2 : τ2

T-Call   Γ ⊢ f : (τ1 … τn) ⇝ τr with row ρf
         Γ ⊢ ei : τi for i = 1..n
         Γ ⊢ ρf ≤ ρcaller
         ─────────────────────────────────────────
         Γ ⊢ f(e1 … en) : τr
```

Side condition T-Call: ρf ≤ ρcaller is row subtyping per SPEC-INDEX §5.3 (ρf ⊆ ρcaller); the caller's row must include every atom of the callee's row. The rule applies to all call forms (direct, indirect through fn pointers, method calls); indirect calls additionally require the pointer's ABI to equal the callee's ABI, else runtime UB-4 (Volume 05 §8.6).

```
T-Field  Γ ⊢ e : τs    field f of struct τs has type τf
         ─────────────────────────────────────────────
         Γ ⊢ e.f : τf

T-Index  Γ ⊢ e : [T] or [T; N]    Γ ⊢ i : usize
         ─────────────────────────────────────────
         Γ ⊢ e[i] : T

T-Cast   Γ ⊢ e : τa    (τa, τb) ∈ cast-table T04-5
         ─────────────────────────────────────────
         Γ ⊢ e as τb : τb

T-Ret    Γ, ret:τr ⊢ e : τr
         ─────────────────────────────────────
         Γ ⊢ return e : !

T-Nev    Γ ⊢ e : !    τ is any well-formed type
         ─────────────────────────────────────
         Γ ⊢ e : τ
```

Side condition T-Nev: the coercion of ! to τ is defined only in expression positions (§4.3.10); it never inserts code.

### 4.0.2 Well-formedness of types

```
WF-Prim   τ ∈ primitives(T04-1)            ─────────────   Γ ⊢ τ wf
WF-Nom    def(d) declares τ    ∀ params p of d: p instantiated or defaulted
          ────────────────────────────────────────────────────────────
          Γ ⊢ τ wf
WF-Refr   Γ ⊢ τ wf    lifetime 'a ∈ Γ
          ─────────────────────────────────
          Γ ⊢ &'a τ wf      Γ ⊢ &'a mut τ wf
WF-Dyn    Γ ⊢ dyn Trait + 'a ok per object-safety predicate §4.9.9
          ─────────────────────────────────────────────────────────
          Γ ⊢ dyn Trait + 'a wf
WF-Fn     ∀i: Γ ⊢ τi wf    Γ ⊢ τr wf    ABI ∈ declared ABIs
          ─────────────────────────────────────────────────────
          Γ ⊢ fn(τ1 … τn) ⇝ τr wf
WF-Arr    Γ ⊢ T wf    N is a usize-valued constant in scope
          ─────────────────────────────────────────────────
          Γ ⊢ [T; N] wf
```

Side condition WF-Arr: N ∈ [0, 2^64−1]. An array type with size(T) × N > MAX_ALLOC_SIZE (Volume 05 §5.11) is rejected at monomorphization with diagnostic E-P04-018.

---

## 4.1 Type categories

### 4.1.1 Master table

Every type category of design.md §6.1 is defined by exactly one row of table T04-1 plus the per-category rules that follow. Size and align are defined by the layout rules of Volume 05 §5.5; the columns here state the derived class.

T04-1 (all 23 categories):

| Category | Syntax | Size | Align | Copy | Drop glue | DST | Identity class |
| -------- | ------ | ---- | ----- | ---- | --------- | --- | -------------- |
| Integer | i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 isize usize | width w ∈ {1,2,4,8,16}; isize/usize = 8 | w | yes | no | no | primitive |
| Float | f32 f64 | 4 / 8 | 4 / 8 | yes | no | no | primitive |
| Boolean | bool | 1 | 1 | yes | no | no | primitive |
| Character | char | 4 | 4 | yes | no | no | primitive |
| Unit | () | 0 | 1 | yes | no | no | primitive |
| Never | ! | 0 | 1 | yes | no | no | primitive, uninhabited |
| Tuple | (T1, …, Tn) | repr(P) sum, padded | max field align | iff all fields Copy | iff any field drops | if last field DST | structural |
| Array | [T; N] | N × size(T) | align(T) | iff T: Copy | iff T drops | no | structural-parameterized |
| Slice | [T] | DST (fat: ptr + len) | 8 | no | no (non-owning) | yes | structural |
| Struct | Nominal | repr(P) | max field align | per fields | per fields | if last field DST | nominal |
| Enum | Nominal | tag + payload (§5.5) | max field align | iff every variant field-set Copy and no Drop impl | yes (active variant) | if any variant field DST (last) | nominal |
| Union | Nominal | max field size | max field align | only via unsafe impl Copy | only via Drop impl (no field glue) | if declared | nominal |
| Reference | &'a T, &'a mut T | 8 thin / 16 fat | 8 | & yes; &mut no | no | fat when T DST | primitive |
| Raw pointer | *const T, *mut T | 8 thin / 16 fat | 8 | yes | no | fat when T DST | primitive |
| Function pointer | fn(τ…) ⇝ τ | 8 | 8 | yes | no | no | signature + ABI |
| Closure | anonymous nominal | captures in repr(P) | max capture align | iff all captures Copy | iff any capture drops | no | closure site |
| Trait object | dyn Trait + 'a | DST (fat: ptr + vtable) | 8 | no | vtable drop_fn | yes | trait ref + lifetime |
| Opaque | impl Trait | layout of hidden instance type | of hidden type | of hidden type | of hidden type | if hidden type DST | definition site |
| Generic | T (parameter) | of instantiation | of instantiation | of instantiation | of instantiation | if instantiated so | parameter |
| Dynamically sized | ?Sized bound | unknown at compile time | 8 minimum | no | via owning pointer | yes | property |
| SIMD | core::simd vector | lanes × elem size | = size | yes | no | no | (elem, lanes) |
| Foreign | extern block type | declared | declared | only if declared | only if declared | if declared unsized | nominal |
| Zero-sized | property (size = 0) | 0 | ≤ 2^29 | allowed | no-op glue | allowed | property |

### 4.1.2 Integer types

T04-2 (exact ranges and overflow class; overflow class is the operator-table column of SPEC-INDEX §5.5; the default class for every Planck integer operator is Panic):

| Type | Bytes | Minimum | Maximum | Default overflow class |
| ---- | ----- | ------- | ------- | ---------------------- |
| i8 | 1 | −128 | 127 | Panic |
| i16 | 2 | −32768 | 32767 | Panic |
| i32 | 4 | −2147483648 | 2147483647 | Panic |
| i64 | 8 | −9223372036854775808 | 9223372036854775807 | Panic |
| i128 | 16 | −2^127 | 2^127 − 1 | Panic |
| u8 | 1 | 0 | 255 | Panic |
| u16 | 2 | 0 | 65535 | Panic |
| u32 | 4 | 0 | 4294967295 | Panic |
| u64 | 8 | 0 | 18446744073709551615 | Panic |
| u128 | 16 | 0 | 2^128 − 1 | Panic |
| isize | 8 | −2^63 | 2^63 − 1 | Panic |
| usize | 8 | 0 | 2^64 − 1 | Panic |

Side condition: isize and usize are 64-bit on both edition-P1 targets (x86-64 SysV, AArch64 AAPCS64, per SPEC-INDEX §3 ABI policy). Representation is two's complement; every bit pattern of an integer type is a valid value. Types larger than 16 bytes or non-power-of-two widths must not exist (no such declarations are possible; primitives are a closed set).

### 4.1.3 Floating-point types

f32 is IEEE-754 binary32; f64 is IEEE-754 binary64. Operations are round-to-nearest, ties-to-even. Every bit pattern including NaN payloads and infinities is a valid value. Equality is IEEE-754 equality (+0 == −0; NaN ≠ NaN). The canonical quiet NaN produced by Planck operations is bit pattern 0x7FC00000 (f32) and 0x7FF8000000000000 (f64); bit-preservation of NaN payloads through operations is not guaranteed (a defined non-determinism within the NaN class, carrying the nondet atom).

### 4.1.4 Boolean

bool has exactly two values: true (bit pattern 0x00) and false (bit pattern 0x01). The set {0x00, 0x01} is the representation invariant of bool. Storage holding any other byte is not a valid bool value: a typed read of such storage is a defined ReprTrap (Volume 05 §5.12, exit code 134), not UB.

### 4.1.5 Character

char is a Unicode scalar value excluding U+D800..U+DFFF, stored in 4 bytes in native little-endian byte order on both targets. The representation invariant is: 4-byte little-endian encoding of a scalar in [0x000000, 0xD7FF] ∪ [0xE000, 0x10FFFF]. Byte patterns outside this set are not valid char values: typed read → ReprTrap (exit code 134).

### 4.1.6 Unit

The unit type () has one value, the empty tuple. size = 0, align = 1. A binding of type () has no storage.

### 4.1.7 Never

The never type ! is uninhabited: it has no values. size = 0, align = 1. The only expressions of type ! are: diverging expressions (panic, infinite loop with effect row containing no return), return e, break/continue to an enclosing loop, and calls to functions returning !. Rule T-Nev allows ! to coerce to every type. The compiler must not emit a ! value; a code path that would produce one is dead. If the compiler cannot prove a !-typed expression diverges or exits, it is a compile-time rejection (E-P04-019).

### 4.1.8 Tuples

A tuple (T1, …, Tn) is a structural product: two tuple types are the same type iff they have the same arity n and Ti = T'i for all i (canonical equality, §4.3.1). Layout is repr(P) (Volume 05 §5.5.1) with fields in positional order. () is the 0-tuple and is the unit type.

### 4.1.9 Arrays

[T; N] is a fixed-length sequence of N values of type T stored contiguously with element stride size(T), align(T). N is a usize-valued constant. Indexing e[i] with i ≥ N or i < 0 (checked as unsigned) raises the defined BoundsTrap (Volume 05 §5.9); the compiler may elide the check only when it proves i ∈ [0, N) statically; the obligation OB-05-8 family (Volume 05) covers elision soundness. [T; 0] is a zero-sized type.

### 4.1.10 Slices

[T] is a dynamically sized sequence with metadata len: usize. A slice value exists only behind a pointer: &[T] and *const [T] are fat pointers (data pointer, len). The byte range of the slice is [data, data + len × size(T)); the reference validity predicate of Volume 05 §5.10 requires the whole range inside the referent allocation. Slice literals [a, b, c] produce arrays; the array-to-slice coercion (§4.3.6) forms slices.

### 4.1.11 Structs

A struct is nominal with fields in declaration order. Field types must be well-formed in the struct's parameter environment. Struct layout is repr(P) by default (SPEC-INDEX §3; exact arithmetic Volume 05 §5.5.1); #[repr(C)], #[repr(packed)], #[repr(packed(N))], #[repr(transparent)], #[repr(align = A)] are the permitted overrides (Volume 05 §5.5). A struct whose last field has a DST type is a DST struct; its metadata is the metadata of the last field. Field access order and projection are defined in Volume 05 §5.1.

### 4.1.12 Enums

An enum is nominal with variants v0 … vk−1. Each variant carries a discriminant: by default d(vi) = i (declaration order); an explicit discriminant `= c` sets d(vi) = c and must not equal any other discriminant (rejection E-P04-020). Discriminants fit in the tag type selected by Volume 05 §5.5.2. Construction of variant vi with fields (a1 … am) produces the unique value of the variant; a typed read of an enum value whose tag byte pattern does not equal any declared discriminant raises ReprTrap (exit code 134) at the read, not UB.

### 4.1.13 Unions

A union is nominal with fields at offset 0 (all fields share storage; Volume 05 §5.5.4). Rules:

1. All fields must be Copy, or the union must not be read outside unsafe code (reading any union field is an unsafe operation with the unsafe atom).
2. Reading field f produces the bytes of the storage reinterpreted as repr(f); if the bytes violate the representation invariant of f's type (bool, char, enum, reference), the result is ReprTrap at the typed read (exit code 134), never UB.
3. Writing field f marks the union storage initialized and replaces the value; the previous value is not dropped (unions have no field drop glue).
4. A Drop impl may be written for a union (unsafe obligation: the impl must not assume any active field).
5. unsafe impl Copy for a union is permitted only if every field is Copy; the impl author carries the UB-6 obligation for any copy of a union whose storage violates a field's representation invariant.

### 4.1.14 References

&'a T (shared) and &'a mut T (exclusive) are first-class values: (allocation ID, offset, capability set, region 'a). Sizes: 8 bytes for thin (T: Sized), 16 bytes fat. &'a T is Copy (reborrow-free copy; the copy shares the same loan); &'a mut T is not Copy; every use of a &mut value performs an implicit reborrow defined in Volume 06 §9.4. A reference value is valid iff the predicate valid_ref of Volume 05 §5.10.1 holds; creation or use of an invalid reference is UB-2. Reference typing:

```
T-Ref    Γ ⊢ e : T    place(e) is defined    borrow checker (Vol 06) grants loan l
         ─────────────────────────────────────────────────────────────────────────
         Γ ⊢ &e : &'l T        Γ ⊢ &mut e : &'l mut T
```

Side condition: l is the loan region; for &mut the loan is exclusive (state machine SPEC-INDEX §5.4: BorrowMut requires Owned; BorrowShared requires Owned, Shared(n), or Frozen(n)).

### 4.1.15 Raw pointers

*const T and *mut T are (address: usize, provenance) pairs; provenance is the record of SPEC-INDEX §3: allocation ID + offset + capability set ⊆ {load, store, compute}. *const T carries {load, compute} at formation from &T; *mut T carries {load, store, compute} at formation from &mut T or from allocation. Pointer casts (§4.3.7) preserve the capability set unchanged; casting never grants capabilities. Dereference of a raw pointer is an unsafe operation; the exact dereference semantics (provenance check, trap on failure) are Volume 05 §5.9. Raw pointers implement Copy and have no drop glue.

### 4.1.16 Function pointers

The function pointer type is fn(τ1, …, τn) ⇝ τr with two type-level flags: ABI ∈ {Planck, C} (edition P1 ABI set per SPEC-INDEX §3) and safety ∈ {safe, unsafe}. Typed as a thin 8-byte value (address) whose provenance has capability {compute} at minimum. Calling a fn pointer is an indirect call: the callee's ABI must equal the pointer's ABI and the callee's signature must equal the pointer's signature; violation is UB-4 (Volume 05 §8.6). A null fn pointer (bit pattern 0) is a defined value; calling it raises CallTrap (exit code 134), not UB.

### 4.1.17 Closures

Every closure expression elaborates to a unique anonymous nominal type (identity §4.2.9) holding its captures in declaration order of first capture (repr(P)). The closure's kind is inferred exactly: FnOnce if any capture is moved out during the body or the body consumes a capture by value; FnMut if any capture is assigned or mutably borrowed and no capture is moved out; Fn otherwise. The closure implements the corresponding core trait (§4.6). A closure with capture set ∅ coerces to a fn pointer of its signature (§4.3.8). The full feature-template application is §4.11.2.

### 4.1.18 Trait objects

dyn Trait + 'a is the existential type {∃ X. X: Trait, X: 'a} represented as a fat pointer (data pointer, vtable pointer). Construction, layout, dispatch, and the object-safety predicate are §4.9. dyn is a DST: it exists only behind pointers.

### 4.1.19 Opaque types

`impl Trait` in return position introduces an anonymous nominal type O whose identity is the definition site (function DefId + which return position) and whose hidden type is fixed per instantiation by the compiler. `impl Trait` in argument position is exactly a fresh generic type parameter with the stated bounds. The hidden type must be unique per instantiation (all return paths yield the same type; rejection E-P04-021 otherwise). Opaque types are never named by user code and are not dyn-constructible.

### 4.1.20 Generic types

A generic type is a nominal type with open parameters; it is not a type until every parameter is determined (full substitution, §4.5.7). Partially substituted generics cannot be named as runtime values (E-P04-022, §4.5.9).

### 4.1.21 Dynamically sized types

The ?Sized relaxation on a parameter or type declares the size may be unknown at compile time. The exact set of DST types is: [T], dyn Trait + 'a, structs whose last field is a DST, foreign types declared unsized. A DST cannot be a local binding, a function parameter or return by value, or a field other than the last field (rejection E-P04-016 for each violation).

### 4.1.22 SIMD types

The SIMD types are core::simd::{iN x L, uN x L, fN x L} with element ∈ {i8,u8,i16,u16,i32,u32,i64,u64,f32,f64} and lanes L ∈ {1,2,4,8,16,32,64} subject to width = L × elem-size ≤ 512 bits. Size = width/8 bytes; align = size. SIMD values have value semantics (Copy, no interior mutability). Lane-wise operations are intrinsics defined in Volume 10 §16.3; this volume defines only the types, their layout class, and their Copy status. The set of SIMD types is closed.

### 4.1.23 Foreign types

A foreign type is declared in an extern block (Volume 10 §17.2) with a declared layout: either #[repr(C)] field list, or `extern type` (unsized, opaque, align 1, size unknown). Foreign types have no Copy, Clone, or Drop unless the declaration states it; their fields are accessible only inside unsafe code with the unsafe atom. Foreign types participate in the orphan rule as foreign types.

### 4.1.24 Zero-sized types

A zero-sized type (ZST) is any type with size 0: (), !, [T; 0], structs/enums all of whose fields (or variant payloads) are ZSTs. Rules: a ZST has align ≤ 2^29 (its align is the max field align, possibly 1); a ZST occupies no bytes and has no padding; copying, moving, and dropping a ZST are no-ops; a ZST may be passed by value in the Planck convention ( lowered as no register) per Volume 10 §17.1. A pointer to a ZST place has provenance with offset arbitrary within the referent allocation (the validity predicate of Volume 05 §5.10.1 requires only offset ≤ allocation size for ZSTs).

---

## 4.2 Type identity

### 4.2.1 Canonical form

Type identity is defined by a total function canon(τ) → CanonicalType. Two types are the same type iff canon(τ1) = canon(τ2). canon is computed after: (1) name resolution (Volume 03) replaced every path by a DefId; (2) aliases erased; (3) all inference variables solved or rejected; (4) projections normalized (§4.2.6); (5) substitutions canonicalized (per-parameter components canonized recursively; parameter order fixed). canon is a pure, deterministic function (G14): identical inputs in any compilation unit produce identical canonical forms.

### 4.2.2 Nominal type identity

For a nominal item (struct, enum, union, foreign type) with DefId d and substitution σ: canon = Nom(d, canon∘σ). Two structs with identical field lists but different DefIds are different types. The same item instantiated with different substitutions is a different type (§4.2.5).

### 4.2.3 Structural type identity

Tuples, arrays, slices, references, raw pointers, fn pointers, and SIMD types are structural: canon is the labeled tree of their components. Two tuple types (A, B) and (A, B) are the same type regardless of where they are written. Primitive types are their own canonical atoms.

### 4.2.4 Type aliases

A type alias `type X = T` is transparent: it has no runtime or type-identity existence; canon erases it to canon(T). Aliases are checked for well-formedness at declaration and expanded at every use. An alias never introduces a new type; it cannot implement traits.

### 4.2.5 Newtype identity

A single-field struct `struct M(T)` is nominal: canon(M) = Nom(dM, ∅). M and T are different types with different method sets and different trait impls, even though their layout is identical (layout compatibility is a separate relation, Volume 05 §5.5.9).

### 4.2.6 Instantiated type identity and associated type identity

Generic instantiation identity: canon(Nom(d, σ)) with σ fully determined. Two instantiations are equal iff same d and canon-equal substitutions. Associated type identity: the projection ⟨T as Trait⟩::Assoc is canonically equal to the projection's unique normal form: if the trait solver (§4.6.9) determines a unique impl for ⟨T as Trait⟩, the projection normalizes to that impl's associated-type definition applied to the impl's substitution; two projections are equal iff their normal forms are equal; a projection with ambiguous normalization is a type error (E-P04-012), never a fresh type.

### 4.2.7 Opaque type identity

canon(impl Trait at site s) = Opaque(s, σ) where σ is the substitution of the enclosing generic parameters. Opaque types with different sites are always distinct, even with identical hidden types. The hidden type is visible only to the compiler (for layout, codegen); it is not identity-relevant.

### 4.2.8 Function type identity

canon(fn(τ1 … τn) ⇝ τr) = Fn(ABI, safety, [canon τi], canon τr). Parameter names are not part of identity; ABI and safety flag are. fn() ⇝ i32 with ABI C and with ABI Planck are different types.

### 4.2.9 Closure type identity

canon(closure at expression site e with captures c1 … cm) = Closure(e, [canon ci]). Two closure expressions, even textually identical, have distinct Closure(e) atoms. A closure type is never nameable; it implements Fn/FnMut/FnOnce per §4.1.17.

### 4.2.10 Dynamic type identity

canon(dyn T + 'a) = Dyn(canon(T), 'a, auto-set A) where A is the explicit auto-trait set (§4.9.7). dyn Trait + 'a and dyn Trait + 'b with 'a ≠ 'b are different types. Two dyn types over the same trait ref, lifetime, and auto set are the same type even if formed at different source locations.

T04-3 (identity summary, one row per §6.2 bullet):

| §6.2 bullet | Identity rule | Anchor |
| ----------- | ------------- | ------ |
| Nominal type identity | Nom(DefId, σ) | §4.2.2 |
| Structural type identity | labeled tree of components | §4.2.3 |
| Type aliases | erased, transparent | §4.2.4 |
| Newtype identity | distinct nominal | §4.2.5 |
| Generic type identity | Nom(DefId, open σ) | §4.2.6 |
| Instantiated type identity | Nom(DefId, full σ) | §4.2.6 |
| Associated type identity | unique normal form | §4.2.6 |
| Opaque type identity | Opaque(site, σ) | §4.2.7 |
| Function type identity | Fn(ABI, safety, params, ret) | §4.2.8 |
| Closure type identity | Closure(site, captures) | §4.2.9 |
| Dynamic type identity | Dyn(trait ref, 'a, autos) | §4.2.10 |

---

## 4.3 Type compatibility

### 4.3.1 Exact type equality

Equality is canon equality (§4.2.1). It is total, decidable, deterministic, and O(|τ|) after normalization.

### 4.3.2 Type unification

Unification unify(τ1, τ2) is first-order unification over the canonical grammar extended with inference variables (§4.4.1), with the occurs check.

Algorithm: structural descent; on variable v: if bound, recurse on the bound; else bind v := τ after occurs check (v must not occur in τ; violation = unification failure). On nominal nodes: require equal DefId, then unify component-wise. On Lifetimes: unify region variables per Volume 06 §9.5. On IntVar/FloatVar: unify with integer/float atoms of matching class.

Input domain: finite canonical types with |τ| ≤ node budget from the Volume 12 limits table. Output: most general unifier σ or Failure(reason). Termination: each recursive step strictly reduces total node count or binds a variable (occurs check prevents self-binding growth); measure = total node count of the two types; worst-case complexity O(|τ1| + |τ2|). Unification is deterministic (G14): variable pick order is declaration order of variables.

### 4.3.3 Coercion

Coercion ⇝ is the relation of table T04-4, applied implicitly at typed positions (argument, let-annotation, return, field). Coercion never changes representation except as stated (fat-pointer construction for unsizing). At most one coercion is applied per position (no chains beyond: autoref + unsizing + lifetime weakening, in that order, one each).

T04-4 (implicit coercion table):

| From | To | Kind | Rule |
| ---- | -- | ---- | ---- |
| τ | τ' where τ <: τ' | subtyping | lifetime/variance only (§4.3.9) |
| &mut T | &T | mut-to-shared decay | same provenance; capability set narrowed to {load, compute} |
| &T | *const T | pointer decay | provenance preserved; caps {load, compute} |
| &mut T | *mut T | pointer decay | provenance preserved; caps {load, store, compute} |
| [T; N] | [T] | unsizing | metadata len = N |
| &[T; N] | &[T] | autoref + unsizing | metadata len = N |
| T (Sized) | dyn Trait + 'a | unsizing | requires T: Trait, T: 'a, object safety; vtable built per §4.6.13 |
| &T | &dyn Trait + 'a | unsizing at reference | same |
| fn item | matching fn pointer | function coercion | address of item |
| closure with ∅ captures | matching fn pointer | function coercion | §4.3.8 |
| ! | any τ | never coercion | §4.3.10 |
| &'a T | &'b T with 'b: 'a? no — 'a outlives 'b | lifetime weakening | 'a: 'b required |

### 4.3.4 Implicit conversions

Implicit conversions are exactly the coercions of T04-4 plus numeric literal fallback (§4.4.3). Nothing else converts implicitly: user impls cannot add implicit coercions; there is no Deref-based coercion beyond the single deref step inside method resolution (§4.6.10).

### 4.3.5 Explicit conversions

Explicit conversion is the `as` cast, defined by table T04-5. Casts are total; a cast never traps; its semantics are exact per row.

T04-5 (cast table):

| From | To | Semantics |
| ---- | -- | --------- |
| integer iN | integer jM | truncate to M bytes, reinterpret two's complement (wrapping mod 2^(8M)) |
| integer iN | f32/f64 | nearest value, round-to-nearest ties-to-even; exact if representable |
| f32/f64 | integer jM | saturating: NaN → 0; values ≥ max(jM) → max(jM); ≤ min → min; else truncation toward zero |
| f32 | f64 | exact widening |
| f64 | f32 | nearest f32, ties-to-even; overflow → ±inf |
| bool | integer jM | false = 0, true = 1 |
| char | u32/u128/u64/i64/isize? — char casts to any integer of width ≥ 4 bytes | scalar value as unsigned integer |
| u8 | char | always valid (u8 ⊆ scalar range) |
| enum (C-like) | tag integer type | discriminant value |
| *const T / *mut T | *const U / *mut U | bit-preserving; provenance and capabilities preserved unchanged |
| fn pointer | fn pointer (same ABI, arity) | address-preserving; signature checked only at call (UB-4 rule) |
| *const T/*mut T | usize | address bits; marks provenance exposed (Volume 05 §5.8.11) |
| usize | *const T/*mut T | address bits; provenance per expose model (Volume 05 §5.8.12); capabilities {compute} only |

Prohibition: casts between &T and *T are coercions (decay), not `as` casts; `as` between references and integers is banned (E-P04-023); `as` between different function-pointer ABIs is banned (E-P04-024).

### 4.3.6 Numeric conversions

Numeric conversions are the integer/float rows of T04-5. Every numeric conversion is exact per row; no conversion depends on target beyond the fixed widths of T04-2. Integer literal → wider type is always exact; narrower requires the literal to fit (literal-level rule, SPEC-INDEX §5.6).

### 4.3.7 Pointer conversions

Pointer conversions are the pointer rows of T04-5 plus decay rows of T04-4. Capability sets are never widened by any conversion. Casting does not validate: validity is checked only at dereference/reference-formation time (Volume 05 §5.9, §5.10).

### 4.3.8 Reference conversions

Reference conversions are: variance subtyping (§4.3.9), lifetime weakening, mut-to-shared decay, reborrow &mut T → &T via *r, and unsizing at the reference level. A reborrow shares the loan region and is checked by the borrow checker (Volume 06 §9.4).

### 4.3.9 Array-to-slice conversions

[T; N] ⇝ [T] and &[T; N] ⇝ &[T] are unsizing coercions with metadata len = N. The fat pointer's data address equals the array's address; provenance is preserved unchanged.

### 4.3.10 Function coercions

A function item coerces to fn pointer type with its own signature and ABI = Planck (C if declared extern). A closure with capture set = ∅ coerces to fn pointer of the closure's signature; capture emptiness is checked after capture inference (Volume 06 §9.2). A closure with captures must not coerce (rejection E-P04-025). A fn item type is a zero-sized nominal unique to the item; its only operation is address formation.

### 4.3.11 Trait object coercions

T ⇝ dyn Trait + 'a requires all of: T: Trait (by an impl visible in Σ), T: Sized, T: 'a, Trait object-safe (§4.9.9), and any auto traits listed at the dyn type are structurally implied by T (§4.9.7). The vtable is built per §4.6.13. Upward coercion dyn Sub + 'a ⇝ dyn Super + 'a requires Sub: Super and is a no-op on the vtable pointer (prefix property, §4.6.13).

### 4.3.12 Unsizing

Unsizing is the coercion family changing Sized to DST form. The only unsize coercions are: [T; N] → [T]; T → dyn Trait; struct-last-field unization: Struct⟨…, tail = U⟩ ⇝ Struct⟨…, tail = U'⟩ where U ⇝ U'. User impls of the internal trait Unsize are banned (E-P04-026); the builtin set above is closed.

### 4.3.13 Subtyping

Subtyping exists in edition P1 exactly as lifetime/variance-driven subtyping plus the never type; there is no nominal subtyping (no inheritance between structs/enums).

```
S-Refl   ──────────── Γ ⊢ τ <: τ
S-Cov    Γ ⊢ τ1 <: τ2    p covariant in F
         ────────────────────────────── Γ ⊢ F[..τ1..] <: F[..τ2..]
S-Contra Γ ⊢ τ2 <: τ1    p contravariant in F
         ────────────────────────────── Γ ⊢ F[..τ1..] <: F[..τ2..]
S-Inv    p invariant in F
         ───────────────────────────── Γ ⊢ F[..τ1..] <: F[..τ2..] only if τ1 = τ2
S-Lif    'a: 'b in Γ
         ─────────── Γ ⊢ &'b T <: &'a T      Γ ⊢ &'b mut T <: &'a mut T is invalid
S-Nev   (no premise)
         ─────────── Γ ⊢ ! <: τ for any τ
```

Side condition S-Lif: the exclusive-reference rule is stated as: &'b mut T <: &'a mut T requires 'b = 'a (invariance through &mut). Subtyping is reflexive and transitive (obligation OB-04-7).

### 4.3.14 Never-type coercion

! coerces to every type in expression position, in match-arm unification, and in return position. It does not coerce in type-argument position (T = ! as a generic argument is allowed but no coercion applies to substituted uses). Codegen of a never-coercion is empty.

### 4.3.15 Lifetime compatibility

The outlives judgment 'a: 'b is generated by: declared bounds (T: 'a, impl bounds, fn bounds), variance-implied implied bounds (§4.8), and 'static: 'b for all 'b. Region solving is Volume 06 §9.5; this volume defines the judgment and its use in WF-Refr and S-Lif.

---

## 4.4 Type inference

### 4.4.1 Inference variables

Three kinds: TyVar (any type), IntVar (one of the 12 integer types), FloatVar (f32/f64). Each variable has lattice states: Unbound ⊑ Bound(τ). A variable bound twice to different canonical types is a type error (E-P04-001). IntVar/FloatVar also carry a constraint set of permitted types from literal suffixes and operator signature propagation.

### 4.4.2 Constraint solving

Constraints are accumulated per function body; solving runs after body checking: (1) union-find resolution of TyVar bindings to fixed point (termination: each union-find link strictly reduces remaining variables; measure = number of variables; O(n α(n))); (2) trait obligations (§4.6.9) solved to fix point with depth limit SOLVER_DEPTH_LIMIT = 64; (3) fallback application (§4.4.3); (4) remaining unbound TyVar ⇒ E-P04-002.

### 4.4.3 Numeric literal inference

An unsuffixed integer literal creates IntVar with constraint "all 12 integer types permitted"; an unsuffixed float literal creates FloatVar {f32, f64}. A literal used in a position expecting a concrete type binds to it if permitted. At the end of solving: unbound IntVar ⇒ i32; unbound FloatVar ⇒ f64. Fallback never applies inside a const generic argument position (must be determined; rejection E-P04-002).

### 4.4.4 Per-construct inference rules

T04-6 (all 15 §6.4 bullets):

| Bullet | Exact rule | Anchor |
| ------ | ---------- | ------ |
| Local variable inference | let x = e: x's type is the solved type of e; annotation, if present, is unified with e's type (coercion applied) | §4.4.2 |
| Numeric literal inference | §4.4.3 lattice + fallback i32 / f64 | §4.4.3 |
| Function argument inference | per call site: unify formal param type with argument's solved type; coercions after unification | §4.4.2 |
| Return type inference | fn return types are closed: declared type unified with every return expression; closures may leave return type unannotated (inferred from body) | §4.4.2 |
| Generic inference | type args determined by unification at call/constructor sites; turbofish pins them explicitly | §4.4.2 |
| Closure parameter inference | closure params without annotations infer from first use within the closure's defining body; annotations pin | §4.4.2 |
| Closure return inference | inferred from body (single type; multiple divergent types unified with ! rule) | §4.4.2 |
| Trait constraint inference | obligations queued per use; solved §4.6.9 | §4.4.2 |
| Associated type inference | projection normalized when the impl is unique; ambiguity = E-P04-012 | §4.2.6 |
| Lifetime inference | region variables generated per borrow; solved by Volume 06 §9.5; this volume generates constraints only | §4.3.15 |
| Effect inference | row variables unified per SPEC-INDEX §5.3: join = ∪, requirement rows checked by subset | §4.0.1 T-Call |
| Const generic inference | const args must be determined by the call site (no fallback); literal args unified with param type | §4.4.2 |
| Ambiguity handling | no guessing: any unsolved TyVar, ambiguous projection, or ambiguous candidate is a compile-time rejection | §4.4.2 |
| Inference failure diagnostics | exact codes T04-12 | §4.4.6 |
| Explicit type constraints | type annotation `e: T` and turbofish `f::<T>()` assert types; ascription checks (coerces if allowed, else E-P04-001) | §4.4.2 |

### 4.4.5 Ambiguity policy

Edition P1 has no guessing: an inference problem with more than one solution in the stated lattice is rejected, never resolved by heuristic. This is a deliberate soundness-over-convenience settlement of design.md §6.4 "Ambiguity handling".

### 4.4.6 Diagnostics

T04-12 (inference diagnostics, complete):

| Code | Condition |
| ---- | --------- |
| E-P04-001 | type mismatch: two distinct canonical types unified |
| E-P04-002 | unsolved inference variable after fallback |
| E-P04-003 | trait obligation not satisfied (no candidate) |
| E-P04-004 | ambiguous trait candidate (multiple candidates) |
| E-P04-005 | orphan rule violation |
| E-P04-006 | overlapping impls |
| E-P04-007 | variance violation (unsound variance requested) |
| E-P04-008 | object-unsafe trait used as dyn |
| E-P04-009 | monomorphization depth > 32 |
| E-P04-010 | const generic argument mismatch |
| E-P04-011 | trait solver depth > 64 |
| E-P04-012 | associated type unbound/ambiguous |
| E-P04-013 | lifetime outlives violation (reported by Vol 06, generated here) |
| E-P04-014 | repr(P) aggregate passed across FFI boundary |
| E-P04-015 | overlapping specialization requested |
| E-P04-016 | DST in illegal position |
| E-P04-017 | Drop impl on Copy type |
| E-P04-018 | array type larger than MAX_ALLOC_SIZE |
| E-P04-019 | unprovable !-typed expression |
| E-P04-020 | duplicate enum discriminant |
| E-P04-021 | opaque return with multiple hidden types |
| E-P04-022 | partially applied generic used as value |
| E-P04-023 | reference-to-integer cast |
| E-P04-024 | cast between different fn-pointer ABIs |
| E-P04-025 | closure with captures coerced to fn pointer |
| E-P04-026 | user impl of Unsize |
| E-P04-027 | capability registered outside closed set (§4.7.9) |

---

## 4.5 Generic system

### 4.5.1 Parameter kinds

Four parameter kinds, closed in P1: lifetime ('a), type (T), const (const N: CT with CT ∈ {integer types, bool, char}), effect row (one anonymous row parameter ρ per item, always present, always last). Type parameters carry the implicit Sized bound unless relaxed with ?Sized.

### 4.5.2 Generic bounds

A bound is a trait ref `Trait⟨…⟩` applied to a parameter, a lifetime bound `'a: 'b` or `T: 'a`, or a const-expression bound (usage site requirement only). Bounds are obligations on the instantiation (checked at monomorphization) and grant capabilities inside the body (polymorphic use of trait methods).

### 4.5.3 Multiple bounds

Multiple bounds on one parameter are a conjunction: `T: A + B + 'c` requires every conjunct at instantiation. A parameter satisfying all conjuncts may use methods of all of them; the method-resolution candidate set is the union (ambiguity per §4.6.10).

### 4.5.4 Associated types and constants

Traits may declare associated types (exactly one impl per projection, §4.2.6) and associated constants (const C: CT with CT determined per impl). Associated items are accessed as ⟨T as Trait⟩::Assoc / ⟨T as Trait⟩::C.

### 4.5.5 Generic defaults

Type parameters on struct/enum/type-alias declarations may have defaults `T = D`; the default is used only when the parameter is omitted in a type position (never in expression-position inference). Functions, traits, and impls must not declare parameter defaults (rejection E-P04-028). Defaults must not reference later parameters.

### 4.5.6 Parameter ordering

Canonical order in every generic list: lifetimes first (declaration order), then type parameters (declaration order), then const parameters (declaration order), then the anonymous effect row (implicit, always last). Elided argument lists bind in canonical order. This order is part of instantiation identity (§4.2.6).

### 4.5.7 Generic inference and instantiation identity

Instantiation = full substitution of all four parameter kinds. Identity of an instance is Nom(DefId, σ) with σ canonical (§4.2.6). Effect row instantiation: the row parameter is instantiated with the caller-supplied row (or the declared row when omitted); row unification per SPEC-INDEX §5.3.

### 4.5.8 Recursive generic instantiation

Recursion depth is bounded: MONO_DEPTH_LIMIT = 32 (pinned, SPEC-INDEX §3). Depth is counted per instantiation chain (instance whose substitution references itself directly or transitively). Exceeding 32 ⇒ E-P04-009. Infinitely recursive instantiations (no base case) are rejected at the same limit, never by divergence.

### 4.5.9 Monomorphization and partial monomorphization

Monomorphization is the only specialization strategy (pinned): every executed code path corresponds to a fully substituted instance. Partial monomorphization: a generic item with undetermined parameters has no runtime value representation; naming it as a value is rejected (E-P04-022). Generic items are first-class only as fully instantiated values or through erased dispatch (§4.5.10).

### 4.5.10 Erased polymorphism

Erased polymorphism is dyn Trait (§4.9): one compiled body serving all instances behind vtable dispatch. The erased body is compiled from the trait's default-less method contracts; associated types must be bound at construction (§4.9.4).

### 4.5.11 Hybrid specialization

Hybrid specialization is defined as: the coexistence of (a) monomorphized instances for statically known instantiations and (b) an erased instance for dynamic dispatch, both derived from the same generic definition by substitution. There is no third mechanism; devirtualization of an erased call to a monomorphized body is permitted only when the receiver's dynamic type is provably unique at that program point (single-candidate proof; otherwise the erased body is kept). #[specialize]-style overlapping specialization does not exist in P1 (E-P04-015; coherence §4.6.8 governs).

### 4.5.12 Specialization legality and coherence

Specialization is by substitution only (pinned). Legality: an instance may be compiled iff its substitution satisfies all bounds. Coherence: impls must obey the orphan rule (§4.6.8) and must not overlap (§4.6.7); there is no attribute that permits overlapping. Semantics of every instance must equal the semantics of the generic definition under the substitution (obligation OB-04-6).

### 4.5.13 Code-size control

Controls, exact: (1) instance deduplication — instances with identical canonical identity share one body per linkage unit (Volume 17 defines symbol merging); (2) unreferenced instances are not emitted (garbage collection over the instance graph, roots = reachable from main and exported symbols); (3) the total instance count is bounded by TOTAL_INSTANCE_LIMIT, whose exact value is fixed in the Volume 12 limits table (the definition home of compiler limits); exceeding it is a defined compilation failure (E-P04-029) with the count reported; (4) MONO_DEPTH_LIMIT = 32 bounds per-chain depth (§4.5.8).

### 4.5.14 Instantiation caching

The instance cache maps CanonicalInstance → compiled body handle. Cache key = canonical identity (§4.2.6). Cache is content-addressed and deterministic (G14): identical (program, target, toolchain) inputs yield identical cache keys and identical bodies. Cache hits must not change semantics (the body is a function of the key alone). Persistence and invalidation rules are Volume 12 §19.

T04-7 assigns every §6.5 bullet to its anchor: Type parameters §4.5.1; Const parameters §4.5.1; Lifetime parameters §4.5.1; Effect parameters §4.5.1 (supported, pinned algebra); Generic bounds §4.5.2; Multiple bounds §4.5.3; Associated types §4.5.4; Associated constants §4.5.4; Generic defaults §4.5.5; Parameter ordering §4.5.6; Generic inference §4.5.7; Instantiation identity §4.5.7; Recursive generic instantiation §4.5.8; Monomorphization §4.5.9; Partial monomorphization §4.5.9; Erased polymorphism §4.5.10; Hybrid specialization §4.5.11; Specialization legality §4.5.12; Specialization coherence §4.5.12; Code-size control §4.5.13; Instantiation caching §4.5.14.

---

## 4.6 Trait system

### 4.6.1 Trait declarations

`trait Name⟨params⟩ : Superbounds { items }` declares a trait with DefId, its own generic parameters, supertrait bounds, associated types, associated constants, and methods. Trait methods take an implicit Self parameter of the receiver forms listed in §4.6.12. Traits are nominal; a trait ref is Trait(d, σ).

### 4.6.2 Required methods

A method without a body is required: every impl must provide it. A required method's signature is checked at each impl against the trait signature with Self and the impl's substitution applied (exact equality, §4.3.1).

### 4.6.3 Default methods

A method with a body is defaulted: impls may omit it; the default body is instantiated per impl (it is generic over the impl's substitution). Defaults may call required methods of the same trait. The default body is compiled per instance after substitution.

### 4.6.4 Associated types and constants (trait-level)

See §4.5.4. Every impl must bind every associated type without a default; an associated type with a default is bound by the default unless overridden.

### 4.6.5 Generic traits and supertraits

A trait may have its own generic parameters; Self is always the first parameter positionally. Supertrait bounds `trait A : B` require every impl of A to have an impl of B for the same Self and arguments; supertrait methods are callable on Self values inside A's default methods and inside dyn A (§4.6.13 prefix layout).

### 4.6.6 Trait implementation and blanket implementations

`impl⟨params⟩ Trait⟨…⟩ for T where WC { items }` — the impl header is (Trait(d, σT), Self = T). A blanket impl has a parameter as Self or as a free type parameter of the header (impl<T: Display> Display for T). Blanket impls are subject to the orphan rule and overlap rules like any impl.

### 4.6.7 Overlap detection

Two impls I1, I2 of the same trait overlap iff their headers unify (unification §4.3.2 with associated-type equality constraints respected) and the conjunction of both where-clauses is not provably unsatisfiable. Provably unsatisfiable = the obligation solver (§4.6.9, depth 64) derives a contradiction. If unification succeeds and unsatisfiability is not derived, the impls are rejected (E-P04-006). The check is semantic, per SPEC-INDEX §3 (overlap rejection is semantic).

### 4.6.8 Orphan rule and coherence

Orphan rule (pinned, exact): an impl is legal iff (a) the trait is local to the crate, or (b) the self type is local, or (c) at least one type parameter of the impl header is a local type that appears before any foreign type parameter in the header's type-parameter list. "Local" = declared in the current crate; a local type wrapped in a foreign generic constructor (Vec<Local>) counts per (c) only if the local type parameter appears before all foreign type parameters of the header. No fundamental-type escape hatch exists in edition P1 (pinned); #[fundamental] is not an attribute of P1. Violation ⇒ E-P04-005.

### 4.6.9 Trait selection and recursive solving

The solver resolves an obligation Σ ⊢ τ : Trait⟨…⟩:

1. Candidates: all impls in scope whose trait ref unifies with the obligation (§4.3.2); plus param-env candidates (bounds in Γ) and auto-trait derivations (§4.6.11).
2. Filter: candidates whose nested obligations cannot be solved (depth ≤ SOLVER_DEPTH_LIMIT = 64) are discarded; solver depth exhaustion ⇒ E-P04-011.
3. Unique survivor ⇒ obligation holds; zero survivors ⇒ E-P04-003; multiple ⇒ E-P04-004.
4. Results are memoized (cache key = canonical obligation). Cycle detection: an obligation being re-solved on its own stack is answered "cannot prove" (used to refute negative cycles), never diverges.

Termination: depth limit 64 bounds the proof tree; memoization bounds re-solving; total solver work is O(candidates × depth) per obligation, with the program-wide bound fixed by the Volume 12 limits table. Termination is an obligation (OB-04-10, G11).

### 4.6.10 Method resolution

Method resolution (receiver r of type T0):

1. Build the candidate receiver chain: R = [T0]; append Ti+1 = Deref target of Ti while a Deref impl applies; then append the chain of &T0's deref targets, then &mut T0's.
2. For each Rc in R in order, probe steps [Rc, &Rc, &mut Rc] in that order.
3. Within one probe step: inherent methods of Rc first, then trait methods from bounds in scope; collect all methods applicable with self type exactly the probe step.
4. First probe step with a non-empty candidate set selects; multiple candidates within the selecting step ⇒ E-P04-004; empty everywhere ⇒ E-P04-003 (reported with the probe trace).
5. Selected call is typed by T-Method: receiver coerced to the probe step form (autoref/deref inserted exactly once).

The algorithm is deterministic (G14) and total over the deref-closure of T0 (which is finite: the deref graph of a program is acyclic by WF checks; cycles in Deref impls are rejected, E-P04-030).

### 4.6.11 Auto traits

The auto traits are exactly: Send, Sync, Unpin, Freeze (closed set; adding an auto trait is an edition change). Derivation is structural: a nominal type derives A iff every field's type derives A (§4.7 tables give per-capability special rules). Auto obligations are solved by derivation, not by impl lookup; explicit unsafe impls may override derivation per §4.7.3.

### 4.6.12 Unsafe traits and negative implementations

An unsafe trait requires `unsafe impl` at every implementation; the implementor carries the trait's documented safety obligation (each unsafe trait's obligation is stated in its declaration; the obligation for the two unsafe traits in P1 core: allocator-provider trait (Volume 05 §5.11) and the pin-projection trait (Volume 06 §9.9)). Negative implementations: `impl !AutoTrait for T` is permitted only in the crate that owns the core/stdlib definitions of the auto traits (core and std crates); user crates must not write negative impls (rejection E-P04-031). A negative impl prunes candidates in §4.6.9 step 1 and blocks auto-derivation through fields of that type.

### 4.6.13 Object safety and vtable semantics

Object-safety predicate (a trait ref Trait⟨…⟩ is object-safe iff): (1) every method is dispatchable: no type-generic parameters (lifetime parameters allowed), receiver is &Self or &mut Self (with the method's own region), and Self appears in the signature only as the receiver; (2) the trait has no associated constants; (3) every associated type without default is bound at the dyn construction site (§4.9.4); (4) the trait ref's own type parameters are determined. Non-object-safe trait used as dyn ⇒ E-P04-008.

Vtable layout (exact, per trait ref instance): word 0: drop_fn: code pointer (drops the concrete type); word 1: size: u64; word 2: align: u64; words 3..: method slots. Slot order: methods of supertraits first, in supertrait declaration order, depth-first from the leftmost supertrait; then the trait's own methods in declaration order. Prefix property: the slots of any supertrait form a prefix of the subtrait's vtable; upcasting (§4.3.11) therefore needs no vtable reconstruction. Method slot i holds the code pointer of the instance's method implementation. All pointers are 8 bytes on both targets.

### 4.6.14 Dynamic dispatch

A call through a dyn receiver loads slot i of the vtable and calls it with (data pointer, args). Slot index i is computed statically from the method's position in the slot order (§4.6.13). Dispatch carries the effects of the target method; the caller's row must include the union of all rows of possible targets at the obligation (effect row of a dyn call = join over the object-safety-checked method rows, per SPEC-INDEX §5.3 join).

### 4.6.15 Trait upcasting

Upcasting dyn Sub + 'a ⇝ dyn Super + 'a is a coercion requiring Sub: Super (declared supertrait edge, direct or transitive). The vtable pointer is unchanged (prefix property); only the type changes. Upcasting to a non-supertrait is banned (E-P04-032).

### 4.6.16 Trait implementation visibility

Impls are visible where their trait and types resolve per Volume 03 visibility. An impl used for obligation solving must be in scope at the use site; impls are never "leaked" across crate boundaries except through re-exports of the trait or type (Volume 03 §4). Method visibility inside an impl follows the method's own visibility; a public method on a private type is callable only where the type is nameable.

T04-8 assigns every §6.6 bullet: Trait declarations §4.6.1; Required methods §4.6.2; Default methods §4.6.3; Associated types §4.6.4; Associated constants §4.6.4; Generic traits §4.6.5; Supertraits §4.6.5; Trait implementation §4.6.6; Blanket implementations §4.6.6; Orphan rules §4.6.8; Coherence §4.6.8; Overlap detection §4.6.7; Method resolution §4.6.10; Trait selection §4.6.9; Recursive trait solving §4.6.9; Auto traits §4.6.11; Unsafe traits §4.6.12; Negative implementations §4.6.12; Object safety §4.6.13; Dynamic dispatch §4.6.14; Trait upcasting §4.6.15; Vtable semantics §4.6.13; Trait implementation visibility §4.6.16.

---

## 4.7 Capabilities

The capability set is closed in edition P1: exactly Copy, Clone, Drop, Send, Sync, Sized, Unpin, Freeze (§4.7.9 settles "additional capabilities"). T04-9 defines every capability against all seven design.md aspects; column headers are the aspect bullets of design.md §6.7 lines 473–479.

T04-9:

| Capability | Semantic meaning | Automatic derivation | Explicit impl permission | Generic constraint behavior | Interactions | Unsafe impl obligation | Compiler enforcement |
| ---------- | ---------------- | -------------------- | ------------------------ | --------------------------- | ------------ | ---------------------- | -------------------- |
| Copy | value is bitwise-duplicable; reading a place does not move it | structurally: all fields Copy and no Drop impl (auto) | `impl Copy` permitted only for bitwise-copyable types (no interior mutability, no drop glue); `unsafe impl Copy` for unions with all-Copy fields | `T: Copy` enables copy-on-read in generic code; generic moves of Copy params are copies | excludes Drop (E-P04-017); implies Clone semantics (clone = copy) | for unions: every field must be Copy; copies of invalid representations raise ReprTrap not UB | moves lowered as copies; borrow checker treats reads as non-consuming (state machine SPEC-INDEX §5.4 Read from Owned stays Owned) |
| Clone | explicit deep duplicate via clone() | none automatic; `#[derive(Clone)]` expands to field-wise clone (each field must be Clone) | any crate owning the type (orphan rule); impl must return an independent value | `T: Clone` used by containers (e.g., collection growth) | requires no interaction with Drop correctness: cloned value owns its resources | an incorrect clone (aliasing internals) is a library-contract violation enforced by review + tests, not UB | none beyond method typing; derive expansion is exact field order |
| Drop | destructor: user code runs before field glue | none (absence of impl = no-op glue) | one impl per nominal type, in the owning crate; banned on Copy types (E-P04-017); banned on primitives, tuples, arrays, references, fn pointers | `T: Drop` is not a usable bound (Drop is not a trait users can bound on; the compiler treats drop-glue existence internally) | drop runs before field destruction (Volume 05 §5.4.3); drop of MaybeInit places uses flags (Volume 05 §5.2.12) | must not read moved-out fields; must not assume aliasing; double-drop prevented by init tracking | drop glue emitted per Volume 05 §5.4; drop effect rows propagate (Volume 05 §5.4.6) |
| Send | value may be transferred to another thread | structural: all fields Send; raw pointers and foreign unsized types are !Send (core negative impls); UnsafeCell<T> is Send iff T: Send | unsafe impl permitted with justification; negative impl only in core/std (E-P04-031) | `T: Send` required at thread spawn and channel transfer (Volume 08) | Send + Sync orthogonal; !Send types pin to the creating thread | implementor must prove no thread-unsafe state escapes | spawn/transfer APIs are typed to require Send (Volume 08 §12.2) |
| Sync | &T is shareable across threads | structural: all fields Sync; UnsafeCell<T> is Sync iff T: Send; raw pointers !Sync (core negative impls) | unsafe impl permitted with justification; negative impl core/std only | `T: Sync` required where &T crosses threads (Volume 08) | T: Sync does not imply T: Send (Rc-analog asymmetry) | implementor must prove safe concurrent &T access | same enforcement point as Send |
| Sized | size known at compile time | automatic for every type of known size; parameters carry implicit Sized unless ?Sized | never manually implemented (banned, E-P04-033); compiler-derived | `T: Sized` enables by-value use; `?Sized` restricts to pointer indirection | DST types are !Sized; Sized is prerequisite for Copy (a !Sized type is never Copy) | not applicable because Sized is compiler-derived, there is no user-level unsafe surface | WF checks: by-value positions require Sized (E-P04-016) |
| Unpin | value may be moved out of Pin without unsafe | structural: all fields Unpin; every primitive, reference, fn pointer is Unpin | unsafe impl permitted with justification (self-referential types opt out) | `T: Unpin` unlocks Pin::get_mut-style APIs (Volume 06 §9.9) | interacts with Drop: !Unpin + Drop must not move self | implementor must prove address-stability invariant is unneeded | Pin API typed on Unpin (Volume 06) |
| Freeze | no interior mutability anywhere in the value | structural: all fields Freeze and no UnsafeCell in the type graph | user impls banned (compiler-derived only, E-P04-034) | internal bound used by optimizer for read-caching legality (Volume 14 §21) | Freeze + shared reference = freely readable; used by alias model (Volume 05 §5.10) | not applicable because Freeze has no user unsafe surface by design | derivation is exact and closed; errors are compiler-internal (no user diagnostic) |

### 4.7.9 Additional capabilities

The bullet "Any additional ownership or representation capabilities" is settled: edition P1 adds no capability beyond the eight of T04-9. The capability namespace is closed; adding a capability is an edition change with the same process as adding an effect atom (SPEC-INDEX §5.3). Any attempt to register a ninth capability is rejected (E-P04-027); the closed-set check is enforced by the compiler's capability registry.

---

## 4.8 Variance

### 4.8.1 The three variances

For a type constructor F and parameter position p: p is covariant iff τ1 <: τ2 implies F[..τ1..] <: F[..τ2..]; contravariant iff τ2 <: τ1 implies F[..τ1..] <: F[..τ2..]; invariant iff F[..τ1..] <: F[..τ2..] only when τ1 = τ2 (canon equality). Variance applies to both lifetime and type parameters; the variance of a lifetime parameter describes how region ordering propagates.

### 4.8.2 Variance of each constructor

T04-10 (exact variance table; entries not listed are invariant):

| Constructor | Variance per position |
| ----------- | --------------------- |
| &'a T | 'a covariant; T covariant |
| &'a mut T | 'a covariant; T invariant |
| *const T / *mut T | T covariant (identity of referent unaffected: types carry no runtime witness) |
| fn(τ1…τn) ⇝ τr | each τi contravariant; τr covariant |
| [T], [T; N] | T covariant (slice only behind pointer; metadata unchanged) |
| tuple (…Ti…) | Ti covariant |
| struct/enum field of type τ | variance of τ's constructor applied to the field |
| dyn Trait + 'a | 'a covariant; trait type parameters per the trait's declared variances (computed like struct fields) |
| UnsafeCell<T> | T invariant |
| fn pointer type params of closures | capture positions: covariant unless captured by &mut (then invariant) |

### 4.8.3 Lifetime variance, function variance, mutable-reference variance, container variance

These bullets are the rows above applied: lifetime variance is row &'a T / &'a mut T; function variance is the fn row (arguments contravariant, return covariant); mutable-reference variance is invariance in T (S-Lif side condition); generic container variance is the computed per-field result for nominal types. Nominal variance is computed from fields; a type containing any invariant position is invariant in that position (composition: covariance composed with covariance is covariance; anything composed with invariance is invariance; contravariance composed with contravariance is covariance).

### 4.8.4 Trait object variance

dyn Trait subtyping follows the 'a covariance of the dyn constructor and the trait's own variances for its type parameters, computed from the trait's method signatures by the same rules (methods are fn positions: parameters contravariant in their types' variances, return covariant). The computed trait variances are part of trait identity metadata (deterministic, G14).

### 4.8.5 Variance inference

Variance is computed per declaration by fixed-point iteration over the type/trait graph:

1. Initialize every parameter covariant.
2. For each nominal type in dependency order: compute per-field variance composition; update parameters that occur under invariant or contravariant contexts.
3. Iterate to fix point over strongly connected components (SCCs of the dependency graph, computed by planck-graph-math, SPEC-INDEX §5.7).

Termination: each parameter's state moves along the chain Cov → Contra → Inv only toward Inv (a 3-value lattice, monotone updates); the number of updates is bounded by 3 × |params|; total complexity O(V + E) per SCC pass. Result is deterministic (G14). Measure and bound = obligation OB-04-9 (G11).

### 4.8.6 Variance validation

Declared or inferred variance is validated: a parameter used in a position requiring deeper invariance than computed is an error (E-P04-007). Validation examples with exact rules: a T inside &mut T forces T invariant; a T used as an array element inside a struct stored in a cell forces invariance through the cell. Validation is part of WF checking of every impl and instantiation (checked at §4.3.2 unification points).

---

## 4.9 Dynamic types

### 4.9.1 Trait object construction

Construction is the unsizing coercion §4.3.11: from a concrete T with T: Trait, T: Sized, T: 'a. The produced fat pointer is (data = address of the T value, vtable = VTrait⟨T⟩). Construction may be virtualized until materialization (Volume 05 §5.6.10).

### 4.9.2 Trait object layout

A dyn value behind a pointer occupies 16 bytes: 8-byte data pointer, 8-byte vtable pointer, align 8. The pointee layout is the concrete type's layout; the dyn pointer carries no type information beyond the vtable.

### 4.9.3 Vtables

Exact layout §4.6.13: [drop_fn, size, align, slots…]. One vtable instance exists per (trait ref, concrete type) pair per linkage unit; vtables are static data with program lifetime (Volume 05 §5.7 static storage).

### 4.9.4 Associated type bindings

At dyn construction, every trait associated type without a default must be bound: `dyn Trait<A = U>` syntax binds A := U. Unbound associated type in a dyn type ⇒ E-P04-012. A binding is part of dyn identity (§4.2.10).

### 4.9.5 Dynamic method dispatch

Exact rule §4.6.14: slot load, indirect call, effect-row join over targets. A call to a non-object-safe method through dyn is impossible by construction (E-P04-008 at the dyn site).

### 4.9.6 Fat pointers

A fat pointer is any pointer value with metadata: &[T] (len), *const [T]/*mut [T] (len), &[T; N] before coercion is thin, &dyn Trait (vtable), *const dyn Trait (vtable), pointers to DST structs (metadata of the tail). Fat pointer arithmetic scales by the element/tail stride (Volume 05 §5.8.9). Fat pointers are Copy; their metadata is part of value equivalence (Volume 05 §5.1.6).

### 4.9.7 Trait object ownership, lifetimes, auto trait combinations

A dyn value does not own its pointee; ownership remains with the original owner (Box/dyn combinations are library types, Volume 19). Object lifetime default: in a type position &'a dyn Trait + 'b, the object lifetime defaults to 'a unless explicitly bound; standalone dyn Trait in a where-clause defaults to 'static. Auto trait combinations: `dyn Trait + Send + Sync` is legal syntax; the auto set A ∈ P({Send, Sync, Unpin, Freeze}) is part of identity (§4.2.10) and is checked at construction: T must derive every auto trait in A (§4.3.11).

### 4.9.8 Dynamic conversion

Dynamic conversion is: concrete → dyn (§4.3.11), dyn Sub → dyn Super (§4.6.15). There is no downcast in edition P1 core (any_cast does not exist; reflection is §4.10.3 type identity only). A downcast attempt is a compile-time rejection (E-P04-035), not a runtime trap.

### 4.9.9 Object safety

The exact predicate is §4.6.13. This bullet and §6.6 "Object safety" are the same predicate; both bullets are satisfied by that single definition (one normative home, two ledger rows).

---

## 4.10 Type-level programming

### 4.10.1 Const generics

Const parameters (§4.5.1) may appear in array lengths, where-clauses, and associated constants. Const arguments are const-evaluated expressions (Volume 09 §14.1); their values participate in instance identity (§4.2.6). Const parameters are invariant (no subtyping over const values).

### 4.10.2 Compile-time type computations

The type-level computation language in P1 is exactly: trait projection normalization (§4.2.6) + const arithmetic in const contexts (Volume 09). Type-level functions defined as traits with associated types are permitted and evaluated by the solver with depth limit 64 (§4.6.9). Type-level lambdas, higher-kinded types, and variadic type lists must not exist (rejected by grammar; Volume 02 §2.5 has no such productions).

### 4.10.3 Type reflection

core::meta::type_id::<T>() -> TypeId where TypeId is a u128 computed as FNV-1a-128 over the canonical ASCII encoding of canon(T) (T: 'static required). Two distinct canonical types must produce distinct TypeIds; the obligation is OB-04-12 (Z3 over the bounded corpus + REVIEW of injectivity argument). TypeId is the only reflection primitive in P1; there is no runtime name table (name reflection is a Volume 09 const-fn facility operating on canonical strings, not runtime data).

### 4.10.4 Layout reflection

core::meta::layout_of::<T>() -> (u64, u64) (size, align) is a const fn requiring T: Sized; its results are exactly the Volume 05 §5.5 layout rules (obligation OB-05-3 covers the math). Layout reflection of a DST is rejected (E-P04-036).

### 4.10.5 Type predicates

Type predicates are trait bounds; there is no separate predicate language. The queryable set in P1: Copy, Clone, Send, Sync, Sized, Unpin, Freeze (§4.7) and any trait. Auto-trait predicates are answered by derivation (§4.6.11); others by the solver (§4.6.9).

### 4.10.6 Type-level constants

A const item or associated const usable in type position (array length, const generic argument) is a type-level constant. Its value is computed by Volume 09 §14.1 rules; values must be determined (no inference fallback) and are part of identity.

### 4.10.7 Compile-time trait queries

Trait solving in const context uses the same solver (§4.6.9) with the same depth limit; results are deterministic and cached (G14). A query evaluated in const context must resolve to a unique candidate; ambiguity ⇒ E-P04-004 at the const site.

### 4.10.8 Recursive type computations and termination

Recursive associated-type computations (trait-level fixpoints) terminate by: solver depth 64 (§4.6.9) + monomorphization depth 32 (§4.5.8). Exceeding either limit is a defined rejection (E-P04-011 / E-P04-009), never divergence (G11 obligations OB-04-10, OB-04-8).

### 4.10.9 Evaluation limits

The limits governing type-level computation, with definition homes: SOLVER_DEPTH_LIMIT = 64 (this volume, §4.6.9); MONO_DEPTH_LIMIT = 32 (this volume, pinned SPEC-INDEX §3); TOTAL_INSTANCE_LIMIT (Volume 12 limits table, referenced §4.5.13); CONST_EVAL_STEP_BUDGET and const-eval memory budget (Volume 09 §14.1, the definition home of const evaluation; Volume 12 tabulates the same values). Every limit is a named constant; no limit is implicit or unbounded.

### 4.10.10 Termination requirements

All type-level computation must terminate on every input accepted by the grammar. The termination argument per mechanism: unification (occurs check + node-count measure, §4.3.2); variance fixpoint (3-value lattice, §4.8.5); trait solving (depth 64 + memoization + cycle answer, §4.6.9); monomorphization (depth 32 + instance limit, §4.5); const evaluation (step budget, Volume 09). Each is an obligation row in the matrix (G11).

---

## 4.11 Feature template applications

### 4.11.1 dyn Trait (design.md §40 16-row template)

| Specification requirement | What must be established |
| ------------------------- | ------------------------ |
| 1. Syntax | `dyn Trait⟨Args⟩ + Autos + 'a` in type position; construction is coercion, no construction syntax | 
| 2. Static semantics | object-safety predicate §4.6.13; bound autos checked §4.3.11; bindings §4.9.4 |
| 3. Type semantics | input: any T with T: Trait + Sized + 'a; output: fat pointer type Dyn(…) |
| 4. Value semantics | value = (data pointer, vtable pointer); equivalence includes both words (Volume 05 §5.1.6) |
| 5. Evaluation semantics | coercion materializes vtable reference; dispatch loads slot (§4.6.14) |
| 6. Ownership | dyn does not own the pointee; moves of the fat pointer are pointer moves (Volume 05 §5.3) |
| 7. Effects | dispatch row = join over target rows (§4.6.14, SPEC-INDEX §5.3) |
| 8. Control flow | dispatch may panic only if the target panics; no extra paths |
| 9. Compile-time behavior | one erased body per trait ref (§4.5.10); vtables static (§4.9.3) |
| 10. Memory behavior | fat pointer 16 bytes; pointee layout unchanged (§4.9.2) |
| 11. ABI behavior | indirect call per Planck convention (Volume 10 §17.1); dyn across FFI banned (E-P04-014) |
| 12. IR 1 mapping | virtual call node with vtable operand (Volume 13 §20) |
| 13. IR 2 mapping | devirtualization only on single-candidate proof (§4.5.11; Volume 14 §21) |
| 14. Machine IR mapping | indirect call instruction, target from memory load (Volume 15 §22) |
| 15. Diagnostics | E-P04-008 (object safety), E-P04-012 (bindings), E-P04-014 (FFI) |
| 16. Validation | OB-04-13 (vtable layout EXH), OB-04-6 (dispatch semantics = static call semantics) |

### 4.11.2 Closures (design.md §40 16-row template)

| Specification requirement | What must be established |
| ------------------------- | ------------------------ |
| 1. Syntax | `|p1: τ1, …| -> τr body`; label-free; grammar Volume 02 §2.5 |
| 2. Static semantics | capture inference (Volume 06 §9.2); kind inference §4.1.17 |
| 3. Type semantics | unique nominal type (§4.2.9); implements Fn/FnMut/FnOnce per kind |
| 4. Value semantics | value = captured tuple; call applies the body with captured bindings |
| 5. Evaluation semantics | call evaluates body in captured environment; one call per FnOnce |
| 6. Ownership | captures moved or borrowed per capture mode; drop glue = captures' glue |
| 7. Effects | closure row = row of body; call rule T-Call applies |
| 8. Control flow | body control flow is the enclosing language's (Volume 07 §11) |
| 9. Compile-time behavior | inlining permitted by pass rules (Volume 14); no separate mono step unless generic |
| 10. Memory behavior | capture layout repr(P) in first-capture order (§4.1.17) |
| 11. ABI behavior | ∅-capture closure to fn pointer uses Planck convention (§4.3.10) |
| 12. IR 1 mapping | closure body inlined as a function node with captures as free variables (Volume 13 §20) |
| 13. IR 2 mapping | closure-to-call lowering (Volume 14 §21) |
| 14. Machine IR mapping | direct call after inlining, else pointer-call through capture struct (Volume 15 §22) |
| 15. Diagnostics | E-P04-025 (capture coercion), E-P04-001 (annotation mismatch) |
| 16. Validation | OB-04-14 (kind inference EXH over capture-use corpus), OB-04-1/2 (typing soundness) |

---

## Verification matrix

Methods per SPEC-INDEX §4: Z3 (bounded SMT, tools/fv/, 10 s timeout), EXH (exhaustive over the stated finite domain), PROP (deterministic property test paired with Z3/EXH), REVIEW (named invariant, reason not machine-checkable). Classes per §54: FV-0 machine-checked zero gaps; FV-1 bounded with documented gap + bridge; FV-2 tooling.

| ID | Property (exact) | Method | Class | Artifact | G |
| -- | ---------------- | ------ | ----- | -------- | - |
| OB-04-1 | Preservation: if Γ ⊢ e : τ with row ρ and e → e', then Γ' ⊢ e' : τ with row ρ' ⊆ ρ, where Γ' extends Γ with the regions still live after the step; trap and panic steps are permitted terminal transitions | REVIEW | FV-1 | reviewer checklist in this volume §4.0.1 + differential corpus tools/fv/typing/ | G3 |
| OB-04-2 | Progress: every closed, well-typed expression is a value, takes a step, raises a defined panic, or raises a defined trap (exit code 134); no third state exists | REVIEW | FV-1 | §4.0.1 + Volume 07 §11 step relation | G3 |
| OB-04-3 | Unification terminates with the most general unifier or Failure, on all inputs with node count ≤ 10^4 | Z3 | FV-1 | tools/fv/unify.z3 (bound 10^4 nodes; outside-bound bridge = runtime occurs-check) | G11 |
| OB-04-4 | Effect row laws: join ∪ associative/commutative/idempotent with identity ∅; meet ∩ distributes over join; row subtyping is a partial order — over the full 256-row lattice (2^8 atoms) | EXH | FV-0 | tools/fv/effect-rows.z3 + math/planck-effect-math | G3 |
| OB-04-5 | Variance computation: for all generated type graphs with ≤ 12 declarations, ≤ 4 fields each, ≤ 2 nesting levels (domain size ≤ 10^6), computed variances match the semantic definition (τ1 <: τ2 ⟹ F[τ1] <: F[τ2] per declared variance) | EXH | FV-0 | tools/fv/variance/ exhaustive generator | G3, G11 |
| OB-04-6 | Specialization-by-substitution: for every emitted instance G[σ], the semantics of the instance body equals the semantics of the generic body under σ (checked per instance pair by differential execution over the instance's test corpus) | PROP | FV-1 | tools/fv/mono-diff/ (PRNG xorshift64*, paired with REVIEW of substitution correctness) | G1 |
| OB-04-7 | Subtyping is reflexive and transitive over the canonical grammar restricted to lifetime/never differences (bounded: types of ≤ 6 constructors, ≤ 3 lifetimes: 10^6 domain) | Z3 | FV-0 | tools/fv/subtype.z3 | G3 |
| OB-04-8 | Monomorphization terminates: instance graph worklist processes each instance once; depth bounded by 32; total instances bounded by TOTAL_INSTANCE_LIMIT | REVIEW | FV-1 | §4.5.8, §4.5.13 (measure: instance count, decreasing worklist) | G11 |
| OB-04-9 | Variance fixpoint terminates: ≤ 3 × |params| updates, monotone Cov→Contra→Inv | REVIEW | FV-1 | §4.8.5 (lattice-height measure) | G11 |
| OB-04-10 | Trait solver terminates on every obligation: proof-tree depth ≤ 64, memoized nodes ≤ obligation count, cycle answers "cannot prove" | REVIEW | FV-1 | §4.6.9 (measure: depth × candidates) | G11 |
| OB-04-11 | Orphan rule: impl accepted iff (trait local) ∨ (self type local) ∨ (∃ local type parameter before any foreign type parameter); checked exhaustively over header shapes {local/foreign}^k for k ≤ 6 | EXH | FV-0 | tools/fv/orphan/ | G3 |
| OB-04-12 | TypeId injectivity: distinct canonical types in the corpus (10^6 generated canonical forms ≤ 12 constructors) yield distinct u128 FNV-1a-128 hashes | Z3 | FV-1 | tools/fv/typeid.z3 (bounded corpus; full injectivity is REVIEW: hash is not injective on u128, uniqueness is per-corpus with collision bridge = canonical string comparison on collision report) | G3 |
| OB-04-13 | Vtable layout: for every trait shape with ≤ 8 supertraits and ≤ 32 methods, slot order = depth-first supertrait order then declaration order; prefix property holds | EXH | FV-0 | tools/fv/vtable/ | G3, G1 |
| OB-04-14 | Closure kind inference: for all capture-use sequences over ≤ 3 captures × 4 use kinds (domain ≤ 10^6), inferred kind matches the §4.1.17 definition | EXH | FV-0 | tools/fv/closure-kind/ | G3 |
| OB-04-15 | Method resolution determinism: for all candidate sets of ≤ 32 methods × probe chains of ≤ 4 types, the selected method is unique and independent of candidate enumeration order | EXH | FV-0 | tools/fv/method-res/ | G14, G3 |
| OB-04-16 | Capability derivation consistency: structural derivation tables (T04-9) are closed under field composition for all type graphs of ≤ 8 fields (domain ≤ 10^6) | EXH | FV-0 | tools/fv/caps/ | G3 |

G-mappings used: G3 type soundness, G1 semantic preservation, G11 termination, G14 determinism. REVIEW rows state their exact invariant in the Property column; the FV-1 gap for REVIEW rows is the absence of a machine check, bridged by the named reviewer checklist plus the paired differential/EXH artifacts listed.

---

## Coverage ledger

Ledger of design.md §6 (lines 323–524). One row per bullet; anchors are sections/tables above.

### §6.1 Type categories (23 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Integer types | §4.1.2 T04-2; §4.1.1 row "Integer" |
| Floating-point types | §4.1.3 |
| Boolean | §4.1.4 |
| Character | §4.1.5 |
| Unit | §4.1.6 |
| Never | §4.1.7 |
| Tuples | §4.1.8 |
| Arrays | §4.1.9 |
| Slices | §4.1.10 |
| Structs | §4.1.11 |
| Enums | §4.1.12 |
| Unions | §4.1.13 |
| References | §4.1.14 |
| Raw pointers | §4.1.15 |
| Function pointers | §4.1.16 |
| Closures | §4.1.17; §4.11.2 |
| Trait objects | §4.1.18; §4.9; §4.11.1 |
| Opaque types | §4.1.19 |
| Generic types | §4.1.20; §4.5 |
| Dynamically sized types | §4.1.21 |
| SIMD types | §4.1.22 |
| Foreign types | §4.1.23 |
| Zero-sized types | §4.1.24 |

### §6.2 Type identity (11 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Nominal type identity | §4.2.2 |
| Structural type identity | §4.2.3 |
| Type aliases | §4.2.4 |
| Newtype identity | §4.2.5 |
| Generic type identity | §4.2.6 |
| Instantiated type identity | §4.2.6 |
| Associated type identity | §4.2.6 |
| Opaque type identity | §4.2.7 |
| Function type identity | §4.2.8 |
| Closure type identity | §4.2.9 |
| Dynamic type identity | §4.2.10 |

### §6.3 Type compatibility (15 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Exact type equality | §4.3.1 |
| Type unification | §4.3.2 |
| Coercion | §4.3.3 T04-4 |
| Implicit conversions | §4.3.4 |
| Explicit conversions | §4.3.5 T04-5 |
| Numeric conversions | §4.3.6 |
| Pointer conversions | §4.3.7 |
| Reference conversions | §4.3.8 |
| Array-to-slice conversions | §4.3.9 |
| Function coercions | §4.3.10 |
| Trait object coercions | §4.3.11 |
| Unsizing | §4.3.12 |
| Subtyping, if supported | §4.3.13 (supported: lifetime/never only) |
| Never-type coercion | §4.3.14 |
| Lifetime compatibility | §4.3.15 |

### §6.4 Type inference (15 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Local variable inference | §4.4.4 T04-6 row 1 |
| Numeric literal inference | §4.4.3 |
| Function argument inference | §4.4.4 T04-6 row 3 |
| Return type inference | §4.4.4 T04-6 row 4 |
| Generic inference | §4.4.4 T04-6 row 5 |
| Closure parameter inference | §4.4.4 T04-6 row 6 |
| Closure return inference | §4.4.4 T04-6 row 7 |
| Trait constraint inference | §4.4.4 T04-6 row 8; §4.6.9 |
| Associated type inference | §4.4.4 T04-6 row 9; §4.2.6 |
| Lifetime inference | §4.4.4 T04-6 row 10 |
| Effect inference | §4.4.4 T04-6 row 11; §4.0.1 T-Call |
| Const generic inference | §4.4.4 T04-6 row 12 |
| Ambiguity handling | §4.4.5 |
| Inference failure diagnostics | §4.4.6 T04-12 |
| Explicit type constraints | §4.4.4 T04-6 row 15 |

### §6.5 Generic system (21 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Type parameters | §4.5.1 |
| Const parameters | §4.5.1 |
| Lifetime parameters | §4.5.1 |
| Effect parameters, if supported | §4.5.1 (supported; pinned algebra) |
| Generic bounds | §4.5.2 |
| Multiple bounds | §4.5.3 |
| Associated types | §4.5.4 |
| Associated constants | §4.5.4 |
| Generic defaults | §4.5.5 |
| Parameter ordering | §4.5.6 |
| Generic inference | §4.5.7 |
| Instantiation identity | §4.5.7 |
| Recursive generic instantiation | §4.5.8 |
| Monomorphization | §4.5.9 |
| Partial monomorphization | §4.5.9 |
| Erased polymorphism | §4.5.10 |
| Hybrid specialization | §4.5.11 |
| Specialization legality | §4.5.12 |
| Specialization coherence | §4.5.12 |
| Code-size control | §4.5.13 |
| Instantiation caching | §4.5.14 |

### §6.6 Trait system (23 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Trait declarations | §4.6.1 |
| Required methods | §4.6.2 |
| Default methods | §4.6.3 |
| Associated types | §4.6.4 |
| Associated constants | §4.6.4 |
| Generic traits | §4.6.5 |
| Supertraits | §4.6.5 |
| Trait implementation | §4.6.6 |
| Blanket implementations | §4.6.6 |
| Orphan rules | §4.6.8 |
| Coherence | §4.6.8 |
| Overlap detection | §4.6.7 |
| Method resolution | §4.6.10 |
| Trait selection | §4.6.9 |
| Recursive trait solving | §4.6.9 |
| Auto traits | §4.6.11 |
| Unsafe traits | §4.6.12 |
| Negative implementations | §4.6.12 |
| Object safety | §4.6.13; §4.9.9 |
| Dynamic dispatch | §4.6.14 |
| Trait upcasting | §4.6.15 |
| Vtable semantics | §4.6.13 |
| Trait implementation visibility | §4.6.16 |

### §6.7 Capabilities (16 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Copy | §4.7 T04-9 row Copy |
| Clone | §4.7 T04-9 row Clone |
| Drop | §4.7 T04-9 row Drop |
| Send | §4.7 T04-9 row Send |
| Sync | §4.7 T04-9 row Sync |
| Sized | §4.7 T04-9 row Sized |
| Unpin | §4.7 T04-9 row Unpin |
| Freeze | §4.7 T04-9 row Freeze |
| Any additional ownership or representation capabilities | §4.7.9 (closed set, none in P1) |
| Semantic meaning | §4.7 T04-9 column "Semantic meaning" |
| Automatic derivation conditions | §4.7 T04-9 column "Automatic derivation" |
| Explicit implementation permissions | §4.7 T04-9 column "Explicit impl permission" |
| Generic constraint behavior | §4.7 T04-9 column "Generic constraint behavior" |
| Interaction with other capabilities | §4.7 T04-9 column "Interactions" |
| Unsafe implementation obligations | §4.7 T04-9 column "Unsafe impl obligation" |
| Compiler enforcement | §4.7 T04-9 column "Compiler enforcement" |

### §6.8 Variance (10 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Covariance | §4.8.1; §4.8.2 T04-10 |
| Contravariance | §4.8.1; §4.8.2 T04-10 |
| Invariance | §4.8.1; §4.8.2 T04-10 |
| Lifetime variance | §4.8.3 |
| Function variance | §4.8.3 |
| Mutable-reference variance | §4.8.3 |
| Generic container variance | §4.8.3 |
| Trait object variance | §4.8.4 |
| Inference | §4.8.5 |
| Variance validation | §4.8.6 |

### §6.9 Dynamic types (11 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Trait object construction | §4.9.1 |
| Trait object layout | §4.9.2 |
| Vtables | §4.9.3; §4.6.13 |
| Associated type bindings | §4.9.4 |
| Dynamic method dispatch | §4.9.5; §4.6.14 |
| Fat pointers | §4.9.6 |
| Trait object ownership | §4.9.7 |
| Trait object lifetimes | §4.9.7 |
| Auto trait combinations | §4.9.7 |
| Dynamic conversion | §4.9.8 |
| Object safety | §4.9.9; §4.6.13 |

### §6.10 Type-level programming (10 bullets)

| Bullet | Defined in |
| ------ | ---------- |
| Const generics | §4.10.1 |
| Compile-time type computations | §4.10.2 |
| Type reflection | §4.10.3 |
| Layout reflection | §4.10.4 |
| Type predicates | §4.10.5 |
| Type-level constants | §4.10.6 |
| Compile-time trait queries | §4.10.7 |
| Recursive type computations | §4.10.8 |
| Evaluation limits | §4.10.9 |
| Termination requirements | §4.10.10 |

bullets in scope: 155, defined: 155.
