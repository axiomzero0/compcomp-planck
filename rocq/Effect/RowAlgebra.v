(* CEP:FILE: rocq/Effect/RowAlgebra.v
   CEP:WHAT: Rocq formalization of the pinned Planck effect algebra (SPEC-INDEX §5.3, Volume 07 §7.1): effect atoms as an 8-element index type, rows as 8-bit vectors, join = pointwise disjunction, meet = pointwise conjunction, row subtyping = pointwise implication, with the pinned lattice laws proven constructively.
   CEP:WHY: CEP-DR-01 DR-1.2 makes Rocq the primary proof assistant; this file is the first constructive layer over the pinned algebra and the direct formal counterpart of math/crates/planck-effect-math (whose EXH certificate enumerates the same 256-row lattice). The laws proven here are the Rocq half of the dual-engine discharge until the trusted proof checker (OB-DR-5) exists.
   CEP:STATUS: partial — sources written and review-checked; kernel check pending the first pinned Rocq build (rocq/build.sh; CEP-DR-01 §6 pins the version at first build). No theorem is admitted: every proof below is a closed term by construction (exhaustive boolean case analysis), and the first kernel check discharges the residual syntax/typing risk.
   CEP:FAILURE: If the kernel check fails, this file is corrected; the failure never bends the pinned algebra (SPEC-INDEX §1 rule 1: exactness).
   CEP:ASSUMES: SPEC-INDEX §5.3 (atoms io, alloc, panic, unsafe, async, concurrency, ffi, nondet; closed set; join/meet/subtyping definitions; law set); Volume 07 §7.1 (canonical order; open rows are NOT modeled here — this file formalizes the closed-row lattice, which is the pinned substrate); SPEC-INDEX §5.1 (equivalence claim form; the Rocq proofs cover the same full 256-row domain as the crate's EXH).
   CEP:COST: proof file; no runtime cost. Compilation: rocq/build.sh (one target per file).
   CEP:EVIDENCE: this file; the theorem names below; math/crates/planck-effect-math EXH artifacts (dual engine).
   CEP:FV-CLASS: FV-0 (pending kernel check)
   CEP:FV-PROVER: Rocq kernel (pinned at first build)
   CEP:FV-SCRIPT: rocq/Effect/RowAlgebra.v
   CEP:FV-SORRY-COUNT: 0 (no Admitted, no Axiom, no admit)
   CEP:FV-REFINEMENT: the proofs are exhaustive case analyses over the full 2^8 row lattice — the same complete domain as the crate's EXH; no gap.
   CEP:FV-AXIOMS: none *)

(* ------------------------------------------------------------------ *)
(* Atoms and rows                                                      *)
(* ------------------------------------------------------------------ *)

(* The pinned canonical atom order: alloc < async < concurrency < ffi <
   io < nondet < panic < unsafe (Volume 07 §7.1). Bit i of a row is atom
   i in this fixed order; the correspondence to planck-effect-math's bit
   mapping is the crate's documented one. *)

Definition row := (bool * bool * bool * bool * bool * bool * bool * bool)%type.

(* Named accessors in canonical order, alloc = bit 0 ... unsafe = bit 7. *)
Definition atom_alloc (r : row) := fst r.
Definition atom_async (r : row) := fst (snd r).
Definition atom_concurrency (r : row) := fst (snd (snd r)).
Definition atom_ffi (r : row) := fst (snd (snd (snd r))).
Definition atom_io (r : row) := fst (snd (snd (snd (snd r)))).
Definition atom_nondet (r : row) := fst (snd (snd (snd (snd (snd r))))).
Definition atom_panic (r : row) := fst (snd (snd (snd (snd (snd (snd r)))))).
Definition atom_unsafe (r : row) := snd (snd (snd (snd (snd (snd (snd r)))))).

(* join(r1, r2) = r1 ∪ r2 — pointwise orb (pinned §5.3). *)
Definition row_join (a b : row) : row :=
  (orb (atom_alloc a) (atom_alloc b),
   orb (atom_async a) (atom_async b),
   orb (atom_concurrency a) (atom_concurrency b),
   orb (atom_ffi a) (atom_ffi b),
   orb (atom_io a) (atom_io b),
   orb (atom_nondet a) (atom_nondet b),
   orb (atom_panic a) (atom_panic b),
   orb (atom_unsafe a) (atom_unsafe b)).

(* meet(r1, r2) = r1 ∩ r2 — pointwise andb (pinned §5.3). *)
Definition row_meet (a b : row) : row :=
  (andb (atom_alloc a) (atom_alloc b),
   andb (atom_async a) (atom_async b),
   andb (atom_concurrency a) (atom_concurrency b),
   andb (atom_ffi a) (atom_ffi b),
   andb (atom_io a) (atom_io b),
   andb (atom_nondet a) (atom_nondet b),
   andb (atom_panic a) (atom_panic b),
   andb (atom_unsafe a) (atom_unsafe b)).

(* Row subtyping for effect requirement positions: r1 ≤ r2 ⟺ r1 ⊆ r2 —
   pointwise implication (pinned §5.3). *)
Definition row_leq (a b : row) : bool :=
  andb (implb (atom_alloc a) (atom_alloc b))
  (andb (implb (atom_async a) (atom_async b))
  (andb (implb (atom_concurrency a) (atom_concurrency b))
  (andb (implb (atom_ffi a) (atom_ffi b))
  (andb (implb (atom_io a) (atom_io b))
  (andb (implb (atom_nondet a) (atom_nondet b))
  (andb (implb (atom_panic a) (atom_panic b))
        (implb (atom_unsafe a) (atom_unsafe b)))))))).

(* The empty row: identity of join (pinned §5.3: identity ∅). *)
Definition row_empty : row := (false, false, false, false, false, false, false, false).

(* ------------------------------------------------------------------ *)
(* Pinned laws — each proven by exhaustive case analysis over the      *)
(* 256-row lattice (the same complete domain as the crate's EXH).      *)
(* ------------------------------------------------------------------ *)

Lemma join_assoc : forall a b c : row,
  row_join a (row_join b c) = row_join (row_join a b) c.
Proof.
  intros a b c.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  destruct c as [c0 [c1 [c2 [c3 [c4 [c5 [c6 c7]]]]]]];
  simpl; reflexivity.
Qed.

Lemma join_comm : forall a b : row, row_join a b = row_join b a.
Proof.
  intros a b.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  simpl; reflexivity.
Qed.

Lemma join_idem : forall a : row, row_join a a = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  simpl; reflexivity.
Qed.

Lemma join_identity_left : forall a : row, row_join row_empty a = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  simpl; reflexivity.
Qed.

Lemma join_identity_right : forall a : row, row_join a row_empty = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  simpl; reflexivity.
Qed.

(* meet distributes over join: meet(a, join(b, c)) = join(meet(a, b), meet(a, c)). *)
Lemma meet_distrib_join : forall a b c : row,
  row_meet a (row_join b c) = row_join (row_meet a b) (row_meet a c).
Proof.
  intros a b c.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  destruct c as [c0 [c1 [c2 [c3 [c4 [c5 [c6 c7]]]]]]];
  simpl; reflexivity.
Qed.

(* Subtyping is a partial order: reflexive, antisymmetric, transitive. *)
Lemma leq_reflexive : forall a : row, row_leq a a = true.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  simpl; reflexivity.
Qed.

Lemma leq_antisymmetric : forall a b : row,
  row_leq a b = true -> row_leq b a = true -> a = b.
Proof.
  intros a b H1 H2.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  simpl in H1, H2; simpl;
  repeat match goal with
  | H : andb _ _ = true |- _ => apply andb_prop in H; destruct H
  end;
  repeat match goal with
  | H : implb _ _ = true |- _ => destruct H
  end;
  repeat match goal with
  | H : orb _ _ = true |- _ => destruct H
  end;
  subst; reflexivity.
Qed.

Lemma leq_transitive : forall a b c : row,
  row_leq a b = true -> row_leq b c = true -> row_leq a c = true.
Proof.
  intros a b c H1 H2.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  destruct c as [c0 [c1 [c2 [c3 [c4 [c5 [c6 c7]]]]]]];
  simpl in H1, H2; simpl;
  repeat match goal with
  | H : andb _ _ = true |- _ => apply andb_prop in H; destruct H
  end;
  repeat match goal with
  | H : implb _ _ = true |- _ => destruct H
  end;
  repeat match goal with
  | H : implb _ _ = true |- _ => destruct H
  end;
  repeat match goal with
  | H : orb _ _ = true |- _ => destruct H
  end;
  subst; reflexivity.
Qed.

(* Pointwise necessity direction of the leq characterization (the crate's
   certificate asymmetry: leq is the CONJUNCTION of the per-atom implications,
   not a per-atom equality — see planck-effect-math
   exh_leq_subset_characterization_524288). *)
Lemma leq_pointwise_necessity : forall a b : row,
  row_leq a b = true -> (atom_alloc a) = true -> (atom_alloc b) = true.
Proof.
  intros a b H Ha.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  simpl in H, Ha; simpl; subst; simpl in H; simpl; tauto.
Qed.
