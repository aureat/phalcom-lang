---
id: LANG007.C4.P12-P16
kind: implementation-walkthrough
title: "LANG007.C4.P12–P16 Telos Text Model Substrate Walkthrough"
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
recorded_against_revision: 11084cd9630ee77a2886f7847c210ca0a6042b47
recorded: 2026-10-08
branch: main
---

# LANG007.C4.P12–P16 — Telos Text Model Substrate Walkthrough

## 1. Overview and Objectives

This walkthrough documents the design, implementation, verification, and main-branch consolidation of the complete Telos Text Model standard library substrate (`LANG007.C4.P12` through `LANG007.C4.P16`).

The implementation establishes a comprehensive text substrate across five core layers:
1. **Layer 1 (Scalar & Indexation):** `StringIndex`, byte/char length metrics, UTF-8 scalar boundaries, Unicode case mapping, `Char.repr`, and `String.repr`.
2. **Layer 2 (Zero-Copy Views):** `Substring` non-allocating range view, range slicing (`text[0..5]`, `text.slice(start, end)`), symmetric string/substring equality.
3. **Layer 3 (Abstractions & Search):** `Text` and `TextPattern` traits, multi-type search (`contains`, `startsWith`, `endsWith`, `find`, `findLast`, `findRange`), zero-copy trimming (`trimmed`, `trimmedStart`, `trimmedEnd`), and prefix/suffix stripping (`stripPrefix`, `stripSuffix`).
4. **Layer 4 (Decomposition & Tooling):** Lazy `Split`, `SplitWhitespace`, and `Lines` iterators conforming to `CanonicalIterable`/`CanonicalIterator`, `splitOnce`, `replaceFirst`, `replaceAll`, `TextWriter` trait, `StringBuilder` mutable buffer with linear-time native `rawJoin`, `TextCursor` sequential scanner, and `LineMap` (O(log L) coordinate indexing with `SourceLocation`).
5. **Layer 5 (Conversions & Parsing):** Text error types (`Utf8Error`, `IntParseError`, `FloatParseError`, `BoolParseError`), `String.fromUtf8` / `String.fromUtf8Lossy`, typed target parsing (`Int.parse`, `Float.parse`, `Bool.parse`), and radix formatting (`Int.toString(radix:)` for base 2..36).

---

## 2. Completed Work by Task

### Task 1: Test Harness & Contract Alignment (`05377561`)
- Aligned `contracts.rs` with the live `Option<Int>` contract of `String.rawByteAt(_)`.
- Updated mutable accumulator variable `let mut byteSum = 0` in `canonical_bytes_and_string_views_iterate_their_stateful_surfaces` in `iteration.rs` to satisfy semantic immutability enforcement.

### Task 2: StringIndex, Counts, Casing, and Scalar Indexing (`b80af4cd`)
- Added `scalar/string_index.ph` declaring `class StringIndex(byteOffset: Int, charOffset: Int)`.
- Added `byteCount`, `charCount`, `isCharBoundary(byteOffset:)`, `indexAt(charOffset:)`, `scalarIndex(byteOffset:)`, and `indexOffset(index, offsetBy:)` on `String`.
- Added `simpleLowercase` and `simpleUppercase` on `Char` and `String`.
- Implemented `Char.repr` and `String.repr` for escape inspection.

### Task 3: Zero-Copy Views & Range Slicing (`649e9cac`)
- Created `text/substring.ph` defining `class Substring(string: String, startByte: Int, endByte: Int)`.
- Implemented range slicing `slice(start, end)` on `String` returning `Substring`.
- Implemented `text[start..end]` subscript overload via `slice(range.start, range.end)`.
- Added symmetric equality between `String` and `Substring`.

### Task 4: Text and TextPattern Protocols (`60fd8c51`)
- Created `text/traits.ph` defining `trait Text` with associated types and core inspection methods.
- Created `text/patterns.ph` defining `trait TextPattern` with `matchAt(text, start) -> Option<Int>`.
- Conformed `Char`, `String`, and `Substring` to `Text` and `TextPattern`.

### Task 5: Search, Trim, and Strip Operations (`958b71bd`)
- Implemented pattern search methods (`contains`, `startsWith`, `endsWith`, `find`, `findLast`, `findRange`) accepting any `P: TextPattern`.
- Implemented zero-copy trimming (`trimmed`, `trimmedStart`, `trimmedEnd`) returning `Substring`.
- Implemented `stripPrefix` and `stripSuffix` returning `Option<Substring>`.

### Task 6: Lazy Splitting, Lines, and Replacement (`cb0e64ef`)
- Created lazy `Split`, `SplitWhitespace`, and `Lines` iterators conforming to `CanonicalIterable` and `CanonicalIterator`.
- Implemented `splitOnce(pattern)` returning `Option<Pair<Substring, Substring>>`.
- Implemented `replaceFirst(pattern, replacement)` and `replaceAll(pattern, replacement)`.

### Task 7: StringBuilder, TextWriter, and Linear-Time Joining (`8c039fe0`)
- Defined `trait TextWriter` with `write(text:)` and `writeChar(char:)`.
- Created `class StringBuilder` conforming to `TextWriter`.
- Added `String.rawJoin(list)` native primitive in `primitive/string.rs` joining segments without quadratic intermediate allocations.
- Updated `Iterable.join` and `List.join` to utilize `rawJoin`.

### Task 8: TextCursor and LineMap Tooling (`6cb6f3b3`)
- Created `class TextCursor` for sequential UTF-8 scanning, peek, advance, and span tracking.
- Created `class LineMap` with binary search coordinate resolution yielding `SourceLocation(line: Int, column: Int)`.

### Task 9: UTF-8 Conversions, Typed Target Parsing, and Radix Formatting (`af16bf52`)
- Created `text/errors.ph` defining `Utf8Error`, `IntParseError`, `FloatParseError`, and `BoolParseError`.
- Implemented `String.fromUtf8(bytes)` and `String.fromUtf8Lossy(bytes)`.
- Implemented typed parsing `Int.parse(text, radix:)`, `Float.parse(text)`, and `Bool.parse(text)` returning `Result<T, E>`.
- Implemented `Int.toString(radix:)` with radix validation `2..=36` using native `BigInt::to_str_radix`.

### Task 10: Universe Packaging & Acceptance Integration (`93cb6e07`)
- Wired all text modules into `UNIVERSE_NODES` in `phalcom-modules/src/builtin.rs`.
- Re-exported all text facilities under `universe.text` (`text/package.ph`) and root `universe` (`package.ph`).
- Added debug display rendering for `Substring` in `value/render.rs`.
- Authored comprehensive end-to-end integration test suite in `phalcom-core/tests/core/language/strings_text_model.rs`.

---

## 3. Verification Evidence

All verification commands executed during implementation and consolidation:

1. **Text Model Integration Suite:**
   `RUSTFLAGS='' cargo test -p phalcom-core --test core language::strings_text_model`
   - **Result:** `PASS` (8/8 tests passed in 84.92s)
   - Tested: `string_index_and_counts`, `substring_slicing`, `text_pattern_matching`, `search_trim_strip`, `lazy_splitting_and_lines`, `string_builder_and_writer`, `text_cursor_and_line_map`, `parsing_and_formatting`.

2. **Scalar Char & Unicode Pin Suite:**
   `RUSTFLAGS='' cargo test -p phalcom-core --test core language::chars`
   - **Result:** `PASS` (11/11 tests passed)

3. **String Language Suite:**
   `RUSTFLAGS='' cargo test -p phalcom-core --test core string`
   - **Result:** `PASS` (22/22 tests passed)

4. **Native Surface Contracts:**
   `RUSTFLAGS='' cargo test -p phalcom-core --test core contracts`
   - **Result:** `PASS` (16/16 tests passed)

5. **Native Surface Drift Check:**
   `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check`
   - **Result:** `PASS` (411 primitive declarations confirmed current)

6. **Workspace Rustfmt Check:**
   `cargo fmt --all -- --check`
   - **Result:** `PASS` (Clean formatting across entire workspace)

7. **Workspace Clippy Lint Check:**
   `RUSTFLAGS='' cargo clippy --workspace --all-targets`
   - **Result:** `PASS` (0 warnings in active text model surfaces; clean pass across all crates)

8. **Baseline Failure Audit (Class D):**
   - `language::iteration::canonical_range_for_publishes_semantic_iteration_lane` panics on synthetic assignment to immutable binding (`let sum = 0`). Confirmed to reproduce identically on `main` baseline commit `61c8734f`.
