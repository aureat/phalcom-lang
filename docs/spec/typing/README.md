# Historical Phalcom Typing Design Series

> **Historical / non-effective.** This directory preserves an earlier typing
> design series for reference. It is not the active authority for behavioral
> contract declarations. The effective C3 rule is [First-Class Traits](../extensions/traits.md).
> In particular, `@protocol class` and signature-only protocol declarations
> must not be treated as current Phalcom syntax or semantics.

This workspace contains the historical incremental design of Phalcom's optional
reflective typing system. The series is retained to preserve rationale and
future-facing material; it does not override `docs/spec/` current chapters or
ratified extensions.

## Series status

| No. | Document | Status |
|---:|---|---|
| 01 | [Protocol Foundation](01-protocol-foundation.md) | Historical; superseded for behavioral contracts |
| 02 | [Type Expression Foundation](02-type-expression-foundation.md) | Historical design reference |
| 03 | [Type Parameters and Generic Signatures](03-type-parameters-and-generic-signatures.md) | Historical; trait generics follow the effective trait rule |
| 04 | Type Application and Applied Types | Next |
| 05 | Substitution and Applied Member Views | Planned |
| 06 | Applied-Type Class-Side Forwarding | Planned |
| 07 | Type Lattice and Special Types | Planned |
| 08 | Variance and Generic Subtyping | Planned |
| 09 | Bounds, Constraints, and Inference | Planned |
| 10 | Structural Protocol Conformance | Planned |
| 11 | Abstract Classes and Obligations | Planned |
| 12 | Generic Inheritance and Self | Planned |
| 13 | Block Types and Callables | Planned |
| 14 | Data Classes and Immutable Types | Planned |
| 15 | Sealed Classes, Variants, and Generic ADTs | Planned |
| 16 | Type Aliases, Intersections, and Composition | Planned |
| 17 | Reflection Metadata and Bytecode Encoding | Planned |
| 18 | Bootstrap, Native Floor, and Security | Planned |
| 19 | Checker Modes, Diagnostics, and Tooling | Planned |
| 20 | Complete Typing Module Reference | Planned |
| 21 | Typing Conformance Suite | Planned |

## Historical policy

The visible Phalcom source in each document records the historical proposal. It
is not an effective language rule. Native implementations and later plans must
follow the current specification and checkpoint records instead.

Type metadata is reflectively observable but never implicitly changes selector identity, method lookup, overload resolution, instance layout, allocation, inline-cache identity, or automatic value validation.

## Checkpoint contents

- `docs/spec/design/typing/01-protocol-foundation.md`
- `docs/spec/design/typing/02-type-expression-foundation.md`
- `docs/spec/design/typing/03-type-parameters-and-generic-signatures.md`
- `STATUS.md`
- `CHANGELOG.md`

The Phase 1 `.ph` package is a separate design reference and is not copied into this specification workspace.
