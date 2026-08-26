# Plan 1 — Deep Semantic Assertions for the Existing Capability Suite

**Project:** Phalcom  
**Crate:** `phalcom-semantic`  
**Repository:** `aureat/phalcom-lang`  
**Repository snapshot:** `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`  
**Primary current files:** `phalcom-semantic/tests/semantic_capabilities.rs`, `phalcom-semantic/tests/semantic_capabilities/*.rs`

## 1. Objective

Keep the existing semantic-capability programs, but turn them from mostly end-result checks into semantic derivation checks.

The historical baseline was the 40-test run:

- 12/40 passed;
- 12/32 after excluding eight stale parser fixtures;
- dispatch 5/5;
- generics 0/4;
- loops 0/4;
- branches 1/12;
- structural 3/8.

Current `main` has already grown past that exact count: `semantic_capabilities/generics.rs` contains four additional epistemic generic tests, so the target currently contains 44 tests. Preserve the original 40 as a named baseline ledger, but deepen **every capability test present when the migration starts**.

The new standard is not merely:

> did the checker produce the expected `TypeId`?

It is:

> did the checker produce the expected semantic fact, at the correct epistemic strength, from the correct semantic source, through the correct call/relation/flow path, while preserving declarations, recovery state, explanations, diagnostics and dependencies?

## 2. Repository-grounded observation products

The plan must inspect compiler-owned products; helpers must never reimplement inference.

### `ExpressionAnalysis`

`phalcom-semantic/src/checker/analysis.rs` exposes expression identity/range, `TypeKnowledge`, resolved callable, denotation, `AnalysisStatus`, `CausalInvalidity`, explanation and call-resolution identity.

Use it for every important intermediate expression.

### `BindingState`

The same module exposes `declared`, `contract`, `current`, denotation, `BindingConsistency`, causal invalidity, mutability, version and explanation.

This is the core object for the Phalcom authority law:

```text
declaration = constraint
current     = checker-owned semantic knowledge
```

### `CallableAnalysis`

The current product contains expression and binding indexes, flow graph, entry flow, `BodyExitFacts`, diagnostics, explanation arena, direct callable dependencies, semantic dependencies, dependency fingerprint and status.

### Evidence

`phalcom-semantic/src/types/evidence.rs` exposes:

```text
TypeKnowledge
  Known(TypeEvidence)
  Unknown(UnknownReason)
  Dynamic(DynamicReason)

EvidenceStatus
  Established
  Assumed

EvidenceOrigin
  Syntax
  DeclarationSemantics
  ConstructorSemantics
  CallableSignature
  NativeSignature
  DeveloperAnnotation
  GenericInference
  Flow
  ContextualDerivation
  PatternDecomposition
```

Tests must use exact `UnknownReason` / `DynamicReason` where the reason is part of the law.

### Canonical type structure

`TypeData` in `types/store.rs` exposes:

```text
Never
Unit
ClassObject
Nominal
Applied
Union
Tuple
Record
Callable
Parameter
Lambda
SelfType
```

Do not parse display strings in the test layer.

### Generic signatures and `Self`

`types/parameter.rs` exposes type-parameter owner/index/name/kind/variance, `GenericConstraint`, `GenericSignature`, and owner-relative `SelfTypeTerm`.

`signature.rs` exposes canonical callable/field signatures.

### Explanations

`explain/node.rs` exposes structural rules/evidence/parents, including literal synthesis, binding contracts, call return, generic instantiation, flow refinement, branch join, iteration element resolution, assignment propagation and return checking.

Tests may assert explanation structure; never assert explanation prose.

### Source index

`SemanticSnapshot` already exposes source-site/formal query products. Deep tests should increasingly use the canonical source index rather than invent a second occurrence model.

---

## 3. Assertion doctrine

Every capability test gets a one- or two-line semantic law.

Example:

```rust
// LAW: a broad source contract validates a narrower established initializer
// without replacing that narrower current fact.
```

Each important test may then assert four dimensions:

1. **positive facts** — what must be true;
2. **negative facts** — what must not be inferred;
3. **causal facts** — where the result must come from;
4. **recovery facts** — what remains valid after nearby refutation/unknown state.

Do not normally assert raw numeric IDs, vector order, debug formatting, solver iteration counts or user-facing explanation strings.

Successful fixtures must assert diagnostic cleanliness. Intentional failures must assert the owning diagnostic and absence of unrelated cascades.

---

## 4. Assertion-depth model

### Level A — baseline

Use for narrow type/shape laws.

Assert:

- final expression/binding type or structure;
- basic knowledge state;
- diagnostic cleanliness.

Typical: 4–7 assertions.

### Level B — deep

Use when authority, provenance, relation or dispatch matters.

Assert as relevant:

- type;
- `EvidenceStatus`;
- `EvidenceOrigin`;
- declared/current split;
- consistency;
- resolved callable/selector/side;
- relation;
- diagnostic;
- one negative invariant.

Typical: 8–15 assertions.

### Level C — trace

Use for high-value composition laws.

Assert as relevant:

- leaf facts;
- receiver;
- exact callable;
- formal signature;
- specialized result;
- generic parameter identity;
- branch/flow result;
- binding contract/current state;
- consistency/causal invalidity;
- explanation rule/evidence/parents;
- dependencies;
- body exits;
- diagnostic ownership;
- negative laws.

Typical: 15–30 assertions.

Target roughly ten Level-C tests in the historical 40. Do not make every test a trace test.

---

## 5. Expand `support.rs` into a semantic assertion DSL

Keep the current convenience helpers; add richer optional expectations.

### 5.1 `KnowledgeExpectation`

Target style:

```rust
f.assert_knowledge(
    expr.knowledge(),
    known(int_ty)
        .established()
        .origin(EvidenceOrigin::Syntax),
);
```

```rust
f.assert_knowledge(
    binding.current(),
    known(number)
        .assumed()
        .origin(EvidenceOrigin::DeveloperAnnotation),
);
```

```rust
f.assert_knowledge(
    expr.knowledge(),
    unknown(UnknownReason::UnderconstrainedTypeVariable),
);
```

Required semantic dimensions:

```rust
pub struct KnowledgeExpectation {
    pub ty: Option<TypeExpectation>,
    pub state: KnowledgeStateExpectation,
    pub status: Option<EvidenceStatus>,
    pub origin: Option<EvidenceOrigin>,
}
```

### 5.2 `TypeExpectation`

Support canonical structural matching:

```text
nominal("Cat")
class_object("Cat")
applied("List", [Int])
union([Cat, Dog])
tuple([Int, String])
labeled_tuple(...)
record({name: String, age: Int})
callable(...)
parameter(owner,index)
lambda(...)
self_type(owner,side,role)
unit()
never()
kind(...)
```

Required inspection:

- applied origin and arguments;
- union set;
- tuple labels/elements;
- record fields/tail;
- callable parameter labels/rest/result;
- kind;
- type-parameter owner/index/name/kind/variance;
- type-lambda binder kinds/body/result kind;
- `Self` owner/side/role.

### 5.3 `BindingExpectation`

Target style:

```rust
f.assert_binding(
    run,
    "x",
    binding()
        .declared(number)
        .source_contract(number)
        .current(known(int_ty).established().origin(EvidenceOrigin::Syntax))
        .validated()
        .causal_clean(),
);
```

Intentional refutation:

```rust
f.assert_binding(
    run,
    "x",
    binding()
        .declared(string_ty)
        .current(known(int_ty).established().origin(EvidenceOrigin::Syntax))
        .refuted(int_ty, string_ty),
);
```

Support:

- declaration;
- contract type/origin;
- current knowledge;
- consistency;
- causal invalidity;
- mutability;
- relative binding identity;
- version only in incrementality tests;
- explanation only when explicitly requested.

Convenience laws:

```text
assert_precision_preserved_under_contract
assert_refutation_preserves_actual
assert_assumption_does_not_become_established
assert_distinct_shadow_bindings
```

### 5.4 `ExpressionExpectation`

Support:

- knowledge;
- analysis status;
- denotation;
- causal invalidity;
- resolved callable;
- call-resolution existence;
- explanation.

### 5.5 Callable/signature expectations

Support:

- owner;
- dispatch side;
- selector;
- generic signature;
- parameter names/labels/rest;
- parameter `TypeTerm`;
- return `TypeTerm`;
- implementation kind;
- effects/raises/flow/lifecycle only when that law is under test.

Generic signature assertions must inspect owner/index/kind/variance/constraints.

### 5.6 Call expectations

A call-oriented language needs a first-class helper.

Example:

```rust
f.assert_call(
    run,
    site("factory-call"),
    call()
        .target(factory_choose)
        .side(DispatchSide::Class)
        .selector("choose(_)")
        .result(known(int_ty)
            .established()
            .origin(EvidenceOrigin::GenericInference)),
);
```

For generic calls, assert the externally meaningful chain:

```text
formal return T
argument/receiver evidence implies T := Int
specialized expression result Int
```

Do not expose solver-private substitution maps solely for tests.

### 5.7 Diagnostics

Required operations:

```text
assert_no_error_diagnostics
assert_diagnostic(code,site,count)
assert_no_diagnostic(code)
assert_only_error_codes([...])
```

### 5.8 Explanations

Structural expectations only:

```text
rule
status
origin
evidence references
parent relationships
```

Invariants:

- explanation status/origin agrees with expression/binding knowledge;
- branch join has relevant arm parents;
- generic instantiation has relevant call/type evidence;
- suppressed/blocked expressions do not carry fabricated proof explanations.

### 5.9 Dependencies

Support:

- direct callable dependency;
- `DeclarationShell`;
- `CallableSignature`;
- `DeclarationSurface`;
- `HierarchyEdge`;
- `LinkedInterface`;
- positive and negative dependency assertions.

A locally correct type with incomplete dependencies is not incrementally sound.

### 5.10 Body exits

Add helpers for:

- normal return values;
- explicit returns;
- throws;
- unreachable exits;
- normal-return summary/publication.

---

## 6. Source-site location

Keep exact-text occurrence lookup as compatibility, but introduce:

```rust
pub enum SourceLocator<'a> {
    Text { text: &'a str, occurrence: usize },
    Offset(usize),
    Range(Range<usize>),
    Site(SourceSiteId),
}
```

Resolve through `SemanticSnapshot`/`SourceSemanticIndex` where possible. Do not create a test-only mini-parser.

Deep tests should be able to say “inspect this call site” without relying on raw `ExpressionId`.

---

## 7. Deepening matrix

### Branches — historical 12

| Test | Depth | Required additional assertions |
|---|---|---|
| `same_type_branch_results_establish_single_result_type` | B | both arm facts, join status/origin, binding current, no diagnostics |
| `heterogeneous_branch_results_join_into_union` | C | arm facts, union, Flow provenance, explanation parents, no widening |
| `branch_union_validates_common_supertype_without_widening_current_fact` | C | constructors, `Cat|Dog`, declared `Animal`, subtype, Validated, hierarchy dependencies |
| `returning_branch_does_not_contribute_value_to_continuing_join` | C | return exit, continuing value, `BodyExitFacts`, post-join fact |
| `throwing_branch_is_excluded_from_reachable_value_join` | B | throw exit excluded, continuing result retained |
| `same_type_writes_in_both_branches_preserve_flow_type` | B | write facts, same BindingId, post-join Flow fact |
| `divergent_branch_assignments_join_current_binding_types` | C | preheader, writes, union, post-read, flow explanation |
| `branch_join_preserves_narrow_flow_under_broad_declared_contract` | C | declared/current split, `Int|Float`, subtype to Number, Validated |
| `refuted_branch_assignment_does_not_fabricate_declared_flow_fact` | C | actual String write retained, owning diagnostic, no fabricated Number |
| `branch_local_shadow_does_not_mutate_outer_binding_flow` | B | distinct BindingIds, inner/outer facts, outside read from outer |
| `nested_branch_results_compose_transitively` | C | inner and outer joins, flattened union, nested explanation |
| `known_branch_does_not_hide_reachable_unknown_branch_in_formal_analysis` | C | exact UnknownReason, reachable Unknown remains incomplete, no laundering |

Repair retired `var` syntax separately before diagnosing semantics.

### Loops/blocks — historical 4

| Test | Depth | Required additional assertions |
|---|---|---|
| `loop_same_type_assignment_preserves_current_type` | B | preheader, body write, loop join, post-read |
| `loop_join_includes_preheader_and_body_types` | C | zero-iteration path, body/backedge, union, declared/current, subtype |
| `break_and_continue_preserve_loop_exit_and_backedge_facts` | C | preheader, continue/backedge, break/exit, final union |
| `captured_block_write_is_not_applied_until_execution_is_proven` | C | capture identity, closure-body fact, outer pre-invocation fact, no speculative mutation |

**Do not** inspect VM bytecode, jump count, closure allocation or loop performance in these tests.

### Structural — historical 8

| Test | Depth | Required additional assertions |
|---|---|---|
| `nested_tuple_composes_exact_constituent_facts` | B | nested canonical tuple, leaf status/origin |
| `tuple_supertype_annotation_preserves_specific_product_fact` | C | declared/current products, component subtype, Validated |
| `tuple_component_refutation_preserves_actual_product_fact` | C | actual product retained, Refuted, one diagnostic |
| `branch_product_results_preserve_component_precision` | B | arm tuple facts, normalized product join |
| `heterogeneous_collection_infers_union_element_type` | B | List origin, union element, literal evidence |
| `record_literal_preserves_structural_field_types` | B | exact record field names/types, not only `TypeData::Record` |
| `tuple_destructuring_establishes_independent_component_bindings` | C | source tuple, distinct bindings, PatternDecomposition evidence |
| `tuple_destructuring_with_broad_contract_keeps_specific_components` | C | decompose current `(Int,Some)`, not declared `(Number,Option)` |

### Dispatch — historical 5

| Test | Depth | Required additional assertions |
|---|---|---|
| `chained_dispatch_preserves_constructor_specialization_without_binding_storage` | C | constructor target, Self specialization, next target, results, dependencies |
| `multiple_hop_call_chain_preserves_each_intermediate_result` | C | every intermediate receiver/result + exact callable chain |
| `wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown` | C | correct side for good calls, exact failure class for bad calls, never Dynamic fabrication |
| `selector_label_mismatch_is_distinguished_from_argument_type_mismatch` | B | selector identity, wrong shape, no ArgumentMismatch misclassification |
| `argument_refutation_preserves_independently_known_call_return_type` | C | argument fact, target, mismatch, fixed return fact, causal invalidity |

### Generics — historical 4 + current-main additions

| Test | Depth | Required additional assertions |
|---|---|---|
| `generic_identity_solves_parameter_from_argument_and_specializes_return` | C | callable-owned T, formal parameter/return, Int/String specializations |
| `generic_pair_solves_two_independent_variables` | C | A/B identities, independent constraints, tuple result |
| `expected_result_context_constrains_generic_without_merely_overwriting_call_fact` | C | argument-derived Int, expected Number as constraint, call stays Int |
| `conflicting_generic_constraints_are_refuted_instead_of_using_expected_annotation_as_fact` | C | String evidence retained, InferenceConflict/refutation, no fabricated Int |
| `assumed_generic_argument_yields_assumed_generic_return` | B | assumed parameter -> assumed generic result |
| `mixed_generic_return_uses_weakest_value_support` | B | mixed support -> assumed composite |
| `independent_fixed_generic_return_stays_established` | C | assumed generic input does not weaken independent fixed return |
| `expected_context_cannot_fabricate_missing_generic_return` | C | underconstrained result remains Unknown, expected type cannot invent proof |

### Callables — historical 4

| Test | Depth | Required additional assertions |
|---|---|---|
| `branch_derived_tail_type_is_published_to_unannotated_callable_signature` | C | branch facts, exits, publication, downstream call, dependency |
| `explicit_broad_return_contract_preserves_narrow_branch_evidence` | C | public Animal signature vs Cat/Dog body facts |
| `one_bad_return_branch_is_refuted_without_rewriting_branch_fact` | C | String remains String, one ReturnMismatch, recovery |
| `recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type` | C | exact recursive/fixpoint incompleteness, no Unit/nominal fabrication |

### Iteration/advisory — historical 3

`custom_iterable_element_type_comes_from_protocol_not_first_generic_argument` becomes a flagship trace test.

Required chain:

```text
Weird<A,B>
  A owner/index/kind
  B owner/index/kind

Probe.run parameter
  Weird<String,Int>

iteration resolution
  receiver Weird<String,Int>
  target Weird#iteratorValue(_)
  formal result B
  specialize B := Int

loop binding
  value current Int
  NOT String
  origin/status according to canonical iteration/pattern policy

let observed
  initializer Int
  binding Int

dependencies
  Weird declaration product(s)
  iteratorValue callable signature
```

Also distinguish:

```text
iteratorValue body `mystery()` may be Unknown
but its explicit caller-facing return contract is B.
```

That is not unknown laundering.

The other two iteration/advisory tests should deeply assert nested collection/branch precision and declaration-assumption rules respectively.

---

## 8. Rich failure output

Deep helper failures should report domain facts:

```text
binding `value` semantic mismatch

expected:
  declared:     <none>
  current:      Int
  status:       Established
  origin:       PatternDecomposition

actual:
  declared:     <none>
  current:      String
  status:       Established
  origin:       ContextualDerivation
```

For calls, show receiver, selector, target and result.  
For diagnostics, print all unexpected error diagnostics with source context.

---

## 9. Implementation workstreams

### A. Fixture-only syntax repair

Modify current capability source files that still contain removed syntax:

- `semantic_capabilities/branches.rs`;
- `semantic_capabilities/loops_blocks.rs`;
- any additional stale file found by parser validation.

Use current mutable `let`; use current explicit closure syntax for standalone closure values.

Keep semantic assertions unchanged in this commit.

### B. Expectation vocabulary

Expand/split current `semantic_capabilities/support.rs` with:

- knowledge;
- type;
- binding;
- expression;
- diagnostic;
- locator;
- renderer.

Do not rewrite every test yet.

### C. Call/signature/explanation/dependency support

Add stable semantic inspection wrappers. Any production API addition must expose a real semantic product useful beyond tests, not solver-private temporary state.

### D. Deepen category by category

Recommended order:

1. dispatch — already mostly green, validates helpers;
2. structural;
3. generics;
4. branches;
5. iteration/advisory;
6. callables;
7. loops/blocks.

### E. Move Level-C sites to canonical source-index lookup

After locator support is stable, remove brittle exact-text/nth lookup first from trace tests.

---

## 10. TDD rule for checker repairs

When a deep test fails:

1. preserve the red source integration test;
2. find the earliest incorrect semantic product;
3. add a lower-level invariant test if the defect is local;
4. repair production code;
5. require both levels to pass.

Never weaken the source test to current behavior.

---

## 11. Commands

Before Plan 4 consolidation:

```bash
cargo fmt --check
cargo test -p phalcom-semantic --test semantic_capabilities --no-run
cargo test -p phalcom-semantic --test semantic_capabilities
cargo test -p phalcom-semantic
cargo clippy -p phalcom-semantic --tests -- -D warnings
```

After Plan 4:

```bash
cargo test -p phalcom-semantic --test semantic --no-run
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic --test semantic capabilities::generics
cargo test -p phalcom-semantic --test semantic capabilities::branches::heterogeneous_branch_results_join_into_union
cargo clippy -p phalcom-semantic --tests -- -D warnings
```

---

## 12. Acceptance criteria

Plan 1 is complete when:

- historical 40 laws are retained;
- all additional capability tests present at migration time are retained;
- stale syntax is fixed separately from checker repairs;
- every test states its primary semantic law;
- success tests check diagnostic cleanliness;
- failures check owning diagnostics and cascade discipline;
- helpers distinguish Known/Unknown/Dynamic, Established/Assumed and EvidenceOrigin;
- structural types are inspected structurally;
- selected calls assert exact callable identity and specialized result;
- selected tests assert structured explanations;
- selected tests assert semantic dependencies;
- callable publication tests inspect body exits;
- at least ~10 capability tests are Level-C traces;
- failures render semantic names, not only raw IDs;
- no helper implements inference;
- no test asserts runtime loop/codegen performance.

The deliverable is a trustworthy oracle for Plans 2 and 3.
