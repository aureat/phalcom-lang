# Data Objects

A `data` declaration introduces a **data object type**: a nominal, transparent, shallowly immutable object type whose complete semantic instance state consists exactly of its declared **data components**.

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

A value such as:

```phalcom
Point(
  x: 10,
  y: 20,
)
```

is a **data object**.

Data objects participate in the ordinary Phalcom object model. They may have behavior, participate in generic abstraction, conform to traits, cross `Dynamic` boundaries, and be observed through reflection.

Their objecthood does not imply an observable allocation identity.

> **Core invariant**
>
> A data object is semantically determined by its exact nominal type and its declared component values, not by the allocation or physical representation used to realize it.

A data object type is:

- **nominal** — its type identity originates from its declaration;
- **transparent** — its complete semantic constituent state is defined by its declared components;
- **shallowly immutable** — component bindings cannot be replaced after construction;
- **allocation-identity-free** — backing allocation is not part of language-level identity;
- **non-inheritable** — it cannot serve as the superclass of another type;
- **representation-closed** — its semantic instance state is completely determined by its `data` declaration;
- **behavior-extensible** — authorized `impl` declarations may contribute inherent behavior.

A `data` declaration is not class sugar, a one-case enum, or an alias for an anonymous structural Record.

---

# Semantic Category

## Data declarations

A **data declaration** is a declaration introduced by the `data` keyword.

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

It introduces a new nominal data object type.

## Declaration forms

A data declaration may use a parenthesized product form or a braced record-shaped form. Both forms introduce the same semantic category of nominal data object. They differ only in their declared component/constructor shape.

```phalcom
data Point(
  x: Int,
  y: Int,
)

data Person {
  name: String
  age: Int
}
```

The parenthesized form follows Phalcom's ordinary tuple-shaped argument conventions, including positional and labeled lanes. The braced form follows the language's record-shaped component and construction conventions. Neither form implies a stable memory layout.

Generic data declarations introduce families of exact applied data object types:

```phalcom
data Box<T>(
  value: T,
)
```

Here, `Box` denotes the generic declaration, while `Box<Int>` and `Box<String>` denote distinct exact applied types.

## Data objects

A **data object** is a value whose type is a data object type.

The shorter term **data value** may be used where no ambiguity arises.

## Data components

A **data component** is one of the semantic constituent values declared by a data declaration.

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

`x` and `y` are data components.

A component is not defined as an ordinary hidden field plus a synthesized getter. Property-like access syntax does not change its semantic category.

## Primary data constructors

Every data declaration has a **primary data constructor** corresponding to its declared component structure.

The constructor belongs directly to the declaration. It is not implicitly an ordinary method named `new`.

## Transparency

A data object is **transparent** because its complete constituent semantic state is defined by the language.

Transparency does not expose physical storage.

```text
semantic transparency
    ≠
representation transparency
```

A program may know that a `Point` consists of `x` and `y` without being able to observe whether those components are boxed, flattened, scalar-replaced, stored inline, or represented in some other way.

> **Guarantee**
>
> Transparency exposes semantic constituents, never implementation layout.

---

# Nominal Identity

Every data declaration introduces a distinct nominal type identity.

```phalcom
data Position(
  x: Int,
  y: Int,
)

data Velocity(
  x: Int,
  y: Int,
)
```

`Position` and `Velocity` remain distinct types even though they have identical component structures.

```text
structural equivalence
    does not imply
nominal identity
```

A nominal data object is likewise not interchangeable with an anonymous Tuple or Record merely because the two have equivalent shapes.

> **Invariant**
>
> The declaration that introduces a data object type is part of its type identity.

---

# Semantic State

## Components are the complete state

The complete semantic instance state of a data object consists exactly of the components declared by its `data` declaration.

```phalcom
data User(
  id: Int,
  name: String,
)
```

Every `User` semantically consists of exactly:

```text
id
name
```

No inherent `impl`, subclassing mechanism, derived behavior, optimizer, or runtime representation may introduce additional semantic instance state.

An implementation may associate runtime metadata with a value or its type for purposes such as dispatch, garbage collection, exact type reification, or reflection. Such metadata is not part of the object's semantic state.

> **Representation-closure invariant**
>
> A data object's semantic state is fixed completely and exclusively by its `data` declaration.

## Components are semantic members

Data components are first-class language-level semantic entities.

Conceptually:

```text
data component
    ≠
hidden mutable field
    +
synthesized getter
```

Reflection and member analysis must preserve this distinction.

A component may occupy property-like member syntax while retaining component identity rather than callable identity.

## Logical component order

A declaration establishes a logical component order.

```phalcom
data Pair(
  first: Int,
  second: Int,
)
```

has the logical order:

```text
first
second
```

Logical component order is distinct from physical storage order. For both parenthesized and braced declarations, declaration order establishes the logical component order unless another language facility explicitly defines a different ordering for its own purpose.

An implementation may reorder physical storage when doing so preserves language semantics.

---

# Immutability

Data objects are **shallowly immutable**.

Once construction has completed, none of their component bindings may be replaced.

```phalcom
data Point(
  x: Int,
  y: Int,
)

const p = Point(x: 1, y: 2)
```

An operation that attempts to replace `p.x` with another value is invalid.

Immutability does not recursively freeze objects referenced by components.

```phalcom
data Wrapper(
  value: MutableThing,
)
```

The `value` component cannot later be rebound to another object. The `MutableThing` already referenced by the component retains whatever mutation capabilities its own type permits.

```text
data immutability
    constrains component bindings

data immutability
    does not recursively freeze referenced objects
```

> **Guarantee**
>
> Construction fixes the component bindings of a data object for the remainder of that value's lifetime.

---

# Inheritance Boundary

A data object type is **non-inheritable**.

It cannot serve as the superclass of another declaration.

This is stronger and more precise than describing data merely as “sealed.” A sealed type commonly denotes a closed inheritance family. A data object type has no subclass family at all.

Non-inheritability does not prevent:

- inherent behavior through `impl`;
- generic abstraction;
- trait conformance;
- ordinary use anywhere its type is accepted.

Inheritance, inherent behavior, and conformance are distinct mechanisms.

> **Invariant**
>
> A data object may gain behavior, but it cannot gain subclass state.

---

# Construction

## The primary constructor belongs to the declaration

For:

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

construction may be written:

```phalcom
Point(
  x: 10,
  y: 20,
)
```

The language does not define this as syntactic sugar for:

```phalcom
Point.new(...)
```

A separately declared behavior named `new`, if permitted elsewhere by the language, is distinct from the primary data constructor.

## Constructor shape follows the declaration

Constructor arguments are matched against logical components according to ordinary Phalcom argument-shape rules.

```phalcom
data Pair<A, B>(
  _ first: A,
  _ second: B,
)
```

uses the positional shape established by its declaration.

```phalcom
data Point<T>(
  x: T,
  y: T,
)
```

uses its labeled shape.

The constructor's calling convention derives from semantic component structure, not from physical storage.

Braced data declarations use the corresponding record-shaped construction form:

```phalcom
data Person {
  name: String
  age: Int
}

const person = Person {
  name: "Ada",
  age: 37,
}
```

Primary construction participates in ordinary Phalcom generic inference. When generic arguments are omitted and can be inferred from the constructor arguments and surrounding type context, construction determines the same exact applied type that an explicit generic application would denote.

## Evaluation order remains source order

Constructor argument expressions execute according to source evaluation order.

Consider:

```phalcom
Pair(
  second: side2(),
  first: side1(),
)
```

The observable evaluation sequence is:

```text
side2()
side1()
```

The resulting values are then associated with their corresponding logical components.

Three orders must remain distinct:

```text
source expression evaluation order

logical component order

physical storage order
```

> **Evaluation-order guarantee**
>
> Physical representation may never change the observable evaluation order of constructor arguments.

---

# Generic Data Objects

Data declarations participate in the ordinary Phalcom generic type system.

```phalcom
data Pair<A, B>(
  _ first: A,
  _ second: B,
)
```

Each exact application denotes its own exact data object type.

```text
Pair<Int, String>
Pair<String, Int>
```

are distinct exact applied types.

Generic inference and constraints follow the general Phalcom type system rather than a data-specific generic mechanism.

## Every generic argument participates in exact type identity

All generic arguments are semantically significant, including arguments that do not influence any component type.

```phalcom
data Id<Kind>(
  _ raw: Int,
)
```

The types:

```text
Id<User>
Id<Order>
```

are distinct exact semantic types even if both admit exactly the same physical representation.

The generic parameter `Kind` is therefore semantically meaningful despite being absent from stored component values.

```text
exact applied type identity
    ≠
physical layout identity
```

> **Generic identity invariant**
>
> An implementation must never merge exact applied data object types merely because their physical representations coincide.

---

# Nullary Data Objects

A data declaration may contain no components.

```phalcom
data Signal<State>()
```

For each exact applied nullary data object type, exactly one semantic value exists.

Therefore:

```phalcom
Signal<Connected>() === Signal<Connected>()
```

is true.

But:

```phalcom
Signal<Connected>() === Signal<Disconnected>()
```

is false because the expressions have different exact applied types.

A nullary data object is therefore a **semantic singleton per exact applied type**.

This does not require a singleton heap allocation.

An implementation may use an immediate representation, canonical descriptor, canonical object, or another equivalent mechanism.

> **Guarantee**
>
> Singleton semantics belong to the exact applied type, not to a particular heap address.

---

# Inherent Behavior

Data object types may receive inherent behavior through `impl`.

```phalcom
data Point(
  x: Int,
  y: Int,
)

impl Point {
  magnitude {
    ...
  }
}
```

The contributed behavior is owned semantically by `Point`.

An inherent `impl` is not:

- runtime class reopening;
- load-order-sensitive mutation;
- an extension-method namespace;
- a separate wrapper type.

## Behavior does not alter representation

An `impl` may contribute behavior permitted by the general implementation-declaration specification.

It may not:

- add data components;
- add hidden instance state;
- change existing components;
- introduce a superclass;
- modify the component set of the primary constructor;
- alter the semantic representation established by the data declaration.

```text
data declaration
    owns semantic representation

impl declaration
    contributes inherent behavior
```

> **Representation/behavior separation**
>
> A data declaration determines what a value *is made of*. An `impl` determines what behavior that type provides.

## Components reserve their member surface

A component is not a synthesized callable, but its property selector occupies the corresponding semantic member namespace.

Therefore:

```phalcom
data Point(
  x: Int,
)

impl Point {
  x { 42 }
}
```

is invalid.

The inherent member cannot override or shadow the data component.

---

# Exactness and Identity

## Data objects have no observable allocation identity

A data object remains semantically the same kind of value regardless of whether an implementation:

- allocates it;
- copies it;
- shares immutable backing storage;
- keeps it entirely in local execution state;
- scalar-replaces it;
- flattens its components;
- stack-represents it;
- uses an immediate encoding;
- rematerializes it later.

The following are not part of a data object's language-level identity:

```text
heap address
allocation event
box identity
storage-layout identity
physical component offsets
```

> **Allocation-independence invariant**
>
> No valid Phalcom program may distinguish otherwise equivalent data objects solely from differences in backing allocation.

## `===` is structural over an exact nominal type

Two data objects are `===` exactly when:

- they have the same exact reified nominal data object type; and
- their corresponding components are recursively `===`.

Thus:

```phalcom
Point(x: 1, y: 2) === Point(x: 1, y: 2)
```

is true whenever the corresponding component values are recursively `===`, even when the two values were independently materialized.

Generic identity participates in exactness.

```phalcom
data Id<Kind>(
  _ raw: Int,
)
```

means:

```phalcom
Id<User>(42) === Id<Order>(42)
```

is false.

The identical component value does not overcome the differing exact applied types.

> **Exactness guarantee**
>
> Data-object `===` observes exact semantic type and component values, never backing representation identity.

An implementation-level pointer or handle-identity operation must not define this language-level relation.

---

# Equality, Hashing, and Derived Behavior

This specification establishes representation-independence requirements but does not define the final policy for automatically deriving:

```text
==
hash
toString
copy/update helpers
deconstruction helpers
```

Those facilities are specified separately.

Whatever policies are adopted must obey the following constraint:

> **Representation-observation prohibition**
>
> Equality, hashing, rendering, derived behavior, or any other language-level operation must not produce different results solely because semantically equivalent data objects use different physical representations.

For example, boxing one value while scalar-replacing another cannot itself affect ordinary equality or hashing.

---

# Exact Runtime Type

Data objects preserve exact applied type information wherever that exact type is semantically established and later becomes runtime-observable.

```phalcom
fn make<T>(_ x: T) -> Dynamic {
  Point<T>(
    x: x,
    y: x,
  )
}

make(42)
```

must reify as:

```text
Point<Int>
```

not merely:

```text
Point
```

nor:

```text
Point<T>
```

nor:

```text
Point<Dynamic>
```

The same rule applies to phantom generic arguments.

```phalcom
data Id<Kind>(
  _ raw: Int,
)
```

must preserve the distinction between:

```text
Id<User>
Id<Order>
```

whenever the exact applied types were established by the program's semantics.

## Runtime observation cannot invent static precision

Exact type information must be preserved when known.

It must not be fabricated from runtime component values when it was not statically established.

Runtime inspection cannot retroactively redefine the semantic type of an existing value.

> **Reification invariant**
>
> Runtime type information must faithfully preserve semantic type knowledge: neither erase known exactness nor invent unproven exactness.

---

# `Dynamic` Boundaries

Conversion to `Dynamic` does not erase the exact runtime semantic type of a data object where that type is known.

```phalcom
const value: Dynamic = Id<User>(42)
```

must remain distinguishable, through facilities capable of observing exact type, from:

```phalcom
const value: Dynamic = Id<Order>(42)
```

even if both values have identical physical layouts.

`Dynamic` limits static knowledge available at a use site. It does not authorize the runtime to replace the actual semantic type of the value.

> **Guarantee**
>
> `Dynamic` is an abstraction boundary, not a type-identity erasure boundary.

---

# Reflection

Reflection must preserve the semantic distinctions established by data declarations.

Where the reflection system exposes the relevant concepts, it must be possible to distinguish:

- a data declaration from a class declaration;
- a data declaration from an enum declaration;
- nominal data from structural Tuple and Record types;
- a generic declaration from an exact applied data object type;
- individual data components;
- data components from ordinary callable members;
- inherent behavior from representation-defining components.

For generic data objects, reflection of an exact type must preserve semantically established generic arguments, including phantom arguments.

Reflection descriptors report semantic identities. They do not create those identities.

The allocation, caching, lifetime, or wrapper identity of a reflection object cannot change data-object semantics.

> **Reflection invariant**
>
> Reflection reveals semantic distinctions; it does not define or collapse them.

---

# Representation Freedom

The physical representation of an ordinary data object is unspecified.

A conforming implementation may use:

```text
heap allocation
stack-local representation
scalar replacement
local/register representation
component flattening
compact packing
shared immutable storage
immediate representation
rematerialization
```

or another semantically equivalent strategy.

Different exact data types may share one physical representation.

The same exact data type may also use different physical representations in different execution contexts.

Programs must not rely on:

- object headers;
- component offsets;
- allocation counts;
- pointer stability;
- backing-store sharing;
- packing strategy;
- whether a value was ever materially allocated.

> **Representation-freedom guarantee**
>
> Every representation strategy is permitted if all observable data-object semantics remain unchanged.

Unless a separate specification explicitly establishes an ABI or representation contract, a `data` declaration does not define stable external binary layout.

---

# Optimization Equivalence

Optimized and materialized execution of a data object must be observationally equivalent.

```phalcom
const p = Point(
  x: calculateX(),
  y: calculateY(),
)

use(p.x)
use(p.y)
```

may execute without creating a materialized `Point`.

Such optimization must preserve all observable semantics, including:

- expression evaluation order;
- component values;
- exact applied type;
- component access;
- `===`;
- behavior across `Dynamic`;
- reflection at observation boundaries;
- inherent behavior dispatch;
- exceptions and control flow.

A virtualized, flattened, boxed, scalar-replaced, or rematerialized `Point<Int>` is always semantically the same category of `Point<Int>` value.

> **Optimization invariant**
>
> Optimization may eliminate representation. It may not eliminate semantics.

---

# Relationship to Structural Products

Data objects share product structure with Tuple and Record but differ in type identity.

```text
Tuple / Record
    structural transparent products

data
    nominal transparent products
```

A declaration such as:

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

is not the same type as an anonymous structural product that happens to contain equivalent `x` and `y` values.

An implementation may share physical product machinery between these categories without collapsing their semantic distinction.

---

# Relationship to Enums

A data object is not a one-variant enum.

Enums denote closed sums whose semantics include variant identity.

A data declaration introduces one nominal product type directly and has no variant discriminant as part of its semantic model.

Data and enum payloads may share physical storage machinery without becoming the same language category.

> **Category invariant**
>
> Shared product representation does not imply shared type or declaration semantics.

---

# Relationship to Classes

A data object is not an ordinary class instance.

Both are objects and both may provide behavior, but they have different semantic models.

```text
class
    opaque nominal object abstraction

data
    transparent nominal immutable object abstraction
```

A class may encapsulate opaque instance representation according to class semantics.

A data object's complete constituent state is defined by its components.

An implementation may reuse class-like dispatch tables, behavior identities, runtime descriptors, or other machinery for data objects. Such reuse does not make the data declaration a class declaration.

> **Guarantee**
>
> Implementation sharing between `data` and `class` cannot erase their source-level or reflective semantic distinction.

---

# Relationship to Traits

A data object type may conform to traits.

Trait conformance is not inheritance and does not alter data-object representation.

A conformance may establish behavioral capabilities for a type such as:

```phalcom
data Point(
  x: Int,
  y: Int,
)
```

but cannot cause additional components or hidden semantic state to exist in each `Point`.

Data components may participate as witnesses for trait requirements where the trait-conformance rules permit them to do so.

The details of witness selection, associated types, conditional conformance, and conformance coherence belong to the trait specifications.

---

# Identity Boundaries

The following notions are distinct:

```text
data declaration identity

exact applied type identity

data component identity

inherent callable identity

reflection descriptor identity

physical representation identity
```

They must not be collapsed merely because an implementation can encode several of them using shared machinery.

For example:

- two exact applied types may share a physical layout;
- multiple objects may share backing storage without becoming one identity-bearing object;
- a callable introduced through `impl` belongs to the target type's behavior but is not a data component;
- a reflection descriptor reports a type identity but does not constitute that identity.

> **Foundational identity invariant**
>
> Semantic declaration identity, exact type identity, component identity, behavior identity, reflective identity, and representation identity are separate concepts.

---

# Invalid Programs

The following classes of program are invalid under data-object semantics.

## Mutating a component binding

A component cannot be replaced after construction.

## Inheriting from a data object type

A data object type cannot be used as a superclass.

## Adding representation through `impl`

An `impl` cannot add components, fields, hidden semantic instance state, or another representation-defining element.

## Colliding with a component member

An inherent member cannot occupy a selector reserved by a data component where member-resolution rules define the two as conflicting.

## Invalid primary construction

A construction must satisfy the primary constructor's argument shape and component typing requirements.

## Invalid generic application

Generic arguments must satisfy the declaration's generic constraints.

Diagnostics and recovery behavior are specified separately from semantic validity.

---

# Non-Normative Implementation Model

This section is non-normative.

A Phalcom implementation may share immutable product-storage machinery between data objects, Tuple, Record, and enum payloads.

An implementation will generally need to preserve separate notions corresponding to:

```text
nominal declaration identity

exact runtime type identity

logical component identity

behavior/dispatch identity

physical product layout
```

Multiple exact data object types may map to the same physical layout.

Generic execution may require compact runtime evidence sufficient to reify an exact applied type when a data object becomes observable.

Such evidence should represent generic substitutions established by semantic analysis rather than attempting to reconstruct static generic arguments from runtime classes of component values.

None of these implementation mechanisms are part of Phalcom source semantics.

---

# Reserved and Separately Specified Questions

The following policies are intentionally not settled by this chapter.

## Primary-constructor visibility

The access-control and visibility model of primary data constructors is specified separately.

This includes whether a data type may expose components while restricting direct raw construction.

## Automatically derived behavior

The language policy governing automatic availability, synthesis, customization, or derivation of:

```text
==
hash
toString
copy/update operations
deconstruction helpers
```

is specified separately.

## Pattern matching

This specification establishes that data components are transparent semantic constituents.

It does not define new syntax for destructuring a data object in patterns.

Any data-object pattern form belongs to the pattern-matching specification.

## Stable ABI representation

An ordinary `data` declaration does not imply a stable C ABI, FFI layout, field offset contract, or binary representation.

Any facility providing such guarantees requires an explicit specification.

## Trait semantics

Data object types may conform to traits, but trait declarations, conformance resolution, witness selection, associated types, conditional conformance, and coherence are defined independently.

---

# Governing Guarantees

Every conforming implementation of Phalcom must preserve the following guarantees for data objects.

> **Nominality**
>
> A data object's type identity derives from its declaration rather than merely from its structural shape.

> **Complete state**
>
> The declared components constitute the entirety of a data object's semantic instance state.

> **Shallow immutability**
>
> Component bindings cannot be replaced after construction.

> **Non-inheritability**
>
> A data object type cannot serve as a superclass.

> **Behavioral extensibility**
>
> Authorized `impl` declarations may add behavior without altering representation.

> **Exact generic identity**
>
> Every semantically relevant generic argument, including a phantom argument, contributes to exact applied type identity.

> **Evaluation-order preservation**
>
> Constructor evaluation follows source evaluation order independently of component or storage order.

> **Allocation independence**
>
> Backing allocation is not observable data-object identity.

> **Representation-independent exactness**
>
> `===` depends on exact applied type and recursively exact component values rather than pointer or box identity.

> **Truthful reification**
>
> Exact runtime type information known by language semantics survives materialization and `Dynamic` boundaries.

> **Representation freedom**
>
> A conforming implementation may box, flatten, scalar-replace, share, copy, or eliminate physical data-object representation without changing meaning.

> **Semantic-category preservation**
>
> Sharing runtime machinery with classes, enums, Tuple, or Record never makes those categories semantically interchangeable.

The governing rule is:

> **A data object is the value described by its exact nominal type and its declared components. Everything about how that value is physically realized is implementation freedom unless another language specification explicitly makes it observable.**