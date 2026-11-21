//! CEP:FILE: tools/fv/matp-driver/src/main.rs
//! CEP:WHAT: The MATP obligation driver for the Planck math kernel: loads each TPTP CNF problem under tools/fv/matp/problems/, translates its clauses into a MATP SearchSpace (re-interning symbols clause-locally), saturates with the MATP given-clause loop (superposition/resolution, KBO), and records Theorem/Saturated artifacts under tools/fv/matp/artifacts/.
//! CEP:WHY: verification standards G15/G18 require independently checkable proof artifacts; MATP (github.com/compilecomp/MATP) is the project ATP per SPEC-INDEX section 3 (compiler verification target). The driver is the CEP:FV-SCRIPT for every MATP obligation.
//! CEP:CLASS: CEP-2
//! CEP:STATUS: complete
//! CEP:FAILURE: Exits nonzero if any problem fails to parse, any outcome mismatches its expectation, or the iteration bound fires (GaveUp is never accepted as a pass).
//! CEP:ASSUMES: MAPT is pinned at revision 677bea9 (trusted computing base, verification standards G16); problems are repository-trusted; include directives resolve under the problems root.
//! CEP:COST: offline tool; runtime cost irrelevant.
//! CEP:EVIDENCE: tools/fv/matp/artifacts/*.result (one per obligation; each records the actual outcome as-is — a result other than its EXPECTED value has STATUS: fail and the obligation is discharged by another engine per SPEC-INDEX section 4, never silently).
//! CEP:SECURITY: repository-trusted inputs; no network; bounded by the documented iteration limit.
//! CEP:FV-CLASS: FV-0/FV-2
//! CEP:FV-PROVER: MATP (given-clause saturation, superposition + resolution, KBO)
//! CEP:FV-SCRIPT: tools/fv/matp-driver/src/main.rs
//! CEP:FV-SORRY-COUNT: 0 (a GaveUp or parse failure fails the run; no admitted obligation)
//! CEP:FV-REFINEMENT: the .p problems are the machine-readable form of the Volume 06/07 pinned contracts; the Rust kernels in math/ carry EXH runtime bridges over the same laws (SPEC-INDEX section 5.1).
//! CEP:FV-AXIOMS: none (every clause is a pinned-contract fact or an explicitly negated goal).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mapt::cold::parsing::{parse_tptp, ParsedProblem};
use mapt::cold::symbol_table_builder::SymbolTableBuilder;
use mapt::hot::inference::{InferenceContext, OrderingKind};
use mapt::hot::ir::clause::ClausePtr;
use mapt::hot::ir::literal::Literal;
use mapt::hot::ir::symbol_table::SortId;
use mapt::hot::ir::symbol_table::SymbolTable;
use mapt::hot::ir::term::{TermPtr, TermStore, TermTag};
use mapt::hot::memory::arena::Arena;
use mapt::hot::ordering::precedence::PrecedenceTable;
use mapt::hot::search::given_clause::{
    saturate, SearchOptions, SearchOutcome, SearchSpace, SearchStats,
};

/// CEP:WHAT: The documented iteration bound (CEP&CC 54.5.2): a run past this bound is GaveUp and fails the driver; it is never treated as a pass.
const ITERATION_BOUND: u64 = 200_000;

/// CEP:WHAT: Per-obligation arena bytes; one leaked arena per obligation keeps runs independent.
/// CEP:WHY (sizing): parse_tptp allocates its own TermStore (kTermHashTableCapacity = 2^21 u32 cells = 8 MiB) plus the parsed IR; the driver allocates a second TermStore (8 MiB) and the SearchSpace tables on the same arena. 64 MiB covers both stores, the search tables, and the problem IR with headroom; it is below kArenaCapacityBytes (256 MiB) so the region always validates.
const ARENA_BYTES: usize = 64 * 1024 * 1024;

/// CEP:WHAT: The expectation of one obligation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// The input must refute: SearchOutcome::Theorem.
    Theorem,
    /// The input must saturate: SearchOutcome::Saturated (a model exists).
    Saturated,
}

impl Expect {
    fn label(self) -> &'static str {
        match self {
            Expect::Theorem => "theorem",
            Expect::Saturated => "saturated",
        }
    }
}

struct Obligation {
    name: &'static str,
    expect: Expect,
}

const OBLIGATIONS: &[Obligation] = &[
    Obligation { name: "harness_mp", expect: Expect::Theorem },
    Obligation { name: "harness_sat", expect: Expect::Saturated },
    Obligation { name: "eff_join_assoc", expect: Expect::Theorem },
    Obligation { name: "eff_join_comm", expect: Expect::Theorem },
    Obligation { name: "eff_join_idem", expect: Expect::Theorem },
    Obligation { name: "eff_join_ident", expect: Expect::Theorem },
    Obligation { name: "eff_meet_distrib", expect: Expect::Theorem },
    Obligation { name: "eff_absorb", expect: Expect::Theorem },
    Obligation { name: "own_endborrow_borrowed_sh", expect: Expect::Theorem },
    Obligation { name: "own_endborrow_borrowed_mut", expect: Expect::Theorem },
    Obligation { name: "own_endborrow_borrowed_fr", expect: Expect::Theorem },
    Obligation { name: "own_endborrow_zero", expect: Expect::Theorem },
    Obligation { name: "own_table_cells", expect: Expect::Theorem },
    // DO-1 extension pass (VR-14(d)): the charclass and operator domains
    Obligation { name: "cc_ident_continue_union", expect: Expect::Theorem },
    Obligation { name: "cc_whitespace_truth_agreement", expect: Expect::Theorem },
    Obligation { name: "op_precedence_compare_agreement", expect: Expect::Theorem },
];

/// CEP:WHAT: Leaks one per-run arena via MATP's own aligned region host (safe code; CEP-2 tooling).
fn make_arena(capacity_bytes: usize) -> &'static Arena {
    let region = mapt::cold::arena_host::allocate_aligned_region(capacity_bytes as u32)
        .expect("region allocation failed");
    let arena = Arena::new(region).expect("arena construction failed");
    Box::leak(Box::new(arena))
}

/// CEP:WHAT: What the pre-scan learned about one used symbol.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SymKind {
    Function,
    Predicate,
}

/// CEP:WHAT: Collects the symbol inventory of one parsed problem: every name used as a function or a predicate, with its arity (BTreeMap keeps declaration order deterministic).
#[derive(Default)]
struct Inventory {
    funs: BTreeMap<String, u16>,
    preds: BTreeMap<String, u16>,
}

fn main() {
    let repo = match std::env::var("PLANCK_REPO_ROOT") {
        Ok(r) => PathBuf::from(r),
        Err(_) => std::env::current_dir().expect("cwd"),
    };
    let problems = repo.join("tools/fv/matp/problems");
    let artifacts = repo.join("tools/fv/matp/artifacts");
    std::fs::create_dir_all(&artifacts).expect("artifacts dir");

    let mut failures: Vec<String> = Vec::new();
    for ob in OBLIGATIONS {
        match run_one(ob, &problems, &artifacts) {
            Ok(status) => println!("[pass] {status}"),
            Err(err) => {
                println!("[FAIL] {} : {err}", ob.name);
                failures.push(format!("{}: {err}", ob.name));
            }
        }
    }
    if !failures.is_empty() {
        eprintln!("matp-fv-driver: {} obligation(s) failed", failures.len());
        for f in &failures {
            eprintln!("  {f}");
        }
        std::process::exit(1);
    }
    println!(
        "matp-fv-driver: all {} obligations passed (bound: {ITERATION_BOUND} iterations, documented per CEP&CC 54.5.2)",
        OBLIGATIONS.len()
    );
}

fn run_one(ob: &Obligation, problems: &Path, artifacts: &Path) -> Result<String, String> {
    let path = problems.join(format!("{}.p", ob.name));
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;

    let arena = make_arena(ARENA_BYTES);
    let problem = parse_tptp(arena, &bytes, Some(problems))
        .map_err(|e| format!("MATP parse of {}: {e:?}", path.display()))?;

    // Pre-scan: inventory of used symbols (kind + arity) from the parsed clause bodies.
    let mut inv = Inventory::default();
    for &ptr in &problem.clause_ptrs {
        scan_clause(&problem, ptr, &mut inv)?;
    }

    // Declare the mirrored symbol table, freeze it, and build the run fixture.
    let mut builder = SymbolTableBuilder::new();
    let sort = SortId(
        builder
            .declare_sort("U")
            .map_err(|e| format!("declare sort: {e:?}"))?,
    );
    for (name, arity) in &inv.funs {
        let arg_sorts = vec![sort; *arity as usize];
        builder
            .declare_function(name, *arity, 1, &arg_sorts, sort)
            .map_err(|e| format!("declare fn {name}: {e:?}"))?;
    }
    for (name, arity) in &inv.preds {
        let arg_sorts = vec![sort; *arity as usize];
        builder
            .declare_predicate(name, *arity, 1, &arg_sorts)
            .map_err(|e| format!("declare pred {name}: {e:?}"))?;
    }
    let symbols = builder.finalize(arena).map_err(|e| format!("finalize: {e:?}"))?;
    let terms = TermStore::new(arena).map_err(|e| format!("term store: {e:?}"))?;
    let precedence = PrecedenceTable::from_id_order(arena, builder.symbol_count())
        .map_err(|e| format!("precedence: {e:?}"))?;
    let options = SearchOptions::new();
    // Capacities (CEP&CC 34.3, each documented): the law obligations close by
    // resolution case analysis over the 8-projection bit domain, which needs a
    // passive set beyond the 8192 default (the effect laws derive ~10k clauses).
    let space = SearchSpace::new_with(arena, &options, 65536, 32768, 65536, 65536, 1_048_576, 65536)
        .map_err(|e| format!("search space: {e:?}"))?;
    let context = InferenceContext {
        terms: &terms,
        symbols: &symbols,
        substitution: &space.substitution,
        precedence: &precedence,
        ordering: OrderingKind::Kbo,
    };

    // Translate every parsed clause into the run fixture and add it as an input clause.
    for &ptr in &problem.clause_ptrs {
        let lits = translate_clause(&problem, ptr, &terms, &symbols, &builder)?;
        space
            .add_input_clause(&context, &options, &lits)
            .map_err(|e| format!("add clause: {e:?}"))?;
    }

    let mut stats = SearchStats::default();
    let outcome = saturate(&space, &context, &options, &stop_at_bound, &mut stats)
        .map_err(|e| format!("saturate: {e:?}"))?;

    let (outcome_name, pass) = match (&outcome, ob.expect) {
        (SearchOutcome::Theorem { .. }, Expect::Theorem) => ("Theorem", true),
        (SearchOutcome::Saturated, Expect::Saturated) => ("Saturated", true),
        (o, e) => (outcome_label(o), false),
    };
    write_artifact(ob, &path, artifacts, outcome_name, pass, &stats)?;
    if !pass {
        return Err(format!(
            "expected {}, got {} (given {} derived {})",
            ob.expect.label(),
            outcome_name,
            stats.given_processed,
            stats.inferences
        ));
    }
    Ok(format!(
        "{} -> {} (given {}, inferences {})",
        ob.name,
        outcome_name,
        stats.given_processed,
        stats.inferences
    ))
}

/// CEP:WHAT: The documented stop callback: true (stop) at the iteration bound.
fn stop_at_bound(iterations: u64) -> bool {
    iterations >= ITERATION_BOUND
}

fn outcome_label(o: &SearchOutcome) -> &'static str {
    match o {
        SearchOutcome::Theorem { .. } => "Theorem",
        SearchOutcome::Saturated => "Saturated",
        SearchOutcome::GaveUp { .. } => "GaveUp",
    }
}

fn write_artifact(
    ob: &Obligation,
    problem_path: &Path,
    artifacts: &Path,
    outcome: &str,
    pass: bool,
    stats: &SearchStats,
) -> Result<(), String> {
    let text = format!(
        "CEP:FILE: tools/fv/matp/artifacts/{}.result\nCEP:WHAT: MATP verification artifact for obligation {}.\nCEP:WHY: Independently checkable proof artifact (verification standards G15, G18); produced by tools/fv/matp-driver.\nCEP:STATUS: {}\nCEP:FAILURE: Regenerate with the matp-fv-driver if the problem or the pinned MATP revision changes.\nCEP:ASSUMES: MAPT pinned at revision 677bea9 (trusted computing base).\nCEP:COST: offline artifact.\nCEP:EVIDENCE: this file.\nCEP:SECURITY: repository-trusted input.\n\nOBLIGATION: {}\nPROBLEM: {}\nEXPECTED: {}\nOUTCOME: {}\nGIVEN_PROCESSED: {}\nBOUND: {} iterations (documented per CEP&CC 54.5.2; no wall-clock bound)\nSTATUS: {}\nMAPT_REVISION: 677bea9 (github.com/compilecomp/MATP)\n",
        ob.name,
        ob.name,
        if pass { "complete" } else { "partial" },
        ob.name,
        problem_path.display(),
        ob.expect.label(),
        outcome,
        stats.given_processed,
        ITERATION_BOUND,
        if pass { "pass" } else { "fail" },
    );
    let out = artifacts.join(format!("{}.result", ob.name));
    std::fs::write(&out, text).map_err(|e| format!("write {}: {e}", out.display()))
}

/// CEP:WHAT: Scans one parsed clause and records every symbol usage with its kind and arity.
fn scan_clause(
    problem: &ParsedProblem<'_>,
    ptr: ClausePtr,
    inv: &mut Inventory,
) -> Result<(), String> {
    let clause = problem
        .clauses
        .clause(ptr)
        .map_err(|e| format!("clause read: {e:?}"))?;
    for i in 0..clause.literal_count {
        let lit = problem
            .clauses
            .literal(clause, i)
            .map_err(|e| format!("literal read: {e:?}"))?;
        scan_atom(problem, lit.atom(), inv, 0)?;
    }
    Ok(())
}

fn scan_atom(
    problem: &ParsedProblem<'_>,
    ptr: TermPtr,
    inv: &mut Inventory,
    depth: u32,
) -> Result<(), String> {
    if depth > 64 {
        return Err("scan depth cap exceeded".to_string());
    }
    let view = problem
        .terms
        .term(ptr)
        .map_err(|e| format!("term read: {e:?}"))?;
    match view.tag() {
        TermTag::Variable => Ok(()),
        TermTag::Equality => {
            for child in view.children() {
                scan_atom(problem, child, inv, depth + 1)?;
            }
            Ok(())
        }
        TermTag::Predicate => {
            let name = symbol_name(problem, view.symbol())?;
            inv.preds.insert(name, view.children().count() as u16);
            for child in view.children() {
                scan_atom(problem, child, inv, depth + 1)?;
            }
            Ok(())
        }
        TermTag::Function => {
            let name = symbol_name(problem, view.symbol())?;
            inv.funs.insert(name, view.children().count() as u16);
            for child in view.children() {
                scan_atom(problem, child, inv, depth + 1)?;
            }
            Ok(())
        }
        other => Err(format!("unexpected term tag {other:?} in CNF input")),
    }
}

fn symbol_name(problem: &ParsedProblem<'_>, id: u32) -> Result<String, String> {
    problem
        .builder
        .name_of(id)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("symbol {id} has no name"))
}

/// CEP:WHAT: Rebuilds one parsed clause in the run fixture's term store: variables keep their clause-local indices; function and predicate symbols resolve through the pre-declared mirrored table; equality atoms rebuild both sides.
fn translate_clause(
    problem: &ParsedProblem<'_>,
    ptr: ClausePtr,
    terms: &TermStore<'_>,
    symbols: &SymbolTable<'_>,
    builder: &SymbolTableBuilder,
) -> Result<Vec<Literal>, String> {
    let clause = problem
        .clauses
        .clause(ptr)
        .map_err(|e| format!("clause read: {e:?}"))?;
    let mut lits = Vec::new();
    for i in 0..clause.literal_count {
        let lit = problem
            .clauses
            .literal(clause, i)
            .map_err(|e| format!("literal read: {e:?}"))?;
        let atom = build_atom(problem, lit.atom(), terms, symbols, builder, 0)?;
        lits.push(Literal::new(atom, lit.is_positive()));
    }
    Ok(lits)
}

fn build_atom(
    problem: &ParsedProblem<'_>,
    ptr: TermPtr,
    terms: &TermStore<'_>,
    symbols: &SymbolTable<'_>,
    builder: &SymbolTableBuilder,
    depth: u32,
) -> Result<TermPtr, String> {
    if depth > 64 {
        return Err("build depth cap exceeded".to_string());
    }
    let view = problem
        .terms
        .term(ptr)
        .map_err(|e| format!("term read: {e:?}"))?;
    match view.tag() {
        TermTag::Variable => terms
            .intern_var(view.symbol())
            .map_err(|e| format!("intern var: {e:?}")),
        TermTag::Equality => {
            let kids: Vec<TermPtr> = view.children().collect();
            if kids.len() != 2 {
                return Err("equality arity != 2".to_string());
            }
            let l = build_atom(problem, kids[0], terms, symbols, builder, depth + 1)?;
            let r = build_atom(problem, kids[1], terms, symbols, builder, depth + 1)?;
            terms
                .intern_eq(symbols, l, r)
                .map_err(|e| format!("intern eq: {e:?}"))
        }
        TermTag::Predicate | TermTag::Function => {
            let name = symbol_name(problem, view.symbol())?;
            let kids: Vec<TermPtr> = view.children().collect();
            let mut built = Vec::with_capacity(kids.len());
            for k in kids {
                built.push(build_atom(problem, k, terms, symbols, builder, depth + 1)?);
            }
            // Resolve the mirrored symbol id by name.
            let mut found: Option<u32> = None;
            for id in 0..builder.symbol_count() {
                if builder.name_of(id) == Some(name.as_str()) {
                    found = Some(id);
                    break;
                }
            }
            let id = found.ok_or_else(|| format!("symbol {name} not declared"))?;
            if view.tag() == TermTag::Predicate {
                terms
                    .intern_pred(symbols, id, &built)
                    .map_err(|e| format!("intern pred {name}: {e:?}"))
            } else {
                terms
                    .intern_fun(symbols, id, &built)
                    .map_err(|e| format!("intern fun {name}: {e:?}"))
            }
        }
        other => Err(format!("unexpected term tag {other:?} in CNF input")),
    }
}
