# Planck trust boundary — code provenance layers

CEP:FILE: spec/TRUST-BOUNDARY.md
CEP:WHAT: The binding declaration of which repository layers may contain executable compiler implementation, which layers are proofs, which layers are untrusted reference material, and which layers are tooling — the audit record for the standing standard ("do not write the code yourself; it is supposed to be mechanically extracted, and separated") and for the formal extraction-only rule (rocq/Effect/CEP/CEPStandard.v, lemmas hand_written_implementation_is_non_conforming and publication_gates_are_sequential).
CEP:WHY: The separation demand is only real if each directory's provenance role is declared and mechanically enforced; without this record a hand-written file could silently masquerade as implementation. The declaration does not delete or demote any recorded artifact: every layer keeps its recorded verification status; what changes is the SHIPMENT claim (what may execute as compiler implementation).
CEP:STATUS: complete
CEP:FAILURE: An executable artifact that claims to be Planck compiler implementation while living outside extract/ is non-conforming and must be either regenerated under extract/ (the only repair) or reclassified by an amendment to this record. A drift detected by planck-extractor check is a violation of layer I's ban, not an exception to it.
CEP:ASSUMES: CEP-DR-01 §4 (layered TCB) and §8 (extraction discipline DR-8.1..8.7); SPEC-INDEX §4 (methods) and §5.1 (equivalence-over-the-domain claim); standards.md CEP&CC 0.4.
CEP:COST: document; enforcement cost is the existing pipeline (build.sh, lint-cep, check, dif).
CEP:EVIDENCE: extract/planck_effect/src/lib.rs + ground_truth/*_table.rs (generated, CEP:PROVENANCE headers); tools/fv/extract/artifacts/ (DIF records with sha256 tables); spec/VOTE-RECORD.md (vote rounds 1-5); rocq/Effect/CEP/CEPStandard.v (the formal rules).
CEP:SECURITY: repository-trusted inputs; no secrets.

---

## Layer I — implementation (the ONLY shippable compiler code)

`extract/`

- 100% mechanically generated from kernel-checked Rocq sources by
  `tools/extractor` (planck-extractor; FV-2): `src/lib.rs`, `tests/`, and
  `ground_truth/*_table.rs` are generated and every one of them carries a
  generated `CEP:PROVENANCE` header naming the source .v, its sha256, and
  the extractor; `Cargo.toml` is extractor-emitted; `Cargo.lock` is a cargo
  build output; `rust-toolchain.toml` is the deployment-layer pin (CEP
  block, no source .v).
- Hand edits to generated files are non-conforming (CEP-DR-01 DR-8.2).
  Enforcement: `planck-extractor check` (regenerate + diff for
  kernel-independent files; header/boundary/declared-and-actual-count
  guards for the oracle tables) and the DR-8.7 guard, which rejects
  plain-text ground truth under `ground_truth/` (the guard's scope).
- Qualification: complete-domain DIF against the Coq kernel
  (`tools/extractor dif`; records under `tools/fv/extract/artifacts/`).

## Layer II — proofs (no executable implementation content)

`rocq/`

- Kernel-checked constructive proofs (Coq 8.20.1, pinned; `build.sh` +
  `coqchk`, zero axioms). Current files: `Effect/RowAlgebra.v` (foundation),
  `Effect/DifGroundTruth.v` (generated kernel oracle),
  `Effect/CEP/CEPStandard.v` (the CEP standard itself, formalized).
- Proofs never enter extracted code (Prop erasure is structural; formalized
  in CEPStandard.v section 8).

## Layer III — untrusted reference kernels (verification scaffolding; never compiler implementation)

`math/`

- Hand-written Rust kernels, exhaustively or bounded verified (EXH/Z3/MATP;
  94 tests green under the pinned rustc at round 3). They remain:
  (a) the DIF pairing counterpart until OB-DR-5 gives G18 its independent
  checker, and (b) the executable reference of the SPEC-INDEX §5.1 claim.
- BY THIS RECORD they are reclassified for shipment: `math/` is reference
  and verification scaffolding, NEVER linked into a shipped compiler
  artifact, and is superseded per-domain by the extracted equivalents as
  DO-1..DO-3 progress (effect domain: already superseded by
  `extract/planck_effect`).
- This reclassification deletes nothing: every verification claim recorded
  for `math/` stands exactly as recorded in SPEC-INDEX and VOTE-RECORD.

## Layer IV — tooling (build-time instruments, not shipped)

`tools/` — planck-extractor (FV-2), MATP (pinned 677bea9), Z3, fv drivers.
Tools verify; they do not execute as compiler implementation.

## Layer V — deployment

`rustc` pinned by the `rust-toolchain.toml` files (1.100.0); assumption set
is OB-DR-4's obligation.

## Enforcement chain (mechanical, in order)

1. `bash rocq/build.sh` — pin assertion + kernel check + coqchk (zero axioms).
2. `planck-extractor lint-cep` — CEP + FV field sets and banned tokens on
   every `rocq/**/*.v`.
3. `planck-extractor extract` — regenerate the extractor-emitted
   implementation deliverables (one of the two legal write paths into
   `extract/`; the other is `dif`'s oracle-table writes, DR-8.7).
4. `planck-extractor check` — hand-edit ban + DR-8.7 guards.
5. `planck-extractor dif` — complete-domain DIF + both-profile cargo tests +
   dated sha256 artifact.
