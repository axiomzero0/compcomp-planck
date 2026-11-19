<!-- CEP:FILE: spec/volume-15-machine-ir.md
CEP:WHY: design.md §22 (lines 2461–2608) requires a complete normative definition of the Machine IR: the MIR model, the register model per target with exact register sets, the instruction description format, machine memory operands, machine control flow, machine operations, MIR invariants, the MIR optimization passes on the 10-row template of design.md §40, and the four-stage machine verifier with the instruction-selection interface contract. This volume is that definition; every §22 bullet has exactly one home here.
CEP:STATUS: complete
CEP:FAILURE: A MIR function that fails any check of the applicable verifier stage (§15.9) is rejected; a pass that emits a function violating invariant MI1–MI10 is non-conforming and must be corrected, not the invariant bent. An omitted §22 bullet in the coverage ledger is a missing definition.
CEP:ASSUMES: SPEC-INDEX.md §1 (method), §3 (ABI policy: x86-64 SysV and AArch64 AAPCS64), §5.7 (pinned graph math, embedded in Volume 13 §13.2), §5.8 (register allocation math), §5.9 (relocation math); Volume 13 §13.12 cost units; Volume 14 §14.15 W11/W12 boundary; design.md §40 (10-row pass template); standards.md §3; verification standards.md G4/G5/G11/G13/G14/G15; Volumes 10 (ABI, asm), 16 (targets, encodings, scheduling tables), 17 (relocations, emission).
CEP:COST: document. Fixes compile-time costs: verifier stages O(n_inst) each (§15.9), pass costs in §15.8; MIR size limits charged against the Volume 12 limits table.
CEP:EVIDENCE: Verification matrix (OB-15-1..OB-15-12) and coverage ledger (bullets in scope: 105) at the end of this file.
CEP:FILE: spec/volume-15-machine-ir.md -->

# Volume 15 — Machine IR

## 15.0 Scope and traceability

This volume defines design.md §22 "Machine IR specification", lines 2461–2608, including the header prose (2461–2465) and the deliverables line (2605). It contains 105 bullets over subsections §22.1–§22.9. Mapping:

| design.md subsection | Lines | Bullets | Defined in |
| --- | --- | --- | --- |
| 22.1 Machine IR model | 2467–2484 | 16 | §15.1 |
| 22.2 Register model | 2486–2499 | 12 | §15.2 |
| 22.3 Instruction model | 2501–2518 | 14 | §15.3 |
| 22.4 Machine memory operands | 2520–2534 | 13 | §15.4 |
| 22.5 Machine-level control flow | 2536–2548 | 11 | §15.5 |
| 22.6 Machine-level operations | 2550–2565 | 14 | §15.6 |
| 22.7 Machine IR invariants | 2567–2578 | 10 | §15.7 |
| 22.8 Machine IR optimization | 2580–2592 | 11 | §15.8 |
| 22.9 Machine IR verification | 2594–2603 | 4 | §15.9 |

Bullets in scope: 105, defined: 105 (ledger at end of file). Deliverables of line 2605 map to: Machine IR Specification = §15.1, §15.3–§15.7; Register Model Specification = §15.2; Instruction Description Format = §15.3; Machine Verifier = §15.9. The universal/target distinction required by line 2465 is normative in §15.3: the MOpcode schema is universal; per-target instruction descriptions only instantiate it.

## 15.1 Machine IR model (§22.1)

| Bullet | Normative definition |
| --- | --- |
| Basic blocks | A MIR function is an ordered list of blocks; each block has a `BlockId` (u16), a label (§15.1 Labels), an instruction list, and exactly one terminator as its last instruction. MAX_BLOCKS = 2^16 per function. Block order is the emission order candidate; control flow is given by terminator targets, not by block order. |
| Machine instructions | One `Inst` = (opcode, operands, span-metadata); MAX_INSTS = 2^20 per function; MAX_OPERANDS = 8 per instruction. Instructions are the only mutation unit; no implicit behavior attaches to instruction order except memory dependency order and scheduling freedom (§15.8 scheduling). |
| Virtual registers | `VReg(u32)`; MAX_VREGS = 2^20 per function. VRegs carry a register class (§15.2) and, until register allocation, are the only mutable-value carriers. SSA is destroyed at the IR 2 → MIR boundary (Volume 14 §14.15 W11/W12): MIR maintains no SSA; correctness is by liveness (SPEC-INDEX §5.8 live intervals), never by dominance. |
| Physical registers | `PReg` is an enum per target with exactly the registers of §15.2; PRegs are never allocated by the register allocator's freedom — only by constraint resolution; all other allocation is of VRegs. |
| Register classes | Closed set per target: x86-64: `GPR` (64-bit general), `FPR` (128-bit xmm), `CCR` (RFLAGS), `SPR` (special, non-allocatable); AArch64: `GPR` (64-bit x registers), `FPR` (128-bit v registers), `CCR` (NZCV), `SPR`. Width-specific operands use subregisters (§15.2). |
| Stack slots | `Slot(u32)` with (size, alignment, kind ∈ {Spill, Frame, Debug}); slots are frame-relative; MAX_SLOTS = 2^16; slot alignment must satisfy its kind's natural alignment (Spill: value width; Frame: object alignment from the layout decision). |
| Machine operands | Closed operand set: `VReg(VReg)`, `PReg(PReg)`, `Imm(i64 or i128 per width)`, `Mem(MemAmode)` (§15.4), `Label(LabelId)`, `SlotRef(Slot, offset)`, `Symbol(SymId)`, `BlockRef(BlockId)`. Each opcode schema fixes the operand types per slot (§15.3). |
| Control-flow edges | Derived from terminators: each block has explicit successor and predecessor lists computed once and maintained by passes; edges are (from, to, kind ∈ {Jump, Branch, Unwind, Return}); no implicit fallthrough edges exist (every transfer is a terminator). |
| Machine memory operands | `MemAmode { base, index: Option, scale: 1|2|4|8, disp: i64 }` restricted per target (§15.4); used only in opcodes whose schema admits Mem operands. |
| Labels | `LabelId` is either a block label (BlockId), a local symbol label, or an external symbol label (SymId from Volume 12); labels are unique per function; label targets must exist (V-RA-pre). |
| Calls | `Call { callee: Direct(SymId) | Indirect(operand), args: per ABI, ret: per ABI, clobbers: caller-saved set, unwind: Option<BlockId> }`; the ABI descriptor comes from Volume 10; calls clobber all caller-saved registers (§15.2). |
| Returns | `Ret` with implicit uses of the return registers per the ABI; functions returning the never type end in Trap or Unreachable instead. |
| Branches | `Jcc { cond: CondCode, target: BlockId, fallthrough-implicit: none }` and `Jmp { target: BlockId }`; condition codes per target (§15.5). |
| Terminators | Exactly the set: `Jmp`, `Jcc`, `Jind`, `Ret`, `TailCall`, `Trap`, `Unreachable`; exactly one per block, last; no other instruction may transfer control. |
| Inline assembly | `Asm { template, operands, clobber_list, option_list }`; clobber_list is the explicit physical register + "memory" + "cc" set declared by the source (Volume 10); the declared clobbers are treated as physical definitions; violating the declared clobber/condition lists is UB-5 (SPEC-INDEX §3); the verifier checks operand classes against declared constraints (V-RA-pre). |
| Target intrinsics | `TargetIntrinsic(id, operands)`; ids come from the Volume 16 per-target intrinsic table; they are the only opaque instructions and must declare their full metadata (§15.3). |

**Formal data model (Rust signatures).** The MIR data structures are exactly:

```rust
pub struct MirFunction {
    pub name: SymId,                          // Volume 12 symbol id
    pub abi: AbiDescriptor,                   // Volume 10 convention instance
    pub blocks: IndexVec<BlockId, MirBlock>,
    pub vregs: IndexVec<VReg, VRegData>,      // VRegData { class: RegClass, from_repr: Repr }
    pub slots: IndexVec<SlotId, SlotData>,    // SlotData { size: u32, align: u32, kind: SlotKind }
    pub frame: FrameLayout,                   // total size, callee-save set, sp offsets
}
pub struct MirBlock { pub id: BlockId, pub insts: Vec<Inst>, pub term: Terminator }
pub struct Inst { pub opcode: MOpcode, pub operands: SmallVec<[Operand; 4]>, pub spans: SpanList }
pub enum Terminator {
    Jmp(BlockId), Jcc { cond: CondCode, target: BlockId }, Jind(Operand),
    Ret, TailCall(CallPayload), Trap, Unreachable,
}
pub enum Operand {
    VReg(VReg), PReg(PReg), Imm(ImmValue), Mem(MemAmode),
    Label(LabelId), SlotRef(SlotId, u32), Symbol(SymId), BlockRef(BlockId),
}
pub struct MemAmode {
    pub base: AddrReg,                       // AddrReg = VReg(VReg) | PReg(PReg); PReg only post-RA
    pub index: Option<AddrReg>,
    pub scale: AddrScale,                    // 1 | 2 | 4 | 8 (| 16 on AArch64 natural-width forms)
    pub disp: i64,                           // restricted per §15.4 at verify time
    pub space: AddrSpace,                    // AS0 | AS1(TLS)
    pub alias_set: AliasSetId,               // from IR 2 partitions; universal set when unknown
    pub volatile: bool,
}
pub enum ImmValue { I8(i8), I16(i16), I32(i32), I64(i64), I128(i128), F32(f32), F64(f64) }
```

Every field above is checked by the verifier stage that owns it (§15.9); the enums are closed and complete; adding an operand form or terminator is an edition change with a verifier update.

## 15.2 Register model (§22.2)

**x86-64 (SysV, per SPEC-INDEX §3).** The allocatable-addressable GPR set is exactly rax, rbx, rcx, rdx, rsi, rdi, r8, r9, r10, r11, r12, r13, r14, r15 (plus rsp, rbp with the roles below). Roles:

| Register | Role |
| --- | --- |
| rax | Return value register; accumulator for div/mul; caller-saved. |
| rbx | Callee-saved base; allocatable. |
| rcx | 4th integer argument; shift-count register for variable shifts; caller-saved. |
| rdx | 3rd integer argument; high part of div/mul results; caller-saved. |
| rsi | 2nd integer argument; caller-saved. |
| rdi | 1st integer argument; caller-saved. |
| r8–r9 | 5th–6th integer arguments; caller-saved. |
| r10–r11 | Caller-saved scratch. |
| r12–r15 | Callee-saved; allocatable. |
| rbp | Frame pointer when FRAME_POINTER=on (default in debug builds and whenever unwinding metadata or dynamic allocas require it); allocatable when FRAME_POINTER=off. |
| rsp | Stack pointer; reserved, never allocatable. |
| rip | Instruction pointer; implicit in rip-relative addressing only; never an operand. |

Flags: RFLAGS is the single CCR; integer ops define flags, `Jcc`/`cmovcc`/`setcc` use them; a flag clobber between def and use is a verifier error (V-RA-post). FPR: xmm0–xmm15, all caller-saved; xmm0–xmm7 pass floating/vector arguments and return float values. Widths: 8/16/32/64-bit subregisters of GPRs (al/ah/ax/eax/rax … r15b/r15d/r15); 32-bit writes zero the upper half, 8/16-bit writes do not (verifier-tracked fact); FPR widths 32/64/128 bits. Aliases: rax/eax/ax/al/ah name one physical register with different widths; the alias table is per-target data. Subregisters: a subregister operand restricts the width class; writes to ah/bh/ch/dh conflict with writes to the full register (verifier-tracked). Register pairs: div/mul use the fixed rdx:rax pair; the pair is expressed as the opcode's fixed-register constraint, never as a first-class pair operand. Fixed registers: rsp, rip, and rbp when FRAME_POINTER=on. Reserved: rsp always; rbp when FP on; x86-64 reserves no other GPR. Caller-saved: rax, rcx, rdx, rsi, rdi, r8–r11, all xmm. Callee-saved: rbx, rbp (if FP off it must still be preserved if used), r12–r15. Special-purpose: rsp, rip, RFLAGS. Register constraints: per-opcode constraint table with entries {RegClass, Fixed(PReg), TiedTo(slot), EarlyClobber, DefOnly, UseOnly}; inline-asm constraints use the Volume 10 constraint vocabulary. Allocatable GPR count R_GPR_X86 = 14 with FRAME_POINTER=on, 15 with FP off; Volume 14 §14.16 uses the FP-kept value 14.

**AArch64 (AAPCS64).** GPR set is exactly x0–x30 plus the separate stack pointer sp and zero register xzr/wzr (operand forms, not allocatable registers). Roles:

| Register | Role |
| --- | --- |
| x0–x7 | Argument/return registers; caller-saved. |
| x8 | Indirect result location register (struct return); caller-saved; allocatable when no indirect return. |
| x9–x15 | Caller-saved temporaries; allocatable. |
| x16, x17 | IP0/IP1 intra-procedure-call scratch; caller-saved; allocatable as scratch but clobbered by veneers and PLT entries. |
| x18 | Platform register; reserved; never allocatable. |
| x19–x28 | Callee-saved; allocatable. |
| x29 | Frame pointer; reserved (FP always kept on AArch64). |
| x30 | Link register; reserved; holds the return address across calls. |
| sp | Stack pointer; separate from x31/xzr; never allocatable. |

SP rules (exact): sp must remain 16-byte aligned at every instruction boundary that is, or could be, a call boundary (V-RA-post); the prologue decrements sp by a 16-aligned amount; sp may be a base register in unsigned-offset load/store forms with scaled 12-bit offsets, and in pre/post-index forms in prologue/epilogue patterns only; sp is never an index register. Flags: NZCV is the single CCR; `Jcc` uses it. FPR: v0–v31; v0–v7 arguments/returns; v8–v15 callee-saved (bottom 64 bits per AAPCS64), v16–v31 caller-saved. Widths: b/h/s/d/q (8/16/32/64/128). Aliases: w0–w30 are 32-bit views of x0–x30. Subregisters: w views; writes to w zero the upper 32 bits (verifier-tracked). Register pairs: none as first-class operands; 128-bit values use FPR q forms or explicit pairs of instructions. Fixed registers: sp, x29, x30, rip-equivalent (pc is not addressable). Reserved: sp, x18, x29, x30. Caller-saved: x0–x17, v0–v7, v16–v31. Callee-saved: x19–x28, v8–v15 (low 64 bits). Special-purpose: sp, x29, x30, NZCV. Register constraints: per-opcode table as x86-64, plus the load/store-pair tied-offset constraints (two destinations tied base+offset). Allocatable GPR count R_GPR_A64 = 28 (x0–x17, x19–x28).

## 15.3 Instruction model (§22.3)

Every target instruction is described by the universal schema below; per-target descriptions only fill the columns (line 2465's universal/target distinction is this schema). The instruction description format is exactly these 14 fields, stored as target data tables, never as code:

| Field | Bullet | Definition |
| --- | --- | --- |
| opcode | Opcode | `MOpcode`, a closed enum per target; every instance carries its opcode; no unnamed encodings. |
| operand count | Operand count | Fixed per opcode: (n_defs, n_uses); MAX_OPERANDS = 8 total. |
| operand types | Operand types | Per-slot operand type from §15.1's closed set. |
| operand constraints | Operand constraints | Per-slot: {RegClass, Fixed(PReg), TiedTo(slot), EarlyClobber, ImmRange(min,max)}; from §15.2's constraint tables. |
| definitions | Definitions | Slots written by the instruction (outputs). |
| uses | Uses | Slots read by the instruction (inputs). |
| register effects | Register effects | For each defined physical register or class: whether the definition is full-width; subregister write semantics (x86 partial writes, A64 zero-extends) are schema facts. |
| flag effects | Flag effects | One of {None, Defines, Uses, DefinesAndUses, Clobbers}; Defines/Clobbers kill the CCR live range. |
| memory effects | Memory effects | One of {None, Read, Write, ReadWrite} plus the alias-set slot (§15.4); volatile and atomic memory effects are non-eliminable and non-reorderable (except atomics among themselves per Volume 08 ordering). |
| latency model | Latency model | Integer cycle count per target from Volume 16's latency table; default 1 when the table has no entry (the table must eventually be total; missing entries are a target-description bug, G15). |
| throughput model | Throughput model | uops-per-instruction (u8) per target from Volume 16's throughput table; default 1. |
| encoding requirements | Encoding requirements | The encoding form id + fixed fields (prefix, opcode byte(s), modrm layout for x86; opcode space for A64); encodability is checked by V-emit-pre. |
| feature requirements | Feature requirements | The target feature set (members are the Volume 16 feature names, such as sse2, avx2, lse) required to encode/execute; a function must be compiled under the feature set its instructions require (checked at selection). |
| scheduling properties | Scheduling properties | The SchedClass id mapping to (latency, throughput, resource set) in Volume 16's scheduling tables; the scheduler uses only these. |

**Core opcode families and operand forms (exact).** The universal families below are the spine of every target's MOpcode set; per-target tables instantiate them (Volume 16) and may add members only inside these families or the TargetIntrinsic family. Operand forms are from §15.1's closed set; widths w ∈ {8, 16, 32, 64} for GPR and {32, 64, 128} for FPR unless stated.

| Family (opcode stem) | Operand form (defs, uses) | Constraints | Notes |
| --- | --- | --- | --- |
| `MOVRw` (register move) | def: Reg(w); uses: Reg(w) | class match; TiedTo allowed | copy propagation target |
| `MOVIw` (immediate load) | def: Reg(w); uses: Imm(w-range) | ImmRange per width | i128 → two-instruction sequence on 64-bit targets |
| `LDw` | def: Reg(w); uses: Mem | mem-eligible slot; align per §15.4 | volatile/atomic variants carry flags |
| `STw` | defs: none (memory); uses: Reg(w), Mem | mem-eligible slot | memory effect Write |
| `ADDRw` | def: Reg(w); uses: Reg(w), Reg(w) | class match | ADD/SUB: flag effect Defines |
| `MULw` | def: Reg(w); uses: Reg(w), Reg(w) | class match | flag Defines; x86-64 div/mul variants use the rdx:rax fixed pair |
| `DIVw/REMw` | def: Reg(w); uses: Reg(w), Reg(w) | x86-64: Fixed(rax), Fixed(rdx), EarlyClobber on rax | div-by-zero → Trap per §15.6 |
| `CMPw` | defs: CCR; uses: Reg(w), Reg(w) or Reg(w), Imm | CCR defined | feeds Jcc/setcc/csel |
| `Jcc` (terminator) | uses: CCR; BlockRef | cond per §15.5 | no CCR use when cond = AL (AArch64) |
| `CALL` | defs: caller-clobbered set, CCR; uses: per ABI args, callee operand | ABI per Volume 10 | unwind target optional |
| `RET` (terminator) | uses: ABI return regs | — | epilogue precedes |
| `FARITHf` (f = 32/64) | def: FReg(f); uses: FReg(f), FReg(f) | FPR class | IEEE-754 per §15.6 |
| `FLOADf/FSTOREf` | as LDw/STw with FReg | FPR class; align natural | — |
| `VECop` | def: VReg(128); uses: VReg(128) × (1..3) | FPR class; feature required | per Volume 16 vector tables |
| `ATOMICop` | def: Reg(w) (+ CCR); uses: Reg(w), Mem | mem-eligible; ordering field | sequence per §15.6 pinned mapping |
| `FENCE` | defs: none; uses: none | ordering field | mfence / DMB forms |
| `ASM` | defs/uses: declared clobber list | declared constraints | UB-5 gate |
| `TINTRIN(id)` | per Volume 16 table | per table | full §15.3 metadata required |

Every family member's instance must satisfy its schema exactly; an instruction outside the families and outside the per-target table additions is an error `OperandForm` (V-RA-pre).

## 15.4 Machine memory operands (§22.4)

`MemAmode { base, index, scale, disp }` with per-target restrictions:

| Bullet | Definition |
| --- | --- |
| Base registers | x86-64: any GPR 64-bit form, rsp allowed, rip allowed (rip-relative form only, no index); AArch64: any GPR x0–x28 or sp (sp only in unsigned-offset or pre/post-index forms; never as index). |
| Index registers | x86-64: any GPR except rsp; AArch64: any x0–x30 GPR in register-offset forms. |
| Scale | x86-64: 1, 2, 4, 8 exactly; AArch64: 1, 2, 4, 8, 16 for register-offset with natural-width extension. |
| Displacement | x86-64: disp8 (−128..127) or disp32 (−2^31..2^31−1); rip-relative: disp32; AArch64: unsigned pimm 0..4095 scaled by access size, or simm9 (−256..255) for pre/post-index, or 16-bit literal forms per opcode. |
| Addressing modes | Closed set x86-64: [base], [base+disp], [base+index×scale+disp], [rip+disp32], [disp32] absolute; AArch64: [base], [base, Xm], [base, Xm, ext/scale], [base, #simm9] pre/post-index, [base, #pimm12], literal. |
| Alignment | The access's required alignment from the layout decision (Volume 14 §14.8); misaligned access requires the unaligned-access fact and, on AArch64, only the forms that permit it; x86-64 permits misaligned for all widths but charges +1 cost (Volume 14 §14.16). |
| Memory width | 1, 2, 4, 8, 16 bytes exactly; the width must match the opcode schema. |
| Volatility | volatile flag from IR 2; volatile accesses are non-eliminable, non-combinable, non-reorderable relative to each other or to I/O. |
| Atomic ordering | One of {Relaxed, Acquire, Release, AcqRel, SeqCst} carried by atomic opcodes; mapping to per-target sequences per §15.6. |
| Address-space information | AS0 = default linear address space; AS1 = TLS; each MemAmode carries an address-space byte; only AS0/AS1 exist in edition P1. |
| Alias metadata | The alias-set id carried from IR 2 partitions (Volume 14 §14.15 W8); MIR passes use it for dependency preservation; unknown alias set = the universal set. |
| Stack references | Slots lower to sp-relative or rbp-relative (x86-64) / sp-relative (AArch64) MemAmodes with the slot's frame offset; offset range must fit disp (V-emit-pre; larger frames use a scratch register pattern emitted by the frame-setup pass). |
| TLS references | A MemAmode with address space AS1 and a TLS symbol operand; the TLS model (GeneralDynamic, LocalDynamic, InitialExec, LocalExec) is chosen by Volume 17's lowering; MIR carries the symbol + model and the per-target TLS access sequence comes from Volume 16. |

## 15.5 Machine-level control flow (§22.5)

| Bullet | Definition |
| --- | --- |
| Conditional branches | `Jcc(cond, target)`; condition codes: x86-64 the 16-entry CC set {o, no, b, ae, e, ne, be, a, s, ns, p, np, l, ge, le, g}; AArch64 the 15-entry cond set {EQ, NE, CS, CC, MI, PL, VS, VC, HI, LS, GE, LT, GT, LE, AL}; NV is banned. |
| Unconditional branches | `Jmp(target)`; every non-sequential transfer is explicit; no implicit fallthrough. |
| Indirect branches | `Jind(operand)`; operand is a GPR or Mem; used for switch dispatch and computed jumps only. |
| Calls | `Call` per §15.1; clobbers all caller-saved registers of §15.2; the unwind target, when present, is an Unwind edge to a landing-pad block. |
| Returns | `Ret`; implicit use of the ABI return registers; on AArch64 the epilogue restores x30 into a scratch or uses `ret x30`. |
| Switch lowering | Exact algorithm over cases (c1..cn, sorted) with span = max−min+1: (1) n ≤ 3 → compare chain; (2) 4 ≤ n ≤ 8 → compare chain when span ≤ 4n, else binary search tree; (3) n ≥ 8 and n/span ≥ 1/2 → jump table with bounds check; (4) otherwise → binary search tree. Thresholds pinned: JUMPTABLE_MIN_CASES = 8, JUMPTABLE_MIN_DENSITY = 1/2, BSEARCH_SPAN_RATIO = 4. |
| Jump tables | A read-only table of block labels in the rodata section (Volume 17); entry width = 4 bytes (PcRel32) or 8 (Abs64) per Volume 17; the bounds check precedes the table load; the table's relocation and range math is Volume 17's per SPEC-INDEX §5.9. |
| Branch probabilities | Per-edge metadata (u16, probability in 1/100 units): defaults: backedge 90, forward-taken 50, bounds-check-failure 1, panic/unwind edge 0; PGO overrides defaults when available; used by Volume 16 layout and by §14.16's cost model. |
| Tail calls | `TailCall(callee, args)`; preconditions: same calling convention as caller; all arguments assignable to the callee's argument registers after the caller-argument shuffle; caller stack-argument area unused or identical; no dynamic alloca in the caller frame; emitted in place of the epilogue+call sequence; counts as a terminator. |
| Exception edges | Unwind edges from Call to landing-pad blocks exist only when panic = unwind; with panic = abort, Panic lowers to Trap and no unwind edges exist; landing-pad blocks are reachable only via Unwind edges (V-RA-pre). |
| Unreachable instructions | `Unreachable` terminator: lowers to the target's debug trap (x86-64: int3; AArch64: brk #0); marks paths proven impossible; execution reaching it is a compiler bug, never defined behavior. |

## 15.6 Machine-level operations (§22.6)

The operation inventory is per-target but the categories are universal; the table fixes what each category must provide; exact opcode lists are Volume 16 target tables (MIR consumes them).

| Bullet | Required provision |
| --- | --- |
| Integer arithmetic | add, sub, mul, signed/unsigned div, signed/unsigned rem per width 8/16/32/64; flag-defining and flag-non-defining variants; 128-bit multiply/divide via the fixed-pair pattern on x86-64, via 128-bit forms on AArch64. |
| Floating-point arithmetic | add, sub, mul, div, fma, sqrt, abs, neg, min, max per binary32/binary64; IEEE-754 round-to-nearest-ties-to-even per Volume 05; no fast-math variants in edition P1. |
| Vector arithmetic | Per enabled feature set: add/sub/mul/min/max/shift/compare/blend/permute for widths 16/32 bytes; exact tables in Volume 16. |
| Comparisons | Integer compare (flags on x86-64, cmps/csel sources on AArch64), float compare with ordered/unordered variants; condition materialization via setcc (x86-64) or cset/csel (AArch64). |
| Conversions | sext, zext, trunc per integer widths; f2i (with exact in-range check or defined trap), i2f, f2f (binary32↔binary64), bitcast (no-op reinterpretation, exact width match required). |
| Bit operations | and, or, xor, not, popcnt, clz, ctz, bswap, bitrev, ror, rol; AArch64 extras (extr, bic) from Volume 16. |
| Shifts | shl, shr logical, shr arithmetic per width; variable shift count from a register; x86-64 masks the count mod width (schema fact); AArch64 uses the width-bounded forms. |
| Loads | Widths 1/2/4/8/16 bytes, signed/unsigned for sub-word; addressing per §15.4; volatile/atomic variants per flags. |
| Stores | Widths 1/2/4/8/16 bytes; addressing per §15.4; volatile/atomic variants. |
| Atomic operations | x86-64: lock-prefixed add/xchg/cmpxchg/xadd sequences; Acquire/Release on x86-64 use the SeqCst instruction plus compile-time ordering facts (x86 TSO) — pin: Acquire load = plain load + compile-time barrier fact; Release store = plain store + fact; SeqCst RMW = lock op; SeqCst fence = mfence. AArch64: LDAR/STLR for Acquire/Release/SeqCst loads/stores, LDXR/STXR or CAS/SWOP (feature lse) for RMW, DMB ISH for fences. The mapping table is Volume 16 data; MIR carries the ordering. |
| Fences | x86-64: mfence (SeqCst), lfence (Acquire), sfence (Release) as pinned above; AArch64: DMB variants OSH/ISH/NSH/LD/ST; the edition P1 mapping pins ISH for thread-level SeqCst. |
| Calls | Per §15.1; argument placement per Volume 10 ABI tables; stack-argument setup instructions are regular stores before the call. |
| Stack operations | No push/pop opcodes exist; prologue/epilogue are explicit sequences: x86-64 `sub rsp, N` + stores; AArch64 `stp/sub sp` pairs; frame-setup is a fixed pass emitting these patterns; dynamic alloca emits an sp adjustment with an alignment check. |
| Target-specific intrinsics | `TargetIntrinsic` per Volume 16's per-target intrinsic tables; each carries the full §15.3 metadata; they are the only non-derived opcodes. |

## 15.7 Machine IR invariants (§22.7)

The invariants MI1–MI10; SSA is destroyed at this point (§15.1): MI checks are liveness-based, never SSA-based.

| Bullet | Invariant (exact) |
| --- | --- |
| MI1 Virtual registers are correctly defined | Every VReg has ≥ 1 definition; every use of a VReg is reachable from one of its definitions (def-before-use on the path into the use; checked per block in linear order and across block edges by a liveness fixpoint ≤ blocks iterations). |
| MI2 Uses are valid | Every use names an existing VReg/PReg/Slot/Label/Block; slot indices within the opcode schema; no use of an undefined label or block. |
| MI3 Physical register constraints are legal | Every TiedTo pair holds the same location after RA; EarlyClobber defs are disjoint from all other operands' locations; Fixed(PReg) operands use exactly that register; constraint satisfaction is checked per instruction. |
| MI4 Instructions use supported operand forms | Every instruction's operands match its schema's operand types and constraint table (x86-64 memory operands only in mem-eligible slots; immediates within their ImmRange; no operand type outside §15.1's closed set). |
| MI5 Control flow is valid | Exactly one terminator per block, last; every BranchRef/Label target exists; every block is reachable from the entry block or removed; predecessor/successor lists agree with terminators; landing-pad blocks reachable only via Unwind edges. |
| MI6 Memory operations preserve required semantics | Volatile operations are never eliminated, duplicated, or reordered relative to each other or to I/O; atomic orderings are preserved per §15.6's pinned mapping; alias-set dependencies are respected by scheduling and RA (spill loads/stores may not cross volatile/atomic ops). |
| MI7 Calling conventions are respected | Call arguments occupy exactly the ABI's registers/stack positions per Volume 10; return values in the ABI's return registers; caller-saved registers are dead or saved across a call; callee-saved registers used by the function are saved in the prologue and restored in the epilogue on every return path. |
| MI8 Stack alignment is correct | sp is 16-byte aligned at every call boundary and at function entry/exit on both targets; stack-argument area is 16-aligned; frame size multiple of 16. |
| MI9 Special registers are handled | rsp/sp, rbp (FP-on), x29, x30, rip/pc are never allocated, never clobbered except by the fixed patterns that define them (prologue, call, epilogue); x18 never appears; RFLAGS/NZCV live ranges do not cross a flag-clobbering instruction between def and use. |
| MI10 No unlowered abstract operation remains | Every opcode ∈ the target's MOpcode set; no IR 1/IR 2 opcode, no RepConvert, no VirtualObject access, no IR-level intrinsic that Volume 16 has not mapped (V-RA-pre and V-emit-pre both enforce). |

Additionally, the two boundary invariants of the assignment: every value has a location — after register allocation every defined value resides in a PReg or a Slot on every path where it is live (V-RA-post); no virtual registers after register allocation — V-RA-post rejects any remaining VReg operand.

## 15.8 Machine IR optimization (§22.8)

The 11-pass catalog on the 10-row template of design.md §40. Rows: Pre / Analyses / Algorithm / Preserves / Invalid-if / Complexity / Profit / Verifier / Tests / Proof. All passes run on MI1–MI10-valid input and must emit MI1–MI10-valid output; the applicable verifier stage runs after every pass (G5). Costs in Volume 13 §13.12 units; mem operand cost 3.

| Pass | Pre | Analyses | Algorithm | Preserves | Invalid-if | Complexity | Profit | Verifier | Tests | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Machine constant propagation | MI1–MI10 | flag-state per block | Fold ops whose operands are all immediates (exact semantics per §15.6, incl. overflow/shift-mask rules); rewrite Jcc with constant flags; iterate ≤ 3 sweeps | S,E,M per G4 | fold would raise a trap the input didn't (drop the fold) | O(n_inst) per sweep | −1 inst per fold | stage verifier | fold corpus | Z3 over bounded domains + EXH boundaries |
| Machine copy propagation | MI1–MI10 | def-use chains | Forward copies (mov vA, vB; mov rA, rB): replace uses of the copy's target by its source while the source's location is unchanged | S,E,M | intervening def of either location | O(n_inst) | −1 copy per fwd | post-stage | copy corpus | REVIEW per G4 |
| Machine CSE | MI1–MI10; pure ops only | value hash per block | Deduplicate identical pure instructions (same opcode, same operand values, non-memory) within a block | S,E,M | memory or flag-dependent op | O(n_inst) | −1 inst per dup | post-stage | CSE corpus | REVIEW |
| Redundant move elimination | MI1–MI10 | move chains | Delete mov r,r where both sides identical; collapse A→B→C move chains to A→C where B is dead after | S,E,M | B still used | O(n_inst) | −1 mov per del | post-stage | move corpus | REVIEW |
| Peephole optimization | MI1–MI10 | flag liveness | Closed per-target rule table (Volume 16); representative rules: add 1 → inc when flags dead on x86-64, mov+add → add mem, test+setcc patterns; rules are data, never ad hoc | S,E,M | flags live and rule changes them | O(n_inst × rules), rules ≤ 64 | −1 inst per match | post-stage | peephole corpus per rule | EXH over rule table on 16-bit domains |
| Instruction combining | MI1–MI10 | use chains | Fuse patterns: cmp+Jcc → single flag-form Jcc; mul+add → lea (x86-64) / madd (AArch64); load+op → op-with-mem when addressing fits §15.4 | S,E,M | fused form unavailable for width/feature | O(n_inst) | −1 inst per fuse | post-stage | combine corpus | REVIEW per G4 |
| Branch simplification | MI1–MI10 | CFG; flag state | Remove Jcc whose both targets are the same block; invert Jcc to eliminate an empty block; remove Jmp to the immediately following block (fallthrough-safe: replaced by explicit target fixups) | S,E,M | — | O(n_blocks) | −1 branch per fold | post-stage | branch corpus | REVIEW |
| Register coalescing | pre-RA only | interference graph (§5.8) | Coalesce move-connected live ranges when non-interfering and class-compatible; coalescibility predicate per SPEC-INDEX §5.8; priority by move count | S,E,M | interference exists; ABI-fixed ranges | O(E ∝ n²) worst | −1 mov per coalesce | post-stage | coalesce corpus | Z3 on bounded interference instances + §5.8 laws |
| Machine-level dead-code elimination | MI1–MI10; pre-RA | liveness | Delete instructions whose defs are all dead and which have no flag/memory/stack side effect; iterate to fixpoint ≤ 3 sweeps | S,E,M | flag use downstream; volatile/atomic; call; Unreachable | O(n_inst) per sweep | −1 inst per del | post-stage | DCE corpus | REVIEW per G4 DCE row |
| Instruction scheduling | post-RA, pre-emission | §15.3 latency/throughput/dependency tables | Per block list scheduling: ready set by dependency graph (data, flags, memory via alias sets); order by critical-path priority; never reorder across volatile/atomic or unwind edges | S,E,M; ordering of volatiles/atomics; control flow | dependency violation; latency violated | O(n_inst × ready-set), ready ≤ 16 | hides ≥ 1 cycle per scheduled block | post-scheduling verifier | sched corpus | REVIEW + differential execution |
| Target-specific optimization | MI1–MI10 | per-target tables | Closed per-target pass hooks (Volume 16) with rule tables of the same discipline as Peephole; each hook carries its own 10-row entry in Volume 16 | S,E,M | rule table missing metadata | O(n_inst × rules) | per rule | post-stage | per-target corpus | EXH over rule tables |

## 15.9 Machine IR verification (§22.9)

A verifier runs after every major lowering stage; the four stages and their exact check lists:

**Pre-register-allocation verifier (V-RA-pre).** Checks: (1) operand forms per schema (MI4); (2) vreg def-before-use and liveness consistency (MI1); (3) regclass of every vreg matches its uses' class demands; (4) no physical register operand except Fixed-constraint slots and the ABI roles at Call/Ret (arg/return registers); (5) control flow valid (MI5); (6) call sites: argument placement per Volume 10, clobber sets correct, unwind target well-formed; (7) stack alignment metadata consistent (MI8); (8) memory operands well-formed per §15.4 (MI6 form part); (9) no unlowered abstract operation (MI10); (10) inline-asm operand classes match declared constraints and clobber list is honored (UB-5 gate); (11) all blocks reachable or removed; (12) MIR size limits (§15.1). Termination: single pass over instructions plus one liveness fixpoint bounded by block count; complexity O(n_inst + blocks²) worst case for the fixpoint.

**Post-register-allocation verifier (V-RA-post).** Checks: (1) zero VReg operands remain; (2) every defined value's location is a PReg or Slot on every live path (§15.7 boundary invariants); (3) constraints satisfied per instruction (MI3: tied pairs equal, early-clobber disjoint, fixed regs exact); (4) no two interfering live ranges share a PReg — computed via live intervals per SPEC-INDEX §5.8 ((v, start, end), half-open, interference = overlap and not coalescible); (5) callee-saved registers saved/restored on every return path; (6) special registers per MI9; (7) stack alignment per MI8; (8) memory semantics per MI6; (9) control flow unchanged in structure (MI5). Termination and complexity: O(n_inst × avg live ranges) with interval construction O(n_inst log n_inst).

**Post-scheduling verifier (V-sched-post).** Checks: (1) the instruction multiset (opcode + operands) equals the pre-schedule multiset (order-only change); (2) every data dependence (def before use) preserved; (3) every flag def→use chain preserved; (4) memory dependencies preserved: for aliasing accesses in the same alias set, relative order preserved; volatile/atomic relative order preserved; (5) control flow unchanged; (6) the latency model respected: no consumer scheduled earlier than producer's latency allows. Complexity O(n_inst × deps).

**Pre-emission verifier (V-emit-pre).** Checks: (1) every instruction encodable: an encoding form exists and its fixed fields fit (immediate ranges exact per §15.4/§15.6: x86 disp8 −128..127, imm8 −128..127, imm32 ±2^31; A64 pimm ≤ 4095 scaled, simm9 −256..255, movz/movk 16-bit halves); (2) all labels and block references resolved; (3) frame layout consistent: every SlotRef within the frame size, callee-saved save/restore set matches the used-register set; (4) sp offsets within disp ranges or the scratch pattern present; (5) branch and jump-table relocation ranges pre-checked per SPEC-INDEX §5.9 (final patching is Volume 17); (6) no unlowered abstract operation (MI10, final gate). Complexity O(n_inst).

**Instruction-selection interface contract.** Selection is the lowering stage from IR 2 to MIR and is specified as the interface:

`fn select(f: &Ir2Function, t: &TargetDesc) -> Result<MirFunction, SelError>`

with the exact contract: (C1) coverage — every IR 2 node of f must map to a MIR instruction sequence; the output must contain no IR 2 opcode (MI10 gates this); (C2) semantic preservation — for every IR 2 node n, the selected sequence implements n's operation per the G4 table row "Instruction selection: selected instruction sequence implements the original operation"; (C3) regclass fidelity — every VReg carries the register class implied by its IR 2 representation (ImmScalar width → GPR/FPR, Vector → FPR/VR width, aggregates → the ABI decomposition); (C4) determinism — pattern matching is by longest-match first, ties by lowest pattern id; the pattern tables are target data; (C5) pattern-table completeness — every IR 2 opcode × legal operand-form combination has a pattern (EXH-checked table; a missing entry is a target-description bug caught by OB-15-4, and selection reports `SelError::UnsupportedPattern` naming the node rather than miscompiling); (C6) feature conformance — instructions emitted only under the function's enabled feature set. `SelError` is complete with exactly: `UnsupportedPattern`, `FeatureMissing`, `RegClassExhausted`, `ImpossibleConstraint`, `LimitExceeded`. No panics, no partial outputs: on error, no MIR function is returned.

**Deliverables mapping (line 2605).** Machine IR Specification = §15.1, §15.3–§15.7; Register Model Specification = §15.2; Instruction Description Format = §15.3 (the 14-field schema); Machine Verifier = §15.9 (four stages + selection contract).

**Verifier error enums (complete).** Each stage reports one of a complete enum; no stage panics; the first failing check aborts with the failing instruction or block named (G17 counterexample):

```rust
pub enum VrAPreError {
    OperandForm { inst: InstId, slot: u8 },          // MI4
    VregClassMismatch { vreg: VReg, need: RegClass }, // §15.9 (3)
    StrayPhysicalOperand { inst: InstId, preg: PReg },// §15.9 (4)
    DefBeforeUseViolation { vreg: VReg, block: BlockId }, // MI1
    ControlFlowInvalid { block: BlockId },            // MI5
    CallAbiMismatch { inst: InstId },                 // §15.9 (6)
    StackAlignmentInconsistent { site: InstId },      // MI8 metadata
    MemOperandInvalid { inst: InstId },               // §15.4 ranges/forms
    UnloweredOp { opcode: u32 },                      // MI10
    AsmConstraintViolation { inst: InstId },          // UB-5 gate
    UndefinedLabel { label: LabelId },                // MI2
    UnreachableBlock { block: BlockId },              // §15.9 (11)
    LimitExceeded,                                    // §15.1 limits
}
pub enum VrAPostError {
    VirtualRegisterRemains { vreg: VReg },
    ConstraintViolation { inst: InstId },             // MI3
    InterferenceViolation { a: Location, b: Location },
    CalleeSaveUnrestored { preg: PReg },
    SpecialRegisterViolation { preg: PReg },          // MI9
    AlignmentViolation { site: InstId },              // MI8
    MemoryOrderingViolation { a: InstId, b: InstId }, // MI6
    UnvaluelessPath { vreg: VReg, block: BlockId },   // location-per-path rule
}
pub enum VrSchedError {
    MultisetMismatch { opcode: MOpcode, count: i64 },
    DataDepViolation { a: InstId, b: InstId },
    FlagDepViolation { a: InstId, b: InstId },
    MemoryDepViolation { a: InstId, b: InstId },
    LatencyViolation { consumer: InstId },
}
pub enum VrEmitError {
    Unencodable { inst: InstId, reason: EncReason },  // reason: ImmRange | DispRange | FormMissing | FeatureMissing
    UnresolvedLabel { label: LabelId },
    FrameLayoutInconsistent { slot: SlotId },
    RelocRangeViolation { site: InstId, kind: RelocKind },
    UnloweredOp { opcode: u32 },
}
pub enum SelError { UnsupportedPattern, FeatureMissing, RegClassExhausted, ImpossibleConstraint, LimitExceeded }
```

`RelocKind` and the range formulas are SPEC-INDEX §5.9 (Volume 17 elaborates); the pre-emission check is conservative where the final layout is unknown (§15.11 RG-15-3).

## 15.10 Integration points and MIR pipeline

The MIR pipeline is fixed and order-bound; each stage boundary runs its verifier (G5 discipline applied to lowering):

| Order | Stage | Contract |
| --- | --- | --- |
| 1 | Instruction selection (§15.9 contract) | IR 2 function (post W11/W12) → MIR function with vregs; errors per SelError |
| 2 | V-RA-pre verifier | Must pass before any MIR pass runs |
| 3 | Machine passes (§15.8, order: constant propagation, copy propagation, CSE, combining, peephole, branch simplification, DCE, move elimination) | Each pass re-runs V-RA-pre |
| 4 | Frame layout and prologue/epilogue insertion | Emits stack-slot references and save/restore sets; re-runs V-RA-pre |
| 5 | Register allocation (`planck-regalloc-math`, SPEC-INDEX §5.8) with coalescing (§15.8) | Pre-RA pass order per §15.8; spills into Spill slots |
| 6 | V-RA-post verifier | Must pass before scheduling |
| 7 | Instruction scheduling (§15.8) | Re-runs V-sched-post |
| 8 | Branch finalization and label resolution | Re-runs V-sched-post |
| 9 | V-emit-pre verifier | Must pass before emission |
| 10 | Emission (Volume 17) | Relocations, patching, and object generation per §5.9 |

Integration points with neighboring volumes: Volume 10 (ABI descriptor, inline-asm constraint vocabulary, unsafe boundaries), Volume 14 §14.15 (the W11/W12 boundary this volume's C1 consumes), Volume 16 (per-target MOpcode sets, pattern tables, latency/throughput/scheduling tables, target-specific pass hooks, vtables), Volume 17 (emission, relocations, jump-table data), `planck-regalloc-math` (interference, coloring, spill priority — the pinned §5.8 math is normative here and re-implementation inside the backend is banned, G16).

## 15.11 Known limitations (named refinement gaps)

Per standards.md §54.2.2 these are the only named gaps in this volume; each has a runtime bridge; none is an admitted obligation:

| Gap | Statement | Bridge |
| --- | --- | --- |
| RG-15-1 | The latency/throughput models are static per-target tables (§15.3); MIR does not simulate microarchitectural state (ports, caches, rename width). Cost estimates are as accurate as the tables, nothing more. | Correctness never depends on the tables (only performance); §14.16 uses the same tables, so profitability and cost never disagree; table completeness is a Volume 16 obligation (G15). |
| RG-15-2 | x86-64 partial EFLAGS behavior (`inc`/`adc` partial flag writes) is modeled as whole-register flags. | The peephole and combining rule tables are banned from creating patterns whose correctness depends on partial-flag state; V-sched-post's flag check is conservative (any flag-defining instruction kills the whole CCR live range). |
| RG-15-3 | Pre-emission relocation range checks use worst-case offsets where the final layout is unknown. | The exact check is Volume 17's patch math per §5.9; a V-emit-pre pass plus a Volume 17 failure yields a compile-time error, never a wrong binary. |

## Verification matrix

| Obligation | Property (exact) | Method | Class | Artifact | G-mapping |
| --- | --- | --- | --- | --- | --- |
| OB-15-1 | Register model closure: every PReg operand names a register of the §15.2 per-target sets; reserved registers never allocated | EXH over the PReg enum × role table | FV-0 | `tools/fv/mir_regtable.rs` | G14, G15 |
| OB-15-2 | Constraint satisfaction: every instruction satisfies its constraint table (tied pairs equal, early-clobber disjoint, fixed exact, immediate ranges exact) | Z3 over bounded operand/constraint instances + EXH per-opcode table check | FV-0 | `tools/fv/mir_constraints.smt2` | G4 |
| OB-15-3 | Register allocation soundness: post-RA, no two interfering live ranges share a PReg; every value has one location on every live path; zero vregs remain | Z3 on generated interval sets (n ≤ 24 intervals, 14/28 registers) + EXH small instances; §5.8 laws | FV-0 | `tools/fv/mir_regalloc.smt2` + `planck-regalloc-math` certificate | G4, G13 |
| OB-15-4 | Pattern-table completeness: every IR 2 opcode × operand-form combination has a selection pattern per target | EXH over the opcode × form cross-product | FV-0 | `tools/fv/mir_patterns.rs` | G4, G15 |
| OB-15-5 | Selection correctness: selected sequence implements the original operation (G4 row) on the differential corpus, including overflow and flag behavior | PROP (differential, deterministic PRNG) paired with EXH pattern-table checks | FV-1 (gap: corpus-bounded; bridge: V-RA-pre + V-emit-pre gates every function) | `tools/fv/mir_select_diff.rs` | G4, G5 |
| OB-15-6 | Switch lowering equivalence: the lowered sequence transfers to the arm the source Switch semantics selects, for all case counts 1..64 and density classes | EXH over case-count/density classes 1..64 | FV-0 | `tools/fv/mir_switch.rs` | G4 |
| OB-15-7 | Scheduling legality: post-schedule order preserves data, flag, and alias-set dependencies; volatile/atomic order untouched; instruction multiset unchanged | Z3 on bounded dependency instances + PROP differential | FV-1 (gap: property sampling; bridge: V-sched-post on every function) | `tools/fv/mir_sched.smt2` | G4, G5, G21 |
| OB-15-8 | ABI conformance: call arguments, returns, clobbers, and callee-saved save/restore match Volume 10 on both targets | EXH over the ABI argument/return classification table (Volume 10 classes × both targets) | FV-0 | `tools/fv/mir_abi.rs` | G4 |
| OB-15-9 | Stack alignment: sp 16-aligned at every call boundary and frame sizes multiple of 16 | Z3 frame arithmetic + EXH boundary frames | FV-0 | `tools/fv/mir_align.smt2` | G4 |
| OB-15-10 | Verifier stage completeness: each of the four stages implements its exact check list of §15.9; every check exercised by a negative test | REVIEW with checklist + EXH negative corpus | FV-2 | `spec/volume-15-machine-ir.md` §15.9 + verifier source | G15, G22 |
| OB-15-11 | Determinism: identical IR 2 function + target → byte-identical MIR serialization (selection order, pass order, scheduling ties pinned) | PROP over corpus (byte-hash) + REVIEW of pinned tie-breaks (C4, §15.8 priorities) | FV-1 (gap: sampling; bridge: all tie-break rules pinned in §15.8/§15.9) | `tools/fv/mir_determinism.rs` | G14 |
| OB-15-12 | Coverage: every §22.8 pass has the 10-row template filled; every §22 bullet mapped in the ledger | REVIEW against §15.8 and the ledger | FV-2 | coverage ledger below | G15 |
| OB-15-13 | Refinement gaps RG-15-1..RG-15-3 are the only gaps in this volume and each names its bridge | REVIEW against §15.11 | FV-1 (bridges stated in §15.11) | `spec/volume-15-machine-ir.md` §15.11 | G16, G15 |

## Coverage ledger

| design.md bullet (line, abbreviated) | Defined in |
| --- | --- |
| l2469 Basic blocks | §15.1 |
| l2470 Machine instructions | §15.1 |
| l2471 Virtual registers | §15.1 |
| l2472 Physical registers | §15.1 |
| l2473 Register classes | §15.1 |
| l2474 Stack slots | §15.1 |
| l2475 Machine operands | §15.1 |
| l2476 Control-flow edges | §15.1 |
| l2477 Machine memory operands | §15.1, §15.4 |
| l2478 Labels | §15.1 |
| l2479 Calls | §15.1 |
| l2480 Returns | §15.1 |
| l2481 Branches | §15.1, §15.5 |
| l2482 Terminators | §15.1 |
| l2483 Inline assembly | §15.1 |
| l2484 Target intrinsics | §15.1 |
| l2488 Register identities | §15.2 |
| l2489 Register classes | §15.2 |
| l2490 Register widths | §15.2 |
| l2491 Register aliases | §15.2 |
| l2492 Subregisters | §15.2 |
| l2493 Register pairs | §15.2 |
| l2494 Fixed registers | §15.2 |
| l2495 Reserved registers | §15.2 |
| l2496 Caller-saved registers | §15.2 |
| l2497 Callee-saved registers | §15.2 |
| l2498 Special-purpose registers | §15.2 |
| l2499 Register constraints | §15.2 |
| l2505 Opcode | §15.3 |
| l2506 Operand count | §15.3 |
| l2507 Operand types | §15.3 |
| l2508 Operand constraints | §15.3 |
| l2509 Definitions | §15.3 |
| l2510 Uses | §15.3 |
| l2511 Register effects | §15.3 |
| l2512 Flag effects | §15.3 |
| l2513 Memory effects | §15.3 |
| l2514 Latency model | §15.3 |
| l2515 Throughput model | §15.3 |
| l2516 Encoding requirements | §15.3 |
| l2517 Feature requirements | §15.3 |
| l2518 Scheduling properties | §15.3 |
| l2522 Base registers | §15.4 |
| l2523 Index registers | §15.4 |
| l2524 Scale | §15.4 |
| l2525 Displacement | §15.4 |
| l2526 Addressing modes | §15.4 |
| l2527 Alignment | §15.4 |
| l2528 Memory width | §15.4 |
| l2529 Volatility | §15.4 |
| l2530 Atomic ordering | §15.4 |
| l2531 Address-space information | §15.4 |
| l2532 Alias metadata | §15.4 |
| l2533 Stack references | §15.4 |
| l2534 TLS references | §15.4 |
| l2538 Conditional branches | §15.5 |
| l2539 Unconditional branches | §15.5 |
| l2540 Indirect branches | §15.5 |
| l2541 Calls | §15.5 |
| l2542 Returns | §15.5 |
| l2543 Switch lowering | §15.5 |
| l2544 Jump tables | §15.5 |
| l2545 Branch probabilities | §15.5 |
| l2546 Tail calls | §15.5 |
| l2547 Exception edges | §15.5 |
| l2548 Unreachable instructions | §15.5 |
| l2552 Integer arithmetic | §15.6 |
| l2553 Floating-point arithmetic | §15.6 |
| l2554 Vector arithmetic | §15.6 |
| l2555 Comparisons | §15.6 |
| l2556 Conversions | §15.6 |
| l2557 Bit operations | §15.6 |
| l2558 Shifts | §15.6 |
| l2559 Loads | §15.6 |
| l2560 Stores | §15.6 |
| l2561 Atomic operations | §15.6 |
| l2562 Fences | §15.6 |
| l2563 Calls | §15.6 |
| l2564 Stack operations | §15.6 |
| l2565 Target-specific intrinsics | §15.6 |
| l2569 Virtual registers are correctly defined | §15.7 MI1 |
| l2570 Uses are valid | §15.7 MI2 |
| l2571 Physical register constraints are legal | §15.7 MI3 |
| l2572 Instructions use supported operand forms | §15.7 MI4 |
| l2573 Control flow is valid | §15.7 MI5 |
| l2574 Memory operations preserve required semantics | §15.7 MI6 |
| l2575 Calling conventions are respected | §15.7 MI7 |
| l2576 Stack alignment is correct | §15.7 MI8 |
| l2577 Special registers are handled | §15.7 MI9 |
| l2578 No unlowered abstract operation remains | §15.7 MI10 |
| l2582 Machine constant propagation | §15.8 |
| l2583 Machine copy propagation | §15.8 |
| l2584 Machine CSE | §15.8 |
| l2585 Redundant move elimination | §15.8 |
| l2586 Peephole optimization | §15.8 |
| l2587 Instruction combining | §15.8 |
| l2588 Branch simplification | §15.8 |
| l2589 Register coalescing | §15.8 |
| l2590 Machine-level dead-code elimination | §15.8 |
| l2591 Instruction scheduling | §15.8 |
| l2592 Target-specific optimization | §15.8 |
| l2600 Pre-register-allocation verifier | §15.9 |
| l2601 Post-register-allocation verifier | §15.9 |
| l2602 Post-scheduling verifier | §15.9 |
| l2603 Pre-emission verifier | §15.9 |

Bullets in scope: 105, defined: 105.
