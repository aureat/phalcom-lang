# Walkthrough — `phalcom-core/tests/core/monads`

## 1. Purpose

The `monads` package is a compiler conformance suite for Phalcom's higher-kinded generic type system.

Its primary subject is **not** the mathematical Functor, Applicative, or Monad laws.

Instead, those abstractions are deliberately used as a demanding test vehicle for several compiler features that must work together:

- constructor-kinded type parameters;
- higher-kinded generic arguments;
- type lambdas;
- partial application of generic types;
- capture-avoiding substitution;
- beta reduction of type lambdas;
- alpha-equivalence/canonicalization;
- generic inheritance;
- superclass receiver specialization;
- inherited generic methods;
- distinction between class-owned and method-owned generic parameters;
- higher-order generic inference;
- reconstruction of type constructors from applied types;
- generic constraint solving;
- proof/explanation provenance;
- deterministic generic inference failure;
- generic algorithms whose types remain symbolic inside their bodies;
- concrete override selection;
- runtime execution of code using the same abstractions.

The central surface is:

```ph
class Functor<F: Type -> Type> {
    map<A, B>(
        _ value: F<A>,
        _ f: (A) -> B
    ) -> F<B> {
        ...
    }
}

class Applicative<F: Type -> Type> is Functor<F> {
    ...
}

class Monad<F: Type -> Type> is Applicative<F> {
    ...
}
```

and especially:

```ph
class EitherMonad<E> is Monad<<X> =>> Either<E, X>> {
    ...
}
```

That one inheritance clause forces the compiler to understand that:

```text
E
```

belongs to `EitherMonad`, while:

```text
X
```

belongs to the type lambda, and that:

```text
<X> =>> Either<E, X>
```

has kind:

```text
Type -> Type
```

rather than ordinary proper kind:

```text
Type
```

This is the key pressure point around which the package is built.

---

# 2. Package organization

The merged package is:

```text
phalcom-core/tests/core/monads/
├── README.md
├── LAWS.md
├── mod.rs
│
├── monads.ph
├── semantic_probes.ph
├── runtime_probes.ph
│
├── support.rs
│
├── kinds.rs
├── type_lambdas.rs
├── inheritance.rs
├── inherited_methods.rs
├── inference.rs
├── constructor_agreement.rs
├── composition.rs
├── bodies.rs
├── overrides.rs
├── rejection.rs
└── runtime.rs
```

The test module is registered from:

```text
phalcom-core/tests/core/mod.rs
```

The design has three major layers:

```text
Phalcom test program
        │
        ▼
semantic probes / runtime probes
        │
        ▼
Rust test harness inspecting compiler products
```

The Rust tests do not merely ask whether a Phalcom program happens to compile.

Where relevant they inspect the compiler's actual semantic structures and explanation graph.

---

# 3. `monads.ph` — the common language-level fixture

`monads.ph` is the foundation of the entire suite.

It contains the abstractions that every other test exercises.

## 3.1 `Either<L, R>`

The fixture defines:

```ph
enum Either<L, R> {
    @variant
    Left(_ value: L)

    @variant
    Right(_ value: R)

    ...
}
```

with:

```ph
fold
map
flatMap
```

This gives the test package a simple real two-parameter generic type from which a unary constructor can be constructed.

For example:

```text
Either<String, Int>
```

is a proper type.

But:

```ph
<X> =>> Either<String, X>
```

is a unary type constructor.

Conceptually:

```text
Either<String, _> : Type -> Type
```

This makes `Either` ideal for testing partial generic application.

---

## 3.2 `Box<T>`

The package also defines:

```ph
class Box<T> {}
```

`Box` is important because it distinguishes two cases:

```text
Box
```

is already a nominal unary constructor.

By contrast:

```text
Either<String, _>
```

must be represented through a type lambda.

This allows the test suite to ensure the compiler does not unnecessarily turn every unary constructor into a lambda.

For example:

```text
F<A> ~ Box<Int>
```

should infer:

```text
F = Box
A = Int
```

rather than synthesizing something like:

```ph
<X> =>> Box<X>
```

unless there is a reason to do so.

---

# 4. The Functor → Applicative → Monad hierarchy

The fixture defines:

```ph
class Functor<F: Type -> Type> {
    map<A, B>(
        _ value: F<A>,
        _ f: (A) -> B
    ) -> F<B> {
        throw Error.new("Functor.map is a contract stub")
    }
}
```

then:

```ph
class Applicative<F: Type -> Type> is Functor<F> {
    pure<A>(_ value: A) -> F<A> { ... }

    map2<A, B, C>(
        _ left: F<A>,
        _ right: F<B>,
        _ f: (A, B) -> C
    ) -> F<C> {
        ...
    }
}
```

and:

```ph
class Monad<F: Type -> Type> is Applicative<F> {
    flatMap<A, B>(
        _ value: F<A>,
        _ f: (A) -> F<B>
    ) -> F<B> {
        ...
    }
}
```

These contract implementations intentionally throw.

Their purpose is to give the semantic system real callable bodies and signatures without pretending that arbitrary `F` itself possesses methods such as `map`.

The hierarchy creates several simultaneous generic scopes.

For example:

```ph
class Monad<F: Type -> Type> {
    flatMap<A, B>(...)
}
```

contains:

```text
F     declaration-owned type parameter
A     flatMap-owned type parameter
B     flatMap-owned type parameter
```

That distinction is crucial.

A compiler that identifies generics only by names or by their textual appearance can easily confuse parameters across generic owners.

The MON tests therefore inspect the actual `TypeParameterId`s.

---

# 5. Two different Either-Monad hierarchies

One of the strongest design choices in the package is that there are deliberately **two** concrete-looking hierarchies.

## 5.1 Contract-only hierarchy

```ph
class ContractEitherMonad<E>
    is Monad<<X> =>> Either<E, X>> {}

class StringContractEitherMonad
    is ContractEitherMonad<String> {}
```

There are no overrides here.

Therefore:

```ph
monad.map(...)
```

on `StringContractEitherMonad` really must find:

```text
Functor.map
```

through:

```text
StringContractEitherMonad
→ ContractEitherMonad
→ Monad
→ Applicative
→ Functor
```

This hierarchy exists specifically for testing inherited specialization.

---

## 5.2 Executable hierarchy

Separately:

```ph
class EitherMonad<E>
    is Monad<<X> =>> Either<E, X>> {
    ...
}
```

implements real:

```text
map
pure
map2
flatMap
```

and:

```ph
class StringEitherMonad is EitherMonad<String> {}
```

provides a concrete runtime specialization.

This second hierarchy is used for:

- runtime execution;
- override selection;
- generic algorithms.

This separation prevents an important false positive.

Without it, a test claiming to verify inherited `Functor.map` could accidentally resolve directly to:

```text
EitherMonad.map
```

and completely bypass the inheritance machinery it was supposed to test.

The suite explicitly checks that the two callables have different canonical identities.

---

# 6. `MonadAlgorithms` — forcing higher-kinded inference

The fixture then provides algorithms written against abstract:

```text
Monad<F>
```

rather than directly against `Either`.

This is where the type system is really stressed.

---

## 6.1 `bind`

```ph
bind<F: Type -> Type, A, B>(
    _ monad: Monad<F>,
    _ value: F<A>,
    _ next: (A) -> F<B>
) -> F<B>
```

This is perhaps the most useful inference probe.

The same `F` occurs in three structurally different positions:

```text
Monad<F>
F<A>
(A) -> F<B>
```

If called with:

```text
StringEitherMonad
Either<String, Int>
(Int) -> Either<String, Bool>
```

the solver must reconcile all three and derive:

```text
F = <X> =>> Either<String, X>
A = Int
B = Bool
```

The suite checks not only that result.

It checks that the proof graph contains independent evidence for `F` from all three arguments.

---

## 6.2 `sameConstructor`

```ph
sameConstructor<F: Type -> Type, A, B>(
    _ left: F<A>,
    _ right: F<B>
) -> F<B>
```

This removes the `Monad<F>` anchor entirely.

Given:

```text
Either<String, Int>
Either<String, Bool>
```

the compiler must independently discover:

```text
F = <X> =>> Either<String, X>
A = Int
B = Bool
```

This is a very strong HKT test.

The compiler cannot simply copy `F` out of an explicit `Monad<F>` argument.

It has to infer the common constructor from two applied types.

---

## 6.3 `constructorIdentity`

```ph
constructorIdentity<F: Type -> Type, A>(
    _ value: F<A>
) -> F<A>
```

This tests constructor reconstruction from a single applied type.

For:

```text
Either<String, Int>
```

the solver must decompose:

```text
F<A>
~
Either<String, Int>
```

into:

```text
F = <X> =>> Either<String, X>
A = Int
```

For:

```text
Box<Int>
```

it must instead derive:

```text
F = Box
A = Int
```

Those two cases test the difference between:

- nominal constructor recovery;
- partial-constructor abstraction.

---

## 6.4 `sequence`

The package contains a real implementation:

```ph
sequence<F: Type -> Type, A>(
    _ monad: Monad<F>,
    _ values: List<F<A>>
) -> F<List<A>>
```

It actually invokes:

```text
pure
flatMap
map
```

while transforming:

```text
List<F<A>>
```

into:

```text
F<List<A>>
```

For the concrete `Either` specialization:

```text
List<Either<String, Int>>
```

becomes:

```text
Either<String, List<Int>>
```

This tests nested generic decomposition and reconstruction.

---

## 6.5 Kleisli composition

```ph
kleisli<F: Type -> Type, A, B, C>(
    _ monad: Monad<F>,
    _ first: (A) -> F<B>,
    _ second: (B) -> F<C>
) -> (A) -> F<C>
```

Here `F` appears inside callable return types.

That matters because an inference engine that handles:

```text
F<A>
```

at the top level may still fail to recursively find it in:

```text
(A) -> F<B>
```

The concrete test derives:

```text
(String) -> Either<String, Bool>
```

from:

```text
(String) -> Either<String, Int>
(Int) -> Either<String, Bool>
```

---

## 6.6 `traverse`

```ph
traverse<F: Type -> Type, A, B>(
    _ monad: Monad<F>,
    _ values: List<A>,
    _ transform: (A) -> F<B>
) -> F<List<B>>
```

This combines several difficult shapes:

```text
Monad<F>
List<A>
(A) -> F<B>
F<List<B>>
```

The concrete semantic test requires:

```text
F = <X> =>> Either<String, X>
A = Int
B = Bool
```

and therefore:

```text
F<List<B>>
=
Either<String, List<Bool>>
```

This is one of the best end-to-end semantic tests in the package.

---

# 7. `semantic_probes.ph`

`semantic_probes.ph` exists to create stable, named call sites for the Rust harness.

Examples include:

```ph
let mapped = monad.map(...)
```

```ph
let bound = MonadAlgorithms.bind(...)
```

```ph
let agreed = MonadAlgorithms.sameConstructor(...)
```

```ph
let sequenced = MonadAlgorithms.sequence(...)
```

```ph
let composed = MonadAlgorithms.kleisli(...)
```

```ph
let traversed = MonadAlgorithms.traverse(...)
```

This lets the Rust test harness locate:

- a specific callable;
- a specific binding;
- a specific expression;
- its inferred type;
- its selected callable;
- its explanation graph.

The semantic source used by those tests is effectively:

```text
monads.ph
+
semantic_probes.ph
```

---

# 8. `support.rs` — semantic test harness

`support.rs` is a major part of the package.

It deliberately tests semantic products rather than treating "no compiler error" as sufficient evidence.

The central type is:

```rust
Fixture
```

which:

1. parses the Phalcom source;
2. requires zero parser errors;
3. runs `analyze_single_module`;
4. requires zero internal analyzer incidents;
5. exposes the resulting semantic snapshot.

---

# 9. Exact declaration identity

For nominal types, the harness does not merely compare strings such as:

```text
"Either"
```

It constructs the expected:

```text
DeclarationId
```

and verifies that the canonical `TypeData::Nominal` refers to that exact declaration.

That protects the suite against same-named declarations in different semantic scopes/modules.

---

# 10. Exact generic identity

The harness retrieves real:

```text
TypeParameterId
```

values for:

- declaration generics;
- callable generics.

This is especially important for code such as:

```text
Functor<F>
Monad<F>
map<A,B>
flatMap<A,B>
bind<F,A,B>
```

where many different generic parameters have identical textual names.

A test should not prove merely:

```text
some "A" became Int
```

It proves:

```text
this exact callable-owned TypeParameterId became Int
```

---

# 11. Exact callable resolution

For a call expression, the harness can assert:

```text
AnalysisStatus::Ready
```

plus:

```text
exact CallableId
```

plus:

```text
exact result TypeId
```

For example, an inherited map call must resolve exactly to:

```text
Functor.map
```

while an executable concrete call must resolve exactly to:

```text
EitherMonad.map
```

These are separate semantic invariants.

---

# 12. Receiver specialization proof paths

The explanation graph is inspected for:

```text
CallableSelection
```

nodes.

The test can therefore require the exact path:

```text
StringContractEitherMonad
→ ContractEitherMonad
→ Monad
→ Applicative
→ Functor
```

rather than merely observing that the eventual return type happened to be correct.

This catches bugs where:

- the wrong method was selected;
- an intermediate generic substitution was skipped;
- a different owner happened to expose a compatible method.

---

# 13. Generic constraint provenance

The harness inspects:

```text
ExplanationStep::GenericConstraint
```

including both:

```text
GenericConstraintOrigin
```

and:

```text
GenericConstraintRelation
```

For example, it can prove that the `A` of `Functor.map` was constrained by argument 0:

```text
source: Either<String, Int>
```

and that `B` was constrained by argument 1:

```text
(Int) -> Bool
```

It is therefore testing not only the final inferred type but how the type system arrived there.

---

# 14. Evidence status

Generic solutions also record:

```text
EvidenceStatus
```

The tests distinguish cases such as:

```text
Assumed
```

and:

```text
Established
```

where that distinction is meaningful.

This is important for Phalcom because its semantic type model differentiates declared/static assumptions from facts established through inference or expression analysis.

---

# 15. `MON-KIND-*` — constructor kinds

Defined in:

```text
kinds.rs
```

These tests establish the basic kind system.

## MON-KIND-01

```ph
F: Type -> Type
```

must really be represented as:

```text
Type -> Type
```

and must not collapse to:

```text
Type
```

## MON-KIND-02

The nominal constructor:

```text
Box
```

is valid for:

```text
F: Type -> Type
```

## MON-KIND-03

The type lambda:

```ph
<X> =>> Either<E, X>
```

is also valid.

## MON-KIND-04

This is invalid:

```ph
Monad<Int>
```

because:

```text
Int : Type
```

rather than:

```text
Type -> Type
```

The expected diagnostic is specifically:

```text
ApplicationArgumentKindMismatch
```

## MON-KIND-05

This is also invalid:

```ph
Monad<Either>
```

because unsaturated `Either` has incompatible constructor arity.

Again the exact kind-mismatch diagnostic is required.

These are important because rejection tests ensure the kind system is not merely decorative metadata.

---

# 16. `MON-LAMBDA-*` — type lambda correctness

Defined in:

```text
type_lambdas.rs
```

This family directly inspects Phalcom's internal type-lambda representation.

## MON-LAMBDA-01

The superclass argument in:

```ph
Monad<<X> =>> Either<E, X>>
```

must be a real canonical:

```text
TypeData::Lambda
```

not a textual or syntax-only placeholder.

## MON-LAMBDA-02

The body must distinguish:

```text
Either    free canonical type
E         free enclosing declaration parameter
X         bound lambda variable
```

The test actually inspects scoped lambda nodes.

## MON-LAMBDA-03

Specializing:

```text
EitherMonad<String>
```

must rewrite free:

```text
E
```

inside the lambda to:

```text
String
```

while leaving bound:

```text
X
```

alone.

This is capture-avoiding substitution.

It is a particularly important compiler invariant.

## MON-LAMBDA-04

Applying the resulting constructor to:

```text
Int
```

must beta-reduce to:

```text
Either<String, Int>
```

## MON-LAMBDA-05

These must canonicalize to the same semantic constructor:

```ph
<X> =>> Either<String, X>
<Y> =>> Either<String, Y>
```

The name of a bound variable must not affect semantic identity.

That is alpha-equivalence.

---

# 17. `MON-INHERIT-*` — generic superclass specialization

Defined in:

```text
inheritance.rs
```

These tests move from standalone type lambdas into generic inheritance.

## MON-INHERIT-01

```ph
Applicative<F> is Functor<F>
```

must preserve the exact `Applicative.F` parameter.

## MON-INHERIT-02

Likewise:

```ph
Monad<F> is Applicative<F>
```

must preserve the exact `Monad.F`.

## MON-INHERIT-03 / 04

Starting with:

```text
EitherMonad<String>
```

the compiler projects all the way through:

```text
EitherMonad
→ Monad
→ Applicative
→ Functor
```

The final `Functor.F` must still be a unary constructor equivalent to:

```ph
<X> =>> Either<String, X>
```

The test then applies that constructor to `Int`.

It requires:

```text
Functor.F<Int>
=
Either<String, Int>
```

That final application is important: merely verifying that `F` is "some lambda" would be insufficient.

## MON-INHERIT-05

The package adds another hop:

```text
StringEitherMonad
→ EitherMonad<String>
→ Monad<...>
→ Applicative<...>
→ Functor<...>
```

and verifies that the captured:

```text
String
```

still survives.

---

# 18. `MON-CALL-*` — inherited generic calls

Defined in:

```text
inherited_methods.rs
```

These tests use:

```text
StringContractEitherMonad
```

specifically so there are no concrete overrides to interfere.

## Inherited `map`

Given:

```text
Either<String, Int>
(Int) -> Bool
```

the compiler must select exact:

```text
Functor.map
```

and produce:

```text
Either<String, Bool>
```

The test also checks:

```text
A = Int
B = Bool
```

for the exact generic parameters owned by `Functor.map`.

## Inherited `pure`

The receiver must project only as far as:

```text
Applicative
```

and:

```ph
monad.pure(42)
```

must become:

```text
Either<String, Int>
```

## Inherited `flatMap`

The test verifies:

```text
Monad.flatMap
```

directly.

It also explicitly checks that its:

```text
A
B
```

are callable-owned parameters belonging to that exact `flatMap`.

This guards against binder-scope confusion.

---

# 19. `MON-SOLVE-*` — HKT inference

The main tests are in:

```text
inference.rs
constructor_agreement.rs
```

This is the largest semantic family.

## MON-SOLVE-01..05 — `bind`

A single call must reconcile `F` evidence from:

```text
Monad<F>
F<A>
(A) -> F<B>
```

and produce one canonical constructor solution.

The explanation graph must contain multiple independent constraints for the exact same `F`.

This matters because the final type could accidentally look correct even if one source of evidence had been ignored.

---

## MON-SOLVE-04 — nested inference

The suite also feeds:

```text
List<F<A>>
```

to the solver.

This proves recursive decomposition through another generic constructor.

The inference engine must find `F<A>` even though it is nested inside `List`.

---

## MON-SOLVE-06 — constructor abstraction

This is one of the most interesting tests in the entire package.

From:

```text
F<A> ~ Either<String, Int>
```

the compiler must derive:

```text
F = <X> =>> Either<String, X>
A = Int
```

It then applies the inferred `F` to `Bool` and demands:

```text
Either<String, Bool>
```

So the test proves the inferred constructor is actually reusable.

---

## MON-SOLVE-07 — nominal constructor reconstruction

For:

```text
F<A> ~ Box<Int>
```

the solver must recover:

```text
F = Box
A = Int
```

This distinguishes nominal constructor inference from lambda synthesis.

---

## MON-SOLVE-08 — generic capture

Inside:

```ph
class CapturedConstructorInferenceProbe<E> {
    run(_ source: Either<E, Int>) {
        ...
    }
}
```

inference must synthesize approximately:

```ph
<X> =>> Either<E, X>
```

where:

```text
E
```

remains the exact caller-owned free parameter, and `X` is the new lambda binder.

This is another capture-avoidance test, but occurring during inference rather than declared inheritance.

---

## MON-SOLVE-09 — common constructor inference

Given:

```text
Either<String, Int>
Either<String, Bool>
```

against:

```text
F<A>
F<B>
```

the solver must independently infer:

```text
F = <X> =>> Either<String, X>
A = Int
B = Bool
```

There is no `Monad<F>` parameter in this algorithm.

This makes it a particularly pure test of constructor decomposition and agreement.

---

# 20. `MON-COMP-*` — composition

Defined in:

```text
composition.rs
```

These move beyond isolated inference examples into useful generic FP algorithms.

## MON-COMP-01 — sequence

For:

```text
StringEitherMonad
List<Either<String, Int>>
```

the generic:

```text
sequence
```

must produce:

```text
Either<String, List<Int>>
```

The test verifies both:

- final nested type;
- independent `F` constraints from the monad and values arguments.

---

## MON-COMP-02 — Kleisli

Given:

```text
String -> Either<String, Int>
Int -> Either<String, Bool>
```

the resulting callable must be exactly:

```text
String -> Either<String, Bool>
```

with:

```text
F = Either<String, _>
A = String
B = Int
C = Bool
```

---

## MON-COMP-03 / 04 — traverse

For:

```text
Monad<F>
List<Int>
(Int) -> F<Bool>
```

with:

```text
F = Either<String, _>
```

the result must beta-reduce all the way to:

```text
Either<String, List<Bool>>
```

---

## MON-COMP-05 — proof convergence

The composition tests also inspect that constructor evidence comes independently from different parameter positions and converges to one `F`.

They are intended to catch accidental fallback to:

```text
Dynamic
```

or an inference path that silently ignores some structural evidence.

---

# 21. `MON-BODY-*` — generic function bodies

Defined in:

```text
bodies.rs
```

This family is especially important.

Most generic tests validate calls *to* generic functions.

These tests inspect the implementation **inside** generic functions while their type parameters are still symbolic.

That catches an entirely different class of compiler bugs.

---

## MON-BODY-01 — generic `bind`

Inside:

```ph
bind<F, A, B>(...)
```

the expression:

```ph
monad.flatMap(value, next)
```

must resolve exact:

```text
Monad.flatMap
```

while its inner generic parameters map onto the outer symbolic parameters:

```text
flatMap.A = bind.A
flatMap.B = bind.B
```

They must not become:

```text
Dynamic
```

or unrelated fresh generic variables.

---

## MON-BODY-02 — generic Kleisli

Inside:

```ph
kleisli<F, A, B, C>
```

the internal:

```ph
monad.flatMap(...)
```

must map:

```text
flatMap.A = kleisli.B
flatMap.B = kleisli.C
```

This verifies correct symbolic generic-to-generic substitution.

---

## MON-BODY-03 — generic inherited capabilities

Inside `traverse`, while `F` is still symbolic:

```ph
monad.pure(...)
monad.flatMap(...)
monad.map(...)
```

must resolve respectively through:

```text
Applicative.pure
Monad.flatMap
Functor.map
```

with receiver:

```text
Monad<F>
```

This proves inheritance lookup itself works with an unresolved constructor parameter.

---

# 22. `MON-OVERRIDE-*` — concrete override selection

Defined in:

```text
overrides.rs
```

These tests switch from the contract-only hierarchy to:

```text
StringEitherMonad
```

## MON-OVERRIDE-01

```ph
monad.map(...)
```

must choose:

```text
EitherMonad.map
```

rather than inherited:

```text
Functor.map
```

It must still infer:

```text
A
B
```

correctly.

## MON-OVERRIDE-02

Likewise:

```text
EitherMonad.map2
```

must be selected and solve:

```text
A
B
C
```

independently.

## MON-OVERRIDE-03

The package explicitly proves:

```text
Functor.map CallableId
!=
EitherMonad.map CallableId
```

This is the invariant that keeps the inherited-method tests and executable tests semantically separated.

One thing the suite deliberately does **not** claim here is a declaration-time rule saying an ordinary-class override must have a subtype-equivalent signature.

That rule has not been encoded into this package as a language law.

What is tested is the behavior Phalcom currently relies on:

```text
nearest same-selector override wins
```

---

# 23. `MON-REJECT-*` — negative semantics

Defined primarily in:

```text
rejection.rs
```

with the kind failures in:

```text
kinds.rs
```

These tests are important because a powerful inference engine must fail correctly as well as succeed correctly.

---

## MON-REJECT-01 — unrelated constructors conflict

Suppose:

```text
Monad<F>
```

has established:

```text
F = Either<String, _>
```

but another argument supplies:

```text
Box<Int>
```

and:

```text
(Int) -> Box<Bool>
```

The solver must not:

- choose one arbitrarily;
- degrade to `Dynamic`;
- silently widen the result.

It must emit exactly:

```text
GenericInferenceConflict
```

and the explanation graph must contain:

```text
GenericConflict
```

---

## MON-REJECT-02 — incompatible partial constructors

These:

```text
Either<String, Int>
Either<Bool, Bool>
```

cannot share a constructor:

```text
F<_>
```

because their fixed left arguments disagree:

```text
String != Bool
```

The expected result is again:

```text
GenericInferenceConflict
```

with a formal conflict explanation.

---

## MON-REJECT-03 — truly underconstrained constructor

Consider:

```ph
fabricate<F: Type -> Type, A>(_ value: A) -> F<A>
```

and:

```ph
fabricate(42)
```

There is evidence for:

```text
A = Int
```

but none whatsoever for `F`.

The compiler must not invent a constructor.

The test requires:

```text
Unknown(UnderconstrainedTypeVariable)
```

and:

```text
GenericInferenceUnderconstrained
```

The call must be:

```text
Blocked
```

rather than `Ready`.

Most importantly:

```text
F
```

must not silently become:

```text
Dynamic
```

---

## MON-REJECT-04 / 05

These alias the kind-level negative laws:

```text
Int cannot inhabit Type -> Type
Either cannot inhabit unary Type -> Type
```

---

# 24. `runtime_probes.ph` — execution-level conformance

Static correctness is only half the package.

The same abstractions are also run through the actual Phalcom compiler and VM.

The executable monad is:

```text
StringEitherMonad
```

The runtime fixture exposes primitive module-level observations so the Rust test does not need to interpret complex Phalcom objects directly.

---

# 25. Semantic preflight before runtime

Before runtime results count as evidence, the exact source passed to the VM is independently parsed and semantically analyzed.

That is:

```text
MON-RUNTIME-00
```

This prevents a VM test from succeeding through some runtime behavior while static semantic analysis is already invalid.

Thus:

```text
semantic correctness
```

and:

```text
runtime correctness
```

remain separate requirements.

---

# 26. Runtime `map`

The runtime suite checks:

```ph
Right(41).map(+1)
```

produces:

```text
Right(42)
```

It also maps over:

```text
Left("boom")
```

and checks:

```text
Left("boom")
```

is preserved.

But importantly, it does more than check the returned value.

A mutable counter verifies that the mapping closure is invoked:

```text
0 times
```

on `Left`.

That proves short-circuit behavior.

---

# 27. Runtime `pure`

```ph
runtimeMonad.pure(7)
```

must produce a successful `Either` containing:

```text
7
```

---

# 28. Runtime `map2`

Success:

```text
Right(1)
Right(2)
```

with addition produces:

```text
Right(3)
```

The test separately checks:

```text
Left failure on argument 1
```

and:

```text
Left failure on argument 2
```

and in both cases verifies that the combining closure is never executed.

So this is testing both:

```text
result semantics
```

and:

```text
evaluation semantics
```

---

# 29. Runtime `flatMap`

For a successful input:

```text
Right(41)
```

the continuation executes.

For:

```text
Left("boom")
```

the continuation must not execute.

Again this is verified with a side-effect counter rather than inferred from the return value alone.

---

# 30. Runtime Kleisli composition

Two functions:

```text
Int -> Either<String, Int>
Int -> Either<String, Bool>
```

are composed through generic:

```text
MonadAlgorithms.kleisli
```

and executed.

The resulting callable is invoked with:

```text
41
```

and must reach:

```text
Right(true)
```

This verifies that a higher-kinded generic algorithm is not merely statically expressible—it actually executes through the concrete monad specialization.

---

# 31. Runtime traverse

The successful case traverses:

```ph
[1, 2, 3]
```

through a transform adding ten.

The resulting values must be exactly:

```text
[11, 12, 13]
```

The failure case deliberately fails on:

```text
2
```

and records how many times the transform was called.

Expected count:

```text
2
```

not:

```text
3
```

Therefore traversal must stop processing after the effect has failed.

---

# 32. What the runtime test ultimately observes

`runtime.rs` reads module slots such as:

```text
monadMappedRightValue
monadMappedLeftPreserved
monadMapLeftShortCircuited

monadPureValue

monadMap2Value
monadMap2FailurePreserved
monadMap2LeftShortCircuited
monadMap2RightFailurePreserved
monadMap2RightShortCircuited

monadFlatMapValue
monadFlatMapFailurePreserved
monadFlatMapShortCircuited

runtimeKleisliValue

runtimeTraverseSuccessValue
runtimeTraverseFailurePreserved
runtimeTraverseShortCircuited
```

and finally:

```text
runtimeAll
```

must be:

```text
true
```

---

# 33. Overall law catalog

`LAWS.md` is the authoritative inventory.

The current law namespaces are:

```text
MON-KIND-01 .. MON-KIND-05
MON-LAMBDA-01 .. MON-LAMBDA-05
MON-INHERIT-01 .. MON-INHERIT-05
MON-CALL-01 .. MON-CALL-05
MON-SOLVE-01 .. MON-SOLVE-09
MON-COMP-01 .. MON-COMP-05
MON-BODY-01 .. MON-BODY-03
MON-OVERRIDE-01 .. MON-OVERRIDE-03
MON-REJECT-01 .. MON-REJECT-05
MON-RUNTIME-00 .. MON-RUNTIME-07
```

That is 53 catalogued `MON-*` law identifiers.

Some identifiers intentionally share a physical Rust test where a single scenario proves several closely related invariants.

For example:

```text
MON-REJECT-04
MON-REJECT-05
```

refer back to the corresponding kind-rejection tests.

---

# 34. What a positive MON test actually means

A positive semantic test generally requires substantially more than:

```text
program did not produce an error
```

Depending on the law, it may establish:

```text
1. exact canonical result TypeId

2. exact DeclarationId

3. exact CallableId

4. exact TypeParameterId

5. AnalysisStatus::Ready

6. exact receiver TypeId

7. exact declaring owner

8. exact inheritance/specialization path

9. exact generic solution

10. exact constraint origin

11. exact GenericConstraintRelation

12. appropriate EvidenceStatus

13. no semantic error diagnostics

14. no internal analyzer incidents
```

This is what makes the package useful as a compiler test rather than merely a syntax demo.

---

# 35. What a negative MON test means

Negative tests similarly do not accept:

```text
"some error occurred"
```

The harness compares exact diagnostic-code multisets.

For example:

```text
GenericInferenceConflict
```

must not accidentally become:

```text
ApplicationArgumentKindMismatch
```

or:

```text
GenericInferenceUnderconstrained
```

Similarly, where appropriate, the expression state is checked:

```text
Invalid
```

versus:

```text
Blocked
```

and the explanation graph is expected to expose the corresponding semantic reason.

---

# 36. Compiler regressions this package is designed to detect

The package should detect regressions such as:

### Kind erasure

```text
F: Type -> Type
```

accidentally collapsing into ordinary:

```text
Type
```

### Opaque lambda substitution

A specialization mechanism that leaves:

```ph
<X> =>> Either<E, X>
```

untouched when specializing:

```text
E = String
```

### Lambda capture bugs

Replacing or capturing the lambda-bound:

```text
X
```

while substituting the outer:

```text
E
```

### Broken alpha canonicalization

Treating:

```ph
<X> =>> ...
```

and:

```ph
<Y> =>> ...
```

as distinct semantic types.

### Broken superclass propagation

Losing `F` while traversing:

```text
Monad
→ Applicative
→ Functor
```

### Wrong inherited callable selection

Choosing a concrete override when an inheritance test requires:

```text
Functor.map
```

or vice versa.

### Generic binder collision

Confusing:

```text
Monad.F
```

with:

```text
flatMap.A
```

or confusing same-named generic parameters belonging to different callables.

### Shallow generic inference

Handling:

```text
F<A>
```

but failing to inspect:

```text
List<F<A>>
```

or:

```text
(A) -> F<B>
```

### Failure to reconstruct higher-kinded constructors

Being unable to infer:

```text
F = Either<String, _>
```

from:

```text
Either<String, Int>
```

### Excessive lambda synthesis

Representing simple:

```text
Box
```

as a synthesized lambda when a nominal constructor is already available.

### Failure to reconcile independent constructor evidence

Allowing different appearances of `F` to infer incompatible constructors without reporting a conflict.

### Dynamic escape during inference

Falling back to:

```text
Dynamic
```

when a constructor variable is underconstrained or conflicting.

### Public-signature-only correctness

Accepting:

```ph
bind<F,A,B>(...) -> F<B>
```

while its actual body internally resolves calls incorrectly.

The `MON-BODY-*` family exists specifically for this class of bug.

### Runtime dispatch mismatch

Static analysis selecting one abstraction while VM execution behaves according to a different method.

### Incorrect short-circuit semantics

Evaluating mapping/continuation functions after an `Either.Left` has already determined the result.

---

# 37. What the package deliberately does not test

## Mathematical Monad laws

It currently does **not** attempt to prove equations such as:

```text
left identity
right identity
associativity
```

nor Functor laws such as:

```text
identity
composition
```

nor Applicative laws such as:

```text
identity
homomorphism
interchange
composition
```

Those would be a separate behavioral/algebraic suite.

The current use of the term `monads` refers to the abstraction being used to stress the compiler.

---

## Formal `CallResolutionId`

The semantic model has:

```text
ExpressionAnalysis.call
```

but that product is not currently populated authoritatively by the checker.

Therefore the suite tests currently authoritative products:

```text
ExpressionAnalysis.callable
ExpressionAnalysis.status
type knowledge
explanation graph
```

rather than creating an unrelated failure around unfinished call-resolution publication.

---

## Hypothetical override-subtyping rules

The package does not assert a language rule saying:

```text
every ordinary-class override signature must be a subtype-equivalent
specialization of the inherited signature
```

because that is not the behavior currently being specified by these tests.

The tested invariant is:

```text
nearest matching concrete override is selected
```

---

# 38. Conceptual testing pipeline

The whole package can be understood as this progression:

```text
        F: Type -> Type
              │
              ▼
       kind representation
              │
              ▼
       type-lambda model
              │
              ▼
    capture / substitution
              │
              ▼
       beta reduction
              │
              ▼
     generic inheritance
              │
              ▼
 inherited method resolution
              │
              ▼
 constructor-level inference
              │
              ▼
 independent proof convergence
              │
              ▼
  symbolic generic function bodies
              │
              ▼
 concrete override resolution
              │
              ▼
 deterministic rejection
              │
              ▼
       actual VM execution
```

A regression at almost any point in that chain should cause a relatively localized `MON-*` law to fail.

---

# 39. The most important tests

If only a handful of cases were inspected when debugging the higher-kinded implementation, these are particularly valuable.

## `MON-LAMBDA-03`

```text
receiver_specialization_substitutes_free_outer_parameter_inside_lambda
```

Tests capture-avoiding specialization through a type lambda.

A failure here indicates the compiler cannot safely specialize partially applied generic types.

---

## `MON-INHERIT-04`

```text
EitherMonad<String>
→ Monad
→ Applicative
→ Functor
```

followed by:

```text
F<Int> = Either<String, Int>
```

Tests the complete generic-inheritance substitution chain.

---

## `MON-SOLVE-06`

```text
F<A> ~ Either<String, Int>
```

must reconstruct:

```text
F = Either<String, _>
```

This is a central HKT inference capability.

---

## `MON-SOLVE-09`

Infers one constructor from:

```text
Either<String, Int>
Either<String, Bool>
```

without an explicit `Monad<F>` anchor.

This strongly tests structural higher-kinded inference.

---

## `MON-COMP-03/04`

`traverse` combines:

```text
Monad<F>
List<A>
(A) -> F<B>
F<List<B>>
```

into one inference problem.

This is close to the kind of complexity real functional APIs generate.

---

## `MON-BODY-03`

Verifies:

```text
pure
flatMap
map
```

inside a generic algorithm while `F` is still symbolic.

This tests semantic behavior that call-site-only tests cannot cover.

---

## `MON-REJECT-03`

Makes sure missing constructor evidence remains formally:

```text
Unknown(UnderconstrainedTypeVariable)
```

instead of being hidden behind `Dynamic`.

This protects Phalcom's proof discipline.

---

## `MON-RUNTIME-07`

Confirms failure propagation has actual evaluation consequences:

```text
traverse
```

stops invoking transforms after the first failure.

---

# 40. Summary

The `monads` package is best thought of as an integration test for Phalcom's advanced generic semantic machinery.

It begins with:

```ph
F: Type -> Type
```

and follows that constructor through virtually every compiler phase where it can become difficult:

```text
kind checking
→ canonical type representation
→ type-lambda binding
→ free-variable capture
→ substitution
→ beta reduction
→ generic inheritance
→ inherited method lookup
→ method-generic specialization
→ constraint generation
→ higher-order unification
→ constructor reconstruction
→ nested generic composition
→ symbolic body analysis
→ override dispatch
→ diagnostic generation
→ runtime execution
```

The most important property of the suite is that it generally refuses to equate:

```text
"the final type looked right"
```

with:

```text
"the type system proved it correctly"
```

It inspects both the semantic result and the path used to establish that result.

That makes `phalcom-core/tests/core/monads` a conformance package not merely for higher-kinded syntax, but for the interaction between Phalcom's declaration model, canonical type representation, inference engine, proof/explanation machinery, inheritance specialization, and runtime execution.