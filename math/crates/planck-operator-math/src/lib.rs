// CEP:FILE: math/crates/planck-operator-math/src/lib.rs
// CEP:WHAT: The pinned operator table of Planck 0.1 edition P1 (Volume 02 §2.4.1; columns pinned by SPEC-INDEX §5.5), embedded verbatim as a no_std kernel crate: exactly 44 rows (OP-01..OP-44), each with operator token, precedence level (integer, higher binds tighter), associativity (Left/Right/None), arity, overloadability (bool), and overflow behavior class (Panic/Wrap/Unchecked/N/A); plus binary-search lookup by the exact Token/position key and the precedence-comparison, associativity, arity, overloadability, and overflow accessors.
// CEP:WHY: SPEC-INDEX §5.1 pins planck-operator-math as the extraction home of the §5.5 operator table; Volume 02 §2.8 OB-02-8 requires operator lookup to return the exact §2.4.1 row for each of the 44 operator shapes and no row for the 6 non-operator delimiters, and the Pratt climber (§2.6.4) consumes exactly the precedence/associativity of these rows, so the frontend must call exactly these functions and no others.
// CEP:STATUS: complete
// CEP:FAILURE: LookupError, which is complete over this crate's failure set: UnknownToken — the queried string is not exactly one of the 44 pinned Token/position keys of §2.4.1. Every other public function is total over the finite pinned domains (the closed 44-variant OpId set and the const table) and has no failure set. No panic, no unwrap, no allocation, no dynamic dispatch, no unsafe in non-test code.
// CEP:ASSUMES: Volume 02 §2.4.1 is the definition of record for every row (cells transcribed verbatim, markdown backticks stripped); SPEC-INDEX §5.5 pins the column set. Two cell-encoding conflicts are resolved in favor of the volume's exact rows per SPEC-INDEX §5 preamble: (1) arity — §5.5 names the column domain "(1, 2)" but the volume's OP-16 row pins arity 0, so Arity = {Zero, One, Two} with the volume authoritative; (2) overloadable — the pinned column is bool, but OP-41/OP-43 carry the cell text "dispatch per Volume 04", mapped to true because Volume 04 trait dispatch is exactly the overload mechanism, and OP-02's "no (desugar §2.4.6)" is mapped to false (the parenthetical is a cross-reference, not a value). Per §2.4.1: postfix applicative forms (OP-41, OP-42, OP-43) have receiver + argument as the two operands; the Wrap and Unchecked classes are used by no row of this table (they belong exclusively to the Volume 10 arithmetic intrinsics); unary `-` on unsigned types is a Volume 04 type error, not an overflow. SPEC-INDEX §5.2 (kernel implementation law: core:: only, no allocation, no panic, no unsafe in non-test code); §5.1 (equivalence claim form).
// CEP:COST: lookup is O(log2 44) <= 6 byte-lexicographic comparisons on &str, allocation-free; all accessors are O(1) const fns; the table is compile-time const data; no allocation anywhere.
// CEP:EVIDENCE: EXH exh_table_matches_transcription_44 (every row's six columns vs the hand-transcribed §2.4.1 table; the transcription enumerates all 44 rows and each OpId appears exactly once); EXH exh_lookup_keys_and_ascii_len_le2 (44 pinned keys + empty string + all 128 one-byte ASCII strings + all 16,384 two-byte ASCII strings + the 6 delimiters = 16,563 cases vs an independent linear-scan reference; OB-02-8); EXH exh_table_sorted_unique_keys (strict monotonicity of the token keys over the whole const array); EXH exh_accessors_match_rows_44 (all five accessors + index + arity count on every row); EXH exh_precedence_compare_all_pairs_1936 (44 x 44 pairs vs the reference ordering); EXH exh_overflow_wrap_unchecked_absent (Wrap/Unused-Unchecked absence and the exact 8-row Panic set); PROP prop_lookup_vs_linear_reference (10,000 deterministic PRNG strings: random, exact-key, and mutated-key draws, paired with the EXH lookup tests above).
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over the complete pinned domains, test-as-proof)
// CEP:FV-SCRIPT: crates/planck-operator-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete: the pinned lookup domain is exactly the 44-row table domain of SPEC-INDEX §5.5 / Volume 02 §2.4.1 and the EXH tests enumerate all of it, plus the fully enumerated stated non-operator domain (all 1- and 2-byte ASCII strings, the empty string, and the 6 delimiters); the const-array domain (44 rows) is enumerated for content, sortedness, key uniqueness, all accessors, and all 44 x 44 precedence pairs. No function claims equivalence outside these bounded domains; there is no untested pinned region and therefore no refinement gap.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

use core::cmp::Ordering;

// ---------------------------------------------------------------------------
// Named precedence constants (Law 7: constants are named, never magic literals).
// The levels and their member rows are Volume 02 §2.4.1; higher binds tighter.
// ---------------------------------------------------------------------------

/// Precedence 0: the assignment family OP-01..OP-11 (Volume 02 §2.4.1).
pub const PREC_ASSIGN: u8 = 0;
/// Precedence 1: the range forms OP-12..OP-16 (Volume 02 §2.4.1).
pub const PREC_RANGE: u8 = 1;
/// Precedence 2: logical or OP-17 (Volume 02 §2.4.1).
pub const PREC_LOGICAL_OR: u8 = 2;
/// Precedence 3: logical and OP-18 (Volume 02 §2.4.1).
pub const PREC_LOGICAL_AND: u8 = 3;
/// Precedence 4: binary bitwise or OP-19 (Volume 02 §2.4.1).
pub const PREC_BIT_OR: u8 = 4;
/// Precedence 5: binary bitwise xor OP-20 (Volume 02 §2.4.1).
pub const PREC_BIT_XOR: u8 = 5;
/// Precedence 6: binary bitwise and OP-21 (Volume 02 §2.4.1).
pub const PREC_BIT_AND: u8 = 6;
/// Precedence 7: equality OP-22, OP-23 (Volume 02 §2.4.1).
pub const PREC_EQUALITY: u8 = 7;
/// Precedence 8: ordering comparison OP-24..OP-27 (Volume 02 §2.4.1).
pub const PREC_COMPARISON: u8 = 8;
/// Precedence 9: shifts OP-28, OP-29 (Volume 02 §2.4.1).
pub const PREC_SHIFT: u8 = 9;
/// Precedence 10: additive OP-30, OP-31 (Volume 02 §2.4.1).
pub const PREC_ADDITIVE: u8 = 10;
/// Precedence 11: multiplicative OP-32..OP-34 (Volume 02 §2.4.1).
pub const PREC_MULTIPLICATIVE: u8 = 11;
/// Precedence 12: cast `as` OP-35 (Volume 02 §2.4.1).
pub const PREC_CAST: u8 = 12;
/// Precedence 13: unary OP-36..OP-39 (Volume 02 §2.4.1).
pub const PREC_UNARY: u8 = 13;
/// Precedence 14: postfix OP-40..OP-44; the highest pinned level, binds tightest (Volume 02 §2.4.1).
pub const PREC_POSTFIX: u8 = 14;
/// Minimum pinned precedence level, inclusive (Volume 02 §2.4.1).
pub const PREC_MIN: u8 = PREC_ASSIGN;
/// Maximum pinned precedence level, inclusive (Volume 02 §2.4.1).
pub const PREC_MAX: u8 = PREC_POSTFIX;

/// Exact row count of the pinned operator table: "exactly 44 rows" (Volume 02 §2.4.1).
pub const OPERATOR_TABLE_LEN: usize = 44;
/// OP number of the first row (Volume 02 §2.4.1: OP-01).
pub const OP_FIRST: u8 = 1;
/// OP number of the last row (Volume 02 §2.4.1: OP-44).
pub const OP_LAST: u8 = 44;

// ---------------------------------------------------------------------------
// Pinned column enums.
// ---------------------------------------------------------------------------

/// Operator identity: exactly the 44 rows OP-01..OP-44 of the pinned table
/// (Volume 02 §2.4.1; SPEC-INDEX §5.5). The discriminant of each variant is
/// its OP number; declaration order is the volume row order.
///
/// CEP:WHAT: Closed 44-variant operator identity enumeration, OP-01..OP-44 in volume row order.
/// CEP:WHY: The table is the normative home of operator syntax, precedence, associativity, arity, overloadability, and overflow class (Volume 02 §2.2.19); this enum is the row handle.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum; no other operator exists in edition P1, §2.2.19).
/// CEP:ASSUMES: Declaration order equals volume row order and `index()` equals the OP number; verified by EXH exh_table_matches_transcription_44.
/// CEP:COST: Copy type, no allocation; discriminants are u8.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_accessors_match_rows_44.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum OpId {
    /// OP-01 `=` — assignment (prec 0, None, arity 2, not overloadable, N/A).
    Assign = 1,
    /// OP-02 `+=` — compound add; desugars per §2.4.6 (prec 0, None, arity 2, not overloadable, N/A).
    AddAssign = 2,
    /// OP-03 `-=` — compound subtract (prec 0, None, arity 2, not overloadable, N/A).
    SubAssign = 3,
    /// OP-04 `*=` — compound multiply (prec 0, None, arity 2, not overloadable, N/A).
    MulAssign = 4,
    /// OP-05 `/=` — compound divide (prec 0, None, arity 2, not overloadable, N/A).
    DivAssign = 5,
    /// OP-06 `%=` — compound remainder (prec 0, None, arity 2, not overloadable, N/A).
    RemAssign = 6,
    /// OP-07 `&=` — compound bitwise and (prec 0, None, arity 2, not overloadable, N/A).
    BitAndAssign = 7,
    /// OP-08 `|=` — compound bitwise or (prec 0, None, arity 2, not overloadable, N/A).
    BitOrAssign = 8,
    /// OP-09 `^=` — compound bitwise xor (prec 0, None, arity 2, not overloadable, N/A).
    BitXorAssign = 9,
    /// OP-10 `<<=` — compound shift left (prec 0, None, arity 2, not overloadable, N/A).
    ShlAssign = 10,
    /// OP-11 `>>=` — compound shift right (prec 0, None, arity 2, not overloadable, N/A).
    ShrAssign = 11,
    /// OP-12 `a..b` — exclusive range form (prec 1, None, arity 2, not overloadable, N/A).
    Range = 12,
    /// OP-13 `a..=b` — inclusive range form (prec 1, None, arity 2, not overloadable, N/A).
    RangeInclusive = 13,
    /// OP-14 `a..` — range-from form (prec 1, None, arity 1, not overloadable, N/A).
    RangeFrom = 14,
    /// OP-15 `..b` — range-to form (prec 1, None, arity 1, not overloadable, N/A).
    RangeTo = 15,
    /// OP-16 `..` — full range form; the only arity-0 row of the table (prec 1, None, arity 0, not overloadable, N/A).
    RangeFull = 16,
    /// OP-17 `||` — logical or, short-circuit per §2.4.3 (prec 2, Left, arity 2, not overloadable, N/A).
    LogicalOr = 17,
    /// OP-18 `&&` — logical and, short-circuit per §2.4.3 (prec 3, Left, arity 2, not overloadable, N/A).
    LogicalAnd = 18,
    /// OP-19 `|` at binary position (prec 4, Left, arity 2, overloadable, N/A).
    BitOr = 19,
    /// OP-20 `^` at binary position (prec 5, Left, arity 2, overloadable, N/A).
    BitXor = 20,
    /// OP-21 `&` at binary position (prec 6, Left, arity 2, overloadable, N/A).
    BitAnd = 21,
    /// OP-22 `==` — equality (prec 7, None, arity 2, overloadable, N/A).
    Eq = 22,
    /// OP-23 `!=` — inequality (prec 7, None, arity 2, overloadable, N/A).
    Ne = 23,
    /// OP-24 `<` — less-than (prec 8, None, arity 2, overloadable, N/A).
    Lt = 24,
    /// OP-25 `>` — greater-than (prec 8, None, arity 2, overloadable, N/A).
    Gt = 25,
    /// OP-26 `<=` — less-or-equal (prec 8, None, arity 2, overloadable, N/A).
    Le = 26,
    /// OP-27 `>=` — greater-or-equal (prec 8, None, arity 2, overloadable, N/A).
    Ge = 27,
    /// OP-28 `<<` — shift left; Panic class on integer operands (prec 9, Left, arity 2, overloadable, Panic).
    Shl = 28,
    /// OP-29 `>>` — shift right; Panic class on integer operands (prec 9, Left, arity 2, overloadable, Panic).
    Shr = 29,
    /// OP-30 `+` — add (prec 10, Left, arity 2, overloadable, Panic).
    Add = 30,
    /// OP-31 `-` at binary position (prec 10, Left, arity 2, overloadable, Panic).
    Sub = 31,
    /// OP-32 `*` at binary position (prec 11, Left, arity 2, overloadable, Panic).
    Mul = 32,
    /// OP-33 `/` — divide; divisor-0 and min(T)/-1 panics per §2.4.4 (prec 11, Left, arity 2, overloadable, Panic).
    Div = 33,
    /// OP-34 `%` — remainder; panics per §2.4.4 (prec 11, Left, arity 2, overloadable, Panic).
    Rem = 34,
    /// OP-35 `as` — cast; exact results per §2.4.7 (prec 12, Left, arity 2, not overloadable, N/A).
    Cast = 35,
    /// OP-36 `-` at unary position; Panic class (unsigned negation is a Volume 04 type error) (prec 13, Right, arity 1, overloadable, Panic).
    Neg = 36,
    /// OP-37 `!` at unary position (prec 13, Right, arity 1, overloadable, N/A).
    Not = 37,
    /// OP-38 `*` at deref position; `{unsafe}` on raw pointers per §2.4.5 (prec 13, Right, arity 1, not overloadable, N/A).
    Deref = 38,
    /// OP-39 `&` / `&mut` at borrow position (prec 13, Right, arity 1, not overloadable, N/A).
    Borrow = 39,
    /// OP-40 `.` at field position (prec 14, Left, arity 2, not overloadable, N/A).
    Field = 40,
    /// OP-41 `.name(args)` method call; postfix applicative form = receiver + argument; overloadable per Volume 04 dispatch (prec 14, Left, arity 2, overloadable, N/A).
    MethodCall = 41,
    /// OP-42 `[i]` index; postfix applicative form = receiver + argument (prec 14, Left, arity 2, overloadable, N/A).
    Index = 42,
    /// OP-43 `(args)` call; postfix applicative form = receiver + argument; overloadable per Volume 04 dispatch (prec 14, Left, arity 2, overloadable, N/A).
    Call = 43,
    /// OP-44 `?` — try propagation per Volume 07 (prec 14, Left, arity 1, not overloadable, N/A).
    Try = 44,
}

/// Associativity: exactly the pinned column values Left, Right, None
/// (Volume 02 §2.4.1; SPEC-INDEX §5.5). `None` is the pinned "None" cell value
/// (the operator does not associate); it is not `Option::None`.
///
/// CEP:WHAT: Closed associativity enumeration with exactly the pinned values.
/// CEP:WHY: §2.6.4 derives binding powers from (precedence, associativity): (p, p+1) Left, (p, p) Right, (p+1, p) None.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum).
/// CEP:ASSUMES: Volume 02 §2.4.1 Assoc column is the definition of record.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_accessors_match_rows_44.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    /// Left-associative: 18 rows (OP-17..OP-21, OP-28..OP-35, OP-40..OP-44).
    Left,
    /// Right-associative: 4 rows (OP-36..OP-39).
    Right,
    /// Non-associative (the pinned cell value "None"): 22 rows.
    None,
}

/// Arity: the pinned arity column (SPEC-INDEX §5.5 names the column domain
/// "(1, 2)"; the volume's OP-16 row pins arity 0, and Volume 02 §2.4 is the
/// definition of the exact rows, so Zero is a member).
///
/// CEP:WHAT: Closed arity enumeration {Zero, One, Two} covering every value in the §2.4.1 table.
/// CEP:WHY: SPEC-INDEX §5.5 pins the arity column; the volume's exact rows require Zero for OP-16 (see file CEP:ASSUMES, conflict 1).
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum).
/// CEP:ASSUMES: Volume 02 §2.4.1 Arity column is authoritative over the §5.5 column-domain prose.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_accessors_match_rows_44.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arity {
    /// Arity 0: exactly one row (OP-16 `..`).
    Zero,
    /// Arity 1: 7 rows (OP-14, OP-15, OP-36..OP-39, OP-44).
    One,
    /// Arity 2: 36 rows, including the postfix applicative forms whose two operands are receiver + argument (§2.4.1).
    Two,
}

impl Arity {
    /// The arity as its exact numeric value 0, 1, or 2 (Volume 02 §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Arity column. Domain: the closed Arity enum (total).
    /// Failure set: none. Certificate: EXH `exh_accessors_match_rows_44` (count vs the transcribed numeric arity of every row).
    ///
    /// CEP:WHAT: Numeric projection of the arity column.
    /// CEP:WHY: Callers and the certificate need the pinned integer value, not just the variant.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function over the closed enum).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn count(self) -> u8 {
        match self {
            Arity::Zero => 0,
            Arity::One => 1,
            Arity::Two => 2,
        }
    }
}

/// Overflow behavior class: exactly the pinned column values Panic, Wrap,
/// Unchecked, N/A (Volume 02 §2.4.1; SPEC-INDEX §5.5). `NotApplicable` is the
/// pinned "N/A" cell value. Per §2.4.1, the Wrap and Unchecked classes are
/// used by no row of this table (they belong exclusively to the Volume 10
/// arithmetic intrinsics); both variants exist because the pinned column
/// domain has four values.
///
/// CEP:WHAT: Closed overflow-class enumeration with exactly the four pinned values.
/// CEP:WHY: SPEC-INDEX §5.5 pins the overflow behavior class column; §2.4.4 defines the exact Panic conditions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum).
/// CEP:ASSUMES: Volume 02 §2.4.1 Overflow class column is the definition of record; Wrap/Unchecked appear in no row (EXH-enforced).
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_overflow_wrap_unchecked_absent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overflow {
    /// The pinned "Panic" class: 8 rows (OP-28..OP-34, OP-36); exact panic conditions are §2.4.4.
    Panic,
    /// The pinned "Wrap" class: used by no row of this table (Volume 10 intrinsics only).
    Wrap,
    /// The pinned "Unchecked" class: used by no row of this table (Volume 10 intrinsics only).
    Unchecked,
    /// The pinned "N/A" class: the remaining 36 rows.
    NotApplicable,
}

// ---------------------------------------------------------------------------
// Row type and the pinned table (const, sorted by token key for binary search).
// ---------------------------------------------------------------------------

/// One row of the pinned operator table (Volume 02 §2.4.1; SPEC-INDEX §5.5).
///
/// The six fields are exactly the pinned columns plus the row identity:
/// `token` is the verbatim Token/position cell of §2.4.1 (markdown backticks
/// stripped) and is the unique binary-search key of the row; `precedence`
/// is an integer, higher binds tighter; `assoc` ∈ {Left, Right, None};
/// `arity` ∈ {Zero, One, Two}; `overloadable` is the pinned bool;
/// `overflow` ∈ {Panic, Wrap, Unchecked, NotApplicable}.
///
/// CEP:WHAT: The row record of the pinned §2.4.1 table: identity + the six pinned columns.
/// CEP:WHY: lookup returns this record; its fields are the definition of record of each column.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (pure const data).
/// CEP:ASSUMES: `token` is unique across all 44 rows (EXH-enforced); the two cell-encoding conflicts of the file CEP:ASSUMES are resolved as stated there.
/// CEP:COST: Copy type of 8+8+1+1+1+1+1 padded bytes; no allocation.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_table_sorted_unique_keys; EXH exh_accessors_match_rows_44.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OperatorRow {
    /// Row identity OP-01..OP-44 (the "Op" column of §2.4.1).
    pub id: OpId,
    /// Verbatim Token/position cell of §2.4.1 (backticks stripped); the unique lookup key.
    pub token: &'static str,
    /// Precedence level; higher binds tighter (the "Prec" column).
    pub precedence: u8,
    /// Associativity (the "Assoc" column).
    pub assoc: Assoc,
    /// Arity (the "Arity" column).
    pub arity: Arity,
    /// Overloadable (the "Overloadable" column).
    pub overloadable: bool,
    /// Overflow behavior class (the "Overflow class" column).
    pub overflow: Overflow,
}

/// The pinned operator table of Volume 02 §2.4.1, embedded verbatim: exactly
/// `OPERATOR_TABLE_LEN` = 44 rows, sorted ascending by the unique `token`
/// key (byte-lexicographic `str` order) for binary search. Every cell is the
/// §2.4.1 cell value (markdown backticks stripped); nothing added, dropped,
/// or altered.
///
/// Refinement anchor: Volume 02 §2.4.1 / SPEC-INDEX §5.5. Domain: a fixed 44-element const array.
/// Failure set: none (const data). Certificate: EXH `exh_table_matches_transcription_44`
/// (all 44 rows vs the hand-transcribed table), EXH `exh_table_sorted_unique_keys`
/// (sortedness + uniqueness over the whole array).
///
/// CEP:WHAT: The 44-row §2.4.1 table as sorted const data; the extraction's definition of record.
/// CEP:WHY: SPEC-INDEX §5.5: "planck-operator-math embeds exactly that table as a const array and exposes lookup with binary search".
/// CEP:STATUS: complete
/// CEP:FAILURE: A row that disagrees with the transcribed §2.4.1 table, a duplicate key, or a wrong sort order fails the EXH tests and the crate must not ship.
/// CEP:ASSUMES: Rows are sorted by `token` in byte-lexicographic (str Ord) order; the order is itself EXH-checked, so any transcription of the sort is validated, not assumed.
/// CEP:COST: Compile-time const data; zero runtime cost.
/// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_table_sorted_unique_keys; EXH exh_lookup_keys_and_ascii_len_le2.
pub const OPERATOR_TABLE: [OperatorRow; OPERATOR_TABLE_LEN] = [
    // Sorted ascending by `token` (byte-lexicographic). Each row is §2.4.1 verbatim.
    OperatorRow { id: OpId::Not, token: "! (unary)", precedence: PREC_UNARY, assoc: Assoc::Right, arity: Arity::One, overloadable: true, overflow: Overflow::NotApplicable }, // OP-37
    OperatorRow { id: OpId::Ne, token: "!=", precedence: PREC_EQUALITY, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-23
    OperatorRow { id: OpId::Rem, token: "%", precedence: PREC_MULTIPLICATIVE, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-34
    OperatorRow { id: OpId::RemAssign, token: "%=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-06
    OperatorRow { id: OpId::BitAnd, token: "& (binary)", precedence: PREC_BIT_AND, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-21
    OperatorRow { id: OpId::Borrow, token: "& / &mut (borrow)", precedence: PREC_UNARY, assoc: Assoc::Right, arity: Arity::One, overloadable: false, overflow: Overflow::NotApplicable }, // OP-39
    OperatorRow { id: OpId::LogicalAnd, token: "&&", precedence: PREC_LOGICAL_AND, assoc: Assoc::Left, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-18
    OperatorRow { id: OpId::BitAndAssign, token: "&=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-07
    OperatorRow { id: OpId::Call, token: "(args) call", precedence: PREC_POSTFIX, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-43
    OperatorRow { id: OpId::Mul, token: "* (binary)", precedence: PREC_MULTIPLICATIVE, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-32
    OperatorRow { id: OpId::Deref, token: "* (deref)", precedence: PREC_UNARY, assoc: Assoc::Right, arity: Arity::One, overloadable: false, overflow: Overflow::NotApplicable }, // OP-38
    OperatorRow { id: OpId::MulAssign, token: "*=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-04
    OperatorRow { id: OpId::Add, token: "+", precedence: PREC_ADDITIVE, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-30
    OperatorRow { id: OpId::AddAssign, token: "+=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-02
    OperatorRow { id: OpId::Sub, token: "- (binary)", precedence: PREC_ADDITIVE, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-31
    OperatorRow { id: OpId::Neg, token: "- (unary)", precedence: PREC_UNARY, assoc: Assoc::Right, arity: Arity::One, overloadable: true, overflow: Overflow::Panic }, // OP-36
    OperatorRow { id: OpId::SubAssign, token: "-=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-03
    OperatorRow { id: OpId::Field, token: ". field", precedence: PREC_POSTFIX, assoc: Assoc::Left, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-40
    OperatorRow { id: OpId::RangeFull, token: "..", precedence: PREC_RANGE, assoc: Assoc::None, arity: Arity::Zero, overloadable: false, overflow: Overflow::NotApplicable }, // OP-16
    OperatorRow { id: OpId::RangeTo, token: "..b", precedence: PREC_RANGE, assoc: Assoc::None, arity: Arity::One, overloadable: false, overflow: Overflow::NotApplicable }, // OP-15
    OperatorRow { id: OpId::MethodCall, token: ".name(args) method call", precedence: PREC_POSTFIX, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-41
    OperatorRow { id: OpId::Div, token: "/", precedence: PREC_MULTIPLICATIVE, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-33
    OperatorRow { id: OpId::DivAssign, token: "/=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-05
    OperatorRow { id: OpId::Lt, token: "<", precedence: PREC_COMPARISON, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-24
    OperatorRow { id: OpId::Shl, token: "<<", precedence: PREC_SHIFT, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-28
    OperatorRow { id: OpId::ShlAssign, token: "<<=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-10
    OperatorRow { id: OpId::Le, token: "<=", precedence: PREC_COMPARISON, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-26
    OperatorRow { id: OpId::Assign, token: "=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-01
    OperatorRow { id: OpId::Eq, token: "==", precedence: PREC_EQUALITY, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-22
    OperatorRow { id: OpId::Gt, token: ">", precedence: PREC_COMPARISON, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-25
    OperatorRow { id: OpId::Ge, token: ">=", precedence: PREC_COMPARISON, assoc: Assoc::None, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-27
    OperatorRow { id: OpId::Shr, token: ">>", precedence: PREC_SHIFT, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::Panic }, // OP-29
    OperatorRow { id: OpId::ShrAssign, token: ">>=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-11
    OperatorRow { id: OpId::Try, token: "?", precedence: PREC_POSTFIX, assoc: Assoc::Left, arity: Arity::One, overloadable: false, overflow: Overflow::NotApplicable }, // OP-44
    OperatorRow { id: OpId::Index, token: "[i] index", precedence: PREC_POSTFIX, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-42
    OperatorRow { id: OpId::BitXor, token: "^ (binary)", precedence: PREC_BIT_XOR, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-20
    OperatorRow { id: OpId::BitXorAssign, token: "^=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-09
    OperatorRow { id: OpId::RangeFrom, token: "a..", precedence: PREC_RANGE, assoc: Assoc::None, arity: Arity::One, overloadable: false, overflow: Overflow::NotApplicable }, // OP-14
    OperatorRow { id: OpId::RangeInclusive, token: "a..=b", precedence: PREC_RANGE, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-13
    OperatorRow { id: OpId::Range, token: "a..b", precedence: PREC_RANGE, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-12
    OperatorRow { id: OpId::Cast, token: "as", precedence: PREC_CAST, assoc: Assoc::Left, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-35
    OperatorRow { id: OpId::BitOr, token: "| (binary)", precedence: PREC_BIT_OR, assoc: Assoc::Left, arity: Arity::Two, overloadable: true, overflow: Overflow::NotApplicable }, // OP-19
    OperatorRow { id: OpId::BitOrAssign, token: "|=", precedence: PREC_ASSIGN, assoc: Assoc::None, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-08
    OperatorRow { id: OpId::LogicalOr, token: "||", precedence: PREC_LOGICAL_OR, assoc: Assoc::Left, arity: Arity::Two, overloadable: false, overflow: Overflow::NotApplicable }, // OP-17
];

// ---------------------------------------------------------------------------
// Lookup (the pinned binary-search API) and precedence comparison.
// ---------------------------------------------------------------------------

/// The complete failure set of `lookup` (Volume 02 §2.4.1 lookup contract).
///
/// Complete by construction: `lookup` has exactly one failure mode — the
/// queried string is not one of the 44 pinned Token/position keys — and this
/// enum names it. There are no other variants and no other failures.
///
/// CEP:WHAT: Complete lookup error set: exactly one variant, UnknownToken.
/// CEP:WHY: SPEC-INDEX §5.2 requires an explicit, complete error enum for every failure.
/// CEP:STATUS: complete
/// CEP:FAILURE: This type IS the failure set of lookup; no other public function in this crate can fail.
/// CEP:ASSUMES: Key equality is exact byte-lexicographic equality on the verbatim Token/position cell.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_lookup_keys_and_ascii_len_le2 (the variant is constructed and matched on every non-key probe of the 16,563-case domain).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LookupError {
    /// No row of the pinned table has this exact Token/position key
    /// (Volume 02 §2.4.1; OB-02-8). Note: a bare spelling that is only part
    /// of a shape-qualified key (e.g. `-`, `*`, `&`, `|`, `^`, `!`, `.`) is
    /// UnknownToken — those rows are keyed by their full verbatim cells
    /// ("- (binary)", "- (unary)", ...).
    UnknownToken,
}

/// Look up an operator row by its exact Token/position key.
///
/// Refinement anchor: Volume 02 §2.4.1 ("exposes lookup by binary search"),
/// SPEC-INDEX §5.5, OB-02-8. The key is the verbatim Token/position cell of
/// the row (markdown backticks stripped), e.g. `=`, `+=`, `a..b`, `| (binary)`,
/// `- (unary)`, `& / &mut (borrow)`, `. field`, `.name(args) method call`,
/// `[i] index`, `(args) call`. The 44 keys are pairwise distinct, so exact
/// match identifies at most one row.
/// Input domain: every `&str` (total; terminates in <= 6 comparisons). The
/// defined (hit) domain is exactly the 44 pinned keys; every other input,
/// including the 6 delimiters `(`, `)`, `[`, `]`, `{`, `}`, returns
/// `Err(LookupError::UnknownToken)`.
/// Failure set: `LookupError::UnknownToken` (complete; single-variant).
/// Certificate: EXH `exh_lookup_keys_and_ascii_len_le2` — all 44 keys plus the
/// empty string plus all 128 one-byte and all 16,384 two-byte ASCII strings
/// plus the 6 delimiters (16,563 cases) against an independent linear-scan
/// reference; PROP `prop_lookup_vs_linear_reference` (paired, 10,000 draws).
///
/// CEP:WHAT: Binary-search lookup of the pinned §2.4.1 table by exact Token/position key.
/// CEP:WHY: SPEC-INDEX §5.5 pins "lookup with binary search"; OB-02-8 pins the 44-shape hit set and the 6-delimiter miss set.
/// CEP:STATUS: complete
/// CEP:FAILURE: LookupError::UnknownToken — the only failure; the error enum is complete (single variant).
/// CEP:ASSUMES: OPERATOR_TABLE is sorted ascending by unique token key (EXH-enforced, not assumed); `&str` comparison is byte-lexicographic per core Ord.
/// CEP:COST: O(log2 44) <= 6 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_lookup_keys_and_ascii_len_le2; PROP prop_lookup_vs_linear_reference; EXH exh_table_sorted_unique_keys.
#[inline]
pub fn lookup(token: &str) -> Result<&'static OperatorRow, LookupError> {
    // Hand-rolled binary search: invariant — the key, if present, lies in OPERATOR_TABLE[lo..hi].
    let mut lo: usize = 0;
    let mut hi: usize = OPERATOR_TABLE_LEN;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let row = &OPERATOR_TABLE[mid];
        if row.token == token {
            return Ok(row);
        }
        if row.token < token {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    Err(LookupError::UnknownToken)
}

/// Compare the precedence of two operators: higher binds tighter
/// (Volume 02 §2.4.1 "Prec" column; SPEC-INDEX §5.5).
///
/// Refinement anchor: Volume 02 §2.4.1 Prec column (levels 0..=14).
/// Input domain: the closed 44 x 44 pair set over OpId (total).
/// Failure set: none.
/// Certificate: EXH `exh_precedence_compare_all_pairs_1936` — every pair equals
/// `precedence(a).cmp(&precedence(b))`.
///
/// CEP:WHAT: Precedence comparison, the primitive consumed by the Pratt climber (§2.6.4).
/// CEP:WHY: SPEC-INDEX §5.5 pins "higher binds tighter"; this function is that judgment.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total const function over the closed pair domain).
/// CEP:ASSUMES: Equal precedences compare Equal (the caller applies the §2.6.4 associativity rule at equal precedence).
/// CEP:COST: O(1), const-evaluable; CEP-0.
/// CEP:EVIDENCE: EXH exh_precedence_compare_all_pairs_1936; EXH exh_accessors_match_rows_44.
#[inline]
pub const fn precedence_compare(a: OpId, b: OpId) -> Ordering {
    let pa = a.precedence();
    let pb = b.precedence();
    if pa < pb {
        Ordering::Less
    } else if pa > pb {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

// ---------------------------------------------------------------------------
// Accessors over OpId (const, total, closed-domain).
// ---------------------------------------------------------------------------

impl OpId {
    /// The OP number of this row, exactly 1..=44 (the "Op" column of §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 row numbering. Domain: the closed OpId enum (total).
    /// Failure set: none. Certificate: EXH `exh_table_matches_transcription_44`
    /// (declaration order equals volume row order) and `exh_accessors_match_rows_44`.
    ///
    /// CEP:WHAT: OP-number projection (the row identity as its pinned integer).
    /// CEP:WHY: Diagnostics and certificates reference rows by their pinned OP number.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: Enum discriminants are the OP numbers (verified by EXH).
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_table_matches_transcription_44; EXH exh_accessors_match_rows_44.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The precedence level of this row, exactly 0..=14; higher binds tighter
    /// (the "Prec" column of §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Prec column. Domain: the closed OpId enum (total).
    /// Failure set: none.
    /// Certificate: EXH `exh_accessors_match_rows_44` (every row's accessor equals its transcribed precedence).
    ///
    /// CEP:WHAT: Precedence accessor over the closed operator set.
    /// CEP:WHY: The Pratt climber (§2.6.4) and the comparison primitive consume exactly these levels.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: The match arms duplicate the table's Prec column; agreement with the table is EXH-enforced, not assumed.
    /// CEP:COST: O(1), const-evaluable, allocation-free; CEP-0.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44; EXH exh_precedence_compare_all_pairs_1936.
    pub const fn precedence(self) -> u8 {
        match self {
            OpId::Assign
            | OpId::AddAssign
            | OpId::SubAssign
            | OpId::MulAssign
            | OpId::DivAssign
            | OpId::RemAssign
            | OpId::BitAndAssign
            | OpId::BitOrAssign
            | OpId::BitXorAssign
            | OpId::ShlAssign
            | OpId::ShrAssign => PREC_ASSIGN,
            OpId::Range | OpId::RangeInclusive | OpId::RangeFrom | OpId::RangeTo | OpId::RangeFull => PREC_RANGE,
            OpId::LogicalOr => PREC_LOGICAL_OR,
            OpId::LogicalAnd => PREC_LOGICAL_AND,
            OpId::BitOr => PREC_BIT_OR,
            OpId::BitXor => PREC_BIT_XOR,
            OpId::BitAnd => PREC_BIT_AND,
            OpId::Eq | OpId::Ne => PREC_EQUALITY,
            OpId::Lt | OpId::Gt | OpId::Le | OpId::Ge => PREC_COMPARISON,
            OpId::Shl | OpId::Shr => PREC_SHIFT,
            OpId::Add | OpId::Sub => PREC_ADDITIVE,
            OpId::Mul | OpId::Div | OpId::Rem => PREC_MULTIPLICATIVE,
            OpId::Cast => PREC_CAST,
            OpId::Neg | OpId::Not | OpId::Deref | OpId::Borrow => PREC_UNARY,
            OpId::Field | OpId::MethodCall | OpId::Index | OpId::Call | OpId::Try => PREC_POSTFIX,
        }
    }

    /// The associativity of this row: exactly Left, Right, or None
    /// (the "Assoc" column of §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Assoc column. Domain: the closed OpId enum (total).
    /// Failure set: none.
    /// Certificate: EXH `exh_accessors_match_rows_44`.
    ///
    /// CEP:WHAT: Associativity accessor over the closed operator set.
    /// CEP:WHY: §2.6.4 binding-power derivation is a function of (precedence, associativity).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: The match arms duplicate the table's Assoc column; agreement is EXH-enforced.
    /// CEP:COST: O(1), const-evaluable, allocation-free; CEP-0.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn associativity(self) -> Assoc {
        match self {
            OpId::LogicalOr
            | OpId::LogicalAnd
            | OpId::BitOr
            | OpId::BitXor
            | OpId::BitAnd
            | OpId::Shl
            | OpId::Shr
            | OpId::Add
            | OpId::Sub
            | OpId::Mul
            | OpId::Div
            | OpId::Rem
            | OpId::Cast
            | OpId::Field
            | OpId::MethodCall
            | OpId::Index
            | OpId::Call
            | OpId::Try => Assoc::Left,
            OpId::Neg | OpId::Not | OpId::Deref | OpId::Borrow => Assoc::Right,
            OpId::Assign
            | OpId::AddAssign
            | OpId::SubAssign
            | OpId::MulAssign
            | OpId::DivAssign
            | OpId::RemAssign
            | OpId::BitAndAssign
            | OpId::BitOrAssign
            | OpId::BitXorAssign
            | OpId::ShlAssign
            | OpId::ShrAssign
            | OpId::Range
            | OpId::RangeInclusive
            | OpId::RangeFrom
            | OpId::RangeTo
            | OpId::RangeFull
            | OpId::Eq
            | OpId::Ne
            | OpId::Lt
            | OpId::Gt
            | OpId::Le
            | OpId::Ge => Assoc::None,
        }
    }

    /// The arity of this row: Zero (OP-16 only), One, or Two
    /// (the "Arity" column of §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Arity column. Domain: the closed OpId enum (total).
    /// Failure set: none.
    /// Certificate: EXH `exh_accessors_match_rows_44`.
    ///
    /// CEP:WHAT: Arity accessor over the closed operator set.
    /// CEP:WHY: The parser builds operand counts from exactly these values; postfix applicative forms carry receiver + argument (§2.4.1).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: The match arms duplicate the table's Arity column; agreement is EXH-enforced.
    /// CEP:COST: O(1), const-evaluable, allocation-free; CEP-0.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn arity(self) -> Arity {
        match self {
            OpId::Assign
            | OpId::AddAssign
            | OpId::SubAssign
            | OpId::MulAssign
            | OpId::DivAssign
            | OpId::RemAssign
            | OpId::BitAndAssign
            | OpId::BitOrAssign
            | OpId::BitXorAssign
            | OpId::ShlAssign
            | OpId::ShrAssign
            | OpId::Range
            | OpId::RangeInclusive
            | OpId::LogicalOr
            | OpId::LogicalAnd
            | OpId::BitOr
            | OpId::BitXor
            | OpId::BitAnd
            | OpId::Eq
            | OpId::Ne
            | OpId::Lt
            | OpId::Gt
            | OpId::Le
            | OpId::Ge
            | OpId::Shl
            | OpId::Shr
            | OpId::Add
            | OpId::Sub
            | OpId::Mul
            | OpId::Div
            | OpId::Rem
            | OpId::Cast
            | OpId::Field
            | OpId::MethodCall
            | OpId::Index
            | OpId::Call => Arity::Two,
            OpId::RangeFrom | OpId::RangeTo | OpId::Neg | OpId::Not | OpId::Deref | OpId::Borrow | OpId::Try => Arity::One,
            OpId::RangeFull => Arity::Zero,
        }
    }

    /// The overloadability of this row (the "Overloadable" column of §2.4.1).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Overloadable column. Domain: the closed OpId enum (total).
    /// Failure set: none.
    /// Certificate: EXH `exh_accessors_match_rows_44`.
    /// Cell-encoding note: OP-41/OP-43 carry "dispatch per Volume 04" in the
    /// volume cell, mapped to `true` (Volume 04 trait dispatch is the overload
    /// mechanism); OP-02 carries "no (desugar §2.4.6)", mapped to `false`.
    ///
    /// CEP:WHAT: Overloadability accessor over the closed operator set.
    /// CEP:WHY: SPEC-INDEX §5.5 pins the column as bool; Volume 04 dispatch refines the true rows.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: The two non-bool cell texts are mapped exactly as documented (file CEP:ASSUMES, conflict 2).
    /// CEP:COST: O(1), const-evaluable, allocation-free; CEP-0.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn overloadable(self) -> bool {
        match self {
            OpId::BitOr
            | OpId::BitXor
            | OpId::BitAnd
            | OpId::Eq
            | OpId::Ne
            | OpId::Lt
            | OpId::Gt
            | OpId::Le
            | OpId::Ge
            | OpId::Shl
            | OpId::Shr
            | OpId::Add
            | OpId::Sub
            | OpId::Mul
            | OpId::Div
            | OpId::Rem
            | OpId::Neg
            | OpId::Not
            | OpId::MethodCall
            | OpId::Index
            | OpId::Call => true,
            OpId::Assign
            | OpId::AddAssign
            | OpId::SubAssign
            | OpId::MulAssign
            | OpId::DivAssign
            | OpId::RemAssign
            | OpId::BitAndAssign
            | OpId::BitOrAssign
            | OpId::BitXorAssign
            | OpId::ShlAssign
            | OpId::ShrAssign
            | OpId::Range
            | OpId::RangeInclusive
            | OpId::RangeFrom
            | OpId::RangeTo
            | OpId::RangeFull
            | OpId::LogicalOr
            | OpId::LogicalAnd
            | OpId::Cast
            | OpId::Deref
            | OpId::Borrow
            | OpId::Field
            | OpId::Try => false,
        }
    }

    /// The overflow behavior class of this row: Panic (8 rows), or the pinned
    /// N/A (36 rows) — Wrap and Unchecked occur in no row of the §2.4.1 table
    /// (Volume 10 intrinsics only) (the "Overflow class" column).
    ///
    /// Refinement anchor: Volume 02 §2.4.1 Overflow class column; exact panic conditions §2.4.4.
    /// Domain: the closed OpId enum (total). Failure set: none.
    /// Certificate: EXH `exh_accessors_match_rows_44`; EXH `exh_overflow_wrap_unchecked_absent`.
    ///
    /// CEP:WHAT: Overflow-class accessor over the closed operator set.
    /// CEP:WHY: IR lowering (§2.4.6) inserts exactly the §2.4.4 checks for the Panic rows.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: The match arms duplicate the table's Overflow column; agreement is EXH-enforced.
    /// CEP:COST: O(1), const-evaluable, allocation-free; CEP-0.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44; EXH exh_overflow_wrap_unchecked_absent.
    pub const fn overflow(self) -> Overflow {
        match self {
            OpId::Shl | OpId::Shr | OpId::Add | OpId::Sub | OpId::Mul | OpId::Div | OpId::Rem | OpId::Neg => Overflow::Panic,
            OpId::Assign
            | OpId::AddAssign
            | OpId::SubAssign
            | OpId::MulAssign
            | OpId::DivAssign
            | OpId::RemAssign
            | OpId::BitAndAssign
            | OpId::BitOrAssign
            | OpId::BitXorAssign
            | OpId::ShlAssign
            | OpId::ShrAssign
            | OpId::Range
            | OpId::RangeInclusive
            | OpId::RangeFrom
            | OpId::RangeTo
            | OpId::RangeFull
            | OpId::LogicalOr
            | OpId::LogicalAnd
            | OpId::BitOr
            | OpId::BitXor
            | OpId::BitAnd
            | OpId::Eq
            | OpId::Ne
            | OpId::Lt
            | OpId::Gt
            | OpId::Le
            | OpId::Ge
            | OpId::Cast
            | OpId::Not
            | OpId::Deref
            | OpId::Borrow
            | OpId::Field
            | OpId::MethodCall
            | OpId::Index
            | OpId::Call
            | OpId::Try => Overflow::NotApplicable,
        }
    }
}

// ---------------------------------------------------------------------------
// Accessors over OperatorRow (field projections; total).
// ---------------------------------------------------------------------------

impl OperatorRow {
    /// The row's precedence level (the "Prec" column of §2.4.1).
    ///
    /// CEP:WHAT: Field projection of the pinned precedence column.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (field read).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn precedence(&self) -> u8 {
        self.precedence
    }

    /// The row's associativity (the "Assoc" column of §2.4.1).
    ///
    /// CEP:WHAT: Field projection of the pinned associativity column.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (field read).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn associativity(&self) -> Assoc {
        self.assoc
    }

    /// The row's arity (the "Arity" column of §2.4.1).
    ///
    /// CEP:WHAT: Field projection of the pinned arity column.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (field read).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn arity(&self) -> Arity {
        self.arity
    }

    /// The row's overloadability (the "Overloadable" column of §2.4.1).
    ///
    /// CEP:WHAT: Field projection of the pinned overloadability column.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (field read).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn overloadable(&self) -> bool {
        self.overloadable
    }

    /// The row's overflow behavior class (the "Overflow class" column of §2.4.1).
    ///
    /// CEP:WHAT: Field projection of the pinned overflow-class column.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (field read).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_accessors_match_rows_44.
    pub const fn overflow(&self) -> Overflow {
        self.overflow
    }
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{
        lookup, precedence_compare, Arity, Assoc, LookupError, OpId, Overflow, OP_FIRST, OP_LAST,
        OPERATOR_TABLE, OPERATOR_TABLE_LEN, PREC_ADDITIVE, PREC_ASSIGN, PREC_BIT_AND, PREC_BIT_OR,
        PREC_BIT_XOR, PREC_CAST, PREC_COMPARISON, PREC_EQUALITY, PREC_LOGICAL_AND,
        PREC_LOGICAL_OR, PREC_MAX, PREC_MIN, PREC_MULTIPLICATIVE, PREC_POSTFIX, PREC_RANGE,
        PREC_SHIFT, PREC_UNARY,
    };
    use core::cmp::Ordering;

    /// ASCII domain size for the exhaustive 1- and 2-byte string enumeration (0x00..=0x7F).
    const ASCII_LEN: usize = 128;
    /// Number of PROP draws, paired with the EXH lookup tests (SPEC-INDEX §4.3).
    const PROP_ROUNDS: u32 = 10_000;
    /// The 6 non-operator delimiters of OB-02-8 (Volume 02 §2.2.20).
    const DELIMITERS: [&str; 6] = ["(", ")", "[", "]", "{", "}"];
    /// PROP sampling alphabet: every operator byte, both range spellings' bytes, delimiters, letters of the qualified keys, and space.
    const PROP_ALPHABET: [u8; 21] = [
        b'!', b'%', b'&', b'(', b')', b'*', b'+', b',', b'-', b'.', b'/', b'0', b'<', b'=', b'>',
        b'a', b'b', b'i', b'm', b'|', b' ',
    ];

    /// Hand transcription of the complete pinned §2.4.1 table, in volume row
    /// order OP-01..OP-44. Each entry is (Op, Token/position, Prec, Assoc,
    /// Arity, Overloadable, Overflow class) exactly as the volume prints it
    /// (markdown backticks stripped; the two cell-encoding conflicts resolved
    /// per the file CEP:ASSUMES). This transcription IS the certificate domain:
    /// it enumerates every table row.
    const EXPECTED: [(OpId, &'static str, u8, Assoc, Arity, bool, Overflow); OPERATOR_TABLE_LEN] = [
        (OpId::Assign, "=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-01
        (OpId::AddAssign, "+=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-02
        (OpId::SubAssign, "-=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-03
        (OpId::MulAssign, "*=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-04
        (OpId::DivAssign, "/=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-05
        (OpId::RemAssign, "%=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-06
        (OpId::BitAndAssign, "&=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-07
        (OpId::BitOrAssign, "|=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-08
        (OpId::BitXorAssign, "^=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-09
        (OpId::ShlAssign, "<<=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-10
        (OpId::ShrAssign, ">>=", PREC_ASSIGN, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-11
        (OpId::Range, "a..b", PREC_RANGE, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-12
        (OpId::RangeInclusive, "a..=b", PREC_RANGE, Assoc::None, Arity::Two, false, Overflow::NotApplicable), // OP-13
        (OpId::RangeFrom, "a..", PREC_RANGE, Assoc::None, Arity::One, false, Overflow::NotApplicable), // OP-14
        (OpId::RangeTo, "..b", PREC_RANGE, Assoc::None, Arity::One, false, Overflow::NotApplicable), // OP-15
        (OpId::RangeFull, "..", PREC_RANGE, Assoc::None, Arity::Zero, false, Overflow::NotApplicable), // OP-16
        (OpId::LogicalOr, "||", PREC_LOGICAL_OR, Assoc::Left, Arity::Two, false, Overflow::NotApplicable), // OP-17
        (OpId::LogicalAnd, "&&", PREC_LOGICAL_AND, Assoc::Left, Arity::Two, false, Overflow::NotApplicable), // OP-18
        (OpId::BitOr, "| (binary)", PREC_BIT_OR, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-19
        (OpId::BitXor, "^ (binary)", PREC_BIT_XOR, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-20
        (OpId::BitAnd, "& (binary)", PREC_BIT_AND, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-21
        (OpId::Eq, "==", PREC_EQUALITY, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-22
        (OpId::Ne, "!=", PREC_EQUALITY, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-23
        (OpId::Lt, "<", PREC_COMPARISON, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-24
        (OpId::Gt, ">", PREC_COMPARISON, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-25
        (OpId::Le, "<=", PREC_COMPARISON, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-26
        (OpId::Ge, ">=", PREC_COMPARISON, Assoc::None, Arity::Two, true, Overflow::NotApplicable), // OP-27
        (OpId::Shl, "<<", PREC_SHIFT, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-28
        (OpId::Shr, ">>", PREC_SHIFT, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-29
        (OpId::Add, "+", PREC_ADDITIVE, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-30
        (OpId::Sub, "- (binary)", PREC_ADDITIVE, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-31
        (OpId::Mul, "* (binary)", PREC_MULTIPLICATIVE, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-32
        (OpId::Div, "/", PREC_MULTIPLICATIVE, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-33
        (OpId::Rem, "%", PREC_MULTIPLICATIVE, Assoc::Left, Arity::Two, true, Overflow::Panic), // OP-34
        (OpId::Cast, "as", PREC_CAST, Assoc::Left, Arity::Two, false, Overflow::NotApplicable), // OP-35
        (OpId::Neg, "- (unary)", PREC_UNARY, Assoc::Right, Arity::One, true, Overflow::Panic), // OP-36
        (OpId::Not, "! (unary)", PREC_UNARY, Assoc::Right, Arity::One, true, Overflow::NotApplicable), // OP-37
        (OpId::Deref, "* (deref)", PREC_UNARY, Assoc::Right, Arity::One, false, Overflow::NotApplicable), // OP-38
        (OpId::Borrow, "& / &mut (borrow)", PREC_UNARY, Assoc::Right, Arity::One, false, Overflow::NotApplicable), // OP-39
        (OpId::Field, ". field", PREC_POSTFIX, Assoc::Left, Arity::Two, false, Overflow::NotApplicable), // OP-40
        (OpId::MethodCall, ".name(args) method call", PREC_POSTFIX, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-41
        (OpId::Index, "[i] index", PREC_POSTFIX, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-42
        (OpId::Call, "(args) call", PREC_POSTFIX, Assoc::Left, Arity::Two, true, Overflow::NotApplicable), // OP-43
        (OpId::Try, "?", PREC_POSTFIX, Assoc::Left, Arity::One, false, Overflow::NotApplicable), // OP-44
    ];

    /// Independent reference for the lookup certificate: a linear scan over the
    /// transcription (a different algorithm and a different data source than
    /// the binary search over OPERATOR_TABLE). Returns the index of the unique
    /// row whose token equals `s`, or None if `s` is not a pinned key.
    fn reference_scan(s: &str) -> Option<usize> {
        let mut i = 0usize;
        while i < EXPECTED.len() {
            if EXPECTED[i].1 == s {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Asserts lookup(s) agrees with the independent reference on every column.
    fn assert_lookup_agrees(s: &str) {
        match reference_scan(s) {
            Some(i) => {
                let exp = &EXPECTED[i];
                match lookup(s) {
                    Ok(row) => {
                        assert_eq!(row.id, exp.0, "id mismatch for key {:?}", exp.1);
                        assert_eq!(row.token, exp.1, "token mismatch for key {:?}", exp.1);
                        assert_eq!(row.precedence, exp.2, "prec mismatch for key {:?}", exp.1);
                        assert_eq!(row.assoc, exp.3, "assoc mismatch for key {:?}", exp.1);
                        assert_eq!(row.arity, exp.4, "arity mismatch for key {:?}", exp.1);
                        assert_eq!(row.overloadable, exp.5, "overloadable mismatch for key {:?}", exp.1);
                        assert_eq!(row.overflow, exp.6, "overflow mismatch for key {:?}", exp.1);
                    }
                    Err(LookupError::UnknownToken) => panic!("binary search missed pinned key {:?}", exp.1),
                }
            }
            None => match lookup(s) {
                Ok(row) => panic!("unexpected hit for non-key {:?} -> {:?}", s, row.token),
                Err(LookupError::UnknownToken) => {}
            },
        }
    }

    /// Pinned in-crate xorshift64* PRNG (seed 0x9E3779B97F4A7C15) for the PROP test; deterministic.
    struct Xorshift64Star {
        state: u64,
    }
    impl Xorshift64Star {
        const SEED: u64 = 0x9E37_79B9_7F4A_7C15;
        const MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;
        const fn new() -> Self {
            Xorshift64Star { state: Self::SEED }
        }
        fn next_u64(&mut self) -> u64 {
            let mut x = self.state;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.state = x;
            x.wrapping_mul(Self::MULTIPLIER)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next_u64() % n
        }
    }

    /// EXH certificate: the embedded table equals the hand-transcribed §2.4.1
    /// table on every row and every column. Domain: all 44 rows x 6 columns,
    /// plus (a) each OpId appears in the table exactly once, (b) each table row
    /// is transcribed exactly once, (c) the OpId declaration order is the volume
    /// row order (index() == position + 1), (d) all precedence levels lie in
    /// PREC_MIN..=PREC_MAX. Full domain, test-as-proof.
    #[test]
    fn exh_table_matches_transcription_44() {
        assert_eq!(OPERATOR_TABLE.len(), OPERATOR_TABLE_LEN);
        assert_eq!(EXPECTED.len(), OPERATOR_TABLE_LEN);
        // (a) every transcribed row is in the table exactly once, with all six columns equal
        for exp in &EXPECTED {
            let mut found = 0usize;
            for row in &OPERATOR_TABLE {
                if row.id == exp.0 {
                    found += 1;
                    assert_eq!(row.token, exp.1, "token {:?}", exp.1);
                    assert_eq!(row.precedence, exp.2, "prec {:?}", exp.1);
                    assert_eq!(row.assoc, exp.3, "assoc {:?}", exp.1);
                    assert_eq!(row.arity, exp.4, "arity {:?}", exp.1);
                    assert_eq!(row.overloadable, exp.5, "overloadable {:?}", exp.1);
                    assert_eq!(row.overflow, exp.6, "overflow {:?}", exp.1);
                }
            }
            assert_eq!(found, 1, "row {:?} found {} times in table", exp.1, found);
        }
        // (b) every table row is transcribed exactly once (no extra row exists)
        for row in &OPERATOR_TABLE {
            let mut found = 0usize;
            for exp in &EXPECTED {
                if exp.0 == row.id {
                    found += 1;
                }
            }
            assert_eq!(found, 1, "table row {:?} transcribed {} times", row.token, found);
        }
        // (c) the transcription order is the volume row order OP-01..OP-44
        for (i, exp) in EXPECTED.iter().enumerate() {
            assert_eq!(exp.0.index(), OP_FIRST + i as u8, "OpId order at position {}", i);
        }
        assert_eq!(EXPECTED[EXPECTED.len() - 1].0.index(), OP_LAST);
        // (d) every transcribed precedence lies in the pinned level range
        for exp in &EXPECTED {
            assert!(exp.2 >= PREC_MIN && exp.2 <= PREC_MAX, "prec {} out of range {:?}", exp.2, exp.1);
        }
    }

    /// EXH certificate (OB-02-8): lookup returns the exact §2.4.1 row for each
    /// of the 44 operator shapes and no row for the 6 non-operator delimiters.
    /// Stated lookup domain, fully enumerated: the 44 pinned keys + the empty
    /// string + all 128 one-byte ASCII strings + all 16,384 two-byte ASCII
    /// strings + the 6 delimiters = 16,563 cases, each checked against the
    /// independent linear-scan reference (hit: all six columns; miss:
    /// LookupError::UnknownToken).
    #[test]
    fn exh_lookup_keys_and_ascii_len_le2() {
        // (a) every pinned key resolves to its exact row
        let mut i = 0usize;
        while i < EXPECTED.len() {
            assert_lookup_agrees(EXPECTED[i].1);
            i += 1;
        }
        // (b) the empty string is not a key
        assert_lookup_agrees("");
        // (c) all 128 one-byte ASCII strings
        let mut b = 0usize;
        while b < ASCII_LEN {
            let buf = [b as u8];
            let s = core::str::from_utf8(&buf).unwrap();
            assert_lookup_agrees(s);
            b += 1;
        }
        // (d) all 16,384 two-byte ASCII strings
        let mut b1 = 0usize;
        while b1 < ASCII_LEN {
            let mut b2 = 0usize;
            while b2 < ASCII_LEN {
                let buf = [b1 as u8, b2 as u8];
                let s = core::str::from_utf8(&buf).unwrap();
                assert_lookup_agrees(s);
                b2 += 1;
            }
            b1 += 1;
        }
        // (e) the 6 non-operator delimiters, named explicitly for OB-02-8
        for d in DELIMITERS {
            assert_lookup_agrees(d);
        }
    }

    /// EXH certificate: the const table is strictly ascending by token key
    /// (sortedness + uniqueness in one property). Domain: all 43 adjacent
    /// pairs of the 44-row array (the whole array).
    #[test]
    fn exh_table_sorted_unique_keys() {
        let mut i = 0usize;
        while i + 1 < OPERATOR_TABLE_LEN {
            let a = OPERATOR_TABLE[i].token;
            let b = OPERATOR_TABLE[i + 1].token;
            assert!(a < b, "keys not strictly ascending at {}: {:?} !< {:?}", i, a, b);
            i += 1;
        }
    }

    /// EXH certificate: every accessor equals its transcribed column on every
    /// row, both entry points (OpId accessors vs EXPECTED; OperatorRow
    /// accessors vs their own fields), plus the numeric arity projection.
    /// Domain: 44 rows x 8 checks (full domain).
    #[test]
    fn exh_accessors_match_rows_44() {
        for exp in &EXPECTED {
            let op = exp.0;
            assert_eq!(op.index(), exp.0.index(), "index {:?}", exp.1);
            assert_eq!(op.precedence(), exp.2, "prec {:?}", exp.1);
            assert_eq!(op.associativity(), exp.3, "assoc {:?}", exp.1);
            assert_eq!(op.arity(), exp.4, "arity {:?}", exp.1);
            assert_eq!(op.overloadable(), exp.5, "overloadable {:?}", exp.1);
            assert_eq!(op.overflow(), exp.6, "overflow {:?}", exp.1);
            let want_count = match exp.4 {
                Arity::Zero => 0u8,
                Arity::One => 1,
                Arity::Two => 2,
            };
            assert_eq!(op.arity().count(), want_count, "arity count {:?}", exp.1);
        }
        for row in &OPERATOR_TABLE {
            let exp_idx = reference_scan(row.token).expect("table key must be a transcribed key");
            let exp = &EXPECTED[exp_idx];
            assert_eq!(row.id, exp.0);
            assert_eq!(row.precedence(), row.precedence);
            assert_eq!(row.precedence, exp.2);
            assert_eq!(row.associativity(), row.assoc);
            assert_eq!(row.assoc, exp.3);
            assert_eq!(row.arity(), row.arity);
            assert_eq!(row.arity, exp.4);
            assert_eq!(row.overloadable(), row.overloadable);
            assert_eq!(row.overloadable, exp.5);
            assert_eq!(row.overflow(), row.overflow);
            assert_eq!(row.overflow, exp.6);
        }
    }

    /// EXH certificate: precedence_compare equals the reference ordering for
    /// every ordered pair of operators. Domain: all 44 x 44 = 1,936 pairs
    /// (full domain, <= 10^6).
    #[test]
    fn exh_precedence_compare_all_pairs_1936() {
        for a in &EXPECTED {
            for b in &EXPECTED {
                let want = a.2.cmp(&b.2);
                let got = precedence_compare(a.0, b.0);
                assert_eq!(got, want, "compare {:?} vs {:?}", a.1, b.1);
                // The three outcomes are exactly the pinned semantics: higher binds tighter.
                match got {
                    Ordering::Less => assert!(a.2 < b.2),
                    Ordering::Greater => assert!(a.2 > b.2),
                    Ordering::Equal => assert_eq!(a.2, b.2),
                }
            }
        }
    }

    /// EXH certificate: the Wrap and Unchecked classes occur in no row of the
    /// table (Volume 02 §2.4.1: they belong to the Volume 10 intrinsics), and
    /// the Panic set is exactly the 8 rows OP-28..OP-34 and OP-36.
    /// Domain: all 44 rows (full domain).
    #[test]
    fn exh_overflow_wrap_unchecked_absent() {
        for exp in &EXPECTED {
            assert!(
                exp.6 == Overflow::Panic || exp.6 == Overflow::NotApplicable,
                "row {:?} carries {:?}; Wrap/Unchecked belong to no §2.4.1 row",
                exp.1,
                exp.6
            );
            let is_panic_row = matches!(
                exp.0,
                OpId::Shl | OpId::Shr | OpId::Add | OpId::Sub | OpId::Mul | OpId::Div | OpId::Rem | OpId::Neg
            );
            assert_eq!(exp.6 == Overflow::Panic, is_panic_row, "panic set membership {:?}", exp.1);
        }
        let panic_rows = OPERATOR_TABLE
            .iter()
            .filter(|r| r.overflow == Overflow::Panic)
            .count();
        assert_eq!(panic_rows, 8, "exactly 8 Panic rows expected");
    }

    /// PROP certificate (paired with exh_lookup_keys_and_ascii_len_le2 per
    /// SPEC-INDEX §4.3): 10,000 deterministic PRNG draws — random short strings
    /// over the operator alphabet, exact pinned keys, and single-byte-mutated
    /// pinned keys — agree with the independent linear-scan reference.
    /// Sampling domain: strings of length 0..=24 over the 21-byte PROP_ALPHABET
    /// and the pinned key set.
    #[test]
    fn prop_lookup_vs_linear_reference() {
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < PROP_ROUNDS {
            let mode = rng.below(3);
            match mode {
                0 => {
                    // Random string, length 1..=8, over the operator alphabet.
                    let len = 1usize + rng.below(8) as usize;
                    let mut buf = [b' '; 8];
                    let mut k = 0usize;
                    while k < len {
                        buf[k] = PROP_ALPHABET[rng.below(PROP_ALPHABET.len() as u64) as usize];
                        k += 1;
                    }
                    let s = core::str::from_utf8(&buf[..len]).unwrap();
                    assert_lookup_agrees(s);
                }
                1 => {
                    // Exact pinned key.
                    let key = EXPECTED[rng.below(EXPECTED.len() as u64) as usize].1;
                    assert_lookup_agrees(key);
                }
                _ => {
                    // Single-byte-mutated pinned key: near-miss probes of the binary search.
                    let exp = &EXPECTED[rng.below(EXPECTED.len() as u64) as usize];
                    let token = exp.1;
                    let mut buf = [b' '; 32];
                    assert!(token.len() <= buf.len(), "transcription token exceeds probe buffer");
                    buf[..token.len()].copy_from_slice(token.as_bytes());
                    let pos = rng.below(token.len() as u64) as usize;
                    buf[pos] = PROP_ALPHABET[rng.below(PROP_ALPHABET.len() as u64) as usize];
                    let s = core::str::from_utf8(&buf[..token.len()]).unwrap();
                    assert_lookup_agrees(s);
                }
            }
            n += 1;
        }
    }

    /// Contract spot-check: the pinned column domains are exactly as pinned —
    /// precedence levels are the 15 named constants, and the row accessor
    /// entry points agree with the OpId entry points for the whole table.
    #[test]
    fn exh_column_domains_and_accessor_agreement() {
        // The 15 pinned levels, named (Law 7).
        const LEVELS: [u8; 15] = [
            PREC_ASSIGN,
            PREC_RANGE,
            PREC_LOGICAL_OR,
            PREC_LOGICAL_AND,
            PREC_BIT_OR,
            PREC_BIT_XOR,
            PREC_BIT_AND,
            PREC_EQUALITY,
            PREC_COMPARISON,
            PREC_SHIFT,
            PREC_ADDITIVE,
            PREC_MULTIPLICATIVE,
            PREC_CAST,
            PREC_UNARY,
            PREC_POSTFIX,
        ];
        // Each named level is distinct and in ascending pinned order.
        let mut i = 0usize;
        while i + 1 < LEVELS.len() {
            assert!(LEVELS[i] < LEVELS[i + 1], "pinned levels not ascending at {}", i);
            i += 1;
        }
        assert_eq!(LEVELS[0], PREC_MIN);
        assert_eq!(LEVELS[LEVELS.len() - 1], PREC_MAX);
        // Row accessors == OpId accessors for every table row (both API entry points).
        for row in &OPERATOR_TABLE {
            assert_eq!(row.precedence(), row.id.precedence());
            assert_eq!(row.associativity(), row.id.associativity());
            assert_eq!(row.arity(), row.id.arity());
            assert_eq!(row.overloadable(), row.id.overloadable());
            assert_eq!(row.overflow(), row.id.overflow());
        }
    }

    /// Completeness spot-check of the crate's failure set: LookupError is
    /// constructible and matchable in exactly one variant — there is no other
    /// variant to construct, which is what "complete" means here.
    #[test]
    fn exh_lookuperror_is_complete_single_variant() {
        let err = LookupError::UnknownToken;
        match err {
            LookupError::UnknownToken => {}
        }
        assert_eq!(err, LookupError::UnknownToken);
        assert_ne!(lookup("="), Err(LookupError::UnknownToken)); // a pinned key never fails
    }
}
