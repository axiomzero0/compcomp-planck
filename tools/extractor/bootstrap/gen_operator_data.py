#!/usr/bin/env python3
"""Bootstrap transcription generator for rocq/Effect/OperatorTableData.v.

Committed in-repo at tools/extractor/bootstrap/ (the byte-identity
cross-check instrument of CEP-DR-01 §8 DR-8.8 v2.15).

Parses the reference kernel math/crates/planck-operator-math/src/lib.rs and
emits the generated extraction unit carrying the 44 pinned rows mechanically.
The final generator is `planck-extractor transcribe` (Rust, FV-2); this
bootstrap must produce byte-identical output and is cross-checked against it.

Part of the DR-8.8 v2.15 transcription discipline: the .v data is generated
from the reference source; regeneration is the only edit path.
"""
import hashlib
import re
import sys

REF = "/home/z/my-project/compcomp-planck/math/crates/planck-operator-math/src/lib.rs"
OUT = "/home/z/my-project/compcomp-planck/rocq/Effect/OperatorTableData.v"

src = open(REF, "r", encoding="utf-8").read()
sha = hashlib.sha256(src.encode("utf-8")).hexdigest()

# 1. PREC_* constants
prec = dict(re.findall(r"pub const (PREC_[A-Z_]+): u8 = (\d+);", src))
assert len(prec) == 15, f"expected 15 digit-valued PREC consts, got {len(prec)}"

# 2. OpId enum declaration order (volume row order OP-01..OP-44)
m = re.search(r"pub enum OpId \{(.*?)\n\}", src, re.S)
assert m, "OpId enum not found"
decl_order = re.findall(r"^\s{4}([A-Z][A-Za-z0-9]*) = \d+,", m.group(1), re.M)
assert len(decl_order) == 44, f"expected 44 OpId variants, got {len(decl_order)}"

# 3. The pinned table rows (array order = sorted order)
row_re = re.compile(
    r'OperatorRow \{ id: OpId::([A-Za-z0-9]+), token: "([^"]*)", '
    r"precedence: ([A-Z_]+), assoc: Assoc::([A-Za-z]+), arity: Arity::([A-Za-z]+), "
    r"overloadable: (true|false), overflow: Overflow::([A-Za-z]+) \}"
)
rows = row_re.findall(src)
assert len(rows) == 44, f"expected 44 rows, got {len(rows)}"

# 4. Sorted-order sanity: byte-lexicographic strictly ascending tokens
toks = [r[1] for r in rows]
for a, b in zip(toks, toks[1:]):
    assert a.encode() < b.encode(), f"table not sorted: {a!r} >= {b!r}"
assert len(set(toks)) == 44, "duplicate keys"

def ascii_list(s):
    codes = [ord(c) for c in s]
    assert all(1 <= c <= 126 for c in codes), f"non-printable-ASCII token: {s!r}"
    return "[" + "; ".join(str(c) for c in codes) + "]"

def resolve_prec(name):
    if name.startswith("PREC_") and name in prec:
        return prec[name]
    raise AssertionError(f"unresolved precedence const {name}")

L = []
A = L.append
A("(* GENERATED FILE - DO NOT EDIT. Generated mechanically from")
A("   math/crates/planck-operator-math/src/lib.rs (the reference kernel,")
A("   EXH-certified against Volume 02 §2.4.1) by the pinned transcription")
A("   procedure of CEP-DR-01 §8 DR-8.8 v2.15; regenerate with")
A("   `planck-extractor transcribe`. Hand edits are non-conforming.")
A("   CEP:FILE: rocq/Effect/OperatorTableData.v")
A("   CEP:PROVENANCE: source math/crates/planck-operator-math/src/lib.rs (sha256 " + sha + "); generated mechanically by the DR-8.8 v2.15 transcription procedure — hand edits are non-conforming (CEP-DR-01 §8 DR-8.2), regenerate with `planck-extractor transcribe`.")
A("   CEP:WHAT: The 44 pinned §2.4.1 operator rows as kernel-checked Coq data: the ASCII key definitions (subset v2.6 list-nat literals), the right-nested 7-tuple rows (key, opid, precedence, assoc, arity, overloadable, overflow), the sorted table, the closed row_at dispatch, and the seven mechanical row projections.")
A("   CEP:WHY: The operator-table foundation domain (VR-14(d), DO-1 phase 1) needs its data in the proof layer so the accessor-agreement, distinctness, sortedness, and lookup laws are kernel-checked and the extracted Rust is DIF-verified against this data; generating from the reference source closes the transcription loop mechanically.")
A("   CEP:STATUS: complete (generated)")
A("   CEP:FAILURE: A row that disagrees with the reference table fails the three-way DIF (extracted vs kernel vs reference) over the closed 44-row domain; regeneration is the only edit path.")
A("   CEP:ASSUMES: the reference table is the definition of record (EXH-certified to Volume 02 §2.4.1); token cells are printable ASCII; the array order is the sorted order (asserted mechanically at generation).")
A("   CEP:COST: generation build-time only; the data is const.")
A("   CEP:EVIDENCE: this file; DIF ground truth generated from it by planck-extractor; the reference-axis tables carry the same sha256 provenance.")
A("   CEP:SECURITY: repository-trusted inputs; generated mechanically, hand edits are non-conforming.")
A("   CEP:FV-CLASS: FV-2 (generated kernel-checked data; qualified by the DIF it feeds)")
A("   CEP:FV-PROVER: Coq 8.20.1 kernel (coqc + coqchk, zero axioms)")
A("   CEP:FV-SCRIPT: planck-extractor transcribe (bootstrap: tools/extractor/bootstrap/gen_operator_data.py)")
A("   CEP:FV-SORRY-COUNT: N/A")
A("   CEP:FV-REFINEMENT: the row domain is closed (44 rows); the DIF covers all of it; no sampling.")
A("   CEP:FV-AXIOMS: none *)")
A("")
A("Require Import Bool.")
A("Require Import Arith.")
A("Require Import List.")
A("Require Import Effect.OperatorEnums.")
A("Import ListNotations.")
A("Import PinnedEnums.")
A("")
A("(* The row record: the six pinned columns plus the identity and the key. *)")
A("Definition oprow : Type :=")
A("  (key * (opid * (nat * (assoc * (arity * (bool * overflow)))))).")
A("")
A("(* The 44 pinned keys (ASCII code points, subset v2.6). *)")
for i, r in enumerate(rows):
    A(f"Definition row_{i:02d}_key : key := {ascii_list(r[1])}.")
A("")
A("(* The 44 pinned rows (Volume 02 §2.4.1, sorted order). *)")
for i, r in enumerate(rows):
    (oid, tok, p, assoc, arity, over, ovf) = r
    A(f"Definition row_{i:02d} : oprow :=")
    A(f"  (row_{i:02d}_key, ({oid}, ({resolve_prec(p)}, ({assoc}, ({arity}, ({over}, {ovf})))))).")
A("")
A("(* The seven mechanical row projections. *)")
A("Definition row_key (r : oprow) : key := fst r.")
A("Definition row_opid (r : oprow) : opid := fst (snd r).")
A("Definition row_precedence (r : oprow) : nat := fst (snd (snd r)).")
A("Definition row_assoc (r : oprow) : assoc := fst (snd (snd (snd r))).")
A("Definition row_arity (r : oprow) : arity := fst (snd (snd (snd (snd r)))).")
A("Definition row_overloadable (r : oprow) : bool :=")
A("  fst (snd (snd (snd (snd (snd r))))).")
A("Definition row_overflow (r : oprow) : overflow :=")
A("  snd (snd (snd (snd (snd (snd r))))).")
A("")
A("(* The closed row_at dispatch: index -> row, total by the pinned final arm. *)")
A("(* Kernel-side shape: S-nested nat match (single O(i) walk); the extractor  *)")
A("(* emits the pinned row_at -> array-index mapping instead of this body. *)")
A("Definition row_at (i : nat) : oprow :=")
A("  match i with")
A("  | 0 => row_00")
for i in range(1, 44):
    pat = "S (" * i + "0" + ")" * i
    A(f"  | {pat} => row_{i:02d}")
A("  | _ => row_43")
A("  end.")
A("")
A("(* The closed 44-variant enumeration in volume row order (declaration order). *)")
A("Definition all_opids : list opid := [" + "; ".join(decl_order) + "].")
A("")
A("(* The pinned table: the 44 rows in sorted order (the lookup array). *)")
A("Definition operator_table_sorted : list oprow :=")
A("  [" + "; ".join(f"row_{i:02d}" for i in range(44)) + "].")
A("")

open(OUT, "w", encoding="utf-8").write("\n".join(L))
print(f"OK: wrote {OUT} (44 rows, source sha256 {sha[:16]}...)")
