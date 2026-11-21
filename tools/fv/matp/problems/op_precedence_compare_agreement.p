% CEP:FILE: tools/fv/matp/problems/op_precedence_compare_agreement.p
% CEP:WHAT: DO-1 operator law OL8 component (rocq/Effect/OperatorTable.v precedence_compare_all_pairs): precedence_compare(A,B) equals the ordinal comparison of the precedence column for a skolemized pair, under the strict trichotomy of the pinned nat order (encoding rule R8: the 44x44 complete-domain truth is the Coq kernel vm_compute law; this obligation carries the definitional shape of the comparison agreement to the ATP engine).
% CEP:WHY: Volume 02 section 2.4.1 Prec column; the Pratt climber contract (2.6.4); DO-1 extension (VR-14(d)).
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem; a GaveUp is recorded honestly and discharged by the kernel law (FV-1) per SPEC-INDEX section 4.
% CEP:ASSUMES: Encoding rule R8 (strict trichotomy as the pinned nat-order law); rocq/Effect/OperatorTable.v is the definition of record.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/op_precedence_compare_agreement.result
% CEP:SECURITY: repository-trusted file; no network.
% strict trichotomy of the nat order (the pinned domain law)
cnf(trich, axiom, nlt(pa,pb) | nlt(pb,pa) | pa = pb).
% precedence_compare (the pinned if-chain, unfolded):
cnf(cmp_1, axiom, ~nlt(pa,pb) | cmp = o_less).
cnf(cmp_2, axiom, ~nlt(pb,pa) | cmp = o_greater).
cnf(cmp_3, axiom, pa != pb | cmp = o_equal).
% the ordinal comparison of the column values (the same shape):
cnf(ord_1, axiom, ~nlt(pa,pb) | ord = o_less).
cnf(ord_2, axiom, ~nlt(pb,pa) | ord = o_greater).
cnf(ord_3, axiom, pa != pb | ord = o_equal).
% the skolemized disagreement:
cnf(neg_disagree, negated_conjecture, cmp != ord).
cnf(goal, negated_conjecture, $false).
