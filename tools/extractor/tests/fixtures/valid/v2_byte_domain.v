Definition byte : Type := nat.
Definition TAB : byte := 9.
Definition is_ws (b : byte) : bool := Nat.eqb b TAB.
