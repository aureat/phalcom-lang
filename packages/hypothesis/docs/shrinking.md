# Shrinking

Shrinking edits the immutable semantic `Example` and reruns the complete property. Strategies never own QuickCheck-style shrink trees.

Checkpoint 05 installs the authoritative structural shrinker over the typed choices and semantic spans introduced in Checkpoints 03 and 04.

## Strict complexity order

Every example is ordered by:

1. number of primitive choices;
2. structural span weight;
3. aggregate distance from choice shrink targets;
4. stable example signature.

An accepted candidate must strictly decrease this tuple. The engine therefore has an explicit termination argument in addition to `Settings.maxShrinks`.

## Pass order

The standard shrinker repeatedly applies:

1. discardable-span deletion;
2. trailing-choice shortening;
3. branch/index minimization;
4. individual integer and Boolean minimization;
5. related integer-block minimization;
6. bytes/text simplification;
7. recursive-branch collapse.

After one candidate is accepted, the shrinker restarts from the first pass.

## Structural deletion

List and set elements, map entries, and text characters are represented by discardable spans. Deleting one of these spans also reduces the enclosing `#length` choice and adjusts all later span ranges. This can remove an element from the middle while preserving choices for later elements.

Recursive expansion records a `#recursive` Boolean followed by a `#recursiveBranch` payload span. Collapsing a subtree changes the Boolean to the base case and deletes the payload range, leaving later sibling choices available for replay.

## Candidate classification

Replay candidates are accepted only when they:

- reproduce an interesting failure;
- preserve the exact `FailureOrigin`;
- normalize to a strictly smaller example.

Passing, invalid, overrun, flaky, and origin-changing candidates are not shrink successes.

`find` uses the same passes and ordering, accepting candidates that still satisfy the predicate instead of encoding success as an exception.
