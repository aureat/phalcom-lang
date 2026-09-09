# AUD-SEM-006 — Empty tuple literal is not canonical Unit

## Classification

**Severity:** Medium. **Category:** Completeness / Diagnostics. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/checker/expression.rs:1823–1882` (`synthesize_tuple_literal`); `phalcom-semantic/src/types/store.rs:509–513` (`tuple`); `phalcom-semantic/src/types/annotation.rs:828,863` (Unit annotations).

## Observed behavior

Both `let a: Unit = ()` and `let b: () = ()` receive BindingInitializerMismatch. The expression path interns Tuple([]), while both annotation spellings resolve to store.unit().

## Expected invariant

The empty tuple value and unit value/type have one canonical semantic identity. See `docs/spec/typing/01-core-type-lattice-and-unit.md:200–239`.

## Root cause

Tuple synthesis handles zero entries like every other arity; TypeStore::tuple does not normalize an empty array to Unit. The subtype relation has no equivalence bridge. This is a real type mismatch before diagnostic formatting.

## Minimal reproducer

`unit_spellings_have_one_source_identity` in [the probes](evidence/semantic_audit_probe.rs).

## Impact

Ordinary valid Unit initializers and explicit unit-return paths can be rejected. Separate IDs also threaten equality-keyed substitutions and unions involving the same semantic unit.

## Fix direction

Canonicalize empty tuple formation to Unit at the type-store construction boundary; reconcile tuple projections, matching and runtime Tuple membership with that singleton representation. Do not merely rename Tuple([]) in diagnostics.

## Tests required

The source probe plus direct store.tuple([])==store.unit(), explicit return (), default unit result, tuple expansion of (), Unit in unions, and source/runtime reflection agreement.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
