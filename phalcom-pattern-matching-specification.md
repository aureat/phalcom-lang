# Phalcom Pattern Matching Specification

**Status:** Consolidated language and semantic architecture specification  
**Version:** 1.0-draft  
**Date:** 2026-09-04  
**Repository baseline:** `aureat/phalcom-lang` at `e932aac4e21a5b346e719ede5a24f94e7b924ab3`  
**Audience:** language designers, compiler and semantic-analyzer implementers, runtime implementers, tooling authors, standard-library authors, and conformance-test maintainers

---

# 1. Purpose

This document specifies Phalcom's pattern-matching system as a coherent language feature and semantic subsystem.

It consolidates the currently implemented and ratified ADT/GADT matching model with the architectural requirements discovered during the SC-4.8 recursive-exhaustiveness investigation. It defines:

- `match` as a first-class expression;
- the recursive pattern language;
- exact variant patterns and associated variant-family patterns;
- contextual pattern resolution;
- wildcard and binding semantics;
- tuple, list, record, map, nested, and or-patterns;
- strict exhaustiveness over statically closed domains;
- usefulness, redundancy, and impossible-pattern detection;
- union and exact-case participation;
- GADT elimination and branch-local equality proofs;
- constructor-local generic opening during elimination;
- recursive pattern matching and termination requirements;
- demand-driven constructor decomposition;
- the pattern-matrix model for exhaustiveness and usefulness;
- branch-local type refinement and match-result typing;
- missing-case witness generation;
- the semantic/runtime authority boundary;
- diagnostics, conformance laws, and performance guarantees;
- reserved extensions such as guards, literal patterns, pinned values, aliases, and type-test patterns.

The goal is not merely to describe syntax. Pattern matching in Phalcom is a **static elimination system**: it connects source syntax, ADT/GADT identity, type refinement, exhaustiveness proofs, control flow, compiler lowering, and runtime constructor identity.

---

# 2. Normative Status and Precedence

## 2.1 Current authoritative core

The following capabilities form the current authoritative core of the pattern-matching system:

- `match` expressions;
- wildcard patterns;
- name-binding patterns;
- exact variant patterns;
- contextual owner shorthand;
- associated variant-family patterns;
- callable selector-family patterns using `...`;
- whole-family patterns using `*`;
- tuple patterns;
- list patterns with rest binding;
- record patterns;
- map patterns;
- or-patterns;
- nested recursive patterns;
- ordered first-match semantics;
- strict exhaustiveness for statically closed domains;
- impossible/redundant/useful classification;
- GADT equality refinement;
- exact-case evidence;
- constructor-local generic opening during GADT elimination;
- branch-local proof environments;
- union participation;
- transparent alias participation through canonical types;
- stable semantic match products consumed by lowering.

## 2.2 Architectural correction ratified by this specification

The coverage engine must obey the following root architectural rule:

> **Pattern matching is a finite symbolic elimination procedure over source patterns. It is not an eager enumeration of all values inhabiting the scrutinee type.**

Recursive ADT/GADT payloads are therefore not recursively expanded merely because their static types are recursively decomposable.

Types determine which constructors are possible at the current scrutiny position. Source patterns determine how deeply constructor payloads are inspected.

## 2.3 Reserved or planned extensions

The following concepts exist in broader design documents but are not treated here as already-ratified syntax unless separately approved:

- guards on match arms;
- literal patterns;
- ranges;
- pinned-value patterns;
- as/alias patterns;
- nominal `is` type-test patterns inside the pattern grammar;
- view/active patterns;
- user-defined extractors.

This document specifies their compatibility requirements where useful, but clearly marks them as extensions.

## 2.4 Precedence

Where older implementation documents conflict with this document, precedence is:

1. explicit ratified project decisions;
2. this consolidated pattern-matching specification;
3. current ADT/GADT semantic architecture;
4. current repository implementation where it does not conflict with the above;
5. older exploratory or draft documents.

---

# 3. Design Goals

Phalcom pattern matching has the following design goals.

## 3.1 Totality

Every accepted `match` over a statically closed domain is proven exhaustive.

The compiler does not silently permit a missing constructor and defer failure to ordinary runtime fallthrough.

## 3.2 Usefulness

Every unguarded match arm must contribute at least one reachable value not already covered by earlier arms.

Completely redundant arms are rejected.

## 3.3 GADT correctness

Matching a GADT constructor is a proof-producing operation. Constructor observation may establish type equalities that hold only inside the selected branch.

## 3.4 Object-model compatibility

Variants remain ordinary runtime values/classes according to Phalcom's ADT runtime model. Pattern syntax does not introduce an unrelated second value system.

## 3.5 Trusted structural matching

Pattern matching must not depend on user-overridable message dispatch for primitive constructor identity or payload extraction.

## 3.6 Finite semantic analysis

Recursive datatypes may describe infinitely many finite values. Pattern analysis must still terminate for every finite source pattern matrix, subject only to explicit semantic-analysis budgets.

## 3.7 Stable semantic authority

The semantic analyzer resolves exact constructor identities, field identities, GADT proofs, bindings, and coverage facts. Compiler lowering consumes those semantic products and must not rediscover them from source spelling.

## 3.8 Expressive recursive decomposition

Nested ADT/GADT patterns must support arbitrary finite source depth without requiring eager expansion of recursive types.

---

# 4. Core Terminology

## 4.1 Scrutinee

The expression being matched:

```phalcom
match result {
    Result::Ok(value) => value
    Result::Err(error) => recover(error)
}
```

`result` is the scrutinee.

## 4.2 Pattern

A pattern describes a set of values and may introduce bindings or semantic refinement.

Patterns are not expressions. A source form may have different meaning in expression position and pattern position.

## 4.3 Pattern domain

The static set of values admitted at one scrutiny position.

A domain may contain infinitely many values while still having a finite constructor decomposition.

For example:

```text
Expression<F,T>
```

may have infinitely many expression trees, but its outer constructor set is finite.

## 4.4 Pattern region

The subset of the current domain accepted by one pattern.

## 4.5 Closed decomposition

A domain has a closed decomposition when the compiler can determine every possible alternative at the current scrutiny position.

Examples include:

- a closed enum/ADT;
- a GADT after feasibility filtering;
- an exact-case type;
- a closed union;
- a tuple with known arity;
- `Bool` when literal matching is available;
- other compiler-authoritative closed structural domains.

## 4.6 Open domain

A domain is open when the compiler cannot enumerate every possible alternative.

Typical examples include:

- `Dynamic`;
- `Object`;
- open class hierarchies;
- unconstrained type parameters;
- open protocols or structural domains not proven closed.

## 4.7 Irrefutable pattern

A pattern is irrefutable relative to a domain when it accepts every value in that domain.

The canonical examples are:

```phalcom
_
value
```

A name pattern binds the value; `_` discards it.

## 4.8 Refutable pattern

A pattern is refutable when some value in the current domain can fail to match it.

Examples:

```phalcom
Option::Some(x)
Expression::Add(left, right)
(a, b)
[first, *rest]
```

whether a particular pattern is actually refutable can depend on the current static domain.

## 4.9 Constructor head

The outer structural discriminator used by coverage analysis.

For ADTs/GADTs this is a `VariantId`.

## 4.10 Exact case

An exact-case type denotes one specific variant constructor under a specific enclosing enum specialization.

Exact-case evidence can be introduced by successful pattern matching and is useful for branch-local flow refinement.

## 4.11 Coverage

Coverage is the static proof problem of determining which values are accepted by previous patterns and whether any reachable value remains uncovered.

## 4.12 Usefulness

A pattern is useful if it matches at least one reachable value not matched by previous relevant patterns.

---

# 5. Match Is an Expression

The canonical form is:

```phalcom
match scrutinee {
    pattern => expression
    pattern => {
        statements
        expression
    }
}
```

Example:

```phalcom
const message = match result {
    Result::Ok(value) => "ok: ${value}"
    Result::Err(error) => {
        log(error)
        "failed"
    }
}
```

Normative properties:

1. `match` evaluates its scrutinee once.
2. Arms are considered in source order.
3. The first successful arm is selected.
4. Each arm body is an expression.
5. A braced arm body contributes its block result.
6. Match-result typing joins the result types of reachable normal-completing arms.
7. A match whose reachable arms cannot complete normally has type `Never`.
8. Exhaustiveness is independent of whether the result is used.
9. Pattern matching is elimination/control flow, not ordinary message dispatch.

---

# 6. Core Grammar

Conceptually:

```text
MatchExpression
    ::= "match" Expression "{" MatchArm* "}"

MatchArm
    ::= Pattern "=>" Expression

Pattern
    ::= WildcardPattern
     |  BindingPattern
     |  VariantPattern
     |  OrPattern
     |  TuplePattern
     |  ListPattern
     |  RecordPattern
     |  MapPattern
```

The current parser supports patterns equivalent to:

```text
_                       wildcard
x                       name binding
(p1, p2, ...)           tuple
[p1, p2, *rest]         list
#{ ... }                 record
{# ... }                 map
A::Variant(...)          qualified variant
Variant(...)             contextual variant
A::Variant*              whole family
Variant(...)             contextual family/member
Variant(x, ..., named:y) callable family selector pattern
p1 | p2                  or-pattern
```

The variant/family grammar reuses Phalcom's selector and associated-lookup model rather than inventing a separate constructor naming system.

---

# 7. Wildcard Pattern

The wildcard is:

```phalcom
_
```

It:

- matches every value in the current domain;
- binds no name;
- introduces no equality relationship;
- is recursive and may appear at any pattern position;
- is not a special-case syntax reserved only for the final arm.

Examples:

```phalcom
match option {
    Option::Some(_) => true
    Option::None() => false
}
```

```phalcom
match expression {
    Expression::Add(_, _) => "addition"
    _ => "other"
}
```

```phalcom
match value {
    Result::Err(_, context: _) => recover()
    _ => continueProcessing()
}
```

For coverage purposes, a wildcard means:

```text
all inhabitants of the current static subject
```

and therefore does not require recursive decomposition of that subject.

---

# 8. Binding Patterns

A bare name in pattern position binds the matched value:

```phalcom
match option {
    Option::Some(value) => value
    Option::None() => fallback()
}
```

`value` is a new branch-local binding.

A binding pattern is irrefutable relative to its current domain.

For coverage it is equivalent to `_`, but semantically it additionally publishes:

- a `BindingId`;
- `TypeKnowledge`;
- when required, a branch-local `LocalType` containing constructor-local rigid variables;
- source provenance.

Repeated name binding inside one alternative is invalid under the current linear-binding model. A repeated spelling is not an equality constraint.

---

# 9. Exact Variant Patterns

Phalcom variant identity is selector-aware. Distinct selector shapes may denote distinct variants even when they share the same base name.

Consider:

```phalcom
enum Animal {
    @variant Dog
    @variant Dog()
    @variant Dog(_ name: String)
    @variant Dog(_ name: String, named age: Int)
}
```

These are distinct exact patterns:

```phalcom
Animal::Dog
Animal::Dog()
Animal::Dog(name)
Animal::Dog(name, named: age)
```

Pattern matching therefore aligns with the same selector identity model used by associated lookup and invocation.

## 9.1 Singleton variant

```phalcom
Animal::Dog
```

matches the singleton/getter-shaped exact variant.

## 9.2 Nullary constructor variant

```phalcom
Animal::Dog()
```

matches the exact callable nullary constructor variant.

It is semantically distinct from a singleton variant even though both carry no payload.

## 9.3 Positional constructor variant

```phalcom
Animal::Dog(name)
```

matches the exact callable selector with the corresponding positional shape.

## 9.4 Labeled constructor variant

```phalcom
Animal::Dog(name, named: age)
```

matches the exact selector shape including labels.

Payload positions are resolved through semantic `VariantFieldId`s, not by re-reading source declarations during lowering.

---

# 10. Contextual Variant Shorthand

When the expected pattern domain identifies a unique owner, the owner may be omitted:

```phalcom
match animal {
    Dog(name) => handleDog(name)
    Cat(name) => handleCat(name)
}
```

instead of:

```phalcom
match animal {
    Animal::Dog(name) => handleDog(name)
    Animal::Cat(name) => handleCat(name)
}
```

Contextual shorthand is semantic resolution, not textual guessing.

The analyzer must:

1. determine owners compatible with the current expected domain;
2. resolve the selector shape;
3. retain all declaration-backed candidates if ambiguity remains;
4. issue a diagnostic if one unambiguous semantic interpretation cannot be established where required.

Compiler lowering must never repeat this contextual name-resolution process.

---

# 11. Variant Families

Phalcom's associated lookup model permits multiple related variants to form a family.

Pattern matching reuses that model directly.

## 11.1 Whole-family pattern

```phalcom
Animal::Dog*
```

matches every reachable member of the `Dog` variant family.

A family pattern may therefore match several exact `VariantId`s.

Example:

```phalcom
enum Message {
    @variant Event()
    @variant Event(_ payload: String)
    @variant Event(_ payload: String, source: String)
    @variant Error(_ message: String)
}

match message {
    Message::Event* => handleAnyEvent(message)
    Message::Error(error) => fail(error)
}
```

The first arm is a closed set operation over the declaration-backed `Event` family, not a runtime name-prefix test.

## 11.2 Contextual family shorthand

```phalcom
match message {
    Event* => handleAnyEvent(message)
    Error(error) => fail(error)
}
```

is valid when the scrutinee domain identifies the owner.

---

# 12. Callable Selector-Family Patterns

Phalcom can structurally match callable selector families using `...`.

Examples:

```phalcom
Animal::Dog(...)
Animal::Dog(name, ...)
Animal::Dog(..., named: age)
Animal::Dog(name, ..., named: age)
```

The `...` gap denotes zero or more selector slots between a required prefix and suffix.

This does not mean arbitrary payload sequence matching. It is a static selector-family constraint.

Given variants:

```phalcom
@variant Request(_ url: String)
@variant Request(_ url: String, timeout: Int)
@variant Request(_ url: String, timeout: Int, retries: Int)
```

then:

```phalcom
Request(url, ...)
```

can denote every reachable `Request` constructor whose selector begins with the required positional `url` slot.

Fields not explicitly constrained by the source pattern are treated as wildcard payload positions.

Example:

```phalcom
match request {
    Request(url, ...) => logRequest(url)
    _ => ignore()
}
```

Only `url` is decomposed/bound. Uninspected trailing fields remain semantically universal at their specialized types.

---

# 13. Nested Variant Patterns

Patterns compose recursively.

```phalcom
match value {
    Option::Some(Result::Ok(value)) => use(value)
    Option::Some(Result::Err(error)) => recover(error)
    Option::None() => absent()
}
```

This is statically exhaustive for:

```text
Option<Result<T,E>>
```

because the nested `Result` domain is decomposed only when the `Some` branch requires it.

Another example:

```phalcom
match expression {
    Expression::Add(
        Expression::IntLiteral(left),
        Expression::IntLiteral(right)
    ) => left + right
    _ => evaluateNormally(expression)
}
```

The compiler does not recursively inspect every `Expression` payload. It inspects exactly the positions demanded by the nested source pattern.

---

# 14. Or-Patterns

Alternatives are written with `|`:

```phalcom
Result::Ok(value) | Result::Cached(value) => use(value)
```

An or-pattern denotes the union of its alternative pattern regions.

## 14.1 Binding consistency

Every successful alternative must introduce the same binding names.

Valid:

```phalcom
Result::Ok(value) | Result::Cached(value) => use(value)
```

Invalid:

```phalcom
Result::Ok(value) | Result::Cached(cached) => use(value)
```

because one alternative binds `value` and the other binds `cached`.

## 14.2 Binding types

The type of a shared binding is the semantic join of the alternative-specific binding types when the alternatives can legitimately produce different but joinable types.

## 14.3 Proof intersection

Only proof facts valid for every successful alternative survive into the common branch.

If one GADT alternative establishes:

```text
T = Int
```

and another establishes:

```text
T = Bool
```

then the shared branch cannot assume either equality unless a common proof relation exists.

## 14.4 Alternative usefulness

Or-pattern alternatives are checked in source order for internal redundancy.

Example:

```phalcom
A(_) | A(0)
```

has a redundant second alternative if the first already covers every `A` case admitted at that position.

---

# 15. Tuple Patterns

Tuple patterns match fixed tuple structure:

```phalcom
match pair {
    (left, right) => combine(left, right)
}
```

Nested tuples are allowed:

```phalcom
match value {
    ((x, y), z) => f(x, y, z)
}
```

Tuple component patterns are recursively resolved against their component types.

A tuple pattern is structurally closed when the scrutinee type is a known tuple of matching arity.

Overlap remains meaningful:

```phalcom
match pair {
    (true, _) => first()
    (_, true) => second()
    (false, false) => third()
}
```

The second arm is useful because `(false, true)` remains uncovered after the first.

---

# 16. List Patterns

List patterns use bracket syntax:

```phalcom
match values {
    [] => empty()
    [first] => one(first)
    [first, second] => two(first, second)
    [first, *rest] => many(first, rest)
}
```

The parser's pattern model distinguishes fixed prefix elements from an optional rest pattern.

The rest binding receives the remaining sequence, not one element.

Example:

```phalcom
match path {
    [root, *segments] => walk(root, segments)
    [] => currentDirectory()
}
```

Nested list elements may themselves be patterns:

```phalcom
match results {
    [Result::Ok(first), *rest] => process(first, rest)
    [Result::Err(error), *rest] => fail(error, rest)
    [] => done()
}
```

Whether all list shapes can be proven exhaustive depends on the list-space algebra available to the analyzer. The compiler must not manufacture totality where structural proof is incomplete.

---

# 17. Record Patterns

Record patterns use:

```phalcom
#{ ... }
```

Example:

```phalcom
match user {
    #{name: name, age: age} => describe(name, age)
}
```

For a statically known record row, each requested field is resolved against the corresponding field type.

Example with nested matching:

```phalcom
match response {
    #{status: Result::Ok(code), body: body} => accept(code, body)
    #{status: Result::Err(error), body: _} => reject(error)
}
```

## 17.1 Open rows

For open record rows, the presence of explicitly mentioned fields may be provable while the complete object domain remains open.

A record pattern is therefore generally a refutable structural predicate unless the analyzer can prove it irrefutable for the current row type.

Record coverage must not silently close an open row.

## 17.2 Field-local recursion

Only mentioned fields are recursively pattern-resolved.

Unmentioned fields do not cause recursive coverage decomposition.

---

# 18. Map Patterns

Map patterns use:

```phalcom
{# ... }
```

Example:

```phalcom
match metadata {
    {# "status": status, "requestId": id } => handle(status, id)
    _ => missingMetadata()
}
```

A map pattern represents required runtime key tests.

A finite list of required-key patterns does **not** normally prove that every value of a general map domain is covered.

Therefore map patterns remain refutable unless a stronger closed-domain proof is available.

Nested value patterns are supported:

```phalcom
match payload {
    {# "result": Result::Ok(value) } => use(value)
    {# "result": Result::Err(error) } => recover(error)
    _ => malformed()
}
```

---

# 19. Pattern Composition

Pattern forms compose arbitrarily at finite source depth.

Example:

```phalcom
match message {
    Envelope::Data(
        #{headers: {# "kind": kind }, body: Option::Some(Result::Ok(value))}
    ) => route(kind, value)

    Envelope::Data(
        #{headers: _, body: Option::Some(Result::Err(error))}
    ) => recover(error)

    Envelope::Data(
        #{headers: _, body: Option::None()}
    ) => missingBody()

    Envelope::Control(command) => execute(command)
}
```

The key property is that the analyzer follows the finite pattern tree. It does not recursively expand the entire static type graph before analyzing the source pattern.

---

# 20. Ordered First-Match Semantics

Arms are ordered.

```phalcom
match value {
    Some(_) => general()
    Some(0) => special()
    None() => absent()
}
```

If literal patterns are eventually introduced, the second arm would be redundant because the first already covers it.

The current semantic categories are:

```text
Useful
Redundant
Impossible
```

These categories are semantically distinct.

---

# 21. Impossible Patterns

A pattern is impossible when it matches no value in the original scrutinee domain.

Example GADT:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

For:

```phalcom
fn(_ value: Expr<Int>) {
    match value {
        Expr::Bool(x) => x
        Expr::Int(x) => x
    }
}
```

`Expr::Bool(x)` is impossible because observing that constructor would require:

```text
Int = Bool
```

which is contradictory.

Impossible is measured against the original domain, not merely against values left by earlier arms.

---

# 22. Redundant Patterns

A pattern is redundant when:

1. it is possible in the original domain; but
2. every value it could match has already been covered by earlier arms.

Example:

```phalcom
match value {
    Result::Ok(x) => first(x)
    Result::Ok(y) => second(y)
    Result::Err(error) => recover(error)
}
```

The second `Ok` arm is redundant.

This distinction matters for diagnostics:

```text
Impossible:
    the pattern can never occur for this scrutinee type

Redundant:
    the pattern could occur, but earlier arms already handle it
```

---

# 23. Strict Exhaustiveness

For every statically closed domain, an accepted `match` must cover every reachable inhabited value.

Example:

```phalcom
enum TrafficLight {
    @variant Red()
    @variant Yellow()
    @variant Green()
}
```

Valid:

```phalcom
match light {
    TrafficLight::Red() => stop()
    TrafficLight::Yellow() => prepare()
    TrafficLight::Green() => go()
}
```

Invalid:

```phalcom
match light {
    TrafficLight::Red() => stop()
    TrafficLight::Green() => go()
}
```

The compiler must report the missing reachable case, conceptually:

```text
TrafficLight::Yellow()
```

---

# 24. Closed and Open Domains

## 24.1 Closed domain

For a closed ADT:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None()
}
```

this is exhaustive:

```phalcom
match option {
    Option::Some(value) => value
    Option::None() => fallback()
}
```

## 24.2 Open domain

For a scrutinee whose static domain is open:

```text
Object
Dynamic
open hierarchy
```

structured alternatives alone cannot usually prove totality.

A final irrefutable arm is required:

```phalcom
match value {
    KnownVariant::A(x) => useA(x)
    KnownVariant::B(x) => useB(x)
    _ => fallback(value)
}
```

The analyzer must never confuse:

```text
closed but not yet decomposed
```

with:

```text
semantically open / not enumerable
```

This distinction is fundamental to recursive matching.

---

# 25. Union Types

Canonical union types participate directly in coverage.

Suppose:

```text
A | B | C
```

The initial domain is the union of the reachable spaces of `A`, `B`, and `C`.

A match may cover the union through any combination of patterns whose semantic regions cover all members.

Transparent type aliases do not create a separate matching concept. By semantic matching time, canonical type identity governs coverage.

Example conceptually:

```phalcom
type Response = Success | Failure

match response {
    Success::Value(value) => value
    Failure::Error(error) => recover(error)
}
```

Exhaustiveness is proved over the canonical union domain, not alias spelling.

---

# 26. Exact-Case Types

A value may be statically known to an exact variant case.

For example, if flow analysis proves:

```text
x : exact Result::Ok<Int,E>
```

then matching it against `Result::Err` is impossible.

An exact-case domain can therefore collapse a multi-constructor enum to a single reachable constructor.

Exact-case information is also produced by successful variant observation and may refine a stable scrutinee binding inside the selected branch.

---

# 27. GADT Matching

GADT matching is one of Phalcom's most powerful pattern capabilities.

Consider:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

A generic evaluator can be written:

```phalcom
class Eval {
    @class
    eval<T>(_ value: Expr<T>) -> T {
        match value {
            Expr::Int(x) => x
            Expr::Bool(x) => x
        }
    }
}
```

In the first branch, observing `Expr::Int` proves:

```text
T = Int
```

so `x : Int` is a valid result of the generic method returning `T`.

In the second branch:

```text
T = Bool
```

so `x : Bool` is likewise valid as `T` inside that branch.

GADT elimination therefore cannot be modeled as ordinary subtype filtering.

It is equality-producing proof refinement.

---

# 28. GADT Branch Reachability

Given:

```phalcom
eval(_ value: Expr<Int>) {
    match value {
        Expr::Bool(x) => x
        Expr::Int(x) => x
    }
}
```

the `Bool` case is rejected as impossible.

The analyzer solves constructor result compatibility:

```text
Expr<Bool> ~ Expr<Int>
```

which reduces to:

```text
Bool ~ Int
```

and fails.

For a generic scrutinee:

```text
Expr<T>
```

both constructors remain reachable because each contributes a satisfiable branch-local equality.

---

# 29. Constructor-Local Generics in Patterns

Phalcom supports constructor-local generic parameters:

```phalcom
enum Expr<T> {
    @variant
    Wrap<U>(_ value: U) -> Expr<List<U>>
}
```

When matching `Wrap`, the `U` in that constructor is existential from the eliminator's perspective.

Each observation freshly opens that parameter as a branch-local rigid variable.

Conceptually:

```text
Wrap<U>
```

becomes:

```text
Wrap<α>
```

for a fresh rigid `α` scoped to that case observation.

The branch may learn:

```text
T = List<α>
```

but must not arbitrarily guess `α` to make an incompatible concrete index fit.

---

# 30. Freshness of Local Constructor Generics

Two independent observations of the same constructor-local generic are not the same existential witness.

Conceptually:

```text
first observation:  Wrap<α>
second observation: Wrap<β>
```

with:

```text
α != β
```

unless additional proof establishes a relationship.

These local rigid variables:

- are query-local;
- may appear in branch-local `LocalType`s;
- may participate in local constraints and equalities;
- must not be interned as durable canonical global `TypeId` metadata.

This property is essential for sound GADT elimination.

---

# 31. Higher-Kinded GADT Example

Phalcom's matching model supports higher-kinded recursive GADTs.

```phalcom
enum Expression<F: Type -> Type, T> {
    @variant
    Pure<A>(_ value: A) -> Expression<F, A>

    @variant
    IntLiteral(_ value: Int) -> Expression<F, Int>

    @variant
    BoolLiteral(_ value: Bool) -> Expression<F, Bool>

    @variant
    Add(
        _ left: Expression<F, Int>,
        _ right: Expression<F, Int>
    ) -> Expression<F, Int>

    @variant
    If<A>(
        _ condition: Expression<F, Bool>,
        _ yes: Expression<F, A>,
        _ no: Expression<F, A>
    ) -> Expression<F, A>

    @variant
    Map<A, B>(
        _ source: Expression<F, A>,
        _ transform: (A) -> B
    ) -> Expression<F, B>

    @variant
    FlatMap<A, B>(
        _ source: Expression<F, A>,
        _ next: (A) -> Expression<F, B>
    ) -> Expression<F, B>

    @variant
    Apply<A, B>(
        _ function: Expression<F, (A) -> B>,
        _ argument: Expression<F, A>
    ) -> Expression<F, B>

    @variant
    Lift<A>(_ effect: F<A>) -> Expression<F, A>
}
```

A type-safe evaluator can then match every constructor:

```phalcom
class ExpressionEvaluation {
    @class
    eval<F: Type -> Type, T>(
        _ monad: Monad<F>,
        _ expression: Expression<F, T>
    ) -> F<T> {
        match expression {
            Expression::Pure(value) => monad.pure(value)

            Expression::IntLiteral(value) => monad.pure(value)

            Expression::BoolLiteral(value) => monad.pure(value)

            Expression::Add(left, right) => monad.flatMap(
                ExpressionEvaluation.eval(monad, left),
                |leftValue| {
                    monad.map(
                        ExpressionEvaluation.eval(monad, right),
                        |rightValue| { leftValue + rightValue }
                    )
                }
            )

            Expression::If(condition, yes, no) => monad.flatMap(
                ExpressionEvaluation.eval(monad, condition),
                |conditionValue| {
                    if (conditionValue) {
                        ExpressionEvaluation.eval(monad, yes)
                    } else {
                        ExpressionEvaluation.eval(monad, no)
                    }
                }
            )

            Expression::Map(source, transform) => monad.map(
                ExpressionEvaluation.eval(monad, source),
                transform
            )

            Expression::FlatMap(source, next) => monad.flatMap(
                ExpressionEvaluation.eval(monad, source),
                |value| {
                    ExpressionEvaluation.eval(monad, next.call(value))
                }
            )

            Expression::Apply(function, argument) => monad.flatMap(
                ExpressionEvaluation.eval(monad, function),
                |functionValue| {
                    monad.map(
                        ExpressionEvaluation.eval(monad, argument),
                        |argumentValue| {
                            functionValue.call(argumentValue)
                        }
                    )
                }
            )

            Expression::Lift(effect) => effect
        }
    }
}
```

This example demonstrates simultaneously:

- higher-kinded generic parameters;
- recursive GADTs;
- constructor-local generics;
- result-index refinement;
- typed recursive evaluation;
- callable payloads;
- effect-polymorphic interpretation;
- exhaustive matching without `Dynamic`.

---

# 32. Recursive Pattern Domains

Recursive ADTs/GADTs denote potentially infinite sets of finite values.

Example:

```phalcom
enum Tree<T> {
    @variant Leaf(_ value: T)
    @variant Node(_ left: Tree<T>, _ right: Tree<T>)
}
```

The domain is conceptually recursive:

```text
Tree<T>
    = Leaf(T)
    | Node(Tree<T>, Tree<T>)
```

The compiler must **not** expand this as:

```text
Leaf(T)
| Node(
    Leaf(T) | Node(...),
    Leaf(T) | Node(...)
  )
```

That representation is infinite.

Instead, the head-constructor decomposition is finite:

```text
Leaf(_)
Node(_, _)
```

and payloads remain universal typed subjects until a nested source pattern inspects them.

---

# 33. Demand-Driven Decomposition

This is the central termination law.

> **A payload is decomposed only when the source pattern scrutinizes that payload.**

For:

```phalcom
match tree {
    Tree::Leaf(value) => leaf(value)
    Tree::Node(left, right) => node(left, right)
}
```

both `left` and `right` are bindings.

Coverage therefore treats them as wildcards and never recursively decomposes `Tree<T>` inside `Node`.

For:

```phalcom
match tree {
    Tree::Node(Tree::Leaf(value), _) => leftLeaf(value)
    _ => other()
}
```

only the first `Node` payload is decomposed one additional level.

The second remains untouched.

Therefore:

```text
analysis recursion depth <= finite source pattern depth
```

rather than:

```text
analysis recursion depth = recursive datatype depth
```

which may be unbounded.

---

# 34. Why Recursive GADT Indices Require This Architecture

The `Apply` constructor from `Expression` has type:

```text
Apply<A,B>(
    Expression<F, A -> B>,
    Expression<F, A>
) -> Expression<F,B>
```

If an exhaustiveness checker eagerly expands payload types, starting from:

```text
Expression<F,T>
```

it can derive:

```text
Expression<F, A -> T>
Expression<F, A2 -> (A -> T)>
Expression<F, A3 -> (A2 -> (A -> T))>
...
```

Every type is structurally distinct.

No exact-`TypeId` cycle guard can make this a finite universe.

Demand-driven matching solves the problem semantically:

```phalcom
Expression::Apply(function, argument)
```

has two binding child patterns, so after the outer `Apply` constructor is selected, both child columns are wildcards and decomposition stops immediately.

---

# 35. Pattern Matrix Model

Exhaustiveness and usefulness should be implemented as a symbolic pattern-matrix algorithm.

Each row is one arm pattern and its action.

Each column is one currently scrutinized subterm.

For:

```phalcom
match value {
    Some(Ok(x)) => a(x)
    Some(Err(e)) => b(e)
    None() => c()
}
```

conceptually:

```text
rows
--------------------------------
Some(Ok(_))
Some(Err(_))
None()
```

The algorithm specializes only the constructor positions needed to distinguish the rows.

## 35.1 Constructor specialization

Given a subject of closed ADT type, the engine obtains its feasible outer constructors.

Selecting a constructor replaces the selected column with that constructor's payload columns.

Bindings and wildcards expand to wildcard payload columns.

## 35.2 Base case

When the first applicable row consists entirely of wildcards for the current columns, that row covers every remaining value along that specialization path.

## 35.3 Usefulness

A pattern is useful if specialization can construct at least one witness value admitted by the pattern but not by previous rows.

## 35.4 Exhaustiveness

After all arms, ask whether an all-wildcard row is still useful.

If no:

```text
match is exhaustive
```

If yes:

```text
match is non-exhaustive
```

and the usefulness search can construct a missing-case witness.

---

# 36. One Constructor-Decomposition Authority

Pattern resolution and coverage analysis must not maintain separate GADT-constructor semantics.

One internal semantic operation should answer:

```text
For this subject and current proof environment,
which constructors are reachable,
what payload subjects do they expose,
and what proof does selecting each constructor establish?
```

Conceptually:

```rust
DomainDecomposition::Closed {
    constructors: [
        ConstructorCase {
            variant,
            fields,
            exact_case,
            proof,
            case_instantiation,
        },
        ...
    ]
}
```

For GADT constructors, the operation must perform:

```text
canonical declaration specialization
    -> solve GADT result equalities
    -> discard contradictory constructors
    -> freshly open constructor-local generics
    -> solve local case proof
    -> specialize payload subjects
    -> publish constructor proof delta
```

Pattern resolution and exhaustiveness must consume the same authority.

---

# 37. Branch-Local Proof Environments

A successful GADT pattern may establish equalities that hold only in that branch.

Conceptually:

```rust
BranchProofEnvironment {
    canonical bindings,
    canonical equalities,
    local rigid bindings,
    local equalities,
}
```

Example:

```phalcom
match expr {
    Expr::Int(value) => ...
    Expr::Bool(value) => ...
}
```

The first branch may know:

```text
T = Int
```

The second may know:

```text
T = Bool
```

Neither proof leaks into the sibling branch or post-match environment.

---

# 38. Pattern Bindings and GADT Refinement

Suppose:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

then:

```phalcom
eval<T>(_ expr: Expr<T>) -> T {
    match expr {
        Expr::Int(x) => x
        Expr::Bool(x) => x
    }
}
```

works because the branch-local binding knowledge is interpreted together with the constructor proof.

Inside `Expr::Int`:

```text
x : Int
T = Int
therefore x is valid where T is required
```

Inside `Expr::Bool`:

```text
x : Bool
T = Bool
therefore x is valid where T is required
```

---

# 39. Flow Refinement

When the scrutinee is a stable binding, successful exact variant matching may refine flow knowledge inside the branch.

Conceptually:

```phalcom
match result {
    Result::Ok(value) => {
        // result is known here to be the exact Ok case
        use(value)
    }
    Result::Err(error) => {
        // result is known here to be the exact Err case
        recover(error)
    }
}
```

Coverage state and ordinary flow state remain distinct abstractions:

```text
coverage
    which values remain possible for match proof

flow
    facts about program bindings and expressions
```

A nested residual pattern domain must not be widened merely to fit ordinary flow predicates.

---

# 40. Match Result Typing

A `match` expression's result type is the semantic join of reachable normal-completing branch result types.

Example:

```phalcom
const value = match option {
    Option::Some(x) => x
    Option::None() => 0
}
```

If both branches produce `Int`, the match produces `Int`.

If branches produce different compatible types, the ordinary type-knowledge join rules apply.

Impossible and redundant branches do not contribute reachable normal result values.

If no reachable branch can complete normally:

```text
match result type = Never
```

---

# 41. Missing-Case Witnesses

A non-exhaustive match should report concrete structured witness patterns when possible.

Example:

```phalcom
match optionResult {
    Option::Some(Result::Ok(value)) => use(value)
    Option::None() => absent()
}
```

for:

```text
Option<Result<T,E>>
```

should be able to produce:

```text
Option::Some(Result::Err(_))
```

rather than only:

```text
some case is missing
```

Witness generation must use the same constructor feasibility and GADT proof rules as exhaustiveness.

A witness must never describe an impossible or uninhabited case.

Witness generation should be bounded for diagnostics.

---

# 42. Inhabitation

Closed constructor knowledge is not identical to inhabitation.

Consider:

```phalcom
enum Loop {
    @variant Next(_ value: Loop)
}
```

If runtime values must be finite, there is no finite inhabitant of `Loop`.

Coverage analysis should therefore eventually distinguish:

```text
Inhabited
Uninhabited
Unknown
```

through a memoized/fixpoint inhabitation analysis rather than recursive value-space expansion.

For GADTs, constructor reachability and index constraints participate in inhabitation.

Unknown inhabitation must be treated conservatively and must not manufacture an exhaustiveness proof.

---

# 43. Recursive Exhaustiveness Example

```phalcom
enum Tree<T> {
    @variant Leaf(_ value: T)
    @variant Node(_ left: Tree<T>, _ right: Tree<T>)
}
```

This match is exhaustive:

```phalcom
match tree {
    Tree::Leaf(value) => leaf(value)
    Tree::Node(left, right) => node(left, right)
}
```

The proof requires only:

```text
reachable outer constructors(Tree<T>)
    = Leaf | Node
```

It does not require enumerating the recursively inhabited payload values.

A deeper match:

```phalcom
match tree {
    Tree::Node(Tree::Leaf(x), Tree::Leaf(y)) => pair(x, y)
    Tree::Node(_, _) => branch()
    Tree::Leaf(value) => leaf(value)
}
```

remains finite because recursive decomposition follows finite source pattern depth.

---

# 44. Generic Recursive Transformation Example

```phalcom
enum ListTree<T> {
    @variant Item(_ value: T)
    @variant Branch(_ children: List<ListTree<T>>)
}
```

A matcher can inspect only the tree shape it needs:

```phalcom
match tree {
    ListTree::Item(value) => visit(value)
    ListTree::Branch([]) => emptyBranch()
    ListTree::Branch([first, *rest]) => {
        visit(first)
        visitAll(rest)
    }
}
```

The recursive `ListTree<T>` element type inside `children` is not unfolded unless nested element patterns demand it.

---

# 45. Protocol/State-Machine Example

Pattern matching can encode state transitions clearly.

```phalcom
enum Connection {
    @variant Disconnected()
    @variant Connecting(_ attempt: Int)
    @variant Connected(_ socket: Socket)
    @variant Failed(_ error: Error)
}
```

```phalcom
next(_ state: Connection, _ event: Event) -> Connection {
    match (state, event) {
        (Connection::Disconnected(), Event::Connect()) =>
            Connection::Connecting(1)

        (Connection::Connecting(attempt), Event::Connected(socket)) =>
            Connection::Connected(socket)

        (Connection::Connecting(attempt), Event::Failed(error)) =>
            Connection::Failed(error)

        (Connection::Connected(socket), Event::Disconnect()) => {
            socket.close()
            Connection::Disconnected()
        }

        (state, _) => state
    }
}
```

This demonstrates tuple composition, exact constructors, payload binding, and a final irrefutable fallback.

---

# 46. Typed Compiler AST Example

```phalcom
enum Node<T> {
    @variant Int(_ value: Int) -> Node<Int>
    @variant Bool(_ value: Bool) -> Node<Bool>
    @variant Pair<A, B>(
        _ left: Node<A>,
        _ right: Node<B>
    ) -> Node<(A, B)>
}
```

```phalcom
evaluate<T>(_ node: Node<T>) -> T {
    match node {
        Node::Int(value) => value
        Node::Bool(value) => value
        Node::Pair(left, right) => (
            evaluate(left),
            evaluate(right)
        )
    }
}
```

The `Pair` constructor establishes the result index:

```text
T = (A, B)
```

and the recursive calls retain their own specialized result indices.

---

# 47. Variant-Family Routing Example

Suppose an API models command overloads as related variants:

```phalcom
enum Command {
    @variant Send(_ message: String)
    @variant Send(_ message: String, to: Address)
    @variant Send(_ message: String, to: Address, priority: Int)
    @variant Stop()
}
```

A broad family handler:

```phalcom
match command {
    Command::Send* => enqueue(command)
    Command::Stop() => stop()
}
```

A selector-pattern handler:

```phalcom
match command {
    Command::Send(message, ...) => audit(message)
    Command::Stop() => stop()
}
```

A suffix-constrained family pattern can select variants containing a required labeled trailing slot if such a selector family exists:

```phalcom
Command::Send(..., priority: priority)
```

This demonstrates a feature uncommon in mainstream ADT systems: the same structural selector/family model used for lookup can be used statically for pattern families.

---

# 48. Either/Result Example

```phalcom
enum Either<L, R> {
    @variant Left(_ value: L)
    @variant Right(_ value: R)
}
```

```phalcom
fold<L, R, T>(
    _ value: Either<L, R>,
    _ onLeft: (L) -> T,
    _ onRight: (R) -> T
) -> T {
    match value {
        Either::Left(left) => onLeft.call(left)
        Either::Right(right) => onRight.call(right)
    }
}
```

Nested use:

```phalcom
match value {
    Either::Left(Result::Err(error)) => recover(error)
    Either::Left(Result::Ok(value)) => useLeft(value)
    Either::Right(value) => useRight(value)
}
```

---

# 49. Data Validation Pipeline Example

```phalcom
enum Validation<T, E> {
    @variant Valid(_ value: T)
    @variant Invalid(_ errors: List<E>)
}
```

```phalcom
match validation {
    Validation::Valid(#{name: name, age: age}) =>
        persist(name, age)

    Validation::Invalid([first, *rest]) =>
        report(first, rest)

    Validation::Invalid([]) =>
        internalInvariantFailure()
}
```

This combines ADT, record, and list patterns.

---

# 50. Recursive Expression Simplifier Example

```phalcom
simplify(_ expression: Expression<F, Int>) -> Expression<F, Int> {
    match expression {
        Expression::Add(
            Expression::IntLiteral(left),
            Expression::IntLiteral(right)
        ) => Expression::IntLiteral(left + right)

        Expression::Add(left, right) =>
            Expression::Add(
                simplify(left),
                simplify(right)
            )

        Expression::IntLiteral(value) =>
            Expression::IntLiteral(value)

        other => other
    }
}
```

The first arm uses deep matching for one optimization. The second matches the broader `Add` case. Ordered usefulness correctly permits both because the second still covers values not handled by the first.

---

# 51. Exhaustiveness with GADT Index Restrictions

Consider:

```phalcom
enum Query<T> {
    @variant Count() -> Query<Int>
    @variant Name() -> Query<String>
    @variant IsReady() -> Query<Bool>
}
```

For:

```phalcom
run(_ query: Query<Int>) -> Int {
    match query {
        Query::Count() => executeCount()
    }
}
```

this is exhaustive for `Query<Int>` because the other constructors are impossible under the concrete index.

The checker must not demand syntactic coverage of constructors already refuted by GADT equalities.

---

# 52. Generic GADT Exhaustiveness

For:

```phalcom
run<T>(_ query: Query<T>) -> T {
    match query {
        Query::Count() => executeCount()
        Query::Name() => executeName()
        Query::IsReady() => executeReady()
    }
}
```

all three constructors are satisfiable under different branch-local proofs:

```text
T = Int
T = String
T = Bool
```

and therefore all must participate in generic exhaustiveness.

---

# 53. Exact Case + Nested Matching

Suppose an earlier branch or API gives an exact `Some` case:

```text
value : exact Option::Some<Result<Int,E>>
```

then:

```phalcom
match value {
    Option::Some(Result::Ok(x)) => x
    Option::Some(Result::Err(error)) => recover(error)
}
```

can be exhaustive without an `Option::None()` arm because `None` is impossible in the exact-case domain.

---

# 54. Pattern Families and GADT Reachability

Family patterns must be filtered by GADT feasibility.

If one family contains several overloaded constructors but only some can produce the current scrutinee specialization, only those reachable exact `VariantId`s participate.

The family itself is not blindly treated as every declaration with the same base name.

This preserves the invariant:

```text
family resolution
    ∩ GADT reachability
    = actual pattern candidate set
```

---

# 55. Pattern Matching and Associated Lookup

The language uses one semantic identity system for:

- variant declaration;
- associated lookup;
- family lookup;
- construction;
- pattern matching;
- runtime lowering.

This prevents divergent meanings such as:

```text
expression position says one selector
pattern position guesses another
runtime lowering tests a third spelling
```

A resolved pattern candidate contains exact declaration-backed identity. That identity is the authority consumed by lowering.

---

# 56. Trusted Runtime Operations

Primitive pattern matching must lower to trusted runtime operations such as:

```text
IsVariant
GetVariantPayload
```

or equivalent compiler/runtime primitives.

The following are not acceptable as the semantic basis of exact variant matching:

```text
user-overridable getter calls
runtime string-name comparisons
ordinary open-class .class lookup
re-running associated lookup during lowering
```

Pattern components are compiler-authoritative projections from the variant representation.

---

# 57. Test-Then-Commit Execution

Runtime matching should conceptually follow:

```text
1. test candidate structure
2. extract candidate payloads into temporary locations
3. test nested patterns
4. only after complete pattern success, commit branch bindings
5. execute branch
```

This matters especially for:

- or-patterns;
- nested failures;
- future guards;
- multiple payload bindings.

A failed pattern must not leak partially initialized branch bindings.

---

# 58. Semantic Match Product

The semantic layer should publish stable products conceptually containing:

```rust
MatchResolution {
    expression,
    scrutinee,
    arms,
    result,
    exhaustiveness,
}
```

Each arm contains information equivalent to:

```rust
MatchArmResolution {
    arm_index,
    pattern,
    bindings,
    proof,
    usefulness,
    branch_result,
    coverage_summary,
}
```

A resolved variant candidate includes:

```rust
ResolvedVariantCandidate {
    variant: VariantId,
    exact_case,
    fields,
    proof,
    case_instantiation,
}
```

This product is part of the semantic-to-lowering authority boundary.

---

# 59. Pattern Resolution vs Coverage

The implementation must distinguish two questions.

## 59.1 Pattern resolution

Answers:

```text
What does this source pattern mean?
Which variant/family identity does it denote?
Which fields correspond to its arguments?
Which bindings does it introduce?
What GADT/local generic evidence comes from observing it?
```

## 59.2 Coverage

Answers:

```text
Can this pattern match any reachable value?
Does it add any value beyond earlier arms?
What values remain uncovered?
Is the match total?
```

They may share constructor-decomposition services, but they should not be collapsed into one unstructured subsystem.

---

# 60. Closed Universal Payload Subjects

A recursive payload must be representable as:

```text
all inhabitants of T
```

without immediately decomposing `T`.

This semantic notion differs from an open/unknown domain.

For example:

```text
AnyClosed(Expression<F,Int>)
```

means:

```text
this field can contain any legal Expression<F,Int>,
and the compiler knows how to decompose it later if required
```

while:

```text
Open(Object)
```

means:

```text
the compiler cannot enumerate every possible structural alternative
```

The implementation may use different internal names, but the semantic distinction is normative.

---

# 61. Pattern Matching Must Not Eagerly Materialize Products

For a constructor:

```text
C(A, B, C)
```

coverage must not eagerly build:

```text
D(A) × D(B) × D(C)
```

when the source pattern is:

```phalcom
C(_, _, _)
```

All three payloads are already fully covered.

Product decomposition is required only where nested patterns create distinctions.

---

# 62. Performance and Termination Properties

The following are semantic engineering requirements, not optional optimizations.

## 62.1 Finite-pattern termination

Every finite pattern matrix over semantically well-formed types must terminate or return an explicit blocked/budget-exceeded result.

## 62.2 No recursive type-universe construction

Recursive payload types must not be recursively expanded simply to build an initial scrutinee space.

## 62.3 No exact-TypeId termination dependence

Termination must not depend on eventually seeing the exact same canonical `TypeId` again.

## 62.4 Query-local memoization

Constructor decomposition, inhabitation, and usefulness states should be memoized where appropriate.

## 62.5 Structural sharing

If symbolic residual trees remain in diagnostics/tooling, their implementation should prefer DAG/arena/interned sharing over repeated deep cloning.

## 62.6 Bounded diagnostics

Witness generation and coverage summaries must have explicit bounds.

## 62.7 Analysis budget

Pathological pattern matrices may consume an explicit semantic-analysis budget.

Budget exhaustion must not produce a false proof of exhaustiveness.

---

# 63. Exhaustiveness Failure Modes

The public result model should distinguish at least:

```text
Proven
Missing(witnesses)
Blocked(reason)
```

`Blocked` is appropriate when the analyzer cannot complete a proof due to a formal semantic limitation or resource budget.

The rule is:

> **Failure to prove exhaustiveness is never equivalent to proof of exhaustiveness.**

---

# 64. Diagnostics

A high-quality pattern system should diagnose the semantic reason, not merely report generic failure.

Required diagnostic categories include:

- non-exhaustive match;
- redundant arm;
- impossible arm;
- redundant or-pattern alternative;
- or-pattern binding mismatch;
- duplicate binding in one pattern alternative;
- variant owner ambiguity;
- unknown variant/family;
- selector-shape mismatch;
- variant field mismatch;
- incompatible GADT constructor;
- local generic constraint contradiction;
- invalid tuple/list/record/map pattern shape;
- blocked exhaustiveness proof;
- internal semantic incident only for invariant violations.

Example:

```text
error: impossible match arm

Expr::Bool cannot match Expr<Int>

constructor result:
    Expr<Bool>

scrutinee type:
    Expr<Int>

contradictory equality:
    Bool = Int
```

Example:

```text
error: non-exhaustive match

reachable values remain uncovered

missing pattern:
    Option::Some(Result::Err(_))
```

---

# 65. Usefulness Laws

## PAT-USE-01 — Original-domain impossibility

A pattern is `Impossible` iff its region has empty intersection with the original reachable scrutinee domain.

## PAT-USE-02 — Residual redundancy

A possible pattern is `Redundant` iff it adds no value beyond previous relevant rows.

## PAT-USE-03 — Useful otherwise

A possible non-redundant pattern is `Useful`.

## PAT-USE-04 — Ordered semantics

Usefulness respects source order.

## PAT-USE-05 — Or-alternative ordering

Alternatives inside one or-pattern are likewise checked for internal redundancy.

---

# 66. Exhaustiveness Laws

## PAT-EXH-01 — Closed-domain totality

Every accepted match over a closed reachable domain covers every inhabited value.

## PAT-EXH-02 — GADT filtering

Constructors refuted by branch-result equalities are excluded from required coverage.

## PAT-EXH-03 — Open-domain fallback

Open domains require an irrefutable pattern unless some stronger static proof establishes totality.

## PAT-EXH-04 — No false proof

Unknown, blocked, or budget-exceeded states cannot produce `Proven`.

## PAT-EXH-05 — Witness soundness

Every reported missing witness denotes a reachable value region.

---

# 67. Recursive Coverage Laws

## PAT-REC-01 — Demand-driven decomposition

Payload domains are decomposed only when a nested source pattern or finite proof query requires their structure.

## PAT-REC-02 — Closedness preservation

A closed payload remains semantically closed even when represented lazily/unopened.

## PAT-REC-03 — Pattern-depth bound

Type recursion alone does not increase coverage decomposition depth.

## PAT-REC-04 — No eager recursive universe

No semantic phase may attempt to construct the complete recursive value universe of a recursive ADT/GADT.

## PAT-REC-05 — Explicit nested recursion allowed

The analyzer must permit arbitrarily deep finite nested source patterns subject to normal analysis budgets.

---

# 68. GADT Laws

## PAT-GADT-01 — Equality-producing observation

GADT constructor selection produces equalities from the constructor result type and scrutinee specialization.

## PAT-GADT-02 — Contradiction eliminates constructor

An unsatisfiable equality makes the constructor impossible for that subject.

## PAT-GADT-03 — Fresh local opening

Each elimination of a constructor-local generic creates fresh rigid variables.

## PAT-GADT-04 — Local proof scope

Local constructor proof facts are branch-local.

## PAT-GADT-05 — No guessing rigids

Rigid constructor-local variables cannot be assigned arbitrary concrete types merely to make an otherwise incompatible pattern reachable.

## PAT-GADT-06 — No canonical leakage

Query-local rigid variables never become durable canonical type identities.

## PAT-GADT-07 — Shared authority

Pattern resolution and coverage use the same constructor feasibility and local-opening semantics.

---

# 69. Binding Laws

## PAT-BIND-01 — Bare names bind

A bare name in pattern position creates a new binding.

## PAT-BIND-02 — Wildcard does not bind

`_` never introduces a binding.

## PAT-BIND-03 — Linear alternatives

A name may not be independently rebound multiple times in one pattern alternative.

## PAT-BIND-04 — Or-pattern coherence

Every successful alternative of an or-pattern introduces the same binding-name set.

## PAT-BIND-05 — Branch locality

Pattern bindings exist only in the selected branch.

## PAT-BIND-06 — Test before commit

Failed nested alternatives do not leak partial bindings.

---

# 70. Variant and Family Laws

## PAT-VAR-01 — Semantic identity

Exact variant matching is defined by `VariantId`, not runtime strings.

## PAT-VAR-02 — Selector shape matters

Singleton, nullary callable, positional, and labeled variants with the same base name remain distinct exact patterns.

## PAT-VAR-03 — Family is declaration-backed

A family pattern denotes a declaration-backed `VariantFamilyId` and its reachable members.

## PAT-VAR-04 — Family reachability is filtered

GADT-incompatible family members do not participate in the pattern region.

## PAT-VAR-05 — Contextual shorthand is static

Omitted owners are resolved from static pattern context, not runtime lookup.

---

# 71. Structural Pattern Laws

## PAT-STR-01 — Tuple arity

Tuple patterns must respect the semantic tuple shape.

## PAT-STR-02 — List rest is sequence-valued

`*rest` binds the remaining sequence, not one element.

## PAT-STR-03 — Record openness preserved

Record patterns must not silently close an open record row.

## PAT-STR-04 — Map key tests are refutable

Required map keys do not normally imply total coverage of the map domain.

## PAT-STR-05 — Nested fields are demand-driven

Only fields named by nested source patterns require recursive structural decomposition.

---

# 72. Lowering Laws

## PAT-LOWER-01 — Semantic authority

Lowering consumes resolved pattern identities and must not repeat source-level name/family resolution.

## PAT-LOWER-02 — Trusted identity tests

Exact variant tests use compiler/runtime-authoritative constructor identity.

## PAT-LOWER-03 — Trusted payload projection

Payload extraction uses declared field representation, not overridable getters.

## PAT-LOWER-04 — Binding atomicity

Bindings become visible only after the structural pattern succeeds.

## PAT-LOWER-05 — Source order preserved

Generated decision trees/DAGs preserve first-match semantics.

---

# 73. Decision-Tree Compilation

Semantic usefulness/exhaustiveness and executable lowering are related but distinct.

The semantic matrix proves:

```text
which rows are possible
which are useful
whether coverage is total
what each constructor/payload means
```

The compiler may then compile the resolved matrix into an optimized decision tree or DAG.

A typical strategy:

```text
choose discriminating column
    -> test constructor once
    -> project required fields
    -> branch into specialized rows
    -> share continuation/join points where profitable
```

Compiler optimization must preserve:

- source-order semantics;
- binding atomicity;
- GADT-resolved constructor identities;
- future guard fallthrough semantics if guards are added.

---

# 74. Optimization Opportunities

The semantic architecture enables aggressive lowering optimizations.

## 74.1 Single constructor test

Multiple arms sharing the same outer variant can share one runtime tag test.

## 74.2 Projection elimination

Wildcard or ignored payloads need not be projected.

Example:

```phalcom
Expression::Add(_, _) => ...
```

may require only an `Add` identity test if no payload value is used.

## 74.3 Nested test sharing

Patterns such as:

```phalcom
Some(Ok(_))
Some(Err(_))
None()
```

can test `Some` once, project its payload once, then branch on the nested `Result` tag.

## 74.4 Exact-case optimization

If static flow already knows an exact case, the corresponding runtime constructor test may be proven redundant and removed where defensive execution policy allows.

## 74.5 Family compaction

A family pattern over several variants may lower to compact ordinal/range/set tests where runtime variant metadata permits.

---

# 75. Interaction with Gradual Typing

Phalcom's type system permits dynamic/unknown boundaries.

Pattern matching must preserve the distinction between:

```text
Known closed type
Known open type
Unknown analysis result
Dynamic runtime boundary
```

A `Dynamic` value cannot be treated as though its static constructor universe were closed merely because some known variant patterns appear in the match.

An irrefutable fallback is therefore normally required.

Pattern matching also does not silently convert reflective generic type annotations into arbitrary runtime generic validation unless the runtime type-reification model explicitly guarantees such checks.

---

# 76. Interaction with Higher-Kinded Types

A constructor payload may contain higher-kinded applications:

```text
F<A>
```

Coverage treats this as a typed payload subject.

Unless `F<A>` itself has a statically closed pattern decomposition and a nested pattern requests that structure, the payload remains uninspected.

This is what allows `Expression::Lift(effect)` to be matched without trying to enumerate values of an arbitrary effect constructor `F`.

---

# 77. Interaction with Callable Types

Callable payloads are ordinary typed payload values.

For:

```phalcom
Expression::Map(source, transform)
```

where:

```text
transform : (A) -> B
```

a binding pattern simply binds the function.

Coverage does not attempt to enumerate callable inhabitants.

Likewise:

```text
(A) -> Expression<F,B>
```

inside `FlatMap` remains opaque/universal unless a future pattern feature explicitly gives callable values a closed decomposition—which ordinary function values do not have.

---

# 78. Pattern Matching as an API Design Tool

Pattern matching makes ADT APIs expressive without requiring every consumer operation to be encoded as a virtual method.

Example:

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Err(_ error: E)
}
```

Consumers can write domain-specific elimination directly:

```phalcom
match result {
    Result::Ok(value) => commit(value)
    Result::Err(error) => rollback(error)
}
```

while `Result` can still expose convenience methods such as `map`, `flatMap`, `isOk`, and `getOrElse`.

Pattern matching and method APIs complement each other.

---

# 79. Pattern Matching as a Typed DSL Foundation

The `Expression<F,T>` example illustrates an important design capability: Phalcom can encode typed embedded languages whose AST index guarantees semantic result types.

Additional examples include:

```phalcom
enum SqlExpr<T> {
    @variant IntColumn(_ name: String) -> SqlExpr<Int>
    @variant TextColumn(_ name: String) -> SqlExpr<String>
    @variant Equals<T>(_ left: SqlExpr<T>, _ right: SqlExpr<T>) -> SqlExpr<Bool>
    @variant Add(_ left: SqlExpr<Int>, _ right: SqlExpr<Int>) -> SqlExpr<Int>
}
```

Then:

```phalcom
compile<T>(_ expr: SqlExpr<T>) -> SqlFragment<T> {
    match expr {
        SqlExpr::IntColumn(name) => compileIntColumn(name)
        SqlExpr::TextColumn(name) => compileTextColumn(name)
        SqlExpr::Equals(left, right) => compileEquals(left, right)
        SqlExpr::Add(left, right) => compileAdd(left, right)
    }
}
```

The result type is preserved through GADT refinement.

---

# 80. Typed Command Interpreter Example

```phalcom
enum Command<T> {
    @variant ReadLine() -> Command<String>
    @variant RandomInt(_ upper: Int) -> Command<Int>
    @variant Print(_ text: String) -> Command<()>
}
```

```phalcom
execute<T>(_ command: Command<T>) -> T {
    match command {
        Command::ReadLine() => Console.readLine()
        Command::RandomInt(upper) => Random.nextInt(upper)
        Command::Print(text) => {
            Console.print(text)
            ()
        }
    }
}
```

Each branch's result type is justified by the constructor index.

---

# 81. Typed Heterogeneous Routing Example

```phalcom
enum Request<T> {
    @variant UserById(_ id: Int) -> Request<User>
    @variant Users() -> Request<List<User>>
    @variant Health() -> Request<Bool>
}
```

```phalcom
handle<T>(_ request: Request<T>) -> T {
    match request {
        Request::UserById(id) => repository.user(id)
        Request::Users() => repository.users()
        Request::Health() => true
    }
}
```

No `Dynamic` return is required.

This is a direct demonstration of GADT-based API routing.

---

# 82. Type-Safe Serialization Example

```phalcom
enum Format<T> {
    @variant IntFormat() -> Format<Int>
    @variant StringFormat() -> Format<String>
    @variant PairFormat<A, B>(
        _ left: Format<A>,
        _ right: Format<B>
    ) -> Format<(A, B)>
}
```

```phalcom
encode<T>(_ format: Format<T>, _ value: T) -> Bytes {
    match format {
        Format::IntFormat() => encodeInt(value)
        Format::StringFormat() => encodeString(value)
        Format::PairFormat(left, right) => {
            const (a, b) = value
            encode(left, a) + encode(right, b)
        }
    }
}
```

The `PairFormat<A,B>` case refines:

```text
T = (A, B)
```

allowing tuple decomposition of `value` with the appropriate component types.

---

# 83. Exhaustive Error Taxonomy Example

```phalcom
enum ParseError {
    @variant UnexpectedToken(_ token: Token)
    @variant UnexpectedEof()
    @variant InvalidEscape(_ value: String)
    @variant InvalidNumber(_ text: String)
}
```

```phalcom
describe(_ error: ParseError) -> String {
    match error {
        ParseError::UnexpectedToken(token) =>
            "unexpected token ${token}"

        ParseError::UnexpectedEof() =>
            "unexpected end of input"

        ParseError::InvalidEscape(value) =>
            "invalid escape ${value}"

        ParseError::InvalidNumber(text) =>
            "invalid number ${text}"
    }
}
```

Adding a new error constructor forces every exhaustive consumer to be revisited by the compiler.

---

# 84. Reserved Extension: Guards

Guards are not treated here as already-ratified current syntax. If introduced, they must obey the following semantics.

Conceptually:

```phalcom
match value {
    Some(number) <guard-keyword> number > 0 => positive(number)
    Some(number) => nonPositive(number)
    None() => absent()
}
```

Required properties:

1. structural pattern matching occurs before guard evaluation;
2. pattern bindings are visible to the guard;
3. guard failure continues to later arms;
4. guarded arms do **not** by themselves contribute total static coverage, because arbitrary boolean predicates are not generally statically complete;
5. a later structurally identical unguarded arm remains useful;
6. runtime lowering must preserve textual fallthrough semantics.

The exact keyword (`if`, `when`, or another spelling) remains a separate surface decision.

---

# 85. Reserved Extension: Literal Patterns

If literal patterns are introduced, examples may include:

```phalcom
match flag {
    true => yes()
    false => no()
}
```

or:

```phalcom
match count {
    0 => empty()
    _ => nonEmpty()
}
```

Literal coverage must integrate with the same usefulness matrix rather than adding a parallel switch-analysis subsystem.

For infinite literal domains such as `Int`, a finite list of literals is not exhaustive without an irrefutable remainder.

---

# 86. Reserved Extension: Pinned Values

A future pinned-value pattern would be required if Phalcom wants to match against an existing binding rather than introduce a new one.

The crucial design rule is:

```text
bare name => binds
explicit pin syntax => compares against existing value
```

This avoids local ambiguity between binding and equality matching.

---

# 87. Reserved Extension: Type-Test Patterns

Future nominal type patterns may support open-hierarchy matching such as:

```phalcom
match value {
    is Int as number => useInt(number)
    is String as text => useString(text)
    _ => fallback(value)
}
```

Such patterns are ordinarily not sufficient for exhaustiveness over `Object` because the class universe is open.

Type-test patterns must remain semantically distinct from exact ADT variant patterns.

---

# 88. Reserved Extension: As/Alias Patterns

A future alias pattern could bind the whole value while also destructuring it, conceptually:

```text
whole @ Some(value)
```

or another Phalcom-specific spelling.

Any such feature should be implemented as binding metadata over the same structural pattern, not as a second coverage concept.

---

# 89. Reserved Extension: Active/View Patterns

User-defined extractors or active patterns are much harder to integrate with strict exhaustiveness because arbitrary user code does not expose a statically closed mathematical region.

If Phalcom later adds them, they should normally be treated as refutable opaque tests unless the extractor carries compiler-authoritative proof metadata.

They must not weaken the soundness of closed ADT/GADT exhaustiveness.

---

# 90. Interaction with `if let` and `while let`

Refutable constructs such as `if let` and `while let` should share the same formal pattern resolver and constructor identity model.

They differ from `match` primarily in control-flow obligation:

```text
match
    requires total coverage

if-let / while-let
    deliberately accept pattern failure
```

The compiler should not maintain an unrelated legacy pattern system for these constructs.

---

# 91. Static and Runtime Authority Boundary

The semantic analyzer owns:

```text
source pattern resolution
owner/context resolution
variant/family candidate resolution
field identity resolution
GADT feasibility
constructor-local generic opening
branch proof construction
binding types
usefulness
exhaustiveness
witnesses
```

Compiler/runtime lowering owns:

```text
runtime constructor tests
payload extraction
temporary storage
branching
binding commitment
decision-tree/DAG optimization
```

The runtime compiler must not re-solve GADT constraints or associated family lookup.

---

# 92. Semantic Stability and Incrementality

Pattern semantic products participate in incremental analysis.

Changes to any of the following may invalidate affected match analysis:

- enum variant set;
- variant selector shape;
- payload field types;
- result type template;
- constructor-local generic signature;
- GADT case environment;
- associated family membership;
- relevant type aliases or union forms;
- source pattern structure.

Implementation fingerprints should reflect semantic identity and proof-relevant structure, not transient query-local rigid IDs.

---

# 93. Conformance Testing Requirements

A complete conformance suite should cover at least the following categories.

## 93.1 Syntax

- wildcard;
- name binding;
- exact singleton;
- exact nullary constructor;
- exact positional constructor;
- labeled constructor;
- contextual shorthand;
- whole family;
- callable family pattern;
- or-pattern;
- tuple;
- list/rest;
- record;
- map;
- arbitrary nesting.

## 93.2 Resolution

- unique owner resolution;
- ambiguous contextual owner rejection;
- exact `VariantId` identity;
- selector-shape mismatch;
- field identity mapping;
- family candidate enumeration.

## 93.3 Usefulness

- useful arm;
- impossible arm;
- redundant arm;
- partial overlap;
- redundant or-alternative.

## 93.4 Exhaustiveness

- closed enum complete;
- closed enum missing case;
- exact-case complete;
- union complete;
- nested complete;
- open-domain fallback;
- structured witness generation.

## 93.5 GADT

- generic index refinement;
- concrete incompatible case rejection;
- exact-case refinement;
- multi-parameter GADT;
- nested GADT proof;
- constructor-local generic opening;
- local generic constraints;
- independent rigid freshness;
- no rigid guessing;
- family pattern filtered by index reachability.

## 93.6 Recursive coverage

- ordinary recursive ADT;
- binary recursive payloads;
- nested recursive source patterns;
- recursive GADT;
- index-growing recursion such as `Apply`;
- higher-kinded recursion;
- callable recursive payload;
- uninhabited recursive family;
- bounded type-store growth;
- bounded constructor-decomposition counts.

## 93.7 Runtime projection

- exact constructor test;
- nested payload extraction;
- ignored payload not unnecessarily projected;
- or-pattern temporary binding isolation;
- source-order preservation;
- family-pattern candidate lowering.

---

# 94. Performance Regression Requirements

Performance tests should assert architecture, not only wall-clock duration.

Useful counters include:

```text
constructor decompositions
usefulness states
proof merges
inhabitation states
witness branches
symbolic coverage nodes
```

For the outer-only `ExpressionEvaluation.eval` match, a strong regression property is:

```text
recursive decomposition of Expression payloads = 0
```

beyond the root constructor decomposition, because every arm binds or ignores its recursive children rather than structurally matching them.

The compiler should also assert bounded `TypeStore` growth during coverage so recursive index chains cannot silently return.

---

# 95. Examples of Invalid Architecture

The following implementation strategies are explicitly non-conforming.

## 95.1 Eager recursive universe construction

Invalid:

```text
space(T)
    recursively constructs complete spaces for every constructor payload type
```

This fails for recursive GADTs.

## 95.2 Exact-TypeId cycle detection as termination theorem

Invalid:

```text
stop only when the same TypeId is encountered again
```

Indexed recursion may create genuinely distinct type structure indefinitely.

## 95.3 Nominal-head-only recursion cutoff

Invalid:

```text
if Expression was seen once, never decompose Expression again
```

This incorrectly rejects legitimate finite nested patterns.

## 95.4 Open-domain conflation

Invalid:

```text
closed recursive payload represented as permanently unknown/open
```

This loses nested exhaustiveness precision.

## 95.5 Runtime source-name matching

Invalid:

```text
compare variant base names at runtime
```

Exact semantic identity must be used.

---

# 96. Canonical Architectural Invariant

The pattern system should be understood through the following invariant:

> **Types tell the matcher which constructors may exist at the current position. Patterns tell the matcher which positions must be inspected. Proofs tell the matcher which constructor branches are semantically reachable.**

This yields three clean dimensions:

```text
Type/domain structure
    -> possible constructor heads

Pattern structure
    -> demanded decomposition depth

GADT proof environment
    -> branch feasibility and type refinement
```

Keeping these dimensions separate is what gives Phalcom both expressive power and predictable semantic behavior.

---

# 97. Summary of the Pattern System's Power

Phalcom's pattern system supports much more than syntactic destructuring.

It can express:

- ordinary algebraic data elimination;
- nested option/result pipelines;
- exact and overloaded variant selectors;
- whole associated variant families;
- partial callable selector-family patterns;
- tuple/list/record/map structural matching;
- GADT-indexed interpreters;
- higher-kinded typed AST evaluators;
- constructor-local existential unpacking;
- type-safe command/query routing;
- typed serialization descriptions;
- recursive compiler AST transformations;
- state-machine transition systems;
- statically checked protocol handlers;
- precise unreachable/redundant-case diagnostics;
- exact missing-case witnesses;
- compile-time totality guarantees over closed domains.

Its defining strength is the combination of:

```text
rich source patterns
+
selector/family identity
+
GADT proof refinement
+
strict exhaustiveness
+
demand-driven recursive decomposition
```

That combination allows Phalcom to use pattern matching as a genuine typed elimination mechanism rather than a convenience wrapper around runtime conditionals.

---

# 98. Final Normative Statement

The complete system is governed by the following rule:

> **A Phalcom pattern match is an ordered, statically resolved, proof-producing elimination over a symbolic value domain. Exact variant and family identity come from the declaration/selector model; GADT constructor observation introduces branch-local equality evidence; bindings inherit refined payload types; exhaustiveness and usefulness are decided over a finite demand-driven pattern matrix; recursive datatype structure is opened only where finite source patterns require it; and lowering consumes resolved semantic identities without reinterpreting source syntax.**

This is the architectural foundation on which future guard, literal, type-test, pin, alias, and active-pattern extensions must build.

---

# Appendix A — Surface Quick Reference

```phalcom
_                                   // wildcard
value                               // binding

Animal::Dog                         // exact singleton variant
Animal::Dog()                       // exact nullary constructor
Animal::Dog(name)                   // exact positional constructor
Animal::Dog(name, named: age)       // exact labeled constructor

Dog(name)                           // contextual owner shorthand
Animal::Dog*                        // whole variant family
Animal::Dog(...)                    // callable family pattern
Animal::Dog(name, ...)              // prefix-constrained family pattern
Animal::Dog(..., named: age)        // suffix-constrained family pattern
Animal::Dog(name, ..., named: age)  // prefix + suffix family pattern

A(x) | B(x)                         // or-pattern

(a, b)                              // tuple
[first, *rest]                      // list
#{name: name, age: age}             // record
{# "status": status }              // map

Some(Ok(value))                     // nested pattern
```

---

# Appendix B — Semantic Quick Reference

```text
Wildcard / binding
    covers current subject without decomposition

Exact variant
    resolve VariantId
    solve GADT feasibility
    open constructor-local generics
    specialize payload subjects

Family pattern
    resolve VariantFamilyId / SelectorPattern
    enumerate exact declaration-backed candidates
    filter by GADT reachability

Nested pattern
    decompose only requested payload positions

Or-pattern
    union of alternatives
    same binding-name set required
    retain only common proof facts

Usefulness
    impossible in original domain?
    otherwise adds new value beyond previous rows?

Exhaustiveness
    wildcard useful after all rows?
        yes -> missing case
        no  -> proven
```

---

# Appendix C — Repository Reconciliation Baseline

This specification was consolidated against the repository state at:

```text
aureat/phalcom-lang
e932aac4e21a5b346e719ede5a24f94e7b924ab3
feat(semantic): complete SC-4.8 typing integration
```

Primary implementation/specification surfaces reconciled include:

```text
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/pattern_space.rs
phalcom-semantic/src/checker/exhaustiveness.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/types/case_instantiation.rs
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-ast/src/parser.rs

docs/impl/adt-gadt-associated-lookup/part-5/
    05.1-match-surface-pattern-semantics-exhaustiveness-gadt-proofs-technical-spec.md

docs/spec/next/phalcom-pattern-matching-spec.md
```

The principal architectural correction relative to the older Part 05.1 coverage implementation is the prohibition on eager recursive payload-space construction and the adoption of demand-driven, pattern-matrix-based exhaustiveness/usefulness as the target semantic architecture.
