# Phalcom Generic Typing and Semantic Analysis — Generic Instantiation, Constraint Solving, and Applied-Type Laws

**Status:** Technical Specification
**Area:** Typing / Semantic Analysis / Generics
**Scope:** Rank-1 nominal generics, generic callables, generic methods, generic ADTs, applied types, inference, contextual typing, substitution, and constraint solving
**Reference fixture:** `Either<L, R>`

---

## 1. Purpose

This document defines the semantic laws governing generic typing in Phalcom.

The laws specify how the semantic analyzer must:

- represent generic declarations;
- instantiate generic declarations;
- infer generic arguments;
- collect and solve type constraints;
- use argument types and expected types together;
- specialize generic methods from receiver types;
- preserve and replace generic parameters correctly;
- handle repeated type variables;
- recursively solve nested applied types;
- type higher-order generic calls;
- keep generic instantiations independent;
- distinguish underconstrained typing from dynamic typing;
- reject contradictory substitutions;
- preserve canonical applied-type identity;
- type generic enum variant constructors;
- preserve the complete applied type through variant reconstruction;
- and expose enough semantic information for tooling and verification.

The laws are expressed using the generic algebraic data type `Either<L, R>` because it exercises multiple independent generic parameters and permits operations which preserve, replace, reorder, and recursively compose those parameters.

These laws apply generally. `Either` is a reference fixture, not a special semantic case.

---

# 2. Generic model

Phalcom generics are nominal and participate in the language's gradual static typing model.

A generic declaration introduces type parameters:

```phalcom
class Box<T> {
}
```

An application of that declaration produces an applied type:

```phalcom
Box<Int>
```

For an enum:

```phalcom
enum Either<L, R> {
    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)
}
```

the following are distinct applied types:

```text
Either<String, Int>
Either<String, Bool>
Either<Int, String>
```

They share the same nominal generic declaration `Either`, but have different generic arguments.

The semantic analyzer must retain this distinction.

It must not collapse:

```text
Either<String, Int>
```

to:

```text
Either
```

during ordinary typed analysis.

---

# 3. Reference `Either<L, R>` definition

The following definition is used throughout this specification.

```phalcom
enum Either<L, R> {

    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)

    isLeft -> Bool {
        match self {
            Left(value) => true
            Right(value) => false
        }
    }

    isRight -> Bool {
        match self {
            Left(value) => false
            Right(value) => true
        }
    }

    fold<T>(
        left: (value: L) -> T,
        right: (value: R) -> T
    ) -> T {
        match self {
            Left(value) => left.call(value)
            Right(value) => right.call(value)
        }
    }

    map<R2>(
        _ f: (value: R) -> R2
    ) -> Either<L, R2> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => Either::Right(f.call(value))
        }
    }

    mapLeft<L2>(
        _ f: (value: L) -> L2
    ) -> Either<L2, R> {
        match self {
            Left(value) => Either::Left(f.call(value))
            Right(value) => Either::Right(value)
        }
    }

    bimap<L2, R2>(
        left: (value: L) -> L2,
        right: (value: R) -> R2
    ) -> Either<L2, R2> {
        match self {
            Left(value) => Either::Left(left.call(value))
            Right(value) => Either::Right(right.call(value))
        }
    }

    flatMap<R2>(
        _ f: (value: R) -> Either<L, R2>
    ) -> Either<L, R2> {
        match self {
            Left(value) => Either::Left(value)
            Right(value) => f.call(value)
        }
    }

    orElse<L2>(
        _ other: Either<L2, R>
    ) -> Either<L2, R> {
        match self {
            Left(value) => other
            Right(value) => Either::Right(value)
        }
    }

    recover(
        _ f: (value: L) -> R
    ) -> R {
        match self {
            Left(value) => f.call(value)
            Right(value) => value
        }
    }

    getOrElse(_ fallback: R) -> R {
        match self {
            Left(value) => fallback
            Right(value) => value
        }
    }

    swap -> Either<R, L> {
        match self {
            Left(value) => Either::Right(value)
            Right(value) => Either::Left(value)
        }
    }

    zip<R2>(
        _ other: Either<L, R2>
    ) -> Either<L, (R, R2)> {
        match self {
            Left(value) => Either::Left(value)

            Right(value) => match other {
                Left(otherValue) => Either::Left(otherValue)
                Right(otherValue) => Either::Right((value, otherValue))
            }
        }
    }
}
```

---

# 4. Terminology

## 4.1 Generic declaration

A declaration containing generic parameters:

```phalcom
Either<L, R>
```

`L` and `R` are generic parameters owned by the declaration.

---

## 4.2 Applied type

A generic declaration with concrete or otherwise resolved type arguments:

```phalcom
Either<String, Int>
```

Conceptually:

```text
AppliedType {
    base: Either,
    arguments: [String, Int]
}
```

The precise compiler representation is implementation-defined, but the semantic identity is not.

---

## 4.3 Generic instantiation

Instantiation creates a fresh use of a generic declaration or callable and establishes substitutions for its generic parameters.

For:

```phalcom
identity<T>(_ value: T) -> T
```

the invocation:

```phalcom
identity(42)
```

may instantiate:

```text
T := Int
```

producing the specialized callable:

```text
(Int) -> Int
```

---

## 4.4 Substitution

A substitution maps a generic parameter to a type:

```text
T  -> Int
L  -> String
R  -> Bool
```

A substitution is associated with a specific generic parameter identity, not merely its textual name.

---

## 4.5 Constraint

A constraint expresses a semantic requirement which must hold for an instantiation to be valid.

Examples include:

```text
T = Int
```

```text
Either<L, R> = Either<String, Int>
```

or, where subtype compatibility rather than exact equality is appropriate:

```text
Actual <: Formal
```

Constraint direction depends on the relevant typing rule, variance model, and conversion semantics.

The generic laws in this document focus primarily on generic-variable identity and substitution.

---

## 4.6 Expected type

The expected type is contextual type information flowing from the surrounding expression into the expression being analyzed.

Example:

```phalcom
let value: Either<String, Int> =
    Either::Left("error")
```

The initializer is analyzed under the expected type:

```text
Either<String, Int>
```

Expected types are legitimate inputs to generic inference.

---

## 4.7 `Unknown`

`Unknown` represents a type that the analyzer has not proved.

It does not mean that arbitrary dynamic operations are valid.

Conceptually:

```text
Unknown = analyzer lacks sufficient proof
```

---

## 4.8 `Dynamic`

`Dynamic` is the explicit dynamic escape hatch.

Conceptually:

```text
Dynamic = defer ordinary static guarantees to dynamic semantics
```

`Unknown` and `Dynamic` must not be conflated.

---

# 5. Core generic laws

## GEN-01 — Generic parameters have declaration-scoped identity

Generic type variables are identified by declaration ownership and parameter identity, not solely by their textual names.

These two `T`s are independent:

```phalcom
@class
first<T>(_ value: T) -> T {
    value
}

@class
second<T>(_ value: T) -> T {
    value
}
```

They must not share substitutions.

Likewise, nested or shadowing declarations must not unify generic parameters merely because they have the same spelling.

### Required semantic property

Conceptually:

```text
first::T != second::T
```

even though:

```text
displayName(first::T) == "T"
displayName(second::T) == "T"
```

### Invalid implementation behavior

A compiler must not maintain generic substitutions in a global map keyed only by:

```text
"T"
```

---

## GEN-02 — Every generic invocation receives a fresh instantiation

Each call to a generic callable creates an independent instantiation environment.

Given:

```phalcom
@class
identity<T>(_ value: T) -> T {
    value
}
```

these calls are independent:

```phalcom
let a = Probe.identity(42)
let b = Probe.identity("hello")
```

Required result:

```text
a : Int
b : String
```

Conceptually:

```text
call #1:
    T := Int

call #2:
    T := String
```

The second call must not inherit:

```text
T := Int
```

from the first.

---

## GEN-03 — Argument-derived constraints specialize generic parameters

A generic parameter appearing in a formal parameter type may be solved from the corresponding argument.

Given:

```phalcom
@class
identity<T>(_ value: T) -> T {
    value
}
```

and:

```phalcom
let value = Probe.identity(42)
```

the argument produces:

```text
actual argument : Int
formal parameter: T

constraint:
    T := Int
```

The return type is consequently specialized:

```text
T
→ Int
```

and:

```text
value : Int
```

Generic inference must propagate substitutions through the entire instantiated callable signature.

---

## GEN-04 — A solved generic parameter specializes every occurrence of that parameter

A substitution applies consistently to every occurrence of the same generic parameter within its scope.

Given:

```phalcom
@class
pair<T>(
    _ first: T,
    _ second: T
) -> T {
    first
}
```

calling:

```phalcom
Probe.pair(1, 2)
```

solves:

```text
T := Int
```

and specializes:

```text
(T, T) -> T
```

to:

```text
(Int, Int) -> Int
```

The analyzer must not instantiate each occurrence independently.

---

## GEN-05 — Repeated generic parameters impose consistency constraints

A generic parameter appearing multiple times denotes the same type variable.

Given:

```phalcom
@class
merge<T>(
    _ value: Either<T, T>
) -> T {
    value.fold(
        left: |v| { v },
        right: |v| { v }
    )
}
```

this is valid:

```phalcom
let x: Either<Int, Int> = Either::Right(42)
let value = Probe.merge(x)
```

with:

```text
T := Int
```

This is invalid:

```phalcom
let x: Either<Int, String> =
    Either::Right("hello")

let value = Probe.merge(x)
```

The constraints would require:

```text
T := Int
T := String
```

which are contradictory.

The invocation must be rejected.

The analyzer must not treat the two `T` positions independently.

---

## GEN-06 — Generic constructors may be only partially constrained by their arguments

A variant constructor need not expose every generic parameter in its payload.

For:

```phalcom
Either<L, R>::Left(L)
```

the invocation:

```phalcom
Either::Left("failure")
```

directly determines:

```text
L := String
```

but provides no argument-derived evidence for `R`.

Similarly:

```phalcom
Either::Right(42)
```

directly determines:

```text
R := Int
```

but provides no argument-derived evidence for `L`.

Partial inference is valid and must not itself cause rejection.

The analyzer must continue solving from other available sources.

---

## GEN-07 — Expected types participate in generic constraint solving

Expected type information may solve generic parameters not derivable from arguments.

Given:

```phalcom
let value: Either<String, Int> =
    Either::Left("failure")
```

argument-derived evidence gives:

```text
L := String
```

The expected type provides:

```text
Either<L, R>
=
Either<String, Int>
```

which additionally solves:

```text
R := Int
```

Final type:

```text
Either<String, Int>
```

This must be accepted without requiring explicit generic arguments.

The symmetric law applies to:

```phalcom
let value: Either<String, Int> =
    Either::Right(42)
```

where:

```text
R := Int
```

comes from the argument and:

```text
L := String
```

comes from context.

---

## GEN-08 — Contextual inference and argument inference form one constraint problem

The analyzer must not treat expected-type inference as an unrelated post-processing coercion.

For:

```phalcom
let value: Either<String, Int> =
    Either::Left("failure")
```

the correct semantic model is conceptually:

```text
instantiate:
    Either::Left<L, R>(L) -> Either<L, R>

argument constraint:
    L := String

context constraint:
    Either<L, R> compatible with Either<String, Int>

solution:
    L := String
    R := Int

result:
    Either<String, Int>
```

The analyzer must not first conclude:

```text
Either<String, Dynamic>
```

and subsequently permit assignment to:

```text
Either<String, Int>
```

through dynamic compatibility.

The generic arguments must be solved directly whenever sufficient static evidence exists.

---

## GEN-09 — Insufficient evidence produces `Unknown`, not implicit `Dynamic`

When a generic parameter remains genuinely unsolved after available static evidence is exhausted, the analyzer must represent lack of proof as `Unknown`, unless another language rule explicitly supplies a default.

Given:

```phalcom
let value = Either::Left("failure")
```

the analyzer can prove:

```text
L := String
```

but cannot prove `R`.

The semantic result is conceptually:

```text
Either<String, Unknown>
```

Likewise:

```phalcom
let value = Either::Right(42)
```

is conceptually:

```text
Either<Unknown, Int>
```

The analyzer must not silently convert unresolved generic parameters to:

```text
Dynamic
```

unless dynamic typing is explicitly introduced by language semantics.

---

## GEN-10 — `Unknown` must remain distinguishable from `Dynamic`

These types carry different semantic meaning:

```text
Either<String, Unknown>
Either<String, Dynamic>
```

The first means:

```text
the analyzer has not proved R
```

The second means:

```text
R is intentionally dynamic
```

A successful generic inference path must therefore be testable for accidental introduction of `Dynamic`.

Where sufficient static evidence exists:

```phalcom
let value: Either<String, Int> =
    Either::Left("failure")
```

the resulting type must be exactly:

```text
Either<String, Int>
```

and not a type involving either `Unknown` or `Dynamic`.

---

# 6. Applied-type laws

## GEN-11 — Applied generic types preserve nominal identity and ordered arguments

An applied type consists semantically of:

```text
generic declaration identity
+
ordered generic arguments
```

Therefore:

```text
Either<String, Int>
```

is not equivalent to:

```text
Either<Int, String>
```

and neither is merely:

```text
Either
```

The order of generic arguments is semantically significant.

---

## GEN-12 — Applied types are structurally traversable for generic solving

Although generic types are nominal, their applied arguments must be recursively available to the constraint solver.

Given a formal parameter:

```text
Either<L, R>
```

and an actual type:

```text
Either<String, Int>
```

the analyzer may derive:

```text
L := String
R := Int
```

because the nominal generic constructors agree and their corresponding applied arguments can be constrained.

This does not make `Either` structurally typed.

It is nominal matching followed by argument-level constraint solving.

---

## GEN-13 — Generic solving recursively traverses nested applied types

Given:

```phalcom
@class
flatten<L, R>(
    _ value: Either<L, Either<L, R>>
) -> Either<L, R> {
    value.flatMap(|inner| {
        inner
    })
}
```

and an argument with type:

```text
Either<String, Either<String, Int>>
```

the solver must recursively derive:

```text
outer:
    L := String

inner:
    L := String
    R := Int
```

Final substitution:

```text
L := String
R := Int
```

Final return type:

```text
Either<String, Int>
```

Nested applied-type solving must not stop after the outer generic layer.

---

## GEN-14 — Repeated variables across nested applied types remain identical

In:

```text
Either<L, Either<L, R>>
```

both occurrences of `L` denote the same generic parameter.

Therefore:

```text
Either<String, Either<String, Int>>
```

is compatible.

But:

```text
Either<String, Either<Bool, Int>>
```

requires:

```text
L := String
L := Bool
```

and must be rejected.

A recursive solver must preserve generic-variable identity across nesting.

---

## GEN-15 — Canonical applied types should converge to canonical semantic identity

When the type store canonicalizes applied types, independent derivations of:

```text
Either<String, Int>
```

should resolve to the same canonical semantic type identity.

For example:

```phalcom
let a: Either<String, Int> =
    Either::Left("a")

let b: Either<String, Int> =
    Either::Right(1)
```

Although `a` and `b` are produced through different variant constructors and inference paths, their static applied type must be the same canonical:

```text
Either<String, Int>
```

where canonicalization is part of the type-store contract.

---

# 7. Receiver specialization laws

## GEN-16 — An applied generic receiver specializes declaration-owned parameters

Consider:

```phalcom
let value: Either<String, Int> = ...
```

and:

```phalcom
value.map(...)
```

The receiver fixes the enum-owned parameters:

```text
L := String
R := Int
```

before solving method-local generic parameters.

The method declaration:

```text
map<R2>((R) -> R2) -> Either<L, R2>
```

is therefore receiver-specialized to:

```text
map<R2>((Int) -> R2) -> Either<String, R2>
```

before or as part of call inference.

The exact implementation order is internal, but the observable semantics must be equivalent.

---

## GEN-17 — Receiver-owned substitutions and method-owned substitutions are distinct

For:

```phalcom
Either<L, R>.map<R2>
```

there are two generic scopes:

```text
Either declaration:
    L
    R

map declaration:
    R2
```

For:

```phalcom
let source: Either<String, Int> = ...
let mapped = source.map(|value| {
    value > 0
})
```

the semantic substitutions are:

```text
receiver:
    L := String
    R := Int

method:
    R2 := Bool
```

Method-local solving must not overwrite declaration-owned substitutions.

---

## GEN-18 — Generic transformations preserve untouched parameters exactly

For:

```text
map<R2>((R) -> R2) -> Either<L, R2>
```

`map` changes `R` to `R2` but preserves `L`.

Given:

```text
receiver:
    Either<String, Int>

transform:
    (Int) -> Bool
```

required result:

```text
Either<String, Bool>
```

The preserved `String` must not become:

```text
Unknown
Dynamic
fresh L
Object
```

or any widened approximation unless some independent typing rule explicitly requires that transformation.

Likewise:

```text
mapLeft<L2>
```

must preserve `R` exactly.

---

## GEN-19 — Generic transformations may replace multiple parameters simultaneously

For:

```phalcom
bimap<L2, R2>(
    left: (value: L) -> L2,
    right: (value: R) -> R2
) -> Either<L2, R2>
```

given:

```text
receiver:
    Either<String, Int>

left transform:
    (String) -> Bool

right transform:
    (Int) -> Float
```

the solver must derive:

```text
L2 := Bool
R2 := Float
```

and return:

```text
Either<Bool, Float>
```

Both substitutions belong to one generic invocation and must be solved coherently.

---

## GEN-20 — Generic transformations may permute generic arguments

Generic parameters need not remain in declaration order in a result type.

For:

```phalcom
swap -> Either<R, L>
```

a receiver:

```text
Either<String, Int>
```

must produce:

```text
Either<Int, String>
```

The analyzer must apply substitutions by parameter identity, then instantiate the result expression.

It must not assume result generic arguments retain receiver ordering.

---

# 8. Higher-order generic laws

## GEN-21 — Callable argument types contribute generic constraints

Given:

```phalcom
@class
lift<L, A, B>(
    _ value: Either<L, A>,
    _ transform: (value: A) -> B
) -> Either<L, B> {
    value.map(transform)
}
```

and:

```text
value:
    Either<String, Int>

transform:
    (Int) -> Bool
```

the invocation must solve:

```text
L := String
A := Int
B := Bool
```

producing:

```text
Either<String, Bool>
```

The generic solver must extract constraints from callable parameter and return types.

---

## GEN-22 — Constraints may be distributed across multiple arguments

A generic parameter need not be solvable from one argument alone.

For:

```phalcom
lift<L, A, B>(
    _ value: Either<L, A>,
    _ transform: (value: A) -> B
) -> Either<L, B>
```

the first argument solves:

```text
L
A
```

while the second argument solves or constrains:

```text
A
B
```

The combined constraint environment yields the full substitution.

Generic inference is invocation-wide rather than argument-local.

---

## GEN-23 — Closure parameter types may be contextually specialized from generic formals

Given:

```phalcom
let source: Either<String, Int> =
    Either::Right(42)

let mapped =
    source.map(|value| {
        value > 0
    })
```

after receiver specialization, `map` expects:

```text
(Int) -> R2
```

Therefore the closure parameter:

```phalcom
value
```

must be analyzed as:

```text
Int
```

The closure body:

```phalcom
value > 0
```

then yields:

```text
Bool
```

which supplies:

```text
R2 := Bool
```

This creates bidirectional information flow:

```text
generic receiver
    -> closure parameter type
    -> closure body analysis
    -> generic return parameter
```

The semantic analyzer must support this inference path.

---

## GEN-24 — Closure result types may solve generic return parameters

For:

```phalcom
map<R2>(
    _ f: (value: R) -> R2
) -> Either<L, R2>
```

and:

```phalcom
source.map(|value| {
    value == 42
})
```

the closure body has type:

```text
Bool
```

therefore:

```text
R2 := Bool
```

and the method invocation has type:

```text
Either<L, Bool>
```

after applying the receiver's `L`.

The analyzer must not leave `R2` unresolved when the closure return type is known.

---

## GEN-25 — Higher-order constraints must detect contradictions

Given a generic callable requiring:

```text
(A) -> B
```

a closure or callable argument incompatible with the solved `A` must be rejected.

For example, if receiver inference has established:

```text
A := Int
```

the callable parameter must be compatible with the required input position according to Phalcom's callable compatibility rules.

Generic inference must not ignore callable structure merely because the return type can otherwise be solved.

---

# 9. Method-local generic freshness laws

## GEN-26 — Method-local generic variables are instantiated freshly for every call

Consider two invocations:

```phalcom
let firstInput: Either<String, Int> =
    Either::Right(41)

let first =
    EitherGenericProbe.lift(
        firstInput,
        |value| {
            value == 41
        }
    )

let secondInput: Either<Int, Bool> =
    Either::Right(true)

let second =
    EitherGenericProbe.lift(
        secondInput,
        |value| {
            "second"
        }
    )
```

Required instantiations:

```text
call #1:
    L := String
    A := Int
    B := Bool

call #2:
    L := Int
    A := Bool
    B := String
```

Required result:

```text
first  : Either<String, Bool>
second : Either<Int, String>
```

No substitution from call #1 may survive into call #2.

---

## GEN-27 — Generic instantiation state must not leak across branches or sibling expressions

Freshness applies not only across sequential calls but across all distinct invocation expressions.

For example:

```phalcom
if condition {
    Probe.identity(42)
} else {
    Probe.identity("hello")
}
```

Each invocation receives a separate generic instantiation.

Any later type joining or branch reconciliation is a separate semantic operation.

The generic solver must not force both calls to share one `T`.

---

# 10. Reconstruction and ADT laws

## GEN-28 — Reconstructing a variant must preserve the complete expected applied type

Consider:

```phalcom
map<R2>(_ f: (value: R) -> R2) -> Either<L, R2> {
    match self {
        Left(value) => Either::Left(value)
        Right(value) => Either::Right(f.call(value))
    }
}
```

In the `Left` branch:

```phalcom
Either::Left(value)
```

the payload establishes only:

```text
Left parameter := L
```

There is no value corresponding to `R2`.

However, the branch is expected to produce:

```text
Either<L, R2>
```

Therefore the expected branch/result type must complete the constructor instantiation.

Required constructor result:

```text
Either<L, R2>
```

not:

```text
Either<L, Unknown>
```

and not:

```text
Either<L, Dynamic>
```

when `R2` is already known from the enclosing method instantiation.

---

## GEN-29 — Runtime branch absence does not erase static generic information

Given:

```phalcom
let source: Either<String, Int> =
    Either::Left("failure")

let mapped: Either<String, Bool> =
    source.map(|value| {
        value > 0
    })
```

the runtime value is a `Left`.

No runtime `Bool` payload exists.

Nevertheless the static type is:

```text
Either<String, Bool>
```

because the applied type describes the entire type of the value, not merely the payload carried by its currently active variant.

The analyzer must not derive generic arguments solely from the runtime-selected variant.

---

## GEN-30 — Exact variant knowledge and family applied type are related but distinct

A value may be known to be a particular variant while still retaining a complete generic family type.

For example, analysis may know:

```text
exact case:
    Either::Left

family type:
    Either<String, Bool>
```

The absence of a right payload does not make the family's `R` parameter meaningless.

This is required for:

- exhaustive matching;
- generic transformations;
- reflection;
- typed multiple dispatch where applicable;
- assignment compatibility;
- later control-flow joins;
- and reconstruction of family values.

---

# 11. Chained specialization laws

## GEN-31 — Generic substitutions compose across chained operations

Given:

```phalcom
let initial: Either<String, Int> =
    Either::Right(41)

let mapped =
    initial.map(|value| {
        value == 41
    })

let leftMapped =
    mapped.mapLeft(|value| {
        100
    })

let swapped =
    leftMapped.swap
```

the analyzer must derive:

```text
initial:
    Either<String, Int>

map:
    R2 := Bool

mapped:
    Either<String, Bool>

mapLeft:
    L2 := Int

leftMapped:
    Either<Int, Bool>

swap:
    Either<Bool, Int>
```

Every operation must consume the fully specialized output type of the preceding operation.

---

## GEN-32 — A chained call must not revert to an unspecialized generic declaration

Given:

```text
Either<String, Bool>
```

as the result of one call, resolving the next method must use:

```text
receiver = Either<String, Bool>
```

not:

```text
receiver = Either<L, R>
```

without the established substitutions.

Generic application information is part of the receiver's semantic type and must survive expression composition.

---

# 12. Generic return-type laws

## GEN-33 — Generic return types are specialized from the final solved substitution

Given:

```phalcom
@class
preserve<L, R>(
    _ value: Either<L, R>
) -> Either<L, R> {
    value
}
```

and:

```phalcom
let source: Either<String, Int> = ...
let result = Probe.preserve(source)
```

the return type must become:

```text
Either<String, Int>
```

after substitution.

The analyzer must not expose the unspecialized return:

```text
Either<L, R>
```

outside the invocation.

---

## GEN-34 — Return types may contain generic parameters solved only indirectly

For:

```phalcom
@class
lift<L, A, B>(
    _ value: Either<L, A>,
    _ f: (value: A) -> B
) -> Either<L, B> {
    value.map(f)
}
```

`B` may be solved entirely from the second argument's return type.

The return type:

```text
Either<L, B>
```

must still be fully specialized after the complete invocation constraint set has been solved.

Return-type specialization is based on the final substitution, not only argument-local substitutions.

---

## GEN-35 — Expected return context may further constrain otherwise-unsolved generic parameters

Where Phalcom permits bidirectional contextual inference, expected invocation result types may constrain generic parameters that remain unsolved by arguments.

Conceptually:

```phalcom
let result: SomeGeneric<Int> =
    Probe.make(...)
```

may provide constraints to generic parameters appearing in `Probe.make`'s return type.

This is the same general principle established by `GEN-07`.

Any implementation limits on contextual inference must be explicitly specified rather than implemented accidentally.

---

# 13. Contradiction and failure laws

## GEN-36 — Contradictory generic constraints must produce semantic failure

When inference requires mutually incompatible substitutions for the same generic parameter, analysis must produce a diagnostic.

Example:

```phalcom
@class
merge<T>(
    _ value: Either<T, T>
) -> T {
    value.fold(
        left: |v| { v },
        right: |v| { v }
    )
}

let source: Either<Int, String> =
    Either::Right("hello")

let bad = Probe.merge(source)
```

Constraints:

```text
T := Int
T := String
```

Required outcome:

```text
semantic diagnostic
```

not:

```text
T := Dynamic
```

not:

```text
T := Unknown
```

and not arbitrary selection of one constraint.

---

## GEN-37 — Contradiction is distinct from underconstraint

These are different situations.

### Underconstrained

```phalcom
let x = Either::Left("failure")
```

Known:

```text
L := String
```

Unsolved:

```text
R
```

Conceptual result:

```text
Either<String, Unknown>
```

### Contradictory

```text
required:
    T := Int
    T := String
```

Result:

```text
diagnostic
```

An unsolved variable may remain `Unknown`.

A variable proven to require incompatible types must not.

---

## GEN-38 — Failed generic inference must not silently escape through `Dynamic`

Generic inference failure is not permission to erase static information.

Given a contradictory invocation, the analyzer must not recover by converting the problematic parameter to:

```text
Dynamic
```

unless the program explicitly introduces dynamic semantics in a manner authorized by the language.

This law is necessary to preserve the meaning of Phalcom's gradual typing model.

---

## GEN-39 — Invalid expected-type completion must be rejected

Expected context may provide constraints, but those constraints must remain compatible with argument-derived evidence.

Example:

```phalcom
let bad: Either<String, Int> =
    Either::Left(42)
```

Argument inference requires:

```text
L := Int
```

Expected type requires:

```text
L := String
R := Int
```

The conflict:

```text
L := Int
L := String
```

must produce a semantic diagnostic.

Contextual typing cannot overwrite contradictory evidence.

---

# 14. No-accidental-dynamic laws

## GEN-40 — Fully solved generic expressions must remain fully static

Where all generic parameters can be solved, the final type must contain no incidental:

```text
Unknown
```

or:

```text
Dynamic
```

Example:

```phalcom
let source: Either<String, Int> =
    Either::Right(42)

let mapped =
    source.map(|value| {
        value > 0
    })
```

Required:

```text
mapped : Either<String, Bool>
```

Semantic verification should assert:

```text
type == Either<String, Bool>
type != Either<String, Unknown>
type != Either<String, Dynamic>
AnalysisStatus == Known
```

or the equivalent concepts used by the implementation.

---

## GEN-41 — A correct final type must be reached through valid generic evidence

A test of generic inference should not rely solely on the final apparent type.

The semantic system should make it possible, at least in test infrastructure, to distinguish:

```text
valid path:
    receiver specialization
    +
    closure inference
    +
    generic substitution
```

from an invalid path such as:

```text
erase to Dynamic
    ->
accept dynamically
    ->
later coerce to expected static type
```

The implementation need not expose unstable internal solver machinery publicly.

However, semantic test infrastructure should be capable of observing stable facts such as:

- instantiated callable identity;
- generic parameter substitutions;
- specialized result type;
- analysis status;
- diagnostics;
- and, where practical, constraint provenance.

---

# 15. Inference provenance laws

## GEN-42 — Generic substitution provenance should be semantically observable in tests

For critical generic operations, the test harness should be able to verify not merely the final type but the source of generic specialization.

For:

```phalcom
let source: Either<String, Int> =
    Either::Right(42)

let mapped =
    source.map(|value| {
        value > 0
    })
```

a useful semantic observation is:

```text
receiver substitution:
    L := String
    R := Int

method-local substitution:
    R2 := Bool

result:
    Either<String, Bool>
```

A stable test-facing observation surface is preferable to direct dependence on temporary solver internals.

---

## GEN-43 — Constraint provenance must distinguish major inference sources where available

If provenance is retained, at minimum the system should be able to distinguish conceptually between constraints originating from:

```text
argument
receiver
expected type
callable parameter
callable result
generic bound
explicit generic argument
```

For example:

```phalcom
let x: Either<String, Int> =
    Either::Left("failure")
```

may be represented conceptually as:

```text
L := String
    source: argument

R := Int
    source: expected type
```

The exact API or data representation is implementation-defined.

---

# 16. Generic callable specialization laws

## GEN-44 — A generic callable has a formal type and an instantiated type

Given:

```phalcom
@class
identity<T>(_ value: T) -> T
```

its formal callable type is conceptually:

```text
forall T. (T) -> T
```

Phalcom does not need this exact user-facing notation.

For:

```phalcom
identity(42)
```

the invocation creates an instantiated callable type:

```text
(Int) -> Int
```

The formal declaration remains generic.

Instantiation must not mutate the declaration globally.

---

## GEN-45 — Specialization is per use, not destructive mutation

After:

```phalcom
identity(42)
```

the declaration must remain capable of independently producing:

```text
(String) -> String
```

for:

```phalcom
identity("hello")
```

The compiler must not rewrite the declaration itself from:

```text
(T) -> T
```

to:

```text
(Int) -> Int
```

as shared semantic state.

---

# 17. Explicit generic arguments

## GEN-46 — Explicit generic arguments constrain the same generic parameters as inference

Where syntax explicitly supplies generic arguments, those arguments establish substitutions directly.

Conceptually:

```phalcom
Probe.identity<Int>(42)
```

establishes:

```text
T := Int
```

before or during ordinary argument checking.

Argument types must then be compatible with the explicitly specialized callable.

---

## GEN-47 — Explicit generic arguments must still satisfy all other constraints

Explicit specialization does not disable semantic checking.

Conceptually:

```phalcom
Probe.identity<Int>("hello")
```

must be rejected because:

```text
explicit:
    T := Int

formal parameter:
    Int

actual:
    String
```

is incompatible.

Explicit arguments constrain inference; they do not bypass it.

---

# 18. Generic ADT constructor laws

## GEN-48 — Variant constructors participate in ordinary generic inference

`Either::Left` and `Either::Right` must not require compiler-special-cased generic semantics.

Conceptually, `Left` behaves as a generic constructor whose type depends on the enclosing enum parameters:

```text
Either::Left<L, R> : (L) -> Either<L, R>
Either::Right<L, R>: (R) -> Either<L, R>
```

The exact reflective representation may differ, but type checking must obey equivalent semantics.

---

## GEN-49 — Generic parameters absent from constructor payloads remain available for contextual solving

For:

```text
Either::Left<L, R>(L)
```

`R` is absent from the argument list but remains a parameter of the resulting applied family type.

Therefore:

```phalcom
let x: Either<String, Int> =
    Either::Left("failure")
```

must solve `R` contextually.

The compiler must not conclude that because `R` is absent from the payload, `Left` has no meaningful `R`.

---

## GEN-50 — Different variants may infer different subsets of one family's generic parameters

For:

```text
Left(L)
Right(R)
```

the argument-derived constraint subsets differ:

```text
Left:
    solves L

Right:
    solves R
```

Both constructors nevertheless produce values of the same generic family:

```text
Either<L, R>
```

The family parameter model must therefore be independent of payload layout.

---

# 19. Generic method examples

## 19.1 `map`

Declaration:

```phalcom
map<R2>(
    _ f: (value: R) -> R2
) -> Either<L, R2>
```

Given:

```text
receiver:
    Either<String, Int>
```

receiver specialization produces:

```text
f:
    (Int) -> R2

result:
    Either<String, R2>
```

Given closure:

```phalcom
|value| {
    value > 0
}
```

closure analysis produces:

```text
(Int) -> Bool
```

therefore:

```text
R2 := Bool
```

final result:

```text
Either<String, Bool>
```

---

## 19.2 `mapLeft`

Declaration:

```phalcom
mapLeft<L2>(
    _ f: (value: L) -> L2
) -> Either<L2, R>
```

Given:

```text
receiver:
    Either<String, Int>
```

and:

```text
f:
    (String) -> Bool
```

the result must be:

```text
Either<Bool, Int>
```

`R` remains exactly `Int`.

---

## 19.3 `bimap`

Declaration:

```phalcom
bimap<L2, R2>(
    left: (value: L) -> L2,
    right: (value: R) -> R2
) -> Either<L2, R2>
```

Given:

```text
receiver:
    Either<String, Int>

left:
    (String) -> Bool

right:
    (Int) -> Float
```

solve:

```text
L2 := Bool
R2 := Float
```

result:

```text
Either<Bool, Float>
```

---

## 19.4 `flatMap`

Declaration:

```phalcom
flatMap<R2>(
    _ f: (value: R) -> Either<L, R2>
) -> Either<L, R2>
```

Given:

```text
receiver:
    Either<String, Int>
```

the method specializes to:

```text
f:
    (Int) -> Either<String, R2>

result:
    Either<String, R2>
```

If the closure returns:

```text
Either<String, Bool>
```

then:

```text
R2 := Bool
```

and the invocation returns:

```text
Either<String, Bool>
```

The closure must not return:

```text
Either<Int, Bool>
```

because the preserved left parameter is already constrained to:

```text
String
```

---

## 19.5 `swap`

Declaration:

```phalcom
swap -> Either<R, L>
```

Receiver:

```text
Either<String, Int>
```

Result:

```text
Either<Int, String>
```

No new generic parameter is introduced.

This is pure substitution permutation.

---

## 19.6 `zip`

Declaration:

```phalcom
zip<R2>(
    _ other: Either<L, R2>
) -> Either<L, (R, R2)>
```

Given:

```text
receiver:
    Either<String, Int>

other:
    Either<String, Bool>
```

derive:

```text
L := String
R := Int
R2 := Bool
```

result:

```text
Either<String, (Int, Bool)>
```

If `other` instead has:

```text
Either<Error, Bool>
```

then the shared `L` requirement is violated unless `Error` and `String` satisfy whatever exact compatibility relation the signature requires.

---

# 20. Runtime consistency laws

## GEN-51 — Static generic specialization must agree with produced runtime values

Generic typing tests must verify both static semantics and actual language execution.

For:

```phalcom
let source: Either<String, Int> =
    Either::Right(41)

let mapped =
    source.map(|value| {
        value + 1
    })
```

static analysis must determine:

```text
mapped : Either<String, Int>
```

and runtime execution must produce:

```text
Either::Right(42)
```

A correct static type with an incorrect runtime variant or payload is a failure.

---

## GEN-52 — Branch-preserving generic transformations must preserve runtime payloads

For:

```phalcom
let source: Either<String, Int> =
    Either::Left("failure")

let mapped =
    source.map(|value| {
        value + 1
    })
```

static type:

```text
Either<String, Int>
```

runtime result:

```text
Either::Left("failure")
```

The right-side transformation must not run.

The left payload must remain unchanged.

---

## GEN-53 — Runtime variant identity does not replace static applied-type identity

At runtime:

```text
Either::Left("failure")
```

may expose exact variant identity.

At the static level it may simultaneously have:

```text
Either<String, Bool>
```

after an operation such as `map`.

Runtime case identity and applied generic family identity represent different dimensions of information and must remain compatible.

---

# 21. Semantic verification requirements

The `Either` generic conformance package should verify several categories of semantic facts.

## 21.1 Binding types

For stable named bindings, Rust-side semantic tests should assert exact inferred applied types.

Examples:

```text
contextualLeft:
    Either<String, Int>

mapped:
    Either<String, Bool>

mappedLeft:
    Either<Bool, Int>

swapped:
    Either<Int, String>

flattened:
    Either<String, Int>
```

---

## 21.2 Analysis status

Where inference is complete, the analyzer should report the equivalent of:

```text
AnalysisStatus::Known
```

The precise status type is implementation-defined.

The test must ensure the expression was not merely accepted through incomplete or dynamic typing.

---

## 21.3 Substitutions

Where a generic invocation is observable, tests should verify the solved substitutions.

Example:

```text
map invocation:

receiver:
    L := String
    R := Int

method:
    R2 := Bool
```

---

## 21.4 Diagnostics

Negative fixtures must assert specific diagnostic categories rather than merely checking that analysis failed.

Important failures include:

```text
conflicting constructor context
wrong map result
wrong mapLeft result
repeated-variable conflict
flatMap preserved-parameter conflict
nested repeated-variable conflict
```

---

## 21.5 Runtime values

Runtime tests should verify:

```text
variant identity
payload value
branch preservation
transform application
nested values
```

where runtime infrastructure permits direct inspection.

A top-level Boolean `run` function may additionally aggregate language-side assertions using Phalcom's Boolean operator:

```phalcom
testOne
    and testTwo
    and testThree
```

Phalcom does not use `&&` for Boolean conjunction.

---

# 22. Recommended conformance laws

The following law identifiers form the normative generic conformance catalog defined by this specification:

```text
GEN-01  Generic parameters have declaration-scoped identity
GEN-02  Every generic invocation receives a fresh instantiation
GEN-03  Argument-derived constraints specialize generic parameters
GEN-04  A solved parameter specializes every occurrence
GEN-05  Repeated generic parameters impose consistency constraints
GEN-06  Generic constructors may be only partially argument-constrained
GEN-07  Expected types participate in generic constraint solving
GEN-08  Context and arguments form one constraint problem
GEN-09  Insufficient evidence produces Unknown, not implicit Dynamic
GEN-10  Unknown remains distinguishable from Dynamic

GEN-11  Applied generic types preserve nominal identity and ordered arguments
GEN-12  Applied types are recursively traversable for solving
GEN-13  Generic solving traverses nested applied types
GEN-14  Repeated variables across nesting remain identical
GEN-15  Equivalent applied types converge to canonical semantic identity

GEN-16  Applied receivers specialize declaration-owned parameters
GEN-17  Receiver-owned and method-owned substitutions are distinct
GEN-18  Generic transformations preserve untouched parameters exactly
GEN-19  Generic transformations may replace multiple parameters
GEN-20  Generic transformations may permute generic arguments

GEN-21  Callable argument types contribute generic constraints
GEN-22  Constraints may be distributed across multiple arguments
GEN-23  Closure parameters may be contextually specialized
GEN-24  Closure result types may solve generic result parameters
GEN-25  Higher-order constraints detect contradictions

GEN-26  Method-local generic variables are fresh for every call
GEN-27  Generic instantiation state does not leak across expressions

GEN-28  Variant reconstruction preserves complete expected applied type
GEN-29  Runtime branch absence does not erase static generic information
GEN-30  Exact variant knowledge and family applied type remain distinct

GEN-31  Generic substitutions compose across chained operations
GEN-32  Chained calls retain specialized receiver types

GEN-33  Generic return types specialize from final substitutions
GEN-34  Return parameters may be solved indirectly
GEN-35  Expected result context may constrain generic parameters

GEN-36  Contradictory generic constraints produce semantic failure
GEN-37  Contradiction is distinct from underconstraint
GEN-38  Failed generic inference does not silently escape through Dynamic
GEN-39  Invalid expected-type completion is rejected

GEN-40  Fully solved generic expressions remain fully static
GEN-41  Correct final types must arise from valid generic evidence
GEN-42  Generic substitution provenance should be observable in tests
GEN-43  Constraint provenance should identify major inference sources

GEN-44  Generic callables have formal and instantiated types
GEN-45  Specialization is per-use rather than destructive mutation

GEN-46  Explicit generic arguments constrain ordinary inference variables
GEN-47  Explicit generic arguments remain subject to compatibility checks

GEN-48  Variant constructors participate in ordinary generic inference
GEN-49  Payload-absent family parameters remain contextually solvable
GEN-50  Different variants may infer different subsets of family parameters

GEN-51  Static specialization agrees with produced runtime values
GEN-52  Branch-preserving transformations preserve runtime payloads
GEN-53  Runtime variant identity does not replace static applied-type identity
```

---

# 23. Reference positive conformance scenarios

A comprehensive `Either` suite should include at least the following successful programs.

## Contextual constructor completion

```phalcom
let left: Either<String, Int> =
    Either::Left("failure")

let right: Either<String, Int> =
    Either::Right(42)
```

Expected:

```text
left  : Either<String, Int>
right : Either<String, Int>
```

---

## Right-side transformation

```phalcom
let source: Either<String, Int> =
    Either::Right(42)

let mapped =
    source.map(|value| {
        value > 0
    })
```

Expected:

```text
mapped : Either<String, Bool>
```

---

## Left-side transformation

```phalcom
let source: Either<String, Int> =
    Either::Left("failure")

let mapped =
    source.mapLeft(|value| {
        value == "failure"
    })
```

Expected:

```text
mapped : Either<Bool, Int>
```

---

## Two-sided transformation

```phalcom
let source: Either<String, Int> =
    Either::Right(42)

let mapped =
    source.bimap(
        left: |value| {
            value == "failure"
        },
        right: |value| {
            value > 0
        }
    )
```

Expected:

```text
mapped : Either<Bool, Bool>
```

---

## Nested solving

```phalcom
let inner: Either<String, Int> =
    Either::Right(73)

let outer: Either<String, Either<String, Int>> =
    Either::Right(inner)

let flattened =
    EitherGenericProbe.flatten(outer)
```

Expected:

```text
flattened : Either<String, Int>
```

---

## Fresh repeated instantiation

```phalcom
let firstInput: Either<String, Int> =
    Either::Right(41)

let first =
    EitherGenericProbe.lift(
        firstInput,
        |value| {
            value == 41
        }
    )

let secondInput: Either<Int, Bool> =
    Either::Right(true)

let second =
    EitherGenericProbe.lift(
        secondInput,
        |value| {
            "second"
        }
    )
```

Expected:

```text
first  : Either<String, Bool>
second : Either<Int, String>
```

---

## Substitution composition

```phalcom
let initial: Either<String, Int> =
    Either::Right(41)

let mapped =
    initial.map(|value| {
        value == 41
    })

let leftMapped =
    mapped.mapLeft(|value| {
        100
    })

let swapped =
    leftMapped.swap
```

Expected sequence:

```text
Either<String, Int>
Either<String, Bool>
Either<Int, Bool>
Either<Bool, Int>
```

---

# 24. Reference rejection scenarios

## Constructor/context contradiction

```phalcom
let bad: Either<String, Int> =
    Either::Left(42)
```

Conflicting constraints:

```text
L := Int
L := String
```

Must reject.

---

## Wrong `map` result expectation

```phalcom
let source: Either<String, Int> =
    Either::Right(1)

let bad: Either<String, String> =
    source.map(|value| {
        value > 0
    })
```

Actual inferred result:

```text
Either<String, Bool>
```

Expected:

```text
Either<String, String>
```

Must reject.

---

## Repeated-variable contradiction

```phalcom
let source: Either<Int, String> =
    Either::Right("hello")

let bad =
    EitherGenericProbe.merge(source)
```

Required generic constraints:

```text
T := Int
T := String
```

Must reject.

---

## Nested repeated-variable contradiction

Formal:

```text
Either<L, Either<L, R>>
```

Actual:

```text
Either<String, Either<Bool, Int>>
```

Required:

```text
L := String
L := Bool
```

Must reject.

---

## `flatMap` preserved-side contradiction

Given:

```text
receiver:
    Either<String, Int>
```

and:

```text
flatMap<R2>:
    (Int) -> Either<String, R2>
```

a closure returning:

```text
Either<Int, Bool>
```

must not be accepted as:

```text
Either<String, Bool>
```

The preserved left-side generic parameter is already fixed to `String`.

---

# 25. Required architectural properties

An implementation conforming to these laws should have semantic concepts equivalent to:

```text
GenericParameterIdentity
AppliedType
GenericInstantiation
SubstitutionEnvironment
ConstraintSet
ExpectedType
AnalysisStatus
```

The exact structures and names are implementation-defined.

The implementation should avoid architectures in which generic correctness depends on:

- textual generic parameter names;
- global mutable substitution maps;
- destructive specialization of declarations;
- automatic conversion of unresolved variables to `Dynamic`;
- shallow matching of only the outer applied type;
- variant payload shape being treated as the complete generic family type;
- or assignment checking repairing an incorrectly inferred generic type after the fact.

---

# 26. Testing strategy

The canonical `Either` semantic package should exercise these laws at three independent levels.

### Semantic outcome

Assert:

```text
binding type
expression type
applied generic arguments
analysis status
diagnostics
```

### Inference-path verification

Where supported, assert:

```text
instantiated callable
generic substitutions
constraint origin
receiver specialization
expected-type completion
```

### Runtime behavior

Assert:

```text
variant
payload
branch selection
transformed value
preserved value
nested value
```

No one layer substitutes for another.

A runtime-correct program may still have unsound or accidentally dynamic static analysis.

A statically correct inferred type may still lower or execute incorrectly.

A correct final type may still have been obtained through an invalid inference path.

The `Either` conformance package is intended to detect all three classes of defect.

---

# 27. Summary

Phalcom generic inference is fundamentally a constraint-solving process over fresh generic parameter identities.

Arguments, receivers, closures, explicit generic arguments, nested applied types, and expected types may all contribute constraints.

A valid generic instantiation must:

1. create fresh parameters for the invocation;
2. preserve parameter identity across repeated occurrences;
3. recursively gather applicable constraints;
4. combine argument and contextual evidence;
5. reject contradictions;
6. retain `Unknown` for genuinely unproved parameters rather than inventing `Dynamic`;
7. specialize the complete callable and result type;
8. preserve canonical applied-type identity;
9. keep static family information even when a runtime variant carries only a subset of its parameters;
10. and remain independently verifiable by semantic and runtime tests.

`Either<L, R>` provides a compact but demanding reference model for these guarantees.

If the laws in this document hold for an ordinary user-defined `Either`, Phalcom's generic machinery demonstrates the essential semantics required for user-defined nominal generic ADTs, generic methods, higher-order generic functions, nested applied types, and contextual generic inference.
