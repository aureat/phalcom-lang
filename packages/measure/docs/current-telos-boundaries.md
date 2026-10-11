# Current Telos boundaries

The skeleton avoids features not proven in the current implementation:

- type-based selector overloading,
- compiler-owned unit semantics,
- unit suffix syntax such as `5km`,
- type-level dimension exponent vectors,
- automatic `Rational` -> `Int` result narrowing,
- arbitrary user-defined const method execution.

The most valuable future feature would be checked user-defined const
evaluation, making declarations such as these possible without a compile-time VM:

```telos
const kilo = Prefix("k", Rational.pow10(3))
const km = kilo(m)
const N = LinearUnit.define(Force(), "N", kg * m / s**2)
```

Other desirable facilities: associated static values, class-side trait
conformance, standard heterogeneous arithmetic traits, immutable ordered maps,
standard `Rational`/`Decimal`, and derivable `Eq`/`Hash`/`Ord`.
