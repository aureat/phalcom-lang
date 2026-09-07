# STDL001.C1 — Numeric library spec

**Canonical implementation plan:** [full numeric tower implementation plan](P1-numeric-tower-contract-and-migration.md). The six
unit files below are compact execution cards; they do not replace the plan's requirements,
invariants, write-set, gates, and scope boundary.

Implements [numeric specs](../../../spec/library/numbers/README.md) and ratified
[PDR-0027](../../../pdr/0027-float-protocol-and-explicit-narrowing.md). This is one dependency
chain; do not merge a later phase over a red earlier phase.

| Phase | Unit | Depends on | Exit gate |
|---|---|---|---|
| 1 | [value model and classes](P4-numeric-value-model-and-classes.md) | current tower plan | `Value::Int`/`Float`, `LargeInt`, concrete classes, abstract allocation |
| 2 | [literals and power syntax](../C2-numeric-literals-and-integer-arithmetic/P1-numeric-literals-and-power-syntax.md) | 1 | typed numeric literals and right-associative `**` parse with spans |
| 3 | [Int and LargeInt](../C2-numeric-literals-and-integer-arithmetic/P2-integer-and-large-integer.md) | 1, 2 | exact arithmetic, limits, constant GC-root gate |
| 4 | [Float, text, and keys](../C3-float-protocol-and-text/P1-float-protocol-and-text.md) | 1, 2, 3 | IEEE protocol, canonical text, constructor behavior, Map/Set coherence |
| 5 | [numeric errors and tracebacks](../C4-numeric-errors-and-verification/P1-numeric-errors-and-tracebacks.md) | 2–4 | structured errors with rich source labels where available |
| 6 | [integration and verification](../C4-numeric-errors-and-verification/P2-numeric-integration-and-verification.md) | 1–5 | floor census, conformance matrix, fuzz/GC and graph refresh |

Non-goals live in [numeric follow-ups](../../deferred/numeric-followups.md). In particular,
do not sneak strict Float-index rejection, serialization, or an extended math library into this
unit.
