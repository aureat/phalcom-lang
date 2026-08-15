# Heap, Aliasing, and Frame Conditions

## The frame problem

To prove:

```text
self.balance == old(self.balance) + amount
```

you must know what other operations can mutate `self.balance` or aliased objects.

## Heap model

Conceptual heap:

```text
H : (ObjectId, FieldId) -> Value
```

Mutation creates new version:

```text
H1 = store(H0, self, balanceField, newValue)
```

Reading:

```text
select(H1, self, balanceField)
```

## Frame conditions

A method summary states what may change:

```text
modifies { self.balance }
```

Then all other modeled locations remain equal between pre/post states.

User syntax need not expose `modifies` immediately; compiler can infer conservative effects or derive from trusted source.

## Aliasing

If `other === self`, writing `other.field` affects `self.field`. Source restrictions on receiver-local field access can simplify ordinary Phalcom code, but calls may mutate receiver/arguments indirectly.

## `old` values

Postconditions often reference pre-state:

```text
old(expr)
```

Snapshot required heap/value terms at method entry rather than re-evaluating `expr` after mutation.

## Collections

Mutable collection contents are heap state even if represented by native Rust structures. Model abstract sequence/map content or use library contracts instead of exposing implementation fields.

## Native/FFI mutation

Without effect contracts, assume passed mutable objects/buffers may change. This can make proofs weak but preserves soundness.

## Separation logic

For sophisticated heap reasoning, separation logic expresses disjoint ownership and local reasoning. It is valuable if Phalcom later adopts ownership/capability concepts, but is probably beyond a first prover unless heap contracts demand it.

---

## Deep treatment: local reasoning over mutable objects

### Aliasing relation

Source variables do not identify disjoint objects. If symbolic terms `a` and `b` may alias, a write through one must affect facts about the other. Alias information can have states such as:

```text
MustAlias(a,b)
MayAlias(a,b)
NoAlias(a,b)
Unknown
```

Only `NoAlias` justifies independent framing. `MayAlias` must conservatively preserve the possibility that the write hits the tracked location.

### Heap SSA

Let:

```text
H0 = entry heap
H1 = store(H0, o, f, v)
H2 = post-call heap
```

Then:

```text
select(H1,o,f) = v
```

and for other locations a frame property is required. In array theory, store/select supplies some of this automatically for exact `(object,field)` pairs, but abstract collection/native state may need separate functions.

### Effect regions

Enumerating every object-field pair is often impossible. Use effect regions such as:

```text
ReceiverFields(self)
ArgumentReachable(arg0)
ModuleState(ModuleId)
CollectionContents(obj)
DispatchTable(ClassId)
IOWorld
SchedulerState
```

A may-write effect set over regions is conservative. Proof facts mention dependencies on regions, and a call invalidates facts whose read regions intersect may-write regions.

### Read/write footprints

For expression/predicate `P`, compute a read footprint `R(P)`. A frame fact survives operation `C` if:

```text
R(P) ∩ MayWrite(C) = ∅
```

or a stronger relational postcondition re-establishes it. This gives a precise bridge between effect analysis and proof.

### `old` and mutable references

If `old(x)` where `x` is an object reference simply snapshots identity, later field reads through that reference still observe the new heap unless the logic explicitly pairs old values with `H0`:

```text
old(x.field)  -> select(H0, x0, field)
old(x)        -> x0   (identity value)
```

Then `old(x).field` is ambiguous unless the language defines whether field access is also evaluated in old state. Prefer a clear syntax/logic rule rather than accidental AST rewriting semantics.

### Deep snapshots

A deep snapshot of an arbitrary object graph is expensive and semantically fraught with cycles/identity. Do not define `old(object)` as deep copy unless the language explicitly wants that behavior. Logical old-state heaps are usually cleaner.

### Collections

For mutable collection `c`:

```text
contents(H,c) : Seq<Value>
```

An append summary may be:

```text
contents(H1,c) = concat(contents(H0,c), [x])
length(H1,c) = length(H0,c)+1
```

with a frame saying unrelated collections/fields remain unchanged. Native representation can stay opaque.

### Escape and ownership opportunities

If semantic analysis proves an object is freshly allocated and does not escape, aliases are limited. This can strengthen frame reasoning dramatically. Future ownership/capability features could formalize such guarantees, but the prover should consume them only when the language/analysis provides a sound theorem.

### Callbacks and reentrancy

A call that invokes user code can mutate shared/aliased objects even if the immediate native primitive itself writes nothing. Effects must be transitive and include callbacks. FFI summary `pure` is unsound if it calls back into arbitrary Phalcom.

### Separation logic boundary

Separation logic's separating conjunction:

```text
P * Q
```

asserts ownership of disjoint heap portions, enabling local frame reasoning. It is powerful when ownership/disjointness is first-class. Without language support, aliasing and dynamic object graphs may make annotations burdensome. A first Phalcom prover can use effect regions + alias analysis and revisit separation logic if heap proofs dominate.

### Tests

- must-alias write updates both symbolic views;
- may-alias write prevents preservation of potentially affected fact;
- framed unrelated field remains stable;
- native callback invalidates reachable shared state;
- `old(field)` reads entry heap even after multiple writes;
- mutable collection contract is representation-independent;
- call with `ArgumentReachable(arg)` effect invalidates nested aliases.
