# AUD-I1 — Callable-body invalidation lacks absence-dependency certification

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Medium; incremental correctness certification gap
- **Status:** Plausible architecture, unproven edge case at `ee7e8ee7`
- **Current owner:** semantic database invalidation and conformance clients

This is a missing proof obligation, not a confirmed production failure. The
callable-body fingerprint narrowing must not be certified until the absence
transitions below are tested.

## Issue

The C4.1 corrective work removed broad workspace and linked-program products
from the direct callable-body input fingerprint. Body queries now rely more
heavily on recorded semantic dependencies, including exact callable signatures,
callable definitions, trait surfaces, and `ConformanceDispatch(ImplId)` keys.

That direction is sound for dependencies that existed in the previous
revision. It is harder for a dependency on absence:

```text
revision N:
    no conformance exists
    consumer sees Missing
    no ConformanceDispatch(new_impl) edge can exist

revision N+1:
    a conformance is added
    consumer must be reconsidered
```

The reverse transition is equally important:

```text
revision N:
    conformance exists and consumer uses it

revision N+1:
    conformance is removed
    former consumer must be reconsidered and evidence retracted
```

## Evidence in the live tree

- `db/fingerprint.rs` now describes direct body-relative inputs and omits broad
  source, linked-component, and aggregate conformance fingerprints.
- `checker/iteration.rs` and `checker/context.rs` record
  `SemanticDependency::ConformanceDispatch` when positive evidence is consumed.
- `session.rs:6295-6325` compares settled conformance plans and computes changed
  plan reverse closures.
- `session.rs:1781-1810` retains prior conformance clients when structural
  contract products changed.
- Existing incremental tests cover positive conformance plan changes and body
  reuse, but no focused test found in this audit demonstrates absent → added or
  present → removed cross-module parity.

## Why this matters

Dependency-driven invalidation must model both positive dependency edges and
the semantic search space in which a missing candidate was established. A
consumer that asks a conformance query and receives `Missing` may record a
declaration-surface or trait-surface dependency rather than an impl-specific
edge. If that dependency is not sufficient to invalidate the consumer when a
new implementation appears, the incremental snapshot can retain stale
dispatch, diagnostics, body analysis, or generic evidence.

The narrowing is therefore not safe to certify from positive-use tests alone.

## Consequences if the gap is real

- A newly added conformance may be invisible to an unchanged consumer until a
  cold rebuild.
- A removed conformance may leave stale positive dispatch or stale proof facts.
- Cross-module consumers can differ from same-module consumers because their
  source and aggregate invalidation paths differ.
- Revalidation may update diagnostics while retaining an old callable analysis,
  or update a body without retracting old conformance dependencies.
- The semantic snapshot can violate incremental/cold parity while all local
  focused tests remain green.

## Correct architectural approach

The implementation must retain a dependency on the authority that answers the
negative query. There are two acceptable designs; the choice must be made from
live dependency products rather than guessed:

1. **Negative-query dependency:** record a stable query key for the relevant
   trait/target search space whenever conformance resolution returns `Missing`,
   `Unknown`, or another non-positive result that influenced checking. A later
   conformance plan change invalidates that key.
2. **Existing aggregate invalidation:** prove that every addition/removal
   changes an already-recorded declaration, trait, conformance aggregate, or
   settled-plan product that is already in the consumer's dependency closure.
   Add explicit assertions that this remains true for cross-module absence.

Do not restore broad workspace fingerprints merely to make the tests pass. The
goal is exact dependency ownership, including negative observations and
retraction.

For removal, the old positive dependency must be retired with the old product,
and the consumer must be re-evaluated against the new conformance index. For
addition, the new implementation cannot be assumed to have been referenced in
the previous graph; the provider/search authority must be the invalidation
root.

## Testing strategy

Build one cross-module fixture with a trait, a consumer callable, and a target
type. Keep the consumer source and its public signature unchanged across
revisions.

### Absent → added

1. Analyze revision N with no conformance.
2. Assert the consumer observes missing dispatch/evidence or the expected
   generic outcome.
3. Add the conformance in a provider module only.
4. Update the same session incrementally.
5. Assert the consumer product, dispatch/evidence, diagnostics, and dependency
   edges reflect the new conformance.
6. Analyze revision N+1 cold and compare the complete relevant products.

### Present → removed

1. Analyze revision N with a valid conformance and a positive consumer.
2. Remove only the provider conformance.
3. Update incrementally.
4. Assert stale positive evidence and dispatch are gone.
5. Compare the result with a cold analysis of the provider-without-conformance
   workspace.

Use `Arc::ptr_eq` only as supplementary evidence. Equality of semantic
products, diagnostics, selected callable, completeness, and cold parity are
the behavioral assertions.

## Investigation leads

- `db/query.rs` dependency conversion and reverse-closure code around the
  `ConformanceDispatch` cases.
- `session.rs:1781-1810` prior-client seeding.
- `session.rs:6295-6385` settled plan change propagation.
- Existing incremental trait tests around conformance plan changes.
- `QueryKey::ConformanceDispatch` and any trait/declaration surface keys used
  when a lookup returns `Missing`.

## Acceptance condition

The issue is closed when both absence transitions pass incrementally and cold,
the relevant dependency or aggregate invalidation is asserted, stale evidence
is retracted on removal, and no broad fingerprint is reintroduced without an
explicit architectural decision.
