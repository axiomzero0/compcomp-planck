% CEP:FILE: tools/fv/matp/problems/eff_join_assoc.p
% CEP:WHAT: Effect-row law L1 (SPEC-INDEX section 5.3; Volume 07 section 7.1): join is associative, join(join(X,Y),Z) = join(X,join(Y,Z)); stated as the skolemized relational negation (the negated law is a satisfiable existential: rows a,b,c with joins ab, bc and results w1, w2 where w1 is not roweq w2).
% CEP:WHY: Volume 07 section 7.1 law L1; the negation is included as the last clauses (no clausifier in edition P1), so Theorem proves the law.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: Encoding rule R1 (relational skolemized negation): the negation of the law is the clause set that asserts witnesses for every existential of the negation; row identity is roweq per eff_algebra_axioms.p. Theorem is the outcome iff the law is a semantic consequence of the axiom file.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_join_assoc.result
% CEP:SECURITY: repository-trusted file; no network.
include('eff_algebra_axioms.p').
cnf(neg_j1, axiom, joinr(a, b, ab)).
cnf(neg_j2, axiom, joinr(ab, c, w1)).
cnf(neg_j3, axiom, joinr(b, c, bc)).
cnf(neg_j4, axiom, joinr(a, bc, w2)).
cnf(neg_neq, axiom, ~roweq(w1, w2)).
