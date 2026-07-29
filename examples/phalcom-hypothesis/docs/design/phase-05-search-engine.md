# Phase 05 Search Engine and Structural Shrinker

Checkpoint 05 moves property evaluation, phase ordering, shrinking, final replay verification, and `find` out of `_internal/legacy_adapter.ph` and into the authoritative `src/engine` slice.

## Ownership boundary

The engine owns:

- immutable `PropertySpec<T...>` descriptions;
- complete-property evaluation through generation and replay;
- explicit, reuse, generation, shrink, and verification phases;
- deterministic example complexity ordering;
- ordered structural shrink passes;
- stable `FailureOrigin` preservation;
- flaky/non-reproducible classification;
- engine-level `find` using the same evaluator and shrinker.

The engine does not own:

- reflective `@Given` discovery or annotation inference;
- public builder APIs;
- persistent database formats or directory storage;
- reporter events and targeting;
- stateful bundles, rule applicability, or action-reference shrinking.

Those remain later phases. The compatibility adapter now delegates search to `SearchEngine` and retains only temporary discovery, in-memory database bridging, reporting, and stateful metadata/execution.

## Evaluation model

`_Evaluator` draws every strategy through `DrawData`, invokes the complete target, and classifies the result as:

- `ExampleStatus.Valid`;
- `ExampleStatus.Invalid` for `_RejectedExample`;
- `ExampleStatus.Overrun` for replay/choice-budget/health-check engine conditions;
- `ExampleStatus.Interesting` with an immutable `Failure`.

Generation and replay use the same evaluator. Replay normalizes retained choices against the current requests, so stale examples become invalid or overrun cache misses rather than counterexamples.

Find mode uses `_SearchResult.Found` as a value result. It does not throw or catch a fake success exception.

## Phase ordering

`SearchEngine.check` applies the approved order:

1. explicit examples;
2. supplied reuse examples;
3. generation;
4. structural shrinking;
5. final replay verification.

A failing explicit example returns immediately and is not shrunk. Invalid or overrun reuse examples are ignored as stale. Generation overrun is an engine error. Discard exhaustion is inconclusive.

The accepted minimal failure is replayed twice. Both replays must remain interesting with the same source-aware `FailureOrigin`; otherwise the property is `Errored` with `_FlakyFailure`.

## Complexity ordering

`ExampleComplexity` is a deterministic lexicographic tuple:

1. primitive choice count;
2. structural span weight;
3. distance of choices from their declared shrink targets;
4. stable example signature.

Every accepted shrink must be strictly less than the current example under this ordering. The shrinker records accepted complexities for testing and diagnostics. Equal or greater candidates are never evaluated as accepted shrink steps.

## Ordered passes

`Shrinker.standard` runs these passes in order and restarts from the first pass after each acceptance:

1. delete discardable spans;
2. shorten trailing choices;
3. minimize branch/index choices;
4. minimize individual integer and Boolean choices;
5. minimize contiguous integer blocks;
6. simplify bytes and text-related choices;
7. collapse recursive branches and delete their semantic payload spans.

A candidate is accepted only when replay is interesting, preserves the original failure origin, and produces a strictly smaller normalized example. Invalid, overrun, passing, or origin-changing candidates are ignored.

Find shrinking uses the same pass sequence and complexity ordering, but accepts candidates that continue satisfying the predicate.

## Middle-span deletion

`Example.deleteRange` returns a new example, removes the selected choice range, shifts later spans, contracts ancestors, and drops spans fully contained in the removed range.

For discardable list/set elements, map entries, and text characters, `_DeleteDiscardableSpans` also decrements the nearest enclosing `#length` integer choice. This permits deleting a middle element while retaining later choices. Prefix truncation cannot provide that behavior.

## Recursive structures

Expanded recursive strategies now open a discardable `#recursiveBranch` span after the `#recursive` Boolean decision. `_MinimizeRecursiveStructures` changes that decision to `false` and deletes the expanded branch payload while retaining later sibling choices. Replay then reconstructs the base case at that position.

## Failure origins

`Failure.from` preserves an error-provided `failureOrigin` when available and otherwise retains the existing unknown source origin. The shrinker compares `Failure.sameOrigin`, never only the exception class or message.

Full automatic traceback-frame extraction depends on the corresponding Phalcom reflection surface and could not be executed in this checkpoint environment. The engine contract is already source-origin-aware and tests use explicit origins to verify preservation.

## Compatibility bridge

The adapter no longer declares `_LegacyEngine`, `_LegacyPropertySpec`, `_LegacyFailureSignature`, the greedy choice-local shrinker, or `_LegacyFoundExample`.

`Property.forAll`, `Property.find`, reflective `PropertyRunner`, and the temporary stateful runner construct authoritative engine specifications and call `SearchEngine`. The adapter may supply and record examples through its temporary memory database; final database keying and codecs remain Phase 08 work.
