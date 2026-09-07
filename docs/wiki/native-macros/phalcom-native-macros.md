# Phalcom Native Macros

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-native-macros source snapshot](../raw/native-macros/2026-09-07-phalcom-native-macros.md)
> Updated: 2026-09-07

## Overview

`phalcom-native-macros` provides the procedural `#[primitive]` attribute used to declare native Phalcom primitives. It parses the attribute at the Rust item boundary, delegates shared normalization/validation, then emits compile-time errors or expanded metadata-aware declarations.

## Attribute surface

The attribute begins with a `UniverseKey` owner and a selector string. Optional fields cover parameter and return spellings, a full callable `types` contract, raised expressions, effects, dispatch side, visibility, stability, anchor, lifecycle, ABI, return flow, intrinsic, trust, and conceptual documentation.

Parameter syntax supports positional types, labeled entries, and rest entries. Type contracts are parsed through `phalcom-type-syntax`; selector legality is checked through `phalcom-common`; normalized declaration semantics come from `phalcom-native-decl`; and static output types come from `phalcom-native-meta`.

## Expansion and diagnostics

The macro validates exact selector spelling, including the compatibility spelling for rest-family selectors, and enforces the relationship between internal selector prefixes and visibility. It maps surface strings into static metadata for effects, raised types, return flow, lifecycle, intrinsic identity, trust, callable types, and return shape.

Invalid declarations are converted into spans and compile errors through `syn`/`quote` rather than causing procedural-macro panics. Attached Rust documentation is retained for downstream Phaldoc or tooling consumers.

## Evidence and tests

The macro crate is a proc-macro library. Its dependencies show the intended integration boundary: `syn`, `quote`, `proc-macro2`, and the shared common, type-syntax, native-declaration, and native-metadata crates.

## See Also

- [Phalcom Native Declarations](../native-decl/phalcom-native-decl.md)
- [Phalcom Type Syntax](../type-system/phalcom-type-syntax.md)
- [Phalcom Native Surface Generator](../native-surface-gen/phalcom-native-surface-gen.md)

