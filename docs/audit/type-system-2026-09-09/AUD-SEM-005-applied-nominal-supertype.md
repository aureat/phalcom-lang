# AUD-SEM-005 — Applied subclasses lose nongeneric supertypes

## Classification

**Severity:** Medium. **Category:** Completeness. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/types/relation.rs:306–393` (nominal/applied arms), `:485–488` (Object fallback and refutation); `phalcom-semantic/src/checker/inference.rs:2500` delegates concrete subtyping.

## Observed behavior

Child<Int> is rejected as Base even with `class Child<T> is Base`. The source call emits GenericInferenceConflict and ArgumentMismatch; the isolated relation with the hierarchy edge also returns false.

## Expected invariant

Specializing a generic subclass must preserve its declared nongeneric superclass relation.

## Root cause

The relation handles Nominal→Nominal, Nominal→Applied and Applied→Applied, but lacks Applied→Nominal ancestor projection except the special Object fallback. Receiver-member specialization has a separate hierarchy walker that does support nongeneric ancestors, producing an overlapping but inconsistent boundary.

## Minimal reproducer

`generic_inheritance_source_accepts_base_parameter` and `applied_subclass_retains_nongeneric_superclass` in [the probes](evidence/semantic_audit_probe.rs).

## Impact

Valid argument passing, assignments and generic bound checks reject applied subclasses. This affects collection-style hierarchies such as List<T> is Iterable as well, although the retained source case uses user classes to isolate the rule.

## Fix direction

Project the actual receiver through specialized superclass templates for either target representation, sharing the owner-projection contract with member specialization. Preserve kinds and do not treat the unsaturated Child constructor as a proper value type.

## Tests required

The two probes above; add Base<T>→Middle<U>→Concrete, nongeneric intermediary, class-side counterpart, negative unrelated ancestor and preserved invariant arguments.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
