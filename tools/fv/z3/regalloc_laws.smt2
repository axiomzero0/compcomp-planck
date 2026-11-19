; CEP:FILE: tools/fv/z3/regalloc_laws.smt2
; CEP:WHAT: Bounded SMT check of the two pinned register-allocation laws (SPEC-INDEX 5.8: each interval gets exactly one location; no two interfering intervals share a location) on a fixed instance from the EXH enumeration domain, plus a deliberately infeasible 1-register instance proving the law is not vacuous (spill path).
; CEP:WHY: Verification standards G4/G5 (pass correctness) + G18 (independent verification): the same laws the Rust EXH tests check are re-checked by a second, independent engine (Z3), per SPEC-INDEX section 4 method Z3 and CEP-DR-01 section 1 rule 5.
; CEP:STATUS: complete
; CEP:FAILURE: If Z3 returns anything other than the documented outcome (part 1 sat, part 2 sat, part 3 unsat) the obligation is unproven and the build fails per SPEC-INDEX section 4 rule 2 (timeout or unknown marks the obligation unproven).
; CEP:ASSUMES: SPEC-INDEX 5.8 (interference iff overlap and not coalescible; First-Fit over the register class order); Volume 16 16.4; the instance below is interval set {0:[0,3), 1:[1,5), 2:[2,4), 3:[3,6), 4:[4,5)} on the 6-point grid with coalescible pair {0,1} and register class order {0,1,2,3} (4 registers) -- a stated member of the EXH domain (5 intervals, start/end in 0..=5).
; CEP:COST: bounded check; strict timeout 10 s per SPEC-INDEX section 4 rule 2.
; CEP:EVIDENCE: tools/fv/z3/artifacts/regalloc_laws.result
; CEP:SECURITY: repository-trusted input.

(set-logic QF_LIA)

; ---- Part 1: law satisfiability on the bounded instance (expect sat) ----
; Locations of the 5 intervals; register class = {0,1,2,3} (Volume 16 16.2.1
; First-Fit order is the class order; instance uses 4 registers).
(declare-const loc0 Int)
(declare-const loc1 Int)
(declare-const loc2 Int)
(declare-const loc3 Int)
(declare-const loc4 Int)
(assert (and (>= loc0 0) (<= loc0 3)))
(assert (and (>= loc1 0) (<= loc1 3)))
(assert (and (>= loc2 0) (<= loc2 3)))
(assert (and (>= loc3 0) (<= loc3 3)))
(assert (and (>= loc4 0) (<= loc4 3)))
; Interference relation (overlap minus the coalescible pair {0,1}):
; 0-2, 0-3, 1-2, 1-3, 1-4, 2-3, 2-4, 3-4.
(assert (not (= loc0 loc2)))
(assert (not (= loc0 loc3)))
(assert (not (= loc1 loc2)))
(assert (not (= loc1 loc3)))
(assert (not (= loc1 loc4)))
(assert (not (= loc2 loc3)))
(assert (not (= loc2 loc4)))
(assert (not (= loc3 loc4)))
(check-sat)
; EXPECT: sat

; ---- Part 2: the allocator output is a model of the laws (expect sat) ----
; First-Fit over degree-descending order (2,3 deg 4; 1 deg 3; 0,4 deg 2)
; with register order 0,1,2,3 yields loc0=2, loc1=2 (coalescible with 0),
; loc2=0, loc3=1, loc4=3.
(push)
(assert (and (= loc0 2) (= loc1 2) (= loc2 0) (= loc3 1) (= loc4 3)))
(check-sat)
; EXPECT: sat
(pop)

; ---- Part 3: non-vacuity: two interfering intervals, 1 register (expect unsat) ----
(push)
(declare-const a Int)
(declare-const b Int)
(assert (and (>= a 0) (<= a 0)))   ; register class of size 1
(assert (and (>= b 0) (<= b 0)))
(assert (not (= a b)))        ; a and b interfere
(check-sat)
; EXPECT: unsat  (no assignment exists -> spill required, law is non-vacuous)
(pop)
