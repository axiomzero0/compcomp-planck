Unset Automatic Proposition Inductives.
Inductive assoc : Type := Left | Right | None.
Definition bad (a : assoc) : nat := match a with | Left => 0 | Right => 1 end.
