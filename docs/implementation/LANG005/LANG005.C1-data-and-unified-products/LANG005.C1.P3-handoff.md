# Implementer Handoff — LANG005.C1.P3 Anonymous Product Convergence, Value-Semantic Optimization, and Runtime Reification Closure

You are the implementing agent for:

> **LANG005.C1.P3 — Anonymous Product Convergence, Value-Semantic Optimization, and Runtime Reification Closure**

This is the final implementation checkpoint of LANG005.C1.

You are not being asked to design the feature. The semantic and architectural decisions are already made. Your responsibility is to implement those decisions faithfully, adapt them mechanically to the current repository, verify every checkpoint, and stop on architectural contradictions rather than silently choosing a different design.

The governing patch-grade implementation plan is:

```text
docs/implementation/LANG005/LANG005.C1-/
  LANG005.C1.P3-anonymous-product-convergence-and-reification-closure-plan.md
```

Read that plan completely before editing code.

Also read:

```text
docs/implementation/LANG005/LANG005.C1-/implementation-state.md

docs/implementation/LANG005/LANG005.C1-/
  LANG005.C1.P1-first-class-data-and-unified-product-enum-representation-plan.md

docs/implementation/LANG005/LANG005.C1-/
  LANG005.C1.P2-representation-aware-product-optimizer-scalar-replacement-plan.md

docs/implementation/LANG005/LANG005.C1-/
  LANG005.C1.P2-handoff.md

docs/decisions/
  PDR-0035 or the final landed filename for the `data` value-product ruling

docs/internals/data-representation/
```

The P3 plan is the implementation checklist.

This handoff adds implementation-time architectural direction, current repository facts, and several places where you must not mechanically follow stale pre-P1/P2 assumptions from the original plan.

---

# 1. Current repository state

The P3 plan was originally prepared before P1/P2 had fully landed.

Current `main` at handoff preparation is:

```text
d8b823e857b4054420cc070cda9d6b030f151391
```

with commit:

```text
feat(core,semantic): resolve LANG005.C1.P1 audit findings
                     and implement P2 product optimization
```

Before doing anything:

```bash
git status --short
git rev-parse HEAD
git log --oneline -20
```

If HEAD has advanced, inspect intervening commits touching:

```text
phalcom-core/src/product/
phalcom-core/src/heap/
phalcom-core/src/value/
phalcom-core/src/compiler/
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/frame.rs
phalcom-core/src/typing/
phalcom-semantic/src/
phalcom-type-meta/
Tuple / Record core library
fiber / block / invocation machinery
```

Preserve unrelated working-tree changes.

Do not reset, clean, stash, or overwrite unrelated user work.

---

# 2. What has already landed since the P3 plan was written

Do not blindly reimplement P3.C0 work that the repository already satisfies.

The current P1/P2 landing includes at least:

```text
- first-class data
- ProductLayout / ProductStorage
- RuntimeDataDescriptor / DataObject
- compact product representation
- enum payload convergence
- data non-subclassability repair
- ProductStorage field encapsulation
- fail-closed missing data lowering
- P2 ProductOptimizationMode
- P2 scalar replacement / virtual products
- P2 nested data virtualization
- P2 exact enum-case optimization support
- Enabled / Disabled optimizer testing infrastructure
```

The latest P1 repair explicitly rejects `data` as a class superclass.

Therefore P3 Task 2's hierarchy repair may already be satisfied.

Do not rewrite it merely because it exists in the historical plan.

Instead:

```text
verify
    ↓
record evidence
    ↓
only patch if invariant is still incomplete
```

Likewise, constructor source-order preservation must be re-verified against the **canonical Disabled/P1 path**, not assumed broken merely because it was a historical P3 requirement.

---

# 3. One known P2 defect still exists and P3 must repair it

The current P2 optimizer uses canonical `BindingId` as a map key, but the planner currently manufactures those IDs itself.

The landed implementation contains logic morally equivalent to:

```rust
struct ProductPlanner<'a> {
    candidates: BTreeMap<BindingId, CandidateBinding>,

    // WRONG semantic authority:
    name_stack: Vec<BTreeMap<String, BindingId>>,
    next_binding_id: u32,
}

fn alloc_binding_id(&mut self) -> BindingId {
    ...
}

fn resolve_visible_binding(&self, name: &str) -> Option<BindingId> {
    ...
}
```

This is not acceptable as the final LANG005 architecture.

P3.C0 Task 3 is mandatory.

The optimizer must consume the real semantic `BindingId`, not manufacture an equivalent-looking integer.

The rule is:

```text
semantic analysis:
    determines WHICH binding an occurrence refers to

optimizer AST walk:
    determines HOW that occurrence is used
```

Never:

```text
optimizer:
    resolves name -> declaration itself
```

The final optimizer identity must be based on semantic authority.

Conceptually:

```rust
struct ProductBindingKey {
    callable: CallableId,
    binding: BindingId,
}
```

or simply `BindingId` if current callable/body scope makes it unambiguous.

Use current canonical source semantic attachments.

Do not introduce another lexical resolver.

This repair is foundational before Tuple/Record virtualization.

---

# 4. Core P3 semantic decision

This checkpoint ratifies and implements:

> Tuple and Record are transparent immutable structural value products with semantic value identity rather than allocation identity.

They now have the same physical optimization freedom as `data`.

Therefore an implementation may:

```text
scalar replace Tuple/Record
flatten them
keep their components in locals
pack them
share materialized backing storage
rematerialize them
eliminate their allocation
answer some observations without materialization
```

provided language semantics are unchanged.

The backing:

```text
ObjRef
address
allocation event
ProductLayoutId
ProductShapeId
```

is not language-visible identity.

---

# 5. Transparent product categories remain semantically distinct

Sharing physical infrastructure must never collapse these language categories:

```text
data
    nominal transparent immutable product

Tuple
    structural ordered transparent immutable product

Record
    structural key-based transparent immutable product

enum
    nominal closed sum; payload uses product machinery

class
    opaque identity-bearing object
```

Physical reuse does not imply semantic equivalence.

For example:

```phalcom
data Point(x: Int, y: Int)

const d = Point(x: 1, y: 2)
const t = (1, 2)
```

Even if both physically become:

```text
two packed scalar words
```

they are not the same semantic type or value category.

Never use:

```text
ProductLayoutId
```

as semantic type identity.

---

# 6. Exact relation `===` after P3

P3 changes language-level exactness for Tuple and Record.

Internal representation identity such as:

```rust
Value::same_as(...)
```

must remain available for VM implementation purposes.

It is not the language-level authority for transparent products.

The language relation is:

## Unit

```text
Unit === Unit
```

There is one zero product.

## Tuple

Two positive Tuple values are `===` iff:

```text
same tuple structural shape
AND
corresponding components recursively ===
```

Tuple shape includes:

```text
arity
positional/labeled lane boundary
ordered labels
```

Therefore:

```phalcom
(1, 2) === (1, 2)
```

is true even if independently materialized.

But:

```phalcom
(x: 1, y: 2) === (y: 2, x: 1)
```

is false.

## Record

Two Records are `===` iff:

```text
same key set
AND
per-key values recursively ===
```

Presentation order does not participate.

Therefore:

```phalcom
const a = #{ name: "A", age: 24 }
const b = #{ age: 24, name: "A" }

a === b
```

must be true.

But iteration and rendering must still differ:

```text
a:
    name, age

b:
    age, name
```

## data

Preserve P1:

```text
same exact reified nominal data type
AND
components recursively ===
```

Thus:

```phalcom
Id<User>(42) === Id<Order>(42)
```

is false even if both have the same physical layout.

---

# 7. `==`, hash, and `===` are not the same operation

Do not accidentally redefine collection APIs while implementing exactness.

Existing public contracts remain:

```text
Tuple ==
    structural ordinary equality
    ordered

Tuple hash
    agrees with Tuple ==

Record ==
    key/value equality
    independent of presentation order

Record hash
    independent of presentation order
    agrees with Record ==

===
    recursive exact relation described above
```

Do not implement `===` simply by forwarding to `==`.

Do not rewrite `==` to use `===`.

They are distinct language operations.

---

# 8. Unit remains the only zero product

No runtime empty Tuple or Record object may exist.

These all normalize to Unit:

```phalcom
()
#{}
```

The current repository still has deliberately annotated P3 FIXME sites around direct empty Tuple creation.

P3 must eliminate every executable bypass.

Search the repository for:

```text
TupleObject::positional(Vec::new())
TupleObject::new(empty...)
RecordObject::new(empty...)
Object::Tuple(empty...)
Object::Record(empty...)
```

Every runtime construction route must satisfy:

```text
arity == 0
    -> Value::unit()
```

Do not allocate:

```text
RuntimeAnonymousProductDescriptorId
TupleObject
RecordObject
ProductStorage
```

for zero arity.

---

# 9. Do not confuse four different identities

P3 relies on a strict identity model.

Keep these separate:

```text
RuntimeTypeRef
    semantic structural type

ProductShapeId
    runtime coordinate/presentation shape

ProductLayoutId
    physical storage layout

RuntimeAnonymousProductDescriptorId
    materialized runtime descriptor joining shape/layout/
    optional exact type

ClassId
    behavior class: Tuple or Record

ObjRef
    heap backing handle
```

They answer different questions.

For example, two Records:

```phalcom
#{ name: "A", age: 24 }

#{ age: 24, name: "A" }
```

may have:

```text
same RuntimeTypeRef
same ProductLayoutId
different ProductShapeId
different RuntimeAnonymousProductDescriptorId
same Record ClassId
different ObjRef
```

and still be:

```text
===
```

---

# 10. ProductShape is presentation/coordinate metadata, not type identity

Implement shared, VM-owned shape metadata.

The plan proposes information equivalent to:

```rust
#[repr(transparent)]
pub struct ProductShapeId(pub u32);

pub enum ProductShape {
    Tuple(TupleProductShape),
    Record(RecordProductShape),
}

pub struct TupleProductShape {
    pub positional_len: u32,
    pub labels: Box<[Symbol]>,
}

pub struct RecordProductShape {
    pub presentation_labels: Box<[Symbol]>,
    pub logical_labels: Box<[Symbol]>,
    pub presentation_to_logical: Box<[u32]>,
}
```

Mechanical spelling may change.

The information boundaries may not.

The shape registry:

```text
is VM-owned
is shared
is not GC heap payload
is not copied per product value
```

---

# 11. Record canonical order versus presentation order

This is the single highest-risk structural-product bug.

For:

```phalcom
const a = #{ name: "A", age: 24 }
const b = #{ age: 24, name: "A" }
```

semantic type identity is order-independent.

The runtime may canonicalize logical storage:

```text
logical 0 -> age
logical 1 -> name
```

for both.

But their ProductShape metadata must preserve:

```text
a:
    presentation 0 name -> logical 1
    presentation 1 age  -> logical 0

b:
    presentation 0 age  -> logical 0
    presentation 1 name -> logical 1
```

Operations must use the appropriate coordinate system.

Use logical order for:

```text
semantic field coordinate
canonical storage
type identity
Record equality
Record hash
Record ===
```

Use presentation order for:

```text
iteration
printing
reflection presentation
spread/expansion
serialization default
Record -> sequence-like enumeration
```

Do not accidentally make:

```text
ProductShapeId == Record TypeId
```

or:

```text
presentation order == semantic record field order
```

---

# 12. ProductLayout must stay purely physical

P1 already provides `ProductLayout` / `ProductStorage`.

Do not add labels to `ProductLayout`.

A layout answers:

```text
how many words
which component uses which words
primitive representation
Value representation
GC trace map
alignment/size if relevant
```

It does not answer:

```text
field name
tuple label
record presentation order
nominal data identity
exact generic type
```

The P3 split is:

```text
semantic type
    |
    | independent
    v

ProductShape
    presentation/coordinates

ProductLayout
    physical representation

Runtime anonymous descriptor
    joins the runtime facts required by materialized object
```

---

# 13. ProductStorage is already encapsulated — preserve that repair

The latest P1 audit fix encapsulates ProductStorage internals.

Use its canonical APIs such as:

```text
layout_id()
words()
word_len()
checked stores/loads
```

or their final repository equivalents.

Do not reopen fields merely because Tuple/Record migration needs access.

Do not write raw words directly from Tuple/Record implementation if P1 already owns checked encoding helpers.

Continue these invariants:

```text
Value raw-word conversion remains encapsulated
checked component indexing
checked layout/storage agreement
checked word arithmetic
precise trace plan
```

---

# 14. Materialized Tuple/Record target representation

Retain the existing distinct heap variants:

```text
Object::Tuple
Object::Record
```

Do not replace them with:

```text
Object::Product
```

in P3.

Their internals converge instead.

Target:

```rust
pub struct TupleObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}

pub struct RecordObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}
```

After migration there should be no:

```rust
values: Box<[Value]>
labels: Box<[Symbol]>
```

inside either value object.

This gives physical convergence while keeping runtime semantic categories obvious and minimizing dangerous fanout.

---

# 15. Static and dynamic products converge on one storage substrate

There are two construction paths, not two representations.

## Static closed product

Example:

```phalcom
const t = (10, 20)
const r = #{ name: "A", age: 24 }
```

Compiler knows fixed shape.

Use:

```text
precomputed ProductShape
precomputed ProductLayout
optional RuntimeTypeRecipe
source-to-logical mapping
    ↓
ProductStorage
```

Primitive-width packing should be possible.

## Dynamic product

Example: pack/spread/computed structure.

Use:

```text
runtime labels / values
    ↓
intern ProductShape
    ↓
universal Value-slot ProductLayout
    ↓
ProductStorage
```

Dynamic construction does not justify preserving legacy:

```text
Box<[Value]>
Box<[Symbol]>
```

forever.

Both routes should converge on the same `TupleObject`/`RecordObject` representation.

---

# 16. Static product construction lowering

Introduce compiler-facing lowering metadata only from semantic facts already established.

Conceptually:

```rust
struct AnonymousProductConstructionLoweringSpec {
    kind: AnonymousProductKind,
    shape: AnonymousProductShapeSpec,
    type_recipe: Option<RuntimeTypeRecipe>,
    layout: ProductLayoutSpec,

    // Source expression ordinal -> logical/storage coordinate
    source_to_logical: Box<[u32]>,
}
```

The critical rule is:

```text
source evaluation order
    !=
logical order
    !=
physical order
```

Do not compile fields in canonical Record order.

For:

```phalcom
#{
  z: sideEffectZ(),
  a: sideEffectA(),
}
```

evaluate:

```text
sideEffectZ()
sideEffectA()
```

even if logical/storage order is:

```text
a
z
```

Store results according to:

```text
source_to_logical
```

after or during source-order evaluation.

---

# 17. Do not fabricate exact structural type information

A runtime key set is not proof of a closed semantic Record type.

If the semantic analyzer only established:

```text
Dynamic
broad Record
open row
intersection
otherwise non-closed structural knowledge
```

then a runtime result:

```text
#{a: ..., b: ...}
```

does not permit the runtime to manufacture a new exact semantic type solely because those keys were observed.

Runtime operational shape may be known:

```text
ProductShape = [a, b]
```

while:

```text
exact_type = None
```

or retains the actual broad semantic type recipe.

This distinction is mandatory.

Never infer static semantics from runtime payload inspection.

---

# 18. Add representation-neutral Tuple/Record views

After changing TupleObject/RecordObject storage, do not scatter:

```text
descriptor lookup
shape lookup
layout lookup
ProductStorage decode
presentation/logical mapping
```

through every primitive.

Create one runtime read abstraction.

Conceptual surface:

```text
TupleView
    len
    positional_len
    labeled_len
    value_at(index)
    label_at(...)
    lookup_label(...)
    positionals()
    labeled_entries()

RecordView
    len
    label_at_presentation_index(...)
    value_at_presentation_index(...)
    value_at_logical_index(...)
    lookup_label(...)
    entries_in_presentation_order()
```

Every runtime consumer should use this view.

Centralize:

```text
ProductStorage logical load
```

there.

---

# 19. Consumer migration must be exhaustive

Do not stop after Tuple/Record primitives compile.

Search all direct consumers.

Audit at least:

```text
Tuple primitive methods
Record primitive methods
Value rendering/debug
argument expansion
* / ** / ***
rest capture
block invocation
bound method invocation
send argument routing
pattern/destructuring
reflection adapters
typing/reflection Tuple construction
Map <-> Record conversion
GC
serialization/presentation helpers
native library bridges
tests that inspect raw object representation
```

After migration, direct object-local `values`/`labels` access should disappear because those fields no longer exist.

---

# 20. Preserve argument-pack semantics exactly

Tuple is also an argument-pack representation in multiple runtime paths.

P3 must not accidentally redefine:

```text
positionals
labeled suffix
rest
*
**
***
```

because Tuple storage changed.

Important distinctions:

```text
Tuple can have arbitrary arity as a value

send/call arity may have separate selector/runtime limits
```

Do not impose call limits on Tuple storage.

Preserve:

```text
left-to-right evaluation
duplicate-label diagnostics
label suffix semantics
rest Unit normalization
pack expansion ordering
```

---

# 21. Pattern/destructuring migration

Tuple/Record patterns must become storage-independent.

Pattern semantics operate on:

```text
arity
labels
components
```

not:

```text
Box<[Value]> memory
ObjRef identity
```

Preserve current failure timing and binding rollback.

Nested destructuring must keep current scratch-slot discipline.

Do not rewrite the pattern algorithm while changing the representation.

Use views as the adapter.

---

# 22. P3 exactness implementation must be VM-aware

Do not globally change `Value::same_as`.

That function still has valid uses for:

```text
sentinel checks
raw runtime representation identity
internal handles
```

Create or extend the VM-aware language exactness helper introduced for `data`.

Conceptually:

```rust
fn language_exact(
    vm: &VM,
    lhs: Value,
    rhs: Value,
) -> PhResult<bool>
```

Then:

```text
Data
    exact semantic descriptor + recursive components

Tuple
    shape semantics + recursive components

Record
    key set + per-key recursive components

other values
    existing exact semantics
```

Both:

```text
Bytecode::Same
Object#===
```

must reach the same language authority.

Audit all `same_as` call sites and distinguish:

```text
language ===
vs
VM representation identity
```

Do not mechanically replace every `same_as`.

---

# 23. P2 optimizer architecture is extended, not replaced

P2 is already landed.

Do not build a new optimizer.

Extend:

```text
ProductOptimizationMode
ProductFunctionPlan
VirtualShapePlan
ActiveVirtualProduct
leaf-slot substrate
Enabled/Disabled differential harness
```

Current P2 has:

```rust
enum VirtualProductKind {
    Data { ... },
    Variant { ... },
}
```

Extend to:

```rust
enum VirtualProductKind {
    Data { ... },
    Tuple { ... },
    Record { ... },
    Variant { ... },
}
```

Current:

```rust
NestedData
```

becomes:

```rust
NestedProduct
```

Keep current exact-enum restrictions.

---

# 24. First repair P2 semantic binding identity

Before adding Tuple/Record optimizer candidates:

remove:

```text
name_stack
alloc_binding_id
next_binding_id
resolve_visible_binding(name)
```

as sources of binding semantic identity.

Do not merely rename them.

Project real semantic binding identity into compiler-visible lowering.

Recommended authority:

```text
semantic source occurrence
    -> BindingId

optimizer visitor
    -> ProductUseKind for that occurrence
```

Use an exact occurrence identity such as:

```text
ExpressionId
SourceSiteId
LoweringSite
```

according to current repository architecture.

SourceRange may participate in diagnostics/fallback lookup if repository constraints require it, but it cannot be the semantic identity.

Test hostile cases:

```text
same-name shadowing
parameter shadows local
nested block
loop binding
pattern binding
source range shifts
incremental reparse
same spelling in multiple scopes
```

If any optimizer decision changes because a source range moved while semantic identity did not, the repair is incomplete.

---

# 25. Virtual leaves remain ordinary `Value` frame slots

Do not introduce raw machine integer/float locals in P3.

Virtual:

```phalcom
(1, object)
```

becomes logically:

```text
slot N     = Value(Int...)
slot N + 1 = Value(ObjRef...)
```

This retains:

```text
GC roots
fiber stack preservation
exception unwinding
upvalues around the slots
existing frame layout
existing send ABI
```

Primitive-width ProductStorage packing and virtual frame representation are different concerns.

Materialized:

```text
may use Int64 slot
```

Virtual frame:

```text
uses Value
```

Do not conflate them.

---

# 26. Extend scalar replacement to Tuple and Record

Eligible:

```phalcom
const p = (a(), b())
useStructurally(p)
```

when all uses satisfy optimizer proof.

Eligible Record:

```phalcom
const r = #{
  y: effectY(),
  x: effectX(),
}
```

may exist only as leaf slots.

But effects must remain:

```text
effectY()
effectX()
```

once each.

The aggregate allocation may disappear.

The component computations may not.

---

# 27. Ordinary method/index/getter dispatch remains a boundary

Identity-free does not imply:

```text
record.foo
```

means direct key lookup.

Phalcom dispatch semantics may permit a getter/method before Record fallback.

Likewise:

```phalcom
tuple[0]
record[#name]
x == y
x.hash
x.toString
```

may involve ordinary overridable dispatch.

Therefore P3 may direct-load a virtual product component only where semantic/compiler lowering proves the operation is a structural intrinsic or an existing guarded optimization guarantees equivalence.

Otherwise:

```text
materialize
send normally
```

This is a critical rule.

Do not make scalar replacement a devirtualization engine.

---

# 28. `.class` may avoid materialization

Transparent product `.class` does not require a backing object.

For virtual values:

```text
virtual data   .class -> declaration behavior class
virtual Tuple  .class -> Tuple
virtual Record .class -> Record
```

No DataObject/TupleObject/RecordObject allocation is required.

This is allowed because `.class` exposes behavior category, not backing allocation.

---

# 29. Exact type observation may avoid materialization

If the virtual product retains a closed or instantiable runtime type recipe:

```text
virtual product
    +
RuntimeTypeRecipe
    +
current RuntimeTypeEnvironment
    ↓
RuntimeTypeRef
```

the compiler/runtime may answer exact type observation without materializing the value.

Do not create a temporary ProductObject merely to ask it for its type.

---

# 30. Virtual `===` may avoid materialization

Because transparent-product exactness is intrinsic after P3, two compatible virtual products may be compared leaf-by-leaf.

Conceptually:

```text
compare product semantic kind

Data:
    compare exact type recipe/result

Tuple:
    compare Tuple shape semantics

Record:
    compare key set, independent of presentation

then recursively compare leaves
```

Do not compare ProductShapeId blindly if two semantically equivalent shapes can have different non-semantic descriptor metadata.

For Record, presentation-order difference must not make virtual `===` false.

Unsupported virtual-vs-materialized combinations may conservatively materialize the virtual side and call the canonical runtime helper.

Correctness first.

---

# 31. Continue materializing for ordinary `==`, hash, toString

Unless existing guarded compiler machinery already proves a send safe to bypass:

```text
==      -> materialize
hash    -> materialize
toString -> materialize
ordinary getter/method/index -> materialize
```

Do not widen P3 into arbitrary method devirtualization.

---

# 32. Mixed recursive flattening

P3 should flatten across:

```text
data
Tuple
Record
```

when construction expressions are themselves exact transparent products.

Example:

```phalcom
data Point(x: Int, y: Int)

data Envelope(
  point: Point,
  metadata: #{ code: Int, ok: Bool },
)

const value = (
  Envelope(
    point: Point(x: 1, y: 2),
    metadata: #{ ok: true, code: 9 },
  ),
  label: #{ value: Point(x: 3, y: 4) },
)
```

An eligible plan may contain only scalar `Value` leaves.

No intermediate:

```text
Point
Record
Envelope
Tuple
Record
Point
```

allocations are required.

But each nested node must retain its own rematerialization metadata.

For Record specifically:

```text
presentation shape must survive flattening
```

Do not assume a canonical Record TypeId can recover source presentation order later.

---

# 33. Selective nested rematerialization

If:

```text
outer virtual Tuple
    contains virtual Record
        contains virtual data
```

and code needs only the Record as a whole:

```text
materialize Record
```

not:

```text
materialize outer Tuple
```

The recursive shape tree must support:

```text
subshape location
subshape recipe
subshape presentation metadata
```

P2 already has subshape helpers for nested data.

Generalize them rather than introducing a second tree.

---

# 34. Preserve the global leaf budget

Keep P2's centralized:

```rust
MAX_VIRTUAL_PRODUCT_LEAVES = 8
```

unless final C5 performance evidence explicitly justifies changing it.

Do not increase it merely because nested Tuple/Record examples exceed eight leaves.

A product beyond the budget falls back.

This is not a semantic failure.

Checked arithmetic is mandatory when summing nested leaf counts.

---

# 35. Enum virtualization remains stricter

Do not apply new transparent-product identity semantics to enum case objects.

P3 changes:

```text
data
Tuple
Record
```

not general enum allocation identity.

Existing P2 exact-case virtualization remains subject to its stricter rule:

```text
whole enum case must never become observable
```

Do not add enum rematerialization merely because data/Tuple/Record can be rematerialized.

Do not modify NativeOption generalization.

---

# 36. Runtime generic reification is a core P3 requirement, not optional follow-up

This is the deepest new subsystem in P3.

Problem:

```phalcom
fn make<T>(_ x: T) -> Dynamic {
  Point<T>(x: x, y: x)
}

make(42)
```

Shared bytecode knows:

```text
Point<T>
```

Runtime must materialize/reify:

```text
Point<Int>
```

not:

```text
Point
Point<T>
Point<Dynamic>
```

P1's closed descriptor path is insufficient when exact type depends on runtime generic activation evidence.

P3 therefore introduces:

```text
RuntimeTypeRecipe
RuntimeTypeEnvironmentId
RuntimeTypeEnvironment
```

---

# 37. RuntimeTypeRecipe semantics

Conceptually:

```rust
enum RuntimeTypeRecipe {
    Closed(RuntimeTypeRef),
    Template(RuntimeTypeRef),
}
```

`Closed`:

```text
already exact
```

`Template`:

```text
retained metadata type graph may contain stable type-parameter refs
```

Instantiation:

```text
Template(Point<T>)
+
environment { T -> Int }
    ↓
Point<Int>
```

Use existing runtime overlay/type descriptor interning.

Do not mutate compiler metadata.

Do not eagerly clone entire type graphs.

Do not create applied runtime ClassObjects.

---

# 38. Runtime type environment semantics

Conceptually:

```rust
#[repr(transparent)]
struct RuntimeTypeEnvironmentId(u32);

struct RuntimeTypeEnvironment {
    bindings: Box<[
        (StableTypeParameterRef, RuntimeTypeRef)
    ]>,
}
```

Environment identity is compact.

The registry is VM-owned.

Bindings must be structurally canonical/deterministic.

No environment contains runtime Values.

It contains type evidence only.

Therefore it should not create GC object cycles.

---

# 39. Narrowest-lifetime rule for generic evidence

Use the shortest lifetime capable of preserving semantics.

```text
compile-time closed fact
    ↓ if enough
no runtime evidence needed

runtime generic invocation
    ↓
frame RuntimeTypeEnvironmentId

escaping universal product
    ↓
closed RuntimeDataDescriptorId /
RuntimeAnonymousProductDescriptorId
```

Do not jump straight to:

```rust
DataObject {
    generic_arguments: Vec<RuntimeTypeRef>
}
```

Never attach per-value generic vectors.

---

# 40. CallFrame integration

The plan's default design adds:

```rust
pub type RuntimeTypeEnvironmentId = ...

pub struct CallFrame {
    // existing fields
    type_environment: RuntimeTypeEnvironmentId,
}
```

Keep:

```rust
CallFrame: Copy
```

unless current implementation has changed that invariant.

Before widening the frame, inspect:

```text
size_of::<CallFrame>()
fiber stack footprint
copy/move sites
hot send paths
```

If adding one compact integer materially violates a current frame-size invariant, stop and record an incident before inventing a bigger mechanism.

The allowed alternative is a compact side-table/key architecture preserving identical semantics.

You may choose between:

```text
direct compact ID in CallFrame
```

and:

```text
equivalent compact side metadata
```

only if repository evidence makes the direct field materially unsuitable.

You may not change the semantic model.

---

# 41. Generic call-site environments come from semantic inference

The VM must not infer generic types from runtime argument classes.

Wrong:

```text
argument runtime class = Int
therefore T = Int
```

Correct:

```text
semantic generic application selected:
    T -> Int

compiler exports call-site environment recipe

VM resolves recipe against caller environment

callee frame receives environment ID
```

This matters for:

```text
subtyping
unions
phantom parameters
higher-order calls
generic types whose runtime value class is erased
```

Runtime argument-class guessing is unsound.

---

# 42. Nested generic calls require environment composition

Example:

```phalcom
fn outer<T>(_ x: T) {
  inner<List<T>>( ...)
}
```

The call-site environment recipe may itself mention caller parameters.

The callee environment must be formed by:

```text
caller environment
+
compiler-published substitution recipe
    ↓
closed callee bindings
```

Do not require every call site to contain only closed runtime type refs.

Support substitution through the caller environment.

---

# 43. Lexical blocks must capture runtime type evidence

A block created inside a generic activation may outlive that activation:

```phalcom
fn maker<T>(_ x: T) {
  return {
    Point<T>(x: x, y: x)
  }
}
```

Ordinary value upvalues do not preserve:

```text
T -> Int
```

unless that type evidence is separately captured.

Therefore escaping lexical `BlockObject` captures:

```text
RuntimeTypeEnvironmentId
```

alongside its existing activation metadata.

When the block runs later, it uses that captured type environment.

Do not duplicate bindings per block if environment IDs are interned.

---

# 44. Fibers preserve environments naturally

Fiber suspension is not a materialization or reification boundary.

If:

```text
CallFrame contains environment ID
```

and frames already survive fiber suspension:

```text
type evidence survives with frame
```

Do not eagerly close product descriptors solely because a fiber yields.

Likewise virtual `Value` leaves remain normal stack roots.

Suspension should not force transparent product allocation.

---

# 45. Reflective/dynamic invocation must fail closed when generic evidence is absent

Some runtime invocation paths may invoke a callable without compiler-resolved static generic application.

Audit:

```text
first-class callable invocation
bound methods
reflective invoke
dynamic associated invocation
constructor invocation
block invocation
```

If a generic callable requires runtime type evidence and that path has none:

```text
use existing underconstrained/invalid semantics
or
internal fail-closed path
```

Do not synthesize type arguments from argument runtime classes.

Do not replace them with `Dynamic`.

Do not use declaration raw generic form as if it were exact.

---

# 46. Materializing generic data

At materialization:

```text
DataConstructionLoweringSpec
    contains RuntimeTypeRecipe
+
current RuntimeTypeEnvironment
    ↓
closed RuntimeTypeRef
    ↓
RuntimeDataDescriptor interning
    ↓
DataObject or DataSingleton
```

For:

```phalcom
data Id<K>(_ value: Int)
```

under:

```text
K = User
K = Order
```

you may get:

```text
same ProductLayoutId
different RuntimeDataDescriptorId
different exact RuntimeTypeRef
```

That is correct.

---

# 47. Nullary generic data remains exact

For:

```phalcom
data Signal<State>()
```

these are distinct:

```phalcom
Signal<Connected>()
Signal<Disconnected>()
```

even though both have zero payload words.

Runtime:

```text
DataSingleton(descriptor for Signal<Connected>)
DataSingleton(descriptor for Signal<Disconnected>)
```

No generic arg array.

No heap DataObject.

No specialized ClassObject.

P3 must prove this still works through:

```text
generic activation
Dynamic
reflection
```

---

# 48. Materializing generic Tuple and Record

Examples:

```phalcom
fn pair<T>(_ x: T) -> Dynamic {
  (x, x)
}
```

when called with `Int`:

```text
anonymous descriptor exact type:
    (Int, Int)
```

if semantic analysis retained/exported that exact structural type.

Likewise:

```phalcom
fn record<T>(_ x: T) -> Dynamic {
  #{ value: x }
}
```

may close to the exact structural Record type if the semantic analyzer actually established one.

If it did not, do not fabricate one.

---

# 49. `.class` versus exact type remains separate

For:

```phalcom
const p: Point<Int> = ...
```

the model remains:

```text
p.class
    -> Point behavior class

exact value type
    -> Point<Int> runtime type descriptor
```

For Tuple:

```text
t.class
    -> Tuple

exact type
    -> structural tuple descriptor
```

For Record:

```text
r.class
    -> Record

exact type
    -> structural record descriptor when semantically retained
```

Never create:

```text
Point<Int> ClassObject
Tuple<Int,Int> ClassObject
Record{name:Int} ClassObject
```

---

# 50. Product descriptors are not reflection objects

Runtime semantic handles/descriptors may be compact and strongly registry-owned as implementation metadata.

User-facing reflection objects/descriptors may be weakly cached according to existing runtime typing architecture.

Do not make a reflected wrapper object's liveness define semantic type identity.

If the reflection object is collected, reification must reproduce the same semantic type descriptor identity/meaning.

---

# 51. Static product compiler opcodes

The plan proposes dedicated static opcodes:

```rust
BuildStaticTuple { spec: u16 }
BuildStaticRecord { spec: u16 }
```

This direction is deliberate.

Do not overload:

```text
BuildTuple
BuildRecord
```

with hidden dual semantics if that makes dynamic pack construction ambiguous.

Static opcodes:

```text
consume component values
do not consume known labels as Values
reference compiler semantic spec
```

Dynamic opcodes remain valid for:

```text
computed label
spread
pack
runtime-assembled products
```

Follow the repository's stable opcode numbering discipline.

Update:

```text
decoder
disassembler
stack effects
VM dispatch
tests
```

---

# 52. Static materialization must not push label Values

For:

```phalcom
#{ name: "A", age: 24 }
```

if labels are statically known:

do not push:

```text
#name
"A"
#age
24
```

merely to recover shape.

Emit only the component values required by storage.

The spec carries labels/presentation mapping.

This is part of the allocation/memory improvement.

---

# 53. Dynamic products use universal `Value` layouts

When shape is runtime-computed, P3 does not need to invent dynamic primitive layout specialization.

Safe floor:

```text
one Value slot per component
```

using ProductStorage.

This still removes:

```text
per-instance labels array
legacy Box<[Value]> representation
```

while retaining correctness.

Do not overextend C1 into dynamic adaptive layout specialization.

---

# 54. GC rules

GC correctness is release-blocking.

Materialized ProductStorage uses the shared ProductLayout trace plan.

For each slot:

```text
Int64   -> no GC edge
Float64 -> no GC edge
Bool    -> no GC edge
Symbol  -> according to Symbol representation
Value   -> inspect Value GC edge
```

TupleObject/RecordObject tracing must call shared storage tracing.

Do not copy trace maps into each object.

Virtual products require no new GC machinery because leaves are ordinary `Value` frame slots.

Hostile tests:

```text
only live object reference exists inside packed Tuple Value slot
GC
object survives
```

Same for Record.

Also:

```text
only live reference exists in virtual Tuple/Record leaf slot
GC
object survives
```

---

# 55. Record equality/hash must use labels, not presentation position

After canonical storage migration, an easy bug is:

```rust
zip(lhs.presentation_values, rhs.presentation_values)
```

for Record equality.

Wrong.

Use:

```text
key -> logical coordinate
```

or canonical logical order.

For:

```text
#{a:1,b:2}
#{b:2,a:1}
```

both:

```text
==
===
hash
```

must agree despite differing presentation sequence.

Iteration/render still differs.

Test all four together.

---

# 56. Tuple label ordering remains semantic

Do not apply Record canonical permutation rules to Tuple.

Tuple:

```text
ordered coordinate sequence
```

Labels describe an ordered labeled suffix.

Therefore:

```phalcom
(x: 1, y: 2)
```

and:

```phalcom
(y: 2, x: 1)
```

are distinct Tuple shapes/types.

Tuple logical order should ordinarily remain source tuple coordinate order.

---

# 57. Anonymous descriptor registry interning

Intern by runtime descriptor facts:

```text
kind
shape
layout
optional exact type
```

Do not infer `exact_type` from runtime components.

Two descriptors may share layout.

Two Record descriptors may share exact type and layout but differ by presentation shape.

Lookup caches may exist beside shape metadata.

Do not include optimization-only caches in semantic hashing/identity.

---

# 58. Shape lookup performance

Current Tuple/Record label lookup is linear because labels are per instance.

P3 moves labels to shared metadata.

You may add a shared lookup cache such as:

```text
Symbol -> logical coordinate
```

on interned shape metadata.

That cache:

```text
is not ProductShape semantic identity
is not serialized deterministic metadata
is not copied per value
```

Do not make Record hot lookup slower merely because storage became canonical.

---

# 59. Source-order effects under scalar replacement

Every source expression executes:

```text
once
in source order
with the same error timing
```

Example:

```phalcom
const r = #{
  b: sideB(),
  a: sideA(),
}
```

Even if storage becomes:

```text
a
b
```

run:

```text
sideB()
sideA()
```

If `sideB()` raises:

```text
sideA() is never run
```

Scalar replacement cannot change this.

Nested flattening must preserve the full source expression tree order, not merely flattened logical leaf order.

---

# 60. Do not optimize away unused component effects

Example:

```phalcom
const t = (
  pureLooking(),
  effectful(),
)

useOnlyFirst(t)
```

Even if only first leaf is ever observed:

```text
effectful() still executes
```

Product scalar replacement is not dead-code elimination of constructor arguments.

---

# 61. Enabled/Disabled remains the semantic oracle

For all P3 optimizer changes:

```text
ProductOptimizationMode::Disabled
    = canonical materialized execution oracle
```

Enabled may change:

```text
bytecode
allocation count
performance
```

It may not change:

```text
source diagnostics
return values
stdout
errors
stack traces where stable
side-effect order
reflection results
.class
===
==
hash
presentation
```

Use the same semantic snapshot/lowering where the harness supports it.

---

# 62. Differential test matrix is mandatory

At minimum include:

## Tuple

```text
literal -> structural projection
local -> destructuring
nested Tuple/Data/Record
one materialization boundary
.class
exact type observation
virtual/virtual ===
ordinary send bailout
computed/dynamic shape bailout
pack expansion bailout
```

## Record

```text
source order differs from canonical order
direct structural projection
destructuring
presentation order after rematerialization
same-key/different-order ===
same-key/different-order ==
same-key/different-order hash
nested Record/Data/Tuple
one materialization boundary
.class
exact type observation
ordinary getter/index bailout
computed key/spread bailout
```

## Cross-cutting

```text
source effects
throwing expression
GC
fiber suspension
block capture fallback
mutation fallback
alias fallback if P2 still does
leaf budget
Disabled oracle
```

---

# 63. Allocation evidence must be deterministic

Do not use only:

```text
heap live count at end
```

Use:

```text
disassembly/opcode evidence
+
allocation counter
+
object/storage size assertions
```

Expected examples:

```text
eligible Tuple -> projection:
    no BuildStaticTuple
    zero TupleObject allocations

eligible Record -> destructuring:
    no BuildStaticRecord
    zero RecordObject allocations

materialized static Tuple(Int,Int):
    one TupleObject
    two scalar storage words
    no per-instance labels

materialized static Record(Int,Int):
    one RecordObject
    two scalar storage words
    no per-instance labels
```

---

# 64. Dynamic boundary means materialize truthfully

Example:

```phalcom
const p = Point(x: 1, y: 2)
const d: Dynamic = p
```

If `p` was virtual:

```text
virtual leaves
    ↓
Dynamic boundary
    ↓
canonical materialization
    ↓
exact RuntimeDataDescriptor
```

Dynamic is not a reason to erase type identity.

Likewise Tuple/Record crossing Dynamic materialize to universal `Value` with truthful descriptor metadata where semantic exact type is known.

---

# 65. Dynamic component does not poison outer packing

Example:

```phalcom
data Event(
  sequence: Int,
  payload: Dynamic,
  valid: Bool,
)
```

P3 must preserve P1 layout:

```text
Int64
Value
Bool
```

not:

```text
Value
Value
Value
```

The existence of Dynamic inside one component does not make the whole product semantically Dynamic.

---

# 66. Broad Record and Dynamic Record distinction

A broad runtime Record can still have an operational ProductShape.

Example:

```text
runtime presentation:
    [a, b, c]

layout:
    Value, Value, Value

exact semantic type:
    absent/broad according to checker result
```

That is correct.

Do not require exact closed type metadata simply to operate on the Record.

---

# 67. Incrementality

P3 optimizer decisions remain compiler-local.

Do not create:

```text
QueryKey::ProductOptimization
```

or a semantic DB containing virtual product plans.

Semantic DB owns:

```text
type identity
BindingId
expression facts
generic inference
source identities
```

Compiler derives:

```text
ProductFunctionPlan
VirtualShapePlan
allocation decisions
```

Runtime owns:

```text
ProductShapeRegistry
ProductLayoutRegistry
type environment registry
runtime descriptors
```

Keep those layers separate.

---

# 68. RuntimeTypeRecipe metadata must be deterministic

Compiler semantic artifacts that encode type recipes must survive:

```text
cold compile
incremental compile
artifact load
```

with the same semantic meaning.

Do not serialize store-local `TypeId`.

Use existing stable runtime type metadata references.

Any runtime template parameter reference must be a stable parameter reference.

---

# 69. LSP must remain unaware of physical representation

Optimizer mode must not affect:

```text
hover type
go-to-definition
references
semantic tokens
source occurrences
binding IDs
record field semantics
diagnostics
```

P3 should require almost no language-server behavior change.

If LSP output changes because Tuple now uses ProductStorage, something is architecturally wrong.

---

# 70. Reflection boundary

Do not invent a P3-specific public reflection API.

Use whatever exact type observation/reflection operations currently exist.

If exact value-type reflection is not yet exposed publicly, use internal runtime descriptor assertions to verify reification.

P3 implements the substrate.

LANG005.C7 will expose the full final reflection surface.

---

# 71. Weak reflection caches are not semantic evidence

If runtime reflection wrappers are weakly cached:

```text
wrapper collected
```

must not erase:

```text
RuntimeTypeRef
RuntimeTypeEnvironment
RuntimeDataDescriptor semantic identity
```

Keep semantic metadata and reflective heap wrappers separate.

---

# 72. No per-specialization classes

Do not implement:

```text
Point<Int> -> runtime class
Point<String> -> runtime class
```

The architecture remains:

```text
behavior class:
    Point

exact runtime type descriptor:
    Point<Int>
```

Same for generic data, Tuple, Record synthetic structural type forms.

---

# 73. No per-value generic arguments

Run negative searches after C4.

There should be no design like:

```rust
struct DataObject {
    generic_arguments: Box<[RuntimeTypeRef]>,
}

struct TupleObject {
    type_arguments: ...
}

struct RecordObject {
    type_arguments: ...
}
```

Exact materialized values carry compact descriptor IDs only.

---

# 74. Runtime generic environment registry must not retain Values

Environment:

```text
parameter -> RuntimeTypeRef
```

not:

```text
parameter -> Value
```

This matters for:

```text
GC
lifetime
cycles
semantic layering
```

A runtime type environment is typing evidence, not a captured runtime environment.

---

# 75. Closure value captures and type captures are different

A generic escaping block may need:

```text
ordinary upvalue x
+
RuntimeTypeEnvironmentId
```

Do not assume capturing `x: T` tells runtime reflection what `T` is.

Example:

```phalcom
T = SomeAbstractProtocolType
x runtime class = ConcreteObject
```

Runtime class does not recover the static generic argument.

Preserve explicit type evidence.

---

# 76. Generic evidence must survive higher-order calls

Audit:

```text
callable stored in variable
bound method
block passed elsewhere
callback invoked later
fiber task
```

where generic materialization/reification may occur later.

Do not fix only direct syntactic calls.

The P3 plan explicitly requires auditing all invocation routes.

---

# 77. Missing type environment is an invariant failure, not erasure permission

If:

```text
RuntimeTypeRecipe::Template(...)
```

requires binding `T`

and current environment lacks `T`:

wrong:

```text
use Dynamic
use Object
use raw Point
infer from payload
```

correct:

```text
fail closed
```

Use current internal/runtime typing error conventions.

Add hostile tests that intentionally exercise missing bindings.

---

# 78. Cycle/budget safety in runtime type recipe instantiation

Generic/runtime type graphs may be recursive.

Do not recursively instantiate without guards.

Reuse existing runtime metadata cycle/budget conventions.

Required outcomes should distinguish:

```text
ready
unsupported
cycle
budget exhausted
missing binding
invalid metadata
```

according to existing runtime typing architecture.

Do not silently return Dynamic on budget exhaustion.

---

# 79. Hot-path caution for CallFrame

Runtime type environment propagation touches every call frame.

Keep cost minimal.

Nongeneric calls should use:

```text
RuntimeTypeEnvironmentId::EMPTY
```

with no map allocation.

Do not allocate an empty environment per call.

Intern environment only where generic substitution actually differs.

Measure frame size before/after and invocation benchmark in C5.

---

# 80. Hot-path caution for anonymous descriptors

Static product construction should reuse interned descriptors.

Do not hash full shape/layout/type graphs for every Tuple construction if compiler/VM can cache descriptor resolution in executable semantic metadata or a small runtime cache.

The semantic requirement is:

```text
shared metadata
```

not repeated registry work.

Profile/measure if needed in C5.

Do not prematurely create complex inline caches unless deterministic evidence shows descriptor lookup dominates.

---

# 81. Runtime ProductShape lookup caches

A Record shape may maintain:

```text
label -> logical index
```

for fast read.

Do not recompute a linear scan on every field lookup if a shared cache is cheap.

However:

```text
lookup cache state
```

must not participate in shape equality/interning.

---

# 82. Do not merge shape and descriptor registries

Keep:

```text
ProductShapeRegistry
RuntimeAnonymousProductDescriptorRegistry
ProductLayoutRegistry
runtime typing registry
```

as conceptually separate authorities.

This allows:

```text
same shape + different exact type
same layout + different shape
same semantic type + different presentation shape
```

which are all necessary cases.

---

# 83. Existing Tuple/Record object APIs will need deliberate breakage

Current `TupleObject` exposes:

```text
values()
positionals()
labeled_values()
labels()
get()
get_label()
labeled_entries()
```

Those methods assume metadata and values live in the object.

Do not preserve those APIs by reconstructing temporary arrays on every call.

Replace representation-assuming APIs with registry-aware views.

Call-site compiler failures are useful here: migrate each consumer explicitly.

Avoid compatibility helpers that silently perform expensive temporary reconstruction.

---

# 84. Prefer compile failures to hidden dual representation

During Tuple/Record migration it may be tempting to keep:

```rust
enum TupleStorage {
    Legacy { values, labels },
    Product { descriptor, storage },
}
```

Do not leave a permanent dual representation unless a checkpoint explicitly requires a temporary staged state.

The intended C1 completion state is one positive Tuple/Record representation.

Use short-lived commit staging if necessary, but complete C1 removes legacy arrays.

---

# 85. Static and dynamic materializations must interoperate

A Tuple built statically with primitive-width packing and the same logical Tuple built dynamically with universal Value slots must behave identically:

```text
===
==
hash
iteration
labels
at
pack expansion
reflection
.class
```

The physical layout may differ.

Tests must compare these two paths explicitly.

Same for Record.

---

# 86. Virtual and materialized values must interoperate

The same semantic value may appear:

```text
virtual
materialized compact
materialized universal
```

Exact/equality/reflection operations must be representation-independent.

If an operation cannot operate directly on a virtual value, materialize through its canonical recipe.

Never expose representation choice.

---

# 87. Record rematerialization must preserve original presentation order

Virtual Record leaves may be stored in canonical logical order.

Rematerialization must use retained shape metadata.

Wrong:

```text
rebuild record in logical sorted-key order
```

Correct:

```text
rebuild storage in logical order
attach original presentation ProductShape
```

Thus:

```phalcom
const r = #{ z: 1, a: 2 }
```

may store:

```text
a
z
```

physically but print/iterate:

```text
z
a
```

after any number of virtual/materialized transitions.

---

# 88. Mixed materialization recipes belong to each nested node

A flattened virtual shape tree must retain enough metadata to materialize any node independently.

Conceptually:

```text
VirtualShapePlan
  kind
  product recipe
  component map
  nested subplans
  leaf count
```

Do not keep only one outer construction spec.

Otherwise selective nested rematerialization becomes impossible.

---

# 89. P2 enum nodes remain scalar boundaries inside transparent products

Example:

```phalcom
const x = (
  Result::Ok(1),
  #{a: 2},
)
```

Unless the exact enum candidate independently satisfies P2 enum virtualization rules:

```text
Result::Ok(...)
```

is one scalar Value leaf inside the transparent outer product.

Do not recursively flatten enum payload merely because it physically uses ProductStorage.

Semantic optimization policy remains different.

---

# 90. Class instances remain scalar leaves

A transparent product containing:

```phalcom
someClassObject
```

stores/carries that Value as one leaf.

Do not flatten class fields.

Opaque class representation remains untouched.

---

# 91. Mutation policy remains P2 conservative

P3 does not add mutable virtual products.

Mutable/reassigned local:

```phalcom
let t = (1, 2)
t = (3, 4)
```

uses canonical materialization/fallback according to current binding semantics.

Do not implement SSA phi/product versioning.

---

# 92. Capture policy remains conservative

P3 does not implement multi-leaf product upvalues.

If an entire product or structural component semantics require capturing the aggregate under current P2 policy:

```text
fallback
```

Do not redesign closure ABI.

Runtime type environment capture for generic evidence is a separate mechanism and does not imply captured virtual aggregate support.

---

# 93. Aliasing remains conservative unless already safely implemented

Example:

```phalcom
const a = (1, 2)
const b = a
```

Do not automatically make `b` another view over virtual leaves unless P2 already has a proven alias model.

P3's optimization target is direct product construction with statically understood use.

Fallback is acceptable.

---

# 94. Whole-value rematerialization retains P2 policies

Use existing P2 allocation-sinking rules.

Important existing restriction:

```text
one static whole-value use
```

is not enough if that use may execute repeatedly in a loop.

Do not replace:

```text
one eager materialization
```

with:

```text
one materialization per loop iteration
```

for a product created outside the loop.

Preserve P2's repeated-loop whole-use bailout.

Apply it equally to Tuple/Record.

---

# 95. Static projection must be semantically proven

Do not make optimizer decisions by syntax such as:

```text
Expr::MethodCall name == "at"
index constant == 0
```

unless semantic lowering proves this is the builtin structural operation under current override/guard rules.

This checkpoint does not weaken Phalcom's dynamic dispatch.

Use compiler-owned semantic resolution.

---

# 96. C0 should be treated as a takeover audit, not blind patching

P3.C0 has four historical tasks:

```text
T1 structural product PDR/ruling
T2 P1 source-order and inheritance repairs
T3 canonical BindingId optimizer identity
T4 zero-product bypass closure
```

Current status at handoff:

```text
T1
    still required unless already landed separately

T2
    data superclass rejection appears landed
    source-order behavior must be re-verified against canonical path
    patch only remaining gaps

T3
    definitely still required:
    landed P2 contains secondary name-stack/BindingId allocation

T4
    still required:
    current commit deliberately marks empty-Tuple sites as P3 FIXME
```

Record this updated status in implementation-state before editing C1 representation.

---

# 97. C1 implementation order

Do not mix generic runtime environments into initial Tuple/Record migration.

Implement in this sequence:

```text
ProductShape registry
    ↓
anonymous descriptor registry
    ↓
static lowering spec
    ↓
TupleObject/RecordObject ProductStorage conversion
    ↓
static + dynamic finalizers
    ↓
static build bytecodes
```

Only after materialized representation is stable proceed to runtime semantic consumer migration.

This sequencing keeps representation bugs separable from reification bugs.

---

# 98. C2 implementation order

After materialized Tuple/Record are green:

```text
language ===
    ↓
Tuple/Record views
    ↓
primitives/render
    ↓
pack/rest/call consumers
    ↓
patterns/destructuring
    ↓
Record order/equality/hash/reflection/map interop
```

Do not begin P2 optimizer generalization while half the runtime still reads legacy arrays.

---

# 99. C3 implementation order

Only after all materialized consumers are representation-neutral:

```text
generalize VirtualProductKind
    ↓
candidate/use classification
    ↓
virtual Tuple/Record construction
    ↓
rematerialization
    ↓
mixed nesting
    ↓
.class / exact type / virtual ===
    ↓
Enabled/Disabled hostile matrix
```

Keep each stage independently reviewable.

---

# 100. C4 implementation order

Only after transparent-product optimization is stable:

```text
RuntimeTypeRecipe
    ↓
RuntimeTypeEnvironment registry
    ↓
frame/call-site propagation
    ↓
BlockObject capture
    ↓
descriptor instantiation
    ↓
Dynamic/reflection boundary tests
    ↓
closure/fiber lifetime tests
```

Do not debug runtime generic environments and Tuple storage migration simultaneously.

---

# 101. C5 is a release checkpoint, not documentation cleanup

C5 must prove the whole LANG005.C1 architecture.

It includes:

```text
incrementality
artifact determinism
source-index invariance
LSP invisibility
allocation counts
word counts
GC
benchmarks
negative searches
workspace tests
Clippy
full verify script
implementation-state closure
```

Do not call P3 complete after C4.

---

# 102. Required deterministic C5 assertions

At minimum prove:

```text
size_of::<Value>() == 16
```

Materialized data:

```text
(Int, Int payload)
one DataObject
two scalar words
no per-instance labels
```

Materialized Tuple:

```text
(Int, Int)
one TupleObject
two scalar words
no per-instance labels
```

Materialized Record:

```text
{name:Int, age:Int}
one RecordObject
two scalar words
no per-instance labels
shared ProductShape
```

Same Record type, different presentation:

```text
may share ProductLayout
retain distinct ProductShape
```

Eligible virtual product:

```text
zero product allocations
```

Nested eligible mixed product:

```text
zero intermediate transparent-product allocations
```

Dynamic boundary:

```text
exactly necessary materialization
no duplicate materialization
```

Phantom generic specializations:

```text
distinct exact descriptors
possibly same physical layout
```

---

# 103. Required negative searches

At C5, prove there are no remaining production paths matching:

```text
TupleObject contains Box<[Value]>
TupleObject contains Box<[Symbol]>

RecordObject contains Box<[Value]>
RecordObject contains Box<[Symbol]>

direct empty Tuple allocation
direct empty Record allocation

optimizer binding identity derived from name stack / source name
optimizer locally allocates semantic BindingId

per-value generic arguments on data/Tuple/Record

ClassObject per applied product type

ProductLayout contains labels

ProductLayoutId used as semantic type identity

runtime payload keys used to manufacture exact Record Type
```

Do not delete:

```text
Value::same_as
dynamic Tuple/Record builders
weak reflection cache
```

just because their responsibilities narrow.

---

# 104. PDR work

P3 must land the structural-product semantic ruling if it has not already been ratified into the repository.

Create/update the intended PDR equivalent to:

```text
PDR-0036 — Tuple and Record Are Transparent Structural Value Products
```

It must state:

```text
backing allocation identity unobservable
Tuple === shape + recursive ===
Record === key set + recursive per-key ===
Record presentation order observable
Unit is sole zero product
Tuple/Record == and hash retain contracts
.class remains Tuple/Record
exact type descriptor distinct from ClassId
weak refs/finalizers/address APIs cannot expose backing box
```

Search normative docs for contradictory statements about:

```text
Tuple ===
Record ===
object identity
```

Update them.

Do not leave two simultaneous normative definitions.

---

# 105. Existing opaque-object `===` remains identity-based

The transparent-product exception does not redefine class objects.

For ordinary opaque instance:

```text
=== remains existing object identity semantics
```

Do not globally reinterpret all object values structurally.

Transparent products are a deliberate exception.

---

# 106. Weak refs/finalizers must not expose backing product boxes

If Phalcom has APIs capable of observing:

```text
ObjRef lifetime
finalization
weak reference identity
address
```

transparent Tuple/Record/data backing objects must not be exposed through them as user identity.

Audit relevant runtime APIs.

Do not make a P3 ProductObject weak-referenceable merely because it is heap backed.

---

# 107. Error/failure protocol

If any checkpoint violates a core invariant, stop that checkpoint.

Classify the incident.

Use categories:

```text
C0 CONTRACT
    structural-product semantic ruling conflict

P1 BASELINE
    Disabled materialized data semantics wrong

P2 BINDING AUTHORITY
    optimizer cannot consume canonical BindingId

SHAPE
    ProductShape identity/presentation bug

LAYOUT
    ProductLayout/ProductStorage bug

STATIC LOWERING
    source/logical mapping wrong

CONSUMER MIGRATION
    runtime still assumes legacy arrays

EXACTNESS
    === differs by representation

RECORD ORDER
    canonical storage leaks into presentation

OPTIMIZER
    Enabled differs from Disabled

GC
    packed/virtual reference not traced

RUNTIME TYPE RECIPE
    semantic type cannot instantiate

TYPE ENVIRONMENT
    invocation evidence lost

CLOSURE/FIBER
    environment lifetime lost

DYNAMIC BOUNDARY
    exact type erased or fabricated

INCREMENTAL
    metadata invalidation incorrect

PLAN DRIFT
    repository changed mechanically
```

For each incident record:

```text
checkpoint/task
HEAD
minimal reproducer
expected invariant
actual result
Disabled/P1 result
owning subsystem
smallest reproducing command
next investigation action
```

Do not continue layering optimization over an unresolved semantic error.

---

# 108. Do not solve failures by weakening semantics

Forbidden shortcuts include:

```text
if exact type missing:
    use Dynamic

if environment missing:
    infer from runtime class

if Record shape difficult:
    store labels per instance again

if optimizer binding unclear:
    resolve source names in compiler

if ProductLayout does not fit:
    use layout as type identity

if method dispatch is inconvenient:
    direct-load record field

if virtual === is hard:
    compare ObjRef

if generic block escapes:
    materialize all products eagerly

if Tuple/Record migration is hard:
    keep permanent legacy/new dual representation
```

These are architecture violations, not acceptable implementation compromises.

---

# 109. Performance priorities

Optimize in this order:

```text
correctness
    ↓
representation invariants
    ↓
deterministic allocation/word-count wins
    ↓
hot-path benchmark validation
```

Do not prematurely optimize registry internals before the representation model is proven.

But also do not accept obvious regressions such as:

```text
every Record lookup scans a large label list
every Tuple construction rehashes its full shape
every nongeneric call allocates an empty type environment
```

Shared metadata exists specifically to avoid such costs.

---

# 110. Keep hot metadata out of heap objects

Tuple/Record values should contain only compact references necessary for interpretation plus ProductStorage.

Do not add:

```text
cached hash
generic argument arrays
label maps
reflection wrappers
source spans
semantic TypeId
```

to each value.

A cache may be considered later if performance evidence justifies it.

P3 does not introduce universal cached hash.

---

# 111. Preserve moving-GC friendliness

Do not put raw object pointers into ProductStorage or type environments.

Continue using:

```text
Value
ObjRef
registry IDs
```

according to existing runtime conventions.

Raw u64 slots must represent data exactly according to ProductSlotRepr and P1 checked encoding.

---

# 112. Keep ProductShape registries non-GC-owned

ProductShape and layout metadata are VM metadata.

Do not represent ProductShape itself as:

```text
Object::ProductShape
```

or another heap GC value.

Product values reference compact IDs.

---

# 113. Test static versus dynamic representation equivalence

Explicitly create logically equivalent values through:

```text
static literal path
dynamic pack/finalizer path
```

and prove:

```text
===
==
hash
render
iteration
labels
lookup
class
reflection
```

match their specified contracts.

This is essential because they may have different ProductLayouts.

---

# 114. Test virtual versus materialized representation equivalence

For each of:

```text
data
Tuple
Record
```

compare:

```text
Disabled canonical materialization
Enabled virtual execution
Enabled rematerialized execution
```

No semantic difference is allowed.

---

# 115. Test record presentation after multiple transitions

Use hostile sequence:

```text
source Record in non-canonical order
    ↓
virtualize
    ↓
select subproduct
    ↓
rematerialize
    ↓
store through Dynamic
    ↓
reflect/render/iterate
```

Original presentation order must survive every step.

---

# 116. Test generic record presentation separately from type identity

A generic Record:

```phalcom
fn make<T>(_ x: T) {
  #{ z: x, a: x }
}
```

may close exact type through environment while retaining:

```text
presentation z,a
canonical semantic field set a,z
```

Do not let runtime type recipe canonicalization erase presentation metadata.

---

# 117. Do not put presentation order in RuntimeTypeRef

Record exact structural type remains order-independent.

Presentation lives in ProductShape.

If runtime typing metadata already canonicalizes Record rows:

```text
keep it
```

Do not modify semantic record type identity to preserve encounter order.

That would be a severe type-system regression.

---

# 118. Test phantom specializations through shared layout

Required explicit test:

```phalcom
data Id<K>(_ raw: Int)

const a: Dynamic = Id<User>(42)
const b: Dynamic = Id<Order>(42)
```

Assert:

```text
layout(a) == layout(b)    // internal test allowed
exactType(a) != exactType(b)
a === b == false
```

Do not expose layout equality publicly.

---

# 119. Test nullary phantom specializations

Required:

```phalcom
data Signal<S>()

Signal<Connected>()
Signal<Disconnected>()
```

Assert:

```text
zero DataObject allocations
different exact descriptors
same declaration behavior class
different ===
correct after Dynamic round trip
```

---

# 120. Test type environment through escaping block

Example architecture test:

```phalcom
fn make<T>(_ value: T) {
  return {
    Point<T>(x: value, y: value)
  }
}

const block = make<Int>(42)
const x: Dynamic = block()
```

The resulting exact descriptor must be:

```text
Point<Int>
```

even though the defining generic frame is gone.

This test proves BlockObject type-environment capture.

---

# 121. Test type environment through fiber suspension

Generic activation:

```text
T = Int
construct product after await/yield
```

Suspend before materialization/reification.

Resume.

Exact type must remain intact.

Do not force materialization before suspension.

---

# 122. Test nested generic environments

Example:

```text
outer<T>
    calls inner<U = List<T>>
        constructs Box<U>
```

Result must reify to:

```text
Box<List<Int>>
```

for outer `T = Int`.

This proves environment recipes compose through caller environments.

---

# 123. Test parameter shadowing in runtime type environments

Generic owners matter.

Two parameters named `T` belonging to different callables must not collide.

Use stable parameter identity:

```text
owner + index
```

not parameter name.

This is analogous to the BindingId rule.

---

# 124. Do not use type parameter names as runtime environment keys

Wrong:

```rust
HashMap<String, RuntimeTypeRef>
```

Correct:

```text
StableTypeParameterRef -> RuntimeTypeRef
```

Names are presentation.

Identity is owner + index.

---

# 125. Runtime type environment interning

Canonicalize environment ordering before hashing.

Equivalent environments created from different inference traversal order must intern identically.

Example:

```text
{T -> Int, U -> String}
```

and insertion:

```text
U first, T second
```

must produce equivalent environment identity.

Use sorted stable parameter refs.

---

# 126. Recipe instantiation cache

Repeated:

```text
Template(Point<T>)
+
Env(T=Int)
```

should reuse the same runtime semantic type handle.

Do not create duplicate structural/applied overlay nodes for every materialization.

Use existing runtime overlay interning.

---

# 127. C5 benchmark requirements

Measure at least:

```text
Tuple construction throughput
Record construction throughput
Tuple component read/destructure
Record lookup/destructure
nested product construction
GC pressure
pack/rest path
generic Dynamic boundary
Enabled vs Disabled optimizer
```

Timing is supporting evidence.

Release-critical evidence is:

```text
allocation count
word count
semantic equivalence
```

---

# 128. State-file discipline

Maintain:

```text
docs/implementation/LANG005/LANG005.C1-/implementation-state.md
```

Do not create a competing P3 state file unless repository convention already changed.

Record:

```markdown
## C1.P3 base revision

## P1/P2 landed interface map

## C0 takeover audit

## Checkpoint status

## Established invariants

## Active incident

## ProductShape / descriptor final APIs

## RuntimeTypeRecipe / environment final APIs

## Enabled vs Disabled matrix

## Allocation / word-count matrix

## Generic / Dynamic reification matrix

## Negative searches

## Performance evidence

## Broad gate evidence

## Next resume action
```

Do not store private reasoning.

Store reproducible facts, commands, results, and architectural decisions from the governing plan.

---

# 129. Current implementation-state inconsistency to watch

The landed C1 implementation-state says the P2 planner does not create secondary name resolution.

The actual landed `product_opt.rs` still contains:

```text
name_stack
alloc_binding_id
resolve_visible_binding
```

Treat code as authoritative evidence.

Record this as a P3 takeover repair, not as a reason to reinterpret the plan.

After C0, the documentation and code must agree.

---

# 130. Do not weaken P2 Disabled-mode byte-for-byte oracle unnecessarily

P3 naturally changes canonical materialized Tuple/Record lowering, so old Tuple/Record bytecode may change.

For **data**, however, P3 should preserve the established P2 rule:

```text
ProductOptimizationMode::Disabled
    emits canonical materialized semantics
```

When runtime type recipes replace closed-only descriptor fields, adapt baseline representations carefully and update tests only for intentional metadata shape changes.

Do not destroy the ability to run Enabled/Disabled differential tests.

---

# 131. Compiler fail-closed behavior

Missing semantic lowering spec for a statically optimized product must not trigger heuristic recovery.

For example:

```text
compiler sees static Record literal
semantic anonymous-product lowering missing
```

Use:

```text
dynamic canonical construction
```

only if that fallback preserves exact established semantics and the semantic lowering contract explicitly permits fallback.

If the compiler had already committed to a semantic operation requiring the missing spec:

```text
internal Missing...LoweringSemantics
```

follow current P1 fail-closed precedent.

Do not reconstruct closed structural type/layout from syntax in codegen.

---

# 132. Semantic analysis remains type authority

P3 compiler may ask semantic lowering:

```text
this Tuple expression has this structural type
this Record has these logical coordinates
this occurrence refers to BindingId X
this call resolved to this structural operation
this generic call selected these type substitutions
```

The compiler does not answer those questions itself.

This separation is non-negotiable.

---

# 133. Do not make P3 dependent on C2 `impl`

C1.P3 must complete before C2.

Do not use inherent-impl infrastructure to solve Tuple/Record runtime behavior migration.

Tuple/Record existing built-in/native behavior continues through current runtime/core classes.

C2 begins only after P3 completion.

---

# 134. Scope exclusions

Do not implement any of these as part of P3:

```text
general interprocedural escape analysis
captured multi-leaf virtual-product upvalues
mutable virtual products
global/field virtual product storage
multi-value function ABI
untagged VM local/register representation
generic monomorphization
whole-program specialization
inline List<Data>/List<Tuple>
array-of-struct optimization
JIT/deoptimization
FFI layout guarantees
arbitrary ==/hash/toString devirtualization
new public reflection language syntax
enum variants-only syntax
enum behavior -> impl migration
traits
associated types
trait evidence
final data-derived behavior policy
```

If these become necessary to make P3 work, stop and report an architectural incident.

---

# 135. Checkpoint completion criteria

## C0 complete

Only when:

```text
transparent Tuple/Record semantics ratified
P1 source-order behavior verified/fixed
data incoming inheritance rejection verified
P2 uses canonical semantic BindingId
all empty Tuple/Record allocation bypasses closed
baseline green
```

## C1 complete

Only when:

```text
ProductShape registry exists
anonymous descriptor registry exists
TupleObject uses ProductStorage
RecordObject uses ProductStorage
no per-instance labels
static compact layouts work
dynamic universal layouts work
GC green
static build bytecodes work
```

## C2 complete

Only when:

```text
language === is representation-independent
all Tuple/Record runtime consumers use views
pack/rest/block paths work
patterns/destructuring work
Record order semantics proven
== / hash / reflection / map interop proven
legacy arrays gone
```

## C3 complete

Only when:

```text
Tuple/Record scalar replacement works
mixed transparent flattening works
one-use sinking works
.class can avoid materialization
exact type can avoid materialization when provable
virtual === works
ordinary sends still bail/materialize
Enabled/Disabled/GC/fiber matrix green
```

## C4 complete

Only when:

```text
RuntimeTypeRecipe works
RuntimeTypeEnvironment works
generic calls propagate environments
blocks capture environments
fibers preserve environments
generic data descriptors close exactly
generic structural descriptors close exactly
phantom/nullary exactness survives Dynamic
missing evidence fails closed
no per-value generic arrays
```

## C5 complete

Only when:

```text
incrementality green
LSP/source identity unchanged
artifact metadata deterministic
allocation/word-count gates green
performance evidence recorded
negative searches green
workspace tests green
clippy green
full verification green
state/PDR closed
```

---

# 136. Final broad validation

Use repository-current canonical commands.

At minimum:

```bash
cargo +stable fmt --all -- --check
cargo +stable check --workspace --all-targets

cargo +stable test -p phalcom-ast
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-semantic
cargo +stable test -p phalcom-type-meta
cargo +stable test -p phalcom-core
cargo +stable test -p phalcom-lsp

cargo +stable test --workspace --all-targets

cargo +stable clippy --workspace --all-targets -- -D warnings

./scripts/verify.sh --full
```

If repository toolchain/CI differs at execution time, record the canonical replacements in implementation-state first.

Run focused gates after each task before broad workspace gates.

---

# 137. Final completion statement

Do not declare LANG005.C1.P3 complete merely because Tuple and Record use ProductStorage.

The checkpoint is complete only when the following architectural statement is true and supported by executed evidence:

> Phalcom has one reusable immutable-product physical substrate across nominal `data`, enum payloads, Tuple, and Record while preserving all semantic category distinctions. Data, Tuple, and Record are transparent identity-free values and may be scalar-replaced, flattened, compactly materialized, and rematerialized without observable allocation identity. Record semantic field identity remains independent of encounter order while presentation order remains observable. Generic/runtime type evidence is retained at the narrowest lifetime that needs it—compile-time metadata, invocation environment, or compact materialized descriptor—never as per-value generic argument arrays or per-specialization classes. Dynamic and reflection boundaries reconstruct truthful exact semantic types where the semantic analyzer established them, and fail closed rather than inventing or erasing type information. Compiler optimization consumes semantic identity rather than resolving language meaning independently, and Enabled execution remains semantically equivalent to canonical Disabled execution.

At that point LANG005.C1 is closed and the next work item is:

```text
LANG005.C2 — First-Class `impl` and Behavioral Separation
```

Do not begin C2 work inside this implementation session.