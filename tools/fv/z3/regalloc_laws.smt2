; CEP:FILE: tools/fv/z3/regalloc_laws.smt2
; CEP:WHAT: Bounded SMT check of the two pinned register-allocation laws (SPEC-INDEX 5.8: each interval gets exactly one location; no two interfering intervals share a location) on a concrete instance from the EXH census domain, plus a deliberately infeasible 1-register instance proving the law is not vacuous (spill path).
; CEP:WHY: Verification standards G4/G5 (pass correctness) + G18 (independent verification): the same laws the Rust EXH tests check are re-checked by a second, independent engine (Z3), per SPEC-INDEX section 4 method Z3 and CEP-DR-01 section 1 rule 5.
; CEP:STATUS: complete
; CEP:FAILURE: If Z3 returns anything other than the documented outcome (part 1 sat, part 2 sat, part 3 unsat) the obligation is unproven and the build fails per SPEC-INDEX section 4 rule 2 (timeout or unknown marks the obligation unproven).
; CEP:ASSUMES: SPEC-INDEX 5.8 (interference iff HALF-OPEN overlap and not coalescible: [s1,e1) and [s2,e2) overlap iff s1 < e2 and s2 < e1; First-Fit over the register class order); Volume 16 16.4; the instance below is interval set {0:[0,3), 1:[1,5), 2:[2,4), 3:[3,6), 4:[4,5)} on the 6-point grid with coalescible pair {0,1} (either direction suffices per the pinned "at least one is not coalescible") and register class order {0,1,2,3} (4 registers) -- a stated member of the EXH census (5 intervals, start/end in 0..=5).
; CEP:COST: bounded check; strict timeout 10 s (set below, enforced by the solver; also documented per SPEC-INDEX section 4 rule 2).
; CEP:EVIDENCE: tools/fv/z3/artifacts/regalloc_laws.result
; CEP:SECURITY: repository-trusted input.

(set-option :timeout 10000)
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
; HALF-OPEN overlap on {0:[0,3), 1:[1,5), 2:[2,4), 3:[3,6), 4:[4,5)}:
;   0&1: [1,3) overlap; coalescible pair {0,1} -> NOT interfering (either-direction rule)
;   0&2: [2,3) overlap, not coalescible -> interfere
;   0&3: [3,3) empty, 0&4: empty -> no constraint
;   1&2: [2,4) interfere; 1&3: [3,5) interfere; 1&4: [4,5) interfere
;   2&3: [3,4) interfere; 2&4: [4,4) empty -> no constraint; 3&4: [4,5) interfere
(assert (not (= loc0 loc2)))
(assert (not (= loc1 loc2)))
(assert (not (= loc1 loc3)))
(assert (not (= loc1 loc4)))
(assert (not (= loc2 loc3)))
(assert (not (= loc3 loc4)))
(check-sat)
; EXPECT: sat

; ---- Part 2: the kernel's First-Fit output is a model of the laws (expect sat) ----
; planck-regalloc-math allocate() on this exact instance (degree-descending,
; start-ascending order 1,2,3,4,0; register order 0,1,2,3) yields
; loc1=0, loc2=1, loc3=2, loc4=1, loc0=0 (coalescible 0&1 share register 0;
; 0 interferes only with 2=1). Recorded here so the artifact verifies the
; kernel's actual output, not an invented model.
(push)
(assert (and (= loc0 0) (= loc1 0) (= loc2 1) (= loc3 2) (= loc4 1)))
(check-sat)
; EXPECT: sat
(pop)

; ---- Part 3: non-vacuity: two interfering intervals, 1 register (expect unsat) ----
(push)
(declare-const a Int)
(declare-const b Int)
(assert (and (>= a 0) (<= a 0)))   ; register class of size 1
(assert (and (>= b 0) (<= b 0)))
(assert (not (= a b)))             ; a and b interfere
(check-sat)
; EXPECT: unsat  (no assignment exists -> spill required, law is non-vacuous)
(pop)
