# Phalcom Language Specification — Values

## Scope

This specification defines the foundational semantics of **values** in Phalcom.

It establishes:

- what a value is;
- what it means for a Phalcom value to be an object;
- the distinction between surface values and internal runtime sentinels;
- the distinction between semantic objecthood and heap allocation;
- semantic state versus physical representation;
- semantic identity and the general meaning of exactness;
- the foundational contract of the `===` relation;
- representation freedom and optimization equivalence;
- the relationship between values and the downstream object, type, product, data, enum, reflection, and derived-behavior specifications.

This specification is intentionally more foundational than the specifications for classes, data objects, enums, traits, products, or types.

It does **not** define:

- the class/metaclass tower;
- class inheritance or method lookup;
- static type formation;
- kinds or type constructors;
- structural Tuple/Record typing;
- data-object component semantics;
- enum variant semantics;
- trait conformance;
- ordinary equality or hashing;
- reflection APIs;
- physical VM layout.

Those subjects are defined by their respective specifications.

> **Foundational rule**
>
> Every observable Phalcom runtime value is an object in the language-semantic sense. Objecthood does not imply heap allocation, pointer representation, mutable state, or allocation identity.

---

## Values

A **value** is the result of evaluating a Phalcom expression or otherwise producing an observable runtime datum according to the language semantics.

Examples include:

```phalcom
42
3.14
true
"hello"
#name
()
(1, 2)
#{ name: "Ada" }
Option::Some(42)
User
|x| { x + 1 }
```

A value may be represented by:

- an immediate machine-level payload;
- an immutable singleton representation;
- a handle to heap storage;
- a shared canonical representation;
- a compact specialized representation;
- a representation synthesized only when observation requires it.

None of those implementation choices changes whether the result is a Phalcom value.

> **Value invariant**
>
> A value is a semantic runtime result. Its status as a value is independent of how the implementation stores it.

---

## Objects

Every observable Phalcom value participates in the Phalcom object model.

Accordingly, every surface value is an **object**.

This includes values that an implementation may represent without allocating a heap object, such as integers, booleans, Unit, symbols, singleton enum cases, or other immediate values.

The language-level term **object** therefore does not mean “heap allocation.”

The following distinction is normative:

```text
value
    an observable runtime datum

object
    a value participating in Phalcom's object semantics

heap object
    one possible physical realization of an object
```

A runtime implementation may use a separate internal category for heap-backed objects. That category does not define the language meaning of objecthood.

> **Objecthood invariant**
>
> Every surface value is an object, but not every object is represented by an independently allocated heap object.

---

## Observable and Internal Values

Phalcom distinguishes **surface values** from runtime-only sentinels and implementation state.

A surface value is a value that may be produced or observed by conforming Phalcom programs.

An implementation may additionally use private runtime values for purposes such as:

- uninitialized storage;
- internal control-flow state;
- temporary compiler/VM builders;
- GC or scheduler bookkeeping;
- dispatch machinery;
- transient lowering state.

Such runtime-private values are not automatically language values.

### The private `nil` sentinel

Phalcom has no user-visible `nil`, `null`, or `undefined` value.

An implementation may use an internal `nil`-like sentinel for uninitialized or private runtime state, but that sentinel:

- has no source literal;
- is not a valid surface result;
- cannot be produced by user code;
- cannot be observed by ordinary Phalcom programs;
- must not escape a runtime boundary as though it were a language value.

Absence at the language level is represented by ordinary language constructs such as `Option`, not by the private sentinel.

> **Sentinel invariant**
>
> Private runtime sentinels are outside the surface value universe and must never become observable substitutes for language-level absence.

---

## Object Categories

Values belong to semantic categories whose detailed rules are defined by specialized specifications.

Important categories include:

```text
immediate scalar values
structural product values
data objects
enum values
class instances
class objects
callable values
module and package values
reflection descriptor values
concurrency values
other standard-library/runtime objects
```

These categories may share runtime machinery while remaining semantically distinct.

For example, an implementation may route many different values through a common class-based behavior mechanism without making those values instances of the same semantic category.

> **Category invariant**
>
> Sharing runtime representation or dispatch machinery does not merge semantic categories.

---

## Semantic State

A value's **semantic state** is the state that the language specification declares relevant to that value's meaning.

Semantic state is category-specific.

Examples include:

- the components of a transparent product;
- the payload of an enum case;
- the declared components of a data object;
- the mutable or immutable state of an identity-bearing class instance where class semantics make that state observable;
- the captured environment of a closure where callable semantics make it relevant;
- the current semantic state of a Fiber or Future where the concurrency specification exposes it.

Semantic state must be distinguished from implementation metadata.

The following are not semantic state merely because the runtime stores them:

- GC headers;
- object handles;
- arena indices;
- forwarding metadata;
- allocation generation counters;
- hidden caches;
- inline-cache state;
- runtime descriptor IDs;
- physical layout IDs;
- compiler bookkeeping;
- memory addresses.

A specialized specification may make some otherwise-internal fact observable, but absent such a rule, implementation metadata is not part of value semantics.

> **State invariant**
>
> Runtime storage is not semantic state unless a language rule makes that storage fact observable.

---

## Representation

A value's **representation** is the implementation strategy used to realize that value at runtime.

Representation may include:

- immediate tagged payloads;
- boxed storage;
- unboxed storage;
- arena handles;
- heap allocations;
- inline fields;
- compact discriminants;
- shared singleton encodings;
- scalar replacement;
- stack storage;
- specialized product layouts;
- rematerialized values;
- cached descriptors.

Representation is not itself semantic identity.

A conforming implementation may change representation across:

- optimization levels;
- compilation units;
- runtime phases;
- generic specializations;
- GC cycles;
- JIT or AOT compilation;
- materialization boundaries.

Such changes are permitted whenever all defined observations are preserved.

> **Representation invariant**
>
> Physical realization may vary freely unless another specification explicitly makes a representation property observable.

---

## Materialization

A value is **materialized** when the runtime realizes it in a form suitable for an operation that requires a concrete runtime representation.

Materialization is an implementation concept, not a source-language identity event.

An implementation may:

- materialize a value that was previously scalar-replaced;
- dematerialize or optimize away an allocation;
- recreate an equivalent representation later;
- copy a representation;
- share immutable backing;
- change representation across an abstraction boundary.

Materialization does not by itself create new semantic identity for an allocation-identity-free value.

Conversely, identity-bearing values must preserve their semantic identity even if the underlying representation moves or changes.

> **Materialization guarantee**
>
> Creating, removing, moving, copying, or reconstructing a physical representation must not alter the semantic identity discipline of the value's category.

---

## Identity

Phalcom uses several distinct notions of identity.

They must not be conflated.

At minimum, the language model distinguishes:

```text
semantic value/object identity
declaration identity
exact type identity
variant identity
member/callable identity
implementation provenance identity
conformance identity
runtime descriptor identity
physical representation identity
allocation identity
memory address
```

Not every value category possesses every kind of identity.

For example:

- a class instance may have semantic instance identity;
- a data object is allocation-identity-free;
- a Tuple is structurally exact rather than allocation-identical;
- an enum variant has semantic variant identity independent of its runtime discriminant;
- a reflection descriptor may designate another semantic entity without becoming that entity.

> **Identity separation invariant**
>
> Distinct identity domains remain distinct unless a specification explicitly defines a relationship between them.

---

## Allocation Identity

**Allocation identity** means identity derived from the continued existence of one particular runtime allocation.

Allocation identity is not universal in Phalcom.

Some categories are explicitly allocation-identity-free.

For those categories, the implementation may:

- allocate or not allocate;
- duplicate storage;
- canonicalize values;
- share storage;
- move values;
- reconstruct values;

without creating a language-observable identity difference.

Other categories may be identity-bearing.

Even then, semantic identity is not defined as a raw memory address.

A runtime may preserve semantic identity through:

- moving garbage collection;
- handles;
- forwarding;
- indirection;
- stable runtime IDs;
- other implementation techniques.

> **Allocation invariant**
>
> Memory address is never the general language definition of semantic identity.

---

## Exactness

Phalcom defines a general **exactness relation** written:

```phalcom
a === b
```

`===` asks whether two values are exactly the same according to the semantic identity discipline of their categories.

It is not ordinary equality.

It is not a request to compare raw memory.

It is not universally pointer identity.

It is not universally structural equality.

Its precise rule is category-specific, but all category-specific rules must satisfy the foundational guarantees in this specification.

Examples of category-specific exactness include:

```text
identity-bearing object
    same semantic instance identity

data object
    same exact nominal applied type
    + recursively exact corresponding components

Tuple
    same structural Tuple shape
    + recursively exact corresponding components

Record
    same key set
    + recursively exact corresponding per-key values

Unit
    the unique zero-product value

enum
    case/value exactness defined by enum semantics
```

A specialized specification may provide a stricter formal definition for its category.

> **Exactness invariant**
>
> `===` observes semantic exactness, never raw allocation strategy.

---

## Exactness Is Representation-Independent

An implementation may use an optimized representation whose raw bits differ between two semantically exact values.

Conversely, two values may temporarily share identical raw representation without being semantically exact if the representation omits semantic evidence that the language preserves elsewhere.

Therefore a low-level same-bits or same-handle test is not the universal definition of `===`.

Implementations may use such tests as fast paths only when they are sound for the values being compared.

A general implementation may conceptually perform:

```text
cheap representation-exact fast path
    ↓ if inconclusive
category-specific semantic exactness
```

The source-language result must be determined by semantic exactness.

> **Exactness implementation guarantee**
>
> Representation equality may prove semantic exactness where sound; representation inequality must not refute semantic exactness for categories whose semantics permit multiple equivalent realizations.

---

## Recursive Exactness

Where a category defines exactness recursively over constituent values, the recursion uses `===`, not ordinary `==`.

For example, if a transparent product defines exactness by corresponding components, each corresponding component is compared according to that component's own exactness discipline.

This preserves composition across mixed categories.

Conceptually:

```text
Outer(a1, a2, ...)
    ===
Outer(b1, b2, ...)

iff
    exact outer semantic identity/shape matches
and
    a1 === b1
and
    a2 === b2
and
    ...
```

The enclosing category determines what qualifies as matching outer identity or shape.

---

## Exactness and Ordinary Equality

`===` and `==` are separate language relations.

`===` is governed by semantic exactness.

`==` is ordinary equality and may be:

- value-based;
- user-defined;
- derived;
- polymorphic;
- category-specific;
- numerically coercive where specified.

The existence of an ordinary equality rule does not redefine exactness.

Examples may include values for which:

```text
a == b
```

is true while:

```text
a === b
```

is false.

Likewise, two independently realized allocation-identity-free values may be `===` when their category defines exactness structurally or component-wise.

The detailed behavior of `==`, `hash`, ordering, and derived standard behavior is specified elsewhere.

> **Equality separation invariant**
>
> `==` answers the language's ordinary equality question; `===` answers the exactness question. Neither is defined as an alias of the other.

---

## Semantic Identity of Identity-Bearing Objects

For an identity-bearing object category, `===` observes the object's semantic instance identity.

That identity must survive changes in physical representation.

A runtime may move the object, relocate storage, change an arena slot, or update an internal handle strategy without changing the object's semantic identity.

Two separately created identity-bearing objects are not `===` merely because their visible state happens to be equal.

The category-specific specification determines which categories are identity-bearing.

> **Identity-bearing guarantee**
>
> Semantic instance identity is stable across representation changes and is distinct from structural state equality.

---

## Allocation-Identity-Free Values

An allocation-identity-free category defines exactness without reference to a particular allocation.

Such values may be represented:

- inline;
- on the stack;
- in registers;
- on the heap;
- by shared storage;
- by canonical singletons;
- by rematerialized temporary storage.

No program may observe a distinction solely because the implementation chose one of these realizations.

Data objects and structural products are allocation-identity-free categories according to their specialized specifications.

Their exact rules are given by their respective specifications.

> **Identity-free guarantee**
>
> Allocation, copying, sharing, or rematerialization cannot create a new semantic identity distinction for an allocation-identity-free value.

---

## Surface Absence

Absence in Phalcom is represented by language values, not by a hidden null-like runtime sentinel.

The private runtime `nil` sentinel is not surface absence.

Language-level absence is represented through the ordinary language type/value system. For example, the standard `Option` enum represents presence and absence using associated variants:

```phalcom
Option::Some(42)
Option::None
```

Consequently:

```text
uninitialized runtime storage
    may use an internal sentinel

surface absence
    uses a language value
```

A boundary that converts internal uninitialized state into a surface result must produce the language-defined surface value required by the relevant feature.

> **Absence invariant**
>
> Internal sentinel state and surface absence are different semantic domains.

---

## Immediate Values

An **immediate value** is a value whose runtime representation does not require an independently allocated heap object.

Immediate representation is an optimization/implementation category.

It does not create a separate language-semantic hierarchy.

An immediate value:

- is still a Phalcom value;
- is still an object;
- still participates in runtime behavior classification;
- may still participate in static typing;
- may still be reflected where the language permits;
- is not semantically inferior to heap-backed objects.

Examples may include numeric values, booleans, Unit, symbols, or singleton cases.

The set of immediate representations is not a stable source-language ABI.

> **Immediate-value invariant**
>
> Immediate representation is physically distinct from heap allocation but semantically transparent to ordinary objecthood.

---

## Heap-Backed Values

A value may require or use heap-backed runtime storage.

Heap-backed storage may be used for:

- mutable class instances;
- strings;
- collections;
- closures;
- modules;
- descriptors;
- fibers;
- methods;
- large numeric values;
- product values;
- data or enum payload objects;
- other runtime entities.

The fact that a value is heap-backed does not imply that its language semantics include allocation identity.

Heap backing and semantic identity are separate axes.

---

## Mutable and Immutable Values

Mutability is category-specific.

A value may be:

- deeply immutable;
- shallowly immutable;
- mutable;
- externally immutable while containing mutable referenced values;
- immutable in semantic state while backed by mutable implementation metadata.

The representation may use mutation internally even for semantically immutable values, provided the mutation cannot change defined observations.

Examples include:

- cache population;
- descriptor interning;
- lazy hash computation;
- GC relocation;
- memoization.

Such implementation mutation does not make the language value mutable.

> **Mutability invariant**
>
> Semantic mutability is determined by permitted language operations, not by whether the runtime mutates hidden representation state.

---

## Transparency and Opacity

Some object categories are **semantically transparent**.

For a transparent value, the language specifies its complete semantic constituent structure.

Other categories are **semantically opaque**.

For an opaque value, the language does not define the value merely as the structural tuple of all of its implementation state.

Transparency is a semantic property.

It does not expose:

- physical field offsets;
- object headers;
- ABI layout;
- boxing decisions;
- runtime storage format.

Accordingly:

```text
semantic transparency
    ≠
representation transparency
```

Structural products and data objects are transparent according to their specialized specifications.

Ordinary class instances are opaque object abstractions according to class/object semantics.

> **Transparency invariant**
>
> Exposing semantic constituents does not expose physical layout.

---

## State and Behavior Are Orthogonal

A value's semantic state and the behavior available on that value are different concepts.

Behavior may arise from:

- inherent members;
- class inheritance;
- exact-case behavior;
- trait conformance;
- standard-library protocols;
- other language-defined mechanisms.

Adding behavior does not by itself add semantic state.

Likewise, two categories may use the same dispatch machinery without acquiring the same state model.

For example:

```text
representation-closed
    no new semantic instance state may be added

behavior-extensible
    new behavior may be attached where authorized
```

is a valid combination.

> **Behavior/state invariant**
>
> Behavior composition does not alter semantic state unless a specification explicitly defines a state-changing declaration mechanism.

---

## Runtime Classification

Every surface value participates in runtime object classification.

The detailed meaning of:

```phalcom
value.class
```

belongs to the object-model specification.

This specification establishes only the foundational requirement:

> Every surface value has a valid runtime object-model classification.

This includes values represented immediately and values represented through heap storage.

Runtime class is not, by itself, the complete static semantic type of the value.

That distinction is defined by the type specification.

---

## Static Typing Is a Separate Relation

A value may simultaneously participate in:

```text
runtime object classification
static value typing
exact semantic type evidence
reflection
```

These are distinct relations.

A value's runtime class may help implement behavior dispatch, but the runtime class does not universally determine its complete semantic type.

For example, structurally typed products, generic applications, exact enum cases, and other semantically precise types may contain information not recoverable from one coarse runtime class.

The type and kind ontology is defined by the type specification.

> **Classification separation invariant**
>
> Runtime class and semantic type are related but are not interchangeable notions.

---

## Runtime Shape Is Not Semantic Type

An implementation may maintain runtime shape information such as:

- product arity;
- labels;
- field/key sets;
- layout IDs;
- variant payload shape;
- collection capacity;
- object storage layout.

Runtime shape may support execution and reflection.

It does not automatically prove one exact static semantic type.

Conversely, semantic type evidence may distinguish values whose operational runtime shape is identical.

Therefore:

> **Known semantic precision must not be erased, and unknown semantic precision must not be invented from runtime shape alone.**

This requirement applies especially across dynamic boundaries, reflection, generic execution, product materialization, and specialized representation.

---

## Presentation Is Not Semantic Identity

A value may have presentation properties that are observable without becoming part of its exact semantic identity.

Examples may include:

- source encounter order;
- reflection presentation order;
- rendering order;
- insertion order;
- declaration order.

A category specification determines which presentation properties are observable.

Presentation order, semantic structure, evaluation order, and physical storage order must not be conflated.

> **Order separation invariant**
>
> Semantic order, presentation order, evaluation order, and physical storage order are independent unless a specification explicitly relates them.

---

## Construction Does Not Imply Identity

Constructing a new runtime representation does not universally imply creation of a new semantic identity.

For identity-bearing categories, construction may create a fresh semantic object identity.

For allocation-identity-free categories, construction produces a semantic value whose exactness is determined by the category's value rules rather than allocation freshness.

Thus:

```text
fresh allocation
    does not universally imply
fresh semantic identity
```

and:

```text
shared representation
    does not universally imply
same semantic identity
```

The category specification determines the correct identity discipline.

---

## Copying

Copying representation is not one universal semantic operation.

A runtime may copy or move a physical representation as an implementation detail.

This does not necessarily correspond to a language-level copy operation.

For allocation-identity-free values, representation copying does not create a new identity distinction.

For identity-bearing objects, an implementation-level relocation/copy performed to preserve one object's identity must not create a second semantic object.

A source-level operation explicitly defined to create a distinct object is governed by the category-specific specification.

---

## Canonicalization

An implementation may canonicalize values where the language semantics permit it.

Canonicalization may include:

- singleton reuse;
- symbol interning;
- immutable product sharing;
- descriptor caching;
- numeric immediate normalization;
- nullary data or enum singleton reuse.

Canonicalization is legal only if it preserves all language-visible distinctions.

> **Canonicalization invariant**
>
> The implementation may merge physical realizations only where the language defines no semantic distinction between them.

---

## Optimization Equivalence

Two implementation strategies are **semantically equivalent** when no conforming Phalcom program can distinguish them using defined observations.

Valid transformations may include:

- boxing versus unboxing;
- stack versus heap storage;
- scalar replacement;
- flattening;
- inline representation;
- shared immutable storage;
- singleton representation;
- representation specialization;
- GC relocation;
- rematerialization;
- layout specialization.

For a transformation to be valid, it must preserve all relevant observations, including where applicable:

- `===`;
- `==`;
- `hash`;
- runtime behavior;
- reflection;
- static/runtime type reification;
- variant or declaration identity;
- mutability;
- iteration/presentation guarantees;
- concurrency semantics.

> **Optimization guarantee**
>
> Optimization may eliminate representation. It may not eliminate semantics.

---

## Reflection Boundary

Reflection may expose runtime objects that describe values, classes, types, declarations, members, variants, conformances, or other semantic entities.

A reflection descriptor is itself a value/object.

The descriptor does not become identical to the entity it describes merely because it designates that entity.

Conceptually:

```text
semantic entity
    ≠
reflection descriptor value
```

Descriptor caching or descriptor allocation identity must not redefine the identity of the described semantic entity.

The detailed reflection model is defined by the reflection specification.

> **Reflection invariant**
>
> Reflection observes semantic identity; it does not create the semantic identity it reports.

---

## Relationship to the Object Model

The object-model specification defines:

- runtime classes;
- `value.class`;
- `Object`;
- `Behavior`;
- `Class`;
- `Metaclass`;
- class-instance identity;
- inheritance;
- the metaclass hierarchy;
- runtime behavior lookup.

This specification is prior to that model.

It establishes that every surface value is an object and therefore must participate in the runtime object model, regardless of physical representation.

The object-model specification must not redefine “object” to mean “heap object.”

---

## Relationship to Types and Kinds

The type specification defines:

- value typing;
- nominal and structural type identity;
- exact applied types;
- type constructors;
- kinds;
- `Never`;
- `Object` as the proper static top type;
- `Dynamic`;
- subtyping;
- refinement;
- exact runtime semantic type evidence;
- type denotation and reification.

This specification establishes the runtime value universe those typing rules classify.

A value's class and its static type are distinct semantic relations.

---

## Relationship to Structural Products

The product specification defines the semantics of:

- Unit;
- Tuple;
- Record;
- structural product shape;
- product exactness;
- structural typing;
- record rows;
- presentation order;
- product representation freedom.

This specification establishes only the general rules they inherit:

- products are values and objects;
- their objecthood does not require allocation;
- their exactness must be semantic rather than raw-representation-based;
- their physical layout is implementation freedom unless otherwise specified.

---

## Relationship to Data Objects

The data-object specification defines data objects as nominal, transparent, shallowly immutable, allocation-identity-free objects whose complete semantic state consists of declared components.

Data exactness is defined by exact nominal applied type and recursive component exactness.

This specification supplies the general concepts of:

- value;
- objecthood;
- semantic state;
- exactness;
- allocation-identity freedom;
- representation freedom.

The data-object specification specializes those concepts.

---

## Relationship to Enumerated Types

The enum specification defines:

- enum root identity;
- variant identity;
- exact-case types;
- payload state;
- closed sums;
- enum behavior;
- representation-independent case semantics.

This specification supplies the general value, objecthood, identity, exactness, and representation concepts.

Runtime discriminants or specialized payload encodings must not replace semantic variant identity.

---

## Relationship to Classes

Ordinary class instances are values and objects.

The class and object-model specifications define:

- whether and how class instances possess semantic instance identity;
- field/state semantics;
- construction;
- mutation;
- inheritance;
- runtime behavior.

This specification requires only that any identity-bearing class semantics remain independent of raw memory address or allocation strategy.

---

## Relationship to Traits and Conformance

Traits are semantic contracts rather than instance-state categories.

Trait declarations and conformance do not themselves create new value state.

A conformance may affect available behavior or dispatch but must not alter the underlying object's semantic state unless another specification explicitly says so.

The traits specification remains authoritative for witness and conformance identity.

---

## Relationship to Callables

Closures, methods, bound methods, callable families, and other first-class callable entities are values/objects when exposed by the language.

Their captured state, callable identity, binding identity, and invocation semantics are defined by the callable specifications.

This specification establishes only that callable representation strategy does not alter their semantic category or defined identity.

---

## Relationship to Modules and Packages

Module and package runtime objects are values participating in the object model.

Logical import paths, module semantic identity, and runtime module descriptor/object identity are distinct concepts.

This specification does not define import resolution or module topology.

---

## Relationship to Concurrency Objects

Fibers, Futures, and other first-class concurrency entities are values/objects.

Their state and identity are governed by the concurrency specification.

Physical scheduler representation, stack storage, or VM scheduling data is not semantic state unless the concurrency model explicitly exposes it.

---

## Invalid Semantic Collapses

The following interpretations are invalid.

### Object equals heap allocation

Incorrect:

```text
only heap-backed values are objects
```

All surface values are objects.

### Runtime representation defines semantic identity

Incorrect:

```text
same raw bits implies universal semantic identity
different raw bits implies universal semantic difference
```

Category semantics define exactness.

### `===` equals pointer equality

Incorrect:

```text
a === b iff addresses match
```

Memory address is not the general semantic identity relation.

### `===` equals ordinary equality

Incorrect:

```text
=== is merely a stricter spelling of ==
```

The relations have different semantic roles.

### Hidden sentinel equals surface absence

Incorrect:

```text
runtime nil is the language's absence value
```

Surface absence is represented by language values.

### Runtime class equals complete semantic type

Incorrect:

```text
value.class fully determines the value's static/exact semantic type
```

Runtime class and semantic type are distinct relations.

### Reflection descriptor equals described entity

Incorrect:

```text
a type/class/member descriptor object is the semantic entity itself
```

A descriptor designates or reifies an entity.

### Transparent equals physically exposed

Incorrect:

```text
semantic constituents fix memory layout
```

Transparency exposes semantic structure, not storage format.

---

## Required Implementation Properties

A conforming implementation must ensure:

- every surface value participates in runtime object classification;
- private runtime sentinels do not escape as surface values;
- category-specific exactness is preserved across representation strategies;
- moving or compacting storage does not alter semantic identity;
- allocation-identity-free values do not acquire observable identity from incidental allocation;
- identity-bearing values retain identity across legal representation changes;
- semantic type evidence is not reconstructed unsafely from runtime shape;
- reflection does not become the authority that creates semantic identity;
- optimized representations preserve all defined observations.

Implementations may use fast paths, tags, handles, cached descriptors, synthetic behavior classes, or specialized layouts provided these requirements remain true.

---

## Non-Normative Runtime Correspondence

A conforming implementation may use a representation resembling:

```text
Value
    immediate scalar / singleton payload
    or
    reference/handle to heap Object

heap Object
    class instance
    class
    method
    closure
    collection
    product
    module
    descriptor
    fiber
    data object
    enum payload object
    ...
```

Such an implementation model is compatible with this specification because the language-level object category is broader than the heap-object subset.

Likewise, low-level implementation helpers may compare:

- tags;
- payload words;
- handles;
- descriptor IDs;
- layout IDs;

as optimizations.

Those helpers are not automatically the normative definitions of language identity or exactness.

---

## Governing Guarantees

> **Universal objecthood**
>
> Every observable Phalcom value is an object, regardless of whether it is represented immediately or through heap storage.

> **No surface null sentinel**
>
> Runtime-private sentinel values are not part of the surface value universe. Language-level absence uses ordinary language values.

> **Representation independence**
>
> Physical representation is implementation freedom unless another specification explicitly makes a representation property observable.

> **State separation**
>
> Semantic state is distinct from runtime metadata, caches, object handles, GC state, layout IDs, and physical storage.

> **Identity separation**
>
> Semantic value identity, type identity, descriptor identity, allocation identity, and memory address are distinct concepts.

> **Exactness**
>
> `===` observes semantic exactness according to the value category. It is not universally pointer equality, same-bits equality, or ordinary equality.

> **Recursive exactness**
>
> Where a category defines exactness through constituents, those constituents are compared recursively using their own `===` semantics.

> **Allocation independence**
>
> Allocation-identity-free values remain exact according to their semantic value rules regardless of copying, sharing, canonicalization, or rematerialization.

> **Identity-bearing stability**
>
> Identity-bearing objects preserve semantic identity across permitted movement or representation changes.

> **Immediate-value equivalence**
>
> Immediate representation does not place a value outside the object model.

> **Classification separation**
>
> Runtime class, static type, exact semantic type, runtime shape, and reflection descriptor are distinct relations or entities.

> **Transparency boundary**
>
> Semantic transparency never implies stable memory layout or ABI exposure.

> **Optimization equivalence**
>
> Implementations may eliminate or transform representation, but they may not eliminate or transform defined semantics.

> **Reflection fidelity**
>
> Reflection may reify semantic entities as values, but descriptor objects do not define the identities they report.

The governing rule is:

> **A Phalcom value is what the language says is observable, not the storage strategy used to realize it. Every surface value is an object; semantic state, identity, exactness, behavior, type, reflection, and physical representation remain distinct dimensions unless another specification explicitly connects them.**
