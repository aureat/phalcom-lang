# Semantic Identities, Arenas, Interning, and Lifetime

## 1. Identity is a semantic relation

An ID is not merely a performance optimization. Choosing an identity relation determines when two references are considered to denote the same semantic entity, what survives edits, and how caches/dependencies compose.

Distinguish at least:

```text
textual equality        same spelling
source identity         same authored syntax node/revision location
semantic identity       same declaration/entity according to language rules
revision identity       same entity-version in an analysis generation
runtime identity        same allocated object during execution
representation identity same Rust allocation/index
```

These can coincide, but relying on that coincidence creates brittle architecture.

## 2. Current Phalcom IDs

**CURRENT:** `phalcom-lsp/src/semantic/ids.rs` uses module-qualified structural IDs:

```rust
ModuleId(String)
ClassId { module: ModuleId, name: String }
CallableId { owner: ClassId, selector: String, side: DispatchSide }
FieldId { owner: ClassId, name: String, side: DispatchSide }
```

This is already much safer than unqualified strings. It encodes key semantic dimensions: module namespace and instance/class dispatch side. Lexical bindings use a separate `BindingId` in scope machinery. These IDs should be understood as the **current semantic identity scheme**, not an eternal requirement that all IDs remain string-backed.

## 3. Structural IDs versus interned IDs

A structural ID stores its semantic key directly:

```rust
struct ClassId {
    module: ModuleId,
    name: String,
}
```

Advantages: transparent debugging, serialization, deterministic ordering, simple construction. Costs: repeated string storage/cloning/hashing and potentially expensive map keys.

An interned ID stores a compact index:

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ClassId(u32);

struct ClassData {
    module: ModuleId,
    name: Symbol,
    declaration: SourceAnchor,
}
```

Advantages: cheap copy/hash/equality and shared metadata. Costs: lifetime/generation management, reverse lookup, stale-ID hazards, interner memory growth, and harder persistence across sessions.

Do not migrate to integer IDs simply because they are faster. First decide **identity lifetime**.

## 4. Identity lifetime classes

A useful taxonomy:

- **ephemeral node ID:** valid only for one parse tree/revision;
- **file-generation semantic ID:** valid while a file snapshot is alive;
- **workspace-generation ID:** valid inside one coherent semantic snapshot;
- **stable logical ID:** intended to survive ordinary edits if declaration identity is semantically unchanged;
- **persistent external ID:** serialized across process runs/package indexes.

An arena index is naturally generation-scoped. A module-qualified structural key can be stable across generations. If an incremental engine wants compact arena IDs plus cross-generation stability, use an explicit mapping rather than pretending the arena index itself is permanent.

## 5. Source anchors and edit stability

Offsets are poor stable identity because insertions before a declaration move them. Names alone are poor identity because overload-like selector forms, scopes, class side, modules, and shadowing distinguish declarations.

For top-level/class members, Phalcom's semantic key often already contains strong logical components: canonical module identity, class identity, canonical selector, dispatch side. For locals, lexical identity is inherently tied to a callable/body revision; renaming or structural edits may require remapping.

A stable-local remapping system can use fingerprints such as:

```text
(callable logical ID,
 lexical-parent path,
 declaration kind,
 source-relative ordinal/fingerprint)
```

but remapping is a heuristic/editor optimization unless the language defines persistent identity. Never use remapping to change semantic resolution.

## 6. Typed newtypes are mandatory at boundaries

Avoid generic integer soup:

```rust
// bad
fn field_owner(id: u32) -> u32;
```

Prefer:

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ModuleId(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ClassId(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct CallableId(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct BindingId(u32);
```

The type system prevents accidentally indexing one arena with another concept's ID. If generational validity matters:

```rust
struct GenId<T> {
    generation: Generation,
    index: u32,
    _marker: PhantomData<fn() -> T>,
}
```

or place generation ownership on the snapshot that resolves the ID. The exact representation is less important than making stale cross-generation use impossible or detectable.

## 7. Arena invariants

An arena is useful when many facts refer to immutable semantic entities. Core invariants:

1. insertion returns an ID that resolves to exactly one entry for the arena lifetime;
2. existing IDs do not silently change referent;
3. published snapshots expose immutable entries;
4. deletion either waits for generation reclamation or uses a generation counter/tombstone discipline;
5. dependencies are stored by IDs only when both share compatible lifetime;
6. source ranges are data on the entity, not its only identity.

A simple immutable-generation arena is often easiest:

```rust
struct SemanticSnapshot {
    generation: Generation,
    classes: Arc<[ClassData]>,
    callables: Arc<[CallableData]>,
    bindings_by_file: Arc<FileTables>,
}
```

Rebuilds can share unchanged slabs/`Arc`s. Query code borrows only from the snapshot it owns.

## 8. Interning: what should be interned?

Common candidates are selector strings, identifier symbols, normalized module URIs, type constructors, and immutable compound semantic keys.

Intern only if equality/hash traffic or duplication is material. An interner needs:

```text
key               canonical value being interned
scope/lifetime     per file, workspace generation, process, persistent cache
memory bound       how entries are reclaimed or bounded
threading          lock/shard/single-writer policy
normalization      whether different spellings map to same key
```

Selector interning is particularly attractive because selector equality is semantically frequent. However, canonicalization must be delegated to Phalcom selector semantics; the interner must not invent normalization.

## 9. Semantic equality versus representation equality

Suppose two independently allocated `ClassData` entries have the same logical `(module, name)`. Representation inequality does not necessarily mean semantic inequality. Conversely, two source occurrences with the same spelling can denote different bindings.

Define equality at the right layer:

```text
ClassId equality          semantic class identity
SourceAnchor equality     source/revision location identity
Arc::ptr_eq               allocation/reuse observation only
Type semantic equality    owned by type system, not ClassId equality
runtime object ===        owned by runtime semantics
```

Never use pointer equality as semantic equality unless the interning invariant explicitly guarantees it.

## 10. Identity and invalidation

Dependency edges require durable endpoints. If a callable summary stores dependency `CallableId A -> B`, and a source edit reconstructs B with a fresh arbitrary ID despite being semantically the same callable, invalidation loses continuity and all dependents appear changed.

This does not mean IDs must never change. It means the engine needs one of:

- stable logical keys across generations;
- old-to-new remapping before dependency comparison;
- generation-local dependency graphs rebuilt where identity changed.

The current structural `CallableId` has useful stability for ordinary body edits because its key is owner + selector + side.

## 11. Identity and reflection

Reflection adds a second axis. A source `ClassId` can identify the declaration of `Foo`; runtime reflection may produce a class object whose identity changes only under runtime rules. The bridge should be explicit:

```text
source ClassId ----compile/load mapping----> runtime class object
```

A static analyzer must not assume that every runtime class object has a source declaration. Native/core/generated classes may use synthetic semantic identities or semantic contracts.

Open-world method mutation also means `CallableId` identifies a source/member declaration, while “method currently installed for selector S on class C” may be a runtime state relation. Static caches need a mutation/version assumption to treat them as equivalent.

## 12. Testing obligations

Test identities as relations, not just constructors:

- same class name in two modules => distinct `ClassId`;
- same selector on instance and class side => distinct callable identity;
- body-only edit => logical callable identity preserved;
- rename => old identity no longer resolves and references remap through the edit operation, not accidentally by stale range;
- whitespace/comment insertion => unrelated IDs/facts do not change merely due to offsets;
- shadowed locals => distinct `BindingId`s despite same spelling;
- snapshot generation mismatch => stale IDs cannot read unrelated entries;
- deterministic full rebuild => equivalent source yields equivalent logical identities independent of hash iteration order.

## 13. Review questions

1. What semantic equivalence relation does this ID encode?
2. Does the ID need to survive a source edit, a snapshot, a process restart, or only a traversal?
3. Is any source range being used as identity rather than provenance/location?
4. Could runtime mutation make a source identity insufficient for the fact being cached?
5. If this is interned, what bounds memory growth?
6. Can an ID from generation `g` accidentally index generation `g+1`?
7. Does deterministic ordering matter for diagnostics/snapshots/tests?
8. Is string cloning/hashing actually a measured hot path before replacing transparent IDs with arenas?
