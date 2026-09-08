# Phalcom SC-1 Correctness Amendment Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Treat this document as an amendment: when it conflicts with either historical correctness plan, this document wins for the current implementation sequence.

**Goal:** Reconcile the two existing SC-1 correctness/stabilization plans with the current repository, remove stale or already-completed work, add current blockers that were absent from the historical reviews, and establish an implementation-safe order for the remaining semantic/runtime correctness work.

**Architecture:** Preserve canonical source-owned `ModuleId` / `DeclarationId` / `VariantId` identity as the semantic authority. Make module interfaces and source declarations authoritative before runtime/native overlays, make runtime realization consume canonical identity instead of source spelling, and postpone behavior-preserving baseline extraction until the corrected behavior is proven. Cross-layer correctness is accepted only when semantic, module, runtime, reflection, and durable-metadata identities agree.

**Tech stack:** Rust; `phalcom-modules`; `phalcom-semantic`; `phalcom-core`; `phalcom-native-meta`; `phalcom-type-meta`; canonical Universe `.ph` source; existing unified test targets.

**Amends:**

- `docs/impl/semantic/semantic-completeness/sc-1/phalcom-pre-sc1-stabilization-patch-grade-implementation-plan.md`
- `docs/impl/semantic/semantic-completeness/sc-1/phalcom-post-universe-review-correctness-remediation-plan.md`

**Historical evidence reviewed:**

- `docs/impl/semantic/semantic-completeness/sc-1/sc-blockers-1.md`
- `docs/impl/semantic/semantic-completeness/sc-1/sc-blockers-2.md`

---

## 0. Purpose and scope

This document is a **current-HEAD correctness amendment** to the two plans named above. It is not a replacement SC-1 feature plan and it is not a second broad code review. Its purpose is to tell an implementer exactly which historical tasks still apply, which no longer apply, which need to be rewritten, and in what order the surviving work can safely be executed.

The historical analyses and plans are evidence only. Source, tests, test registration, current ledgers, and current compiler/runtime ownership boundaries are authoritative.

### 0.1 Explicit exclusions

The following are intentionally outside this amendment:

1. all Rust toolchain selection work;
2. Cargo configuration and compiler-version work;
3. CI toolchain configuration or CI compiler-version remediation;
4. the lightweight/native `Result<T, E>` representation plan;
5. any change to `Result` physical storage, `Value` wrapper bits, spill objects, native unary wrappers, or related representation optimization;
6. unrelated parser completion, full Family semantics, broad GADT completion, or golden-pipeline completion merely because current ignored tests expose those future gaps.

`Result` remains a `General` runtime ADT for this correctness amendment. The only `Result` work here is **identity correctness**: one canonical runtime root, canonical variant identity, correct source terminology, and agreement between registries/reflection/dispatch.

### 0.2 Repository-preservation rule

Implementation must preserve unrelated dirty, staged, untracked, and parallel-owned work. No implementation slice may begin until the implementer records:

```bash
git rev-parse HEAD
git status --short
```

The amendment document itself is the only artifact this planning task is permitted to add. Implementers must not edit the two historical plans.

### 0.3 Audit provenance and local-checkout preflight

The live repository state available to this amendment audit was the current public `main` at:

```text
4148de61f5415729fe5fe4ccfcef383292548ffe
```

with commit subject:

```text
docs: add build benchmark follow-up guide
```

The relevant production-source audit was originally grounded while `main` was at `5a1dee0db6e4e60554159be3eef34c5cf3eb701a`. Before finalization, `main` advanced by three commits to the SHA above: one CI-only commit and two documentation-only commits. No production source inspected for the correctness findings changed in that delta, so the source dispositions below remain grounded against the current public tip while the intervening toolchain change remains explicitly out of scope.

The requested local checkout path was not available to the planning environment after repository handoff was declined, so local branch identity, uncommitted changes, and local-only graphify output could not be independently observed here. **Before implementation, the executor must re-run the preflight above in `/Users/altunhasanli/dev/phalcom/phalcom`; if local HEAD differs from the audited commit, re-ground every named symbol before editing.**

`AGENTS.md` requires graphify-first navigation when the knowledge graph is present. The accessible repository snapshot did not expose `graphify-out/`. On the real checkout, run the repository's graphify query first if `graphify-out/` exists, then use the source paths in this amendment as targeted follow-up navigation.

The exact requested amendment filename was not present on the audited public `main`. If the local checkout already contains it, do **not** overwrite it; use a variant such as:

```text
phalcom-sc1-correctness-amendment-plan-current-head.md
```

and record the collision in the commit/implementation notes.

---

# 1. Current repository baseline

## 1.1 Status vocabulary

This amendment uses five status levels deliberately. Do not collapse them.

| Status | Meaning |
|---|---|
| **implemented** | Production source contains the intended mechanism. |
| **partial** | Some layers are implemented, but a required authority/invariant is still split or incomplete. |
| **focused-tested** | A registered focused test exercises the relevant behavior and currently passes. |
| **gated** | Tests or code exist, but the repository explicitly does not treat them as release evidence yet; tests may be ignored, vacuous, incomplete, or dependent on another capability. |
| **release-complete** | Focused tests, package gates, cross-layer invariants, and broad correctness gates all pass with no unexplained regressions. |

A finding can be implemented without being release-complete. A green focused test cannot promote a cross-layer invariant by itself.

## 1.2 Current semantic-gate evidence

The current `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` records:

```text
56 READY
16 STAGED
34 GATED
```

and, for the unified semantic test binary:

```text
active semantic gate: 905 passed, 2 failed, 51 ignored
forced ignored run:   20 passed, 31 failed
```

The two active failures are materially different:

1. `foundations::expression_engine::test_keyword_argument_mismatch_detected` — a real semantic implementation bug: a keyword argument with a value incompatible with its declared `Int` parameter is not producing `ArgumentMismatch`.
2. `support::regressions::union_expectation_rejects_wrong_structural_members` — a test-harness bug: the oracle intentionally panics while the workspace uses aborting panic behavior, so `catch_unwind` cannot provide the intended assertion mechanism.

These failures are current baseline evidence. They must be separated from regressions introduced by amendment implementation.

The forced-ignored failures include parser/fixture prerequisites, broader semantic gaps, placeholder tests, stale tests, and intentional process termination. They do **not** all become SC-1 correctness blockers merely because they fail when forced.

## 1.3 Current test-target registration

Historical commands must not be copied blindly.

`phalcom-semantic` has `autotests = false` and one registered integration target:

```toml
[[test]]
name = "semantic"
path = "tests/semantic.rs"
```

Therefore semantic focused tests are run through:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic <filter> -- --nocapture
```

`phalcom-core` also has `autotests = false` and one registered integration target:

```toml
[[test]]
name = "core"
path = "tests/core/mod.rs"
```

`tests/core/mod.rs` includes `../native_adt_runtime.rs`, so `native_adt_runtime.rs` is live test code, but **not** an independent Cargo test target. The historical command:

```text
cargo test -p phalcom-core --test native_adt_runtime
```

is obsolete. Use:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_adt_runtime -- --nocapture
```

`phalcom-modules` uses normal integration-test discovery. Current registered test files include at least:

```text
builtin_catalog.rs
builtin_provider.rs
declaration_shells.rs
dunder_policy.rs
graph.rs
identity_foundation.rs
integration.rs
interface_extraction.rs
linker.rs
package_info_semantics.rs
```

## 1.4 Current SC-1 type-formation baseline

Several SC-1 findings in the original review are no longer implementation tasks. Current source already contains the intended mechanisms for:

- explicit `TypeFormationOutcome<T>` terminal states;
- invalid kind syntax not becoming `KindId::TYPE`;
- missing declaration type products not fabricating a nominal type;
- `TypeLevelBinding::RecordRow` rather than `parameter_form` for row-kinded binders;
- scoped type-lambda bound nodes;
- contextual `TypeFormationSite` / `Self` dispatch side;
- explicit generic-signature formation outcomes.

Two additional tasks that the pre-SC-1 plan still described as open have also moved:

- ordinary record annotation lowering now detects a non-empty tail and returns `TypeFormationInvalid::UnsupportedOpenRecordTail` instead of erasing the tail;
- `phalcom-modules::InterfaceBuilder` now handles `Statement::TypeAlias` in declaration pass 1, and `phalcom-semantic::SemanticWorkspaceSession` has alias tables/lowering paths.

Alias release coverage remains gated in the ledger, so “alias module declaration exists” is **implemented**, while “all alias publication/dependency/invalidation laws are release-complete” is **not yet proven**.

---

# 2. Architectural invariants for every amendment slice

1. **Names locate identities; identities determine semantics.** Once a symbol has a `ModuleId`, `DeclarationId`, `VariantId`, `ClassId`, or stable declaration identity, no later layer may reconstruct its meaning from a leaf spelling.
2. `phalcom-modules` is the authority for package/module/import/export/exposure identity.
3. `phalcom-semantic` is the authority for language declaration/type/kind/ADT identity.
4. Universe `.ph` source owns ordinary language declarations and their generic/type contracts.
5. Native metadata attaches implementation/bootstrap facts to source-owned declarations; it is not a second declaration authority.
6. Runtime-support-only classes may exist, but they must not leak into ordinary lexical type lookup merely because runtime bootstrap needs them.
7. For every canonical nominal declaration, runtime realization is a function:

   ```text
   DeclarationId -> one authoritative root ClassId
   ```

   No registry or materializer may silently choose another root.
8. `VariantId` is the semantic identity of a variant. Constructor spelling is never a substitute after semantic resolution.
9. Prelude membership, export visibility, runtime primordial status, native implementation, and eager initialization are independent properties.
10. Stable metadata may never serialize `ResolvedProjectId` (`proj#N`) as durable project identity.
11. Discovery of the complete Universe source catalog is not the same operation as runtime initialization.
12. `TypeId` remains store-relative. Do not create a process-global semantic baseline containing store-local IDs unless the store itself is shared/frozen by design.
13. The amendment must not change `Result` physical representation.
14. Focused tests establish a slice; release completion requires adjacent package gates and broad gates.

---

# 3. Disposition matrix

## 3.1 Historical finding disposition

| ID | Historical finding | Current disposition | Current evidence / reason |
|---|---|---|---|
| F-01 | CI/toolchain verification failure | **out of scope** | User explicitly excluded toolchain, Cargo configuration, CI toolchain, and compiler-version work. Correctness verification remains required, but this amendment contains no toolchain-remediation task. |
| F-02 | Universe root creates synthetic flattened declarations | **retain with revised implementation** | `BuiltinInterfaceBuilder::build_from_parsed` still inserts missing exported `UNIVERSE_BINDINGS` as root local declarations. `UnlinkedExportTarget` still lacks a canonical-declaration target. |
| F-03 | bare type lookup uses `UniverseKey::from_name` rather than prelude policy | **retain with revised implementation** | `LinkedTypeResolver::resolve_type_name` still performs the unconditional Universe-key fallback after local/import/re-export lookup. Fold into C-02. |
| F-04 | native metadata remains semantic declaration authority | **split** | Still valid. The correctness half must move source authority first; baseline extraction must be postponed until behavior is proven. |
| F-05 | runtime generic constructor arity inferred by class name | **retain with revised implementation** | `typing/inspect.rs::class_constructor_arity` still scans `UNIVERSE_TYPE_FORMS` by class display name. However the old source test using `class List` is stale because kernel class names are now reserved. |
| F-06 | `Option<T>` unsound contracts | **supersede** | Superseded by the broader C-03 slice: `unwrapOr`, `okOr`, `match`, `map`, `flatMap`, `filter`, and `orElse` must be checked together. |
| F-07 | Universe imports bypass `expose` | **retain with revised implementation** | `ImportRootTarget::Universe` still returns after provider lookup without the resolved-project exposure walk. Fold into H-01. |
| F-08 | Universe relative dependency resolution is separate | **retain with revised implementation** | `ModuleResolver` relative resolution still assumes a `ResolvedProjectId`; Universe bootstrap still reconstructs dependency targets in `native/source.rs`. Fold into H-01. |
| F-09 | qualified type resolution drops path components | **retain with revised implementation** | Current resolver still uses `members.last()` and discards intermediate components. |
| F-10 | native/source conformance does not prove source/module identity | **retain with revised implementation** | `_resolver` and `_current_module` remain unused in `core_surface/conformance.rs`; this is an authority bug, not merely dead-parameter cleanup. |
| F-11 | no reusable `UniverseSemanticBaseline` | **defer** | Still absent, but extraction is not a correctness prerequisite. Extract only after source/native/runtime behavior is corrected and proven. |
| F-12 | native source association uses leaf names | **retain with revised implementation** | `native/source.rs` still uses `UniverseKey::from_name`. Current runtime also has post-resolution name-based paths in `VM::resolve_builtin_class_name`, `vm/associated.rs`, `modules/materialize.rs`, and `modules/context.rs`. |
| F-13 | legacy `core` / `std` stable-metadata fixtures | **retain unchanged** | Current reflection/type-metadata fixtures still model canonical Phalcom identities as `core` / `std`. Production Universe identity is `universe`. |
| legacy-core sentinel | checker excludes `universe.core` from query ownership | **retain unchanged** | `checker/context.rs::is_query_owned_module` still contains the legacy compatibility exclusion. |
| SC1-01 | missing explicit formation outcome algebra | **already resolved** | Current semantic type formation has explicit terminal outcomes. Regression-only. |
| SC1-02 | invalid kinds recover to `Type` | **already resolved** | Current lowering has `InvalidKindSyntax`; no reimplementation. |
| SC1-03 | open record tail erased | **already resolved** | Current annotation lowering rejects unsupported open tail rather than closing it silently. |
| SC1-04 | source type lambdas fail to bind parameters | **already resolved** | Scoped bound-node lowering exists. |
| SC1-05 | `Self` always instance-side | **already resolved** | `TypeFormationSite` carries owner/side. |
| SC1-06 | row binder reaches `parameter_form` assertion | **already resolved** | Row-domain lexical binding exists. |
| SC1-07 | type alias not a module declaration | **already resolved** | `InterfaceBuilder` includes `Statement::TypeAlias`; semantic alias products exist. Alias dependency/publication release claims remain gated. |
| SC1-08 | type-formation failures collapse to coarse Unknown | **already resolved** | Explicit outcome algebra exists. |
| B-01 | current build/CI blocker | **out of scope** | Toolchain/CI work explicitly excluded. |
| C-01 | canonical `Result` / `Ordering` can receive duplicate runtime roots | **retain with revised implementation** | General ADT root allocation remains fresh; typing materialization uses primordial classes. Registry early return does not detect root conflicts. |
| C-02 | semantic prelude leaks runtime/internal declarations | **retain with revised implementation** | Resolver still has synthetic prelude-module lookup plus unconditional Universe-key fallback. |
| C-03 | canonical `Option<T>` signatures unsound/overly Dynamic | **retain with revised implementation** | Current `option.ph` still has Dynamic/unsound signatures. |
| H-01 | Universe module resolution bypasses normal exposure/relative rules | **retain with revised implementation** | Both absolute exposure and relative Universe resolution remain split. |
| H-02 | bootstrap executes entire Universe corpus | **split** | Correctness requirement: derive initialization from canonical dependency graph. Performance/laziness requirement: defer; current root imports nearly the whole catalog, so closure-from-root alone gives little meaningful laziness. |
| H-03 | fallback match lowering guesses builtins by spelling | **retain with revised implementation** | `match_expr.rs` still maps `Some/None`, `Ok/Error/Err`, and Ordering spellings to canonical owners. |
| H-04 | lightweight `Result` is missing | **out of scope** | Separate future representation plan, explicitly excluded. |
| H-05 | stable metadata uses `proj#N` and zero fingerprint | **retain with revised implementation** | `stable_identity.rs` still serializes `ResolvedProjectId::to_string()` and `Fingerprint128::ZERO`. Context-aware source/revision identity is required. |
| M-01 | semantic Universe baseline rebuilt inline per session | **defer** | Still true, but extraction is deliberately last. |
| M-02 | Universe `__package__` differs from ordinary package semantics | **retain unchanged** | `builtin_materialize.rs` still assigns parent package to nested Universe package and module alike. |
| M-03 | non-root Universe interface exports too broadly | **retain with revised implementation** | `BuiltinInterfaceBuilder` still auto-exports every non-root declaration. Fold into source-interface authority slice. |
| M-04 | canonical `Error` coexists with `Err` compatibility semantics | **retain with revised implementation** | Production match fallback still recognizes `Err`; fixtures still need classification. Method names such as `mapErr` remain valid. |
| M-05 | checked-in Cargo config/toolchain posture | **out of scope** | Explicitly excluded by task scope. |

## 3.2 Historical task disposition

Every historical task is assigned one of the exact amendment dispositions required by this review: `retain unchanged`, `retain with revised implementation`, `split`, `supersede`, `already resolved`, `stale`, `defer`, or `out of scope`.

### Pre-SC-1 stabilization plan

| Historical task | Disposition | Amendment |
|---|---|---|
| Task 1 — repair canonical Rust toolchain / CI | **out of scope** | Toolchain, Cargo, CI toolchain, and compiler-version work is explicitly excluded. |
| Task 2 — replace synthetic Universe-root declarations with canonical export targets | **retain with revised implementation** | Execute in source/interface authority Slice 1; also remove non-root source-authority bypass. |
| Task 3 — replace leaf-name Universe fallback with canonical prelude map | **retain with revised implementation** | Build the map only after canonical source declaration identities are authoritative. |
| Task 4 — source authority plus reusable semantic baseline | **split** | Source-authority correction executes early; baseline extraction is deferred to final Slice 9. |
| Task 5 — remove runtime generic arity inference by class name | **retain with revised implementation** | Keep the identity fix, but discard the old user-`List` fixture because kernel class names are reserved. |
| Task 6 — repair Option contracts | **supersede** | The broader C-03 slice validates and fixes the complete Option generic surface together. |
| Task 7 — enforce `expose` traversal for Universe imports | **retain with revised implementation** | Implement through one provider-neutral module resolver in MOD-01. |
| Task 8 — unify Universe dependency-path resolution | **retain with revised implementation** | Bootstrap must consume the canonical module resolver/dependency graph, not duplicate path meaning. |
| Task 9 — qualified type resolution | **retain with revised implementation** | Traverse every component or fail closed; never reinterpret `root.a.Leaf` as `root.Leaf`. |
| Task 10 — source/native conformance | **retain with revised implementation** | Convert it into a true source/interface/semantic/runtime identity proof; unused parameters are evidence of the missing authority. |
| Task 11 — native source indexing by full owner identity | **retain with revised implementation** | Expand to all current post-resolution runtime class lookups that still use leaf names. |
| Task 12 — stale `core` / `std` metadata fixtures | **retain unchanged** | Fixture correctness only; keep generic schema-compat namespaces when intentionally non-Phalcom. |
| Task 13 — dead legacy-`core` semantic dependency exclusion | **retain unchanged** | Remove after a query-dependency regression proves no live canonical module depends on it. |
| Task 14 — reject open record tails | **already resolved** | Current lowering rejects unsupported tails instead of erasing them; retain regression coverage only. |
| Task 15 — type aliases as module declarations | **already resolved** | `InterfaceBuilder` now collects `Statement::TypeAlias`; remaining alias release coverage stays in the owning SC-1 plan. |
| Task 16 — certify already-fixed SC-1 invariants | **retain unchanged** | Verification-only task; no production rewrite unless current regression tests expose a real regression. |

### Post-Universe correctness remediation plan

| Historical task | Disposition | Amendment |
|---|---|---|
| Task 1 — portable/stable Cargo configuration | **out of scope** | Explicitly excluded. |
| Task 2 — formatting and red/green verification baseline | **supersede** | Replaced by this amendment's current target-registration and verification plan; no toolchain remediation is included. |
| Task 3 — reuse primordial root classes for canonical Universe enums | **retain with revised implementation** | Root reuse is necessary but insufficient; add exact owner identity and conflict detection. |
| Task 4 — cross-registry runtime identity assertions | **retain with revised implementation** | Expand to typing registry, module/global class identity, behavior superclass, reflection, dispatch, and pattern runtime paths. |
| Task 5 — canonical semantic prelude type map | **retain with revised implementation** | Derive from canonical source declaration identity plus explicit prelude policy. |
| Task 6 — store/use prelude map for editor visibility | **retain with revised implementation** | Same map must drive checker and visible-symbol/editor semantics. |
| Task 7 — replace unsound/Dynamic Option signatures | **retain with revised implementation** | Validate the complete generic source surface through the semantic pipeline. |
| Task 8 — Option semantic/runtime regression coverage | **retain with revised implementation** | Re-ground tests under current unified `semantic` and `core` targets. |
| Task 9 — provider-neutral package-surface validation | **retain with revised implementation** | One `ModuleResolver` authority for resolved projects and Universe. |
| Task 10 — relative Universe dependency resolution | **retain with revised implementation** | Eliminate bootstrap-only path semantics; runtime consumes canonical module edges. |
| Task 11 — match fallback string guessing | **retain with revised implementation** | Variant fallback must return a structured semantic-lowering-required error; also promote current ambiguity/visibility tests. |
| Task 12 — stable project lookup infrastructure | **retain with revised implementation** | Use existing `ProjectSourceIdentity` / stable project infrastructure and add/consume revision authority as needed. |
| Task 13 — context-aware stable metadata conversion | **retain with revised implementation** | Must prove graph-order independence and source-revision sensitivity, not merely replace `proj#N` with a path string. |
| Task 14 — stop non-root Universe auto-export | **retain with revised implementation** | Fold into source/interface authority Slice 1; ordinary source export rules apply to Universe. |
| Task 15 — align Universe `__package__` | **retain unchanged** | Re-ground the tests under the registered `core` target. |
| Task 16 — remove `Err` as canonical Result variant identity | **retain with revised implementation** | Remove production/fixture aliasing only where it claims canonical Result; keep API names such as `mapErr`. |
| Task 17 — execute only runtime-reachable Universe modules | **split** | Retain canonical dependency-graph/discovery-vs-execution correctness; defer root-topology/laziness redesign because current root imports nearly the whole catalog. |
| Task 18 — full verification gate | **retain with revised implementation** | Use current registered test targets, current baseline failures, and no toolchain-specific remediation. |
| Task 19 — extract immutable Universe semantic baseline | **defer** | Execute only after full correctness gates; extraction must be behavior-preserving. |

---

# 4. Current blocker matrix

## A-01 — Canonical Universe source and interface authority

**Severity:** Critical

**Current behavior:**

`phalcom-modules/src/builtin_interface.rs::BuiltinInterfaceBuilder::build_from_parsed` still modifies ordinary source-derived interfaces in two identity-affecting ways:

1. Universe root overlays exported native catalog bindings by manufacturing a local declaration when the root source does not own one;
2. non-root Universe modules automatically export every declaration, even when ordinary source export semantics would not.

Separately, `phalcom-semantic/src/session.rs::SemanticWorkspaceSession::with_workspace` still begins ordinary Universe type authority from `bootstrap_universe_declarations(...)`, then parses source later to augment the model.

**Invariant violated:**

```text
actual source declaration owner
== linked export target owner
== semantic DeclarationId owner
```

and ordinary source declarations must be derived from source, not silently invented by native catalog metadata.

**Authoritative source of truth:**

- `InterfaceBuilder::build` over actual Universe source;
- source-owned `DeclarationId` from `ModuleId + declaration name`;
- canonical Universe `.ph` declarations and their generic syntax.

**Affected files and symbols:**

- `phalcom-modules/src/builtin_interface.rs::BuiltinInterfaceBuilder::build_from_parsed`
- `phalcom-modules/src/interface.rs::UnlinkedExportTarget`
- `phalcom-modules/src/linker.rs::LinkContext::resolve_export`
- `phalcom-semantic/src/session.rs::SemanticWorkspaceSession::with_workspace`
- `phalcom-semantic/src/declarations.rs::bootstrap_universe_declarations`
- `phalcom-core/core/universe/src/package.ph`

**Dependency ordering:** First implementation slice. C-02, C-01 conformance, native identity cleanup, and late baseline extraction depend on this authority being unambiguous.

**Required implementation slice:**

1. Introduce a canonical export target capable of carrying a real `DeclarationId`/`SymbolId` instead of forging root-local ownership.
2. Replace root synthetic declaration injection with root aliases/re-exports to canonical source owners.
3. Remove non-root “export every declaration” post-processing. If language source declarations are meant to be public by default, implement that rule once in ordinary `InterfaceBuilder`, not in a Universe wrapper.
4. Build a source declaration catalog for ordinary Universe language declarations.
5. Change semantic bootstrap so source declarations/generic kinds are formed from source-owned identities first.
6. Keep runtime-support-only catalog entries in a separate internal/native layer; do not pretend they are ordinary source declarations.
7. Make native metadata mismatches fail conformance instead of silently defining language meaning.

**Focused validation:**

- root `Int`, `List`, `Option`, `Result`, `Ordering` aliases resolve to their actual owner module;
- root alias does not appear as a local declaration/global owner;
- non-root interface export set equals ordinary `InterfaceBuilder` semantics;
- direct owner import and root convenience import yield the same linked `SymbolId`;
- a source-only Universe declaration can be semantically predeclared without adding a `UniverseKey`.

**Acceptance condition:** No ordinary canonical Universe declaration has two declaration owners, and source/native conformance fails on owner/kind/generic mismatch.

**Regression risks:** accidental eager dependencies from root aliases; breaking root convenience imports; accidentally making runtime-support classes lexical; store-local `TypeId` leakage into a process-global catalog.

---

## C-01 — One canonical runtime root per canonical enum declaration

**Severity:** Critical

**Current behavior:**

- `semantic_lowering.rs` assigns `NativeOption` only to canonical `Option`; `Result` and `Ordering` remain `General`.
- `vm/adt.rs::allocate_general_enum_classes` allocates a fresh root `ClassId` for every `General` enum.
- `materialize.rs` binds canonical Universe typing metadata using primordial `self.universe.classes.resolve(binding.key)`.
- `RuntimeAdtRegistry::register_enum_with_representation` silently returns an existing enum for a repeated semantic owner without checking that the supplied root/representation agrees.

Therefore one `DeclarationId` can still be associated with distinct runtime roots, and “first registration wins” can hide the conflict.

**Invariant violated:**

```text
canonical DeclarationId(Result)
    -> exactly one root ClassId
```

The same must hold for `Option` and `Ordering`, and all of these authorities must agree:

```text
UniverseClasses / primordial table
RuntimeAdtRegistry
TypingRegistry nominal binding
module global/class registry
variant behavior-class superclass
reflection
runtime dispatch / pattern matching
```

**Authoritative source of truth:** Canonical `DeclarationId`; runtime root realization is a deterministic projection of that identity.

**Affected files and symbols:**

- `phalcom-core/src/vm/adt.rs::{class_binding_for_enum,allocate_general_enum_classes,register_enum_from_spec}`
- `phalcom-core/src/adt.rs::RuntimeAdtRegistry::register_enum_with_representation`
- `phalcom-core/src/modules/semantic_lowering.rs::lower_enum`
- `phalcom-core/src/modules/materialize.rs` semantic metadata registration
- `phalcom-core/src/vm/api.rs`
- `phalcom-core/src/vm/associated.rs::resolve_declaration_class`
- `phalcom-core/tests/native_adt_runtime.rs` (included by registered `core` target)

**Dependency ordering:** Source declaration identity (A-01) first. Runtime physical `Result` representation is explicitly not a dependency.

**Required implementation slice:**

1. Resolve canonical enum root binding by full `DeclarationId`, not by name.
2. For canonical `Option`, `Result`, and `Ordering`, reuse the primordial root class while retaining current physical representation (`Result`/`Ordering` stay `General`).
3. Split root selection from hidden variant behavior-class allocation.
4. Make repeated registry registration idempotent only when owner, root, and representation agree.
5. Return a structured runtime/internal error when an existing semantic owner is re-registered with a different root or representation; never silently accept the mismatch.
6. Add a read-only query for test/runtime assertion of enum root by `DeclarationId` if needed; do not expose mutable registry internals.
7. Add assertions at semantic metadata materialization boundaries that canonical Universe ADTs already agree with the primordial table.

**Focused validation:**

- canonical `Option`, `Result`, `Ordering` root identity equality;
- conflicting duplicate registration is rejected;
- hidden case behavior class has the authoritative root as superclass;
- runtime `value_is_variant`, `case_behavior_class`, reflection class, and typing nominal binding agree;
- ordinary user enum still receives its own root.

**Acceptance condition:** There is no execution path capable of registering or reflecting a second root `ClassId` for the same canonical enum declaration.

**Regression risks:** confusing semantic enum identity with physical representation; accidentally giving a user enum native Option storage; breaking variant behavior inheritance; masking conflict with registry early return.

---

## C-02 — Explicit semantic prelude authority

**Severity:** Critical

**Current behavior:**

`LinkedTypeResolver::resolve_type_name` still attempts:

1. local declaration;
2. selective import;
3. linked re-export/current namespace;
4. a declaration synthesized under `prelude_module`;
5. `UniverseKey::from_name(root)` followed by canonical Universe declaration lookup.

The final lookup ignores explicit `prelude` policy and can expose runtime-support or non-prelude declarations.

**Invariant violated:** Prelude visibility is a policy over canonical source declarations, not “all names recognized by the native catalog.”

**Authoritative source of truth:** canonical source `DeclarationId` plus explicit prelude policy from the native/Universe binding policy table.

**Affected files and symbols:**

- `phalcom-semantic/src/resolver.rs::LinkedTypeResolver`
- `phalcom-semantic/src/session.rs::SemanticWorkspaceSession`
- source-index/editor visible-symbol construction
- `phalcom-native-meta::UNIVERSE_BINDINGS`

**Dependency ordering:** After A-01; before Option semantic validation and broad editor/semantic gates.

**Required implementation slice:**

1. Add one canonical `PreludeTypeMap` mapping bare source name to canonical source-owned `DeclarationId`.
2. Derive entries only from explicit prelude policy and valid source declarations; runtime-support classes are excluded unless separately ratified as source names.
3. Remove synthetic `prelude_module` declaration reconstruction and unconditional `UniverseKey::from_name(root)` fallback from lexical type resolution.
4. Preserve resolution precedence: local -> selective import -> linked namespace/re-export -> prelude.
5. Store/share the prelude product within the semantic session.
6. Make editor visible-symbol logic consume the same map; associated variant completion remains separate.

**Focused validation:**

Positive bare resolution:

```text
Object, Int, Bool, String, Option, Result, List, Map
```

Negative without import according to policy:

```text
Nil, Some, None, Behavior, Metaclass, Method, Family
```

Then explicitly import an exported non-prelude declaration and prove it resolves.

**Acceptance condition:** Every implicit type name comes from one explicit prelude map, and no runtime-support declaration enters lexical type lookup through catalog spelling.

**Regression risks:** changing local/import shadow precedence; accidentally removing legal explicit imports; editor/completion divergence from checker lookup.

---

## C-03 — Sound canonical `Option<T>` source contracts

**Severity:** Critical

**Current behavior:** Current `option.ph` still declares Dynamic or unsound surfaces including:

```text
match(some: Dynamic, none: Dynamic) -> Dynamic
map(_ f) -> Self | Option<Dynamic>
flatMap(_ f) -> Self | Option<Dynamic>
filter(_ pred) -> Self | Option<Dynamic>
unwrapOr<U>(_ default: U) -> U
okOr<E>(_ err) -> Result<T, E>
```

`unwrapOr<U>` is formally unsound because the `Some` arm returns `T`, not arbitrary `U`; `okOr<E>` does not type its error argument.

**Invariant violated:** A public generic source signature must describe every runtime return value and parameter relationship.

**Authoritative source of truth:** `phalcom-core/core/universe/src/option/option.ph` validated through the normal semantic pipeline.

**Affected files and symbols:**

- `phalcom-core/core/universe/src/option/option.ph`
- semantic callable-signature publication/conformance tests
- native Option surface descriptors only if a native signature actually exists for the affected method

**Dependency ordering:** C-02 first so the canonical source can resolve its types without prelude leakage.

**Required implementation slice:** Adopt source-level contracts equivalent to:

```phalcom
match<R>(some: (value: T) -> R, none: () -> R) -> R
map<U>(_ f: (value: T) -> U) -> Option<U>
flatMap<U>(_ f: (value: T) -> Option<U>) -> Option<U>
filter(_ pred: (value: T) -> Bool) -> Option<T>
unwrapOr(_ default: T) -> T
okOr<E>(_ err: E) -> Result<T, E>
```

`orElse` must be re-grounded against its current runtime/source behavior; if it is lazy fallback, the intended form is:

```phalcom
orElse(_ f: () -> Option<T>) -> Option<T>
```

Do not introduce a new heterogeneous `unwrapOr` bound design in this amendment.

**Focused validation:**

- semantic signature assertions for each method;
- positive `Option<Int>.map` -> `Option<String>` specialization;
- positive `flatMap` specialization;
- negative `unwrapOr` fallback type mismatch;
- `okOr` error parameter constrains `E`;
- canonical Universe Option source analyzes without signature errors;
- runtime behavior tests still produce the same values.

**Acceptance condition:** No canonical Option method claims a return or parameter type that runtime execution can violate, and no generic information is discarded to Dynamic where the relationship is expressible.

**Regression risks:** stale native metadata disagreement; accidentally changing runtime behavior while changing only contracts; introducing unsupported inference requirements.

---

## R-01 — Qualified type resolution must preserve the full path

**Severity:** High

**Current behavior:** `LinkedTypeResolver::resolve_type_name` resolves the root module alias, then constructs a declaration from `members.last()`, silently discarding intermediate qualification components.

**Invariant violated:** Every written qualification component must contribute to resolution or cause an explicit failure.

**Authoritative source of truth:** linked module/export graph.

**Affected files and symbols:**

- `phalcom-semantic/src/resolver.rs::LinkedTypeResolver::resolve_type_name`
- linked module/export query products
- semantic imported-resolution tests

**Dependency ordering:** After A-01 and module identity cleanup; before match ambiguity promotion.

**Required implementation slice:**

1. Delete the `members.last()` reinterpretation.
2. If current linked namespace products can traverse nested namespaces/modules, resolve every intermediate component.
3. If they cannot yet represent deep member traversal, fail closed for `members.len() > 1` rather than inventing `root::leaf` meaning.
4. Record dependency edges for every resolved intermediate owner if traversal is implemented.

**Focused validation:**

- ordinary one-hop qualified type still resolves;
- malformed intermediate component cannot resolve merely because the final leaf exists;
- two paths with the same leaf but different intermediate owners never collapse;
- unsupported deep qualification produces the correct unresolved/invalid diagnostic rather than a false success.

**Acceptance condition:** No semantic resolution path drops user-written qualification components.

**Regression risks:** breaking valid aliases; introducing another namespace traversal outside `phalcom-modules`; under-recording incremental dependencies.

---

## MATCH-01 — Match lowering must be `VariantId`-driven and fail closed

**Severity:** High

**Current behavior:** `phalcom-core/src/compiler/lib/match_expr.rs` fallback still guesses canonical ADT identity from spelling:

```text
Some / None -> Option
Ok / Error / Err -> Result
Less / Equal / Greater / Unordered -> Ordering
```

It then manufactures `VariantId` and projection information.

Current ignored semantic tests also record genuine missing ambiguity/visibility behavior:

- `match_diag_02_ambiguous_variant_has_owner_candidates`
- `match_diag_03_inaccessible_variant_points_at_explicit_name`
- `match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate`

**Invariant violated:** only semantic resolution can choose a variant owner; compiler fallback may not guess meaning by spelling.

**Authoritative source of truth:** semantic `VariantId` / `MatchLoweringSpec`.

**Affected files and symbols:**

- `phalcom-core/src/compiler/lib/match_expr.rs::compile_match_expr`
- `phalcom-core/src/compiler/lib/error.rs`
- semantic match resolution/diagnostics
- canonical Result source/fixtures

**Dependency ordering:** A-01 and R-01 first. C-01 root fix may proceed in parallel but must be green before cross-layer match/runtime acceptance.

**Required implementation slice:**

1. Normal analyzed compilation uses the semantic lowering product unchanged.
2. Variant pattern fallback without semantic lowering returns a structured compiler error such as `VariantPatternRequiresSemanticLowering(range)`.
3. Delete all builtin owner/variant string guessing from fallback.
4. Remove `"Err"` as a canonical Result variant alias. Canonical source is `Error`.
5. Keep method names such as `isErr`, `mapErr`, `unwrapErr`, `expectErr` unchanged.
6. Promote the three current genuine ambiguity/visibility resolution tests once their exact semantic products/diagnostics are implemented.

**Focused validation:**

- local enum with `Ok` / `Error` uses local `VariantId` in analyzed compilation;
- local `Option::Some` and `Ordering::Equal` equivalents do not bind to Universe builtins by spelling;
- compiler path intentionally invoked without semantic variant lowering returns the structured error;
- `Result::Err` is unresolved unless a user-defined declaration explicitly provides it;
- ambiguous contextual owner yields no arbitrary candidate and reports candidate owners.

**Acceptance condition:** Production compiler code contains no branch that chooses a canonical builtin variant owner from a constructor string.

**Regression risks:** unintentionally breaking non-variant match fallback; changing API method terminology; moving semantic resolution into compiler rather than requiring the semantic product.

---

## ID-01 — Eliminate post-resolution leaf-name runtime class reconstruction

**Severity:** High

**Current behavior:** The historical F-12 finding remains and has expanded current impact:

- `native/source.rs::{index_class,index_enum}` call `UniverseKey::from_name(...)`;
- `vm/api.rs::resolve_builtin_class_name` maps a leaf name to a Universe class;
- `vm/associated.rs::resolve_declaration_class` falls back from a canonical `DeclarationId` to name-only builtin resolution;
- `modules/materialize.rs` hydrates Universe symbols through `resolve_builtin_class_name(&symbol.name)`;
- `modules/context.rs` has the same name-only class materialization seam;
- `typing/inspect.rs::class_constructor_arity` derives generic arity from the class display name.

The original “user-defined `List`” source regression is incorrectly scoped on current code: `CompilerError::ClassReservedName` deliberately rejects kernel class names such as `List`, `Object`, and `Number` for non-core modules.

**Invariant violated:** a resolved declaration/class may not lose its owner path and be re-identified by leaf spelling.

**Authoritative source of truth:** `DeclarationId` or exact `(ModuleId, declaration name)`; for primordial Universe classes, a canonical declaration-to-`UniverseKey`/`ClassId` map.

**Affected files and symbols:**

- `phalcom-core/src/native/source.rs`
- `phalcom-core/src/vm/api.rs::resolve_builtin_class_name`
- `phalcom-core/src/vm/associated.rs::resolve_declaration_class`
- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/modules/context.rs`
- `phalcom-core/src/typing/inspect.rs::class_constructor_arity`

**Dependency ordering:** A-01 and C-01 first; native/source conformance can then validate the new identity seam.

**Required implementation slice:**

1. Add/centralize exact Universe declaration-to-runtime-class lookup, e.g. conceptually:

   ```text
   resolve_universe_declaration_class(&DeclarationId) -> Option<ClassId>
   ```

   It must verify owner module/path and name.
2. Post-resolution consumers pass `DeclarationId`/`SymbolId`, not a leaf name.
3. Native source association validates `UniverseKey::source_path()` plus declaration name.
4. `class_constructor_arity` resolves canonical semantic declaration/generic signature from `ClassId`; canonical Universe classes may use exact `ClassId -> UniverseKey`, never display-name matching.
5. Retain `ClassReservedName` behavior. Do not weaken language name reservation to create a regression fixture.

**Focused validation:**

- wrong-path `@native` class with a familiar leaf does not associate with a native key;
- a runtime/internal class with the same display name as a generic builtin does not inherit builtin arity;
- a legal non-kernel same-leaf collision across modules remains distinct;
- reserved kernel-name rejection remains green;
- every canonical Universe declaration class lookup verifies the owner path.

**Acceptance condition:** Search-based audit finds no post-resolution runtime/type identity decision that depends only on `.name` / `UniverseKey::from_name`.

**Regression risks:** removing legitimate pre-resolution source-name lookup; confusing display/presentation names with identity; forcing every runtime call site to know native catalog internals instead of using one canonical helper.

---

## NATIVE-01 — Source/native conformance must be a real cross-layer proof

**Severity:** High

**Current behavior:** `validate_native_surface_conformance` accepts resolver/module context as `_resolver` / `_current_module` but derives native declaration identity directly through canonical-key helpers. It can prove native metadata is internally coherent without proving that actual source/interface ownership and signatures match it.

**Invariant violated:** Native metadata is an attachment to source authority, not a parallel source definition.

**Authoritative source of truth:** source interface/declaration products plus semantic generic/callable signatures.

**Affected files and symbols:**

- `phalcom-semantic/src/core_surface/conformance.rs::validate_native_surface_conformance`
- canonical Universe source declaration catalog from A-01
- native surface registration/import
- runtime class correspondence from C-01 / ID-01

**Dependency ordering:** Semantic half after A-01/C-02/C-03; runtime class half after C-01/ID-01.

**Required implementation slice:**

For every native-backed ordinary source declaration, verify:

```text
UniverseKey.source_path + name
    == source ModuleId + declaration name
    == semantic DeclarationId
```

and compare:

- declaration kind;
- generic arity and parameter kinds;
- source superclass relationship where applicable;
- callable selector and dispatch side;
- formal parameter/return type contract;
- source/native ownership/provenance;
- runtime root class for primordial/native nominal declarations.

Runtime-support-only rows must use an explicit exemption/category rather than being smuggled through ordinary source conformance.

**Focused validation:** Inject owner-path mismatch, generic-arity mismatch, wrong parameter type, wrong dispatch side, and wrong runtime root; each must fail conformance with a specific mismatch.

**Acceptance condition:** Removing `_` from resolver/context parameters reflects actual use, and conformance proves agreement with the source-derived model rather than merely catalog self-consistency.

**Regression risks:** circular dependency during bootstrap; making source declarations depend on native metadata before conformance; over-constraining intentional runtime-support-only rows.

---

## MOD-01 — Universe must use canonical package/import resolution semantics

**Severity:** High

**Current behavior:**

- Absolute `ImportRootTarget::Universe` provider lookup returns before ordinary package `expose` traversal.
- Relative import logic has a `ResolvedProjectId`-specific importer requirement, so `ProjectIdentity::Universe` cannot use the ordinary relative path path.
- `phalcom-core/src/native/source.rs` contains a separate `universe_dependency_target` resolver for bootstrap dependencies.

**Invariant violated:** Universe has a special source provider, not special language import semantics.

**Authoritative source of truth:** `phalcom-modules::ModuleResolver` and source-derived package interfaces.

**Affected files and symbols:**

- `phalcom-modules/src/resolver.rs::ModuleResolver::resolve_import_with_trace`
- package surface/exposure helpers
- `phalcom-core/src/native/source.rs::universe_dependency_target` and initialization dependency construction
- Universe provider/interface loading

**Dependency ordering:** A-01 first; bootstrap graph work consumes this slice.

**Required implementation slice:**

1. Generalize module locate/package-surface helpers over `ProjectIdentity` or an equivalent provider abstraction.
2. Route Universe absolute import targets through the same hierarchical exposure validation used for external resolved projects.
3. Make relative import semantics work for Universe module/package identity without inventing a fake `ResolvedProjectId`.
4. Reuse the module-layer resolved dependency edges in runtime bootstrap; delete semantic reimplementation in `native/source.rs` where possible.
5. Keep provider-specific source loading behind the common resolver boundary.

**Focused validation:**

- exposed Universe child imports successfully;
- non-exposed child fails at the correct intermediate package;
- relative Universe import resolves;
- re-export/selective import resolves through the same authority;
- module query/LSP-visible child set and compiler resolution agree;
- bootstrap dependency target equals canonical module resolver target.

**Acceptance condition:** There is one import/path meaning for Universe and user projects, differing only in source provider.

**Regression risks:** recursive interface loads; changing project-root semantics; bootstrap cycle/order behavior; accidentally forcing runtime initialization during source discovery.

---

## META-01 — Durable metadata identity must be context-aware and revision-sensitive

**Severity:** High

**Current behavior:** `phalcom-semantic/src/metadata/stable_identity.rs` still converts:

```text
ProjectIdentity::Resolved(id)
```

to a `SourceArtifact` using:

```text
logical_uri = id.to_string()   // proj#N
source_fingerprint = ZERO
```

`metadata/export.rs` calls free conversion functions without resolved-project/source revision context.

Current `phalcom-modules` already has the right identity substrate:

- `ResolvedProject.source_identity: ProjectSourceIdentity`
- `ProjectUniverse::get_project`
- `StableProjectKey { source: ProjectSourceIdentity }`
- `ProjectRevisionFingerprint([u8; 16])`

but the metadata export path does not propagate it.

**Invariant violated:** persisted identity cannot depend on graph allocation order and must distinguish revisions when the schema claims a source fingerprint.

**Authoritative source of truth:** resolved project source identity plus the semantic/module source revision fingerprint authority available at snapshot/export time.

**Affected files and symbols:**

- `phalcom-semantic/src/metadata/stable_identity.rs`
- `phalcom-semantic/src/metadata/export.rs`
- `phalcom-modules/src/project.rs::ResolvedProject / ProjectUniverse`
- `phalcom-modules/src/identity.rs::{StableProjectKey,ProjectRevisionFingerprint}`
- semantic snapshot/source fingerprint products

**Dependency ordering:** Module/source authority first; may be implemented after MATCH-01 but before broad reflection certification.

**Required implementation slice:**

1. Replace context-free resolved-project conversion with a `StableIdentityContext` (name may vary) carrying `&ProjectUniverse` and the source/revision fingerprint authority needed by export.
2. Derive logical project identity from `ResolvedProject.source_identity` or canonical logical artifact identity, never `ResolvedProjectId`.
3. Derive the revision fingerprint from stable source/module inputs sorted by stable module key, or consume the current canonical project-revision fingerprint product if one exists by implementation time.
4. **Do not fake revision identity by hashing only the project path.** A source edit must change the fingerprint.
5. Construct the conversion context once in metadata export and thread it through module/declaration/callable/field conversion.
6. Universe remains a stable builtin `universe` identity; synthetic executions remain explicitly session-local.

**Focused validation:**

- same project loaded under different graph allocation order -> equal stable declaration refs;
- distinct project roots with same module/declaration names -> different stable refs;
- source edit -> revision/source fingerprint changes;
- Universe stable identity remains `universe`;
- synthetic identity remains non-durable/session-scoped.

**Acceptance condition:** No durable metadata path serializes `proj#N` or zero revision fingerprint for a resolved source project.

**Regression risks:** non-deterministic filesystem traversal in fingerprint composition; path-only fingerprinting; cross-platform path normalization; threading source context too late after identity was already serialized.

---

## PKG-01 — Universe `__package__` must match ordinary package semantics

**Severity:** Medium

**Current behavior:** `phalcom-core/src/modules/builtin_materialize.rs` assigns `module.package = parent` to nested Universe packages and ordinary modules alike. Ordinary materialization treats a package's language-visible `__package__` as the package itself.

**Invariant violated:** the same `ModuleKind` has the same lexical `__package__` semantics regardless of source provider.

**Authoritative source of truth:** ordinary module/package materialization semantics.

**Affected files and symbols:**

- `phalcom-core/src/modules/builtin_materialize.rs`
- ordinary `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/tests/core/modules/universe.rs`

**Dependency ordering:** After MOD-01 source/module topology is stable.

**Required implementation slice:**

- Universe root package: `__package__ == Some(root package)`;
- nested Universe package: `__package__ == Some(self)`;
- ordinary Universe module: `__package__ == Some(parent package)`.

Compare object identity, not names.

**Focused validation:** registered `core` target Universe/package tests plus ordinary user-package counterpart.

**Acceptance condition:** package intrinsic behavior is source-provider-neutral.

**Regression risks:** confusing internal parent-package pointer with language-visible `__package__` value.

---

## BOOT-01 — Canonical runtime dependency graph versus meaningful laziness

**Severity:** Medium correctness; deferred performance objective

**Current behavior:** `NativeSourceIndex::initialization_order()` topologically sorts the complete indexed Universe unit set, and `VM::run_universe_modules()` executes that order. The root Universe `package.ph` currently imports/exports most top-level catalog areas, including object, scalar, errors, callable, option, concurrency, collections, reflection, I/O, filesystem/path, text/regex/json, math/random/time/process/net/concurrent/testing.

**Invariant violated:** discovery and execution should be separate products, and execution dependencies should come from canonical module resolution.

**Authoritative source of truth:** canonical module dependency graph from MOD-01.

**Affected files and symbols:**

- `phalcom-core/src/native/source.rs::{initialization_order,dependency construction}`
- `phalcom-core/src/vm/bootstrap.rs::run_universe_modules`
- `phalcom-core/core/universe/src/package.ph`

**Dependency ordering:** MOD-01 first.

**Required implementation slice:**

1. Separate complete source catalog/index from executable initialization set/order.
2. Compute execution order from canonical module dependency edges.
3. Measure/record the root-reachable closure under the current `package.ph` before claiming startup laziness.
4. If the root closure is nearly the full catalog, **do not redesign root imports in this amendment**. Record meaningful laziness as a deferred library/bootstrap-topology task.
5. Preserve deterministic topological ordering and cycle diagnostics.

**Focused validation:** graph edge parity with module resolver; deterministic closure/order; source catalog still contains uninitialized modules for tooling/reflection discovery.

**Acceptance condition:** bootstrap no longer has a second interpretation of dependency paths. This amendment does not require a large startup reduction if current root topology defeats laziness.

**Regression risks:** conflating discoverability with initialization; initializing a needed primordial module too late; claiming a performance win without measuring reachable closure.

---

## LEG-01 — Legacy `core` / `std` compatibility residue

**Severity:** Low to Medium cleanup, but required for identity certification

**Current behavior:**

- `checker/context.rs::is_query_owned_module` excludes a one-component Universe path named `core` from query ownership;
- reflection/type-metadata fixtures still contain canonical-Phalcom examples using `StableProjectRef::Builtin { namespace: "core" | "std" }`.

**Invariant violated:** retired compatibility identities must not remain in canonical correctness tests or dependency ownership unless a deliberate compatibility feature still consumes them.

**Authoritative source of truth:** `ProjectIdentity::Universe` and production stable Universe identity.

**Affected files and symbols:**

- `phalcom-semantic/src/checker/context.rs::is_query_owned_module`
- `phalcom-core/tests/core/reflection/reflection.rs`
- `phalcom-core/tests/core/reflection/type_metadata.rs`

**Dependency ordering:** After source authority and metadata identity changes.

**Required implementation slice:**

1. Prove no canonical live semantic module requires the `universe.core` query-ownership exception, then delete it.
2. Change canonical Phalcom reflection fixtures to `universe` identity.
3. Keep intentionally generic schema-compat fixture namespaces if they are testing wire format rather than Phalcom canonical identity.

**Focused validation:** query dependency capture for Universe module; reflection/stable-metadata round-trip using `universe`; grep for production `core`/`std` identity reconstruction.

**Acceptance condition:** remaining `core`/`std` hits are either historical docs or explicit legacy-rejection/schema-compat tests, not canonical production identity.

**Regression risks:** deleting a compatibility sentinel before replacing the staged DB product it hid; over-editing generic schema tests.

---

## GATE-01 — Keyword argument type checking skips a mismatch

**Severity:** High semantic correctness

**Current behavior:** The active semantic gate currently fails `foundations::expression_engine::test_keyword_argument_mismatch_detected`: a value such as `"invalid"` passed to a keyword parameter declared as `Int` produces no `ArgumentMismatch`.

**Invariant violated:** every argument lane—positional, labeled/keyword, rest—must validate actual value knowledge against the selected formal parameter contract.

**Authoritative source of truth:** selected callable semantic signature and call argument binding result.

**Affected files and symbols:**

- `phalcom-semantic/tests/semantic/foundations/expression_engine.rs::test_keyword_argument_mismatch_detected`
- `phalcom-semantic/src/checker/call.rs` argument-to-formal validation path using `ctx.apply_assignability(..., DiagnosticCode::ArgumentMismatch, ...)`
- selector/argument binding helpers called by that path

**Dependency ordering:** Independent of Universe identity. Treat as a known baseline failure; repair before the final full semantic gate. It may be implemented early if file ownership does not conflict.

**Required implementation slice:** Trace the labeled argument binding path and ensure the same assignability/refutation logic used for positional arguments is invoked after selector/label matching. Do not special-case the fixture.

**Focused validation:** failing test above plus a positive labeled `Int` argument and a mixed positional+labeled call that proves each lane validates its own formal.

**Acceptance condition:** active semantic gate no longer has this failure and the diagnostic owner/range points to the incompatible argument.

**Regression risks:** duplicate diagnostics, validating against the wrong overload, losing labels during generic substitution.

---

## GATE-02 — Panic-abort regression oracle is not a valid in-process assertion

**Severity:** Medium verification blocker; not a production semantic bug

**Current behavior:** `support::regressions::union_expectation_rejects_wrong_structural_members` expects to catch an intentional panic with `catch_unwind`, but the workspace's panic behavior aborts the process.

**Invariant violated:** the test harness must be capable of observing the failure mode it claims to verify.

**Authoritative source of truth:** regression oracle behavior, not Cargo configuration.

**Affected files and symbols:**

- `phalcom-semantic/tests/semantic/support/regressions.rs::union_expectation_rejects_wrong_structural_members`
- the assertion/oracle helper invoked by that test
- existing child-process fail-fast regression support in the same semantic test suite, if the child-process route is selected

**Dependency ordering:** Independent. Fix the harness before the final semantic gate. No Cargo or toolchain configuration change is permitted by this amendment.

**Required implementation slice:** Prefer a non-panicking oracle API that returns a structured failure result. If the production/foundation oracle intentionally must abort/panic, assert it in a child process, following the repository's existing fail-fast child-process pattern.

**Focused validation:** named regression test runs to completion and proves the intended wrong-structural-member rejection without depending on in-process unwinding.

**Acceptance condition:** the active semantic gate has no harness-only failure.

**Regression risks:** weakening the oracle so the test no longer proves rejection; converting a fail-fast invariant into silent recovery.

---

# 5. Known current gated failures that do not belong to this amendment

Do not silently absorb the entire ignored-test ledger.

The following current failures are real but belong to other semantic/parser programs unless an implementation slice above directly touches them:

- callable-family residual classification (`match_exh_06_callable_family_leaves_singleton_residual`);
- nested GADT branch proof (`match_gadt_06_nested_gadt_proof_is_branch_local`);
- parser/fixture prerequisites for tuple/list/string-literal/abrupt match patterns;
- broad golden pipeline gaps for Families, variance, type-lambda parsing/constraints, row/effect contracts, recursive fixed points, and mixed pipelines;
- placeholder cross-module visibility tests with no product verdict;
- the stale incremental visibility test whose fixture changes a literal instead of visibility;
- intentional fail-fast child tests;
- performance-only or vacuous ignored tests.

Three current ignored match tests **are** promoted into MATCH-01 because they exercise the exact ambiguity/visibility authority this amendment changes:

```text
match_diag_02_ambiguous_variant_has_owner_candidates
match_diag_03_inaccessible_variant_points_at_explicit_name
match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate
```

Alias tests that pass when forced remain evidence, not release completion. Do not reimplement basic alias declaration surfaces merely to unignore them; promote them only when their dependency/publication oracles are genuinely complete under the owning SC-1 plan.

---

# 6. Amended execution plan

The order below is normative. Do not hide prerequisites inside later tasks.

## Slice 0 — Capture current local evidence before edits

**Files changed:** none.

- [ ] Record `git rev-parse HEAD` and `git status --short`.
- [ ] If `graphify-out/` exists, run the repository-required graphify query for Universe declaration identity, semantic prelude, runtime ADT registration, and stable metadata before grep-based exploration.
- [ ] Record the two active semantic baseline failures separately from amendment regressions.
- [ ] Record ignored-test counts/status from the local `COVERAGE_LEDGER.md` if they differ from this audit.
- [ ] Verify the amendment target filename does not already exist before creating/using it.

**Exit evidence:** local SHA, dirty-state snapshot, graphify result or “graph absent,” baseline test failure list.

---

## Slice 1 — Canonical Universe source/interface authority

**Closes/revises:** F-02, F-04 semantic half, M-03; prerequisites for F-10.

**Files:**

- `phalcom-modules/src/interface.rs`
- `phalcom-modules/src/builtin_interface.rs`
- `phalcom-modules/src/linker.rs`
- add/factor module-layer Universe source declaration catalog in the established module structure
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/declarations.rs`
- `phalcom-semantic/src/core_surface/conformance.rs` semantic half

- [ ] Write failing module tests proving root aliases preserve actual owner and non-root Universe does not gain implicit exports.
- [ ] Add canonical declaration export target and linker preservation.
- [ ] Remove synthetic root declaration ownership and non-root export-all behavior.
- [ ] Build ordinary Universe declaration/generic shells from source.
- [ ] Keep runtime-support-only metadata in a distinct bootstrap/native attachment path.
- [ ] Make source/native declaration owner/kind/generic mismatch a conformance failure.
- [ ] Run focused module tests, then full `phalcom-modules` gate.

**Slice acceptance:** linked source owner and semantic `DeclarationId` agree for canonical Universe declarations before moving on.

---

## Slice 2 — Canonical runtime enum roots and post-resolution class identity

**Closes/revises:** C-01; begins F-05/F-12 cleanup.

**Files:**

- `phalcom-core/src/vm/adt.rs`
- `phalcom-core/src/adt.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/vm/api.rs`
- `phalcom-core/src/vm/associated.rs`
- `phalcom-core/tests/native_adt_runtime.rs`
- relevant reflection tests under `phalcom-core/tests/core/reflection/`

- [ ] Write failing canonical root identity tests for Option/Result/Ordering under the registered `core` target.
- [ ] Write a conflicting duplicate registry registration test.
- [ ] Make canonical enum root selection full-identity-based and reuse primordial roots.
- [ ] Keep Result/Ordering physical representation `General`.
- [ ] Make registry repeat registration validate root/representation agreement.
- [ ] Replace post-resolution class lookup in this slice with exact declaration lookup where required for ADT identity.
- [ ] Assert typing registry/materializer root equality.
- [ ] Run focused core identity/reflection tests, then adjacent `core` target filters.

**Slice acceptance:** one `DeclarationId` has one root `ClassId` across ADT registry, primordial table, typing registry, behavior classes, and reflection.

---

## Slice 3 — Prelude authority and Option soundness

**Closes/revises:** C-02/F-03, C-03/F-06.

**Files:**

- `phalcom-semantic/src/prelude.rs` or the repository-appropriate new module
- `phalcom-semantic/src/resolver.rs`
- `phalcom-semantic/src/session.rs`
- semantic editor/source-index visibility path
- `phalcom-core/core/universe/src/option/option.ph`
- semantic native/source conformance tests

- [ ] Add prelude positive/negative tests first.
- [ ] Build one canonical `PreludeTypeMap` from source identity + explicit policy.
- [ ] Remove synthetic/fallback prelude reconstruction.
- [ ] Reuse the same map in editor visible symbols.
- [ ] Add semantic call-site tests for the intended Option generic contracts.
- [ ] Change `option.ph` contracts only after the tests express the desired typing.
- [ ] Run focused semantic prelude/Option tests and native conformance.

**Slice acceptance:** non-prelude runtime/internal names cannot leak bare, and every canonical Option signature is sound through actual semantic analysis.

---

## Slice 4 — Resolver and match identity

**Closes/revises:** F-09, H-03, M-04; promotes three current match gaps.

**Files:**

- `phalcom-semantic/src/resolver.rs`
- semantic match resolution/diagnostics
- `phalcom-core/src/compiler/lib/match_expr.rs`
- `phalcom-core/src/compiler/lib/error.rs`
- canonical Result-model fixtures that still use `Err`

- [ ] Add a deep-qualification leaf-collision negative test.
- [ ] Remove path-component dropping; traverse fully or fail closed.
- [ ] Activate/repair ambiguity and inaccessible-variant diagnostic tests.
- [ ] Add compiler fallback red test proving variant fallback cannot guess a builtin.
- [ ] Make fallback return structured semantic-lowering-required error.
- [ ] Delete builtin variant string inference and canonical `Err` aliasing.
- [ ] Run semantic match filters and core compiler/match filters.

**Slice acceptance:** semantic resolution chooses exact variant identity; compiler never chooses a builtin variant from spelling.

---

## Slice 5 — Metadata identity, native source/runtime identity, and full native conformance

**Closes/revises:** H-05, F-05, F-10, F-12.

**Files:**

- `phalcom-core/src/native/source.rs`
- `phalcom-core/src/vm/api.rs`
- `phalcom-core/src/vm/associated.rs`
- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/modules/context.rs`
- `phalcom-core/src/typing/inspect.rs`
- `phalcom-semantic/src/core_surface/conformance.rs`
- `phalcom-semantic/src/metadata/stable_identity.rs`
- `phalcom-semantic/src/metadata/export.rs`
- `phalcom-modules/src/project.rs`
- `phalcom-modules/src/identity.rs`

- [ ] Add wrong-owner native source association test.
- [ ] Add generic arity test that does not violate reserved kernel-name rules.
- [ ] Centralize declaration-aware Universe runtime class lookup; replace post-resolution name-only uses.
- [ ] Strengthen conformance through source declaration -> semantic declaration -> runtime class.
- [ ] Add stable identity context with project/source/revision information.
- [ ] Add graph-allocation-order and source-edit fingerprint tests.
- [ ] Run focused modules identity, semantic metadata/conformance, and core reflection tests.

**Slice acceptance:** no post-resolution class/type identity is reconstructed from a leaf name, and durable metadata contains neither `proj#N` nor zero resolved-project revision fingerprint.

---

## Slice 6 — Exports/module exposure, package intrinsics, bootstrap graph correctness, and legacy cleanup

**Closes/revises:** H-01/F-07/F-08, M-02, H-02 correctness half, F-13, legacy-core sentinel.

**Files:**

- `phalcom-modules/src/resolver.rs`
- `phalcom-core/src/native/source.rs`
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-semantic/src/checker/context.rs`
- reflection/type metadata fixtures

- [ ] Add Universe exposure and relative-import tests.
- [ ] Generalize resolver semantics to Universe provider.
- [ ] Make bootstrap consume canonical dependency targets/edges.
- [ ] Separate complete catalog from initialization set/order.
- [ ] Measure root-reachable closure; do not promise significant laziness if root imports most of the catalog.
- [ ] Align Universe `__package__` with ordinary packages.
- [ ] Remove the dead `universe.core` query-ownership sentinel after a dependency regression proves it unnecessary.
- [ ] Replace canonical reflection fixture `core`/`std` namespaces with `universe`.
- [ ] Run modules resolver/package tests and core Universe/reflection filters.

**Slice acceptance:** package/import semantics are source-provider-neutral, bootstrap uses canonical dependency meaning, and retired canonical identities are absent from production/current fixtures.

---

## Slice 7 — Repair known active semantic gate blockers

**Closes:** GATE-01, GATE-02.

These are current-HEAD blockers missing from the historical correctness plans.

- [ ] Fix labeled/keyword argument contract validation using the ordinary call-checking relation.
- [ ] Replace the in-process panic-catching regression oracle with structured failure or child-process assertion.
- [ ] Run each named failing test alone.
- [ ] Run the entire semantic target and record the new pass/fail/ignored counts.

**Slice acceptance:** neither of the two recorded current active failures remains; any new failure is treated as a regression until proven otherwise.

---

## Slice 8 — Full correctness gates

Do not declare SC-1 correctness remediation complete from focused tests.

- [ ] Run all module tests.
- [ ] Run the complete semantic integration target.
- [ ] Run the complete core integration target.
- [ ] Run workspace compile/test gates using the repository's current default toolchain invocation; this plan does not alter toolchain configuration.
- [ ] Run the search/deletion gates below.
- [ ] Compare ignored-test classification with the baseline ledger. No ignored test becomes “fixed” merely because it happens to pass when forced; promotion needs a complete oracle.
- [ ] Record expected baseline/deferred failures separately from regressions.

Do not move to baseline extraction until these gates are clean for the amendment-owned scope.

---

## Slice 9 — Baseline extraction only after behavior is proven

**Closes/revises:** F-11 / M-01.

**Files:**

- create `phalcom-semantic/src/universe_baseline.rs` or the repository-appropriate equivalent
- `phalcom-semantic/src/session.rs`
- incremental/workspace tests

This is a **behavior-preserving extraction**, not another semantic redesign.

- [ ] Snapshot the proven pre-extraction semantic outputs/tests.
- [ ] Move the validated source/native baseline construction into a first-class baseline object.
- [ ] Do not move mutable workspace source/diagnostic/body state into the baseline.
- [ ] Do not create a global `OnceLock` containing store-local `TypeId`s unless the `TypeStore` itself becomes a safely shared immutable authority.
- [ ] Prefer a source-stable baseline plus store-local instantiation if raw semantic IDs are store-relative.
- [ ] Re-run the exact same focused/package/full gates and structural cold/incremental comparisons.

**Slice acceptance:** behavior and structural semantic products are unchanged by extraction, and session construction no longer contains the long ad-hoc bootstrap sequence.

---

# 7. Test and verification plan

## 7.1 First principle: focused red/green, then package, then broad

For each amendment slice:

1. add the narrow regression first;
2. run it and record the pre-fix failure;
3. implement only that slice;
4. re-run the focused test;
5. run the adjacent package target;
6. run cross-layer tests where the invariant spans crates;
7. only at phase boundaries run broad workspace gates.

A focused test that passes without first demonstrating the historical/current failure is useful coverage but not evidence that the patch fixed the defect.

## 7.2 Re-grounded focused commands

### `phalcom-modules`

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_catalog -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test interface_extraction -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test integration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test identity_foundation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test package_info_semantics -- --nocapture
```

### `phalcom-semantic`

All focused semantic tests run through the registered `semantic` target:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic <filter> -- --nocapture
```

Required filters/scenarios after their slices exist:

```text
prelude
native_conformance
imported_resolution
metadata
matching
match_diag_02_ambiguous_variant_has_owner_candidates
match_diag_03_inaccessible_variant_points_at_explicit_name
match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate
foundations::expression_engine::test_keyword_argument_mismatch_detected
support::regressions::union_expectation_rejects_wrong_structural_members
type_annotations
authority_boundaries
```

Do not invent a separate `--test metadata` or `--test native_conformance`; those modules live under the unified `semantic` target unless Cargo registration changes before implementation.

### `phalcom-core`

All integration filters run through the registered `core` target:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <filter> -- --nocapture
```

Required filters/scenarios:

```text
native_adt_runtime
universe
reflection
type_metadata
option
match
semantic_boundary
package
```

Do not use `--test native_adt_runtime`; it is included by `tests/core/mod.rs` rather than registered independently.

## 7.3 Package and workspace gates

After focused tests for a slice:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --nocapture
```

At the full correctness boundary:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
```

Use the repository's current default toolchain. This amendment neither selects nor modifies the toolchain.

`cargo fmt --all -- --check` and `git diff --check` remain ordinary quality gates, not toolchain-remediation tasks.

## 7.4 Expected existing failures versus regressions

Before implementation, classify the local semantic target result into:

**Known active baseline failures:**

- keyword argument mismatch diagnostic bug;
- panic-abort regression harness bug.

**Known ignored/gated, amendment-promoted:**

- match ambiguous owner candidates;
- inaccessible explicit variant diagnostic;
- ambiguous contextual owner fail-closed behavior.

**Known ignored/gated, not amendment-owned:** callable-family residual, nested GADT proof, parser prerequisite fixtures, broad golden semantic gaps, placeholder/vacuous visibility tests, performance-only tests, intentional fail-fast child.

**Regression:** any test that was active and passing at local Slice-0 baseline and fails after an amendment slice, unless the expected behavior is intentionally changed and the amendment explicitly updates its oracle.

Never “fix” a regression by ignoring the test.

## 7.5 Required new/strengthened fixtures and negative tests

### Identity and source authority

- root convenience import versus direct owner import -> same `SymbolId`;
- wrong owner path with same leaf does not match native catalog;
- source-only Universe declaration works without native key;
- root alias creates no local owner declaration.

### Runtime root identity

- Option/Result/Ordering root equality across all registries;
- conflicting repeat ADT registration returns error;
- variant behavior superclass equals authoritative root;
- user enum remains independent.

### Prelude

- bare positive prelude list;
- bare negative runtime/non-prelude list;
- explicit import of exported non-prelude name succeeds.

### Option

- generic `map`/`flatMap` specialization;
- `unwrapOr` incompatible fallback rejected;
- `okOr` error argument constrains `E`;
- runtime results unchanged.

### Qualified resolution

Create two nested owners with the same final leaf. A bad/missing intermediate qualifier must not “jump” to the final leaf in the root module.

### Match identity

- local enum uses `Ok`, `Error`, `Some`, or `Equal` spellings without binding to Universe;
- fallback variant pattern with no semantic lowering is a structured compile error;
- `Result::Err` unresolved for canonical Result.

### Runtime leaf-name cleanup / F-05

Do **not** use `class List {}` as the primary regression because current language rules reserve kernel class names. Instead:

1. keep a test proving the reserved-name rule itself;
2. construct an internal/runtime class or legal non-kernel same-leaf identity with a familiar display name and prove arity/native association depends on canonical identity, not the display string.

### Stable metadata

- graph allocation order independence;
- same names, different source roots -> different refs;
- source revision edit -> changed fingerprint;
- Universe builtin identity stable;
- synthetic session identity explicitly ephemeral.

### Package/bootstrap

- exposed/non-exposed Universe paths;
- relative Universe import;
- root/nested package/module `__package__` identity;
- bootstrap dependency edge parity with module resolver;
- deterministic reachable order.

### Active semantic gate

- bad labeled argument -> `ArgumentMismatch`;
- good labeled argument -> accepted;
- mixed argument lanes validate the correct formal;
- union-structural regression assertion works without in-process unwind dependency.

## 7.6 Search/deletion gates

After the relevant slices, inspect every hit rather than blindly requiring zero strings.

### Semantic prelude fallback

```bash
rg -n 'UniverseKey::from_name\(root\)|prelude_module' phalcom-semantic/src
```

Expected: no lexical-prelude reconstruction path.

### Match builtin spelling semantics

```bash
rg -n 'v\.base == "(Some|None|Ok|Error|Err|Less|Equal|Greater|Unordered)"|"Err"\s*=>.*Result' phalcom-core/src/compiler
```

Expected: no semantic owner/variant construction by spelling.

### Durable metadata

```bash
rg -n 'res_id\.to_string\(\)|source_fingerprint:\s*Fingerprint128::ZERO' phalcom-semantic/src/metadata
```

Expected: no resolved-project durable identity path.

### Post-resolution runtime name authority

```bash
rg -n 'resolve_builtin_class_name|UniverseKey::from_name' \
  phalcom-core/src/native \
  phalcom-core/src/vm \
  phalcom-core/src/modules \
  phalcom-core/src/typing
```

Every remaining hit must be classified. Pre-resolution source/catalog lookup may be legitimate; a resolved `DeclarationId`/`SymbolId` consumer falling back to a leaf name is not.

### Legacy canonical identity

```bash
rg -n 'namespace:\s*"core"|namespace:\s*"std"|components\[0\].as_str\(\) == "core"' \
  phalcom-core/tests/core/reflection \
  phalcom-semantic/src \
  phalcom-core/src
```

Remaining hits must be explicit schema-compat/legacy rejection, not current identity.

### Already-resolved SC-1 regressions

```bash
rg -n 'tail:\s*_' phalcom-semantic/src/types
rg -n 'KindSyntax::Invalid.*KindId::TYPE' phalcom-semantic/src
rg -n 'ScopedTypeData::Free\(body' phalcom-semantic/src/types
```

Expected: no reintroduction of historical shortcuts.

## 7.7 Release-completion rule

SC-1 correctness remediation is not release-complete until all of these are true:

- focused red/green evidence exists for every retained blocker;
- module, semantic, and core package gates pass;
- the two active baseline failures are fixed rather than normalized as expected failures;
- amendment-promoted ignored match tests are active and pass;
- no amendment slice introduces a new active failure;
- cross-layer root identity tests compare actual IDs, not names;
- stable metadata tests prove graph-order independence and revision sensitivity;
- search gates contain no unexplained identity reconstruction paths;
- workspace compile/test gates pass;
- late baseline extraction, if performed, is proven behavior-preserving;
- unrelated gated parser/Family/GADT/golden gaps remain explicitly recorded rather than falsely claimed complete.

---

# 8. Baseline extraction design constraint

The historical plans disagree subtly about when to create `UniverseSemanticBaseline`. This amendment resolves the ordering:

```text
source/interface authority
        ↓
semantic prelude + canonical contracts
        ↓
runtime identity + native conformance
        ↓
module/path/bootstrap correctness
        ↓
full correctness gates
        ↓
behavior-preserving baseline extraction
```

Do not create the baseline first and then fix behavior inside it. That would hide the source of failures and risk turning stale native metadata into a frozen authority.

The extracted product may contain store-local semantic IDs only if its `TypeStore` ownership makes those IDs valid for every consumer. Otherwise split:

```text
UniverseSourceBaseline
    stable source/module/declaration facts
        ↓ instantiate into store
UniverseSemanticBaseline
    store-local TypeId/kind/dispatch products
```

This is an implementation constraint, not permission to redesign `TypeStore` in this amendment.

---

# 9. Implementation handoff

## 9.1 Documents to read, in this order

1. repository `AGENTS.md`;
2. this amendment;
3. `sc-blockers-1.md` and `sc-blockers-2.md` as historical evidence;
4. `phalcom-pre-sc1-stabilization-patch-grade-implementation-plan.md`;
5. `phalcom-post-universe-review-correctness-remediation-plan.md`;
6. `SC-1-type-formation-kinds-generics-technical-spec.md` for the type-formation laws that are already implemented and must not regress;
7. current `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md`.

**Do not use the lightweight Result representation plan for this implementation.** The physical representation work is a separate future program.

## 9.2 Exact amendment tasks to execute

Execute Slices 0 through 8 in order. Execute Slice 9 only when Slice 8 broad correctness evidence is clean.

Do not revive historical Tasks 14/15 (open-record-tail rejection / alias declaration collection) as implementation tasks unless current local HEAD has regressed relative to the audited state.

## 9.3 Files/symbols to inspect first

Start with:

```text
phalcom-modules/src/builtin_interface.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/resolver.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/core_surface/conformance.rs
phalcom-core/src/vm/adt.rs
phalcom-core/src/adt.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/native/source.rs
phalcom-core/src/vm/api.rs
phalcom-core/src/vm/associated.rs
phalcom-core/src/typing/inspect.rs
phalcom-core/src/compiler/lib/match_expr.rs
phalcom-semantic/src/metadata/stable_identity.rs
phalcom-semantic/src/metadata/export.rs
phalcom-modules/src/project.rs
phalcom-modules/src/identity.rs
phalcom-core/src/modules/builtin_materialize.rs
phalcom-core/core/universe/src/option/option.ph
phalcom-core/core/universe/src/package.ph
phalcom-semantic/src/checker/context.rs
```

## 9.4 Prohibited scope expansion

Do not:

- modify Cargo/toolchain/CI configuration as part of this amendment;
- implement native/lightweight `Result` storage;
- redesign `Value` metadata;
- convert all ignored semantic tests into this plan;
- implement unrelated parser features simply to make forced-ignored tests run;
- finish general Family or GADT semantics here;
- change the kernel reserved-class-name policy to manufacture an F-05 fixture;
- extract `UniverseSemanticBaseline` before correctness is proven;
- create another import resolver for Universe;
- create another semantic identity map keyed only by spelling;
- treat comments/refactors as correctness fixes without regression evidence;
- edit the two historical plans.

## 9.5 First verification command

After local preflight/graphify and before any implementation edit, run the module authority baseline first:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_catalog -- --nocapture
```

Then capture the current semantic active gate so known failures are fixed in place rather than misattributed later:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --nocapture
```

Do not move on without recording exact results.

## 9.6 Completion evidence required before the next slice

Every slice handoff must include:

1. local HEAD and `git status --short` showing unrelated work preserved;
2. exact focused test command and its red-before/green-after result for newly fixed behavior;
3. adjacent package-gate result;
4. any cross-layer ID assertions introduced by the slice;
5. new/changed failure list compared with Slice-0 baseline;
6. search-gate output for the forbidden old path owned by that slice;
7. explicit list of deferred/gated failures that were not touched;
8. no claim of release completion based solely on focused tests.

For C-01 specifically, evidence must include actual `ClassId` equality across authorities. Registry-only tests are insufficient.

For H-05 specifically, evidence must show identity stability under different resolved-project graph allocation order **and** fingerprint change after source revision; replacing `proj#N` with a path string alone is insufficient.

For Slice 9 baseline extraction, evidence must include before/after structural semantic equivalence and the same broad test results.

---

# 10. Final amendment acceptance checklist

- [ ] Toolchain/Cargo/CI/compiler-version remediation is absent from this implementation plan.
- [ ] Lightweight/native Result representation is absent from this implementation plan.
- [ ] Historical findings are classified rather than blindly repeated.
- [ ] SC1-03 open-record-tail rejection is marked already resolved on audited HEAD.
- [ ] SC1-07 alias declaration-surface publication is marked already resolved, with release coverage separately gated.
- [ ] C-01 includes registry conflict detection and end-to-end runtime identity, not merely primordial root reuse.
- [ ] F-05 keeps the correctness issue but rejects the stale user-`List` fixture because kernel names are reserved.
- [ ] F-12 is expanded to current post-resolution leaf-name runtime paths.
- [ ] F-10 is treated as a real conformance authority bug, not dead-parameter cleanup.
- [ ] H-02 separates dependency-graph correctness from deferred meaningful laziness.
- [ ] H-05 propagates project/source/revision context rather than replacing one string locally.
- [ ] Current active semantic-gate failures missing from the historical plans are included.
- [ ] Current ignored failures are classified and unrelated ones are not smuggled into scope.
- [ ] All test commands use currently registered test targets.
- [ ] `phalcom-core/tests/native_adt_runtime.rs` is invoked through `--test core`, not as a nonexistent independent target.
- [ ] Baseline extraction is last and behavior-preserving.
- [ ] No retained blocker can be accepted solely from a registry-local or focused-only test.

