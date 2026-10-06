---
status: BLOCKED
completion: PARTIAL
verification: FOCUSED_TESTED
reported: 2026-10-06
evidence_revision: ad1841f2
---

# Implementation incident: generic strategy dispatch is blocked

## Summary

The Hypothesis package now keeps strategy and provider types statically typed
through `Property.check`, `SearchEngine`, and `DrawData<P>`. The vertical
property-search fixture compiles, starts running, and then fails when the VM
tries to invoke the generic trait method `Strategy.draw<P>`. A separate replay
probe panics during compiler lowering while resolving the owner of a
conformance callable.

These are two observed failures at adjacent generic-conformance boundaries.
Their common root cause has not been established. They block a passing
generation/replay/shrink/verification flow. The package's choice, settings,
failure, complexity, and integer-shrink utility fixtures pass independently.

## Lifecycle state

| Axis | Value | Evidence |
| --- | --- | --- |
| Status | `BLOCKED` | The vertical search and replay paths depend on generic conformance dispatch that currently fails. |
| Completion | `PARTIAL` | Package fixes and fixtures exist; the property-testing kernel has no passing vertical result. |
| Verification | `FOCUSED_TESTED` | The active runtime corpus and the replay blocker probe were run; one active fixture failed and one remains blocked. |

The package verification ledger is
[LANG007.C4.P4.R1-hypothesis-runtime-test-results.md](LANG007.C4.P4.R1-hypothesis-runtime-test-results.md).
The compiler lifecycle cross-reference is in the
[LANG007.C3 checkpoint](../../../../../../../docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md).

## Reproduction and observed failures

Evidence was collected on 2026-10-06 at revision `ad1841f2`, using the existing
`target/debug/phalcom` executable. The package fixtures are compiled and run by
the runtime harness.

### A. Generic `Strategy.draw` has no active runtime conformance target

Fixture:
[`deterministic-property-search-test.ph`](../../../../../../../packages/hypothesis/tests/runtime/src/engine/deterministic-property-search-test.ph),
at its `PropertyResult::Errored` assertion.

Command:

```sh
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --phalcom target/debug/phalcom
```

The full active corpus reported **9 passed, 1 failed, 1 blocked**. The
deterministic property-search fixture compiled and entered execution, then
returned `Errored` with this runtime failure:

```text
no active conformance target for requirement
TraitRequirementId {
  owner: DeclarationId {
    module: ModuleId {
      project: Resolved(ResolvedProjectId(1)),
      path: strategies.strategy
    },
    name: "Strategy"
  },
  selector: Selector {
    base: Named("draw"),
    kind: Method,
    slots: [Label("drawData")]
  },
  side: Instance
}
```

The VM's `invoke_trait_requirement` reads the active frame's conformance
environment and looks up the selected requirement slot. The lookup fails at
[`dispatch.rs`](../../../../../../../phalcom-core/src/vm/dispatch.rs#L1553). This proves the
required slot is unavailable in the active invocation frame; it does not yet
identify which compiler or VM transition failed to provide it.

The source path under test is:

```text
Property.check
  -> SearchEngine.run
  -> Evaluator.evaluate
  -> PropertySpec.strategy.draw<Provider>
  -> Strategy<Int> conformance requirement
```

The failing fixture expects the scripted value `37` to shrink to `10`, then to
confirm the same failure twice in `Verify`. Those shrink and verification
assertions are not reached.

### B. Integer replay panics on a conformance-owned callable

Fixture:
[`integer-replay-test.ph`](../../../../../../../packages/hypothesis/tests/runtime/src/blocked/strategies/integer-replay-test.ph).

Command:

```sh
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --probe-blocked \
  --phalcom target/debug/phalcom \
  packages/hypothesis/tests/runtime/src/blocked/strategies/integer-replay-test.ph
```

The focused probe reported **still blocked**, with this compiler panic:

```text
phalcom-semantic/src/identity.rs:303:43:
conformance callable has no declaration owner:
ImplId { module: strategies.primitives, local: ImplLocalId(2) }
```

`CallableOwnerId::Conformance` is intentionally identified by `ImplId`; it has
no `DeclarationId`. The declaration-only accessor panics for that owner kind
at [`identity.rs`](../../../../../../../phalcom-semantic/src/identity.rs#L303). The caller
that reached the accessor in this compilation path still needs to be traced.

The blocked marker now matches the observed panic signature. This probe did
not establish a passing direct strategy replay.

## Issues that need fixing

These are repair targets derived from the evidence, not a claim that the exact
faulting transition has already been located:

1. **Preserve generic conformance evidence through trait-method invocation.**
   Find where the `Strategy.draw<P>` requirement slot should be installed and
   where it becomes unavailable before `invoke_trait_requirement` reads the
   active frame. Keep the selected conformance and provider type evidence
   canonical through that call.
2. **Handle conformance callable ownership without assuming a declaration
   owner.** Trace the caller of `CallableOwnerId::declaration()` and route
   conformance-owned callables through their `ImplId`/module ownership, or
   return a structured error where declaration ownership is genuinely
   required. Compilation must not panic on a valid conformance-owned callable.
3. **Prove both paths with minimal discriminators.** A generic trait method
   should run with a concrete provider, and `IntStrategy.draw` should replay
   from a scripted choice without a VM conformance-slot failure or compiler
   panic.
4. **Restore the vertical Hypothesis acceptance.** Run deterministic search
   through generation, shrinking, and both verification replays. The existing
   fixture must minimize `37` to `10`, report `Phase::Verify`, and preserve its
   statistics. Once replay succeeds, move its fixture out of the runner's
   `blocked/` tree and run the complete package corpus with all 11 fixtures
   passing and none blocked.

## Scope and disposition

The scope for this incident is package-only: record the compiler/runtime
blocker without changing compiler or VM production code. The exact underlying
cause, its relation (if any) to earlier conformance-environment failures, and
its owning follow-up plan are not established here. Do not classify it as an
inherited baseline or claim a compiler regression without a clean-predecessor
comparison.

Until an owner and plan are assigned for the compiler/runtime repair, the
Hypothesis kernel remains `BLOCKED / PARTIAL / FOCUSED_TESTED`; its utility
fixtures are executable, but generation, replay, shrinking, and verification
do not have a passing end-to-end result.
