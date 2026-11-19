CEP:FILE: spec/volume-09-compile-time.md
CEP:WHAT: Normative specification of Planck compile-time programming: constant evaluation, static initialization, reflection, generated code, macros, and compile-time dependencies (design.md §14).
CEP:WHY: design.md §14 lists 68 items that each require an explicit definition; without this volume, const evaluation has no legality or termination rules, and the macro system is an open policy question.
CEP:STATUS: complete
CEP:FAILURE: A compile-time construct without a defined legality rule, a missing termination bound, or an unspecified hygiene rule makes evaluation non-deterministic and is a conformance failure of this volume.
CEP:ASSUMES: SPEC-INDEX.md §3 settled decisions and §5 pinned contracts; Volume 02 (lexical and syntactic surface grammar); Volume 04 (type system); Volume 05 (value semantics); Volume 07 (effect algebra); Volume 10 (attributes, incl. `#[init]`); Volume 12 (limits table, diagnostics registry, incremental cache).
CEP:COST: document; the interpreter and expansion budgets it defines are binding compiler resource limits (G13).
CEP:EVIDENCE: verification matrix OB-09-1..OB-09-14; coverage ledger (bullets in scope: 68, defined: 68).

---

## 9.0 Scope and traceability

This volume defines design.md §14 "Compile-time programming" (lines 1200–1295) and nothing else. Bullet census: §14.1 = 14, §14.2 = 8, §14.3 = 10, §14.4 = 10, §14.5 = 17, §14.6 = 9; total 68. Section mapping: 9.1↔§14.1, 9.2↔§14.2, 9.3↔§14.3, 9.4↔§14.4, 9.5↔§14.5, 9.6↔§14.6. Design deliverables map as: Constant Evaluation Specification → §9.1–§9.2; Reflection Specification → §9.3; Macro System Specification → §9.4–§9.5; Compile-time Execution Contract → §9.1.11 and §9.6.

Settled policy decisions of this volume (normative, consistent with SPEC-INDEX §3; none is deferrable):

| ID | Decision | Settled value |
| -- | -------- | ------------- |
| D1 | Macro systems | Declarative token-tree macros and built-in compiler macros exist in edition P1; procedural, attribute, and user-defined derive macros are banned (§9.5). |
| D2 | Const-eval termination | Step budget of 10^7 interpreter steps per top-level evaluation request (§9.1.10). |
| D3 | Reflection | Closed capability set R1–R8; method reflection, function-signature reflection, and RTTI are banned (§9.3). |
| D4 | Code generation | Only declarative macro expansion and built-in derive generate code (§9.4). |
| D5 | Build scripts | Banned in edition P1; replaced by the declarative build manifest of Volume 20 (§9.6). |

Diagnostic identifiers used here are allocated from block PLC-0400..PLC-0499 of the registry defined in Volume 12 §12.5.

## 9.1 Constant evaluation (design §14.1)

### 9.1.1 Const-legality judgment

Two judgments are defined. **Const-legality** `e const-legal` is a syntax-directed predicate over the closed whitelist W1–W14 below; an expression that contains any construct outside W1–W14 or any entry of the forbidden list F is not const-legal. **Const evaluation** `C ⊢ e ⇓ v : τ` evaluates a const-legal expression `e` of type `τ` in context `C` (locals, symbolic allocation arena, budgets) to a promoted value `v`, or rejects compilation with a named error PLC-04xx. Every value `v` is a target-typed value as defined in Volume 05; the interpreter models target widths exactly (fidelity, SPEC-INDEX §1.4), never host widths.

Whitelist (closed; construct | rule):

| W | Construct | Rule |
| - | --------- | ---- |
| W1 | Integer, bool, char literals | Value per §5.6 of SPEC-INDEX; type by inference. |
| W2 | Float literals and float arithmetic | IEEE-754 binary32/binary64 per target, round-to-nearest ties-to-even; results are bit-exact and environment-independent. |
| W3 | Integer arithmetic `+ - * / % & | ^ << >>` | Two's complement. Overflow, division or modulo by zero, or shift distance ≥ bit width is a compile-time rejection (PLC-0410), never a wrapping runtime event. |
| W4 | Unary `!` `-`; comparisons `== != < <= > >=` | Semantics per the operator table of Volume 02 (SPEC-INDEX §5.5). |
| W5 | `&&` `||` | Short-circuit defined: right operand evaluated only when left operand permits. |
| W6 | Casts `as` | int→int truncates (two's complement); int→char requires value in 0..=0x10FFFF excluding D800..DFFF, else PLC-0409; float→int requires the value integral and representable, else PLC-0409; float→float rounds exactly once (no double rounding). |
| W7 | Aggregate construction | Tuples, arrays, repeat `[e; n]` with const `n: usize` (n may be 0), struct literals, enum variant construction. |
| W8 | Field and index access | `x.f`, `x.0`, `x[i]` with const index; out-of-bounds index is PLC-0411. |
| W9 | Shared borrow `&e` | Yields a symbolic pointer (AllocationId, byte offset) into the evaluation arena. Mutable borrows are forbidden. |
| W10 | `const` item references | Each use evaluates a fresh copy of the const's initializer (no shared state). |
| W11 | `const fn` calls | Callee body must itself be const-legal; recursion bounded per §9.1.9. |
| W12 | Control flow | `if`, `match`, `while`, `loop` with `break`/`continue`. `for` is forbidden (requires trait resolution). |
| W13 | Locals | `let` bindings, assignment to local variables; locals must not be borrowed. |
| W14 | Panics | Any reached `panic!` is a compile-time rejection PLC-0407 whose message is the diagnostic text. |

Forbidden list F (closed; presence of any member is PLC-0402): raw-pointer dereference; pointer arithmetic other than field-offset computation within one symbolic allocation; mutable references; `unsafe` blocks; trait-method calls; closures; `for`; atomics and thread operations; I/O and FFI operations; interior-mutable access; dynamic allocation that escapes evaluation (see §9.1.6).

### 9.1.2 `const` declarations and `const fn`

`const NAME: τ = e;` requires `e const-legal`, `C ⊢ e ⇓ v : τ`, and `v` promotable (§9.1.6); `τ` must satisfy StaticSafe (§9.2.1). `const fn f(...) -> τ { body }` is a function whose body must be const-legal; it may also be called at runtime with unchanged semantics (dual-use). In edition P1 a `const fn` must not be a trait method and must not be generic over lifetimes other than `'static`; both are compile-time rejections (PLC-0404).

### 9.1.3 Evaluation order

Operands evaluate left-to-right. Assignment evaluates the right operand before the left place. Function arguments evaluate left-to-right. Aggregate fields evaluate in declaration order. `match` arms are tested top-to-bottom. `&&`/`||` short-circuit per W5. This order is normative and observable through evaluation errors.

### 9.1.4 Permitted operations
The permitted set is exactly W1–W14. No other construct may be accepted; the whitelist is closed and adding an entry is an edition change.

### 9.1.5 Forbidden operations
The forbidden set is exactly F. Accepting any F entry is a conformance bug of the compiler, not a language option.

### 9.1.6 Pointer and allocation restrictions

Pointers inside evaluation are symbolic pairs (AllocationId, byte offset); absolute addresses never exist during evaluation and are never observable. Pointer arithmetic outside the owning allocation is PLC-0411. A value is **promotable** iff it is fixed-size, contains no interior mutability, transitively contains only promotable values, and its total size is ≤ MAX_PROMOTED_BYTES = 65536 bytes. The final result of a const evaluation must be promotable; otherwise PLC-0412, or PLC-0413 when the cause is the size limit. Promoted data is emitted as shared, read-only, effectively `'static` data (Volume 17 placement).

### 9.1.7 Allocation restrictions

The interpreter allocates in a per-request arena bounded by MAX_CONST_MEM_CELLS = 2^20 eight-byte words (8 MiB). Cyclic allocations (a promoted value referencing itself) are PLC-0412. No allocation outlives its evaluation request.

### 9.1.8 Effects during evaluation

The effect row of every evaluated expression must be ∅ (SPEC-INDEX §5.3 atom set). The interpreter has no host effects: it executes no syscalls, no I/O, no clock reads, and no randomness (structural sandbox, §9.6). Reaching a `panic` is PLC-0407 (W14); it is never runtime behavior.

### 9.1.9 Recursion

Recursion is permitted through `const fn` and `match`. The call-depth limit is MAX_CONST_CALL_DEPTH = 512; exceeding it is PLC-0414.

### 9.1.10 Termination

Termination is enforced by a step budget: MAX_CONST_STEPS = 10^7 interpreter steps per top-level evaluation request. Every interpreter transition (arithmetic step, control transfer, allocation, copy) strictly decrements the remaining-step counter; the counter is a well-founded termination measure (OB-09-2, G11). Exceeding the budget is PLC-0415. No const evaluation may run unbounded; there is no "infinite loop in const context" outcome other than this rejection.

### 9.1.11 Evaluation resource limits

| Constant | Value | Rejection |
| -------- | ----- | --------- |
| MAX_CONST_STEPS | 10^7 steps per request | PLC-0415 |
| MAX_CONST_CALL_DEPTH | 512 frames | PLC-0414 |
| MAX_CONST_MEM_CELLS | 2^20 words (8 MiB) | PLC-0416 |
| MAX_PROMOTED_BYTES | 65536 bytes per promoted value | PLC-0413 |
| CONST_ARENA_LIMIT | 2^28 bytes (256 MiB) per crate | PLC-0416 |

All five are rows L07–L09, L27, and L30 of the Volume 12 limits table; the wall-clock guard is the Volume 12 limit L18 (600 s per crate), not a separate const-specific timer.

### 9.1.12 Diagnostics

Const-evaluation errors use block PLC-0400..PLC-0499 (registry: Volume 12 §12.5). Fixed assignments: PLC-0401 integer-literal overflow; PLC-0402 forbidden operation; PLC-0403 non-const construct in const context; PLC-0404 `const fn` structural violation; PLC-0405 static not StaticSafe; PLC-0406 static requires runtime init without `#[init]`; PLC-0407 panic reached during evaluation; PLC-0408 type mismatch in const context; PLC-0409 cast out of range; PLC-0410 undefined arithmetic (overflow, div/mod by zero, bad shift); PLC-0411 index or offset out of bounds; PLC-0412 non-promotable or cyclic value; PLC-0413 promotion size limit exceeded; PLC-0414 call depth exceeded; PLC-0415 step budget exceeded; PLC-0416 memory limit exceeded. Each carries the primary span of the failing subexpression and the evaluation stack (frames, innermost first, depth ≤ 512).

### 9.1.13 Constant caching

A const-evaluation result is cached under key K = (DefId of the const, canonical generic substitution, target triple, content-hash chain of the body and of transitively referenced consts, env-record when `env!`/`option_env!` participated). The env-record is the sorted list of (name, value) pairs of environment variables read. The hash is FNV-1a-64 over the canonical serialization of K (hash definition: Volume 12 §12.1). A cache entry is reused only on exact key equality; any change recomputes (G23; invalidation contract Volume 12 §12.10). Cache reuse must not alter semantics: the cached value is the promoted byte image plus its type.

## 9.2 Static initialization (design §14.2)

### 9.2.1 Static values

`static S: τ = e;` requires τ to satisfy **StaticSafe**: τ is composed only of Copy types without `Drop` impls, atomic types, `&'static T`, and arrays/tuples/structs of such types; interior mutability is permitted only in atomic types. Violation is PLC-0405. Statics are immutable in const contexts (F).

### 9.2.2 Constant initialization

If `C ⊢ e ⇓ v : τ` and `v` is promotable, S is **const-initialized**: its image is placed in read-only data at link time (Volume 17) and no runtime initializer exists.

### 9.2.3 Runtime initialization, if supported

Runtime initialization is supported only through the `#[init(path)]` attribute (grammar and effects: Volume 10 §10.3). Requirements, all compile-time checked: `path` is a function `fn()` with effect row ⊆ {alloc}; the static's type satisfies **Zeroable** (composed of integers, floats, bools, atomics, arrays, tuples, structs of those; no references, no enums, no pointers); the image is all-zero bits. The generated initializer runs before `main` (§9.2.4). A static that is neither const-initializable nor `#[init]`-annotated is PLC-0406.

### 9.2.4 Initialization order

Runtime initializers run in dependency order of the **static dependency graph**: nodes = `#[init]` statics and statics referenced by initializers; edges = data dependency (initializer reads or takes the address of another static). The order is a topological order computed by Kahn's algorithm over `planck-graph-math` with tie-break by ascending symbol ID (deterministic, G14; OB-09-7). Const-initialized statics have no runtime initializer and impose no ordering edges.

### 9.2.5 Dependency cycles

Any cycle among `#[init]` statics in the static dependency graph is a compile-time rejection PLC-0418 listing the cycle as a node path. Cycles among const-initialized statics are permitted (no runtime work exists to order).

### 9.2.6 Initialization failure

An initializer cannot fail: its effect row excludes `panic`, and if a panic nonetheless occurs during startup, it is the defined trap (abort, exit code 134) at process start, before `main`; no partially initialized static is observable (Volume 18 startup sequence).

### 9.2.7 Destructor behavior

Statics have no destructors in edition P1: StaticSafe excludes `Drop` impls, exit-time destruction does not exist, and memory is reclaimed by the operating system at process exit. There is no attribute for exit-time destruction; the design bullet "Destruction" is settled by this ban, and any such attribute is rejected as an unknown attribute (Volume 10 §10.4).

### 9.2.8 Thread-local initialization

`#[thread_local]` statics must be const-initialized (§9.2.2); otherwise PLC-0419. Lazy initialization with guard variables is banned (Law 1: guard variables are hidden cost). TLS access ABI is Volume 10 §10.8 (thread-local ABI).

## 9.3 Reflection (design §14.3)

### 9.3.1 Closed capability set

Reflection consists of exactly the capabilities R1–R8. Anything not listed is banned; in particular **method reflection**, **function-signature reflection**, and runtime type information (RTTI) are banned (PLC-0420). Adding a capability is an edition change.

| ID | Capability | Exact form |
| -- | ---------- | ---------- |
| R1 | Type reflection | `TypeOf<T>::NAME: &'static str`, `TypeOf<T>::SIZE: usize`, `TypeOf<T>::ALIGN: usize` (const items). |
| R2 | Field reflection | For structs with `#[repr(P)]` or `#[repr(C)]`: field name, field type, and byte offset per field, in declaration order. |
| R3 | Trait reflection | `Implements<T, TraitName>: bool` for any trait visible at the query site; associated-type projection query for the same traits. |
| R4 | Enum reflection | For `#[repr(u8|u16|u32|u64|i8|i16|i32|i64)]` enums: variant names, discriminant values, per-variant field types, in declaration order. |
| R5 | Layout reflection | Const fns `size_of::<T>()`, `align_of::<T>()`, `offset_of!(T.f)` with the exact layout math of Volume 05. |
| R6 | Generic reflection | Name and type of each concrete generic argument of a fully substituted instance. |
| R7 | Const-generic reflection | Value of each const-generic argument of a fully substituted instance. |
| R8 | Compile-time queries | Target triple, pointer width, endianness, enabled target feature set, compiler version string, current crate name and version (Volume 16 defines the target parameter values). |

### 9.3.2 Visibility restrictions

Reflection observes only items visible at the query site under the name-resolution rules of Volume 03; in particular private fields of foreign types are not reflected and foreign crates are reflected only through their public surface.

### 9.3.3 Reflection stability

Results are stable within one compiler version. Ordering of reflected lists is declaration order (deterministic, G14). Cross-version stability is not guaranteed: adding a field, reordering variants, or changing layout may change reflection results, and this is a minor-version (not patch-version) event. Reflection results are not part of the stable ABI contract (Volume 10 §10.8).

### 9.3.4 Runtime metadata policy

Metadata is emitted only for types reachable from a reflection query used in the crate; unreachable metadata is stripped. No RTTI tables exist in generated objects unless a query requires them (Law 1: RTTI is hidden cost). The metadata record format is defined in Volume 17; the query-to-record mapping is: R1–R5 from the target's layout math, R3 from the impl registry of Volume 12 §12.2.

## 9.4 Generated code (design §14.4)

### 9.4.1 Compile-time code generation
The only code-generation mechanisms in edition P1 are declarative macro expansion (§9.5) and built-in derive (§9.4.4). Every other mechanism — procedural generation, source printing, compiler plugins — is banned (PLC-0421).

### 9.4.2 Generated declarations
A macro expansion may produce item declarations (fn, struct, enum, union, trait, impl, type alias, const, static, use, macro). Generated names follow the hygiene rule of §9.5.5; item-level names resolve at the invocation site so that surrounding code can reference them.

### 9.4.3 Type generation
Generated type declarations participate in coherence as local types of the crate containing the invocation (orphan rule, SPEC-INDEX §3). A generated type must be well-formed under the Volume 04 rules; the checker treats it exactly like source-written types.

### 9.4.4 Implementation generation
Built-in `#[derive(...)]` generates implementations for exactly the closed trait set {Copy, Clone, Debug, Default, PartialEq, Eq, Hash}. The expansion is the compiler's fixed per-trait algorithm over the type's fields in declaration order; user-defined derive macros are banned (§9.5). If a listed trait has no corresponding stdlib definition (Volume 19), the derive is a compile-time rejection PLC-0422.

### 9.4.5 Syntax tree construction
Generators operate on token trees (§9.5.3) only; no generator constructs or mutates AST nodes directly. The parser builds the AST of the expansion output like any other source.

### 9.4.6 Generated source locations
Every token produced by an expansion carries the span of the macro invocation; constructs internal to the macro body carry a synthetic span whose SourceId identifies the expansion (SourceId scheme: Volume 11 §18.2). Generated nodes are traceable to (invocation span, expansion depth, macro DefId).

### 9.4.7 Diagnostics
Errors inside generated code report the generated-code span and attach the macro invocation span and the expansion traceback (Volume 12 §12.5). Error codes come from the block of the failing semantic phase; the traceback is always attached when depth ≥ 1.

### 9.4.8 Hygiene
Generated code follows the mixed-site hygiene rule of §9.5.5 exactly; there is no separate hygiene policy for generated items versus macros.

### 9.4.9 Generated-code safety
Generated code passes the complete frontend pipeline (resolution, types, ownership, effects, borrow check); no mechanism may bypass a check for generated code. Derive-generated impls must satisfy the trait contract of Volume 19 for the derived trait; a derive on a type shape for which the algorithm is undefined (e.g., `Copy` on a type with a `Drop` field) is a compile-time rejection PLC-0422.

### 9.4.10 Determinism
Expansion is a pure function of (macro definition, input token trees, invocation span, macro registry version). Identical inputs produce byte-identical expansions (OB-09-10, G14). No generator may read the environment, the clock, the filesystem, or randomness; `env!`/`option_env!` inside macro bodies are ordinary const-context built-ins recorded per §9.6.7.

## 9.5 Macros (design §14.5)

### 9.5.1 Macro system selection (settled)

The design requires an explicit choice of macro systems. The choice, normative and closed:

| Category | Status in edition P1 | Effect |
| -------- | -------------------- | ------ |
| Declarative macros (token-tree) | Supported | §9.5.2–§9.5.12 define semantics exactly. |
| Procedural macros | Banned | Any `proc_macro`-style definition or use is PLC-0423. Rationale: arbitrary code execution at compile time violates the sandbox contract of §9.6 and determinism (G14). |
| Attribute macros | Banned | Any attribute invoking a macro is PLC-0424; attributes are the closed registry of Volume 10. |
| Derive macros | Banned (user-defined) | User derive is PLC-0425; the built-in derive of §9.4.4 remains. |
| Built-in compiler macros | Supported | Closed set of 13 (§9.5.2). |

### 9.5.2 Built-in compiler macros

Closed set, exact effects:

| Macro | Effect |
| ----- | ------ |
| `stringify!(tt)` | Produces a string literal of the input tokens in source spelling. |
| `concat!(lit, ...)` | Concatenates ≥ 1 string literals. |
| `line!()`, `column!()`, `file!()` | Invocation line (u32), column (u32, 1-based), file path (string). |
| `module_path!()` | Current module path as a string. |
| `env!("NAME")` | Value of environment variable NAME; unset is PLC-0426. |
| `option_env!("NAME")` | `Some("v")`/`None` const of the variable; never errors. |
| `include!("path")` | Parses the file's contents as items in place (registered SourceId, §9.6.2). |
| `include_str!("path")` / `include_bytes!("path")` | File contents as string literal / byte-string literal. |
| `compile_error!("msg")` | Reports error PLC-0427 with message `msg` at the invocation span. |
| `cfg!(key)` / `cfg!(all(..))` / `cfg!(any(..))` / `cfg!(not(..))` | Boolean const of the cfg predicate (cfg grammar: Volume 10 §10.3). |

### 9.5.3 Invocation syntax and expansion inputs
Invocation syntax is `name ! ( tt )`, `name ! [ tt ]`, or `name ! { tt }`, where `tt` is one token tree: a single token or a delimiter-balanced group. The invocation delimiter need not match the definition delimiter. Inputs are token trees, not AST; inputs are immutable; a fragment transmitted into an expansion is spliced without re-lexing.

### 9.5.4 Expansion outputs and timing
Outputs are token trees spliced at the invocation site and parsed as part of the enclosing item. Expansion runs after parsing of the containing item and before resolution; order is outside-in (an invocation is expanded before any invocation contained in its expansion). Fixpoint = no remaining invocations. Fragment specifiers are closed: {expr, ty, pat, stmt, block, item, ident, literal, lifetime, path, tt, meta}; a fragment's grammar is the corresponding Volume 02/Vol 11 AST production; mismatch is PLC-0428.

### 9.5.5 Hygiene (exact rule)
Every identifier carries a hygiene context: DefSite(expansion_id) or CallSite. Rule H: (a) identifiers written in the macro body that are not transmitted fragments resolve at the definition site for locals and labels; (b) item-level names introduced by an expansion resolve at the invocation site; (c) identifiers transmitted through fragment inputs resolve at their original site; (d) hygienic locals are interned as (expansion_id, name) pairs (interner: Volume 12 §12.1). Identical programs differing only in hygiene context may resolve differently; this is the hygiene contract (OB-09-8).

### 9.5.6 Name resolution
Macros are declared as items (`macro name(pattern) { expansion }`; surface grammar per Volume 02) and live in the macro namespace of their module with normal visibility rules; `pub macro` exports through crate metadata; `use path::name;` imports into the macro namespace. Macro definitions resolve at their definition site before any expansion in the same module.

### 9.5.7 Recursion
Recursion is permitted with the limits MAX_MACRO_DEPTH = 128 nested expansion levels (shared with §9.4.6 expansion depth) and MAX_MACRO_TOKENS = 2^20 tokens emitted per crate. Loop detection: if the pair (macro DefId, input token-tree sequence) repeats within the current expansion chain, expansion halts with PLC-0429. Exceeding depth is PLC-0430; exceeding tokens is PLC-0431.

### 9.5.8 Error diagnostics
`compile_error!("msg")` produces PLC-0427. Parse and semantic errors inside an expansion report the generated-code span plus the invocation span and traceback (§9.4.7). Macro-definition pattern errors are reported at definition time, before any invocation.

### 9.5.9 Build reproducibility
Expansion is pure (§9.4.10). The only environment reads are `env!`/`option_env!`; every read is recorded as a build dependency (name, value at compile time) in the dependency manifest consumed by the build system (Volume 20), so rebuilding after an environment change invalidates the affected units (G23).

### 9.5.10 Sandbox restrictions
Macros cannot perform I/O, read the clock, use randomness, spawn threads, or access the network — structurally, because expansion is token rewriting and the const interpreter has no such primitives (§9.1.8). There is no escape hatch; procedural execution is banned (§9.5.1).

### 9.5.11 Dependency handling
Macros from dependencies are loaded from dependency metadata before resolution, in post-order of the crate graph with tie-break by ascending crate name. A macro's identity includes the exporting crate's name and version; two crates exporting the same macro name are disambiguated by path.

### 9.5.12 Compiler-version compatibility
Declarative macros are pure source and carry no compiler-version stamp; their grammar is stable across compiler versions within edition P1. The built-in macro set is pinned to edition P1: adding or removing a built-in macro is an edition change. No compatibility shims exist; a macro that relied on changed built-in behavior must be updated in source.

## 9.6 Compile-time dependencies (design §14.6)

### 9.6.1 Host execution
All compile-time execution (const evaluation, macro expansion, derive expansion) runs inside the compiler host process, single-threaded per compilation unit, without spawning subprocesses or interpreters outside the compiler.

### 9.6.2 Target execution restrictions
No target-binary execution occurs at compile time. The target influences compile-time execution only through declared target parameters (pointer width, endianness, layout, feature set; Volume 16).

### 9.6.3 Host versus target types
The interpreter models target types exactly (Volume 05 widths; Volume 16 target parameters). Host integer size, host endianness, and host pointer width are never used (Law 2: no silent assumptions).

### 9.6.4 Cross-compilation
Const evaluation is parameterized by the target triple; the cache key includes it (§9.1.13). Compiling one crate for two targets re-evaluates every target-dependent constant; target-independent constants may be shared across targets only if their key omits target dependence, and the omission must be derivable from the const's declared type and body.

### 9.6.5 Build scripts
Build scripts are banned in edition P1: a package containing an executable build script is rejected at load with PLC-0432. The replacement is the declarative build manifest defined in Volume 20 §27 (declarative library search paths, link flags, and feature maps). The consequence is that no user code executes at build time outside the compiler sandbox.

### 9.6.6 Compile-time library dependencies
A `const fn` in a dependency is callable at compile time only if the dependency's metadata carries its body as **const bytecode**: a stack-machine program over the closed opcode set {PUSH_LIT, PUSH_LOCAL, POP, ADD, SUB, MUL, DIV, REM, AND, OR, XOR, SHL, SHR, NEG, NOT, CMP, CAST, LOAD_FIELD, INDEX, AGG_BUILD, AGG_GET, BR, BR_IF, CALL, RET, TRAP_PANIC} with uLEB128 operands. Bytecode is verified at load (opcode validity, stack-depth bounds per opcode, jump targets in range); invalid bytecode is PLC-0433 and is never executed.

### 9.6.7 Caching
Const results cache per §9.1.13. Macro expansions cache per (macro DefId, input tokens, registry version). Both caches are content-addressed and invalidation-safe under the Volume 12 §12.10 contract (G23).

### 9.6.8 Sandboxing
The sandbox is structural: the interpreter's opcode set (§9.6.6) contains no I/O, clock, randomness, network, or process primitives; expansion is token rewriting; env access exists only as the recorded `env!`/`option_env!` built-ins. Memory is bounded by §9.1.11; time by §9.1.10 and the Volume 12 wall-clock limit.

### 9.6.9 Deterministic evaluation
Same (inputs, target triple, env-record) ⇒ same results, bitwise (OB-09-3, G14). The interpreter and expander are deterministic by construction; PRNGs are banned in all CEP-0/CEP-1 compile-time components.

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-09-1 | Every construct accepted by `e const-legal` is a member of W1–W14; every W entry has a defined evaluation rule. | EXH over the enumerated construct set of Volume 02 | FV-0 | tests/vol09/legality | G3 |
| OB-09-2 | Every interpreter transition strictly decrements the remaining-step counter; no request exceeds 10^7 steps. | Z3 (bounded transition system) + REVIEW of the measure | FV-0 | tools/fv/ob09-02.z3 | G11 |
| OB-09-3 | Same (context, expression, target, env-record) yields bitwise-identical values. | EXH (corpus) paired with PROP (deterministic PRNG) | FV-0 | tests/vol09/determinism | G14 |
| OB-09-4 | Integer overflow, division/modulo by zero, and bad shifts in const context reject (PLC-0410); no silent wrap. | EXH over 10 integer types × 9 operators × boundary operands | FV-0 | tests/vol09/overflow | G3 |
| OB-09-5 | The effect row of every const-evaluated expression is ∅; reaching `panic` rejects (PLC-0407). | EXH corpus | FV-0 | tests/vol09/effects | G3 |
| OB-09-6 | Every promoted value contains no interior mutability and is ≤ 65536 bytes; cyclic values reject. | EXH over generated value shapes to depth 4 | FV-0 | tests/vol09/promotion | G2 |
| OB-09-7 | Static initialization order is topological with ascending-symbol-ID tie-break; any `#[init]` cycle rejects (PLC-0418). | EXH over graphs of ≤ 64 nodes | FV-0 | tests/vol09/static-init | G14 |
| OB-09-8 | Hygiene: token programs differing only in hygiene context resolve per rule H (b) and (c). | EXH corpus of 20 context-pairs + REVIEW of rule H totality | FV-1 | tests/vol09/hygiene | G3 |
| OB-09-9 | Expansion terminates: depth ≤ 128, tokens ≤ 2^20, and identical (macro, input) pairs within a chain halt with PLC-0429. | EXH over adversarial expansion corpus | FV-0 | tests/vol09/expansion-term | G11 |
| OB-09-10 | Identical (definition, input tokens, invocation span, registry version) produce byte-identical expansions. | EXH | FV-0 | tests/vol09/expansion-det | G14 |
| OB-09-11 | Each of the 13 built-in macros has exactly the effect of the §9.5.2 table; the set is closed. | REVIEW (invariant: table totality against registry) | FV-1 | review/vol09/builtin-macros | G26 |
| OB-09-12 | Any semantic change to (body, referenced consts, substitution, target, env-record) changes the cache key. | PROP paired with REVIEW of key construction | FV-1 | tests/vol09/cache-keys | G23 |
| OB-09-13 | Reflection capabilities are exactly R1–R8; every banned form is rejected with PLC-0420. | EXH corpus | FV-0 | tests/vol09/reflection | G26 |
| OB-09-14 | StaticSafe and Zeroable are total predicates over the type grammar to depth 4, with exact accept/reject per §9.2. | EXH over generated type shapes | FV-0 | tests/vol09/static-safe | G2 |

## Coverage ledger

| Bullet (design.md §14, abbreviated) | Defined in |
| ----------------------------------- | ---------- |
| Constant expressions | §9.1.1 |
| `const` declarations | §9.1.2 |
| `const fn` | §9.1.2 |
| Evaluation order | §9.1.3 |
| Permitted operations | §9.1.4 |
| Forbidden operations | §9.1.5 |
| Pointer restrictions | §9.1.6 |
| Allocation restrictions | §9.1.7 |
| Effects during evaluation | §9.1.8 |
| Recursion | §9.1.9 |
| Termination | §9.1.10 |
| Evaluation resource limits | §9.1.11 |
| Diagnostics | §9.1.12 |
| Constant caching | §9.1.13 |
| Static values | §9.2.1 |
| Constant initialization | §9.2.2 |
| Runtime initialization, if supported | §9.2.3 |
| Initialization order | §9.2.4 |
| Dependency cycles | §9.2.5 |
| Initialization failure | §9.2.6 |
| Destructor behavior | §9.2.7 |
| Thread-local initialization | §9.2.8 |
| Type reflection | §9.3.1 (R1) |
| Field reflection | §9.3.1 (R2) |
| Method reflection | §9.3.1 (banned, R-set closure) |
| Trait reflection | §9.3.1 (R3) |
| Layout reflection | §9.3.1 (R5) |
| Generic reflection | §9.3.1 (R6) |
| Compile-time queries | §9.3.1 (R8) |
| Visibility restrictions | §9.3.2 |
| Reflection stability | §9.3.3 |
| Runtime metadata policy | §9.3.4 |
| Compile-time code generation | §9.4.1 |
| Generated declarations | §9.4.2 |
| Type generation | §9.4.3 |
| Implementation generation | §9.4.4 |
| Syntax tree construction | §9.4.5 |
| Generated source locations | §9.4.6 |
| Diagnostics (generated code) | §9.4.7 |
| Hygiene (generated code) | §9.4.8 |
| Generated-code safety | §9.4.9 |
| Determinism (generated code) | §9.4.10 |
| Declarative macros | §9.5.1, §9.5.3–§9.5.12 |
| Procedural macros | §9.5.1 (banned) |
| Attribute macros | §9.5.1 (banned) |
| Derive macros | §9.5.1 (banned), §9.4.4 (built-in derive) |
| Built-in compiler macros | §9.5.2 |
| Invocation syntax | §9.5.3 |
| Expansion timing | §9.5.4 |
| Expansion inputs | §9.5.3 |
| Expansion outputs | §9.5.4 |
| Hygiene (macros) | §9.5.5 |
| Name resolution (macros) | §9.5.6 |
| Recursion (macros) | §9.5.7 |
| Error diagnostics (macros) | §9.5.8 |
| Build reproducibility | §9.5.9 |
| Sandbox restrictions | §9.5.10 |
| Dependency handling | §9.5.11 |
| Compiler-version compatibility | §9.5.12 |
| Host execution | §9.6.1 |
| Target execution restrictions | §9.6.2 |
| Host versus target types | §9.6.3 |
| Cross-compilation | §9.6.4 |
| Build scripts | §9.6.5 |
| Compile-time library dependencies | §9.6.6 |
| Caching | §9.6.7 |
| Sandboxing | §9.6.8 |
| Deterministic evaluation | §9.6.9 |

bullets in scope: 68, defined: 68
