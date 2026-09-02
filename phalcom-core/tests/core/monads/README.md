# `monads` — Higher-Kinded / Generic-Inheritance Conformance Package

This package is a user-level proving ground for Phalcom's constructor-kinded generics. It is intentionally analogous to the `either` package: one coherent Phalcom program is inspected through the semantic API and executed through the VM.

The package does **not** test whether `Functor`, `Applicative`, and `Monad` satisfy category-theory laws. They are useful pressure vessels for the type system.

## Source syntax under test

```ph
class Functor<F: Type -> Type> {
    map<A, B>(_ value: F<A>, _ f: (A) -> B) -> F<B> { ... }
}

class Applicative<F: Type -> Type> is Functor<F> { ... }
class Monad<F: Type -> Type> is Applicative<F> { ... }

class EitherMonad<E> is Monad<<X> =>> Either<E, X>> { ... }
```

`F` is a type constructor parameter. `<X> =>> Either<E, X>` is a source type lambda whose `X` is lambda-bound and whose `E` is captured from the enclosing declaration.

## Verification rule

A positive inference test is incomplete unless it checks both:

1. the canonical semantic result (`TypeId` / `TypeData`, binding type, declaration template, etc.); and
2. the explanation/proof path that established the result whenever inference or inherited receiver specialization is involved.

Runtime tests are independent: they compile and execute Phalcom code and assert exported primitive observations. Runtime success must not substitute for semantic proof assertions.

## Laws

### `MON-KIND-*` — kinds and constructor bounds

- **MON-KIND-01** — `F: Type -> Type` publishes a declaration parameter whose kind is exactly unary `Type -> Type`, not proper `Type`.
- **MON-KIND-02** — a unary nominal constructor such as `Box` can specialize `Monad<F>`.
- **MON-KIND-03** — a unary type lambda such as `<X> =>> Either<E, X>` can specialize the same parameter.
- **MON-KIND-04** — a proper type cannot satisfy a constructor-kinded parameter.
- **MON-KIND-05** — a constructor of incompatible kind/arity cannot satisfy the parameter.

### `MON-LAMBDA-*` — type-lambda integration

- **MON-LAMBDA-01** — `EitherMonad<E>` stores a real canonical `TypeData::Lambda` in its generic superclass template.
- **MON-LAMBDA-02** — the lambda's `X` is represented as a bound scoped node while enclosing `E` remains a free declaration-parameter form.
- **MON-LAMBDA-03** — specializing `EitherMonad<String>` substitutes the free `E` without rewriting/capturing `X`.
- **MON-LAMBDA-04** — applying the specialized constructor to `Int` beta-reduces to canonical `Either<String, Int>`.
- **MON-LAMBDA-05** — alpha-renamed equivalent constructor lambdas canonicalize identically where they are independently formed.

### `MON-INHERIT-*` — generic superclass projection

- **MON-INHERIT-01** — `Applicative<F> is Functor<F>` preserves the exact constructor argument.
- **MON-INHERIT-02** — `Monad<F> is Applicative<F>` preserves it again.
- **MON-INHERIT-03** — `EitherMonad<String>` projects to `Monad<<X> =>> Either<String, X>>`.
- **MON-INHERIT-04** — multi-hop projection reaches `Functor<<X> =>> Either<String, X>>` with the same constructor substitution.
- **MON-INHERIT-05** — an additional concrete subclass hop composes substitutions correctly.

### `MON-CALL-*` — inherited generic methods

- **MON-CALL-01** — inherited `Functor.map` specializes `F<A>`/`F<B>` through an `EitherMonad<String>` receiver.
- **MON-CALL-02** — receiver-owned `F` and method-owned `A`/`B` remain distinct binder identities.
- **MON-CALL-03** — a call solves `A` from the value argument and `B` from the closure result.
- **MON-CALL-04** — the explanation trace records the defining owner plus the complete receiver-specialization path.
- **MON-CALL-05** — methods introduced at `Applicative` and `Monad` specialize through the same hierarchy.

### `MON-SOLVE-*` — constructor-level constraint solving

- **MON-SOLVE-01** — `Monad<F>` receiver/argument evidence fixes the same `F` used by value arguments.
- **MON-SOLVE-02** — `F<A>` then solves the proper type parameter `A` without erasing constructor evidence.
- **MON-SOLVE-03** — `(A) -> F<B>` contributes compatible return-constructor evidence and solves `B`.
- **MON-SOLVE-04** — nested `List<F<A>>` occurrences are recursively decomposed.
- **MON-SOLVE-05** — repeated independent occurrences converge on one canonical solution.
- **MON-SOLVE-06** — direct constructor abstraction from `F<A> ~ Either<String, Int>` is a deliberate gap detector: if supported it must synthesize an equivalent unary constructor; if unsupported the outcome must be explicit and isolated rather than silently fabricated.

### `MON-COMP-*` — higher-order composition

- **MON-COMP-01** — `sequence` has result `F<List<A>>` when `F<A>` appears beneath `List`.
- **MON-COMP-02** — Kleisli composition propagates `F` through callable return positions.
- **MON-COMP-03** — `traverse` reconciles constructor evidence from `Monad<F>`, `List<A>`, `(A) -> F<B>`, and `F<List<B>>`.
- **MON-COMP-04** — the `Either` specialization yields `Either<ParseError, List<Int>>`.
- **MON-COMP-05** — its explanation trace shows independent evidence converging on the same constructor rather than a late `Dynamic` fallback.

### `MON-REJECT-*` — negative semantics

- **MON-REJECT-01** — `Int` cannot inhabit `F: Type -> Type`.
- **MON-REJECT-02** — an incompatible unsaturated constructor cannot be passed where unary `F` is required.
- **MON-REJECT-03** — constructor evidence from `EitherMonad<String>` conflicts with an unrelated unary constructor value.
- **MON-REJECT-04** — differing fixed parts of partially-applied `Either` constructors cannot be unified as one `F`.
- **MON-REJECT-05** — a genuinely unconstrained constructor parameter remains underconstrained and is not invented as `Dynamic`.

### `MON-RUNTIME-*` — VM execution

- **MON-RUNTIME-01** — concrete `EitherMonad.map` transforms `Right` and preserves `Left`.
- **MON-RUNTIME-02** — `pure` produces `Right`.
- **MON-RUNTIME-03** — `map2` combines two successes and preserves the failure path.
- **MON-RUNTIME-04** — `flatMap` chains successes and short-circuits failure.
- **MON-RUNTIME-05** — generic Kleisli composition executes through the monad contract.
- **MON-RUNTIME-06** — traversal produces the expected success and failure observations.

## Deliberate separation of concerns

The generic base hierarchy supplies the contracts that exercise inherited signatures. Phalcom currently has no interface/typeclass obligation saying an arbitrary `F` has a `map`/`flatMap` method, so those generic base bodies must not pretend otherwise. Concrete executable behavior is supplied by `EitherMonad<E>` overrides. This lets the package test inherited typing honestly while still executing real functional behavior in the VM.
