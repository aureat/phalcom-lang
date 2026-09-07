# Phalcom ADT / GADT / Match Comprehensive Testing Scenario Catalog

**Purpose:** This document is the executable specification for the scenarios referenced by `phalcom-adt-comprehensive-testing-implementation-plan.md`. Each scenario describes what to build, why it exists, what information must be asserted, expected failure behavior, and what to inspect when it fails.

**Repository baseline while written:** `feat/adts@72d6eca53944c588c653ad76e8b44056df9bef4d`.

**Status vocabulary:**
- `EXISTING-DEEP`: already present with an adequate oracle; preserve.
- `EXISTING-SHALLOW`: scenario exists but assertion depth must be strengthened.
- `ADD`: missing scenario to implement.
- `RED`: scenario is expected to expose a current implementation gap.
- `GATED`: language/product prerequisite not implemented; record but do not fake.

## How to use this catalog

For each scenario:
1. Prefer source-driven analysis unless the law is specifically an internal algebra law.
2. Keep fixtures minimal: include only declarations needed to isolate the law.
3. Assert canonical products before rendered strings.
4. For negative scenarios, assert the precise machine diagnostic and range.
5. When a scenario spans semantic and runtime layers, split into a semantic test and a core test with the same law ID.
6. If the current implementation contradicts the ratified spec, keep the expected result from the spec and classify the test `RED`; do not “fix” the test to current behavior.

---

# D — Enum Declaration Scenarios

## ADT-DECL-01 — Ordinary closed enum declaration
**State:** ADD or strengthen existing `declarations.rs`.

**Program:**
```phalcom
enum Status {
    @variant Active -> Status
    @variant Inactive -> Status
}
```

**Proves:** the enum owner exists as one nominal declaration and both variants belong to it.

**Assert:**
- one `DeclarationId` for `Status`;
- two exact `VariantId`s;
- both variant owners equal `Status` declaration;
- variant order is declaration order if order is part of `EnumSemanticTable`;
- no diagnostics.

**Do not settle for:** `diagnostics.is_empty()`.

**Debug:** dump enum table owner and selector map. If IDs differ by accidentally synthesized hidden-class owner, this is an identity-layer bug.

## ADT-DECL-02 — Generic enum parameter ownership
```phalcom
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}
```

**Assert:**
- `T` is owned by the enum declaration;
- `Some` payload declared type references that exact type parameter;
- result template is `Option<T>`;
- `None` result template shares the same enum generic environment.

## ADT-DECL-03 — Multi-parameter enum
Use `Result<T,E>`.
Assert parameter ordering/identity and distinct substitution into `Ok` versus `Err`.

## ADT-DECL-04 — GADT result specialization
Use `Expr<T>` with `Int -> Expr<Int>` and `Bool -> Expr<Bool>`.
Assert each `VariantInfo.result_type_template` and `CaseTypeEnvironment`.

## ADT-DECL-05 — Same base, distinct selector declarations
```phalcom
enum Animal {
    @variant Dog
    @variant Dog()
    @variant Dog(_ name: String)
    @variant Dog(named age: Int)
}
```
Assert four `VariantId`s, one `VariantFamilyId("Dog")`, no accidental collapse.

## ADT-DECL-06 — Duplicate exact selector
Declare the same exact selector twice.
Assert declaration diagnostic is specific to duplicate variant/selector identity, and no second variant silently overwrites the first.

## ADT-DECL-07 — Field identity ordering
Variant with positional + labeled fields.
Assert every `VariantFieldId`, declaration order, external label, and declared type.

## ADT-DECL-08 — Source ranges
Assert enum, variant base, and payload-field source ranges map to exact source spans used by diagnostics/tooling.

---

# V — Variant Shape and Identity Scenarios

## ADT-VARIANT-01 — Singleton/getter variant
`@variant Dog`
Assert getter-shaped selector and zero payload fields.

## ADT-VARIANT-02 — Nullary constructor
`@variant Dog()`
Assert callable/nullary selector and zero payload fields.

## ADT-VARIANT-03 — Singleton and nullary coexist
Declare both `Dog` and `Dog()`.
Assert:
- IDs differ;
- exact-case types differ;
- associated lookup `Animal::Dog` and `Animal::Dog()` resolve differently;
- matching one cannot cover the other.

## ADT-VARIANT-04 — Positional payload
`Dog(_ name: String)`.
Assert selector slot is positional and field's local name is not selector identity.

## ADT-VARIANT-05 — Labeled payload
`Dog(named age: Int)`.
Assert external label `named` participates in selector identity; local `age` does not.

## ADT-VARIANT-06 — Mixed payload
Assert field IDs and selector slots remain aligned.

## ADT-VARIANT-07 — Family identity
All same-base variants share exactly one `VariantFamilyId`.

## ADT-VARIANT-08 — Name visibility
Private/inaccessible variant name cannot be explicitly acquired outside allowed scope.

## ADT-VARIANT-09 — Construction visibility independent from matching
A variant that cannot be constructed from caller scope must still participate in match exhaustiveness and may be matchable if naming visibility permits.

## ADT-VARIANT-10 — Payload visibility
Explicit binding/nested projection of private payload is rejected; wildcard-only payload ignore remains allowed where spec permits.

---

# C — Constructor Surface Scenarios

## ADT-CONSTR-01 — Payload constructor result
`Option::Some(42)` produces enclosing `Option<Int>` as ordinary value type while semantic constructor result retains the exact-case identity where the checker publishes it.

Assert both declared/formal call result and observed exact-case knowledge if the product differentiates them.

## ADT-CONSTR-02 — Nullary constructor invocation
`Animal::Dog()` constructs a fresh nullary constructor value and does not resolve singleton `Dog`.

## ADT-CONSTR-03 — Singleton access is not invocation
`Animal::Dog` yields singleton variant value; there is no synthesized zero-argument call.

## ADT-CONSTR-04 — Labeled constructor routing
Correct external label succeeds; wrong label produces selector/call diagnostic.

## ADT-CONSTR-05 — Generic specialization through constructor
`Result::Ok(42)` under expected `Result<Int,String>` specializes owner generics without fabricating unsupported evidence.

## ADT-CONSTR-06 — Constructor family member identity
Capturing exact constructor versus whole family retains exact selector/family identity.

---

# E — Exact-Case Type Scenarios

## ADT-EXACT-01 — Exact case <: enum root
Assert relation is true.

## ADT-EXACT-02 — Sibling exact cases are not interchangeable
`Some<Int>` exact case is not subtype/equivalent to `None<Int>` exact case.

## ADT-EXACT-03 — Exact-case union normalization
Union of exact cases retains members, deduplicates duplicates, and does not widen to enum root.

## ADT-EXACT-04 — Binding preservation
A value constructed as exact `Some` should retain exact observed evidence until a contract/flow join requires widening.

## ADT-EXACT-05 — Generic substitution
Exact case payload type and enclosing enum application specialize together.

## ADT-EXACT-06 — Alias union
A transparent alias over exact cases resolves to same canonical union as direct spelling.

---

# G — Generic ADT / Declaration-Level GADT Scenarios

## ADT-GEN-01 — Generic `Option<T>` payload substitution
`Option<Int>` specializes `Some.value` to `Int`.

## ADT-GEN-02 — `Result<T,E>` independent parameters
`Ok` uses T, `Err` uses E.

## ADT-GEN-03 — Nested generic payload
`Option<Result<Int,String>>` retains nested application exactly.

## ADT-GADT-01 — Case environment stored on declaration
For `Expr::Int`, assert declaration-owned equality binding `T = Int`.

## ADT-GADT-02 — Multi-parameter equality
```phalcom
enum Equal<A,B> {
    @variant Refl -> Equal<A,A>
}
```
Assert B/A relationship is represented by case constraints/substitution according to current canonical model.

## ADT-GADT-03 — Contradictory specialization detectable
`Expr<Int>` with `Bool` case must be classified impossible during elimination, not merely fail ordinary subtyping.

---

# B — ADT Behavior and Requirement Scenarios

## ADT-BEH-01 — Enum-wide method inherited by all variants
Declare shared behavior on enum, invoke through each variant value.

**Semantic assert:** method contract belongs to enum surface.
**Core assert:** runtime dispatch reaches shared implementation.

## ADT-BEH-02 — Per-variant override
One variant overrides common method. Assert semantic dispatch target and runtime result.

## ADT-BEH-03 — Per-variant additional method
Method exists only on one exact case. Assert exact-case receiver accepts it while root type does not incorrectly promise it.

## ADT-REQ-01 — Shared method contract satisfied
Common declaration contract plus variant implementation with compatible signature.

## ADT-REQ-02 — Missing implementation
Variant missing required behavior => dedicated requirement diagnostic.

## ADT-REQ-03 — Wrong selector/arity
Method base matches but selector shape differs; must not satisfy requirement.

## ADT-REQ-04 — Wrong return type
Incompatible return fails requirement compatibility.

## ADT-REQ-05 — Generic specialization
Requirement with T specializes per enum application.

---

# A — Associated Lookup / Family Scenarios

## ADT-ASSOC-01 — Exact singleton lookup
`Animal::Dog` resolves exact singleton `VariantId`.

## ADT-ASSOC-02 — Exact nullary lookup
`Animal::Dog::()` or current ratified exact associated callable syntax resolves nullary constructor identity. Use current repository syntax; do not invent dot notation.

## ADT-ASSOC-03 — Exact payload constructor
Exact selector resolves one constructor.

## ADT-ASSOC-04 — Whole family capture
`Animal::Dog::*` or current family-value syntax resolves one `VariantFamilyId` and freezes/represents members according to Part 03/03.5 semantics.

Assert exact member IDs, not count.

## ADT-ASSOC-05 — Callable-pattern family
Pattern family includes callable members but not getter/singleton where selector kind excludes it.

## ADT-ASSOC-06 — Static family invocation
Call shape statically chooses exact family member.

## ADT-ASSOC-07 — Dynamic family pack routing
When dynamic routing is implemented, varied shapes route to correct member. If currently unimplemented, keep `RED` with expected semantics.

## ADT-ASSOC-08 — Positional rest routing
Candidate with rest positional slots receives appropriate shapes.

## ADT-ASSOC-09 — Labeled rest routing
Same for labeled rest.

## ADT-ASSOC-10 — Generic specialization
Captured family over generic owner specializes member signatures correctly.

## ADT-ASSOC-11 — Inheritance
Inherited associated/family member retains semantic owner/dispatch identity.

## ADT-ASSOC-12 — Exact selector miss
Missing route does not select nearest shape.

## ADT-ASSOC-13 — Wrong call shape
Family call with incompatible shape reports family/call diagnostic rather than invoking arbitrary overload.

## ADT-ASSOC-14 — Visibility filtering
Private member not explicitly acquirable.

## ADT-ASSOC-15 — Capability confinement
A captured/frozen descriptor must not silently acquire later/live hierarchy members outside its captured semantic set where confinement is the ratified rule.

## ADT-ASSOC-16 — Capability escape visibility
Passing/storing family value must not grant inaccessible members.

---

# M1 — Match Resolution Scenarios

## MATCH-RES-01 — Wildcard root
```phalcom
match x { _ => 1 }
```
Assert resolution recorded, wildcard pattern, initial/reachable/final residual relationship, proven totality.

## MATCH-RES-02 — Qualified singleton
Pattern `Animal::Dog`.
Assert exact singleton candidate only.

## MATCH-RES-03 — Qualified nullary
Pattern `Animal::Dog()`.
Assert exact nullary candidate only and ID differs from singleton.

## MATCH-RES-04 — Exact positional constructor
`Animal::Dog(name)`.
Assert exact selector and exact field ID.

## MATCH-RES-05 — Exact labeled constructor
Assert external label selects correct field/variant.

## MATCH-RES-06 — Contextual `Some(x)`
Scrutinee `Option<Int>`.
Assert owner resolved from expected pattern domain, not global lookup.

## MATCH-RES-07 — Contextual `None`
Same for singleton.

## MATCH-RES-08 — Ambiguous contextual owner
Union/domain where two owners expose same compatible base.
Assert `MatchVariantAmbiguous`, no arbitrary candidate.

## MATCH-RES-09 — Nested contextual resolution
`Some(Ok(x))`.
Assert inner owner is resolved from specialized `Some` payload domain.

## MATCH-RES-10 — `Dog(...)`
Assert all callable Dog candidates, excludes singleton.

## MATCH-RES-11 — `Dog(x, ...)`
Assert prefix slot constraint and candidate-specific field projection.

## MATCH-RES-12 — `Dog(..., named: y)`
Assert suffix label matching through canonical `SelectorPattern::matches`.

## MATCH-RES-13 — `Dog(x, ..., named: y)`
Assert prefix/suffix fields and gap omission.

## MATCH-RES-14 — `Dog*`
Assert whole family includes singleton and callable members.

## MATCH-RES-15 — Candidate ordering
Assert deterministic semantic candidate ordering is stable across repeated analysis.

---

# M2 — Recursive Pattern Scenarios

## MATCH-PAT-01 — Wildcard payload
`Some(_)` succeeds without binding.

## MATCH-PAT-02 — Nested ADT
`Some(Ok(x))`.
Assert nested resolution and exact inner candidate.

## MATCH-PAT-03 — Tuple of ADTs
`(Red, Green)` etc. Assert recursive component spaces.

## MATCH-PAT-04 — ADT payload tuple
Variant field tuple pattern specializes child expected type.

## MATCH-PAT-05 — Or-pattern nested in payload
`Some(Ok(x) | Cached(x))`.
Assert one `PatternResolution::Or` under the payload, not parser duplication into arms.

## MATCH-PAT-06 — Family pattern recursive fields
Candidate-specific child projection can differ by physical/semantic field IDs while producing one source pattern binding.

## MATCH-PAT-07 — List empty/non-empty
Where 05.1 list shape algebra is implemented, prove exact length/prefix semantics.

---

# M3 — Binding Scenarios

## MATCH-BIND-01 — Simple payload binding
Assert one `BindingId`, source range, `Int` knowledge.

## MATCH-BIND-02 — Labeled payload binding
Binding type derives from mapped `VariantFieldId`.

## MATCH-BIND-03 — Wildcard binds nothing
Assert empty binding list.

## MATCH-BIND-04 — Duplicate binding in one alternative
Current HEAD contains this regression. Strengthen to assert exact `DiagnosticCode::MatchDuplicateBinding`, primary range of second occurrence, and no valid branch binding product.

## MATCH-BIND-05 — Or alternatives same binding
`Left(x) | Right(x)`.
Assert one branch-visible binding identity and joined type `Int | String`.

## MATCH-BIND-06 — Or alternatives different names
Current HEAD regression. Assert exact mismatch diagnostic and both binding-set notes/ranges if available.

## MATCH-BIND-07 — Family candidate join
One family pattern binds `x` from multiple variants of different payload types. Assert joined `TypeKnowledge`.

## MATCH-BIND-08 — GADT proof/common binding distinction
`Expr::Int(x) | Expr::Bool(x)`.
Assert `x: Int | Bool` while neither `T=Int` nor `T=Bool` survives common proof.

---

# P — Pattern Space Algebra Scenarios

These are lower-level semantic laws. Directly test algebra where possible.

## MATCH-SPACE-01 — Normalize Empty
`normalize(Empty) == Empty`.

## MATCH-SPACE-02 — Flatten nested union
`Union(A, Union(B,C)) -> Union(A,B,C)` canonical form.

## MATCH-SPACE-03 — Remove Empty members
`Union(A,Empty) -> A`.

## MATCH-SPACE-04 — Deduplicate
`Union(A,A) -> A`.

## MATCH-SPACE-05 — Preserve exact variant
Exact `VariantSpace` must not become opaque/root enum.

## MATCH-SPACE-06 — Preserve opaque member beside closed member
`Union(Option<Int>, Object)` retains opaque Object component.

## MATCH-SPACE-07 — Intersection identity
`S ∩ S = S`.

## MATCH-SPACE-08 — Intersection empty
`S ∩ Empty = Empty`.

## MATCH-SPACE-09 — Union distribution
`(A|B) ∩ A = A`.

## MATCH-SPACE-10 — Subtract empty
`S - Empty = S`.

## MATCH-SPACE-11 — Self subtraction
`S - S = Empty`.

## MATCH-SPACE-12 — Union subtraction
`(A|B) - A = B`.

## MATCH-SPACE-13 — Root Option minus Some
Residual exactly None.

## MATCH-SPACE-14 — Nested subtraction
`Some(Result) - Some(Ok(_))` preserves `Some(Error(...))`, not root widening.

## MATCH-SPACE-15 — Nested + sibling
`Option<Result> - Some(Ok(_))` preserves `None | Some(Error(...))`.

## MATCH-SPACE-16 — Tuple subtraction
Cover one tuple product region while preserving other component combinations.

## MATCH-SPACE-17 — Opaque conservative subtraction
Matching one known nominal subcase cannot erase opaque domain.

## MATCH-SPACE-18 — Wildcard consumes opaque
Explicit wildcard consumes entire expected opaque space.

---

# X — Exhaustiveness / Usefulness / Witness Scenarios

## MATCH-EXH-01 — Two-case closed enum total
Some + None => `Proven`, final residual empty.

## MATCH-EXH-02 — Missing singleton
Omit None. Assert `MatchNonExhaustive` and witness None exact variant.

## MATCH-EXH-03 — Singleton/nullary distinction in coverage
Cover `Dog` but omit `Dog()`. Assert nullary witness.

## MATCH-EXH-04 — Payload variant totality
All distinct exact constructors covered.

## MATCH-EXH-05 — Family pattern totality
`Dog*` covers every Dog family member but not unrelated Cat.

## MATCH-EXH-06 — Callable family excludes singleton
`Dog(...)` leaves singleton Dog residual.

## MATCH-EXH-07 — Exact-case scrutinee
Known exact case requires only that case.

## MATCH-EXH-08 — Exact-case alias union
Alias A|B exhaustiveness does not require sibling C from enclosing enum.

## MATCH-EXH-09 — Mixed closed + opaque union
Closed ADT arms leave opaque witness.

## MATCH-EXH-10 — Wildcard closes opaque
Add `_`; proven total.

## MATCH-EXH-11 — Nested totality
`Some(Ok(_))`, `Some(Error(_))`, `None`.

## MATCH-EXH-12 — Nested missing witness
Omit `Some(Error(_))`; witness should preserve nested shape.

## MATCH-EXH-13 — Tuple product
Closed ADT components with complete product patterns.

## MATCH-EXH-14 — List partition
`[]` and `[head,*tail]` proven total where supported.

## MATCH-EXH-15 — Open Object
Known cases alone not total; wildcard required.

## MATCH-USE-01 — Wildcard then case redundant
Current HEAD has this basic law. Assert `PatternUsefulness::Redundant` on second arm where product is published and error severity.

## MATCH-USE-02 — Exact duplicate arm redundant
Same exact variant twice.

## MATCH-USE-03 — Family subsumes exact member
`Dog*` before `Dog()` => later redundant.

## MATCH-USE-04 — Or duplicate alternative
`Red | Red` => second alternative redundant.

## MATCH-USE-05 — Family alternative subsumes exact
`Dog* | Dog()` => second alternative redundant.

## MATCH-IMP-01 — GADT impossible
`Expr<Int>` with Bool arm => `Impossible`, not redundant.

## MATCH-IMP-02 — Disjoint union impossible
Pattern variant not in static union.

---

# R — GADT Elimination / Proof Scenarios

## MATCH-GADT-01 — Generic evaluator reachability
Current HEAD includes source regression:
```phalcom
eval<T>(e: Expr<T>) -> T {
    match e {
        Expr::Int(x) => x
        Expr::Bool(x) => x
    }
}
```

**Strengthen existing test. Assert:**
- no diagnostics;
- both exact candidate IDs present;
- arm 0 proof binds enum `T -> Int`;
- arm 1 proof binds enum `T -> Bool`;
- `x` is Int / Bool respectively;
- match result knowledge is T according to formal return context/result model.

**Failure interpretation:** if Bool is dropped under generic `Expr<T>`, implementation is filtering by ordinary subtyping before introducing case equalities.

## MATCH-GADT-02 — `Expr<Int>` excludes Bool
Assert Bool classified impossible with contradiction facts.

## MATCH-GADT-03 — `Expr<Bool>` excludes Int
Mirror.

## MATCH-GADT-04 — Generic root keeps all compatible cases
Add third specialization and prove all remain reachable under type parameter.

## MATCH-GADT-05 — Multi-parameter proof
Use an equality/indexed GADT and assert all declaration-owned equalities in branch proof.

## MATCH-GADT-06 — Nested GADT payload
Parent variant payload contains `Expr<T>`; nested match introduces proof at inner branch only.

## MATCH-GADT-07 — GADT inside union
Union initial pattern space combines specialized case spaces correctly.

## MATCH-GADT-08 — Or proof intersection
Int|Bool same arm retains only common proof facts.

## MATCH-GADT-09 — Proof does not leak sibling arm
Inspect sibling branch proof.

## MATCH-GADT-10 — Proof does not leak after match
Post-match flow/type environment must not globally assert branch-specific equality.

## MATCH-GADT-11 — Blocked is not impossible
Construct/fixture an analysis-blocked compatibility boundary; must never be omitted to fake exhaustiveness.

---

# F — Match Result Typing and Flow Scenarios

## MATCH-FLOW-01 — Homogeneous result
All arms Int => match result Int.

## MATCH-FLOW-02 — Heterogeneous result
Int + String => canonical union `Int | String`.

## MATCH-FLOW-03 — Abrupt arm excluded
One arm `return`/throw, other Int => expression join uses normally completing arm only.

## MATCH-FLOW-04 — All abrupt
Match result is Never/no normal completion according to semantic product.

## MATCH-FLOW-05 — Expected type
Match under expected supertype checks each reachable arm independently.

## MATCH-FLOW-06 — Wrong branch result
One arm violates expected return/annotation; diagnostic range points to offending branch.

## MATCH-FLOW-07 — Stable scrutinee exact refinement
Inside `Some` arm stable local has exact-case knowledge where representable.

## MATCH-FLOW-08 — Family candidate union refinement
Family arm scrutinee narrows to union of candidate exact cases.

## MATCH-FLOW-09 — Binding scope
Pattern binding exists only inside branch.

## MATCH-FLOW-10 — Branch writes join
Outer mutable variable written in both normal arms joins using ordinary flow machinery.

## MATCH-FLOW-11 — Negative knowledge across arms
Later arm reachable space excludes earlier pattern's covered values.

## MATCH-FLOW-12 — Nested residual not forced into TypeId
Coverage retains precise nested `PatternSpace` even if flow projection must use a broader type.

---

# N — Diagnostic / Explanation Scenarios

Provide one enabled scenario for each code.

## MATCH-DIAG-01 — `match.pattern.variant_unresolved`
Explicit qualified/bare variant form has no candidate.
Assert base range primary.

## MATCH-DIAG-02 — `match.pattern.variant_ambiguous`
Contextual shorthand resolves multiple owners.
Assert candidate owner notes.

## MATCH-DIAG-03 — `match.pattern.variant_inaccessible`
Explicit private name.
Assert name/base range.

## MATCH-DIAG-04 — `match.pattern.payload_inaccessible`
Attempt binding/nested payload test on inaccessible field.
Assert field/pattern range.

## MATCH-DIAG-05 — `match.pattern.selector_invalid`
Syntactically valid pattern projects invalid selector constraint according to semantic selector rules.

## MATCH-DIAG-06 — `match.pattern.selector_no_candidate`
Valid selector pattern matches no family member.

## MATCH-DIAG-07 — `match.pattern.shape_mismatch`
Exact callable pattern wrong arity/shape.

## MATCH-DIAG-08 — `match.pattern.label_mismatch`
Wrong external label.

## MATCH-DIAG-09 — `match.pattern.duplicate_binding`
Strengthen current HEAD regression.

## MATCH-DIAG-10 — `match.pattern.or_binding_mismatch`
Strengthen current HEAD regression.

## MATCH-DIAG-11 — `match.pattern.or_redundant`
Second or alternative contributes no space.

## MATCH-DIAG-12 — `match.pattern.impossible`
GADT contradiction; notes should explain scrutinee equality vs case equality.

## MATCH-DIAG-13 — `match.arm.redundant`
Strengthen current HEAD regression with range/usefulness product.

## MATCH-DIAG-14 — `match.non_exhaustive`
Assert structured witness and help for wildcard when residual opaque.

## MATCH-DIAG-15 — `match.analysis.blocked`
Blocked proof does not become false `Proven`.

For every diagnostic above, if explanation DAG integration exists:
- assert diagnostic has explanation reference;
- inspect step category relevant to candidate/proof/coverage;
- do not snapshot entire debug DAG.

---

# I — Incremental / Fingerprint Scenarios

## ADT-INCR-01 — Add enum case
Initial A|B match exhaustive. Edit enum add C without changing match source.
Assert callable match analysis invalidates and now reports missing C.

## ADT-INCR-02 — Remove enum case
Candidate universe shrinks; match product/fingerprint changes.

## ADT-INCR-03 — Add family member
`Dog*` candidate set changes; remains exhaustive for Dog family where appropriate.

## ADT-INCR-04 — Add callable family member affecting `Dog(...)`
Selector-pattern candidate set changes.

## ADT-INCR-05 — Change payload type
Binding type/product invalidates.

## ADT-INCR-06 — Change GADT result specialization
Branch reachability/proof invalidates.

## ADT-INCR-07 — Alias union expansion
Alias A|B -> A|B|C makes previous match non-exhaustive.

## ADT-INCR-08 — Alias contraction
Residual/witness updates.

## ADT-INCR-09 — Visibility edit
Explicit acquisition may become invalid/valid; exhaustiveness universe remains semantically correct.

## ADT-INCR-10 — Unrelated method edit
Match product should be reusable if dependency graph says unrelated.

## ADT-INCR-11 — Source-range/whitespace-only edit
Semantic fingerprint should not change solely because rendered/source ranges changed, where fingerprint design excludes them.

## ADT-INCR-12 — Candidate semantic change
Exact candidate set change must alter product fingerprint.

**Debug:** inspect recorded `SemanticDependency::EnumDeclaration`, `AssociatedSurface`, type/alias resolution dependencies, and callable body product fingerprint components.

---

# L — Semantic-to-Core Lowering Scenarios

## ADT-LOWER-01 — One match site
One semantic `MatchResolution` produces one `LoweringSiteKind::Match`.

## ADT-LOWER-02 — Multiple match sites
Two match expressions have distinct source-keyed sites.

## ADT-LOWER-03 — Exact candidate identity
Executable candidate retains exact semantic `VariantId`.

## ADT-LOWER-04 — Physical payload slot
A labeled/mixed-field variant maps `VariantFieldId` to correct u16 slot.

## ADT-LOWER-05 — Candidate-specific slots
Partial selector family with different variant layouts maps same source binding from different slots per candidate.

## ADT-LOWER-06 — Wildcard elision
Wildcard-only child produces no executable extraction projection.

## ADT-LOWER-07 — Selector-gap elision
Gap-consumed fields are not extracted.

## ADT-LOWER-08 — Cross-module field layout
Consumer module match lowers field declared in another module correctly.

## ADT-LOWER-09 — Candidate order
Preserve semantic order.

## ADT-LOWER-10 — Proof erasure
Executable IR contains no `CaseTypeEnvironment`, GADT substitution, `PatternSpace`, witnesses.

## ADT-LOWER-11 — Missing semantic product
Compiler returns typed internal missing-lowering error; no source fallback.

## ADT-LOWER-12 — Non-proven match
Executable lowering rejects match not semantically accepted/proven.

---

# Q — Runtime Match Scenarios

## ADT-RUN-01 — Singleton execution
Construct/get singleton and match exact case. Assert selected result and `IsVariant`.

## ADT-RUN-02 — Nullary constructor execution
Two constructed nullary values may be distinct runtime values but share exact variant identity. Match succeeds by `VariantId`, not singleton identity.

## ADT-RUN-03 — Singleton/nullary distinction
Enum contains both. Each pattern selects only its shape.

## ADT-RUN-04 — Payload binding
Match payload and return bound value. Assert `GetVariantPayload(0)`.

## ADT-RUN-05 — Labeled payload
Correct semantic field slot used; compiler never reads source label to compute runtime slot.

## ADT-RUN-06 — Nested ADT
Two levels of `IsVariant` and extraction; final binding correct.

## ADT-RUN-07 — Wildcard payload
No `GetVariantPayload` when field is entirely ignored.

## ADT-RUN-08 — Or-pattern
First alternative failure then second success commits second value.

## ADT-RUN-09 — Family pattern
Multiple exact candidate tests, no runtime family object.

## ADT-RUN-10 — Selector-gap family
Candidate-specific projections produce same branch binding destinations.

## ADT-RUN-11 — Match expression result
Selected branch value becomes expression value.

## ADT-RUN-12 — Braced arm tail
Block tail is branch result, compiled inline.

## ADT-RUN-13 — Return inside braced arm
Returns from enclosing callable, proving no closure-valued arm.

## ADT-RUN-14 — Scrutinee once
Side-effect counter increments once despite many candidates/arms.

## ADT-RUN-15 — Selected branch once
Only selected branch's counter changes.

## ADT-RUN-16 — Exhaustive fallthrough invariant
Generated bytecode includes internal trap after all test failures if implementation retains it; no public MatchError.

## ADT-RUN-17 — GADT runtime erasure
Generic evaluator emits only case/payload bytecodes, no runtime equality/type-specialization test.

---

# PC — Shared Pattern Context Scenarios

## PAT-CTX-01 — `if let` nested success
Same formal executable pattern as match; success binding correct.

## PAT-CTX-02 — `if let` nested failure no leak
Outer case succeeds, inner fails. Else path sees no committed binding.

## PAT-CTX-03 — `if let` or-pattern
Either successful alternative exposes one shared binding.

## PAT-CTX-04 — `while let` one RHS evaluation per iteration
Count producer calls equals attempted iterations.

## PAT-CTX-05 — `while let` failed final iteration no leak
Partially matched final value must not commit bindings.

## PAT-CTX-06 — Required destructuring
Uses shared emitter but mismatch follows required-pattern error semantics.

## PAT-CTX-07 — `for` pattern
Where general patterns are supported, uses shared executable pattern and correct per-item binding.

## PAT-CTX-08 — Captured pattern binding
Closure captures committed visible binding, not hidden staging slot.

**Debug bytecode:** visible `SetLocal`/commit should occur only on common success edge after full pattern success.

---

# GC — Runtime Ownership / GC Scenarios

## ADT-GC-01 — Payload object traced
ADT case containing heap object keeps payload alive.

## ADT-GC-02 — Nested ADT traced
Outer case keeps nested case and nested payload alive.

## ADT-GC-03 — Singleton descriptor/owner survives required lifetime
No dangling variant behavior identity.

## ADT-GC-04 — Family descriptor traces owner/members
Captured family remains usable across GC.

## ADT-GC-05 — Unreachable case collected
No permanent leak from runtime variant registry beyond intended descriptors.

## ADT-GC-06 — Match scratch roots payload
Force GC after extraction/staging before branch use if harness allows; bound payload remains alive.

## ADT-GC-07 — Closure capture after match arm
Closure returned from selected arm retains bound object after arm scratch cleanup.

---

# VERT — Vertical Conformance Programs

These tests intentionally cross layers. Keep the number small and assertions rich.

## ADT-VERT-01 — Generic Result
**Program:** generic `Result<T,E>` with Ok/Err, constructor use, exact case types, exhaustive match, payload bindings.

**Semantic assertions:**
- variant/family IDs;
- constructor specialization;
- match candidates;
- binding types;
- proven exhaustiveness;
- result type.

**Core assertions:**
- executes Ok and Err paths;
- exact case bytecodes;
- correct payload slots.

## ADT-VERT-02 — GADT evaluator
Use `Expr<T>` Int/Bool.

**Semantic:** branch equality proofs, specialized bindings, result.
**Core:** proof-erased `IsVariant`/payload execution for both cases.

## ADT-VERT-03 — Multi-selector Dog family
Include `Dog`, `Dog()`, `Dog(_)`, `Dog(named:)`.

**Semantic:** four IDs, one family, exact/selector/family pattern candidate sets.
**Core:** each runtime value routes to correct match arm; singleton/nullary stay distinct.

## ADT-VERT-04 — Nested Option<Result>
```phalcom
match value {
    Some(Ok(x) | Cached(x)) => x
    Some(Error(_, reason: message)) => recover(message)
    None => fallback()
}
```
Use equivalent actual declared variants if Cached/Error shapes differ.

**Semantic:** nested candidates, shared or binding, field IDs, residual after each arm, proven totality.
**Core:** nested/or execution and no partial binding leak.

## ADT-VERT-05 — Visibility
Variant cannot be constructed externally but remains in exhaustiveness universe; wildcard can cover inaccessible case; payload privacy enforced separately.

## ADT-VERT-06 — Cross-module
Module A declares generic enum with labeled payload. Module B imports and matches it.

**Semantic:** exact imported `VariantId` and field ID.
**Lowering:** cross-module physical slot.
**Runtime:** correct payload result; no leaf-name/global-class lookup.

---

# Failure Triage Appendix

## A. A positive semantic test suddenly emits diagnostics
1. Print all machine codes and ranges.
2. Determine whether failure occurs before match analysis (parse/declaration/type resolution) or inside pattern/match.
3. Do not suppress unrelated diagnostics just to reach the target assertion; fix fixture if it accidentally violates another law.

## B. Candidate set wrong
Inspect:
```text
expected PatternSpace
owner candidates
family membership
selector constraint
SelectorPattern::matches result
GADT compatibility classification
visibility filtering
```

## C. Binding type wrong
Inspect each candidate's specialized field type first, then the join. A wrong union may originate from candidate formation, not type join.

## D. Non-exhaustive when it should be exhaustive
Dump:
```text
initial
arm matched
intersect(remaining, matched)
residual_after
```
per arm. Compare exact variant IDs before inspecting witnesses.

## E. Falsely exhaustive
Treat as soundness-critical. Look for:
- blocked GADT candidate dropped as impossible;
- opaque residual discarded;
- alias union widened/contracted incorrectly;
- nested subtraction widened to root and then consumed.

## F. Runtime mismatch with correct semantic product
Never patch semantic tests first. Inspect lowering:
```text
semantic VariantId
runtime target index
VariantFieldId
physical slot
emitted bytecode
runtime registry mapping
```

## G. Incremental stale result
Compare dependency sets and fingerprints; re-run from fresh database to separate inference bug from invalidation bug.

---

# Suggested Scenario Count

The catalog defines roughly:
- declarations/variants/constructors/exact/generic: 35;
- behavior/requirements: 8;
- associated/family: 16;
- match resolution/patterns/bindings: 30;
- pattern-space algebra: 18;
- exhaustiveness/usefulness/impossible: 22;
- GADT elimination: 11;
- flow: 12;
- diagnostics: 15;
- incremental: 12;
- lowering: 12;
- runtime: 17;
- shared pattern contexts: 8;
- GC: 7;
- vertical: 6.

Total: approximately **229 explicitly described scenarios/laws**.

This number is intentionally not the required number of Rust test functions. Table-driven tests and multi-assertion source scenarios are encouraged where they keep one coherent law together.
