//! GENERATED FILE - DO NOT EDIT.
//!
//! Generated mechanically by planck-extractor from rocq/Effect/OperatorEnums.v (sha256 a1ad5d0677f5fb012649ec6fe95b66016691361b2e327ac111d6a9057bf9bcdc).
//! Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
//! extraction output as unverified until qualified, and the standing
//! standard requires every executable artifact to be mechanically
//! extracted. Regenerate with `planck-extractor extract`; drift is
//! detected by `planck-extractor check`.
//!
//! The proofs in rocq/Effect/OperatorEnums.v are Prop-erased: Lemma..Qed blocks never enter
//! this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_operator/src/lib.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorEnums.v (sha256 a1ad5d0677f5fb012649ec6fe95b66016691361b2e327ac111d6a9057bf9bcdc); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the rocq/Effect/OperatorEnums.v computational content (pinned Gallina subset v1+v2 per DR-8.8): the closed column enums (opid 44 variants, assoc, arity, overflow, ordering, lookup_error), the pinned operator-row data (keys as &'static str via the v2.6 mapping, 7-tuple rows, sorted table), the five column accessors, the OP-number projection, the precedence comparison, and the fuel-shaped binary-search lookup over the pinned const array
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset (DR-8.1/DR-8.8); Rust 2021; the pinned prelude mappings (tools/extractor/src/emit.rs); core str Ord is byte-lexicographic.
// CEP:COST: runtime: O(1)/O(log n) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 a1ad5d0677f5fb012649ec6fe95b66016691361b2e327ac111d6a9057bf9bcdc; ground truth extract/planck_operator/ground_truth/*_table.rs (generated Rust source, kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_operator)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with rocq/Effect/OperatorEnums.v over the complete pinned domain (DR-8.8 v2.9); the proofs stay in rocq/ (Prop erasure is structural).
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum assoc {
    Left,
    Right,
    None,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum arity {
    Zero,
    One,
    Two,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum overflow {
    Panic,
    Wrap,
    Unchecked,
    NotApplicable,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ordering {
    Less,
    Equal,
    Greater,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum lookup_error {
    UnknownToken,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum opid {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
    Range,
    RangeInclusive,
    RangeFrom,
    RangeTo,
    RangeFull,
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitXor,
    BitAnd,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Cast,
    Neg,
    Not,
    Deref,
    Borrow,
    Field,
    MethodCall,
    Index,
    Call,
    Try,
}

#[allow(non_camel_case_types)]
pub type lookup_result = Result<oprow, lookup_error>;

#[allow(non_camel_case_types)]
pub type key = &'static str;

// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor from rocq/Effect/OperatorTableData.v (sha256 d755b7fa4cc94385f57ecd993b785b28d743a0e8a86a9b627d93beb89ac0a76f).
// Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
// extraction output as unverified until qualified, and the standing
// standard requires every executable artifact to be mechanically
// extracted. Regenerate with `planck-extractor extract`; drift is
// detected by `planck-extractor check`.
//
// The proofs in rocq/Effect/OperatorTableData.v are Prop-erased: Lemma..Qed blocks never enter
// this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_operator/src/lib.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTableData.v (sha256 d755b7fa4cc94385f57ecd993b785b28d743a0e8a86a9b627d93beb89ac0a76f); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the rocq/Effect/OperatorTableData.v computational content (pinned Gallina subset v1+v2 per DR-8.8): the closed column enums (opid 44 variants, assoc, arity, overflow, ordering, lookup_error), the pinned operator-row data (keys as &'static str via the v2.6 mapping, 7-tuple rows, sorted table), the five column accessors, the OP-number projection, the precedence comparison, and the fuel-shaped binary-search lookup over the pinned const array
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset (DR-8.1/DR-8.8); Rust 2021; the pinned prelude mappings (tools/extractor/src/emit.rs); core str Ord is byte-lexicographic.
// CEP:COST: runtime: O(1)/O(log n) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 d755b7fa4cc94385f57ecd993b785b28d743a0e8a86a9b627d93beb89ac0a76f; ground truth extract/planck_operator/ground_truth/*_table.rs (generated Rust source, kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_operator)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with rocq/Effect/OperatorTableData.v over the complete pinned domain (DR-8.8 v2.9); the proofs stay in rocq/ (Prop erasure is structural).
// CEP:FV-AXIOMS: none

#[allow(non_camel_case_types)]
pub type oprow = (key, (opid, (u8, (assoc, (arity, (bool, overflow))))));

#[allow(non_upper_case_globals)]
pub const row_00_key: key = "! (unary)";

#[allow(non_upper_case_globals)]
pub const row_01_key: key = "!=";

#[allow(non_upper_case_globals)]
pub const row_02_key: key = "%";

#[allow(non_upper_case_globals)]
pub const row_03_key: key = "%=";

#[allow(non_upper_case_globals)]
pub const row_04_key: key = "& (binary)";

#[allow(non_upper_case_globals)]
pub const row_05_key: key = "& / &mut (borrow)";

#[allow(non_upper_case_globals)]
pub const row_06_key: key = "&&";

#[allow(non_upper_case_globals)]
pub const row_07_key: key = "&=";

#[allow(non_upper_case_globals)]
pub const row_08_key: key = "(args) call";

#[allow(non_upper_case_globals)]
pub const row_09_key: key = "* (binary)";

#[allow(non_upper_case_globals)]
pub const row_10_key: key = "* (deref)";

#[allow(non_upper_case_globals)]
pub const row_11_key: key = "*=";

#[allow(non_upper_case_globals)]
pub const row_12_key: key = "+";

#[allow(non_upper_case_globals)]
pub const row_13_key: key = "+=";

#[allow(non_upper_case_globals)]
pub const row_14_key: key = "- (binary)";

#[allow(non_upper_case_globals)]
pub const row_15_key: key = "- (unary)";

#[allow(non_upper_case_globals)]
pub const row_16_key: key = "-=";

#[allow(non_upper_case_globals)]
pub const row_17_key: key = ". field";

#[allow(non_upper_case_globals)]
pub const row_18_key: key = "..";

#[allow(non_upper_case_globals)]
pub const row_19_key: key = "..b";

#[allow(non_upper_case_globals)]
pub const row_20_key: key = ".name(args) method call";

#[allow(non_upper_case_globals)]
pub const row_21_key: key = "/";

#[allow(non_upper_case_globals)]
pub const row_22_key: key = "/=";

#[allow(non_upper_case_globals)]
pub const row_23_key: key = "<";

#[allow(non_upper_case_globals)]
pub const row_24_key: key = "<<";

#[allow(non_upper_case_globals)]
pub const row_25_key: key = "<<=";

#[allow(non_upper_case_globals)]
pub const row_26_key: key = "<=";

#[allow(non_upper_case_globals)]
pub const row_27_key: key = "=";

#[allow(non_upper_case_globals)]
pub const row_28_key: key = "==";

#[allow(non_upper_case_globals)]
pub const row_29_key: key = ">";

#[allow(non_upper_case_globals)]
pub const row_30_key: key = ">=";

#[allow(non_upper_case_globals)]
pub const row_31_key: key = ">>";

#[allow(non_upper_case_globals)]
pub const row_32_key: key = ">>=";

#[allow(non_upper_case_globals)]
pub const row_33_key: key = "?";

#[allow(non_upper_case_globals)]
pub const row_34_key: key = "[i] index";

#[allow(non_upper_case_globals)]
pub const row_35_key: key = "^ (binary)";

#[allow(non_upper_case_globals)]
pub const row_36_key: key = "^=";

#[allow(non_upper_case_globals)]
pub const row_37_key: key = "a..";

#[allow(non_upper_case_globals)]
pub const row_38_key: key = "a..=b";

#[allow(non_upper_case_globals)]
pub const row_39_key: key = "a..b";

#[allow(non_upper_case_globals)]
pub const row_40_key: key = "as";

#[allow(non_upper_case_globals)]
pub const row_41_key: key = "| (binary)";

#[allow(non_upper_case_globals)]
pub const row_42_key: key = "|=";

#[allow(non_upper_case_globals)]
pub const row_43_key: key = "||";

#[allow(non_upper_case_globals)]
pub const row_00: oprow = (row_00_key, (opid::Not, (13, (assoc::Right, (arity::One, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_01: oprow = (row_01_key, (opid::Ne, (7, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_02: oprow = (row_02_key, (opid::Rem, (11, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_03: oprow = (row_03_key, (opid::RemAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_04: oprow = (row_04_key, (opid::BitAnd, (6, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_05: oprow = (row_05_key, (opid::Borrow, (13, (assoc::Right, (arity::One, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_06: oprow = (row_06_key, (opid::LogicalAnd, (3, (assoc::Left, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_07: oprow = (row_07_key, (opid::BitAndAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_08: oprow = (row_08_key, (opid::Call, (14, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_09: oprow = (row_09_key, (opid::Mul, (11, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_10: oprow = (row_10_key, (opid::Deref, (13, (assoc::Right, (arity::One, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_11: oprow = (row_11_key, (opid::MulAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_12: oprow = (row_12_key, (opid::Add, (10, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_13: oprow = (row_13_key, (opid::AddAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_14: oprow = (row_14_key, (opid::Sub, (10, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_15: oprow = (row_15_key, (opid::Neg, (13, (assoc::Right, (arity::One, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_16: oprow = (row_16_key, (opid::SubAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_17: oprow = (row_17_key, (opid::Field, (14, (assoc::Left, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_18: oprow = (row_18_key, (opid::RangeFull, (1, (assoc::None, (arity::Zero, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_19: oprow = (row_19_key, (opid::RangeTo, (1, (assoc::None, (arity::One, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_20: oprow = (row_20_key, (opid::MethodCall, (14, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_21: oprow = (row_21_key, (opid::Div, (11, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_22: oprow = (row_22_key, (opid::DivAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_23: oprow = (row_23_key, (opid::Lt, (8, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_24: oprow = (row_24_key, (opid::Shl, (9, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_25: oprow = (row_25_key, (opid::ShlAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_26: oprow = (row_26_key, (opid::Le, (8, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_27: oprow = (row_27_key, (opid::Assign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_28: oprow = (row_28_key, (opid::Eq, (7, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_29: oprow = (row_29_key, (opid::Gt, (8, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_30: oprow = (row_30_key, (opid::Ge, (8, (assoc::None, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_31: oprow = (row_31_key, (opid::Shr, (9, (assoc::Left, (arity::Two, (true, overflow::Panic))))));

#[allow(non_upper_case_globals)]
pub const row_32: oprow = (row_32_key, (opid::ShrAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_33: oprow = (row_33_key, (opid::Try, (14, (assoc::Left, (arity::One, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_34: oprow = (row_34_key, (opid::Index, (14, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_35: oprow = (row_35_key, (opid::BitXor, (5, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_36: oprow = (row_36_key, (opid::BitXorAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_37: oprow = (row_37_key, (opid::RangeFrom, (1, (assoc::None, (arity::One, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_38: oprow = (row_38_key, (opid::RangeInclusive, (1, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_39: oprow = (row_39_key, (opid::Range, (1, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_40: oprow = (row_40_key, (opid::Cast, (12, (assoc::Left, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_41: oprow = (row_41_key, (opid::BitOr, (4, (assoc::Left, (arity::Two, (true, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_42: oprow = (row_42_key, (opid::BitOrAssign, (0, (assoc::None, (arity::Two, (false, overflow::NotApplicable))))));

#[allow(non_upper_case_globals)]
pub const row_43: oprow = (row_43_key, (opid::LogicalOr, (2, (assoc::Left, (arity::Two, (false, overflow::NotApplicable))))));

#[inline]
pub fn row_key(r: oprow) -> key {
    (r).0
}

#[inline]
pub fn row_opid(r: oprow) -> opid {
    ((r).1).0
}

#[inline]
pub fn row_precedence(r: oprow) -> u8 {
    (((r).1).1).0
}

#[inline]
pub fn row_assoc(r: oprow) -> assoc {
    ((((r).1).1).1).0
}

#[inline]
pub fn row_arity(r: oprow) -> arity {
    (((((r).1).1).1).1).0
}

#[inline]
pub fn row_overloadable(r: oprow) -> bool {
    ((((((r).1).1).1).1).1).0
}

#[inline]
pub fn row_overflow(r: oprow) -> overflow {
    ((((((r).1).1).1).1).1).1
}

#[allow(non_upper_case_globals)]
pub const all_opids: &'static [opid] = &[opid::Assign, opid::AddAssign, opid::SubAssign, opid::MulAssign, opid::DivAssign, opid::RemAssign, opid::BitAndAssign, opid::BitOrAssign, opid::BitXorAssign, opid::ShlAssign, opid::ShrAssign, opid::Range, opid::RangeInclusive, opid::RangeFrom, opid::RangeTo, opid::RangeFull, opid::LogicalOr, opid::LogicalAnd, opid::BitOr, opid::BitXor, opid::BitAnd, opid::Eq, opid::Ne, opid::Lt, opid::Gt, opid::Le, opid::Ge, opid::Shl, opid::Shr, opid::Add, opid::Sub, opid::Mul, opid::Div, opid::Rem, opid::Cast, opid::Neg, opid::Not, opid::Deref, opid::Borrow, opid::Field, opid::MethodCall, opid::Index, opid::Call, opid::Try];

#[allow(non_upper_case_globals)]
pub const operator_table_sorted: &'static [oprow] = &[row_00, row_01, row_02, row_03, row_04, row_05, row_06, row_07, row_08, row_09, row_10, row_11, row_12, row_13, row_14, row_15, row_16, row_17, row_18, row_19, row_20, row_21, row_22, row_23, row_24, row_25, row_26, row_27, row_28, row_29, row_30, row_31, row_32, row_33, row_34, row_35, row_36, row_37, row_38, row_39, row_40, row_41, row_42, row_43];

// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor from rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9).
// Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
// extraction output as unverified until qualified, and the standing
// standard requires every executable artifact to be mechanically
// extracted. Regenerate with `planck-extractor extract`; drift is
// detected by `planck-extractor check`.
//
// The proofs in rocq/Effect/OperatorTable.v are Prop-erased: Lemma..Qed blocks never enter
// this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_operator/src/lib.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the rocq/Effect/OperatorTable.v computational content (pinned Gallina subset v1+v2 per DR-8.8): the closed column enums (opid 44 variants, assoc, arity, overflow, ordering, lookup_error), the pinned operator-row data (keys as &'static str via the v2.6 mapping, 7-tuple rows, sorted table), the five column accessors, the OP-number projection, the precedence comparison, and the fuel-shaped binary-search lookup over the pinned const array
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset (DR-8.1/DR-8.8); Rust 2021; the pinned prelude mappings (tools/extractor/src/emit.rs); core str Ord is byte-lexicographic.
// CEP:COST: runtime: O(1)/O(log n) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9; ground truth extract/planck_operator/ground_truth/*_table.rs (generated Rust source, kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_operator)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with rocq/Effect/OperatorTable.v over the complete pinned domain (DR-8.8 v2.9); the proofs stay in rocq/ (Prop erasure is structural).
// CEP:FV-AXIOMS: none

#[inline]
pub fn lookup_hitb(r: lookup_result) -> bool {
    match r {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[inline]
pub fn precedence(o: opid) -> u8 {
    match o {
        opid::Assign | opid::AddAssign | opid::SubAssign | opid::MulAssign | opid::DivAssign | opid::RemAssign | opid::BitAndAssign | opid::BitOrAssign | opid::BitXorAssign | opid::ShlAssign | opid::ShrAssign => 0,
        opid::Range | opid::RangeInclusive | opid::RangeFrom | opid::RangeTo | opid::RangeFull => 1,
        opid::LogicalOr => 2,
        opid::LogicalAnd => 3,
        opid::BitOr => 4,
        opid::BitXor => 5,
        opid::BitAnd => 6,
        opid::Eq | opid::Ne => 7,
        opid::Lt | opid::Gt | opid::Le | opid::Ge => 8,
        opid::Shl | opid::Shr => 9,
        opid::Add | opid::Sub => 10,
        opid::Mul | opid::Div | opid::Rem => 11,
        opid::Cast => 12,
        opid::Neg | opid::Not | opid::Deref | opid::Borrow => 13,
        opid::Field | opid::MethodCall | opid::Index | opid::Call | opid::Try => 14,
    }
}

#[inline]
pub fn associativity(o: opid) -> assoc {
    match o {
        opid::LogicalOr | opid::LogicalAnd | opid::BitOr | opid::BitXor | opid::BitAnd | opid::Shl | opid::Shr | opid::Add | opid::Sub | opid::Mul | opid::Div | opid::Rem | opid::Cast | opid::Field | opid::MethodCall | opid::Index | opid::Call | opid::Try => assoc::Left,
        opid::Neg | opid::Not | opid::Deref | opid::Borrow => assoc::Right,
        opid::Assign | opid::AddAssign | opid::SubAssign | opid::MulAssign | opid::DivAssign | opid::RemAssign | opid::BitAndAssign | opid::BitOrAssign | opid::BitXorAssign | opid::ShlAssign | opid::ShrAssign | opid::Range | opid::RangeInclusive | opid::RangeFrom | opid::RangeTo | opid::RangeFull | opid::Eq | opid::Ne | opid::Lt | opid::Gt | opid::Le | opid::Ge => assoc::None,
    }
}

#[inline]
pub fn arity_of(o: opid) -> arity {
    match o {
        opid::Assign | opid::AddAssign | opid::SubAssign | opid::MulAssign | opid::DivAssign | opid::RemAssign | opid::BitAndAssign | opid::BitOrAssign | opid::BitXorAssign | opid::ShlAssign | opid::ShrAssign | opid::Range | opid::RangeInclusive | opid::LogicalOr | opid::LogicalAnd | opid::BitOr | opid::BitXor | opid::BitAnd | opid::Eq | opid::Ne | opid::Lt | opid::Gt | opid::Le | opid::Ge | opid::Shl | opid::Shr | opid::Add | opid::Sub | opid::Mul | opid::Div | opid::Rem | opid::Cast | opid::Field | opid::MethodCall | opid::Index | opid::Call => arity::Two,
        opid::RangeFrom | opid::RangeTo | opid::Neg | opid::Not | opid::Deref | opid::Borrow | opid::Try => arity::One,
        opid::RangeFull => arity::Zero,
    }
}

#[inline]
pub fn overloadable(o: opid) -> bool {
    match o {
        opid::BitOr | opid::BitXor | opid::BitAnd | opid::Eq | opid::Ne | opid::Lt | opid::Gt | opid::Le | opid::Ge | opid::Shl | opid::Shr | opid::Add | opid::Sub | opid::Mul | opid::Div | opid::Rem | opid::Neg | opid::Not | opid::MethodCall | opid::Index | opid::Call => true,
        opid::Assign | opid::AddAssign | opid::SubAssign | opid::MulAssign | opid::DivAssign | opid::RemAssign | opid::BitAndAssign | opid::BitOrAssign | opid::BitXorAssign | opid::ShlAssign | opid::ShrAssign | opid::Range | opid::RangeInclusive | opid::RangeFrom | opid::RangeTo | opid::RangeFull | opid::LogicalOr | opid::LogicalAnd | opid::Cast | opid::Deref | opid::Borrow | opid::Field | opid::Try => false,
    }
}

#[inline]
pub fn overflow_of(o: opid) -> overflow {
    match o {
        opid::Shl | opid::Shr | opid::Add | opid::Sub | opid::Mul | opid::Div | opid::Rem | opid::Neg => overflow::Panic,
        opid::Assign | opid::AddAssign | opid::SubAssign | opid::MulAssign | opid::DivAssign | opid::RemAssign | opid::BitAndAssign | opid::BitOrAssign | opid::BitXorAssign | opid::ShlAssign | opid::ShrAssign | opid::Range | opid::RangeInclusive | opid::RangeFrom | opid::RangeTo | opid::RangeFull | opid::LogicalOr | opid::LogicalAnd | opid::BitOr | opid::BitXor | opid::BitAnd | opid::Eq | opid::Ne | opid::Lt | opid::Gt | opid::Le | opid::Ge | opid::Cast | opid::Not | opid::Deref | opid::Borrow | opid::Field | opid::MethodCall | opid::Index | opid::Call | opid::Try => overflow::NotApplicable,
    }
}

#[inline]
pub fn arity_count(a: arity) -> u8 {
    match a {
        arity::Zero => 0,
        arity::One => 1,
        arity::Two => 2,
    }
}

#[inline]
pub fn precedence_compare(a: opid, b: opid) -> ordering {
    if precedence(a) < precedence(b) { ordering::Less } else { if precedence(b) < precedence(a) { ordering::Greater } else { ordering::Equal } }
}

#[inline]
pub fn opid_index(o: opid) -> u8 {
    match o {
        opid::Assign => 1,
        opid::AddAssign => 2,
        opid::SubAssign => 3,
        opid::MulAssign => 4,
        opid::DivAssign => 5,
        opid::RemAssign => 6,
        opid::BitAndAssign => 7,
        opid::BitOrAssign => 8,
        opid::BitXorAssign => 9,
        opid::ShlAssign => 10,
        opid::ShrAssign => 11,
        opid::Range => 12,
        opid::RangeInclusive => 13,
        opid::RangeFrom => 14,
        opid::RangeTo => 15,
        opid::RangeFull => 16,
        opid::LogicalOr => 17,
        opid::LogicalAnd => 18,
        opid::BitOr => 19,
        opid::BitXor => 20,
        opid::BitAnd => 21,
        opid::Eq => 22,
        opid::Ne => 23,
        opid::Lt => 24,
        opid::Gt => 25,
        opid::Le => 26,
        opid::Ge => 27,
        opid::Shl => 28,
        opid::Shr => 29,
        opid::Add => 30,
        opid::Sub => 31,
        opid::Mul => 32,
        opid::Div => 33,
        opid::Rem => 34,
        opid::Cast => 35,
        opid::Neg => 36,
        opid::Not => 37,
        opid::Deref => 38,
        opid::Borrow => 39,
        opid::Field => 40,
        opid::MethodCall => 41,
        opid::Index => 42,
        opid::Call => 43,
        opid::Try => 44,
    }
}

#[inline]
pub fn assoc_ord(a: assoc) -> u8 {
    match a {
        assoc::Left => 0,
        assoc::Right => 1,
        assoc::None => 2,
    }
}

#[inline]
pub fn arity_ord(a: arity) -> u8 {
    match a {
        arity::Zero => 0,
        arity::One => 1,
        arity::Two => 2,
    }
}

#[inline]
pub fn overflow_ord(o: overflow) -> u8 {
    match o {
        overflow::Panic => 0,
        overflow::Wrap => 1,
        overflow::Unchecked => 2,
        overflow::NotApplicable => 3,
    }
}

#[inline]
pub fn ordering_ord(o: ordering) -> u8 {
    match o {
        ordering::Less => 0,
        ordering::Equal => 1,
        ordering::Greater => 2,
    }
}

#[inline]
pub fn nat_compare_ord(a: u8, b: u8) -> u8 {
    if a < b { 0 } else { if b < a { 2 } else { 1 } }
}

pub fn bsearch_aux(fuel: u8, lo: u8, hi: u8, k: key) -> lookup_result {
    match fuel {
        0 => Err(lookup_error::UnknownToken),
        _ => { let f = fuel - 1; if lo <= hi { { let m = (lo).saturating_add(((hi).saturating_sub(lo)) / 2); if k == row_key(operator_table_sorted[(m) as usize]) { Ok(operator_table_sorted[(m) as usize]) } else { if row_key(operator_table_sorted[(m) as usize]) < k { bsearch_aux(f, (m).saturating_add(1), hi, k) } else { bsearch_aux(f, lo, m, k) } } } } else { Err(lookup_error::UnknownToken) } },
    }
}

#[inline]
pub fn lookup(k: key) -> lookup_result {
    bsearch_aux(44, 0, 43, k)
}

#[allow(non_upper_case_globals)]
pub const delimiter_keys: &'static [key] = &["(", ")", "[", "]", "{", "}"];

