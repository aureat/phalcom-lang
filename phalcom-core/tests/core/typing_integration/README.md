# `typing_integration` — Unified Typing Integration Conformance Package

This package is a user-level proving ground for composition across Phalcom's generic ADTs, constructor-kinded generics, type lambdas, generic inheritance, inherited method specialization, inference provenance, GADT refinement, and VM execution.

It does **not** claim to prove category-theoretic Functor/Applicative/Monad laws. Those abstractions are test vehicle, not subject of specification.

## Package architecture

`sources/either.ph` is sole canonical user-level `Either<L, R>` declaration.
`sources/monads.ph` defines `Box`, `Functor`, `Applicative`, `Monad`, and
`MonadAlgorithms`; root `support.rs` composes feature-isolated source layers.

Focused sub-suites:

- `either/` — direct generic ADT inference, substitution, rejection, and runtime.
- `monads/` — HKT, type-lambda, inheritance, provenance, rejection, and runtime.
- `expression/` — effectful GADT/HKT integration added by C4.

Each runtime fixture is semantically analyzed before compilation and VM execution.

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

[`LAWS.md`](./LAWS.md) is authoritative catalog. It maps GEN laws to
`docs/spec/typing-generics.md`, preserves every `MON-*` law, and reserves
`GEX-*`/`INT-*` for Expression integration.

Current Monad law families are:

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

Direct `Either` tests retain GEN law identifiers and source authority from the
typing-generics specification. They use same canonical source as Monad tests.

## Verification discipline

A positive inference test is incomplete unless it verifies the canonical semantic result and, where inference/dispatch is involved, the proof path that established it. The harness therefore prefers exact `DeclarationId`, `CallableId`, and `TypeParameterId` assertions, checks generic constraint relations/origins, and asserts expression readiness and selected callables.

Runtime tests are independent. The exact runtime fixture is first semantically analyzed, then compiled and executed, and exported primitive observations are asserted. Runtime success is never accepted as a substitute for correct static inference/provenance.

## Separation of contract and execution hierarchies

`ContractEitherMonad<E>` deliberately inherits generic contracts without
overriding them. It proves inherited `Functor`/`Applicative`/`Monad`
specialization.

`EitherMonad<E>` supplies executable overrides for VM tests. This keeps
inherited-method and concrete-override evidence distinct while allowing same
higher-kinded abstraction to execute end-to-end.
