Unset Automatic Proposition Inductives.
Module PinnedEnums.
  Inductive assoc : Type := Left | Right | None.
End PinnedEnums.
Import PinnedEnums.
Definition a1 : assoc := Left.
Definition aord (a : assoc) : nat := match a with | Left => 0 | Right => 1 | None => 2 end.
