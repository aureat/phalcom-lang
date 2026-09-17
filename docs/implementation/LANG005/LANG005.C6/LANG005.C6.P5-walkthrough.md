---
id: LANG005.C6.P5
kind: implementation-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
revision: c0ede6acceaae541d3ba161a2dad980b2a1d7d1d
prepared: 2026-09-17
---

# LANG005.C6.P5 walkthrough

This continuation is based on `c0ede6acceaae541d3ba161a2dad980b2a1d7d1d`
with the implementation changes retained in the working tree.

Part I landed the semantic and executable canonical iteration vertical for
`List`, while preserving the staged compatibility boundary for sources that do
not yet publish canonical `Iterable` evidence.

## Implementation slices

| Slice | Result |
|---|---|
| T1 | `CallableAnalysis.for_iteration_lanes` publishes canonical source/item/iterator products with exact, assumed, or derived evidence. |
| T2 | Semantic lowering projects selected trait calls or generic requirement slots; compiler emission consumes those frozen specs. |
| T3 | `ListIterator<T>` owns fresh mutable state; `List` publishes canonical `Iterable` conformance and returns a fresh iterator. |
| T4–T5 | Canonical single and parallel loops preserve source ordering, `Option` payloads, patterns, ordinals, captures, break/continue, and strict zip mismatch behavior. |

## Focused verification

| Surface | Result |
|---|---:|
| `cargo check -p phalcom-semantic` | passed |
| `cargo check -p phalcom-core` | passed |
| `cargo check -p phalcom-modules` | passed |
| semantic `capabilities::iteration` | 6 passed |
| semantic `capabilities::derived_conformance` | 38 passed |
| core `language::derived_conformance` | 11 passed |
| core `language::iteration` | 5 passed |
| `git diff --check` | passed |

The positive `language-corpus::iteration` lane remains baseline-blocked by the
pre-existing generic `Option.unwrapOr` call-entry recipe failure. The
non-List built-in migration and compatibility cursor retirement are deferred
to the later P5 parts.

## Part II continuation — built-in stateful surfaces

The Part II pass added stateful canonical iterators for `Bytes`, string byte
and code-point views, `Set<T>`, `Map<K,V>`, and Map key/value/entry views. Map
canonical items use the required labeled `(key: K, value: V)` shape. Legacy
cursor methods remain in place for the staged compatibility boundary.

The focused Bytes/String test passed. Set and Map direct iterator/view tests
now pass after the getter transport correction. Tuple was attempted and
reverted when the live semantic model rejected `Dynamic` as a publishable
associated Item type. Range now has a source-authored stateful iterator, but
its focused path reaches the inherited `Option<Number>` runtime recipe
boundary. These remain explicit blockers; no assertions or semantic baseline
expectations were weakened.

The resulting state is `IMPLEMENTED / BASELINE_BLOCKED` for the landed
Bytes/String slice, with Part II partial overall. G3 is not passed, and T9–T11
were not started.

## Advisory corrective slice — applied class receivers

The supplied advisory decision was implemented as a bounded prerequisite
repair. Applied nominal type forms now project through
`ModuleLoweringSemantics.type_form_values` to the ordinary declaration class
object; their generic arguments remain semantic call-specialization data. The
native Set/Map class-side `new()` contracts now publish receiver-flow `Self`
with constructor semantics, and iterator source bodies retain explicit
generic constructor forms.

Focused regressions pass for contextual `Set<Int>`/`Set<String>` inference,
applied `Set<Int>` and `Map<Int, String>` construction, concrete and generic
getter recipe transport, metadata-pool rebasing, direct Set iteration, and
direct Map iteration/views. Range's iterator implementation is present but
its `Option<Number>` observation still reaches the inherited baseline. No VM
inference or permissive template fallback was added; Tuple's `Dynamic` Item
surface remains deferred, G3 remains unpassed, and T9–T11 are not started.

## Lifecycle state

Part I is `IMPLEMENTED` for the landed canonical List/`for` slice, but P5 and
C6 remain `PARTIAL / BASELINE_BLOCKED`. The next implementation owner is Part
II T6.
