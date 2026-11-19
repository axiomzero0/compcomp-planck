% CEP:FILE: tools/fv/matp/problems/own_endborrow_borrowed_fr.p
% CEP:WHAT: Ownership obligation OC-3c (pinned law: EndBorrow on a frozen place never rejects): trans(fr_n1, a_endb) = r_sh_n1, hence not rj; negated as the single unit claiming the cell rejects (relational encoding, rule R2).
% CEP:WHY: SPEC-INDEX section 5.4 EndBorrow-never-rejects law, frozen case; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: The refutation resolves the unit with trans_func, instantiates R2 by the cell fact trans(fr_n1, a_endb, r_sh_n1), and kills the residual equality rj = r_sh_n1 with the distinctness fact rj != r_sh_n1.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/own_endborrow_borrowed_fr.result
% CEP:SECURITY: repository-trusted file; no network.
include('own_table_axioms.p').
cnf(goal_neg, axiom, trans(fr_n1, a_endb, rj)).
