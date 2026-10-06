# Current strategies

The active kernel has a small static strategy interface. `Strategy<T>` produces
one value of `T` from a generic `DrawData<P>` context, where `P` conforms to
`ChoiceProvider`. The concrete strategy and provider types remain known through
property search; traits are not stored as runtime values.

## Implemented strategies

- `IntStrategy(min, max, shrinkTowards)` requests one inclusive integer choice.
- `BoolStrategy(shrinkTowards)` requests one Boolean choice.

Both strategies use the same `DrawData` API for generation and replay. Their
provider supplies typed `ChoiceRequest` values, which the buffer snapshots into
the resulting `Example` tape.

The provider layer also models `Index` and `Bytes` requests and has scripted,
replay, and seeded-random providers. Those choice kinds are not yet exposed by
a built-in strategy.

## Not implemented yet

The active package does not yet provide a `Gen` facade, strategy combinators,
filter/assume rejection, collection strategies, recursive strategies,
automatic derivation, or reflection-driven discovery. These features are
tracked as future work and are not part of the current API.

The old 0.1.0 strategy protocol and its combinator documentation are preserved
under `../archive/legacy-0.1.0/docs/strategies.md` as historical reference.
