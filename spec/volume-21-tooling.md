CEP:FILE: spec/volume-21-tooling.md
CEP:WHAT: Normative specification of the Planck developer toolchain: the language-server contract with a closed capability list, the idempotence-bound formatter, the documentation generator, the debugger (DAP) and profiler interfaces, the test harness contract, and the complete developer command set with exact CLI signatures and exit codes.
CEP:WHY: design.md §28 (lines 3243–3361) declares tooling a production requirement, not optional decoration; every listed capability needs an exact contract so that editors, CI, and the verification infrastructure (Volume 22) can rely on stable, deterministic tool behavior.
CEP:STATUS: complete
CEP:FAILURE: A tool that produces nondeterministic output for identical inputs, a formatter that is not idempotent, a diagnostic transport that loses or reorders diagnostics, or a command that exits with an undocumented code is non-conforming (Severity 1 or higher per standards.md §34).
CEP:ASSUMES: SPEC-INDEX.md §1, §3 (effect algebra, ownership state machine, targets); standards.md §3 (normative verbs), §5 (CEP classes), §13 (measurement law); Volumes 02 (grammar), 04 (type printer), 05 (layout), 07 (effects), 08 (task state), 12 (limits), 13–15 (IR formats), 15 (instruction table), 18 (runtime modes), 20 (build system, cache keys, exit-code base), 22 (fuzzing protocol), 23 (benchmark protocol).
CEP:COST: Tooling: LSP server, formatter, doc generator, DAP adapter, profiler agent are CEP-1/CEP-2; the profiler sampling agent is CEP-0; test harness orchestration is CEP-1.
CEP:EVIDENCE: Verification matrix (OB-21-1..OB-21-16) and coverage ledger (bullets in scope: 88, defined: 88) in this file.

---

## 21.0 Scope and traceability

This volume defines design.md **§28 Toolchain ecosystem** (lines 3243–3361). Bullet inventory: §28.1 = 16, §28.2 = 9, §28.3 = 11, §28.4 = 13, §28.5 = 11, §28.6 = 11, §28.7 = 17. Bullets in scope: 88, defined: 88 (ledger at end of file). The §28 prose directive "This is a production requirement, not optional decoration" is honored normatively: every capability below carries a verification row; a capability without an obligation row is a definition failure (G15).

Global conventions. All tools in this volume: (a) accept and emit UTF-8; (b) use the exit codes 0 = success, 1 = failure of the requested operation, 2 = usage error, 3 = internal error (a Severity 0 compiler/tool defect); (c) support `--format=json` with the per-command schema stated below; (d) are deterministic — identical inputs produce identical outputs, byte for byte, on every run (G14); (e) never read wall-clock time, host name, user name, or environment variables outside their documented set. Machine-readable outputs are canonical JSON per Volume 20 §20.0.1. Limits follow Volume 12.

## 21.1 Language server (defines design.md 28.1, bullets B28-1..B28-16)

The server is `plkc lsp`, speaking JSON-RPC 2.0 over stdio per the LSP 3.17 specification. **Position encoding**: the server advertises `positionEncoding: "utf-8"`; if the client does not support it, the server falls back to UTF-16 and states the negotiated encoding in its `initialize` result. The analysis engine is the compiler's query system; invalidation follows Volume 20 §20.5 exactly.

**Capability list (closed).** The server sends exactly these capabilities and no others; adding a capability is an edition change:

| Capability | Value |
| --- | --- |
| `textDocumentSync` | `{openClose: true, change: 2 (incremental), save: {includeText: false}}` |
| `completionProvider` | `{triggerCharacters: [".", ":"], resolveProvider: true}` |
| `hoverProvider` | true |
| `definitionProvider` | true |
| `referencesProvider` | true |
| `documentSymbolProvider` | true |
| `workspaceSymbolProvider` | true |
| `codeActionProvider` | `{codeActionKinds: ["quickfix", "refactor"]}` |
| `renameProvider` | `{prepareProvider: true}` |
| `semanticTokensProvider` | `{full: {delta: true}, legend: §21.1 legend}` |
| `foldingRangeProvider` | true |
| `documentFormattingProvider` | true |
| `executeCommandProvider` | `{commands: ["planck.insertUse"]}` |
| `positionEncoding` | `"utf-8"` |

Custom methods (closed list): `planck/expandDocument`, `planck/explainBorrow`, `planck/explainConstraint`, `planck/effectInfo`. Unknown `planck/*` requests return method-not-found; unknown standard requests fall through per LSP.

**Semantic tokens legend (exact).** Token types by index: 0 `type`, 1 `function`, 2 `method`, 3 `variable`, 4 `parameter`, 5 `property`, 6 `enumMember`, 7 `keyword`, 8 `modifier`, 9 `comment`, 10 `string`, 11 `number`, 12 `operator`, 13 `macro`, 14 `genericParameter`, 15 `effect`. Token modifiers by bit: 0 `definition`, 1 `documentation`, 2 `deprecated`, 3 `async`. Tokens are emitted only for resolved identifiers and fixed syntax; a token's classification is a pure function of the resolved document.

**Completion** (B28-1). Items carry one rank: keywords 0; snippets 1; in-scope locals 2; members of the receiver's type 3; module-visible items 4; import suggestions 5. Sort key = (rank ascending, prefix-match descending, label ascending), where prefix-match is true iff the label starts with the (case-insensitive) typed prefix. The item list for a given (document version, position) is deterministic (OB-21-1). `resolveProvider` fills `documentation` lazily from the doc-comment text.

**Hover information** (B28-2). Contents = `MarkupContent{kind: "markdown"}`; first line = the item's signature rendered by the Volume 04 type printer; body = the doc-comment text up to the first blank line; `range` = the identifier's span. Hover is a pure function of (document, position).

**Go to definition** (B28-3). Returns the primary span of the binding item. If the target is produced by macro expansion, the server returns the macro invocation site's span (defined fallback, not an error).

**Find references** (B28-4). Honors `context.includeDeclaration`; results sorted by (file path ascending, position ascending). References inside macro-generated code are attributed to the expansion site.

**Rename** (B28-5). `prepareRename` returns the identifier span and placeholder. `rename` applies only to syntactically written identifiers; renaming a macro-generated identifier fails with `InvalidRename::MacroGenerated`; renaming an attribute path fails with `InvalidRename::Attribute`. The rename edits all workspace references; if the new name would capture a different binding at any reference site (shadowing check per Volume 03 rules), the rename fails with `InvalidRename::CaptureConflict` naming the first conflicting site in canonical order.

**Type information** (B28-6). Types are surfaced through hover and through `documentSymbol`/`planck/effectInfo`; the rendered type string is exactly the Volume 04 canonical type printer output. No other type-information channel exists in edition P1.

**Diagnostics** (B28-7). Diagnostics are published per open document, full replacement, tagged with the document version; publication occurs after a change once the document has been idle 100 ms or on save, whichever is first. Diagnostic codes are the stable codes of the diagnostic registry (Volumes 01–19); this volume defines transport only: a diagnostic's (code, range, severity, message) for a given document version is deterministic (OB-21-1). No diagnostic is dropped silently (G22).

**Code actions** (B28-8). The quickfix set is the closed diagnostic-to-fix mapping of the diagnostic registry: each code with a defined fix yields exactly one quickfix; codes without a defined fix yield none. The `refactor` kind contains exactly: rename (delegates to `renameProvider`) and insert-import (`planck.insertUse`). No other actions are sent.

**Import suggestions** (B28-9). An unresolvable-identifier diagnostic carries a code action that inserts `use <path>::<Name>;`. Insertion position: the import block after module doc comments and attributes; ordering: three groups (external crates, crate-internal, current-crate items) each sorted alphabetically, groups separated by one blank line. The action is deterministic for the diagnostic.

**Refactoring** (B28-10). The closed refactoring set of edition P1 is: rename, insert-import. Every other refactoring (extract-function, inline, etc.) is declared-not-implemented and must not be advertised.

**Semantic highlighting** (B28-11). Emission per the legend above; unresolved identifiers emit no token; macro-generated identifiers emit token type 13 (`macro`) with modifier bit 1. Delta encoding per LSP `full: {delta: true}`.

**Macro expansion inspection** (B28-12). `planck/expandDocument` returns an array of `{range, expandedText}` for every macro invocation in the document, sorted by range start; `expandedText` is the canonical token spelling of the expansion (single spaces between tokens, per §21.2 Macro formatting spacing).

**Borrow explanation** (B28-13). `planck/explainBorrow(position)` returns `{place, state, conflictingActions, chain}` where `state` is a state of the pinned ownership machine (SPEC-INDEX §5.4), `conflictingActions` lists the actions rejected from that state, and `chain` is the ordered list of program points that produced the state. The result is a pure function of the resolved document.

**Generic constraint explanation** (B28-14). `planck/explainConstraint(position)` returns `{goal, candidateImpls, reason}`: the failed trait goal in canonical form, the impl candidates considered (paths, sorted), and the reason class from the Volume 04 solver (`NoImpl`, `Ambiguous`, `UnificationFailure`).

**Effect inspection** (B28-15). `planck/effectInfo(position)` returns `{row: [atoms sorted ascending], rowOpen: bool}` for the enclosing function; atoms are members of the pinned closed atom set {`io`, `alloc`, `panic`, `unsafe`, `async`, `concurrency`, `ffi`, `nondet`} (SPEC-INDEX §5.3).

**Incremental analysis** (B28-16). The server reuses the compiler's query system; unchanged queries must not be recomputed (must; verified by OB-21-3). Responsiveness budget on the Volume 23 reference machine: p95 query latency ≤ 100 ms for documents ≤ 2000 lines; exceeding the budget is a performance defect tracked by Volume 23 §23.3 Background analysis, not a correctness failure.

## 21.2 Formatter (defines design.md 28.2, bullets B28-17..B28-25)

The formatter is `plkc fmt`, operating on parseable Planck source. Idempotence obligation: `format(format(x)) = format(x)` for every parseable `x` (OB-21-2, EXH over the pinned corpus plus PROP over grammar-generated trees, paired per SPEC-INDEX §4.3).

**Formatting rules** (B28-17). The rule set is closed and ordered as follows; the output is the unique fixpoint of these rules:

| # | Rule |
| --- | --- |
| 1 | Indentation = 4 spaces per nesting level; tab characters are banned in output. |
| 2 | Maximum line width = 100 columns; overflow breaks at the highest-priority legal break point: after `,` > after a binary operator > after `(`; a token longer than the width is placed on its own line unsplit. |
| 3 | Opening brace on the same line as its introducing token, preceded by one space; closing brace alone on a line at the enclosing indentation. |
| 4 | Trailing commas inserted in multi-line list contexts, removed in single-line contexts. |
| 5 | Exactly one blank line between adjacent items; runs of blank lines collapse to one; no blank line immediately after `{` or before `}`. |
| 6 | One space around binary operators; no space after unary `!`, `-`, `&`, `*`; no space before `(`; one space before `{`. |
| 7 | Method chains with more than 3 links, or any chain exceeding the width, break one link per line at indent + 1. |
| 8 | String literals, char literals, comments, and numeric literals are copied verbatim; bases and spellings are never rewritten. |
| 9 | Import lines are sorted within each contiguous run; run boundaries (blank lines) are preserved. |
| 10 | Attributes on their own line, in source order (see B28-21). |

**Stability** (B28-18). The idempotence property above holds unconditionally for parseable inputs; the corpus for the EXH check is `tests/fmt/corpus/` (hash-pinned, ≥ 5000 files at adoption, growth append-only), and the PROP generator is the Volume 02 grammar-directed generator of Volume 22 §22.5 at depth ≤ 12. A stability failure is a Severity 0 tool defect.

**Configuration** (B28-19). `planck-fmt.json` at the workspace root, canonical JSON, fields exactly: `max_width` (integer 40..=200, default 100), `indent` (enum {2, 4, 8}, default 4), `reorder_imports` (bool, default true), `generated_handling` (enum {`skip`, `format`}, default `skip`). Unknown fields are rejected (`FmtError::UnknownConfigField`). `indent` overrides rule 1's default; other rules are not configurable.

**Comments** (B28-20). Comment text is never modified. Attachment rule: a standalone comment line attaches to the following syntactic item unless a blank line separates them, in which case it attaches to the preceding item; inline comments remain on their code line at one space after the code.

**Attributes** (B28-21). Each attribute occupies its own line, in source order; attributes are never reordered, merged, or deduplicated (order and repetition are semantically significant per Volume 10 §15).

**Macro formatting** (B28-22). The formatter does not parse macro bodies. Macro invocation content is re-emitted as a token stream with canonical spacing: one space between tokens; no space after `(` or before `)`; no space before `,` or `;`; one space after `,` and `;`. This re-emission participates in the idempotence obligation (OB-21-2 covers macro-bearing inputs).

**Generated code behavior** (B28-23). A file whose first 3 lines contain the marker `// @generated` is skipped: no write, exit 0, reported in `--format=json` output as `{"skipped": [paths]}`. `--generated=format` formats generated files anyway.

**Error recovery** (B28-24). Unparseable input: exit 2, the parse diagnostics of Volume 11 are printed to stderr, and no file is modified. Writes are atomic: the formatter writes to a sibling temporary file and renames only after a successful full-file format.

**Version compatibility** (B28-25). For the same (formatter version, configuration, input), output bytes are identical on every platform. Within edition P1, the configuration schema is additive-only: new fields receive their defaults in old configs, and no existing field changes meaning. Formatting-style changes across compiler minor versions are permitted; idempotence must hold for every version.

## 21.3 Documentation system (defines design.md 28.3, bullets B28-26..B28-36)

The documentation generator is `plkc doc`, producing a static HTML site under `target/doc/`. Output is deterministic: the same inputs produce byte-identical HTML (G14, OB-21-4); no timestamps, no absolute paths, no generator-version strings in the HTML.

**API documentation** (B28-26). Doc comments are `///` (line) and `/** ... */` (block); they attach to the following item. Each documented item renders one page with, in order: canonical path, signature block (Volume 04 printer), effect row, doc body, generic parameters, `# Safety` section if applicable, `# Examples` section, trait implementations. Undocumented public items render with an explicit "No documentation" body (never omitted silently).

**Module documentation** (B28-27). `//!` comments render at the top of the module page; the module tree mirrors the source tree; `src/lib.plk` docs render as the crate root page.

**Code examples** (B28-28). Fenced code blocks in doc comments use the closed language-tag set: `planck` (compiled and executed as a doc test), `planck,no_run` (compiled, not executed), `planck,ignore` (neither compiled nor executed), `text` (verbatim). Any other tag is rendered verbatim and not compiled. Doc-test failures fail `plkc doc --test` (exit 1).

**Cross-references** (B28-29). The link syntax `` [`Path::to::Item`] `` resolves against the item index of the documented crate graph. An unresolved link produces `DocWarning::UnresolvedXref` with the literal text; `--deny-warnings` turns all doc warnings into failures.

**Generic documentation** (B28-30). Generic parameters render with their bounds; instance documentation renders substitutions as `Name<T1, T2>` with the canonical type strings.

**Trait documentation** (B28-31). A trait page lists: required methods, provided methods, and all implementors visible in the build graph, sorted by canonical path; each implementor links to its impl page.

**Effect documentation** (B28-32). Every function signature line includes its effect row in `{atom, atom}` form with atoms sorted ascending; a generated legend page defines each atom of the pinned set (SPEC-INDEX §5.3). Open rows render as `{..}` with the bound atoms listed.

**Safety documentation** (B28-33). Every `unsafe` item must carry a `# Safety` section; absence produces `DocWarning::MissingSafety`. This check is mechanical and part of the doc test pass.

**Example execution** (B28-34). `planck` doc examples are compiled and executed by the test harness (§21.6) with a per-example timeout of 30 seconds; a timeout, non-zero exit, or panic is a doc-test failure with the captured output attached. Examples marked `no_run` or `ignore` are exempt from execution but `no_run` examples must still compile.

**Search** (B28-35). The generator emits `search-index.json`: a trigram index over item paths plus the first 512 bytes of each item's doc text. Total index size must be ≤ 67 108 864 bytes (64 MiB); exceeding = `DocError::IndexTooLarge`. Query semantics: trigram intersection over the query string, ranked by (name match descending, path length ascending); search runs client-side; no server component exists.

**Versioned documentation** (B28-36). Output layout: `target/doc/<version>/…` where `<version>` is the package version string; `target/doc/index.html` lists previously generated versions found on disk, sorted by version ascending. Regeneration overwrites only the current version's directory.

## 21.4 Debugger support (defines design.md 28.4, bullets B28-37..B28-49)

The debugger interface is the DAP adapter `plkc dap` (Debug Adapter Protocol 1.11) over stdio, driving the compiler's debug output; the machine-level contract is DWARF 5 on both implemented targets.

**Debug symbols** (B28-37). `-g` levels: 0 = none, 1 = line tables only, 2 = full (variables, types, CFI), 3 = full plus macro information. Profile defaults: dev = 2, release = 0. Symbols are emitted deterministically (G14).

**Variable locations** (B28-38). Each local variable is described by a DWARF location list; at PCs where no location is live the variable is reported `unavailable`. The compiler must not fabricate locations; a wrong location is a Severity 0 defect.

**Source mapping** (B28-39). Every emitted instruction carries a line-table entry derived from its source span (must); the line table is part of the reproducibility contract (byte-identical builds, OB-21-5).

**Type inspection** (B28-40). Every type used by the unit, including `#[repr(P)]` layout results (Volume 05 layout math), has a DWARF type unit; layout fields (size, alignment, member offsets) match the Volume 05 computation exactly (OB-21-6, EXH over the layout corpus).

**Generic type inspection** (B28-41). Monomorphized instances are named `Base<!T1,T2>`: the source name, `<`, the canonical type strings joined by `,` (no space), `>`. This rendering is used in DWARF `DW_AT_name` and in all tool output.

**Trait object inspection** (B28-42). The vtable header carries a concrete-type id slot whose layout is fixed by Volume 18 §25; the DAP reads that slot and prints the concrete type name resolved through the type registry. Inspection of a non-trait-object pointer returns `EvalError::NotSupported` rather than guessing.

**Closure inspection** (B28-43). Closure captures are emitted as named fields with their source names (capture struct layout per Volume 07 §10); the DAP displays the capture struct with these names, plus the closure's function signature.

**Optimized variable tracking** (B28-44). At opt-level > 0, variable availability follows the backend's location lists; the guaranteed property is line-table completeness (B28-39), not variable availability. This is a documented refinement gap (FV-1, class per §54.2.2) with the runtime bridge: unavailable variables are reported as `unavailable`, never as wrong values (OB-21-7, REVIEW).

**Async state inspection** (B28-45). Task state structs (Volume 08 §12) are emitted as DWARF structs; the current await point is exposed as the enum-typed variable `__await_state`; the DAP displays the task tree from the runtime's task registry (Volume 18).

**Stack unwinding** (B28-46). `.eh_frame` CFI is emitted for every function with a stack frame (must); unwinding behavior, including panic propagation, is defined by Volume 18 §25. A frame without CFI is a Severity 0 defect because it breaks both the debugger and the runtime unwinder.

**Breakpoints** (B28-47). Kinds: source line, function, exception category `panic` (the only exception breakpoint filter). Source breakpoints snap to the nearest line ≥ the requested line that has an executable line-table entry in the same file; if none exists within 20 lines, the DAP returns resolution `failed` with reason `"no executable code"`. Breakpoint state (verified/pending/failed) is deterministic.

**Watchpoints** (B28-48). Data breakpoints on memory ranges use hardware watchpoints: at most 4 concurrently, width ∈ {1, 2, 4, 8} bytes, address aligned to the width. Requests beyond these limits fail with `EvalError::NotSupported`; the failure is reported, never silently downgraded.

**Expression evaluation** (B28-49). `evaluate` supports the closed expression subset: variable paths, field access `.name`, pointer dereference `*e`, index with an integer constant `e[n]`, and casts `(T)e` between integer types. Function calls in expressions are declared-not-implemented in edition P1 (returns `NotSupported`). Errors are the closed set `EvalError::{Unavailable, NotSupported, Syntax}`; evaluation never reports a value it cannot prove from DWARF data.

## 21.5 Profiling (defines design.md 28.5, bullets B28-50..B28-60)

The profiler is `plkc prof run -- <command>`; measurement follows standards.md §13 (pinned environment, documented configuration, stored artifacts). All profiler outputs are deterministic for identical inputs and counters (OB-21-8).

**CPU profiling** (B28-50). `--mode cpu` runs the target under the sampling profiler: samples of (monotonic timestamp ns, thread id, call stack of ≤ 128 frames) walked via `.eh_frame` CFI. Output: `prof-out.folded` (collapsed stacks, `frame;frame;frame count` lines sorted lexicographically) and `prof-out.json` (raw sample events).

**Sampling** (B28-51). Default interval 1000 µs; `--interval` accepts exactly {100, 250, 500, 1000, 2000} µs. The sampling agent is CEP-0: allocation-free steady state, no locks on the sample path, bounded buffer (16384 samples; on overflow the agent drops the oldest sample and increments a dropped-counter recorded in the output).

**Instrumentation** (B28-52). `-C instrument=functions` inserts one u64 counter per function; increments are saturating (overflow is not a trap and not UB); instrumentation must not change observable program behavior (G4 obligations carry over; OB-21-9). Instrumented builds report counters at exit via the runtime's status channel (Volume 18).

**Allocation profiling** (B28-53). The runtime allocator hook (Volume 18) records one event per alloc/free: (site id, size class, timestamp ns, thread id); the site id is the static call-site index from debug metadata. Output: allocation histogram by site and size class plus a JSONL timeline; events are ordered by timestamp with ties broken by thread id.

**Memory profiling** (B28-54). `--mode heap` walks the heap at the configured interval (default 10 ms; allowed set {1, 5, 10, 50, 100} ms) and emits one size-class histogram per snapshot; snapshots are numbered deterministically from 0.

**Call graphs** (B28-55). Derived from sample stacks: a caller→callee edge for each adjacent frame pair, weighted by sample count; recursion is merged into a cycle node with an explicit self-weight. The graph is emitted as `prof-out.graph.json` (nodes sorted by name, edges sorted by (caller, callee)).

**Branch profiling** (B28-56). Branch metrics come from hardware counters (B28-57); the report lists per-function `branch-instructions` and `branch-misses` with the derived miss ratio in fixed-point per-mille (integer arithmetic: misses × 1000 / branches, saturating).

**Hardware counters** (B28-57). The counter set is closed: `cycles`, `instructions`, `branch-instructions`, `branch-misses`, `cache-references`, `cache-misses`, `L1-dcache-loads`, `L1-dcache-load-misses` (Linux `perf_event_open`). Counters are attributed to functions by DWARF symbolization. If the perf subsystem is unavailable, `plkc prof` fails with `ProfError::PerfUnavailable` (exit 1) rather than approximating.

**Async task profiling** (B28-58). Task lifecycle events {`spawn`, `poll-begin`, `poll-end`, `complete`, `cancel`} are emitted with (task id, timestamp ns, thread id) into the profiler JSONL stream; the event set is closed and ordered by (timestamp, task id).

**Compiler pass profiling** (B28-59). `plkc build --profile-passes` emits `pass-profile.json`: per query kind {`kind`, `count`, `total_ns`, `peak_bytes`}, keys sorted ascending. Values come from the same instrumentation as Volume 23 §23.1 phase budgets; the schema is identical to the Volume 23 measurement schema so that budgets and reports are comparable.

**Optimization remarks** (B28-60). `-C remark=all|<pass>` emits one remark per optimization decision: `{"pass", "function", "decision", "span"}` in JSONL, ordered by (function, application order). The stream is deterministic (G14) and is the machine-readable form of the `plkc opt-decisions` command (§21.7).

## 21.6 Testing tools (defines design.md 28.6, bullets B28-61..B28-71)

The test harness is `plkc test`; it owns discovery, compilation, execution, filtering, and reporting. Harness output is deterministic: the report for a given test set is byte-identical regardless of thread count (OB-21-10).

**Unit tests** (B28-61). A unit test is a zero-argument function annotated `#[test]` returning unit or `Result<T, E>`; failure = returned error or panic (message captured). Tests execute in-process in the test binary; default worker count = min(8, logical CPUs); `--test-threads N`, N ∈ 1..=256. Captured stdout is shown only for failing tests unless `--nocapture`.

**Integration tests** (B28-62). Each `tests/*.plk` file is one binary crate linking the package library, executed as a separate process; process exit 0 = pass. Integration tests use the dev profile by default (`--profile` overrides).

**Compile-fail tests** (B28-63). Files under `tests/compile-fail/` carry one annotation per expected diagnostic: `// @error:<CODE>:<substring>`. The test passes iff compilation fails and every annotation matches (code exactly; substring contained in the message) and no diagnostic lacks an annotation. A compile-fail test that compiles cleanly fails the suite.

**Runtime tests** (B28-64). Tests may use effects {`io`, `alloc`, `panic`}; they run under the normal runtime with the standard traps (SPEC-INDEX §3). Contracts are otherwise identical to unit tests.

**Benchmark tests** (B28-65). `#[bench]` functions run under the Volume 23 §23.10 protocol: 3 warmup iterations, then measured iterations until relative MAD < 2% or 100 iterations maximum (≥ 20 required); the harness reports median and MAD. Benchmark execution must not run under sanitizers or instrumentation.

**Documentation tests** (B28-66). Doc tests (§21.3 Example execution) run under `plkc test --doc`; each example is reported as one test named `doctest::<file>::<index>`; the pass criteria are §21.3's.

**Fuzz tests** (B28-67). `#[fuzz]` functions take one generated input and express a property; the generator, budget, and corpus rules are Volume 22 §22.5 exactly. Defaults: 10^5 executions per run; `--fuzz-runs` ∈ 1..=10^7; the seed is derived deterministically from the test path.

**Sanitizer execution** (B28-68). `--sanitizer=address|thread` builds with the runtime's instrumented allocator / scheduler-stress modes (Volume 18 §25); `--sanitizer=memory` enables the provenance-and-validity checking runtime (Volume 05 §7 checks). A sanitizer report is a test failure; the process trap exit code 134 is treated as failure with the trap report attached.

**Test filtering** (B28-69). `--filter SUB` selects tests whose full path `crate::module::name` contains SUB as a substring; `--exact` requires full-path equality; repeated `--filter` flags form a union. Filtering is deterministic and does not change execution order semantics.

**Parallel execution** (B28-70). Tests are distributed by a work-stealing scheduler; aggregation is deterministic: results are reported sorted by test path ascending, and the totals (passed, failed, ignored) are identical for every worker count. Scheduling order never affects any reported value (OB-21-10).

**Test discovery** (B28-71). Discovery is the compile-time AST scan for test attributes (`#[test]`, `#[bench]`, `#[fuzz]`, doc tests); the discovered list is sorted lexicographically. Completeness obligation: for every corpus file, the discovered set equals the set derived from a second, independent AST pass (OB-21-11, EXH over the corpus).

## 21.7 Developer commands (defines design.md 28.7, bullets B28-72..B28-88)

All commands share: exit codes 0/1/2/3 (§21.0); `--format=json` machine output with the schema stated; `--manifest-dir <dir>` to select the package; deterministic output. Flags not listed are usage errors (exit 2).

**Building** (B28-72). `plkc build [--release|--debug] [--target T] [--features F1,F2] [-j N] [--locked]`. Compiles per Volume 20 §20.3; stdout carries progress lines in node-id order; on success prints the artifact paths; exit 1 on any build error with the diagnostics on stderr.

**Testing** (B28-73). `plkc test [filters] [--doc] [--sanitizer M] [--test-threads N]`. Runs the harness of §21.6; report on stdout; exit 1 iff any test fails.

**Formatting** (B28-74). `plkc fmt [--check] [--generated=skip|format] [paths]`. `--check` writes nothing and exits 1 iff any file would change; without `--check`, rewrites per §21.2 with atomic writes.

**Documentation** (B28-75). `plkc doc [--test] [--deny-warnings] [--open]`. Builds the doc site of §21.3; `--test` runs doc tests; `--open` launches the configured browser (`PLANCK_BROWSER` env, the only environment variable read).

**Benchmarking** (B28-76). `plkc bench [filters] [--baseline NAME]`. Runs `#[bench]` targets under the Volume 23 §23.10 protocol; with `--baseline`, compares against the stored baseline and applies the Volume 23 §23.8 regression gates.

**Checking without code generation** (B28-77). `plkc check`. Runs the full frontend pipeline (lexing through borrow check and effect inference) and emits diagnostics only; writes no artifacts and performs no code generation; identical diagnostics to `plkc build` for the same inputs (OB-21-12).

**Cleaning** (B28-78). `plkc clean [--release|--debug]`. Deletes exactly: `target/planck-cache/` and the artifact directories recorded in the build manifests of the selected profile; refuses to delete anything not recorded (exit 2 if a manifest is missing and artifacts exist, unless `--force`).

**Inspecting dependencies** (B28-79). `plkc deps [--depth N] [--locked]`. Prints the resolved dependency tree from the lockfile as an ASCII tree; children sorted by (name, version); N ∈ 0..=64, default 64. With `--format=json`, emits the lockfile graph as `{nodes, edges}` in canonical order.

**Inspecting compiler diagnostics** (B28-80). `plkc explain <CODE>`. Prints the long-form explanation for a stable diagnostic code from the diagnostic registry: title, cause, and the fix list of the registry entry. Unknown code = exit 2.

**Emitting IR 1** (B28-81). `plkc emit ir1 [--out DIR]`. Writes the canonical serialized IR 1 (Volume 13 §20 format) per function to `target/emit/ir1/`, one file per function, named by canonical function path; output is deterministic (G14).

**Emitting IR 2** (B28-82). `plkc emit ir2 [--out DIR]`. Same contract with the Volume 14 §21 format and `target/emit/ir2/`.

**Emitting Machine IR** (B28-83). `plkc emit mir [--out DIR]`. Same contract with the Volume 15 §22 format and `target/emit/mir/`; requires `--target` for target-dependent forms.

**Disassembling** (B28-84). `plkc disasm <artifact>`. Disassembles per the Volume 15 instruction table: one line per instruction = `address  bytes-hex  mnemonic operands`; output is deterministic and derived only from the artifact bytes and the instruction table.

**Inspecting assembly** (B28-85). `plkc asm <unit>`. Prints the pre-encoding assembly listing of the unit's MIR (same line format as `disasm`, plus the register-class annotation column); used for code review of instruction selection.

**Inspecting optimization decisions** (B28-86). `plkc opt-decisions <unit>`. Prints the remark stream of §21.5 Optimization remarks for the unit: pass, decision, span, one per line; `--format=json` emits the JSONL verbatim.

**Inspecting layout** (B28-87). `plkc layout <type>`. Prints name, size, alignment, and per-field offsets computed by the Volume 05 layout rules; matches the DWARF type unit (OB-21-6).

**Inspecting type information** (B28-88). `plkc types <item>`. Prints the signatures of the item and of all items in its scope, rendered by the Volume 04 printer, sorted by canonical path; matches the hover rendering exactly.

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-21-1 | LSP determinism: completion, hover, diagnostics, and semantic tokens are pure functions of (document version, position); EXH over the editor corpus replay (every corpus file × every defined mutation). | EXH | FV-1 | CI LSP replay job | G14, G22 |
| OB-21-2 | Formatter idempotence: format(format(x)) = format(x) for every parseable x; EXH over `tests/fmt/corpus` plus PROP over grammar-generated trees (depth ≤ 12), paired. | EXH+PROP | FV-0 | `tests/fmt/` + `tools/fv/fmt_idempotence` | G3, G14 |
| OB-21-3 | LSP incremental reuse: unchanged queries are not recomputed (query-cache hit accounting on the replay corpus). | EXH | FV-1 | CI LSP replay job | G23 |
| OB-21-4 | Doc determinism: byte-identical HTML for identical inputs; EXH over the doc corpus. | EXH | FV-1 | CI doc job | G14 |
| OB-21-5 | Line-table completeness: every emitted instruction has a line-table entry; EXH over the object corpus (scan all instruction/table pairs). | EXH | FV-0 | `tools/fv/line_table.z3` + CI | G1, G14 |
| OB-21-6 | DWARF layout fidelity: DWARF type units match Volume 05 layout math for every type in the layout corpus. | EXH | FV-0 | `tests/debuginfo/` | G19 |
| OB-21-7 | Debug availability honesty: no fabricated variable locations; unavailable variables are reported unavailable. | REVIEW | FV-1 | Debug-info review record | G22 |
| OB-21-8 | Profiler determinism: identical inputs and counters produce byte-identical folded/graph outputs. | EXH | FV-1 | CI profiler job | G14 |
| OB-21-9 | Instrumentation neutrality: instrumented builds preserve observable behavior on the conformance corpus (differential vs. uninstrumented). | EXH | FV-1 | CI differential job | G4 |
| OB-21-10 | Harness determinism: test report bytes independent of worker count; EXH over thread counts {1,2,4,8} × corpus suites. | EXH | FV-0 | CI harness job | G14 |
| OB-21-11 | Discovery completeness: discovered test set equals the independent AST-scan set for every corpus file. | EXH | FV-0 | `tools/fv/discovery.z3` | G15 |
| OB-21-12 | check/build diagnostic parity: `plkc check` emits the identical diagnostic stream as `plkc build` for the same inputs. | EXH | FV-0 | CI parity job | G22 |
| OB-21-13 | Exit-code contract: every command maps each failure class to exactly the documented exit code; EXH over the command × failure matrix. | EXH | FV-0 | `tools/fv/exit_codes.z3` | G22 |
| OB-21-14 | Formatter rule fixpoint: the rule table of §21.2 has a unique output per input (two independent implementations agree on the corpus). | EXH | FV-1 | `tests/fmt/crosscheck/` | G14 |
| OB-21-15 | Breakpoint snapping: source breakpoints resolve per the 20-line rule, deterministically; EXH over the line-table corpus. | EXH | FV-1 | `tools/fv/bpsnap.smt2` | G22 |
| OB-21-16 | Capability closure: the advertised capability set equals the closed list of §21.1; REVIEW of the initialize response against the table. | REVIEW | FV-2 | LSP contract test | G15 |

## Coverage ledger

design.md §28, lines 3243–3361. Bullets in scope: 88, defined: 88.

| # | Bullet (abbreviated) | Defined in |
| --- | --- | --- |
| B28-1 | Completion | §21.1 Completion |
| B28-2 | Hover information | §21.1 Hover information |
| B28-3 | Go to definition | §21.1 Go to definition |
| B28-4 | Find references | §21.1 Find references |
| B28-5 | Rename | §21.1 Rename |
| B28-6 | Type information | §21.1 Type information |
| B28-7 | Diagnostics | §21.1 Diagnostics |
| B28-8 | Code actions | §21.1 Code actions |
| B28-9 | Import suggestions | §21.1 Import suggestions |
| B28-10 | Refactoring | §21.1 Refactoring |
| B28-11 | Semantic highlighting | §21.1 Semantic highlighting |
| B28-12 | Macro expansion inspection | §21.1 Macro expansion inspection |
| B28-13 | Borrow explanation | §21.1 Borrow explanation |
| B28-14 | Generic constraint explanation | §21.1 Generic constraint explanation |
| B28-15 | Effect inspection | §21.1 Effect inspection |
| B28-16 | Incremental analysis | §21.1 Incremental analysis |
| B28-17 | Formatting rules | §21.2 Formatting rules |
| B28-18 | Stability | §21.2 Stability |
| B28-19 | Configuration | §21.2 Configuration |
| B28-20 | Comments | §21.2 Comments |
| B28-21 | Attributes | §21.2 Attributes |
| B28-22 | Macro formatting | §21.2 Macro formatting |
| B28-23 | Generated code behavior | §21.2 Generated code behavior |
| B28-24 | Error recovery | §21.2 Error recovery |
| B28-25 | Version compatibility | §21.2 Version compatibility |
| B28-26 | API documentation | §21.3 API documentation |
| B28-27 | Module documentation | §21.3 Module documentation |
| B28-28 | Code examples | §21.3 Code examples |
| B28-29 | Cross-references | §21.3 Cross-references |
| B28-30 | Generic documentation | §21.3 Generic documentation |
| B28-31 | Trait documentation | §21.3 Trait documentation |
| B28-32 | Effect documentation | §21.3 Effect documentation |
| B28-33 | Safety documentation | §21.3 Safety documentation |
| B28-34 | Example execution | §21.3 Example execution |
| B28-35 | Search | §21.3 Search |
| B28-36 | Versioned documentation | §21.3 Versioned documentation |
| B28-37 | Debug symbols | §21.4 Debug symbols |
| B28-38 | Variable locations | §21.4 Variable locations |
| B28-39 | Source mapping | §21.4 Source mapping |
| B28-40 | Type inspection | §21.4 Type inspection |
| B28-41 | Generic type inspection | §21.4 Generic type inspection |
| B28-42 | Trait object inspection | §21.4 Trait object inspection |
| B28-43 | Closure inspection | §21.4 Closure inspection |
| B28-44 | Optimized variable tracking | §21.4 Optimized variable tracking |
| B28-45 | Async state inspection | §21.4 Async state inspection |
| B28-46 | Stack unwinding | §21.4 Stack unwinding |
| B28-47 | Breakpoints | §21.4 Breakpoints |
| B28-48 | Watchpoints | §21.4 Watchpoints |
| B28-49 | Expression evaluation | §21.4 Expression evaluation |
| B28-50 | CPU profiling | §21.5 CPU profiling |
| B28-51 | Sampling | §21.5 Sampling |
| B28-52 | Instrumentation | §21.5 Instrumentation |
| B28-53 | Allocation profiling | §21.5 Allocation profiling |
| B28-54 | Memory profiling | §21.5 Memory profiling |
| B28-55 | Call graphs | §21.5 Call graphs |
| B28-56 | Branch profiling | §21.5 Branch profiling |
| B28-57 | Hardware counters | §21.5 Hardware counters |
| B28-58 | Async task profiling | §21.5 Async task profiling |
| B28-59 | Compiler pass profiling | §21.5 Compiler pass profiling |
| B28-60 | Optimization remarks | §21.5 Optimization remarks |
| B28-61 | Unit tests | §21.6 Unit tests |
| B28-62 | Integration tests | §21.6 Integration tests |
| B28-63 | Compile-fail tests | §21.6 Compile-fail tests |
| B28-64 | Runtime tests | §21.6 Runtime tests |
| B28-65 | Benchmark tests | §21.6 Benchmark tests |
| B28-66 | Documentation tests | §21.6 Documentation tests |
| B28-67 | Fuzz tests | §21.6 Fuzz tests |
| B28-68 | Sanitizer execution | §21.6 Sanitizer execution |
| B28-69 | Test filtering | §21.6 Test filtering |
| B28-70 | Parallel execution | §21.6 Parallel execution |
| B28-71 | Test discovery | §21.6 Test discovery |
| B28-72 | Building | §21.7 Building |
| B28-73 | Testing | §21.7 Testing |
| B28-74 | Formatting | §21.7 Formatting |
| B28-75 | Documentation | §21.7 Documentation |
| B28-76 | Benchmarking | §21.7 Benchmarking |
| B28-77 | Checking without code generation | §21.7 Checking without code generation |
| B28-78 | Cleaning | §21.7 Cleaning |
| B28-79 | Inspecting dependencies | §21.7 Inspecting dependencies |
| B28-80 | Inspecting compiler diagnostics | §21.7 Inspecting compiler diagnostics |
| B28-81 | Emitting IR 1 | §21.7 Emitting IR 1 |
| B28-82 | Emitting IR 2 | §21.7 Emitting IR 2 |
| B28-83 | Emitting Machine IR | §21.7 Emitting Machine IR |
| B28-84 | Disassembling | §21.7 Disassembling |
| B28-85 | Inspecting assembly | §21.7 Inspecting assembly |
| B28-86 | Inspecting optimization decisions | §21.7 Inspecting optimization decisions |
| B28-87 | Inspecting layout | §21.7 Inspecting layout |
| B28-88 | Inspecting type information | §21.7 Inspecting type information |
