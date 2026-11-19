//! GENERATED FILE - DO NOT EDIT.
//!
//! Generated mechanically by planck-extractor from rocq/Effect/RowAlgebra.v (sha256 97dd6c25c8cfd0d9ff3c4868214506aa3a28d3ad6d986c128525eaedd7885399).
//! Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
//! extraction output as unverified until qualified, and the standing
//! standard requires every executable artifact to be mechanically
//! extracted. Regenerate with `planck-extractor extract`; drift is
//! detected by `planck-extractor check`.
//!
//! The proofs in rocq/Effect/RowAlgebra.v are Prop-erased: Lemma..Qed blocks never enter
//! this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_effect/src/lib.rs
// CEP:PROVENANCE: source rocq/Effect/RowAlgebra.v (sha256 97dd6c25c8cfd0d9ff3c4868214506aa3a28d3ad6d986c128525eaedd7885399); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the rocq/Effect/RowAlgebra.v computational content (pinned Gallina subset): the row domain (2^8-element tuple-of-bool lattice), the eight accessors, join/meet/leq, and the pinned prelude mapping (orb -> x|y, andb -> x&y, implb -> if x { y } else { true }).
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset; Rust 2021; the prelude mapping above (pinned in tools/extractor/src/emit.rs).
// CEP:COST: runtime: O(1) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 97dd6c25c8cfd0d9ff3c4868214506aa3a28d3ad6d986c128525eaedd7885399; ground truth extract/planck_effect/ground_truth/*.txt (kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_effect)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with rocq/Effect/RowAlgebra.v over all 2^8 x 2^8 input pairs; the proofs stay in rocq/ (Prop erasure is structural).
// CEP:FV-AXIOMS: none

#[inline]
pub fn orb(x: bool, y: bool) -> bool {
    x | y
}

#[inline]
pub fn andb(x: bool, y: bool) -> bool {
    x & y
}

#[inline]
pub fn implb(x: bool, y: bool) -> bool {
    if x {
        y
    } else {
        true
    }
}

#[allow(non_camel_case_types)]
pub type row = (bool, (bool, (bool, (bool, (bool, (bool, (bool, bool)))))));

#[inline]
pub fn atom_alloc(r: row) -> bool {
    (r).0
}

#[inline]
pub fn atom_async(r: row) -> bool {
    ((r).1).0
}

#[inline]
pub fn atom_concurrency(r: row) -> bool {
    (((r).1).1).0
}

#[inline]
pub fn atom_ffi(r: row) -> bool {
    ((((r).1).1).1).0
}

#[inline]
pub fn atom_io(r: row) -> bool {
    (((((r).1).1).1).1).0
}

#[inline]
pub fn atom_nondet(r: row) -> bool {
    ((((((r).1).1).1).1).1).0
}

#[inline]
pub fn atom_panic(r: row) -> bool {
    (((((((r).1).1).1).1).1).1).0
}

#[inline]
pub fn atom_unsafe(r: row) -> bool {
    (((((((r).1).1).1).1).1).1).1
}

#[inline]
pub fn row_join(a: row, b: row) -> row {
    (orb(atom_alloc(a), atom_alloc(b)), (orb(atom_async(a), atom_async(b)), (orb(atom_concurrency(a), atom_concurrency(b)), (orb(atom_ffi(a), atom_ffi(b)), (orb(atom_io(a), atom_io(b)), (orb(atom_nondet(a), atom_nondet(b)), (orb(atom_panic(a), atom_panic(b)), orb(atom_unsafe(a), atom_unsafe(b)))))))))
}

#[inline]
pub fn row_meet(a: row, b: row) -> row {
    (andb(atom_alloc(a), atom_alloc(b)), (andb(atom_async(a), atom_async(b)), (andb(atom_concurrency(a), atom_concurrency(b)), (andb(atom_ffi(a), atom_ffi(b)), (andb(atom_io(a), atom_io(b)), (andb(atom_nondet(a), atom_nondet(b)), (andb(atom_panic(a), atom_panic(b)), andb(atom_unsafe(a), atom_unsafe(b)))))))))
}

#[inline]
pub fn row_leq(a: row, b: row) -> bool {
    andb(implb(atom_alloc(a), atom_alloc(b)), andb(implb(atom_async(a), atom_async(b)), andb(implb(atom_concurrency(a), atom_concurrency(b)), andb(implb(atom_ffi(a), atom_ffi(b)), andb(implb(atom_io(a), atom_io(b)), andb(implb(atom_nondet(a), atom_nondet(b)), andb(implb(atom_panic(a), atom_panic(b)), implb(atom_unsafe(a), atom_unsafe(b)))))))))
}

#[allow(non_upper_case_globals)]
pub const row_empty: row = (false, (false, (false, (false, (false, (false, (false, false)))))));

