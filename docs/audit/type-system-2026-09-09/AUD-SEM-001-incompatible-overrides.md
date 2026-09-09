# AUD-SEM-001 — Incompatible overrides invalidate nominal call contracts

## Classification

**Severity:** Critical. **Category:** Soundness. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/types/relation.rs:306` (nominal subtyping); `phalcom-semantic/src/dispatch.rs:295` (`resolve_dispatch_with_trace`); `phalcom-semantic/src/surface.rs:62` (`add_callable_with_visibility`); `phalcom-semantic/src/checker/context.rs:1961` (signature selection/specialization); `phalcom-core/src/vm/send.rs:450` (receiver method lookup).

## Observed behavior

The semantic analyzer accepts the complete reproducer without error. Calling a method through a `Base` parameter uses its `Int` signature. Actual execution prints `wrong`, a String returned by the overriding method.

## Expected invariant

Every dynamically reachable override must satisfy the statically selected base contract. Return types must be covariant, parameters contravariant, and generic constraints must not strengthen the caller obligation.

## Root cause

Nominal subclass acceptance and per-owner selector lookup are independent. Published surfaces contain each method signature, but the traced publication/dispatch path does not establish compatibility with overridden ancestor signatures. The VM dispatches using the actual Child receiver; neither this call nor its enclosing Int return enforces the contradicted contract.

## Minimal reproducer

[override-return.ph](override-return.ph)

## Impact

An entirely statically annotated call path, with no Dynamic escape, produces String where the analyzer permits an Int return. Compiler optimizations cannot use that contract as an unconditional proof.

## Fix direction

Validate overrides at declaration publication, after specializing ancestor generics into the descendant owner. Preserve selector kind, side, labels/rest shape, generic binder identities and receiver-relative Self. Reject incompatible declarations or explicitly prevent treating their nominal hierarchy edge as a proof. Runtime boundary validation is additional defense, not a replacement for the static obligation.

## Tests required

`incompatible_override_must_not_certify_int` in the retained semantic probes; add argument narrowing, inherited generic constraints, multi-level override, class-side, getter/setter/indexer and Self variants. Assert the offending declaration/signature status and diagnostic ownership. Runtime regression must preflight semantics and verify that an accepted Int result is actually Int.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
