# Phalcom Native Declarations

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-native-decl source snapshot](../raw/native-decl/2026-09-07-phalcom-native-decl.md)
> Updated: 2026-09-07

## Overview

`phalcom-native-decl` is the shared parser, normalizer, and validator for native primitive declarations. It gives procedural macros and source tooling one owned, VM-free representation instead of making each consumer reinterpret attribute tokens independently.

## Normalized declaration

A `NormalizedPrimitiveDecl` contains a `PrimitiveDeclKey`—universe owner plus selector—alongside the original normalized fields and parsed metadata. The normalized slots cover parameter and return text, a callable type contract, raised types, effects, dispatch side, visibility, stability, anchor policy, lifecycle strings, ABI, return flow, intrinsic, trust, conceptual text, and captured documentation.

The parser accepts a `syn` attribute whose first values identify the universe owner and selector. It records every field as a name/value pair, recognizes bracketed values where needed, parses enum-like metadata, and captures attached Rust `///` documentation without interpreting the prose.

## Validation rules

Validation decodes the selector through the shared exact-selector contract and accepts only its canonical spelling, with the documented compatibility spelling for rest markers. Internal selectors beginning with `_$` must be explicitly internal; public selectors cannot claim internal visibility. A replacement requires a deprecation marker.

Unknown fields, duplicate fields, malformed selectors, and invalid metadata are represented by the public `DeclError` variants rather than panics. This makes declaration errors available to both macro diagnostics and standalone tooling.

## Evidence and tests

The focused tests verify that an internal selector with internal visibility is accepted and the same selector with public visibility is rejected. The manifest ties the crate to `phalcom-common`, `phalcom-native-meta`, `syn`, `quote`, `proc-macro2`, and `thiserror`.

## See Also

- [Phalcom Common](../common/phalcom-common.md)
- [Phalcom Native Metadata](../native-meta/phalcom-native-meta.md)
- [Phalcom Native Macros](../native-macros/phalcom-native-macros.md)

