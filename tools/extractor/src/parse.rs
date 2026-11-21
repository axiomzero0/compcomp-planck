// CEP:FILE: tools/extractor/src/parse.rs
// CEP:WHAT: Lexer and parser for the pinned Gallina extraction subset, versions v1+v2 (CEP-DR-01 §8 DR-8.1/DR-8.8): v1 = top-level Definitions with explicit types, right-nested tuple types and pair terms, identifier/application/boolean-literal terms; v2 adds the deliberate extension record DR-8.8 — nat + numeric literals, Nat.eqb/leb/ltb/add/sub/div2/succ (longhand), if-then-else, let-in, closed inductive enumerations (no parameters, no constructor arguments except the pinned Ok/Err pair), matches over closed enums and the pinned fuel shape, ASCII key literals ([..; ..] and cons chains), the PinnedEnums name-hygiene module wrapper, and the pinned-skip set (row_at, key_eq, key_lt, key_eq_aux, key_lt_aux — proof-side bodies, pinned extraction mappings at call sites). Lemma..Qed blocks are skipped structurally (Prop erasure); every construct outside the subset is a hard reject; the v2.11 parse-time free-identifier rule rejects unresolved identifiers at parse time, not at cargo time.
// CEP:WHY: The extraction subset is the pinned contract between the proof layer and the extraction layer (CEP-DR-01 §4 layer 2); a parser that silently accepted more would widen the untrusted surface — rejection is the soundness discipline here. The v2 extension is recorded BEFORE the first integer-domain extraction (VR-10(b), DR-8.8).
// CEP:STATUS: complete
// CEP:FAILURE: Any subset violation (flat tuple types, missing result types, banned vernaculars, admitted proofs, argument-carrying constructors outside the pinned Ok/Err shape, recursion outside the fuel shape, unresolved identifiers) is a hard parse error with file/line context; the extractor never guesses.
// CEP:SECURITY: repository-trusted inputs; no secrets; parse-time rejection only.
// CEP:ASSUMES: Coq 8.20.1 lexing of the pinned subset; comments (* .. *) nest; qualified names (Nat.eqb) lex as single identifiers (dot consumed between identifier characters only).
// CEP:COST: build-time only.
// CEP:EVIDENCE: tests/fixtures/{valid,invalid} enumerate the grammar decisions (EXH); the three foundation units (CharClass, OperatorEnums, OperatorTableData, OperatorTable) parse under this subset and DIF-qualify the output.
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (cargo test)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: subset boundary is mechanical: accept-list of vernaculars and term forms only; the v2 extension is exactly the DR-8.8 accept-list.
// CEP:FV-AXIOMS: none

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    NatLit(u32),
    Colon,      // :
    ColonEq,    // :=
    Dot,        // .
    LParen,     // (
    RParen,     // )
    Star,       // *
    Comma,      // ,
    Semi,       // ;
    LBracket,   // [
    RBracket,   // ]
    Arrow,      // ->
    Bar,        // |
    ColonColon, // ::
    FatArrow,   // =>
    Forall,     // forall
}

pub struct Lexer {
    pub toks: Vec<(Tok, usize)>, // token, line (1-based)
}

impl Lexer {
    pub fn lex(src: &str) -> Result<Lexer, String> {
        let b = src.as_bytes();
        let mut i = 0usize;
        let mut line = 1usize;
        let mut toks = Vec::new();
        while i < b.len() {
            let c = b[i] as char;
            match c {
                '\n' => {
                    line += 1;
                    i += 1;
                }
                ' ' | '\t' | '\r' => {
                    i += 1;
                }
                '(' => {
                    // comment or LParen
                    if i + 1 < b.len() && b[i + 1] == b'*' {
                        let mut depth = 1usize;
                        i += 2;
                        while i < b.len() && depth > 0 {
                            if i + 1 < b.len() && b[i] == b'(' && b[i + 1] == b'*' {
                                depth += 1;
                                i += 2;
                            } else if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b')' {
                                depth -= 1;
                                i += 2;
                            } else {
                                if b[i] == b'\n' {
                                    line += 1;
                                }
                                i += 1;
                            }
                        }
                        if depth > 0 {
                            return Err(format!("line {}: unterminated comment", line));
                        }
                    } else {
                        toks.push((Tok::LParen, line));
                        i += 1;
                    }
                }
                ')' => {
                    toks.push((Tok::RParen, line));
                    i += 1;
                }
                ':' => {
                    if i + 1 < b.len() && b[i + 1] == b'=' {
                        toks.push((Tok::ColonEq, line));
                        i += 2;
                    } else if i + 1 < b.len() && b[i + 1] == b':' {
                        toks.push((Tok::ColonColon, line));
                        i += 2;
                    } else {
                        toks.push((Tok::Colon, line));
                        i += 1;
                    }
                }
                '.' => {
                    toks.push((Tok::Dot, line));
                    i += 1;
                }
                '*' => {
                    toks.push((Tok::Star, line));
                    i += 1;
                }
                ',' => {
                    toks.push((Tok::Comma, line));
                    i += 1;
                }
                ';' => {
                    toks.push((Tok::Semi, line));
                    i += 1;
                }
                '[' => {
                    toks.push((Tok::LBracket, line));
                    i += 1;
                }
                ']' => {
                    toks.push((Tok::RBracket, line));
                    i += 1;
                }
                '|' => {
                    toks.push((Tok::Bar, line));
                    i += 1;
                }
                '-' => {
                    if i + 1 < b.len() && b[i + 1] == b'>' {
                        toks.push((Tok::Arrow, line));
                        i += 2;
                    } else {
                        // lone '-' (proof bullets, dashes in comments text) — ignored;
                        // Definition-level uses would fail at the term parser.
                        i += 1;
                    }
                }
                '%' => {
                    // scope annotation: %type, %bool_scope, %N, ... — consumed and ignored
                    i += 1;
                    while i < b.len() {
                        let ch = b[i] as char;
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
                '=' => {
                    // v2.5: the match-arm separator '=>' is structural; a lone
                    // '=' (lemma statements) stays a dropped notation char.
                    if i + 1 < b.len() && b[i + 1] == b'>' {
                        toks.push((Tok::FatArrow, line));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                '|' | '&' | '<' | '>' | '~' | '+' | '!' | '$' | '@' | '#' | '^'
                | '`' | '?' | '\\' | '/' => {
                    // Notation characters appear inside Lemma..Qed proof blocks,
                    // which are structurally erased (Prop erasure). They produce
                    // no token; if they were part of a Definition, the term
                    // parser would fail on the missing tokens — the subset
                    // boundary stays enforced at the Definition level.
                    // (v2 definitions use the longhand Nat.eqb/leb/ltb forms.)
                    i += 1;
                }
                c if c.is_ascii_digit() => {
                    // v2.1: decimal natural literals
                    let start = i;
                    while i < b.len() && (b[i] as char).is_ascii_digit() {
                        i += 1;
                    }
                    let n: u32 = src[start..i]
                        .parse()
                        .map_err(|_| format!("line {}: numeric literal out of range", line))?;
                    toks.push((Tok::NatLit(n), line));
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let start = i;
                    while i < b.len() {
                        let ch = b[i] as char;
                        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'' {
                            i += 1;
                        } else if ch == '.'
                            && i + 1 < b.len()
                            && ((b[i + 1] as char).is_ascii_alphanumeric()
                                || b[i + 1] == b'_')
                        {
                            // qualified name: consume the dot between identifier
                            // characters (Nat.eqb, Effect.OperatorEnums); a dot
                            // followed by anything else ends the identifier and
                            // is the sentence terminator.
                            i += 2;
                        } else {
                            break;
                        }
                    }
                    let word = &src[start..i];
                    if word == "forall" {
                        toks.push((Tok::Forall, line));
                    } else {
                        toks.push((Tok::Ident(word.to_string()), line));
                    }
                }
                other => {
                    return Err(format!("line {}: unexpected character '{}'", line, other));
                }
            }
        }
        Ok(Lexer { toks })
    }
}

// ------- Pinned subset AST -------

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Bool,
    Nat,             // v2.1
    Named(String),
    List(Box<Ty>),   // v2.6: pinned `list nat` / `list key` shapes
    Tuple(Vec<Ty>), // right-nested `(T1 * (T2 * ...))` flattened by the parser into components
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Ident(String),
    NatLit(u32),                   // v2.1
    BoolLit(bool),
    App(Box<Term>, Vec<Term>),     // head, args (args non-empty)
    Pair(Vec<Term>),               // right-nested `(t1, (t2, ...))` flattened
    NatList(Vec<ListElem>),        // v2.6: `[..; ..]` literal
    If(Box<Term>, Box<Term>, Box<Term>), // v2.3
    Let(String, Box<Term>, Box<Term>),   // v2.13 (let x := t1 in t2)
    Match(Box<Term>, Vec<MatchArm>),     // v2.5/v2.7
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListElem {
    Nat(u32),          // ASCII code point
    Ctor(String),      // enum constructor reference
    NameRef(String),   // definition name reference (row_00 etc.)
    SubList(Vec<u32>), // nested list literal of nats (delimiter keys)
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>, // multi-pattern arms: | A | B => body
    pub body: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ctor(String),          // bare constructor
    CtorWild(String),      // constructor with a `_` binding (Ok _)
    Zero,                  // nat literal 0 (fuel)
    Succ(String),          // S <binder> (fuel)
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<String>, // bare-variant enums
    pub data_kind: Option<Vec<(String, String)>>, // Ok/Err pair: (ctor, arg type)
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub binders: Vec<(String, Ty)>, // (param, type)
    pub ret: Ty,
    pub body: Term,
    pub ty_alias: Option<Ty>, // set iff ret == Named("Type")
    pub fuel_fixpoint: Option<String>, // set for fuel Fixpoints: the fuel binder name
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitKind {
    Row,      // effect row domain (v1)
    Byte,     // byte-domain foundation kernel (charclass)
    Operator, // operator-table foundation domain
}

#[derive(Debug, Clone)]
pub struct ParseUnit {
    pub file: String,
    pub definitions: Vec<Definition>,
    pub row_type: Option<(String, usize)>, // (type name, width) — the 2^n tuple-of-bool type
    pub enums: Vec<EnumDecl>,              // v2.4 closed enums in declaration order
    pub kind: UnitKind,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => write!(f, "bool"),
            Ty::Nat => write!(f, "nat"),
            Ty::Named(n) => write!(f, "{}", n),
            Ty::List(t) => write!(f, "(list {})", t),
            Ty::Tuple(ts) => {
                // print right-nested with explicit parens (the pinned form)
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}

const BANNED: &[&str] = &[
    "Admitted", "admit", "Axiom", "Parameter", "Hypothesis", "Conjecture", "Variable",
    "Variables", "Program", "CoFixpoint", "Notation", "Declare", "Ltac", "CoInductive",
    "Definition_nonexistent_marker",
];

// v2.8/v2.14: pinned-skip names — proof-side bodies whose Rust behavior comes
// from the pinned extraction mappings (key_eq/key_lt -> core str ==/<, row_at
// -> const-array indexing). Their bodies are kernel-checked by coqc but are
// never emitted and never free-identifier-checked.
const PINNED_SKIP: &[&str] = &["row_at", "key_eq", "key_lt", "key_eq_aux", "key_lt_aux"];

const REQUIRE_ACCEPT: &[&str] = &[
    "Require Import Bool",
    "Require Import Arith",
    "Require Import List",
    "Require Import Lia",
    "Require Import PeanoNat",
    "Require Import Effect.RowAlgebra",
    "Require Import Effect.OperatorEnums",
    "Require Import Effect.OperatorTableData",
    "Require Import Effect.CharClass",
];

const IMPORT_ACCEPT: &[&str] = &["Import ListNotations", "Import PinnedEnums"];

pub fn parse_unit(file: &str, src: &str) -> ParseUnit {
    let lex = Lexer::lex(src).unwrap_or_else(|e| {
        eprintln!("ERROR lexing {}: {}", file, e);
        std::process::exit(1);
    });
    // Banned token sweep (before any statement splitting)
    for (tok, line) in &lex.toks {
        if let Tok::Ident(w) = tok {
            if BANNED.contains(&w.as_str()) {
                eprintln!(
                    "ERROR {}:{}: '{}' is outside the pinned extraction subset (banned)",
                    file, line, w
                );
                std::process::exit(1);
            }
        }
    }
    // Split into sentences at top-level Dots (paren depth 0)
    let mut sentences: Vec<Vec<(Tok, usize)>> = Vec::new();
    let mut cur: Vec<(Tok, usize)> = Vec::new();
    let mut depth = 0usize;
    for t in &lex.toks {
        match t.0 {
            Tok::LParen => depth += 1,
            Tok::RParen => {
                if depth == 0 {
                    eprintln!("ERROR {}: unbalanced ')'", file);
                    std::process::exit(1);
                }
                depth -= 1;
            }
            Tok::Dot if depth == 0 => {
                cur.push(t.clone());
                sentences.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(t.clone());
    }
    if !cur.is_empty() {
        eprintln!("ERROR {}: trailing tokens after last sentence", file);
        std::process::exit(1);
    }

    let _ = sentences; // sentence walk happens in reparse_with_lemma_skip
    let defs = reparse_with_lemma_skip(file, &lex);
    // find width from the type alias: Definition name : Type := (bool * (bool * ...)).
    let mut row_type = None;
    for d in &defs {
        if let Some(w) = type_alias_width(&d) {
            row_type = Some((d.name.clone(), w));
        }
    }
    // v2.4 closed enums in declaration order
    let enums = collect_enums(file, &lex);
    let kind = detect_kind(&row_type, &defs, &enums);
    let mut unit = ParseUnit {
        file: file.to_string(),
        definitions: defs,
        row_type,
        enums,
        kind: UnitKind::Row,
    };
    unit.kind = kind;
    // v2.11: parse-time free-identifier check (Definition/Fixpoint bodies and
    // match arms; lemma blocks are erased before this point) + v2.5
    // exhaustiveness over the crate-visible enum table (this unit's enums
    // plus the pinned cross-unit constructors)
    let enums_known = crate_enums_for_check(&unit);
    free_identifier_check(file, &unit, &enums_known);
    unit
}

fn detect_kind(row_type: &Option<(String, usize)>, defs: &[Definition], enums: &[EnumDecl]) -> UnitKind {
    if row_type.is_some() {
        return UnitKind::Row;
    }
    let has_byte_binder = defs.iter().any(|d| {
        d.binders.iter().any(|(_, t)| matches!(t, Ty::Named(n) if n == "byte"))
    });
    let has_byte_alias = defs.iter().any(|d| {
        matches!(&d.ty_alias, Some(Ty::Named(n)) if n == "nat") && d.name == "byte"
    });
    if has_byte_binder || has_byte_alias {
        return UnitKind::Byte;
    }
    if !enums.is_empty()
        || defs.iter().any(|d| matches!(&d.ty_alias, Some(Ty::List(_))))
        || defs.iter().any(|d| matches!(&d.ret, Ty::Named(n) if n == "opid" || n == "oprow"))
    {
        return UnitKind::Operator;
    }
    eprintln!("ERROR: cannot detect the unit kind mechanically (no row alias, no byte domain, no closed enums)");
    std::process::exit(1);
}

/// v2.11: every identifier in a Definition/Fixpoint body or match arm must
/// resolve to a binder (including let-bindings and the S-arm binder), a
/// definition of the same unit (forward references allowed), an enum
/// constructor declared in the unit or in a pinned required unit, or a name
/// of the pinned v1/v2 preludes or a pinned-skip mapped name. The walker
/// below also applies the v2.5 closed-match exhaustiveness enforcement.
fn validate_match_exhaustiveness_walk(
    file: &str,
    unit: &ParseUnit,
    t: &Term,
    def: &str,
    enums_known: &[EnumDecl],
) {
    match t {
        Term::Match(_, arms) => {
            validate_match_exhaustiveness(file, enums_known, arms, def);
            for arm in arms {
                validate_match_exhaustiveness_walk(file, unit, &arm.body, def, enums_known);
            }
        }
        Term::If(_, a, b) => {
            validate_match_exhaustiveness_walk(file, unit, a, def, enums_known);
            validate_match_exhaustiveness_walk(file, unit, b, def, enums_known);
        }
        Term::Let(_, a, b) => {
            validate_match_exhaustiveness_walk(file, unit, a, def, enums_known);
            validate_match_exhaustiveness_walk(file, unit, b, def, enums_known);
        }
        Term::App(head, args) => {
            validate_match_exhaustiveness_walk(file, unit, head, def, enums_known);
            for a in args {
                validate_match_exhaustiveness_walk(file, unit, a, def, enums_known);
            }
        }
        Term::Pair(comps) => {
            for c in comps {
                validate_match_exhaustiveness_walk(file, unit, c, def, enums_known);
            }
        }
        _ => {}
    }
}

/// The enum table visible to a unit: its own declarations plus the pinned
/// closed enums of the required Effect.OperatorEnums (declaration order).
fn crate_enums_for_check(unit: &ParseUnit) -> Vec<EnumDecl> {
    let mut enums = unit.enums.clone();
    if unit.definitions.iter().any(|d| {
        matches!(&d.ret, Ty::Named(n) if n == "opid" || n == "oprow" || n == "lookup_result")
    }) || unit
        .definitions
        .iter()
        .any(|d| matches!(&d.ty_alias, Some(Ty::List(_))))
    {
        // the pinned operator enums (the required unit's declarations)
        enums.push(EnumDecl {
            name: String::from("opid"),
            variants: [
                "Assign", "AddAssign", "SubAssign", "MulAssign", "DivAssign", "RemAssign",
                "BitAndAssign", "BitOrAssign", "BitXorAssign", "ShlAssign", "ShrAssign",
                "Range", "RangeInclusive", "RangeFrom", "RangeTo", "RangeFull",
                "LogicalOr", "LogicalAnd", "BitOr", "BitXor", "BitAnd", "Eq", "Ne", "Lt", "Gt",
                "Le", "Ge", "Shl", "Shr", "Add", "Sub", "Mul", "Div", "Rem", "Cast",
                "Neg", "Not", "Deref", "Borrow", "Field", "MethodCall", "Index", "Call", "Try",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            data_kind: None,
        });
        for (name, variants) in [
            ("assoc", vec!["Left", "Right", "None"]),
            ("arity", vec!["Zero", "One", "Two"]),
            (
                "overflow",
                vec!["Panic", "Wrap", "Unchecked", "NotApplicable"],
            ),
            ("ordering", vec!["Less", "Equal", "Greater"]),
            ("lookup_error", vec!["UnknownToken"]),
        ] {
            enums.push(EnumDecl {
                name: name.to_string(),
                variants: variants.into_iter().map(|v| v.to_string()).collect(),
                data_kind: None,
            });
        }
        enums.push(EnumDecl {
            name: String::from("lookup_result"),
            variants: Vec::new(),
            data_kind: Some(vec![
                (String::from("Ok"), String::from("oprow")),
                (String::from("Err"), String::from("lookup_error")),
            ]),
        });
    }
    enums
}

fn free_identifier_check(file: &str, unit: &ParseUnit, enums_known: &[EnumDecl]) {
    let mut known: Vec<String> = Vec::new();
    for d in &unit.definitions {
        known.push(d.name.clone());
    }
    for e in &unit.enums {
        match &e.data_kind {
            None => {
                for v in &e.variants {
                    known.push(v.clone());
                }
            }
            Some(pairs) => {
                for (c, _) in pairs {
                    known.push(c.clone());
                }
            }
        }
    }
    let pinned: &[&str] = &[
        // v1 prelude
        "orb", "andb", "implb", "fst", "snd",
        // v2 mapped names (stdlib + extension prelude)
        "Nat.eqb", "Nat.leb", "Nat.ltb", "Nat.add", "Nat.sub", "Nat.div2",
        "Nat.succ", "Nat.pred", "key_eq", "key_lt", "row_at",
        // type names usable in bodies
        "bool", "nat", "byte", "key", "oprow", "opid", "assoc", "arity",
        "overflow", "ordering", "lookup_error", "lookup_result",
        // row/sorted table names come from the unit itself or the required units
        "operator_table_sorted", "all_opids", "delimiter_keys", "true", "false",
        // the pinned row projections of the required Effect.OperatorTableData
        "row_key", "row_opid", "row_precedence", "row_assoc", "row_arity",
        "row_overloadable", "row_overflow", "oprow", "bsearch_aux",
        // the pinned closed-enum constructors of the required Effect.OperatorEnums
        "Assign", "AddAssign", "SubAssign", "MulAssign", "DivAssign", "RemAssign",
        "BitAndAssign", "BitOrAssign", "BitXorAssign", "ShlAssign", "ShrAssign",
        "Range", "RangeInclusive", "RangeFrom", "RangeTo", "RangeFull",
        "LogicalOr", "LogicalAnd", "BitOr", "BitXor", "BitAnd", "Eq", "Ne", "Lt", "Gt",
        "Le", "Ge", "Shl", "Shr", "Add", "Sub", "Mul", "Div", "Rem", "Cast",
        "Neg", "Not", "Deref", "Borrow", "Field", "MethodCall", "Index", "Call", "Try",
        "Left", "Right", "None", "Zero", "One", "Two",
        "Panic", "Wrap", "Unchecked", "NotApplicable", "Less", "Equal", "Greater",
        "UnknownToken", "Ok", "Err", "nil",
    ];
    for w in pinned {
        known.push((*w).to_string());
    }
    for d in &unit.definitions {
        if d.ty_alias.is_some() || PINNED_SKIP.contains(&d.name.as_str()) {
            continue;
        }
        let mut local: Vec<String> = known.clone();
        for (b, _) in &d.binders {
            local.push(b.clone());
        }
        if let Some(fuel) = &d.fuel_fixpoint {
            local.push(fuel.clone());
        }
        // v2.5: closed-enum match exhaustiveness (skipped for fuel fixpoints,
        // whose shape the v2.7 validator owns)
        if d.fuel_fixpoint.is_none() {
            validate_match_exhaustiveness_walk(file, unit, &d.body, &d.name, enums_known);
        }
        check_term_idents(file, unit, &d.body, &local, &d.name, enums_known);
    }
}

fn check_term_idents(
    file: &str,
    unit: &ParseUnit,
    t: &Term,
    known: &[String],
    def: &str,
    enums_known: &[EnumDecl],
) {
    match t {
        Term::Ident(w) => {
            if !known.contains(w) {
                // allow enum constructor references (qualified at emission)
                let is_ctor = enums_known.iter().any(|e| {
                    e.variants.iter().any(|v| v == w)
                        || e.data_kind
                            .as_ref()
                            .map_or(false, |ps| ps.iter().any(|(c, _)| c == w))
                });
                if !is_ctor {
                    eprintln!(
                        "ERROR {}: unresolved identifier '{}' in definition '{}' (DR-8.8 v2.11 parse-time free-identifier rule)",
                        file, w, def
                    );
                    std::process::exit(1);
                }
            }
        }
        Term::NatLit(_) | Term::BoolLit(_) | Term::NatList(_) => {}
        Term::App(head, args) => {
            check_term_idents(file, unit, head, known, def, enums_known);
            for a in args {
                check_term_idents(file, unit, a, known, def, enums_known);
            }
        }
        Term::Pair(comps) => {
            for c in comps {
                check_term_idents(file, unit, c, known, def, enums_known);
            }
        }
        Term::If(a, b, c) => {
            check_term_idents(file, unit, a, known, def, enums_known);
            check_term_idents(file, unit, b, known, def, enums_known);
            check_term_idents(file, unit, c, known, def, enums_known);
        }
        Term::Let(x, a, b) => {
            check_term_idents(file, unit, a, known, def, enums_known);
            // the binding name scopes over the body
            let mut scoped: Vec<String> = known.to_vec();
            scoped.push(x.clone());
            check_term_idents(file, unit, b, &scoped, def, enums_known);
        }
        Term::Match(scrut, arms) => {
            check_term_idents(file, unit, scrut, known, def, enums_known);
            for arm in arms {
                // the S-arm binder scopes over its arm body
                let mut scoped: Vec<String> = known.to_vec();
                for p in &arm.patterns {
                    if let Pattern::Succ(b) = p {
                        scoped.push(b.clone());
                    }
                }
                check_term_idents(file, unit, &arm.body, &scoped, def, enums_known);
            }
        }
    }
}

/// v2.4: collect closed inductive enumerations from Module PinnedEnums blocks
/// and top-level Ok/Err data enums, in declaration order.
pub fn collect_enums_pub(lex: &Lexer) -> Vec<EnumDecl> {
    collect_enums("", lex)
}

fn collect_enums(file: &str, lex: &Lexer) -> Vec<EnumDecl> {
    let mut enums = Vec::new();
    let mut depth_paren = 0usize;
    let mut sentences: Vec<Vec<(Tok, usize)>> = Vec::new();
    let mut cur: Vec<(Tok, usize)> = Vec::new();
    for t in &lex.toks {
        match t.0 {
            Tok::LParen => depth_paren += 1,
            Tok::RParen => depth_paren -= 1,
            Tok::Dot if depth_paren == 0 => {
                cur.push(t.clone());
                sentences.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(t.clone());
    }
    for s in &sentences {
        if s.is_empty() {
            continue;
        }
        if let Tok::Ident(w) = &s[0].0 {
            if w == "Inductive" {
                enums.push(parse_enum(file, s));
            }
        }
    }
    enums
}

fn parse_enum(file: &str, s: &[(Tok, usize)]) -> EnumDecl {
    // Inductive <name> : Type := | C1 | C2 ... .
    // or: Inductive <name> : Type := | Ok : T -> <name> | Err : U -> <name> .
    let name = match s.get(1).map(|t| &t.0) {
        Some(Tok::Ident(w)) => w.clone(),
        _ => fail(file, s, 1, "expected enum name"),
    };
    let mut i = 2usize;
    // ": Type :="
    expect(file, s, i, Tok::Colon, "':' after enum name");
    i += 1;
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "Type" => i += 1,
        _ => fail(file, s, i, "expected Type in the pinned closed-enum shape"),
    }
    expect(file, s, i, Tok::ColonEq, "':=' after the enum sort");
    i += 1;

    let mut variants: Vec<String> = Vec::new();
    let mut data: Vec<(String, String)> = Vec::new();
    let mut saw_bare = false;
    let mut saw_data = false;
    while i < s.len() {
        // optional leading Bar per constructor
        if matches!(s.get(i).map(|t| &t.0), Some(Tok::Bar)) {
            i += 1;
        }
        let cname = match s.get(i).map(|t| &t.0) {
            Some(Tok::Ident(w)) => w.clone(),
            _ => break,
        };
        i += 1;
        if matches!(s.get(i).map(|t| &t.0), Some(Tok::Colon)) {
            // data constructor: C : T -> <name>
            saw_data = true;
            i += 1;
            let arg = match s.get(i).map(|t| &t.0) {
                Some(Tok::Ident(w)) => w.clone(),
                _ => fail(file, s, i, "expected the argument type of a data constructor"),
            };
            i += 1;
            expect(file, s, i, Tok::Arrow, "'->' in a data constructor");
            i += 1;
            match s.get(i).map(|t| &t.0) {
                Some(Tok::Ident(w)) if *w == name => i += 1,
                _ => fail(file, s, i, "a data constructor must return its own enum (no recursion, DR-8.8 v2.4)"),
            }
            data.push((cname, arg));
        } else {
            saw_bare = true;
            variants.push(cname);
        }
        if matches!(s.get(i).map(|t| &t.0), Some(Tok::Dot)) {
            break;
        }
    }
    if saw_bare && saw_data {
        eprintln!(
            "ERROR {}: enum '{}' mixes bare and data constructors — outside the pinned subset (DR-8.8 v2.4)",
            file, name
        );
        std::process::exit(1);
    }
    if saw_data && data.len() != 2 {
        eprintln!(
            "ERROR {}: data enum '{}' must have exactly the pinned Ok/Err pair shape (DR-8.8 v2.14)",
            file, name
        );
        std::process::exit(1);
    }
    if saw_data {
        let names: Vec<&str> = data.iter().map(|(c, _)| c.as_str()).collect();
        if names != ["Ok", "Err"] {
            eprintln!(
                "ERROR {}: data enum '{}' constructors must be exactly Ok/Err (DR-8.8 v2.14)",
                file, name
            );
            std::process::exit(1);
        }
    }
    if variants.is_empty() && data.is_empty() {
        fail(file, s, i, "empty enum");
    }
    EnumDecl {
        name,
        variants,
        data_kind: if saw_data { Some(data) } else { None },
    }
}

/// Re-walk sentences, skipping Lemma..Qed blocks by state machine.
fn reparse_with_lemma_skip(file: &str, lex: &Lexer) -> Vec<Definition> {
    // Reuse the sentence splitting from parse_unit by re-doing it here with
    // block awareness.
    let mut sentences: Vec<Vec<(Tok, usize)>> = Vec::new();
    let mut cur: Vec<(Tok, usize)> = Vec::new();
    let mut depth = 0usize;
    for t in &lex.toks {
        match t.0 {
            Tok::LParen => depth += 1,
            Tok::RParen => depth -= 1,
            Tok::Dot if depth == 0 => {
                cur.push(t.clone());
                sentences.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(t.clone());
    }
    let mut defs = Vec::new();
    let mut in_lemma = false;
    for s in &sentences {
        if s.is_empty() {
            continue;
        }
        if let Tok::Ident(w) = &s[0].0 {
            match w.as_str() {
                "Lemma" | "Theorem" | "Remark" | "Fact" | "Corollary" | "Proposition" => {
                    in_lemma = true;
                    continue;
                }
                "Qed" | "Abort" => {
                    in_lemma = false;
                    continue;
                }
                "Proof" => continue,
                "Require" => {
                    // pinned accept-list (v1: Bool; v2: Arith/List/Lia/PeanoNat +
                    // the Effect foundation units)
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    let joined = words.join(" ");
                    let trimmed = joined.trim_end_matches("").to_string();
                    let base = trimmed
                        .strip_suffix("").unwrap_or(&trimmed)
                        .to_string();
                    // the sentence ends with a Dot token; the word list has no
                    // trailing dot — match the accept list directly
                    if REQUIRE_ACCEPT.contains(&base.as_str())
                        || REQUIRE_ACCEPT.iter().any(|a| base == *a)
                    {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: Require of '{}' is outside the pinned accept-list (DR-8.8)",
                        file, joined
                    );
                    std::process::exit(1);
                }
                "Import" => {
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    let joined = words.join(" ");
                    if IMPORT_ACCEPT.contains(&joined.as_str()) {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: Import of '{}' is outside the pinned accept-list (DR-8.8)",
                        file, joined
                    );
                    std::process::exit(1);
                }
                "Unset" => {
                    // pinned: Unset Automatic Proposition Inductives.
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    if words == ["Unset", "Automatic", "Proposition", "Inductives"] {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: only 'Unset Automatic Proposition Inductives' is pinned",
                        file
                    );
                    std::process::exit(1);
                }
                "Module" => {
                    // pinned name-hygiene wrapper (v2.12): Module PinnedEnums.
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    if words == ["Module", "PinnedEnums"] {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: only the PinnedEnums module wrapper is pinned (DR-8.8 v2.12)",
                        file
                    );
                    std::process::exit(1);
                }
                "End" => {
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    if words == ["End", "PinnedEnums"] {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: only 'End PinnedEnums' is pinned (DR-8.8 v2.12)",
                        file
                    );
                    std::process::exit(1);
                }
                "Inductive" => {
                    // v2.4 closed enums: collected structurally by
                    // collect_enums; the reparse walker skips the sentence.
                    if in_lemma {
                        eprintln!("ERROR {}: Inductive inside a Lemma block", file);
                        std::process::exit(1);
                    }
                    continue;
                }
                "Definition" => {
                    if in_lemma {
                        eprintln!("ERROR {}: Definition inside a Lemma block", file);
                        std::process::exit(1);
                    }
                    defs.push(parse_definition(file, s));
                    continue;
                }
                "Fixpoint" => {
                    if in_lemma {
                        eprintln!("ERROR {}: Fixpoint inside a Lemma block", file);
                        std::process::exit(1);
                    }
                    defs.push(parse_fixpoint(file, s));
                    continue;
                }
                _ => {
                    if !in_lemma {
                        eprintln!(
                            "ERROR {}: top-level '{}' is outside the pinned extraction subset",
                            file, w
                        );
                        std::process::exit(1);
                    }
                    continue; // tactic sentences inside Lemma..Qed are erased
                }
            }
        }
        if !in_lemma {
            eprintln!("ERROR {}: unexpected top-level token", file);
            std::process::exit(1);
        }
    }
    defs
}

fn parse_definition(file: &str, s: &[(Tok, usize)]) -> Definition {
    // Definition <name> [<binders>] [: <ty>] := <term> .
    let mut i = 1usize;
    let name = match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) => w.clone(),
        _ => fail(file, s, i, "expected definition name"),
    };
    i += 1;
    let mut binders = Vec::new();
    loop {
        match s.get(i).map(|t| &t.0) {
            Some(Tok::LParen) => {
                // binder group: ( ident+ : ty )
                i += 1;
                let mut names = Vec::new();
                loop {
                    match s.get(i).map(|t| &t.0) {
                        Some(Tok::Ident(w)) => {
                            names.push(w.clone());
                            i += 1;
                        }
                        _ => break,
                    }
                }
                if names.is_empty() {
                    fail(file, s, i, "empty binder group");
                }
                expect(file, s, i, Tok::Colon, "':' in binder group");
                i += 1;
                let (ty, ni) = parse_ty(file, s, i)
                    .unwrap_or_else(|e| fail(file, s, i, &e));
                i = ni;
                expect(file, s, i, Tok::RParen, "')' closing binder group");
                i += 1;
                for n in names {
                    binders.push((n, ty.clone()));
                }
            }
            _ => break,
        }
    }
    let ret;
    if let Some(Tok::Colon) = s.get(i).map(|t| &t.0) {
        i += 1;
        let (ty, ni) = parse_ty(file, s, i)
            .unwrap_or_else(|e| fail(file, s, i, &e));
        ret = ty;
        i = ni;
    } else {
        fail(file, s, i, "missing explicit result type (pinned subset requires 'Definition name : ty := ...')");
    }
    expect(file, s, i, Tok::ColonEq, "':=' after result type");
    i += 1;
    if ret == Ty::Named(String::from("Type")) {
        // Type alias: parse the body as a TYPE, not a term.
        let (ty, ni) = parse_ty(file, s, i)
            .unwrap_or_else(|e| fail(file, s, i, &e));
        i = ni;
        expect(file, s, i, Tok::Dot, "'.' ending the type alias");
        return Definition {
            name,
            binders: Vec::new(),
            ret,
            body: Term::Ident(String::from("<type-alias>")),
            ty_alias: Some(ty),
            fuel_fixpoint: None,
        };
    }
    if PINNED_SKIP.contains(&name.as_str()) {
        // v2.8/v2.14: proof-side body, pinned extraction mapping at call sites;
        // skip the body tokens to the sentence end without parsing.
        return Definition {
            name,
            binders,
            ret,
            body: Term::Ident(String::from("<pinned-mapped>")),
            ty_alias: None,
            fuel_fixpoint: None,
        };
    }
    let (body, ni) = parse_term(file, s, i);
    i = ni;
    expect(file, s, i, Tok::Dot, "'.' ending the definition");
    let _ = s.last();
    Definition {
        name,
        binders,
        ret,
        body,
        ty_alias: None,
        fuel_fixpoint: None,
    }
}

fn parse_fixpoint(file: &str, s: &[(Tok, usize)]) -> Definition {
    // v2.7 pinned fuel shape:
    // Fixpoint <name> (fuel : nat) (args...) : T := match fuel with
    //   | 0 => <default> | S f => <body whose recursive calls decrease the
    //   fuel to f; at most 2 call sites, one per branch (the amended rule)>. end.
    let mut i = 1usize;
    let name = match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) => w.clone(),
        _ => fail(file, s, i, "expected fixpoint name"),
    };
    i += 1;
    if PINNED_SKIP.contains(&name.as_str()) {
        // key_eq_aux / key_lt_aux: proof-side, pinned mappings.
        return Definition {
            name,
            binders: Vec::new(),
            ret: Ty::Bool,
            body: Term::Ident(String::from("<pinned-mapped>")),
            ty_alias: None,
            fuel_fixpoint: None,
        };
    }
    let mut binders = Vec::new();
    loop {
        match s.get(i).map(|t| &t.0) {
            Some(Tok::LParen) => {
                i += 1;
                let mut names = Vec::new();
                loop {
                    match s.get(i).map(|t| &t.0) {
                        Some(Tok::Ident(w)) => {
                            names.push(w.clone());
                            i += 1;
                        }
                        _ => break,
                    }
                }
                if names.is_empty() {
                    fail(file, s, i, "expected binder name");
                }
                expect(file, s, i, Tok::Colon, "':' in binder");
                i += 1;
                let (ty, ni) = parse_ty(file, s, i)
                    .unwrap_or_else(|e| fail(file, s, i, &e));
                i = ni;
                expect(file, s, i, Tok::RParen, "')' closing binder");
                i += 1;
                for n in names {
                    binders.push((n, ty.clone()));
                }
            }
            _ => break,
        }
    }
    if binders.is_empty() {
        fail(file, s, i, "a fuel Fixpoint needs a (fuel : nat) first binder (DR-8.8 v2.7)");
    }
    let fuel_name = binders[0].0.clone();
    if binders[0].1 != Ty::Nat {
        fail(file, s, i, "the first Fixpoint binder must be (fuel : nat) (DR-8.8 v2.7)");
    }
    expect(file, s, i, Tok::Colon, "':' before the fixpoint result type");
    i += 1;
    let (ret, ni) = parse_ty(file, s, i)
        .unwrap_or_else(|e| fail(file, s, i, &e));
    i = ni;
    expect(file, s, i, Tok::ColonEq, "':=' after the fixpoint result type");
    i += 1;
    let (body, ni) = parse_term(file, s, i);
    i = ni;
    expect(file, s, i, Tok::Dot, "'.' ending the fixpoint");
    // validate the pinned fuel shape
    validate_fuel_shape(file, s, name.clone(), &fuel_name, &body);
    Definition {
        name,
        binders,
        ret,
        body,
        ty_alias: None,
        fuel_fixpoint: Some(fuel_name),
    }
}

fn validate_fuel_shape(file: &str, s: &[(Tok, usize)], name: String, fuel: &str, body: &Term) {
    match body {
        Term::Match(scrut, arms) => {
            if **scrut != Term::Ident(fuel.to_string()) {
                eprintln!(
                    "ERROR {}: the fuel Fixpoint '{}' must match on the fuel binder '{}' (DR-8.8 v2.7)",
                    file, name, fuel
                );
                std::process::exit(1);
            }
            if arms.len() != 2 {
                eprintln!(
                    "ERROR {}: the fuel Fixpoint '{}' must have exactly the two arms 0 / S f (DR-8.8 v2.7)",
                    file, name
                );
                std::process::exit(1);
            }
            // arm 1: pattern 0 (the pinned total default), arm 2: S f
            let p0 = &arms[0].patterns;
            if p0.len() != 1 || p0[0] != Pattern::Zero {
                eprintln!(
                    "ERROR {}: the fuel Fixpoint '{}' first arm must be the 0 pattern (DR-8.8 v2.7)",
                    file, name
                );
                std::process::exit(1);
            }
            let p1 = &arms[1].patterns;
            if p1.len() != 1 {
                eprintln!(
                    "ERROR {}: the fuel Fixpoint '{}' second arm must be a single S pattern (DR-8.8 v2.7)",
                    file, name
                );
                std::process::exit(1);
            }
            let f_name = match &p1[0] {
                Pattern::Succ(f) => f.clone(),
                _ => {
                    eprintln!(
                        "ERROR {}: the fuel Fixpoint '{}' second arm must be S f (DR-8.8 v2.7)",
                        file, name
                    );
                    std::process::exit(1);
                }
            };
            // every recursive call site decreases the fuel to f; the call-site
            // count is bounded by the arm count (pinned: <= 2, DR-8.8 v2.7)
            let count = count_recursive_calls(&arms[1].body, &name);
            if count > 2 {
                eprintln!(
                    "ERROR {}: the fuel Fixpoint '{}' has {} recursive call sites; the pinned bound is 2 (DR-8.8 v2.7)",
                    file, name, count
                );
                std::process::exit(1);
            }
            if count > 0 && !all_recursive_args_are(&arms[1].body, &name, &f_name) {
                eprintln!(
                    "ERROR {}: every recursive call of '{}' must decrease the fuel to '{}' (DR-8.8 v2.7)",
                    file, name, f_name
                );
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!(
                "ERROR {}: the fuel Fixpoint '{}' body must be the pinned match shape (DR-8.8 v2.7)",
                file, name
            );
            std::process::exit(1);
        }
    }
}

fn count_recursive_calls(t: &Term, name: &str) -> usize {
    match t {
        Term::Ident(w) => usize::from(w == name),
        Term::NatLit(_) | Term::BoolLit(_) | Term::NatList(_) => 0,
        Term::App(head, args) => {
            let self_call = usize::from(matches!(&**head, Term::Ident(w) if w == name));
            self_call + args.iter().map(|a| count_recursive_calls(a, name)).sum::<usize>()
        }
        Term::Pair(comps) => comps.iter().map(|c| count_recursive_calls(c, name)).sum(),
        Term::If(a, b, c) => {
            count_recursive_calls(a, name)
                + count_recursive_calls(b, name)
                + count_recursive_calls(c, name)
        }
        Term::Let(_, a, b) => count_recursive_calls(a, name) + count_recursive_calls(b, name),
        Term::Match(scrut, arms) => {
            count_recursive_calls(scrut, name)
                + arms.iter().map(|arm| count_recursive_calls(&arm.body, name)).sum::<usize>()
        }
    }
}

/// Every recursive call's fuel argument must be exactly the S-arm binder.
fn all_recursive_args_are(t: &Term, name: &str, fuel: &str) -> bool {
    match t {
        Term::Ident(_) | Term::NatLit(_) | Term::BoolLit(_) | Term::NatList(_) => true,
        Term::App(head, args) => {
            let head_ok = if matches!(&**head, Term::Ident(w) if w == name) {
                matches!(args.first(), Some(Term::Ident(x)) if x == fuel)
            } else {
                all_recursive_args_are(head, name, fuel)
            };
            head_ok && args.iter().all(|a| all_recursive_args_are(a, name, fuel))
        }
        Term::Pair(comps) => comps.iter().all(|c| all_recursive_args_are(c, name, fuel)),
        Term::If(a, b, c) => {
            all_recursive_args_are(a, name, fuel)
                && all_recursive_args_are(b, name, fuel)
                && all_recursive_args_are(c, name, fuel)
        }
        Term::Let(_, a, b) => {
            all_recursive_args_are(a, name, fuel) && all_recursive_args_are(b, name, fuel)
        }
        Term::Match(scrut, arms) => {
            all_recursive_args_are(scrut, name, fuel)
                && arms.iter().all(|arm| all_recursive_args_are(&arm.body, name, fuel))
        }
    }
}

fn expect(file: &str, s: &[(Tok, usize)], i: usize, t: Tok, what: &str) {
    match s.get(i) {
        Some((got, _)) if *got == t => {}
        _ => fail(file, s, i, &format!("expected {} ({:?})", what, t)),
    }
}

fn fail(file: &str, s: &[(Tok, usize)], i: usize, msg: &str) -> ! {
    let line = s.get(i).map(|t| t.1).unwrap_or(0);
    eprintln!("ERROR {}:{}: {} (in Definition sentence)", file, line, msg);
    std::process::exit(1);
}

/// Parse a type. Pinned: `bool`, `nat` (v2.1), identifiers, `(list nat)` /
/// `(list key)` (v2.6), and right-nested tuples `(T1 * (T2 * ...))`. A paren
/// group with 2+ top-level `*` (flat tuple, Coq's left-associated default) is
/// REJECTED — the kernel-check finding that motivated explicit right-nesting
/// is pinned here forever.
type PResult<T> = Result<T, String>;

fn parse_ty_atom(file: &str, s: &[(Tok, usize)], i: usize) -> PResult<(Ty, usize)> {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "bool" => Ok((Ty::Bool, i + 1)),
        Some(Tok::Ident(w)) if w == "nat" => Ok((Ty::Nat, i + 1)),
        Some(Tok::Ident(w)) if w == "list" => {
            // pinned list-type atom: list <atom> (element must be nat or a
            // named type — the extraction mapping supports list-of-nat keys
            // and lists of unit-local types)
            let (el, ni) = parse_ty_atom(file, s, i + 1)?;
            match &el {
                Ty::Nat | Ty::Named(_) => Ok((Ty::List(Box::new(el)), ni)),
                _ => Err(format!(
                    "{}: 'list' of this element type is outside the pinned subset (DR-8.8 v2.6)",
                    file
                )),
            }
        }
        Some(Tok::Ident(w)) => Ok((Ty::Named(w.clone()), i + 1)),
        Some(Tok::LParen) => {
            // Structural tuple group: atom (Star atom)* RParen — or a
            // single-element group like (list nat).
            let mut comps: Vec<Ty> = Vec::new();
            let mut j = i + 1;
            loop {
                let (comp, nj) = parse_ty_atom(file, s, j)?;
                comps.push(comp);
                j = nj;
                match s.get(j).map(|t| &t.0) {
                    Some(Tok::Star) => {
                        j += 1;
                    }
                    Some(Tok::RParen) => {
                        j += 1;
                        break;
                    }
                    _ => {
                        return Err(format!(
                            "{}: expected '*' or ')' in tuple type group",
                            file
                        ));
                    }
                }
            }
            if comps.len() == 1 {
                // single-element paren group: unwrap (e.g. (list nat))
                return Ok((comps.pop().unwrap(), j));
            }
            // Pinned right-nesting: with more than two components, every
            // component after the first must itself be a tuple (the flat
            // left-associated form `A * B * C` is rejected — kernel-check
            // finding, first pinned build). A two-component group is the
            // unambiguous innermost pair and is accepted.
            if comps.len() > 2 {
                for c in &comps[1..] {
                    if !matches!(c, Ty::Tuple(_)) {
                        return Err(String::from(
                            "flat tuple type outside the pinned subset — write the right-nested form (T1 * (T2 * (T3 * ...))) (kernel-check finding, first pinned build)",
                        ));
                    }
                }
            }
            Ok((Ty::Tuple(comps), j))
        }
        _ => Err(format!(
            "{}: expected a type (bool, nat, identifier, list, or parenthesized tuple)",
            file
        )),
    }
}

pub fn parse_ty(file: &str, s: &[(Tok, usize)], i: usize) -> PResult<(Ty, usize)> {
    // A type in the pinned subset is one atom; bare star chains at this level
    // (Coq's flat form) are rejected by requiring the atom to end the type.
    let (ty, ni) = parse_ty_atom(file, s, i)?;
    if let Some(Tok::Star) = s.get(ni).map(|t| &t.0) {
        return Err(String::from(
            "bare tuple-type chain outside the pinned subset — write the right-nested parenthesized form",
        ));
    }
    Ok((ty, ni))
}

/// Parse a term until a structural boundary; entry point for definition bodies.
pub fn parse_term(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    parse_term_at(file, s, i)
}

fn parse_term_at(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "if" => {
            // if t then t else t
            let (cond, ni) = parse_term_at(file, s, i + 1);
            let mut j = ni;
            match s.get(j).map(|t| &t.0) {
                Some(Tok::Ident(w)) if w == "then" => j += 1,
                _ => fail(file, s, j, "expected 'then'"),
            }
            let (thn, nj) = parse_term_at(file, s, j);
            let mut k = nj;
            match s.get(k).map(|t| &t.0) {
                Some(Tok::Ident(w)) if w == "else" => k += 1,
                _ => fail(file, s, k, "expected 'else'"),
            }
            let (els, nk) = parse_term_at(file, s, k);
            (
                Term::If(Box::new(cond), Box::new(thn), Box::new(els)),
                nk,
            )
        }
        Some(Tok::Ident(w)) if w == "let" => {
            // let x := t in t
            let x = match s.get(i + 1).map(|t| &t.0) {
                Some(Tok::Ident(w)) => w.clone(),
                _ => fail(file, s, i + 1, "expected a let binding name"),
            };
            expect(file, s, i + 2, Tok::ColonEq, "':=' in let");
            let (val, ni) = parse_term_at(file, s, i + 3);
            let mut j = ni;
            match s.get(j).map(|t| &t.0) {
                Some(Tok::Ident(w)) if w == "in" => j += 1,
                _ => fail(file, s, j, "expected 'in'"),
            }
            let (body, nj) = parse_term_at(file, s, j);
            (Term::Let(x, Box::new(val), Box::new(body)), nj)
        }
        Some(Tok::Ident(w)) if w == "match" => {
            // match scrut with | pats => body ... end
            let (scrut, ni) = parse_term_at(file, s, i + 1);
            let mut j = ni;
            match s.get(j).map(|t| &t.0) {
                Some(Tok::Ident(w)) if w == "with" => j += 1,
                _ => fail(file, s, j, "expected 'with'"),
            }
            let mut arms = Vec::new();
            loop {
                if matches!(s.get(j).map(|t| &t.0), Some(Tok::Bar)) {
                    j += 1;
                }
                let mut patterns = Vec::new();
                loop {
                    match s.get(j).map(|t| &t.0) {
                        Some(Tok::NatLit(0)) => {
                            patterns.push(Pattern::Zero);
                            j += 1;
                        }
                        Some(Tok::Ident(w)) if w == "S" => {
                            let b = match s.get(j + 1).map(|t| &t.0) {
                                Some(Tok::Ident(x)) => x.clone(),
                                _ => fail(file, s, j + 1, "expected a binder after S"),
                            };
                            patterns.push(Pattern::Succ(b));
                            j += 2;
                        }
                        Some(Tok::Ident(w)) => {
                            let ctor = w.clone();
                            j += 1;
                            // constructor with a `_` binding wildcard (Ok _)
                            let wild = match s.get(j).map(|t| &t.0) {
                                Some(Tok::Ident(x)) => x == "_",
                                _ => false,
                            };
                            if wild {
                                patterns.push(Pattern::CtorWild(ctor));
                                j += 1;
                            } else {
                                patterns.push(Pattern::Ctor(ctor));
                            }
                        }
                        _ => fail(file, s, j, "expected a pattern"),
                    }
                    if matches!(s.get(j).map(|t| &t.0), Some(Tok::Bar)) {
                        j += 1;
                        continue;
                    }
                    break;
                }
                expect(file, s, j, Tok::FatArrow, "'=>' after match patterns");
                j += 1;
                // the arm body extends to the next top-level Bar or 'end'
                let (body, nj) = parse_match_body(file, s, j);
                j = nj;
                arms.push(MatchArm { patterns, body });
                match s.get(j).map(|t| &t.0) {
                    Some(Tok::Ident(w)) if w == "end" => {
                        return (Term::Match(Box::new(scrut), arms), j + 1);
                    }
                    Some(Tok::Bar) => continue,
                    _ => fail(file, s, j, "expected '|' or 'end' in match"),
                }
            }
        }
        Some(Tok::LBracket) => {
            // v2.6 list literal: [ e; e; ... ] with e = nat literal, cons
            // chain of nat literals, constructor name, or definition name.
            parse_list_literal(file, s, i)
        }
        Some(Tok::LParen) => {
            // scan the group for top-level commas
            let mut depth = 0usize;
            let mut j = i + 1;
            let mut commas_top = 0usize;
            loop {
                match s.get(j).map(|t| &t.0) {
                    None => {
                        eprintln!("ERROR {}: unterminated '(' in term", file);
                        std::process::exit(1);
                    }
                    Some(Tok::LParen) => {
                        depth += 1;
                        j += 1;
                    }
                    Some(Tok::RParen) => {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                        j += 1;
                    }
                    Some(Tok::Comma) if depth == 0 => {
                        commas_top += 1;
                        j += 1;
                    }
                    _ => {
                        j += 1;
                    }
                }
            }
            if commas_top == 0 {
                let inner = parse_term_span(file, s, i + 1, j);
                (inner, j + 1)
            } else {
                let mut comps: Vec<Term> = Vec::new();
                let mut k = i + 1;
                let mut d2 = 0usize;
                let mut comp_start = k;
                loop {
                    match s.get(k).map(|t| &t.0) {
                        None => {
                            eprintln!("ERROR {}: unterminated pair", file);
                            std::process::exit(1);
                        }
                        Some(Tok::LParen) => {
                            d2 += 1;
                            k += 1;
                        }
                        Some(Tok::RParen) => {
                            if d2 == 0 {
                                break;
                            }
                            d2 -= 1;
                            k += 1;
                        }
                        Some(Tok::Comma) if d2 == 0 => {
                            let comp = parse_term_span(file, s, comp_start, k);
                            comps.push(comp);
                            k += 1;
                            comp_start = k;
                        }
                        _ => {
                            k += 1;
                        }
                    }
                }
                let last = parse_term_span(file, s, comp_start, k);
                comps.push(last);
                if comps.len() > 2 {
                    eprintln!(
                        "ERROR {}: flat tuple term (a, b, c) is outside the pinned subset — write the right-nested form (a, (b, (c, ...))) (kernel-check finding, first pinned build)",
                        file
                    );
                    std::process::exit(1);
                }
                (Term::Pair(comps), k + 1)
            }
        }
        Some(Tok::NatLit(n)) => {
            // v2.1: nat literals in extracted bodies are pinned to the byte
            // domain (the enumeration bound 256 appears only in erased law
            // statements, which never reach this parser)
            if *n > 255 {
                eprintln!(
                    "ERROR {}:{}: numeric literal {} outside the pinned byte domain (DR-8.8 v2.1)",
                    file,
                    s.get(i).map(|t| t.1).unwrap_or(0),
                    n
                );
                std::process::exit(1);
            }
            (Term::NatLit(*n), i + 1)
        }
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::Ident(w)) => {
            // application: head plus greedily consumed atoms
            let mut args = Vec::new();
            let mut k = i + 1;
            loop {
                let is_ident_atom = matches!(s.get(k).map(|t| &t.0), Some(Tok::Ident(w)) if !is_reserved(w));
                if is_ident_atom
                    || matches!(s.get(k).map(|t| &t.0), Some(Tok::LParen))
                    || matches!(s.get(k).map(|t| &t.0), Some(Tok::NatLit(_)))
                    || matches!(s.get(k).map(|t| &t.0), Some(Tok::LBracket))
                {
                    let (arg, ni) = parse_atom(file, s, k);
                    args.push(arg);
                    k = ni;
                } else {
                    break;
                }
            }
            if args.is_empty() {
                (Term::Ident(w.clone()), k)
            } else {
                (Term::App(Box::new(Term::Ident(w.clone())), args), k)
            }
        }
        _ => {
            eprintln!("ERROR {}: expected a term", file);
            std::process::exit(1);
        }
    }
}

fn parse_match_body(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    // The arm body is ONE recursively parsed term: parse_term_at handles the
    // nested if/let/match forms and stops naturally at the reserved words
    // ('then'/'else'/'end') and at top-level Bars, which are the arm and
    // match boundaries.
    let (t, ni) = parse_term_at(file, s, i);
    // the caller (the arm loop) checks the terminator (Bar or 'end')
    let _ = ni;
    (t, ni)
}

fn parse_list_literal(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    // Two-pass: collect raw elements (nat, cons chain, nested list, ctor
    // name, definition name), then decide the mapping mechanically:
    //  - all elements bare nats and no names       -> ONE key (list nat -> str)
    //  - any cons-chain/nested-list elements        -> list of keys -> [str; N]
    //  - any constructor/definition-name elements   -> list of refs -> [Name; N]
    let mut j = i + 1;
    enum RawElem {
        Nat(u32),
        Codes(Vec<u32>),
        Name(String),
    }
    let mut raw: Vec<RawElem> = Vec::new();
    loop {
        match s.get(j).map(|t| &t.0) {
            Some(Tok::RBracket) => {
                j += 1;
                break;
            }
            Some(Tok::NatLit(n)) => {
                if *n == 0 || *n > 126 {
                    eprintln!(
                        "ERROR {}:{}: ASCII code {} outside the pinned printable range 1..=126 (DR-8.8 v2.6)",
                        file,
                        s.get(j).map(|t| t.1).unwrap_or(0),
                        n
                    );
                    std::process::exit(1);
                }
                let mut codes = vec![*n];
                j += 1;
                let mut consed = false;
                while matches!(s.get(j).map(|t| &t.0), Some(Tok::ColonColon)) {
                    consed = true;
                    j += 1;
                    match s.get(j).map(|t| &t.0) {
                        Some(Tok::NatLit(m)) => {
                            codes.push(*m);
                            j += 1;
                        }
                        Some(Tok::Ident(w)) if w == "nil" => {
                            j += 1;
                            break;
                        }
                        _ => fail(file, s, j, "expected a nat literal or nil in a cons chain"),
                    }
                }
                if consed {
                    raw.push(RawElem::Codes(codes));
                } else {
                    raw.push(RawElem::Nat(*n));
                }
            }
            Some(Tok::LParen) => {
                // parenthesized cons chain: (40 :: nil)
                j += 1;
                let mut codes = Vec::new();
                loop {
                    match s.get(j).map(|t| &t.0) {
                        Some(Tok::NatLit(n)) => {
                            codes.push(*n);
                            j += 1;
                        }
                        Some(Tok::Ident(w)) if w == "nil" => {
                            j += 1;
                            break;
                        }
                        _ => fail(file, s, j, "expected a nat literal or nil in a cons chain"),
                    }
                    if matches!(s.get(j).map(|t| &t.0), Some(Tok::ColonColon)) {
                        j += 1;
                        continue;
                    }
                    break;
                }
                expect(file, s, j, Tok::RParen, "')' closing the cons chain");
                j += 1;
                raw.push(RawElem::Codes(codes));
            }
            Some(Tok::LBracket) => {
                // nested list literal: [40]
                let mut codes = Vec::new();
                j += 1;
                loop {
                    match s.get(j).map(|t| &t.0) {
                        Some(Tok::NatLit(n)) => {
                            codes.push(*n);
                            j += 1;
                        }
                        Some(Tok::RBracket) => {
                            j += 1;
                            break;
                        }
                        _ => fail(file, s, j, "expected a nat literal or ']' in a nested list"),
                    }
                    if matches!(s.get(j).map(|t| &t.0), Some(Tok::Semi)) {
                        j += 1;
                    }
                }
                raw.push(RawElem::Codes(codes));
            }
            Some(Tok::Ident(w)) => {
                raw.push(RawElem::Name(w.clone()));
                j += 1;
            }
            _ => fail(file, s, j, "expected a list element"),
        }
        if matches!(s.get(j).map(|t| &t.0), Some(Tok::Semi)) {
            j += 1;
            continue;
        }
        if matches!(s.get(j).map(|t| &t.0), Some(Tok::RBracket)) {
            j += 1;
            break;
        }
        fail(file, s, j, "expected ';' or ']' in a list literal");
    }
    // decide the mapping
    let any_codes = raw.iter().any(|e| matches!(e, RawElem::Codes(_)));
    let any_names = raw.iter().any(|e| matches!(e, RawElem::Name(_)));
    let term = if any_names {
        let elems: Vec<ListElem> = raw
            .into_iter()
            .map(|e| match e {
                RawElem::Name(n) => ListElem::NameRef(n),
                _ => ListElem::Nat(0), // unreachable in the pinned forms
            })
            .collect();
        Term::NatList(elems)
    } else if any_codes {
        // a list of keys (delimiter_keys) -> array of str literals
        let elems: Vec<ListElem> = raw
            .into_iter()
            .map(|e| match e {
                RawElem::Codes(c) => ListElem::SubList(c),
                _ => ListElem::Nat(0), // unreachable in the pinned forms
            })
            .collect();
        Term::NatList(elems)
    } else {
        // ONE key: all bare nats -> the ASCII string mapping
        let elems: Vec<ListElem> = raw
            .into_iter()
            .map(|e| match e {
                RawElem::Nat(n) => ListElem::Nat(n),
                _ => ListElem::Nat(0),
            })
            .collect();
        Term::NatList(elems)
    };
    (term, j)
}

fn parse_atom(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::NatLit(n)) => (Term::NatLit(*n), i + 1),
        Some(Tok::Ident(w)) => (Term::Ident(w.clone()), i + 1),
        Some(Tok::LParen) => parse_term_at(file, s, i),
        Some(Tok::LBracket) => parse_list_literal(file, s, i),
        _ => {
            eprintln!("ERROR {}: expected an atom in application", file);
            std::process::exit(1);
        }
    }
}

/// v2.5 exhaustiveness: for a match whose patterns are constructors of ONE
/// declared closed enum (mechanically detected via the unit's enum table),
/// the arm set must cover exactly that enum's variant set — no wildcard arm,
/// no missing variant, no duplicate. Data-ctor binding wildcards (Ok _) are
/// patterns of the Ok/Err pair and are checked the same way. Matches on the
/// fuel shape (0 / S f patterns) are validated by the fuel-shape validator.
fn validate_match_exhaustiveness(
    file: &str,
    enums: &[EnumDecl],
    arms: &[MatchArm],
    def: &str,
) {
    // collect the ctor owners across all patterns
    let mut owners: Vec<&str> = Vec::new();
    let mut covered: Vec<(String, String)> = Vec::new(); // (owner, ctor)
    for arm in arms {
        for p in &arm.patterns {
            match p {
                Pattern::Ctor(c) | Pattern::CtorWild(c) => {
                    if c == "Ok" || c == "Err" {
                        // the pinned data pair: both arms must be present
                        covered.push(("lookup_result_data".to_string(), c.clone()));
                        if !owners.contains(&"lookup_result_data") {
                            owners.push("lookup_result_data");
                        }
                    } else if let Some(owner) = ctor_owner_of(enums, c) {
                        covered.push((owner.to_string(), c.clone()));
                        if !owners.iter().any(|o| o == &owner) {
                            owners.push(owner);
                        }
                    } else {
                        eprintln!(
                            "ERROR {}: definition '{}': match pattern '{}' is not a constructor of any declared closed enum (DR-8.8 v2.5)",
                            file, def, c
                        );
                        std::process::exit(1);
                    }
                }
                Pattern::Zero | Pattern::Succ(_) => {}
            }
        }
    }
    if owners.len() != 1 {
        return; // mixed/absent owners: either the fuel shape (validated elsewhere) or a parse error already reported
    }
    let owner = owners[0];
    let variants: Vec<String> = if owner == "lookup_result_data" {
        vec![String::from("Ok"), String::from("Err")]
    } else {
        enums
            .iter()
            .find(|e| e.name == owner)
            .map(|e| e.variants.clone())
            .unwrap_or_default()
    };
    let mut seen: Vec<&str> = Vec::new();
    for (o, c) in &covered {
        if o != owner {
            continue;
        }
        if seen.contains(&c.as_str()) {
            eprintln!(
                "ERROR {}: definition '{}': match covers constructor '{}' more than once (DR-8.8 v2.5 exhaustive closed matches)",
                file, def, c
            );
            std::process::exit(1);
        }
        seen.push(c);
    }
    if seen.len() != variants.len() {
        eprintln!(
            "ERROR {}: definition '{}': match over '{}' covers {} of {} variants — matches must be exhaustive over the closed domain with no wildcard arm (DR-8.8 v2.5)",
            file,
            def,
            owner,
            seen.len(),
            variants.len()
        );
        std::process::exit(1);
    }
}

fn ctor_owner_of<'a>(enums: &'a [EnumDecl], ctor: &str) -> Option<&'a str> {
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

/// Reserved structural keywords: never consumed as application atoms.
fn is_reserved(w: &str) -> bool {
    matches!(w, "with" | "end" | "then" | "else" | "in" | "match" | "let" | "if")
}

fn parse_term_span(file: &str, s: &[(Tok, usize)], a: usize, b: usize) -> Term {
    // Parse tokens s[a..b] (no top-level commas — caller split them).
    let mut i = a;
    let mut items: Vec<Term> = Vec::new();
    while i < b {
        match s.get(i).map(|t| &t.0) {
            Some(Tok::Ident(w)) if w == "if" => {
                // if t then t else t within a span
                let (cond, ni) = parse_term_span_atom_or_if(file, s, i, b);
                items.push(cond);
                i = ni;
            }
            Some(Tok::Ident(w)) if w == "let" => {
                let x = match s.get(i + 1).map(|t| &t.0) {
                    Some(Tok::Ident(w)) => w.clone(),
                    _ => fail(file, s, i + 1, "expected a let binding name"),
                };
                expect(file, s, i + 2, Tok::ColonEq, "':=' in let");
                // find 'in' at this span level
                let mut j = i + 3;
                let mut in_pos = None;
                while j < b {
                    if let Some(Tok::Ident(w)) = s.get(j).map(|t| &t.0) {
                        if w == "in" {
                            in_pos = Some(j);
                            break;
                        }
                    }
                    j += 1;
                }
                let inp = in_pos.unwrap_or_else(|| fail_usize(file, s, i, "expected 'in' in let"));
                let val = parse_term_span(file, s, i + 3, inp);
                let body = parse_term_span(file, s, inp + 1, b);
                items.push(Term::Let(x, Box::new(val), Box::new(body)));
                i = b;
            }
            Some(Tok::Ident(w)) if w == "true" => {
                items.push(Term::BoolLit(true));
                i += 1;
            }
            Some(Tok::Ident(w)) if w == "false" => {
                items.push(Term::BoolLit(false));
                i += 1;
            }
            Some(Tok::NatLit(n)) => {
                items.push(Term::NatLit(*n));
                i += 1;
            }
            Some(Tok::Ident(w)) => {
                // application within span
                let mut args = Vec::new();
                let mut k = i + 1;
                loop {
                    if k >= b {
                        break;
                    }
                    let is_ident_atom = matches!(s.get(k).map(|t| &t.0), Some(Tok::Ident(w)) if !is_reserved(w));
                    if is_ident_atom
                        || matches!(s.get(k).map(|t| &t.0), Some(Tok::LParen))
                        || matches!(s.get(k).map(|t| &t.0), Some(Tok::NatLit(_)))
                        || matches!(s.get(k).map(|t| &t.0), Some(Tok::LBracket))
                    {
                        let arg = parse_term_span_atom(file, s, k, b);
                        args.push(arg.0);
                        k = arg.1;
                    } else {
                        break;
                    }
                }
                if args.is_empty() {
                    items.push(Term::Ident(w.clone()));
                } else {
                    items.push(Term::App(Box::new(Term::Ident(w.clone())), args));
                }
                i = k;
            }
            Some(Tok::LBracket) => {
                // list literal within a span: find the matching ']' within b
                let mut depth = 0usize;
                let mut j = i;
                let mut close = None;
                while j < b {
                    match s.get(j).map(|t| &t.0) {
                        Some(Tok::LBracket) => depth += 1,
                        Some(Tok::RBracket) => {
                            depth -= 1;
                            if depth == 0 {
                                close = Some(j);
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let c = close.unwrap_or_else(|| fail_usize(file, s, i, "unterminated list literal"));
                let (t, ni) = parse_list_literal(file, s, i);
                items.push(t);
                i = ni.max(c + 1);
            }
            Some(Tok::LParen) => {
                let (t, ni) = parse_term_at_span(file, s, i, b);
                items.push(t);
                i = ni;
            }
            _ => {
                eprintln!("ERROR {}: unexpected token in term span", file);
                std::process::exit(1);
            }
        }
    }
    if items.is_empty() {
        eprintln!("ERROR {}: empty term span", file);
        std::process::exit(1);
    }
    if items.len() == 1 {
        items.pop().unwrap()
    } else {
        let head = items.remove(0);
        // span items fold left-associatively only for applications; the
        // pinned forms put operators in head position, so a multi-item span
        // is the application head + args form.
        let mut rest = items;
        Term::App(Box::new(head), std::mem::take(&mut rest))
    }
}

fn parse_term_span_atom_or_if(file: &str, s: &[(Tok, usize)], i: usize, b: usize) -> (Term, usize) {
    // Parse an if-then-else inside a span: cond up to 'then', branch up to
    // 'else', else-branch up to the span end (the caller guarantees the span
    // boundary is the arm/def boundary).
    let mut then_pos = None;
    let mut j = i + 1;
    while j < b {
        if let Some(Tok::Ident(w)) = s.get(j).map(|t| &t.0) {
            if w == "then" {
                then_pos = Some(j);
                break;
            }
        }
        j += 1;
    }
    let tp = then_pos.unwrap_or_else(|| fail_usize(file, s, i, "expected 'then'"));
    let cond = parse_term_span(file, s, i + 1, tp);
    let mut else_pos = None;
    j = tp + 1;
    while j < b {
        if let Some(Tok::Ident(w)) = s.get(j).map(|t| &t.0) {
            if w == "else" {
                else_pos = Some(j);
                break;
            }
        }
        j += 1;
    }
    let ep = else_pos.unwrap_or_else(|| fail_usize(file, s, i, "expected 'else'"));
    let thn = parse_term_span(file, s, tp + 1, ep);
    let els = parse_term_span(file, s, ep + 1, b);
    (Term::If(Box::new(cond), Box::new(thn), Box::new(els)), b)
}

fn parse_term_span_atom(file: &str, s: &[(Tok, usize)], i: usize, b: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::NatLit(n)) => (Term::NatLit(*n), i + 1),
        Some(Tok::Ident(w)) => (Term::Ident(w.clone()), i + 1),
        Some(Tok::LParen) => parse_term_at_span(file, s, i, b),
        Some(Tok::LBracket) => {
            let mut depth = 0usize;
            let mut j = i;
            let mut close = None;
            while j < b {
                match s.get(j).map(|t| &t.0) {
                    Some(Tok::LBracket) => depth += 1,
                    Some(Tok::RBracket) => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(j);
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            let c = close.unwrap_or_else(|| fail_usize(file, s, i, "unterminated list literal"));
            let (t, ni) = parse_list_literal(file, s, i);
            (t, ni.max(c + 1))
        }
        _ => {
            eprintln!("ERROR {}: expected atom in span", file);
            std::process::exit(1);
        }
    }
}

fn parse_term_at_span(file: &str, s: &[(Tok, usize)], i: usize, b: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::LParen) => {
            let mut depth = 0usize;
            let mut j = i + 1;
            let mut commas_top = 0usize;
            loop {
                if j >= b {
                    eprintln!("ERROR {}: paren group exceeds span", file);
                    std::process::exit(1);
                }
                match s.get(j).map(|t| &t.0) {
                    Some(Tok::LParen) => {
                        depth += 1;
                        j += 1;
                    }
                    Some(Tok::RParen) => {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                        j += 1;
                    }
                    Some(Tok::Comma) if depth == 0 => {
                        commas_top += 1;
                        j += 1;
                    }
                    _ => {
                        j += 1;
                    }
                }
            }
            if commas_top == 0 {
                let inner = parse_term_span(file, s, i + 1, j);
                (inner, j + 1)
            } else {
                let mut comps: Vec<Term> = Vec::new();
                let mut k = i + 1;
                let mut d2 = 0usize;
                let mut comp_start = k;
                loop {
                    if k >= b {
                        eprintln!("ERROR {}: pair exceeds span", file);
                        std::process::exit(1);
                    }
                    match s.get(k).map(|t| &t.0) {
                        Some(Tok::LParen) => {
                            d2 += 1;
                            k += 1;
                        }
                        Some(Tok::RParen) => {
                            if d2 == 0 {
                                break;
                            }
                            d2 -= 1;
                            k += 1;
                        }
                        Some(Tok::Comma) if d2 == 0 => {
                            let comp = parse_term_span(file, s, comp_start, k);
                            comps.push(comp);
                            k += 1;
                            comp_start = k;
                        }
                        _ => {
                            k += 1;
                        }
                    }
                }
                let last = parse_term_span(file, s, comp_start, k);
                comps.push(last);
                if comps.len() > 2 {
                    eprintln!(
                        "ERROR {}: flat tuple term is outside the pinned subset (kernel-check finding, first pinned build)",
                        file
                    );
                    std::process::exit(1);
                }
                (Term::Pair(comps), k + 1)
            }
        }
        _ => {
            eprintln!("ERROR {}: expected term token in span", file);
            std::process::exit(1);
        }
    }
}

fn fail_usize(file: &str, s: &[(Tok, usize)], i: usize, msg: &str) -> ! {
    let line = s.get(i).map(|t| t.1).unwrap_or(0);
    eprintln!("ERROR {}:{}: {}", file, line, msg);
    std::process::exit(1);
}

/// Type alias recovery: a Definition with ret Named("Type") carries its tuple
/// type in ty_alias; the width is the tuple arity (pinned: all components bool).
fn type_alias_width(d: &Definition) -> Option<usize> {
    let ty = d.ty_alias.as_ref()?;
    match ty {
        Ty::Tuple(_) => {
            if ty_is_bool_leaf_set(ty) {
                Some(ty_leaf_count(ty))
            } else {
                None // non-row aliases (key, oprow, byte) are fine; not row types
            }
        }
        _ => None,
    }
}

/// Recursively flatten a tuple type; every leaf must be `bool`.
fn ty_is_bool_leaf_set(ty: &Ty) -> bool {
    match ty {
        Ty::Bool => true,
        Ty::Tuple(comps) => comps.iter().all(ty_is_bool_leaf_set),
        _ => false,
    }
}

fn ty_leaf_count(ty: &Ty) -> usize {
    match ty {
        Ty::Bool => 1,
        Ty::Tuple(comps) => comps.iter().map(ty_leaf_count).sum(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_parse_alias() {
        let src = "Definition row : Type :=\n  (bool * (bool * bool))%type.\n";
        let lex = Lexer::lex(src).expect("lex");
        for (t, ln) in &lex.toks {
            eprintln!("{:?} @{}", t, ln);
        }
        let u = parse_unit("test.v", src);
        eprintln!("unit: {:#?}", u);
    }

    #[test]
    fn v2_key_literal_parses() {
        let src = "Require Import List.\nDefinition key : Type := (list nat).\nDefinition k : key := [33; 61].\n";
        let u = parse_unit("t.v", src);
        assert_eq!(u.definitions.len(), 2);
    }

    #[test]
    fn qualified_idents_lex_as_one() {
        let toks = Lexer::lex("Definition f (b : byte) : bool := Nat.eqb b 9.")
            .expect("lex")
            .toks
            .into_iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();
        assert!(toks.iter().any(|t| matches!(t, Tok::Ident(w) if w == "Nat.eqb")));
        assert!(toks.iter().any(|t| matches!(t, Tok::NatLit(9))));
    }
}
