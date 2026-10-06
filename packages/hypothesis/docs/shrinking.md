# Current shrinking

Shrinking edits the immutable `Example` choice tape and replays the complete
strategy and property. A candidate is considered only when its
`ExampleComplexity` is strictly smaller than the current tape.

## Primitive pass

`IntegerShrinkPass` handles integer and index choices. It proposes, in stable
order:

1. the configured target;
2. in-domain offsets from the target up to 16 in both directions;
3. repeated halving probes from the current value toward the target.

Candidates that do not move closer, leave the requested domain, or duplicate
an earlier candidate are omitted. Boolean choices move directly to their
configured target. Byte choices are not shrunk yet.

After the first same-origin failure is accepted, the shrinker restarts the pass
from the new tape. It stops when no candidate is accepted or when
`Settings.maxShrinkSteps` accepted improvements have been made. Each candidate
evaluation is counted separately from accepted improvements. The final tape is
replayed twice in `Verify` phase before returning `Falsified`.

## Failure identity and replay

A failure origin contains both a category and the identity symbol from its
`PropertySpec`. Predicate `false` and a predicate exception have distinct
categories. Shrinking accepts only a falsification with the same category and
property identity. Replay providers report unconsumed tape entries, so a
candidate that leaves choices unused cannot silently pass verification.

Structural spans, deletion of collection elements, byte simplification, and
recursive-branch collapse are not implemented. Their former design is kept in
the archived 0.1.0 material at
`../archive/legacy-0.1.0/docs/shrinking.md`.
