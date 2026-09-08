# Phalcom Type-System Completion — Repository-Grounded Patch-Grade Implementation Plan

**Plan type:** Checkpoint-driven, semantic-risk-aware, patch-grade implementation handoff  
**Repository:** `aureat/phalcom-lang`  
**Baseline:** remote `main` at `e17f2733f98cb20e2a8ead5794d75ca647a950ce`  
**Prepared:** 2026-09-04  
**Local working-tree status:** unavailable in this planning session; repository tools exposed the remote repository state only. The implementing agent must perform the drift/baseline check described in C0 before editing.  
**Companion specification:** `phalcom-type-system-completion-technical-spec.md`

---

# 1. Implementation Program

This program completes the remaining planned generic/GADT semantics while preserving the repository's existing canonical type, generic-call, Family, exact-case, metadata, and incremental architecture.

The program is intentionally ordered around semantic dependencies rather than syntax categories.

The dominant sequence is:

```text
repair generic application ownership
        ↓
complete callable-generic surfaces
        ↓
preserve applied receiver/class-side specialization
        ↓
add variant-local generic declarations
        ↓
route generic variant construction through ordinary inference
        ↓
add scoped rigid variables
        ↓
open GADT constructor locals existentially
        ↓
enforce non-escape/exact-case rules
        ↓
close native/generated/publication parity
        ↓
certify incremental and whole-repository behavior
```

The most important architectural constraint is:

> Do not generalize the current `merge_constructor_generic_signatures` pattern. It mixes declaration-owned and callable-owned `TypeParameterId`s under one callable-owned `GenericSignature`, which conflicts with `GenericSignature::validate_publishable`. Replace that application-time need with explicit multi-domain generic application composition before implementing generic variants.

---

# 2. Repository State and Evidence Model

## 2.1 Baseline inspected

Remote repository evidence was inspected at:

```text
aureat/phalcom-lang
e17f2733f98cb20e2a8ead5794d75ca647a950ce
```

The planning session did **not** have a mounted local checkout and therefore cannot state:

- the implementer's active branch;
- local uncommitted changes;
- whether local HEAD has advanced beyond this revision.

Before C0 begins, the implementing agent must record those facts in the implementation state file.

## 2.2 Primary current evidence anchors

```text
phalcom-ast/src/ast.rs
    EnumDef
    VariantDecl
    GenericParameterSyntax
    WhereClauseSyntax
    GetterDef
    SetterDef
    IndexMethodDef

phalcom-ast/src/parser.rs
    parse_generic_parameters
    parse_where_clause
    class/enum member parsing
    variant parsing

phalcom-ast/src/selector.rs
    setter/index/variant selector formation

phalcom-semantic/src/identity.rs
    VariantId
    VariantConstructorId
    CallableOwnerId
    CallableId
    InvocationTargetId

phalcom-semantic/src/types/parameter.rs
    TypeParameterOwner
    GenericSignature
    GenericSignature::validate_publishable

phalcom-semantic/src/checker/declaration_signature.rs
    callable_id_for_syntax
    resolve_callable_local_generics
    merge_constructor_generic_signatures
    declaration_type_level_bindings_for_side

phalcom-semantic/src/checker/call.rs
    generic callable application / expected-result path

phalcom-semantic/src/checker/inference.rs
    InferenceSession and inference terms

phalcom-semantic/src/enum_semantics.rs
    VariantConstructorSignature
    VariantInfo
    EnumInfo

phalcom-semantic/src/checker/enum_declaration.rs
    build_enum_semantics and variant products

phalcom-semantic/src/types/case_environment.rs
    CaseTypeEnvironment

phalcom-semantic/src/checker/gadt_proof.rs
    solve_gadt_branch_proof
    equality unification

phalcom-semantic/src/checker/exhaustiveness.rs
    exact-case/pattern-space integration

phalcom-semantic/src/types/store.rs
    TypeData::Applied
    TypeData::ExactCase

phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/instantiation.rs

phalcom-semantic/src/types/family.rs
phalcom-semantic/src/checker/associated.rs

phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/metadata/export.rs

phalcom-type-meta/src/type_node.rs
    TypeNode::Applied

phalcom-core/src/typing/reify.rs
    applied type descriptor reification
```

## 2.3 Existing evidence that must remain green

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
phalcom-semantic/tests/semantic/capabilities/getters.rs
phalcom-semantic/tests/semantic/adts/
phalcom-semantic/tests/semantic/families/
phalcom-semantic/tests/semantic/incremental/
```

The committed SC-4/4.5 state records generic getter parser/semantic tests as passing. That historical evidence is useful but must not be treated as a substitute for rerunning the focused baseline in C0.

---

# 3. Source-of-Truth Register

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Generic declaration binder identity | `TypeParameterId` owned by `TypeParameterOwner::Declaration` | type application, receiver specialization, metadata | parameter name string |
| Callable-local binder identity | `TypeParameterId` owned by `TypeParameterOwner::Callable(CallableId)` | call inference, constraints, metadata, Families | syntax-specific generic-variable IDs |
| Callable identity | `CallableId` | signature table, dispatch, source index, Families | specialized callable IDs per type argument |
| Variant identity | `VariantId` | `VariantConstructorId`, exact case, Family, pattern matching | variant name alone |
| Variant executable construction identity | `VariantConstructorId` / `InvocationTargetId::VariantConstructor` | lowering/runtime | a newly invented generic variant runtime ID |
| Variant generic binder owner | canonical variant-constructor `CallableId` derived from exact `VariantId` | `GenericSignature`, metadata | `TypeParameterOwner::Variant` |
| Declaration-index GADT equations | `CaseTypeEnvironment` | branch proof solver | branch-local global generic mutation |
| Branch-local constructor existential identity | new scoped rigid variable / `CaseInstantiation` product | payload typing, local constraints, branch proofs | inference metavariable or ordinary declaration parameter |
| Exact case | `TypeData::ExactCase { variant, enum_type }` | narrowing, relation, exhaustiveness | exact case containing durable branch rigid IDs |
| Applied semantic type | canonical `TypeData::Applied` / metadata `TypeNode::Applied` | receiver specialization, reflection, future runtime state | erased declaration name plus ad-hoc argument vector |
| Applied class-side invocation owner | new durable invocation specialization product | lowering/future runtime class storage | selector identity or callable cloning |
| Durable generic metadata | `GenericSignature::validate_publishable` + metadata exporter | runtime reflection/tooling | unchecked merged constructor signature |
| Incremental semantics | structural fingerprints of canonical declaration/products | query invalidation | raw solver/rigid allocation IDs |

---

# 4. Tempting Wrong Fixes — Global Guardrails

Do not:

1. change `GenericSignature::validate_publishable` to permit mixed owners merely to preserve `merge_constructor_generic_signatures`;
2. create `ConstructorGenericSignature`, `GetterGenericSignature`, `VariantGenericVariable`, or index-specific solver types;
3. encode generic arguments into selectors or `CallableId` identity;
4. create specialized semantic declarations per instantiation;
5. represent GADT skolems as ordinary inference variables;
6. represent branch-local skolems as declaration-owned `TypeParameterId`s;
7. make `CaseTypeEnvironment` the owner of fresh branch-local existential IDs;
8. publish rigid IDs into canonical metadata/fingerprints;
9. solve existential escape by checking only branch return expressions;
10. change exact-case canonical identity merely to store hidden constructor locals;
11. implement applied generic class storage in this program's runtime phase; only static/lowering prerequisites are in scope;
12. restore the older ambient `Type.currentApplication` design as the static mechanism for applied class-side state;
13. fix a failed generic call by weakening a type to `Dynamic` or `Unknown` unless the existing semantic contract explicitly requires that boundary;
14. add a native-only generic inference path;
15. re-test every task independently when one checkpoint-level semantic test proves the integrated invariant.

---

# 5. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–3 | Baseline is reproducible; generic getter status and constructor publication defect are pinned to the active checkout | focused parser/getter/generic/metadata baseline; repository-state record | crate/workspace tests |
| C1 | 4–8 | Declaration-owned and callable-owned generic domains compose in one application without corrupting canonical `GenericSignature` ownership | generic constructor inference + metadata publication + hostile owner test | ADT, native, workspace |
| C2 | 9–15 | Setter and index getter/setter surfaces are first-class callable-local generic declarations using the same solver and stable selectors | AST/parser regressions + semantic positive/conflict tests | full semantic crate |
| C3 | 16–21 | Generic class-side declarations form parameterized templates and every inferred/explicit applied receiver survives invocation specialization | class-side specialization tests + raw unsaturated hostile case + constructor receiver publication | runtime storage execution |
| C4 | 22–28 | Variant-local generic binders/`where` clauses are canonical callable-owned declaration products | parser/AST + enum semantic ownership + metadata/fingerprint tests | construction/matching |
| C5 | 29–34 | Generic variant construction and retained Families use ordinary generic application across enum + variant domains | construction inference + expected result/conflict + Family target tests | existential elimination |
| C6 | 35–41 | Scoped rigid variables exist as a reusable local type category and cannot be solved/exported as flexible variables | unit relation/substitution tests + publication rejection + alpha-equivalence helper tests | pattern integration |
| C7 | 42–48 | Full GADT elimination opens constructor-local parameters existentially and reuses existing index proof machinery | shared-rigid/freshness/index-proof/bound hostile tests | escape/exact-case completion |
| C8 | 49–55 | Branch-local rigids cannot escape and exact cases reconstruct hidden locals freshly | return/assignment/wrapper/closure/widening/exact-case hostile tests | native + broad incremental |
| C9 | 56–61 | Native/generated/intrinsic inputs and durable metadata can express the completed callable/variant generic semantics without parallel authority | native parity + generated accessor compilation + metadata round-trip | full incremental/workspace |
| C10 | 62–67 | Incremental analysis, cold analysis, Families, GADT proofs, applied receiver products, and publication are equivalent under edits | focused incremental suites + semantic crate + core protected suites | final workspace delivery gates |
| Final Gate | — | Repository-wide delivery readiness with obsolete mechanism checks | fmt/check/test/clippy + negative searches + deferred-evidence audit | none |

---

# 6. Implementation State File Protocol

Create or reuse one concise state document for the program, preferably adjacent to the implementation specs, for example:

```text
docs/impl/semantic/type-system-completion/IMPLEMENTATION-STATE.md
```

If repository convention dictates another existing state file, reuse it rather than adding a duplicate.

After every checkpoint record:

```md
## Established invariants

- I-001: ...

## Decisions

- D-001: ...

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion evidence

- command → result

## Deferred gates

- command → destination checkpoint

## Active incident

None.

## Next resume action

Begin C<N> Task <M>.
```

Do not record private chain-of-thought or verbose scratch work. Record only reviewable facts, decisions, anchors, and evidence.

---

# 7. Repository Drift Protocol

At entry to every checkpoint:

1. verify the primary files still exist;
2. verify the named symbols still own the expected responsibility;
3. inspect changes from prior checkpoints that modify shared APIs;
4. search for newly added callers where fanout matters;
5. adapt mechanical edits to the current repository;
6. do not silently alter the semantic design.

Escalate as `PLAN DRIFT` if:

- `GenericSignature` ownership semantics changed materially;
- `CallableId`/`CallableOwnerId` was redesigned;
- variants no longer use `VariantConstructorId`/`InvocationTargetId` as inspected;
- exact-case representation changed;
- a new rigid/existential abstraction already landed;
- applied class-side runtime state has already been implemented under a different canonical model.

---

# Checkpoint C0 — Baseline, Repository Drift, and Defect Pinning

Tasks:
- Task 1 — Record active checkout and relevant drift.
- Task 2 — Reconfirm generic getter implementation and baseline generic-call behavior.
- Task 3 — Add/identify a focused regression exposing constructor generic-signature publication ownership.

Why this is a checkpoint:

The implementation program changes the shared generic application boundary. Before touching it, the implementing agent needs one reproducible baseline proving that current getter semantics are real and that the constructor merge issue is understood in the active checkout. This prevents later failures from being misclassified as new regressions.

Entry conditions:
- repository is buildable enough to run focused `phalcom-ast`/`phalcom-semantic` tests;
- companion technical specification is available;
- no implementation edits from this program have begun.

Working set:

Primary:
- `phalcom-semantic/src/checker/declaration_signature.rs` — constructor signature composition and getter generics;
- `phalcom-semantic/src/types/parameter.rs` — publication ownership invariant;
- `phalcom-semantic/src/metadata/export.rs` — generic signature export;
- `phalcom-semantic/tests/semantic/capabilities/getters.rs`;
- `phalcom-semantic/tests/semantic/integration/metadata.rs`;
- `phalcom-ast/tests/` getter/parser tests.

Secondary — inspect only if evidence requires it:
- `phalcom-semantic/src/checker/call.rs`;
- `phalcom-semantic/tests/semantic/foundations/generic_application.rs`.

Out of scope for this checkpoint:
- setter/index parser changes;
- variant parser changes;
- GADT changes;
- runtime class storage.

Semantic contract established by this checkpoint:
- the active checkout's generic getter behavior is known and reproducible;
- the current constructor generic publication shape is pinned to actual code;
- later checkpoints have a trustworthy baseline.

Semantic risks:
- planning against stale remote assumptions;
- mistaking a local regression for intended current behavior;
- weakening publication validation instead of repairing constructor composition.

Hostile cases:
- a generic constructor on a generic declaration must not be "fixed" by disabling owner validation;
- a local checkout that still rejects generic getters must be classified as drift rather than prompting reimplementation on top of stale code.

Required evidence:

1. `git status --short && git branch --show-current && git rev-parse HEAD` — records local branch, HEAD, and working-tree state. If the execution environment lacks git metadata, record that limitation.
2. `cargo test -p phalcom-ast --test integration getter -- --nocapture` — proves the active parser accepts the current generic-getter surface and preserves getter grammar.
3. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::getters -- --nocapture` — proves current generic getter semantics.
4. `cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_application -- --nocapture` — proves expected-result and bound/defaulting generic laws before shared changes.
5. Focused metadata regression added or identified in `semantic::integration::metadata` that attempts to export the generic-constructor callable signature on a generic declaration. It should either reproduce the ownership-invalid current shape or prove the active checkout has already repaired it.

Do not run yet:
- `cargo test -p phalcom-semantic --test semantic` — deferred until C10 because C0 changes no semantics;
- workspace tests — Final Gate.

Escalate immediately if:
- generic getter tests fail on a checkout claiming to be at/after the investigated baseline;
- `merge_constructor_generic_signatures` no longer exists because another implementation already replaced it;
- metadata export no longer calls `GenericSignature::validate_publishable`;
- `TypeParameterOwner` semantics materially changed.

Checkpoint completion:
- [ ] repository state recorded;
- [ ] getter parser/semantic baseline passes or drift incident is documented;
- [ ] generic application baseline passes;
- [ ] constructor publication defect is reproduced or proven already repaired;
- [ ] implementation state updated;
- [ ] no active incident remains.

Suggested commit grouping:
- No production commit required if C0 adds only a failing/characterization test. If a RED regression is intentionally committed, use one test-only commit and clearly mark it as the C1 target.

---

## Task 1 — Record active checkout and bounded drift

Purpose:
Establish the exact repository state the implementing agent will modify.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:
- implementation state file only.

Inspect before editing:
- `phalcom-semantic/src/checker/declaration_signature.rs::merge_constructor_generic_signatures`;
- `phalcom-semantic/src/types/parameter.rs::GenericSignature::validate_publishable`;
- `phalcom-semantic/src/checker/declaration_signature.rs::resolve_callable_local_generics`.

Do not inspect unless evidence forces expansion:
- VM/runtime;
- parser internals unrelated to getters;
- LSP.

Dependencies:
- none.

Source of truth:
- local git checkout state.

Implementation boundary:

Changes:
- create/update implementation state with branch, HEAD, dirty paths relevant to this program;
- compare active HEAD against `e17f2733...` if it differs.

Must not:
- reset or discard user changes;
- modify production code in this task.

Edit operations:
1. RUN `git status --short`.
2. RUN `git branch --show-current`.
3. RUN `git rev-parse HEAD`.
4. If HEAD differs, RUN a bounded `git log --oneline e17f2733..HEAD -- <primary paths>` or equivalent.
5. Record only materially relevant drift.

Testing classification:
- No standalone behavioral test. Validated by C0.

Checkpoint state update:
Record repository revision, dirty relevant paths, and any drift affecting the plan.

---

## Task 2 — Reconfirm current getter/generic behavior

Purpose:
Prevent stale `docs/work/deferred/generic-on-getter.md` assumptions from entering the implementation.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned files and symbols:
- no production edits expected;
- `phalcom-semantic/tests/semantic/capabilities/getters.rs` — baseline authority;
- `phalcom-ast` getter parser tests — syntax authority.

Inspect before editing:
- `GetterDef` fields;
- parser generic getter path;
- `resolve_callable_local_generics` getter arm.

Source of truth:
- current AST/parser/semantic code and focused tests.

Implementation boundary:
- documentation classification only unless local drift has regressed the feature.

Must not:
- add a second getter-generic implementation;
- restore the stale prohibition.

Testing classification:
- Focused baseline evidence at C0.

---

## Task 3 — Characterize constructor publication ownership

Purpose:
Create the smallest executable proof that a constructor's declaration-owned and callable-owned generics must not be represented as one mixed-owner published `GenericSignature`.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file only if active code already diverged

Owned files and symbols:
- `phalcom-semantic/src/checker/declaration_signature.rs::merge_constructor_generic_signatures`;
- `phalcom-semantic/src/types/parameter.rs::GenericSignature::validate_publishable`;
- `phalcom-semantic/src/metadata/export.rs::export_generic_signature`;
- `phalcom-semantic/tests/semantic/integration/metadata.rs`.

Inspect before editing:
- how `CallableSemanticSignature.generics` is exported;
- existing metadata generic-signature tests;
- fixture helpers for locating constructor signatures.

Source of truth:
- actual `TypeParameterData.owner` for each `TypeParameterId`.

Implementation boundary:

Changes:
- add a regression using a generic declaration plus constructor-local generic binder;
- assert declaration binder remains `Declaration(owner)` and constructor binder remains `Callable(constructor)`;
- assert durable export cannot rely on a false re-ownering.

Must not:
- change validation in C0;
- patch production behavior yet.

Code instructions:

STRUCTURAL:
```rust
class Box<T> {
  @constructor
  new<U>(_ value: T, _ metadata: U) {}
}
```

Retrieve the canonical constructor callable signature and inspect its generic/publication representation. The regression should be written against the desired C1 contract if the test harness supports a RED test; otherwise add a lower-level unit test that demonstrates the current mixed-owner publication failure without broadening the public API.

Testing classification:
- Focused regression required because this is the architectural defect C1 will repair.

---

# Checkpoint C1 — Canonical Multi-Domain Generic Application

Tasks:
- Task 4 — Remove mixed-owner constructor signature publication.
- Task 5 — Introduce/reuse an application-level generic-domain composition product.
- Task 6 — Route constructor inference through declaration + callable domains.
- Task 7 — Publish separate solved substitutions and saturated receiver application.
- Task 8 — Close metadata/diagnostic/caller migration.

Why this is a checkpoint:

Tasks 4–8 are meaningful only together. Removing the merge without feeding owner generics into inference breaks constructor inference; adding a new application product without migrating publication leaves duplicate authorities. C1 is complete only when generic construction still infers exactly as before while canonical signature ownership becomes valid and durable.

Entry conditions:
- C0 COMPLETE;
- current generic getter/generic application baselines are green;
- constructor publication defect is characterized.

Working set:

Primary:
- `phalcom-semantic/src/checker/declaration_signature.rs`;
- `phalcom-semantic/src/checker/call.rs`;
- `phalcom-semantic/src/checker/inference.rs`;
- `phalcom-semantic/src/checker/context.rs` if domain access belongs there;
- `phalcom-semantic/src/signature.rs` / canonical callable signature product;
- `phalcom-semantic/src/metadata/export.rs`;
- relevant generic constructor tests.

Secondary — inspect only if evidence requires it:
- `phalcom-semantic/src/dispatch.rs`;
- `phalcom-semantic/src/checker/expression.rs`;
- `phalcom-semantic/src/types/substitution.rs`;
- explanation/trace products.

Out of scope for this checkpoint:
- setter/index syntax;
- variants;
- class-side generic storage semantics beyond retaining receiver application already required for constructors;
- rigid variables.

Semantic contract established by this checkpoint:
- canonical constructor `GenericSignature` contains only constructor-local binders;
- declaration generics are obtained from the declaration signature separately;
- one inference session can solve both domains;
- solved substitutions are distinguishable by owner;
- result and receiver are canonical proper applied types;
- metadata publication succeeds without relaxing owner validation.

Semantic risks:
- losing declaration-owned constructor inference;
- duplicate inference variables for the same declaration parameter;
- prematurely committing receiver inference before expected-result evidence;
- accidental callable identity specialization;
- diagnostics losing owner information;
- metadata retaining obsolete mixed-owner signature.

Hostile cases:
- generic class + generic constructor solves both `T` and `U` differently;
- explicit `Box<Int>` plus contradictory `String` value rejects rather than respecializing;
- declaration `T` solved but constructor `U` underconstrained reports `U`, not generic raw receiver failure;
- metadata validates the constructor local signature;
- generic getter/method behavior remains unchanged.

Required evidence:
1. focused generic-constructor regression module — proves declaration and callable substitutions specialize simultaneously;
2. metadata regression — proves canonical constructor signature exports successfully and owner validation remains enabled;
3. `cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_application -- --nocapture` — proves shared application solver behavior remains sound;
4. `cargo test -p phalcom-semantic --test semantic semantic::foundations::receiver_specialization -- --nocapture` — proves declaration/callable receiver composition remains correct;
5. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::getters -- --nocapture` — proves C1 did not regress an existing callable-local zero-arg surface;
6. negative search: `rg 'merge_constructor_generic_signatures' phalcom-semantic` must return zero production occurrences after migration.

Do not run yet:
- all ADT tests — C5/C7;
- full semantic package — C10;
- workspace — Final Gate.

Escalate immediately if:
- the solver API fundamentally assumes one `GenericSignature` owner and cannot accept a composed parameter set without changing `InferenceSession` ownership semantics;
- a durable consumer expects `CallableSemanticSignature.generics` to include declaration parameters;
- removing the merge breaks native declarations in a way that indicates native metadata treats constructor declaration parameters differently.

Checkpoint completion:
- [ ] all tasks implemented;
- [ ] declaration/callable domains remain separate;
- [ ] constructor inference still works;
- [ ] metadata owner validation passes;
- [ ] hostile cases pass;
- [ ] obsolete merge removed;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `refactor(semantic): separate constructor generic application domains`
- `fix(semantic): preserve constructor receiver and callable substitutions`
- `test(semantic): certify constructor generic ownership and publication`

---

## Task 4 — Remove mixed-owner canonical constructor signature

Purpose:
Restore `GenericSignature`'s homogeneous-owner invariant without losing constructor inference.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/declaration_signature.rs::semantic_signature_for_syntax` or current signature builder;
- `merge_constructor_generic_signatures`;
- `CallableSemanticSignature.generics`.

Inspect before editing:
- all production reads of `signature.generics` for constructors;
- `CheckingContext::declaration_generic_signature`;
- metadata export path.

Do not inspect unless evidence forces expansion:
- GADT code;
- parser;
- VM.

Dependencies:
- C0 constructor ownership regression.

Source of truth:
- `TypeParameterOwner` of each canonical `TypeParameterId`.

Implementation boundary:

Changes:
- canonical constructor callable signature keeps only callable-local generic binders;
- declaration generic signature remains owned/published by the declaration;
- remove constructor merge from signature formation.

Must not:
- weaken `GenericSignature::validate_publishable`;
- clone declaration binders with callable ownership;
- create new type parameters solely for constructor application.

Current implementation:
`declaration_signature.rs` calls `merge_constructor_generic_signatures(owner_generics, callable_generics, callable.clone())`, concatenates parameters/constraints, and rewrites the resulting signature owner to `TypeParameterOwner::Callable(callable)`.

Target implementation:
`CallableSemanticSignature.generics` contains only constructor-local binders. Declaration generics are composed later at application time.

Edit operations:
1. OPEN `phalcom-semantic/src/checker/declaration_signature.rs`.
2. FIND the post-match constructor block beginning with the comment describing instantiation of declaration owner + constructor-local binders.
3. REMOVE the assignment that merges declaration generics into `generics`.
4. REMOVE `merge_constructor_generic_signatures` once all callers are migrated.
5. SEARCH `rg 'merge_constructor_generic_signatures|\.generics' phalcom-semantic/src` and inspect constructor-sensitive consumers.
6. CLEAN imports/comments that describe the merged signature as canonical.

Code instructions:

EXACT semantic requirement:
```text
Do not alter `resolve_callable_local_generics`.
Do not alter constructor-local TypeParameterOwner creation.
After this task, the canonical callable signature's `generics` field must represent callable-local binders only.
```

Testing classification:
- No standalone behavioral test. C1 proves this after application migration.

Optional compile checkpoint:
```bash
cargo check -p phalcom-semantic
```
Reason: removal may expose callers that incorrectly depended on merged signature shape.

Checkpoint state update:
Record that `CallableSemanticSignature.generics` is now callable-local-only.

---

## Task 5 — Introduce application-level generic domain composition

Purpose:
Represent multiple generic ownership domains without corrupting declaration products.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/call.rs` — application authority;
- `phalcom-semantic/src/checker/inference.rs` — solver input;
- optionally a small new internal type in the checker/application layer.

Inspect before editing:
- function that converts `GenericSignature` parameters into inference variables;
- fixed receiver substitution path;
- expected-result constraint path;
- constructor-specific application branch.

Source of truth:
- canonical declaration `GenericSignature` + canonical callable `GenericSignature`.

Implementation boundary:

Changes:
- add an application-only product capable of enumerating/fixing variables from multiple owner domains;
- keep canonical signatures unchanged;
- feed one solver session.

Must not:
- expose this application product as a replacement declaration signature;
- persist it in metadata as a generic signature;
- create duplicated inference variables for one canonical parameter.

Code instructions:

STRUCTURAL:
```rust
struct GenericApplicationDomains<'a> {
    declaration: Option<&'a GenericSignature>,
    callable: Option<&'a GenericSignature>,
}
```

or a repository-native equivalent.

Required operations:

```text
iterate canonical parameters across domains
lookup domain by TypeParameterId/owner
build fixed substitutions from receiver application
allocate flexible variables for unsolved parameters
reconcile constraints from both domains
return owner-preserving solved substitutions
```

Use existing substitution/interner/map types. Do not add a global registry.

Testing classification:
- No standalone test; C1 integrated constructor cases prove the abstraction.

---

## Task 6 — Route generic constructors through composed domains

Purpose:
Recover existing generic receiver inference after Task 4 while keeping owner separation.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- generic callable application function(s) in `checker/call.rs`;
- receiver specialization helper(s);
- `CheckingContext::declaration_generic_signature`.

Inspect before editing:
- how explicit applied receiver arguments become fixed generics;
- how bare generic class-object constructor calls currently create owner inference variables;
- how expected result reaches generic application.

Source of truth:
- receiver's declaration application plus canonical signatures.

Changes:
- when target is constructor-like, supply declaration generic domain in addition to callable-local domain;
- map explicit receiver arguments to fixed declaration parameters;
- allocate unsolved declaration parameters for raw generic receiver inference;
- include both domains' constraints;
- preserve one reconciliation pipeline.

Must not:
- create constructor-only inference algorithm;
- commit declaration substitution before processing expected result unless existing solver requires/finalizes it safely.

Testing classification:
- C1 hostile constructor tests.

---

## Task 7 — Publish owner-separated solutions and saturated receiver

Purpose:
Make the solver result semantically useful to diagnostics, lowering, and later applied class-side storage.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- call application outcome / `TypedExpression`-adjacent invocation facts;
- existing explanation/trace products if they already publish substitutions;
- receiver specialization product.

Inspect before editing:
- current call outcome struct;
- expression publication fields;
- lowering's selected target consumption.

Source of truth:
- solved canonical `TypeParameterId -> TypeId` mapping plus canonical receiver type.

Changes:
- ensure call result can distinguish declaration-owned and callable-owned solutions;
- retain canonical saturated receiver application for constructor calls;
- avoid changing `CallableId`.

Must not:
- introduce runtime monomorphization IDs;
- store raw inference variable IDs.

Code instructions:

STRUCTURAL:
```rust
struct InvocationSpecialization {
    target: InvocationTargetId,
    receiver_application: Option<TypeId>,
    declaration_substitution: TypeSubstitution,
    callable_substitution: TypeSubstitution,
}
```

This exact struct need not land in C1 if an existing application/trace product can represent the same information. Reuse existing publication infrastructure whenever possible. C3 will make receiver retention mandatory for all applied class-side sends.

Testing classification:
- focused C1 constructor result/trace assertions.

---

## Task 8 — Metadata, diagnostics, and caller closure

Purpose:
Remove remaining dependencies on the mixed-owner signature and ensure failures name the correct ownership domain.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- metadata exporter generic signature path;
- generic inference diagnostics;
- call explanation/publication.

Inspect before editing:
- `MetadataExporter::export_generic_signature`;
- generic underconstrained diagnostic construction;
- tests that inspect callable signature owner.

Source of truth:
- canonical declaration/callable signatures, never combined fake owner.

Changes:
- export callable-local constructor signature normally;
- ensure declaration signature remains separately exported through declaration metadata;
- ensure underconstrained constructor local reports callable binder;
- remove stale comments referring to one merged constructor generic signature.

Must not:
- special-case metadata validation for constructors.

Testing classification:
- C1 metadata and diagnostic evidence.


# Checkpoint C2 — Generic Setter and Index Member Surfaces

Tasks:
- Task 9 — Extend setter AST and parser with generic binders/`where`.
- Task 10 — Extend index member AST and parser with generic binders/`where`.
- Task 11 — Publish setter-local generic signatures.
- Task 12 — Publish index-getter-local generic signatures.
- Task 13 — Publish index-setter-local generic signatures and RHS inference.
- Task 14 — Preserve selector/callable identity and source-index consumers.
- Task 15 — Fingerprint and test accessor/index generic contracts.

Why this is a checkpoint:

Setter and index generic syntax is low-level plumbing until semantic signature formation and call application are integrated. Testing after every AST edit would prove almost nothing. C2 becomes meaningful when each surface is a canonical callable with `TypeParameterOwner::Callable`, stable selector identity, normal constraints, and the shared application solver from C1.

Entry conditions:
- C1 COMPLETE;
- `CallableSemanticSignature.generics` is callable-local-only;
- multi-domain generic application is available for receiver + callable cases.

Working set:

Primary:
- `phalcom-ast/src/ast.rs` — `SetterDef`, `IndexMethodDef`;
- `phalcom-ast/src/parser.rs` — member header parsing;
- `phalcom-ast/src/selector.rs` — identity confirmation only;
- `phalcom-semantic/src/checker/declaration_signature.rs` — setter/index signature arms;
- `phalcom-semantic/src/source_index/builder.rs` — synthesized source callable/binding indexing;
- `phalcom-semantic/src/db/fingerprint.rs` — semantic source fingerprints;
- `phalcom-core/src/compiler/attributes.rs` — generated `@set` accessor construction;
- `phalcom-semantic/tests/semantic/capabilities/`.

Secondary — inspect only if evidence requires it:
- enum behavior member parsing, because setters/indexers also appear in `EnumBehaviorMember`;
- LSP selector wrappers only if AST construction changes break compilation.

Out of scope for this checkpoint:
- variant-local generics;
- rigid variables;
- class-side generic storage semantics beyond normal receiver specialization;
- native metadata generics.

Semantic contract established by this checkpoint:
- setters, index getters, and index setters can declare callable-local generic binders and `where` constraints;
- binder ownership is canonical `CallableId` ownership;
- assigned value participates in setter/index-setter inference as a normal argument;
- generic type arguments do not alter selectors;
- generated non-generic setters remain valid with empty generic metadata.

Semantic risks:
- parser ambiguity around `<...>` placement;
- accidentally including put-value shape in selector identity;
- resolving parameter annotations under declaration scope but not local generic scope;
- setter `where` constraints omitted from fingerprints;
- generated accessor AST constructors missing new fields;
- enum behavior setter/index paths diverging from class member paths.

Hostile cases:
- setter RHS determines `T` correctly for both `Int` and `String` calls using the same `CallableId`;
- bound failure rejects rather than recovering to `Dynamic`;
- index key and assigned value provide conflicting evidence and fail;
- `Store<Int>` receiver plus index-local `U` solves both domains simultaneously;
- two instantiations keep identical selectors/`CallableId`s;
- generated `@set` accessor remains non-generic and compilable.

Required evidence:
1. parser-focused tests for generic setter/index getter/index setter headers and `where` clauses;
2. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::setters -- --nocapture` after adding `setters.rs` — proves RHS-driven local inference and constraints;
3. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::index_generics -- --nocapture` — proves key/receiver/RHS/expected-result interactions and selector stability;
4. `cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application -- --nocapture` — proves assignment/index call application still follows canonical expression semantics;
5. focused compiler attribute test or `cargo test -p phalcom-core --test core` filtered to generated accessor tests if existing tests exist — proves new AST fields do not break `@set` synthesis.

Do not run yet:
- full `phalcom-semantic` suite — C10;
- ADT suite — C5/C7;
- workspace — Final Gate.

Escalate immediately if:
- generic binder grammar cannot be added without ambiguity with existing setter/index syntax;
- selector formation currently depends on parameter types rather than syntax slots/labels;
- index setter return semantics are relied on as non-`Unit` by production consumers. In that case classify as a semantic design incident before changing return contracts.

Checkpoint completion:
- [ ] AST/parser complete;
- [ ] setter/index semantic signatures publish callable-local generics;
- [ ] shared solver handles RHS/key/expected result;
- [ ] selector identity hostile cases pass;
- [ ] fingerprints include new contracts;
- [ ] generated accessors compile;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(ast): add generic binders to setters and index members`
- `feat(semantic): apply canonical generic inference to setters and indexers`
- `test(semantic): certify generic accessor and index laws`

---

## Task 9 — Extend `SetterDef` and setter parser

Purpose:
Make generic setters a first-class syntax surface using the existing generic binder grammar.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/ast.rs::SetterDef`;
- `phalcom-ast/src/parser.rs` setter member-header branch;
- AST parser tests.

Inspect before editing:
- current `GetterDef` fields and parser path — use as the closest working model;
- current setter parser path that recognizes trailing `=`;
- generic binder placement for methods/getters.

Do not inspect unless evidence forces expansion:
- semantic call solver;
- VM.

Dependencies:
- C1 generic application remains canonical.

Source of truth:
- AST's `GenericParameterSyntax` and `WhereClauseSyntax`.

Implementation boundary:

Changes:
- add `generic_parameters: Vec<GenericParameterSyntax>`;
- add `where_clause: Option<WhereClauseSyntax>`;
- parse local generic binders at the least disruptive position consistent with the getter/method grammar;
- parse optional `where` after the setter header according to existing grammar conventions.

Must not:
- create setter-specific generic syntax nodes;
- alter setter selector encoding.

Current implementation:
`SetterDef` has setter name, one parameter, return/body/attributes/range data but no local generic binder or `where` surface.

Target implementation:
Setter header carries the same reusable generic syntax nodes as methods/getters.

Edit operations:
1. OPEN `phalcom-ast/src/ast.rs`.
2. FIND `pub struct SetterDef`.
3. ADD the two generic fields adjacent to declaration/header metadata, following `GetterDef` conventions.
4. OPEN `phalcom-ast/src/parser.rs`.
5. FIND the branch constructing `ClassMember::Setter(SetterDef { ... })`.
6. REUSE `parse_generic_parameters(...)` and `parse_where_clause()` rather than duplicating grammar.
7. UPDATE every `SetterDef {` constructor using `rg 'SetterDef \{'` across the repository.
8. For generated non-generic setters, initialize `generic_parameters: Vec::new()` and `where_clause: None`.
9. ADD parser tests for binder + `where`, no-`where`, and invalid variance if callable binder variance is prohibited by existing binder rules.

Code instructions:

STRUCTURAL:
```rust
pub struct SetterDef {
    // existing fields...
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    // existing fields...
}
```

Use the same callable binder context currently used by methods/getters. Do not invent `GenericBinderContext::Setter` unless parser validation actually differs; prefer existing callable context.

Testing classification:
- Parser tests are required but run at C2 boundary.

Optional compile checkpoint:
```bash
cargo check -p phalcom-ast -p phalcom-core
```
Reason: finds all struct-literal fanout after adding fields, including attribute-generated accessors.

Checkpoint state update:
Record final generic setter syntax and the binder context reused.

---

## Task 10 — Extend `IndexMethodDef` and index parser

Purpose:
Make index getter/setter local generic binders first-class without altering index selector identity.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/ast.rs::IndexMethodDef`;
- `phalcom-ast/src/parser.rs` index member parsing;
- `phalcom-ast/src/selector.rs` inspect-only identity authority.

Inspect before editing:
- exact index getter/setter grammar;
- `IndexAccessor::{Get, Set}`;
- how labels/arity become `Selector::subscript_get` / `Selector::subscript_set`.

Source of truth:
- existing index selector slots/labels, not generic arguments.

Changes:
- add `generic_parameters` and `where_clause` to `IndexMethodDef`;
- parse them at a consistent callable-header location;
- preserve `IndexAccessor` and selector formation unchanged.

Must not:
- include put value or generic types in selector slots;
- create generic-specific index selector kinds.

Edit operations:
1. OPEN `phalcom-ast/src/ast.rs` and FIND `IndexMethodDef`.
2. ADD generic binder and `where` fields.
3. OPEN `phalcom-ast/src/parser.rs` and FIND index member parser/construction.
4. REUSE callable generic binder parser.
5. UPDATE every `IndexMethodDef {` struct literal.
6. ASSERT via parser/selector test that `[]`/`[]=` selector identity is unchanged between generic/non-generic declarations with identical labels.

Code instructions:

STRUCTURAL:
```rust
pub struct IndexMethodDef {
    // existing index params/accessor fields
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    // existing return/body/attributes/range
}
```

Testing classification:
- Parser + selector assertions at C2.

---

## Task 11 — Publish generic setter semantic signatures

Purpose:
Use the canonical callable-local generic machinery for setters.

Risk:
- Semantic: HIGH
- Implementation fanout: local/multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/declaration_signature.rs` setter arm;
- `resolve_callable_local_generics`;
- setter parameter `parameter_fact` path.

Inspect before editing:
- method/getter generic resolver construction;
- current setter arm, which currently returns `generics: None`;
- setter body binding/source-index handling.

Dependencies:
- Task 9 AST fields;
- C1 callable-local signature invariant.

Source of truth:
- setter `CallableId` and `TypeParameterOwner::Callable(setter_id)`.

Changes:
- call `resolve_callable_local_generics` using setter generic fields;
- construct a setter-local `ScopedTypeResolver` layered over declaration receiver scope;
- resolve setter parameter annotation and local constraints in that scope;
- return `Unit` through existing setter semantics.

Must not:
- infer generic parameters at declaration construction time;
- generate a distinct selector per instantiation.

Current implementation:
Setter parameter is resolved directly with `declaration_resolver`; semantic signature returns `(None, [parameter], Unit)`.

Target implementation:
Setter mirrors getter/method local binder publication and keeps one value parameter.

Edit operations:
1. OPEN `checker/declaration_signature.rs`.
2. FIND `CallableSyntaxRef::Setter(setter)`.
3. ADD `resolve_callable_local_generics(...)` call analogous to getter.
4. BUILD a local type-parameter binding map from that signature.
5. RESOLVE `setter.param` under local resolver.
6. RETURN local `generics` rather than `None`.
7. LEAVE selector generation unchanged.

Code instructions:

STRUCTURAL:
Follow the getter arm's local generic resolver construction, then retain the setter-specific `Unit` result.

Testing classification:
- C2 setter semantic tests.

---

## Task 12 — Publish generic index getter semantic signatures

Purpose:
Resolve index parameter/result annotations under index-local callable binders.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `checker/declaration_signature.rs` index arm.

Inspect before editing:
- `CallableSyntaxRef::Index` current branch;
- index selector construction in `callable_id_for_syntax`;
- expected-result inference path for zero/nonzero argument generic calls.

Source of truth:
- canonical index `CallableId`.

Changes:
- resolve local generic signature;
- layer local resolver over declaration resolver;
- resolve index params and getter return annotation through local resolver;
- return local signature in `CallableSemanticSignature.generics`.

Must not:
- modify selector construction.

Testing classification:
- C2 index getter tests.

---

## Task 13 — Publish generic index setter and RHS constraints

Purpose:
Make the put value an ordinary constraint-bearing parameter of a generic index setter.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `checker/declaration_signature.rs` index setter branch;
- call application path handling assignment/index set arguments.

Inspect before editing:
- current code appending `put_semantic` after index parameters;
- assignment expression construction of actual arguments;
- canonical result handling.

Source of truth:
- setter callable parameter list: index params followed by put parameter.

Changes:
- resolve both index params and `put` under same local generic resolver;
- make all actual arguments reach ordinary generic inference in order;
- preserve `Unit` assignment expression behavior.

Must not:
- model put value as post-call validation outside inference;
- silently widen key/value conflicts to `Dynamic`.

Testing classification:
- C2 conflicting evidence hostile case required.

---

## Task 14 — Preserve selector identity and source-index consumers

Purpose:
Prevent generic syntax from leaking into identity or tooling.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/selector.rs`;
- `phalcom-semantic/src/checker/declaration_signature.rs::callable_id_for_syntax`;
- `phalcom-semantic/src/source_index/builder.rs::visit_member` and index branch;
- LSP selector wrappers only if compiler errors require updates.

Source of truth:
- canonical selector subsystem.

Changes:
- generally no selector code change should be necessary;
- update source-index AST field destructuring if compilation requires it;
- add identity assertions to tests.

Must not:
- append generic arity/type names to selector base or slots.

Testing classification:
- no separate test; C2 identity assertions.

---

## Task 15 — Fingerprint and regression closure for accessors/indexers

Purpose:
Ensure generic contract edits invalidate semantic consumers.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `phalcom-semantic/src/db/fingerprint.rs` member hashing;
- `capabilities/mod.rs` new test modules.

Inspect before editing:
- getter's existing `hash_generic_contract_source` use;
- setter/index member hash branches.

Source of truth:
- source generic binder/constraint text normalized through existing fingerprint helper.

Changes:
- add setter generic contract hashing;
- add index generic contract hashing;
- include `where` clauses;
- add focused semantic tests.

Must not:
- hash inferred substitutions or runtime values.

Testing classification:
- C2 focused test modules.

---

# Checkpoint C3 — Applied Generic Class-Side Templates and Durable Receiver Specialization

Tasks:
- Task 16 — Amend class-side declaration generic scope for template formation.
- Task 17 — Specialize class-side fields/members under proper applied receivers.
- Task 18 — Define/extend durable invocation specialization with `receiver_application`.
- Task 19 — Route inferred generic construction to the same applied receiver endpoint.
- Task 20 — Preserve bound Family receiver application.
- Task 21 — Add class-side applied semantics tests without implementing runtime storage.

Why this is a checkpoint:

The new per-application class-storage law changes a previously ratified semantic boundary. It is not enough for `Box<Int>` to exist as a result type; class-side templates must be able to mention `T`, member lookup must specialize them under an applied receiver, and invocation publication must retain the applied owner for future lowering/runtime state. These tasks establish that static contract as one coherent boundary.

Entry conditions:
- C1 COMPLETE;
- C2 may be complete or independent, but C3 must not begin on an unresolved C1 incident;
- canonical applied type application and receiver specialization remain live.

Working set:

Primary:
- `phalcom-semantic/src/checker/declaration_signature.rs::declaration_type_level_bindings_for_side`;
- `phalcom-semantic/src/checker/declaration.rs` member/field signature formation;
- receiver/member specialization code in `checker/call.rs`, `dispatch.rs`, `associated.rs` as confirmed by drift check;
- semantic expression/application publication product;
- Family bound target representation;
- semantic tests for receiver specialization/fields/Families.

Secondary — inspect only if evidence requires it:
- `phalcom-core/src/modules/semantic_lowering.rs` to ensure the applied receiver can be carried forward without implementing runtime storage;
- metadata presentation if class-side specialized signatures are published there.

Out of scope for this checkpoint:
- actual runtime per-application storage table;
- new applied metaclass object allocation;
- `Type.currentApplication` public API;
- variant generics.

Semantic contract established by this checkpoint:
- generic class-side declarations may refer to declaration parameters as templates;
- use-site access requires a proper applied receiver or enough inference to produce one;
- `Box<Int>` specializes class-side signatures/fields under `T := Int`;
- `Box<String>` specializes independently;
- raw `Box.member` remains underconstrained when the member depends on `T` and context cannot solve it;
- invocation semantics retain the applied receiver as a durable fact;
- inferred constructor call and explicit `Box<Int>.new(...)` converge on the same receiver application.

Semantic risks:
- accidentally treating bare `Box` as implicit `Box<T>` at value-use sites;
- exposing declaration generics on class side without saturation checks;
- erasing receiver application after type checking;
- tying type arguments to selector identity;
- changing runtime dispatch prematurely;
- breaking non-generic class-side members that do not depend on `T`.

Hostile cases:
- `Box<Int>.instances : List<Box<Int>>` and `Box<String>.instances : List<Box<String>>`;
- those accesses use the same `CallableId`;
- raw `Box.instances` is underconstrained rather than erased/globalized;
- a class-side member independent of `T` remains callable on raw `Box` if existing semantics allow it;
- `Box.new(10)` publishes receiver `Box<Int>`;
- `Box.new("x")` publishes receiver `Box<String>`;
- bound Family captured from `Box<Int>` does not later act like raw `Box`.

Required evidence:
1. focused new `semantic::integration::applied_class_side` tests — proves specialization/underconstraint/identity;
2. `cargo test -p phalcom-semantic --test semantic semantic::foundations::receiver_specialization -- --nocapture` — preserves existing receiver rules;
3. generic constructor tests from C1 — prove inferred and explicit receiver convergence;
4. focused Family invocation tests if bound applied class-side capture exists/gets added;
5. a semantic publication assertion that the resolved invocation product exposes canonical `Box<Int>` rather than only bare `Box`.

Do not run yet:
- runtime storage tests — explicitly out of scope;
- full core runtime suite — C10/Final Gate;
- workspace — Final Gate.

Escalate immediately if:
- current class-side member lookup is structurally incapable of representing an applied receiver without changing runtime class-object identity;
- the only available application publication product is runtime-specific and would force premature VM changes;
- raw generic class-side access semantics conflict with an independently ratified current language spec that postdates the supplied per-application storage decision.

Checkpoint completion:
- [ ] class-side template formation amended;
- [ ] applied class-side specialization works;
- [ ] raw unsaturated hostile case behaves correctly;
- [ ] invocation product retains applied receiver;
- [ ] constructor convergence proven;
- [ ] Family receiver retention handled where applicable;
- [ ] runtime storage remains untouched;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(semantic): parameterize generic class-side declaration templates`
- `feat(semantic): retain canonical applied receiver specialization`
- `test(semantic): certify applied class-side static semantics`

---

## Task 16 — Amend class-side declaration generic scope for template formation

Purpose:
Supersede the current rule that class-side declaration formation has no declaration type-parameter bindings.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/declaration_signature.rs::declaration_type_level_bindings_for_side`;
- `phalcom-semantic/src/checker/declaration.rs` field/member formation callers.

Inspect before editing:
- all callers of `declaration_type_level_bindings_for_side`;
- distinction between declaration template formation and use-site member access;
- tests relying on class-side generic parameter rejection.

Source of truth:
- generic declaration `GenericSignature`.

Implementation boundary:

Changes:
- declaration template formation for class-side members/fields must be able to resolve `T`;
- do not globally redefine bare class-object use as an applied type.

Must not:
- simply delete the class-side guard without checking all callers' semantics;
- introduce an ambient runtime application context.

Current implementation:
```rust
if side == DispatchSide::Class {
    return HashMap::new();
}
declaration_type_level_bindings(ctx, owner)
```

Target implementation:
Template formation sees declaration bindings; use-site saturation/inference remains separate.

Edit operations:
1. OPEN `checker/declaration_signature.rs`.
2. FIND `declaration_type_level_bindings_for_side`.
3. SEARCH all callers with `rg 'declaration_type_level_bindings_for_side'`.
4. CLASSIFY each caller as declaration-template formation vs value/use-site resolution.
5. REFACTOR helper naming/API if necessary so template formation explicitly requests declaration binders on either side while use-site code cannot accidentally obtain ambient unresolved parameters.
6. UPDATE comments/spec references that encode the old rule.

Code instructions:

STRUCTURAL:
Prefer making the distinction explicit in the API rather than turning a side-aware helper into an unconditional map if some caller uses it as a use-site guard.

Testing classification:
- no standalone test; C3 applied class-side tests.

---

## Task 17 — Specialize class-side signatures and fields under applied receivers

Purpose:
Make `Box<Int>.class`-side semantic views substitute declaration parameters exactly like instance-side owner specialization.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- receiver/member specialization functions discovered in C3 drift check;
- field signature specialization path;
- class-side callable specialization.

Inspect before editing:
- existing `substitution_for_applied` usage;
- `receiver_specialization.rs` tests;
- field signature lookup/specialization code.

Source of truth:
- canonical applied receiver `TypeId` and declaration generic signature.

Changes:
- apply receiver substitution to class-side field/member templates;
- reuse existing substitution/materialization machinery;
- return canonical substituted types.

Must not:
- duplicate declarations per application;
- cache by display name/type text;
- alter selectors.

Testing classification:
- C3 focused applied class-side tests.

---

## Task 18 — Retain applied receiver in durable invocation specialization

Purpose:
Preserve the semantic fact future lowering/runtime class storage needs.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-module

Owned files and symbols:
- call application outcome/publication types;
- typed expression/invocation target products;
- snapshot tables if application facts are stored separately.

Inspect before editing:
- selected callable/target publication;
- explanation traces;
- source-index semantic target tables;
- lowering consumer of invocation target.

Source of truth:
- canonical proper applied receiver `TypeId` produced by semantic specialization.

Changes:
- add or reuse a field/product carrying `receiver_application`;
- ensure it survives snapshots at least through lowering;
- keep type substitutions owner-separated if available.

Must not:
- put receiver type arguments into `CallableId`;
- use raw source syntax as the durable receiver identity.

Code instructions:

STRUCTURAL:
```rust
InvocationSpecialization {
    target,
    receiver_application: Option<TypeId>,
    declaration_substitution,
    callable_substitution,
}
```

Use an existing product if one already carries equivalent information; do not create a second authority.

Testing classification:
- C3 publication assertion.

---

## Task 19 — Converge inferred construction on applied receiver endpoint

Purpose:
Ensure `Box.new(10)` and `Box<Int>.new(10)` reach the same semantic receiver application.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- C1 generic constructor application path;
- C3 invocation specialization publication.

Source of truth:
- solved declaration substitution canonicalized through `TypeStore::apply_type_form` or existing equivalent.

Changes:
- after constructor owner variables solve, construct canonical applied receiver once;
- publish it in invocation specialization;
- use the same member specialization endpoint as explicit applied receiver calls.

Must not:
- leave inferred constructor calls with a `None` receiver application merely because result is already `Box<Int>`.

Testing classification:
- C3 constructor convergence hostile case.

---

## Task 20 — Preserve applied receiver in bound Families

Purpose:
Prevent first-class Family capture from erasing declaration specialization needed when invoked later.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/types/family.rs`;
- `phalcom-semantic/src/checker/associated.rs`;
- Family invocation tests.

Inspect before editing:
- bound callable Family representation;
- how target + receiver are retained today;
- current rank-1 generic Family application.

Source of truth:
- canonical target plus receiver specialization.

Changes:
- reuse current bound target receiver representation if it already retains specialized type;
- otherwise extend it with canonical applied receiver;
- do not instantiate callable-local generics until invocation.

Must not:
- eagerly monomorphize a Family;
- replace receiver identity with result type guess.

Testing classification:
- one C3 Family integration test if current surface supports class-side Family capture.

---

## Task 21 — Add applied class-side semantic tests

Purpose:
Prove the new static contract without accidentally expanding into VM storage implementation.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only

Owned files and symbols:
- proposed `phalcom-semantic/tests/semantic/integration/applied_class_side.rs`;
- `integration/mod.rs`.

Source of truth:
- semantic snapshot callable/field/application facts.

Required cases:
```phalcom
class Box<T> {
  @class
  const _instances: List<Box<T>>

  @class
  instances -> List<Box<T>> { _instances }

  @constructor
  new(_ value: T) {}
}
```

Assert:
- `Box<Int>.instances : List<Box<Int>>`;
- `Box<String>.instances : List<Box<String>>`;
- callable identity equal between applications;
- receiver application differs;
- raw `Box.instances` underconstrained if no expected context solves `T`;
- inferred constructor receiver is `Box<Int>` / `Box<String>`.

Testing classification:
- checkpoint ownership-layer evidence.


# Checkpoint C4 — Variant-Local Generic Declaration Products

Tasks:
- Task 22 — Extend `VariantDecl` with local generic binders and `where`.
- Task 23 — Parse variant-local binders using existing callable generic grammar.
- Task 24 — Define canonical variant-constructor `CallableId` for binder ownership.
- Task 25 — Resolve variant-local generic signatures under enum declaration scope.
- Task 26 — Resolve payload/result templates under enum + variant local scope.
- Task 27 — Extend `VariantConstructorSignature`/`VariantInfo` publication.
- Task 28 — Fingerprint, metadata, and declaration tests.

Why this is a checkpoint:

Variant-local generics must become canonical declaration products before construction or matching can safely use them. C4 deliberately stops before invocation: it proves that a source declaration like `@variant Equal<U>(...)` yields stable, owner-correct semantic metadata and canonical payload/result templates.

Entry conditions:
- C1 COMPLETE;
- C2 COMPLETE if shared callable generic helpers were changed there;
- C3 may proceed independently, but no unresolved generic ownership incident may remain.

Working set:

Primary:
- `phalcom-ast/src/ast.rs::VariantDecl`;
- `phalcom-ast/src/parser.rs` variant declaration parser;
- `phalcom-ast/src/selector.rs` variant selector formation;
- `phalcom-semantic/src/identity.rs` — existing `CallableOwnerId::Variant`, optional helper only;
- `phalcom-semantic/src/enum_semantics.rs::VariantConstructorSignature`, `VariantInfo`;
- `phalcom-semantic/src/checker/enum_declaration.rs::build_enum_semantics` and local helpers;
- `phalcom-semantic/src/types/annotation.rs` generic signature resolver;
- `phalcom-semantic/src/db/fingerprint.rs` enum/variant contract hashing;
- ADT declaration/generic tests.

Secondary — inspect only if evidence requires it:
- `phalcom-semantic/src/checker/declaration_signature.rs::callable_id_for_syntax` for identity conventions;
- `phalcom-semantic/src/db/query.rs` variant-owned callable query lookup;
- metadata exporter if enum variant signatures are exported through another product.

Out of scope for this checkpoint:
- generic variant invocation;
- GADT skolems;
- existential escape;
- runtime construction changes.

Semantic contract established by this checkpoint:
- `VariantDecl` can introduce local generic binders and `where` constraints;
- each local binder is owned by a stable callable identity derived from the exact `VariantId`;
- payload/result annotations see enum declaration binders plus variant-local binders;
- `VariantConstructorSignature` publishes local generic signature without mutating enum declaration signature;
- local generic arguments never enter variant selector identity.

Semantic risks:
- using enum declaration ownership for local binders;
- creating a new `TypeParameterOwner::Variant` parallel authority;
- collision between constructor callable identity and variant case behavior callable identity;
- reparsing source annotations at invocation time rather than storing canonical templates;
- omitting local constraints from fingerprints;
- changing `VariantId` because generic syntax exists.

Hostile cases:
- enum `T` and variant `U` with same textual shape remain distinct parameter IDs/owners;
- a variant-local binder shadows/duplicates according to existing generic binder rules, not silently aliases enum `T`;
- two overloaded variants sharing a family base retain distinct exact `VariantId`s and distinct constructor callable owners;
- variant constructor callable identity is stable across cold analysis;
- adding `where U <: Bound` changes semantic fingerprint without changing selector identity.

Required evidence:
1. parser tests for `@variant V<T>(...)`, `where`, result annotations, and invalid binder forms;
2. `cargo test -p phalcom-semantic --test semantic semantic::adts::declarations -- --nocapture` — preserves existing enum declaration laws;
3. focused additions in `semantic::adts::generics` — prove variant-local owner identity, nested scope resolution, and constraints;
4. metadata/fingerprint test — prove local signature is durable and owner-valid;
5. identity assertion — exact `VariantId`/selector unchanged by generic binder syntax.

Do not run yet:
- constructor/matching ADT suites — C5/C7;
- full ADT package — after C7;
- full semantic package — C10.

Escalate immediately if:
- current `CallableId` cannot represent a collision-free variant-constructor binder owner without adding a new canonical identity helper;
- variant constructor metadata is serialized through a path that cannot refer to `GenericSignature` without a schema change broader than planned;
- parser grammar for `@variant Name<U>(...)` conflicts with associated-family syntax.

Checkpoint completion:
- [ ] AST/parser binder surface complete;
- [ ] constructor callable owner stable;
- [ ] local signature owner-valid;
- [ ] payload/result resolve under both scopes;
- [ ] fingerprints/metadata updated;
- [ ] hostile identity cases pass;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(ast): add variant-local generic binders`
- `feat(semantic): publish variant constructor generic signatures`
- `test(adts): certify variant-local generic declaration laws`

---

## Task 22 — Extend `VariantDecl`

Purpose:
Represent local generic binders/constraints directly in the AST rather than reconstructing them later.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/ast.rs::VariantDecl`;
- all `VariantDecl {` constructors.

Inspect before editing:
- `EnumDef.generic_parameters` / `where_clause`;
- `MethodDef`/`GetterDef` local generic fields;
- variant parser construction.

Source of truth:
- `GenericParameterSyntax` and `WhereClauseSyntax`.

Changes:
- add `generic_parameters: Vec<GenericParameterSyntax>`;
- add `where_clause: Option<WhereClauseSyntax>`.

Must not:
- put local binders into `EnumDef.generic_parameters`;
- encode them in payload `ParameterDef`s.

Edit operations:
1. OPEN `phalcom-ast/src/ast.rs`.
2. FIND `pub struct VariantDecl`.
3. ADD generic/where fields near name/header data before payload/result.
4. SEARCH `rg 'VariantDecl \{'` repository-wide.
5. UPDATE every construction site with empty fields where appropriate.
6. CLEAN historical comments claiming variant generics are unsupported if present.

Testing classification:
- parser evidence at C4.

Optional compile checkpoint:
```bash
cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core
```
Reason: AST struct-literal fanout can span compiler transformations/tests.

---

## Task 23 — Parse variant-local generic binders and `where`

Purpose:
Use the existing generic grammar for variants.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned files and symbols:
- `phalcom-ast/src/parser.rs` variant parser;
- parser tests.

Inspect before editing:
- function currently parsing variant name, optional payload, optional `->` result, optional body;
- `parse_generic_parameters` contexts;
- `parse_where_clause` newline handling.

Source of truth:
- callable generic binder grammar.

Changes:
- after variant name, parse `<...>` if present;
- parse `where` at the same semantic header boundary accepted by other declarations;
- ensure result/payload/body token boundaries remain unambiguous.

Must not:
- add new generic token syntax;
- allow declaration-site variance if current callable generic binder rules forbid it.

Code instructions:

STRUCTURAL:
```text
@variant Name<local binders>(payload...) -> Result
where ...
{ optional case body }
```

Use the repository's actual accepted ordering for `where` relative to return annotation/body; do not invent a second ordering if method/getter grammar already standardizes it.

Testing classification:
- C4 parser tests.

---

## Task 24 — Define canonical variant-constructor callable identity

Purpose:
Provide a stable `CallableId` solely for generic binder ownership/publication while retaining `VariantConstructorId` for execution.

Risk:
- Semantic: HIGH
- Implementation fanout: local/multi-file

Owned files and symbols:
- `phalcom-semantic/src/identity.rs::CallableId`, `CallableOwnerId`, `VariantId`;
- `enum_declaration.rs` constructor signature construction.

Inspect before editing:
- `CallableId::case_method`;
- all variant-owned callables in enum behavior;
- `db/query.rs` lookup for `CallableOwnerId::Variant`;
- stable callable metadata conversion.

Source of truth:
- exact `VariantId`.

Implementation boundary:

Recommended identity:
```text
owner    = CallableOwnerId::Variant(variant.clone())
selector = variant.selector.clone()
side     = DispatchSide::Class
```

This is disjoint from case behavior instance callables by side and derives entirely from exact variant identity.

Changes:
- add a helper such as `CallableId::variant_constructor_generic_owner(variant)` or more concise repository-native name if it improves consistency;
- use that callable only for generic signature/type-parameter ownership and metadata;
- do not replace `VariantConstructorId` in `InvocationTargetId`.

Must not:
- make constructor type arguments part of `VariantId`;
- add `TypeParameterOwner::Variant`;
- conflate this callable with a case behavior method.

Code instructions:

EXACT shape if no collision is found during inspect-before-edit:
```rust
pub fn variant_constructor(variant: VariantId) -> Self {
    Self {
        selector: variant.selector.clone(),
        owner: CallableOwnerId::Variant(variant),
        side: DispatchSide::Class,
    }
}
```

Name may be adapted to avoid collision with existing terminology. If existing stable metadata assumes variant-owned callable side is always instance, stop and classify PLAN DRIFT before committing this representation.

Testing classification:
- C4 identity test.

---

## Task 25 — Resolve variant-local `GenericSignature`

Purpose:
Create canonical callable-owned type parameters for the variant constructor.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/enum_declaration.rs`;
- `types::annotation::resolve_generic_signature`;
- `TypeFormationSite` construction for variants.

Inspect before editing:
- current enum declaration generic resolver;
- how variant payload annotations currently resolve enum `T`;
- visibility/source span publication.

Dependencies:
- Tasks 22–24.

Source of truth:
- canonical variant-constructor `CallableId`.

Changes:
- create parent resolver containing enum declaration parameters;
- call existing generic signature resolver with `TypeParameterOwner::Callable(variant_constructor_callable)`;
- use existing callable binder site/kind rules;
- publish diagnostics through normal type-formation outcomes.

Must not:
- synthesize parameters by manually interning names without `resolve_generic_signature`;
- merge enum declaration parameters into the local `GenericSignature`.

Testing classification:
- C4 semantic owner test.

---

## Task 26 — Resolve payload/result templates under nested scope

Purpose:
Allow both enum-owned and variant-local parameters in payload and GADT result templates.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `enum_declaration.rs` variant field/result resolution helpers.

Inspect before editing:
- current `ScopedTypeResolver` chain;
- exact result annotation handling and default enum result;
- case environment derivation.

Source of truth:
- enum declaration resolver parent + variant local resolver child.

Changes:
- build variant-local binding map from local `GenericSignature`;
- resolve every payload parameter and result annotation through that child resolver;
- preserve canonical `TypeId` templates containing the correct `TypeParameterId`s.

Must not:
- substitute local binders at declaration time;
- derive GADT result equations before canonical templates exist.

Testing classification:
- C4 generic variant declaration tests.

---

## Task 27 — Extend `VariantConstructorSignature` publication

Purpose:
Make constructor-local generic information available to construction, Families, and later existential elimination without re-resolving syntax.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/enum_semantics.rs::VariantConstructorSignature`;
- constructors in `enum_declaration.rs`;
- all readers of `VariantConstructorSignature`.

Inspect before editing:
- constructor call application;
- Family construction;
- GADT/pattern readers;
- clone/equality/fingerprint expectations.

Source of truth:
- canonical local `GenericSignature` plus canonical templates.

Changes:
- add `generic_signature: Option<GenericSignature>` or equivalent clearly named field;
- optionally add canonical binder-owner `CallableId` explicitly if otherwise recomputation is awkward;
- keep `VariantConstructorId`, parameters, result/exact-case templates unchanged.

Must not:
- mutate `EnumInfo.generic_signature`;
- store a combined enum + variant signature.

Code instructions:

STRUCTURAL:
```rust
pub struct VariantConstructorSignature {
    pub constructor: VariantConstructorId,
    pub generic_signature: Option<GenericSignature>,
    pub parameters: Box<[VariantConstructorParameter]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}
```

If stable metadata already has a different constructor-specific record, adapt mechanically while preserving ownership semantics.

Testing classification:
- C4 metadata/publication test.

---

## Task 28 — Fingerprint and metadata closure for generic variants

Purpose:
Make variant-local contract changes durable and incrementally visible.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/db/fingerprint.rs` enum/variant source hashing;
- `phalcom-semantic/src/metadata/export.rs` if variant constructor signature export exists;
- ADT generic tests.

Inspect before editing:
- current enum fingerprinting of generic declaration/result/payload syntax;
- metadata representation of enum/variant products.

Source of truth:
- canonical variant declaration product.

Changes:
- hash local generic binders, kinds, `where`, payload/result contract;
- export local generic signature through existing durable generic-signature table where applicable;
- add owner-validation regression.

Must not:
- hash inferred call-site substitutions;
- serialize source-only binder names as identity without stable callable owner/index.

Testing classification:
- C4 focused metadata/fingerprint evidence.

---

# Checkpoint C5 — Generic Variant Construction and Family Application

Tasks:
- Task 29 — Feed enum declaration + variant local domains into ordinary application inference.
- Task 30 — Solve payload argument constraints and canonical result specialization.
- Task 31 — Integrate expected-result constraints and conflict handling.
- Task 32 — Preserve stable variant executable identity.
- Task 33 — Integrate generic variant constructors with Families/associated lookup.
- Task 34 — Add construction/Family hostile tests.

Why this is a checkpoint:

C4 only proves declaration representation. C5 proves the central design claim: generic variant construction is ordinary generic callable application, not a new GADT constructor solver. It also ensures Families preserve the same generic target rather than adding a parallel specialization mechanism.

Entry conditions:
- C4 COMPLETE;
- C1 multi-domain application is stable;
- existing non-generic ADT constructor tests are green at baseline.

Working set:

Primary:
- `phalcom-semantic/src/checker/associated.rs` — variant family resolution/application;
- variant constructor invocation path in `checker/expression.rs` / `checker/call.rs` as discovered by drift check;
- `phalcom-semantic/src/enum_semantics.rs::VariantConstructorSignature`;
- `phalcom-semantic/src/types/family.rs`;
- ADT constructor/generic/associated/Family tests.

Secondary — inspect only if evidence requires it:
- `checker/inference.rs` for exact-case inference terms;
- semantic lowering target publication;
- exhaustiveness is not needed yet.

Out of scope for this checkpoint:
- fresh existential rigids;
- pattern payload opening;
- escape checking.

Semantic contract established by this checkpoint:
- variant-local parameters are universally instantiated at construction;
- enum declaration and variant local domains can solve together;
- result/exact-case types are canonicalized;
- variant construction keeps one `VariantConstructorId` regardless of instantiation;
- Family capture preserves the generic constructor target and instantiates at invocation.

Semantic risks:
- adding a variant-specific inference engine;
- solving enum declaration parameters by mutating declaration state;
- creating specialized variant IDs;
- expected-result inference overriding contradictory payload evidence;
- Family capture eagerly fixing local generics too early.

Hostile cases:
- `Literal(42)` -> `Expr<Int>` and `Literal("x")` -> `Expr<String>` share one constructor identity;
- generic enum `Container<T>` + generic variant `V<U>` publishes separate substitutions;
- expected result can solve a result-only local generic where ordinary generic calls already permit it;
- argument/result conflict rejects;
- bound failure rejects;
- captured generic variant Family can be invoked at different instantiations if Family semantics permit repeated use.

Required evidence:
1. `cargo test -p phalcom-semantic --test semantic semantic::adts::constructors -- --nocapture` — existing constructor behavior remains green;
2. focused `semantic::adts::generics` additions — generic variant argument/expected result/bounds/owner domains;
3. `cargo test -p phalcom-semantic --test semantic semantic::adts::associated -- --nocapture` — associated family resolution remains canonical;
4. `cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture` or the repository's focused Family invocation modules — generic target retention/application;
5. identity assertion that `VariantConstructorId` is unchanged across `U=Int` / `U=String` calls.

Do not run yet:
- matching/GADT suites — C7;
- full ADT suite — after C7;
- runtime ADT execution suite — C10/Final Gate unless semantic lowering changed materially.

Escalate immediately if:
- associated resolution creates generic specializations before canonical target selection;
- Family representation cannot retain `VariantConstructorId` plus signature without changing rank semantics;
- current constructor application directly substitutes only enum declaration parameters and cannot accept local domain without bypassing C1 application APIs.

Checkpoint completion:
- [ ] generic variant introduction works;
- [ ] separate domains published;
- [ ] expected-result/conflict laws pass;
- [ ] stable constructor identity proven;
- [ ] Family integration proven;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(adts): instantiate variant-local generics through canonical call inference`
- `feat(families): retain generic variant constructor targets`
- `test(adts): certify generic variant introduction`

---

## Task 29 — Compose enum + variant generic domains for construction

Purpose:
Reuse C1 multi-domain application for variant construction.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- associated/variant invocation path;
- C1 generic application-domain product.

Inspect before editing:
- current code collecting owner supplied arguments for variants;
- current use of `VariantConstructorSignature.parameters` and `result_type_template`;
- how call-site expected result reaches variant construction.

Source of truth:
- `EnumInfo.generic_signature` + `VariantConstructorSignature.generic_signature`.

Changes:
- construct application domains from enum declaration signature and variant local signature;
- fixed enum arguments come from explicit/specialized enum receiver if present;
- unsolved domains allocate normal inference variables through C1 mechanism.

Must not:
- concatenate canonical signatures;
- create per-variant solver class.

Testing classification:
- C5 generic ADT construction tests.

---

## Task 30 — Solve payload arguments and canonicalize result

Purpose:
Use payload parameters as normal inference constraints and publish canonical result/exact case.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- variant constructor call application;
- `TypeStore` application/materialization helpers.

Source of truth:
- canonical constructor parameter/result templates under solved substitution.

Changes:
- constrain payload actuals against constructor parameter templates;
- reconcile all domains;
- substitute into result/exact-case template;
- intern/canonicalize final type form.

Must not:
- leave ephemeral inference terms in `VariantInfo` or call result.

Testing classification:
- C5 focused tests.

---

## Task 31 — Expected-result inference and conflicts for variants

Purpose:
Match ordinary generic callable bidirectional behavior.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- expected result forwarding into variant application;
- generic conflict diagnostics.

Inspect before editing:
- ordinary call expected-result constraints in `generic_application.rs`;
- current GADT constructor result compatibility handling.

Source of truth:
- one constraint set; no arbitrary argument-vs-result precedence.

Changes:
- record expected result constraints against constructor result template;
- reconcile with payload-derived constraints;
- report conflict/constraint failure through existing diagnostics.

Must not:
- default underconstrained local parameters from bounds;
- silently ignore expected result for variants if ordinary generic call path already supports it.

Testing classification:
- C5 expected-result and conflict hostile cases.

---

## Task 32 — Preserve stable `VariantConstructorId`

Purpose:
Make generic instantiation specialization data, not executable identity.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `InvocationTargetId::VariantConstructor` publication;
- Family target construction;
- semantic lowering target selection.

Source of truth:
- exact `VariantId` / `VariantConstructorId`.

Changes:
- no new specialized target ID;
- add tests comparing invocation target identity across instantiations.

Must not:
- encode `U` substitutions in `VariantConstructorId`.

Testing classification:
- C5 identity assertion.

---

## Task 33 — Integrate generic variant Families

Purpose:
Preserve rank-1 polymorphism for first-class variant constructor families.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `types/family.rs`;
- `checker/associated.rs`;
- Family invocation target extraction.

Inspect before editing:
- current `Family` members for variants;
- current target extraction funnel cited by SC-4 state;
- generic method Family instantiation path.

Source of truth:
- retained exact `InvocationTargetId` plus canonical constructor signature.

Changes:
- when a retained variant constructor target is invoked, load its `VariantConstructorSignature.generic_signature` and route through ordinary application;
- do not eagerly instantiate during Family capture.

Must not:
- add `forall` to ordinary `TypeData`;
- clone Family members per type application.

Testing classification:
- C5 Family test.

---

## Task 34 — Generic variant construction hostile suite

Purpose:
Defeat easy wrong implementations before adding GADT existential complexity.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only

Owned files and symbols:
- `phalcom-semantic/tests/semantic/adts/generics.rs`;
- Family tests if separate.

Required cases:
- argument-derived local generic;
- enum + local generic simultaneous solve;
- expected-result-only local generic if semantically expressible;
- bound does not invent a solution;
- payload/result conflict;
- stable `VariantConstructorId`;
- generic Family application.

Testing classification:
- C5 ownership-layer evidence.

---

# Checkpoint C6 — Scoped Rigid Variable Kernel

Tasks:
- Task 35 — Audit local/scoped type representation and select rigid ownership seam.
- Task 36 — Add `RigidScopeId`, `RigidTypeVariableId`, origin/kind metadata.
- Task 37 — Extend local type terms/walking with rigid leaves.
- Task 38 — Make equality/relation/substitution rigid-aware without making rigids solvable.
- Task 39 — Add free-rigid/scope analysis.
- Task 40 — Add publication guards and alpha-normalized comparison support.
- Task 41 — Unit-test the rigid kernel independently of GADT matching.

Why this is a checkpoint:

Rigid variables are a reusable semantic primitive, not a GADT pattern special case. Integrating them into matching before the type kernel can safely compare, walk, scope-check, and reject publication would make debugging impossible. C6 isolates that new primitive and proves its core laws before GADT branches consume it.

Entry conditions:
- C5 COMPLETE;
- canonical generic variants exist and publish local binders;
- no GADT branch changes have begun.

Working set:

Primary:
- `phalcom-semantic/src/types/` scoped/local type representation discovered during Task 35;
- `types/type_lambda.rs` / scoped type arena if suitable;
- `checker/inference.rs` only where inference terms need a rigid leaf;
- `types/relation.rs`;
- `types/substitution.rs` / local substitution equivalent;
- metadata/export publication guard;
- test modules under semantic foundations or internal unit tests.

Secondary — inspect only if evidence requires it:
- `types/store.rs` — only to confirm rigids should not become canonical `TypeData`;
- `types/environment.rs`;
- diagnostics formatting.

Out of scope for this checkpoint:
- pattern matching;
- GADT branch proof generation;
- existential escape at user boundaries;
- exact-case opening.

Semantic contract established by this checkpoint:
- a rigid is fixed, scoped, kinded, and distinct from inference variables/parameters;
- one rigid can occur inside composite local types;
- same rigid compares identically to itself, different rigids do not become equal without explicit proof;
- ordinary inference cannot assign a rigid;
- free-rigid walking can determine scope containment;
- rigid-containing local types cannot be exported as canonical metadata;
- alpha-equivalent scoped representations can be compared without raw rigid ID dependence.

Semantic risks:
- polluting global `TypeStore` with branch-local IDs;
- allowing `InferenceSession` to bind a rigid;
- treating rigid equality as subtype compatibility;
- missing a composite type walker branch and allowing hidden escape later;
- using raw rigid allocation IDs in fingerprints.

Hostile cases:
- `κ1` cannot unify with `Int` merely because a flexible variable could;
- `α = κ1` may solve flexible `α` to rigid `κ1` locally if the solver semantics require it, but never solves `κ1` itself;
- `κ1 != κ2` absent proof;
- `List<κ1>` free-rigid set contains `κ1`;
- metadata export rejects rigid-containing form;
- alpha-equivalent scopes with differently numbered IDs compare structurally equivalent for incremental tests.

Required evidence:
1. internal/unit tests for rigid identity, kind, and scope;
2. inference/relation test proving rigid is not an assignment target;
3. composite free-rigid walker tests;
4. metadata publication rejection test;
5. alpha-normalization/equivalence helper test;
6. `cargo test -p phalcom-semantic --lib checker::inference::tests:: -- --nocapture` — proves flexible inference baseline remains green if inference internals changed.

Do not run yet:
- ADT matching tests — C7;
- full semantic package — C10;
- incremental suite — C10 after branch products exist.

Escalate immediately if:
- no existing scoped/local type representation can carry a rigid without globally interning it;
- relation APIs accept only canonical `TypeId` and would require a broad type-kernel redesign to compare local terms;
- metadata/published snapshots currently require every branch binding type to be a canonical `TypeId` with no local view layer.

Checkpoint completion:
- [ ] rigid representation selected and documented;
- [ ] identities/scopes/kinds implemented;
- [ ] relation/inference rules proven;
- [ ] free-rigid analysis exists;
- [ ] publication guard exists;
- [ ] alpha-equivalence support exists;
- [ ] no GADT pattern semantics changed yet;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(types): add scoped rigid type variables`
- `feat(semantic): enforce rigid relation and publication laws`
- `test(types): certify rigid variable kernel`

---

## Task 35 — Audit and choose the rigid representation seam

Purpose:
Prevent a premature `TypeData::Rigid` design from violating existing canonical-store/publication architecture.

Risk:
- Semantic: HIGH
- Implementation fanout: investigate-before-edit

Owned files and symbols:
- `phalcom-semantic/src/types/type_lambda.rs` and scoped type arena;
- `types/id.rs` scoped IDs;
- `checker/inference.rs::InferenceTerm`;
- `types/store.rs::TypeData`;
- branch binding type publication APIs.

Inspect before editing:
1. how scoped lambda-bound variables are represented;
2. how `ScopedTypeData::Free(TypeId)` composes with global types;
3. whether branch-local binding facts can store a local/scoped type view;
4. how relation APIs consume types;
5. how metadata exporter distinguishes global/scoped forms.

Do not inspect unless evidence forces expansion:
- parser;
- runtime.

Dependencies:
- C5 complete.

Source of truth:
- semantic scope identity; canonical global store remains authority for durable types.

Implementation boundary:

INVESTIGATE-BEFORE-EDIT:
Choose one of these only after inspection:

A. extend existing scoped type arena with `ScopedTypeData::Rigid(...)`;
B. introduce a checker-local `LocalTypeTerm` that can reference canonical `TypeId` plus rigid leaves;
C. another existing repository abstraction already supports rigid/free local variables.

Reject globally interned `TypeData::Rigid` unless repository constraints prove all branch-local types are necessarily `TypeId` and a scoped non-exportable arena cannot be integrated without disproportionate redesign. If forced into global store, require a hard scope/non-exportable tag and explicit evidence that incremental/store lifetime cannot leak stale rigids.

Testing classification:
- no behavior test; Task 41 certifies selected design.

Checkpoint state update:
Record chosen representation and rejected alternatives with concrete evidence.

---

## Task 36 — Add rigid IDs, scope IDs, and metadata

Purpose:
Give branch existential variables stable identity inside one analysis scope.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- selected type/local-id module from Task 35.

Source of truth:
- monotonic scope-local allocator owned by checking context/session, not global semantic identity.

Changes:
- add compact `RigidScopeId`;
- add compact `RigidTypeVariableId`;
- add kind and origin lookup/record;
- allocator creates distinct IDs per scope/binder.

Must not:
- derive durable equality from raw numeric ID across analyses;
- use binder name as identity.

Code instructions:

STRUCTURAL:
```rust
struct RigidTypeVariable {
    id: RigidTypeVariableId,
    scope: RigidScopeId,
    kind: KindId,
    origin: RigidOrigin,
}
```

Testing classification:
- Task 41.

---

## Task 37 — Extend local type terms and walkers

Purpose:
Allow rigids inside `Expr<κ>`, tuples, records, callables, and other supported composite local forms.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- selected local/scoped type enum;
- type walkers/materializers/formatters.

Inspect before editing:
- every exhaustive match over selected type enum;
- recursive free-variable/substitution functions.

Source of truth:
- local type term structure.

Changes:
- add rigid leaf;
- update recursive walkers;
- update formatting to stable diagnostic form such as `κ#scope.index` internally, user-friendly origin externally.

Must not:
- accidentally materialize rigid leaf into public `TypeId`.

Optional compile checkpoint:
```bash
cargo check -p phalcom-semantic
```
Reason: exhaustive Rust matches provide useful migration evidence after adding a new local type variant.

Testing classification:
- Task 41.

---

## Task 38 — Make relation/inference rigid-aware

Purpose:
Permit constraints involving rigids without making them flexible.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `types/relation.rs` or local relation equivalent;
- `checker/inference.rs` term/unification code.

Inspect before editing:
- flexible variable assignment branch;
- type parameter handling;
- exact-case inference term handling.

Source of truth:
- rigid identity equality and explicit branch proof substitutions, not solver candidate choice.

Required rules:
```text
rigid κ == rigid κ       true
rigid κ1 == rigid κ2     false unless proof layer explicitly relates them
flex α == rigid κ        α may be solved to κ locally if type-term representation supports it
rigid κ == concrete T    cannot solve κ := T
```

Must not:
- route rigid through `InferVarId` assignment map.

Testing classification:
- C6 inference hostile test.

---

## Task 39 — Add free-rigid and scope analysis

Purpose:
Provide the reusable primitive C8 needs for escape checking.

Risk:
- Semantic: HIGH
- Implementation fanout: local/multi-file

Owned files and symbols:
- type walker utility module selected in Task 35.

Source of truth:
- rigid leaves appearing structurally in local type term.

Changes:
- `free_rigids(type)` or equivalent;
- `contains_rigid_from_scope(type, scope)` or equivalent;
- walk all composite forms.

Must not:
- special-case only top-level rigid type.

Testing classification:
- C6 composite walker tests.

---

## Task 40 — Publication guards and alpha-normalized comparison

Purpose:
Prevent local IDs from becoming durable and prepare incremental parity.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- metadata exporter boundary;
- snapshot/canonical materialization boundary;
- stable comparison/fingerprint helpers used by incremental tests.

Inspect before editing:
- existing `MetadataExportError::InferenceVariable` / non-exportable handling;
- existing scoped lambda alpha-equivalence/fingerprint machinery.

Source of truth:
- durable metadata contains only canonical globally meaningful forms; scoped binders compare structurally.

Changes:
- rigid-containing publication returns a dedicated/non-exportable error;
- incremental structural comparison renumbers local rigids by binder introduction order/scope nesting rather than raw allocator ID.

Must not:
- serialize raw rigid IDs;
- silently widen rigids during metadata export.

Testing classification:
- C6 publication/alpha tests.

---

## Task 41 — Rigid kernel tests

Purpose:
Certify the new primitive before GADT integration.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only

Owned files and symbols:
- unit tests colocated with new type module and/or `semantic::foundations`.

Required cases:
- identity/freshness;
- kind retention;
- rigid non-assignment;
- flexible-to-rigid local solution if supported;
- nested composite free-rigid walk;
- metadata rejection;
- alpha-equivalent local scopes.

Testing classification:
- C6 required evidence.


# Checkpoint C7 — Full GADT Elimination with Constructor-Local Existentials

Tasks:
- Task 42 — Introduce `CaseInstantiation` or equivalent branch-local product.
- Task 43 — Allocate one rigid substitution per matched generic variant case.
- Task 44 — Instantiate payload/result/local constraints through the shared rigid substitution.
- Task 45 — Extend GADT equality proofing to rigid-containing local types.
- Task 46 — Publish payload bindings and branch proofs with shared rigid identity.
- Task 47 — Integrate variant-local bounds into branch evidence.
- Task 48 — Extend exhaustiveness/reachability and add hostile GADT tests.

Why this is a checkpoint:

This is the semantic heart of the program. Construction has already been completed in C5; C7 adds the dual elimination rule. The entire checkpoint must integrate before the behavior is meaningful: fresh rigids, payload specialization, result-index equations, branch proofing, and pattern bindings all need the same case instantiation.

Entry conditions:
- C5 COMPLETE;
- C6 COMPLETE;
- no active incident in the rigid kernel;
- existing declaration-indexed GADT tests are green before edits.

Working set:

Primary:
- `phalcom-semantic/src/types/case_environment.rs` — existing declaration-index equation authority;
- new `types/case_instantiation.rs` or checker-local equivalent if justified;
- `phalcom-semantic/src/checker/gadt_proof.rs`;
- pattern matching code under `phalcom-semantic/src/checker/` discovered by drift check;
- `phalcom-semantic/src/checker/exhaustiveness.rs`;
- `phalcom-semantic/src/checker/pattern_space.rs`;
- `phalcom-semantic/src/enum_semantics.rs::VariantConstructorSignature`;
- `phalcom-semantic/tests/semantic/adts/matching.rs`;
- `phalcom-semantic/tests/semantic/adts/vertical_gadt.rs`;
- `phalcom-semantic/tests/semantic/adts/generics.rs`.

Secondary — inspect only if evidence requires it:
- flow/refinement modules that consume branch proofs;
- explanation traces;
- exact-case tests, deferred to C8 unless needed for compilation.

Out of scope for this checkpoint:
- general existential packages;
- escape of branch-local values beyond basic internal scope representation;
- closure capture policy implementation;
- native generic variants;
- runtime witness passing.

Semantic contract established by this checkpoint:
- variant-local universal binders become fresh existential rigids at elimination;
- one local binder maps to one rigid per case instantiation;
- independent matches get fresh rigids;
- payload/result/local constraints share that substitution;
- declaration-index GADT proofs continue through existing proof authority;
- local variant bounds become branch evidence;
- impossible cases are proved/refuted without guessing rigid identities.

Semantic risks:
- allocating one rigid per payload occurrence rather than per binder;
- reusing one rigid across independent match occurrences;
- solving a rigid to satisfy GADT equality;
- putting branch-local equations into global generic state;
- conflating local binder substitution with `CaseTypeEnvironment.bindings`;
- losing existing exact-case/exhaustiveness proof behavior.

Hostile cases:
- `Equal<U>(left: Expr<U>, right: Expr<U>)` gives exactly one shared rigid;
- two separate `Equal` matches produce distinct rigids;
- `Wrap<U> -> Expr<List<U>>` proves outer index `X = List<κ>`;
- `U <: Show` is usable in branch but does not become a concrete `Show` type;
- a case requiring `κ = Int` without evidence remains unresolved/incompatible, never guessed;
- nested GADT matches merge declaration-index proofs without merging unrelated rigids;
- impossible indexed case remains impossible.

Required evidence:
1. `cargo test -p phalcom-semantic --test semantic semantic::adts::vertical_gadt -- --nocapture` — preserves/extends indexed GADT core;
2. `cargo test -p phalcom-semantic --test semantic semantic::adts::matching -- --nocapture` — proves branch integration/exhaustiveness;
3. focused additions in `semantic::adts::generics` for shared/fresh local existentials;
4. internal assertion/test that one binder maps to one rigid in `CaseInstantiation`;
5. hostile test that unrelated matches do not share a rigid;
6. hostile test that rigid is not solved to make a branch succeed;
7. `cargo test -p phalcom-semantic --test semantic semantic::adts::exact_cases -- --nocapture` only if C7 changes exact-case proof code transitively; otherwise defer to C8.

Do not run yet:
- full ADT suite until C8;
- full semantic package — C10;
- incremental suite — C10;
- workspace — Final Gate.

Escalate immediately if:
- existing branch binding/publication representation cannot carry the local rigid-containing type selected in C6;
- GADT proof solver is architecturally tied to `TypeSubstitution<TypeParameterId, TypeId>` with no safe local-term extension;
- exhaustiveness requires durable canonical types for branch-local payloads, forcing a broader local-type abstraction than C6 anticipated.

Checkpoint completion:
- [ ] fresh case instantiation implemented;
- [ ] shared-rigid law passes;
- [ ] independent freshness law passes;
- [ ] result-index proof with rigid passes;
- [ ] local constraint evidence passes;
- [ ] existing declaration-index GADT tests remain green;
- [ ] no global generic mutation introduced;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(gadt): open variant-local binders as scoped existentials`
- `feat(gadt): merge rigid result indices into branch proofs`
- `test(gadt): certify shared identity freshness and local bounds`

---

## Task 42 — Introduce `CaseInstantiation`

Purpose:
Separate fresh existential opening from the existing declaration-owned `CaseTypeEnvironment`.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new local product module near `types/case_environment.rs` or checker GADT module;
- `VariantConstructorSignature` consumers.

Inspect before editing:
- how `CaseTypeEnvironment` is currently cloned/consumed;
- `BranchProofEnvironment` construction;
- pattern payload type specialization.

Dependencies:
- C6 rigid type API.

Source of truth:
- canonical variant constructor signature + fresh branch scope.

Implementation boundary:

Changes:
- create one product per candidate/exact case elimination;
- retain variant ID, rigid scope, binder-to-rigid map, instantiated local constraints/result/payload view.

Must not:
- add local rigids to canonical `VariantInfo`;
- modify `CaseTypeEnvironment` to own fresh variables.

Code instructions:

STRUCTURAL:
```rust
pub(crate) struct CaseInstantiation {
    pub variant: VariantId,
    pub scope: RigidScopeId,
    pub local_rigids: BTreeMap<TypeParameterId, RigidTypeVariableId>,
    // repository-native local/scoped substitutions/products
}
```

Keep only data actually needed across branch proof/payload binding steps. Do not overbuild a persistent object graph.

Testing classification:
- no standalone test beyond Task 48; low-level map identity may have a unit assertion.

---

## Task 43 — Allocate one rigid per variant-local binder per elimination

Purpose:
Enforce shared existential identity.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `CaseInstantiation` constructor/opening helper;
- C6 rigid allocator.

Source of truth:
- `VariantConstructorSignature.generic_signature.parameters` in source order.

Changes:
- create a fresh rigid scope for each elimination occurrence;
- allocate exactly one rigid for each constructor-local parameter;
- preserve kind from `TypeParameterData`/generic signature;
- store map once.

Must not:
- allocate while recursively walking each payload field;
- cache case instantiation globally by `VariantId`.

Code instructions:

EXACT semantic loop:
```text
for each parameter in constructor generic signature:
    allocate one rigid with parameter.kind and current case scope
    local_rigids[parameter] = rigid
```

Testing classification:
- C7 shared/freshness tests.

---

## Task 44 — Instantiate payload, result, and local constraints with one rigid substitution

Purpose:
Make all occurrences of the same hidden binder agree.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- local type substitution/materialization utility selected in C6;
- `VariantConstructorSignature.parameters`;
- `result_type_template`;
- local `GenericSignature.constraints`.

Inspect before editing:
- current declaration substitution path for variant payload/result;
- handling of `TypeTerm::SelfType` / canonical terms in constraints.

Source of truth:
- one `TypeParameterId -> rigid local term` mapping.

Changes:
- substitute local binder occurrences in all constructor templates;
- combine with already-known enum declaration receiver substitution as appropriate;
- retain local constraints in branch-local form.

Must not:
- canonicalize a rigid-containing result into global `TypeStore` if C6 selected scoped local forms.

Testing classification:
- C7 GADT tests.

---

## Task 45 — Extend GADT equality proofing for rigid-containing terms

Purpose:
Reuse `solve_gadt_branch_proof` for declaration-index equations that contain local rigids.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/gadt_proof.rs::solve_gadt_branch_proof`;
- equality normalization/unification helper;
- local type relation API from C6.

Inspect before editing:
- current `TypeSubstitution` use;
- where exact equality is distinguished from subtype;
- cycle/normalization behavior.

Source of truth:
- branch equality relation; rigids are opaque constants unless proof equates surrounding declaration variables to terms containing them.

Required behavior:
```text
X flexible/declaration parameter may be refined to List<κ>
κ itself is never rewritten to satisfy X's prior shape
```

Must not:
- store `κ` in global declaration generic substitution outside branch;
- treat superclass/subtyping as exact equality.

Testing classification:
- C7 `Wrap<U>` index proof + rigid guessing hostile case.

---

## Task 46 — Publish payload bindings and branch proofs with shared rigids

Purpose:
Make pattern-bound values use the exact same local identities consumed by proofs.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- pattern binding creation path;
- branch type/refinement environment;
- `BranchProofEnvironment`.

Inspect before editing:
- current field payload specialization after variant resolution;
- pattern binding contract/current knowledge representation;
- nested match proof merge.

Source of truth:
- `CaseInstantiation` local payload view + branch proof.

Changes:
- bind `left`/`right` etc. from already-instantiated payload templates;
- apply declaration-index branch proof without replacing local rigid identity;
- ensure nested matches preserve scope ownership.

Must not:
- independently respecialize each payload field;
- materialize two different rigids for repeated local parameter occurrences.

Testing classification:
- C7 shared identity/nested tests.

---

## Task 47 — Publish variant-local constraints as branch evidence

Purpose:
Make `where U <: Trait` useful after existential opening.

Risk:
- Semantic: HIGH
- Implementation fanout: local/multi-file

Owned files and symbols:
- branch proof/evidence environment;
- generic constraint checking/relation API.

Inspect before editing:
- how generic constraints are represented after substitution;
- how branch-local subtype facts are consumed by calls/member resolution.

Source of truth:
- instantiated local generic constraints from `CaseInstantiation`.

Changes:
- introduce branch-local relation evidence for rigid-containing constraints;
- ensure evidence scope ends with branch;
- do not synthesize runtime witness.

Must not:
- replace `κ` with upper bound type;
- publish local bound globally.

Testing classification:
- C7 local bound positive/negative tests.

---

## Task 48 — Exhaustiveness, reachability, and GADT hostile suite

Purpose:
Prove full elimination composes with existing pattern-space/exhaustiveness semantics.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file/test-heavy

Owned files and symbols:
- `checker/exhaustiveness.rs`;
- `checker/pattern_space.rs`;
- ADT matching/vertical GADT tests.

Inspect before editing:
- initial pattern space construction for `ExactCase`;
- case compatibility check;
- missing witness generation.

Source of truth:
- canonical variant identity + branch proof satisfiability.

Changes:
- where candidate reachability depends on result-index equations containing a rigid, use the local case instantiation/proof result;
- do not enumerate concrete possibilities for the rigid;
- preserve existing impossible-case behavior.

Required hostile tests:
- shared rigid;
- fresh independent rigids;
- `X = List<κ>` proof;
- no rigid guessing;
- nested local + declaration proof;
- impossible indexed case;
- exhaustive match still recognized.

Testing classification:
- C7 primary evidence.

---

# Checkpoint C8 — Existential Non-Escape and Exact-Case Reconstruction

Tasks:
- Task 49 — Define all scope-exit/publication boundaries that can leak local rigids.
- Task 50 — Enforce match-result non-escape.
- Task 51 — Enforce outer assignment/container/call boundary non-escape.
- Task 52 — Enforce closure capture policy.
- Task 53 — Permit safe rigid-free widening/abstraction.
- Task 54 — Reconstruct hidden generic locals when eliminating exact cases.
- Task 55 — Add hostile escape/exact-case suite and diagnostic coverage.

Why this is a checkpoint:

C7 makes constructor-local existential types usable inside a branch. Without C8, the system is unsound because those scoped identities can leak. Escape enforcement also cannot be limited to branch return expressions: assignment, aggregate construction, closure capture, and exact-case payload observation all cross semantic publication boundaries.

Entry conditions:
- C7 COMPLETE;
- C6 free-rigid/scope walker available;
- no active rigid/GADT incident.

Working set:

Primary:
- match expression result typing / branch joins;
- binding assignment/flow publication paths;
- return checking;
- closure capture/publication code;
- collection/tuple/record construction type publication if branch results flow through them;
- `TypeData::ExactCase` consumers in substitution/environment/associated/pattern code;
- diagnostics;
- ADT exact-case and new existential escape tests.

Secondary — inspect only if evidence requires it:
- compiler lowering, only if semantic publication cannot block an invalid branch cleanly;
- LSP presentation of local invalid types.

Out of scope for this checkpoint:
- user-visible existential packaging syntax;
- runtime existential boxes;
- native generic metadata;
- runtime storage.

Semantic contract established by this checkpoint:
- no branch-local rigid appears in a durable outward type;
- structural wrappers do not hide escape;
- an escaping closure cannot capture a rigid-dependent value in the first version;
- sound widening to a rigid-free supertype is allowed;
- exact-case payload observation freshly opens constructor-local generics rather than persisting them globally;
- metadata remains a final hard guard.

Semantic risks:
- checking only explicit `return`;
- allowing `Option<κ>`/`List<κ>` to escape because top-level type is not rigid;
- falsely rejecting safe widening to `Object`/another declared abstraction;
- letting closure environment act as an untracked existential package;
- storing fresh rigid in canonical exact-case `TypeId`;
- reporting generic inference failure instead of existential escape.

Hostile cases:
- direct branch result escape rejected;
- outer mutable variable assignment rejected;
- collection/wrapper escape rejected;
- call requiring concrete incompatible type rejected;
- escaping closure capture rejected;
- rigid-free widening allowed;
- two reads/eliminations of same exact case get fresh existential scopes unless proof ties them;
- exact case remains canonical `TypeData::ExactCase { variant, enum_type }`.

Required evidence:
1. new `semantic::adts::existentials` test module with direct/wrapper/assignment/call/widening cases;
2. closure capture hostile test in the same module or callable-publication suite;
3. `cargo test -p phalcom-semantic --test semantic semantic::adts::exact_cases -- --nocapture` — exact-case canonical behavior + hidden local reconstruction;
4. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::flow_branches -- --nocapture` if branch join/publication code changed;
5. `cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication -- --nocapture` if closure capture code changed;
6. metadata export negative test for escaped rigid as defense-in-depth.

Do not run yet:
- full semantic package — C10;
- native/core ADT suite — C9/C10;
- workspace — Final Gate.

Escalate immediately if:
- branch result joins currently require canonical global `TypeId` before escape can be checked;
- closure capture types are erased/unknown before semantic escape analysis can inspect them;
- safe widening cannot be represented without introducing a general existential package.

Checkpoint completion:
- [ ] all identified escape boundaries enforce scope rule;
- [ ] direct/wrapper/assignment/call/closure hostile cases pass;
- [ ] safe widening passes;
- [ ] exact-case hidden local reconstruction passes;
- [ ] canonical exact-case shape preserved;
- [ ] metadata hard guard passes;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(gadt): prevent constructor existential escape`
- `feat(adts): reconstruct hidden generics from exact cases`
- `test(gadt): certify escape and exact-case laws`

---

## Task 49 — Enumerate and codify escape boundaries

Purpose:
Turn non-escape into a general scope publication rule rather than a return-expression special case.

Risk:
- Semantic: HIGH
- Implementation fanout: investigate-before-edit / multi-file

Owned files and symbols:
- branch/match result checker;
- binding assignment checker;
- return checker;
- closure capture publication;
- function/call argument compatibility where local type may be coerced/published.

Inspect before editing:
- every path that moves branch-local `TypeKnowledge` into an outer-scope durable binding/result;
- flow joins and contract/current knowledge separation;
- closure environment capture registration.

Source of truth:
- rigid defining scope + free-rigid set of outward type.

Implementation boundary:

INVESTIGATE-BEFORE-EDIT:
Create a small internal checklist/API for "may this semantic type leave scope S?" and reuse it at publication boundaries. Do not add independent escape logic to each caller beyond invoking the shared check and choosing diagnostics/recovery.

Testing classification:
- C8 hostile suite.

Checkpoint state update:
Record the exact list of protected boundaries and owning symbols.

---

## Task 50 — Match-result non-escape

Purpose:
Reject match expressions whose joined/public outward type contains a branch-local rigid.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- match expression branch result/join path.

Source of truth:
- branch-local scope ownership + outward joined type.

Changes:
- before publishing branch result outside scope, inspect free rigids;
- emit dedicated existential escape diagnostic;
- do not widen automatically unless an existing expected type legitimately causes a sound rigid-free abstraction.

Must not:
- turn result into `Dynamic` merely to continue.

Testing classification:
- C8 direct return/result hostile test.

---

## Task 51 — Outer assignment, aggregate, and call-boundary non-escape

Purpose:
Close indirect leaks.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- assignment/flow state update;
- aggregate type construction if it publishes branch result;
- argument compatibility when outward receiver/callee contract is outside rigid scope.

Source of truth:
- type actually published to outer contract, not origin of value.

Changes:
- reject outer binding acquiring `κ` or `Container<κ>`;
- allow passing value through a contract that erases/widens to rigid-free type when normal assignability proves it;
- preserve branch-local uses.

Must not:
- ban all use of existential values outside pattern binding expression while still inside branch.

Testing classification:
- C8 wrapper/assignment/call tests.

---

## Task 52 — Closure capture policy

Purpose:
Prevent closures from becoming accidental first-class existential packages.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- closure capture analysis;
- callable publication/capture metadata.

Inspect before editing:
- distinction between immediately invoked/non-escaping block and stored/returned closure if such escape analysis exists;
- current closure capture type facts.

Source of truth:
- published closure/capture lifetime relative to rigid scope.

Changes:
- first-version conservative rule: if closure can leave rigid scope and captures a rigid-dependent value/type, reject;
- if existing compiler has a proven non-escaping closure classification, it may permit local use only if capture lifetime is bounded by the branch.

Must not:
- silently erase captured type to `Dynamic`.

Testing classification:
- C8 closure hostile case.

---

## Task 53 — Safe widening/abstraction

Purpose:
Avoid making existential opening unusably strict.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- expected-type/assignability path at escape boundaries.

Source of truth:
- normal subtype/assignability relation under branch evidence.

Changes:
- check final externally published type after sound coercion/widening;
- if it contains no rigid, allow escape;
- do not invent widening merely to hide a rigid unless ordinary context/annotation calls for it.

Example:
```text
κ <: Object
value : κ
outward declared result : Object
→ allowed if ordinary assignability proves κ <: Object
```

Testing classification:
- C8 positive widening test.

---

## Task 54 — Exact-case hidden local reconstruction

Purpose:
Preserve compact canonical exact-case identity while making generic variant payloads usable.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- exact-case payload/member/pattern observation paths;
- `TypeData::ExactCase` consumers;
- C7 case instantiation helper.

Inspect before editing:
- exact-case narrowing in `associated.rs`;
- substitution/materialization of exact cases;
- exhaustive pattern handling.

Source of truth:
- exact `VariantId` + canonical `VariantConstructorSignature`.

Changes:
- on elimination/observation, create a fresh `CaseInstantiation` from constructor signature;
- never persist the fresh rigid in `TypeData::ExactCase`;
- apply enum result specialization as before.

Must not:
- add `hidden_arguments` containing local rigids to canonical exact case unless repository evidence proves unavoidable and a new durable existential representation is ratified.

Testing classification:
- C8 exact-case tests.

---

## Task 55 — Existential diagnostics and hostile suite

Purpose:
Make soundness failures clear and prove wrappers/captures cannot bypass the rule.

Risk:
- Semantic: HIGH
- Implementation fanout: test/diagnostic

Owned files and symbols:
- `phalcom-semantic/src/diagnostic.rs` diagnostic code registry;
- new ADT existential test module.

Inspect before editing:
- current GADT/index diagnostic codes;
- diagnostic source span policy for pattern branches.

Source of truth:
- scope escape event.

Changes:
- add `ExistentialEscape` / repository-consistent diagnostic code;
- message identifies hidden local origin and outward type when possible;
- test exact diagnostic classification, not brittle full prose unless repository conventions require snapshots.

Required cases:
- direct escape;
- aggregate escape;
- assignment;
- incompatible concrete use;
- closure capture;
- safe widening;
- exact-case fresh opening.

Testing classification:
- C8 primary evidence.


# Checkpoint C9 — Native, Generated, Intrinsic, and Durable Metadata Parity

Tasks:
- Task 56 — Extend native callable metadata/import for generic signatures.
- Task 57 — Route native generic callables through canonical `TypeParameterOwner::Callable` products.
- Task 58 — Extend native/core variant metadata if generic variant constructors are represented there.
- Task 59 — Update compiler-generated accessor/member construction for complete AST contracts.
- Task 60 — Ensure metadata export preserves separated declaration/callable ownership and variant local generics.
- Task 61 — Add source/native/generated parity tests.

Why this is a checkpoint:

The source implementation is not complete if core/native/generated declarations require different generic semantics. The repository already converges native callables on `CallableSemanticSignature`; C9 extends the metadata/import source so that convergence is real rather than `generics: None` everywhere. This checkpoint also prevents newly added AST fields from being omitted by compiler synthesis.

Entry conditions:
- C5 COMPLETE for generic variant introduction;
- C8 COMPLETE for full source semantics;
- no unresolved generic ownership incident.

Working set:

Primary:
- `phalcom-native-meta` generic/native surface schema;
- `phalcom-native-surface`;
- `phalcom-native-surface-gen`;
- `phalcom-semantic/src/types/native.rs` or current native registration/import module;
- `phalcom-semantic/src/metadata/export.rs`;
- `phalcom-type-meta/src/generic.rs` / callable metadata records;
- `phalcom-core/src/compiler/attributes.rs` generated getter/setter/index construction;
- native/core ADT semantic tests;
- metadata integration tests.

Secondary — inspect only if evidence requires it:
- runtime reflection consumer of generic callable records;
- core typing inspection APIs;
- native catalog fingerprint implementation.

Out of scope for this checkpoint:
- new runtime applied storage;
- new native language syntax;
- runtime GADT witness storage;
- broad LSP feature work.

Semantic contract established by this checkpoint:
- native/generated/intrinsic declarations can publish the same canonical generic products as source declarations where semantics match;
- native generic binders receive stable callable ownership;
- native metadata does not contain a separate inference model;
- generated AST members initialize new generic/where fields;
- durable metadata can represent constructor-local/variant-local signatures without mixed ownership;
- source and native equivalent callable contracts behave equivalently under inference.

Semantic risks:
- introducing a parallel native generic parameter identity;
- treating metadata binder names as identity;
- changing native selector identity based on generic arity;
- catalog fingerprints ignoring generic constraints;
- generated accessors compiling but fingerprinting differently from source equivalents;
- exporting a combined constructor signature to avoid schema changes.

Hostile cases:
- source and native `identity<T>(T)->T` infer the same type and fail the same bound case;
- source/native getter generics use expected-result inference identically;
- a native generic callable with a bound does not default from the bound;
- two native generic instantiations retain one callable identity;
- generated non-generic setter/indexer remains `generics: None`/empty, not a synthetic bogus binder;
- metadata round-trip preserves parameter owner/index/kind/constraint shape.

Required evidence:
1. focused native metadata/import unit tests — generic binder ownership and constraint lowering;
2. `cargo test -p phalcom-semantic --test semantic semantic::adts::native_core -- --nocapture` if native ADT path changed;
3. source/native generic callable parity semantic tests;
4. `cargo test -p phalcom-semantic --test semantic semantic::integration::metadata -- --nocapture` — generic signatures export/validate;
5. focused `phalcom-core` attribute/generated accessor tests after AST field changes;
6. native catalog fingerprint test proving generic contract edit changes structural fingerprint if that schema participates in cache identity.

Do not run yet:
- full workspace — Final Gate;
- full semantic package — C10, immediately after C9 if C9 touches shared registration broadly.

Escalate immediately if:
- native metadata schema is intentionally monomorphic by ratified design and generic core declarations are instead source-authored shells; determine the authoritative path before changing schema;
- stable callable metadata cannot identify variant-owned constructor callable owners;
- adding generic native metadata would require ABI/version migration beyond this program's declared compatibility boundary.

Checkpoint completion:
- [ ] native generic signature source exists or source-authored canonical alternative documented;
- [ ] native import produces canonical type parameters/signatures;
- [ ] generated accessors updated;
- [ ] metadata round-trip/ownership passes;
- [ ] source/native parity hostile cases pass;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(native-meta): describe callable generic contracts`
- `feat(semantic): import native generics into canonical signatures`
- `fix(core): initialize generic metadata on generated accessors`
- `test(metadata): certify source/native generic parity`

---

## Task 56 — Extend native callable generic metadata

Purpose:
Give native declarations enough structural information to construct canonical generic signatures.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Owned files and symbols:
- `phalcom-native-meta` callable/surface record;
- `phalcom-native-surface-gen` extraction/generation;
- catalog fingerprinting.

Inspect before editing:
- exact current `NativeSurfaceRecord` / callable metadata fields;
- whether generic type syntax is already representable by `phalcom-type-syntax`;
- schema/version compatibility policy;
- how native parameter/result type expressions are currently lowered.

Dependencies:
- C1 owner-separated signatures.

Source of truth:
- native authored metadata converted into the same canonical semantic model.

Implementation boundary:

STRUCTURAL:
Native metadata needs enough information to describe:

```text
generic parameter sequence
parameter names for source presentation only
kind shapes
constraints
```

Identity is not the native string name; identity is created during semantic import from the canonical target `CallableId` + parameter index.

Must not:
- add solver-local or store-local type IDs to static native metadata;
- create a native-only `NativeGenericId` as semantic authority.

Testing classification:
- C9 native metadata tests.

---

## Task 57 — Import native binders as canonical callable-owned parameters

Purpose:
Make native generic surfaces indistinguishable from source after semantic registration.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- current native registration path that initializes an empty type-parameter map and `generics: None`;
- `TypeStore` parameter interning APIs;
- callable signature table.

Inspect before editing:
- source generic signature lowering APIs suitable for reuse;
- native type resolver/type-expression lowering;
- stable source span/provenance handling for native declarations.

Source of truth:
- canonical native target `CallableId`.

Changes:
- create `TypeParameterId`s owned by `TypeParameterOwner::Callable(callable)`;
- build canonical `GenericSignature` with kinds/constraints;
- resolve parameter/return types under that local parameter map;
- publish through ordinary `CallableSemanticSignature`.

Must not:
- route native calls around ordinary application inference.

Testing classification:
- C9 source/native parity.

---

## Task 58 — Extend native/core variant generic metadata only where required

Purpose:
Avoid hardcoded generic behavior for core ADTs if native/core declarations need variant-local generics.

Risk:
- Semantic: HIGH
- Implementation fanout: investigate-before-edit / cross-crate

Owned files and symbols:
- core enum/native declaration representation;
- native ADT registration path;
- `semantic::adts::native_core` tests.

Inspect before editing:
- whether core `Option`/`Result` variants are source-defined, native metadata-defined, or mixed;
- whether any currently planned core variant actually needs variant-local binders.

Source of truth:
- canonical enum/variant semantic declaration, irrespective of source origin.

Implementation boundary:

INVESTIGATE-BEFORE-EDIT:
If native/core variants are already created through source `EnumDef`/`VariantInfo`, do not add a duplicate native variant generic schema. If native metadata owns variant constructor signatures, extend that path minimally to express the same local generic signature product.

Must not:
- hardcode `Option`/`Result` names in generic inference.

Testing classification:
- C9 native ADT tests only if path changed.

---

## Task 59 — Update generated member AST construction

Purpose:
Keep compiler-generated accessors semantically well-formed after AST field additions.

Risk:
- Semantic: LOW
- Implementation fanout: multi-file mechanical

Owned files and symbols:
- `phalcom-core/src/compiler/attributes.rs`;
- any compiler macro/derivation constructing `SetterDef`, `IndexMethodDef`, `VariantDecl`.

Inspect before editing:
- repository-wide `rg 'SetterDef \{|IndexMethodDef \{|VariantDecl \{'`.

Source of truth:
- AST declaration structs.

Changes:
- initialize generic lists to empty and `where_clause` to `None` for generated non-generic members;
- if any generated member is intentionally generic, construct the real binder contract rather than leaving it inconsistent.

Must not:
- invent generic parameters to satisfy struct construction.

Testing classification:
- no standalone semantic test unless generated accessor behavior changes; compile + existing attribute tests at C9.

Optional compile checkpoint:
```bash
cargo check -p phalcom-core --all-targets
```
Reason: exhaustive struct-literal migration and generated compiler code compilation.

---

## Task 60 — Complete durable metadata ownership

Purpose:
Make metadata reflect canonical declaration/callable separation, including variant locals.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/metadata/export.rs`;
- `phalcom-type-meta/src/generic.rs`;
- callable/enum metadata records;
- stable callable identity conversion for `CallableOwnerId::Variant`.

Inspect before editing:
- current `sig_map` key `(TypeParameterOwner, usize)` and whether it assumes owner/count uniquely identifies a signature despite differing constraints;
- stable callable conversion for variant-owned callables;
- enum/variant metadata availability.

Source of truth:
- canonical `GenericSignature` validated against `TypeStore`.

Changes:
- export every canonical signature separately;
- ensure constructor declaration generic signature remains declaration-owned while local constructor signature is callable-owned;
- export variant-local constructor signature with stable owner;
- preserve kind/constraint shapes.

Must not:
- export application-time combined domain object as one generic signature;
- export local rigids.

Investigation note:
If `sig_map` currently deduplicates only by `(owner, parameter_count)`, inspect whether multiple signatures with same owner/count but changed constraints can coexist during one export. If the owner is unique per canonical declaration snapshot, it may be sufficient; do not widen key without evidence.

Testing classification:
- C9 metadata integration.

---

## Task 61 — Source/native/generated parity suite

Purpose:
Prove convergence on the same semantic authority.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only

Owned files and symbols:
- semantic native integration tests;
- metadata tests;
- core attribute tests.

Required cases:
- source vs native identity generic;
- result-only expected context if native metadata can represent it;
- bound pass/fail;
- stable `CallableId` across instantiations;
- generated setter remains valid with no local generics;
- variant generic parity if native variant path exists.

Testing classification:
- C9 required evidence.

---

# Checkpoint C10 — Incremental, Publication, and Semantic Certification

Tasks:
- Task 62 — Extend fingerprints/dependencies for all new generic declaration surfaces.
- Task 63 — Add cold-versus-incremental tests for callable generic contract edits.
- Task 64 — Add cold-versus-incremental tests for variant local binders/GADT proofs.
- Task 65 — Verify alpha-equivalent rigid scopes across incremental reanalysis.
- Task 66 — Run focused protected semantic/core suites and diagnose any integration incidents.
- Task 67 — Update authoritative docs/state and prepare final delivery gates.

Why this is a checkpoint:

Every preceding checkpoint can be locally correct while still violating Phalcom's established cold/incremental publication guarantees. C10 certifies the final semantic products under edits and then runs the broad affected-crate suites once, after all shared changes have landed.

Entry conditions:
- C0–C9 COMPLETE;
- no unresolved incident;
- all focused checkpoint evidence recorded.

Working set:

Primary:
- `phalcom-semantic/src/db/fingerprint.rs`;
- query/dependency registration modules affected by declaration/callable/variant products;
- `phalcom-semantic/tests/semantic/incremental/`;
- semantic test module registry;
- protected `phalcom-core` monad/Either/ADT tests;
- documentation/state files.

Secondary — inspect only if evidence requires it:
- LSP snapshot publication if semantic snapshot schema changed;
- compiler lowering tests if invocation specialization added a required field;
- metadata runtime reflection tests.

Out of scope for this checkpoint:
- new features;
- performance optimization not required for correctness;
- runtime applied storage.

Semantic contract established by this checkpoint:
- every new declaration contract participates in structural invalidation;
- cold and incremental analyses publish equivalent canonical semantic facts;
- rigid allocation IDs do not cause false semantic differences;
- generic Families, GADT proofs, applied receivers, metadata, and source signatures remain coherent after edits;
- all affected semantic tests pass together.

Semantic risks:
- stale constructor/variant signatures reused after binder edit;
- pattern proof not invalidated after variant result-index change;
- local rigid raw IDs entering fingerprints;
- metadata cache failing to invalidate after `where` edit;
- old last-known-good products masking terminal invalid analysis;
- source index/LSP consumers receiving stale callable/variant identity if snapshot products changed.

Hostile cases:
- change constructor `<U>` bound and dependent call invalidates;
- change setter/index local bound and call rechecks;
- add/remove variant local binder or bound and construction + matches recheck;
- change `Wrap<U> -> Expr<List<U>>` result template and GADT proof/exhaustiveness changes;
- incremental and cold analysis agree on type/target/diagnostic despite different raw rigid allocations;
- invalid edit does not publish stale successful GADT proof as current.

Required evidence:
1. focused new incremental tests for callable local binders/constraints;
2. focused new incremental tests for generic variant declaration/construction/matching;
3. cold-vs-incremental structural comparison of branch existential result/proof using alpha-equivalent local binders;
4. `cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture` — all incremental semantic tests;
5. `cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture` — full ADT semantic suite once after C8–C10 integration;
6. `cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture` — Family integration;
7. `cargo test -p phalcom-semantic --test semantic` — full semantic package integration;
8. `cargo test -p phalcom-core --test core monads:: -- --nocapture` and `cargo test -p phalcom-core --test core either:: -- --nocapture` — protected generic/ADT core behavior if still present under these names;
9. targeted core typing/reflection tests if metadata schema changed.

Do not run yet:
- workspace fmt/check/test/clippy — Final Gate; C10 should finish semantic incidents before expensive delivery validation.

Escalate immediately if:
- incremental comparison can only pass by ignoring meaningful rigid scope/binder structure;
- variant edits fail to invalidate match/exhaustiveness consumers because dependency edges do not include enum semantic products;
- full semantic suite reveals an unrelated baseline failure; classify it under failure protocol instead of patching outward.

Checkpoint completion:
- [ ] all new fingerprints/dependencies verified;
- [ ] cold/incremental hostile cases pass;
- [ ] raw rigid IDs absent from durable fingerprints;
- [ ] full semantic package passes;
- [ ] protected core generic/ADT suites pass or documented baseline incident resolved;
- [ ] docs/state updated;
- [ ] no deferred semantic evidence remains before Final Gate;
- [ ] no active incident.

Suggested commit grouping:
- `feat(incremental): track callable and variant generic contract edits`
- `test(incremental): certify existential and generic reanalysis parity`
- `docs(semantic): record type-system completion invariants`

---

## Task 62 — Complete source/interface fingerprints

Purpose:
Ensure every newly added generic contract is part of semantic invalidation.

Risk:
- Semantic: HIGH
- Implementation fanout: local/multi-file

Owned files and symbols:
- `phalcom-semantic/src/db/fingerprint.rs`;
- declaration/interface fingerprint helpers.

Inspect before editing:
- getter's current `hash_generic_contract_source` use;
- setter/index changes from C2;
- enum/variant hashing from C4;
- callable signature structural fingerprinting in metadata/native catalog.

Source of truth:
- authored declaration contract.

Changes:
Verify hashing includes:

```text
constructor local generic binders + where
getter local binders + where (already expected)
setter local binders + where
index getter/setter local binders + where
variant local binders + kinds + where
variant payload/result templates
```

Must not:
- hash call-site inferred substitutions;
- hash rigid IDs.

Testing classification:
- C10 incremental edit tests.

---

## Task 63 — Incremental callable-generic edits

Purpose:
Prove local generic signature edits invalidate calls precisely.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only / dependency fix if failure

Owned files and symbols:
- `phalcom-semantic/tests/semantic/incremental/` appropriate module.

Source of truth:
- canonical callable signature fingerprint/dependency edge.

Required edit scenarios:
- constructor local bound added/removed;
- setter local constraint edited;
- index setter local binder/constraint edited;
- expected-result generic getter remains equivalent if unchanged.

Compare:
```text
result type
CallableId
receiver application
diagnostic code/status
```

Testing classification:
- C10 required.

---

## Task 64 — Incremental variant/GADT edits

Purpose:
Prove introduction/elimination dependencies both invalidate.

Risk:
- Semantic: HIGH
- Implementation fanout: test-only / dependency fix if failure

Owned files and symbols:
- incremental ADT test module.

Required edits:
- add variant local bound;
- change payload `U` use;
- change result from `Expr<U>` to `Expr<List<U>>` or equivalent;
- verify construction call and match branch proof both recompute;
- verify exhaustiveness/reachability where result-index change matters.

Source of truth:
- variant semantic product fingerprint.

Testing classification:
- C10 required.

---

## Task 65 — Alpha-equivalent rigid incremental comparison

Purpose:
Ensure local allocation order is not mistaken for semantic change.

Risk:
- Semantic: HIGH
- Implementation fanout: test/helper

Owned files and symbols:
- C6 alpha-normalization helper;
- incremental fixture comparison helpers.

Source of truth:
- binder introduction structure, not raw `RigidTypeVariableId`.

Changes:
- compare cold vs incremental branch products by canonical/alpha-normalized local structure;
- assert same payload/proof shape even if allocator IDs differ.

Must not:
- simply omit local existential information from equivalence comparison.

Testing classification:
- C10 hostile incremental evidence.

---

## Task 66 — Broad affected semantic/core integration gate

Purpose:
Run broad suites once after all semantic changes, diagnose smallest-first if anything fails.

Risk:
- Semantic: HIGH
- Implementation fanout: verification

Owned files and symbols:
- no planned production edits unless a classified incident identifies a narrow owner.

Verification order:
```bash
cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-core --test core monads:: -- --nocapture
cargo test -p phalcom-core --test core either:: -- --nocapture
```

Each failure enters the failure protocol in §11 before changing code.

Testing classification:
- C10 broad affected-layer gate.

---

## Task 67 — Documentation and state closure

Purpose:
Make the completed semantics authoritative and remove stale guidance.

Risk:
- Semantic: LOW
- Implementation fanout: documentation

Owned files and symbols:
- companion technical spec if committed into repository;
- implementation state file;
- stale `docs/work/deferred/generic-on-getter.md` classification/removal according to repository documentation policy;
- historical specs only where current normative behavior needs an amendment note.

Source of truth:
- implemented semantics + passing evidence.

Changes:
- mark generic getters as implemented/current;
- document multi-domain constructor application;
- document variant local universal/existential split;
- document new per-applied-type class-side semantic requirement as superseding older class-side rule;
- ensure no doc claims runtime per-application storage was implemented if it was not.

Testing classification:
- no standalone test; Final Gate checks docs/build only where repository tooling does so.

---

# 8. Failure / Incident Protocol

If any required evidence fails unexpectedly, set the current checkpoint state to:

```text
C<N> — INCIDENT
```

Do not continue dependent checkpoints.

Before repair, record:

## 8.1 Exact reproduction

```text
command:
...

failing test/check:
...

important output:
...
```

## 8.2 Direct path

Example:

```text
test fixture
→ semantic session
→ callable signature lookup
→ generic application domains
→ inference reconciliation
→ publication assertion
```

## 8.3 Passing comparator

Find the nearest still-working behavior, for example:

```text
explicit Box<Int>.new works, inferred Box.new fails
```

or:

```text
construction works, Family invocation fails
```

or:

```text
cold GADT branch works, incremental branch is stale
```

## 8.4 Classification

Use one:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## 8.5 Narrow repair boundary

State exact allowed files/symbols before editing.

## 8.6 Rejected broad fixes

Do not:

- weaken `GenericSignature::validate_publishable`;
- turn a constraint failure into `Dynamic`;
- add type arguments to selectors;
- persist rigid IDs globally to satisfy a test;
- special-case `Option`, `Result`, `Expr`, `Box`, or another type by name;
- add a GADT-specific constructor inference path;
- disable incremental comparison of meaningful local binder structure;
- implement runtime applied storage to hide missing semantic receiver publication;
- modify parser grammar during a semantic failure unless the failing source was actually parsed incorrectly.

Only after this evidence exists should implementation resume.

---

# 9. Testing Strategy Summary

Tests are scheduled by semantic risk, not task count.

## 9.1 Ownership-layer tests

Use semantic tests for:

```text
generic ownership/inference
receiver application
variant generic identity
GADT proofs
existential escape
incremental products
```

Use AST parser tests for syntax only.

Use native metadata tests for native schema/lowering only.

Use core/runtime tests only where metadata/compiled integration changes cross the semantic boundary.

## 9.2 Smallest-first verification

When diagnosing:

```text
exact regression
    ↓
focused module
    ↓
affected semantic category
    ↓
crate-wide semantic suite
    ↓
workspace delivery gate
```

## 9.3 What key commands prove

```bash
cargo check -p phalcom-semantic
```

Proves:
- shared Rust APIs compile;
- exhaustive match/caller migration is complete at compile level.

Does not prove:
- generic ownership/inference semantics.

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_application -- --nocapture
```

Proves:
- expected-result generic inference and bound/defaulting laws remain correct.

```bash
cargo test -p phalcom-semantic --test semantic semantic::adts::vertical_gadt -- --nocapture
```

Proves:
- focused indexed GADT semantics, including new existential cases added there.

```bash
cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture
```

Proves:
- registered incremental semantic scenarios recompute/publish consistently.

It does not replace hostile tests specifically targeting new rigid alpha-equivalence unless those tests are added to that suite.

---

# 10. Negative / Deletion Gates

These searches are part of migration evidence.

## After C1

```bash
rg 'merge_constructor_generic_signatures' phalcom-semantic
```

Expected:
- zero production hits;
- historical documentation occurrences may remain only if clearly describing superseded behavior.

## After C2

```bash
rg 'CallableSyntaxRef::Setter|CallableSyntaxRef::Index' phalcom-semantic/src/checker/declaration_signature.rs
```

Manual expected result:
- setter/index arms use callable-local generic resolution when AST binders are present;
- no hardcoded `None` generic result remains for those arms except deliberate non-generic conditional logic.

## After C4

```bash
rg 'TypeParameterOwner::Variant' phalcom-semantic phalcom-type-meta phalcom-native-meta
```

Expected:
- zero production hits. Variant locals use `TypeParameterOwner::Callable`.

```bash
rg 'generic variant.*unsupported|variant-local.*unsupported' phalcom-ast phalcom-semantic docs
```

Expected:
- no active/current documentation or production diagnostic claiming the now-supported surface is unsupported;
- historical archived docs may remain with clear status.

## After C6/C8

```bash
rg 'TypeData::Rigid|GadtSkolem' phalcom-semantic/src
```

Expected:
- if the chosen representation did not use these names, zero hits;
- if a scoped rigid representation deliberately uses a similar name, every hit must belong to the canonical scoped/local abstraction, not a competing one-off GADT type.

## Final

```bash
rg 'generic parameters.*getter.*unsupported|generic-on-getter' docs phalcom-ast phalcom-semantic
```

Expected:
- stale deferred document is removed, archived, or clearly marked superseded;
- no production prohibition remains.

```bash
rg 'Type\.currentApplication|currentApplication' docs/impl/semantic phalcom-semantic phalcom-core
```

Expected:
- no new implementation added by this program;
- pre-existing historical/explicit reflection references are reviewed and justified, because per-application storage semantics must not be implemented by resurrecting ambient context.

---

# 11. Cross-Consumer Consistency Evidence

Where canonical facts cross layers, require these equalities/relations.

## 11.1 Callable generic identity

```text
source declaration CallableId
== semantic callable signature CallableId
== TypeParameterOwner::Callable owner
== metadata stable callable owner
```

## 11.2 Generic variant identity

```text
VariantId
→ VariantConstructorId
→ canonical constructor generic CallableId owner
→ GenericSignature owner
```

The generic constructor callable identity and executable `VariantConstructorId` correspond to the same exact variant but remain distinct semantic ID domains.

## 11.3 Applied class-side invocation

```text
semantic receiver application Box<Int>
== invocation specialization receiver_application
→ lowering/future runtime applied storage owner input
```

Do not require:

```text
Box<Int> CallableId != Box<String> CallableId
```

They must remain equal at declaration identity level.

## 11.4 GADT local existential

```text
one VariantConstructorSignature local binder U
→ one CaseInstantiation rigid κ
→ every payload occurrence of U uses same κ
→ result-index proof uses same κ
```

No durable metadata consumer receives `κ`.

---

# 12. Planned Commit Groups

Commits should align with semantic ownership, not task count.

Recommended sequence:

```text
C0   test(semantic): characterize constructor generic publication

C1.1 refactor(semantic): separate constructor generic application domains
C1.2 fix(semantic): preserve owner-specific generic solutions
C1.3 test(semantic): certify constructor ownership and metadata

C2.1 feat(ast): add generic setter and index member binders
C2.2 feat(semantic): apply canonical generics to setters and indexers
C2.3 test(semantic): certify accessor/index generic laws

C3.1 feat(semantic): parameterize generic class-side templates
C3.2 feat(semantic): retain applied receiver invocation specialization
C3.3 test(semantic): certify applied class-side semantics

C4.1 feat(ast): add variant-local generic binders
C4.2 feat(semantic): publish variant constructor generic signatures
C4.3 test(adts): certify variant-local declaration laws

C5.1 feat(adts): infer generic variant construction via canonical solver
C5.2 feat(families): retain generic variant targets
C5.3 test(adts): certify generic variant introduction

C6.1 feat(types): add scoped rigid type variables
C6.2 feat(semantic): enforce rigid relation/publication laws
C6.3 test(types): certify rigid kernel

C7.1 feat(gadt): open constructor locals as existentials
C7.2 feat(gadt): integrate rigid result-index proofs
C7.3 test(gadt): certify shared/fresh existential identity

C8.1 feat(gadt): enforce existential non-escape
C8.2 feat(adts): reconstruct exact-case hidden locals
C8.3 test(gadt): certify escape/exact-case semantics

C9.1 feat(native-meta): describe canonical generic contracts
C9.2 feat(semantic): import native callable generics
C9.3 fix(core): initialize generated generic fields
C9.4 test(metadata): certify source/native parity

C10.1 feat(incremental): fingerprint completed generic/GADT contracts
C10.2 test(incremental): certify cold/reanalysis parity
C10.3 docs(semantic): publish completion state
```

These are suggested groups, not a requirement to create a commit after every line item.

---

# 13. Final Broad Gates

Run only after C10 is COMPLETE and no incident remains.

## 13.1 Formatting

```bash
cargo +stable fmt --all -- --check
```

Proves:
- repository Rust formatting is clean.

Does not prove:
- semantics.

## 13.2 Workspace compile

```bash
cargo +stable check --workspace --all-targets
```

Proves:
- cross-crate API migrations compile for all workspace targets.

Particularly important for:
- AST field fanout;
- native metadata schema changes;
- semantic snapshot/invocation product changes.

## 13.3 Workspace tests

```bash
cargo +stable test --workspace --all-targets
```

Proves:
- broad compatibility and delivery readiness across repository tests.

It does not replace the focused checkpoint evidence that established semantic laws.

## 13.4 Clippy

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Proves:
- no new compiler/lint warnings under repository stable toolchain policy.

## 13.5 Project-specific protected suites

If not already encompassed/reliable under the workspace command, explicitly retain the C10 protected commands:

```bash
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-core --test core monads:: -- --nocapture
cargo test -p phalcom-core --test core either:: -- --nocapture
```

Do not rerun them after the workspace gate if the exact same binaries/tests were already executed successfully and no code changed afterward; record evidence reuse instead.

---

# 14. Final Negative / Deletion Gates

Run after Final Broad Gates, with no code changes afterward unless a gate fails.

```bash
rg 'merge_constructor_generic_signatures' phalcom-semantic
```

Expected:
- zero production hits.

```bash
rg 'TypeParameterOwner::Variant' phalcom-semantic phalcom-type-meta phalcom-native-meta
```

Expected:
- zero production hits.

```bash
rg 'generic parameters.*getter.*unsupported|generic getter.*unsupported' phalcom-ast phalcom-semantic docs
```

Expected:
- zero production/current-spec prohibitions;
- archived historical references explicitly marked superseded are acceptable.

```bash
rg 'constructor.*GenericSignature.*merge|merged.*constructor.*generic' phalcom-semantic/src docs/impl/semantic
```

Expected:
- no current documentation/production comments describe mixed-owner signature as canonical;
- historical evidence may remain only in archived implementation history.

```bash
rg 'Type\.currentApplication|currentApplication' phalcom-semantic phalcom-core docs/impl/semantic
```

Expected:
- this implementation program did not add ambient applied generic runtime context;
- every remaining hit is reviewed and justified as historical/reflection documentation or unrelated existing API.

```bash
rg 'VariantConstructorSignature' phalcom-semantic/src
```

Manual gate:
- all construction/elimination paths consume the canonical generic signature product rather than reparsing variant syntax.

```bash
rg 'RigidTypeVariableId|RigidScopeId' phalcom-semantic/src
```

Manual gate:
- all hits belong to the single canonical scoped-rigid abstraction;
- no second GADT-only skolem registry exists.

---

# 15. Deferred-Evidence Audit

Before declaring release-complete, every item recorded under `Deferred gates` must be one of:

1. executed successfully;
2. explicitly removed from scope with written justification and supervisor approval;
3. recorded as a known release blocker, in which case the program is **not** release-complete.

There must be no unowned deferred command.

---

# 16. Known Scope Exclusions

Do not silently add these during implementation:

```text
general forall syntax
rank-2/rank-N polymorphism
impredicative polymorphism
first-class existential package syntax/runtime values
runtime typeclass/protocol witness passing
dependent types
monomorphization
specialized machine-code generation
runtime per-applied-type class storage tables
runtime applied metaclass object implementation
new collection/array representations
LSP-specific type semantics
new reflection API design beyond metadata parity required by existing products
performance optimization not required for correctness
```

The static program **does** retain `receiver_application` specifically so future runtime applied-class storage can be implemented without changing semantic identity again.

---

# 17. State-File Completion Requirements

At final completion, the implementation state file must contain:

## Established invariants

At minimum:

```text
I-001 canonical GenericSignature ownership is homogeneous
I-002 constructor application composes declaration + callable domains without merging signatures
I-003 setter/index/variant locals use TypeParameterOwner::Callable
I-004 generic instantiation preserves selectors/CallableIds/VariantConstructorIds
I-005 class-side generic declarations are templates specialized by applied receivers
I-006 invocation specialization retains canonical applied receiver
I-007 variant locals are universal at construction and existential at elimination
I-008 one local binder maps to one rigid per case instantiation
I-009 branch-local rigids cannot escape
I-010 ExactCase remains variant + enum_type and reconstructs hidden locals freshly
I-011 native/generated inputs converge on canonical signatures
I-012 cold/incremental products are alpha-equivalent across rigid allocation
```

## Decisions

Record final choices for:

```text
setter generic grammar
index generic grammar
variant constructor callable identity helper
rigid representation seam
closure capture escape rule
invocation specialization product location
index setter return semantics if changed/ratified
```

## Evidence ledger

Every checkpoint command/result/claim.

## Negative gates

Every migration search and result.

## Deferred gates

Must be empty at release-complete.

## Active incident

Must be:

```text
None.
```

---

# 18. Checkpoint Completion Report Template

After each checkpoint, report:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant semantic contract>

Changed:
    <path> — <symbol/responsibility>
    ...

Evidence:
    <command> — PASS — proves <invariant>

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — <expected result>

Deferred:
    <command> → C<M>/Final Gate

Unexpected findings:
    none | <bounded fact>

Next:
    C<N+1> — <name>
```

If required evidence fails, report `INCIDENT`, not `COMPLETE`.

---

# 19. Checkpoint Evidence Summary — Planned

This table is a plan, not a claim of executed status.

| Checkpoint | Semantic contract | Planned evidence | Status at plan creation |
|---|---|---|---|
| C0 | baseline/drift pinned | getter/parser/generic/metadata characterization | NOT RUN |
| C1 | canonical multi-domain constructor application | constructor inference + metadata + receiver/generic baselines | NOT RUN |
| C2 | generic setter/index callable surfaces | parser + setter/index semantics + selector hostility | NOT RUN |
| C3 | applied class-side static specialization | applied member/field/receiver publication tests | NOT RUN |
| C4 | variant-local generic declaration products | parser + owner + metadata/fingerprint tests | NOT RUN |
| C5 | generic variant universal construction + Families | constructor/generic/associated/Family tests | NOT RUN |
| C6 | scoped rigid kernel | relation/walker/publication/alpha unit tests | NOT RUN |
| C7 | full GADT existential elimination | vertical GADT + matching + shared/fresh hostile tests | NOT RUN |
| C8 | non-escape + exact-case reconstruction | existential/flow/closure/exact-case tests | NOT RUN |
| C9 | native/generated/metadata parity | native metadata + source/native parity + generated accessor tests | NOT RUN |
| C10 | incremental and semantic certification | incremental + ADT + Family + full semantic/core protected suites | NOT RUN |
| Final Gate | delivery readiness | fmt/check/workspace test/clippy + negative gates | NOT RUN |

No checkpoint should be marked complete until its required evidence is actually executed against the implementing checkout.

---

# 20. Release-Complete Criteria

The implementation program is complete only when:

- [ ] C0 through C10 are all `COMPLETE`;
- [ ] every checkpoint's semantic evidence passes;
- [ ] every high-risk hostile case passes;
- [ ] `merge_constructor_generic_signatures` is removed from production code;
- [ ] no mixed-owner canonical `GenericSignature` remains;
- [ ] generic setter/index/variant local binders use canonical callable ownership;
- [ ] generic variant construction uses ordinary call inference;
- [ ] scoped rigid variables are a single reusable abstraction;
- [ ] full GADT elimination uses fresh/shared rigids correctly;
- [ ] existential escape is rejected at every identified publication boundary;
- [ ] exact-case hidden locals are reconstructed freshly without changing canonical exact-case identity;
- [ ] applied class-side templates specialize under canonical applied receivers;
- [ ] invocation publication retains applied receiver identity for future per-application storage;
- [ ] native/generated/intrinsic semantic inputs converge on canonical products where applicable;
- [ ] cold/incremental parity passes with alpha-equivalent rigid scopes;
- [ ] all obsolete/stale mechanisms pass negative/deletion gates;
- [ ] every deferred evidence item is resolved;
- [ ] final format/check/workspace test/clippy gates pass;
- [ ] implementation state contains no unresolved incident;
- [ ] documentation is updated to distinguish implemented static semantics from deferred runtime applied-storage work.

---

# 21. Final Handoff Guidance

An implementing agent should not redo the full repository investigation before every task.

At each checkpoint:

1. perform the bounded drift check;
2. inspect only the checkpoint's primary symbols and the explicit inspect-before-edit consumers;
3. implement all preparatory tasks before running checkpoint evidence unless a strategic `cargo check` is listed;
4. run required evidence smallest-first;
5. classify any failure before expanding scope;
6. record checkpoint facts/evidence;
7. continue only from `COMPLETE` checkpoints.

The semantic design is not negotiable during mechanical adaptation:

```text
canonical identity over names
domain composition over signature corruption
one solver over syntax-specific solvers
scoped rigids over flexible skolems
publication barriers over erasure
applied receiver specialization over selector specialization
```

If current repository evidence contradicts one of those design assumptions, stop and escalate with the exact code evidence rather than silently choosing an easier local patch.
