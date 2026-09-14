# Language Extensions

This directory contains ratified optional language extensions. Each extension
has one canonical behavioral specification; implementation plans and historical
designs do not replace that rule.

## Canonical extensions

| Extension | Effective rule | Status |
|---|---|---|
| [First-class traits](traits.md) | Non-storage behavioral contracts with optional defaults | Ratified for C3 |

## Migration note

`traits.md` is the canonical replacement for the protocol-shaped behavioral
contract rules that were previously described in the following retained design
documents:

- `docs/spec/typing/01-protocol-foundation.md`
- `docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md`
- `docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md`
- `docs/spec/typing/03-type-parameters-and-generic-signatures.md`
- `docs/spec/next/phalcom-meta-dispatch-and-type-extension-spec.md`

Those documents remain available as historical or deferred design material, but
their `@protocol class` contract model is not an active language rule. The
migration is intentionally narrow: ordinary uses of the word “protocol”, such
as the iteration protocol, are unaffected.
