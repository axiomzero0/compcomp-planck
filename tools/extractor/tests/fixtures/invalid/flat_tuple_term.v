Definition row : Type := (bool * (bool * bool))%type.
Definition bad (a b : row) : row := (fst a, fst b, fst a).
