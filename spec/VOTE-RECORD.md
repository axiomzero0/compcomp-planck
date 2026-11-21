# Planck Master Specification Library — Five-Agent Vote Record

CEP:FILE: spec/VOTE-RECORD.md
CEP:WHAT: The binding record of the five-agent vote on the completeness and optimality of the Planck definition pass (scope: the 26 spec volumes + SPEC-INDEX + CEP-DR-01 + the math kernel crates + FV artifacts + the Rocq scaffold), per the standing requirement that five agents all vote the definitions complete and optimal; three rounds were run (round 1: 1/5 pass, 4 defect reports; round 2: 3/5; round 3: 5/5 unanimous).
CEP:WHY: SPEC-INDEX §8 (review status) requires this record after the vote completes; the standing instruction ("you have to have 5 agents all vote that it is complete and optimal; do not get rid of anything because it is too complicated, or out of scope") makes 5/5 the acceptance bar, and "YOU ARE HELD TO STANDARDS,, BY THE LETTER" makes CEP&CC 0.4 the lens of the conformance agent.
CEP:STATUS: complete
CEP:FAILURE: A future revision that contradicts a recorded resolution here without updating this record is non-conforming; the record is the completeness-and-optimality acceptance instrument for the definition pass.
CEP:ASSUMES: standards.md (CEP&CC 0.4); verification standards.md G1–G26; doc standards.md; design.md (byte-retained, verified by DR-7.3 check); SPEC-INDEX §0–§8; spec/CEP-DR-01-rocq-peregrine-toolchain.md; the vote agents review only and run builds/tests where mandated.
CEP:COST: document; the vote cost 13 agent reviews across 3 rounds (5 + 4 + 2 re-votes) plus the fix commits listed below.
CEP:EVIDENCE: the per-round tables below; fix commits 40cd30b (definition pass), fb4f042 (kernel extraction), b58ac40 (Rocq scaffold), 950d4e6 (round-1 fixes), fb02879 + b8fbaaf (round-2 fixes); test logs 94/94 release and debug at b8fbaaf.
CEP:SECURITY: repository-trusted inputs.

---

## 1. Vote lenses (five distinct mandates, per the standing requirement)

| Agent | Lens | Mandate core |
| ----- | ---- | ------------ |
| V1 | Completeness | every design.md bullet has exactly one normative definition home; coverage ledgers hold; exactness sweep (no banned-phrase governing normative content) |
| V2 | Standards conformance | CEP&CC by the letter: CEP block field sets on every public item, complete error enums with variant lists in CEP:FAILURE, no_std law, placeholder scan, doc-standards structure |
| V3 | Mathematical soundness | kernels implement the pinned contracts exactly (effect lattice, ownership machine, operator table, graph algorithms, regalloc, relocation ranges); certificates genuinely enumerate their stated domains |
| V4 | Toolchain integration & extraction feasibility | CEP-DR-01 faithfully encodes the Rocq + Peregrine + MATP decision and the four qualification points as OB-DR-1..4; layered TCB coherent; no contradiction across the library; Peregrine facts site-grounded |
| V5 | Verification integrity | artifacts support their claims: tests pass in both profiles, EXH domain arithmetic matches loops, MATP/Z3 artifacts consistent with scripts, no phantom evidence, gap discipline holds |

## 2. Round 1 (initial vote)

| Agent | Verdict | Blocking findings (summary) |
| ----- | ------- | --------------------------- |
| V1 | DEFECT | design.md §41 (dependency order) and §42 (11 completion gates) had no definition home (SPEC-INDEX §2 mapped only §1–§39); 1 exactness hit (volume-16 EM10 "as needed") |
| V2 | DEFECT | graph-math 0/17 and reloc-math 0/40 public items carried CEP blocks; GraphError/RelocError variant lists not in CEP:FAILURE; volume-06 §6.12 cited nonexistent artifacts; doc-standards §3 structure correspondence unstated |
| V3 | DEFECT | **planck-reloc-math ArmBranch26 imm26 placed at bits [30:5] — impossible for the A64 B/BL encoding (opcode [31:26], imm26 [25:0]); overwrote opcode bits**; Z3 script used closed-interval semantics and asserted a First-Fit model the kernel does not produce; graph evidence line named a nonexistent test |
| V4 | COMPLETE_AND_OPTIMAL | none blocking (7 non-blocking observations: RowAlgebra.v missing `Require Import Bool`; SPEC-INDEX §4/§6 sync; volume-22 TCB enumeration stale vs amended §3 + MAPT typo; rust-toolchain.toml dangling ref; cross-ref imprecision; DIF method formalization; bit-order sentence) |
| V5 | DEFECT | MATP artifacts: 10/14 STATUS fail (6 honestly mirrored, 4 ownership fails undisclosed outside tools/fv/); driver evidence line false ("one per obligation, STATUS: pass"); stale orphan artifact; regalloc Z3 claim overstated; graph phantom test name; charclass mask-test domain misstated (128 vs 256); Z3 timeout documented not enforced; volume-16 OB-16-5 stale artifact path |

Round-1 score: **1/5 COMPLETE_AND_OPTIMAL.** All defects fixed in commit 950d4e6.

## 3. Round 2 (re-vote of the four defect lenses)

| Agent | Verdict | Notes |
| ----- | ------- | ----- |
| V1 | COMPLETE_AND_OPTIMAL | both §41/§42 homes verified 11/11 verbatim, DO-1..DO-3 + CG-1/CG-2 exact; exactness sweep clean (0 governing hits); ledgers unchanged |
| V2 | DEFECT | the round-1 fix converter had silently DELETED fn/type doc blocks instead of converting them (graph 5/17, reloc 22/40 — consts only) and truncated multi-line summaries; variant docs cut; artifact-existence and variant-enumeration checks from round 1 all passed |
| V3 | COMPLETE_AND_OPTIMAL | BRANCH_IMM_AT=0 verified against the A64 encoding; half-open interference re-derived independently (6 pairs, exact); kernel probe confirmed the Z3 part-2 model; 94/94 both profiles; census arithmetic re-verified (4,165 + 1,024 graphs; 30,240 lattice; 698,720 instances); round-1 findings 2/3 fixed; observation adopted: placement-sensitive hand-word test recommended |
| V5 | DEFECT | two residuals: regalloc crate-level CEP:EVIDENCE still carried the "8 intervals symbolic" parenthetical (line 8; the FV-REFINEMENT and doc comments were fixed but the header line was missed); volume-16 OB-16-5 Method column still the uninstantiated envelope; exh_-name sweep otherwise clean (0 phantom) |

Round-2 score: **3/5.** Residual fixes in commits fb02879 and b8fbaaf (full doc-block reconstruction: 17/17 and 40/40 real CEP coverage; variant docs restored; junk tails removed; exh_branch26_hand_words added, tests 93→94; regalloc EVIDENCE tail and OB-16-5 Method column right-sized; volume-06 duplicated phrase fixed).

## 4. Round 3 (re-vote of the two residual lenses)

| Agent | Verdict | Notes |
| ----- | ------- | ----- |
| V2 | COMPLETE_AND_OPTIMAL | 17/17 and 40/40 verified programmatically (102 + 240 field lines, full 6-field sets); 0 truncation, 0 junk tails; scans clean |
| V5 | COMPLETE_AND_OPTIMAL | residuals (a)/(b) fixed and verified against the actual script; 94/94 both profiles; Z3 sat/sat/unsat reproduced with hand-re-derived kernel model; 0 phantom evidence across all 8 crates; MATP set 1:1 with driver obligations, no regression |

Round-3 score: **5/5 COMPLETE_AND_OPTIMAL — the vote passes unanimously.**

## 5. Resolutions and standing decisions recorded by the vote

1. VR-1: design.md §41 is defined by SPEC-INDEX §7.1 (DO-1..DO-3); design.md §42 by SPEC-INDEX §7.2 (11-gate table with owner volumes, CG-1..CG-2). The DO-2 per-ledger phase annotation is binding on each volume's next revision (round-2 V1 residual, kept open deliberately — a one-annotation pass per ledger, not a definition change).
2. VR-2: ArmBranch26's imm26 field is bits [25:0] of the A64 B/BL word (opcode [31:26] = 000101/100101), pinned in `planck-reloc-math` (BRANCH_IMM_AT = 0) and cited through R7 → R_AARCH64_CALL26; the placement-sensitive certificate is exh_branch26_hand_words (round-2 V3 observation adopted as a test).
3. VR-3: the Z3 regalloc artifact's instance is the concrete bounded one (5 intervals, 4 registers, half-open interference {0-2, 1-2, 1-3, 1-4, 2-3, 3-4}, coalescible {0,1} both directions) and its part-2 model is the kernel's actual First-Fit output, verified by direct probe; the OB-16-5 Method column carries exactly that envelope.
4. VR-4: MATP artifacts record actual outcomes (never rewritten to pass); obligations that MATP could not close are discharged by full-domain EXH per SPEC-INDEX §4, with the attempt disclosure carried in the crate that owns the obligation (effect-math mirrors the 6 GaveUp records; ownership-math discloses the 3 Saturated-no-refutation records).
5. VR-5: the doc-standards §3 structure correspondence is stated in SPEC-INDEX §1 rule 8 (Abstract ≙ CEP:WHAT + overview; Formal Definitions ≙ d-bullets + kernel-extraction Rust signatures; Algorithm ≙ algorithm subsections; Invariants ≙ invariant lists; Integration Points ≙ cross-volume blocks; Known Limitations ≙ stated-limitations rows + refinement gaps); doc standards.md itself is byte-identical.
6. VR-6: the item-level CEP:WHY calibration follows the exemplar's own precedent (planck-charclass omits WHY on some items); all 57 graph/reloc public items carry the full 6-field set.
7. VR-7 (non-blocking, kept on record for the next revision cycle): (a) three passing EXH tests are not cited in any CEP:EVIDENCE token (charclass exh_comment_introducer_byte_is_0x2f; operator-math exh_column_domains_and_accessor_agreement, exh_lookuperror_is_complete_single_variant) — cite on next touch; (b) the spec library's verification matrices reference ~200 planned tools/fv/ artifact paths that do not exist yet — planned-artifact paths, to be marked as such or implemented per the volume's own matrix; (c) volume-22 §22.13 TCB enumeration is revised at its next revision per DR-7.2 (amendment pointer already in place); (d) DO-2 ledger annotations per VR-1.

## 6. Final state at acceptance

- Definition pass: 26/26 volumes + SPEC-INDEX + CEP-DR-01 + VOTE-RECORD (this file); design.md byte-retained.
- Math extraction: 8/8 kernel crates, 94 tests green in release AND debug (overflow-checks on); certificates: EXH domains verified to the case by the soundness agent (4,165 + 1,024 graphs; 917,532 ownership cells; 698,720 + 26,222 regalloc instances; 30,240 reloc lattice cases; 16,563 lookup probes; 256-row effect lattice; 524,288 leq-characterization points; 66,430 unification cases; 981,640 token cases).
- FV artifacts: 13 MATP results (4 pass, 9 honest fail-with-disclosure, all pinned at MATP 677bea9) + 1 Z3 result (pass, Z3 5.4.1, sat/sat/unsat) + 1 Rust placement test.
- Rocq scaffold: Effect/RowAlgebra.v (pinned laws proven by exhaustive 256-row case analysis, no Admitted/Axiom; kernel check pending first pinned build per CEP-DR-01 §6 policy).
- Toolchain: Rocq (foundation, bootstrapping) + Peregrine (first extraction route, untrusted until OB-DR-1..4) + MATP (complementary FO prover, pinned) — the user's decision, integrated and gated.

## 7. Round 4 (mechanical extraction pass vote, 2026-11-19)

Scope: the mechanical extraction pass — rewritten and kernel-checked `rocq/Effect/RowAlgebra.v` (first pinned build, Coq 8.20.1, coqchk zero axioms), the `tools/extractor` extraction instrument (FV-2), the separated generated tree `extract/planck_effect` with kernel ground truth, the DIF artifact `tools/fv/extract/artifacts/`, CEP-DR-01 §6 pin + §8 extraction-layer rules, and SPEC-INDEX §4/§6 amendments. Five lenses, five independent agent reviews, each reproducing the evidence chain itself.

| Agent | Verdict | Notes |
| ----- | ------- | ----- |
| V1 | COMPLETE_AND_OPTIMAL | §8 defines homes for all six new mechanisms; CEP coverage verified on every new file; exactness sweep clean; dif/check/lint/build all reproduced |
| V2 | COMPLETE_AND_OPTIMAL | complete 9-field CEP + 6-field FV sets on all new artifacts; pin string byte-matches build.sh stdout; no_std judgment: §5.2 scopes no_std to math/ only, extract/ references no std items anyway |
| V3 | COMPLETE_AND_OPTIMAL | all 11 pinned law lemmas verified constructive (34 lemmas / 34 Qed / 0 admits); independent Python re-derivation matched the kernel oracle and ground truth 196,608/196,608; sha256 chain verified; shared-pinned-convention argument closed (generator emits both sides from one parsed width) |
| V4 | COMPLETE_AND_OPTIMAL | DR-8 coherent with DR-1..7 and OB-DR gates; Peregrine stays untrusted and separate; extractor subset honestly pinned; effect-math EXH 37/37 as the pairing counterpart |
| V5 | COMPLETE_AND_OPTIMAL | every claim reproduced; hand-edit ban VALIDATED by live drift probe (append comment → check fails exit 1 → byte-identical restore → check passes); no phantom evidence; git status matches the claim set exactly |

Round-4 score: **5/5 COMPLETE_AND_OPTIMAL — the vote passes unanimously.**

Resolutions and standing decisions recorded by this round:

1. VR-8: the "kernel check pending" state of §6 is RESOLVED: the first pinned build is Coq 8.20.1 (CEP-DR-01 §6), kernel check + coqchk green, and CEP-DR-01 §8 records the extraction-layer execution discipline (DR-8.1..8.6).
2. VR-9: the five lenses' non-blocking observations were adopted on the spot and fixed before the acceptance commit: literal `CEP:PROVENANCE:` field on every generated artifact (DR-8.2's own wording), the `2^{w}` template slip, build.sh now compiles EVERY .v under rocq/ (foundation file first) with a loud pin-drift assertion (exit 3), artifact STATUS form aligned to the z3/matp precedent (CEP:STATUS complete + body STATUS: pass), EXH pairing cross-cite added to the DIF artifact, artifact date derived from the run clock (no same-date overwrites), rocq/README.md §2 staleness fixed, .gitignore for regenerable build outputs, and rust-toolchain.toml pins added to tools/extractor and extract/planck_effect. The logical prefix is pinned as `-Q Effect Effect` (uniform Require path and coqchk module name `Effect.RowAlgebra`).
3. VR-10 (kept open for the next revision cycle, non-blocking): (a) DIF coverage names row_join/row_meet/row_leq; the eight accessors rest on extractor correctness (FV-2) and are implicitly covered by the pointwise composition of the pinned functions — enumerate them explicitly if the extractor's role grows; (b) the pinned Gallina subset is boolean-algebra-only and states no extension procedure — a deliberate subset-extension record is required before integer-domain foundation kernels can extract; (c) CEP-LINT-STATUS vocabularies (cep.rs vs standards.md) should unify at the next standards touch; (d) unknown prelude identifiers fail late (cargo) rather than at parse — acceptable but a parse-time free-identifier check is the better shape.

## 8. Round 5 (the "extraction output is real Rust" correction, 2026-11-20)

Trigger: the user's correction that the extraction pipeline's own deliverable form was still partly plain text — the kernel ground truth lived under `extract/planck_effect/ground_truth/*.txt` as integer-per-line dumps, so the extracted tree was not uniformly real, compilable Rust. Scope of this round: the ground-truth form supersession (generated Rust `pub const` tables compiled into the DIF test via `include!`; the three .txt artifacts removed mechanically), rule DR-8.7 (extraction output is Rust compilation units, never plain-text dumps), the DR-8.7 check guards (legacy-.txt rejection, header/boundary/declared-and-actual-count guards, value truth left to the DIF), the in-pipeline lint-cep step, `rocq/Effect/CEP/CEPStandard.v` (the CEP standard itself formalized: extraction-only rule, hand-edit ban, pipeline gate transition system with the no-bypass theorem, pin law, DO order, five-vote rule, complete-domain law, Prop-erasure law — 30 statements, 30 Qed, zero axioms, coqchk-verified), `rocq/build.sh`'s second coqchk with output-gated OK echoes, `spec/TRUST-BOUNDARY.md` (layer provenance declaration: extract/ = only shippable implementation; rocq/ = proofs; math/ = untrusted reference kernels reclassified for shipment only, verification claims unchanged; tools/ = FV-2 tooling; rustc = deployment), and the dated DIF artifact `tools/fv/extract/artifacts/dif-rowalgebra-2026-11-20.md`. Five lenses, five independent agent reviews, each reproducing the evidence chain itself.

| Agent | Verdict | Notes |
| ----- | ------- | ----- |
| V1 | COMPLETE_AND_OPTIMAL | all four correction elements have exactly one normative home + one mechanical instrument (mapping recorded in findings); orphan sweep clean; exactness sweep clean; content preservation verified (65,536 entries per table, element-for-element equal to the superseded dumps) |
| V2 | COMPLETE_AND_OPTIMAL | 9+6 field sets verified programmatically on all changed/new artifacts; STATUS vocabulary clean; DR-8.7 vs DR-8.2 distinction honest; independent Python re-derivation of all three tables (0/196,608 mismatches; leq true pairs = 6,561 = 3^8) |
| V3 | COMPLETE_AND_OPTIMAL | kernel check + both coqchk runs reproduced (Axioms: none); no bypass constructors; section-9 honesty note verified against the proof term; no false lemma; DifGroundTruth.v byte-identical to HEAD; spot value + full-domain re-encoding verified; index encoders consistent across .v/gt.rs/dif_effect.rs |
| V4 | COMPLETE_AND_OPTIMAL | DR-8.7 coherent with DR-1..DR-8.6 and OB gates; TRUST-BOUNDARY consistent with CEP-DR-01 §4 and SPEC-INDEX §4/§5.1 (shipment-claim-only reclassification drawn exactly); Peregrine route untouched; pipeline order verified in code and live |
| V5 | COMPLETE_AND_OPTIMAL | every claim reproduced: build.sh exit 0 (both coqchk Axioms: none), lint-cep 3/3, check OK, cargo test debug+release green, sha256 of all three tables 3/3 match the artifact, live drift probe (append → check exit 1 → byte-identical restore → check exit 0), no phantom evidence |

Round-5 score: **5/5 COMPLETE_AND_OPTIMAL — the vote passes unanimously.**

Resolutions and standing decisions recorded by this round:

1. VR-11: the ground-truth artifact form is superseded by rule DR-8.7 — kernel oracle tables live as generated Rust source (`ground_truth/*_table.rs`) compiled into the DIF test harness; a plain-text dump is non-conforming as extraction output. The superseded .txt artifacts were removed mechanically (dif migration step), with content parity verified by two independent lenses (V1, V5) and full re-derivation by V2.
2. VR-12: the CEP standard itself is now a kernel-checked object: `rocq/Effect/CEP/CEPStandard.v` encodes the extraction-only rule, the hand-edit ban, the pipeline gates (no-bypass theorem), the pin law, the DO order, the five-vote acceptance rule, the complete-domain law, and Prop erasure as constructive definitions and theorems; `rocq/build.sh` coqchk-verifies it with output-gated OK echoes (the V4 round-5 observation that the echoes were asserted rather than parsed was adopted in the same pass).
3. VR-13: adopted on the spot before the acceptance commit: DR-8.7's scope quantifier narrowed to extractor-emitted deliverables (Cargo.lock named as a build output); DR-8.7's date corrected to the run clock (2026-11-20); TRUST-BOUNDARY wording tightened (guard scope named as ground_truth/; layer-I provenance claim split per file class; layer-III header reworded to "verification scaffolding; never compiler implementation"; extract/ write paths stated as extract + dif); declared-length guard added to check_ground_truth; CEP:SECURITY field added to the extractor tooling headers and both generator templates; the contentless lemma `revote_preserves_verdict` removed from CEPStandard.v; the drifted-artifact lemma's comment now states the premise's role honestly.
4. VR-14 (kept open for the next revision cycle, non-blocking): (a) the DIF artifact filename is date-only with same-day overwrite — a same-day second run rewrites the record (VR-4's "never rewritten" intent suggests time-or-sequence-qualified names); (b) lint-cep's banned-statement sweep does not yet cover Parameter/Variable/Hypothesis/Conjecture statements (build.sh's gated coqchk axiom check is the backstop and reported none); (c) CEP:SECURITY on the remaining hand-authored tooling sources (main.rs, parse.rs, cep.rs) rides the next standards touch per VR-10(c); (d) DO-1 extension (charclass + operator-table domains) is the next construction pass — proofs first, code only via extraction, on the DR-8.7 form.
5. Publication state: round 5 is committed locally (recorded as f872679 at vote time; the acceptance commit as pushed is e9a69b3, which adds the round-5 correction pass on top of the voted content); the push to github.com/axiomzero0/compcomp-planck is BLOCKED because the access token stored outside the repository was lost with the recycled session environment and no credentials exist in the current one. The commit lands on the next push once a fresh token is provided; the recorded verification chain above is reproducible locally without credentials (`rocq/build.sh`, `planck-extractor lint-cep|check|dif`, `cargo test` in `tools/extractor` and `extract/planck_effect`).

## 9. Round 6 (the DO-1 extension pass: charclass + operator-table domains, 2026-11-21)

Trigger: VR-14(d) of round 5 — "DO-1 extension (charclass + operator-table domains) is the next construction pass — proofs first, code only via extraction, on the DR-8.7 form" — plus the round-5 residuals VR-14(a)-(c) and VR-10(b)/(d) scheduled for this revision cycle. Scope: the DR-8.8 subset-extension record (written BEFORE the first integer-domain extraction, per VR-10(b)'s own wording); the four foundation units `rocq/Effect/CharClass.v`, `rocq/Effect/OperatorEnums.v`, `rocq/Effect/OperatorTableData.v` (generated), `rocq/Effect/OperatorTable.v`; the extractor's subset-v2 (lexer/parser/validator/emitter, the transcribe instrument, the free-identifier rule v2.11, the pinned-skip discipline); the two new extracted crates with complete-domain DIF against the kernel AND the reference; the three new MATP obligations; build.sh's dependency-ordered compile sequence and extended coqchk gates; the SPEC-INDEX §9 amendment.

| Agent | Verdict | Notes |
| ----- | ------- | ----- |
| V1 | COMPLETE_AND_OPTIMAL | all round-5 mandates mapped home+instrument; DR-8.8 v2.1-v2.15 normative and matching the implementation; bootstrap committed in-repo; v2 fixtures inventoried (13 invalid / 7 valid); content preservation clean (design.md byte-retained, spec diffs additions-only); the first-pass findings F-1..F-8 and the v2.12 wording slip all verified fixed |
| V2 | COMPLETE_AND_OPTIMAL | 7/7 .v and 7/7 extractor sources carry the full CEP+FV sets with CEP:SECURITY; 30/30 generated files carry GENERATED headers with sha256 provenance (the two dif_test.rs provenances fixed and hash-exact vs their sources); STATUS vocabulary clean; independent re-derivation of is_whitespace (256/256) and the reference axis (REF_KEYS, REF_OP_OF_SORTED_KEY, REF_LOOKUP_SWEEP 44/44 + 5 samples) ALL EXACT; lint-cep exit 0 |
| V3 | COMPLETE_AND_OPTIMAL | build.sh exit 0 with 7/7 gated INDEPENDENT_CHECK: OK (after the V3-found SIGPIPE bug in the printf\|grep -q gates was fixed with bash string matching — the fix itself was V3's contribution); coqchk individually re-verified per unit (Axioms: none, no unsafe fixpoints, no assumed positivity); admit sweep 0; Lemma/Qed counts matched (CharClass 11/11, OperatorEnums 4/4, OperatorTable 15/15); the 1936-pair and 256-byte laws quoted and assessed complete-domain |
| V4 | COMPLETE_AND_OPTIMAL | v2.1-v2.15 each mapped to implementing code with no silent widening; v2.4/v2.14 contradiction resolved in the record; v2.5 exhaustiveness enforced mechanically with fixtures; v2.11 letter aligned; TRUST-BOUNDARY amended with consistent numerals; v2.7 amended rule verified against the validator; Peregrine untouched; CEPStandard.v and RowAlgebra.v byte-untouched |
| V5 | COMPLETE_AND_OPTIMAL | every claim reproduced: check/lint exit 0; live drift probe fail-then-restore-then-pass; transcription byte-identity (bootstrap == transcribe, sha d755b7fa...); MATP 3 new obligations Theorem with the 9 pre-existing honest failures never rewritten (OUTCOME diff 0); sha256 audit 21/21; final drift confined to the acceptance-commit deliverable set |

Round-6 score: **5/5 COMPLETE_AND_OPTIMAL — the vote passes unanimously.**

Resolutions and standing decisions recorded by this round:

1. VR-15: the extraction subset is versioned normatively as v1 (DR-8.1, boolean algebra, unchanged for the effect lane) plus v2 (DR-8.8, items v2.1-v2.15): nat and byte literals bounded to the pinned domain at the term parser, the longhand Nat.* comparisons and the saturating arithmetic mappings, if/let/match forms, closed inductive enumerations with exactly the pinned Ok/Err data pair (mapped to core Result), mechanical closed-match exhaustiveness (no wildcard arms; the Ok/Err binding wildcards are patterns, not arms), ASCII key literals with the 1..=126 bound, the fuel-Fixpoint shape (every recursive call passes the S-arm binder; at most 2 call sites, one per branch), the pinned-skip discipline (row_at/key_eq/key_lt and their fuel helpers mapped at call sites, discharged by the DIF), the PinnedEnums name-hygiene wrapper, the parse-time free-identifier rule (v2.11, forward references allowed, pinned-skip exempt), and the transcription record (v2.15, byte-identity cross-checked against the committed bootstrap at tools/extractor/bootstrap/gen_operator_data.py). The record was written BEFORE the first integer-domain extraction (VR-10(b)'s own wording) and amended within this pass exactly where the vote lenses found the implementation ahead of the letter — every amendment is recorded, none silent.
2. VR-16: the DO-1 phase-1 foundation domains are accepted as extraction units with the complete-domain evidence chain: CharClass (13 tabulated byte->bool functions, 256-byte three-way DIF vs kernel AND reference), OperatorTable (44-row closed domain, 1936 pairs x key_eq/key_lt/precedence_compare, the pinned 16,563-key lookup sweep, three-way vs the generator's independent linear scan), all in debug AND release; artifacts time-qualified per VR-14(a).
3. VR-17 (adopted on the spot before the acceptance commit, per the VR-13 precedent): the build.sh coqchk gates use bash string matching instead of printf|grep -q under pipefail (the V3-found SIGPIPE misfire on large outputs — a gate bug, never a proof failure); the bootstrap generator moved into the repo at tools/extractor/bootstrap/ with the emitted provenance updated; the two new crates carry their own rust-toolchain.toml pins; .gitignore covers rocq/.lia.cache and rustc long-type diagnostics; TRUST-BOUNDARY.md carries the dated DO-1 amendment; VOTE-RECORD's round-5 commit reference corrected to the as-pushed e9a69b3.
4. VR-18 (kept open for the next revision cycle, non-blocking): (a) MATP's 9 pre-existing effect/ownership obligations remain honest fail-with-disclosure (GaveUp/DepthExceeded/Saturated), discharged by full-domain EXH per SPEC-INDEX §4 — unchanged by this pass; (b) the operator sweep GT encodes OP numbers while the reference scan was corrected to the same encoding — the encoding convention could be stated once in DR-8.8 at its next touch; (c) the row_at out-of-target behavior (nat >= 44 maps to row_43 in the kernel, array-index panics in Rust) is bounded by the pinned domains but could carry an explicit clamping record if the extractor's role grows.
5. Publication state: round 6 is committed locally and pushed to github.com/axiomzero0/compcomp-planck (the acceptance commit follows this record).
