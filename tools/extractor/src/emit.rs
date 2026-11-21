// CEP:FILE: tools/extractor/src/emit.rs
// CEP:WHAT: Rust emission from parsed Gallina-subset units (v1+v2): type aliases, the pinned preludes (v1: orb/andb/implb; v2: Nat.eqb -> ==, Nat.leb -> <=, Nat.ltb -> <, Nat.add -> saturating_add, Nat.sub -> saturating_sub, Nat.div2 -> /2, Nat.succ -> saturating_add(1), Nat.pred -> saturating_sub(1), nat -> u8, closed Inductive -> #[derive(..)] enum, key literal -> &'static str, list-of-X literal -> const array, if -> if/else, let -> let, match -> match with owner-qualified patterns, Ok/Err -> Result, fuel Fixpoint -> recursive fn, row_at -> const-array indexing, key_eq -> ==, key_lt -> <); writes the separated extract/ tree (one crate per foundation domain: planck_effect, planck_charclass, planck_operator) with CEP:PROVENANCE headers; every emitted artifact is a Rust compilation unit (DR-8.7).
// CEP:WHY: DR-1.4 extraction layer; the emitted Rust is the compiler's code and must carry provenance (source .v, extractor revision, source sha256) and the hand-edit ban in its header.
// CEP:STATUS: complete
// CEP:FAILURE: An emission bug surfaces as a DIF mismatch (kernel ground truth vs extracted Rust over the complete domain) or a cargo failure; both fail the build.
// CEP:SECURITY: repository-trusted inputs; emitted artifacts carry source sha256 provenance; no secrets.
// CEP:ASSUMES: the pinned term/type grammar of parse.rs (v1+v2, DR-8.8); Rust 2021; core str Ord is byte-lexicographic (the reference's own recorded assumption).
// CEP:COST: build-time only.
// CEP:EVIDENCE: extract/planck_effect, extract/planck_charclass, extract/planck_operator (generated); ground truth */ground_truth/*_table.rs (DR-8.7); tools/fv/extract/artifacts (DIF records).
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (cargo test; extractor check)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: one Gallina Definition maps to exactly one Rust item; names are preserved verbatim (snake_case kept, non_camel_case/non_upper_case allowed) for provenance traceability; the pinned mappings are discharged by the DIF over the complete domains (DR-8.8 v2.9).
// CEP:FV-AXIOMS: none

use crate::parse::{Definition, EnumDecl, ListElem, MatchArm, ParseUnit, Pattern, Term, Ty, UnitKind};
use std::path::Path;

pub fn ty_to_rust(ty: &Ty) -> String {
    match ty {
        Ty::Bool => String::from("bool"),
        Ty::Nat => String::from("u8"),
        Ty::Named(n) => match n.as_str() {
            "nat" => String::from("u8"),
            _ => n.clone(),
        },
        Ty::List(el) => match &**el {
            Ty::Nat => String::from("&'static str"), // the pinned key mapping
            Ty::Named(t) => format!("&'static [{}]", t),
            _ => String::from("&'static str"),
        },
        Ty::Tuple(comps) => {
            let inner: Vec<String> = comps.iter().map(ty_to_rust).collect();
            format!("({})", inner.join(", "))
        }
    }
}

/// The unit-wide constructor -> owning-enum table (for qualified emission).
fn ctor_owner<'a>(enums: &'a [EnumDecl], ctor: &str) -> Option<&'a str> {
    for e in enums {
        if e.variants.iter().any(|v| v == ctor) {
            return Some(&e.name);
        }
        if let Some(pairs) = &e.data_kind {
            if pairs.iter().any(|(c, _)| c == ctor) {
                return Some(&e.name);
            }
        }
    }
    None
}

fn qualify_ctor(enums: &[EnumDecl], ctor: &str) -> String {
    match ctor_owner(enums, ctor) {
        Some(owner) => {
            // Ok/Err stay bare (they map to Result)
            if ctor == "Ok" || ctor == "Err" {
                ctor.to_string()
            } else {
                format!("{}::{}", owner, ctor)
            }
        }
        None => ctor.to_string(),
    }
}

/// Emit a term. v1 mappings: fst/snd as projection chains; orb/andb/implb as
/// prelude calls. v2 mappings (DR-8.8 v2.8): Nat.* arithmetic/comparison
/// operators, key_eq/key_lt/row_at pinned mappings, if/let/match structural
/// forms, Ok/Err, constructor qualification, nat literals.
pub fn term_to_rust(t: &Term, enums: &[EnumDecl]) -> String {
    match t {
        Term::BoolLit(b) => {
            if *b {
                String::from("true")
            } else {
                String::from("false")
            }
        }
        Term::NatLit(n) => n.to_string(),
        Term::Ident(n) => match n.as_str() {
            "Ok" | "Err" => n.clone(),
            _ => match ctor_owner(enums, n) {
                Some(_) if n != "Ok" && n != "Err" => qualify_ctor(enums, n),
                _ => n.clone(),
            },
        },
        Term::Pair(comps) => {
            let inner: Vec<String> = comps.iter().map(|c| term_to_rust(c, enums)).collect();
            format!("({})", inner.join(", "))
        }
        Term::NatList(elems) => emit_list_literal(elems, enums),
        Term::If(c, a, b) => format!(
            "if {} {{ {} }} else {{ {} }}",
            term_to_rust(c, enums),
            term_to_rust(a, enums),
            term_to_rust(b, enums)
        ),
        Term::Let(x, v, b) => format!(
            "{{ let {} = {}; {} }}",
            x,
            term_to_rust(v, enums),
            term_to_rust(b, enums)
        ),
        Term::Match(scrut, arms) => {
            let s = term_to_rust(scrut, enums);
            let mut out = format!("match {} {{\n", s);
            for arm in arms {
                let pats: Vec<String> = arm
                    .patterns
                    .iter()
                    .map(|p| emit_pattern(p, enums))
                    .collect();
                out.push_str(&format!(
                    "        {} => {},\n",
                    pats.join(" | "),
                    term_to_rust(&arm.body, enums)
                ));
            }
            out.push_str("    }");
            out
        }
        Term::App(head, args) => {
            // pinned operator mappings
            if let Term::Ident(n) = &**head {
                let a0 = args.first().map(|x| term_to_rust(x, enums));
                let a1 = args.get(1).map(|x| term_to_rust(x, enums));
                match (n.as_str(), args.len()) {
                    ("Nat.eqb", 2) => return format!("{} == {}", a0.unwrap(), a1.unwrap()),
                    ("Nat.leb", 2) => return format!("{} <= {}", a0.unwrap(), a1.unwrap()),
                    ("Nat.ltb", 2) => return format!("{} < {}", a0.unwrap(), a1.unwrap()),
                    ("Nat.add", 2) => {
                        return format!(
                            "({}).saturating_add({})",
                            a0.unwrap(),
                            a1.unwrap()
                        )
                    }
                    ("Nat.sub", 2) => {
                        return format!(
                            "({}).saturating_sub({})",
                            a0.unwrap(),
                            a1.unwrap()
                        )
                    }
                    ("Nat.div2", 1) => return format!("({}) / 2", a0.unwrap()),
                    ("Nat.succ", 1) => {
                        return format!("({}).saturating_add(1)", a0.unwrap())
                    }
                    ("Nat.pred", 1) => {
                        return format!("({}).saturating_sub(1)", a0.unwrap())
                    }
                    ("eqb", 2) => return format!("{} == {}", a0.unwrap(), a1.unwrap()),
                    ("key_eq", 2) => return format!("{} == {}", a0.unwrap(), a1.unwrap()),
                    ("key_lt", 2) => return format!("{} < {}", a0.unwrap(), a1.unwrap()),
                    ("row_at", 1) => {
                        return format!(
                            "operator_table_sorted[({}) as usize]",
                            a0.unwrap()
                        )
                    }
                    _ => {}
                }
            }
            match &**head {
                Term::Ident(n) if n == "fst" && args.len() == 1 => {
                    format!("({}).0", term_to_rust(&args[0], enums))
                }
                Term::Ident(n) if n == "snd" && args.len() == 1 => {
                    format!("({}).1", term_to_rust(&args[0], enums))
                }
                _ => {
                    if args.is_empty() {
                        term_to_rust(head, enums)
                    } else {
                        let a: Vec<String> =
                            args.iter().map(|x| term_to_rust(x, enums)).collect();
                        format!("{}({})", term_to_rust(head, enums), a.join(", "))
                    }
                }
            }
        }
    }
}

fn emit_pattern(p: &Pattern, enums: &[EnumDecl]) -> String {
    match p {
        Pattern::Ctor(c) => {
            if c == "Ok" || c == "Err" {
                format!("{}(_)", c) // data constructors bind their payload
            } else {
                qualify_ctor(enums, c)
            }
        }
        Pattern::CtorWild(c) => {
            if c == "Ok" || c == "Err" {
                format!("{}(_)", c)
            } else {
                format!("{}(_)", qualify_ctor(enums, c))
            }
        }
        Pattern::Zero => String::from("0"),
        Pattern::Succ(_) => String::from("_"), // the fuel S-arm: base case is 0
    }
}

fn emit_list_literal(elems: &[ListElem], enums: &[EnumDecl]) -> String {
    // Decides between: a key string literal, an array of key strings, an
    // array of constructors, or an array of definition-name references.
    let all_nat = elems.iter().all(|e| matches!(e, ListElem::Nat(_)));
    if all_nat {
        // a single key: ASCII decode
        let mut s = String::new();
        for e in elems {
            if let ListElem::Nat(n) = e {
                s.push(char::from_u32(*n).unwrap_or('?'));
            }
        }
        return format!("{:?}", s);
    }
    let parts: Vec<String> = elems
        .iter()
        .map(|e| match e {
            ListElem::Nat(_) => String::from("?"),
            ListElem::SubList(codes) => {
                let s: String = codes
                    .iter()
                    .map(|c| char::from_u32(*c).unwrap_or('?'))
                    .collect();
                format!("{:?}", s)
            }
            ListElem::Ctor(c) => qualify_ctor(enums, c),
            ListElem::NameRef(n) => {
                // constructor references are owner-qualified; definition
                // references stay verbatim
                match ctor_owner(enums, n) {
                    Some(_) => qualify_ctor(enums, n),
                    None => n.clone(),
                }
            }
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

pub fn def_to_rust(d: &Definition, enums: &[EnumDecl]) -> String {
    if let Some(ty) = &d.ty_alias {
        return format!(
            "#[allow(non_camel_case_types)]\npub type {} = {};",
            d.name,
            ty_to_rust(ty)
        );
    }
    // closed enums are emitted by emit_enums, not here
    if d.binders.is_empty() {
        // consts: nat/bool literals, list literals, tuples
        let mut body = term_to_rust(&d.body, enums);
        if matches!(&d.ret, Ty::List(el) if !matches!(&**el, Ty::Nat)) && body.starts_with('[') {
            body = format!("&{}", body);
        }
        return format!(
            "#[allow(non_upper_case_globals)]\npub const {}: {} = {};",
            d.name,
            ty_to_rust(&d.ret),
            body
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
        term_to_rust(&d.body, enums)
    )
}

/// Fuel Fixpoints emit as recursive fns: the 0 arm is the base case; the S arm
/// binds `f = fuel - 1` (fuel >= 1 there, so the u8 subtraction is safe).
pub fn fixpoint_to_rust(d: &Definition, enums: &[EnumDecl]) -> String {
    let fuel = d.fuel_fixpoint.clone().unwrap_or_default();
    let params: Vec<String> = d
        .binders
        .iter()
        .map(|(n, t)| format!("{}: {}", n, ty_to_rust(t)))
        .collect();
    let body = match &d.body {
        Term::Match(_, arms) => {
            let default_arm = &arms[0];
            let rec_arm = &arms[1];
            format!(
                "match {} {{\n        0 => {},\n        _ => {{ let {} = {} - 1; {} }},\n    }}",
                fuel,
                term_to_rust(&default_arm.body, enums),
                rec_arm
                    .patterns
                    .first()
                    .map(|p| match p {
                        Pattern::Succ(f) => f.clone(),
                        _ => String::from("_f"),
                    })
                    .unwrap_or_else(|| String::from("_f")),
                fuel,
                term_to_rust(&rec_arm.body, enums)
            )
        }
        other => term_to_rust(other, enums),
    };
    format!(
        "pub fn {}({}) -> {} {{\n    {}\n}}",
        d.name,
        params.join(", "),
        ty_to_rust(&d.ret),
        body
    )
}

pub fn enums_to_rust(enums: &[EnumDecl]) -> String {
    let mut out = String::new();
    for e in enums {
        match &e.data_kind {
            None => {
                out.push_str("#[allow(non_camel_case_types)]\n#[derive(Clone, Copy, PartialEq, Eq, Debug)]\n");
                out.push_str(&format!("pub enum {} {{\n", e.name));
                for v in &e.variants {
                    out.push_str(&format!("    {},\n", v));
                }
                out.push_str("}\n\n");
            }
            Some(pairs) => {
                // v2.14: the pinned Ok/Err pair maps to core Result
                assert_eq!(pairs.len(), 2);
                assert_eq!(pairs[0].0, "Ok");
                assert_eq!(pairs[1].0, "Err");
                out.push_str("#[allow(non_camel_case_types)]\n");
                out.push_str(&format!(
                    "pub type {} = Result<{}, {}>;\n\n",
                    e.name,
                    ty_to_rust(&Ty::Named(pairs[0].1.clone())),
                    ty_to_rust(&Ty::Named(pairs[1].1.clone()))
                ));
            }
        }
    }
    out
}

/// The pinned-skip names (v2.8/v2.14): their Rust behavior comes from the
/// pinned extraction mappings applied at call sites; nothing is emitted for
/// them (their proof-side bodies stay in rocq/).
pub const PINNED_SKIP: &[&str] = &[
    "row_at", "key_eq", "key_lt", "key_eq_aux", "key_lt_aux",
];

pub const PRELUDE_V1: &str = r#"#[inline]
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

pub const PRELUDE_V2: &str = r#"#[inline]
pub fn eqb(x: bool, y: bool) -> bool {
    x == y
}
"#;

fn sha256_hex(data: &[u8]) -> String {
    crate::dif::sha256_hex(data)
}

pub fn unit_header_pub(source_rel: &str, source_sha: &str, what: &str, extract_dir: &str) -> String {
    unit_header(source_rel, source_sha, what, extract_dir, true)
}

/// A per-unit provenance header; for non-first units in a crate the inner
/// doc comments (//!) become plain comments (//) because they cannot appear
/// after items.
pub fn unit_header_plain(source_rel: &str, source_sha: &str, what: &str, extract_dir: &str) -> String {
    unit_header(source_rel, source_sha, what, extract_dir, false)
}

fn unit_header(source_rel: &str, source_sha: &str, what: &str, extract_dir: &str, inner: bool) -> String {
    let raw = format!(
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
// CEP:FILE: {dir}/src/lib.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor extract`.
// CEP:WHAT: Mechanically extracted Rust of the {src} computational content (pinned Gallina subset v1+v2 per DR-8.8): {what}
// CEP:WHY: CEP-DR-01 DR-1.4 proof-to-Rust extraction; the standing standard bans hand-written compiler code — this file IS the extracted artifact and is never edited by hand.
// CEP:STATUS: complete (generated; qualified by DIF against the Coq kernel over the complete finite domain)
// CEP:FAILURE: Any disagreement with the kernel ground truth fails the DIF test; a hand edit is detected by planck-extractor check (regenerate + diff).
// CEP:ASSUMES: the pinned Gallina subset (DR-8.1/DR-8.8); Rust 2021; the pinned prelude mappings (tools/extractor/src/emit.rs); core str Ord is byte-lexicographic.
// CEP:COST: runtime: O(1)/O(log n) per operation; build-time: cargo.
// CEP:EVIDENCE: source sha256 {sha}; ground truth {dir}/ground_truth/*_table.rs (generated Rust source, kernel vm_compute output, Coq 8.20.1); DIF artifacts under tools/fv/extract/artifacts/.
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:FV-CLASS: FV-0-equivalent-by-DIF (function-level equivalence over the complete finite domain; the extraction step itself is FV-2 tooling)
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle for the DIF ground truth)
// CEP:FV-SCRIPT: tools/extractor (extractor dif; cargo test in {dir})
// CEP:FV-SORRY-COUNT: N/A (generated Rust; equivalence discharged by DIF, not admitted)
// CEP:FV-REFINEMENT: function-level (definitional) equivalence with {src} over the complete pinned domain (DR-8.8 v2.9); the proofs stay in rocq/ (Prop erasure is structural).
// CEP:FV-AXIOMS: none
"#,
        src = source_rel,
        sha = source_sha,
        what = what,
        dir = extract_dir
    );
    if inner {
        raw
    } else {
        raw.replace("//!", "//")
    }
}

/// Group the extraction units into crates: row -> planck_effect, byte ->
/// planck_charclass, operator -> planck_operator (units in ROOT_FILES order).
pub fn crate_for(u: &ParseUnit) -> &'static str {
    match u.kind {
        UnitKind::Row => "planck_effect",
        UnitKind::Byte => "planck_charclass",
        UnitKind::Operator => "planck_operator",
    }
}

pub fn emit_all(root: &Path, units: &[ParseUnit]) {
    // group units by crate, preserving order
    let mut order: Vec<&str> = Vec::new();
    let mut grouped: Vec<(&str, Vec<&ParseUnit>)> = Vec::new();
    for u in units {
        let c = crate_for(u);
        if !order.contains(&c) {
            order.push(c);
            grouped.push((c, Vec::new()));
        }
        for g in grouped.iter_mut() {
            if g.0 == c {
                g.1.push(u);
            }
        }
    }

    for (crate_name, group) in &grouped {
        let dir_rel = format!("extract/{}", crate_name);
        let dir = root.join(&dir_rel);
        std::fs::create_dir_all(dir.join("src")).expect("mkdir extract src");
        std::fs::create_dir_all(dir.join("tests")).expect("mkdir extract tests");
        std::fs::create_dir_all(dir.join("ground_truth")).expect("mkdir ground_truth");

        let mut lib = String::new();
        // the constructor table is crate-wide: enums may be declared in one
        // unit and used in another (OperatorEnums declares, OperatorTable uses)
        let crate_enums: Vec<EnumDecl> =
            group.iter().flat_map(|u| u.enums.clone()).collect();
        for u in group {
            let src_path = root.join(&u.file);
            let src_bytes = std::fs::read(&src_path).expect("read source for sha");
            let sha = sha256_hex(&src_bytes);
            let what = match u.kind {
                UnitKind::Row => String::from("the row domain (2^8-element tuple-of-bool lattice), the eight accessors, join/meet/leq, and the pinned v1 prelude mapping (orb -> x|y, andb -> x&y, implb -> if x { y } else { true })"),
                UnitKind::Byte => String::from("the pinned ASCII character-class predicates over the u8 domain (whitespace, decimal/hex digit, identifier start/continue, comment-introducer lead bytes), the pinned named constants, and the v1 prelude mapping"),
                UnitKind::Operator => String::from("the closed column enums (opid 44 variants, assoc, arity, overflow, ordering, lookup_error), the pinned operator-row data (keys as &'static str via the v2.6 mapping, 7-tuple rows, sorted table), the five column accessors, the OP-number projection, the precedence comparison, and the fuel-shaped binary-search lookup over the pinned const array"),
            };
            let is_first = lib.is_empty();
            if is_first {
                lib.push_str(&unit_header_pub(&u.file, &sha, &what, &dir_rel));
            } else {
                lib.push_str(&unit_header_plain(&u.file, &sha, &what, &dir_rel));
            }
            lib.push_str("\n");
            // preludes and enums are crate-level: emit them once, with the
            // first unit of the crate
            if is_first {
                match u.kind {
                    UnitKind::Row => {
                        lib.push_str(PRELUDE_V1);
                        lib.push_str("\n");
                    }
                    UnitKind::Byte => {
                        lib.push_str(PRELUDE_V1);
                        lib.push_str("\n");
                        lib.push_str(PRELUDE_V2);
                        lib.push_str("\n");
                    }
                    UnitKind::Operator => {
                        lib.push_str(PRELUDE_V1);
                        lib.push_str("\n");
                        lib.push_str(PRELUDE_V2);
                        lib.push_str("\n");
                        lib.push_str(&enums_to_rust(&crate_enums));
                    }
                }
            }
            for d in &u.definitions {
                if PINNED_SKIP.contains(&d.name.as_str()) {
                    continue; // pinned-mapped: nothing is emitted
                }
                if d.fuel_fixpoint.is_some() {
                    lib.push_str(&fixpoint_to_rust(d, &crate_enums));
                } else {
                    lib.push_str(&def_to_rust(d, &crate_enums));
                }
                lib.push_str("\n\n");
            }
        }

        std::fs::write(dir.join("src").join("lib.rs"), &lib).expect("write lib.rs");

        let cargo = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndescription = \"Mechanically extracted Rust of the pinned foundation proofs (generated by planck-extractor; DO NOT EDIT).\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            crate_name.replace('_', "-")
        );
        std::fs::write(dir.join("Cargo.toml"), &cargo).expect("write Cargo.toml");

        // DIF test per crate kind
        let test = match group[0].kind {
            UnitKind::Row => {
                let u0 = group[0];
                let (ty_name, width) = u0.row_type.clone().expect("row type alias");
                dif_test_source(
                    &u0.file,
                    &sha_of(root, &u0.file),
                    &ty_name,
                    width,
                    &crate::gt::tabulated_functions(u0),
                )
            }
            UnitKind::Byte => dif_test_source_charclass(root, group),
            UnitKind::Operator => dif_test_source_operator(root, group),
        };
        std::fs::write(dir.join("tests").join("dif_test.rs"), &test).expect("write dif test");
    }
}

fn sha_of(root: &Path, rel: &str) -> String {
    let bytes = std::fs::read(root.join(rel)).expect("read source for sha");
    sha256_hex(&bytes)
}

/// The v1 DIF test (effect rows): unchanged in shape; the ground truth is
/// compiled in as generated Rust const data (DR-8.7).
pub fn dif_test_source(
    source_rel: &str,
    sha: &str,
    _ty_name: &str,
    width: usize,
    tabulated: &(Vec<String>, Vec<String>),
) -> String {
    let (row_fns, bool_fns) = tabulated;
    let mut make = String::from("fn make_row(i: usize) -> row {\n    let fields = [");
    for k in 0..width {
        make.push_str(&format!("field_bit(i, {}), ", k));
    }
    make.push_str("];\n    row_of_fields(&fields)\n}");

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

    let mut fields_fn = String::from("fn row_fields(t: &row) -> [bool; WIDTH] {\n    [");
    for k in 0..width {
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

    let mut includes = String::new();
    for name in row_fns.iter().chain(bool_fns.iter()) {
        includes.push_str(&format!(
            "    include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/ground_truth/{}_table.rs\"));\n",
            name
        ));
    }

    let mut asserts = String::new();
    for name in row_fns {
        asserts.push_str(&format!(
            "            assert_eq!(row_index(&{f}(a, b)), gt::{F}_TABLE[k] as usize, \"{f} mismatch at pair ({{}},{{}})\", i, j);\n",
            f = name,
            F = name.to_uppercase()
        ));
    }
    for name in bool_fns {
        asserts.push_str(&format!(
            "            assert_eq!(if {f}(a, b) {{ 1 }} else {{ 0 }}, gt::{F}_TABLE[k] as usize, \"{f} mismatch at pair ({{}},{{}})\", i, j);\n",
            f = name,
            F = name.to_uppercase()
        ));
    }

    format!(
        r#"//! GENERATED FILE - DO NOT EDIT. Generated by planck-extractor from {src} (sha256 {sha}).
//!
//! DIF: differential equivalence of the extracted Rust against the Coq
//! kernel (vm_compute) over the COMPLETE finite domain: all 2^{w} rows as
//! inputs, all 2^{w} x 2^{w} pairs for the binary functions. The ground
//! truth sources under ground_truth/ are generated Rust compilation units
//! carrying captured kernel output (Coq 8.20.1, DR-8.7), regenerated by
//! `planck-extractor dif` and never hand-edited.
// CEP:FILE: extract/planck_effect/tests/dif_test.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2).
// CEP:WHAT: DIF differential test: extracted row functions vs kernel ground truth over the complete 2^{w} x 2^{w} domain, per the pinned index encoding (bit W-1-k = tuple field k); the oracle is compiled in as generated Rust const data (DR-8.7).
// CEP:WHY: CEP-DR-01 DR-1.5 adds method DIF for untrusted extraction output; this test is the discharge instrument and must pass in debug AND release (overflow-checks on).
// CEP:STATUS: complete (generated)
// CEP:FAILURE: any index, order, or value mismatch is a hard test failure; a missing ground-truth source fails the build (include!).
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:ASSUMES: ground truth Rust sources generated from kernel vm_compute (Coq 8.20.1) by planck-extractor dif; enumeration order identical on both sides (pinned by the same generator).
// CEP:COST: test-time only (65,536 evaluations per function, pure bool arithmetic).
// CEP:EVIDENCE: this file; ground_truth/*_table.rs sha256 recorded under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-2 harness discharging a DIF claim over a complete domain
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: extract/planck_effect/tests/dif_test.rs
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain DIF; no sampling, no gap.
// CEP:FV-AXIOMS: none
use planck_effect::*;

const WIDTH: usize = {w};
const DOMAIN: usize = 1usize << WIDTH;

// Kernel ground truth, compiled in (DR-8.7): each table is a GENERATED Rust
// source under ground_truth/, included verbatim — no text parsing anywhere.
mod gt {{
{includes}}}

// Index encoding pinned by the same generator that emits the kernel oracle:
// index i, field k (0 = leftmost tuple field): bit (WIDTH-1-k) of i.
fn field_bit(i: usize, k: usize) -> bool {{
    ((i >> (WIDTH - 1 - k)) & 1) == 1
}}

{make_row}

{row_of_fields}

{row_fields}

{row_index_fn}

#[test]
fn domain_size_is_pinned() {{
    assert_eq!(DOMAIN, 1 << WIDTH);
}}

#[test]
fn dif_row_functions_full_domain() {{
    for i in 0..DOMAIN {{
        for j in 0..DOMAIN {{
            let a = make_row(i);
            let b = make_row(j);
            let k = i * DOMAIN + j;
{asserts}        }}
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

/// The v2 charclass DIF test: 256-byte complete domain x every tabulated
/// byte->bool function (all Definitions named is_* plus the pinned truth
/// forms; check_ helpers excluded), kernel ground truth + the reference
/// truth tables (three-way agreement).
pub fn dif_test_source_charclass(root: &Path, group: &[&ParseUnit]) -> String {
    let sha = crate::dif::sha256_hex(
        &std::fs::read(root.join(&group[0].file)).expect("read source for sha"),
    );
    let src = group[0].file.clone();
    let mut includes = String::new();
    let mut asserts = String::new();
    for u in group {
        for d in &u.definitions {
            if d.ty_alias.is_some() || d.binders.len() != 1 {
                continue;
            }
            if !matches!(&d.ret, Ty::Bool) {
                continue;
            }
            if d.name.starts_with("check_") {
                continue;
            }
            includes.push_str(&format!(
                "    include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/ground_truth/{}_table.rs\"));\n",
                d.name
            ));
            asserts.push_str(&format!(
                "        assert_eq!(if {f}(b) {{ 1 }} else {{ 0 }}, gt::{F}_TABLE[b as usize] as usize, \"{f} mismatch at byte {{}}\", b);\n",
                f = d.name,
                F = d.name.to_uppercase()
            ));
        }
    }
    format!(
        r#"//! GENERATED FILE - DO NOT EDIT. Generated by planck-extractor.
//!
//! DIF: differential equivalence of the extracted character-class predicates
//! against the Coq kernel (vm_compute) over the COMPLETE 256-byte domain
//! (DR-8.8 v2.9: every value the target u8 can hold is checked), with
//! three-way agreement against the reference kernel's truth tables
//! (REF_*_TRUTH tables generated from math/crates/planck-charclass).
//! Generated from {src} (sha256 {sha}).
// CEP:FILE: extract/planck_charclass/tests/dif_test.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2).
// CEP:WHAT: DIF differential test: extracted byte predicates vs kernel ground truth and reference truth tables over the complete 256-byte domain.
// CEP:WHY: CEP-DR-01 DR-1.5 method DIF discharges the extraction over the complete target domain; the reference axis closes the transcription loop mechanically.
// CEP:STATUS: complete (generated)
// CEP:FAILURE: any predicate/table mismatch is a hard test failure; a missing ground-truth source fails the build (include!).
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:ASSUMES: ground truth generated from kernel vm_compute (Coq 8.20.1); reference tables generated from the reference source by the same extractor run.
// CEP:COST: test-time only (256 evaluations per function).
// CEP:EVIDENCE: this file; ground_truth/*_table.rs sha256 recorded under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-2 harness discharging a DIF claim over a complete domain
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: extract/planck_charclass/tests/dif_test.rs
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete-domain DIF; no sampling, no gap.
// CEP:FV-AXIOMS: none
use planck_charclass::*;

mod gt {{
{includes}}}
mod ref_tables {{
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/ref_charclass_tables.rs"));
}}

#[test]
fn dif_predicates_full_domain() {{
    for b in 0u8..=255 {{
{asserts}    }}
}}

#[test]
fn reference_axis_agreement() {{
    for b in 0u8..=255 {{
        assert_eq!(if is_whitespace(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_WS[b as usize] as usize, "ws ref {{}}", b);
        assert_eq!(if is_decimal_digit(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_DEC[b as usize] as usize, "dec ref {{}}", b);
        assert_eq!(if is_hex_digit(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_HEX[b as usize] as usize, "hex ref {{}}", b);
        assert_eq!(if is_ident_start(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_IDS[b as usize] as usize, "ids ref {{}}", b);
        assert_eq!(if is_ident_continue(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_IDC[b as usize] as usize, "idc ref {{}}", b);
        assert_eq!(if is_line_comment_start(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_COMMENT[b as usize] as usize, "comment ref {{}}", b);
        assert_eq!(if is_block_comment_start(b) {{ 1 }} else {{ 0 }}, ref_tables::REF_COMMENT[b as usize] as usize, "comment ref {{}}", b);
    }}
}}
"#,
        src = src,
        sha = sha,
    )
}

/// The v2 operator DIF test: closed 44-row domain x accessors (kernel GT +
/// reference axis), 1936 precedence pairs, key_eq/key_lt pair tables, and
/// the pinned 16,563-key lookup sweep (kernel GT + reference linear scan).
pub fn dif_test_source_operator(root: &Path, group: &[&ParseUnit]) -> String {
    let sha = crate::dif::sha256_hex(
        &std::fs::read(root.join(&group[0].file)).expect("read source for sha"),
    );
    let src = group[0].file.clone();
    format!(
        r#"//! GENERATED FILE - DO NOT EDIT. Generated by planck-extractor.
//!
//! DIF: differential equivalence of the extracted operator table against
//! the Coq kernel (vm_compute) over the complete closed domains (44 rows,
//! 1936 pairs, the pinned 16,563-key lookup sweep) and against the
//! reference kernel (three-way agreement).
// CEP:FILE: extract/planck_operator/tests/dif_test.rs
// CEP:PROVENANCE: source {src} (sha256 {sha}); extractor planck-extractor; generated mechanically — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2).
// CEP:WHAT: DIF differential test: extracted operator accessors, lookup, key comparisons, and precedence comparison vs kernel ground truth and the reference axis over the complete closed domains.
// CEP:WHY: CEP-DR-01 DR-1.5 method DIF discharges the extraction and the pinned v2.8 mappings (key_eq -> ==, key_lt -> <, row_at -> array index) over the complete pinned domains.
// CEP:STATUS: complete (generated)
// CEP:FAILURE: any mismatch is a hard test failure; a missing ground-truth source fails the build (include!).
// CEP:SECURITY: repository-trusted inputs; generated mechanically — hand edits are non-conforming.
// CEP:ASSUMES: ground truth generated from kernel vm_compute (Coq 8.20.1); the sweep enumeration and key encoding pinned by the same generator on both sides; reference tables generated from the reference source by the same extractor run.
// CEP:COST: test-time only (44 + 1936 + 16,563 evaluations).
// CEP:EVIDENCE: this file; ground_truth/*_table.rs sha256 recorded under tools/fv/extract/artifacts/.
// CEP:FV-CLASS: FV-2 harness discharging a DIF claim over a complete domain
// CEP:FV-PROVER: Coq 8.20.1 kernel (oracle)
// CEP:FV-SCRIPT: extract/planck_operator/tests/dif_test.rs
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: complete closed-domain DIF; no sampling, no gap.
// CEP:FV-AXIOMS: none
use planck_operator::*;

mod gt {{
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_columns.rs"));
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_table_columns.rs"));
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_key_pairs.rs"));
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_precedence_pairs.rs"));
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_lookup_sweep.rs"));
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/op_sweep_keys.rs"));
}}
mod ref_tables {{
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/ground_truth/ref_operator_tables.rs"));
}}

fn opid_ord(o: opid) -> u8 {{
    // mechanically mirrors the extracted opid_index (declaration order)
    match o {{
        opid::Assign => 1, opid::AddAssign => 2, opid::SubAssign => 3, opid::MulAssign => 4,
        opid::DivAssign => 5, opid::RemAssign => 6, opid::BitAndAssign => 7, opid::BitOrAssign => 8,
        opid::BitXorAssign => 9, opid::ShlAssign => 10, opid::ShrAssign => 11, opid::Range => 12,
        opid::RangeInclusive => 13, opid::RangeFrom => 14, opid::RangeTo => 15, opid::RangeFull => 16,
        opid::LogicalOr => 17, opid::LogicalAnd => 18, opid::BitOr => 19, opid::BitXor => 20,
        opid::BitAnd => 21, opid::Eq => 22, opid::Ne => 23, opid::Lt => 24, opid::Gt => 25,
        opid::Le => 26, opid::Ge => 27, opid::Shl => 28, opid::Shr => 29, opid::Add => 30,
        opid::Sub => 31, opid::Mul => 32, opid::Div => 33, opid::Rem => 34, opid::Cast => 35,
        opid::Neg => 36, opid::Not => 37, opid::Deref => 38, opid::Borrow => 39,
        opid::Field => 40, opid::MethodCall => 41, opid::Index => 42, opid::Call => 43,
        opid::Try => 44,
    }}
}}

fn make_opid(i: u8) -> opid {{
    match i {{
        1 => opid::Assign, 2 => opid::AddAssign, 3 => opid::SubAssign, 4 => opid::MulAssign,
        5 => opid::DivAssign, 6 => opid::RemAssign, 7 => opid::BitAndAssign, 8 => opid::BitOrAssign,
        9 => opid::BitXorAssign, 10 => opid::ShlAssign, 11 => opid::ShrAssign, 12 => opid::Range,
        13 => opid::RangeInclusive, 14 => opid::RangeFrom, 15 => opid::RangeTo, 16 => opid::RangeFull,
        17 => opid::LogicalOr, 18 => opid::LogicalAnd, 19 => opid::BitOr, 20 => opid::BitXor,
        21 => opid::BitAnd, 22 => opid::Eq, 23 => opid::Ne, 24 => opid::Lt, 25 => opid::Gt,
        26 => opid::Le, 27 => opid::Ge, 28 => opid::Shl, 29 => opid::Shr, 30 => opid::Add,
        31 => opid::Sub, 32 => opid::Mul, 33 => opid::Div, 34 => opid::Rem, 35 => opid::Cast,
        36 => opid::Neg, 37 => opid::Not, 38 => opid::Deref, 39 => opid::Borrow,
        40 => opid::Field, 41 => opid::MethodCall, 42 => opid::Index, 43 => opid::Call,
        44 => opid::Try,
        _ => panic!("opid index out of range"),
    }}
}}

fn assoc_ord(a: assoc) -> u8 {{
    match a {{ assoc::Left => 0, assoc::Right => 1, assoc::None => 2 }}
}}
fn arity_ord(a: arity) -> u8 {{
    match a {{ arity::Zero => 0, arity::One => 1, arity::Two => 2 }}
}}
fn overflow_ord(o: overflow) -> u8 {{
    match o {{ overflow::Panic => 0, overflow::Wrap => 1, overflow::Unchecked => 2, overflow::NotApplicable => 3 }}
}}
fn ordering_ord(o: ordering) -> u8 {{
    match o {{ ordering::Less => 0, ordering::Equal => 1, ordering::Greater => 2 }}
}}

#[test]
fn op_columns_dif_full_44() {{
    for i in 1u8..=44 {{
        let o = make_opid(i);
        let k = (i - 1) as usize;
        assert_eq!(opid_index(o) as usize, gt::OPID_INDEX_TABLE[k] as usize, "opid_index {{}}", i);
        assert_eq!(precedence(o) as usize, gt::PRECEDENCE_TABLE[k] as usize, "precedence {{}}", i);
        assert_eq!(assoc_ord(associativity(o)) as usize, gt::ASSOC_TABLE[k] as usize, "assoc {{}}", i);
        assert_eq!(arity_ord(arity_of(o)) as usize, gt::ARITY_TABLE[k] as usize, "arity {{}}", i);
        assert_eq!(arity_count(arity_of(o)) as usize, gt::ARITY_COUNT_TABLE[k] as usize, "arity_count {{}}", i);
        assert_eq!(if overloadable(o) {{ 1 }} else {{ 0 }}, gt::OVERLOADABLE_TABLE[k] as usize, "overloadable {{}}", i);
        assert_eq!(overflow_ord(overflow_of(o)) as usize, gt::OVERFLOW_TABLE[k] as usize, "overflow {{}}", i);
        // reference axis (generated from math/crates/planck-operator-math)
        assert_eq!(precedence(o) as usize, ref_tables::REF_PRECEDENCE_TABLE[k] as usize, "ref precedence {{}}", i);
        assert_eq!(assoc_ord(associativity(o)), ref_tables::REF_ASSOC_TABLE[k], "ref assoc {{}}", i);
        assert_eq!(arity_ord(arity_of(o)), ref_tables::REF_ARITY_TABLE[k], "ref arity {{}}", i);
        assert_eq!(arity_count(arity_of(o)) as usize, ref_tables::REF_COUNT_TABLE[k] as usize, "ref count {{}}", i);
        assert_eq!(if overloadable(o) {{ 1 }} else {{ 0 }}, ref_tables::REF_OVERLOADABLE_TABLE[k] as usize, "ref overload {{}}", i);
        assert_eq!(overflow_ord(overflow_of(o)), ref_tables::REF_OVERFLOW_TABLE[k], "ref overflow {{}}", i);
    }}
}}

#[test]
fn sorted_table_columns_dif_44() {{
    assert_eq!(operator_table_sorted.len(), 44);
    for i in 0..44 {{
        let r = operator_table_sorted[i];
        // row identity against the reference keys (the transcription loop)
        assert_eq!(r.0, ref_tables::REF_KEYS_TABLE[i], "row key {{}}", i);
        // the row's opid is the one the reference pins for that key
        assert_eq!(
            opid_index(row_opid(r)) as usize,
            ref_tables::REF_OP_OF_SORTED_KEY_TABLE[i] as usize,
            "row opid at sorted position {{}}",
            i
        );
        // columns against the kernel row tables
        assert_eq!(precedence(row_opid(r)) as usize, gt::TBL_PRECEDENCE_TABLE[i] as usize, "tbl prec {{}}", i);
        assert_eq!(assoc_ord(row_assoc(r)) as usize, gt::TBL_ASSOC_TABLE[i] as usize, "tbl assoc {{}}", i);
        assert_eq!(arity_ord(row_arity(r)) as usize, gt::TBL_ARITY_TABLE[i] as usize, "tbl arity {{}}", i);
        assert_eq!(if row_overloadable(r) {{ 1 }} else {{ 0 }}, gt::TBL_OVERLOADABLE_TABLE[i] as usize, "tbl overload {{}}", i);
        assert_eq!(overflow_ord(row_overflow(r)) as usize, gt::TBL_OVERFLOW_TABLE[i] as usize, "tbl overflow {{}}", i);
    }}
}}

#[test]
fn key_pairs_dif_1936() {{
    let mut k = 0usize;
    for a in operator_table_sorted {{
        for b in operator_table_sorted {{
            // the pinned v2.8 mappings (key_eq -> ==, key_lt -> <) applied at
            // the call sites; the DIF against the kernel tables discharges them
            assert_eq!(if a.0 == b.0 {{ 1 }} else {{ 0 }}, gt::KEY_EQ_TABLE[k] as usize, "key_eq {{}}", k);
            assert_eq!(if a.0 < b.0 {{ 1 }} else {{ 0 }}, gt::KEY_LT_TABLE[k] as usize, "key_lt {{}}", k);
            k += 1;
        }}
    }}
    assert_eq!(k, 1936);
}}

#[test]
fn precedence_pairs_dif_1936() {{
    let mut k = 0usize;
    for i in 1u8..=44 {{
        for j in 1u8..=44 {{
            let a = make_opid(i);
            let b = make_opid(j);
            assert_eq!(ordering_ord(precedence_compare(a, b)) as usize, gt::PRECEDENCE_COMPARE_TABLE[k] as usize, "prec_cmp {{}}", k);
            k += 1;
        }}
    }}
    assert_eq!(k, 1936);
}}

#[test]
fn lookup_sweep_dif_16563() {{
    assert_eq!(gt::SWEEP_KEYS.len(), 16563);
    assert_eq!(gt::LOOKUP_SWEEP_TABLE.len(), 16563);
    for (idx, key) in gt::SWEEP_KEYS.iter().enumerate() {{
        let expect = gt::LOOKUP_SWEEP_TABLE[idx] as usize;
        match lookup(key) {{
            Ok(row) => {{
                assert!(expect >= 1, "lookup hit where kernel says miss at {{}} ({{}})", idx, key);
                assert_eq!(opid_index(row_opid(row)) as usize, expect, "lookup hit index at {{}} ({{}})", idx, key);
                // the hit row's key must be exactly the queried key (key
                // distinctness makes this the unique row identity check)
                assert_eq!(row.0, *key, "hit row key at {{}}", idx);
            }}
            Err(_) => {{
                assert_eq!(expect, 0, "lookup miss where kernel says hit at {{}} ({{}})", idx, key);
            }}
        }}
        // reference axis: the generator's independent linear scan over the
        // reference keys must agree with the extracted binary search
        assert_eq!(expect, ref_tables::REF_LOOKUP_SWEEP_TABLE[idx] as usize, "ref sweep {{}}", idx);
    }}
}}

#[test]
fn delimiter_misses() {{
    for k in ["(", ")", "[", "]", "{{", "}}"] {{
        assert!(matches!(lookup(k), Err(_)), "delimiter {{}} must miss", k);
    }}
}}
"#,
        src = src,
        sha = sha,
    )
}
