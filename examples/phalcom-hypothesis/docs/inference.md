# Type-Driven Strategy Inference

Type annotations are reflective metadata. They do not add automatic runtime checks, affect dispatch, or change selector identity.

## `@Given` modes

The property layer supports exactly three forms.

### Fully inferred

```phalcom
@Given
property(value: Int, items: List<String>, point: Point) {
  ...
}
```

Every inferred parameter must have a reflected type annotation that the active `StrategyRegistry` can resolve.

### Fully explicit

```phalcom
@Given(Gen.int, Gen.list(of: Gen.text))
property(value: Int, items: List<String>) {
  ...
}
```

The strategy count must equal reflected parameter arity exactly.

### Named partial overrides

```phalcom
@Given(
  GivenArgs.new()
    .for(#value, use: Gen.int(min: 1, max: 100))
)
property(value: Int, items: List<String>) {
  ...
}
```

Overrides bind by reflected parameter name. Every remaining parameter is inferred. Duplicate and unknown names are discovery errors.

## Resolution precedence

`StrategyRegistry` resolves a type in this strict order:

1. an exact `register(type:, strategy:)` entry;
2. an exact entry installed from a zero-argument `@strategy(Type)` provider method;
3. a cached derived strategy;
4. a built-in applied-type decomposition;
5. opt-in automatic derivation for a type marked `@arbitrary`.

Exact entries therefore always override automatic derivation. The compatibility spelling `register(type:, use:)` delegates to the canonical `strategy:` form.

An annotated provider is installed explicitly:

```phalcom
class DomainStrategies {
  @strategy(UserId)
  userIds() -> Strategy<UserId> {
    return Gen.just(UserId.new(value: 42))
  }
}

const registry = StrategyRegistry.standard.register(DomainStrategies)
```

The attribute is passive metadata. It does not mutate a process-global registry.

## Standard registry

`StrategyRegistry.standard` contains exact entries for:

- `Int` → `Gen.int`;
- `Bool` → `Gen.bool`;
- `Float` → `Gen.float`;
- `Bytes` → `Gen.bytes`;
- `String` → `Gen.text`.

It recursively decomposes canonical applied types:

- `Option<T>` → `Gen.option(strategy(T))`;
- `List<T>` → `Gen.list(of: strategy(T))`;
- `Tuple<A, B, ...>` → `Gen.tuple(strategy(A), strategy(B), ...)`;
- `Set<T>` → `Gen.set(of: strategy(T))`;
- `Map<K, V>` → `Gen.map(keys: strategy(K), values: strategy(V))`;
- `Result<T, E>` → `Gen.result(ok: strategy(T), error: strategy(E))`.

Applied-type decomposition expects reflective descriptors exposing `origin` and immutable `arguments`.

## Automatic derivation

Automatic derivation is opt-in:

```phalcom
@arbitrary
@data
@immutable
class Point {
  const _x: Int
  const _y: Int
}
```

The registry selects the single reflected constructor, reads its ordered parameter metadata, recursively resolves every parameter annotation, and invokes that constructor with generated values. Parameter names and labels participate in the derived fingerprint so constructor-shape changes invalidate persistent examples.

Ordinary classes without `@arbitrary` are never guessed. A derivable constructor must:

- be the only reflected constructor;
- have a reflected type for every parameter;
- have no rest parameter;
- have no reflected precondition contract.

### Sealed variants

An opt-in sealed hierarchy derives a stable `oneOf` over its reflected variants:

```phalcom
@arbitrary
@data
@sealed
class Token {
  @variant Integer(value: Int)
  @variant Name(text: String)
}
```

Variants are sorted by stable class name before strategy construction.

Recursive sealed hierarchies are partitioned into terminal and recursive variants. Terminal variants form the base strategy; recursive variants are added through `Gen.recursive`, so generation size reaches a guaranteed terminal case at size zero.

```phalcom
@arbitrary
@data
@sealed
class Expression {
  @variant Literal(value: Int)
  @variant Negate(value: Expression)
  @variant Add(left: Expression, right: Expression)
}
```

Recursive references nested inside `Option`, `List`, `Set`, `Tuple`, `Map`, or `Result` are replaced with the size-bounded child strategy. A recursive hierarchy with no terminal variant is rejected before search.

## Constrained constructors

Arbitrary constructor contracts are not converted into rejection filters. Doing so would make performance and validity depend on an opaque predicate and could create pathological discard rates.

A constructor with `@requires` or equivalent reflected preconditions is rejected with a diagnostic recommending an exact registration or `@strategy(Type)` provider. The package does not attempt symbolic contract solving.

## Resolution paths

Nested failures include the complete resolution path. A failure may identify:

```text
Envelope -> new(payload:).payload -> List element Opaque
```

This distinguishes the unsupported leaf type from the outer property parameter and every constructor/container step that led to it.

## Diagnostics

Discovery stops before search for:

- missing parameter annotations in inferred positions;
- unsupported or unapplied type descriptors;
- malformed generic arity;
- explicit strategy-count mismatch;
- unknown or duplicate named overrides;
- `@Case` values whose arity differs from method parameters;
- multiple `@Given` or `@WithSettings` attributes on one property;
- unmarked domain classes;
- ambiguous, missing, rest-parameter, or constrained constructors;
- recursive sealed hierarchies without a terminal variant;
- malformed or duplicate annotated providers.

Diagnostics include the property identity and parameter name where applicable, plus the registry resolution path for nested model derivation.
