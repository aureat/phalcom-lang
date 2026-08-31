# Associated Lookup Surfaces

Phalcom uses `::` for associated lookup.

Associated lookup resolves a name relative to a nominal declaration or another associated declaration.

```phalcom
Option::Some::(_)
Result::Ok::(_)
Shape::Circle(r)
Parser::parse(_, mode, context)
PrintUtilities::print::(***)
SumUtilities::sum::(*)
RecordUtilities::from::(**)
```

An associated name does not denote one universal kind of entity.

A declaration may publish one or more distinct associated surfaces under the same name. The syntactic use of the lookup determines which surface is requested.

The principal associated surfaces are:

```text
type surface
value surface
family surface
```

Variant declarations participate in the same associated-lookup model as methods and other callable families.

## Surface Selection

Surface selection is driven by use rather than by a precedence rule between declarations.

Conceptually:

```text
E::V<T>       type lookup
E::V          value lookup
E::V(...)     family lookup + invocation
E::V::*       first-class family lookup
```

These forms are related, but they are not aliases for one another.

The language must not resolve a failed lookup on one surface by silently substituting an entity from another surface.

For example, a constructor family must not automatically become the result of bare value lookup merely because no associated value exists.

---

## Variant Type Surface

A data-carrying variant introduces a nominal variant type.

```phalcom
@builtin
enum Option<T> {
    @variant Some(
      _ value: T
    )
    @variant None
}
```

The associated variant types are:

```phalcom
Option::Some<T>
Option::None<T>
```

Conceptually:

```text
Option<T>          nominal enum type
Option::Some<T>    nominal variant type
Option::None<T>    nominal variant type
```

A variant type is a proper subtype or member type of its enclosing enum according to the enum type system.

For example:

```text
Option::Some<Int> <: Option<Int>
Option::None<Int> <: Option<Int>
```

The associated path identifies the variant nominally. Variant identity is therefore preserved rather than erased into the enclosing enum type.

---

## Variant Value Surface

Bare associated lookup requests the value surface.

```phalcom
Option::Some
```

means:

```text
look up the associated value named Some on Option
```

It does not mean:

```text
obtain the Some constructor family
```

and it does not mean:

```text
construct Some
```

If `Some` has no associated singleton value, then the bare value lookup does not resolve.

For the declaration:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
}
```

this is valid:

```phalcom
Option::Some(10)
```

but this does not imply that the following expression is valid:

```phalcom
Option::Some
```

There is no `Some` value merely because there is a variant constructor named `Some`.

Likewise, a nullary constructor does not create a singleton value:

```phalcom
enum Variants {
    @variant Nullary()
}
```

```phalcom
Variants::Nullary()
```

constructs a fresh `Variants::Nullary::()` value.

The existence of that constructor does not imply the existence of a canonical value returned by:

```phalcom
Variants::Nullary
```

Nullary construction and singleton lookup are separate semantics.

If Phalcom provides declarations that introduce singleton variants or other associated values, those declarations may legitimately populate this surface.

---

## Family Surface

Callable declarations are represented through callable families.

The family associated with a name is explicitly obtained using `::*`.

For a variant constructor:

```phalcom
Option::Some::*
```

denotes the constructor family associated with the base `Some`, starting with `Option::Some`.

Conceptually:

```text
Option::Some::*
    : callable family containing the constructor signatures for Some
```

For the simple declaration:

```phalcom
enum Option<T> {
    @variant Some(
      _ value: T
    )
}
```

the family contains a constructor shape conceptually equivalent to:

```text
<T>(T) -> Option::Some<T>
```

The exact internal representation of the family is implementation-defined, but its semantic identity is not.

`::*` produces the family itself rather than invoking one of its members.

This permits families to be passed, stored, reflected upon, constrained, or otherwise manipulated as first-class callable-family entities where the type system permits.

For example:

```phalcom
const constructor = Option::Some::*
```

The resulting value represents the `Some` constructor family, not a constructed `Some` value.

---

## Constructor Invocation

Calling a variant through its associated name performs family lookup followed by ordinary callable-family resolution.

```phalcom
Option::Some(10)
```

is semantically equivalent to:

```text
1. resolve the associated name Some on Option
2. select its family surface
3. find a member compatible with the supplied call shape
4. instantiate required generic parameters
5. invoke the selected constructor
```

It is not equivalent to first evaluating:

```phalcom
Option::Some
```

as a value.

Thus:

```phalcom
Option::Some
Option::Some::*
Option::Some(10)
```

are three different operations.

Conceptually:

```text
Option::Some
    associated value lookup

Option::Some::*
    associated family lookup

Option::Some(10)
    associated family lookup + dispatch + invocation
```

The first may fail while the latter two remain valid.

---

## Nullary Constructor Invocation

The same rule applies to nullary constructors.

```phalcom
enum Variants {
    @variant Nullary()
}
```

The constructor family contains a zero-argument member:

```text
() -> Variants::Nullary
```

The explicit family is:

```phalcom
Variants::Nullary::*
```

The invocation is:

```phalcom
Variants::Nullary()
```

The bare lookup:

```phalcom
Variants::Nullary
```

still requests the value surface.

It must not be rewritten into either:

```phalcom
Variants::Nullary::*
```

or:

```phalcom
Variants::Nullary()
```

---

## Family and Value Coexistence

The same associated name may legitimately exist on several surfaces.

For example, a language construct could theoretically publish all of:

```text
Foo::bar<T>     associated type
Foo::bar        associated value
Foo::bar::*     callable family
```

These entities share an associated name but are semantically distinct.

This is not ordinary name shadowing.

They occupy different lookup surfaces.

Consequently, the presence of one does not prevent another from existing.

The resolver must preserve the distinction through semantic analysis rather than collapsing all associated declarations into a single symbol kind.

---

## Family Lookup Is Structural Over Call Shapes

A callable family is not merely a pointer to one function.

It represents a set of callable members associated under one semantic name and distinguished by their callable shapes and constraints.

Conceptually:

```text
Family F = {
    signature₁,
    signature₂,
    ...
    signatureₙ
}
```

A family may therefore respond to more than one valid invocation shape.

For example, an associated family could conceptually expose:

```text
() -> A
(Int) -> B
(String, with: Context) -> C
```

The family itself is obtained using:

```phalcom
Owner::member::*
```

A specific member is selected only when a call is made or when the surrounding type context requires a compatible callable member.

This distinction is important for overloads, constructors, methods, generic callables, typed dispatch, and higher-order programming.

---

## Variant Constructor Families

Every constructible variant publishes a constructor family.

For:

```phalcom
enum Result<T, E> {
    @variant Ok(value: T)
    @variant Err(error: E)
}
```

the associated constructor families are:

```phalcom
Result::Ok::*
Result::Err::*
```

Conceptually:

```text
Result::Ok::*
    contains <T, E>(T) -> Result::Ok<T, E>

Result::Err::*
    contains <T, E>(E) -> Result::Err<T, E>
```

Generic parameters not inferable solely from constructor arguments may be supplied by explicit specialization or contextual expected types according to the generic-inference rules.

Constructor families remain associated with the variant declaration rather than being flattened into an enum-wide constructor namespace.

---

## Generic Specialization

Associated lookup may begin from a generic or specialized owner.

Conceptually:

```phalcom
Option::Some::*
```

denotes the polymorphic constructor family.

Where supported by generic associated lookup:

```phalcom
Option<Int>::Some::*
```

denotes the same family under the substitution:

```text
T := Int
```

Likewise, invoking through a specialized owner constrains constructor resolution:

```phalcom
Option<Int>::Some(10)
```

and produces:

```text
Option::Some<Int>
```

which belongs to:

```text
Option<Int>
```

Specialization affects the family environment; it does not create a different nominal variant declaration.

---

## Variant Families and GADTs

GADT variants use the same lookup model.

```phalcom
enum Expr<T> {
    @variant Int(value: Int) : Expr<Int>
    @variant Bool(value: Bool) : Expr<Bool>
}
```

The variant type paths are:

```phalcom
Expr::Int
Expr::Bool
```

and the constructor families are:

```phalcom
Expr::Int::*
Expr::Bool::*
```

Their constructor result types retain the variant's GADT refinement:

```text
Expr::Int::*:
    (Int) -> Expr::Int
    where Expr::Int <: Expr<Int>

Expr::Bool::*:
    (Bool) -> Expr::Bool
    where Expr::Bool <: Expr<Bool>
```

Invoking:

```phalcom
Expr::Int(42)
```

therefore selects the `Int` constructor family and constructs a value whose variant identity and refined enum result are both known to the semantic analyzer.

Associated lookup does not erase that proof information.

---

## Passing Variant Constructor Families

Because family lookup is explicit, APIs may distinguish accepting a constructed value from accepting a constructor family.

For example:

```phalcom
consume(Option::Some(10))
```

passes a value.

Where the parameter accepts an appropriate callable family:

```phalcom
mapConstructor(Option::Some::*)
```

passes the constructor family.

This distinction also applies to nullary constructors:

```phalcom
enqueue(Variants::Nullary())
```

passes a newly constructed Nullary value.

```phalcom
register(Variants::Nullary::*)
```

passes the family capable of constructing Nullarys.

The type system must not treat these expressions as interchangeable.

---

## Associated Family Lookup Beyond Variants

Variant constructors use the general associated-family mechanism rather than introducing variant-specific lookup rules.

The same family projection applies to other associated callables:

```phalcom
Parser::parse::*
Factory::build::*
Collection::from::*
```

Therefore:

```phalcom
Option::Some::*
```

is not special syntax meaning "variant constructor."

It means:

```text
obtain the callable family associated with Some
```

and `Some` happens to be a variant whose associated family is its constructor family.

This uniformity allows methods, constructors, variants, overload sets, and other callable declarations to participate in the same higher-order model.

---

## Reflection

Reflection must preserve the same semantic distinctions.

A reflected variant declaration may expose, independently:

```text
variant declaration identity
variant nominal type
associated singleton value, if one exists
constructor family
constructor family members / signatures
```

Reflection must not fabricate a singleton variant value for a variant that only provides a constructor.

Likewise, reflection must not represent a constructor family as though it were the result of bare associated value lookup.

For example, reflection over:

```phalcom
enum Option<T> {
    @variant Some(value: T)
}
```

may report that `Some` has a constructor family even though:

```phalcom
Option::Some
```

has no value-surface result.

The reflective model and source-language lookup model must agree.

---

## Resolution Requirements

Associated lookup must preserve the following invariants.

```text
1. :: performs associated lookup.

2. One associated name may publish entities on multiple surfaces.

3. Surface selection is determined by syntactic and typing context.

4. Bare E::name requests the value surface.

5. E::name<T> or another type-required position requests the type surface.

6. E::name::* explicitly requests the callable-family surface.

7. E::name(...) resolves and invokes the callable-family surface.

8. A failed lookup on one surface must not silently fall back to another.

9. A variant constructor does not imply an associated singleton value.

10. A nullary variant constructor remains a constructor and produces a
    fresh value on each invocation.

11. Variant constructors use the same callable-family abstraction as
    other associated callables.

12. Variant nominal identity must survive constructor-family lookup,
    invocation, generic specialization, GADT refinement, and reflection.
```

The central semantic distinction is therefore:

```text
associated name
    ├── type surface
    ├── value surface
    └── family surface
```

For a typical payload-carrying variant:

```text
Option::Some<T>       variant type
Option::Some          no value, unless separately provided
Option::Some::*       variant constructor family
Option::Some(value)   constructor-family invocation
```

For a nullary non-singleton variant:

```text
Variants::Nullary         no value, unless separately provided
Variants::Nullary::*      () -> Variants::Nullary family
Variants::Nullary()       fresh Variants::Nullary value
```

This separation is normative. Variant identity, associated values, and callable families must not be collapsed into a single lookup entity.