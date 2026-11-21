// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete closed domains (CEP-DR-01 §8 DR-8.7:
// extraction deliverables are Rust compilation units, never plain-text
// dumps). Hand edits are non-conforming (DR-8.2).
// CEP:FILE: extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically.
// CEP:WHAT: Kernel ground-truth tables (opid_index, precedence, assoc, arity, arity_count, overloadable, overflow) over the complete closed domains, as compiled Rust const data consumed by the generated DIF test via include!.
// CEP:WHY: The DIF oracle is the kernel itself; DR-8.7 requires generated artifacts under extract/ to be Rust compilation units.
// CEP:STATUS: complete (generated)
// CEP:FAILURE: A value edit breaks equivalence and fails the DIF loudly; a format edit fails cargo or planck-extractor check.
// CEP:SECURITY: repository-trusted inputs.
// CEP:ASSUMES: kernel vm_compute output (Coq 8.20.1, pinned).
// CEP:COST: compile-time only.
// CEP:EVIDENCE: rocq/Effect/DifGroundTruth.v; DIF artifacts under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-2 (generated oracle data; qualified by the DIF it feeds)
// CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete closed-domain oracle data; no sampling.
// CEP:FV-AXIOMS: none

// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for opid_index over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const OPID_INDEX_TABLE: [u8; 44] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 
    33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for precedence over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const PRECEDENCE_TABLE: [u8; 44] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 
    2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 8, 9, 9, 10, 10, 11, 
    11, 11, 12, 13, 13, 13, 13, 14, 14, 14, 14, 14
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for assoc over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const ASSOC_TABLE: [u8; 44] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 
    0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for arity over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const ARITY_TABLE: [u8; 44] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 0, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for arity_count over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const ARITY_COUNT_TABLE: [u8; 44] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 0, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for overloadable over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const OVERLOADABLE_TABLE: [u8; 44] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0
];


// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete domain. This is the extraction
// deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle data is a
// compiled Rust source included by the DIF test harness — a plain-text dump
// (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly; any
// format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/op_columns.rs
// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 7149fa7f2228903cc25f8e1837a062b62067ef098dbca3f41d7289c0d7345bb9); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for overflow over the complete pinned domain (44 entries), as compiled Rust const data consumed by the generated DIF test via include!.
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
pub const OVERFLOW_TABLE: [u8; 44] = [
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 
    0, 0, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3
];
