// CEP:FILE: tools/extractor/src/emit.rs
// CEP:WHAT: Rust emission from parsed Gallina-subset units: type aliases, pinned prelude functions (orb/andb/implb), fst/snd as projection chains, applications as calls, right-nested pairs as Rust tuples; writes the separated extract/ tree (lib.rs, dif test, Cargo.toml) with CEP:PROVENANCE headers.
// CEP:WHY: DR-1.4 extraction layer; the emitted Rust is the compiler's code and must carry provenance (source .v, extractor revision, source sha256) and the hand-edit ban in its header.
// CEP:STATUS: complete
// CEP:FAILURE: An emission bug surfaces as a DIF mismatch (kernel ground truth vs extracted Rust over the complete domain) or a cargo failure; both fail the build.
// CEP:ASSUMES: the pinned term/type grammar of parse.rs; Rust 2021 tuples and fns.
// CEP:COST: build-time only.
// CEP:EVIDENCE: extract/planck_effect (generated); tools/fv/extract/artifacts (DIF records).
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (cargo test; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: one Gallina Definition maps to exactly one Rust item; names are preserved verbatim (snake_case kept, non_camel_case_types allowed) for provenance traceability.
// CEP:FV-AXIOMS: none

use crate::parse::{Definition, ParseUnit, Term, Ty};
use std::path::Path;

pub fn ty_to_rust(ty: &Ty) -> String {
    match ty {
        Ty::Bool => String::from("bool"),
        Ty::Named(n) => n.clone(),
        Ty::Tuple(comps) => {
            let inner: Vec<String> = comps.iter().map(ty_to_rust).collect();
            format!("({})", inner.join(", "))
        }
    }
}

/// Emit a term. fst/snd applications become projection chains; everything
/// else becomes a function call or tuple literal with identical shape.
pub fn term_to_rust(t: &Term) -> String {
    match t {
        Term::BoolLit(b) => {
            if *b {
                String::from("true")
            } else {
                String::from("false")
            }
        }
        Term::Ident(n) => n.clone(),
        Term::Pair(comps) => {
            let inner: Vec<String> = comps.iter().map(|c| term_to_rust(c)).collect();
            format!("({})", inner.join(", "))
        }
        Term::App(head, args) => match &**head {
            Term::Ident(n) if n == "fst" && args.len() == 1 => {
                format!("({}).0", term_to_rust(&args[0]))
            }
            Term::Ident(n) if n == "snd" && args.len() == 1 => {
                format!("({}).1", term_to_rust(&args[0]))
            }
            _ => {
                if args.is_empty() {
                    term_to_rust(head)
                } else {
                    let a: Vec<String> = args.iter().map(|x| term_to_rust(x)).collect();
                    format!("{}({})", term_to_rust(head), a.join(", "))
                }
            }
        },
    }
}

pub fn def_to_rust(d: &Definition) -> String {
    if let Some(ty) = &d.ty_alias {
        return format!(
            "#[allow(non_camel_case_types)]\npub type {} = {};",
            d.name,
            ty_to_rust(ty)
        );
    }
    if d.binders.is_empty() {
        return format!(
            "#[allow(non_upper_case_globals)]\npub const {}: {} = {};",
            d.name,
            ty_to_rust(&d.ret),
            term_to_rust(&d.body)
        );
    }
    let params: Vec<String> = d
        .binders
        .iter()
        .map(|(n, t)| format!("{}: {}", n, ty_to_rust(t)))
        .collect();
    format!(
        "#[inline]\npub fn {}({}) -> {} {{\n    {}\n}}",
        d.name,
        params.join(", "),
        ty_to_rust(&d.ret),
        term_to_rust(&d.body)
    )
}

pub const PRELUDE: &str = r#"#[inline]
pub fn orb(x: bool, y: bool) -> bool {
    x | y
}

#[inline]
pub fn andb(x: bool, y: bool) -> bool {
    x & y
}

#[inline]
pub fn implb(x: bool, y: bool) -> bool {
    if x {
        y
    } else {
        true
    }
}
"#;

fn sha256_hex(data: &[u8]) -> String {
    crate::dif::sha256_hex(data)
}

pub fn lib_rs_header(source_rel: &str, source_sha: &str, width: usize) -> String {
    format!(
        r#"//! GENERATED FILE - DO NOT EDIT.
//!
//! Generated mechanically by planck-extractor from {src} (sha256 {sha}).
//! Hand edits are non-conforming: CEP-DR-01 section 4 layer 2 treats
//! extraction output as unverified until qualified, and the standing
//! standard requires every executable artifact to be mechanically
//! extracted. Regenerate with `planck-extractor extract`; drift is
//! detected by `planck-extractor check`.
//!
//! The proofs in {src} are Prop-erased: Lemma..Qed blocks never enter
//! this file (structural erasure in the extractor lexer).
// CEP:FILE: extract/planck_effect/src/lib.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the {src} computational content (pinned Gallina subset): the row domain (2^{w}-element tuple-of-bool lattice), the eight accessors, join/meet/leq, and the pinned prelude mapping (orb -> x|y, andb -> x&y, implb -> if x {{ y }} else {{ true }}).
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset; Rust 2021; the prelude mapping above (pinned in tools/extractor/src/emit.rs).
// CEP:COST: runtime: O(1) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 {sha}; ground truth extract/planck_effect/ground_truth/*.txt (kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in extract/planck_effect)
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with {src} over all 2^{w} x 2^{w} input pairs; the proofs stay in rocq/ (Prop erasure is structural).
// CEP:FV-AXIOMS: none
"#,
        src = source_rel,
        sha = source_sha,
        w = width
    )
}

pub fn emit_all(root: &Path, units: &[ParseUnit]) {
    for u in units {
        let src_path = root.join(&u.file);
        let src_bytes = std::fs::read(&src_path).expect("read source for sha");
        let sha = sha256_hex(&src_bytes);
        let (ty_name, width) = u
            .row_type
            .clone()
            .expect("unit must declare a row type alias");

        let mut lib = lib_rs_header(&u.file, &sha, width);
        lib.push_str("\n");
        lib.push_str(PRELUDE);
        lib.push_str("\n");
        for d in &u.definitions {
            lib.push_str(&def_to_rust(d));
            lib.push_str("\n\n");
        }

        let dir = root.join("extract").join("planck_effect");
        std::fs::create_dir_all(dir.join("src")).expect("mkdir extract src");
        std::fs::create_dir_all(dir.join("tests")).expect("mkdir extract tests");
        std::fs::create_dir_all(dir.join("ground_truth")).expect("mkdir ground_truth");
        std::fs::write(dir.join("src").join("lib.rs"), &lib).expect("write lib.rs");

        let cargo = format!(
            "[package]\nname = \"planck-effect\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndescription = \"Mechanically extracted Rust of {} (generated by planck-extractor; DO NOT EDIT).\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            u.file
        );
        std::fs::write(dir.join("Cargo.toml"), &cargo).expect("write Cargo.toml");

        let test = dif_test_source(&u.file, &sha, &ty_name, width);
        std::fs::write(dir.join("tests").join("dif_effect.rs"), &test)
            .expect("write dif test");
    }
}

/// The DIF test: reconstruct the pinned row enumeration (index bit W-1-k is
/// field k of the tuple, matching the kernel generator's weights), evaluate
/// every (row,row) pair through the extracted functions, and require exact
/// agreement with the kernel vm_compute ground truth over the complete domain.
/// The row tuple construction and the index encoder are GENERATED from the
/// parsed row width — nothing is hand-matched to the source.
pub fn dif_test_source(source_rel: &str, sha: &str, _ty_name: &str, width: usize) -> String {
    // Mechanically build, from width W:
    //  - make_row(i): nested tuple with field k = bit (W-1-k) of i
    //  - row_index(t): sum of field k * 2^(W-1-k)
    let mut make = String::from("fn make_row(i: usize) -> row {\n    let fields = [");
    for k in 0..width {
        make.push_str(&format!("field_bit(i, {}), ", k));
    }
    make.push_str("];\n    row_of_fields(&fields)\n}");

    // Build the right-nested row constructor from width W:
    // f0, (f1, ( ... (f{W-2}, f{W-1}) ... ))
    let mut parts: Vec<String> = Vec::new();
    for k in 0..width {
        parts.push(format!("f[{}]", k));
    }
    let mut tuple_expr = parts[width - 1].clone();
    for k in (0..width - 1).rev() {
        tuple_expr = format!("{}, ({})", parts[k], tuple_expr);
    }
    let row_of = format!("fn row_of_fields(f: &[bool]) -> row {{\n    ({})\n}}", tuple_expr);

    let mut index_fn = String::from("fn row_index(t: &row) -> usize {\n    let f = row_fields(t);\n    let mut acc = 0usize;\n");
    index_fn.push_str("    ");
    for k in 0..width {
        index_fn.push_str(&format!("if f[{}] {{ acc += 1usize << {}; }} ", k, width - 1 - k));
    }
    index_fn.push_str("\n    acc\n}");

    // row_fields(t): [t.0, (t.1).0, ...] generated from width
    let mut fields_fn = String::from("fn row_fields(t: &row) -> [bool; WIDTH] {\n    [");
    for k in 0..width {
        // k .1 accesses then .0 (except last: .1)
        let mut expr = String::from("t");
        for _ in 0..k {
            expr.push_str(".1");
        }
        if k < width - 1 {
            expr.push_str(".0");
        }
        fields_fn.push_str(&expr);
        fields_fn.push_str(", ");
    }
    fields_fn.push_str("]\n}");

    format!(
        r#"//! GENERATED FILE - DO NOT EDIT. Generated by planck-extractor from {src} (sha256 {sha}).
//!
//! DIF: differential equivalence of the extracted Rust against the Coq
//! kernel (vm_compute) over the COMPLETE finite domain: all 2^{w} rows as
//! inputs, all 2^{w} x 2^{w} pairs for the binary functions. The ground
//! truth files under ground_truth/ are captured kernel output (Coq 8.20.1),
//! regenerated by `planck-extractor dif` and never hand-edited.
// CEP:FILE: extract/planck_effect/tests/dif_effect.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2).
// CEP:WHAT: DIF differential test: extracted row functions vs kernel ground truth over the complete 2^{w} x 2^{w} domain, per the pinned index encoding (bit W-1-k = tuple field k).
// CEP:WHY: CEP-DR-01 DR-1.5 adds method DIF for untrusted extraction output; this test is the discharge instrument and must pass in debug AND release (overflow-checks on).
// CEP:STATUS: complete (generated)
// CEP:FAILURE: any index, order, or value mismatch is a hard test failure; a missing ground-truth file fails the build (include_str!).
// CEP:ASSUMES: ground truth files captured from kernel vm_compute (Coq 8.20.1) by planck-extractor dif; enumeration order identical on both sides (pinned by the same generator).
// CEP:COST: test-time only (65,536 evaluations per function, pure bool arithmetic).
// CEP:EVIDENCE: this file; ground_truth/*.txt sha256 recorded under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-2 harness discharging a DIF claim over a complete domain
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: extract/planck_effect/tests/dif_effect.rs
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain DIF; no sampling, no gap.
// CEP:FV-AXIOMS: none
use planck_effect::*;

const WIDTH: usize = {w};
const DOMAIN: usize = 1usize << WIDTH;

// Index encoding pinned by the same generator that emits the kernel oracle:
// index i, field k (0 = leftmost tuple field): bit (WIDTH-1-k) of i.
fn field_bit(i: usize, k: usize) -> bool {{
    ((i >> (WIDTH - 1 - k)) & 1) == 1
}}

{make_row}

{row_of_fields}

{row_fields}

{row_index_fn}

fn parse_gt(text: &str, name: &str) -> Vec<usize> {{
    let v: Vec<usize> = text
        .split_whitespace()
        .map(|s| s.parse().expect("ground truth integer"))
        .collect();
    assert_eq!(v.len(), DOMAIN * DOMAIN, "ground truth size mismatch in {{}}", name);
    v
}}

#[test]
fn domain_size_is_pinned() {{
    assert_eq!(DOMAIN, 1 << WIDTH);
}}

#[test]
fn dif_row_functions_full_domain() {{
    let gt_join = parse_gt(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/row_algebra_join.txt")),
        "row_algebra_join.txt",
    );
    let gt_meet = parse_gt(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/row_algebra_meet.txt")),
        "row_algebra_meet.txt",
    );
    let gt_leq = parse_gt(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/row_algebra_leq.txt")),
        "row_algebra_leq.txt",
    );

    for i in 0..DOMAIN {{
        for j in 0..DOMAIN {{
            let a = make_row(i);
            let b = make_row(j);
            let jj = row_join(a, b);
            let mm = row_meet(a, b);
            let ll = row_leq(a, b);
            let k = i * DOMAIN + j;
            assert_eq!(row_index(&jj), gt_join[k], "join mismatch at pair ({{}},{{}})", i, j);
            assert_eq!(row_index(&mm), gt_meet[k], "meet mismatch at pair ({{}},{{}})", i, j);
            assert_eq!(if ll {{ 1 }} else {{ 0 }}, gt_leq[k], "leq mismatch at pair ({{}},{{}})", i, j);
        }}
    }}
}}
"#,
        src = source_rel,
        sha = sha,
        w = width,
        make_row = make,
        row_of_fields = row_of,
        row_fields = fields_fn,
        row_index_fn = index_fn,
    )
}
