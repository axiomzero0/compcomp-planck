// CEP:FILE: math/crates/planck-regalloc-math/src/lib.rs
// CEP:WHAT: Pinned register-allocation math of Planck 0.1 edition P1 (Volume 16 §16.4, pinned by SPEC-INDEX §5.8), embedded verbatim as a no_std kernel crate: half-open live-interval overlap, coalescibility-table-adjusted interference, max-simultaneous-live register pressure, the checked-u64 spill priority `uses × 10^3 + loop_depth × 10^6 − size_cost`, and the pinned coloring pipeline (degree-ordered First-Fit over a caller-provided register class order with an ascending-priority spill loop) together with the two pinned allocation laws (exactly-one-location; no-two-interfering-share).
// CEP:WHY: SPEC-INDEX §5.1 pins planck-regalloc-math as the extraction home of the §5.8 register-allocation math; every Volume 16 §16.4 rule (RA2 intervals, RA4 classes, RA6 pipeline, RA7 spill decisions, RA13 fixed registers, RA15 verification) refines exactly these functions, so the backend must call exactly these functions and no others.
// CEP:STATUS: complete
// CEP:FAILURE: A `RegAllocError` on any input outside the enumerated failure set, a law violation on any accepted allocation, or a divergence between the implementation and the pinned §5.8/§16.4 definitions on any FV-enumerated instance is non-conforming; the EXH/PROP tests fail and the crate must not ship. Every failure returns an explicit error enum; the complete variant sets are: `RegAllocError` = {TooManyIntervals, OutputBufferTooSmall, TooManyRegisters, DuplicateRegister, InvalidInterval, UsesOutOfRange, LoopDepthOutOfRange, SizeCostOutOfRange, DuplicateVreg, TooManyCoalescePairs, CoalesceSelfPair, CoalesceVregUnknown, SpillImpossible, SpillPriorityOverflow, SpillPriorityUnderflow} and `SpillPriorityError` = {Overflow, Underflow}. No other failure mode exists; no function panics.
// CEP:ASSUMES: SPEC-INDEX §5.8 (pinned contract, embedded verbatim in Volume 16 §16.4); Volume 16 §16.4 RA2 (half-open intervals, u32 positions), RA4 (available = 14/28 GPR, 16 XMM, 32 V128; the class First-Fit order is caller-provided per §16.2.1/§16.2.2/§16.2.3 — the crate embeds no target), RA6 (degree-ordered First-Fit, tie-breaks degree desc / start asc / interval ID asc), RA7 (checked u64 priority, domain bounds uses ≤ 2^20, depth ≤ 64, size ≤ 64, underflow/overflow = E-RA-01, ascending benefit = ascending priority, tie-break ascending vreg), RA9 (coalescibility is extensional — the kernel takes the caller-provided predicate table as given, including its asymmetry; RA9's "neither fixed" table-construction rule and move scanning are upstream), RA13 (fixed/pre-colored intervals are never spilled), OB-16-7 (the spill loop terminates with pressure ≤ available or all non-fixed intervals spilled; fixed ≤ available is the caller invariant, violation surfaces as SpillImpossible); Volume 12 §12.9 limits table has no max-intervals-per-function row (nearest: L13 = 2^24 IR nodes per function, which no fixed-capacity kernel can size for), so the kernel working capacity is the named constant MAX_INTERVALS = 4096 with TooManyIntervals as the defined rejection and the RA15 verifier + RA6 pipeline as the runtime bridge beyond capacity; MAX_REGS = 32 = the largest §16.2 class (V128 = 32, §16.2.2); SPEC-INDEX §5.2 (kernel implementation law) and §5.1 (equivalence claim form); SPEC-INDEX §4.3 (PROP pairing).
// CEP:COST: overlap/coalescible/interfere/spill_priority O(1)/O(P)/O(P)/O(1); register_pressure O(n²); allocate worst case O(n²·log P) per coloring pass, spill loop ≤ n+1 passes → O(n³·log P) at the MAX_INTERVALS = 4096 capacity (the FV suite exercises n ≤ 64; the production pipeline per RA6 uses adjacency structures and the RA15 verifier bridges the gap). All stack working storage is fixed-capacity (~93 KiB); zero allocation, zero external dependencies.
// CEP:EVIDENCE: EXH exh_overlap_halfopen_grid_441 (all 441 shape pairs on the 6-point grid vs the independent set-intersection reference); EXH exh_interference_formula_cases (symmetric and asymmetric coalescence table cases, degenerate intervals); EXH exh_allocation_laws_grid_multisets (698,720 instances = all multisets of ≤ 5 intervals over the 21 interval shapes of the 6-point grid, paired with the pinned coalescibility-configuration rule of the census, both laws asserted on every allocator output); EXH exh_fixed_intervals_spill_paths (26,222 instances exercising fixed/unspillable intervals, Ok and SpillImpossible outcomes); EXH exh_register_pressure_grid_multisets_2024 (pressure vs independent event-sweep reference); EXH exh_spill_priority_corner_grid_27 (all 27 corner combinations of {0, 1, max} per OB-16-6, exact checked results); EXH exh_allocate_known_instances (hand-computed allocations incl. the RA7 pressure-counts-intervals rule); EXH exh_error_variants_reachable; EXH exh_empty_input_totality; PROP prop_laws_larger_instances (5,000 deterministic xorshift64* instances ≤ 64 intervals, paired with the EXH census and the Z3 check per SPEC-INDEX §4.3); Z3 tools/fv/z3/regalloc_laws.smt2 (concrete bounded instance: 5 intervals, 4 registers, half-open interference from this census; plus a 1-register non-vacuity case; artifact tools/fv/z3/artifacts/regalloc_laws.result, kernel-probe-verified).
// CEP:FV-CLASS: FV-1
// CEP:FV-PROVER: Rust test harness (EXH over the enumerated grid census + PROP) and Z3 5.4.1 (tools/fv/z3/regalloc_laws.smt2, self-contained SMT-LIB2)
// CEP:FV-SCRIPT: crates/planck-regalloc-math/src/lib.rs (#[cfg(test)] mod tests); tools/fv/z3/regalloc_laws.smt2; artifact tools/fv/z3/artifacts/regalloc_laws.result
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: FV-1 with bounded-instance gap, stated exactly: the pinned laws are machine-checked EXH-style over ALL interval multisets of size ≤ 5 on the 6-point grid (start/end ∈ 0..=5, 724,942 allocate() instances in total across the two censuses), Z3 on a concrete bounded instance from that census (5 intervals, 4 registers, half-open interference relation) plus a 1-register non-vacuity case, and PROP-style over 5,000 deterministic instances of ≤ 64 intervals. Beyond those bounds (larger interval counts, wider position grids, richer constraint systems) the laws are not machine-checked here; the runtime bridge is the RA15 allocation verifier (Volume 16 §16.4 RA15), which re-checks both laws (V1 exactly-one-location, V2 no-two-interfering-share) on every compilation before the backend proceeds. spill_priority itself is FV-0 over the OB-16-6 corner domain (complete) with the stated bound gap beyond {uses ≤ 2^20, depth ≤ 64, size ≤ 64}.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named capacity and scale constants (Law 7: constants are named, never magic
// literals). Capacities are derived from Volume 16/12 limits as documented.
// ---------------------------------------------------------------------------

/// Kernel working capacity: maximum intervals per `allocate` call (4096).
///
/// Volume 12 §12.9 (limits table) defines no max-intervals-per-function row; the
/// nearest row is L13 (max IR nodes per function = 2^24), which no fixed-capacity
/// kernel array can size for. MAX_INTERVALS = 4096 is therefore the pinned kernel
/// capacity: exceeding it is the defined error `TooManyIntervals`, and the runtime
/// bridge beyond capacity is the RA6 production pipeline + RA15 verifier
/// (Volume 16 §16.4).
pub const MAX_INTERVALS: usize = 4096;

/// Maximum physical registers in a caller-provided class order (32).
///
/// Derived from Volume 16 §16.2: the largest class of any declared target is
/// AArch64 `V128` with available = 32 (§16.2.2); x86-64 GPR available = 14 and
/// XMM = 16 (§16.2.1); RV64 GPR available = 28 (§16.2.3, declared). Exceeding it
/// is the defined error `TooManyRegisters`.
pub const MAX_REGS: usize = 32;

/// Maximum coalescibility pairs in the caller-provided predicate table (4096).
///
/// The table is the extensional encoding of the pinned "caller-provided predicate
/// table" of SPEC-INDEX §5.8 (Volume 16 §16.4 RA9 gives coalescibility as a set of
/// move pairs). Exceeding it is the defined error `TooManyCoalescePairs`.
pub const MAX_COALESCE_PAIRS: usize = 4096;

/// Spill-priority uses scale: exactly 10^3 (SPEC-INDEX §5.8; Volume 16 §16.4 RA7).
pub const SPILL_PRIORITY_USES_SCALE: u64 = 1_000;

/// Spill-priority loop-depth scale: exactly 10^6 (SPEC-INDEX §5.8; Volume 16 §16.4 RA7).
pub const SPILL_PRIORITY_DEPTH_SCALE: u64 = 1_000_000;

/// Domain bound on `uses` per interval: 2^20 (Volume 16 §16.4 RA7).
pub const MAX_USES_PER_INTERVAL: u64 = 1_048_576;

/// Domain bound on `loop_depth` per interval: 64 (Volume 16 §16.4 RA7).
pub const MAX_LOOP_DEPTH: u64 = 64;

/// Domain bound on `size_cost` per interval: 64 (Volume 16 §16.4 RA7,
/// `align_up(width_bytes, 8)` of a machine value ≤ 64 bytes wide).
pub const MAX_SIZE_COST: u64 = 64;

// Compile-time structural assumptions (fail the build, never the runtime).
const _: () = assert!(MAX_INTERVALS % 64 == 0, "spill bitset words must tile MAX_INTERVALS");
const _: () = assert!(MAX_REGS <= 32, "First-Fit busy mask is a u32 over order positions");

/// Sentinel order-position marking "no register assigned this pass".
const POS_UNASSIGNED: u8 = 0xFF;

// ---------------------------------------------------------------------------
// Types.
// ---------------------------------------------------------------------------

/// One pinned live interval `(v, start, end)` with half-open range `[start, end)`
/// (SPEC-INDEX §5.8; Volume 16 §16.4 RA2: `start` = defining-write position,
/// `end` = last-use position + 1; positions are u32 in block layout order).
///
/// The spill-decision metadata of RA7 (`uses`, `loop_depth`, `size_cost`) and the
/// RA13 fixed flag are carried per interval. Domain (enforced by `allocate`):
/// `start ≤ end`, `uses ≤ 2^20`, `loop_depth ≤ 64`, `size_cost ≤ 64`, unique `vreg`.
///
/// CEP:WHAT: Pinned live-interval record with RA7 spill metadata and RA13 fixed flag.
/// CEP:WHY: The §5.8 contract defines every law over exactly these fields; the kernel takes them as input and never recomputes them (liveness/splitting are RA1–RA3, upstream).
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data record); field validity is enforced by `allocate` and `register_pressure` per their CEP:FAILURE sets.
/// CEP:ASSUMES: `fixed` models RA13 fixed/pre-colored intervals at the §5.8 abstraction level: the kernel guarantees a fixed interval is never spilled and never displaced; the concrete pre-assignment is realized by the caller through input preparation and the caller-provided class order (the kernel embeds no target).
/// CEP:COST: 24 bytes (Copy); zero runtime cost.
/// CEP:EVIDENCE: EXH exh_allocation_laws_grid_multisets; EXH exh_fixed_intervals_spill_paths.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LiveInterval {
    /// Vreg/interval identity; unique within one allocation problem (RA6/RA7 tie-breaks are defined over it).
    pub vreg: u32,
    /// Half-open range start (defining-write position, RA2); `start ≤ end` required.
    pub start: u32,
    /// Half-open range end (last-use position + 1, RA2).
    pub end: u32,
    /// Use positions strictly inside `[start, end)` (RA7); ≤ 2^20.
    pub uses: u32,
    /// Loop nesting depth of the block containing `start` (RA7); ≤ 64.
    pub loop_depth: u32,
    /// Spill slot bytes `align_up(width_bytes, 8)` (RA7); ≤ 64.
    pub size_cost: u32,
    /// RA13 fixed/pre-colored: never spilled, never displaced by the spill loop.
    pub fixed: bool,
}

/// One ordered coalescibility entry `(a, b)`: interval with vreg `a` is
/// coalescible with interval with vreg `b` in that direction.
///
/// The pinned interference law reads the table directionally ("at least one is
/// not coalescible with the other", SPEC-INDEX §5.8), so a symmetric relation
/// must be listed in both directions. RA9's construction rule (move scan, same
/// class, neither fixed) is upstream table construction; the kernel takes the
/// table as given.
///
/// CEP:WHAT: Directional coalescibility table entry over vreg ids.
/// CEP:WHY: The §5.8 interference definition subtracts coalescibility from overlap; the table is the only channel through which the caller supplies it.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data record); `a == b` and unknown endpoints are rejected by `allocate` (`CoalesceSelfPair`, `CoalesceVregUnknown`).
/// CEP:ASSUMES: vreg ids of `a`/`b` refer to intervals of the same allocation problem.
/// CEP:COST: 8 bytes (Copy).
/// CEP:EVIDENCE: EXH exh_interference_formula_cases; EXH exh_allocation_laws_grid_multisets.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CoalescePair {
    /// Coalescible-from vreg id.
    pub a: u32,
    /// Coalescible-to vreg id.
    pub b: u32,
}

/// The complete input of one allocation problem (one register class per call;
/// the pinned §5.8 loop is per class, RA4/RA7).
///
/// CEP:WHAT: Caller-provided allocation problem: intervals, coalescibility predicate table, register class First-Fit order.
/// CEP:WHY: SPEC-INDEX §5.8 takes interference (via the table) and the class order as caller-provided; the crate embeds no target.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data record); every field constraint is enforced by `allocate` per its CEP:FAILURE set.
/// CEP:ASSUMES: `class_order` lists distinct physical-register encodings in the target's pinned First-Fit order (Volume 16 §16.2.1/§16.2.2 T5 rows); its length is the available-register count.
/// CEP:COST: three references; zero runtime cost.
/// CEP:EVIDENCE: EXH exh_allocation_laws_grid_multisets; EXH exh_allocate_known_instances.
#[derive(Debug)]
pub struct RegAllocProblem<'a> {
    /// Live intervals of the class; `len ≤ MAX_INTERVALS`, unique vregs.
    pub intervals: &'a [LiveInterval],
    /// Coalescibility predicate table (ordered pairs, extensional); `len ≤ MAX_COALESCE_PAIRS`.
    pub coalesce: &'a [CoalescePair],
    /// Register class First-Fit order (distinct encodings); `len ≤ MAX_REGS`; length = available registers.
    pub class_order: &'a [u16],
}

/// One assigned location: a physical register from the class order or a private
/// spill slot (SPEC-INDEX §5.8 law 1: every interval gets exactly one location).
///
/// CEP:WHAT: Allocation location = Register(encoding) | Slot(index).
/// CEP:WHY: The pinned laws are stated over locations; spill slots are first-class so V1 holds for spilled intervals.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data value).
/// CEP:ASSUMES: Slot index = spill-decision sequence number (0, 1, 2, …) in this call; the frame-layout placement of Volume 16 §16.4 RA8 / §16.6 FR3 is downstream. Each spilled interval owns exactly one slot; slots are never shared.
/// CEP:COST: 8 bytes (Copy).
/// CEP:EVIDENCE: EXH exh_allocation_laws_grid_multisets (slot distinctness and range asserted per instance).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Location {
    /// A physical register encoding taken from the caller-provided class order.
    Register(u16),
    /// A private spill slot of this allocation call (RA7/RA8; index < spilled count).
    Slot(u32),
}

/// Statistics of a successful allocation.
///
/// CEP:WHAT: assigned/spilled counts and the final RA7 pressure of the accepted allocation.
/// CEP:WHY: The spill-loop exit condition (pressure ≤ available) is part of the pinned contract; the stats expose it for verification.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data value).
/// CEP:ASSUMES: `assigned + spilled == interval count`; `final_pressure ≤ class_order.len()`; RA7 pressure counts register-assigned intervals covering a point (coalescible overlapping intervals sharing a register still count separately — RA9 merges them into one interval upstream, so the kernel applies RA7 literally).
/// CEP:COST: 12 bytes (Copy).
/// CEP:EVIDENCE: EXH exh_allocation_laws_grid_multisets (pressure ≤ available asserted per instance).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RegAllocStats {
    /// Intervals assigned a register.
    pub assigned: u32,
    /// Intervals assigned a private spill slot.
    pub spilled: u32,
    /// Final max count of register-assigned intervals covering one position (RA7 pressure).
    pub final_pressure: u32,
}

/// Complete error set of `allocate` and `register_pressure` (every variant listed;
/// no other failure mode exists; nothing panics).
///
/// CEP:WHAT: Complete allocation error enum (input-domain violations + spill-loop outcomes + RA7 checked-math outcomes).
/// CEP:WHY: SPEC-INDEX §5.2: every failure returns an explicit complete error enum; Volume 16 §16.4 RA7 names E-RA-01 (SpillPriorityUnderflow/SpillPriorityOverflow) and OB-16-7 fixes the spill-impossible arm.
/// CEP:STATUS: complete
/// CEP:FAILURE: The variants themselves are the enumerated failure set (see crate CEP:FAILURE). `SpillPriorityOverflow` is unreachable inside `allocate` within the validated domain (max priority = 2^20·10^3 + 64·10^6 = 1,112,576,000 < 2^64) but is retained because RA7 mandates checked math and E-RA-01; it is reachable through `spill_priority`'s full u64 domain.
/// CEP:ASSUMES: Validation order is fixed and deterministic: buffer/count capacities, class order, per-interval fields, vreg uniqueness, coalescibility table; the first violated rule is returned.
/// CEP:COST: 1 byte discriminant; zero runtime cost.
/// CEP:EVIDENCE: EXH exh_error_variants_reachable; EXH exh_fixed_intervals_spill_paths (SpillImpossible); EXH exh_spill_priority_corner_grid_27.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RegAllocError {
    /// More than `MAX_INTERVALS` intervals (kernel capacity; Volume 12 §12.9 has no per-function interval row).
    TooManyIntervals,
    /// Output buffer shorter than the interval count.
    OutputBufferTooSmall,
    /// Class order longer than `MAX_REGS` (Volume 16 §16.2 maximum class = 32).
    TooManyRegisters,
    /// A physical-register encoding appears twice in the class order.
    DuplicateRegister,
    /// An interval has `start > end` (violates the pinned `start ≤ end`, §5.8).
    InvalidInterval,
    /// An interval has `uses > 2^20` (RA7 domain bound).
    UsesOutOfRange,
    /// An interval has `loop_depth > 64` (RA7 domain bound).
    LoopDepthOutOfRange,
    /// An interval has `size_cost > 64` (RA7 domain bound).
    SizeCostOutOfRange,
    /// Two intervals share a vreg id (interval identity must be total for the RA6/RA7 tie-breaks).
    DuplicateVreg,
    /// More than `MAX_COALESCE_PAIRS` coalescibility pairs.
    TooManyCoalescePairs,
    /// A coalescibility pair has `a == b` (RA9 moves are between distinct vregs).
    CoalesceSelfPair,
    /// A coalescibility pair endpoint is not a vreg of the problem.
    CoalesceVregUnknown,
    /// Spill loop exhausted: every non-fixed interval is spilled and the remaining fixed intervals still cannot satisfy pressure ≤ available with all laws (OB-16-7 arm 2; RA13 fixed are never spilled). RA15 turns this into an ICE in the compiler; here it is a defined error.
    SpillImpossible,
    /// RA7 checked priority math overflowed u64 (E-RA-01).
    SpillPriorityOverflow,
    /// RA7 checked priority math underflowed below 0 (E-RA-01).
    SpillPriorityUnderflow,
}

/// Complete error set of `spill_priority` (Volume 16 §16.4 RA7, E-RA-01 variants).
///
/// CEP:WHAT: Checked-u64 spill-priority outcome errors.
/// CEP:WHY: RA7 mandates checked multiply/add/sub with named errors SpillPriorityOverflow/SpillPriorityUnderflow.
/// CEP:STATUS: complete
/// CEP:FAILURE: {Overflow: the positive terms exceeded u64::MAX; Underflow: uses·10^3 + depth·10^6 < size_cost so the u64 result would be negative}. No other variant.
/// CEP:ASSUMES: full u64 inputs (the fn is the checked-math witness; `allocate` additionally enforces the RA7 domain bounds 2^20/64/64 before calling it).
/// CEP:COST: O(1), 2 multiplications + 2 additions/subtractions, all checked.
/// CEP:EVIDENCE: EXH exh_spill_priority_corner_grid_27 (all 27 corner combinations, exact).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpillPriorityError {
    /// `uses × 10^3 + loop_depth × 10^6` exceeded `u64::MAX`.
    Overflow,
    /// `uses × 10^3 + loop_depth × 10^6 − size_cost` is negative; cannot be represented in u64.
    Underflow,
}

// ---------------------------------------------------------------------------
// Pinned pure predicates (CEP-0).
// ---------------------------------------------------------------------------

/// Half-open interval overlap: do `[a.start, a.end)` and `[b.start, b.end)`
/// intersect as position sets?
///
/// Definition anchor: SPEC-INDEX §5.8 ("two intervals interfere iff they
/// overlap") + Volume 16 §16.4 RA2 (half-open `[start, end)`). The exact pinned
/// reading of "overlap" for half-open ranges is non-empty set intersection:
/// `max(a.start, b.start) < min(a.end, b.end)`. This is exact also for
/// degenerate intervals `[p, p)` (RA2 never produces them, but the §5.8 domain
/// `start ≤ end` admits them): a degenerate interval is the empty position set
/// and overlaps nothing.
/// Input domain: total over all u32 starts/ends; the pinned semantics above hold
/// for every pair with `start ≤ end` in both intervals (all of §5.8's domain).
/// Failure set: none (pure predicate).
/// Certificate: EXH `exh_overlap_halfopen_grid_441` — all 21×21 = 441 shape pairs
/// of the 6-point grid (start/end ∈ 0..=5) against the independent existential
/// reference "∃p: both contain p"; exercised on every census instance.
///
/// CEP:WHAT: Pinned half-open overlap predicate (non-empty intersection).
/// CEP:WHY: Interference (§5.8) is defined from exactly this overlap; the half-open reading is what lets a value die at p and a new value be born at p without interfering (RA2).
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; grid-exhaustive against the set-intersection reference.
/// CEP:ASSUMES: positions are discrete u32; intervals are half-open per RA2.
/// CEP:COST: O(1), 2 min/max + 1 comparison, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_overlap_halfopen_grid_441; EXH exh_allocation_laws_grid_multisets; PROP prop_laws_larger_instances.
#[inline]
pub fn overlap(a: LiveInterval, b: LiveInterval) -> bool {
    let lo = if a.start >= b.start { a.start } else { b.start };
    let hi = if a.end <= b.end { a.end } else { b.end };
    lo < hi
}

/// Coalescibility lookup in the caller-provided predicate table: is the
/// interval `a` (vreg `a.vreg`) coalescible with `b` (vreg `b.vreg`) in the
/// a→b direction?
///
/// Definition anchor: SPEC-INDEX §5.8 ("coalescibility given by a
/// caller-provided predicate table") + Volume 16 §16.4 RA9 (coalescibility is
/// extensional — the set of move pairs). `coalescible(table, a, b)` holds iff
/// the table contains the ordered pair `(a.vreg, b.vreg)`. A symmetric relation
/// must be listed in both directions (RA9 pairs are directional by move
/// position).
/// Input domain: total over any table slice and any two intervals (linear scan;
/// `allocate` enforces `MAX_COALESCE_PAIRS`, endpoint validity, and `a ≠ b` for
/// its own table use).
/// Failure set: none (pure predicate).
/// Certificate: EXH `exh_interference_formula_cases` (directional cases) and the
/// full census `exh_allocation_laws_grid_multisets` (tables of ≤ 2 ordered pairs
/// for sizes ≤ 3, symmetric single pairs for sizes 4–5).
///
/// CEP:WHAT: Directional coalescibility predicate over the caller-provided table.
/// CEP:WHY: The pinned interference formula subtracts directional coalescibility from overlap; a single fn keeps table semantics identical for the allocator and the law checker.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate.
/// CEP:ASSUMES: the table is the caller's definition of record; the kernel adds no hidden conditions (RA9's "neither fixed" and same-class rules are upstream table construction, per §5.8).
/// CEP:COST: O(P) linear scan, P ≤ table length; allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_interference_formula_cases; EXH exh_allocation_laws_grid_multisets.
#[inline]
pub fn coalescible(table: &[CoalescePair], a: LiveInterval, b: LiveInterval) -> bool {
    let mut i = 0usize;
    while i < table.len() {
        let p = table[i];
        if p.a == a.vreg && p.b == b.vreg {
            return true;
        }
        i += 1;
    }
    false
}

/// Pinned interference predicate: two intervals interfere iff they overlap and
/// at least one is not coalescible with the other.
///
/// Definition anchor: SPEC-INDEX §5.8 (embedded verbatim in Volume 16 §16.4).
/// "At least one is not coalescible with the other" is read exactly:
/// `overlap(a,b) ∧ ¬(coal(a→b) ∧ coal(b→a))`. With a symmetric table (both
/// directions listed) this reduces to `overlap ∧ ¬coal(a,b)`; with an
/// asymmetric table the single direction is not sufficient to remove
/// interference — pinned behavior, exercised by EXH and by the Z3 script's
/// directional coalescence.
/// Input domain: total over all interval pairs and any table slice; pinned
/// semantics on the §5.8 domain (`start ≤ end` per interval).
/// Failure set: none (pure predicate).
/// Certificate: EXH `exh_interference_formula_cases` (incl. the asymmetric case
/// and degenerate intervals); Z3 `regalloc_laws.smt2` checks the composed
/// laws on a concrete bounded instance (5 intervals, 4 registers, half-open
/// interference from this census) plus a 1-register non-vacuity case; full
/// census + PROP assert the two laws through this predicate.
///
/// CEP:WHAT: Pinned interference = overlap ∧ ¬(coal(a→b) ∧ coal(b→a)).
/// CEP:WHY: This is the exact relation over which both pinned allocation laws and the RA6 coloring are defined.
/// CEP:STATUS: complete
/// CEP:FAILURE: none: total pure predicate; directional semantics pinned by EXH and mirrored in Z3.
/// CEP:ASSUMES: coalescibility is directional (RA9 move pairs); the empty-position reading of degenerate intervals applies via `overlap`.
/// CEP:COST: O(P) (two table scans), allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_interference_formula_cases; EXH exh_allocation_laws_grid_multisets; Z3 tools/fv/z3/regalloc_laws.smt2; PROP prop_laws_larger_instances.
#[inline]
pub fn interfere(a: LiveInterval, b: LiveInterval, table: &[CoalescePair]) -> bool {
    if !overlap(a, b) {
        return false;
    }
    !(coalescible(table, a, b) && coalescible(table, b, a))
}

/// Register pressure of a set of live intervals: the max simultaneous live
/// count (SPEC-INDEX §5.8, pinned definition).
///
/// Definition anchor: SPEC-INDEX §5.8 ("register pressure = max simultaneous
/// live count"). Exact reading: max over all positions p of the number of
/// intervals i with `i.start ≤ p < i.end`. The max is attained at some
/// interval's start (the count only changes at starts/ends), so the sweep needs
/// only the start points.
/// Input domain: `intervals.len() ≤ MAX_INTERVALS` and `start ≤ end` for every
/// interval; the function reads only `start`/`end` (RA7 metadata is not read
/// and not validated here).
/// Failure set: `RegAllocError::TooManyIntervals`, `RegAllocError::InvalidInterval`.
/// Certificate: EXH `exh_register_pressure_grid_multisets_2024` — all multisets
/// of size ≤ 3 over the 21 grid shapes (2,024 instances) against an independent
/// position-sweep reference; paired with PROP instances ≤ 64 intervals.
///
/// CEP:WHAT: Pinned max-simultaneous-live-count pressure.
/// CEP:WHY: Pressure is the termination input of the pinned spill loop; the same definition is RA14's recorded statistic upstream.
/// CEP:STATUS: complete
/// CEP:FAILURE: TooManyIntervals (n > MAX_INTERVALS); InvalidInterval (any start > end). No other failure; never panics.
/// CEP:ASSUMES: positions are discrete u32; degenerate `[p,p)` intervals contain no position and never raise the count (they overlap nothing).
/// CEP:COST: O(n²) sweep, n ≤ MAX_INTERVALS; allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_register_pressure_grid_multisets_2024; PROP prop_laws_larger_instances.
pub fn register_pressure(intervals: &[LiveInterval]) -> Result<u32, RegAllocError> {
    let n = intervals.len();
    if n > MAX_INTERVALS {
        return Err(RegAllocError::TooManyIntervals);
    }
    let mut i = 0usize;
    while i < n {
        if intervals[i].start > intervals[i].end {
            return Err(RegAllocError::InvalidInterval);
        }
        i += 1;
    }
    let mut best = 0u32;
    i = 0;
    while i < n {
        let p = intervals[i].start;
        let mut c = 0u32;
        let mut j = 0usize;
        while j < n {
            let it = intervals[j];
            if it.start <= p && p < it.end {
                c += 1;
            }
            j += 1;
        }
        if c > best {
            best = c;
        }
        i += 1;
    }
    Ok(best)
}

/// Spill priority with the pinned formula, exact checked u64 integer math:
/// `uses × 10^3 + loop_depth × 10^6 − size_cost`.
///
/// Definition anchor: SPEC-INDEX §5.8 + Volume 16 §16.4 RA7 (checked multiply/
/// add/sub mandatory; overflow or underflow yields E-RA-01). SPEC-INDEX §5.8 is
/// silent on the subtraction; Volume 16 §16.4 RA7 is more precise and wins: the
/// subtraction is CHECKED and underflow is the defined error
/// `SpillPriorityError::Underflow` (reachable: uses = 0, depth = 0, size > 0),
/// not saturating and not wrapping.
/// Input domain: all u64 triples (the checked-math witness for RA7; `allocate`
/// additionally enforces the RA7 domain bounds uses ≤ 2^20, depth ≤ 64,
/// size ≤ 64 before calling this).
/// Failure set: `SpillPriorityError::Overflow`, `SpillPriorityError::Underflow`.
/// Certificate: EXH `exh_spill_priority_corner_grid_27` — all 27 corner
/// combinations of uses ∈ {0, 1, 2^20}, depth ∈ {0, 1, 64}, size ∈ {0, 1, 64}
/// (the OB-16-6 corner domain, complete) against an exact i128 reference.
///
/// CEP:WHAT: Pinned checked spill-priority formula (SPILL_PRIORITY_USES_SCALE / SPILL_PRIORITY_DEPTH_SCALE).
/// CEP:WHY: The spill loop selects victims by ascending priority; the exact arithmetic (including its failure set) is pinned by RA7.
/// CEP:STATUS: complete
/// CEP:FAILURE: Overflow (positive terms exceed u64::MAX); Underflow (result would be negative). No other failure; never panics.
/// CEP:ASSUMES: scales are exactly 10^3 and 10^6 (named constants); within the RA7 domain bounds the positive terms are ≤ 1,112,576,000, so Overflow is impossible there and Underflow is the only reachable variant.
/// CEP:COST: O(1), 4 checked operations, allocation-free; CEP-0.
/// CEP:EVIDENCE: EXH exh_spill_priority_corner_grid_27; EXH exh_allocation_laws_grid_multisets (priority used for victim selection).
#[inline]
pub fn spill_priority(uses: u64, loop_depth: u64, size_cost: u64) -> Result<u64, SpillPriorityError> {
    let u = uses
        .checked_mul(SPILL_PRIORITY_USES_SCALE)
        .ok_or(SpillPriorityError::Overflow)?;
    let d = loop_depth
        .checked_mul(SPILL_PRIORITY_DEPTH_SCALE)
        .ok_or(SpillPriorityError::Overflow)?;
    let sum = u.checked_add(d).ok_or(SpillPriorityError::Overflow)?;
    sum.checked_sub(size_cost)
        .ok_or(SpillPriorityError::Underflow)
}

// ---------------------------------------------------------------------------
// Pinned allocation pipeline (RA6 coloring + RA7 spill loop).
// ---------------------------------------------------------------------------

/// Sorted copy of the coalescibility table for O(log P) membership inside the
/// allocator loop (private; the public `coalescible` scans the caller's slice).
struct SortedPairs {
    buf: [CoalescePair; MAX_COALESCE_PAIRS],
    len: usize,
}

impl SortedPairs {
    fn build(src: &[CoalescePair]) -> Self {
        let mut buf = [CoalescePair { a: 0, b: 0 }; MAX_COALESCE_PAIRS];
        let len = src.len();
        let mut i = 0usize;
        while i < len {
            buf[i] = src[i];
            i += 1;
        }
        // Insertion sort by (a, b): total, deterministic, allocation-free.
        let mut t = 1usize;
        while t < len {
            let cur = buf[t];
            let mut s = t;
            while s > 0 {
                let prev = buf[s - 1];
                if (prev.a, prev.b) <= (cur.a, cur.b) {
                    break;
                }
                buf[s] = prev;
                s -= 1;
            }
            buf[s] = cur;
            t += 1;
        }
        SortedPairs { buf, len }
    }

    fn contains(&self, a: u32, b: u32) -> bool {
        let mut lo = 0usize;
        let mut hi = self.len;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let p = self.buf[mid];
            if (p.a, p.b) < (a, b) {
                lo = mid + 1;
            } else if (p.a, p.b) > (a, b) {
                hi = mid;
            } else {
                return true;
            }
        }
        false
    }
}

fn bit_is_set(w: &[u64; MAX_INTERVALS / 64], i: usize) -> bool {
    (w[i / 64] >> (i % 64)) & 1 == 1
}

fn set_bit(w: &mut [u64; MAX_INTERVALS / 64], i: usize) {
    w[i / 64] |= 1u64 << (i % 64);
}

/// Internal interference on interval indices through the sorted table
/// (identical semantics to the public `interfere`).
fn interfere_idx(
    intervals: &[LiveInterval],
    pairs: &SortedPairs,
    i: usize,
    j: usize,
) -> bool {
    let a = intervals[i];
    let b = intervals[j];
    if !overlap(a, b) {
        return false;
    }
    !(pairs.contains(a.vreg, b.vreg) && pairs.contains(b.vreg, a.vreg))
}

/// Allocate registers for one register class: degree-ordered First-Fit over the
/// caller-provided class order with the pinned ascending-priority spill loop
/// (SPEC-INDEX §5.8; Volume 16 §16.4 RA6 + RA7).
///
/// Definition anchor: SPEC-INDEX §5.8 + Volume 16 §16.4. Exact pinned algorithm
/// (this is the whole specification of this function):
/// 1. Validate the input domain (below); first violation is returned.
/// 2. Loop: color the not-yet-spilled intervals — sorted by (interference degree
///    descending, start ascending, interval ID = vreg ascending) over the set
///    being colored (RA6 order, degrees recomputed per pass over that set) —
///    each interval takes the first register in `class_order` not held by an
///    already-assigned interfering interval (graph-coloring First-Fit: a
///    register is free iff no assigned *interfering* interval holds it, so
///    non-interfering intervals may share and a register is reusable after its
///    holder's range ends).
/// 3. If every not-spilled interval got a register AND the RA7 pressure over
///    register-assigned intervals (max count of register-assigned intervals
///    covering one position) is ≤ available, return Ok. The conjunction is the
///    exact refinement making RA7 and RA15 V1 jointly consistent: RA7's
///    "while max pressure > available" alone cannot give every interval a
///    location when greedy First-Fit blocks a register that no single point
///    over-uses, and RA15 V1 requires exactly one location per interval.
/// 4. Otherwise spill the candidate with smallest spill priority (ascending
///    benefit order = ascending priority; tie-break ascending vreg ID, RA7)
///    among intervals that are neither spilled nor fixed (RA13: fixed are never
///    spilled). Candidate priorities are all computed (checked u64, RA7), so an
///    E-RA-01 overflow/underflow on any candidate aborts the loop with
///    `SpillPriorityOverflow`/`SpillPriorityUnderflow`. The victim gets the next
///    private slot. When no candidate exists and step 3 still fails,
///    `SpillImpossible` is returned (OB-16-7 arm 2; the caller invariant
///    fixed(class) ≤ available(class) of §16.2 is violated).
/// Termination: each iteration either returns or spills one more non-fixed
/// interval; at most n + 1 iterations.
/// Input domain (all enforced): `intervals.len() ≤ MAX_INTERVALS`;
/// `out.len() ≥ intervals.len()`; `class_order.len() ≤ MAX_REGS` with distinct
/// encodings (empty order = 0 available registers, legal); per interval
/// `start ≤ end`, `uses ≤ 2^20`, `loop_depth ≤ 64`, `size_cost ≤ 64`, unique
/// `vreg`; `coalesce.len() ≤ MAX_COALESCE_PAIRS` with `a ≠ b` and endpoints that
/// are vregs of the problem.
/// Failure set: every `RegAllocError` variant (see the enum; complete).
/// Output: on Ok, `out[0..n]` holds exactly one `Location` per interval
/// (register from the class order, or private slot with index < spilled count)
/// and the returned stats satisfy `assigned + spilled == n` and
/// `final_pressure ≤ available`. On Err the contents of `out` are unspecified.
/// Certificate: EXH `exh_allocation_laws_grid_multisets` (698,720 instances:
/// both pinned laws + slot discipline + pressure bound + determinism asserted on
/// every accepted output; totality asserted for non-fixed inputs) + EXH
/// `exh_fixed_intervals_spill_paths` (26,222 fixed-interval instances covering
/// the Ok and SpillImpossible arms) + Z3 `regalloc_laws.smt2` (no-conflict law
/// on the concrete bounded instance + 1-register non-vacuity case) + PROP
/// `prop_laws_larger_instances` (5,000 instances ≤ 64 intervals, paired per
/// SPEC-INDEX §4.3). Runtime bridge beyond the FV bounds: RA15 verifier.
///
/// CEP:WHAT: Pinned degree-ordered First-Fit coloring with the ascending-priority spill loop over a caller-provided class order.
/// CEP:WHY: This is the kernel of Volume 16 §16.4 RA6+RA7; the backend coloring must be exactly this function of its inputs (G14 determinism).
/// CEP:STATUS: complete
/// CEP:FAILURE: complete `RegAllocError` set: TooManyIntervals, OutputBufferTooSmall, TooManyRegisters, DuplicateRegister, InvalidInterval, UsesOutOfRange, LoopDepthOutOfRange, SizeCostOutOfRange, DuplicateVreg, TooManyCoalescePairs, CoalesceSelfPair, CoalesceVregUnknown, SpillImpossible, SpillPriorityOverflow, SpillPriorityUnderflow. Nothing panics; no partial success exists.
/// CEP:ASSUMES: (a) degrees are recomputed per coloring pass over the set being colored (RA6 does not pin the computation point; this is the pinned kernel choice); (b) RA7 pressure counts register-assigned intervals covering a position — coalescible overlapping intervals that share a register still count separately; RA9 merges such pairs into one interval upstream, so production inputs normally contain no overlapping coalescible pairs, and the kernel applies RA7 literally; (c) all candidate priorities are computed each spill step (selection of the minimum requires it), so E-RA-01 fires on any candidate, not only the selected victim; (d) rematerialization (RA10) and per-operand constraints (RA5) are upstream: every spill candidate is spilled to its own slot; (e) fixed intervals are colored by the same First-Fit and are never spilled (RA13 at §5.8 abstraction; their concrete pre-assignment is caller-side); (f) SpillPriorityOverflow is unreachable within the validated domain (retained per RA7's mandated checked math).
/// CEP:COST: per coloring pass O(n²·log P) (degrees + First-Fit scans + binary-search coalescence) + O(n log n) sort; spill loop ≤ n+1 passes → O(n³·log P) worst case at n = MAX_INTERVALS; ~93 KiB fixed stack working storage; zero allocation; CEP-1.
/// CEP:EVIDENCE: EXH exh_allocation_laws_grid_multisets; EXH exh_fixed_intervals_spill_paths; EXH exh_allocate_known_instances; EXH exh_error_variants_reachable; Z3 tools/fv/z3/regalloc_laws.smt2; PROP prop_laws_larger_instances.
pub fn allocate(
    problem: &RegAllocProblem,
    out: &mut [Location],
) -> Result<RegAllocStats, RegAllocError> {
    let intervals = problem.intervals;
    let n = intervals.len();
    if n > MAX_INTERVALS {
        return Err(RegAllocError::TooManyIntervals);
    }
    if out.len() < n {
        return Err(RegAllocError::OutputBufferTooSmall);
    }
    let order = problem.class_order;
    let avail = order.len();
    if avail > MAX_REGS {
        return Err(RegAllocError::TooManyRegisters);
    }
    let mut i = 0usize;
    while i < avail {
        let mut j = i + 1;
        while j < avail {
            if order[i] == order[j] {
                return Err(RegAllocError::DuplicateRegister);
            }
            j += 1;
        }
        i += 1;
    }

    // Per-interval domain validation + vreg uniqueness (sorted copy, adjacent scan).
    let mut vregs = [0u32; MAX_INTERVALS];
    i = 0;
    while i < n {
        let it = intervals[i];
        if it.start > it.end {
            return Err(RegAllocError::InvalidInterval);
        }
        if (it.uses as u64) > MAX_USES_PER_INTERVAL {
            return Err(RegAllocError::UsesOutOfRange);
        }
        if (it.loop_depth as u64) > MAX_LOOP_DEPTH {
            return Err(RegAllocError::LoopDepthOutOfRange);
        }
        if (it.size_cost as u64) > MAX_SIZE_COST {
            return Err(RegAllocError::SizeCostOutOfRange);
        }
        vregs[i] = it.vreg;
        i += 1;
    }
    let mut t = 1usize;
    while t < n {
        let cur = vregs[t];
        let mut s = t;
        while s > 0 && vregs[s - 1] > cur {
            vregs[s] = vregs[s - 1];
            s -= 1;
        }
        vregs[s] = cur;
        t += 1;
    }
    i = 1;
    while i < n {
        if vregs[i] == vregs[i - 1] {
            return Err(RegAllocError::DuplicateVreg);
        }
        i += 1;
    }

    // Coalescibility table validation + sorted copy.
    let pairs = problem.coalesce;
    if pairs.len() > MAX_COALESCE_PAIRS {
        return Err(RegAllocError::TooManyCoalescePairs);
    }
    i = 0;
    while i < pairs.len() {
        let p = pairs[i];
        if p.a == p.b {
            return Err(RegAllocError::CoalesceSelfPair);
        }
        // Endpoint membership in the sorted vreg set (binary search).
        let mut lo = 0usize;
        let mut hi = n;
        let mut found = false;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if vregs[mid] < p.a {
                lo = mid + 1;
            } else if vregs[mid] > p.a {
                hi = mid;
            } else {
                found = true;
                break;
            }
        }
        if !found {
            return Err(RegAllocError::CoalesceVregUnknown);
        }
        lo = 0;
        hi = n;
        found = false;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if vregs[mid] < p.b {
                lo = mid + 1;
            } else if vregs[mid] > p.b {
                hi = mid;
            } else {
                found = true;
                break;
            }
        }
        if !found {
            return Err(RegAllocError::CoalesceVregUnknown);
        }
        i += 1;
    }
    let sorted = SortedPairs::build(pairs);

    // Fixed-capacity working storage (~93 KiB total; no allocation).
    let mut loc = [Location::Register(0); MAX_INTERVALS];
    let mut pos = [0u8; MAX_INTERVALS]; // order-position this pass; POS_UNASSIGNED = none
    let mut deg = [0u32; MAX_INTERVALS];
    let mut idx = [0u16; MAX_INTERVALS];
    let mut spilled = [0u64; MAX_INTERVALS / 64];
    let mut spilled_count: u32 = 0;

    loop {
        // ---- Coloring pass over the not-spilled set (RA6 order + First-Fit) ----
        i = 0;
        while i < n {
            if !bit_is_set(&spilled, i) {
                pos[i] = POS_UNASSIGNED;
            }
            i += 1;
        }
        i = 0;
        while i < n {
            if bit_is_set(&spilled, i) {
                i += 1;
                continue;
            }
            let mut d = 0u32;
            let mut j = 0usize;
            while j < n {
                if j != i && !bit_is_set(&spilled, j) && interfere_idx(intervals, &sorted, i, j) {
                    d += 1;
                }
                j += 1;
            }
            deg[i] = d;
            i += 1;
        }
        let mut rem = 0usize;
        i = 0;
        while i < n {
            if !bit_is_set(&spilled, i) {
                idx[rem] = i as u16;
                rem += 1;
            }
            i += 1;
        }
        // Insertion sort: degree descending, start ascending, vreg ascending.
        t = 1;
        while t < rem {
            let cur = idx[t];
            let c = cur as usize;
            let mut s = t;
            while s > 0 {
                let p = idx[s - 1] as usize;
                let c_before = deg[c] > deg[p]
                    || (deg[c] == deg[p]
                        && (intervals[c].start < intervals[p].start
                            || (intervals[c].start == intervals[p].start
                                && intervals[c].vreg < intervals[p].vreg)));
                if !c_before {
                    break;
                }
                idx[s] = idx[s - 1];
                s -= 1;
            }
            idx[s] = cur;
            t += 1;
        }
        // First-Fit over the class order.
        let mut incomplete = false;
        t = 0;
        while t < rem {
            let cur = idx[t] as usize;
            let mut busy = 0u32;
            let mut j = 0usize;
            while j < n {
                if j != cur
                    && !bit_is_set(&spilled, j)
                    && pos[j] != POS_UNASSIGNED
                    && interfere_idx(intervals, &sorted, cur, j)
                {
                    busy |= 1u32 << pos[j];
                }
                j += 1;
            }
            let mut p = 0u8;
            while (p as usize) < avail && (busy >> (p as u32)) & 1 == 1 {
                p += 1;
            }
            if (p as usize) < avail {
                pos[cur] = p;
                loc[cur] = Location::Register(order[p as usize]);
            } else {
                incomplete = true;
            }
            t += 1;
        }
        // RA7 pressure over register-assigned intervals.
        let mut pressure = 0u32;
        i = 0;
        while i < n {
            if bit_is_set(&spilled, i) || pos[i] == POS_UNASSIGNED {
                i += 1;
                continue;
            }
            let p = intervals[i].start;
            let mut c = 0u32;
            let mut j = 0usize;
            while j < n {
                if !bit_is_set(&spilled, j) && pos[j] != POS_UNASSIGNED {
                    let it = intervals[j];
                    if it.start <= p && p < it.end {
                        c += 1;
                    }
                }
                j += 1;
            }
            if c > pressure {
                pressure = c;
            }
            i += 1;
        }
        if !incomplete && pressure <= avail as u32 {
            i = 0;
            while i < n {
                out[i] = loc[i];
                i += 1;
            }
            return Ok(RegAllocStats {
                assigned: n as u32 - spilled_count,
                spilled: spilled_count,
                final_pressure: pressure,
            });
        }

        // ---- Spill step (RA7): lowest priority first, tie-break ascending vreg ----
        let mut best = n; // sentinel
        let mut best_pri = 0u64;
        i = 0;
        while i < n {
            if bit_is_set(&spilled, i) || intervals[i].fixed {
                i += 1;
                continue;
            }
            let it = intervals[i];
            let pri = spill_priority(
                it.uses as u64,
                it.loop_depth as u64,
                it.size_cost as u64,
            )
            .map_err(|e| match e {
                SpillPriorityError::Overflow => RegAllocError::SpillPriorityOverflow,
                SpillPriorityError::Underflow => RegAllocError::SpillPriorityUnderflow,
            })?;
            let better = best == n
                || pri < best_pri
                || (pri == best_pri && it.vreg < intervals[best].vreg);
            if better {
                best = i;
                best_pri = pri;
            }
            i += 1;
        }
        if best == n {
            return Err(RegAllocError::SpillImpossible);
        }
        set_bit(&mut spilled, best);
        loc[best] = Location::Slot(spilled_count);
        spilled_count += 1;
    }
}

// ---------------------------------------------------------------------------
// Tests (CEP-2; deterministic; no external crates; in-crate pinned PRNG).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;

    // ----- Shared test helpers ---------------------------------------------

    /// Pinned in-crate xorshift64* PRNG (seed 0x9E3779B97F4A7C15) for PROP; deterministic.
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

    /// Pinned census metadata rule: deterministic per interval shape (s, e),
    /// inside the RA7 domain, and never underflowing (uses ≥ 1 keeps
    /// priority ≥ 1000 − 15 > 0).
    fn census_meta(s: u32, e: u32) -> (u32, u32, u32) {
        (
            1 + ((s + e) % 3), // uses ∈ {1,2,3} ≤ 2^20
            s % 2,             // loop_depth ∈ {0,1} ≤ 64
            8 + (s % 8),       // size_cost ∈ {8..=15} ≤ 64
        )
    }

    /// Multiset successor over `base` shapes in canonical non-decreasing form
    /// (standard lexicographic successor for combinations with repetition):
    /// raises the rightmost position d that still admits a non-decreasing
    /// continuation (`sel[d] + 1 ≤ sel[d+1]`, with the sentinel
    /// `sel[k−1] + 1 < base` for the last position) and resets the suffix to
    /// that value. Starting from all zeros it enumerates exactly
    /// C(base + k − 1, k) multisets of size k.
    fn next_multiset(sel: &mut [usize], k: usize, base: usize) -> bool {
        let mut d = k;
        while d > 0 {
            d -= 1;
            let can_raise = if d + 1 == k {
                sel[d] + 1 < base
            } else {
                sel[d] + 1 <= sel[d + 1]
            };
            if can_raise {
                sel[d] += 1;
                let mut e = d + 1;
                while e < k {
                    sel[e] = sel[d];
                    e += 1;
                }
                return true;
            }
        }
        false
    }

    /// Law checker shared by EXH and PROP (the pairing mechanism): asserts both
    /// pinned laws and the output discipline on an accepted allocation.
    /// (V1) exactly one location per interval, registers from the class order,
    ///      slots distinct with index < spilled count, fixed never spilled;
    /// (V2) no two interfering intervals share a location (re-derived from the
    ///      inputs through the public `interfere`);
    /// (V3) recomputed RA7 pressure over register-assigned intervals ≤ available.
    fn assert_laws_hold(
        intervals: &[LiveInterval],
        pairs: &[CoalescePair],
        order: &[u16],
        out: &[Location],
        stats: RegAllocStats,
    ) {
        let n = intervals.len();
        assert_eq!(out.len(), n);
        assert_eq!(stats.assigned + stats.spilled, n as u32);
        let mut seen_slots: u64 = 0;
        let mut i = 0usize;
        while i < n {
            match out[i] {
                Location::Register(r) => {
                    let mut in_order = false;
                    let mut k = 0usize;
                    while k < order.len() {
                        if order[k] == r {
                            in_order = true;
                        }
                        k += 1;
                    }
                    assert!(in_order, "register not in class order");
                }
                Location::Slot(k) => {
                    assert!(k < stats.spilled, "slot index out of range");
                    assert!(k < 64, "census slots must fit the seen mask");
                    assert!(seen_slots & (1u64 << k) == 0, "slot reused");
                    seen_slots |= 1u64 << k;
                    assert!(!intervals[i].fixed, "fixed interval spilled");
                }
            }
            i += 1;
        }
        assert_eq!(seen_slots.count_ones(), stats.spilled);
        // (V2) no two interfering intervals share a location.
        let mut a = 0usize;
        while a < n {
            let mut b = a + 1;
            while b < n {
                if interfere(intervals[a], intervals[b], pairs) {
                    assert_ne!(out[a], out[b], "interfering intervals share a location");
                }
                b += 1;
            }
            a += 1;
        }
        // (V3) RA7 pressure over register-assigned intervals ≤ available.
        let mut pressure = 0u32;
        let mut x = 0usize;
        while x < n {
            if let Location::Register(_) = out[x] {
                let p = intervals[x].start;
                let mut c = 0u32;
                let mut y = 0usize;
                while y < n {
                    if let Location::Register(_) = out[y] {
                        if intervals[y].start <= p && p < intervals[y].end {
                            c += 1;
                        }
                    }
                    y += 1;
                }
                if c > pressure {
                    pressure = c;
                }
            }
            x += 1;
        }
        assert!(
            pressure <= order.len() as u32,
            "accepted allocation exceeds available registers"
        );
        assert_eq!(pressure, stats.final_pressure);
    }

    // ----- EXH 1: overlap on the full 6-point grid (441 shape pairs) -------

    /// EXH certificate: overlap equals the independent existential reference
    /// "∃p in 0..=6: both intervals contain p" for all 21×21 = 441 pairs of
    /// shapes (s, e) with 0 ≤ s ≤ e ≤ 5. Full shape-pair domain, test-as-proof.
    #[test]
    fn exh_overlap_halfopen_grid_441() {
        let mut shapes: [(u32, u32); 21] = [(0, 0); 21];
        let mut k = 0usize;
        let mut s = 0u32;
        while s <= 5 {
            let mut e = s;
            while e <= 5 {
                shapes[k] = (s, e);
                k += 1;
                e += 1;
            }
            s += 1;
        }
        assert_eq!(k, 21);
        let mk = |se: (u32, u32)| LiveInterval {
            vreg: 0,
            start: se.0,
            end: se.1,
            uses: 1,
            loop_depth: 0,
            size_cost: 8,
            fixed: false,
        };
        let mut a = 0usize;
        while a < 21 {
            let mut b = 0usize;
            while b < 21 {
                let (s1, e1) = shapes[a];
                let (s2, e2) = shapes[b];
                // Independent reference: some grid position is in both ranges.
                let mut p = 0u32;
                let mut ref_overlap = false;
                while p <= 6 {
                    if s1 <= p && p < e1 && s2 <= p && p < e2 {
                        ref_overlap = true;
                    }
                    p += 1;
                }
                assert_eq!(overlap(mk(shapes[a]), mk(shapes[b])), ref_overlap, "shapes {a},{b}");
                b += 1;
            }
            a += 1;
        }
        // Degenerate intervals are the empty position set: never overlapping.
        let deg = mk((3, 3));
        let mut i = 0usize;
        while i < 21 {
            assert!(!overlap(deg, mk(shapes[i])));
            assert!(!overlap(mk(shapes[i]), deg));
            i += 1;
        }
    }

    // ----- EXH 2: interference formula (directional coalescence) ------------

    /// EXH certificate: the pinned interference formula on exact directional
    /// cases: overlap ∧ ¬(coal(a→b) ∧ coal(b→a)); a single-direction table does
    /// NOT remove interference; a both-direction table does; no overlap never
    /// interferes.
    #[test]
    fn exh_interference_formula_cases() {
        let a = LiveInterval { vreg: 1, start: 0, end: 4, uses: 1, loop_depth: 0, size_cost: 8, fixed: false };
        let b = LiveInterval { vreg: 2, start: 1, end: 5, uses: 1, loop_depth: 0, size_cost: 8, fixed: false };
        let c = LiveInterval { vreg: 2, start: 4, end: 6, uses: 1, loop_depth: 0, size_cost: 8, fixed: false }; // disjoint from a? [4,6) vs [0,4): adjacent, no overlap
        let both = [CoalescePair { a: 1, b: 2 }, CoalescePair { a: 2, b: 1 }];
        let ab_only = [CoalescePair { a: 1, b: 2 }];
        let ba_only = [CoalescePair { a: 2, b: 1 }];
        // Overlapping pair.
        assert!(interfere(a, b, &[]));
        assert!(interfere(a, b, &ab_only), "single direction must not remove interference");
        assert!(interfere(a, b, &ba_only), "single direction must not remove interference");
        assert!(!interfere(a, b, &both), "both directions remove interference");
        // Non-overlapping (adjacent half-open) pair never interferes.
        assert!(!interfere(a, c, &[]));
        assert!(!interfere(a, c, &both));
        // Degenerate interval overlaps nothing, even when "coalescible".
        let deg = LiveInterval { vreg: 3, start: 2, end: 2, uses: 1, loop_depth: 0, size_cost: 8, fixed: false };
        let deg_pairs = [CoalescePair { a: 1, b: 3 }, CoalescePair { a: 3, b: 1 }];
        assert!(!interfere(a, deg, &deg_pairs));
    }

    // ----- EXH 3: register pressure vs independent sweep reference ----------

    /// EXH certificate: register_pressure equals the independent position-sweep
    /// reference for ALL multisets of size ≤ 3 over the 21 grid shapes:
    /// 1 + 21 + 231 + 1771 = 2,024 instances (full sub-domain, test-as-proof).
    #[test]
    fn exh_register_pressure_grid_multisets_2024() {
        let mut shapes: [(u32, u32); 21] = [(0, 0); 21];
        let mut k = 0usize;
        let mut s = 0u32;
        while s <= 5 {
            let mut e = s;
            while e <= 5 {
                shapes[k] = (s, e);
                k += 1;
                e += 1;
            }
            s += 1;
        }
        let mk = |se: (u32, u32), v: u32| LiveInterval {
            vreg: v,
            start: se.0,
            end: se.1,
            uses: 1,
            loop_depth: 0,
            size_cost: 8,
            fixed: false,
        };
        let mut count = 0u64;
        let mut size = 0usize;
        while size <= 3 {
            let mut sel = [0usize; 3];
            loop {
                let ivs: Vec<LiveInterval> = (0..size).map(|t| mk(shapes[sel[t]], t as u32)).collect();
                let got = register_pressure(&ivs).expect("pressure ok");
                // Independent reference: sweep every grid position.
                let mut want = 0u32;
                let mut p = 0u32;
                while p <= 6 {
                    let mut c = 0u32;
                    let mut t = 0usize;
                    while t < size {
                        let (s2, e2) = shapes[sel[t]];
                        if s2 <= p && p < e2 {
                            c += 1;
                        }
                        t += 1;
                    }
                    if c > want {
                        want = c;
                    }
                    p += 1;
                }
                assert_eq!(got, want, "instance {count}");
                count += 1;
                if !next_multiset(&mut sel, size, 21) {
                    break;
                }
            }
            size += 1;
        }
        assert_eq!(count, 2_024);
        assert_eq!(register_pressure(&[]).unwrap(), 0);
    }

    // ----- EXH 4: spill priority corner grid (27 combinations, OB-16-6) -----

    /// EXH certificate: all 27 corner combinations of uses ∈ {0, 1, 2^20},
    /// depth ∈ {0, 1, 64}, size ∈ {0, 1, 64} against an exact i128 reference
    /// (OB-16-6 corner domain, complete): overflow/underflow detected, exact
    /// values otherwise.
    #[test]
    fn exh_spill_priority_corner_grid_27() {
        let uses_c: [u64; 3] = [0, 1, MAX_USES_PER_INTERVAL];
        let depth_c: [u64; 3] = [0, 1, MAX_LOOP_DEPTH];
        let size_c: [u64; 3] = [0, 1, MAX_SIZE_COST];
        let mut i = 0usize;
        while i < 3 {
            let mut j = 0usize;
            while j < 3 {
                let mut k = 0usize;
                while k < 3 {
                    let (u, d, s) = (uses_c[i], depth_c[j], size_c[k]);
                    let exact = (u as i128) * (SPILL_PRIORITY_USES_SCALE as i128)
                        + (d as i128) * (SPILL_PRIORITY_DEPTH_SCALE as i128)
                        - (s as i128);
                    let expected = if exact < 0 {
                        Err(SpillPriorityError::Underflow)
                    } else if exact > u64::MAX as i128 {
                        Err(SpillPriorityError::Overflow)
                    } else {
                        Ok(exact as u64)
                    };
                    assert_eq!(spill_priority(u, d, s), expected, "u={u} d={d} s={s}");
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        // Overflow is reachable only outside the RA7 domain bounds (full u64 domain).
        assert_eq!(spill_priority(u64::MAX, 0, 0), Err(SpillPriorityError::Overflow));
        assert_eq!(
            spill_priority(MAX_USES_PER_INTERVAL, MAX_LOOP_DEPTH, 0),
            Ok(MAX_USES_PER_INTERVAL * SPILL_PRIORITY_USES_SCALE
                + MAX_LOOP_DEPTH * SPILL_PRIORITY_DEPTH_SCALE)
        );
    }

    // ----- EXH 5: hand-computed allocation instances -------------------------

    /// EXH certificate: exact hand-computed allocations pinning (a) half-open
    /// adjacency reuse (a register frees at its holder's end), (b) degree-order
    /// First-Fit + lowest-priority spill victim, (c) the RA7 pressure rule that
    /// counts register-assigned intervals (coalescible sharers included).
    #[test]
    fn exh_allocate_known_instances() {
        // (a) Chain of adjacent half-open intervals on ONE register: no overlaps.
        let chain = [
            LiveInterval { vreg: 0, start: 0, end: 1, uses: 1, loop_depth: 0, size_cost: 8, fixed: false },
            LiveInterval { vreg: 1, start: 1, end: 2, uses: 1, loop_depth: 0, size_cost: 8, fixed: false },
            LiveInterval { vreg: 2, start: 2, end: 3, uses: 1, loop_depth: 0, size_cost: 8, fixed: false },
        ];
        let prob = RegAllocProblem { intervals: &chain, coalesce: &[], class_order: &[7] };
        let mut out = [Location::Register(0); 3];
        let st = allocate(&prob, &mut out).expect("chain allocates");
        assert_eq!(out, [Location::Register(7); 3]);
        assert_eq!(st, RegAllocStats { assigned: 3, spilled: 0, final_pressure: 1 });

        // (b) Triangle A=[0,4) B=[1,5) C=[2,6), 2 registers: degrees all equal →
        // start order A,B,C; A:r0, B:r1, C blocked → spill lowest priority = B
        // (pri 992 < A 1_002_992 < C 2_001_984) → A:r0, C:r1.
        let tri = [
            LiveInterval { vreg: 0, start: 0, end: 4, uses: 3, loop_depth: 1, size_cost: 8, fixed: false },
            LiveInterval { vreg: 1, start: 1, end: 5, uses: 1, loop_depth: 0, size_cost: 8, fixed: false },
            LiveInterval { vreg: 2, start: 2, end: 6, uses: 2, loop_depth: 2, size_cost: 16, fixed: false },
        ];
        let prob = RegAllocProblem { intervals: &tri, coalesce: &[], class_order: &[0, 1] };
        let st = allocate(&prob, &mut out).expect("triangle allocates");
        assert_eq!(out, [Location::Register(0), Location::Slot(0), Location::Register(1)]);
        assert_eq!(st, RegAllocStats { assigned: 2, spilled: 1, final_pressure: 2 });

        // (c) Coalescible overlapping pair share r0 (law 2 permits it), but the
        // RA7 pressure counts BOTH intervals → 2 > 1 available → the lowest
        // priority interval is spilled (RA7 counts intervals, not registers;
        // RA9 merges such pairs upstream in the production pipeline).
        let pair = [
            LiveInterval { vreg: 0, start: 0, end: 4, uses: 2, loop_depth: 0, size_cost: 8, fixed: false },
            LiveInterval { vreg: 1, start: 1, end: 5, uses: 1, loop_depth: 0, size_cost: 8, fixed: false },
        ];
        let both = [CoalescePair { a: 0, b: 1 }, CoalescePair { a: 1, b: 0 }];
        let prob = RegAllocProblem { intervals: &pair, coalesce: &both, class_order: &[0] };
        let mut out2 = [Location::Register(0); 2];
        let st = allocate(&prob, &mut out2).expect("pair allocates after spill");
        assert_eq!(out2, [Location::Register(0), Location::Slot(0)]);
        assert_eq!(st, RegAllocStats { assigned: 1, spilled: 1, final_pressure: 1 });
        // With two registers the pair shares r0 with NO spill: pressure 2 ≤ 2.
        let prob = RegAllocProblem { intervals: &pair, coalesce: &both, class_order: &[0, 1] };
        let st = allocate(&prob, &mut out2).expect("pair shares with 2 regs");
        assert_eq!(out2, [Location::Register(0), Location::Register(0)]);
        assert_eq!(st, RegAllocStats { assigned: 2, spilled: 0, final_pressure: 2 });
    }

    // ----- EXH 6: every error variant reachable ------------------------------

    /// EXH certificate: one constructed input per `RegAllocError` variant proves
    /// the failure set is reachable exactly as documented (SpillPriorityOverflow
    /// is domain-unreachable inside `allocate` per its CEP and is proven at the
    /// `spill_priority` surface instead).
    #[test]
    fn exh_error_variants_reachable() {
        let mk = |v: u32, s: u32, e: u32| LiveInterval {
            vreg: v,
            start: s,
            end: e,
            uses: 2,
            loop_depth: 1,
            size_cost: 8,
            fixed: false,
        };
        let mut buf = [Location::Register(0); 3];
        // TooManyIntervals / OutputBufferTooSmall / TooManyRegisters / DuplicateRegister.
        let big: Vec<LiveInterval> = (0..=MAX_INTERVALS).map(|i| mk(i as u32, 0, 1)).collect();
        let prob = RegAllocProblem { intervals: &big, coalesce: &[], class_order: &[0] };
        let mut obuf: Vec<Location> = Vec::new();
        assert_eq!(allocate(&prob, &mut obuf), Err(RegAllocError::TooManyIntervals));
        let one = [mk(0, 0, 1)];
        let prob = RegAllocProblem { intervals: &one, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut []), Err(RegAllocError::OutputBufferTooSmall));
        let order33: Vec<u16> = (0..=32).map(|i| i as u16).collect();
        let prob = RegAllocProblem { intervals: &one, coalesce: &[], class_order: &order33 };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::TooManyRegisters));
        let prob = RegAllocProblem { intervals: &one, coalesce: &[], class_order: &[5, 5] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::DuplicateRegister));
        // Field-domain variants.
        let bad = [LiveInterval { vreg: 0, start: 2, end: 1, uses: 2, loop_depth: 1, size_cost: 8, fixed: false }];
        let prob = RegAllocProblem { intervals: &bad, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::InvalidInterval));
        let bad = [LiveInterval { vreg: 0, start: 0, end: 1, uses: (MAX_USES_PER_INTERVAL as u32) + 1, loop_depth: 1, size_cost: 8, fixed: false }];
        let prob = RegAllocProblem { intervals: &bad, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::UsesOutOfRange));
        let bad = [LiveInterval { vreg: 0, start: 0, end: 1, uses: 2, loop_depth: 65, size_cost: 8, fixed: false }];
        let prob = RegAllocProblem { intervals: &bad, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::LoopDepthOutOfRange));
        let bad = [LiveInterval { vreg: 0, start: 0, end: 1, uses: 2, loop_depth: 1, size_cost: 65, fixed: false }];
        let prob = RegAllocProblem { intervals: &bad, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::SizeCostOutOfRange));
        // DuplicateVreg.
        let dup = [mk(9, 0, 1), mk(9, 2, 3)];
        let prob = RegAllocProblem { intervals: &dup, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::DuplicateVreg));
        // Table variants.
        let two = [mk(0, 0, 1), mk(1, 2, 3)];
        let pairs: Vec<CoalescePair> = (0..=MAX_COALESCE_PAIRS).map(|_| CoalescePair { a: 0, b: 1 }).collect();
        let prob = RegAllocProblem { intervals: &two, coalesce: &pairs, class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::TooManyCoalescePairs));
        let prob = RegAllocProblem {
            intervals: &two,
            coalesce: &[CoalescePair { a: 1, b: 1 }],
            class_order: &[0],
        };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::CoalesceSelfPair));
        let prob = RegAllocProblem {
            intervals: &two,
            coalesce: &[CoalescePair { a: 0, b: 7 }],
            class_order: &[0],
        };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::CoalesceVregUnknown));
        // SpillImpossible: two overlapping FIXED intervals, one register; no
        // non-fixed candidate can ever be spilled.
        let fixed2 = [
            LiveInterval { vreg: 0, start: 0, end: 4, uses: 1, loop_depth: 0, size_cost: 8, fixed: true },
            LiveInterval { vreg: 1, start: 1, end: 5, uses: 1, loop_depth: 0, size_cost: 8, fixed: true },
        ];
        let prob = RegAllocProblem { intervals: &fixed2, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::SpillImpossible));
        // SpillPriorityUnderflow through the spill loop: a 0-use, 0-depth,
        // positive-size candidate is a spill candidate when pressure > available.
        let under = [
            LiveInterval { vreg: 0, start: 0, end: 3, uses: 2, loop_depth: 1, size_cost: 8, fixed: false },
            LiveInterval { vreg: 1, start: 1, end: 4, uses: 5, loop_depth: 0, size_cost: 8, fixed: false },
            LiveInterval { vreg: 2, start: 2, end: 5, uses: 0, loop_depth: 0, size_cost: 8, fixed: false },
        ];
        let prob = RegAllocProblem { intervals: &under, coalesce: &[], class_order: &[0] };
        assert_eq!(allocate(&prob, &mut buf), Err(RegAllocError::SpillPriorityUnderflow));
        // SpillPriorityOverflow is reachable at the spill_priority surface.
        assert_eq!(spill_priority(u64::MAX, 0, 0), Err(SpillPriorityError::Overflow));
    }

    // ----- EXH 7: empty-input totality ---------------------------------------

    /// EXH certificate: the empty problem is total: zero intervals allocate with
    /// zero pressure for any class order, including the empty order.
    #[test]
    fn exh_empty_input_totality() {
        let mut out: [Location; 0] = [];
        let prob = RegAllocProblem { intervals: &[], coalesce: &[], class_order: &[0, 1] };
        assert_eq!(allocate(&prob, &mut out), Ok(RegAllocStats { assigned: 0, spilled: 0, final_pressure: 0 }));
        let prob = RegAllocProblem { intervals: &[], coalesce: &[], class_order: &[] };
        assert_eq!(allocate(&prob, &mut out), Ok(RegAllocStats { assigned: 0, spilled: 0, final_pressure: 0 }));
        assert_eq!(register_pressure(&[]), Ok(0));
    }

    // ----- EXH 8: main census — the two pinned laws over the grid multisets --

    /// Census shape table: all (s, e) with 0 ≤ s ≤ e ≤ 5 — exactly 21 shapes.
    fn grid_shapes() -> [(u32, u32); 21] {
        let mut shapes: [(u32, u32); 21] = [(0, 0); 21];
        let mut k = 0usize;
        let mut s = 0u32;
        while s <= 5 {
            let mut e = s;
            while e <= 5 {
                shapes[k] = (s, e);
                k += 1;
                e += 1;
            }
            s += 1;
        }
        shapes
    }

    /// EXH certificate (main census): ALL multisets of size 0..=5 over the 21
    /// interval shapes of the 6-point grid (start/end ∈ 0..=5), paired with the
    /// pinned coalescibility-configuration rule: for sizes 0..=3 every table of
    /// at most 2 ordered pairs (covering the empty/non-coalescible, single-
    /// direction asymmetric, and both-direction symmetric configurations);
    /// for sizes 4..=5 the empty table and every single unordered pair listed
    /// in both directions. Exactly 698,720 instances (≤ 10^6), each allocated
    /// with an available-register order of length 1 + (instance index mod 3)
    /// over encodings {0,1,2}. For EVERY instance: the allocation must succeed
    /// (totality of the pinned loop for non-fixed inputs), both laws must hold
    /// (V1 exactly-one-location incl. slot discipline; V2 no-two-interfering-
    /// share via the public `interfere`), the RA7 pressure bound must hold, and
    /// a second run must be byte-identical (determinism, G14).
    #[test]
    fn exh_allocation_laws_grid_multisets() {
        let shapes = grid_shapes();
        let mut out1 = [Location::Register(0); 5];
        let mut out2 = [Location::Register(0); 5];
        let mut instance: u64 = 0;
        let mut size = 0usize;
        while size <= 5 {
            let mut sel = [0usize; 5];
            loop {
                // Build the intervals of this multiset (pinned metadata rule).
                let mut ivs = [LiveInterval {
                    vreg: 0,
                    start: 0,
                    end: 0,
                    uses: 0,
                    loop_depth: 0,
                    size_cost: 0,
                    fixed: false,
                }; 5];
                let mut t = 0usize;
                while t < size {
                    let (s, e) = shapes[sel[t]];
                    let (u, d, c) = census_meta(s, e);
                    ivs[t] = LiveInterval {
                        vreg: t as u32,
                        start: s,
                        end: e,
                        uses: u,
                        loop_depth: d,
                        size_cost: c,
                        fixed: false,
                    };
                    t += 1;
                }
                let intervals = &ivs[..size];
                // Enumerate the coalescibility configurations of this size.
                let mut tables: Vec<Vec<CoalescePair>> = Vec::new();
                if size <= 3 {
                    // All ordered pairs (i, j), i ≠ j.
                    let mut op: Vec<CoalescePair> = Vec::new();
                    let mut a = 0usize;
                    while a < size {
                        let mut b = 0usize;
                        while b < size {
                            if a != b {
                                op.push(CoalescePair { a: a as u32, b: b as u32 });
                            }
                            b += 1;
                        }
                        a += 1;
                    }
                    // Every subset of ≤ 2 ordered pairs (sizes ≤ 3 → ≤ 6 pairs → ≤ 64 masks).
                    let mut mask = 0u64;
                    let total_masks = 1u64 << op.len();
                    while mask < total_masks {
                        if mask.count_ones() <= 2 {
                            let mut tab = Vec::new();
                            let mut q = 0usize;
                            while q < op.len() {
                                if mask & (1u64 << q) != 0 {
                                    tab.push(op[q]);
                                }
                                q += 1;
                            }
                            tables.push(tab);
                        }
                        mask += 1;
                    }
                } else {
                    // Empty table + every single unordered pair in both directions.
                    tables.push(Vec::new());
                    let mut a = 0usize;
                    while a < size {
                        let mut b = a + 1;
                        while b < size {
                            tables.push(vec![
                                CoalescePair { a: a as u32, b: b as u32 },
                                CoalescePair { a: b as u32, b: a as u32 },
                            ]);
                            b += 1;
                        }
                        a += 1;
                    }
                }
                // Allocate once per (multiset, table); cycle the order length 1..=3.
                let mut ti = 0usize;
                while ti < tables.len() {
                    let l = 1 + (instance % 3) as usize;
                    let order: Vec<u16> = (0..l).map(|r| r as u16).collect();
                    let prob = RegAllocProblem {
                        intervals,
                        coalesce: &tables[ti],
                        class_order: &order,
                    };
                    let st = allocate(&prob, &mut out1[..size])
                        .unwrap_or_else(|e| panic!("instance {instance} table {ti}: unexpected {e:?}"));
                    assert_laws_hold(intervals, &tables[ti], &order, &out1[..size], st);
                    // Determinism: a second run must be identical (G14).
                    let st2 = allocate(&prob, &mut out2[..size])
                        .unwrap_or_else(|e| panic!("instance {instance}: second run {e:?}"));
                    assert_eq!(st, st2);
                    assert_eq!(out1[..size], out2[..size]);
                    instance += 1;
                    ti += 1;
                }
                // Multiset successor over the 21 grid shapes.
                if !next_multiset(&mut sel, size, 21) {
                    break;
                }
            }
            size += 1;
        }
        assert_eq!(instance, 698_720, "census must enumerate exactly 698,720 instances");
    }

    // ----- EXH 9: fixed-interval spill paths ---------------------------------

    /// EXH certificate (fixed census): all multisets of size 1..=3 over the 21
    /// grid shapes × every non-empty fixed-interval mask (2^k − 1) × the orders
    /// {[] (0 available), [0] (1 available)}, empty coalescibility table:
    /// exactly 26,222 instances. Ok outcomes obey both laws (fixed intervals are
    /// never spilled); Err outcomes must be exactly `SpillImpossible`
    /// (OB-16-7 arm 2); with 0 available registers every instance must fail with
    /// `SpillImpossible` because at least one fixed interval cannot be located.
    #[test]
    fn exh_fixed_intervals_spill_paths() {
        let shapes = grid_shapes();
        let mut out = [Location::Register(0); 3];
        let mut instance: u64 = 0;
        let mut size = 1usize;
        while size <= 3 {
            let mut sel = [0usize; 3];
            loop {
                // Every non-empty fixed mask over the k intervals.
                let mut fmask = 1u64;
                let total = 1u64 << size;
                while fmask < total {
                    let mut ivs = [LiveInterval {
                        vreg: 0,
                        start: 0,
                        end: 0,
                        uses: 0,
                        loop_depth: 0,
                        size_cost: 0,
                        fixed: false,
                    }; 3];
                    let mut t = 0usize;
                    while t < size {
                        let (s, e) = shapes[sel[t]];
                        let (u, d, c) = census_meta(s, e);
                        ivs[t] = LiveInterval {
                            vreg: t as u32,
                            start: s,
                            end: e,
                            uses: u,
                            loop_depth: d,
                            size_cost: c,
                            fixed: fmask & (1u64 << t) != 0,
                        };
                        t += 1;
                    }
                    let intervals = &ivs[..size];
                    // Order with 0 available registers: fixed cannot ever be
                    // located → SpillImpossible exactly.
                    let prob0 = RegAllocProblem { intervals, coalesce: &[], class_order: &[] };
                    assert_eq!(
                        allocate(&prob0, &mut out[..size]),
                        Err(RegAllocError::SpillImpossible),
                        "instance {instance} order []"
                    );
                    instance += 1;
                    // Order with 1 available register: Ok → laws (fixed never
                    // spilled); Err → exactly SpillImpossible.
                    let order1 = [0u16];
                    let prob1 = RegAllocProblem { intervals, coalesce: &[], class_order: &order1 };
                    match allocate(&prob1, &mut out[..size]) {
                        Ok(st) => {
                            assert_laws_hold(intervals, &[], &order1, &out[..size], st);
                        }
                        Err(e) => {
                            assert_eq!(e, RegAllocError::SpillImpossible, "instance {instance}");
                        }
                    }
                    instance += 1;
                    fmask += 1;
                }
                // Multiset successor over the 21 grid shapes.
                if !next_multiset(&mut sel, size, 21) {
                    break;
                }
            }
            size += 1;
        }
        assert_eq!(instance, 26_222, "fixed census must enumerate exactly 26,222 instances");
    }

    // ----- PROP: larger instances, paired with the EXH census and Z3 ---------

    /// PROP certificate (paired with EXH exh_allocation_laws_grid_multisets and
    /// the Z3 bounded check per SPEC-INDEX §4.3): 5,000 deterministic xorshift64*
    /// instances of 1..=64 intervals on a 0..=223 position grid, random
    /// directional coalescibility tables (≤ 8 ordered pairs, asymmetric tables
    /// included), fixed flags at 1/8 rate, class orders of 1..=4 registers over
    /// encodings {0,1,2,3}. Every Ok output must satisfy both pinned laws, the
    /// slot discipline, and the pressure bound (checked by the SAME
    /// `assert_laws_hold` used by the EXH census); every Err must be exactly
    /// `SpillImpossible`; every instance must be deterministic across two runs.
    #[test]
    fn prop_laws_larger_instances() {
        const PROP_INSTANCES: u64 = 5_000;
        let mut rng = Xorshift64Star::new();
        let mut instance: u64 = 0;
        while instance < PROP_INSTANCES {
            let n = 1 + rng.below(64) as usize; // 1..=64 intervals
            let mut intervals: Vec<LiveInterval> = Vec::with_capacity(n);
            let mut i = 0usize;
            while i < n {
                let s = rng.below(192) as u32;
                let e = s + rng.below(32) as u32; // e ≤ 223, s ≤ e
                intervals.push(LiveInterval {
                    vreg: i as u32,
                    start: s,
                    end: e,
                    uses: 1 + rng.below(7) as u32,   // ≥ 1 → priority never underflows
                    loop_depth: rng.below(9) as u32, // ≤ 8 ≤ 64
                    size_cost: rng.below(65) as u32, // ≤ 64
                    fixed: rng.below(8) == 0,
                });
                i += 1;
            }
            // Random directional coalescibility table (≤ 8 ordered pairs).
            let mut pairs: Vec<CoalescePair> = Vec::new();
            if n >= 2 {
                let p = rng.below(9) as usize;
                let mut q = 0usize;
                while q < p {
                    let a = rng.below(n as u64) as u32;
                    let b = (a + 1 + rng.below((n as u64) - 1) as u32) % (n as u32);
                    pairs.push(CoalescePair { a, b }); // a ≠ b by construction
                    q += 1;
                }
            }
            let l = 1 + rng.below(4) as usize; // 1..=4 registers
            let order: Vec<u16> = (0..l).map(|r| r as u16).collect();
            let prob = RegAllocProblem {
                intervals: &intervals,
                coalesce: &pairs,
                class_order: &order,
            };
            let mut out1 = vec![Location::Register(0); n];
            let mut out2 = vec![Location::Register(0); n];
            let r1 = allocate(&prob, &mut out1);
            match r1 {
                Ok(st) => {
                    assert_laws_hold(&intervals, &pairs, &order, &out1, st);
                }
                Err(e) => {
                    assert_eq!(e, RegAllocError::SpillImpossible, "instance {instance}");
                }
            }
            // Determinism: the second run must return the identical result and
            // identical locations (both for Ok and for SpillImpossible).
            let r2 = allocate(&prob, &mut out2);
            assert_eq!(r1, r2, "instance {instance} nondeterministic result");
            assert_eq!(out1, out2, "instance {instance} nondeterministic locations");
            instance += 1;
        }
    }
}
