# Phalcom Native Metadata

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-native-meta source snapshot](../raw/native-meta/2026-09-07-phalcom-native-meta.md)
> Updated: 2026-09-07

## Overview

`phalcom-native-meta` is the VM-free vocabulary for native primitive metadata, static symbolic type/callable specifications, and the canonical universe catalog. It is the declarative layer consumed by declaration parsing, procedural expansion, generated surfaces, runtime registration, and tooling.

## Primitive contracts

A `PrimitiveSurfaceSpec` binds a stable `PrimitiveKey` to visibility, stability, anchor policy, parameter/return/callable type specifications, raised-type and effect contracts, return flow, termination, lifecycle, intrinsic identity, trust, documentation, and conceptual text.

The metadata enums distinguish instance/class dispatch, public/internal visibility, unspecified/experimental/stable lifecycle, value/shape ABI, ordinary/privileged trust, source/native/generated/abstract/external implementation kinds, and return flows such as value, receiver, argument, never, or unknown. Effects can be pure, unknown, or a static list covering mutation, I/O, scheduling, reflection, nondeterminism, and blocking.

## Static type specifications

The type specification model mirrors the syntax layer while remaining static and shareable: unknown, never, self, universe keys, named parameters, applied types, unions, tuples, labeled/rest parameters, generic parameters, and subtype/equivalence constraints. Callable specifications own type parameters, parameter tuples, return types, and constraints through static references.

## Universe catalog

The universe module declares native surface schema version 1, the stable `UniverseKey` enumeration, runtime-owned superclass relations, canonical bindings, and universe type forms. The catalog includes core object/value classes, callable and method-family classes, collection and module/project classes, errors, fibers/resources, and identity/manifest classes. Source-only helper classes are intentionally absent from the runtime-owned class relation table.

## Evidence and tests

The crate root re-exports primitive, type, and universe modules. The manifest describes it as VM-free and currently has no runtime dependency; the type-syntax dependency remains commented as an integration boundary.

## See Also

- [Phalcom Type Syntax](../type-system/phalcom-type-syntax.md)
- [Phalcom Native Declarations](../native-decl/phalcom-native-decl.md)
- [Phalcom Native Surface](../native-surface/phalcom-native-surface.md)

