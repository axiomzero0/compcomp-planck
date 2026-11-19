% CEP:FILE: tools/fv/matp/problems/eff_join_comm.p
% CEP:WHAT: Effect-row law L2: join is commutative, join(X,Y) = join(Y,X); skolemized relational negation (encoding rule R1).
% CEP:WHY: Volume 07 section 7.1 law L2; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: Encoding rule R1 (see eff_join_assoc.p).
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_join_comm.result
% CEP:SECURITY: repository-trusted file; no network.
include('eff_algebra_axioms.p').
cnf(neg_j1, axiom, joinr(a, b, w1)).
cnf(neg_j2, axiom, joinr(b, a, w2)).
cnf(neg_neq, axiom, ~roweq(w1, w2)).
