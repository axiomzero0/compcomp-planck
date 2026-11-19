% CEP:FILE: tools/fv/matp/problems/own_endborrow_zero.p
% CEP:WHAT: Ownership obligation OC-2 (pinned law: Shared(0) and Frozen(0) normalize to Owned at EndBorrow): the two pinned cells hold r_owned; negated as the disjunction that one of them is wrong (relational encoding, rule R2).
% CEP:WHY: SPEC-INDEX section 5.4 EndBorrow-zero law; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: The negation of the conjunction of the two pinned cells is the disjunction of the two negated trans atoms; the refutation resolves each disjunct with its cell fact.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/own_endborrow_zero.result
% CEP:SECURITY: repository-trusted file; no network.
include('own_table_axioms.p').
cnf(goal_neg, axiom, ~trans(sh_n0, a_endb, r_owned) | ~trans(fr_n0, a_endb, r_owned)).
