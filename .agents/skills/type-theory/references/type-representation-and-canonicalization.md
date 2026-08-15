# Type Representation, Canonicalization, Interning, and Relation Caches

## Purpose

This reference bridges type theory to production compiler representation. It does not prescribe Phalcom's final Rust structs; it gives invariants an implementation must preserve when mapping semantic type expressions into IDs, arenas, interners, substitution views, and relation caches.

The core rule:

> Representation may accelerate semantic equality, but representation equality must not accidentally define semantics that were never ratified.

## 1. Separate permanent semantic types from transient solver terms

A useful conceptual split:

```text
TypeId             canonical finalized semantic type
TypeParamId        source/declaration binder identity
InferenceVarId     temporary solver metavariable
TypeTerm           solver term: TypeId or InferenceVarId/composite
ConstraintId       obligation + provenance
RuntimeDescriptor  optional reflected object for a TypeId
```

Do not intern unresolved inference variables into the permanent `TypeId` arena. Otherwise:

- solver-local state leaks into reflection;
- cache keys depend on solve order;
- equivalent final types can retain different metavariable histories;
- lifetime management becomes difficult.

## 2. A possible semantic type algebra

Conceptual, not normative:

```text
TypeData =
  Nominal(ClassId)
  Protocol(ProtocolId)
  TypeParam(TypeParamId)
  Applied { origin: TypeId, args: InternedSlice<TypeId> }
  Union(InternedSlice<TypeId>)
  Intersection(InternedSlice<TypeId>)
  Callable(CallableTypeId)
  SelfType(SelfOwnerId)
  ForAll { binders, body }
  Existential { binders, body }
  Recursive(...)
  Special(Any | Dynamic | Never | Unit | ...)
  Alias(...)
```

Actual Phalcom design currently makes bare `Class` and `Protocol` objects type expressions directly at the reflective level. Internal `TypeId` may reference those descriptor identities rather than wrapping them observably.

The algebra must be broad enough to represent language types without forcing runtime classes to become the entire type universe.

## 3. Canonicalization goals

For types where semantic equivalence has canonical algebraic laws, a constructor can enforce:

```text
A ≡ B => canonical_id(A) == canonical_id(B)
```

Benefits:

- cheap equality/hash;
- deterministic diagnostics/order;
- compact caches;
- sharing;
- easier serialization.

But canonicalization must be based on **stable semantic laws**.

Unsafe canonicalization examples:

- merging named isomorphic types;
- using current open-world protocol conformance to erase union members permanently;
- replacing missing annotation with `Dynamic`;
- normalizing `Self` before receiver context exists.

## 4. Canonical constructors, not public raw allocation

Prefer APIs:

```text
make_applied(origin,args)
make_union(members)
make_intersection(members)
make_callable(domain,result,effects)
```

that validate and normalize before interning.

Avoid exposing:

```text
arena.alloc(TypeData::Union(raw_members))
```

to arbitrary passes, because malformed/noncanonical nodes then proliferate.

In Rust, keep raw constructors private/internal and expose validated builders/query APIs.

## 5. Interning

Hash-consing maps canonical structural keys to stable IDs:

```text
key -> TypeId
```

Requirements:

- equality/hash of key must reflect semantic canonical form;
- key ordering deterministic;
- no source spans inside semantic identity unless source identity is semantically relevant;
- recursive nodes need special construction strategy;
- memory growth policy explicit.

Interning can be per-compilation snapshot, global process cache, or module database. Lifetime choice affects ID stability and invalidation.

## 6. Stable identity versus structural equality

Three notions:

```text
TypeId identity           ID in one semantic generation/database
semantic equivalence      relation stable under normalization
source identity           annotation/declaration occurrence identity
```

An edit can allocate a new `TypeId` while preserving semantic equivalence. Conversely, a stable declaration ID can now denote changed member metadata after an edit.

Do not use `TypeId` generation equality as source occurrence identity.

## 7. Bare nominal descriptors

If a class descriptor `String` is itself the reflective type expression, internal representation can store:

```text
TypeData::Nominal(ClassId::String)
```

without creating a public wrapper object.

Rule:

```text
reflect(TypeId::Nominal(C)) -> canonical runtime Class descriptor C
```

This maintains one observable descriptor identity while allowing compiler internals to use compact typed IDs.

## 8. Applied types

Canonical key:

```text
AppliedKey {
  origin: TypeConstructorIdentity,
  args: [TypeId]
}
```

Construction obligations:

1. origin is applicable constructor;
2. arity exact under current policy;
3. arguments well formed;
4. bounds/finite constraints validated;
5. args canonicalized;
6. interner lookup/reuse;
7. publish immutable descriptor/view.

Application must not clone runtime class behavior unless separately required.

## 9. Substituted member views

Avoid materializing a full specialized class graph for each `Box<Int>`.

Query:

```text
member_type(applied_type, member_id)
```

can compute:

```text
origin annotation + application environment -> substituted TypeId
```

Cache key:

```text
(applied TypeId, member semantic ID, member declaration generation)
```

Value includes substituted type plus provenance back to source annotation.

## 10. Union/intersection canonical keys

If commutative/idempotent:

```text
UnionKey = sorted unique canonical member IDs
```

Apply cheap stable identities (`Never`, `Any`) before interning.

Be cautious with semantic subsumption elimination because it may invoke context-dependent/open-world relations. Stable canonical identity should not depend on mutable class surfaces unless type generations encode those dependencies.

## 11. Callable canonicalization

Canonical key must include every semantically relevant feature:

```text
parameter lane shape
labels
rest/default acceptance semantics if part of type equality
parameter TypeIds
result TypeId
control/effect component if typed
binder structure for generic callables
```

Do not include source parameter names if labels are not semantically part of callable identity; do include labels if selector/call shape requires them.

## 12. Type parameter representation

Use semantic identity:

```text
TypeParamId(owner,index)
```

Metadata such as spelling, variance, bound, source span lives on owner/signature descriptors.

A `TypeData::TypeParam(p)` can be canonical by `p` identity. Renaming display name need not alter identity inside same declaration generation.

## 13. `Self` representation

`Self` should carry binder/context identity:

```text
SelfType { owner: SelfOwnerId, side: Instance | Class }
```

possibly plus semantics mode if Phalcom distinguishes lexical/dynamic self forms.

Do not intern all `Self` occurrences as one global singleton if owner matters.

## 14. Aliases

Two broad designs:

### Transparent alias

```text
type Name = T
```

Subtyping/equivalence may expand it. Reflection may still preserve alias descriptor/name.

### Nominal/newtype alias

Alias creates distinct semantic identity and may not be equivalent to target.

Representation must encode which. Do not implement all aliases as string synonyms before language semantics decides.

## 15. Recursive type construction

Interning cyclic graph cannot require completed child hashes recursively.

Options:

- nominal recursive IDs break cycle;
- allocate provisional node IDs, build SCC, freeze after validation;
- canonical `μ` syntax with de Bruijn indices;
- keep recursive aliases nominal.

Never expose partially initialized descriptor to ordinary reflection. Trusted bootstrap/provisional shells need explicit freeze invariants.

## 16. Relation caches

A relation query:

```text
is_subtype(A,B,context)
```

may depend on more than `A`/`B` structural IDs.

Cache key/dependencies can include:

```text
A TypeId
B TypeId
relation mode
substitution environment
candidate member-surface generations
protocol requirement generations
module visibility/access context
checker mode / feature version
```

For immutable nominal inheritance encoded in TypeIds, dependencies may be smaller. For open structural conformance, they are larger.

## 17. In-progress relation state

Recursive relation cache should distinguish:

```text
Vacant
InProgress(obligation path/polarity)
Proven
Disproven(reason)
Blocked
```

Do not store `true` immediately on entry. Coinductive cycles need guarded policy and must not poison cache if a child later fails.

## 18. Negative cache invalidation

Example:

```text
C !conforms P
```

then user adds required method. If negative result cached only by IDs, stale failure persists.

Dependency-directed invalidation should mark relation cache stale when candidate surface changes.

A stale correct-looking type error is still incorrect.

## 19. Provenance side tables

Canonical TypeIds intentionally forget source history. Diagnostics still need it.

Keep side structures:

```text
TypeOccurrenceId -> {source span, source syntax, normalized TypeId}
ConstraintId -> causal source/provenance
AppliedMemberView -> original annotation occurrence + substitution environment
```

Do not put source span into `TypeId` just to recover diagnostics; that destroys sharing/equivalence.

## 20. Hash/equality invariants

For canonical keys:

```text
semantic_equivalent(a,b) => hash(key(a)) == hash(key(b))
```

If using canonical IDs as equality:

```text
id(a)==id(b) => a ≡ b
```

and ideally within supported canonical domain:

```text
a ≡ b => id(a)==id(b)
```

Test these properties with generated types.

## 21. Determinism

Compiler/LSP output should not vary with hash-map iteration order.

Use deterministic canonical ordering for:

- union/intersection members;
- constraint diagnostics;
- protocol requirement iteration where diagnostic order matters;
- serialized type graphs.

Rust `HashMap` iteration should not define user-visible ordering.

## 22. Memory bounds

Type interning can grow from:

- many editor revisions;
- generated union combinations;
- inferred applied generic types;
- reflection-created descriptors.

Specify lifetime:

```text
per snapshot
per project generation
weak global cache
bounded LRU for derived views
```

Never recommend a cache without key, value, validity condition, invalidation event, concurrency policy, and memory bound.

## 23. Snapshot/concurrency model

LSP semantic queries benefit from immutable published generations:

```text
revision N -> frozen type arena + relation facts
revision N+1 builds separately / incrementally
queries hold snapshot handle
```

Long-lived raw references into mutable arenas are hazardous. Use IDs/snapshot-owned handles.

This is an implementation boundary with `rust-compiler-engineering`/`lsp-development`, but type identity semantics determine what can be snapshotted.

## 24. Serialization/bytecode metadata

Persisted TypeIds cannot be raw process-local indexes unless relocated.

Serialize semantic structure/references through stable declaration identities and reconstruct canonical TypeIds on load.

Validate before interning authoritative metadata. Malformed package should produce diagnostic/error, not corrupt interner invariants.

## 25. Testing obligations

- canonical equivalent types share IDs where promised;
- non-equivalent isomorphic named types do not collapse;
- type parameter identity survives spelling shadow/rename correctly;
- union order normalization deterministic;
- applied type repeated construction canonical;
- substituted member views use owner-qualified binders;
- recursive types terminate/freeze safely;
- relation cache invalidates on relevant semantic changes;
- incremental and clean rebuild produce semantically equivalent canonical graphs;
- stress tests bound interner/cache memory.

## 26. Failure modes

- `Type = ClassId`.
- Permanent arena contains inference metavariables.
- Source span participates in semantic type hash.
- Applied type clones whole class graph.
- Open-world conformance simplification becomes permanent canonical equality.
- Cache lacks invalidation/memory policy.
- Runtime pointer address defines canonical order.
- Partial recursive descriptor visible to user reflection.

## 27. Competency questions

1. Why separate `TypeId` from `InferenceVarId`?
2. Which normalization laws are safe to use in a permanent canonical key?
3. Why is a substituted member view better than cloning whole generic class graph?
4. Why can relation cache require semantic-generation dependencies beyond `(A,B)`?
5. How do you retain diagnostic source information without putting spans into canonical type identity?
6. What invariants must hold between semantic equivalence, hashing, and canonical IDs?
