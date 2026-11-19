CEP:FILE: spec/volume-20-packages-build.md
CEP:WHAT: Normative specification of the Planck package manager, build system, incremental-compilation engine, compiler cache, and reproducibility contract: manifest and lockfile formats, the deterministic dependency-resolution algorithm, content-addressed caching, and the byte-exact reproducibility obligations of the build pipeline.
CEP:WHY: design.md §27 (lines 3140–3242) requires an explicit definition for every package-management and build-system item; the compiler's correctness claims (Volume 22) and performance budgets (Volume 23) are stated relative to the inputs and artifacts this volume defines.
CEP:STATUS: complete
CEP:FAILURE: A build whose output bytes depend on anything outside the declared input set (§20.4) is a Severity 0 defect; a resolver that is nondeterministic, a lockfile that is not canonical-JSON stable, or a cache key that does not cover all inputs each violate G14 and fail the verification matrix.
CEP:ASSUMES: SPEC-INDEX.md §1 (definition method), §3 (settled decisions: initial targets, verification target, G14), §5.7 (planck-graph-math contract); standards.md (CEP&CC 0.4) classes and §13 measurement law; verification standards.md G7, G13, G14; Volumes 12 (limits), 16 (targets), 17 (object/archive formats), 18 (runtime entry contract).
CEP:COST: Tooling (CEP-2 for resolver, lockfile writer, cache store, manifest writer); build orchestration is CEP-1; no runtime cost for compiled programs.
CEP:EVIDENCE: Verification matrix (OB-20-1..OB-20-20) and coverage ledger (bullets in scope: 77, defined: 77) in this file.

---

## 20.0 Scope and traceability

This volume defines design.md **§27 Package management** (lines 3140–3242) of the Planck compiler, plus the **Deliverables** line at line 3239. Bullet inventory: §27.1 = 15, §27.2 = 9, §27.3 = 18, §27.4 = 10, §27.5 = 13, §27.6 = 12. Bullets in scope: 77, defined: 77 (ledger at end of file). The §27.5 prose directive "This needs particularly careful design" is honored by making §20.5 a total query-invalidation specification: every invalidation rule below is exact and testable (OB-20-13).

Normative verbs follow standards.md §3. Every constant below has an exact value. Error enums are complete: every variant named here is part of the contract; an implementation must not produce any other error at these boundaries.

### 20.0.1 Data encodings (normative for the whole volume)

| Encoding | Definition |
| --- | --- |
| Canonical JSON | RFC 8785 (JSON Canonicalization Scheme): UTF-8; object keys sorted by UTF-16 code-unit order; numbers serialized per ECMAScript `Number::toString`; no insignificant whitespace; literals `true`, `false`, `null`. |
| BLAKE3-256 | BLAKE3, 32-byte output, no key, no context string. Rendered as 64 lowercase hex characters. |
| Length-prefix | `lenp(x)` = the byte length of `x` as an unsigned 64-bit little-endian integer, followed by the bytes of `x`. |
| Digest chain | `digest(list)` = BLAKE3-256 over `concat(lenp(item_1), …, lenp(item_n))` with items in the order stated. |
| Integer types | Machine widths per standards.md §54.4 fidelity law: counts are `u32` unless stated `u64`; no arbitrary-precision arithmetic in build tooling. |
| Limits | MAX_MANIFEST_BYTES = 1 048 576; MAX_RESOLVE_PACKAGES = 4096; MAX_RESOLVE_DEPTH = 512; MAX_BACKTRACKS = 1024; MAX_VERSIONS_PER_PACKAGE = 4096; MAX_BUILD_UNITS = 4096; MAX_FEATURES = 128; MAX_DEPS_PER_PACKAGE = 512. Exceeding any limit is a defined compilation failure (G13), never a crash. |

## 20.1 Package model (defines design.md 27.1, bullets B27-1..B27-15)

**Package identity** (B27-1). A package is identified by the pair (registry URL, name), where name matches the **package-name grammar**: `[a-z][a-z0-9_]{0,63}` (ASCII, 1–64 bytes, lowercase start). The string form of identity is `<registry-host>/<name>`. A path or git source carries identity `(path-or-url, name)` with the same name grammar. Two packages with equal names but different sources are distinct packages; a single resolution must not mix sources for one name (§20.1 Dependency resolution, error `ResolutionError::ConflictingSources`).

**Package versions** (B27-2). A version is a SemVer 2.0.0 subset: `MAJOR.MINOR.PATCH` where each component is a decimal integer in `0..=4294967295` with no leading zeros; an optional pre-release suffix `-<prerelease>` with identifiers `[0-9A-Za-z-]{1,32}`; build metadata (`+…`) is permitted in manifest text, ignored in all comparisons, and must not appear in lockfiles. Version strings are at most 32 bytes. **Precedence** is SemVer 2.0.0 rule 11 (major, minor, patch numeric; pre-release lower than the release; pre-release identifiers compared numerically if numeric, ASCII-lexically otherwise; shorter pre-release set loses). Order is total; equality means byte-equal strings.

**Package manifests** (B27-3). The manifest file is `planck.json` at the package root, encoded as strict UTF-8 JSON (no comments, no trailing commas, no duplicate keys, no NaN/Infinity), size ≤ MAX_MANIFEST_BYTES. Exact schema (additional fields are rejected with `ManifestError::UnknownField`):

| Field | Type | Required | Constraint |
| --- | --- | --- | --- |
| `package.name` | string | yes | package-name grammar |
| `package.version` | string | yes | version grammar |
| `package.edition` | string | yes | exactly `"P1"` |
| `package.authors` | array\<string\> | no | ≤ 64 entries, each ≤ 128 bytes |
| `package.license` | string | no | SPDX expression, ≤ 128 bytes |
| `package.description` | string | no | ≤ 512 bytes |
| `dependencies` | object\<name, dep\> | no | ≤ MAX_DEPS_PER_PACKAGE |
| `dev-dependencies` | object\<name, dep\> | no | same bound |
| `build-dependencies` | object\<name, dep\> | no | same bound |
| `target."spec".dependencies` | object\<name, dep\> | no | `spec` is a cfg predicate (§20.3) |
| `features` | object\<name, array\<string\>\> | no | ≤ MAX_FEATURES; names match feature grammar `[a-z][a-z0-9_-]{0,63}` |
| `patch` | object\<name, dep\> | no | replacement sources (§20.2) |
| `planck` | object | no | compiler settings: `panic` ∈ {`"unwind"`, `"abort"`}, `overflow_checks` ∈ bool |
| `profile.<name>` | object | no | build profiles (§20.3) |

A dependency value `dep` has exactly the fields: `version` (requirement string, required unless `path`/`git` is set), optional `registry` (URL), `git` (URL), `rev` (40 lowercase hex), `path` (relative path), `optional` (bool, default false), `default_features` (bool, default true), `features` (array of feature names).

**Dependencies** (B27-4). Dependency kinds, closed: `normal` (dependencies table), `dev` (dev-dependencies; used only by test/bench/example targets and never propagated to dependents), `build` (build-dependencies; compiled for the host and executed as build scripts, §20.3), `target` (target-scoped dependencies under a cfg predicate), `patch` (replacement). A dependency graph edge exists from package P to package D iff D is a normal or target-scoped dependency of P active for the current feature and cfg evaluation, or D is a build dependency of P (build edges terminate at build-script units and do not propagate).

**Dependency constraints** (B27-5). A requirement is a string with the grammar:

```
req        := "*" | comparator ( "," comparator )*
comparator := op version_partial
op         := "=" | ">=" | "<=" | ">" | "<" | "^" | "~"
version_partial := MAJOR [ "." MINOR [ "." PATCH [ "-" prerelease ] ] ]
```

Default operator for a bare version is `^`. Intervals: `^1.2.3` = [1.2.3, 2.0.0); `^0.2.3` = [0.2.3, 0.3.0); `^0.0.3` = [0.0.3, 0.0.4); `^0.0` = [0.0.0, 0.1.0); `^0` = [0.0.0, 1.0.0); `~1.2.3` = [1.2.3, 1.3.0); `~1.2` = [1.2.0, 1.3.0); `~1` = [1.0.0, 2.0.0); `=` is the closed point; relational operators give half-open intervals at the stated bound. A comma chain is the intersection of its intervals; the empty interval is unsatisfiable. `*` = [0.0.0, 4294967295.4294967295.4294967295]. A requirement that names a pre-release admits pre-release versions of that major.minor.patch only. Malformed requirements are rejected with `ResolutionError::MalformedRequirement`.

**Feature selection** (B27-6). Features are additive and per-package. The selected feature set of package P = union over all dependents of P in the resolved graph of (explicitly requested features of P) ∪ (the `default` feature set of P, unless every dependent that names P set `default_features = false`) ∪ (the names of optional dependencies of P enabled by any selected feature). Feature entries are feature names or `"dep:<name>"`, which enables optional dependency `<name>`. Each optional dependency implicitly defines a feature with its own name. The feature list is stored sorted ascending and deduplicated; this canonical list is the `feature_list_canonical` used in every cache key (§20.3, §20.6).

**Optional dependencies** (B27-7). An optional dependency is absent from the graph unless its enabling feature is selected. An optional dependency that is not enabled contributes no compilation units and no build-graph edges. `ResolutionError::FeatureCycle` is produced if feature selection reaches a fixed-point cycle through `dep:` entries (detection: iteration over the ≤ MAX_FEATURES feature set with a worklist; termination measure = feature count).

**Platform-specific dependencies** (B27-8). Dependencies under `target."spec"` are active iff the cfg predicate `spec` evaluates to true for the build target's fact table (§20.3 Conditional compilation). Evaluation is a pure function of (predicate, fact table). Platform-specific requirements join the requirement set of §20.1 Dependency resolution identically to normal ones.

**Development dependencies** (B27-9). Dev-dependencies are resolved and locked like normal dependencies, but contribute edges only to test, bench, and example units of the declaring package, and are excluded from the interface digest of the declaring package (§20.5 Cross-module invalidation).

**Build dependencies** (B27-10). Build-dependencies compile for the host triple with the dev profile, regardless of the build target; they are available to the build script as a linkable library. Build dependencies must not appear in the target package's runtime link line; violation = `BuildError::BuildDepLeaked`.

**Dependency resolution** (B27-11). Settled decision (per SPEC-INDEX §3 and the task contract): resolution is SAT-free, deterministic, minimal-version selection with ascending candidate order. Input domain: (root manifest set, registry index snapshot or source trees, policy file, patch set, target facts, feature roots). Output: the resolution graph, a DAG with ≤ MAX_RESOLVE_PACKAGES nodes. Algorithm `MinimalResolve`, exactly:

1. Initialize requirement map REQ: name → set of (requirement, provenance), from the root package's dependency tables (normal, target-active, dev, build, feature-derived).
2. Loop. Each iteration performs exactly one event and increments the event counter E (termination measure: E, bounded by MAX_RESOLVE_PACKAGES + MAX_BACKTRACKS = 5120):
   a. Resolve event: let n = lexicographically smallest unresolved name in REQ (exact tie-break). Compute the conjunction S(n) of all requirements in REQ[n]. Candidate set = versions of n in its unique source, sorted ascending by precedence; pre-release versions admitted only if S(n) names a pre-release; yanked versions excluded unless already locked. Choose the minimal candidate v with v ∈ S(n) and depth(n) ≤ MAX_RESOLVE_DEPTH. If none exists, produce a failure event (step 3). Otherwise set SOL[n] = v and add the requirements of n@v's active dependencies to REQ (bounded by MAX_RESOLVE_PACKAGES; exceeding = `ResolutionError::TooManyPackages`).
   b. Invalidation event: for each already-resolved name m whose SOL[m] violates a requirement added in step 2a, reset SOL[m] to unresolved, push a decision frame onto the stack, and increment the backtrack counter. If the counter exceeds MAX_BACKTRACKS, fail with `ResolutionError::BacktrackLimit`.
3. Failure and backtracking: pop the most recent decision frame (LIFO), retry the next candidate of that frame in ascending order; exhausted candidates pop further frames. Fully exhausted stack = `ResolutionError::Unsatisfiable` carrying the full requirement set of the blocking name (G17 counterexample data).
4. Source tie-breaks, exact: a name must be satisfied from one source kind; requirements citing different source kinds for one name fail with `ResolutionError::ConflictingSources`. Within one kind: path sources resolve by canonical path; git sources by the pinned rev (byte-equal required); registry sources by registry order = manifest declaration order, first match wins. Two registries serving the same name+version is a first-match situation, not an error.

Worst-case complexity: each resolve event scans ≤ MAX_VERSIONS_PER_PACKAGE candidates with an interval check of cost O(|REQ[n]|); total O(5120 × 4096 × |REQ|max) interval checks, each O(1) after parsing. The registry index snapshot digest (§20.2) is part of the resolution input; identical inputs produce byte-identical lockfiles (OB-20-2).

**Lockfiles** (B27-12). The lockfile is `planck.lock`, written as canonical JSON of the lock document, LF-terminated. Document fields: `lock_version` = 1; `root` = root package name; `packages` = array of package records sorted by (name asc, version asc, source precedence asc) where source precedence = path < git < registry. Record fields: `name`, `version`, `source` ∈ {`path`, `git`, `registry`}, `source_url` (registry/git only), `rev` (git only), `path` (path only, package-root-relative canonical), `checksum` (BLAKE3-256 of the archive bytes, §20.2), `content_hash`, `dependencies` (sorted by name: name, resolved version, kind, optional, features, target predicate), `features` (canonical list). `content_hash` = base16(BLAKE3-256(canonical JSON of the record with the `content_hash` field removed)). The loader recomputes every `content_hash`; any mismatch = `LockfileError::Corrupt` (build fails). Lockfiles must not contain timestamps, absolute paths, or host names. `--locked` mode: resolution must not consult the network; a cache miss is a hard error `RetrievalError::LockedNoFetch`.

**Registry support** (B27-13). A registry is an HTTPS index with base URL `https://<host>`. Index path for package n: `<base>/index/<n[0:2]>/<n[2:4]>/<n>` (path segments; names shorter than 4 bytes use what exists). The index file for n is one JSON object per line (JSON Lines), each line: `{"vers": string, "deps": [{"name","req","features","optional","default_features","target"}], "cksum": hex, "yanked": bool, "features": {...}}`, lines sorted ascending by version. The registry snapshot digest = `tree_hash` = BLAKE3-256 over digest(list) of the index-file bytes sorted by path (§20.0.1). Registries are configured in `registries.toml` (grammar: array of {`url`, `root_key` (optional Ed25519 public key, 32 bytes base16), `require_signed` (bool)}). Download endpoint: `<base>/crates/<name>/<version>.tar` serving the ustar archive (§20.2 Package checksums).

**Git dependencies** (B27-14). `git` must be an `https://` URL; `rev` must be a full 40-character lowercase hexadecimal SHA-1 commit id (short ids and branch/tag names are banned). Retrieval = clone at the exact rev, then serialize the worktree: files sorted by relative path, ustar headers with mtime = 0, uid = gid = 0, mode 0644 (0755 for directories), no pax headers, no nested `.git` directories, no submodules (submodule presence = `RetrievalError::SubmoduleUnsupported`). The bytes are digested exactly like a registry archive and the digest is the record checksum. The same rev must yield the same digest; a digest change across fetches of the same rev = `RetrievalError::GitDigestDrift` (hard fail).

**Local dependencies** (B27-15). A `path` dependency resolves to the canonicalized path relative to the declaring manifest's directory. Path dependencies carry no checksum; the lockfile records the canonical relative path, and their source inputs enter cache keys as file digests (§20.3). Path dependencies must lie inside the workspace root; an escaping path (`..` beyond root) = `ManifestError::PathEscapesWorkspace`. A path dependency's manifest is validated by the same schema (§20.1 Package manifests) and may itself have path dependencies, with depth ≤ MAX_RESOLVE_DEPTH.

## 20.2 Dependency security (defines design.md 27.2, bullets B27-16..B27-24)

**Dependency authenticity** (B27-16). All registry and git retrieval uses HTTPS with TLS 1.2 or later, mandatory certificate validation against the system CA store, no bypass flag. Artifacts are trusted only after checksum verification against the index (`cksum`) or the lockfile (`checksum`); the lockfile checksum wins when both exist, and a disagreement between index and lockfile checksum = `RetrievalError::ChecksumConflict` (hard fail). Authenticated identity = (source URL, name, version, checksum); anything less is not authenticity.

**Package checksums** (B27-17). The artifact archive format is fixed: tar, ustar headers, entries sorted by path ascending, each file mode 0644 (directories 0755), mtime 0, uid = gid = 0, no pax headers, no compression. `checksum` = BLAKE3-256 over the exact archive bytes. Verification precedes extraction; a mismatch is terminal (`RetrievalError::ChecksumMismatch`), with no fallback, retry-from-other-source, or warning tolerance.

**Signature verification** (B27-18). A registry may publish `<base>/index.sig` = an Ed25519 signature (RFC 8032, pureEd25519) over the canonical JSON of `{"registry": <url>, "epoch": <u64>, "tree_hash": <hex>}`, where `tree_hash` is the snapshot digest of §20.1 Registry support. A registry with `require_signed = true` must have a pinned `root_key`; builds verify the signature against the pinned key and check `tree_hash` against the fetched index; failure = `RetrievalError::SignatureInvalid`. Ed25519 is the only signature algorithm in edition P1.

**Registry trust** (B27-19). Trust is explicit configuration, not observed behavior: a registry is trusted iff it is listed in `registries.toml` (package-level or user-level `~/.planck/registries.toml`). Trust-on-first-use is banned. A registry not in the configuration and not allowed by the policy file is never contacted (`ResolutionError::RegistryNotAllowed`). Changing a pinned `root_key` is an explicit human action; the build manifest records the key fingerprint = base16(BLAKE3-256(public key bytes)) (§20.4).

**Dependency auditing** (B27-20). The advisory database is a JSON Lines document `advisories.jsonl`; each record: `{"id": string, "package": name, "affected": req, "patched": [version], "severity": "low"|"medium"|"high"|"critical", "url": string}`. `plkc audit` (command contract in Volume 21 §21.7) reads the resolved graph from `planck.lock` and reports every advisory whose `affected` interval contains a resolved version that is not in `patched`. Exit code 1 iff at least one unresolved advisory exists. The database fetch is pinned: the pinned database digest is recorded in the build manifest; a database older than the policy limit (§20.2 Supply-chain policy) fails with `AuditError::StaleDatabase`.

**Vulnerability reporting** (B27-21). The audit report is a canonical JSON document `audit-report.json`: `{"schema": 1, "database_digest": hex, "database_date": "YYYY-MM-DD", "findings": [{"package", "version", "advisory_id", "severity", "patched_in"}]}` sorted by (package, advisory_id). Reporting is reproducible: the same (lockfile, database digest) produces byte-identical reports. Reports never contain host paths or timestamps beyond `database_date`.

**Dependency replacement** (B27-22). The `[patch]` table replaces the source of a named dependency for the entire graph: `patch.<name>` is a dependency value (§20.1 Package manifests) with `path`, `git`, or `registry` + `version`. Rules: the patch applies to every requirement on that name whose interval contains the patched version; a patch of a patch is banned (a patched package's own `[patch]` table is ignored, and its presence = `ManifestError::RecursivePatch`); a patch that makes the resolution cycle = `ResolutionError::PatchCycle`. Patch entries are locked like any package.

**Supply-chain policy** (B27-23). The policy file `planck-policy.json` (canonical JSON) has exactly the fields: `allowed_registries` (array of URLs), `git_allowed` (bool), `path_allowed` (bool), `require_signed_registries` (bool), `advisory_max_age_days` (u32), `deny` (array of names), `max_build_units` (u32 ≤ MAX_BUILD_UNITS). Absent policy = defaults: all configured registries allowed, git and path allowed, signing not required, `advisory_max_age_days` = 30, `deny` = [], `max_build_units` = MAX_BUILD_UNITS. Enforcement points: registry selection (§20.1), source-kind admission, audit staleness, deny-list membership (any resolved name in `deny` = `PolicyError::DeniedPackage`), graph size. Policy violations fail the build before any fetch.

**Reproducible dependency retrieval** (B27-24). Retrieval is a total function of (lockfile, local cache content); network access occurs only for content-addressed misses, and every fetched artifact is checksum-verified before it enters the cache. Given a complete cache, `--locked` builds perform zero network operations (OB-20-3). Retrieval must not consult: the system clock, the hostname, the user name, unset environment, or anything else outside (lockfile, cache, policy, registries.toml). Violation of this input-closure rule is a Severity 0 defect.

## 20.3 Build system (defines design.md 27.3, bullets B27-25..B27-42)

**Build targets** (B27-25). A build target is a triple `<arch>-<vendor>-<os>-<abi>`. Implemented set in edition P1 (per SPEC-INDEX §3): `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`. Declared-not-implemented triples are maintained by Volume 16 §23; selecting one = `BuildError::TargetNotImplemented`. The host triple is auto-detected and must be one of the implemented set, else `BuildError::HostUnsupported`. The target fact table (below) is the sole source of `cfg` target facts.

**Compilation units** (B27-26). A package has targets: one `[lib]` (source `src/lib.plk`, name = package name), any number of binaries `[[bin]]` (default `src/bin/<name>.plk`), and tests (`tests/*.plk`, one unit per file). A **compilation unit** is the tuple U = (package, target-kind ∈ {build-script, lib, bin, test}, feature_list_canonical, profile, target-triple). Source files have extension `.plk`. Units are the nodes of the build graph; unit count ≤ MAX_BUILD_UNITS.

**Build graph** (B27-27). The build graph G = (V, E): V = all units of the resolved graph for the selected (target, profile, features), plus one build-script unit per package that has `build.plk`. Edge u → w iff w consumes u's outputs (library link, build-script library, or generated source input). G must be acyclic; a cycle = `BuildError::UnitCycle` listing the cycle nodes (found by planck-graph-math cycle detection per SPEC-INDEX §5.7). G has ≤ MAX_BUILD_UNITS nodes (exceeding = `BuildError::TooManyUnits`).

**Dependency graph** (B27-28). The term *dependency graph* in design.md §27.3 denotes the package-level graph induced by the lockfile (§20.1); the build graph (B27-27) is its unit-level expansion. The expansion rule: each package-level edge (P → D, kind normal) becomes unit edges (lib unit of P → lib unit of D) plus, for build dependencies, (build-script unit of P → lib unit of D compiled for host). Expansion is deterministic; the mapping is part of the reproducibility contract (OB-20-4).

**Build ordering** (B27-29). Units are built in topological order of G computed by Kahn's algorithm with stable tie-break by ascending node id (planck-graph-math contract, SPEC-INDEX §5.7). Node ids are assigned by the canonical ordering key: (kind rank: build-script = 0, lib = 1, test = 2, bin = 3; then package name asc; then feature_list_canonical asc; then profile asc). The order is a pure function of G; parallelism (B27-31) must not change it.

**Incremental builds** (B27-30). Defined in §20.5 and §20.6; the build driver consults the cache keys of §20.6 and skips any unit whose key matches a stored, hash-verified artifact. A unit is *rebuilt* iff its key is absent from the cache or its stored blob fails verification.

**Parallel builds** (B27-31). Worker count J = value of `-j`, default min(8, logical CPU count), legal range 1..=256. The scheduler dispatches only units whose dependency edge count of unfinished parents is zero, in ascending node-id order among ready units. Results, logs, and diagnostics are emitted in node-id order regardless of completion order (output determinism, OB-20-5). Shared state between workers is limited to the cache store and the append-only log, both with per-record atomicity; no worker reads another worker's partial output.

**Build caching** (B27-32). Unit artifacts are content-addressed. The unit key:

```
key(U) = base16(BLAKE3-256( concat(
  lenp("plk-unit-v1"), lenp(kind_tag), lenp(compiler_fingerprint),
  lenp(target_triple), lenp(profile_tag), lenp(feature_list_canonical),
  lenp(source_inputs) )))
```

where kind_tag ∈ {`build-script`, `lib`, `bin`, `test`}; profile_tag = canonical JSON of the resolved profile object; source_inputs = digest list of, in order: (a) each input file sorted by canonical relative path, item = path + BLAKE3-256(file bytes); (b) each declared `rerun-if-env-changed` variable name and its value; (c) the keys of dependency units in canonical order. compiler_fingerprint is §20.4. Two units with equal keys must have byte-identical artifacts (OB-20-6).

**Remote caching** (B27-33). A remote cache is an HTTPS key-value store: `GET /cache/<key>` returns the blob; `PUT /cache/<key>` stores it; authentication is a bearer token from the environment variable `PLANCK_CACHE_TOKEN`. Every downloaded blob is verified: BLAKE3-256(blob) must equal the key; on mismatch the blob is discarded and a warning is emitted (never a build failure). Upload and download failures degrade to local compilation with a warning; they must not fail the build. The remote cache is outside the verification TCB (Volume 22 §22.13): a malicious cache can only remove speed or inject bytes that fail the key check.

**Cross-compilation** (B27-34). `--target <triple>` selects the target; target units compile for the target triple; build-script units always compile for the host triple with the dev profile. Linking for a non-host target requires a sysroot directory (`--sysroot <dir>`) containing the target's libc and startup objects per Volume 16 §23; a missing sysroot = `BuildError::SysrootMissing`. Cache keys include the target triple, so per-target artifacts never collide (§20.6 Target-specific cache).

**Build profiles** (B27-35). Profiles are named objects with exactly the fields: `opt` ∈ {0,1,2,3}, `debug` ∈ {0,1,2,3}, `overflow_checks` ∈ {true,false}, `panic` ∈ {`unwind`,`abort`}, `lto` ∈ {`off`,`full`}, `inline_threshold` ∈ 0..=1000. Defaults:

| Profile | opt | debug | overflow_checks | panic | lto | inline_threshold |
| --- | --- | --- | --- | --- | --- | --- |
| dev (default) | 0 | 2 | true | unwind | off | 64 |
| release | 3 | 0 | false | unwind | off | 256 |

Custom profiles are declared in the manifest `[profile.<name>]` and must set `inherits` = `dev` | `release` plus any subset of the fields; inherited values are overridden field-wise. Profile resolution = base profile fields overridden by the manifest fields; the resolved profile is serialized as `profile_tag` (canonical JSON) in every unit key. `panic = "abort"` makes trap behavior immediate per SPEC-INDEX §3.

**Debug and release modes** (B27-36). `--debug` selects profile `dev`, `--release` selects profile `release`; both flags together = usage error (exit code 2, Volume 21 §21.7). Default = dev. Modes are only profile selections; they add no other behavior.

**Feature configurations** (B27-37). `--features F1,F2` adds features to the root package's feature roots (§20.1 Feature selection); `--no-default-features` sets `default_features = false` on the root's edges. Features of non-root packages come only from their dependents' manifests. The resolved feature sets of all packages are recorded in the build manifest (§20.4) and participate in every unit key.

**Conditional compilation** (B27-38). The cfg predicate grammar: `cfg := fact | fact "=" string | "all(" cfg ("," cfg)* ")" | "any(" cfg ("," cfg)* ")" | "not(" cfg)"`, where `fact` is an identifier. The target fact table, closed: `target_os` ∈ {`linux`}, `target_arch` ∈ {`x86_64`, `aarch64`}, `target_family` = `unix`, `target_env` = `gnu`, `target_abi` = `gnu`, `target_pointer_width` = `64`, `target_endian` = `little`, plus `feature = "F"` for each selected feature. Evaluation is the obvious Boolean semantics; an unknown fact identifier = `CfgError::UnknownFact` (compile-time rejection). The same grammar and fact table govern `#[cfg(...)]` attributes in source (Volume 10 §15 owns attribute semantics; this volume owns dependency-scoped evaluation).

**Generated source** (B27-39). A package root may contain `build.plk`: a build script compiled as a host unit (dev profile) and executed before the package's target units. Execution environment, exactly: cwd = package root; env = {`PLANCK_MANIFEST_DIR`, `PLANCK_OUT_DIR`, `PLANCK_TARGET_TRIPLE`, `PLANCK_FEATURES` (comma list), `PLANCK_PROFILE`, `PLANCK_HOST_TRIPLE`}; no other variables are guaranteed, and reads of undeclared variables must not change outputs. Stdout protocol: one `directive=value` per line; the directive set is closed: `rerun-if-changed=<path>`, `rerun-if-env-changed=<name>`, `link-arg=<arg>`, `link-search=<dir>`, `cfg=<fact>=<value>` (adds a package-local cfg fact). The build script may write files only under `PLANCK_OUT_DIR`; any other write = `BuildScriptError::WriteOutsideOutDir`. Rerun rule: if no `rerun-if-*` directive is printed, the script reruns whenever any input file digest changes; otherwise only the declared files/env names trigger reruns, and declared env values enter the unit key (B27-32). Generated files are inputs of the consuming units by digest. Build script outputs (files + stdout directives) must be byte-identical across executions with identical inputs (normative must; nondeterminism degrades caching and violates the reproducibility contract, OB-20-7).

**Static library generation** (B27-40). A lib unit with `crate_type = staticlib` produces `<name>.a`: a GNU ar archive of the unit's object files (Volume 17 §24 object format), entries sorted by member name, with a symbol-table index member `/` first containing big-endian u32 offsets. Archive mtimes = SOURCE_DATE_EPOCH (§20.4). Two builds with identical inputs produce byte-identical archives (OB-20-4).

**Shared library generation** (B27-41). `crate_type = dylib` produces `lib<name>.so.<major>` (ELF ET_DYN) where `<major>` = package version major; the SONAME field is exactly `lib<name>.so.<major>`. Exported symbols = the package's public ABI surface per Volume 17 §24; symbol order in the dynamic table is sorted ascending (G14). Versioned symlinks are created by the build driver, not embedded in the artifact.

**Executable generation** (B27-42). A bin unit produces the file `<bin-name>`: ELF ET_EXEC (edition P1: no PIE), entry symbol `_start` per Volume 18 §25, with the runtime linked per Volume 17 §24. The binary contains no build timestamps, no absolute paths (§20.4 File path handling), and no undeclared environment dependencies.

## 20.4 Build reproducibility (defines design.md 27.4, bullets B27-43..B27-52; contract goal G14)

**Reproducibility contract.** For identical (source files, manifests, lockfile, policy, compiler binary, target triple, profile, feature roots, SOURCE_DATE_EPOCH), every artifact byte sequence is identical across machines, users, times, and worker counts J ∈ {1, 2, 4, 8} (OB-20-4). This is the volume's implementation of G14.

**Deterministic compiler output** (B27-43). The compiler must not read, and artifacts must not contain: wall-clock time, process ids, host names, user names, absolute paths, environment values other than SOURCE_DATE_EPOCH, thread scheduling order, hash-map iteration order, or any random source. CI enforces byte-identity by double compilation of the conformance corpus with different `-j` values (method EXH over the corpus; OB-20-4).

**Deterministic dependency resolution** (B27-44). Resolution is a pure function of (root manifest set, registry snapshot digest, policy, patch set, target facts) per §20.1; the lockfile is the persisted result. Fresh (lockless) CI resolution uses a pinned snapshot digest recorded in the build manifest; two resolutions over the same inputs are byte-identical (OB-20-2).

**Environment isolation** (B27-45). The only environment variable permitted to influence build output is `SOURCE_DATE_EPOCH`; the only permitted to influence cache-network behavior is `PLANCK_CACHE_TOKEN`. Undeclared variables must not affect any artifact byte, any diagnostic, or any exit code. Build scripts that need variables must declare them via `rerun-if-env-changed`, and declared values enter unit keys (B27-32, B27-39).

**Toolchain identification** (B27-46). `compiler_fingerprint` = `plkc-<semver>+<hex16>` where `<semver>` is the compiler version and `<hex16>` = the first 16 hex characters of BLAKE3-256 over the `plkc` executable's bytes. The fingerprint appears in: every cache key, the build manifest, provenance sidecars, and diagnostics headers. Two binaries with equal fingerprints are interchangeable for reproducibility purposes.

**Build manifests** (B27-47). Each build writes `target/planck/build-manifest.json` (canonical JSON), fields exactly: `schema` = 1; `toolchain` {`version`, `fingerprint`}; `target`; `profile_tag`; `features` (root feature roots); `lockfile_hash` (BLAKE3-256 of `planck.lock` bytes); `index_snapshot_digest`; `source_date_epoch`; `config` (§20.4 Compiler configuration capture); `units` (array sorted by node id: `key`, `kind`, `name`, `artifact_path`, `artifact_hash`, `wall_ms`, `peak_mem_bytes`); `total_wall_ms`; `build_started_unix_ms`. The manifest is a report, not an input: it must never influence artifact bytes.

**Build provenance** (B27-48). Each artifact gains a sidecar `<artifact>.provenance.json` (canonical JSON): `{"schema": 1, "artifact_hash": hex, "unit_key": hex, "toolchain_fingerprint", "target", "profile_tag", "feature_list_canonical", "source_inputs": [{"path", "blake3"}], "lockfile_hash"}`. The sidecar is a pure function of the unit inputs; it contains no wall-clock field (OB-20-8). Provenance is the machine-checkable link from artifact to inputs (verification in §20.4 Artifact verification).

**Timestamp handling** (B27-49). `SOURCE_DATE_EPOCH` = unix seconds, legal range 0..=253402300799, default 0 when unset. It is the only time source inside the build; all embedded times (archive mtimes, debug-info epochs) equal SOURCE_DATE_EPOCH. Reading any other clock inside the compiler's artifact path is banned.

**File path handling** (B27-50). All paths embedded in artifacts (debug info, symbol names, diagnostics, archives) are package-root-relative, forward-slash, UTF-8, without `..` segments or absolute prefixes. The compiler must reject (not silently rewrite) an input whose canonical relative path cannot be computed. Absolute paths in artifacts are a Severity 0 reproducibility defect.

**Compiler configuration capture** (B27-51). The configuration is the ordered list of normalized flags given to the driver; normalization = flag parsing with defaults applied, then re-serialization as the canonical flag list. The list is recorded verbatim in the build manifest field `config`. Two builds whose `config` lists differ may differ in output; two builds with equal `config` lists and equal inputs must not (OB-20-4).

**Artifact verification** (B27-52). `plkc verify` recomputes provenance: for each unit in the build manifest, in node-id order: recompute inputs → recompute key → fetch from cache (local, then remote) or recompile → compare artifact_hash. Exit 0 iff every hash matches; on the first mismatch (canonical order) it exits 1 reporting the unit key, stored hash, and recomputed hash, and performs no repair. Verification reads no clocks and no network except the declared remote cache.

## 20.5 Incremental compilation (defines design.md 27.5, bullets B27-53..B27-65)

The incremental engine is the compiler's query system. Every compilation result is a query value with a key and a recorded dependency set (the keys of subqueries and file digests read during its computation). A query is clean iff all recorded dependencies are clean; dirty queries recompute. Correctness requirement (G23): an incremental build must produce artifacts byte-identical to a full build over the same inputs (OB-20-9), for the whole conformance corpus under single-file-edit mutations.

**Dependency tracking** (B27-53). Dependencies are recorded as (query key, input digest) pairs during evaluation; recording is complete: every value that can change the result must be recorded. Omission is a soundness defect (G23) caught by the cross-check of OB-20-9.

**Source invalidation** (B27-54). A file's parse state is keyed by BLAKE3-256(file bytes); a byte change dirties the parse query; derived AST nodes are identified by (file digest, byte span). No mtime-based invalidation is permitted (mtime is not in the input closure).

**Type invalidation** (B27-55). The query `type_of(item)` is keyed by (item path, item digest, digests of the item's recorded type dependencies). A type change dirties exactly the queries that recorded the old digest as an input; over-invalidation is permitted only up to the precision metric of Volume 23 §23.3 (dependency precision ≥ 0.95 on the corpus).

**Generic instance invalidation** (B27-56). Instance query key = (generic item path, canonical substitution string, option digest). Any change to either component dirties the instance; instances are never reused across option digests.

**Trait implementation invalidation** (B27-57). For each trait T, `impl_table_digest(T)` = BLAKE3-256 over the sorted digests of all impls of T (impl digest = BLAKE3-256 over the impl's canonical serialization). Any goal mentioning T records `impl_table_digest(T)`; a change dirties all such goals. Coarse per-trait granularity is the defined contract; finer granularity is permitted but must remain sound.

**Constant evaluation invalidation** (B27-58). Const-eval query key = (const item path, item digest, type digest); the memoized value is the canonical evaluated value. Const items in cyclic dependency with themselves are rejected before caching (Volume 09 owns const semantics; this volume owns key stability).

**Macro expansion invalidation** (B27-59). Expansion query key = (macro path, macro digest, input token stream digest). Expansion outputs are recorded as spans attributed to the expansion site; a changed expansion dirties the parse of the consuming file segment and all recorded dependents.

**IR cache invalidation** (B27-60). Per-function IR queries (IR 1, IR 2, MIR) are keyed by (function path, function digest, resolved option digest, upstream IR digests in canonical order). An upstream change dirties the function's IR query; the transitive closure is implicit through recorded dependencies, not recomputed by rule.

**Cross-module invalidation** (B27-61). A module's interface digest = BLAKE3-256 over the canonical serialization of its public surface (sorted exported item signatures, types included). Dependent modules record upstream interface digests; a change dirties resolution, type checking, and IR of dependents exactly as recorded. Dev-dependency changes never affect the interface digest (§20.1 Development dependencies).

**Cache serialization** (B27-62). The incremental cache is one file `target/planck-cache/incr-v1.bin`: header = magic `PLKINCR01` (8 bytes) + `format_version` (u32 LE = 1) + compiler_fingerprint (32 bytes raw BLAKE3) + target triple bytes (lenp) + profile_tag (lenp); then records: `query_key` (32 bytes) + `value_len` (u64 LE) + `value_hash` (32 bytes BLAKE3-256 of value) + value bytes. Records are appended in write order; a `fsync` occurs at build end.

**Cache compatibility** (B27-63). A cache file is accepted iff: magic matches; format_version = 1; fingerprint, target, and profile_tag all equal the current build's values. Any mismatch = the file is discarded whole (never an error); the discarding event is counted in build statistics.

**Cache corruption recovery** (B27-64). Each record's `value_hash` is verified on load; a mismatch drops that record only and continues. Trailing bytes shorter than a full record are dropped. Cache recovery never fails a build and never produces a wrong value: values are keyed content, and mis-keyed values are caught by the hash check plus the full-build equivalence of OB-20-9.

**Compilation restart behavior** (B27-65). After abnormal termination: the loader reads the header, then frames records until EOF; a truncated tail is discarded; a corrupted header recreates the file. Restart performs no interactive recovery and no timestamps; the resulting build is defined to be exactly equivalent to a build with those cache records absent.

## 20.6 Compiler caching (defines design.md 27.6, bullets B27-66..B27-77)

**Frontend cache** (B27-66). Kind tag `fe`; value = token stream and AST for one file; key inputs = file digest, edition tag, compiler fingerprint. Reuse across builds requires byte-identical files.

**Generic specialization cache** (B27-67). Kind tag `ginst`; value = one monomorphized instance's MIR; key inputs = generic item path, canonical substitution string, resolved option digest, upstream type digests.

**IR 1 cache** (B27-68). Kind tag `ir1`; value = serialized IR 1 per function; key inputs = function digest, frontend digests, effect-row digest, option digest, upstream IR 1 keys.

**IR 2 cache** (B27-69). Kind tag `ir2`; same discipline as IR 1 with IR 2 option digest and IR 1 key as inputs.

**Machine IR cache** (B27-70). Kind tag `mir`; value = serialized MIR per function; key inputs = IR 2 key, target triple, target feature set, option digest.

**Object cache** (B27-71). Kind tag = unit kind (§20.3 Build caching); value = the object or artifact bytes; key = unit key formula. The object cache and the build-result cache share the key discipline and the store.

**Build result cache** (B27-72). Kind tag `artifact`; value = final artifact bytes (`.a`, `.so`, executable); key = the unit key of the producing unit; verified on load by BLAKE3-256 comparison with the key-embedded input digest set (values carry their own `value_hash` per §20.5 Cache serialization framing).

**Target-specific cache** (B27-73). There is no separate target store: `target_triple` is a mandatory field of every key formula (§20.3 Build caching, §20.6 entries above), so all caches are partitioned per target by construction. Sharing or ignoring the target field is a key-collision defect (Severity 0).

**Cache keys** (B27-74). All caches use one formula: `key = base16(BLAKE3-256(concat(lenp("plk-cache-v1"), lenp(kind_tag), lenp(compiler_fingerprint), lenp(target_triple), lenp(profile_tag), lenp(feature_list_canonical), lenp(inputs))))`, where `inputs` is the kind-specific digest list of the entries above, in the order stated. Field order is fixed by this list; reordering fields is a breaking format change (§20.6 Cache invalidation).

**Cache invalidation** (B27-75). Invalidation is key-identity only: a changed input changes the key, so the old entry is unreachable. Time-to-live, mtime checks, clock reads, and heuristic invalidation are banned in cache decisions. Format changes bump the `plk-cache-v1` tag to `plk-cache-v2`, which orphans all prior entries (accepted, not an error).

**Cache storage** (B27-76). Layout: `<workspace>/target/planck-cache/<kind_tag>/<key[0:2]>/<key[2:64]>`; file = header {key (32 bytes raw), kind_tag (lenp), value_len (u64 LE)} + blob. Maximum cached entry size = 536 870 912 bytes (512 MiB); larger results are recomputed, never cached. The store root is `target/planck-cache` by default and configurable to one directory; stores are per-workspace and never shared across workspaces without the shared remote cache (§20.3 Remote caching).

**Cache eviction** (B27-77). Eviction is LRU over entries of one store, executed at build start. Each entry records `last_use` = the monotonic build counter (u64, incremented per build; wall-clock time banned). Default capacity = 10 GiB (config `cache.size_gib`, integer 1..=1024); eviction removes lowest-`last_use` entries until usage ≤ 90% of capacity. Eviction is deterministic given the same access history (OB-20-10) and never evicts entries created by the current build.

## 20.7 Deliverables mapping (design.md line 3239)

| Deliverable | Defined by |
| --- | --- |
| Package Manager Specification | §20.1, §20.2 |
| Build System Specification | §20.3 |
| Incremental Compilation Specification | §20.5, §20.6 |
| Reproducible Builds Specification | §20.4 |

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-20-1 | Manifest parsing accepts exactly the schema of §20.1 and rejects every field/type/range violation; EXH over the grammar cross-product of field presence × type error classes (≤ 10^6 cases). | EXH | FV-0 | `tools/fv/manifest_schema.z3` + test table | G19, G22 |
| OB-20-2 | Resolution determinism: identical (manifests, snapshot digest, policy, target facts) produce byte-identical lockfiles; PROP over generated requirement sets, paired with the termination bound EXH (E ≤ 5120). | PROP+EXH | FV-1 | `tools/fv/resolve_determinism.smt2` | G14, G11 |
| OB-20-3 | Requirement-interval semantics: every requirement string maps to exactly the interval of §20.1; EXH over the comparator × version-boundary domain (boundary values 0, x-1, x, x+1, MAX per component). | EXH | FV-0 | `tools/fv/req_intervals.smt2` | G19 |
| OB-20-4 | Build reproducibility: for the conformance corpus, artifacts are byte-identical across double compilation with J ∈ {1, 2, 4, 8} and across machines with equal fingerprints. | EXH | FV-1 | CI double-compile job | G14, G7 |
| OB-20-5 | Build-order determinism: scheduled unit order equals Kahn stable topological order for every corpus build graph. | EXH | FV-0 | `planck-graph-math` certificate + CI log | G14, G11 |
| OB-20-6 | Cache-key injectivity on the corpus: two units differing in any input field have different keys (EXH over field-mutation domain per unit kind). | EXH | FV-1 | `tools/fv/cache_key.smt2` | G14 |
| OB-20-7 | Build-script sandbox: writes outside OUT_DIR and reads of undeclared env are rejected; EXH over the directive grammar. | EXH | FV-0 | `tools/fv/buildscript_protocol.z3` | G3, G12 |
| OB-20-8 | Provenance purity: sidecar bytes are a function of unit inputs only (recompute-and-compare over corpus). | EXH | FV-1 | CI provenance check | G14, G7 |
| OB-20-9 | Incremental equivalence: single-file-edit incremental builds produce artifacts byte-identical to full builds over the conformance corpus. | EXH | FV-1 | CI incremental job | G23, G14 |
| OB-20-10 | Eviction determinism: same access history ⇒ same evicted set; EXH over generated histories (≤ 10^6 events). | EXH | FV-0 | `tools/fv/cache_lru.z3` | G14 |
| OB-20-11 | Lockfile integrity: every `content_hash` recomputes; a single flipped byte in any field is detected. EXH over single-field mutations of corpus lockfiles. | EXH | FV-0 | CI lockfile check | G14 |
| OB-20-12 | Checksum enforcement: any archive whose BLAKE3-256 differs from the record checksum is refused before extraction; EXH over bit-flip mutations (one byte, all offsets of a 4 KiB sample). | EXH | FV-0 | `tools/fv/checksum_gate.z3` | G2, G3 |
| OB-20-13 | Invalidation completeness: for every corpus mutation, the set of dirty queries is a superset of the recorded-dependency closure; PROP over edit sequences paired with OB-20-9. | PROP | FV-1 | CI incremental job | G23 |
| OB-20-14 | Policy enforcement: every §20.2 policy violation class halts resolution before fetch; EXH over policy × source matrix. | EXH | FV-0 | `tools/fv/policy_gate.z3` | G3 |
| OB-20-15 | cfg evaluation: predicate evaluation equals Boolean semantics over the closed fact table; EXH over grammar-generated predicates (depth ≤ 4). | EXH | FV-0 | `tools/fv/cfg_eval.smt2` | G19 |
| OB-20-16 | Input closure: no compiler artifact byte depends on banned inputs; REVIEW over the input-closure argument plus the OB-20-4 differential evidence. | REVIEW | FV-1 | Reproducibility review record | G14, G16 |
| OB-20-17 | Limits are honored: every MAX_* constant of §20.0.1 is enforced at the stated boundary with the stated error; EXH over boundary values. | EXH | FV-0 | `tools/fv/limits_gate.z3` | G13 |
| OB-20-18 | Resolver termination: the event counter E is bounded by 5120 and every loop strictly increments it; Z3 over the state-machine abstraction (bounded instances, 4096 packages × 4096 versions modeled symbolically). | Z3 | FV-1 | `tools/fv/resolve_termination.smt2` | G11 |
| OB-20-19 | Canonical JSON: serializer output is RFC 8785-conformant on the corpus of all lock/manifest/provenance documents; EXH over the document corpus. | EXH | FV-0 | CI canonicalization check | G14 |
| OB-20-20 | Git digest stability: same rev ⇒ same digest across two fetches; EXH over the git-dependency corpus. | EXH | FV-1 | CI retrieval job | G14, G7 |

## Coverage ledger

design.md §27, lines 3140–3242. Bullets in scope: 77, defined: 77. The Deliverables line (line 3239) is defined in §20.7.

| # | Bullet (abbreviated) | Defined in |
| --- | --- | --- |
| B27-1 | Package identity | §20.1 Package identity |
| B27-2 | Package versions | §20.1 Package versions |
| B27-3 | Package manifests | §20.1 Package manifests |
| B27-4 | Dependencies | §20.1 Dependencies |
| B27-5 | Dependency constraints | §20.1 Dependency constraints |
| B27-6 | Feature selection | §20.1 Feature selection |
| B27-7 | Optional dependencies | §20.1 Optional dependencies |
| B27-8 | Platform-specific dependencies | §20.1 Platform-specific dependencies |
| B27-9 | Development dependencies | §20.1 Development dependencies |
| B27-10 | Build dependencies | §20.1 Build dependencies |
| B27-11 | Dependency resolution | §20.1 Dependency resolution |
| B27-12 | Lockfiles | §20.1 Lockfiles |
| B27-13 | Registry support | §20.1 Registry support |
| B27-14 | Git dependencies | §20.1 Git dependencies |
| B27-15 | Local dependencies | §20.1 Local dependencies |
| B27-16 | Dependency authenticity | §20.2 Dependency authenticity |
| B27-17 | Package checksums | §20.2 Package checksums |
| B27-18 | Signature verification | §20.2 Signature verification |
| B27-19 | Registry trust | §20.2 Registry trust |
| B27-20 | Dependency auditing | §20.2 Dependency auditing |
| B27-21 | Vulnerability reporting | §20.2 Vulnerability reporting |
| B27-22 | Dependency replacement | §20.2 Dependency replacement |
| B27-23 | Supply-chain policy | §20.2 Supply-chain policy |
| B27-24 | Reproducible dependency retrieval | §20.2 Reproducible dependency retrieval |
| B27-25 | Build targets | §20.3 Build targets |
| B27-26 | Compilation units | §20.3 Compilation units |
| B27-27 | Build graph | §20.3 Build graph |
| B27-28 | Dependency graph | §20.3 Dependency graph |
| B27-29 | Build ordering | §20.3 Build ordering |
| B27-30 | Incremental builds | §20.3 Incremental builds (→ §20.5, §20.6) |
| B27-31 | Parallel builds | §20.3 Parallel builds |
| B27-32 | Build caching | §20.3 Build caching |
| B27-33 | Remote caching | §20.3 Remote caching |
| B27-34 | Cross-compilation | §20.3 Cross-compilation |
| B27-35 | Build profiles | §20.3 Build profiles |
| B27-36 | Debug and release modes | §20.3 Debug and release modes |
| B27-37 | Feature configurations | §20.3 Feature configurations |
| B27-38 | Conditional compilation | §20.3 Conditional compilation |
| B27-39 | Generated source | §20.3 Generated source |
| B27-40 | Static library generation | §20.3 Static library generation |
| B27-41 | Shared library generation | §20.3 Shared library generation |
| B27-42 | Executable generation | §20.3 Executable generation |
| B27-43 | Deterministic compiler output | §20.4 Deterministic compiler output |
| B27-44 | Deterministic dependency resolution | §20.4 Deterministic dependency resolution |
| B27-45 | Environment isolation | §20.4 Environment isolation |
| B27-46 | Toolchain identification | §20.4 Toolchain identification |
| B27-47 | Build manifests | §20.4 Build manifests |
| B27-48 | Build provenance | §20.4 Build provenance |
| B27-49 | Timestamp handling | §20.4 Timestamp handling |
| B27-50 | File path handling | §20.4 File path handling |
| B27-51 | Compiler configuration capture | §20.4 Compiler configuration capture |
| B27-52 | Artifact verification | §20.4 Artifact verification |
| B27-53 | Dependency tracking | §20.5 Dependency tracking |
| B27-54 | Source invalidation | §20.5 Source invalidation |
| B27-55 | Type invalidation | §20.5 Type invalidation |
| B27-56 | Generic instance invalidation | §20.5 Generic instance invalidation |
| B27-57 | Trait implementation invalidation | §20.5 Trait implementation invalidation |
| B27-58 | Constant evaluation invalidation | §20.5 Constant evaluation invalidation |
| B27-59 | Macro expansion invalidation | §20.5 Macro expansion invalidation |
| B27-60 | IR cache invalidation | §20.5 IR cache invalidation |
| B27-61 | Cross-module invalidation | §20.5 Cross-module invalidation |
| B27-62 | Cache serialization | §20.5 Cache serialization |
| B27-63 | Cache compatibility | §20.5 Cache compatibility |
| B27-64 | Cache corruption recovery | §20.5 Cache corruption recovery |
| B27-65 | Compilation restart behavior | §20.5 Compilation restart behavior |
| B27-66 | Frontend cache | §20.6 Frontend cache |
| B27-67 | Generic specialization cache | §20.6 Generic specialization cache |
| B27-68 | IR 1 cache | §20.6 IR 1 cache |
| B27-69 | IR 2 cache | §20.6 IR 2 cache |
| B27-70 | Machine IR cache | §20.6 Machine IR cache |
| B27-71 | Object cache | §20.6 Object cache |
| B27-72 | Build result cache | §20.6 Build result cache |
| B27-73 | Target-specific cache | §20.6 Target-specific cache |
| B27-74 | Cache keys | §20.6 Cache keys |
| B27-75 | Cache invalidation | §20.6 Cache invalidation |
| B27-76 | Cache storage | §20.6 Cache storage |
| B27-77 | Cache eviction | §20.6 Cache eviction |
