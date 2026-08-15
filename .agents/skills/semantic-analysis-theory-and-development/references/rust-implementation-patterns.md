# Rust Implementation Patterns for Semantic Engines

## 1. Represent semantic ownership in types

Rust can make semantic invariants cheap if the data model uses typed IDs, immutable published data, and clear generation ownership. It can also make architecture painful if long-lived references point into mutable tables or if every pass clones large nested structures to satisfy the borrow checker.

Prefer IDs across semantic boundaries:

```rust
struct BindingId(u32);
struct CallableId(/* logical/interned key */);

fn fact(&self, id: BindingId, point: ProgramPoint) -> Option<&Fact>;
```

rather than storing `&BindingInfo` in another long-lived table. IDs break borrow cycles and make snapshot serialization/indexing possible.

## 2. Mutable worker, immutable publication

A robust pattern is:

```text
single writer / worker owns mutable candidate state
    -> computes replacement products
    -> cancellation check
    -> publishes Arc<SemanticSnapshot>
readers clone Arc and query immutable structures
```

**CURRENT:** Phalcom's semantic engine already uses this pattern with `Arc` state, candidate cloning/copy-on-write, and an `RwLock<Arc<SemanticSnapshot>>` publication point. Preserve the coherence property even if storage changes.

## 3. Avoid raw-reference graphs

This is tempting:

```rust
struct Use<'a> { binding: &'a BindingInfo }
```

but it couples lifetimes across mutable incremental storage. Prefer:

```rust
struct Use { binding: BindingId }
```

and resolve within the snapshot. This makes replacing a file/module table tractable and makes stale-generation checks possible.

## 4. Arenas and indexed storage

Use arenas when:

- many edges refer to immutable semantic entities;
- IDs are compact/hot;
- bulk generation reclamation is acceptable;
- deterministic iteration can be controlled.

Use `Vec<T>`/typed index for dense generation-local data, `BTreeMap` where deterministic key order is useful and cardinality modest, and hash maps only when profiling justifies them. Deterministic diagnostics/tests are a semantic engineering benefit, not mere aesthetics.

## 5. Interners

Intern selector/name/module strings only with an explicit lifetime and memory policy. Process-global interners can grow unbounded during editor sessions with generated/edited identifiers. Workspace-generation or reclaiming interners may be safer.

An interned ID should not leak interner implementation into public semantics. Debug rendering should still recover canonical names.

## 6. Copy-on-write costs

`Arc::make_mut` is effective for snapshot reuse until mutating one entry clones a very large map. Measure:

```text
candidate-state clones
COW map clones / bytes
Arc strong counts
per-edit allocations
```

If clone amplification is hot, consider per-module maps, persistent collections, arenas with replacement slabs, or a query database. Do not replace clear COW architecture based on theoretical concerns alone.

## 7. Dependency storage

Keep forward and reverse edges when invalidation needs both:

```rust
struct Dependencies {
    forward: BTreeMap<CallableId, BTreeSet<CallableId>>,
    reverse: BTreeMap<CallableId, BTreeSet<CallableId>>,
}
```

Update both transactionally. Tests should check graph consistency. For large graphs, compact adjacency vectors/interned IDs may reduce allocations.

## 8. Bounded domains

Any collection-valued abstract domain can explode. Make bounds part of the representation/policy:

```rust
const MAX_SHAPE_UNION: usize = 8;
```

**CURRENT:** `ValueShape` uses a bounded union and widens to `Unknown`. Future domains need their own justified limits. Record metrics for widening frequency; a cap that triggers constantly is a precision bug even if memory is bounded.

## 9. Error and uncertainty types

Do not use `Option<T>` for every semantic failure. `None` cannot distinguish missing, ambiguous, blocked, cancelled, or unreachable. Use enums:

```rust
enum Resolution<T> {
    Resolved(T),
    Missing,
    Ambiguous(Vec<T>),
    Blocked(DependencyId),
    Recovery,
}
```

Cancellation should generally abort candidate computation, not become an ordinary semantic fact.

## 10. Concurrency

Prefer immutable snapshots to fine-grained locks around semantic tables. If parallel analysis is introduced, define ownership of each fact/summary and merge deterministically. Avoid global locks in completion/reference hot paths.

A parallel solver must preserve fixed-point semantics independent of scheduling; use monotone joins, deterministic work queues where output order matters, and transactional publication.

## 11. Source ranges

Store ranges as provenance/location data with explicit file/source identity. Never assume a `SourceRange` alone globally identifies a declaration. Conversions between byte offsets and LSP UTF-16 positions belong in adapters.

## 12. Unsafe/native boundaries

Semantic infrastructure should normally be safe Rust. If FFI/native metadata uses unsafe code, encapsulate it behind immutable semantic contracts/owned descriptors. Never keep raw pointers to VM objects inside long-lived LSP snapshots without a rooting/lifetime design.

## 13. Review questions

1. Could this reference become stale after mutation?
2. Should this edge store an ID instead?
3. What generation owns this arena/interner ID?
4. What bounds union/interner/cache growth?
5. Are COW clones measured?
6. Are forward/reverse dependencies updated consistently?
7. Does `Option` erase a semantically important failure reason?
8. Can a reader query without taking a global mutation lock?
