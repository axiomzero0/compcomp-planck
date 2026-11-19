% CEP:FILE: tools/fv/matp/problems/eff_meet_distrib.p
% CEP:WHAT: Effect-row law L5: meet distributes over join, meet(X,join(Y,Z)) = join(meet(X,Y),meet(X,Z)); skolemized relational negation (encoding rule R1).
% CEP:WHY: Volume 07 section 7.1 law L5; Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: Encoding rule R1 (see eff_join_assoc.p).
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_meet_distrib.result
% CEP:SECURITY: repository-trusted file; no network.
include('eff_algebra_axioms.p').
cnf(neg_j1, axiom, joinr(b, c, bc)).
cnf(neg_m1, axiom, meetr(a, bc, m1)).
cnf(neg_m2, axiom, meetr(a, b, mab)).
cnf(neg_m3, axiom, meetr(a, c, mac)).
cnf(neg_j2, axiom, joinr(mab, mac, w2)).
cnf(neg_neq, axiom, ~roweq(m1, w2)).
