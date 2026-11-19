% CEP:FILE: tools/fv/matp/problems/harness_sat.p
% CEP:WHAT: Harness sanity obligation: two ground facts with no complementary pair must saturate.
% CEP:WHY: Validates the Saturated outcome path of the driver.
% CEP:CLASS: FV-2
% CEP:STATUS: complete
% CEP:FAILURE: Driver fails if the outcome is not Saturated.
% CEP:ASSUMES: none.
% CEP:COST: offline.
% CEP:EVIDENCE: tools/fv/matp/artifacts/harness_sat.result
% CEP:SECURITY: repository-trusted file.
cnf(ax1, axiom, p(c1)).
cnf(ax2, axiom, q(c1)).
