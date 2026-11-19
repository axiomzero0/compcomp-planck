% CEP:FILE: tools/fv/matp/problems/harness_mp.p
% CEP:WHAT: Harness sanity obligation: modus ponens (~p(X) | q(X), p(c1), ~q(c1)) must refute.
% CEP:WHY: Validates the driver and the MATP engine before the math obligations run.
% CEP:CLASS: FV-2
% CEP:STATUS: complete
% CEP:FAILURE: Driver fails if the outcome is not Theorem.
% CEP:ASSUMES: none.
% CEP:COST: offline.
% CEP:EVIDENCE: tools/fv/matp/artifacts/harness_mp.result
% CEP:SECURITY: repository-trusted file.
cnf(ax1, axiom, ~p(X) | q(X)).
cnf(ax2, axiom, p(c1)).
cnf(ax3, axiom, ~q(c1)).
