# Phalcom Pattern Matching Specification

**Status:** Draft language specification
**Version:** 0.1
**Date:** 2026-07-26
**Audience:** Language designers, compiler and interpreter implementers, standard-library authors, tooling authors, and conformance-test maintainers

---

## 1. Purpose

This document specifies native pattern matching for Phalcom, including:

- exhaustive matching over statically closed pattern domains;
- unreachable and impossible-pattern detection;
- sealed families and variants;
- constructor, literal, binding, pin, wildcard, OR, alias, rest, tuple, record, and type-test patterns;
- recursive coverage and inhabitation analysis;
- generic specialization and uninhabited variants;
- match-arm type refinement;
- runtime behavior in the presence of dynamic values;
- reflection and object-model obligations;
- diagnostics and conformance requirements.

It also records unresolved design questions separately from ratified semantics. Recommendations in the open-questions section are non-normative until ratified.

---

## 2. Design goals

Native matching must satisfy all of the following:

1. **Exhaustiveness is a compile-time guarantee.** Every match over a statically closed domain covers every inhabited case.
2. **Every arm is useful.** A wholly unreachable or impossible arm is a compile-time error.
3. **Closed families remain compatible with Phalcom's object model.** Variants are ordinary runtime classes and their instances are ordinary objects.
4. **Patterns are locally readable.** A bare name always binds; matching an existing value is explicit.
5. **Pattern semantics do not depend on overridable dispatch.** Variant identity and component extraction are trusted operations.
6. **Static proof does not produce runtime unsafety.** Every match retains a hidden defensive `MatchError` path.
7. **Type annotations remain reflective rather than implicitly enforced.** Matching does not silently convert applied generic annotations into runtime validation.
8. **The feature generalizes beyond sealed variants.** Exhaustiveness applies to every compiler-provable closed decomposition.

---

## 3. Normative terminology

### 3.1 Scrutinee

The expression whose value is matched:

```phalcom
match result {
  ...
}
```

`result` is the scrutinee.

### 3.2 Pattern domain

The set of runtime values described by a static type, represented symbolically through constructors, literals, products, unions, and inhabitation facts.

A domain need not have finitely many values. It needs a statically closed decomposition. `Result<Int, Error>` may contain infinitely many values, but its top-level constructor space is closed:

```text
Ok(Int) | Err(Error)
```

### 3.3 Pattern region

The subset of the scrutinee domain accepted by a pattern.

### 3.4 Closed decomposition

A decomposition whose complete set of alternatives is known to the compiler. Examples include:

- a declaration-owned `@sealed` family;
- `Bool`;
- a closed union;
- a tuple or record whose component domains are known;
- a constrained generic with a closed alternative set;
- a literal union;
- a recursive sealed family whose constructors are known.

### 3.5 Inhabited type

A type for which at least one finite runtime value can be constructed.

`Nothing` is uninhabited. A recursive sealed family with no finite base constructor may also be uninhabited.

### 3.6 Irrefutable pattern

A pattern accepting every value in its current static domain. `_` and a bare binding are irrefutable. A nominal type pattern may also be irrefutable relative to a narrower static domain.

---

## 4. Match syntax

The recommended surface grammar is:

```text
MatchExpression
  ::= "match" Expression "{" MatchArm* "}"

MatchArm
  ::= Pattern Guard? "=>" ArmBody

Guard
  ::= "if" Expression
```

Example:

```phalcom
match result {
  Ok(value: value) => use(value)
  Err(err: error) => recover(error)
}
```

A `match` is intended to be an expression. When used in statement position, its result is discarded. Exhaustiveness does not depend on whether the result is consumed.

The exact arm-body grammar, separators, and expression-result typing remain open in Section 38.

---

## 5. Strict exhaustiveness

### 5.1 Core rule

When the compiler can determine a closed static pattern domain, every `match` must cover the complete inhabited domain. A missing case is a compile-time error.

This rule applies equally when the match:

- produces a value;
- is used only for effects;
- appears in statement position;
- appears in a context where the programmer believes a missing case is impossible.

```phalcom
match result {
  Ok(value: value) => System.print(value)
}
```

```text
error: non-exhaustive match

missing pattern:
  Err(err: _)
```

Changing the match to statement position does not weaken this rule.

### 5.2 Open or unknown domains

When the compiler cannot enumerate the complete domain, the match requires an irrefutable fallback unless an earlier pattern is already irrefutable over the static domain.

```phalcom
match value {
  is Int as number => useInteger(number)
  is String as text => useText(text)
  _ => unsupported(value)
}
```

A scrutinee typed as `Dynamic`, `Any`, `Object`, an open class hierarchy, an unconstrained type parameter, or an open protocol normally requires such a fallback.

### 5.3 Symbolic domain model

Conceptually:

```text
D(Bool)
  = true | false

D(SealedRoot<A...>)
  = Variant1(fields...) | Variant2(fields...) | ...

D(A | B)
  = D(A) ∪ D(B)

D((A, B))
  = D(A) × D(B)

D(T in (A, B, C))
  = D(A) ∪ D(B) ∪ D(C)

D(Nothing)
  = ∅
```

The compiler should reason symbolically rather than eagerly enumerating large products.

---

## 6. Usefulness, redundancy, and impossible patterns

### 6.1 Unreachable arms

Every arm must accept at least one value not already accepted by preceding unguarded arms.

```phalcom
match result {
  Ok(value: value) => use(value)
  Ok(value: 0) => specialCase()
  Err(err: error) => recover(error)
}
```

```text
error: unreachable match arm

pattern:
  Ok(value: 0)

is already covered by:
  Ok(value: value)
```

### 6.2 Duplicate patterns

Exact duplicates are errors:

```phalcom
match flag {
  true => first()
  true => second()
  false => third()
}
```

### 6.3 Impossible patterns

A pattern with no intersection with the scrutinee's static domain is an error:

```phalcom
const result: Result<Int, Error> = ...

match result {
  Ok(value: value) => ...
  None => ...
  Err(err: error) => ...
}
```

```text
error: impossible pattern

None cannot match Result<Int, Error>
```

### 6.4 Partial overlap

Overlap is permitted when the later pattern still contributes uncovered values:

```phalcom
match pair {
  (true, _) => ...
  (_, true) => ...
  (false, false) => ...
}
```

The second arm remains useful because it accepts `(false, true)`.

---

## 7. Guards

### 7.1 Coverage

Guards do not contribute to static exhaustiveness. A guarded arm removes no region from the statically uncovered domain because the guard may evaluate to `false`.

```phalcom
match value {
  is Int as number if number > 0 => positive(number)
  is Int as number if number <= 0 => nonPositive(number)
}
```

```text
error: non-exhaustive match

missing pattern:
  is Int
```

The correct form includes an unguarded structural fallback:

```phalcom
match value {
  is Int as number if number > 0 => positive(number)
  is Int as number => nonPositive(number)
}
```

### 7.2 Runtime order

For each arm:

1. test the structural pattern;
2. establish bindings;
3. evaluate the guard;
4. execute the arm body if the guard is true;
5. otherwise continue with the next arm.

### 7.3 Usefulness

A guarded arm does not make a later structurally identical unguarded arm unreachable:

```phalcom
match option {
  Some(value: number) if number > 0 => positive(number)
  Some(value: number) => other(number)
  None => absent()
}
```

---

## 8. Sealed families

### 8.1 `@sealed`

A `@sealed` declaration defines a declaration-owned closed algebraic family.

```phalcom
@sealed @data
class Result<out T, out E> {
  @variant Ok(value: T)
  @variant Err(err: E)
}
```

A sealed root:

- is abstract;
- cannot be instantiated directly;
- is inhabited only through variants declared in its own body;
- owns the complete immutable variant set;
- cannot be externally subclassed;
- provides compiler-authoritative coverage metadata;
- may contain shared methods that execute on variant instances.

Invalid:

```phalcom
Result.new(...)
```

Invalid:

```phalcom
class Pending<T, E> is Result<T, E> {}
```

Adding a variant requires editing the sealed root declaration.

### 8.2 Empty sealed families

A sealed root with no inhabited variants is uninhabited:

```phalcom
@sealed @data
class Never {}
```

```text
Never = ∅
```

### 8.3 Shared behavior

A sealed root may declare common methods:

```phalcom
@sealed @data
class Result<out T, out E> {
  isOk -> Bool {
    return match self {
      Ok(value: _) => true
      Err(err: _) => false
    }
  }

  @variant Ok(value: T)
  @variant Err(err: E)
}
```

Shared methods do not make the root directly instantiable.

---

## 9. Variants

### 9.1 `@variant`

A `@variant` declaration defines one constructor of its enclosing sealed family.

```phalcom
@variant Ok(value: T)
```

A variant is:

- an ordinary runtime class object;
- a final subclass of its sealed root;
- a compiler-authoritative constructor identity;
- associated with a fixed pattern-component shape;
- included in the root's immutable variant metadata;
- eligible for constructor-pattern matching;
- not externally subclassable.

Variant instances are ordinary objects.

### 9.2 Finality

Every variant is implicitly final:

```phalcom
class CachedOk<T> is Result.Ok<T> {}
```

```text
error: cannot subclass variant Result.Ok

variants are final constructors of their sealed family
```

### 9.3 Runtime identity

Conceptually:

```phalcom
const result = Result.Ok.new(value: 42)

result.class
// Result.Ok

result is Result
// true

result is Result.Ok
// true
```

Normal dispatch, identity, allocation, garbage collection, equality, hashing, and reflection continue to use the ordinary object model.

### 9.4 Reflection

A minimum reflection surface should expose:

```phalcom
Result.isSealed
Result.isAbstract
Result.variants

Result.Ok.isVariant
Result.Ok.sealedFamily
Result.Ok.isFinal
Result.Ok.patternComponents
```

Pattern-component metadata preserves:

- positional or labeled status;
- label, when present;
- declaration order;
- reflective type;
- source location;
- relevant attributes.

### 9.5 Internal tags

An implementation may assign a compact internal ordinal:

```text
Result.Ok  → 0
Result.Err → 1
```

This is an implementation detail. Semantics are defined by constructor identity, not a user-visible numeric tag.

---

## 10. Object-model compatibility

### 10.1 Variants are not a separate value species

The hierarchy remains conceptually:

```text
Object
  └── ordinary class instances
        ├── open classes
        ├── abstract classes
        ├── final classes
        ├── sealed root classes
        └── final variant classes
```

Variants are special declarations and descriptors, not values outside the object model.

### 10.2 Patterns are not expressions

Types and classes remain first-class expressions:

```phalcom
const constructor = Result.Ok
const type = Result<Int, Error>
```

In expression position, `Result.Ok` evaluates to a class object. In pattern position:

```phalcom
Result.Ok(value: item)
```

does not invoke that class object. It tests exact declared variant identity and extracts trusted pattern components.

This does not contradict types being expressions. `Pattern` is a distinct grammar and semantic category.

### 10.3 Intrinsic matching operations

Primitive matching must not use user-overridable dispatch. Trusted operations include:

- exact variant identity;
- nominal runtime class membership;
- direct access to declared variant components;
- tuple and record component access;
- built-in literal matching;
- hidden transfer to `MatchError`.

User interception cannot make an arbitrary object claim to be `Result.Ok`.

### 10.4 Pattern components are not getters

```phalcom
Ok(value: item)
```

must access the declared `value:` component directly. It must not conceptually send `subject.value`.

A getter may expose the same component to normal code, but pattern decomposition must remain stable even when getters are overridden, intercepted, computed, or effectful.

---

## 11. Constructor patterns

### 11.1 Authority

Constructor-pattern syntax is initially available only to compiler-authoritative destructurable forms:

- declared `@variant` constructors;
- tuples;
- immutable records;
- future explicitly derived pattern constructors.

Ordinary class constructors are not automatically invertible.

### 11.2 Ordinary constructors are not patterns

```phalcom
class User {
  @constructor
  new(email: String) {
    _email = email.trim.lowercase
    _createdAt = Clock.now
  }
}
```

Phalcom must not infer that `User(email: address)` reverses `User.new(email:)`. Constructors may normalize, validate, discard, derive, cache, or substitute values.

### 11.3 Exact identity

A variant pattern tests exact final constructor identity:

```phalcom
Ok(value: item)
```

It does not mean open-ended subclass membership. Nominal membership uses a distinct type-test pattern.

---

## 12. Constructor parameter shape

Variant patterns mirror the declared parameter shape.

```phalcom
@sealed @data
class Message {
  @variant Text(content: String)
  @variant Move(Int, Int)
  @variant Rename(from: String, to: String)
  @variant Call(receiver: Expression, Expression, arguments: List<Expression>)
}
```

Valid:

```phalcom
match message {
  Text(content: text) => ...
  Move(x, y) => ...
  Rename(from: oldName, to: newName) => ...
  Call(receiver: target, callee, arguments: args) => ...
}
```

Invalid:

```phalcom
Text(text)
// Text.content is labeled
```

```phalcom
Move(x: x, y: y)
// Move has positional components
```

```phalcom
Rename(oldName, newName)
// Rename.from and Rename.to are labeled
```

### 12.1 Labeled order

Labeled fields may appear in any order. Each label may appear at most once. Unknown or duplicate labels are errors.

### 12.2 Complete structural claim

Without a rest marker, a constructor pattern claims to enumerate the complete component shape.

```phalcom
Failure(error: error)
```

is invalid when other fields exist:

```text
error: incomplete variant pattern

missing fields:
  location:
  notes:

add the missing fields or use `..`
```

---

## 13. Rest patterns

### 13.1 Ignoring omitted components

`..` explicitly ignores every constructor component not otherwise matched:

```phalcom
Failure(error: error, ..)
```

This remains valid when new fields are added later.

### 13.2 Positional placement

A positional `..` may appear once at any position:

```phalcom
Entry(first, .., last)
```

Patterns before `..` align from the beginning; patterns after it align from the end.

For `Entry(A, B, C, D, E)`:

```text
field 0 → first
fields 1–3 → ignored
field 4 → last
```

Prefix and suffix requirements must not overlap.

### 13.3 Rest capture

`..name` captures every omitted component as an immutable structural record:

```phalcom
Entry(first, ..middle, last)
```

```text
first  : A
middle : (B, C, D)
last   : E
```

Capturing zero components produces `()`.

### 13.4 Mixed positional and labeled components

A captured remainder includes every omitted component, positional and labeled, in declaration order.

```phalcom
@variant Invocation(
  receiver: Expression,
  Expression,
  List<Expression>,
  metadata: Metadata,
  location: SourceLocation
)
```

```phalcom
Invocation(
  receiver: target,
  callee,
  ..rest,
  location: location
)
```

```text
rest: (List<Expression>, metadata: Metadata)
```

The residual record is immutable, preserves labels and order, and may be scalar-replaced when it does not escape.

---

## 14. Binding patterns

### 14.1 Bare identifiers bind

An unqualified bare identifier in binding position always introduces a new immutable arm-local binding.

```phalcom
match value {
  item => use(item)
}
```

An outer declaration does not alter the pattern's meaning:

```phalcom
const expected = 42

match value {
  expected => use(expected)
}
```

The arm binds a new `expected`; it does not compare against `42`.

### 14.2 Scope

Bindings are visible in the arm guard, arm body, and nested closures. They are not visible in other arms.

### 14.3 Linearity

A pattern may not bind the same name twice:

```phalcom
(left, left)
```

```text
error: duplicate pattern binding `left`
```

Repeated names do not imply equality. Use a guard:

```phalcom
(left, right) if left == right => ...
```

---

## 15. Pin patterns

### 15.1 Existing-value matching

`^reference` matches against an existing lexical value rather than introducing a binding.

```phalcom
const expected = 42

match value {
  ^expected => equal()
  actual => different(actual)
}
```

### 15.2 Nested pinning

```phalcom
const currentUserId = session.userId

match event {
  UserUpdated(id: ^currentUserId, changes: changes, ..) => apply(changes)
  UserUpdated(id: otherId, ..) => ignore(otherId)
}
```

### 15.3 Operand restriction

The recommended initial rule permits stable references:

```phalcom
^expected
^Constants.defaultPort
```

and rejects arbitrary expressions:

```phalcom
^(minimum + offset)
```

Compute such a value before the match.

### 15.4 Coverage

A runtime pin generally contributes no complete coverage because its value is not statically known.

```phalcom
check(flag: Bool, expected: Bool) {
  match flag {
    ^expected => equal()
  }
}
```

This is non-exhaustive.

The exact equality operation remains open.

---

## 16. Literal and qualified value patterns

Literals compare directly:

```phalcom
match value {
  0 => zero()
  1 => one()
  "quit" => quit()
  true => enabled()
  false => disabled()
}
```

Qualified names may be stable value patterns when they resolve unambiguously:

```phalcom
match color {
  Color.red => stop()
  Color.yellow => wait()
  Color.green => go()
}
```

Bare names never become value patterns merely because a declaration of that name exists.

---

## 17. OR-patterns

### 17.1 Coverage

`p | q` matches the union of the regions matched by `p` and `q`.

```phalcom
match result {
  Ok(value: 0) | Err(err: NotFound()) => fallback()
  Ok(value: value) => use(value)
  Err(err: error) => fail(error)
}
```

### 17.2 Binding agreement

Every alternative must establish the same binding environment.

Valid:

```phalcom
Some(value: value) | Success(value: value) => use(value)
```

Invalid:

```phalcom
Some(value: value) | None => use(value)
```

Invalid:

```phalcom
Some(value: left) | Success(value: right) => ...
```

Invalid:

```phalcom
Some(value: value) | Success(value: ^value) => ...
```

### 17.3 Shared binding types

Corresponding bindings with different types receive a normalized union type.

```phalcom
DogEvent(animal: pet) | CatEvent(animal: pet) => {
  // pet: Dog | Cat
}
```

Normalization includes:

```text
Dog | Dog           → Dog
Dog | Nothing       → Dog
Dog | Animal        → Animal, when Dog <: Animal
(Dog | Cat) | Bird  → Dog | Cat | Bird
```

Only selectors valid for every union member are available without further narrowing.

### 17.4 Alternative usefulness

Redundancy is checked within an OR-pattern and against preceding arms. A redundant alternative is an error even if another alternative remains useful; diagnostics should identify the redundant fragment precisely.

---

## 18. Alias patterns

### 18.1 Syntax

`pattern as name` binds the complete value matched by the pattern.

```phalcom
Some(value: item) as present => {
  use(item)
  cache(present)
}
```

```text
item    : T
present : Some<T>
```

### 18.2 Nested aliases

Aliases may appear at any pattern node:

```phalcom
Response(
  payload: Some(value: item) as payload,
  metadata: metadata,
  ..
) as response => ...
```

### 18.3 No reconstruction

An alias binds the already-matched value. It does not copy or reconstruct it.

### 18.4 Linearity

Alias bindings participate in duplicate-name checks:

```phalcom
Some(value: value) as value
```

is invalid.

### 18.5 Coverage

Aliases do not alter coverage.

---

## 19. Runtime type-test patterns

### 19.1 Ratified conceptual distinction

Constructor patterns and runtime type-test patterns are distinct pattern kinds. They must not share ambiguous syntax such as `Int(number)`, and type syntax must not conflict with labeled constructor fields.

### 19.2 Recommended syntax

The recommended atomic type pattern is:

```phalcom
is Type
```

Binding reuses alias syntax:

```phalcom
is Int as number
```

Example:

```phalcom
match value {
  is Int as number => useInteger(number)
  is String as text => useText(text)
  _ => unsupported(value)
}
```

Nested:

```phalcom
Response(payload: (is Bytes as bytes), ..) => decode(bytes)
```

OR-pattern:

```phalcom
(is Dog | is Cat) as pet => handlePet(pet)
```

### 19.3 Nominal semantics

`is Class` performs nominal runtime membership testing, including subclasses unless the class is final.

```phalcom
match animal {
  is Dog as dog => specialized(dog)
  is Animal as animal => general(animal)
}
```

Reversing the arms makes the `Dog` arm unreachable.

### 19.4 Variant interaction

A final variant type pattern and its constructor pattern may cover the same region:

```phalcom
match result {
  is Result.Ok as ok => use(ok)
  Ok(value: value) => useValue(value)
  Err(err: error) => recover(error)
}
```

The constructor arm is unreachable.

### 19.5 Runtime-testable descriptors

A type pattern may use only a statically resolved, runtime-testable type descriptor.

Initially valid:

```phalcom
is Int
is String
is List
is Animal
is Result.Ok
```

Initially invalid:

```phalcom
is computeType()
is configuration.selectedType
```

Dynamic checks use explicit reflection or guards.

---

## 20. Applied generic types

Applied generic annotations are reflective metadata unless a particular runtime type explicitly reifies and enforces its arguments.

This is generally invalid:

```phalcom
is List<Int>
```

```text
error: type is not runtime-testable in a pattern

List<Int> does not retain or enforce its type argument
use `is List` and preserve generic information through static narrowing
```

Static narrowing retains applied arguments:

```phalcom
process(value: List<Int> | String) {
  match value {
    is List as numbers => {
      // numbers: List<Int>
    }

    is String as text => {
      // text: String
    }
  }
}
```

The runtime checks `List`; the static union supplies the refined `List<Int>` type.

---

## 21. Protocols

Protocols are first-class structural descriptors, not nominal runtime classes. A protocol should not initially be accepted by nominal `is`:

```phalcom
is Strategy<Int> as strategy
```

The initial explicit form uses a guard:

```phalcom
match value {
  candidate if Strategy<Int>.satisfiedBy(candidate) => use(candidate)
  _ => reject(value)
}
```

Because it is a guard, it contributes no coverage. A future dedicated structural pattern such as `satisfies Strategy<Int> as strategy` requires a separate specification.

---

## 22. Generic specialization and uninhabited variants

### 22.1 Specialized coverage

Coverage uses the inhabited portion of the specialized type.

```phalcom
@sealed @data
class Result<out T, out E> {
  @variant Ok(value: T)
  @variant Err(err: E)
}
```

For:

```phalcom
const result: Result<Int, Nothing> = ...
```

this is exhaustive:

```phalcom
match result {
  Ok(value: number) => number
}
```

`Err(err: Nothing)` is uninhabited and removed from the required domain. Matching it explicitly is an impossible-pattern error.

### 22.2 Recursive emptiness

```text
D(Nothing)                 = ∅
D((Int, Nothing))          = ∅
D(Option<Nothing>)         = None
D(Result<Nothing, Error>)  = Err(Error)
```

A container is not necessarily empty because its element type is empty.

### 22.3 Fixed-point inhabitation

```phalcom
@sealed @data
class Natural {
  @variant Zero()
  @variant Successor(previous: Natural)
}
```

`Zero()` establishes inhabitation; therefore `Natural` and `Successor(...)` are inhabited.

```phalcom
@sealed @data
class Impossible {
  @variant Again(value: Impossible)
}
```

With no finite base constructor, `Impossible` is uninhabited under ordinary finite construction.

---

## 23. Empty matches

A zero-arm match is permitted when the scrutinee is provably uninhabited:

```phalcom
absurd<T>(value: Nothing) -> T {
  return match value {}
}
```

The same applies to a user-defined uninhabited type.

A zero-arm match over an inhabited or unknown domain is non-exhaustive.

---

## 24. Hidden runtime fallback

### 24.1 Requirement

Every compiled match retains a hidden defensive fallback. If no arm matches at runtime, the VM throws `MatchError`, even when static analysis proved the match exhaustive.

### 24.2 Motivation

Type annotations are reflective and not automatically enforced. Dynamic invocation, FFI, reflection, unsafe facilities, or implementation defects may pass a value outside the assumed static domain. Static exhaustiveness must not become undefined behavior.

### 24.3 Empty matches

If a real value dynamically enters `match value {}` where `value` was statically `Nothing`, the hidden fallback throws `MatchError`.

### 24.4 Diagnostic payload

`MatchError` should include:

- actual runtime class;
- expected static pattern domain;
- source location;
- optional compiler-generated pattern summary;
- no arbitrary full object dump by default.

---

## 25. Conceptual lowering

```phalcom
match result {
  Ok(value: value) => use(value)
  Err(err: error) => recover(error)
}
```

may lower conceptually to:

```phalcom
const _subject = result

if intrinsicExactVariant(_subject, Result.Ok) {
  const value =
    intrinsicPatternComponent(_subject, Result.Ok, #value)

  use(value)
} else if intrinsicExactVariant(_subject, Result.Err) {
  const error =
    intrinsicPatternComponent(_subject, Result.Err, #err)

  recover(error)
} else {
  throw MatchError.new(
    actualClass: intrinsicClassOf(_subject),
    expectedDomain: Result,
    location: #sourceLocation
  )
}
```

An implementation may instead use a tag switch, jump table, decision tree, or fused tests. Observable behavior must remain unchanged.

---

## 26. Pattern-space analysis

The compiler must perform:

1. scrutinee-domain construction;
2. generic substitution;
3. inhabitation pruning;
4. pattern-domain intersection;
5. arm usefulness checking;
6. OR-alternative usefulness checking;
7. uncovered-domain calculation;
8. representative missing-pattern generation;
9. arm-local type refinement;
10. match-result type calculation.

Guards are excluded from static region subtraction.

The implementation should use a pattern-matrix algorithm or equivalent symbolic decomposition. Recursive analysis must memoize type and substitution states and compute inhabitation through a terminating fixed point.

---

## 27. Diagnostics

Diagnostics are part of the language contract.

### 27.1 Non-exhaustive match

```text
error: non-exhaustive match

scrutinee type:
  Result<Int, Error>

missing pattern:
  Err(err: _)
```

### 27.2 Unreachable arm

```text
error: unreachable match arm

pattern:
  Ok(value: 0)

is already covered by:
  Ok(value: value)
```

### 27.3 Impossible pattern

```text
error: impossible pattern

Err(err: _) cannot inhabit Result<Int, Nothing>

reason:
  Err.err has type Nothing
```

### 27.4 OR binding mismatch

```text
error: OR-pattern alternatives bind different names

left alternative binds:
  value

right alternative binds:
  <none>
```

### 27.5 Duplicate binding

```text
error: duplicate pattern binding `value`

each bare identifier introduces a new binding
use a guard to express equality
```

### 27.6 Non-reifiable type pattern

```text
error: type is not runtime-testable in a pattern

List<Int> does not retain or enforce its type argument
use `is List` instead
```

### 27.7 Incomplete constructor pattern

```text
error: incomplete variant pattern

missing fields:
  location:
  notes:

add the missing fields or use `..`
```

### 27.8 Variant subclassing

```text
error: cannot subclass variant Result.Ok

variants are final constructors of their sealed family
```

---

## 28. Existing generated `.match(...)` eliminators

Existing `@sealed` and `@variant` machinery may generate a labeled eliminator:

```phalcom
status.match(
  valid: { value => ... },
  invalid: { error => ... }
)
```

Native `match` is richer because it supports recursive patterns, literals, OR-patterns, aliases, rest patterns, guards, static refinement, usefulness diagnostics, and uninhabited specialization.

The migration policy remains open. Native matching must never lower to overridable `.match(...)` dispatch.

---

## 29. Reflection obligations

Recommended immutable descriptors:

```phalcom
@data @immutable
class VariantDescriptor {
  const _owner: Class
  const _variantClass: Class
  const _ordinal: Int
  const _components: List<PatternComponent>
}
```

```phalcom
@data @immutable
class PatternComponent {
  const _position: Option<Int>
  const _label: Option<Symbol>
  const _type: TypeRef
  const _attributes: List<Attribute>
  const _sourceLocation: SourceLocation
}
```

Reflection must not permit mutation that adds or removes variants, changes pattern shape, removes finality, or makes the root directly instantiable.

---

## 30. Static typing and narrowing

### 30.1 Arm-local refinement

Each arm receives the intersection of the incoming static type and the arm's pattern region.

```phalcom
process(value: Int | String) {
  match value {
    is Int as number => {
      // number: Int
    }

    is String as text => {
      // text: String
    }
  }
}
```

### 30.2 Constructor refinement

```phalcom
consume(result: Result<Int, Error>) {
  match result {
    Ok(value: value) as ok => {
      // value: Int
      // ok: Result.Ok<Int>
    }

    Err(err: error) as err => {
      // error: Error
      // err: Result.Err<Error>
    }
  }
}
```

### 30.3 OR refinement

```phalcom
(is Dog | is Cat) as pet => {
  // pet: Dog | Cat
}
```

### 30.4 Union selector availability

A selector is available on a union binding only when every union member provides a compatible selector.

---

## 31. Generic variant representation

For:

```phalcom
@sealed @data
class Result<out T, out E> {
  @variant Ok(value: T)
  @variant Err(err: E)
}
```

one clean conceptual relation is:

```text
Result.Ok<T>  <: Result<T, Nothing>
Result.Err<E> <: Result<Nothing, E>
```

This depends on variance and bottom-type semantics. The exact generic expansion remains open.

Ordinary inheritance is not equivalent to `@variant`, because inheritance alone does not establish closed-family membership, pattern-constructor authority, immutable registration, finality, canonical components, or exhaustiveness participation.

---

## 32. Positive conformance examples

### 32.1 Exhaustive sealed match

```phalcom
describe(result: Result<Int, Error>) -> String {
  return match result {
    Ok(value: value) => "ok: \(value)"
    Err(err: error) => "error: \(error)"
  }
}
```

### 32.2 Uninhabited specialization

```phalcom
unwrap(result: Result<Int, Nothing>) -> Int {
  return match result {
    Ok(value: value) => value
  }
}
```

### 32.3 Nested closed match

```phalcom
match result {
  Ok(value: Some(value: item)) => use(item)
  Ok(value: None) => absent()
  Err(err: error) => fail(error)
}
```

### 32.4 OR-pattern union binding

```phalcom
match event {
  DogEvent(animal: pet) | CatEvent(animal: pet) => handlePet(pet)
  BirdEvent(animal: bird) => handleBird(bird)
}
```

### 32.5 Alias and nested alias

```phalcom
match response {
  Response(
    payload: Some(value: item) as payload,
    ..
  ) as response => {
    cache(response)
    use(payload)
    consume(item)
  }

  Response(payload: None, ..) => absent()
}
```

### 32.6 Rest capture

```phalcom
match invocation {
  Invocation(
    receiver: target,
    callee,
    ..rest,
    location: location
  ) => inspect(target, callee, rest, location)
}
```

### 32.7 Open-domain type matching

```phalcom
match value {
  is Int as number => number.abs
  is String as text => text.size
  _ => 0
}
```

---

## 33. Negative conformance examples

The following must fail:

### 33.1 Missing variant

```phalcom
match result {
  Ok(value: value) => use(value)
}
```

### 33.2 Useless arm

```phalcom
match result {
  Ok(value: value) => use(value)
  Ok(value: 0) => zero()
  Err(err: error) => recover(error)
}
```

### 33.3 Guard-only coverage

```phalcom
match value {
  is Int as number if number > 0 => ...
  is Int as number if number <= 0 => ...
}
```

### 33.4 Duplicate binding

```phalcom
match pair {
  (value, value) => ...
}
```

### 33.5 OR mismatch

```phalcom
match option {
  Some(value: value) | None => ...
}
```

### 33.6 External sealed extension

```phalcom
class Pending<T, E> is Result<T, E> {}
```

### 33.7 Variant subclassing

```phalcom
class CachedOk<T> is Result.Ok<T> {}
```

### 33.8 Non-reified applied generic test

```phalcom
match value {
  is List<Int> as numbers => ...
  _ => ...
}
```

### 33.9 Protocol as nominal type

```phalcom
match value {
  is Strategy<Int> as strategy => ...
  _ => ...
}
```

---

## 34. Compiler obligations

The compiler must:

1. parse patterns separately from expressions;
2. resolve constructor patterns only through authoritative pattern constructors;
3. preserve exact variant identity;
4. construct symbolic pattern domains;
5. substitute generic parameters;
6. compute inhabitation through fixed points;
7. prune impossible constructors;
8. check arm and OR-alternative usefulness;
9. reject duplicate bindings;
10. verify OR binding agreement;
11. infer union types for OR-bound names;
12. infer residual-record types;
13. type-check guards in the binding environment;
14. ignore guards for static coverage;
15. infer arm-local refinements;
16. compute match result types;
17. generate a hidden `MatchError` fallback;
18. emit source-ranged diagnostics;
19. preserve reflection metadata;
20. guarantee that separate compilation cannot invalidate sealed closure.

---

## 35. Interpreter and VM obligations

The interpreter and VM must provide trusted primitives for:

- exact variant testing;
- nominal class-membership testing;
- direct pattern-component extraction;
- tuple and record decomposition;
- hidden match failure;
- efficient variant dispatch;
- safe handling of dynamically invalid values.

These primitives are not user-overridable.

An optimized VM may assign ordinals, generate jump tables, fuse nested tests, and scalar-replace residual records or aliases. Normal semantics retain defensive safety.

---

## 36. Tooling obligations

### 36.1 IDEs

Editors should support:

- completion for missing variants;
- quick fixes adding missing arms;
- quick fixes adding `..`;
- display of arm-local refined types;
- unreachable-arm diagnostics;
- navigation from patterns to variant declarations;
- rename support for labeled components.

### 36.2 Formatter

The formatter should handle long constructor patterns, OR-patterns, nested aliases, mixed components, guards, and rest captures.

### 36.3 Documentation

Generated documentation should show sealed status, the complete variant list, component shapes, finality, generic relationships, and matching examples.

---

## 37. Acceptance-test matrix

A conforming implementation must test at least:

### 37.1 Coverage

- every variant covered;
- one missing variant;
- wildcard fallback;
- nested sealed families;
- tuple and Boolean products;
- closed unions;
- constrained generics;
- empty domains;
- recursive inhabited and uninhabited families.

### 37.2 Usefulness

- duplicate arms;
- broad-before-narrow and narrow-before-broad;
- partial overlap;
- redundant OR alternatives;
- impossible specialized variants;
- impossible type tests.

### 37.3 Guards

- guarded arm with unguarded fallback;
- complementary guards without fallback;
- binding access;
- throwing and effectful guards;
- source-order evaluation.

### 37.4 Bindings

- arm-local scope;
- aliases and nested aliases;
- duplicate binding rejection;
- pin versus binding;
- shadowing;
- OR name agreement;
- OR union inference.

### 37.5 Constructor shape

- positional, labeled, and mixed components;
- reordered labels;
- missing fields without rest;
- unknown and duplicate labels;
- rest at beginning, middle, and end;
- empty rest capture;
- mixed residual records.

### 37.6 Runtime fallback

- dynamically invalid values entering exhaustive and empty matches;
- `MatchError` payload;
- absence of undefined behavior.

### 37.7 Object model

- variant reflection;
- ordinary dispatch on variant instances;
- finality;
- root construction rejection;
- external extension rejection;
- interception cannot falsify identity;
- getters are not invoked by decomposition.

### 37.8 Generics and protocols

- `Nothing` pruning;
- static generic narrowing through a raw class test;
- rejection of non-reified applied types;
- rejection of nominal protocol `is`;
- explicit protocol guard behavior.

---

## 38. Open decisions and recommendations

This section is non-normative until each decision is ratified.

### 38.1 Exact type-pattern spelling

**Question:** How should nominal runtime type testing be written?

**Recommendation:**

```phalcom
is Int as number
```

**Justification:** `is Int` is an atomic test-only pattern; `as number` reuses aliasing. `number: Int` conflicts with labeled patterns, `Int(number)` conflicts with constructor patterns, and `number is Int` creates a second complete-value binding form.

### 38.2 Nullary variant syntax

**Question:** Should a nullary user variant appear as `Pending` or `Pending()`?

**Recommendation:** Require:

```phalcom
Pending()
```

**Justification:** Bare identifiers always bind. A bare nullary constructor would recreate scope-sensitive ambiguity. The local rules become:

```text
name       → bind
^name      → compare with an existing value
Name()     → constructor pattern
is Name    → nominal type test
```

Language-defined literals such as `true`, `false`, and perhaps `None` may remain literal patterns.

### 38.3 `_` versus `else`

**Question:** Should both exist?

**Recommendation:** Make `_` the core construct. Consider `else` only as terminal syntactic sugar for `_`.

**Justification:** `_` composes inside nested patterns. Two semantically equivalent fallback forms may create unnecessary style divergence.

### 38.4 Match result typing

**Question:** How is a match expression typed?

**Recommendation:** Use the normalized union of all reachable arm result types, ignoring `Nothing`.

```text
Dog | Cat | Nothing → Dog | Cat
```

This matches OR-binding union semantics and preserves more precision than immediate widening.

### 38.5 Arm bodies and separators

**Recommendation:** Permit either one expression or a block after `=>`, with newline separation consistent with Phalcom blocks. Avoid mandatory commas.

```phalcom
match value {
  Ok(value: value) => value

  Err(err: error) => {
    log(error)
    fallback
  }
}
```

### 38.6 Alias and OR precedence

**Recommendation:** `as` binds more weakly than `|`:

```phalcom
is Dog | is Cat as pet
```

means:

```phalcom
(is Dog | is Cat) as pet
```

Formatters should add parentheses in complex code.

### 38.7 Pin equality semantics

**Question:** Does `^expected` use ordinary equality, identity, or a dedicated relation?

**Recommendation:** Use ordinary Phalcom equality, with these constraints:

- evaluate the pinned reference once;
- perform at most one equality operation per pin test;
- propagate equality exceptions normally;
- do not count runtime pins toward exhaustiveness unless constant-folded;
- reserve identity matching for a distinct future pattern.

Because runtime pins do not prove coverage, overridable equality does not undermine exhaustiveness.

### 38.8 Constant patterns

**Recommendation:** Only compiler-known immutable constants with stable evaluation contribute literal-like coverage. Dynamic getters and mutable variables require pinning or guards and do not establish closed coverage.

### 38.9 Generic variant expansion

For covariant `Result`, the clean relation is:

```text
Ok<T>  <: Result<T, Nothing>
Err<E> <: Result<Nothing, E>
```

**Recommendation:** Use `Nothing` for omitted covariant parameters. For invariant or contravariant parameters, require explicit variant parameters, an explicit existential family mechanism, or a diagnostic requiring compatible variance. Never silently substitute `_`, `Any`, or `Dynamic`.

### 38.10 Variant namespacing

**Recommendation:** Give variants canonical qualified identities:

```phalcom
Result.Ok
Result.Err
```

Permit unqualified `Ok(...)` and `Err(...)` when imported or resolved from the scrutinee's sealed family.

### 38.11 Exact versus subclass type tests

**Recommendation:** `is Type` includes subclasses, matching ordinary nominal membership. A future exact form may be:

```phalcom
is exactly Type
```

Do not overload constructor patterns for ordinary exact-class tests.

### 38.12 User-defined extractors

**Recommendation:** Exclude them from the first version.

A future extractor system must define purity, effects, failure, exceptions, exhaustiveness authority, overlap, caching, optimization, and interception.

### 38.13 `@data` destructuring

**Recommendation:** Defer automatic destructuring for arbitrary `@data` classes. It is safe only if `@data` guarantees canonical immutable component metadata independent of constructors and getters.

### 38.14 Protocol patterns

**Recommendation:** Defer to a distinct future pattern:

```phalcom
satisfies Strategy<Int> as strategy
```

Such patterns should probably remain non-covering because protocols are open structural properties.

### 38.15 Applied generic runtime patterns

**Recommendation:** Permit only through a future explicit reification contract defining storage, mutation, inheritance, variance, reflection stability, and runtime cost. Annotation syntax alone must not imply reification.

### 38.16 Existing `.match(...)` API

**Recommendation:** Retain generated eliminators temporarily for compatibility, make native `match` canonical, and eventually gate generation behind an explicit compatibility attribute or remove it in a major version. Native matching never lowers to `.match(...)` dispatch.

### 38.17 Shadowing diagnostics

**Recommendation:** Permit pattern bindings to shadow outer locals, with an optional lint:

```text
pattern binding `value` shadows an existing local
use `^value` if comparison was intended
```

### 38.18 Redundant OR fragments

**Recommendation:** Reject redundant alternatives even when the arm remains useful, and identify the exact redundant fragment.

### 38.19 Match-arm fallthrough

**Recommendation:** Do not support fallthrough or grouped headers. Use OR-patterns:

```phalcom
Dog(..) | Cat(..) => handlePet()
```

Fallthrough complicates binding environments and result typing without adding expressive power.

### 38.20 Exhaustiveness over open inheritance

**Recommendation:** An open hierarchy is exhaustive without `_` only when an arm is irrefutable over the static domain:

```phalcom
match animal {
  is Animal as value => use(value)
}
```

Enumerating currently known subclasses never establishes exhaustiveness.

---

## 39. Initial non-goals

The initial version does not include:

- user-overridable pattern dispatch;
- arbitrary constructor inversion;
- theorem proving for guards;
- structural protocol exhaustiveness;
- implicit generic runtime validation;
- dynamic sealed-family mutation;
- variant subclassing;
- direct sealed-root construction;
- arm fallthrough;
- duplicate-binding equality semantics;
- arbitrary expressions inside pin patterns;
- arbitrary computed nominal type patterns;
- undefined behavior for failed supposedly exhaustive matches.

---

## 40. Ratification checklist

Before final ratification, explicitly resolve:

1. exact `is Type` syntax;
2. mandatory `Variant()` syntax for nullary user variants;
3. `_` versus `else`;
4. match result type inference;
5. arm-body and separator grammar;
6. alias and OR precedence;
7. pin equality semantics;
8. generic variant expansion under variance;
9. canonical variant namespacing;
10. migration from generated `.match(...)`.

The core implementation can proceed once these syntax and typing boundaries are fixed.

---

## 41. Summary of ratified semantics

The ratified core is:

- exhaustiveness is a hard compile-time requirement;
- it applies to every compiler-provable closed decomposition;
- guards contribute no static coverage;
- wholly unreachable and impossible arms are errors;
- sealed families are declaration-owned and closed;
- sealed roots are abstract and not directly inhabited;
- variants are final;
- specialized uninhabited variants are pruned;
- empty matches are allowed for uninhabited scrutinees;
- every match retains a hidden runtime `MatchError`;
- constructor patterns mirror positional and labeled declaration shape;
- labeled components may be reordered;
- omitted components require `..`;
- positional rest may appear once anywhere;
- `..name` captures the complete omitted residual record;
- bare identifiers always bind;
- `^name` matches an existing value;
- pattern bindings are linear;
- OR alternatives bind the same names;
- OR-bound values retain normalized union types;
- `pattern as name` aliases the complete matched value;
- aliases may occur at any pattern node;
- constructor patterns and runtime type tests are distinct;
- variants remain ordinary classes and their instances ordinary objects;
- matching uses trusted metadata rather than overridable dispatch;
- applied generic annotations are not automatically runtime-testable;
- protocols are not nominal runtime classes.

This model is coherent with Phalcom's reflective type expressions, dynamic runtime, selector-oriented object model, and attribute-driven class generation.
