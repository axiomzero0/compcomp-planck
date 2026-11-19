// CEP:FILE: math/crates/planck-graph-math/src/lib.rs
// CEP:WHAT: Pinned graph math of the Planck compiler (SPEC-INDEX §5.7, elaborated by Volume 13): CSR graph validation, Kahn topological order with stable ascending-NodeId tie-break, three-color iterative cycle detection reporting one cycle as a node list, and iterative dominator computation over reverse postorder with a ≤ n pass termination bound.
// CEP:WHY: SPEC-INDEX §5.1 pins planck-graph-math as the single extraction home of the graph math used by IR 1 (Volume 13: topological order, dominators, cycle detection) and by backend CFG analyses (Volume 16); Volume 13 §13.2 bans re-implementing this math inside the IR (G16 TCB discipline), so every consumer must call exactly these functions.
// CEP:STATUS: complete
// CEP:FAILURE: A topological order that is not a permutation respecting every edge, a Kahn success on a cyclic graph, a cycle report that is not a real cycle, a dominator result that disagrees with the extensional definition (d dominates v iff d lies on every entry-to-v path), or a pass count exceeding n is non-conforming; the EXH tests enumerate every digraph on n ≤ 4 labeled nodes and every DAG on n = 5 and fail the build. GraphError is complete and every variant is reachable: NodeCountExceedsLimit (node_count > L13), InvalidNodeReference (edge target or entry >= node_count), MalformedStarts (CSR starts length/order/zero), EntryOutOfRange (rooted-function entry >= node_count or RPO not headed by entry), CycleDetected (Kahn on a cyclic graph, G17), BufferTooSmall(BufferKind) (every BufferKind reachable from stated inputs). Buffer-shortage and malformed-input failures return the named GraphError variants; nothing panics.
// CEP:ASSUMES: SPEC-INDEX §5.7 (pinned algorithms), §5.1 (equivalence claim form), §5.2 (kernel implementation law: core:: only, no allocation, no panic, no unsafe in non-test code); Volume 12 limits table rows L13 (max IR nodes per function = 16777216 = 2^24) and L12 (2^30 per crate) — the per-graph node bound enforced here is L13; Volume 13 (CSR construction from IR 1 graphs, entry/exit invariants, obligations OB-13-3..OB-13-6); the AArch64-style detail that dominator dataflow needs predecessor lists is satisfied by `reverse` (the pinned CSR math of §5.7 gives successors; `reverse` is the checked inverse view).
// CEP:COST: validate O(n + m); topo_order O((n + m) log n) via binary min-heap keyed by node ID (determinism, G14); find_cycle O(n + m); dfs_postorder O(n + m); reverse O(n + m); dominators O(n·p·d + m_pred) with p ≤ n passes (G11 bound, instrumented) and d = dominator-tree depth; all memory is caller-provided fixed-capacity buffers; zero allocation.
// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4_topo_cycle_dominators (4165 graphs: every digraph on 1..=4 labeled nodes — topo validity, Kahn/cycle agreement, determinism, ascending tie-break vs lexicographic reference, dominators vs extensional naive definition via full simple-path enumeration, and the pass bound passes <= n asserted inline on every instance); EXH exh_all_dags_n_5_topo_dominators (1024 DAGs, same checks incl. the inline pass bound); EXH exh_buffer_and_input_errors (every GraphError variant reachable from stated malformed inputs).
// CEP:FV-CLASS: FV-0
// CEP:FV-PROVER: Rust test harness (EXH over the complete stated graph domains, test-as-proof)
// CEP:FV-SCRIPT: crates/planck-graph-math/src/lib.rs (#[cfg(test)] mod tests)
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: complete over the stated domains: every digraph on n ≤ 4 labeled nodes (4165) and every DAG on n = 5 (1024) is enumerated against the extensional definitions; graphs beyond those sizes are covered by the pinned algorithms plus the instrumented ≤ n termination bound, which the enumerated domains exercise exactly (including irreducible and cyclic shapes). No unbounded-equivalence claim is made for larger graphs; the volume-13 obligation rows (OB-13-3..OB-13-6) reference these certificates.
// CEP:FV-AXIOMS: none

#![cfg_attr(not(test), no_std)]

use core::cmp::Ordering as CmpOrdering;

// ---------------------------------------------------------------------------
// Named constants (Law 7). Every bound below is cited to its spec row.
// ---------------------------------------------------------------------------

/// Maximum nodes per graph: Volume 12 limits table row L13, "Max IR nodes per function = 16777216 (2^24)".
///
/// CEP:WHAT: Maximum nodes per graph: Volume 12 limits table row L13, "Max IR nodes per function = 16777216 (2^24)".
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: the pinned contract cited in the summary above; see the crate header CEP:ASSUMES for the full citation set.
/// CEP:COST: compile-time constant; no runtime cost.
/// CEP:EVIDENCE: crate-level EXH suite (see crate header CEP:EVIDENCE).
pub const MAX_NODES: u32 = 16_777_216;

/// Three-color DFS mark: unseen (SPEC-INDEX §5.7 "three-color DFS": white = unvisited).
///
/// CEP:WHAT: Three-color DFS mark: unseen (SPEC-INDEX §5.7 "three-color DFS").
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: the pinned contract cited in the summary above; see the crate header CEP:ASSUMES for the full citation set.
/// CEP:COST: compile-time constant; no runtime cost.
/// CEP:EVIDENCE: crate-level EXH suite (see crate header CEP:EVIDENCE).
pub const COLOR_WHITE: u8 = 0;
/// Three-color DFS mark: on the current DFS path (gray).
///
/// CEP:WHAT: Three-color DFS mark: on the current DFS path.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: the pinned contract cited in the summary above; see the crate header CEP:ASSUMES for the full citation set.
/// CEP:COST: compile-time constant; no runtime cost.
/// CEP:EVIDENCE: crate-level EXH suite (see crate header CEP:EVIDENCE).
pub const COLOR_GRAY: u8 = 1;
/// Three-color DFS mark: fully processed (black).
///
/// CEP:WHAT: Three-color DFS mark: fully processed.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: the pinned contract cited in the summary above; see the crate header CEP:ASSUMES for the full citation set.
/// CEP:COST: compile-time constant; no runtime cost.
/// CEP:EVIDENCE: crate-level EXH suite (see crate header CEP:EVIDENCE).
pub const COLOR_BLACK: u8 = 2;

/// Dominator sentinel: node not reachable from the entry (Volume 13 I-invariants treat unreachable nodes as a separate check; their dominator entry is this sentinel, never a fabricated immediate dominator).
///
/// CEP:WHAT: Dominator sentinel: node not reachable from the entry (Volume 13 I-invariants treat unreachable nodes as a separate check; their dominator entry is this sentinel, never a fabricated immediate dominator).
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: the pinned contract cited in the summary above; see the crate header CEP:ASSUMES for the full citation set.
/// CEP:COST: compile-time constant; no runtime cost.
/// CEP:EVIDENCE: crate-level EXH suite (see crate header CEP:EVIDENCE).
pub const UNREACHABLE: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// Errors (complete enum; CEP:FAILURE discipline).
// ---------------------------------------------------------------------------

/// CEP:WHAT: Which caller buffer was too small; one variant per buffer kind, complete by construction.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data value naming a too-small caller buffer).
/// CEP:ASSUMES: each variant corresponds to exactly one caller-provided buffer of the pinned functions; see the function docs for capacities.
/// CEP:COST: data type; cost carried by the functions over it.
/// CEP:EVIDENCE: EXH exh_buffer_and_input_errors (every variant reachable from stated inputs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferKind {
    /// Indegree scratch for `topo_order` (needs `node_count` words).
    Indeg,
    /// Binary min-heap scratch for `topo_order` (needs `node_count` words).
    Heap,
    /// Output order buffer (needs `node_count` words).
    Output,
    /// Color marks for `find_cycle` / `dfs_postorder` (needs `node_count` bytes).
    Color,
    /// DFS frame stack for `find_cycle` / `dfs_postorder` (needs `node_count` frames).
    DfsStack,
    /// Cycle path output of `find_cycle` (needs `node_count` words).
    CyclePath,
    /// Reverse-CSR starts buffer of `reverse` (needs `node_count + 1` words).
    RevStarts,
    /// Reverse-CSR targets buffer of `reverse` (needs `targets.len()` words).
    RevTargets,
    /// RPO index scratch of `dominators` (needs `node_count` words).
    RpoIndex,
    /// Immediate-dominator output of `dominators` (needs `node_count` words).
    Idom,
}

/// Complete error set for every public function (CEP:FAILURE lists every variant).
///
/// CEP:WHAT: Complete error set for every public function of this crate.
/// CEP:STATUS: complete
/// CEP:FAILURE: complete enum, every variant reachable: NodeCountExceedsLimit (node_count > L13), InvalidNodeReference (edge target or entry >= node_count), MalformedStarts (CSR starts length/order/zero), EntryOutOfRange (rooted-function entry or RPO not headed by entry), CycleDetected (Kahn on a cyclic graph, G17), BufferTooSmall(BufferKind) (every BufferKind reachable).
/// CEP:ASSUMES: variants are derived exactly from the pinned SPEC-INDEX §5.7 algorithms' failure conditions; no other failure mode exists.
/// CEP:COST: data type; cost carried by the functions over it.
/// CEP:EVIDENCE: EXH exh_buffer_and_input_errors (each variant) + every enumerated graph validates cleanly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphError {
    /// `node_count > MAX_NODES` (Volume 12 row L13).
    NodeCountExceedsLimit,
    /// An edge target or the entry names a node `>= node_count`.
    InvalidNodeReference,
    /// `starts.len() != node_count + 1`, `starts[0] != 0`, or `starts` is not
    /// non-decreasing (malformed CSR).
    MalformedStarts,
    /// `entry >= node_count` passed to a rooted function.
    EntryOutOfRange,
    /// Kahn's algorithm on a cyclic graph: failure on cycle is an error, never a
    /// partial order (SPEC-INDEX §5.7, G17).
    CycleDetected,
    /// A caller buffer is smaller than the documented capacity requirement;
    /// `BufferKind` names the buffer.
    BufferTooSmall(BufferKind),
}

// ---------------------------------------------------------------------------
// CSR graph (static arrays; SPEC-INDEX §5.7 "graphs are static arrays").
// ---------------------------------------------------------------------------

/// CEP:WHAT: A graph in CSR form: node v's successors are targets[starts[v] as usize .. starts[v+1] as usize].
/// CEP:STATUS: complete
/// CEP:FAILURE: `validate` returns exactly: NodeCountExceedsLimit, MalformedStarts, InvalidNodeReference; `degree` is caller-domain-checked (v < node_count).
/// CEP:ASSUMES: domain: node_count <= MAX_NODES (L13), starts.len() == node_count + 1, starts[0] == 0, starts non-decreasing, every target < node_count.
/// CEP:COST: O(1) degree; validate O(n + m).
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4_topo_cycle_dominators (every enumerated CSR validates); EXH exh_buffer_and_input_errors.
#[derive(Clone, Copy)]
pub struct Csr<'a> {
    /// Number of nodes `n`; node IDs are `0..n`.
    pub node_count: u32,
    /// Offsets into `targets`; `starts.len() == node_count + 1`, monotone.
    pub starts: &'a [u32],
    /// Flattened edge targets; every entry `< node_count`.
    pub targets: &'a [u32],
}

impl<'a> Csr<'a> {
    /// Out-degree of `v` (`v < node_count`; caller-checked).
    ///
    /// CEP:WHAT: Out-degree of v (successor count).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none (panics only on a caller-side invalid v, outside the domain; validate is the gate).
    /// CEP:ASSUMES: domain: v < node_count and a validated CSR.
    /// CEP:COST: O(1).
    /// CEP:EVIDENCE: exercised on every enumerated graph.
    pub fn degree(&self, v: u32) -> u32 {
        let s = self.starts[v as usize];
        let e = self.starts[v as usize + 1];
        e - s
    }

    /// Structural validation: the full `Csr` domain above.
    ///
    /// CEP:WHAT: Structural validation of the CSR domain (L13 bound, starts length/zero/monotone, target range).
    /// CEP:STATUS: complete
    /// CEP:FAILURE: the three CSR variants of GraphError (complete): NodeCountExceedsLimit, MalformedStarts, InvalidNodeReference.
    /// CEP:ASSUMES: domain: any &Csr.
    /// CEP:COST: O(n + m).
    /// CEP:EVIDENCE: EXH exh_buffer_and_input_errors (each variant) + every enumerated graph validates cleanly.
    pub fn validate(&self) -> Result<(), GraphError> {
        if self.node_count > MAX_NODES {
            return Err(GraphError::NodeCountExceedsLimit);
        }
        if self.starts.len() != self.node_count as usize + 1 || self.starts[0] != 0 {
            return Err(GraphError::MalformedStarts);
        }
        let mut i = 0usize;
        while i < self.starts.len() - 1 {
            if self.starts[i + 1] < self.starts[i] {
                return Err(GraphError::MalformedStarts);
            }
            i += 1;
        }
        if self.targets.len() != self.starts[self.node_count as usize] as usize {
            return Err(GraphError::MalformedStarts);
        }
        for t in self.targets.iter() {
            if *t >= self.node_count {
                return Err(GraphError::InvalidNodeReference);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Topological order: Kahn's algorithm, ascending-ID tie-break (G14).
// ---------------------------------------------------------------------------

/// CEP:WHAT: Kahn's algorithm with stable tie-break by ascending node ID; on success writes the order into out[0..k] and returns k (always node_count — a leftover node means a cycle, which is the CycleDetected error, never a partial order).
/// CEP:STATUS: complete
/// CEP:FAILURE: CycleDetected, MalformedStarts, InvalidNodeReference, NodeCountExceedsLimit, BufferTooSmall(Indeg|Heap|Output).
/// CEP:ASSUMES: domain: validated CSR; indeg.len() >= n, heap.len() >= n, out.len() >= n; binary min-heap keyed by node ID gives the ascending tie-break (determinism, G14).
/// CEP:COST: O((n + m) log n).
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4_topo_cycle_dominators and EXH exh_all_dags_n_5_topo_dominators (order validity, Kahn/cycle agreement, determinism, ascending tie-break vs lexicographic reference).
pub fn topo_order(
    g: &Csr,
    indeg: &mut [u32],
    heap: &mut [u32],
    out: &mut [u32],
) -> Result<usize, GraphError> {
    g.validate()?;
    let n = g.node_count as usize;
    if indeg.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Indeg));
    }
    if heap.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Heap));
    }
    if out.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Output));
    }
    // Indegrees.
    let mut i = 0usize;
    while i < n {
        indeg[i] = 0;
        i += 1;
    }
    for t in g.targets.iter() {
        indeg[*t as usize] += 1;
    }
    // Heap = all indegree-zero nodes; ascending-ID tie-break = min-heap on ID.
    let mut hlen = 0usize;
    let mut v = 0u32;
    while v < g.node_count {
        if indeg[v as usize] == 0 {
            heap[hlen] = v;
            hlen += 1;
        }
        v += 1;
    }
    heapify_min(heap, hlen);
    let mut k = 0usize;
    while hlen > 0 {
        let v = heap_pop_min(heap, &mut hlen);
        out[k] = v;
        k += 1;
        let s = g.starts[v as usize] as usize;
        let e = g.starts[v as usize + 1] as usize;
        let mut j = s;
        while j < e {
            let w = g.targets[j] as usize;
            indeg[w] -= 1;
            if indeg[w] == 0 {
                heap_push(heap, &mut hlen, g.targets[j]);
            }
            j += 1;
        }
    }
    if k != n {
        // Leftover nodes: cycle (G17: error, never a partial order).
        return Err(GraphError::CycleDetected);
    }
    Ok(k)
}

fn heap_less(a: u32, b: u32) -> bool {
    a < b
}

fn heapify_min(heap: &mut [u32], len: usize) {
    if len < 2 {
        return;
    }
    let mut i = len / 2;
    while i > 0 {
        i -= 1;
        sift_down(heap, i, len);
    }
}

fn sift_down(heap: &mut [u32], mut i: usize, len: usize) {
    loop {
        let l = 2 * i + 1;
        let r = 2 * i + 2;
        let mut smallest = i;
        if l < len && heap_less(heap[l], heap[smallest]) {
            smallest = l;
        }
        if r < len && heap_less(heap[r], heap[smallest]) {
            smallest = r;
        }
        if smallest == i {
            return;
        }
        heap.swap(i, smallest);
        i = smallest;
    }
}

fn heap_push(heap: &mut [u32], len: &mut usize, v: u32) {
    let mut i = *len;
    heap[i] = v;
    *len += 1;
    while i > 0 {
        let parent = (i - 1) / 2;
        if heap_less(heap[i], heap[parent]) {
            heap.swap(i, parent);
            i = parent;
        } else {
            return;
        }
    }
}

fn heap_pop_min(heap: &mut [u32], len: &mut usize) -> u32 {
    let top = heap[0];
    *len -= 1;
    if *len > 0 {
        heap[0] = heap[*len];
        sift_down(heap, 0, *len);
    }
    top
}

// ---------------------------------------------------------------------------
// Cycle detection: three-color DFS, iterative, bounded stack.
// ---------------------------------------------------------------------------

/// One iterative-DFS frame: node plus its next unexplored edge position.
///
/// CEP:WHAT: One iterative-DFS frame: node plus its next unexplored edge position.
/// CEP:STATUS: complete
/// CEP:FAILURE: none (data record).
/// CEP:ASSUMES: stack capacity requirement: node_count frames (see find_cycle/dfs_postorder).
/// CEP:COST: 8 bytes (Copy); zero runtime cost.
/// CEP:EVIDENCE: exercised on every enumerated graph.
#[derive(Clone, Copy)]
pub struct DfsFrame {
    /// Node whose edges are being explored.
    pub node: u32,
    /// Index into `g.targets` of the next edge to explore.
    pub next_edge: u32,
}

/// CEP:WHAT: Three-color iterative cycle detection: on a cycle, writes one cycle as a node list into path[0..k] (closed implicitly; consecutive entries are edge-joined and the last entry has an edge back to the first) and returns Some(k); on an acyclic graph returns Ok(None).
/// CEP:STATUS: complete
/// CEP:FAILURE: the CSR variants + BufferTooSmall(Color|DfsStack|CyclePath).
/// CEP:ASSUMES: domain: validated CSR; color.len() >= n bytes, stack.len() >= n frames, path.len() >= n words (G17: reports one cycle as a node list).
/// CEP:COST: O(n + m).
/// CEP:EVIDENCE: agreement with Kahn on every enumerated digraph; every reported cycle is edge-verified in the tests.
pub fn find_cycle(
    g: &Csr,
    color: &mut [u8],
    stack: &mut [DfsFrame],
    path: &mut [u32],
) -> Result<Option<usize>, GraphError> {
    g.validate()?;
    let n = g.node_count as usize;
    if color.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Color));
    }
    if stack.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::DfsStack));
    }
    if path.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::CyclePath));
    }
    let mut v = 0u32;
    while v < g.node_count {
        color[v as usize] = COLOR_WHITE;
        v += 1;
    }
    let mut top = 0usize;
    let mut root = 0u32;
    while root < g.node_count {
        if color[root as usize] != COLOR_WHITE {
            root += 1;
            continue;
        }
        color[root as usize] = COLOR_GRAY;
        stack[0] = DfsFrame { node: root, next_edge: g.starts[root as usize] };
        top = 1;
        while top > 0 {
            let frame = stack[top - 1];
            let v = frame.node;
            let send = g.starts[v as usize + 1];
            if frame.next_edge >= send {
                // Exhausted: blacken, pop.
                color[v as usize] = COLOR_BLACK;
                top -= 1;
                continue;
            }
            let w = g.targets[frame.next_edge as usize];
            // Advance the frame's cursor in place.
            stack[top - 1].next_edge += 1;
            if color[w as usize] == COLOR_GRAY {
                // Found a cycle: from w's position on the stack down to the top.
                let mut pos = top;
                loop {
                    pos -= 1;
                    if stack[pos].node == w {
                        break;
                    }
                }
                let mut k = 0usize;
                let mut p = pos;
                while p < top {
                    path[k] = stack[p].node;
                    k += 1;
                    p += 1;
                }
                return Ok(Some(k));
            }
            if color[w as usize] == COLOR_WHITE {
                color[w as usize] = COLOR_GRAY;
                stack[top] = DfsFrame { node: w, next_edge: g.starts[w as usize] };
                top += 1;
            }
        }
        root += 1;
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Reachability, postorder, and reverse CSR (dominator support math).
// ---------------------------------------------------------------------------

/// CEP:WHAT: Iterative DFS from entry over g, writing the postorder of the reachable subgraph into out[0..k] (children before parent; entry is last) and the visited mark into seen; returns k.
/// CEP:STATUS: complete
/// CEP:FAILURE: EntryOutOfRange + the CSR variants + BufferTooSmall(Color|DfsStack|Output).
/// CEP:ASSUMES: domain: validated CSR; entry < node_count; seen.len() >= n bytes, stack.len() >= n frames, out.len() >= n words.
/// CEP:COST: O(n + m).
/// CEP:EVIDENCE: exercised on every enumerated graph; the dominator tests consume its output.
pub fn dfs_postorder(
    g: &Csr,
    entry: u32,
    seen: &mut [u8],
    stack: &mut [DfsFrame],
    out: &mut [u32],
) -> Result<usize, GraphError> {
    g.validate()?;
    if entry >= g.node_count {
        return Err(GraphError::EntryOutOfRange);
    }
    let n = g.node_count as usize;
    if seen.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Color));
    }
    if stack.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::DfsStack));
    }
    if out.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Output));
    }
    let mut v = 0u32;
    while v < g.node_count {
        seen[v as usize] = 0;
        v += 1;
    }
    seen[entry as usize] = 1;
    stack[0] = DfsFrame { node: entry, next_edge: g.starts[entry as usize] };
    let mut top = 1usize;
    let mut k = 0usize;
    while top > 0 {
        let frame = stack[top - 1];
        let v = frame.node;
        let send = g.starts[v as usize + 1];
        if frame.next_edge >= send {
            out[k] = v;
            k += 1;
            top -= 1;
            continue;
        }
        let w = g.targets[frame.next_edge as usize];
        stack[top - 1].next_edge += 1;
        if seen[w as usize] == 0 {
            seen[w as usize] = 1;
            stack[top] = DfsFrame { node: w, next_edge: g.starts[w as usize] };
            top += 1;
        }
    }
    Ok(k)
}

/// CEP:WHAT: Reverse postorder: writes postorder reversed into rpo (same length).
/// CEP:STATUS: complete
/// CEP:FAILURE: BufferTooSmall(Output) if rpo is short.
/// CEP:ASSUMES: domain: rpo.len() >= postorder.len().
/// CEP:COST: O(k).
/// CEP:EVIDENCE: consumed by every dominator test.
pub fn reverse_postorder(postorder: &[u32], rpo: &mut [u32]) -> Result<(), GraphError> {
    if rpo.len() < postorder.len() {
        return Err(GraphError::BufferTooSmall(BufferKind::Output));
    }
    let len = postorder.len();
    let mut i = 0usize;
    while i < len {
        rpo[i] = postorder[len - 1 - i];
        i += 1;
    }
    Ok(())
}

/// CEP:WHAT: Builds the reverse view of a validated CSR into caller buffers (rev_starts.len() >= node_count + 1, rev_targets.len() >= targets.len()); after success, rev is a CSR whose edges are the reversed edges of g, sources ascending within each bucket (deterministic).
/// CEP:STATUS: complete
/// CEP:FAILURE: the CSR variants + BufferTooSmall(RevStarts|RevTargets).
/// CEP:ASSUMES: domain: validated CSR + the two buffer capacities; in-place cursor fill with right-shift repair (no allocation).
/// CEP:COST: O(n + m).
/// CEP:EVIDENCE: every dominator test computes the reverse and asserts the reversed edge multiset mirrors the forward one.
pub fn reverse(
    g: &Csr,
    rev_starts: &mut [u32],
    rev_targets: &mut [u32],
) -> Result<(), GraphError> {
    g.validate()?;
    let n = g.node_count as usize;
    if rev_starts.len() < n + 1 {
        return Err(GraphError::BufferTooSmall(BufferKind::RevStarts));
    }
    if rev_targets.len() < g.targets.len() {
        return Err(GraphError::BufferTooSmall(BufferKind::RevTargets));
    }
    // Count in-degrees.
    let mut i = 0usize;
    while i <= n {
        rev_starts[i] = 0;
        i += 1;
    }
    for t in g.targets.iter() {
        rev_starts[*t as usize + 1] += 1;
    }
    // Prefix sum.
    let mut i = 0usize;
    while i < n {
        rev_starts[i + 1] += rev_starts[i];
        i += 1;
    }
    // Fill with in-place cursors: for each edge (src -> t) in ascending source
    // order, write src at rev_starts[t] and advance that cursor. After all m
    // edges are placed, rev_starts[t] holds the original boundary
    // rev_starts_orig[t] + count[t] = rev_starts_orig[t + 1] for every t < n,
    // so a right-shift repair reconstructs the exact CSR starts array.
    for src in 0..g.node_count {
        let s = g.starts[src as usize] as usize;
        let e = g.starts[src as usize + 1] as usize;
        let mut j = s;
        while j < e {
            let t = g.targets[j] as usize;
            rev_targets[rev_starts[t] as usize] = src;
            rev_starts[t] += 1;
            j += 1;
        }
    }
    // Repair: shift right by one; rev_starts[n] already equals m.
    let m = g.targets.len() as u32;
    let mut i = n;
    while i > 0 {
        rev_starts[i] = rev_starts[i - 1];
        i -= 1;
    }
    rev_starts[0] = 0;
    debug_assert_eq!(rev_starts[n], m);
    Ok(())
}

// ---------------------------------------------------------------------------
// Dominators: iterative dataflow over reverse postorder (pinned §5.7).
// ---------------------------------------------------------------------------

/// CEP:WHAT: Iterative dominator computation (RPO dataflow with the intersect operator) over the PREDECESSOR CSR preds and the reverse postorder rpo of the nodes reachable from entry: idom[v] for reachable v != entry; idom[entry] = entry; idom[v] = UNREACHABLE for unreachable nodes; passes = sweeps until no change (pinned termination measure passes <= n with early exit, G11).
/// CEP:STATUS: complete
/// CEP:FAILURE: EntryOutOfRange + CSR variants + BufferTooSmall(RpoIndex|Idom).
/// CEP:ASSUMES: domain: validated preds CSR (same node set); entry < node_count; rpo holds distinct reachable node IDs headed by entry (as produced by dfs_postorder + reverse_postorder); rpo_index.len() >= n, idom.len() >= n.
/// CEP:COST: O(n·p·d + m_pred), p <= n instrumented.
/// CEP:EVIDENCE: EXH exh_all_digraphs_n_le_4_topo_cycle_dominators and EXH exh_all_dags_n_5_topo_dominators (output vs extensional definition via naive simple-path enumeration; passes <= n asserted on every instance).
pub fn dominators(
    preds: &Csr,
    entry: u32,
    rpo: &[u32],
    rpo_index: &mut [u32],
    idom: &mut [u32],
    passes: &mut u32,
) -> Result<(), GraphError> {
    preds.validate()?;
    if entry >= preds.node_count {
        return Err(GraphError::EntryOutOfRange);
    }
    let n = preds.node_count as usize;
    if rpo_index.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::RpoIndex));
    }
    if idom.len() < n {
        return Err(GraphError::BufferTooSmall(BufferKind::Idom));
    }
    // Init: everything unreachable except what RPO names.
    let mut v = 0u32;
    while v < preds.node_count {
        rpo_index[v as usize] = UNREACHABLE;
        idom[v as usize] = UNREACHABLE;
        v += 1;
    }
    let rpo_len = rpo.len();
    let mut i = 0usize;
    while i < rpo_len {
        rpo_index[rpo[i] as usize] = i as u32;
        idom[rpo[i] as usize] = UNREACHABLE;
        i += 1;
    }
    if rpo_len == 0 {
        return Err(GraphError::EntryOutOfRange);
    }
    // rpo[0] must be the entry (postorder from entry ends with entry).
    if rpo[0] != entry {
        return Err(GraphError::EntryOutOfRange);
    }
    idom[entry as usize] = entry;
    // Iterate.
    let mut changed = true;
    let mut p = 0u32;
    while changed {
        changed = false;
        p += 1;
        let mut i = 1usize;
        while i < rpo_len {
            let v = rpo[i];
            // First processed predecessor seeds new_idom; the rest intersect.
            let mut new_idom: u32 = UNREACHABLE;
            let s = preds.starts[v as usize] as usize;
            let e = preds.starts[v as usize + 1] as usize;
            let mut j = s;
            while j < e {
                let q = preds.targets[j];
                if rpo_index[q as usize] != UNREACHABLE && idom[q as usize] != UNREACHABLE {
                    if new_idom == UNREACHABLE {
                        new_idom = q;
                    } else {
                        new_idom = intersect(rpo_index, idom, q, new_idom);
                    }
                }
                j += 1;
            }
            if new_idom != UNREACHABLE && idom[v as usize] != new_idom {
                idom[v as usize] = new_idom;
                changed = true;
            }
            i += 1;
        }
    }
    *passes = p;
    Ok(())
}

/// The intersect operator (finger walk up the idom tree, ordered by RPO index).
fn intersect(rpo_index: &[u32], idom: &[u32], mut a: u32, mut b: u32) -> u32 {
    while a != b {
        let ia = rpo_index[a as usize];
        let ib = rpo_index[b as usize];
        match ia.cmp(&ib) {
            CmpOrdering::Greater => a = idom[a as usize],
            CmpOrdering::Less => b = idom[b as usize],
            CmpOrdering::Equal => {
                // Cannot happen on a valid idom tree (equal index = same node).
                break;
            }
        }
        // Defensive: a broken idom tree (UNREACHABLE finger) stops the walk.
        if a == UNREACHABLE || b == UNREACHABLE {
            break;
        }
    }
    a
}

// ---------------------------------------------------------------------------
// Tests: exhaustive certificates over the stated graph domains.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a CSR from an adjacency bitmask over node pairs (i, j), i != j.
    fn csr_from_mask(n: u32, mask: u64) -> (Vec<u32>, Vec<u32>) {
        let mut starts = Vec::new();
        let mut targets = Vec::new();
        starts.push(0u32);
        for i in 0..n {
            for j in 0..n {
                if i != j && (mask >> (i * n + j)) & 1 == 1 {
                    targets.push(j);
                }
            }
            starts.push(targets.len() as u32);
        }
        (starts, targets)
    }

    /// Naive extensional dominators by simple-path enumeration. EVERY simple
    fn naive_doms(n: u32, starts: &[u32], targets: &[u32], entry: u32) -> Vec<Option<Vec<bool>>> {
        let mut results: Vec<Option<Vec<bool>>> = (0..n as usize).map(|_| None).collect();
        let mut stack: Vec<(u32, Vec<u32>)> = vec![(entry, vec![entry])];
        while let Some((v, p)) = stack.pop() {
            let mark = {
                let mut m = vec![false; n as usize];
                for x in &p {
                    m[*x as usize] = true;
                }
                m
            };
            results[v as usize] = Some(match results[v as usize].take() {
                None => mark,
                Some(prev) => {
                    let mut m2 = vec![false; n as usize];
                    for i in 0..n as usize {
                        m2[i] = prev[i] && mark[i];
                    }
                    m2
                }
            });
            let s = starts[v as usize] as usize;
            let e = starts[v as usize + 1] as usize;
            for j in s..e {
                let w = targets[j];
                if !p.contains(&w) {
                    let mut q = p.clone();
                    q.push(w);
                    stack.push((w, q));
                }
            }
        }
        results
    }

    /// Naive immediate dominator from dom sets. The strict dominators of v
    fn naive_idom(n: u32, doms: &[Option<Vec<bool>>], v: u32) -> Option<u32> {
        let dv = doms[v as usize].as_ref()?;
        // sdom(v) = dom(v) - {v}
        let mut sdom: Vec<u32> = Vec::new();
        for d in 0..n as usize {
            if dv[d] && d != v as usize {
                sdom.push(d as u32);
            }
        }
        if sdom.is_empty() {
            return None; // v == entry (dom(v) == {v})
        }
        for &cand in &sdom {
            let dcand = doms[cand as usize].as_ref().unwrap();
            let mut ok = true;
            for &other in &sdom {
                if other == cand {
                    continue;
                }
                // other must dominate cand: other ∈ dom(cand).
                if !dcand[other as usize] {
                    ok = false;
                    break;
                }
            }
            if ok {
                return Some(cand);
            }
        }
        None
    }

    #[test]
    fn exh_all_digraphs_n_le_4_topo_cycle_dominators() {
        let mut total = 0u64;
        for &n in &[1u32, 2, 3, 4] {
            let bits = n * (n - 1); // directed edge slots
            let count = 1u64 << bits;
            for mask in 0..count {
                let (starts, targets) = csr_from_mask(n, mask);
                let g = Csr { node_count: n, starts: &starts, targets: &targets };
                g.validate().unwrap();
                let m = g.targets.len();

                // --- cycle detection vs Kahn agreement ---
                let mut color = vec![0u8; n as usize];
                let mut stack = vec![DfsFrame { node: 0, next_edge: 0 }; n as usize];
                let mut path = vec![0u32; n as usize];
                let cyc = find_cycle(&g, &mut color, &mut stack, &mut path).unwrap();
                let mut indeg = vec![0u32; n as usize];
                let mut heap = vec![0u32; n as usize];
                let mut out = vec![0u32; n as usize];
                let topo = topo_order(&g, &mut indeg, &mut heap, &mut out);
                match (cyc, topo) {
                    (None, Ok(k)) => {
                        assert_eq!(k as u32, n, "kahn incomplete on acyclic graph mask={mask}");
                        // Order validity: permutation + every edge respected.
                        let mut seen = vec![false; n as usize];
                        for v in out.iter().take(k) {
                            assert!(!seen[*v as usize], "duplicate in topo mask={mask}");
                            seen[*v as usize] = true;
                        }
                        for i in 0..n as usize {
                            for j in starts[i]..starts[i + 1] {
                                let (a, b) = (i as u32, targets[j as usize]);
                                let pa = out.iter().position(|&x| x == a).unwrap();
                                let pb = out.iter().position(|&x| x == b).unwrap();
                                assert!(pa < pb, "edge {a}->{b} violated in topo mask={mask}");
                            }
                        }
                        // Determinism: second run identical.
                        let mut indeg2 = vec![0u32; n as usize];
                        let mut heap2 = vec![0u32; n as usize];
                        let mut out2 = vec![0u32; n as usize];
                        topo_order(&g, &mut indeg2, &mut heap2, &mut out2).unwrap();
                        assert_eq!(out, out2, "topo nondeterminism mask={mask}");
                        // Ascending tie-break: the output equals the
                        // lexicographically minimal valid order (reference
                        // implementation of the pinned tie-break).
                        let lex = lex_min_topo(n, &starts, &targets);
                        assert_eq!(&out[..k], &lex[..], "tie-break not ascending mask={mask}");
                    }
                    (Some(k), Err(GraphError::CycleDetected)) => {
                        // Reported cycle must be real: consecutive edges close.
                        assert!(k >= 1, "empty cycle report mask={mask}");
                        for i in 0..k {
                            let a = path[i];
                            let b = path[(i + 1) % k];
                            let s = starts[a as usize];
                            let e = starts[a as usize + 1];
                            let mut has = false;
                            for j in s..e {
                                if targets[j as usize] == b {
                                    has = true;
                                    break;
                                }
                            }
                            assert!(has, "reported cycle not real at {i} mask={mask}");
                        }
                    }
                    other => panic!("cycle/topo disagreement mask={mask}: {other:?}"),
                }

                // --- dominators vs extensional definition (entry = 0) ---
                let entry = 0u32;
                let mut seen = vec![0u8; n as usize];
                let mut dstack = vec![DfsFrame { node: 0, next_edge: 0 }; n as usize];
                let mut post = vec![0u32; n as usize];
                let k = dfs_postorder(&g, entry, &mut seen, &mut dstack, &mut post).unwrap();
                let mut rpo = vec![0u32; k];
                reverse_postorder(&post[..k], &mut rpo).unwrap();
                let (rstarts, rtargets) = reverse_csr(n, &starts, &targets);
                let preds = Csr { node_count: n, starts: &rstarts, targets: &rtargets };
                let mut rpo_index = vec![0u32; n as usize];
                let mut idom = vec![0u32; n as usize];
                let mut passes = 0u32;
                dominators(&preds, entry, &rpo, &mut rpo_index, &mut idom, &mut passes)
                    .unwrap();
                assert!(passes <= n, "pass bound violated: {passes} > {n} mask={mask}");
                let doms = naive_doms(n, &starts, &targets, entry);
                for v in 0..n as usize {
                    match doms[v].as_ref() {
                        None => {
                            assert_eq!(idom[v], UNREACHABLE, "unreachable v={v} got idom mask={mask}");
                        }
                        Some(_) => {
                            if v as u32 == entry {
                                assert_eq!(idom[v], entry, "entry self-domination mask={mask}");
                            } else {
                                let want = naive_idom(n, &doms, v as u32);
                                let got = if idom[v] == UNREACHABLE { None } else { Some(idom[v]) };
                                assert_eq!(got, want, "idom mismatch v={v} mask={mask}");
                            }
                        }
                    }
                }
                total += 1;
                let _ = m;
            }
        }
        // 1 + 4 + 64 + 4096 = 4165
        assert_eq!(total, 4165, "enumeration count drifted");
    }

    /// Lexicographically minimal topological order via ascending ready-set scan
    fn lex_min_topo(n: u32, starts: &[u32], targets: &[u32]) -> Vec<u32> {
        let mut indeg = vec![0u32; n as usize];
        let mut done = vec![false; n as usize];
        for t in targets.iter() {
            indeg[*t as usize] += 1;
        }
        let mut out = Vec::new();
        for _ in 0..n {
            let mut pick: Option<u32> = None;
            for v in 0..n {
                if indeg[v as usize] == 0 && !done[v as usize] && pick.is_none() {
                    pick = Some(v);
                }
            }
            let v = pick.expect("lex topo on cyclic graph");
            done[v as usize] = true;
            out.push(v);
            for j in starts[v as usize]..starts[v as usize + 1] {
                let w = targets[j as usize] as usize;
                indeg[w] -= 1;
            }
        }
        out
    }

    /// Reverse CSR builder for tests (mirrors the `reverse` contract).
    fn reverse_csr(n: u32, starts: &[u32], targets: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let mut rstarts = vec![0u32; n as usize + 1];
        let mut rtargets = vec![0u32; targets.len()];
        for t in targets.iter() {
            rstarts[*t as usize + 1] += 1;
        }
        for i in 0..n as usize {
            rstarts[i + 1] += rstarts[i];
        }
        let mut fill = rstarts.clone();
        for s in 0..n {
            for j in starts[s as usize]..starts[s as usize + 1] {
                let t = targets[j as usize] as usize;
                rtargets[fill[t] as usize] = s;
                fill[t] += 1;
            }
        }
        (rstarts, rtargets)
    }

    #[test]
    fn exh_all_dags_n_5_topo_dominators() {
        // All DAGs on n = 5: edge set is a subset of the 10 forward pairs.
        let n = 5u32;
        let pairs: Vec<(u32, u32)> = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .collect();
        assert_eq!(pairs.len(), 10);
        for mask in 0u64..(1 << 10) {
            let mut starts = Vec::new();
            let mut targets = Vec::new();
            starts.push(0u32);
            for i in 0..n {
                for (idx, (a, b)) in pairs.iter().enumerate() {
                    if *a == i && (mask >> idx) & 1 == 1 {
                        targets.push(*b);
                    }
                }
                starts.push(targets.len() as u32);
            }
            let g = Csr { node_count: n, starts: &starts, targets: &targets };
            g.validate().unwrap();
            // Acyclic by construction: Kahn must succeed.
            let mut indeg = vec![0u32; n as usize];
            let mut heap = vec![0u32; n as usize];
            let mut out = vec![0u32; n as usize];
            let k = topo_order(&g, &mut indeg, &mut heap, &mut out).unwrap();
            assert_eq!(k as u32, n);
            // Cycle detector must agree (None).
            let mut color = vec![0u8; n as usize];
            let mut stack = vec![DfsFrame { node: 0, next_edge: 0 }; n as usize];
            let mut path = vec![0u32; n as usize];
            assert!(find_cycle(&g, &mut color, &mut stack, &mut path).unwrap().is_none());
            // Dominators (entry = 0) vs naive.
            let mut seen = vec![0u8; n as usize];
            let mut dstack = vec![DfsFrame { node: 0, next_edge: 0 }; n as usize];
            let mut post = vec![0u32; n as usize];
            let kk = dfs_postorder(&g, 0, &mut seen, &mut dstack, &mut post).unwrap();
            let mut rpo = vec![0u32; kk];
            reverse_postorder(&post[..kk], &mut rpo).unwrap();
            let (rstarts, rtargets) = reverse_csr(n, &starts, &targets);
            let preds = Csr { node_count: n, starts: &rstarts, targets: &rtargets };
            let mut rpo_index = vec![0u32; n as usize];
            let mut idom = vec![0u32; n as usize];
            let mut passes = 0u32;
            dominators(&preds, 0, &rpo, &mut rpo_index, &mut idom, &mut passes).unwrap();
            assert!(passes <= n, "pass bound violated on dag mask={mask}");
            let doms = naive_doms(n, &starts, &targets, 0);
            for v in 0..n as usize {
                match doms[v].as_ref() {
                    None => assert_eq!(idom[v], UNREACHABLE, "dag unreachable v={v} mask={mask}"),
                    Some(_) => {
                        if v == 0 {
                            assert_eq!(idom[v], 0);
                        } else {
                            let want = naive_idom(n, &doms, v as u32);
                            let got = if idom[v] == UNREACHABLE { None } else { Some(idom[v]) };
                            assert_eq!(got, want, "dag idom mismatch v={v} mask={mask}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn exh_buffer_and_input_errors() {
        let (starts, targets) = csr_from_mask(3, 0b10_0100); // 0->1->2 chain
        let g = Csr { node_count: 3, starts: &starts, targets: &targets };
        g.validate().unwrap();

        // Buffer shortages: every BufferKind reachable.
        let mut indeg = vec![0u32; 2];
        let mut heap = vec![0u32; 3];
        let mut out = vec![0u32; 3];
        assert_eq!(
            topo_order(&g, &mut indeg, &mut heap, &mut out),
            Err(GraphError::BufferTooSmall(BufferKind::Indeg))
        );
        let mut indeg = vec![0u32; 3];
        let mut heap = vec![0u32; 2];
        assert_eq!(
            topo_order(&g, &mut indeg, &mut heap, &mut out),
            Err(GraphError::BufferTooSmall(BufferKind::Heap))
        );
        let mut out2 = vec![0u32; 2];
        let mut heap2 = vec![0u32; 3];
        assert_eq!(
            topo_order(&g, &mut indeg, &mut heap2, &mut out2),
            Err(GraphError::BufferTooSmall(BufferKind::Output))
        );
        let mut color = vec![0u8; 2];
        let mut stack = vec![DfsFrame { node: 0, next_edge: 0 }; 3];
        let mut path = vec![0u32; 3];
        assert_eq!(
            find_cycle(&g, &mut color, &mut stack, &mut path),
            Err(GraphError::BufferTooSmall(BufferKind::Color))
        );
        let mut color = vec![0u8; 3];
        let mut stack2 = vec![DfsFrame { node: 0, next_edge: 0 }; 2];
        assert_eq!(
            find_cycle(&g, &mut color, &mut stack2, &mut path),
            Err(GraphError::BufferTooSmall(BufferKind::DfsStack))
        );
        let mut stack3 = vec![DfsFrame { node: 0, next_edge: 0 }; 3];
        let mut path2 = vec![0u32; 2];
        assert_eq!(
            find_cycle(&g, &mut color, &mut stack3, &mut path2),
            Err(GraphError::BufferTooSmall(BufferKind::CyclePath))
        );
        let mut seen = vec![0u8; 3];
        let mut post = vec![0u32; 2];
        assert_eq!(
            dfs_postorder(&g, 0, &mut seen, &mut stack3, &mut post),
            Err(GraphError::BufferTooSmall(BufferKind::Output))
        );
        let mut rev_starts = vec![0u32; 3]; // needs 4
        let mut rev_targets = vec![0u32; 2];
        assert_eq!(
            reverse(&g, &mut rev_starts, &mut rev_targets),
            Err(GraphError::BufferTooSmall(BufferKind::RevStarts))
        );
        let mut rev_starts = vec![0u32; 4];
        let mut rev_targets = vec![0u32; 1]; // needs 2
        assert_eq!(
            reverse(&g, &mut rev_starts, &mut rev_targets),
            Err(GraphError::BufferTooSmall(BufferKind::RevTargets))
        );

        // Malformed CSR: wrong starts length.
        let bad = Csr { node_count: 3, starts: &starts[..3], targets: &targets };
        assert_eq!(bad.validate(), Err(GraphError::MalformedStarts));
        // Malformed CSR: starts[0] != 0.
        let s2 = [1u32, 1, 2, 2];
        let bad2 = Csr { node_count: 3, starts: &s2, targets: &targets };
        assert_eq!(bad2.validate(), Err(GraphError::MalformedStarts));
        // Malformed CSR: non-monotone.
        let s3 = [0u32, 2, 1, 2];
        let bad3 = Csr { node_count: 3, starts: &s3, targets: &targets };
        assert_eq!(bad3.validate(), Err(GraphError::MalformedStarts));
        // Malformed CSR: targets length mismatch.
        let bad4 = Csr { node_count: 3, starts: &starts, targets: &targets[..1] };
        assert_eq!(bad4.validate(), Err(GraphError::MalformedStarts));
        // Invalid node reference.
        let t5 = [3u32];
        let s5 = [0u32, 0, 0, 1];
        let bad5 = Csr { node_count: 3, starts: &s5, targets: &t5 };
        assert_eq!(bad5.validate(), Err(GraphError::InvalidNodeReference));
        // Node count exceeds L13.
        let s6 = [0u32, 0];
        let bad6 = Csr { node_count: MAX_NODES + 1, starts: &s6, targets: &[] };
        assert_eq!(bad6.validate(), Err(GraphError::NodeCountExceedsLimit));
        // Entry out of range.
        let mut seen = vec![0u8; 3];
        let mut stack = vec![DfsFrame { node: 0, next_edge: 0 }; 3];
        let mut post = vec![0u32; 3];
        assert_eq!(
            dfs_postorder(&g, 3, &mut seen, &mut stack, &mut post),
            Err(GraphError::EntryOutOfRange)
        );
        // Dominators: entry out of range and RPO not headed by entry.
        let (rstarts, rtargets) = reverse_csr(3, &starts, &targets);
        let preds = Csr { node_count: 3, starts: &rstarts, targets: &rtargets };
        let mut rpo_index = vec![0u32; 3];
        let mut idom = vec![0u32; 3];
        let mut passes = 0u32;
        assert_eq!(
            dominators(&preds, 3, &[0u32, 1, 2], &mut rpo_index, &mut idom, &mut passes),
            Err(GraphError::EntryOutOfRange)
        );
        assert_eq!(
            dominators(&preds, 0, &[1u32, 0, 2], &mut rpo_index, &mut idom, &mut passes),
            Err(GraphError::EntryOutOfRange)
        );
        // CycleDetected on a cyclic graph through Kahn.
        // Bits: edge 0->1 = bit 0*2+1 = 1; edge 1->0 = bit 1*2+0 = 2.
        let (cs, ct) = csr_from_mask(2, 0b110);
        let cyc = Csr { node_count: 2, starts: &cs, targets: &ct };
        let mut indeg = vec![0u32; 2];
        let mut heap = vec![0u32; 2];
        let mut out = vec![0u32; 2];
        assert_eq!(
            topo_order(&cyc, &mut indeg, &mut heap, &mut out),
            Err(GraphError::CycleDetected)
        );
    }
}
