# Whole-Lane Rest Inference — Post-Implementation Audit

**Repository:** `aureat/phalcom-lang`
**Audited revision:** `22cf74739a59fe178c0e3c1bb75c258a14327535`
**Commit:** `feat(semantic): support whole-lane rest inference`
**Baseline parent:** `fb24f981ae6ab5614c131ee4636f2e2830252ae0`

## Executive verdict

**I would not close TEL-1 as fully correct yet.**

The implementation gets the central language architecture substantially right. The motivating `GoalQuery.of(*goals: *Goals)` path is implemented coherently; contextual syntax is well-scoped; `*args: T` remains an element contract; `*args: *P` is represented distinctly; body-local captures become proper `Pack` types; generic whole-pack inference works for the basic empty and heterogeneous cases; labeled and complete packs have working happy paths; and the new contract participates in fingerprinting and several specialization paths.

That matches the principal architectural objectives of the plan. :chatgpt-content-reference{index="0"}

However, the audit found **four high-severity correctness/integration defects**, several medium-severity recovery/compatibility defects, and material gaps in the claimed test certification. The largest problems are:

| ID | Severity | Finding | Effect |
|---|---|---|---|
| **TEL1-AUD-01** | **HIGH** | Any inference variable anywhere inside a lane causes whole-lane `Equivalent` matching | Mixed generic/concrete lane contracts use equality where slot-wise conformance is required |
| **TEL1-AUD-02** | **HIGH** | Unknown/dynamic lane argument types are fabricated as `Unit` | Can solve a type pack to false `Unit` slots and contaminate inference |
| **TEL1-AUD-03** | **HIGH** | Trait witness compatibility still consumes `declared_type`, not `ParameterContract` | Whole-lane rest trait requirements cannot be checked correctly |
| **TEL1-AUD-04** | **HIGH** | Override generic alpha-renaming does not substitute lane contracts | Alpha-equivalent generic lane overrides are rejected |
| **TEL1-AUD-05** | Medium-high | Error recovery manufactures `Unit` body-element types and invalid lanes become `Unconstrained` | Invalid/blocked facts become invented semantic contracts |
| **TEL1-AUD-06** | Medium | Lane override compatibility uses identity equality, not callable-domain contravariance | Valid widening overrides are rejected |
| **TEL1-AUD-07** | Medium | Enum requirement/native compatibility paths still ignore `ParameterContract` | Additional callable-contract boundaries are incomplete |
| **TEL1-AUD-08** | Medium | Several mandatory tests are vacuous or weaker than their names | Current 17-test green suite does not certify the stated invariants |
| **TEL1-AUD-09** | Medium | Incremental test only proves fingerprint changes | Dependent recomputation and cold/warm parity remain unproven |
| **TEL1-AUD-10** | Medium | TEL-1 commit contains unrelated `packages/parse` changes while records claim those files were untouched | Lifecycle record is factually inconsistent with the commit |

**Recommended lifecycle state:** **implemented / partially verified / audit remediation required**, not final `COMPLETE`.

The implementation should be repaired in place rather than reverted.

---

# 1. What is implemented correctly

## 1.1 Contextual syntax design is correct

`phalcom-ast/src/ast.rs:1985-2053` introduces:

```rust
pub enum ParameterAnnotationSyntax {
    Type(TypeAnnotation),
    RestLane(TypePackExpansionSyntax),
}
```

This is the right representation. It avoids contaminating general `TypeAnnotationExpr` with an expansion construct that is not itself a proper type.

`phalcom-ast/src/parser.rs:6430-6445` parses `*P`, `**P`, and `***P` only through the parameter-annotation grammar:

```rust
fn parse_parameter_annotation(
    &mut self,
) -> ParserResult<ParameterAnnotationSyntax> {
    if let Some(mode) = self.expansion_mode_at_cursor() {
        ...
        Ok(ParameterAnnotationSyntax::RestLane(...))
    } else {
        Ok(ParameterAnnotationSyntax::Type(
            self.parse_type_annotation()?
        ))
    }
}
```

This preserves the decision we made:

```telos
*args: P     // ordinary element annotation
*args: *P    // explicit positional lane annotation
```

There is no contextual “oh, `P` happens to be a TypePack, so expand it” rule.

That part is strong.

---

## 1.2 The `TypePack` / proper `Pack` distinction is represented correctly

The valid path now distinguishes:

```text
P : TypePack

*args: *P
    caller-side positional lane contract

args : Pack<*P>
    body-local proper Pack value
```

`ParameterContract` is introduced in `phalcom-semantic/src/signature.rs`:

```rust
pub enum ParameterContract {
    Fixed(DeclaredTypeFact),

    Rest {
        mode: RestMode,
        contract: RestValueContract,
    },
}

pub enum RestValueContract {
    Unconstrained,
    Element(DeclaredTypeFact),
    Lane(TypeId),
}
```

This is the correct semantic decomposition.

The implementation also correctly avoids encoding unannotated rest as `Element(Unknown)`:

```rust
RestValueContract::Unconstrained
```

That preserves the important distinction between **semantic absence of a constraint** and **epistemic lack of knowledge**.

---

## 1.3 Lane lowering reuses canonical projection authority

`checker/declaration_signature.rs::parameter_fact` resolves the lane operand and projects through:

```rust
store.project_positionals(...)
store.project_labeled(...)
store.project_complete(...)
```

rather than insisting the operand syntactically denote a `TypePack`.

That correctly permits structural forms such as:

```telos
*args: *(Animal, String)
```

and maintains consistency with existing expansion semantics.

The rejection of:

```telos
<*P>
fn f(*args: P)
```

as an element contract with a non-`Type` annotation is also implemented.

This is exactly the semantic behavior we wanted.

---

## 1.4 Body-local capture typing is fundamentally correct

`checker/context.rs:1997-2011` now does:

```rust
let body_type_fact =
    parameter.contract.derive_body_type(self.store);
```

rather than binding the body-local identifier directly to `parameter.declared_type`.

That fixes the original architectural seam.

For valid source:

```telos
fn f(*xs: Int)
```

the body sees a proper open Pack type.

For:

```telos
fn f<*P>(*xs: *P)
```

the body sees a symbolic proper:

```text
PackFromPacks(P, EmptyLabeled)
```

rather than `P` itself.

This is a major improvement.

---

## 1.5 Basic whole-pack inference is correct

The implementation correctly establishes empty and heterogeneous positional inference:

```telos
f()
```

can infer:

```text
P = TypePack[]
```

and:

```telos
f(1, "two")
```

can infer:

```text
P = TypePack[Int, String]
```

`ConstraintOrigin::RestLane` was also added, which solves the zero-argument causality problem cleanly.

The motivating `GoalQuery` fixture is therefore on the right architecture rather than being patched specially.

---

## 1.6 Labeled and complete happy paths are real

The new tests inspect the canonical resulting terms, not merely absence of diagnostics:

```rust
rest_labeled_lane_preserves_labels_and_order
rest_complete_capture_preserves_both_lanes
```

They check exact labels, ordering, payload types, and both complete-pack lanes.

Those are useful regressions.

---

## 1.7 Fingerprinting notices parameter-contract changes

`db/fingerprint.rs:93-115` hashes:

```rust
Fixed
Rest(mode, Unconstrained)
Rest(mode, Element(...))
Rest(mode, Lane(...))
```

and `hash_callable_semantic_signature` incorporates it at lines 542-561.

So:

```telos
*args: Int
```

and:

```telos
*args: *P
```

do not have the same callable semantic fingerprint.

That is architecturally necessary and correctly implemented.

---

# 2. HIGH — mixed structural lane contracts use the wrong relation

This is the most important call-checking defect.

The plan explicitly distinguishes whole-pack variable inference from concrete pointwise conformance. :chatgpt-content-reference{index="1"}

The implementation instead branches on:

```rust
if term_contains_any_infer_var(&lane_param_term) {
    ...
    session.add_constraint(
        InferenceRelation::Equivalent(
            captured_lane_term,
            lane_param_term,
        ),
        ...
    );
} else {
    apply_concrete_lane_conformance(...)
}
```

at `checker/call.rs:2129-2228`.

That predicate is too broad.

## Why this is wrong

Consider:

```telos
class Animal {}
class Dog is Animal {}

class C {
    @class f<T>(
        *args: *(T, Animal)
    ) -> T {
        ...
    }

    @class run() {
        const x = C.f(Dog.new(), Dog.new())
    }
}
```

The declared lane is structurally:

```text
TypePack[T, Animal]
```

After instantiating `T`, its solver representation contains an inference variable, so:

```rust
term_contains_any_infer_var(...)
```

returns true.

The implementation therefore asks for:

```text
TypePack[Dog, Dog]
    ≡
TypePack[T, Animal]
```

Whole-pack structural equivalence eventually requires the second positions to unify as equals:

```text
Dog ≡ Animal
```

But the actual callable contract is:

```text
Dog <: T
Dog <: Animal
```

The second argument is perfectly valid.

### Correct law

Whole-pack equality is appropriate when the **whole lane itself is the captured inference variable**:

```text
P ≡ TypePack[Dog, Dog]
```

It is not appropriate merely because an inference variable occurs somewhere inside the expected structure.

The decision needs to distinguish:

```text
expected = Var(P : TypePack)
    -> exact whole-lane unification

expected = TypePack[T, Animal]
    -> exact topology + pointwise argument relations

expected = TypePack[T, U]
    -> exact topology + ordinary constraints for each slot
```

### Required correction

Do not use:

```rust
term_contains_any_infer_var(...)
```

as the semantic classification.

Introduce an explicit lane-contract matching operation. At minimum:

```rust
match normalized_expected_lane {
    InferenceTerm::Var(v)
        if kind(v) == required_lane_kind =>
    {
        // whole lane capture
        Equivalent(captured, Var(v))
    }

    structural_lane => {
        // deterministic lane topology alignment
        // each actual element <: corresponding expected term
    }
}
```

Complete and labeled lanes need the corresponding structural handling.

**Severity: HIGH.** The motivating simple `*P` case works, but a natural generalization of the feature is semantically wrong.

---

# 3. HIGH — unknown arguments are fabricated as `Unit`

At `checker/call.rs:2133-2180`, every lane mode contains this pattern:

```rust
let term = typed
    .symbolic_result
    ...
    .or_else(|| {
        let ty = typed.knowledge.ty()?;
        Some(...)
    })
    .unwrap_or_else(
        || InferenceTerm::Canonical(ctx.store.unit())
    );
```

This is not a safe recovery strategy.

If the argument's type is:

```text
Unknown
Dynamic
Blocked
```

and there is no usable symbolic result, the compiler invents:

```text
Unit
```

and places that into the inferred lane.

For example, the solver can be told:

```text
P ≡ TypePack[Int, Unit, String]
```

even though the second argument has never been proven to be `Unit`.

The implementation separately records the real premise state, but that does **not** make the fabricated equality sound.

## Required rule

“No usable type term” must remain “no usable type term.”

It must not become a canonical value.

A better result type for lane assembly is something like:

```rust
enum CapturedLaneTerm {
    Ready(InferenceTerm),
    Blocked(BlockReason),
    Dynamic(DynamicReason),
    Unknown(UnknownReason),
}
```

or the equivalent using existing checker outcome machinery.

If any exact whole-lane component lacks a sound inference term:

- preserve its causal evidence;
- mark inference/call status appropriately;
- do not solve the whole pack to an invented slot;
- retain/residualize the obligation if the architecture supports that.

This also needs explicit regression tests.

**Severity: HIGH.** It is a semantic fabrication that can pollute generic solutions.

---

# 4. HIGH — trait witness compatibility does not understand lane contracts

The implementation introduces `ParameterContract` as the caller-facing parameter authority, but an important preexisting compatibility boundary still reads `declared_type`.

`impls.rs:1613-1638`:

```rust
for (required_parameter, candidate_parameter)
    in required.signature.parameters
        .iter()
        .zip(candidate.parameters.iter())
{
    if required_parameter.rest != candidate_parameter.rest {
        ...
    }

    let required_type =
        witness_fact_type(
            &required_parameter.declared_type
        )?;

    let candidate_type =
        witness_fact_type(
            &candidate_parameter.declared_type
        )?;

    ...
}
```

For a lane contract:

```telos
*args: *P
```

`ParameterContract::declared_type()` intentionally produces:

```rust
UnknownReason::NoTypeEvidence
```

because a `TypePack` is not a proper parameter value type.

Therefore `witness_fact_type()` returns `WitnessCompatibility::Unknown`.

So a trait requirement such as:

```telos
trait R {
    call<*P>(*args: *P)
}
```

cannot be correctly compared with its implementation through the canonical witness checker.

This directly contradicts the purpose of making `ParameterContract` the semantic authority.

## The problem continues into trait dispatch

`trait_dispatch.rs:461-475` specializes:

```rust
parameter.declared_type
```

but does **not** specialize:

```rust
parameter.contract
```

for derived guarded requirements.

Thus even once witness comparison is corrected, instantiated lane contracts can remain attached to the wrong generic environment.

## Required correction

There should be one reusable parameter-contract operation used by:

```text
override compatibility
trait witness compatibility
trait requirement instantiation
enum requirement compatibility
native/source compatibility where applicable
```

Conceptually:

```rust
specialize_parameter_contract(...)
substitute_parameter_contract(...)
parameter_contract_accepts(...)
```

For witness variance:

```text
candidate parameter domain
must be >= required parameter domain
```

because parameter positions are contravariant.

For whole generic lanes, corresponding pack binders need alpha alignment before comparison.

**Severity: HIGH.** This is a major integration boundary, not an edge-case diagnostic.

---

# 5. HIGH — generic lane overrides fail alpha-equivalence

There is another independent override defect.

`checker/declaration.rs::align_callable_generic_binders`, lines 152-188, correctly builds:

```rust
candidate_parameter -> inherited_parameter
```

substitutions.

But it only applies them to:

```rust
parameter.ty
return_type
generic constraints
```

It does **not** transform:

```rust
parameter.contract
```

Then override compatibility later compares lane contracts using:

```rust
(
    RestValueContractKnowledge::Lane(c_lane),
    RestValueContractKnowledge::Lane(i_lane),
) => c_lane == i_lane
```

Because callable generic parameters have owner-qualified identities, these two valid methods:

```telos
class Base {
    m<*P>(*args: *P) {}
}

class Derived is Base {
    m<*Q>(*args: *Q) {}
}
```

contain distinct canonical parameter identities before alpha-renaming.

`parameter.ty` cannot save this because lane parameters intentionally expose `NoTypeEvidence` there.

The result is that a semantically alpha-equivalent override can be reported as incompatible.

## Required correction

During alpha alignment:

```rust
for parameter in &mut candidate.parameters {
    parameter.contract =
        substitute_callable_parameter_contract(
            store,
            &substitution,
            &parameter.contract,
        );
}
```

The compatibility check then compares/reasons about normalized contracts.

A dedicated positive regression is mandatory:

```text
generic_lane_override_is_alpha_equivalent
```

The existing test only checks the negative:

```text
Element vs Lane -> incompatible
```

so this bug was invisible.

---

# 6. Medium — override lane compatibility is too strict even after alpha-renaming

The existing implementation accepts two lane contracts only when their canonical `TypeId`s are equal.

That is not callable parameter substitutability.

Suppose:

```text
Dog <: Animal
String <: Object
```

An inherited contract:

```telos
*args: *(Dog, String)
```

may safely be overridden by a candidate accepting:

```telos
*args: *(Animal, Object)
```

because every input accepted by the base method is accepted by the override.

The candidate has widened its accepted domain.

The implementation currently rejects that because:

```text
laneA != laneB
```

Likewise:

```text
inherited: *args: Animal
candidate: *args
```

is normally safe because an unconstrained rest accepts at least the inherited inputs, but only `Unconstrained` vs `Unconstrained` is presently accepted.

This is conservative rather than unsound, but it is semantically incomplete.

The rest-lane compatibility operation should reuse the same topology-plus-slot relation as call checking, with the direction reversed appropriately for callable parameter variance.

---

# 7. Medium-high — recovery paths manufacture semantics

There are two related problems.

## 7.1 Invalid element contract becomes `Pack<*: Unit>`

`signature.rs:90-101`:

```rust
let elem_ty = fact
    .canonical_type()
    .unwrap_or_else(|| store.unit());
```

If the declared element contract is unresolved, invalid, or otherwise lacks a canonical type, the compiler derives a body Pack constrained to `Unit`.

That is false information.

It should instead derive:

```text
Unknown / invalid body type
```

while preserving the original reason.

---

## 7.2 Complete lane projection uses `Unit` as a failure fallback

`signature.rs:119-124`:

```rust
let pos = store
    .project_positionals(*lane_id)
    .unwrap_or_else(|_| store.unit());

let lab = store
    .project_labeled(*lane_id)
    .unwrap_or_else(|_| store.unit());
```

The subsequent `pack_from_packs` will normally reject those wrong-kinded substitutes, so this is less dangerous than the element case, but the architecture is still wrong.

A failed projection is not `Unit`.

Use explicit error propagation.

---

## 7.3 Invalid lane lowering falls back to `Unconstrained`

`parameter_fact` emits a diagnostic for mismatched/failed lane formation but then constructs an unconstrained rest contract for recovery.

That broadens:

```text
invalid contract
```

into:

```text
accept anything
```

inside the semantic model.

A better internal state would be an explicit invalid/unknown lane contract, for example:

```rust
RestValueContract::Invalid(UnknownReason)
```

or a general checked lane fact capable of retaining non-ready state.

The important invariant is:

> diagnostic recovery must not invent a stronger or broader callable contract.

---

# 8. Medium — additional compatibility consumers are incomplete

The repo contains more `CallableSemanticSignature` consumers than the implementation plan fully migrated.

## Enum requirements

`enum_requirements.rs:139-186` compares:

```rust
rest
declared_type
```

and never inspects `ParameterContract`.

A whole-lane requirement has no proper `declared_type`, so it will be classified incompatible.

## Native/source contract comparison

`signature.rs:393-415` compares:

```text
label
rest mode
declared_type
```

and treats unknown declared parameter types permissively.

It does not compare the new contract category or lane.

If native metadata ever represents these contracts, two different whole-lane contracts with the same rest mode can compare as compatible.

## Trait-dispatch specialization

As noted above, `trait_dispatch.rs` instantiates `declared_type` but not `contract`.

These are precisely the kinds of consumers that a “single canonical parameter contract authority” migration needs to census.

The current implementation migrated many important sites, but not all of them.

---

# 9. Concrete-lane helper can silently abandon symbolic non-flat contracts

`apply_concrete_lane_conformance` contains forms such as:

```rust
let TypeData::TypePack(expected_elems) =
    ctx.store.get(expected_pos).clone()
else {
    return;
};
```

The labeled equivalent behaves similarly.

For ordinary closed tuple lanes this works.

But if a normalized lane is a valid non-flat symbolic form such as a projection/residual that contains no invocation-local inference variable, this helper simply returns:

```text
no constraint
no block
no diagnostic
```

That is an unsafe default.

A valid-but-not-decomposable contract should yield an explicit residual/blocked semantic result. It must not mean “accept the call.”

This should be fixed while replacing the broader lane-matching heuristic.

---

# 10. Testing audit

The implementation plan explicitly called for all 15 anti-regressions and a full rest-contract suite. :chatgpt-content-reference{index="2"}

The repository contains 17 tests and the walkthrough records all 17 as locally passing.

That is useful evidence, but **the suite does not actually certify several of the names it advertises**.

I did not independently execute the tests during this audit. The implementation records report successful local execution, while GitHub exposes no workflow run for commit `22cf7473`. My testing judgment below is therefore based on source inspection plus the recorded local results.

## 10.1 `rest_argument_analysis_preserves_source_order` does not test order

Current test:

```rust
#[test]
fn rest_argument_analysis_preserves_source_order() {
    ...
    Trace.target(
        Trace.log(1),
        Trace.log(2),
        label: Trace.log(3)
    )
    ...
    f.assert_no_error_diagnostics();
}
```

There is no observable trace.

If the compiler analyzed or executed:

```text
3, 1, 2
```

or evaluated an expression twice, this test could still pass.

The implementation itself does iterate `args` in source order, so the source audit is encouraging. The **test**, however, does not prove the invariant.

Required replacement: an observable runtime side effect or an analysis instrumentation assertion that detects both ordering and exactly-once evaluation.

---

## 10.2 `rest_lane_inference_does_not_partition_pack_concat` is vacuous

Current test:

```rust
#[test]
fn rest_lane_inference_does_not_partition_pack_concat() {
    let f = Fixture::new(r#"
class C {
    @class test<*P>(*args: *P) {}
}
"#);
    f.assert_no_error_diagnostics();
}
```

There is:

- no concat,
- no call,
- no equation,
- no unknown segment,
- no assertion about `P`.

It proves nothing about pack decomposition.

This test needs to operate directly against inference terms, e.g.:

```text
TypePackConcat[
    Var(P),
    TypePack[String]
]

≡

TypePack[Int, Bool, String]
```

and assert that the solver does **not** infer:

```text
P = TypePack[Int, Bool]
```

without predetermined segment correspondence.

---

## 10.3 Incremental test does not test dependent invalidation

`rest_contract_change_invalidates_dependent_calls` asserts:

```rust
initial_fp != edited_fp
```

for `Callee.run`.

That proves the callee signature fingerprint changes.

It does **not** prove:

```text
Caller.call recomputed
dependency invalidated
warm result == cold result
unrelated callables reused
```

The test name overstates its coverage.

A proper incremental test should inspect the update statistics/dependency products and compare the warm second snapshot with a fresh cold workspace.

---

## 10.4 Nested `Pack<*P>` test is weak

`rest_pack_of_expansion_remains_proper_pack_element_contract` combines a valid and invalid call and then accepts either:

```text
ArgumentMismatch
OR
GenericInferenceUnderconstrained
```

as success.

It does not establish that the valid call succeeded cleanly, nor verify the inferred `P`.

Split it:

```text
proper_pack_element_contract_accepts_pack_argument
proper_pack_element_contract_rejects_scalar_arguments
```

The positive test should inspect the inferred pack lane.

---

## 10.5 No mixed structural-generic lane test

This is why TEL1-AUD-01 escaped.

Required immediately:

```telos
class Animal {}
class Dog is Animal {}

class C {
    @class f<T>(
        *args: *(T, Animal)
    ) -> T { ... }

    @class run() {
        const x = C.f(
            Dog.new(),
            Dog.new()
        )
    }
}
```

Assert:

```text
T = Dog
no errors
```

A second negative case should demonstrate that the concrete slot still rejects a non-`Animal`.

---

## 10.6 No unknown/dynamic lane evidence test

This allowed the `Unit` fabrication to escape.

Tests need to show:

```text
Unknown  != Unit
Dynamic  != Unit
Blocked  != Unit
```

during whole-lane inference.

No published generic specialization should contain `Unit` unless the supplied argument is actually `Unit`.

---

## 10.7 Override coverage is only negative

Current:

```text
Element vs Lane -> incompatible
```

Good, but insufficient.

Missing mandatory positive/variance cases:

```telos
Base.m<*P>(*args: *P)
Sub.m<*Q>(*args: *Q)       // must be alpha-equivalent

Base.m(*args: *(Dog))
Sub.m(*args: *(Animal))    // widening should be valid

Base.m(*args: Animal)
Sub.m(*args)               // if unconstrained rest is top acceptance domain
```

and reversed unsound cases must fail.

---

## 10.8 Trait witness coverage is absent

There is no rest-contract trait/conformance test.

This is now a mandatory blocker because the static audit found the compatibility code still reading `declared_type`.

Add:

```text
trait_lane_requirement_accepts_alpha_equivalent_witness
trait_lane_requirement_rejects_element_contract_witness
trait_lane_requirement_applies_contravariant_lane_relation
trait_lane_requirement_specializes_owner_and_method_generics
```

---

# 11. Coverage assessment

| Area | Audit assessment |
|---|---|
| Contextual parser syntax | **Good** |
| General-type-position isolation | **Good** |
| Plain element-rest preservation | **Good** |
| Bare TypePack rejection | **Good** |
| Body-local proper Pack type | **Good on valid inputs** |
| Positional `*P` empty inference | **Good** |
| Positional heterogeneous `*P` inference | **Good** |
| Labeled generic lane happy path | **Good** |
| Complete generic lane happy path | **Good** |
| Closed concrete positional lane conformance | **Good basic coverage** |
| Concrete labeled/complete negative cases | **Thin** |
| Mixed generic/concrete lanes | **Missing; implementation defective** |
| Unknown/dynamic/blocked propagation | **Missing; implementation defective** |
| Source-order evaluation | **Implementation plausible; test ineffective** |
| Whole-pack no-partition invariant | **Test vacuous** |
| Override category mismatch | **Covered** |
| Override alpha-equivalence | **Missing; implementation defective** |
| Override variance | **Missing/incomplete** |
| Trait witness compatibility | **Missing; implementation defective** |
| Enum requirement compatibility | **Missing/incomplete** |
| Native/source compatibility | **Not audited by tests / incomplete code path** |
| Fingerprint sensitivity | **Covered** |
| Actual incremental recomputation | **Not covered** |
| Cold/warm parity | **Not covered** |
| Metadata round-trip | **No new coverage found** |
| Performance/large lanes | **Not covered** |

So the 17 focused tests are a useful smoke suite, but they are **not sufficient for correctness certification**.

---

# 12. Implementation quality

## Strong aspects

The most important design choices are good:

- contextual AST carrier rather than globally adding lane expansion;
- canonical `ParameterContract`;
- explicit `Unconstrained`;
- reuse of `project_positionals` / `project_labeled` / `project_complete`;
- proper-Pack body binding;
- exact empty-lane evidence;
- `ConstraintOrigin::RestLane`;
- preservation of data-constructor machinery;
- contract-aware receiver specialization in `checker/context.rs`;
- contract-aware witness signature specialization in `impls.rs`;
- contract-sensitive semantic fingerprinting.

These indicate the implementation understood the language model rather than merely forcing the GoalQuery test green.

## Weaker aspects

### `call.rs` has duplicated relation machinery

There are separate generic and non-generic concrete lane conformance paths, plus repeated positional/labeled/complete lane assembly.

That has already contributed to divergent behavior.

A better architecture would centralize:

```text
captured-lane formation
lane topology comparison
slot relation generation
evidence aggregation
diagnostic topology
```

and let generic/non-generic callers choose how the generated relations are discharged.

### `ParameterContract` is conceptually authoritative but not structurally sole-authority

Both semantic and dispatch parameter records still expose mutable duplicates:

```text
rest
declared_type / ty
contract
```

The compatibility helpers try to keep them synchronized.

This is acceptable as a migration bridge, but it remains an invariant surface. Public mutators such as `with_rest` can reconstruct/alter contract state.

Longer term, `rest` and caller-facing type should be projections from `contract`, not peer mutable facts.

### Fallible semantic operations use fabricated fallback values

`unwrap_or_else(|| store.unit())` is especially inappropriate in type-system code. Failures should remain failures/unknowns.

---

# 13. Implementation-record and commit hygiene

There is a concrete documentation inconsistency.

The TEL-1 walkthrough says:

> `packages/parse/` and `phalcom-core/tests/core/mod.rs` untouched.

The handoff repeats that they were “preserved without modifications.”

But commit `22cf7473` modifies **38 files**, including:

```text
packages/parse/README.md
packages/parse/src/core/rule.ph
packages/parse/src/main.ph
packages/parse/src/run.ph
packages/parse/tests/runtime/src/clients/dsl-test.ph
packages/parse/tests/runtime/src/clients/toml-test.ph
packages/parse/tests/runtime/src/core/recursion-test.ph
packages/parse/tests/runtime/src/core/sequence-test.ph
packages/parse/tests/runtime/src/expr/operators-test.ph
packages/parse/tests/runtime/src/support/test.ph
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/modules/parse_package.rs
```

Those changes are in the **same commit** as TEL-1.

This matters because it:

- makes semantic regression bisect/revert less clean;
- invalidates the stated “preserved in-flight state” record;
- obscures whether unrelated parse changes influenced broad test behavior;
- makes the implementation record non-auditable as written.

I would correct the walkthrough/handoff immediately. Do not rewrite Git history solely for this if the shared main branch has moved on, but the lifecycle record must describe the actual commit.

---

# 14. Plan quality audit

Some defects originated in the implementation, but a few were already latent in the plan.

The plan correctly demanded separate whole-variable equivalence and pointwise concrete matching, contextual syntax, and explicit RestLane causality. :chatgpt-content-reference{index="3"}

However:

### The plan itself suggested the `Unit` body-type fallback

Its sample `derive_body_type` contained:

```rust
fact.canonical_type()
    .unwrap_or_else(|| store.unit())
```

That should never have been specified.

### The override plan used exact lane equality

The proposed compatibility code used:

```rust
c_lane == i_lane
```

which ignores callable parameter variance.

### The proposed source-order test was not observational

The plan's test only called several `Trace.log` expressions and checked diagnostics. That does not measure order.

So the implementer should fix these, but the active plan should also be amended so a future agent does not faithfully restore the same defects.

---

# 15. Required remediation before closure

I recommend a bounded **TEL-1 corrective pass**, not a redesign.

### P0 — semantic correctness

1. Replace `term_contains_any_infer_var` as the whole-lane classification rule. Only an actual whole-lane inference target should use exact `Equivalent`; structured lanes containing variables require deterministic topology matching plus per-slot relations.

2. Remove every `Unit` fallback used as a substitute for unavailable type evidence in rest-contract formation and captured-lane inference.

3. Make trait witness compatibility consume `ParameterContract`, including generic alpha-substitution and directional parameter-domain compatibility.

4. Alpha-substitute `CallableParameterContract` inside `align_callable_generic_binders`.

5. Specialize `ParameterContract` in the derived trait-dispatch requirement path.

### P1 — semantic integration

6. Generalize lane compatibility into a shared operation and use it for overrides, trait witnesses, and appropriate enum requirement checks.

7. Make invalid lane lowering preserve an invalid/unknown contract state rather than converting it to `Unconstrained`.

8. Audit and migrate the remaining `CallableSemanticSignature` consumers that compare `rest + declared_type`, especially:
  - `enum_requirements.rs`;
  - native/source contract comparison;
  - any reflection or metadata publication that exposes caller-facing parameter contracts.

9. Make non-flat symbolic concrete-lane handling explicitly blocked/residual rather than silently returning.

### P1 — verification

10. Replace the vacuous/weak tests with tests that actually observe the promised invariant: mixed structural generics, unknown propagation, real concat non-decomposition, runtime/exact-once source order, alpha-equivalent overrides, trait witnesses, and incremental cold/warm parity.

---

# 16. Tests I would require before changing the audit verdict

At minimum, the corrective pass should add these named regressions:

```text
rest_mixed_generic_concrete_lane_uses_pointwise_conformance
rest_mixed_generic_lane_infers_only_generic_slots
rest_unknown_argument_does_not_infer_unit_lane_element
rest_dynamic_argument_does_not_infer_unit_lane_element

rest_generic_lane_override_is_alpha_equivalent
rest_concrete_lane_override_is_contravariant
rest_unconstrained_override_can_widen_element_rest
rest_narrowing_override_is_rejected

trait_rest_lane_alpha_equivalent_witness_is_compatible
trait_rest_lane_element_witness_is_incompatible
trait_rest_lane_contract_specializes_generic_environment

rest_lane_concat_does_not_infer_unknown_prefix
rest_argument_evaluation_is_source_order_and_exactly_once

rest_incremental_contract_edit_recomputes_dependent_call
rest_incremental_contract_edit_matches_cold_analysis

rest_invalid_lane_does_not_recover_as_unconstrained
rest_invalid_element_contract_does_not_derive_unit_pack
```

Then rerun the current focused suites plus the relevant trait/impl, enum-requirement, incremental, and callable-signature suites.

---

# Final assessment

The implementation is **substantive and directionally strong**. It is not a failed implementation: the original TypePack/Pack conflation has been removed on the principal path, the syntax is much cleaner, the body-capture model is now conceptually right, and the motivating generic `GoalQuery` case is solved using the correct general machinery.

But the feature currently has a **happy-path-complete / semantic-integration-incomplete** profile.

The central rule that still needs correction is:

```text
whole lane variable
    -> exact whole-lane inference

structured lane containing generic variables
    -> structural topology + ordinary slot relations
```

not:

```text
contains any inference variable
    -> whole-lane equality
```

And `ParameterContract` must become authoritative at **every callable compatibility boundary**, not only call checking, override category checking, and receiver specialization.

**Audit disposition: REMEDIATION REQUIRED before TEL-1 can be called semantically complete.** The current commit is a solid base for that corrective pass; I would preserve it and patch the identified gaps rather than revisiting the language design.