# Phalcom Type, Protocol, Record, Callable, and Unit Semantics

**Status:** Consolidated design specification and decision ledger
**Scope:** Decisions and unresolved questions developed in the associated design conversation
**Audience:** Language designers, compiler implementers, runtime implementers, standard-library authors, tooling authors, and specification reviewers

---

## 1. Purpose

This document consolidates the language concepts, decisions, provisional directions, superseded proposals, and unresolved questions developed during the design discussion.

It is intentionally explicit about status. Some ideas were accepted and later reconsidered. Some were explored in detail without being ratified. Some are firm language decisions. The document therefore distinguishes:

- **Ratified:** accepted as the current language direction.
- **Provisional:** preferred or strongly supported, but not yet finalized.
- **Deferred:** intentionally left for later design.
- **Open:** requires further investigation or a formal decision.
- **Superseded:** previously accepted or recommended, but later reopened or replaced.

Where an earlier proposal conflicts with a later reconsideration, the later state controls.

---

## 2. Normative terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** describe normative requirements where the surrounding section is marked Ratified.

For Provisional or Open sections, those words describe proposed semantics rather than final language law.

---

## 3. Decision summary

### 3.1 Ratified decisions

The following decisions are currently accepted:

1. `Never` is the public bottom type.
2. A method without an explicit return annotation has effective return type `Dynamic`.
3. Union types use `A | B`.
4. Intersection types use `A & B`.
5. `|(_)` and `&(_)` are real reflectable operator methods; duplicate English-named core methods are not required.
6. Type aliases use `@type const`.
7. The runtime object representing a declared alias is named `TypeAlias`.
8. Type aliases are transparent rather than nominal newtypes.
9. Generic upper bounds use `<T: Bound>`.
10. Generic angle application is modeled as a real `<>(*)` operation on authorized type-constructor objects.
11. Generic binders such as `<T, U>` are lexical declaration syntax, not calls to `<>`.
12. Protocols are structural capability types, not ordinary multiple implementation inheritance.
13. Protocols are intended to be object-model-justified special class-like objects rather than unrelated descriptor objects.
14. Type annotations are reflective metadata and do not automatically control ordinary dispatch or enforce values at runtime.
15. Applied generic types do not automatically create new runtime classes.
16. `()` is the unit value and unit type.
17. `Unit` may be provided as a transparent alias for `()`.
18. `None` remains semantically distinct from unit and represents absence.
19. `Never`, unit, and `NoneType` are distinct concepts.
20. A normally completing implemented method with no explicit result returns `()`.

### 3.2 Important reopened decisions

The following were once treated as agreed but are now open again:

1. Whether labeled record fields are ordered or unordered.
2. Whether labeled method arguments are semantically ordered or unordered.
3. Whether method argument domains are literally records.
4. Whether `*rest` captures an ordinary record or a distinct argument-packet object.
5. Whether callable parameter domains use `RecordType` or a separate `ArgumentsType`.
6. Whether selectors canonicalize labeled parameters as an unordered set or preserve their declared sequence.

### 3.3 Major deferred areas

1. General bodyless-method semantics.
2. Exact protocol-default adoption and invocation API.
3. Runtime type-test API names.
4. Closed generic constraints such as `<T in A | B>`.
5. Recursive alias support in the first release.
6. Optional/default/rest parameter type syntax.
7. Postfix optional type syntax `T?`.
8. Type negation or complement.
9. Record width subtyping.
10. Exact class/type/metaclass bootstrap hierarchy.

---

# Part I — Foundational type model

## 4. Types are runtime objects

**Status: Provisional architectural direction**

Phalcom should not create a parallel type universe disconnected from the runtime object model.

The preferred model is:

> Every type is an object. Every runtime class object is a type. Not every type is a runtime class.

Conceptually:

```text
Object
└── Type
    ├── Class
    ├── ProtocolClass
    ├── AppliedType
    ├── UnionType
    ├── IntersectionType
    ├── FunctionType
    ├── RecordType
    ├── TypeAlias
    ├── TypeParameter
    ├── SelfType
    ├── TopType
    ├── BottomType
    └── DynamicType
```

This hierarchy is illustrative, not yet a final bootstrap hierarchy.

### 4.1 Why not make every type a class?

Some types do not correspond to runtime allocation classes:

```phalcom
Int | String
List<Int>
(Int, String) -> Bool
Never
```

These are genuine types, but they are not necessarily classes from which instances are directly allocated.

### 4.2 Why not create a separate annotation-descriptor hierarchy?

A disconnected descriptor universe would duplicate:

- naming;
- equality;
- reflection;
- generic application;
- subtype relations;
- method lookup for type operations;
- serialization;
- diagnostics.

The integrated model allows type operations to be ordinary reflected object behavior while preserving distinctions between runtime classes and synthetic types.

### 4.3 Two graphs must remain distinct

Implementation inheritance and type subtyping are related but not identical.

Example:

```text
Implementation inheritance:
    Dog → Animal → Object

Type subtyping:
    Dog <: Animal
    Dog <: Hashable
    Dog <: Animal & Hashable
```

Structural protocol conformance extends the subtype graph without adding implementation inheritance.

### 4.4 Open questions

- Is `Type` user-subclassable?
- Is `Type` an abstract built-in class?
- Is `ProtocolClass` a subclass of `Class`, or are they siblings under a common class-object abstraction?
- What are the exact metaclasses of `Object`, `Type`, and `Class`?
- How are synthetic types interned?
- What is the relationship between type identity and type equality?
- What is the exact meaning of `Class<T>` or metatype types?

---

## 5. Type annotations are reflective metadata

**Status: Ratified**

Type annotations do not silently change ordinary Phalcom dispatch.

They are retained metadata available to:

- reflection;
- static analysis;
- protocol conformance analysis;
- diagnostics;
- IDE tooling;
- documentation;
- explicit runtime type operations.

They do not automatically:

- enforce assigned values;
- select ordinary overloads;
- create hidden runtime wrappers;
- change the runtime class of an object.

Example:

```phalcom
parse(input: String) -> Json {
  ...
}
```

The annotation exposes a declared callable contract. It does not by itself make the runtime reject every non-`String` value unless an explicit checking mode or checking operation is used.

### 5.1 Declared versus effective types

Reflection should distinguish declared and effective metadata.

Example:

```phalcom
method() {
  ...
}
```

If no return annotation is written:

```text
declaredReturnType = absent
effectiveReturnType = Dynamic
```

This distinction is important for:

- documentation;
- protocol matching;
- diagnostics;
- preservation of source intent.

---

# Part II — Special foundational types

## 6. `Never`

**Status: Ratified**

The public name of the bottom type is:

```phalcom
Never
```

`Nothing` is not used as the public spelling.

`Never` has no values:

```text
inhabitants(Never) = ∅
```

It is a subtype of every type:

```text
Never <: T
```

for all types `T`.

### 6.1 Typical uses

```phalcom
fail(message: String) -> Never {
  throw Error.new(message)
}
```

```phalcom
loopForever() -> Never {
  while true {
    ...
  }
}
```

`Never` describes computations that do not complete normally.

### 6.2 Distinction from unit

```text
Never:
    no value exists;
    normal completion is impossible.

():
    exactly one value exists;
    normal completion occurred without meaningful information.
```

---

## 7. `Dynamic`

**Status: Ratified in the specific return-annotation rule**

When a method omits a return annotation, its effective return type is:

```phalcom
Dynamic
```

This does not mean the method necessarily returns every type. It means the declaration does not provide a statically useful return constraint.

Example:

```phalcom
compute() {
  ...
}
```

Reflection:

```text
declaredReturnType = absent
effectiveReturnType = Dynamic
```

### 7.1 Open questions

- Is `Dynamic` a top type, a gradual-typing escape hatch, or a distinct checking mode?
- Is `T <: Dynamic` always true?
- Is `Dynamic <: T` treated as true, unknown, or permitted only in gradual compatibility?
- How does `Dynamic` participate in protocol conformance?
- How does it affect union and intersection normalization?

---

## 8. Unit

**Status: Ratified**

Phalcom’s unit value and unit type are both written:

```phalcom
()
```

The empty record is the unit.

### 8.1 Semantic definition

The unit type has exactly one value:

```text
inhabitants(()) = { () }
```

It represents successful normal completion without meaningful result information.

Example:

```phalcom
save(user: User) -> () {
  database.persist(user)
}
```

### 8.2 Implicit method result

An implemented method that reaches the end of its body without an explicit result returns:

```phalcom
()
```

Example:

```phalcom
notify() {
  System.print("done")
}
```

is semantically equivalent in result behavior to:

```phalcom
notify() -> () {
  System.print("done")
  return ()
}
```

Its declared return type may still be absent and its effective annotation type may still be `Dynamic` under the general missing-annotation rule. Runtime result behavior and reflective annotation behavior are separate questions.

This produces an important distinction:

```text
Runtime fallthrough result:
    ()

Effective return annotation when omitted:
    Dynamic
```

A future static analyzer may infer `()` for particular bodies, but omission itself does not declare `()`.

### 8.3 `Unit` alias

The standard library may define:

```phalcom
@type
const Unit = ()
```

This is a transparent alias, not a new nominal type.

Therefore:

```text
Unit == ()
```

under transparent alias expansion.

### 8.4 Distinction from `None`

`None` represents absence.

```phalcom
findUser(id: Int) -> User | NoneType
```

Unit represents successful completion without result information.

```phalcom
save(user: User) -> ()
```

They are not interchangeable merely because both types have one inhabitant.

The resulting conceptual cardinalities are:

```text
Never:
    zero inhabitants

():
    one inhabitant, ()

NoneType:
    one inhabitant, None

Bool:
    two inhabitants, true and false
```

### 8.5 Open questions

- Is `Unit` a mandatory standard-library alias or merely permitted?
- Is the type of the unit value spelled `()` in reflection, `Unit`, or canonically one with the other as an alias?
- Is `()` represented by a singleton runtime object?
- Does `()` share the general record implementation?
- How does unit print in diagnostics?
- Does `return` without an expression mean `return ()`?
- May constructors explicitly declare `-> ()`, or do constructors have a special result contract?

---

## 9. `None` and optionality

**Status: Partially ratified, partly open**

`None` remains dedicated to absence.

Its type is currently referred to as:

```phalcom
NoneType
```

Optional shorthand was discussed:

```phalcom
T?
```

with intended meaning:

```phalcom
T | NoneType
```

This shorthand is not ratified.

### 9.1 Important distinction

A required parameter whose value may be absent:

```phalcom
value: T | NoneType
```

is not the same as an optional parameter that may be omitted entirely.

Optional value type and optional argument presence must remain separate concepts.

### 9.2 Open questions

- Is `NoneType` the final public type name?
- Does `T?` become a postfix type operator?
- Is postfix `?` restricted to type objects?
- Does it conflict with optional chaining `?.`?
- Does it conflict with future error-propagation syntax?
- Is `None` truthy or falsey?
- Does `None` participate in implicit conversions?

---

# Part III — Type algebra

## 10. Union types

**Status: Ratified**

Union syntax:

```phalcom
A | B
```

The operation is a real operator method:

```phalcom
A.|(B)
```

with selector:

```phalcom
#|(_)
```

The result is a `UnionType`.

### 10.1 Core semantics

A value belongs to `A | B` when it belongs to at least one member.

```text
x : A | B
iff
x : A or x : B
```

### 10.2 Reflection

A union should expose its normalized member alternatives:

```phalcom
union.members
union.alternatives
```

Exact names remain open.

### 10.3 No mandatory English duplicate

The core type interface need not also define:

```phalcom
A.union(B)
```

The operator is already:

- callable;
- reflectable;
- pinnable;
- documentable.

Duplicating the operation creates questions about which form is primitive and whether independent overriding is possible.

### 10.4 Open questions

- Flattening: is `A | (B | C)` normalized to `A | B | C`?
- Deduplication: is `A | A == A`?
- Ordering: are union members semantically unordered?
- Interning: are equivalent unions identical?
- Absorption: does `A | Never == A`?
- How does `Dynamic` behave in a union?
- How do aliases affect printed members?
- Are union operators restricted to `Type`?
- Can arbitrary objects define `|` for unrelated purposes?

---

## 11. Intersection types

**Status: Ratified**

Intersection syntax:

```phalcom
A & B
```

The operation is a real operator method:

```phalcom
A.&(B)
```

with selector:

```phalcom
#&(_)
```

The result is an `IntersectionType`.

### 11.1 Core semantics

A value belongs to `A & B` when it belongs to both members.

```text
x : A & B
iff
x : A and x : B
```

### 11.2 No mandatory English duplicate

The core interface need not add:

```phalcom
A.intersection(B)
```

for the same reasons described for unions.

### 11.3 Open questions

- Flattening and deduplication.
- Whether intersections are semantically unordered.
- `A & Any`.
- `A & Never`.
- Contradictory intersections.
- Protocol and class intersections.
- Distribution over unions.
- Runtime testing strategy.
- Normalization cost.
- User override rules for `&`.

---

## 12. Type negation

**Status: Deferred**

Possible syntax:

```phalcom
~T
```

or:

```phalcom
!T
```

would mean all values not belonging to `T`.

Example:

```phalcom
A & ~B
```

This feature is deferred because complement types interact badly with:

- open-world classes;
- structural protocols;
- dynamic loading;
- gradual typing;
- union/intersection normalization;
- runtime testing.

Optionality does not require type negation.

---

# Part IV — Type aliases

## 13. Source syntax

**Status: Ratified**

A type alias is declared with:

```phalcom
@type
const Identifier = Int | String
```

A generic alias:

```phalcom
@type
const Pair<T> = Tuple<T, T>
```

### 13.1 Why `const`

A type alias is a stable declaration.

Permitting reassignment would destabilize:

- annotations;
- protocol conformance caches;
- subtype caches;
- imported metadata;
- reflection;
- diagnostics.

Ordinary variables may still hold type objects:

```phalcom
var selectedType = Int
selectedType = String
```

Such a variable is not a declared `TypeAlias`.

---

## 14. Runtime object name

**Status: Ratified**

The runtime object representing a declared type alias is named:

```phalcom
TypeAlias
```

The source decorator remains:

```phalcom
@type
```

This separates concise source syntax from precise reflective terminology.

---

## 15. Transparency

**Status: Ratified**

Type aliases are transparent.

```phalcom
@type
const Identifier = Int | String
```

does not create a new nominal type distinct from its target.

Subtype checks, compatibility checks, and type operations expand the alias to its target semantics.

The alias may still retain declaration identity for:

- reflection;
- documentation;
- source locations;
- import/export metadata;
- diagnostics.

### 15.1 Open representation choice

Two implementations remain possible:

1. Bind the declared name directly to the target type and store alias metadata separately.
2. Bind the name to a `TypeAlias` wrapper that transparently delegates type semantics to its target.

The second model preserves stronger reflective identity but requires careful equality and expansion rules.

---

## 16. Recursive aliases

**Status: Concept explained; support not fully ratified**

A recursive alias refers to itself directly or indirectly.

Direct recursion:

```phalcom
@type
const Json =
  NoneType
  | Bool
  | Int
  | Float
  | String
  | List<Json>
  | Map<String, Json>
```

Generic recursion:

```phalcom
@type
const Tree<T> = T | List<Tree<T>>
```

Mutual recursion:

```phalcom
@type
const Expression = Literal | BinaryExpression

@type
const BinaryExpression =
  Tuple<Expression, Operator, Expression>
```

### 16.1 Predeclaration

Ordinary constant initialization usually follows:

```text
1. Evaluate initializer.
2. Bind resulting value to name.
```

That fails for recursion because the alias name is needed while evaluating its own initializer.

Predeclaration uses:

```text
1. Create the alias binding.
2. Bind an unfinished TypeAlias object.
3. Resolve the alias body in an environment where the alias exists.
4. Finalize the TypeAlias.
```

Possible states:

```phalcom
@data
@sealed
class TypeAliasState {
  @variant Declared
  @variant Resolving
  @variant Resolved(type:)
  @variant Invalid(error:)
}
```

### 16.2 Productive and unproductive recursion

Unproductive recursion:

```phalcom
@type
const A = A
```

Mutual unproductive recursion:

```phalcom
@type
const A = B

@type
const B = A
```

Productive or guarded recursion:

```phalcom
@type
const Node = Int | List<Node>
```

The recursive reference occurs through another type constructor.

### 16.3 Compiler recognition

`@type` must be compiler-recognized if generic or recursive aliases are supported because:

- generic parameters require lexical binding before body resolution;
- recursive names require predeclaration;
- a normal runtime decorator runs too late.

### 16.4 Scope restriction

A proposal was made to initially restrict `@type const` to module scope.

This is not ratified.

Reasons supporting module-only v1:

- stable identity;
- stable import/export behavior;
- deterministic declaration phases;
- easier recursive cycle analysis;
- stable diagnostics;
- simpler tooling.

Reasons to eventually allow class-scoped aliases:

```phalcom
class Response<T> {
  @type
  const Handler = (T,) -> ()
}
```

This is useful but introduces generic substitution and inheritance questions.

Local aliases introduce runtime identity and capture questions.

### 16.5 Open questions

- Are recursive aliases supported in v1?
- What exact forms count as productive recursion?
- Are aliases equirecursive or iso-recursive?
- Are class-scoped aliases permitted?
- Are local aliases permitted?
- Can aliases capture lexical values?
- Can aliases be inherited or overridden?
- How are aliases printed after normalization?
- Does equality preserve alias spelling?

---

# Part V — Generics

## 17. Generic application

**Status: Ratified direction; exact mechanics still open**

Surface syntax:

```phalcom
Map<String, User>
```

is modeled as a real operation:

```phalcom
Map.<>(String, User)
```

The generic application method accepts a rest sequence of type arguments:

```phalcom
<>(*arguments: Type) -> Type
```

Likely selector:

```phalcom
#<>(*)
```

The exact rest-selector spelling is not final.

### 17.1 Applied result

Generic application creates an `AppliedType`.

It does not automatically create a new runtime class.

```phalcom
List<Int>
```

describes a type application whose origin is `List` and whose argument is `Int`.

### 17.2 Controlled extensibility

The preferred model is controlled extensibility:

- `<>` is a real reflected operation.
- Ordinary objects cannot provide arbitrary generic-application semantics.
- Authorized type-constructor metaobjects may implement it.
- User-defined metaclass extensions are likely deferred.

This preserves checker reasoning and prevents arbitrary angle-bracket semantics from undermining the type system.

---

## 18. Generic binders

**Status: Ratified**

Declaration syntax:

```phalcom
class Map<K, V> {
  ...
}
```

does not invoke `Map.<>(K, V)`.

Instead, `<K, V>` is lexical binder syntax.

The compiler creates first-class reflective objects such as:

```phalcom
TypeParameter
GenericSignature
```

and places `K` and `V` into the declaration’s lexical type environment.

### 18.1 Open questions

- Variance syntax and defaults.
- Type-parameter identity.
- Higher-kinded types.
- Default generic arguments.
- Generic methods.
- Generic alias substitution.
- Class-side forwarding through applied types.

---

## 19. Upper bounds

**Status: Ratified**

Upper bounds use:

```phalcom
<T: Vehicle>
```

or:

```phalcom
<T: A | B>
```

A type argument satisfies the binder when it is a subtype of the bound.

### 19.1 Closed alternatives

Possible syntax:

```phalcom
<T in A | B>
```

would mean that `T` must be one of the explicitly listed alternatives rather than any subtype of their union.

This is deferred.

The distinction matters for:

- correlated generic arguments;
- exact finite alternatives;
- preserving a chosen branch;
- exhaustive generic reasoning.

---

## 20. Applied generics at runtime

**Status: Ratified principle**

Applied generic types do not automatically imply reified per-instance arguments.

Example:

```phalcom
const values: List<Int> = [1, 2, 3]
```

The runtime class may remain:

```phalcom
List
```

A check against:

```phalcom
List<Int>
```

may only prove the origin class and not the argument.

### 20.1 Reified evidence through nominal subclassing

A nominal class may carry applied-supertype evidence:

```phalcom
class IntList is List<Int> {
  ...
}
```

Then:

```phalcom
IntList.new() is List<Int>
```

may be conclusive because the class declaration itself records the applied supertype.

### 20.2 Protocol evidence

Applied protocols can often be checked through reflected signatures.

```phalcom
@protocol
class Parser<I, out O> {
  parse(input: I) -> O
}
```

A class:

```phalcom
class JsonParser {
  parse(input: String) -> Json {
    ...
  }
}
```

may conclusively satisfy:

```phalcom
Parser<String, Json>
```

because generic substitution yields concrete method requirements.

Missing annotations may produce `Unknown` rather than a false match.

---

# Part VI — Runtime type testing

## 21. `is` and exact tests

**Status: Partly decided, public API names open**

The intended conceptual distinction is:

```phalcom
value is T
```

tests subtype membership or structural conformance.

An exact test should test exact runtime class or exact type-descriptor identity.

A possible operator spelling discussed was:

```phalcom
value is! T
```

This spelling is not formally ratified in this document.

### 21.1 Exactness limitations

Protocols, unions, intersections, aliases, and many applied types are not exact runtime classes.

An exact test against such types may be:

- invalid;
- always false;
- defined in terms of descriptor identity rather than value class;
- represented through a diagnostic result.

This remains open.

---

## 22. Evidenceful type-test result

**Status: Provisional**

A boolean-only type test loses important information for erased generics and structural uncertainty.

A proposed sealed result:

```phalcom
@data
@sealed
class TypeTest {
  @variant Match
  @variant NoMatch(reason)
  @variant Erased(
    matched: Bool,
    checkedType: Type,
    ignoredArguments: const List<Type>
  )
  @variant Unknown(reason)
}
```

Possible primitive operations:

```phalcom
type.test(value)
type.testExact(value)
```

Possible boolean conveniences:

```phalcom
type.matches(value)
type.matchesExactly(value)
```

Protocol-specific convenience:

```phalcom
protocol.satisfiedBy(value)
```

No names are ratified.

### 22.1 Boolean `is` with erased evidence

A direct erased check such as:

```phalcom
values is List<Int>
```

might produce an evidence result conceptually equivalent to:

```text
Erased(
  matched: true,
  checkedType: List,
  ignoredArguments: [Int]
)
```

Whether boolean `is` returns true with a warning, rejects the check, or uses another rule is open.

### 22.2 Structural checks prove contracts, not behavior

A protocol check can prove that declared callable signatures are compatible.

It cannot prove that the implementation behaves correctly.

---

# Part VII — Protocols

## 23. Protocol purpose

**Status: Ratified**

Protocols express structural capabilities and structural supertypes.

They are not ordinary multiple implementation inheritance.

A class may satisfy multiple protocols without inheriting implementation state or ordinary class ancestry from them.

Example:

```phalcom
@protocol
class Hashable {
  hash -> Int
}
```

A class conforms when its reflected surface satisfies the requirements.

---

## 24. Protocol object model

**Status: Ratified direction; exact hierarchy open**

Protocols should be specialized class-like objects integrated into the object model.

Rejected extremes:

1. Completely unrelated protocol descriptor objects.
2. Ordinary classes with only an `isProtocol` boolean and scattered special cases.

Preferred direction:

- share class/member/generic/reflection infrastructure;
- centralize protocol-specific rules in a specialized metaclass or class policy;
- prevent ordinary allocation;
- prevent instance storage and constructors;
- perform structural conformance.

### 24.1 Open questions

- Is `ProtocolClass <: Class`?
- Is `ProtocolClass` a sibling of `Class`?
- Is a protocol itself an instance of a specialized metaclass?
- Can users construct protocols without `@protocol`?
- Can protocols inherit other protocols?
- Can protocols include class-side requirements?
- Are protocol objects final or subclassable?

---

## 25. Generic protocols

**Status: Ratified principle**

Generic protocol application substitutes type arguments into requirements.

Example:

```phalcom
@protocol
class Parser<I, out O> {
  parse(input: I) -> O
}
```

Applying:

```phalcom
Parser<String, Json>
```

produces an applied requirement equivalent to:

```phalcom
parse(input: String) -> Json
```

Structural conformance compares the candidate class’s reflected method signatures against the substituted requirements.

Variance must be honored when compatibility is checked.

---

## 26. Protocol defaults

**Status: Provisional; exact semantics open**

A protocol may contain a bodyful method:

```phalcom
@protocol
class Sized {
  size -> Int

  isEmpty -> Bool {
    return self.size == 0
  }
}
```

The bodyful method offers a reusable default implementation.

The major principle discussed is:

> Protocol defaults should not silently enter every conforming class’s normal dispatch chain.

Explicit adoption or invocation is preferred.

Possible explicit invocation:

```phalcom
Sized.defaultFor(#isEmpty)
```

or:

```phalcom
Sized.defaults(on: self).isEmpty
```

Possible declarative adoption:

```phalcom
@useDefault(Sized)
isEmpty -> Bool
```

No API is ratified.

### 26.1 Remaining semantic fork

Two models remain:

#### Strict requirement model

A bodyful protocol method is still a requirement.

A class conforms only if it:

- implements the method; or
- explicitly adopts the protocol default.

#### Provided-operation model

Only bodyless methods are requirements.

Bodyful methods are optional operations available through explicit protocol-default invocation.

This means a conforming value may not respond directly to every selector declared in the protocol.

The language must choose one.

### 26.2 Conflict handling

If multiple protocols provide the same default selector:

- implicit priority should be avoided;
- explicit qualification or adoption should resolve the conflict.

Exact conflict rules remain open.

---

# Part VIII — Bodyless methods

## 27. Syntax distinction

**Status: Ratified distinction, semantics deferred**

These forms are structurally different:

```phalcom
method()
```

```phalcom
method() {}
```

The first has no body.

The second has a concrete empty body.

The AST and reflection model must preserve this distinction.

### 27.1 Empty body

A concrete empty method completes normally and returns unit:

```phalcom
()
```

Example:

```phalcom
touch() {}
```

### 27.2 Bodyless method

Potential meanings include:

- protocol requirement;
- abstract-class member;
- unimplemented ordinary-class member;
- native member;
- generated/default-adoption declaration.

General semantics are deferred.

### 27.3 Explicit unimplemented form

A possible explicit spelling was explored:

```phalcom
@unimplemented
method() {}
```

possibly equivalent to a bodyless declaration in an ordinary class.

This is not ratified.

### 27.4 Return type when omitted

A bodyless declaration without a return annotation has effective return type:

```phalcom
Dynamic
```

It does not imply `Never`.

### 27.5 Open questions

- Are bodyless methods legal in ordinary concrete classes?
- Do they fail at class construction, method invocation, or verification?
- Is an abstract marker required?
- How do native methods use bodyless syntax?
- How are implementation kinds reflected?
- Can a bodyless declaration reserve a selector for subclasses?
- Does `@unimplemented` exist?
- How do protocol defaults interact with bodyless declarations?

---

# Part IX — Sealed variants, unions, and alternatives

## 28. Nominal variants and untagged unions

**Status: Ratified distinction**

A sealed variant hierarchy such as:

```phalcom
@sealed
@data
class Result<T, E> {
  @variant Ok(value: T)
  @variant Err(error: E)
}
```

is nominal and tagged.

A union:

```phalcom
Ok<T> | Err<E>
```

is anonymous and untagged.

They are related but not identical.

### 28.1 Variant substitution

A conceptual generic model:

```text
Ok<T> extends Result<T, Never>
Err<E> extends Result<Never, E>
```

uses `Never` for the impossible branch.

### 28.2 Reflective bridge

Possible APIs:

```phalcom
Result<Int, Error>.variants
```

```phalcom
Result<Int, Error>.variantUnion
```

The latter could yield:

```phalcom
Ok<Int> | Err<Error>
```

A common capability may expose alternatives:

```phalcom
@protocol
class AlternativeType {
  alternatives -> const List<Type>
}
```

No exact API is ratified.

### 28.3 Open questions

- Is a sealed root type-equivalent to the union of its variants?
- Does exhaustiveness operate on the nominal root, the variant union, or both?
- Are variant types ordinary classes?
- Can variants carry independent inheritance?
- How are nested sealed families represented?
- Does alias expansion preserve nominal exhaustiveness information?

---

# Part X — Records

## 29. Record role

**Status: Foundational concept accepted; ordering semantics open**

Records are intended to be immutable heterogeneous product values.

Potential uses:

- tuples;
- labeled product values;
- callable parameter domains;
- structured returns;
- destructuring;
- pattern matching;
- lightweight immutable data;
- argument capture and forwarding;
- serialization;
- reflective invocation.

Record syntax under discussion:

```phalcom
()
(value,)
(first, second)
(name: value)
(first, name: value)
```

### 29.1 Positional and labeled groups

A strong common rule across all models is:

> Positional fields appear before labeled fields.

Example:

```phalcom
(first, second, name: value, enabled: true)
```

A positional field after a labeled field would be rejected:

```phalcom
(first, name: value, second)
```

This rule remains highly favored but should be formally ratified with the final record model.

---

## 30. Ordered versus unordered labeled fields

**Status: Open and central**

### 30.1 Model A — unordered labeled fields

The record consists of:

```text
ordered positional sequence
+
unordered label-to-value mapping
```

Then:

```phalcom
(name: "Altun", age: 30)
```

and:

```phalcom
(age: 30, name: "Altun")
```

have the same labeled shape and may be equal.

#### Strengths

- labels act as genuine field identities;
- reordering does not change structural meaning;
- structural record typing is natural;
- labeled destructuring is order-independent;
- refactoring field presentation does not change type identity;
- closely resembles Dart-style records;
- maps naturally to structural products.

#### Costs

- declaration order is no longer semantic;
- canonical serialization and printing need a policy;
- source order and semantic order diverge;
- exact call forwarding may need a separate argument object;
- method selector ordering may not align with record shape;
- two equal records may have different source presentations.

### 30.2 Model B — ordered labeled fields

The record is one ordered sequence of fields, some positional and some labeled.

Then:

```phalcom
(name: "Altun", age: 30)
```

and:

```phalcom
(age: 30, name: "Altun")
```

have different ordered shapes.

#### Strengths

- one canonical declared sequence;
- reflection and printing are direct;
- serialization order is preserved;
- destructuring can mirror declaration order;
- records can model ordered method argument packets;
- callable signatures and records can share one shape model;
- exact forwarding loses no order information;
- implementation may use a simple immutable field sequence.

#### Costs

- labels are not sufficient to establish field identity;
- harmless field reordering becomes a type change;
- two values with the same named projections may compare differently;
- structural width and compatibility become more sequence-sensitive;
- named data behaves less like a map or conventional record;
- refactoring declaration order may break APIs.

### 30.3 Model C — separate records and argument packets

Under this model:

```text
Record:
    ordered positional fields
    + unordered labeled fields

Arguments:
    ordered positional fields
    + ordered labeled slots
```

#### Strengths

- each abstraction gets semantics suited to its purpose;
- records remain structural data;
- calls remain ordered messages;
- exact forwarding is preserved;
- selector identity remains direct.

#### Costs

- conceptual duplication;
- function-domain syntax must choose `RecordType` or `ArgumentsType`;
- conversions need defined information-loss rules;
- the attractive “calls are records” unification is weakened.

### 30.4 Current inclination

The user’s latest inclination was toward ordered labeled records because a declared order may be meaningful and there may be little value in permitting arbitrary reordering.

This is not ratified.

The next design discussion should begin from use cases for both models and remain neutral.

---

## 31. Record equality, hashing, and type identity

**Status: Open**

The chosen ordering model must consistently define:

- value equality;
- hashing;
- type equality;
- assignment compatibility;
- pattern compatibility;
- serialization identity.

### 31.1 Under unordered labels

Equality compares:

1. positional fields by ordered position;
2. label sets by identity;
3. labeled values by label.

Hashing must be:

- order-sensitive for positionals;
- order-insensitive but label-sensitive for labeled fields.

### 31.2 Under ordered labels

Equality compares the complete field sequence, including labels and order.

Hashing follows the same sequence.

### 31.3 No mixed rule without justification

The language should avoid a model where:

- record value equality ignores label order;
- record type identity preserves it;

unless a strong semantic reason is established.

Such a split would surprise users and complicate reflection.

---

## 32. Record reflection

**Status: Open**

Possible APIs:

```phalcom
record.fields
record.positionalValues
record.labeledValues
record.labels
record.at(index)
record.valueFor(label)
```

Under unordered semantics, the authoritative model should likely separate:

```text
positionals: ordered sequence
labels: mapping
```

Under ordered semantics, one field sequence may be authoritative.

Even under unordered semantics, source declaration order may be retained as metadata for:

- diagnostics;
- formatting;
- documentation.

Source order must not accidentally determine equality if the semantic model is unordered.

---

## 33. Record printing and serialization

**Status: Open**

Possible printing policies:

1. Preserve original source order.
2. Preserve runtime construction order.
3. Canonically sort labels.
4. Use declared type order.
5. Allow formatter-specific presentation.

Serialization introduces a stronger question:

> Is labeled order part of the data contract or merely presentation?

Ordered records naturally preserve serialization order.

Unordered records require either:

- canonical label sorting;
- serializer-defined ordering;
- explicit schema ordering;
- source-order metadata.

---

## 34. Record field access and destructuring

**Status: Open**

Labeled access likely uses ordinary member syntax:

```phalcom
record.name
```

Positional access remains undecided:

```phalcom
record.at(0)
record.0
record.$1
```

Possible destructuring:

```phalcom
const (first, second) = pair
```

```phalcom
const (name: name, age: age) = person
```

Under unordered labeled semantics, labeled pattern order should not matter.

Under ordered labeled semantics, the language must decide whether destructuring:

- follows exact field order;
- matches labels independently;
- supports both modes.

This distinction may expose whether labels are true identities or annotations on slots.

---

## 35. Record width subtyping

**Status: Deferred/Open**

Example:

```phalcom
Small = (name: String)
Large = (name: String, age: Int)
```

Possible width relation:

```text
Large <: Small
```

because every `Large` value contains the required `name`.

Exact-shape typing is simpler:

```text
Large is not a subtype of Small
```

Width subtyping affects:

- assignment;
- pattern matching;
- function variance;
- overload resolution;
- reflection;
- equality expectations;
- record conversion.

Exact shape is the safer initial model, but no final decision exists.

---

# Part XI — Methods, labels, and selectors

## 36. Current selector model

**Status: Existing design**

Phalcom currently models method selectors as ordered slot sequences.

Example call:

```phalcom
method(a, b, c: d, e: f)
```

corresponds conceptually to:

```phalcom
#method(_, _, c:, e:)
```

Under the current semantics:

```phalcom
#method(_, _, c:, e:)
```

and:

```phalcom
#method(_, _, e:, c:)
```

are distinct selectors.

Labeled arguments are ordered labeled slots, not unordered keyword bindings.

### 36.1 Existing advantages

- simple method-table keys;
- direct argument-to-parameter slot alignment;
- no runtime permutation;
- exact selector pinning;
- overloads may differ by ordered label sequence;
- labels can form an ordered message phrase.

### 36.2 Existing cost

The model does not align literally with unordered structural records.

---

## 37. External labels and internal parameter names

**Status: Open but strongly relevant**

A future syntax may distinguish public call labels from local names:

```phalcom
move(from source: Point, to destination: Point) {
  ...
}
```

Call:

```phalcom
move(from: origin, to: target)
```

Conceptually:

```text
external label: from
internal name: source

external label: to
internal name: destination
```

### 37.1 Intended principles

- External labels contribute to the public selector.
- Internal names are local implementation bindings.
- Internal names do not affect selector identity.
- Reflection should expose both.
- Renaming an internal parameter should not break callers.
- Changing an external label is an API change.

### 37.2 Open syntax questions

- How is an omitted external label represented?
- Can a positional parameter have an internal name only?
- Can the same spelling serve as both external and internal name?
- What is the syntax for an intentionally unlabeled external parameter?
- Can two parameters share an external label?
- Are labels required to be unique within a selector?

---

## 38. Ordered method labels

**Status: Existing behavior and serious candidate**

Under ordered semantics:

```phalcom
request(path: p, timeout: t, retries: r)
```

must be called in that order.

Reordering labels produces a different selector or an invalid call:

```phalcom
request(retries: r, path: p, timeout: t)
```

### 38.1 Design principle

Labels are parts of an ordered API phrase, not keys in a dictionary.

This resembles Swift’s philosophy:

- declarations establish canonical phrasing;
- labels improve readability at the call site;
- parameter order is part of the API;
- external and internal names serve different purposes.

### 38.2 Strengths

- one canonical call spelling;
- clear documentation and autocomplete;
- simple dispatch;
- message syntax reads as a phrase;
- exact selector identity;
- natural rest-prefix matching;
- no hidden argument permutation.

### 38.3 Costs

- callers cannot reorder labels for convenience;
- method arguments are not ordinary unordered records;
- callable domains may need an ordered argument type;
- labels act partly as slot annotations rather than pure keys.

---

## 39. Unordered method labels

**Status: Explored, then reopened**

Under unordered semantics, selector identity would be:

```text
base name
+ positional arity
+ unordered labeled-name set
```

These would select the same method:

```phalcom
request(path: p, timeout: t, retries: r)
request(retries: r, path: p, timeout: t)
```

### 39.1 Strengths

- labels are true binding names;
- calls align with unordered records;
- callable domains may be ordinary record types;
- reordering is semantically harmless;
- declarations differing only by labeled order become duplicates.

### 39.2 Costs

- multiple valid spellings of one call;
- declaration order is advisory rather than enforced;
- call binding requires label-to-slot mapping;
- source evaluation order differs from callee binding order;
- exact ordered message-selector identity is lost.

### 39.3 Current status

The user questioned why arbitrary reordering should be allowed when declarations already establish a canonical documented order.

Therefore unordered method labels are no longer accepted as the current direction.

---

## 40. Evaluation order

**Status: Strongly favored; should be ratified**

Regardless of label-binding semantics, argument expressions should evaluate left to right in source order.

Example:

```phalcom
method(first: produceFirst(), second: produceSecond())
```

Evaluation order:

```text
1. produceFirst()
2. produceSecond()
3. invoke method
```

If labels become unordered for binding, source evaluation order still remains ordered.

This distinction must be explicit:

```text
source order:
    controls side effects and evaluation

declaration order:
    controls presentation and possibly frame layout

label identity:
    controls binding under unordered semantics

selector slot order:
    controls dispatch under ordered semantics
```

---

# Part XII — Callable types and `->`

## 41. Function-type syntax

**Status: Preferred syntax, semantic left-hand model open**

Preferred surface:

```phalcom
(Int, String, c: Bool) -> Bool
```

Other examples:

```phalcom
() -> String
(Int,) -> String
(Int, String) -> Bool
(Request, timeout: Duration) -> Response
```

Singleton positional syntax requires a trailing comma:

```phalcom
(Int,)
```

because:

```phalcom
(Int)
```

is a grouped expression.

### 41.1 Zero arguments and unit

```phalcom
() -> Result
```

uses `()` as the empty parameter domain.

```phalcom
(Input,) -> ()
```

describes a one-argument callable returning unit.

---

## 42. `->` as a real operator method

**Status: Provisional**

The arrow is being explored as an ordinary operator dispatch:

```phalcom
left.->(right)
```

with selector:

```phalcom
#->(_)
```

This would make function-type construction one use of a general operator.

Potential explicit calls:

```phalcom
record.->(Result)
```

Potential other arities:

```phalcom
object.->(a, b, option: c)
```

Potential selector pinning:

```phalcom
object::#->(_)
```

### 42.1 Declaration context

In method declarations:

```phalcom
method() -> Result
```

the arrow marks a return annotation.

In expression context:

```phalcom
A -> B
```

it is an operator.

The parser must distinguish these contexts.

### 42.2 Associativity

Right associativity was recommended:

```phalcom
A -> B -> C
```

means:

```phalcom
A -> (B -> C)
```

This is not ratified.

### 42.3 Open questions

- Precedence.
- Associativity.
- Whether arbitrary objects may implement `->`.
- Whether only type-domain objects may produce `FunctionType`.
- Whether operator tokens are legal after dot syntax.
- Exact selector-literal grammar.
- Interaction with blocks, which use `=>`.
- Error behavior when the right operand is not a type.

---

## 43. Callable parameter-domain representation

**Status: Open due to record-ordering reconsideration**

Three models remain.

### 43.1 RecordType domain

```phalcom
(Int, String, c: Bool) -> Bool
```

uses an ordinary record type as the input domain.

This works best when method arguments and record fields share semantics.

### 43.2 ArgumentsType domain

The left side describes an ordered call packet:

```text
ordered positional parameter types
+
ordered labeled parameter slots
```

This is compatible with ordered selectors even if ordinary records are unordered.

### 43.3 Contextual domain expression

The same syntax:

```phalcom
(Int, String, c: Bool)
```

could mean:

- `Record` in value context;
- `RecordType` in annotation context;
- `ArgumentsType` to the left of `->`.

This is concise but context-sensitive.

### 43.4 Required future decision

The language must decide whether:

```phalcom
(Int, String, c: Bool)
```

is literally one semantic object across records and callables, or a shared notation for related but distinct domain objects.

---

## 44. FunctionType versus MethodSignature

**Status: Ratified distinction in principle**

A callable type does not necessarily include a method base selector.

```phalcom
(Int, String, c: Bool) -> Bool
```

describes callable compatibility.

A method signature also includes:

- base selector name;
- instance-side or class-side status;
- parameter metadata;
- generic parameters;
- implementation attributes;
- source location.

Conceptual model:

```phalcom
@data
@immutable
class FunctionType is Type {
  const parameters
  const result
}
```

```phalcom
@data
@immutable
class MethodSignature {
  const selector
  const parameters
  const result
  const typeParameters
  const isClassSide
}
```

Exact fields depend on the final ordered/unordered decision.

---

# Part XIII — Rest arguments and forwarding

## 45. Unified rest capture

**Status: Promising proposal, not ratified**

A unified rest parameter would capture both remaining positional and labeled arguments:

```phalcom
forward(prefix, *rest) {
  return target(*rest)
}
```

Example call:

```phalcom
forward(
  "request",
  body,
  encoding: #utf8,
  timeout: Duration.seconds(10)
)
```

Potential binding:

```text
prefix = "request"

rest contains:
    positional: body
    labeled:
        encoding: #utf8
        timeout: Duration.seconds(10)
```

This may replace separate Python-style `*args` and `**kwargs`.

### 45.1 Why this is attractive

It supports:

- proxies;
- forwarding decorators;
- dispatch interception;
- `doesNotUnderstand`;
- middleware;
- reflection;
- RPC adapters;
- default-method forwarding;
- logging and tracing.

---

## 46. Rest capture type

**Status: Open**

### 46.1 Ordinary record

If records preserve all argument-order information, `*rest` may capture a record directly.

### 46.2 Separate `Arguments`

If records use unordered labels but calls preserve ordered selector slots, a separate immutable type is more honest:

```phalcom
@data
@immutable
class Arguments {
  const positionals: Tuple
  const labeled: const List<LabeledArgument>
}
```

with:

```phalcom
@data
@immutable
class LabeledArgument {
  const label: Symbol
  const value: Any
}
```

Then:

```text
Arguments:
    ordered call packet

Record:
    structural data value
```

### 46.3 Information-losing conversion

An ordered `Arguments` packet may convert to an unordered record by forgetting label order.

The reverse conversion requires an ordering policy or target declaration.

No conversion API is ratified.

---

## 47. Rest placement

**Status: Provisional recommendation**

A combined rest parameter should probably be terminal:

```phalcom
method(first, *rest)
```

Fixed parameters after a combined rest capture are difficult to define because the rest consumes all remaining positional and labeled arguments.

However, alternative semantics could allow named extraction around a rest packet. This requires explicit investigation before finalization.

---

## 48. Call expansion

**Status: Open**

Potential syntax:

```phalcom
target(*arguments)
```

Potential mixed call:

```phalcom
target(prefix, mode: #safe, *rest)
```

Potential rules:

1. Evaluate explicit arguments left to right.
2. Evaluate the expansion object.
3. Expand already evaluated values.
4. Reject duplicate labels.
5. Resolve the resulting selector.
6. Invoke without reevaluating original expressions.

Multiple expansions:

```phalcom
target(*first, *second)
```

were suggested for deferral.

### 48.1 Open questions

- Must expansion be final?
- May explicit arguments follow expansion?
- Can multiple expansions appear?
- How are duplicate labels handled?
- Does last-one-wins ever apply?
- Can ordinary records be expanded?
- Is expansion restricted to `Arguments`?
- Does expansion preserve label order?
- What selector diagnostics are produced?

---

## 49. Rest selectors and overload resolution

**Status: Open**

A method:

```phalcom
process(first, *rest)
```

accepts a selector family rather than one fixed selector.

Possible selector notation:

```phalcom
#process(_, *)
```

Potential preference:

1. exact selector;
2. matching rest selector with the longest fixed prefix;
3. ambiguity error.

Example:

```phalcom
process(value) { ... }
process(first, *rest) { ... }
```

A one-argument call prefers the exact method.

A larger call uses the rest method.

This model aligns naturally with ordered selector prefixes. Under unordered label binding, specificity is more complex.

---

# Part XIV — Reflection model

## 50. Core reflective objects

**Status: Provisional inventory**

Likely objects include:

```phalcom
Type
Class
ProtocolClass
TypeParameter
GenericSignature
TypeSubstitution
Method
MethodSignature
Parameter
Selector
TypeAlias
AppliedType
UnionType
IntersectionType
FunctionType
RecordType
ArgumentsType
Arguments
```

Not every listed object is ratified.

### 50.1 Method reflection

Potential surface:

```phalcom
method.selector
method.owner
method.parameters
method.returnType
method.declaredReturnType
method.typeParameters
method.attributes
method.implementationKind
method.sourceLocation
```

### 50.2 Parameter reflection

Potential surface:

```phalcom
parameter.name
parameter.label
parameter.position
parameter.type
parameter.declaredType
parameter.kind
parameter.defaultValue
parameter.attributes
```

If external and internal names are adopted:

```text
parameter.label:
    external call-site label

parameter.name:
    local implementation name
```

### 50.3 Selector reflection

The final selector shape depends on label ordering:

#### Ordered model

```text
base name
+ ordered slot sequence
```

#### Unordered labeled model

```text
base name
+ positional arity
+ label set
```

Reflection must not hide this language-level choice.

---

# Part XV — Remaining design decisions

## 51. Highest-priority open questions

### 51.1 Ordered versus unordered labeled records

This is the next major discussion.

The analysis should start from concrete use cases rather than prior aesthetic preference.

Required examples:

- configuration records;
- structured return values;
- database rows;
- HTTP options;
- geometry and coordinates;
- serialization;
- pattern matching;
- record equality;
- record refactoring;
- method argument packets;
- forwarding;
- callable types.

### 51.2 Relationship between calls and records

Choose among:

1. Calls are literally record-shaped.
2. Calls and records share syntax but not semantics.
3. Calls use `Arguments`; records remain separate.
4. Records become ordered specifically to support call unification.

### 51.3 External and internal labels

Specify:

- syntax;
- reflection;
- uniqueness;
- selector participation;
- interaction with positionals;
- compatibility rules.

### 51.4 Rest capture

Decide:

- one combined rest or separate positional/labeled rest;
- capture object;
- placement;
- expansion;
- duplicate labels;
- overload specificity.

### 51.5 Callable domain type

Decide whether:

```phalcom
(Int, c: Bool)
```

is:

- a `RecordType`;
- an `ArgumentsType`;
- a contextual domain expression;
- one universal product type.

---

## 52. Protocol open questions

1. Exact protocol metaclass hierarchy.
2. Whether bodyful protocol methods remain conformance requirements.
3. Explicit default adoption syntax.
4. Default conflict resolution.
5. Class-side requirements.
6. Protocol inheritance.
7. Protocol-associated types, if any.
8. Conformance evidence objects.
9. `Dynamic` in structural conformance.
10. Exact runtime conformance API.

---

## 53. Type alias open questions

1. Recursive aliases in v1.
2. Predeclaration phases.
3. Productive-recursion rules.
4. Module-only restriction.
5. Class-scoped aliases.
6. Local aliases.
7. Alias wrapper versus side metadata.
8. Equality and identity.
9. Printing and diagnostics.
10. Generic alias specialization and caching.

---

## 54. Type algebra open questions

1. Canonical normalization of unions and intersections.
2. Interner and equality rules.
3. `Any`, `Dynamic`, and `Never` interactions.
4. Type negation.
5. Optional postfix `?`.
6. User override rules for type operators.
7. Distribution and absorption laws.
8. Recursive-type normalization limits.

---

## 55. Runtime type-test open questions

1. Final method names.
2. `is!` spelling and semantics.
3. Boolean behavior for erased applied generics.
4. Warning or error policy.
5. Exact meaning of `Unknown`.
6. Exact meaning of `Erased`.
7. Protocol checks with missing annotations.
8. Caching and invalidation.
9. Test behavior for aliases.
10. Test behavior for union and intersection types.

---

## 56. Bodyless-method open questions

1. Ordinary-class legality.
2. Abstract classes.
3. Native declarations.
4. Runtime failure timing.
5. `@unimplemented`.
6. Reflection states.
7. Default adoption.
8. Effective return types.
9. Constructor interactions.
10. Tooling diagnostics.

---

## 57. Record open questions

1. Ordered or unordered labels.
2. Equality and hashing.
3. Type identity.
4. Width subtyping.
5. Destructuring.
6. Positional access syntax.
7. Iteration order.
8. Printing.
9. Serialization.
10. Mutation policy.
11. Conversion to data classes.
12. Optional/default fields.
13. Rest fields.
14. Generic records.
15. ABI layout.
16. Source-order metadata.
17. Pattern exhaustiveness.

---

# Part XVI — Conformance examples for ratified decisions

## 58. Unit behavior

Valid:

```phalcom
log(message: String) -> () {
  System.print(message)
}
```

Valid transparent alias:

```phalcom
@type
const Unit = ()

log(message: String) -> Unit {
  System.print(message)
}
```

Runtime fallthrough:

```phalcom
touch() {
}
```

returns:

```phalcom
()
```

Incorrect conceptual substitution:

```phalcom
findUser(id: Int) -> ()
```

This means the operation returns no meaningful information, not that a user may be absent.

Absence:

```phalcom
findUser(id: Int) -> User | NoneType
```

Non-return:

```phalcom
abort(message: String) -> Never {
  throw Error.new(message)
}
```

---

## 59. Union and intersection

```phalcom
@type
const Identifier = Int | String
```

```phalcom
@type
const SerializableEntity = Entity & Serializable
```

Operator reflection:

```phalcom
Identifier::#|(_)
SerializableEntity::#&(_)
```

No required duplicate:

```phalcom
Identifier.union(String)
```

---

## 60. Generic application and binding

Application:

```phalcom
Map<String, User>
```

Conceptually:

```phalcom
Map.<>(String, User)
```

Binding:

```phalcom
class Map<K, V> {
}
```

does not call `Map.<>(K, V)`.

The compiler creates lexical type-parameter bindings.

---

## 61. Transparent aliases

```phalcom
@type
const UserId = Int
```

`UserId` is not a nominally distinct integer type.

A nominal wrapper or newtype would require a different language feature.

---

## 62. Protocol generic substitution

```phalcom
@protocol
class Parser<I, out O> {
  parse(input: I) -> O
}
```

Application:

```phalcom
Parser<String, Json>
```

Requirement after substitution:

```phalcom
parse(input: String) -> Json
```

A candidate method is checked structurally against that effective requirement.

---

# Part XVII — Non-goals and cautions

## 63. Do not infer implementation inheritance from protocols

Structural conformance does not add protocol bodies or state to the implementation inheritance chain.

Defaults, if supported, require explicit semantics.

---

## 64. Do not treat annotation metadata as automatic dispatch

Ordinary method dispatch continues to use selectors and the object model.

Type annotations may support explicit multimethod libraries or future controlled facilities, but do not silently alter dispatch.

---

## 65. Do not conflate `None`, unit, and `Never`

```text
None:
    absence

():
    successful no-information result

Never:
    no normal result
```

These distinctions are foundational.

---

## 66. Do not claim record/call unification before ordering is decided

The notation:

```phalcom
(Int, String, c: Bool)
```

may eventually unify:

- record types;
- callable domains;
- method arguments.

That unification is not yet settled because labeled-order semantics remain open.

---

## 67. Do not silently normalize away user-visible identity

Type normalization must preserve enough metadata for:

- useful diagnostics;
- alias names;
- source locations;
- reflection;
- exhaustive matching.

Semantic equality and user-facing presentation may require separate representations.

---

# Part XVIII — Recommended next design session

## 68. Starting task

Begin with a neutral comparison of ordered and unordered labeled records.

Do not begin by assuming that:

- structural records must ignore order;
- canonical declarations must preserve order;
- method arguments must be records;
- existing selector implementation must control record semantics.

Instead, evaluate concrete cases.

### 68.1 Ordered-record stress tests

Investigate:

- database row column order;
- CSV and binary serialization;
- protocol wire formats;
- positional-plus-labeled destructuring;
- exact argument forwarding;
- stable reflection;
- code generation;
- ABI layout;
- ordered named coordinates or dimensions.

### 68.2 Unordered-record stress tests

Investigate:

- configuration values;
- structural data interchange;
- API response records;
- field projection;
- refactoring field declaration order;
- width subtyping;
- schema compatibility;
- pattern matching by label;
- hash-based equality.

### 68.3 Separate-abstraction stress tests

Investigate whether:

```text
Record
Arguments
RecordType
ArgumentsType
FunctionType
MethodSignature
Selector
```

are justified separate objects or unnecessary duplication.

### 68.4 Required outcome

The next session should produce:

1. a use-case matrix;
2. equality and hashing consequences;
3. type-identity consequences;
4. selector consequences;
5. forwarding consequences;
6. reflection consequences;
7. a recommendation with rejected alternatives;
8. explicit migration consequences for current Phalcom selectors.

---

# Appendix A — Compact status ledger

| Topic | Current status |
|---|---|
| `Never` name | Ratified |
| Unit is `()` | Ratified |
| `Unit` transparent alias | Ratified as permitted |
| `None` distinct from unit | Ratified |
| Missing return annotation → `Dynamic` | Ratified |
| Union `A \| B` | Ratified |
| Intersection `A & B` | Ratified |
| Named duplicate union/intersection methods | Not required |
| `@type const` aliases | Ratified |
| Runtime alias object `TypeAlias` | Ratified |
| Transparent aliases | Ratified |
| Recursive aliases | Open/deferred |
| Module-only aliases | Proposed, not ratified |
| Generic application through `<>(*)` | Ratified direction |
| Generic binders are lexical | Ratified |
| Upper bounds `<T: Bound>` | Ratified |
| Closed constraints `<T in ...>` | Deferred |
| Applied generics create runtime class | Rejected by default |
| Protocols structural | Ratified |
| Protocols specialized class-like objects | Ratified direction |
| Protocol defaults | Provisional |
| Bodyless methods | Semantics deferred |
| `A -> B` operator method | Provisional |
| Callable domain is record | Reopened |
| Records immutable | Strongly intended |
| Positional fields before labeled fields | Strongly favored, final ratification pending |
| Labeled record order | Open |
| Labeled method order | Existing ordered model; reconsideration open |
| External/internal labels | Open |
| Unified `*rest` | Promising proposal |
| Rest captured as record | Open |
| Separate `Arguments` type | Open |
| `T?` optional shorthand | Open |
| Type negation | Deferred |
| Type-test API names | Open |
| Sealed root equals variant union | Open |

---

# Appendix B — Core semantic distinctions

```text
Type value:
    Any runtime object participating in type semantics.

Class:
    A type that is also a runtime class object.

AppliedType:
    A generic type constructor applied to type arguments.

TypeAlias:
    A stable transparent declaration that names another type expression.

Protocol:
    A structural capability type.

Record:
    An immutable heterogeneous product value.

RecordType:
    A type describing record values.

Arguments:
    A possible immutable exact call packet.

ArgumentsType:
    A possible type describing ordered call packets.

FunctionType:
    A callable parameter domain plus result type.

MethodSignature:
    A function contract plus selector and declaration metadata.

():
    The empty record, unit value, and unit type.

None:
    The absence value.

Never:
    The uninhabited bottom type.

Dynamic:
    The absence of a useful static constraint in gradual/reflection contexts.
```

---

# Appendix C — Central unresolved tension

Phalcom currently faces a foundational choice.

## Unified structural model

```text
record shape
=
call argument shape
=
callable parameter domain
```

This is most natural when labeled fields are unordered names.

## Unified ordered-message model

```text
record field sequence
=
selector slot sequence
=
callable parameter sequence
```

This is most natural when labeled fields remain ordered.

## Specialized model

```text
records:
    structural data

arguments:
    ordered messages
```

This preserves domain-specific semantics but introduces more concepts.

The next design session should decide based on concrete use cases and language-wide coherence, not implementation convenience alone.
