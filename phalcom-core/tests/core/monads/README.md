# `monads` — Higher-Kinded / Generic-Inheritance Conformance Package

This package is a user-level proving ground for Phalcom's constructor-kinded generics. It uses ordinary Phalcom `Functor`, `Applicative`, and `Monad` classes to force higher-kinded types, type lambdas, generic inheritance, inherited method specialization, inference provenance, and VM execution to interact in one coherent program.

It does **not** claim to prove the category-theoretic Functor/Applicative/Monad laws. Those abstractions are the test vehicle, not the subject of the specification.

## Source syntax under test

```ph
class Functor<F: Type -> Type> {
    map<A, B>(_ value: F<A>, _ f: (A) -> B) -> F<B> { ... }
}

class Applicative<F: Type -> Type> is Functor<F> { ... }
class Monad<F: Type -> Type> is Applicative<F> { ... }

class EitherMonad<E> is Monad<<X> =>> Either<E, X>> { ... }
```

`F` is a constructor-kinded type parameter. `<X> =>> Either<E, X>` is a unary type lambda: `X` is lambda-bound and `E` is captured from the enclosing declaration.

## Authoritative law catalog

[`LAWS.md`](./LAWS.md) is the authoritative catalog. It lists every implemented `MON-*` law, its required semantic/runtime evidence, and the exact Rust test function that exercises it.

The current law families are:

- `MON-KIND-*` — constructor kinds and kind rejection;
- `MON-LAMBDA-*` — binding, capture, substitution, beta reduction, alpha equivalence;
- `MON-INHERIT-*` — higher-kinded generic superclass projection;
- `MON-CALL-*` — inherited generic method specialization and proof paths;
- `MON-SOLVE-*` — constructor-level inference, abstraction, agreement, and nested solving;
- `MON-COMP-*` — real sequence, Kleisli, and traverse composition;
- `MON-BODY-*` — semantic verification inside generic algorithm bodies;
- `MON-OVERRIDE-*` — nearest specialized override selection;
- `MON-REJECT-*` — deterministic negative semantics;
- `MON-RUNTIME-*` — semantic preflight plus VM behavior and short-circuiting.

## Verification discipline

A positive inference test is incomplete unless it verifies the canonical semantic result and, where inference/dispatch is involved, the proof path that established it. The harness therefore prefers exact `DeclarationId`, `CallableId`, and `TypeParameterId` assertions, checks generic constraint relations/origins, and asserts expression readiness and selected callables.

Runtime tests are independent. The exact runtime fixture is first semantically analyzed, then compiled and executed, and exported primitive observations are asserted. Runtime success is never accepted as a substitute for correct static inference/provenance.

## Separation of contract and execution hierarchies

`ContractEitherMonad<E>` deliberately inherits the generic contracts without overriding them. It exists solely to prove inherited `Functor`/`Applicative`/`Monad` specialization.

`EitherMonad<E>` supplies executable overrides for VM tests. This prevents an inherited-method test from accidentally succeeding because a concrete override was selected, while still allowing the same higher-kinded abstraction to execute end-to-end.
