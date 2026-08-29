# Part 04 — Predicate Proofs, Branch Refinement, and Constant Branches Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make path-sensitive branch reasoning sound by distinguishing trusted observations from assumption-dependent filtering, preserving evidence strength through narrowing, turning established contradictions into unreachable paths, preventing overloaded syntax from authorizing proof, and pruning trivially constant branches.

**Architecture:** Keep `FlowPredicate`, `FactSet`, `FlowState`, and the Part-3 control-region/branch executor. Replace the current `Option<AppliedFlowRefinement>` transfer contract with an explicit `PredicateTransfer` outcome and attach a small `PredicateAuthority` to extracted predicates. Trusted runtime type tests can supply new established evidence; negative filters and other narrowing operations preserve the authority of the premises they depend on. Contradictions terminate only when they conflict with established knowledge. Add a tiny `ConditionTruth` evaluator for literal booleans and negation; do not build a general constant evaluator.

**Tech Stack:** Rust, `phalcom-semantic`, `FlowPredicate`, `FlowState`, `TypeKnowledge`, `EvidenceStatus`, `TypeHierarchy`, `RelationOutcome`, Part-3 `checker/control.rs`, `TypedExpression.callable`, canonical `CallableId`, semantic capability/foundation tests, deterministic semantic fingerprints.

**Spec:** This plan implements Part 4 of the ratified six-part typing-correctness architecture. It depends on Parts 1–3. In particular, Part 1 supplies evidence monotonicity, Part 3 supplies canonical branch execution and reachability, and this plan changes only the proof applied at those branch boundaries. Repository source is authoritative; the grounding revision below records the pre-Part-1/2/3 shape that must be re-resolved after those plans land.

## Repository grounding

Freshly grounded against `aureat/phalcom-lang` `main` at:

```text
24fc9fd98f3c3c534c4d52b613962a39b9374185
feat(semantic): add rich type diagnostics tests and polish presentation
```

Current predicate/branch anchors at that revision:

- `phalcom-semantic/src/checker/flow/predicate.rs`
  - `FlowPredicate` already models type tests, nil checks, equality, literal equality, ordered predicates, and truthiness;
  - `extract_predicate(...)` recognizes syntax by shape/spelling;
  - `extract_trusted_predicate(...)` performs canonical `Object#is`/`Object#is!` identity validation only for type-test predicates;
  - non-type-test predicates currently bypass semantic-identity validation.
- `phalcom-semantic/src/checker/flow/transfer.rs`
  - `apply_predicate(...) -> Option<AppliedFlowRefinement>`;
  - positive `IsInstance` writes `Established` target/refined knowledge;
  - negative `IsNotInstance` union filtering writes `Established` residual knowledge regardless of prior status;
  - nil/equality filtering likewise writes established facts;
  - no explicit contradiction result exists.
- `phalcom-semantic/src/checker/context.rs`
  - `apply_flow_predicate(...)` assumes transfer either refined or did nothing;
  - it records `FlowRefinement` explanations but has no contradiction/unreachable path result.
- `phalcom-semantic/src/checker/expression.rs`
  - branch execution currently asks `extract_trusted_predicate` for true/false paths;
  - Part 3 moves this orchestration to `checker/control.rs` and makes reachability canonical.
- `phalcom-semantic/src/checker/flow/state.rs`
  - `mark_unreachable()` and reachable-only joins already exist;
  - this is the correct representation for a proven contradictory branch.
- `phalcom-semantic/src/types/evidence.rs`
  - `TypeKnowledge::derive_known_type` and `map_type` preserve evidence status;
  - Part 1 adds/standardizes `EvidenceStatus::meet`.
- `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
  - already protects canonical type-test refinement and the rule that an overridden method named `is` cannot refine;
  - lacks authority-preservation and contradiction matrices.
- `phalcom-semantic/tests/semantic/capabilities/authority.rs`
  - already provides useful source-assumption vs established-evidence fixtures.

The current transfer implementation is small and should remain small. This plan does not replace it with a theorem prover or a generic refinement calculus.

---

# 1. Dependency gates

Part 4 begins only after Part 3 is GREEN.

Required Part-3 properties:

```text
- checker/control.rs owns executable branch orchestration
- FlowState::reachable is the sole local fallthrough authority
- abrupt branch results have no normal value
- branch joins use only reachable normal completions
- if-let no longer executes through Expr::Block
```

Before editing predicate code:

```sh
git status --short
git rev-parse HEAD
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

Expected: Part 3 GREEN.

---

# 2. Problem statement

Predicate transfer answers two different questions that the current implementation conflates:

```text
Observation:
    Did this branch condition itself establish a new fact?

Filtering:
    Given existing possible types, what possibilities survive this branch?
```

Those operations have different authority.

Example 1 — authoritative positive observation:

```text
prior: Unknown
condition: canonical value.is(Int) is true
result: Established<Int>
```

The runtime type test itself supplies new evidence.

Example 2 — assumption-dependent negative filtering:

```text
prior: Assumed<Int | String>
condition: canonical value.is(Int) is false
result: Assumed<String>
```

The runtime observation proves “not Int”, but `String` comes from the developer's assumed exhaustiveness of `Int | String`. It cannot become established.

Current code produces `Established<String>` in the second case.

A third distinction is contradiction. If the checker has already established `String` and a trusted branch requires the same value to be `Int`, that branch is impossible. But if `String` was only a developer assumption, the runtime test is allowed to reveal that the assumption was wrong; the branch is not impossible merely because the annotation disagrees.

---

# 3. Required semantic laws

## Law P1 — Predicate authority is explicit

Every predicate applied to formal flow carries an authority classification describing whether the condition supplies an independent observation or merely authorizes filtering of existing evidence.

## Law P2 — Filtering cannot strengthen its premise

For a transformation that derives a residual/subset from prior alternatives:

```text
authority(refined) <= authority(prior)
```

unless the refined type is fully established by an independent authoritative observation.

## Law P3 — Trusted positive runtime type tests can establish the tested type

For canonical `Object#is` / `Object#is!`, a true type-test branch can establish the target type when the target itself is the sufficient result.

Examples:

```text
Unknown                  + is(Int) true -> Established<Int>
Assumed<Int | String>    + is(Int) true -> Established<Int>
Established<Object>      + is(Int) true -> Established<Int>
```

## Law P4 — A broader observation does not establish a narrower assumption

```text
Assumed<Cat | Dog> + is(Animal) true
```

must remain assumption-dependent if the result retained is `Cat | Dog`. The observation proves `Animal`, not the narrower union.

## Law P5 — Negative observations preserve assumption-dependent residuals

```text
Assumed<Int | String> + not Int -> Assumed<String>
Established<Int | String> + not Int -> Established<String>
```

## Law P6 — Only established contradictions make a path impossible

```text
Established<String> + is(Int) true -> Contradiction / unreachable
Established<Int>    + is(Int) false -> Contradiction / unreachable
```

By contrast:

```text
Assumed<String> + is(Int) true
```

is not a contradiction. A trusted runtime observation can refute a developer assumption.

## Law P7 — Refuted assumptions degrade rather than fabricate

When a negative trusted observation eliminates the only assumed concrete type and no residual type is representable:

```text
Assumed<Int> + not Int -> Unknown(InferenceConflict)
```

The branch stays reachable. Do not keep `Assumed<Int>`, do not mark it unreachable, and do not invent a top type unless a canonical top-type proof is explicitly available.

`InferenceConflict` is used because its existing contract-assumption eligibility prevents the source contract from immediately laundering the same contradicted assumption back into current knowledge.

## Law P8 — Dynamic remains an explicit escape

Do not silently convert `Dynamic` to established static knowledge in this part. Trusted predicates over `Dynamic` may be represented in `FactSet`, but current `TypeKnowledge` remains `Dynamic` unless/until Phalcom explicitly ratifies dynamic-to-static narrowing semantics.

## Law P9 — Method spelling is never proof

User-defined or overridden methods named:

```text
is
is!
==
!=
```

must not authorize formal type refinement solely by name.

## Law P10 — Contradiction is a control result

Predicate transfer reports contradiction explicitly. `CheckingContext`/Part-3 control orchestration marks the path unreachable. Do not encode contradiction by assigning `Never` as the binding's ordinary current type.

## Law P11 — Constant branch pruning is intentionally tiny

Part 4 recognizes only:

```text
true
false
not <recognized constant boolean>
```

for `ConditionTruth`. No arithmetic folding, string comparison, method evaluation, or user-defined constant execution is introduced.

## Law P12 — Skipped constant branches do not execute semantically

An `if true` false arm and an `if false` true arm contribute no flow, value, diagnostics from expression checking, field writes, or callable exits.

Parsing still validates syntax; unreachable-code linting is outside this plan.

---

# 4. Target data model

## 4.1 Predicate authority

Add to `checker/flow/predicate.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PredicateAuthority {
    /// The condition itself supplies a runtime/compiler-trusted observation.
    AuthoritativeObservation,
    /// Refinement depends on existing formal knowledge and may not strengthen it.
    DerivedFilter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedFlowPredicate {
    pub predicate: FlowPredicate,
    pub authority: PredicateAuthority,
}
```

`TrustedFlowPredicate` means “safe to feed to formal flow.” It does not mean every result is established.

## 4.2 Predicate transfer result

Replace `Option<AppliedFlowRefinement>` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredicateTransfer {
    Unchanged,
    Refined(AppliedFlowRefinement),
    Contradiction {
        binding: BindingId,
        prior: TypeKnowledge,
    },
}
```

The transfer function becomes:

```rust
pub fn apply_predicate(
    state: &mut FlowState,
    predicate: &TrustedFlowPredicate,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer;
```

Do not mark reachability inside the pure transfer function. `flow/transfer.rs` computes the result; `CheckingContext::apply_flow_predicate` applies the control consequence.

## 4.3 Condition truth

Add to Part-3 `checker/control.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConditionTruth {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

pub(crate) fn condition_truth(expr: &Expr) -> ConditionTruth {
    match expr {
        Expr::Boolean { value: true, .. } => ConditionTruth::AlwaysTrue,
        Expr::Boolean { value: false, .. } => ConditionTruth::AlwaysFalse,
        Expr::Unary(unary) if matches!(unary.op, UnaryOp::Not) => match condition_truth(&unary.expr) {
            ConditionTruth::AlwaysTrue => ConditionTruth::AlwaysFalse,
            ConditionTruth::AlwaysFalse => ConditionTruth::AlwaysTrue,
            ConditionTruth::Unknown => ConditionTruth::Unknown,
        },
        _ => ConditionTruth::Unknown,
    }
}
```

Adapt exact AST field names to the parser's current `Boolean` representation; do not extend the recognized set in this plan.

---

# 5. Trust model

## 5.1 Canonical type tests

Keep the existing canonical identity rule:

```text
owner    = core::Object
selector = is(_:) or is!(_:)
side     = Instance
```

A resolved call target matching that canonical `CallableId` produces:

```rust
PredicateAuthority::AuthoritativeObservation
```

## 5.2 Equality/nil predicates

Formal equality-derived narrowing is permitted only when the analyzed condition's resolved callable identity is a compiler/native canonical equality operation that guarantees the semantics required by the refinement.

At the grounding revision, `extract_predicate` recognizes `==`/`!=` syntax independently of callable identity. Change the formal path so this syntax is **not** trusted automatically.

Implement a small local helper in `predicate.rs`:

```rust
fn is_canonical_core_equality(callable: Option<&CallableId>) -> bool {
    // compare exact owner/selector/side against the canonical core equality callable(s)
}
```

Use actual canonical callable IDs from the registered core/native surface after rebasing. Do not compare owner/type names obtained from source text.

If a syntax form has no canonical semantic identity available in the current core surface, return `None` from `extract_trusted_predicate` for that refinement. Sound loss of narrowing is preferable to unsound proof.

This plan does not create the Part-6 `CoreSemanticIds` registry; one or two local canonical-ID helpers are acceptable here and are explicitly scheduled for consolidation in Part 6.

## 5.3 Literal/ordered predicates

`EqualLiteral`, `NotEqualLiteral`, and `OrderedPredicate` currently do not change `TypeKnowledge`; they only enter the fact set. Because fact sets are formal proof state, only emit them when their operation is semantically trusted.

Do not retain syntax-only formal facts for user-overloadable comparisons.

## 5.4 Truthy/falsy

A direct condition on a binding may retain `Truthy`/`Falsy` as `DerivedFilter` because it does not currently manufacture a concrete static type. If future code starts deriving a type from these facts, that consumer must obey the same authority rules.

---

# 6. Transfer rules

These rules are normative and should be encoded as focused unit tests before implementation.

## 6.1 Positive `IsInstance`

Let `P` be prior knowledge and `T` target type.

### Prior Unknown

Trusted authoritative observation:

```text
Unknown + is(T) true -> Established<T>
```

Derived/non-authoritative predicate:

```text
Unknown -> unchanged
```

### Prior Dynamic

```text
Dynamic -> unchanged Dynamic
```

### Prior established known type

If established prior and target are proven disjoint:

```text
Contradiction
```

Otherwise compute the most precise representable intersection using existing nominal/union relations.

```text
Established<Object>       + is(Int) -> Established<Int>
Established<Int|String>   + is(Int) -> Established<Int>
Established<Cat|Dog>      + is(Animal) -> Established<Cat|Dog>
```

Because the prior is established, retaining its narrower detail is sound.

### Prior assumed known type

If `T <: prior_type` is proven, the authoritative observation alone is sufficient to establish `T`:

```text
Assumed<Int|String> + is(Int) -> Established<Int>
Assumed<Object>     + is(Int) -> Established<Int>
```

If the result is narrower than `T` only because of the prior assumption, preserve assumed authority:

```text
Assumed<Cat|Dog> + is(Animal) -> Assumed<Cat|Dog>
```

If the prior assumption is incompatible with `T`, do not mark contradiction; trusted observation wins:

```text
Assumed<String> + is(Int) -> Established<Int>
```

Any persistent binding contract remains separate and may become refuted when reconciled by later contract-sensitive operations. The flow fact itself follows the new authoritative observation.

## 6.2 Negative `IsNotInstance`

### Union prior

Filter members proven subtypes of `T`.

```text
Established<Int|String> - Int -> Established<String>
Assumed<Int|String>     - Int -> Assumed<String>
```

Use `prior.derive_known_type(refined, EvidenceOrigin::Flow)` or equivalent so filtering preserves evidence status/provenance.

### Established prior wholly inside target

```text
Established<Int> - Int -> Contradiction
```

### Assumed prior wholly inside target

```text
Assumed<Int> - Int -> Unknown(InferenceConflict)
```

and keep the path reachable.

### Unknown/Dynamic/non-representable residual

Leave unchanged unless the prior assumption has been positively contradicted as above.

## 6.3 Nil/equality

Apply the same positive/negative authority rules, with canonical `Unit`/nil type identity only after the predicate itself has passed semantic trust validation.

Do not hard-code “string named `None` means Unit” as the proof boundary. Resolve the canonical semantic value/type through existing declaration/denotation machinery. If the current source syntax still uses `None` as a compatibility spelling, its semantic target must resolve canonically before refinement.

## 6.4 Literal and ordered predicates

No `TypeKnowledge` mutation in Part 4. Record only trusted facts.

---

# 7. File/ownership map

| Area | File | Responsibility after Part 4 |
|---|---|---|
| Predicate representation/trust | `checker/flow/predicate.rs` | `FlowPredicate`, `PredicateAuthority`, `TrustedFlowPredicate`, canonical semantic identity validation |
| Pure refinement transfer | `checker/flow/transfer.rs` | `PredicateTransfer`, authority-preserving narrowing, contradiction detection |
| Flow exports | `checker/flow/mod.rs` | Export new predicate/transfer types |
| Context application | `checker/context.rs` | Convert contradiction to unreachable flow; record refinement explanations |
| Branch execution | `checker/control.rs` | Ask for trusted predicates, prune constant arms, join remaining normal results |
| Expression synthesis | `checker/expression.rs` | No predicate-specific branch duplication after Part 3 |
| Foundation tests | new `tests/semantic/foundations/predicate_transfer.rs` or module-local tests | Direct authority/contradiction matrix |
| Capability tests | `tests/semantic/capabilities/flow_branches.rs` | Real-source narrowing, impossible branch, overload spoofing, constant branches |
| Test wiring | `tests/semantic/foundations/mod.rs` if new file | Register predicate transfer foundation tests |
| Fingerprints | `db/fingerprint.rs` | Only if published semantic product changes; ephemeral authority enums need no hashing by themselves |

Prefer direct unit/foundation tests for the transfer algebra and source integration tests for semantic identity + branch composition.

---

# 8. Execution order

```text
Task 0  Rebase/gate on Part 3
   |
Task 1  Introduce PredicateAuthority + explicit transfer outcome
   |
Task 2  Fix positive type-test authority
   |
Task 3  Fix negative filtering + contradiction semantics
   |
Task 4  Tighten trusted predicate extraction for equality/comparisons
   |
Task 5  Integrate contradiction with branch reachability/explanations
   |
Task 6  Add literal boolean branch pruning
   |
Task 7  Composition audit and final closure gate
```

---

## Task 0: Rebase onto Part 3 and capture the predicate baseline

**Files:**
- Read: final `checker/control.rs`
- Read: `checker/flow/predicate.rs`
- Read: `checker/flow/transfer.rs`
- Read: `checker/context.rs`
- Read: `tests/semantic/capabilities/flow_branches.rs`

**Interfaces:**
- Consumes: Part-3 `analyze_branch_pair` / branch-join helper and canonical reachability.
- Produces: exact post-Part-3 symbol map in implementation work log.

- [ ] **Step 1: Verify baseline.**

```sh
git status --short
git rev-parse HEAD
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

- [ ] **Step 2: Locate all predicate producers/consumers.**

```sh
rg "extract_predicate|extract_trusted_predicate|apply_predicate|apply_flow_predicate|FlowPredicate::" phalcom-semantic/src phalcom-semantic/tests
```

Every formal producer must be accounted for by Task 4's trust audit.

- [ ] **Step 3: Record canonical callable IDs currently used for type tests/equality.**

Use the semantic/native surface code, not method spelling from a fixture. Record exact owner/side/selector identities in the work log.

---

## Task 1: Introduce explicit predicate authority and transfer outcomes

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/predicate.rs`
- Modify: `phalcom-semantic/src/checker/flow/transfer.rs`
- Modify: `phalcom-semantic/src/checker/flow/mod.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` for compile migration only.
- Test: create `phalcom-semantic/tests/semantic/foundations/predicate_transfer.rs` if the foundation harness supports direct construction; otherwise use module-local tests in `transfer.rs`.

**Interfaces:**
- Produces: `PredicateAuthority`, `TrustedFlowPredicate`, `PredicateTransfer`.

- [ ] **Step 1: Write RED transfer-shape tests.**

At minimum:

```rust
#[test]
fn unchanged_predicate_is_explicit() { /* expect PredicateTransfer::Unchanged */ }

#[test]
fn narrowing_predicate_reports_refined_prior_and_result() { /* expect Refined */ }

#[test]
fn impossible_established_predicate_reports_contradiction() { /* expect Contradiction */ }
```

- [ ] **Step 2: Add `PredicateAuthority` and `TrustedFlowPredicate`.**

Use the exact target definitions above.

- [ ] **Step 3: Change transfer signature to return `PredicateTransfer`.**

Mechanical behavior first:

```text
current Some(refinement) -> Refined
current None             -> Unchanged
```

Do not fix authority in this step yet; keep it compiling so the data-model change is separately reviewable.

- [ ] **Step 4: Change `CheckingContext::apply_flow_predicate`.**

Return an application result that the branch executor can inspect. A small context-level type is sufficient:

```rust
pub(crate) enum FlowPredicateApplication {
    Unchanged,
    Refined(ExplanationId),
    Contradiction,
}
```

On `Contradiction`, mark `self.flow` unreachable. Do not mutate the binding to `Never`.

- [ ] **Step 5: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic checker::flow -- --nocapture
git add phalcom-semantic/src/checker/flow phalcom-semantic/src/checker/context.rs
git commit -m "refactor(semantic): make predicate authority and outcomes explicit"
```

---

## Task 2: Make positive canonical type tests establish only what the observation proves

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/transfer.rs`
- Test: foundation predicate transfer tests.
- Test: `tests/semantic/capabilities/flow_branches.rs`.

**Interfaces:**
- Consumes: Part-1 `EvidenceStatus::meet`, subtype relation, union representation.
- Produces: sound positive `IsInstance` transfer.

- [ ] **Step 1: Add the RED authority matrix.**

Direct transfer tests must include:

```text
Unknown + authoritative is(Int) -> Established<Int>
Assumed<Object> + authoritative is(Int) -> Established<Int>
Assumed<Int|String> + authoritative is(Int) -> Established<Int>
Established<Object> + authoritative is(Int) -> Established<Int>
Established<Int|String> + authoritative is(Int) -> Established<Int>
Assumed<Cat|Dog> + authoritative is(Animal) -> Assumed<Cat|Dog>
Established<Cat|Dog> + authoritative is(Animal) -> Established<Cat|Dog>
Dynamic + authoritative is(Int) -> Dynamic unchanged
```

- [ ] **Step 2: Add source integration test for assumed parameter narrowing.**

Callable parameters are currently source-contract assumptions, making them a good integration fixture:

```rust
#[test]
fn canonical_type_test_can_establish_exact_target_from_assumed_parameter() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Object) {
    if (value.is(Int)) {
      let narrowed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "narrowed", f.ty("Int"));
}
```

This test is intentionally different from “parameter assumption becomes established automatically”: only the trusted branch observation establishes `Int`.

- [ ] **Step 3: Implement representable intersection helper.**

Keep it local to `transfer.rs`:

```rust
fn positive_type_refinement(
    prior: &TypeKnowledge,
    target: TypeId,
    authority: PredicateAuthority,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer
```

Use existing union member filtering and subtype checks. Do not add arbitrary intersection types.

- [ ] **Step 4: Preserve assumption when retained precision depends on prior.**

Use relation checks to distinguish:

```text
target <: prior     -> observation can establish target
prior <: target     -> retained prior precision depends on prior status
partial union match -> preserved prior status unless prior itself established
```

- [ ] **Step 5: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic canonical_type_test_can_establish_exact_target_from_assumed_parameter -- --nocapture
cargo test -p phalcom-semantic checker::flow::transfer -- --nocapture
git add phalcom-semantic/src/checker/flow/transfer.rs phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "fix(semantic): preserve proof authority in positive refinements"
```

---

## Task 3: Fix negative filtering and established-only contradiction pruning

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/transfer.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Test: foundation transfer tests.
- Test: `tests/semantic/capabilities/flow_branches.rs`.

**Interfaces:**
- Consumes: `TypeKnowledge::derive_known_type`, `EvidenceStatus`, `UnknownReason::InferenceConflict`.
- Produces: status-preserving residual filtering and contradiction outcome.

- [ ] **Step 1: Add RED negative authority tests.**

```text
Established<Int|String> - Int -> Established<String>
Assumed<Int|String>     - Int -> Assumed<String>
Unknown                 - Int -> unchanged Unknown
Dynamic                 - Int -> unchanged Dynamic
```

The crucial regression is:

```rust
assert_eq!(refined.status(), Some(EvidenceStatus::Assumed));
```

for assumed union filtering.

- [ ] **Step 2: Add RED contradiction tests.**

```text
Established<Int> + IsNotInstance(Int) -> Contradiction
Established<String> + IsInstance(Int) -> Contradiction when nominally disjoint
Assumed<Int> + IsNotInstance(Int) -> reachable Unknown(InferenceConflict)
Assumed<String> + IsInstance(Int) -> reachable Established<Int> under authoritative observation
```

- [ ] **Step 3: Implement negative union filtering with status preservation.**

Replace constructions like:

```rust
TypeKnowledge::established(refined, EvidenceOrigin::Flow)
```

with a status-preserving transformation from `prior`:

```rust
prior.derive_known_type(refined, EvidenceOrigin::Flow)
```

or a public/internal helper with equivalent provenance behavior.

- [ ] **Step 4: Detect contradiction only from established proof.**

Before returning contradiction, require:

```rust
prior.status() == Some(EvidenceStatus::Established)
```

and a formal relation showing the predicate cannot hold.

Do not infer disjointness from unequal `TypeId`s alone; use nominal/union relation rules.

- [ ] **Step 5: Degrade contradicted assumptions.**

For `Assumed<T>` plus trusted negative `T`, assign:

```rust
TypeKnowledge::Unknown(UnknownReason::InferenceConflict)
```

and return `Refined` rather than `Contradiction`.

- [ ] **Step 6: Add source integration test for impossible established branch.**

Use an established literal/local:

```rust
#[test]
fn trusted_type_test_prunes_branch_contradicting_established_literal_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = "hello"
    let result = if (value.is(Int)) {
      1
    } else {
      "ok"
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "result", f.ty("String"));
}
```

This is a proof test, not constant folding: the condition is runtime syntax, but the established prior type makes the true path contradictory.

- [ ] **Step 7: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic trusted_type_test_prunes_branch_contradicting_established_literal_type -- --nocapture
cargo test -p phalcom-semantic checker::flow::transfer -- --nocapture
git add phalcom-semantic/src/checker/flow/transfer.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "fix(semantic): make negative refinements authority preserving"
```

---

## Task 4: Tighten predicate extraction so overloaded syntax cannot become formal proof

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/predicate.rs`
- Test: `tests/semantic/capabilities/flow_branches.rs`
- Test: foundation predicate tests as useful.

**Interfaces:**
- Consumes: `TypedExpression.callable`, canonical core `CallableId` construction, `FlowPredicate` syntax extraction.
- Produces: `extract_trusted_predicate(...) -> Option<TrustedFlowPredicate>`.

- [ ] **Step 1: Preserve existing type-test spoof regression.**

The existing test:

```text
overridden_is_method_does_not_gain_builtin_refinement_authority
```

must remain GREEN throughout this task.

- [ ] **Step 2: Add overloaded equality spoof RED test.**

Use parser-valid equality method override syntax from current language fixtures. The semantic requirement:

```phalcom
class Liar {
  ==(_ other) -> Bool { true }
}
```

or the repository's canonical operator declaration syntax must not let `if value == None/...` narrow `value` unless the resolved callable is the canonical trusted equality operation.

Assert the branch-local value retains its original type/authority.

- [ ] **Step 3: Change extractor return type.**

```rust
pub fn extract_trusted_predicate(...) -> Option<TrustedFlowPredicate>
```

Type-test path:

```text
canonical Object#is/is! -> AuthoritativeObservation
noncanonical is/is!     -> None
```

- [ ] **Step 4: Gate equality/nil/literal/ordered predicates by semantic identity.**

Do not let `extract_predicate`'s syntax recognition itself authorize formal application.

Keep `extract_predicate` private or rename it to make its role obvious, for example:

```rust
fn extract_predicate_shape(...) -> Option<FlowPredicate>
```

Only `extract_trusted_predicate` should be exported to formal branch execution.

- [ ] **Step 5: Keep truthy/falsy non-type-changing facts explicitly derived.**

Return:

```rust
TrustedFlowPredicate {
    predicate,
    authority: PredicateAuthority::DerivedFilter,
}
```

for direct truthiness only where the language control semantics itself guarantees the branch proposition.

- [ ] **Step 6: Run trust regressions.**

```sh
cargo test -p phalcom-semantic --test semantic overridden_is_method_does_not_gain_builtin_refinement_authority -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/flow/predicate.rs phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "fix(semantic): require canonical identity for branch predicates"
```

---

## Task 5: Integrate predicate contradiction with Part-3 branch reachability

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/control.rs`
- Modify: explanation code only if needed to remove hard-coded status.
- Test: `tests/semantic/capabilities/flow_branches.rs`
- Test: `tests/semantic/capabilities/control_regions.rs`.

**Interfaces:**
- Consumes: `FlowPredicateApplication::Contradiction`, `ExecutableRegionResult`, branch joins.
- Produces: impossible paths excluded before executing their regions.

- [ ] **Step 1: Make context contradiction terminate flow.**

```rust
match apply_predicate(...) {
    PredicateTransfer::Contradiction { .. } => {
        self.flow.mark_unreachable();
        FlowPredicateApplication::Contradiction
    }
    // ...
}
```

The pure transfer function must remain free of control mutation.

- [ ] **Step 2: Do not execute a region after predicate contradiction.**

In `checker/control.rs`, after applying the branch predicate:

```rust
if !ctx.flow.is_reachable() {
    return ExecutableRegionResult {
        value: None,
        flow: ctx.flow.clone(),
        causal_invalidity: CausalInvalidity::Clean,
    };
}
```

Do not call `analyze_executable_region` on that arm.

- [ ] **Step 3: Add dead contradictory-arm side-effect test.**

```rust
#[test]
fn contradictory_branch_does_not_publish_bindings_or_diagnostics() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = "hello"
    if (value.is(Int)) {
      let impossible = mystery()
    }
    let observed = value
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert!(f.bindings_named(run, "impossible").is_empty());
    assert!(f.diagnostics(phalcom_semantic::DiagnosticCode::UnresolvedName).is_empty());
    f.assert_binding_established(run, "observed", f.ty("String"));
}
```

Use the repository's actual unresolved-name diagnostic code if renamed after rebasing.

- [ ] **Step 4: Fix branch explanation status if still hard-coded.**

Any `BranchJoin` or `FlowRefinement` node must use the resulting evidence status. Do not stamp `Established` on an assumed residual.

- [ ] **Step 5: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/control.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "feat(semantic): prune proven contradictory branches"
```

---

## Task 6: Add literal boolean `ConditionTruth` pruning

**Files:**
- Modify: `phalcom-semantic/src/checker/control.rs`
- Test: `tests/semantic/capabilities/flow_branches.rs`
- Test: `tests/semantic/capabilities/control_regions.rs` if nested exits are involved.

**Interfaces:**
- Produces: `ConditionTruth`, `condition_truth`.

- [ ] **Step 1: Add RED direct truth tests.**

```rust
#[test]
fn if_true_uses_only_true_branch_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = if true { 1 } else { "dead" }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "value", f.ty("Int"));
}
```

```rust
#[test]
fn if_false_uses_only_false_branch_value() {
    // mirror above; expected String
}
```

- [ ] **Step 2: Add negation tests.**

```text
if not true  -> false arm only
if not false -> true arm only
```

Do not add compound boolean folding.

- [ ] **Step 3: Add dead-arm semantic side-effect test.**

```phalcom
let x = if true {
  1
} else {
  mystery()
}
```

Expected:

```text
x = Established<Int>
no unresolved-name diagnostic from mystery()
```

- [ ] **Step 4: Add callable-exit pruning test.**

```phalcom
if false {
  return 1
}
return "live"
```

Expected: exactly one recorded normal return, `String`.

This proves constant pruning happens before executable-region execution, not merely during final value joining.

- [ ] **Step 5: Implement `ConditionTruth`.**

Use only Boolean literals and unary `not` recursion as specified.

- [ ] **Step 6: Integrate into branch pair before path predicate application/body execution.**

```text
AlwaysTrue:
  then starts from entry
  else is FlowState::unreachable(), no body execution

AlwaysFalse:
  then unreachable/no execution
  else starts from entry

Unknown:
  existing predicate split
```

The condition expression itself must still be analyzed exactly once before pruning so type/operation diagnostics on the condition remain available.

- [ ] **Step 7: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic if_true_uses_only_true_branch_value -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
git add phalcom-semantic/src/checker/control.rs phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "feat(semantic): prune literal boolean branches"
```

---

## Task 7: Composition audit and final Part-4 closure gate

**Files:**
- Modify: `tests/semantic/capabilities/flow_branches.rs`
- Modify/create: foundation predicate-transfer tests.
- Modify: `tests/semantic/COVERAGE_LEDGER.md` if applicable.
- Modify: `db/fingerprint.rs` only if published product shapes changed.

**Interfaces:**
- Consumes: complete Part-4 predicate proof architecture.
- Produces: proof-law regression matrix and no-authority-laundering closure.

- [ ] **Step 1: Add composed callable-contract + narrowing test.**

```rust
#[test]
fn trusted_narrowing_can_certify_declared_return_without_laundering_assumptions() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  normalize(_ value: Object) -> Int {
    if (value.is(Int)) {
      return value
    }
    0
  }
}
"#,
    );
    let normalize = f.callable("Probe", "normalize", DispatchSide::Class);
    // Part 1: public return contract validated.
    // Part 4: branch-local `value` is Established<Int> only because canonical is(Int) observed it.
    f.assert_normal_return_types(normalize, &[f.ty("Int"), f.ty("Int")]);
    f.assert_no_error_diagnostics();
}
```

Adapt helper name to the final Part-1 fixture API; the substantive assertions must inspect both return facts and public signature authority.

- [ ] **Step 2: Add assumption-dependent broad-test composition.**

Use classes `Animal`, `Cat`, `Dog` and an assumed `Cat | Dog` source/parameter fact. Inside `value.is(Animal)`, assert retained `Cat | Dog` remains `Assumed`, not established.

This is the guard against “trusted predicate means every narrowed result is established.”

- [ ] **Step 3: Add negative assumed-union integration test.**

```text
incoming Assumed<Int|String>
false branch of canonical is(Int)
branch-local value == Assumed<String>
```

If the source syntax cannot directly annotate unions at that position, construct the assumed union in a foundation transfer test and use a compositional source test through a branch-produced assumed union.

- [ ] **Step 4: Audit all established constructions in predicate code.**

```sh
rg "TypeKnowledge::established" phalcom-semantic/src/checker/flow phalcom-semantic/src/checker/control.rs
```

Every remaining construction must be classifiable as one of:

```text
compiler-owned control value (Unit/Never)
authoritative trusted observation
established transformation from established premises
```

No negative filter may unconditionally construct established knowledge.

- [ ] **Step 5: Audit formal predicate producers.**

```sh
rg "extract_predicate|extract_trusted_predicate|TrustedFlowPredicate|FlowPredicate \{" phalcom-semantic/src/checker
```

Expected: structured branch execution consumes only the trusted formal path. Raw syntax extraction is private/non-authoritative.

- [ ] **Step 6: Fingerprint check.**

`PredicateAuthority`, `PredicateTransfer`, and `ConditionTruth` are transient checker mechanics and do not need fingerprints by themselves.

If branch reachability changes existing published `BodyExitFacts`, binding facts, or explanations, existing product hashing should naturally reflect the changed products. Update `db/fingerprint.rs` only if a new published field was added.

- [ ] **Step 7: Full semantic gate.**

```sh
cargo fmt --all -- --check
cargo test -p phalcom-semantic --test semantic -- --nocapture
cargo test -p phalcom-semantic
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

- [ ] **Step 8: Commit final tests/docs.**

```sh
git add phalcom-semantic/tests/semantic \
        phalcom-semantic/src/db/fingerprint.rs
git commit -m "test(semantic): close predicate proof invariants"
```

---

# 9. Mandatory proof matrix

The implementation is not complete until these exact semantic classes are tested.

| Prior knowledge | Predicate | Trust | Expected path/result |
|---|---|---|---|
| Unknown | `is(Int)` true | canonical | reachable `Established<Int>` |
| Dynamic | `is(Int)` true | canonical | reachable Dynamic unchanged |
| Assumed<Object> | `is(Int)` true | canonical | `Established<Int>` |
| Assumed<Int\|String> | `is(Int)` true | canonical | `Established<Int>` |
| Assumed<Cat\|Dog> | `is(Animal)` true | canonical | `Assumed<Cat\|Dog>` |
| Established<Cat\|Dog> | `is(Animal)` true | canonical | `Established<Cat\|Dog>` |
| Established<String> | `is(Int)` true | canonical | contradiction / unreachable |
| Assumed<String> | `is(Int)` true | canonical | reachable `Established<Int>` |
| Established<Int\|String> | `is(Int)` false | canonical | `Established<String>` |
| Assumed<Int\|String> | `is(Int)` false | canonical | `Assumed<String>` |
| Established<Int> | `is(Int)` false | canonical | contradiction / unreachable |
| Assumed<Int> | `is(Int)` false | canonical | reachable `Unknown(InferenceConflict)` |
| any | overridden `is` | untrusted | no type refinement |
| any | overloaded equality | untrusted | no formal equality refinement |
| established literal Bool true | `if` | intrinsic literal | false arm not executed |
| established literal Bool false | `if` | intrinsic literal | true arm not executed |

---

# 10. Required negative assertions

Tests must explicitly prove the checker does **not**:

```text
- convert an assumed residual union into Established after negative filtering
- prune a branch merely because a developer annotation conflicts with a trusted runtime observation
- treat unequal nominal TypeIds as automatically disjoint without relation proof
- refine on a user method merely named `is`, `is!`, `==`, or `!=`
- assign Never to an ordinary binding as the representation of branch contradiction
- execute a branch body after predicate contradiction
- execute the impossible arm of `if true` / `if false`
- use constant folding beyond literal booleans and unary not
- turn Dynamic into a static Established type in this part
```

---

# 11. Explicit non-goals

Part 4 does not include:

- arbitrary intersection or negation types;
- exhaustive pattern refinement for `if let`;
- general constant evaluation;
- arithmetic range analysis;
- symbolic execution;
- dependent/refinement types;
- whole-program theorem proving;
- loop fixed points;
- comparison-chain/membership operation correctness (Part 6);
- global canonical builtin-ID registry (Part 6);
- CFG-driven checking;
- unreachable-code diagnostics;
- dynamic-to-static narrowing semantics.

---

# 12. Definition of done

Part 4 is complete only when:

1. formal predicate application carries explicit authority;
2. transfer returns `Unchanged`, `Refined`, or `Contradiction` explicitly;
3. negative filtering preserves prior evidence status;
4. trusted positive type tests establish only facts justified by the observation itself;
5. established contradictions make branch flow unreachable;
6. assumed contradictions do not make branches impossible;
7. user-overloadable method spelling cannot authorize formal proof;
8. constant `true`/`false`/`not` branches are pruned before region execution;
9. branch joins naturally exclude contradictory/constant-impossible arms through Part-3 reachability;
10. authority, branch, control-region, callable-publication, and field suites remain GREEN.

The resulting proof pipeline should be:

```text
condition expression
    ↓
resolved semantic callable / intrinsic identity
    ↓
TrustedFlowPredicate + PredicateAuthority
    ↓
pure PredicateTransfer
    ├─ Unchanged
    ├─ Refined (authority preserved/justified)
    └─ Contradiction
            ↓
       FlowState unreachable
            ↓
Part-3 executable branch selection/join
```

This gives Part 5 a trustworthy path-sensitive state to carry around loop backedges without first having to repair proof authority inside the loop solver.
