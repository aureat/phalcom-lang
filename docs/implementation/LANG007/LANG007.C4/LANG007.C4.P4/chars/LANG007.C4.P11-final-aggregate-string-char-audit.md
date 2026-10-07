# LANG007.C4.P11 — Final Aggregate String/Char Implementation Audit

**Repository:** `aureat/phalcom-lang`  
**Audited Char branch:** `codex/lang007-c4-p4-r1-stdlib`  
**Baseline:** `615f5887`  
**Char implementation:** `800e179325a4fa0d97e3cd1efa636d09d6fab393` — `feat(char): add Unicode scalar values and literal patterns`  
**Lifecycle record:** `51c4d35a6820b334599a3872185fa64d8fab0651` — lifecycle/documentation only  
**Audit date:** 2026-10-07  
**Status recommendation:** **HOLD — `IN_PROGRESS / PARTIAL / UNVERIFIED` remains correct**

---

## 1. Executive verdict

This report consolidates two independent implementation audits and a final repository re-check of the disputed or newly surfaced areas.

The result is clear:

> **The Char ontology, immediate representation, canonical semantic identity, lexer strategy, static-value integration, native API design, and closed Unicode-scalar match model are architecturally sound. P11 should not be redesigned.**
>
> **P11 nevertheless cannot be accepted yet.** A serious legacy/raw-AST pattern-lowering defect remains in `if let`, `while let`, and `for`; ordinary `match` still has an unchecked compiler fallback that can bypass semantic exhaustiveness authority; declaration-level Char literal rejection is owned by the compiler rather than semantics; the required semantic/runtime regression suite was not added or run; and several pre-existing String contracts are materially incoherent and must be repaired in P12/P13.

The two audits strongly agree on the core Char implementation. The independent audit additionally identified several important String defects and the standalone `match` fallback. The first audit identified a more severe pattern-binding defect that the independent audit missed: **or-pattern test branching was repaired, but raw-AST binding declaration and commit still operate over every alternative rather than the successful semantic alternative.**

That issue is the highest-priority implementation correction.

### Final disposition

| Area | Aggregate verdict |
|---|---|
| Char semantic model | **APPROVE** |
| Unicode scalar representation | **APPROVE** |
| Lexer/parser implementation | **APPROVE implementation; reject verification coverage** |
| Static Char identity/materialization | **APPROVE** |
| Runtime Char class/native API | **APPROVE with focused tests required** |
| INC-6 closed scalar-domain model | **APPROVE** |
| Canonical `ExecutablePattern::Char` matching | **APPROVE** |
| `if let` / `while let` / `for` raw-AST or-pattern lowering | **BLOCKING DEFECT** |
| Ordinary `match` compiler fallback | **REQUEST ARCHITECTURAL HARDENING** |
| Declaration Char-literal rejection | **WRONG AUTHORITY LAYER** |
| Char exhaustive-set operational contract | **NEEDS BOUNDED-CHECKER CLARIFICATION** |
| String UTF-8 backing invariant | **GOOD FOUNDATION** |
| `rawByteAt` type contract | **BROKEN** |
| String Int index handling | **BROKEN / inconsistent with arbitrary-precision `Int`** |
| `leadByteLen` boundary model | **BROKEN** |
| `StringCodePointSequence` indexing model | **BROKEN** |
| String→Char iteration/decoding bridge | **NOT IMPLEMENTED — correctly belongs to P12** |
| P11 verification | **BLOCKER: no behavioral tests/checks executed** |
| P11 lifecycle state | **CORRECT: keep `IN_PROGRESS / PARTIAL / UNVERIFIED`** |

---

# 2. Audit basis and reconciliation method

The aggregate audit uses three evidence classes:

1. **Implementation branch evidence** from `800e1793` and the lifecycle tip `51c4d35a`.
2. **The first independent review**, which focused heavily on semantic ownership, compiler lowering, coverage behavior, and String/Char integration.
3. **The supplied second independent audit**, which independently reviewed the implementation and added findings around String native type contracts, arbitrary-precision indices, String sequence semantics, Unicode-version pinning, compiler fallback authority, stale source comments, and sibling-branch generated-surface integration.

A final re-check was performed against the live branch for the material disagreements and newly surfaced defects.

Important verified branch facts:

- `615f5887...800e1793` changes **52 files**, with approximately **2,396 additions / 90 deletions**.
- Commit `800e1793` is the behavioral Char implementation.
- Commit `51c4d35a` is documentation/lifecycle only.
- No GitHub workflow run is associated with either audited commit.
- The lifecycle documents explicitly say no behavioral tests or crate checks were run.
- Only weak AST smoke tests were added for Char.
- `codex/lang007-c4-p4-r1-stdlib` and `stdlib-random-math` currently diverge from the common `615f5887` merge base and both touch generated/native integration surfaces.

---

# 3. Comparison of the two audits

## 3.1 Findings both audits agree on

Both audits independently converge on the following conclusions:

- `Char` should remain a distinct Unicode scalar type.
- `Char` should not alias `Int` or a one-scalar `String`.
- Immediate runtime representation is the correct choice.
- Rust `char` is an appropriate validated host representation for a Unicode scalar.
- `Expr::Char`, `Pattern::Char`, `StaticValueData::Char`, and `ExecutablePattern::Char` are the correct cross-layer shapes.
- `Char.fromCodePoint` is well designed and correctly uses arbitrary-precision Telos `Int` conversion.
- exact pattern matching should use scalar identity / `Same`, never user-defined equality.
- INC-6's closed Unicode scalar-domain ruling is logically coherent.
- the lexer/parser test additions are grossly insufficient.
- no semantic/runtime acceptance evidence exists yet.
- `rawByteAt` currently has a static/runtime type mismatch.
- `leadByteLen` is not a correct arbitrary-byte-offset character-boundary abstraction.
- String's current code-point iteration remains an `Int` API and is pre-P12.
- String backing storage remains valid UTF-8 and native slicing validates UTF-8 boundaries.
- P11 must remain `IN_PROGRESS / PARTIAL / UNVERIFIED`.

These findings form the stable core of the aggregate report.

## 3.2 Important valid findings added by the second audit

The second audit adds several findings that are accepted into this final report:

1. **Unchecked ordinary `match` fallback**  
   `compile_match_expr` synthesizes `ExecutablePattern`s directly from AST when no semantic lowering spec exists, but still emits `MatchInvariantFailure` after all arms. That weakens the checker→compiler authority boundary because the invariant failure is valid only after semantic exhaustiveness proof.

2. **String raw index natives violate arbitrary-precision `Int` semantics**  
   `rawByteAt` and `rawSlice` use `Value::as_int()` and even accept integral `Float`, while the semantic signatures require `Int`. Heap `LargeInt` values are therefore mishandled.

3. **`StringCodePointSequence` has incompatible coordinate systems**  
   `.size` counts Unicode scalars while `.at(i)` interprets `i` as a byte offset.

4. **Unicode property version should have an executable pin**  
   `Char.isWhitespace` / `isControl` delegate to the pinned Rust toolchain. If Unicode 17.0.0 is language-visible policy, a test should assert the toolchain Unicode table version.

5. **Stale source-level audit comments are materially misleading**  
   `phalcom-core/src/primitive/string.rs` still claims several issues that are no longer true.

6. **Sibling stdlib branch integration is a generated-surface risk**  
   Char and current math work diverge from the same baseline and touch overlapping native surface owners. Authored sources must be reconciled before regenerating derived native artifacts.

All six are valid.

## 3.3 Important finding present in the first audit but missed by the second

The second audit says Char match architecture is largely sound and focuses on the ordinary `match` fallback. It does **not** identify the more severe raw-AST binding bug affecting other pattern contexts.

The live branch still contains:

```rust
Pattern::Or { alternatives, .. } => {
    for p in alternatives {
        self.declare_pattern_locals(p, mutable)?;
    }
}
```

and later:

```rust
Pattern::Or { alternatives, .. } => {
    for p in alternatives {
        self.assign_pattern_from_slot(p, value_slot)?;
    }
}
```

`compile_if_let`, `compile_while_let`, and canonical `for` lanes all execute the legacy sequence:

```text
declare_pattern_locals(...)
emit_pattern_match_tests(...)
commit_pattern_bindings(...)
```

The test phase now branches among alternatives correctly, but declaration and commit do not know which alternative succeeded.

This is a **real blocking compiler defect**, not merely a missing test.

---

# 4. What is architecturally correct today

Before discussing defects, it is important not to destabilize the parts that are already right.

## 4.1 Char value ontology

The adopted ontology is correct:

```text
Char = exactly one Unicode scalar value
domain = U+0000..U+D7FF ∪ U+E000..U+10FFFF
```

It follows that:

```text
'a'       : Char
97        : Int
"a"       : String
```

These are semantically distinct values and types.

No implicit equivalence should exist between them.

## 4.2 Immediate runtime representation

`ValueTag::Char` plus a scalar payload is the appropriate representation.

The constructor:

```rust
pub const fn char(value: char) -> Self
```

prevents invalid scalar construction through the normal public route.

This preserves:

- allocation-free scalar values;
- distinct runtime tag identity;
- exact scalar equality;
- direct hashing;
- direct runtime class mapping;
- no GC tracing edge.

Do not replace this with heap-allocated character objects.

## 4.3 Lexer scalar validation

The lexer decodes a body/escape and requires exactly one scalar after decoding.

That is the correct rule.

A multibyte UTF-8 scalar such as:

```telos
'é'
```

is one Char.

A grapheme composed of multiple Unicode scalars such as `e` + U+0301 is **not** one Char.

Using `char::from_u32` for `\u{...}` validation correctly excludes surrogates and values above U+10FFFF.

## 4.4 Semantic/static identity

A dedicated:

```rust
StaticValueData::Char(char)
```

is correct.

Char participates independently in:

- static interning;
- semantic equality;
- semantic hash;
- content fingerprints;
- type computation;
- static evaluation;
- materialization.

This correctly avoids treating runtime VM values as semantic static authority.

## 4.5 Universe/native ownership

The Universe source declaration is the correct canonical type owner.

Native metadata and generated surfaces should enrich/project that declaration, not invent a second type identity.

The current Char API is a sensible first slice:

```text
codePoint
utf8Length
toString
isAscii
isAsciiDigit
isAsciiAlphabetic
isAsciiLowercase
isAsciiUppercase
isAsciiAlphanumeric
isAsciiHexDigit
isWhitespace
isControl
hash
Char.fromCodePoint
```

`Char.fromCodePoint` is particularly well implemented because it uses the arbitrary-precision Int path before converting to a Unicode scalar.

## 4.6 INC-6 scalar coverage semantics

The accepted rule is correct:

- Char is closed for match coverage.
- Char is non-subclassable for match purposes.
- one literal covers one exact scalar.
- duplicate literals are redundant.
- wildcard covers all remaining Char values.
- unresolved/underconstrained subjects remain blocked rather than falsely impossible.
- surrogates are not members of the domain.
- runtime matching is exact and non-overridable.

The semantic layer, not the compiler, must remain the sole authority for this coverage model.

---

# 5. Blocking finding CHAR-COMP-01 — raw-AST or-pattern bindings are still incorrect

**Severity:** **CRITICAL / P11 acceptance blocker**

## 5.1 Root cause

The compiler now correctly branches *tests* for a raw AST `Pattern::Or`, but its binding machinery is still structurally unaware of alternative selection.

The current legacy pipeline is:

```text
AST Pattern
  ↓
declare every binding leaf
  ↓
emit tests
  ↓
commit binding values by re-walking AST
```

That pipeline cannot correctly implement or-pattern binding semantics unless the declaration and commit steps share the same alternative-selection state.

They currently do not.

### Declaration bug

For:

```text
Left(x) | Right(x)
```

`declare_pattern_locals` recursively declares the `x` binding in **both** alternatives.

That is incompatible with the semantic rule that compatible or-pattern alternatives join to one logical/canonical binding.

Depending on local-declaration rules, this can produce:

- duplicate local declaration failure;
- accidental shadowing;
- multiple physical slots for one semantic binding;
- inconsistent body resolution.

### Commit bug

`assign_pattern_from_slot` then executes:

```rust
Pattern::Or { alternatives, .. } => {
    for p in alternatives {
        self.assign_pattern_from_slot(p, value_slot)?;
    }
}
```

That means after the test phase proves one alternative succeeded, commit walks **all** alternatives.

For a structural or-pattern such as:

```telos
Left(x) | Right(x)
```

matching `Left(42)`, the compiler can:

1. correctly test `Left`;
2. jump to or-pattern success;
3. commit `Left(x)`;
4. then also attempt to commit `Right(x)` from the same value.

This may issue invalid projection bytecode, overwrite bindings, or fail at runtime.

## 5.2 Affected compiler contexts

The live branch still uses the raw AST test/commit pipeline in:

- `compile_if_let`
- `compile_while_let`
- canonical `for` lane pattern binding

The bug is therefore broader than Char itself, but the P11 commit changed this path and claimed to repair disjunction behavior. It must be corrected before accepting P11.

## 5.3 Architecturally correct fix

Do **not** add another AST-side boolean such as `selected_or_alternative`.

The compiler should stop rediscovering pattern binding structure from raw syntax in these refutable/executable contexts.

The repository already has the correct abstraction:

```text
PatternResolution
    ↓ semantic projection
ExecutablePattern
+ canonical ExecutableBindingSpec[]
```

Use it.

### Required execution model

For each executable pattern site:

```text
AST pattern
    ↓
semantic pattern analysis
    ↓
canonical binding join
    ↓
PatternResolution
    ↓
ExecutablePattern + canonical binding table
    ↓
compiler PatternExecutionFrame
    ↓
stage bindings while testing
    ↓
commit canonical bindings exactly once on total success
```

`ExecutablePattern::Or` already supports alternative control flow while using shared staging slots indexed by canonical binding index.

That is the correct architecture.

### Required contextual failure semantics

The same executable pattern machine can serve multiple source contexts while the *failure edge* remains context-specific:

```text
match arm       -> try next arm
if let          -> else / None path
while let       -> terminate loop
for             -> raise ordinary pattern mismatch
```

Pattern semantics and binding staging should be shared; only failure continuation differs.

## 5.4 Required regression tests

At minimum, introduce a two-case algebraic value and verify all of:

```telos
if let Left(x) | Right(x) = Left(42) { ... }
if let Left(x) | Right(x) = Right(42) { ... }

while let Left(x) | Right(x) = ... { ... }

for Left(x) | Right(x) in values { ... }
```

Required assertions:

- exactly one logical `x` exists;
- successful Left case binds `x == 42`;
- successful Right case binds `x == 42`;
- failed alternatives do not mutate the visible binding;
- no projection of the non-selected alternative is executed;
- captured `x` behaves correctly;
- branch/loop failure releases scratch slots correctly.

---

# 6. Finding CHAR-COMP-02 — ordinary `match` fallback bypasses semantic proof

**Severity:** **HIGH**

## 6.1 Current behavior

`compile_match_expr` first looks for semantic `MatchLoweringSpec`.

If absent, it currently synthesizes executable patterns directly from AST for syntax considered "structurally complete":

```text
Wildcard
Char
Name
Tuple
Or
List
```

The compiler then lowers those patterns through the same match body and unconditionally emits:

```rust
Bytecode::MatchInvariantFailure
```

after all arms.

## 6.2 Why this violates the authority boundary

`MatchInvariantFailure` means:

> normal execution cannot reach this point because semantic analysis proved the match total.

That implication is valid only if semantic exhaustiveness has actually been established.

AST syntax alone cannot establish it.

A standalone/unlinked compile of conceptually:

```telos
match c {
    'a' => 1
}
```

must not silently acquire the invariant "all Char values are covered".

## 6.3 Required architectural invariant

Adopt:

```text
MatchInvariantFailure may be emitted
IFF
a canonical semantic match product proves exhaustiveness.
```

That invariant should be general, not Char-specific.

## 6.4 Correct fix

Preferred:

- normal executable `match` compilation requires semantic `MatchLoweringSpec`;
- if absent, return `MissingMatchLoweringSemantics` or equivalent;
- preserve an explicitly named test-only/unchecked compiler helper only if isolated unit tests genuinely need it.

Do not keep an implicit production fallback whose result is indistinguishable from proof-backed lowering.

If a non-semantic fallback must exist for tooling/bootstrap reasons, its fallthrough must have ordinary runtime mismatch behavior rather than `MatchInvariantFailure`, and its contract must be explicit.

---

# 7. Finding CHAR-SEMA-01 — declaration Char-literal rejection is owned by the compiler

**Severity:** **HIGH architectural defect; may be behaviorally masked**

The P11 implementation prevents declaration destructuring containing a Char literal by calling a compiler-side `contains_char_literal(pattern)` and raising `InvalidExecutablePattern`.

The desired language rule is acceptable.

The ownership is not.

## 7.1 Why compiler-only rejection is wrong

Whether a source pattern is legal in a declaration context is a source-language semantic rule.

If only the compiler rejects it:

- semantic tooling can consider invalid source valid;
- LSP/IDE diagnostics disagree with execution;
- static analysis can create products for a program codegen later refuses;
- compiler internal errors leak into a user-language validation concern.

## 7.2 Correct fix

Pattern checking should be explicitly context-sensitive.

Introduce/use a semantic context classification comparable to:

```text
MatchArm
IfLet
WhileLet
ForBinding
DeclarationBinding
ParameterBinding
```

Then validate whether each resolved pattern form is legal for the context.

For the accepted P11 rule:

```text
PatternResolution::Char(_)
```

must be rejected in declaration binding context, including when nested inside tuple/list/record/etc.

The compiler may keep a defensive assertion, but it must no longer be the primary language validator.

## 7.3 Diagnostic requirement

The semantic diagnostic should explain the actual source rule, e.g.:

```text
character literal patterns are refutable and cannot be used in declaration destructuring
```

rather than exposing `InvalidExecutablePattern`.

---

# 8. Finding CHAR-COV-01 — scalar-domain semantics are sound, but bounded operational behavior must be stated precisely

**Severity:** **MEDIUM**

The implementation mathematically models the Char domain as 1,112,064 scalar values:

```text
0x110000 - 0x800 = 1,112,064
```

and a complete singleton set can theoretically exhaust it.

That is logically correct.

However, the semantic engine also has a default query step budget of:

```text
max_steps = 10,000
```

and the coverage code charges bounded work while traversing matrix rows/specializations.

A source program containing more than one million literal arms is therefore not realistically expected to complete a single bounded semantic query merely because the mathematical set is finite.

## 8.1 This is not a semantic unsoundness bug

A resource-bounded checker is allowed to return a blocked/budget-exceeded result for pathological source.

The mistake would be documenting:

> every complete million-literal source program is operationally guaranteed to receive an exhaustiveness proof under the default budget.

That guarantee does not exist.

## 8.2 Required correction

Clarify the contract:

```text
The Char domain is formally closed.
A catch-all proves totality immediately.
A complete singleton set is mathematically total.
Semantic checking remains resource-bounded and may return Blocked/BudgetExceeded
for pathological proofs.
```

Do **not** raise the default semantic budget to >1,112,064 merely to make the pathological case pass.

## 8.3 Optional future representation improvement

If future language work introduces scalar ranges or property patterns, replace the literal-only `BTreeSet<char>` model with a scalar interval/complement abstraction such as:

```text
ScalarSet =
    intervals over
    [0x0000, 0xD7FF] ∪ [0xE000, 0x10FFFF]
```

with:

- singleton insertion;
- union;
- subtraction;
- `is_full`;
- `first_missing`.

That is not required to validate the current literal-only semantics.

---

# 9. Finding CHAR-VER-01 — verification is far below the acceptance contract

**Severity:** **BLOCKER**

The implementation changes 52 files across:

```text
lexer
parser
AST
semantic typing
pattern resolution
coverage
static evaluation
static-value storage
fingerprints
materialization
compiler lowering
VM representation
runtime class mapping
Universe source
native metadata
generated native surface
tooling/source indexing
runtime primitives
```

Yet:

- no behavioral tests were run;
- no crate checks were run;
- no semantic/core regression test files were added;
- no GitHub workflow run exists for the audited commits.

The only new AST tests are effectively:

```rust
assert!(Lexer::new("'é'").next().unwrap().is_ok());
```

and:

```rust
assert_eq!(parse_source("'é'", 0).unwrap().statements.len(), 1);
```

These do not even prove that the resulting token/AST node is Char.

## 9.1 Required acceptance rule

P11 must not move beyond:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

until focused tests exist and have actually been run.

A clean tree and `git diff --check` are not behavioral evidence.

---

# 10. Char lexer/parser verification requirements

The lexer implementation is structurally good. The missing work is exact regression coverage.

## 10.1 Valid literal cases

At minimum:

```text
'a'
'é'
'中'
'🙂'
'\n'
'\r'
'\t'
'\\'
'\''
'\u{0}'
'\u{41}'
'\u{D7FF}'
'\u{E000}'
'\u{10FFFF}'
```

For each case assert:

- exact `Token::Char(expected)`;
- exact source byte range;
- exact `Expr::Char` value/range;
- pattern parsing where applicable.

## 10.2 Invalid literal cases

At minimum:

```text
''
'ab'
'é'                 # e + combining acute: two scalars
raw LF
raw CR
raw CRLF
'\q'
'\u'
'\u{}'
'\u{XYZ}'
'\u{1234567}'
'\u{D800}'
'\u{DFFF}'
'\u{110000}'
unterminated quote
unterminated Unicode escape
```

Assert:

- exact lexical diagnostic kind;
- exact source range;
- recovery behavior / EOF behavior;
- no accidental String or Int token.

## 10.3 Minor lexer hardening

The implementation contains an internal:

```rust
expect("one decoded scalar was recorded")
```

whose invariant is currently valid.

This is low risk, but the lexer can avoid a latent panic by representing the scalar-count state structurally or matching `(count, value)` without `expect`.

This is not a P11 blocker.

---

# 11. Char semantic/static/runtime test requirements

## 11.1 Type identity

```telos
let c = 'a'
```

must prove exactly:

```text
type(c) == Char
```

and remain disjoint from:

```text
Int
String
```

## 11.2 Static value identity

Required tests:

```text
intern Char('a') == intern Char('a')
intern Char('a') != intern Char('b')
Char('a') != Int(97)
equal Char semantic values hash equally
content fingerprint is deterministic
cold and incremental analysis agree
materialization produces Value::char
```

A change from `'a'` to `'b'` in a static constant must invalidate the appropriate fingerprint/dependency result.

## 11.3 Runtime representation

Required:

```text
'a'.class == Char
'a'.codePoint == 97
'a'.utf8Length == 1
'é'.utf8Length == 2
'中'.utf8Length == 3
'🙂'.utf8Length == 4
'é'.toString == "é"
```

`Char.fromCodePoint`:

```text
97          -> Some('a')
0           -> Some('\u{0}')
0xD7FF      -> Some(...)
0xE000      -> Some(...)
0x10FFFF    -> Some(...)
-1          -> None
0xD800      -> None
0xDFFF      -> None
0x110000    -> None
huge BigInt -> None
```

Identity distinctions:

```text
'a' != 97
'a' != "a"
'a' == 'a'
'a' !== 'b'
```

## 11.4 Classification

Exercise representative positives/negatives for:

- ASCII;
- decimal digit;
- alphabetic upper/lower;
- alphanumeric;
- hex digit;
- Unicode White_Space;
- Unicode control (`Cc`).

---

# 12. Finding CHAR-UNICODE-01 — Unicode classification version needs an executable guard

**Severity:** **MEDIUM**

Language documentation pins Unicode classification to the Rust toolchain / Unicode 17.0.0.

Runtime methods delegate to:

```rust
char::is_whitespace()
char::is_control()
```

This is a reasonable implementation.

But if the exact Unicode data version is language-visible, the repository should detect a toolchain upgrade that silently changes these predicates.

## Correct fix

Add a focused regression against the standard library Unicode-version constant supported by the pinned toolchain.

Conceptually:

```rust
assert_eq!(std::char::UNICODE_VERSION, (17, 0, 0));
```

Use the actual constant type exposed by the pinned nightly.

This turns a documentary assumption into an executable language compatibility check.

---

# 13. Finding CHAR-REPR-01 — raw-word Char invariant is not fully defended

**Severity:** **LOW hardening**

Normal construction through:

```rust
Value::char(char)
```

is safe.

However:

```rust
Value::from_raw_words(payload, meta)
```

only asserts reserved-bit validity and does not validate that a `Char` tag has a valid Unicode-scalar payload.

Later:

- `as_char()` correctly rejects invalid payloads;
- raw equality/hash cast the payload to `u32`.

Because this is an internal trusted representation boundary, it is not presently a user-reachable correctness defect.

Still, a debug representation validator should assert:

```text
if tag == Char:
    payload <= U+10FFFF
    payload not in D800..DFFF
```

especially if raw words are used by serialization, image bootstrapping, cache transport, or bytecode constants in future.

---

# 14. String audit — foundation versus defects

The String backing representation has one major strength:

> runtime Strings are Rust `String` values and therefore contain valid UTF-8.

Native `rawSlice` also checks:

```rust
s.is_char_boundary(start)
s.is_char_boundary(end)
```

before slicing.

Therefore P12 should **not** create another general malformed-UTF-8 decoder inside guest Telos code.

The correct design is to use the valid backing representation as the authority for scalar boundaries and decoding.

However, the current public/internal String model has several concrete defects.

---

# 15. Finding STR-01 — `rawByteAt` has a false result type

**Severity:** **HIGH**

Universe declaration:

```telos
internal native rawByteAt(_ index: Int) -> Int
```

Native metadata also says:

```text
(Int) -> Int
```

But the runtime returns `None` for:

- negative index;
- out-of-range index;
- unsupported numeric representation.

Actual behavior is:

```text
Int -> Option<Int>
```

## Correct fix

Change the owner declarations to:

```telos
internal native rawByteAt(_ index: Int) -> Option<Int>
```

and native metadata to:

```text
(Int) -> Option<Int>
```

Then regenerate the native surface.

Do not hide the mismatch behind `Dynamic`.

All callers should be type-checked against the real optional result.

This is mandatory P13 type hygiene and is also a prerequisite for a clean P12 text substrate.

---

# 16. Finding STR-02 — String index natives do not implement Telos `Int`

**Severity:** **HIGH**

`Char.fromCodePoint` correctly consumes arbitrary-precision Int:

```rust
expect_int_big(...)
```

String raw helpers do not.

`string_raw_byte_at` and `string_raw_slice` currently use:

```rust
args[n].as_int()
```

which only observes immediate integer representation, and then also accept:

```rust
args[n].as_float()
```

when integral.

That is the reverse of the declared contract.

## Required invariant

```text
A native whose semantic signature requires Int accepts Telos Int values,
including heap LargeInt, and does not opportunistically accept Float.
```

## Correct fix

Use the canonical Int extraction helper:

```rust
let index = crate::primitive::int::expect_int_big(&args[0], vm)?;
```

Then apply operation-specific conversion.

### `rawByteAt`

- negative -> `None`;
- nonnegative mathematical integer not representable as `usize` -> `None`;
- >= byte length -> `None`;
- otherwise -> byte Int.

### `rawSlice`

For each endpoint:

- reject negative according to the defined range-error policy;
- convert with `to_usize()`;
- conversion overflow means out-of-range, not "not an Int";
- remove Float coercion entirely.

---

# 17. Finding STR-03 — `leadByteLen` is not a character-boundary operation

**Severity:** **HIGH for P12**

Current logic:

```telos
if b < 128 {
    1
} else if b < 224 {
    2
} else if b < 240 {
    3
} else {
    4
}
```

misclassifies continuation bytes `0x80..0xBF` as two-byte lead bytes.

For:

```text
"é" = C3 A9
```

byte offset `1` contains `A9`, a continuation byte, but current `leadByteLen(1)` returns `2`.

## Minimal fix

If this helper remains:

```text
00..7F -> 1
80..BF -> None
C0..DF -> 2
E0..EF -> 3
F0..F7 -> 4
```

although because Strings are already guaranteed valid UTF-8, invalid leading ranges should not appear at true boundaries.

## Better architectural fix

P12 should make host-owned String operations authoritative:

```telos
String.isCharBoundary(_ byteOffset: Int) -> Bool
String.decodeCharAt(_ byteOffset: Int) -> Option<Char>
```

implemented from Rust `str` behavior:

```text
is_char_boundary
chars().next()
char.len_utf8()
```

Then iteration should use these operations or equivalent direct native machinery.

Do not maintain several independent guest-side UTF-8 decoders.

---

# 18. Finding STR-04 — `StringCodePointSequence` violates sequence indexing semantics

**Severity:** **HIGH**

Current object:

```text
StringCodePointSequence.size
```

counts Unicode scalars.

But:

```telos
at(_ byteOffset) { _string.codePointAt(byteOffset) }
```

takes a byte offset.

These are incompatible coordinate systems.

For:

```text
"éa"
bytes = C3 A9 61
size = 2
```

a normal sequence API implies:

```text
at(0) = first element
at(1) = second element
```

but current `at(1)` addresses byte `A9`, a continuation byte.

## Correct P12 migration

Make:

```telos
String.chars -> StringChars
```

the canonical scalar iteration API.

`StringChars` should implement:

```text
Iterable<Item = Char>
```

and should not claim O(1) random access.

If `String.codePoints` remains:

- either make `at(i)` ordinal scalar indexing, explicitly O(n);
- or remove/demote its indexed-sequence interface.

Keep byte-coordinate operations explicitly named:

```text
decodeCharAt(byteOffset)
codePointAt(byteOffset)
sliceBytes(start, end)
isCharBoundary(byteOffset)
```

Coordinate systems must be obvious from API names.

---

# 19. Finding STR-05 — current String scalar iteration returns Int instead of Char

**Severity:** **EXPECTED P12 gap**

Current:

```text
StringCodePointIterator.next -> Option<Int>
```

P11 intentionally did not change this.

The correct P12 target is:

```text
String.chars -> Iterable<Char>
```

with direct Char values.

Do not re-encode a scalar as Int at the iteration boundary and ask parser/TOML consumers to reconstruct Char.

The text stack should become:

```text
UTF-8 String
    ↓ byte offset
decodeCharAt
    ↓
Char
    ↓ utf8Length
next byte offset
```

Parser source spans must remain byte-addressed.

---

# 20. Finding STR-06 — stale String source comments are misleading

**Severity:** **MEDIUM maintenance defect**

`phalcom-core/src/primitive/string.rs` still opens with a "BLOCKING ISSUES — IMPLEMENTATION INCOMPLETE / NON-FUNCTIONAL" comment that claims, among other things:

- `split()` crashes because `List.new` lacks call syntax;
- trim dispatch is broken;
- `codePointAt` is still a stub;
- the String test corpus is empty.

Those statements no longer describe the live source.

Historical audit notes should not remain as authoritative-looking module documentation after the implementation changes.

## Correct fix

Delete or rewrite the header.

Keep only current invariants and real known limitations in source.

Historical defects belong in incident/lifecycle documents.

Apply the same cleanup to stale fixture comments.

---

# 21. String performance debt — valid but not a P11/P12 blocker

Several String algorithms repeatedly allocate immutable concatenations.

Examples include repetition and replacement patterns of the form:

```text
result = result + ...
```

This can create quadratic byte movement.

The correct future abstraction is the planned P16 `StringBuilder`.

Do not complicate P12 by introducing another ad-hoc mutable byte builder.

P12 should establish semantic correctness and the String↔Char boundary first.

---

# 22. P12 architectural requirements

P12 should establish a single coherent byte-coordinate/scalar-value model.

## 22.1 Required public/internal primitives

Recommended minimum:

```telos
String.isCharBoundary(_ byteOffset: Int) -> Bool
String.decodeCharAt(_ byteOffset: Int) -> Option<Char>
String.chars -> StringChars
```

Potentially:

```telos
String.sliceBytes(_ start: Int, _ end: Int) -> String
```

with typed/structured failure semantics if current raw slicing remains internal.

## 22.2 Required boundary behavior

For:

```text
"Aé中🙂"
```

UTF-8 boundaries are:

```text
A   0 -> 1
é   1 -> 3
中  3 -> 6
🙂  6 -> 10
```

`isCharBoundary` should be true at:

```text
0, 1, 3, 6, 10
```

and false at continuation positions.

`decodeCharAt` should return:

```text
0 -> 'A'
1 -> 'é'
3 -> '中'
6 -> '🙂'
```

and `None` for continuation positions and EOF.

EOF is a valid slicing boundary but does not begin another Char:

```text
isCharBoundary(10) == true
decodeCharAt(10) == None
```

That distinction must be tested.

## 22.3 Character iteration

`StringChars` must yield:

```text
'A', 'é', '中', '🙂'
```

as runtime Char values.

Iterator state may retain a byte offset internally. That is an implementation detail.

## 22.4 Source coordinate invariant

Do not change source spans to scalar indices.

Keep:

```text
source ranges = UTF-8 byte offsets
text element value = Char
```

This preserves parser/LSP byte-range consistency.

---

# 23. P13 type-hygiene requirements

P13 should repair String signatures rather than preserving Dynamic-era compatibility.

At minimum:

- `rawByteAt : Int -> Option<Int>`;
- exact Int handling through arbitrary-precision Int ownership;
- no integral-Float coercion for Int-only primitives;
- deterministic String-returning methods typed as String;
- split results typed as `List<String>`;
- prefix/suffix/search arguments typed as String;
- avoid `List<Dynamic>` where the result is statically known.

Do not paper over native declaration mismatches with `Dynamic`.

---

# 24. Generated-surface and sibling-branch integration

The independent audit correctly raised a merge risk that is external to Char semantics but relevant to acceptance.

The current branches:

```text
codex/lang007-c4-p4-r1-stdlib
stdlib-random-math
```

diverge from merge base:

```text
615f5887
```

and touch overlapping authored/generated integration surfaces including native metadata and generated native records.

The correct integration rule is:

1. merge/rebase authored source changes;
2. reconcile canonical Universe/native registration owners;
3. resolve primitive-module registration;
4. resolve lifecycle docs;
5. rerun the native-surface generator;
6. accept the freshly generated combined artifact;
7. run generated-surface consistency tests.

Do not manually merge `phalcom-native-surface/src/generated.rs` as though it were an independent source of truth.

---

# 25. Lifecycle-record corrections required now

The existing high-level status is honest and should remain unchanged.

However, the walkthrough/handoff should be amended before the next implementation cycle.

## 25.1 Keep

```text
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

## 25.2 Clarify AST test evidence

Replace wording that may imply useful coverage with:

> Minimal lexer/parser smoke tests for `'é'` exist, but they do not assert the Char token/AST shape and no complete valid/invalid literal matrix has been added or run.

## 25.3 Correct the disjunction claim

The current record says conditional-pattern lowering was adjusted.

It should explicitly state:

> Or-pattern alternative test branching was repaired, but raw-AST declaration/commit remains incorrect because bindings are still declared and committed across every alternative. `if let`, `while let`, and `for` must converge on canonical semantic executable-pattern lowering.

## 25.4 Record compiler fallback debt

Add:

> Ordinary `match` retains a standalone AST-to-executable fallback that can bypass semantic exhaustiveness authority. `MatchInvariantFailure` must be restricted to proof-backed semantic lowering.

## 25.5 Separate P11/P12/P13 ownership

Keep the boundaries explicit:

```text
P11 — Char identity, literals, scalar patterns, runtime class
P12 — String↔Char decoding, boundaries, chars iteration
P13 — typed String API/native signature hygiene
P16 — StringBuilder/performance substrate
```

---

# 26. Required cross-layer invariants

The remediation should be judged against these invariants.

| ID | Invariant |
|---|---|
| `CHAR-SCALAR` | Every Char denotes exactly one valid Unicode scalar. |
| `CHAR-DISTINCT` | Char is never semantically represented as Int or one-scalar String. |
| `CHAR-STATIC` | Static Char identity belongs to semantic static values, not runtime VM tags. |
| `CHAR-IMMEDIATE` | Runtime Char is allocation-free. |
| `CHAR-UNIVERSE` | Canonical class/type identity originates in Universe source. |
| `CHAR-MATCH` | Literal matching uses exact scalar identity, never user equality. |
| `CHAR-COVERAGE` | Coverage/exhaustiveness belongs exclusively to semantics. |
| `CHAR-UNKNOWN` | Underconstrained subjects remain blocked rather than falsely impossible/total. |
| `PATTERN-JOIN` | Compatible or-pattern alternatives share one canonical binding identity. |
| `PATTERN-ATOMIC` | Failed alternatives commit no user-visible binding. |
| `PATTERN-PROJECTION` | Compiler executes semantic pattern products; it does not reconstruct semantic identity from AST. |
| `MATCH-TOTAL` | `MatchInvariantFailure` is reachable only in code generated from a semantically proven-total match. |
| `DECL-PATTERN` | Declaration-pattern legality is decided by semantic context checking. |
| `STRING-VALID` | Every runtime String remains valid UTF-8. |
| `STRING-COORD` | Byte offsets and scalar ordinals are never silently conflated. |
| `STRING-BOUNDARY` | Public byte slicing validates UTF-8 boundaries. |
| `STRING-CHAR` | Canonical character iteration yields Char. |
| `STRING-INDEX` | Any `at(i)` defines `i` in the same coordinate system as `size`. |
| `INT-EXACT` | Int-typed native parameters accept Telos arbitrary-precision Int semantics. |
| `NATIVE-TYPE` | Native runtime values always inhabit declared semantic result types. |
| `UNICODE-PIN` | Language-visible Unicode property behavior is pinned by executable toolchain-version evidence. |
| `GENERATED` | Generated native surface is regenerated after authored integration. |

---

# 27. Required implementation correction order

This is **not** the next-turn patch plan; it is the dependency order the implementation plan must respect.

## Phase A — close the compiler correctness hole

1. Project semantic pattern products for `if let`, `while let`, and relevant `for` binding sites.
2. Use canonical `ExecutableBindingSpec` identity and `PatternExecutionFrame`.
3. Stage successful-alternative bindings.
4. Commit exactly once on total pattern success.
5. Remove/deprecate the raw AST `declare/test/commit` route for these contexts.
6. Add or-pattern binding regressions.

This is the highest-priority fix.

## Phase B — restore semantic authority

1. Move declaration Char-literal legality checks into semantic pattern-context validation.
2. Keep compiler checks only as assertions/defense in depth.
3. Require semantic `MatchLoweringSpec` for production `match` compilation.
4. Restrict `MatchInvariantFailure` to semantically proven exhaustive matches.

## Phase C — actually verify P11

Add the complete lexer/parser, semantic/static, coverage, compiler, and runtime test matrix.

Run:

- focused `phalcom-ast` tests;
- focused `phalcom-semantic` tests;
- affected `phalcom-core` tests/corpus;
- directly affected crate checks;
- native-surface consistency/generator checks.

Only after real evidence should lifecycle verification advance.

## Phase D — clean bounded-coverage and compatibility contracts

- document bounded checker behavior for pathological million-literal coverage;
- add Unicode version pin;
- add raw Char representation debug invariant if appropriate;
- remove stale String review comments.

## Phase E — P12 String↔Char bridge

Implement:

```text
isCharBoundary
decodeCharAt
String.chars
checked byte slicing contract
```

and migrate text consumers toward Char while preserving byte source spans.

## Phase F — P13 String type hygiene

Fix:

```text
rawByteAt Option<Int>
LargeInt index handling
Float coercion
typed deterministic String methods
List<String> results
```

## Phase G — integrate sibling native work

Reconcile authored Char/math native sources, regenerate generated surface, and run combined consistency tests.

---

# 28. Acceptance test matrix

The next implementation plan should turn this table into concrete test targets.

| Layer | Mandatory evidence |
|---|---|
| Lexer | ASCII, 2/3/4-byte scalar, every supported escape, NUL, scalar boundaries, malformed escape, surrogate, >max, empty, multiple scalar, raw CR/LF |
| Parser | exact `Expr::Char`, exact `Pattern::Char`, or-pattern literal parsing, exact ranges |
| Semantic typing | literal type exactly Char; Int/String disjointness |
| Static values | interning, equality, hash, fingerprint, incremental invalidation, materialization |
| Coverage | literal+wildcard totality, duplicate redundancy, impossible wrong subject, unknown blocked, surrogate-gap witness |
| Canonical match compiler | exact Char match/mismatch; no user equality dispatch |
| Or-pattern compiler | `if let`, `while let`, `for`; both alternatives; canonical single binding; no failed-alternative commit |
| Declaration semantics | direct and nested Char literal pattern rejected semantically |
| Runtime | class, codePoint, utf8Length 1..4, toString, equality, hash |
| `fromCodePoint` | negative, D7FF, D800, DFFF, E000, 10FFFF, 110000, huge BigInt |
| Unicode predicates | ASCII families, whitespace, control, Unicode-version guard |
| String raw index | immediate Int, LargeInt, negative, huge positive, Float rejection |
| String bridge P12 | boundaries, continuation rejection, EOF distinction, decode to Char, chars iteration |
| String sequence | coherent ordinal indexing or removal of inconsistent indexed contract |
| Generated native | regeneration and parity/consistency tests after branch integration |

---

# 29. Findings register

| ID | Severity | Disposition |
|---|---:|---|
| `CHAR-COMP-01` raw-AST or-pattern declaration/commit across all alternatives | **Critical** | Must fix before P11 acceptance |
| `CHAR-VER-01` no meaningful behavioral verification | **Blocker** | Must add/run focused test suite |
| `CHAR-COMP-02` ordinary `match` fallback bypasses semantic proof | **High** | Require semantic lowering or explicit unchecked contract |
| `CHAR-SEMA-01` declaration literal rejection lives in compiler | **High** | Move source legality to semantic context validation |
| `STR-01` `rawByteAt` lies about return type | **High** | Change to `Option<Int>` and regenerate surface |
| `STR-02` raw String indices violate arbitrary-precision Int contract / accept Float | **High** | Use canonical BigInt extraction; reject Float |
| `STR-03` `leadByteLen` accepts continuation offsets | **High** | Replace with boundary-aware host-owned decoding |
| `STR-04` code-point sequence size/index coordinate mismatch | **High** | Make ordinal or replace with `chars` iterable |
| `CHAR-COV-01` formal scalar closure vs bounded operational proof | **Medium** | Clarify bounded checker contract; no huge budget increase |
| `CHAR-UNICODE-01` Unicode data version not executable | **Medium** | Add pinned Unicode-version regression |
| `STR-06` stale String source audit comments | **Medium** | Remove/update |
| native-surface sibling-branch integration | **Medium integration risk** | Merge authored sources, regenerate derived surface |
| `CHAR-REPR-01` raw-word Char payload not debug-validated | **Low** | Add defensive invariant if raw-word transport remains |
| lexer internal `expect` | **Low** | Optional structural-state cleanup |

---

# 30. Final architectural recommendation

The correct direction is **convergence**, not another feature-specific patch.

The repository already contains the pieces of a proper architecture:

```text
semantic PatternResolution
canonical binding identity
ExecutablePattern
PatternExecutionFrame
staged bindings
exact runtime pattern primitives
```

The main compiler defect exists because some control-flow forms still execute an older AST-driven pattern engine in parallel with the semantic executable-pattern path.

The next implementation should make the semantic projection path the universal executable pattern authority.

That simultaneously fixes:

- or-pattern binding identity;
- alternative atomicity;
- Char pattern consistency;
- future literal pattern families;
- semantic/compiler diagnostic agreement;
- pattern lowering duplication.

Likewise, P12 should make String's valid Rust UTF-8 backing representation the single decoding/boundary authority rather than multiplying guest-side UTF-8 classifiers.

The resulting architecture should look like this:

```text
                    SOURCE PATTERN
                         │
                         ▼
                Semantic Pattern Analysis
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
      Coverage / legality      Canonical bindings
             │                       │
             └───────────┬───────────┘
                         ▼
                 ExecutablePattern
                         │
                         ▼
                PatternExecutionFrame
                         │
             stage → test → commit
                         │
          context-specific failure edge
```

and:

```text
                  Rust UTF-8 String
                         │
           byte boundary / byte offset
                         ▼
                String decoding owner
                         │
                         ▼
                        Char
                         │
                  Iterable<Char>
```

These are the two structural corrections that should guide the implementation plan.

---

# 31. Final acceptance recommendation

## P11 Char

**HOLD. Do not redesign. Do not mark complete.**

The Char feature is well designed, but completion requires:

1. fixing legacy/raw-AST or-pattern execution;
2. hardening semantic match authority;
3. moving declaration literal legality to semantics;
4. adding and running the full verification matrix;
5. recording actual evidence.

Until then:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

is the only defensible lifecycle state.

## P12 String–Char bridge

**Proceed after/alongside the P11 compiler remediation, using the accepted architecture.**

Mandatory root fixes include:

- boundary-aware decoding;
- Char iteration;
- coherent byte-vs-scalar coordinates;
- no malformed UTF-8 creation.

## P13 String type hygiene

**Treat the confirmed native contract defects as mandatory, not compatibility quirks.**

In particular:

```text
rawByteAt -> Option<Int>
exact arbitrary-precision Int handling
no Float coercion for Int signatures
precise collection/string result types
```

## Generated/native integration

**Reconcile authored Char and sibling stdlib changes before certifying generated surfaces.**

Generated files are outputs. Regenerate them from the merged authoritative sources.

---

# 32. Decision for the next turn

The next turn can now produce a patch-grade implementation plan without revisiting the fundamental Char design.

That plan should be organized around these implementation tracks:

1. **Pattern semantic/executable convergence**
2. **P11 test and verification closure**
3. **semantic authority cleanup**
4. **String native contract root fixes**
5. **P12 boundary/Char iteration implementation**
6. **P13 type hygiene**
7. **generated-surface integration and final gates**

No additional architecture decision is required for the central Char model.

The only remaining design choice worth making explicit during planning is how broadly to migrate non-`match` pattern contexts to semantic executable products in the first patch. The audit recommendation is to migrate `if let`, `while let`, and `for` together because they currently share the same defective raw-AST binding machinery and because partial migration would leave duplicated semantic authority in place.
