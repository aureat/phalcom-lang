# Semantic Identities, Arenas, and Interning

## Why IDs

Long-lived semantic graphs are easier to own in Rust with compact IDs than references threaded through self-referential structures.

Examples:

```text
ModuleId
ClassId
CallableId
FieldId
BindingId
ScopeId
ExprId
BasicBlockId
TypeId
ConstraintId
```

## Identity categories

### Stable across edits/project lifetime

Potentially module/package identity, canonical selector symbol, built-in/core descriptor identity.

### Stable within a snapshot/body revision

Binding IDs, expression IDs, CFG block IDs can be regenerated on reparse/lowering.

### Canonical semantic identity

Types/applied types may be interned/canonicalized by structural/nominal key.

Document lifetime explicitly.

## Arenas

Store semantic nodes in vectors/arenas:

```rust
struct Arena<T> { data: Vec<T> }
struct ExprId(u32);
```

Advantages:

- compact IDs;
- cache locality;
- borrow-friendly ownership;
- deterministic iteration;
- easy serialization/debugging.

Use generation IDs if stale handles can outlive arena replacement.

## Interning

Intern repeated immutable values such as selectors, names, canonical types or signatures when identity/equality semantics support it.

Do not intern source ranges or mutable facts merely to save allocation.

## IDs versus hashes

A content hash is useful for cache invalidation but is not automatically semantic identity. Two distinct nominal declarations can have identical content hashes.

## Source ranges

Ranges locate syntax/diagnostics and help source mapping. They are unstable under edits and should not define declaration identity where a semantic ID exists.

## Cross-module references

Use qualified IDs/references rather than borrowed pointers into another module's mutable analysis data. This mirrors the strongest lesson from incremental semantic systems: copied remote facts become stale.
