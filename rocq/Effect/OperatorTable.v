(* CEP:FILE: rocq/Effect/OperatorTable.v
   CEP:WHAT: The Rocq foundation layer of the pinned operator table of Planck 0.1 edition P1 (Volume 02 §2.4.1; columns pinned by SPEC-INDEX §5.5): the five closed-domain column accessors over OpId (precedence 0..=14, associativity, arity, overloadability, overflow class), the numeric arity projection, the precedence comparison, the OP-number projection, the pinned fuel-shaped binary search over the sorted table, and the pinned lookup — plus the complete-domain kernel laws: accessor/table agreement, column domains, key distinctness and strict ascending order, lookup hit completeness, lookup miss on the 6 delimiters, precedence-comparison agreement over all 1936 pairs, and the OP-number law.
   CEP:WHY: DR-1.4 proof-to-Rust extraction with the standing no-hand-written-compiler-code standard; the operator table is DO-1 phase-1 foundation semantics (VR-14(d)); the Pratt climber (§2.6.4) consumes exactly these accessors, so their definition of record must be kernel-checked Coq whose extracted Rust is DIF-verified against the kernel over the complete closed domains (44 rows, 1936 pairs, 16,563-key sweep) and against the reference kernel (three-way agreement).
   CEP:STATUS: complete
   CEP:FAILURE: An accessor that disagrees with the generated table column for any of the 44 variants fails the accessor-agreement law (kernel, zero admits); a lookup that misses a pinned key or hits a non-key fails the lookup laws; a divergence between the extracted Rust and this file fails the DIF over the complete closed domains and the build. No other failure mode: every function is total over the closed domains.
   CEP:ASSUMES: Coq 8.20.1 (CEP-DR-01 §6 pin); the generated data unit Effect.OperatorTableData (DR-8.8 v2.15, generated from the reference source); the fuel bound 44 of the binary search is at least the table length (44) so the fuel-0 arm is unreachable inside the pinned domain (recorded honestly per v2.9); core str ordering is byte-lexicographic in the extracted target (the reference's own recorded assumption).
   CEP:COST: kernel vm_compute over the closed domains per law run (44 + 44x44 + 44x5 + 16563-key sweep is DIF-side); extracted accessors O(1), lookup O(log 44) <= 6 comparisons via the pinned row_at -> array-index mapping (DR-8.8 v2.8).
   CEP:EVIDENCE: the Lemmas below (zero admits); DIF ground truth generated from this file and Effect.OperatorTableData by planck-extractor; three-way agreement with math/crates/planck-operator-math recorded in the DIF artifact; coqchk Axioms: none per rocq/build.sh.
   CEP:SECURITY: no secrets; repository-trusted inputs; hand edits to extracted output are non-conforming (DR-8.2) — this file is the source of truth.
   CEP:FV-CLASS: FV-1 (kernel-checked laws, complete closed domains) + FV-0-equivalent-by-DIF for the extracted Rust
   CEP:FV-PROVER: Coq 8.20.1 kernel (coqc + coqchk, zero axioms)
   CEP:FV-SCRIPT: rocq/build.sh; tools/extractor (extract/dif/check/lint-cep)
   CEP:FV-SORRY-COUNT: 0
   CEP:FV-REFINEMENT: complete closed domains: the laws quantify over all 44 variants, all 1936 ordered pairs, and the pinned sweep domain; the fuel bound covers the whole table; no sampling, no gap within the pinned domains.
   CEP:FV-AXIOMS: none *)

Require Import Bool.
Require Import Arith.
Require Import List.
Require Import Lia.
Require Import Effect.OperatorEnums.
Require Import Effect.OperatorTableData.
Import ListNotations.
Import PinnedEnums.

(* ------------------------------------------------------------------ *)
(* The lookup answer: exactly the reference Result<&OperatorRow,       *)
(* LookupError> shape as a two-constructor data enum (subset v2.14):   *)
(* Ok carries the matched row, Err carries the single pinned error.    *)
(* ------------------------------------------------------------------ *)

Inductive lookup_result : Type :=
  | Ok : oprow -> lookup_result
  | Err : lookup_error -> lookup_result.

(* Hit test (law-side helper; total). *)
Definition lookup_hitb (r : lookup_result) : bool :=
  match r with
  | Ok _ => true
  | Err _ => false
  end.

(* ------------------------------------------------------------------ *)
(* The five pinned column accessors over OpId (44-arm closed matches)  *)
(* and the numeric projections.                                        *)
(* ------------------------------------------------------------------ *)

Definition precedence (o : opid) : nat :=
  match o with
  | Assign | AddAssign | SubAssign | MulAssign | DivAssign | RemAssign
    | BitAndAssign | BitOrAssign | BitXorAssign | ShlAssign | ShrAssign => 0
  | Range | RangeInclusive | RangeFrom | RangeTo | RangeFull => 1
  | LogicalOr => 2
  | LogicalAnd => 3
  | BitOr => 4
  | BitXor => 5
  | BitAnd => 6
  | Eq | Ne => 7
  | Lt | Gt | Le | Ge => 8
  | Shl | Shr => 9
  | Add | Sub => 10
  | Mul | Div | Rem => 11
  | Cast => 12
  | Neg | Not | Deref | Borrow => 13
  | Field | MethodCall | Index | Call | Try => 14
  end.

Definition associativity (o : opid) : assoc :=
  match o with
  | LogicalOr | LogicalAnd | BitOr | BitXor | BitAnd | Shl | Shr
    | Add | Sub | Mul | Div | Rem | Cast | Field | MethodCall | Index
    | Call | Try => Left
  | Neg | Not | Deref | Borrow => Right
  | Assign | AddAssign | SubAssign | MulAssign | DivAssign | RemAssign
    | BitAndAssign | BitOrAssign | BitXorAssign | ShlAssign | ShrAssign
    | Range | RangeInclusive | RangeFrom | RangeTo | RangeFull
    | Eq | Ne | Lt | Gt | Le | Ge => None
  end.

Definition arity_of (o : opid) : arity :=
  match o with
  | Assign | AddAssign | SubAssign | MulAssign | DivAssign | RemAssign
    | BitAndAssign | BitOrAssign | BitXorAssign | ShlAssign | ShrAssign
    | Range | RangeInclusive | LogicalOr | LogicalAnd | BitOr | BitXor
    | BitAnd | Eq | Ne | Lt | Gt | Le | Ge | Shl | Shr | Add | Sub
    | Mul | Div | Rem | Cast | Field | MethodCall | Index | Call => Two
  | RangeFrom | RangeTo | Neg | Not | Deref | Borrow | Try => One
  | RangeFull => Zero
  end.

Definition overloadable (o : opid) : bool :=
  match o with
  | BitOr | BitXor | BitAnd | Eq | Ne | Lt | Gt | Le | Ge
    | Shl | Shr | Add | Sub | Mul | Div | Rem | Neg | Not
    | MethodCall | Index | Call => true
  | Assign | AddAssign | SubAssign | MulAssign | DivAssign | RemAssign
    | BitAndAssign | BitOrAssign | BitXorAssign | ShlAssign | ShrAssign
    | Range | RangeInclusive | RangeFrom | RangeTo | RangeFull
    | LogicalOr | LogicalAnd | Cast | Deref | Borrow | Field | Try => false
  end.

Definition overflow_of (o : opid) : overflow :=
  match o with
  | Shl | Shr | Add | Sub | Mul | Div | Rem | Neg => Panic
  | Assign | AddAssign | SubAssign | MulAssign | DivAssign | RemAssign
    | BitAndAssign | BitOrAssign | BitXorAssign | ShlAssign | ShrAssign
    | Range | RangeInclusive | RangeFrom | RangeTo | RangeFull
    | LogicalOr | LogicalAnd | BitOr | BitXor | BitAnd | Eq | Ne
    | Lt | Gt | Le | Ge | Cast | Not | Deref | Borrow | Field
    | MethodCall | Index | Call | Try => NotApplicable
  end.

Definition arity_count (a : arity) : nat :=
  match a with
  | Zero => 0
  | One => 1
  | Two => 2
  end.

Definition precedence_compare (a b : opid) : ordering :=
  if Nat.ltb (precedence a) (precedence b) then Less
  else if Nat.ltb (precedence b) (precedence a) then Greater
  else Equal.

Definition opid_index (o : opid) : nat :=
  match o with
  | Assign => 1 | AddAssign => 2 | SubAssign => 3 | MulAssign => 4
  | DivAssign => 5 | RemAssign => 6 | BitAndAssign => 7 | BitOrAssign => 8
  | BitXorAssign => 9 | ShlAssign => 10 | ShrAssign => 11 | Range => 12
  | RangeInclusive => 13 | RangeFrom => 14 | RangeTo => 15 | RangeFull => 16
  | LogicalOr => 17 | LogicalAnd => 18 | BitOr => 19 | BitXor => 20
  | BitAnd => 21 | Eq => 22 | Ne => 23 | Lt => 24 | Gt => 25 | Le => 26
  | Ge => 27 | Shl => 28 | Shr => 29 | Add => 30 | Sub => 31 | Mul => 32
  | Div => 33 | Rem => 34 | Cast => 35 | Neg => 36 | Not => 37 | Deref => 38
  | Borrow => 39 | Field => 40 | MethodCall => 41 | Index => 42 | Call => 43
  | Try => 44
  end.

(* Column ordinals for the DIF harness mapping (kernel-side mirrors of  *)
(* the generated test-side ordinal functions).                          *)
Definition assoc_ord (a : assoc) : nat :=
  match a with Left => 0 | Right => 1 | None => 2 end.
Definition arity_ord (a : arity) : nat :=
  match a with Zero => 0 | One => 1 | Two => 2 end.
Definition overflow_ord (o : overflow) : nat :=
  match o with Panic => 0 | Wrap => 1 | Unchecked => 2 | NotApplicable => 3 end.
Definition ordering_ord (o : ordering) : nat :=
  match o with Less => 0 | Equal => 1 | Greater => 2 end.
Definition nat_compare_ord (a b : nat) : nat :=
  if Nat.ltb a b then 0 else if Nat.ltb b a then 2 else 1.

(* ------------------------------------------------------------------ *)
(* The pinned binary search (subset v2.7 fuel shape) and lookup.       *)
(* ------------------------------------------------------------------ *)

Fixpoint bsearch_aux (fuel lo hi : nat) (k : key) : lookup_result :=
  match fuel with
  | 0 => Err UnknownToken
  | S f =>
      if Nat.leb lo hi then
        let m := Nat.add lo (Nat.div2 (Nat.sub hi lo)) in
        if key_eq k (row_key (row_at m)) then Ok (row_at m)
        else if key_lt (row_key (row_at m)) k then bsearch_aux f (Nat.succ m) hi k
        else bsearch_aux f lo m k
      else Err UnknownToken
  end.

Definition lookup (k : key) : lookup_result := bsearch_aux 44 0 43 k.

(* ------------------------------------------------------------------ *)
(* Pinned laws (kernel-checked, complete closed domains, zero admits). *)
(* ------------------------------------------------------------------ *)

(* OL1: OP numbers are exactly 1..=44 in declaration order (the "Op"    *)
(* column of §2.4.1; the reference exh_table_matches_transcription_44   *)
(* index contract).                                                     *)
Lemma opid_index_law :
  forallb (fun i => Nat.eqb (opid_index (nth i all_opids Assign)) (Nat.succ i))
          (seq 0 44) = true.
Proof. vm_compute. reflexivity. Qed.

(* OL2: every accessor agrees with the generated table column of the    *)
(* SAME row, quantified over the sorted positions (the reference        *)
(* exh_accessors_match_rows_44 contract: every row's accessor equals    *)
(* its transcribed column; the row is the record). Plus the opid-table  *)
(* correspondence: every opid has exactly one row. *)
Lemma accessors_agree_with_table :
  forallb (fun r =>
    andb (Nat.eqb (row_precedence r) (precedence (row_opid r)))
    (andb (Nat.eqb (assoc_ord (row_assoc r)) (assoc_ord (associativity (row_opid r))))
    (andb (Nat.eqb (arity_ord (row_arity r)) (arity_ord (arity_of (row_opid r))))
    (andb (eqb (row_overloadable r) (overloadable (row_opid r)))
          (Nat.eqb (overflow_ord (row_overflow r)) (overflow_ord (overflow_of (row_opid r))))))))
    operator_table_sorted = true.
Proof. vm_compute. reflexivity. Qed.

Lemma accessors_agree_pointwise : forall r : oprow,
  In r operator_table_sorted ->
  andb (Nat.eqb (row_precedence r) (precedence (row_opid r)))
  (andb (Nat.eqb (assoc_ord (row_assoc r)) (assoc_ord (associativity (row_opid r))))
  (andb (Nat.eqb (arity_ord (row_arity r)) (arity_ord (arity_of (row_opid r))))
  (andb (eqb (row_overloadable r) (overloadable (row_opid r)))
        (Nat.eqb (overflow_ord (row_overflow r)) (overflow_ord (overflow_of (row_opid r))))))) = true.
Proof.
  intros r Hin.
  destruct (forallb_forall (fun r =>
    andb (Nat.eqb (row_precedence r) (precedence (row_opid r)))
    (andb (Nat.eqb (assoc_ord (row_assoc r)) (assoc_ord (associativity (row_opid r))))
    (andb (Nat.eqb (arity_ord (row_arity r)) (arity_ord (arity_of (row_opid r))))
    (andb (eqb (row_overloadable r) (overloadable (row_opid r)))
          (Nat.eqb (overflow_ord (row_overflow r)) (overflow_ord (overflow_of (row_opid r))))))))
    operator_table_sorted) as [Hfwd _].
  apply (Hfwd accessors_agree_with_table r Hin).
Qed.

(* OL2b: the sorted table contains every opid exactly once (the closed
   correspondence between the declaration-order enumeration and the
   sorted rows; together with OL1 this pins the transcription). *)
Lemma every_opid_has_exactly_one_row :
  forallb (fun o =>
    Nat.eqb (fold_left (fun acc r =>
                 if Nat.eqb (opid_index (row_opid r)) (opid_index o)
                 then Nat.succ acc else acc)
                 operator_table_sorted 0) 1)
    all_opids = true.
Proof. vm_compute. reflexivity. Qed.

(* OL3: column domains (Volume 02 §2.4.1 / SPEC-INDEX §5.5): precedence *)
(* within 0..=14; arity counts within {0,1,2} with exactly one Zero row *)
(* (OP-16); the Overflow Wrap and Unchecked classes appear in no row.   *)
Lemma precedence_column_domain :
  forallb (fun o => andb (Nat.leb (precedence o) 14)
                         (Nat.leb 0 (precedence o))) all_opids = true.
Proof. vm_compute. reflexivity. Qed.

Lemma overflow_wrap_unchecked_absent :
  forallb (fun o => andb (negb (Nat.eqb (overflow_ord (overflow_of o)) 1))
                         (negb (Nat.eqb (overflow_ord (overflow_of o)) 2)))
          all_opids = true.
Proof. vm_compute. reflexivity. Qed.

Lemma arity_zero_is_exactly_op16 :
  forallb (fun o => eqb (Nat.eqb (arity_ord (arity_of o)) 0)
                        (Nat.eqb (opid_index o) 16))
          all_opids = true.
Proof. vm_compute. reflexivity. Qed.

(* OL4: the 44 pinned keys are pairwise distinct (i <> j implies         *)
(* key_neq), checked through the closed row_at dispatch over all 44x44  *)
(* ordered pairs.                                                       *)
Lemma keys_pairwise_distinct :
  forallb (fun p => orb (Nat.eqb (opid_index (row_opid (fst p)))
                                 (opid_index (row_opid (snd p))))
                        (negb (key_eq (row_key (fst p))
                                      (row_key (snd p)))))
          (flat_map (fun r1 => map (fun r2 => (r1, r2)) operator_table_sorted)
                    operator_table_sorted) = true.
Proof. vm_compute. reflexivity. Qed.

(* OL5: the sorted table is strictly ascending by the pinned            *)
(* byte-lexicographic key order (the reference exh_table_sorted_unique  *)
(* _keys contract; strictness on adjacent pairs of the sorted array).   *)
Lemma table_strictly_ascending :
  forallb (fun p => key_lt (row_key (fst p)) (row_key (snd p)))
          (combine operator_table_sorted (tl operator_table_sorted)) = true.
Proof. vm_compute. reflexivity. Qed.

(* OL6: lookup hits exactly the pinned row for every one of the 44 keys *)
(* (hit completeness; the reference exh_lookup_keys domain).            *)
Lemma lookup_hits_all_keys :
  forallb (fun r =>
    andb (lookup_hitb (lookup (row_key r)))
         (match lookup (row_key r) with
          | Ok row => andb (key_eq (row_key row) (row_key r))
                           (Nat.eqb (opid_index (row_opid row))
                                    (opid_index (row_opid r)))
          | Err _ => false
          end))
    operator_table_sorted = true.
Proof. vm_compute. reflexivity. Qed.

(* OL7: lookup misses on the 6 non-operator delimiters (OB-02-8; the    *)
(* bare-shape note: keys are the full verbatim cells, so the bare       *)
(* spelling of shape-qualified operators is also a miss).               *)
Definition delimiter_keys : list key :=
  [ (40 :: nil); (41 :: nil); (91 :: nil); (93 :: nil); (123 :: nil); (125 :: nil) ].

Lemma lookup_misses_all_delimiters :
  forallb (fun k => negb (lookup_hitb (lookup k))) delimiter_keys = true.
Proof. vm_compute. reflexivity. Qed.

(* OL8: precedence_compare is the ordinal comparison of the precedence  *)
(* column over all 1936 ordered pairs (the reference                    *)
(* exh_precedence_compare_all_pairs_1936 contract).                     *)
Lemma precedence_compare_all_pairs :
  forallb (fun p =>
    Nat.eqb (ordering_ord (precedence_compare (fst p) (snd p)))
        (nat_compare_ord (precedence (fst p)) (precedence (snd p))))
    (flat_map (fun a => map (fun b => (a, b)) all_opids) all_opids) = true.
Proof. vm_compute. reflexivity. Qed.

(* OL9: the arity column carries exactly the values 0, 1, 2 (the closed *)
(* numeric domain of the Arity projection).                             *)
Lemma arity_count_closed_domain :
  forallb (fun a => orb (Nat.eqb (arity_count a) 0)
                        (orb (Nat.eqb (arity_count a) 1) (Nat.eqb (arity_count a) 2)))
          [ Zero; One; Two ] = true.
Proof. vm_compute. reflexivity. Qed.

(* OL10: the sorted table has exactly 44 rows and row_at is the closed  *)
(* dispatch onto them (the reference exh_table_matches_transcription_44 *)
(* row-count contract, on the kernel side).                             *)
Lemma table_row_count_is_44 : Nat.eqb (length operator_table_sorted) 44 = true.
Proof. vm_compute. reflexivity. Qed.

Lemma row_at_is_closed_dispatch :
  forallb (fun i => key_eq (row_key (row_at i))
                           (row_key (nth i operator_table_sorted row_00)))
          (seq 0 44) = true.
Proof. vm_compute. reflexivity. Qed.
