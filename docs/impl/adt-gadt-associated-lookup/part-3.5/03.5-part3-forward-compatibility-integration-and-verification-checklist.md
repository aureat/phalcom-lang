# Phalcom ADT/GADT + Associated Lookup
## Part 3.5 — Part 3 Integration and Forward-Compatibility Verification Checklist

**Status:** Implementation review gate / companion checklist  
**Series:** ADT/GADT + Associated Lookup, Part 3.5  
**Purpose:** keep Part 3 Option-A semantics compatible with future declaration-polymorphic Option B  
**Not an Option B implementation plan:** this checklist must not cause `forall` to be implemented in Part 3  
**Repository baseline checked:** `feat/adts` at `2c8b5840fc5a864968cb2a832540fbcba868d9f8`

---

# 1. How to Use This While Part 3 Is Being Implemented

Part 3 implementation may proceed immediately.

This checklist is a parallel review gate, not a prerequisite project.

For every Part 3 task that touches associated generics, family types, captured denotation, exact variant constructors, or call inference:

```text
implement the Part 3 behavior
    ↓
check the relevant Part 3.5 invariants
    ↓
continue
```

If the Part 3 implementation plan still describes G1 as an unresolved decision gate, this document closes it:

```text
G1 = Option A for Part 3
```

Do not stop implementation to design Option B.

Do stop and refactor if Part 3 code introduces an assumption that would require associated-resolution redesign merely to add universal value schemes later.

---

# 2. Part 3 Semantic Contract

The implementation must satisfy all of the following.

- [ ] `Option<Int>::Some::(_)` may reify a concrete exact constructor reference.
- [ ] `Option<Int>::Some::*` may reify a concrete family.
- [ ] An expected callable type may specialize `Option::Some::(_)` before expression publication.
- [ ] An expected family type, when available internally, may specialize `Option::Some::*` before expression publication.
- [ ] Bare `Option::Some::(_)` is rejected if owner declaration generics remain unresolved at escape/finalization.
- [ ] Bare `Option::Some::*` is rejected on the same basis.
- [ ] Bare generic singleton `Option::None` is rejected when its exact case type still contains unresolved owner parameters.
- [ ] Exact reification of a member-local generic callable is also concrete under Option A: if its member-local binders cannot be solved contextually, reification is underconstrained rather than silently publishing a polymorphic callable.
- [ ] Underconstrained reification does not degrade to `Dynamic`, `Object`, or `Any`.
- [ ] Underconstrained reification does not create an existential package implicitly.
- [ ] Underconstrained reification does not publish solver metavariables into binding state.
- [ ] No stored family/member is specialized by its first later invocation.
- [ ] Direct `Option::Some(42)` remains inferable from invocation arguments.
- [ ] Expected result type remains available as generic-call evidence.
- [ ] GADT equalities participate in direct invocation specialization before the result is published.

---

# 3. Schema/View Separation Gate

Part 3 must retain immutable generic declaration semantics and derive specialized views from them.

- [ ] There is a canonical declaration/member schema or equivalent canonical product containing declaration-owned generic information.
- [ ] Owner type parameters are represented by stable canonical parameter identities, not anonymous solver variables.
- [ ] Callable-local generic parameters retain a distinguishable owner from declaration-owned parameters.
- [ ] Parameter kinds remain available after associated-surface construction.
- [ ] Declaration `where` constraints remain available after associated-surface construction.
- [ ] GADT result equalities remain available after enum/variant declaration processing.
- [ ] Specializing a schema to `Int` does not mutate the schema.
- [ ] A later independent specialization to `String` sees the original schema rather than the prior `Int` substitution.
- [ ] Specialized parameter/result `TypeId`s are products of substitution/materialization, not replacements for generic source truth.
- [ ] Residual unsolved declaration parameters can be enumerated when a Part 3 underconstraint diagnostic is emitted.

Recommended conceptual invariant:

```text
schema + substitution/environment -> view
```

Never:

```text
schema mutated in place -> permanently specialized declaration
```

---

# 4. Inference Variable Containment Gate

Solver-local metavariables must remain solver-local.

- [ ] Fresh inference variables created while checking an associated invocation do not escape the inference session.
- [ ] Fresh inference variables used during contextual reification are fully solved before a `TypedExpression`/`ExpressionAnalysis` value is published as Ready.
- [ ] If such variables remain unsolved, the expression is blocked/invalid according to the canonical underconstraint policy.
- [ ] Binding flow state never stores a family/member whose formal type contains a solver-local metavariable intended to be fixed later.
- [ ] Re-analyzing uses in a different source order cannot change the type assigned to the original binding.

A useful negative test is conceptually:

```phalcom
const some = Option::Some::(_)

const a = some(1)
const b = some("x")
```

Part 3 must reject the `some` declaration itself rather than choosing the first call's type.

---

# 5. Family Type Gate

The family type must remain a structural operation capability type.

- [ ] Family types are not encoded as unions of callable/member types.
- [ ] Getter `#name` and method `#name()` remain distinct operation keys.
- [ ] Setter identity remains distinct from both getter and method identity.
- [ ] Subscript get and subscript set remain distinct.
- [ ] Operator selectors reuse the canonical selector model.
- [ ] Singleton variants are representable as value members, not fake zero-argument callables.
- [ ] Zero-argument variant constructors remain callable members distinct from singleton variants.
- [ ] Family member types preserve enough exact-case information for variant constructors.
- [ ] Structural subtype/equivalence logic operates over operation requirements, not `AssociatedFamilyId` equality.
- [ ] Nominal associated family identity is retained separately from structural family type.
- [ ] Family type interning/canonicalization does not require the source associated base name to be the sole identity key.

Part 3.5 intentionally leaves the future source-level base-sensitive versus base-erased relation open, with a current preference for base-erased capability typing. The internal model must preserve enough information to choose later.

---

# 6. Captured Associated Value Gate

A first-class family/member is more than its structural `TypeId`.

Verify that the formal semantic product or denotation can retain:

- [ ] nominal associated family/member identity;
- [ ] lookup owner declaration;
- [ ] owner type form / concrete specialization;
- [ ] defining declaration for inherited behavioral members;
- [ ] access-filtered exact member set;
- [ ] exact `CallableId`, `VariantId`, or `VariantConstructorId` as applicable;
- [ ] specialized member types;
- [ ] enough information for Part 4 to lower without re-resolving associated syntax.

A later call on a stored family should consume the captured capability/value plus its family type. It should not repeat lexical associated visibility lookup.

---

# 7. Generic Ownership Gate

Check at least these shapes.

## 7.1 Owner-only generic

```phalcom
class Box<T> {
    @class
    make(_ value: T) -> Box<T>
}
```

The associated schema depends on declaration parameter `T`.

- [ ] `Box<Int>::make::(_)` specializes `T = Int`.
- [ ] bare `Box::make::(_)` reports residual declaration parameter `T` under Option A when no context specializes it.

## 7.2 Callable-local generic

```phalcom
class Factory {
    @class
    make<T>(_ value: T) -> Box<T>
}
```

- [ ] callable-local `T` is not confused with an owner type parameter.
- [ ] the existing generic callable model remains authoritative for **direct invocation**.
- [ ] exact/member-family reification does not assume that invocation-time generic inference automatically makes the stored value polymorphic.
- [ ] if the current first-class callable/family type cannot carry a member-local generic scheme, that binder must be solved contextually before escape or the reification is underconstrained.
- [ ] Part 3 does not invent `forall` semantics merely because the declaration itself is generic.

## 7.3 Owner plus callable-local generic

```phalcom
class Factory<E> {
    @class
    make<T>(_ value: T) -> Result<Box<T>, E>
}
```

- [ ] owner `E` and callable `T` have different canonical owners.
- [ ] owner specialization and callable inference can be applied in the correct order.
- [ ] no flattened anonymous parameter vector loses that distinction.

## 7.4 Higher-kinded declaration parameter

```phalcom
class Mapper<F: Type -> Type> { ... }
```

- [ ] residual generic reporting retains `F : Type -> Type` rather than assuming `Type`.

---

# 8. GADT Gate

Given:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

verify:

- [ ] `Expr::Int(1)` can infer the result index from the exact variant declaration.
- [ ] `Expr<Int>::Int(1)` is compatible.
- [ ] `Expr<String>::Int(1)` is rejected as an owner/GADT contradiction.
- [ ] exact constructor reification retains the exact case result template.
- [ ] concrete reification does not erase the theorem/equality evidence used to prove the exact result.
- [ ] future universal wrapping would be able to use the same immutable result schema; no GADT-specific `forall` redesign should be required.

---

# 9. Contextual Reification Gate

Option A is contextual, not “explicit type arguments only.”

Check the reification path can receive expected type information early enough to solve residual owner and member-local parameters that must be concrete under Option A.

- [ ] exact associated callable analysis accepts an `ExpectedType` or equivalent bidirectional context.
- [ ] whole-family reification can use an expected concrete family type when one is available internally.
- [ ] expected result/callable constraints are added before deciding that reification is underconstrained.
- [ ] underconstraint is emitted only after the contextual solving attempt.
- [ ] a failed contextual relation is reported as a real type conflict when contradictory evidence exists, not downgraded to mere underconstraint.

Distinguish:

```text
no evidence for T
    underconstrained

context proves T = Int and T = String
    conflict
```

---

# 10. Exact Case Representation Gate

Part 3 may use internal `TypeData::ExactCase` or the actual Part 2 equivalent; source syntax is deferred.

Verify:

- [ ] exact case type contains stable variant identity;
- [ ] exact case type retains the specialized enum/root type;
- [ ] exact case is a subtype of the specialized enum root;
- [ ] singleton exact case and zero-argument constructor exact case remain distinct identities;
- [ ] different payload selector shapes remain distinct exact cases;
- [ ] GADT index contradiction can be detected before a malformed exact case type is published.

Future candidate source spellings documented by Part 3.5:

```phalcom
Case<Option<Int>::None>
Case<Option<Int>::None()>
Case<Option<Int>::Some(_)>
Case<Expr<Int>::Int(_)>
```

Do not implement this grammar as part of Part 3 unless separately requested.

---

# 11. Future `forall` Extension-Point Gate

Part 3 does not implement universal value types, but code organization should leave a natural place for them.

A future implementation should be able to add something conceptually equivalent to:

```text
TypeScheme {
    binders,
    body
}
```

or:

```text
UniversalType {
    binders,
    body
}
```

without changing:

- [ ] `VariantId` meaning;
- [ ] `VariantConstructorId` meaning;
- [ ] `AssociatedFamilyId` meaning;
- [ ] static associated lookup semantics;
- [ ] family effective-member computation;
- [ ] exact selector identity;
- [ ] GADT exact-case identity;
- [ ] Part 4's semantic target handoff.

Future universal syntax direction:

```phalcom
forall<T> (T) -> Option<T>
forall<T, E> (T, E) -> Result<Box<T>, E>
```

Future family member schemes:

```phalcom
Family<#{
    convert(_): forall<T> (T) -> Box<T>
    convert(_,or): forall<T, E> (T, E) -> Result<Box<T>, E>
}>
```

These examples are design tests, not parser requirements.

---

# 12. Source-Syntax Non-Dependency Gate

Part 3 implementation must not depend on having source syntax for full family types.

- [ ] `FamilyType` can be constructed directly by semantic code.
- [ ] family subtyping/equality can be tested directly in Rust semantic tests.
- [ ] family hover/debug presentation can be deterministic without being parseable source.
- [ ] no parser work is required merely to validate structural family semantics.
- [ ] no `forall` grammar is added to satisfy Part 3.
- [ ] no `Case<...>` grammar is added to satisfy Part 3.

The preferred future source representation is still useful as a design constraint:

```phalcom
Family<#{
    None: Case<Option<Int>::None>
    None(): () -> Case<Option<Int>::None()>
    Some(_): (Int) -> Case<Option<Int>::Some(_)>
}>
```

If the internal model cannot faithfully represent this hypothetical type, it is probably too weak.

---

# 13. Diagnostics Gate

Under Option A, diagnostics should explain the missing specialization rather than describing the declaration as unknown.

- [ ] exact reification underconstraint identifies the associated member/family.
- [ ] diagnostic identifies residual declaration parameter names when available.
- [ ] diagnostic can mention the residual parameter kind when useful.
- [ ] guidance suggests explicit owner specialization when appropriate.
- [ ] guidance may suggest a contextual annotation.
- [ ] no guidance suggests “call it once to infer the type.”
- [ ] dynamic-boundary wording is not used unless a genuine Dynamic value is involved.
- [ ] owner/GADT contradiction has a different diagnostic path from underconstraint.

Suggested conceptual examples:

```text
associated.generic.underconstrained
associated.generic.owner_conflict
associated.gadt.owner_conflict
```

Use the repository's actual diagnostic-code conventions when implementing.

---

# 14. Explanation/Proof Gate

The explanation graph should make the Option A decision inspectable.

Where practical, preserve nodes/evidence for:

- [ ] associated owner resolution;
- [ ] exact family/member selection;
- [ ] declaration binder specialization;
- [ ] contextual generic constraint;
- [ ] GADT equality application;
- [ ] residual unresolved declaration parameter;
- [ ] final reification success or underconstraint failure.

Do not treat advisory family shape as formal proof evidence.

---

# 15. Source Index / LSP Gate

Source/editor products should consume formal semantic targets.

- [ ] associated syntax tokens are indexed syntactically without trying to perform semantic family resolution in the source-index builder.
- [ ] after formal analysis, exact member/variant targets are attached through the existing formal attachment pipeline or its Part 3 evolution.
- [ ] family acquisition can expose a family semantic target if the final Part 3 identity model includes one.
- [ ] inherited behavioral acquisition points to the defining callable while retaining lookup-owner context in formal resolution.
- [ ] no LSP-side re-resolution of associated generics is introduced.

---

# 16. Part 4 Handoff Gate

Part 4 may choose runtime representations freely, but it must receive enough resolved semantics from Part 3.

- [ ] direct associated invocation records the selected exact invocation target.
- [ ] exact reification records the selected exact member/constructor identity.
- [ ] family reification records the frozen effective member set or a stable semantic handle to it.
- [ ] ordinary later call on a stored family records selected structural family operation when statically known.
- [ ] Part 4 does not need to recompute associated inheritance, visibility, owner specialization, or GADT compatibility from AST.
- [ ] runtime erasure of type parameters does not erase static proof identity from semantic products.

---

# 17. Concrete Regression Matrix

At minimum, add or preserve coverage for these semantic forms as Part 3 tasks land.

## Concrete exact references

- [ ] `Option<Int>::None`
- [ ] `Option<Int>::None::`
- [ ] `Option<Int>::None::()`
- [ ] `Option<Int>::Some::(_)`
- [ ] exact behavioral getter
- [ ] exact zero-argument behavioral method
- [ ] exact setter
- [ ] exact operator
- [ ] exact subscript get
- [ ] exact subscript set

## Concrete family values

- [ ] `Option<Int>::Some::*`
- [ ] family containing singleton + zero-arg constructor + payload constructor
- [ ] inherited behavioral family capture
- [ ] access-filtered family capture

## Generic Option A behavior

- [ ] explicit owner specialization succeeds
- [ ] contextual specialization succeeds
- [ ] bare exact generic reification underconstrained
- [ ] bare generic family underconstrained
- [ ] bare generic singleton underconstrained
- [ ] direct invocation infers
- [ ] expected result completes otherwise residual generic
- [ ] contradictory owner specialization is a conflict, not underconstraint

## Architecture regressions

- [ ] specializing one use does not mutate declaration schema
- [ ] independent second specialization produces different concrete view
- [ ] owner and callable-local generic identities remain distinct
- [ ] generic member reification follows the same Option A concrete-escape rule as owner-generic reification
- [ ] higher-kinded residual binder retains its kind
- [ ] no inference metavariable reaches stable binding/family type state

---

# 18. Review Questions for Every Relevant Pull Request

A reviewer should ask:

1. Is this code operating on a declaration schema or on one specialized view?
2. Could this substitution accidentally mutate canonical declaration semantics?
3. If a generic remains unresolved, can the code tell whether it came from the owner, callable, GADT equality, or current inference session?
4. Is a solver metavariable being stored somewhere longer-lived than the inference operation?
5. Does the family type retain exact selector-kind distinctions?
6. Is nominal family identity being confused with structural family type identity?
7. Is captured visibility/specialization information being discarded after reification?
8. Would adding `forall` later require changing associated lookup itself? If yes, why?
9. Would adding `forall` later require changing variant identity or exact-case identity? It should not.
10. Is a source-syntax choice being baked into the semantic representation unnecessarily?

Any “yes” to questions 2, 4, 6, 7, 8, 9, or 10 is a reason for architectural review before merging.

---

# 19. Explicit Non-Tasks for Part 3

Do not expand Part 3 scope to implement:

- [ ] `forall<T>` parser syntax;
- [ ] universal/rank-N type checking;
- [ ] automatic let-polymorphism;
- [ ] `Family<#{...}>` source annotations;
- [ ] `Case<...>` source annotations;
- [ ] a protocol/interface system;
- [ ] runtime generic dictionaries solely to support deferred Option B.

Those are future design/implementation units.

---

# 20. Completion Gate

Part 3 is Part-3.5-compatible when all of the following are true:

- [ ] Option A behavior is implemented consistently for exact members, whole families, and singleton values.
- [ ] direct invocation retains generic inference.
- [ ] underconstrained escaping reification fails statically rather than becoming Dynamic or first-use-specialized.
- [ ] canonical generic declaration schemas remain intact after specialization.
- [ ] binder ownership, kind, constraints, and GADT equalities remain recoverable.
- [ ] structural family types are independent from nominal associated family identity.
- [ ] captured associated semantic views retain specialization/access/target information needed by later calls and Part 4.
- [ ] no source family-type or `forall` syntax is required to make the semantic model complete.
- [ ] a future universal scheme layer could wrap the preserved declaration/member schemas without redesigning associated resolution.

The final implementation report for Part 3 should include a short subsection titled:

```text
Part 3.5 Forward-Compatibility Verification
```

and state which tests/invariants establish these points.
