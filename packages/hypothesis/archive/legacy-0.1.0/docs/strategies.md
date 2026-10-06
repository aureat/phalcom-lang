# Strategies

A strategy is a compositional description of how a value consumes semantic primitive choices. It does not own randomness, replay state, persistence, or shrinking.

```phalcom
protocol Strategy<out T> {
  draw(data: DrawData) -> T
  map<U>(transform: [T] -> U) -> Strategy<U>
  filter(predicate: [T] -> Bool) -> Strategy<T>
  flatMap<U>(transform: [T] -> Strategy<U>) -> Strategy<U>
  named(label: Symbol) -> Strategy<T>
  fingerprint -> String
}
```

Every built-in strategy draws exclusively through `DrawData`. Generation and replay therefore execute the same strategy code. Per-strategy shrink trees are not part of the model; the engine later transforms the immutable `Example` and reruns the full strategy/property.

## Standard façade

`Gen` supplies:

```phalcom
Gen.int
Gen.int(min: -100, max: 100)
Gen.bool
Gen.float
Gen.float(min: -10.0, max: 10.0)
Gen.bytes
Gen.bytes(minSize: 1, maxSize: 32)
Gen.text
Gen.text(alphabet: Gen.sampledFrom(const [65, 66]), minSize: 1, maxSize: 8)

Gen.just(value)
Gen.sampledFrom(values)
Gen.oneOf(first, second)
Gen.option(strategy)
Gen.result(ok: successStrategy, error: errorStrategy)
Gen.list(of: strategy)
Gen.set(of: strategy)
Gen.map(keys: keyStrategy, values: valueStrategy)
Gen.tuple(first, second)
Gen.build { draw => ... }
Gen.deferred { ... }
Gen.recursive(base: baseStrategy, extend: { child => ... })
```

`map`, `filter`, `flatMap`, and `named` are ordinary strategy messages.

## Primitive semantics

- Unbounded `Gen.int` draws from `[-2^size, 2^size]` and shrinks toward zero.
- Bounded integers shrink toward zero when zero is in range, otherwise toward the nearest bound.
- `Gen.bool` uses a typed Boolean choice and shrinks toward `false`.
- Phase 04 floats are finite, bounded, and quantized to six decimal places. They are represented by normalized integer choices with scale `1_000_000`; NaN and infinities are intentionally outside this initial surface.
- `Gen.bytes` uses one typed bytes choice, not one integer choice per byte.
- Default `Gen.text` uses printable ASCII code points `32...126`. A custom code-point strategy supplies other alphabets.
- `sampledFrom` and `oneOf` use typed index choices that shrink toward the first item/branch.

## Collection spans

A list opens a non-discardable `#list` span. Every generated item opens a discardable `#element` span. Sets, maps, tuples, and text use analogous structural spans.

Set elements and map keys must be unique. Duplicate candidates are recorded as local rejections. Exhausting the uniqueness budget invalidates the example rather than producing a counterexample.

## Composite and recursive strategies

`Gen.build` receives a `Draw` object:

```phalcom
const interval = Gen.build { draw =>
  const start = draw.from(Gen.int)
  const width = draw.from(Gen.int(min: 0, max: 100))
  Interval.new(start: start, end: start + width)
}
```

`Gen.deferred` resolves its factory only when drawn, enabling mutually recursive declarations.

`Gen.recursive` uses `DrawData.size`. At size zero it must draw the base strategy. An expanded child observes `size - 1`, so recursive expansion terminates without a separate generator or hidden random source.

## Rejections

A failed filter candidate increments `DrawData.rejectionCount`. If no candidate satisfies the predicate within the configured attempt budget, the strategy raises `_RejectedExample`; `DrawData.attempt` classifies that as `ExampleStatus.Invalid`, never `Interesting`.

## Registry

`StrategyRegistry.standard` contains exact reflective entries for:

- `Int`
- `Bool`
- `Float`
- `Bytes`
- `String`

Generic decomposition such as `List<Int>` and `Option<String>` is implemented with reflected type arguments during Phase 06. Registering a custom exact type is already supported:

```phalcom
registry.register(UserId, use: userIds)
```
