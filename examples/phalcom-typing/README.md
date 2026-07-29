# Phalcom Typing Specification Series

This workspace contains the normative incremental design of Phalcom's optional reflective typing system. The series defines visible Phalcom standard-library source, compiler and VM obligations, reflection, diagnostics, and conformance tests without making type metadata participate in ordinary dispatch.

## Series status

| No. | Document | Status |
|---:|---|---|
| 01 | [Protocol Foundation](docs/spec/design/typing/01-protocol-foundation.md) | Complete draft |
| 02 | [Type Expression Foundation](docs/spec/design/typing/02-type-expression-foundation.md) | Complete draft |
| 03 | [Type Parameters and Generic Signatures](docs/spec/design/typing/03-type-parameters-and-generic-signatures.md) | Complete draft; checkpoint ready for review |
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

## Normative policy

The visible Phalcom source in each document is authoritative. Native Rust implementations may provide trusted construction, bootstrap support, GC integration, caching, and acceleration only when they preserve the source contract exactly.

Type metadata is reflectively observable but never implicitly changes selector identity, method lookup, overload resolution, instance layout, allocation, inline-cache identity, or automatic value validation.

## Checkpoint contents

- `docs/spec/design/typing/01-protocol-foundation.md`
- `docs/spec/design/typing/02-type-expression-foundation.md`
- `docs/spec/design/typing/03-type-parameters-and-generic-signatures.md`
- `STATUS.md`
- `CHANGELOG.md`

The Phase 1 `.ph` package is a separate design reference and is not copied into this specification workspace.
