// CEP:FILE: math/crates/planck-ownership-math/src/lib.rs
// CEP:WHAT: Pinned ownership/loan state machine of Planck 0.1 edition P1 (SPEC-INDEX §5.4, embedded verbatim as Volume 06 §6.1.1), extracted as a no_std kernel crate: the six place states (Uninitialized, Owned, MovedFrom, Shared { loans: u16 }, MutableBorrowed, Frozen { loans: u16 }), the seven place actions (Read, Write, MoveOut, BorrowShared, BorrowMut, Drop, EndBorrow), the total transition function trans : State × Action → State ⊔ {Rejection} over the exact pinned 6×7 table with Rejection as a VALUE (not an error), the derived predicates permits / conflict_at / conflict / incompatible of Volume 06 §6.1.3, loan-count saturation at exactly 65535 (P06-E020 regime), EndBorrow normalization of Shared(0)/Frozen(0) to Owned, and a checked state-construction surface (state_from_parts) whose complete failure set is the StateError enum.
// CEP:WHY: SPEC-INDEX §5.1 pins planck-ownership-math as the extraction home of the §5.4 ownership/loan state machine and conflict predicate; the Volume 06 borrow checker evaluates trans at every program point (§6.7.5) and derives its diagnostics from the rejection entries (§6.7.5, OB-06-10), so the checker must call exactly these functions and no others.
// CEP:STATUS: complete
// CEP:FAILURE: trans, permits, conflict_at, conflict, and incompatible are total pure functions with no failure mode; a Rejection is the value Transition::Rejected, not an error. The only fallible public function is state_from_parts, whose complete failure set is exactly StateError::InvalidStateKind(u8) (kind code outside 0..=5) and StateError::StrayLoanCount { kind, loans } (a nonzero loan count supplied for one of the four counter-free kinds 0, 1, 2, 4); every other (kind, loans) input is accepted, and every u16 loan count is a valid Shared/Frozen counter per the pinned table. Any divergence of trans from the pinned table over the complete concrete domain (131,076 states × 7 actions = 917,532 cells) fails the EXH certificates and the crate must not ship.
// CEP:ASSUMES: SPEC-INDEX §5.4 (pinned table, copied byte-for-byte into Volume 06 §6.1.1 and normative without modification; per Volume 06 §6.1.1/§0.5, §5.4 wins over any divergent reading); Volume 06 §6.1.2 (a rejected action does not change the place state — modeled here by Rejected carrying no successor state); §6.1.3 (derived predicates: permits, conflictAt, incompatible); §6.1.4 (Frozen is the shared-borrowed state reached when a mutable borrow is attempted against outstanding shared loans; Shared(n) and Frozen(n) are behaviorally identical on every action in the flag-erasure sense of "differ only in recorded history" — the literal OB-06-5 equation is contradicted by the pinned table's own Read/BorrowShared cells and is flagged in the extraction report, OB-06-5); §6.5 d3 (loan counter is u16 with saturating rejection at 65535, P06-E020); SPEC-INDEX §5.2 (kernel implementation law: core:: only, no allocation, no panic, no unsafe in non-test code); §5.1 (equivalence claim form). Found inconsistencies resolved here and flagged in the extraction report: (1) Volume 06 §6.1.3's incompatible-pair enumeration lists (MoveOut, MoveOut), (MoveOut, BorrowMut), and (BorrowMut, BorrowMut) although those pairs violate the section's own definitional formula incompatible(a,b) ⟺ permits(a) ∩ permits(b) = ∅ — both actions are individually permitted in Owned — so the definitional formula is implemented (it is exactly what SPEC-INDEX §5.4 pins: "cannot both be permitted in the same state" and what OB-06-6 states as the EXH property); (2) §6.1.3's permits row for BorrowShared says "Shared(n) for all n" while the pinned table rejects BorrowShared at n = 65535; the pinned table wins (saturation carve-out); (3) the pinned table's result spelling `Uninit` denotes the state `Uninitialized` of the pinned state list.
// CEP:COST: O(1) per call: trans is one two-level match (≤ 3 comparisons); permits / conflict_at / conflict / incompatible are O(1) evaluations over it; state_from_parts is one match; incompatible additionally reduces to a bitwise AND of two named family masks; no allocation anywhere; hot functions #[inline].
// CEP:EVIDENCE: EXH exh_trans_pinned_cells_42_families (all 42 family cells of the pinned table enumerated against literal §5.4 values, 84 concrete cells over loan representatives {0, 1, 65534, 65535}); EXH exh_trans_full_domain_131076x7 (the complete concrete domain 131,076 states × 7 actions = 917,532 cells vs an independent action-first transcription, with permits/conflict_at consistency and loan-range law per cell); EXH exh_endborrow_law_full_u16_sweep (OB-06-2/OB-06-3: EndBorrow over the full u16 loan domain for Shared and Frozen plus MutableBorrowed and all non-borrowed states, 131,073+ cases); EXH exh_loan_counter_law_full_sweep (OB-06-4: BorrowShared iff-form over all n ∈ 0..=65535 for both families, 131,072 cases); EXH exh_loan_saturation_boundaries_65534_65535 (exact boundary cells); EXH exh_shared_frozen_identity_full_sweep (OB-06-5: 65,536 × 7 × 2 = 917,504 comparisons, classification plus flag-erasure successor identity per §6.1.4); EXH exh_conflict_in_state_294_triples (the pinned 294 family triples, plus the saturation regime 98 triples); EXH exh_incompatible_49_pairs (all 49 ordered pairs vs the full-domain witness definition, symmetry, irreflexivity, and machine reconciliation of Volume 06 §6.1.3's pair list); EXH exh_permits_family_sets_131076 (observed permit family sets over the full domain vs §6.1.3's table and the named masks); EXH exh_state_from_parts_full_domain (kind domain 0..=255 × boundary loans, full 65,536-loan sweep for both counter kinds, round-trips); PROP prop_random_trans_matches_reference (10,000 deterministic PRNG samples, paired with the EXH above).
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over the complete state × action domain, test-as-proof)
// CEP:FV-SCRIPT: crates/planck-ownership-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-METHOD: EXH for every public function (PROP only as the paired sampler per SPEC-INDEX §5.1/§4.3)
// CEP:FV-BOUND: full 42-cell pinned table + 294 conflict triples — lifted over the complete concrete domain: 131,076 states × 7 actions = 917,532 trans cells; 294 family conflict triples + 98 saturation-regime triples; 49 incompatibility pairs; full u16 loan sweeps (0..=65535) for the EndBorrow, counter, identity, and permit laws
// CEP:FV-GAP: none
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete: every public function's stated domain is enumerated by an EXH test (trans over all 131,076 × 7 concrete cells; conflict/conflict_at over the 42 family cells, 294 pinned triples and the saturation regime; incompatible over all 49 ordered pairs against the full-domain witness definition; permits over the full domain; state_from_parts over its complete kind domain and full valid loan domain); there is no untested region and therefore no refinement gap. MATP attempt disclosure (G18 dual-engine record): MATP at pinned revision 677bea9 was attempted on the ownership obligations tools/fv/matp/problems/own_table_cells.p, own_table_axioms.p, own_endborrow_zero.p (Theorem, artifacts STATUS: pass) and own_endborrow_borrowed_sh.p, own_endborrow_borrowed_mut.p, own_endborrow_borrowed_fr.p (Saturated without refutation within the documented bound — recorded honestly as STATUS: fail in their artifacts, not discharged by MATP). The EXH certificates in this crate are the complete discharge for every obligation including the EndBorrow-from-borrowed-state laws those three problems encode (exh_endborrow_law_full_u16_sweep enumerates the full domain MATP could not close); per SPEC-INDEX section 4 an EXH enumeration over the full finite domain is a complete proof for that domain, so no obligation remains open.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named constants (Law 7: constants are named, never magic literals).
// ---------------------------------------------------------------------------

/// Upper bound of the loan counter, exactly 65535: `BorrowShared` on a place with
/// n = 65535 outstanding shared loans is rejected (pinned table cell; Volume 06 §6.5 d3
/// "u16, saturating rejection at 65535, P06-E020").
///
/// Refinement anchor: SPEC-INDEX §5.4 / Volume 06 §6.1.1, §6.5 d3. Domain: constant.
/// Failure set: none. Certificate: EXH `exh_loan_counter_law_full_sweep`,
/// `exh_loan_saturation_boundaries_65534_65535`.
///
/// CEP:WHAT: The pinned loan-count saturation bound, exactly 65535 = u16::MAX.
/// CEP:WHY: The saturation cell of the pinned table is expressed against this constant, never a bare literal.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: loans: u16 makes every value 0..=65535 a representable counter; 65535 is the largest n with a defined (non-saturating) EndBorrow decrement.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_loan_counter_law_full_sweep; EXH exh_loan_saturation_boundaries_65534_65535.
pub const MAX_LOANS: u16 = 65_535;

/// Loan count produced by `BorrowShared` on an `Owned` place: exactly Shared(1) (pinned table cell `Owned + BorrowShared → Shared(1)`).
///
/// CEP:WHAT: Initial shared-loan count of a fresh shared borrow.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: SPEC-INDEX §5.4 pins the Owned + BorrowShared successor as Shared(1).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_trans_pinned_cells_42_families; EXH exh_trans_full_domain_131076x7.
pub const INITIAL_SHARED_LOANS: u16 = 1;

/// Kind code of `Uninitialized` in the pinned state list order (SPEC-INDEX §5.4: state 1 of 6).
///
/// CEP:WHAT: Packed kind code used by state_from_parts / State::kind.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: kind codes follow the declaration order of the pinned §5.4 state list: Uninitialized, Owned, MovedFrom, Shared, MutableBorrowed, Frozen.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_UNINITIALIZED: u8 = 0;
/// Kind code of `Owned` (state 2 of the pinned §5.4 list).
///
/// CEP:WHAT: Packed kind code 1.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned §5.4 state-list order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_OWNED: u8 = 1;
/// Kind code of `MovedFrom` (state 3 of the pinned §5.4 list).
///
/// CEP:WHAT: Packed kind code 2.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned §5.4 state-list order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_MOVED_FROM: u8 = 2;
/// Kind code of `Shared { loans }` (state 4 of the pinned §5.4 list).
///
/// CEP:WHAT: Packed kind code 3; the only kind besides Frozen that carries a counter.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned §5.4 state-list order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_SHARED: u8 = 3;
/// Kind code of `MutableBorrowed` (state 5 of the pinned §5.4 list).
///
/// CEP:WHAT: Packed kind code 4.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned §5.4 state-list order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_MUTABLE_BORROWED: u8 = 4;
/// Kind code of `Frozen { loans }` (state 6 of the pinned §5.4 list).
///
/// CEP:WHAT: Packed kind code 5.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned §5.4 state-list order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub const KIND_FROZEN: u8 = 5;

/// Number of pinned state kinds, exactly 6 (SPEC-INDEX §5.4 state list).
///
/// CEP:WHAT: Cardinality of the pinned state-kind set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: the pinned list enumerates exactly six states.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain (kinds 0..=5 valid, all others rejected).
pub const STATE_KIND_COUNT: u8 = 6;

/// Column index of `Read` in the pinned §5.4 table (column 1 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: column order is the pinned table order: Read, Write, MoveOut, BorrowShared, BorrowMut, Drop, EndBorrow.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_READ: usize = 0;
/// Column index of `Write` in the pinned §5.4 table (column 2 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_WRITE: usize = 1;
/// Column index of `MoveOut` in the pinned §5.4 table (column 3 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_MOVE_OUT: usize = 2;
/// Column index of `BorrowShared` in the pinned §5.4 table (column 4 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_BORROW_SHARED: usize = 3;
/// Column index of `BorrowMut` in the pinned §5.4 table (column 5 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_BORROW_MUT: usize = 4;
/// Column index of `Drop` in the pinned §5.4 table (column 6 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_DROP: usize = 5;
/// Column index of `EndBorrow` in the pinned §5.4 table (column 7 of 7).
///
/// CEP:WHAT: Pinned action column index.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
pub const ACTION_INDEX_END_BORROW: usize = 6;

/// Number of pinned actions, exactly 7 (SPEC-INDEX §5.4 action list).
///
/// CEP:WHAT: Cardinality of the pinned action set.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: the pinned list enumerates exactly seven actions.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_trans_pinned_cells_42_families (7 columns enumerated).
pub const ACTION_COUNT: usize = 7;

/// The pinned action set in pinned table column order (SPEC-INDEX §5.4).
///
/// CEP:WHAT: Enumeration carrier for the seven pinned actions.
/// CEP:WHY: EXH tests and downstream checkers iterate the pinned columns in order.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant array).
/// CEP:ASSUMES: order = pinned table column order.
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7 (iterates exactly these seven actions).
pub const ACTIONS: [Action; ACTION_COUNT] = [
    Action::Read,
    Action::Write,
    Action::MoveOut,
    Action::BorrowShared,
    Action::BorrowMut,
    Action::Drop,
    Action::EndBorrow,
];

/// Family-level permit sets, one bitmask per pinned action column; bit k (k = kind code)
/// is set iff SOME concrete state of family k permits the action (Volume 06 §6.1.3
/// `permits(a) = { s ∈ State : trans(s, a) ≠ Rej }`, lifted to families).
///
/// Saturation note (flagged inconsistency, resolved in favor of the pinned table):
/// §6.1.3 writes BorrowShared's set as "Shared(n) for all n, Frozen(n) for all n", but the
/// pinned table rejects BorrowShared at n = 65535; the family mask records family membership,
/// and EXH `exh_incompatible_49_pairs` proves over the complete concrete domain that family-mask
/// disjointness coincides exactly with concrete disjointness (no witness exists in either case).
///
/// Refinement anchor: Volume 06 §6.1.3 / SPEC-INDEX §5.4. Domain: bits 0..=5 (kind codes).
/// Failure set: none. Certificate: EXH `exh_permits_family_sets_131076`, `exh_incompatible_49_pairs`.
///
/// CEP:WHAT: Family-level permit bitmask per pinned action (definition-of-record for incompatible).
/// CEP:WHY: incompatible(a, b) must be O(1) and const-expressible; the masks carry the exact permit families as const data.
/// CEP:STATUS: complete
/// CEP:FAILURE: A mask that disagrees with the observed permit family set over the full concrete domain fails the EXH test.
/// CEP:ASSUMES: bit k encodes state-kind k (kind codes 0..=5, per the pinned §5.4 state-list order).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_permits_family_sets_131076; EXH exh_incompatible_49_pairs.
pub const PERMIT_FAMILY_MASK: [u8; ACTION_COUNT] = {
    let uninit_bit = 1u8 << KIND_UNINITIALIZED;
    let owned_bit = 1u8 << KIND_OWNED;
    let moved_from_bit = 1u8 << KIND_MOVED_FROM;
    let shared_bit = 1u8 << KIND_SHARED;
    let mutable_borrowed_bit = 1u8 << KIND_MUTABLE_BORROWED;
    let frozen_bit = 1u8 << KIND_FROZEN;
    let mut masks = [0u8; ACTION_COUNT];
    // Volume 06 §6.1.3 permits table, one row per pinned action column.
    masks[ACTION_INDEX_READ] = owned_bit | shared_bit | frozen_bit;
    masks[ACTION_INDEX_WRITE] = uninit_bit | owned_bit | moved_from_bit;
    masks[ACTION_INDEX_MOVE_OUT] = owned_bit;
    masks[ACTION_INDEX_BORROW_SHARED] = owned_bit | shared_bit | frozen_bit;
    masks[ACTION_INDEX_BORROW_MUT] = owned_bit;
    masks[ACTION_INDEX_DROP] = uninit_bit | owned_bit | moved_from_bit;
    masks[ACTION_INDEX_END_BORROW] = shared_bit | frozen_bit | mutable_borrowed_bit;
    masks
};

// ---------------------------------------------------------------------------
// Pinned state space (SPEC-INDEX §5.4 state list).
// ---------------------------------------------------------------------------

/// A place state of the pinned ownership machine (SPEC-INDEX §5.4; Volume 06 §6.1.1).
///
/// Exactly the six pinned states; `Shared` and `Frozen` carry the u16 loan counter whose
/// whole range 0..=65535 is valid (65535 is the saturation boundary of `BorrowShared`).
///
/// Refinement anchor: SPEC-INDEX §5.4 / Volume 06 §6.1.1. Domain: the six variants; loans ∈ 0..=65535 (u16).
/// Failure set: none (value carrier). Certificate: EXH `exh_trans_full_domain_131076x7` (every concrete state exercised), `exh_state_from_parts_full_domain`.
///
/// CEP:WHAT: The pinned place-state enum of the ownership machine.
/// CEP:WHY: Every borrow-checker decision (Volume 06 §6.7.5) evaluates trans on exactly this state space.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: the enum is exactly the pinned six-state list; every u16 loan count is valid, so no construction of this type is malformed (checked construction is state_from_parts with StateError).
/// CEP:ASSUMES: the pinned table's result spelling `Uninit` denotes `Uninitialized`; Frozen(n) records that a mutable borrow was attempted against a shared-borrowed place (§6.1.4) and is behaviorally identical to Shared(n) on every action.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7; EXH exh_shared_frozen_identity_full_sweep; EXH exh_state_from_parts_full_domain.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// No value has ever been written to the place (pinned state 1).
    Uninitialized,
    /// The place holds a fully owned value (pinned state 2).
    Owned,
    /// The value was moved out; only Write/Drop are defined (pinned state 3).
    MovedFrom,
    /// Shared-borrowed with `loans` outstanding shared loans, 0..=65535 (pinned state 4).
    Shared {
        /// Outstanding shared-loan count, 0..=65535 (u16; saturation at MAX_LOANS).
        loans: u16,
    },
    /// Exclusively borrowed; no counter (pinned state 5).
    MutableBorrowed,
    /// Shared-borrowed against which a mutable borrow was attempted; `loans` 0..=65535 (pinned state 6; Volume 06 §6.1.4).
    Frozen {
        /// Outstanding shared-loan count, 0..=65535 (u16; saturation at MAX_LOANS).
        loans: u16,
    },
}

impl State {
    /// The pinned kind code of this state (0..=5, SPEC-INDEX §5.4 state-list order).
    ///
    /// Refinement anchor: SPEC-INDEX §5.4 / Volume 06 §6.1.1. Domain: all six states (total).
    /// Failure set: none. Certificate: EXH `exh_state_from_parts_full_domain` (round-trip with state_from_parts over the full kind and loan domains).
    ///
    /// CEP:WHAT: Packed kind code of the state, inverse of state_from_parts' kind input.
    /// CEP:WHY: The checker serializes states; the code is the pinned declaration order.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure function).
    /// CEP:ASSUMES: kind codes are the named KIND_* constants.
    /// CEP:COST: O(1), const-evaluable match; CEP-0.
    /// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
    #[inline]
    pub const fn kind(self) -> u8 {
        match self {
            State::Uninitialized => KIND_UNINITIALIZED,
            State::Owned => KIND_OWNED,
            State::MovedFrom => KIND_MOVED_FROM,
            State::Shared { .. } => KIND_SHARED,
            State::MutableBorrowed => KIND_MUTABLE_BORROWED,
            State::Frozen { .. } => KIND_FROZEN,
        }
    }

    /// The loan counter carried by this state: `loans` for Shared/Frozen, exactly 0 otherwise.
    ///
    /// Refinement anchor: SPEC-INDEX §5.4 (only Shared/Frozen carry { loans: u16 }).
    /// Domain: all six states (total). Failure set: none.
    /// Certificate: EXH `exh_state_from_parts_full_domain` (full 0..=65535 sweep for both counter kinds); EXH `exh_trans_full_domain_131076x7` (every Applied result has loan_count() <= MAX_LOANS).
    ///
    /// CEP:WHAT: Loan-count accessor; 0 for the four counter-free kinds.
    /// CEP:WHY: The OB-06-4 law ("no transition produces a counter outside 0..=65535") is stated against this accessor.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure function).
    /// CEP:ASSUMES: counter-free kinds report exactly 0, never a stale value.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain; EXH exh_trans_full_domain_131076x7.
    #[inline]
    pub const fn loan_count(self) -> u16 {
        match self {
            State::Shared { loans } | State::Frozen { loans } => loans,
            State::Uninitialized | State::Owned | State::MovedFrom | State::MutableBorrowed => 0,
        }
    }

    /// Is this a borrowed state? Exactly true for Shared(_), Frozen(_), MutableBorrowed
    /// (Volume 06 OB-06-2: "borrowed states are exactly {Shared(n), Frozen(n), MutableBorrowed}").
    ///
    /// Refinement anchor: Volume 06 §6.12 OB-06-2 / SPEC-INDEX §5.4 EndBorrow law.
    /// Domain: all six states (total). Failure set: none.
    /// Certificate: EXH `exh_endborrow_law_full_u16_sweep` — over the complete concrete domain,
    /// is_borrowed(s) ⟺ permits(s, EndBorrow) (EndBorrow never yields Rej from a borrowed state
    /// and is rejected from every non-borrowed state).
    ///
    /// CEP:WHAT: Borrowed-state classifier backing the pinned EndBorrow law.
    /// CEP:WHY: OB-06-2 states the EndBorrow totality law over exactly this set.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure predicate); divergence from EndBorrow-permissibility fails the EXH sweep.
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_endborrow_law_full_u16_sweep.
    #[inline]
    pub const fn is_borrowed(self) -> bool {
        matches!(
            self,
            State::Shared { .. } | State::Frozen { .. } | State::MutableBorrowed
        )
    }
}

// ---------------------------------------------------------------------------
// Pinned action set (SPEC-INDEX §5.4 action list).
// ---------------------------------------------------------------------------

/// An action on a place, exactly the seven pinned actions (SPEC-INDEX §5.4; Volume 06 §6.1.1).
///
/// Refinement anchor: SPEC-INDEX §5.4 / Volume 06 §6.1.1. Domain: the seven variants.
/// Failure set: none (value carrier). Certificate: EXH `exh_trans_full_domain_131076x7` (every action exercised against every state).
///
/// CEP:WHAT: The pinned action enum of the ownership machine.
/// CEP:WHY: The pinned 6×7 table is indexed by exactly these columns, in ACTIONS order.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: the enum is exactly the pinned seven-action list; no other action exists in edition P1.
/// CEP:ASSUMES: none.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Read the place's value (column 1).
    Read,
    /// Write / initialize the place (column 2).
    Write,
    /// Move the value out, transferring ownership (column 3).
    MoveOut,
    /// Create a shared loan, incrementing the counter (column 4).
    BorrowShared,
    /// Create an exclusive loan (column 5).
    BorrowMut,
    /// Discharge the owner obligation / destroy (column 6).
    Drop,
    /// Terminate one loan, decrementing the counter or restoring ownership (column 7).
    EndBorrow,
}

impl Action {
    /// The pinned column index of this action, 0..=6 (ACTIONS order = pinned table column order).
    ///
    /// Refinement anchor: SPEC-INDEX §5.4 table column order. Domain: all seven actions (total).
    /// Failure set: none. Certificate: EXH `exh_permits_family_sets_131076` (masks indexed by this value equal the observed permit sets).
    ///
    /// CEP:WHAT: Column index of the action in the pinned table.
    /// CEP:WHY: PERMIT_FAMILY_MASK and the table-driven tests are indexed by exactly this value.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure function; every result is < ACTION_COUNT).
    /// CEP:ASSUMES: ACTIONS and ACTION_INDEX_* agree (both pinned to the §5.4 column order).
    /// CEP:COST: O(1), const-evaluable; CEP-0.
    /// CEP:EVIDENCE: EXH exh_permits_family_sets_131076.
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            Action::Read => ACTION_INDEX_READ,
            Action::Write => ACTION_INDEX_WRITE,
            Action::MoveOut => ACTION_INDEX_MOVE_OUT,
            Action::BorrowShared => ACTION_INDEX_BORROW_SHARED,
            Action::BorrowMut => ACTION_INDEX_BORROW_MUT,
            Action::Drop => ACTION_INDEX_DROP,
            Action::EndBorrow => ACTION_INDEX_END_BORROW,
        }
    }
}

// ---------------------------------------------------------------------------
// Transition result: State ⊔ {Rejection} (Rejection is a VALUE, not an error).
// ---------------------------------------------------------------------------

/// The result of one application of the pinned transition function: a successor state or
/// the value `Rejected` (SPEC-INDEX §5.4 codomain `State ⊔ {Rejection}`).
///
/// A `Rejected` outcome is a well-defined VALUE of the machine, never an error: per Volume 06
/// §6.1.2 a rejected action produces one diagnostic and leaves the place state unchanged,
/// which this type models by carrying no successor state.
///
/// Refinement anchor: SPEC-INDEX §5.4 (codomain) / Volume 06 §6.1.2 (rejection semantics).
/// Domain: Applied carries exactly one concrete successor State; Rejected carries nothing.
/// Failure set: none (value carrier). Certificate: EXH `exh_trans_full_domain_131076x7`.
///
/// CEP:WHAT: Sum type State ⊔ {Rejection} returned by trans.
/// CEP:WHY: SPEC-INDEX §5.4 and the task pin make Rejection a value of the machine, distinct from the crate's error enum for malformed constructions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (value carrier; no construction of this type is malformed).
/// CEP:ASSUMES: a rejected action leaves the state unchanged (Volume 06 §6.1.2), so Rejected carries no successor.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7 (every cell classified against the independent transcription).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transition {
    /// The action was permitted; the place is now in the carried successor state.
    Applied(State),
    /// The action was rejected by the pinned table; the place state is unchanged (Volume 06 §6.1.2).
    Rejected,
}

impl Transition {
    /// Did the pinned table reject this action?
    ///
    /// Refinement anchor: SPEC-INDEX §5.4 rejection entries / Volume 06 §6.1.3 conflictAt.
    /// Domain: all Transition values (total). Failure set: none.
    /// Certificate: EXH `exh_trans_full_domain_131076x7` (conflict_at(s, a) ⟺ trans(s, a).is_rejected() over all 917,532 cells).
    ///
    /// CEP:WHAT: Rejection classifier of the trans codomain.
    /// CEP:WHY: conflict_at and permits are defined exactly through this predicate.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure predicate).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7.
    #[inline]
    pub const fn is_rejected(self) -> bool {
        matches!(self, Transition::Rejected)
    }

    /// The successor state iff the action was applied, else `None`.
    ///
    /// Refinement anchor: SPEC-INDEX §5.4 codomain. Domain: all Transition values (total).
    /// Failure set: none. Certificate: EXH `exh_trans_full_domain_131076x7` (Applied cells carry the pinned successor; every carried state satisfies loan_count() <= MAX_LOANS).
    ///
    /// CEP:WHAT: Successor-state accessor of Applied.
    /// CEP:WHY: Downstream checkers consume the successor state directly.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (total pure function returning Option; no unwrap anywhere).
    /// CEP:ASSUMES: none.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7.
    #[inline]
    pub const fn state(self) -> Option<State> {
        match self {
            Transition::Applied(s) => Some(s),
            Transition::Rejected => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Error enum for malformed state constructions (complete; the ONLY fallible surface).
// ---------------------------------------------------------------------------

/// The complete failure set of `state_from_parts` — the crate's only fallible function.
///
/// A `Rejection` of the transition function is NOT an error (it is the value
/// `Transition::Rejected`); this enum exists only for malformed loan-state constructions
/// supplied through the checked packed-construction surface.
///
/// Refinement anchor: SPEC-INDEX §5.4 state list (which kinds exist and which carry a counter) / §5.2 (explicit, complete error enums). Domain: see variant docs.
/// Failure set: this type IS the failure set; both variants are listed and reachable (EXH-tested).
/// Certificate: EXH `exh_state_from_parts_full_domain` (every variant constructed over its full stated domain).
///
/// CEP:WHAT: Complete error enum for malformed packed state constructions.
/// CEP:WHY: SPEC-INDEX §5.2 requires every failure to return an explicit, complete error enum; trans itself cannot fail, so this is the crate's entire failure surface.
/// CEP:STATUS: complete
/// CEP:FAILURE: This type IS the failure set; exactly the two variants below exist and both are EXH-reachable.
/// CEP:ASSUMES: kind codes are the KIND_* constants 0..=5; a counter-free kind (0, 1, 2, 4) must be constructed with loans == 0; any u16 loan count is valid for kinds 3 and 5.
/// CEP:COST: Copy type, no allocation.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StateError {
    /// `kind` is not one of the six pinned state kinds 0..=5 (SPEC-INDEX §5.4 state list);
    /// payload is the offending kind code.
    InvalidStateKind(u8),
    /// A nonzero loan count was supplied for a counter-free kind (Uninitialized, Owned,
    /// MovedFrom, MutableBorrowed); payloads are the offending kind code and loan count.
    StrayLoanCount {
        /// The counter-free kind code the construction was attempted against (0, 1, 2, or 4).
        kind: u8,
        /// The offending nonzero loan count.
        loans: u16,
    },
}

// ---------------------------------------------------------------------------
// The pinned transition function (SPEC-INDEX §5.4, table copied EXACTLY).
// ---------------------------------------------------------------------------

/// The pinned total transition function `trans : State × Action → State ⊔ {Rejection}`
/// (SPEC-INDEX §5.4; Volume 06 §6.1.1 embeds the table verbatim).
///
/// Pinned 42-cell table (state families × actions; `Uninit` result spelling = `Uninitialized`):
///
/// | State \ Action  | Read       | Write | MoveOut   | BorrowShared                  | BorrowMut | Drop  | EndBorrow                        |
/// |-----------------|------------|-------|-----------|-------------------------------|-----------|-------|----------------------------------|
/// | Uninitialized   | Rej        | Owned | Rej       | Rej                           | Rej       | Uninit| Rej                              |
/// | Owned           | Owned      | Owned | MovedFrom | Shared(1)                     | MutableBorrowed | Uninit | Rej                        |
/// | MovedFrom       | Rej        | Owned | Rej       | Rej                           | Rej       | Uninit| Rej                              |
/// | Shared(n)       | Shared(n)  | Rej   | Rej       | Shared(n+1) if n < 65535 else Rej | Rej   | Rej   | n≥1 → Shared(n−1); n=0 → Owned   |
/// | MutableBorrowed | Rej        | Rej   | Rej       | Rej                           | Rej       | Rej   | Owned                            |
/// | Frozen(n)       | Frozen(n)  | Rej   | Rej       | Frozen(n+1) if n < 65535 else Rej | Rej   | Rej   | n≥1 → Shared(n−1); n=0 → Owned   |
///
/// The Frozen EndBorrow successor `Shared(n−1)` is copied EXACTLY as pinned (the frozen flag
/// is dropped on the first loan termination). Behavioral identity with Shared (OB-06-5) holds
/// modulo this flag: the pinned table's own Read and BorrowShared cells preserve the row kind
/// (Shared(n)→Shared(n), Frozen(n)→Frozen(n)), so the literal equation trans(Shared(n), a) =
/// trans(Frozen(n), a) is false for those two cells; Volume 06 §6.1.4's "differ only in
/// recorded history" is machine-checked as equality after flag erasure (see the identity EXH).
///
/// Refinement anchor: SPEC-INDEX §5.4 / Volume 06 §6.1.1 (verbatim table), §6.1.2 (rejection leaves state unchanged), §6.5 d3 (saturation at MAX_LOANS = 65535).
/// Input domain: every concrete state (all six kinds; loans over the full u16 range 0..=65535) × all seven actions — total function, 131,076 × 7 = 917,532 concrete cells.
/// Failure set: none (Rejection is the value `Transition::Rejected`, not an error; arithmetic guarded so no overflow: n+1 only when n < 65535, n−1 only when n ≥ 1).
/// Certificate: EXH `exh_trans_pinned_cells_42_families` (all 42 family cells vs literal §5.4 values) and EXH `exh_trans_full_domain_131076x7` (complete 917,532-cell domain vs independent action-first transcription); PROP `prop_random_trans_matches_reference` paired.
///
/// CEP:WHAT: The pinned 6×7 ownership transition table as a total function.
/// CEP:WHY: Volume 06 §6.7.5 evaluates exactly this function at every program point; SPEC-INDEX §5.1 names it this crate's central deliverable.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure function; every cell defined (OB-06-1 totality); a rejected action is the value Transition::Rejected and the caller's state is unchanged per §6.1.2.
/// CEP:ASSUMES: the table above is copied exactly from SPEC-INDEX §5.4; Shared(0)/Frozen(0) + EndBorrow normalize to Owned (pinned law); BorrowShared saturates at n = 65535 with Rejection (P06-E020 regime).
/// CEP:COST: O(1), one two-level match, ≤ 3 comparisons, allocation-free, #[inline]; CEP-0.
/// CEP:EVIDENCE: EXH exh_trans_pinned_cells_42_families; EXH exh_trans_full_domain_131076x7; PROP prop_random_trans_matches_reference.
#[inline]
pub fn trans(state: State, action: Action) -> Transition {
    match state {
        State::Uninitialized => match action {
            Action::Read | Action::MoveOut | Action::BorrowShared | Action::BorrowMut
            | Action::EndBorrow => Transition::Rejected,
            Action::Write => Transition::Applied(State::Owned),
            Action::Drop => Transition::Applied(State::Uninitialized),
        },
        State::Owned => match action {
            Action::Read => Transition::Applied(State::Owned),
            Action::Write => Transition::Applied(State::Owned),
            Action::MoveOut => Transition::Applied(State::MovedFrom),
            Action::BorrowShared => {
                Transition::Applied(State::Shared { loans: INITIAL_SHARED_LOANS })
            }
            Action::BorrowMut => Transition::Applied(State::MutableBorrowed),
            Action::Drop => Transition::Applied(State::Uninitialized),
            Action::EndBorrow => Transition::Rejected,
        },
        State::MovedFrom => match action {
            Action::Read | Action::MoveOut | Action::BorrowShared | Action::BorrowMut
            | Action::EndBorrow => Transition::Rejected,
            Action::Write => Transition::Applied(State::Owned),
            Action::Drop => Transition::Applied(State::Uninitialized),
        },
        State::Shared { loans } => match action {
            Action::Read => Transition::Applied(State::Shared { loans }),
            Action::Write | Action::MoveOut | Action::BorrowMut | Action::Drop => {
                Transition::Rejected
            }
            Action::BorrowShared => {
                if loans < MAX_LOANS {
                    Transition::Applied(State::Shared { loans: loans + 1 })
                } else {
                    Transition::Rejected
                }
            }
            Action::EndBorrow => {
                if loans >= 1 {
                    Transition::Applied(State::Shared { loans: loans - 1 })
                } else {
                    Transition::Applied(State::Owned)
                }
            }
        },
        State::MutableBorrowed => match action {
            Action::Read | Action::Write | Action::MoveOut | Action::BorrowShared
            | Action::BorrowMut | Action::Drop => Transition::Rejected,
            Action::EndBorrow => Transition::Applied(State::Owned),
        },
        State::Frozen { loans } => match action {
            Action::Read => Transition::Applied(State::Frozen { loans }),
            Action::Write | Action::MoveOut | Action::BorrowMut | Action::Drop => {
                Transition::Rejected
            }
            Action::BorrowShared => {
                if loans < MAX_LOANS {
                    Transition::Applied(State::Frozen { loans: loans + 1 })
                } else {
                    Transition::Rejected
                }
            }
            Action::EndBorrow => {
                if loans >= 1 {
                    // Copied EXACTLY as pinned: Frozen(n) + EndBorrow → Shared(n-1).
                    Transition::Applied(State::Shared { loans: loans - 1 })
                } else {
                    Transition::Applied(State::Owned)
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Derived predicates (Volume 06 §6.1.3, derived exactly from the rejection entries).
// ---------------------------------------------------------------------------

/// Is `action` permitted in `state`? Exactly `trans(state, action) ≠ Rejection`
/// (Volume 06 §6.1.3: `permits(a) = { s ∈ State : trans(s, a) ≠ Rej }`, membership form).
///
/// Refinement anchor: Volume 06 §6.1.3 / SPEC-INDEX §5.4 rejection entries.
/// Input domain: all concrete states (loans 0..=65535) × all seven actions — total predicate.
/// Failure set: none.
/// Certificate: EXH `exh_trans_full_domain_131076x7` (permits(s, a) ⟺ !trans(s, a).is_rejected() over all 917,532 cells) and EXH `exh_permits_family_sets_131076` (observed family sets equal PERMIT_FAMILY_MASK and Volume 06 §6.1.3's permits table with the pinned saturation carve-out at n = 65535 for BorrowShared).
///
/// CEP:WHAT: Permit membership predicate derived exactly from the rejection entries.
/// CEP:WHY: conflict and incompatible are defined through permit sets; the checker's per-point test is exactly this predicate.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total pure predicate).
/// CEP:ASSUMES: Volume 06 §6.1.3's permits table is family-level; at n = 65535 the pinned table rejects BorrowShared (saturation wins per §6.1.1).
/// CEP:COST: O(1), one trans call, #[inline]; CEP-0.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7; EXH exh_permits_family_sets_131076.
#[inline]
pub fn permits(state: State, action: Action) -> bool {
    !trans(state, action).is_rejected()
}

/// Does a single action hit a rejection cell in `state`? Exactly
/// `conflict_at(state, action) ⟺ trans(state, action) = Rejection`
/// (Volume 06 §6.1.3 derived notion 1: the per-state conflict used by the checker at each program point, §6.7).
///
/// Refinement anchor: Volume 06 §6.1.3 (notion 1) / SPEC-INDEX §5.4 rejection entries.
/// Input domain: all concrete states (loans 0..=65535) × all seven actions — total predicate.
/// Failure set: none.
/// Certificate: EXH `exh_conflict_in_state_294_triples` (conflict_at consistent with conflict over the 294 pinned triples and the saturation regime) and EXH `exh_trans_full_domain_131076x7` (conflict_at(s, a) ⟺ trans(s, a).is_rejected() over all 917,532 cells — the OB-06-6 conflictAt obligation).
///
/// CEP:WHAT: Per-state, per-action conflict predicate (OB-06-6, notion 1).
/// CEP:WHY: Volume 06 §6.7 evaluates exactly this predicate per program point; conflict is its pairwise lift.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total pure predicate).
/// CEP:ASSUMES: none.
/// CEP:COST: O(1), one trans call, #[inline]; CEP-0.
/// CEP:EVIDENCE: EXH exh_trans_full_domain_131076x7; EXH exh_conflict_in_state_294_triples.
#[inline]
pub fn conflict_at(state: State, action: Action) -> bool {
    trans(state, action).is_rejected()
}

/// The pinned conflict predicate over an action PAIR in one state: `conflict(state, a, b)`
/// holds iff the two actions cannot both be permitted in `state` — exactly
/// `trans(state, a) = Rejection or trans(state, b) = Rejection` (SPEC-INDEX §5.4: "conflict(a, b)
/// holds iff two actions on the same place cannot both be permitted in the same state").
///
/// Equivalent closed forms, all machine-checked: `conflict(state, a, b) ⟺
/// conflict_at(state, a) ∨ conflict_at(state, b) ⟺ ¬(permits(state, a) ∧ permits(state, b))`.
///
/// Note on scope: this is the state-indexed pair form. Volume 06 §6.1.3 additionally derives
/// the state-free closure `incompatible(a, b) ⟺ permits(a) ∩ permits(b) = ∅` (∀ states:
/// conflict); see `incompatible`. The converse of a sequential-composition reading ("if both
/// actions are individually permitted then applying them in both orders succeeds") is NOT a
/// property of the pinned table — e.g. in `Owned` both `MoveOut` and `Read` are individually
/// permitted, yet `MoveOut` followed by `Read` is rejected (`MovedFrom` rejects `Read`) — so no
/// such claim is made; what IS machine-checked is: successful sequential composition in both
/// orders ⟹ both actions individually permitted in the start state ⟹ ¬conflict (EXH over all
/// 294 family triples plus the saturation regime).
///
/// Refinement anchor: SPEC-INDEX §5.4 conflict predicate / Volume 06 §6.1.3 (notion 1 lifted to pairs).
/// Input domain: all concrete states × all 7×7 = 49 ordered action pairs — total predicate (294 family-representative triples enumerated by the pinned EXH).
/// Failure set: none.
/// Certificate: EXH `exh_conflict_in_state_294_triples` (both closed forms and symmetry over the 294 pinned triples + 98 saturation-regime triples) and EXH `exh_trans_full_domain_131076x7` (cell-level consistency of conflict_at).
///
/// CEP:WHAT: Pinned pairwise conflict predicate in one state, derived exactly from the rejection entries.
/// CEP:WHY: The checker rejects an action pair on one place exactly when this predicate holds; SPEC-INDEX §5.1 names the conflict predicate a central deliverable.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total pure predicate).
/// CEP:ASSUMES: symmetric by construction; conflict(state, a, a) ⟺ conflict_at(state, a); the strong sequential-order law is not a property of the pinned table (documented above with its exact counterexample class).
/// CEP:COST: O(1), two trans calls, #[inline]; CEP-0.
/// CEP:EVIDENCE: EXH exh_conflict_in_state_294_triples; EXH exh_trans_full_domain_131076x7.
#[inline]
pub fn conflict(state: State, a: Action, b: Action) -> bool {
    conflict_at(state, a) || conflict_at(state, b)
}

/// State-free action-pair incompatibility (Volume 06 §6.1.3 derived notion 2, definitional
/// formula): `incompatible(a, b) ⟺ permits(a) ∩ permits(b) = ∅` — there is NO state in which
/// both actions are individually permitted; equivalently the universal closure over all
/// concrete states of `conflict`.
///
/// Exact value over the seven pinned actions: the incompatible unordered pairs are exactly
/// {Write, EndBorrow}, {MoveOut, EndBorrow}, {BorrowMut, EndBorrow}, {Drop, EndBorrow}
/// (8 ordered pairs of the 49). This is the definitional formula's answer, and it is what
/// SPEC-INDEX §5.4 pins ("cannot both be permitted in the same state") and what Volume 06
/// OB-06-6 states as the EXH-checked property.
///
/// FLAGGED INCONSISTENCY (found during extraction, resolution machine-checked): Volume 06
/// §6.1.3's prose additionally lists (MoveOut, MoveOut), (MoveOut, BorrowMut), and
/// (BorrowMut, BorrowMut) as "incompatible", but those pairs violate the section's own
/// definitional formula — permits(MoveOut) = permits(BorrowMut) = {Owned}, so the permit
/// sets intersect (both actions are individually permitted in `Owned`); they are instead
/// pairs that cannot be successfully applied in SEQUENCE from any common state. OB-06-6's
/// claim that the prose list "is exactly the set of pairs with disjoint permit sets" is
/// therefore false for those three entries. Per SPEC-INDEX §5.4 (pinned, unmodifiable) and
/// Volume 06 §6.1.1's authority rule ("§5.4 wins"), the definitional formula is implemented;
/// the three extra prose entries are rejected by EXH `exh_incompatible_49_pairs`, which
/// asserts the Owned joint-permit witness for each.
///
/// Refinement anchor: Volume 06 §6.1.3 (notion 2, formula) / SPEC-INDEX §5.4 conflict predicate.
/// Input domain: all 7 × 7 = 49 ordered action pairs (total predicate over Action × Action).
/// Failure set: none.
/// Certificate: EXH `exh_incompatible_49_pairs` — incompatible(a, b) ⟺ no concrete state among all 131,076 permits both (full-domain witness sweep), plus symmetry, irreflexivity, and the machine reconciliation with Volume 06 §6.1.3's pair list.
///
/// CEP:WHAT: State-free incompatibility closure of the pinned conflict predicate (permit-set disjointness).
/// CEP:WHY: SPEC-INDEX §5.1 names the conflict predicate a deliverable; the state-free form is Volume 06 §6.1.3's notion 2 and is what OB-06-6 exhausts over 49 pairs.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (total pure predicate); the three extra prose entries of Volume 06 §6.1.3 are flagged and machine-rejected as inconsistent with the definitional formula.
/// CEP:ASSUMES: PERMIT_FAMILY_MASK records the exact permit families (with the pinned BorrowShared saturation carve-out at n = 65535, which changes no pairwise disjointness result); family-mask disjointness coincides with concrete disjointness (EXH-proven over the full domain).
/// CEP:COST: O(1), two array reads and a bitwise AND, #[inline]; CEP-0.
/// CEP:EVIDENCE: EXH exh_incompatible_49_pairs; EXH exh_permits_family_sets_131076.
#[inline]
pub fn incompatible(a: Action, b: Action) -> bool {
    PERMIT_FAMILY_MASK[a.index()] & PERMIT_FAMILY_MASK[b.index()] == 0
}

/// Checked construction of a `State` from its pinned kind code and loan count.
///
/// Valid kinds are exactly 0..=5 (`KIND_UNINITIALIZED`..`KIND_FROZEN`, SPEC-INDEX §5.4
/// state-list order). The counter-free kinds 0, 1, 2, 4 require `loans == 0`; the counter
/// kinds 3 (Shared) and 5 (Frozen) accept every u16 loan count 0..=65535 (all are valid
/// states of the pinned table; 65535 is the BorrowShared saturation boundary, not an
/// invalid state).
///
/// Refinement anchor: SPEC-INDEX §5.4 state list / §5.2 (explicit error enum for malformed constructions). Volume 06 §6.1.5 constrains where machines exist; this function validates a state VALUE only.
/// Input domain: kind ∈ 0..=255, loans ∈ 0..=65535 (total function on u8 × u16).
/// Failure set: exactly `StateError::InvalidStateKind(kind)` for kind ∉ 0..=5, and `StateError::StrayLoanCount { kind, loans }` for kind ∈ {0, 1, 2, 4} with loans ≠ 0 — the complete enum, both variants listed in `StateError`'s CEP:FAILURE.
/// Certificate: EXH `exh_state_from_parts_full_domain` — the full kind domain 0..=255 × boundary loans {0, 1, 65534, 65535} (1,024 cases), the full valid loan domain 0..=65535 for both counter kinds (131,072 cases), and kind()/loan_count() round-trips.
///
/// CEP:WHAT: Checked packed-construction surface; the crate's only fallible public function.
/// CEP:WHY: SPEC-INDEX §5.2 demands an explicit error enum for malformed inputs; the checker serializes/deserializes states as (kind, loans) pairs.
/// CEP:STATUS: complete
/// CEP:FAILURE: exactly the two StateError variants (InvalidStateKind for kind codes 6..=255; StrayLoanCount for counter-free kinds constructed with loans ≠ 0); no other failure exists.
/// CEP:ASSUMES: kind codes follow the pinned §5.4 state-list declaration order; every u16 loan count is valid for Shared/Frozen.
/// CEP:COST: O(1), one match, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_state_from_parts_full_domain.
pub fn state_from_parts(kind: u8, loans: u16) -> Result<State, StateError> {
    match kind {
        KIND_SHARED => Ok(State::Shared { loans }),
        KIND_FROZEN => Ok(State::Frozen { loans }),
        KIND_UNINITIALIZED | KIND_OWNED | KIND_MOVED_FROM | KIND_MUTABLE_BORROWED => {
            if loans == 0 {
                Ok(match kind {
                    KIND_UNINITIALIZED => State::Uninitialized,
                    KIND_OWNED => State::Owned,
                    KIND_MOVED_FROM => State::MovedFrom,
                    _ => State::MutableBorrowed,
                })
            } else {
                Err(StateError::StrayLoanCount { kind, loans })
            }
        }
        _ => Err(StateError::InvalidStateKind(kind)),
    }
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;

    /// Total number of concrete states: 4 counter-free kinds + 65,536 Shared + 65,536 Frozen.
    const CONCRETE_STATE_COUNT: u32 = 131_076;

    /// Loan-count representatives covering both pinned boundary regimes: normalization (0),
    /// ordinary (1), last non-saturating (MAX_LOANS - 1 = 65534), saturated (MAX_LOANS = 65535).
    const LOAN_BOUNDARY_REPS: [u16; 4] = [0, 1, MAX_LOANS - 1, MAX_LOANS];

    /// The 294 pinned conflict-triple representatives: one per pinned state family.
    /// Shared(1)/Frozen(1) represent the ordinary loan regime; the saturation regime
    /// (n = 65535) is enumerated separately in exh_conflict_in_state_294_triples.
    const FAMILY_REPRESENTATIVES: [State; 6] = [
        State::Uninitialized,
        State::Owned,
        State::MovedFrom,
        State::Shared { loans: 1 },
        State::MutableBorrowed,
        State::Frozen { loans: 1 },
    ];

    /// Concrete state by sweep index: 0 Uninit, 1 Owned, 2 MovedFrom, 3 MutableBorrowed,
    /// 4..=65539 Shared(k), 65540..=131075 Frozen(k). Total CONCRETE_STATE_COUNT = 131,076.
    fn concrete_state(i: u32) -> State {
        match i {
            0 => State::Uninitialized,
            1 => State::Owned,
            2 => State::MovedFrom,
            3 => State::MutableBorrowed,
            4..=65_539 => State::Shared {
                loans: (i - 4) as u16,
            },
            _ => State::Frozen {
                loans: (i - 65_540) as u16,
            },
        }
    }

    /// Independent transcription of the pinned SPEC-INDEX §5.4 table, written action-first
    /// to be structurally distinct from `trans` (state-first match). EXH proves the two
    /// transcriptions agree on the complete 131,076 × 7 concrete domain.
    fn reference_trans(state: State, action: Action) -> Transition {
        if action == Action::Read {
            return match state {
                State::Owned => Transition::Applied(State::Owned),
                State::Shared { loans } => Transition::Applied(State::Shared { loans }),
                State::Frozen { loans } => Transition::Applied(State::Frozen { loans }),
                State::Uninitialized | State::MovedFrom | State::MutableBorrowed => {
                    Transition::Rejected
                }
            };
        }
        if action == Action::Write {
            return match state {
                State::Uninitialized | State::Owned | State::MovedFrom => {
                    Transition::Applied(State::Owned)
                }
                State::Shared { .. } | State::Frozen { .. } | State::MutableBorrowed => {
                    Transition::Rejected
                }
            };
        }
        if action == Action::MoveOut {
            return match state {
                State::Owned => Transition::Applied(State::MovedFrom),
                _ => Transition::Rejected,
            };
        }
        if action == Action::BorrowShared {
            return match state {
                State::Owned => Transition::Applied(State::Shared {
                    loans: INITIAL_SHARED_LOANS,
                }),
                State::Shared { loans } => {
                    if loans < MAX_LOANS {
                        Transition::Applied(State::Shared { loans: loans + 1 })
                    } else {
                        Transition::Rejected
                    }
                }
                State::Frozen { loans } => {
                    if loans < MAX_LOANS {
                        Transition::Applied(State::Frozen { loans: loans + 1 })
                    } else {
                        Transition::Rejected
                    }
                }
                State::Uninitialized | State::MovedFrom | State::MutableBorrowed => {
                    Transition::Rejected
                }
            };
        }
        if action == Action::BorrowMut {
            return match state {
                State::Owned => Transition::Applied(State::MutableBorrowed),
                _ => Transition::Rejected,
            };
        }
        if action == Action::Drop {
            return match state {
                State::Uninitialized | State::Owned | State::MovedFrom => {
                    Transition::Applied(State::Uninitialized)
                }
                State::Shared { .. } | State::Frozen { .. } | State::MutableBorrowed => {
                    Transition::Rejected
                }
            };
        }
        // Action::EndBorrow — the pinned column: never Rej from a borrowed state.
        match state {
            State::Shared { loans } => {
                if loans >= 1 {
                    Transition::Applied(State::Shared { loans: loans - 1 })
                } else {
                    Transition::Applied(State::Owned)
                }
            }
            State::Frozen { loans } => {
                if loans >= 1 {
                    // Pinned cell: Frozen(n) + EndBorrow → Shared(n-1).
                    Transition::Applied(State::Shared { loans: loans - 1 })
                } else {
                    Transition::Applied(State::Owned)
                }
            }
            State::MutableBorrowed => Transition::Applied(State::Owned),
            State::Uninitialized | State::Owned | State::MovedFrom => Transition::Rejected,
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

    /// EXH certificate: every one of the 42 family cells of the pinned SPEC-INDEX §5.4 table,
    /// enumerated against the literal pinned values. The four unparameterized rows contribute
    /// 28 exact cells; the Shared/Frozen rows are enumerated at the loan representatives
    /// {0, 1, 65534, 65535} (56 further concrete cells; 84 total here, with the full n-domain
    /// covered by exh_trans_full_domain_131076x7).
    #[test]
    fn exh_trans_pinned_cells_42_families() {
        // Row Uninitialized (pinned: Rej | Owned | Rej | Rej | Rej | Uninit | Rej).
        assert_eq!(trans(State::Uninitialized, Action::Read), Transition::Rejected);
        assert_eq!(trans(State::Uninitialized, Action::Write), Transition::Applied(State::Owned));
        assert_eq!(trans(State::Uninitialized, Action::MoveOut), Transition::Rejected);
        assert_eq!(trans(State::Uninitialized, Action::BorrowShared), Transition::Rejected);
        assert_eq!(trans(State::Uninitialized, Action::BorrowMut), Transition::Rejected);
        assert_eq!(trans(State::Uninitialized, Action::Drop), Transition::Applied(State::Uninitialized));
        assert_eq!(trans(State::Uninitialized, Action::EndBorrow), Transition::Rejected);
        // Row Owned (pinned: Owned | Owned | MovedFrom | Shared(1) | MutableBorrowed | Uninit | Rej).
        assert_eq!(trans(State::Owned, Action::Read), Transition::Applied(State::Owned));
        assert_eq!(trans(State::Owned, Action::Write), Transition::Applied(State::Owned));
        assert_eq!(trans(State::Owned, Action::MoveOut), Transition::Applied(State::MovedFrom));
        assert_eq!(
            trans(State::Owned, Action::BorrowShared),
            Transition::Applied(State::Shared { loans: INITIAL_SHARED_LOANS })
        );
        assert_eq!(trans(State::Owned, Action::BorrowMut), Transition::Applied(State::MutableBorrowed));
        assert_eq!(trans(State::Owned, Action::Drop), Transition::Applied(State::Uninitialized));
        assert_eq!(trans(State::Owned, Action::EndBorrow), Transition::Rejected);
        // Row MovedFrom (pinned: Rej | Owned | Rej | Rej | Rej | Uninit | Rej).
        assert_eq!(trans(State::MovedFrom, Action::Read), Transition::Rejected);
        assert_eq!(trans(State::MovedFrom, Action::Write), Transition::Applied(State::Owned));
        assert_eq!(trans(State::MovedFrom, Action::MoveOut), Transition::Rejected);
        assert_eq!(trans(State::MovedFrom, Action::BorrowShared), Transition::Rejected);
        assert_eq!(trans(State::MovedFrom, Action::BorrowMut), Transition::Rejected);
        assert_eq!(trans(State::MovedFrom, Action::Drop), Transition::Applied(State::Uninitialized));
        assert_eq!(trans(State::MovedFrom, Action::EndBorrow), Transition::Rejected);
        // Row MutableBorrowed (pinned: Rej | Rej | Rej | Rej | Rej | Rej | Owned).
        assert_eq!(trans(State::MutableBorrowed, Action::Read), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::Write), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::MoveOut), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::BorrowShared), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::BorrowMut), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::Drop), Transition::Rejected);
        assert_eq!(trans(State::MutableBorrowed, Action::EndBorrow), Transition::Applied(State::Owned));
        // Rows Shared(n) and Frozen(n) at every pinned loan boundary representative.
        for n in LOAN_BOUNDARY_REPS {
            // Shared(n) (pinned: Shared(n) | Rej | Rej | Shared(n+1) if n<65535 else Rej | Rej | Rej | Shared(n-1) if n>=1 else Owned).
            assert_eq!(trans(State::Shared { loans: n }, Action::Read), Transition::Applied(State::Shared { loans: n }), "Shared({n}) Read");
            assert_eq!(trans(State::Shared { loans: n }, Action::Write), Transition::Rejected, "Shared({n}) Write");
            assert_eq!(trans(State::Shared { loans: n }, Action::MoveOut), Transition::Rejected, "Shared({n}) MoveOut");
            let want_bs = if n < MAX_LOANS {
                Transition::Applied(State::Shared { loans: n + 1 })
            } else {
                Transition::Rejected
            };
            assert_eq!(trans(State::Shared { loans: n }, Action::BorrowShared), want_bs, "Shared({n}) BorrowShared");
            assert_eq!(trans(State::Shared { loans: n }, Action::BorrowMut), Transition::Rejected, "Shared({n}) BorrowMut");
            assert_eq!(trans(State::Shared { loans: n }, Action::Drop), Transition::Rejected, "Shared({n}) Drop");
            let want_eb = if n >= 1 {
                Transition::Applied(State::Shared { loans: n - 1 })
            } else {
                Transition::Applied(State::Owned)
            };
            assert_eq!(trans(State::Shared { loans: n }, Action::EndBorrow), want_eb, "Shared({n}) EndBorrow");
            // Frozen(n) (pinned: Frozen(n) | Rej | Rej | Frozen(n+1) if n<65535 else Rej | Rej | Rej | Shared(n-1) if n>=1 else Owned).
            assert_eq!(trans(State::Frozen { loans: n }, Action::Read), Transition::Applied(State::Frozen { loans: n }), "Frozen({n}) Read");
            assert_eq!(trans(State::Frozen { loans: n }, Action::Write), Transition::Rejected, "Frozen({n}) Write");
            assert_eq!(trans(State::Frozen { loans: n }, Action::MoveOut), Transition::Rejected, "Frozen({n}) MoveOut");
            let want_bs = if n < MAX_LOANS {
                Transition::Applied(State::Frozen { loans: n + 1 })
            } else {
                Transition::Rejected
            };
            assert_eq!(trans(State::Frozen { loans: n }, Action::BorrowShared), want_bs, "Frozen({n}) BorrowShared");
            assert_eq!(trans(State::Frozen { loans: n }, Action::BorrowMut), Transition::Rejected, "Frozen({n}) BorrowMut");
            assert_eq!(trans(State::Frozen { loans: n }, Action::Drop), Transition::Rejected, "Frozen({n}) Drop");
            let want_eb = if n >= 1 {
                Transition::Applied(State::Shared { loans: n - 1 })
            } else {
                Transition::Applied(State::Owned)
            };
            assert_eq!(trans(State::Frozen { loans: n }, Action::EndBorrow), want_eb, "Frozen({n}) EndBorrow");
        }
    }

    /// EXH certificate (master): the COMPLETE concrete domain of the pinned function —
    /// 131,076 states × 7 actions = 917,532 cells — equals the independent action-first
    /// transcription cell-for-cell; per cell additionally: permits(s, a) ⟺ !rejected,
    /// conflict_at(s, a) ⟺ rejected (OB-06-6 conflictAt), totality (every cell defined,
    /// OB-06-1), and the loan-range law (every Applied successor has loan_count() <= MAX_LOANS,
    /// OB-06-4's second clause). Domain: full stated domain, test-as-proof.
    #[test]
    fn exh_trans_full_domain_131076x7() {
        let mut i = 0u32;
        while i < CONCRETE_STATE_COUNT {
            let s = concrete_state(i);
            for &a in &ACTIONS {
                let got = trans(s, a);
                let want = reference_trans(s, a);
                assert_eq!(got, want, "trans cell s={s:?} a={a:?}");
                assert!(matches!(
                    got,
                    Transition::Applied(_) | Transition::Rejected
                ), "cell must be a defined codomain value (totality, OB-06-1)");
                assert_eq!(permits(s, a), !got.is_rejected(), "permits cell s={s:?} a={a:?}");
                assert_eq!(conflict_at(s, a), got.is_rejected(), "conflict_at cell s={s:?} a={a:?}");
                if let Transition::Applied(next) = got {
                    assert!(
                        next.loan_count() <= MAX_LOANS,
                        "loan-range law violated at s={s:?} a={a:?} -> {next:?}"
                    );
                }
            }
            i += 1;
        }
    }

    /// EXH certificate (OB-06-2, OB-06-3): EndBorrow over the full u16 loan domain for both
    /// counter families (2 × 65,536 cells) plus MutableBorrowed and the three non-borrowed
    /// states: EndBorrow never yields Rej from a borrowed state; Shared(n≥1)/Frozen(n≥1) +
    /// EndBorrow → Shared(n−1) exactly as pinned; Shared(0)/Frozen(0) + EndBorrow → Owned
    /// (normalization law); MutableBorrowed + EndBorrow → Owned; EndBorrow is rejected from
    /// every non-borrowed state; and is_borrowed(s) ⟺ permits(s, EndBorrow) over the whole
    /// concrete domain (the borrowed states are exactly the EndBorrow-permitting ones).
    #[test]
    fn exh_endborrow_law_full_u16_sweep() {
        let mut n = 0u32;
        while n <= MAX_LOANS as u32 {
            let loans = n as u16;
            let s = State::Shared { loans };
            let got = trans(s, Action::EndBorrow);
            let want = if loans >= 1 {
                Transition::Applied(State::Shared { loans: loans - 1 })
            } else {
                Transition::Applied(State::Owned)
            };
            assert_eq!(got, want, "Shared({loans}) EndBorrow");
            assert!(!got.is_rejected(), "EndBorrow must never reject from borrowed Shared({loans})");
            assert!(s.is_borrowed() && permits(s, Action::EndBorrow));
            let s = State::Frozen { loans };
            let got = trans(s, Action::EndBorrow);
            assert_eq!(got, want, "Frozen({loans}) EndBorrow");
            assert!(!got.is_rejected(), "EndBorrow must never reject from borrowed Frozen({loans})");
            assert!(s.is_borrowed() && permits(s, Action::EndBorrow));
            n += 1;
        }
        // Normalization law, pinned exactly: Shared(0)/Frozen(0) + EndBorrow → Owned.
        assert_eq!(trans(State::Shared { loans: 0 }, Action::EndBorrow), Transition::Applied(State::Owned));
        assert_eq!(trans(State::Frozen { loans: 0 }, Action::EndBorrow), Transition::Applied(State::Owned));
        // MutableBorrowed is borrowed and EndBorrow restores Owned.
        assert!(State::MutableBorrowed.is_borrowed());
        assert_eq!(trans(State::MutableBorrowed, Action::EndBorrow), Transition::Applied(State::Owned));
        // Non-borrowed states reject EndBorrow (completes the is_borrowed ⟺ EB-permitted law).
        for s in [State::Uninitialized, State::Owned, State::MovedFrom] {
            assert!(!s.is_borrowed());
            assert_eq!(trans(s, Action::EndBorrow), Transition::Rejected);
            assert!(!permits(s, Action::EndBorrow));
        }
    }

    /// EXH certificate (OB-06-4): the counter law over the full loan domain — for all
    /// n ∈ 0..=65535, trans(Shared(n), BorrowShared) is Shared(n+1) iff n < 65535, else Rej;
    /// same for Frozen; and no result ever carries a counter above MAX_LOANS or below 0.
    #[test]
    fn exh_loan_counter_law_full_sweep() {
        let mut n = 0u32;
        while n <= MAX_LOANS as u32 {
            let loans = n as u16;
            let want = if loans < MAX_LOANS {
                Transition::Applied(State::Shared { loans: loans + 1 })
            } else {
                Transition::Rejected
            };
            assert_eq!(trans(State::Shared { loans }, Action::BorrowShared), want, "Shared({loans}) BorrowShared");
            let want = if loans < MAX_LOANS {
                Transition::Applied(State::Frozen { loans: loans + 1 })
            } else {
                Transition::Rejected
            };
            assert_eq!(trans(State::Frozen { loans }, Action::BorrowShared), want, "Frozen({loans}) BorrowShared");
            n += 1;
        }
        // No transition anywhere in the full domain produces a counter outside 0..=65535.
        let mut i = 0u32;
        while i < CONCRETE_STATE_COUNT {
            let s = concrete_state(i);
            for &a in &ACTIONS {
                if let Transition::Applied(next) = trans(s, a) {
                    assert!(next.loan_count() <= MAX_LOANS, "counter above MAX at s={s:?} a={a:?}");
                }
            }
            i += 1;
        }
    }

    /// EXH certificate: the pinned loan saturation boundaries n = 65534 and 65535, exactly.
    /// BorrowShared at 65534 lands on exactly 65535 (both families); BorrowShared at 65535
    /// is the pinned Rejection (P06-E020 regime); EndBorrow at 65535 decrements to exactly
    /// 65534 (Frozen's pinned successor Shared(65534) included); Read at 65535 is permitted
    /// and preserves the count exactly.
    #[test]
    fn exh_loan_saturation_boundaries_65534_65535() {
        // n = MAX_LOANS - 1 = 65534: the last non-saturating BorrowShared.
        assert_eq!(
            trans(State::Shared { loans: MAX_LOANS - 1 }, Action::BorrowShared),
            Transition::Applied(State::Shared { loans: MAX_LOANS })
        );
        assert_eq!(
            trans(State::Frozen { loans: MAX_LOANS - 1 }, Action::BorrowShared),
            Transition::Applied(State::Frozen { loans: MAX_LOANS })
        );
        // n = MAX_LOANS = 65535: BorrowShared is rejected (saturation), everything else pinned.
        assert_eq!(trans(State::Shared { loans: MAX_LOANS }, Action::BorrowShared), Transition::Rejected);
        assert_eq!(trans(State::Frozen { loans: MAX_LOANS }, Action::BorrowShared), Transition::Rejected);
        assert_eq!(
            trans(State::Shared { loans: MAX_LOANS }, Action::Read),
            Transition::Applied(State::Shared { loans: MAX_LOANS })
        );
        assert_eq!(
            trans(State::Frozen { loans: MAX_LOANS }, Action::Read),
            Transition::Applied(State::Frozen { loans: MAX_LOANS })
        );
        // EndBorrow at the boundary decrements exactly one step, to 65534.
        assert_eq!(
            trans(State::Shared { loans: MAX_LOANS }, Action::EndBorrow),
            Transition::Applied(State::Shared { loans: MAX_LOANS - 1 })
        );
        assert_eq!(
            trans(State::Frozen { loans: MAX_LOANS }, Action::EndBorrow),
            Transition::Applied(State::Shared { loans: MAX_LOANS - 1 })
        );
        // The pair is mutually consistent: saturation undoes the previous increment exactly.
        let stepped = trans(State::Shared { loans: MAX_LOANS - 1 }, Action::BorrowShared);
        assert_eq!(stepped, Transition::Applied(State::Shared { loans: MAX_LOANS }));
        assert_eq!(
            trans(stepped.state().unwrap(), Action::EndBorrow),
            Transition::Applied(State::Shared { loans: MAX_LOANS - 1 })
        );
    }

    /// EXH certificate (OB-06-5): behavioral identity of Shared and Frozen over the full
    /// domain — for all n ∈ 0..=65535 and all 7 actions (65,536 × 7 × 2 = 917,504 comparisons):
    /// (i) classification identity — the two rows are permitted or rejected identically on
    /// every action; (ii) successor identity modulo the frozen flag — applied successors are
    /// equal after erasing the flag, exactly Volume 06 §6.1.4's "they differ only in recorded
    /// history". FLAGGED INCONSISTENCY (found during extraction): OB-06-5's literal equation
    /// trans(Shared(n), a) = trans(Frozen(n), a) contradicts the pinned table's own Read and
    /// BorrowShared cells, which preserve the row kind (Shared(n)→Shared(n) but
    /// Frozen(n)→Frozen(n)); the literal equation therefore cannot be asserted, and the
    /// flag-erasure form is the exact machine-checkable content of §6.1.4.
    #[test]
    fn exh_shared_frozen_identity_full_sweep() {
        // Flag erasure: Frozen(k) viewed as Shared(k) — the shared-borrowed family without
        // the recorded mutable-borrow-attempt history (Volume 06 §6.1.4).
        const fn erase_frozen(s: State) -> State {
            match s {
                State::Frozen { loans } => State::Shared { loans },
                other => other,
            }
        }
        let mut n = 0u32;
        while n <= MAX_LOANS as u32 {
            let loans = n as u16;
            for &a in &ACTIONS {
                let ls = trans(State::Shared { loans }, a);
                let lf = trans(State::Frozen { loans }, a);
                assert_eq!(ls.is_rejected(), lf.is_rejected(), "classification identity Shared({loans}) vs Frozen({loans}) action {a:?}");
                match (ls, lf) {
                    (Transition::Applied(xs), Transition::Applied(xf)) => {
                        assert_eq!(erase_frozen(xf), xs, "successor identity modulo flag Shared({loans}) vs Frozen({loans}) action {a:?}");
                    }
                    _ => assert!(ls.is_rejected() && lf.is_rejected()),
                }
            }
            n += 1;
        }
    }

    /// EXH certificate: the pinned conflict-predicate consistency over 6 states × 7 × 7 = 294
    /// action pairs (family representatives, both closed forms checked per triple):
    /// conflict(s, a, b) ⟺ trans(s,a) = Rej ∨ trans(s,b) = Rej ⟺ ¬(permits(s,a) ∧ permits(s,b));
    /// symmetry conflict(s,a,b) = conflict(s,b,a); and the true sequential-composition law:
    /// if applying a then b succeeds AND b then a succeeds from s, then both actions are
    /// individually permitted in s (so ¬conflict(s,a,b)). The saturation regime (n = 65535
    /// for both counter families) adds 98 further triples.
    #[test]
    fn exh_conflict_in_state_294_triples() {
        let mut checked = 0u32;
        for &s in &FAMILY_REPRESENTATIVES {
            for &a in &ACTIONS {
                for &b in &ACTIONS {
                    let got = conflict(s, a, b);
                    let want = trans(s, a).is_rejected() || trans(s, b).is_rejected();
                    assert_eq!(got, want, "conflict triple s={s:?} a={a:?} b={b:?}");
                    assert_eq!(got, !(permits(s, a) && permits(s, b)), "conflict vs permits s={s:?} a={a:?} b={b:?}");
                    assert_eq!(conflict(s, b, a), got, "conflict symmetry s={s:?} a={a:?} b={b:?}");
                    // True sequential law: both orders succeed ⟹ ¬conflict in s.
                    if let (Some(sa), Some(sb)) = (trans(s, a).state(), trans(s, b).state()) {
                        let ab = trans(sa, b);
                        let ba = trans(sb, a);
                        if !ab.is_rejected() && !ba.is_rejected() {
                            assert!(!got, "sequential witness contradicts conflict at s={s:?} a={a:?} b={b:?}");
                        }
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 294, "the pinned 294-triple domain must be enumerated exactly");
        // Saturation regime: the same 49 pairs at Shared(65535) and Frozen(65535), where
        // BorrowShared joins the rejection columns.
        for s in [State::Shared { loans: MAX_LOANS }, State::Frozen { loans: MAX_LOANS }] {
            for &a in &ACTIONS {
                for &b in &ACTIONS {
                    let got = conflict(s, a, b);
                    let want = trans(s, a).is_rejected() || trans(s, b).is_rejected();
                    assert_eq!(got, want, "saturation conflict triple s={s:?} a={a:?} b={b:?}");
                    assert_eq!(got, !(permits(s, a) && permits(s, b)));
                }
            }
        }
    }

    /// EXH certificate (OB-06-6, notion 2): all 49 ordered action pairs satisfy
    /// incompatible(a, b) ⟺ permits(a) ∩ permits(b) = ∅, verified against the CONCRETE
    /// definition (no state among all 131,076 permits both — full-domain witness sweep),
    /// plus symmetry, irreflexivity, and the machine reconciliation of Volume 06 §6.1.3's
    /// prose list: the four agreeing entries hold; the three extra entries
    /// (MoveOut, MoveOut), (MoveOut, BorrowMut), (BorrowMut, BorrowMut) are rejected, with
    /// the exact joint-permit witness state Owned asserted for each (the flagged inconsistency
    /// documented on `incompatible`).
    #[test]
    fn exh_incompatible_49_pairs() {
        for &a in &ACTIONS {
            for &b in &ACTIONS {
                let got = incompatible(a, b);
                // Concrete definition: no concrete state permits both actions.
                let mut witness_exists = false;
                let mut i = 0u32;
                while i < CONCRETE_STATE_COUNT {
                    let s = concrete_state(i);
                    if permits(s, a) && permits(s, b) {
                        witness_exists = true;
                        break;
                    }
                    i += 1;
                }
                assert_eq!(got, !witness_exists, "incompatible({a:?}, {b:?}) vs concrete witness sweep");
                assert_eq!(incompatible(b, a), got, "incompatible symmetry {a:?}/{b:?}");
            }
        }
        // Irreflexivity: every pinned action is permitted somewhere, so no pair (a, a) is incompatible.
        for &a in &ACTIONS {
            assert!(!incompatible(a, a), "incompatible({a:?}, {a:?}) must be false");
        }
        // Volume 06 §6.1.3 prose entries that agree with the definitional formula.
        assert!(incompatible(Action::Write, Action::EndBorrow));
        assert!(incompatible(Action::MoveOut, Action::EndBorrow));
        assert!(incompatible(Action::BorrowMut, Action::EndBorrow));
        assert!(incompatible(Action::Drop, Action::EndBorrow));
        // The three flagged extra entries: the definitional formula (and the concrete sweep
        // above) rejects them because both actions are individually permitted in Owned.
        for (a, b) in [
            (Action::MoveOut, Action::MoveOut),
            (Action::MoveOut, Action::BorrowMut),
            (Action::BorrowMut, Action::BorrowMut),
        ] {
            assert!(!incompatible(a, b), "flagged §6.1.3 extra entry ({a:?}, {b:?}) violates the formula");
            assert!(permits(State::Owned, a) && permits(State::Owned, b), "Owned joint-permit witness for ({a:?}, {b:?})");
        }
    }

    /// EXH certificate: the permit FAMILY sets observed over the complete concrete domain
    /// (each of the 7 actions swept against all 131,076 states) equal both (a) the named
    /// PERMIT_FAMILY_MASK and (b) Volume 06 §6.1.3's permits table — with the pinned
    /// saturation carve-out that BorrowShared is not permitted at Shared(65535)/Frozen(65535),
    /// which the family-level table of §6.1.3 omits ("for all n") and the pinned table wins
    /// (flagged on PERMIT_FAMILY_MASK and in the extraction report).
    #[test]
    fn exh_permits_family_sets_131076() {
        let mut observed = [0u8; ACTION_COUNT];
        let mut i = 0u32;
        while i < CONCRETE_STATE_COUNT {
            let s = concrete_state(i);
            let bit = 1u8 << s.kind();
            for &a in &ACTIONS {
                if permits(s, a) {
                    observed[a.index()] |= bit;
                }
            }
            i += 1;
        }
        for &a in &ACTIONS {
            assert_eq!(observed[a.index()], PERMIT_FAMILY_MASK[a.index()], "permit family mask for {a:?}");
        }
        // Volume 06 §6.1.3 permits table, family-level, with the pinned carve-out applied:
        // BorrowShared's family set holds for all n EXCEPT n = 65535 (pinned Rej).
        let owned_shared_frozen = (1u8 << KIND_OWNED) | (1u8 << KIND_SHARED) | (1u8 << KIND_FROZEN);
        let uninit_owned_moved = (1u8 << KIND_UNINITIALIZED) | (1u8 << KIND_OWNED) | (1u8 << KIND_MOVED_FROM);
        let shared_frozen_mutable = (1u8 << KIND_SHARED) | (1u8 << KIND_FROZEN) | (1u8 << KIND_MUTABLE_BORROWED);
        assert_eq!(observed[ACTION_INDEX_READ], owned_shared_frozen);
        assert_eq!(observed[ACTION_INDEX_WRITE], uninit_owned_moved);
        assert_eq!(observed[ACTION_INDEX_MOVE_OUT], 1u8 << KIND_OWNED);
        assert_eq!(observed[ACTION_INDEX_BORROW_SHARED], owned_shared_frozen);
        assert_eq!(observed[ACTION_INDEX_BORROW_MUT], 1u8 << KIND_OWNED);
        assert_eq!(observed[ACTION_INDEX_DROP], uninit_owned_moved);
        assert_eq!(observed[ACTION_INDEX_END_BORROW], shared_frozen_mutable);
        // The carve-out, stated exactly: BS is permitted in family Shared/Frozen at n = 0
        // (so the family mask is right) but rejected exactly at n = 65535 (pinned table).
        assert!(permits(State::Shared { loans: 0 }, Action::BorrowShared));
        assert!(permits(State::Frozen { loans: 0 }, Action::BorrowShared));
        assert!(!permits(State::Shared { loans: MAX_LOANS }, Action::BorrowShared));
        assert!(!permits(State::Frozen { loans: MAX_LOANS }, Action::BorrowShared));
    }

    /// EXH certificate: the complete domain of the crate's only fallible function.
    /// (a) Full kind domain 0..=255 × boundary loans {0, 1, 65534, 65535} = 1,024 cases:
    /// kinds 0..=5 produce the exact pinned states (counter-free kinds only with loans = 0),
    /// kinds 6..=255 produce Err(InvalidStateKind(k)) exactly. (b) The full valid loan
    /// domain 0..=65535 for both counter kinds = 131,072 cases: every value is Ok and
    /// round-trips through kind()/loan_count(). (c) Round-trips for all six kinds.
    #[test]
    fn exh_state_from_parts_full_domain() {
        let boundary_loans = [0u16, 1, MAX_LOANS - 1, MAX_LOANS];
        let mut kind = 0u32;
        while kind <= u8::MAX as u32 {
            let k = kind as u8;
            for &loans in &boundary_loans {
                let got = state_from_parts(k, loans);
                match k {
                    KIND_UNINITIALIZED => {
                        let want = if loans == 0 {
                            Ok(State::Uninitialized)
                        } else {
                            Err(StateError::StrayLoanCount { kind: k, loans })
                        };
                        assert_eq!(got, want, "kind {k} loans {loans}");
                    }
                    KIND_OWNED => {
                        let want = if loans == 0 {
                            Ok(State::Owned)
                        } else {
                            Err(StateError::StrayLoanCount { kind: k, loans })
                        };
                        assert_eq!(got, want, "kind {k} loans {loans}");
                    }
                    KIND_MOVED_FROM => {
                        let want = if loans == 0 {
                            Ok(State::MovedFrom)
                        } else {
                            Err(StateError::StrayLoanCount { kind: k, loans })
                        };
                        assert_eq!(got, want, "kind {k} loans {loans}");
                    }
                    KIND_SHARED => assert_eq!(got, Ok(State::Shared { loans }), "kind {k} loans {loans}"),
                    KIND_MUTABLE_BORROWED => {
                        let want = if loans == 0 {
                            Ok(State::MutableBorrowed)
                        } else {
                            Err(StateError::StrayLoanCount { kind: k, loans })
                        };
                        assert_eq!(got, want, "kind {k} loans {loans}");
                    }
                    KIND_FROZEN => assert_eq!(got, Ok(State::Frozen { loans }), "kind {k} loans {loans}"),
                    _ => assert_eq!(got, Err(StateError::InvalidStateKind(k)), "kind {k} loans {loans}"),
                }
            }
            kind += 1;
        }
        // Full valid loan domain for both counter kinds: every u16 value constructs and round-trips.
        let mut loans = 0u32;
        while loans <= MAX_LOANS as u32 {
            let l = loans as u16;
            let s = state_from_parts(KIND_SHARED, l).unwrap();
            assert_eq!(s, State::Shared { loans: l });
            assert_eq!(s.kind(), KIND_SHARED);
            assert_eq!(s.loan_count(), l);
            assert_eq!(state_from_parts(s.kind(), s.loan_count()), Ok(s));
            let s = state_from_parts(KIND_FROZEN, l).unwrap();
            assert_eq!(s, State::Frozen { loans: l });
            assert_eq!(s.kind(), KIND_FROZEN);
            assert_eq!(s.loan_count(), l);
            assert_eq!(state_from_parts(s.kind(), s.loan_count()), Ok(s));
            loans += 1;
        }
        // Counter-free kinds carry loan_count() == 0 and round-trip.
        for s in [State::Uninitialized, State::Owned, State::MovedFrom, State::MutableBorrowed] {
            assert_eq!(s.loan_count(), 0);
            assert_eq!(state_from_parts(s.kind(), 0), Ok(s));
            assert_eq!(state_from_parts(s.kind(), 1), Err(StateError::StrayLoanCount { kind: s.kind(), loans: 1 }));
        }
    }

    /// PROP certificate (paired with the EXH master sweep): 10,000 deterministic PRNG samples
    /// of (state, action) satisfy trans == reference transcription, conflict_at ⟺ rejected,
    /// and is_borrowed ⟺ EndBorrow-permitted. Sampling domain: kind uniform over 0..=5 with
    /// loan counts uniform over 0..=65535 biased toward the four pinned boundary values.
    #[test]
    fn prop_random_trans_matches_reference() {
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < 10_000 {
            let kind = rng.below(STATE_KIND_COUNT as u64) as u8;
            let loans = if kind == KIND_SHARED || kind == KIND_FROZEN {
                if rng.below(4) == 0 {
                    LOAN_BOUNDARY_REPS[rng.below(LOAN_BOUNDARY_REPS.len() as u64) as usize]
                } else {
                    rng.below(u16::MAX as u64 + 1) as u16
                }
            } else {
                0
            };
            let s = state_from_parts(kind, loans).unwrap();
            let a = ACTIONS[rng.below(ACTION_COUNT as u64) as usize];
            assert_eq!(trans(s, a), reference_trans(s, a), "sample {n}: s={s:?} a={a:?}");
            assert_eq!(conflict_at(s, a), trans(s, a).is_rejected());
            assert_eq!(s.is_borrowed(), permits(s, Action::EndBorrow));
            n += 1;
        }
    }
}
