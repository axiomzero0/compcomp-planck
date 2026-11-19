# CEP:FILE: rocq/README.md
# CEP:WHAT: Build and pin procedure for the Rocq foundation layer (CEP-DR-01 DR-1.2, DR-1.5, DR-7.4): directory map, version pinning, kernel-check commands, and the integration rule with the obligation matrix.
# CEP:WHY: CEP-DR-01 section 1 rule 5 requires every ROCQ obligation row to name the exact Rocq version, the file under rocq/, and the compiled .vo check command; this README is the procedure those rows cite.
# CEP:STATUS: partial — the layer is bootstrapping: Effect/RowAlgebra.v is written (kernel check pending); the version pin lands at the first successful build.
# CEP:FAILURE: A ROCQ obligation row that cites an unpinned version or an unbuilt .vo is non-conforming.
# CEP:ASSUMES: CEP-DR-01 (toolchain decision, pinned-version policy §6); SPEC-INDEX §4 (method ROCQ added by DR-1.5); the Rocq standard library only (no external packages) until Peregrine integration (OB-DR-1..4).
# CEP:COST: build-time only.
# CEP:EVIDENCE: this file; the .vo files produced by build.sh; spec/CEP-DR-01-rocq-peregrine-toolchain.md.

## 1. Directory map

- `Effect/RowAlgebra.v` — the pinned effect algebra (SPEC-INDEX §5.3) as a
  constructive Rocq development: rows as 8-bit vectors over the canonical atom
  order, join/meet/leq, and the pinned law set (associative, commutative,
  idempotent, identity ∅, meet-distributivity, partial order) proven by
  exhaustive case analysis over the full 256-row lattice. Formal counterpart
  of `math/crates/planck-effect-math` (EXH, same domain).

## 2. Build and pin procedure (run at first opportunity; records the pin)

```sh
cd rocq
./build.sh
```

`build.sh`:

1. Prints the exact version string of `rocq` (or `coqc` on older installs).
   That string is the CEP-DR-01 §6 pin; record it in `Effect/PIN.v`-style
   comment and in the decision record's table (§6, row Rocq).
2. Compiles every `.v` under `rocq/` (one `_CoqProject` listing, `rocq make`
   or `coq_makefile`).
3. A build with zero errors and zero warnings-as-errors is the kernel check;
   the `.vo` files are the evidence artifacts (DR-1.5).

Until that first build succeeds, every file here carries CEP:STATUS partial
and no ROCQ obligation row may cite it (CEP-DR-01 DR-1.5, by the letter).

## 3. Integration rules

1. Definitions here refine the pinned spec contracts; where a volume is more
   precise, the volume wins (SPEC-INDEX §0 authority chain).
2. Every proof must be a closed term: no `Admitted`, no `Axiom`, no `admit`
   (SPEC-INDEX §1 rule 5, no sorry analogues). A proof that cannot be closed
   is stated as a named refinement gap in the volume's verification matrix
   (§54.2.2), never as an admitted theorem.
3. Equivalence with `math/` kernels: the Rocq development and the Rust EXH
   certificate cover the same complete domains (G18 dual-engine discharge
   until OB-DR-5 lands).
4. Peregrine integration (OB-DR-2): the `rocq/subset-report.md` artifact
   enumerates the Planck surface accepted by the MetaRocq/ConCert route;
   `rocq/rustc-assumptions.md` enumerates the OB-DR-4 assumptions. Neither
   exists yet; both are prerequisite artifacts of the OB-DR rows.
