## Match Lowering Optimizations — Deferred

Initial Part 05.2 lowering should prioritize correctness and preserve the semantic/backend boundary using straightforward ordered `IsVariant` tests.

Backlog optimizations:

- **Variant jump tables**
  - Dispatch directly from runtime variant/discriminant to the matching arm.
  - Avoid repeated linear `IsVariant` tests for large closed enums.

- **Decision DAG factoring**
  - Compile overlapping/nested patterns into a shared decision graph.
  - Reuse common tests and payload projections across arms and or-pattern alternatives.

- **Shared-prefix factoring**
  - Avoid repeating prefixes such as:
    ```phalcom
    Some(Ok(x))
    Some(Error(e))
    ```
  - Test `Some` and extract its payload once, then branch on the nested case.

- **Payload extraction reuse**
  - Cache/stage an extracted payload when multiple downstream tests need the same field.
  - Avoid duplicate `GetVariantPayload` instructions.

- **Candidate-set optimization**
  - Optimize broad patterns such as `Animal::Dog*` / `Animal::Dog(...)` that expand to many exact `VariantId`s.

These are strictly backend optimizations. They must consume the same resolved semantic pattern/candidate information and must never introduce compiler-side variant lookup, selector resolution, exhaustiveness reasoning, or GADT solving.