---
id: LANG006.C1.P2
category: LANG
program: LANG006
checkpoint: LANG006.C1
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
depends_on:
  - LANG006.C1.P1
follows: LANG006.C1.P1
supersedes: null
repository: aureat/phalcom-lang
branch: main
planning_revision: 9f1d2c9fa98e94edc6088aee1834520361e08d3e
prepared: 2026-09-18
---

# LANG006.C1.P2 — Corrective Access Provenance and Implementation-Kind Closure

## 0. Executor contract

This is the corrective closure plan for `LANG006.C1`. It exists because the post-P1 audit found that the source-syntax migration landed successfully, but the access/provenance/implementation-kind vertical was not actually complete.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture and invariants below;
- must not reopen or redesign the successful P1 syntax/Universe migration;
- must not reintroduce selector/field-prefix semantics;
- must not invent a foreign ABI, loader, ownership model, or FFI representation;
- must not use `is_universe` or compiler-internal authority as a substitute for ordinary access provenance;
- must not duplicate package ownership or type/signature normalization in a consumer;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must update the shared `LANG006.C1-CHECKPOINT.md` rather than creating a competing state record;
- must create the required P2 walkthrough and handoff.

Testing is evidence gathering, not a ritual. Do not run broad suites merely because a mechanical edit occurred.

---

## 1. Goal

Close the audited `LANG006.C1` gaps by making declaration access provenance exact and canonical from modules through semantic analysis and runtime dispatch, converging the language `ImplementationKind` axis, completing the bounded `foreign` scaffold, enforcing exact native source/descriptor conformance, and certifying the missing access/tooling/specification evidence.

---

## 2. Checkpoint acceptance objective

This plan is **corrective checkpoint closure**.

`LANG006.C1` is accepted only when all of the following are simultaneously true:

1. semantic member resolution distinguishes member existence from member accessibility and applies the canonical access relation to the selected callable;
2. `Module`, `Package`, and `Internal` authorization use exact module/package/project identities, with no path/string reconstruction;
3. every ordinary source-authored or source-derived runtime method that participates in user-visible dispatch carries the declaration origin needed for runtime authorization;
4. native-installed methods inherit exact declaration origin rather than blanket Universe/project authority;
5. native source/descriptor verification compares the complete canonical callable contract, not merely annotation presence;
6. the language implementation category is one canonical `Phalcom | Native | Foreign` axis, while native/generated/abstract/external execution provenance is a separately named fact;
7. legal `foreign` declarations survive semantic products and fail explicitly at unsupported binding/lowering rather than degrading into “missing body” or native behavior;
8. LSP completion/hover/reflection consume the same canonical facts;
9. the authoritative declaration specification agrees with the accepted C1 modifier-order contract;
10. the previously deferred C1 access/cache/ownership evidence is replaced by concrete tests;
11. the checkpoint record no longer claims completion without evidence.

P1 remains a historical predecessor. Do not rewrite P1 as if its implementation never happened. P2 records and fixes the corrective delta.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 9f1d2c9fa98e94edc6088aee1834520361e08d3e
commit: feat(lang006): complete declaration modifier migration
checkpoint: docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
predecessor: LANG006.C1.P1
```

Verified planning facts at this revision:

- `phalcom-common/src/modifier.rs` owns the source-language `Visibility` and `ImplementationKind`.
- the lexer recognizes `native`, `foreign`, `private`, `protected`, `module`, `package`, and `internal`; `public` is not a keyword;
- `phalcom-semantic/src/editor.rs` contains `is_member_accessible`, but its `Package` case has a path-based fallback;
- `EditorSemanticQuery::access_context_at` currently passes no caller-package identity;
- canonical formal checker dispatch in `CheckingContext::resolve_dispatch_target_with_specialization` resolves callables without applying `is_member_accessible`;
- `MethodObject::new` initializes `origin` to `RuntimeAccessOrigin::none()`;
- source method creation paths set visibility/holder/access-owner but do not consistently stamp declaration origin;
- `phalcom-core/src/native/install.rs` sets native methods to `RuntimeAccessOrigin::universe()`, which carries project authority but not exact declaration module/package authority;
- `NativeMemberAnchor` does not currently carry the source module;
- `phalcom-native-meta/src/primitive.rs` still declares a second `ImplementationKind { Source, NativePrimitive, Generated, Abstract, External }`;
- semantic signatures and reflection/presentation consume that second axis;
- legal `Foreign` survives the AST but does not have the required semantic/lowering scaffold and does not emit the required explicit unsupported-binding diagnostic;
- current native verification checks visibility and annotation presence but does not prove exact canonical source/descriptor parameter and return-type equality;
- LSP semantic-token handling includes the seven keywords, while keyword hover does not enumerate them;
- `docs/specs/source-declarations.md` says modifier order is otherwise insignificant, while the accepted C1 implementation contract requires visibility before implementation and rejects noncanonical permutations;
- the P1 walkthrough recorded missing/deferred standalone evidence for module/package/cross-project/cache/standalone-ownership behavior.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

The implementer must begin by running:

```sh
git status --short
git rev-parse HEAD
```

Do not reset, clean, restore, or overwrite unrelated work.

---

## 4. Required reads before implementation

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P1-HEADER.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P1-BODY.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P1-walkthrough.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P1-handoff.md

docs/specs/README.md
docs/specs/source-declarations.md
docs/specs/universe.md
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md

phalcom-common/src/modifier.rs
phalcom-modules/src/query.rs
phalcom-modules/src/topology.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/core_surface/conformance.rs
phalcom-core/src/method/object.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/native/source.rs
phalcom-core/src/native/verify.rs
phalcom-core/src/native/install.rs
phalcom-native-meta/src/primitive.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/semantic_tokens.rs

phalcom-semantic/tests/semantic/integration/access_control.rs
phalcom-core/tests/core/compiler/declaration_dispatch.rs
phalcom-core/tests/core/execution/compiler_authority.rs
relevant module/package topology tests
relevant native conformance tests
relevant LSP hover/completion tests
```

Avoid broad exploration unless evidence from these paths makes it necessary.

---

## 5. Normative authority

### 5.1 Normative behavior

The following are fixed:

1. `docs/specs/README.md` owns the current specification precedence model.
2. `docs/specs/source-declarations.md` owns declaration modifiers, access domains, implementation origin, and legacy namespace retirement.
3. TDR-0082 fixes:
   - seven declaration keywords;
   - no `public` keyword;
   - names do not imply visibility or implementation kind;
   - `Private` = exact owner;
   - `Protected` = owner/subclass;
   - `Module` = exact module;
   - `Package` = exact package;
   - `Internal` = exact project;
   - direct/cached/dynamic/reflective/native calls use the same relation;
   - native execution inherits declaration authority;
   - `Foreign` remains distinct and bounded;
   - `is_universe` is independent bootstrap provenance.
4. The accepted C1 parser contract requires canonical authored modifier order:
   - visibility modifier first;
   - implementation modifier second;
   - noncanonical permutations such as `native internal foo` are rejected.
5. `@primitive` / `@intrinsic` remain metadata-only and Universe-only.
6. authored `_$name` and general implementation `__name` semantics remain retired; standardized dunders remain intact.

### 5.2 Current implementation

Current code is evidence, not authority where it conflicts with the rules above.

Known defects are implementation defects, not new language rules:

- package path fallback;
- absent source-method runtime origin;
- project-only native origin;
- duplicate language/provenance `ImplementationKind` axes;
- generic missing-body treatment of legal `Foreign`;
- incomplete source/native signature comparison;
- LSP hover omissions.

### 5.3 Future / non-normative work excluded here

This plan does not decide:

- a foreign C ABI;
- a Rust ABI;
- dynamic library discovery/loading;
- FFI memory ownership;
- pointer/value marshaling;
- foreign object representation;
- public reflection API expansion beyond restoring the already-specified implementation category;
- a new package model;
- a new method lookup model.

---

## 6. Takeover state

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| source visibility | `phalcom_common::modifier::Visibility` | `phalcom-common` | six exact cases, `Public` default |
| source implementation category | `phalcom_common::modifier::ImplementationKind` | `phalcom-common` | `Phalcom | Native | Foreign` |
| package ownership | `ModuleQueryFacade::owning_package` / module topology | `phalcom-modules` | identity, never path approximation |
| semantic access context | `AccessContext` | `phalcom-semantic` | actual caller provenance |
| semantic member access helper | `is_member_accessible` | `phalcom-semantic` | one relation shared by checker/editor |
| canonical dispatch | `DispatchTable::resolve_dispatch_with_trace` + `CheckingContext::resolve_dispatch_target_with_specialization` | `phalcom-semantic` | lookup identity separate from access authorization |
| runtime method metadata | `MethodObject` | `phalcom-core` | visibility + owner + origin |
| runtime caller origin | `VM::current_access_origin` / `access_origin_for_module` | `phalcom-core` | exact runtime projection of module/package/project |
| native source index | `NativeSourceIndex`, `NativeMemberAnchor` | `phalcom-core/native` | source declaration is binding authority |
| native descriptor metadata | `phalcom-native-*` | native infrastructure | descriptor must agree with source |
| compiler-only bypass | `compiler_internal_dispatch_depth` or live successor | compiler/VM | synthetic seam only, never ordinary source authorization |
| Universe provenance | `ProjectIdentity::Universe`, `is_universe` | modules/bootstrap | not ordinary `Internal` privilege |
| keyword tokenization | lexer + LSP semantic tokens | AST/LSP | seven keywords, no `public` |
| authoritative source-declaration spec | `docs/specs/source-declarations.md` | specification corpus | must match accepted C1 behavior |

---

## 7. Architecture

### 7.1 Direction of authority

```text
phalcom-modules
    owns canonical ModuleId -> owning Package ModuleId + ProjectIdentity
        ↓
phalcom-semantic
    publishes declaration access origin + caller AccessContext
    resolves member identity
    authorizes the selected member with one access relation
        ↓
compiler/lowering
    projects already-known declaration/module identity
        ↓
phalcom-core runtime
    stores compact RuntimeAccessOrigin on executable MethodObject
    authorizes every invocation path against the same relation
```

No layer may reconstruct package identity from filesystem/module-path shape once canonical ownership exists.

### 7.2 Canonical semantic access product

Introduce or normalize one semantic product equivalent to:

```rust
struct DeclarationAccessOrigin {
    module: ModuleId,
    package: ModuleId,
    project: ProjectIdentity,
}
```

The exact symbol name is mechanically flexible.

Rules:

- `module` is the exact declaring module;
- `package` comes from canonical module topology/query;
- `project` is the declaring module's project identity;
- no field is optional for an ordinary module-backed declaration once module topology is established;
- synthetic declarations without source/module ownership are explicitly classified rather than assigned guessed provenance.

`AccessContext` must carry the caller-side equivalent, including canonical `caller_package`.

### 7.3 Lookup first, authorize the selected target second

Visibility is access control, not method-lookup fallback.

```text
resolve canonical callable according to normal dispatch
    ↓
recover selected callable visibility + declaration origin
    ↓
apply canonical access relation using actual caller context
    ↓
allowed: continue type/call analysis
denied: emit/return a distinct access failure
```

Do not skip an inaccessible subclass method and silently fall through to a same-selector superclass method. Runtime lookup does not do that; semantic analysis must match it.

A denied member must not be reported as “member missing.” Representation is flexible:

- explicit `ResolvedDispatchResult::Inaccessible`, or
- a checker-level access-failure result retaining the resolved callable.

The distinction between existence and accessibility is fixed.

### 7.4 Runtime origin projection

Every source-authored or source-derived executable method reachable through ordinary user dispatch must receive exact runtime origin before invocation.

Preferred compiler shape:

```rust
method.origin = vm.access_origin_for_module(current_module_ref);
```

or an equivalent centralized helper.

Audit every production `MethodObject::new` / `new_single` producer and classify:

```text
A — source-authored method/accessor/impl member
B — source-derived/generated executable behavior owned by a source module
C — native-installed declaration implementation
D — VM/bootstrap/compiler synthetic method with no ordinary source declaration
E — test-only construction
```

Requirements:

- A/B: carry source module/package/project origin;
- C: handled by native declaration-origin projection;
- D: remain explicitly synthetic; do not fake a user declaration origin;
- E: update only as needed for tests.

Holder/access-owner and module/package/project origin remain distinct facts.

### 7.5 Native declaration origin

`NativeMemberAnchor` must retain enough source identity to identify its exact declaring module.

Preferred minimal change:

```rust
pub struct NativeMemberAnchor {
    ...
    pub module: ModuleId,
}
```

Installation:

```text
native descriptor key
    ↔ verified source NativeMemberAnchor
        ↓ exact source ModuleId
resolve/materialize corresponding runtime module handle
        ↓
VM::access_origin_for_module(module_handle)
        ↓
installed MethodObject.origin
```

Do not use `RuntimeAccessOrigin::universe()` as final per-method declaration origin.

### 7.6 Language implementation category vs execution provenance

Language category:

```rust
phalcom_common::ImplementationKind {
    Phalcom,
    Native,
    Foreign,
}
```

Execution/provenance may continue representing:

```text
Source
NativePrimitive
Generated
Abstract
External
```

but it must be renamed to a provenance-oriented type such as `ImplementationProvenance`. Exact name is mechanically flexible; retaining the conflicting name is forbidden.

Rules:

- `CallableSemanticSignature.implementation` means the language category and uses the shared common enum;
- `Generated` and `Abstract` are not substitute language implementation categories;
- provenance is stored separately only where consumers genuinely need it;
- reflection/tooling property named `implementationKind` presents only `phalcom | native | foreign`;
- do not invent a new public reflection property for provenance unless an existing normative spec requires it.

### 7.7 Bounded foreign scaffold

A legal bodyless foreign callable retains:

```text
ImplementationKind::Foreign
```

through:

```text
AST
→ declaration/signature/member semantic products
→ source index/reflection/tooling
→ compiler/lowering boundary
```

With no foreign backend, compilation/linking fails explicitly:

```text
foreign.binding_unimplemented
```

or the repository's exact approved diagnostic spelling.

Forbidden outcomes:

- generic “missing body”;
- treating `Foreign` as `Native`;
- installing a nil/stub body;
- runtime method-missing as backend;
- selecting an ABI.

### 7.8 Exact native source/descriptor conformance

Complete contract:

```text
owner identity
dispatch side
selector
member kind
parameter/selector shape
canonical parameter types
canonical return type
visibility
ImplementationKind::Native
```

Do not implement a second type normalizer in `phalcom-core`.

Preferred architecture:

```text
source declaration
    ↓ existing canonical semantic signature formation
CallableSemanticSignature
    ↔
native descriptor imported into canonical semantic signature
    ↓
one canonical conformance comparison
```

`phalcom-core/src/native/verify.rs` may keep bootstrap presence/visibility/source-anchor checks, but canonical type equality must reuse semantic type/signature products.

If exact bootstrap timing makes canonical semantic signatures unavailable without material lifecycle redesign, STOP AND CONSULT. Do not fall back to string comparison or duplicate AST type lowering.

### 7.9 Tooling and specification convergence

- semantic tokens keep all seven keywords;
- hover keyword tables include all seven;
- completion/member filtering uses canonical caller/declaration package identity;
- reflection exposes canonical language implementation kind;
- `docs/specs/source-declarations.md` states:
  - visibility precedes implementation;
  - normalized facts are orthogonal, but source permutations are not all accepted.

Do not modify historical decision records merely to erase old wording.

---

## 8. Ownership boundaries

### Owns

P2 owns:

- exact declaration/caller access provenance transport;
- semantic access enforcement for ordinary member dispatch;
- runtime source-method origin stamping;
- native declaration-derived runtime origin;
- exact native signature conformance integration;
- implementation-category/provenance separation;
- bounded `Foreign` semantic/lowering completion;
- C1 LSP/reflection/spec parity;
- missing C1 access/cache/ownership certification evidence;
- checkpoint correction/reclosure.

### Does not own

P2 does not own:

- new module/package semantics;
- FFI design;
- general semantic dispatch redesign unrelated to accessibility;
- trait coherence redesign;
- new reflection APIs;
- Universe API redesign;
- method cache redesign unless a verified access-control defect requires a local cache correction;
- broad repository cleanup;
- unrelated incremental checker or iterator/fold failures.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| declaration visibility | common AST/semantic modifier fact | semantic/compiler/runtime/LSP | prefix/attribute inference |
| language implementation category | `phalcom_common::ImplementationKind` | semantic/reflection/compiler/LSP | native-meta duplicate enum |
| module identity | `phalcom-modules` | semantic/compiler/runtime | source-path comparison |
| package ownership | `phalcom-modules` topology/query | semantic/compiler/runtime/LSP | parent-path inference |
| project identity | `phalcom-modules` | semantic/runtime | Universe boolean as authorization |
| selected callable | semantic dispatch | checker/lowering | editor-side re-resolution |
| accessibility | one semantic access relation | checker/editor; runtime equivalent projection | per-consumer policy |
| runtime declaration origin | projection from runtime module object | VM dispatch | hard-coded Universe origin |
| native callable type contract | canonical semantic signature | conformance/bootstrap verification | AST-string comparison |
| execution provenance | renamed provenance type | internal consumers only where needed | overloading language `ImplementationKind` |

---

## 9. Global invariants

```text
INV-01 — Names never imply visibility, implementation kind, or runtime privilege.
INV-02 — Visibility has exactly Public/Private/Protected/Module/Package/Internal semantics.
INV-03 — ImplementationKind has exactly Phalcom/Native/Foreign language semantics.
INV-04 — Module authorization is exact declaring-module identity.
INV-05 — Package authorization is exact owning-package identity.
INV-06 — Internal authorization is exact project identity.
INV-07 — Package identity is never reconstructed from module-path parent relationships in a correctness consumer.
INV-08 — Semantic member existence and member accessibility remain distinct facts.
INV-09 — Semantic lookup never falls through an inaccessible selected method to a superclass method.
INV-10 — Checker and editor consume the same semantic access relation.
INV-11 — Runtime direct/cached/dynamic/bound/reflective invocation uses the same effective access relation.
INV-12 — Source-authored/source-derived runtime methods carry exact declaration module/package/project origin.
INV-13 — Native methods execute with the authority of the exact Phalcom declaration they implement.
INV-14 — Compiler-generated synthetic privilege remains a narrow compiler seam and never repairs ordinary source authorization.
INV-15 — is_universe remains bootstrap provenance, not ordinary Internal access.
INV-16 — Native source/descriptor equality includes canonical signature/type equality, visibility, and implementation category.
INV-17 — Canonical type/signature normalization is not duplicated in phalcom-core.
INV-18 — Execution provenance (Generated, Abstract, etc.) is not the language implementation category.
INV-19 — A legal Foreign declaration remains Foreign through semantic/reflection/lowering products.
INV-20 — An unbound Foreign declaration fails explicitly before runtime and does not select an ABI.
INV-21 — @primitive / @intrinsic remain metadata-only and Universe-only.
INV-22 — Authored _$ and general implementation-field semantics remain retired; standardized dunders remain intact.
INV-23 — Source syntax accepts only canonical C1 modifier order: visibility before implementation.
INV-24 — No test is made green by weakening access checks, widening caller privilege, or substituting path heuristics.
INV-25 — Existing private/protected, trait, incremental, and Universe behavior outside the corrective delta remains unchanged.
INV-26 — Visibility must not be used as a semantic total order; deterministic sorting uses an explicit non-semantic key.
```

---

## 10. Non-goals

Do not:

- redesign `Visibility`;
- add `public` syntax;
- revive retired modifier attributes;
- revive `_$`/general `__field` implementation namespaces;
- change standardized dunder protocols;
- redesign module topology;
- invent foreign binding/loading/ABI/ownership;
- redesign trait witness selection;
- make compiler-internal bypass a general access token;
- make Universe natives blanket-privileged;
- introduce an LSP-owned package cache;
- introduce a second semantic signature/type normalizer;
- broaden reflection surface without normative authority;
- perform unrelated formatter cleanup;
- fix known unrelated semantic incremental or VM iteration/fold failures unless evidence proves P2 caused them.

---

## 11. Expected impact map

### Expected source areas

- `phalcom-modules/src/query.rs`, `topology.rs`, or existing snapshot-facing module product — canonical package ownership.
- `phalcom-semantic/src/editor.rs` — remove package fallback and populate caller package.
- `phalcom-semantic/src/dispatch.rs` — preserve selected visibility/access metadata.
- `phalcom-semantic/src/checker/context.rs` and nearby checker call sites — enforce access.
- `phalcom-semantic/src/signature.rs` and construction paths — canonical language `ImplementationKind`.
- `phalcom-semantic/src/core_surface/{source,merge,presentation,conformance}.rs` — implementation-kind separation, Foreign retention, native conformance.
- `phalcom-native-meta/src/primitive.rs` and direct consumers — rename/split provenance enum.
- `phalcom-core/src/method/object.rs` — source-method origin helper if useful.
- `phalcom-core/src/compiler/lib/{class_decl,impl_decl}.rs` and other source method constructors — origin stamping / Foreign lowering.
- `phalcom-core/src/vm/{send,dispatch,mod}.rs` — runtime parity only where necessary.
- `phalcom-core/src/native/{source,verify,install}.rs` — source module provenance and native origin/conformance.
- `phalcom-lsp/src/{hover,completion,backend,semantic_tokens}.rs` — keyword/package parity.
- `docs/specs/source-declarations.md` — modifier-order correction.
- `docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md` — reopen/reclose.

### Expected tests

- semantic access-control integration;
- semantic ordinary call/checker;
- module topology/package ownership;
- core compiler/runtime declaration dispatch;
- dynamic/bound/cache/reflection parity;
- native source/descriptor verification;
- native install origin;
- Foreign semantic/lowering;
- LSP hover/completion;
- existing private/protected/Universe regressions.

### Expected documentation/state

Create:

```text
docs/implementation/LANG006/LANG006.C1/
  LANG006.C1.P2-corrective-access-provenance-and-implementation-kind-closure.md
  LANG006.C1.P2-walkthrough.md
  LANG006.C1.P2-handoff.md
```

Update:

```text
LANG006.C1-CHECKPOINT.md
docs/specs/source-declarations.md
```

### Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when that entry changes architecture, identity ownership, semantic authority, bootstrap lifecycle, or the Foreign boundary.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- exact Module/Package/Project identity semantics;
- lookup-then-authorize behavior;
- non-fallthrough on inaccessible selected methods;
- package ownership authority in `phalcom-modules`;
- common `ImplementationKind` as language category;
- separation of language implementation category from execution provenance;
- source/native signature comparison through canonical semantic products;
- exact source declaration origin for native methods;
- Foreign remaining unbound and ABI-free;
- explicit pre-runtime unsupported-Foreign failure;
- metadata-only `@primitive` / `@intrinsic`;
- independent Universe provenance;
- canonical modifier order;
- compiler-synthetic authority remaining separate.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact helper/type names such as `DeclarationAccessOrigin`;
- whether access helpers live in `editor.rs` or a new semantic `access.rs`;
- private decomposition of checker authorization;
- whether resolved visibility is stored on `ResolvedDispatch` or obtained through canonical member records;
- exact internal provenance-type name, provided it is not `ImplementationKind`;
- equivalent existing module-query APIs;
- exact test module/file placement;
- mechanical Rust signature/import/borrow changes;
- test fixture mechanics;
- diagnostic enum placement, provided Foreign failure remains explicit/stable.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| semantic snapshot can query canonical owning package | snapshot/module query integration | add narrow canonical projection; do not path-infer |
| `ResolvedDispatch` can recover selected visibility without global redesign | dispatch/member surfaces | add minimal selected-member metadata |
| checker can derive caller declaration/module | `CheckingContext` | add scoped access context; do not query LSP/editor state |
| source method creation is concentrated in identified constructors | `rg 'MethodObject::(new|new_single)' phalcom-core/src` | classify additional producers under T3 |
| native source anchor can carry `ModuleId` without cycle | `phalcom-core/src/native/source.rs` | STOP AND CONSULT if identity/dependency redesign appears |
| native install can resolve source module runtime handle | module registry/materialization API | adapt locally if direct lookup exists; consult if bootstrap order changes |
| canonical semantic source/native signatures are available before installation | semantic/bootstrap conformance path | STOP AND CONSULT if lifecycle redesign is required |
| native-meta provenance enum can be renamed without material compatibility issue | dependents/API tests | deprecated alias only if it cannot remain semantic authority |
| Foreign can fail at compiler/link boundary without ABI choice | compiler declaration lowering | if backend choice becomes necessary, STOP AND CONSULT |
| Visibility ordering derives are not semantically required | ordering/sort consumers | remove or replace with explicit non-semantic sort key |

---

## 13. Global STOP / CONSULT triggers

Stop editing and build a consultation packet if:

1. exact package identity cannot be obtained from existing module topology without redefining package ownership;
2. formal checker access enforcement requires changing method lookup semantics rather than authorizing the selected method;
3. an inaccessible selected method would need to fall through to a superclass to satisfy existing tests;
4. exact source-method origin cannot be attached without changing runtime method identity/lifetime representation;
5. native declaration origin requires material bootstrap-order redesign;
6. exact source/native canonical signature comparison cannot be performed without a second type normalizer or major phase movement;
7. the old native-meta `ImplementationKind` is externally versioned so the semantic split is materially incompatible;
8. Generated/Abstract/External semantics cannot be separated from language implementation category without changing accepted behavior;
9. Foreign implementation pressure requires choosing ABI, loader, ownership, marshaling, or foreign object representation;
10. a cache needs new access-result-cache architecture instead of invocation-time authorization;
11. trait/coherence semantics must change to repair ordinary visibility;
12. source spec, TDR-0082, and accepted C1 modifier-order contract cannot be reconciled by the documented correction;
13. the same nontrivial semantic failure persists after one serious correction;
14. tests pass only by weakening an invariant;
15. patch scope becomes materially more cross-cutting than planned.

A triggered consultation cannot be self-waived.

Consultation packet:

```text
Observed
Expected invariant
Minimal call/data flow
Relevant source excerpts
Failing discriminating test
Attempted correction
Why the planned owner/interface is insufficient
Decision question
```

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

### Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure. If it persists: STOP AND CONSULT.

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| P2-CV-01 | canonical semantic success | 04,08,10 | same-module `module` call accepted |
| P2-CV-02 | semantic negative | 04,08 | cross-module `module` call is inaccessible, not missing |
| P2-CV-03 | package identity | 05,07 | sibling modules in same package may access |
| P2-CV-04 | nested-package boundary | 05,07 | nested/different package denied despite path prefix |
| P2-CV-05 | project identity | 06 | same-project `internal` accepted |
| P2-CV-06 | cross-project boundary | 06 | different project `internal` denied |
| P2-CV-07 | private semantic parity | 02,08,10 | owner accepted, external caller denied |
| P2-CV-08 | protected semantic parity | 02,08,10 | subclass accepted, unrelated caller denied |
| P2-CV-09 | lookup/access separation | 08,09 | inaccessible subclass method does not fall through |
| P2-CV-10 | runtime source origin | 04,05,06,12 | source module/package/internal positive + negative |
| P2-CV-11 | cached dispatch | 11 | warm cache authorization equals cold |
| P2-CV-12 | dynamic dispatch | 11 | dynamic/perform cannot bypass |
| P2-CV-13 | bound/reference path | 11 | bound method/reference cannot bypass |
| P2-CV-14 | reflection path | 11 | reflection cannot bypass module/package/internal |
| P2-CV-15 | native origin | 13 | installed native origin contains exact source module/package/project |
| P2-CV-16 | native access | 04,05,06,13 | native access matches source access |
| P2-CV-17 | native visibility mismatch | 16 | mismatch rejected |
| P2-CV-18 | native parameter mismatch | 16,17 | canonical parameter mismatch rejected |
| P2-CV-19 | native return mismatch | 16,17 | canonical return mismatch rejected |
| P2-CV-20 | implementation-kind identity | 03,18 | semantic signature uses common enum; provenance separate |
| P2-CV-21 | Foreign retention | 19 | bodyless Foreign remains Foreign in semantic/reflection product |
| P2-CV-22 | Foreign lowering | 20 | explicit unsupported-binding diagnostic |
| P2-CV-23 | LSP keyword parity | 23 | hover recognizes all seven; `public` ordinary |
| P2-CV-24 | LSP package completion | 05,07,10 | exact same/nested/different package completion |
| P2-CV-25 | standalone ownership | 04,05,06 | standalone Package/Module uses canonical topology |
| P2-CV-26 | modifier order/spec parity | 23 | spec/parser agree; reverse order rejected |
| P2-CV-27 | dunder/legacy regression | 01,22 | standardized dunders remain; `_$` remains nonsemantic |
| P2-CV-28 | metadata regression | 21 | primitive/intrinsic remain Universe-only metadata |
| P2-CV-29 | compiler synthetic authority | 14 | synthetic privilege remains transient |
| P2-CV-30 | no semantic total order | 26 | no access logic compares visibility ordinally |

---

## 16. Verification execution budget

### BUILD MODE

Use only exact regressions/directly affected tests. Do not broaden on PASS.

Typical commands using the planned P2 regression names (mechanically adapt only if the repository test harness requires qualification):

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::access_control -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_declaration_dispatch -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_authority -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib lang006_c1_p2_keyword_hover -- --nocapture
```

Do not run all after every task.

### STABILIZE MODE

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules lang006_c1_p2_package_ownership_is_exact -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::access_control -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core lang006_c1_p2_source_access_domains_runtime_parity -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-surface
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-decl --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib lang006_c1_p2 -- --nocapture
```

### CERTIFY MODE

Use G5 in §19.

### Only if evidence demands

- full `phalcom-core` library/unit suite;
- macro-generator tests if provenance rename changes generated code;
- snapshot/golden updates if intentional presentation changes require them.

### Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
broad unrelated language corpora
full performance suites
```

---

## 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by P2
B — probably caused by P2
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks acceptance.
- Do not weaken/skip assertions to make broad gates green.

Known predecessor baseline/deferred items:

- semantic incremental caller-identity assertion unrelated to C1 changes;
- core VM walk/fold ordering failure;
- repository-wide formatting drift.

Reclassify only if P2 evidence links them causally.

---

# 18. Tasks

## T0 — Reopen C1 and pin the corrective regression surface

### Purpose

Make checkpoint state truthful and create smallest failing tests that prove audited gaps before architecture edits.

### Preconditions

- live revision/status checked;
- P1 records read;
- unrelated work preserved.

### Consumes

- audit findings;
- P1 coverage ledger;
- current focused tests.

### Produces

- checkpoint reopened to P2;
- concrete discriminating regressions;
- exact live test filters.

### Files and symbols

**Modify:**

- `LANG006.C1-CHECKPOINT.md`
- focused existing tests or one coherent P2 module per owning test binary

### Required implementation shape

At plan start update checkpoint approximately:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG006.C1.P2
latest_completed_plan: LANG006.C1.P1
next_plan: null
```

Record that P1 remains historically completed as a plan record but post-P1 audit disproved checkpoint closure.

Add minimal discriminators for:

```text
semantic ordinary private/module/package/internal authorization
runtime source module/package/internal positive/negative
cross-project Internal
nested/sibling package distinction
Foreign explicit lowering diagnostic
native exact type mismatch
LSP new-keyword hover
```

It is acceptable for them to fail before later tasks.

### Forbidden approaches

- no production architecture fix in T0 except neutral testability hooks;
- no broad suite run;
- no rewriting P1 walkthrough/handoff.

### Test changes required

Coverage: P2-CV-01..06, P2-CV-18..19, P2-CV-22..23.

### Tests to run now

Run only new exact discriminators individually.

### Tests explicitly deferred

Cache/dynamic/bound/reflection parity, full semantic/core binaries, broad native suites, workspace checks.

### Acceptance

Checkpoint truthfully reopened and each critical defect has a reproducer or documented harness limitation.

### Local STOP / CONSULT triggers

Stop if expected audit defects cannot be reproduced because live tree materially changed.

### Checkpoint update

Record starting revision and reproduced defects.

---

## T1 — Publish exact semantic declaration/caller provenance

### Purpose

Make canonical module/package/project identity available to semantic access consumers without path inference.

### Preconditions

T0 complete.

### Consumes

`ModuleId`, canonical owning-package query/topology, `ProjectIdentity`, declaration owner identities.

### Produces

A semantic declaration/caller provenance product equivalent to module/package/project, with package always canonical for ordinary module-backed declarations.

### Files and symbols

- `phalcom-modules/src/query.rs`
- `phalcom-modules/src/topology.rs`
- semantic snapshot/module integration
- `phalcom-semantic/src/editor.rs`
- semantic access/context definitions
- module/semantic access tests

### Required implementation shape

1. Add/reuse snapshot/query `owning_package(ModuleId) -> ModuleId`.
2. Populate `AccessContext.caller_package` in production paths.
3. Obtain declaration package from declaration module through canonical products.
4. `Package` access becomes only exact package equality.
5. Delete parent-path/project heuristic fallback.
6. Keep Module/Internal exact equality.
7. Moving the access helper to a neutral semantic access module is preferred if it reduces checker/editor coupling.

### Forbidden approaches

No `path.parent()` access semantics, string matching, LSP-owned cache, filesystem inference, or optional fallback for ordinary module-backed declarations.

### Test changes required

P2-CV-03, 04, 05, 06, 25.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules lang006_c1_p2_package_ownership_is_exact -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::access_control -- --nocapture
```

### Tests explicitly deferred

Formal checker calls to T2, runtime to T3, LSP completion to T7.

### Acceptance

No package path fallback remains in semantic access and exact package tests pass.

### Local STOP / CONSULT triggers

Stop if package ownership cannot be queried without redefining module topology/package semantics.

### Checkpoint update

Record canonical semantic package-origin interface.

---

## T2 — Enforce canonical access in formal semantic dispatch

### Purpose

Make ordinary type/checker member resolution obey the same access relation as editor/runtime semantics.

### Preconditions

T1 exact provenance available.

### Consumes

Selected callable identity, selected visibility, declaration origin, checker access context, hierarchy.

### Produces

Formal access authorization and distinct inaccessible-member behavior.

### Files and symbols

- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/checker/context.rs`
- call/expression paths using canonical dispatch
- semantic diagnostics if needed
- conditional/inherent member metadata
- semantic checker tests

### Required implementation shape

1. Preserve enough selected-member metadata to authorize it.
2. After lookup selects target, call one semantic access relation.
3. On denial, retain existence and emit/return distinct access failure.
4. Do not continue superclass lookup after denial.
5. Apply to conditional inherent/ordinary user-visible member selection.
6. Preserve existing trait witness/coherence architecture.
7. Editor completion continues using same helper.

### Forbidden approaches

No editor-only correctness, no pre-filter that changes lookup shadowing, no “inaccessible == missing”, no superclass fallback, no checker rule weaker than runtime.

### Test changes required

P2-CV-01, 02, 07, 08, 09 plus existing trait visibility regressions.

### Tests to run now

Run new semantic access/call filters, then once:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Runtime/LSP/full semantic binary.

### Acceptance

Normal checked expressions cannot resolve inaccessible members and still distinguish them from nonexistent selectors.

### Local STOP / CONSULT triggers

Stop if access enforcement requires changing selector precedence, inheritance dispatch, or trait coherence.

### Checkpoint update

Record semantic authorization seam/access-failure representation.

---

## T3 — Stamp exact runtime origin on source methods and prove invocation parity

### Purpose

Repair source `module`, `package`, and `internal` runtime authorization and major invocation paths.

### Preconditions

T1 identities stable; T2 expected semantics established.

### Consumes

Runtime module object, `VM::access_origin_for_module`, source method constructors, central authorization helper.

### Produces

Exact source `MethodObject.origin` and direct/cached/dynamic/bound/reflection parity.

### Files and symbols

Read every production `MethodObject::new/new_single` producer, especially:

- method object;
- class/impl compiler lowering;
- chunk/data/ADT source-derived methods;
- VM send/dispatch;
- Universe/bootstrap synthetic constructors.

### Required implementation shape

1. Census/classify all method constructors A/B/C/D/E from §7.4.
2. Centralize source origin stamping where practical.
3. A/B use exact owning source module origin.
4. Keep `access_owner` separate.
5. Do not change `RuntimeAccessOrigin::none()` globally merely to hide missing call sites.
6. Keep authorization centralized.
7. Verify cache hits pass through authorization.
8. Verify binding/reference/dynamic/reflection cannot bypass.
9. Preserve transient compiler synthetic authority.

### Forbidden approaches

No target origin copied from current caller, Universe shortcut, compiler bypass for source methods, global Universe default, or weakened cache rule.

### Test changes required

P2-CV-10, 11, 12, 13, 14, 29.

### Tests to run now

Exact new core filters, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

### Tests explicitly deferred

Native to T4, full core binary to stabilization.

### Acceptance

Cold/warm source dispatch authorize Module/Package/Internal correctly and alternate paths cannot bypass.

### Local STOP / CONSULT triggers

Stop if cache architecture materially bypasses authorization and requires redesign.

### Checkpoint update

Record constructor classification and runtime stamping seam.

---

## T4 — Make native authority declaration-derived and enforce exact contracts

### Purpose

Replace project-only native origin with exact source declaration origin and close signature-conformance gaps.

### Preconditions

T1 and T3 stable.

### Consumes

Native source index/anchor, descriptor surface, canonical semantic signatures/types, module registry/materialization.

### Produces

Exact source module on native anchors, exact native runtime origin, exact canonical conformance.

### Files and symbols

- `phalcom-core/src/native/source.rs`
- `verify.rs`
- `install.rs`
- module registry/materialization lookup
- `phalcom-semantic/src/core_surface/conformance.rs`
- native semantic import/type formation
- native tests

### Required implementation shape

#### Source module identity

Add exact `ModuleId` to `NativeMemberAnchor` and populate during census.

#### Runtime origin

```text
anchor.module
→ runtime module handle
→ VM::access_origin_for_module
→ MethodObject.origin
```

No final `RuntimeAccessOrigin::universe()`.

#### Exact comparison

Verify owner, side, selector, member kind, shape, parameter types, return type, visibility, Native implementation kind.

`typed: bool` remains completeness precondition, not equality.

### Forbidden approaches

No string type comparison, duplicate type lowerer, nonpublic→Internal collapse, installer-context origin, or blanket native access.

### Test changes required

P2-CV-15..19.

### Tests to run now

Exact native filters, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

### Tests explicitly deferred

Broad native packages/full core until G3/G5.

### Acceptance

Native authority equals source declaration authority and canonical signature/type mismatches are rejected.

### Local STOP / CONSULT triggers

Stop if canonical signatures require major bootstrap lifecycle/dependency redesign.

### Checkpoint update

Record anchor module identity and conformance seam.

---

## T5 — Converge language ImplementationKind and separate execution provenance

### Purpose

Remove the second semantic authority named `ImplementationKind`.

### Preconditions

T4 conformance interface understood.

### Consumes

Common enum, native-meta provenance enum, semantic signatures, presentation/reflection.

### Produces

One language implementation category and separately named provenance.

### Files and symbols

- `phalcom-native-meta/src/primitive.rs`
- all consumers of `phalcom_native_meta::ImplementationKind`
- `phalcom-semantic/src/signature.rs`
- signature construction/projection
- core-surface presentation
- `phalcom-core/src/primitive/method.rs`
- generated metadata if affected

### Required implementation shape

1. Rename native-meta axis to provenance-oriented type.
2. Language-facing semantic `implementation` fields use common enum.
3. Preserve provenance separately only where required.
4. Audit old provenance variants contextually:
   - Source normally Phalcom + Source provenance;
   - NativePrimitive Native + NativePrimitive;
   - Generated language kind comes from declaration contract;
   - Abstract language kind comes from declaration semantics, abstractness separate;
   - External is not automatically Foreign without source authority.
5. `implementationKind` reflection/presentation returns only language categories.
6. No two active semantic authorities remain.

### Forbidden approaches

No ordinal enum mapping, unconditional Generated→Phalcom or External→Foreign, no new public provenance API.

### Test changes required

P2-CV-20, 30 plus existing presentation/reflection regressions.

### Tests to run now

Focused semantic signature/presentation/method-reflection filters; generated-code tests only if affected.

### Tests explicitly deferred

Foreign lowering to T6, broad native suites to G3.

### Acceptance

Repository uses common language `ImplementationKind` for language facts; old Source/NativePrimitive/Generated/Abstract/External axis has provenance name.

### Local STOP / CONSULT triggers

Stop if stable external API semantics make simple split materially incompatible.

### Checkpoint update

Record final language-kind/provenance split.

---

## T6 — Complete bounded Foreign semantic and lowering scaffold

### Purpose

Make Foreign survive required products and fail explicitly at unsupported binding boundary.

### Preconditions

T5 complete.

### Consumes

AST Foreign, semantic publication, source index/reflection, compiler lowering.

### Produces

Foreign-retaining semantic products and explicit unsupported-binding diagnostic.

### Files and symbols

- semantic declaration/signature builders
- core-surface source/presentation
- source index
- class/impl compiler lowering
- compiler diagnostics
- LSP/reflection consumers

### Required implementation shape

1. Preserve `ImplementationKind::Foreign` in semantic signatures/member records.
2. Keep P1 parser legality.
3. Legal bodyless Foreign without backend emits `foreign.binding_unimplemented` or exact approved spelling.
4. This precedes generic declaration-without-body error.
5. Reflection/tooling reports `foreign`.
6. No runtime method installation.

### Forbidden approaches

No native registry, stub body, method-missing backend, ABI, pointer/marshal/ownership rules.

### Test changes required

P2-CV-21, 22 and retain P1 parser Foreign tests.

### Tests to run now

Exact semantic Foreign-retention and compiler Foreign-diagnostic filters.

### Tests explicitly deferred

No external FFI tests; no backend exists by design.

### Acceptance

Foreign is distinct in semantic/reflection products and deterministically fails pre-runtime with dedicated diagnostic.

### Local STOP / CONSULT triggers

Any ABI/loader/ownership/marshaling pressure.

### Checkpoint update

Record Foreign boundary/diagnostic.

---

## T7 — Converge LSP, reflection, modifier spec, and visibility-order hardening

### Purpose

Finish user-facing parity and repair authoritative specification contradiction.

### Preconditions

T1/T2 and T5/T6 stable.

### Consumes

Canonical access context, common implementation category, accepted modifier order.

### Produces

Exact-package completion, seven-keyword hover, corrected current spec, no semantic Visibility total-order trap.

### Files and symbols

- `phalcom-lsp/src/hover.rs`
- LSP completion/backend consumers
- LSP tests
- `docs/specs/source-declarations.md`
- `phalcom-common/src/modifier.rs`
- any visibility ordering consumers

### Required implementation shape

#### LSP

- hover docs/spelling for all seven keywords;
- `public` remains ordinary identifier;
- completion obtains exact caller package from semantic product;
- no LSP path inference.

#### Specification

Use wording equivalent to:

```text
At most one visibility modifier and at most one implementation modifier may
precede a declaration. When both are present, visibility precedes
implementation. Normalized products are orthogonal, but noncanonical authored
permutations are rejected.
```

#### Visibility ordinal hardening

Search ordering/sorting uses. If `Ord/PartialOrd` are not stable/API requirements, remove. If deterministic presentation needs ordering, use named non-semantic sort key. No access logic may depend on enum ordinal.

### Forbidden approaches

No `public` keyword docs, no legalizing `native internal foo`, no path package completion, no semantic `visibility >= ...`.

### Test changes required

P2-CV-23, 24, 26, 30.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib lang006_c1_p2_keyword_hover -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib lang006_c1_p2_package_completion -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --lib lang006_modifier_negative_diagnostics -- --nocapture
```

### Tests explicitly deferred

Full LSP to G4/G5.

### Acceptance

Tooling and current spec present same seven-keyword, exact-package, visibility-before-implementation model.

### Local STOP / CONSULT triggers

Stop if authoritative specs/TDR materially conflict with preserving accepted order rather than containing identified wording defect.

### Checkpoint update

Record spec correction/tooling parity.

---

## T8 — Stabilize, certify, and re-close LANG006.C1

### Purpose

Replace P1 deferred closure assertions with concrete evidence and durable records.

### Preconditions

T1–T7 locally accepted; no unresolved consultation or invariant violation.

### Consumes

All coverage obligations, baseline ledger, final diff.

### Produces

Final coverage ledger, corrected checkpoint status, P2 walkthrough, P2 handoff.

### Required implementation shape

1. Run G1–G5.
2. Record every actual test/result.
3. Classify failures A/B/C/D.
4. Do not claim RELEASE_COMPLETE without release evidence.
5. If all acceptance passes, checkpoint may return to:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
active_plan: null
latest_completed_plan: LANG006.C1.P2
next_plan: null
```

6. If unrelated baseline blocks a named gate, use truthful blocked verification state.
7. Walkthrough states P2 corrects post-P1 audit findings.

### Forbidden approaches

No “implemented by inspection” for required coverage, no deferred access/cache/package/cross-project obligation, no workspace claim if not run, no broad formatting rewrite.

### Acceptance

All P2 coverage has passing evidence and durable records are truthful.

### Local STOP / CONSULT triggers

Stop before completion if any invariant or required coverage remains unresolved.

### Checkpoint update

Final closure.

---

# 19. Verification gates

## G1 — Exact semantic access identity

Run exact live equivalents:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules lang006_c1_p2_package_ownership_is_exact -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::access_control -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic lang006_c1_p2_checker_access_domains -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Expected:

```text
same module/package/project positives pass
cross-domain negatives are access failures
nested/sibling package exact
private/protected regressions pass
inaccessible shadow does not fall through
no path fallback remains
```

---

## G2 — Runtime invocation parity

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core lang006_c1_p2_source_access_domains_runtime_parity -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core lang006_c1_p2_invocation_path_access_parity -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_internal_dispatch_authority_is_transient -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Expected direct == cached == dynamic == bound/reference == reflection authorization, with exact Module/Package/Internal and transient compiler authority.

---

## G3 — Native and Foreign boundary correctness

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-surface
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-decl --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic lang006_c1_p2_foreign_semantic_retention -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core lang006_c1_p2_native_origin_conformance_and_foreign_boundary -- --nocapture
```

Expected exact native origin, mismatch rejection, one language ImplementationKind, distinct bounded Foreign, no ABI/loader.

---

## G4 — Tooling/specification parity

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib lang006_c1_p2 -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --lib lang006_declaration_modifiers_and_ast_storage -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --lib lang006_modifier_negative_diagnostics -- --nocapture
```

Inspect:

```sh
rg -n 'order is otherwise insignificant|native internal' docs/specs/source-declarations.md docs/implementation/LANG006/LANG006.C1
rg -n 'path\.parent\(\)' phalcom-semantic/src/editor.rs phalcom-lsp/src
rg -n 'enum ImplementationKind' phalcom-common phalcom-native-meta phalcom-semantic
```

Expected:

```text
seven keyword hover/token parity
public remains non-keyword
completion exact-package
spec visibility-before-implementation
no conflicting language ImplementationKind
no package-path fallback
```

Classify historical diagnostic examples rather than blindly deleting grep matches.

---

## G5 — Corrective checkpoint certification

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-common -p phalcom-ast -p phalcom-modules -p phalcom-semantic -p phalcom-native-meta -p phalcom-native-surface -p phalcom-core -p phalcom-lsp
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
git diff --check
```

If known baseline failure appears:

- classify under §17;
- run smallest discriminator proving unrelatedness;
- record it;
- do not repair unrelated semantics.

Do not automatically run:

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
```

G5 passes only if P2-required coverage is green and remaining red tests are credibly unrelated.

---

## 20. Final focused acceptance

The P2 walkthrough must map every coverage ID to a concrete test and PASS result.

No P2-specific access/cache/package/cross-project/native-signature/Foreign/tooling obligation may remain `DEFERRED`.

The plan may finish `FOCUSED_TESTED`; it must not claim release certification without release-level evidence.

---

## 21. Performance/resource evidence

P2 has no performance optimization objective.

Required negative discipline:

- no per-send filesystem/path package resolution;
- no repeated source parsing for authorization;
- no per-instance access metadata;
- preserve cached module/package/runtime identities;
- access checks remain compact identity comparisons plus hierarchy check for Protected.

No benchmarks required unless evidence shows material hot-path regression.

---

## 22. Checkpoint bookkeeping

### At plan start

Set:

```text
status = IN_PROGRESS
completion = PARTIAL
verification = UNVERIFIED
active_plan = LANG006.C1.P2
latest_completed_plan = LANG006.C1.P1
```

Record starting revision, audit-driven reopening, reproduced defects, carried baseline failures.

### During plan

Update only for durable interfaces, consultation/amendment, deferred baseline failures, and coherent G1–G4 gates.

### At plan completion

Record P2 completion, final origin architecture, implementation-kind split, Foreign boundary, native conformance seam, tests, classified failures, final verification, next action.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-walkthrough.md
```

Must contain:

- final result;
- audit findings corrected;
- semantic access/selected-call authorization flow;
- exact module/package/project provenance flow;
- runtime source-origin classification;
- native source-origin/conformance flow;
- language ImplementationKind vs provenance;
- Foreign bounded behavior;
- tooling/spec corrections;
- important code changes;
- deviations;
- consultations;
- P2 coverage table with exact tests;
- tests run/not run;
- baseline classifications;
- verification classification;
- residual risks.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-handoff.md
```

Must contain:

- current revision/state;
- stable P2 interfaces;
- canonical owners for access facts;
- semantic access seam;
- runtime origin seam;
- native origin/conformance seam;
- implementation-kind/provenance split;
- Foreign boundary;
- invariants;
- unrelated deferred failures;
- first recommended future commands;
- things not to redesign;
- whether C1 is honestly closed.

If closed, state no additional `LANG006.C1` plan is required.

---

## 25. Completion truth table

Do not conflate:

```text
source written
tests added
targeted tests passed
affected suite passed
checkpoint accepted
release certified
```

- source implemented but focused tests incomplete → `PARTIAL / UNVERIFIED`;
- P2 implemented + focused gates pass → `IMPLEMENTED / FOCUSED_TESTED`;
- checkpoint returns to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only after G5 + full P2 coverage;
- no `RELEASE_COMPLETE` without release-level gates.

---

## 26. Plan self-review

### Architecture

- [x] one package owner;
- [x] semantic authority distinct from editor presentation;
- [x] lookup/authorization order explicit;
- [x] runtime origin is projection, not guess;
- [x] native authority source-derived;
- [x] type/signature equality reuses semantic products;
- [x] language implementation category separate from provenance;
- [x] Foreign bounded/ABI-free;
- [x] synthetic privilege separate.

### Luna executability

- [x] dependency-ordered tasks;
- [x] FIXED/FLEXIBLE/VERIFY-FIRST explicit;
- [x] objective STOP/CONSULT triggers;
- [x] bounded debugging;
- [x] named source/test areas;
- [x] broad testing deferred during BUILD.

### Testing

- [x] every audited correctness gap covered;
- [x] package nested/sibling boundaries explicit;
- [x] source/native origins covered;
- [x] cache/dynamic/bound/reflection parity covered;
- [x] exact native type mismatches covered;
- [x] Foreign retention/lowering covered;
- [x] LSP/spec parity covered;
- [x] unrelated failure policy included.

### Documentation lifecycle

- [x] checkpoint reopening/reclosure explicit;
- [x] walkthrough/handoff mandatory;
- [x] P1 preserved historically.

### Premium-compute discipline

- [x] no routine coding requires adviser;
- [x] bootstrap/type-conformance lifecycle conflict pre-identified;
- [x] Foreign ABI pressure immediate escalation;
- [x] package-identity redesign immediate escalation.

---

# Final implementation order

```text
T0  reopen + pin regressions
 ↓
T1  canonical semantic module/package/project provenance
 ↓
T2  formal checker access enforcement
 ↓
T3  source MethodObject runtime origin + invocation parity
 ↓
T4  native declaration origin + exact source/descriptor conformance
 ↓
T5  one language ImplementationKind + separate provenance
 ↓
T6  bounded Foreign semantic/lowering scaffold
 ↓
T7  LSP/reflection/spec/order convergence + ordinal hardening
 ↓
T8  stabilize, certify, record, re-close C1
```

Do not reorder T2 before T1, T4 before T3, or T6 before T5. Those dependencies are architectural, not ceremonial.
