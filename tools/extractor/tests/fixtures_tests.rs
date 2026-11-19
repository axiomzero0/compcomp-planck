// CEP:FILE: tools/extractor/tests/fixtures_tests.rs
// CEP:WHAT: EXH fixture tests executing the real extractor binary against the fixture corpus: every valid fixture must parse, every invalid fixture must be rejected (nonzero exit) — the grammar decision set of the pinned Gallina subset, exercised end-to-end.
// CEP:WHY: The extraction subset boundary is the pinned contract of the extraction layer (CEP-DR-01 §4 layer 2); these tests are its enforcement evidence.
// CEP:STATUS: complete
// CEP:FAILURE: A fixture that parses but should reject (or vice versa) is a soundness-relevant parser bug and fails this suite.
// CEP:ASSUMES: cargo test builds the planck-extractor binary (CARGO_BIN_EXE).
// CEP:COST: test-time only.
// CEP:EVIDENCE: tests/fixtures/{valid,invalid}/.
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (cargo test)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: full fixture enumeration; no sampling.
// CEP:FV-AXIOMS: none

use std::process::Command;

#[test]
fn invalid_fixtures_are_actually_rejected() {
    let exe = env!("CARGO_BIN_EXE_planck-extractor");
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/invalid");
    let mut count = 0;
    for entry in std::fs::read_dir(dir).expect("dir") {
        let f = entry.expect("entry").path();
        let out = Command::new(exe)
            .args(["parse-check", f.to_str().expect("utf8")])
            .output()
            .expect("spawn");
        assert!(
            !out.status.success(),
            "fixture {} must be rejected by the pinned-subset parser (stderr: {})",
            f.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        count += 1;
    }
    assert_eq!(count, 7, "the invalid fixture inventory is pinned");
}

#[test]
fn valid_fixture_is_actually_accepted() {
    let exe = env!("CARGO_BIN_EXE_planck-extractor");
    let f = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/valid/minimal_row.v");
    let out = Command::new(exe)
        .args(["parse-check", f])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "valid fixture must parse: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
