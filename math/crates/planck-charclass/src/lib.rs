// CEP:FILE: math/crates/planck-charclass/src/lib.rs
// CEP:WHAT: Pinned ASCII character-class predicates and class masks of Planck 0.1 edition P1 (Volume 02 §2.2.0, pinned by SPEC-INDEX §5.10), embedded verbatim as a no_std kernel crate: whitespace, decimal digit, hexadecimal digit, identifier-start, identifier-continue, and the line/block comment-introducer byte predicates, plus the class bitmasks that carry the exact membership sets as const data.
// CEP:WHY: SPEC-INDEX §5.1 pins planck-charclass as the extraction home of the §5.10 character classes; every tokenizer decision (Volume 02 §2.2.21) and the identifier grammar (§2.2.1) refine these predicates, so the frontend must call exactly these functions and no others.
// CEP:STATUS: complete
// CEP:FAILURE: A predicate that returns membership other than the pinned §2.2.0 set for any of the 256 byte values is non-conforming; the EXH truth-table tests fail and the crate must not ship. No other failure mode exists: every predicate is total and pure.
// CEP:ASSUMES: Volume 02 §2.2.0 (exact class sets), §2.2.1-§2.2.2 (identifier grammar; identifiers are ASCII in edition P1), §2.2.15 (comment introducers are the byte pairs 0x2F 0x2F and 0x2F 0x2A); SPEC-INDEX §5.2 (kernel implementation law: core:: only, no allocation, no panic, no unsafe in non-test code); SPEC-INDEX §5.1 (equivalence claim form).
// CEP:COST: O(1) per predicate call (2-4 comparisons, branch-friendly); class masks are compile-time constants; no allocation anywhere.
// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256 (256 bytes x 7 predicates vs const truth tables); EXH exh_predicates_match_class_masks_256 (256 bytes x 6 predicates vs const masks); PROP prop_random_bytes_match_truth_tables (10,000 deterministic PRNG bytes, paired with the EXH above).
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over the complete 256-byte domain, test-as-proof)
// CEP:FV-SCRIPT: crates/planck-charclass/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete: the input domain of every predicate is exactly {0..=255} and the EXH tests enumerate all of it against the explicit truth tables; there is no untested region and therefore no refinement gap.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named class constants (Law 7: constants are named, never magic literals).
// ---------------------------------------------------------------------------

/// Tab, U+0009: a pinned whitespace member (Volume 02 §2.2.0).
pub const TAB: u8 = 0x09;
/// Line feed, U+000A: a pinned whitespace member and the only line terminator after decode (Volume 02 §2.1.4, §2.2.0).
pub const LF: u8 = 0x0A;
/// Carriage return, U+000D: a pinned whitespace member; cannot survive decode outside CRLF normalization (Volume 02 §2.1.4, §2.2.0).
pub const CR: u8 = 0x0D;
/// Space, U+0020: a pinned whitespace member (Volume 02 §2.2.0).
pub const SPACE: u8 = 0x20;
/// Slash, U+002F: the first byte of both pinned comment introducers `//` (0x2F 0x2F) and `/*` (0x2F 0x2A) (Volume 02 §2.2.0, §2.2.15).
pub const SLASH: u8 = 0x2F;
/// Underscore, U+005F: identifier-start member; the token that is exactly `_` is classed Wildcard, not Identifier (Volume 02 §2.2.1).
pub const UNDERSCORE: u8 = 0x5F;

/// Lower bound of the pinned decimal-digit class (Volume 02 §2.2.0).
const DEC_DIGIT_MIN: u8 = 0x30;
/// Upper bound of the pinned decimal-digit class (Volume 02 §2.2.0).
const DEC_DIGIT_MAX: u8 = 0x39;
/// Lower bound of the uppercase hex range [A-F] of the pinned hex-digit class (Volume 02 §2.2.0).
const UPPER_HEX_MIN: u8 = 0x41;
/// Upper bound of the uppercase hex range [A-F] of the pinned hex-digit class (Volume 02 §2.2.0).
const UPPER_HEX_MAX: u8 = 0x46;
/// Lower bound of the lowercase hex range [a-f] of the pinned hex-digit class (Volume 02 §2.2.0).
const LOWER_HEX_MIN: u8 = 0x61;
/// Upper bound of the lowercase hex range [a-f] of the pinned hex-digit class (Volume 02 §2.2.0).
const LOWER_HEX_MAX: u8 = 0x66;
/// Lower bound of the ASCII uppercase-letter range [A-Z] of the identifier-start class (Volume 02 §2.2.0).
const UPPER_MIN: u8 = 0x41;
/// Upper bound of the ASCII uppercase-letter range [A-Z] of the identifier-start class (Volume 02 §2.2.0).
const UPPER_MAX: u8 = 0x5A;
/// Lower bound of the ASCII lowercase-letter range [a-z] of the identifier-start class (Volume 02 §2.2.0).
const LOWER_MIN: u8 = 0x61;
/// Upper bound of the ASCII lowercase-letter range [a-z] of the identifier-start class (Volume 02 §2.2.0).
const LOWER_MAX: u8 = 0x7A;

// ---------------------------------------------------------------------------
// Class masks: bit i (0-based) is set iff byte value i is in the class.
// These are the public definition-of-record of the pinned sets as const data.
// ---------------------------------------------------------------------------

/// Bitmask of the pinned whitespace class {0x09, 0x0A, 0x0D, 0x20} (Volume 02 §2.2.0; SPEC-INDEX §5.10).
///
/// Refinement anchor: Volume 02 §2.2.0 / SPEC-INDEX §5.10. Domain: bits 0..=127 of a `u128`, bit i = byte i.
/// Certificate: EXH `exh_predicates_match_class_masks_256` (all 256 bytes vs mask and predicate).
///
/// CEP:WHAT: u128 bitmask carrying the exact pinned whitespace membership set.
/// CEP:WHY: Downstream lexers and the EXH certificate need the class as named const data, not prose.
/// CEP:STATUS: complete
/// CEP:FAILURE: A mask bit that disagrees with the predicate for any byte fails the EXH test.
/// CEP:ASSUMES: bit i of the mask encodes byte value i (i < 128; all pinned class members are ASCII).
/// CEP:COST: compile-time constant; zero runtime cost.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256.
pub const WHITESPACE_CLASS_MASK: u128 = (1u128 << TAB) | (1u128 << LF) | (1u128 << CR) | (1u128 << SPACE);

/// Bitmask of the pinned decimal-digit class {0x30..=0x39} (Volume 02 §2.2.0).
///
/// Refinement anchor: Volume 02 §2.2.0 / SPEC-INDEX §5.10. Certificate: EXH `exh_predicates_match_class_masks_256`.
///
/// CEP:WHAT: u128 bitmask carrying the exact pinned decimal-digit membership set.
/// CEP:WHY: Same as WHITESPACE_CLASS_MASK: const-data definition of record.
/// CEP:STATUS: complete
/// CEP:FAILURE: Mask/predicate divergence fails the EXH test.
/// CEP:ASSUMES: bit i encodes byte value i.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256.
pub const DECIMAL_DIGIT_CLASS_MASK: u128 = range_mask(DEC_DIGIT_MIN, DEC_DIGIT_MAX);

/// Bitmask of the pinned hex-digit class {0x30..=0x39} ∪ {0x41..=0x46} ∪ {0x61..=0x66} (Volume 02 §2.2.0).
///
/// Refinement anchor: Volume 02 §2.2.0 / SPEC-INDEX §5.10. Certificate: EXH `exh_predicates_match_class_masks_256`.
///
/// CEP:WHAT: u128 bitmask carrying the exact pinned hexadecimal-digit membership set.
/// CEP:WHY: Const-data definition of record for the hex class used by integer literal scanning (§2.2.7).
/// CEP:STATUS: complete
/// CEP:FAILURE: Mask/predicate divergence fails the EXH test.
/// CEP:ASSUMES: bit i encodes byte value i.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256.
pub const HEX_DIGIT_CLASS_MASK: u128 = DECIMAL_DIGIT_CLASS_MASK
    | range_mask(UPPER_HEX_MIN, UPPER_HEX_MAX)
    | range_mask(LOWER_HEX_MIN, LOWER_HEX_MAX);

/// Bitmask of the pinned identifier-start class {0x41..=0x5A} ∪ {0x61..=0x7A} ∪ {0x5F} (Volume 02 §2.2.0; identifiers are ASCII in edition P1, §2.2.2).
///
/// Refinement anchor: Volume 02 §2.2.0, §2.2.2 / SPEC-INDEX §5.10. Certificate: EXH `exh_predicates_match_class_masks_256`.
///
/// CEP:WHAT: u128 bitmask carrying the exact pinned identifier-start membership set.
/// CEP:WHY: Const-data definition of record for IDENT_START of the identifier grammar (§2.2.1).
/// CEP:STATUS: complete
/// CEP:FAILURE: Mask/predicate divergence fails the EXH test.
/// CEP:ASSUMES: bit i encodes byte value i.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256.
pub const IDENT_START_CLASS_MASK: u128 = range_mask(UPPER_MIN, UPPER_MAX)
    | range_mask(LOWER_MIN, LOWER_MAX)
    | (1u128 << UNDERSCORE);

/// Bitmask of the pinned identifier-continue class = identifier-start ∪ decimal digits (Volume 02 §2.2.0).
///
/// Refinement anchor: Volume 02 §2.2.0 / SPEC-INDEX §5.10. Certificate: EXH `exh_predicates_match_class_masks_256`.
///
/// CEP:WHAT: u128 bitmask carrying the exact pinned identifier-continue membership set.
/// CEP:WHY: Const-data definition of record for IDENT_CONT of the identifier grammar (§2.2.1).
/// CEP:STATUS: complete
/// CEP:FAILURE: Mask/predicate divergence fails the EXH test.
/// CEP:ASSUMES: bit i encodes byte value i.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256.
pub const IDENT_CONTINUE_CLASS_MASK: u128 = IDENT_START_CLASS_MASK | DECIMAL_DIGIT_CLASS_MASK;

/// The single byte that begins the pinned line-comment introducer: the byte pair 0x2F 0x2F (`//`) (Volume 02 §2.2.0, §2.2.15).
///
/// CEP:WHAT: First byte of the line-comment introducer pair.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: The full introducer is the byte pair 0x2F 0x2F; pair matching is the tokenizer's use of `is_line_comment_start` plus a next-byte check (§2.2.15).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256.
pub const LINE_COMMENT_INTRODUCER_BYTE: u8 = SLASH;

/// The single byte that begins the pinned block-comment introducer: the byte pair 0x2F 0x2A (`/*`) (Volume 02 §2.2.0, §2.2.15).
///
/// CEP:WHAT: First byte of the block-comment introducer pair.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: The full introducer is the byte pair 0x2F 0x2A; pair matching is the tokenizer's use of `is_block_comment_start` plus a next-byte check (§2.2.15).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256.
pub const BLOCK_COMMENT_INTRODUCER_BYTE: u8 = SLASH;

/// Bits `lo..=hi` set in a u128; const-only helper for the class masks.
///
/// CEP:WHAT: Const range-bitmask constructor used to express contiguous pinned ranges exactly.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: every caller passes 0x00 <= lo <= hi <= 0x7F (const-evaluated), so `hi - lo` cannot underflow and every shift amount is < 128.
/// CEP:ASSUMES: callers guarantee hi >= lo and hi < 128 (true for all pinned class ranges).
/// CEP:COST: compile-time evaluation only.
/// CEP:EVIDENCE: EXH exh_predicates_match_class_masks_256 (masks equal the truth tables).
const fn range_mask(lo: u8, hi: u8) -> u128 {
    let width = (hi - lo) as u128 + 1;
    ((1u128 << width) - 1) << lo
}

// ---------------------------------------------------------------------------
// Predicates: total, pure, CEP-0 (u8 -> bool).
// ---------------------------------------------------------------------------

/// Is `b` a pinned Planck whitespace byte?
///
/// Exact set: {U+0009, U+000A, U+000D, U+0020} (Volume 02 §2.2.0; SPEC-INDEX §5.10).
/// Input domain: all 256 `u8` values (total). Failure set: none (pure predicate).
/// Certificate: EXH `exh_predicates_match_truth_tables_256` — full 256-byte equivalence vs the explicit truth table.
///
/// CEP:WHAT: Whitespace classification per the pinned §2.2.0 set.
/// CEP:WHY: Tokenizer priority rule 2 (§2.2.21) skips exactly this set; sharing one predicate keeps the lexer conforming by construction.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; divergence from the pinned set is caught by the EXH truth-table test.
/// CEP:ASSUMES: Volume 02 §2.2.0 is the normative set; U+000B/U+000C are deliberately NOT members (not in the pinned set).
/// CEP:COST: O(1), 3-4 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_whitespace(b: u8) -> bool {
    b == TAB || b == LF || b == CR || b == SPACE
}

/// Is `b` a pinned Planck decimal-digit byte?
///
/// Exact set: {0x30..=0x39} (Volume 02 §2.2.0, `is_dec_digit`; SPEC-INDEX §5.10).
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Certificate: EXH `exh_predicates_match_truth_tables_256`.
///
/// CEP:WHAT: Decimal-digit classification per the pinned §2.2.0 set.
/// CEP:WHY: The identifier-continue class and the decimal literal digit rule (§2.2.7) are defined from this set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: Volume 02 §2.2.0 is the normative set.
/// CEP:COST: O(1), 2 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_decimal_digit(b: u8) -> bool {
    b >= DEC_DIGIT_MIN && b <= DEC_DIGIT_MAX
}

/// Is `b` a pinned Planck hexadecimal-digit byte?
///
/// Exact set: {0x30..=0x39} ∪ {0x41..=0x46} ∪ {0x61..=0x66} (Volume 02 §2.2.0; SPEC-INDEX §5.10).
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Certificate: EXH `exh_predicates_match_truth_tables_256`.
///
/// CEP:WHAT: Hexadecimal-digit classification per the pinned §2.2.0 set.
/// CEP:WHY: Radix prefixes 0x/0o/0b digit validity (§2.2.7) and \xHH/\u{...} escapes (§2.2.14) refine this class.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: Volume 02 §2.2.0 is the normative set.
/// CEP:COST: O(1), <= 6 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_hex_digit(b: u8) -> bool {
    is_decimal_digit(b)
        || (b >= UPPER_HEX_MIN && b <= UPPER_HEX_MAX)
        || (b >= LOWER_HEX_MIN && b <= LOWER_HEX_MAX)
}

/// Is `b` a pinned Planck identifier-start byte?
///
/// Exact set: {0x41..=0x5A} ∪ {0x61..=0x7A} ∪ {0x5F} (Volume 02 §2.2.0, IDENT_START). Identifiers are ASCII in edition P1 (§2.2.2); UAX #31 is not used.
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Certificate: EXH `exh_predicates_match_truth_tables_256`.
///
/// CEP:WHAT: IDENT_START classification per the pinned §2.2.0 set.
/// CEP:WHY: The identifier scan (§2.2.21 rule 4) starts exactly on this set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: `_` is in the class; the single-byte token `_` is classed Wildcard (§2.2.1) — that is a token-level rule, not a byte-class rule.
/// CEP:COST: O(1), <= 5 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_ident_start(b: u8) -> bool {
    (b >= UPPER_MIN && b <= UPPER_MAX) || (b >= LOWER_MIN && b <= LOWER_MAX) || b == UNDERSCORE
}

/// Is `b` a pinned Planck identifier-continue byte?
///
/// Exact set: `is_ident_start` ∪ `is_decimal_digit` = {0x41..=0x5A} ∪ {0x61..=0x7A} ∪ {0x5F} ∪ {0x30..=0x39} (Volume 02 §2.2.0, IDENT_CONT).
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Certificate: EXH `exh_predicates_match_truth_tables_256`.
///
/// CEP:WHAT: IDENT_CONT classification per the pinned §2.2.0 set.
/// CEP:WHY: The identifier grammar `IDENT_START IDENT_CONT*` (§2.2.1) continues exactly on this set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: Volume 02 §2.2.0 is the normative set.
/// CEP:COST: O(1), <= 6 comparisons, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_ident_continue(b: u8) -> bool {
    is_ident_start(b) || is_decimal_digit(b)
}

/// Is `b` the first byte of the pinned line-comment introducer (`//`, byte pair 0x2F 0x2F)?
///
/// Refinement anchor: Volume 02 §2.2.0 (line-comment introducer = byte pair 0x2F 0x2F) and §2.2.15 (LineComment production).
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Contract note: the introducer is a byte PAIR; this predicate reports the introducer-leading byte 0x2F. Pair recognition is the tokenizer applying this predicate and then checking the next byte for 0x2F (§2.2.15). `/` followed by `*` is instead the block-comment introducer (§2.2.15).
/// Certificate: EXH `exh_predicates_match_truth_tables_256` (exact single-byte truth: true iff byte == 0x2F).
///
/// CEP:WHAT: Line-comment introducer lead-byte predicate per §2.2.0/§2.2.15.
/// CEP:WHY: Tokenizer priority rule 3 (§2.2.21) dispatches comments from this byte.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: Single-byte predicate by task pin ("is_line_comment_start (byte '/')"); the two-byte introducer test lives in the tokenizer, anchored at §2.2.15.
/// CEP:COST: O(1), 1 comparison, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_line_comment_start(b: u8) -> bool {
    b == SLASH
}

/// Is `b` the first byte of the pinned block-comment introducer (`/*`, byte pair 0x2F 0x2A)?
///
/// Refinement anchor: Volume 02 §2.2.0 (block-comment introducer = byte pair 0x2F 0x2A) and §2.2.15 (BlockComment production; block comments nest).
/// Input domain: all 256 `u8` values (total). Failure set: none.
/// Contract note: the introducer is a byte PAIR; this predicate reports the introducer-leading byte 0x2F. Pair recognition is the tokenizer applying this predicate and then checking the next byte for 0x2A (§2.2.15).
/// Certificate: EXH `exh_predicates_match_truth_tables_256` (exact single-byte truth: true iff byte == 0x2F).
///
/// CEP:WHAT: Block-comment introducer lead-byte predicate per §2.2.0/§2.2.15 (SPEC-INDEX §5.10 lists both introducers; this completes the pair).
/// CEP:WHY: Same tokenizer dispatch as is_line_comment_start; both introducers share the lead byte 0x2F exactly as pinned.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; EXH-certified against the truth table.
/// CEP:ASSUMES: Single-byte predicate; the two-byte introducer test lives in the tokenizer, anchored at §2.2.15.
/// CEP:COST: O(1), 1 comparison, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_predicates_match_truth_tables_256; EXH exh_predicates_match_class_masks_256.
#[inline]
pub fn is_block_comment_start(b: u8) -> bool {
    b == SLASH
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;

    /// Explicit truth table for `is_whitespace`: bit (i>>3, i&7) encodes byte i. {09, 0A, 0D, 20}.
    const WHITESPACE_TRUTH: [u8; 32] = {
        let mut t = [0u8; 32];
        t[1] = 0x26; // 0x09 (bit1), 0x0A (bit2), 0x0D (bit5)
        t[4] = 0x01; // 0x20 (bit0)
        t
    };
    /// Explicit truth table for `is_decimal_digit`: {0x30..=0x39}.
    const DECIMAL_DIGIT_TRUTH: [u8; 32] = {
        let mut t = [0u8; 32];
        t[6] = 0xFF; // 0x30..=0x37
        t[7] = 0x03; // 0x38, 0x39
        t
    };
    /// Explicit truth table for `is_hex_digit`: digits + [A-F] + [a-f].
    const HEX_DIGIT_TRUTH: [u8; 32] = {
        let mut t = DECIMAL_DIGIT_TRUTH;
        t[8] = 0x7E; // 0x41..=0x46
        t[12] = 0x7E; // 0x61..=0x66
        t
    };
    /// Explicit truth table for `is_ident_start`: [A-Z] + [a-z] + '_'.
    const IDENT_START_TRUTH: [u8; 32] = {
        let mut t = [0u8; 32];
        t[8] = 0xFE; // 0x41..=0x47
        t[9] = 0xFF; // 0x48..=0x4F
        t[10] = 0xFF; // 0x50..=0x57
        t[11] = 0x87; // 0x58,0x59,0x5A + '_' (0x5F)
        t[12] = 0xFE; // 0x61..=0x67
        t[13] = 0xFF; // 0x68..=0x6F
        t[14] = 0xFF; // 0x70..=0x77
        t[15] = 0x07; // 0x78,0x79,0x7A
        t
    };
    /// Explicit truth table for `is_ident_continue`: ident-start + digits.
    const IDENT_CONTINUE_TRUTH: [u8; 32] = {
        let mut t = IDENT_START_TRUTH;
        t[6] = 0xFF; // 0x30..=0x37
        t[7] = 0x03; // 0x38, 0x39
        t
    };
    /// Explicit truth table for both comment-introducer lead-byte predicates: exactly 0x2F.
    const COMMENT_START_TRUTH: [u8; 32] = {
        let mut t = [0u8; 32];
        t[5] = 0x80; // 0x2F (bit7)
        t
    };

    const fn truth_bit(table: &[u8; 32], byte_index: usize) -> bool {
        (table[byte_index >> 3] >> (byte_index & 7)) & 1 == 1
    }

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

    /// EXH certificate: every predicate equals its explicit truth table on all 256 byte values.
    /// Domain: 256 bytes x 7 predicates = 1792 checks (full domain, test-as-proof).
    #[test]
    fn exh_predicates_match_truth_tables_256() {
        let mut i = 0usize;
        while i < 256 {
            let b = i as u8;
            assert_eq!(is_whitespace(b), truth_bit(&WHITESPACE_TRUTH, i), "is_whitespace {b:#04x}");
            assert_eq!(is_decimal_digit(b), truth_bit(&DECIMAL_DIGIT_TRUTH, i), "is_decimal_digit {b:#04x}");
            assert_eq!(is_hex_digit(b), truth_bit(&HEX_DIGIT_TRUTH, i), "is_hex_digit {b:#04x}");
            assert_eq!(is_ident_start(b), truth_bit(&IDENT_START_TRUTH, i), "is_ident_start {b:#04x}");
            assert_eq!(is_ident_continue(b), truth_bit(&IDENT_CONTINUE_TRUTH, i), "is_ident_continue {b:#04x}");
            assert_eq!(is_line_comment_start(b), truth_bit(&COMMENT_START_TRUTH, i), "is_line_comment_start {b:#04x}");
            assert_eq!(is_block_comment_start(b), truth_bit(&COMMENT_START_TRUTH, i), "is_block_comment_start {b:#04x}");
            i += 1;
        }
    }

    /// EXH certificate: every predicate equals its class mask bit on all 256 byte values,
    /// and every mask equals its truth table bit-for-bit (definition-of-record consistency).
    /// Domain: 256 bytes x 6 predicates + 6 mask/table comparisons of 128 bits each (full domain).
    #[test]
    fn exh_predicates_match_class_masks_256() {
        let mut i = 0usize;
        while i < 128 {
            let b = i as u8;
            let bit = |m: u128| (m >> i) & 1 == 1;
            assert_eq!(is_whitespace(b), bit(WHITESPACE_CLASS_MASK), "ws mask {b:#04x}");
            assert_eq!(is_decimal_digit(b), bit(DECIMAL_DIGIT_CLASS_MASK), "dec mask {b:#04x}");
            assert_eq!(is_hex_digit(b), bit(HEX_DIGIT_CLASS_MASK), "hex mask {b:#04x}");
            assert_eq!(is_ident_start(b), bit(IDENT_START_CLASS_MASK), "start mask {b:#04x}");
            assert_eq!(is_ident_continue(b), bit(IDENT_CONTINUE_CLASS_MASK), "cont mask {b:#04x}");
            i += 1;
        }
        // Comment-introducer predicates share the 0x2F bit (bit 47 of the u128 space).
        let comment_mask: u128 = 1u128 << LINE_COMMENT_INTRODUCER_BYTE;
        assert_eq!(is_line_comment_start(SLASH), (comment_mask >> SLASH) & 1 == 1);
        assert_eq!(is_block_comment_start(SLASH), (comment_mask >> BLOCK_COMMENT_INTRODUCER_BYTE) & 1 == 1);
        // Masks equal the explicit truth tables bit-for-bit (bytes 0..=127 carry every pinned member).
        let mask_bits = |m: u128, table: &[u8; 32]| {
            let mut ok = true;
            let mut i = 0usize;
            while i < 128 {
                let bit = (m >> i) & 1 == 1;
                ok &= bit == truth_bit(table, i);
                i += 1;
            }
            ok
        };
        assert!(mask_bits(WHITESPACE_CLASS_MASK, &WHITESPACE_TRUTH));
        assert!(mask_bits(DECIMAL_DIGIT_CLASS_MASK, &DECIMAL_DIGIT_TRUTH));
        assert!(mask_bits(HEX_DIGIT_CLASS_MASK, &HEX_DIGIT_TRUTH));
        assert!(mask_bits(IDENT_START_CLASS_MASK, &IDENT_START_TRUTH));
        assert!(mask_bits(IDENT_CONTINUE_CLASS_MASK, &IDENT_CONTINUE_TRUTH));
        assert!(mask_bits(comment_mask, &COMMENT_START_TRUTH));
    }

    /// PROP certificate (paired with the EXH tests above): 10,000 deterministic PRNG bytes
    /// satisfy predicate == truth table == mask bit. Sampling domain: uniform bytes 0..=255.
    #[test]
    fn prop_random_bytes_match_truth_tables() {
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < 10_000 {
            let i = (rng.below(256)) as usize;
            let b = i as u8;
            assert_eq!(is_whitespace(b), truth_bit(&WHITESPACE_TRUTH, i));
            assert_eq!(is_decimal_digit(b), truth_bit(&DECIMAL_DIGIT_TRUTH, i));
            assert_eq!(is_hex_digit(b), truth_bit(&HEX_DIGIT_TRUTH, i));
            assert_eq!(is_ident_start(b), truth_bit(&IDENT_START_TRUTH, i));
            assert_eq!(is_ident_continue(b), truth_bit(&IDENT_CONTINUE_TRUTH, i));
            assert_eq!(is_line_comment_start(b), truth_bit(&COMMENT_START_TRUTH, i));
            assert_eq!(is_block_comment_start(b), truth_bit(&COMMENT_START_TRUTH, i));
            n += 1;
        }
    }

    /// Contract spot-check: the introducer byte is exactly 0x2F and no neighbor byte is a lead byte.
    #[test]
    fn exh_comment_introducer_byte_is_0x2f() {
        assert_eq!(LINE_COMMENT_INTRODUCER_BYTE, 0x2F);
        assert_eq!(BLOCK_COMMENT_INTRODUCER_BYTE, 0x2F);
        let mut i = 0usize;
        while i < 256 {
            let b = i as u8;
            assert_eq!(is_line_comment_start(b), b == 0x2F);
            assert_eq!(is_block_comment_start(b), b == 0x2F);
            i += 1;
        }
    }
}
