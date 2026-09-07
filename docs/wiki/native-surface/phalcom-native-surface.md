# Phalcom Native Surface

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-native-surface source snapshot](../raw/native-surface/2026-09-07-phalcom-native-surface.md)
> Updated: 2026-09-07

## Overview

`phalcom-native-surface` is the canonical VM-free native-member catalog shared by runtime registration and tooling. It deliberately avoids VM, AST, and LSP dependencies while preserving enough declarative metadata for lookup, validation, reflection, and editor consumers.

## Surface records and lookup

A `NativeSurfaceRecord` combines a `PrimitiveSurfaceSpec` with member category, ABI, and semantic return shape. The record exposes owner, dispatch side, selector, visibility, stability, anchor, parameter, return, callable, effect, flow, intrinsic, trust, documentation, conceptual, and lifecycle accessors.

`NativeSurfaceCatalog` keeps the generated records and lazily builds indexes for full primitive keys, owner/side/selector lookups, and selector-only lookups. Selector-only lookup sorts by stable key before choosing the first match, keeping ambiguous legacy lookups deterministic. Convenience functions expose canonical lookup and owner-filtered iteration.

The generated source currently declares 319 authored primitive declarations in its header and provides the static `NATIVE_SURFACES` record array. The generated body is intentionally kept in the source tree rather than duplicated in the wiki raw snapshot.

## Fingerprints and invariants

Catalog fingerprints sort records by stable key and hash structural metadata, including callable/type contracts and lifecycle/effect/flow information. Debug source locations are excluded, so source-line movement does not change the structural catalog identity.

Validation reports duplicate keys, inconsistent lifecycle metadata, and illegal intrinsic expectations. The built-in intrinsic contract currently checks boolean `and(_)`, `or(_)`, and `not` fallback selectors and their arities.

`NativeReturnShape` makes result projection explicit: unknown, canonical instance, receiver, class object, or an unchanged argument.

## Evidence and tests

Tests require catalog uniqueness and intrinsic safety, require fingerprint stability under record reordering, and verify that changing a generic callable contract changes the fingerprint.

## See Also

- [Phalcom Native Metadata](../native-meta/phalcom-native-meta.md)
- [Phalcom Native Surface Generator](../native-surface-gen/phalcom-native-surface-gen.md)
- [Phalcom Benchmarks](../benchmarks/benchmarks.md)

