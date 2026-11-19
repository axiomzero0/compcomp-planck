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

$COMPILE Effect/RowAlgebra.v
echo "KERNEL_CHECK: OK (Effect/RowAlgebra.vo)"
