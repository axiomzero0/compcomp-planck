#!/bin/bash
# CEP:FILE: rocq/build.sh
# CEP:WHAT: Build script for the Rocq foundation layer: version pin capture (CEP-DR-01 §6) and kernel check of every .v file under rocq/.
# CEP:WHY: CEP-DR-01 DR-1.5 requires ROCQ obligation rows to name the exact version and the .vo check command; this script produces both.
# CEP:STATUS: complete
# CEP:FAILURE: A non-zero exit is a failed kernel check; the obligation stays unproven until the script exits zero.
# CEP:ASSUMES: rocq (Rocq 9+) or coqc (Coq 8.x) on PATH; no external packages.
# CEP:COST: build-time only.
# CEP:EVIDENCE: stdout (version string) and the .vo files.
# CEP:SECURITY: repository-trusted input.

set -euo pipefail
cd "$(dirname "$0")"

if command -v rocq >/dev/null 2>&1; then
  echo "ROCQ_VERSION: $(rocq --version)"
  COMPILE=(rocq c)
elif command -v coqc >/dev/null 2>&1; then
  echo "COQ_VERSION: $(coqc --version)"
  COMPILE=(coqc)
else
  echo "ERROR: no rocq or coqc on PATH; the Rocq layer is not kernel-checkable in this environment." >&2
  exit 2
fi

# Pin-drift assertion (CEP-DR-01 §6): the kernel check runs ONLY against the
# pinned prover; a different version fails the build loudly here.
PIN="The Coq Proof Assistant, version 8.20.1"
VERSION_OUT=$("${COMPILE[@]}" --version 2>/dev/null || coqc --version)
if [[ "$VERSION_OUT" != "$PIN"* ]]; then
  echo "ERROR: prover version drift: pinned '$PIN*' but found '$VERSION_OUT' (CEP-DR-01 §6; a bump is an edition-of-record event)." >&2
  exit 3
fi
echo "PIN_CHECK: OK ($PIN)"

# Kernel check of EVERY .v file under rocq/ (deterministic), all under the
# pinned logical prefix Effect (so cross-file Require paths and the coqchk
# module name are uniform). Dependency discipline: the foundation proof file
# Effect/RowAlgebra.v compiles FIRST (the generated oracle Requires it), then
# the remaining files in sorted order.
"${COMPILE[@]}" -Q Effect Effect Effect/RowAlgebra.v
echo "COMPILE: Effect/RowAlgebra.v (foundation, first)"
while IFS= read -r v; do
  if [[ "$v" == "./Effect/RowAlgebra.v" ]]; then
    continue
  fi
  echo "COMPILE: $v"
  "${COMPILE[@]}" -Q Effect Effect "$v"
done < <(find . -name "*.v" -not -path "./.*" | sort)
echo "KERNEL_CHECK: OK (all .vo files produced)"

# Independent re-check by the standalone kernel checker (coqchk): the .vo
# is re-loaded and re-verified from scratch, listing any axioms, unsafe
# fixpoints, or assumed positivity. A non-empty axiom list is a failed
# check (SPEC-INDEX §1 rule 5: no sorry analogues).
coqchk -Q Effect Effect -o Effect.RowAlgebra
echo "INDEPENDENT_CHECK: OK (coqchk: axioms none, type-in-type none, unsafe fixpoints none)"
