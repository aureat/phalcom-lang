# Phase 11 Extension Hardening Design

## Objective

Phase 11 turns the package's existing structural protocols into dependable extension boundaries. It adds practical alternate implementations and conformance evidence while preserving the central architecture: strategies consume primitive choices through `DrawData`; immutable examples are replayed and structurally shrunk by one authoritative kernel.

## Provider architecture

Generation now resolves one `ChoiceProviderFactory` per search and asks it for a fresh `ChoiceProvider` per example. The default factory owns the seeded `Random`, so provider instances have isolated consumption counters while the random stream remains continuous and deterministic. Scripted factories select one immutable script by example index.

System-random, scripted, and replay providers share `_ChoiceNormalization`. This prevents an alternate provider from smuggling obsolete bounds, labels, or shrink targets into the recorded example. Provider exhaustion and invalid source values are `_ChoiceOverrun` outcomes.

## Strategy and shrink extensions

`Strategy<out T>` remains structural. `StrategyBase<T>` exposes the package's ordinary combinators as reusable implementation, but inheritance is optional.

`ShrinkPass` remains proposal-only. The central `Shrinker` owns uniqueness, complexity ordering, replay, invalid-candidate handling, and failure-origin preservation. Candidate identity combines the choice signature and span signature so deduplication cannot erase a distinct structural candidate.

## Database hardening

Database adapters preserve copy isolation and bounded newest-first deduplication. Directory save and delete operations enter one shared process-local exclusion guard, reread the latest visible bucket, merge the mutation, encode the bounded result, flush, close, and atomically replace the destination.

The lock fails closed on overlapping same-process modification. It does not claim cross-process exclusion because the current package baseline does not define a portable runtime file-lock primitive. Atomic replacement and corruption recovery remain the cross-process integrity boundary.

## Reporter failure boundary

Reporters receive synchronous typed events. `_CheckedReporter` wraps each delivery and converts extension exceptions into `ReporterFailure`. `SearchEngine.check` catches that exact extension error and returns `PropertyResult.Errored` with the statistics accumulated before failure. User failures are never constructed from reporter exceptions.

## Performance corrections

The implementation removes accidental quadratic behavior in source-visible hot paths:

1. scoped stacks use `removeAt(last)` rather than copying all preceding entries;
2. `ChoiceBuffer` reserves one closed-span slot at `beginSpan` and fills it at `endSpan`, so source-order extraction is one linear pass;
3. signatures accumulate parts and join once;
4. shrink candidates are deduplicated before evaluator replay.

All changes retain primitive choice order, semantic spans, immutable snapshots, failure origin, report ordering, and persistence behavior.

## Verification

Phase 11 adds provider, strategy, shrink-pass, database, and reporter conformance fixtures; provider-equivalence integration coverage; regression cases for every corrected defect; five benchmark workloads; and a mutation verifier that proves the source gate rejects removed guarantees.

When no `phalcom` executable is present, only source/static verification, mutation execution, archive integrity, and Python verifier behavior are observed. Runtime behavior and benchmark timing remain unobserved.
