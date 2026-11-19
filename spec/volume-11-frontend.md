CEP:FILE: spec/volume-11-frontend.md
CEP:WHAT: Normative specification of the Planck frontend architecture: driver, source management, lexer, parser, AST, desugaring, resolution, type checking, trait solver, constant evaluator, exhaustiveness analysis, frontend control-flow graph, ownership/effect validation, and the typed semantic handoff (design.md §18).
CEP:WHY: design.md §18 lists 171 items defining how source text becomes a validated semantic program; without fixed algorithms, recovery sets, and pass contracts the frontend has no termination, recovery, or determinism guarantees.
CEP:STATUS: complete
CEP:FAILURE: A stage without an exact interface, a parser recovery rule that can fail to consume input, or a handoff field without a defined producer makes the frontend non-terminating or non-deterministic and is a conformance failure of this volume.
CEP:ASSUMES: SPEC-INDEX.md §1 method, §3 settled decisions, §5 pinned contracts (operator table, literal math, ownership state machine, effect algebra, graph math); Volumes 02–09 for language semantics; Volume 10 attributes; Volume 12 for limits, interning, diagnostics registry, pass infrastructure.
CEP:COST: document; fixes the compiler's frontend resource behavior (limits referenced from Volume 12).
CEP:EVIDENCE: verification matrix OB-11-1..OB-11-14; coverage ledger (bullets in scope: 171, defined: 171).

---

## 11.0 Scope, pipeline, and traceability

This volume defines design.md §18 "Frontend architecture" (lines 1470–1719). Bullet census: §18.1 = 16, §18.2 = 10, §18.3 = 8, §18.4 = 10, §18.5 = 18, §18.6 = 16, §18.7 = 10, §18.8 = 13, §18.9 = 14, §18.10 = 9, §18.11 = 10, §18.12 = 14, §18.13 = 11, §18.14 = 12; total 171. Section mapping: 11.n ↔ §18.n for n = 1..14. Design deliverables map as: Frontend Architecture Specification → §11.1–§11.13; Frontend Semantic Handoff Contract → §11.14.

The frontend pipeline is fixed. Stage interfaces are exact; CEP classes follow standards.md §5:

| Stage | Component | Interface (exact) | Class |
| ----- | --------- | ----------------- | ----- |
| S1 | Source loading | `load(paths) -> Result<SourceMap, SourceError>` | CEP-1 |
| S2 | Lexer | `lex(&SourceFile, &mut Arena) -> LexOutput { tokens, errors }` | CEP-0 |
| S3 | Parser | `parse_crate(&[Token], &mut Arena) -> ParseOutput { ast: &'arena File, errors }` | CEP-0 |
| S4 | Macro expansion | `expand(&mut File, &MacroRegistry, &Limits) -> Result<ExpansionReport, ExpansionError>` (interpreter core CEP-0) | CEP-1 |
| S5 | Desugaring | `desugar(&mut File, &mut Ctx) -> Result<SpanMap, DesugarError>` | CEP-1 |
| S6 | Resolution | `resolve(&File, &mut Ctx) -> Result<ResolutionMap, ResolveError>` | CEP-1 |
| S7 | Type checking | `type_check(&ResolutionMap, &mut Ctx) -> Result<TypeCheckOutput, TypeError>` | CEP-1 |
| S8 | Trait solving | `solve(&[Goal], &ImplRegistry) -> Result<Solution, SolverError>` (inner loop CEP-0) | CEP-1 |
| S9 | Const evaluation | Volume 09 §9.1 interpreter (CEP-0) | CEP-0 |
| S10 | Exhaustiveness | `analyze_matches(&File, &TyCtx) -> Vec<MatchDiagnostic>` | CEP-1 |
| S11 | FCFG construction | `build_cfg(&TypedBody) -> Fcfg` | CEP-1 |
| S12 | Ownership/effects | `validate(&Fcfg, &TyCtx) -> Result<OwnershipFacts, BorrowError>` (transition core CEP-0) | CEP-1 |
| S13 | Handoff | `assemble(...) -> TypedProgram` | CEP-1 |

Diagnostic identifiers use block PLC-0600..PLC-0699 (registry: Volume 12 §12.5). All limits cited are rows of the Volume 12 limits table.

## 11.1 Compiler driver (design §18.1)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Command-line interface | `plkc [OPTIONS] INPUT`; the option table of this section is closed; unknown option = usage error. |
| Input file handling | Exactly one crate root file per invocation; the file must exist and be readable UTF-8 ≤ 268435456 bytes (limit L02); otherwise usage error or PLC-0620. |
| Multiple input files | Additional `.plk` files given are treated as crate roots of a multi-crate session compiled in command-line order; module files are discovered via `mod` declarations, not listed on the command line. |
| Compilation modes | `--emit={meta,dep-info,ast,expanded,ir1,ir2,mir,asm,obj,link,exe}` (closed, 11 values); multiple `--emit` flags accumulate. |
| Target selection | `--target={x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu}` (closed, 2 values, both implemented per SPEC-INDEX §3); anything else = usage error exit 2. |
| Optimization level | `-O0`..`-O3` (default `-O0`); mapping to optimization pipelines is Volume 23. |
| Debug information options | `--debug-info={none,line-tables,full}` (default `none`); output format DWARF 5 (Volume 17). |
| Output selection | `--out-dir DIR` (per-emit default names) and `-o FILE` (single emit only); both given with multiple emits = usage error. |
| Diagnostic configuration | `--error-format={human,json}` and `--color={auto,always,never}` (§11.0 registry; JSON schema: Volume 12 §12.5). |
| Feature selection | `--feature NAME` accumulates enabled cfg-features; names must be declared in the build manifest (Volume 20), else PLC-0621. |
| Build configuration | `--profile={dev,release}`; exact mapping: dev = `-O0, debug_assertions=on, debug-info=none`; release = `-O3, debug_assertions=off, debug-info=none`. Profile flags are overridden by explicit flags. |
| Compiler configuration files | `plkc.toml` in the crate root; closed keys: `[compiler] emit/target/opt/debug-info/error-format`, `[features] names`, `[target] triple`. Load order: defaults < config file < environment < command line (each later source overrides). |
| Environment variables | Closed set {PLANCK_TARGET, PLANCK_OPT_LEVEL, PLANCK_FEATURES, PLANCK_ERROR_FORMAT, PLANCK_INCREMENTAL}; other PLANCK_* variables are ignored. |
| Exit codes | Total function over outcomes: 0 = success; 1 = compilation errors; 2 = usage error; 101 = internal compiler error; 130 = cancelled. No other code is produced (OB-11-14). |
| Cancellation | SIGINT/SIGTERM cancel at the next stage boundary; no partial output file is written; exit 130. |
| Compiler invocation protocol | `plkc @argfile` reads one argument per line; `--version` prints `plkc 0.1.0 (edition P1)`; the JSON diagnostic schema (Volume 12 §12.5) is the only machine-readable protocol. |

## 11.2 Source management (design §18.2)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Source file storage | All loaded files live in a source map (arena-backed); file contents are immutable after load; content hash = FNV-1a-64 (Volume 12 §12.1). |
| Source identifiers | SourceId = u32; 0 is reserved for synthetic spans; limit L01 caps files at 65536 per crate. |
| Source locations | A location is (byte offset: u32, line: u32, column: u32), columns 1-based, computed on demand. |
| Source spans | Span = {source: SourceId, start: u32, len: u32}; max file size 4 GiB − 1 by representation, 256 MiB by limit L02. |
| Source maps | Expansion mapping: each generated token carries (invocation Span, expansion depth, macro DefId) per Volume 09 §9.4.6. |
| Included files | `include!`/`include_str!`/`include_bytes!` register the file as a SourceId with a dependency edge to the including file. |
| Generated source | Synthetic SourceIds identify expansion outputs; dumping (`-Zdump-expanded`) writes their text; they are never written back to disk as inputs. |
| Macro expansion locations | Depth ≤ 128 (limit L05); the location chain is the expansion traceback (Volume 12 §12.5). |
| File caching | Cache key = (path, content hash, mtime); mtime mismatch without hash mismatch reuses the cache. |
| Incremental invalidation | A changed file marks its SourceId dirty; dirty propagation follows the Volume 12 §12.10 contract (G23). |

## 11.3 Lexer (design §18.3)

Pipeline stages (exact, in order; each stage is a total function over its input domain):

| Stage | Interface | Definition |
| ----- | --------- | ---------- |
| L1 UTF-8 validation | `validate(bytes) -> Result<&str, LexError>` | Invalid byte sequence = PLC-0600 at that offset; validation is complete before scanning. |
| L2 Token scan | `scan(&str, &mut Arena) -> (TokenVec, Vec<LexError>)` | Character classification via `planck-charclass` (SPEC-INDEX §5.10); identifiers are ASCII in edition P1 (non-ASCII identifier = PLC-0607). |
| L3 Literal decoding | `decode(token) -> Result<TokenValue, LexError>` | Integer/float/char/string values per SPEC-INDEX §5.6 via `planck-token-math`; overflow at literal level is PLC-0604/PLC-0605. |
| L4 Keyword/ident interning | `intern(name) -> SymId` | Keywords recognized by binary search over the sorted keyword array of Volume 02 (the set itself is Volume 02's definition); identifiers interned via the Volume 12 interner. |

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Token generation | Token = {kind, span, value}; TokenKind closed: {Ident, Keyword, IntLit, FloatLit, CharLit, StrLit, ByteStringLit, Lifetime, Punct, OpenDelim, CloseDelim, Eof, Error} (13 kinds). |
| Token positions | Every token carries its Span (§11.2); positions are exact and 0-byte tokens are banned except Eof. |
| Literal decoding | Per stage L3; decoding is total over well-formed tokens and returns a typed error otherwise. |
| Keyword recognition | Binary search, O(log k) comparisons, k = keyword count of Volume 02; no hashing (determinism). |
| Error tokens | Unterminated string → Error token to end of line; unterminated block comment → Error token to EOF; invalid UTF-8 → per-byte Error token; lexing continues after every error. |
| Lexical diagnostics | Block PLC-0600..0609, assigned: 0600 invalid UTF-8; 0601 unterminated string; 0602 unterminated comment; 0603 invalid char literal; 0604 invalid integer literal; 0605 invalid float literal; 0606 unknown escape; 0607 non-ASCII identifier; 0608 unexpected character; 0609 literal too large for type. |
| Incremental lexing | After an edit, only affected line ranges are re-lexed; the resulting token stream must be identical to a full lex (OB-11-1 covers both paths). |
| Malformed input recovery | Error tokens are the recovery mechanism; the lexer always terminates by consuming the whole input (measure: input length strictly decreases per token emitted; OB-11-2). |

## 11.4 Parser (design §18.4)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Grammar implementation | Recursive descent, one function per Volume 02 production, with **precedence climbing** for binary operators driven by the pinned operator table (SPEC-INDEX §5.5): precedence levels and associativity are read from the table, never duplicated in the parser. |
| AST construction | The parser constructs arena nodes (§11.5) in source order; NodeIds are assigned in construction order (deterministic). |
| Operator precedence | Exactly the table columns of SPEC-INDEX §5.5; a prefix/postfix layer runs before precedence climbing. |
| Generic parsing | At `path<T, ...>` the parser attempts to parse a generic argument list with bounded backtracking (max 8 tokens of lookahead); failure falls back to less-than parsing; the disambiguation rule is fixed, not heuristic. |
| Error recovery | **Panic mode**: on a syntax error the parser emits PLC-0610, then discards tokens until the first synchronization token of the innermost applicable set (below), then resumes. Recovery always consumes at least one token; if the discarded region contained the opener of a closing delimiter, the closer is consumed with the region. |
| Partial AST construction | Recovery splices an ErrorNode (§11.5) at the failure point; parsing continues to EOF; a parse with errors never reaches expansion (pipeline gate). |
| Parser diagnostics | Block PLC-0610..0619: 0610 unexpected token (with expected set); 0611 unbalanced delimiter; 0612 malformed item; 0613 malformed pattern; 0614 malformed type; 0615 malformed expression; 0616 attribute not allowed here; 0617 visibility not allowed here; 0618 where-clause misplaced; 0619 trailing tokens after item. |
| Incremental parsing | Function-granular reuse: a function whose byte range and hash are unchanged and whose enclosing context is unchanged reuses its AST subtree (G23; correctness = OB-11-3 corpus). |
| Syntax tree ownership | The AST is owned by the crate arena (Volume 12 §12.4); no reference escapes the arena lifetime; arena is dropped at crate-pass end. |
| AST validation | After parsing, a validator checks the closed invariant list of §11.5 (well-formedness); a failed invariant is an ICE (exit 101), since the parser must never produce a malformed tree silently. |

Synchronization token sets (exact):

| Level | Sync set |
| ----- | -------- |
| Item | {Eof, `}`, `fn`, `struct`, `enum`, `union`, `impl`, `trait`, `type`, `const`, `static`, `use`, `mod`, `macro`} |
| Statement | Item set ∪ {`;`, `let`, `if`, `match`, `while`, `loop`, `for`, `return`, `break`, `continue`} |
| Expression | {Eof, `}`, `)`, `]`, `;`, `,`} |

## 11.5 AST (design §18.5)

**Node inventory (closed, 82 kinds in 11 groups)** — the schemas of these groups are the definitions of the design bullets:

| Group | Kinds | Count |
| ----- | ----- | ----- |
| Modules | File, Module | 2 |
| Declarations | FnDecl, StructDecl, EnumDecl, UnionDecl, TraitDecl, ImplDecl, TypeAliasDecl, ConstDecl, StaticDecl, UseDecl, ExternBlockDecl, ExternFnDecl, MacroDef, MacroInvoke | 14 |
| Types | NamedType, RefType, PtrType, ArrayType, SliceType, TupleType, FnType, PathType, GenericType, InferType, NeverType | 11 |
| Expressions | LitExpr, PathExpr, FieldExpr, IndexExpr, CallExpr, MethodCallExpr, UnaryExpr, BinaryExpr, CastExpr, TupleExpr, ArrayExpr, RepeatArrayExpr, StructLitExpr, IfExpr, MatchExpr, LoopExpr, WhileExpr, ForExpr, ReturnExpr, BreakExpr, ContinueExpr, BlockExpr, AssignExpr, CompoundAssignExpr, ClosureExpr, AsyncBlockExpr, AwaitExpr, TryExpr, RangeExpr, ParenExpr, MacroCallExpr | 31 |
| Statements | LetStmt, ExprStmt, ItemStmt, SemiStmt | 4 |
| Patterns | WildPat, LitPat, RangePat, TuplePat, StructPat, EnumPat, RefPat, SlicePat, OrPat, BindingPat, PathPat, RestPat | 12 |
| Attributes | Attribute | 1 |
| Generic parameters | TypeParam, LifetimeParam, ConstParam, WhereClause | 4 |
| Effect clauses | EffectClause | 1 |
| Function bodies | FunctionBody | 1 |
| Error nodes | ErrorNode | 1 |

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Node IDs | NodeId = u32, dense per crate, assigned in construction order (deterministic, G14); space 2^32−1; the practical bound is the macro token budget (limit L06). |
| Source locations | Every node stores exactly one Span; synthesized nodes carry the synthetic spans of §11.2. |
| Parent-child relationships | Nodes store children only; no parent pointers; a parent map is built on demand for diagnostics (CEP-1, not stored in the arena). |
| Node ownership | The crate bump arena owns all nodes; node data has no interior heap pointers except interned SymIds. |
| AST mutation rules | After the expansion fixpoint the AST is immutable except: (a) ErrorNode replacement during recovery (before fixpoint), (b) append-only insertion during expansion. Any other mutation is an ICE. |
| AST traversal interfaces | One visitor interface, pre-order, explicit recursion with depth limit 256 (limit L03); no walk-generation in P1. |
| Serialization, if required | Not required in edition P1 and banned: the AST is never serialized; metadata (Volume 17) carries semantic data only. |

## 11.6 Desugaring (design §18.6)

**Common contract** (applies to every desugaring; these define the design's per-desugaring bullets): the table below states the **source construct** and **lowered form**; **semantic preservation**: the desugared program and the sugar have identical semantics under Volumes 04–07 (OB-11-6, G4); **ownership preservation**: no place is moved, borrowed, or dropped differently than written (the compound-assignment temp and closure captures are the only introduced places, and they are listed); **effect preservation**: the lowered effect row equals the source row; **diagnostic source mapping**: every introduced node carries a synthetic span mapping to the source span, and desugaring never erases ownership or diagnostic information (design's binding sentence).

| Source construct | Lowered form (exact) |
| ---------------- | -------------------- |
| Compound assignments | `p op= e` → evaluate `e` into temp `t`, evaluate place `p` once, then `p = p op t`; overflow class per operator table; `t` is a hygienic local. |
| For loops | `for pat in e { b }` → `{ let __it = IntoIterator::into_iter(e); loop { match __it.next() { Some(pat) => b, None => break } } }` (Iterator/Option per Volume 19); `__it` is a hygienic local; `pat` binds by the normal pattern rules. |
| Method calls | `r.m(a)` → candidate resolution (§11.7) then a direct call `Type::m(r, a)` or trait call with autoref tried in the order: by-value, `&r`, `&mut r`; the receiver chain is fixed. |
| Async functions | `async fn f` → a state-machine struct whose fields are the locals live across each `await`, plus a poll method (full encoding: Volume 08); effect row gains the `async` atom. |
| Closures | `|args| body` → an anonymous struct capturing the environment (by-ref or by-copy per capture analysis) implementing the Volume 19 call traits; capture mode per variable is fixed by the smallest capability that type-checks. |
| Error propagation | `e?` → `match e { Ok(v) => v, Err(err) => return Err(From::from(err)) }` for Result, the analogous Option form, in function bodies returning the same shape; otherwise PLC-0630. |
| Pattern matching | `match` → exhaustiveness analysis (§11.11) then a decision tree of `SwitchInt`/`Branch` FCFG blocks; lowering happens after checking. |
| Range expressions | `a..b`, `a..=b`, `a..`, `..b`, `..` → construction of the corresponding Volume 19 range structs. |
| Derive-generated implementations | `#[derive(T)]` → the built-in derive expansion of Volume 09 §9.4.4, injected as impl items before resolution. |
| Other syntactic sugar | None exists in edition P1 beyond the nine constructs above; the list is closed and adding sugar is an edition change. |

## 11.7 Name resolution (design §18.7)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Scope creation | Scopes: crate, module, function, block, closure, match arm. Creation is syntax-directed; the scope tree is built in source order. |
| Declaration collection | Phase 1 collects every declaration into its scope in source order (two-phase resolution: collection before binding). |
| Import resolution | `use` = path alias into the importing scope; `use p::*` = glob; explicit declarations shadow globs; two globs providing the same name conflict only at the use site (lazy ambiguity). |
| Visibility validation | Visibility rules are Volume 03's; a reference to a non-visible item is PLC-0640 with both spans. |
| Generic resolution | Type/lifetime/const parameters bind in their item's scope; `impl` blocks add their generics to the impl scope. |
| Method lookup | Candidate set = inherent methods of the receiver's autoref chain (order: by-value, `&`, `&mut`) ∪ trait methods where the trait is in scope or a bound; the first non-empty candidate tier wins; ties inside a tier are ambiguity errors. |
| Trait lookup | Trait names resolve in the type namespace of scope; a used-but-unimported trait is reported as a suggestion, not an error, until constraint solving (§11.8) confirms the need. |
| Associated item lookup | `Type::Item` / `Trait::Item` resolve through the impl registry (Volume 12 §12.2); unresolvable associated items become solver goals. |
| Module dependency resolution | `mod` declarations load files on demand; a cycle in the module tree is PLC-0641; file-not-found is PLC-0642. |
| Ambiguity reporting | Every ambiguous reference is PLC-0643 listing all candidates with spans; ambiguity is always an error, never silent resolution. |

## 11.8 Type checking (design §18.8)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Expression type checking | Bidirectional: items and declarations are checked against declared types; local expressions infer. Inference variables are u32; unification is structural with occurs check. |
| Declaration type checking | Every item is checked exactly once per crate (after expansion); duplicate item names are PLC-0650. |
| Function checking | Body checked against its signature; return-type verification uses the FCFG (§11.12): every path returns a value of the return type or diverges. |
| Generic checking | All where-clause bounds are collected as solver goals (§11.9); a function is checked once generically, not per instantiation. |
| Trait constraint collection | Every obligation site emits a Goal (§11.9); goals carry their spans for diagnostics. |
| Type inference | Termination measure: ≤ 2^20 inference steps per function body (limit L29 of Volume 12); exceeding is PLC-0651 (OB-11-8, G11). |
| Coercion checking | Closed coercion set (7): `&mut T` → `&T` reborrow; fn item → fn pointer; uncapturing closure → fn pointer; sized weakening `T` → `T?` positions; `[T; n]` → `[T]`; `T` → `dyn Trait` (object-safe T); `!` → any type. Nothing else coerces implicitly. |
| Pattern type checking | Patterns check against the scrutinee type; binding modes (`ref`, move) are fixed per Volume 06; a pattern that cannot match is PLC-0652. |
| Return-type verification | Via FCFG reachability plus the `!` coercion; a missing return on a reachable path is PLC-0653. |
| Closure type inference | Capture inference is a fixed-point over the closure body; termination measure = capture set grows monotonically, bounded by the variable count (OB-11-8). |
| Trait object validation | `dyn Trait` requires: no generic methods, no `Self` in method signatures except the receiver, receiver is `self`/`&self`/`&mut self`/`Box<Self>`-free; violation = PLC-0654. |
| Type normalization | Associated types are normalized through the solver (§11.9); canonicalization is Volume 12 §12.1. |
| Type error recovery | On a type error the checker emits the error and continues with the recovery type (TypeId 0, Volume 12 §12.1); total error cap = limit L21 (256 per crate); after the cap the compilation stops with exit 1. |

## 11.9 Trait solver (design §18.9)

This is a distinct subsystem with its own API (`solve`, §11.0); it must not be embedded inside type checking (design's binding sentence, normative).

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Goal representation | Goal = (TraitRef, generic arguments, environment of in-scope bounds); canonicalized per Volume 12 §12.1 before solving. |
| Constraint generation | Emitted by type checking (§11.8) at obligation sites; each Goal carries its source Span. |
| Implementation candidate lookup | Candidates = impls from the registry matching the trait, coherence-filtered, plus parameter bounds from the environment. |
| Candidate selection | Winnowing: discard candidates whose arguments do not unify; exactly one survivor = selected; more than one = ambiguous; zero = PLC-0655 "trait not implemented" (using `#[diagnostic(on_unimplemented)]` text when present, §10.3). |
| Generic substitution | Substitution applies with depth ≤ 32 (pinned monomorphization depth, SPEC-INDEX §3); deeper is PLC-0656. |
| Associated type normalization | Projection goals: `T::Assoc` normalizes via the selected impl's associated-type binding; unnormalized projections at monomorphization = PLC-0657. |
| Recursive goals | Recursion is allowed; solver depth ≤ 128 (limit L16); exceeding is PLC-0658. |
| Cyclic goals | A cycle (a goal recursing to itself with identical canonical form) is a definite error PLC-0659; there is no coinduction in edition P1. |
| Ambiguous goals | Ambiguity is deferred to a queue resolved after more constraints arrive; a goal still ambiguous at monomorphization is PLC-0660. |
| Overflow handling | Solver steps per goal ≤ 10^6 (limit L15); exceeding is PLC-0661 "trait solver overflow" — a defined compilation failure, never a hang (OB-11-9, G11). |
| Coherence validation | Orphan rule and overlap rejection exactly per SPEC-INDEX §3 (trait coherence); checked at impl registration, before any solving. |
| Specialization selection | No specialization exists in edition P1 (monomorphization only, SPEC-INDEX §3); any specialization attribute is rejected as unknown (§10.4). |
| Caching | Cache key = canonical Goal; value = selected candidate or Ambiguous; content-addressed (FNV-1a-64, Volume 12 §12.1). |
| Incremental invalidation | The impl-set fingerprint (sorted list of impl hashes) is part of every cache key; change ⇒ re-solve (Volume 12 §12.10, G23). |

## 11.10 Constant evaluator (design §18.10)

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Constant expression evaluation | The Volume 09 §9.1 interpreter is the evaluator; the frontend triggers it, it does not reimplement it. |
| Compile-time function execution | `const fn` calls execute as const bytecode (Volume 09 §9.6.6) or in-crate interpreted MIR-equivalent; semantics identical to runtime (OB-11-6 covers one representative equivalence). |
| Generic constant evaluation | Evaluation runs per canonical substitution; cache keys include the substitution (Volume 09 §9.1.13). |
| Static initializer evaluation | Per Volume 09 §9.2 (StaticSafe, promotion, `#[init]` handling). |
| Type-level computation | None beyond const generics: no type-level functions or type-level arithmetic exist in edition P1; attempts are PLC-0662. |
| Resource limits | Volume 09 §9.1.11 exactly (steps 10^7, depth 512, memory 2^20 words); wall clock via Volume 12 limit L18. |
| Evaluation caching | Volume 09 §9.1.13 keys; the frontend owns the cache, the interpreter is stateless across requests. |
| Error reporting | PLC-04xx diagnostics with the invocation span added by the frontend; evaluation stacks are preserved (Volume 09 §9.1.12). |
| Cross-target evaluation | Target-parameterized per Volume 09 §9.6.4; the frontend passes the target triple of §11.1. |

## 11.11 Exhaustiveness analysis (design §18.11)

Algorithm: the usefulness judgment `p useful against matrix M` (Maranget-style) over constructor grids, extended with integer intervals. Usefulness of arm i = does pattern `p_i` match a value unmatched by arms < i. Exhaustiveness of a match = the wildcard pattern `_` is not useful after all arms. Termination measure: the total pattern-matrix size (rows × columns) strictly decreases per recursive step; bounded by limits L03/L06 (OB-11-10, G11).

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Match exhaustiveness | Non-exhaustive matches are PLC-0663 with a computed minimal witness pattern (below). |
| Unreachable patterns | Arm i with `p_i` not useful is warning PLC-0664. |
| Variant analysis | Enum constructors are useful iff the constructor (or its wildcard) is unmatched; all variants required for exhaustiveness unless a wildcard or total constructor set covers them. |
| Integer range coverage | Integer ranges are tracked as interval sets; usefulness = the pattern's interval minus the union of previous intervals is non-empty; open types require a wildcard or full-range arm for exhaustiveness. |
| Guard handling | An arm with a guard contributes nothing to exhaustiveness (its usefulness is unknown); the checker reports exhaustiveness as if the arm were absent. |
| Nested patterns | Usefulness recurses on sub-patterns column-wise; matrix splitting follows the constructor of the leftmost useful column. |
| Slice patterns | Fixed-length `[a, b]` match length exactly; `[a, ..rest, b]` splits on length ≥ prefix + suffix; length-based case analysis is exact. |
| Or patterns | `p | q` expands to two rows before analysis (no special case). |
| Generic enum patterns | The constructor list comes from the enum definition under the scrutinee's substitution (§11.9 normalization); an unknown constructor after substitution is PLC-0665. |
| Diagnostic suggestions | The witness = the unmatched value constructed by the usefulness recursion, printed in pattern syntax; the witness is minimal by construction order, not by size. |

## 11.12 Frontend control-flow analysis (design §18.12)

The **FCFG** (frontend control-flow graph) is a temporary analysis structure per function; it must not replace the three optimizing IRs (design's binding sentence, normative). Construction: from the desugared, type-checked body.

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Basic blocks | A block is a maximal straight-line sequence of lowered statements; blocks are numbered densely per function. |
| Edges | Edge = (from_block, to_block, kind); kinds: {normal, exceptional}; stored as CSR adjacency (planck-graph-math layout, SPEC-INDEX §5.7). |
| Branch conditions | Branch edges carry their condition expression node; SwitchInt edges carry (discriminant, value → target) tables. |
| Loop structure | Loops = maximal strongly-connected subgraphs with a back edge; the header = the dominator of the SCC; identified via dominator information. |
| Returns | `return e` lowers to a Return terminator (below); the return value is stored before the terminator. |
| Breaks | `break` lowers to Goto the innermost loop's exit block; `break` outside a loop is a type-check rejection (§11.8). |
| Continues | `continue` lowers to Goto the innermost loop's header. |
| Panic exits | Every operation whose effect row contains the `panic` atom gets an exceptional edge to the function's shared unwind root block. |
| Unreachable blocks | Blocks unreachable from entry are detected by graph reachability and deleted before borrow checking; no diagnostic is required for them. |
| Exceptional edges | From every potentially-panicking operation and every call whose callee row contains `panic`; nothing else creates exceptional edges (catch boundaries are encoded as normal edges per Volume 07). |
| Await suspension points | `await` lowers to an Await terminator; the borrowed-live-across-await rule is §11.13. |
| Yield suspension points | `yield` does not exist in source in edition P1; the Yield terminator is reserved for the async state-machine encoding of §11.6. |
| Terminators | Closed set (9): {Goto, Branch, SwitchInt, Return, Unreachable, Panic, Await, Yield, Call {func, args, ret, unwind}}. Every block ends in exactly one terminator (OB-11-11). |
| Dominance information | Computed by the iterative dominator algorithm of `planck-graph-math` (SPEC-INDEX §5.7): deterministic, ≤ n passes with early exit, tie-breaks by ascending block ID (G14, G11). |

## 11.13 Ownership and effect validation (design §18.13)

All analyses run over the FCFG; the place state machine is exactly the pinned table of SPEC-INDEX §5.4 (6 states × 7 actions, 42 cells; OB-11-12 re-verifies totality).

| Bullet | Definition (normative) |
| ------ | ---------------------- |
| Initialization dataflow | Definite-assignment analysis: a place must be in state Owned/Uninitialized per the table's `Write` row before any `Read`; use of an Uninitialized place is PLC-0670. |
| Move analysis | `MoveOut` transitions apply exactly per the pinned table; use of a MovedFrom place is PLC-0671. |
| Borrow checking | `BorrowShared`/`BorrowMut`/`EndBorrow` transitions apply per the pinned table; every loan has a region and a span. |
| Lifetime inference | Regions are inferred from the loan graph: constraints are outlives relations; solving is round-based propagation with measure = remaining constraint count strictly decreasing; regions per function ≤ 65536 (limit L17); exceeding is PLC-0672 (OB-11-8, G11). |
| Loan conflicts | `conflict(a, b)` is derived exactly from the pinned table's rejection entries (SPEC-INDEX §5.4); a conflicting action is PLC-0673. |
| Drop analysis | Drop flags per block are computed from initialization states; a MovedFrom place is not dropped; drop order is reverse declaration order per block; destructor information feeds the handoff (§11.14). |
| Effect inference | Rows are inferred bottom-up with exact row unification of `planck-effect-math` (SPEC-INDEX §5.3). |
| Effect contract validation | Declared row must satisfy r_inferred ≤ r_declared (row subtyping, SPEC-INDEX §5.3); violation is PLC-0674 with both rows printed canonically. |
| Unsafe authority validation | Every operation carrying the `unsafe` atom (unsafe fns, intrinsic calls, extern calls, union field reads) must be lexically inside `unsafe` authority (unsafe block or unsafe fn); violation is PLC-0675. |
| Async borrowing validation | A borrow may not be live across an Await suspension point in edition P1: violation is PLC-0676 (rejection; Volume 08 may relax in later editions only). |
| Send and Sync checking | Auto-trait Send/Sync obligations are solver goals (§11.9) computed structurally per Volume 08; a non-Send value crossing an await point is PLC-0677. |

## 11.14 Typed semantic boundary (design §18.14)

The handoff is the struct `TypedProgram`; its fields are exactly the design's list, in order. The handoff is the **only** input to IR 1 construction (Volume 13); IR 1 must not re-run semantic checks — it consumes facts, it does not re-derive them (normative).

```
struct TypedProgram {
    resolved_names:      BidiMap<NodeId, DefId>,        // resolved names
    semantic_types:      Map<NodeId, TypeId>,            // canonical types (Volume 12 §12.1)
    generic_constraints: Vec<SolvedObligation>,          // obligations with substitutions applied
    ownership_facts:     Map<NodeId, PlaceState>,        // per-place states + loan set
    lifetime_facts:      Map<LoanId, RegionId>,          // region assignments
    initialization_facts: Map<BlockId, InitState>,       // per-block init states
    effect_summaries:    Map<DefId, EffectRow>,          // canonical rows
    control_flow_facts:  Map<DefId, Fcfg>,               // block/edge tables (§11.12)
    destructor_info:     Map<BlockId, DropSequence>,     // drop flags + order per block
    unsafe_requirements: Vec<NodeId>,                    // nodes requiring unsafe authority
    abi_requirements:    Map<DefId, AbiSpec>,             // convention + repr/layout per type
    source_mappings:     BidiMap<NodeId, Span>,          // spans incl. expansion chains
}
```

Validity conditions (all checked before assembly): every NodeId maps in all twelve tables where applicable; every DefId has a canonical effect row and ABI spec; every loan has a region; control-flow tables satisfy §11.12's terminator invariant. Assembly failure of any validity condition is an ICE (exit 101). The handoff contract is complete when all twelve fields are present and valid (OB-11-13).

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-11-1 | The incremental lexer produces a token stream identical to the full lexer for every edit corpus member. | EXH over edit corpus (insert/delete/replace, 10^4 cases) | FV-0 | tests/vol11/lexer-incremental | G4, G23 |
| OB-11-2 | Lexer and parser terminate on every input: each recovery step consumes ≥ 1 token; input length strictly decreases per token. | Z3 (consumption model) + REVIEW of the measure | FV-0 | tools/fv/ob11-02.z3 | G11 |
| OB-11-3 | Panic-mode recovery on the malformed corpus yields a resumed parse (never a hang) and reports PLC-0610..0619 codes only. | EXH over malformed corpus | FV-0 | tests/vol11/parser-recovery | G11, G22 |
| OB-11-4 | Precedence climbing with the pinned operator table yields the same parse tree as the table's reference semantics for expressions ≤ 8 operators. | EXH over operator/operand combinations | FV-0 | tests/vol11/precedence | G3 |
| OB-11-5 | The AST inventory is exactly the 82 kinds of §11.5 and every kind has a schema. | REVIEW (inventory totality against parser productions) | FV-1 | review/vol11/ast-inventory | G15 |
| OB-11-6 | Each desugaring preserves semantics, ownership, effects, and spans on the representative corpus (compound assignment, `?`, ranges). | Z3 (bounded) + PROP paired | FV-1 | tools/fv/ob11-06.z3 | G4 |
| OB-11-7 | Every reference resolves to exactly one declaration or reports PLC-0640/0643; no silent resolution. | EXH over resolution corpus (shadowing, globs, generics) | FV-0 | tests/vol11/resolution | G3 |
| OB-11-8 | Type checking, closure capture inference, and lifetime inference terminate within their declared measures (2^20 steps; monotone capture set; decreasing constraint count). | REVIEW of measures + EXH on stress corpus | FV-1 | review/vol11/termination | G11 |
| OB-11-9 | The trait solver terminates: depth ≤ 128, steps ≤ 10^6 per goal, cycles reject (PLC-0659), overflow rejects (PLC-0661). | Z3 (bounded solver model) + REVIEW | FV-0 | tools/fv/ob11-09.z3 | G11 |
| OB-11-10 | Usefulness judgment agrees with value semantics on all patterns over 2-constructor enums and integer ranges of width ≤ 16. | EXH over bounded pattern space | FV-0 | tests/vol11/exhaustiveness | G3 |
| OB-11-11 | Every FCFG has a single entry, every block exactly one terminator from the closed set, and dominance is computable by the pinned algorithm. | EXH over generated bodies + Z3 (dominator invariant) | FV-0 | tools/fv/ob11-11.z3 | G14 |
| OB-11-12 | The ownership transition table is total: all 42 (state, action) cells are defined per SPEC-INDEX §5.4. | EXH (42 cells) | FV-0 | tests/vol11/ownership-table | G2 |
| OB-11-13 | `TypedProgram` contains all twelve fields, and every validity condition of §11.14 is checked before assembly. | REVIEW (field/validity checklist) | FV-1 | review/vol11/handoff | G15 |
| OB-11-14 | The driver maps every outcome to exactly one exit code from {0, 1, 2, 101, 130} and no other value. | EXH over outcome set | FV-0 | tests/vol11/exit-codes | G22 |

## Coverage ledger

| Bullet (design.md §18, abbreviated) | Defined in |
| ----------------------------------- | ---------- |
| Command-line interface | §11.1 |
| Input file handling | §11.1 |
| Multiple input files | §11.1 |
| Compilation modes | §11.1 |
| Target selection | §11.1 |
| Optimization level | §11.1 |
| Debug information options | §11.1 |
| Output selection | §11.1 |
| Diagnostic configuration | §11.1 |
| Feature selection | §11.1 |
| Build configuration | §11.1 |
| Compiler configuration files | §11.1 |
| Environment variables | §11.1 |
| Exit codes | §11.1 |
| Cancellation | §11.1 |
| Compiler invocation protocol | §11.1 |
| Source file storage | §11.2 |
| Source identifiers | §11.2 |
| Source locations | §11.2 |
| Source spans | §11.2 |
| Source maps | §11.2 |
| Included files | §11.2 |
| Generated source | §11.2 |
| Macro expansion locations | §11.2 |
| File caching | §11.2 |
| Incremental invalidation (source) | §11.2 |
| Token generation | §11.3 |
| Token positions | §11.3 |
| Literal decoding | §11.3 |
| Keyword recognition | §11.3 |
| Error tokens | §11.3 |
| Lexical diagnostics | §11.3 |
| Incremental lexing | §11.3 |
| Malformed input recovery | §11.3 |
| Grammar implementation | §11.4 |
| AST construction | §11.4 |
| Operator precedence | §11.4 |
| Generic parsing | §11.4 |
| Error recovery | §11.4 |
| Partial AST construction | §11.4 |
| Parser diagnostics | §11.4 |
| Incremental parsing | §11.4 |
| Syntax tree ownership | §11.4 |
| AST validation | §11.4 |
| Modules (AST) | §11.5 |
| Declarations (AST) | §11.5 |
| Types (AST) | §11.5 |
| Expressions (AST) | §11.5 |
| Statements (AST) | §11.5 |
| Patterns (AST) | §11.5 |
| Attributes (AST) | §11.5 |
| Generic parameters (AST) | §11.5 |
| Effect clauses (AST) | §11.5 |
| Function bodies (AST) | §11.5 |
| Error nodes (AST) | §11.5 |
| Node IDs | §11.5 |
| Source locations (AST) | §11.5 |
| Parent-child relationships | §11.5 |
| Node ownership | §11.5 |
| AST mutation rules | §11.5 |
| AST traversal interfaces | §11.5 |
| Serialization, if required | §11.5 (banned in P1) |
| Compound assignments | §11.6 |
| For loops | §11.6 |
| Method calls | §11.6 |
| Async functions | §11.6 |
| Closures | §11.6 |
| Error propagation | §11.6 |
| Pattern matching | §11.6 |
| Range expressions | §11.6 |
| Derive-generated implementations | §11.6 |
| Other syntactic sugar | §11.6 (closed: none) |
| Source construct (per desugaring) | §11.6 table col. 1 |
| Lowered form (per desugaring) | §11.6 table col. 2 |
| Semantic preservation rules | §11.6 common contract |
| Ownership preservation | §11.6 common contract |
| Effect preservation | §11.6 common contract |
| Diagnostic source mapping | §11.6 common contract |
| Scope creation | §11.7 |
| Declaration collection | §11.7 |
| Import resolution | §11.7 |
| Visibility validation | §11.7 |
| Generic resolution | §11.7 |
| Method lookup | §11.7 |
| Trait lookup | §11.7 |
| Associated item lookup | §11.7 |
| Module dependency resolution | §11.7 |
| Ambiguity reporting | §11.7 |
| Expression type checking | §11.8 |
| Declaration type checking | §11.8 |
| Function checking | §11.8 |
| Generic checking | §11.8 |
| Trait constraint collection | §11.8 |
| Type inference | §11.8 |
| Coercion checking | §11.8 |
| Pattern type checking | §11.8 |
| Return-type verification | §11.8 |
| Closure type inference | §11.8 |
| Trait object validation | §11.8 |
| Type normalization | §11.8 |
| Type error recovery | §11.8 |
| Goal representation | §11.9 |
| Constraint generation | §11.9 |
| Implementation candidate lookup | §11.9 |
| Candidate selection | §11.9 |
| Generic substitution | §11.9 |
| Associated type normalization | §11.9 |
| Recursive goals | §11.9 |
| Cyclic goals | §11.9 |
| Ambiguous goals | §11.9 |
| Overflow handling | §11.9 |
| Coherence validation | §11.9 |
| Specialization selection | §11.9 (none in P1) |
| Caching (solver) | §11.9 |
| Incremental invalidation (solver) | §11.9 |
| Constant expression evaluation | §11.10 |
| Compile-time function execution | §11.10 |
| Generic constant evaluation | §11.10 |
| Static initializer evaluation | §11.10 |
| Type-level computation | §11.10 |
| Resource limits | §11.10 |
| Evaluation caching | §11.10 |
| Error reporting (const) | §11.10 |
| Cross-target evaluation | §11.10 |
| Match exhaustiveness | §11.11 |
| Unreachable patterns | §11.11 |
| Variant analysis | §11.11 |
| Integer range coverage | §11.11 |
| Guard handling | §11.11 |
| Nested patterns | §11.11 |
| Slice patterns | §11.11 |
| Or patterns | §11.11 |
| Generic enum patterns | §11.11 |
| Diagnostic suggestions | §11.11 |
| Basic blocks | §11.12 |
| Edges | §11.12 |
| Branch conditions | §11.12 |
| Loop structure | §11.12 |
| Returns | §11.12 |
| Breaks | §11.12 |
| Continues | §11.12 |
| Panic exits | §11.12 |
| Unreachable blocks | §11.12 |
| Exceptional edges | §11.12 |
| Await suspension points | §11.12 |
| Yield suspension points | §11.12 |
| Terminators | §11.12 |
| Dominance information | §11.12 |
| Initialization dataflow | §11.13 |
| Move analysis | §11.13 |
| Borrow checking | §11.13 |
| Lifetime inference | §11.13 |
| Loan conflicts | §11.13 |
| Drop analysis | §11.13 |
| Effect inference | §11.13 |
| Effect contract validation | §11.13 |
| Unsafe authority validation | §11.13 |
| Async borrowing validation | §11.13 |
| Send and Sync checking | §11.13 |
| Resolved names | §11.14 |
| Fully determined semantic types | §11.14 |
| Generic constraints | §11.14 |
| Ownership facts | §11.14 |
| Lifetime facts | §11.14 |
| Initialization facts | §11.14 |
| Effect summaries | §11.14 |
| Control-flow facts | §11.14 |
| Destructor information | §11.14 |
| Unsafe requirements | §11.14 |
| ABI requirements | §11.14 |
| Source mappings | §11.14 |

bullets in scope: 171, defined: 171
