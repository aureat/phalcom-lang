# ADT / GADT / Associated-Family Conformance Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the remaining semantic, lowering, runtime, source-index, visibility, and vertical tests required by the ADT/GADT/associated-family conformance program.

**Architecture:** Keep formal semantic products authoritative. Extend compiler lowering to consume `FamilyApplicationResolution` and emit frozen-capability bytecode; make VM family dispatch select only from the captured descriptor/candidate set. Add source-driven tests at each semantic boundary, then compose a small set of end-to-end scenarios.

**Tech Stack:** Rust 2024, Cargo integration tests, `phalcom-semantic` snapshots, `phalcom-core` bytecode/VM, `graphify`.

**Spec:** `/Users/altunhasanli/.codex/attachments/8affab36-6302-4bfd-baee-9bbb37df897f/pasted-text.txt`

## Global Constraints

- Formal `SemanticSnapshot` and its `AssociatedResolution` / `FamilyApplicationResolution` products remain the only semantic authority consumed by lowering.
- A captured associated family is a frozen capability; invocation must not rediscover members through an owner, hierarchy walk, or does-not-understand fallback.
- Singleton variants remain canonical values; zero-argument and payload constructors remain fresh callable cases.
- GADT equality/proof state is compile-time only and must not be added to runtime ADT objects.
- Visibility is checked at acquisition; authorized capabilities may cross later lexical boundaries without re-checking source scope.
- Preserve unrelated dirty files and classify existing full-workspace failures separately from owned test failures.

---

### Task 1: Wire semantic family application into compiler lowering

**Files:**
- Modify: `phalcom-core/src/compiler/lib/expr.rs` around the disabled `Expr::Call` arm.
- Modify: `phalcom-core/src/compiler/lib/associated.rs` to compile `FamilyApplicationLoweringSpec`.
- Modify: `phalcom-core/src/modules/semantic_lowering.rs` only where projection needs missing provenance.
- Test: `phalcom-semantic/tests/semantic/families/invocation.rs`.
- Test: `phalcom-core/tests/associated_lowering.rs`.
- Test: `phalcom-core/tests/semantic_lowering.rs`.

**Interfaces:**
- Consumes: `SemanticSnapshot::callable_analyses[*].family_applications`, `FamilyApplicationResolution`, and `FamilyApplicationSelection`.
- Produces: one `FamilyApplicationLoweringSpec` per `LoweringSiteKind::FamilyApplication`, and bytecode for static and dynamic family calls.

- [ ] **Step 1: Write semantic resolution tests first.** Add source-driven tests for an immediate family call and a stored family call. Assert the `ExpressionId` has `FamilyApplicationResolution`, static selection retains `FamilyOperationShape`, and dynamic selection retains every `FamilyApplicationCandidate`.

```rust
let resolution = callable.family_applications.get(&expression.id).expect("family application");
assert!(matches!(resolution.selection, FamilyApplicationSelection::Static { .. }));
```

- [ ] **Step 2: Run the semantic target and record the current failure.**

```text
cargo test -p phalcom-semantic --test semantic families::invocation -- --nocapture
```

Expected current result: the source call has no published family-application resolution because `Expr::Call` code generation is disabled.

- [ ] **Step 3: Implement the smallest compiler bridge.** In `Expr::Call`, find the formal lowering record by source range and emit the static operation bytecode or dynamic-pack bytecode. Compile the callee, preserve positional/labeled/expanded pack lanes, and return `CompilerError::MissingFamilyApplicationResolution` when no formal record exists.

- [ ] **Step 4: Add lowering assertions.** Assert `ModuleLoweringSemantics.family_applications` contains one record for each call, static operations preserve labels/slots, and dynamic candidates preserve target identity and candidate order.

- [ ] **Step 5: Run focused gates.**

```text
cargo test -p phalcom-semantic --test semantic families::invocation -- --nocapture
cargo test -p phalcom-core --test associated_lowering --test semantic_lowering -- --nocapture
```

- [ ] **Step 6: Commit the task.**

```text
git add phalcom-core/src/compiler/lib/expr.rs phalcom-core/src/compiler/lib/associated.rs phalcom-core/src/modules/semantic_lowering.rs phalcom-semantic/tests/semantic/families/invocation.rs phalcom-core/tests/associated_lowering.rs phalcom-core/tests/semantic_lowering.rs
git commit -m "feat: lower associated family applications from semantic products"
```

### Task 2: Make runtime family invocation honor frozen capabilities

**Files:**
- Modify: `phalcom-core/src/vm/dispatch.rs` in `InvokeAssociatedFamilyStatic` and `InvokeAssociatedFamilyPack` arms.
- Modify: `phalcom-core/src/error.rs` only if a typed mismatch error is missing.
- Test: `phalcom-core/tests/associated_family_runtime.rs`.
- Test: `phalcom-core/tests/associated_family_gc.rs`.

**Interfaces:**
- Consumes: `AssociatedFamilyObject.descriptor`, `FamilyApplicationLoweringSpec`, runtime pack lanes, and `ExecutableInvocationTarget`.
- Produces: exact static selection, deterministic dynamic candidate selection, typed mismatch errors, and no owner/hierarchy rediscovery.

- [ ] **Step 1: Add failing runtime tests.** Cover immediate static invocation, stored-family invocation, dynamic pack selection, and a confinement case where the receiver's live hierarchy exposes a member absent from the captured descriptor.

```rust
assert_eq!(run_family_call("factory()"), Value::int(7));
assert!(matches!(run_family_call("family(***pack)"), Err(PhError::Runtime(RuntimeError::AssociatedFamilyNoMatchingCandidate))));
```

- [ ] **Step 2: Run the focused target.**

```text
cargo test -p phalcom-core --test associated_family_runtime -- --nocapture
```

Expected current result: dynamic dispatch selects the first compiler candidate with zero arity and ignores the runtime pack and family descriptor.

- [ ] **Step 3: Implement static selection.** Match the encoded `FamilyOperationShape` against `fam.descriptor.entries`; consume exactly `arity` arguments; dispatch singleton, constructor, and behavioral targets without a hierarchy search.

- [ ] **Step 4: Implement dynamic selection.** Decode the positional/labeled/complete pack lanes, match only `cand_set.candidates`, reject candidates not present in the frozen family, and pass the selected argument lanes and arity to the target.

- [ ] **Step 5: Add GC assertions using compiler-produced families.** Capture a class-side family with a bound owner, root the family, force GC, and assert the owner survives; drop the family and assert both objects are collectible.

- [ ] **Step 6: Run focused and runtime primitive gates.**

```text
cargo test -p phalcom-core --test associated_family_runtime --test associated_family_gc --test adt_case_primitives -- --nocapture
```

- [ ] **Step 7: Commit the task.**

```text
git add phalcom-core/src/vm/dispatch.rs phalcom-core/src/error.rs phalcom-core/tests/associated_family_runtime.rs phalcom-core/tests/associated_family_gc.rs
git commit -m "fix: dispatch associated families from frozen candidates"
```

### Task 3: Cover associated visibility and capability transfer

**Files:**
- Inspect/modify: `phalcom-semantic/src/surface.rs`, `phalcom-semantic/src/checker/associated.rs`, and associated visibility helpers only when a failing test identifies a defect.
- Create: `phalcom-semantic/tests/semantic/associated/visibility.rs`.
- Extend: `phalcom-semantic/tests/semantic/families/values.rs` and `phalcom-semantic/tests/semantic/integration/family_capabilities.rs`.
- Test: `phalcom-core/tests/associated_family_runtime.rs`.

**Interfaces:**
- Consumes: `MemberVisibility`, associated surface family filtering, `AssociatedResolutionKind::Family`, and frozen runtime descriptors.
- Produces: acquisition-time authorization and capability-use tests.

- [ ] **Step 1: Add semantic tests for private exact acquisition, private family filtering, privileged capture, and all-members-inaccessible diagnostics.** Assert diagnostic code, primary range, and exact retained members.

- [ ] **Step 2: Add an escape test.** Capture `Vault::open::*` inside an authorized class-side method, return it, and invoke it from an unprivileged caller. Assert no second lexical visibility check occurs.

- [ ] **Step 3: Add a confinement test.** Omit a private member from capture, then invoke with a dynamic pack outside the scope; assert `AssociatedFamilyNoMatchingCandidate`, not successful hierarchy lookup.

- [ ] **Step 4: Run focused gates.**

```text
cargo test -p phalcom-semantic --test semantic associated::visibility families::values integration::family_capabilities -- --nocapture
cargo test -p phalcom-core --test associated_family_runtime -- --nocapture
```

- [ ] **Step 5: Commit the task.**

```text
git add phalcom-semantic/tests/semantic/associated/visibility.rs phalcom-semantic/tests/semantic/families/values.rs phalcom-semantic/tests/semantic/integration/family_capabilities.rs phalcom-core/tests/associated_family_runtime.rs
git commit -m "test: cover associated capability visibility"
```

### Task 4: Complete family type, rest-routing, and flow laws

**Files:**
- Create or extend: `phalcom-semantic/tests/semantic/families/invocation.rs`.
- Create: `phalcom-semantic/tests/semantic/families/rest_routing.rs`.
- Extend: `phalcom-semantic/tests/semantic/families/types.rs` and `phalcom-semantic/tests/semantic/families/flow.rs`.
- Inspect: `phalcom-semantic/src/types/family.rs` and family application selection code.

**Interfaces:**
- Consumes: `FamilyOperationShape`, structural `FamilyType`, `FamilyApplicationSelection`, and `ValueSemanticFact::merge`.
- Produces: exact-before-rest, lane-preserving, structural-subtyping, and capability-flow evidence.

- [ ] **Step 1: Add type-law tests.** Assert value/callable kind distinction, setter/subscript/operator shapes, duplicate identical-operation canonicalization, conflicting duplicate rejection, family substitution, and structural subset subtyping.

- [ ] **Step 2: Add rest-routing tests.** Build families containing exact, positional-rest, labeled-rest, and complete-rest operations. Assert exact wins, lanes remain separate, deterministic candidate order is preserved, and inaccessible exact members do not fall through.

- [ ] **Step 3: Add flow tests.** Merge identical captures and assert denotation survives; merge different captures and assert only safe structural type remains; assert later invocation does not reconstruct lost nominal provenance.

- [ ] **Step 4: Run focused gates.**

```text
cargo test -p phalcom-semantic --test semantic families:: -- --nocapture
```

- [ ] **Step 5: Commit the task.**

```text
git add phalcom-semantic/tests/semantic/families
git commit -m "test: cover structural family routing and flow laws"
```

### Task 5: Expand source-index dependencies and fingerprints

**Files:**
- Create: `phalcom-semantic/tests/semantic/source_semantics/dependencies.rs`.
- Create: `phalcom-semantic/tests/semantic/source_semantics/fingerprints.rs`.
- Extend: `phalcom-semantic/tests/semantic/source_semantics/source_index.rs`.
- Inspect/modify: `phalcom-semantic/src/source_index/mod.rs`, `phalcom-semantic/src/session.rs`, and dependency publication code only when tests fail.

**Interfaces:**
- Consumes: `SourceSemanticIndex`, `SemanticTargetId`, `SourceSiteId`, semantic graph dependencies, and incremental snapshot publication.
- Produces: exact target attachment, invalidation, and range/fingerprint metamorphic evidence.

- [ ] **Step 1: Add target mapping tests.** For singleton, constructor reference, direct constructor call, behavioral reference, inherited behavioral reference, and family capture, locate the selector source site and assert its exact `SemanticTargetId`.

- [ ] **Step 2: Add dependency tests.** Change an unrelated body, associated signature, variant selector, and GADT result independently; assert only the semantically dependent products are invalidated.

- [ ] **Step 3: Add fingerprint metamorphic tests.** Keep semantic identity stable across range-only and local-binding-name edits; change external labels/visibility/candidate sets and assert the relevant fingerprint changes.

- [ ] **Step 4: Run focused incremental gates.**

```text
cargo test -p phalcom-semantic --test semantic source_semantics:: incremental:: -- --nocapture
```

- [ ] **Step 5: Commit the task.**

```text
git add phalcom-semantic/tests/semantic/source_semantics
git commit -m "test: verify semantic source dependencies and fingerprints"
```

### Task 6: Fill ADT behavior, requirements, and composition gaps

**Files:**
- Create: `phalcom-semantic/tests/semantic/adts/behavior.rs`.
- Create: `phalcom-semantic/tests/semantic/adts/composition.rs`.
- Extend: `phalcom-semantic/tests/semantic/adts/requirements.rs`.
- Inspect/modify: `phalcom-semantic/src/enum_requirements.rs` and enum behavior publication only when a test fails.

**Interfaces:**
- Consumes: `EnumInfo`, `VariantInfo`, case-owned `CallableId`, `EnumRequirementTable`, and GADT case environments.
- Produces: behavior override/addition, payload proof, root-default, singleton/private-construction, and multi-product composition evidence.

- [ ] **Step 1: Add behavior tests.** Assert root behavior applies to all cases, case overrides shadow root behavior, case-added behavior is case-local, payload fields are visible only under the case environment, and constructors are not class behaviors.

- [ ] **Step 2: Strengthen requirement tests.** Cover wrong return type, positional/labeled parameter mismatch, selector mismatch, root default satisfaction, case override incompatibility, singleton participation, private-construction participation, and GADT-specialized results.

- [ ] **Step 3: Add composition test.** Analyze one generic `Result<T,E>` program and assert declaration, variant identity, exact case type, constructor signature, requirement status, payload type, and case callable ownership in one snapshot.

- [ ] **Step 4: Run focused gates.**

```text
cargo test -p phalcom-semantic --test semantic adts:: -- --nocapture
```

- [ ] **Step 5: Commit the task.**

```text
git add phalcom-semantic/tests/semantic/adts
git commit -m "test: cover ADT behavior requirements and composition"
```

### Task 7: Close vertical conformance and release gates

**Files:**
- Extend: `phalcom-core/tests/adt_end_to_end.rs`.
- Extend: `phalcom-core/tests/associated_reification.rs` and `phalcom-core/tests/associated_family_runtime.rs`.
- Extend: `phalcom-semantic/tests/semantic/integration/adt_associated.rs`, `generic_adts.rs`, `gadt_associated.rs`, and `family_capabilities.rs`.
- Update: this plan's checkboxes with PASS/FAIL evidence after each vertical gate; no separate coverage ledger currently exists in the repository.

**Interfaces:**
- Consumes: all formal semantic, lowering, runtime, visibility, source-index, and family-application products from Tasks 1–6.
- Produces: decisive end-to-end evidence for three variant forms, generic constructors, GADT erasure, inherited behavior, stored family calls, dynamic confinement, visibility capabilities, and ordinary ADT behavior.

- [ ] **Step 1: Add the three-form vertical scenario.** Follow singleton, zero-argument constructor, and payload constructor from `VariantId` through semantic lowering, bytecode, runtime identity, freshness, and payload layout.

- [ ] **Step 2: Add generic/GADT vertical scenarios.** Assert `Option<Int>::Some(42)` and `Expr<Int>::IntLit(42)` retain exact semantic targets while runtime objects contain only variant identity and payload.

- [ ] **Step 3: Add inherited and stored-capability scenarios.** Execute an inherited class-side associated call, then capture/store/pass/invoke a frozen family and assert defining owner, lookup owner, and receiver behavior remain distinct.

- [ ] **Step 4: Run all owned gates.**

```text
cargo test -p phalcom-semantic --test semantic --no-fail-fast -- --nocapture
cargo test -p phalcom-core --tests --no-fail-fast
rustfmt --check --edition 2024 \
  phalcom-core/src/compiler/lib/expr.rs \
  phalcom-core/src/compiler/lib/associated.rs \
  phalcom-core/src/modules/semantic_lowering.rs \
  phalcom-core/src/vm/dispatch.rs \
  phalcom-semantic/src/checker/associated.rs \
  phalcom-semantic/tests/semantic/adts/*.rs \
  phalcom-semantic/tests/semantic/associated/*.rs \
  phalcom-semantic/tests/semantic/families/*.rs \
  phalcom-semantic/tests/semantic/integration/*.rs \
  phalcom-semantic/tests/semantic/source_semantics/*.rs \
  phalcom-core/tests/adt_*.rs \
  phalcom-core/tests/associated_*.rs \
  phalcom-core/tests/semantic_lowering.rs
git diff --check
graphify update .
```

- [ ] **Step 5: Classify failures.** Keep unrelated advisory/type-store/runtime fixture failures as baseline evidence; a conformance task is complete only when all newly added tests and all named family-application gates are green.

- [ ] **Step 6: Commit the completed conformance slice.**

```text
git add phalcom-semantic/tests/semantic/integration phalcom-core/tests/adt_end_to_end.rs phalcom-core/tests/associated_reification.rs phalcom-core/tests/associated_family_runtime.rs docs
git commit -m "test: complete ADT associated-family vertical conformance"
```
