# DIF artifact — extracted row functions vs Coq kernel (complete domain)

CEP:FILE: tools/fv/extract/artifacts/dif-rowalgebra-2026-11-20.md
CEP:WHAT: DIF differential-equivalence record: planck_effect extracted Rust vs kernel vm_compute ground truth over the complete 2^8 x 2^8 domain (65,536 pairs per function; join, meet, leq), in debug AND release. Ground truth is emitted as generated Rust source under extract/planck_effect/ground_truth/ (DR-8.7: extraction deliverables are Rust compilation units, never plain-text dumps).
CEP:WHY: CEP-DR-01 DR-1.5 method DIF; the standing standard requires mechanically extracted code proven equivalent under all workloads of its domain.
CEP:STATUS: complete
CEP:FAILURE: any mismatch would have failed the run and this artifact would record the failure instead.

STATUS: pass
CEP:ASSUMES: Coq 8.20.1 kernel (oracle); pinned index encoding; enumeration identical on both sides (single generator).
CEP:COST: kernel evaluation + cargo test.
CEP:EVIDENCE: the table below; ground truth under extract/planck_effect/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles.
CEP:SECURITY: repository-trusted inputs.

## Ground truth (kernel vm_compute, Coq 8.20.1)

| file | entries | sha256 |
| ---- | ------- | ------ |
| row_join_table.rs | 65536 | 40dc8bae7126bc56a1afb8c159975d8da05374de28bba09f08aa25bf03c52416 |
| row_meet_table.rs | 65536 | 3a1da17a0cf182ea96d8fe966124c941f2b7b8d4162a78db0d4346cda8b5cf93 |
| row_leq_table.rs | 65536 | 982418f7ebba8f39c6c40e8096949902993cd948e8e2232e4dadc9a1412f1808 |

## Result

- domain: 2^8 x 2^8 = 65,536 pairs per function (COMPLETE domain; no sampling)
- functions: row_join, row_meet, row_leq (extracted from rocq/Effect/RowAlgebra.v)
- pairing counterpart (SPEC-INDEX §4 pairing rule): the same pinned laws are
  discharged by full-domain EXH in math/crates/planck-effect-math
  (exh_leq_subset_characterization_524288 and the effect-lattice certificates) —
  the two engines are the dual-engine discharge of G18 until OB-DR-5 lands
- cargo test debug: PASS
- cargo test release: PASS
- verdict: DIF PASS — function-level equivalence over the complete finite domain
