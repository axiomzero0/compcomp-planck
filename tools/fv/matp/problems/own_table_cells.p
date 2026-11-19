% CEP:FILE: tools/fv/matp/problems/own_table_cells.p
% CEP:WHAT: Ownership obligation OC-1: four pinned table cells hold exactly the SPEC-INDEX section 5.4 values (shared-borrow from Shared(1) increments; mutable borrow from Owned; move-out from Owned leaves MovedFrom; read from Owned keeps Owned); stated as the negated disjunction that one of the four cells is wrong, in the relational encoding (rule R2).
% CEP:WHY: Cell-level spot check of the committed transition table against the pinned values; Theorem proves the four cells. A table edited to any other value in these cells leaves the goal satisfiable and fails the driver.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: The driver fails unless the outcome is Theorem.
% CEP:ASSUMES: The negation of the conjunction of the four pinned cells is the disjunction of the four negated trans atoms; the refutation resolves each disjunct with its cell fact.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/own_table_cells.result
% CEP:SECURITY: repository-trusted file; no network.
include('own_table_axioms.p').
cnf(goal_neg, axiom, ~trans(sh_n1, a_bsh, r_sh_n2) | ~trans(s_owned, a_bmut, r_mut) | ~trans(s_owned, a_mv, r_moved) | ~trans(s_owned, a_read, r_owned)).
