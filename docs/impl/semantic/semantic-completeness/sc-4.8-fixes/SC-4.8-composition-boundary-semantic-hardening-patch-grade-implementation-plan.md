# SC-4.8 Composition-Boundary Semantic Hardening
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Exact remote HEAD:** `e932aac4e21a5b346e719ede5a24f94e7b924ab3`  
**HEAD subject:** `feat(semantic): complete SC-4.8 typing integration`  
**Parent:** `7ed4f571f16cdd1fdd957740079e42b01d654d4d`  
**Preparation date:** 2026-09-04  
**Companion specification:** `SC-4.8-composition-boundary-semantic-hardening-technical-spec.md`

> **Repository-state limitation**
>
> This plan was prepared from the GitHub repository state at the exact revision above. The remote `main` branch and commit were inspected directly. This session cannot independently inspect the implementer's local working tree, staged changes, untracked files, or local-only commits. Before execution, the implementing agent must perform the C0 drift/working-tree checks locally and record any relevant divergence.

---

# 1. Implementation Program

This program hardens the already-implemented SC-4.8 type-system completion work to full theoretical correctness at composition boundaries.

The program is **not** a redesign of SC-4.8. The following existing architecture remains authoritative:

- declaration-owned and callable-owned generic domains remain distinct canonical `GenericSignature` products;
- `CallableApplicationTarget` composes declaration and callable generic domains only in one query-local application;
- variant-local generic binders remain owned by the deterministic variant-constructor `CallableId`;
- `InvocationTargetId::VariantConstructor` remains the executable construction identity;
- `TypeData::ExactCase { variant, enum_type }` remains the durable exact-case representation;
- `RigidArena`, `RigidScopeId`, `RigidTypeVariableId`, and `LocalType` remain query-local existential machinery;
- `TypeStore` remains rigid-free;
- generic setters/indexers/getters/constructors/variants continue to use the ordinary generic application machinery;
- receiver specialization remains separate from callable identity;
- Family types remain first-class structural values rather than being re-resolved by name.

The remediation closes defects in:

1. evidence-sensitive GADT equality;
2. alpha-equivalence and branch-proof compatibility;
3. recursive local pattern typing;
4. lexical existential scope ownership;
5. proof-aware escape/publication;
6. local existential use through generic calls;
7. construction-side GADT equation ordering;
8. record-row, Family, and type-lambda structural completeness;
9. recovery/field correspondence;
10. index-setter `Unit` conformance;
11. source-index parity;
12. durable variant-generic metadata;
13. incremental binder-normalized semantics;
14. avoidable local allocations and duplicate structural walkers.

---

# 2. Repository Investigation Summary

## 2.1 Repository state

The remote default branch is `main`, currently at:

```text
e932aac4e21a5b346e719ede5a24f94e7b924ab3
feat(semantic): complete SC-4.8 typing integration
```

The relevant parent is:

```text
7ed4f571f16cdd1fdd957740079e42b01d654d4d
```

No local working-tree claim is made by this plan.

## 2.2 Relevant crates/subsystems

| Concern | Repository owner |
|---|---|
| AST / generic accessor / enum syntax | `phalcom-ast` |
| canonical type algebra / kinds / rows / lambdas / Families | `phalcom-semantic/src/types/` |
| transient inference | `phalcom-semantic/src/checker/inference.rs` |
| record-row application inference | `phalcom-semantic/src/checker/row_inference.rs`, row solver types |
| canonical enum/variant declaration products | `phalcom-semantic/src/enum_semantics.rs` |
| case opening | `phalcom-semantic/src/types/case_instantiation.rs` |
| GADT proof solving | `phalcom-semantic/src/checker/gadt_proof.rs` |
| pattern typing and pattern binding | `phalcom-semantic/src/checker/pattern.rs` |
| exhaustiveness value-space algebra | `phalcom-semantic/src/checker/pattern_space.rs`, `exhaustiveness.rs` |
| match expression branch activation | `phalcom-semantic/src/checker/expression.rs` |
| generic callable application | `phalcom-semantic/src/checker/call.rs` |
| associated / Family specialization | `phalcom-semantic/src/checker/associated.rs`, `expression.rs` |
| checking context / flow / publication | `phalcom-semantic/src/checker/context.rs` |
| callable declaration signatures | `phalcom-semantic/src/checker/declaration_signature.rs` |
| canonical semantic source index | `phalcom-semantic/src/source_index/` |
| incremental fingerprints | `phalcom-semantic/src/db/fingerprint.rs` |
| durable metadata exporter | `phalcom-semantic/src/metadata/export.rs` |
| metadata schema | `phalcom-type-meta` |
| semantic tests | `phalcom-semantic/tests/semantic/` |
| core typing-integration regressions | `phalcom-core/tests/core/typing_integration/` |

## 2.3 Important repository facts discovered during planning

### Existing rigid inference atom — reuse it

`phalcom-semantic/src/checker/inference.rs::InferenceTerm` already contains:

```rust
InferenceTerm::Rigid(RigidTypeVariableId)
```

and documents the intended law:

```text
flexible variables may solve to a rigid term;
the rigid itself may not be assigned by ordinary inference.
```

`InferenceSession` also already contains `var_terms`, which retains non-canonical solver-local term assignments when canonical `TypeId` materialization is impossible.

**Planning consequence:** do **not** add another rigid-inference representation. C5 extends and correctly publishes/routes the existing one.

### Existing call-result relocalization — reuse it

`phalcom-semantic/src/checker/call.rs::local_type_from_arguments` already reconstructs a query-local return view from canonical call results and argument-local mappings captured through `CheckingContext::record_call_local_type`.

**Planning consequence:** do **not** create a second call-result-localization subsystem. C5 repairs argument routing and noncanonical solution handling, then reuses this existing authority where it remains sufficient.

### Existing branch proof product — evolve it

`phalcom-semantic/src/match_semantics.rs::BranchProofEnvironment` already owns:

```rust
bindings: BTreeMap<TypeParameterId, TypeId>
equalities: Box<[TypeConstraint]>
local_bindings: BTreeMap<TypeParameterId, LocalType>
local_equalities: Box<[LocalConstraint]>
```

**Planning consequence:** do not create a parallel durable “GADT proof object.” The new local equality solver operates on/through this proof product and the lexical existential frame derived from it.

### Existing rigid scope topology is present but underused

`RigidArena` already has:

- `fresh_scope(parent)`;
- per-rigid `scope`;
- per-rigid `kind`;
- `scope_contains`;
- `variable_in_scope`.

The missing behavior is operational threading: production `CaseInstantiation::open` sites in pattern resolution currently use `None` rather than the actual lexical parent.

### Existing match-arm local state is narrowly scoped

`CheckingContext::active_local_constraints` is installed around match-arm branch analysis in `checker/expression.rs` and otherwise copied through checker probes.

**Planning consequence:** replacing it with a coherent existential-frame stack is bounded. Do not redesign unrelated flow state.

### Current `LocalType` is structurally incomplete

`types/rigid.rs::LocalType` currently has:

```text
Canonical
Rigid
Applied
ExactCase
Union
Tuple
Record(fields only)
Callable
```

It currently:

- drops `RecordRowTail`;
- treats Family and Lambda as canonical opaque leaves;
- allocates a `BTreeSet` for `free_rigids`;
- performs alpha-equivalence using raw rigid maps without `RigidArena` kind/scope information.

### Duplicate canonical type walkers disagree

Current examples:

- `TypeStore::contains_type_parameter` sees record-row tail parameters but misses Family members and lambda captures;
- `checker/call.rs::type_contains_any_parameter` sees Family members but misses record-row tails;
- `checker/associated.rs::contains_any_type_parameter` sees Family members but misses record-row tails;
- `checker/associated.rs::contains_only_variant_constructor_parameters` likewise walks record fields but not row tails.

**Planning consequence:** C1 establishes one authoritative targeted parameter-occurrence traversal and deletes these competing partial definitions.

### Current GADT local unifier is too rigid

`checker/gadt_proof.rs::unify_local_types` currently rejects:

```text
Rigid(κ) vs Canonical(Int)
```

except for narrow parameter/exact-case cases. This prevents legitimate GADT evidence from establishing `κ ≡ Int`.

### Current proof merge uses raw local equality

`checker/gadt_proof.rs::merge_branch_proofs` compares local bindings with ordinary `LocalType` equality and merges `local_equalities` by raw containment.

**Planning consequence:** alpha-compatible proof merging must use one correlated renaming session for the complete proof.

### Current nested pattern local typing is post-hoc

`checker/pattern.rs::attach_local_type_to_bindings` assigns a variant field's entire local payload type to every descendant binding introduced while resolving the child pattern.

**Planning consequence:** remove this mechanism; recursively propagate an exact local expected type alongside the canonical expected type.

### Current payload storage can shift field correspondence

`CaseInstantiation::open` constructs `payload_types` through `filter_map`, then callers address them by original field index.

**Planning consequence:** replace with one identity-preserving entry per declared `VariantFieldId`.

### Construction-side GADT checks run before local generic inference

Both:

- `checker/expression.rs::synthesize_associated_invoke`; and
- `checker/associated.rs::specialize_associated_member`

perform owner/GADT compatibility through mutual canonical subtype checks against `VariantInfo.case_environment` before variant-local generic inference can solve constructor binders.

**Planning consequence:** C6 moves those equations into the ordinary application constraint session.

### Index-setter canonical signature is inconsistent

`checker/declaration_signature.rs::semantic_signature_for_syntax`, index-set branch, currently uses:

```rust
let result = put_semantic.declared_type.clone();
```

while property setters already synthesize canonical `Unit` and assignment expressions return `Unit`.

**Planning consequence:** C7 makes index-setter declaration and fallback call signatures return `Unit`.

### Source index currently skips new syntax surfaces

`source_index/builder.rs::TypeReferenceTargetCollector`:

- extends generic bound names for methods;
- does not do the equivalent for generic getters/setters/indexes;
- skips `Statement::Enum`;
- skips class-member variants.

`SourceScopeBuilder::visit_statements` also skips `Statement::Enum`.

**Planning consequence:** C7 adds source-index parity without inventing an LSP-specific semantic resolver.

### Durable metadata has no enum semantic input

`MetadataExporter` currently accepts canonical:

- `TypeStore`;
- declarations;
- aliases;
- callable signatures;
- field signatures;

but not `EnumSemanticTable`.

`VariantInfo` is the canonical owner of variant-constructor `GenericSignature`.

`phalcom-type-meta::SemanticMetadataBundle` has declarations, callables, fields, generic signatures, etc., but no dedicated enum/variant record surface at this revision.

**Planning consequence:** D-19 requires a real cross-crate metadata schema/export extension in C8.

---

# 3. Architectural Source-of-Truth Register

| Semantic fact | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| canonical type form | `TypeStore` / `TypeData` | checker, metadata, inference conversion | duplicated partial type walkers |
| generic binder identity | `TypeParameterId` + `TypeParameterOwner` | inference, metadata, diagnostics | binder name strings |
| variant identity | `VariantId` | `VariantInfo`, patterns, source index, invocation | selector/name alone |
| variant field identity | `VariantFieldId` | payload semantic products, patterns | compacted array position after recovery |
| variant constructor generic contract | `VariantInfo::constructor.generic_signature` | direct construction, Family view, metadata | reconstructed generic signature from Family type |
| GADT declaration index equations | `VariantInfo::case_environment` | introduction constraints, elimination proof | mutual-subtype preflight |
| one elimination opening | `CaseInstantiation` + `RigidArena` | local pattern typing | canonical `TypeStore` |
| raw existential witness identity | `RigidTypeVariableId` within one `RigidArena` | local bindings/proofs | alpha-normalized identity |
| facts proven about witnesses | `BranchProofEnvironment` / active local proof | local relation, publication, diagnostics | mutating the rigid into a metavariable |
| generic application inference | `InferenceSession` | calls, constructors, accessors | feature-specific mini-solvers |
| lexical existential lifetime | explicit checker existential frame + `RigidArena` parent relation | escape/publication | “contains any rigid” |
| durable exported generics | canonical semantic metadata exporter | tooling/reflection/downstream | query-local `LocalType` / rigid IDs |
| source occurrence identity | compiler-owned source index | editor/LSP consumers | LSP-only resolver |

---

# 4. Tempting Wrong Fixes — Explicitly Forbidden

The implementing agent must not use any of these shortcuts.

1. **Do not turn rigids into inference variables.**  
   A GADT proof may establish `κ ≡ Int`; ordinary inference may not assign `κ := Int`.

2. **Do not simply replace `left_type != right_type` with one-off `alpha_equivalent` calls.**  
   Proof compatibility needs one correlated alpha-renaming across the entire proof.

3. **Do not keep `CaseInstantiation::open(..., None)` everywhere and special-case escape afterward.**  
   Actual lexical scope topology is part of the theory.

4. **Do not fix nested pattern typing by another bulk rewrite of descendant bindings.**  
   The local expected type must recursively decompose with the pattern.

5. **Do not close an open record row during local conversion.**

6. **Do not treat Family or type-lambda forms as opaque if they can contain local existential captures.**

7. **Do not repair construction-side GADT ordering by merely moving the same mutual-subtyping check later.**  
   The GADT case equation belongs in equality constraint solving.

8. **Do not add a second constructor/Families generic solver.**

9. **Do not reject all generic call arguments containing a live rigid.**  
   Local polymorphic consumption is legal inside the rigid's scope.

10. **Do not “fix” failures by weakening to `Dynamic` or `Unknown`.**

11. **Do not create a second source-index/LSP generic-binder resolver.**

12. **Do not export `RigidTypeVariableId`, `RigidScopeId`, `LocalType`, inference variables, or branch proof state.**

13. **Do not introduce `TypeData::Rigid` or `TypeParameterOwner::Variant`.**

14. **Do not keep an index-setter callable return of the put-value type.**  
    The ratified return is `Unit`.

15. **Do not add speculative persistent caches while repairing correctness.**

---

# 5. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 — Lock the incident | 1–3 | Exact baseline, hostile reproductions, and state-file incident are established before repair | exact hostile regressions reproduce; baseline focused suites recorded; negative architecture searches | full semantic/workspace gates |
| C1 — Complete local structural algebra | 4–7 | Local existential representation and canonical parameter traversal are total over current relevant type/kind algebra | row/lambda/Family structural tests; structural-completeness tests; affected crate check | GADT proof behavior |
| C2 — Establish evidence-sensitive local equality | 8–11 | GADT evidence can prove facts about immutable rigids; alpha/proof comparison is binder-correct | concrete revelation; no-guess hostile case; alpha/correlation/kind laws | pattern recursion, scope exit |
| C3 — Make pattern elimination structurally exact | 12–15 | Case payload identity, recursive local pattern typing, or-pattern join, and pattern-space algebra are correct | nested bindings; payload recovery; redundancy/algebra tests | branch publication |
| C4 — Make existential scope and escape lexical | 16–19 | Nested openings have real parent scopes; publication is proof-normalized and scope-relative | outer/inner scope tests; proof-discharge/widen/escape matrix | generic call/constructor composition |
| C5 — Compose local existentials with ordinary generic calls | 20–24 | Existing inference machinery consumes local rigid terms without treating local calls as escape | local `id(κ)`, structural local term, row-rigid cases, no rigid guessing | variant construction GADT equations |
| C6 — Integrate constructor GADT equations and Family specialization | 25–28 | Direct and captured-Family construction solve fixed receiver, callable generics, and GADT equality in one application proof | explicit applied constructor; captured Family; ground contradiction; residual domain tests | tooling/metadata |
| C7 — Close accessor and source-index parity | 29–32 | Index-setter type is `Unit`; source indexing understands SC-4.8 generic accessor/variant syntax | canonical signature tests; source reference/definition hostility | durable metadata |
| C8 — Publish canonical variant generics durably and normalize incrementality | 33–36 | variant generic metadata has a stable schema/export path; local proof comparison is allocation-independent | schema validation; metadata parity; cold/incremental semantic equality | broad delivery gates |
| C9 — Remove avoidable work and certify the repaired abstraction | 37–40 | duplicate/obsolete mechanisms are deleted, allocation cleanup is complete, focused semantic certification is green | property-law suite; full semantic; Monad/Either; workspace check; negative searches | repository-wide fmt/clippy/workspace test final gates |

---

# 6. Program-Level Drift Protocol

Before every checkpoint:

1. confirm the primary files named by that checkpoint still exist;
2. confirm primary symbols still own the behavior described here;
3. inspect only changes made by earlier checkpoints that affect the next API;
4. search new callers if a public/internal signature changed;
5. adapt mechanics to repository drift, but do not silently change the ratified semantics.

Escalate as **PLAN DRIFT** if:

- `LocalType` ownership moved out of `types/rigid.rs`;
- generic application no longer routes through `apply_generic_callable_in_context`;
- `BranchProofEnvironment` is replaced by another canonical branch-proof product;
- variant GADT equations no longer live in `VariantInfo::case_environment`;
- source-index ownership moved from `phalcom-semantic`;
- metadata schema ownership moved from `phalcom-type-meta`.

---

# 7. Checkpoint C0 — Lock the SC-4.8 Composition Incident

Tasks:
- Task 1 — Record exact baseline and remediation state
- Task 2 — Add core hostile GADT/existential regressions
- Task 3 — Add structural/accessor/tooling regression anchors

## Why this is a checkpoint

The known defects span several subsystems. Before modifying shared proof and type algebra, the repository must contain focused tests that distinguish the correct theory from plausible wrong fixes.

C0 does not attempt to make those new tests green. It establishes the exact defect ledger and protects already-correct architecture.

## Entry conditions

- remote baseline is still `e932aac4...`, or local drift is explicitly documented;
- existing SC-4.8 semantic suites are available;
- companion technical specification is accepted;
- all ratified decisions D-01 through D-22 remain in force.

## Working set

Primary:
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`
- `phalcom-semantic/tests/semantic/adts/existentials.rs`
- `phalcom-semantic/tests/semantic/adts/generics.rs`
- `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs`
- `phalcom-semantic/tests/semantic/capabilities/index_generics.rs`
- `phalcom-semantic/tests/semantic/integration/source_index.rs`
- `phalcom-semantic/tests/semantic/integration/metadata.rs`

Secondary:
- `phalcom-semantic/tests/semantic/adts/matching/mod.rs`
- existing test support/Fixture APIs only as required

Out of scope:
- production semantic edits;
- parser changes;
- VM/runtime changes;
- broad workspace cleanup.

## Semantic contract established by this checkpoint

- the remediation starts from a reproducible, explicit incident;
- tests encode the distinction between rigid guessing and evidence-proven equality;
- tests encode construction-side equation ordering;
- tests encode local generic use, structural preservation, payload identity, index-setter `Unit`, source-index parity, and metadata requirements;
- existing architecture baselines remain separately recorded.

## Semantic risks

- writing positive-only regressions that still permit rigid guessing;
- writing tests that accidentally depend on unsupported syntax rather than the semantic defect;
- masking current failures as baseline noise;
- overwriting historical implementation-state evidence.

## Hostile cases

- `Expr<List<Int>>` must reveal `κ ≡ Int`, while unrelated `κ -> Int` coercion must still fail;
- applied constructor equation involving variant-local generic must not pre-reject;
- local `id(κ)` must not count as escape;
- unresolved payload field must not shift a later field;
- source generic binder must not resolve to a same-spelled nominal declaration.

## Required evidence

1. Run the existing focused suites before adding new tests and record their actual baseline result.
2. Run each newly added regression individually and classify expected current failures as the known product defect.
3. `git diff --check` — proves test/state edits are mechanically clean.
4. Negative searches listed in Task 1 — records architectural baseline before remediation.

## Do not run yet

```bash
cargo +stable test --workspace --all-targets
cargo +stable clippy --workspace --all-targets -- -D warnings
cargo +stable fmt --all -- --check
```

These are delivery gates; they provide no extra diagnosis at C0.

## Escalate immediately if

- an expected known defect already passes because local repository changes have repaired it;
- an existing previously-green focused suite fails before any production edit;
- the fixture cannot express a ratified hostile case with current syntax;
- a test failure occurs below semantic analysis in parser/runtime harness unexpectedly.

## Checkpoint completion

- [ ] baseline revision/local drift recorded
- [ ] implementation state marks SC-4.8 composition hardening as active
- [ ] hostile tests added
- [ ] current failures classified
- [ ] no production semantic code changed
- [ ] exact next failing assertions recorded
- [ ] C0 marked `COMPLETE — INCIDENT REPRODUCED`

---

## Task 1 — Record exact baseline and remediation state

Purpose:
Create a durable resume point and prevent the old “SC-4.8 COMPLETE” wording from being mistaken for current semantic ratification.

Risk:
- Semantic: LOW
- Implementation fanout: local documentation

Owned files and symbols:
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md` — authoritative implementation evidence ledger.

Inspect before editing:
- current C10/SC-4.8 completion section;
- existing workspace fmt/clippy/test baseline notes;
- previous evidence ledger.

Do not inspect unless evidence forces expansion:
- old SC-1/SC-2 plans;
- runtime;
- parser.

Source of truth:
- actual commands executed by the implementing agent;
- exact Git revision.

Implementation boundary:

Changes:
- append a new **SC-4.8 composition-boundary hardening** section;
- retain historical evidence;
- mark the new semantic incident explicitly;
- record local HEAD and working-tree state;
- record that workspace release-complete status is separate.

Must not:
- rewrite earlier PASS evidence as though it never occurred;
- claim local clean state without checking;
- convert unrelated workspace baseline issues into SC-4.8 failures.

Edit operations:

1. OPEN `typing-integration-implementation-state.md`.
2. FIND the current C10 / SC-4.8 completion summary.
3. APPEND a remediation section with:
   - baseline commit;
   - root causes R0–R15;
   - status `SC-4.8 existential/GADT composition closure: INCIDENT`;
   - current checkpoint `C0`;
   - deferred global gates.
4. Run and record:
   ```bash
   git rev-parse HEAD
   git branch --show-current
   git status --short
   ```
5. Run baseline searches:
   ```bash
   rg -n 'TypeData::Rigid|GadtSkolem|TypeParameterOwner::Variant|merge_constructor_generic_signatures' phalcom-semantic
   rg -n 'attach_local_type_to_bindings|active_local_constraints' phalcom-semantic/src
   rg -n 'AssociatedGadtOwnerConflict' phalcom-semantic/src/checker
   ```
6. Record results as baseline, not deletion gates yet.

Testing classification:
- no behavioral test;
- state/evidence task.

Checkpoint state update:
Record the exact local revision, any worktree drift, and the fact that C0 hostile regressions are expected to expose product defects.

---

## Task 2 — Add core hostile GADT/existential regressions

Purpose:
Lock the theory before changing `gadt_proof`, pattern typing, scope, or generic application.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only multi-file

Owned files and symbols:
- `tests/semantic/adts/matching/gadt_refinement.rs`
- `tests/semantic/adts/existentials.rs`
- `tests/semantic/adts/generics.rs`
- optionally new `tests/semantic/adts/matching/existential_equality.rs` if the existing file becomes unwieldy.

Source of truth:
- current semantic products: `MatchResolution`, `PatternResolution`, `BranchProofEnvironment`, `PatternBindingResolution`.

Changes:
Add the smallest tests that prove:

1. **Concrete revelation**
   ```phalcom
   Wrap<U>(_ value: U) -> Expr<List<U>>
   Expr<List<Int>>
   ```
   branch reachable; binding retains one local rigid; branch evidence proves/effectively permits `Int`.

2. **No guessing**
   A branch-local rigid without corresponding GADT equality still cannot satisfy an `Int` sink.

3. **Local generic consumption**
   `id<T>(x)` where `x : κ` remains valid within branch.

4. **Applied construction**
   `Expr<List<Int>>::Wrap(1)` succeeds.

5. **Captured Family construction**
   capture `Expr<List<Int>>::Wrap::*`, invoke with `1`, succeeds.

6. **Nested local leaf**
   `(κ, Int)` destructuring gives leaf `κ`, not tuple local type.

7. **Outer/inner scope**
   an outer existential remains usable after an inner match; an inner existential does not escape.

Testing classification:
- focused regressions required now because they define the central semantic repair.

Required current-state command examples:

```bash
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::matching::gadt_refinement -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::existentials -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::generics -- --nocapture
```

What they prove at C0:
- existing baselines remain intact;
- new hostile cases expose precisely the target defects.

They do **not** prove:
- the remediation is implemented.

---

## Task 3 — Add structural/accessor/tooling regression anchors

Purpose:
Prevent later shared-structure refactors from overlooking row tails, lambda captures, source indexing, metadata, and index-setter signature products.

Risk:
- Semantic: MEDIUM
- Implementation fanout: test-only multi-file/cross-crate schema preparation

Owned files:
- `tests/semantic/capabilities/index_generics.rs`
- `tests/semantic/integration/source_index.rs`
- `tests/semantic/integration/metadata.rs`
- appropriate row/type-lambda foundation tests
- `phalcom-type-meta/tests/schema_compat.rs` only after C8 schema work; at C0 record the missing surface rather than inventing a failing schema construction.

Add/prepare tests for:

- open row tail survives localize/materialize;
- row-tail-only generic remains discoverable;
- local type lambda free capture is visible to rigid traversal;
- Family member containing a local rigid is visible;
- payload field identity cannot shift;
- canonical index-setter signature return is `Unit`;
- generic getter/setter/index type-reference collector honors callable-local binders/where clauses;
- enum/variant type references are visited;
- metadata exporter rejects local rigid products and later must export canonical variant generics.

Testing classification:
- ownership-layer tests, not broad integration duplication.

---

# 8. Checkpoint C1 — Complete the Local Structural Algebra

Tasks:
- Task 4 — Make `LocalType` total over rows, Families, and lambda captures
- Task 5 — Introduce non-allocating local rigid traversal
- Task 6 — Centralize canonical generic-parameter occurrence traversal
- Task 7 — Audit and update all `LocalType` structural consumers

## Why this is a checkpoint

Equality, pattern typing, escape, and generic application cannot be repaired safely while the local representation itself loses structure. C1 creates a faithful local type algebra first.

## Entry conditions

- C0 COMPLETE;
- hostile structural tests exist;
- no production repair has begun.

## Working set

Primary:
- `phalcom-semantic/src/types/rigid.rs`
- `phalcom-semantic/src/types/store.rs`
- `phalcom-semantic/src/types/type_lambda.rs`
- `phalcom-semantic/src/types/family.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/associated.rs`

Secondary:
- `checker/statement.rs`
- `checker/expression.rs`
- `checker/context.rs`
- `metadata/export.rs`
- `types/substitution.rs`

Out of scope:
- GADT proof semantics;
- pattern recursion;
- match scope activation;
- constructor equation ordering.

## Semantic contract established by this checkpoint

- no supported row tail, Family member, or type-lambda free capture can hide a local existential;
- local rigid traversal is kind/structure complete;
- generic-parameter occurrence queries have one canonical authority;
- new `TypeData` forms cannot silently be omitted from critical occurrence walkers.

## Semantic risks

- treating `RecordRow` as a proper `Type`;
- changing canonical lambda bound-variable semantics while localizing only free captures;
- accidental recursive cycles in Family/lambda traversal;
- changing substitution semantics while only occurrence traversal was intended.

## Hostile cases

- `#{x: Int | R}` remains open after a rigid-free round trip;
- `R : RecordRow` is not represented as `TypeData::Parameter`;
- a lambda free capture of constructor-local `U` is found;
- a Family member can contain `κ`;
- a generic used only in a row tail is still detected.

## Required evidence

```bash
cargo +stable check -p phalcom-semantic --all-targets
```

Proves structural API/caller migration compiles.

Focused structural tests prove semantic preservation.

Negative search:
```bash
rg -n 'fn type_contains_any_parameter|pub fn contains_any_type_parameter' \
  phalcom-semantic/src/checker phalcom-semantic/src/types
```

Expected after C1:
- only the chosen authoritative TypeStore/type traversal API plus justified compatibility wrappers, if any.

## Do not run yet

- full GADT matching suite beyond a smoke regression;
- full workspace tests.

## Escalate immediately if

- implementing local lambda captures requires changing parser/type-lambda binding representation;
- Family localization appears to require runtime Family changes;
- canonical row solver requires a new persistent row identity.

## Checkpoint completion

- [ ] tasks complete
- [ ] structural tests pass
- [ ] crate check passes
- [ ] duplicate walkers removed/delegated
- [ ] structural-completeness table recorded
- [ ] no active C1 incident

---

## Task 4 — Make `LocalType` total over rows, Families, and lambda captures

Purpose:
Turn `LocalType` from a partial structural view into a semantics-preserving local existential view for all currently reachable parameter-capturing forms.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `types/rigid.rs` — `LocalType`, `LocalRecordField`, `from_canonical`, `from_canonical_types`, `materialize`, alpha/traversal match arms.
- `types/type_lambda.rs` — free-capture inspection/materialization helpers only if required.
- `types/family.rs` — canonical Family shape, read-only reference.

Source of truth:
- canonical `TypeData`;
- canonical `RecordRowData`;
- canonical `TypeLambdaArena` scoped body;
- canonical Family members.

Current implementation:
- `LocalType::Record` stores only fields;
- record materialization forces `RecordRowTail::Closed`;
- Lambda/Family fall through to `Canonical`.

Target implementation:

STRUCTURAL:

```rust
pub enum LocalRecordRowTail {
    Closed,
    Parameter(TypeParameterId),
    Rigid(RigidTypeVariableId),
}

pub enum LocalType {
    Canonical(TypeId),
    Rigid(RigidTypeVariableId),
    Applied { ... },
    ExactCase { ... },
    Union(...),
    Tuple(...),
    Record {
        fields: Box<[LocalRecordField]>,
        tail: LocalRecordRowTail,
    },
    Callable(...),

    // Prefer an explicit structural Family because the repository already
    // represents Family members structurally in InferenceTerm.
    Family(Box<[LocalFamilyMember]>),

    // Preserve canonical lambda binder identity; localize only free captures.
    Lambda {
        lambda: TypeLambdaId,
        captures: Box<[LocalLambdaCapture]>,
    },
}
```

Exact field names are STRUCTURAL, not paste-ready.

For lambda capture entries, prefer a deterministic canonical-free-type key:

```text
captured canonical free TypeId
→ localized LocalType
```

Only store entries whose local view differs from the canonical free type, unless a full capture list simplifies equality.

Edit operations:

1. OPEN `types/rigid.rs`.
2. FIND `LocalType` and all exhaustive matches in the file.
3. CHANGE Record representation to retain tail.
4. ADD Family local member form carrying operation/member kind/local member type.
5. ADD Lambda overlay keyed by canonical free captures.
6. UPDATE `from_canonical`:
   - Record fields + tail;
   - Family members recursively;
   - Lambda: call `TypeLambdaArena::collect_free_types`, localize free `TypeId`s under replacements, retain changed captures.
7. UPDATE `from_canonical_types` equivalently.
8. UPDATE `materialize`:
   - stable row tail remains open;
   - rigid row tail returns `Rigid` materialization failure;
   - Family recursively materializes members;
   - Lambda materializes capture overlay and rebuilds/substitutes free captures without altering bound de Bruijn structure.
9. ADD debug assertions that row-tail rigid kind is `RecordRow` where an arena is available; hard kind validation occurs in consumers/equality.
10. SEARCH all `LocalType::Record` pattern matches and migrate.
11. ADD round-trip tests.

Must not:
- turn row tail into a fake proper type;
- rebuild lambda bound variables manually if the canonical arena can preserve them;
- publish localized lambda captures to canonical store while any rigid remains.

Testing classification:
- focused foundation tests required at C1.

---

## Task 5 — Introduce non-allocating local rigid traversal

Purpose:
Make boolean rigid queries fast and create one structural visitor authority for local forms.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local/multi-file caller cleanup

Owned symbols:
- `LocalType::free_rigids`
- `LocalType::contains_rigid_from_scope`
- escape and metadata callers

Target API shape:

STRUCTURAL:

```rust
impl LocalType {
    pub fn has_free_rigid(&self) -> bool;
    pub fn for_each_free_rigid(&self, f: impl FnMut(RigidTypeVariableId));
    pub fn contains_rigid_from_scope(
        &self,
        arena: &RigidArena,
        leaving: RigidScopeId,
    ) -> bool;
    pub fn free_rigids(&self) -> BTreeSet<RigidTypeVariableId>; // keep only if tests/diagnostics need collection
}
```

`has_free_rigid` and `contains_rigid_from_scope` MUST short-circuit without allocating a `BTreeSet`.

Edit operations:
1. implement one recursive traversal covering every LocalType variant;
2. implement collection as a consumer of that traversal;
3. migrate boolean callers;
4. preserve collection callers only for diagnostics/tests.

Testing classification:
- no dedicated behavior test beyond C1 structural traversal tests;
- performance property is inspected through code/bench later.

---

## Task 6 — Centralize canonical generic-parameter occurrence traversal

Purpose:
Delete divergent notions of “type contains parameter.”

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:
- `types/store.rs` — authoritative targeted traversal
- `checker/call.rs`
- `checker/associated.rs`
- `checker/context.rs` specialization generic-retention logic

Source of truth:
- `TypeStore` canonical structural graph.

Target API:

EXACT in responsibility, STRUCTURAL in final naming:

```rust
TypeStore::contains_type_parameter(ty, parameter)
TypeStore::contains_any_type_parameter(ty)
TypeStore::contains_any_type_parameter_from(ty, &[TypeParameterId])
TypeStore::all_type_parameters_satisfy(ty, predicate)
```

Do not necessarily implement all four if two generic visitor helpers can express them cleanly.

Required traversal coverage:
- Applied origin + arguments;
- ExactCase enum type;
- Union;
- Tuple;
- Record fields;
- `RecordRowTail::Parameter`;
- Callable params/return;
- Family member types;
- Lambda canonical free captures;
- Parameter;
- leaves explicitly classified.

Edit operations:
1. strengthen existing `TypeStore::contains_type_parameter`;
2. add set/predicate form(s);
3. remove/delegate:
   - `call.rs::type_contains_any_parameter`;
   - `associated.rs::contains_any_type_parameter`;
   - recursive core of `contains_only_variant_constructor_parameters`;
4. migrate all consumers.
5. use exhaustive matches; avoid `_ => false` where a future composite form could be skipped.

Required hostile test:
```text
class Schema<R: RecordRow> {
    @class shape -> #{ id: Int | R }
}
Schema.shape
```
must remain underconstrained on raw owner.

---

## Task 7 — Audit and update every LocalType consumer

Purpose:
Prevent newly complete LocalType forms from being silently ignored outside `rigid.rs`.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Inspect:
```bash
rg -n 'LocalType::|LocalType\b' phalcom-semantic/src
```

Known consumers include:
- `checker/gadt_proof.rs`
- `checker/context.rs`
- `checker/statement.rs`
- `checker/call.rs`
- `checker/expression.rs`
- `types/case_instantiation.rs`
- metadata local-type rejection tests/export helper.

Changes:
- migrate structural pattern matches;
- ensure statement-level tuple/list local decomposition works with new representation;
- do not “handle” new variants with blanket `_ => None` where they are semantically decomposable;
- add a structural-completeness test/table that fails or requires update when a new `TypeData` variant appears.

Testing classification:
- checkpoint-level focused tests.

---

# 9. Checkpoint C2 — Evidence-Sensitive GADT Equality and Alpha Proof Semantics

Tasks:
- Task 8 — Make local GADT equality an authoritative proof operation
- Task 9 — Add proof-aware local normalization
- Task 10 — Replace raw-rigid proof merge with shared correlated alpha compatibility
- Task 11 — Add binder-normalized local semantic comparison

## Why this is a checkpoint

This checkpoint establishes the mathematical core required by every later composition repair.

## Entry conditions

- C1 COMPLETE;
- LocalType can represent every tested structure without erasure.

## Working set

Primary:
- `checker/gadt_proof.rs`
- `match_semantics.rs`
- `types/rigid.rs`
- `checker/pattern_space.rs` only for compile adaptation; semantic use in C3.

Secondary:
- `types/case_environment.rs`
- `types/relation.rs` for comparison only, not redesign.

Out of scope:
- generic call inference;
- match scope activation;
- constructor introduction.

## Semantic contract

- a rigid remains an immutable witness;
- GADT evidence may establish `κ ≡ T`;
- proof normalization uses those equalities;
- alpha comparison is kind/scope/correlation aware;
- proof merge is independent of allocator IDs.

## Hostile cases

- `κ ≡ Int` from GADT evidence is accepted;
- unrelated `κ` is not guessed to Int;
- `Pair<κ,κ>` is not alpha-equivalent to `Pair<κ1,κ2>`;
- Type and RecordRow rigids cannot alpha-map;
- independently opened semantically identical proofs merge.

## Required evidence

Focused:
```bash
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::matching::gadt_refinement -- --nocapture
```

Plus new equality/proof modules.

Do not run full workspace.

---

## Task 8 — Make local GADT equality an authoritative proof operation

Purpose:
Repair the distinction between “not assignable by ordinary inference” and “provably equal from GADT evidence.”

Risk:
- Semantic: HIGH
- Implementation fanout: local core algorithm + consumers

Owned symbols:
- `solve_local_case_proof`
- `unify_local_types`
- `LocalConstraint::Equivalent`
- `BranchProofEnvironment::local_equalities`

Source of truth:
- GADT observation itself plus canonical case environment;
- rigid identity remains `RigidArena`.

Current implementation:
`unify_local_types` treats concrete canonical types as incompatible with a rigid in ordinary result observation.

Target implementation:

STRUCTURAL:

Introduce a bounded local equality solver in `checker/gadt_proof.rs`, for example:

```rust
struct LocalEqualitySolver<'a> {
    store: &'a TypeStore,
    arena: &'a RigidArena,
    // rigid equivalence / representative and optional proven term
    ...
}
```

Responsibilities:
- decompose equal structural constructors;
- validate rigid kind;
- establish rigid↔rigid equality;
- establish rigid↔ground/local-term equality;
- reject contradictions;
- preserve occurs/correlation rules;
- emit a normalized set of `LocalConstraint::Equivalent` facts into branch proof.

Do not make this a general purpose inference solver.

Edit operations:
1. preserve canonical `solve_gadt_branch_proof`;
2. replace the local equality path only;
3. remove `exact_case_observation` as an ad-hoc authority if the new evidence solver subsumes it; otherwise narrow and document it;
4. make `solve_local_case_proof` return/refine authoritative local equalities;
5. add `EQ-01` concrete revelation;
6. keep hostile no-guess test.

EXACT semantic law:

```text
ordinary inference:
    κ ?= Int
    cannot assign κ

GADT equality observation:
    List<κ> ≡ List<Int>
    may prove κ ≡ Int
```

---

## Task 9 — Add proof-aware local normalization

Purpose:
Allow downstream consumers to ask what a local type means under current branch evidence without destroying raw witness identity.

Risk:
- Semantic: HIGH
- Implementation fanout: local API, used later by context/call

Source of truth:
- raw `LocalType`;
- `BranchProofEnvironment.local_equalities`;
- local proof solver.

Target API shape:

STRUCTURAL:

```rust
pub(crate) fn normalize_local_type(
    store: &TypeStore,
    arena: &RigidArena,
    proof: &BranchProofEnvironment,
    ty: &LocalType,
) -> Result<LocalType, LocalProofContradiction>
```

or a `LocalProofView` object exposing:

```rust
proof.normalize(ty)
proof.equivalent(left, right)
proof.entails_subtype(...)
```

Requirements:
- preserve unresolved rigid identity;
- substitute only proven equalities;
- terminate on cycles;
- retain kind;
- recursively normalize containers;
- allow `κinner ≡ κouter`.

Must not:
- mutate binding-local raw types;
- canonicalize a rigid without evidence.

Testing:
- direct normalization unit tests plus GADT integration.

---

## Task 10 — Replace raw-rigid proof merge with shared correlated alpha compatibility

Purpose:
Make independently fresh openings compare by semantics rather than allocator identity.

Risk:
- Semantic: HIGH
- Implementation fanout: proof merge / exhaustiveness

Owned symbols:
- `LocalType::alpha_equivalent`
- `merge_branch_proofs`
- `BranchProofEnvironment.local_bindings/local_equalities`

Source of truth:
- rigid kind/scope topology from `RigidArena`;
- one comparison-wide alpha-renaming.

Target:

STRUCTURAL:

```rust
pub struct AlphaRenaming { ... }

impl AlphaRenaming {
    fn compare_local(..., left_arena, right_arena, left, right) -> bool;
}
```

If both proofs are in one arena, pass the same arena for both sides.

Requirements:
- bijection;
- kind match;
- relative scope topology match;
- same mapping reused across every binding/equality in proof;
- raw ID ignored;
- origin metadata ignored for mathematical alpha identity.

Edit operations:
1. stop allocating fresh mapping per individual LocalType comparison in proof merge;
2. add comparison method accepting reusable context;
3. migrate `merge_branch_proofs`;
4. normalize equality order where needed so equivalent sets compare independent of insertion order;
5. add correlation/kind/topology laws.

Forbidden:
```rust
if left_type.alpha_equivalent(right_type) { ... }
```
inside each independent map entry with a new mapping each time.

---

## Task 11 — Add binder-normalized local semantic comparison

Purpose:
Provide deterministic semantic comparison for cold/incremental parity and tests.

Risk:
- Semantic: MEDIUM/HIGH
- Implementation fanout: local + incremental tests

Target:
Introduce a deterministic test/comparison form that renumbers:

```text
scope/binder by traversal order and lexical parent topology
```

while preserving kind and correlation.

Do not store it in `TypeStore`.

Possible API:
```rust
BinderNormalizedLocalForm::from_type(...)
BinderNormalizedProof::from_branch(...)
```

Testing classification:
- foundation tests now;
- incremental use in C8.

---

# 10. Checkpoint C3 — Structurally Exact Pattern Elimination

Tasks:
- Task 12 — Preserve payload field identity in `CaseInstantiation`
- Task 13 — Thread local expected types recursively through patterns
- Task 14 — Implement conservative or-pattern local joins
- Task 15 — Repair pattern-space proof algebra

## Why this is a checkpoint

C2 fixes proof semantics. C3 makes pattern structure consume those semantics without losing or inventing witness identity.

## Working set

Primary:
- `types/case_instantiation.rs`
- `checker/pattern.rs`
- `checker/gadt_proof.rs`
- `checker/pattern_space.rs`
- matching tests

Out of scope:
- match branch publication/escape;
- generic application.

## Semantic contract

- every pattern leaf gets its exact local structural type;
- payload recovery never shifts field correspondence;
- or-pattern alternatives do not invent shared witness identity;
- pattern-space operations are alpha-stable.

## Required evidence

```bash
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::matching -- --nocapture
```

If this filter is too broad, run new nested/proof/pattern-space modules individually first, then existing matching module.

Negative:
```bash
rg -n 'attach_local_type_to_bindings' phalcom-semantic/src
```
Expected: zero production hits.

---

## Task 12 — Preserve payload field identity in `CaseInstantiation`

Purpose:
Eliminate `filter_map` cardinality/index corruption and repeated field rescans.

Risk:
- Semantic: HIGH
- Implementation fanout: local multi-file

Owned symbols:
- `CaseInstantiation::open`
- `CaseInstantiation::payload_type`
- variant-field consumers in `pattern.rs`

Source of truth:
- `VariantFieldSemantic::id`.

Target representation:

STRUCTURAL, strongly recommended:

```rust
pub struct LocalPayloadField {
    pub field: VariantFieldId,
    pub ty: Option<LocalType>,
}

pub struct CaseInstantiation {
    ...
    pub payload_fields: Box<[LocalPayloadField]>,
}
```

Requirements:
- one entry per `variant.fields` element;
- preserve declaration order;
- preserve exact ID;
- unavailable canonical type becomes `None`;
- no compaction.

Edit operations:
1. replace `payload_types`;
2. remove `filter_map`;
3. add identity lookup or carry matching index/entry through pattern resolution;
4. remove repeated `.position(...)` scan;
5. update tests and debug formatting;
6. reduce repeated `replacements()` allocations where touched.

---

## Task 13 — Thread local expected types recursively through patterns

Purpose:
Fix nested existential binding precision.

Risk:
- Semantic: HIGH
- Implementation fanout: one large recursive function + callers

Owned symbol:
- `resolve_pattern_with_mode`

Change signature structurally from:

```rust
resolve_pattern_with_mode(..., expected_ty: TypeId, ...)
```

to a contract equivalent to:

```rust
resolve_pattern_with_mode(
    ...,
    expected_ty: TypeId,
    local_expected: Option<&LocalType>,
    parent_rigid_scope: Option<RigidScopeId>, // finalized in C4
    ...
)
```

For C3, `parent_rigid_scope` may be prepared but top-level lexical parent wiring completes in C4.

Required local decomposition:
- name: exact local expected;
- tuple: local element;
- list prefix: local element;
- list rest: local list;
- record property: local field;
- variant: `CaseInstantiation` local payload field;
- nested variant: child opening uses current case scope as parent;
- wildcard/literal: no binding;
- or pattern: same incoming local expected independently for each alternative.

Delete:
`attach_local_type_to_bindings`.

Testing:
- nested tuple;
- repeated correlation;
- nested variant;
- deep `((List<T>, Option<T>), ...)`;
- record/list shape.

---

## Task 14 — Implement conservative or-pattern local joins

Purpose:
Prevent alpha-equivalent but distinct alternative witnesses from being treated as one actual witness.

Risk:
- Semantic: HIGH
- Implementation fanout: local pattern join logic

Owned symbols:
- `commit_shared_bindings`
- `joined_local_type` or equivalent helpers

Ratified rule:
Retain `local_type` only when:
- rigid-free; or
- all alternatives refer to genuinely shared live rigid identities.

Do **not** retain the first local type merely because independent alternatives are alpha-equivalent.

Canonical common knowledge may still be joined/preserved.

Also remove avoidable cloning of complete alternative-binding vectors while touching this path; prefer borrowed active alternatives.

---

## Task 15 — Repair pattern-space proof algebra

Purpose:
Make usefulness/redundancy/exhaustiveness independent of fresh rigid IDs.

Risk:
- Semantic: HIGH
- Implementation fanout: local proof consumers

Owned file:
- `checker/pattern_space.rs`

Changes:
- use C2 proof compatibility in `intersect` and `subtract`;
- do not add independent alpha logic here;
- preserve existing variant identity checks.

Required laws:
```text
S ∩ α(S) = S
S \ α(S) = ∅
α(S) \ S = ∅
```

Hostile:
two genuinely distinct existential constraints must remain distinguishable.

---

# 11. Checkpoint C4 — Lexical Existential Scope and Proof-Aware Publication

Tasks:
- Task 16 — Introduce the lexical existential frame stack
- Task 17 — Thread real parent scopes through nested openings
- Task 18 — Replace global rigid escape with proof-aware scope-relative publication
- Task 19 — Audit publication boundaries and improve diagnostics

## Why this is a checkpoint

Correct local terms and equality are insufficient unless the checker knows exactly which existential lifetime ends at each boundary.

## Working set

Primary:
- `checker/context.rs`
- `checker/expression.rs`
- `checker/pattern.rs`
- `types/rigid.rs`
- `tests/semantic/adts/existentials.rs`

Secondary:
- closure/statement checking code where publication guards are called

Out of scope:
- generic call constraint routing;
- constructor introduction equations.

## Semantic contract

- nested scopes form a real parent hierarchy;
- active local proof belongs to the lexical existential frame;
- escape checks only rigids whose lifetime actually ends;
- proof discharge occurs before escape rejection;
- safe bound widening remains legal.

---

## Task 16 — Introduce the lexical existential frame stack

Purpose:
Replace scattered `active_local_constraints` with one lexical owner for scope and local proof.

Risk:
- Semantic: HIGH
- Implementation fanout: context cloning/probes + match branch

Owned symbols:
- `CheckingContext`
- context constructors/clones/probes
- `active_local_constraints`

Target:

STRUCTURAL:

```rust
pub(crate) struct ExistentialFrame {
    pub scope: RigidScopeId,
    pub proof: BranchProofEnvironment, // or a compact active local-proof view
}

existential_frames: Vec<ExistentialFrame>
```

Avoid duplicating canonical `BranchProofEnvironment` content unnecessarily. If storing a borrowed proof is lifetime-hostile, store only the local proof state needed for branch checking.

Provide:
```text
current_existential_scope()
push_existential_frame(...)
pop_existential_frame(...)
active_local_proof_view()
```

Edit all context clone/probe constructors.

Negative gate after migration:
```bash
rg -n 'active_local_constraints' phalcom-semantic/src
```
Expected: zero production hits.

---

## Task 17 — Thread real parent scopes through nested openings

Purpose:
Make `RigidArena` parent topology operational.

Risk:
- Semantic: HIGH
- Implementation fanout: pattern recursion / branch nesting

Rules:
- top-level match inside no existential frame opens with parent `None`;
- match nested inside an outer existential branch uses outer active scope;
- nested variant patterns within one pattern use the enclosing case opening scope;
- siblings do not parent one another.

Update:
- pattern resolver's `CaseInstantiation::open` call;
- recursive nested variant path;
- any exact-case reopening path.

Required direct assertion:
```text
arena.parent(inner_scope) == Some(outer_scope)
```
Expose a read-only `scope_parent` helper on `RigidArena` if not already available.

---

## Task 18 — Replace global rigid escape with proof-aware scope-relative publication

Purpose:
Implement ratified publication algorithm.

Risk:
- Semantic: HIGH
- Implementation fanout: context + boundary callers

Owned symbol:
- `CheckingContext::check_local_type_escape`

Replace its semantic contract with something equivalent to:

```rust
check_local_type_publication(
    local_type,
    leaving_scope,
    expected,
    additional_proof_or_constraints,
    boundary_kind,
    range,
) -> PublicationDecision
```

A boolean return is acceptable only if callers do not need the normalized local/canonical result; prefer a structured result if proof discharge needs to update returned knowledge.

Algorithm:
1. normalize under active local equality proof;
2. determine rigids owned by leaving scope/descendants;
3. if none remain, materialize/publish canonical view when needed;
4. otherwise try sound rigid-free widening;
5. otherwise emit `ExistentialEscape`.

Required cases:
- outer rigid crosses inner boundary;
- inner rigid rejects;
- `κinner ≡ κouter` crosses as outer;
- `κ ≡ Int` publishes as Int;
- `κ <: Object` widens to Object.

Do not use `free_rigids().is_empty()` as the sole gate.

---

## Task 19 — Audit publication boundaries and improve diagnostics

Purpose:
Prove completeness of publication protection and source-semantic diagnostics.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file callers

Inventory and classify:
- explicit/implicit return;
- match/conditional join;
- outer local assignment;
- field write;
- closure capture;
- aggregate construction when the aggregate crosses scope;
- call argument (scope-preserving by default; C5 adjusts);
- call result;
- metadata;
- exact-case observation;
- flow joins.

Add `LocalPublicationBoundary` enum if it reduces caller ambiguity.

Diagnostics:
Use `RigidOrigin::VariantParameter` to report:
- variant;
- source binder name where available through `TypeStore::type_parameter`;
- outward target/boundary;
- normalized outward type.

Do not make raw `κ17` the only user-facing explanation.

---

# 12. Checkpoint C5 — Ordinary Generic Application with Live Existentials

Tasks:
- Task 20 — Add LocalType-to-existing-InferenceTerm lowering
- Task 21 — Treat noncanonical `var_terms` as locally solved application evidence
- Task 22 — Route generic arguments through local terms and stop premature escape rejection
- Task 23 — Publish/relocalize generic results without canonicalizing rigids
- Task 24 — Extend row-domain inference for fixed local row rigids

## Why this is a checkpoint

The repository already has `InferenceTerm::Rigid`, `var_terms`, symbolic inference results, and call-result relocalization. C5 connects those existing pieces correctly.

## Source of truth

- ordinary generic application remains `InferenceSession`;
- local witness identity remains `RigidArena`;
- raw local expression view remains `TypedExpression.local_type`;
- no new mini-solver.

## Hostile cases

- flexible generic variable may solve to `κ`;
- rigid itself may not be solved to concrete type by ordinary inference;
- call succeeds locally but later outward publication still rejects;
- row-kind rigid is not treated as proper-type rigid.

---

## Task 20 — Add LocalType-to-existing-InferenceTerm lowering

Purpose:
Feed actual local argument structure to the already-rigid-aware inference graph.

Risk:
- Semantic: HIGH
- Implementation fanout: inference + call

Owned file:
- `checker/inference.rs`

Add an authoritative conversion:

STRUCTURAL:

```rust
InferenceSession::local_type_to_inference(
    &self,
    local: &LocalType,
    store: &TypeStore,
) -> Result<InferenceTerm, LocalInferenceConversionError>
```

Map:
- Canonical → existing canonical conversion;
- Rigid → existing `InferenceTerm::Rigid`;
- Applied / ExactCase / Union / Tuple / Callable / Record / Family recursively;
- Lambda:
  - retain canonical lambda when captures are rigid-free;
  - if local captures exist, represent through existing symbolic lambda/canonical free-capture mechanism if possible;
  - do not invent a canonical TypeId.

If Lambda cannot be consumed by current generic application without a new symbolic term, escalate rather than silently using stale canonical captures.

---

## Task 21 — Treat noncanonical `var_terms` as locally solved application evidence

Purpose:
Stop a flexible variable successfully unified with `Rigid(κ)` from being reported as underconstrained merely because no canonical `TypeId` substitution exists.

Risk:
- Semantic: HIGH
- Implementation fanout: shared inference outcome consumers

Current behavior:
`InferenceSession::solve` considers a variable solved only when `substitutions` contains canonical `TypeId`; `var_terms` is ignored by final unsolved classification.

Target:
A variable with a stable, non-variable local term in `var_terms` is **locally solved**, though not canonically materialized.

Prefer extending query-local solution APIs rather than forcing `TypeId`.

STRUCTURAL option:

```rust
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
    pub term_substitutions: HashMap<InferVarId, InferenceTerm>,
    ...
}
```

or expose a `resolved_term(var)` API and make `InferenceOutcome` solved classification consult it.

Requirements:
- no rigid enters durable products;
- canonical consumers continue using canonical substitution projection;
- local call publication can inspect resolved terms;
- underconstrained diagnostics include only truly unsolved variables.

Search/update all `InferenceSolution` consumers before choosing the exact representation.

Optional compile checkpoint:
```bash
cargo +stable check -p phalcom-semantic --all-targets
```
Reason: shared inference API fanout is high.

---

## Task 22 — Route generic arguments through local terms and stop premature escape rejection

Purpose:
Make `id(κ)` a normal local generic application.

Risk:
- Semantic: HIGH
- Implementation fanout: call application

Owned symbols:
- `analyze_application_argument`
- `apply_generic_callable_in_context`
- argument constraint construction

Changes:
1. `analyze_application_argument` still records local type in call capture.
2. Remove the rule that any local argument with no canonical `expected.ty()` is immediately an escape.
3. Determine whether the current boundary actually exits the existential scope; ordinary generic argument consumption does not.
4. When argument has `local_type`, build `argument_term` from C5 Task 20 rather than solely from canonical `knowledge.ty()`.
5. Add it to the ordinary `InferenceRelation`.
6. Preserve evidence/support/provenance.
7. Keep concrete non-generic sink checks sound: a call requiring canonical `Int` must still reject unrelated `κ`.

Must not:
- omit ordinary parameter relation checking;
- simply skip all local argument checks.

---

## Task 23 — Publish/relocalize generic results without canonicalizing rigids

Purpose:
Allow a locally solved generic return such as `T = κ` to remain usable inside the branch.

Risk:
- Semantic: HIGH
- Implementation fanout: call return path

Existing authorities to reuse:
- `SymbolicInferenceResult`;
- `local_type_from_arguments`;
- `CallCheckResult.local_type`.

Target:
- canonical `TypeKnowledge` remains the durable/canonical view;
- local result carries `LocalType`;
- if return term resolves to a local noncanonical term, do not call `InferenceSession::materialize` and report an invariant failure;
- publish a local result term/view for `apply_resolved_callable`;
- use existing relocalization when canonical result + captured map is sufficient.

Do not add a second “local call result table.”

Required test:
```text
let y = id(x)    // x: κ
use y locally    // succeeds
return y as Int  // rejects unless proof κ ≡ Int
```

---

## Task 24 — Extend row-domain inference for fixed local row rigids

Purpose:
Honor D-09 for `RecordRow`-kind constructor locals.

Risk:
- Semantic: HIGH
- Implementation fanout: inference + row inference

Owned files:
- `checker/row_inference.rs`
- `checker/inference.rs`
- row solver only if fixed-row witness support cannot remain at the inference-record layer.

Current:
```rust
InferenceRecordTail::{Closed, Parameter, Var}
```

Required:
A fixed local row witness must be representable distinctly from a flexible row solver variable.

Recommended:

```rust
InferenceRecordTail::Rigid(RigidTypeVariableId)
```

with invariant:
```text
rigid.kind == RecordRow
```

Rules:
- row flexible variable may be related/bound to fixed row-rigid evidence where sound;
- fixed row rigid is never assigned by row inference;
- canonical materialization with unresolved row rigid fails locally rather than closing the row;
- escape sees it.

Hostile:
Type-kind rigid must not be accepted in row tail.

---

# 13. Checkpoint C6 — Constructor GADT Equations and Applied Family Specialization

Tasks:
- Task 25 — Add query-local application equality inputs to callable targets
- Task 26 — Remove premature direct variant GADT prechecks
- Task 27 — Repair associated/Family variant specialization and residual generic domains
- Task 28 — Certify direct/Family parity and ground conflicts

## Why this is a checkpoint

Introduction must use the same generic/equality proof authority as ordinary application. Direct and captured-Family construction are distinct consumers of the same canonical variant metadata.

## Working set

Primary:
- `checker/call.rs`
- `checker/expression.rs`
- `checker/associated.rs`
- `enum_semantics.rs` read-only source of truth
- `adts/generics.rs`

Out of scope:
- canonical generic ownership redesign;
- runtime Family dispatch;
- selector identity changes.

## Semantic contract

For:

```text
fixed declaration:
    T = List<Int>

callable argument:
    U = Int

GADT case equation:
    T ≡ List<U>
```

one application proof succeeds.

A fully ground contradiction fails.

---

## Task 25 — Add query-local application equality inputs to callable targets

Purpose:
Represent GADT case equations without merging owners or pre-checking them with subtyping.

Risk:
- Semantic: HIGH
- Implementation fanout: call target construction + generic solver

Source of truth:
`VariantInfo::case_environment.bindings`.

Add a query-local field to `CallableApplicationTarget`, e.g.:

STRUCTURAL:

```rust
pub(crate) struct ApplicationEquality {
    pub left: TypeId,
    pub right: TypeId,
    pub origin: ApplicationEqualityOrigin,
}

application_equalities: Vec<ApplicationEquality>
```

The exact product MAY store `TypeTerm` instead of `TypeId` if needed.

After `var_map` contains:
- fresh declaration vars;
- fresh callable vars;
- fixed receiver generics;

lower both equation sides through the same `type_id_to_inference(..., &var_map, ...)` and add:

```rust
InferenceRelation::Equivalent(left, right)
```

to the same session.

Prefer a dedicated `ConstraintOrigin::GadtCase` or equivalent for diagnostic provenance.

Do not put these equations into a synthetic publishable `GenericSignature`.

---

## Task 26 — Remove premature direct variant GADT prechecks

Purpose:
Fix `Expr<List<Int>>::Wrap(1)` ordering.

Risk:
- Semantic: HIGH
- Implementation fanout: local expression associated invoke

Owned symbol:
- `synthesize_associated_invoke`

Current anchor:
comment equivalent to:
```text
// GADT check if owner type arguments were explicitly supplied
```

Delete:
mutual `is_subtype` preflight involving unspecialized `constrained_ty`.

Replace:
- construct variant target;
- attach fixed declaration generics;
- attach declaration generic domain;
- attach application equality inputs from case environment;
- invoke ordinary application.

Ground conflicts should emerge through application equality solving and preserve a stable diagnostic category.

Negative search:
```bash
rg -n 'is_subtype\(.*constrained_ty|constrained_ty.*is_subtype' \
  phalcom-semantic/src/checker/expression.rs
```
Expected zero GADT owner-precheck hits.

---

## Task 27 — Repair associated/Family variant specialization and residual generic domains

Purpose:
Make capture of `Expr<List<Int>>::Wrap::*` retain and exploit owner-derived GADT evidence.

Risk:
- Semantic: HIGH
- Implementation fanout: associated resolution + Family invocation

Owned symbol:
- `specialize_associated_member`
- reification underconstraint helpers
- Family invocation path in `expression.rs`

Current defect:
`specialize_associated_member` performs the same premature mutual-subtyping precheck and then builds a canonical structural callable type.

Target:
1. specialize declaration-owned receiver parameters;
2. project GADT equations;
3. solve any callable-local binder fully determined by fixed receiver evidence;
4. expose a residual callable generic domain:
   ```text
   original binders - proven solved binders
   ```
5. retain stable variant constructor invocation identity;
6. preserve unresolved callable-local generics for rank-1 invocation;
7. pass remaining equality evidence to Family invocation.

Implementation note:
A structural `TypeData::Callable` alone does not encode generic signature. Reuse the existing associated denotation/member identity so Family invocation can recover the canonical `VariantInfo`/constructor generic contract by identity; do **not** infer generic authority from Family type shape alone.

Also migrate `contains_only_variant_constructor_parameters` to the C1 traversal authority.

---

## Task 28 — Certify direct/Family parity and ground conflicts

Required tests:

```text
CONSTRUCT-01 Expr<List<Int>>::Wrap(1) → PASS
CONSTRUCT-02 captured Expr<List<Int>>::Wrap::* then invoke → PASS
CONSTRUCT-03 owner equation fully determines U; residual view is monomorphic
CONSTRUCT-04 partial owner evidence removes only solved local binders
CONSTRUCT-05 ground contradictory owner equation → FAIL CLOSED
CONSTRUCT-06 generic selector/CallableId identity unchanged
```

Run:

```bash
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::generics -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::associated -- --nocapture
```

Use the actual available filter if `semantic::associated` is broader/differently nested.

---

# 14. Checkpoint C7 — Accessor Conformance and Source-Index Parity

Tasks:
- Task 29 — Make every index-setter callable signature return `Unit`
- Task 30 — Extend type-reference collection for generic accessors and enums
- Task 31 — Extend source scope/occurrence construction for enum/variant syntax
- Task 32 — Add cross-consumer source-index and accessor evidence

## Why this is a checkpoint

These are language-visible parity defects adjacent to the central proof work but do not justify delaying correctness checkpoints C1–C6.

## Working set

Primary:
- `checker/declaration_signature.rs`
- `checker/call.rs`
- `source_index/builder.rs`
- `identity.rs`
- existing source-index tests

Secondary:
- source occurrence/presentation only if required by existing target API

Out of scope:
- parser grammar (AST already carries required generic/where/enum data);
- LSP-specific semantic resolver.

---

## Task 29 — Make every index-setter callable signature return `Unit`

Purpose:
Enforce ratified setter semantic consistency.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local plus fallback

EXACT declaration-signature edit:

Open `checker/declaration_signature.rs`.

Find index setter branch:

```rust
let result = put_semantic.declared_type.clone();
parameters.push(put_semantic);
result
```

Replace semantic result construction with the same canonical pattern used by property setters:

```rust
let unit = TypeKnowledge::established(
    ctx.store.unit(),
    EvidenceOrigin::DeclarationSemantics,
);
parameters.push(put_semantic);
DeclaredTypeFact::from_knowledge_with_basis(
    &unit,
    DeclaredTypeBasis::DeclarationSemantics,
)
```

Adapt formatting/imports to exact surrounding code.

Also inspect/update:
- `call.rs::structural_list_index_set_target` — currently returns element type; make fallback return canonical `Unit`;
- any Map/set structural index setter equivalent;
- native index-set signatures if generated separately.

Tests must inspect:
- `CallableSemanticSignature.declared_return`;
- projected call signature;
- assignment expression result;
- generic ownership;
- selector identity.

Negative:
```bash
rg -n 'let result = put_semantic\.declared_type\.clone' phalcom-semantic/src
```
Expected zero.

---

## Task 30 — Extend type-reference collection for generic accessors and enums

Purpose:
Make compiler-owned type-reference resolution respect newly legal binders/where clauses.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local source-index collector

Owned:
`source_index/builder.rs::TypeReferenceTargetCollector`

Changes for getter/setter/index:
- clone class-bound set;
- extend with that member's `generic_parameters`;
- visit parameter/return annotations under extended set;
- visit `where_clause`.

Changes for `Statement::Enum`:
- create enum-bound set from enclosing + enum generic parameters;
- visit enum `where_clause`;
- for every variant:
  - extend enum-bound with variant-local generics;
  - visit variant payload annotations;
  - visit result annotation;
  - visit variant `where_clause`;
- visit enum behavior member type annotations using same accessor/method rules.

Hostile:
same-spelled nominal declaration must not capture a local generic binder reference.

---

## Task 31 — Extend source scope/occurrence construction for enum/variant syntax

Purpose:
Stop `SourceScopeBuilder` from skipping enums/variants.

Risk:
- Semantic: MEDIUM/HIGH
- Implementation fanout: source index + identity consumers

Inspect before edit:
- current `SemanticTargetId` variant cases in `identity.rs`;
- prior ADT source occurrence behavior;
- `SourceCallableKind` support for enum behavior;
- existing variant source identities.

Use existing `VariantId`/`VariantFamilyId` target variants. Do not introduce string-only variant targets.

Implement:
- enum declaration source info using canonical `DeclarationId`;
- variant declaration/source target using current `VariantId` identity;
- constructor/family selector sites using existing source-target contracts;
- enum behavior callables through existing callable visitor where possible.

If source index intentionally excludes type-level generic binders as `SourceBindingInfo`, do not invent value-binding records for them; TypeReferenceTargetCollector handles lexical type binder exclusion.

---

## Task 32 — Add cross-consumer source-index and accessor evidence

Tests:
- generic getter binder shadows nominal same name;
- setter/index binder equivalent;
- accessor where-clause external type resolves to canonical declaration target;
- enum variant payload/result external types resolve;
- variant local binder does not become nominal occurrence;
- exact variant source occurrence target matches semantic `VariantId`;
- index setter signature and expression agree on `Unit`.

Run:

```bash
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::source_index -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::capabilities::index_generics -- --nocapture
```

No LSP suite unless the source-index API change causes adapter failure; source index is the semantic authority.

---

# 15. Checkpoint C8 — Durable Variant Generic Metadata and Incremental Normalization

Tasks:
- Task 33 — Extend the durable metadata schema with enum/variant records
- Task 34 — Export canonical `EnumSemanticTable` / variant-constructor generics
- Task 35 — Validate schema compatibility and forbid local proof leakage
- Task 36 — Add cold/incremental binder-normalized semantic parity

## Why this is a checkpoint

The repository inspection confirms this is not merely a missing test: current `MetadataExporter` has no enum semantic table input, and `SemanticMetadataBundle` has no dedicated enum/variant record collection.

This checkpoint crosses `phalcom-semantic` and `phalcom-type-meta`; keep it isolated from core proof repair.

## Source of truth

- `EnumInfo` / `VariantInfo` in `EnumSemanticTable`;
- generic signature exporter already present in `MetadataExporter`;
- stable identities from existing metadata stable-identity conversion.

Forbidden:
- regenerating a variant generic signature from source AST;
- serializing local rigid/proof state.

---

## Task 33 — Extend the durable metadata schema with enum/variant records

Purpose:
Create a stable schema surface for canonical enum/variant declaration semantics.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate / schema compatibility

Primary:
- `phalcom-type-meta/src/bundle.rs`
- `phalcom-type-meta/src/header.rs`
- `phalcom-type-meta/src/validate.rs`
- module containing declaration/callable records; create a new enum/variant record module only if consistent with crate organization
- `phalcom-type-meta/tests/schema_compat.rs`

STRUCTURAL schema requirements:

Enum record:
```text
stable declaration identity
root type form
optional generic signature
default result type
stable variant identities/list
```

Variant record:
```text
stable VariantId-derived identity
shape
payload fields with stable field identity/name/label/type
result type template
exact-case template if exportable under current schema
optional constructor:
    stable constructor/callable identity
    optional generic signature record
    parameter field correspondence
    result type
```

Before defining exact serialized identity types, inspect existing stable variant identity support. Reuse it; do not encode selector text as sole identity.

Bump metadata schema/model version according to existing compatibility policy.

Testing classification:
- focused `phalcom-type-meta` schema/validation tests required.

---

## Task 34 — Export canonical enum/variant semantics

Purpose:
Wire `EnumSemanticTable` into `MetadataExporter`.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Owned:
- `phalcom-semantic/src/metadata/export.rs`
- exporter construction call sites/session

Changes:
1. add optional/required enum semantic table to exporter inputs consistent with declarations/callables;
2. export generic signature using existing `export_generic_signature` authority;
3. export parameter kinds through existing type parameter exporter;
4. export payload/result type forms through existing type-node export;
5. export stable variant/field/constructor identity;
6. populate bundle enum/variant collections;
7. ensure module roots/fingerprints include new records per schema policy.

Must not:
- export `CaseInstantiation`;
- export `BranchProofEnvironment`;
- export `LocalType`;
- export rigid IDs.

---

## Task 35 — Validate schema compatibility and forbid local proof leakage

Purpose:
Make the new durable surface mechanically valid and hard-barrier local state.

Risk:
- Semantic: MEDIUM/HIGH
- Implementation fanout: schema validation + tests

Extend:
- bundle validator;
- schema compatibility sample bundles;
- structural fingerprinting if bundle fingerprints cover record collections.

Retain existing `MetadataExportError::ScopedLocalType` hard guard.

Tests:
- generic variant constructor exports callable-owned generic binder;
- kind and constraints survive;
- no local rigid ID exists in output types;
- metadata round-trip/validation succeeds;
- non-enum metadata remains compatible according to version policy.

---

## Task 36 — Add cold/incremental binder-normalized semantic parity

Purpose:
Ensure local allocator order never becomes semantics.

Risk:
- Semantic: HIGH
- Implementation fanout: incremental tests; fingerprint code only if evidence shows local proof contributes directly

Primary:
- existing incremental semantic tests;
- `db/fingerprint.rs` only if relevant product is fingerprinted.

Before editing fingerprint code:
1. inspect whether `MatchResolution`/branch local proof is part of durable callable fingerprint;
2. if it is not, do not add unnecessary binder normalization to fingerprints;
3. compare cold/incremental match semantic products via C2 binder-normalized representation.

Required cases:
- simple variant-local generic;
- nested GADT;
- `κ ≡ Int`;
- open row;
- or-pattern;
- Family constructor;
- body-only edit;
- constructor generic signature edit invalidation.

Never compare raw rigid IDs.

---

# 16. Checkpoint C9 — Cleanup and Full Semantic Certification

Tasks:
- Task 37 — Remove avoidable allocations and repeated scans
- Task 38 — Add deterministic bounded algebra/property certification
- Task 39 — Run affected-layer and full semantic certification
- Task 40 — Close state, deletion gates, and delivery handoff

## Why this is a checkpoint

Correctness is established earlier. C9 removes the small avoidable work identified during review and proves the repaired abstractions as a whole without repeatedly running workspace-scale tests during development.

---

## Task 37 — Remove avoidable allocations and repeated scans

Purpose:
Finish the requested code-quality/performance cleanup after semantics stabilize.

Risk:
- Semantic: LOW/MEDIUM
- Implementation fanout: multi-file mechanical

Targets:

1. `free_rigids` boolean callers → nonallocating visitor.
2. repeated `CaseInstantiation::replacements()` map construction → borrowed/local substitution view or one map per opening.
3. or-pattern active alternative clone → borrowed views.
4. repeated variant-field `.position(...)` scan → carried ID/entry.
5. escape constraint vector cloning → iterate existential-frame proof/constraints plus additional constraints.
6. repeated alpha mapping allocation → one proof-wide `AlphaRenaming`.
7. duplicated canonical parameter walkers already removed in C1.

Do not add persistent caches.

Optional benchmark/check only after cleanup:
run representative semantic fixtures and record timings if the repository has a benchmark harness; otherwise record code-level allocation removals and defer precise performance measurement.

---

## Task 38 — Add deterministic bounded algebra/property certification

Purpose:
Protect the repaired semantic algebra against combinations not worth spelling individually.

Risk:
- Semantic: MEDIUM
- Implementation fanout: test-only

Use deterministic bounded generators, no new property-testing dependency initially.

Generate small LocalType trees from:
- Canonical;
- Rigid;
- Applied;
- ExactCase;
- Union;
- Tuple;
- Record with closed/stable/rigid tail;
- Callable;
- Family;
- supported lambda overlays.

Laws:
```text
alpha reflexive
alpha symmetric
alpha transitive
alpha renaming invariant
kind preservation
correlation preservation
S ∩ S = S
S \ S = ∅
S ∩ α(S) = S
rigid-free localize/materialize round trip
```

Keep generator depth small enough for normal CI.

---

## Task 39 — Run affected-layer and full semantic certification

Purpose:
Prove no SC-4.8 adjacent semantic subsystem regressed.

Risk:
- Semantic: evidence only

Smallest-first sequence:

```bash
# focused repaired domains
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::existentials -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::matching -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::adts::generics -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::capabilities::index_generics -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::source_index -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::metadata -- --nocapture

# full semantic owner layer
RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic

# core semantic integration authorities
RUSTFLAGS='' cargo +stable test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture

RUSTFLAGS='' cargo +stable test -p phalcom-core --test core \
  typing_integration::either:: -- --nocapture

# compile integration
cargo +stable check --workspace --all-targets
```

What these prove:
- focused tests prove repaired semantic laws;
- full semantic suite proves crate-level semantic compatibility;
- Monad/Either prove established generic/HKT/ADT authorities remain intact;
- workspace check proves Rust/API integration, not semantic correctness by itself.

---

## Task 40 — Close state, deletion gates, and delivery handoff

Purpose:
Make completion auditable and ensure old authorities cannot silently coexist.

Risk:
- Semantic: LOW
- Implementation fanout: docs/search

Run final negative searches:

```bash
rg -n 'TypeData::Rigid|GadtSkolem|TypeParameterOwner::Variant' phalcom-semantic

rg -n 'merge_constructor_generic_signatures' phalcom-semantic

rg -n 'attach_local_type_to_bindings' phalcom-semantic/src

rg -n 'active_local_constraints' phalcom-semantic/src

rg -n 'let result = put_semantic\.declared_type\.clone' \
  phalcom-semantic/src/checker/declaration_signature.rs

rg -n 'fn type_contains_any_parameter|pub fn contains_any_type_parameter' \
  phalcom-semantic/src/checker phalcom-semantic/src/types

rg -n 'AssociatedGadtOwnerConflict' \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/associated.rs
```

Expected:
- prohibited type/owner/synthetic-signature mechanisms: zero production hits;
- bulk pattern local attachment: zero;
- old constraint vector field: zero;
- put-type index return: zero;
- duplicate walkers: zero or explicitly documented delegating wrappers;
- `AssociatedGadtOwnerConflict` may remain as a diagnostic code/emission for **ground/solver-proven conflict**, but no premature mutual-subtype owner preflight remains.

Update implementation state with all evidence/deferred gates.

---

# 17. Cross-Cutting Task Guidance

## 17.1 `LocalType` versus canonical `TypeId`

Implementers must keep these roles separate:

```text
TypeId
    durable/canonical semantic type form

LocalType
    query-local view capable of containing existential witnesses

InferenceTerm
    query-local generic solver term, already capable of Rigid atoms

BranchProofEnvironment
    branch proof product: canonical GADT parameter facts + local existential facts

RigidArena
    allocation/kind/scope/provenance of local witnesses
```

No one of these should absorb the others.

## 17.2 Exact-case identity

Do not alter:

```rust
TypeData::ExactCase { variant, enum_type }
```

A match reopens hidden constructor-local binders from canonical `VariantInfo`.

## 17.3 Declaration versus callable generics

Keep `GenericApplicationDomains`.

Do not merge:
```text
Enum<T>
+
Variant<U>
```
into one publishable owner-invalid signature.

## 17.4 Equality versus subtyping

Use:
- `Equivalent` for GADT index equations;
- `Subtype` for ordinary argument/contract relations;
- proof equality to normalize rigids before outward relation.

Do not use bidirectional subtyping as a substitute for generic equality when unsolved terms exist.

---

# 18. Detailed Testing Extension Catalog

The implementing agent should map these IDs onto repository-native test modules rather than creating a completely new directory hierarchy if existing files already own the behavior.

## Equality

```text
SC48-EQ-01 concrete GADT existential revelation
SC48-EQ-02 no rigid guessing without evidence
SC48-EQ-03 rigid-rigid proven equality
SC48-EQ-04 structural equality decomposition
SC48-EQ-05 contradictory local equality refutes branch
```

## Nested patterns

```text
SC48-NEST-01 tuple leaf
SC48-NEST-02 repeated correlation
SC48-NEST-03 nested generic variant
SC48-NEST-04 deep nested structure
SC48-NEST-05 record field
SC48-NEST-06 list element
SC48-NEST-07 list rest
```

## Alpha/proofs

```text
SC48-ALPHA-01 fresh openings equivalent
SC48-ALPHA-02 repeated correlation
SC48-ALPHA-03 correlation mismatch rejected
SC48-ALPHA-04 bijection not collapse
SC48-ALPHA-05 kind mismatch
SC48-ALPHA-06 scope topology
SC48-ALPHA-07 reflexivity
SC48-ALPHA-08 symmetry
SC48-ALPHA-09 transitivity
```

## Pattern space

```text
SC48-SPACE-01 S ∩ α(S) = S
SC48-SPACE-02 S \ α(S) = ∅
SC48-SPACE-03 α(S) \ S = ∅
SC48-SPACE-04 duplicate independently opened GADT arm redundant
SC48-SPACE-05 different constraints remain distinct
```

## Scope/publication

```text
SC48-SCOPE-01 outer survives inner
SC48-SCOPE-02 inner cannot escape
SC48-SCOPE-03 sibling isolation
SC48-SCOPE-04 actual parent topology
SC48-SCOPE-05 three levels
SC48-SCOPE-06 rigid-free unaffected
SC48-SCOPE-07 inner equal outer normalizes before exit

SC48-ESC-01 unresolved κ -> Int reject
SC48-ESC-02 κ <: Object -> Object accept
SC48-ESC-03 κ ≡ Int -> Int accept
SC48-ESC-04 deep List<Option<(κ,Int)>> detection
SC48-ESC-05 lambda capture
SC48-ESC-06 Family capture
SC48-ESC-07 rigid row tail
```

## Rows

```text
SC48-ROW-01 row-tail-only generic occurrence
SC48-ROW-02 open row local round trip
SC48-ROW-03 rigid RecordRow tail
SC48-ROW-04 alpha row rigid
SC48-ROW-05 kind mismatch
SC48-ROW-06 nested List<open row>
SC48-ROW-07 underconstraint detection
SC48-ROW-08 local row generic consumption if supported path is reached
```

## Lambda/Family

```text
SC48-LAMBDA-01 local free capture
SC48-LAMBDA-02 rigid visitor sees capture
SC48-LAMBDA-03 proof discharge
SC48-LAMBDA-04 escape rejection
SC48-LAMBDA-05 rigid-free round trip

SC48-FAMILY-01 local member visibility
SC48-FAMILY-02 alpha
SC48-FAMILY-03 escape
SC48-FAMILY-04 proof discharge
SC48-FAMILY-05 applied variant Family owner equation
```

## Generic application

```text
SC48-LOCAL-GEN-01 id(κ)
SC48-LOCAL-GEN-02 id(List<κ>)
SC48-LOCAL-GEN-03 no ordinary κ := Int
SC48-LOCAL-GEN-04 local result reuse
SC48-LOCAL-GEN-05 later escape rejected

SC48-GENAPP-01 applied receiver + callable generic + κ
SC48-GENAPP-02 row-tail declaration generic + callable generic
SC48-GENAPP-03 α := List<κ>
SC48-GENAPP-04 rigid fixed
SC48-GENAPP-05 generic setter local use
SC48-GENAPP-06 indexer local use
```

## Construction

```text
SC48-CONSTRUCT-01 Expr<List<Int>>::Wrap(1)
SC48-CONSTRUCT-02 captured Family invocation
SC48-CONSTRUCT-03 ground contradiction
SC48-CONSTRUCT-04 same solver session
SC48-CONSTRUCT-05 residual binder elimination
```

## Recovery

```text
SC48-REC-01 first field missing
SC48-REC-02 middle field missing
SC48-REC-03 binder correspondence
SC48-REC-04 malformed sibling does not corrupt local type
SC48-REC-05 recovery cannot launder contradiction
SC48-REC-06 exact VariantFieldId retained
```

## Or-pattern

```text
SC48-OR-01 rigid-free local join
SC48-OR-02 shared outer rigid join
SC48-OR-03 independent alpha-equivalent witnesses not retained
SC48-OR-04 correlation mismatch
SC48-OR-05 kind mismatch
SC48-OR-06 canonical join survives
```

## Index/source/metadata/incremental

```text
SC48-INDEX-01 semantic declared return Unit
SC48-INDEX-02 projected return Unit
SC48-INDEX-03 expression Unit
SC48-INDEX-04 generic ownership
SC48-INDEX-05 selector stability

SC48-SOURCE-01 generic getter binder
SC48-SOURCE-02 generic setter binder
SC48-SOURCE-03 generic index binder
SC48-SOURCE-04 accessor where reference
SC48-SOURCE-05 variant binder
SC48-SOURCE-06 payload/result references
SC48-SOURCE-07 variant where reference
SC48-SOURCE-08 enum behavior member

SC48-META-01 stable variant constructor identity
SC48-META-02 kind
SC48-META-03 constraints
SC48-META-04 payload/result
SC48-META-05 no local IDs
SC48-META-06 validation/round trip

SC48-INCR-01 simple existential
SC48-INCR-02 nested GADT
SC48-INCR-03 κ ≡ Int
SC48-INCR-04 or-pattern
SC48-INCR-05 open row
SC48-INCR-06 lambda
SC48-INCR-07 Family
SC48-INCR-08 body-only edit
SC48-INCR-09 constructor-generic edit invalidation
SC48-INCR-10 row-tail edit invalidation
```

---

# 19. Structural Completeness Audit Template

At C1 completion, place this table in implementation state or a focused test comment/document and fill every row.

| `TypeData` | parameter occurrence | localize | free-rigid visibility | materialize | alpha/proof | publication |
|---|---|---|---|---|---|---|
| `Never` | leaf | canonical leaf | none | identity | identity | safe |
| `Unit` | leaf | canonical leaf | none | identity | identity | safe |
| `ClassObject` | leaf | canonical leaf | none | identity | identity | safe |
| `Nominal` | leaf | canonical leaf | none | identity | identity | safe |
| `Applied` | recursive | recursive | recursive | recursive | recursive | recursive |
| `ExactCase` | enum type | recursive | recursive | recursive | recursive | recursive |
| `Union` | recursive | recursive | recursive | recursive | recursive | recursive |
| `Tuple` | recursive | recursive | recursive | recursive | recursive | recursive |
| `Record` | fields + tail | fields + tail | fields + tail | preserve tail | kind-aware | recursive |
| `Callable` | params + return | recursive | recursive | recursive | recursive | recursive |
| `Family` | members | recursive | recursive | recursive | recursive | recursive |
| `Parameter` | yes | replacement/leaf | replacement dependent | canonical if stable | normal | normal |
| `Lambda` | free captures | capture overlay | capture overlay | only if rigid-free | capture-aware | capture-aware |
| `SelfType` | explicit leaf by semantics | canonical leaf | none | identity | identity | normal |

Any new `TypeData` variant added later must require a new row.

---

# 20. Failure Protocol

When any required checkpoint evidence fails unexpectedly:

## 20.1 Record exact reproduction

```text
command:
test/check:
key output:
first bad assertion/diagnostic:
```

## 20.2 Trace direct path

Examples:

```text
nested pattern test
→ resolve_pattern_with_mode
→ resolve_variant_pattern
→ CaseInstantiation payload
→ binding local type
```

```text
applied constructor test
→ synthesize_associated_invoke
→ CallableApplicationTarget
→ apply_generic_callable_in_context
→ InferenceSession
```

## 20.3 Find a passing comparator

Examples:
- raw `Expr::Wrap(1)` works while applied `Expr<List<Int>>::Wrap(1)` fails;
- nonlocal generic `id(Int)` works while `id(κ)` fails;
- cold match works while incremental normalized comparison fails.

## 20.4 Classify

Use exactly:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## 20.5 Narrow repair boundary

Write the one allowed subsystem/symbol before editing.

## 20.6 Rejected broad fixes

Never:
- turn failure into `Dynamic`;
- disable a hostile assertion;
- add parser changes without syntax evidence;
- restore premature subtyping;
- compare identities by names;
- add local state to TypeStore.

A checkpoint with failing required evidence is:

```text
C<N> — INCIDENT
```

not “mostly complete.”

---

# 21. Working-State Protocol

After each checkpoint append/update a concise state section:

```md
## SC-4.8 hardening — C<N>

Status:
- C<N> COMPLETE | INCIDENT

Established invariants:
- ...

Changed anchors:
- `<path>` — `<symbol>`

Decisions:
- ...

Rejected approaches:
- ...

Evidence ledger:
| Command | Result | Proves |
|---|---|---|

Negative gates:
- ...

Deferred:
- `<command>` → C<M>/Final Gate

Unexpected findings:
- None | ...

Active incident:
- None | ...

Next resume action:
- Begin C<N+1> Task ...
```

Do not record scratch reasoning. Record only facts, decisions, evidence, and resume anchors.

---

# 22. Suggested Commit Groups

Do not force one commit per task. Suggested checkpoint-aligned groups:

```text
C0
test(semantic): lock SC-4.8 composition regressions
docs(semantic): open SC-4.8 hardening incident

C1
refactor(semantic): complete local existential structural algebra
refactor(semantic): centralize canonical parameter traversal
test(semantic): enforce local structural completeness

C2
fix(semantic): add evidence-sensitive local GADT equality
fix(semantic): make branch proof alpha compatibility binder-aware

C3
fix(semantic): propagate local types recursively through patterns
fix(semantic): preserve variant payload field identity
fix(semantic): stabilize existential pattern-space algebra

C4
fix(semantic): operationalize lexical existential scopes
fix(semantic): make existential publication proof and scope aware

C5
fix(semantic): compose local existentials with generic application
fix(semantic): support fixed rigid row tails in local inference

C6
fix(semantic): solve variant GADT equations in application inference
fix(semantic): specialize applied generic variant Families

C7
fix(semantic): make index setters return Unit
fix(semantic): index generic accessor and enum type references

C8
feat(type-meta): export canonical enum and variant generic metadata
test(semantic): normalize incremental existential semantics

C9
perf(semantic): remove redundant local-type allocations and scans
test(semantic): certify SC-4.8 composition laws
docs(semantic): close SC-4.8 hardening evidence
```

---

# 23. Final Delivery Gates

These gates are run only after every checkpoint is `COMPLETE`.

## 23.1 Formatting

```bash
cargo +stable fmt --all -- --check
```

Purpose:
- delivery formatting consistency.

Does not prove:
- semantic correctness.

If it fails only on known unrelated baseline drift, record exact diff classification rather than claiming PASS.

## 23.2 Workspace compile

```bash
cargo +stable check --workspace --all-targets
```

Purpose:
- complete Rust API/caller integration;
- all targets compile.

## 23.3 Workspace tests

```bash
cargo +stable test --workspace --all-targets
```

Purpose:
- cross-crate regression compatibility.

This command previously lacked a completed baseline result because a long core fixture/corpus test was manually interrupted. A final release-complete claim requires an actual completed result or a formally recorded release blocker.

## 23.4 Clippy

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Purpose:
- lint/delivery quality.

Existing unrelated `phalcom-core` baseline lints must remain separately classified if still present.

## 23.5 Type metadata crate

After C8 schema changes:

```bash
cargo +stable test -p phalcom-type-meta
```

Purpose:
- schema compatibility, validation, serialization contracts.

## 23.6 Full semantic + core integration

Repeat C9 owner-layer gates only if any code changed after their PASS.

Do not rerun otherwise merely for ritual.

---

# 24. Final Negative/Deletion Gates

Required:

```bash
# Never introduce canonical rigid state
rg -n 'TypeData::Rigid|GadtSkolem' phalcom-semantic

# Never introduce a Variant generic owner
rg -n 'TypeParameterOwner::Variant' phalcom-semantic

# Old mixed-owner signature authority remains absent
rg -n 'merge_constructor_generic_signatures' .

# Post-hoc nested binding rewrite removed
rg -n 'attach_local_type_to_bindings' phalcom-semantic/src

# Scattered constraint state removed
rg -n 'active_local_constraints' phalcom-semantic/src

# Wrong index-set return removed
rg -n 'put_semantic\.declared_type\.clone' \
  phalcom-semantic/src/checker/declaration_signature.rs

# Duplicate parameter walkers removed/delegated
rg -n 'fn type_contains_any_parameter|pub fn contains_any_type_parameter' \
  phalcom-semantic/src/checker

# Premature GADT mutual-subtyping patterns removed
rg -n 'constrained_ty' \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/associated.rs
```

Any remaining occurrence must be listed and justified.

---

# 25. Deferred-Evidence Audit

Before delivery:

- [ ] every command marked deferred was either run successfully;
- [ ] or explicitly removed from scope with written justification;
- [ ] or recorded as a known release blocker;
- [ ] no deferred test is silently assumed PASS;
- [ ] no checkpoint relies on evidence scheduled later without having its own semantic proof.

---

# 26. Known Scope Exclusions

This plan intentionally does **not** implement:

- first-class existential packages;
- implicit existential repackaging across or-pattern alternatives;
- runtime existential boxes;
- runtime GADT proof witnesses;
- `TypeData::Rigid`;
- `TypeParameterOwner::Variant`;
- rank-N polymorphism;
- monomorphization;
- per-applied-class runtime static storage;
- a new general higher-order unifier;
- a constructor-only generic solver;
- a setter/indexer-specific solver;
- unrelated repository-wide fmt/clippy debt;
- unrelated parser syntax changes;
- unrelated VM/runtime collection optimization.

---

# 27. Checkpoint Evidence Summary Template

Fill during execution.

| Checkpoint | Semantic contract | Required evidence | Status |
|---|---|---|---|
| C0 | incident reproduced and guarded | hostile RED baseline + existing focused green baseline | PENDING |
| C1 | local structural algebra complete | structural tests + crate check + walker deletion | PENDING |
| C2 | evidence-sensitive equality/alpha proof | equality/alpha hostile tests | PENDING |
| C3 | recursive pattern elimination exact | matching/pattern-space tests | PENDING |
| C4 | lexical scope/publication sound | scope/escape matrix | PENDING |
| C5 | local generic consumption sound | local generic/row tests | PENDING |
| C6 | construction GADT equations unified | direct + Family construction tests | PENDING |
| C7 | accessor/source index parity | index/source-index tests | PENDING |
| C8 | durable metadata + incremental parity | type-meta/schema + incremental tests | PENDING |
| C9 | cleanup + semantic certification | property/full semantic/core/check | PENDING |

No row may be marked COMPLETE without its evidence.

---

# 28. Release-Complete Criteria

The implementation program is complete only when all of the following are true.

## Semantic

- [ ] GADT evidence can prove `κ ≡ Concrete` without turning the rigid into a metavariable.
- [ ] ordinary inference still cannot guess a rigid.
- [ ] nested pattern bindings receive exact recursive local types.
- [ ] proof merge is alpha-aware with global correlation.
- [ ] alpha comparison checks kind and scope topology.
- [ ] nested rigid scopes have actual parentage.
- [ ] escape is relative to the leaving scope.
- [ ] proof normalization can discharge a local existential before publication.
- [ ] sound bound widening still works.
- [ ] local generic calls consume live existentials.
- [ ] constructor GADT equations use the ordinary application solver.
- [ ] direct and Family variant construction agree.
- [ ] record-row tails survive local conversion.
- [ ] RecordRow-kind constructor locals remain kind-correct.
- [ ] lambda free captures cannot hide rigids.
- [ ] Family members cannot hide rigids.
- [ ] recovery never shifts variant field correspondence.
- [ ] or-pattern joins do not invent witness identity.
- [ ] index-setter canonical return is `Unit`.
- [ ] source indexing understands generic accessors/enums/variants.
- [ ] durable metadata exports canonical variant generic contracts.
- [ ] cold/incremental semantics ignore raw rigid allocation order.

## Architecture

- [ ] no `TypeData::Rigid`;
- [ ] no `TypeParameterOwner::Variant`;
- [ ] no mixed-owner synthetic constructor signature;
- [ ] no feature-specific generic solver;
- [ ] no competing parameter-occurrence walker;
- [ ] no post-hoc descendant local-type attachment;
- [ ] no scattered `active_local_constraints`;
- [ ] no query-local rigid/proof state exported durably.

## Evidence

- [ ] all checkpoints COMPLETE;
- [ ] hostile tests PASS;
- [ ] deterministic law/property tests PASS;
- [ ] full `phalcom-semantic` suite PASS;
- [ ] Monad/Either regressions PASS;
- [ ] `cargo +stable check --workspace --all-targets` PASS;
- [ ] `phalcom-type-meta` schema tests PASS;
- [ ] every deferred broad gate resolved;
- [ ] final workspace format/test/clippy status accurately recorded;
- [ ] no unresolved state-file INCIDENT remains.

The normative completion statement may then be restored to:

```text
SC-4.8 semantic implementation:
    COMPLETE
```

with the stronger meaning:

> Every canonical type form, generic kind, pattern form, branch-proof relation, lexical scope transition, local generic-consumption path, and scope-exiting publication boundary reachable through SC-4.8 has an explicit and tested disposition for branch-local existential structure.

---

# 29. First Resume Action for the Implementing Agent

Start with **C0 Task 1**.

Do not edit production semantic code first.

Locally record:

```bash
git rev-parse HEAD
git branch --show-current
git status --short
```

If HEAD is not `e932aac4e21a5b346e719ede5a24f94e7b924ab3`, perform the bounded drift check described in §6 before applying any edit.

Then run the existing focused GADT/existential/generic suites and add the hostile C0 regressions.

Only after C0 is `COMPLETE — INCIDENT REPRODUCED` should C1 production work begin.
