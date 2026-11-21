(* CEP:FILE: rocq/Effect/CharClass.v
   CEP:WHAT: The Rocq foundation layer of the pinned ASCII character-class predicates of Planck 0.1 edition P1 (Volume 02 §2.2.0, SPEC-INDEX §5.10): whitespace, decimal digit, hexadecimal digit, identifier-start, identifier-continue, and the line/block comment-introducer lead-byte predicates, over the pinned foundation byte domain {0..=255}, plus the pinned explicit range-form characterizations and the complete-domain kernel laws that bind the two. Extraction unit under the subset v2 accept-list (CEP-DR-01 §8 DR-8.8): every computational Definition here is mechanically extracted to Rust by planck-extractor; the Lemma..Qed blocks are Prop-erased and never extracted.
   CEP:WHY: DR-1.4 requires proof-to-Rust extraction and the standing user standard bans hand-written compiler code; the frontend character classes are DO-1 phase-1 foundation semantics (VR-14(d)), so their definition of record must be kernel-checked Coq whose extracted Rust is differentially verified against the kernel over the complete 256-byte domain (DR-8.3/DR-8.8 v2.9) and against the reference kernel's truth tables (three-way agreement).
   CEP:STATUS: complete
   CEP:FAILURE: A predicate that disagrees with its pinned range-form characterization for any of the 256 byte values fails the kernel laws below (vm_compute, zero admits); a divergence between the extracted Rust and this file fails the DIF over the complete 256-byte domain and the build. There is no other failure mode: every predicate is total and pure.
   CEP:ASSUMES: Coq 8.20.1 (CEP-DR-01 §6 pin); the pinned byte domain {0..=255} with the u8 extraction mapping (DR-8.8 v2.8/v2.9); Volume 02 §2.2.0 as the normative class sets; Volume 02 §2.2.15 for the comment introducer pairs (the lead-byte predicate is the pinned single-byte contract); stdlib lemmas forallb_forall/in_seq/lia are axiom-free (coqchk-verified per build).
   CEP:COST: kernel vm_compute over 256 bytes x 9 checks per law run; extraction build-time only; extracted predicates are O(1).
   CEP:EVIDENCE: the Lemmas below (complete-domain, zero admits); DIF ground truth generated from this file by planck-extractor (kernel vm_compute, 256 entries per tabulated function); three-way agreement with math/crates/planck-charclass truth tables recorded in the DIF artifact; coqchk Axioms: none per rocq/build.sh.
   CEP:SECURITY: no secrets; repository-trusted inputs; hand edits to extracted output are non-conforming (DR-8.2) — this file is the source of truth.
   CEP:FV-CLASS: FV-1 (kernel-checked laws, complete finite domain) + FV-0-equivalent-by-DIF for the extracted Rust
   CEP:FV-PROVER: Coq 8.20.1 kernel (coqc + coqchk, zero axioms)
   CEP:FV-SCRIPT: rocq/build.sh; tools/extractor (extract/dif/check/lint-cep)
   CEP:FV-SORRY-COUNT: 0
   CEP:FV-REFINEMENT: complete-domain: the laws quantify over all b with Nat.ltb b 256 = true (exactly the pinned {0..=255} target domain, DR-8.8 v2.9), discharged by exhaustive vm_compute over seq 0 256 plus the constructive forallb_forall/in_seq lift; no sampling, no gap within the pinned domain.
   CEP:FV-AXIOMS: none *)

Require Import Bool.
Require Import Arith.
Require Import List.
Require Import Lia.

(* ------------------------------------------------------------------ *)
(* The pinned foundation byte domain (subset v2.1; extraction target   *)
(* u8 per DR-8.8 v2.8; the DIF covers the complete target domain).     *)
(* ------------------------------------------------------------------ *)

Definition byte : Type := nat.

(* ------------------------------------------------------------------ *)
(* Pinned named constants (Law 7: constants are named, never magic     *)
(* literals; values are Volume 02 §2.2.0 verbatim).                    *)
(* ------------------------------------------------------------------ *)

Definition TAB : byte := 9.
Definition LF : byte := 10.
Definition CR : byte := 13.
Definition SPACE : byte := 32.
Definition SLASH : byte := 47.
Definition UNDERSCORE : byte := 95.
Definition DEC_DIGIT_MIN : byte := 48.
Definition DEC_DIGIT_MAX : byte := 57.
Definition UPPER_HEX_MIN : byte := 65.
Definition UPPER_HEX_MAX : byte := 70.
Definition LOWER_HEX_MIN : byte := 97.
Definition LOWER_HEX_MAX : byte := 102.
Definition UPPER_MIN : byte := 65.
Definition UPPER_MAX : byte := 90.
Definition LOWER_MIN : byte := 97.
Definition LOWER_MAX : byte := 122.

(* The single lead byte of both pinned comment introducers: the byte    *)
(* pairs 0x2F 0x2F (line) and 0x2F 0x2A (block), Volume 02 §2.2.0/§2.2.15. *)
Definition LINE_COMMENT_INTRODUCER_BYTE : byte := SLASH.
Definition BLOCK_COMMENT_INTRODUCER_BYTE : byte := SLASH.

(* ------------------------------------------------------------------ *)
(* The pinned predicates: total, pure, byte -> bool. These are the     *)
(* computational content; planck-extractor emits them as Rust fns.     *)
(* ------------------------------------------------------------------ *)

Definition is_whitespace (b : byte) : bool :=
  orb (Nat.eqb b TAB) (orb (Nat.eqb b LF) (orb (Nat.eqb b CR) (Nat.eqb b SPACE))).

Definition is_decimal_digit (b : byte) : bool :=
  andb (Nat.leb DEC_DIGIT_MIN b) (Nat.leb b DEC_DIGIT_MAX).

Definition is_hex_digit (b : byte) : bool :=
  orb (is_decimal_digit b)
      (orb (andb (Nat.leb UPPER_HEX_MIN b) (Nat.leb b UPPER_HEX_MAX))
           (andb (Nat.leb LOWER_HEX_MIN b) (Nat.leb b LOWER_HEX_MAX))).

Definition is_ident_start (b : byte) : bool :=
  orb (andb (Nat.leb UPPER_MIN b) (Nat.leb b UPPER_MAX))
      (orb (andb (Nat.leb LOWER_MIN b) (Nat.leb b LOWER_MAX))
           (Nat.eqb b UNDERSCORE)).

Definition is_ident_continue (b : byte) : bool :=
  orb (is_ident_start b) (is_decimal_digit b).

Definition is_line_comment_start (b : byte) : bool := Nat.eqb b SLASH.

Definition is_block_comment_start (b : byte) : bool := Nat.eqb b SLASH.

(* ------------------------------------------------------------------ *)
(* Pinned explicit set characterizations: the definition-of-record      *)
(* membership sets written independently as range unions over the      *)
(* pinned literals, so agreement with the predicates is a real          *)
(* computation (not a definitional tautology).                          *)
(* ------------------------------------------------------------------ *)

Definition ws_truth (b : byte) : bool :=
  orb (andb (Nat.leb 9 b) (Nat.leb b 10))
      (orb (Nat.eqb b 13) (Nat.eqb b 32)).

Definition dec_truth (b : byte) : bool :=
  andb (Nat.leb 48 b) (Nat.leb b 57).

Definition hex_truth (b : byte) : bool :=
  orb (andb (Nat.leb 48 b) (Nat.leb b 57))
      (orb (andb (Nat.leb 65 b) (Nat.leb b 70))
           (andb (Nat.leb 97 b) (Nat.leb b 102))).

Definition ids_truth (b : byte) : bool :=
  orb (andb (Nat.leb 65 b) (Nat.leb b 90))
      (orb (andb (Nat.leb 97 b) (Nat.leb b 122))
           (Nat.eqb b 95)).

Definition idc_truth (b : byte) : bool :=
  orb (ids_truth b) (dec_truth b).

Definition comment_truth (b : byte) : bool := Nat.eqb b 47.

(* ------------------------------------------------------------------ *)
(* Pinned laws (kernel-checked, complete finite domain, zero admits).  *)
(* ------------------------------------------------------------------ *)

(* The constructive lift: a checker true over the full enumeration is   *)
(* true at every point of the pinned domain. (forallb_forall/in_seq     *)
(* are stdlib, axiom-free; lia closes the index bound.)                 *)
Lemma lift_complete_domain : forall (f : byte -> bool),
  forallb f (seq 0 256) = true ->
  forall b : byte, Nat.ltb b 256 = true -> f b = true.
Proof.
  intros f Hf b Hb.
  destruct (forallb_forall f (seq 0 256)) as [Hfwd _].
  apply (Hfwd Hf).
  apply in_seq.
  apply Nat.ltb_lt in Hb. lia.
Qed.

(* L1: identifier-continue is exactly identifier-start ∪ decimal-digit  *)
(* (Volume 02 §2.2.0 IDENT_CONT; definitional, holds for every byte).   *)
Lemma ident_continue_is_the_union : forall b : byte,
  is_ident_continue b = orb (is_ident_start b) (is_decimal_digit b).
Proof. reflexivity. Qed.

(* L2: both comment introducers share the one lead-byte predicate      *)
(* (Volume 02 §2.2.15; definitional, holds for every byte).             *)
Lemma comment_predicates_identical : forall b : byte,
  is_line_comment_start b = is_block_comment_start b.
Proof. reflexivity. Qed.

(* L3: every predicate agrees with its pinned set characterization over *)
(* the complete 256-byte domain (Volume 02 §2.2.0; exhaustive).         *)
Lemma predicates_agree_with_sets_complete_domain :
  forallb (fun b => andb (eqb (is_whitespace b) (ws_truth b))
         (andb (eqb (is_decimal_digit b) (dec_truth b))
         (andb (eqb (is_hex_digit b) (hex_truth b))
         (andb (eqb (is_ident_start b) (ids_truth b))
         (andb (eqb (is_ident_continue b) (idc_truth b))
         (andb (eqb (is_line_comment_start b) (comment_truth b))
               (eqb (is_block_comment_start b) (comment_truth b))))))))
          (seq 0 256) = true.
Proof. vm_compute. reflexivity. Qed.

(* The constructive equality lift: pointwise bool-equality agreement over
   the full enumeration lifts to pointwise equality on the pinned domain. *)
Lemma lift_complete_domain_eq : forall (f g : byte -> bool),
  forallb (fun b => eqb (f b) (g b)) (seq 0 256) = true ->
  forall b : byte, Nat.ltb b 256 = true -> f b = g b.
Proof.
  intros f g Hf b Hb.
  apply eqb_prop.
  destruct (forallb_forall (fun b => eqb (f b) (g b)) (seq 0 256)) as [Hfwd _].
  apply (Hfwd Hf).
  apply in_seq. apply Nat.ltb_lt in Hb. lia.
Qed.

(* L4..L10: the pointwise guarded forms (the pinned domain {0..=255}).  *)
Lemma whitespace_law : forall b : byte,
  Nat.ltb b 256 = true -> is_whitespace b = ws_truth b.
Proof. apply lift_complete_domain_eq. vm_compute. reflexivity. Qed.

Lemma decimal_digit_law : forall b : byte,
  Nat.ltb b 256 = true -> is_decimal_digit b = dec_truth b.
Proof. apply lift_complete_domain_eq. vm_compute. reflexivity. Qed.

Lemma hex_digit_law : forall b : byte,
  Nat.ltb b 256 = true -> is_hex_digit b = hex_truth b.
Proof. apply lift_complete_domain_eq. vm_compute. reflexivity. Qed.

Lemma ident_start_law : forall b : byte,
  Nat.ltb b 256 = true -> is_ident_start b = ids_truth b.
Proof. apply lift_complete_domain_eq. vm_compute. reflexivity. Qed.

Lemma ident_continue_law : forall b : byte,
  Nat.ltb b 256 = true -> is_ident_continue b = idc_truth b.
Proof. apply lift_complete_domain_eq. vm_compute. reflexivity. Qed.

(* L11: the comment introducer lead byte is exactly 0x2F and no neighbor *)
(* byte is a lead byte (the reference exh_comment_introducer_byte_is_0x2f *)
(* contract, over the complete domain).                                  *)
Lemma comment_introducer_byte_is_0x2f : forall b : byte,
  Nat.ltb b 256 = true ->
  andb (eqb (is_line_comment_start b) (Nat.eqb b 47))
       (eqb (is_block_comment_start b) (Nat.eqb b 47)) = true.
Proof.
  apply lift_complete_domain.
  vm_compute. reflexivity.
Qed.
