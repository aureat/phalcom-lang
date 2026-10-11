---
id: LANG007.C4.P4.R1-STD
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-proposal
title: "C4 Standard Library Text Substrate for parse v1 and TOML"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
parent_record: LANG007.C4.P4.R1
prepared_against_revision: 22cf74739a59fe178c0e3c1bb75c258a14327535
prepared: 2026-10-07
updated: 2026-10-07
---

# LANG007.C4.P4.R1-STD — Standard Library Text Substrate

> **For agentic workers:** this is a planning proposal. After approval, split the
> ordered slices into numbered C4 execution plans and use
> `superpowers:executing-plans` or `superpowers:subagent-driven-development`.
> Do not implement this proposal as one unreviewed cross-layer patch.

**Goal:** Establish the smallest strong Telos standard-library substrate required
for Unicode-correct `parse` v1 and production-oriented `toml`, while preserving
byte-addressed source spans and the existing C4 semantic/runtime ownership model.

**Architecture:** Add a distinct immediate `Char` scalar representing one valid
Unicode scalar value, then build typed UTF-8 decoding, character iteration, safe
byte-boundary slicing, and text APIs over the existing immutable UTF-8 `String`.
Add immutable homogeneous snapshots, typed numeric conversion, and a mutable
`StringBuilder` as separate follow-on slices. `parse` remains responsible for
parser state, composition, failure, commitment, and parser-specific spans; it
consumes the standard-library text substrate and does not become a second UTF-8
or character implementation.

**Tech Stack:** Telos source modules; Rust lexer/parser, semantic, compiler,
VM/value, heap, native metadata, and Universe bootstrap; existing `Iterable` /
`Iterator` associated-type contracts; `Option`/`Result`; focused Rust integration
and Telos runtime fixtures.

---

## 1. Proposal status and C4 relationship

This proposal is a formal `LANG007.C4` follow-on. It is filed under the existing
`LANG007.C4.P4.R1` companion directory at the user's requested location so the
proposal is co-located with the current C4 corrective record. It does not replace
the R1 issue register, change P4's FieldKey ownership, or claim that any current
P4/R1 issue is resolved.

The eventual execution records should be added to the canonical C4 checkpoint
ledger as separate numbered plans. The provisional decomposition is:

| Proposed plan | Scope | Dependency | Acceptance role |
|---|---|---|---|
| `LANG007.C4.P11` | `Char` scalar, literals, semantic/runtime integration | Current C4 language/runtime authority | Character identity gate |
| `LANG007.C4.P12` | String–Char bridge, UTF-8 boundaries, iteration | P11 | Unicode text correctness gate |
| `LANG007.C4.P13` | Typed String hygiene and text-position utilities | P12 | Standard String API gate |
| `LANG007.C4.P14` | Immutable homogeneous indexed snapshots | P11; existing iteration laws | Stable parser-value gate |
| `LANG007.C4.P15` | Typed integer/floating conversion | P11/P12 for diagnostics only | Numeric/TOML gate |
| `LANG007.C4.P16` | `StringBuilder`, parse migration, TOML substrate proof | P12/P13; consumes P14/P15 | Follow-on closure |

The C4 checkpoint has accepted the follow-on and registered P11–P16. P11 is
active under its own execution plan; P12–P16 remain proposed and must not be
treated as started. P11's current implementation is committed as `800e1793`
and remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; its interim
[walkthrough](../../LANG007.C4.P11-walkthrough.md),
[handoff](../../LANG007.C4.P11-handoff.md), and
[INC-6 record](../../LANG007.C4.P11-pattern-coverage-incident.md) track the
implementation and required verification. C4 closure will require this
follow-on's focused acceptance in addition to its existing structural/static
gates.

This follow-on is orthogonal to the current P4 semantic-owner correction. It may
be planned in parallel with P4, but it must not modify P4's semantic owner,
StaticConstant, FieldKey, row, or computed-key architecture. It is a prerequisite
for serious Unicode-capable `parse` and production-oriented `toml`; an exploratory
ASCII-only TOML vertical may remain separate and explicitly provisional.

---

## 2. Acceptance objective

The follow-on is accepted only when all of the following are true:

```text
Char is a first-class Unicode-scalar value and character literals produce Char.
String can decode one Char at a byte offset and report the next byte boundary.
Public String slicing cannot create malformed UTF-8.
String character iteration has coherent Char sequence semantics.
parse.text consumes Char predicates and preserves byte-addressed spans.
Core String results and iterator views retain statically known element types.
Parser source snapshots use actual immutable sequence ownership.
Int and Float expose typed conversion paths for format libraries.
StringBuilder provides efficient immutable String construction.
TOML-shaped examples need no ASCII constants, manual UTF-8 decoding,
Dynamic string collections, or parser-owned encoders.
```

The correctness boundary is P11–P12. The library-quality/TOML readiness boundary
is P13–P16.

---

## 3. Repository baseline and current facts

Planning was grounded against the live tree at:

```text
revision: 22cf74739a59fe178c0e3c1bb75c258a14327535
date:     2026-10-07
```

The working tree contained unrelated ongoing edits in the P4/R1 records and
`phalcom-semantic` checker, declaration, dispatch, fingerprint, trait, and test
files. Future execution must preserve those files unless a task explicitly owns
them. The proposal does not authorize cleanup, reset, restore, or broad format
changes.

Verified current facts:

1. `Char` does not exist as a live language/runtime type. There is no current
   `ValueTag`, AST literal, semantic core declaration, native metadata identity,
   or Universe class for it.
2. The lexer currently rejects a bare apostrophe. Double-quoted strings are
   handled by `phalcom-ast/src/lexer.rs`; character literals need a dedicated
   token and lexical path.
3. `String` is immutable valid UTF-8, but its public `size`, `slice`, `bytes`,
   and raw operations are byte-oriented. `codePointAt` returns `Int`.
   `string_raw_slice` already checks bounds and both UTF-8 boundaries before
   allocating. The required slicing work adds typed failures and explicit names;
   it does not repair a currently unchecked String slice.
4. `StringCodePointSequence.size` counts code points while `at(index)` treats
   the argument as a byte offset. This is not a coherent random-access sequence
   contract and must not be the canonical character API.
5. `packages/parse/src/source/text-source.ph` uses `Token = Int` bytes and
   byte-addressed offsets. `packages/parse/src/text.ph` exposes text operations
   as raw-byte `Int` predicates and `Text.char(String)`.
6. `List.toList` preserves identity rather than taking a snapshot. `ListSource`
   currently performs a hand-written defensive copy. No generic immutable
   homogeneous indexed sequence exists in the targeted Universe source.
7. `Int` is arbitrary precision with immediate `i64` and heap `LargeInt` forms.
   `Float` is `f64`. Existing constructors have numeric text behavior but do not
   expose the proposed typed `Result` APIs.
8. `String.+` allocates immutable strings. Existing `BytesWriter` is a byte
   accumulator, not a guest-visible text builder.
9. Canonical iteration contracts live in
   `phalcom-core/core/universe/src/collections/iterator_laws.ph`; associated
   `Item` and `Iterator` relationships must be preserved.

Primary entry points:

```text
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/scalar/number.ph
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/src/heap/string.rs
phalcom-core/src/heap/list.rs
phalcom-core/src/primitive/string.rs
phalcom-core/src/primitive/int.rs
phalcom-core/src/primitive/float.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/native/install.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface-gen/src/main.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/materialization.rs
phalcom-core/src/compiler/lib/expr.rs
packages/parse/src/source/text-source.ph
packages/parse/src/source/list-source.ph
packages/parse/src/text.ph
```

---

## 4. Normative authorities

- `docs/specs/README.md` — specification ownership and precedence.
- `docs/specs/objects/values.md` — every observable value is a language object;
  objecthood does not require heap allocation.
- `docs/specs/vm/values-and-objects.md` — value representation and exhaustive
  obligations for new immediate/heap forms.
- `docs/specs/universe.md` — one canonical Universe source identity; native
  metadata enriches declarations and is not a second declaration database.
- `docs/specs/static-evaluation.md` — static values are distinct from runtime
  `Value`; any static `Char` representation must follow that boundary.
- `docs/specs/associated-types.md` — trait-owned associated declarations and
  conformance evidence.
- `docs/specs/structural-products.md` — Tuple/product semantics; Tuple is not a
  replacement for a variable-length homogeneous sequence.
- `docs/specs/vm/products.md` — product representation if Char or snapshots cross
  product/materialization boundaries.
- `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md` — shared C4
  state and lifecycle authority.
- `packages/parse/MAIN.md` and `packages/parse/README.md` — parser ownership,
  typed rules, and the current package test harness.

No current authoritative specification defines `Char` or single-quoted
character literals. P11 must create or amend the owning normative specification
before implementation. It must update `docs/specs/README.md` if a new semantic
owner is introduced. The implementer must verify whether a newer text/value
specification exists first; a wiki, plan, fixture, or native implementation
cannot be the sole authority.

---

## 5. Architecture and ownership

### 5.1 `Char`

**FIXED:** `Char` is a distinct immediate scalar with a canonical `Char` class.
It represents exactly one Unicode scalar:

```text
U+0000..U+D7FF
U+E000..U+10FFFF
```

Surrogates and values above `U+10FFFF` are invalid. `Char` is not an `Int`
alias, a one-character `String`, or a heap allocation per decoded character.

The minimum typed surface is:

```telos
Char.codePoint -> Int
Char.utf8Length -> Int
Char.toString -> String
Char.fromCodePoint(_ value: Int) -> Option<Char>  // class-side
Char.isAscii -> Bool
Char.isAsciiDigit -> Bool
Char.isAsciiAlphabetic -> Bool
Char.isAsciiLowercase -> Bool
Char.isAsciiUppercase -> Bool
Char.isAsciiAlphanumeric -> Bool
Char.isAsciiHexDigit -> Bool
Char.isWhitespace -> Bool
Char.isControl -> Bool
```

ASCII predicates use their named ASCII ranges. `isWhitespace` uses Unicode
White_Space; `isControl` uses Unicode general category Cc. P11 records the
Unicode data version supplied by the pinned Rust toolchain in the normative
contract; future table changes require an explicit version update. `Char` is
prelude-visible, derives directly from Object, compares by scalar value, and
has scalar-value exactness and consistent hashing. There is no numeric coercion
or arithmetic on Char. Equal repeated literals require no interning or heap
identity.

P11 must update every exhaustive value seam:

```text
ValueTag and constructors/accessors
Value::class and concrete Char class mapping
ordinary equality, exact sameness, and hash
rendering/debugging/serialization
Option wrapping and GC edge extraction where applicable
product/materialization handling where applicable
native metadata and Universe class installation
```

The exact tag encoding is **VERIFY-FIRST**. If the current two-word value layout
cannot admit the representation without violating the VM specification, stop
and consult before selecting a heap fallback or changing the value contract.

### 5.2 Character literals

**FIXED:** single-quoted literals produce `Char`:

```telos
'a'  '0'  '_'  '['
'\n'  '\r'  '\t'  '\\'  '\''
'\u{00E9}'  '\u{1F642}'
```

After escape decoding, a literal has exactly one Unicode scalar. `\u{...}` has
one through six hexadecimal digits. Empty/multiple scalars, raw CR/LF, unknown
or incomplete escapes, surrogates, and out-of-range values are lexical errors
with source ranges. Literal scanning participates in nested interpolation
lexical modes so an escaped quote or brace cannot terminate the surrounding
mode. Combining sequences remain multiple scalars and are rejected in one
literal. NUL is valid through `'\u{0}'`.

### 5.3 Bytes and characters

**FIXED:** `TextSource` and parser spans remain byte-addressed. Text combinators
decode a `Char` at the current byte offset and advance by that scalar's UTF-8
width:

```text
String UTF-8 bytes
    -> TextSource byte offset
    -> decodeCharAt(offset)
    -> Char + nextByteOffset
    -> Char predicate
    -> next byte boundary
```

`Source.at` remains a byte-token operation on TextSource and its associated
Token remains Int. `Cursor.advance` remains the generic one-token increment.
Text combinators call `source.text.decodeCharAt(cursor.offset)` separately and
use `advanceBy(nextByteOffset - cursor.offset)`. They must never replace
`Source.at` with Char decoding or use generic single-token advancement for a
decoded scalar.

At EOF, `decodeCharAt` returns None and ordinary text rules fail uncommitted.
A negative, out-of-range, or continuation-byte position introduced through
generic byte rules or a manually constructed cursor is an invariant error in
the text layer, not EOF or a character mismatch. TextSource's generic byte lane
remains available, but mixing it with text rules requires restoring a valid
boundary first. Rules producing text spans validate both endpoints.

### 5.4 String validity and iteration

**FIXED:** every public `String` operation preserves valid UTF-8. Byte-range
slicing validates both endpoints or returns a typed failure. `String.chars` is an
iterable of `Char` without random indexing. `decodeCharAt(byteOffset)` is the
explicit byte-coordinate operation. The old code-point view may remain only as a
clearly documented compatibility API; it is not the canonical character sequence.

`decodeCharAt` returns None for negative, continuation, EOF, and out-of-range
offsets; valid String input never needs replacement-character decoding.
`isCharBoundary` is true at zero and byteCount, false outside the range or at a
continuation byte. `sliceBytes` rejects negative/reversed/out-of-range ranges
and invalid endpoints; empty slices at valid boundaries succeed. Error
categories distinguish invalid range, start boundary, and end boundary and
retain the requested offsets. Legacy `slice` delegates to the checked operation
and retains its throwing contract. No new unchecked public slice is added.

Legacy `codePoints` retains Int values through a projection of canonical Char
decoding. Its `at(index)` is migrated to ordinal scalar indexing with documented
O(n) cost; byte-position callers use `codePointAt(byteOffset)` or the new decode
operation. This removes the contradictory size/at contract. `leadByteLen` rejects
continuation positions. Compatibility fixtures are migrated at the same gate.

`StringChars` exposes iterable/iterator conformance, without indexed `at` or a
constant-time size claim. An explicit count, if provided, is O(n). Iterator
exhaustion is stable. `String.fromChars<I>` consumes a finite iterable with
`I |= Iterable<Item = Char>` and returns String; it uses the builder introduced
in S6 rather than adding a second encoder.

### 5.5 Immutable sequence

**FIXED:** parser snapshots require owned structural immutability, not a
read-only alias. The initial representation is a dedicated homogeneous heap
sequence with `Value` storage, full GC tracing, and no runtime generic type token
unless a later accepted decision requires one.

Initial contract:

```telos
ImmutableSequence<T>
    size -> Int
    at(_ index: Int) -> T
    get(_ index: Int) -> Option<T>
    slice(_ start: Int, _ end: Int) -> Result<ImmutableSequence<T>, SequenceRangeError>
    iterator -> ImmutableSequenceIterator<T>
    toList -> List<T>
```

`slice` is an owned copy for the first implementation. `toList` is a fresh
mutable materialization. Elements are shallow values; the sequence does not
deep-freeze referenced objects.

`at` is the checked indexed read required by Source consumers; an invalid
index raises the existing bounds exception category. `get` provides algebraic
absence. Negative indices are invalid, with no wrapping. Slice rejects negative,
reversed, or oversized bounds through Result and accepts valid empty ranges.
The initial sequence follows ordinary nominal class identity for equality,
exactness, and hashing. It does not invent structural hashing of mutable
elements. Distinct snapshots can contain equal values without being identical.
This is a stable-membership collection, not a new transparent structural product.

`from<I>` accepts a finite typed iterable with Item = T and owns a new shallow
snapshot; construction from an already immutable instance may reuse it.
`List.copy -> List<T>` always creates independent mutable membership/storage;
`List.snapshot -> ImmutableSequence<T>` creates an owned immutable snapshot.
`List.toList` retains its existing identity contract. Snapshot construction is
O(n), size and indexed read O(1), copied slice O(k).

### 5.6 Numeric conversion and builder

**FIXED:** format libraries own lexical grammar. `Int.parse` and `Float.parse`
convert validated text through typed APIs. TOML retains responsibility for
underscores, leading-zero rules, signs, radix prefixes, and its special-value
policy.

The new parse APIs consume the full string and never trim whitespace. Integer
parsing accepts an optional ASCII sign and at least one ASCII digit; radix is
2..36, default 10, with ASCII letters representing digits 10..35 in either case.
It rejects separators, radix prefixes, whitespace, and trailing data. Leading
zeros are accepted by stdlib; TOML decides whether they are legal. Invalid radix
takes precedence, then empty input, missing digits after sign, and the first
invalid digit. Error offsets are UTF-8 byte offsets into the original text;
invalid digit errors retain Char. Arbitrary precision means there is no semantic
Int overflow at i64 limits. Tests must prove promotion and values on both sides
of those limits rather than impose a fixed-width bound. Allocation exhaustion
follows existing resource/error policy, not an invented numeric Overflow result.

Float parsing accepts signed decimal digits with an optional dot and exponent
`e`/`E` with optional sign; at least one mantissa digit is required, and an
exponent requires digits. `.5`, `1.`, and integer-looking floats are accepted.
Whitespace, separators, hex floats, prefixes, and trailing data are rejected.
Exact special spellings are `inf`, `+inf`, `-inf`, `nan`, `+nan`, `-nan`;
these explicitly return IEEE special values. Decimal conversion that overflows
to infinity returns Overflow. Underflow rounds to a subnormal or signed zero;
signed zero is preserved. Use the existing host f64 conversion engine for
correct rounding after validating this grammar. Errors are Empty, InvalidSyntax
with first failing byte offset (EOF allowed), and Overflow. Do not catch every
legacy constructor exception and convert it into an undifferentiated error.

`StringBuilder` is mutable internal state producing immutable `String` values.
It supports amortized append of `String` and `Char` and never exposes mutable
UTF-8 storage.

Telos selectors are identified by name and labels, so two type-selected
`append(_)` overloads are invalid. The contract uses `append(_ text: String)`
and `appendChar(_ char: Char)`. `build` returns an immutable snapshot and leaves
the builder usable with its current contents; later appends cannot mutate any
earlier result. `size` and `byteCount` both report UTF-8 bytes. Construction is
amortized linear in total appended bytes; each build may copy O(n) bytes.

### 5.7 Ownership table

| Fact | Owner | Forbidden duplicate |
|---|---|---|
| Unicode scalar validity | `Char` core/native implementation | parser scalar validation |
| Character literal grammar | lexer/parser and language spec | `String` or `parse` workaround |
| Char identity | canonical Universe declaration | compiler-only builtin type |
| Immediate representation | VM `Value` layer | parser integer encoding |
| UTF-8 storage/decoding/boundaries | `String` | `parse.text` decoder |
| Char iteration | `StringChars`/iterator | incoherent byte-offset sequence |
| String prefix/search/split | `String` | consumer-local copies |
| Immutable membership/order | `ImmutableSequence<T>` | `ListSource` copy loop |
| Numeric conversion | `Int`/`Float` | TOML conversion engine |
| Text construction | `StringBuilder` | repeated parser concatenation |
| Cursor/failure/commit/span | `parse` | String/stdlib parser behavior |
| Format grammar | future `toml` | generic numeric parser |

---

## 6. Fixed, flexible, and verify-first decisions

### FIXED

- Dedicated immediate `Char`, distinct from `Int`.
- Unicode scalar validity excludes surrogates.
- Character literals have `Char` type.
- UTF-8 strings remain valid and immutable.
- Byte offsets remain canonical parser/source coordinates.
- Character iteration yields `Char` and does not claim O(1) scalar indexing.
- New absence/failure surfaces use `Option`/`Result`.
- `TextSource` may retain `Token = Int` bytes until a separate `Byte` plan.
- Immutable snapshot means owned structural immutability.
- Static values remain semantic values, not runtime `Value` or `ObjRef` substitutes.
- Char literals are admitted in the existing closed static-value domain with
  semantic scalar equality/hash and materialization descriptors. Native Char
  method execution is not implicitly allowed in static evaluation.
- `parse` and TOML do not own Unicode encoding, UTF-8 validation, or numeric
  conversion engines.

### MECHANICALLY FLEXIBLE

- Concrete `Char` tag and private Rust helpers.
- Existing scalar module versus a new text/value source module.
- `DecodedChar` field names and private iterator representation.
- Native versus Telos classification implementation.
- `ImmutableSequence` versus `ImmutableArray` surface name, once selected.
- Typed error variant names that preserve categories and offsets.
- Builder backing as `Vec<u8>` or validated `String`.
- Private file decomposition.

### VERIFY-FIRST

- Value-tag capacity and all exhaustive value seams.
- Lexer escapes and interpolation modes.
- Canonical Universe source/catalog generation.
- Static-value/materialization changes for a new scalar.
- Generic sequence representation and runtime type-token requirements.
- Numeric scanner grammar and current error categories.
- A newer normative String specification, if one exists.
- Exact test selectors at implementation time.

If verification changes ownership, representation, semantics, or a cross-layer
contract, stop and consult under the repository escalation protocol.

---

## 7. Proposed execution plans

Each slice becomes a separate execution plan with its own walkthrough and
handoff. The following tasks are the required implementation shape.

### S1 — Proposed `LANG007.C4.P11`: Char scalar and character literals

**Purpose:** Establish the language/runtime identity required by all later text
work.

**Files/symbols to verify and likely modify:**

```text
authoritative Char/literal specification selected by the authority audit
docs/specs/README.md, if a new owner is created
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-semantic/src/canonical_baseline.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/materialization.rs
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/core/universe/src/scalar/package.ph
phalcom-native-meta/src/universe.rs
phalcom-native-surface-gen/ authoring/generation path
```

**Tasks:**

1. Write failing lexer/parser tests for ordinary, escaped, Unicode, empty,
   multi-scalar, surrogate, and out-of-range literals.
2. Add the normative scalar/literal contract before implementation semantics are
   encoded in tests.
3. Add a dedicated token and AST expression. Do not use `Expr::String`.
4. Add semantic literal identity, static-value interning/fingerprints, value-domain
   typing, persistence seams, and materialization support. Include Char literal
   patterns, type diagnostics, source indexing, and editor literal presentation.
   Char does not extend FieldKey or Symbol-only call/product labels.
5. Add the immediate runtime representation and canonical `Char` class mapping.
6. Add the complete minimum Char surface in section 5.1 and test each predicate,
   scalar construction, and String round-trip for ASCII/2/3/4-byte scalars.
7. Regenerate native surface artifacts from authoring declarations.
8. Add scalar round-trip and runtime fixture coverage.

**Forbidden:** `Char` as an `Int` alias; heap allocation per decoded character;
parser-only pseudo-types; unchecked invalid scalar values; a second Universe
declaration registry; arbitrary native execution during static evaluation.

**Focused verification:** confirm selectors with `-- --list`, then run the
narrowest applicable targets from:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast lexer
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast parser
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::expression_analysis
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
```

**Acceptance:** `Char` has distinct class and semantic identity; scalar validity,
literal parsing, lowering, and affected semantic/runtime targets pass.

**Execution state (2026-10-07):** The numbered P11 execution plan is active and
supersedes this proposal's S1 as the implementation contract. Its current code
is committed as `800e1793`; P11 remains partial and unverified because no tests
or crate checks have run. See the P11 interim walkthrough, handoff, and INC-6
record linked above. This proposal remains active for P12–P16 and does not mark
S1 accepted.

### S2 — Proposed `LANG007.C4.P12`: String–Char bridge and UTF-8 boundaries

**Purpose:** Make `String` the safe bridge between Unicode scalars and byte
coordinates, then migrate `parse.text`.

**Files/symbols:**

```text
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/src/primitive/string.rs
phalcom-core/src/heap/string.rs
packages/parse/src/source/text-source.ph
packages/parse/src/text.ph
packages/parse/src/core/cursor.ph
packages/parse/src/core/span.ph
packages/parse/README.md
packages/parse/tests/runtime/src/clients/
packages/parse/tests/runtime/src/core/
packages/parse/src/ and all affected runtime fixture call sites
phalcom-core/tests/fixtures/language/strings/
```

**API direction:**

```telos
data DecodedChar(char: Char, nextByteOffset: Int)
String.byteCount -> Int
String.decodeCharAt(_ byteOffset: Int) -> Option<DecodedChar>
String.isCharBoundary(_ byteOffset: Int) -> Bool
String.sliceBytes(_ start: Int, _ end: Int) -> Result<String, StringBoundaryError>
String.chars -> StringChars
```

**Tasks:**

1. Test `"aé中🙂z"`: byte count, each lead-byte decode, exact next offsets,
   continuation-byte rejection, and five-element Char iteration.
2. Keep the existing valid-UTF-8 String storage invariant. Malformed-byte cases
   are tested only at a raw byte-to-text construction/decoder boundary that
   actually admits them. Do not manufacture invalid String objects merely to
   test decodeCharAt, or build a second UTF-8 engine over Rust String storage.
3. Add one combined decode-and-advance primitive so parse does not decode twice.
4. Test boundaries at `0`, `byteCount`, negative offsets, continuation offsets,
   and out-of-range offsets.
5. Replace the incoherent code-point random-access view with `String.chars` and
   a Char iterator satisfying the existing associated-type laws.
6. Make public byte slicing validate endpoints; retain trusted native slicing only
   behind a non-public invariant.
7. Add boundary validation to the text-rule entry points without changing
   TextSource's byte Source contract. Use the separate decoding/advanceBy path
   fixed in section 5.3.
8. Change `Text.char`, `satisfy`, `oneOf`, `noneOf`, `takeWhile`, and `takeWhile1`
   to use/produce `Char`; keep `Text.literal` byte-efficient.
   Cut over asciiDigit/asciiLetter/hexDigit and introduce scalar whitespace in
   this slice. `whitespace` uses Char.isWhitespace; preserve an explicitly
   named ASCII-whitespace rule if compatibility needs its narrower policy.
   Migrate all library and core/client fixture calls from Text.char(String),
   byte arithmetic, and Int-typed alternatives in the same slice. Use
   `char.codePoint - '0'.codePoint` only where a numeric digit value is required.
9. Make `TextSource.lineColAt` count Unicode scalars per line while retaining
   byte offsets; define LF/CRLF policy.
10. Add parse fixtures for `é`, `中`, `🙂`, mixed sequences, and exact spans.
    Preserve existing commitment, attempt/lookahead, failure-offset, and
    repetition-progress behavior when consuming multibyte scalars. Text.literal
    validates the starting boundary, permits the empty literal, and advances
    by literal.byteCount on success. No byte of a mismatching literal is consumed.

**Forbidden:** character-indexed parser spans; arbitrary Int characters;
continuation-byte replacement characters; malformed public String values; a
second parser-local UTF-8 decoder.

**Focused verification:**

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

**Acceptance:** String values remain valid UTF-8; `String.chars` yields Char;
text parsing advances by scalar width; parse spans remain byte offsets.

### S3 — Proposed `LANG007.C4.P13`: Typed String hygiene and positions

**Purpose:** Remove avoidable type erasure and local prefix/search workarounds.

**Files:**

```text
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/src/primitive/string.rs
packages/parse/src/text.ph
packages/parse/README.md
phalcom-core/tests/fixtures/language/strings/
```

**API direction:**

```telos
String.startsWith(_ prefix: String) -> Bool
String.endsWith(_ suffix: String) -> Bool
String.startsWith(_ prefix: String, atByte offset: Int) -> Bool
String.indexOf(_ needle: String, fromByte offset: Int) -> Option<Int>
String.split(_ delimiter: String) -> List<String>
```

Offset prefix matching returns false for invalid offsets and accepts an empty
prefix at every valid boundary including EOF. Offset search returns None for
invalid offsets, returns that offset for an empty needle at a valid boundary,
and otherwise returns an absolute byte offset. Legacy indexOf keeps its existing
sentinel/empty-needle contract; it is not silently changed by the new method.
Split retains the current nonempty-delimiter rule and preserves leading,
trailing, and adjacent empty fields.

**Tasks:**

1. Test prefix/suffix and offset-prefix behavior for ASCII and multibyte input.
2. Implement allocation-free prefix matching for `Text.literal`.
3. Add offset search without repeated temporary suffix allocation.
4. Make `split` preserve `String` element typing and test typed iteration.
   Audit and annotate deterministic existing String results: toString, trim,
   trimStart, trimEnd, replace and repetition return String; size/isEmpty return
   Int/Bool; contains/includes return Bool. StringByteSequence and iterator keep
   Item = Int, and an out-of-bounds byte read is explicitly Option<Int>.
   Reconcile rawByteAt's current declared Int/actual None mismatch at the owning
   native declaration, generated surface, and source caller sites. Do not claim
   a typed gate while preserving that mismatch.
5. Make byte-oriented names explicit while retaining compatibility aliases only
   where migration is safe.
6. Update keyword boundaries to use Char classification and the prefix surface.
7. Keep line/column on TextSource for v1, using 1-based scalar columns. LF and
   CRLF each count as one line break; a lone CR is ordinary content. The byte
   boundary between CR and LF is on the preceding line. Invalid offsets raise
   a source-coordinate invariant error rather than being clamped. Display-cell
   width, tabs expanded to cells, and other Unicode line separators are deferred.

**Forbidden:** byte columns presented as scalar columns, substring allocation
for prefix checks, new sentinel APIs, erased String result types, or untyped
native absence disguised as Int.

**Focused verification:** add typed source cases to the existing semantic
integration harness and runtime cases to String corpus/parse fixtures. Inspect
`phalcom-core/tests/README.md` and semantic test placement guidance before
registration. Commands for the planned gate are:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --check
```

Add exact semantic fixture filters when authoring P13. Allocation-free matching
is verified by the implementation call path/allocator instrumentation where
available; a timing assertion alone does not prove no temporary substring.

**Acceptance:** consumers no longer need local prefix/suffix/search operations;
`split` is typed; `Text.literal` avoids substring allocation; columns count
Unicode scalars.

### S4 — Proposed `LANG007.C4.P14`: Immutable indexed snapshots

#### Lifecycle reconciliation — 2026-10-11

The contract below records the original proposal; it is not evidence that a
dedicated `ImmutableSequence<T>` runtime type was implemented. The completed C4
parser behavior copied caller membership into a private `List<T>` inside
`ListSource<T>`. The zero-copy `Substring` work is a separate P12 text view.

COLL006.C1.P5 is replacing that parser-specific copy loop with an owned
`Array<T>` snapshot retained by a read-only `Slice<T>`. The accepted C1 `Vector`
has a static dimension and is not the representation for runtime-sized parser
snapshots. This convergence adds no `ImmutableSequence<T>` type and does not
retroactively change the earlier implementation history. `ParseError.expected`
and `.contexts` remain `List<String>` in the current package and are outside
this P5 migration.

**Purpose:** Replace parser-owned list copying and mutable error-value exposure.

**Files/symbols:**

```text
Universe collection source for ImmutableSequence<T> or approved name
phalcom-core/src/heap/mod.rs and owning collection object
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/repr.rs and equality/hash seams
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface-gen/ authoring/generation path
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/list.ph
packages/parse/src/source/list-source.ph
packages/parse/src/core/error.ph
```

**Tasks:**

1. Specify the section 5.5 snapshot/bounds/identity contract normatively before
   implementing it. Use owned immutable boxed Value storage and existing nominal
   runtime generic behavior; any stronger reification requirement is escalation.
2. Test source mutation after snapshot, missing mutation methods, typed access,
   and iterator stability.
3. Add immutable storage, allocation/accessors, runtime class, tracing, rendering,
   and equality/hash policy.
4. Add `Iterator::Item = T` and `Iterable::Item = T` conformance.
5. Add explicit snapshot construction and fresh mutable `toList`.
6. Migrate `ListSource` to immutable backing and remove its custom copy loop.
7. Migrate `ParseError.expected` and `.contexts` to immutable typed storage;
   retain defensive copies until the new surface exists.
8. Test GC/rooting with sequences containing object references.

**Forbidden:** readonly aliases presented as snapshots; Tuple as variable-length
storage; runtime generic tokens without an accepted reification decision;
`List<Dynamic>` where `T` is known; deep-freezing elements.

**Focused verification:**

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-semantic
```

**Acceptance:** snapshot membership/order is stable, iterator associated types
are preserved, GC safety is proven, ListSource is immutable, and parser error
collections cannot be externally mutated.

### S5 — Proposed `LANG007.C4.P15`: Typed numeric parsing

**Purpose:** Give format libraries typed numeric conversion without moving their
lexical grammar into the standard library.

**Files:**

```text
phalcom-core/core/universe/src/scalar/number.ph
phalcom-core/src/primitive/int.rs
phalcom-core/src/primitive/float.rs
phalcom-core/src/error.rs
phalcom-core/tests/core/language/numeric_values.rs
phalcom-core/tests/core/language/numeric_diagnostics.rs
phalcom-core/tests/core/audit_probes/numeric_contracts.rs
```

**API direction:**

```telos
Int.parse(_ text: String) -> Result<Int, IntParseError>
Int.parse(_ text: String, radix: Int) -> Result<Int, IntParseError>
Float.parse(_ text: String) -> Result<Float, FloatParseError>
```

**Tasks:**

1. Audit `Int.new(String)` and `Float.new(String)` scanner behavior and reuse
   only the conversion logic whose grammar matches the typed contract.
2. Test zero, signs, i64 boundaries, arbitrary-precision promotion, invalid
   digits, radix 2/8/10/16, and invalid radix.
3. Test fractions, exponents, large/small values, invalid syntax, rounding,
   overflow, underflow/signed zero, and section 5.6 NaN/infinity policy.
4. Implement typed APIs without `Dynamic` conversion or stringify/reparse loops.
5. Document the TOML boundary: TOML validates underscores, leading zeros,
   prefixes, signs, and special values before conversion.
6. Add a parse/TOML-facing conversion fixture after core numeric tests pass.

**Forbidden:** arbitrary-width conversion in TOML; changing committed numeric
representation; silently adopting TOML grammar; new sentinel failures.

**Focused verification:**

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core numeric_diagnostics
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core numeric_values
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::numeric_contracts
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

**Acceptance:** typed numeric conversions cover limits/errors and preserve the
committed numeric model; TOML-specific lexical policy remains in TOML.

### S6 — Proposed `LANG007.C4.P16`: Builder and parse/TOML closure

**Purpose:** Finish the substrate and prove that `parse` is a thin text-parsing
layer.

**Files:**

```text
Universe source for StringBuilder
phalcom-core/src/heap/ builder object and tracing
phalcom-core/src/primitive/string.rs or dedicated builder primitive
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface-gen/ authoring/generation path
phalcom-core/core/universe/src/scalar/string.ph
packages/parse/src/text.ph
packages/parse/src/source/text-source.ph
packages/parse/src/core/error.ph
packages/parse/tests/runtime/src/clients/toml-test.ph
packages/parse/tests/runtime/src/clients/
packages/parse/README.md
packages/parse/MAIN.md
```

**API direction:**

```telos
StringBuilder.new() -> StringBuilder
StringBuilder.append(_ text: String) -> Unit
StringBuilder.appendChar(_ char: Char) -> Unit
StringBuilder.size -> Int
StringBuilder.byteCount -> Int
StringBuilder.build -> String
```

**Tasks:**

1. Test empty output, many appends, mixed String/Char appends, multibyte scalars,
   output immutability, and large accumulation.
2. Implement rooted mutable storage and immutable snapshot build using section
   5.6 semantics. Add String.fromChars on top of canonical Char encoding and this
   builder, preserving the Iterable Item = Char constraint.
3. Migrate repeated concatenation in parser diagnostics and transformations
   where the builder materially improves construction.
4. Migrate TOML-oriented integer/text accumulation only after the Char API is
   complete; do not implement TOML grammar here.
5. Finish documentation and consumer examples already cut over in S2; do not
   defer compiling core/client fixture migrations until this final slice.
6. Add TOML substrate proof cases for bare keys, horizontal whitespace, and
   basic-string characters.
7. Add parse regressions for `Text.char('é')`, `é` then `x`, `Text.oneOf("éö")`,
   and multiscalar `takeWhile` with exact byte spans.
8. Remove provisional byte-oriented public text API documentation. Internal byte
   helpers may remain for literal matching and source spans.
9. Update the C4 checkpoint and create walkthrough/handoff records for each slice.

**Focused verification:**

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-semantic
```

**Acceptance:** parse fixtures pass with Char APIs and byte spans; builder and
numeric facilities use typed public contracts; TOML-shaped examples require no
raw ASCII constants, UTF-8 decoder, Dynamic string lists, or parser-owned encoder.

---

## 8. Cross-slice invariants

1. **One scalar model:** public character operations use `Char`; consumers do
   not reinterpret arbitrary `Int` values as scalars.
2. **Two coordinate models:** byte offsets are explicit source coordinates;
   `Char` values are Unicode scalars. APIs name the distinction.
3. **Valid String:** no public operation constructs malformed UTF-8.
4. **Coherent iteration:** character iteration describes characters and does not
   expose byte-offset `at` as scalar indexing.
5. **Stable source:** parser snapshots own membership/order.
6. **Typed core:** deterministic String/collection results preserve known types;
   `Dynamic` does not hide known contracts.
7. **Semantic/runtime separation:** static Char values are
   semantic values, never runtime heap identity substitutes.
8. **Canonical Universe identity:** source declarations own identity; native
   metadata enriches them.
9. **TOML boundary:** TOML validates format grammar; stdlib converts validated
   values.
10. **Parser ownership:** `parse` owns cursor, backtracking, failure, and spans;
    it does not own UTF-8 validity or general text storage.

---

## 9. Non-goals

This follow-on excludes:

```text
Byte/UInt8 redesign as a prerequisite
immutable byte storage as a prerequisite
zero-copy StringSlice views
grapheme clusters, normalization, locale collation, and full case folding
regular expressions, parser generators, and lexer generators
streaming decoders, ropes, and incremental text buffers
async parser input and general serialization derivation
TOML grammar completion beyond substrate proof cases
general static type-system redesign
runtime generic type evidence for every collection instance
```

Byte/immutable-byte facilities and StringSlice remain P2 work unless a focused
slice proves they are required for correctness. Performance optimization beyond
the stated complexity contracts requires measured evidence and a plan amendment.

---

## 10. Failure modes and STOP/CONSULT triggers

| Failure | Required response |
|---|---|
| Char is encoded as ordinary Int | Stop; restore distinct identity |
| Decoder accepts continuation, overlong, surrogate, or truncated input | Fix at Char/String owner |
| String slice creates malformed UTF-8 | Fix boundary validation before parse migration |
| Character iterator exposes byte-offset indexing | Remove or relabel the incoherent API |
| TextSource changes spans to character indexes | Restore byte spans |
| New typed API returns `List<Dynamic>` | Preserve associated-type typing or consult |
| Immutable sequence is a readonly alias | Implement owned storage or stop |
| TOML implements numeric conversion | Move conversion to Int/Float owner |
| Native metadata becomes declaration authority | Restore canonical Universe ownership |
| New heap/value arm misses tracing/equality/rendering | Stop at runtime integration |
| Static evaluator calls arbitrary native behavior | Use approved semantic intrinsic boundary |
| P4 semantic files are touched for convenience | Re-partition or consult |
| Same nontrivial semantic failure survives one serious correction | Escalate |
| Scope expands into GC, Byte redesign, or evaluator redesign | Amend plan before editing |

An incident packet must contain observed facts, hypothesis, evidence, predicted
effect, discriminating test, architectural trigger, and requested decision.

---

## 11. Testing and verification budget

Applicable dimensions are scalar validity, literal grammar, malformed UTF-8,
byte/scalar coordinates, sequence aliasing, associated types, GC rooting,
numeric errors/overflow, builder growth, Unicode parser composition, and
diagnostic positions.

Use this ladder:

```text
T0 — exact new regression
T1 — slice-specific test
T2 — owning crate/module suite
T3 — parse package runtime integration
T4 — affected crate and native/semantic integration
T5 — C4/workspace certification
```

During BUILD, run T0/T1 and the narrowest informative T2 check. Run the parse
fixture gate at S2 and S6. Do not run workspace-wide tests, workspace clippy,
release gates, or broad unrelated semantic suites during BUILD unless a named
checkpoint gate or causal failure requires them. Enter CERTIFY mode only after
S6 and explicit checkpoint authorization.

Classify unexpected failures:

```text
A — definitely caused: fix.
B — probably caused: investigate and fix.
C — unclear: one bounded classification pass, then defer if nonblocking.
D — unrelated/baseline: record and continue.
```

---

## 12. Lifecycle and checkpoint bookkeeping

Before S1 execution:

1. Amend `LANG007.C4-CHECKPOINT.md` with the approved follow-on ledger and
   dependency statement, extending its bounded objective and ownership map.
2. Record approval and any architecture consultation.
3. Establish a live baseline revision and focused test result.

At each slice gate:

1. Update the checkpoint only for durable interfaces, invariants, meaningful
   verification evidence, consultations, or deferred failures.
2. Keep `IMPLEMENTED`, `FOCUSED_TESTED`, and `RELEASE_COMPLETE` distinct.
3. Preserve the current P4/R1 semantic issue classifications.

At slice completion:

1. Create the slice walkthrough recording what shipped and what actually ran.
2. Create the next-slice handoff with inherited interfaces and residual risks.
3. Update the plan ledger and next action in the checkpoint.

The follow-on may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when
all named focused gates pass. `RELEASE_COMPLETE` requires separately authorized
C4/workspace certification.

---

## 13. Final C4 follow-on gate

The following TOML-shaped expressions must compile and run through the canonical
stdlib/parse APIs:

```telos
let bareKey = Text.takeWhile1("bare key") |char| {
    char.isAsciiAlphabetic
        or char.isAsciiDigit
        or char == '_'
        or char == '-'
}

let horizontalWhitespace = Text.satisfy("whitespace") |char| {
    char == ' ' or char == '\t'
}

let basicStringChar = Text.satisfy("string character") |char| {
    char != '"'
        and char != '\\'
        and not char.isControl
}
```

The following regressions must pass:

```telos
Parse.run(Text.char('é'), TextSource.new("éx"))
Text.char('é').then(Text.char('x'))
Text.oneOf("éö")
Text.takeWhile |char| { char != 'x' }
```

Running the last parser over `"é中x"` returns `"é中"` and ends at the correct
UTF-8 byte offset. No test may pass by restoring a public byte/`Int` character
API.

These are target developer-experience examples. The current parse README records
that `and`/`or` in value position may produce Object rather than Bool. Fixtures
may use the existing `if predicate { true } else { false }` idiom until that
compiler issue is resolved. Record the deviation and owning issue; it does not
authorize Int predicates or a silent type-erasure fallback. Likewise, an
associated-type dispatch or data-method lowering blocker is recorded at its
owning layer rather than hidden inside stdlib implementations.

### Requirements-to-gates ledger

| User requirement | Owning slice | Required evidence |
|---|---|---|
| P0 Char and literals | S1 | `a`, `é`, `中`, `🙂`, escapes, invalid/surrogate/upper-bound rejection, distinct identity |
| P0 decode and byte/scalar separation | S2 | `aé中🙂z` offsets 0/1/3/6/10, next offsets 1/3/6/10/11; continuation and EOF rejection |
| P0 safe slice | S2 | each scalar width, invalid start/end, range failures, empty boundary slice |
| P1 classifications | S1 | all ASCII predicates, White_Space and Cc positive/negative representatives |
| P1 iteration | S2 | five Char items, typed Item, stable exhaustion, coherent legacy ordinal indexing |
| P1 typed String/prefix/search | S3 | typed split and deterministic signatures, multibyte and empty prefix/search policies |
| P1 snapshot/copy | S4 | source mutation isolation, List.copy independence, snapshot immutable structure and shallow elements |
| P1 numbers | S5 | Int radix/promotion/errors; Float correct rounding, overflow, underflow and specials |
| P1 construction/builder | S6 | Char/String append, fromChars typing, earlier builds unchanged after mutation |
| P1 line/column | S2, inherited by S3 | scalar columns and precise LF/CRLF/boundary policy |
| parse v1 migration | S2/S4/S6 | all existing fixtures plus Unicode span/commit/backtracking regressions |
| TOML sufficiency | S6 | bare-key/whitespace/string predicates and typed conversion without low-level workarounds |
| P2 Byte/bytes/views/case facilities | Deferred | no dependency introduced by P0/P1 work |

Each numbered plan must turn its rows into exact test registrations and G<n>
gates. Tests belong at the semantic identity/type layer, compiler projection
layer, or runtime/consumer layer that owns the assertion. A package fixture is
not sufficient proof of canonical semantic identity or incremental parity.

Complexity acceptance is O(1) scalar width and ASCII classification, at most
four bytes examined per decode, O(prefix byte length) prefix matching without a
substring allocation, O(1) snapshot size/indexed access, O(n) snapshot creation,
and amortized O(total bytes) builder appends. Arbitrary-precision Int conversion
must use the existing BigInt engine; do not promise fixed-width linear bit-cost
for unbounded integers. Float conversion is linear in text length through the
host conversion engine. Verify algorithm/call-path properties and allocator
instrumentation where useful; avoid timing thresholds as correctness tests.

S1's semantic acceptance includes literal pattern behavior, static-domain Char
identity, static content fingerprints, immutable publication/materialization,
and cold versus incremental agreement when editing Char constants. Schema
changes must follow the owning metadata/version contract; inventing a parallel
Char store or serializing VM tags as semantic identity is forbidden.

---

## 14. Residual risks and deferred work

Byte/`Byte` cleanup, zero-copy String slices, rich Unicode categories, runtime
generic type evidence, and the complete TOML grammar remain outside this
follow-on. Full workspace certification may encounter existing baseline failures.
Those failures must be recorded without weakening the Unicode, validity, typing,
or ownership invariants established here.
