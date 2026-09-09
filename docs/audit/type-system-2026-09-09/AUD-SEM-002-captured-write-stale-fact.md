# AUD-SEM-002 — Invoked closure leaves a stale established Int fact

## Classification

**Severity:** Critical. **Category:** Soundness. **Status:** Confirmed; see the evidence qualification below. Audited HEAD `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09. No implementation fix.

## Location

`phalcom-semantic/src/checker/expression.rs:557–567` (restore outer flow after closure construction); `phalcom-semantic/src/checker/call.rs:2158,2696,2767` (call invalidation); `phalcom-semantic/src/checker/flow/state.rs:822` (`invalidate_opaque_calls`).

## Observed behavior

The analyzer accepts `run() -> Int` even after an invoked closure assigns String to its captured `Int | String` local. Runtime prints `changed`. The inspection probe retained x.current as Established Int, mutable=true, version=0, consistency=Validated.

## Expected invariant

After a call that can mutate a captured cell, current value knowledge must reflect that write or be weakened to the valid persistent contract. A previous initializer fact is not a post-call proof.

## Root cause

Restoring outer flow when constructing a closure is correct. At invocation, however, invalidation removes mutable predicates and field-current facts while leaving mutable local BindingState.current untouched. The captured assignment is legal under the union contract, so no earlier error blocks the later false Int proof.

## Minimal reproducer

[captured-write.ph](captured-write.ph)

## Impact

A legal mutation breaks the analyzer's returned type with no Dynamic annotation, invalid assignment or speculative aliasing. Existing predicate invalidation alone does not protect the separate current-knowledge channel.

## Fix direction

Carry captured-write effects into invocation or conservatively invalidate current knowledge of potentially written cells, preserving declared contracts and causal state. Do not apply closure writes at construction; do not erase immutable unrelated facts. Reconcile exact known synchronous writes separately from opaque calls.

## Tests required

`captured_write_must_invalidate_previous_int_fact`; assert post-call current type/evidence and return summary. Add never-invoked closure, nested helper invocation, conditional invocation, alias to closure, same-type write and a union-contract mutation. Existing `flow_loops::captured_block_write_is_not_applied_until_execution_is_proven` covers construction only.

## Evidence and limitations

See [retained evidence](evidence/README.md) for exact probes, commands and completed results. Semantic invariant failures are expected audit failures, not fixed behavior. IDs 001–002 have source-analysis plus executable runtime evidence; 003 has static and runtime evidence; 004–006 have pure/source semantic evidence; 007 has an executed pair-accounting counterexample and source-led depth concerns. No workspace-release certification is claimed.
