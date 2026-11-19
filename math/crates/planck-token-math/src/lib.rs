// CEP:FILE: math/crates/planck-token-math/src/lib.rs
// CEP:WHAT: Literal and token mathematics of Planck 0.1 edition P1 (Volume 02): integer literal radix folding over the arbitrary-precision value space (§2.3.1), exact integer-type range checks (§2.3.3), the identifier grammar validator (§2.2.1), the float-literal grammar validator (§2.2.8), the complete LiteralError set (§2.3.11), and char-literal body decoding with the pinned escape table (§2.2.10, §2.2.14, §2.3.6).
// CEP:WHY: SPEC-INDEX §5.1 pins planck-token-math as the extraction home of the identifier grammar and the integer/float literal value spaces; §5.6 pins radix folding, range checking, and totality (Result everywhere, complete error enums), so the frontend must call exactly these functions for literal math.
// CEP:STATUS: complete
// CEP:FAILURE: LiteralError, which is complete over this crate's failure set; every variant maps to one exact Volume 02 §2.3.11 diagnostic code: EmptyDigits (E-LEX-06), InvalidDigit (E-LIT-01), ValueOverflow (E-LIT-02), TooManyDigits (E-LEX-12), EmptyCharLiteral (E-LEX-07), MultipleChars (E-LEX-07), InvalidEscape (E-LEX-08), SurrogateCodePoint (E-LIT-05 char-literal rule / E-LIT-04 escape rule), CodePointOutOfRange (E-LIT-04). The three grammar validators return bool and have no failure set (total predicates). No panic, no unwrap, no unsafe in non-test code.
// CEP:ASSUMES: Volume 02 §2.2.0 digit classes (embedded locally by value to keep the crate dependency-free per the kernel law); §2.2.7 DigitGroup underscore rules and MAX_LIT_DIGITS = 4096; §2.2.8 float grammar and validity sentence; §2.3.10 (a negative number is not a literal, so literal values are non-negative); §2.1.5 (the tokenizer only receives pre-validated UTF-8); SPEC-INDEX §5.2 (CEP-0 law).
// CEP:COST: O(digits) per parse with checked u128 arithmetic, allocation-free; validators are O(len) single passes over the byte slice, allocation-free.
// CEP:EVIDENCE: EXH exh_radix_fold_len_le4_union_alphabet (OB-02-4: every digit string of length <= 4 over the 22-byte union alphabet x 4 radices vs an independent reference fold); EXH exh_invalid_digit_sets_all_256 (per radix, all 256 bytes classified); EXH exh_overflow_and_length_boundaries; EXH exh_fits_type_boundaries_12_types (OB-02-5); EXH exh_identifier_full_domain_len_le3 + exh_identifier_single_bytes_256; EXH exh_float_len_le4_alphabet_11 + exh_float_boundary_cases; EXH exh_char_escapes_xhh_256 + exh_char_named_escapes_and_u_boundaries (OB-02-7) + exh_char_literal_raw_ascii_256; PROP prop_radix_fold_vs_reference / prop_identifier_vs_reference / prop_float_vs_reference (20,000 cases each, deterministic PRNG, paired with the EXH tests above).
// CEP:FV-CLASS: FV-1
// CEP:FV-PROVER: Rust test harness (EXH over the stated bounded obligation domains + paired deterministic PROP sampling)
// CEP:FV-SCRIPT: crates/planck-token-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: The EXH domains are exactly the pinned obligation domains of Volume 02 §2.8 (OB-02-4 digit strings of length <= 4; OB-02-5 boundary values; OB-02-7 all 256 \xHH values and the 8 scalar boundaries of \u). Inputs outside those bounds (longer digit strings; float/identifier strings beyond the stated alphabet/length) are covered by PROP sampling paired with the EXH per SPEC-INDEX §4.3, and by the runtime bridge of Volume 02 §2.7: all compilation paths call this same function, so equivalence holds by implementation identity. No sorries; the gap is bounded and named.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named specification constants (Law 7).
// ---------------------------------------------------------------------------

/// Maximum number of digits in one literal digit group; a longer digit string is E-LEX-12 (Volume 02 §2.2.7).
pub const MAX_LIT_DIGITS: usize = 4096;

/// Maximum Unicode scalar value U+10FFFF (Volume 02 §2.3.7).
pub const UNICODE_SCALAR_MAX: u32 = 0x10_FFFF;
/// Lower bound of the surrogate block U+D800 (Volume 02 §2.3.6-§2.3.7).
pub const SURROGATE_MIN: u32 = 0xD800;
/// Upper bound of the surrogate block U+DFFF (Volume 02 §2.3.6-§2.3.7).
pub const SURROGATE_MAX: u32 = 0xDFFF;
/// Upper bound of the `\xHH` byte value in char/string contexts, 0x7F (Volume 02 §2.2.14).
pub const CHAR_CONTEXT_X_MAX: u32 = 0x7F;

/// Exact maximum of the pinned integer type bounds table, i8 (Volume 02 §2.3.3).
pub const I8_MAX: u128 = 127;
/// Exact maximum of the pinned integer type bounds table, i16 (Volume 02 §2.3.3).
pub const I16_MAX: u128 = 32_767;
/// Exact maximum of the pinned integer type bounds table, i32 (Volume 02 §2.3.3).
pub const I32_MAX: u128 = 2_147_483_647;
/// Exact maximum of the pinned integer type bounds table, i64 (Volume 02 §2.3.3).
pub const I64_MAX: u128 = 9_223_372_036_854_775_807;
/// Exact maximum of the pinned integer type bounds table, i128 (Volume 02 §2.3.3).
pub const I128_MAX: u128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
/// Exact maximum of the pinned integer type bounds table, u8 (Volume 02 §2.3.3).
pub const U8_MAX: u128 = 255;
/// Exact maximum of the pinned integer type bounds table, u16 (Volume 02 §2.3.3).
pub const U16_MAX: u128 = 65_535;
/// Exact maximum of the pinned integer type bounds table, u32 (Volume 02 §2.3.3).
pub const U32_MAX: u128 = 4_294_967_295;
/// Exact maximum of the pinned integer type bounds table, u64 (Volume 02 §2.3.3).
pub const U64_MAX: u128 = 18_446_744_073_709_551_615;
/// Exact maximum of the pinned integer type bounds table, u128 (Volume 02 §2.3.3).
pub const U128_MAX: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455;

/// u8 class bounds embedded by value from Volume 02 §2.2.0 (kept local to keep zero dependencies per SPEC-INDEX §5.2).
const DEC_DIGIT_MIN: u8 = 0x30;
const DEC_DIGIT_MAX: u8 = 0x39;
const UPPER_HEX_MIN: u8 = 0x41;
const UPPER_HEX_MAX: u8 = 0x46;
const LOWER_HEX_MIN: u8 = 0x61;
const LOWER_HEX_MAX: u8 = 0x66;
const UPPER_MIN: u8 = 0x41;
const UPPER_MAX: u8 = 0x5A;
const LOWER_MIN: u8 = 0x61;
const LOWER_MAX: u8 = 0x7A;
const UNDERSCORE: u8 = 0x5F;
const BACKSLASH: u8 = 0x5C;
const DOT: u8 = 0x2E;

/// Local decimal-digit predicate (Volume 02 §2.2.0 `is_dec_digit`, embedded by value; crate stays dependency-free).
const fn is_dec_digit(b: u8) -> bool {
    b >= DEC_DIGIT_MIN && b <= DEC_DIGIT_MAX
}

/// Local identifier-start predicate (Volume 02 §2.2.0 `is_ident_start`, embedded by value).
const fn ident_start(b: u8) -> bool {
    (b >= UPPER_MIN && b <= UPPER_MAX) || (b >= LOWER_MIN && b <= LOWER_MAX) || b == UNDERSCORE
}

/// Local identifier-continue predicate (Volume 02 §2.2.0 `is_ident_continue`, embedded by value).
const fn ident_continue(b: u8) -> bool {
    ident_start(b) || is_dec_digit(b)
}

/// Local hex-digit predicate (Volume 02 §2.2.0 `is_hex_digit`, embedded by value).
const fn is_hex_digit(b: u8) -> bool {
    is_dec_digit(b)
        || (b >= UPPER_HEX_MIN && b <= UPPER_HEX_MAX)
        || (b >= LOWER_HEX_MIN && b <= LOWER_HEX_MAX)
}

/// Local hex value 0..15 for a byte already checked by `is_hex_digit`.
/// CEP:ASSUMES: caller checked is_hex_digit(b), so b - DEC_DIGIT_MIN cannot underflow on any branch.
const fn hex_val(b: u8) -> u32 {
    if b <= 0x39 {
        (b - DEC_DIGIT_MIN) as u32
    } else if b >= LOWER_HEX_MIN {
        (b - LOWER_HEX_MIN) as u32 + 10
    } else {
        (b - UPPER_HEX_MIN) as u32 + 10
    }
}

// ---------------------------------------------------------------------------
// Error enum (complete; each variant maps to one exact §2.3.11 diagnostic code).
// ---------------------------------------------------------------------------

/// The complete failure set of this crate's parsing functions (Volume 02 §2.3.11).
///
/// Every variant names its exact diagnostic code; the error enum is complete: every
/// `Err` this crate can return is exactly one of these variants, and every variant is
/// reachable from at least one public function (EXH-tested).
///
/// CEP:WHAT: Complete literal/token error set with exact Volume 02 §2.3.11 code mapping.
/// CEP:WHY: SPEC-INDEX §5.2 requires explicit, complete error enums and a listed FAILURE set.
/// CEP:STATUS: complete
/// CEP:FAILURE: This type IS the failure set; see variant docs for the per-variant code.
/// CEP:ASSUMES: radix and byte payloads carry the offending values exactly (first bad digit per §2.3.11 span rule).
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_invalid_digit_sets_all_256, exh_overflow_and_length_boundaries, exh_char_escapes_xhh_256, exh_char_named_escapes_and_u_boundaries, exh_char_literal_raw_ascii_256 (every variant constructed and matched).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiteralError {
    /// Digit group with zero digits: the radix prefix must be followed by >= 1 digit (E-LEX-06, Volume 02 §2.2.7).
    EmptyDigits,
    /// A digit invalid for the radix; `byte` is the first offending digit (E-LIT-01, Volume 02 §2.2.7/§2.3.1/§2.3.11).
    InvalidDigit { radix: Radix, byte: u8 },
    /// The folded literal value exceeds the representable value space; `max_value` is the maximum representable u128 (E-LIT-02 class at the literal-value level, Volume 02 §2.3.1/§2.3.3; overflow at the literal level is a compile-time rejection, never a runtime event).
    ValueOverflow { max_value: u128 },
    /// Digit count exceeds MAX_LIT_DIGITS = 4096 (E-LEX-12, Volume 02 §2.2.7/§2.3.11).
    TooManyDigits { max: usize },
    /// Char literal body is empty: not exactly one scalar (E-LEX-07, Volume 02 §2.2.10/§2.3.11).
    EmptyCharLiteral,
    /// Char literal body is not exactly one CharBody scalar: multiple scalars, a raw excluded byte (' , \, LF, CR), or invalid UTF-8 (E-LEX-07, Volume 02 §2.2.10/§2.3.11; §2.1.5 makes invalid UTF-8 unreachable from the tokenizer).
    MultipleChars,
    /// Unknown escape name, malformed `\xHH` (bad hex or out of the char/string context range 0x00..=0x7F), or malformed `\u{...}` production (E-LEX-08, Volume 02 §2.2.14/§2.3.11).
    InvalidEscape,
    /// Char scalar is a surrogate U+D800..=U+DFFF (E-LIT-05 char-literal rule §2.3.6; the same payload in a `\u{...}` escape is E-LIT-04 per §2.2.14 — same rejection class, code chosen by context).
    SurrogateCodePoint,
    /// `\u{...}` payload exceeds U+10FFFF and is not a scalar value (E-LIT-04, Volume 02 §2.2.14/§2.3.7/§2.3.11).
    CodePointOutOfRange { value: u32, max: u32 },
}

// ---------------------------------------------------------------------------
// Radix / integer types.
// ---------------------------------------------------------------------------

/// Literal radix, exactly the pinned set r ∈ {2, 8, 10, 16} (Volume 02 §2.2.7, §2.3.1).
///
/// CEP:WHAT: Closed radix enumeration of the integer literal grammar.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum; no other radix exists in edition P1).
/// CEP:ASSUMES: Volume 02 §2.2.7 RadixPrefix = "0x" | "0o" | "0b" and default decimal.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_invalid_digit_sets_all_256 (per-radix digit sets).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Radix {
    /// Binary radix 2 ("0b").
    Bin,
    /// Octal radix 8 ("0o").
    Oct,
    /// Decimal radix 10 (default).
    Dec,
    /// Hexadecimal radix 16 ("0x").
    Hex,
}

impl Radix {
    /// The radix base value, exactly 2, 8, 10, or 16 (Volume 02 §2.3.1).
    ///
    /// CEP:WHAT: Base value of the radix for the pinned fold v' = v*r + d.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_radix_fold_len_le4_union_alphabet.
    pub const fn value(self) -> u128 {
        match self {
            Radix::Bin => 2,
            Radix::Oct => 8,
            Radix::Dec => 10,
            Radix::Hex => 16,
        }
    }

    /// Digit value of `b` for this radix, or None if the byte is invalid for the radix (E-LIT-01).
    ///
    /// Exact digit sets (Volume 02 §2.2.7): Bin {0,1}; Oct {0..=7}; Dec {0..=9};
    /// Hex {0..=9, A..=F, a..=F}. Uppercase hex digits are valid.
    ///
    /// CEP:WHAT: Per-radix digit classification and value, the E-LIT-01 judgment.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: None for bytes outside the radix digit set (the caller maps to InvalidDigit).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1), allocation-free, const-evaluable.
    /// CEP:EVIDENCE: EXH exh_invalid_digit_sets_all_256 (all 256 bytes per radix).
    pub const fn digit_value(self, b: u8) -> Option<u32> {
        match self {
            Radix::Bin => {
                if b == 0x30 || b == 0x31 {
                    Some((b - DEC_DIGIT_MIN) as u32)
                } else {
                    None
                }
            }
            Radix::Oct => {
                if b >= DEC_DIGIT_MIN && b <= 0x37 {
                    Some((b - DEC_DIGIT_MIN) as u32)
                } else {
                    None
                }
            }
            Radix::Dec => {
                if is_dec_digit(b) {
                    Some((b - DEC_DIGIT_MIN) as u32)
                } else {
                    None
                }
            }
            Radix::Hex => {
                if is_dec_digit(b) {
                    Some((b - DEC_DIGIT_MIN) as u32)
                } else if b >= LOWER_HEX_MIN && b <= LOWER_HEX_MAX {
                    Some((b - LOWER_HEX_MIN) as u32 + 10)
                } else if b >= UPPER_HEX_MIN && b <= UPPER_HEX_MAX {
                    Some((b - UPPER_HEX_MIN) as u32 + 10)
                } else {
                    None
                }
            }
        }
    }
}

/// Target pointer width for the per-target usize/isize bounds (Volume 02 §2.3.3: 64-bit targets equal u64/i64; the declared i686 target equals u32/i32).
///
/// CEP:WHAT: Pointer-width parameter pinning the per-target integer bounds.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum: only the two declared target families exist in edition P1).
/// CEP:ASSUMES: Volume 02 §1.4.8/§2.3.3 target-pointer-width policy.
/// CEP:COST: Copy type.
/// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PtrWidth {
    /// 32-bit pointer width (the declared i686 target family).
    P32,
    /// 64-bit pointer width (the default target family).
    P64,
}

/// Integer literal target type for range checking; exactly the pinned bounds-table types (Volume 02 §2.2.7 suffixes, §2.3.3 bounds).
///
/// CEP:WHAT: Closed enumeration of the integer types a literal can be checked against.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (closed enum over the 12 pinned integer types).
/// CEP:ASSUMES: Volume 02 §2.3.3 bounds table is the normative min/max source.
/// CEP:COST: Copy type.
/// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IntTy {
    /// i8.
    I8,
    /// i16.
    I16,
    /// i32.
    I32,
    /// i64.
    I64,
    /// i128.
    I128,
    /// u8.
    U8,
    /// u16.
    U16,
    /// u32.
    U32,
    /// u64.
    U64,
    /// u128.
    U128,
    /// isize (bounds per target pointer width).
    Isize,
    /// usize (bounds per target pointer width).
    Usize,
}

impl IntTy {
    /// Exact `max(T)` of the pinned bounds table (Volume 02 §2.3.3).
    ///
    /// CEP:WHAT: Upper bound max(T) per type and pointer width.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: Volume 02 §2.3.3 table; usize/isize per PtrWidth.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types (consts cross-checked against the §2.3.3 decimal table).
    pub const fn max_value(self, pw: PtrWidth) -> u128 {
        match self {
            IntTy::I8 => I8_MAX,
            IntTy::I16 => I16_MAX,
            IntTy::I32 => I32_MAX,
            IntTy::I64 => I64_MAX,
            IntTy::I128 => I128_MAX,
            IntTy::U8 => U8_MAX,
            IntTy::U16 => U16_MAX,
            IntTy::U32 => U32_MAX,
            IntTy::U64 => U64_MAX,
            IntTy::U128 => U128_MAX,
            IntTy::Isize => match pw {
                PtrWidth::P32 => I32_MAX,
                PtrWidth::P64 => I64_MAX,
            },
            IntTy::Usize => match pw {
                PtrWidth::P32 => U32_MAX,
                PtrWidth::P64 => U64_MAX,
            },
        }
    }

    /// Exact `min(T)` of the pinned bounds table (Volume 02 §2.3.3).
    ///
    /// CEP:WHAT: Lower bound min(T) per type and pointer width (documentation of the full pinned range).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total const function).
    /// CEP:ASSUMES: Volume 02 §2.3.3 table; min(T) <= 0 <= v always holds for literal values (§2.3.10), so it never rejects.
    /// CEP:COST: O(1), const-evaluable.
    /// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types.
    pub const fn min_value(self, pw: PtrWidth) -> i128 {
        match self {
            IntTy::I8 => -128,
            IntTy::I16 => -32_768,
            IntTy::I32 => -2_147_483_648,
            IntTy::I64 => -9_223_372_036_854_775_808,
            IntTy::I128 => -170_141_183_460_469_231_731_687_303_715_884_105_728,
            IntTy::U8 | IntTy::U16 | IntTy::U32 | IntTy::U64 | IntTy::U128 => 0,
            IntTy::Isize => match pw {
                PtrWidth::P32 => -2_147_483_648,
                PtrWidth::P64 => -9_223_372_036_854_775_808,
            },
            IntTy::Usize => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Integer literal folding (Volume 02 §2.2.7, §2.3.1; SPEC-INDEX §5.6).
// ---------------------------------------------------------------------------

/// Fold a digit string of the given radix into its literal value (Volume 02 §2.3.1, pinned: `v_{i+1} = v_i · r + digit_i`, starting `v_0 = 0`).
///
/// Refinement anchor: Volume 02 §2.2.7 (digit sets, MAX_LIT_DIGITS) and §2.3.1 (arbitrary-precision fold), pinned by SPEC-INDEX §5.6.
/// Input domain: `digits` is the digit group with radix prefixes, integer suffixes, and separator underscores already removed (the fold is over digits; underscores are a tokenization concern and are invalid digits here); every byte is any of the 256 u8 values; length 0..=usize::MAX.
/// Failure set: `TooManyDigits` (len > MAX_LIT_DIGITS, E-LEX-12), `EmptyDigits` (len = 0, E-LEX-06), `InvalidDigit { radix, byte }` (first bad digit, E-LIT-01), `ValueOverflow { max_value }` (fold exceeds u128; `max_value` = U128_MAX, the max representable). Total function: every input yields exactly one outcome.
/// Certificate: EXH `exh_radix_fold_len_le4_union_alphabet` (OB-02-4 full domain), `exh_invalid_digit_sets_all_256`, `exh_overflow_and_length_boundaries`; PROP `prop_radix_fold_vs_reference` (paired with EXH).
///
/// CEP:WHAT: Radix fold of the pinned integer literal interpretation with checked arithmetic.
/// CEP:WHY: §2.3.1 defines the literal value of record; §2.3.2 keeps the digit string until type fixing, so the kernel must fold without truncation or overflow panics.
/// CEP:STATUS: complete
/// CEP:FAILURE: LiteralError::{TooManyDigits, EmptyDigits, InvalidDigit, ValueOverflow} — see the enum docs for the exact diagnostic codes.
/// CEP:ASSUMES: digits contain no separators (documented above); checked_mul/checked_add make the fold panic-free; overflow returns ValueOverflow with the max representable value.
/// CEP:COST: O(len) time, O(1) space, no allocation; CEP-0.
/// CEP:EVIDENCE: EXH exh_radix_fold_len_le4_union_alphabet; EXH exh_invalid_digit_sets_all_256; EXH exh_overflow_and_length_boundaries; PROP prop_radix_fold_vs_reference.
pub fn parse_integer_radix(digits: &[u8], radix: Radix) -> Result<u128, LiteralError> {
    // E-LEX-12 (§2.2.7): a longer digit string is rejected at the literal level.
    if digits.len() > MAX_LIT_DIGITS {
        return Err(LiteralError::TooManyDigits { max: MAX_LIT_DIGITS });
    }
    // E-LEX-06 (§2.2.7): the radix prefix must be followed by >= 1 digit.
    if digits.is_empty() {
        return Err(LiteralError::EmptyDigits);
    }
    let r = radix.value();
    let mut v: u128 = 0;
    for &b in digits {
        // E-LIT-01 (§2.3.1): a digit invalid for r is rejected; the error carries the first bad digit.
        let d = match radix.digit_value(b) {
            Some(d) => d,
            None => return Err(LiteralError::InvalidDigit { radix, byte: b }),
        };
        // Checked fold: overflow is a defined error, never a panic (§5.2 kernel law).
        v = match v.checked_mul(r).and_then(|m| m.checked_add(d as u128)) {
            Some(next) => next,
            None => return Err(LiteralError::ValueOverflow { max_value: U128_MAX }),
        };
    }
    Ok(v)
}

/// Is the non-negative literal value `v` inside the pinned range of integer type `ty` on the default 64-bit target family?
///
/// Refinement anchor: Volume 02 §2.3.3 (pinned: well-formed iff min(T) <= v <= max(T)), §2.3.10 (a negative number is not a literal, so v >= 0 >= min(T) always; only the max side can reject), SPEC-INDEX §5.6.
/// Input domain: all u128 values x all 12 IntTy variants (total).
/// Failure set: none (total predicate; out-of-range is `false`, not an error).
/// Certificate: EXH `exh_fits_type_boundaries_12_types` (OB-02-5: the boundary values per type).
///
/// CEP:WHAT: Literal acceptance exactly = min(T) <= v <= max(T) for the 64-bit default family.
/// CEP:WHY: §2.3.3 pins range checking as a compile-time rejection (E-LIT-02), never a runtime event; this is that judgment.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total predicate).
/// CEP:ASSUMES: usize/isize resolve to the 64-bit bounds per §2.3.3; use `fits_type_on` for the declared i686 family.
/// CEP:COST: O(1), allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types.
pub fn fits_type(value: u128, ty: IntTy) -> bool {
    fits_type_on(value, ty, PtrWidth::P64)
}

/// `fits_type` with an explicit target pointer width for usize/isize (Volume 02 §2.3.3).
///
/// Refinement anchor: Volume 02 §2.3.3 per-target row ("64-bit targets equal u64/i64; the declared i686 target equals u32/i32").
/// Input domain: all u128 values x 12 types x 2 widths (total). Failure set: none.
/// Certificate: EXH `exh_fits_type_boundaries_12_types` (both widths at the boundaries).
///
/// CEP:WHAT: Range check with the exact per-target usize/isize bounds.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total predicate).
/// CEP:ASSUMES: PtrWidth carries the declared target families only.
/// CEP:COST: O(1), allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_fits_type_boundaries_12_types.
pub fn fits_type_on(value: u128, ty: IntTy, pw: PtrWidth) -> bool {
    // min(T) <= v holds for every literal value because v >= 0 >= min(T) (§2.3.10);
    // the check therefore reduces exactly to the max side of the pinned table.
    value <= ty.max_value(pw)
}

// ---------------------------------------------------------------------------
// Identifier grammar (Volume 02 §2.2.1).
// ---------------------------------------------------------------------------

/// Is `bytes` exactly one identifier token of the edition P1 grammar?
///
/// Refinement anchor: Volume 02 §2.2.1 (`Identifier = IDENT_START IDENT_CONT*`, IDENT_START/IDENT_CONT per §2.2.0) and §2.2.2 (ASCII identifiers in P1).
/// Input domain: any byte string (length 0..=usize::MAX; total).
/// Failure set: none (total predicate). `false` means "not an identifier token".
/// Exactness notes: (1) the token consisting of exactly `_` is classed **Wildcard**, not Identifier (§2.2.1), so `is_valid_identifier(b"_")` is false while `b"_a"` is true; (2) keywords are NOT excluded here — keyword classification is a lexer step applied after the identifier scan (§2.2.21 rule 4), so `b"as"` is a valid identifier scan; (3) identifiers are ASCII (§2.2.2): any byte >= 0x80 makes the result false.
/// Certificate: EXH `exh_identifier_full_domain_len_le3` + `exh_identifier_single_bytes_256`; PROP `prop_identifier_vs_reference` (paired with EXH).
///
/// CEP:WHAT: Identifier grammar validator, exact per the §2.2.1 production.
/// CEP:WHY: Name resolution and keyword classification need the token-level identifier judgment, including the Wildcard carve-out.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total predicate).
/// CEP:ASSUMES: input bytes are raw source bytes; the tokenizer supplies maximal-munch scans.
/// CEP:COST: O(len) single pass, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_identifier_full_domain_len_le3; EXH exh_identifier_single_bytes_256; PROP prop_identifier_vs_reference.
pub fn is_valid_identifier(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false; // grammar requires >= 1 IDENT_START
    }
    if bytes.len() == 1 && bytes[0] == UNDERSCORE {
        return false; // exactly "_" is the Wildcard token, not an Identifier (§2.2.1)
    }
    if !ident_start(bytes[0]) {
        return false;
    }
    let mut i = 1;
    while i < bytes.len() {
        if !ident_continue(bytes[i]) {
            return false;
        }
        i += 1;
    }
    true
}

// ---------------------------------------------------------------------------
// Float literal grammar (Volume 02 §2.2.8).
// ---------------------------------------------------------------------------

/// Is `bytes` exactly one float-literal token of the edition P1 grammar?
///
/// Refinement anchor: Volume 02 §2.2.8 (`FloatLiteral = FloatBody FloatSuffix?`, `FloatBody = DigitGroup "." DigitGroup Exp? | DigitGroup Exp`, `Exp = ("e"|"E") ["+"|"-"] DigitGroup`, `FloatSuffix = "f32" | "f64"`) with the underscore rules of §2.2.7 (underscores only between digits; a digit group must not start or end with `_`; no two adjacent `_`) and the validity sentence (a float literal must contain a fraction part with >= 1 digit after `.`, or an exponent with >= 1 digit, or a float suffix — `1f32` is valid, `1.` is not a token).
/// Input domain: any byte string (total). The validator checks the WHOLE input as one token (full consumption required); maximal-valid-prefix retry is tokenizer behavior (§2.2.8), not validator behavior.
/// Failure set: none (total predicate).
/// Exactness notes: (1) `1.` and `1.e3` are invalid (the fraction DigitGroup needs >= 1 digit); (2) the pre-suffix `_` slot exists only in the §2.2.7 INTEGER production, so `1_f32` is invalid for floats (the FloatLiteral production has no underscore slot; "underscore rules are those of §2.2.7" imports the digit-group rules); (3) `1f32` is valid per the explicit validity sentence although it derives only through DigitGroup FloatSuffix?; (4) a numeric token followed by an IDENT_START byte is E-LEX-06 — realized here as full-consumption failure.
/// Certificate: EXH `exh_float_len_le4_alphabet_11` (all strings of length <= 4 over an 11-byte alphabet vs an independent segment-enumeration reference), `exh_float_boundary_cases`; PROP `prop_float_vs_reference` (paired with EXH).
///
/// CEP:WHAT: Float-literal grammar validator, exact per the §2.2.8 productions and validity sentence.
/// CEP:WHY: The tokenizer tries FloatLiteral before IntegerLiteral (§2.2.21 rule 6) and needs the exact acceptance set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total predicate).
/// CEP:ASSUMES: exponent digit groups follow the §2.2.7 DigitGroup underscore rules; suffixes are lowercase exactly `f32`/`f64`.
/// CEP:COST: O(len) single pass, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_float_len_le4_alphabet_11; EXH exh_float_boundary_cases; PROP prop_float_vs_reference.
pub fn is_valid_float_literal(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len == 0 {
        return false;
    }
    // Mantissa DigitGroup (§2.2.7 rules): >= 1 digit, underscores only between digits.
    let end1 = match scan_digit_group(bytes, 0) {
        Some(e) => e,
        None => return false,
    };
    let mut i = end1;
    if i < len && bytes[i] == DOT {
        // Alternative 1: DigitGroup "." DigitGroup Exp? — the fraction needs >= 1 digit,
        // and no other alternative can consume '.', so failure here is final.
        i += 1;
        match scan_digit_group(bytes, i) {
            Some(e2) if e2 > i => i = e2,
            _ => return false,
        }
        match scan_exp(bytes, i) {
            Some(e) => i = e,
            None => return false,
        }
        i = scan_float_suffix(bytes, i);
        return i == len;
    }
    // Alternative 2: DigitGroup Exp; or the validity-sentence suffix form DigitGroup FloatSuffix?
    let mut had_exp = false;
    if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
        match scan_exp(bytes, i) {
            Some(e) if e > i => {
                i = e;
                had_exp = true;
            }
            _ => return false, // exponent present but malformed (needs >= 1 digit)
        }
    }
    let had_suffix = scan_float_suffix(bytes, i) > i;
    i = scan_float_suffix(bytes, i);
    // Validity sentence (§2.2.8): fraction >= 1 digit (alternative 1 returned above),
    // or exponent >= 1 digit, or a float suffix; and the whole input is consumed.
    (had_exp || had_suffix) && i == len
}

/// Scan `DigitGroup = Digit { "_" Digit }` (§2.2.7) from `start`; Some(end) with >= 1 digit, or None.
/// The scan naturally enforces: no leading `_`, no trailing `_`, no adjacent `_`.
const fn is_exp_byte(b: u8) -> bool {
    b == b'e' || b == b'E'
}

fn scan_digit_group(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    if start >= len || !is_dec_digit(bytes[start]) {
        return None;
    }
    let mut i = start + 1;
    while i < len {
        if bytes[i] == UNDERSCORE {
            // '_' continues the group only if followed by a digit (§2.2.7 rules).
            if i + 1 < len && is_dec_digit(bytes[i + 1]) {
                i += 2;
            } else {
                break;
            }
        } else if is_dec_digit(bytes[i]) {
            i += 1;
        } else {
            break;
        }
    }
    Some(i)
}

/// Scan `Exp = ("e"|"E") ["+"|"-"] DigitGroup` (§2.2.8) from `i`.
/// Some(i): no exponent at i. Some(end): exponent consumed. None: exponent present but malformed.
fn scan_exp(bytes: &[u8], i: usize) -> Option<usize> {
    let len = bytes.len();
    if i >= len || !is_exp_byte(bytes[i]) {
        return Some(i);
    }
    let mut j = i + 1;
    if j < len && (bytes[j] == b'+' || bytes[j] == b'-') {
        j += 1;
    }
    match scan_digit_group(bytes, j) {
        Some(e) if e > j => Some(e),
        _ => None,
    }
}

/// Consume `FloatSuffix = "f32" | "f64"` (§2.2.8) at `i` if present; returns the new index.
fn scan_float_suffix(bytes: &[u8], i: usize) -> usize {
    let len = bytes.len();
    if i + 2 < len && bytes[i] == b'f' {
        if bytes[i + 1] == b'3' && bytes[i + 2] == b'2' {
            return i + 3;
        }
        if bytes[i + 1] == b'6' && bytes[i + 2] == b'4' {
            return i + 3;
        }
    }
    let _ = len;
    i
}

// ---------------------------------------------------------------------------
// Char literal body decoding (Volume 02 §2.2.10, §2.2.14, §2.3.6).
// ---------------------------------------------------------------------------

/// Decode the body of a char literal (the bytes between the quotes) to its single scalar value.
///
/// Refinement anchor: Volume 02 §2.2.10 (`CharLiteral = "'" (CharBody | Escape) "'"`, CharBody excludes `'`, `\`, U+000A, U+000D), §2.2.14 (complete escape table), §2.3.6 (exactly one Unicode scalar; surrogates rejected).
/// Input domain: any byte string of length >= 0 (total). The tokenizer supplies the body and guarantees the source is valid UTF-8 (§2.1.5); the function is nevertheless total over all byte inputs.
/// Failure set: `EmptyCharLiteral` (empty body, E-LEX-07), `MultipleChars` (body is not exactly one CharBody scalar: multiple scalars, a raw excluded byte including a lone `'\'`, trailing bytes after an escape, or invalid UTF-8; E-LEX-07), `InvalidEscape` (unknown escape, malformed `\xHH` or out of the char context range 0x00..=0x7F, malformed `\u{...}` production; E-LEX-08), `SurrogateCodePoint` (surrogate scalar; E-LIT-05 at the char-literal level, E-LIT-04 at the `\u` escape level per §2.2.14), `CodePointOutOfRange { value, max }` (`\u` payload > U+10FFFF; E-LIT-04).
/// Certificate: EXH `exh_char_literal_raw_ascii_256` (all 256 single-byte raw bodies), `exh_char_escapes_xhh_256` (all 256 \xHH forms; OB-02-7), `exh_char_named_escapes_and_u_boundaries` (all 9 named escape forms + the 8 scalar boundaries of \u; OB-02-7).
///
/// CEP:WHAT: Char-literal body decoding with the pinned escape table.
/// CEP:WHY: §2.3.6 pins one-scalar semantics with surrogate rejection; the escape table is the complete §2.2.14 set.
/// CEP:STATUS: complete
/// CEP:FAILURE: LiteralError::{EmptyCharLiteral, MultipleChars, InvalidEscape, SurrogateCodePoint, CodePointOutOfRange} — see the enum docs.
/// CEP:ASSUMES: byte contexts (byte char / byte string) are separate token forms (§2.2.10, §2.3.9) and are not decoded here.
/// CEP:COST: O(len) single pass, allocation-free (core::str::from_utf8 borrows); CEP-0.
/// CEP:EVIDENCE: EXH exh_char_literal_raw_ascii_256; EXH exh_char_escapes_xhh_256; EXH exh_char_named_escapes_and_u_boundaries.
pub fn decode_char_literal(body: &[u8]) -> Result<char, LiteralError> {
    if body.is_empty() {
        return Err(LiteralError::EmptyCharLiteral); // E-LEX-07
    }
    if body[0] == BACKSLASH {
        if body.len() == 1 {
            // A lone '\' is neither a CharBody (excluded byte, §2.2.10) nor a complete escape
            // (the §2.2.14 table has no one-byte escape), so the body is not exactly one
            // scalar value: E-LEX-07.
            return Err(LiteralError::MultipleChars);
        }
        let (scalar, consumed) = decode_escape(body)?;
        if consumed != body.len() {
            return Err(LiteralError::MultipleChars); // trailing bytes after the escape (E-LEX-07)
        }
        return scalar_to_char(scalar);
    }
    // Raw CharBody: exactly one Unicode scalar value (§2.2.10).
    match core::str::from_utf8(body) {
        Ok(s) => {
            let mut it = s.chars();
            match (it.next(), it.next()) {
                (Some(c), None) => {
                    // CharBody excludes ' , \ , U+000A , U+000D (§2.2.10): such bytes cannot
                    // appear raw in a token; the body does not derive CharBody (E-LEX-07 class).
                    if c == '\'' || c == '\\' || c == '\u{0A}' || c == '\u{0D}' {
                        Err(LiteralError::MultipleChars)
                    } else {
                        Ok(c)
                    }
                }
                _ => Err(LiteralError::MultipleChars), // not exactly one scalar (E-LEX-07)
            }
        }
        // Invalid UTF-8 cannot be one scalar (E-LEX-07); unreachable from the tokenizer (§2.1.5).
        Err(_) => Err(LiteralError::MultipleChars),
    }
}

/// Decode one escape per the complete pinned table (Volume 02 §2.2.14). `body[0] == '\\'` is assumed.
/// Returns the scalar value and the number of bytes consumed.
fn decode_escape(body: &[u8]) -> Result<(u32, usize), LiteralError> {
    if body.len() < 2 {
        return Err(LiteralError::InvalidEscape); // lone backslash (E-LEX-08)
    }
    match body[1] {
        b'n' => Ok((0x0A, 2)),   // \n -> U+000A
        b't' => Ok((0x09, 2)),   // \t -> U+0009
        b'r' => Ok((0x0D, 2)),   // \r -> U+000D
        b'\\' => Ok((0x5C, 2)),  // \\ -> U+005C
        b'0' => Ok((0x00, 2)),   // \0 -> U+0000
        b'\'' => Ok((0x27, 2)),  // \' -> U+0027
        b'"' => Ok((0x22, 2)),   // \" -> U+0022
        b'x' => {
            // \xHH; char/string context range 0x00..=0x7F; out of range is E-LEX-08 (§2.2.14).
            if body.len() < 4 || !is_hex_digit(body[2]) || !is_hex_digit(body[3]) {
                return Err(LiteralError::InvalidEscape);
            }
            let v = (hex_val(body[2]) << 4) | hex_val(body[3]);
            if v > CHAR_CONTEXT_X_MAX {
                return Err(LiteralError::InvalidEscape); // \xHH out of context range (E-LEX-08)
            }
            Ok((v, 4))
        }
        b'u' => decode_unicode_escape(body),
        _ => Err(LiteralError::InvalidEscape), // escape not in the table (E-LEX-08)
    }
}

/// Decode `\u{H{1,6}}` (§2.2.14): 1..=6 hex digits inside braces; payload must be a scalar value.
fn decode_unicode_escape(body: &[u8]) -> Result<(u32, usize), LiteralError> {
    // Malformed production (missing brace, non-hex digit, missing terminator, > 6 digits): E-LEX-08.
    if body.len() < 4 || body[2] != b'{' {
        return Err(LiteralError::InvalidEscape);
    }
    let mut v: u32 = 0;
    let mut n = 0;
    let mut i = 3;
    while i < body.len() && body[i] != b'}' {
        if !is_hex_digit(body[i]) {
            return Err(LiteralError::InvalidEscape);
        }
        n += 1;
        if n > 6 {
            return Err(LiteralError::InvalidEscape); // H{1,6} production violated (E-LEX-08)
        }
        // checked arithmetic: no panic on huge digit runs (§5.2 kernel law).
        v = match v.checked_mul(16).and_then(|m| m.checked_add(hex_val(body[i]))) {
            Some(next) => next,
            None => return Err(LiteralError::CodePointOutOfRange { value: u32::MAX, max: UNICODE_SCALAR_MAX }),
        };
        i += 1;
    }
    if i >= body.len() {
        return Err(LiteralError::InvalidEscape); // missing '}' (E-LEX-08)
    }
    if n == 0 {
        return Err(LiteralError::InvalidEscape); // \u{} is not a production (E-LEX-08)
    }
    let consumed = i + 1;
    // Payload must be a scalar value: <= U+10FFFF and not a surrogate (§2.2.14, §2.3.6-§2.3.7).
    if v >= SURROGATE_MIN && v <= SURROGATE_MAX {
        return Err(LiteralError::SurrogateCodePoint); // E-LIT-05/E-LIT-04 (see variant docs)
    }
    if v > UNICODE_SCALAR_MAX {
        return Err(LiteralError::CodePointOutOfRange { value: v, max: UNICODE_SCALAR_MAX }); // E-LIT-04
    }
    Ok((v, consumed))
}

/// Convert a validated scalar to `char`; the checks make `None` unreachable except via payloads
/// already classified, so the error arm is the exact pinned classification (E-LIT-05 / E-LIT-04).
fn scalar_to_char(scalar: u32) -> Result<char, LiteralError> {
    match core::char::from_u32(scalar) {
        Some(c) => Ok(c),
        None => {
            if scalar >= SURROGATE_MIN && scalar <= SURROGATE_MAX {
                Err(LiteralError::SurrogateCodePoint)
            } else {
                Err(LiteralError::CodePointOutOfRange { value: scalar, max: UNICODE_SCALAR_MAX })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;

    const U128_MAX: u128 = u128::MAX;

    /// Pinned in-crate xorshift64* PRNG (seed 0x9E3779B97F4A7C15) for PROP tests; deterministic.
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

    // ------------------------- integer folding -------------------------

    const RADICES: [Radix; 4] = [Radix::Bin, Radix::Oct, Radix::Dec, Radix::Hex];

    /// Independent reference: per-radix digit validity (different formulation from Radix::digit_value).
    fn ref_digit(radix: Radix, b: u8) -> Option<u32> {
        let ok = match radix {
            Radix::Bin => b == b'0' || b == b'1',
            Radix::Oct => (b >= b'0' && b <= b'7'),
            Radix::Dec => (b >= b'0' && b <= b'9'),
            Radix::Hex => {
                (b >= b'0' && b <= b'9')
                    || (b >= b'a' && b <= b'f')
                    || (b >= b'A' && b <= b'F')
            }
        };
        if ok {
            Some(match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a') as u32 + 10,
                _ => (b - b'A') as u32 + 10,
            })
        } else {
            None
        }
    }

    /// Independent reference fold: pre-check multiply (v > (MAX - d)/r) instead of checked_mul.
    fn ref_fold(digits: &[u8], radix: Radix) -> Option<u128> {
        let r = radix.value();
        let mut v: u128 = 0;
        for &b in digits {
            let d = ref_digit(radix, b)? as u128;
            if v > (U128_MAX - d) / r {
                return None; // overflow
            }
            v = v * r + d;
        }
        Some(v)
    }

    /// EXH certificate (OB-02-4): for EVERY digit string of length <= 4 over the 22-byte union
    /// alphabet {0-9, a-f, A-F} and every pinned radix, parse_integer_radix equals the reference.
    /// Domain: (22 + 484 + 10648 + 234256) x 4 = 981,640 cases (full stated domain).
    #[test]
    fn exh_radix_fold_len_le4_union_alphabet() {
        const ALPHA: [u8; 22] = [
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
            b'a', b'b', b'c', b'd', b'e', b'f', b'A', b'B', b'C', b'D', b'E', b'F',
        ];
        let mut cases = 0u64;
        // length 1
        for &a in &ALPHA {
            cases += check4(&[a]);
        }
        // length 2
        for &a in &ALPHA {
            for &b in &ALPHA {
                cases += check4(&[a, b]);
            }
        }
        // length 3
        for &a in &ALPHA {
            for &b in &ALPHA {
                for &c in &ALPHA {
                    cases += check4(&[a, b, c]);
                }
            }
        }
        // length 4
        for &a in &ALPHA {
            for &b in &ALPHA {
                for &c in &ALPHA {
                    for &d in &ALPHA {
                        cases += check4(&[a, b, c, d]);
                    }
                }
            }
        }
        assert_eq!(cases, 981_640, "EXH domain size must be the full 981,640 cases");
    }

    /// Compare parse vs reference for one digit string over all 4 radices; returns 4.
    fn check4(digits: &[u8]) -> u64 {
        for radix in RADICES {
            match (parse_integer_radix(digits, radix), ref_fold(digits, radix)) {
                (Ok(v), Some(e)) => assert_eq!(v, e, "fold {digits:?} radix {radix:?}"),
                (Ok(_), None) => panic!("impl missed overflow {digits:?} radix {radix:?}"),
                (Err(LiteralError::InvalidDigit { radix: r, byte }), None) => {
                    assert_eq!(r, radix);
                    let first_bad = digits.iter().copied().find(|&b| ref_digit(radix, b).is_none());
                    assert_eq!(Some(byte), first_bad, "first bad digit {digits:?} radix {radix:?}");
                }
                (Err(other), _) => panic!("unexpected error {other:?} for {digits:?} radix {radix:?}"),
                (Err(_), Some(_)) => panic!("impl rejected valid digits {digits:?} radix {radix:?}"),
            }
        }
        4
    }

    /// EXH: per radix, ALL 256 bytes are classified exactly (in set -> value; out of set -> InvalidDigit).
    /// Domain: 4 radices x 256 bytes = 1024 cases (full domain).
    #[test]
    fn exh_invalid_digit_sets_all_256() {
        for radix in RADICES {
            let mut b = 0usize;
            while b < 256 {
                let byte = b as u8;
                let digits = [byte];
                match (radix.digit_value(byte), parse_integer_radix(&digits, radix)) {
                    (Some(d), Ok(v)) => assert_eq!(v, d as u128),
                    (None, Err(LiteralError::InvalidDigit { radix: r, byte: e })) => {
                        assert_eq!(r, radix);
                        assert_eq!(e, byte);
                    }
                    (other, res) => panic!("radix {radix:?} byte {byte:#04x}: {other:?} vs {res:?}"),
                }
                b += 1;
            }
        }
    }

    /// EXH: overflow, length-limit, and empty-input boundaries at the exact pinned constants.
    #[test]
    fn exh_overflow_and_length_boundaries() {
        // Empty digit group: E-LEX-06.
        assert_eq!(parse_integer_radix(b"", Radix::Dec), Err(LiteralError::EmptyDigits));
        // u128::MAX in decimal, hex, octal, binary folds exactly to the boundary.
        let dec_max = b"340282366920938463463374607431768211455";
        assert_eq!(parse_integer_radix(dec_max, Radix::Dec), Ok(U128_MAX));
        let hex_max = [b'f'; 32];
        assert_eq!(parse_integer_radix(&hex_max, Radix::Hex), Ok(U128_MAX));
        let bin_max = [b'1'; 128];
        assert_eq!(parse_integer_radix(&bin_max, Radix::Bin), Ok(U128_MAX));
        // oct(u128::MAX) = "3" + 42x'7' (43 digits): 4*8^42 - 1 = 2^128 - 1.
        let oct_max_str = b"3777777777777777777777777777777777777777777";
        assert_eq!(oct_max_str.len(), 43);
        assert_eq!(parse_integer_radix(oct_max_str, Radix::Oct), Ok(U128_MAX));
        // One past the boundary in each radix: ValueOverflow with max_value = u128::MAX.
        let dec_over = b"340282366920938463463374607431768211456";
        assert_eq!(
            parse_integer_radix(dec_over, Radix::Dec),
            Err(LiteralError::ValueOverflow { max_value: U128_MAX })
        );
        let hex_over_33 = [b'f'; 33];
        assert_eq!(
            parse_integer_radix(&hex_over_33, Radix::Hex),
            Err(LiteralError::ValueOverflow { max_value: U128_MAX })
        );
        let bin_over_129 = [b'1'; 129];
        assert_eq!(
            parse_integer_radix(&bin_over_129, Radix::Bin),
            Err(LiteralError::ValueOverflow { max_value: U128_MAX })
        );
        // One past MAX in octal: 4*8^42 = 2^128 = "4" + 42x'0' (43 digits).
        let oct_over = b"4000000000000000000000000000000000000000000";
        assert_eq!(oct_over.len(), 43);
        assert_eq!(
            parse_integer_radix(oct_over, Radix::Oct),
            Err(LiteralError::ValueOverflow { max_value: U128_MAX })
        );
        // Length limit: MAX_LIT_DIGITS = 4096 is allowed (then overflows on value), 4097 is E-LEX-12.
        let ok_len = [b'9'; MAX_LIT_DIGITS];
        assert_eq!(
            parse_integer_radix(&ok_len, Radix::Dec),
            Err(LiteralError::ValueOverflow { max_value: U128_MAX })
        );
        let too_long = [b'9'; MAX_LIT_DIGITS + 1];
        assert_eq!(
            parse_integer_radix(&too_long, Radix::Dec),
            Err(LiteralError::TooManyDigits { max: MAX_LIT_DIGITS })
        );
        // Underscores are not digits in the fold input (the digit group arrives separator-stripped).
        assert_eq!(
            parse_integer_radix(b"1_0", Radix::Dec),
            Err(LiteralError::InvalidDigit { radix: Radix::Dec, byte: b'_' })
        );
    }

    /// PROP certificate (paired with EXH): 20,000 random digit strings (length 0..=40, alphabet =
    /// radix digits + noise) equal the independent reference (value, overflow, or first-bad-digit).
    #[test]
    fn prop_radix_fold_vs_reference() {
        const ALPHA: [u8; 22] = [
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
            b'a', b'b', b'c', b'd', b'e', b'f', b'A', b'B', b'C', b'D', b'E', b'F',
        ];
        const NOISE: [u8; 6] = [b'_', b'g', b'z', b'G', b'Z', b'.'];
        let mut rng = Xorshift64Star::new();
        for _ in 0..20_000 {
            let len = rng.below(41) as usize;
            let mut buf = [0u8; 40];
            let mut i = 0;
            while i < len {
                let pick = rng.below(28) as usize;
                buf[i] = if pick < ALPHA.len() { ALPHA[pick] } else { NOISE[pick - ALPHA.len()] };
                i += 1;
            }
            let digits = &buf[..len];
            for radix in RADICES {
                if digits.is_empty() {
                    // Empty digit group is E-LEX-06 (§2.2.7); the reference fold is undefined there.
                    assert_eq!(parse_integer_radix(digits, radix), Err(LiteralError::EmptyDigits));
                    continue;
                }
                match (parse_integer_radix(digits, radix), ref_fold(digits, radix)) {
                    (Ok(v), Some(e)) => assert_eq!(v, e),
                    (Ok(_), None) => panic!("missed overflow {digits:?} {radix:?}"),
                    (Err(LiteralError::InvalidDigit { radix: r, byte }), None) => {
                        assert_eq!(r, radix);
                        let first_bad = digits.iter().copied().find(|&b| ref_digit(radix, b).is_none());
                        assert_eq!(Some(byte), first_bad);
                    }
                    (Err(LiteralError::ValueOverflow { max_value }), None) => assert_eq!(max_value, U128_MAX),
                    (Err(other), _) => panic!("unexpected {other:?} for {digits:?} {radix:?}"),
                    (Err(_), Some(_)) => panic!("rejected valid {digits:?} {radix:?}"),
                }
            }
        }
    }

    // ------------------------- fits_type -------------------------

    /// EXH certificate (OB-02-5): acceptance is exactly the pinned range at the boundary values
    /// per integer type; named consts are cross-checked against the §2.3.3 decimal table.
    #[test]
    fn exh_fits_type_boundaries_12_types() {
        const TYPES: [IntTy; 12] = [
            IntTy::I8, IntTy::I16, IntTy::I32, IntTy::I64, IntTy::I128,
            IntTy::U8, IntTy::U16, IntTy::U32, IntTy::U64, IntTy::U128,
            IntTy::Isize, IntTy::Usize,
        ];
        // §2.3.3 max(T) table, written independently as decimal literals.
        let max64 = [
            127u128, 32767, 2147483647, 9223372036854775807, 170141183460469231731687303715884105727,
            255, 65535, 4294967295, 18446744073709551615, 340282366920938463463374607431768211455,
            9223372036854775807, 18446744073709551615,
        ];
        let min128 = [
            -128i128, -32768, -2147483648, -9223372036854775808, -170141183460469231731687303715884105728,
            0, 0, 0, 0, 0, -9223372036854775808, 0,
        ];
        for k in 0..TYPES.len() {
            let ty = TYPES[k];
            let max = ty.max_value(PtrWidth::P64);
            assert_eq!(max, max64[k], "max(T) table row {k}");
            assert_eq!(ty.min_value(PtrWidth::P64), min128[k], "min(T) table row {k}");
            assert!(fits_type(0, ty), "0 fits {ty:?}");
            assert!(fits_type(1, ty));
            if max > 0 {
                assert!(fits_type(max - 1, ty), "max-1 fits {ty:?}");
            }
            assert!(fits_type(max, ty), "max fits {ty:?}");
            if max < U128_MAX {
                assert!(!fits_type(max + 1, ty), "max+1 rejects {ty:?}");
                assert!(!fits_type(max + 2, ty), "max+2 rejects {ty:?}");
            }
        }
        // Per-target usize/isize bounds (§2.3.3: i686 family = u32/i32).
        assert_eq!(IntTy::Usize.max_value(PtrWidth::P32), 4294967295);
        assert_eq!(IntTy::Isize.max_value(PtrWidth::P32), 2147483647);
        assert_eq!(IntTy::Usize.max_value(PtrWidth::P64), 18446744073709551615);
        assert_eq!(IntTy::Isize.min_value(PtrWidth::P32), -2147483648);
        assert!(fits_type_on(4294967295, IntTy::Usize, PtrWidth::P32));
        assert!(!fits_type_on(4294967296, IntTy::Usize, PtrWidth::P32));
        assert!(fits_type_on(4294967296, IntTy::Usize, PtrWidth::P64));
        assert!(fits_type_on(2147483647, IntTy::Isize, PtrWidth::P32));
        assert!(!fits_type_on(2147483648, IntTy::Isize, PtrWidth::P32));
        assert!(fits_type_on(2147483648, IntTy::Isize, PtrWidth::P64));
        // i128: exactly the two-sided boundary of the u128-literal domain.
        assert!(fits_type(I128_MAX, IntTy::I128));
        assert!(!fits_type(I128_MAX + 1, IntTy::I128));
        // u128: the whole u128 domain fits by definition.
        assert!(fits_type(U128_MAX, IntTy::U128));
    }

    // ------------------------- identifiers -------------------------

    /// Independent reference for the §2.2.1 grammar (whole-slice formulation).
    fn ref_ident(s: &[u8]) -> bool {
        if s.is_empty() {
            return false;
        }
        if s == b"_" {
            return false; // exactly "_" is classed Wildcard, not Identifier (§2.2.1)
        }
        let first = s[0];
        let start_ok = (first >= b'A' && first <= b'Z') || (first >= b'a' && first <= b'z') || first == b'_';
        if !start_ok {
            return false;
        }
        let mut all = true;
        for &c in &s[1..] {
            all &= (c >= b'A' && c <= b'Z')
                || (c >= b'a' && c <= b'z')
                || (c >= b'0' && c <= b'9')
                || c == b'_';
        }
        all
    }

    /// EXH: every string of length <= 3 over {a, Z, _, 0, 9} plus the empty string (156 cases,
    /// full stated domain) equals the independent reference.
    #[test]
    fn exh_identifier_full_domain_len_le3() {
        const ALPHA: [u8; 5] = [b'a', b'Z', b'_', b'0', b'9'];
        assert_eq!(is_valid_identifier(b""), ref_ident(b""));
        let mut cases = 1u64;
        for &a in &ALPHA {
            cases += check_ident(&[a]);
            for &b in &ALPHA {
                cases += check_ident(&[a, b]);
                for &c in &ALPHA {
                    cases += check_ident(&[a, b, c]);
                }
            }
        }
        assert_eq!(cases, 156);
        // Curated exactness points from §2.2.1/§2.2.4.
        assert!(is_valid_identifier(b"as"), "keywords are valid identifier scans (§2.2.21 rule 4)");
        assert!(is_valid_identifier(b"Self"));
        assert!(is_valid_identifier(b"_a"));
        assert!(is_valid_identifier(b"__"));
        assert!(is_valid_identifier(b"a1_"));
        assert!(!is_valid_identifier(b"_"), "exactly '_' is the Wildcard token (§2.2.1)");
        assert!(!is_valid_identifier(b"9a"));
        assert!(!is_valid_identifier(b"a-b"));
        assert!(!is_valid_identifier(b"a b"));
        // Multi-byte UTF-8 is rejected (identifiers are ASCII, §2.2.2).
        assert!(!is_valid_identifier(&[0xC3, 0xA9]));
        assert!(!is_valid_identifier(&[b'a', 0xC3]));
    }

    fn check_ident(s: &[u8]) -> u64 {
        assert_eq!(is_valid_identifier(s), ref_ident(s), "ident {s:?}");
        1
    }

    /// EXH: all 256 single-byte strings (full domain) equal the reference.
    #[test]
    fn exh_identifier_single_bytes_256() {
        let mut i = 0usize;
        while i < 256 {
            let s = [i as u8];
            assert_eq!(is_valid_identifier(&s), ref_ident(&s), "single byte {i:#04x}");
            i += 1;
        }
    }

    /// PROP certificate (paired with EXH): 20,000 random strings over an identifier-ish alphabet
    /// plus arbitrary bytes equal the independent reference.
    #[test]
    fn prop_identifier_vs_reference() {
        const ALPHA: [u8; 16] = [
            b'a', b'Z', b'_', b'0', b'9', b'q', b'M', b'x', b'B', b'7', b'c', b'N', b'y', b'C', b'2', b't',
        ];
        let mut rng = Xorshift64Star::new();
        for _ in 0..20_000 {
            let len = rng.below(9) as usize;
            let mut buf = [0u8; 8];
            let mut i = 0;
            while i < len {
                buf[i] = if rng.below(5) == 0 {
                    rng.below(256) as u8
                } else {
                    ALPHA[rng.below(ALPHA.len() as u64) as usize]
                };
                i += 1;
            }
            let s = &buf[..len];
            assert_eq!(is_valid_identifier(s), ref_ident(s), "ident {s:?}");
        }
    }

    // ------------------------- float literals -------------------------

    /// Independent segment-enumeration reference for the §2.2.8 grammar (structurally different
    /// from the implementation's sequential scanner).
    fn ref_float(s: &[u8]) -> bool {
        let n = s.len();
        if n == 0 {
            return false;
        }
        // Segment the string as [dg1]['.' dg2][exp][suffix] with boundary brute force.
        for b1 in 1..=n {
            if !ref_dg(&s[..b1]) {
                continue;
            }
            // Option A: '.' then dg2 (alternative 1).
            for b3 in b1..=n {
                if b1 < n && s[b1] == b'.' && b3 >= b1 + 1 {
                    let dg2 = &s[b1 + 1..b3];
                    if !ref_dg(dg2) {
                        continue;
                    }
                    for b4 in b3..=n {
                        if !ref_exp_opt(&s[b3..b4]) {
                            continue;
                        }
                        if ref_suffix(&s[b4..]) {
                            return true;
                        }
                    }
                }
                // Option B: no '.' (alternative 2 / suffix form).
                if b3 == b1 {
                    for b4 in b3..=n {
                        if !ref_exp_seg(&s[b3..b4]) {
                            continue;
                        }
                        let has_exp = b4 > b3;
                        // Validity sentence (§2.2.8): an exponent with >= 1 digit, or an exact
                        // f32/f64 suffix; bare DigitGroup alone is an integer token, not a float.
                        let suffix = &s[b4..];
                        let suffix_ok = suffix == b"f32" || suffix == b"f64";
                        if (has_exp || suffix_ok) && (b4 == n || suffix_ok) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Whole-segment DigitGroup validator: non-empty, digits with '_' only between digits.
    fn ref_dg(seg: &[u8]) -> bool {
        if seg.is_empty() || !seg.iter().all(|&c| (c >= b'0' && c <= b'9') || c == b'_') {
            return false;
        }
        if seg[0] == b'_' || seg[seg.len() - 1] == b'_' {
            return false;
        }
        let mut prev_us = false;
        for &c in seg {
            if c == b'_' {
                if prev_us {
                    return false;
                }
                prev_us = true;
            } else {
                prev_us = false;
            }
        }
        true
    }

    /// Segment is empty or a well-formed exponent (used after fraction in alternative 1).
    fn ref_exp_opt(seg: &[u8]) -> bool {
        seg.is_empty() || ref_exp_seg(seg)
    }

    /// Segment is empty or a well-formed exponent; returns false for malformed non-empty.
    fn ref_exp_seg(seg: &[u8]) -> bool {
        if seg.is_empty() {
            return true;
        }
        if seg[0] != b'e' && seg[0] != b'E' {
            return false;
        }
        let rest = if seg.len() > 1 && (seg[1] == b'+' || seg[1] == b'-') {
            &seg[2..]
        } else {
            &seg[1..]
        };
        ref_dg(rest)
    }

    /// Segment is empty or exactly the float suffix.
    fn ref_suffix(seg: &[u8]) -> bool {
        seg.is_empty() || seg == b"f32" || seg == b"f64"
    }

    /// EXH: every string of length <= 4 over the 11-byte alphabet {1, 0, 9, _, ., e, +, -, f, 3, 6}
    /// (16,105 cases, full stated domain) equals the independent segment-enumeration reference.
    #[test]
    fn exh_float_len_le4_alphabet_11() {
        const ALPHA: [u8; 11] = [b'1', b'0', b'9', b'_', b'.', b'e', b'+', b'-', b'f', b'3', b'6'];
        assert_eq!(is_valid_float_literal(b""), ref_float(b""));
        let mut cases = 1u64;
        for &a in &ALPHA {
            cases += check_float(&[a]);
            for &b in &ALPHA {
                cases += check_float(&[a, b]);
                for &c in &ALPHA {
                    cases += check_float(&[a, b, c]);
                    for &d in &ALPHA {
                        cases += check_float(&[a, b, c, d]);
                    }
                }
            }
        }
        assert_eq!(cases, 16_105, "EXH domain size must be the full 16,105 cases");
    }

    fn check_float(s: &[u8]) -> u64 {
        assert_eq!(is_valid_float_literal(s), ref_float(s), "float {s:?}");
        1
    }

    /// EXH: curated boundary cases with hand-derived expectations from the §2.2.8 productions.
    #[test]
    fn exh_float_boundary_cases() {
        // Valid per the productions and the validity sentence.
        for s in [
            &b"1.0"[..], b"0.0", b"1.5", b"1_0.0", b"0.0_1", b"1.0e2", b"1.0E2", b"1.0e-2", b"1.0e+2",
            b"1e5", b"1E5", b"1e+5", b"1e-5", b"1e1_0", b"1f32", b"1f64", b"1.0f32", b"1.0f64",
            b"1e5f32", b"1.0e2f64", b"0e0", b"9.9e9", b"1.0E-3f32",
        ] {
            assert!(is_valid_float_literal(s), "expected valid: {}", core::str::from_utf8(s).unwrap_or("?"));
        }
        // Invalid: no fraction digits / no exponent / no suffix; malformed underscore placement;
        // malformed exponent; trailing garbage; wrong suffix spelling; pre-suffix underscore.
        for s in [
            &b"1"[..], b"1_", b"_1.0", b"1_.0", b"1__0", b"1_0_", b"1.", b"1.e3", b"1.f32", b".5",
            b"1.0e", b"1.0e+", b"1.0e_", b"1e", b"1e_", b"1e+", b"1.0x", b"1.0f32x", b"1.0f3",
            b"1.0f321", b"1.0F32", b"1.5.5", b"1e5e5", b"1_f32", b"1.0f16", b"1.0..", b"",
        ] {
            assert!(!is_valid_float_literal(s), "expected invalid: {}", core::str::from_utf8(s).unwrap_or("?"));
        }
    }

    /// PROP certificate (paired with EXH): 20,000 random strings over the float alphabet plus
    /// noise equal the independent segment-enumeration reference.
    #[test]
    fn prop_float_vs_reference() {
        const ALPHA: [u8; 16] = [
            b'0', b'1', b'9', b'_', b'.', b'e', b'E', b'+', b'-', b'f', b'3', b'6', b'4', b'2', b'8', b'x',
        ];
        let mut rng = Xorshift64Star::new();
        for _ in 0..20_000 {
            let len = rng.below(9) as usize;
            let mut buf = [0u8; 8];
            let mut i = 0;
            while i < len {
                buf[i] = ALPHA[rng.below(ALPHA.len() as u64) as usize];
                i += 1;
            }
            let s = &buf[..len];
            assert_eq!(is_valid_float_literal(s), ref_float(s), "float {s:?}");
        }
    }

    // ------------------------- char literals -------------------------

    /// EXH (OB-02-7): all 256 `\xHH` forms in char context: <= 0x7F decodes, > 0x7F is E-LEX-08.
    #[test]
    fn exh_char_escapes_xhh_256() {
        const HEXC: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f'];
        let mut v = 0usize;
        while v < 256 {
            let hi = HEXC[(v >> 4) & 0xF];
            let lo = HEXC[v & 0xF];
            let body = [b'\\', b'x', hi, lo];
            let expected = if v <= CHAR_CONTEXT_X_MAX as usize {
                Ok(v as u8 as char)
            } else {
                Err(LiteralError::InvalidEscape)
            };
            assert_eq!(decode_char_literal(&body), expected, "\\xHH {v:#04x}");
            v += 1;
        }
    }

    /// EXH (OB-02-7): all 9 named escape forms, and the 8 scalar boundaries of `\u{...}`.
    #[test]
    fn exh_char_named_escapes_and_u_boundaries() {
        // The 7 two-character named forms of the §2.2.14 table.
        assert_eq!(decode_char_literal(b"\\n"), Ok('\u{0A}'));
        assert_eq!(decode_char_literal(b"\\t"), Ok('\u{09}'));
        assert_eq!(decode_char_literal(b"\\r"), Ok('\u{0D}'));
        assert_eq!(decode_char_literal(b"\\\\"), Ok('\u{5C}')); // body is two bytes: the \\ escape
        assert_eq!(decode_char_literal(b"\\0"), Ok('\u{00}'));
        assert_eq!(decode_char_literal(b"\\'"), Ok('\u{27}'));
        assert_eq!(decode_char_literal(b"\\\""), Ok('\u{22}'));
        // The 8 scalar boundaries of \u{...}: in-set {0, D7FF, E000, 10FFFF}, out-set {D800, DFFF, 110000, FFFFFF}.
        assert_eq!(decode_char_literal(b"\\u{0}"), Ok('\u{0}'));
        assert_eq!(decode_char_literal(b"\\u{D7FF}"), Ok('\u{D7FF}'));
        assert_eq!(decode_char_literal(b"\\u{E000}"), Ok('\u{E000}'));
        assert_eq!(decode_char_literal(b"\\u{10FFFF}"), Ok('\u{10FFFF}'));
        assert_eq!(decode_char_literal(b"\\u{D800}"), Err(LiteralError::SurrogateCodePoint));
        assert_eq!(decode_char_literal(b"\\u{DFFF}"), Err(LiteralError::SurrogateCodePoint));
        assert_eq!(
            decode_char_literal(b"\\u{110000}"),
            Err(LiteralError::CodePointOutOfRange { value: 0x110000, max: UNICODE_SCALAR_MAX })
        );
        assert_eq!(
            decode_char_literal(b"\\u{FFFFFF}"),
            Err(LiteralError::CodePointOutOfRange { value: 0xFFFFFF, max: UNICODE_SCALAR_MAX })
        );
        // Malformed \u productions and unknown escapes: E-LEX-08.
        assert_eq!(decode_char_literal(b"\\u{}"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\u{1234567}"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\u{G}"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\u"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\q"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\"), Err(LiteralError::MultipleChars)); // lone backslash: E-LEX-07
        assert_eq!(decode_char_literal(b"\\x4"), Err(LiteralError::InvalidEscape));
        assert_eq!(decode_char_literal(b"\\xGG"), Err(LiteralError::InvalidEscape));
        // Trailing bytes after a single escape: not exactly one scalar (E-LEX-07).
        assert_eq!(decode_char_literal(b"\\x41X"), Err(LiteralError::MultipleChars));
        assert_eq!(decode_char_literal(b"\\nn"), Err(LiteralError::MultipleChars));
        // \x80 is out of the char/string context range (E-LEX-08, §2.2.14).
        assert_eq!(decode_char_literal(b"\\x80"), Err(LiteralError::InvalidEscape));
    }

    /// EXH: all 256 single-byte raw bodies (full domain): ASCII (minus the 4 excluded CharBody
    /// bytes) decodes to itself; excluded bytes and invalid UTF-8 lead bytes are E-LEX-07 class.
    #[test]
    fn exh_char_literal_raw_ascii_256() {
        let mut i = 0usize;
        while i < 256 {
            let body = [i as u8];
            let expected = if i < 0x80 {
                match i as u8 {
                    0x27 | 0x5C | 0x0A | 0x0D => Err(LiteralError::MultipleChars),
                    _ => Ok(i as u8 as char),
                }
            } else {
                Err(LiteralError::MultipleChars) // invalid UTF-8 single byte
            };
            assert_eq!(decode_char_literal(&body), expected, "raw body {i:#04x}");
            i += 1;
        }
        // Exactly-one-scalar checks: two ASCII scalars and a valid 2-byte UTF-8 scalar.
        assert_eq!(decode_char_literal(b"ab"), Err(LiteralError::MultipleChars));
        assert_eq!(decode_char_literal(&[0xC3, 0xA9]), Ok('é'));
        assert_eq!(decode_char_literal(b""), Err(LiteralError::EmptyCharLiteral));
    }
}
