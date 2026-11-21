// CEP:FILE: tools/extractor/src/dif.rs
// CEP:WHAT: The DIF driver (v2, multi-domain): regenerate-and-diff (check, enforcing the hand-edit ban and the DR-8.7 no-plain-text rule over all three extracted crates), and the full differential run — reference transcription (transcribe), kernel ground truth capture from vm_compute stdout per domain (effect rows: 65,536 pairs per function; charclass: the complete 256-byte domain; operator: the closed 44-row domain, the 1936-pair tables, and the pinned 16,563-key lookup sweep), ground-truth emission as generated Rust source, reference-axis table emission, CEP lint of the proof layer, cargo test in debug AND release for every crate, and the sha256-recorded artifacts under tools/fv/extract/artifacts/ with time-qualified names (VR-14(a): a same-day re-run writes a NEW artifact, never overwrites the record).
// CEP:WHY: CEP-DR-01 DR-1.5 method DIF discharges untrusted extraction output over complete domains against the kernel; the reference axis closes the transcription loop mechanically (DR-8.8 v2.15); honest outcomes only (VR-4).
// CEP:STATUS: complete
// CEP:FAILURE: A parse, emission, or kernel mismatch fails the run loudly; a hand edit to extract/ is non-conforming and caught by the check subcommand before every DIF run.
// CEP:SECURITY: repository-trusted inputs; no secrets; artifacts record actual outcomes only.
// CEP:ASSUMES: coqc (pinned Coq 8.20.1) and cargo on PATH; bash for rocq/build.sh.
// CEP:COST: kernel evaluation + cargo test build time.
// CEP:EVIDENCE: tools/fv/extract/artifacts/*.md (sha256 of ground truth + results); extract/*/ground_truth (generated Rust source).
// CEP:FV-CLASS: FV-2 (harness discharging DIF claims over complete domains)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain DIF per domain (DR-8.8 v2.9); no sampling.
// CEP:FV-AXIOMS: none

use crate::{emit, gt, parse, transcribe};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const ARTIFACT_DIR: &str = "tools/fv/extract/artifacts";

#[derive(Clone, Copy, PartialEq)]
pub enum ArtifactKind {
    RowAlgebra,
    CharClass,
    Operator,
}

impl ArtifactKind {
    pub fn slug(self) -> &'static str {
        match self {
            ArtifactKind::RowAlgebra => "rowalgebra",
            ArtifactKind::CharClass => "charclass",
            ArtifactKind::Operator => "operator",
        }
    }
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// UTC date (YYYY-MM-DD) + time (THHMMSSZ) from the system clock via the unix
/// epoch — civil-from-days (Howard Hinnant's algorithm), no dependencies. The
/// time-qualified names implement VR-14(a): a same-day re-run writes a NEW
/// artifact instead of overwriting the historical record (VR-4: never
/// rewritten).
fn utc_stamp_now() -> (String, String) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (
        format!("{:04}-{:02}-{:02}", y, m, d),
        format!("T{:02}{:02}{:02}Z", sod / 3600, (sod % 3600) / 60, sod % 60),
    )
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = String::with_capacity(64);
    for x in h.iter() {
        out.push_str(&format!("{:08x}", x));
    }
    out
}

// ------- generated-file inventory (for the hand-edit ban) -------

pub fn parse_all_units(root: &Path) -> Vec<parse::ParseUnit> {
    let mut units = Vec::new();
    for rel in crate::ROOT_FILES {
        let src = std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| {
                eprintln!("ERROR: cannot read {}: {}", rel, e);
                std::process::exit(2);
            });
        units.push(parse::parse_unit(rel, &src));
    }
    units
}

pub fn generated_files(root: &Path, units: &[parse::ParseUnit]) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();

    // the transcribed data unit (regenerate + diff — DR-8.8 v2.15)
    let op_src = std::fs::read_to_string(root.join(transcribe::REF_OPERATOR))
        .unwrap_or_else(|e| die(&format!("cannot read the reference source: {}", e)));
    let op_sha = sha256_hex(op_src.as_bytes());
    let (rows, decl_order, _prec) = transcribe::parse_ref_operator(&op_src);
    out.push((
        root.join("rocq/Effect/OperatorTableData.v"),
        transcribe::transcribe_operator_data_v(&rows, &decl_order, &op_sha),
    ));

    // per-crate generated sources: lib.rs, Cargo.toml, dif test
    let mut order: Vec<&'static str> = Vec::new();
    for u in units {
        let c = emit::crate_for(u);
        if !order.contains(&c) {
            order.push(c);
        }
    }
    for c in order {
        let group: Vec<&parse::ParseUnit> = units.iter().filter(|u| emit::crate_for(u) == c).collect();
        let dir = root.join("extract").join(c);

        // regenerate lib.rs exactly as emit_all does
        let mut lib = String::new();
        let crate_enums: Vec<parse::EnumDecl> =
            group.iter().flat_map(|u| u.enums.clone()).collect();
        for u in &group {
            let src_bytes = std::fs::read(root.join(&u.file)).expect("read source for sha");
            let sha = sha256_hex(&src_bytes);
            let dir_rel = format!("extract/{}", c);
            let what = match u.kind {
                parse::UnitKind::Row => String::from("the row domain (2^8-element tuple-of-bool lattice), the eight accessors, join/meet/leq, and the pinned v1 prelude mapping (orb -> x|y, andb -> x&y, implb -> if x { y } else { true })"),
                parse::UnitKind::Byte => String::from("the pinned ASCII character-class predicates over the u8 domain (whitespace, decimal/hex digit, identifier start/continue, comment-introducer lead bytes), the pinned named constants, and the v1 prelude mapping"),
                parse::UnitKind::Operator => String::from("the closed column enums (opid 44 variants, assoc, arity, overflow, ordering, lookup_error), the pinned operator-row data (keys as &'static str via the v2.6 mapping, 7-tuple rows, sorted table), the five column accessors, the OP-number projection, the precedence comparison, and the fuel-shaped binary-search lookup over the pinned const array"),
            };
            let is_first = lib.is_empty();
            if is_first {
                lib.push_str(&emit::unit_header_pub(&u.file, &sha, &what, &dir_rel));
            } else {
                lib.push_str(&emit::unit_header_plain(&u.file, &sha, &what, &dir_rel));
            }
            lib.push_str("\n");
            // preludes and enums are crate-level: emit them once, with the
            // first unit of the crate
            if is_first {
                match u.kind {
                    parse::UnitKind::Row => {
                        lib.push_str(emit::PRELUDE_V1);
                        lib.push_str("\n");
                    }
                    _ => {
                        lib.push_str(emit::PRELUDE_V1);
                        lib.push_str("\n");
                        lib.push_str(emit::PRELUDE_V2);
                        lib.push_str("\n");
                    }
                }
                if u.kind == parse::UnitKind::Operator {
                    lib.push_str(&emit::enums_to_rust(&crate_enums));
                }
            }
            for d in &u.definitions {
                if emit::PINNED_SKIP.contains(&d.name.as_str()) {
                    continue; // pinned-mapped: nothing is emitted
                }
                if d.fuel_fixpoint.is_some() {
                    lib.push_str(&emit::fixpoint_to_rust(d, &crate_enums));
                } else {
                    lib.push_str(&emit::def_to_rust(d, &crate_enums));
                }
                lib.push_str("\n\n");
            }
        }
        out.push((dir.join("src").join("lib.rs"), lib));

        let cargo = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndescription = \"Mechanically extracted Rust of the pinned foundation proofs (generated by planck-extractor; DO NOT EDIT).\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            c.replace('_', "-")
        );
        out.push((dir.join("Cargo.toml"), cargo));

        let test = match group[0].kind {
            parse::UnitKind::Row => {
                let u0 = group[0];
                let (ty_name, width) = u0.row_type.clone().expect("row type alias");
                emit::dif_test_source(
                    &u0.file,
                    &sha256_hex(&std::fs::read(root.join(&u0.file)).expect("src")),
                    &ty_name,
                    width,
                    &gt::tabulated_functions(u0),
                )
            }
            parse::UnitKind::Byte => emit::dif_test_source_charclass(root, &group),
            parse::UnitKind::Operator => emit::dif_test_source_operator(root, &group),
        };
        out.push((dir.join("tests").join("dif_test.rs"), test));
    }

    // the kernel oracle .v (all units)
    let gtv = gt::ground_truth_v(units);
    out.push((root.join("rocq").join("Effect").join("DifGroundTruth.v"), gtv));
    out
}

fn die(msg: &str) -> ! {
    eprintln!("ERROR: {}", msg);
    std::process::exit(1);
}

pub fn check(root: &Path) -> Result<(), String> {
    let units = parse_all_units(root);
    let mut drift = Vec::new();
    for (path, expected) in generated_files(root, &units) {
        let actual = std::fs::read_to_string(&path).unwrap_or_default();
        if actual != expected {
            drift.push(path.display().to_string());
        }
    }
    let failures: Vec<String> = drift
        .into_iter()
        .map(|p| format!("{} drifted from its mechanical provenance", p))
        .collect();
    check_ground_truth(root, &units)?;
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "HAND-EDIT BAN VIOLATION (CEP-DR-01 §4 layer 2): the following generated files are non-conforming — regenerate with `planck-extractor extract` and re-run `planck-extractor dif`:\n  {}",
            failures.join("\n  ")
        ))
    }
}

/// DR-8.7 guard, extended to all three crates: ground truth under extract/ is
/// generated Rust source with the pinned header/provenance/boundary form and
/// mechanically-known declared lengths; value truth stays the DIF's job.
fn check_ground_truth(root: &Path, units: &[parse::ParseUnit]) -> Result<(), String> {
    // expected declared sizes per const name (mechanically derived from the
    // parsed units and the pinned domain sizes)
    let mut sizes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for u in units {
        match u.kind {
            parse::UnitKind::Row => {
                let (_, row_fns, bool_fns) = effect_tables(u);
                for name in row_fns.iter().chain(bool_fns.iter()) {
                    sizes.insert(format!("{}_TABLE", name.to_uppercase()), 65_536);
                }
            }
            parse::UnitKind::Byte => {
                for name in gt::tabulated_byte_functions(u) {
                    sizes.insert(format!("{}_TABLE", name.to_uppercase()), 256);
                }
                for (name, _) in REF_CC_TABLES {
                    sizes.insert(name.to_string(), 256);
                }
            }
            parse::UnitKind::Operator => {
                for (name, sz) in OP_TABLE_SIZES {
                    sizes.insert(name.to_string(), sz);
                }
            }
        }
    }
    for crate_name in ["planck_effect", "planck_charclass", "planck_operator"] {
        let gt_dir = root.join("extract").join(crate_name).join("ground_truth");
        if let Ok(rd) = std::fs::read_dir(&gt_dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "txt").unwrap_or(false) {
                    return Err(format!(
                        "DR-8.7 VIOLATION: legacy plain-text ground truth {} — extraction deliverables are Rust compilation units; run `planck-extractor dif` to regenerate as generated Rust source",
                        p.display()
                    ));
                }
                if p.extension().map(|x| x == "rs").unwrap_or(false) {
                    let text = std::fs::read_to_string(&p)
                        .map_err(|e| format!("{}: {}", p.display(), e))?;
                    if !text.starts_with("// GENERATED FILE - DO NOT EDIT.") {
                        return Err(format!(
                            "{}: header tampered or missing (hand-edit ban)",
                            p.display()
                        ));
                    }
                    if !text.contains("CEP:PROVENANCE:") {
                        return Err(format!(
                            "{}: missing CEP:PROVENANCE (DR-8.2)",
                            p.display()
                        ));
                    }
                    if !text.trim_end().ends_with("];") {
                        return Err(format!(
                            "{}: trailing content after the last const block (hand-edit ban)",
                            p.display()
                        ));
                    }
                    // per-const declared length + entry count
                    let mut rest = text.as_str();
                    while let Some(pos) = rest.find("pub const ") {
                        let after = &rest[pos + "pub const ".len()..];
                        let name_end = after.find(':').unwrap_or(0);
                        let name = after[..name_end].trim().to_string();
                        let decl_end = after.find("= [").unwrap_or_else(|| {
                            die(&format!("{}: malformed const {}", p.display(), name))
                        });
                        let type_seg = &after[name_end + 1..decl_end];
                        let bracket = type_seg.find(']').unwrap_or_else(|| {
                            die(&format!("{}: malformed const type {}", p.display(), name))
                        });
                        let declared: usize = type_seg[..bracket]
                            .split(';')
                            .nth(1)
                            .unwrap_or("")
                            .trim()
                            .parse()
                            .map_err(|_| {
                                format!(
                                    "{}: missing or bad declared length for {}",
                                    p.display(),
                                    name
                                )
                            })?;
                        let expected = sizes.get(&name).unwrap_or_else(|| {
                            die(&format!(
                                "{}: const {} is not in the mechanical size table",
                                p.display(),
                                name
                            ))
                        });
                        if declared != *expected {
                            return Err(format!(
                                "{}: {} declares {} entries, expected {}",
                                p.display(),
                                name,
                                declared,
                                expected
                            ));
                        }
                        let body = &after[decl_end + 3..];
                        let body_end = find_array_terminator(body).unwrap_or_else(|| {
                            die(&format!("{}: malformed const body {}", p.display(), name))
                        });
                        let count = count_array_entries(&body[..body_end]);
                        if count != *expected {
                            return Err(format!(
                                "{}: {} has {} entries, expected {} (value edits also fail the DIF)",
                                p.display(),
                                name,
                                count,
                                expected
                            ));
                        }
                        rest = &body[body_end + 2..];
                    }
                }
            }
        }
    }
    Ok(())
}

const REF_CC_TABLES: [(&str, ()); 6] = [
    ("REF_WS", ()),
    ("REF_DEC", ()),
    ("REF_HEX", ()),
    ("REF_IDS", ()),
    ("REF_IDC", ()),
    ("REF_COMMENT", ()),
];

const OP_TABLE_SIZES: [(&str, usize); 26] = [
    ("OPID_INDEX_TABLE", 44),
    ("PRECEDENCE_TABLE", 44),
    ("ASSOC_TABLE", 44),
    ("ARITY_TABLE", 44),
    ("ARITY_COUNT_TABLE", 44),
    ("OVERLOADABLE_TABLE", 44),
    ("OVERFLOW_TABLE", 44),
    ("TBL_PRECEDENCE_TABLE", 44),
    ("TBL_ASSOC_TABLE", 44),
    ("TBL_ARITY_TABLE", 44),
    ("TBL_OVERLOADABLE_TABLE", 44),
    ("TBL_OVERFLOW_TABLE", 44),
    ("KEY_EQ_TABLE", 1_936),
    ("KEY_LT_TABLE", 1_936),
    ("PRECEDENCE_COMPARE_TABLE", 1_936),
    ("LOOKUP_SWEEP_TABLE", 16_563),
    ("SWEEP_KEYS", 16_563),
    ("REF_LOOKUP_SWEEP_TABLE", 16_563),
    ("REF_KEYS_TABLE", 44),
    ("REF_OP_OF_SORTED_KEY_TABLE", 44),
    ("REF_PRECEDENCE_TABLE", 44),
    ("REF_ASSOC_TABLE", 44),
    ("REF_ARITY_TABLE", 44),
    ("REF_COUNT_TABLE", 44),
    ("REF_OVERLOADABLE_TABLE", 44),
    ("REF_OVERFLOW_TABLE", 44),
];

/// String-aware scan for the array terminator `];` at depth 0 (outside any
/// string literal) — a &str array whose elements contain the substring `];`
/// (e.g. the two-byte ASCII sweep keys) must not truncate the body scan.
fn find_array_terminator(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => in_string = true,
            ']' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b';' && !in_string {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Escape-aware top-level element counter for a Rust array literal body:
/// walks the text tracking string literals (with backslash escapes) and
/// counts depth-0 separating commas. Works for both numeric and &str arrays.
fn count_array_entries(body: &str) -> usize {
    let mut count = 0usize;
    let mut seen_any = false;
    let mut in_string = false;
    let mut escaped = false;
    for c in body.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                seen_any = true;
            }
            ',' => {
                if seen_any {
                    count += 1;
                    seen_any = false;
                }
            }
            c if !c.is_whitespace() => {
                seen_any = true;
            }
            _ => {}
        }
    }
    if seen_any {
        count += 1;
    }
    count
}

fn effect_tables(u: &parse::ParseUnit) -> (usize, Vec<String>, Vec<String>) {
    let (w, row_fns, bool_fns) = {
        let (r, b) = gt::tabulated_functions(u);
        (u.row_type.clone().expect("row").1, r, b)
    };
    (w, row_fns, bool_fns)
}

pub fn check_exit(root: &Path) {
    match check(root) {
        Ok(()) => println!("check: OK — all generated files match their mechanical provenance"),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}

// ------- the DIF run -------

fn run_capture(cmd: &mut Command) -> (String, String, bool) {
    let out = cmd.output().expect("spawn");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

/// Parse the `= <n> : nat` enumeration-length dump from coqc stdout.
fn parse_nat_len(stdout: &str) -> Result<usize, String> {
    let mut rest = stdout;
    loop {
        match rest.find("= ") {
            None => return Err(String::from("no '= <n> : nat' dump found")),
            Some(p) => {
                let after = &rest[p + 2..];
                match after.find(" : nat") {
                    None => {
                        rest = after;
                        continue;
                    }
                    Some(q) => {
                        let body = after[..q].trim();
                        let v: usize = body
                            .parse()
                            .map_err(|_| format!("bad length token '{}'", body))?;
                        return Ok(v);
                    }
                }
            }
        }
    }
}

/// Parse the nth `= <expr> : list nat` vm_compute dump from coqc stdout.
fn parse_list_nat(stdout: &str, nth: usize) -> Result<Vec<usize>, String> {
    let mut found = Vec::new();
    let mut rest = stdout;
    loop {
        match rest.find("= ") {
            None => break,
            Some(p) => {
                let after = &rest[p + 2..];
                match after.find(" : list nat") {
                    None => break,
                    Some(q) => {
                        found.push(&after[..q]);
                        rest = &after[q + " : list nat".len()..];
                    }
                }
            }
        }
    }
    if nth >= found.len() {
        return Err(format!(
            "expected at least {} list dumps, found {}",
            nth + 1,
            found.len()
        ));
    }
    let body = found[nth].trim().to_string();
    let mut vals = Vec::new();
    if body.starts_with('[') && body.ends_with(']') {
        let inner = &body[1..body.len() - 1];
        for tok in inner.split(';') {
            let t = tok.trim();
            if t.is_empty() {
                continue;
            }
            let v: usize = t
                .parse()
                .map_err(|_| format!("bad integer token '{}' in dump {}", t, nth))?;
            vals.push(v);
        }
        return Ok(vals);
    }
    for tok in body.split("::") {
        let t = tok.trim();
        if t.is_empty() || t == "nil" || t == "[]" {
            continue;
        }
        let v: usize = t
            .parse()
            .map_err(|_| format!("bad integer token '{}' in dump {}", t, nth))?;
        vals.push(v);
    }
    Ok(vals)
}

/// One dump expectation: (table key, crate, expected size, generator file).
struct DumpPlan {
    name: String,
    crate_name: &'static str,
    size: usize,
}

/// Build the dump plan mechanically from the generated oracle text (the
/// Eval lines in order) — the same generator that emits the .v builds the
/// plan, so both sides share one pinned order.
fn build_dump_plan(gtv: &str) -> Vec<DumpPlan> {
    let mut plan = Vec::new();
    let mut line_no_ignored = 0usize;
    let _ = line_no_ignored;
    for line in gtv.lines() {
        let t = line.trim();
        if t.starts_with("Eval vm_compute in (map (fun r2 => row_index (") {
            plan.push(DumpPlan {
                name: chunk_fn_name(t),
                crate_name: "planck_effect",
                size: 256, // chunk; assembled per function below
            });
        } else if t.starts_with("Eval vm_compute in (map (fun r2 => b2n (") {
            plan.push(DumpPlan {
                name: chunk_fn_name(t),
                crate_name: "planck_effect",
                size: 256,
            });
        } else if t.starts_with("Eval vm_compute in (length all_rows)") {
            plan.push(DumpPlan {
                name: String::from("<length>"),
                crate_name: "planck_effect",
                size: 0,
            });
        } else if t.starts_with("Eval vm_compute in (") && t.ends_with("_byte_table).") {
            let name = t
                .trim_start_matches("Eval vm_compute in (")
                .trim_end_matches("_byte_table).")
                .to_string();
            plan.push(DumpPlan {
                name,
                crate_name: "planck_charclass",
                size: 256,
            });
        } else if t.starts_with("Eval vm_compute in (map opid_index all_opids)") {
            plan.push(DumpPlan {
                name: String::from("opid_index"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t.starts_with("Eval vm_compute in (map precedence all_opids)") {
            plan.push(DumpPlan {
                name: String::from("precedence"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun o => assoc_ord (associativity o)) all_opids)")
        {
            plan.push(DumpPlan {
                name: String::from("assoc"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t.starts_with("Eval vm_compute in (map (fun o => arity_ord (arity_of o)) all_opids)")
        {
            plan.push(DumpPlan {
                name: String::from("arity"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun o => arity_count (arity_of o)) all_opids)")
        {
            plan.push(DumpPlan {
                name: String::from("arity_count"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t.starts_with("Eval vm_compute in (map (fun o => b2n (overloadable o)) all_opids)")
        {
            plan.push(DumpPlan {
                name: String::from("overloadable"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun o => overflow_ord (overflow_of o)) all_opids)")
        {
            plan.push(DumpPlan {
                name: String::from("overflow"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t.starts_with("Eval vm_compute in (map row_precedence operator_table_sorted)") {
            plan.push(DumpPlan {
                name: String::from("tbl_precedence"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun r => assoc_ord (row_assoc r)) operator_table_sorted)")
        {
            plan.push(DumpPlan {
                name: String::from("tbl_assoc"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun r => arity_ord (row_arity r)) operator_table_sorted)")
        {
            plan.push(DumpPlan {
                name: String::from("tbl_arity"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun r => b2n (row_overloadable r)) operator_table_sorted)")
        {
            plan.push(DumpPlan {
                name: String::from("tbl_overloadable"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t
            .starts_with("Eval vm_compute in (map (fun r => overflow_ord (row_overflow r)) operator_table_sorted)")
        {
            plan.push(DumpPlan {
                name: String::from("tbl_overflow"),
                crate_name: "planck_operator",
                size: 44,
            });
        } else if t.starts_with("Eval vm_compute in (key_eq_table)") {
            plan.push(DumpPlan {
                name: String::from("key_eq"),
                crate_name: "planck_operator",
                size: 1_936,
            });
        } else if t.starts_with("Eval vm_compute in (key_lt_table)") {
            plan.push(DumpPlan {
                name: String::from("key_lt"),
                crate_name: "planck_operator",
                size: 1_936,
            });
        } else if t.starts_with("Eval vm_compute in (precedence_compare_table)") {
            plan.push(DumpPlan {
                name: String::from("precedence_compare"),
                crate_name: "planck_operator",
                size: 1_936,
            });
        } else if t.starts_with("Eval vm_compute in (lookup_sweep_table)") {
            plan.push(DumpPlan {
                name: String::from("lookup_sweep"),
                crate_name: "planck_operator",
                size: 16_563,
            });
        }
    }
    plan
}

fn chunk_fn_name(t: &str) -> String {
    let marker = if t.contains("row_index (") {
        "row_index ("
    } else {
        "b2n ("
    };
    let rest = t.split(marker).nth(1).unwrap_or("");
    rest.split(|c: char| c == ' ' || c == '(')
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn run(root: &Path) {
    // 0. One-time migration (DR-8.7): remove superseded plain-text ground
    //    truth mechanically over all crates.
    for crate_name in ["planck_effect", "planck_charclass", "planck_operator"] {
        let gt_dir = root.join("extract").join(crate_name).join("ground_truth");
        if let Ok(rd) = std::fs::read_dir(&gt_dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "txt").unwrap_or(false) {
                    std::fs::remove_file(&p).expect("remove legacy txt ground truth");
                    println!("dif: migrated superseded plain-text ground truth out: {}", p.display());
                }
            }
        }
    }

    // 0.5 Reference transcription (DR-8.8 v2.15): regenerate the operator
    //     data unit from the reference source — regeneration is the only
    //     edit path.
    transcribe::transcribe(root);

    // 1. Enforce the hand-edit ban first: drift fails the run loudly.
    if let Err(e) = check(root) {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }

    // 2. Kernel-check the proof layer and build the .vo the oracle Requires.
    let (so, se, ok) = run_capture(Command::new("bash").arg("build.sh").current_dir(root.join("rocq")));
    if !ok {
        eprintln!("ERROR: rocq/build.sh failed:\n{}\n{}", so, se);
        std::process::exit(1);
    }

    // 2.5 CEP lint of the proof layer (DR-8.4 mechanical enforcement).
    let exe = std::env::current_exe().expect("current exe");
    let (so, se, ok) = run_capture(Command::new(exe).arg("lint-cep").current_dir(root));
    if !ok {
        eprintln!("ERROR: lint-cep failed:\n{}\n{}", so, se);
        std::process::exit(1);
    }

    // 3. Compile the ground-truth oracle and capture the kernel dumps.
    let (so, se, ok) = run_capture(
        Command::new("coqc")
            .args(["-Q", "Effect", "Effect", "Effect/DifGroundTruth.v"])
            .current_dir(root.join("rocq")),
    );
    if !ok {
        eprintln!("ERROR: coqc DifGroundTruth.v failed:\n{}\n{}", so, se);
        std::process::exit(1);
    }

    // 4. Parse the dumps per the mechanical plan.
    let gtv = std::fs::read_to_string(root.join("rocq/Effect/DifGroundTruth.v")).expect("gtv");
    let plan = build_dump_plan(&gtv);
    let list_region = match so.find(" : nat\n") {
        Some(p) => &so[p + " : nat\n".len()..],
        None => die("cannot locate the length-dump terminator"),
    };
    let enum_len = parse_nat_len(&so).unwrap_or_else(|e| die(&format!("parsing enumeration length: {}", e)));
    if enum_len != 256 {
        die(&format!(
            "enumeration length dump mismatch: {} vs expected 256",
            enum_len
        ));
    }
    // assemble effect tables from chunks (256 per chunk, one chunk per row)
    let mut tables: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    let mut effect_chunk_rows: std::collections::HashMap<String, Vec<Vec<usize>>> =
        std::collections::HashMap::new();
    let mut dump_i = 0usize;
    for d in plan.iter() {
        if d.name == "<length>" {
            // the length dump is ': nat', not a list dump — skip it without
            // consuming a list-dump index
            continue;
        }
        let chunk = parse_list_nat(list_region, dump_i)
            .unwrap_or_else(|e| die(&format!("parsing kernel dump {} for {}: {}", dump_i, d.name, e)));
        if chunk.len() != d.size {
            die(&format!(
                "dump {} for {} has {} entries, expected {}",
                dump_i,
                d.name,
                chunk.len(),
                d.size
            ));
        }
        if d.crate_name == "planck_effect" {
            effect_chunk_rows
                .entry(d.name.clone())
                .or_default()
                .push(chunk);
        } else {
            tables.insert(d.name.clone(), chunk);
        }
        dump_i += 1;
    }
    for (name, chunks) in effect_chunk_rows {
        let flat: Vec<usize> = chunks.into_iter().flatten().collect();
        if flat.len() != 65_536 {
            die(&format!(
                "effect table {} has {} entries, expected 65536",
                name,
                flat.len()
            ));
        }
        tables.insert(name, flat);
    }

    // 5. Reference-axis data (mechanical transcription from the reference
    //    sources) + ground-truth emission as generated Rust source (DR-8.7).
    let op_ref_src = std::fs::read_to_string(root.join(transcribe::REF_OPERATOR)).expect("ref op");
    let op_ref_sha = sha256_hex(op_ref_src.as_bytes());
    let (ref_rows, decl_order, _prec) = transcribe::parse_ref_operator(&op_ref_src);
    let ref_keys_sorted: Vec<String> = ref_rows.iter().map(|r| r.token.clone()).collect();
    let sweep = gt::sweep_keys(&ref_keys_sorted);
    let sweep_expect: Vec<usize> = {
        // the reference axis of the sweep: independent linear scan over the
        // reference keys (the generator's own implementation, distinct from
        // both the extracted binary search and the kernel's list machinery),
        // encoded as the OP NUMBER of the hit row (the same encoding as the
        // kernel oracle's lookup_ix: opid_index of the hit row's opid)
        let mut op_of_sorted_key: Vec<usize> = Vec::new();
        {
            let by_token: std::collections::HashMap<&str, &transcribe::RefRow> =
                ref_rows.iter().map(|r| (r.token.as_str(), r)).collect();
            for tk in &ref_keys_sorted {
                let r = by_token.get(tk.as_str())
                    .unwrap_or_else(|| die(&format!("no reference row for key {}", tk)));
                let pos = decl_order.iter().position(|o| o == &r.id)
                    .unwrap_or_else(|| die(&format!("no OP number for {}", r.id)));
                op_of_sorted_key.push(pos + 1);
            }
        }
        let mut out = Vec::new();
        for k in &sweep {
            let mut hit = 0usize;
            for (i, rk) in ref_keys_sorted.iter().enumerate() {
                let rkb: Vec<u32> = rk.chars().map(|c| c as u32).collect();
                if k == &rkb {
                    hit = op_of_sorted_key[i];
                    break;
                }
            }
            out.push(hit);
        }
        assert_eq!(out.len(), 16_563);
        out
    };

    // effect GT files
    let effect_gt_dir = root.join("extract/planck_effect/ground_truth");
    std::fs::create_dir_all(&effect_gt_dir).expect("mkdir gt");
    let mut all_hashes: Vec<(String, String, String, usize)> = Vec::new(); // (crate, file, sha, entries)
    for name in ["row_join", "row_meet", "row_leq"] {
        let vals = match tables.get(name) {
            Some(v) => v,
            None => die(&format!("effect table {} missing from the kernel dumps", name)),
        };
        let text = gt::ground_truth_rs(name, vals, "rocq/Effect/RowAlgebra.v", &sha_of(root, "rocq/Effect/RowAlgebra.v"));
        let h = sha256_hex(text.as_bytes());
        all_hashes.push(("planck_effect".into(), format!("{}_table.rs", name), h, vals.len()));
        std::fs::write(effect_gt_dir.join(format!("{}_table.rs", name)), &text).expect("write gt");
    }

    // charclass GT files + reference axis
    let cc_gt_dir = root.join("extract/planck_charclass/ground_truth");
    std::fs::create_dir_all(&cc_gt_dir).expect("mkdir gt");
    let cc_src = std::fs::read_to_string(root.join(transcribe::REF_CHARCLASS)).expect("ref cc");
    let cc_sha = sha256_hex(cc_src.as_bytes());
    for u in parse_all_units(root).iter() {
        if u.kind != parse::UnitKind::Byte {
            continue;
        }
        for name in gt::tabulated_byte_functions(u) {
            let vals = match tables.get(&name) {
                Some(v) => v,
                None => die(&format!("charclass table {} missing from the kernel dumps", name)),
            };
            let text = gt::ground_truth_rs(&name, vals, "rocq/Effect/CharClass.v", &sha_of(root, "rocq/Effect/CharClass.v"));
            let h = sha256_hex(text.as_bytes());
            all_hashes.push(("planck_charclass".into(), format!("{}_table.rs", name), h, vals.len()));
            std::fs::write(cc_gt_dir.join(format!("{}_table.rs", name)), &text).expect("write gt");
        }
    }
    let ref_cc = gt::ref_charclass_tables_rs(&cc_src, &cc_sha);
    all_hashes.push((
        "planck_charclass".into(),
        "ref_charclass_tables.rs".into(),
        sha256_hex(ref_cc.as_bytes()),
        6 * 256,
    ));
    std::fs::write(cc_gt_dir.join("ref_charclass_tables.rs"), &ref_cc).expect("write ref cc");

    // operator GT files (bundled) + sweep + reference axis
    let op_gt_dir = root.join("extract/planck_operator/ground_truth");
    std::fs::create_dir_all(&op_gt_dir).expect("mkdir gt");
    let op_src_sha = sha_of(root, "rocq/Effect/OperatorTable.v");
    let bundle = |name: &str, members: &[(&str, &str)]| -> String {
        let mut parts = Vec::new();
        for (k, src) in members {
            let vals = match tables.get(*k) {
                Some(v) => v,
                None => die(&format!("operator table {} missing from the kernel dumps", k)),
            };
            let member = gt::ground_truth_rs(k, vals, "rocq/Effect/OperatorTable.v", &op_src_sha)
                .replace(&format!("{}_TABLE", k.to_uppercase()), src)
                .replace(
                    &format!("// CEP:FILE: (generated ground truth) {}_table.rs", k),
                    &format!("// CEP:FILE: (member of bundle) extract/planck_operator/ground_truth/{}", name),
                );
            parts.push(member);
        }
        let joined = parts.join("\n\n");
        // one shared header for the bundle
        let header = format!(
            "// GENERATED FILE - DO NOT EDIT.\n//\n// Generated mechanically by planck-extractor (dif) from the Coq kernel's\n// vm_compute output over the complete closed domains (CEP-DR-01 §8 DR-8.7:\n// extraction deliverables are Rust compilation units, never plain-text\n// dumps). Hand edits are non-conforming (DR-8.2).\n// CEP:FILE: extract/planck_operator/ground_truth/{}\n// CEP:PROVENANCE: source rocq/Effect/OperatorTable.v (sha256 {}); kernel oracle rocq/Effect/DifGroundTruth.v (vm_compute, Coq 8.20.1); extractor planck-extractor (dif); generated mechanically.\n// CEP:WHAT: Kernel ground-truth tables ({}) over the complete closed domains, as compiled Rust const data consumed by the generated DIF test via include!.\n// CEP:WHY: The DIF oracle is the kernel itself; DR-8.7 requires generated artifacts under extract/ to be Rust compilation units.\n// CEP:STATUS: complete (generated)\n// CEP:FAILURE: A value edit breaks equivalence and fails the DIF loudly; a format edit fails cargo or planck-extractor check.\n// CEP:SECURITY: repository-trusted inputs.\n// CEP:ASSUMES: kernel vm_compute output (Coq 8.20.1, pinned).\n// CEP:COST: compile-time only.\n// CEP:EVIDENCE: rocq/Effect/DifGroundTruth.v; DIF artifacts under tools/fv/extract/artifacts/.\n// CEP:FV-CLASS: FV-2 (generated oracle data; qualified by the DIF it feeds)\n// CEP:FV-PROVER: Coq 8.20.1 kernel (vm_compute)\n// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)\n// CEP:FV-SORRY-COUNT: N/A\n// CEP:FV-REFINEMENT: complete closed-domain oracle data; no sampling.\n// CEP:FV-AXIOMS: none\n\n",
            name, op_src_sha,
            members.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
        );
        header + &joined
    };
    let op_columns = bundle(
        "op_columns.rs",
        &[
            ("opid_index", "OPID_INDEX_TABLE"),
            ("precedence", "PRECEDENCE_TABLE"),
            ("assoc", "ASSOC_TABLE"),
            ("arity", "ARITY_TABLE"),
            ("arity_count", "ARITY_COUNT_TABLE"),
            ("overloadable", "OVERLOADABLE_TABLE"),
            ("overflow", "OVERFLOW_TABLE"),
        ],
    );
    let op_tbl_columns = bundle(
        "op_table_columns.rs",
        &[
            ("tbl_precedence", "TBL_PRECEDENCE_TABLE"),
            ("tbl_assoc", "TBL_ASSOC_TABLE"),
            ("tbl_arity", "TBL_ARITY_TABLE"),
            ("tbl_overloadable", "TBL_OVERLOADABLE_TABLE"),
            ("tbl_overflow", "TBL_OVERFLOW_TABLE"),
        ],
    );
    let op_key_pairs = bundle(
        "op_key_pairs.rs",
        &[("key_eq", "KEY_EQ_TABLE"), ("key_lt", "KEY_LT_TABLE")],
    );
    let op_prec_pairs = bundle(
        "op_precedence_pairs.rs",
        &[("precedence_compare", "PRECEDENCE_COMPARE_TABLE")],
    );
    let op_sweep = bundle("op_lookup_sweep.rs", &[("lookup_sweep", "LOOKUP_SWEEP_TABLE")]);
    let sweep_keys_text = gt::sweep_keys_rs(&sweep, &op_ref_sha);
    let ref_op = gt::ref_operator_tables_rs(&ref_rows, &decl_order, &ref_keys_sorted, &sweep_expect, &op_ref_sha);
    for (fname, text) in [
        ("op_columns.rs", op_columns),
        ("op_table_columns.rs", op_tbl_columns),
        ("op_key_pairs.rs", op_key_pairs),
        ("op_precedence_pairs.rs", op_prec_pairs),
        ("op_lookup_sweep.rs", op_sweep),
        ("op_sweep_keys.rs", sweep_keys_text),
        ("ref_operator_tables.rs", ref_op),
    ] {
        let h = sha256_hex(text.as_bytes());
        let entries = match fname {
            "op_columns.rs" => 7 * 44,
            "op_table_columns.rs" => 5 * 44,
            "op_key_pairs.rs" => 2 * 1936,
            "op_precedence_pairs.rs" => 1936,
            "op_lookup_sweep.rs" => 16_563,
            "op_sweep_keys.rs" => 16_563,
            "ref_operator_tables.rs" => 7 * 44 + 44 + 16_563,
            _ => 0,
        };
        all_hashes.push(("planck_operator".into(), fname.to_string(), h, entries));
        std::fs::write(op_gt_dir.join(fname), &text).expect("write gt");
    }

    // 6. cargo test in debug AND release for every crate (VR-4 discipline).
    for crate_name in ["planck_effect", "planck_charclass", "planck_operator"] {
        let pkg = root.join("extract").join(crate_name);
        for profile in ["", "--release"] {
            let (so, se, ok) = run_capture(
                Command::new("cargo")
                    .args(["test", profile])
                    .current_dir(&pkg),
            );
            if !ok {
                eprintln!(
                    "ERROR: cargo test {} in {} failed:\n{}\n{}",
                    profile, crate_name, so, se
                );
                std::process::exit(1);
            }
        }
    }

    // 7. Artifacts (VR-4: actual outcomes only; VR-14(a): time-qualified names).
    let art_dir = root.join(ARTIFACT_DIR);
    std::fs::create_dir_all(&art_dir).expect("mkdir artifacts");
    let (date, time) = utc_stamp_now();

    let group = |c: &str| -> Vec<(String, String, usize)> {
        all_hashes
            .iter()
            .filter(|(cc, _, _, _)| cc == c)
            .map(|(_, f, h, n)| (f.clone(), h.clone(), *n))
            .collect()
    };
    let rows_table = |g: &Vec<(String, String, usize)>| -> String {
        g.iter()
            .map(|(f, h, n)| format!("| {} | {} | {} |\n", f, n, h))
            .collect::<Vec<_>>()
            .join("")
    };

    let mk_artifact = |slug: &str, title: &str, what: &str, evidence: &str, tables: String| -> String {
        format!(
            r#"# DIF artifact — {title}

CEP:FILE: tools/fv/extract/artifacts/dif-{slug}-{date}{time}.md
CEP:WHAT: {what}
CEP:WHY: CEP-DR-01 DR-1.5 method DIF; the standing standard requires mechanically extracted code proven equivalent under all workloads of its domain (DR-8.8 v2.9: the complete target domain).
CEP:STATUS: complete
CEP:FAILURE: any mismatch would have failed the run and this artifact would record the failure instead.

STATUS: pass
CEP:ASSUMES: Coq 8.20.1 kernel (oracle); pinned enumerations and encodings shared by both sides (single generator); the reference axis generated mechanically from the reference sources (DR-8.8 v2.15).
CEP:COST: kernel evaluation + cargo test.
CEP:EVIDENCE: {evidence}
CEP:SECURITY: repository-trusted inputs.

## Ground truth and reference tables (sha256)

| file | entries | sha256 |
| ---- | ------- | ------ |
{tables}
## Result

- {result}
- cargo test debug: PASS
- cargo test release: PASS
- verdict: DIF PASS — function-level equivalence over the complete pinned domain(s)
"#,
            slug = slug,
            title = title,
            date = date,
            time = time,
            what = what,
            evidence = evidence,
            tables = tables,
            result = evidence
        )
    };

    let art_effect = mk_artifact(
        "rowalgebra",
        "extracted row functions vs Coq kernel (complete domain)",
        "DIF differential-equivalence record: planck_effect extracted Rust vs kernel vm_compute ground truth over the complete 2^8 x 2^8 domain (65,536 pairs per function; join, meet, leq), in debug AND release. Ground truth is emitted as generated Rust source under extract/planck_effect/ground_truth/ (DR-8.7).",
        "the table below; ground truth under extract/planck_effect/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles; pairing counterpart: math/crates/planck-effect-math full-domain EXH (exh_leq_subset_characterization_524288 and the effect-lattice certificates)",
        rows_table(&group("planck_effect")),
    );
    let art_cc = mk_artifact(
        "charclass",
        "extracted character-class predicates vs Coq kernel and the reference (complete 256-byte domain)",
        "DIF differential-equivalence record: planck_charclass extracted Rust vs kernel vm_compute ground truth over the complete 256-byte domain (13 tabulated byte->bool functions) AND vs the reference kernel's EXH-certified truth tables (three-way agreement), in debug AND release (DR-8.8 v2.9: every value the target u8 can hold is checked).",
        "the table below; ground truth under extract/planck_charclass/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles; kernel laws: rocq/Effect/CharClass.v (complete-domain, zero admits)",
        rows_table(&group("planck_charclass")),
    );
    let art_op = mk_artifact(
        "operator",
        "extracted operator table vs Coq kernel and the reference (closed domains + 16,563-key sweep)",
        "DIF differential-equivalence record: planck_operator extracted Rust vs kernel vm_compute ground truth over the closed domains (44 rows x 12 column/accessor tables, 44 sorted-table columns, 1936 key_eq/key_lt pairs, 1936 precedence_compare pairs) and the pinned 16,563-key lookup sweep, AND vs the reference kernel (three-way agreement), in debug AND release; discharges the pinned v2.8 mappings (key_eq -> ==, key_lt -> <, row_at -> array index, nat -> u8) over the complete pinned domains.",
        "the table below; ground truth under extract/planck_operator/ground_truth/ (generated Rust source, sha256 recorded); lint-cep exit 0; cargo test exit 0 in both profiles; kernel laws: rocq/Effect/OperatorTable.v + OperatorEnums.v (complete closed domains, zero admits)",
        rows_table(&group("planck_operator")),
    );
    for (slug, art) in [
        ("rowalgebra", art_effect),
        ("charclass", art_cc),
        ("operator", art_op),
    ] {
        let art_path = art_dir.join(format!("dif-{}-{}{}.md", slug, date, time));
        std::fs::write(&art_path, art).expect("write artifact");
        println!("dif: PASS ({}), artifact {}", slug, art_path.display());
    }
}

fn sha_of(root: &Path, rel: &str) -> String {
    let bytes = std::fs::read(root.join(rel)).expect("read source for sha");
    sha256_hex(&bytes)
}
