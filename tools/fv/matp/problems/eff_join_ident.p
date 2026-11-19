% CEP:FILE: tools/fv/matp/problems/eff_join_ident.p
% CEP:WHAT: Effect-row law L4: the empty row z is the join identity, join(X,z) = X; skolemized relational negation (encoding rule R1).
% CEP:WHY: Volume 07 section 7.1 law L4; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: Encoding rule R1 (see eff_join_assoc.p).
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_join_ident.result
% CEP:SECURITY: repository-trusted file; no network.
include('eff_algebra_axioms.p').
cnf(neg_j1, axiom, joinr(a, z, w)).
cnf(neg_neq, axiom, ~roweq(w, a)).
