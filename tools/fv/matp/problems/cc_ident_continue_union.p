% CEP:FILE: tools/fv/matp/problems/cc_ident_continue_union.p
% CEP:WHAT: DO-1 charclass law L1 (rocq/Effect/CharClass.v ident_continue_is_the_union): the union law of IDENT_CONTINUE over one skolemized byte, in the contrapositive form, with the range predicates abstracted as shared propositions (encoding rule R6: the domain-level truth of the range facts is the Coq kernel's complete-domain vm_compute law; this obligation carries the propositional shape of the union law to the ATP engine).
% CEP:WHY: Volume 02 section 2.2.0 IDENT_CONT; the DO-1 extension pass (VR-14(d)) brings the charclass domain to the dual-engine discipline.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem; a GaveUp is recorded honestly and discharged by the kernel law (FV-1) per SPEC-INDEX section 4.
% CEP:ASSUMES: Encoding rule R6 (shared propositions for both sides of the definitional law); rocq/Effect/CharClass.v is the definition of record.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/cc_ident_continue_union.result
% CEP:SECURITY: repository-trusted file; no network.
% the pinned definition of IDENT_CONTINUE (union law, as two implications):
cnf(def_cont_1, axiom, ~idcont(B) | idstart(B) | decdig(B)).
cnf(def_cont_2, axiom, ~idstart(B) | idcont(B)).
cnf(def_cont_3, axiom, ~decdig(B) | idcont(B)).
% the skolemized outside point: B in neither class
cnf(fact_1, axiom, ~idstart(B)).
cnf(fact_2, axiom, ~decdig(B)).
% the union law's contrapositive: B is not IDENT_CONTINUE
cnf(neg_goal, negated_conjecture, idcont(B)).
cnf(goal, negated_conjecture, $false).
