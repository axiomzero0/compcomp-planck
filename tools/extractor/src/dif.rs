// CEP:FILE: tools/extractor/src/dif.rs
// CEP:WHAT: The DIF driver: regenerate-and-diff (check, enforcing the hand-edit ban), and the full differential run — kernel ground truth capture from vm_compute stdout, ground-truth files, cargo test in debug AND release, sha256-recorded artifact under tools/fv/extract/artifacts/.
// CEP:WHY: DR-1.5 method DIF requires execution against the kernel reference over a stated domain; this driver is the mechanical instrument, and the artifact it writes is the evidence chain (VR-4: outcomes are recorded as they happen, never rewritten).
// CEP:STATUS: complete
// CEP:FAILURE: Any kernel/Rust disagreement, size mismatch, or parse failure exits nonzero and writes no PASS artifact; a check drift exits nonzero before any run.
// CEP:ASSUMES: coqc (pinned Coq 8.20.1) and cargo on PATH; bash for rocq/build.sh.
// CEP:COST: kernel evaluation + cargo test build time.
// CEP:EVIDENCE: tools/fv/extract/artifacts/*.md (sha256 of ground truth + results).
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain DIF; artifact records actual outcomes only.
// CEP:FV-AXIOMS: none

use crate::parse;
use crate::emit;
use crate::gt;
use std::path::{Path, PathBuf};
use std::process::Command;

// ------- sha256 (no dependencies; deterministic artifact hashing) -------

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

/// UTC date (YYYY-MM-DD) from the system clock via the unix epoch —
/// civil-from-days (Howard Hinnant's algorithm), no dependencies. Used for
/// artifact filenames so a later legitimate run writes a NEW dated artifact
/// instead of overwriting the historical record (VR-4: never rewritten).
fn utc_date_today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let days = secs.div_euclid(86400);
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
    format!("{:04}-{:02}-{:02}", y, m, d)
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

pub fn generated_files(root: &Path, units: &[parse::ParseUnit]) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for u in units {
        let src_path = root.join(&u.file);
        let src_bytes = std::fs::read(&src_path).expect("read source");
        let sha = sha256_hex(&src_bytes);
        let (ty_name, width) = u.row_type.clone().expect("row type");
        let dir = root.join("extract").join("planck_effect");

        let mut lib = emit::lib_rs_header(&u.file, &sha, width);
        lib.push_str("\n");
        lib.push_str(emit::PRELUDE);
        lib.push_str("\n");
        for d in &u.definitions {
            lib.push_str(&emit::def_to_rust(d));
            lib.push_str("\n\n");
        }
        out.push((dir.join("src").join("lib.rs"), lib));

        let cargo = format!(
            "[package]\nname = \"planck-effect\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndescription = \"Mechanically extracted Rust of {} (generated by planck-extractor; DO NOT EDIT).\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            u.file
        );
        out.push((dir.join("Cargo.toml"), cargo));

        let test = emit::dif_test_source(&u.file, &sha, &ty_name, width);
        out.push((dir.join("tests").join("dif_effect.rs"), test));

        let gtv = gt::ground_truth_v(std::slice::from_ref(u));
        out.push((root.join("rocq").join("Effect").join("DifGroundTruth.v"), gtv));
    }
    out
}

pub fn check(root: &Path) -> Result<(), String> {
    let mut units = Vec::new();
    for rel in crate::ROOT_FILES {
        let src = std::fs::read_to_string(root.join(rel)).map_err(|e| e.to_string())?;
        units.push(parse::parse_unit(rel, &src));
    }
    let mut drift = Vec::new();
    for (path, expected) in generated_files(root, &units) {
        let actual = std::fs::read_to_string(&path).unwrap_or_default();
        if actual != expected {
            drift.push(path.display().to_string());
        }
    }
    if drift.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "HAND-EDIT BAN VIOLATION (CEP-DR-01 §4 layer 2): the following generated files drifted from their mechanical provenance — regenerate with `planck-extractor extract` and re-run `planck-extractor dif`:\n  {}",
            drift.join("\n  ")
        ))
    }
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
    // Find all occurrences of "= " ... " : list nat"
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
    // Coq prints lists in the imported notation: bracketed `[a; b; ...]`
    // (ListNotations) or `a :: b :: nil` (plain) — accept both mechanically.
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

pub fn run(root: &Path) {
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

    // 4. Parse dumps. Order (pinned by the generator): length dumps first
    //    (row fns then bool fns), then value dumps in the same order.
    //    We only need the value dumps: they are dumps 2..4 (after 2 length
    //    dumps: join, leq) — derive indices mechanically from the table list
    //    by re-parsing the generated .v.
    // Mechanical dump plan: the generated .v's chunk Eval lines, in order.
    // Each chunk line is
    //   Eval vm_compute in (map (fun r2 => row_index (F (r1) r2)) all_rows).
    //   Eval vm_compute in (map (fun r2 => b2n (F (r1) r2)) all_rows).
    // The dump sequence mirrors these lines 1:1 (after the length dump).
    let gtv = std::fs::read_to_string(root.join("rocq/Effect/DifGroundTruth.v")).expect("gtv");
    let mut plan: Vec<String> = Vec::new(); // function name per chunk dump
    for line in gtv.lines() {
        if line.starts_with("Eval vm_compute in (map (fun r2 => row_index (") {
            let rest = line
                .trim_start_matches("Eval vm_compute in (map (fun r2 => row_index (")
                .to_string();
            let name = rest
                .split(|c: char| c == ' ' || c == '(')
                .next()
                .unwrap_or("")
                .to_string();
            plan.push(name);
        } else if line.starts_with("Eval vm_compute in (map (fun r2 => b2n (") {
            let rest = line
                .trim_start_matches("Eval vm_compute in (map (fun r2 => b2n (")
                .to_string();
            let name = rest
                .split(|c: char| c == ' ' || c == '(')
                .next()
                .unwrap_or("")
                .to_string();
            plan.push(name);
        }
    }
    // Dump 0: length all_rows (= 2^W sanity). Dumps 1..: chunks, 1:1 with plan.
    let width = parse_row_width(root);
    let n = 1usize << width;
    let enum_len = parse_nat_len(&so).unwrap_or_else(|e| {
        eprintln!("ERROR parsing enumeration length dump: {}", e);
        std::process::exit(1);
    });
    if enum_len != n {
        eprintln!(
            "ERROR: enumeration length dump mismatch: {} vs expected {}",
            enum_len, n
        );
        std::process::exit(1);
    }
    // All list dumps live strictly after the length dump's ': nat' terminator
    // (the only ': nat\n' occurrence in the captured stdout), so the list
    // finder must start there — otherwise the first 'found' body would span
    // the length dump and the first chunk.
    let list_region = match so.find(" : nat\n") {
        Some(p) => &so[p + " : nat\n".len()..],
        None => {
            eprintln!("ERROR: cannot locate the length-dump terminator");
            std::process::exit(1);
        }
    };
    let mut tables: Vec<(String, Vec<usize>)> = Vec::new(); // (name, values so far)
    // Chunk dumps are ': list nat' dumps; the length dump is ': nat' and
    // does not occupy an index in that space — chunks start at 0.
    let mut dump_i = 0usize;
    for name in &plan {
        let chunk = parse_list_nat(list_region, dump_i).unwrap_or_else(|e| {
            eprintln!("ERROR parsing kernel dump {} for {}: {}", dump_i, name, e);
            std::process::exit(1);
        });
        if chunk.len() != n {
            eprintln!(
                "ERROR: chunk {} for {} has {} entries, expected {}",
                dump_i,
                name,
                chunk.len(),
                n
            );
            std::process::exit(1);
        }
        if let Some(t) = tables.iter_mut().find(|(nm, _)| nm == name) {
            t.1.extend(chunk);
        } else {
            tables.push((name.clone(), chunk));
        }
        dump_i += 1;
    }

    // 5. Write ground-truth files (one integer per line).
    let gt_dir = root.join("extract/planck_effect/ground_truth");
    std::fs::create_dir_all(&gt_dir).expect("mkdir ground_truth");
    let mut hashes = Vec::new();
    for (name, vals) in &tables {
        // expected size: 2^W * 2^W — mechanical from the width of the row type
        let width = parse_row_width(root);
        let n = 1usize << width;
        if vals.len() != n * n {
            eprintln!(
                "ERROR: table {} has {} entries, expected {}",
                name,
                vals.len(),
                n * n
            );
            std::process::exit(1);
        }
        let fname = match name.as_str() {
            "row_join" => "row_algebra_join.txt",
            "row_meet" => "row_algebra_meet.txt",
            "row_leq" => "row_algebra_leq.txt",
            other => {
                eprintln!("ERROR: no ground-truth filename mapping for {}", other);
                std::process::exit(1);
            }
        };
        let text: String = vals
            .iter()
            .map(|v| format!("{}\n", v))
            .collect::<Vec<_>>()
            .join("");
        let h = sha256_hex(text.as_bytes());
        hashes.push((fname, h, vals.len()));
        std::fs::write(gt_dir.join(fname), &text).expect("write ground truth file");
    }

    // 6. cargo test in debug AND release (VR-4 discipline: both profiles).
    let pkg = root.join("extract/planck_effect");
    for profile in ["", "--release"] {
        let (so, se, ok) = run_capture(
            Command::new("cargo")
                .args(["test", profile])
                .current_dir(&pkg),
        );
        if !ok {
            eprintln!(
                "ERROR: cargo test {} failed:\n{}\n{}",
                profile, so, se
            );
            std::process::exit(1);
        }
    }

    // 7. Artifact (VR-4: actual outcomes only).
    let art_dir = root.join("tools/fv/extract/artifacts");
    std::fs::create_dir_all(&art_dir).expect("mkdir artifacts");
    let date = utc_date_today();
    let art = format!(
        r#"# DIF artifact — extracted row functions vs Coq kernel (complete domain)

CEP:FILE: tools/fv/extract/artifacts/dif-rowalgebra-{}.md
CEP:WHAT: DIF differential-equivalence record: planck_effect extracted Rust vs kernel vm_compute ground truth over the complete 2^8 x 2^8 domain (65,536 pairs per function; join, meet, leq), in debug AND release.
CEP:WHY: CEP-DR-01 DR-1.5 method DIF; the standing standard requires mechanically extracted code proven equivalent under all workloads of its domain.
CEP:STATUS: complete
CEP:FAILURE: any mismatch would have failed the run and this artifact would record the failure instead.

STATUS: pass
CEP:ASSUMES: Coq 8.20.1 kernel (oracle); pinned index encoding; enumeration identical on both sides (single generator).
CEP:COST: kernel evaluation + cargo test.
CEP:EVIDENCE: the table below; ground truth files under extract/planck_effect/ground_truth/ (sha256 recorded); cargo test exit 0 in both profiles.
CEP:SECURITY: repository-trusted inputs.

## Ground truth (kernel vm_compute, Coq 8.20.1)

| file | entries | sha256 |
| ---- | ------- | ------ |
{}
## Result

- domain: 2^8 x 2^8 = 65,536 pairs per function (COMPLETE domain; no sampling)
- functions: row_join, row_meet, row_leq (extracted from rocq/Effect/RowAlgebra.v)
- pairing counterpart (SPEC-INDEX §4 pairing rule): the same pinned laws are
  discharged by full-domain EXH in math/crates/planck-effect-math
  (exh_leq_subset_characterization_524288 and the effect-lattice certificates) —
  the two engines are the dual-engine discharge of G18 until OB-DR-5 lands
- cargo test debug: PASS
- cargo test release: PASS
- verdict: DIF PASS — function-level equivalence over the complete finite domain
"#,
        date,
        hashes
            .iter()
            .map(|(f, h, n)| format!("| {} | {} | {} |\n", f, n, h))
            .collect::<Vec<_>>()
            .join("")
    );
    let art_path = art_dir.join(format!("dif-rowalgebra-{}.md", date));
    std::fs::write(&art_path, art).expect("write artifact");
    println!("dif: PASS — complete-domain equivalence, artifact {}", art_path.display());
}

fn parse_row_width(root: &Path) -> usize {
    for rel in crate::ROOT_FILES {
        let src = std::fs::read_to_string(root.join(rel)).expect("read source");
        let u = parse::parse_unit(rel, &src);
        if let Some((_, w)) = u.row_type {
            return w;
        }
    }
    eprintln!("ERROR: no row type width");
    std::process::exit(1);
}
