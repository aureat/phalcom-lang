# TDR-0091 — Bracket literals form contextual homogeneous sequences

- Ratified: 2026-10-09
- Status: Accepted
- Date: 2026-10-09
- Supersedes: none
- Amends: [TDR-0025](./0025-list-literal-syntax.md) and [TDR-0028](./0028-collections-representation-and-literals.md) for bracket-literal family selection, defaults, and lowering authority
- Related: `COLL006.C1`; [Sequence Literals](../../specs/sequence-literals.md); [Static Evaluation](../../specs/static-evaluation.md); [TDR-0049](./0049-index-operator-as-real-selector.md); [TDR-0090](./0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md)

## Context

TDR-0025 originally defined every bare bracket literal as a `List` construction
send. The collection upgrade adds fixed-extent `Array<T>` and immutable,
statically dimensioned `Vector<T, N>` alongside resizable `List<T>`. One
context-sensitive sequence syntax must select among these existing families
without making the parser or compiler guess from runtime behavior.

## Decision

`[...]` is a homogeneous sequence literal. Its elements share one inferred or
contextually supplied element type. The checked semantic family determines the
constructed sequence; the parser does not intrinsically construct a `List`.

Family selection follows this order:

1. An explicit literal head selects its family: `<T>[...]` selects `Array<T>`;
   `<T, N>[...]` selects `Vector<T, N>`.
2. A unique expected `List<T>`, `Array<T>`, or `Vector<T, N>` selects that
   family and guides element/extent inference.
3. With no explicit or unique contextual family, a runtime expression selects
   `Array<T>` and a static expression selects `Vector<T, N>`.

The phase default applies only after overload and contextual resolution. It MUST
NOT break an overload tie or make one otherwise ambiguous candidate win. The
programmer must supply a family context or explicit literal head when more than
one collection family remains viable.

`List<T>` remains the dynamically resizable family and is selected explicitly
through a type context or named constructor. Ordinary `Array<T>` has fixed
runtime extent and mutable elements. `Vector<T, N>` is immutable and carries an
exact static extent; `N` equals the number of produced elements after the
literal's spreads are resolved. Element type and extent are never invented
from an empty literal: ordinary inference must provide their evidence.

Static sequence evaluation follows the static-value and materialization rules
owned by `docs/specs/static-evaluation.md`. A closed static `Vector<T, N>` value
requires exact extent evidence. If a required Vector identity spread retains a
symbolic extent, static evaluation remains symbolic and publishes no closed
sequence value; reduction may proceed once the extent is a closed static
integer. This rule does not add symbolic arithmetic or evaluator-side generic
substitution.

Bracket construction remains distinct from the subscript selectors governed
by TDR-0049. A computed Record field key written `[expression]` is also a
separate syntax form.

## Consequences

- The original List literal is amended without changing the `List<T>` API or
  runtime representation.
- Semantic analysis owns family selection. Lowering consumes the checked
  family and may not infer it from phase or reconstruct it from syntax.
- Static defaults use the same Vector extent rules as explicit/contextual
  Vector literals. Runtime defaults preserve Array fixed-extent behavior.
- Existing List-specific source and tests must state their List context.

## Alternatives considered

- **Keep every bare literal as List.** Rejected because it prevents the same
  syntax from expressing the distinct runtime Array and static Vector defaults
  already required by the collection family.
- **Let the compiler choose from its execution phase.** Rejected because the
  compiler consumes checked semantic products and does not own phase-based
  language meaning.
- **Use the phase default to rank overload candidates.** Rejected because it
  silently converts an ambiguous call into a family preference.
