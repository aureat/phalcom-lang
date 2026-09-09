# AUD-SEM-004 — Successful union checks leak active-cycle state

## Classification

**Severity:** Medium. **Category:** Completeness. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/types/relation.rs:261` (visited insertion), `:286–294` (union-supertype branch), `:490` (cleanup); `check_subtype_impl`.

## Observed behavior

`let value: (Int | String, Int | String) = (1, 2)` receives BindingInitializerMismatch. The pure relation probe also refutes `(Int, Int) <: (Number | String, Number | String)` with Int <: Number.

## Expected invariant

Independent sibling occurrences of a proven relation must remain provable. The active recursion stack must contain only currently active calls; candidate order cannot alter the relation.

## Root cause

A successful union member returns immediately, bypassing visited.remove at the function end. The second equal component sees the first component's leftover pair and returns CycleDetected despite no recursion. Other early exits deserve the same cleanup audit.

## Minimal reproducer

`tuple_union_source_accepts_both_components` and `repeated_union_obligation_is_not_recursion` in [the probes](evidence/semantic_audit_probe.rs).

## Impact

Valid tuple/record/callable/family structures with repeated union obligations can be falsely rejected. Allocation-order permutation is tested separately in the retained probe; its completed result is recorded in the evidence index.

## Fix direction

Make active-pair removal unconditional on every return, or use explicit enter/leave stack frames. Keep completed-result memoization separate from active recursion detection. Preserve cancellation/budget outcomes when revisiting candidates.

## Tests required

The two probes above plus `union_candidate_allocation_order_is_irrelevant`; repeat through records, callable parameters/results and family members. Check failed candidate followed by valid candidate and both allocation orders.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
