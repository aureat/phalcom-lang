# Phalcom Language Specification — Characters

## Scope

This specification defines the source-language value and literal semantics of
`Char`. A `Char` is one Unicode scalar value. It is distinct from `Int` and
from a `String` containing one scalar. This specification owns scalar identity,
character literal syntax, and the minimum classification operations; UTF-8
storage, decoding, boundaries, and iteration over text are owned by the String
text substrate.

## Character values

A `Char` value denotes exactly one Unicode scalar value in these ranges:

```text
U+0000..U+D7FF
U+E000..U+10FFFF
```

Surrogate code points and integers outside those ranges do not denote a Char.
A Char is allocation-identity-free. Two Char values are exactly the same and
ordinarily equal if and only if they denote the same scalar value. Their hash
is consistent with ordinary equality. The physical representation and
materialization of Char do not alter these rules.

Char is a distinct value category and has canonical class identity `Char`,
directly under `Object`. It is not an alias of `Int`; there is no implicit
conversion, arithmetic, or numeric ordering between Char and Int. Explicit
conversion is through `codePoint` and `fromCodePoint(_)`.

## Character literals

A single-quoted literal denotes one Char:

```telos
'a'  '0'  '_'  '['
'\n'  '\r'  '\t'  '\\'  '\''
'\u{00E9}'  '\u{1F642}'  '\u{0}'
```

The literal body is decoded before its scalar count is checked. A plain body
must contain exactly one Unicode scalar. Supported escapes are `\n`, `\r`,
`\t`, `\\`, `\'`, and `\u{H...}`, where the braced form contains one through
six hexadecimal digits and denotes a valid Unicode scalar.

An empty body, multiple decoded scalars, raw carriage return or line feed,
unknown or incomplete escapes, malformed Unicode escapes, surrogate values,
and values above `U+10FFFF` are lexical errors. Each error retains the
half-open UTF-8 byte range of the offending literal or escape. Combining
sequences are multiple scalars; a grapheme cluster is not one Char unless it
contains exactly one scalar.

## Character literal patterns

A character literal may also appear as a pattern. It matches exactly the Char
with the same Unicode scalar value. Char is a closed scalar domain and is
non-subclassable for matching. A literal pattern is impossible when its subject
cannot contain a Char; for an unknown or unresolved subject, the checker must
retain uncertainty instead of declaring the pattern impossible.

Each literal pattern contributes its singleton scalar to match coverage.
Repeated literals are redundant once that scalar is already covered. A match
over Char is exhaustive when its patterns cover every valid Unicode scalar or
include a catch-all pattern. Surrogates are not part of the domain. Exact
matching compares the scalar value directly and does not invoke user-defined
equality.

## Classification surface

The canonical Char surface provides:

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
Char.hash -> Int
```

ASCII classifications use the corresponding named ASCII ranges. `isWhitespace`
uses the Unicode `White_Space` property, and `isControl` is true exactly for
Unicode General_Category `Cc`. These properties use Unicode 17.0.0 data, as
provided by the pinned `nightly-2026-07-10` Rust toolchain. Changing that
property-data version is an explicit Unicode-contract update.

`fromCodePoint(_)` returns `None` for a negative, out-of-range, or surrogate
integer, and `Some(char)` for every valid Unicode scalar value. `utf8Length` is
the scalar's UTF-8 encoded width from one through four bytes.

## Ownership boundary

`String` owns UTF-8 encoding/decoding and byte-boundary operations. Consumers
may use a Char's code point when a numeric scalar is explicitly needed, but may
not reinterpret arbitrary Int values as characters. Character iteration over
String is defined by the String text substrate and yields Char values.
