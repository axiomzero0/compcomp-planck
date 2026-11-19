CEP:FILE: spec/volume-10-attributes-lowlevel.md
CEP:WHAT: Normative specification of Planck attributes and compiler directives, compiler intrinsics, inline assembly, SIMD, FFI/ABI, and linkage (design.md §15–§17).
CEP:WHY: design.md §15–§17 list 120 items requiring explicit definitions; the attribute set, intrinsic preconditions, assembly contract, and calling conventions are open policy questions until this volume settles them.
CEP:STATUS: complete
CEP:FAILURE: An attribute outside the closed registry, an intrinsic without a declared precondition, or an unspecified register assignment makes low-level code unverifiable and is a conformance failure of this volume.
CEP:ASSUMES: SPEC-INDEX.md §3 (UB model UB-5/UB-6; ABI policy x86-64 SysV + AArch64 AAPCS64) and §5 pinned contracts; Volume 04 (types); Volume 05 (layout math); Volume 09 (const evaluation, `#[init]` semantics); Volume 12 (mangling, diagnostics registry); Volume 15–17 (Machine IR, backend, objects).
CEP:COST: document; defines codegen-affecting contracts (register assignment, alignment) that are binding on the backend.
CEP:EVIDENCE: verification matrix OB-10-1..OB-10-12; coverage ledger (bullets in scope: 120, defined: 120).

---

## 10.0 Scope and traceability

This volume defines design.md §15 "Attributes and compiler directives" (lines 1299–1350, 34 bullets), §16 "Intrinsics and inline assembly" (lines 1351–1405, 42 bullets), and §17 "FFI and ABI" (lines 1406–1464, 44 bullets); total 120. Section mapping: 10.1↔§15.1, 10.2↔§15.2, 10.3↔§15.3, 10.4↔§15.4, 10.5↔§16.1, 10.6↔§16.2, 10.7↔§16.3, 10.8↔§17.1, 10.9↔§17.2, 10.10↔§17.3, 10.11↔§17.4. Design deliverables map as: Attribute Specification → §10.1–§10.4; Intrinsic Registry → §10.5; Assembly Contract → §10.6 (and §10.7 for vector lowering); ABI Specification → §10.8; FFI Safety Specification → §10.9–§10.11.

Settled policy decisions of this volume (normative; none deferrable):

| ID | Decision | Settled value |
| -- | -------- | ------------- |
| D1 | Attribute set | Closed registry of 26 built-in attributes (§10.1–§10.3); unknown attributes are compile-time rejections; library-defined attributes are banned. |
| D2 | Guarantee classes | Each attribute is classified **guarantee** (compiler must obey) or **advisory** (best effort, observable only through performance) in §10.2. |
| D3 | Intrinsic set | Closed registry of 31 declared entries in namespace `core::intrinsics`; declared-precondition violations are UB-6. |
| D4 | Inline assembly | Declared clobber/option contract; any violation of a declared list is UB-5; no labels, no unwinding in edition P1. |
| D5 | Planck fast convention | Fully specified in §10.8 for x86-64 and AArch64; C ABI at FFI boundaries is exactly SysV/AAPCS64. |
| D6 | SIMD | `#[repr(simd)]` user-defined vector structs; closed lane types and widths; scalar fallbacks are bit-exact. |

Diagnostic identifiers use block PLC-0500..PLC-0599 (registry: Volume 12 §12.5). CEP classes: intrinsic lowering tables and register-classification math are CEP-0 (hot compiler paths, embedded in the backend); attribute validation and FFI checking are CEP-1; DOT/registry dump tooling is CEP-2.

## 10.1 Layout attributes (design §15.1)

Registry rows (attribute | applies to | grammar | effect | class):

| Attribute | Applies to | Effect (exact) | Class |
| --------- | ---------- | -------------- | ----- |
| `#[repr(C)]` | structs, unions, enums | Field order = declaration order; layout = C struct layout computed by Volume 05 layout math (each field aligned to its alignment, size padded to max field alignment). On enums: discriminant type = 32-bit signed C int. | Guarantee |
| `#[repr(transparent)]` | single-field structs | The single non-zero-sized field must be the only field; size, alignment, and ABI classification of the wrapper equal the field's. | Guarantee |
| `#[repr(packed)]` | structs | Alignment of the struct and of every field = 1; no padding. Taking a reference to a field or a packed struct is a compile-time rejection PLC-0501; copy-out is permitted. | Guarantee |
| `#[repr(align(N))]` | structs, enums | N must be a power of two in 1..=4096; alignment of the type = max(N, natural); size rounded up to alignment. | Guarantee |
| `#[repr(simd)]` | single-array-field structs | Lane and width constraints of §10.7; size = lane_count × lane_size, alignment = size. | Guarantee |
| `#[repr(u8)]`…`#[repr(i64)]` (8 forms) | enums | Discriminant type = the named integer; variant values must fit; layout = that integer. | Guarantee |
| Enum representation (default) | enums | Without a repr attribute: smallest unsigned integer type (≥ 1 byte) that holds all discriminants; data-carrying variants use tagged layout per Volume 05. | Guarantee |
| Discriminant selection | enum variants | Explicit discriminant syntax `Variant = const-expr` (grammar: Volume 02/04); values must fit the repr type and be unique; default = previous + 1 starting at 0. Duplicate or out-of-range = PLC-0502. | Guarantee |

## 10.2 Optimization attributes (design §15.2)

| Attribute | Applies to | Effect | Class |
| --------- | ---------- | ------ | ----- |
| `#[inline]` | fns | Advisory: inlining is enabled at the inliner's discretion. | Advisory |
| `#[inline(never)]` | fns | Guarantee: the function is never inlined into any caller. | Guarantee |
| `#[inline(always)]` | fns | Guarantee: the function must be inlined into every caller; a recursive function marked `inline(always)` is rejected PLC-0503. | Guarantee |
| `#[cold]` | fns | Advisory: function is predicted rarely called; code placed out of line, branch weights lowered. | Advisory |
| `#[hot]` | fns | Advisory: function is hot; enables CEP-0 cost-measurement requirements (standards.md §5.1) and aggressive scheduling. | Advisory |
| `#[likely]` / `#[unlikely]` | `if`, `while`, `match` statements | Advisory: annotates the branch decision of the statement's top-level condition as predict-true / predict-false; lowered to Machine IR branch weights (Volume 15). Both on one statement = conflict PLC-0544. | Advisory |
| `#[optimize(size)]` / `#[optimize(speed)]` | fns | Advisory: per-function optimization bias overriding the global level for that function only. | Advisory |
| `#[target_feature(enable = "f1,f2")]` | fns | Guarantee about codegen: the named features (from the target feature table, Volume 16) are enabled for the function body. Call restriction: a caller must itself be compiled with a superset of the callee's features; otherwise PLC-0504. Unknown feature name = PLC-0505. | Guarantee |
| Link-time optimization controls | package level | No per-item attribute exists; LTO is controlled exclusively by the compiler flag `-C lto={off,thin,full}` (Volume 11 §18.1, Volume 20). An attempt to write an LTO attribute is rejected as an unknown attribute (PLC-0540). | Guarantee (flag semantics) |
| `#[used]` | statics | Guarantee: the static is kept in the generated object even if unreferenced (equivalent to `retain` linker semantics). | Guarantee |

Code-generation hints: the hints of this section (`cold`, `hot`, `likely`, `unlikely`, `optimize`) are exactly the code-generation hints of edition P1; no other hint attribute exists.

## 10.3 Semantic attributes (design §15.3)

| Attribute | Grammar | Effect |
| --------- | ------- | ------ |
| `#[derive(A, B)]` | list of trait names | Built-in derive only; A, B ∈ closed set {Copy, Clone, Debug, Default, PartialEq, Eq, Hash} (Volume 09 §9.4.4). Anything else = PLC-0425. |
| `#[must_use]` / `#[must_use = "msg"]` | optional string | A value of the attributed type returned but not used triggers warning PLC-0510 with the given message (default message if absent). |
| `#[deprecated(since = "X.Y.Z", note = "...")]` | strings | Use of the item triggers warning PLC-0511; `since` must parse as semver. |
| `#[diagnostic(on_unimplemented = "msg")]` | string | On traits: replaces the default "trait not implemented" message when a bound fails (used by trait-solver diagnostics, Volume 11 §18.9). |
| `#[cfg(...)]` | cfg expression | Conditional compilation. Keys (closed): {target_arch, target_os, target_endian, target_pointer_width, target_feature, feature, test, debug_assertions, panic}; `all(..)`, `any(..)`, `not(..)` combinators; values: target keys = target parameter strings (Volume 16), `feature` = declared feature name (Volume 20), `test`/`debug_assertions` = booleans, `panic` = {abort, unwind}. Unknown key = PLC-0512. A `cfg`-false item is dropped before its body is parsed; multiple `cfg` attributes on one item are a conjunction. |
| `#[test]` | fns | Registers the function in the test table (harness: Volume 26); `#[ignore]` and `#[should_panic(expected = "msg")]` modify execution semantics. |
| `#[link(name = "n", kind = "static|dylib")]` | strings | Adds a native library dependency to the link line (§10.11); unknown kind = PLC-0513. |
| `#[link_name = "sym"]` | string | Renames the imported symbol of an `extern` declaration. |
| `#[link_section = "name"]` | string | Places a static in the named object section; name must be a valid ELF section name. |
| `#[export_name = "sym"]` | string | Sets the exported symbol name verbatim (§10.8.9); conflicting with `#[no_mangle]` = PLC-0544. |
| `#[no_mangle]` | none | Symbol name = the item's source name, unmangled. |
| `#[init(path)]` | path | Runtime static initialization (semantics: Volume 09 §9.2.3); applies to statics only. |
| `#[thread_local]` | statics | Thread-local storage (semantics: Volume 09 §9.2.8; ABI: §10.8.8). |
| Initialization/Destruction beyond `#[init]` | — | No exit-time destruction exists (Volume 09 §9.2.7); no attribute for it is in the registry. |
| Safety requirements | — | No attribute exists; `unsafe` authority is carried by effect rows containing the `unsafe` atom (SPEC-INDEX §3, Volume 07) and checked by the frontend (Volume 11 §18.13). |

## 10.4 Attribute infrastructure (design §15.4)

**Attribute grammar** (exact productions; lexical classes from Volume 02):

```
Attribute   ::= "#" "[" AttrPath AttrArgs? "]"
AttrPath    ::= Ident ( "::" Ident )*
AttrArgs    ::= "(" ")" | "(" AttrValue ")" | "(" NameEq ")" | "(" AttrArgList ")"
AttrArgList ::= AttrArg "," AttrArg [ "," ]
AttrArg     ::= NameEq | AttrValue | Path
NameEq      ::= Ident "=" AttrValue
AttrValue   ::= StringLit | IntLit | BoolLit | Path
```

**Attribute argument types** are closed: string literal, integer literal (u64 range), boolean literal, path. Nesting beyond one AttrArgs level is a syntax rejection.

**Attribute validation** runs in fixed order, first failure reports: (V1) parse; (V2) registry lookup — unknown attribute is PLC-0540 (closed-set policy; namespaced tool attributes do not exist in P1); (V3) apply-target check — attribute permitted on this item kind is PLC-0541 when violated; (V4) argument check against the per-attribute grammar — PLC-0542; (V5) duplicate attribute on one item — PLC-0543; (V6) conflict check against the table below — PLC-0544; (V7) effect application.

**Attribute conflicts** (incompatible pairs; everything unlisted is compatible; same-attribute duplicates are handled at V5):

| Pair | Reason |
| ---- | ------ |
| `inline(always)` × `inline(never)` | contradictory |
| `inline` × `inline(never)` | contradictory |
| `inline` × `inline(always)` | duplicate class |
| `repr(packed)` × `repr(align(N))` | alignment contradiction |
| `repr(transparent)` × any other `repr(...)` | transparent is exclusive |
| `repr(simd)` × any other `repr(...)` | simd is exclusive |
| `repr(C)` × enum-integer repr (on enums) | discriminant type contradiction |
| two different enum-integer reprs | discriminant type contradiction |
| `cold` × `hot` | contradictory |
| `likely` × `unlikely` (same statement) | contradictory |
| `no_mangle` × `export_name` | symbol naming contradiction |

**Attribute expansion**: attributes never expand into code; they are resolved, checked, applied, and erased after the semantic passes. `cfg` attributes act at item-enumeration time (§10.3). **Stability**: the registry is versioned; additions are minor-version changes; removals or semantic changes are edition changes. **Built-in versus library-defined**: all 26 registry attributes are built-in; library-defined attributes are banned (consequence of the macro policy, Volume 09 §9.5.1).

## 10.5 Compiler intrinsics (design §16.1)

**Intrinsic namespace**: `core::intrinsics::`. The registry below is closed; adding an entry is an edition change. Calling an undeclared name in the namespace is an ordinary resolution error.

**Registry** (signature | type requirements | target requirements | semantic contract | const-eval | effect row | unsafe | lowering | fallback):

| Entry | Contract |
| ----- | -------- |
| `abort() -> !` | Defined trap: abort with exit code 134. Row ∅; safe; lowers to `MIR.abort`. Fallback: none needed. |
| `trap() -> !` | Defined trap (illegal-instruction abort, exit code 134). Row ∅; safe; `MIR.trap`. |
| `unreachable_unchecked() -> !` | Declared precondition: the call site is provably unreachable. Violation is **UB-6**. Unsafe; `MIR.unreachable`. |
| `copy_nonoverlapping::<T>(src: *const T, dst: *mut T, count: usize)` | Preconditions: source and destination regions disjoint or count = 0; both valid per UB-2; count × size_of::<T>() ≤ isize::MAX. Violation of a precondition is **UB-6**. T: Sized. Unsafe; row {unsafe}; `MIR.copy`; fallback: element loop. |
| `copy_overlapping::<T>(src, dst, count)` | As above without the disjointness precondition (memmove semantics). Unsafe; `MIR.memmove`; fallback: byte loop. |
| `write_bytes::<T>(dst: *mut T, byte: u8, count: usize)` | Precondition: dst valid for count × size_of::<T>(). Unsafe; `MIR.memset`; fallback: byte loop. |
| `load_volatile::<T>(p: *const T) -> T` / `store_volatile::<T>(p: *mut T, v: T)` | Precondition: p aligned to align_of::<T>() and valid. Unsafe; `MIR.vload`/`MIR.vstore`; not optimized away. |
| `ptr_offset_from::<T>(a: *const T, b: *const T) -> isize` | Precondition: same allocation, byte(a) ≥ byte(b), distance a multiple of size_of::<T>(). Violation = **UB-6**. Unsafe; `MIR.ptr_diff`. |
| `black_box::<T>(x: T) -> T` | Returns x; the compiler must not constant-fold, eliminate, or reorder observable operations across the value. Row {nondet}; safe; `MIR.blackbox`. |
| `assume(b: bool)` | Declared precondition: b is true at execution. Violation = **UB-6**. Unsafe; `MIR.assume`. |
| `bswap<W>`, `ctpop<W>`, `ctlz<W>`, `cttz<W>`, `ctlz_nonzero<W>`, `cttz_nonzero<W>`, `rotate_left<W>`, `rotate_right<W>` | W ∈ {u8,u16,u32,u64,i8,i16,i32,i64,usize,isize} (10 widths; 8 families = 80 instantiations). `ctlz_nonzero`/`cttz_nonzero` declared precondition: argument ≠ 0 (**UB-6**). Rows ∅; safe; const-evaluable per Volume 09 §9.1; lower to `MIR.bswap`, `MIR.cpop`, `MIR.clz`, `MIR.ctz`, `MIR.rotl`, `MIR.rotr`; fallbacks are shift/loop sequences with identical results. |
| `sqrt_f32/f64`, `abs_f32/f64`, `floor_f32/f64`, `ceil_f32/f64`, `trunc_f32/f64`, `round_ties_even_f32/f64` (12 entries) | IEEE-754 binary32/binary64 per lane, exact per-op semantics (G4). Rows ∅; safe; const-evaluable; lower to `MIR.fsqrt`, `MIR.fabs`, `MIR.ffloor`, `MIR.fceil`, `MIR.ftrunc`, `MIR.frintn`; fallbacks are software sequences producing bit-identical results. |

**Type requirements**: monomorphic instantiation only; no trait bounds; `T` parameters must be Sized. **Target requirements**: pointer entries require 64-bit pointers; math entries require IEEE-754 binary32/64 (both hold for the two initial targets). **Constant evaluation**: bit and math families are const-legal additions to the Volume 09 whitelist; all other entries are rejected in const context (Volume 09 §9.1.5). **Effect classification**: memory/pointer/control entries carry the `unsafe` atom; `black_box` carries `nondet`; math/bit entries carry ∅. **Unsafe requirements**: entries marked unsafe must be called inside `unsafe` authority (Volume 11 §18.13); calling outside is an ordinary rejection. **Fallback implementations**: every entry has a defined portable fallback on both initial targets; no intrinsic is optional.

## 10.6 Inline assembly (design §16.2)

**Assembly syntax** (exact):

```
AsmCall  ::= "asm" "!" "(" Template ("," AsmPiece)* [","] ")"
Template ::= StringLit
AsmPiece ::= Operand | Clobber | "options" "(" OptList ")"
Operand  ::= ("in" | "out" | "lateout" | "inlateout") "(" RegSpec ")" Expr ["=>" Expr]
RegSpec  ::= RegClass | '"' RegName '"'
Clobber  ::= "clobber" "(" '"' RegName '"' ")"
OptList  ::= Opt ("," Opt)* ;  Opt ∈ {nomem, readonly, preserves_flags, nostack, pure}
```

**Template substitutions**: `$N` (N ≥ 0) substitutes the operand with index N. No modifiers exist; `$N:` is rejected PLC-0550. **Input operands** (`in`) are read-only expressions; **output operands** (`out`, `lateout`) are places written by the block; `lateout` is assigned after all inputs are consumed; **in/out operands** (`inlateout`) read the input expression and write the output place, sharing one register. Operand kinds are closed: {in, out, lateout, inlateout}.

**Register classes** (per target; closed):

| Target | Classes | Named registers | Reserved (never operands or clobbers) |
| ------ | ------- | ---------------- | ------------------------------------- |
| x86-64 | `reg` (GPR), `reg_abcd` (RAX/RBX/RCX/RDX), `xmm` | "rax".."r15", "xmm0".."xmm15" | RSP, RBP (when frame pointer on) |
| AArch64 | `reg` (X0–X30), `v` (V0–V31) | "x0".."x30", "v0".."v31" | SP (X31), X29 (FP), X30 (LR), X18 (platform) |

**Fixed registers**: the reserved table above; using a reserved name is PLC-0551. **Clobber lists**: `clobber("name")` entries plus implicit clobbers from outputs; vocabulary = the target's named registers plus "memory". **Condition codes**: `preserves_flags` declares that CPU flags survive the block; otherwise flags are clobbered. **Memory effects**: by default the block may read and write memory and is a compiler memory barrier; `nomem` declares no memory access; `readonly` declares read-only memory access; both imply tighter optimization freedom. **Stack effects**: `nostack` declares the stack pointer is unchanged; asm that adjusts SP must restore it before block end and maintain 16-byte alignment. **Control-flow effects**: the block must not transfer control outside itself; templates contain no labels in edition P1 (PLC-0552). **Volatility**: an asm block is volatile (never eliminated, never duplicated with different results) unless options include {pure, nomem, readonly}, in which case it may be CSE'd or removed if results are unused. **Unwinding**: the block cannot unwind; no `may_unwind` form exists; an unwind escaping asm is UB-5. **Debug information**: the whole block carries its invocation span; per-template-line mapping does not exist in P1. **Target-specific constraints**: the constraint vocabulary is exactly the RegSpec table (class or named register); single-letter constraint strings are not in the grammar. **Compiler optimization barriers**: a block with default memory effects or `clobber("memory")` is a compiler memory barrier; memory operations must not be reordered across it; a {pure, nomem, readonly} block is not a barrier.

**UB-5 statement**: violating any declared list — clobbers, `nomem`, `readonly`, `preserves_flags`, `nostack`, `pure` — or the control-flow and unwind rules is undefined behavior category **UB-5** (SPEC-INDEX §3). The static checker must flag every statically detectable violation (OB-10-4); dynamically undetectable violations remain UB-5 by declaration.

## 10.7 SIMD (design §16.3)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Vector types | `#[repr(simd)]` structs containing exactly one array field; the layout is guarantee-level (§10.1). |
| Vector lane types | Closed set {u8, u16, u32, u64, i8, i16, i32, i64, f32, f64}. |
| Vector widths | Lane count N ∈ 1..=64 with N × sizeof(lane) ≤ 64 bytes; width = size in bytes. |
| Masks | Comparison results are mask vectors of the same lane width: true = all bits 1, false = all bits 0. |
| Arithmetic | Elementwise `+ - * / %`; integer overflow follows the operator-table class of the scalar op (Panic/Wrap/Unchecked); float ops are IEEE-754 per lane. |
| Comparisons | Elementwise `== != < <= > >=` producing masks; float comparisons are ordered per IEEE-754 (NaN compares false except `!=`). |
| Shuffles | `simd_shuffle(v1, v2, idx)` with const `idx: [usize; M]`, M ≤ 64, every index < 2N; out-of-range = PLC-0553. |
| Reductions | Closed set {sum, product, min, max, and, or, xor}; `and/or/xor` integer lanes only; float set = {sum, product, min, max}. |
| Memory loads | `simd_load::<V>(p)` (unaligned, total) and `simd_load_aligned::<V>(p)`; the aligned variant declares precondition alignment_of(V) = size(V); violation = **UB-6**. |
| Memory stores | `simd_store::<V>(p, v)` (unaligned) and `simd_store_aligned::<V>(p, v)` (aligned precondition as above). |
| Alignment | `#[repr(simd)]` alignment = vector size in bytes. |
| Masked operations | `simd_masked_load(mask, default, p)` returns lane i of `p` where mask lane i is all-ones, else lane i of `default`; `simd_masked_store(mask, p, v)` writes only active lanes. |
| Floating-point semantics | Per-lane IEEE-754 binary32/64; no fused multiply-add contraction in edition P1; every op is a distinct rounding step. |
| Saturating operations | `simd_saturating_add`, `simd_saturating_sub` for integer lane types only; saturation clamps to the lane type's min/max. |
| Target feature detection | Baseline: x86-64 = SSE2, AArch64 = NEON. Ops beyond baseline require the target feature set of Volume 16; compiling for a target lacking a required feature is PLC-0560. |
| Portable fallback behavior | Every SIMD op has a scalar-loop fallback producing bit-identical per-lane results (OB-10-10); the fallback is selected when vector lowering is unavailable for the target feature set. |

## 10.8 FFI and ABI (design §17.1)

### 10.8.1 Calling conventions
Two conventions exist. **planck-fast** is the default for all Planck-defined functions. **C** is the convention of every `extern "C"` boundary, with exactly the x86-64 System V ABI and the AArch64 AAPCS64 as published (normative summaries in §10.8.10). Any other convention string is a compile-time rejection PLC-0561. Both initial targets are declared and implemented; all other targets are declared, not implemented, in edition P1.

### 10.8.2 Argument classification (planck-fast)
Each argument is classified in order: (a) integer/pointer/bool/char scalar of size ≤ 8 bytes → one GPR; (b) float scalar of size 4 or 8 → one float register; (c) aggregate of size ≤ 16 bytes → split into two 8-byte fields in declaration order, each routed integer-kind → GPR, float-kind → float register; if the required register class is exhausted the whole argument passes on the stack (no per-field spill); (d) aggregate of size > 16 bytes → passed by pointer to a caller-allocated copy the callee treats as read-only. Classification is a total function (OB-10-6).

### 10.8.3 Register usage (planck-fast)

| Target | Argument GPRs | Argument FP regs | Callee-saved | Caller-saved |
| ------ | ------------- | ---------------- | ------------ | ------------ |
| x86-64 | RDI, RSI, RDX, RCX, R8, R9 | XMM0–XMM7 | RBX, RBP, R12–R15 | RAX, RCX, RDX, RSI, RDI, R8–R11, XMM0–XMM15 |
| AArch64 | X0–X7 | V0–V7 | X19–X28, X29 (FP), X30 (LR), V8–V15 (low 64 bits) | X0–X17, V0–V7, V16–V31 |

### 10.8.4 Stack layout and alignment
Stack-passed arguments occupy ascending addresses starting at SP+0 at function entry; the stack pointer is 16-byte aligned at every call instruction on both targets. There is no red zone in planck-fast frames. The caller cleans stack arguments. Alignment is a backend invariant (OB-10-9).

### 10.8.5 Structure passing and aggregate return
Aggregates ≤ 16 bytes are returned in registers by the classification of §10.8.2: integer-kind ≤ 8 → RAX/X0; ≤ 16 → RAX:RDX / X0:X1; float-kind → XMM0/V0 (≤ 16 → XMM0:XMM1 / V0:V1). Aggregates > 16 bytes use a hidden result pointer (first argument position; RDI/X0), the callee writes through it, and the callee returns that pointer in RAX/X0. Structure passing reuses §10.8.2 exactly.

### 10.8.6 Variadic arguments
planck-fast has no variadic form; a variadic Planck-defined function is PLC-0562. Variadic declarations exist only at `extern "C"` boundaries and follow the C ABI (§10.8.10).

### 10.8.7 Unwinding
Panics unwind through Planck frames only (Volume 07/18 landing pads). Unwinding across an `extern "C"` boundary is a compile-time rejection (SPEC-INDEX §3 destruction model; OB-10-11).

### 10.8.8 Thread-local ABI
`#[thread_local]` statics use ELF TLS: x86-64 FS-relative (initial-exec when the image is an executable or non-PIC; general-dynamic via `__tls_get_addr` otherwise); AArch64 TPIDR_EL0-relative with the same model choice. Access sequences are defined in Volume 15.

### 10.8.9 Symbol naming and export visibility
Planck symbols are mangled per Volume 12 §12.2; `#[no_mangle]` and `#[export_name]` override (§10.3). An item is exported iff it is `pub` and reachable from the crate root, or carries `#[no_mangle]`/`#[export_name]`; all other symbols get hidden visibility and internal linkage by default (also §10.11).

### 10.8.10 C ABI at FFI boundaries (normative summary)
x86-64 SysV: arguments classified INTEGER/SSE per eightbyte (8-byte chunks), first six INTEGER in RDI,RSI,RDX,RCX,R8,R9, first eight SSE in XMM0–XMM7; MEMORY class (aggregates > 16 bytes or register exhaustion) passes on the stack right-to-left with 8-byte slots; returns: INTEGER in RAX(+RDX), SSE in XMM0(+XMM1), MEMORY via hidden pointer; callee-saved RBX, RBP, R12–R15; 16-byte stack alignment at calls; 128-byte red zone applies inside C ABI frames. AArch64 AAPCS64: arguments in X0–X7 / V0–V7; HFAs (1–4 same-type float fields, ≤ 16 bytes) in consecutive V registers; composites ≤ 16 bytes in register pairs; larger composites by address; returns analogously; callee-saved X19–X28, X29, X30, V8–V15; 16-byte stack alignment. Any conflict between this summary and the published ABIs is resolved in favor of the published ABIs.

## 10.9 Foreign data (design §17.2)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| C-compatible integers | Planck integer types have fixed widths and map to C as: u8→unsigned char, u16→unsigned short, u32→unsigned int, u64→unsigned long long, i8→signed char, i16→short, i32→int, i64→long long, usize→size_t, isize→ptrdiff_t. The exact widths of C's own types per target are recorded in the Volume 16 target table. |
| Foreign floating-point types | f32 = IEEE-754 binary32, f64 = binary64; identical on both sides of the boundary. |
| Opaque types | `extern type T;` declares an unsized foreign type; it exists only behind pointers; dereferencing requires `unsafe` and is UB-2-checked like any typed place. |
| C-compatible structs | `#[repr(C)]` structs (§10.1) are C-interoperable by construction; layout math is Volume 05. |
| C-compatible unions | `#[repr(C)]` unions; writing any field is safe; reading a field is an `unsafe` operation (carries the `unsafe` atom) but is a defined reinterpretation, not UB (the UB list is closed). |
| Enum interoperability | Only enums with an integer repr (§10.1) cross the boundary; discriminant values are preserved exactly. |
| Function pointers | `extern "C" fn(τ...) -> ρ` pointer types; non-null by construction; nullable forms use `Option<extern "C" fn ...>`; calling through a wrong-ABI pointer is **UB-4**. |
| Variadic types | `...` permitted only as the last parameter of an `extern "C"` declaration; calling a variadic foreign function is an `unsafe` operation. |
| Foreign allocation | `core::ffi` exposes malloc/free wrappers; declared precondition of every free wrapper: the pointer was produced by the same allocator (**UB-6** if violated). |
| Foreign ownership | Ownership never transfers implicitly across the boundary; Planck-owned values are never freed by foreign code; foreign-owned memory is never dropped by Planck destructors; transfers are manual and unsafe. |

## 10.10 FFI safety (design §17.3)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Imported function safety | Every `extern` call carries the `unsafe` atom and requires `unsafe` authority (Volume 11 §18.13). |
| Exported function safety | An exported `extern "C" fn` must not unwind: the compiler emits abort-on-panic (a panic inside is the defined trap, exit code 134); parameter and return types must be C-ABI-safe (integer, float, `#[repr(C)]` aggregate, pointer, fn pointer), else PLC-0563. |
| Pointer validity | The caller guarantees every pointer argument is valid per UB-2 for the callee's declared access; every `extern` import carries the declared precondition "the foreign function accesses only memory it is permitted to access" — violation is **UB-6**. |
| Ownership crossing the boundary | Manual only (§10.9); ownership facts (Volume 11 §18.14) stop at the boundary and are not tracked across it. |
| Lifetime crossing | A reference with a lifetime shorter than `'static` may not appear in an extern signature: PLC-0564. Pointers carry no lifetime obligations and are the FFI currency. |
| Callback lifetime | A fn pointer passed to foreign code carries the declared precondition "the callee invokes it only while the pointer is valid"; violation is **UB-6**. |
| Callback threading | A callback may be invoked on any thread; its body must satisfy the Send discipline of Volume 08; accessing non-Send state from a callback is a data race = **UB-3**. |
| Panic crossing | Unwinding across the boundary is a compile-time rejection on the export side (§10.8.7) and impossible by contract on the import side. |
| Exception interoperability | Foreign exceptions must not cross Planck frames: Planck frames are emitted no-unwind (`.cantunwind` on AArch64; no LSDA on x86-64), so a foreign unwinder terminates the process as the defined trap (abort, exit code 134). |
| Foreign allocator pairing | Allocation and deallocation must use the same allocator (§10.9); the pairing rule is a declared precondition (**UB-6** on violation). |
| String interoperability | `core::ffi::CStr` = NUL-terminated byte string, interior NUL forbidden; `CString` = owned, NUL-terminated UTF-8 without interior NUL; construction from interior-NUL data is a defined error value, not UB. |
| ABI mismatch detection | Static: the compiler validates Planck-side signatures against the §10.8 classification tables. Runtime detection is out of scope; a mismatch between the declaration and the actual foreign function violates the import's declared precondition = **UB-6**. |

## 10.11 Linkage (design §17.4)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Static linkage | Default: all Planck crates are statically linked into the final image; duplicate monomorphized instances resolve by symbol identity (Volume 12 §12.2). |
| Dynamic linkage | `-C link=dylib` builds a cdylib; exported items per §10.8.9 get default visibility; all other symbols hidden. |
| Weak symbols | `#[linkage(weak)]` on `extern` imports and exported fns; an unresolved weak fn symbol reads as null; calling it is the defined trap (abort, exit code 134); weak/strong resolution follows the platform linker. |
| Symbol visibility | Default hidden; exported per §10.8.9; `#[link_section]` does not affect visibility. |
| Symbol versioning | Not supported in edition P1; any version-script or `#[link_version]` attempt is PLC-0565. |
| Import libraries | Not applicable to ELF targets; import-library inputs are rejected PLC-0566. |
| Export libraries | Artifact kinds: rlib (static object + metadata), staticlib (static archive), cdylib (dynamic object), exe (Volume 17 defines formats). |
| Foreign libraries | Native libraries are added from `#[link]` attributes (§10.3) plus the declarative build manifest (Volume 20); order = declaration order, tie-break ascending library name (deterministic, G14). |
| Runtime library dependencies | The runtime is libplanck_rt (Volume 18) plus the platform libc and libm on both initial targets; no other runtime dependency exists in edition P1. |

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-10-1 | Every accepted attribute parses per §10.4 grammar and every registry attribute has a defined apply rule. | EXH over attribute corpus | FV-0 | tests/vol10/attr-grammar | G3 |
| OB-10-2 | Every attribute pair is classified compatible/incompatible exactly per §10.4 conflict table. | EXH over registry pairs | FV-0 | tests/vol10/attr-conflicts | G3 |
| OB-10-3 | Every attribute outside the 26-entry registry rejects with PLC-0540. | EXH | FV-0 | tests/vol10/attr-unknown | G3 |
| OB-10-4 | Every statically detectable asm violation of a declared list is flagged; undetectable ones are UB-5 by declaration. | Z3 (bounded asm programs) + REVIEW | FV-1 | tools/fv/ob10-04.z3 | G2 |
| OB-10-5 | Every intrinsic precondition statically decidable is checked before lowering; each UB-6 declaration is listed in the registry. | EXH per intrinsic | FV-0 | tests/vol10/intrinsic-pre | G2 |
| OB-10-6 | Every argument and return value of a planck-fast call receives exactly one location per §10.8.2/§10.8.5. | Z3 over classification instances | FV-0 | tools/fv/ob10-06.z3 | G4 |
| OB-10-7 | SysV classification matches the published table for all type shapes ≤ 64 bytes. | EXH over generated shapes | FV-0 | tests/vol10/sysv | G24 |
| OB-10-8 | AAPCS64 classification matches the published table for all type shapes ≤ 64 bytes. | EXH over generated shapes | FV-0 | tests/vol10/aapcs64 | G24 |
| OB-10-9 | The stack pointer is 16-byte aligned at every emitted call site. | Z3 (frame invariant) + REVIEW | FV-1 | tools/fv/ob10-09.z3 | G4 |
| OB-10-10 | Every SIMD lowering and every portable fallback produce bit-identical lane results. | Z3 (bounded vectors: 4×i32, 2×f64, 16×i8) | FV-0 | tools/fv/ob10-10.z3 | G4 |
| OB-10-11 | Exported fns abort on panic; imported unwinding is rejected at compile time; foreign unwind ends in the defined trap. | EXH boundary corpus | FV-0 | tests/vol10/ffi-panic | G1 |
| OB-10-12 | Layout attribute effects agree with Volume 05 layout math on all shapes ≤ 4096-byte alignment. | EXH over generated shapes | FV-0 | tests/vol10/repr-layout | G3 |

## Coverage ledger

| Bullet (design.md §15–§17, abbreviated) | Defined in |
| --------------------------------------- | ---------- |
| `repr(C)` | §10.1 |
| `repr(transparent)` | §10.1 |
| `repr(packed)` | §10.1 |
| `repr(align(N))` | §10.1 |
| `repr(simd)` | §10.1, §10.7 |
| Enum representation | §10.1 |
| Discriminant selection | §10.1 |
| Inline hints | §10.2 |
| No-inline | §10.2 |
| Cold and hot hints | §10.2 |
| Branch prediction hints | §10.2 |
| Optimization level hints | §10.2 |
| Target features | §10.2 |
| Link-time optimization controls | §10.2 |
| Code-generation hints | §10.2 |
| Guarantee vs advisory classification | §10.2 (Class column) |
| Derive | §10.3 |
| Must-use | §10.3 |
| Deprecation | §10.3 |
| Diagnostics (`#[diagnostic]`) | §10.3 |
| Conditional compilation | §10.3 |
| Testing | §10.3 |
| Foreign linkage | §10.3 |
| Export control | §10.3 |
| Initialization | §10.3 |
| Destruction | §10.3 (no exit-time destruction) |
| Safety requirements | §10.3 (effect-row authority) |
| Attribute grammar | §10.4 |
| Attribute argument types | §10.4 |
| Attribute validation | §10.4 |
| Attribute expansion | §10.4 |
| Attribute conflicts | §10.4 |
| Unknown attributes | §10.4 |
| Stability | §10.4 |
| Built-in versus library-defined attributes | §10.4 |
| Intrinsic namespace | §10.5 |
| Type requirements | §10.5 |
| Target requirements | §10.5 |
| Semantic contracts | §10.5 |
| Constant evaluation (intrinsics) | §10.5 |
| Effect classification | §10.5 |
| Unsafe requirements | §10.5 |
| Lowering | §10.5 |
| Fallback implementations | §10.5 |
| Assembly syntax | §10.6 |
| Template substitutions | §10.6 |
| Input operands | §10.6 |
| Output operands | §10.6 |
| In/out operands | §10.6 |
| Register classes | §10.6 |
| Fixed registers | §10.6 |
| Clobber lists | §10.6 |
| Condition codes | §10.6 |
| Memory effects | §10.6 |
| Stack effects | §10.6 |
| Control-flow effects | §10.6 |
| Volatility | §10.6 |
| Unwinding (asm) | §10.6 |
| Debug information (asm) | §10.6 |
| Target-specific constraints | §10.6 |
| Compiler optimization barriers | §10.6 |
| Vector types | §10.7 |
| Vector lane types | §10.7 |
| Vector widths | §10.7 |
| Masks | §10.7 |
| Arithmetic (SIMD) | §10.7 |
| Comparisons (SIMD) | §10.7 |
| Shuffles | §10.7 |
| Reductions | §10.7 |
| Memory loads | §10.7 |
| Memory stores | §10.7 |
| Alignment (SIMD) | §10.7 |
| Masked operations | §10.7 |
| Floating-point semantics | §10.7 |
| Saturating operations | §10.7 |
| Target feature detection | §10.7 |
| Portable fallback behavior | §10.7 |
| Calling conventions | §10.8.1 |
| Argument classification | §10.8.2 |
| Return-value classification | §10.8.5 |
| Register usage | §10.8.3 |
| Stack layout | §10.8.4 |
| Alignment (ABI) | §10.8.4 |
| Structure passing | §10.8.2, §10.8.5 |
| Aggregate return | §10.8.5 |
| Variadic arguments | §10.8.6 |
| Unwinding (ABI) | §10.8.7 |
| Thread-local ABI | §10.8.8 |
| Symbol naming | §10.8.9 |
| Export visibility | §10.8.9 |
| C ABI summaries (SysV/AAPCS64) | §10.8.10 |
| C-compatible integers | §10.9 |
| Foreign floating-point types | §10.9 |
| Opaque types | §10.9 |
| C-compatible structs | §10.9 |
| C-compatible unions | §10.9 |
| Enum interoperability | §10.9 |
| Function pointers | §10.9 |
| Variadic types | §10.9 |
| Foreign allocation | §10.9 |
| Foreign ownership | §10.9 |
| Imported function safety | §10.10 |
| Exported function safety | §10.10 |
| Pointer validity | §10.10 |
| Ownership crossing the boundary | §10.10 |
| Lifetime crossing | §10.10 |
| Callback lifetime | §10.10 |
| Callback threading | §10.10 |
| Panic crossing | §10.10 |
| Exception interoperability | §10.10 |
| Foreign allocator pairing | §10.10 |
| String interoperability | §10.10 |
| ABI mismatch detection | §10.10 |
| Static linkage | §10.11 |
| Dynamic linkage | §10.11 |
| Weak symbols | §10.11 |
| Symbol visibility | §10.11 |
| Symbol versioning | §10.11 |
| Import libraries | §10.11 |
| Export libraries | §10.11 |
| Foreign libraries | §10.11 |
| Runtime library dependencies | §10.11 |

Note: the ledger contains 122 rows for 120 dash bullets; the two additional rows ("Guarantee vs advisory classification", "C ABI summaries") trace design.md prose requirements of §15.2 (line 1324) and §17.1 that are not dash bullets.

bullets in scope: 120, defined: 120
