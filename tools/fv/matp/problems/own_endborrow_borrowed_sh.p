% CEP:FILE: tools/fv/matp/problems/own_endborrow_borrowed_sh.p
% CEP:WHAT: Ownership obligation OC-3a (pinned law: EndBorrow on a place with outstanding shared loans never rejects): trans(sh_n1, a_endb) = r_sh_n0, hence not rj; negated as the single unit claiming the cell rejects (relational encoding, rule R2).
% CEP:WHY: SPEC-INDEX section 5.4 EndBorrow-never-rejects law, shared-loan case; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: The refutation resolves the unit with trans_func (functionality of trans over the modeled slice), instantiates R2 by the cell fact trans(sh_n1, a_endb, r_sh_n0), and kills the residual equality rj = r_sh_n0 with the distinctness fact rj != r_sh_n0; a unit goal keeps every derived literal eligible for ordered resolution.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/own_endborrow_borrowed_sh.result
% CEP:SECURITY: repository-trusted file; no network.
include('own_table_axioms.p').
cnf(goal_neg, axiom, trans(sh_n1, a_endb, rj)).
