(* GENERATED FILE - DO NOT EDIT. Generated mechanically from
   math/crates/planck-operator-math/src/lib.rs (the reference kernel,
   EXH-certified against Volume 02 §2.4.1) by the pinned transcription
   procedure of CEP-DR-01 §8 DR-8.8 v2.15; regenerate with
   `planck-extractor transcribe`. Hand edits are non-conforming.
   CEP:FILE: rocq/Effect/OperatorTableData.v
   CEP:PROVENANCE: source math/crates/planck-operator-math/src/lib.rs (sha256 3ac181dc1d2db93e0b635942383fcc22df69b305e8b490df24871f5a51036e76); generated mechanically by the DR-8.8 v2.15 transcription procedure — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor transcribe`.
   CEP:WHAT: The 44 pinned §2.4.1 operator rows as kernel-checked Coq data: the ASCII key definitions (subset v2.6 list-nat literals), the right-nested 7-tuple rows (key, opid, precedence, assoc, arity, overloadable, overflow), the sorted table, the closed row_at dispatch, and the seven mechanical row projections.
   CEP:WHY: The operator-table foundation domain (VR-14(d), DO-1 phase 1) needs its data in the proof layer so the accessor-agreement, distinctness, sortedness, and lookup laws are kernel-checked and the extracted Rust is DIF-verified against this data; generating from the reference source closes the transcription loop mechanically.
   CEP:STATUS: complete (generated)
   CEP:FAILURE: A row that disagrees with the reference table fails the three-way DIF (extracted vs kernel vs reference) over the closed 44-row domain; regeneration is the only edit path.
   CEP:ASSUMES: the reference table is the definition of record (EXH-certified to Volume 02 §2.4.1); token cells are printable ASCII; the array order is the sorted order (asserted mechanically at generation).
   CEP:COST: generation build-time only; the data is const.
   CEP:EVIDENCE: this file; DIF ground truth generated from it by planck-extractor; the reference-axis tables carry the same sha256 provenance.
   CEP:SECURITY: repository-trusted inputs; generated mechanically, hand edits are non-conforming.
   CEP:FV-CLASS: FV-2 (generated kernel-checked data; qualified by the DIF it feeds)
   CEP:FV-PROVER: Coq 8.20.1 kernel (coqc + coqchk, zero axioms)
   CEP:FV-SCRIPT: planck-extractor transcribe (bootstrap: tools/extractor/bootstrap/gen_operator_data.py)
   CEP:FV-SORRY-COUNT: N/A
   CEP:FV-REFINEMENT: the row domain is closed (44 rows); the DIF covers all of it; no sampling.
   CEP:FV-AXIOMS: none *)

Require Import Bool.
Require Import Arith.
Require Import List.
Require Import Effect.OperatorEnums.
Import ListNotations.
Import PinnedEnums.

(* The row record: the six pinned columns plus the identity and the key. *)
Definition oprow : Type :=
  (key * (opid * (nat * (assoc * (arity * (bool * overflow)))))).

(* The 44 pinned keys (ASCII code points, subset v2.6). *)
Definition row_00_key : key := [33; 32; 40; 117; 110; 97; 114; 121; 41].
Definition row_01_key : key := [33; 61].
Definition row_02_key : key := [37].
Definition row_03_key : key := [37; 61].
Definition row_04_key : key := [38; 32; 40; 98; 105; 110; 97; 114; 121; 41].
Definition row_05_key : key := [38; 32; 47; 32; 38; 109; 117; 116; 32; 40; 98; 111; 114; 114; 111; 119; 41].
Definition row_06_key : key := [38; 38].
Definition row_07_key : key := [38; 61].
Definition row_08_key : key := [40; 97; 114; 103; 115; 41; 32; 99; 97; 108; 108].
Definition row_09_key : key := [42; 32; 40; 98; 105; 110; 97; 114; 121; 41].
Definition row_10_key : key := [42; 32; 40; 100; 101; 114; 101; 102; 41].
Definition row_11_key : key := [42; 61].
Definition row_12_key : key := [43].
Definition row_13_key : key := [43; 61].
Definition row_14_key : key := [45; 32; 40; 98; 105; 110; 97; 114; 121; 41].
Definition row_15_key : key := [45; 32; 40; 117; 110; 97; 114; 121; 41].
Definition row_16_key : key := [45; 61].
Definition row_17_key : key := [46; 32; 102; 105; 101; 108; 100].
Definition row_18_key : key := [46; 46].
Definition row_19_key : key := [46; 46; 98].
Definition row_20_key : key := [46; 110; 97; 109; 101; 40; 97; 114; 103; 115; 41; 32; 109; 101; 116; 104; 111; 100; 32; 99; 97; 108; 108].
Definition row_21_key : key := [47].
Definition row_22_key : key := [47; 61].
Definition row_23_key : key := [60].
Definition row_24_key : key := [60; 60].
Definition row_25_key : key := [60; 60; 61].
Definition row_26_key : key := [60; 61].
Definition row_27_key : key := [61].
Definition row_28_key : key := [61; 61].
Definition row_29_key : key := [62].
Definition row_30_key : key := [62; 61].
Definition row_31_key : key := [62; 62].
Definition row_32_key : key := [62; 62; 61].
Definition row_33_key : key := [63].
Definition row_34_key : key := [91; 105; 93; 32; 105; 110; 100; 101; 120].
Definition row_35_key : key := [94; 32; 40; 98; 105; 110; 97; 114; 121; 41].
Definition row_36_key : key := [94; 61].
Definition row_37_key : key := [97; 46; 46].
Definition row_38_key : key := [97; 46; 46; 61; 98].
Definition row_39_key : key := [97; 46; 46; 98].
Definition row_40_key : key := [97; 115].
Definition row_41_key : key := [124; 32; 40; 98; 105; 110; 97; 114; 121; 41].
Definition row_42_key : key := [124; 61].
Definition row_43_key : key := [124; 124].

(* The 44 pinned rows (Volume 02 §2.4.1, sorted order). *)
Definition row_00 : oprow :=
  (row_00_key, (Not, (13, (Right, (One, (true, NotApplicable)))))).
Definition row_01 : oprow :=
  (row_01_key, (Ne, (7, (None, (Two, (true, NotApplicable)))))).
Definition row_02 : oprow :=
  (row_02_key, (Rem, (11, (Left, (Two, (true, Panic)))))).
Definition row_03 : oprow :=
  (row_03_key, (RemAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_04 : oprow :=
  (row_04_key, (BitAnd, (6, (Left, (Two, (true, NotApplicable)))))).
Definition row_05 : oprow :=
  (row_05_key, (Borrow, (13, (Right, (One, (false, NotApplicable)))))).
Definition row_06 : oprow :=
  (row_06_key, (LogicalAnd, (3, (Left, (Two, (false, NotApplicable)))))).
Definition row_07 : oprow :=
  (row_07_key, (BitAndAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_08 : oprow :=
  (row_08_key, (Call, (14, (Left, (Two, (true, NotApplicable)))))).
Definition row_09 : oprow :=
  (row_09_key, (Mul, (11, (Left, (Two, (true, Panic)))))).
Definition row_10 : oprow :=
  (row_10_key, (Deref, (13, (Right, (One, (false, NotApplicable)))))).
Definition row_11 : oprow :=
  (row_11_key, (MulAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_12 : oprow :=
  (row_12_key, (Add, (10, (Left, (Two, (true, Panic)))))).
Definition row_13 : oprow :=
  (row_13_key, (AddAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_14 : oprow :=
  (row_14_key, (Sub, (10, (Left, (Two, (true, Panic)))))).
Definition row_15 : oprow :=
  (row_15_key, (Neg, (13, (Right, (One, (true, Panic)))))).
Definition row_16 : oprow :=
  (row_16_key, (SubAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_17 : oprow :=
  (row_17_key, (Field, (14, (Left, (Two, (false, NotApplicable)))))).
Definition row_18 : oprow :=
  (row_18_key, (RangeFull, (1, (None, (Zero, (false, NotApplicable)))))).
Definition row_19 : oprow :=
  (row_19_key, (RangeTo, (1, (None, (One, (false, NotApplicable)))))).
Definition row_20 : oprow :=
  (row_20_key, (MethodCall, (14, (Left, (Two, (true, NotApplicable)))))).
Definition row_21 : oprow :=
  (row_21_key, (Div, (11, (Left, (Two, (true, Panic)))))).
Definition row_22 : oprow :=
  (row_22_key, (DivAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_23 : oprow :=
  (row_23_key, (Lt, (8, (None, (Two, (true, NotApplicable)))))).
Definition row_24 : oprow :=
  (row_24_key, (Shl, (9, (Left, (Two, (true, Panic)))))).
Definition row_25 : oprow :=
  (row_25_key, (ShlAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_26 : oprow :=
  (row_26_key, (Le, (8, (None, (Two, (true, NotApplicable)))))).
Definition row_27 : oprow :=
  (row_27_key, (Assign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_28 : oprow :=
  (row_28_key, (Eq, (7, (None, (Two, (true, NotApplicable)))))).
Definition row_29 : oprow :=
  (row_29_key, (Gt, (8, (None, (Two, (true, NotApplicable)))))).
Definition row_30 : oprow :=
  (row_30_key, (Ge, (8, (None, (Two, (true, NotApplicable)))))).
Definition row_31 : oprow :=
  (row_31_key, (Shr, (9, (Left, (Two, (true, Panic)))))).
Definition row_32 : oprow :=
  (row_32_key, (ShrAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_33 : oprow :=
  (row_33_key, (Try, (14, (Left, (One, (false, NotApplicable)))))).
Definition row_34 : oprow :=
  (row_34_key, (Index, (14, (Left, (Two, (true, NotApplicable)))))).
Definition row_35 : oprow :=
  (row_35_key, (BitXor, (5, (Left, (Two, (true, NotApplicable)))))).
Definition row_36 : oprow :=
  (row_36_key, (BitXorAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_37 : oprow :=
  (row_37_key, (RangeFrom, (1, (None, (One, (false, NotApplicable)))))).
Definition row_38 : oprow :=
  (row_38_key, (RangeInclusive, (1, (None, (Two, (false, NotApplicable)))))).
Definition row_39 : oprow :=
  (row_39_key, (Range, (1, (None, (Two, (false, NotApplicable)))))).
Definition row_40 : oprow :=
  (row_40_key, (Cast, (12, (Left, (Two, (false, NotApplicable)))))).
Definition row_41 : oprow :=
  (row_41_key, (BitOr, (4, (Left, (Two, (true, NotApplicable)))))).
Definition row_42 : oprow :=
  (row_42_key, (BitOrAssign, (0, (None, (Two, (false, NotApplicable)))))).
Definition row_43 : oprow :=
  (row_43_key, (LogicalOr, (2, (Left, (Two, (false, NotApplicable)))))).

(* The seven mechanical row projections. *)
Definition row_key (r : oprow) : key := fst r.
Definition row_opid (r : oprow) : opid := fst (snd r).
Definition row_precedence (r : oprow) : nat := fst (snd (snd r)).
Definition row_assoc (r : oprow) : assoc := fst (snd (snd (snd r))).
Definition row_arity (r : oprow) : arity := fst (snd (snd (snd (snd r)))).
Definition row_overloadable (r : oprow) : bool :=
  fst (snd (snd (snd (snd (snd r))))).
Definition row_overflow (r : oprow) : overflow :=
  snd (snd (snd (snd (snd (snd r))))).

(* The closed row_at dispatch: index -> row, total by the pinned final arm. *)
(* Kernel-side shape: S-nested nat match (single O(i) walk); the extractor  *)
(* emits the pinned row_at -> array-index mapping instead of this body. *)
Definition row_at (i : nat) : oprow :=
  match i with
  | 0 => row_00
  | S (0) => row_01
  | S (S (0)) => row_02
  | S (S (S (0))) => row_03
  | S (S (S (S (0)))) => row_04
  | S (S (S (S (S (0))))) => row_05
  | S (S (S (S (S (S (0)))))) => row_06
  | S (S (S (S (S (S (S (0))))))) => row_07
  | S (S (S (S (S (S (S (S (0)))))))) => row_08
  | S (S (S (S (S (S (S (S (S (0))))))))) => row_09
  | S (S (S (S (S (S (S (S (S (S (0)))))))))) => row_10
  | S (S (S (S (S (S (S (S (S (S (S (0))))))))))) => row_11
  | S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))) => row_12
  | S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))) => row_13
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))) => row_14
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))) => row_15
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))) => row_16
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))) => row_17
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))) => row_18
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))) => row_19
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))) => row_20
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))) => row_21
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))) => row_22
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))) => row_23
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))) => row_24
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))) => row_25
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))) => row_26
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))) => row_27
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))) => row_28
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))) => row_29
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))) => row_30
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))) => row_31
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))) => row_32
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))) => row_33
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))))) => row_34
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))))) => row_35
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))))))) => row_36
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))))))) => row_37
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))))))))) => row_38
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))))))))) => row_39
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))))))))))) => row_40
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))))))))))) => row_41
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0)))))))))))))))))))))))))))))))))))))))))) => row_42
  | S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (S (0))))))))))))))))))))))))))))))))))))))))))) => row_43
  | _ => row_43
  end.

(* The closed 44-variant enumeration in volume row order (declaration order). *)
Definition all_opids : list opid := [Assign; AddAssign; SubAssign; MulAssign; DivAssign; RemAssign; BitAndAssign; BitOrAssign; BitXorAssign; ShlAssign; ShrAssign; Range; RangeInclusive; RangeFrom; RangeTo; RangeFull; LogicalOr; LogicalAnd; BitOr; BitXor; BitAnd; Eq; Ne; Lt; Gt; Le; Ge; Shl; Shr; Add; Sub; Mul; Div; Rem; Cast; Neg; Not; Deref; Borrow; Field; MethodCall; Index; Call; Try].

(* The pinned table: the 44 rows in sorted order (the lookup array). *)
Definition operator_table_sorted : list oprow :=
  [row_00; row_01; row_02; row_03; row_04; row_05; row_06; row_07; row_08; row_09; row_10; row_11; row_12; row_13; row_14; row_15; row_16; row_17; row_18; row_19; row_20; row_21; row_22; row_23; row_24; row_25; row_26; row_27; row_28; row_29; row_30; row_31; row_32; row_33; row_34; row_35; row_36; row_37; row_38; row_39; row_40; row_41; row_42; row_43].
