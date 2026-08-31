# Phalcom ADT / GADT / Match Comprehensive Testing and Test-Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Use test-driven development for every behavioral expansion. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize Phalcom's semantic and core test infrastructure around stable language responsibilities, eliminate duplicate integration-test compilation, and establish comprehensive source-to-runtime coverage for enums, ADTs, GADTs, variants, associated families, match, exhaustiveness, proof refinement, semantic lowering, execution, GC, and incremental invalidation.

**Architecture:** `phalcom-semantic` remains the sole static semantic authority and keeps one integration-test binary rooted at `tests/semantic.rs`. `phalcom-core` is consolidated into two integration binaries, `language` and `core`, with source-facing language behavior separated from runtime/platform contracts. ADT tests use source-driven semantic oracles wherever possible, assert canonical semantic identities rather than only diagnostic absence, and pair static conformance with runtime/lowering tests where the feature crosses the executable boundary.

**Tech Stack:** Rust 2024, Cargo integration tests, `phalcom-ast`, `phalcom-semantic`, `phalcom-core`, `SemanticSnapshot`, `MatchResolution`, `PatternSpace`, `VariantId`, `VariantFamilyId`, `VariantFieldId`, semantic lowering, bytecode/disassembly assertions, VM execution, temporary project fixtures.

**Normative design inputs:**
- `docs/impl/adt-gadt-associated-lookup/part-5.1/05.1-match-surface-pattern-semantics-exhaustiveness-gadt-proofs-technical-spec.md`
- `docs/impl/adt-gadt-associated-lookup/part-5.1/05.1-match-surface-pattern-semantics-exhaustiveness-gadt-proofs-implementation-plan.md`
- `docs/impl/adt-gadt-associated-lookup/part-5.2/05.2-executable-pattern-projection-match-lowering-shared-pattern-runtime-integration-technical-spec.md`
- `docs/impl/adt-gadt-associated-lookup/part-5.2/05.2-executable-pattern-projection-match-lowering-shared-pattern-runtime-integration-implementation-plan.md`
- Companion scenario specification: `phalcom-adt-comprehensive-testing-scenario-catalog.md`

**Verified repository baseline while writing this plan:**
- repository: `aureat/phalcom-lang`
- branch: `feat/adts`
- HEAD: `72d6eca53944c588c653ad76e8b44056df9bef4d`
- subject: `test: expose residual match semantic gaps`
- parent: `9bcf6331f7eb4fa72a4cc1cc8d1817a7957805ba`
- Part 05.1 implementation is therefore committed on the branch; the current HEAD adds focused regression tests over that implementation.

## Global Constraints

- Preserve `phalcom-semantic` as the sole semantic authority.
- Do not add compiler/runtime source-name re-resolution to make tests easier.
- Test organization names describe language/runtime responsibility, never implementation history (`part_*`, `stage*`, `spec03`, `F.*`, etc.).
- Scenario count is the planning unit; Rust `#[test]` count is not a success metric.
- Move/reorganization commits and assertion-deepening commits must be separate.
- A test that claims a semantic identity law must assert canonical IDs/types/products, not merely “no diagnostics”.
- A runtime test must not duplicate semantic proof logic; it checks executable projection and behavior.
- Existing known baseline failures must be classified before test expansion. New failures must not be hidden with `#[ignore]` unless the scenario is explicitly `GATED`/`PENDING`.
- `PASS`, `NEGATIVE`, and `PENDING` corpus meanings remain distinct.
- Keep shared support small; ADT-local helpers stay under the ADT suite unless generically useful.
- Preserve focused Cargo filtering by module path.
- Add coverage-ledger entries only after executable tests exist.
- Do not claim completion without running the final verification matrix with fresh output.

---

# 1. Repository Findings That Drive the Plan

## 1.1 Semantic suite

The live semantic suite already uses one test binary:

```text
phalcom-semantic/tests/semantic.rs
```

with modules under:

```text
phalcom-semantic/tests/semantic/
```

The repository README establishes semantic-responsibility modules and focused execution such as:

```text
cargo test -p phalcom-semantic --test semantic capabilities::generics
cargo test -p phalcom-semantic --test semantic incremental::db
```

Current ADT modules include:

```text
declarations.rs
exact_cases.rs
exhaustiveness.rs
gadt_cases.rs
gadt_proofs.rs
match_basic.rs
match_bindings.rs
match_diagnostics.rs
match_matrix.rs
match_patterns.rs
match_regressions.rs
pattern_space.rs
requirements.rs
variants.rs
```

The current organization is usable, but matching concerns are flat and several broad tests assert only diagnostic absence. This plan reorganizes and deepens them.

## 1.2 Core suite

`phalcom-core/Cargo.toml` currently explicitly registers many integration binaries. ADT/associated coverage is spread across standalone binaries such as:

```text
adt_case_primitives
adt_runtime
adt_behavior
adt_gc
associated_family_gc
associated_lowering
semantic_lowering
adt_end_to_end
associated_family_runtime
associated_reification
```

and `tests/integration.rs` also includes `adt_runtime` as a module. This creates duplicated compilation and weak conceptual ownership.

The target is exactly two core integration binaries:

```toml
[[test]]
name = "language"
path = "tests/language.rs"

[[test]]
name = "core"
path = "tests/core.rs"
```

---

# 2. Target Test Trees

## 2.1 `phalcom-semantic`

```text
phalcom-semantic/tests/
  semantic.rs
  semantic/
    mod.rs
    README.md
    COVERAGE_LEDGER.md
    support/

    adts/
      mod.rs
      support.rs

      declarations.rs
      variants.rs
      constructors.rs
      exact_cases.rs
      generics.rs
      behavior.rs
      requirements.rs

      associated/
        mod.rs
        lookup.rs
        families.rs
        specialization.rs
        visibility.rs

      matching/
        mod.rs
        resolution.rs
        patterns.rs
        bindings.rs
        pattern_space.rs
        exhaustiveness.rs
        gadt_refinement.rs
        flow.rs
        diagnostics.rs
        conformance.rs

      COVERAGE.md

    incremental/
      ...
      adts.rs
      match_analysis.rs
```

### Responsibility rules

- `declarations.rs`: enum declaration structure, owner identity, generic parameter ownership, declaration ordering.
- `variants.rs`: variant/family identity, shape distinctions, visibility axes.
- `constructors.rs`: constructor typing/invocation surface and constructor identity.
- `exact_cases.rs`: exact-case types, subtyping/joins/substitution.
- `generics.rs`: generic ADT and declaration-level GADT specialization.
- `behavior.rs`: enum-wide/per-variant behavior contracts and overrides.
- `requirements.rs`: shared behavior requirements/compatibility.
- `associated/*`: associated lookup/family capture/specialization/visibility.
- `matching/*`: elimination only.
- `incremental/*`: dependency/fingerprint/reuse laws.
- `conformance.rs`: a small number of vertical source-driven scenarios only.

## 2.2 `phalcom-core`

```text
phalcom-core/tests/
  README.md
  language.rs
  core.rs

  support/
    assertions.rs
    cli.rs
    corpus.rs
    disasm.rs
    program.rs
    project.rs
    vm.rs

  language/
    mod.rs
    corpus.rs

    compiler/
      mod.rs
      declarations.rs
      contracts.rs
      lowering.rs

    algebraic_data/
      mod.rs
      support.rs
      construction.rs
      behavior.rs
      execution.rs
      matching.rs
      associated.rs
      gc.rs
      conformance.rs

  core/
    mod.rs

    execution/
      dispatch.rs
      limits.rs
      regressions.rs

    memory/
      collector.rs
      packs.rs

    observability/
      cli.rs
      disassembly.rs
      traceback.rs

    repl/
      session.rs
      immutability.rs
      source_maps.rs

    modules/
      compile.rs
      linking.rs
      runtime.rs

    universe.rs
    reflection.rs
    type_metadata.rs
    native_surface.rs
    object_model.rs
```

---

# 3. Test-Oracle Policy

Before adding scenarios, implementers must classify what a scenario is proving.

## 3.1 Source acceptance only

Allowed for syntax/corpus smoke tests:

```rust
assert_no_diagnostics(...)
```

This is insufficient when the law concerns candidate identity, exact typing, proof state, residual space, or lowering.

## 3.2 Semantic identity

Assert canonical semantic products:

```text
DeclarationId
VariantId
VariantFamilyId
VariantFieldId
TypeId / TypeData
TypeKnowledge
BindingId
MatchResolution
PatternResolution
BranchProofEnvironment
PatternSpaceSummary
PatternUsefulness
ExhaustivenessResult
CoverageWitness
```

## 3.3 Diagnostics

Assert at least:

```text
DiagnosticCode
severity where part of the contract
primary SourceRange
important labels/notes/witnesses
ExplanationRef when required
```

Do not make prose rendering the primary semantic contract.

## 3.4 Lowering

Assert backend facts only:

```text
exact VariantId
candidate ordering
physical payload slot
executable alternative structure
binding destination table
lowering-site attachment
```

Do not assert GADT proof objects in core lowering; they should be absent.

## 3.5 Runtime

Assert:

```text
returned value
side-effect counts
branch execution count
binding values
bytecode/disassembly architectural invariants
GC reachability where relevant
```

---

# 4. Task 0 — Baseline, Inventory, and Failure Classification

**Files:**
- Create: `phalcom-semantic/tests/semantic/adts/COVERAGE.md`
- Create: `phalcom-core/tests/README.md`
- No production code changes.

**Consumes:** current branch HEAD and existing test tree.

**Produces:** immutable baseline report for later comparison.

- [ ] Record:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log -1 --oneline
```

- [ ] Enumerate semantic ADT tests:

```bash
find phalcom-semantic/tests/semantic/adts -maxdepth 2 -type f -print | sort
```

- [ ] Enumerate core tests and manifest test targets:

```bash
find phalcom-core/tests -maxdepth 2 -type f -print | sort
rg -n '^\[\[test\]\]|^name = |^path = ' phalcom-core/Cargo.toml
```

- [ ] Run current focused semantic ADT suite:

```bash
cargo test -p phalcom-semantic --test semantic adts:: -- --nocapture
```

- [ ] Run current ADT/core standalone targets that exist on this HEAD.

- [ ] Record failures in a baseline table:

```text
command | failing test | existing/new | known unrelated? | diagnostic/error
```

- [ ] Populate `adts/COVERAGE.md` with law IDs but mark only scenarios already proven by deep enough enabled tests as `READY`.

- [ ] Commit documentation only.

---

# 5. Task 1 — Mechanical Semantic ADT Reorganization

**Files:**
- Move current match files under `semantic/adts/matching/`.
- Split GADT placement only where mechanical extraction can preserve tests unchanged.
- Modify `semantic/adts/mod.rs`.
- Create `semantic/adts/matching/mod.rs`.
- Create `semantic/adts/associated/mod.rs` only if existing associated ADT-specific tests are moved in this task.

**Rule:** no assertion strengthening in this task.

Suggested mapping:

```text
match_basic.rs        -> matching/resolution.rs + matching/flow.rs (mechanical split only)
match_patterns.rs     -> matching/patterns.rs
match_bindings.rs     -> matching/bindings.rs
pattern_space.rs      -> matching/pattern_space.rs
exhaustiveness.rs     -> matching/exhaustiveness.rs
match_diagnostics.rs  -> matching/diagnostics.rs
match_matrix.rs       -> matching/conformance.rs
match_regressions.rs  -> distribute to matching/{gadt_refinement,bindings,exhaustiveness}.rs
gadt_proofs.rs        -> matching/gadt_refinement.rs where elimination-specific
```

- [ ] Move one module group at a time.
- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic adts:: -- --nocapture
```

after each meaningful group.
- [ ] Require identical test behavior to baseline.
- [ ] Commit with a pure `test:`/`refactor(test):` move commit.

---

# 6. Task 2 — Build ADT Semantic Test Support

**Files:**
- Create: `phalcom-semantic/tests/semantic/adts/support.rs`
- Modify: `semantic/adts/mod.rs`
- Refactor a small number of existing tests to prove helper correctness.

**Produces:** reusable source-driven semantic fixture API.

Implement helpers conceptually equivalent to:

```rust
pub struct AdtCase {
    pub module: ModuleId,
    pub source: Arc<str>,
    pub analysis: SemanticAnalysisResult,
}

pub fn analyze_adt(source: &str) -> AdtCase;

impl AdtCase {
    pub fn diagnostics(&self) -> &[SemanticDiagnostic];
    pub fn assert_no_diagnostics(&self);
    pub fn declaration(&self, name: &str) -> &DeclarationInfo;
    pub fn variant(&self, owner: &str, selector: Selector) -> &VariantInfo;
    pub fn only_match(&self) -> MatchHandle<'_>;
    pub fn match_in_callable(&self, owner: &str, selector: Selector, index: usize) -> MatchHandle<'_>;
}
```

`MatchHandle` should expose semantic assertions without callers reimplementing snapshot plumbing:

```rust
pub struct MatchHandle<'a> {
    resolution: &'a MatchResolution,
    snapshot: &'a SemanticSnapshot,
}

impl MatchHandle<'_> {
    pub fn arm(&self, index: usize) -> ArmHandle<'_>;
    pub fn assert_exhaustive(&self);
    pub fn assert_initial_space(&self, expected: &PatternSpaceSummary);
    pub fn assert_result_type(&self, expected: TypeId);
}
```

Add identity-oriented utilities:

```text
variant_id(owner, selector)
family_id(owner, base)
field_id(variant, selector-slot)
find_binding(name)
diagnostic(code)
```

Do not make stringified type presentation the only assertion path; provide canonical `TypeData` access.

Verification:

```bash
cargo test -p phalcom-semantic --test semantic adts:: -- --nocapture
```

---

# 7. Task 3 — Declaration, Variant, Constructor, Exact-Case Coverage

**Files:**
- Extend/create:
  - `adts/declarations.rs`
  - `adts/variants.rs`
  - `adts/constructors.rs`
  - `adts/exact_cases.rs`
  - `adts/generics.rs`

**Scenario source:** companion catalog sections D, V, C, E, G.

Add all scenarios described there. For each scenario:

- [ ] Add the source fixture or direct semantic construction specified.
- [ ] Add the deepest required oracle.
- [ ] Run the exact module filter.
- [ ] If the scenario unexpectedly fails, follow the catalog debugging procedure before changing expected results.
- [ ] Update `adts/COVERAGE.md` after the scenario passes.
- [ ] Commit by responsibility, not by individual test.

Focused commands:

```bash
cargo test -p phalcom-semantic --test semantic adts::declarations -- --nocapture
cargo test -p phalcom-semantic --test semantic adts::variants -- --nocapture
cargo test -p phalcom-semantic --test semantic adts::constructors -- --nocapture
cargo test -p phalcom-semantic --test semantic adts::exact_cases -- --nocapture
cargo test -p phalcom-semantic --test semantic adts::generics -- --nocapture
```

---

# 8. Task 4 — ADT Behavior and Requirements

**Files:**
- Extend/create:
  - `adts/behavior.rs`
  - `adts/requirements.rs`

Test:
- enum-wide behavior inherited by variants;
- per-variant override;
- per-variant additional behavior;
- shared method contract satisfaction;
- missing required behavior;
- wrong selector/arity;
- wrong return compatibility;
- generic requirement specialization;
- visibility interactions.

Do not use runtime-only success to prove semantic contract compatibility.

---

# 9. Task 5 — Associated Lookup and Family Coverage

**Files:**
- Create/extend:
  - `adts/associated/lookup.rs`
  - `families.rs`
  - `specialization.rs`
  - `visibility.rs`

**Scenario source:** companion catalog section A.

Assertions must include exact member/family identities. Where a family captures multiple members, assert the exact ordered or canonical set required by the semantic API.

Key negative laws:
- exact selector miss;
- wrong call shape;
- ambiguous owner;
- inaccessible explicit acquisition;
- live hierarchy member outside captured/frozen capability where confinement requires rejection;
- capability escape/visibility rules.

Focused command:

```bash
cargo test -p phalcom-semantic --test semantic adts::associated:: -- --nocapture
```

---

# 10. Task 6 — Formal Match Resolution

**Files:**
- `adts/matching/resolution.rs`

**Scenario source:** companion catalog section M1.

Strengthen existing match tests that currently only prove no diagnostics.

Every identity-sensitive match scenario must inspect `MatchResolution`:

```text
owner
family
selector constraint
ordered candidate VariantIds
candidate exact_case
candidate field projections
```

For selector-gap/family patterns, assert candidate-specific field IDs, not counts only.

Current existing regression tests from HEAD should be preserved but deepened where appropriate:
- generic GADT cases remain reachable before branch equalities;
- or-pattern binding mismatch;
- duplicate binding;
- redundant arm severity.

---

# 11. Task 7 — Recursive Patterns and Bindings

**Files:**
- `adts/matching/patterns.rs`
- `adts/matching/bindings.rs`

**Scenario source:** catalog sections M2 and M3.

Required deep laws:
- wildcard creates zero bindings/projections when value is unused;
- nested contextual resolution uses the parent payload expected space;
- one shared `BindingId` is visible after an or-pattern;
- same-name alternatives join type knowledge;
- different binding sets are rejected;
- duplicate names within one alternative are rejected;
- family candidates join value types but not alternative-specific GADT proof facts.

---

# 12. Task 8 — Pattern-Space Algebra

**Files:**
- `adts/matching/pattern_space.rs`
- If direct internal APIs are not publicly callable, add narrowly scoped test-only/public(crate) seams rather than testing only through diagnostics.

**Scenario source:** catalog section P.

Implement table-driven laws for normalization, intersection, subtraction, nested residuals, union exactness, tuple decomposition, and opaque conservatism.

Do not weaken tests to rendered summaries if exact structured `PatternSpace` comparison is available.

---

# 13. Task 9 — Exhaustiveness, Usefulness, Witnesses

**Files:**
- `adts/matching/exhaustiveness.rs`

**Scenario source:** catalog section X.

Required categories:
- closed ADT totality;
- exact-case totality;
- alias/union totality;
- family/selector totality;
- nested residual totality;
- tuple/list structural totality where supported;
- opaque/open domain requires wildcard;
- `Impossible` vs `Redundant`;
- exact structured missing witnesses.

Redundant and impossible are compile errors in the current Part 05.1 contract.

---

# 14. Task 10 — GADT Elimination and Proof Refinement

**Files:**
- `adts/matching/gadt_refinement.rs`

**Scenario source:** catalog section R.

This task is soundness-critical.

For the canonical generic evaluator:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

the test must prove more than no diagnostics:
- both candidates are reachable under `Expr<T>`;
- `Int` arm proof contains `T = Int`;
- `Bool` arm proof contains `T = Bool`;
- binding `x` specializes accordingly;
- shared/or branch proof retains only common facts;
- proof does not escape the branch.

Add specialization-impossible tests for `Expr<Int>` and `Expr<Bool>`.

---

# 15. Task 11 — Match Result Typing and Flow

**Files:**
- `adts/matching/flow.rs`

**Scenario source:** catalog section F.

Test:
- homogeneous result join;
- heterogeneous union;
- abrupt branch exclusion;
- all-abrupt `Never`;
- expected-type propagation;
- stable scrutinee exact-case refinement;
- family candidate-union refinement;
- branch-local binding scope;
- post-match flow join;
- negative knowledge across ordered arms.

Inspect both formal/declared and observed/refined knowledge where the product exposes them.

---

# 16. Task 12 — Match Diagnostics and Explanation DAG

**Files:**
- `adts/matching/diagnostics.rs`

**Scenario source:** catalog section N.

Provide one enabled source scenario for every match diagnostic code required by Part 05.1.

For every scenario assert:
- machine code;
- severity if meaningful;
- primary range;
- important labels/notes;
- coverage witness for non-exhaustive;
- explanation linkage where the diagnostic contract promises it.

Avoid whole-render golden tests except as a secondary presentation check.

---

# 17. Task 13 — Incremental ADT / Match Testing

**Files:**
- Create: `semantic/incremental/adts.rs`
- Create: `semantic/incremental/match_analysis.rs`
- Modify: `semantic/incremental/mod.rs`

**Scenario source:** catalog section I.

Metamorphic edit tests:
- add/remove reachable enum case;
- add/remove family member;
- change payload type;
- change GADT result specialization;
- alias union expansion/contraction;
- visibility change;
- unrelated edit reuse;
- semantic match fingerprint changes only for semantic product changes.

Use the repository's existing incremental DB/test fixture model; do not create a second database harness.

---

# 18. Task 14 — Convert `phalcom-core` to Two Integration Binaries

**Files:**
- Modify: `phalcom-core/Cargo.toml`
- Create:
  - `tests/language.rs`
  - `tests/core.rs`
  - module trees listed in Section 2.2
- Move existing tests mechanically.

**Rule:** no behavior/assertion changes during this task.

Manifest end state:

```toml
[[test]]
name = "language"
path = "tests/language.rs"

[[test]]
name = "core"
path = "tests/core.rs"
```

Mechanical placement guidance:

```text
adt_case_primitives.rs
adt_runtime.rs
adt_end_to_end.rs
    -> language/algebraic_data/execution.rs

adt_behavior.rs
    -> language/algebraic_data/behavior.rs

adt_gc.rs
associated_family_gc.rs
    -> language/algebraic_data/gc.rs

associated_lowering.rs
semantic_lowering.rs
    -> language/compiler/lowering.rs and/or language/algebraic_data/associated.rs

associated_family_runtime.rs
associated_reification.rs
    -> language/algebraic_data/associated.rs

f2_pack_gc.rs
    -> core/memory/packs.rs

spec02_invariants.rs
    -> core/type_metadata.rs

spec03_* reflection/metadata tests
    -> core/reflection.rs or core/type_metadata.rs
```

Remove duplicate inclusion through old `integration`.

After each move group run filtered tests.

---

# 19. Task 15 — Core Test Support Consolidation

**Files:**
- Create/normalize:
  - `tests/support/program.rs`
  - `project.rs`
  - `vm.rs`
  - `disasm.rs`
  - `assertions.rs`
  - `corpus.rs`
  - `cli.rs`
- Keep ADT-only helpers in `language/algebraic_data/support.rs`.

Generic helpers may include:

```text
compile_inline
run_inline
compile_project
run_project
disassemble
assert_opcode_present
assert_opcode_absent
assert_runtime_value
assert_compile_error
```

Do not introduce giant all-purpose support objects.

---

# 20. Task 16 — Part 05.2 Lowering Architecture Tests

**Files:**
- `language/compiler/lowering.rs`
- `language/algebraic_data/matching.rs`

**Scenario source:** catalog section L.

Assert:
- one lowering site per semantic match;
- exact `VariantId` retained;
- `VariantFieldId` maps to physical slot;
- cross-module projection;
- wildcard/gap extraction elision;
- candidate order;
- missing/non-proven semantic product fails closed;
- no GADT proof/exhaustiveness reasoning in backend IR.

Architectural source searches are test gates, not substitutes for behavior tests.

---

# 21. Task 17 — Executable Match Runtime Coverage

**Files:**
- `language/algebraic_data/matching.rs`
- `language/algebraic_data/execution.rs`

**Scenario source:** catalog section Q.

Run real source through compiler+VM. Cover:
- singleton;
- nullary constructor;
- payload/labeled payload;
- nested ADT;
- wildcard;
- or-pattern;
- family/selector-gap;
- match result;
- braced arm;
- return from braced arm;
- one scrutinee evaluation;
- one selected branch execution.

Pair representative runtime scenarios with bytecode architecture assertions:
- `IsVariant`;
- `GetVariantPayload`;
- no `.class` ADT identity;
- no source-name global lookup;
- no generic/GADT runtime proof machinery.

---

# 22. Task 18 — Shared Pattern Context Coverage

**Files:**
- `language/algebraic_data/matching.rs`
- existing compiler pattern-context files as implementation requires.

Run equivalent patterns through:
- `match`;
- `if let`;
- `while let`;
- required destructuring;
- `for pattern` where grammar allows.

Required no-leak scenario:
1. candidate matches outer variant;
2. payload is extracted/staged;
3. nested child test fails;
4. failure branch/context runs;
5. source-visible binding must not have been committed.

Where source scope cannot observe leakage directly, inspect generated bytecode/staging writes.

---

# 23. Task 19 — GC, Ownership, Cross-Module, and Vertical Conformance

**Files:**
- `language/algebraic_data/gc.rs`
- `language/algebraic_data/conformance.rs`
- semantic `adts/matching/conformance.rs`

**Scenario source:** catalog sections GC and VERT.

Require at least six vertical programs:
1. Generic `Result<T,E>`.
2. GADT `Expr<T>`.
3. Same-base multi-selector variant family.
4. Nested `Option<Result<...>>`.
5. Visibility/construction/payload elimination.
6. Cross-module enum match and payload layout.

Each semantic vertical test inspects formal products.
Each core vertical test executes and inspects representative bytecode.

---

# 24. Task 20 — Coverage Ledger and Documentation

**Files:**
- Update: `phalcom-semantic/tests/semantic/adts/COVERAGE.md`
- Update: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` only where broad semantic slots are genuinely promoted.
- Update: `phalcom-core/tests/README.md`.

`adts/COVERAGE.md` columns:

```text
Law ID | Status | Semantic test | Core test | Spec section | Notes
```

Status:

```text
READY
RED-CAPABILITY
STAGED
GATED
```

A law becomes `READY` only with the required enabled oracle.

---

# 25. Debugging Strategy Required During Execution

When a new scenario fails, do not immediately weaken the expected result.

Use this order:

## 25.1 Parsing/surface

If source fails to parse:
- isolate with `phalcom-ast` parser tests;
- compare syntax with ratified 05.1 surface;
- distinguish unsupported syntax from semantic failure.

## 25.2 Declaration identity

If variant/family lookup differs:
- dump owner `DeclarationId`;
- enumerate `EnumSemanticTable` variants;
- print selectors, `VariantId`, `VariantFamilyId`, field IDs;
- verify singleton vs nullary vs callable selector projection.

## 25.3 Type/proof state

If a GADT or exact-case test fails:
- inspect canonical `TypeData`;
- inspect `CaseTypeEnvironment`;
- inspect branch `BranchProofEnvironment`;
- compare pre-branch and branch substitutions;
- verify the implementation is not using ordinary subtype filtering where equality introduction is required.

## 25.4 Coverage

If exhaustiveness is wrong:
- print `initial_space`;
- print resolved arm matched space;
- print `reachable_space`;
- print `residual_after` after every arm;
- classify whether the error is candidate formation, intersection, subtraction, or witness generation.

## 25.5 Flow

If binding/result type is wrong:
- inspect branch-local binding `TypeKnowledge`;
- inspect evidence origin;
- inspect only normally completing branch joins;
- verify coverage state was not incorrectly widened into ordinary `FlowState`.

## 25.6 Lowering

If runtime candidate/field is wrong:
- inspect semantic `MatchResolution` first;
- then inspect `ModuleLoweringSemantics`;
- verify `VariantFieldId -> physical slot`;
- never patch compiler source-name resolution.

## 25.7 Runtime

If VM output is wrong:
- disassemble;
- verify scrutinee evaluation count;
- verify `IsVariant` targets;
- verify payload slots;
- verify staged bindings commit only after full success;
- verify branch jumps and scratch cleanup.

## 25.8 Incremental

If edit invalidation is wrong:
- compare semantic dependencies before/after;
- inspect callable product fingerprint;
- distinguish missing dependency edge from over-broad invalidation.

---

# 26. Commit Strategy

Recommended commit boundaries:

```text
test: reorganize semantic ADT suite
test: add ADT semantic fixture support
test: deepen ADT identity and exact-case coverage
test: complete associated family semantic coverage
test: complete formal match resolution coverage
test: prove pattern-space algebra and exhaustiveness
test: complete GADT elimination proof coverage
test: cover match flow diagnostics and explanations
test: cover ADT incremental invalidation
test: consolidate core integration binaries
test: reorganize core ADT language coverage
test: cover executable match lowering
test: cover shared pattern runtime execution
test: complete ADT vertical conformance
docs: record ADT testing coverage
```

Avoid commits that combine test-tree moves with semantic/runtime bug fixes.

---

# 27. Final Verification Matrix

## Formatting and compile

```bash
cargo fmt --all -- --check
cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core
```

## Semantic focused

```bash
cargo test -p phalcom-semantic --test semantic adts:: -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental::adts -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental::match_analysis -- --nocapture
```

## Semantic full

```bash
cargo test -p phalcom-semantic --test semantic -- --nocapture
cargo test -p phalcom-semantic --lib -- --nocapture
```

## Core focused

```bash
cargo test -p phalcom-core --test language language::algebraic_data:: -- --nocapture
cargo test -p phalcom-core --test language language::compiler::lowering:: -- --nocapture
cargo test -p phalcom-core --test core core::memory:: -- --nocapture
```

## Core full

```bash
cargo test -p phalcom-core --test language -- --nocapture
cargo test -p phalcom-core --test core -- --nocapture
cargo test -p phalcom-core --lib -- --nocapture
```

## Architecture searches

```bash
rg -n 'constructor == "Some"|constructor == "None"|emit_class_test\(value_slot, constructor' phalcom-core/src/compiler
rg -n 'SelectorPattern::matches|CaseTypeEnvironment|PatternSpace|ExhaustivenessResult' phalcom-core/src/compiler
```

Formal ADT pattern paths must not contain source-level semantic rediscovery.

## Repository state

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log --oneline --decorate -20
```

Report:
- starting HEAD;
- final HEAD;
- commits;
- files moved/created;
- test counts/scenarios;
- exact command outcomes;
- baseline failures still present;
- newly discovered semantic/runtime bugs;
- any `STAGED`/`GATED` laws remaining.

---

# 28. Completion Criteria

This project is complete when:

- semantic ADT tests have stable conceptual organization;
- core tests compile through only `language` and `core` integration binaries;
- duplicate ADT runtime compilation through old targets is removed;
- every law in the scenario catalog has an explicit state;
- identity-sensitive tests assert canonical semantic identities;
- match tests assert candidate/projection/proof/coverage products;
- GADT tests prove branch-local equality introduction and non-leakage;
- incremental edits invalidate exactly the semantic products they affect;
- runtime tests prove `IsVariant`/`GetVariantPayload` execution without semantic rediscovery;
- failed refutable patterns do not commit partial bindings;
- vertical tests prove source → semantic theorem → lowering → runtime coherence;
- coverage documentation reflects actual enabled tests rather than intended future work;
- the final verification matrix has been executed and results recorded.
