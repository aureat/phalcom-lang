# Plan 4 — Canonical `phalcom-semantic/tests/` Organization

> **Fixture syntax invariant:** The reorganized test infrastructure must not normalize or rewrite Phalcom block syntax. Inline fixtures use canonical pipe blocks (`|x| { ... }`, `|x| value`, and their zero-parameter forms).

**Project:** Phalcom  
**Crate:** `phalcom-semantic`  
**Repository snapshot:** `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`  
**Decision:** converge the crate on **one Cargo integration-test binary** with a module tree underneath it.

## 1. Current problem

The current `tests/` tree mixes:

- many top-level `.rs` files, each becoming a separate Cargo integration binary;
- `integration.rs` + `integration/`;
- `semantic_capabilities.rs` + `semantic_capabilities/`;
- historical phase names such as `spec04_5_*`;
- feature names such as `kinds.rs`;
- infrastructure names such as `db.rs` and `workspace.rs`;
- authority/composition suites.

The coverage is valuable; the topology is fragmented.

The new organization must preserve all tests while making it obvious where new tests belong and centralizing deep semantic helpers.

## 2. One canonical Cargo target

Create:

```text
phalcom-semantic/tests/semantic.rs
```

with:

```rust
mod semantic;
```

Everything else moves below:

```text
phalcom-semantic/tests/semantic/
```

After migration, `tests/semantic.rs` should be the only top-level Rust file under `phalcom-semantic/tests/`.

The old `integration.rs`, `semantic_capabilities.rs`, and all other standalone top-level test sources disappear **only after their tests are migrated and parity-verified**.

## 3. Final tree

```text
phalcom-semantic/
└── tests/
    ├── semantic.rs
    └── semantic/
        ├── README.md
        ├── mod.rs
        │
        ├── support/
        │   ├── mod.rs
        │   ├── fixture.rs
        │   ├── workspace.rs
        │   ├── locator.rs
        │   ├── expect.rs
        │   ├── types.rs
        │   ├── diagnostics.rs
        │   ├── explanations.rs
        │   ├── dependencies.rs
        │   ├── incremental.rs
        │   └── render.rs
        │
        ├── foundations/
        │   ├── mod.rs
        │   ├── type_model.rs
        │   ├── knowledge.rs
        │   ├── declarations.rs
        │   ├── binding_contracts.rs
        │   ├── type_annotations.rs
        │   ├── kinds.rs
        │   ├── generics_core.rs
        │   ├── substitution.rs
        │   ├── inference.rs
        │   ├── bidirectional_calls.rs
        │   ├── expression_engine.rs
        │   ├── expression_analysis.rs
        │   ├── flow_graph.rs
        │   ├── causal.rs
        │   ├── diagnostics.rs
        │   ├── explanations.rs
        │   ├── advisory_domain.rs
        │   └── identity_diagnostics.rs
        │
        ├── capabilities/
        │   ├── mod.rs
        │   ├── checker_smoke.rs
        │   ├── authority.rs
        │   ├── bindings.rs
        │   ├── dispatch.rs
        │   ├── self_types.rs
        │   ├── method_families.rs
        │   ├── generics.rs
        │   ├── constraints.rs
        │   ├── type_lambdas.rs
        │   ├── variance.rs
        │   ├── structural.rs
        │   ├── patterns.rs
        │   ├── flow_branches.rs
        │   ├── flow_loops.rs
        │   ├── iteration.rs
        │   ├── callables.rs
        │   ├── callable_publication.rs
        │   ├── fields.rs
        │   ├── aliases.rs
        │   ├── modules.rs
        │   └── diagnostics.rs
        │
        ├── integration/
        │   ├── mod.rs
        │   ├── workspace.rs
        │   ├── source_index.rs
        │   ├── denotation.rs
        │   ├── presentation.rs
        │   ├── advisory.rs
        │   ├── metadata.rs
        │   ├── native_conformance.rs
        │   └── compiler_capabilities.rs
        │
        ├── incremental/
        │   ├── mod.rs
        │   ├── db.rs
        │   ├── fingerprints.rs
        │   ├── product_stability.rs
        │   ├── callable_dependencies.rs
        │   ├── checker_dependencies.rs
        │   ├── query_ownership.rs
        │   ├── advisory.rs
        │   └── type_store_revisions.rs
        │
        ├── advanced/
        │   ├── mod.rs
        │   ├── integration_matrix.rs
        │   ├── record_rows.rs
        │   ├── effects_control.rs
        │   ├── termination.rs
        │   └── contracts_prover.rs
        │
        └── golden/
            ├── mod.rs
            ├── generic_self_chain.rs
            ├── flow_pattern_publication.rs
            ├── iterator_chain.rs
            ├── family_callable.rs
            ├── type_lambda_constraints.rs
            ├── workspace_chain.rs
            ├── unknown_authority.rs
            ├── variance_recovery.rs
            ├── closure_flow.rs
            └── mixed_pipeline.rs
```

## 4. Taxonomy rule

Use this decision tree:

```text
pure canonical relation/data-structure law?
  -> foundations/

source program proving a language semantic capability?
  -> capabilities/

workspace/source-index/presentation/advisory/native integration?
  -> integration/

revision/invalidation/query reuse/fingerprint?
  -> incremental/

advanced effects/termination/contracts layer?
  -> advanced/

one of the ten deliberately broad composition programs?
  -> golden/
```

Examples:

```text
generic occurs check                         foundations/inference.rs
identity<T>(42) -> Int                      capabilities/generics.rs
Family pattern chooses labeled route        capabilities/method_families.rs
edit B invalidates caller A                 incremental/callable_dependencies.rs
source offset resolves to formal expression integration/source_index.rs
GOLDEN-03 iterator service chain            golden/iterator_chain.rs
```

## 5. Shared support architecture

### `support/fixture.rs`

Own:

- `Fixture::new`;
- parser validation;
- `analyze_single_module`;
- source/module storage;
- declaration/type lookup;
- callable lookup;
- binding lookup;
- expression lookup through `SourceLocator`;
- convenient snapshot/store access.

It should not own expectation logic.

### `support/workspace.rs`

Introduce `WorkspaceFixture` to hide current `workspace.rs` linker/project boilerplate.

Target style:

```rust
let f = WorkspaceFixture::new()
    .module("app.model", MODEL)
    .module("app.service", SERVICE)
    .module("app.controller", CONTROLLER)
    .entry("app.controller")
    .analyze();

let user = f.decl("app.model", "User");
let run = f.callable(
    "app.controller",
    "Controller",
    "run",
    DispatchSide::Class,
);
```

It must create real `ModuleId` / linked-source products. Never fake cross-module behavior by concatenating source.

### `support/locator.rs`

Own `SourceLocator` and canonical source-index resolution.

Compatibility:

```text
site("Factory.choose(42)")
site_n("value",2)
```

but route through `SemanticSnapshot` source-index APIs where possible.

### `support/expect.rs`

Own:

```text
KnowledgeExpectation
BindingExpectation
ExpressionExpectation
CallableExpectation
CallExpectation
```

No inference logic.

### `support/types.rs`

Own canonical `TypeExpectation` and structural matching.

### `support/diagnostics.rs`

Own exact diagnostic set/range/count/cascade assertions.

### `support/explanations.rs`

Own structured explanation matching.

### `support/dependencies.rs`

Own callable and `SemanticDependency` assertions.

### `support/incremental.rs`

Own reusable source-edit/revision scaffolding. Low-level `SemanticDb` mechanics stay in `incremental/db.rs`.

### `support/render.rs`

Own readable failure rendering:

- semantic type names;
- declaration/callable names;
- source snippets;
- status/origin;
- expected/actual diff.

## 6. Preserve both shallow and deep ergonomics

Simple:

```rust
f.assert_binding_type(run, "x", int_ty);
f.assert_subtype(cat, animal);
```

Deep:

```rust
f.assert_binding(
    run,
    "x",
    binding()
        .declared(animal)
        .current(
            known(cat)
                .established()
                .origin(EvidenceOrigin::ConstructorSemantics),
        )
        .validated()
        .causal_clean(),
);
```

The helper design is successful only if both remain pleasant.

## 7. Current-file migration map

The repository tree was inventoried at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`.

| Current file | Destination | Purpose |
|---|---|---|
| `advisory_analysis.rs` | `semantic/integration/advisory.rs` | Advisory analysis integration. |
| `advisory_analyzer.rs` | `semantic/integration/advisory.rs` | Merge analyzer cases where laws overlap. |
| `advisory_domain.rs` | `semantic/foundations/advisory_domain.rs` | Low-level advisory domain invariants. |
| `advisory_incrementality.rs` | `semantic/incremental/advisory.rs` | Advisory revision behavior. |
| `binding_contract_semantics.rs` | `semantic/foundations/binding_contracts.rs` | Pure reconciliation laws. |
| `callable_dependency_invalidation.rs` | `semantic/incremental/callable_dependencies.rs` | Callable invalidation. |
| `checker_dependency_tracking.rs` | `semantic/incremental/checker_dependencies.rs` | SemanticDependency recording. |
| `class_side_dispatch.rs` | `semantic/capabilities/dispatch.rs` | Merge class-side dispatch. |
| `constructor_self_type.rs` | `semantic/capabilities/self_types.rs` | Constructor/inherited Self. |
| `core_surface_conformance.rs` | `semantic/integration/native_conformance.rs` | Core/native surface agreement. |
| `db.rs` | `semantic/incremental/db.rs` | SemanticDb basics. |
| `declaration_types.rs` | `semantic/foundations/declarations.rs` | Declaration type model. |
| `denotation.rs` | `semantic/integration/denotation.rs` | Formal denotation. |
| `export.rs` | `semantic/capabilities/modules.rs` | Module export semantics. |
| `formal_query_ownership.rs` | `semantic/incremental/query_ownership.rs` | Formal query ownership. |
| `identity_diagnostic_foundation.rs` | `semantic/foundations/identity_diagnostics.rs` | Identity/diagnostic primitives. |
| `integration/checker.rs` | `semantic/capabilities/checker_smoke.rs` | Existing checker integration smoke. |
| `integration/compiler_capabilities.rs` | `semantic/integration/compiler_capabilities.rs` | Compiler/semantic integration. |
| `kinds.rs` | `semantic/foundations/kinds.rs` | Kinds. |
| `metadata_export.rs` | `semantic/integration/metadata.rs` | Metadata export. |
| `phase2_expression_engine.rs` | `semantic/foundations/expression_engine.rs` | Expression engine infrastructure. |
| `presentation.rs` | `semantic/integration/presentation.rs` | Presentation/query projection. |
| `product_stability_invalidation.rs` | `semantic/incremental/product_stability.rs` | Stable products. |
| `semantic_authority_composition.rs` | `semantic/capabilities/authority.rs` | Declared/current/evidence authority. |
| `semantic_capabilities/branches.rs` | `semantic/capabilities/flow_branches.rs` | Historical branch capabilities. |
| `semantic_capabilities/callables.rs` | `semantic/capabilities/callable_publication.rs` | Publication/recursive inference. |
| `semantic_capabilities/dispatch.rs` | `semantic/capabilities/dispatch.rs` | Exact dispatch. |
| `semantic_capabilities/generics.rs` | `semantic/capabilities/generics.rs` | Generic integration. |
| `semantic_capabilities/iteration_advisory.rs` | `semantic/capabilities/iteration.rs` | Iteration/unknown composition. |
| `semantic_capabilities/loops_blocks.rs` | `semantic/capabilities/flow_loops.rs` | Loop/capture flow. |
| `semantic_capabilities/structural.rs` | `semantic/capabilities/structural.rs` | Tuple/record/list/pattern. |
| `semantic_capabilities/support.rs` | `semantic/support/*` | Split shared fixture/assertion infrastructure. |
| `semantic_db_incremental.rs` | `semantic/incremental/db.rs` | Merge DB revision/reuse. |
| `semantic_fingerprints.rs` | `semantic/incremental/fingerprints.rs` | Fingerprints. |
| `semantic_knowledge_invariants.rs` | `semantic/foundations/knowledge.rs` | TypeKnowledge invariants. |
| `source_semantic_index.rs` | `semantic/integration/source_index.rs` | Source semantic index. |
| `spec01_5_invariants.rs` | `semantic/foundations/generics_core.rs` | Generic/lambda/variance algebra. |
| `spec01_invariants.rs` | `semantic/foundations/type_model.rs` | Foundational type model. |
| `spec04_5_bidirectional_and_calls.rs` | `semantic/foundations/bidirectional_calls.rs` | Expected-type/call primitives. |
| `spec04_5_causal_suppression.rs` | `semantic/foundations/causal.rs` | Suppression. |
| `spec04_5_diagnostics.rs` | `semantic/foundations/diagnostics.rs` | Diagnostic primitives. |
| `spec04_5_explanation_arena.rs` | `semantic/foundations/explanations.rs` | Explanation arena. |
| `spec04_5_explanation_graph.rs` | `semantic/foundations/explanations.rs` | Explanation graph. |
| `spec04_5_expression_analysis.rs` | `semantic/foundations/expression_analysis.rs` | ExpressionAnalysis invariants. |
| `spec04_5_flow_graph.rs` | `semantic/foundations/flow_graph.rs` | CFG/flow primitives. |
| `spec04_5_inference_session.rs` | `semantic/foundations/inference.rs` | Inference primitive laws. |
| `spec05_integration_matrix.rs` | `semantic/advanced/integration_matrix.rs` | Advanced integration matrix. |
| `spec05_phase1_record_rows.rs` | `semantic/advanced/record_rows.rs` | Record rows. |
| `spec05_phase2_effects_control.rs` | `semantic/advanced/effects_control.rs` | Effects/control. |
| `spec05_phase3_termination_effects.rs` | `semantic/advanced/termination.rs` | Termination/effects. |
| `spec05_phase4_contracts_prover.rs` | `semantic/advanced/contracts_prover.rs` | Contracts/prover. |
| `substitution.rs` | `semantic/foundations/substitution.rs` | Substitution. |
| `type_annotations.rs` | `semantic/foundations/type_annotations.rs` | Type-form lowering. |
| `type_store_revisions.rs` | `semantic/incremental/type_store_revisions.rs` | Type identity across revisions. |
| `workspace.rs` | `semantic/integration/workspace.rs` | Workspace/multi-module analysis. |

The old wrapper files `integration.rs` and `semantic_capabilities.rs` themselves disappear after their modules move.

## 8. Module declarations

`tests/semantic/mod.rs`:

```rust
pub(crate) mod advanced;
pub(crate) mod capabilities;
pub(crate) mod foundations;
pub(crate) mod golden;
pub(crate) mod incremental;
pub(crate) mod integration;
pub(crate) mod support;
```

Prefer ordinary module declarations over many `#[path = "..."]` attributes after migration.

## 9. Migration sequence

### Phase 1 — skeleton

Create:

- `tests/semantic.rs`;
- `tests/semantic/mod.rs`;
- subdirectory `mod.rs` files.

Verify:

```bash
cargo test -p phalcom-semantic --test semantic --no-run
```

### Phase 2 — shared support

Move/split current `semantic_capabilities/support.rs` into `semantic/support/`. Keep temporary compatibility re-exports if needed.

### Phase 3 — capability target

Move `semantic_capabilities/*` to `semantic/capabilities/`. Verify category test count and pass/fail parity, then remove `tests/semantic_capabilities.rs`.

### Phase 4 — current integration target

Move `integration/checker.rs` and `integration/compiler_capabilities.rs`, then remove `tests/integration.rs`.

### Phase 5 — foundational standalone targets

Move Spec-01/01.5, Spec-04.5, kinds, substitution, annotations, declaration/knowledge/diagnostic tests.

### Phase 6 — incremental

Move DB, fingerprints, dependency/invalidation and type-store revision tests.

### Phase 7 — integration products

Move workspace, source-index, presentation, denotation, advisory, native, metadata. Build `WorkspaceFixture` rather than copying workspace boilerplate.

### Phase 8 — advanced

Move Spec-05 phase tests.

### Phase 9 — Plan 2 growth

Only now add the large new capability surface.

### Phase 10 — golden

Add Plan 3 after all support layers exist.

### Phase 11 — delete legacy top-level targets

Check that `tests/semantic.rs` is the only top-level `*.rs`.

## 10. Pure-move invariant

Do not mix semantic repairs into migration commits.

For each batch:

1. move;
2. adjust imports/module paths;
3. compile;
4. run;
5. verify same count and same expected pass/fail;
6. commit;
7. repair/deepen in later commits.

Suggested commits:

```text
test(semantic): add canonical integration target skeleton
test(semantic): centralize semantic fixture helpers
test(semantic): migrate capability modules
test(semantic): migrate foundation invariants
test(semantic): migrate incremental tests
test(semantic): migrate workspace and integration products
test(semantic): migrate advanced typing tests
test(semantic): remove legacy standalone targets
```

## 11. Running tests

Full:

```bash
cargo test -p phalcom-semantic --test semantic
```

Category:

```bash
cargo test -p phalcom-semantic --test semantic capabilities::generics
cargo test -p phalcom-semantic --test semantic incremental::db
cargo test -p phalcom-semantic --test semantic golden
```

Single law:

```bash
cargo test -p phalcom-semantic --test semantic \
  capabilities::iteration::custom_iterable_element_type_comes_from_protocol_not_first_generic_argument
```

Quality:

```bash
cargo fmt --check
cargo clippy -p phalcom-semantic --tests -- -D warnings
cargo test -p phalcom-semantic
```

## 12. One-binary trade-off

Advantages:

- one link target instead of historical target proliferation;
- one helper system;
- coherent namespace;
- category filtering;
- easier 150+ test growth;
- no obsolete phase naming at top level.

Costs:

- a support change recompiles a larger binary;
- a compile error in one module blocks that binary;
- less Cargo target-level parallelism.

The costs are accepted because the user explicitly wants one binary and current fragmentation has a higher maintenance cost. Revisit only with measured compile-time evidence.

## 13. Naming convention

Prefer semantic law names:

```text
broad_contract_preserves_narrow_established_current
family_exact_call_preserves_bound_receiver_specialization
reachable_unknown_is_absorbing_at_branch_join
generic_method_parameter_shadows_class_parameter_by_owner
```

Do not add new names such as `spec07_gate3_case2` or `issue_184`.

Historical tests may keep function names during pure migration; new files use semantic naming.

## 14. Test-tree README

Add `tests/semantic/README.md` documenting:

- taxonomy;
- one-binary rule;
- Level A/B/C assertion depth;
- `Fixture`;
- `WorkspaceFixture`;
- source locators;
- knowledge/binding/call expectations;
- READY / RED-CAPABILITY / STAGED / GATED;
- commands;
- explicit warning that VM bytecode/performance is a separate test domain.

The four implementation plans remain the detailed architecture record.

## 15. Acceptance criteria

Plan 4 is complete when:

- `tests/semantic.rs` is the only top-level Rust integration target;
- every old test has a destination and is preserved;
- pure-move parity is verified before semantic edits;
- shared helpers are centralized;
- `WorkspaceFixture` exists for new multi-module tests;
- shallow and deep tests are both concise;
- source programs stay visible;
- no helper implements inference;
- category/single-test filtering is straightforward;
- the full `semantic` target builds/runs;
- fmt/clippy/full crate gates pass;
- runtime/codegen performance tests remain separate.
