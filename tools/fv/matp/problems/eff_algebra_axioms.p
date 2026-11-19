% CEP:FILE: tools/fv/matp/problems/eff_algebra_axioms.p
% CEP:WHAT: The pinned effect-row algebra (SPEC-INDEX section 5.3; Volume 07 section 7.1) in the relational encoding: 8 effect atoms as 8 bit projections bitI(R,V) over the ground bit domain {b0, b1}, join and meet as the relations joinr/3 and meetr/3 defined projection-wise by the ground truth tables ort/3 and andt/3, row identity as the predicate roweq/2 (bitwise identity), and the empty row z.
% CEP:WHY: MATP verifies the algebraic laws of the effect system by resolution over these clauses; every eff_* obligation includes this file. The encoding is relational because the pinned MATP atom grammar (github.com/compilecomp/MATP cold/parsing/tptp_parser.rs parse_atomic_shape) admits only variables and 0-ary functions on the written left of '=' / '!=', and the pinned given-clause engine (hot/inference/superposition.rs, demodulation.rs) uses equations strictly in their written orientation under the KBO side condition left > right; definitional equalities between compound terms are therefore stated as definitional predicates and discharged by ordered resolution with selection, which is complete without any equality-inference rule.
% CEP:CLASS: FV-0
% CEP:STATUS: complete
% CEP:FAILURE: If any clause here contradicts SPEC-INDEX section 5.3, the effect obligations fail.
% CEP:ASSUMES: The encoding is exact: bitI is functional over {b0,b1} (dom_i = at least one bit value, def_i = at most one); ort/andt are the ground truth tables of bitwise or/and made functional by ort_def/andt_def plus the distinctness b0 != b1; joinr/3 (resp. meetr/3) relates two rows to a row whose every projection is the or (resp. and) of the inputs' projections (jf_i forward, jb_i backward, and the meet analogs); roweq/2 is bitwise identity (roweq_f_i transfer both ways, roweq_b extensionality over the eqb_i witness predicates); z has all bits clear. Two rows are equal in the law statements exactly when roweq holds, which is the representation equality of the u8 bitmask kernel extracted in math/crates/planck-effect-math.
% CEP:COST: offline proof artifact; runtime cost irrelevant.
% CEP:EVIDENCE: tools/fv/matp/artifacts/eff_*.result
% CEP:SECURITY: repository-trusted file; no network.
%
% --- Bit domain: each projection has exactly one value in {b0, b1} ---
cnf(dom_0, axiom, bit0(R, b0) | bit0(R, b1)).
cnf(dom_1, axiom, bit1(R, b0) | bit1(R, b1)).
cnf(dom_2, axiom, bit2(R, b0) | bit2(R, b1)).
cnf(dom_3, axiom, bit3(R, b0) | bit3(R, b1)).
cnf(dom_4, axiom, bit4(R, b0) | bit4(R, b1)).
cnf(dom_5, axiom, bit5(R, b0) | bit5(R, b1)).
cnf(dom_6, axiom, bit6(R, b0) | bit6(R, b1)).
cnf(dom_7, axiom, bit7(R, b0) | bit7(R, b1)).
cnf(def_0, axiom, ~bit0(R, b0) | ~bit0(R, b1)).
cnf(def_1, axiom, ~bit1(R, b0) | ~bit1(R, b1)).
cnf(def_2, axiom, ~bit2(R, b0) | ~bit2(R, b1)).
cnf(def_3, axiom, ~bit3(R, b0) | ~bit3(R, b1)).
cnf(def_4, axiom, ~bit4(R, b0) | ~bit4(R, b1)).
cnf(def_5, axiom, ~bit5(R, b0) | ~bit5(R, b1)).
cnf(def_6, axiom, ~bit6(R, b0) | ~bit6(R, b1)).
cnf(def_7, axiom, ~bit7(R, b0) | ~bit7(R, b1)).
% --- Bit distinctness ---
cnf(b_distinct, axiom, b0 != b1).
% --- Ground truth tables: bitwise or ---
cnf(ort_00, axiom, ort(b0, b0, b0)).
cnf(ort_01, axiom, ort(b0, b1, b1)).
cnf(ort_10, axiom, ort(b1, b0, b1)).
cnf(ort_11, axiom, ort(b1, b1, b1)).
% --- Ground truth tables: bitwise and ---
cnf(andt_00, axiom, andt(b0, b0, b0)).
cnf(andt_01, axiom, andt(b0, b1, b0)).
cnf(andt_10, axiom, andt(b1, b0, b0)).
cnf(andt_11, axiom, andt(b1, b1, b1)).
% --- Truth-table functionality (ort/andt are functions on {b0,b1}^2) ---
cnf(ort_def, axiom, ~ort(VX, VY, V1) | ~ort(VX, VY, V2) | V1 = V2).
cnf(andt_def, axiom, ~andt(VX, VY, V1) | ~andt(VX, VY, V2) | V1 = V2).
% --- join definition: every projection of the join is the or of the projections ---
% Forward: the join's bit i is the or of the input bits (resolved case-wise).
cnf(jf_0, axiom, ~joinr(X, Y, Z) | ~bit0(X, VX) | ~bit0(Y, VY) | ~ort(VX, VY, VZ) | bit0(Z, VZ)).
cnf(jf_1, axiom, ~joinr(X, Y, Z) | ~bit1(X, VX) | ~bit1(Y, VY) | ~ort(VX, VY, VZ) | bit1(Z, VZ)).
cnf(jf_2, axiom, ~joinr(X, Y, Z) | ~bit2(X, VX) | ~bit2(Y, VY) | ~ort(VX, VY, VZ) | bit2(Z, VZ)).
cnf(jf_3, axiom, ~joinr(X, Y, Z) | ~bit3(X, VX) | ~bit3(Y, VY) | ~ort(VX, VY, VZ) | bit3(Z, VZ)).
cnf(jf_4, axiom, ~joinr(X, Y, Z) | ~bit4(X, VX) | ~bit4(Y, VY) | ~ort(VX, VY, VZ) | bit4(Z, VZ)).
cnf(jf_5, axiom, ~joinr(X, Y, Z) | ~bit5(X, VX) | ~bit5(Y, VY) | ~ort(VX, VY, VZ) | bit5(Z, VZ)).
cnf(jf_6, axiom, ~joinr(X, Y, Z) | ~bit6(X, VX) | ~bit6(Y, VY) | ~ort(VX, VY, VZ) | bit6(Z, VZ)).
cnf(jf_7, axiom, ~joinr(X, Y, Z) | ~bit7(X, VX) | ~bit7(Y, VY) | ~ort(VX, VY, VZ) | bit7(Z, VZ)).
% Backward: the join's bit i forces the or of the input bits (definiteness of the
% inputs makes this exactly the definitional equality, discharged by resolution).
cnf(jb_0, axiom, ~joinr(X, Y, Z) | ~bit0(Z, VZ) | ~bit0(X, VX) | ~bit0(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_1, axiom, ~joinr(X, Y, Z) | ~bit1(Z, VZ) | ~bit1(X, VX) | ~bit1(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_2, axiom, ~joinr(X, Y, Z) | ~bit2(Z, VZ) | ~bit2(X, VX) | ~bit2(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_3, axiom, ~joinr(X, Y, Z) | ~bit3(Z, VZ) | ~bit3(X, VX) | ~bit3(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_4, axiom, ~joinr(X, Y, Z) | ~bit4(Z, VZ) | ~bit4(X, VX) | ~bit4(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_5, axiom, ~joinr(X, Y, Z) | ~bit5(Z, VZ) | ~bit5(X, VX) | ~bit5(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_6, axiom, ~joinr(X, Y, Z) | ~bit6(Z, VZ) | ~bit6(X, VX) | ~bit6(Y, VY) | ort(VX, VY, VZ)).
cnf(jb_7, axiom, ~joinr(X, Y, Z) | ~bit7(Z, VZ) | ~bit7(X, VX) | ~bit7(Y, VY) | ort(VX, VY, VZ)).
% --- meet definition: every projection of the meet is the and of the projections ---
cnf(mf_0, axiom, ~meetr(X, Y, Z) | ~bit0(X, VX) | ~bit0(Y, VY) | ~andt(VX, VY, VZ) | bit0(Z, VZ)).
cnf(mf_1, axiom, ~meetr(X, Y, Z) | ~bit1(X, VX) | ~bit1(Y, VY) | ~andt(VX, VY, VZ) | bit1(Z, VZ)).
cnf(mf_2, axiom, ~meetr(X, Y, Z) | ~bit2(X, VX) | ~bit2(Y, VY) | ~andt(VX, VY, VZ) | bit2(Z, VZ)).
cnf(mf_3, axiom, ~meetr(X, Y, Z) | ~bit3(X, VX) | ~bit3(Y, VY) | ~andt(VX, VY, VZ) | bit3(Z, VZ)).
cnf(mf_4, axiom, ~meetr(X, Y, Z) | ~bit4(X, VX) | ~bit4(Y, VY) | ~andt(VX, VY, VZ) | bit4(Z, VZ)).
cnf(mf_5, axiom, ~meetr(X, Y, Z) | ~bit5(X, VX) | ~bit5(Y, VY) | ~andt(VX, VY, VZ) | bit5(Z, VZ)).
cnf(mf_6, axiom, ~meetr(X, Y, Z) | ~bit6(X, VX) | ~bit6(Y, VY) | ~andt(VX, VY, VZ) | bit6(Z, VZ)).
cnf(mf_7, axiom, ~meetr(X, Y, Z) | ~bit7(X, VX) | ~bit7(Y, VY) | ~andt(VX, VY, VZ) | bit7(Z, VZ)).
cnf(mb_0, axiom, ~meetr(X, Y, Z) | ~bit0(Z, VZ) | ~bit0(X, VX) | ~bit0(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_1, axiom, ~meetr(X, Y, Z) | ~bit1(Z, VZ) | ~bit1(X, VX) | ~bit1(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_2, axiom, ~meetr(X, Y, Z) | ~bit2(Z, VZ) | ~bit2(X, VX) | ~bit2(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_3, axiom, ~meetr(X, Y, Z) | ~bit3(Z, VZ) | ~bit3(X, VX) | ~bit3(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_4, axiom, ~meetr(X, Y, Z) | ~bit4(Z, VZ) | ~bit4(X, VX) | ~bit4(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_5, axiom, ~meetr(X, Y, Z) | ~bit5(Z, VZ) | ~bit5(X, VX) | ~bit5(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_6, axiom, ~meetr(X, Y, Z) | ~bit6(Z, VZ) | ~bit6(X, VX) | ~bit6(Y, VY) | andt(VX, VY, VZ)).
cnf(mb_7, axiom, ~meetr(X, Y, Z) | ~bit7(Z, VZ) | ~bit7(X, VX) | ~bit7(Y, VY) | andt(VX, VY, VZ)).
% --- Projection-equality witness predicates eqb0..eqb7 ---
% eqbI(W1, W2) means: rows W1 and W2 have the same projection-i bit.
% Forward: eqbI transfers the bit value both ways.
cnf(eqb0_f0, axiom, ~eqb0(W1, W2) | ~bit0(W1, V) | bit0(W2, V)).
cnf(eqb0_f1, axiom, ~eqb0(W1, W2) | ~bit0(W2, V) | bit0(W1, V)).
cnf(eqb1_f0, axiom, ~eqb1(W1, W2) | ~bit1(W1, V) | bit1(W2, V)).
cnf(eqb1_f1, axiom, ~eqb1(W1, W2) | ~bit1(W2, V) | bit1(W1, V)).
cnf(eqb2_f0, axiom, ~eqb2(W1, W2) | ~bit2(W1, V) | bit2(W2, V)).
cnf(eqb2_f1, axiom, ~eqb2(W1, W2) | ~bit2(W2, V) | bit2(W1, V)).
cnf(eqb3_f0, axiom, ~eqb3(W1, W2) | ~bit3(W1, V) | bit3(W2, V)).
cnf(eqb3_f1, axiom, ~eqb3(W1, W2) | ~bit3(W2, V) | bit3(W1, V)).
cnf(eqb4_f0, axiom, ~eqb4(W1, W2) | ~bit4(W1, V) | bit4(W2, V)).
cnf(eqb4_f1, axiom, ~eqb4(W1, W2) | ~bit4(W2, V) | bit4(W1, V)).
cnf(eqb5_f0, axiom, ~eqb5(W1, W2) | ~bit5(W1, V) | bit5(W2, V)).
cnf(eqb5_f1, axiom, ~eqb5(W1, W2) | ~bit5(W2, V) | bit5(W1, V)).
cnf(eqb6_f0, axiom, ~eqb6(W1, W2) | ~bit6(W1, V) | bit6(W2, V)).
cnf(eqb6_f1, axiom, ~eqb6(W1, W2) | ~bit6(W2, V) | bit6(W1, V)).
cnf(eqb7_f0, axiom, ~eqb7(W1, W2) | ~bit7(W1, V) | bit7(W2, V)).
cnf(eqb7_f1, axiom, ~eqb7(W1, W2) | ~bit7(W2, V) | bit7(W1, V)).
% Backward: matching bits at projection i establish eqbI.
cnf(eqb0_b0, axiom, eqb0(W1, W2) | ~bit0(W1, b0) | ~bit0(W2, b0)).
cnf(eqb0_b1, axiom, eqb0(W1, W2) | ~bit0(W1, b1) | ~bit0(W2, b1)).
cnf(eqb1_b0, axiom, eqb1(W1, W2) | ~bit1(W1, b0) | ~bit1(W2, b0)).
cnf(eqb1_b1, axiom, eqb1(W1, W2) | ~bit1(W1, b1) | ~bit1(W2, b1)).
cnf(eqb2_b0, axiom, eqb2(W1, W2) | ~bit2(W1, b0) | ~bit2(W2, b0)).
cnf(eqb2_b1, axiom, eqb2(W1, W2) | ~bit2(W1, b1) | ~bit2(W2, b1)).
cnf(eqb3_b0, axiom, eqb3(W1, W2) | ~bit3(W1, b0) | ~bit3(W2, b0)).
cnf(eqb3_b1, axiom, eqb3(W1, W2) | ~bit3(W1, b1) | ~bit3(W2, b1)).
cnf(eqb4_b0, axiom, eqb4(W1, W2) | ~bit4(W1, b0) | ~bit4(W2, b0)).
cnf(eqb4_b1, axiom, eqb4(W1, W2) | ~bit4(W1, b1) | ~bit4(W2, b1)).
cnf(eqb5_b0, axiom, eqb5(W1, W2) | ~bit5(W1, b0) | ~bit5(W2, b0)).
cnf(eqb5_b1, axiom, eqb5(W1, W2) | ~bit5(W1, b1) | ~bit5(W2, b1)).
cnf(eqb6_b0, axiom, eqb6(W1, W2) | ~bit6(W1, b0) | ~bit6(W2, b0)).
cnf(eqb6_b1, axiom, eqb6(W1, W2) | ~bit6(W1, b1) | ~bit6(W2, b1)).
cnf(eqb7_b0, axiom, eqb7(W1, W2) | ~bit7(W1, b0) | ~bit7(W2, b0)).
cnf(eqb7_b1, axiom, eqb7(W1, W2) | ~bit7(W1, b1) | ~bit7(W2, b1)).
% --- Row identity: roweq(W1, W2) iff all eight projections match ---
cnf(roweq_f_0, axiom, ~roweq(W1, W2) | eqb0(W1, W2)).
cnf(roweq_f_1, axiom, ~roweq(W1, W2) | eqb1(W1, W2)).
cnf(roweq_f_2, axiom, ~roweq(W1, W2) | eqb2(W1, W2)).
cnf(roweq_f_3, axiom, ~roweq(W1, W2) | eqb3(W1, W2)).
cnf(roweq_f_4, axiom, ~roweq(W1, W2) | eqb4(W1, W2)).
cnf(roweq_f_5, axiom, ~roweq(W1, W2) | eqb5(W1, W2)).
cnf(roweq_f_6, axiom, ~roweq(W1, W2) | eqb6(W1, W2)).
cnf(roweq_f_7, axiom, ~roweq(W1, W2) | eqb7(W1, W2)).
cnf(roweq_b, axiom, roweq(W1, W2) | ~eqb0(W1, W2) | ~eqb1(W1, W2) | ~eqb2(W1, W2) | ~eqb3(W1, W2) | ~eqb4(W1, W2) | ~eqb5(W1, W2) | ~eqb6(W1, W2) | ~eqb7(W1, W2)).
% --- The empty row z: all projections clear ---
cnf(z_def_0, axiom, bit0(z, b0)).
cnf(z_def_1, axiom, bit1(z, b0)).
cnf(z_def_2, axiom, bit2(z, b0)).
cnf(z_def_3, axiom, bit3(z, b0)).
cnf(z_def_4, axiom, bit4(z, b0)).
cnf(z_def_5, axiom, bit5(z, b0)).
cnf(z_def_6, axiom, bit6(z, b0)).
cnf(z_def_7, axiom, bit7(z, b0)).
