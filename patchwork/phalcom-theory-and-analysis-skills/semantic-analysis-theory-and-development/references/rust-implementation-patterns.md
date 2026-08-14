# Rust Implementation Patterns for Semantic Infrastructure

## Typed newtypes

Prefer distinct IDs:

```rust
struct ModuleId(...);
struct ClassId(...);
struct TypeId(u32);
```

Do not pass raw `u32`/`String` across APIs when concepts differ.

## Arenas and side tables

Use arena IDs plus side tables:

```text
ExprId -> SourceMap
ExprId -> TypeFact
ExprId -> ConstFact
BlockId -> FlowState
```

This keeps IR immutable while analyses attach independent data.

## Immutable publication

Build/mutate worker state, then publish `Arc<Snapshot>`. Query methods take `&self` and avoid locking per tiny fact when data is immutable.

## Deterministic containers

Use `BTreeMap/BTreeSet` or sort outputs when order reaches snapshots/tests/diagnostics. Hash maps can be used internally on hot paths if nondeterminism is normalized.

## Borrow boundaries

Avoid storing references across arena/state owners. IDs make cloning cheap and reduce lifetime tangles.

## Small data

Use compact enums/IDs/ranges. Avoid cloning whole AST nodes into every occurrence/fact. Keep `Arc<Program>` per module if source AST retention is needed.

## Error handling

User-source uncertainty returns semantic enums/options/results; `panic!/expect` only for construction invariants. Distinguish poison/internal infrastructure errors from semantic invalidity.

## Recursion guards

Recursive relation/graph algorithms use explicit visited/SCC/memo state rather than call-stack depth limits.

## Instrumentation

Counters/tracing around:

```text
files/callables rebuilt
flow passes
fixed-point iterations
query latency
allocations/union widening
```

Keep test-visible rebuild traces where useful.

## Documentation

Repository requires rustdoc on public API. Semantic invariants belong in module/item docs, not only external skill files.
