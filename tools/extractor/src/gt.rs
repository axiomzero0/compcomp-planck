// CEP:FILE: tools/extractor/src/gt.rs
// CEP:WHAT: Generation of rocq/Effect/DifGroundTruth.v — the kernel oracle: the full 2^W row enumeration as Gallina literals, the pinned index encoding (bit W-1-k = field k), and per-function index tables evaluated by kernel vm_compute over the complete 2^W x 2^W domain.
// CEP:WHY: The DIF ground truth must come from the Coq kernel itself (vm_compute is kernel reduction), not from any hand-translated expectation; the enumeration and encoding are emitted by the same extractor that emits the Rust test, so both sides share one pinned convention.
// CEP:STATUS: complete
// CEP:FAILURE: A wrong enumeration or weight would mismatch the Rust side and fail the DIF; both sides are generated from the same parsed width and field order.
// CEP:ASSUMES: the parsed row type (width W, all-bool tuple, canonical field order); functions with signature (row,row) -> row and (row,row) -> bool in the unit; Coq 8.20.1 list printing via Eval vm_compute.
// CEP:COST: kernel evaluation time (vm_compute over 65,536 pairs per function).
// CEP:EVIDENCE: rocq/Effect/DifGroundTruth.v (generated); captured stdout emitted as generated Rust source extract/planck_effect/ground_truth/*_table.rs (DR-8.7: extraction deliverables are Rust compilation units, never plain-text dumps).
// CEP:SECURITY: repository-trusted inputs; emitted tables carry source sha256 provenance.
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)
// CEP:FV-SCRIPT: tools/extractor (extractor ground-truth; extractor dif)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: the oracle covers the complete finite input domain; no sampling.
// CEP:FV-AXIOMS: none

use crate::parse::{ParseUnit, Ty};
use std::path::Path;

fn gallina_row(i: usize, width: usize) -> String {
    // field k = bit (width-1-k) of i
    let mut parts: Vec<String> = Vec::new();
    for k in 0..width {
        let bit = (i >> (width - 1 - k)) & 1;
        parts.push(if bit == 1 {
            String::from("true")
        } else {
            String::from("false")
        });
    }
    // right-nested: b0, (b1, ( ... (b_{w-2}, b_{w-1}) ... ))
    let mut expr = parts[width - 1].clone();
    for k in (0..width - 1).rev() {
        expr = format!("{}, ({})", parts[k], expr);
    }
    format!("({})", expr)
}

fn weights(width: usize) -> Vec<usize> {
    (0..width).map(|k| 1usize << (width - 1 - k)).collect()
}

/// Mechanically pick the functions to tabulate from their parsed signature:
/// two row-typed binders; row-typed result -> index table; bool result -> b2n
/// table. Returns (row_fns, bool_fns) in definition order. Shared by the .v
/// oracle generator, the Rust test emitter, and the check guard so all three
/// derive the table set from one parsed source of truth.
pub fn tabulated_functions(u: &ParseUnit) -> (Vec<String>, Vec<String>) {
    let (ty_name, _width) = u.row_type.clone().expect("row type alias");
    let mut row_fns: Vec<String> = Vec::new();
    let mut bool_fns: Vec<String> = Vec::new();
    for d in &u.definitions {
        if d.ty_alias.is_some() || d.binders.len() != 2 {
            continue;
        }
        let both_row = d
            .binders
            .iter()
            .all(|(_, t)| matches!(t, Ty::Named(n) if *n == ty_name));
        if !both_row {
            continue;
        }
        match &d.ret {
            Ty::Named(n) if *n == ty_name => row_fns.push(d.name.clone()),
            Ty::Bool => bool_fns.push(d.name.clone()),
            _ => {}
        }
    }
    if row_fns.is_empty() && bool_fns.is_empty() {
        eprintln!("ERROR: no (row,row)-arity functions found to tabulate");
        std::process::exit(1);
    }
    (row_fns, bool_fns)
}

pub fn ground_truth_v(units: &[ParseUnit]) -> String {
    let u = &units[0];
    let (ty_name, width) = u.row_type.clone().expect("row type alias");
    let (row_fns, bool_fns) = tabulated_functions(u);

    let ws = weights(width);
    // Build the row_index definition text from fst/snd chains (mechanical):
    let mut field_exprs: Vec<String> = Vec::new();
    for k in 0..width {
        let mut expr = String::from("r");
        for _ in 0..k {
            expr = format!("snd ({})", expr);
        }
        if k < width - 1 {
            expr = format!("fst ({})", expr);
        }
        field_exprs.push(expr);
    }
    let mut row_index_def = String::from("Definition row_index (r : ");
    row_index_def.push_str(&ty_name);
    row_index_def.push_str(") : nat := ");
    for (k, w) in ws.iter().enumerate() {
        if k == width - 1 {
            row_index_def.push_str(&format!("{} * b2n ({})", w, field_exprs[k]));
        } else {
            row_index_def.push_str(&format!("{} * b2n ({}) + ", w, field_exprs[k]));
        }
    }
    row_index_def.push_str(".");

    let mut all_rows = String::from("Definition all_rows : list ");
    all_rows.push_str(&ty_name);
    all_rows.push_str(" :=\n  [ ");
    let n = 1usize << width;
    for i in 0..n {
        if i > 0 {
            all_rows.push_str("; ");
            if i % 4 == 0 {
                all_rows.push_str("\n    ");
            }
        }
        all_rows.push_str(&gallina_row(i, width));
    }
    all_rows.push_str(" ].");

    let mut out = String::new();
    out.push_str("(* GENERATED FILE - DO NOT EDIT. Generated by planck-extractor.\n");
    out.push_str(&format!("   Kernel oracle for DIF: the complete 2^{} row domain, the pinned\n", width));
    out.push_str("   index encoding (bit W-1-k = field k), and per-function index tables\n");
    out.push_str("   evaluated by kernel vm_compute. Hand edits are non-conforming.\n");
    out.push_str("   CEP:FILE: rocq/Effect/DifGroundTruth.v\n");
    out.push_str("   CEP:PROVENANCE: extractor planck-extractor; generated mechanically from the parsed row domain — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2).\n");
    out.push_str("   CEP:WHAT: Kernel ground-truth tables for the DIF differential equivalence method: full row enumeration + pinned index encoding + vm_compute tables over the complete domain.\n");
    out.push_str("   CEP:WHY: The DIF oracle must be the Coq kernel itself; this file is generated by the same extractor that emits the Rust test, so both sides share one pinned enumeration and encoding.\n");
    out.push_str("   CEP:STATUS: complete (generated)\n");
    out.push_str("   CEP:FAILURE: Any mismatch between this file's encoding and the Rust test's encoding fails the DIF loudly; regeneration is the only edit path.\n");
    out.push_str("   CEP:ASSUMES: Coq 8.20.1; the source unit's row type and (row,row)-arity functions.\n");
    out.push_str("   CEP:COST: kernel vm_compute over 65,536 pairs per function.\n");
    out.push_str("   CEP:EVIDENCE: the Eval vm_compute outputs captured by planck-extractor dif into extract/planck_effect/ground_truth/.\n");
    out.push_str("   CEP:SECURITY: no secrets; repository-trusted inputs; generated mechanically, hand edits are non-conforming.\n");
    out.push_str("   CEP:FV-CLASS: FV-2\n");
    out.push_str("   CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)\n");
    out.push_str("   CEP:FV-SCRIPT: tools/extractor (extractor dif)\n");
    out.push_str("   CEP:FV-SORRY-COUNT: N/A\n");
    out.push_str("   CEP:FV-REFINEMENT: complete-domain oracle; no sampling.\n");
    out.push_str("   CEP:FV-AXIOMS: none *)\n\n");
    out.push_str("Require Import Effect.RowAlgebra.\nRequire Import List.\nImport ListNotations.\n\n");
    out.push_str("Definition b2n (b : bool) : nat := if b then 1 else 0.\n\n");
    out.push_str(&row_index_def);
    out.push_str("\n\n");
    out.push_str(&all_rows);
    out.push_str("\n\n");

    // The full table for each function is DEFINED for documentation (kernel
    // type-checked), but EVALUATED in chunks: one vm_compute per fixed r1
    // over all r2. Chunking keeps kernel memory bounded (unary nats at full
    // table size were OOM-killed in the first pinned run).
    for f in &row_fns {
        out.push_str(&format!(
            "Definition {}_index_table : list nat :=\n  flat_map (fun r1 => map (fun r2 => row_index ({} r1 r2)) all_rows) all_rows.\n\n",
            f, f
        ));
    }
    for f in &bool_fns {
        out.push_str(&format!(
            "Definition {}_index_table : list nat :=\n  flat_map (fun r1 => map (fun r2 => b2n ({} r1 r2)) all_rows) all_rows.\n\n",
            f, f
        ));
    }

    // Sanity: enumeration size.
    out.push_str("Eval vm_compute in (length all_rows).\n");

    // Chunked value dumps. Order pinned: row functions first, then bool
    // functions; within a function, r1 in all_rows order (index 0..2^W-1).
    let n = 1usize << width;
    for i in 0..n {
        let r1 = gallina_row(i, width);
        for f in &row_fns {
            out.push_str(&format!(
                "Eval vm_compute in (map (fun r2 => row_index ({} ({}) r2)) all_rows).\n",
                f, r1
            ));
        }
        for f in &bool_fns {
            out.push_str(&format!(
                "Eval vm_compute in (map (fun r2 => b2n ({} ({}) r2)) all_rows).\n",
                f, r1
            ));
        }
    }
    out
}

/// Generated Rust source for one kernel ground-truth table (DR-8.7): the
/// extraction deliverable form is a Rust compilation unit — a `pub const`
/// array included verbatim by the generated DIF test — never a plain-text
/// dump. The entry type is chosen mechanically from the table maximum.
pub fn ground_truth_rs(
    fn_name: &str,
    vals: &[usize],
    source_rel: &str,
    source_sha: &str,
    width: usize,
) -> String {
    let n = 1usize << width;
    let total = n * n;
    assert_eq!(vals.len(), total, "ground truth table size");
    let max = vals.iter().copied().max().unwrap_or(0);
    let ty = if max <= 255 {
        "u8"
    } else if max <= 65535 {
        "u16"
    } else {
        eprintln!(
            "ERROR: table value {} exceeds the pinned u16 bound (widen the entry type rule in gt.rs)",
            max
        );
        std::process::exit(1);
    };
    let const_name = format!("{}_TABLE", fn_name.to_uppercase());
    let fname = format!("{}_table.rs", fn_name);
    let mut body = String::new();
    for (i, v) in vals.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
            if i % 16 == 0 {
                body.push_str("\n    ");
            }
        }
        body.push_str(&v.to_string());
    }
    format!(
        r"// GENERATED FILE - DO NOT EDIT.
//
// Generated mechanically by planck-extractor (dif) from the Coq kernel's
// vm_compute output over the complete 2^{w} x 2^{w} domain. This is the
// extraction deliverable form pinned by CEP-DR-01 §8 DR-8.7: kernel oracle
// data is a compiled Rust source included by the DIF test harness — a
// plain-text dump (.txt) is non-conforming as extraction output.
// Hand edits are non-conforming: any value change fails the DIF loudly;
// any format tamper fails cargo (include!) or `planck-extractor check`.
// CEP:FILE: extract/planck_effect/ground_truth/{fname}
// CEP:PROVENANCE: source {src} (sha256 {sha}); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2/DR-8.7).
// CEP:WHAT: Kernel ground-truth table for {fnn} over the complete 2^{w} x 2^{w} domain (65,536 pairs), as compiled Rust const data consumed by the generated DIF test via include!.
// CEP:WHY: CEP-DR-01 DR-1.5 method DIF requires the oracle to be the kernel itself; DR-8.7 requires every generated artifact under extract/ to be a Rust compilation unit — the oracle data therefore compiles into the test harness instead of being parsed from text at test time.
// CEP:STATUS: complete (generated)
// CEP:FAILURE: A value edit breaks equivalence and fails the DIF loudly; a format edit fails the cargo build or the planck-extractor check guard.
// CEP:ASSUMES: kernel vm_compute output (Coq 8.20.1, pinned); index encoding bit W-1-k = field k; entries fit {ty} (mechanically chosen from the table maximum).
// CEP:COST: compile-time only (const data in the test binary).
// CEP:EVIDENCE: rocq/Effect/DifGroundTruth.v; DIF artifacts under tools/fv/extract/artifacts/ (sha256 of this file recorded per run).
// CEP:SECURITY: repository-trusted inputs.
// CEP:FV-CLASS: FV-2 (generated oracle data; qualified by the DIF it feeds)
// CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain oracle data; no sampling.
// CEP:FV-AXIOMS: none

#[allow(non_upper_case_globals)]
pub const {const_name}: [{ty}; {total}] = [
    {body}
];
",
        w = width,
        fname = fname,
        src = source_rel,
        sha = source_sha,
        fnn = fn_name,
        ty = ty,
        total = total,
        const_name = const_name,
        body = body
    )
}

pub fn emit_ground_truth(root: &Path, units: &[ParseUnit]) {
    let text = ground_truth_v(units);
    let p = root.join("rocq").join("Effect").join("DifGroundTruth.v");
    std::fs::write(&p, text).expect("write DifGroundTruth.v");
}
