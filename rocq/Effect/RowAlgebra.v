(* CEP:FILE: rocq/Effect/RowAlgebra.v
   CEP:WHAT: Rocq formalization of the pinned Planck effect algebra (SPEC-INDEX §5.3, Volume 07 §7.1): effect atoms as an 8-element index type, rows as 8-bit vectors, join = pointwise disjunction, meet = pointwise conjunction, row subtyping = pointwise implication, with the pinned lattice laws proven constructively.
   CEP:WHY: CEP-DR-01 DR-1.2 makes Rocq the primary proof assistant; this file is the first constructive layer over the pinned algebra and the direct formal counterpart of math/crates/planck-effect-math (whose EXH certificate enumerates the same 256-row lattice). The laws proven here are the Rocq half of the dual-engine discharge until the trusted proof checker (OB-DR-5) exists.
   CEP:STATUS: complete — kernel-checked by the pinned prover (Coq 8.20.1, the first pinned build; CEP-DR-01 §6) via rocq/build.sh, which compiles this file to Effect/RowAlgebra.vo and re-verifies it with the standalone checker coqchk (axioms: none; type-in-type: none; unsafe fixpoints: none; assumed positivity: none). No theorem is admitted: every proof below is a closed term (exhaustive boolean case analysis or reduction), and the kernel check discharges the residual syntax/typing risk. The first build surfaced four real proof defects, all fixed here: (a) Coq's tuple notation and `*` are left-associative, so the row type and its constructors are written right-nested explicitly; (b) `reflexivity` alone cannot close pointwise orb/andb laws, so boolean associativity/commutativity helpers are proven first; (c) `simpl` refuses to unfold fst/snd chains, so row_ext uses unfold+cbn; (d) the andb-chain peel must chain on the peeled tail, not the original hypothesis.
   CEP:FAILURE: If the kernel check fails, this file is corrected; the failure never bends the pinned algebra (SPEC-INDEX §1 rule 1: exactness).
   CEP:ASSUMES: SPEC-INDEX §5.3 (atoms io, alloc, panic, unsafe, async, concurrency, ffi, nondet; closed set; join/meet/subtyping definitions; law set); Volume 07 §7.1 (canonical order; open rows are NOT modeled here — this file formalizes the closed-row lattice, which is the pinned substrate); SPEC-INDEX §5.1 (equivalence claim form; the Rocq proofs cover the same full 256-row domain as the crate's EXH).
   CEP:COST: proof file; no runtime cost. Compilation: rocq/build.sh (one target per file).
   CEP:EVIDENCE: this file; the theorem names below; math/crates/planck-effect-math EXH artifacts (dual engine); the DIF ground truth extract/planck_effect/ground_truth/ (kernel vm_compute).
   CEP:SECURITY: no secrets; repository-trusted inputs; the extraction consumer (planck-extractor) parses only this file and the pinned subset.
   CEP:FV-CLASS: FV-0 (kernel-checked; zero gaps)
   CEP:FV-PROVER: Coq 8.20.1 (pinned; CEP-DR-01 §6) — kernel + coqchk
   CEP:FV-SCRIPT: rocq/build.sh (coqc -Q Effect Effect Effect/RowAlgebra.v && coqchk -Q Effect Effect -o Effect.RowAlgebra)
   CEP:FV-SORRY-COUNT: 0 (no Admitted, no Axiom, no admit)
   CEP:FV-REFINEMENT: the proofs are exhaustive case analyses over the full 2^8 row lattice — the same complete domain as the crate's EXH; no gap.
   CEP:FV-AXIOMS: none *)

(* ------------------------------------------------------------------ *)
(* Atoms and rows                                                      *)
(* ------------------------------------------------------------------ *)

(* implb and andb_prop live in the Bool library, not the auto-loaded
   prelude (kernel-check finding, vote round 1). *)
Require Import Bool.

(* The pinned canonical atom order: alloc < async < concurrency < ffi <
   io < nondet < panic < unsafe (Volume 07 §7.1). Bit i of a row is atom
   i in THIS canonical order; this is deliberately distinct from
   planck-effect-math's storage bit order (Volume 12 §12.1: bit0 io ...
   bit7 nondet) — the proven laws are permutation-invariant, and the
   crate documents the same distinction. *)

(* Right-nested explicitly: Coq's `*` notation is left-associative, so a
   flat `bool * ... * bool` would parse left-nested and the fst/snd
   accessors below (right-nested by construction) would not type-check
   (kernel-check finding, first pinned build). *)
Definition row : Type :=
  (bool * (bool * (bool * (bool * (bool * (bool * (bool * bool)))))))%type.

(* Named accessors in canonical order, alloc = bit 0 ... unsafe = bit 7. *)
Definition atom_alloc (r : row) : bool := fst r.
Definition atom_async (r : row) : bool := fst (snd r).
Definition atom_concurrency (r : row) : bool := fst (snd (snd r)).
Definition atom_ffi (r : row) : bool := fst (snd (snd (snd r))).
Definition atom_io (r : row) : bool := fst (snd (snd (snd (snd r)))).
Definition atom_nondet (r : row) : bool := fst (snd (snd (snd (snd (snd r))))).
Definition atom_panic (r : row) : bool := fst (snd (snd (snd (snd (snd (snd r)))))).
Definition atom_unsafe (r : row) : bool := snd (snd (snd (snd (snd (snd (snd r)))))).

(* join(r1, r2) = r1 ∪ r2 — pointwise orb (pinned §5.3).
   Right-nested pairs are written with explicit inner parentheses:
   Coq's comma notation is left-associative, so the flat form would
   build the mirrored tuple (kernel-check finding, first pinned build). *)
Definition row_join (a b : row) : row :=
  (orb (atom_alloc a) (atom_alloc b),
   (orb (atom_async a) (atom_async b),
    (orb (atom_concurrency a) (atom_concurrency b),
     (orb (atom_ffi a) (atom_ffi b),
      (orb (atom_io a) (atom_io b),
       (orb (atom_nondet a) (atom_nondet b),
        (orb (atom_panic a) (atom_panic b),
             orb (atom_unsafe a) (atom_unsafe b)))))))).

(* meet(r1, r2) = r1 ∩ r2 — pointwise andb (pinned §5.3); right-nested as above. *)
Definition row_meet (a b : row) : row :=
  (andb (atom_alloc a) (atom_alloc b),
   (andb (atom_async a) (atom_async b),
    (andb (atom_concurrency a) (atom_concurrency b),
     (andb (atom_ffi a) (atom_ffi b),
      (andb (atom_io a) (atom_io b),
       (andb (atom_nondet a) (atom_nondet b),
        (andb (atom_panic a) (atom_panic b),
             andb (atom_unsafe a) (atom_unsafe b)))))))).

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

(* The empty row: identity of join (pinned §5.3: identity ∅); right-nested as above. *)
Definition row_empty : row :=
  (false, (false, (false, (false, (false, (false, (false, false))))))).

(* ------------------------------------------------------------------ *)
(* ------------------------------------------------------------------ *)
(* Boolean layer — each helper closed by complete case analysis.        *)
(* Kernel-check findings of the first pinned build (Coq 8.20.1):        *)
(*   (a) Coq's tuple notation and `*` are left-associative; the row     *)
(*       type and its constructors are written right-nested explicitly. *)
(*   (b) `reflexivity` alone cannot close pointwise orb/andb laws —     *)
(*       boolean associativity/commutativity helpers are proven first.  *)
(* Extraction contract (CEP-DR-01 DR-1.5, method DIF): the computational *)
(* content of this file — row, the eight accessors, row_join, row_meet,  *)
(* row_leq, row_empty — is the extraction source; every Lemma is Prop-   *)
(* erased and never enters the generated Rust. The extracted artifact is *)
(* separated under extract/ and is never hand-edited.                    *)
(* ------------------------------------------------------------------ *)

Lemma orb_assoc_b : forall x y z : bool, (x || y) || z = x || (y || z).
Proof. intros [] [] []; reflexivity. Qed.

Lemma orb_comm_b : forall x y : bool, (x || y) = (y || x).
Proof. intros [] []; reflexivity. Qed.

Lemma orb_idem_b : forall x : bool, (x || x) = x.
Proof. intros []; reflexivity. Qed.

Lemma orb_false_l_b : forall x : bool, (false || x) = x.
Proof. intros []; reflexivity. Qed.

Lemma orb_false_r_b : forall x : bool, (x || false) = x.
Proof. intros []; reflexivity. Qed.

Lemma andb_distr_orb_b : forall x y z : bool,
  x && (y || z) = (x && y) || (x && z).
Proof. intros [] [] []; reflexivity. Qed.

Lemma andb_true_peel : forall x y : bool,
  x && y = true -> x = true /\ y = true.
Proof. intros [] [] H; simpl in H; try discriminate; split; reflexivity. Qed.

Lemma implb_refl_b : forall x : bool, implb x x = true.
Proof. intros []; reflexivity. Qed.

(* implb x y = true is exactly the propositional implication x = true -> y = true
   — stated as two directional lemmas (no iff, no setoid rewriting). *)
Lemma implb_elim_b : forall x y : bool,
  implb x y = true -> x = true -> y = true.
Proof.
  intros [] [] H1 H2; simpl in *;
  try reflexivity; (discriminate H1 || discriminate H2).
Qed.

Lemma implb_intro_b : forall x y : bool,
  (x = true -> y = true) -> implb x y = true.
Proof.
  intros [] [] H; try reflexivity.
  specialize (H eq_refl); discriminate H.
Qed.

Lemma implb_trans_b : forall x y z : bool,
  implb x y = true -> implb y z = true -> implb x z = true.
Proof.
  intros x y z H1 H2.
  pose proof (implb_elim_b x y H1) as K1.
  pose proof (implb_elim_b y z H2) as K2.
  apply (implb_intro_b x z); intros Hx; apply K2; apply K1; exact Hx.
Qed.

Lemma bool_antisym_b : forall x y : bool,
  (x = true -> y = true) -> (y = true -> x = true) -> x = y.
Proof.
  intros [] [] H1 H2; try reflexivity.
  - specialize (H1 eq_refl); discriminate H1.
  - specialize (H2 eq_refl); discriminate H2.
Qed.

(* Row layer — extensionality: rows are equal iff all eight accessors agree. *)
Lemma row_ext : forall a b : row,
  atom_alloc a = atom_alloc b ->
  atom_async a = atom_async b ->
  atom_concurrency a = atom_concurrency b ->
  atom_ffi a = atom_ffi b ->
  atom_io a = atom_io b ->
  atom_nondet a = atom_nondet b ->
  atom_panic a = atom_panic b ->
  atom_unsafe a = atom_unsafe b ->
  a = b.
Proof.
  intros [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]]
         [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]]
         H0 H1 H2 H3 H4 H5 H6 H7;
  unfold atom_alloc, atom_async, atom_concurrency, atom_ffi,
         atom_io, atom_nondet, atom_panic, atom_unsafe in *;
  cbn in H0, H1, H2, H3, H4, H5, H6, H7;
  subst; reflexivity.
Qed.

(* Component-wise congruence for the right-nested row tuple: the one
   constructor-level fact f_equal cannot see (orb applications are not
   constructors, so injection stops exactly at the pair level). *)
Lemma row_cong : forall
  (a1 a2 b1 b2 c1 c2 d1 d2 e1 e2 f1 f2 g1 g2 h1 h2 : bool),
  a1 = a2 -> b1 = b2 -> c1 = c2 -> d1 = d2 ->
  e1 = e2 -> f1 = f2 -> g1 = g2 -> h1 = h2 ->
  (a1, (b1, (c1, (d1, (e1, (f1, (g1, h1))))))) =
  (a2, (b2, (c2, (d2, (e2, (f2, (g2, h2))))))).
Proof.
  intros a1 a2 b1 b2 c1 c2 d1 d2 e1 e2 f1 f2 g1 g2 h1 h2
         Ha Hb Hc Hd He Hf Hg Hh;
  subst; reflexivity.
Qed.

Lemma join_assoc : forall a b c : row,
  row_join a (row_join b c) = row_join (row_join a b) c.
Proof.
  intros a b c.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  destruct c as [c0 [c1 [c2 [c3 [c4 [c5 [c6 c7]]]]]]];
  unfold row_join in *; cbn; apply row_cong;
  [ symmetry; apply orb_assoc_b | symmetry; apply orb_assoc_b
  | symmetry; apply orb_assoc_b | symmetry; apply orb_assoc_b
  | symmetry; apply orb_assoc_b | symmetry; apply orb_assoc_b
  | symmetry; apply orb_assoc_b | symmetry; apply orb_assoc_b ].
Qed.

Lemma join_comm : forall a b : row, row_join a b = row_join b a.
Proof.
  intros a b.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  unfold row_join in *; cbn; apply row_cong;
  [ apply orb_comm_b | apply orb_comm_b | apply orb_comm_b | apply orb_comm_b
  | apply orb_comm_b | apply orb_comm_b | apply orb_comm_b | apply orb_comm_b ].
Qed.

Lemma join_idem : forall a : row, row_join a a = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  unfold row_join in *; cbn; apply row_cong;
  [ apply orb_idem_b | apply orb_idem_b | apply orb_idem_b | apply orb_idem_b
  | apply orb_idem_b | apply orb_idem_b | apply orb_idem_b | apply orb_idem_b ].
Qed.

Lemma join_identity_left : forall a : row, row_join row_empty a = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  unfold row_join, row_empty in *; cbn; apply row_cong;
  [ try reflexivity; apply orb_false_l_b | try reflexivity; apply orb_false_l_b
  | try reflexivity; apply orb_false_l_b | try reflexivity; apply orb_false_l_b
  | try reflexivity; apply orb_false_l_b | try reflexivity; apply orb_false_l_b
  | try reflexivity; apply orb_false_l_b | try reflexivity; apply orb_false_l_b ].
Qed.

Lemma join_identity_right : forall a : row, row_join a row_empty = a.
Proof.
  intros a.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  unfold row_join, row_empty in *; cbn; apply row_cong;
  [ apply orb_false_r_b | apply orb_false_r_b | apply orb_false_r_b | apply orb_false_r_b
  | apply orb_false_r_b | apply orb_false_r_b | apply orb_false_r_b | apply orb_false_r_b ].
Qed.

(* meet distributes over join: meet(a, join(b, c)) = join(meet(a, b), meet(a, c)). *)
Lemma meet_distrib_join : forall a b c : row,
  row_meet a (row_join b c) = row_join (row_meet a b) (row_meet a c).
Proof.
  intros a b c.
  destruct a as [a0 [a1 [a2 [a3 [a4 [a5 [a6 a7]]]]]]];
  destruct b as [b0 [b1 [b2 [b3 [b4 [b5 [b6 b7]]]]]]];
  destruct c as [c0 [c1 [c2 [c3 [c4 [c5 [c6 c7]]]]]]];
  unfold row_join, row_meet in *; cbn; apply row_cong;
  [ apply andb_distr_orb_b | apply andb_distr_orb_b
  | apply andb_distr_orb_b | apply andb_distr_orb_b
  | apply andb_distr_orb_b | apply andb_distr_orb_b
  | apply andb_distr_orb_b | apply andb_distr_orb_b ].
Qed.

(* Pointwise characterization of leq: the conjunction of the per-atom
   implications (the crate's certificate asymmetry: leq is the CONJUNCTION
   of the per-atom implications, not a per-atom equality — see
   planck-effect-math exh_leq_subset_characterization_524288). *)
Lemma row_leq_pointwise : forall a b : row,
  row_leq a b = true ->
  implb (atom_alloc a) (atom_alloc b) = true /\
  implb (atom_async a) (atom_async b) = true /\
  implb (atom_concurrency a) (atom_concurrency b) = true /\
  implb (atom_ffi a) (atom_ffi b) = true /\
  implb (atom_io a) (atom_io b) = true /\
  implb (atom_nondet a) (atom_nondet b) = true /\
  implb (atom_panic a) (atom_panic b) = true /\
  implb (atom_unsafe a) (atom_unsafe b) = true.
Proof.
  intros a b H.
  unfold row_leq in H.
  pose proof (andb_true_peel _ _ H) as HP0; destruct HP0 as [H0 Hr0];
  pose proof (andb_true_peel _ _ Hr0) as HP1; destruct HP1 as [H1 Hr1];
  pose proof (andb_true_peel _ _ Hr1) as HP2; destruct HP2 as [H2 Hr2];
  pose proof (andb_true_peel _ _ Hr2) as HP3; destruct HP3 as [H3 Hr3];
  pose proof (andb_true_peel _ _ Hr3) as HP4; destruct HP4 as [H4 Hr4];
  pose proof (andb_true_peel _ _ Hr4) as HP5; destruct HP5 as [H5 Hr5];
  pose proof (andb_true_peel _ _ Hr5) as HP6; destruct HP6 as [H6 H7];
  repeat split; [exact H0 | exact H1 | exact H2 | exact H3
                | exact H4 | exact H5 | exact H6 | exact H7].
Qed.

Lemma row_leq_intro : forall a b : row,
  implb (atom_alloc a) (atom_alloc b) = true ->
  implb (atom_async a) (atom_async b) = true ->
  implb (atom_concurrency a) (atom_concurrency b) = true ->
  implb (atom_ffi a) (atom_ffi b) = true ->
  implb (atom_io a) (atom_io b) = true ->
  implb (atom_nondet a) (atom_nondet b) = true ->
  implb (atom_panic a) (atom_panic b) = true ->
  implb (atom_unsafe a) (atom_unsafe b) = true ->
  row_leq a b = true.
Proof.
  intros a b H0 H1 H2 H3 H4 H5 H6 H7.
  unfold row_leq.
  rewrite H0, H1, H2, H3, H4, H5, H6, H7; reflexivity.
Qed.

(* Subtyping is a partial order: reflexive, antisymmetric, transitive. *)
Lemma leq_reflexive : forall a : row, row_leq a a = true.
Proof.
  intros a.
  apply row_leq_intro; apply implb_refl_b.
Qed.

(* Per-atom necessity of leq — one lemma per atom in the canonical order
   (explicit copies; the kernel checks each accessor path literally). *)
Lemma leq_imp_alloc : forall a b : row,
  row_leq a b = true -> atom_alloc a = true -> atom_alloc b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [H0 _];
  pose proof (implb_elim_b (atom_alloc a) (atom_alloc b) H0) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_async : forall a b : row,
  row_leq a b = true -> atom_async a = true -> atom_async b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [H1 _]];
  pose proof (implb_elim_b (atom_async a) (atom_async b) H1) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_concurrency : forall a b : row,
  row_leq a b = true -> atom_concurrency a = true -> atom_concurrency b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [H2 _]]];
  pose proof (implb_elim_b (atom_concurrency a) (atom_concurrency b) H2) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_ffi : forall a b : row,
  row_leq a b = true -> atom_ffi a = true -> atom_ffi b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [_ [H3 _]]]];
  pose proof (implb_elim_b (atom_ffi a) (atom_ffi b) H3) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_io : forall a b : row,
  row_leq a b = true -> atom_io a = true -> atom_io b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [_ [_ [H4 _]]]]];
  pose proof (implb_elim_b (atom_io a) (atom_io b) H4) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_nondet : forall a b : row,
  row_leq a b = true -> atom_nondet a = true -> atom_nondet b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [_ [_ [_ [H5 _]]]]]];
  pose proof (implb_elim_b (atom_nondet a) (atom_nondet b) H5) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_panic : forall a b : row,
  row_leq a b = true -> atom_panic a = true -> atom_panic b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [_ [_ [_ [_ [H6 _]]]]]]];
  pose proof (implb_elim_b (atom_panic a) (atom_panic b) H6) as K;
  apply K; exact Ha.
Qed.

Lemma leq_imp_unsafe : forall a b : row,
  row_leq a b = true -> atom_unsafe a = true -> atom_unsafe b = true.
Proof.
  intros a b H Ha.
  pose proof (row_leq_pointwise a b H) as HP;
  destruct HP as [_ [_ [_ [_ [_ [_ [_ H7]]]]]]];
  pose proof (implb_elim_b (atom_unsafe a) (atom_unsafe b) H7) as K;
  apply K; exact Ha.
Qed.

Lemma leq_antisymmetric : forall a b : row,
  row_leq a b = true -> row_leq b a = true -> a = b.
Proof.
  intros a b H1 H2.
  pose proof (row_leq_pointwise a b H1) as HP1;
  destruct HP1 as [Ha0 [Ha1 [Ha2 [Ha3 [Ha4 [Ha5 [Ha6 Ha7]]]]]]];
  pose proof (row_leq_pointwise b a H2) as HP2;
  destruct HP2 as [Hb0 [Hb1 [Hb2 [Hb3 [Hb4 [Hb5 [Hb6 Hb7]]]]]]];
  apply row_ext; apply bool_antisym_b;
  [ apply (implb_elim_b (atom_alloc a) (atom_alloc b)); exact Ha0
  | apply (implb_elim_b (atom_alloc b) (atom_alloc a)); exact Hb0
  | apply (implb_elim_b (atom_async a) (atom_async b)); exact Ha1
  | apply (implb_elim_b (atom_async b) (atom_async a)); exact Hb1
  | apply (implb_elim_b (atom_concurrency a) (atom_concurrency b)); exact Ha2
  | apply (implb_elim_b (atom_concurrency b) (atom_concurrency a)); exact Hb2
  | apply (implb_elim_b (atom_ffi a) (atom_ffi b)); exact Ha3
  | apply (implb_elim_b (atom_ffi b) (atom_ffi a)); exact Hb3
  | apply (implb_elim_b (atom_io a) (atom_io b)); exact Ha4
  | apply (implb_elim_b (atom_io b) (atom_io a)); exact Hb4
  | apply (implb_elim_b (atom_nondet a) (atom_nondet b)); exact Ha5
  | apply (implb_elim_b (atom_nondet b) (atom_nondet a)); exact Hb5
  | apply (implb_elim_b (atom_panic a) (atom_panic b)); exact Ha6
  | apply (implb_elim_b (atom_panic b) (atom_panic a)); exact Hb6
  | apply (implb_elim_b (atom_unsafe a) (atom_unsafe b)); exact Ha7
  | apply (implb_elim_b (atom_unsafe b) (atom_unsafe a)); exact Hb7 ].
Qed.

Lemma leq_transitive : forall a b c : row,
  row_leq a b = true -> row_leq b c = true -> row_leq a c = true.
Proof.
  intros a b c H1 H2.
  pose proof (row_leq_pointwise a b H1) as HP1;
  destruct HP1 as [Ha0 [Ha1 [Ha2 [Ha3 [Ha4 [Ha5 [Ha6 Ha7]]]]]]];
  pose proof (row_leq_pointwise b c H2) as HP2;
  destruct HP2 as [Hb0 [Hb1 [Hb2 [Hb3 [Hb4 [Hb5 [Hb6 Hb7]]]]]]];
  apply row_leq_intro;
  [ exact (implb_trans_b (atom_alloc a) (atom_alloc b) (atom_alloc c) Ha0 Hb0)
  | exact (implb_trans_b (atom_async a) (atom_async b) (atom_async c) Ha1 Hb1)
  | exact (implb_trans_b (atom_concurrency a) (atom_concurrency b) (atom_concurrency c) Ha2 Hb2)
  | exact (implb_trans_b (atom_ffi a) (atom_ffi b) (atom_ffi c) Ha3 Hb3)
  | exact (implb_trans_b (atom_io a) (atom_io b) (atom_io c) Ha4 Hb4)
  | exact (implb_trans_b (atom_nondet a) (atom_nondet b) (atom_nondet c) Ha5 Hb5)
  | exact (implb_trans_b (atom_panic a) (atom_panic b) (atom_panic c) Ha6 Hb6)
  | exact (implb_trans_b (atom_unsafe a) (atom_unsafe b) (atom_unsafe c) Ha7 Hb7) ].
Qed.

(* Pointwise necessity direction of the leq characterization, alloc atom:
   the certificate asymmetry named in planck-effect-math
   exh_leq_subset_characterization_524288. *)
Lemma leq_pointwise_necessity : forall a b : row,
  row_leq a b = true -> (atom_alloc a) = true -> (atom_alloc b) = true.
Proof.
  intros a b H Ha.
  apply leq_imp_alloc with (a := a) (b := b); assumption.
Qed.
