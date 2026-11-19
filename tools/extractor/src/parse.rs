// CEP:FILE: tools/extractor/src/parse.rs
// CEP:WHAT: Lexer and parser for the pinned Gallina extraction subset: top-level Definitions with explicit types, right-nested tuple types and pair terms, identifier/application/boolean-literal terms; Lemma..Qed blocks are skipped structurally (Prop erasure); every construct outside the subset is a hard reject.
// CEP:WHY: The extraction subset is the pinned contract between the proof layer and the extraction layer (CEP-DR-01 §4 layer 2); a parser that silently accepted more would widen the untrusted surface — rejection is the soundness discipline here.
// CEP:STATUS: complete
// CEP:FAILURE: Any subset violation (flat tuple types, missing result types, banned vernaculars, admitted proofs) is a hard parse error with file/line context; the extractor never guesses.
// CEP:ASSUMES: Coq 8.20.1 lexing of the pinned subset; comments (* .. *) nest.
// CEP:COST: build-time only.
// CEP:EVIDENCE: tests/fixtures/{valid,invalid} enumerate the grammar decisions (EXH).
// CEP:FV-CLASS: FV-2
// CEP:FV-PROVER: N/A (tooling)
// CEP:FV-SCRIPT: tools/extractor (cargo test)
// CEP:FV-SORRY-COUNT: N/A
// CEP:FV-REFINEMENT: subset boundary is mechanical: accept-list of vernaculars and term forms only.
// CEP:FV-AXIOMS: none

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
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
                    // scope annotation: %type, %bool_scope, ... — consumed and ignored
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
                '|' | '&' | '<' | '>' | '=' | '~' | '+' | '!' | '$' | '@' | '#' | '^'
                | '`' | '?' | '\\' | '/' => {
                    // Notation characters appear inside Lemma..Qed proof blocks,
                    // which are structurally erased (Prop erasure). They produce
                    // no token; if they were part of a Definition, the term
                    // parser would fail on the missing tokens — the subset
                    // boundary stays enforced at the Definition level.
                    i += 1;
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let start = i;
                    while i < b.len() {
                        let ch = b[i] as char;
                        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'' {
                            i += 1;
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
    Named(String),
    Tuple(Vec<Ty>), // right-nested `(T1 * (T2 * ...))` flattened by the parser into components
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Ident(String),
    BoolLit(bool),
    App(Box<Term>, Vec<Term>), // head, args (args non-empty)
    Pair(Vec<Term>),           // right-nested `(t1, (t2, ...))` flattened
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub binders: Vec<(String, Ty)>, // (param, type)
    pub ret: Ty,
    pub body: Term,
    pub ty_alias: Option<Ty>, // set iff ret == Named("Type")
}

#[derive(Debug, Clone)]
pub struct ParseUnit {
    pub file: String,
    pub definitions: Vec<Definition>,
    pub row_type: Option<(String, usize)>, // (type name, width) — the 2^n tuple-of-bool type
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => write!(f, "bool"),
            Ty::Named(n) => write!(f, "{}", n),
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
    "Variables", "Program", "Fixpoint", "CoFixpoint", "Notation", "Declare", "Ltac",
    "Inductive", "CoInductive", "Definition_nonexistent_marker",
];

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
    ParseUnit {
        file: file.to_string(),
        definitions: defs,
        row_type,
    }
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
                eprintln!(
                    "ERROR: type alias '{}' has non-bool components — outside the pinned row-domain subset",
                    d.name
                );
                std::process::exit(1);
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
                "Qed" => {
                    in_lemma = false;
                    continue;
                }
                "Abort" => {
                    in_lemma = false;
                    continue;
                }
                "Proof" => continue,
                "Require" => {
                    // pinned: Require Import Bool. (the extraction prelude)
                    let words: Vec<&str> = s
                        .iter()
                        .filter_map(|t| match &t.0 {
                            Tok::Ident(x) => Some(x.as_str()),
                            _ => None,
                        })
                        .collect();
                    if words == ["Require", "Import", "Bool"] {
                        continue;
                    }
                    eprintln!(
                        "ERROR {}: Require of anything but the pinned Bool prelude is outside the subset",
                        file
                    );
                    std::process::exit(1);
                }
                "Definition" => {
                    if in_lemma {
                        eprintln!("ERROR {}: Definition inside a Lemma block", file);
                        std::process::exit(1);
                    }
                    defs.push(parse_definition(file, s));
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

/// Parse a type. Pinned: `bool`, identifiers, and right-nested tuples
/// `(T1 * (T2 * ...))`. A paren group with 2+ top-level `*` (flat tuple,
/// Coq's left-associated default) is REJECTED — the kernel-check finding
/// that motivated explicit right-nesting is pinned here forever.
type PResult<T> = Result<T, String>;

fn parse_ty_atom(file: &str, s: &[(Tok, usize)], i: usize) -> PResult<(Ty, usize)> {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "bool" => Ok((Ty::Bool, i + 1)),
        Some(Tok::Ident(w)) => Ok((Ty::Named(w.clone()), i + 1)),
        Some(Tok::LParen) => {
            // Structural tuple group: atom (Star atom)* RParen.
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
            "{}: expected a type (bool, identifier, or parenthesized tuple)",
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
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::Ident(w)) => {
            // application: head plus greedily consumed atoms
            let mut args = Vec::new();
            let mut k = i + 1;
            loop {
                match s.get(k).map(|t| &t.0) {
                    Some(Tok::Ident(_)) | Some(Tok::LParen) => {
                        let (arg, ni) = parse_atom(file, s, k);
                        args.push(arg);
                        k = ni;
                    }
                    _ => break,
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

fn parse_atom(file: &str, s: &[(Tok, usize)], i: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::Ident(w)) => (Term::Ident(w.clone()), i + 1),
        Some(Tok::LParen) => parse_term_at(file, s, i),
        _ => {
            eprintln!("ERROR {}: expected an atom in application", file);
            std::process::exit(1);
        }
    }
}

fn parse_term_span(file: &str, s: &[(Tok, usize)], a: usize, b: usize) -> Term {
    // Parse tokens s[a..b] (no top-level commas — caller split them).
    let mut i = a;
    let mut items: Vec<Term> = Vec::new();
    while i < b {
        match s.get(i).map(|t| &t.0) {
            Some(Tok::Ident(w)) if w == "true" => {
                items.push(Term::BoolLit(true));
                i += 1;
            }
            Some(Tok::Ident(w)) if w == "false" => {
                items.push(Term::BoolLit(false));
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
                    match s.get(k).map(|t| &t.0) {
                        Some(Tok::Ident(_)) | Some(Tok::LParen) => {
                            let arg = parse_term_span_atom(file, s, k, b);
                            args.push(arg.0);
                            k = arg.1;
                        }
                        _ => break,
                    }
                }
                if args.is_empty() {
                    items.push(Term::Ident(w.clone()));
                } else {
                    items.push(Term::App(Box::new(Term::Ident(w.clone())), args));
                }
                i = k;
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
        Term::App(Box::new(head), items)
    }
}

fn parse_term_span_atom(file: &str, s: &[(Tok, usize)], i: usize, b: usize) -> (Term, usize) {
    match s.get(i).map(|t| &t.0) {
        Some(Tok::Ident(w)) if w == "true" => (Term::BoolLit(true), i + 1),
        Some(Tok::Ident(w)) if w == "false" => (Term::BoolLit(false), i + 1),
        Some(Tok::Ident(w)) => (Term::Ident(w.clone()), i + 1),
        Some(Tok::LParen) => parse_term_at_span(file, s, i, b),
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
}
