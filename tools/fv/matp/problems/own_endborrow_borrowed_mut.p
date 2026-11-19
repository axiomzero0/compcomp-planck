% CEP:FILE: tools/fv/matp/problems/own_endborrow_borrowed_mut.p
% CEP:WHAT: Ownership obligation OC-3b (pinned law: EndBorrow on a mutably borrowed place never rejects): trans(s_mut, a_endb) = r_owned, hence not rj; negated as the single unit claiming the cell rejects (relational encoding, rule R2).
% CEP:WHY: SPEC-INDEX section 5.4 EndBorrow-never-rejects law, mutable-borrow case; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: The refutation resolves the unit with trans_func, instantiates R2 by the cell fact trans(s_mut, a_endb, r_owned), and kills the residual equality rj = r_owned with the distinctness fact rj != r_owned (reversed-order fact res_distR_4).
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/own_endborrow_borrowed_mut.result
% CEP:SECURITY: repository-trusted file; no network.
include('own_table_axioms.p').
cnf(goal_neg, axiom, trans(s_mut, a_endb, rj)).
