# Phase 03 Semantic Choice Model

Checkpoint 03 establishes the immutable example representation shared by generation, replay, future serialization, and structural shrinking.

## Typed primitive choices

`Choice` and `ChoiceRequest` are sealed immutable families with four variants:

- `Integer`: exact integer value, inclusive bounds, shrink target, and optional label;
- `Boolean`: Boolean value, shrink target, and optional label;
- `Index`: branch/index value, domain size, shrink target, and optional label;
- `Bytes`: byte sequence, size bounds, shrink target, and optional label.

Requests describe the domain offered to a provider. A provider returns a normalized `Choice` carrying the current request metadata. Replay therefore reuses the recorded primitive value while updating bounds, labels, and shrink targets to match the current draw request.

Integer and index choices retain the temporary simplification API used by the compatibility shrinker. Structural shrinking moves to dedicated passes in Phase 05.

## Immutable examples

`ChoiceBuffer` is a mutable construction worker. It records choices and nested semantic spans, then freezes them into an immutable `Example`.

An `Example` contains:

- normalized primitive choices;
- semantic spans in stable opening/source order;
- the generation size used by size-sensitive strategies.

The freeze boundary copies choice and span containers. Public accessors return copies rather than the retained lists. Byte values and byte shrink targets are copied at ordinary API boundaries so mutation of an input or returned `Bytes` buffer does not alter the recorded choice through the standard `Choice` API.

`Span` uses a half-open `[start, end)` choice range, a stable numeric identifier, an optional parent identifier, a semantic label, and a discardable flag. `ChoiceBuffer.withSpan(...)` closes spans through `ensure`, including when the enclosed draw throws.

## Providers and replay

`ChoiceProvider` is the structural provider protocol:

```phalcom
protocol ChoiceProvider {
  choose(request: ChoiceRequest) -> Choice
  consumedChoices -> Int
}
```

`_RandomChoiceProvider` is the only Phase 03 provider that consults `Random`. `_ReplayChoiceProvider` is a deterministic cursor over an existing `Example`; its implementation has no random dependency.

> **Phase 11 note:** the private random provider remains only as a compatibility alias. The authoritative public implementation is `SystemRandomChoiceProvider`; `ScriptedChoiceProvider` supplies deterministic extension input, and system, scripted, and replay providers all pass source values through the same `_ChoiceNormalization` path.

`DrawData` is the strategy-facing primitive interface. It provides typed draw operations for integers, Booleans, indices, and bytes, records the normalized choices in a buffer, and exposes the completed immutable example.

Replay exhaustion, request/type mismatch, changed request bounds, choice-budget exhaustion, and unclosed spans are `_EngineOverrun` conditions. `DrawData.attempt(...)` maps these to `ExampleStatus.Overrun`, never `ExampleStatus.Interesting`.

## Compatibility boundary

The Phase 01 compatibility engine no longer owns choice, tape, generation-data, or replay-data classes. It uses `Choice`, `Example`, and `DrawData` from `src/choices` while retaining its strategy, search, database, and stateful implementations until their dedicated phases.
