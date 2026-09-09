# AUD-SEM-003 — Applied type value lowers to None

## Classification

**Severity:** High. **Category:** Correctness. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-core/src/compiler/lib/expr.rs:1441–1449` (`Expr::TypeForm` arm); `phalcom-semantic/src/checker/context.rs:1951–1985` (distinct dispatch and applied specialization receiver).

## Observed behavior

`Box<Int>.new()` receives an applied Box<Int> semantic result in the dedicated probe. Running the source fixture fails with `None does not understand new()`. The compiler emits Nil for the applied type-form receiver.

## Expected invariant

An accepted runtime type-form expression must lower to the represented class/type value, preserving whatever specialization metadata the runtime contract requires. It cannot silently become None.

## Root cause

Type-form lowering handles only Reference; every other annotation form emits Bytecode::Nil. Static class-object dispatch plus application specialization therefore has no corresponding executable value for this path.

## Minimal reproducer

[applied-constructor.ph](applied-constructor.ph)

## Impact

Common applied construction fails before an instance exists. This also prevents this path from establishing reified List<Int>/List<String> object behavior. It is a runtime correctness failure; this example does not demonstrate mutable generic metadata corruption.

## Fix direction

Consume canonical type denotation/executable semantic products in lowering. Build the supported applied runtime type value or intentionally erase to the origin only if the effective semantics allow that erasure. Reject unsupported forms explicitly until implemented; never substitute Nil as successful lowering.

## Tests required

`applied_constructor_has_precise_static_type` is a positive static control. Add production-compiler/Universe regression executing this same source, then distinguish Box<Int>, Box<String>, raw Box and applied class-side Self. Repeat for native List and applied enum construction.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
