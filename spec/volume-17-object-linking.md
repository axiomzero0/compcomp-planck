<!--
CEP:FILE: spec/volume-17-object-linking.md
CEP:WHY: design.md §24 (lines 2757–2858) states 62 items on object formats, relocations, linker integration, link-time optimization, and executable generation that each need an explicit definition; this volume settles the open policy questions (ELF-only in P1, internal + platform linker, LTO supported) and defines every item normatively.
CEP:WHAT: Normative specification of Planck Volume 17 — object generation and linking: object format contracts (ELF implemented; PE/COFF, Mach-O, WebAssembly declared), the pinned relocation math and its elaboration, symbol table math, linker integration and static linking, link-time optimization, executable generation with the exact program-header set, and layout determinism.
CEP:STATUS: complete
CEP:FAILURE: A design.md §24 bullet without a coverage-ledger row here is an undefined linking behavior; an implementation contradicting a rule here (including the pinned §5.9 relocation math) is non-conforming and must be corrected.
CEP:ASSUMES: SPEC-INDEX.md §5.9 (pinned relocation math, embedded verbatim in §17.2); Volume 16 (emitted relocations, symbols, sections); Volume 18 (runtime entry points, TLS and unwind contracts); Volume 19 (runtime libraries selected at link time); Volume 14 (IR 2 serialization for LTO); Volume 12 (limits); verification standards.md G9, G11, G13, G14, G15, G24, G25.
CEP:COST: document; the pinned relocation checks bound linker work (patch math O(#relocs); archive resolution ≤ Σ member passes; GC linear in the section graph); no runtime cost of its own.
CEP:EVIDENCE: verification matrix ("Verification matrix", OB-17-1 … OB-17-14) and coverage ledger ("Coverage ledger") at the end of this file.
-->

# Volume 17 — Object generation and linking

## 17.0 Scope and traceability

This volume defines design.md §24 "Object generation", lines 2757–2858, subsections 24.1–24.5.

Bullets in scope: 62, distributed as follows.

| design.md subsection | Bullets | Defined in |
| -------------------- | ------- | ---------- |
| 24.1 Object formats | 15 | §17.1 (F1–F15) |
| 24.2 Relocations | 11 | §17.2 (R1–R11) |
| 24.3 Linker integration | 15 | §17.3 (L1–L15) |
| 24.4 Link-time optimization | 11 | §17.4 (LT1–LT11) |
| 24.5 Executable generation | 10 | §17.5 (EX1–EX10) |

Upstream input: emitted code, relocations, symbols, sections, and unwind metadata from Volume 16; runtime archives from Volume 18. Downstream output: ET_REL object files and ET_EXEC executables. The non-bullet sentence of §24.1 ("Potentially add WebAssembly modules as a separate target format") is defined normatively in §17.1 (F4, declared).

## 17.1 Object formats (defines 24.1)

Settled decision: edition P1 implements exactly one object format, ELF, with the three types `ET_REL` (object files), `ET_EXEC` (static executables), and `ET_DYN` (shared objects and PIE executables, produced via the platform linker, L5). F2–F4 are declared: their schema interfaces exist, but requesting PE/COFF, Mach-O, or WebAssembly output in edition P1 is a compile-time rejection E-OBJ-01 naming the supported set (ELF).

F1 ELF. ELF64, little-endian, for both implemented targets. Exact header contract: `e_ident` = `0x7F 'E' 'L' 'F'`, `ELFCLASS64` (2), `ELFDATA2LSB` (1), `EV_CURRENT` (1), `EI_OSABI` = 0 (SYSV); `e_type` ∈ {1 (ET_REL), 2 (ET_EXEC), 3 (ET_DYN)}; `e_machine`: `EM_X86_64` = 62, `EM_AARCH64` = 183; `e_version` = 1; `e_entry` = 0 in ET_REL, `_start` address in ET_EXEC; `e_phoff` = 64 in executables, 0 in ET_REL; `e_shoff` computed; `e_flags` = 0; `e_ehsize` = 64; `e_phentsize` = 56; `e_shentsize` = 64; `e_phnum`, `e_shnum`, `e_shstrndx` computed.

F2 PE/COFF. Declared. The format field schema of F5–F15 (sections, symbols, relocations, alignment, visibility, weak symbols, TLS, debug information, unwind metadata, architecture metadata, versioning, limitations) is the mandatory contract any future PE/COFF implementation must fill; no emission exists in P1; requests are rejected with E-OBJ-01.

F3 Mach-O. Declared. Same contract status as F2.

F4 WebAssembly modules. Declared as a separate target format. Same contract status as F2; a WebAssembly backend would additionally be a target (Volume 16 §16.2) and must define its own selection table there.

Per-format specification, normative for ELF (F5–F15):

F5 Sections. The exact section list produced by the compiler (section index 0 is the NULL section, `SHT_NULL`, all zero). Every row: name, type, flags, entsize, alignment.

| # | Name | Type | Flags | entsize | Align |
| - | ---- | ---- | ----- | ------- | ----- |
| 1 | .text | SHT_PROGBITS | A+X | 0 | 16 |
| 2 | .rodata | SHT_PROGBITS | A | 0 | 8 |
| 3 | .data.rel.ro | SHT_PROGBITS | A+W | 0 | 8 |
| 4 | .data | SHT_PROGBITS | A+W | 0 | 8 |
| 5 | .bss | SHT_NOBITS | A+W | 0 | 8 |
| 6 | .tdata | SHT_PROGBITS | A+W+T | 0 | 8 |
| 7 | .tbss | SHT_NOBITS | A+W+T | 0 | 8 |
| 8 | .init_array | SHT_INIT_ARRAY | A+W | 8 | 8 |
| 9 | .fini_array | SHT_FINI_ARRAY | A+W | 8 | 8 |
| 10 | .rela.text | SHT_RELA | 0 | 24 | 8 |
| 11 | .rela.dyn | SHT_RELA | 0 | 24 | 8 |
| 12 | .symtab | SHT_SYMTAB | 0 | 24 | 8 |
| 13 | .strtab | SHT_STRTAB | 0 | 0 | 1 |
| 14 | .shstrtab | SHT_STRTAB | 0 | 0 | 1 |
| 15 | .eh_frame | SHT_PROGBITS | A | 8 | 8 |
| 16 | .eh_frame_hdr | SHT_PROGBITS | A | 0 | 4 |
| 17 | .planck.lto | SHT_PROGBITS | 0 | 0 | 8 |
| 18 | .note.planck | SHT_NOTE | A | 0 | 4 |
| 19 | .debug_line | SHT_PROGBITS | 0 | 0 | 1 |
| 20 | .debug_info | SHT_PROGBITS | 0 | 0 | 1 |
| 21 | .debug_abbrev | SHT_PROGBITS | 0 | 0 | 1 |
| 22 | .debug_str | SHT_PROGBITS | 0 | 0 | 1 |

Empty sections are omitted except the NULL section; section order in `e_shdr` is exactly the row order above (G14).

F6 Symbols. `Elf64_Sym` (24 bytes): `st_name` (offset into .strtab), `st_info` (bind in the high nibble, type in the low nibble), `st_other` (visibility), `st_shndx`, `st_value` (u64), `st_size` (u64). Symbol table math: `SymId` is a u32 index into `.symtab`. Assignment rule per output object: section symbols first, one per produced section, IDs 1..S in the F5 row order; then LOCAL symbols in creation order (functions in definition order, then statics in definition order, then anonymous literals in ascending literal-pool order); then GLOBAL symbols in creation order. The stable symbol ordering rule (G14) is: wherever symbols are ordered (symtab layout, relocation tie-breaks, GC iteration, `.init_array` ordering), the order is ascending symbol ID. Binds: LOCAL (0), GLOBAL (1), WEAK (2, restricted per F10). Types: NOTYPE (0), OBJECT (1), FUNC (2), TLS (6). Visibilities: DEFAULT (0), HIDDEN (2), PROTECTED (3).

F7 Relocations. `SHT_RELA` records (24 bytes): `r_offset` (u64), `r_info` = (sym << 32) | type, `r_addend` (i64, signed). Sections: `.rela.text` for code, `.rela.dyn` for data/TLS. The internal kind → ELF type mapping is the closed table of §17.2 R7. Relocation records are emitted in ascending `r_offset` order.

F8 Alignment. Section alignments exactly: .text 16, .rodata 8, .data.rel.ro 8, .data 8, .bss 8, .tdata 8, .tbss 8, .init_array 8, .fini_array 8, .rela.* 8, .symtab 8, .strtab/.shstrtab 1, .eh_frame 8, .eh_frame_hdr 4, .planck.lto 8, .note.planck 4, .debug_* 1.

F9 Symbol visibility. Mapping from language visibility: `pub` item → GLOBAL + DEFAULT; non-pub item with its address taken or exported → GLOBAL + HIDDEN when it must survive internalization analysis, otherwise LOCAL; `#[ffi]` declarations (Volume 10) → GLOBAL + DEFAULT; language has no `protected` attribute in P1, PROTECTED is reserved for the runtime's exported compatibility symbols only.

F10 Weak symbols. Restricted: user code must not produce WEAK definitions in P1 (compile-time rejection E-OBJ-02). WEAK is produced only by the runtime shim layer: `planck_allocator_override` and `planck_panic_hook_override` are WEAK fallback symbols resolved by the linker to the strong definitions in the runtime archive when present.

F11 TLS. TLS is carried by `.tdata` (initialized image) and `.tbss` (zero-initialized), a `PT_TLS` program header (§17.5), and the TLS relocation pairs of §17.2 R6. TLS models in P1: local-exec for ET_EXEC, initial-exec for ET_DYN; dynamic TLS (`__tls_get_addr`) is declared, not implemented.

F12 Debug information. DWARF 4 line tables (`.debug_line` + minimal `.debug_info`/`.debug_abbrev`/`.debug_str` CU), exactly as emitted by Volume 16 EM10.

F13 Unwind metadata. `.eh_frame` (CIE/FDE records per Volume 16 FR12) plus `.eh_frame_hdr` for binary search by the runtime: header version 1, `eh_frame_ptr_enc` = 0x1B (sdata4 pcrel), `fde_count_enc` = 0x03 (udata4), `table_enc` = 0x1B (sdata4 pcrel); FDE entries sorted ascending by initial location, count exact.

F14 Architecture metadata. `.note.planck`: `namesz` = 8, name = `"PLANCK\0\0"` (padded to 8), `descsz` = 32, `type` = NT_PLANCK_BUILD (1). Description layout (32 bytes, little-endian): compiler version hash u64, target ID u32 (0 = X86_64, 1 = AArch64), ABI version u32 (§17.5 EX9), edition u8 (= 1 for P1), flags u8, reserved 22 bytes zeroed.

F15 Object versioning. Versioning is exactly the `.note.planck` fields (F14) plus the ELF identity; there are no timestamps, no build IDs, and no environment-derived bytes in the object (G14). ABI version mismatch is a link-time error (L14).

F16 (format-specific limitations, closing the §24.1 field list) Exact limitations of ELF support in P1: ELF64 only (no ELF32); little-endian only; no COMDAT groups beyond the runtime's single group; maximum sections per object = 65536; maximum symbols = 2^31 − 1; maximum relocations per section = 2^32 − 1; `.planck.lto` payload limited by Volume 12 limits.

## 17.2 Relocations (defines 24.2)

The pinned contract of SPEC-INDEX §5.9 is embedded verbatim and is authoritative; the elaboration below must not contradict it.

> Pinned §5.9 (SPEC-INDEX.md), embedded verbatim:
>
> - Relocation entry: `(offset: u64, kind: RelocKind, symbol: SymId, addend: i64)`.
> - Kinds (initial set, exact): `Abs64`, `Abs32`, `PcRel32`, `PcRel16`, `ArmAdrp`, `ArmBranch26`, `X86RipRel32`.
> - Patch math: field extraction by `(offset, width, bit_lower)` with mask math in checked arithmetic; overflow or range violation returns `RelocError` (complete enum); PC-relative range checks are exact (e.g., `PcRel32` requires `−2^31 ≤ target − site ≤ 2^31 − 1` after addend).
> - Endianness: explicit per-target parameter; no host-endian assumptions (Law 7).

Elaboration:

R1 Absolute relocations. `Abs64`: value = `S + A`, written as 64 bits; `S` = final symbol value, `A` = addend; the addition is checked (u64 overflow → `PatchOverflow`). `Abs32`: value = `S + A`, written as 32 bits; exact range check `−2^31 ≤ S + A ≤ 2^31 − 1` (else `FieldOutOfRange`).

R2 Relative relocations. The internal notion "section-relative" is used only by the writer for LOCAL symbols: for a local symbol the writer substitutes `S = section_base + st_value`; the patch math is then identical to the absolute or PC-relative case. There is no separate internal relocation kind for section-relative patching.

R3 PC-relative relocations. `PcRel32`: value = `S + A − P` with `P = site`, exact range `−2^31 ≤ S + A − P ≤ 2^31 − 1`. `PcRel16`: value = `S + A − P`, 16-bit field, exact range `−2^15 ≤ S + A − P ≤ 2^15 − 1`. `X86RipRel32`: value = `S + A − P` with `P = site + 4` (RIP after the instruction), range as `PcRel32`. `ArmBranch26`: value = `(S + A − P) / 4` with `P = site`; requires `(S + A − P) ≡ 0 (mod 4)` (else `BranchRangeExceeded`) and `−2^25 ≤ (S + A − P)/4 ≤ 2^25 − 1`. `ArmAdrp`: value = `(page(S + A) − page(P)) / 4096` with `page(x) = x & !0xFFF`; requires the delta divisible by 4096 by construction and `−2^20 ≤ (page(S+A) − page(P))/4096 ≤ 2^20 − 1` (else `AdrpRangeExceeded`); the 21-bit field is split exactly as `immlo` = bits 30–29, `immhi` = bits 23–5 of the instruction word.

R4 GOT relocations. P1 strategy: GOT indirection is expressed with the pinned kinds against synthesized symbols. The internal linker (or the platform linker via the mapping table) materializes a `.got` slot for symbol x and a synthetic symbol `__got:x` whose value is the slot address. Code loads through the slot: X86_64 uses `X86RipRel32` against `__got:x`; AArch64 uses `ArmAdrp` + `ldr`/`add` against `__got:x`. At the ELF boundary the slot is emitted as `R_X86_64_GLOB_DAT` / `R_AARCH64_GLOB_DAT` in `.rela.dyn` (R7 table).

R5 PLT relocations. Same strategy: synthetic symbol `__plt:x` denotes the PLT stub address; calls to preemptible symbols use `X86RipRel32` / `ArmBranch26` against `__plt:x`; the slot is emitted as `R_X86_64_JUMP_SLOT` / `R_AARCH64_JUMP_SLOT`. PLT stubs are generated only by the platform linker in P1 (the internal linker does not emit PLT or dynamic linking, L5).

R6 TLS relocations. Models: local-exec for ET_EXEC, initial-exec for ET_DYN. X86_64 local-exec: `mov` with `Abs32` against synthetic symbol `__tls_le:x` whose value is the TP-relative offset; emitted at the boundary as `R_X86_64_TPOFF32`. X86_64 initial-exec: `X86RipRel32` against `__got_tls_ie:x`; boundary type `R_X86_64_GOTPC32_TLSDESC` is not used; the IE offset is materialized by the runtime linker into the GOT slot via `R_X86_64_TPOFF64` in `.rela.dyn`. AArch64: `ArmAdrp`+`add` against `__tls_le:x` / `__got_tls_ie:x`, boundary types `R_AARCH64_TLS_TPREL` / `R_AARCH64_TLS_IE_ADD` with `R_AARCH64_TLS_IE_ADR` for the adrp half. The TP-relative offset formula is Volume 18 §18.7 (TLS ABI).

R7 Architecture-specific relocations (ELF boundary mapping). Closed tables; one internal record may expand into ≤ 2 ELF relocations (this is the expansion mechanism referenced by Volume 16 EM6).

| Internal kind | X86_64 ELF type(s) | AArch64 ELF type(s) |
| ------------- | ------------------- | --------------------- |
| Abs64 | R_X86_64_64 (1) | R_AARCH64_ABS64 (257) |
| Abs32 | R_X86_64_32 (10) | R_AARCH64_ABS32 (258) |
| PcRel32 | R_X86_64_PC32 (2) | R_AARCH64_ADR_PREL_LO21 (276) |
| PcRel16 | declared (no P1 producer) | declared |
| ArmAdrp | n/a | R_AARCH64_ADR_PREL_PG_HI21 (275) + R_AARCH64_ADD_ABS_LO12_NC (277) |
| ArmBranch26 | n/a | R_AARCH64_CALL26 (283) |
| X86RipRel32 | R_X86_64_PC32 (2), addend adjusted by −4 at the boundary | n/a |

R8 Relocation overflow. The complete error enum is exactly: `FieldOutOfRange` (patched value outside the field's signed/unsigned range), `PageDeltaUnaligned` (ArmAdrp delta not 4 KiB aligned), `BranchRangeExceeded` (ArmBranch26 out of ±128 MiB or misaligned), `AdrpRangeExceeded` (±4 GiB page range violated), `SymbolUndefined` (resolution failure, also L9), `InvalidOffset` (offset outside the section or inside non-patchable bytes), `UnsupportedKind` (kind not valid for the section or target), `PatchOverflow` (checked arithmetic overflow). Every violation must return the named variant; silent truncation is banned.

R9 Relocation relaxation. Exactly two rules, applied only when the referenced symbol is LOCAL or GLOBAL+HIDDEN and the section is not preemptible: (RX-1) a call/jmp through `__plt:x` is rewritten to a direct call against x (X86_64: `X86RipRel32` to x; AArch64: `ArmBranch26` to x); (RX-2) an AArch64 `adrp`+`add` pair whose target resolves within the same 4 KiB page as the site is rewritten to `adr` (PcRel21) when the addend fits. Rules are applied in ascending offset order, are deterministic, and every relaxation is followed by re-validation (R11).

R10 Symbol resolution. Exact rules: strong definition beats weak; two strong definitions of one name = link error E-LNK-01 naming both objects; multiple weak definitions resolve to the one with the smallest (input_index, member_order, symbol_id) tuple; local symbols never cross module boundaries; an undefined symbol referenced with WEAK binding resolves to 0; an undefined strong reference is E-LNK-02. Resolution is computed from the closed rule set with no tie-breaking heuristics beyond the stated tuple (G14).

R11 Relocation validation. After patching, a validation pass re-reads every patched field from the output bytes, recomputes the expected value from the pinned formulas, and re-checks the range conditions; any mismatch = E-LNK-05. This pass is the runtime bridge for the bounded proofs of the verification matrix.

## 17.3 Linker integration (defines 24.3)

Settled decision (the §24.3 "Decide whether to" bullets): the compiler supports both, exactly as follows.

L1 Invoke platform linkers. Default for hosted executable and shared-object outputs. Invocation contract: the driver invokes the `cc` found in PATH with this exact argv template, in this order: `[cc] [-o out] [object files in ascending input order] [archive files in ascending input order] [-L search path per L8] [-Wl,--eh-frame-hdr] [-Wl,-z,relro,-z,now] [-lpthread] [-lm] [-lc] [-static | -shared]`. The argv construction is deterministic (G14); no environment variables influence it.

L2 Implement an internal linker. The internal linker implements exactly two products in P1: ET_REL → ET_EXEC (static) and ET_REL → ET_REL (combine). It is used when `--linker=internal` is passed, and always for freestanding/declared targets. It does not implement dynamic linking, PLT generation, or symbol versioning in P1 (stated limitation). Its relocation math is the pinned §5.9 math with the same `RelocError` set.

L3 Support both. Both paths are supported and selected by flag; default = platform linker for hosted targets, internal linker for freestanding and declared targets. Both paths must accept the same relocation, symbol, and section inputs from Volume 16.

L4 Static linking. Archive semantics: members are processed in archive order, archives in command-line order; extraction loop: while any undefined symbol is resolvable from an unextracted member of any archive, extract those members; the loop is bounded by the total member count (each pass extracts ≥ 1 member or terminates). Section merging: `.text`, `.rodata`, `.data`, `.data.rel.ro` concatenated in input order; `.bss` merged; `.init_array`/`.fini_array` concatenated; `.eh_frame` merged with CIE deduplication by exact byte equality of the CIE record. COMDAT groups: first group by input order wins, duplicates discarded (single-group scope per F16).

L5 Dynamic linking. P1: consumption of a dynamic libc at platform-linker time is supported (L1 argv); the internal linker produces no ET_DYN and no PLT (stated limitation). Planck-produced shared objects require the platform linker.

L6 Shared libraries. Produced only via the platform linker in P1. Export rule: a symbol is exported iff its visibility is DEFAULT (F9).

L7 Executable linking. ET_EXEC layout per §17.5 (EX-section) with the exact program-header set; static executables via either linker; PIE via the platform linker only.

L8 Library search paths. Ordered list, searched in this order: (1) entries of `--library-path`, left to right; (2) `<sysroot>/lib`; (3) `/lib`; (4) `/usr/lib`. Freestanding targets: only explicit paths. Library names: `-lfoo` resolves to `libfoo.a` then `libfoo.so` in each directory, in that order.

L9 Symbol resolution. As R10; the linker consumes the same rules for archive and object inputs.

L10 Library ordering. Resolution is order-sensitive in the standard way: each archive's symbols resolve only currently-undefined references at the time the archive is processed, and the extraction loop of L4 re-scans archives left-to-right until fixpoint, bounded by the member count. The output is deterministic for a fixed input ordering (G14); changing the input ordering may change resolution by design (documented, POSIX-compatible).

L11 Dead-section elimination. GC over the section graph: roots = `_start`, all DEFAULT-visibility defined symbols in shared outputs, and the contents of `.init_array`; reachability follows relocations at section granularity; unreached sections are dropped. GC is disabled by `--no-gc-sections`. Mark order is ascending section ID; the surviving set is order-independent (it is a fixpoint of reachability), so the output is deterministic (G14). A symbol referenced only from a dropped section is reported as removed in the link map.

L12 Link-time optimization. §17.4; the linker hands `.planck.lto` sections to the LTO pipeline before final layout.

L13 Symbol versioning. P1: the compiler produces no GNU symbol-version records; versioned dynamic symbols consumed from libc resolve to their default version (exact rule: the version index with the highest bit-clear entry in `.gnu.version_r` for the name). Producing version scripts is declared, not implemented.

L14 Runtime library selection. Exact rules: `libplanck_rt.a` (Volume 18) is always linked; `libplanck_net.a` is linked when the `net` feature is used; the system libc is linked dynamically by default (hosted targets); `--static` requires a static libc in the sysroot, else link error E-LNK-03. The runtime archive's ABI version (`.note.planck`, F14) must equal the object ABI version, else link error E-LNK-06.

L15 Cross-linking. `--target` determines machine, endianness (T11 of Volume 16), and sysroot. In P1, cross object generation is supported; cross executables require a target sysroot containing a working linker for the target; absent one, the driver rejects with E-LNK-04 (exact, no implicit host linking).

## 17.4 Link-time optimization (defines 24.4)

Settled decision: LTO is supported in edition P1 in exactly two modes, `--lto=thin` and `--lto=full`; default is off.

LT1 Link-time IR representation. Serialized IR 2 modules (Volume 14 format, version-tagged) stored in the `.planck.lto` section (F5 row 17): payload = u32 format version + module bytes. Version mismatch with the compiler's expected version = LTO-stage failure → LT10 fallback.

LT2 Module summaries. One summary record per defined function, exact fields: `sym_id: u32`, `name_hash: u64`, `effect_row: u16`, `callee_count: u32`, `callee_ids: [u32]`, `inline_cost: u32` (selector cost units of Volume 16 §16.3), `alloc_bytes: u32`, `is_exported: bool`, `features: u32`. Summaries are stored in the same section and must be consistent with the module bytes (checked, E-LTO-01 on mismatch).

LT3 Cross-module analysis. Fixpoint over summaries: propagate two monotone flags — `address_taken` and `preemptible` (join = logical OR) — and effect rows (join = row union per SPEC-INDEX §5.3) along the callee graph until stable. Termination: each flag only grows and rows only grow, so the fixpoint is reached within Σ(modules·symbols) passes (G11).

LT4 Cross-module inlining. Rule: a callee may be inlined at LTO iff its summary `inline_cost ≤ LTO_INLINE_BUDGET = 300` and the callee is not `preemptible` and the caller is not exported from a shared output. Cost accounting uses the same selector cost model as Volume 16 §16.3; the threshold constant is pinned here at 300.

LT5 Cross-module specialization. Monomorphization at LTO time follows the Volume 09 substitution semantics with the total-instance limit of Volume 12 (M-INST-TOTAL); exceeding it = link error E-LTO-02, not silent truncation.

LT6 Symbol visibility analysis. A symbol is internalized (visibility → HIDDEN, enabling RX-1 relaxation and GC) iff no external module's summary references it and the output is not a shared object. Internalization is computed from summaries only, deterministically (G14).

LT7 Whole-program optimization. Exact pass order: (1) LT3 analysis; (2) LT6 internalization; (3) LT4 inlining; (4) LT5 specialization; (5) Volume 13/14 optimization pipeline on the combined IR; (6) Volume 16 backend; (7) final link per §17.3.

LT8 Code elimination. GC per L11 applied after LT7; LTO does not bypass GC.

LT9 Cache behavior. LTO artifacts are cached keyed by `SHA-256(module bytes ‖ compiler version string ‖ target ‖ feature set)`. No wall-clock time, no environment values, no paths enter the key (G14). Cache hits must produce byte-identical outputs to misses for the same key.

LT10 LTO failure recovery. Any LTO-stage failure (format version mismatch, summary inconsistency E-LTO-01, instance limit E-LTO-02) emits diagnostic W-LTO-01 naming the failure, then retries the link in non-LTO mode; the exit code is that of the retry. LTO failures never corrupt outputs.

LT11 Thin versus full LTO. Thin: per-module codegen in parallel, with cross-module decisions computed from summaries only; Full: one combined module through the whole pipeline. Both modes must produce byte-identical output regardless of worker-thread count (obligation OB-17-10); the output must also be byte-identical between thin and full for the same input set (both are pure functions of the input).

## 17.5 Executable generation (defines 24.5)

EX1 Executable entry point. The entry symbol is `_start`, defined in the runtime startup object (Volume 18 §18.2); `e_entry` = the final address of `_start`. Programs must not define `_start` themselves (duplicate definition = link error E-LNK-01).

EX2 Startup code. The startup object `planck_crt.o` from the runtime provides `_start` (per-target assembly contract in Volume 18 §18.2) which sets up arguments and calls `planck_start`; the linker's only obligation is to place `planck_crt.o` first in the link order and satisfy its relocations.

EX3 Runtime initialization. The exact initialization order at process startup: (1) the kernel/libc has placed the TLS image for the main thread (EX5); (2) `planck_rt_init` runs (allocator init, canary from `AT_RANDOM`, std stream handles) — Volume 18 §18.2; (3) `.init_array` entries run in ascending link order (which, by the symbol ordering rule, is ascending symbol ID within each object and object order across objects); (4) `main(argc, argv)` is called; (5) `planck_exit` runs the shutdown path (EX7).

EX4 Static initialization. Language statics requiring non-constant initialization emit `.init_array` entries; their order is the stable symbol ordering rule = ascending symbol ID (G14). Constant-initialized statics are in `.data`/`.rodata` with no init entry.

EX5 TLS initialization. `.tdata` is the TLS initialization image; `.tbss` contributes size only. The kernel/libc copies the image and zeroes the remainder for the main thread; TLS relocations (R6) resolve to local-exec or initial-exec offsets computed by the linker per the Volume 18 §18.7 formula.

EX6 Runtime linking. For dynamically linked executables the exact `DT_NEEDED` list is `libc.so.6` only (hosted default); additional shared libraries enter via user `#[ffi]` declarations and platform-linker flags. The internal linker path produces no `DT_NEEDED` entries (static only).

EX7 Process termination. Normal termination: `main` returns → `planck_exit(code)` runs exit handlers in reverse registration order, flushes the standard streams (Volume 19 I5 contract), then invokes `exit_group(2)` with the final exit code. Abnormal termination paths (trap, abort, double panic) are Volume 18 contracts; they end in the same `exit_group(2)` call with code 134.

EX8 Exit codes. Exact mapping: success = 0; `main`'s i32 return value → exit code = `main_return & 0xFF` (the lowest 8 bits, POSIX semantics); trap/abort/panic-with-abort/double panic = 134; runtime initialization failure = 126. These are the only exit codes produced by the executable contract itself; compiler driver exit codes are Volume 20/12 concerns.

EX9 Program metadata. The exact metadata set: `.note.planck` (F14) carrying the build record; `.eh_frame_hdr` (F13); no GNU build-id, no timestamps, no comment sections beyond the fixed list (G14 determinism).

EX10 Platform-specific entry requirements. X86_64 Linux at `_start`: `RSP` points to `argc` (u64); stack layout `[argc][argv… NULL][envp… NULL][auxv pairs]`; kernel guarantees RSP ≡ 0 (mod 16). AArch64 Linux at `_start`: `SP` points to `argc` with the same layout; kernel guarantees SP ≡ 0 (mod 16); X29/X30 contents are undefined and `_start` must not rely on them. Register/stack setup performed by `_start` itself (zeroing the frame pointer, aligning the stack) is specified in Volume 18 §18.2.

Program headers, exact set (settled contract for ET_EXEC produced by either linker). p_align = 0x1000 for all PT_LOAD; every PT_LOAD satisfies `p_offset ≡ p_vaddr (mod 0x1000)`.

| Order | Type | Contents | Flags | Condition |
| ----- | ---- | -------- | ----- | --------- |
| 1 | PT_PHDR | the program header table | R | always |
| 2 | PT_LOAD | .text (+ AArch64 literal pools) | R+X | always |
| 3 | PT_LOAD | .rodata, .eh_frame, .eh_frame_hdr | R | always |
| 4 | PT_TLS | .tdata, .tbss | R | when TLS is used |
| 5 | PT_LOAD | .data.rel.ro, .data, .bss | R+W | always |
| 6 | PT_GNU_EH_FRAME | pointer to .eh_frame_hdr | R | always |
| 7 | PT_GNU_STACK | (empty) | R+W, non-executable | always |

Without TLS the PT_TLS row is omitted (6 headers); with TLS the table is exactly 7 headers in the row order above. ET_DYN outputs via the platform linker follow the platform linker's own header policy (out of P1 scope).

## Verification matrix

| ID | Property (exact) | Method | Class | Artifact | G-mapping |
| -- | ---------------- | ------ | ----- | -------- | --------- |
| OB-17-1 | Relocation patch math: for every kind, patch(offset, width, bit_lower, value, endianness) writes exactly the field bits specified and leaves all other bits unchanged | Z3 (bounded: all 7 kinds × widths {1,2,4,8} × offsets {0, 1, width−1} × 4-byte buffers) + EXH (full domain: 7 kinds × all (width, bit_lower) pairs in the closed table × values at range corners, little and big endian) | FV-0 (enumerated table domain) + FV-1 beyond | tools/fv/ob17-1-patch.smt2 | G9, G24 |
| OB-17-2 | PC-relative range checks are exact: PcRel32 rejects S+A−P outside [−2^31, 2^31−1]; PcRel16 outside [−2^15, 2^15−1]; ArmBranch26 outside ±2^25 units; ArmAdrp outside ±2^20 pages or non-4KiB delta | Z3 (boundaries ±1 of each range) + EXH (all boundary values per kind) | FV-0 | tools/fv/ob17-2-ranges.smt2 | G9, G24 |
| OB-17-3 | `RelocError` is complete: every violation path of R1–R9 returns exactly one named variant, and every variant is reachable by some input in the bounded domain | EXH (violation-path table × generated inputs) | FV-0 | tools/fv/ob17-3-errors | G15, G17 |
| OB-17-4 | ELF structural conformance: every object produced by the compiler corpus parses as ELF64 with the exact F1 header fields and the F5 section list/order | EXH (full test corpus) + REVIEW (spec-conformance argument per field) | FV-1 (corpus-bounded) | tools/fv/ob17-4-elf | G9, G24 |
| OB-17-5 | Symbol ordering determinism: two compilations of identical inputs produce byte-identical objects (symbol IDs, section order, relocation order all ascending per F6) | EXH (module sets ≤ 4 modules × 6 symbols, all orderings) + PROP (paired, deterministic PRNG) | FV-1 | tools/fv/ob17-5-determinism | G14, G24 |
| OB-17-6 | Resolution rules are total and deterministic: every (strong, weak, undefined) combination resolves or errors per R10 | EXH (3×3 conflict matrix × tie cases) | FV-0 | tools/fv/ob17-6-resolve | G24 |
| OB-17-7 | Archive extraction loop terminates within Σ members passes (each pass extracts ≥ 1 member or stops) | REVIEW (invariant: extracted-set grows monotonically; finite member set) | FV-2 | review/ob17-7-archives.md | G11, G24 |
| OB-17-8 | GC drops no reachable section and terminates: the surviving set is the exact fixpoint of reachability from the roots | Z3 (section graphs ≤ 12 nodes, ≤ 2 roots) | FV-1 | tools/fv/ob17-8-gc.smt2 | G24 |
| OB-17-9 | Executable program headers match the exact §17.5 table for both machines, with and without TLS (4 instances) | EXH (4 instances) | FV-0 | tools/fv/ob17-9-phdr | G9, G24 |
| OB-17-10 | LTO outputs are byte-identical across worker-thread counts and between thin and full modes for the same input set | PROP (paired with OB-17-5 EXH; deterministic PRNG over module sets ≤ 4) | FV-1 | tools/fv/ob17-10-lto | G14, G24 |
| OB-17-11 | TLS offset formula (Volume 18 §18.7): computed tpoff values lie in the valid IE/LE range for ≤ 8 TLS symbols with sizes ≤ 2^12 | Z3 (bounded instance set) | FV-1 | tools/fv/ob17-11-tls.smt2 | G9, G24 |
| OB-17-12 | Exit-code mapping: exit = main_return & 0xFF; trap/abort = 134; runtime-init failure = 126; success = 0 | EXH (i32 corners: 0, 1, 255, 256, 2^31−1, −1, −2^31) | FV-0 | tools/fv/ob17-12-exit | G24 |
| OB-17-13 | Endianness parameter is honored: all field writers/readers use the declared endianness; flipping the parameter flips every multi-byte field consistently | EXH (test vectors per field width {1,2,4,8,16,21-bit-pair,26}) | FV-0 | tools/fv/ob17-13-endian | G9, G24 |
| OB-17-14 | Relaxation rules RX-1/RX-2 apply only to LOCAL/HIDDEN symbols on non-preemptible sections and preserve semantics of the patched instruction pair | Z3 (bounded instance set: ≤ 8 relocations) + REVIEW (semantic-preservation argument per rule) | FV-1 | tools/fv/ob17-14-relax.smt2 | G9, G24 |

## Coverage ledger

Bullets in scope: 62, defined: 62.

| design.md bullet (§) | Abbreviated bullet | Defined in |
| -------------------- | ------------------ | ---------- |
| 24.1 | ELF | §17.1 F1 |
| 24.1 | PE/COFF | §17.1 F2 |
| 24.1 | Mach-O | §17.1 F3 |
| 24.1 | Sections | §17.1 F5 |
| 24.1 | Symbols | §17.1 F6 |
| 24.1 | Relocations | §17.1 F7 |
| 24.1 | Alignment | §17.1 F8 |
| 24.1 | Symbol visibility | §17.1 F9 |
| 24.1 | Weak symbols | §17.1 F10 |
| 24.1 | TLS | §17.1 F11 |
| 24.1 | Debug information | §17.1 F12 |
| 24.1 | Unwind metadata | §17.1 F13 |
| 24.1 | Architecture metadata | §17.1 F14 |
| 24.1 | Object versioning | §17.1 F15 |
| 24.1 | Format-specific limitations | §17.1 F16 |
| 24.2 | Absolute relocations | §17.2 R1 |
| 24.2 | Relative relocations | §17.2 R2 |
| 24.2 | PC-relative relocations | §17.2 R3 |
| 24.2 | GOT relocations | §17.2 R4 |
| 24.2 | PLT relocations | §17.2 R5 |
| 24.2 | TLS relocations | §17.2 R6 |
| 24.2 | Architecture-specific relocations | §17.2 R7 |
| 24.2 | Relocation overflow | §17.2 R8 |
| 24.2 | Relocation relaxation | §17.2 R9 |
| 24.2 | Symbol resolution | §17.2 R10 |
| 24.2 | Relocation validation | §17.2 R11 |
| 24.3 | Invoke platform linkers | §17.3 L1 |
| 24.3 | Implement an internal linker | §17.3 L2 |
| 24.3 | Support both | §17.3 L3 |
| 24.3 | Static linking | §17.3 L4 |
| 24.3 | Dynamic linking | §17.3 L5 |
| 24.3 | Shared libraries | §17.3 L6 |
| 24.3 | Executable linking | §17.3 L7 |
| 24.3 | Library search paths | §17.3 L8 |
| 24.3 | Symbol resolution | §17.3 L9 |
| 24.3 | Library ordering | §17.3 L10 |
| 24.3 | Dead-section elimination | §17.3 L11 |
| 24.3 | Link-time optimization | §17.3 L12 |
| 24.3 | Symbol versioning | §17.3 L13 |
| 24.3 | Runtime library selection | §17.3 L14 |
| 24.3 | Cross-linking | §17.3 L15 |
| 24.4 | Link-time IR representation | §17.4 LT1 |
| 24.4 | Module summaries | §17.4 LT2 |
| 24.4 | Cross-module analysis | §17.4 LT3 |
| 24.4 | Cross-module inlining | §17.4 LT4 |
| 24.4 | Cross-module specialization | §17.4 LT5 |
| 24.4 | Symbol visibility analysis | §17.4 LT6 |
| 24.4 | Whole-program optimization | §17.4 LT7 |
| 24.4 | Code elimination | §17.4 LT8 |
| 24.4 | Cache behavior | §17.4 LT9 |
| 24.4 | LTO failure recovery | §17.4 LT10 |
| 24.4 | Thin versus full LTO | §17.4 LT11 |
| 24.5 | Executable entry point | §17.5 EX1 |
| 24.5 | Startup code | §17.5 EX2 |
| 24.5 | Runtime initialization | §17.5 EX3 |
| 24.5 | Static initialization | §17.5 EX4 |
| 24.5 | TLS initialization | §17.5 EX5 |
| 24.5 | Runtime linking | §17.5 EX6 |
| 24.5 | Process termination | §17.5 EX7 |
| 24.5 | Exit codes | §17.5 EX8 |
| 24.5 | Program metadata | §17.5 EX9 |
| 24.5 | Platform-specific entry requirements | §17.5 EX10 |
