Require Import List.
Definition key : Type := (list nat).
Definition k1 : key := [33; 32; 40].
Definition k2 : key := [47].
Definition ks : (list key) := [ (33 :: nil); (47 :: nil) ].
