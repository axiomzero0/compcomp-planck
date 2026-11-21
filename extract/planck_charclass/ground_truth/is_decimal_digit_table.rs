// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (generated ground truth) is_decimal_digit_table.rs
// CEP:PROVENANCE: source rocq/Effect/CharClass.v (sha256 7d9f06bd1516875ca5fd546ac05510ee78760bba3c20f962b772acfd4604737b); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for is_decimal_digit over the complete pinned domain (256 entries), as compiled Rust const data consumed by the generated DIF test via include!.
// CEP:WHY: CEP-DR-01 DR-1.5 method DIF requires the oracle to be the kernel itself; DR-8.7 requires every generated artifact under extract/ to be a Rust compilation unit.
// CEP:STATUS: complete (generated)
// CEP:FAILURE: A value edit breaks equivalence and fails the DIF loudly; a format edit fails the cargo build or the planck-extractor check guard.
// CEP:SECURITY: repository-trusted inputs.
// CEP:ASSUMES: kernel vm_compute output (Coq 8.20.1, pinned); entries fit u8 (mechanically chosen from the table maximum).
// CEP:COST: compile-time only (const data in the test binary).
// CEP:EVIDENCE: rocq/Effect/DifGroundTruth.v; DIF artifacts under tools/fv/extract/artifacts/ (sha256 of this file recorded per run).
// CEP:FV-CLASS: FV-2 (generated oracle data; qualified by the DIF it feeds)
// CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain oracle data; no sampling.
// CEP:FV-AXIOMS: none

#[allow(non_upper_case_globals)]
pub const IS_DECIMAL_DIGIT_TABLE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];
