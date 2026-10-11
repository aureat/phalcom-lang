# Text and String Model

**Status:** Normative platform specification

**Semantic owner:** UTF-8 text indexing, text views and operations, and the Universe text utility API

**Related specifications:** [Characters](objects/characters.md), [Iterators and Iterables](iterator.md), [Universe](universe.md), and [VM Values and Objects](vm/values-and-objects.md)

## 1. Scope and values

`String` is immutable text encoded as valid UTF-8. Its storage and byte offsets are distinct from Unicode scalar values: a byte is not a character, and a Unicode scalar is not necessarily a grapheme cluster. The [Characters specification](objects/characters.md) owns `Char` validity and scalar classifications. This document owns the rules that connect `Char` values to encoded text.

Text operations must preserve valid UTF-8. A `String` is compared by text contents. A `Substring` compares by the contents of its viewed range; equality between `String` and `Substring` is symmetric.

## 2. Byte offsets and `StringIndex`

`StringIndex` represents a non-negative UTF-8 byte offset. It is valid as an index into a particular text value when it is within the text's inclusive range from `startIndex` to `endIndex` and falls on a Unicode scalar boundary. A `StringIndex` is not a character ordinal and does not carry the identity of a particular string.

For `String` and `Substring`:

- `byteCount` is the number of encoded UTF-8 bytes;
- `size` is the legacy byte-count alias;
- `charCount` is the number of Unicode scalar values;
- `startIndex` is byte offset zero relative to the value;
- `endIndex` is the byte offset equal to `byteCount`;
- `first` and `last` return an optional `Char`;
- `index(after:)` and `index(before:)` move by one scalar and return `None` when there is no next/previous scalar.

Indexing by `StringIndex` decodes the scalar beginning at that boundary. `endIndex` is a valid position but has no scalar to subscript. An out-of-range index or a non-boundary offset is an indexing error. A slice range is half-open `[start, end)`; both endpoints must be in range and on scalar boundaries.

The `bytes` and `utf8` views expose encoded bytes as `Int` values. The `chars` view exposes Unicode scalar values as `Char`. These are separate iterable views; iterating bytes does not perform decoding and iterating characters does not yield byte integers.

## 3. `Text` and `Substring`

`Text` is the common trait implemented by `String` and `Substring`. It provides `byteCount`, `isEmpty`, `charCount`, `chars`, `utf8`, and `asSubstring`.

`String.slice(start, end)`, String range subscripting, and `asSubstring` produce an immutable `Substring` view over a contiguous scalar-aligned range. A substring retains its source string and relative start/end byte offsets; slicing a substring produces another view over the same text. The view does not permit mutation of its source. Converting a substring to `String` produces a string with the same contents.

`Substring` supports byte counts, scalar counts, scalar indexing and iteration, UTF-8 byte iteration, and access to its underlying `string`, `startOffset`, and `endOffset`. Its offsets are relative for indexing and absolute only through the named source-offset properties.

## 4. Search and text patterns

`TextPattern` defines `findIn(source, offset)`, where `source` is a `Substring` and the offset is a byte offset relative to that substring. It returns `None` when no match exists and otherwise returns `(startByteOffset, endByteOffset)`, a half-open range relative to `source`. A returned match must begin and end on scalar boundaries. `Char`, `String`, and `Substring` conform to `TextPattern`.

The text API provides `contains`, `startsWith`, `endsWith`, `find`, `findLast`, and `findRange`. `find` and `findLast` return the start as a `StringIndex`; `findRange` returns the matching half-open range. `stripPrefix` and `stripSuffix` return the remaining `Substring` when the requested pattern matches at the corresponding edge, and `None` otherwise.

`trimmed`, `trimmedStart`, and `trimmedEnd` remove Unicode whitespace from the requested edge or edges and return `Substring` views. They use the `Char.isWhitespace` classification owned by the Characters specification.

`String.trim()`, `trimStart()`, and `trimEnd()` return new strings after removing the four ASCII whitespace characters space, tab, carriage return, and line feed. Their overloads taking a `String` remove any leading or trailing scalar that occurs in that string. These string-producing methods are distinct from the Unicode-whitespace `trimmed*` view methods.

`String.isAscii` is true exactly when every encoded byte is below `0x80`. `lowercase()` and `uppercase()` apply Unicode full case mapping and return a `String`; the result may contain a different number of scalars. `repeat(count)` repeats the complete string a non-negative integer number of times. `padStart(width)` and `padEnd(width)` measure the deficit in Unicode scalars and use a space by default. Their `with` overloads repeat the supplied fill value's string form once per missing scalar; a fill value with multiple scalars may therefore make the result wider than `width`. Padding does not truncate a string already at least `width` scalars long.

`String.repr` and `Char.repr` provide quoted source-style representations. Their escaping must remain consistent with the corresponding literal grammar; representation output is not a byte serialization format. In particular, character NUL uses `\u{0}` because `\0` is not accepted by the character-literal grammar. The exact representation of NUL in a String must likewise remain accepted by the string-literal grammar.

## 5. Decomposition and replacement

`split(pattern)` returns a lazy iterable of `Substring` components separated by successive pattern matches. `splitOnce(pattern)` returns the two views around the first match or `None`. Empty components are retained when a separator match creates them. `splitWhitespace()` lazily separates runs of Unicode whitespace and does not yield empty components.

`lines()` returns a lazy iterable of lines without terminators. It recognizes LF, CR, and CRLF line endings; CRLF is one terminator. `lines(keepingTerminators: true)` includes each terminator in its line. Empty text yields no lines. A final unterminated segment is yielded as the final line; no extra empty line is yielded solely because the text ends with a terminator.

`replaceFirst(pattern, with:)` replaces only the first match. `replaceAll(pattern, with:)` replaces each non-overlapping match from left to right. Replacement must make forward progress for every supported pattern.

The compatibility operation `String.replace(needle, replacement)` replaces all
occurrences of a non-empty `String` needle with a `String` replacement. Unlike
the pattern-based operations, it rejects a needle of another type or an empty
needle.

The split, whitespace-split, and line sequences conform to the canonical `Iterable`/`Iterator` contracts. Their detailed traversal and exhaustion rules are owned by [Iterators and Iterables](iterator.md).

## 6. Text construction, parsing, and formatting

`String.fromUtf8(bytes)` validates a byte sequence and returns `Result<String, Utf8Error>`. It returns `Ok` only for valid UTF-8. `String.fromUtf8Lossy(bytes)` returns text with each invalid sequence replaced by U+FFFD. `Utf8Error.validUpTo` is the byte count of the valid prefix before the first invalid sequence; `errorLength` is the length of that invalid sequence when it can be determined, otherwise `None`.

The scalar parsing APIs return typed results:

```phalcom
Int.parse(text) -> Result<Int, IntParseError>
Int.parse(text, radix: radix) -> Result<Int, IntParseError>
Float.parse(text) -> Result<Float, FloatParseError>
Bool.parse(text) -> Result<Bool, BoolParseError>
```

Invalid input is represented by `Result::Error`, not by a fabricated scalar value. `Bool.parse` accepts exactly `"true"` or `"false"`. Integer radix formatting is exposed by `Int.toString(radix:)`; the supported radix range is 2 through 36. The numeric value and conversion rules remain owned by the numeric type specifications.

`TextWriter` is the output protocol with `write(String)` and `writeChar(Char)`. `StringBuilder` is a mutable text accumulator with `append`, `appendLine`, `clear`, `isEmpty`, `byteCount`, and `toString`; it conforms to `TextWriter`. Appending a `Char` encodes that scalar as UTF-8.

## 7. Scanning and source locations

`TextCursor` is a forward cursor over `String` or `Substring`. Its `offset` and `index` are byte positions. `peek()` observes the next scalar without advancing; `next()` consumes and returns the next scalar; `remaining` returns the unconsumed substring. `startsWith` observes a prefix and `consume` advances only when that prefix matches. `checkpoint` and `restore` save and restore the cursor's byte position.

`LineMap` maps a valid text byte position to a one-based `SourceLocation(line, column)`. Line breaks are LF, CR, and CRLF, with CRLF treated as one line break. Columns count Unicode scalar values from one, not UTF-8 bytes or grapheme clusters. A caller must supply a valid scalar-boundary position in the mapped text.

## 8. Ownership boundaries

This specification defines the Universe text contract, not physical string storage or native ABI. VM string representation belongs to [VM Values and Objects](vm/values-and-objects.md). Scalar identity, literal syntax, and Unicode property versions belong to [Characters](objects/characters.md). Numeric parsing results preserve the meanings of `Int`, `Float`, and `Bool`; text APIs do not redefine those value domains.
