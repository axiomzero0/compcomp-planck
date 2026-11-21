Inductive lookup_error : Type := UnknownToken.
Inductive lookup_result : Type := | Ok : nat -> lookup_result | Err : lookup_error -> lookup_result.
Definition hitb (r : lookup_result) : bool :=
  match r with
  | Ok _ => true
  | Err _ => false
  end.
