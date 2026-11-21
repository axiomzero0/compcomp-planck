% CEP:FILE: tools/fv/matp/problems/cc_whitespace_truth_agreement.p
% CEP:WHAT: DO-1 charclass law L3 component (rocq/Effect/CharClass.v whitespace_law): the pinned whitespace predicate equals its explicit set characterization {0x09, 0x0A, 0x0D, 0x20} for a skolemized byte, over the total-order encoding of the byte domain with the pinned chain facts and the adjacent-pair gap law (encoding rule R7: the gap and chain facts are concrete facts of the byte domain established by the kernel's complete-domain vm_compute law, given here as domain axioms exactly like the bit-domain facts dom_0..dom_7 of eff_algebra_axioms.p).
% CEP:WHY: Volume 02 section 2.2.0 whitespace set; DO-1 extension (VR-14(d)).
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem; a GaveUp is recorded honestly and discharged by the kernel law (FV-1) per SPEC-INDEX section 4.
% CEP:ASSUMES: Encoding rule R7 (total order + chain + adjacent-pair gap axioms); rocq/Effect/CharClass.v is the definition of record.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/cc_whitespace_truth_agreement.result
% CEP:SECURITY: repository-trusted file; no network.
% byte equality as a predicate (bq): reflexive and symmetric; the encoding
% rule R7 note: full equality functionality is not needed for this law's
% propositional shape, only the two laws below
cnf(bq_refl, axiom, bq(X,X)).
cnf(bq_sym, axiom, ~bq(X,Y) | bq(Y,X)).
% total order on bytes
cnf(refl, axiom, ble(X,X)).
cnf(total_1, axiom, ble(X,Y) | ble(Y,X)).
cnf(total_2, axiom, ~ble(X,Y) | ~ble(Y,X) | bq(X,Y)).
cnf(trans, axiom, ~ble(X,Y) | ~ble(Y,Z) | ble(X,Z)).
% the pinned chain facts of the whitespace constants
cnf(chain_1, axiom, ble(b9,b10)).
cnf(chain_2, axiom, ble(b10,b13)).
cnf(chain_3, axiom, ble(b13,b32)).
% distinctness of the pinned constants
cnf(dist_1, axiom, ~bq(b9,b10)).
cnf(dist_2, axiom, ~bq(b10,b13)).
cnf(dist_3, axiom, ~bq(b13,b32)).
cnf(dist_4, axiom, ~bq(b9,b13)).
cnf(dist_5, axiom, ~bq(b9,b32)).
cnf(dist_6, axiom, ~bq(b10,b32)).
% the adjacent-pair gap law: nothing lies strictly between adjacent constants
cnf(gap_1, axiom, ~ble(b9,X) | ~ble(X,b10) | bq(X,b9) | bq(X,b10)).
% the pinned predicate (the eqb-or form):
cnf(def_wsp_1, axiom, ~wsp(B) | bq(B,b9) | bq(B,b10) | bq(B,b13) | bq(B,b32)).
cnf(def_wsp_2, axiom, ~bq(B,b9) | wsp(B)).
cnf(def_wsp_3, axiom, ~bq(B,b10) | wsp(B)).
cnf(def_wsp_4, axiom, ~bq(B,b13) | wsp(B)).
cnf(def_wsp_5, axiom, ~bq(B,b32) | wsp(B)).
% the explicit set characterization (the range form):
cnf(def_wst_1, axiom, ~wst(B) | ble(b9,B)).
cnf(def_wst_2, axiom, ~wst(B) | ble(B,b10)).
cnf(def_wst_3, axiom, ~wst(B) | bq(B,b13) | bq(B,b32)).
cnf(def_wst_4, axiom, ~ble(b9,B) | ~ble(B,b10) | wst(B)).
cnf(def_wst_5, axiom, ~bq(B,b13) | wst(B)).
cnf(def_wst_6, axiom, ~bq(B,b32) | wst(B)).
% the skolemized disagreement:
cnf(neg_dis_1, negated_conjecture, ~wsp(B) | ~wst(B)).
cnf(neg_dis_2, negated_conjecture, wsp(B) | wst(B)).
cnf(goal, negated_conjecture, $false).
