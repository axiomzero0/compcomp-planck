# DIF artifact — extracted row functions vs Coq kernel (complete domain)

CEP:FILE: tools/fv/extract/artifacts/dif-rowalgebra-2026-11-19.md
CEP:WHAT: DIF differential-equivalence record: planck_effect extracted Rust vs kernel vm_compute ground truth over the complete 2^8 x 2^8 domain (65,536 pairs per function; join, meet, leq), in debug AND release.
CEP:WHY: CEP-DR-01 DR-1.5 method DIF; the standing standard requires mechanically extracted code proven equivalent under all workloads of its domain.
CEP:STATUS: complete
CEP:FAILURE: any mismatch would have failed the run and this artifact would record the failure instead.

STATUS: pass
CEP:ASSUMES: Coq 8.20.1 kernel (oracle); pinned index encoding; enumeration identical on both sides (single generator).
CEP:COST: kernel evaluation + cargo test.
CEP:EVIDENCE: the table below; ground truth files under extract/planck_effect/ground_truth/ (sha256 recorded); cargo test exit 0 in both profiles.
CEP:SECURITY: repository-trusted inputs.

## Ground truth (kernel vm_compute, Coq 8.20.1)

| file | entries | sha256 |
| ---- | ------- | ------ |
| row_algebra_join.txt | 65536 | e520fbe357ea962d3010f093a93fdc20d53e9e9405d3b7eb195dc1c7ec727be5 |
| row_algebra_meet.txt | 65536 | c3edc50d9678d98483ccecf078e9f6a6de25a2c9bede340d61979a103299ac2a |
| row_algebra_leq.txt | 65536 | 7645517b542c68da2087e0f2619c81f85b104c6d61ddd8565bb675bce7ef3988 |

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
