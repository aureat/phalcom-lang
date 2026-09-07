# Phalcom Native Surface Generator

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-native-surface-gen source snapshot](../raw/native-surface-gen/2026-09-07-phalcom-native-surface-gen.md)
> Updated: 2026-09-07

## Overview

`phalcom-native-surface-gen` is a deterministic command-line generator and drift gate for the canonical native surface. It scans authored primitive attributes, normalizes and validates them through shared crates, emits static metadata records, formats the result, and optionally checks that the committed generated artifact is current.

## Pipeline

The generator accepts `--check` and `--root <path>`. It recursively collects Rust sources below `phalcom-core/src/primitive`, parses them with `syn`, and stores declarations in a sorted map keyed by universe owner, dispatch side, and selector. This makes declaration ordering and duplicate-key behavior deterministic.

Each normalized declaration is lowered into a `NativeSurfaceRecord`. The lowering maps selector kind, visibility, stability, anchor, ABI, trust, intrinsic, effects, raised types, return flow, lifecycle, documentation, and conceptual metadata. Callable `types` strings are parsed by the type-syntax crate; absent contracts receive conservative unknown type shapes derived from selector slots.

## Generated artifact and drift checking

The output is Rust source for `phalcom-native-surface/src/generated.rs`, including the primitive declaration count and the static `NATIVE_SURFACES` array. The generator sends output through `rustfmt` before writing. In check mode it requires the target file to exist and compares it byte-for-byte with freshly generated output, reporting stale output as an error.

The generator also computes semantic return shapes from explicit flow contracts or recognized return types, and it emits generic constraints as native metadata rather than dropping them.

## Evidence and tests

Tests protect value-array emission for applied type expressions and ensure generic callable constraints become `GenericConstraintSpec` metadata. The manifest ties the generator to common, declaration, native-meta, and type-syntax crates plus `syn`, `quote`, and `proc-macro2`.

## See Also

- [Phalcom Native Surface](../native-surface/phalcom-native-surface.md)
- [Phalcom Native Macros](../native-macros/phalcom-native-macros.md)
- [Phalcom Native Declarations](../native-decl/phalcom-native-decl.md)

