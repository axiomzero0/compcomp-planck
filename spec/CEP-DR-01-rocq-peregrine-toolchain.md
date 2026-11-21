# CEP-DR-01 — Planck verification and extraction toolchain: Rocq + Peregrine + MATP

CEP:FILE: spec/CEP-DR-01-rocq-peregrine-toolchain.md
CEP:WHAT: Decision record that settles the proof-assistant, extraction, and prover toolchain for the Planck compiler (PLK): Rocq is the primary proof assistant, Peregrine is the first extraction route to Rust, MATP is retained as the complementary first-order prover; it also pins the four extractor-qualification obligations that must be discharged before any extraction output is trusted.
CEP:WHY: SPEC-INDEX section 3 pinned a TCB of math kernel + Z3 scripts + MATP, but did not settle a constructive proof foundation or a proof-to-Rust extraction route; first-order superposition (MATP) proves properties but yields no computational content to extract, so an extraction-capable foundation is required by design.md's verified-compiler goal (G1, G16, G25).
CEP:STATUS: complete
CEP:FAILURE: An extraction pipeline built before OB-DR-1..4 are discharged, or a volume that claims Peregrine-Rust output is verified, is non-conforming.
CEP:ASSUMES: standards.md (CEP&CC 0.4); verification standards.md G1-G26; doc standards.md; design.md (roadmap, unchanged); SPEC-INDEX.md section 3 decisions; grounded facts about Peregrine from https://peregrine-project.github.io/ (checked 2026-11-19) and Rocq from https://rocq-prover.org/.
CEP:COST: document; downstream cost is the OB-DR obligation set (section 5).
CEP:EVIDENCE: this record; SPEC-INDEX section 3 row "Compiler verification target" (updated by this record); the Peregrine site facts quoted in section 3.
CEP:SECURITY: no secrets; repository-trusted inputs.

---

## 1. Decision (normative)

The Planck toolchain is:

| Responsibility           | Technology                  | Status    |
| ------------------------ | --------------------------- | --------- |
| Authoritative semantics  | Planck formal specification (`spec/` volumes) | adopted (exists) |
| Constructive proofs      | Rocq                        | adopted (foundation work begins this pass) |
| Algebraic verification   | MATP (github.com/compilecomp/MATP) | adopted (retained) |
| Proof-to-Rust extraction | Peregrine                   | adopted as first route to investigate |
| Independent checking     | Small trusted proof checker | adopted (design begins Volume 22 revision) |
| Optimization             | Planck verified optimizer   | adopted (Volumes 14-15, 23) |
| Final compiler           | Self-hosted Planck          | adopted (G25 chain) |

Rules:

1. DR-1.1: The authoritative semantics of Planck are the `spec/` volumes. Rocq, MATP, Z3, EXH, and PROP are verification engines over those definitions; none of them redefines semantics.
2. DR-1.2: Rocq is the primary proof assistant. Constructive proofs of semantic preservation (G1), type soundness (G3), and pass correctness (G4, G5) are Rocq obligations. Rocq artifacts live under `rocq/` (pinned Rocq version, section 6).
3. DR-1.3: MATP is the complementary prover for first-order algebraic obligations where its theories apply (effect lattice laws, operator-table properties, relocation overflow bounds). It remains pinned at revision 677bea9 as recorded in `tools/fv/matp/artifacts/`.
4. DR-1.4: Peregrine is the first extraction route to Rust. No Peregrine output is trusted until OB-DR-1 through OB-DR-4 (section 5) are discharged.
5. DR-1.5: Z3 (bounded SMT), EXH (exhaustive finite-domain proof), PROP (paired property testing), and REVIEW remain the obligation methods of SPEC-INDEX section 4. Rocq adds a fifth method value `ROCQ` for constructive proofs; a `ROCQ` row must name the exact Rocq version, the file under `rocq/`, and the compiled `.vo` check command. `DIF` (differential equivalence against the `math/` kernels over a stated domain, paired per the pairing rule) is added as a sixth method value for untrusted extraction output; both additions are mirrored in SPEC-INDEX section 4 item 2.
6. DR-1.6: A small trusted proof checker is adopted as the independent-checking instrument for G18. Its design obligation is OB-DR-5; until it exists, G18 independence is provided by dual-engine discharge (Rocq + MATP/Z3) on the same obligation.

## 2. Why not Lean alone, why not MATP alone (recorded rationale, non-normative)

- Lean's Rust extraction ecosystem is less mature than the Rocq-side route; the binding requirement is proof-to-Rust extraction, not general mathematical convenience.
- MATP's superposition engine proves first-order properties but does not inherently provide constructive computational content; extraction from it is not defined. MATP therefore stays in its role from SPEC-INDEX section 4: search engine and discharge engine for FO obligations.

## 3. Grounded facts about Peregrine (as of 2026-11-19, from https://peregrine-project.github.io/)

These facts constrain what OB-DR-1..4 must verify. Nothing here is assumed beyond the site.

1. PF-1: Peregrine is a unified middle-end for code generation from proof assistants, based on an intermediate language called lambda box.
2. PF-2: Frontend support: Agda, Lean, Rocq (via MetaRocq and ConCert).
3. PF-3: Output backends: CakeML; C (via CertiCoq, then x86/ARM/RISC-V via CompCert); Rust (via ConCert); OCaml (verified); Wasm (verified, CertiRocq-wasm).
4. PF-4: The middle end is verified in Rocq; only the OCaml and Wasm backends are marked verified on the site. The Rust (ConCert) backend is NOT marked verified.
5. PF-5: Consequence: the Rust output route is the least-verified leg of the adopted chain. This is exactly the gap the qualification obligations below close or bound.

## 4. Revised trusted computing base (updates SPEC-INDEX section 3, row "Compiler verification target")

The TCB is layered:

1. Proof layer (trusted): the Rocq kernel (pinned version, section 6); the math kernel `math/` (FV-checked per section 54 of standards.md); Z3 scripts in `tools/fv/`; MATP pinned at 677bea9; the review process.
2. Extraction layer (NOT trusted yet): Peregrine lambda-box middle end and ConCert Rust backend. Trust is conditional on OB-DR-1..4. Until then, any Peregrine output is treated as an unverified artifact and must pass differential equivalence checking against the hand-written `math/` kernels (method DIF: differential test + certificate comparison, paired with EXH or Z3 per the SPEC-INDEX section 4 pairing rule).
3. Deployment layer (trusted for execution, outside the proof for now): rustc and its backend. OB-DR-4 bounds what must be assumed of them.

`math/` remains the executable reference: hand-written Rust kernels that are exhaustively or bounded verified, and against which any extracted Rust is differentially checked. This preserves the standing equivalence claim of SPEC-INDEX section 5.1: equivalent over the whole defined input domain, because limits are part of the specification (G13, G14).

## 5. Qualification obligations (the user's four conditions plus the checker-design obligation, normative)

These are the conditions of adoption stated as obligations with verification rows. They are added to the Volume 22 verification matrix.

| ID | Obligation (exact) | Method | G-goals | Status |
| -- | ------------------ | ------ | ------- | ------ |
| OB-DR-1 | Verify the extractor: for the pinned Peregrine revision, prove or bound the correctness of the lambda-box middle-end transformations used by the Rust route, or document each unproven transformation as a named refinement gap with a runtime bridge (section 54.2.2 of standards.md). | ROCQ + REVIEW | G1, G16, G18 | open |
| OB-DR-2 | Verify the supported language subset: enumerate the Planck 0.1 surface constructs that the MetaRocq/ConCert frontend accepts; every Planck construct outside that subset is a named refinement gap with the defined fallback (hand-written `math/` kernel). Produce `rocq/subset-report.md` with one row per Volume 02 production. | REVIEW + EXH | G1, G19 | open |
| OB-DR-3 | Verify the generated Rust semantics: for a bounded but stated class of programs, prove that Peregrine-Rust output implements the same judgment as the source semantics, by differential execution against `math/` kernels and by Z3 checks on the generated arithmetic. The bound is stated in the obligation row. | DIF + Z3 + EXH | G1, G5, G18 | open |
| OB-DR-4 | Verify the Rust compiler assumptions: enumerate exactly what is assumed of rustc (name resolution semantics, overflow panics per profile, layout of used primitives, ABI of generated items, no miscompilation within the assumed subset). Each assumption is a row in `rocq/rustc-assumptions.md`; assumptions outside the assumed subset are runtime-bridged (versioned rustc pin + post-build certificate checks). | REVIEW | G16, G20 | open |
| OB-DR-5 | Design the small trusted proof checker: define its input language, its checking algorithm, its termination measure, and its own TCB; it must be implementable in under 2,000 lines of no_std Rust and checkable by EXH over its grammar domain. Design lands in Volume 22 revision. | REVIEW + EXH | G18, G16 | open |

Rule: DR-5.1 — OB-DR-1..4 are sequenced: 2 gates 3 (no subset, no equivalence claim), 1 and 4 gate any claim of "verified extraction". Until all four are discharged, the toolchain table's Peregrine row carries status "extraction candidate, untrusted".

## 6. Pinned versions (initial; bump is an edition-of-record event logged here)

| Component | Pin | Note |
| --------- | --- | ---- |
| Rocq | The Coq Proof Assistant, version 8.20.1, compiled with OCaml 5.3.0 (pinned at the first `rocq/` build, 2026-11-19; the pin is the exact version string printed by `coqc --version` — the DR-1.5-sanctioned `coqc` form of the Rocq kernel) | kernel is TCB; evidence: `rocq/build.sh` stdout + `Effect/RowAlgebra.vo` + coqchk re-verification |
| Peregrine | revision pinned when the feasibility study lands (`rocq/subset-report.md` prerequisite) | PF-1..PF-5 facts are per-site, versionless until pinned |
| MATP | 677bea9 (github.com/compilecomp/MATP) | existing artifacts cite this |
| Z3 | the version string recorded in each `tools/fv/` script header | existing scheme |
| rustc | pinned by `math/rust-toolchain.toml` (created at the first deployment-layer pin; until then the pin is the toolchain that builds `math/`, recorded in the build log) | deployment layer |

## 7. Integration with the existing library

1. DR-7.1: SPEC-INDEX section 3, row "Compiler verification target", is amended by this record (its old text remains true; this record adds the constructive and extraction layers).
2. DR-7.2: Volume 22 (correctness and verification) gains the OB-DR rows above in its verification matrix on its next revision; until that revision, this record is the normative home of those rows.
3. DR-7.3: Nothing in design.md is modified. This record defines the toolchain that design.md's verification goals (G1, G16, G18, G19, G25, G26) are discharged with.
4. DR-7.4: The `rocq/` directory joins the repository layout of SPEC-INDEX section 6: `rocq/` holds the Rocq formalization (pinned version file, proofs, subset report, rustc assumptions).

## 8. Extraction layer execution record (added 2026-11-19; normative from here on)

The Peregrine route (PF-1..PF-5) remains the long-term proof-to-Rust plan, but its Rust leg is the least-verified (PF-4) and cannot be trusted before OB-DR-1..4. Pending that qualification, the standing standard bans hand-written compiler code outright, so the project operates a **mechanical extraction instrument of its own**, qualified by the same discipline the record already prescribes for untrusted extraction output (§4 layer 2: differential equivalence against a kernel oracle).

Rules (normative):

1. DR-8.1: `tools/extractor` (planck-extractor) is the mechanical Gallina-subset → Rust extractor. It is FV-2 tooling (deterministic, kernel-external). Its pinned subset accepts only: top-level Definitions with explicit result types, right-nested tuple types and pair terms, the pinned Bool prelude mapping (orb → x|y, andb → x&y, implb → if x then y else true), and structurally erases every Lemma..Qed block (Prop erasure). Everything else is a hard reject.
2. DR-8.2: every executable artifact derived from a proof file lives under `extract/` (separated from `rocq/` proofs and from the hand-written `math/` reference kernels) and carries a generated CEP:PROVENANCE header naming the source .v, its sha256, and the extractor. Hand edits to `extract/` are non-conforming; drift is detected mechanically by `planck-extractor check` (regenerate + diff) and the ban is enforced before every DIF run.
3. DR-8.3: the DIF oracle is the Coq kernel itself: `rocq/Effect/DifGroundTruth.v` (generated by the same extractor, sharing one pinned enumeration and index encoding with the Rust side) evaluates the complete finite domain with `vm_compute`; the extracted Rust replays the identical domain. Equivalence is claimed only over the complete stated domain, never by sampling.
4. DR-8.4: CEP on the proof layer is mechanically enforced: `planck-extractor lint-cep` requires the complete CEP + FV field set on every `rocq/**/*.v`, a pinned STATUS vocabulary, and zero banned proof tokens (Admitted/admit/Axiom as statements, plus the Parameter/Hypothesis/Conjecture/Variable(s) statement forms of the VR-14(b) sweep); the kernel check additionally requires coqchk to report zero axioms.
5. DR-8.5: a DIF artifact under `tools/fv/extract/artifacts/` records actual outcomes only (VR-4 discipline): ground-truth sha256 table, entry counts, and the cargo test verdicts in debug AND release.
6. DR-8.6: OB-DR-1..4 continue to gate any Peregrine-based claim; DR-8.1..8.5 gate the planck-extractor lane on the same terms. The two lanes share the DIF method and the `extract/` separation; neither may relax the other's gates.
7. DR-8.7 (added 2026-11-20; the "extraction output is real Rust" correction, user standard): every extractor-emitted deliverable under `extract/` is a Rust compilation unit or a cargo manifest — a plain-text data dump (.txt) is non-conforming as extraction output (cargo-emitted lockfiles are build outputs, not extractor deliverables). The DIF kernel oracle tables are emitted as generated Rust source (`ground_truth/*_table.rs`, `pub const` arrays) compiled into the DIF test harness via `include!`; `planck-extractor check` mechanically rejects legacy .txt ground truth, header/trailing-boundary tamper, and wrong declared or actual entry counts, while value-level truth remains the DIF's own job plus the per-run sha256 audit trail in the dated artifact. Source provenance (sha256 of the owning .v) is carried by every generated header. The layer's provenance roles (which trees may hold implementation, proofs, reference, tooling) are declared in `spec/TRUST-BOUNDARY.md`.
8. DR-8.8 (added 2026-11-21; the deliberate subset-extension record demanded by VR-10(b), normative before any integer-domain foundation kernel extracts): the pinned extraction subset is versioned. Subset v1 (DR-8.1) remains pinned for the already-qualified effect lane; the foundation domains of DO-1 phase 1 that need a byte/integer domain extract under **subset v2**, which extends v1 by exactly the following accept-list, item by item, and by nothing else:
   - v2.1: the type `nat` and decimal natural literals (`0`..`255` in practice; the pinned foundation byte domain is `{0..=255}`).
   - v2.2: the comparisons `Nat.eqb`, `Nat.leb`, `Nat.ltb`, written longhand or as the pinned infix notations `=?`, `<=?`, `<?`.
   - v2.3: `if .. then .. else ..` terms.
   - v2.4: **closed inductive enumerations**: `Inductive N : Type := | C1 | C2 | ... .` — no parameters, no recursion (closed finite domains only). The ONE pinned exception to the no-arguments rule is the two-constructor data pair of v2.14 (`Ok : T -> N | Err : U -> N`, exactly the names Ok/Err, exactly two constructors, each carrying one argument type, returning the enum itself — the reference `Result` shape); any other argument-carrying constructor is a hard reject. Recursion outside the fuel shape of v2.7 stays a hard reject.
   - v2.5: `match` terms over closed inductive enumerations, exhaustive over the variant set, no wildcard arm on the closed domain — exhaustiveness is enforced mechanically at parse time (the arm set must cover exactly the scrutinee enum's variant set, verified against the unit's enum declarations; a top-level `_` arm is a hard reject; the `_` binding wildcard inside a constructor pattern, as in `Ok _`, is not a wildcard arm).
   - v2.6: ASCII key literals: bracket-semicolon lists of natural literals `[..; ..]` at the pinned type `key` (= `list nat`), interpreted as ASCII code points; extraction emits the corresponding `&'static str` literal byte-for-byte.
   - v2.7: **bounded fuel recursion** in exactly one shape: `Fixpoint name (fuel : nat) (args ...) : T := match fuel with 0 => <pinned total default> | S f => <body> end` — every recursive call site passes the S-arm binder as the fuel argument (strictly decreasing, kernel-guarded), the number of recursive call sites is bounded by the number of match arms (pinned: at most 2, the standard one-per-branch shape the binary search needs), and the fuel-0 arm is a pinned total default. Extraction emits a plain recursive `fn` with the fuel-0 arm as the base case.
   - v2.8: the v2 extraction mappings (the extension prelude, pinned like the v1 prelude): `nat` → `u8` on the byte-domain foundation kernels; closed `Inductive` → `#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum` with declaration order preserved; `match` → `match`; `if` → `if/else`; `let .. in` → `let` (v2.13); `Nat.eqb` → `==`, `Nat.leb` → `<=`, `Nat.ltb` → `<`, `Nat.add` → `saturating_add`, `Nat.sub` → `saturating_sub`, `Nat.div2` → `/ 2`, `Nat.succ` → `saturating_add(1)`, `Nat.pred` → `saturating_sub(1)` (saturating forms keep the extracted fns total on the pinned domains — no overflow panics), Bool `eqb` → `==`; key literal → `&'static str`; list-of-key literal → array of `&'static str`; list-of-ctor or list-of-definition-reference literal → const array; fuel `Fixpoint` → recursive fn; the pinned-skip names (`row_at`, `key_eq`, `key_lt` and their fuel helpers, v2.14) are mapped at call sites — `row_at e` → const-array indexing into the unit's pinned sorted table, `key_eq a b` → `(a) == (b)`, `key_lt a b` → `(a) < (b)` (byte-lexicographic per core str Ord, the reference's own recorded assumption), each discharged by the DIF over the complete pinned domains (row_at: the closed 44-row dispatch law plus the full sweep; key_eq/key_lt: the 1936-pair tables and the 16,563-key sweep); the pinned vernacular sentence `Unset Automatic Proposition Inductives.` is accepted verbatim (it keeps the singleton lookup_error enum at type level for the Rust mapping).
   - v2.9 (boundedness honesty clause): Coq `nat` is unbounded and Rust `u8` is not; the equivalence claim of every v2 DIF is made over the **complete target domain** — for the byte domain that is all 256 `u8` values, for the operator row domain all 44 rows, for the lookup sweep the pinned 16,563-key domain (the 44 keys + the empty string + all 128 one-byte ASCII strings + all 16,384 two-byte ASCII strings + the 6 delimiters). Every value the extracted target type can hold inside these domains is kernel-differentially checked; no input of the target types goes unverified, and behavior of the Coq definitions on `nat >= 256` is recorded as out-of-target rather than hidden.
   - v2.10: the kernel oracle file `rocq/Effect/DifGroundTruth.v` remains outside the extraction subset (generated, kernel-only, never extracted to Rust); the v2 accept-list governs extraction units (`rocq/**/*.v` other than the generated oracle).
   - v2.11: the parse-time free-identifier rule (VR-10(d)): every identifier occurring in a Definition/Fixpoint body or a match arm must resolve, at parse time, to a binder of that definition (including a let-binding scoping over its body and the S-arm binder scoping over its arm), a definition of the same unit (forward references allowed — the kernel check and the DIF own the semantic validity of order), an enum constructor declared in the same unit or in a pinned required unit, a name of the pinned v1/v2 preludes, or a pinned-skip mapped name (v2.14) — anything else is a hard reject before extraction, not a late cargo failure. Pinned-skip bodies are structurally skipped and never identifier-checked (their semantics are the kernel-checked proof, the mappings, and the DIF).
   - v2.12: the `Module PinnedEnums. ... End PinnedEnums.` name-hygiene wrapper plus the pinned `Import PinnedEnums.`: a Coq scoping device that keeps every enum variant name fresh inside the module and flattens it for use through the Import, so no variant collides with whatever prelude constructors share its short name (several do — e.g. None/Eq/Lt/Gt in the bare prelude); the extractor accepts exactly this one wrapper, emits the enums at crate root, and emits nothing for the Module/End/Import sentences.
   - v2.13: `let x := t1 in t2` terms → Rust `let x = t1; t2` (the binary-search midpoint binding).
   - v2.14: the pinned data-enum mapping and the pinned-skip discipline: an `Inductive` of exactly the Ok/Err shape (v2.4's exception) maps to `pub type <name> = Result<T, E>` with `Ok`/`Err` constructor applications emitted as Rust `Ok(..)`/`Err(..)`; the mapped names `key_eq`, `key_lt`, `row_at` (and their fuel helpers `key_eq_aux`, `key_lt_aux`) are PINNED-SKIP names — their proof-side bodies are kernel-checked by coqc but never emitted and never identifier-checked; their Rust behavior is the v2.8 call-site mappings, discharged by the DIF over the complete pinned domains.
   - v2.15: the transcription record: the operator-table data unit `rocq/Effect/OperatorTableData.v` is GENERATED mechanically from the reference kernel `math/crates/planck-operator-math/src/lib.rs` (EXH-certified to Volume 02 §2.4.1) by `planck-extractor transcribe`, byte-identical to the committed bootstrap generator `tools/extractor/bootstrap/gen_operator_data.py` (the cross-check instrument); the generator asserts 44 rows, strictly ascending byte-lexicographic token order, and key uniqueness at generation time; regeneration is the only edit path (DR-8.2 applies).
   The extension is recorded here BEFORE the first integer-domain extraction (VR-10(b)'s own wording); the v2 fixtures join the extractor's EXH inventory, and the first v2 extraction (charclass + operator-table domains, DO-1 phase 1, VR-14(d)) is qualified by the complete-domain DIF obligations of v2.9 with honest outcome recording (VR-4/DR-8.5).
