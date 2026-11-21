// CEP:FILE: tools/extractor/src/cep.rs
// CEP:WHAT: Mechanical enforcement of the CEP standard on the mathematical proof layer (rocq/**/*.v): the complete CEP field set, the FV field set, pinned STATUS vocabulary, and the no-sorry/no-axiom token discipline — the CEP standard becomes a machine-checked property of the proofs, not prose.
// CEP:WHY: The standing user standard ("code the CEP standard into the mathematical proofs") — a proof file without a verifiable CEP contract is non-conforming exactly like a code file without one; lint is FV-2 tooling and runs in the same pipeline as the kernel check.
// CEP:STATUS: complete
// CEP:FAILURE: A missing/incomplete CEP or FV field set, an out-of-vocabulary STATUS, or a banned proof token (Admitted/admit/Axiom as a statement) exits nonzero and the proof layer is non-conforming.
// CEP:ASSUMES: CEP&CC 0.4 field sets (standards.md §10.2, §54.7.1); .v files under rocq/.
// CEP:COST: lint-time only.
// CEP:EVIDENCE: this file; the lint run recorded in the DIF artifact chain.
// CEP:SECURITY: repository-trusted inputs; no secrets; lint-time rejection only.
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (extractor lint-cep)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: field presence is checked mechanically; semantics of the fields remain a REVIEW matter per standards.md.
// CEP:FV-AXIOMS: none

use std::path::Path;

const CEP_FIELDS: &[&str] = &[
    "CEP:FILE:", "CEP:WHAT:", "CEP:WHY:", "CEP:STATUS:", "CEP:FAILURE:", "CEP:ASSUMES:",
    "CEP:COST:", "CEP:EVIDENCE:", "CEP:SECURITY:",
];

const FV_FIELDS: &[&str] = &[
    "CEP:FV-CLASS:",
    "CEP:FV-PROVER:",
    "CEP:FV-SCRIPT:",
    "CEP:FV-SORRY-COUNT:",
    "CEP:FV-REFINEMENT:",
    "CEP:FV-AXIOMS:",
];

const STATUS_VOCAB: &[&str] = &["complete", "partial", "deprecated"];

pub fn lint(root: &Path) {
    let rocq = root.join("rocq");
    let mut files = Vec::new();
    collect_v(&rocq, &mut files);
    let mut failures = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_default();
        let rel = f.strip_prefix(root).unwrap_or(f).display().to_string();
        // Strip comments for token discipline (comments may legitimately
        // mention the words; statements may not).
        for field in CEP_FIELDS {
            if !text.contains(field) {
                failures.push(format!("{}: missing CEP field {}", rel, field));
            }
        }
        for field in FV_FIELDS {
            if !text.contains(field) {
                failures.push(format!("{}: missing FV field {}", rel, field));
            }
        }
        // STATUS vocabulary
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("CEP:STATUS:") {
                let v = rest.trim();
                let head = v.split(&[' ', '(', '—', ','][..]).next().unwrap_or("");
                if !STATUS_VOCAB.contains(&head) {
                    failures.push(format!("{}: CEP:STATUS '{}' out of vocabulary", rel, head));
                }
            }
        }
        // Banned statement tokens (outside comments). VR-14(b): the sweep
        // covers Parameter/Variable/Hypothesis/Conjecture/Variable(s)
        // statement forms in addition to Admitted/Axiom/admit.
        let stripped = strip_comments(&text);
        const BANNED_STATEMENTS: &[&str] = &[
            "Admitted",
            "Axiom ",
            "Parameter ",
            "Hypothesis ",
            "Conjecture ",
            "Variable ",
            "Variables ",
        ];
        for (i, line) in stripped.lines().enumerate() {
            let t = line.trim();
            if t == "admit." {
                failures.push(format!("{}:{}: banned proof token '{}'", rel, i + 1, t));
                continue;
            }
            for b in BANNED_STATEMENTS {
                if t.starts_with(b) {
                    failures.push(format!("{}:{}: banned proof token '{}'", rel, i + 1, t));
                    break;
                }
            }
        }
    }
    if failures.is_empty() {
        println!(
            "lint-cep: OK — {} .v file(s) carry the complete CEP + FV field set, no banned tokens",
            files.len()
        );
    } else {
        for f in &failures {
            eprintln!("CEP LINT FAILURE: {}", f);
        }
        std::process::exit(1);
    }
}

fn collect_v(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_v(&p, out);
            } else if p.extension().map(|x| x == "v").unwrap_or(false) {
                // Generated ground-truth oracle carries its own CEP block and is linted too.
                out.push(p);
            }
        }
    }
}

fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'(' && b[i + 1] == b'*' {
            depth += 1;
            i += 2;
            continue;
        }
        if depth > 0 && i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b')' {
            depth -= 1;
            i += 2;
            continue;
        }
        if depth == 0 {
            out.push(b[i] as char);
        } else if b[i] == b'\n' {
            out.push('\n');
        }
        i += 1;
    }
    out
}
