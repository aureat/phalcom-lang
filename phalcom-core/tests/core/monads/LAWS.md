# MON Laws — Higher-Kinded Generics, Type Lambdas, Generic Inheritance, and Monad Conformance

This document is the authoritative catalog for the laws exercised by `phalcom-core/tests/core/monads`.

The package uses `Functor`, `Applicative`, and `Monad` as a pressure vessel for Phalcom's type system. It does **not** claim to prove the category-theoretic Functor/Applicative/Monad laws. Its purpose is to prove that constructor-kinded generics, type lambdas, inheritance, generic method inference, proof provenance, and runtime execution compose correctly.

Source forms under test:

```ph
class Functor<F: Type -> Type> { ... }
class Applicative<F: Type -> Type> is Functor<F> { ... }
class Monad<F: Type -> Type> is Applicative<F> { ... }

class EitherMonad<E> is Monad<<X> =>> Either<E, X>> { ... }
```

## Verification standard

A positive semantic law is not satisfied merely because the program compiles. Where applicable, tests require all of the following:

1. the canonical result type is exact;
2. declarations, callables, and generic parameters are checked by canonical identity (`DeclarationId`, `CallableId`, `TypeParameterId`), not by leaf name alone;
3. the expression is `AnalysisStatus::Ready`;
4. the exact callable selected by semantic analysis is asserted;
5. receiver specialization paths are asserted by exact declaration identity;
6. generic solutions are asserted for the exact type-parameter identity;
7. generic constraint provenance and `GenericConstraintRelation` are asserted;
8. evidence status is asserted where the law depends on whether evidence is established or assumed;
9. no semantic errors or internal analyzer incidents are accepted in positive fixtures.

Runtime laws are separate. The exact Phalcom source executed by the VM is first passed through semantic analysis, then VM-visible primitive observations are asserted. Runtime success never substitutes for proof-path checks.

---

## `MON-KIND-*` — constructor kinds

### MON-KIND-01 — explicit unary constructor kind is preserved

`F: Type -> Type` publishes `F` with kind exactly `Type -> Type`, not proper `Type`.

Test: `kinds::functor_parameter_has_explicit_unary_constructor_kind`

### MON-KIND-02 — nominal unary constructor satisfies the bound

A nominal unary constructor such as `Box` may specialize `Monad<F>`.

Test: `kinds::unary_nominal_constructor_can_specialize_monad`

### MON-KIND-03 — unary type lambda satisfies the bound

`<X> =>> Either<E, X>` has kind `Type -> Type` and may specialize `Monad<F>`.

Test: `kinds::type_lambda_constructor_can_specialize_monad`

### MON-KIND-04 — proper type cannot inhabit constructor slot

`Monad<Int>` is rejected because `Int : Type`, not `Type -> Type`.

Expected diagnostic: `ApplicationArgumentKindMismatch`.

Test: `kinds::proper_type_cannot_specialize_unary_constructor_parameter`

### MON-KIND-05 — incompatible constructor arity cannot inhabit constructor slot

Unsaturated binary `Either` cannot inhabit unary `F: Type -> Type`.

Expected diagnostic: `ApplicationArgumentKindMismatch`.

Test: `kinds::binary_constructor_cannot_specialize_unary_constructor_parameter`

---

## `MON-LAMBDA-*` — type-lambda representation and normalization

### MON-LAMBDA-01 — superclass constructor argument is a canonical type lambda

`EitherMonad<E> is Monad<<X> =>> Either<E, X>>` stores a real `TypeData::Lambda`, not a string/syntax-only placeholder.

Test: `type_lambdas::either_monad_supertype_lambda_binds_x_and_captures_e`

### MON-LAMBDA-02 — bound and captured variables remain distinct

Inside `<X> =>> Either<E, X>`:

- `X` is represented by a bound scoped node;
- enclosing `EitherMonad.E` is represented as a free declaration-owned type form;
- the `Either` constructor is a free canonical type form.

Test: `type_lambdas::either_monad_supertype_lambda_binds_x_and_captures_e`

### MON-LAMBDA-03 — outer substitution is capture-avoiding

Specializing `EitherMonad<String>` replaces free `E` with `String` inside the lambda while preserving bound `X`.

Test: `type_lambdas::receiver_specialization_substitutes_free_outer_parameter_inside_lambda`

### MON-LAMBDA-04 — constructor application beta-reduces canonically

Applying the specialized constructor to `Int` yields canonical `Either<String, Int>`.

Test: `type_lambdas::specialized_constructor_beta_reduces_to_either_string_int`

### MON-LAMBDA-05 — alpha-renaming does not change semantic identity

These closed lambdas canonicalize to the same constructor type:

```ph
<X> =>> Either<String, X>
<Y> =>> Either<String, Y>
```

Test: `type_lambdas::alpha_renamed_constructor_lambdas_canonicalize_identically`

---

## `MON-INHERIT-*` — higher-kinded generic inheritance

### MON-INHERIT-01 — Applicative passes through the exact constructor parameter

`Applicative<F> is Functor<F>` preserves the exact declaration-owned `F` type form.

Test: `inheritance::generic_hierarchy_templates_preserve_constructor_parameter`

### MON-INHERIT-02 — Monad passes through the exact constructor parameter

`Monad<F> is Applicative<F>` preserves the exact declaration-owned `F` type form.

Test: `inheritance::generic_hierarchy_templates_preserve_constructor_parameter`

### MON-INHERIT-03 — concrete Either specialization projects into Monad

`EitherMonad<String>` establishes `F = <X> =>> Either<String, X>` when projected through the generic superclass hierarchy.

Test: `inheritance::either_monad_projects_exact_constructor_through_full_generic_hierarchy`

### MON-INHERIT-04 — multi-hop projection preserves the exact constructor

Projection through

```text
EitherMonad<String>
  -> Monad<...>
  -> Applicative<...>
  -> Functor<...>
```

retains the same unary constructor. The projected `Functor.F<Int>` is required to reduce to exact `Either<String, Int>`.

Test: `inheritance::either_monad_projects_exact_constructor_through_full_generic_hierarchy`

### MON-INHERIT-05 — substitution composes through an additional concrete hop

`StringEitherMonad is EitherMonad<String>` composes with the generic superclass chain without losing the captured `String` or lambda binder.

Test: `inheritance::concrete_subclass_hop_preserves_exact_higher_kinded_specialization`

---

## `MON-CALL-*` — inherited generic method specialization

The contract-only hierarchy deliberately does not override `map`, `pure`, or `flatMap`. Therefore these tests cannot accidentally pass by selecting `EitherMonad` implementations.

### MON-CALL-01 — inherited Functor.map specializes F through the receiver

Calling `map` on `StringContractEitherMonad` selects exact `Functor.map`, with receiver projection through:

```text
StringContractEitherMonad
 -> ContractEitherMonad
 -> Monad
 -> Applicative
 -> Functor
```

and yields `Either<String, Bool>` for an `Either<String, Int>` input and `(Int) -> Bool` transform.

Test: `inherited_methods::inherited_map_specializes_constructor_and_proves_method_generics`

### MON-CALL-02 — class-owned F and method-owned A/B retain distinct identities

`Monad.flatMap<A,B>` inference uses exact callable-owned `A`/`B`; neither can be confused with declaration-owned constructor parameter `F` or with same-named parameters in another scope.

Test: `inherited_methods::inherited_flat_map_keeps_class_and_method_generic_scopes_distinct`

### MON-CALL-03 — inherited method generics are solved from arguments

For inherited `map`, `A` is solved from the value argument and `B` from the transform argument/result, with exact constraint relations recorded.

Test: `inherited_methods::inherited_map_specializes_constructor_and_proves_method_generics`

### MON-CALL-04 — callable-selection proof records exact defining owner and path

The explanation graph must record the exact selected callable, concrete receiver, defining owner, and complete specialization path.

Tests:
- `inherited_methods::inherited_map_specializes_constructor_and_proves_method_generics`
- `inherited_methods::inherited_pure_specializes_through_applicative`
- `inherited_methods::inherited_flat_map_keeps_class_and_method_generic_scopes_distinct`

### MON-CALL-05 — methods introduced at different hierarchy levels specialize coherently

`Functor.map`, `Applicative.pure`, and `Monad.flatMap` all specialize through the same higher-kinded receiver hierarchy while retaining fresh method-owned generic parameters.

Tests:
- `inherited_methods::inherited_map_specializes_constructor_and_proves_method_generics`
- `inherited_methods::inherited_pure_specializes_through_applicative`
- `inherited_methods::inherited_flat_map_keeps_class_and_method_generic_scopes_distinct`

---

## `MON-SOLVE-*` — higher-kinded constructor constraint solving

### MON-SOLVE-01 — Monad<F> contributes constructor evidence

In generic `bind`, a `Monad<F>` argument contributes evidence for the same `F` used by later arguments.

Test: `inference::generic_bind_reconciles_all_constructor_evidence`

### MON-SOLVE-02 — F<A> solves A without erasing F

An `F<A>` argument contributes both constructor evidence for `F` and proper-type evidence for `A`.

Tests:
- `inference::generic_bind_reconciles_all_constructor_evidence`
- `inference::nested_list_of_f_a_contributes_constructor_and_element_evidence`

### MON-SOLVE-03 — callable return F<B> contributes compatible constructor evidence

A parameter `(A) -> F<B>` contributes both constructor evidence for `F` and proper-type evidence for `B`.

Test: `inference::generic_bind_reconciles_all_constructor_evidence`

### MON-SOLVE-04 — nested List<F<A>> is recursively decomposed

Constructor and element evidence is discovered even when `F<A>` occurs underneath `List`.

Test: `inference::nested_list_of_f_a_contributes_constructor_and_element_evidence`

### MON-SOLVE-05 — independent constructor constraints converge on one solution

`bind` must retain independent `F` constraints from:

- `Monad<F>`;
- `F<A>`;
- `(A) -> F<B>`.

The proof graph must contain those distinct constraint origins and one canonical `F` solution.

Test: `inference::generic_bind_reconciles_all_constructor_evidence`

### MON-SOLVE-06 — direct partial-constructor abstraction is supported

From only `F<A> ~ Either<String, Int>`, inference synthesizes an equivalent unary constructor:

```text
F = <X> =>> Either<String, X>
A = Int
```

The synthesized constructor must preserve `String`, retain a bound argument, and applying it to `Bool` must yield `Either<String, Bool>`.

Test: `inference::direct_f_a_constraint_synthesizes_exact_partial_either_constructor`

### MON-SOLVE-07 — nominal unary constructors are recovered directly

From `F<A> ~ Box<Int>`, inference solves:

```text
F = Box
A = Int
```

without unnecessarily manufacturing a type lambda.

Test: `inference::direct_f_a_constraint_recovers_nominal_box_constructor`

### MON-SOLVE-08 — synthesized constructor may capture caller-owned generics

From `Either<E, Int>` inside a generic caller, constructor abstraction preserves the caller-owned `E` as a free type while retaining the synthesized lambda's bound parameter.

Test: `inference::direct_constructor_abstraction_preserves_outer_generic_capture`

### MON-SOLVE-09 — two applied values can infer one constructor without a Monad anchor

Given:

```text
Either<String, Int>
Either<String, Bool>
```

against `F<A>` and `F<B>`, inference derives one shared constructor `<X> =>> Either<String, X>` and independent `A = Int`, `B = Bool` solutions without a `Monad<F>` parameter pre-solving `F`.

Test: `constructor_agreement::two_partial_either_arguments_infer_one_shared_constructor`

---

## `MON-COMP-*` — higher-order/nested composition

### MON-COMP-01 — real sequence preserves nested constructor typing

A real generic `sequence` implementation consumes `List<F<A>>`, uses `pure`, `flatMap`, and `map`, and returns `F<List<A>>`.

For `StringEitherMonad` and `List<Either<String, Int>>`, the result is exact `Either<String, List<Int>>`.

Test: `composition::sequence_specializes_nested_effects_to_either_of_list`

### MON-COMP-02 — Kleisli propagates F through callable return positions

Given:

```text
A -> F<B>
B -> F<C>
```

Kleisli composition returns `A -> F<C>` while preserving one constructor solution across both callable arguments.

Test: `composition::kleisli_composition_preserves_higher_kinded_callable_shape_and_proof`

### MON-COMP-03 — traverse reconciles structurally different evidence sources

`traverse` must reconcile constructor/proper-type evidence from:

```text
Monad<F>
List<A>
(A) -> F<B>
F<List<B>>
```

Test: `composition::traverse_specializes_to_either_of_list_and_records_independent_evidence`

### MON-COMP-04 — traverse beta-reduces the final nested result

For `StringEitherMonad`, `List<Int>`, and `(Int) -> Either<String, Bool>`, the final type is exact:

```text
Either<String, List<Bool>>
```

Test: `composition::traverse_specializes_to_either_of_list_and_records_independent_evidence`

### MON-COMP-05 — composition proof retains independent constructor evidence

Kleisli and traverse must publish multiple independently-originating constraints for the exact `F` parameter and one canonical solution, with no `Dynamic` fallback.

Tests:
- `composition::kleisli_composition_preserves_higher_kinded_callable_shape_and_proof`
- `composition::traverse_specializes_to_either_of_list_and_records_independent_evidence`

---

## `MON-BODY-*` — symbolic generic algorithm bodies

These laws inspect the implementation bodies while `F`, `A`, `B`, etc. are still symbolic. They prevent a correct-looking public signature from masking a body that type-checks through `Dynamic`, `Unknown`, or an unrelated generic parameter.

### MON-BODY-01 — bind body proves symbolic flatMap application

Inside generic `bind`, `monad.flatMap(value, next)` selects exact `Monad.flatMap`; its method-owned generics specialize to the outer `bind` `A` and `B` forms.

Test: `bodies::bind_body_proves_symbolic_flat_map_application`

### MON-BODY-02 — Kleisli body maps inner generics to outer B/C

The inner `Monad.flatMap` in Kleisli composition solves its own `A/B` exactly to the outer Kleisli `B/C` symbolic parameter forms.

Test: `bodies::kleisli_body_proves_symbolic_callable_composition`

### MON-BODY-03 — traverse resolves inherited capabilities while F is symbolic

Inside generic `traverse`:

- `pure` resolves through `Applicative`;
- `flatMap` resolves through `Monad`;
- `map` resolves through `Functor`;

all while the receiver is symbolic `Monad<F>`.

Test: `bodies::traverse_body_resolves_symbolic_inherited_capabilities`

---

## `MON-OVERRIDE-*` — concrete override selection

Phalcom's current relevant invariant is nearest same-selector override selection. This package does **not** invent a declaration-time "override signature must be a subtype" rule that is not currently ratified for ordinary classes.

### MON-OVERRIDE-01 — specialized map override wins

On `StringEitherMonad`, exact `EitherMonad.map` is selected instead of the inherited `Functor.map` contract stub, while method generics are still inferred correctly.

Test: `overrides::specialized_map_override_wins_and_keeps_generic_inference`

### MON-OVERRIDE-02 — specialized map2 override wins and remains generic

`EitherMonad.map2` is the exact selected callable and independently solves `A`, `B`, and `C`.

Test: `overrides::specialized_map2_override_wins_and_solves_three_generics`

### MON-OVERRIDE-03 — contract and executable hierarchies remain semantically distinct

The inherited `Functor.map` contract callable and concrete `EitherMonad.map` callable have distinct canonical identities. This ensures inheritance tests and runtime override tests cannot accidentally test the same method.

Test: `overrides::contract_and_concrete_hierarchies_resolve_to_distinct_callables`

---

## `MON-REJECT-*` — deterministic negative semantics

### MON-REJECT-01 — conflicting constructor families are rejected

`StringEitherMonad` establishes an `Either<String, _>` constructor. Passing `Box<Int>` / `(Int) -> Box<Bool>` to the same `bind` call must produce `GenericInferenceConflict`, mark the call invalid, and publish a `GenericConflict` explanation node.

Test: `rejection::monad_constructor_conflicts_with_unrelated_value_constructor`

### MON-REJECT-02 — different fixed parts of partial constructors cannot unify

`Either<String, Int>` and `Either<Bool, Bool>` cannot satisfy one common unary `F`.

Expected diagnostic: `GenericInferenceConflict` plus formal `GenericConflict` proof.

Test: `rejection::differing_fixed_either_arguments_cannot_unify_as_one_constructor`

### MON-REJECT-03 — unconstrained constructor variables are not fabricated

A constructor parameter that has no value-producing evidence must remain `Unknown(UnderconstrainedTypeVariable)`, produce `GenericInferenceUnderconstrained`, leave the call blocked, and never become `Dynamic`.

Test: `rejection::unconstrained_constructor_parameter_is_reported_not_invented`

### MON-REJECT-04 — proper Type is rejected from unary constructor slot

Alias of the negative kind property exercised by `MON-KIND-04`.

Test: `kinds::proper_type_cannot_specialize_unary_constructor_parameter`

### MON-REJECT-05 — incompatible constructor arity is rejected

Alias of the negative kind property exercised by `MON-KIND-05`.

Test: `kinds::binary_constructor_cannot_specialize_unary_constructor_parameter`

---

## `MON-RUNTIME-*` — executable Phalcom behavior

### MON-RUNTIME-00 — runtime fixture is semantically valid before execution

The exact source passed to the VM must parse/analyze without semantic errors or analyzer incidents.

Test: `runtime::runtime_fixture_is_semantically_valid_before_execution`

### MON-RUNTIME-01 — map transforms Right and preserves/short-circuits Left

- `Right(41)` mapped by `+1` produces `42`.
- `Left("boom")` is preserved.
- the mapping closure is not invoked on `Left`.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-02 — pure produces Right

`pure(7)` produces a successful value observed as `7`.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-03 — map2 combines success and short-circuits both failure sides

- two successes combine to `3`;
- left failure is preserved and the combiner is not invoked;
- right failure is preserved and the combiner is not invoked.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-04 — flatMap chains success and short-circuits failure

- successful flatMap produces the expected value;
- `Left` is preserved;
- the continuation is not invoked for `Left`.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-05 — generic Kleisli composition executes through Monad<F>

The generic Kleisli algorithm composes two `Either`-returning functions and produces the expected final successful value.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-06 — generic traverse produces expected success and failure values

Traversal over `[1,2,3]` produces the expected transformed list on success and preserves the first failure.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

### MON-RUNTIME-07 — traverse stops invoking transforms after failure

When traversal fails on element `2`, the transform runs only for elements `1` and `2`; element `3` is not transformed.

Test: `runtime::monad_higher_kinded_runtime_surface_produces_expected_values`

---

## Harness invariants

These are test-harness requirements rather than language laws:

- nominal declaration assertions compare exact `DeclarationId`;
- generic solution/constraint assertions prefer exact `TypeParameterId`;
- callable selection assertions compare exact `CallableId`, receiver `TypeId`, declaring `DeclarationId`, and specialization path;
- generic constraints assert both origin and `GenericConstraintRelation`;
- evidence status is asserted when material;
- positive call expressions assert `AnalysisStatus::Ready` and exact resolved callable/result type;
- helper-created parameter forms may not return a `TypeId` that exists only in a cloned store;
- negative fixtures assert exact error-code multisets rather than accepting any error;
- positive fixtures reject all semantic errors and internal analyzer incidents.

## Deliberately not asserted

### Formal `CallResolutionId` publication

`ExpressionAnalysis.call` exists as a scaffold but is not currently populated by the checker. The package therefore asserts the current authoritative products (`ExpressionAnalysis.callable`, type knowledge, status, and explanation graph) rather than turning HKT tests into an unrelated test of unfinished call-resolution publication.

### Declaration-time ordinary-class override compatibility

The current relevant Phalcom behavior is nearest same-selector override selection. The package tests that exact behavior. It does not assert a hypothetical rule requiring every ordinary-class override signature to be a subtype/equivalent specialization of its inherited signature unless such a rule is separately ratified and implemented.
