Definition key : Type := (list nat).
Fixpoint key_eq_aux (fuel : nat) (a b : key) : bool :=
  match fuel with
  | 0 => true
  | S f =>
      match a with
      | nil => match b with nil => true | cons _ _ => false end
      | cons x xs =>
          match b with
          | nil => false
          | cons y ys => andb (Nat.eqb x y) (key_eq_aux f xs ys)
          end
      end
  end.
Definition key_eq (a b : key) : bool := key_eq_aux 32 a b.
