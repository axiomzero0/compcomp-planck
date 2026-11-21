//! GENERATED FILE - DO NOT EDIT.
//!
//! Generated mechanically by planck-extractor from rocq/Effect/CharClass.v (sha256 7d9f06bd1516875ca5fd546ac05510ee78760bba3c20f962b772acfd4604737b).
//! Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
//! extraction output as unverified until qualified, and the standing
//! standard requires every executable artifact to be mechanically
//! extracted. Regenerate with `planck-extractor extract`; drift is
//! detected by `planck-extractor check`.
//!
//! The proofs in rocq/Effect/CharClass.v are Prop-erased: Lemma..Qed blocks never enter
//! this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_charclass/src/lib.rs
// CEP:PROVENANCE: source rocq/Effect/CharClass.v (sha256 7d9f06bd1516875ca5fd546ac05510ee78760bba3c20f962b772acfd4604737b); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the rocq/Effect/CharClass.v computational content (pinned Gallina subset v1+v2 per DR-8.8): the pinned ASCII character-class predicates over the u8 domain (whitespace, decimal/hex digit, identifier start/continue, comment-introducer lead bytes), the pinned named constants, and the v1 prelude mapping
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset (DR-8.1/DR-8.8); Rust 2021; the pinned prelude mappings (tools/extractor/src/emit.rs); core str Ord is byte-lexicographic.
// CEP:COST: runtime: O(1)/O(log n) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 7d9f06bd1516875ca5fd546ac05510ee78760bba3c20f962b772acfd4604737b; ground truth extract/planck_charclass/ground_truth/*_table.rs (generated Rust source, kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_charclass)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with rocq/Effect/CharClass.v over the complete pinned domain (DR-8.8 v2.9); the proofs stay in rocq/ (Prop erasure is structural).
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

#[inline]
pub fn eqb(x: bool, y: bool) -> bool {
    x == y
}

#[allow(non_camel_case_types)]
pub type byte = u8;

#[allow(non_upper_case_globals)]
pub const TAB: byte = 9;

#[allow(non_upper_case_globals)]
pub const LF: byte = 10;

#[allow(non_upper_case_globals)]
pub const CR: byte = 13;

#[allow(non_upper_case_globals)]
pub const SPACE: byte = 32;

#[allow(non_upper_case_globals)]
pub const SLASH: byte = 47;

#[allow(non_upper_case_globals)]
pub const UNDERSCORE: byte = 95;

#[allow(non_upper_case_globals)]
pub const DEC_DIGIT_MIN: byte = 48;

#[allow(non_upper_case_globals)]
pub const DEC_DIGIT_MAX: byte = 57;

#[allow(non_upper_case_globals)]
pub const UPPER_HEX_MIN: byte = 65;

#[allow(non_upper_case_globals)]
pub const UPPER_HEX_MAX: byte = 70;

#[allow(non_upper_case_globals)]
pub const LOWER_HEX_MIN: byte = 97;

#[allow(non_upper_case_globals)]
pub const LOWER_HEX_MAX: byte = 102;

#[allow(non_upper_case_globals)]
pub const UPPER_MIN: byte = 65;

#[allow(non_upper_case_globals)]
pub const UPPER_MAX: byte = 90;

#[allow(non_upper_case_globals)]
pub const LOWER_MIN: byte = 97;

#[allow(non_upper_case_globals)]
pub const LOWER_MAX: byte = 122;

#[allow(non_upper_case_globals)]
pub const LINE_COMMENT_INTRODUCER_BYTE: byte = SLASH;

#[allow(non_upper_case_globals)]
pub const BLOCK_COMMENT_INTRODUCER_BYTE: byte = SLASH;

#[inline]
pub fn is_whitespace(b: byte) -> bool {
    orb(b == TAB, orb(b == LF, orb(b == CR, b == SPACE)))
}

#[inline]
pub fn is_decimal_digit(b: byte) -> bool {
    andb(DEC_DIGIT_MIN <= b, b <= DEC_DIGIT_MAX)
}

#[inline]
pub fn is_hex_digit(b: byte) -> bool {
    orb(is_decimal_digit(b), orb(andb(UPPER_HEX_MIN <= b, b <= UPPER_HEX_MAX), andb(LOWER_HEX_MIN <= b, b <= LOWER_HEX_MAX)))
}

#[inline]
pub fn is_ident_start(b: byte) -> bool {
    orb(andb(UPPER_MIN <= b, b <= UPPER_MAX), orb(andb(LOWER_MIN <= b, b <= LOWER_MAX), b == UNDERSCORE))
}

#[inline]
pub fn is_ident_continue(b: byte) -> bool {
    orb(is_ident_start(b), is_decimal_digit(b))
}

#[inline]
pub fn is_line_comment_start(b: byte) -> bool {
    b == SLASH
}

#[inline]
pub fn is_block_comment_start(b: byte) -> bool {
    b == SLASH
}

#[inline]
pub fn ws_truth(b: byte) -> bool {
    orb(andb(9 <= b, b <= 10), orb(b == 13, b == 32))
}

#[inline]
pub fn dec_truth(b: byte) -> bool {
    andb(48 <= b, b <= 57)
}

#[inline]
pub fn hex_truth(b: byte) -> bool {
    orb(andb(48 <= b, b <= 57), orb(andb(65 <= b, b <= 70), andb(97 <= b, b <= 102)))
}

#[inline]
pub fn ids_truth(b: byte) -> bool {
    orb(andb(65 <= b, b <= 90), orb(andb(97 <= b, b <= 122), b == 95))
}

#[inline]
pub fn idc_truth(b: byte) -> bool {
    orb(ids_truth(b), dec_truth(b))
}

#[inline]
pub fn comment_truth(b: byte) -> bool {
    b == 47
}

