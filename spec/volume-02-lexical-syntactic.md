<!--
CEP:FILE: spec/volume-02-lexical-syntactic.md
CEP:WHAT: Volume 02 of the Planck Master Specification Library: the complete lexical, literal, operator, grammar, and parser specification for Planck 0.1, edition P1. Defines design.md §2 (lines 118-232): source files, tokens, literal semantics, operator specification, grammar, parsing behavior.
CEP:WHY: design.md §2 requires a complete normative lexical and syntactic definition; the pinned contracts SPEC-INDEX §5.10 (character classes), §5.5 (operator table columns), and §5.6 (literal math) are elaborated here exactly and are embedded as const data by planck-charclass, planck-token-math, and planck-operator-math.
CEP:STATUS: complete
CEP:FAILURE: A tokenizer, parser, or kernel crate that diverges from a table or production here is non-conforming. A missing coverage-ledger row (§2.9) is a missing definition. Any "appropriate"/"as needed"/TBD phrasing here is a defect.
CEP:ASSUMES: SPEC-INDEX.md §0-§5 (esp. §5.5, §5.6, §5.10 pinned contracts); standards.md §3-§4; verification standards.md §2 (G1-G5); doc standards.md; design.md lines 118-232; Volume 01 (charter), Volume 03 (module file loading referenced by §2.1.9).
CEP:COST: document; the tables herein are extracted as const arrays into math/planck-charclass, math/planck-token-math, math/planck-operator-math (equivalence EXH per SPEC-INDEX §5.1).
CEP:EVIDENCE: Verification matrix (§2.8) and Coverage ledger (§2.9) in this file; kernel test suites in math/.
-->

# Volume 02 — Lexical and syntactic specification

**Abstract.** This volume defines the byte-to-token pipeline (decoding, character classes, tokens), the literal value spaces and their rounding, the complete operator table, the grammar of edition P1, and the parser contract (determinism, recovery, incremental parsing). Every rule is exact: constants carry values, enums enumerate members, grammars are productions, algorithms state domain, output, termination, and complexity.

## 2.0 Scope and traceability

| Field | Value |
| ----- | ----- |
| design.md sections defined | §2.1-§2.6 (lines 118-232) |
| Bullets in scope | 85 (§2.1: 9, §2.2: 21, §2.3: 11, §2.4: 12, §2.5: 21, §2.6: 11) |
| Prose requirements in scope | 2 (design.md line 122 "needs a complete normative definition"; line 231 deliverables) |
| Deliverables satisfied | Lexical Specification (§2.1-§2.3), Grammar Specification (§2.5), Operator Specification (§2.4), Parser Contract (§2.6) |
| Kernel extraction | planck-charclass (§2.2.0), planck-token-math (§2.2.7, §2.2.10, §2.3), planck-operator-math (§2.4.1) |

design.md line 122 requires that the lexical draft becomes a complete normative definition: this file is that definition; any prior draft is superseded and has no normative standing.

## 2.1 Source files

### 2.1.1 Source encoding (design.md L126)

A source file is a sequence of bytes that must be valid UTF-8 (RFC 3629 encoding form). After decoding, the stream is a sequence of Unicode scalar values. Restriction (pinned, SPEC-INDEX §5.10): non-ASCII scalars are permitted only inside comments, string literals, raw string literals, and char literals. A non-ASCII scalar encountered in any other token position is error `E-LEX-01` at that scalar's offset.

### 2.1.2 Unicode version (design.md L127)

The Unicode version is **Unicode 15.0.0**. It affects exactly: scalar-value validity for `\u{...}` escapes and char/string literal contents. Identifiers are ASCII (§2.2.2), so no identifier algorithm depends on it. Changing the Unicode version requires a version-bump note in this file and re-running the literal test suites.

### 2.1.3 Byte-order mark handling (design.md L128)

`U+FEFF` at byte offset 0 is skipped and produces no token. `U+FEFF` at any other offset is error `E-LEX-09` at that offset.

### 2.1.4 Line endings (design.md L129)

At decode time, the two-scalar sequence `U+000D U+000A` is replaced by `U+000A`; this applies everywhere, including raw strings. A `U+000D` not followed by `U+000A` is error `E-LEX-10` at its offset. After decoding, the only line terminator is `U+000A`.

### 2.1.5 Invalid encoding behavior (design.md L130)

The first invalid UTF-8 byte sequence is error `E-LEX-11` with the file ID and byte offset. Decoding is all-or-nothing per file: a file with `E-LEX-11` yields no tokens, no further diagnostics, and no downstream phases.

### 2.1.6 Source locations (design.md L131)

A **source location** is `{ file: u32 (FileId), byte_offset: u64 (0-based, into the decoded stream), line: u32 (1-based), column: u32 (1-based) }`. A **span** is the half-open interval `[start, end)` of two locations. Every diagnostic carries exactly one primary span and 0..=8 secondary spans (constant `MAX_SECONDARY_SPANS = 8`). `line = 1 + (number of U+000A before byte_offset)`.

### 2.1.7 Column numbering (design.md L132)

Column counts Unicode scalar values from the start of the line, starting at 1. A tab advances the column by exactly 1 (no tab stops). Combining marks count as separate columns. Byte offsets are reported alongside columns; columns are never derived from bytes.

### 2.1.8 Tabs and indentation (design.md L133)

Tabs (U+0009) and spaces (U+0020) are both whitespace, are interchangeable, and have no grammatical significance. Indentation is never significant. There is no line-length limit.

### 2.1.9 File inclusion rules (design.md L134)

No textual inclusion directive exists; there is no preprocessor. The only file composition is module loading by `mod` declarations, defined exactly in Volume 03 §4.1. Each file must be loaded by at most one `mod` (Volume 03: `E-DEC-09`); the crate root is loaded once. Loading is acyclic by construction of the module tree.

**Integration points.** Depends on: Volume 03 §4 (module files), Volume 21 (diagnostic rendering). Depended on by: everything downstream (tokens are the only input to parsing).

## 2.2 Tokens

### 2.2.0 Pinned character classes (SPEC-INDEX §5.10, embedded exactly)

The classification predicates are total functions `u8 → bool` over source bytes, embedded verbatim as `planck-charclass`:

| Predicate | Exact set |
| --------- | --------- |
| `is_whitespace` | {0x09, 0x0A, 0x0D, 0x20} |
| `is_dec_digit` | {0x30..=0x39} |
| `is_hex_digit` | {0x30..=0x39} ∪ {0x41..=0x46} ∪ {0x61..=0x66} |
| `is_ident_start` | {0x41..=0x5A} ∪ {0x61..=0x7A} ∪ {0x5F} |
| `is_ident_continue` | `is_ident_start` ∪ `is_dec_digit` |
| line-comment introducer | byte pair 0x2F 0x2F (`//`) |
| block-comment introducer | byte pair 0x2F 0x2A (`/*`) |

Non-ASCII policy is §2.1.1: non-ASCII in string/char literals and comments only; identifiers are ASCII in edition P1.

### 2.2.1 Identifier grammar (design.md L138)

```
Identifier   = IDENT_START IDENT_CONT* ;
IDENT_START  = "A".."Z" | "a".."z" | "_" ;
IDENT_CONT   = IDENT_START | "0".."9" ;
```

The token consisting of exactly `_` is classed **Wildcard**, not Identifier. An identifier's length is unbounded (memory-bounded only). Identifiers are ASCII by construction (§2.2.2).

### 2.2.2 Unicode identifier rules (design.md L139)

Edition P1 admits exactly the ASCII classes of §2.2.0; UAX #31 is not used. A future edition may widen the classes; widening is an edition change (G26) and must update this section, `planck-charclass`, and the ledger row of design.md L139.

### 2.2.3 Identifier normalization (design.md L140)

There is no normalization: identifier identity is byte equality. Two identifiers are the same name if and only if their byte sequences are equal. Case is significant everywhere.

### 2.2.4 Reserved keywords (design.md L141)

The reserved keyword set is exactly these 41 names; a keyword token is produced whenever an identifier scan matches one:

`as, async, asm, await, break, const, continue, crate, dyn, else, enum, extern, false, fn, for, if, impl, in, let, loop, macro, match, mod, move, mut, pub, ref, return, self, Self, static, struct, super, trait, true, type, union, unsafe, use, where, while`

### 2.2.5 Future-reserved keywords (design.md L142)

The future-reserved set is exactly these 13 names; they are lexer keywords (produce a token) and are rejected by the parser with `E-PAR-01` naming the reservation:

`try, gen, yield, abstract, become, box, do, final, override, priv, typeof, unsized, virtual`

Promoting a future-reserved keyword to a keyword is an edition change; using one as an identifier is impossible in P1.

### 2.2.6 Context-sensitive restrictions (design.md L143)

There are none. The keyword set is context-independent; there are no soft keywords and no raw identifiers (`r#name` is not grammar; the sequence `r#` followed by an identifier is a raw-string or parse error). `self` is usable only in receiver parameter position (grammar-restricted, §2.5.6), which is a grammar constraint, not context sensitivity. Primitive type names (`bool, char, str, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize, f32, f64`) are not keywords; they are reserved item names with rules in Volume 03 §3.3.6.

### 2.2.7 Integer literals (design.md L144)

```
IntegerLiteral  = RadixPrefix? DigitGroup [ "_" ] IntegerSuffix? ;
RadixPrefix     = "0x" | "0o" | "0b" ;
DigitGroup      = Digit { "_" Digit } ;        -- underscores only between digits
Digit           = DecDigit                     -- decimal, or per radix below
                | HexDigit                     -- 0x
                | OctDigit                     -- 0o, {0..7}
                | BinDigit ;                   -- 0b, {0,1}
IntegerSuffix   = "i8"|"i16"|"i32"|"i64"|"i128"|"u8"|"u16"|"u32"|"u64"|"u128"
                | "usize" | "isize" ;
```

Rules: the radix prefix must be followed by ≥ 1 digit valid for the radix; the `_` before the suffix is allowed exactly once (e.g. `1_u8`); a digit group must not start or end with `_` and must not contain two adjacent `_`. A numeric token immediately followed by an `IDENT_START` byte is `E-LEX-06`. Value space: §2.3.1. Constant `MAX_LIT_DIGITS = 4096`: a longer digit string is `E-LEX-12`.

### 2.2.8 Floating-point literals (design.md L145)

```
FloatLiteral    = FloatBody FloatSuffix? ;
FloatBody       = DigitGroup "." DigitGroup Exp?
                | DigitGroup Exp ;
Exp             = ("e"|"E") ["+"|"-"] DigitGroup ;
FloatSuffix     = "f32" | "f64" ;
```

Validity: a float literal must contain either a fraction part with ≥ 1 digit after `.`, or an exponent with ≥ 1 digit, or a float suffix (`1f32` is valid; `1.` is not a token). Underscore rules are those of §2.2.7. The scan uses maximal valid prefix: if the longest candidate fails validity, the tokenizer retries shorter prefixes; if none is valid, `E-LEX-06`. Value space and rounding: §2.3.4.

### 2.2.9 Literal suffixes (design.md L146)

The suffix table is exactly the 14 suffixes above. A suffix must directly follow the digits (whitespace between is a different token sequence). Constraints: integer suffixes attach only to integer literals; float suffixes only to float literals; other combinations are `E-LEX-06`. An unsuffixed integer literal defaults to type `i32`; an unsuffixed float literal defaults to `f64` (§2.3.2). Suffix semantics: the suffix fixes the inferred type; it never changes the digit value.

### 2.2.10 Character literals (design.md L147)

```
CharLiteral     = "'" ( CharBody | Escape ) "'" ;
CharBody        = ScalarValue - { "'", "\", U+000A, U+000D } ;
ByteCharLiteral = "b'" ( ByteCharBody | ByteEscape ) "'" ;
ByteCharBody    = ScalarValue - { "'", "\", U+000A, U+000D } restricted to U+0000..U+007F ;
ByteEscape      = "\x" HexDigit HexDigit ;
```

`ByteEscape` values 0x00..=0xFF are valid in byte contexts; in char/string contexts the same form is valid only for 0x00..=0x7F. A byte char with content > 0x7F outside `ByteEscape` is `E-LIT-06`. A char literal whose body is not exactly one scalar value is `E-LEX-07`.

### 2.2.11 String literals (design.md L152 of design.md context; bullet L148)

```
StringLiteral   = '"' ( StringChar | Escape )* '"' ;
StringChar      = ScalarValue - { '"', "\" } ;
```

Line feeds are permitted inside string literals (no line-spanning restriction); carriage returns cannot occur (§2.1.4 normalization). Contents are UTF-8 by construction of the source (§2.3.8).

### 2.2.12 Byte strings (design.md L149)

```
ByteString      = "b\"" ( ByteCharBody | ByteEscape )* '"' ;
```

Byte string contents must be ASCII (U+0000..U+007F) or `ByteEscape`; any other scalar is `E-LIT-06`. A byte string has type `[u8; N]` with N its element count (Volume 04).

### 2.2.13 Raw strings (design.md L150)

```
RawString       = "r" HASH* '"' RawContent '"' HASH* ;
RawByteString   = "br" HASH* '"' RawContent '"' HASH* ;
HASH            = "#" ;
RawContent      = any scalars except the terminator pattern '"' HASH* '"' of the same count ;
```

The number of opening `#` must equal the number of closing `#`; a mismatch at end of file is `E-LEX-03`. No escapes exist inside raw content. `RawByteString` additionally forbids non-ASCII content (`E-LIT-06`). Line-ending normalization applies inside raw strings (§2.1.4).

### 2.2.14 Escape sequences (design.md L151)

The complete escape set:

| Escape | Value | Contexts |
| ------ | ----- | -------- |
| `\n` | U+000A | char, string |
| `\t` | U+0009 | char, string |
| `\r` | U+000D | char, string |
| `\\` | U+005C | char, string |
| `\0` | U+0000 | char, string |
| `\'` | U+0027 | char, string |
| `\"` | U+0022 | char, string |
| `\xHH` | byte HH | char/string: 0x00..=0x7F; byte contexts: 0x00..=0xFF |
| `\u{H{1,6}}` | scalar | char, string; must be a scalar value (≤ U+10FFFF, not surrogate), else `E-LIT-04` |

An escape not in this table is `E-LEX-08`. A `\xHH` out of its context range is `E-LEX-08`.

### 2.2.15 Comments (design.md L152)

```
LineComment     = "//" LineChar* ;                       LineChar = any scalar except U+000A
BlockComment    = "/*" ( BlockComment | AnyScalar )* "*/" ;
```

Block comments nest; an unterminated block comment is `E-LEX-02` with the opening offset. `/*` and `*/` inside line comments have no effect; `//` inside block comments has no effect.

### 2.2.16 Documentation comments (design.md L153)

A line comment is a **doc-outer** comment iff it begins with exactly three `/` and the next scalar is not `/` (four or more slashes make an ordinary comment). A line comment is a **doc-inner** comment iff it begins with `//!`. `/**` and `/*!` are ordinary block comments. Semantics: a doc-outer comment is equivalent to the attribute `#[doc = "content"]` attached to the immediately following item; a doc-inner comment is equivalent to `#![doc = "content"]` of the enclosing module. Multiple doc comments attach in source order, joined with U+000A. Doc comments and attributes may interleave; all attach to the same item. A doc comment with no following item is warning `W-DOC-01`. Doc comments are consumed by the parser and are not part of the item grammar.

### 2.2.17 Attributes (design.md L154)

```
OuterAttr       = "#" "[" AttrItem "]" ;
InnerAttr       = "#!" "[" AttrItem "]" ;
AttrItem        = AttrPath [ AttrInput ] ;
AttrPath        = IDENT { "::" IDENT } ;
AttrInput       = "(" DelimTokenTrees ")" | "=" Literal ;
```

Placement: inner attributes are valid only before the first item of a file or module block; outer attributes are valid before items, struct/enum fields, and `let` statements. Attributes on expressions are not grammar in P1 (banned). Duplicate identical `AttrItem` on one node is warning `W-ATTR-01`; multiplicity semantics are per-attribute in Volume 10. The attribute name inventory is Volume 10 §15.1; unknown names are rejected there, not here.

### 2.2.18 Punctuation (design.md L155)

The punctuation token table, matched longest-first:

| Length | Tokens |
| ------ | ------ |
| 3 | `<<=` `>>=` |
| 2 | `::` `->` `=>` `==` `!=` `<=` `>=` `&&` `\|\|` `<<` `>>` `+=` `-=` `*=` `/=` `%=` `^=` `&=` `\|=` `..` `..=` |
| 1 | `+` `-` `*` `/` `%` `^` `!` `&` `\|` `=` `<` `>` `.` `,` `;` `:` `?` `#` `$` `@` `_` `(` `)` `[` `]` `{` `}` |

The token `_` is the Wildcard token (§2.2.1). `$` is valid only inside macro matcher/transcriber token trees (§2.5.20); `$` elsewhere is `E-LEX-01`.

### 2.2.19 Operators (design.md L156)

The operator inventory is exactly the 44-row table of §2.4.1; that table is the normative home of operator syntax, precedence, associativity, arity, overloadability, and overflow class.

### 2.2.20 Delimiters (design.md L157)

Delimiters are `(`, `)`, `[`, `]`, `{`, `}`. Each opening delimiter must be closed by the matching kind before end of file; an unclosed delimiter is `E-LEX-05` reporting the opening location; a closing delimiter with no match is `E-PAR-02`. Nesting depth must not exceed `MAX_PARSE_DEPTH = 128` (§2.6.4). Delimiter balancing is a lexer invariant: the parser never sees an unbalanced stream.

### 2.2.21 Tokenization precedence (design.md L158)

The tokenizer is one pass, left to right, and deterministic. At each position the first matching rule wins:

| Priority | Rule |
| -------- | ---- |
| 1 | Decode and normalize (§2.1); fail with `E-LEX-11` on invalid UTF-8. |
| 2 | Whitespace (§2.2.0); skip. |
| 3 | Comments: try `///`-doc (not followed by 4th `/`), `//!`, `//`, then nested `/*...*/`. |
| 4 | Identifier: maximal `IDENT_START IDENT_CONT*`; keyword tables of §2.2.4-§2.2.5 then classify it. |
| 5 | Char/lifetime: try `CharLiteral` maximal-valid; else `'` + `Identifier` is a Lifetime token. |
| 6 | Numeric: try `FloatLiteral`, then `IntegerLiteral`, maximal valid prefix (§2.2.7-§2.2.8). |
| 7 | Strings: try `RawByteString`, `RawString`, `ByteString`, `ByteCharLiteral`, `StringLiteral` in that prefix order. |
| 8 | Punctuation: longest match from the §2.2.18 table. |
| 9 | Otherwise: `E-LEX-01` at that offset. |

Output: a token stream where each token carries its span (§2.1.6). Identical input yields an identical stream (G14).

**Integration points.** Depends on: §2.1 decode. Depended on by: §2.5 parser; `planck-charclass`/`planck-token-math`/`planck-operator-math` embed the tables of §2.2.0, §2.2.7-§2.2.9, and §2.4.1 as const data (SPEC-INDEX §5.1).

## 2.3 Literal semantics

### 2.3.1 Integer literal interpretation (design.md L162)

Pinned (SPEC-INDEX §5.6). The literal carries radix `r ∈ {2, 8, 10, 16}` and digit string `d` with all digits valid for `r`. The value is the arbitrary-precision integer `v` computed by `v = fold over digits: v_{i+1} = v_i · r + digit_i`, starting `v_0 = 0`. A digit invalid for `r` is `E-LIT-01`. The function is total over valid digit strings.

### 2.3.2 Arbitrary-precision literal handling during type checking (design.md L163)

The literal is retained as `(digit string, radix, suffix?)` until its integer type `T` is fixed. Fixing order: (1) an explicit suffix fixes `T`; (2) otherwise the context's inferred integer type (Volume 04) fixes `T`; (3) otherwise `T = i32`. The literal is never truncated before `T` is fixed; the digit string is the value of record.

### 2.3.3 Integer range checking (design.md L164)

Pinned: the typed literal is well-formed iff `min(T) ≤ v ≤ max(T)`; violation is a compile-time rejection `E-LIT-02`, never a runtime event. Exact bounds:

| T | min | max |
| --- | --- | --- |
| i8 | -128 | 127 |
| i16 | -32768 | 32767 |
| i32 | -2147483648 | 2147483647 |
| i64 | -9223372036854775808 | 9223372036854775807 |
| i128 | -170141183460469231731687303715884105728 | 170141183460469231731687303715884105727 |
| u8 | 0 | 255 |
| u16 | 0 | 65535 |
| u32 | 0 | 4294967295 |
| u64 | 0 | 18446744073709551615 |
| u128 | 0 | 340282366920938463463374607431768211455 |
| usize / isize | per target pointer width (§1.4.8): 64-bit targets equal u64 / i64; the declared i686 target equals u32 / i32 | |

### 2.3.4 Floating-point rounding (design.md L165)

Pinned: a float literal's decimal value maps to the nearest IEEE-754 value of the literal's float type, round-to-nearest, ties-to-even. Unsuffixed and `f64` literals round to binary64; `f32` literals round to binary32. If the exact decimal magnitude is ≥ 2^1024 for f64 (2^128 for f32), the literal is rejected `E-LIT-03`; subnormal results are allowed and exact; underflow rounds to signed zero. The conversion is the correctly-rounded decimal-to-binary conversion of IEEE 754-2019.

### 2.3.5 NaN and infinity representation (design.md L166)

Pinned: NaN payloads and infinities are named forms, not literal spellings. The float grammar (§2.2.8) cannot produce infinity or NaN; the sequences `inf`, `nan`, `infinity` are identifiers. The constants `core::f64::INFINITY`, `core::f64::NAN`, `core::f32::INFINITY`, `core::f32::NAN` (Volume 19) are the spellings; payload construction is via the Volume 19 bit-pattern constructors.

### 2.3.6 Character validity (design.md L167)

Pinned: a char literal denotes exactly one Unicode scalar value; surrogates (U+D800..U+DFFF) are rejected with `E-LIT-05`.

### 2.3.7 Unicode scalar values (design.md L168)

A **Unicode scalar value** is a code point in U+0000..U+10FFFF excluding U+D800..U+DFFF. The `char` type's value set is exactly this set (Volume 04). `\u{...}` payloads outside the set are `E-LIT-04`.

### 2.3.8 UTF-8 validation (design.md L169)

String literal contents inherit source validity: the source decode (§2.1.1, §2.1.5) guarantees the bytes are valid UTF-8, so no separate string-content validation pass exists. `\u{...}` escapes are validated against the scalar-value set (§2.3.7, `E-LIT-04`).

### 2.3.9 Byte literal ranges (design.md L170)

A byte char denotes a `u8`: ASCII bodies (0x00..0x7F) or `ByteEscape` (0x00..0xFF). Byte string elements are `u8` with the same rule; any non-ASCII scalar in byte content is `E-LIT-06`.

### 2.3.10 Compile-time literal evaluation (design.md L171)

Each literal token is converted to its constant value at parse time and interned in the literal pool (deduplicated by `(type, value)`); the pool is the only literal representation downstream. A negative number is not a literal: `-5` is the unary minus operator applied to literal `5`; folding happens in const evaluation (Volume 09).

### 2.3.11 Invalid literal diagnostics (design.md L172)

The complete literal error set, each with its exact span (the offending sub-token):

| Code | Condition | Span |
| ---- | --------- | ---- |
| E-LIT-01 | digit invalid for radix | first bad digit |
| E-LIT-02 | integer value outside min(T)..max(T) | whole literal |
| E-LIT-03 | float value ≥ 2^1024 (f64) or ≥ 2^128 (f32) | whole literal |
| E-LIT-04 | `\u{...}` payload not a scalar value | escape braces |
| E-LIT-05 | char literal is a surrogate | whole literal |
| E-LIT-06 | non-ASCII scalar in byte content | offending scalar |
| E-LEX-06 | malformed numeric token or bad suffix combination | token start |
| E-LEX-07 | char literal not exactly one scalar | whole literal |
| E-LEX-08 | unknown escape or `\x` out of context range | escape start |
| E-LEX-12 | digit count > MAX_LIT_DIGITS | literal start |

**Integration points.** Depends on: §2.2 token grammar. Depended on by: `planck-token-math` (radix fold, range checks, float rounding: equivalence EXH/Z3 per SPEC-INDEX §5.1), Volume 04 (type fixing), Volume 09 (const pool).

## 2.4 Operator specification

### 2.4.1 The operator table (design.md L177-L180: syntax, precedence, associativity, overloadability; pinned columns of SPEC-INDEX §5.5)

The table has exactly the pinned columns: operator token, precedence level (integer, higher binds tighter), associativity (Left, Right, None), arity (1, 2), overloadable (bool), overflow behavior class (Panic, Wrap, Unchecked, N/A). It has exactly 44 rows; `planck-operator-math` embeds it as a const array and exposes lookup by binary search (equivalence EXH over the table domain). Postfix applicative forms are operators with receiver + argument as the two operands.

| Op | Token/position | Prec | Assoc | Arity | Overloadable | Overflow class |
| -- | -------------- | ---- | ----- | ----- | ------------ | -------------- |
| OP-01 | `=` | 0 | None | 2 | no | N/A |
| OP-02 | `+=` | 0 | None | 2 | no (desugar §2.4.6) | N/A |
| OP-03 | `-=` | 0 | None | 2 | no | N/A |
| OP-04 | `*=` | 0 | None | 2 | no | N/A |
| OP-05 | `/=` | 0 | None | 2 | no | N/A |
| OP-06 | `%=` | 0 | None | 2 | no | N/A |
| OP-07 | `&=` | 0 | None | 2 | no | N/A |
| OP-08 | `\|=` | 0 | None | 2 | no | N/A |
| OP-09 | `^=` | 0 | None | 2 | no | N/A |
| OP-10 | `<<=` | 0 | None | 2 | no | N/A |
| OP-11 | `>>=` | 0 | None | 2 | no | N/A |
| OP-12 | `a..b` | 1 | None | 2 | no | N/A |
| OP-13 | `a..=b` | 1 | None | 2 | no | N/A |
| OP-14 | `a..` | 1 | None | 1 | no | N/A |
| OP-15 | `..b` | 1 | None | 1 | no | N/A |
| OP-16 | `..` | 1 | None | 0 | no | N/A |
| OP-17 | `\|\|` | 2 | Left | 2 | no | N/A |
| OP-18 | `&&` | 3 | Left | 2 | no | N/A |
| OP-19 | `\|` (binary) | 4 | Left | 2 | yes | N/A |
| OP-20 | `^` (binary) | 5 | Left | 2 | yes | N/A |
| OP-21 | `&` (binary) | 6 | Left | 2 | yes | N/A |
| OP-22 | `==` | 7 | None | 2 | yes | N/A |
| OP-23 | `!=` | 7 | None | 2 | yes | N/A |
| OP-24 | `<` | 8 | None | 2 | yes | N/A |
| OP-25 | `>` | 8 | None | 2 | yes | N/A |
| OP-26 | `<=` | 8 | None | 2 | yes | N/A |
| OP-27 | `>=` | 8 | None | 2 | yes | N/A |
| OP-28 | `<<` | 9 | Left | 2 | yes | Panic |
| OP-29 | `>>` | 9 | Left | 2 | yes | Panic |
| OP-30 | `+` | 10 | Left | 2 | yes | Panic |
| OP-31 | `-` (binary) | 10 | Left | 2 | yes | Panic |
| OP-32 | `*` (binary) | 11 | Left | 2 | yes | Panic |
| OP-33 | `/` | 11 | Left | 2 | yes | Panic |
| OP-34 | `%` | 11 | Left | 2 | yes | Panic |
| OP-35 | `as` | 12 | Left | 2 | no | N/A |
| OP-36 | `-` (unary) | 13 | Right | 1 | yes | Panic |
| OP-37 | `!` (unary) | 13 | Right | 1 | yes | N/A |
| OP-38 | `*` (deref) | 13 | Right | 1 | no | N/A |
| OP-39 | `&` / `&mut` (borrow) | 13 | Right | 1 | no | N/A |
| OP-40 | `.` field | 14 | Left | 2 | no | N/A |
| OP-41 | `.name(args)` method call | 14 | Left | 2 | dispatch per Volume 04 | N/A |
| OP-42 | `[i]` index | 14 | Left | 2 | yes | N/A |
| OP-43 | `(args)` call | 14 | Left | 2 | dispatch per Volume 04 | N/A |
| OP-44 | `?` | 14 | Left | 1 | no | N/A |

The `Wrap` and `Unchecked` classes are not used by any row of this table; they are used exclusively by the arithmetic intrinsics of Volume 10, which carry the classes in that volume's intrinsic table. Unary `-` on unsigned types is a type error (Volume 04), not an overflow.

### 2.4.2 Operand types and result types (design.md L181-L182)

| Operator | Operand types (exact) | Result type |
| -------- | --------------------- | ----------- |
| OP-30..OP-34 | both operands the same integer type T, or (OP-30..OP-33 only) the same float type | T |
| OP-28, OP-29 | both operands the same integer type T | T |
| OP-19..OP-21 | both the same integer type T, or both `bool` | T |
| OP-22..OP-27 | both the same type T (T must implement the equality/order operator traits, Volume 04) | `bool` |
| OP-17, OP-18 | both `bool` | `bool` |
| OP-36 | signed integer or float type T | T |
| OP-37 | integer type T or `bool` | T |
| OP-38 | reference or raw pointer to U | place of U |
| OP-39 | any place type | `&T` / `&mut T` |
| OP-35 | source per cast table §2.4.7 | target type |
| OP-12..OP-16 | any type T | the range struct of Volume 19 §26.6 |
| OP-40..OP-43, OP-44 | per Volume 04 dispatch | per dispatch |

No implicit conversion exists between operand types (Volume 04 §6.3 is the only coercion list).

### 2.4.3 Evaluation order (design.md L183)

Uniform rule: for every operator, operands evaluate left to right, fully, before the operation. Exceptions, exactly: OP-17/OP-18 evaluate the left operand, then the right operand only if the left did not already determine the result; OP-43 evaluates the callee, then arguments left to right; OP-41 evaluates the receiver, then arguments left to right; OP-44 evaluates its operand, then propagates. Compound assignment evaluates the left place address once, then the right operand, then the operation, then the store.

### 2.4.4 Overflow semantics and panic behavior (design.md L184-L185)

For the `Panic` class on integer operands, the check conditions are exact:

| Operation | Panic condition | Exact panic message |
| --------- | --------------- | ------------------- |
| `+`, `-`, `*`, unary `-` | mathematical result outside min(T)..max(T) | `arithmetic overflow` |
| `/`, `%` | divisor = 0 | `division by zero` |
| `/`, `%` (signed) | dividend = min(T) and divisor = -1 | `arithmetic overflow` |
| `<<`, `>>` | shift amount ≥ bit-width of T, or negative | `shift overflow` |

In-range shift results: logical shift for unsigned types, arithmetic for signed types. Float division by zero produces IEEE ±inf or NaN per IEEE 754; it never panics (overflow class N/A). Panic unwinds to the enclosing catch boundary per Volume 07; `panic = abort` builds make the trap immediate (exit code 134). Every panic carries the `panic` effect atom unless `panic = abort` (pinned, SPEC-INDEX §5.3).

### 2.4.5 Effects and constant evaluation (design.md L186-L188)

Effects per row: OP-30..OP-34, OP-36, OP-28, OP-29 on integer operands carry `{panic}`; OP-38 on a raw pointer carries `{unsafe}`; OP-41 and OP-43 inherit the callee's effect row; OP-44 inherits the propagation semantics of Volume 07; all other rows are `∅`. Constant evaluation: every row except OP-01..OP-11 (assignment family) and OP-44 is const-evaluable (Volume 09). A Panic-class operation evaluated at compile time with a violating input is a compile-time rejection (Volume 09 catalog). Short-circuit forms evaluate per §2.4.3 in const contexts too.

### 2.4.6 IR lowering (design.md L189)

Exact lowering rules (node kinds are Volume 13; the mapping is pinned here):

| Operator class | IR 1 lowering |
| -------------- | ------------- |
| Panic-class integer op | `Check(cond)` then `Bin(op)`; `cond` per §2.4.4 (range, nonzero-divisor, shift-amount). |
| N/A integer/bool/bit op | `Bin(op)` directly. |
| Float op | `Bin(op)` with the float flag; no check. |
| `as` | `Cast{to}` (semantics §2.4.7). |
| `&&` / `\|\|` | Branch pair: evaluate left; branch to right-arm or constant arm; exact shape `Branch(lhs, rhs_block, const_block)`. |
| Assignment OP-01 | `Store(place, value)`; place address computed once. |
| Compound OP-02..OP-11 | `Load(place)`, `Bin(op)`, `Store(place, ...)`; desugar `l op= r` is exactly `l = l op r` with `l`'s place evaluated once. |
| Ranges OP-12..OP-16 | Struct construction of the Volume 19 §26.6 range types. |
| OP-40 | `AddrOf(place)` with the field offset from Volume 05 layout. |
| OP-42 | bounds `Check` then `AddrOf(base + size · index)`. |
| OP-41, OP-43 | `Call` with dispatch kind Direct or Dynamic{vtable slot} (Volume 04/05 compute the slot). |
| OP-44 | `Try` node (Volume 07). |

### 2.4.7 Cast semantics (`as`, exact)

Allowed casts and their exact results:

| From → To | Result |
| --------- | ------ |
| integer → integer | two's-complement reinterpretation: same width identity; narrowing truncates low bits; widening sign-extends signed and zero-extends unsigned |
| integer → float | round-to-nearest, ties-to-even; overflow to ±inf is a defined value |
| float → integer | truncate toward zero, then saturate to the target range; never traps |
| f64 → f32, f32 → f64 | IEEE convert, round-to-nearest ties-to-even |
| `bool` → integer | 0 or 1 |
| `u32` → `char` | the scalar with that code point; if the value is a surrogate or > U+10FFFF, a defined trap with message `invalid char cast` (exit code 134) |
| `char` → `u32` | the scalar value |

Any cast outside this table is a type error (Volume 04).

**Integration points.** Depends on: Volume 04 (dispatch, coercion), Volume 05 (layout offsets), Volume 07 (panic/try), Volume 09 (const eval), Volume 13 (IR 1). Depended on by: `planck-operator-math`, Volumes 13-15 (lowering), Volume 22 (per-pass proofs of §2.4.6 rules).

## 2.5 Grammar

The grammar is EBNF; terminals are quoted; `IDENT`, `LIFETIME`, and literal names refer to §2.2 token classes. The grammar is the complete syntactic surface of edition P1.

### 2.5.1 Compilation units (design.md L195)

```
CompilationUnit  = InnerAttr* { Item } ;
```

A compilation unit is one file (crate root or module file, Volume 03 §4.1). There is no package syntax inside a unit.

### 2.5.2 Modules (design.md L196)

```
ModItem          = "mod" IDENT ( "{" InnerAttr* { Item } "}" | ";" ) ;
```

`mod name;` loads the module file per Volume 03 §4.1; `mod name { }` is inline. `mod` admits no generic parameters (§2.5.4).

### 2.5.3 Declarations (design.md L197)

```
Item             = { OuterAttr | DocOuter } Vis? ItemBody ;
Vis              = "pub" [ "(" ( "crate" | "super" | "in" SimplePath ) ")" ] ;
ItemBody         = ModItem | UseItem | FnItem | StructItem | EnumItem | UnionItem
                 | TraitItem | ImplItem | ConstItem | StaticItem | TypeAliasItem
                 | ExternBlockItem | MacroDefItem ;
UseItem          = "use" UseTree ";" ;
UseTree          = SimplePath [ "as" ( IDENT | "_" ) ]
                 | SimplePath "::" "*"
                 | SimplePath "::" "{" UseTreeList "}"
                 | "{" UseTreeList "}" ;
UseTreeList      = UseTree { "," UseTree } [ "," ] ;
ConstItem        = "const" IDENT ":" Type "=" Expr ";" ;
StaticItem       = "static" IDENT ":" Type "=" Expr ";" ;
TypeAliasItem    = "type" IDENT GenericParams? WhereClause? "=" Type ";" ;
ExternBlockItem  = "extern" [ STRING ] "{" { ExternItem } "}" ;
ExternItem       = { OuterAttr } ( FnItemNoBody | "static" IDENT ":" Type ";" ) ;
```

`static` has no `mut` form in edition P1 (Volume 03 §3.3.3); the `STRING` of `extern` must be `"C"` (Volume 03 §3.3.16); `FnItemNoBody` is `FnItem` with `;` instead of a block.

### 2.5.4 Generic parameters (design.md L198)

```
GenericParams    = "<" GenericParamList ">" ;
GenericParamList = GenericParam { "," GenericParam } [ "," ] ;
GenericParam     = LifetimeParam | TypeParam ;
LifetimeParam    = LIFETIME [ ":" LifetimeBounds ] ;
LifetimeBounds   = LIFETIME { "+" LIFETIME } ;
TypeParam        = IDENT [ ":" TypeBounds ] [ "=" Type ] ;
WhereClause      = "where" WherePredicate { "," WherePredicate } [ "," ] ;
WherePredicate   = Type ":" TypeBounds | LIFETIME ":" LifetimeBounds ;
```

Edition P1 has no const generic parameters and no default type bounds; the only generic-parameter kinds are lifetime and type. `mod` accepts no `GenericParams`.

### 2.5.5 Type expressions (design.md L199)

```
Type             = ParenType | TupleType | ArrayType | SliceType | RefType | PtrType
                 | FnPtrType | PathType | PrimType | NeverType | UnitType ;
ParenType        = "(" Type ")" ;
TupleType        = "(" [ Type "," Type { "," Type } [ "," ] ] ")" ;
ArrayType        = "[" Type ";" Expr "]" ;
SliceType        = "[" Type "]" ;
RefType          = "&" LIFETIME? [ "mut" ] Type ;
PtrType          = "*" ( "const" | "mut" ) Type ;
FnPtrType        = "fn" "(" [ Type { "," Type } [ "," ] ] ")" [ "->" Type ] ;
PathType         = [ "::" ] TypePath ;
TypePath         = PathSegment { "::" PathSegment } [ GenericArgs ] ;
GenericArgs      = "<" GenericArgList ">" ;
GenericArgList   = GenericArg { "," GenericArg } [ "," ] ;
GenericArg       = Type | LIFETIME | Binding ;
Binding          = IDENT "=" Type ;
PrimType         = "bool" | "char" | "str" | "i8" | "i16" | "i32" | "i64" | "i128"
                 | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "isize" | "f32" | "f64" ;
NeverType        = "!" ;
UnitType         = "(" ")" ;
```

The array length `Expr` must be a const context (Volume 09). `FnPtrType` carries no generic parameters in P1.

### 2.5.6 Function declarations (design.md L200)

```
FnItem           = "fn" IDENT GenericParams? FnParams [ "->" Type ] WhereClause? ( Block | ";" ) ;
FnParams         = "(" [ ParamList ] ")" ;
ParamList        = Param { "," Param } [ "," ] ;
Param            = Receiver | TypedParam ;
Receiver         = [ "mut" ] "self"
                 | "&" LIFETIME? [ "mut" ] "self"
                 | "self" ":" Type ;
TypedParam       = [ "mut" ] ( IDENT | "_" ) ":" Type ;
```

A body-less `fn` is legal only inside a trait (required method) or an `extern` block; elsewhere it is `E-DEC-03` (Volume 03 §3.3.25). Effect rows are inferred, not annotated, in edition P1 (Volume 07); annotation syntax is reserved.

### 2.5.7 Structs (design.md L201)

```
StructItem       = "struct" IDENT GenericParams? WhereClause?
                 ( StructFields | "(" TupleFields ")" ";" | ";" ) ;
StructFields     = "{" [ StructField { "," StructField } [ "," ] ] "}" ;
StructField      = Vis? IDENT ":" Type ;
TupleFields      = TupleField { "," TupleField } [ "," ] ;
TupleField       = Vis? Type ;
```

Unit structs are the `";"` form; tuple structs are the parenthesized form. Layout semantics are Volume 05 (pinned `#[repr(P)]` default).

### 2.5.8 Enums (design.md L202)

```
EnumItem         = "enum" IDENT GenericParams? WhereClause?
                 "{" [ EnumVariant { "," EnumVariant } [ "," ] ] "}" ;
EnumVariant      = IDENT [ ( "(" TupleFields ")" | StructFields ) ] [ "=" Expr ] ;
```

The discriminant `Expr` must be a const integer expression; default discriminants are Volume 03 §3.3.9.

### 2.5.9 Unions (design.md L203)

```
UnionItem        = "union" IDENT GenericParams? WhereClause? StructFields ;
```

Union field restrictions (Copy-only fields) are Volume 03 §3.3.10.

### 2.5.10 Traits (design.md L204)

```
TraitItem        = "trait" IDENT GenericParams? [ ":" TypeBounds ] WhereClause?
                 "{" { TraitAssocItem } "}" ;
TraitAssocItem   = { OuterAttr } ( FnItem | ConstAssocItem | TypeAssocItem ) ;
ConstAssocItem   = "const" IDENT ":" Type [ "=" Expr ] ";" ;
TypeAssocItem    = "type" IDENT [ ":" TypeBounds ] [ "=" Type ] ";" ;
```

A trait `fn` with a body is a default method; without a body it is required. Object-safety rules are Volume 03 §3.3.11.

### 2.5.11 Implementations (design.md L205)

```
ImplItem         = "impl" GenericParams? [ TraitRef "for" ] Type WhereClause?
                 "{" { ImplAssocItem } "}" ;
TraitRef         = [ "::" ] TypePath ;
ImplAssocItem    = { OuterAttr } Vis? ( FnItem | ConstAssocItem | TypeAssocItem ) ;
```

Negative impls (`impl !T for U`) are not grammar in edition P1 (banned; coherence uses no negative reasoning, Volume 03 §3.3.20). Coherence rules are Volume 03 §3.3.12 and §3.3.20.

### 2.5.12 Attributes (design.md L206)

The attribute grammar is §2.2.17; the attribute name inventory and per-attribute semantics are Volume 10 §15.1. Feature template for the attribute feature:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | §2.2.17 productions. |
| 2. Static semantics | Name must be in the Volume 10 inventory; placement per §2.2.17; multiplicity per Volume 10. |
| 3. Type semantics | Not applicable: attributes do not have types. |
| 4. Value semantics | Not applicable: attributes produce no values. |
| 5. Evaluation semantics | Evaluated at the phase named per attribute in Volume 10 (parse, typecheck, codegen). |
| 6. Ownership | Not applicable. |
| 7. Effects | Not applicable. |
| 8. Control flow | Not applicable. |
| 9. Compile-time behavior | Attributes that affect monomorphization/const eval are listed in Volume 10 §15.3. |
| 10. Memory behavior | Only codegen-affecting attributes change layout/storage (Volume 10 §15.1). |
| 11. ABI behavior | `#[no_mangle]`/`#[repr]`-family attributes are the ABI surface (Volume 10). |
| 12. IR 1 mapping | Attributes are carried as IR 1 metadata on nodes they annotate. |
| 13. IR 2 mapping | Hint attributes (`#[inline]`, `#[cold]`) are read by Volume 14 profitability models. |
| 14. Machine IR mapping | Placement/section attributes map to Volume 15 directives. |
| 15. Diagnostics | Unknown/misplaced attribute: `E-PAR-03`; duplicates: `W-ATTR-01`. |
| 16. Validation | EXH over attribute-form shapes (OB-02-14); per-attribute tests in Volume 10. |

### 2.5.13 Statements (design.md L207)

```
Statement        = ";" | Item | LetStatement | ExpressionStatement ;
LetStatement     = "let" [ "mut" ] Pattern [ ":" Type ] [ "=" Expr ] ";" ;
ExpressionStatement = Expr ";" | ExprWithBlock ;
ExprWithBlock    = BlockExpr | IfExpr | MatchExpr | WhileExpr | LoopExpr | ForExpr
                 | UnsafeBlockExpr | AsyncBlockExpr ;
```

A lone `;` is an empty statement and is always valid. Semicolon rules are §2.6.6.

### 2.5.14 Expressions (design.md L208)

Precedence-climbing productions; levels implement §2.4.1 exactly.

```
Expression       = ExprAssign ;
ExprAssign       = ExprRange [ AssignOp ExprAssign ] ;
AssignOp         = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" ;
ExprRange        = [ ExprOr ] ( ".." [ ExprOr ] | "..=" ExprOr ) ;
ExprOr           = ExprAnd { "||" ExprAnd } ;
ExprAnd          = ExprBOr { "&&" ExprBAnd } ;
ExprBOr          = ExprBXor { "|" ExprBXor } ;
ExprBXor         = ExprBAnd { "^" ExprBAnd } ;
ExprBAnd         = ExprEq { "&" ExprEq } ;
ExprEq           = ExprRel [ ( "==" | "!=" ) ExprRel ] ;
ExprRel          = ExprShift [ ( "<" | ">" | "<=" | ">=" ) ExprShift ] ;
ExprShift        = ExprAdd { ( "<<" | ">>" ) ExprAdd } ;
ExprAdd          = ExprMul { ( "+" | "-" ) ExprMul } ;
ExprMul          = ExprCast { ( "*" | "/" | "%" ) ExprCast } ;
ExprCast         = ExprUnary [ "as" CastTarget ] ;
CastTarget       = PrimType | PathType ;
ExprUnary        = UnaryPrefix ExprUnary | ExprPostfix ;
UnaryPrefix      = "-" | "!" | "*" | "&" [ "mut" ] ;
ExprPostfix      = ExprAtom { Postfix } ;
Postfix          = "." IDENT | "." DecLiteralInt | "." IDENT "(" [ CallArgs ] ")"
                 | "(" [ CallArgs ] ")" | "[" Expr "]" | "?" ;
CallArgs         = Expr { "," Expr } [ "," ] ;
ExprAtom         = Literal | PathExpr | ParenOrTupleExpr | ArrayExpr | StructExpr
                 | BlockExpr | IfExpr | MatchExpr | WhileExpr | LoopExpr | ForExpr
                 | ReturnExpr | BreakExpr | ContinueExpr | ClosureExpr | AsyncBlockExpr
                 | UnsafeBlockExpr | AsmExpr | MacroInvocation ;
```

`ReturnExpr = "return" [ Expr ]`; `BreakExpr = "break" [ LIFETIME ] [ Expr ]`; `ContinueExpr = "continue" [ LIFETIME ]`; `IfExpr`, `MatchExpr`, `WhileExpr`, `LoopExpr`, `ForExpr` bodies and conditions are the control-flow productions of Volume 07 §10.1 (this volume fixes their syntactic shells: `if Expr BlockExpr [ "else" ( IfExpr | BlockExpr ) ]`, `while Expr BlockExpr`, `loop BlockExpr`, `for Pattern "in" Expr BlockExpr`, `match Expr "{" { MatchArm } "}"`, `MatchArm = [ OuterAttr ] Pattern [ "if" Expr ] "=>" ( BlockExpr | Expr ) ","?` with the final arm's comma optional before `}`). Ranges with no left operand (`..b`, `..`) are valid only where the grammar's context admits them (patterns and range expressions); the parser accepts them in `ExprRange` and restricts by context.

### 2.5.15 Patterns (design.md L209)

```
Pattern          = OrPattern ;
OrPattern        = AltPattern { "|" AltPattern } ;
AltPattern       = BindingPattern | "_" | LiteralPattern | RangePattern | RefPattern
                 | TuplePattern | TupleStructPattern | StructPattern | PathPattern
                 | "(" Pattern ")" | SlicePattern ;
BindingPattern   = [ "mut" ] IDENT [ "@" Pattern ] ;
LiteralPattern   = [ "-" ] IntegerLiteral | CharLiteral | BoolLiteral ;
RangePattern     = ( [ "-" ] IntegerLiteral | CharLiteral ) "..=" ( [ "-" ] IntegerLiteral | CharLiteral ) ;
RefPattern       = "&" [ "mut" ] Pattern ;
TuplePattern     = "(" [ Pattern { "," Pattern } [ "," ] ] ")" ;
TupleStructPattern = SimplePath "(" [ Pattern { "," Pattern } [ "," ] ] ")" ;
StructPattern    = SimplePath "{" [ FieldPattern { "," FieldPattern } [ "," ] ] [ ".." ] "}" ;
FieldPattern     = IDENT [ ":" Pattern ] ;
PathPattern      = SimplePath ;
SlicePattern     = "[" [ Pattern { "," Pattern } [ "," ] ] [ ".." ] "]" ;
```

String patterns are not grammar in P1 (banned). Range patterns are inclusive-only (`..=`).

### 2.5.16 Closures (design.md L210)

```
ClosureExpr      = [ "move" ] [ "async" ] [ "unsafe" ] ( "||" | "|" [ ClosureParams ] "|" )
                 [ "->" Type ] ( Block | Expr ) ;
ClosureParams    = ClosureParam { "," ClosureParam } [ "," ] ;
ClosureParam     = [ "mut" ] Pattern [ ":" Type ] ;
```

At the start of an expression, `|` or `||` always begins a closure (§2.6.3 rule (i)). Capture semantics and `move` are Volume 06 §9.6.

### 2.5.17 Async blocks (design.md L211)

```
AsyncBlockExpr   = "async" [ "move" ] BlockExpr ;
```

`async fn` uses the `FnItem` shell with the `async` prefix: `"async" FnItem`. An async block's value is a task handle under the pinned structured-concurrency model (SPEC-INDEX §3; Volume 08). Feature template:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | `async` Block, optional `move`; `async fn` per §2.5.6. |
| 2. Static semantics | Block must typecheck as a value of its return type; no detached spawn form exists (Volume 08). |
| 3. Type semantics | Value type is the task handle type of Volume 08 §12.4. |
| 4. Value semantics | A suspended computation producing the block's value on completion. |
| 5. Evaluation semantics | Schedules onto the current task scope's executor (Volume 08). |
| 6. Ownership | Captures per closure rules; `move` forces by-value capture. |
| 7. Effects | Row contains `async`; the atom is pinned (SPEC-INDEX §5.3). |
| 8. Control flow | Await points are cancellation checkpoints (pinned model). |
| 9. Compile-time behavior | State-machine lowering; no separate codegen path (Volume 08 §12.5). |
| 10. Memory behavior | Task frame allocation under §1.2.6 A3 (Volume 01). |
| 11. ABI behavior | Not applicable: async blocks do not cross FFI (compile-time rejection if attempted). |
| 12. IR 1 mapping | Coroutine node with suspend edges (Volume 13). |
| 13. IR 2 mapping | State-machine specialization (Volume 14). |
| 14. Machine IR mapping | Frame layout and resume protocol per Volume 15. |
| 15. Diagnostics | Misuse diagnostics in Volume 08 catalog. |
| 16. Validation | Volume 08 test matrix; structured-concurrency obligation OB-01-15 (Volume 01). |

### 2.5.18 Unsafe blocks (design.md L212)

```
UnsafeBlockExpr  = "unsafe" BlockExpr ;
```

An `unsafe` block admits the operations of Volume 01 §1.3.10. An `unsafe fn` uses the `FnItem` shell with the `unsafe` prefix; its inferred effect row contains the `unsafe` atom (pinned, SPEC-INDEX §5.3). `unsafe` trait definitions are not grammar in P1 (banned; Volume 04).

### 2.5.19 Inline assembly (design.md L213)

```
AsmExpr          = "asm" "(" STRING "," { AsmOperand "," } [ AsmClobbers "," ] [ AsmOptions "," ] ")" ;
AsmOperand       = AsmDir "(" RegSpec ")" Expr ;
AsmDir           = "in" | "out" | "inout" | "inlateout" ;
RegSpec          = IDENT | STRING ;
AsmClobbers      = "clobber" "(" RegSpec { "," RegSpec } [ "," ] ")" ;
AsmOptions       = "options" "(" AsmOption { "," AsmOption } [ "," ] ")" ;
AsmOption        = "pure" | "nomem" | "readonly" | "preserves_flags" | "noreturn" | "nostack" ;
```

`asm` requires an enclosing `unsafe` block. The template STRING and the valid `RegSpec` set per target are Volume 15 §22.4; UB-5 applies to declared clobber/condition violations (Volume 01 §1.3.9). Feature template:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | §2.5.19 productions. |
| 2. Static semantics | Must be in `unsafe`; directions and options must satisfy Volume 15 §22.4 constraints (e.g. `pure` requires exactly one `out`). |
| 3. Type semantics | Operand expressions must be integer/pointer lvalues of register width; widths per Volume 15. |
| 4. Value semantics | Output operands receive register contents at block exit. |
| 5. Evaluation semantics | Inputs evaluated left to right, block executes, outputs read back in order. |
| 6. Ownership | Inputs are reads; `inout` is read-then-write; no moves occur. |
| 7. Effects | Row `{unsafe}`; plus `nomem`/`preserves_flags` refine the volume 07 memory/flag edges. |
| 8. Control flow | `noreturn` makes the block diverge; otherwise straight-line. |
| 9. Compile-time behavior | Not const-evaluable; rejected in const contexts. |
| 10. Memory behavior | Default: reads/writes memory unless `nomem`/`readonly` declared; UB-5 on violation. |
| 11. ABI behavior | Clobbers must be declared exactly; missing clobber is UB-5. |
| 12. IR 1 mapping | Opaque asm node with operand/clobber metadata (Volume 13 §20.7). |
| 13. IR 2 mapping | Not optimizable across; treated as a barrier. |
| 14. Machine IR mapping | Instruction materialization per target (Volume 15). |
| 15. Diagnostics | Volume 21 catalog family `E-ASM-*` (Volume 10 §17.2). |
| 16. Validation | EXH over grammar shapes (OB-02-15); per-target instruction tests (Volume 16). |

### 2.5.20 Macros (design.md L214)

Macros are supported in edition P1 as declarative macros only (settlement; §1.1.6 N5 bans the other families in P1). Syntax:

```
MacroInvocation  = SimplePath "!" DelimTokenTrees ;
MacroDefItem     = "macro" IDENT "(" MacroRuleList ")" ;
MacroRuleList    = MacroRule { "," MacroRule } [ "," ] ;
MacroRule        = Matcher "=>" Transcriber ;
Matcher          = DelimTokenTrees with metavariable productions below ;
Metavar          = "$" IDENT ":" FragmentSpec ;
Repetition       = "$" "(" TokenTree* ")" Sep? ( "*" | "+" ) ;
Sep              = one TokenNotDelim ;
FragmentSpec     = "expr" | "ty" | "pat" | "stmt" | "block" | "item" | "path"
                 | "tt" | "lifetime" | "literal" | "ident" ;
Transcriber      = DelimTokenTrees with "$" IDENT and "$(" ... ")" Sep? ( "*" | "+" ) ;
```

Expansion timing, matching algorithm, and recursion limits are Volume 09 §14.5; expansion depth limit `MAX_MACRO_DEPTH = 64` is pinned here. Edition P1 has no procedural, attribute, or derive macros; the `macro` production above is the only definition form. Feature template:

| Row | Definition |
| --- | ---------- |
| 1. Syntax | §2.5.20 productions. |
| 2. Static semantics | Invocation must resolve in the macro namespace before expansion (Volume 03 §3.1.7); fragment kinds must match use sites. |
| 3. Type semantics | Expansion outputs are typed after expansion; macros add no types. |
| 4. Value semantics | The expansion's semantics; nothing else. |
| 5. Evaluation semantics | Expansion at the Volume 09 phase; depth ≤ 64. |
| 6. Ownership | As expanded code. |
| 7. Effects | As expanded code. |
| 8. Control flow | As expanded code. |
| 9. Compile-time behavior | Expansion before name resolution of items inside it. |
| 10. Memory behavior | As expanded code. |
| 11. ABI behavior | Not applicable. |
| 12. IR 1 mapping | Expansion result lowers as handwritten code. |
| 13. IR 2 mapping | Same. |
| 14. Machine IR mapping | Same. |
| 15. Diagnostics | Match failures report the matcher arm and site (`E-MAC-*`, Volume 09). |
| 16. Validation | EXH over fragment specifier names (OB-02-14); hygiene property OB-03-11 (Volume 03). |

### 2.5.21 Error recovery (design.md L215)

Error recovery is defined normatively in §2.6.8-§2.6.10; the grammar admits recovery by construction: every production level has the sync set named there, and recovery never produces accepted syntax, only diagnostics and a `recovered` parse.

**Integration points.** Depends on: §2.2 tokens, Volumes 03 (modules, declarations), 04-10 (semantic phases), 09 (macros). Depended on by: Volume 11 (frontend pipeline), Volume 21 (IDE tooling consumes the same grammar).

## 2.6 Parsing behavior

### 2.6.1 Ambiguity resolution (design.md L219)

The grammar is deterministic: recursive descent with Pratt precedence climbing for expressions. Every potential ambiguity is resolved by exactly one of these tie-breaks: (a) token classes are disjoint by prefix (§2.2.21 priority order); (b) item vs statement: at statement level, `pub` or an item keyword starts an item only when the position admits an item (top level or after attributes); (c) expression-position `<` after a complete path is always a comparison, never generic arguments; (d) at expression start, `|` or `||` always begins a closure; (e) `-`, `&`, `*` are prefix operators exactly in prefix position; (f) `'` follows §2.2.21 rule 5; (g) patterns have no bitwise operators: `|` is always or-pattern there; (h) `Path {` in pattern position is a struct pattern; in expression position a struct literal, subject to (§2.6.7); (i) `(A)` is not a tuple; `(A,)` is; (j) array type vs array expression is decided by grammar position. No other ambiguity exists; a grammar conflict not resolved by (a)-(j) is a defect in this volume.

### 2.6.2 Generic argument parsing (design.md L220)

In type position, after a complete `TypePath`, `<` begins `GenericArgs`. In expression position, generic arguments require the turbofish `::<` (§2.6.1 (c)); bare `<` is a comparison. In `GenericArgs` context, the token `>>` is split into two `>` tokens and `>>>` into three; `>=` is never split. The split is a parser-level rule, deterministic and context-scoped.

### 2.6.3 Type versus expression disambiguation (design.md L221)

Grammar positions are statically typed: a production names whether it parses a Type or an Expression, so contexts never overlap. The only shared-prefix forms are resolved by §2.6.1 (c), (h), (i), (j) and §2.6.2. A `CastTarget` is restricted to `PrimType | PathType` so `as` cannot capture a tuple or array.

### 2.6.4 Operator precedence parsing (design.md L222)

Algorithm (exact): `parse_expr(min_bp)`; loop: read token t; if t is not an operator of §2.4.1 or `bp(t) < min_bp`, return the current node; consume t; for prefix operators use their fixed bp; otherwise compute `(l_bp, r_bp)` = `(p, p+1)` for Left associativity, `(p, p)` for Right, `(p+1, p)` for None at the none-associative guard; parse rhs = `parse_expr(l_bp)`; combine. Postfix forms loop at bp 14. Input domain: any token stream. Output: an expression tree or the first diagnostic. Termination: each iteration consumes ≥ 1 token. Complexity: O(n) time over n tokens, O(d) stack where d ≤ `MAX_PARSE_DEPTH = 128`; deeper nesting is `E-PAR-09`.

### 2.6.5 Trailing commas (design.md L223)

Trailing commas are allowed in exactly: tuple expressions, tuple types, call arguments, array expressions, struct literal field lists, generic parameter lists, generic argument lists, function parameter lists, closure parameter lists, field declaration lists, enum variant lists, pattern tuple/struct/slice lists, use-tree lists, `where` predicates. A trailing comma anywhere else is `E-PAR-04`. The match-arm rule (final comma optional before `}`) is part of the arm production, not this rule.

### 2.6.6 Semicolon rules (design.md L224)

A `;` is required after: `let` statements; expression statements whose expression is not an `ExprWithBlock`; `use`, `const`, `static`, `type alias`, file-backed `mod`, extern items, macro invocations used as items without braces; tuple struct declarations. No `;` is required after: block-bodied items (`fn`, `struct {}`, `enum {}`, `union {}`, `trait {}`, `impl {}`, `mod {}`, `macro {}`), `ExprWithBlock` statements, match arms. A lone `;` is an empty statement (§2.5.13). A missing `;` is `E-PAR-05` at the next token.

### 2.6.7 Block expressions (design.md L225)

A block `{ Statements? Expr? }` evaluates to its final expression's value; without one it evaluates to `()`; a block ending in a diverging expression has type `!`. A statement may not begin with a struct literal `Path { ... }` (`E-PAR-06`); such literals in statement position must be parenthesized or assigned. This is the only statement-start restriction.

### 2.6.8 Parser recovery (design.md L226)

On a parse error at token t: (1) emit exactly one diagnostic (`E-PAR-01` unless a specific code applies); (2) skip tokens until one in the sync set of the innermost active level: `SYNC-ITEM = {pub, mod, use, fn, struct, enum, union, trait, impl, const, static, type, macro, extern, }, EOF}`; `SYNC-STMT = SYNC-ITEM ∪ {let, if, while, loop, for, match, return, break, continue, ;}`; `SYNC-EXPR = {, ;, ), ], }}`; (3) if the skip first meets the closing delimiter of the innermost group, resume inside that group; recovery never crosses the closer of an outer group; (4) at EOF or unbalanced delimiters, report `E-PAR-02` and stop the file. Termination: each recovery step consumes ≥ 1 token. Complexity: O(n) total skip across a file. The maximal error count per file is `MAX_PARSE_ERRORS = 32`; further diagnostics are suppressed. A file with any parse error is marked `recovered` and downstream phases do not run on it.

### 2.6.9 Malformed input handling (design.md L227)

Every malformed input produces at least one diagnostic and at most `MAX_PARSE_ERRORS`. Exactly one primary diagnostic is emitted per **minimal erroneous region**: the longest token span starting at the error token containing no sync-set token. Diagnostics are deterministic: identical input yields identical diagnostic sequences (G14).

### 2.6.10 Error synchronization (design.md L228)

Synchronization is the sync sets of §2.6.8 plus the delimiter discipline stated there. The delimiter stack is maintained by the lexer invariant (§2.2.20), so recovery re-synchronizes to the nearest enclosing group boundary exactly.

### 2.6.11 Incremental parsing (design.md L229)

The parser is a pure function of the token stream (no global state; invariant I-2 below). The incremental unit is the top-level item (brace-matched, including its attributes and doc comments). On edit: items whose byte ranges are untouched are reused; the item containing the edit is reparsed; if the edit changes delimiter balance, reparse runs from the outermost unclosed item to end of file. Cache key: `(file id, item byte range, FNV-1a 64-bit hash of the range bytes)`. Result invariant: incremental output equals whole-file output for the same content (OB-02-13).

### 2.6.12 Parser invariants

1. I-1: the parser consumes each token at most once; output position is monotonically non-decreasing.
2. I-2: parser output depends only on the token stream, not on file order, environment, or prior files.
3. I-3: on any input, the parser terminates within O(n) steps for n tokens.
4. I-4: accepted ASTs satisfy the grammar productions exactly; recovered parses are marked and never enter later phases.

**Integration points.** Depends on: §2.2 tokenizer, §2.4 table. Depended on by: Volume 11 (pipeline), Volume 21 (LSP incremental reparse, formatter), Volume 22 (differential parser tests).

## 2.7 Known limitations

Identifiers are ASCII in edition P1 (§2.2.2); widening is an edition change. There are no raw identifiers, no const generics, no negative impls, no string patterns, no expression attributes, and no procedural macros; each is a deliberate P1 boundary, revisable only by edition. Float-parse equivalence is Z3-checked for decimal strings with ≤ 6 significant digits and exponent in −20..20; outside the bound the runtime bridge is that compilation and any runtime use call the same `planck-token-math` function, so the equivalence holds by implementation identity. Register names for `asm` are target tables (Volume 15), not defined here.

## 2.8 Verification matrix

| Obligation ID | Property (exact statement) | Method | Class | Artifact | G-mapping |
| ------------- | -------------------------- | ------ | ----- | -------- | --------- |
| OB-02-1 | Each charclass predicate of §2.2.0 returns the exact set for all 256 byte values. | EXH | FV-0 | math/planck-charclass tests | G1 |
| OB-02-2 | The tokenizer is a total deterministic function: for all strings of length ≤ 4 over the alphabet {a, Z, 0, 9, _, ', \, ", /, *, ., x, o, #, u, (, ), space} (83521 cases), the output is a unique token sequence or exactly one `E-LEX-*`. | EXH | FV-0 | math/planck-token-math tests | G14 |
| OB-02-3 | The keyword classifier yields keyword tokens for exactly the 41 + 13 names of §2.2.4-§2.2.5 and identifiers otherwise. | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-02-4 | The radix fold of §2.3.1 equals the reference arbitrary-precision fold for every digit string of length ≤ 4 over the union radix alphabet. | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-02-5 | Literal acceptance is exactly min(T) ≤ v ≤ max(T) at the 4 boundary values per integer type (64 cases). | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-02-6 | Float rounding equals the correctly-rounded value for decimal strings with ≤ 6 significant digits, exponent −20..20, for f32 and f64. | Z3 + PROP | FV-1 | tools/fv/floatparse.smt2 | G1 |
| OB-02-7 | Escape decoding is exact for all 9 named forms, all 256 `\xHH` values per context rule, and the 8 scalar boundaries of `\u{...}`. | EXH | FV-0 | math/planck-token-math tests | G1 |
| OB-02-8 | Operator lookup returns the exact §2.4.1 row for each of the 44 operator shapes and no row for 6 non-operator delimiters. | EXH | FV-0 | math/planck-operator-math tests | G1 |
| OB-02-9 | Pratt parsing (§2.6.4) terminates and produces the reference-precedence AST for every token sequence of length ≤ 7 over {a, +, *, -, !, (, ), ?}. | EXH + PROP | FV-0 | tools/fv/pratt.smt2 | G1 |
| OB-02-10 | Trailing-comma acceptance matches §2.6.5 exactly across the 14 listed positions and 4 non-listed positions. | EXH | FV-0 | parser tests | G1 |
| OB-02-11 | Recovery terminates for every token sequence of length ≤ 6 over {a, (, ), {, }, ;, ", .} and emits at most MAX_PARSE_ERRORS diagnostics. | EXH | FV-0 | parser recovery tests | G11, G22 |
| OB-02-12 | Every invalid literal in the mutation domain (all 1- and 2-byte mutations of 20 valid literal forms) yields exactly one diagnostic from §2.3.11 with the specified span. | EXH | FV-0 | literal diagnostic tests | G17, G22 |
| OB-02-13 | Incremental reparse output equals whole-file reparse output for generated edits (PROP) and for the bounded corpus (EXH). | PROP + EXH | FV-1 | incremental parser tests | G1 |
| OB-02-14 | The fragment-specifier set is exactly the 11 names of §2.5.20 and the attribute-input forms are exactly §2.2.17's three. | EXH | FV-0 | parser tests | G1 |
| OB-02-15 | The asm grammar accepts exactly the §2.5.19 shapes: 4 directions × 3 register-spec forms × 3 operand arities, with the 6 option names. | EXH | FV-0 | parser tests | G1 |
| OB-02-16 | UTF-8 decode accepts exactly valid sequences: all 1-byte and all 2-byte sequences (65536) plus the 128 boundary 3-byte sequences. | EXH | FV-0 | math/planck-charclass tests | G1, G14 |

## 2.9 Coverage ledger

bullets in scope: 85, defined: 85. Prose requirements in scope: 2, defined: 2.

| design.md bullet | Defined in |
| ---------------- | ---------- |
| L122 (prose) lexical draft needs a complete normative definition | §2.0 |
| L126 Source encoding | §2.1.1 |
| L127 Unicode version | §2.1.2 |
| L128 Byte-order mark handling | §2.1.3 |
| L129 Line endings | §2.1.4 |
| L130 Invalid encoding behavior | §2.1.5 |
| L131 Source locations | §2.1.6 |
| L132 Column numbering | §2.1.7 |
| L133 Tabs and indentation | §2.1.8 |
| L134 File inclusion rules | §2.1.9 |
| L138 Identifier grammar | §2.2.1 |
| L139 Unicode identifier rules | §2.2.2 |
| L140 Identifier normalization | §2.2.3 |
| L141 Reserved keywords | §2.2.4 |
| L142 Future-reserved keywords | §2.2.5 |
| L143 Context-sensitive restrictions, if any | §2.2.6 |
| L144 Integer literals | §2.2.7 |
| L145 Floating-point literals | §2.2.8 |
| L146 Literal suffixes | §2.2.9 |
| L147 Character literals | §2.2.10 |
| L148 String literals | §2.2.11 |
| L149 Byte strings | §2.2.12 |
| L150 Raw strings | §2.2.13 |
| L151 Escape sequences | §2.2.14 |
| L152 Comments | §2.2.15 |
| L153 Documentation comments | §2.2.16 |
| L154 Attributes | §2.2.17, §2.5.12 |
| L155 Punctuation | §2.2.18 |
| L156 Operators | §2.4.1 |
| L157 Delimiters | §2.2.20 |
| L158 Tokenization precedence | §2.2.21 |
| L162 Integer literal interpretation | §2.3.1 |
| L163 Arbitrary-precision literal handling during type checking | §2.3.2 |
| L164 Integer range checking | §2.3.3 |
| L165 Floating-point rounding | §2.3.4 |
| L166 NaN and infinity representation | §2.3.5 |
| L167 Character validity | §2.3.6 |
| L168 Unicode scalar values | §2.3.7 |
| L169 UTF-8 validation | §2.3.8 |
| L170 Byte literal ranges | §2.3.9 |
| L171 Compile-time literal evaluation | §2.3.10 |
| L172 Invalid literal diagnostics | §2.3.11 |
| L177 Syntax (per operator) | §2.4.1 |
| L178 Precedence | §2.4.1 |
| L179 Associativity | §2.4.1 |
| L180 Operand types | §2.4.2 |
| L181 Result type | §2.4.2 |
| L182 Overloadability | §2.4.1, §2.4.2 |
| L183 Evaluation order | §2.4.3 |
| L184 Overflow semantics | §2.4.4, §2.4.7 |
| L185 Panic behavior | §2.4.4 |
| L186 Effects | §2.4.5 |
| L187 Constant evaluation | §2.4.5 |
| L188 IR lowering | §2.4.6 |
| L195 Compilation units | §2.5.1 |
| L196 Modules | §2.5.2 |
| L197 Declarations | §2.5.3 |
| L198 Generic parameters | §2.5.4 |
| L199 Type expressions | §2.5.5 |
| L200 Function declarations | §2.5.6 |
| L201 Structs | §2.5.7 |
| L202 Enums | §2.5.8 |
| L203 Unions | §2.5.9 |
| L204 Traits | §2.5.10 |
| L205 Implementations | §2.5.11 |
| L206 Attributes (grammar) | §2.5.12 |
| L207 Statements | §2.5.13 |
| L208 Expressions | §2.5.14 |
| L209 Patterns | §2.5.15 |
| L210 Closures | §2.5.16 |
| L211 Async blocks | §2.5.17 |
| L212 Unsafe blocks | §2.5.18 |
| L213 Inline assembly | §2.5.19 |
| L214 Macros, if supported | §2.5.20 |
| L215 Error recovery | §2.6.8-§2.6.10 (home), §2.5.21 (cross-reference) |
| L219 Ambiguity resolution | §2.6.1 |
| L220 Generic argument parsing | §2.6.2 |
| L221 Type versus expression disambiguation | §2.6.3 |
| L222 Operator precedence parsing | §2.6.4 |
| L223 Trailing commas | §2.6.5 |
| L224 Semicolon rules | §2.6.6 |
| L225 Block expressions | §2.6.7 |
| L226 Parser recovery | §2.6.8 |
| L227 Malformed input handling | §2.6.9 |
| L228 Error synchronization | §2.6.10 |
| L229 Incremental parsing | §2.6.11 |
| L231 (prose) Deliverables: Lexical, Grammar, Operator, Parser Contract | §2.0, this file |
