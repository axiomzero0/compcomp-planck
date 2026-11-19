% CEP:FILE: tools/fv/matp/problems/eff_absorb.p
% CEP:WHAT: Effect-row law L6: absorption, meet(X,join(X,Y)) = X; skolemized relational negation (encoding rule R1).
% CEP:WHY: Volume 07 section 7.1 law L6; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: Encoding rule R1 (see eff_join_assoc.p).
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_absorb.result
% CEP:SECURITY: repository-trusted file; no network.
include('eff_algebra_axioms.p').
cnf(neg_j1, axiom, joinr(a, b, ab)).
cnf(neg_m1, axiom, meetr(a, ab, m)).
cnf(neg_neq, axiom, ~roweq(m, a)).
