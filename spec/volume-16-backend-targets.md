<!--
CEP:FILE: spec/volume-16-backend-targets.md
CEP:WHAT: Normative specification of Planck Volume 16 — target architecture system (target description schema, initial targets), instruction selection, register allocation, instruction scheduling, stack and frame lowering, prologue/epilogue generation, target-specific optimization, and machine-code emission.
CEP:WHY: design.md §23 (lines 2609–2755) states 99 items that each need an explicit definition, implementation contract, or validation strategy; this volume defines every one of them exactly so the backend has exactly one normative home per contract.
CEP:STATUS: complete
CEP:FAILURE: A design.md §23 bullet without a coverage-ledger row here is an undefined backend behavior; an implementation contradicting any rule in this volume is non-conforming and must be corrected, not bent.
CEP:ASSUMES: SPEC-INDEX.md §3 (targets = x86-64 SysV + AArch64 AAPCS64 implemented in edition P1; ABI policy; UB model) and §5.8 (pinned register-allocation math, embedded verbatim in §16.4); Volume 15 (Machine IR = selector input); Volume 10 (calling-convention tables); Volume 17 (relocation and object contracts); Volume 12 (limits); verification standards.md G9, G10, G11, G13, G14, G15, G16.
CEP:COST: document; the pinned algorithms bound compiler work per function (liveness O(E·V); coloring O(n·|class|); scheduler O(n log n); relaxation ≤ 8 passes); no runtime cost of its own.
CEP:EVIDENCE: verification matrix ("Verification matrix", OB-16-1 … OB-16-14) and coverage ledger ("Coverage ledger") at the end of this file.
-->

# Volume 16 — Backend and targets

## 16.0 Scope and traceability

This volume defines design.md §23 "Target architecture system", lines 2609–2755, subsections 23.1–23.9.

Bullets in scope: 99, distributed as follows.

| design.md subsection | Bullets | Defined in |
| -------------------- | ------- | ---------- |
| 23.1 Target description | 16 | §16.1 (T1–T16) |
| 23.2 Initial architecture targets | 3 | §16.2 (S1–S3) |
| 23.3 Instruction selection | 10 | §16.3 (SEL1–SEL10) |
| 23.4 Register allocation | 15 | §16.4 (RA1–RA15) |
| 23.5 Instruction scheduling | 11 | §16.5 (SCH1–SCH11) |
| 23.6 Stack and frame lowering | 13 | §16.6 (FR1–FR13) |
| 23.7 Prologue and epilogue generation | 9 | §16.7 (PE1–PE9) |
| 23.8 Target-specific optimization | 10 | §16.8 (TO1–TO10) |
| 23.9 Machine code emission | 12 | §16.9 (EM1–EM12) |

Upstream input: Machine IR functions (Volume 15), target flags from Volume 12 limits. Downstream outputs: relocation records, symbols, section contents, and unwind metadata consumed by Volume 17 and Volume 18. The two non-bullet sentences of §23.2 ("Each needs a separate architecture specification" and "Do not assume that an instruction available on one target has equivalent behavior on another") are defined normatively in §16.2 (rules CT-1, CT-2).

## 16.1 Target description (defines 23.1)

Every supported target is described by one `TargetDesc` value; the backend must not hardcode target assumptions outside this record.

```rust
pub struct TargetDesc {
    pub arch: Arch,                        // T1
    pub subarch: SubArch,                  // T2
    pub isa_features: FeatureSet,          // T3
    pub regs: &'static [RegDef],           // T4
    pub classes: &'static [RegClass],      // T5
    pub encodings: EncodingModel,          // T6
    pub addr_modes: &'static [AddrMode],   // T7
    pub call_convs: &'static [CallConv],   // T8
    pub stack_rules: StackRules,           // T9
    pub data_layout: DataLayout,           // T10
    pub endianness: Endianness,            // T11
    pub ptr_width: PtrWidth,               // T12
    pub atomics: AtomicCaps,               // T13
    pub simd: SimdCaps,                    // T14
    pub feature_deps: &'static [(Feature, Feature)], // T15
    pub abi_requirements: AbiSpec,         // T16
}
```

Field contracts (each field exactly one of the enumerated values; unknown names are compile-time rejections with error E-TGT-nn as listed):

| ID | Field | Exact contract |
| -- | ----- | -------------- |
| T1 | Architecture name | `enum Arch { X86_64, AArch64, RiscV64 }`; string forms `"x86_64"`, `"aarch64"`, `"riscv64"`; the `--target` arch component must equal the string form, else E-TGT-01. |
| T2 | Subarchitecture | Closed enums: `X86_64 ∈ { X86_64_BASE, X86_64_V2, X86_64_V3, X86_64_V4 }`; `AArch64 ∈ { ARMV8A, ARMV9A }`; `RiscV64 ∈ { RV64I, RV64GC }`. Defaults: `X86_64_BASE`, `ARMV8A`, `RV64GC`. |
| T3 | Instruction set | `FeatureSet` is a finite subset of the per-target feature lists of §16.2.1–§16.2.3. A set is well-formed iff every pair of T15 is satisfied; unknown feature name = E-TGT-03. |
| T4 | Register file | `RegDef { name: &'static str, enc: u8 }`; full tables in §16.2.1–§16.2.3; encodings are the ABI register numbers. |
| T5 | Register classes | `RegClass { name, regs: &[RegDef] }`; the declaration order of `regs` is the First-Fit order consumed by §16.4; classes per target in §16.2.1–§16.2.3. |
| T6 | Instruction encodings | `enum EncodingModel { X86VarLen, A64Fixed4, RvcVarLen }`; X86VarLen: 1–15 bytes, ≤ 4 legacy prefixes + 1 REX (VEX/EVEX declared, not implemented in P1); A64Fixed4: exactly 4 bytes per instruction; RvcVarLen: 2 or 4 bytes (declared). |
| T7 | Addressing modes | Exact per-target enums, §16.2.1–§16.2.3. |
| T8 | Calling conventions | `enum CallConv { SysV64, Aapcs64, PlanckFastX64, PlanckFastA64, C }`; `C` aliases `SysV64` on X86_64 and `Aapcs64` on AArch64; full parameter-classification tables are Volume 10 §ABI; this volume owns the machine-level consequences (§16.6, §16.7). |
| T9 | Stack rules | `StackRules { slot_align: 16, growth: Down, red_zone: u32, probe_threshold: u32, sp_alignment: 16 }`; `red_zone` = 128 on X86_64, 0 on AArch64; `probe_threshold` = 4096 on both. |
| T10 | Data layout | Scalar table (size/align in bytes): `bool 1/1, i8 1/1, u8 1/1, i16 2/2, u16 2/2, i32 4/4, u32 4/4, i64 8/8, u64 8/8, i128 16/16, u128 16/16, f32 4/4, f64 8/8, ptr 8/8`. `usize`/`isize` = 8/8 on all P1 targets. Aggregate layout rules are Volume 05 (`repr` policy is pinned there). |
| T11 | Endianness | `enum Endianness { Little, Big }`; X86_64 = Little; AArch64 = Little (Big declared, not implemented); RiscV64 = Little. |
| T12 | Pointer widths | 64 bits on all P1 targets; valid address range [0, 2^48) for hosted targets; address arithmetic is performed in 64-bit GPRs; provenance is Volume 05. |
| T13 | Atomic capabilities | Exact per-target tables in §16.2.1–§16.2.3 (sizes, RMW ops, ordering support). |
| T14 | SIMD capabilities | Exact per-target tables in §16.2.1–§16.2.3 (vector widths per feature). |
| T15 | Feature dependencies | Closed list of prerequisite pairs per target (§16.2.1–§16.2.3); a feature set containing a feature without its prerequisites is E-TGT-05. |
| T16 | ABI requirements | `AbiSpec { volatile: &[RegDef], nonvolatile: &[RegDef], fp_param_regs, stack_arg_align, aggregate_return }` per target; summary tables in §16.2.1–§16.2.3; full classification tables Volume 10. |

## 16.2 Initial architecture targets (defines 23.2)

Settled per SPEC-INDEX §3: edition P1 implements code generation for exactly two targets, x86-64 (SysV ABI) and AArch64 (AAPCS64). RISC-V RV64 is declared: its `TargetDesc` exists, and selecting `--target=riscv64` with an object/executable emit mode is a compile-time rejection E-TGT-02 naming the implemented set; `--emit=mir` is permitted. Rule CT-1: each implemented target has its own architecture specification in this volume (§16.2.1, §16.2.2); the declared target has the skeleton §16.2.3. Rule CT-2 (banned): no selection pattern, cost entry, or lowering rule may be shared by cross-target analogy with another target's behavior; every pattern belongs to exactly one target's table, and porting a rule to another target requires a new pattern entry with its own verification row.

### 16.2.1 x86-64 (SysV), implemented

| Item | Exact value |
| ---- | ----------- |
| Register file | GPR64 encodings 0–15: RAX=0, RDX=1, RCX=2, RBX=3, RSI=4, RDI=5, RBP=6, RSP=7, R8=8 … R15=15. XMM0–XMM15 encodings 0–15. |
| Reserved registers | RSP (stack pointer), RBP (frame pointer; reserved in all of P1, including FP-eliminated frames). |
| Register classes and First-Fit order | `GPR64`/`GPR32`/`GPR16`/`GPR8`: RAX, RDX, RCX, RBX, RSI, RDI, R8–R15 (available = 14). `XMM`: XMM0–XMM15 (available = 16). Subregister classes are the same physical registers at narrower widths. |
| Caller-saved (volatile) | RAX, RCX, RDX, RSI, RDI, R8–R11, XMM0–XMM15. |
| Callee-saved (nonvolatile) | RBX, R12–R15 (RBP saved/restored by the frame rule of §16.6). |
| Features | Baseline: sse2. Optional: sse4_1, sse4_2, popcnt, avx, avx2, bmi1, bmi2, fma, aes. Declared, not implemented: avx512f. |
| Feature dependencies | sse4_1 → sse2; sse4_2 → sse4_1; popcnt → (none); avx → sse4_2; avx2 → avx; bmi1 → avx; bmi2 → avx2; fma → avx; aes → sse4_1; avx512f → avx2 (declared). |
| Addressing modes | `Reg`, `Imm8/Imm32`, `M { base, index, scale ∈ {1,2,4,8}, disp: i32 }`, `RipRel32`. |
| Atomics | 1/2/4/8-byte loads, stores, and RMW via lock-prefixed instructions; SeqCst store = `xchg`; CAS = `lock cmpxchg`; `cmpxchg16b` (16-byte CAS) declared; no 16-byte plain atomic load/store in P1. |
| SIMD | 128-bit (sse2 baseline); 256-bit (avx2); AVX-512 declared. |
| Conventions | SysV64: integer/pointer args RDI, RSI, RDX, RCX, R8, R9; FP args XMM0–XMM7; return RAX / XMM0; stack args right-to-left in 8-byte slots; RSP ≡ 0 (mod 16) at each call. |

### 16.2.2 AArch64 (AAPCS64), implemented

| Item | Exact value |
| ---- | ----------- |
| Register file | GPR64 X0–X30 encodings 0–30; SP encoding 31 (never allocated); V0–V31 encodings 0–31. |
| Reserved registers | SP, X29 (frame pointer; FP is always kept in P1), X30 (link register), X18 (platform register). |
| Register classes and First-Fit order | `GPR64`/`GPR32`: X0–X17, X19–X28 (available = 28; W0–W28 are the 32-bit views). `V128`: V0–V31 (available = 32). |
| Caller-saved (volatile) | X0–X17, V0–V7, V16–V31. |
| Callee-saved (nonvolatile) | X19–X28, V8–V15 (Planck P1 saves full 128-bit V8–V15; stricter than AAPCS64's low-64-bit requirement, which is permitted). |
| Features | Baseline: fp, advsimd. Optional: crc, lse. Declared, not implemented: rcpc, sve. |
| Feature dependencies | lse → ARMV8A; ARMV9A → ARMV8A; sve → ARMV9A (declared). |
| Addressing modes | `Reg`, `Imm12 (scaled by element size)`, `RegOffset { base, reg, shift ≤ 3 }`, `PcRel21 (adr)`, `PcRelPage21 (adrp)`, `Literal32/64`. |
| Atomics | With lse: `ldadd`/`swp`/`cas` for 1/2/4/8 bytes. Without lse: `ldxr/stxr` retry loops (bound MAX_LDXR_RETRIES = 2^20, exceeded → defined trap abort 134 to avoid livelock). No 16-byte atomics in P1. |
| SIMD | 128-bit AdvSIMD baseline; SVE declared. |
| Conventions | Aapcs64: integer/pointer args X0–X7; FP/SIMD args V0–V7; HFA composites in consecutive V registers; composites > 16 bytes by hidden pointer in X8, returned in X0; SP must be 16-byte aligned at every point where it is read as a base or at a call. |

### 16.2.3 RISC-V RV64GC, declared

RV64GC has a complete `TargetDesc` skeleton: arch `RiscV64`, subarch `RV64GC`, features {i, m, a, f, d, c, v(declared)}, GPR X0–X31 (X0 = zero; X1 = ra, X2 = sp, X8 = fp reserved; available = 28), FPR F0–F31, endianness Little, pointer width 64. No selection patterns, no encodings, and no emission exist for it in P1; the backend rejects object/executable emission for it (E-TGT-02). This skeleton is the contract any future RV64 implementation must extend; it defines the design bullet "RISC-V 64-bit" normatively as declared-not-implemented.

## 16.3 Instruction selection (defines 23.3)

Selection operates on Machine IR functions (Volume 15). The interface is a per-target pattern table:

```rust
pub const MAX_PATTERN_NODES: u32 = 6;
pub const MAX_PATTERNS_PER_TARGET: usize = 2048;

pub struct SelPattern {
    pub id: u32,                          // unique per target, dense from 0
    pub root: MOpcode,
    pub shape: PatternTree,               // rooted tree, ≤ MAX_PATTERN_NODES nodes
    pub result: RegClassId,               // class of the produced value
    pub emission: &'static [MInstrTemplate],
    pub cost: u32,                        // cost units of §16.3.4
    pub requires: FeatureSet,
    pub mem_order: Option<MemOrder>,      // atomic patterns only
}
```

SEL1 Selection rules. Selection must cover every MIR node that is effectful or that has uses; the cover is a forest of pattern applications, one application per matched subtree root. The matcher is a bottom-up dynamic program: nodes are visited in reverse effect order (the effect chain order of Volume 15), state is `(node, RegClassId)`, and the value is the minimum total cost of a cover of the subgraph rooted at the node ending in that class. Multi-node patterns consume child results recursively through the memo table. Worst-case complexity per function: O(N · P_total · C) where N = node count, P_total ≤ MAX_PATTERNS_PER_TARGET, C = class count; termination is guaranteed by acyclicity of the MIR DAG and memoization.

SEL2 Pattern matching. The table is a const array sorted by `id`. Candidates for a node are located by root-opcode index; a pattern applies iff its shape matches structurally, `requires ⊆ isa_features`, and every operand constraint is satisfiable. Exact tie-break: among candidate covers with equal total cost, the matcher chooses the cover whose ordered sequence of pattern IDs is lexicographically smallest. The output for a tie is therefore a pure function of the input (G14). Emission order is post-order over the chosen cover.

SEL3 Legalization. Before matching, a legalization pass rewrites unsupported constructs. Closed rule set for P1: (L-1) 128-bit integer ops on X86_64 expand to register-pair sequences; (L-2) div/rem on a target without a division instruction expand to calls of `__planck_div64`/`__planck_rem64`; (L-3) atomic ops at a width not in T13 expand to calls of `__planck_atomic_<size>_<op>`; (L-4) f32 ops on a soft-float configuration expand to calls; (L-5) immediates outside the encoding range of T7 are materialized into a register or the literal pool. Termination measure: the number of nodes whose opcode is not in the target's legal set strictly decreases per application; hard bound: ≤ 4·N applications per function, exceeded = ICE SEL-01.

SEL4 Instruction costs. Cost unit = 1 per cycle of latency in the target cost model. Generic-profile latency tables (cost = latency in cycles): X86_64: alu 1, mov 1, lea 1 (3-operand lea 3), load 3, store 1, mul64 3, div64 26, fadd 3, fmul 4, fdiv 14, sqrt 16, branch 1, shuffle 1, vadd 1. AArch64: alu 1, mov 1, load 3, store 1, mul 3, madd 3, sdiv 13, fadd 3, fmul 3, fdiv 9, branch 1, shuffle 1. Costs are per tuning profile (TO10); the tables above are the GENERIC profile and are normative for it.

SEL5 Target-specific lowering. The lowering hook interface is fixed: `lower_call`, `lower_return`, `lower_varargs`, `lower_inline_asm`, `lower_atomic`. Each hook is a pure function `(MIR node, TargetDesc) -> Result<[MInstr], LowerError>`; hooks must preserve the effect chain and are the only permitted sites for target-specific instruction sequences that are not pattern-expressible.

SEL6 Complex addressing selection. A memory operand folding `base + index·scale + disp` is selected iff: (a) every input node has def-use count 1 or is a constant; (b) no call or effectful node lies between the address computation and its use in the effect chain; (c) the folded mode exists in T7. When two foldable operands compete, the operand of the use that appears earlier in effect order is folded. These rules make folding deterministic.

SEL7 SIMD selection. A vector op of width w maps to an ISA instruction iff w ≤ max width of T14 for the active feature set and an exact-semantics pattern exists; otherwise the op splits element-wise into scalar ops. Reductions map to target reduction instructions where available (AArch64: `addv`, `fmaxv`; X86_64: the pinned shuffle-reduce sequences of the target table); the x86_64 floating-point reduction sequences preserve left-to-right association exactly (no reassociation is permitted).

SEL8 Atomic selection. The (op, width, ordering) → instruction mapping is the T13 table of §16.2.1/§16.2.2, which is normative. Exact settled choices: X86_64 SeqCst store = `xchg` (never `mov` + `mfence`); CAS failure ordering on X86_64 = Acquire when the requested ordering is ≥ Acquire, else Relaxed; AArch64 without lse uses the `ldxr/stxr` loop with the MAX_LDXR_RETRIES bound; AArch64 SeqCst RMW with lse is `ldadd`/`swp`/`cas` followed by `dmb ish` per the Volume 07 memory model mapping.

SEL9 Inline assembly integration. An `asm!` block (Volume 10) becomes one opaque MIR node with its declared inputs, outputs, and clobbers. The selector emits it verbatim; register allocation sees its inputs/outputs as fixed-register-constrained operands and its clobbers as fixed intervals (§16.4 RA13); scheduling treats it as one instruction with latency UNSPEC_LATENCY = 10 and forbids reordering across it. Violations of the declared clobber/condition lists are UB-5 per SPEC-INDEX §3.

SEL10 Unsupported instruction handling. If any node has no legal cover after legalization (memo value = ∞), compilation is rejected with error E-SEL-01 naming the opcode, the target, and the active feature set. There is no fallback interpretation or emulation path.

## 16.4 Register allocation (defines 23.4)

The pinned contract of SPEC-INDEX §5.8 is embedded verbatim and is authoritative; the elaboration below must not contradict it.

> Pinned §5.8 (SPEC-INDEX.md), embedded verbatim:
>
> - Live intervals: `(v, start, end)` with `start ≤ end`, half-open `[start, end)`.
> - Interference: two intervals interfere iff they overlap and at least one is not coalescible with the other (coalescibility given by a caller-provided predicate table).
> - Coloring: degree-ordered allocation with First-Fit over the register class order; register pressure = max simultaneous live count; spill priority = `uses × 10^3 + loop_depth × 10^6 − size_cost`, exact integer math with checked multiply (u64); spills chosen in ascending benefit order until pressure ≤ available registers.
> - Laws: allocation assigns each interval exactly one location; no two interfering intervals share a location (Z3 obligation on bounded instances + EXH on generated small instances).

Elaboration:

RA1 Liveness analysis. Backward dataflow over the MIR CFG: `live_out(b) = ∪ live_in(s) for s ∈ succ(b)`; `live_in(b) = use(b) ∪ (live_out(b) − def(b))`. Worklist algorithm seeded in reverse block order; termination measure: each vreg/block pair enters the worklist at most once after reaching a fixed point; worst case O(E·V) with E = CFG edges, V = vreg count.

RA2 Live intervals. Instruction positions are consecutive u32 indices in block layout order (the order produced by §16.5, or the input order when scheduling is disabled). For each vreg: `start` = position of its defining write, `end` = last use position + 1; the interval is half-open `[start, end)`. A vreg used without def in a block is an error of the SSA checker (Volume 15), never an allocator input.

RA3 Live-range splitting. P1 performs exactly one form of splitting, applied before coloring: any interval that crosses ≥ 1 call site is split at every call-site position p into `[start, p)` and `[p, end)`. Each segment is an independent interval; its `uses`, `loop_depth`, and `size_cost` are computed per segment. No other splitting is performed in P1 (stated limitation).

RA4 Register classes. Classes and their First-Fit orders are T5 per target (§16.2.1–§16.2.2). `available(class)` is the class size minus reserved registers: GPR64 = 14 (X86_64), 28 (AArch64); XMM = 16; V128 = 32. The register class of a vreg is fixed by its MIR type and selector result class.

RA5 Register constraints. `enum RegConstraint { Any(RegClassId), Fixed(PhysReg), Class(RegClassId) }`, attached per operand by the selector; `Fixed` arises from calling conventions and inline-asm operands (SEL9); constraints beyond these three forms are a Volume 10 contract, not an allocator extension point.

RA6 Register allocation algorithm. The pipeline, in order, per function: (1) RA1 liveness; (2) RA2 intervals; (3) RA3 call-site splitting; (4) fixed intervals per RA11; (5) coalescing per RA9; (6) coloring per the pinned rule: intervals sorted by (degree descending, start ascending, interval ID ascending), each interval takes the first free register in its class order; (7) spill loop per RA7; (8) spill slots per RA8; (9) verification per RA15. Complexity: coloring is O(n log n + n·|class|); the spill loop runs ≤ n iterations.

RA7 Spill decisions. `pressure(class, p)` = count of intervals covering p (half-open) that are assigned registers of that class. While max pressure > available: select the interval with the smallest spill priority; the priority of interval a is computed exactly as `uses(a) × 10^3 + loop_depth(a) × 10^6 − size_cost(a)` in checked u64 arithmetic, where `uses(a)` = number of use positions strictly inside `[start, end)` (reads and writes other than the defining write), `loop_depth(a)` = nesting depth (number of enclosing loop headers) of the block containing `a.start`, and `size_cost(a)` = `align_up(width_bytes(type(a)), 8)` (the spill slot bytes). Checked multiply/add/sub: overflow or underflow yields error E-RA-01 (`SpillPriorityUnderflow`/`SpillPriorityOverflow`); the domain bounds (uses ≤ 2^20, depth ≤ 64, size ≤ 64) make overflow impossible in practice, but the checked math is mandatory. Spills are chosen in ascending benefit order (= ascending priority; lowest priority spilled first); tie-break: ascending vreg ID. An interval eligible for rematerialization (RA10) is rematerialized instead of spilled. When an interval is spilled, all its uses read/write its spill slot.

RA8 Spill slot allocation. Slot size = `size_cost(a)`; slots are placed in descending size, then ascending vreg ID, first-fit at each slot's alignment into the spill region of the frame layout (§16.6 FR1/FR3). Slot offsets are frame-relative and emitted into the affected instructions after coloring.

RA9 Register coalescing. The caller-provided coalescibility predicate of the pinned contract is exactly: `coalescible(a, b) ⟺ ∃ mv` in the function with `mv.op = Move ∧ mv.dst = a.vreg ∧ mv.src = b.vreg ∧ a.start = pos(mv) ∧ class(a.vreg) = class(b.vreg) ∧ neither a nor b is fixed`. Coalescing is a pre-coloring union-find pass over such move pairs, processed in ascending move position; merging two intervals yields one interval whose uses are the union at their positions. Merged intervals do not interfere with each other, implementing the pinned interference definition.

RA10 Rematerialization. Eligible defs: `ConstInt`, `ConstFloat`, `LoadGlobalAddr`, `FrameAddr` (no side effects, empty effect row minus `alloc`). When a spill decision selects an interval whose def is eligible, the def's emission template is re-emitted at each use position as a fresh 1-use interval `[p, p+1)`; the original interval is removed from the allocation problem. Rematerialized intervals do not occupy spill slots.

RA11 Caller-saved handling. Every call site c produces fixed intervals `[pos(c), pos(c)+1)` over every caller-saved register of the class in question (T16 volatile sets). Values live across c therefore interfere with those fixed intervals and cannot reside in caller-saved registers across the call, per the pinned interference law.

RA12 Callee-saved handling. The callee-saved set of a function = the nonvolatile registers (T16) assigned to at least one interval, plus the frame-pointer register rules of §16.6. The prologue saves exactly this set and the epilogue restores it (§16.7); the set is recorded in the frame layout and the unwind metadata (FR12).

RA13 Fixed-register handling. Pre-colored intervals (argument registers, return registers, RSP/RBP, inline-asm constraints) are fixed, never spilled, never coalescible, and never assigned to another interval by the interference law. When an operand constraint requires a register other than the one the computation produced, the selector inserts a move to satisfy the constraint before coloring; the inserted move is an ordinary coalescing candidate.

RA14 Register pressure. Pressure per (class, point) is defined as in RA7; the maximum per class per function is recorded as compiler statistics and is the sole termination input of the spill loop. Pressure is never used to steer selection costs in P1 (stated limitation).

RA15 Allocation verification. A post-pass checker must validate, for every function: (V1) every interval has exactly one location (register or spill slot); (V2) no two interfering intervals share a location; (V3) all `Fixed` constraints are satisfied; (V4) the callee-saved set equals the set of assigned nonvolatile registers. Any failure is an internal compiler error ICE-RA-nn (compilation aborts with diagnostic; exit code 101). This checker is the runtime bridge for the bounded proofs of the verification matrix.

## 16.5 Instruction scheduling (defines 23.5)

Scheduling operates per basic block, after selection, on MIR instructions with virtual registers.

SCH1 Dependency construction. The dependence graph of a block is built from the effect chain (Volume 15): nodes = instructions; arcs = (a) effect-chain successor arcs between all consecutive effectful nodes (which include every memory op), giving a total order on memory effects; (b) data arcs from each vreg def to each use; (c) intra-block control ordering is inherited from block layout. The graph is a DAG by construction; a cycle would require an instruction to precede itself in the effect chain, which the SSA checker rejects.

SCH2 Data dependencies. Def→use arcs carry the source latency from the cost model (SEL4).

SCH3 Memory dependencies. Memory ops that may alias (Volume 05 aliasing query, conservative) are additionally ordered by the effect chain arcs of SCH1; volatile and atomic ops are chained unconditionally.

SCH4 Control dependencies. Cross-block ordering is fixed by CFG layout; the scheduler must not move an instruction across a block boundary.

SCH5 Register pressure. During list scheduling the scheduler counts live vregs; this count is used only as tie-break SCH9-below (rule 2) and is recorded as statistics. Pressure does not abort or reorder the schedule in P1.

SCH6 Instruction latency. Field of the cost model (SEL4); each opcode has exactly one latency value per tuning profile.

SCH7 Throughput. Field of the cost model: `ops_per_cycle: u8` per opcode class (X86_64 GENERIC: alu 4, load 2, store 1, mul 1, div 0.25→recorded as 1 with latency 26, fp-add 2, shuffle 2; AArch64 GENERIC: alu 4, load 2, store 2, mul 2, fp-add 2). In P1 the scheduler uses latency heights only; throughput is recorded and drives tie-break 2 via port classes. This is a stated limitation, not an omission.

SCH8 Execution ports. Port classes per target (X86_64 GENERIC): ALU → {P0, P1, P5, P6}; Load → {P2, P3}; Store → {P4}; Mul → {P1}; Div → {P0}; FP-add → {P1, P5}; Branch → {P6}. AArch64 GENERIC: ALU/Load/Store/FP/Branch classes over ports {P0, P1, P3, P4} with ALU → {P0, P1}, Load → {P3}, Store → {P4}, FP → {P0, P1}, Branch → {P1}. Per-cycle issue limit ISSUE_LIMIT: X86_64 = 4, AArch64 = 4.

SCH9 Scheduling regions. One region = one basic block. Cross-block scheduling, trace scheduling, and global scheduling are not performed in P1 (stated limitation).

SCH10 Loop scheduling. No software pipelining and no loop-aware global scheduling in P1 (stated limitation); loop bodies are scheduled as ordinary blocks. Unrolling and loop performance belong to Volume 23.

SCH11 Target-specific models. Each tuning profile (TO10) supplies one cost model instance (latency, throughput, ports). Models are data, declared per target in `TargetDesc`, and must not change instruction semantics.

Algorithm. Cycle-driven list scheduling per region. Priority is exact: `height(i) = lat(i) + max{ height(j) : j ∈ succ(i) }`, with `height(leaf) = lat(leaf)` (leaf = no successors). Each cycle: from the ready set, issue up to ISSUE_LIMIT instructions in this exact order: (1) greater height first; (2) lower port-class pressure in the current cycle (count of already-issued instructions per port class; choose the instruction whose least-loaded port class has the smallest count); (3) smaller original position. An instruction becomes ready when all predecessors are scheduled and their latencies have elapsed. Termination: each iteration schedules exactly one instruction, so the scheduler terminates within n steps; the wall-clock bound MAX_SCHED_CYCLES = 4096 per region is a safety net: exceeding it falls back to the original instruction order (defined, deterministic fallback). Complexity: O(n log n) with a binary heap keyed by (height, port count, position).

## 16.6 Stack and frame lowering (defines 23.6)

FR1 Stack frame layout. X86_64 (frame pointer kept; offsets from RBP after prologue): `+8` return address; `+0` saved RBP; `+16 + Σ_{j<k} align_up(sz_j, 8)` incoming stack argument k; saved callee-saved registers at `−8·k` in ascending register-encoding push order; canary (if enabled) at `−8·(K+1)` directly below the last saved register; spill slots and locals first-fit downward from `RBP − 8K − 8`; frame size F = `align_up(spill + locals + outgoing_args_max + canary, 16)`. AArch64: `T = align_up(F + 16 + saved_V_bytes, 16)`; prologue `sub sp, sp, #T`; `stp x29, x30, [sp, #(T−16)]`; `mov x29, sp`; saved V8–V15 (when used) as `stp` pairs below the frame record; incoming stack argument k at `[x29, #(T + 8 + Σ_{j<k} align_up(sz_j, 8))]`; spill slots and locals first-fit upward from `sp + 0`. Outgoing stack-argument area is included in F (X86_64) or T (AArch64) as `outgoing_args_max` over all call sites.

FR2 Local stack objects. Frame objects from IR allocations are placed with first-fit: objects sorted by (alignment descending, size descending, creation ID ascending); each object's address respects its alignment; the frame size is then aligned to 16.

FR3 Spill slots. Placed by RA8 into the same region as locals, before locals (spill slots first in the sort order of RA8), so spill slots occupy the highest-address local slots on X86_64 and the lowest on AArch64 as produced by the stated first-fit rules.

FR4 Alignment. Every frame object is aligned to at least its natural alignment; the frame total is 16-aligned; SP is 16-aligned at every call site and (AArch64) whenever SP is a base register.

FR5 Stack arguments. Incoming stack argument k sits at the offset of FR1 on both targets. Outgoing stack arguments are written into the caller's outgoing area below SP in the exact classification order of the Volume 10 tables; slot j is at SP + Σ_{i<j} align_up(sz_i, 8).

FR6 Saved registers. Callee-saved set per RA12. X86_64: pushes emitted in ascending encoding order among {RBX, R12, R13, R14, R15}, pops in reverse. AArch64: GPR saves via `stp` pairs at the top of the frame record; V8–V15 via `stp`/`ldp` pairs when any V8–V15 register was assigned.

FR7 Red zones. X86_64: the 128-byte SysV red zone below RSP may be used only by functions that make no calls and perform no dynamic stack allocation; otherwise FR9 dynamic allocation would clobber it. AArch64: red zone = 0 bytes; no red zone use is permitted.

FR8 Frame pointers. RBP (X86_64) and X29 (AArch64) are reserved in P1. AArch64 always keeps the frame pointer in P1. X86_64 frame-pointer omission is permitted iff the function makes no calls, performs no dynamic stack allocation, requires no stack probe (F ≤ 4096), and has no canary; omitted frames use SP-based CFI (FR12).

FR9 Dynamic stack allocation. `alloca(n)`: round n up to 16; decrement SP by the result; restore SP at scope exit. A single `alloca` exceeding ALLOCA_MAX = 2^31 bytes is a defined trap (abort 134). The alloca size must not be assumed present in the static frame; unwind metadata uses the SP-restoration sequence for such regions.

FR10 Stack probing. If F > probe_threshold (4096), the prologue must touch every page between the old and new SP before the final SP adjustment: for k = 1 to ceil(F/4096), store scratch register r11 (X86_64) / x17 (AArch64) to `[SP − k·4096]` in descending k order. Omission of probing on a frame > 4096 bytes is a specification violation (stack probes are mandatory, not optional).

FR11 Stack protection. Canary mode is a build flag. Canary slot position: FR1 (directly below the callee-saved region on X86_64; directly below the frame record region on AArch64). The canary value is loaded at process startup from the first 8 bytes of the `AT_RANDOM` auxv entry into global `planck_canary` (u64). The epilogue compares the slot against `planck_canary` before restoring registers; mismatch calls `planck_stack_chk_fail` → abort 134.

FR12 Stack unwinding. Every function that contains a call, a panic site, or a `try` boundary must emit CIE/FDE unwind information into `.eh_frame`/`.eh_frame_hdr` (Volume 17 F13; the runtime contract is Volume 18 §18.4). Personality: `planck_eh_personality`. The FDE must cover the entire function body and encode the SP/FP rules of this section exactly; the checker of §16.9 EM12 validates FDE coverage.

FR13 Frame elimination. Defined exactly as FR8: X86_64 permits FP-eliminated frames under the four stated conditions; AArch64 never omits the frame pointer in P1. Frame elimination never removes callee-saved register saves (FR6) and never removes unwind metadata (FR12).

## 16.7 Prologue and epilogue generation (defines 23.7)

PE1 Register saving. Exact prologue templates. X86_64 (FP kept):

```
push rbp
mov  rbp, rsp
push rbx ; push r12 ; push r13 ; push r14 ; push r15   ; only those in the RA12 set, ascending encoding order
sub  rsp, F                                            ; via probe sequence first if F > 4096
mov  rax, [planck_canary] ; mov [rbp-8(K+1)], rax      ; canary mode only
```

AArch64 (FP always kept):

```
; probe pages if T > 4096 (descending k order, x17 scratch)
sub  sp, sp, #T
stp  x29, x30, [sp, #(T-16)]
mov  x29, sp
stp  v8, v9,  [sp, #(T-32)]  … stp v14, v15, [sp, #(T-16-savedV)]  ; only when used
adrp x16, planck_canary ; ldr x16, [x16, #lo] ; str x16, [sp, #canary_slot]   ; canary mode only
```

PE2 Register restoration. Epilogue is the exact reverse of PE1 in reverse order; X86_64 epilogue: `add rsp, F` (or `lea rsp, [rbp-8K]`), canary check (FR11) before any pop, pops in reverse push order, `pop rbp`, `ret`. AArch64 epilogue: canary check, `ldp` restores in reverse, `ldp x29, x30, [sp, #(T-16)]`, `add sp, sp, #T`, `ret`.

PE3 Stack allocation. X86_64: `sub rsp, F` when F ≤ 2^31 (F is always ≤ 2^31 by the Volume 12 function-size limit MAX_FUNCTION_SIZE = 64 KiB); AArch64: `sub sp, sp, #T` when T ≤ 4095; otherwise load T into x17 (`movz/movk` sequence) and `sub sp, sp, x17`. The exact threshold: AArch64 immediate path used iff T ≤ 4095.

PE4 Stack deallocation. X86_64: `add rsp, F` or `lea rsp, [rbp−8K]`; AArch64: `add sp, sp, #T` or `add sp, sp, x17` mirroring PE3. Dynamic allocas are popped to the saved SP slot before PE4 (FR9).

PE5 Frame pointer setup. X86_64: `push rbp; mov rbp, rsp`. AArch64: `stp x29, x30, [sp, #(T−16)]; mov x29, sp`. Eliminated frames (FR13) perform no FP setup and use SP-relative CFI.

PE6 Return value handling. X86_64 (SysV64): integer/pointer ≤ 64 bits → RAX; i128 → RAX:RDX; f32/f64 → XMM0; aggregates > 16 bytes → hidden pointer in RDI, result address also in RAX. AArch64 (Aapcs64): integer ≤ 64 bits → X0; i128 → X0:X1; f32/f64 → V0; HFA of ≤ 4 elements → V0–V3; composites > 16 bytes → hidden pointer in X8, result in X0. Full classification is Volume 10; these rows are the machine-level emission contract.

PE7 Calling convention compliance. The emitted prologue/epilogue/argument placement must match the Volume 10 tables for the selected convention; the compliance checker compares the emitted sequences against the convention record (verification row OB-16-12). A mismatch is ICE PE-01.

PE8 Exception-related cleanup. Functions with panic sites register their LSDA pointer in the FDE (FR12) and label their cleanup blocks as landing pads; the cleanup code is ordinary epilogue code with the unwind register-restore sequence of PE2. Unwinding across FFI boundaries is a compile-time rejection (SPEC-INDEX §3).

PE9 Platform-specific entry requirements. X86_64: at function entry RSP ≡ 8 (mod 16); the prologue must restore RSP ≡ 0 (mod 16) before any call. AArch64: SP must remain 16-byte aligned at all times; the prologue's `T` is 16-aligned. Kernel entry expectations at `_start` are defined in Volume 18 §18.2, not here.

## 16.8 Target-specific optimization (defines 23.8)

All rules in this section are MIR-to-MIR rewrites applied after selection (and, where stated, after allocation); each is guarded, deterministic, and terminating. Termination measure for the combining pass: every application strictly reduces node count or def-use edge count; hard bound ≤ 4·N applications per function, exceeded = ICE TO-01.

TO1 Instruction combining. Closed rule set for P1 (shape → replacement, guard): C-1 `add(x, mul(y, c))` with c ∈ {2,4,8} → folded addressing operand or `lea` (X86_64) / shifted-add (AArch64), guard SEL6; C-2 `add(x, x)` → `shl(x, 1)`; C-3 `sub(0, x)` → `neg(x)`; C-4 `xor(x, −1)` → `not(x)`; C-5 `and(x, x)` → `x`; C-6 `mul(x, 1)` → `x`, `mul(x, 0)` → `0`; C-7 `cmp(and(x, mask), 0)` with single-use and → `test(x, mask)`; C-8 `select(c, a, b)` where c is a comparison result → `cmov`/`csel`. Guard for all: replacement is single-use and no intervening effectful node.

TO2 Addressing-mode folding. Post-allocation peephole form of SEL6: memory operands are re-folded with allocated registers under the same three eligibility conditions and the same tie-break.

TO3 Branch optimization. Closed rules: B-1 invert a conditional branch when its target is the next block in layout; B-2 delete a branch whose target is the immediately following block; B-3 delete an empty block (no effects) whose single successor is its fallthrough; B-4 re-route single-predecessor empty blocks. Guards: no intervening effectful instructions; re-applied to fixpoint, bound ≤ 4·N.

TO4 Vector instruction optimization. Closed rules: V-1 hoist invariant splats out of loops; V-2 combine shuffle-of-shuffle into one shuffle when the target has an expressible shuffle; V-3 split vectors wider than the T14 maximum into halves, element-wise, recursively; V-4 map reductions to target instructions per SEL7 with the no-reassociation rule.

TO5 Multiply/divide lowering. Multiply by constant c: choose the minimum-total-latency chain over {shl k, add, sub, lea/shifted-add} computed by exact dynamic programming over sub-representations of c (each sub-representation is a value reachable by one operation from previously computed values; state space ≤ 64 levels × chain widths; tie-break: shorter chain, then lexicographically smallest op sequence). Division by constant d ≠ 0: the exact Granlund–Montgomery-style construction: for s = 0..63, m = ceil(2^(64+s)/d); accept the smallest s for which m fits u64 and the standard feasibility condition holds; emit multiply-high plus shift; signed division adds the bias term for negative dividends. Correctness obligations are in the verification matrix (OB-16-15 domain EXH + Z3 bound). Divide by power of two: unsigned → shift; signed → shift plus correction add for negative values (exact sequence pinned in the target table).

TO6 Bit manipulation. Exact mapping: `popcount` → `popcnt` (X86_64, feature popcnt) / `adds` across lanes or SWAR sequence with constants 0x5555555555555555, 0x3333333333333333, 0x0f0f0f0f0f0f0f0f, multiply 0x0101010101010101, shift 56; `clz` → `lzcnt`/`bsr` normalization (X86_64) / `clz` (AArch64); `ctz` → `tzcnt`/`bsf` normalization / `rbit` + `clz` (AArch64); `bswap` → `bswap` (X86_64) / `rev` (AArch64); `parity` → popcount & 1; rotates → `ror`/`rol` on both targets. SWAR sequences are the normative fallback and are fixed byte-for-byte by the constants above.

TO7 Atomic instruction selection. Identical to SEL8; the tables of §16.2 are the single normative mapping.

TO8 Cache-related hints. P1 inserts no automatic prefetch. Explicit `prefetch.read` intrinsic → `prefetcht0` (X86_64) / `prfm pldl1keep` (AArch64); `prefetch.write` → no-op on X86_64 (no write-prefetch instruction; documented) / `prfm pstl1keep` (AArch64).

TO9 Architectural instruction idioms. Closed per-target idiom tables, each entry a named guarded rewrite: X86_64: I-1 min/max via `cmp`+`cmov`; I-2 `add`+`mul 3` via `lea`; I-3 zero-extension via 32-bit operand writes; I-4 compare-and-branch fusion; I-5 `setcc` + branch elimination; I-6 `test` instead of `cmp 0`. AArch64: I-1 min/max via `csel`; I-2 bitfield extract via `ubfx`/`sbfx`; I-3 zero-extension via 32-bit writes; I-4 compare-and-branch fusion (`cbz`/`tbz`); I-5 `cset` idioms; I-6 `movk` immediate patching. Each idiom carries a cost comparison guard: applied only when the target cost model (SEL4) shows the replacement cheaper.

TO10 Microarchitecture-specific tuning. Tuning profiles, exact sets: X86_64: {GENERIC, X86_64_V2, X86_64_V3, ZNVER3}; AArch64: {GENERIC, NEOVERSE_N1, NEOVERSE_V1, APPLE_M2}; RiscV64: {GENERIC} (declared). A profile selects one cost model instance (latency, throughput, ports) and may reorder selection tie-breaks; it must never change semantics. The tripartite split of verification standards.md G10 is normative: correctness target-independent (ISA-level semantics), correctness target-dependent (feature availability per T3/T13/T14), performance microarchitecture-dependent (cost models only).

## 16.9 Machine code emission (defines 23.9)

EM1 Instruction encoding. Per-target encoder: `encode(instr) -> Result<EncodedBytes, EncodeError>`; X86_64 instruction ≤ 15 bytes (ISA limit); AArch64 exactly 4 bytes, little-endian words. Encoding tables are const per target and per profile; `EncodeError` variants: `ImmOutOfRange`, `RegMismatch`, `PrefixOverflow`. The encoder is total over the legal-instruction set produced by the selector.

EM2 Instruction sizing. `size(instr)` is exact from the encoding table; during relaxation every branch is conservatively sized at its maximum (X86_64: 5 bytes for near jcc/jmp; AArch64: 4). `mov reg64, imm64` = 10 bytes on X86_64.

EM3 Branch relaxation. X86_64 only; AArch64 branch sizes are fixed and their ranges are validated instead (EM12). Phase 1: emit all branches at maximum size and compute label distances. Phase 2: shrink any conditional branch whose distance d satisfies −128 ≤ d ≤ 127 to the 2-byte rel8 form, and any unconditional `jmp` with 0 < |d| ≤ 127 to the 2-byte form; recompute distances; repeat until no change. Hard bound MAX_RELAX_PASSES = 8, exceeded = ICE EM-02. Distance = target_offset − (branch_offset + current_branch_size).

EM4 Alignment. Function starts are 16-byte aligned. Padding bytes: X86_64 = 0xCC (int3); AArch64 = 0xD503201F (nop), little-endian. Jump tables are 8-byte aligned. Loop-header alignment is not performed in P1 (stated limitation).

EM5 Relocation emission. Relocations are collected per section and emitted in ascending offset order (G14); kinds, patch math, and overflow checks are Volume 17 (pinned §5.9). The emitter must record `(offset, kind, symbol, addend)` exactly as produced by the selector; no target-specific relocation kinds are invented at this stage.

EM6 Symbol references. Fixed reference mapping: named call/jmp → `X86RipRel32` (X86_64) / `ArmBranch26` (AArch64); RIP-relative memory operands → `X86RipRel32`; 64-bit absolute address loads → `Abs64`; AArch64 `adrp`+`add` symbol address materialization → one internal `ArmAdrp` record expanded at object emission into the two ELF relocations of the Volume 17 expansion table.

EM7 Literal pools. AArch64: constants not encodable as `mov/movk` immediates (f32/f64 immediates, 64-bit constants outside the movz/movk budget) are loaded via `ldr (literal)` from a per-function pool appended after the function body; reach is guaranteed because MAX_FUNCTION_SIZE = 64 KiB ≤ POOL_REACH = 1 MiB. X86_64: such constants are placed in `.rodata` and loaded RIP-relative.

EM8 Constant islands. The per-function pool of EM7 is the constant-island mechanism; mid-function islands are not used in P1 (stated limitation).

EM9 Section placement. Mapping: machine code → `.text`; AArch64 literal pools → `.text`; jump tables → `.rodata`; source constants → `.rodata`; mutable initialized globals → `.data`; zero-initialized → `.bss`; TLS → `.tdata`/`.tbss`. Function and section ordering is ascending symbol-ID order (G14); no size or frequency reordering in P1 (stated limitation).

EM10 Debug information. DWARF 4. `.debug_line` line tables with one row per instruction, file/dir tables from the Volume 11 source spans; minimal `.debug_info` compile unit (name, comp_dir, language code 0x8001 = Planck); `.debug_abbrev` contains exactly the abbreviation table the CU's actual DIE set requires (one entry per distinct DIE shape present in `.debug_info`, in ascending tag order); `.debug_str` contains exactly the string table referenced by `.debug_info` and `.debug_line` (each referenced string once, in first-reference order). Both are derivable from the emitted `.debug_info` and `.debug_line` by construction — never hand-selected. Variable-level debug info is not emitted in P1 (stated limitation).

EM11 Code metadata. Per-instruction source spans are retained in compiler memory (not in the object) for diagnostics and G1 differential checking. Object-level metadata is the `.note.planck` note of Volume 17 F14.

EM12 Emission validation. Per function, before the object is written: (V1) decode the emitted bytes with the target's decoder table; the decoded instruction list must equal the emitted list (count, opcodes, operands); (V2) every relocation offset must lie strictly inside its instruction's operand field and inside the section; (V3) every branch distance must be within its encoding range after relaxation; (V4) the FDE of FR12 must cover the full body. Any failure is ICE EM-0n (abort, exit 101).

## 16.10 Deliverables mapping

design.md §23 "Deliverables" line maps as: Target Description Specification = §16.1–§16.2; Instruction Selection Specification = §16.3; Register Allocation Specification = §16.4; Scheduling Specification = §16.5; Code Generation Specification = §16.6–§16.9.

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-16-1 | Every declared `TargetDesc` (3 instances) satisfies all 16 field contracts of §16.1 with complete enumerations | EXH (all 3 descriptors against the schema) | FV-0 | tools/fv/ob16-1-desc.z3 | G15, G13 |
| OB-16-2 | For every pattern application on a bounded MIR subtree (≤ 3 nodes, opcodes {Add, Sub, Mul, And, Or, Xor, Shl, Load, Store, ConstInt}, i64 values mod 2^16), the emitted sequence computes the subtree's value | Z3 (≤ 3-node subtrees) + EXH (all ≤ 3-node subtrees over the op set and value domain mod 2^8) | FV-1 (gap beyond bound; runtime bridge = G1 differential suite) | tools/fv/ob16-2-sel.smt2 | G9, G1 |
| OB-16-3 | Pattern-table totality: every opcode of the closed core MIR op set (48 opcodes) has ≥ 1 applicable pattern per implemented target | EXH (48 × 2 targets) | FV-0 | tools/fv/ob16-3-totality | G9, G15 |
| OB-16-4 | Selection output is a pure function of (MIR, TargetDesc): the tie-break (lexicographic pattern-ID sequence) is total and input-order independent | REVIEW (invariant: all candidate orderings derive from the total order on pattern IDs; matcher consumes arrays in index order) | FV-2 | review/ob16-4-determinism.md | G14, G15 |
| OB-16-5 | Pinned law: allocation assigns each interval exactly one location; no two interfering intervals share a location | Z3 (≤ 8 intervals, ≤ 4 registers) + EXH (all interval sets with ≤ 4 intervals over 3 registers, coalescibility flags exhaustive) | FV-1 (bounded; runtime bridge = RA15 verifier per compilation) | tools/fv/z3/regalloc_laws.smt2 (Z3 5.4.1, artifact tools/fv/z3/artifacts/regalloc_laws.result) + `planck-regalloc-math` EXH censuses | G9, G15 |
| OB-16-6 | Spill priority = uses·10^3 + depth·10^6 − size in checked u64; overflow/underflow → E-RA-01 | Z3 (uses ≤ 2^20, depth ≤ 64, size ≤ 64) + EXH (all 27 corner combinations of {0, 1, max} per component) | FV-0 (corner domain complete) + stated bound gap | tools/fv/ob16-6-spill.smt2 | G9, G13 |
| OB-16-7 | Spill loop terminates with pressure ≤ available or all non-fixed intervals spilled; invariant fixed(class) ≤ available(class) from §16.2 | Z3 (≤ 6 intervals, ≤ 4 registers) + REVIEW (each iteration strictly increases the spilled count) | FV-1 | tools/fv/ob16-7-term.smt2 | G11, G9 |
| OB-16-8 | Schedule correctness: in the emitted schedule every dependence arc (data, memory-chain, control) has its source before its sink | Z3 (DAG ≤ 16 nodes) + EXH (all schedules of 4-node linear chains) | FV-1 | tools/fv/ob16-8-sched.smt2 | G9 |
| OB-16-9 | RA15 checker accepts an assignment iff it satisfies the pinned laws and fixed constraints | EXH (all assignments for ≤ 4 intervals / 3 registers, satisfying and violating instances) | FV-0 | tools/fv/ob16-9-verif | G9, G15 |
| OB-16-10 | After EM3 every branch distance is within its encoding range and the fixpoint is idempotent | Z3 (≤ 10 branches) + EXH (all layouts of ≤ 5 instructions with branches) | FV-1 | tools/fv/ob16-10-relax.smt2 | G9, G11 |
| OB-16-11 | For every frame size F ∈ {0, 8, …, 65536} the emitted prologue keeps SP ≡ 0 (mod 16) at call sites on both implemented targets | EXH (8192 values × 2 targets) | FV-0 | tools/fv/ob16-11-frame | G9 |
| OB-16-12 | Prologue/epilogue/argument placement matches the Volume 10 tables for shapes {i8, i16, i32, i64, f32, f64, pair(i64,i64), agg16, agg>16} × arity ≤ 6 × 2 targets | EXH (285-case instance set) | FV-0 | tools/fv/ob16-12-abi | G9 |
| OB-16-13 | Atomic mapping totality and memory-model conformance: every (op, width, ordering) triple of the language model (8 ops × 4 widths × 6 orderings × 2 targets) has an entry meeting the Volume 07 model | EXH (384 triples) | FV-0 | tools/fv/ob16-13-atomic | G9, G21 |
| OB-16-14 | Encoder round-trip: for the full encoding tables (generic profile, 2 targets), decode(encode(i)) = i | EXH (entire table domain) | FV-0 (table domain; beyond-corpus gap stated) | tools/fv/ob16-14-enc | G9, G17 |
| OB-16-15 | Multiply-by-constant DP chains and magic-number division produce quotients/products identical to the reference semantics | Z3 (64-bit divisor ≤ 2^12, sampled dividends) + EXH (divisor d ∈ [2, 65535], both signed/unsigned, dividend set {0, 1, d−1, d, d+1, 2^31, 2^32, 2^63−1}) | FV-1 | tools/fv/ob16-15-muldiv.smt2 | G9 |

## Coverage ledger

Bullets in scope: 99, defined: 99.

| design.md bullet (§) | Abbreviated bullet | Defined in |
| -------------------- | ------------------ | ---------- |
| 23.1 | Architecture name | §16.1 T1 |
| 23.1 | Subarchitecture | §16.1 T2 |
| 23.1 | Instruction set | §16.1 T3 |
| 23.1 | Register file | §16.1 T4 |
| 23.1 | Register classes | §16.1 T5 |
| 23.1 | Instruction encodings | §16.1 T6 |
| 23.1 | Addressing modes | §16.1 T7 |
| 23.1 | Calling conventions | §16.1 T8 |
| 23.1 | Stack rules | §16.1 T9 |
| 23.1 | Data layout | §16.1 T10 |
| 23.1 | Endianness | §16.1 T11 |
| 23.1 | Pointer widths | §16.1 T12 |
| 23.1 | Atomic capabilities | §16.1 T13 |
| 23.1 | SIMD capabilities | §16.1 T14 |
| 23.1 | Feature dependencies | §16.1 T15 |
| 23.1 | ABI requirements | §16.1 T16 |
| 23.2 | x86-64 | §16.2.1 S1 |
| 23.2 | AArch64 | §16.2.2 S2 |
| 23.2 | RISC-V 64-bit | §16.2.3 S3 |
| 23.3 | Selection rules | §16.3 SEL1 |
| 23.3 | Pattern matching | §16.3 SEL2 |
| 23.3 | Legalization | §16.3 SEL3 |
| 23.3 | Instruction costs | §16.3 SEL4 |
| 23.3 | Target-specific lowering | §16.3 SEL5 |
| 23.3 | Complex addressing selection | §16.3 SEL6 |
| 23.3 | SIMD selection | §16.3 SEL7 |
| 23.3 | Atomic selection | §16.3 SEL8 |
| 23.3 | Inline assembly integration | §16.3 SEL9 |
| 23.3 | Unsupported instruction handling | §16.3 SEL10 |
| 23.4 | Liveness analysis | §16.4 RA1 |
| 23.4 | Live intervals | §16.4 RA2 |
| 23.4 | Live-range splitting | §16.4 RA3 |
| 23.4 | Register classes | §16.4 RA4 |
| 23.4 | Register constraints | §16.4 RA5 |
| 23.4 | Register allocation algorithm | §16.4 RA6 |
| 23.4 | Spill decisions | §16.4 RA7 |
| 23.4 | Spill slot allocation | §16.4 RA8 |
| 23.4 | Register coalescing | §16.4 RA9 |
| 23.4 | Rematerialization | §16.4 RA10 |
| 23.4 | Caller-saved handling | §16.4 RA11 |
| 23.4 | Callee-saved handling | §16.4 RA12 |
| 23.4 | Fixed-register handling | §16.4 RA13 |
| 23.4 | Register pressure | §16.4 RA14 |
| 23.4 | Allocation verification | §16.4 RA15 |
| 23.5 | Dependency construction | §16.5 SCH1 |
| 23.5 | Data dependencies | §16.5 SCH2 |
| 23.5 | Memory dependencies | §16.5 SCH3 |
| 23.5 | Control dependencies | §16.5 SCH4 |
| 23.5 | Register pressure | §16.5 SCH5 |
| 23.5 | Instruction latency | §16.5 SCH6 |
| 23.5 | Throughput | §16.5 SCH7 |
| 23.5 | Execution ports | §16.5 SCH8 |
| 23.5 | Scheduling regions | §16.5 SCH9 |
| 23.5 | Loop scheduling | §16.5 SCH10 |
| 23.5 | Target-specific models | §16.5 SCH11 |
| 23.6 | Stack frame layout | §16.6 FR1 |
| 23.6 | Local stack objects | §16.6 FR2 |
| 23.6 | Spill slots | §16.6 FR3 |
| 23.6 | Alignment | §16.6 FR4 |
| 23.6 | Stack arguments | §16.6 FR5 |
| 23.6 | Saved registers | §16.6 FR6 |
| 23.6 | Red zones | §16.6 FR7 |
| 23.6 | Frame pointers | §16.6 FR8 |
| 23.6 | Dynamic stack allocation | §16.6 FR9 |
| 23.6 | Stack probing | §16.6 FR10 |
| 23.6 | Stack protection | §16.6 FR11 |
| 23.6 | Stack unwinding | §16.6 FR12 |
| 23.6 | Frame elimination | §16.6 FR13 |
| 23.7 | Register saving | §16.7 PE1 |
| 23.7 | Register restoration | §16.7 PE2 |
| 23.7 | Stack allocation | §16.7 PE3 |
| 23.7 | Stack deallocation | §16.7 PE4 |
| 23.7 | Frame pointer setup | §16.7 PE5 |
| 23.7 | Return value handling | §16.7 PE6 |
| 23.7 | Calling convention compliance | §16.7 PE7 |
| 23.7 | Exception-related cleanup | §16.7 PE8 |
| 23.7 | Platform-specific entry requirements | §16.7 PE9 |
| 23.8 | Instruction combining | §16.8 TO1 |
| 23.8 | Addressing-mode folding | §16.8 TO2 |
| 23.8 | Branch optimization | §16.8 TO3 |
| 23.8 | Vector instruction optimization | §16.8 TO4 |
| 23.8 | Multiply/divide lowering | §16.8 TO5 |
| 23.8 | Bit manipulation | §16.8 TO6 |
| 23.8 | Atomic instruction selection | §16.8 TO7 |
| 23.8 | Cache-related hints | §16.8 TO8 |
| 23.8 | Architectural instruction idioms | §16.8 TO9 |
| 23.8 | Microarchitecture-specific tuning | §16.8 TO10 |
| 23.9 | Instruction encoding | §16.9 EM1 |
| 23.9 | Instruction sizing | §16.9 EM2 |
| 23.9 | Branch relaxation | §16.9 EM3 |
| 23.9 | Alignment | §16.9 EM4 |
| 23.9 | Relocation emission | §16.9 EM5 |
| 23.9 | Symbol references | §16.9 EM6 |
| 23.9 | Literal pools | §16.9 EM7 |
| 23.9 | Constant islands | §16.9 EM8 |
| 23.9 | Section placement | §16.9 EM9 |
| 23.9 | Debug information | §16.9 EM10 |
| 23.9 | Code metadata | §16.9 EM11 |
| 23.9 | Emission validation | §16.9 EM12 |
