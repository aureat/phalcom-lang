# Phalcom Common

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-common source snapshot](../raw/common/2026-09-07-phalcom-common.md)
> Updated: 2026-09-07

## Overview

`phalcom-common` contains small, compiler-stage-agnostic utilities that are intended to stay stable across VM redesigns. Its public surface is organized around source ranges and structural selector identity, with no VM or AST dependency.

## Source ranges

`CopyRange<T>` is a `Copy`-enabled half-open interval with inclusive `start` and exclusive `end` bounds. It provides construction, emptiness, point containment, range containment, overlap detection, and convex-hull merging. The `usize` specialization adds saturating length calculation and conversion from `std::ops::Range<usize>`.

`SourceRange` aliases `CopyRange<usize>` for byte offsets into Phalcom source text. `EmptySourceRange` is the zero-length placeholder used for synthetic nodes without an original source location. This keeps spans cheap to copy while allowing downstream diagnostics to retain precise source ownership.

## Selector identity

Selectors model dispatch identity without depending on a parser, AST, or VM representation. A selector has a named or subscript base, a kind (getter, setter, method, subscript get, or subscript set), and ordered positional/label slots. Selector patterns are separate predicates with prefix/suffix slots and an explicit gap; they are not dispatch keys.

The exact decoder validates canonical source-like forms, slot ordering, base/kind compatibility, and slot limits. Runtime decoding is deliberately total: malformed or legacy runtime text is converted to a reflection-safe selector rather than causing a panic. Runtime-originated rest-family markers are accepted by the permissive path but intentionally bypass strict exact-selector validation.

Labels with unsafe or reserved symbol text use a tilde-prefixed hexadecimal escape. The inverse decoder preserves malformed escapes as text, keeping reflection and diagnostics total even when transport input is damaged.

## Lifecycle boundary

The crate documentation records the retirement of the former shared-owner reference aliases after the handle/arena heap redesign. Heap ownership now belongs to the VM heap layer; this crate remains limited to reusable identity and source-location primitives.

## Evidence and tests

Tests cover half-open range behavior, reversed-range length saturation, overlap/merge semantics, selector construction and decoding, pattern round trips, and malformed label escapes. The only direct dependency in the manifest is `thiserror`.

## See Also

- [Phalcom Diagnostics](../diagnostics/phalcom-diagnostics.md)
- [Phalcom Type Syntax](../type-system/phalcom-type-syntax.md)

