# Pyrefly type representation, equality, and canonicalization

## Purpose

This document explains the concrete type data structures and normalization paths that keep Pyrefly's type operations tractable. It covers representation, allocation seams, semantic equality, recursive comparison, union simplification, complexity caps, and the exact transfer implications for Phalcom.

The key finding is that canonicalization is layered. Pyrefly does not rely on one global intern table that makes every type pointer-unique. It combines ordinary structural representation, semantic equality contexts, recursive comparison memoization, simplification, and explicit complexity limits.

## Evidence boundary

Pinned revision: 43467e64e36550f232a18e89f24fda79b1020b6b.

Primary files:

- [types.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/types.rs) — Type enum, Var, unions, type arguments, recursive truncation.
- [heap.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/heap.rs) — TypeHeap factory and heap-identity pointer seam.
- [equality.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/equality.rs) — TypeEqCtx, alpha-equivalence, Arc-pair memoization.
- [simplify.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/simplify.rs) — union/intersection flattening, deduplication, literal collapse, width caps.
- [annotation.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/annotation.rs) — annotation-facing type construction.

Local Phalcom seams:

- phalcom-semantic/src/types/id.rs
- phalcom-semantic/src/types/store.rs
- phalcom-semantic/src/types/relation.rs
- phalcom-semantic/src/types/evidence.rs
- docs/spec/typing/01-core-type-lattice-and-unit.md
- docs/spec/typing/02-type-expression-foundation.md

## Type representation

Pyrefly's Type is a large algebraic data type. It includes ordinary nominal/class forms, callable forms, unions/intersections, literals, type aliases, quantified types, type variables, special forms, tuples, typed dictionaries, and solver placeholders.

The relevant implementation properties are:

- Type derives structural ordering/hashing where appropriate;
- Type::Var represents solver variables;
- Union carries members and optional display metadata;
- recursive generic arguments can be truncated;
- TypeEq is separate from ordinary Eq;
- TypeHeap centralizes construction without yet being a complete arena.

This representation lets the solver carry semantic structure directly. It does not encode every fact in a string or a class-name atom.

## TypeHeap: current reality

At this pinned revision, TypeHeap is a factory seam. Its source documentation says the intended future is a per-module arena with cheap copy references, but the current implementation still passes through Type values and uses boxed composites.

The current heap provides:

- a unique heap identity;
- constructors for union, callable, class, quantified, alias, tuple, and other types;
- TypePtr for erased-lifetime references;
- a runtime check that a TypePtr is used with the heap that created it;
- one construction boundary where a future arena can be introduced.

This distinction matters. It is incorrect to describe the pinned Pyrefly as already having a universal TypeId arena. The efficiency transfer is the seam and the ownership discipline, not a fictional completed arena.

### Phalcom starting point

Phalcom already has a TypeId and TypeStore with:

- TypeData variants;
- Vec<TypeData> storage;
- HashMap<TypeData, TypeId> interning;
- canonical Never and Unit IDs;
- normalized flat unions.

That is a valid base. Do not replace it with an arena only because Pyrefly has a TypeHeap abstraction.

Recommended ownership:

~~~rust
struct TypeStore {
    owner: TypeStoreOwner,
    nodes: Vec<TypeData>,
    intern: HashMap<TypeData, TypeId>,
}

enum TypeStoreOwner {
    Core(CoreRevision),
    SemanticGeneration(SemanticGeneration),
    SolverSession(SolverSessionId),
}
~~~

Persistent types should be owned by a snapshot/generation or an explicitly stable core store. Temporary inference variables and unsolved terms must not enter the persistent intern table.

## Type identity layers

Keep these identities separate:

~~~text
TypeId:
    canonical descriptor in one TypeStore

InferVarId:
    temporary solver variable

TypeParameterId:
    binder-aware generic parameter identity

DeclarationId:
    Phalcom declaration identity

Runtime class identity:
    object-model identity used by dispatch/allocation

Flow fact:
    program-point knowledge about a value
~~~

A TypeId cannot answer every runtime or semantic question. The type relation engine must ask the semantic-order service for hierarchy, member, callable, and dispatch facts.

## Ordinary Eq versus semantic TypeEq

Pyrefly defines a custom TypeEq trait. Ordinary structural Eq/Hash is useful for:

- map keys;
- deterministic ordering;
- simple value equality;
- construction-time deduplication.

TypeEq is needed when equality depends on semantic context:

- alpha-equivalent binders;
- unique identities paired between left and right types;
- recursive structures;
- Arc substructures repeatedly compared;
- type variables whose identities are local to a comparison context;
- quantified identities.

The same type can therefore be structurally unequal while semantically equivalent under a valid context.

Phalcom must expose separate APIs:

~~~rust
fn structural_equal(left: TypeId, right: TypeId) -> bool;
fn semantic_equivalent(ctx: &mut TypeEqContext, left: TypeId, right: TypeId)
    -> Equivalence;
fn subtype(ctx: &RelationContext, left: TypeId, right: TypeId)
    -> RelationResult;
~~~

Do not use Arc::ptr_eq as semantic equality. It only indicates allocation reuse.

## TypeEqCtx: what is memoized

Pyrefly's TypeEqCtx contains mappings for identity-bearing values and a set of paired Arc pointers already declared equal.

The Arc pair cache addresses a concrete complexity problem. If a class has N fields and equality recursively compares the same class structure through multiple fields, repeated deep comparisons can become O(N²). Once a pair of Arc nodes is established as equal, subsequent comparisons skip the repeated traversal.

Conceptually:

~~~rust
struct TypeEqContext {
    unique_pairs: SmallMap<UniqueId, UniqueId>,
    alpha_pairs: SmallMap<BinderId, BinderId>,
    type_var_pairs: SmallMap<TypeVarId, TypeVarId>,
    arc_pairs: SmallSet<(PointerId, PointerId)>,
}
~~~

The first pairing wins. A later occurrence must agree with the original pairing. This is how alpha-equivalence avoids accidentally pairing one left binder with two different right binders.

## Recursive equality

Recursive type equality needs an in-progress policy. A comparison should be modeled as:

~~~text
equal(A, B)
  if (A, B) already Proven:
      true
  if (A, B) already Disproven:
      false
  if (A, B) InProgress:
      apply guarded/coinductive relation policy
  mark InProgress
  compare children
  mark Proven or Disproven
~~~

The memoization pair must belong to the comparison context. Do not put temporary recursive assumptions into a global cache whose lifetime exceeds the type relation query.

Phalcom should include the relation policy in the context:

~~~rust
enum EqualityState {
    InProgress,
    Equal,
    NotEqual(EquivalenceReason),
}

struct TypeEqContext {
    pairs: HashMap<(TypeId, TypeId), EqualityState>,
    binders: HashMap<TypeParameterId, TypeParameterId>,
    budget: EqualityBudget,
}
~~~

## Canonical union construction

Pyrefly's union simplifier performs several stages:

1. flatten nested unions;
2. remove Never members;
3. simplify intersections that are absorbed by an existing member;
4. sort members;
5. deduplicate;
6. collapse zero members to Never;
7. collapse one member to that member;
8. simplify literals and enum members where a broader class makes the literals redundant;
9. simplify tuples, quantified forms, typed dictionaries, and built-in class combinations;
10. enforce a maximum union width;
11. construct the final union through TypeHeap.

The ordering is important. Flattening before sorting makes nested unions canonical. Sorting before deduplication makes duplicates adjacent. Collapsing after simplification avoids retaining a one-member wrapper.

Phalcom TypeStore already performs the core flat-union steps. It needs explicit laws and additional policy boundaries.

## Complexity caps are semantic policy

Pyrefly caps union complexity:

- a general union width cap of 4096 members;
- a literal union cap of 256 members;
- an enum union cap of 4096 members;
- recursive type-argument truncation;
- other solver/type-depth budgets.

When a union remains too wide after simplification, Pyrefly widens to an implicit Any. That is a practical Python checker policy. Phalcom must not copy the value blindly.

The important transfer is:

~~~text
canonicalization
  -> simplify where semantics justify it
  -> detect complexity cliff
  -> return explicit widening/uncertainty evidence
  -> prevent unbounded memory/time
~~~

Recommended Phalcom result:

~~~rust
enum NormalizedType {
    Exact(TypeId),
    Widened {
        fallback: TypeId,
        reason: WideningReason,
        dropped: Arc<[TypeId]>,
    },
    Unknown {
        reason: UnknownReason,
    },
}
~~~

A checker may use a sound top-like type for a declared dynamic boundary, but budget exhaustion must remain distinguishable from an explicit Dynamic annotation.

## Intersection behavior

Pyrefly simplifies intersections conservatively. It flattens nested intersections, removes duplicates, handles object as an identity-like case in specific contexts, returns Never for an empty or bottom-containing intersection, and falls back when it cannot safely flatten a union-containing form.

The general lesson is not “implement all intersections now.” It is:

- define which normalization transformations are sound;
- document coupling invariants between simplification passes;
- avoid a pass that creates a form the next pass assumes cannot appear;
- keep fallback behavior explicit.

Phalcom should add intersections only after its lattice and conformance rules are normative.

## Recursive type-argument truncation

Pyrefly includes truncation for recursively nested type arguments. This prevents a recursive generic from generating ever-growing type trees during fixed-point expansion.

Example shape:

~~~text
Node[Node[Node[...]]]
~~~

The solver must preserve enough structure to answer relations, but it cannot retain unbounded syntactic expansion. Truncation should be:

- owner-aware;
- depth/budget-aware;
- accompanied by a reason;
- stable across equivalent computation paths;
- excluded from trusted proof unless the fallback is sound for that consumer.

## Annotation construction

The type representation must distinguish syntax resolution from semantic normalization:

~~~text
annotation syntax
  -> resolve names and constructors
  -> build TypeTerm
  -> validate arity/kind/binder rules
  -> normalize/inter intern
  -> attach source/evidence
~~~

Do not let a normalizer invoke arbitrary user code. Annotation normalization must operate on descriptors and trusted metadata.

Phalcom's current SimpleTypeResolver handles a subset and defers generic applications, tuples, and callable forms. The next implementation should route all accepted forms through TypeStore normalization and return explicit unsupported/unknown diagnostics for the rest.

## Canonicalization cache keys

A canonicalization operation should key on:

~~~text
operation kind
input TypeIds/terms
semantic policy
store owner
normalization budget class
relevant core/type-system revision
~~~

A result from one type-system revision must not silently be reused after changing variance, union policy, protocol rules, or Dynamic semantics.

## Performance mechanics

Canonicalization improves performance through:

- stable TypeId references instead of deep structural keys at every call site;
- early collapse of empty/singleton unions;
- flattening to avoid recursive wrapper traversal;
- sort/dedup to prevent equivalent union permutations;
- semantic pair memoization to avoid repeated deep comparison;
- query-local recursive cache to avoid repeated relation checks;
- width/depth caps to protect memory;
- TypeHeap as a future allocation seam;
- separate temporary solver variables so the persistent store remains compact.

The trade-off is that sorting and deep normalization also cost time. Measure canonicalization hits and member widths. Do not normalize every temporary solver term eagerly if demand-driven finalization is cheaper.

## Phalcom implementation design

### TypeStore boundary

~~~rust
impl TypeStore {
    fn intern(&mut self, data: TypeData) -> TypeId;
    fn union(&mut self, members: impl IntoIterator<Item = TypeTerm>)
        -> NormalizedType;
    fn normalize(&mut self, term: TypeTerm, budget: &mut NormalizeBudget)
        -> NormalizedType;
}
~~~

### Semantic equality

~~~rust
struct TypeEqContext {
    pair_states: HashMap<(TypeId, TypeId), EqualityState>,
    binder_pairs: HashMap<TypeParameterId, TypeParameterId>,
    budget: EqualityBudget,
}
~~~

### Relation boundary

~~~rust
trait TypeRelations {
    fn equivalent(&self, left: TypeId, right: TypeId) -> Equivalence;
    fn subtype(&self, left: TypeId, right: TypeId) -> RelationResult;
    fn assignable(&self, actual: TypeId, expected: TypeId) -> Assignability;
    fn consistent(&self, left: TypeId, right: TypeId) -> Consistency;
}
~~~

These APIs must not be implemented by one permissive compatibility predicate.

## Verification laws

Required unit/property tests:

- union(A, A) equals A;
- union(A, union(B, C)) equals union(C, A, B);
- union(Never, A) equals A;
- normalization is idempotent;
- structural equality and semantic equivalence are distinct;
- alpha-equivalent binders compare equal;
- binder capture is rejected;
- recursive equality terminates;
- pair memoization does not change the result;
- TypeId ownership is enforced;
- complexity caps produce explicit evidence;
- canonicalization does not mutate declared selector or dispatch identity.

## Measurements

Record:

- TypeStore node count;
- intern hit/miss rate;
- average and maximum union width;
- singleton/empty union collapses;
- normalization time;
- semantic equality comparisons;
- Arc/pair memo hits;
- recursive equality depth;
- cap/widening count;
- temporary solver terms not interned;
- bytes retained per generation;
- type allocation count by constructor.

## Transfer conclusion

Pyrefly's type efficiency comes from multiple small canonical forms and explicit complexity policies. Phalcom already has a TypeId/TypeStore base. The correct transfer is to strengthen ownership, semantic equality, normalization laws, recursive guards, and measurement without pretending that Pyrefly's current TypeHeap is already a finished arena or that Python's Any widening is Phalcom's type philosophy.
