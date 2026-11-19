<!--
CEP:FILE: spec/volume-03-names-modules.md
CEP:WHAT: Volume 03 of the Planck Master Specification Library: name resolution, the module and package model, and declaration semantics for Planck 0.1, edition P1. Defines design.md §3-§5 (lines 233-322).
CEP:WHY: design.md requires name resolution to be its own specification, not a side effect of parsing; this volume settles namespaces, scopes, shadowing, visibility, imports, paths, hygiene, the module/package model, and every declaration kind exactly, consistently with the pinned trait coherence rules (SPEC-INDEX §3).
CEP:STATUS: complete
CEP:FAILURE: A resolver, module loader, or declaration checker that diverges from a rule here is non-conforming. A missing coverage-ledger row (§3.6) is a missing definition. Any "appropriate"/"as needed"/TBD phrasing here is a defect.
CEP:ASSUMES: SPEC-INDEX.md §0-§5 (esp. §3 pinned decisions: coherence, effect rows); standards.md §3-§4; verification standards.md §2 (G1-G5); doc standards.md; design.md lines 233-322; Volume 01 (charter), Volume 02 (grammar productions referenced by section), Volumes 04-10, 19-20 for referenced semantics.
CEP:COST: document; resolution algorithms herein are O(size-of-program) compile-time work; limits named are constants.
CEP:EVIDENCE: Verification matrix (§3.5) and Coverage ledger (§3.6) in this file.
-->

# Volume 03 — Names, modules, and declarations

**Abstract.** This volume defines the name-resolution model (namespaces, scopes, lookup, imports, paths, hygiene), the module and package system (files, roots, ownership, exports, visibility, dependency order, conditional compilation, metadata, caches), and the semantics of every declaration kind (variables through impls). Every rule is exact; every ambiguity of lookup is settled by a total precedence order.

## 3.0 Scope and traceability

| Field | Value |
| ----- | ----- |
| design.md sections defined | §3 (lines 233-263), §4 (lines 265-287), §5 (lines 289-317) |
| Bullets in scope | 67 (§3: 23, §4: 19, §5: 25) |
| Prose requirements in scope | 3 (design.md line 237 "its own specification"; line 267 "complete compilation unit model"; line 319 deliverables) |
| Deliverables satisfied | Name Resolution Specification (§3.1), Module System Specification (§3.2), Declaration Semantics Specification (§3.3) |

design.md line 237 requires resolution to be a standalone specification: §3.1 is that specification. design.md line 267 requires the complete compilation unit model: §3.2 is that model.

## 3.1 Name resolution

### 3.1.1 Identifier namespaces (design.md L241)

There are exactly three namespaces: the **Type namespace** (types, traits, modules, enum variants as types), the **Value namespace** (functions, constants, statics, local bindings, tuple-struct constructors, enum unit-variant constructors), and the **Macro namespace** (macro definitions). Each name binding occurs in exactly one namespace; a module may contain the same name in different namespaces. Lookup is always namespace-targeted by grammatical context: type position → Type; value position → Value; `name!` invocation → Macro.

### 3.1.2 Lexical scopes (design.md L242)

Scopes are: the crate root module scope; one scope per module; one scope per block; one per function body; one per closure body. A `let` binding's scope begins at its binding point (the end of the `let` statement) and ends at the end of the enclosing block. A binding is not in scope inside its own initializer. Function parameters' scope is the function body. A closure parameter's scope is the closure body.

### 3.1.3 Module scopes (design.md L243)

The module tree is a tree rooted at the crate root. Name lookup inside a module walks the chain of enclosing modules from innermost to crate root (privacy permitting, §3.1.10). A child module does not automatically re-export or shadow anything; it simply sees ancestor items after its own.

### 3.1.4 Type namespaces (design.md L244)

The Type namespace of a module holds: `struct`, `enum`, `union`, `trait`, `type alias`, `mod` (module names live here, §3.1.1), and imports/re-exports bound in the Type namespace. Enum variants with data are constructors in the Value namespace whose names also resolve as types in type position through their enum.

### 3.1.5 Value namespaces (design.md L245)

The Value namespace of a scope holds: `fn` items, `const` items, `static` items, `let` bindings, function parameters, closure parameters, tuple-struct constructors, unit-variant and tuple-variant constructors, and imports bound in the Value namespace.

### 3.1.6 Trait namespaces (design.md L246)

Trait names are bindings in the Type namespace (there is no separate trait namespace); their kind is distinguishable at resolution. Traits are referenced by path in bounds, impls, and `dyn` types. Method resolution is not path lookup; it is the exact algorithm: (M1) build the candidate receiver chain `[T, &T, &mut T]` extended by deref steps (each deref step replaces the candidate with `*candidate`; Volume 04 defines the deref chain for references and the Volume 19 box type); (M2) for each candidate in order, search inherent impl methods of that candidate type by name; (M3) then search trait methods from traits whose trait reference is in scope and whose bound is satisfied by the receiver; (M4) the first match wins; two trait candidates matching at the same step is `E-NAM-02`. This order is normative; Volume 04 §6.5 elaborates trait solving for step M3.

### 3.1.7 Macro namespaces, applicability (design.md L247)

The Macro namespace is applicable: edition P1 has declarative macros (Volume 02 §2.5.20), which occupy the Macro namespace. A macro invocation resolves to the nearest macro definition of that name in scope order by source position: definitions must textually precede their uses within the module chain (a use before any definition is `E-NAM-07`). Macro definitions are order-dependent by this rule; items are not (§3.1.23). A macro may not invoke itself directly or indirectly; expansion depth is bounded by `MAX_MACRO_DEPTH = 64` (Volume 02 §2.5.20).

### 3.1.8 Generic parameter scopes (design.md L248)

The generic parameters of an item are in scope in: the item's signature, its `where` clause, and (for functions) its body; for impls, the impl's generic parameters are in scope throughout the impl block. A generic parameter shadows any outer binding of the same name in the same namespace within its scope. Two generic parameters of one item with the same name are `E-DEC-01`.

### 3.1.9 Shadowing (design.md L249)

A new binding shadows all prior bindings of the same name in the same namespace from its new binding point onward. Two `let` bindings of the same name in one block are legal; the later shadows the earlier from its binding point (the earlier remains live for borrow analysis until its scope end, Volume 06). Generic parameters shadow outer names (§3.1.8). Imports shadowed by local items are `E-NAM-04` (§3.1.14 tier rules); local bindings shadowing items are legal (tiers 1 vs 2 below).

### 3.1.10 Name visibility (design.md L250)

The visibility forms are exactly: omitted (private), `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`. Exact rules: private = visible in the defining module and all its descendants; `pub` = visible wherever the enclosing module is reachable; `pub(crate)` = visible in the whole crate; `pub(super)` = visible in the parent module and its descendants; `pub(in path)` = visible in the named ancestor module and its descendants; the `path` must be an ancestor of the defining module, else `E-NAM-06`. Visibility applies to items, struct fields, and use re-exports; enum variants and union fields take the visibility of their enum/union and admit no modifiers. An item used outside its visibility is `E-NAM-06` at the use site.

### 3.1.11 Name imports (design.md L251)

`use` (grammar Volume 02 §2.5.3) binds each leaf path name into the current module in the leaf's own namespace, under the binding name (`as` alias or the leaf name; `_` binds invisibly for trait purposes only). Import resolution is resolved at the importing module with the full lookup of §3.1.16. Two imports binding the same name into one module in one namespace are `E-NAM-05`, even when they target the same item. Feature template for the `use` feature:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | `UseItem` production (Volume 02 §2.5.3). |
| 2. Static semantics | Each leaf must resolve per §3.1.16 tiers; duplicates are `E-NAM-05`. |
| 3. Type semantics | Not applicable: imports introduce no types of their own. |
| 4. Value semantics | Not applicable. |
| 5. Evaluation semantics | Resolved entirely at compile time; no runtime effect. |
| 6. Ownership | Not applicable. |
| 7. Effects | Not applicable. |
| 8. Control flow | Not applicable. |
| 9. Compile-time behavior | Imports are resolved before type checking; glob expansion is order-independent (§3.1.14). |
| 10. Memory behavior | Not applicable. |
| 11. ABI behavior | Not applicable. |
| 12. IR 1 mapping | Not applicable: imports leave no IR node. |
| 13. IR 2 mapping | Not applicable. |
| 14. Machine IR mapping | Not applicable. |
| 15. Diagnostics | `E-NAM-05` duplicates, `E-NAM-01` unresolved leaf, `E-NAM-02` ambiguous glob use. |
| 16. Validation | EXH over bounded import configurations (OB-03-1, OB-03-13). |

### 3.1.12 Wildcard imports (design.md L252)

`use path::*;` binds every publicly visible item of the target module in its own namespace. Wildcard bindings occupy resolution tier 4 (below explicit imports, §3.1.16). A wildcard on a module with a wildcard-imported name re-exports transitively; cycles among wildcard targets are cut at first visit in path order (deterministic: modules are visited in ascending path order).

### 3.1.13 Re-exports (design.md L253)

`pub use path::Name;` (any `use` with a visibility) re-exports the target: the name is bound at the new path with visibility per §3.1.10, and the target must be visible to the re-exporting module (`E-NAM-06` otherwise). The re-export surface contributes to the export closure of §3.2.7.

### 3.1.14 Glob ambiguity (design.md L254)

Resolution tiers, in strict priority order: (1) local bindings (innermost block scope outward); (2) items declared in the current module; (3) explicit imports of the current module; (4) wildcard imports; (5) the extern set (dependency crate names, §3.2.3). A name is **glob-ambiguous** iff tier 4 supplies candidates from two or more distinct glob sources and no higher tier supplies one; the ambiguity is an error `E-NAM-03` at the use site, not at the import site. Within one tier, two candidates that resolve through re-exports to the same item are one candidate; otherwise the tier is ambiguous (`E-NAM-02`) unless a lower tier already decided.

### 3.1.15 Qualified paths (design.md L255)

Qualified path forms: `Type::item` (associated item of a type), `<Type as Trait>::item` (disambiguated trait item), and plain paths (§3.1.16). `<Type as Trait>` requires the impl to exist and be visible; the associated item resolves in the trait's namespace. `Type::item` searches inherent impls first, then visible trait impls (order M2/M3 of §3.1.6); a bare `Type::method` that matches only trait methods resolves if exactly one trait candidate exists, else `E-NAM-02`.

### 3.1.16 Relative paths (design.md L256)

A relative path `a::b::c` resolves its first segment by the tier walk of §3.1.14, namespace-targeted. Subsequent segments must resolve as: module names (Type namespace) for intermediate segments, or the final item in the targeted namespace. An intermediate segment that is not a module or re-export is `E-NAM-01` naming the segment. The result of resolution is a unique definition ID; ambiguity is `E-NAM-02`.

### 3.1.17 Absolute paths (design.md L257)

Two absolute prefixes exist and are synonyms: `crate::` and leading `::`; both denote the current crate root. A path beginning with neither an absolute prefix nor resolving at §3.1.16 tiers is an error `E-NAM-01`. There is no extern-crate absolute prefix in P1; dependency names resolve at tier 5 (§3.2.3).

### 3.1.18 `self`, `Self`, `super`, and `crate` (design.md L258)

| Keyword | Exact meaning |
| ------- | ------------- |
| `self::` | path prefix = current module |
| `super::` | path prefix = parent module; at crate root it is `E-NAM-01` (no parent) |
| `crate::` | path prefix = crate root (synonym of leading `::`) |
| `Self` | inside an impl or trait (including default methods): the implementing/self type; outside: `E-NAM-01` |
| `self` | receiver parameter keyword only (Volume 02 §2.5.6); not a path segment and not a general binding |

### 3.1.19 Name hygiene (design.md L259)

Identifiers produced inside a macro definition body carry the hygiene context of that macro definition. Resolution matches binding pairs only when hygiene contexts are equal, for local variable bindings and labels (fully hygienic). Item-level names, imports, and types are context-insensitive: they resolve at the expansion site's scopes. A macro cannot capture a caller's local variable, and a caller cannot see a macro-internal local. This rule is total over the declarative macro system; the hygiene property is obligation OB-03-11.

### 3.1.20 Unresolved names (design.md L260)

A name that resolves through no tier is `E-NAM-01`, reporting the name, the targeted namespace, and the searched scope chain (module chain list). There are no suggestions in edition P1 diagnostics (deterministic, exact); suggestion tooling is Volume 21 §28.9 and is not part of the language contract.

### 3.1.21 Ambiguous names (design.md L261)

Ambiguity arises only within one resolution tier (§3.1.14): two distinct candidate items in the same namespace. It is `E-NAM-02`, listing both candidate paths. Ambiguity is never resolved silently by order; the only order-based rule in the volume is macro textual order (§3.1.7) and the tier list itself.

### 3.1.22 Recursive declarations (design.md L262)

Items are mutually recursive freely within a module and across modules: resolution is order-independent for items (§3.1.23). Recursion restrictions are semantic and are stated with the declarations: functions may recurse (Volume 03 §3.3.4); types must indirect every cycle (§3.3.24); impls must satisfy coherence (§3.3.20). There is no resolution-level recursion limit beyond `MAX_MACRO_DEPTH` for macro expansion and the Volume 12 limits for monomorphization.

### 3.1.23 Forward declarations (design.md L263)

There is no forward-declaration syntax and none is needed: item resolution within a crate is order-independent; a `fn` may call a `fn` declared later in the same module or a descendant module, and a type may reference a type declared later, subject to §3.3.24. Local bindings require definition before use (§3.1.2). Macros are the exception: definition must precede use (§3.1.7).

**Integration points.** Depends on: Volume 02 §2.5.3 (use grammar), Volume 04 (trait solving, method dispatch), Volume 20 (dependency names). Depended on by: Volume 04 (type checking), Volume 09 (macro expansion), Volume 11 (pipeline ordering).

## 3.2 Module and package system

### 3.2.1 Source modules (design.md L269)

A **source module** is inline (`mod name { ... }`) or file-backed (`mod name;`). File resolution is exact: a file-backed `mod name;` declared in module M, whose file is F, resolves to the file `name.plk` or the file `name/mod.plk` in the directory of F; the first existing candidate wins, and if both exist it is `E-DEC-09`. The source file extension is `.plk`. A missing file is `E-DEC-08`. Feature template for the module feature:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | `ModItem` production (Volume 02 §2.5.2). |
| 2. Static semantics | File must exist (`E-DEC-08`), be loadable at most once (`E-DEC-09`), and parse cleanly. |
| 3. Type semantics | Not applicable: modules have no type. |
| 4. Value semantics | Not applicable. |
| 5. Evaluation semantics | Compile-time only: the module file is parsed when its `mod` item is reached in the crate tree. |
| 6. Ownership | Each file is owned by exactly one declaring `mod` (§3.2.6). |
| 7. Effects | Not applicable. |
| 8. Control flow | Not applicable. |
| 9. Compile-time behavior | The module tree is built before resolution; file-backed mods may be `#[cfg]`-disabled (§3.2.15). |
| 10. Memory behavior | Not applicable. |
| 11. ABI behavior | Not applicable. |
| 12. IR 1 mapping | Not applicable: modules group items; items carry IR. |
| 13. IR 2 mapping | Not applicable. |
| 14. Machine IR mapping | Not applicable. |
| 15. Diagnostics | `E-DEC-08` missing file, `E-DEC-09` multiply loaded, `E-PAR-01` parse errors carry the file ID. |
| 16. Validation | EXH over declared tree shapes (OB-03-9). |

### 3.2.2 Nested modules (design.md L270)

Modules nest without bound except `MAX_MODULE_DEPTH = 64` (deeper nesting is `E-DEC-01` with a depth report). Paths through nested modules follow §3.1.16. Visibility rules compose per §3.1.10 (a descendant module sees ancestor private items).

### 3.2.3 External modules (design.md L271)

An external module is a dependency crate. The **extern set** is the set of dependency names from the package manifest (Volume 20 §27.3). A first path segment resolving at tier 5 (§3.1.14) denotes the dependency's crate root module; subsequent segments resolve inside it against its export surface (§3.2.18). A name not in the extern set is `E-NAM-01`.

### 3.2.4 Packages or crates (design.md L272)

A **crate** is one compilation unit: one root module plus its module tree, compiled to one artifact. A **package** is a versioned distribution unit: it contains exactly one library crate (root `src/lib.plk`) or one or more binary crates (root `src/main.plk`, or `src/bin/<name>.plk` per binary). Manifest and registry mechanics are Volume 20; the crate/package model above is the compilation-unit contract of this volume.

### 3.2.5 Package roots (design.md L273)

Exact roots: library crate root = `src/lib.plk`; default binary root = `src/main.plk`; named binary roots = `src/bin/<name>.plk`. The root file's module path is the crate root; `crate::` denotes it (§3.1.17).

### 3.2.6 Module ownership (design.md L274)

Every module has exactly one parent (crate root's parent is the crate itself); the ownership relation is the declaration relation. Each `.plk` file must be the target of at most one `mod` declaration in the crate; two `mod` declarations resolving to the same file are `E-DEC-09`. The crate root is owned by the package.

### 3.2.7 Public exports (design.md L275)

The **export surface** of a library crate is the closure: start from the crate root's `pub` items; for each `pub use` re-export, add the target if visible per §3.1.10; repeat until closed. The surface is exactly the set of names reachable from `crate::` through public paths. It is computed once and frozen into the interface metadata (§3.2.18).

### 3.2.8 Private declarations (design.md L276)

A private item is visible exactly in its defining module and that module's descendants (§3.1.10). Private items appear in no export surface and cannot be named by dependency crates; naming one is `E-NAM-06` at the use site.

### 3.2.9 Dependency graphs (design.md L277)

Two graphs exist. The **package graph**: nodes are packages; edges are manifest dependencies (Volume 20 §27.4); it must be acyclic (§3.2.10). The **module graph** inside a crate is the module tree plus cross-module item references; it may be cyclic freely (one compilation unit). Item-level cross-crate references must target the export surface of the dependency (§3.2.7).

### 3.2.10 Circular dependency handling (design.md L278)

Circular references between packages are rejected: the package graph must be a DAG; a cycle is `E-DEC-11` reporting one cycle as a package-name list. Circular references between modules within one crate are legal and resolved order-independently (§3.1.22). This split is exact: crates are the cycle boundary.

### 3.2.11 Compilation order (design.md L279)

Packages compile in reverse topological order of the package graph, computed by Kahn's algorithm with stable tie-break by ascending package name (pinned, SPEC-INDEX §5.7: determinism G14; cycle failure is `E-DEC-11`, never a partial order). Within a crate there is no ordering: one compilation unit.

### 3.2.12 Separate compilation (design.md L280)

Each crate compiles independently against its dependencies' interface metadata (§3.2.18). A dependency contributes: its export surface, its type definitions with layouts, its function signatures with inferred effect rows, its trait definitions and impls, its const values, and its re-export map. No dependency source is read at compile time; the metadata file is the only input. Two crates in the graph may compile concurrently; their outputs are independent (G14 determinism holds per crate).

### 3.2.13 Module initialization (design.md L281)

There is none. Planck has no dynamic module initializers: `static` initializers must be const-evaluable (§3.3.3), so no initialization order exists to define. Runtime initialization happens only for thread-locals (first access, Volume 05 §8.8) and for explicitly lazy wrappers (Volume 19). The startup sequence is: process entry, then `main` (Volume 01 §1.4.12); no module code runs in between.

### 3.2.14 Package versioning (design.md L282)

Version strings follow **SemVer 2.0.0** exactly:

```
Version       = Major "." Minor "." Patch [ "-" Prerelease ] [ "+" Build ] ;
Major         = "0" | NonZeroDigit Digit* ;
Minor, Patch  = Digit+ ;                         -- numeric, no leading zeros except "0"
Prerelease    = Identifier ( "." Identifier )* ;  Identifier = AlnumOrDash+ with numeric-identifier no-leading-zero
Build         = Identifier ( "." Identifier )* ;
```

Comparison: Major, then Minor, then Patch numerically; a version with Prerelease has lower precedence than the same version without; prerelease identifiers compare field-wise (numeric identifiers compare numerically and are lower than alphanumeric; alphanumeric compare in ASCII order; a longer field list wins over a prefix); Build metadata is ignored in precedence. Requirement operators are exactly `=`, `^`, `~`, `*`:

| Requirement | Accepted range |
| ----------- | -------------- |
| `=M.m.p` | exactly that version |
| `^M.m.p`, M ≥ 1 | [M.m.p, (M+1).0.0) |
| `^0.m.p`, m ≥ 1 | [0.m.p, 0.(m+1).0) |
| `^0.0.p` | [0.0.p, 0.0.(p+1)) |
| `~M.m.p` | [M.m.p, M.(m+1).0) |
| `*` | all versions |

A version with a prerelease satisfies a requirement only if the requirement carries a prerelease with the same M.m.p base. Comparators `>`, `<`, `>=`, `<=` are not grammar in P1 (banned). The resolver selects the highest-precedence version satisfying all constraints; ties after ignoring Build metadata are broken by the lexicographically greater Build string. Invalid version strings are `E-PKG-01`.

### 3.2.15 Conditional modules (design.md L283)

`#[cfg(...)]` on a `mod` (or any item) conditionally includes it. The cfg grammar and evaluation are exact:

```
Cfg       = CfgTest | "(" "all" CfgList ")" | "(" "any" CfgList ")" | "(" "not" Cfg ")" ;
CfgList   = Cfg { "," Cfg } ;
CfgTest   = Key | Key "=" STRING ;
Key       = "target_os" | "target_arch" | "target_env" | "target_endian"
          | "target_pointer_width" | "feature" ;
```

The configuration environment is fixed: `target_os ∈ {linux, macos, windows, freestanding}`, `target_arch ∈ {x86_64, aarch64}`, `target_env ∈ {gnu, musl, msvc, none}`, `target_endian ∈ {little, big}`, `target_pointer_width ∈ {"64", "32"}` (values per target, Volume 01 §1.4), and `feature` names from the package manifest (Volume 20 §27.5). An unknown key or a value outside the exact set is `E-DEC-10` (strict; no unknown flags). Evaluation is plain boolean algebra over the environment; `all`/`any` with an empty list is a compile-time rejection (`E-DEC-10`).

### 3.2.16 Platform-specific modules (design.md L284)

Platform specialization uses §3.2.15 cfg on `mod` declarations. Exact rule: a cfg-false file-backed `mod` is not read: its file is not opened, not parsed, and contributes no diagnostics. Inline cfg-false module bodies are skipped identically. At least one cfg arm must be true where the crate requires the name (else `E-NAM-01` at first use).

### 3.2.17 Module-level generic declarations (design.md L285)

Modules are not generic in edition P1: the `ModItem` production (Volume 02 §2.5.2) admits no `GenericParams`, and a generic parameter list on `mod` is a parse error (`E-PAR-01`). Genericity lives on items (§3.3.5, §3.3.6, traits, impls). This is the settled definition of the bullet: rejected by grammar.

### 3.2.18 Interface metadata (design.md L286)

The interface metadata of a library crate is one file `lib<name>.plkm` with exactly these sections: (a) the export surface: paths and definition IDs (§3.2.7); (b) type definitions reachable from the surface, with `#[repr]` layouts (Volume 05 math); (c) function signatures: parameter types, return types, inferred effect rows (SPEC-INDEX §5.3); (d) trait definitions and the crate's trait impls with their coherence-relevant headers; (e) const items with their values; (f) the re-export map (public alias → canonical path). The physical encoding is Volume 17 §24.6. A consumer crate must not require anything outside (a)-(f); a metadata file missing a section is a corrupted-input error `E-PKG-03`.

### 3.2.19 Compilation cache boundaries (design.md L287)

The cross-crate cache unit is the crate: key = `(crate name, version, target triple, edition, compiler version, sorted feature set, cfg evaluation of §3.2.15)`. A key change invalidates the crate's metadata and object; modules are not cross-crate cache boundaries. Within a crate, the incremental unit is the item (Volume 02 §2.6.11; Volume 21 §28.10 defines the within-crate cache). Cache keys are content-hashed (FNV-1a 64 for ranges, §2.6.11) and never wall-clock-based (G14).

**Integration points.** Depends on: Volume 02 (grammar, item spans), Volume 20 (manifests, registry), Volume 17 (metadata encoding), Volume 05 (layouts). Depended on by: Volume 04 (type checking against metadata), Volume 12 (limits), Volume 21 (incremental engine).

## 3.3 Declaration semantics

### 3.3.1 Variables (design.md L293)

`let` (Volume 02 §2.5.13) binds a pattern. Exact rules: bindings are immutable unless `mut`; a write to a non-`mut` binding is rejected (Volume 06 state machine action `Write`); the type annotation is optional and when present must accept the initializer (Volume 04); with no annotation and no initializer, the type must be inferable from later writes, else `E-TYPE` (Volume 04 catalog); an uninitialized binding must be definitely assigned before any read (Volume 01 §1.3.3, state `Uninitialized` → `Owned`).

### 3.3.2 Constants (design.md L294)

`const NAME: Type = expr;` requires `expr` to be const-evaluable (Volume 09) and the value to be exactly of `Type`. Each use of the constant is a fresh copy of the value; constants have no address and are not places. Naming: the name must match `[A-Z][A-Z0-9_]*`, else warning `W-NAM-01`.

### 3.3.3 Static variables (design.md L295)

`static NAME: Type = const-expr;` creates one place with program-lifetime provenance. The initializer must be const-evaluable (this eliminates initialization order, §3.2.13). The type must implement the sync marker trait of Volume 08 (`T: Sync`), else rejection. `static mut` is not grammar in edition P1 (Volume 02 §2.5.3); interior mutability uses the Volume 19 unsafe-cell wrapper types. Reading a static is an ordinary read; writes require `mut`-free paths through the wrapper types (Volume 06).

### 3.3.4 Functions (design.md L296)

`fn` per Volume 02 §2.5.6. Calling requires the caller's effect row to include the callee's row (pinned, SPEC-INDEX §5.3). Recursion is legal, including mutual recursion across items (§3.1.22). Parameters are passed by value (move or copy per Volume 05). A body-less `fn` outside trait/extern is `E-DEC-03`.

### 3.3.5 Generic functions (design.md L297)

A `fn` with `GenericParams` is generic; instances are created by monomorphization only (pinned, SPEC-INDEX §3): substitution happens per call site; recursion during monomorphization is bounded by depth `32` (exceeding it is `E-MONO-DEPTH`, Volume 12 limits table); the semantics of an instance equal the generic's semantics (obligation family OB-04, Volume 04). Generic parameters shadow outer names within scope (§3.1.8).

### 3.3.6 Structs (design.md L298)

`struct` per Volume 02 §2.5.7. Layout: default `#[repr(P)]` (pinned): fields in declaration order, alignment = max field alignment, size = size padded to that alignment, no reordering, no niche packing unless `#[repr(niche)]`; exact layout math is Volume 05 §8.2. Primitive names are reserved: a struct (or any type-like item) may not be named one of the 17 `PrimType` names of Volume 02 §2.5.5 (`E-DEC-13`); local bindings may.

### 3.3.7 Tuple structs (design.md L299)

`struct S(Vis? T1, ...);` fields are indexed `.0`, `.1`, ... from 0. A constructor function `S` is generated in the Value namespace with the struct's visibility and signature `(T1, ...) -> S`. Field count is unbounded; empty tuple structs are `struct S();` and are distinct from unit structs (§3.3.8) at the type level (both have size 0, alignment 1).

### 3.3.8 Unit structs (design.md L300)

`struct S;` has exactly one value `S` (Value namespace, struct's visibility), size 0, alignment 1, and no fields.

### 3.3.9 Enums (design.md L301)

`enum` per Volume 02 §2.5.8. Discriminants: explicit `= expr` must be a const integer expression; implicit discriminant = previous + 1, starting at 0. The default repr is the smallest of `u8, u16, u32, u64` whose range covers all discriminant values, or the signed counterpart if any explicit discriminant is negative; `#[repr(iN)]`/`#[repr(uN)]` overrides (Volume 10). Unit variants generate constructor functions in the Value namespace; data variants generate tuple-struct-like constructors. Variant visibility = enum visibility (§3.1.10).

### 3.3.10 Unions (design.md L302)

`union` per Volume 02 §2.5.9. Every field must be a `Copy` type (Volume 04), else rejection. Layout: size = max field size, alignment = max field alignment (pinned `#[repr(P)]`). Writing a field initializes the union; reading a field other than the last-written one reads uninitialized memory: this is UB-1 and is reachable only inside `unsafe` (union field reads are unsafe operations, Volume 01 §1.3.10).

### 3.3.11 Traits (design.md L303)

`trait` per Volume 02 §2.5.10. Trait items: required `fn` (no body), default `fn` (body), `const` (body optional: no body = must be provided by impls), `type` (no body = must be provided). Supertraits (`: Bounds`) must be satisfied by every impl. **Object safety** (exact): a trait is `dyn`-compatible iff (O1) no method has generic type parameters (lifetime parameters allowed); (O2) every method receiver is `&self` or `&mut self`; (O3) no associated consts; (O4) no method has a parameter or return type mentioning `Self` by value. A non-compatible trait used as `dyn T` is a type error (Volume 04). Dispatch is via vtable (slot order = declaration order of the trait's methods, Volume 05 §8.7).

### 3.3.12 Implementations (design.md L304)

`impl` per Volume 02 §2.5.11, in two forms: inherent (`impl Type`) and trait (`impl Trait for Type`). Orphan rule (pinned, exact): an impl is legal iff the trait or the self type is local to the crate, or at least one type parameter is a local type that appears before any foreign type parameter. No fundamental-type escape hatch exists in edition P1. Violation is `E-DEC-06`. Overlap rejection is semantic (pinned): two impls that could both apply to some instantiation are rejected (`E-DEC-07`); the exact check is §3.3.20.

### 3.3.13 Type aliases (design.md L305)

`type Name = Type;` is transparent: identity is the aliased type (Volume 04 nominal identity); aliases are generic-capable (§3.3.5 rules); an alias cycle is `E-DEC-04` (§3.3.24 rule applies to the alias graph).

### 3.3.14 Associated constants (design.md L306)

`const X: T = v;` inside a trait may omit `v` (impls must provide); inside an impl it must have `v`. The impl-provided value must be const-evaluable and of exactly `T`. Access is `Type::X` or `<Type as Trait>::X` (§3.1.15).

### 3.3.15 Associated types (design.md L307)

`type A;` in a trait declares a required projection; `type A = T;` in an impl must appear exactly once per associated type per impl (missing is `E-DEC-12`; a second definition is `E-DEC-01`). Projections `T::A` resolve via the impl (Volume 04 §6.5). An impl's associated types are fixed at impl time; no specialization exists in P1 (pinned, SPEC-INDEX §3).

### 3.3.16 Foreign declarations (design.md L308)

Items in `extern "C" { ... }` (the only supported ABI string; others are `E-PAR-11`): body-less `fn` declarations and `static` declarations. Foreign `fn` signatures are checked only for type well-formedness; their preconditions are the caller's responsibility (UB-4 on ABI mismatch, Volume 01 §1.3.11). Calls to foreign functions carry the `ffi` effect atom (pinned). Unwinding across the boundary is a compile-time rejection (pinned): a Planck `fn` exposed to foreign code must have a `panic`-free wrapper per Volume 10 §16.1.

### 3.3.17 External statics (design.md L309)

`static NAME: Type;` inside an `extern` block declares a link-time place. Its address is resolved by the linker (Volume 17). Access is an unsafe operation (Volume 01 §1.3.10); UB-1/UB-2 judgments apply at the access with the static's declared type. The type must be sized (Volume 04).

### 3.3.18 Methods (design.md L310)

Methods are `fn` items inside impls. Receivers, exactly: `self` (≡ `self: Self`; consumes), `&self` (≡ `self: &Self`), `&mut self` (≡ `self: &mut Self`), and the explicit forms `self: Type` restricted to `Self`, `&Self`, `&mut Self`. A method with no receiver is an associated function, called as `Type::name(...)`. Receiver adjustment at calls follows §3.1.6 M1-M4.

### 3.3.19 Inherent implementations (design.md L311)

`impl Type { ... }` provides items available directly on `Type`. Multiple inherent impl blocks for one type are legal; two items with the same name across them (same namespace) are `E-DEC-02`. Inherent items shadow trait items at call sites (§3.1.6 M2 before M3); the shadowing produces warning `W-MET-01` when both exist with the same name.

### 3.3.20 Trait implementations (design.md L312)

`impl Trait for Type { ... }`. Exact rules: (C1) coherence: orphan rule per §3.3.12 (`E-DEC-06`); (C2) overlap: two trait impls for the same trait overlap iff there is a substitution making both impl headers apply; the check unifies the two headers (trait reference and self type, treating generic parameters as unification variables) ignoring `where` clauses, because P1 has no negative bounds; detected overlap is `E-DEC-07`; (C3) completeness: every required trait item must be provided exactly once (`E-DEC-12` on missing); (C4) constrainedness: every impl generic parameter must appear in the self type or the trait reference, else `E-DEC-05`; (C5) extra items beyond the trait's surface are allowed but unusable through the trait, with warning `W-DEC-01`.

### 3.3.21 Visibility (design.md L313)

Visibility per §3.1.10 applies to all item kinds. Per-kind notes: struct fields have independent visibility; tuple-struct fields likewise; enum variants and union fields inherit their parent's; trait items are as public as the trait; impl items are public if marked `pub` and the impl's types/trait are visible; `extern` block items inherit the block's visibility.

### 3.3.22 Declaration conflicts (design.md L314)

| Conflict | Outcome |
| -------- | ------- |
| Two items, same module, same namespace, same name | `E-DEC-01` |
| Same name in different namespaces | legal (§3.1.1) |
| Two inherent impl items, same name | `E-DEC-02` |
| Inherent method and trait method, same name | legal; inherent wins; `W-MET-01` |
| Impl provides trait item twice | `E-DEC-01` |
| Impl missing required trait item | `E-DEC-12` |
| Two generic parameters, same item, same name | `E-DEC-01` |
| Two `use` bindings, same name, same namespace | `E-NAM-05` |
| `mod` file loaded twice | `E-DEC-09` |

### 3.3.23 Duplicate definitions (design.md L315)

A duplicate definition is any second binding that conflicts per §3.3.22; the outcome table there is complete. Duplicates never merge and never shadow at item level (unlike locals, §3.1.9).

### 3.3.24 Recursive types (design.md L316)

A type definition graph is built: nodes are type items (structs, enums, unions, aliases), edges are uses of one type inside another's fields or variants. A cycle is legal iff every cycle passes through at least one indirection edge: a reference (`&`/`&mut`), raw pointer, or the Volume 19 box type. A direct cycle (no indirection edge) is `E-DEC-04` reporting the cycle. Enum self-reference through a data variant field counts as an edge; enum size math is Volume 05 §8.4.

### 3.3.25 Incomplete declarations (design.md L317)

A declaration is **incomplete** iff it lacks a required body or value: body-less `fn` outside trait/extern (`E-DEC-03`); `const` without a value outside a trait; `impl` missing a required trait item (`E-DEC-12`); `macro` with zero rules (`E-DEC-03`); `extern` block with unsupported ABI string (`E-PAR-11`). Trait-required methods and extern items are complete declarations by definition. No other incompleteness exists; there are no prototypes.

**Integration points.** Depends on: Volumes 02 (grammar), 04 (types, trait solving), 05 (layout, move), 06 (borrow states), 08 (Sync), 09 (const eval), 10 (repr attributes), 17 (linking), 19 (box, wrappers). Depended on by: Volume 11 (pipeline), Volume 12 (limits: monomorphization depth 32), Volume 20 (manifests), Volume 22 (conformance).

## 3.4 Known limitations

Edition P1 has no const generics, no negative impls, no fundamental types in coherence, no comparator requirement operators (`>`, `<`), no dyn-safe associated consts, no macro exporting across crates beyond the export surface (macro definitions re-exported via `pub use` bind in the Macro namespace of the importer), and a conservative overlap check that may reject impls a where-clause-aware checker would accept; a rejected-but-non-overlapping pair must be restructured or awaits an edition change. The prelude is empty by design.

## 3.5 Verification matrix

| Obligation ID | Property (exact statement) | Method | Class | Artifact | G-mapping |
| ------------- | -------------------------- | ------ | ----- | -------- | --------- |
| OB-03-1 | For bounded scope configurations (≤ 4 scopes, ≤ 3 bindings per scope, 3 namespaces, ≤ 3 candidates per name; domain ≤ 10^6), lookup returns a unique result exactly matching the §3.1.14 tier order, or the exact ambiguity/unresolved diagnostic. | EXH | FV-0 | tools/fv resolution checker | G14 |
| OB-03-2 | The 5-tier precedence of §3.1.14 is a strict total order on candidate sources. | REVIEW | FV-1 | This file §3.1.14 (invariant: tiers are pairwise disjoint and ranked) | G14 |
| OB-03-3 | Visibility reachability: for module trees of depth ≤ 3, branching ≤ 2, visibility ∈ 5 forms (78125 configurations), an item is usable at module M iff §3.1.10 says so. | EXH | FV-0 | tools/fv visibility checker | G1 |
| OB-03-4 | The impl-legality predicate accepts exactly the headers matching the pinned orphan rule over all 32 header shapes (trait local/foreign × ≤ 4 parameters local/foreign). | EXH | FV-0 | tools/fv orphan checker | G3 |
| OB-03-5 | Overlap detection: for bounded impl header pairs (≤ 3 type parameters), overlap is reported exactly when the headers unify (ignoring where clauses). | Z3 | FV-1 | tools/fv/coherence.smt2 | G3 |
| OB-03-6 | Package ordering: Kahn's algorithm with ascending-name tie-break yields the unique pinned order for all loopless digraphs on 4 labeled nodes (4096 cases) and reports `E-DEC-11` exactly on cyclic ones. | EXH | FV-0 | tools/fv pkg-order checker | G14 |
| OB-03-7 | The version grammar accepts exactly SemVer 2.0.0 strings and rejects others, for all strings of length ≤ 5 over the version alphabet (161051 cases). | EXH | FV-0 | tools/fv semver checker | G1 |
| OB-03-8 | cfg evaluation matches boolean algebra over the fixed environment for all combinations of ≤ 4 atoms drawn from the exact §3.2.15 key/value sets. | EXH | FV-0 | tools/fv cfg checker | G1 |
| OB-03-9 | File-backed `mod` resolution is unique for all declared tree shapes of depth ≤ 3, branching ≤ 2; a file targeted twice yields `E-DEC-09`. | EXH | FV-0 | tools/fv modfile checker | G1 |
| OB-03-10 | A type graph (≤ 8 nodes) is accepted iff every cycle contains an indirection edge; otherwise `E-DEC-04`. | Z3 | FV-1 | tools/fv/recurse.smt2 | G2 |
| OB-03-11 | Macro expansion introduces no binding visible outside its hygiene context and cannot capture a caller's local (PROP on generated macro programs; invariant: context tags on binding pairs). | PROP + REVIEW | FV-1 | macro hygiene tests | G1 |
| OB-03-12 | Every impl header is classified legal or illegal with no third outcome, over the 64 shapes of OB-03-4 doubled by trait-locality. | EXH | FV-0 | tools/fv orphan checker | G3 |
| OB-03-13 | Glob ambiguity is detected exactly per §3.1.14 for all import configurations of ≤ 3 globs × ≤ 3 names. | EXH | FV-0 | tools/fv resolution checker | G1 |
| OB-03-14 | The export-surface closure equals the set of names reachable via public paths, for bounded module trees (≤ 40 nodes). | EXH | FV-0 | tools/fv surface checker | G15 |
| OB-03-15 | Method resolution (§3.1.6 M1-M4) terminates in O(chain × methods) and returns the unique pinned candidate for bounded receiver chains (≤ 5 deref steps). | EXH + PROP | FV-1 | tools/fv method checker | G1 |

## 3.6 Coverage ledger

bullets in scope: 67, defined: 67. Prose requirements in scope: 3, defined: 3.

| design.md bullet | Defined in |
| ---------------- | ---------- |
| L237 (prose) resolution deserves its own specification | §3.0, §3.1 |
| L241 Identifier namespaces | §3.1.1 |
| L242 Lexical scopes | §3.1.2 |
| L243 Module scopes | §3.1.3 |
| L244 Type namespaces | §3.1.4 |
| L245 Value namespaces | §3.1.5 |
| L246 Trait namespaces | §3.1.6 |
| L247 Macro namespaces, if applicable | §3.1.7 |
| L248 Generic parameter scopes | §3.1.8 |
| L249 Shadowing | §3.1.9 |
| L250 Name visibility | §3.1.10 |
| L251 Name imports | §3.1.11 |
| L252 Wildcard imports | §3.1.12 |
| L253 Re-exports | §3.1.13 |
| L254 Glob ambiguity | §3.1.14 |
| L255 Qualified paths | §3.1.15 |
| L256 Relative paths | §3.1.16 |
| L257 Absolute paths | §3.1.17 |
| L258 `self`, `Self`, `super`, and `crate` | §3.1.18 |
| L259 Name hygiene | §3.1.19 |
| L260 Unresolved names | §3.1.20 |
| L261 Ambiguous names | §3.1.21 |
| L262 Recursive declarations | §3.1.22 |
| L263 Forward declarations | §3.1.23 |
| L267 (prose) define the complete compilation unit model | §3.0, §3.2 |
| L269 Source modules | §3.2.1 |
| L270 Nested modules | §3.2.2 |
| L271 External modules | §3.2.3 |
| L272 Packages or crates | §3.2.4 |
| L273 Package roots | §3.2.5 |
| L274 Module ownership | §3.2.6 |
| L275 Public exports | §3.2.7 |
| L276 Private declarations | §3.2.8 |
| L277 Dependency graphs | §3.2.9 |
| L278 Circular dependency handling | §3.2.10 |
| L279 Compilation order | §3.2.11 |
| L280 Separate compilation | §3.2.12 |
| L281 Module initialization | §3.2.13 |
| L282 Package versioning | §3.2.14 |
| L283 Conditional modules | §3.2.15 |
| L284 Platform-specific modules | §3.2.16 |
| L285 Module-level generic declarations | §3.2.17 |
| L286 Interface metadata | §3.2.18 |
| L287 Compilation cache boundaries | §3.2.19 |
| L293 Variables | §3.3.1 |
| L294 Constants | §3.3.2 |
| L295 Static variables | §3.3.3 |
| L296 Functions | §3.3.4 |
| L297 Generic functions | §3.3.5 |
| L298 Structs | §3.3.6 |
| L299 Tuple structs | §3.3.7 |
| L300 Unit structs | §3.3.8 |
| L301 Enums | §3.3.9 |
| L302 Unions | §3.3.10 |
| L303 Traits | §3.3.11 |
| L304 Implementations | §3.3.12 |
| L305 Type aliases | §3.3.13 |
| L306 Associated constants | §3.3.14 |
| L307 Associated types | §3.3.15 |
| L308 Foreign declarations | §3.3.16 |
| L309 External statics | §3.3.17 |
| L310 Methods | §3.3.18 |
| L311 Inherent implementations | §3.3.19 |
| L312 Trait implementations | §3.3.20 |
| L313 Visibility | §3.3.21 |
| L314 Declaration conflicts | §3.3.22 |
| L315 Duplicate definitions | §3.3.23 |
| L316 Recursive types | §3.3.24 |
| L317 Incomplete declarations | §3.3.25 |
| L319 (prose) Deliverables: Name Resolution, Module System, Declaration Semantics Specifications | §3.0, this file |
