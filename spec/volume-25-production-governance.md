<!-- CEP:FILE: spec/volume-25-production-governance.md -->
<!-- CEP:WHAT: Volume 25 of the Planck master specification library: the Production and Governance Specification. Defines the versioning scheme (semver plus the stable/beta/nightly channel model), the release artifact list with hashes, the language evolution process (RFC lifecycle, edition gates, deprecation for 2 editions then removal), the documentation registry (24 documents, per doc standards.md), and maintenance and ecosystem governance (contribution contract, triage SLAs, disclosure process, registry operations, roles). -->
<!-- CEP:WHY: design.md §33-§36 (lines 3856-3941) enumerates 66 production and governance items as bare bullets. Without exact definitions, release identity, evolution decisions, and maintenance obligations would be discretionary instead of normative, breaking G26 (verified evolution) and the reproducibility success criterion (G14, G25). -->
<!-- CEP:STATUS: complete -->
<!-- CEP:FAILURE: A release without a signed manifest and per-artifact SHA-256; a version string outside the grammar of §25.1.1; a breaking change outside an edition; a removed feature whose removal edition is not exactly 2 editions after its deprecation edition; a document outside the registry of §25.3; a contribution without the CEP&CC §17 checklist attestation. Each of these is a release-blocking defect. -->
<!-- CEP:ASSUMES: SPEC-INDEX.md §2 (volume map) and §4 (verification scheme); standards.md CEP&CC 0.4 (§3 verbs, §17 review checklist); verification standards.md G14, G22, G25, G26; doc standards.md (style, structure, CI enforcement); Volume 24 (§24.4 supply chain, §24.3 sandbox, exit codes); Volumes 01-23 as sources of truth for the documentation registry; Volume 26 (plki ships in the release artifact list). -->
<!-- CEP:COST: document. Release engineering costs are one-time per release (CI jobs named in the verification matrix). Governance costs are process costs with the SLAs stated inline. -->
<!-- CEP:EVIDENCE: the verification matrix (OB-25-01 through OB-25-18) and the coverage ledger (66 design.md bullets) at the end of this file. -->

# Volume 25: Production, release, and governance

## 25.0 Scope and traceability

This volume defines design.md §33 through §36, lines 3856-3941, and is the deliverable named at line 3936: the Production and Governance Specification.

Bullet counts, verified by enumeration of design.md lines 3856-3941: §33 has 16 bullets, §34 has 14, §35 has 24, §36 has 12. Bullets in scope: 66, defined: 66. The prose line 3894 ("You need documentation for:") is satisfied by the documentation registry of §25.3; the deliverable line 3936 is this file.

Precedence. Release and security mechanics are split as follows: signature scheme, provenance format, advisory format, tamper detection, and reproducibility checks are Volume 24 §24.4; this volume defines the release calendar, artifact set, installation lifecycle, evolution process, documentation registry, and governance. Where both volumes state a value (hash algorithm, signing key usage, exit codes), the values are identical by construction; Volume 24 is authoritative for verification mechanics and this volume for release policy.

## 25.1 Release engineering (design.md §33)

### 25.1.1 Version numbering

The compiler version is a SemVer 2.0.0 string with this exact grammar:

```text
version := major "." minor "." patch pre? build?
major   := "0" during edition P1 | "1" or higher from the first post-P1 edition
minor   := digits, value < 2^31, no leading zero
patch   := digits, value < 2^31, no leading zero
pre     := "-beta." k | "-nightly-" date
k       := positive integer, no leading zero
date    := YYYYMMDD, a valid calendar date
build   := "+" 40 lowercase hex characters   (mandatory for nightly, absent otherwise)
```

Rules, exactly: while the edition is P1 the major component is 0; minor increments on any backward-incompatible change that an edition boundary permits (spec-error fixes, diagnostic changes, infra behavior changes); patch increments on backward-compatible fixes and additive changes; the major component becomes 1 at the first edition after P1 that permits breaking language changes. The language edition identifier (P1, P2, ...) is independent of the version. `plkc --version` prints exactly one line: `plkc <version> (edition <E>, channel <C>, commit <40hex>)`. Toolchain identity is the pair `<version>-<channel>`; its installed directory is `<prefix>/toolchains/<version>-<channel>/`.

### 25.1.2 Release channels

There are exactly three channels: stable, beta, nightly. The channel determines: which feature gates are permitted (nightly only, per §25.2.4), the support policy (§25.4.11), the update flow (§25.1.12), and the version grammar branch (§25.1.1). The default channel is recorded in `<prefix>/settings.toml` and is changed only by explicit user action (`plkc default set`, `plkc update` for stable and beta per §25.1.12).

### 25.1.3 Stable releases

The release train: the branch day D is the first Monday of January, April, July, and October. The stable release ships 42 days after D. Stable versions carry no pre-release component and no build metadata. Patch releases (version N.M.(P+1)) ship within 14 days of the fix landing, only for critical security advisories (Volume 24 §24.4.8 severity critical) or miscompilations (S0 in Volume 22), and only within the support window of §25.4.11. A stable release must pass every gate of the release checklist (§25.4.10).

### 25.1.4 Development releases

Beta builds are cut from the release branch: beta.1 at D+14 days, beta.2 at D+28 days, and weekly point builds every Tuesday 12:00 UTC thereafter until the stable day. Beta versions are `0.M.0-beta.K` with K incremented per build. A beta must pass the conformance suite (Volume 22 §29.1) before publication; a failing check holds publication (train stop rule, §25.4.4). Beta builds carry the opt-in flags for the next edition.

### 25.1.5 Nightly builds

A nightly is built from main HEAD every day at 03:00 UTC. Versions are `0.M.0-nightly-YYYYMMDD+<40hex>`; the build metadata is the commit hash, mandatory. Publication gate: the full conformance suite green; on failure the nightly is not published and the channel continues to serve the previous day's build. Nightlies may enable the feature-gate mechanism (`#![feature(...)]` attributes); stable and beta editions reject `#![feature]` with a compile-time rejection. Nightly-only behavior is otherwise identical to the specification; the specification volumes define semantics, and a nightly feature that is not in the volumes is a gate error, not a semantic extension.

### 25.1.6 Compiler distribution

Host platforms, exactly 5: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin. The codegen targets are the Volume 16 set (x86-64 SysV, AArch64); each host artifact can compile for every codegen target offline. Artifact list per release, exactly 8 files: `planck-<version>-<H>.tar.gz` for each of the 5 hosts, `planck-<version>-source.tar.gz`, `release-manifest.json`, `release-manifest.json.minisig`. Tarball compression is gzip level 9 with sorted entry order and fixed mtimes equal to the release commit time. Size limits: binary tarball at most 300 MiB, source tarball at most 100 MiB; an oversized artifact fails the release.

### 25.1.7 Binary packaging

Each host tarball contains exactly this tree:

```text
bin/plkc            compiler driver
bin/plkc-run        sandbox runner (Volume 24 §24.3)
bin/plk-pkg         package manager client
bin/plki            reference interpreter (Volume 26 §26.1)
bin/plk-fmt         formatter (Volume 21)
bin/plk-ls          language server (Volume 21)
lib/planck/std/<target>/    precompiled standard library per codegen target
lib/planck/std-src/         standard library sources
share/doc/                  rendered documentation of the release
manifest.json               toolchain manifest (schema "planck-toolchain/1")
manifest.sha256             per-file SHA-256 list (Volume 24 §24.4.5)
```

Prohibitions, exact: no daemon, no system service, no scheduled task, no writes outside its own installation directory and the user's cache and target directories, no registry modification, no shell configuration file modification.

### 25.1.8 Source packaging

The source tarball is `planck-<version>-source.tar.gz`, produced by a deterministic archive of the release commit with all submodules vendored and a top-level prefix directory `planck-<version>/`. It contains everything needed to build stage 0 offline: the full compiler source, the math kernels (zero external dependencies per SPEC-INDEX §5.2), the vendored stage-0 build inputs, and `rust-toolchain.toml` pinning the stage-0 Rust toolchain. Two archives of the same commit are byte-identical (fixed mtimes, sorted order, uid and gid 0); this is part of the reproducibility obligation OB-24-11.

### 25.1.9 Cross-platform distribution

Each of the 5 host artifacts ships the standard library precompiled for every implemented codegen target, so any host compiles for any target with no network access and no additional downloads (the offline law of Volume 20). Cross-compilation additionally requires the linker of the target platform; the build system documents, per target pair, the linker requirement, and fails with E-LINKER-MISSING, exit 3, when absent. No other cross-platform variance is permitted: compilation results are byte-identical for the same (source, configuration, target, toolchain) regardless of host, per G14.

### 25.1.10 Installation

The installer (install.sh on POSIX, install.ps1 on Windows) performs exactly these steps: (1) verify the release manifest signature (Volume 24 §24.4.1) and print the signing key fingerprint; (2) verify the SHA-256 of every downloaded artifact against the manifest; (3) extract to `<prefix>/toolchains/<id>`; (4) run `plkc doctor --verify` (Volume 24 §24.4.5); (5) update the default pointer in `<prefix>/settings.toml` only when the user requested it; (6) print the exact PATH line to add. The installer must not edit shell configuration files. Default prefix: `$HOME/.planck` on POSIX, `%LOCALAPPDATA%\Planck` on Windows. Installer exit codes, exhaustive: 0 success, 1 signature verification failure, 2 checksum failure, 3 extraction or filesystem failure, 4 doctor verification failure. `plkc toolchain install <id>` performs the same steps through the package manager download path with the same mandatory verification; no verification bypass flag exists (banned).

### 25.1.11 Uninstallation

`plkc toolchain uninstall <id>` deletes exactly the directory `<prefix>/toolchains/<id>/` and nothing else, after the containment check: the canonical path of the directory must be inside `<prefix>/toolchains/`; otherwise the command refuses with exit 3. When the target is the only installed toolchain, the command refuses unless `--force` is given. When the removed toolchain was the default, the pointer moves to the newest remaining stable toolchain, or is cleared if none remains. Cache directories are untouched; `plkc cache clean` is the only cache deletion command.

### 25.1.12 Upgrade handling

Upgrades install side-by-side; an upgrade never overwrites an existing toolchain directory in place. `plkc update` for the stable and beta channels: fetch the release manifest, verify the signature, download the host artifact, verify its checksum, run the installation steps of §25.1.10, and switch the default pointer to the new version. Nightly never switches the default pointer. A workspace file `planck-toolchain.toml` containing `toolchain = "<id>"` pins the toolchain for that workspace and overrides the default pointer; a pinned toolchain that is not installed is a defined error, exit 3, naming the pin.

### 25.1.13 Rollback

Rollback is `plkc default set <previous-id>`: previous toolchains remain installed until explicitly uninstalled (retention is unlimited; disk use is the user's responsibility). Cache compatibility is guaranteed by the compiler version hash in every cache entry (Volume 24 §24.3.10): a rollback finds comp_ver mismatches and recomputes every affected entry; it never reuses and never migrates cache formats. No other rollback action exists, so rollback cannot corrupt a build.

### 25.1.14 Package signatures

Release artifacts carry two signature kinds, both mandatory: the release manifest signature by the release root key (Volume 24 §24.4.1) and, for registry content, the package manifest signatures by the registry key (Volume 24 §24.4.2). Rules, exact: the signature covers the manifest bytes; verification is mandatory in the installer and in `plkc update`; a bypass flag is banned; release testing uses a dev root key in a separate key namespace identified by a key id field in the manifest, so a dev-signed artifact is rejected by production verifiers by key mismatch.

### 25.1.15 Release checksums

`release-manifest.json` lists, for every artifact: path, SHA-256 (64 lowercase hex), and size in bytes. The file `manifest.sha256` inside each tarball lists `<64 hex>  <relative path>` for every shipped file (Volume 24 §24.4.5). The checksum algorithm is SHA-256, exact; no other algorithm is accepted. Verification of both lists is mandatory at install and update time (§25.1.10, §25.1.12); a mismatch aborts with the corresponding installer exit code (2) and leaves no partially installed toolchain (extraction happens into a temporary directory renamed on success).

### 25.1.16 Reproducible builds

The release is reproducible per Volume 24 §24.4.6. Release-side fields in `release-manifest.json` pin the inputs: source tarball SHA-256, stage-0 toolchain hash, target list, exact build command line, and SOURCE_DATE_EPOCH equal to the release commit timestamp. `plkc verify-rebuild <artifact>` rebuilds from the recorded inputs and compares SHA-256; CI runs the two-build equality gate `repro-check` on every release (OB-24-11). A release whose reproducibility check fails is not published.

## 25.2 Language evolution (design.md §34)

### 25.2.1 Feature proposal process

Proposals are RFC documents in the format `planck-rfc/1` with exactly these fields: number (PLK-RFC-NNNN, 4 digits, assigned monotonically), title, feature-gate name (for new features), motivation (at most 500 words), guide-level explanation, reference-level explanation (the semantic spec delta), alternatives, unresolved questions. Lifecycle states, exactly 5: draft, in-review, accepted, rejected, withdrawn. A transition into in-review requires the file `rfcs/NNNN-<slug>.md` to exist and be complete per the field list; in-review lasts at least 14 days. The state machine is total: every state transition names its actor and date, and "deferred" is not a state; a proposal not ready for decision is withdrawn or stays in draft.

### 25.2.2 Language change proposals

Two proposal classes exist: new-feature RFCs (§25.2.1) and change proposals that modify existing semantics. A change proposal carries, in addition to the RFC fields: an impact analysis listing the affected specification volumes by number, migration notes for affected code, and measured cost deltas (compile time and runtime on the Volume 23 benchmark suite). The same 5-state lifecycle applies.

### 25.2.3 Design review

Every RFC in in-review is assigned exactly 2 designated reviewers: one from the language team, one from the implementation team. The decision is made by the language council (5 members, §25.4.9) by consensus; a blocked council decides by majority vote of its members. A decision record block `## Decision` (date, per-member votes, rationale) is appended to the RFC file; the state transition without a decision record is invalid. No proposal is accepted silently.

### 25.2.4 Prototype implementation

Before a feature RFC can be accepted, a prototype must exist behind a nightly feature gate (§25.2.1 feature-gate name). The prototype must include the 16-row feature template of design.md §40, filled for the feature, with any non-applicable row stating "not applicable because ...", and at least one compile-fail test per rejected construction named in the template. A prototype is not a stabilization commitment: acceptance authorizes implementation and review, never stabilization (§25.2.9).

### 25.2.5 Semantic review

The reference-level explanation is reviewed as a semantic spec delta, written in the notation of Volumes 04 through 07 (judgments, typing rules, execution rules, effect rows). The spec delta must be merged into the volume library before the implementation merges into main; the CI check `rfc-spec-present` rejects an implementation PR for a gated feature that does not reference the merged spec delta.

### 25.2.6 Performance evaluation

Every change to IR definitions, passes, backend, or runtime requires a benchmark run on the Volume 23 suite before merge. Gates: no runtime regression greater than 2% and no compile-time regression greater than 5% on any benchmark, without a written waiver stored as `perf-waivers/<date>-<bench>.md` with a justification and an expiry of at most 2 releases. Results are stored as `perf-history/<date>-<commit>.json` with the full measurement record.

### 25.2.7 Safety review

Features that add intrinsics, widen the FFI surface, add layout attributes, or otherwise touch unsafe-adjacent semantics require a safety review per standards.md §22. The review enumerates every interaction of the change with UB-1 through UB-6 (SPEC-INDEX §3) and must establish that no new UB category is created. Adding a UB category requires a council decision and a SPEC-INDEX §3 update; without that update, a change introducing a new UB category is rejected.

### 25.2.8 Compatibility review

Every proposal is classified into exactly one class: additive (no valid edition-P1 program changes observable behavior), bugfix (only invalid or previously-rejected programs change behavior), breaking (some valid program changes observable behavior or stops compiling). Breaking changes are legal only through an edition boundary (§25.2.12), never in a minor or patch release; a breaking proposal must contain a migration guide section and name the edition gate that carries it.

### 25.2.9 Stabilization

A feature stabilizes exactly when all of these hold: the implementation is merged; the spec delta is merged with the 16-row template complete; the conformance tests exist with at least 1 positive and 1 compile-fail test per applicable template row; the documentation registry row (§25.3) is updated; the RFC has no open unresolved question. Stabilization happens only in a minor or major release, never in a patch release and never on nightly. The act of stabilization is: remove the feature gate, record the stabilization note with the RFC number in the release notes.

### 25.2.10 Deprecation

Deprecation policy, settled: deprecated for 2 editions, then removed. Exactly: a feature deprecated in edition E emits the warning W-DEP-<feature> on every use in editions E and E+1 (default warn; `--deny-deprecated` escalates to a rejection); in edition E+2 the feature is a compile-time rejection. Within an edition, minor releases may add deprecation warnings and must not remove the feature. The deprecation announcement names the removal edition, so users know the window: 2 editions of warnings, then removal.

### 25.2.11 Removal

Removal is a parse-level or type-level rejection with the code E-REMOVED whose message contains: the migration text, the edition of deprecation, and the edition of removal. The RFC file and the documentation page remain, marked "removed in <edition>". Editions older than the removal edition keep the feature defined by their specification volumes (edition-gated definitions); removal never rewrites history.

### 25.2.12 Edition policy

Editions are named P1, P2, ... An edition may change: parsing, defaults, the set of removed features (per §25.2.10), and the set of stabilized features. An edition must ship: a migration document listing every behavioral delta, and the mechanical migration tool `plkc migrate <from> <to>` that performs exactly the documented transformations; a transformation not covered by the tool is listed as manual in the migration document. The package manifest field `edition` is mandatory in every package manifest; there is no implicit default edition. `--edition` on the command line overrides the manifest; a mismatch between the two is a defined error, exit 1.

### 25.2.13 RFC archival

After a decision, the RFC file is frozen: any later change is made by a new RFC that references the old number. The repository retains every RFC in its decided form; `rfcs/INDEX.md` lists for every RFC: number, title, state, decision date. The rendered set is published with each release under `share/doc/rfc/`. History is retained indefinitely (§25.4.12).

### 25.2.14 Specification versioning

The specification library (26 volume files plus SPEC-INDEX.md, 27 files) is versioned with the compiler: the spec version string is `<compiler-version>+spec.<h>` where h is the first 8 hexadecimal characters of SHA-256 over the concatenation of the 27 files sorted by path. The release manifest records the field spec_version (§25.1.16). Evolution obligations (G26): a PR that changes a volume must list the affected obligation IDs in its description (CI check `spec-impact-listed`), and the verification matrices of changed volumes re-run in CI. Changing an IR invariant must identify every obligation whose statement depends on it, via the obligation source column of the matrices.

## 25.3 Documentation (design.md §35)

### 25.3.1 Documentation policy

Every document of the project satisfies doc standards.md: objectivity, no marketing language, no emojis, bold restricted to headers and defined terms, bullets at most 2 levels deep, all examples compile in CI, structure per doc standards.md §3 for component documents. Each document carries a header block with the fields: `doc:` (its path), `source-of-truth:` (the volume anchors listed below), `spec-version:` (the §25.2.14 string of the release it was generated or last verified against). Co-location law (doc standards.md §5.1 applied to volumes): a document must be updated in the same PR as a change to its source-of-truth volumes; CI fails a PR that changes a volume without the corresponding document change. Documents for removed features move to `docs/archive/<edition>/` with the banner line "removed in edition <E>" (§25.2.11).

### 25.3.2 Documentation registry

The 24 documents required by design.md §35, each with its exact path, source of truth, required sections, and update trigger:

| # | Document | Path | Source of truth | Required sections | Update trigger |
| --- | --- | --- | --- | --- | --- |
| 1 | Language specification | docs/lang/spec.md | Volumes 01-10 | Syntax; Names; Types; Memory; Ownership; Effects; Concurrency; Compile-time; Attributes and FFI | Same PR as a change to Volumes 01-10 |
| 2 | Compiler architecture | docs/compiler/architecture.md | Volumes 11-17 | Stage list; Pipeline; IR summary; Invariants | Same PR as a change to Volumes 11-17 |
| 3 | Type system | docs/lang/types.md | Volume 04 | Rules; Inference; Limits | Same PR as Volume 04 |
| 4 | Borrow checker | docs/lang/ownership.md | Volume 06 | State machine table; Conflict predicate; Error codes | Same PR as Volume 06 |
| 5 | Memory model | docs/lang/memory-model.md | Volume 05; SPEC-INDEX §3 | UB list UB-1..UB-6; Provenance; Layout; Destruction | Same PR as Volume 05 or SPEC-INDEX §3 |
| 6 | Effects | docs/lang/effects.md | Volume 07; SPEC-INDEX §5.3 | Atoms; Rows; Call rule | Same PR as Volume 07 |
| 7 | Concurrency | docs/lang/concurrency.md | Volume 08 | Memory model for races; Task model | Same PR as Volume 08 |
| 8 | Async | docs/lang/async.md | Volume 08 | Structured concurrency; Cancellation | Same PR as Volume 08 |
| 9 | IR 1 | docs/compiler/ir1.md | Volume 13 | Node set; Typing; Invariants | Same PR as Volume 13 |
| 10 | IR 2 | docs/compiler/ir2.md | Volume 14 | Representations; Passes | Same PR as Volume 14 |
| 11 | Machine IR | docs/compiler/mir.md | Volume 15 | Instruction set; Registers | Same PR as Volume 15 |
| 12 | Backend architecture | docs/compiler/backend.md | Volume 16 | Targets; Selection; Register allocation | Same PR as Volume 16 |
| 13 | Runtime | docs/runtime/runtime.md | Volume 18 | Entry ABI; Traps; Memory; OOM | Same PR as Volume 18 |
| 14 | Standard library | docs/stdlib/ (generated) | Volume 19; doc comments | Per item: Signature; Semantics; Traps and errors; Examples | Same PR as Volume 19 or the item |
| 15 | FFI | docs/lang/ffi.md | Volume 10; Volume 24 §24.6.7 | Declarations; Ownership attributes; Conventions | Same PR as Volume 10 or §24.6.7 |
| 16 | ABI | docs/lang/abi.md | Volume 10 | Conventions; Layout; Mangling | Same PR as Volume 10 |
| 17 | Build system | docs/tools/build.md | Volume 20 | Targets; Profiles; Cache; Reproducibility | Same PR as Volume 20 |
| 18 | Package manager | docs/tools/packages.md | Volume 20 | Manifest reference; Registry policy; Lockfiles | Same PR as Volume 20 |
| 19 | Tooling | docs/tools/README.md | Volume 21 | Command reference per tool | Same PR as Volume 21 |
| 20 | Debugging | docs/tools/debugging.md | Volume 21 §28.4; Volume 17 | Debug info format; Debugger commands | Same PR as either source |
| 21 | Profiling | docs/tools/profiling.md | Volume 21 §28.5; Volume 18 §25.6 | Counters; Tools | Same PR as either source |
| 22 | Formal verification | docs/verification/overview.md | Volume 22; SPEC-INDEX §4 | Obligation map; How to run tools/fv | Same PR as Volume 22 |
| 23 | Compiler development | docs/dev/hacking.md | Volume 12 | Build; Layout; CEP classes; Phase boundaries | Same PR as Volume 12 |
| 24 | Contribution guidelines | CONTRIBUTING.md | standards.md; this volume §25.4 | PR checklist (CEP&CC §17); Review requirements; Code classes; Tests | Same PR as a change to §25.4 or standards.md §17 |

The registry is complete: these 24 rows are all the documents the project maintains, and every row is one design.md bullet.

## 25.4 Maintenance and ecosystem (design.md §36)

### 25.4.1 Issue reporting

Three issue templates exist, exactly: bug, ICE, feature. Bug fields: title, output of `plkc --version` (exact command), minimal reproduction of at most 200 lines, expected and actual behavior, target triple. ICE fields: the ICE report path or attachment (Volume 24 §24.5.2), compiler version, whether the ICE reproduces. Feature fields: problem statement, proposed semantics, alternatives considered. The validation bot labels a template missing its reproduction fields as `needs-repro` and closes the issue after 30 days without a response.

### 25.4.2 Bug triage

Every new issue is triaged within 7 days. Severity classes, exactly: S0 miscompilation, soundness violation, or security issue (fix or mitigation within 14 days); S1 wrong diagnostics, hang, or crash (fix within 60 days); S2 quality issue (fix target: next minor release); S3 wishlist (no SLA). Severity is assigned by 2 maintainers or by the area owner of the affected stage. Issue states, exactly: new, triaged, in-progress, fixed, wontfix, duplicate.

### 25.4.3 Security disclosures

Reports go to the published security address. Service levels, exact: acknowledgment within 2 business days; first assessment within 7 days; fix and coordinated release within 90 days. During the embargo the report is restricted to the security team (at least 3 members, §25.4.9) and the reporter. A critical-severity fix receives a CVE request. The advisory is published on the fix day in the format `planck-advisory/1` (Volume 24 §24.4.8). Credits appear in the release notes unless the reporter declines.

### 25.4.4 Regression policy

A regression is a conformance test, benchmark, or documented behavior that passed in release N and fails in N+1 without an edition change. Rules, exact: every detected regression gets an issue within 1 business day; a regression blocks the release train (the train holds at beta, §25.1.4) until the regression is fixed or waived by a written waiver approved by 2 of the 5 council members; a waiver names the impacted behavior and expires at the next stable release.

### 25.4.5 Compatibility testing

Two corpora are tested against every release candidate: (a) the conformance corpus of Volume 22 §29.1; (b) the ecosystem corpus: the 50 most-downloaded registry packages, pinned by lockfile, built without errors (warnings permitted) on each of the 5 host platforms. A release candidate that fails either corpus does not ship (§25.4.10 checklist gate).

### 25.4.6 External library ecosystem

Package name rules are Volume 20's. Reserved top-level names, exactly: std, core, planck, plk, test, bench; these cannot be registered. Yank is permitted only for a security or critical-bugfix reason and requires a registry note naming the reason. Package ownership: the creator plus added maintainers; an ownership transfer requires a public notice 7 days before it takes effect.

### 25.4.7 Package registry operations

The registry is the signed index plus the tarball store of Volume 20 and Volume 24 §24.4.2. Availability target: 99.5% per calendar month, measured by a probe every 5 minutes. Mirrors are permitted but never trusted: every mirror-served byte passes the client-side signature and checksum verification of Volume 24 §24.4.2. Rate limits, exact: 120 requests per minute per IP for the index, 30 requests per minute per IP for tarballs. An outage longer than 1 hour requires a status page update during the outage and a postmortem published within 14 days.

### 25.4.8 Documentation maintenance

Documentation maintenance is the §25.3 policy plus ownership: each registry row has an area owner (§25.4.9) responsible for the co-location law, the CI link check, the example compile check, and the archive rule for removed features. A registry row without an owner is a governance defect tracked as an S2 issue.

### 25.4.9 Compiler maintainer roles

Roles, exact: the language council, 5 members, decides language changes (§25.2.3) and compatibility waivers (§25.4.4); area owners, one per area, for the 10 areas: frontend, ir1, ir2, backend, runtime, stdlib, tooling, infra, release, security; reviewers, who gain write access after 20 merged non-trivial PRs and 2 sponsorships from council or area owners; the release manager, 1 per release train (§25.4.10); the security team, at least 3 members, holding the disclosure process (§25.4.3). Decision rules, exact: a code change requires 1 area-owner approval and green CI; a language change follows §25.2.3; a security-sensitive change additionally requires 1 security-team approval; a release follows §25.4.10.

### 25.4.10 Release ownership

The release manager of a train performs: cutting the release branch, publishing beta builds, running the release checklist, publishing the stable release and release notes. The release checklist has exactly 8 gates, all mandatory: (1) manifest signature verification; (2) per-artifact checksum verification; (3) repro-check equality (OB-24-11); (4) conformance suite green; (5) ecosystem corpus builds (§25.4.5); (6) `plkc audit` clean at threshold moderate; (7) documentation build clean; (8) migration notes present for every deprecation and removal in the release. Release notes format, exact sections in this order: Highlights (factual list), Stabilizations (with RFC numbers), Deprecations (with removal editions), Advisory fixes, Known issues, Spec version.

### 25.4.11 Support policy

Supported versions: the current stable release N and the previous stable release N-1. Each stable release receives fixes for 17 weeks after its release (the 13-week quarterly train plus a 4-week overlap). Beta receives best-effort fixes; nightly receives no support commitment. There is no long-term-support designation in edition P1; this is a declared policy, revisit requires an RFC.

### 25.4.12 Long-term maintenance

When active development stops, the project enters maintenance mode, declared in the repository file MAINTENANCE.md with the date. Maintenance-mode obligations, exact: security fixes for 24 months; the registry stays operational; all RFC and specification history is retained indefinitely; source tarballs and release manifests are mirrored to 2 independent hosts with a monthly integrity re-verification against the signed manifests. Revival of active development requires an RFC stating the revival plan.

## Verification matrix

| ID | Source | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- | --- |
| OB-25-01 | §33 | The version grammar of §25.1.1 accepts exactly the strings of the three channels and rejects all others; domain: channel in {stable, beta, nightly} x component classes {minor, patch, beta-count, nightly-date, build} | EXH over 3 x 5 component classes with boundary values | FV-0 | tools/fv/ob-25-01 harness | G26 |
| OB-25-02 | §33 | The train calendar is consistent: beta.1 = D+14, beta.2 = D+28, stable = D+42, D = first Monday of the quarter month | REVIEW (calendar arithmetic, checked once per train by the release manager) | FV-2 | release checklist gate record | G26 |
| OB-25-03 | §33 | Every release artifact of §25.1.6 (8 files) appears in the signed manifest with path, SHA-256, and size | EXH over the 8 artifact classes | FV-0 | CI job manifest-completeness | G14 |
| OB-25-04 | §33 | Uninstall and install path containment: the canonical toolchain path must be inside <prefix>/toolchains/ | EXH over the path corpus of OB-24-08 | FV-0 | tools/fv/ob-25-04.smt2 | G12 |
| OB-25-05 | §34 | Deprecation window arithmetic: feature deprecated in edition E warns in E and E+1 and is a rejection in E+2; domain: feature state in {active, deprecated, removed} x edition in {P1..P4} (12 cells) | EXH over 12 cells | FV-0 | tools/fv/ob-25-05.smt2 | G26 |
| OB-25-06 | §34 | Every E-REMOVED message contains the migration text, the deprecation edition, and the removal edition | REVIEW of the removal message builder against §25.2.11 | FV-2 | diag corpus entries for E-REMOVED | G22, G26 |
| OB-25-07 | §34 | The RFC state machine is total over states {draft, in-review, accepted, rejected, withdrawn}: all 25 cells defined, each transition names actor and date | EXH over the 25-cell matrix | FV-0 | tools/fv/ob-25-07.smt2 | G26 |
| OB-25-08 | §34 | The compatibility classifier is total and exclusive: every proposal gets exactly one of {additive, bugfix, breaking}, decided by the definitions of §25.2.8 | REVIEW of the classifier decision procedure | FV-2 | RFC decision records | G26 |
| OB-25-09 | §34 | Performance gates are enforced: no >2% runtime and no >5% compile-time regression without an expiring waiver | REVIEW + PROP over the perf-history records (deterministic replay of gate arithmetic) | FV-1 (gap: measurement noise; bridge: fixed benchmark protocol of Volume 23) | CI job perf-gate | G26 |
| OB-25-10 | §34 | The spec version hash is stable: permuting the 27 files changes nothing, changing any file changes h | EXH over all 6 permutations of a 3-file toy set, plus REVIEW of the sorted concatenation for the real set | FV-0 | tools/fv/ob-25-10 harness | G26, G18 |
| OB-25-11 | §35 | The documentation registry is complete: 24 rows, one per §35 bullet, each with path, source of truth, required sections, trigger | EXH over the 24 rows | FV-0 | tools/fv/ob-25-11 harness | G26 |
| OB-25-12 | §35 | Every document passes doc standards.md: no marketing terms, bold restricted, bullets at most 2 levels, examples compile, links resolve | REVIEW of the CI check configuration (doc standards.md §5.2 rules mapped 1:1 to CI failures) | FV-2 | CI job doc-lint | G26 |
| OB-25-13 | §36 | Triage SLA arithmetic: every state transition deadline is computed from the assignment date exactly per §25.4.2 (7 days, 14, 60) | REVIEW (bot arithmetic reviewed once; timestamps are recorded per issue) | FV-2 | triage bot logs | G26 |
| OB-25-14 | §36 | Disclosure SLA: ack within 2 business days, assessment within 7 days, fix within 90 days; advisory published on fix day | REVIEW of the security team process record per advisory | FV-2 | advisory records PLK-* | G26 |
| OB-25-15 | §36 | Support window: fixes accepted for 17 weeks after each stable release; support set is exactly {N, N-1} | REVIEW (calendar arithmetic per release) | FV-2 | release checklist gate record | G26 |
| OB-25-16 | §36 | Registry rate limits are enforced exactly: 120/min/IP index, 30/min/IP tarball | PROP over request traces, paired with REVIEW of the limiter configuration | FV-1 (gap: trace sampling; bridge: limiter is a named-constant configuration) | CI job registry-limiter tests | G26 |
| OB-25-17 | §36 | The ecosystem corpus (50 pinned packages) builds without errors on every release candidate and host | EXH over 50 packages x 5 hosts per release candidate | FV-0 | CI job ecosystem-corpus | G26, G25 |
| OB-25-18 | §34 | Every edition behavioral delta has a migration step: the set of documented deltas equals the set of plkc migrate transformations plus the manual list | REVIEW of the migration document against the edition delta list | FV-2 | migration documents per edition | G26 |

## Coverage ledger

| design.md bullet (abbreviated) | Defined in |
| --- | --- |
| §33 Version numbering | §25.1.1 |
| §33 Release channels | §25.1.2 |
| §33 Stable releases | §25.1.3 |
| §33 Development releases | §25.1.4 |
| §33 Nightly builds | §25.1.5 |
| §33 Compiler distribution | §25.1.6 |
| §33 Binary packaging | §25.1.7 |
| §33 Source packaging | §25.1.8 |
| §33 Cross-platform distribution | §25.1.9 |
| §33 Installation | §25.1.10 |
| §33 Uninstallation | §25.1.11 |
| §33 Upgrade handling | §25.1.12 |
| §33 Rollback | §25.1.13 |
| §33 Package signatures | §25.1.14, Volume 24 §24.4.1-2 |
| §33 Release checksums | §25.1.15, Volume 24 §24.4.5 |
| §33 Reproducible builds | §25.1.16, Volume 24 §24.4.6 |
| §34 Feature proposal process | §25.2.1 |
| §34 Language change proposals | §25.2.2 |
| §34 Design review | §25.2.3 |
| §34 Prototype implementation | §25.2.4 |
| §34 Semantic review | §25.2.5 |
| §34 Performance evaluation | §25.2.6 |
| §34 Safety review | §25.2.7 |
| §34 Compatibility review | §25.2.8 |
| §34 Stabilization | §25.2.9 |
| §34 Deprecation | §25.2.10 |
| §34 Removal | §25.2.11 |
| §34 Edition policy | §25.2.12 |
| §34 RFC archival | §25.2.13 |
| §34 Specification versioning | §25.2.14 |
| §35 Language specification | §25.3.2 row 1 |
| §35 Compiler architecture | §25.3.2 row 2 |
| §35 Type system | §25.3.2 row 3 |
| §35 Borrow checker | §25.3.2 row 4 |
| §35 Memory model | §25.3.2 row 5 |
| §35 Effects | §25.3.2 row 6 |
| §35 Concurrency | §25.3.2 row 7 |
| §35 Async | §25.3.2 row 8 |
| §35 IR 1 | §25.3.2 row 9 |
| §35 IR 2 | §25.3.2 row 10 |
| §35 Machine IR | §25.3.2 row 11 |
| §35 Backend architecture | §25.3.2 row 12 |
| §35 Runtime | §25.3.2 row 13 |
| §35 Standard library | §25.3.2 row 14 |
| §35 FFI | §25.3.2 row 15 |
| §35 ABI | §25.3.2 row 16 |
| §35 Build system | §25.3.2 row 17 |
| §35 Package manager | §25.3.2 row 18 |
| §35 Tooling | §25.3.2 row 19 |
| §35 Debugging | §25.3.2 row 20 |
| §35 Profiling | §25.3.2 row 21 |
| §35 Formal verification | §25.3.2 row 22 |
| §35 Compiler development | §25.3.2 row 23 |
| §35 Contribution guidelines | §25.3.2 row 24, §25.4.9 |
| §36 Issue reporting | §25.4.1 |
| §36 Bug triage | §25.4.2 |
| §36 Security disclosures | §25.4.3 |
| §36 Regression policy | §25.4.4 |
| §36 Compatibility testing | §25.4.5 |
| §36 External library ecosystem | §25.4.6 |
| §36 Package registry operations | §25.4.7 |
| §36 Documentation maintenance | §25.4.8 |
| §36 Compiler maintainer roles | §25.4.9 |
| §36 Release ownership | §25.4.10 |
| §36 Support policy | §25.4.11 |
| §36 Long-term maintenance | §25.4.12 |

Bullets in scope: 66, defined: 66.
