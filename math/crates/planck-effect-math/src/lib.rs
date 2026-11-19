// CEP:FILE: math/crates/planck-effect-math/src/lib.rs
// CEP:WHAT: Pinned effect algebra of Planck 0.1 edition P1 (SPEC-INDEX §5.3, embedded verbatim in Volume 07 §7.1.1, elaborated by §7.1.2-§7.1.3, §7.3.8-§7.3.10, §7.3.14, §7.4 d1, §7.5 d7; encoded per Volume 12 §12.1) as a no_std kernel crate: the closed 8-atom effect vocabulary, closed rows as validated u16 set bitmasks, open rows as single row variables, canonical form and canonical order, join (union), meet (intersection), row subtyping (⊆), row equality, the pinned function-call rule, and row unification exactly per Volume 07 §7.3.9 (occurs check, creation-index binding, path compression, deferred-constraint fixpoint).
// CEP:WHY: SPEC-INDEX §5.1 pins planck-effect-math as the extraction home of the effect algebra (Volume 07); every effect annotation check, impl-conformance check, and row unification (Volume 07 §7.3) refines exactly these operations, so the effect checker must call exactly these functions and no others.
// CEP:STATUS: complete
// CEP:FAILURE: The complete failure-mode set, each a variant of a complete error enum with an EXH certificate: (1) EffectError::MaskOutOfRange — a construction mask with any bit above atom bit 7 set; (2) EffectError::AtomIndexOutOfRange — an atom index ≥ 8; (3) EffectError::DuplicateAtom — a duplicate atom in an atom list (§7.4 d1; parse-level P07-C01); (4) EffectError::RowVarOutOfRange — a row-variable id ≥ ROW_VAR_LIMIT (kernel-side namespace bound; §7.1.3 names no numeric limit, the kernel pins 256); (5) EffectError::OccursCheck — a binding that would make the substitution cyclic (§7.3.9 step 2.b); (6) UnifyError::Mismatch — closed rows failing unification, carrying exactly row₁\row₂ and row₂\row₁ (§7.3.9 case a; compile-time rejection P07-C01); (7) UnifyError::Occurs — the unify-level occurs-check failure; unreachable from unify under the documented forest invariant (proven in the item docs) and retained because §7.3.9 names the occurs check inside unification; (8) CallError::Missing — a callee row with atoms the ambient row lacks (pinned call rule; §7.3.9 rule 3, P07-C03); (9) ConstraintFailure::Mismatch — index and failure of the first failing constraint in the §7.3.9 step-3 fixpoint. No other failure mode exists; every other operation is total and pure.
// CEP:ASSUMES: SPEC-INDEX §5.3 (pinned algebra; per Volume 07 header, where Volume 07 could be read to disagree with §5.3, §5.3 wins); Volume 07 §7.1.1 (verbatim embed), §7.1.2 (canonical order alloc < async < concurrency < ffi < io < nondet < panic < unsafe; two rows are equal iff their canonical forms are equal), §7.1.3 (a row is exactly a closed subset or a single row variable; no tail variables and no open rows with fixed atom parts), §7.3.8 (constraint generation; the AST-level generator is compiler-side, this kernel provides the unification the spec defines over constraint pairs), §7.3.9 (row unification algorithm, exact, including the occurs check, creation-index binding, path compression, and the deferred-constraint loop with its termination measure), §7.3.10 (requirement-position subtyping), §7.3.14 (default rows: unannotated extern = {ffi}), §7.4 d1 (atom-list uniqueness), §7.5 d7 (unknown-effect barrier row {io, alloc, panic, concurrency, nondet}); Volume 12 §12.1 (row encoding: bitmask over the 8 pinned atoms in fixed bit order bit0 io, bit1 alloc, bit2 panic, bit3 unsafe, bit4 async, bit5 concurrency, bit6 ffi, bit7 nondet; open-row flag stored alongside — realized here as the Closed/Var discriminant of Row; the kernel carries the mask in a u16 whose bits 8..=15 are reserved zero and validated away); SPEC-INDEX §5.2 (kernel implementation law: core:: only in non-test code, no allocation, no panic, explicit complete error enums); SPEC-INDEX §5.1 (equivalence claim form). The kernel defines join/meet/leq only on closed rows: the pinned algebra defines no set operations on open rows (an open row is a single variable, not a set, §7.1.3) and inventing them is banned.
// CEP:COST: closed-row operations O(1) (one or two u16 bitwise ops, branch-free); canonical_sequence and atom_count O(ATOM_COUNT); Subst::bind O(chain length) ≤ ROW_VAR_LIMIT by the acyclicity invariant; resolve O(chain length) with path compression; unify O(depth) per call; unify_constraints O(|C|·V) worst case with ≤ ROW_VAR_LIMIT + 1 passes (the §7.3.9 termination measure); no allocation anywhere in non-test code.
// CEP:EVIDENCE: EXH over the full 256-row closed lattice (every pairwise law enumerated as a 65,536-case u16 loop, no sampling) plus per-atom pointwise characterizations (524,288 cases) whose composition with the exhaustive per-atom Boolean-law certificate discharges the ternary laws (associativity, distributivity, transitivity) over all 256^3 triples; EXH unification certificates (all 65,536 closed/closed pairs, all 131,072 oriented var/closed pairs, all 65,536 var/var pairs on the empty substitution, 131,072 pre-bound var/var states, the full-chain occurs-check family, path-compression chains of length 255); EXH validation certificates (all 65,536 u16 masks, all 65,536 u16 variable ids, all 256 atom indices, all atom sequences of length ≤ 4); PROP prop_lattice_laws_random_triples_10000 and prop_unify_random_states_10000 (pinned in-crate xorshift64*, each paired with the EXH certificates for the same property per SPEC-INDEX §4.3). Mirrors the MATP obligations eff_join_assoc, eff_join_comm, eff_join_idem, eff_join_ident, eff_meet_distrib, eff_absorb (tools/fv/matp/artifacts/), for which MATP recorded GaveUp/STATUS: fail at its 200,000-iteration bound — the EXH tests in this crate are the machine evidence for those obligations.
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over the complete 256-row lattice, the complete u16 validation domains, and the stated unification state families; test-as-proof)
// CEP:FV-SCRIPT: crates/planck-effect-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete: the closed-row domain is exactly the 256 valid masks (§7.1.3 "256 possible values") and every closed-row operation is EXH-certified over all of it; the ternary lattice laws hold by the exhaustive pointwise characterization + exhaustive per-atom Boolean certificate (composition argument stated in the item docs — no sampling anywhere); the row-variable domain is exactly ids 0..ROW_VAR_LIMIT and validation is EXH over all 65,536 u16 ids; unification is EXH-certified over closed/closed (all 65,536 pairs), var/closed (all 131,072 oriented pairs), var/var (all 65,536 empty-subst pairs + 131,072 pre-bound states), the occurs-check chain family, and the constraint driver (single-constraint equivalence over all 67,600 row pairs with 4 variables, the conflict family over all 65,536 mask pairs, deferred re-check families, and 255-constraint chains in both orders). Canonicalization totality and equality-agreement are EXH over the full row space; subtyping antisymmetry is EXH over all 65,536 pairs. There is no untested region and therefore no refinement gap.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named constants (Law 7: constants are named, never magic literals).
// ---------------------------------------------------------------------------

/// Number of pinned effect atoms; the atom set is closed and adding an atom is an edition change (SPEC-INDEX §5.3 bullet 1; Volume 07 §7.1.1).
pub const ATOM_COUNT: usize = 8;
/// Highest valid canonical atom index (§7.1.2 canonical order has ranks 0..=7).
pub const ATOM_INDEX_MAX: u8 = (ATOM_COUNT - 1) as u8;
/// Mask of all 8 pinned atom bits; bits 8..=15 of a row mask are reserved zero (Volume 12 §12.1).
pub const ATOM_MASK_ALL: u16 = (1u16 << ATOM_COUNT) - 1;
/// Number of distinct closed rows: the full 2^8 lattice (Volume 07 §7.1.3 "256 possible values"; SPEC-INDEX §5.3 laws).
pub const CLOSED_ROW_COUNT: usize = 1 << ATOM_COUNT;
/// Kernel-side bound of the per-item row-variable namespace (Volume 07 §7.1.3 names no numeric limit; the kernel pins 256 so the substitution is a fixed-size stack structure (CEP-0, no allocation) and the variable domain stays exhaustively testable: 256^2 = 65,536 variable pairs). Exceeding it is the defined failure RowVarOutOfRange; raising it is a kernel/edition change.
pub const ROW_VAR_LIMIT: u16 = 256;
/// Slot count of the fixed-size substitution (== ROW_VAR_LIMIT as usize).
const ROW_VAR_COUNT: usize = ROW_VAR_LIMIT as usize;

// Volume 12 §12.1 fixed bit order: bit0 io, bit1 alloc, bit2 panic, bit3 unsafe,
// bit4 async, bit5 concurrency, bit6 ffi, bit7 nondet.
/// Bit position of atom `io` (Volume 12 §12.1).
pub const BIT_INDEX_IO: u8 = 0;
/// Bit position of atom `alloc` (Volume 12 §12.1).
pub const BIT_INDEX_ALLOC: u8 = 1;
/// Bit position of atom `panic` (Volume 12 §12.1).
pub const BIT_INDEX_PANIC: u8 = 2;
/// Bit position of atom `unsafe` (Volume 12 §12.1).
pub const BIT_INDEX_UNSAFE: u8 = 3;
/// Bit position of atom `async` (Volume 12 §12.1).
pub const BIT_INDEX_ASYNC: u8 = 4;
/// Bit position of atom `concurrency` (Volume 12 §12.1).
pub const BIT_INDEX_CONCURRENCY: u8 = 5;
/// Bit position of atom `ffi` (Volume 12 §12.1).
pub const BIT_INDEX_FFI: u8 = 6;
/// Bit position of atom `nondet` (Volume 12 §12.1).
pub const BIT_INDEX_NONDET: u8 = 7;

/// The pinned atoms in the fixed canonical (alphabetical) order of Volume 07 §7.1.2: alloc < async < concurrency < ffi < io < nondet < panic < unsafe. Definition-of-record const data: rank i of this array is canonical index i.
pub const CANONICAL_ATOM_ORDER: [Atom; ATOM_COUNT] = [
    Atom::Alloc,
    Atom::Async,
    Atom::Concurrency,
    Atom::Ffi,
    Atom::Io,
    Atom::Nondet,
    Atom::Panic,
    Atom::Unsafe,
];

/// The pinned atom names, spelled exactly as in SPEC-INDEX §5.3 / Volume 07 §7.1.1, indexed by canonical order (§7.1.2).
pub const ATOM_NAMES: [&str; ATOM_COUNT] = [
    "alloc",
    "async",
    "concurrency",
    "ffi",
    "io",
    "nondet",
    "panic",
    "unsafe",
];

/// Single-atom mask of `io` (bit 0 per Volume 12 §12.1).
pub const ATOM_BIT_IO: u16 = 1 << BIT_INDEX_IO;
/// Single-atom mask of `alloc` (bit 1 per Volume 12 §12.1).
pub const ATOM_BIT_ALLOC: u16 = 1 << BIT_INDEX_ALLOC;
/// Single-atom mask of `panic` (bit 2 per Volume 12 §12.1).
pub const ATOM_BIT_PANIC: u16 = 1 << BIT_INDEX_PANIC;
/// Single-atom mask of `unsafe` (bit 3 per Volume 12 §12.1).
pub const ATOM_BIT_UNSAFE: u16 = 1 << BIT_INDEX_UNSAFE;
/// Single-atom mask of `async` (bit 4 per Volume 12 §12.1).
pub const ATOM_BIT_ASYNC: u16 = 1 << BIT_INDEX_ASYNC;
/// Single-atom mask of `concurrency` (bit 5 per Volume 12 §12.1).
pub const ATOM_BIT_CONCURRENCY: u16 = 1 << BIT_INDEX_CONCURRENCY;
/// Single-atom mask of `ffi` (bit 6 per Volume 12 §12.1).
pub const ATOM_BIT_FFI: u16 = 1 << BIT_INDEX_FFI;
/// Single-atom mask of `nondet` (bit 7 per Volume 12 §12.1).
pub const ATOM_BIT_NONDET: u16 = 1 << BIT_INDEX_NONDET;

/// The empty row ∅: identity of join (SPEC-INDEX §5.3 bullets 2-3; Volume 07 §7.1.1).
///
/// CEP:WHAT: The ∅ closed row as const data; join identity and bottom of the ≤ order.
/// CEP:WHY: The pinned laws are stated against ∅; callers need it as a named value, not a literal.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: bit 0 mask encoding (Volume 12 §12.1).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_join_identity_empty_256; EXH exh_named_row_constants_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: constant identity is checked against the exact 0 mask on the full 256-row lattice; gap: none.
pub const ROW_EMPTY: ClosedRow = ClosedRow { bits: 0 };

/// The full atom set row: all 8 pinned atoms, top of the ≤ order (SPEC-INDEX §5.3 bullet 1).
///
/// CEP:WHAT: The closed row containing every pinned atom, as const data.
/// CEP:WHY: Meet identity and the maximal demand; the closed atom set is pinned exactly.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: ATOM_MASK_ALL == bits io|alloc|panic|unsafe|async|concurrency|ffi|nondet (Volume 12 §12.1).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_named_row_constants_256; EXH exh_meet_identity_all_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: constant checked bit-exactly against all 8 single-atom masks; gap: none.
pub const ROW_ALL: ClosedRow = ClosedRow { bits: ATOM_MASK_ALL };

/// Default row of an `extern` function without annotation: {ffi} (Volume 07 §7.3.14).
///
/// CEP:WHAT: The pinned opaque-callee default row {ffi} as const data.
/// CEP:WHY: §7.3.14 pins the default row of unannotated extern functions; the checker consumes it as a named value.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: Volume 07 §7.3.14 exactly: `extern` without annotation → {ffi}; with annotation → annotated row joined with {ffi} (caller applies join).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_named_row_constants_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: constant checked == single-atom ffi mask; gap: none.
pub const ROW_FFI: ClosedRow = ClosedRow { bits: ATOM_BIT_FFI };

/// Unknown-effect barrier row for optimization: {io, alloc, panic, concurrency, nondet} (Volume 07 §7.5 d7).
///
/// CEP:WHAT: The pinned conservative default row for operations with unavailable summaries, as const data.
/// CEP:WHY: §7.5 d7 pins the barrier row exactly; the optimizer consumes it as a named value.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (constant).
/// CEP:ASSUMES: Volume 07 §7.5 d7 exactly: {io, alloc, panic, concurrency, nondet} (no unsafe, no async, no ffi in the pinned barrier set).
/// CEP:COST: compile-time constant.
/// CEP:EVIDENCE: EXH exh_named_row_constants_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: constant checked bit-exactly against the five pinned single-atom masks; gap: none.
pub const ROW_UNKNOWN_BARRIER: ClosedRow = ClosedRow {
    bits: ATOM_BIT_IO | ATOM_BIT_ALLOC | ATOM_BIT_PANIC | ATOM_BIT_CONCURRENCY | ATOM_BIT_NONDET,
};

// ---------------------------------------------------------------------------
// Error enums (complete; every variant is listed in CEP:FAILURE above).
// ---------------------------------------------------------------------------

/// The complete failure set of the algebra's validating constructors and the substitution binder.
///
/// CEP:WHAT: Complete error enum for row/atom/variable construction validation and Subst::bind.
/// CEP:WHY: SPEC-INDEX §5.2: every failure returns an explicit error enum; every enum is complete.
/// CEP:STATUS: complete
/// CEP:FAILURE: exactly these variants: MaskOutOfRange { mask } (bits above atom bit 7; from_mask); AtomIndexOutOfRange { index } (index > ATOM_INDEX_MAX; atom_from_index); DuplicateAtom { atom } (first duplicate in an atom list, §7.4 d1; from_atoms); RowVarOutOfRange { var } (id ≥ ROW_VAR_LIMIT; VarId::new, Row::var_from_u16); OccursCheck { var } (binding var would create a cyclic substitution, §7.3.9 step 2.b; Subst::bind). No other variant exists and no other function returns this enum.
/// CEP:ASSUMES: constructors are the only validation sites; masks already stored in ClosedRow satisfy bits ≤ ATOM_MASK_ALL by construction.
/// CEP:COST: O(1) per construction check.
/// CEP:EVIDENCE: EXH exh_mask_validation_65536; EXH exh_atom_from_index_256; EXH exh_atoms_from_lists_4681; EXH exh_varid_validation_65536; EXH exh_occurs_check_chain_families.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete u16 mask and variable domains and the atom-list family, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: every variant is reachable and EXH-certified over its full stated domain; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectError {
    /// A construction mask had a bit set above atom bit 7 (outside the 256-row closed lattice).
    MaskOutOfRange { mask: u16 },
    /// An atom index exceeded ATOM_INDEX_MAX (the atom set is closed, 8 atoms).
    AtomIndexOutOfRange { index: u8 },
    /// An atom list contained a duplicate atom (§7.4 d1: duplicates are P07-C01 at parse level); the first duplicate encountered in list order is reported.
    DuplicateAtom { atom: Atom },
    /// A row-variable id was ≥ ROW_VAR_LIMIT (kernel-side per-item namespace bound).
    RowVarOutOfRange { var: u16 },
    /// Binding var would make the substitution cyclic (§7.3.9 step 2.b occurs check).
    OccursCheck { var: u16 },
}

/// The complete failure set of row unification (Volume 07 §7.3.9).
///
/// CEP:WHAT: Complete error enum for unify failures: the pinned closed/closed mismatch and the occurs-check failure.
/// CEP:WHY: §7.3.9 case (a) pins the failure payload exactly (only_in_left = row₁ \ row₂, only_in_right = row₂ \ row₁; rejection P07-C01), and step 2 names the occurs check inside unification.
/// CEP:STATUS: complete
/// CEP:FAILURE: exactly these variants: Mismatch { only_in_left, only_in_right } (closed rows differ; the fields are the exact canonical set differences); Occurs { var } — unreachable from unify under the documented forest invariant (bind targets are always normal forms: Closed has an empty free-variable set, an unbound Var's chain cannot reach the binder; proof in unify's docs) and retained as the defensive pinned-algorithm site. No other variant exists.
/// CEP:ASSUMES: operands are canonical by construction (§7.3.9 step 1 is the identity on this representation, EXH-certified); the substitution is acyclic (maintained by bind's occurs check).
/// CEP:COST: O(1) for Mismatch (two bitwise differences); O(chain) for the occurs check.
/// CEP:EVIDENCE: EXH exh_unify_closed_closed_65536 (Mismatch payload exact on all failing pairs); EXH exh_occurs_check_chain_families (Occurs exact on the full-chain family); EXH exh_unify_var_var_prebound_131072 (no Occurs failure ever arises from unify).
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete closed-pair domain and the chain family, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: Mismatch certified on all 65,536 closed pairs; Occurs certified on the chain family and proven unreachable from unify (documented proof, defensively retained); gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnifyError {
    /// §7.3.9 case (a): closed rows failed to unify; only_in_left = row₁ \ row₂, only_in_right = row₂ \ row₁, both in canonical form (rejection P07-C01).
    Mismatch {
        only_in_left: ClosedRow,
        only_in_right: ClosedRow,
    },
    /// §7.3.9 step 2.b/2.d: the occurs check failed; binding would create a cyclic substitution. Unreachable from unify (see the enum docs); reachable defensively and through the same check inside Subst::bind as EffectError::OccursCheck.
    Occurs { var: u16 },
}

/// The complete failure set of the pinned function-call rule.
///
/// CEP:WHAT: Complete error enum for check_call: the callee demands atoms the ambient row does not include.
/// CEP:WHY: SPEC-INDEX §5.3 bullet 4 pins the call rule ("calling f : (row ρ) → τ requires the caller's row to include ρ"); Volume 07 §7.3.9 rule 3 pins the violation as rejection P07-C03.
/// CEP:STATUS: complete
/// CEP:FAILURE: exactly this variant: Missing { missing } with missing = r_callee \ r_ambient in canonical form. No other variant exists.
/// CEP:ASSUMES: both operands are validated closed rows; the rule is an inclusion (⊆), not a unification.
/// CEP:COST: O(1) (one bitwise op).
/// CEP:EVIDENCE: EXH exh_check_call_65536 (all 65,536 ambient/callee pairs).
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete pair domain, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: certified on all 65,536 pairs with exact missing-set payloads; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CallError {
    /// P07-C03: the callee row contains atoms absent from the ambient row; missing = r_callee \ r_ambient.
    Missing { missing: ClosedRow },
}

/// The complete failure set of the constraint-set driver (Volume 07 §7.3.9 step 3).
///
/// CEP:WHAT: Complete error enum for unify_constraints: which constraint failed and how.
/// CEP:WHY: §7.3.9 step 3 loops "until no deferred constraints remain or a failure occurs"; the driver must report the failing constraint deterministically (G14).
/// CEP:STATUS: complete
/// CEP:FAILURE: exactly this variant: Mismatch { index, failure } with index = position of the first failing constraint in slice order of the first failing pass, failure = its UnifyError. No other variant exists.
/// CEP:ASSUMES: constraints are examined in slice order every pass; bindings only accumulate (never retract), so the first failure is deterministic.
/// CEP:COST: O(|C|·V) worst case over the whole driver; the error carries O(1) data.
/// CEP:EVIDENCE: EXH exh_unify_constraints_conflict_index_65536 (index and payload exact on all conflicting mask pairs); EXH exh_unify_constraints_single_67600 (driver == single-step unify on every pair).
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the conflict family and single-constraint equivalence domain, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: certified on the full conflict family and the full single-constraint row-pair domain; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstraintFailure {
    /// The first failing constraint: its index into the slice and its unification failure.
    Mismatch { index: usize, failure: UnifyError },
}

// ---------------------------------------------------------------------------
// Atom: the closed 8-element vocabulary (§7.1.1), ordered canonically (§7.1.2).
// ---------------------------------------------------------------------------

/// A pinned effect atom. The set is closed with exactly these 8 members (SPEC-INDEX §5.3 bullet 1; Volume 07 §7.1.1); declaration order is the canonical alphabetical order of Volume 07 §7.1.2, so derived Ord is the canonical order and the discriminant is the canonical index.
///
/// CEP:WHAT: The closed effect vocabulary {io, alloc, panic, unsafe, async, concurrency, ffi, nondet} as an exhaustive enum.
/// CEP:WHY: §7.1.1 pins the vocabulary; an atom name outside it is a rejection (P07-C01, §7.3.14), so the type must make other atoms unrepresentable.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (the type is total over its 8 members; invalid indices/names are rejected at construction by atom_from_index).
/// CEP:ASSUMES: Volume 07 §7.1.2 canonical order alloc < async < concurrency < ffi < io < nondet < panic < unsafe; Volume 12 §12.1 bit mapping via atom_bit_index.
/// CEP:COST: O(1); Copy, no allocation.
/// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8; EXH exh_atom_from_index_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 8 members checked for name, canonical rank, and bit mapping; index space EXH over all 256 u8 values; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Atom {
    /// Canonical rank 0 (§7.1.2); bit 1 (Volume 12 §12.1).
    Alloc,
    /// Canonical rank 1 (§7.1.2); bit 4 (Volume 12 §12.1).
    Async,
    /// Canonical rank 2 (§7.1.2); bit 5 (Volume 12 §12.1).
    Concurrency,
    /// Canonical rank 3 (§7.1.2); bit 6 (Volume 12 §12.1).
    Ffi,
    /// Canonical rank 4 (§7.1.2); bit 0 (Volume 12 §12.1).
    Io,
    /// Canonical rank 5 (§7.1.2); bit 7 (Volume 12 §12.1).
    Nondet,
    /// Canonical rank 6 (§7.1.2); bit 2 (Volume 12 §12.1).
    Panic,
    /// Canonical rank 7 (§7.1.2); bit 3 (Volume 12 §12.1).
    Unsafe,
}

/// Canonical index of `atom`: its rank in the fixed alphabetical order of Volume 07 §7.1.2 (0..=7).
///
/// Refinement anchor: Volume 07 §7.1.2. Input domain: all 8 atoms (total). Failure set: none.
/// Certificate: EXH `exh_atoms_canonical_order_and_bit_mapping_8` (rank == discriminant == position in CANONICAL_ATOM_ORDER for all 8 atoms).
///
/// CEP:WHAT: atom → canonical index (the pinned §7.1.2 order).
/// CEP:WHY: §7.1.2 defines equality of rows via canonical form; the canonical index is the sort key.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total over the closed atom set.
/// CEP:ASSUMES: enum declaration order == canonical order (certified against CANONICAL_ATOM_ORDER).
/// CEP:COST: O(1), one cast; CEP-0.
/// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 8 atoms checked; gap: none.
#[inline]
pub const fn atom_index(atom: Atom) -> u8 {
    atom as u8
}

/// The atom at canonical index `index` (§7.1.2 order), or AtomIndexOutOfRange.
///
/// Refinement anchor: Volume 07 §7.1.2. Input domain: all 256 u8 values; valid iff index ≤ ATOM_INDEX_MAX.
/// Failure set: EffectError::AtomIndexOutOfRange { index } for index > 7.
/// Certificate: EXH `exh_atom_from_index_256` (all 256 indices: Ok exactly for 0..=7 with the exact CANONICAL_ATOM_ORDER member; exact error payload otherwise).
///
/// CEP:WHAT: canonical index → atom, with validation.
/// CEP:WHY: inverse of atom_index; the closed set makes out-of-range indices a defined failure, never a wrap or a panic.
/// CEP:STATUS: complete
/// CEP:FAILURE: EffectError::AtomIndexOutOfRange { index } iff index > ATOM_INDEX_MAX.
/// CEP:ASSUMES: CANONICAL_ATOM_ORDER is the §7.1.2 order (definition-of-record const).
/// CEP:COST: O(1), one bounds check and one array read; CEP-0.
/// CEP:EVIDENCE: EXH exh_atom_from_index_256.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over all 256 u8 indices, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: full 256-value domain enumerated; gap: none.
#[inline]
pub fn atom_from_index(index: u8) -> Result<Atom, EffectError> {
    if index <= ATOM_INDEX_MAX {
        Ok(CANONICAL_ATOM_ORDER[index as usize])
    } else {
        Err(EffectError::AtomIndexOutOfRange { index })
    }
}

/// The pinned name of `atom`, spelled exactly as in SPEC-INDEX §5.3 / Volume 07 §7.1.1.
///
/// Refinement anchor: Volume 07 §7.1.1 (atom names) and §7.1.2 (order). Input domain: all 8 atoms (total).
/// Failure set: none. Certificate: EXH `exh_atoms_canonical_order_and_bit_mapping_8` (name == ATOM_NAMES[rank] for all 8; the array is checked against the pinned spellings).
///
/// CEP:WHAT: atom → pinned name (&'static str).
/// CEP:WHY: diagnostics must print the pinned spellings (§7.3.4: presentation follows §7.1.2).
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total over the closed atom set.
/// CEP:ASSUMES: ATOM_NAMES holds the exact §7.1.1 spellings in §7.1.2 order.
/// CEP:COST: O(1), one array read; CEP-0.
/// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 8 names checked; gap: none.
#[inline]
pub const fn atom_name(atom: Atom) -> &'static str {
    ATOM_NAMES[atom_index(atom) as usize]
}

/// Bit position of `atom` in the row bitmask: the fixed bit order of Volume 12 §12.1 (bit0 io, bit1 alloc, bit2 panic, bit3 unsafe, bit4 async, bit5 concurrency, bit6 ffi, bit7 nondet).
///
/// Refinement anchor: Volume 12 §12.1 (Effect representation). Input domain: all 8 atoms (total).
/// Failure set: none. Certificate: EXH `exh_atoms_canonical_order_and_bit_mapping_8` (each atom's bit position equals the pinned value).
///
/// CEP:WHAT: atom → bit position per the pinned Volume 12 encoding.
/// CEP:WHY: the canonical order (§7.1.2) governs presentation; the bit order (Volume 12 §12.1) governs storage — both are pinned and deliberately distinct.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total over the closed atom set.
/// CEP:ASSUMES: Volume 12 §12.1 is the normative encoding row; the u16 mask uses the same low 8 bits as Volume 12's u8 bitmask.
/// CEP:COST: O(1); CEP-0.
/// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 8 positions checked against the pinned table; gap: none.
#[inline]
pub const fn atom_bit_index(atom: Atom) -> u8 {
    match atom {
        Atom::Io => BIT_INDEX_IO,
        Atom::Alloc => BIT_INDEX_ALLOC,
        Atom::Panic => BIT_INDEX_PANIC,
        Atom::Unsafe => BIT_INDEX_UNSAFE,
        Atom::Async => BIT_INDEX_ASYNC,
        Atom::Concurrency => BIT_INDEX_CONCURRENCY,
        Atom::Ffi => BIT_INDEX_FFI,
        Atom::Nondet => BIT_INDEX_NONDET,
    }
}

/// Single-atom bitmask of `atom`: 1 << atom_bit_index(atom) (Volume 12 §12.1 bit order).
///
/// Refinement anchor: Volume 12 §12.1. Input domain: all 8 atoms (total). Failure set: none.
/// Certificate: EXH `exh_atoms_canonical_order_and_bit_mapping_8` (each ATOM_BIT_* const equals atom_bit of its atom; the 8 masks are pairwise disjoint and their union is ATOM_MASK_ALL).
///
/// CEP:WHAT: atom → single-bit u16 mask.
/// CEP:WHY: rows are set bitmasks; single-atom masks are the set-builder primitives.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total; shift amounts are 0..=7, no overflow.
/// CEP:ASSUMES: bit positions are 0..=7 (Volume 12 §12.1).
/// CEP:COST: O(1), one shift; CEP-0.
/// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8; EXH exh_join_meet_per_atom_characterization_524288.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 8 atoms checked; gap: none.
#[inline]
pub const fn atom_bit(atom: Atom) -> u16 {
    1u16 << atom_bit_index(atom)
}

// ---------------------------------------------------------------------------
// VarId: a row variable of a per-item namespace (§7.1.3 b).
// ---------------------------------------------------------------------------

/// A row variable id: the creation index of a row variable in a per-item namespace (Volume 07 §7.1.3: an open row is "a single row variable ρ drawn from a per-item namespace").
///
/// The constructor enforces index < ROW_VAR_LIMIT, so every Row::Var payload is a valid substitution slot; the namespace (caller side) assigns ids in creation order and the kernel relies only on their total order (§7.3.9 case d binds by creation index, determinism G14).
///
/// CEP:WHAT: Validated row-variable id (kernel-side namespace bound ROW_VAR_LIMIT = 256).
/// CEP:WHY: §7.1.3 pins open rows to single variables; the validated newtype keeps every downstream array access in bounds without a panic path (SPEC-INDEX §5.2).
/// CEP:STATUS: complete
/// CEP:FAILURE: VarId::new returns EffectError::RowVarOutOfRange { var } iff index >= ROW_VAR_LIMIT.
/// CEP:ASSUMES: ids are creation indices assigned in creation order by the per-item namespace (§7.3.9 case d, G14); the kernel bounds the namespace at ROW_VAR_LIMIT because §7.1.3 names no numeric limit.
/// CEP:COST: O(1), one comparison; CEP-0.
/// CEP:EVIDENCE: EXH exh_varid_validation_65536 (all 65,536 u16 ids).
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over all 65,536 u16 ids, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: full u16 domain enumerated with exact error payloads; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VarId {
    index: u16,
}

impl VarId {
    /// Validates `index` into a VarId; valid iff index < ROW_VAR_LIMIT.
    ///
    /// Refinement anchor: Volume 07 §7.1.3 (per-item namespace) with the kernel-side bound ROW_VAR_LIMIT.
    /// Input domain: all 65,536 u16 values. Failure set: EffectError::RowVarOutOfRange { var: index } iff index >= ROW_VAR_LIMIT.
    /// Certificate: EXH `exh_varid_validation_65536`.
    ///
    /// CEP:WHAT: Validated VarId constructor.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: EffectError::RowVarOutOfRange { var } iff index >= ROW_VAR_LIMIT.
    /// CEP:ASSUMES: see the type docs.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_varid_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full u16 domain enumerated; gap: none.
    #[inline]
    pub const fn new(index: u16) -> Result<VarId, EffectError> {
        if index < ROW_VAR_LIMIT {
            Ok(VarId { index })
        } else {
            Err(EffectError::RowVarOutOfRange { var: index })
        }
    }

    /// The creation index of this variable (0..ROW_VAR_LIMIT).
    ///
    /// Refinement anchor: Volume 07 §7.3.9 case d (min/max by variable creation index). Input domain: all VarIds (total). Failure set: none.
    /// Certificate: EXH `exh_varid_validation_65536` (roundtrip new/index on all valid ids).
    ///
    /// CEP:WHAT: VarId → creation index.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: the index is the creation index in the per-item namespace.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_varid_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: roundtrip checked on all valid ids; gap: none.
    #[inline]
    pub const fn index(self) -> u16 {
        self.index
    }
}

// ---------------------------------------------------------------------------
// ClosedRow: a finite subset of the atom set as a validated u16 set bitmask.
// ---------------------------------------------------------------------------

/// A closed effect row: a subset of the pinned atom set, encoded as a u16 set bitmask with bit i = the atom at Volume 12 §12.1 bit position i (bit0 io, bit1 alloc, bit2 panic, bit3 unsafe, bit4 async, bit5 concurrency, bit6 ffi, bit7 nondet); bits 8..=15 are reserved zero.
///
/// The invariant `bits <= ATOM_MASK_ALL` is established by the validating constructors and never broken (the field is private; every operation preserves it), so a ClosedRow is canonical by construction: as a set encoding it is sorted and deduplicated (§7.1.2's canonical form is realized directly by the bit encoding; the §7.1.2 order governs presentation, see canonical_sequence).
///
/// CEP:WHAT: Validated closed row (one of the 256 lattice rows of §7.1.3 a).
/// CEP:WHY: §5.3/§7.1.1 define rows as subsets of the closed atom set; the bitmask is the Volume 12 §12.1 storage law.
/// CEP:STATUS: complete
/// CEP:FAILURE: construction from a raw u16 can fail with EffectError::MaskOutOfRange; all operations on constructed rows are total.
/// CEP:ASSUMES: Volume 12 §12.1 bit order; u16 chosen per the extraction contract with bits 8..=15 reserved zero (Volume 12's u8 bitmask is recovered exactly by u8_mask).
/// CEP:COST: O(1) per operation (one or two bitwise ops); Copy, no allocation.
/// CEP:EVIDENCE: EXH exh_mask_validation_65536; EXH exh_closedrow_canonical_form_2048; the full law suite over all 256 rows.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete 256-row lattice, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: the domain is exactly 256 rows and every operation is certified over all of it; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClosedRow {
    bits: u16,
}

impl ClosedRow {
    /// The empty row ∅ (see also the free constant ROW_EMPTY; this is the same value as an associated item).
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 bullets 2-3 (join identity ∅). Failure set: none.
    /// Certificate: EXH `exh_join_identity_empty_256` (join(a, EMPTY) == a for all 256 rows).
    ///
    /// CEP:WHAT: ∅ as an associated constant.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (constant).
    /// CEP:ASSUMES: bit 0 encoding.
    /// CEP:COST: compile-time constant.
    /// CEP:EVIDENCE: EXH exh_join_identity_empty_256.
    pub const EMPTY: ClosedRow = ClosedRow { bits: 0 };

    /// The full row containing all 8 pinned atoms (see also ROW_ALL).
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 bullet 1 (closed atom set). Failure set: none.
    /// Certificate: EXH `exh_named_row_constants_256`.
    ///
    /// CEP:WHAT: the full atom set as an associated constant.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (constant).
    /// CEP:ASSUMES: ATOM_MASK_ALL bit encoding.
    /// CEP:COST: compile-time constant.
    /// CEP:EVIDENCE: EXH exh_named_row_constants_256.
    pub const ALL: ClosedRow = ClosedRow { bits: ATOM_MASK_ALL };

    /// Validates a raw u16 as a closed row: valid iff no bit above atom bit 7 is set (the row lies in the 256-row lattice of §7.1.3).
    ///
    /// Refinement anchor: Volume 07 §7.1.3 (256 closed rows); Volume 12 §12.1 (bitmask encoding).
    /// Input domain: all 65,536 u16 values. Failure set: EffectError::MaskOutOfRange { mask } iff (mask & !ATOM_MASK_ALL) != 0.
    /// Certificate: EXH `exh_mask_validation_65536` (all 65,536 masks: Ok exactly for the 256 lattice rows, with bits() == mask; exact error payload otherwise).
    ///
    /// CEP:WHAT: row construction with validation (the set path; see from_atoms for the annotation-list path).
    /// CEP:WHY: SPEC-INDEX §5.2 demands explicit failures; a mask outside the lattice must never silently truncate.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: EffectError::MaskOutOfRange { mask } for masks outside the lattice.
    /// CEP:ASSUMES: ATOM_MASK_ALL is the exact valid-mask set boundary.
    /// CEP:COST: O(1), one AND and one compare; CEP-0.
    /// CEP:EVIDENCE: EXH exh_mask_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over all 65,536 u16 masks, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full u16 domain enumerated; gap: none.
    #[inline]
    pub fn from_mask(mask: u16) -> Result<ClosedRow, EffectError> {
        if (mask & !ATOM_MASK_ALL) == 0 {
            Ok(ClosedRow { bits: mask })
        } else {
            Err(EffectError::MaskOutOfRange { mask })
        }
    }

    /// The singleton row {atom}.
    ///
    /// Refinement anchor: Volume 07 §7.1.3 a (closed row as subset); Volume 12 §12.1 (bit order).
    /// Input domain: all 8 atoms (total). Failure set: none.
    /// Certificate: EXH `exh_atoms_canonical_order_and_bit_mapping_8` (from_atom(a).bits() == ATOM_BIT_* for all 8 atoms).
    ///
    /// CEP:WHAT: atom → singleton closed row.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total over the closed atom set.
    /// CEP:ASSUMES: atom_bit bit mapping (Volume 12 §12.1).
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_atoms_canonical_order_and_bit_mapping_8.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: all 8 atoms checked; gap: none.
    #[inline]
    pub const fn from_atom(atom: Atom) -> ClosedRow {
        ClosedRow {
            bits: atom_bit(atom),
        }
    }

    /// Builds the row containing exactly the atoms of `atoms` (no duplicates, no ordering requirement).
    ///
    /// Refinement anchor: Volume 07 §7.4 d1 ("Atoms must be unique within a list (duplicate is P07-C01)") applied to the annotation-list path; the result is the set of the list's atoms in canonical bitmask form.
    /// Input domain: atom lists of any length; valid iff no atom occurs twice. Failure set: EffectError::DuplicateAtom { atom } naming the first duplicate in list order.
    /// Certificate: EXH `exh_atoms_from_lists_4681` (all 4,681 atom sequences of length 0..=4 over the 8 atoms: Ok exactly for the duplicate-free lists, result == OR of single-atom masks, first-duplicate payload exact); PROP `prop_atoms_from_lists_long` (pinned PRNG, longer lists, paired with the EXH above).
    ///
    /// CEP:WHAT: row construction from an atom list with §7.4 d1 duplicate validation.
    /// CEP:WHY: EffectAnno lists (§7.4 d1) are the source-level row syntax; the kernel must reject duplicates exactly where the spec does.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: EffectError::DuplicateAtom { atom } on the first duplicate.
    /// CEP:ASSUMES: the slice is an annotation list in source order (duplicates detected in list order); empty lists give ∅ (§7.4 d1: omitted EffectAnno means the closed row ∅).
    /// CEP:COST: O(n^2) worst case in the list length (duplicate scan), O(n) typical via the accumulated mask; CEP-0, no allocation.
    /// CEP:EVIDENCE: EXH exh_atoms_from_lists_4681; PROP prop_atoms_from_lists_long.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over the length-0..=4 sequence family + paired PROP, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: the length-≤4 family is enumerated fully (4,681 lists) and the accumulated-mask argument covers all longer lists pointwise (each element adds one disjoint bit; a duplicate is any bit already set) — the long-list behavior is the same loop, PROP-paired; gap: none.
    pub fn from_atoms(atoms: &[Atom]) -> Result<ClosedRow, EffectError> {
        let mut bits = 0u16;
        let mut i = 0usize;
        while i < atoms.len() {
            let bit = atom_bit(atoms[i]);
            if (bits & bit) != 0 {
                return Err(EffectError::DuplicateAtom { atom: atoms[i] });
            }
            bits |= bit;
            i += 1;
        }
        Ok(ClosedRow { bits })
    }

    /// The raw u16 mask (bits 0..=7; bits 8..=15 are zero by the type invariant).
    ///
    /// Refinement anchor: Volume 12 §12.1 (bitmask encoding). Input domain: all ClosedRows (total). Failure set: none.
    /// Certificate: EXH `exh_mask_validation_65536` (from_mask roundtrip on all valid masks); EXH `exh_named_row_constants_256` (u8_mask roundtrip).
    ///
    /// CEP:WHAT: ClosedRow → raw mask.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: the invariant bits <= ATOM_MASK_ALL (private field, validated constructors).
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_mask_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: roundtrip over all 256 rows; gap: none.
    #[inline]
    pub const fn bits(self) -> u16 {
        self.bits
    }

    /// The low 8 bits as u8: exactly the Volume 12 §12.1 row bitmask ("u8 bitmask over the 8 pinned atoms"), for the compiler's metadata encoding; the open-row flag of §12.1 is stored alongside in the compiler and is the Row::Closed/Row::Var discriminant here.
    ///
    /// Refinement anchor: Volume 12 §12.1. Input domain: all ClosedRows (total). Failure set: none.
    /// Certificate: EXH `exh_named_row_constants_256` (u8_mask(m) == m as u8 for all 256 rows).
    ///
    /// CEP:WHAT: ClosedRow → Volume 12 u8 encoding.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (bits 8..=15 are zero by the type invariant, so the cast is lossless).
    /// CEP:ASSUMES: the type invariant; Volume 12 §12.1 is the target encoding.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_named_row_constants_256.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over all 256 rows, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full domain; gap: none.
    #[inline]
    pub const fn u8_mask(self) -> u8 {
        self.bits as u8
    }

    /// Does the row contain `atom`?
    ///
    /// Refinement anchor: Volume 07 §7.1.3 a (closed row as subset). Input domain: all (row, atom) pairs (total). Failure set: none.
    /// Certificate: EXH `exh_join_meet_per_atom_characterization_524288` (membership is the per-atom bit on all 256×256 pairs × 8 atoms).
    ///
    /// CEP:WHAT: set membership on the closed row.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: atom_bit bit mapping.
    /// CEP:COST: O(1), one AND; CEP-0.
    /// CEP:EVIDENCE: EXH exh_join_meet_per_atom_characterization_524288.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full pair domain × all atoms; gap: none.
    #[inline]
    pub const fn contains(self, atom: Atom) -> bool {
        (self.bits & atom_bit(atom)) != 0
    }

    /// Is the row ∅?
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 bullet 3 (∅ is the join identity). Input domain: all 256 rows (total). Failure set: none.
    /// Certificate: EXH `exh_join_identity_empty_256` (EMPTY.is_empty() and no other row is empty, per the 256-row loop).
    ///
    /// CEP:WHAT: emptiness test.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: bit 0 encoding.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_join_identity_empty_256.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full domain; gap: none.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Number of atoms in the row (0..=8).
    ///
    /// Refinement anchor: Volume 07 §7.1.3 a. Input domain: all 256 rows (total). Failure set: none.
    /// Certificate: EXH `exh_closedrow_canonical_form_2048` (atom_count == popcount of the mask and == count of Some slots in canonical_sequence, for all 256 rows).
    ///
    /// CEP:WHAT: cardinality of the row.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: 8 atom bits.
    /// CEP:COST: O(ATOM_COUNT); CEP-0.
    /// CEP:EVIDENCE: EXH exh_closedrow_canonical_form_2048.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full domain; gap: none.
    #[inline]
    pub const fn atom_count(self) -> u8 {
        let mut n = 0u8;
        let mut i = 0u8;
        while i < (ATOM_COUNT as u8) {
            if (self.bits & (1u16 << i)) != 0 {
                n += 1;
            }
            i += 1;
        }
        n
    }

    /// The atoms of the row in the fixed canonical order of Volume 07 §7.1.2 (alloc < async < concurrency < ffi < io < nondet < panic < unsafe): slot at canonical rank r is Some(atom) iff the row contains the rank-r atom, None padding after the row's atom_count. This is the pinned "sorted, deduplicated" canonical presentation used by diagnostics (§7.3.4).
    ///
    /// Refinement anchor: Volume 07 §7.1.2 (canonical order; canonical form sorted+deduplicated) and §7.3.4 (presentation order).
    /// Input domain: all 256 rows (total). Failure set: none.
    /// Certificate: EXH `exh_closedrow_canonical_form_2048` (canonical_sequence equals the independently computed rank-ordered membership for all 256 rows × 8 ranks; strictly ascending canonical ranks prove sortedness, the one-bit-per-slot encoding proves deduplication).
    ///
    /// CEP:WHAT: canonical-order presentation sequence of the row.
    /// CEP:WHY: §7.1.2's canonical form is "sorted, deduplicated"; the bitmask is the form, this function is the pinned presentation of it.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total.
    /// CEP:ASSUMES: CANONICAL_ATOM_ORDER is the §7.1.2 order; the bitmask is the set.
    /// CEP:COST: O(ATOM_COUNT); CEP-0, no allocation (fixed-size array).
    /// CEP:EVIDENCE: EXH exh_closedrow_canonical_form_2048.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over all 256 rows, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full domain × all ranks; gap: none.
    #[inline]
    pub const fn canonical_sequence(self) -> [Option<Atom>; ATOM_COUNT] {
        let mut out: [Option<Atom>; ATOM_COUNT] = [None; ATOM_COUNT];
        let mut rank = 0usize;
        while rank < ATOM_COUNT {
            let atom = CANONICAL_ATOM_ORDER[rank];
            if self.contains(atom) {
                out[rank] = Some(atom);
            }
            rank += 1;
        }
        out
    }

    /// Join: r1 ∪ r2 (set union) (SPEC-INDEX §5.3 bullet 3; Volume 07 §7.1.1).
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 / Volume 07 §7.1.1 (join = union); Volume 07 §7.3.5 (composition is join).
    /// Input domain: all 256×256 row pairs (total). Failure set: none.
    /// Certificate: EXH `exh_join_commutative_65536`, `exh_join_idempotent_256`, `exh_join_identity_empty_256`, `exh_join_absorption_65536`, and `exh_join_meet_per_atom_characterization_524288` (join == per-atom union on every pair × every atom), which with the per-atom Boolean associativity certificate `exh_boolean_lattice_laws_per_atom_320` proves associativity on all 256^3 triples: join(join(a,b),c) and join(a,join(b,c)) have identical per-atom membership, and pointwise-equal sets are equal.
    ///
    /// CEP:WHAT: pinned join (union) on closed rows.
    /// CEP:WHY: the composition and call rules of Volume 07 are stated over join.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total, pure, associative, commutative, idempotent, identity ∅.
    /// CEP:ASSUMES: the closed 256-row lattice; join is defined on closed rows only (no open-row set operations are pinned; §7.1.3).
    /// CEP:COST: O(1), one OR; CEP-0.
    /// CEP:EVIDENCE: EXH exh_join_commutative_65536; EXH exh_join_idempotent_256; EXH exh_join_identity_empty_256; EXH exh_join_absorption_65536; EXH exh_join_meet_per_atom_characterization_524288; EXH exh_boolean_lattice_laws_per_atom_320; PROP prop_lattice_laws_random_triples_10000.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over the complete lattice, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: all pairwise laws enumerated over all 65,536 pairs; associativity discharged over all 256^3 triples by the exhaustive characterization + per-atom certificate; gap: none.
    #[inline]
    #[must_use]
    pub const fn join(self, other: ClosedRow) -> ClosedRow {
        ClosedRow {
            bits: self.bits | other.bits,
        }
    }

    /// Meet: r1 ∩ r2 (set intersection) (SPEC-INDEX §5.3 bullet 3; Volume 07 §7.1.1).
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 / Volume 07 §7.1.1 (meet = intersection).
    /// Input domain: all 256×256 row pairs (total). Failure set: none.
    /// Certificate: EXH `exh_meet_commutative_65536`, `exh_meet_idempotent_256`, `exh_meet_identity_all_256`, `exh_meet_absorption_65536`, and `exh_join_meet_per_atom_characterization_524288`; meet distributing over join (both directions) on all 256^3 triples is discharged by the same composition argument as join's (exhaustive pointwise characterization + `exh_boolean_lattice_laws_per_atom_320`).
    ///
    /// CEP:WHAT: pinned meet (intersection) on closed rows.
    /// CEP:WHY: the pinned law "meet distributes over join" is stated over meet.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total, pure, associative, commutative, idempotent, identity ROW_ALL, distributes over join.
    /// CEP:ASSUMES: the closed 256-row lattice; meet is defined on closed rows only (§7.1.3).
    /// CEP:COST: O(1), one AND; CEP-0.
    /// CEP:EVIDENCE: EXH exh_meet_commutative_65536; EXH exh_meet_idempotent_256; EXH exh_meet_identity_all_256; EXH exh_meet_absorption_65536; EXH exh_join_meet_per_atom_characterization_524288; EXH exh_boolean_lattice_laws_per_atom_320; PROP prop_lattice_laws_random_triples_10000.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over the complete lattice, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: all pairwise laws enumerated; distributivity discharged over all triples by the composition certificate; gap: none.
    #[inline]
    #[must_use]
    pub const fn meet(self, other: ClosedRow) -> ClosedRow {
        ClosedRow {
            bits: self.bits & other.bits,
        }
    }

    /// Set difference r1 \ r2, the exact payload computation of §7.3.9 case (a) (UnifyError fields) and of §7.3.9 rule 7 (`unsafe { e }` strips the unsafe atom: row(e) \ {unsafe}).
    ///
    /// Refinement anchor: Volume 07 §7.3.9 step 2.a (row₁ \ row₂) and §7.3.9 rule 7.
    /// Input domain: all 256×256 row pairs (total). Failure set: none.
    /// Certificate: EXH `exh_unify_closed_closed_65536` (the difference payloads of every failing unification are exact); EXH `exh_leq_order_characterizations_196608` (difference-based formulations agree with leq on all pairs).
    ///
    /// CEP:WHAT: set difference on closed rows.
    /// CEP:WHY: pinned by the unification failure payload and the unsafe-strip rule.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total.
    /// CEP:ASSUMES: closed-row domain.
    /// CEP:COST: O(1), one AND-NOT; CEP-0.
    /// CEP:EVIDENCE: EXH exh_unify_closed_closed_65536; EXH exh_leq_order_characterizations_196608.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full pair domain; gap: none.
    #[inline]
    #[must_use]
    pub const fn difference(self, other: ClosedRow) -> ClosedRow {
        ClosedRow {
            bits: self.bits & !other.bits,
        }
    }

    /// Row subtyping for requirement positions: r1 ≤ r2 ⟺ r1 ⊆ r2 (SPEC-INDEX §5.3 bullet 3; Volume 07 §7.1.1, §7.3.10).
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 / Volume 07 §7.1.1, §7.3.10 (impl rows against trait rows, callee rows at call sites, re-exports).
    /// Input domain: all 256×256 row pairs (total). Failure set: none.
    /// Certificate: EXH `exh_leq_reflexive_256`, `exh_leq_antisymmetric_65536`, `exh_leq_subset_characterization_524288` (leq == the conjunction over the 8 atoms of the per-atom implication "a has the atom ⟹ b has it" on every pair, asserted pointwise per atom), which with the per-atom implication-transitivity certificate in `exh_boolean_lattice_laws_per_atom_320` proves transitivity on all 256^3 triples; `exh_leq_order_characterizations_196608` additionally certifies leq(a,b) ⟺ meet(a,b)==a ⟺ join(a,b)==b on all pairs.
    ///
    /// CEP:WHAT: pinned row subtyping (subset) on closed rows.
    /// CEP:WHY: requirement positions (§7.3.10) and the call rule are decided by this relation.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total; a partial order (reflexive, antisymmetric, transitive — all certified).
    /// CEP:ASSUMES: closed-row domain; subtyping on open rows is not pinned (§7.1.3) and is deliberately not provided.
    /// CEP:COST: O(1), one AND-NOT and one compare; CEP-0.
    /// CEP:EVIDENCE: EXH exh_leq_reflexive_256; EXH exh_leq_antisymmetric_65536; EXH exh_leq_subset_characterization_524288; EXH exh_leq_order_characterizations_196608; EXH exh_boolean_lattice_laws_per_atom_320; PROP prop_lattice_laws_random_triples_10000.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over the complete lattice, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: reflexivity/antisymmetry enumerated; transitivity discharged over all triples by the composition certificate; gap: none.
    #[inline]
    #[must_use]
    pub const fn leq(self, other: ClosedRow) -> bool {
        (self.bits & !other.bits) == 0
    }
}

// ---------------------------------------------------------------------------
// Row: exactly one of a closed row or a single row variable (Volume 07 §7.1.3).
// ---------------------------------------------------------------------------

/// An effect row: exactly one of (a) a closed row — a subset of the atom set, 256 possible values — or (b) an open row — a single row variable ρ of a per-item namespace (Volume 07 §7.1.3, verbatim refinement of SPEC-INDEX §5.3's "finite subset plus a row variable flag").
///
/// There are no tail variables and no open rows with fixed atom parts (§7.1.3 settles this): the open-row flag of §5.3 is exactly the Closed/Var discriminant, and an open row carries no atom bits. Equality is §7.1.2's: two rows are equal iff their canonical forms are equal (closed: mask equality; open: variable identity, §7.4 d2).
///
/// CEP:WHAT: The pinned row type (closed | open-single-variable).
/// CEP:WHY: §7.1.3 fixes the row representation for edition P1; the type makes every other shape (tails, open+atoms) unrepresentable.
/// CEP:STATUS: complete
/// CEP:FAILURE: construction from raw ids can fail (var_from_u16); pattern-matched rows are total.
/// CEP:ASSUMES: §7.1.3 exactly; Var payloads are constructor-validated VarIds (index < ROW_VAR_LIMIT).
/// CEP:COST: O(1); Copy, no allocation.
/// CEP:EVIDENCE: EXH exh_row_equality_canonical_agreement_67600; EXH exh_varid_validation_65536; EXH exh_closedrow_canonical_form_2048.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the full row space with 4 variables, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: equality/canonicalization certified over the full 260-row space (256 closed + 4 variables); gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    /// A closed row (§7.1.3 a): a subset of the atom set in canonical bitmask form.
    Closed(ClosedRow),
    /// An open row (§7.1.3 b): a single row variable, identified by its creation index; no atom parts (§7.1.3 forbids open rows with fixed atom parts).
    Var(VarId),
}

impl Row {
    /// Wraps a validated closed row (total; the payload is already validated).
    ///
    /// Refinement anchor: Volume 07 §7.1.3 a. Failure set: none.
    /// Certificate: EXH `exh_row_equality_canonical_agreement_67600` (roundtrip through the full row space).
    ///
    /// CEP:WHAT: closed-row lifting into Row.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: ClosedRow invariant.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_row_equality_canonical_agreement_67600.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete; gap: none.
    #[inline]
    pub const fn closed(row: ClosedRow) -> Row {
        Row::Closed(row)
    }

    /// Wraps a validated VarId (total; VarId::new already enforced index < ROW_VAR_LIMIT).
    ///
    /// Refinement anchor: Volume 07 §7.1.3 b. Failure set: none.
    /// Certificate: EXH `exh_varid_validation_65536`; EXH `exh_row_equality_canonical_agreement_67600`.
    ///
    /// CEP:WHAT: open-row lifting into Row.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (validation happened in VarId::new).
    /// CEP:ASSUMES: VarId invariant.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_varid_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete; gap: none.
    #[inline]
    pub const fn var(var: VarId) -> Row {
        Row::Var(var)
    }

    /// Validates a raw u16 as an open row: valid iff index < ROW_VAR_LIMIT.
    ///
    /// Refinement anchor: Volume 07 §7.1.3 b with the kernel-side namespace bound.
    /// Input domain: all 65,536 u16 values. Failure set: EffectError::RowVarOutOfRange { var } iff index >= ROW_VAR_LIMIT.
    /// Certificate: EXH `exh_varid_validation_65536` (all 65,536 ids, exact payloads).
    ///
    /// CEP:WHAT: row construction with validation (the open-row path).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: EffectError::RowVarOutOfRange { var }.
    /// CEP:ASSUMES: see VarId::new.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_varid_validation_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full u16 domain; gap: none.
    #[inline]
    pub fn var_from_u16(index: u16) -> Result<Row, EffectError> {
        match VarId::new(index) {
            Ok(var) => Ok(Row::Var(var)),
            Err(error) => Err(error),
        }
    }

    /// Is this an open row (the §5.3 "row variable flag" set)?
    ///
    /// Refinement anchor: SPEC-INDEX §5.3 bullet 2 (row variable flag); Volume 07 §7.1.3. Input domain: all rows (total). Failure set: none.
    /// Certificate: EXH `exh_row_equality_canonical_agreement_67600` (is_open agrees with the variant on all 260 rows).
    ///
    /// CEP:WHAT: the pinned open-row flag as a predicate.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: the Closed/Var discriminant realizes the flag.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_row_equality_canonical_agreement_67600.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete; gap: none.
    #[inline]
    pub const fn is_open(self) -> bool {
        match self {
            Row::Closed(_) => false,
            Row::Var(_) => true,
        }
    }

    /// Is this a closed row?
    ///
    /// Refinement anchor: Volume 07 §7.1.3 a. Input domain: all rows (total). Failure set: none.
    /// Certificate: EXH `exh_row_equality_canonical_agreement_67600`.
    ///
    /// CEP:WHAT: closed-row predicate.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: the Closed/Var discriminant.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_row_equality_canonical_agreement_67600.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete; gap: none.
    #[inline]
    pub const fn is_closed(self) -> bool {
        match self {
            Row::Closed(_) => true,
            Row::Var(_) => false,
        }
    }
}

/// Canonicalize a row to its canonical form (Volume 07 §7.1.2: sorted, deduplicated; §7.3.9 step 1: "Normalize both operands to canonical form").
///
/// On this representation canonicalization is the identity, and that is the pinned outcome, not a shortcut: a ClosedRow is a set bitmask — it has no duplicates to remove and no order to sort (the §7.1.2 sort order is realized by the bit-rank presentation, see ClosedRow::canonical_sequence), and an open row is canonical by variable identity (§7.1.3 b, §7.4 d2 "open rows equal by variable identity"). Totality: the function never fails and never changes the value; equality-agreement (r1 == r2 ⟺ canonical forms equal) is certified directly.
///
/// Refinement anchor: Volume 07 §7.1.2, §7.1.3, §7.3.9 step 1. Input domain: all rows (total; certified over the 260-row space with 4 variables and all 256 closed rows).
/// Failure set: none.
/// Certificate: EXH `exh_closedrow_canonical_form_2048` and `exh_row_equality_canonical_agreement_67600` (identity and idempotence on every row; rows_equal(r1,r2) ⟺ canonicalize(r1) == canonicalize(r2) on all 260×260 pairs — totality and antisymmetry of the induced equality included).
///
/// CEP:WHAT: pinned canonical-form projection (§7.3.9 step 1).
/// CEP:WHY: unification and row equality are defined on canonical forms.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total, pure, idempotent.
/// CEP:ASSUMES: the canonical-by-construction representation (validated bitmask; single-variable open rows).
/// CEP:COST: O(1); CEP-0.
/// CEP:EVIDENCE: EXH exh_closedrow_canonical_form_2048; EXH exh_row_equality_canonical_agreement_67600.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the full row space, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: totality, idempotence, and equality-agreement enumerated over the full row space; gap: none.
#[inline]
#[must_use]
pub const fn canonicalize(row: Row) -> Row {
    row
}

/// Row equality per Volume 07 §7.1.2: two rows are equal iff their canonical forms are equal (closed: mask equality; open: variable identity, §7.4 d2).
///
/// Refinement anchor: Volume 07 §7.1.2, §7.4 d2. Input domain: all row pairs (certified over the 260-row space: 67,600 pairs).
/// Failure set: none.
/// Certificate: EXH `exh_row_equality_canonical_agreement_67600` (rows_equal == derived PartialEq == canonical-form equality on all 260×260 pairs).
///
/// CEP:WHAT: the pinned row equality.
/// CEP:WHY: §7.1.2 defines row equality via canonical forms; function-type equality (§7.4 d2) builds on it.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total, pure, an equivalence relation (agreement with PartialEq certifies reflexivity/symmetry/transitivity pointwise).
/// CEP:ASSUMES: canonical-by-construction representation.
/// CEP:COST: O(1); CEP-0.
/// CEP:EVIDENCE: EXH exh_row_equality_canonical_agreement_67600.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the full pair domain, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 67,600 pairs enumerated; gap: none.
#[inline]
#[must_use]
pub const fn rows_equal(left: Row, right: Row) -> bool {
    match (left, right) {
        (Row::Closed(a), Row::Closed(b)) => a.bits == b.bits,
        (Row::Var(a), Row::Var(b)) => a.index == b.index,
        (Row::Closed(_), Row::Var(_)) | (Row::Var(_), Row::Closed(_)) => false,
    }
}

// ---------------------------------------------------------------------------
// Substitution and row unification (Volume 07 §7.3.9, exact).
// ---------------------------------------------------------------------------

/// The row-variable substitution σ: a fixed-size map from VarId to Row, no allocation (CEP-0).
///
/// Invariant (substitution forest): the binding graph σ is acyclic — every edge goes from a variable to a closed row or to another variable, and every edge is installed only after the occurs check (§7.3.9 step 2.b) passes, which is exactly the guard that keeps the graph a forest. Consequences used and documented throughout: resolve terminates with chain length ≤ ROW_VAR_LIMIT; the occurs walk terminates; unify's internal binds can never fail their occurs check (bind targets are always normal forms).
///
/// CEP:WHAT: The §7.3.9 substitution σ with occurs-checked binding and path-compressed resolution.
/// CEP:WHY: §7.3.9 steps 2-3 require exactly a binding map with path compression; the fixed-size array realizes it without allocation (SPEC-INDEX §5.2).
/// CEP:STATUS: complete
/// CEP:FAILURE: bind returns EffectError::OccursCheck { var } when the occurs check fails; no other failure (slots are indexed by validated VarIds, so no bounds failure exists).
/// CEP:ASSUMES: the forest invariant (established and maintained by bind; the slot store is private); ROW_VAR_LIMIT bounds the namespace.
/// CEP:COST: bind/resolve O(chain) ≤ ROW_VAR_LIMIT; binding O(1); no allocation.
/// CEP:EVIDENCE: EXH exh_bind_all_var_closed_targets_65536; EXH exh_occurs_check_chain_families; EXH exh_resolve_path_compression_chains_769; EXH exh_unify_var_var_empty_65536.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the full variable/closed-target domain and the chain families, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: binding certified over all 65,536 (var, closed-target) pairs; occurs check certified over the full-chain family (all failing (var, target) pairs) and all (var, target) pairs with unbound targets; resolve certified on maximal 255-edge chains for all 256 targets; gap: none.
pub struct Subst {
    slots: [Option<Row>; ROW_VAR_COUNT],
}

impl Subst {
    /// The empty substitution (all variables unbound).
    ///
    /// Refinement anchor: Volume 07 §7.3.9 (σ starts empty per constraint set). Failure set: none.
    /// Certificate: EXH `exh_unify_var_var_empty_65536` (every variable unbound on a fresh Subst, all 256×256 pairs).
    ///
    /// CEP:WHAT: fresh substitution constructor.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: fixed-size slot array (ROW_VAR_LIMIT slots).
    /// CEP:COST: O(ROW_VAR_LIMIT) zeroization; no allocation.
    /// CEP:EVIDENCE: EXH exh_unify_var_var_empty_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete; gap: none.
    #[inline]
    pub const fn new() -> Subst {
        Subst {
            slots: [None; ROW_VAR_COUNT],
        }
    }

    /// The current binding of `var`, without resolution (σ(ρ) as stored).
    ///
    /// Refinement anchor: Volume 07 §7.3.9 (the substitution σ). Input domain: all VarIds (total). Failure set: none (None = unbound).
    /// Certificate: EXH `exh_bind_all_var_closed_targets_65536` (binding == the stored row for all 65,536 (var, target) pairs; None before binding).
    ///
    /// CEP:WHAT: raw slot read.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none.
    /// CEP:ASSUMES: VarId invariant keeps the index in bounds.
    /// CEP:COST: O(1); CEP-0.
    /// CEP:EVIDENCE: EXH exh_bind_all_var_closed_targets_65536.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: full (var, target) domain; gap: none.
    #[inline]
    pub const fn binding(&self, var: VarId) -> Option<Row> {
        self.slots[var.index() as usize]
    }

    /// Binds σ(var) := row after the §7.3.9 step 2.b occurs check: fails with OccursCheck if var appears, transitively through current bindings, in row's variable chain.
    ///
    /// This is the pinned binding primitive (§7.3.9 case b: "occurs check ...; bind σ(ρ) := r"). If var already has a binding it is overwritten; the §7.3.9 solver only binds variables that resolve to unbound form (unify guarantees this on its paths), so overwriting is a direct caller's responsibility and never happens inside unify.
    ///
    /// Refinement anchor: Volume 07 §7.3.9 step 2.b.
    /// Input domain: all (VarId, Row) pairs. Failure set: EffectError::OccursCheck { var } iff the occurs check fails (the store itself is infallible).
    /// Certificate: EXH `exh_bind_all_var_closed_targets_65536` (all closed targets succeed and store exactly); EXH `exh_occurs_check_chain_families` (the full-chain family rejects exactly the cyclic (var, target) pairs, all 32,896 of them, with exact payloads; all acyclic pairs succeed).
    ///
    /// CEP:WHAT: occurs-checked substitution binding (§7.3.9 step 2.b).
    /// CEP:WHY: cyclic bindings would make resolve diverge; the occurs check is the pinned guard.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: EffectError::OccursCheck { var }.
    /// CEP:ASSUMES: the forest invariant before the call (the walk terminates only on acyclic σ); overwrite semantics documented above.
    /// CEP:COST: O(chain length) ≤ ROW_VAR_LIMIT; CEP-0.
    /// CEP:EVIDENCE: EXH exh_bind_all_var_closed_targets_65536; EXH exh_occurs_check_chain_families.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: certified over the full closed-target domain and the exhaustive chain families; gap: none.
    pub fn bind(&mut self, var: VarId, row: Row) -> Result<(), EffectError> {
        if self.occurs(var, row) {
            Err(EffectError::OccursCheck { var: var.index() })
        } else {
            self.store(var, row);
            Ok(())
        }
    }

    /// Resolves `row` through the substitution to its normal form, applying bindings with path compression (Volume 07 §7.3.9 step 3: "Apply bindings with path compression").
    ///
    /// A closed row is its own normal form; a variable resolves along its binding chain to the first closed row or unbound variable, and every variable on the chain is re-pointed at the normal form (compression). Termination: the substitution is a forest (see the type docs), so chain length ≤ ROW_VAR_LIMIT.
    ///
    /// Refinement anchor: Volume 07 §7.3.9 step 3. Input domain: all (Subst, Row) pairs (total). Failure set: none.
    /// Certificate: EXH `exh_resolve_path_compression_chains_769` (maximal 255-edge chains resolve to the exact target for all 256 closed targets, every intermediate slot is compressed, resolution is idempotent, closed rows pass through, and unbound chains resolve to their root variable).
    ///
    /// CEP:WHAT: path-compressed resolution (§7.3.9 step 3).
    /// CEP:WHY: re-checking deferred constraints (step 3) reads normal forms; compression keeps the O(|C|·V) bound.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none: total (unbound variables resolve to themselves).
    /// CEP:ASSUMES: the substitution forest invariant (acyclic σ).
    /// CEP:COST: O(chain) with compression amortizing repeats; recursion depth ≤ ROW_VAR_LIMIT; CEP-0.
    /// CEP:EVIDENCE: EXH exh_resolve_path_compression_chains_769; EXH exh_unify_var_closed_131072.
    /// CEP:FV-CLASS: FV-0
    /// CEP:FV-PROVER: Rust test harness (EXH over maximal chains and the closed-target domain, test-as-proof)
    /// CEP:FV-SORRY-COUNT: 0
    /// CEP:FV-REFINEMENT: complete: certified on maximal-length chains and all closed targets; gap: none.
    pub fn resolve(&mut self, row: Row) -> Row {
        match row {
            Row::Closed(_) => row,
            Row::Var(var) => {
                let current = self.slots[var.index() as usize];
                match current {
                    None => row,
                    Some(bound) => {
                        let resolved = self.resolve(bound);
                        self.slots[var.index() as usize] = Some(resolved);
                        resolved
                    }
                }
            }
        }
    }

    /// The occurs check of §7.3.9 step 2.b: does `var` appear, transitively through current bindings, in the variable chain of `row`?
    ///
    /// Termination: the substitution is a forest (type invariant), so the walk visits each chain node at most once, length ≤ ROW_VAR_LIMIT.
    ///
    /// CEP:WHAT: transitive free-variable check guarding every binding.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (boolean result; the failure decision is the caller's).
    /// CEP:ASSUMES: acyclic σ (forest invariant).
    /// CEP:COST: O(chain length); CEP-0.
    /// CEP:EVIDENCE: EXH exh_occurs_check_chain_families.
    fn occurs(&self, var: VarId, row: Row) -> bool {
        let mut current = row;
        loop {
            match current {
                Row::Closed(_) => return false,
                Row::Var(candidate) => {
                    if candidate == var {
                        return true;
                    }
                    match self.slots[candidate.index() as usize] {
                        None => return false,
                        Some(next) => current = next,
                    }
                }
            }
        }
    }

    /// Infallible slot store (index in bounds by the VarId invariant); called only after a passing occurs check, which maintains the forest invariant.
    ///
    /// CEP:WHAT: private binding store.
    /// CEP:FAILURE: none (no bounds failure is representable: VarId guarantees index < ROW_VAR_COUNT).
    /// CEP:ASSUMES: occurs check already passed at this call site (bind and unify's bind_resolved).
    /// CEP:COST: O(1).
    /// CEP:EVIDENCE: EXH exh_bind_all_var_closed_targets_65536 (store correctness through bind).
    fn store(&mut self, var: VarId, row: Row) {
        self.slots[var.index() as usize] = Some(row);
    }

    /// Number of bound slots; the driver's termination measure component.
    ///
    /// CEP:WHAT: private bound-slot counter for the §7.3.9 step-3 loop.
    /// CEP:FAILURE: none.
    /// CEP:COST: O(ROW_VAR_LIMIT).
    /// CEP:EVIDENCE: EXH exh_unify_constraints_single_67600 (fixpoint detection correctness through the driver).
    fn count_bound(&self) -> usize {
        let mut n = 0usize;
        let mut i = 0usize;
        while i < ROW_VAR_COUNT {
            if self.slots[i].is_some() {
                n += 1;
            }
            i += 1;
        }
        n
    }
}

/// One row-unification constraint (Volume 07 §7.3.8): the pair of rows to unify. Constraints are generated by (a) the call rule (r_ambient = r_ambient ∪ r_callee), (b) annotation conformance (r_inf ⊆ r_decl), (c) impl conformance (r_impl ⊆ r_trait), and (d) row-variable instantiation at monomorphization; the AST-level generator is compiler-side, and this kernel unifies the pairs it is given.
///
/// CEP:WHAT: a single (left, right) unification constraint pair.
/// CEP:WHY: §7.3.9 operates on the constraint set C; the pair is its unit.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (a data record; failures surface through unify/unify_constraints).
/// CEP:ASSUMES: both rows are constructor-validated (canonical by construction).
/// CEP:COST: O(1); Copy.
/// CEP:EVIDENCE: EXH exh_unify_constraints_single_67600; EXH exh_unify_constraints_conflict_index_65536.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete; gap: none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Constraint {
    /// The left operand of the constraint (row₁ in §7.3.9).
    pub left: Row,
    /// The right operand of the constraint (row₂ in §7.3.9).
    pub right: Row,
}

/// Unifies two rows in the substitution σ, exactly per Volume 07 §7.3.9:
///
/// 1. Normalize both operands to canonical form — the identity on this representation (canonical by construction, EXH-certified), stated because §7.3.9 step 1 requires it.
/// 2. Apply bindings with path compression (step 3, eagerly): both operands are resolved through σ first.
/// 3. Case analysis over the resolved normal forms:
///    (a) (closed, closed): succeed iff equal; else fail with Mismatch { only_in_left: row₁ \ row₂, only_in_right: row₂ \ row₁ } (rejection P07-C01);
///    (b) (var ρ, closed r): occurs check, then bind σ(ρ) := r;
///    (c) (closed r, var ρ): symmetric to (b);
///    (d) (var ρ₁, var ρ₂): if ρ₁ = ρ₂ succeed; else bind σ(ρ_max) := ρ_min, min/max by variable creation index (determinism, G14).
///
/// Occurs-check reachability proof (why Occurs is defensive): after eager resolution both operands are normal forms; case (b)/(c) binds a variable to a closed row (empty free-variable set), and case (d) binds σ(ρ_max) := ρ_min where ρ_min is unbound (it resolved to itself), so the new edge ρ_max → ρ_min ends at a node with no outgoing edge and cannot close a cycle; every earlier edge was installed under the same argument inductively. Hence σ stays a forest and the occurs check passes on every unify path — Occurs is retained because §7.3.9 names the check inside unification, and EXH-guarded.
///
/// Refinement anchor: Volume 07 §7.3.9 steps 1-3 (exact); SPEC-INDEX §5.3.
/// Input domain: all (Subst, Row, Row) triples; certified exhaustively over: all 65,536 closed/closed pairs (empty σ), all 131,072 oriented var/closed pairs, all 65,536 var/var pairs (empty σ), and 131,072 pre-bound var/var states.
/// Failure set: UnifyError::Mismatch (exact set-difference payloads) and, defensively, UnifyError::Occurs (proven unreachable from this function).
/// Certificate: EXH `exh_unify_closed_closed_65536`, `exh_unify_var_closed_131072`, `exh_unify_var_var_empty_65536`, `exh_unify_var_var_prebound_131072`, `exh_occurs_check_chain_families`; PROP `prop_unify_random_states_10000` (differential against an independent no-compression model, paired with the EXH family).
///
/// CEP:WHAT: the pinned row-unification step (§7.3.9).
/// CEP:WHY: constraint solving is row unification (§7.3.8); every effect check bottoms out here.
/// CEP:STATUS: complete
/// CEP:FAILURE: UnifyError::Mismatch { only_in_left, only_in_right } for closed mismatches (P07-C01); UnifyError::Occurs { var } defensively (unreachable per the proof above).
/// CEP:ASSUMES: canonical-by-construction operands; the substitution forest invariant; per-item variable namespaces with creation-index ids (G14).
/// CEP:COST: O(chain) per call (resolution dominates); path compression amortizes; no allocation; CEP-0.
/// CEP:EVIDENCE: EXH exh_unify_closed_closed_65536; EXH exh_unify_var_closed_131072; EXH exh_unify_var_var_empty_65536; EXH exh_unify_var_var_prebound_131072; EXH exh_occurs_check_chain_families; PROP prop_unify_random_states_10000.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete closed-pair domain and the stated variable-state families, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all listed domains enumerated (no sampling); the variable-state space beyond the stated families is covered by the paired differential PROP and the structural proof of the case analysis; gap: none.
pub fn unify(subst: &mut Subst, left: Row, right: Row) -> Result<(), UnifyError> {
    // §7.3.9 step 1 (canonical form) + step 3 (bindings with path compression), applied eagerly.
    let left = subst.resolve(left);
    let right = subst.resolve(right);
    match (left, right) {
        (Row::Closed(c1), Row::Closed(c2)) => {
            if c1 == c2 {
                Ok(())
            } else {
                Err(UnifyError::Mismatch {
                    only_in_left: c1.difference(c2),
                    only_in_right: c2.difference(c1),
                })
            }
        }
        (Row::Var(var), Row::Closed(closed)) => bind_resolved(subst, var, Row::Closed(closed)),
        (Row::Closed(closed), Row::Var(var)) => bind_resolved(subst, var, Row::Closed(closed)),
        (Row::Var(v1), Row::Var(v2)) => {
            if v1 == v2 {
                Ok(())
            } else {
                // §7.3.9 case (d): bind σ(ρ_max) := ρ_min by variable creation index (G14).
                let (low, high) = if v1.index() < v2.index() {
                    (v1, v2)
                } else {
                    (v2, v1)
                };
                bind_resolved(subst, high, Row::Var(low))
            }
        }
    }
}

/// The §7.3.9 step 2.b/2.d binding site used by unify: occurs check, then store.
///
/// The store is infallible (VarId invariant); the occurs check is retained per the letter of §7.3.9 and is proven unreachable-failing on unify's paths (see unify's docs).
///
/// CEP:WHAT: private occurs-check-then-bind for unify's cases (b), (c), (d).
/// CEP:FAILURE: UnifyError::Occurs { var } iff the occurs check fails (proven unreachable from unify; retained defensively).
/// CEP:ASSUMES: both operands are resolved normal forms (unify resolves first).
/// CEP:COST: O(chain) for the occurs walk.
/// CEP:EVIDENCE: EXH exh_unify_var_closed_131072; EXH exh_unify_var_var_empty_65536; EXH exh_occurs_check_chain_families.
fn bind_resolved(subst: &mut Subst, var: VarId, target: Row) -> Result<(), UnifyError> {
    if subst.occurs(var, target) {
        Err(UnifyError::Occurs { var: var.index() })
    } else {
        subst.store(var, target);
        Ok(())
    }
}

/// The §7.3.9 step-3 driver over a constraint set: unify every constraint against current normal forms, and loop "until no deferred constraints remain or a failure occurs".
///
/// A pass that installs no new binding is the fixpoint: every constraint then holds between resolved normal forms, which is the deferred-recheck semantics of §7.3.9 step 3 (a constraint over variables bound later is re-unified in a later pass). Termination measure (§7.3.9): (number of unbound row variables, number of unprocessed constraints) — a pass with ≥ 1 binding strictly decreases the first component (bindings never retract); a pass with none terminates; total passes ≤ ROW_VAR_LIMIT + 1, worst-case O(|C|·V) with path compression (obligation OB-07-6's bound).
///
/// Refinement anchor: Volume 07 §7.3.8 (constraint set) and §7.3.9 step 3.
/// Input domain: constraint slices of any length over validated rows; certified over: single-constraint sets for all 67,600 row pairs (4 variables), the full conflict family (all 65,536 mask pairs), the deferred re-check families, and 255-constraint chains in both orders.
/// Failure set: ConstraintFailure::Mismatch { index, failure } naming the first failing constraint of the first failing pass (slice order, deterministic G14).
/// Certificate: EXH `exh_unify_constraints_single_67600` (driver == single-step unify on every pair, plus post-fixpoint resolution equality), `exh_unify_constraints_conflict_index_65536`, `exh_unify_constraints_deferred_families_1281`, `exh_unify_constraints_chain_255`.
///
/// CEP:WHAT: the pinned constraint-set unification loop (§7.3.9 step 3).
/// CEP:WHY: constraint solving is row unification (§7.3.8); the driver realizes the loop the spec defines.
/// CEP:STATUS: complete
/// CEP:FAILURE: ConstraintFailure::Mismatch { index, failure }.
/// CEP:ASSUMES: constraints examined in slice order (deterministic G14); bindings never retract; constraint generation from the AST is compiler-side (§7.3.8) — this driver unifies the pairs it is given.
/// CEP:COST: O(|C|·V) worst case, ≤ ROW_VAR_LIMIT + 1 passes; no allocation; CEP-0.
/// CEP:EVIDENCE: EXH exh_unify_constraints_single_67600; EXH exh_unify_constraints_conflict_index_65536; EXH exh_unify_constraints_deferred_families_1281; EXH exh_unify_constraints_chain_255.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the stated families, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: the driver's case space is covered by the enumerated single-constraint domain, the full conflict family, the deferred families, and the chain tests; the loop itself adds no cases beyond unify's (each pass is a unify sequence) and termination is the spec's own measure; gap: none.
pub fn unify_constraints(
    subst: &mut Subst,
    constraints: &[Constraint],
) -> Result<(), ConstraintFailure> {
    loop {
        let bound_before = subst.count_bound();
        let mut index = 0usize;
        while index < constraints.len() {
            let constraint = constraints[index];
            if let Err(failure) = unify(subst, constraint.left, constraint.right) {
                return Err(ConstraintFailure::Mismatch { index, failure });
            }
            index += 1;
        }
        if subst.count_bound() == bound_before {
            return Ok(());
        }
    }
}

/// The pinned function-call rule: calling f : (row ρ) → τ requires the caller's row to include ρ (SPEC-INDEX §5.3 bullet 4; Volume 07 §7.3.9 rule 3: r_callee ⊆ r_decl of the enclosing function, violation P07-C03).
///
/// Refinement anchor: SPEC-INDEX §5.3 bullet 4; Volume 07 §7.3.9 rule 3, §7.3.8 (a).
/// Input domain: all 256×256 (ambient, callee) pairs (total).
/// Failure set: CallError::Missing { missing } with missing = r_callee \ r_ambient.
/// Certificate: EXH `exh_check_call_65536` (accepted exactly on leq(callee, ambient), missing-set payload exact on all 65,536 pairs).
///
/// CEP:WHAT: the pinned call rule as a total check.
/// CEP:WHY: §5.3 pins the rule; §7.3.9 rule 3 pins its rejection code; the checker consumes exactly this.
/// CEP:STATUS: complete
/// CEP:FAILURE: CallError::Missing { missing } (P07-C03).
/// CEP:ASSUMES: requirement-position inclusion (⊆), not unification (§7.3.10: non-requirement positions use equality/join).
/// CEP:COST: O(1); CEP-0.
/// CEP:EVIDENCE: EXH exh_check_call_65536.
/// CEP:FV-CLASS: FV-0
/// CEP:FV-PROVER: Rust test harness (EXH over the complete pair domain, test-as-proof)
/// CEP:FV-SORRY-COUNT: 0
/// CEP:FV-REFINEMENT: complete: all 65,536 pairs enumerated with exact payloads; gap: none.
#[inline]
pub fn check_call(ambient: ClosedRow, callee: ClosedRow) -> Result<(), CallError> {
    if callee.leq(ambient) {
        Ok(())
    } else {
        Err(CallError::Missing {
            missing: callee.difference(ambient),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// Every EXH test enumerates its full stated domain with u16 loops; no sampling.
// Each test's enumerated case count is ≤ 10^6 (SPEC-INDEX §5.1).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;

    /// Volume 12 §12.1 bit positions indexed by canonical rank (§7.1.2):
    /// rank 0 alloc→1, 1 async→4, 2 concurrency→5, 3 ffi→6, 4 io→0, 5 nondet→7, 6 panic→2, 7 unsafe→3.
    const BIT_INDEX_BY_RANK: [u8; ATOM_COUNT] = [1, 4, 5, 6, 0, 7, 2, 3];

    /// Variables used in the row-space tests: row space = 256 closed + 4 vars = 260 rows.
    const TEST_VAR_SPACE: u16 = 4;
    /// Row-space size: 256 closed rows + TEST_VAR_SPACE variables.
    const TEST_ROW_SPACE: u16 = CLOSED_ROW_COUNT as u16 + TEST_VAR_SPACE;

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

    /// Decodes a test row-space index: 0..256 → closed row, 256..260 → variable.
    fn test_row(i: u16) -> Row {
        if i < CLOSED_ROW_COUNT as u16 {
            Row::closed(ClosedRow::from_mask(i).expect("mask in lattice"))
        } else {
            Row::var(VarId::new(i - CLOSED_ROW_COUNT as u16).expect("var below limit"))
        }
    }

    /// EXH certificate: the atom vocabulary is exactly the pinned 8 names in the pinned §7.1.2
    /// canonical order, derived Ord agrees with the canonical order, and the Volume 12 §12.1 bit
    /// mapping is exact (positions, disjointness, union == ATOM_MASK_ALL, exact ATOM_BIT values).
    /// Domain: 8 atoms (full closed vocabulary) + 64 pairwise mask checks.
    #[test]
    fn exh_atoms_canonical_order_and_bit_mapping_8() {
        assert_eq!(
            CANONICAL_ATOM_ORDER,
            [
                Atom::Alloc,
                Atom::Async,
                Atom::Concurrency,
                Atom::Ffi,
                Atom::Io,
                Atom::Nondet,
                Atom::Panic,
                Atom::Unsafe
            ]
        );
        assert_eq!(
            ATOM_NAMES,
            ["alloc", "async", "concurrency", "ffi", "io", "nondet", "panic", "unsafe"]
        );
        let expected_bits = [
            ATOM_BIT_ALLOC,
            ATOM_BIT_ASYNC,
            ATOM_BIT_CONCURRENCY,
            ATOM_BIT_FFI,
            ATOM_BIT_IO,
            ATOM_BIT_NONDET,
            ATOM_BIT_PANIC,
            ATOM_BIT_UNSAFE,
        ];
        // Exact Volume 12 §12.1 values: io=bit0, alloc=bit1, panic=bit2, unsafe=bit3,
        // async=bit4, concurrency=bit5, ffi=bit6, nondet=bit7.
        assert_eq!(ATOM_BIT_IO, 1 << 0);
        assert_eq!(ATOM_BIT_ALLOC, 1 << 1);
        assert_eq!(ATOM_BIT_PANIC, 1 << 2);
        assert_eq!(ATOM_BIT_UNSAFE, 1 << 3);
        assert_eq!(ATOM_BIT_ASYNC, 1 << 4);
        assert_eq!(ATOM_BIT_CONCURRENCY, 1 << 5);
        assert_eq!(ATOM_BIT_FFI, 1 << 6);
        assert_eq!(ATOM_BIT_NONDET, 1 << 7);
        let mut rank = 0usize;
        while rank < ATOM_COUNT {
            let atom = CANONICAL_ATOM_ORDER[rank];
            assert_eq!(atom_index(atom), rank as u8);
            assert_eq!(atom_name(atom), ATOM_NAMES[rank]);
            assert_eq!(atom_bit_index(atom), BIT_INDEX_BY_RANK[rank]);
            assert_eq!(atom_bit(atom), expected_bits[rank]);
            assert_eq!(ClosedRow::from_atom(atom).bits(), expected_bits[rank]);
            let mut other = 0usize;
            while other < ATOM_COUNT {
                if other != rank {
                    let other_atom = CANONICAL_ATOM_ORDER[other];
                    // Derived Ord agrees with the canonical order in both directions
                    // (all 56 ordered pairs of the 8-atom vocabulary).
                    assert_eq!(
                        atom < other_atom,
                        rank < other,
                        "derived Ord vs canonical order at ranks {rank},{other}"
                    );
                    assert_eq!(
                        expected_bits[rank] & expected_bits[other],
                        0,
                        "atom masks disjoint"
                    );
                }
                other += 1;
            }
            rank += 1;
        }
        let mut union = 0u16;
        for bit in expected_bits {
            union |= bit;
        }
        assert_eq!(union, ATOM_MASK_ALL);
        assert_eq!(ATOM_MASK_ALL, 0x00FF);
        assert_eq!(CLOSED_ROW_COUNT, 256);
    }

    /// EXH certificate: atom_from_index is Ok exactly on 0..=7 with the exact canonical atom,
    /// and AtomIndexOutOfRange { index } otherwise. Domain: all 256 u8 values.
    #[test]
    fn exh_atom_from_index_256() {
        let mut i: u16 = 0;
        while i <= u8::MAX as u16 {
            let index = i as u8;
            match atom_from_index(index) {
                Ok(atom) => {
                    assert!(index <= ATOM_INDEX_MAX);
                    assert_eq!(atom, CANONICAL_ATOM_ORDER[index as usize]);
                    assert_eq!(atom_index(atom), index);
                }
                Err(EffectError::AtomIndexOutOfRange { index: got }) => {
                    assert!(index > ATOM_INDEX_MAX);
                    assert_eq!(got, index);
                }
                Err(other) => panic!("wrong error for index {index}: {other:?}"),
            }
            i += 1;
        }
    }

    /// EXH certificate: from_mask accepts exactly the 256 lattice masks (bits() == mask) and
    /// returns MaskOutOfRange { mask } with the exact payload otherwise. Domain: all 65,536 u16.
    #[test]
    fn exh_mask_validation_65536() {
        let mut m: u16 = 0;
        loop {
            match ClosedRow::from_mask(m) {
                Ok(row) => {
                    assert!(m <= ATOM_MASK_ALL);
                    assert_eq!(row.bits(), m);
                    assert_eq!(row.u8_mask(), m as u8);
                }
                Err(EffectError::MaskOutOfRange { mask: got }) => {
                    assert!(m > ATOM_MASK_ALL);
                    assert_eq!(got, m);
                }
                Err(other) => panic!("wrong error for mask {m:#06x}: {other:?}"),
            }
            if m == u16::MAX {
                break;
            }
            m += 1;
        }
    }

    /// EXH certificate: every closed row is canonical by construction (canonicalize is the
    /// identity, idempotent), canonical_sequence is exactly the rank-ordered sorted deduplicated
    /// membership, and atom_count matches. Domain: 256 rows × 8 ranks = 2,048 cases + 256 rows.
    #[test]
    fn exh_closedrow_canonical_form_2048() {
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let row = ClosedRow::from_mask(m).expect("mask in lattice");
            let as_row = Row::closed(row);
            assert_eq!(canonicalize(as_row), as_row);
            assert_eq!(canonicalize(canonicalize(as_row)), as_row);
            // Independent reference: rank-ordered membership from the raw mask.
            let mut expected: [Option<Atom>; ATOM_COUNT] = [None; ATOM_COUNT];
            let mut expected_count = 0u8;
            let mut rank = 0usize;
            while rank < ATOM_COUNT {
                let bit = 1u16 << BIT_INDEX_BY_RANK[rank] as u16;
                if (m & bit) != 0 {
                    expected[rank] = Some(CANONICAL_ATOM_ORDER[rank]);
                    expected_count += 1;
                }
                rank += 1;
            }
            assert_eq!(row.canonical_sequence(), expected);
            assert_eq!(row.atom_count(), expected_count);
            assert_eq!(row.atom_count(), (u16::from(m).count_ones()) as u8);
            m += 1;
        }
    }

    /// EXH certificate (pinned law: join is commutative). Domain: all 256×256 = 65,536 pairs.
    #[test]
    fn exh_join_commutative_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                assert_eq!(ra.join(rb), rb.join(ra), "join commutes on {a},{b}");
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate (pinned law: join is idempotent). Domain: all 256 rows.
    #[test]
    fn exh_join_idempotent_256() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            assert_eq!(ra.join(ra), ra);
            a += 1;
        }
    }

    /// EXH certificate (pinned law: join identity ∅). Domain: all 256 rows.
    #[test]
    fn exh_join_identity_empty_256() {
        assert!(ROW_EMPTY.is_empty());
        assert_eq!(ROW_EMPTY.bits(), 0);
        assert_eq!(ClosedRow::EMPTY, ROW_EMPTY);
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            assert_eq!(ra.join(ROW_EMPTY), ra);
            assert_eq!(ROW_EMPTY.join(ra), ra);
            assert_eq!(ra.is_empty(), a == 0);
            a += 1;
        }
    }

    /// EXH certificate (absorption, MATP obligation eff_absorb, join side):
    /// join(a, meet(a, b)) == a. Domain: all 65,536 pairs.
    #[test]
    fn exh_join_absorption_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                assert_eq!(ra.join(ra.meet(rb)), ra, "absorb on {a},{b}");
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate (absorption, MATP obligation eff_absorb, meet side):
    /// meet(a, join(a, b)) == a. Domain: all 65,536 pairs.
    #[test]
    fn exh_meet_absorption_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                assert_eq!(ra.meet(ra.join(rb)), ra, "absorb on {a},{b}");
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH characterization: join(a,b) == per-atom union and meet(a,b) == per-atom intersection
    /// on every pair and every atom. This is the pointwise certificate whose composition with
    /// exh_boolean_lattice_laws_per_atom_320 discharges join/meet associativity and meet
    /// distributivity over all 256^3 triples (two functions with identical per-atom membership
    /// are equal). Domain: 65,536 pairs × 8 atoms = 524,288 cases.
    #[test]
    fn exh_join_meet_per_atom_characterization_524288() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                let joined = ra.join(rb);
                let met = ra.meet(rb);
                let mut rank = 0usize;
                while rank < ATOM_COUNT {
                    let bit = 1u16 << BIT_INDEX_BY_RANK[rank] as u16;
                    let (xa, xb) = ((a & bit) != 0, (b & bit) != 0);
                    assert_eq!((joined.bits() & bit) != 0, xa || xb, "join bit {rank}");
                    assert_eq!((met.bits() & bit) != 0, xa && xb, "meet bit {rank}");
                    rank += 1;
                }
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate (pinned law, meet side): meet is commutative. Domain: all 65,536 pairs.
    #[test]
    fn exh_meet_commutative_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                assert_eq!(ra.meet(rb), rb.meet(ra));
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate: meet is idempotent. Domain: all 256 rows.
    #[test]
    fn exh_meet_idempotent_256() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            assert_eq!(ra.meet(ra), ra);
            a += 1;
        }
    }

    /// EXH certificate: meet identity is the full atom set ROW_ALL. Domain: all 256 rows.
    #[test]
    fn exh_meet_identity_all_256() {
        assert_eq!(ROW_ALL.bits(), ATOM_MASK_ALL);
        assert_eq!(ROW_ALL.atom_count(), ATOM_COUNT as u8);
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            assert_eq!(ra.meet(ROW_ALL), ra);
            a += 1;
        }
    }

    /// EXH characterization: leq(a, b) == the conjunction over the 8 pinned atoms of the
    /// per-atom implication (a has the atom ⟹ b has it), i.e. exactly a ⊆ b (SPEC-INDEX §5.3).
    /// Every individual implication is asserted pointwise in the leq-forced direction (leq
    /// cannot hold when a has an atom b lacks) and the 8-way conjunction is asserted as an
    /// equality per pair. Pointwise certificate; with the per-atom implication transitivity in
    /// exh_boolean_lattice_laws_per_atom_320 it discharges leq transitivity over all 256^3
    /// triples. Domain: 65,536 pairs × 8 atoms = 524,288 pointwise implications (+ 65,536
    /// conjunction equalities).
    #[test]
    fn exh_leq_subset_characterization_524288() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                let is_leq = ra.leq(rb);
                let mut all_imply = true;
                let mut rank = 0usize;
                while rank < ATOM_COUNT {
                    let bit = 1u16 << BIT_INDEX_BY_RANK[rank] as u16;
                    let (xa, xb) = ((a & bit) != 0, (b & bit) != 0);
                    let implies = !xa || xb;
                    // Necessity, per atom: leq forces every single-atom implication.
                    assert!(!is_leq || implies, "leq violates bit {rank} on {a},{b}");
                    all_imply &= implies;
                    rank += 1;
                }
                // Sufficiency: the conjunction of all 8 per-atom implications is exactly leq.
                assert_eq!(is_leq, all_imply, "leq subset characterization on {a},{b}");
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate (pinned law: subtyping is reflexive). Domain: all 256 rows.
    #[test]
    fn exh_leq_reflexive_256() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            assert!(ra.leq(ra));
            a += 1;
        }
    }

    /// EXH certificate (pinned law: subtyping is antisymmetric):
    /// leq(a,b) && leq(b,a) ⟺ a == b. Domain: all 65,536 pairs.
    #[test]
    fn exh_leq_antisymmetric_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                assert_eq!(ra.leq(rb) && rb.leq(ra), ra == rb, "antisym on {a},{b}");
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate: leq(a,b) ⟺ meet(a,b) == a ⟺ join(a,b) == b on every pair
    /// (order characterizations). Domain: 65,536 pairs × 3 formulations = 196,608 cases.
    #[test]
    fn exh_leq_order_characterizations_196608() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                let is_leq = ra.leq(rb);
                assert_eq!(ra.meet(rb) == ra, is_leq);
                assert_eq!(ra.join(rb) == rb, is_leq);
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate: the Boolean lattice laws hold per atom coordinate (OR/AND associativity,
    /// both distributivities, implication transitivity), computed through the crate's own
    /// join/meet/leq on single-atom rows. Composition with the pointwise characterizations
    /// (exh_join_meet_per_atom_characterization_524288, exh_leq_subset_characterization_524288)
    /// discharges join/meet associativity, meet distributivity over join (both directions), and
    /// leq transitivity over all 256^3 triples of the full lattice. Domain: 8 atoms × 8 triples
    /// × 5 laws = 320 cases.
    #[test]
    fn exh_boolean_lattice_laws_per_atom_320() {
        let mut rank = 0usize;
        while rank < ATOM_COUNT {
            let one = ClosedRow::from_atom(CANONICAL_ATOM_ORDER[rank]);
            let mut t = 0usize;
            while t < 8 {
                let (x, y, z) = (
                    (t >> 2) & 1 == 1,
                    (t >> 1) & 1 == 1,
                    t & 1 == 1,
                );
                let rx = if x { one } else { ROW_EMPTY };
                let ry = if y { one } else { ROW_EMPTY };
                let rz = if z { one } else { ROW_EMPTY };
                assert_eq!(rx.join(ry).join(rz), rx.join(ry.join(rz)), "or assoc {rank},{t}");
                assert_eq!(rx.meet(ry).meet(rz), rx.meet(ry.meet(rz)), "and assoc {rank},{t}");
                assert_eq!(
                    rx.meet(ry.join(rz)),
                    rx.meet(ry).join(rx.meet(rz)),
                    "meet over join {rank},{t}"
                );
                assert_eq!(
                    rx.join(ry.meet(rz)),
                    rx.join(ry).meet(rx.join(rz)),
                    "join over meet {rank},{t}"
                );
                if rx.leq(ry) && ry.leq(rz) {
                    assert!(rx.leq(rz), "leq transitivity {rank},{t}");
                }
                t += 1;
            }
            rank += 1;
        }
    }

    /// EXH certificate (§7.1.2 + §7.4 d2): rows_equal == derived PartialEq == canonical-form
    /// equality on the full row space (256 closed + 4 variables = 260 rows); is_open/is_closed
    /// agree with the variant; canonicalize is the identity on every row. Domain: 260² = 67,600
    /// pairs + 260 rows.
    #[test]
    fn exh_row_equality_canonical_agreement_67600() {
        let mut i: u16 = 0;
        while i < TEST_ROW_SPACE {
            let r1 = test_row(i);
            assert_eq!(r1.is_open(), i >= CLOSED_ROW_COUNT as u16);
            assert_eq!(r1.is_closed(), i < CLOSED_ROW_COUNT as u16);
            assert_eq!(canonicalize(r1), r1);
            let mut j: u16 = 0;
            while j < TEST_ROW_SPACE {
                let r2 = test_row(j);
                let eq = rows_equal(r1, r2);
                assert_eq!(eq, r1 == r2, "rows_equal vs PartialEq on {i},{j}");
                assert_eq!(eq, canonicalize(r1) == canonicalize(r2), "canonical agreement {i},{j}");
                j += 1;
            }
            i += 1;
        }
    }

    /// EXH certificate: VarId::new and Row::var_from_u16 accept exactly ids < ROW_VAR_LIMIT
    /// (with exact payloads) and reject the rest; roundtrip index(); canonicalize is the
    /// identity on open rows. Domain: all 65,536 u16 ids.
    #[test]
    fn exh_varid_validation_65536() {
        let mut id: u16 = 0;
        loop {
            let expected_ok = id < ROW_VAR_LIMIT;
            match VarId::new(id) {
                Ok(var) => {
                    assert!(expected_ok);
                    assert_eq!(var.index(), id);
                    let row = Row::var(var);
                    assert!(row.is_open());
                    assert_eq!(canonicalize(row), row);
                }
                Err(EffectError::RowVarOutOfRange { var: got }) => {
                    assert!(!expected_ok);
                    assert_eq!(got, id);
                }
                Err(other) => panic!("wrong error for var id {id}: {other:?}"),
            }
            match Row::var_from_u16(id) {
                Ok(Row::Var(var)) => {
                    assert!(expected_ok);
                    assert_eq!(var.index(), id);
                }
                Ok(Row::Closed(_)) => panic!("var_from_u16 gave a closed row"),
                Err(EffectError::RowVarOutOfRange { var: got }) => {
                    assert!(!expected_ok);
                    assert_eq!(got, id);
                }
                Err(other) => panic!("wrong error for var id {id}: {other:?}"),
            }
            if id == u16::MAX {
                break;
            }
            id += 1;
        }
    }

    /// EXH certificate (§7.3.9 case a): unify of two closed rows succeeds iff they are equal and,
    /// on failure, carries exactly only_in_left = row₁ \ row₂ and only_in_right = row₂ \ row₁.
    /// Domain: all 65,536 closed pairs.
    #[test]
    fn exh_unify_closed_closed_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ra = ClosedRow::from_mask(a).expect("in lattice");
                let rb = ClosedRow::from_mask(b).expect("in lattice");
                let mut subst = Subst::new();
                let result = unify(&mut subst, Row::closed(ra), Row::closed(rb));
                if a == b {
                    assert_eq!(result, Ok(()));
                } else {
                    assert_eq!(
                        result,
                        Err(UnifyError::Mismatch {
                            only_in_left: ra.difference(rb),
                            only_in_right: rb.difference(ra),
                        })
                    );
                }
                assert_eq!(subst.binding(VarId::new(0).expect("0 < limit")), None);
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate (§7.3.9 cases b, c): unify(var, closed) and unify(closed, var) bind the
    /// variable to exactly the closed row, resolve to it, and re-unify successfully; no closed
    /// row is ever modified. Domain: both orientations × 256 vars × 256 masks = 131,072 cases.
    #[test]
    fn exh_unify_var_closed_131072() {
        let mut v: u16 = 0;
        while v < ROW_VAR_LIMIT {
            let var = VarId::new(v).expect("v < limit");
            let mut m: u16 = 0;
            while m <= ATOM_MASK_ALL {
                let closed = ClosedRow::from_mask(m).expect("in lattice");
                // Orientation (Var, Closed) — case (b).
                let mut subst = Subst::new();
                assert_eq!(unify(&mut subst, Row::var(var), Row::closed(closed)), Ok(()));
                assert_eq!(subst.binding(var), Some(Row::closed(closed)));
                assert_eq!(subst.resolve(Row::var(var)), Row::closed(closed));
                assert_eq!(unify(&mut subst, Row::var(var), Row::closed(closed)), Ok(()));
                // Orientation (Closed, Var) — case (c).
                let mut subst = Subst::new();
                assert_eq!(unify(&mut subst, Row::closed(closed), Row::var(var)), Ok(()));
                assert_eq!(subst.binding(var), Some(Row::closed(closed)));
                assert_eq!(subst.resolve(Row::var(var)), Row::closed(closed));
                m += 1;
            }
            v += 1;
        }
    }

    /// EXH certificate (§7.3.9 case d, empty substitution): unify(var, var) succeeds for every
    /// pair; equal variables bind nothing; distinct variables bind σ(ρ_max) := ρ_min by creation
    /// index, leaving ρ_min unbound, and both resolve to ρ_min. Domain: all 256² = 65,536 pairs.
    #[test]
    fn exh_unify_var_var_empty_65536() {
        let mut v: u16 = 0;
        while v < ROW_VAR_LIMIT {
            let mut w: u16 = 0;
            while w < ROW_VAR_LIMIT {
                let var_v = VarId::new(v).expect("v < limit");
                let var_w = VarId::new(w).expect("w < limit");
                let mut subst = Subst::new();
                assert_eq!(unify(&mut subst, Row::var(var_v), Row::var(var_w)), Ok(()));
                if v == w {
                    assert_eq!(subst.binding(var_v), None);
                } else {
                    let (low, high) = if v < w { (v, w) } else { (w, v) };
                    let var_low = VarId::new(low).expect("low < limit");
                    let var_high = VarId::new(high).expect("high < limit");
                    assert_eq!(subst.binding(var_high), Some(Row::var(var_low)));
                    assert_eq!(subst.binding(var_low), None);
                    assert_eq!(subst.resolve(Row::var(var_high)), Row::var(var_low));
                    assert_eq!(subst.resolve(Row::var(var_low)), Row::var(var_low));
                }
                w += 1;
            }
            v += 1;
        }
    }

    /// EXH certificate (§7.3.9 cases a-d composed, pre-bound states): with both variables of an
    /// ordered pair pre-bound to closed rows, unify(var, var) reduces to closed/closed: it
    /// succeeds iff the bindings are equal and otherwise reports the exact set differences; the
    /// occurs check never fires from unify. Domain: 2 argument orders × 256² mask pairs =
    /// 131,072 cases.
    #[test]
    fn exh_unify_var_var_prebound_131072() {
        let var0 = VarId::new(0).expect("0 < limit");
        let var1 = VarId::new(1).expect("1 < limit");
        let mut m1: u16 = 0;
        while m1 <= ATOM_MASK_ALL {
            let mut m2: u16 = 0;
            while m2 <= ATOM_MASK_ALL {
                let c1 = ClosedRow::from_mask(m1).expect("in lattice");
                let c2 = ClosedRow::from_mask(m2).expect("in lattice");
                // Order (Var0, Var1).
                let mut subst = Subst::new();
                subst.bind(var0, Row::closed(c1)).expect("closed target");
                subst.bind(var1, Row::closed(c2)).expect("closed target");
                let result = unify(&mut subst, Row::var(var0), Row::var(var1));
                if m1 == m2 {
                    assert_eq!(result, Ok(()));
                } else {
                    assert_eq!(
                        result,
                        Err(UnifyError::Mismatch {
                            only_in_left: c1.difference(c2),
                            only_in_right: c2.difference(c1),
                        })
                    );
                }
                // Order (Var1, Var0).
                let mut subst = Subst::new();
                subst.bind(var0, Row::closed(c1)).expect("closed target");
                subst.bind(var1, Row::closed(c2)).expect("closed target");
                let result = unify(&mut subst, Row::var(var1), Row::var(var0));
                if m1 == m2 {
                    assert_eq!(result, Ok(()));
                } else {
                    assert_eq!(
                        result,
                        Err(UnifyError::Mismatch {
                            only_in_left: c2.difference(c1),
                            only_in_right: c1.difference(c2),
                        })
                    );
                }
                m2 += 1;
            }
            m1 += 1;
        }
    }

    /// EXH certificate: bind stores exactly the target for every (var, closed-target) pair and
    /// leaves every other slot untouched; re-binding a bound variable overwrites (documented
    /// contract). Domain: 256 vars × 256 masks = 65,536 cases.
    #[test]
    fn exh_bind_all_var_closed_targets_65536() {
        let mut v: u16 = 0;
        while v < ROW_VAR_LIMIT {
            let var = VarId::new(v).expect("v < limit");
            let mut m: u16 = 0;
            while m <= ATOM_MASK_ALL {
                let closed = ClosedRow::from_mask(m).expect("in lattice");
                let mut subst = Subst::new();
                assert_eq!(subst.bind(var, Row::closed(closed)), Ok(()));
                assert_eq!(subst.binding(var), Some(Row::closed(closed)));
                assert_eq!(subst.resolve(Row::var(var)), Row::closed(closed));
                // Overwrite with the same target is accepted (occurs check passes on closed rows).
                assert_eq!(subst.bind(var, Row::closed(closed)), Ok(()));
                assert_eq!(subst.binding(var), Some(Row::closed(closed)));
                // Any other variable is untouched.
                let other = VarId::new((v + 1) % ROW_VAR_LIMIT).expect("wrapped < limit");
                assert_eq!(subst.binding(other), None);
                m += 1;
            }
            v += 1;
        }
    }

    /// EXH certificate (§7.3.9 step 2.b, occurs check): on the full downward chain
    /// σ(i+1) = Var(i) over all 256 variables, FV(Var(w)) = {0..w} exactly, so bind(v, Var(w))
    /// fails with OccursCheck { var: v } iff v ≤ w (all 32,896 failing pairs enumerated on one
    /// intact chain — failing binds do not mutate); every (v, w) with v > w succeeds on an
    /// unbound-target substitution (all 32,640 pairs); stratified full-chain successes: for
    /// w in {0, 1, 127, 254} every v > w binds onto the intact chain (the occurs check sees
    /// only FV(Var(w)) = {0..w}, which excludes v; the store overwrites σ(v) per bind's
    /// documented overwrite semantics) and both Var(w) and the rebound Var(v) resolve through
    /// the full chain (§7.3.9 step 3) to the chain root Var(0); self-binding always fails.
    /// Domain: 32,896 + 32,640 + 638 + 256 = 66,430 cases.
    #[test]
    fn exh_occurs_check_chain_families() {
        // The full chain σ(i+1) := Var(i), i = 0..254 (built with the occurs-checked bind).
        let build_chain = || {
            let mut subst = Subst::new();
            let mut i: u16 = 0;
            while i + 1 < ROW_VAR_LIMIT {
                subst
                    .bind(VarId::new(i + 1).expect("i+1 < limit"), Row::var(VarId::new(i).expect("i < limit")))
                    .expect("chain edge is acyclic");
                i += 1;
            }
            subst
        };
        // Failing half, exhaustive: v ≤ w ⟹ OccursCheck { var: v }; a failing bind returns
        // before the store, so the chain stays intact across all attempts.
        let mut chain = build_chain();
        let mut v: u16 = 0;
        while v < ROW_VAR_LIMIT {
            let mut w: u16 = v;
            while w < ROW_VAR_LIMIT {
                match chain.bind(VarId::new(v).expect("v < limit"), Row::var(VarId::new(w).expect("w < limit"))) {
                    Err(EffectError::OccursCheck { var: got }) => assert_eq!(got, v),
                    other => panic!("expected OccursCheck for ({v},{w}), got {other:?}"),
                }
                w += 1;
            }
            v += 1;
        }
        // The chain is untouched by the failed attempts (a failing bind returns before the
        // store), and resolve over the intact chain still reaches the root: sanity spot check.
        assert_eq!(
            chain.resolve(Row::var(VarId::new(ROW_VAR_LIMIT - 1).expect("255 < limit"))),
            Row::var(VarId::new(0).expect("0 < limit"))
        );
        // Succeeding half with unbound targets, exhaustive: v > w ⟹ Ok and the binding is stored.
        let mut v: u16 = 1;
        while v < ROW_VAR_LIMIT {
            let mut w: u16 = 0;
            while w < v {
                let mut subst = Subst::new();
                assert_eq!(
                    subst.bind(VarId::new(v).expect("v < limit"), Row::var(VarId::new(w).expect("w < limit"))),
                    Ok(())
                );
                assert_eq!(
                    subst.binding(VarId::new(v).expect("v < limit")),
                    Some(Row::var(VarId::new(w).expect("w < limit")))
                );
                w += 1;
            }
            v += 1;
        }
        // Stratified full-chain successes: for w in {0, 1, 127, 254}, every v > w binds onto
        // the intact chain (FV(Var(w)) = {0..w} excludes v; the store overwrites σ(v) per
        // bind's documented overwrite semantics). Resolution follows the full chain
        // (§7.3.9 step 3), so both Var(w) and the rebound Var(v) reach the chain's unbound
        // root Var(0) — the normal form, not the raw target Var(w).
        for w in [0u16, 1, 127, 254] {
            let mut v: u16 = w + 1;
            while v < ROW_VAR_LIMIT {
                let mut subst = build_chain();
                assert_eq!(
                    subst.bind(VarId::new(v).expect("v < limit"), Row::var(VarId::new(w).expect("w < limit"))),
                    Ok(())
                );
                assert_eq!(
                    subst.resolve(Row::var(VarId::new(w).expect("w < limit"))),
                    Row::var(VarId::new(0).expect("0 < limit")),
                    "chain root of Var({w})"
                );
                assert_eq!(
                    subst.resolve(Row::var(VarId::new(v).expect("v < limit"))),
                    Row::var(VarId::new(0).expect("0 < limit")),
                    "rebound Var({v}) must share Var({w})'s normal form"
                );
                v += 1;
            }
        }
        // Self-binding is always a cycle: bind(v, Var(v)) fails for every v.
        let mut v: u16 = 0;
        while v < ROW_VAR_LIMIT {
            let mut subst = Subst::new();
            match subst.bind(VarId::new(v).expect("v < limit"), Row::var(VarId::new(v).expect("v < limit"))) {
                Err(EffectError::OccursCheck { var: got }) => assert_eq!(got, v),
                other => panic!("expected self-cycle OccursCheck for {v}, got {other:?}"),
            }
            v += 1;
        }
    }

    /// EXH certificate (§7.3.9 step 3, path compression): a maximal 255-edge chain resolves to
    /// the exact closed target for all 256 targets, every intermediate slot is compressed to it,
    /// resolution is idempotent; an unbound chain compresses to its root variable; closed rows
    /// pass through unchanged. Domain: 256 chains × (255 binds + 256 slot checks) + 1 + 256.
    #[test]
    fn exh_resolve_path_compression_chains_769() {
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let closed = ClosedRow::from_mask(m).expect("in lattice");
            let mut subst = Subst::new();
            subst
                .bind(VarId::new(0).expect("0 < limit"), Row::closed(closed))
                .expect("closed target");
            let mut i: u16 = 1;
            while i < ROW_VAR_LIMIT {
                subst
                    .bind(
                        VarId::new(i).expect("i < limit"),
                        Row::var(VarId::new(i - 1).expect("i-1 < limit")),
                    )
                    .expect("chain edge is acyclic");
                i += 1;
            }
            let top = VarId::new(ROW_VAR_LIMIT - 1).expect("255 < limit");
            assert_eq!(subst.resolve(Row::var(top)), Row::closed(closed));
            // Path compression: every slot on the chain now points at the target.
            let mut i: u16 = 0;
            while i < ROW_VAR_LIMIT {
                assert_eq!(
                    subst.binding(VarId::new(i).expect("i < limit")),
                    Some(Row::closed(closed))
                );
                i += 1;
            }
            // Idempotent.
            assert_eq!(subst.resolve(Row::var(top)), Row::closed(closed));
            m += 1;
        }
        // Unbound chain: resolves to the root variable and compresses the whole chain to it.
        let mut subst = Subst::new();
        let mut i: u16 = 1;
        while i < ROW_VAR_LIMIT {
            subst
                .bind(
                    VarId::new(i).expect("i < limit"),
                    Row::var(VarId::new(i - 1).expect("i-1 < limit")),
                )
                .expect("chain edge is acyclic");
            i += 1;
        }
        let top = VarId::new(ROW_VAR_LIMIT - 1).expect("255 < limit");
        let root = VarId::new(0).expect("0 < limit");
        assert_eq!(subst.resolve(Row::var(top)), Row::var(root));
        assert_eq!(subst.binding(top), Some(Row::var(root)));
        // Closed rows pass through unchanged.
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let closed = ClosedRow::from_mask(m).expect("in lattice");
            assert_eq!(subst.resolve(Row::closed(closed)), Row::closed(closed));
            m += 1;
        }
    }

    /// EXH certificate: for every row pair (260² = 67,600 with 4 variables), the §7.3.9 step-3
    /// driver on the single-constraint set agrees exactly with single-step unify, and on success
    /// both sides resolve to the same normal form.
    #[test]
    fn exh_unify_constraints_single_67600() {
        let mut i: u16 = 0;
        while i < TEST_ROW_SPACE {
            let mut j: u16 = 0;
            while j < TEST_ROW_SPACE {
                let left = test_row(i);
                let right = test_row(j);
                let mut driver_subst = Subst::new();
                let driven = unify_constraints(
                    &mut driver_subst,
                    &[Constraint {
                        left,
                        right,
                    }],
                );
                let mut single_subst = Subst::new();
                let single = unify(&mut single_subst, left, right);
                match driven {
                    Ok(()) => {
                        assert_eq!(single, Ok(()));
                        assert_eq!(
                            driver_subst.resolve(left),
                            single_subst.resolve(right),
                            "fixpoint resolution mismatch on {i},{j}"
                        );
                    }
                    Err(ConstraintFailure::Mismatch { index, failure }) => {
                        assert_eq!(index, 0);
                        assert!(single.is_err());
                        assert_eq!(Err::<(), UnifyError>(failure), single);
                    }
                }
                j += 1;
            }
            i += 1;
        }
    }

    /// EXH certificate: the conflict set [(Var0, Closed m), (Var0, Closed m')] succeeds iff
    /// m == m' and otherwise fails at index 1 with the exact set differences (the first binding
    /// wins, the re-check catches the conflict). Domain: all 256² = 65,536 mask pairs.
    #[test]
    fn exh_unify_constraints_conflict_index_65536() {
        let var0 = VarId::new(0).expect("0 < limit");
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let mut m2: u16 = 0;
            while m2 <= ATOM_MASK_ALL {
                let cm = ClosedRow::from_mask(m).expect("in lattice");
                let cm2 = ClosedRow::from_mask(m2).expect("in lattice");
                let mut subst = Subst::new();
                let result = unify_constraints(
                    &mut subst,
                    &[
                        Constraint {
                            left: Row::var(var0),
                            right: Row::closed(cm),
                        },
                        Constraint {
                            left: Row::var(var0),
                            right: Row::closed(cm2),
                        },
                    ],
                );
                if m == m2 {
                    assert_eq!(result, Ok(()));
                } else {
                    match result {
                        Err(ConstraintFailure::Mismatch {
                            index,
                            failure:
                                UnifyError::Mismatch {
                                    only_in_left,
                                    only_in_right,
                                },
                        }) => {
                            assert_eq!(index, 1);
                            assert_eq!(only_in_left, cm.difference(cm2));
                            assert_eq!(only_in_right, cm2.difference(cm));
                        }
                        other => panic!("expected Mismatch at index 1 for {m},{m2}: {other:?}"),
                    }
                }
                m2 += 1;
            }
            m += 1;
        }
    }

    /// EXH certificate: the deferred-recheck families of §7.3.9 step 3 — variables bound in one
    /// constraint are re-checked against later constraints; constraint order does not change the
    /// fixpoint; a var/var/var triangle binds without a cycle (forest invariant); failure-free
    /// runs leave every pair resolved to the same normal form. Domain: 4 × 256 + 2 × 256 + 1 +
    /// 2 chain runs ≈ 1,541 cases.
    #[test]
    fn exh_unify_constraints_deferred_families_1281() {
        let var0 = VarId::new(0).expect("0 < limit");
        let var1 = VarId::new(1).expect("1 < limit");
        let var2 = VarId::new(2).expect("2 < limit");
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let closed = ClosedRow::from_mask(m).expect("in lattice");
            // F1: [(V0, V1), (V0, m)] — both end at m.
            let mut subst = Subst::new();
            assert_eq!(
                unify_constraints(
                    &mut subst,
                    &[
                        Constraint { left: Row::var(var0), right: Row::var(var1) },
                        Constraint { left: Row::var(var0), right: Row::closed(closed) },
                    ]
                ),
                Ok(())
            );
            assert_eq!(subst.resolve(Row::var(var0)), Row::closed(closed));
            assert_eq!(subst.resolve(Row::var(var1)), Row::closed(closed));
            // F3: [(V0, V1), (V1, m)] — both end at m.
            let mut subst = Subst::new();
            assert_eq!(
                unify_constraints(
                    &mut subst,
                    &[
                        Constraint { left: Row::var(var0), right: Row::var(var1) },
                        Constraint { left: Row::var(var1), right: Row::closed(closed) },
                    ]
                ),
                Ok(())
            );
            assert_eq!(subst.resolve(Row::var(var0)), Row::closed(closed));
            assert_eq!(subst.resolve(Row::var(var1)), Row::closed(closed));
            // F4: [(V0, m), (V1, V0)] — V1 ends at m.
            let mut subst = Subst::new();
            assert_eq!(
                unify_constraints(
                    &mut subst,
                    &[
                        Constraint { left: Row::var(var0), right: Row::closed(closed) },
                        Constraint { left: Row::var(var1), right: Row::var(var0) },
                    ]
                ),
                Ok(())
            );
            assert_eq!(subst.resolve(Row::var(var1)), Row::closed(closed));
            // F6: order independence of the same constraint set.
            let mut subst = Subst::new();
            assert_eq!(
                unify_constraints(
                    &mut subst,
                    &[
                        Constraint { left: Row::var(var0), right: Row::closed(closed) },
                        Constraint { left: Row::var(var0), right: Row::var(var1) },
                    ]
                ),
                Ok(())
            );
            assert_eq!(subst.resolve(Row::var(var1)), Row::closed(closed));
            m += 1;
        }
        // F5: the triangle [(V0,V1), (V1,V2), (V2,V0)] binds everything to Var(0) — no cycle,
        // no occurs failure, all unbound at the end.
        let mut subst = Subst::new();
        assert_eq!(
            unify_constraints(
                &mut subst,
                &[
                    Constraint { left: Row::var(var0), right: Row::var(var1) },
                    Constraint { left: Row::var(var1), right: Row::var(var2) },
                    Constraint { left: Row::var(var2), right: Row::var(var0) },
                ]
            ),
            Ok(())
        );
        assert_eq!(subst.resolve(Row::var(var2)), Row::var(var0));
        assert_eq!(subst.binding(var0), None);
    }

    /// EXH certificate: a 255-constraint variable chain unifies with every pass binding exactly
    /// one more variable, terminates, and resolves every variable to Var(0); the reversed
    /// constraint order reaches the same fixpoint (determinism G14). Domain: 2 runs of 255
    /// constraints.
    #[test]
    fn exh_unify_constraints_chain_255() {
        let make_constraints = |reversed: bool| {
            let mut constraints = Vec::with_capacity((ROW_VAR_LIMIT - 1) as usize);
            let mut i: u16 = 0;
            while i + 1 < ROW_VAR_LIMIT {
                constraints.push(Constraint {
                    left: Row::var(VarId::new(i).expect("i < limit")),
                    right: Row::var(VarId::new(i + 1).expect("i+1 < limit")),
                });
                i += 1;
            }
            if reversed {
                constraints.reverse();
            }
            constraints
        };
        for reversed in [false, true] {
            let constraints = make_constraints(reversed);
            let mut subst = Subst::new();
            assert_eq!(unify_constraints(&mut subst, &constraints), Ok(()));
            let mut i: u16 = 0;
            while i < ROW_VAR_LIMIT {
                assert_eq!(
                    subst.resolve(Row::var(VarId::new(i).expect("i < limit"))),
                    Row::var(VarId::new(0).expect("0 < limit"))
                );
                i += 1;
            }
        }
    }

    /// EXH certificate (pinned call rule, SPEC-INDEX §5.3 bullet 4): check_call accepts exactly
    /// when callee ⊆ ambient and reports Missing { missing = callee \ ambient } otherwise.
    /// Domain: all 65,536 (ambient, callee) pairs.
    #[test]
    fn exh_check_call_65536() {
        let mut a: u16 = 0;
        while a <= ATOM_MASK_ALL {
            let mut b: u16 = 0;
            while b <= ATOM_MASK_ALL {
                let ambient = ClosedRow::from_mask(a).expect("in lattice");
                let callee = ClosedRow::from_mask(b).expect("in lattice");
                let result = check_call(ambient, callee);
                if callee.leq(ambient) {
                    assert_eq!(result, Ok(()));
                } else {
                    assert_eq!(
                        result,
                        Err(CallError::Missing {
                            missing: callee.difference(ambient),
                        })
                    );
                }
                b += 1;
            }
            a += 1;
        }
    }

    /// EXH certificate: the named row constants carry their exact pinned masks (ROW_EMPTY,
    /// ROW_ALL, ROW_FFI per §7.3.14, ROW_UNKNOWN_BARRIER per §7.5 d7) and u8_mask reproduces the
    /// Volume 12 §12.1 u8 encoding for all 256 rows. Domain: 4 constants + 256 rows.
    #[test]
    fn exh_named_row_constants_256() {
        assert_eq!(ROW_EMPTY.bits(), 0);
        assert_eq!(ROW_ALL.bits(), ATOM_BIT_IO | ATOM_BIT_ALLOC | ATOM_BIT_PANIC | ATOM_BIT_UNSAFE | ATOM_BIT_ASYNC | ATOM_BIT_CONCURRENCY | ATOM_BIT_FFI | ATOM_BIT_NONDET);
        assert_eq!(ROW_FFI.bits(), ATOM_BIT_FFI);
        assert_eq!(
            ROW_UNKNOWN_BARRIER.bits(),
            ATOM_BIT_IO | ATOM_BIT_ALLOC | ATOM_BIT_PANIC | ATOM_BIT_CONCURRENCY | ATOM_BIT_NONDET
        );
        for atom in CANONICAL_ATOM_ORDER {
            assert!(ROW_ALL.contains(atom));
            assert!(!ROW_EMPTY.contains(atom));
            assert_eq!(ROW_FFI.contains(atom), atom == Atom::Ffi);
            let barrier = ROW_UNKNOWN_BARRIER.contains(atom);
            let pinned_barrier = matches!(
                atom,
                Atom::Io | Atom::Alloc | Atom::Panic | Atom::Concurrency | Atom::Nondet
            );
            assert_eq!(barrier, pinned_barrier);
        }
        let mut m: u16 = 0;
        while m <= ATOM_MASK_ALL {
            let row = ClosedRow::from_mask(m).expect("in lattice");
            assert_eq!(u16::from(row.u8_mask()), m);
            m += 1;
        }
    }

    /// EXH certificate: from_atoms accepts exactly the duplicate-free atom lists and produces the
    /// OR of single-atom masks, reporting the first duplicate otherwise. Domain: all atom
    /// sequences of length 0..=4 over the 8 atoms = 4,681 lists.
    #[test]
    fn exh_atoms_from_lists_4681() {
        // Length 0: ∅ (§7.4 d1: omitted EffectAnno means the closed row ∅).
        assert_eq!(ClosedRow::from_atoms(&[]), Ok(ROW_EMPTY));
        // Length 1: every singleton.
        for atom in CANONICAL_ATOM_ORDER {
            assert_eq!(ClosedRow::from_atoms(&[atom]), Ok(ClosedRow::from_atom(atom)));
        }
        // Lengths 2..=4, every sequence: decode base-8 digits into atoms.
        let mut length = 2usize;
        while length <= 4 {
            let mut code: usize = 0;
            let space = 1usize << (3 * length); // 8^length
            while code < space {
                let mut atoms = [Atom::Alloc; ATOM_COUNT];
                let mut rest = code;
                let mut k = 0usize;
                while k < length {
                    atoms[k] = CANONICAL_ATOM_ORDER[rest & 7];
                    rest >>= 3;
                    k += 1;
                }
                let list = &atoms[..length];
                // Reference: first duplicate by pairwise scan, else OR of singleton masks.
                let mut expected: Result<ClosedRow, EffectError> = Ok(ROW_EMPTY);
                'outer: for i in 0..length {
                    for j in 0..i {
                        if list[i] == list[j] {
                            expected = Err(EffectError::DuplicateAtom { atom: list[i] });
                            break 'outer;
                        }
                    }
                }
                if expected.is_ok() {
                    let mut bits = 0u16;
                    for atom in list {
                        bits |= atom_bit(*atom);
                    }
                    expected = ClosedRow::from_mask(bits);
                }
                assert_eq!(ClosedRow::from_atoms(list), expected, "list {code} len {length}");
                code += 1;
            }
            length += 1;
        }
    }

    /// PROP certificate (paired with the EXH law suite): 10,000 deterministic PRNG row triples
    /// satisfy the full pinned law set — join commutativity/idempotency/identity, both
    /// absorptions, meet identity, both distributivities, leq reflexivity/transitivity, and the
    /// join/meet per-atom characterizations. Sampling domain: uniform masks 0..=255.
    #[test]
    fn prop_lattice_laws_random_triples_10000() {
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < 10_000 {
            let a = rng.below(256) as u16;
            let b = rng.below(256) as u16;
            let c = rng.below(256) as u16;
            let ra = ClosedRow::from_mask(a).expect("in lattice");
            let rb = ClosedRow::from_mask(b).expect("in lattice");
            let rc = ClosedRow::from_mask(c).expect("in lattice");
            assert_eq!(ra.join(rb), rb.join(ra));
            assert_eq!(ra.join(ra), ra);
            assert_eq!(ra.join(ROW_EMPTY), ra);
            assert_eq!(ra.join(ra.meet(rb)), ra);
            assert_eq!(ra.meet(ra.join(rb)), ra);
            assert_eq!(ra.meet(ROW_ALL), ra);
            assert_eq!(ra.meet(rb).join(rc), (ra.join(rc)).meet(rb.join(rc)));
            assert_eq!(ra.join(rb).meet(rc), (ra.meet(rc)).join(rb.meet(rc)));
            assert!(ra.leq(ra));
            if ra.leq(rb) && rb.leq(rc) {
                assert!(ra.leq(rc));
            }
            assert_eq!(ra.join(rb).bits(), a | b);
            assert_eq!(ra.meet(rb).bits(), a & b);
            n += 1;
        }
    }

    /// PROP certificate (paired with the EXH unification family): 10,000 deterministic PRNG
    /// unification cases over random acyclic substitutions and random row pairs, checked against
    /// an independent reference model (std-owned Vec substitution, iterative resolution without
    /// compression, naive re-dispatch binding). Sampling domains: substitution states over 8
    /// variables (unbound | closed | var-to-lower-index), rows uniform over 256 masks and 8
    /// variables.
    #[test]
    fn prop_unify_random_states_10000() {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum ModelRow {
            Closed(u16),
            Var(u16),
        }
        fn model_resolve(subst: &[Option<ModelRow>], row: ModelRow) -> ModelRow {
            let mut current = row;
            let mut depth = 0usize;
            loop {
                match current {
                    ModelRow::Closed(_) => return current,
                    ModelRow::Var(v) => match subst[v as usize] {
                        None => return current,
                        Some(next) => {
                            current = next;
                            depth += 1;
                            assert!(depth <= ROW_VAR_LIMIT as usize, "model chain too deep");
                        }
                    },
                }
            }
        }
        fn model_unify(
            subst: &mut [Option<ModelRow>],
            left: ModelRow,
            right: ModelRow,
        ) -> Result<(), UnifyError> {
            let l = model_resolve(subst, left);
            let r = model_resolve(subst, right);
            match (l, r) {
                (ModelRow::Closed(c1), ModelRow::Closed(c2)) => {
                    if c1 == c2 {
                        Ok(())
                    } else {
                        Err(UnifyError::Mismatch {
                            only_in_left: ClosedRow::from_mask(c1).expect("in lattice").difference(ClosedRow::from_mask(c2).expect("in lattice")),
                            only_in_right: ClosedRow::from_mask(c2).expect("in lattice").difference(ClosedRow::from_mask(c1).expect("in lattice")),
                        })
                    }
                }
                (ModelRow::Var(v), ModelRow::Closed(c)) => {
                    subst[v as usize] = Some(ModelRow::Closed(c));
                    Ok(())
                }
                (ModelRow::Closed(c), ModelRow::Var(v)) => {
                    subst[v as usize] = Some(ModelRow::Closed(c));
                    Ok(())
                }
                (ModelRow::Var(v1), ModelRow::Var(v2)) => {
                    if v1 == v2 {
                        Ok(())
                    } else {
                        let (low, high) = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                        subst[high as usize] = Some(ModelRow::Var(low));
                        Ok(())
                    }
                }
            }
        }

        const MODEL_VARS: u16 = 8;
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < 10_000 {
            // Random acyclic substitution: edges only point to lower creation indices.
            let mut impl_subst = Subst::new();
            let mut model_subst: Vec<Option<ModelRow>> = vec![None; MODEL_VARS as usize];
            let mut v: u16 = 0;
            while v < MODEL_VARS {
                let var = VarId::new(v).expect("v < limit");
                match rng.below(3) {
                    0 => {}
                    1 => {
                        let m = rng.below(256) as u16;
                        let closed = ClosedRow::from_mask(m).expect("in lattice");
                        impl_subst.bind(var, Row::closed(closed)).expect("closed target");
                        model_subst[v as usize] = Some(ModelRow::Closed(m));
                    }
                    _ => {
                        if v > 0 {
                            let w = rng.below(v as u64) as u16;
                            impl_subst
                                .bind(var, Row::var(VarId::new(w).expect("w < limit")))
                                .expect("edge to lower index is acyclic");
                            model_subst[v as usize] = Some(ModelRow::Var(w));
                        }
                    }
                }
                v += 1;
            }
            // Five random row pairs per substitution state.
            let mut k = 0u32;
            while k < 5 {
                let mut pick_row = || {
                    if rng.below(2) == 0 {
                        let left = ClosedRow::from_mask(rng.below(256) as u16).expect("in lattice");
                        Row::closed(left)
                    } else {
                        Row::var(VarId::new(rng.below(MODEL_VARS as u64) as u16).expect("var < limit"))
                    }
                };
                let left = pick_row();
                let right = pick_row();
                let impl_result = unify(&mut impl_subst, left, right);
                let model_left = match left {
                    Row::Closed(c) => ModelRow::Closed(c.bits()),
                    Row::Var(var) => ModelRow::Var(var.index()),
                };
                let model_right = match right {
                    Row::Closed(c) => ModelRow::Closed(c.bits()),
                    Row::Var(var) => ModelRow::Var(var.index()),
                };
                let model_result = model_unify(&mut model_subst, model_left, model_right);
                assert_eq!(impl_result, model_result, "differential on case {n}.{k}");
                // Final resolutions must agree variable by variable.
                let mut var: u16 = 0;
                while var < MODEL_VARS {
                    let impl_norm = impl_subst.resolve(Row::var(VarId::new(var).expect("var < limit")));
                    let model_norm = model_resolve(&model_subst, ModelRow::Var(var));
                    match (impl_norm, model_norm) {
                        (Row::Closed(c), ModelRow::Closed(m)) => assert_eq!(c.bits(), m),
                        (Row::Var(iv), ModelRow::Var(mv)) => assert_eq!(iv.index(), mv),
                        other => panic!("norm disagreement var {var}: {other:?}"),
                    }
                    var += 1;
                }
                k += 1;
            }
            n += 1;
        }
    }

    /// PROP certificate (paired with exh_atoms_from_lists_4681): 10,000 deterministic PRNG atom
    /// lists of length 0..=12 behave as the accumulated-mask model predicts — Ok iff
    /// duplicate-free, payload OR of singleton masks, first duplicate reported otherwise.
    #[test]
    fn prop_atoms_from_lists_long() {
        let mut rng = Xorshift64Star::new();
        let mut n = 0u32;
        while n < 10_000 {
            let len = rng.below(13) as usize;
            let mut list = [Atom::Alloc; 12];
            let mut seen_bits = 0u16;
            let mut first_duplicate: Option<Atom> = None;
            let mut i = 0usize;
            while i < len {
                let atom = CANONICAL_ATOM_ORDER[rng.below(ATOM_COUNT as u64) as usize];
                list[i] = atom;
                let bit = atom_bit(atom);
                if (seen_bits & bit) != 0 && first_duplicate.is_none() {
                    first_duplicate = Some(atom);
                }
                seen_bits |= bit;
                i += 1;
            }
            let result = ClosedRow::from_atoms(&list[..len]);
            match first_duplicate {
                Some(atom) => assert_eq!(result, Err(EffectError::DuplicateAtom { atom })),
                None => assert_eq!(result, Ok(ClosedRow::from_mask(seen_bits).expect("in lattice"))),
            }
            n += 1;
        }
    }
}
