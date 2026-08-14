# Soundness, Precision, and Cost

## Three axes

An analysis choice should state:

```text
soundness contract
precision target
resource budget
```

You cannot maximize all analyses universally.

## False positives/negatives

For a sound may-analysis used to detect possible errors, over-approximation can produce false positives but should avoid false negatives under modeled assumptions.

For advisory LSP suggestions, usefulness may justify heuristics, but label them as such and never feed them into proof without validation.

## Completeness

Most interesting program properties are undecidable. "Unknown" is a valid result. Do not turn resource exhaustion into acceptance.

## Precision cliffs

Common expensive upgrades:

- path insensitive -> path sensitive;
- context insensitive -> context sensitive;
- field insensitive -> field sensitive;
- allocation-site -> object-sensitive heap;
- intervals -> polyhedra/SMT relational reasoning.

Demand evidence before crossing each cliff.

## Widening visibility

When widening loses precision, store a reason/provenance. Diagnostics can say "cannot prove because recursive analysis widened" rather than claiming source is untyped.

## Trust levels

Facts can carry quality:

```text
Exact syntax/runtime invariant
Sound abstract fact
Declared/trusted contract
Interprocedural sound summary
Heuristic/editor inference
```

Consumers choose minimum trust they accept.

## Resource limits

Use deterministic limits:

- union cardinality;
- path partitions;
- solver iterations;
- recursive relation nodes;
- provenance samples.

Fallback must preserve the promised soundness direction.

## Security analyses

Security checks are high stakes; heuristic silence must not be interpreted as safe. Prefer conservative taint/dataflow with explicit sanitizer/source/sink semantics.
