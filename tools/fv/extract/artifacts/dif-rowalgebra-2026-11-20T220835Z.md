# DIF artifact — extracted row functions vs Coq kernel (complete domain)

CEP:FILE: tools/fv/extract/artifacts/dif-rowalgebra-2026-11-20T220835Z.md
CEP:WHAT: DIF differential-equivalence record: planck_effect extracted Rust vs kernel vm_compute ground truth over the complete 2^8 x 2^8 domain (65,536 pairs per function; join, meet, leq), in debug AND release. Ground truth is emitted as generated Rust source under extract/planck_effect/ground_truth/ (DR-8.7).
CEP:WHY: CEP-DR-01 DR-1.5 method DIF; the standing standard requires mechanically extracted code proven equivalent under all workloads of its domain (DR-8.8 v2.9: the complete target domain).
CEP:STATUS: complete
CEP:FAILURE: any mismatch would have failed the run and this artifact would record the failure instead.

STATUS: pass
CEP:ASSUMES: Coq 8.20.1 kernel (oracle); pinned enumerations and encodings shared by both sides (single generator); the reference axis generated mechanically from the reference sources (DR-8.8 v2.15).
CEP:COST: kernel evaluation + cargo test.
CEP:EVIDENCE: the table below; ground truth under extract/planck_effect/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles; pairing counterpart: math/crates/planck-effect-math full-domain EXH (exh_leq_subset_characterization_524288 and the effect-lattice certificates)
CEP:SECURITY: repository-trusted inputs.

## Ground truth and reference tables (sha256)

| file | entries | sha256 |
| ---- | ------- | ------ |
| row_join_table.rs | 65536 | 9e512498f21104e0900c82f44a6e9998db5b16af3bb3221d7dfca033b7f707cc |
| row_meet_table.rs | 65536 | 48fdae062cd30ef447308a3fd8e9fe65400bd49a690468860df35fa037ee0300 |
| row_leq_table.rs | 65536 | 5d359e14c62e7c1818403c80afe7ed5f5fe8cb6a502ca470dfa7cc26381596a4 |

## Result

- the table below; ground truth under extract/planck_effect/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles; pairing counterpart: math/crates/planck-effect-math full-domain EXH (exh_leq_subset_characterization_524288 and the effect-lattice certificates)
- cargo test debug: PASS
- cargo test release: PASS
- verdict: DIF PASS — function-level equivalence over the complete pinned domain(s)
