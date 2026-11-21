Definition byte : Type := nat.
Definition mid (lo hi : byte) : byte :=
  let m := Nat.add lo (Nat.div2 (Nat.sub hi lo)) in Nat.succ m.
