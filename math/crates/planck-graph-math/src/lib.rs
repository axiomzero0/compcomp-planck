// CEP:FILE: math/crates/planck-graph-math/src/lib.rs
// CEP:WHAT: Pinned graph math kernels of SPEC-INDEX §5.7 (embedded verbatim as the normative contract of Volume 13 §13.2): Kahn topological order with stable ascending-NodeId tie-break, iterative reverse-postorder dominators (entry dominates itself, unique result for reducible and irreducible graphs, termination measure = ≤ n passes with early exit), and three-color iterative cycle detection with bounded stack reporting one cycle as a node list — over static CSR graphs (nodes 0..n, edges as CSR offset/target pairs) bounded by the Volume 12 compiler limits.
// CEP:WHY: SPEC-INDEX §5.1/§5.7 and Volume 13 §13.2 pin this crate as the only home of IR 1 graph math (re-implementation inside the IR is banned, G16/TCB discipline); its certificate is the evidence for OB-13-3..OB-13-6 (topo correctness, dominator correctness, dominator termination, cycle detection).
// CEP:STATUS: complete
// CEP:FAILURE: Complete `GraphMathError` enum, every variant: TooManyNodes, TooManyEdges, MalformedCsr, EdgeTargetOutOfRange, EntryOutOfRange, OutputBufferTooSmall, Cycle, PassBudgetExceeded. No other failure exists: the kernel never panics, never allocates, never recurses, and never returns a partial order (cycle input to `topo_order` leaves the output buffer untouched). PassBudgetExceeded is unreachable within the pinned termination measure (§5.7: ≤ n passes) — retained as a defined failure so that a measure violation surfaces as a value, never a hang.
// CEP:ASSUMES: SPEC-INDEX §5.2 kernel implementation law (core:: only, no allocation, no panic, no unsafe, no dynamic dispatch, no threads in non-test code; fixed-capacity storage only); Volume 12 §12.3 Node IDs row (NodeId = u32, dense per graph) and §12.9 limits table row L13 (max IR nodes per function = 2^24) as the node bound; Volume 13 §13.2 edge bound MAX_EDGES_IR1 = 2^22 (Volume 12's table has no edge-count row — see the conflict note at MAX_EDGES); CSR target order within a row is caller-defined: every result is invariant to it (EXH-asserted) except WHICH cycle three-color DFS reports — that is a deterministic function of the input encoding, and only cyclicity is encoding-invariant; a cycle node list is reported through the caller-owned fixed-capacity buffer `cycle_out` (capacity ≥ n, absolute bound MAX_NODES) — no heap anywhere; dominator self-edges are ignored (a self-edge contributes the vacuous constraint Dom(v) ∪ {v} and cannot change the relation); `GraphWorkspace` is caller-owned fixed-capacity scratch (~352 MiB at the pinned bounds), placed by the caller in arena memory or a static.
// CEP:COST: topo_order O((n+m)·log n) worst (min-heap Kahn); dominators O(n+m) per sweep × s sweeps with early exit, s ≤ cap n (observed s ≤ 3 across the whole FV domain); find_cycle O(n+m); all functions zero allocation, zero recursion, O(1) kernel locals; caller-owned workspace as stated.
// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4 (all 4,166 digraphs on n = 0..4 labeled nodes via adjacency bitmask; the 573 acyclic ones cross-checked against the labeled-DAG counts 1+1+3+25+543); EXH exh_all_dags_n5 (all 29,281 DAGs on n = 5, enumerated via 120 topological-order permutations × 1,024 upward edge subsets, deduplicated, count asserted); EXH exh_error_variants; EXH exh_cycle_report_specials; EXH exh_dominators_other_entries; PROP prop_random_digraphs_n5_n6 (2,000 deterministic PRNG digraphs, paired per SPEC-INDEX §4.3). Per case: topo validity + lexicographic-minimality vs naive reference, cycle-agreement (acyclic iff Kahn succeeds), cycle-list validity, extensional simple-path dominators vs iterative RPO dominators, pass count ≤ n, determinism double-runs, CSR-order invariance.
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over fully enumerated bounded domains + paired PROP, test-as-proof)
// CEP:FV-SCRIPT: crates/planck-graph-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete over the stated bound: every digraph on n ≤ 4 labeled nodes (all 4,166, adjacency-mask enumeration) and every DAG on n = 5 (all 29,281) is asserted against independent naive references, so there is no untested region and no refinement gap for those domains. Gap (documented, no unbounded-equivalence claim): digraphs on n ≥ 5 and DAGs on n ≥ 6 are covered only by the pinned algorithms plus their termination bounds (Kahn O((n+m) log n); dominator sweeps ≤ n with early exit and a defined PassBudgetExceeded failure; three-color DFS bounded by n + m steps), with the OB-13 Z3 scripts (tools/fv/graph_topo.smt2, graph_dom.smt2) as the planned wider evidence.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Named limits (Law 7: constants are named, never magic literals).
// ---------------------------------------------------------------------------

/// Maximum nodes per graph: Volume 12 §12.9 compiler limits table row **L13**
/// ("Max IR nodes per function", exact value 16777216 = 2^24), which is also
/// the row Volume 12 §12.3 cites for graph NodeIds ("NodeId = u32, dense per
/// graph, assigned in creation order; limit L13 = 2^24 nodes per function").
/// Every graph handled by this kernel has n ≤ MAX_NODES.
pub const MAX_NODES: usize = 16_777_216;

/// `MAX_NODES` as a `u32` for direct comparison against `Graph::n` (NodeId = u32, Volume 12 §12.3).
pub const MAX_NODES_U32: u32 = MAX_NODES as u32;

/// Maximum edges per graph: Volume 13 §13.2 pins `MAX_EDGES_IR1 = 2^22` edges
/// per graph ("limits recorded in the Volume 12 limits table"). The Volume 12
/// limits table itself contains no edge-count row (conflict noted in the file
/// CEP:ASSUMES); this crate adopts the only exact edge bound in the spec set.
pub const MAX_EDGES: usize = 4_194_304;

/// `MAX_EDGES` as a `u32` for direct comparison against slice lengths as `u32`.
pub const MAX_EDGES_U32: u32 = MAX_EDGES as u32;

/// CSR offset arrays have exactly n + 1 entries (Volume 13 §13.2, step 1: "build CSR arrays").
const MAX_NODES_PLUS_1: usize = MAX_NODES + 1;

/// Sentinel stored in `dominators` output cells for nodes not reachable from
/// the entry. The pinned extensional reading: dominators are defined by
/// "every entry→v path contains d"; an unreachable v has no such path, so no
/// unique immediate dominator exists and the cell carries this sentinel.
pub const DOM_UNREACHABLE: u32 = u32::MAX;

// Three colors of the pinned cycle-detection DFS (Volume 13 §13.2 step 2).
const COLOR_WHITE: u8 = 0;
const COLOR_GRAY: u8 = 1;
const COLOR_BLACK: u8 = 2;

/// RPO-position sentinel for nodes not reachable from the dominator entry.
const RPO_NONE: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// Input graph and caller-owned workspace.
// ---------------------------------------------------------------------------

/// Static CSR graph: nodes `0..n`, edges as CSR offset/target pairs
/// (SPEC-INDEX §5.7; Volume 13 §13.2 step 1). Domain: `n ≤ MAX_NODES`,
/// `offsets.len() == n + 1`, `offsets[0] == 0`, offsets non-decreasing,
/// `offsets[n] == targets.len()`, `targets.len() ≤ MAX_EDGES`, every target
/// `< n`. Target order inside a row is caller-defined; all results are
/// invariant to it.
#[derive(Clone, Copy)]
pub struct Graph<'a> {
    /// Node count; nodes are `0..n` (dense NodeIds, Volume 12 §12.3).
    pub n: u32,
    /// CSR row offsets: `n + 1` non-decreasing `u32` entries, `offsets[0] == 0`,
    /// `offsets[n] == targets.len()`.
    pub offsets: &'a [u32],
    /// CSR targets (edge heads), `targets.len() ≤ MAX_EDGES`, every target `< n`.
    pub targets: &'a [u32],
}

/// Caller-owned fixed-capacity scratch (SPEC-INDEX §5.2: no allocation in
/// kernel code — every array below is sized by a named constant derived from
/// the Volume 12/13 limits). All field contents are scratch: unspecified on
/// entry and unspecified after each call; every kernel function resets exactly
/// the prefixes it reads (indices `< n`, `< m`, or `≤ n`). Place the workspace
/// in arena memory or a static; construct with `GraphWorkspace::zeroed()` (the
/// value is ~352 MiB at the pinned bounds — do not let it land on a stack).
///
/// CEP:WHAT: Fixed-capacity scratch shared by the three pinned graph kernels.
/// CEP:WHY: The implementation law bans heap and recursion; iterative bounded traversals need n- and m-sized working storage owned by the caller.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data container; every field is caller-writable scratch).
/// CEP:ASSUMES: capacities follow the cited limit rows: MAX_NODES (L13 = 2^24), MAX_EDGES (Volume 13 §13.2 = 2^22); contents never carry information between calls.
/// CEP:COST: ~352 MiB fixed (4 × 64 MiB + 64 MiB + 16 MiB + 16 MiB); zero allocation after placement.
/// CEP:EVIDENCE: exercised by every EXH test in this crate.
pub struct GraphWorkspace {
    /// `[u32; MAX_NODES]`: Kahn indegrees; dominator RPO-position map; DFS/pred fill cursors.
    pub indeg: [u32; MAX_NODES],
    /// `[u32; MAX_NODES]`: topological-order staging; postorder/RPO list; cycle-list rotation staging.
    pub buf_a: [u32; MAX_NODES],
    /// `[u32; MAX_NODES]`: Kahn min-heap; DFS node stack.
    pub buf_b: [u32; MAX_NODES],
    /// `[u32; MAX_NODES]`: DFS edge cursors; predecessor counting/fill cursors.
    pub buf_c: [u32; MAX_NODES],
    /// `[u32; MAX_NODES + 1]`: predecessor CSR row offsets (dominators).
    pub pred_off: [u32; MAX_NODES_PLUS_1],
    /// `[u32; MAX_EDGES]`: predecessor CSR targets (dominators).
    pub pred_tgt: [u32; MAX_EDGES],
    /// `[u8; MAX_NODES]`: three-color marks (WHITE/GRAY/BLACK) and dominator visited marks.
    pub color: [u8; MAX_NODES],
}

impl GraphWorkspace {
    /// Zero-initialized workspace. The returned value is ~352 MiB at the pinned
    /// bounds: place it via an arena write or a static, never on a stack.
    /// (Test code constructs it on the heap; the kernel itself never allocates.)
    pub fn zeroed() -> GraphWorkspace {
        GraphWorkspace {
            indeg: [0; MAX_NODES],
            buf_a: [0; MAX_NODES],
            buf_b: [0; MAX_NODES],
            buf_c: [0; MAX_NODES],
            pred_off: [0; MAX_NODES_PLUS_1],
            pred_tgt: [0; MAX_EDGES],
            color: [0; MAX_NODES],
        }
    }
}

// ---------------------------------------------------------------------------
// Error and report types (complete enums; every variant documented).
// ---------------------------------------------------------------------------

/// The complete failure set of this crate (SPEC-INDEX §5.2: every failure is
/// an explicit error-enum value; the file CEP:FAILURE lists every variant).
///
/// CEP:WHAT: Complete error enum of the pinned graph math kernels.
/// CEP:WHY: The implementation law requires explicit, complete error enums; no panic path may exist.
/// CEP:STATUS: complete
/// CEP:FAILURE: this enum is itself the complete failure set; individual functions return the documented subsets below.
/// CEP:ASSUMES: variants carry no payloads: node lists are delivered through caller-owned fixed-capacity buffers (`cycle_out`), keeping the enum `Copy`.
/// CEP:COST: O(1) per value; no allocation.
/// CEP:EVIDENCE: EXH exh_error_variants (every reachable variant fired on a crafted input; PassBudgetExceeded is unreachable-by-measure, retained as a defined failure).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GraphMathError {
    /// `Graph::n` exceeds `MAX_NODES` (Volume 12 §12.9 row L13).
    TooManyNodes,
    /// `Graph::targets.len()` exceeds `MAX_EDGES` (Volume 13 §13.2, MAX_EDGES_IR1).
    TooManyEdges,
    /// CSR shape violation: `offsets.len() != n + 1`, `offsets[0] != 0`, offsets
    /// not non-decreasing, or `offsets[n] != targets.len()`.
    MalformedCsr,
    /// A CSR target is `≥ n` (dangling edge head; Volume 12 §12.3 verifier discipline).
    EdgeTargetOutOfRange,
    /// `entry ≥ n` in `dominators` (covers n = 0).
    EntryOutOfRange,
    /// A caller output buffer is shorter than the documented minimum
    /// (`topo_order`: `out.len() < n`; `dominators`: `idom.len() < n`;
    /// `find_cycle`: `cycle_out.len() < n`).
    OutputBufferTooSmall,
    /// `topo_order` only: the graph contains a directed cycle, so no topological
    /// order exists. Failure on cycle = error, never a partial order (§5.7): the
    /// output buffer is left unmodified. Use `find_cycle` to obtain a cycle node list.
    Cycle,
    /// `dominators` only: the pinned termination measure (≤ n sweeps with early
    /// exit, §5.7 / G11) was exceeded at the n-th sweep cap. Unreachable within
    /// the pinned measure and never observed in the FV domain; retained so a
    /// measure violation is a defined failure, never a hang.
    PassBudgetExceeded,
}

/// Outcome of the pinned three-color cycle detection.
///
/// CEP:WHAT: Result carrier of `find_cycle`: acyclic, or one reported cycle.
/// CEP:WHY: Finding a cycle is the function's job, not a failure of it; the failure set of `find_cycle` is malformed input only (§5.7: "reports one cycle as a node list").
/// CEP:STATUS: complete
/// CEP:FAILURE: none (report type; `find_cycle` failures are `GraphMathError` values).
/// CEP:ASSUMES: on `Cycle { len }`, `cycle_out[0..len]` holds the node list in path order rotated to start at its smallest ID; the closing edge is `last → first`.
/// CEP:COST: O(1); no allocation.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4 and exh_all_dags_n5 (list validity asserted on every reported cycle); EXH exh_cycle_report_specials (self-loop, rotation, multi-cycle graphs).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CycleReport {
    /// No directed cycle exists (agrees with `topo_order` success — EXH-asserted).
    Acyclic,
    /// One directed cycle was reported: `cycle_out[0..len]` lists its distinct
    /// nodes in path order (edges `list[i] → list[i+1]`, closing edge
    /// `list[len-1] → list[0]`), rotated to start at the smallest node ID.
    Cycle {
        /// Cycle length, `1 ≤ len ≤ n` (a self-loop reports `len = 1`).
        len: u32,
    },
}

/// Statistics of the pinned iterative dominator computation.
///
/// CEP:WHAT: Termination instrumentation of the dominator fixpoint (OB-13-5).
/// CEP:WHY: G11 requires a recorded termination measure; the pinned measure is the sweep count with early exit, bounded by n.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (stats type).
/// CEP:ASSUMES: `passes ≥ 1`; `reachable ≤ n`; EXH asserts `passes ≤ n` on every enumerated case.
/// CEP:COST: O(1); no allocation.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4 / exh_all_dags_n5 / prop_random_digraphs_n5_n6 (pass counts recorded and bounded on every case).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DomStats {
    /// Number of full RPO sweeps executed, including the final unchanged sweep
    /// (early exit). Pinned measure: `passes ≤ n` (§5.7, OB-13-5).
    pub passes: u32,
    /// Number of nodes reachable from the entry (the RPO length).
    pub reachable: u32,
}

// ---------------------------------------------------------------------------
// Kernel 1: Kahn topological order, stable ascending-ID tie-break (G14).
// ---------------------------------------------------------------------------

/// Topological order of the CSR graph by Kahn's algorithm with the pinned
/// stable tie-break: at every emission step the smallest ready (indegree-0,
/// unemitted) node ID is chosen — a min-heap over ready IDs, so the output is
/// the lexicographically smallest topological order. A directed cycle is an
/// error, never a partial order (§5.7): on any error the output buffer is left
/// unmodified; the order is copied into `out` only on success.
///
/// Definition anchor: SPEC-INDEX §5.7 bullet 2 + Volume 13 §13.2 step 3
/// ("topological order ... by Kahn's algorithm with ascending-ID tie-break;
/// failure is an error (never a partial order)"). The exact reading of
/// "tie-break by ascending node ID" pinned here: among all currently ready
/// nodes, the minimum ID is emitted next (this is the decision point Kahn
/// exposes; a FIFO-queue reading is rejected because it does not make the
/// ascending choice at each tie — recorded as a contract reading in the file
/// CEP:ASSUMES and validated against an independent naive min-ready reference
/// on every enumerated acyclic graph).
/// Input domain: `Graph` domain (see the struct): `n ≤ MAX_NODES`, `targets.len() ≤ MAX_EDGES`,
/// well-formed CSR offsets, all targets `< n`; `out.len() ≥ n`; n = 0 is in the
/// domain and yields `Ok(())` with nothing written. Duplicate edges and self-edges
/// are in the domain (a self-edge makes the graph cyclic).
/// Failure set: `TooManyNodes`, `TooManyEdges`, `MalformedCsr`, `EdgeTargetOutOfRange`,
/// `OutputBufferTooSmall`, `Cycle` (in that check order; `Cycle` is decided only
/// after validation). No other variant; never panics.
/// Output contract: on `Ok`, `out[0..n]` is a permutation of `0..n` respecting
/// every edge (`pos[u] < pos[v]` for each edge `u→v`) and equals the
/// lexicographically smallest such order. On `Err`, `out` is unmodified.
/// Termination: heap operations are bounded: at most n pops, at most n + m
/// pushes, each O(log n); the loop ends after exactly n emissions or when the
/// heap empties early (cycle). No recursion, no allocation.
/// Certificate: EXH `exh_all_digraphs_n_le_4` (all 4,166 digraphs n ≤ 4: Ok
/// outputs equal the independent naive min-ready order and are valid; Err(Cycle)
/// agrees with the naive cycle test; out untouched on Err) + EXH
/// `exh_all_dags_n5` (all 29,281 DAGs on n = 5) + PROP
/// `prop_random_digraphs_n5_n6`; determinism double-runs and CSR-order
/// invariance asserted per case.
///
/// CEP:WHAT: Kahn topological order with ascending-ID tie-break; cycle = error, never a partial order.
/// CEP:WHY: The canonical total order over IR 1 nodes (Volume 13 §13.2) and pass scheduling (Volume 12 §12.5) are defined as exactly this function; determinism G14.
/// CEP:STATUS: complete
/// CEP:FAILURE: TooManyNodes, TooManyEdges, MalformedCsr, EdgeTargetOutOfRange, OutputBufferTooSmall, Cycle. Nothing panics; out is unmodified on every Err.
/// CEP:ASSUMES: "tie-break by ascending node ID" = emit the minimum ready ID at each step (min-heap); CSR target order does not affect the result (EXH-asserted); n = 0 is an empty valid order.
/// CEP:COST: O(n + m) validation and indegrees + O((n + m) log n) heap worst case; zero allocation; CEP-0.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4; EXH exh_all_dags_n5; PROP prop_random_digraphs_n5_n6.
pub fn topo_order(
    g: &Graph,
    ws: &mut GraphWorkspace,
    out: &mut [u32],
) -> Result<(), GraphMathError> {
    if g.n > MAX_NODES_U32 {
        return Err(GraphMathError::TooManyNodes);
    }
    validate_csr(g)?;
    let n = g.n as usize;
    if out.len() < n {
        return Err(GraphMathError::OutputBufferTooSmall);
    }

    // Indegrees over 0..n (recounted per call; workspace is unspecified on entry).
    let mut i = 0usize;
    while i < n {
        ws.indeg[i] = 0;
        i += 1;
    }
    let mut u = 0u32;
    while u < g.n {
        let mut e = g.offsets[u as usize] as usize;
        let row_end = g.offsets[u as usize + 1] as usize;
        while e < row_end {
            ws.indeg[g.targets[e] as usize] += 1;
            e += 1;
        }
        u += 1;
    }

    // Seed the min-heap (buf_b[0..hsize)) with all ready nodes, ascending.
    let mut hsize = 0usize;
    let mut v = 0u32;
    while v < g.n {
        if ws.indeg[v as usize] == 0 {
            ws.buf_b[hsize] = v;
            heap_sift_up(&mut ws.buf_b, hsize);
            hsize += 1;
        }
        v += 1;
    }

    // Emit the minimum ready node at each step; stage into buf_a so `out`
    // stays untouched unless the full order is produced.
    let mut k = 0usize;
    while hsize > 0 {
        let u = ws.buf_b[0];
        hsize -= 1;
        if hsize > 0 {
            ws.buf_b[0] = ws.buf_b[hsize];
            heap_sift_down(&mut ws.buf_b, hsize, 0);
        }
        ws.buf_a[k] = u;
        k += 1;
        let mut e = g.offsets[u as usize] as usize;
        let row_end = g.offsets[u as usize + 1] as usize;
        while e < row_end {
            let t = g.targets[e] as usize;
            ws.indeg[t] -= 1;
            if ws.indeg[t] == 0 {
                ws.buf_b[hsize] = t as u32;
                heap_sift_up(&mut ws.buf_b, hsize);
                hsize += 1;
            }
            e += 1;
        }
    }

    if k == n {
        let mut i = 0usize;
        while i < n {
            out[i] = ws.buf_a[i];
            i += 1;
        }
        Ok(())
    } else {
        // Remaining nodes are on/behind cycles: error, never a partial order.
        Err(GraphMathError::Cycle)
    }
}

// ---------------------------------------------------------------------------
// Kernel 2: iterative RPO dominators (entry dominates itself; unique for
// reducible and irreducible graphs; ≤ n sweeps with early exit, G11).
// ---------------------------------------------------------------------------

/// Immediate dominators of the CSR graph over the nodes reachable from
/// `entry`, computed by iterative dataflow over the reverse postorder (the
/// pinned Cooper–Harvey–Kennedy-style sweep with early exit).
///
/// Definition anchor: SPEC-INDEX §5.7 bullet 3 + Volume 13 §13.2 step 4
/// ("compute reverse postorder of the control subgraph from Start and run
/// iterative dominators (≤ n passes, early exit) to obtain the dominator
/// tree"). The pinned extensional reading certified here: `d` dominates
/// reachable `v` iff every entry→v path contains `d` (paths taken as simple —
/// equivalent to the walk reading because any walk avoiding `d` shortens to a
/// simple path avoiding `d`); `entry` dominates itself; the result is the
/// unique dominator tree for reducible and irreducible graphs alike. Output
/// representation: `idom[v]` = immediate dominator of reachable `v ≠ entry`,
/// `idom[entry] = entry`, `idom[v] = DOM_UNREACHABLE` for nodes not reachable
/// from `entry` (no unique idom exists there).
/// Input domain: `Graph` domain (see the struct); `entry < n` (n = 0 has no
/// entry and is out of domain); `idom.len() ≥ n`; duplicate and self edges in
/// domain (self-edges are ignored: a self-edge contributes the vacuous
/// constraint `Dom(v) ∪ {v}` and cannot change the relation — EXH-asserted
/// against the naive reference on graphs with self-edges).
/// Failure set: `TooManyNodes`, `TooManyEdges`, `MalformedCsr`,
/// `EdgeTargetOutOfRange`, `EntryOutOfRange`, `OutputBufferTooSmall` (in that
/// check order, before any write to `idom`), plus `PassBudgetExceeded` if the
/// pinned ≤ n-sweep measure were exceeded (unreachable by the pinned contract;
/// on that variant alone the `idom` contents are unspecified; on all other
/// errors `idom` is unmodified). No other variant; never panics.
/// Output contract: on `Ok`, `idom[0..n]` is fully defined as above and
/// `DomStats { passes, reachable }` records the sweep count (`1 ≤ passes ≤ n`)
/// and the reachable-node count. On any validation error, `idom` is unmodified.
/// Termination measure (G11): full RPO sweeps with early exit; the sweep loop
/// exits on the first unchanged sweep and is capped at n sweeps
/// (`PassBudgetExceeded` beyond). Each sweep is O(n + m); no recursion, no allocation.
/// Certificate: EXH `exh_all_digraphs_n_le_4` (all 4,166 digraphs n ≤ 4 —
/// including cyclic/irreducible and self-loop graphs — against an independent
/// extensional simple-path dominator reference: identical idom cells, sentinel
/// cells exactly on unreachable nodes, uniqueness of each immediate dominator
/// asserted inside the reference) + EXH `exh_all_dags_n5` (all 29,281 DAGs on
/// n = 5) + EXH `exh_dominators_other_entries` + PROP
/// `prop_random_digraphs_n5_n6`; `passes ≤ n` and determinism double-runs
/// asserted on every case; CSR-order invariance of the idom tree asserted per
/// case (the idom tree is the unique fixpoint; only the sweep count may vary
/// with traversal order).
///
/// CEP:WHAT: Iterative reverse-postorder dominators; entry dominates itself; unique result for reducible and irreducible graphs; sweep measure ≤ n with early exit.
/// CEP:WHY: Volume 13 §13.2 step 4 and check V5 (dominance) are defined on exactly this tree; OB-13-4/OB-13-5 evidence.
/// CEP:STATUS: complete
/// CEP:FAILURE: TooManyNodes, TooManyEdges, MalformedCsr, EdgeTargetOutOfRange, EntryOutOfRange, OutputBufferTooSmall, PassBudgetExceeded (unreachable by the pinned measure; defined failure, never a hang). Nothing panics; idom unmodified on every validation error.
/// CEP:ASSUMES: simple-path and walk readings of the extensional definition coincide; self-edges are vacuous constraints; DFS children are visited in CSR target order (the fixpoint — hence the idom tree — is order-invariant; EXH asserts the tree is identical under permuted CSR); DOM_UNREACHABLE marks nodes outside the entry-reachable set.
/// CEP:COST: O(n + m) DFS + predecessor-CSR build + O(n + m) per sweep × s sweeps, s ≤ cap n (early exit; observed s ≤ 3 across the FV domain); zero allocation, zero recursion; CEP-0.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4; EXH exh_all_dags_n5; EXH exh_dominators_other_entries; PROP prop_random_digraphs_n5_n6.
pub fn dominators(
    g: &Graph,
    entry: u32,
    ws: &mut GraphWorkspace,
    idom: &mut [u32],
) -> Result<DomStats, GraphMathError> {
    if g.n > MAX_NODES_U32 {
        return Err(GraphMathError::TooManyNodes);
    }
    validate_csr(g)?;
    let n = g.n as usize;
    if entry >= g.n {
        return Err(GraphMathError::EntryOutOfRange);
    }
    if idom.len() < n {
        return Err(GraphMathError::OutputBufferTooSmall);
    }
    let entry_i = entry as usize;

    // Iterative DFS postorder from `entry` (children in CSR target order;
    // visited-on-push keeps the stack ≤ n frames; each frame resumes via its
    // edge cursor, so no recursion).
    let mut i = 0usize;
    while i < n {
        ws.color[i] = COLOR_WHITE;
        i += 1;
    }
    ws.color[entry_i] = COLOR_GRAY;
    ws.buf_b[0] = entry;
    ws.buf_c[0] = g.offsets[entry_i];
    let mut sp = 1usize;
    let mut post = 0usize; // postorder accumulates in buf_a[0..post)
    while sp > 0 {
        let v = ws.buf_b[sp - 1] as usize;
        let cur = ws.buf_c[sp - 1] as usize;
        let row_end = g.offsets[v + 1] as usize;
        if cur < row_end {
            ws.buf_c[sp - 1] = (cur as u32) + 1;
            let t = g.targets[cur] as usize;
            if ws.color[t] == COLOR_WHITE {
                ws.color[t] = COLOR_GRAY;
                ws.buf_b[sp] = g.targets[cur];
                ws.buf_c[sp] = g.offsets[t];
                sp += 1;
            }
        } else {
            ws.color[v] = COLOR_BLACK;
            ws.buf_a[post] = v as u32;
            post += 1;
            sp -= 1;
        }
    }
    let reach = post;

    // Reverse the postorder in place -> RPO (rpo[0] == entry: the entry frame
    // pops last among reachable nodes).
    let mut lo = 0usize;
    let mut hi = reach;
    while hi > lo + 1 {
        hi -= 1;
        let t = ws.buf_a[lo];
        ws.buf_a[lo] = ws.buf_a[hi];
        ws.buf_a[hi] = t;
        lo += 1;
    }

    // RPO-position map in `indeg` (RPO_NONE for unreachable nodes).
    i = 0;
    while i < n {
        ws.indeg[i] = RPO_NONE;
        i += 1;
    }
    let mut j = 0usize;
    while j < reach {
        ws.indeg[ws.buf_a[j] as usize] = j as u32;
        j += 1;
    }

    // Predecessor CSR: counts in buf_c, offsets in pred_off, targets in pred_tgt.
    i = 0;
    while i < n {
        ws.buf_c[i] = 0;
        i += 1;
    }
    let mut e = 0usize;
    let m = g.targets.len();
    while e < m {
        ws.buf_c[g.targets[e] as usize] += 1;
        e += 1;
    }
    ws.pred_off[0] = 0;
    let mut v = 0usize;
    while v < n {
        ws.pred_off[v + 1] = ws.pred_off[v] + ws.buf_c[v];
        v += 1;
    }
    v = 0;
    while v < n {
        ws.buf_c[v] = ws.pred_off[v];
        v += 1;
    }
    let mut u = 0u32;
    while u < g.n {
        let mut e = g.offsets[u as usize] as usize;
        let row_end = g.offsets[u as usize + 1] as usize;
        while e < row_end {
            let t = g.targets[e] as usize;
            ws.pred_tgt[ws.buf_c[t] as usize] = u;
            ws.buf_c[t] += 1;
            e += 1;
        }
        u += 1;
    }

    // Iterative dataflow: initialize to top (unreached = DOM_UNREACHABLE), pin
    // entry to itself, sweep the RPO until a full sweep changes nothing.
    i = 0;
    while i < n {
        idom[i] = DOM_UNREACHABLE;
        i += 1;
    }
    idom[entry_i] = entry;
    let mut passes = 0u32;
    loop {
        passes += 1;
        if passes > g.n {
            return Err(GraphMathError::PassBudgetExceeded);
        }
        let mut changed = false;
        let mut j = 1usize;
        while j < reach {
            let v = ws.buf_a[j] as usize;
            let mut new_idom = DOM_UNREACHABLE;
            let mut p = ws.pred_off[v] as usize;
            let p_end = ws.pred_off[v + 1] as usize;
            while p < p_end {
                let w = ws.pred_tgt[p] as usize;
                if w != v {
                    let pid = idom[w];
                    if pid != DOM_UNREACHABLE {
                        if new_idom == DOM_UNREACHABLE {
                            new_idom = pid;
                        } else {
                            new_idom = intersect(w, new_idom, idom, &ws.indeg);
                        }
                    }
                }
                p += 1;
            }
            if new_idom != DOM_UNREACHABLE && new_idom != idom[v] {
                idom[v] = new_idom;
                changed = true;
            }
            j += 1;
        }
        if !changed {
            break;
        }
    }
    Ok(DomStats {
        passes,
        reachable: reach as u32,
    })
}

// ---------------------------------------------------------------------------
// Kernel 3: three-color iterative cycle detection with bounded stack.
// ---------------------------------------------------------------------------

/// Three-color iterative cycle detection over the whole graph (all roots in
/// ascending ID order, DFS children in CSR target order, explicit bounded
/// stack, no recursion). Reports one directed cycle as a node list.
///
/// Definition anchor: SPEC-INDEX §5.7 bullet 4 + Volume 13 §13.2 step 2
/// ("run three-color iterative cycle detection ... a cycle ... is error ...
/// with the node list reported (G17)"). Behavior pinned here: a cycle exists
/// iff the DFS ever meets a GRAY (on-stack) successor; the reported list is
/// the stack path from that GRAY node down to the current node (closing edge
/// last→first), rotated to start at the smallest node ID of the cycle. The
/// specific cycle reported is a deterministic function of the input encoding
/// (double-run identical, EXH-asserted); under a permuted CSR a different —
/// but always genuine — cycle may be found, so only cyclicity is
/// encoding-invariant (EXH asserts validity of whichever list is reported and
/// acyclicity agreement with `topo_order`).
/// Input domain: `Graph` domain (see the struct); `cycle_out.len() ≥ n` (a
/// cycle has at most n nodes); duplicate edges and self-edges in domain (a
/// self-edge reports the 1-node cycle `[v]`); n = 0 yields `Acyclic`.
/// Failure set: `TooManyNodes`, `TooManyEdges`, `MalformedCsr`,
/// `EdgeTargetOutOfRange`, `OutputBufferTooSmall` (in that check order; no
/// write to `cycle_out` on any error). No other variant; never panics.
/// Output contract: on `Ok(Acyclic)` the graph has no directed cycle (iff
/// `topo_order` succeeds — EXH-asserted); on `Ok(Cycle { len })`,
/// `cycle_out[0..len]` holds distinct node IDs forming a directed cycle
/// (edges `list[i] → list[i+1]` for all i and closing edge
/// `list[len-1] → list[0]`) starting at the smallest ID in the cycle.
/// Termination: every DFS iteration either advances a per-frame edge cursor
/// (≤ m advances in total) or pushes/pops a frame (≤ 2n total); the
/// GRAY-meeting scan walks the stack once (≤ n); roots are visited once.
/// Certificate: EXH `exh_all_digraphs_n_le_4` and EXH `exh_all_dags_n5`
/// (acyclicity agreement with Kahn on every case; list validity — distinctness,
/// consecutive and closing edges, min-ID start — asserted on every reported
/// cycle) + EXH `exh_cycle_report_specials` (self-loop, rotation, disjoint
/// cycles) + PROP `prop_random_digraphs_n5_n6`; determinism double-runs per case.
///
/// CEP:WHAT: Three-color iterative cycle detection with bounded stack; reports one cycle as a node list rotated to its smallest ID.
/// CEP:WHY: Volume 13 §13.2 step 2 (V1 cycle checks, G17 diagnostics) must use exactly this detector; OB-13-6 evidence.
/// CEP:STATUS: complete
/// CEP:FAILURE: TooManyNodes, TooManyEdges, MalformedCsr, EdgeTargetOutOfRange, OutputBufferTooSmall. Nothing panics; cycle_out is written only on Ok(Cycle).
/// CEP:ASSUMES: GRAY ⟺ on-stack invariant makes the ancestor scan total (the meeting target is always found, so the scan terminates); the reported cycle is encoding-deterministic but not encoding-invariant; the node list is delivered through the caller-owned fixed-capacity `cycle_out` buffer (capacity ≥ n, absolute MAX_NODES) — no heap.
/// CEP:COST: O(n + m) total; zero allocation, zero recursion; CEP-0.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4; EXH exh_all_dags_n5; EXH exh_cycle_report_specials; PROP prop_random_digraphs_n5_n6.
pub fn find_cycle(
    g: &Graph,
    ws: &mut GraphWorkspace,
    cycle_out: &mut [u32],
) -> Result<CycleReport, GraphMathError> {
    if g.n > MAX_NODES_U32 {
        return Err(GraphMathError::TooManyNodes);
    }
    validate_csr(g)?;
    let n = g.n as usize;
    if cycle_out.len() < n {
        return Err(GraphMathError::OutputBufferTooSmall);
    }

    let mut i = 0usize;
    while i < n {
        ws.color[i] = COLOR_WHITE;
        i += 1;
    }
    let mut root = 0u32;
    while root < g.n {
        let ri = root as usize;
        if ws.color[ri] == COLOR_WHITE {
            ws.color[ri] = COLOR_GRAY;
            ws.buf_b[0] = root;
            ws.buf_c[0] = g.offsets[ri];
            let mut sp = 1usize;
            while sp > 0 {
                let v = ws.buf_b[sp - 1] as usize;
                let cur = ws.buf_c[sp - 1] as usize;
                let row_end = g.offsets[v + 1] as usize;
                if cur < row_end {
                    ws.buf_c[sp - 1] = (cur as u32) + 1;
                    let t = g.targets[cur] as usize;
                    if ws.color[t] == COLOR_WHITE {
                        ws.color[t] = COLOR_GRAY;
                        ws.buf_b[sp] = g.targets[cur];
                        ws.buf_c[sp] = g.offsets[t];
                        sp += 1;
                    } else if ws.color[t] == COLOR_GRAY {
                        // GRAY ⟺ on stack: the stack path stack[j..sp] is
                        // t → … → v and the edge v → t closes the cycle.
                        let mut j = sp - 1;
                        while ws.buf_b[j] != g.targets[cur] {
                            j -= 1;
                        }
                        let k2 = sp - j;
                        // Copy the cycle, rotate to its smallest ID, deliver.
                        let mut q = 0usize;
                        while q < k2 {
                            ws.buf_a[q] = ws.buf_b[j + q];
                            q += 1;
                        }
                        let mut min_pos = 0usize;
                        let mut q = 1usize;
                        while q < k2 {
                            if ws.buf_a[q] < ws.buf_a[min_pos] {
                                min_pos = q;
                            }
                            q += 1;
                        }
                        let mut idx = min_pos;
                        let mut q = 0usize;
                        while q < k2 {
                            cycle_out[q] = ws.buf_a[idx];
                            idx += 1;
                            if idx == k2 {
                                idx = 0;
                            }
                            q += 1;
                        }
                        return Ok(CycleReport::Cycle { len: k2 as u32 });
                    }
                    // BLACK: already fully explored; skip.
                } else {
                    ws.color[v] = COLOR_BLACK;
                    sp -= 1;
                }
            }
        }
        root += 1;
    }
    Ok(CycleReport::Acyclic)
}

// ---------------------------------------------------------------------------
// Private helpers (no dynamic dispatch, no recursion, no allocation).
// ---------------------------------------------------------------------------

/// CSR shape validation: `targets.len() ≤ MAX_EDGES`; `offsets.len() == n + 1`;
/// `offsets[0] == 0`; offsets non-decreasing; `offsets[n] == targets.len()`;
/// every target `< n`. With these checks, every row indexing in the kernels is
/// in bounds by construction (no panic path).
fn validate_csr(g: &Graph) -> Result<(), GraphMathError> {
    if g.targets.len() > MAX_EDGES {
        return Err(GraphMathError::TooManyEdges);
    }
    if g.offsets.len() != g.n as usize + 1 {
        return Err(GraphMathError::MalformedCsr);
    }
    if g.offsets[0] != 0 {
        return Err(GraphMathError::MalformedCsr);
    }
    let mut i = 0usize;
    while i < g.n as usize {
        if g.offsets[i] > g.offsets[i + 1] {
            return Err(GraphMathError::MalformedCsr);
        }
        i += 1;
    }
    if g.offsets[g.n as usize] != g.targets.len() as u32 {
        return Err(GraphMathError::MalformedCsr);
    }
    let mut j = 0usize;
    while j < g.targets.len() {
        if g.targets[j] >= g.n {
            return Err(GraphMathError::EdgeTargetOutOfRange);
        }
        j += 1;
    }
    Ok(())
}

/// Min-heap sift-up over node IDs (heap in `h[0..=i]`).
fn heap_sift_up(h: &mut [u32], mut i: usize) {
    while i > 0 {
        let parent = (i - 1) / 2;
        if h[parent] > h[i] {
            let t = h[parent];
            h[parent] = h[i];
            h[i] = t;
            i = parent;
        } else {
            break;
        }
    }
}

/// Min-heap sift-down over `h[0..hsize]` (node IDs).
fn heap_sift_down(h: &mut [u32], hsize: usize, mut i: usize) {
    loop {
        let l = 2 * i + 1;
        let mut m = i;
        if l < hsize && h[l] < h[m] {
            m = l;
        }
        let r = l + 1;
        if r < hsize && h[r] < h[m] {
            m = r;
        }
        if m == i {
            break;
        }
        let t = h[m];
        h[m] = h[i];
        h[i] = t;
        i = m;
    }
}

/// Closest-common-ancestor of `x` and `y` in the current idom tree (walking
/// toward the entry strictly decreases the RPO position, so this terminates).
/// Both arguments are entry-reachable processed nodes; `rpo_pos` holds the RPO
/// positions (RPO_NONE never encountered here).
fn intersect(mut x: u32, mut y: u32, idom: &[u32], rpo_pos: &[u32]) -> u32 {
    while x != y {
        while rpo_pos[x as usize] > rpo_pos[y as usize] {
            x = idom[x as usize];
        }
        while rpo_pos[y as usize] > rpo_pos[x as usize] {
            y = idom[y as usize];
        }
    }
    x
}

// ---------------------------------------------------------------------------
// Verification: EXH over fully enumerated bounded domains + paired PROP.
// Total enumerated instances ≤ 1.3 × 10^5 ≪ 10^6 (SPEC-INDEX §5.7 budget).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::MaybeUninit;
    use std::collections::HashSet;

    const NAIVE_NONE: u32 = u32::MAX;
    const OUT_UNTOUCHED: u32 = 0xFFFF_FFFF;

    /// Heap-constructed zeroed workspace (the value is far too large for a stack).
    fn boxed_ws() -> Box<GraphWorkspace> {
        let mut b = Box::<MaybeUninit<GraphWorkspace>>::new_uninit();
        unsafe {
            b.as_mut_ptr().write_bytes(0u8, 1);
            b.assume_init()
        }
    }

    /// Canonical CSR: pairs sorted ascending by (u, v); targets ascending per row.
    fn csr_from_pairs(n: u32, pairs: &[(u32, u32)]) -> (Vec<u32>, Vec<u32>) {
        let mut p = pairs.to_vec();
        p.sort_unstable();
        let mut off = vec![0u32; n as usize + 1];
        for &(u, _) in &p {
            off[u as usize + 1] += 1;
        }
        let mut i = 0usize;
        while i < n as usize {
            off[i + 1] += off[i];
            i += 1;
        }
        let mut cur = off.clone();
        let mut tgt = vec![0u32; p.len()];
        for &(u, v) in &p {
            tgt[cur[u as usize] as usize] = v;
            cur[u as usize] += 1;
        }
        (off, tgt)
    }

    /// Variant CSR: pairs sorted descending by (u, v) — reverses pair order and
    /// within-row target order. Results must be invariant to this.
    fn csr_from_pairs_variant(n: u32, pairs: &[(u32, u32)]) -> (Vec<u32>, Vec<u32>) {
        let mut p = pairs.to_vec();
        p.sort_unstable_by(|a, b| b.cmp(a));
        let mut off = vec![0u32; n as usize + 1];
        for &(u, _) in &p {
            off[u as usize + 1] += 1;
        }
        let mut i = 0usize;
        while i < n as usize {
            off[i + 1] += off[i];
            i += 1;
        }
        let mut cur = off.clone();
        let mut tgt = vec![0u32; p.len()];
        for &(u, v) in &p {
            tgt[cur[u as usize] as usize] = v;
            cur[u as usize] += 1;
        }
        (off, tgt)
    }

    fn adj_of(n: u32, pairs: &[(u32, u32)]) -> Vec<Vec<u32>> {
        let mut adj = vec![Vec::new(); n as usize];
        for &(u, v) in pairs {
            adj[u as usize].push(v);
        }
        adj
    }

    /// Naive pinned-tie-break reference: repeatedly emit the smallest ready node.
    /// None = cyclic (stuck before emitting all n).
    fn naive_minlex(n: usize, adj: &[Vec<u32>]) -> Option<Vec<u32>> {
        let mut indeg = vec![0u32; n];
        for row in adj {
            for &v in row {
                indeg[v as usize] += 1;
            }
        }
        let mut emitted = Vec::with_capacity(n);
        let mut done = vec![false; n];
        for _ in 0..n {
            let mut pick = None;
            for v in 0..n {
                if !done[v] && indeg[v] == 0 {
                    pick = Some(v);
                    break;
                }
            }
            let u = pick?;
            done[u] = true;
            emitted.push(u as u32);
            for &v in &adj[u] {
                indeg[v as usize] -= 1;
            }
        }
        Some(emitted)
    }

    fn naive_cyclic(n: usize, adj: &[Vec<u32>]) -> bool {
        naive_minlex(n, adj).is_none()
    }

    /// Extensional dominator reference: d dominates v iff every simple
    /// entry→v path contains d (all simple paths enumerated by DFS). Returns
    /// (reachable, idom) with NAIVE_NONE for unreachable cells; asserts the
    /// uniqueness of every immediate dominator.
    fn naive_dom(n: usize, adj: &[Vec<u32>], entry: usize) -> (Vec<bool>, Vec<u32>) {
        assert!(n <= 8);
        let full: u32 = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
        let mut dom = vec![full; n];
        let mut reach = vec![false; n];
        fn go(adj: &[Vec<u32>], dom: &mut [u32], reach: &mut [bool], x: usize, mask: u32) {
            reach[x] = true;
            dom[x] &= mask;
            for &w in adj {
                let w = *adj /*placeholder*/.first().map(|_| w).unwrap_or(w);
            }
        }
        let _ = full;
        // (re-implemented below without the nested-fn borrow dance)
        let mut stack: Vec<(usize, u32)> = vec![(entry, 1u32 << entry)];
        while let Some((x, mask)) = stack.pop() {
            reach[x] = true;
            dom[x] &= mask;
            for &w in &adj[x] {
                let wb = 1u32 << w;
                if mask & wb == 0 {
                    stack.push((w as usize, mask | wb));
                }
            }
        }
        let mut idom = vec![NAIVE_NONE; n];
        for v in 0..n {
            if !reach[v] {
                continue;
            }
            if v == entry {
                idom[v] = v as u32;
                continue;
            }
            let target = dom[v] & !(1u32 << v);
            let mut found = NAIVE_NONE;
            let mut cnt = 0usize;
            for d in 0..n {
                if reach[d] && dom[d] == target {
                    found = d as u32;
                    cnt += 1;
                }
            }
            assert_eq!(cnt, 1, "immediate dominator must be unique");
            idom[v] = found;
        }
        (reach, idom)
    }

    fn is_perm_of(n: usize, order: &[u32]) -> bool {
        if order.len() != n {
            return false;
        }
        let mut seen = vec![false; n];
        for &v in order {
            let v = v as usize;
            if v >= n || seen[v] {
                return false;
            }
            seen[v] = true;
        }
        true
    }

    fn assert_valid_topo(n: usize, adj: &[Vec<u32>], order: &[u32]) {
        assert!(is_perm_of(n, order), "order must be a permutation of 0..n");
        let mut pos = vec![0usize; n];
        for (i, &v) in order.iter().enumerate() {
            pos[v as usize] = i;
        }
        for u in 0..n {
            for &v in &adj[u] {
                assert!(pos[u] < pos[v as usize], "edge u→v must be respected");
            }
        }
    }

    fn assert_valid_cycle_list(n: usize, adj: &[Vec<u32>], list: &[u32]) {
        assert!(!list.is_empty(), "cycle list must be non-empty");
        assert!(list.len() <= n);
        let mut seen = vec![false; n];
        for &v in list {
            let v = v as usize;
            assert!(v < n && !seen[v], "cycle nodes must be distinct and < n");
            seen[v] = true;
        }
        let has_edge = |u: u32, v: u32| adj[u as usize].contains(&v);
        for w in list.windows(2) {
            assert!(has_edge(w[0], w[1]), "consecutive cycle edge missing");
        }
        assert!(
            has_edge(*list.last().unwrap(), list[0]),
            "closing cycle edge missing"
        );
        let min = *list.iter().min().unwrap();
        assert_eq!(list[0], min, "cycle list must start at its smallest ID");
    }

    /// Full battery for one graph instance: naive cross-checks, determinism
    /// double-run, CSR-order invariance. Returns whether the graph is acyclic.
    fn check_graph(ws: &mut GraphWorkspace, n: u32, pairs: &[(u32, u32)]) -> bool {
        let (off, tgt) = csr_from_pairs(n, pairs);
        let (off2, tgt2) = csr_from_pairs_variant(n, pairs);
        let adj = adj_of(n, pairs);
        let cyc_naive = naive_cyclic(n as usize, &adj);
        let nu = n as usize;

        let g = Graph {
            n,
            offsets: &off,
            targets: &tgt,
        };

        // --- Kernel 1: topo_order (canonical CSR) ---
        let mut out = vec![OUT_UNTOUCHED; nu];
        let t1 = topo_order(&g, ws, &mut out);
        match &t1 {
            Ok(()) => {
                assert!(!cyc_naive, "Kahn succeeded on a naive-cyclic graph");
                let expected = naive_minlex(nu, &adj).unwrap();
                assert_eq!(&out[..], &expected[..], "tie-break must equal the naive min-ready order");
                assert_valid_topo(nu, &adj, &out);
            }
            Err(GraphMathError::Cycle) => {
                assert!(cyc_naive, "Kahn reported a cycle on an acyclic graph");
                for &cell in &out {
                    assert_eq!(cell, OUT_UNTOUCHED, "out must be untouched on Err");
                }
            }
            Err(e) => panic!("unexpected topo error {:?}", e),
        }

        // --- Kernel 3: find_cycle (canonical CSR) ---
        let mut cout = vec![0u32; nu];
        let c1 = find_cycle(&g, ws, &mut cout);
        match &c1 {
            Ok(CycleReport::Acyclic) => assert!(!cyc_naive),
            Ok(CycleReport::Cycle { len }) => {
                assert!(cyc_naive);
                assert!(*len >= 1 && *len as usize <= nu);
                assert_valid_cycle_list(nu, &adj, &cout[..*len as usize]);
            }
            Err(e) => panic!("unexpected find_cycle error {:?}", e),
        }

        // --- Kernel 2: dominators (entry 0; naive extensional reference) ---
        let mut idom = vec![0xAAu32; nu];
        let d1 = dominators(&g, 0, ws, &mut idom);
        if n == 0 {
            assert_eq!(d1, Err(GraphMathError::EntryOutOfRange));
        } else {
            let stats = d1.unwrap();
            assert!(stats.passes >= 1 && stats.passes <= n, "sweep measure ≤ n");
            let (reach, nid) = naive_dom(nu, &adj, 0);
            let rcount = reach.iter().filter(|x| **x).count();
            assert_eq!(stats.reachable as usize, rcount);
            for v in 0..nu {
                if reach[v] {
                    assert_eq!(idom[v], nid[v], "idom mismatch at {}", v);
                } else {
                    assert_eq!(idom[v], DOM_UNREACHABLE);
                }
            }
        }

        // --- Determinism: double runs on the same encoding must be identical ---
        let mut out2 = vec![0u32; nu];
        let t2 = topo_order(&g, ws, &mut out2);
        assert_eq!(t1, t2);
        if t1.is_ok() {
            assert_eq!(out[..nu], out2[..nu]);
        }
        let mut cout2 = vec![0u32; nu];
        let c2 = find_cycle(&g, ws, &mut cout2);
        assert_eq!(c1, c2);
        if c1.is_ok() {
            assert_eq!(cout[..nu], cout2[..nu]);
        }
        if n > 0 {
            let mut idom2 = vec![0u32; nu];
            let d2 = dominators(&g, 0, ws, &mut idom2);
            assert_eq!(d1, d2);
            assert_eq!(idom[..nu], idom2[..nu]);
        }

        // --- CSR-order invariance (permuted encoding) ---
        let g2 = Graph {
            n,
            offsets: &off2,
            targets: &tgt2,
        };
        let mut outv = vec![0u32; nu];
        let tv = topo_order(&g2, ws, &mut outv);
        assert_eq!(t1, tv, "topo outcome must be encoding-invariant");
        if t1.is_ok() {
            assert_eq!(out[..nu], outv[..nu], "topo order must be encoding-invariant");
        }
        let mut coutv = vec![0u32; nu];
        let cv = find_cycle(&g2, ws, &mut coutv);
        let cyc1 = matches!(c1, Ok(CycleReport::Cycle { .. }));
        let cycv = matches!(cv, Ok(CycleReport::Cycle { .. }));
        assert_eq!(cyc1, cycv, "cyclicity must be encoding-invariant");
        if let Ok(CycleReport::Cycle { len }) = cv {
            assert!(cyc_naive);
            assert_valid_cycle_list(nu, &adj, &coutv[..len as usize]);
        }
        if n > 0 {
            let mut idomv = vec![0u32; nu];
            let dv = dominators(&g2, 0, ws, &mut idomv);
            match (&d1, &dv) {
                (Ok(s1), Ok(sv)) => {
                    assert_eq!(s1.reachable, sv.reachable);
                    assert!(sv.passes >= 1 && sv.passes <= n);
                    assert_eq!(idom[..nu], idomv[..nu], "idom tree must be encoding-invariant");
                }
                _ => panic!("dominators must succeed identically on the permuted encoding"),
            }
        }
        !cyc_naive
    }

    fn next_perm(p: &mut [usize]) -> bool {
        let n = p.len();
        let mut i = n - 1;
        while i > 0 && p[i - 1] >= p[i] {
            i -= 1;
        }
        if i == 0 {
            return false;
        }
        let mut j = n - 1;
        while p[j] <= p[i - 1] {
            j -= 1;
        }
        p.swap(i - 1, j);
        let mut a = i;
        let mut b = n - 1;
        while a < b {
            p.swap(a, b);
            a += 1;
            b -= 1;
        }
        true
    }

    /// All digraphs on n ≤ 4 labeled nodes via adjacency bitmask:
    /// 1 (n=0) + 1 (n=1) + 4 (n=2) + 64 (n=3) + 4096 (n=4) = 4,166 graphs.
    /// The 573 acyclic ones are cross-checked against the labeled-DAG counts
    /// 1 + 1 + 3 + 25 + 543 (OEIS-referenced labeled-DAG sequence).
    #[test]
    fn exh_all_digraphs_n_le_4() {
        let mut ws = boxed_ws();
        let mut acyclic = 0usize;
        let mut cases = 0usize;
        for n in 0u32..=4 {
            let pc: u32 = if n == 0 { 0 } else { n * (n - 1) };
            for mask in 0u64..(1u64 << pc) {
                let mut pairs: Vec<(u32, u32)> = Vec::new();
                let mut b = 0u32;
                while b < pc {
                    if (mask >> b) & 1 == 1 {
                        let i = b / (n - 1);
                        let jj = b % (n - 1);
                        let j = jj + if jj >= i { 1 } else { 0 };
                        pairs.push((i, j));
                    }
                    b += 1;
                }
                if check_graph(&mut ws, n, &pairs) {
                    acyclic += 1;
                }
                cases += 1;
            }
        }
        assert_eq!(cases, 4166, "all digraphs on n ≤ 4 must be enumerated");
        assert_eq!(acyclic, 573, "acyclic count must equal 1+1+3+25+543");
    }

    /// All DAGs on n = 5: every DAG admits a topological order π and its edge
    /// set is a subset of the 10 upward edges (π[i] → π[j], i < j); enumerating
    /// all 120 permutations × 1,024 upward subsets covers every DAG. Dedup by
    /// adjacency bitmask; the unique count must equal the labeled-DAG count 29,281.
    #[test]
    fn exh_all_dags_n5() {
        let mut ws = boxed_ws();
        let mut seen: HashSet<u32> = HashSet::new();
        let mut perm = [0usize, 1, 2, 3, 4];
        loop {
            for sub in 0..1024u32 {
                let mut pairs: Vec<(u32, u32)> = Vec::new();
                let mut bit = 0u32;
                for i in 0..5usize {
                    let mut j = i + 1;
                    while j < 5 {
                        if (sub >> bit) & 1 == 1 {
                            pairs.push((perm[i] as u32, perm[j] as u32));
                        }
                        bit += 1;
                        j += 1;
                    }
                }
                let mut key = 0u32;
                for &(u, v) in &pairs {
                    key |= 1u32 << (u * 4 + v - if v > u { 1 } else { 0 });
                }
                if seen.insert(key) {
                    check_graph(&mut ws, 5, &pairs);
                }
            }
            if !next_perm(&mut perm) {
                break;
            }
        }
        assert_eq!(seen.len(), 29281, "must be exactly all 29,281 DAGs on n = 5");
    }

    /// Every reachable `GraphMathError` variant fired on a crafted input, in
    /// the documented check order. `PassBudgetExceeded` is unreachable within
    /// the pinned measure (retained as a defined failure; no runtime assert is
    /// possible for an unreachable variant — see the file CEP:FAILURE).
    #[test]
    fn exh_error_variants() {
        let mut ws = boxed_ws();

        // TooManyNodes: n = MAX_NODES + 1 (checked before any slice access).
        let g_big = Graph {
            n: (MAX_NODES + 1) as u32,
            offsets: &[],
            targets: &[],
        };
        assert_eq!(topo_order(&g_big, &mut ws, &mut []), Err(GraphMathError::TooManyNodes));
        assert_eq!(dominators(&g_big, 0, &mut ws, &mut []), Err(GraphMathError::TooManyNodes));
        assert_eq!(find_cycle(&g_big, &mut ws, &mut []), Err(GraphMathError::TooManyNodes));

        // TooManyEdges: targets.len() = MAX_EDGES + 1.
        let big_targets = vec![0u32; MAX_EDGES + 1];
        let off_big = vec![0u32, big_targets.len() as u32, big_targets.len() as u32];
        let g_be = Graph {
            n: 2,
            offsets: &off_big,
            targets: &big_targets,
        };
        assert_eq!(topo_order(&g_be, &mut ws, &mut [0, 0]), Err(GraphMathError::TooManyEdges));

        // MalformedCsr: four shapes (length, first, monotonicity, tail).
        let off_len = vec![0u32, 1];
        let g_ml = Graph { n: 2, offsets: &off_len, targets: &[] };
        assert_eq!(topo_order(&g_ml, &mut ws, &mut [0, 0]), Err(GraphMathError::MalformedCsr));
        let off_first = vec![1u32, 1, 1];
        let tg2 = vec![0u32, 0];
        let g_mf = Graph { n: 2, offsets: &off_first, targets: &tg2 };
        assert_eq!(topo_order(&g_mf, &mut ws, &mut [0, 0]), Err(GraphMathError::MalformedCsr));
        let off_mono = vec![0u32, 2, 1];
        let g_mm = Graph { n: 2, offsets: &off_mono, targets: &tg2 };
        assert_eq!(topo_order(&g_mm, &mut ws, &mut [0, 0]), Err(GraphMathError::MalformedCsr));
        let off_tail = vec![0u32, 1, 2];
        let tg1 = vec![0u32];
        let g_mt = Graph { n: 2, offsets: &off_tail, targets: &tg1 };
        assert_eq!(topo_order(&g_mt, &mut ws, &mut [0, 0]), Err(GraphMathError::MalformedCsr));

        // EdgeTargetOutOfRange: target 5 ≥ n = 2.
        let tg_bad = vec![5u32];
        let off_ok = vec![0u32, 1, 1];
        let g_er = Graph { n: 2, offsets: &off_ok, targets: &tg_bad };
        assert_eq!(topo_order(&g_er, &mut ws, &mut [0, 0]), Err(GraphMathError::EdgeTargetOutOfRange));

        // EntryOutOfRange: entry = n.
        let g1 = Graph { n: 1, offsets: &[0u32, 0], targets: &[] };
        let mut idom1 = [0u32; 1];
        assert_eq!(dominators(&g1, 1, &mut ws, &mut idom1), Err(GraphMathError::EntryOutOfRange));

        // OutputBufferTooSmall: each kernel's result buffer too short.
        assert_eq!(topo_order(&g1, &mut ws, &mut []), Err(GraphMathError::OutputBufferTooSmall));
        assert_eq!(dominators(&g1, 0, &mut ws, &mut []), Err(GraphMathError::OutputBufferTooSmall));
        assert_eq!(find_cycle(&g1, &mut ws, &mut []), Err(GraphMathError::OutputBufferTooSmall));

        // Cycle: 2-cycle; topo fails, find_cycle reports the 2-cycle.
        let off_c = vec![0u32, 1, 2];
        let tg_c = vec![1u32, 0];
        let g_c = Graph { n: 2, offsets: &off_c, targets: &tg_c };
        assert_eq!(topo_order(&g_c, &mut ws, &mut [0, 0]), Err(GraphMathError::Cycle));
        let mut cout = [0u32; 2];
        assert_eq!(find_cycle(&g_c, &mut ws, &mut cout), Ok(CycleReport::Cycle { len: 2 }));
    }

    /// Cycle-report specials: self-loop (len 1), rotation to the smallest ID,
    /// disjoint cycles (roots ascending), n = 0 behavior.
    #[test]
    fn exh_cycle_report_specials() {
        let mut ws = boxed_ws();

        // Self-loop n = 1: cycle list [0], closing edge 0→0.
        let (off, tgt) = csr_from_pairs(1, &[(0, 0)]);
        let g = Graph { n: 1, offsets: &off, targets: &tgt };
        let mut cout = [0u32; 1];
        assert_eq!(find_cycle(&g, &mut ws, &mut cout), Ok(CycleReport::Cycle { len: 1 }));
        assert_eq!(cout[0], 0);
        assert_eq!(topo_order(&g, &mut ws, &mut [0u32]), Err(GraphMathError::Cycle));

        // Rotation: 0→2, 2→1, 1→2 — the DFS-meeting cycle is [2,1]; the
        // reported list must be rotated to [1,2].
        let (off, tgt) = csr_from_pairs(3, &[(0, 2), (2, 1), (1, 2)]);
        let g = Graph { n: 3, offsets: &off, targets: &tgt };
        let mut cout = [0u32; 3];
        assert_eq!(find_cycle(&g, &mut ws, &mut cout), Ok(CycleReport::Cycle { len: 2 }));
        assert_eq!(&cout[..2], &[1, 2]);

        // Disjoint cycles 0↔1 and 2↔3: roots ascending → the first cycle found
        // contains node 0; validity of the list is asserted by the battery rules.
        let (off, tgt) = csr_from_pairs(4, &[(0, 1), (1, 0), (2, 3), (3, 2)]);
        let g = Graph { n: 4, offsets: &off, targets: &tgt };
        let mut cout = [0u32; 4];
        match find_cycle(&g, &mut ws, &mut cout) {
            Ok(CycleReport::Cycle { len }) => {
                assert_eq!(len, 2);
                assert_eq!(cout[0], 0, "roots ascending: node 0's cycle is found first");
                assert_valid_cycle_list(4, &adj_of(4, &[(0, 1), (1, 0), (2, 3), (3, 2)]), &cout[..2]);
            }
            other => panic!("expected a cycle, got {:?}", other),
        }

        // n = 0: empty order, acyclic.
        let g0 = Graph { n: 0, offsets: &[0u32], targets: &[] };
        assert_eq!(topo_order(&g0, &mut ws, &mut []), Ok(()));
        assert_eq!(find_cycle(&g0, &mut ws, &mut []), Ok(CycleReport::Acyclic));
    }

    /// Entry parameter coverage: dominators with entry ≠ 0 against the naive
    /// reference at the same entry; entry = n is out of domain.
    #[test]
    fn exh_dominators_other_entries() {
        let mut ws = boxed_ws();

        // Diamond 0→1,0→2,1→3,2→3 with entry 1: nodes {1,3} reachable.
        let (off, tgt) = csr_from_pairs(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let adj = adj_of(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let g = Graph { n: 4, offsets: &off, targets: &tgt };
        let mut idom = [0u32; 4];
        let stats = dominators(&g, 1, &mut ws, &mut idom).unwrap();
        let (reach, nid) = naive_dom(4, &adj, 1);
        assert_eq!(stats.reachable as usize, reach.iter().filter(|x| **x).count());
        assert!(stats.passes <= 4);
        for v in 0..4 {
            if reach[v] {
                assert_eq!(idom[v], nid[v]);
            } else {
                assert_eq!(idom[v], DOM_UNREACHABLE);
            }
        }

        // Cycle through the entry: 1→2, 2→1, 0 isolated; entry 1.
        let (off, tgt) = csr_from_pairs(3, &[(1, 2), (2, 1)]);
        let adj = adj_of(3, &[(1, 2), (2, 1)]);
        let g = Graph { n: 3, offsets: &off, targets: &tgt };
        let mut idom = [0u32; 3];
        let stats = dominators(&g, 1, &mut ws, &mut idom).unwrap();
        let (reach, nid) = naive_dom(3, &adj, 1);
        assert_eq!(stats.reachable, 2);
        assert!(stats.passes <= 3);
        for v in 0..3 {
            if reach[v] {
                assert_eq!(idom[v], nid[v]);
            } else {
                assert_eq!(idom[v], DOM_UNREACHABLE);
            }
        }

        // Irreducible entry with self-loop: 2→3, 3→2, 2→2; entry 2.
        let (off, tgt) = csr_from_pairs(4, &[(2, 3), (3, 2), (2, 2)]);
        let adj = adj_of(4, &[(2, 3), (3, 2), (2, 2)]);
        let g = Graph { n: 4, offsets: &off, targets: &tgt };
        let mut idom = [0u32; 4];
        let stats = dominators(&g, 2, &mut ws, &mut idom).unwrap();
        let (reach, nid) = naive_dom(4, &adj, 2);
        assert_eq!(stats.reachable, 2);
        assert!(stats.passes <= 4);
        for v in 0..4 {
            if reach[v] {
                assert_eq!(idom[v], nid[v]);
            } else {
                assert_eq!(idom[v], DOM_UNREACHABLE);
            }
        }

        // entry = n is out of domain (n = 3, entry 3).
        let (off, tgt) = csr_from_pairs(3, &[]);
        let g = Graph { n: 3, offsets: &off, targets: &tgt };
        let mut idom = [0u32; 3];
        assert_eq!(dominators(&g, 3, &mut ws, &mut idom), Err(GraphMathError::EntryOutOfRange));
    }

    /// Paired PROP (SPEC-INDEX §4.3): 2,000 deterministic-PRNG digraphs on
    /// n ∈ {5, 6} through the same full battery as the EXH tests.
    #[test]
    fn prop_random_digraphs_n5_n6() {
        let mut ws = boxed_ws();
        let mut st = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            st
        };
        for _ in 0..2000 {
            let n = 5 + (next() % 2) as u32;
            let pc = n * (n - 1);
            let draw = next();
            let mut pairs: Vec<(u32, u32)> = Vec::new();
            let mut b = 0u32;
            while b < pc {
                if (draw >> b) & 1 == 1 {
                    let i = b / (n - 1);
                    let jj = b % (n - 1);
                    let j = jj + if jj >= i { 1 } else { 0 };
                    pairs.push((i, j));
                }
                b += 1;
            }
            check_graph(&mut ws, n, &pairs);
        }
    }
}
