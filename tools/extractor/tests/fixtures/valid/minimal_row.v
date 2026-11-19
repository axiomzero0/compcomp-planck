Require Import Bool.
Definition row : Type := (bool * (bool * bool))%type.
Definition atom_fst (r : row) : bool := fst r.
Definition row_pair (a b : row) : row := (orb (atom_fst a) (atom_fst b), (orb (atom_fst b) (atom_fst a), true)).
Definition row_false : row := (false, (false, false)).
Lemma unused_helper : forall x : bool, x || x = x.
Proof.
  intros []; reflexivity.
Qed.
