# Phase 04 Strategy Model

Checkpoint 04 moves strategy generation out of the compatibility adapter and into the authoritative `src/strategies` slice.

## Ownership boundary

The strategy layer owns:

- the public covariant `Strategy<out T>` protocol;
- shared combinators;
- primitive and collection strategies;
- explicit composite draws;
- deferred and size-bounded recursive strategies;
- the `Gen` construction façade;
- exact built-in entries in `StrategyRegistry`.

It does not own:

- random choice supply;
- replay cursors;
- example evaluation;
- search phases;
- shrinking;
- property discovery;
- persistent databases;
- output formatting.

All primitive decisions cross the `DrawData` boundary. This keeps generation and replay observationally aligned and prevents strategies from introducing a second search model.

## Structural protocol and implementation base

`Strategy<out T>` is the public structural contract. `StrategyBase<T>` is the supported reusable implementation convenience that supplies `map`, `filter`, `flatMap`, and `named`; user strategies may conform structurally without inheriting it. The name was promoted from private `_StrategyBase<T>` in Phase 11 without changing structural conformance.

The protocol remains runtime-inert metadata. It does not insert checks or affect selector dispatch.

## Naming and spans

`named(label)` opens a non-discardable span and installs a scoped default choice label. An explicit label supplied by a nested strategy takes precedence. Scope cleanup uses `ensure`.

Collections create semantic spans independent of value representation. List element spans are discardable, which gives Phase 05 a direct structural deletion unit rather than forcing it to reverse-engineer list-length choices.

## Filtering and uniqueness

Filtering is choice-consuming search inside one example. Each failed predicate increments the draw-data rejection counter. Exhausting the filter attempt budget raises `_RejectedExample`, which is an invalid example.

Set and map uniqueness use the same local rejection accounting. Duplicate candidates do not become successful elements and do not falsify the property.

## Recursive sizing

`DrawData` now has scoped generation-size views. The frozen `Example.generationSize` retains the original outer size, while `DrawData.size` reports the current scoped size.

A recursive strategy:

1. draws its base immediately at size zero;
2. otherwise draws a Boolean expansion choice shrinking toward `false`;
3. passes a child strategy fixed at `size - 1` to the extension;
4. evaluates the extension under the reduced size.

This gives a finite recursive generator without encoding recursion depth in a separate random source.

## Float boundary

The Phase 03 choice algebra intentionally has no float primitive. Phase 04 therefore defines finite float generation as a deterministic integer encoding at six decimal places. This is a compatibility-preserving first implementation, not a claim that all IEEE-754 values are covered.

A future dedicated float choice may add NaN, infinities, signed zero, subnormals, and bit-level shrink semantics without changing the public `Strategy<Float>` surface.

## Registry boundary

The Phase 04 registry resolves exact built-in runtime type descriptors. It intentionally does not decompose applied reflective types. Type-argument recursion, data-class derivation, sealed-variant derivation, and parameter metadata belong to Phase 06.

## Compatibility adapter

The adapter imports `Strategy`, `Gen`, and the shared strategy base; all legacy primitive, combinator, collection, bytes, text, and build strategy classes were removed. Phase 11 names that reusable base `StrategyBase`; the temporary search runner and stateful scenario bridge continue to consume authoritative strategies.
