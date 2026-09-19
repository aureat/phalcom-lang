---
id: LANG006.C2.P1
category: LANG
program: LANG006
checkpoint: LANG006.C2
kind: implementation-plan
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
depends_on:
  - LANG006.C1.P2
  - LANG005.C1
follows: LANG006.C1.P2
supersedes: null
repository: aureat/phalcom-lang
branch: main
planning_revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
planning_revision_subject: "feat(lang006): implement C1 corrective closure"
prepared: 2026-09-19
---

# LANG006.C2.P1 — Validated Data Construction and Primary-Constructor Visibility

## 0. Executor contract

This plan is designed for **GPT-5.6 Luna High/Extra High** as the primary implementer under the Phalcom principal-architect workflow.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, authority boundaries, and invariants below;
- must not invent a second visibility model for `data`;
- must not implement constructor access in the VM as the primary enforcement mechanism;
- must not make `data` opaque merely because construction becomes restricted;
- must not reinterpret top-level declaration visibility or export semantics;
- must not introduce a `public` source keyword;
- must not invent `protected` semantics for non-inheritable `data`;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep the shared checkpoint record current;
- must create the required walkthrough and handoff.

Testing is evidence gathering, not ritual. Do not run broad suites merely because a field, parser branch, or helper changed.

This is a **planning artifact**, not evidence that implementation or tests have already run.

---

# Checkpoint / evidence map

This plan belongs to one repository checkpoint, `LANG006.C2`. Per current Phalcom implementation-record convention, it does **not** invent nested plan-local `C0/C1/...` checkpoint identifiers. Semantic evidence is instead scheduled at named `G` gates.

| Owning checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| `LANG006.C2` / surface + product | T0–T2 | constructor visibility is parsed, normalized, published in the canonical data constructor product, and fingerprinted | AST syntax suite, data semantic product tests, semantic compile check | cross-module access, runtime |
| `LANG006.C2` / direct authorization | T3 | tuple- and record-shaped direct construction use the canonical identity-based access relation | focused data-constructor visibility matrix, hostile private/module/package/internal boundaries | callable-family capture, optimizer |
| `LANG006.C2` / capability closure | T4 | exact constructor references and family capture cannot acquire inaccessible constructors; legitimately acquired capabilities remain stable values | associated data-constructor reference/family tests | broad ADT suite |
| `LANG006.C2` / persistence + representation | T5–T6 | visibility survives generics, nullary construction, re-export, incremental edits, lowering, and optimizer mode without becoming runtime value metadata | incremental cold/edit parity, re-export tests, nullary test, optimizer Enabled/Disabled differential | full semantic/core/workspace suites |
| `LANG006.C2` / stabilization | T7 | specs, checkpoint state, focused verification, walkthrough, and handoff agree with the implemented model | G5 focused acceptance and `git diff --check` | release certification unless explicitly requested |

---

## 1. Goal

Implement Phalcom's **validated-data construction model** by allowing a `data` declaration to restrict its compiler-generated primary constructor with `private`, `module`, `package`, or `internal` visibility while preserving public-by-default construction, transparent immutable components, canonical constructor identity, first-class constructor capability semantics, generic/nullary behavior, and representation-independent lowering.

---

## 2. Checkpoint acceptance objective

`LANG006.C2` is accepted when all of the following are true:

1. These source forms are supported:

   ```phalcom
   data Point(x: Int, y: Int)

   data Date private(
       year: Int,
       month: Month,
       day: Int
   )

   data Token module(
       kind: TokenKind,
       span: Span
   )

   data PackageKey package(
       value: String
   )

   data RuntimeId internal(
       raw: Int
   )

   data Person private {
       firstName: String,
       lastName: String,
       age: Int
   }
   ```

2. Omitted constructor visibility means canonical `Visibility::Public`. There is no `public` source keyword.
3. `protected` is rejected for a data primary constructor because data types are non-inheritable.
4. Primary-constructor visibility is independent from whether the `data` declaration's name is exported/importable.
5. The canonical semantic constructor product owns the visibility fact.
6. The canonical `phalcom_semantic::access::is_member_accessible` relation owns authorization.
7. `private` means the exact declaring `data` owner; another class in the same module is not authorized.
8. `module`, `package`, and `internal` use exact canonical module/package/project identity.
9. Tuple construction and record construction enforce the same rule.
10. Constructor callable references and associated-family capture cannot bypass the rule.
11. A constructor capability that was legally acquired in an authorized context remains a stable capability when passed elsewhere; lexical visibility is not re-run at every eventual invocation.
12. Generics do not alter visibility or `DataConstructorId`.
13. Nullary data construction does not bypass access merely because it lowers to a singleton load.
14. Re-exporting a data type does not widen constructor visibility.
15. A visibility-only edit invalidates the semantic constructor product and dependent acquisition/construction analysis without changing declaration/constructor identity.
16. Product optimization does not change whether construction is authorized.
17. No ordinary `DataObject`, `DataSingleton`, runtime data descriptor, or product layout gains per-value constructor-visibility metadata.
18. The owning `docs/specs/` documents state the final semantics and no longer leave primary-constructor visibility reserved/unspecified.

This plan is intended as **full checkpoint closure** for the validated-data construction feature, subject to the repository's existing unrelated/baseline failures.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
remote branch: main
remote revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
revision subject: feat(lang006): implement C1 corrective closure
inspection date: 2026-09-19
local working tree: NOT VISIBLE through the repository connector
relevant predecessor: LANG006.C1.P2
```

Important verified repository facts:

- `docs/specs/` is the authoritative specification corpus.
- `docs/specs/source-declarations.md` owns the general `Visibility` access relation.
- `docs/specs/objects/data-objects.md` owns data-object and primary-constructor semantics and currently leaves primary-constructor visibility explicitly reserved.
- `phalcom-common/src/modifier.rs::Visibility` already defines `Public`, `Private`, `Protected`, `Module`, `Package`, and `Internal`.
- `phalcom-semantic/src/access.rs::is_member_accessible` already implements the canonical identity-based access relation.
- `phalcom-semantic/src/checker/context.rs` already projects the current declaration/module/package/project context and authorizes ordinary dispatch after target selection.
- `phalcom-ast/src/parser.rs::parse_top_item` intentionally rejects **pre-declaration** top-level visibility modifiers and directs users to exports.
- `phalcom-ast/src/parser.rs::parse_data` currently parses name → generics → optional `where` → shape → optional trailing `where`.
- `phalcom-ast/src/ast.rs::DataDef` currently contains no primary-constructor visibility field.
- `phalcom-semantic/src/data_semantics.rs::DataConstructorSignature` currently contains constructor identity, parameters, result template, and source, but no visibility.
- `phalcom-semantic/src/checker/expression.rs::callable_target_for_data_info` constructs direct data-constructor application targets with no authorization.
- both tuple-style direct construction and record construction reach `callable_target_for_data_info`.
- `CallableApplicationTarget` already has `access_denied`, and `apply_resolved_callable` already fails inaccessible callable application with `DiagnosticCode::MemberInaccessible`.
- `phalcom-semantic/src/checker/associated.rs::specialize_associated_member` currently reifies `AssociatedMemberId::DataConstructor` without a visibility check.
- `phalcom-semantic/tests/semantic/adts/associated/scenarios.rs` already proves exact data-constructor references and whole-family capture:
  - `&Point::Point(_, _)`
  - `&Point::Point...`
- `phalcom-semantic/src/db/fingerprint.rs` hashes data constructor identity/result shape but currently omits constructor visibility.
- `phalcom-semantic/tests/semantic/incremental/adts.rs` already contains `data_incr_*` coverage and an existing cold/incremental data equivalence test.
- `phalcom-core/src/compiler/lib/product_opt.rs` can scalar-replace data products.
- `phalcom-core/tests/core/compiler/product_optimization.rs` already provides `ProductOptimizationMode::Enabled` / `Disabled` differential infrastructure.
- nullary data construction can lower through `LoadDataSingleton`; positive arity uses `ConstructData`.
- the current runtime data representation does not need constructor visibility to represent an already-created data value.

> **Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.**

Do not assume the user's local checkout is exactly the inspected remote revision. Record the actual local revision in the checkpoint before editing.

---

## 4. Required reads before implementation

Read these in order before making code changes:

```text
1. AGENTS.md
2. docs/implementation/README.md
3. docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
4. docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-corrective-access-provenance-and-implementation-kind-closure.md
5. docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-walkthrough.md
6. docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-handoff.md
7. docs/specs/README.md
8. docs/specs/source-declarations.md
9. docs/specs/objects/data-objects.md
10. docs/specs/objects/impl.md
11. docs/decisions/accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md
12. docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md
13. phalcom-common/src/modifier.rs
14. phalcom-ast/src/ast.rs
15. phalcom-ast/src/parser.rs
16. phalcom-ast/tests/data_syntax.rs
17. phalcom-semantic/src/access.rs
18. phalcom-semantic/src/data_semantics.rs
19. phalcom-semantic/src/checker/data_declaration.rs
20. phalcom-semantic/src/checker/context.rs
21. phalcom-semantic/src/checker/call.rs
22. phalcom-semantic/src/checker/expression.rs
23. phalcom-semantic/src/checker/associated.rs
24. phalcom-semantic/src/db/fingerprint.rs
25. phalcom-semantic/tests/semantic/README.md
26. phalcom-semantic/tests/semantic/adts/data_declarations.rs
27. phalcom-semantic/tests/semantic/adts/associated/scenarios.rs
28. phalcom-semantic/tests/semantic/incremental/adts.rs
29. phalcom-semantic/tests/semantic/integration/access_control.rs
30. phalcom-core/tests/README.md
31. phalcom-core/src/modules/semantic_lowering.rs
32. phalcom-core/src/compiler/lib/associated.rs
33. phalcom-core/src/compiler/lib/product_opt.rs
34. phalcom-core/src/data.rs
35. phalcom-core/src/vm/data.rs
36. phalcom-core/tests/core/compiler/product_optimization.rs
```

Avoid broad repository exploration unless one of these files contradicts the plan or a required caller cannot be found.

---

## 5. Normative authority

### 5.1 Current authoritative specifications

`docs/specs/source-declarations.md` owns the general access relation:

```text
Public    any caller
Private   exact declaring owner identity
Protected declaring owner / eligible subclass
Module    exact declaring module
Package   exact declaring package
Internal  exact declaring project
```

`docs/specs/objects/data-objects.md` owns data constructor semantics:

- a primary data constructor belongs to the `data` declaration;
- it is not syntactic sugar for `.new`;
- its shape derives from logical data components;
- generic inference is ordinary Phalcom generic inference;
- data is non-inheritable;
- components are transparent immutable semantic state;
- physical representation may change without changing semantics.

The same file currently says primary-constructor visibility is separately specified. This plan closes that reserved question.

### 5.2 Accepted decision records

`TDR-0080` establishes:

- `data` as a distinct nominal transparent immutable value product;
- primary constructor semantic identity;
- non-subclassability;
- exact generic identity;
- nullary singleton semantics;
- representation freedom.

`TDR-0082` establishes:

- the shared `Visibility` category;
- omitted visibility = Public;
- no `public` keyword;
- owner/module/package/project identity-based access;
- no naming-convention visibility;
- native/runtime mechanics do not create alternate access authority.

### 5.3 User-ratified decisions for this checkpoint

The following are fixed semantic inputs:

```phalcom
data Point(x: Int, y: Int)

data Date private(
    year: Int,
    month: Month,
    day: Int
)

data Token module(
    kind: TokenKind,
    span: Span
)

data PackageKey package(
    value: String
)

data RuntimeId internal(
    raw: Int
)

data Person private {
    firstName: String,
    lastName: String,
    age: Int
}
```

The visibility token is a **post-name primary-constructor modifier**, not top-level declaration visibility.

A validated factory is ordinary behavior declared on the data type and may invoke the raw primary constructor because it executes with the data declaration as its owner:

```phalcom
data Date private(
    year: Int,
    month: Month,
    day: Int
)

impl Date {
    @class
    new(
        year: Int,
        month: Int,
        day: Int
    ) -> Date {
        if month < 1 or month > 12 {
            // reject invalid month
        }

        if day < 1 or day > daysInMonth(year, month) {
            // reject invalid day
        }

        Date(
            year,
            Month.fromInt(month),
            day
        )
    }
}
```

The visibility model controls **raw primary construction**, not data-component observation.

### 5.4 Future / non-normative ideas explicitly excluded

This plan does not settle:

- automatically generated validating constructors;
- invariant declarations on data;
- private/opaque components;
- copy/update syntax;
- data pattern syntax;
- enum constructor-visibility cleanup;
- a new reflection API for data constructors;
- FFI layout;
- protected data constructors.

---

## 6. Takeover state

### Stable interface map

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| visibility category | `phalcom_common::modifier::Visibility` | `phalcom-common` | one shared enum; absent source visibility = Public |
| access relation | `phalcom_semantic::access::is_member_accessible` | `phalcom-semantic` | access is identity/provenance based |
| caller access context | `CheckingContext::access_context` | semantic checker | caller declaration/module/package/project are canonical facts |
| module/package provenance | `ModuleTopology::owning_package` and canonical IDs | `phalcom-modules` | no path-string package inference |
| data AST | `phalcom_ast::ast::DataDef` | `phalcom-ast` | source data declaration facts |
| data semantic product | `DataInfo` / `DataConstructorSignature` | `phalcom-semantic` | canonical data declaration/constructor facts |
| constructor identity | `DataConstructorId` | semantic identity layer | visibility is metadata, not identity |
| direct constructor application | `callable_target_for_data_info` → `apply_resolved_callable` | semantic checker | ordinary canonical call application |
| inaccessible-call gate | `CallableApplicationTarget::access_denied` | semantic checker | selected target may be inaccessible without becoming “missing” |
| associated constructor acquisition | `specialize_associated_member` | semantic checker | exact associated value/capability specialization |
| exact/family constructor references | `synthesize_associated_callable_reference` | semantic checker | capability set is captured semantically before lowering |
| data fingerprint | `data_declaration_*_fingerprint` | semantic DB | semantic changes invalidate relevant products |
| lowering | data constructor lowering specs | `phalcom-core::modules::semantic_lowering` | consumes semantic facts; does not resolve access |
| runtime representation | runtime data descriptor / `DataObject` / `DataSingleton` | `phalcom-core` | represents values, not lexical construction authority |
| optimizer | `compiler/lib/product_opt.rs` | compiler | may eliminate materialization but not semantic authorization |

### Inherited state from LANG006.C1

C1 established the exact access provenance substrate this feature must reuse. Do not reopen:

- definition of `Private`, `Module`, `Package`, or `Internal`;
- canonical package/project identity;
- selected-target authorization ordering;
- Universe special-case removal;
- visibility enum ownership.

### Inherited state from LANG005.C1

The data pipeline already owns:

- nominal data declaration identity;
- component identities;
- primary constructor identity;
- tuple/record construction;
- nullary singleton lowering;
- semantic lowering;
- runtime product representation;
- representation-aware optimization.

C2 extends **authorization**, not data representation.

---

## 7. Architecture

### 7.1 Source-to-runtime authority flow

```text
source
  data Date private(...)
        │
        ▼
phalcom-ast::DataDef
  constructor_visibility = Visibility::Private
        │
        ▼
phalcom-semantic::DataConstructorSignature
  visibility = Visibility::Private
        │
        ├────────────── direct construction ──────────────┐
        │                                                 │
        ▼                                                 ▼
canonical access relation                         associated acquisition
is_member_accessible(...)                         specialize_associated_member(...)
        │                                                 │
        ▼                                                 ▼
CallableApplicationTarget /                      frozen exact/family capability
semantic resolution                                      │
        └──────────────────────┬──────────────────────────┘
                               ▼
                       semantic lowering facts
                               ▼
                         compiler / bytecode
                               ▼
                             VM
```

The downward rule is strict:

```text
semantic facts
    ↓ projected as stable lowering facts
compiler
    ↓ bytecode/runtime metadata
VM/runtime
```

The compiler/VM must never reconstruct constructor visibility from source text, selector spelling, module path strings, or runtime object shape.

### 7.2 Direct construction

Both of these must use the same authorization predicate:

```phalcom
Point(x: 1, y: 2)

Person {
    firstName: "Ada",
    lastName: "Lovelace",
    age: 36
}
```

At the point `CallableApplicationTarget` is created for a data constructor, it must carry the result of canonical constructor authorization. `apply_resolved_callable` remains the canonical application funnel.

### 7.3 Constructor capability acquisition

These are semantically stronger than a direct call because they can escape:

```phalcom
&Point::Point(_, _)
&Point::Point...
```

An inaccessible constructor must not be published into an exact or family capability.

Authorization therefore occurs when the associated constructor capability is **acquired/specialized**, not deferred until arbitrary future invocation.

Once an authorized context has acquired such a capability and intentionally returns/passes it, the capability is a stable value. Do not re-run lexical access at its eventual call site.

This prevents both failure modes:

```text
WRONG A:
capture private constructor anywhere
→ hope runtime invocation later rejects it

WRONG B:
authorized factory returns constructor capability
→ external invocation rechecks lexical private access
→ legitimate capability becomes unusable
```

The correct model is:

```text
acquisition is visibility-sensitive
captured capability is stable
dynamic family application routes only among the frozen authorized candidates
```

### 7.4 Representation boundary

Constructor visibility is **not data value state**.

Do not add it to:

- every `DataObject`;
- `DataSingleton`;
- product slots/layout;
- exact applied runtime type identity;
- equality/hash state;
- GC state.

Ordinary compiled construction should be authorized before lowering. This is required because:

- a nullary constructor may become `LoadDataSingleton`;
- positive-arity construction may become `ConstructData`;
- the product optimizer may scalar-replace construction entirely.

A runtime-only check would therefore make semantics depend on representation.

---

## 8. Ownership boundaries

### Owns

This plan owns:

- data primary-constructor visibility syntax;
- AST normalization for that syntax;
- semantic constructor visibility metadata;
- constructor authorization using existing access laws;
- direct-construction parity;
- associated constructor capability acquisition;
- incremental fingerprint/invalidation for visibility;
- tests for module/package/project boundaries;
- validated factory use cases;
- spec closure of the reserved data-constructor visibility question.

### Does not own

This plan does not own:

- general method/field visibility;
- export/import name visibility;
- new package identity rules;
- class `protected` semantics;
- component visibility;
- data layout;
- pattern semantics;
- generic inference redesign;
- new callable-family type semantics;
- runtime reflection API expansion;
- enum constructor-visibility redesign;
- new validation/invariant language syntax.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| visibility enum | `phalcom_common::modifier::Visibility` | AST, semantic products, diagnostics | data-specific visibility enum |
| primary constructor visibility | `DataConstructorSignature.visibility` derived from `DataDef` | direct application, associated acquisition, tooling if needed | visibility stored independently in associated/lowering/runtime tables |
| access law | `phalcom_semantic::access::is_member_accessible` | ordinary dispatch + data constructor authorization | data-specific owner/package/project comparison |
| defining package | `ModuleTopology::owning_package` | access helper | parent-path or string-prefix inference |
| constructor identity | `DataConstructorId` | semantic/lowering/runtime mapping | visibility-sensitive constructor ID |
| constructor candidate set | semantic associated resolution/denotation | lowering | runtime rediscovery by name |
| data representation | existing runtime descriptor/layout | VM/compiler | constructor visibility in each data value |
| incremental semantic relevance | data declaration fingerprint | semantic DB | ad-hoc caller cache busting |

---

## 9. Global invariants

`INV-01` — Omitted data primary-constructor visibility is `Visibility::Public`.

`INV-02` — Source has no `public` constructor-visibility keyword.

`INV-03` — Data constructor visibility reuses `phalcom_common::modifier::Visibility`; there is no parallel enum.

`INV-04` — `protected` is invalid for a data primary constructor because data declarations are non-inheritable.

`INV-05` — Primary-constructor visibility is independent from top-level declaration export/import visibility.

`INV-06` — `Private` access requires exact declaring data owner identity, not merely the same module.

`INV-07` — `Module` access requires exact defining `ModuleId`.

`INV-08` — `Package` access requires exact canonical owning package identity.

`INV-09` — `Internal` access requires exact canonical project identity.

`INV-10` — No access decision reconstructs module/package/project authority from paths or strings.

`INV-11` — Tuple-shaped and record-shaped primary constructors have identical visibility semantics.

`INV-12` — `DataConstructorId` is unchanged by visibility edits.

`INV-13` — Generic specialization does not change constructor visibility.

`INV-14` — Direct construction and associated constructor capability acquisition consume the same access relation.

`INV-15` — Exact callable references cannot acquire an inaccessible data constructor.

`INV-16` — Whole-family references cannot capture an inaccessible data constructor.

`INV-17` — Dynamic/rest family invocation routes only among the already-authorized frozen candidates.

`INV-18` — A constructor capability legally acquired in an authorized scope remains a stable capability after escape.

`INV-19` — Constructor visibility does not change data component transparency, projection, equality, hashing, class/type identity, or rendering.

`INV-20` — Constructor visibility does not itself restrict pattern/deconstruction semantics.

`INV-21` — Nullary data does not bypass access even if lowering uses `LoadDataSingleton`.

`INV-22` — Product optimization does not alter authorization or diagnostics.

`INV-23` — Re-export/import may make the data type name available but cannot widen constructor visibility.

`INV-24` — Visibility participates in data semantic input/product fingerprints.

`INV-25` — A visibility-only edit invalidates dependent constructor acquisition/application while retaining unaffected products when their dependencies do not change.

`INV-26` — Ordinary runtime data descriptors/values do not become a second constructor-access authority.

`INV-27` — Reflection or tooling that eventually exposes a constructor capability must consume semantic visibility; this plan does not invent such an API.

`INV-28` — Diagnostics distinguish “constructor exists but is inaccessible” from “constructor/member is missing.”

---

## 10. Non-goals

Do **not** expand this plan into:

- `private` data components;
- opaque data values;
- mutation or setters for data components;
- automatic `new` generation;
- automatic validation predicates;
- contracts/invariants syntax;
- generalized constructor declaration syntax;
- multiple primary constructors;
- subclassing data;
- `protected` data-constructor semantics;
- enum constructor visibility cleanup;
- runtime FFI/ABI visibility;
- a new runtime capability security model;
- a new reflection constructor API;
- an LSP redesign;
- whole-workspace baseline repair.

The validated-data model here is deliberately simple:

```text
restricted raw constructor
+
ordinary public factory behavior
=
validated construction policy controlled by user code
```

---

## 11. Expected impact map

### Expected source areas

- `phalcom-ast/src/ast.rs` — add constructor visibility facts to `DataDef`.
- `phalcom-ast/src/parser.rs` — parse post-name constructor visibility and reject invalid forms.
- `phalcom-semantic/src/data_semantics.rs` — publish canonical constructor visibility.
- `phalcom-semantic/src/checker/data_declaration.rs` — project AST visibility into `DataConstructorSignature`.
- `phalcom-semantic/src/checker/context.rs` — expose a reusable declaration-member authorization helper based on the existing access relation.
- `phalcom-semantic/src/checker/expression.rs` — authorize direct tuple/record data construction.
- `phalcom-semantic/src/checker/associated.rs` — authorize data-constructor associated acquisition/specialization.
- `phalcom-semantic/src/checker/call.rs` — only if needed to tailor inaccessible-constructor diagnostics without duplicating call logic.
- `phalcom-semantic/src/db/fingerprint.rs` — include constructor visibility in data fingerprints.

### Expected tests

- `phalcom-ast/tests/data_syntax.rs`
- `phalcom-semantic/tests/semantic/adts/data_declarations.rs`
- `phalcom-semantic/tests/semantic/adts/associated/scenarios.rs`
- `phalcom-semantic/tests/semantic/incremental/adts.rs`
- `phalcom-semantic/tests/semantic/integration/access_control.rs` — underlying relation remains the oracle; likely no semantic changes required.
- recommended new file: `phalcom-semantic/tests/semantic/integration/data_constructor_visibility.rs`
- `phalcom-core/tests/core/compiler/product_optimization.rs`

### Expected documentation/state

- create `docs/implementation/LANG006/LANG006.C2/LANG006.C2-CHECKPOINT.md` if absent;
- place this plan as `docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-validated-data-construction.md`;
- update `docs/specs/objects/data-objects.md`;
- update `docs/specs/source-declarations.md` only if a short delegation sentence is required; do not duplicate data-specific rules there;
- create `LANG006.C2.P1-walkthrough.md`;
- create `LANG006.C2.P1-handoff.md`.

### Verification-only source areas; production edits are not expected

- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/associated.rs`
- `phalcom-core/src/compiler/lib/product_opt.rs`
- `phalcom-core/src/data.rs`
- `phalcom-core/src/vm/data.rs`
- `phalcom-core/src/vm/dispatch.rs`

If implementing visibility correctly appears to require modifying these runtime/compiler authority boundaries, STOP AND CONSULT before doing so.

### Unexpected-touch rule

Touching adjacent helper/test module declarations is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem not named above when that entry changes semantic ownership, constructor identity, package/project provenance, capability lifetime, or runtime representation.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change these decisions:

1. Syntax is post-name/post-generic constructor visibility:

   ```phalcom
   data Date private(...)
   data Person package { ... }
   ```

2. Pre-declaration top-level visibility remains illegal:

   ```phalcom
   private data Date(...) // still invalid
   ```

3. Visibility defaults to Public and source has no `public` keyword.
4. Supported restrictive constructor visibilities are `private`, `module`, `package`, `internal`.
5. `protected` is rejected.
6. `Visibility` remains the shared enum.
7. `DataConstructorSignature` is the canonical semantic owner of constructor visibility.
8. `DataConstructorId` does not contain visibility.
9. `is_member_accessible` remains the access-law authority.
10. package/project identity comes from canonical topology.
11. direct construction must use canonical call application rather than a parallel validator.
12. associated constructor acquisition is visibility-sensitive.
13. legally captured capabilities remain stable after escape.
14. visibility is enforced before lowering/optimization.
15. runtime data representation remains independent from constructor visibility.
16. data transparency remains unchanged.
17. re-export does not widen construction rights.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt without consultation:

- exact helper names;
- whether direct constructor authorization is expressed by:
  - making `callable_target_for_data_info` accept `ctx`, or
  - applying a small authorization wrapper immediately after target creation;
- whether `CallableApplicationTarget` gets a tiny fluent setter such as `with_access_denied`;
- parser helper decomposition;
- exact test helper names;
- whether cross-module visibility cases live in one new integration file or an existing closely matching integration module;
- diagnostic string wording, provided diagnostic category and inaccessible-vs-missing semantics remain correct;
- import ordering and Rust borrow/lifetime mechanics;
- checkpoint prose layout.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `parse_data` still owns data declaration shape parsing | `phalcom-ast/src/parser.rs` | adapt mechanically if equivalent owner moved; consult if grammar ownership changed |
| `DataDef` remains the source product | `phalcom-ast/src/ast.rs` | adapt mechanically if renamed; consult if visibility already lives elsewhere |
| direct tuple and record construction both reach `callable_target_for_data_info` / canonical call application | `checker/expression.rs` | if an alternate construction route exists, include it; consult if it bypasses semantic call products entirely |
| `CallableApplicationTarget.access_denied` remains the selected-call authorization gate | `checker/call.rs` | adapt to equivalent gate; consult if access moved to runtime-only logic |
| `CheckingContext::access_context` has exact caller module/package/project | `checker/context.rs` | use current equivalent; consult if package/project provenance is absent |
| inherent `impl Date` method bodies analyze with `Date` as the enclosing declaration for private access | `checker/body.rs` and relevant impl tests | if false, STOP AND CONSULT; do not special-case constructor factories |
| `specialize_associated_member` remains common acquisition specialization for exact/family data constructor refs | `checker/associated.rs`, `checker/expression.rs` | adapt if equivalent common seam exists; consult if exact and family paths now diverge materially |
| `data_incr_*` tests remain in incremental ADT module | semantic test tree | adapt mechanically |
| optimizer differential helper remains `run_inline_with_mode` / `ProductOptimizationMode` | core product optimization tests | adapt mechanically |
| no shipping reflection API currently hands out data-constructor capabilities independently | reflection search | if such API exists, it becomes in-scope and must consume canonical access; consult before designing new runtime metadata |

---

## 13. Global STOP / CONSULT triggers

STOP editing and prepare a compact consultation packet if any of the following occurs:

1. the post-name syntax conflicts irreducibly with current `where`/generic grammar;
2. supporting the syntax would require changing top-level export/declaration visibility semantics;
3. a data type must become class-like or inheritable for `private` to work;
4. private access inside `impl Date` does not naturally map to existing owner identity;
5. canonical package/project provenance is unavailable at constructor analysis;
6. direct tuple and record construction require materially different access architectures;
7. exact constructor references and family capture cannot share a visibility-aware semantic acquisition seam;
8. capability escape appears to require re-running lexical visibility at invocation time;
9. runtime visibility metadata appears necessary for ordinary compiled execution;
10. the product optimizer would need to know visibility to decide legality;
11. `DataConstructorId` would need to change when visibility changes;
12. a visibility edit cannot be represented in existing incremental dependency/fingerprint architecture;
13. an external import/re-export widens authority unless a new linker rule is invented;
14. the same nontrivial semantic failure persists after one serious corrective attempt;
15. two plausible fixes have materially different capability or ownership semantics;
16. an unexpected subsystem is required;
17. a workaround/special case starts emerging;
18. a duplicate source of truth would be introduced;
19. accepted spec/user semantics conflict with the live repository architecture;
20. an invariant above must be weakened to pass tests;
21. scope expands into enum visibility, reflection API design, FFI, or data opacity.

A triggered consultation cannot be self-waived.

### Consultation packet format

```text
Observed:
Verified call/data path:
Relevant symbols:
Expected invariant:
Why current architecture blocks it:
Two smallest plausible directions:
Tests/reproducer:
Decision requested:
```

Do not send an adviser the entire repository history.

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
→ identify concrete cause
→ make one coherent mechanical correction
→ rerun the smallest discriminating command
```

Examples:

- missing `Visibility` import;
- `DataDef` constructor-site compile fanout;
- new struct field missing in a test fixture;
- borrow/lifetime issue after passing `CheckingContext`;
- test module not exported from `mod.rs`.

### Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant code/fixture state changed.

---

## 15. Testing surface analysis

### Coverage obligations

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | canonical syntax | INV-01, INV-03 | default Public plus `private/module/package/internal` tuple forms |
| CV-02 | record syntax | INV-11 | `data Person private { ... }` |
| CV-03 | invalid syntax | INV-02, INV-04 | `protected` rejected; duplicate restrictive modifier rejected; pre-declaration `private data` remains rejected |
| CV-04 | semantic publication | INV-03, INV-12, INV-24 | `DataConstructorSignature.visibility` matches syntax and identity stays canonical |
| CV-05 | validated factory | INV-06 | inherent `impl Date` class-side factory may call `Date(...)` |
| CV-06 | hostile private boundary | INV-06 | another declaration in the same module cannot call `Date(...)` |
| CV-07 | module boundary | INV-07 | same exact module allowed; other module denied |
| CV-08 | package boundary | INV-08, INV-10 | sibling module in exact package allowed; nested/sibling package denied |
| CV-09 | internal boundary | INV-09 | same Project allowed; different Project denied |
| CV-10 | tuple/record parity | INV-11 | equivalent visibility outcomes for `X(...)` and `X { ... }` |
| CV-11 | exact associated acquisition | INV-14, INV-15, INV-28 | inaccessible `&Date::Date(...)` rejected as inaccessible |
| CV-12 | family acquisition | INV-16, INV-17 | inaccessible `&Date::Date...` cannot capture constructor |
| CV-13 | capability escape | INV-18 | capability acquired inside authorized owner can be returned/passed and invoked without fresh lexical access |
| CV-14 | generic specialization | INV-12, INV-13 | `Box<Int>` and `Box<String>` preserve one constructor identity/visibility law |
| CV-15 | nullary boundary | INV-21 | restricted `data Marker private()` denied outside owner, allowed inside owner |
| CV-16 | export/re-export | INV-05, INV-23 | type remains name-visible while constructor remains restricted |
| CV-17 | incremental edit | INV-24, INV-25 | Public → Private changes constructor product/dependent diagnostics; cold result agrees; IDs stable |
| CV-18 | unaffected reuse | INV-25 | unrelated callable/product retains reuse when visibility dependency does not reach it |
| CV-19 | optimizer differential | INV-21, INV-22, INV-26 | authorized private construction executes equivalently with optimizer Enabled/Disabled |
| CV-20 | representation independence | INV-19, INV-26 | components/equality/type observation unchanged; no runtime constructor visibility state |
| CV-21 | diagnostic quality | INV-28 | inaccessible constructor is not reported as missing |
| CV-22 | reflection future-proofing | INV-27 | search confirms no independent data-constructor capability acquisition route is left unchecked |

### Visibility matrix

The source-level semantic tests must prove this matrix:

| Constructor visibility | inside owning data implementation | other owner, same module | other module, same exact package | other package, same Project | other Project |
|---|---:|---:|---:|---:|---:|
| Public | allow | allow | allow | allow | allow if type is importable |
| Private | allow | deny | deny | deny | deny |
| Module | allow | allow | deny | deny | deny |
| Package | allow | allow | allow | deny | deny |
| Internal | allow | allow | allow | allow | deny |

The adjacent deny boundaries are mandatory hostile cases. Do not only test positive cases.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Goal: implement the planned architecture with sparse, discriminating feedback.

Allowed by default:

- exact new parser tests;
- exact new semantic regression;
- `cargo check -p phalcom-semantic` after structural semantic field fanout;
- exact associated-reference test after acquisition enforcement.

Do **not** run full crates after each task.

#### STABILIZE MODE

Run:

- complete `data_syntax` integration target;
- focused data declaration semantic module;
- focused constructor-visibility integration module;
- focused associated data-constructor scenarios;
- focused incremental `data_incr_` filters;
- exact core optimizer differential;
- owning package checks.

#### CERTIFY MODE

Only when checkpoint/release policy or user explicitly requires it:

- affected whole semantic/core integration targets;
- workspace check;
- workspace tests;
- workspace clippy/format.

Known broad failures inherited from C1 records must be classified rather than opportunistically repaired.

### Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning semantic/parser module suite
T3 — adjacent semantic→compiler integration
T4 — affected crate/package verification
T5 — workspace/release verification
```

### Mandatory during BUILD

- parser exact tests when T1 lands;
- `cargo check -p phalcom-semantic` once the semantic constructor signature changes;
- one exact direct-access regression while T3 is being implemented;
- one exact associated-reference regression while T4 is being implemented.

### Mandatory during STABILIZE

```sh
cargo test -p phalcom-ast --test data_syntax
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
cargo test -p phalcom-semantic --test semantic adts::associated::scenarios::data_associated_
cargo test -p phalcom-semantic --test semantic incremental::adts::data_incr_
cargo test -p phalcom-core --test core compiler::product_optimization
cargo check -p phalcom-semantic
cargo check -p phalcom-core
git diff --check
```

If the final test module name differs mechanically, use the live equivalent and record it.

### Only if evidence demands

- full `phalcom-semantic --test semantic`;
- full `phalcom-core --test core`;
- source-index/LSP tests, if constructor visibility becomes a presented/editor fact;
- reflection tests, if a live data-constructor capability API is found;
- full product optimizer lib tests if the integration differential exposes optimizer-specific behavior.

### Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace ...
cargo fmt --all -- --check
full language corpus
full LSP suite
unrelated concurrency/runtime suites
```

They add little discriminating information while the feature is incomplete and the predecessor checkpoint already documents broad baseline noise.

---

## 17. Baseline / unrelated failure policy

Classify every unexpected failure:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

Rules:

- A/B: active responsibility.
- C: perform one bounded classification pass. If nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not weaken assertions to make broad suites green.
- Do not change iterator, concurrency, native bootstrap, unrelated ADT matching, or other baseline behavior unless evidence directly connects it to constructor visibility.
- Do not consume premium adviser compute on D unless it blocks acceptance.

The `LANG006.C1.P2` records document broad semantic/core baseline failures. Treat those records as historical comparison evidence, not proof that the current checkout still has identical counts.

---

# 18. Tasks

## T0 — Establish LANG006.C2 state and perform a bounded drift check

### Purpose

Create the durable checkpoint context for this feature and verify that the live checkout still matches the architectural premises of this plan before code is changed.

### Risk

- Semantic: LOW
- Implementation fanout: documentation/state only

### Preconditions

- current checkout is available;
- C1 access/provenance implementation exists.

### Consumes

- `LANG006.C1.P2` stable access interfaces;
- this plan.

### Produces

- active `LANG006.C2` checkpoint record;
- actual local revision recorded;
- verified source/test anchors.

### Files and symbols

**Read:**

- `AGENTS.md`
- all required reads in §4

**Create if absent:**

- `docs/implementation/LANG006/LANG006.C2/LANG006.C2-CHECKPOINT.md`

**Place:**

- this plan at `docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-validated-data-construction.md`

### Required implementation shape

The checkpoint record must state:

```text
objective:
    restricted primary data construction without changing data transparency

active plan:
    LANG006.C2.P1

depends on:
    C1 canonical access/provenance
    LANG005 first-class data constructor/runtime pipeline

source-of-truth:
    DataConstructorSignature.visibility
    is_member_accessible
```

Record the actual local revision and whether the working tree already contains relevant edits.

Run a bounded symbol check:

```sh
rg -n 'struct DataDef|fn parse_data' phalcom-ast/src
rg -n 'struct DataConstructorSignature|struct DataInfo' phalcom-semantic/src
rg -n 'is_member_accessible|fn access_context|dispatch_is_accessible' phalcom-semantic/src
rg -n 'callable_target_for_data_info|AssociatedMemberId::DataConstructor' phalcom-semantic/src/checker
rg -n 'data_declaration_(input|product)_fingerprint' phalcom-semantic/src/db
```

### Forbidden approaches

- no broad repository audit;
- no code edits yet;
- no redesign of C1 access.

### Tests to run now

None.

> **Do not run runtime/integration tests after this task. No implementation exists yet.**

### Acceptance

- checkpoint exists;
- revision is recorded;
- all primary symbols still have the expected responsibility;
- no architectural premise is contradicted.

### Local STOP / CONSULT triggers

- C1 canonical access helpers are absent/reverted;
- data construction no longer uses semantic products;
- associated data constructors no longer exist as first-class semantic members.

### Checkpoint update

Mark P1 active and record the local planning/implementation start revision.

---

## T1 — Specify and parse primary-constructor visibility

### Purpose

Close the source-language contract and normalize constructor visibility into `DataDef` without changing top-level declaration/export visibility.

### Risk

- Semantic: MEDIUM
- Implementation fanout: multi-file, parser/AST/spec

### Preconditions

- T0 complete.

### Consumes

- `Visibility`;
- existing `parse_data`;
- existing data-object spec.

### Produces

- authoritative constructor-visibility language rules;
- AST visibility fact;
- parser syntax and diagnostics.

### Owned files and symbols

**Modify:**

- `docs/specs/objects/data-objects.md`
  - `# Construction`
  - current reserved `## Primary-constructor visibility`
- `phalcom-ast/src/ast.rs`
  - `DataDef`
- `phalcom-ast/src/parser.rs`
  - `parse_data`
  - a new focused parser helper if appropriate
- `phalcom-ast/tests/data_syntax.rs`

**Inspect but avoid changing unless delegation text is necessary:**

- `docs/specs/source-declarations.md`

### Source of truth

- language semantics: `docs/specs/objects/data-objects.md`;
- normalized source visibility category: `phalcom_common::modifier::Visibility`.

### Current implementation

`DataDef` currently has:

```rust
pub struct DataDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub shape: DataShapeSyntax,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

`parse_data` parses generics and optional `where` before choosing tuple/record shape.

Top-level modifier parsing already rejects:

```phalcom
private data Date(...)
```

That behavior must remain.

### Target implementation

STRUCTURAL, expected shape:

```rust
pub struct DataDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,

    pub constructor_visibility: Visibility,
    pub constructor_visibility_range: Option<SourceRange>,

    pub shape: DataShapeSyntax,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

Do not place source range inside semantic identity. It is syntax/tooling metadata only.

Grammar:

```text
data-declaration :=
    "data"
    IDENT
    generic-parameters?
    constructor-visibility?
    where-clause?
    data-shape
    where-clause?

constructor-visibility :=
    "private"
  | "module"
  | "package"
  | "internal"
```

The visibility slot occurs **after generic parameters and before any pre-shape `where` clause or shape**.

Preserve both currently accepted `where` positions.

Examples that must parse:

```phalcom
data Point(x: Int, y: Int)

data Date private(year: Int, month: Month, day: Int)

data Box<T> private(_ value: T) where T <: Object

data Box<T> private where T <: Object (_ value: T)

data Person package {
    name: String,
    age: Int
}
```

`protected` at this slot must produce a targeted error. Do not silently normalize it to Private.

### Edit operations

1. OPEN `phalcom-ast/src/ast.rs`.
2. ADD `constructor_visibility` and its optional source range to `DataDef`.
3. OPEN `phalcom-ast/src/parser.rs`.
4. FIND `parse_data`.
5. After generic parameter parsing and newline normalization, parse zero or one data-constructor visibility modifier.
6. Preserve existing optional pre-shape `where`.
7. Reject `protected` with a data-constructor-specific legality diagnostic.
8. Detect a second visibility token and emit the existing-style duplicate-visibility diagnostic rather than falling into “expected `(` or `{`”.
9. Do not route post-name data visibility through the generic top-level declaration-modifier path if that would admit `native`/`foreign` or conflate declaration visibility.
10. Update `DataDef` construction.
11. Extend `data_syntax.rs`.
12. Update the owning data spec:
    - move primary-constructor visibility out of “reserved” status;
    - define default Public;
    - define private/module/package/internal scopes by delegation to `source-declarations.md`;
    - state `protected` is invalid;
    - state declaration export and constructor visibility are orthogonal;
    - state components remain transparent.

### Must not

- accept `public`;
- accept `protected`;
- change `private data X` top-level behavior;
- derive visibility from attributes;
- remove either existing `where` placement;
- make record syntax use a different rule.

### Test changes required

CV-01, CV-02, CV-03.

Minimum AST tests:

```text
data_constructor_visibility_defaults_public
data_constructor_visibility_private_tuple
data_constructor_visibility_module_tuple
data_constructor_visibility_package_record
data_constructor_visibility_internal_generic_where
data_constructor_visibility_rejects_protected
data_constructor_visibility_rejects_duplicate_modifier
top_level_visibility_before_data_remains_illegal
```

Assert normalized enum values, not only parse success.

### Tests to run now

During BUILD, use exact filters while editing. At task completion:

```sh
cargo test -p phalcom-ast --test data_syntax
```

Proves:

- syntax formation;
- AST normalization;
- parser diagnostics.

Does not prove semantic access.

### Acceptance

- all supported source forms parse;
- AST default is Public;
- invalid forms fail as specified;
- existing data syntax remains compatible;
- authoritative data spec states the new rule.

### Local STOP / CONSULT triggers

- preserving the pre-shape `where` form makes post-name visibility ambiguous;
- top-level modifier parsing must be globally redesigned;
- `Visibility` cannot be referenced from `phalcom-ast` without a new dependency cycle.

### Checkpoint update

Record the final grammar and AST fields as durable interfaces.

---

## T2 — Publish constructor visibility in semantic data products and fingerprints

### Purpose

Make primary-constructor visibility a canonical semantic fact and incremental input without changing constructor identity.

### Risk

- Semantic: MEDIUM
- Implementation fanout: semantic multi-file

### Preconditions

- T1 AST fields exist.

### Consumes

- `DataDef.constructor_visibility`.

### Produces

- `DataConstructorSignature.visibility`;
- visibility-sensitive data declaration fingerprints.

### Owned files and symbols

**Modify:**

- `phalcom-semantic/src/data_semantics.rs`
  - `DataConstructorSignature`
- `phalcom-semantic/src/checker/data_declaration.rs`
  - `build_data_semantics`
- `phalcom-semantic/src/db/fingerprint.rs`
  - `data_declaration_input_fingerprint`
  - `data_declaration_product_fingerprint`
- `phalcom-semantic/tests/semantic/adts/data_declarations.rs`

**Update mechanically as compiler errors identify them:**

- direct `DataDef` / `DataConstructorSignature` struct literals in tests or internal fixtures.

### Source of truth

`DataConstructorSignature.visibility`.

`DataInfo` owns the constructor signature but the visibility belongs specifically to the constructor, not the entire data declaration.

### Target implementation

EXACT/near-exact semantic shape:

```rust
use phalcom_common::modifier::Visibility;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorSignature {
    pub constructor: DataConstructorId,
    pub visibility: Visibility,
    pub parameters: Box<[DataConstructorParameter]>,
    pub result_type_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}
```

In `build_data_semantics`:

```rust
let constructor = DataConstructorSignature {
    constructor: DataConstructorId::new(owner.clone()),
    visibility: data_def.constructor_visibility,
    parameters: constructor_params.into_boxed_slice(),
    result_type_template,
    source: Some(...),
};
```

In both canonical data fingerprints, hash:

```rust
product.info.constructor.visibility.hash(&mut hasher);
```

Do **not** hash the visibility source range.

### Edit operations

1. ADD the semantic field.
2. PROJECT the AST value in `build_data_semantics`.
3. UPDATE Rust struct-literal fanout mechanically.
4. ADD visibility assertions to data declaration semantic tests.
5. ADD constructor visibility to both data input and product fingerprints.
6. SEARCH `DataConstructorSignature {` and update all complete literals.
7. Do not change `DataConstructorId`.

### Forbidden approaches

- no `DataInfo.visibility` as a duplicate fact;
- no visibility in `DataConstructorId`;
- no source-range hashing;
- no associated-surface copy of visibility as a second authority;
- no runtime projection yet.

### Test changes required

CV-04 and the product side of CV-17.

At minimum prove:

```text
public data -> Public
private data -> Private
module/package/internal values survive semantic construction
constructor ID for otherwise identical visibility variants uses same owner-derived identity rule
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo check -p phalcom-semantic
```

The compile check is useful here because adding a semantic struct field can have broad Rust constructor fanout.

It does **not** prove access enforcement.

### Tests explicitly deferred

- cross-module visibility;
- associated references;
- runtime/optimizer.

### Acceptance

- visibility is present in the canonical semantic constructor product;
- all semantic constructors compile;
- data fingerprint changes when visibility changes;
- constructor identity shape is unchanged.

### Local STOP / CONSULT triggers

- downstream code requires visibility to become part of constructor identity;
- a second constructor metadata product already claims canonical visibility ownership.

### Checkpoint update

Record `DataConstructorSignature.visibility` as the canonical data-constructor access fact.

---

# G1 — Surface and semantic-product gate

Purpose: prove the feature exists as a normalized language/semantic fact before authorization work begins.

Run:

```sh
cargo test -p phalcom-ast --test data_syntax
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo check -p phalcom-semantic
```

Required result:

```text
PASS
```

Proves:

- syntax/AST;
- semantic publication;
- Rust fanout completeness.

Does not prove access.

Do not broaden after PASS.

---

## T3 — Reuse canonical access control for direct data construction

### Purpose

Make ordinary tuple- and record-shaped primary construction obey the exact same owner/module/package/project access relation as other declaration-backed members.

### Risk

- Semantic: HIGH
- Implementation fanout: semantic checker + integration tests

### Preconditions

- G1 PASS.
- C1 `is_member_accessible` and caller provenance remain intact.

### Consumes

- `DataConstructorSignature.visibility`;
- `CheckingContext::access_context`;
- `ModuleTopology::owning_package`;
- `CallableApplicationTarget.access_denied`;
- `apply_resolved_callable`.

### Produces

- one canonical checker helper for declaration-backed access;
- direct-constructor authorization;
- source-level visibility matrix tests;
- validated-factory behavior.

### Owned files and symbols

**Modify:**

- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/call.rs` only if diagnostic tailoring is needed
- `phalcom-semantic/tests/semantic/adts/data_declarations.rs`
- recommended create:
  - `phalcom-semantic/tests/semantic/integration/data_constructor_visibility.rs`
  - register it in the current integration test module tree

**Inspect:**

- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/tests/semantic/integration/access_control.rs`

### Source of truth

```text
visibility:
    DataConstructorSignature.visibility

authorization law:
    phalcom_semantic::access::is_member_accessible

package identity:
    ModuleTopology::owning_package

caller:
    CheckingContext::access_context
```

### Required implementation shape

Factor or expose a small reusable checker helper rather than reproducing access logic.

STRUCTURAL:

```rust
impl CheckingContext<'_> {
    pub(crate) fn declaration_member_is_accessible(
        &self,
        owner: &DeclarationId,
        visibility: Visibility,
    ) -> bool {
        let declaration_package = self
            .access_topology
            .map(|topology| topology.owning_package(&owner.module));

        is_member_accessible(
            visibility,
            owner,
            declaration_package.as_ref(),
            &self.access_context(),
            &self.hierarchy,
        )
    }

    pub(crate) fn data_constructor_is_accessible(
        &self,
        info: &DataInfo,
    ) -> bool {
        self.declaration_member_is_accessible(
            &info.owner,
            info.constructor.visibility,
        )
    }
}
```

Then make existing ordinary dispatch use the generic helper where practical, so constructor and method access cannot drift.

Do not duplicate this match:

```rust
match visibility {
    Visibility::Private => ...
    Visibility::Module => ...
    ...
}
```

anywhere in data-constructor code.

For direct data construction, authorize the target before `apply_resolved_callable`.

Mechanically acceptable forms include:

```rust
let mut target = callable_target_for_data_info(&data_info);
target.access_denied = !ctx.data_constructor_is_accessible(&data_info);
```

or making the helper itself context-aware:

```rust
let target = callable_target_for_data_info(ctx, &data_info);
```

The architectural requirement is that **every direct tuple/record data target is authorized through the same helper before canonical call application**.

### Direct call sites known at planning revision

`phalcom-semantic/src/checker/expression.rs` currently has:

1. record construction in `synthesize_record_construction`;
2. nominal/unqualified tuple-style construction in the “Constructor or nominal reference” branch.

Update both.

### Private factory law

Before relying on private factory behavior, verify `checker/body.rs` gives an inherent `impl Date` method `Date` as its enclosing declaration/current owner.

Then add a source regression:

```phalcom
data Date private(
    year: Int,
    month: Int,
    day: Int
)

impl Date {
    @class
    new(year: Int, month: Int, day: Int) -> Date {
        Date(year, month, day)
    }
}
```

This must be legal.

Hostile same-module case:

```phalcom
data Date private(
    year: Int,
    month: Int,
    day: Int
)

class Other {
    @class
    make() -> Date {
        Date(2026, 9, 19)
    }
}
```

This must be inaccessible despite sharing the module.

### Diagnostics

Reuse `DiagnosticCode::MemberInaccessible` unless the live diagnostic architecture has a more specific existing access code.

For a data target, prefer a message such as:

```text
primary constructor of `Date` is not accessible from this declaration
```

If visibility is useful and available:

```text
primary constructor of `Date` is private and is not accessible from this declaration
```

Do not report `AssociatedMemberMissing` or unresolved type for an existing but inaccessible direct constructor.

A small diagnostic branch inside `apply_resolved_callable` based on `InvocationTargetId::DataConstructor` is acceptable if it avoids a second call-application path.

### Cross-module/project tests

Use real semantic workspace/module topology, not direct unit calls to `is_member_accessible`, because C1 already tests the relation itself. C2 must prove data construction **consumes** it correctly.

The new integration module must cover the matrix in §15.

Use canonical `ModuleId`, `ResolvedProjectId`/`ProjectIdentity`, package roots, and module topology fixtures. Do not simulate package equality by matching paths in the test oracle.

### Forbidden approaches

- no custom data visibility `match`;
- no runtime authorization;
- no “private means same module” shortcut;
- no `module.path.starts_with(...)` package logic;
- no export-based widening;
- no changing `current_class` solely for data constructors;
- no suppressing construction resolution so the error becomes “missing”.

### Test changes required

CV-05 through CV-10 and CV-21.

### Tests to run now

While implementing, use the exact failing visibility case.

At task completion:

```sh
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
```

Do not run the full semantic suite.

### Acceptance

- validated factory passes;
- same-module non-owner private hostile case fails;
- module/package/internal matrix passes;
- record and tuple parity passes;
- inaccessible constructor remains distinguishable from missing.

### Local STOP / CONSULT triggers

- private factory requires a special “data factory privilege” rather than normal owner identity;
- package/project tests require new identity rules;
- a direct construction path bypasses `apply_resolved_callable` and cannot be adapted locally.

### Checkpoint update

Record the canonical constructor authorization helper and the direct-construction call sites.

---

# G2 — Direct authorization gate

Run:

```sh
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
cargo test -p phalcom-semantic --test semantic adts::data_declarations
```

Required evidence:

- Public default works.
- Private exact owner works.
- Same-module hostile Private fails.
- Module exact module works.
- Package exact package works; nested/sibling package fails.
- Internal same project works; other project fails.
- Tuple/record parity.
- Diagnostic is inaccessible, not missing.

If G2 fails semantically after one serious corrective attempt: STOP AND CONSULT.

---

## T4 — Close associated constructor reference and family capability bypasses

### Purpose

Ensure constructor visibility applies to capability acquisition, not only direct syntax.

### Risk

- Semantic: HIGH
- Implementation fanout: associated resolution / callable reference semantics

### Preconditions

- G2 PASS.

### Consumes

- `data_constructor_is_accessible`;
- `specialize_associated_member`;
- current exact/family constructor reference semantics.

### Produces

- visibility-aware associated data constructor acquisition;
- exact/family no-bypass tests;
- stable authorized capability escape semantics.

### Owned files and symbols

**Modify:**

- `phalcom-semantic/src/checker/associated.rs`
  - `specialize_associated_member`
  - `AssociatedMemberId::DataConstructor` arm
- `phalcom-semantic/src/checker/expression.rs`
  - only where necessary to preserve failure classification in exact/family reference paths
  - `associated_constructor_application_target`
  - `synthesize_associated_callable_reference`
  - family application recovery
- `phalcom-semantic/tests/semantic/adts/associated/scenarios.rs`

**Inspect:**

- `phalcom-semantic/src/types/denotation.rs`
- current family application/capture products

### Source of truth

The same `DataConstructorSignature.visibility` + `CheckingContext` access helper as direct construction.

### Current implementation

At the planning revision, the `DataConstructor` arm in `specialize_associated_member`:

1. obtains `DataInfo`;
2. specializes owner generic environment;
3. builds constructor callable type;
4. returns:

```rust
SpecializedAssociatedMember {
    member: member_id.clone(),
    operation,
    value_type: callable_ty,
    target: Some(InvocationTargetId::DataConstructor(dc.clone())),
}
```

with no visibility check.

`data_associated_exact_callable_reference_resolves_data_constructor` proves:

```phalcom
&Point::Point(_, _)
```

and `data_associated_whole_family_capture_publishes_data_constructor` proves:

```phalcom
&Point::Point...
```

Therefore direct-call-only enforcement is insufficient.

### Target implementation

Before an associated data constructor becomes a `SpecializedAssociatedMember` capability:

```text
lookup owner/data info
→ authorize data constructor in current acquisition context
→ if denied: emit MemberInaccessible and fail closed
→ if allowed: specialize and publish target
```

STRUCTURAL:

```rust
AssociatedMemberId::DataConstructor(dc) => {
    let Some(info) = ctx.data_info(&dc.owner).cloned() else {
        // existing missing behavior
    };

    if !ctx.data_constructor_is_accessible(&info) {
        ctx.emit_diagnostic(... DiagnosticCode::MemberInaccessible ...);
        return Err(AssociatedResolutionError);
    }

    // existing specialization unchanged
}
```

The exact diagnostic plumbing may differ.

### Critical family behavior

For:

```phalcom
&Date::Date...
```

outside `Date`:

- do not publish a family containing the private constructor;
- do not allow the family value to “remember” an inaccessible target for runtime rejection;
- do not silently convert the result into an empty-but-valid family if language rules treat this as explicit inaccessible acquisition;
- preserve inaccessible-vs-missing distinction.

For an authorized owner that captures and returns the family/callable:

```text
inside Date:
    acquire private constructor capability
    return/pass capability

outside Date:
    invoke already-acquired capability
```

the external invocation must not re-resolve `Date`'s private constructor by name and must not apply a fresh lexical private check.

Use the already captured denotation/target.

### Dynamic/rest family rule

If the family application path supports dynamic argument packs:

```text
captured member set
    ↓
shape selection
    ↓
invoke selected frozen target
```

It must never re-query the live associated surface and accidentally introduce an inaccessible constructor after capture.

### Error-category caution

`AssociatedResolutionError` is currently a concrete post-diagnostic error without a detailed enum category. Do not redesign it solely for this feature unless necessary.

If existing family code rewrites every specialization failure into `AssociatedMemberMissing`, minimally preserve the emitted `MemberInaccessible` cause/status rather than introducing a large associated-error taxonomy.

If doing that cleanly requires material redesign: STOP AND CONSULT.

### Forbidden approaches

- no visibility check only in `associated_constructor_application_target` if exact/family reference acquisition can bypass it;
- no runtime filtering of captured family members;
- no family-by-name rediscovery;
- no reauthorization of legitimately captured capabilities at arbitrary call sites;
- no global associated visibility redesign.

### Test changes required

CV-11, CV-12, CV-13, CV-17.

Extend existing data-associated scenarios with:

```text
data_private_associated_exact_reference_is_inaccessible_outside_owner
data_private_associated_family_reference_is_inaccessible_outside_owner
data_private_associated_exact_reference_is_acquirable_inside_owner
data_private_constructor_capability_can_escape_authorized_owner
data_private_family_capability_uses_frozen_authorized_candidate_set
```

Use actual Phalcom callable-reference syntax already present in the file:

```phalcom
&Point::Point(_, _)
&Point::Point...
```

Do not invent a new syntax.

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic adts::associated::scenarios::data_associated_
```

If the new test names do not share that prefix, use exact filters.

### Tests explicitly deferred

- full ADT associated suite;
- runtime associated-family suite unless semantic lowering fails.

### Acceptance

- no exact/family acquisition bypass;
- authorized acquisition still works;
- escaped capability does not cause fresh lexical resolution;
- dynamic family application stays within frozen candidate set.

### Local STOP / CONSULT triggers

- capability invocation currently reconstructs associated targets from names instead of denotation;
- blocking acquisition breaks the established “stable capability” model;
- a runtime-only family mechanism must be redesigned.

### Checkpoint update

Record acquisition-time authorization and stable capability escape as durable semantic laws.

---

# G3 — Constructor capability closure gate

Run:

```sh
cargo test -p phalcom-semantic --test semantic adts::associated::scenarios::data_associated_
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
```

Required result: PASS.

Hostile cases must demonstrate that the easiest incorrect implementation—

```text
"protect Point(...) but leave &Point::Point... unrestricted"
```

—does not pass.

Do not broaden after PASS.

---

## T5 — Close generic, nullary, re-export, and incremental boundaries

### Purpose

Prove constructor visibility is a semantic declaration fact that survives specialization and topology while invalidating precisely when changed.

### Risk

- Semantic: HIGH
- Implementation fanout: semantic tests / incremental products

### Preconditions

- G3 PASS.

### Consumes

- canonical constructor visibility;
- current data generic specialization;
- module/import/export topology;
- data fingerprints.

### Produces

- generic visibility evidence;
- nullary evidence;
- re-export non-widening evidence;
- Public ↔ restricted incremental edit evidence;
- unaffected reuse evidence.

### Owned files and symbols

**Modify tests:**

- `phalcom-semantic/tests/semantic/adts/data_declarations.rs`
- `phalcom-semantic/tests/semantic/integration/data_constructor_visibility.rs`
- `phalcom-semantic/tests/semantic/incremental/adts.rs`

**Modify production only if a focused test proves an actual dependency bug:**

- `phalcom-semantic/src/db/fingerprint.rs`
- query dependency publication code directly connected to data declaration products

### Source of truth

- constructor identity: `DataConstructorId`;
- visibility: semantic constructor signature;
- incremental relevance: data product fingerprints + normal dependency graph.

### Generic cases

Prove a generic private constructor is still private after explicit/inferred specialization.

Example concept:

```phalcom
data Box<T> private(_ value: T)

impl Box<T> {
    @class
    make(_ value: T) -> Box<T> {
        Box(value)
    }
}
```

Do not add specialization-specific visibility copies.

Assert that `Box<Int>` and `Box<String>` do not create different constructor authority or IDs.

### Nullary case

Example:

```phalcom
data Marker private()

impl Marker {
    @class
    make() -> Marker {
        Marker()
    }
}
```

Outside owner:

```phalcom
Marker() // inaccessible
```

This must be rejected before lowering can turn it into `LoadDataSingleton`.

### Re-export case

Construct a real multi-module/project fixture where the data type's name is exportable/re-exported but its constructor remains restricted.

The test must distinguish:

```text
name resolution succeeds
constructor authorization fails
```

Do not use “unresolved name” as the expected result.

### Incremental visibility edit

Extend the existing `data_incr_*` family.

Required scenario:

```text
revision 1:
    data Token(...)
    external dependent constructs Token
    => valid

revision 2:
    data Token module(...)
    dependent is in another module
    => MemberInaccessible
```

Assertions:

1. `QueryKey::DataDeclaration(owner)` product fingerprint changes.
2. `DataConstructorId` is equal before/after.
3. root nominal identity remains equal.
4. dependent callable/body is recomputed or at least its observable semantic result changes correctly.
5. cold analysis of revision 2 agrees with incremental analysis on:
   - diagnostic code;
   - constructor identity;
   - semantic data info;
   - relevant associated surface/capability result.
6. an unrelated callable/product whose dependency graph does not consume the changed data constructor is reused where the existing DB exposes pointer/fingerprint reuse evidence.

If cross-module incremental fixture infrastructure cannot express this without significant new machinery, STOP AND CONSULT rather than replacing it with an unchanged-input “equivalence” test.

### Forbidden approaches

- no manual cache invalidation;
- no version counter specifically for visibility;
- no constructor ID churn;
- no treating re-export target as the constructor owner;
- no path-based package inference in fixtures.

### Test changes required

CV-14 through CV-18.

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic incremental::adts::data_incr_
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
cargo test -p phalcom-semantic --test semantic adts::data_declarations
```

### Acceptance

- generic, nullary, and re-export boundaries obey visibility;
- visibility edits invalidate semantic dependents;
- cold/incremental results agree;
- constructor/declaration identities remain stable;
- unrelated work is not broadly invalidated.

### Local STOP / CONSULT triggers

- visibility edits require changing stable identity;
- dependency tracking cannot observe the data product;
- re-export changes the declaration owner identity;
- current multi-module incremental fixtures cannot represent canonical package/project topology without architectural additions.

### Checkpoint update

Record the incremental fingerprint/dependency law and re-export law.

---

## T6 — Verify lowering/runtime/optimizer representation independence

### Purpose

Prove access control is completed before lowering and remains correct when data construction is materialized, singleton-loaded, or scalar-replaced.

### Risk

- Semantic: HIGH for soundness
- Implementation fanout: tests; production core edits expected to be NONE

### Preconditions

- T5 complete.

### Consumes

- authorized semantic resolutions;
- existing data lowering;
- existing product optimizer modes.

### Produces

- representation-independent execution evidence;
- confirmation that runtime values do not own constructor visibility.

### Verification-only files

Inspect:

- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/associated.rs`
- `phalcom-core/src/compiler/lib/product_opt.rs`
- `phalcom-core/src/data.rs`
- `phalcom-core/src/vm/data.rs`
- `phalcom-core/src/vm/dispatch.rs`

### Test file

Modify:

- `phalcom-core/tests/core/compiler/product_optimization.rs`

or the nearest existing data-construction runtime integration file if live organization provides a more specific owner.

### Required implementation shape

**Expected production change: none.**

Semantic analysis must either:

```text
authorize
→ publish lowering
```

or:

```text
reject
→ compiler never receives a legal data construction lowering fact
```

The compiler must not decide whether a private constructor is legal.

### Optimizer differential

Add one authorized validated-data case whose primary constructor can be optimized.

Conceptual source:

```phalcom
data Pair private(
    _ left: Int,
    _ right: Int
)

impl Pair {
    @class
    sum(_ left: Int, _ right: Int) -> Int {
        let pair = Pair(left, right)
        pair.left + pair.right
    }
}

const result = Pair.sum(20, 22)
```

Run with:

```rust
ProductOptimizationMode::Enabled
ProductOptimizationMode::Disabled
```

and assert equivalent result.

If component access spelling in the live data model differs, adapt mechanically.

Also retain a nullary private-factory execution case if the existing core harness can express it cheaply.

### Negative architecture check

After implementation, inspect the diff and run a targeted search such as:

```sh
rg -n 'is_member_accessible|MemberInaccessible|Visibility::(Private|Module|Package|Internal)' \
  phalcom-core/src/data.rs \
  phalcom-core/src/vm/data.rs \
  phalcom-core/src/compiler/lib/product_opt.rs
```

Expected:

- no newly introduced data-constructor authorization logic in these representation/optimizer owners;
- any pre-existing unrelated visibility code is documented rather than deleted.

### Associated thunk verification

Inspect `compile_data_constructor_thunk`.

It should continue to compile an already-authorized semantic target. Do not add a visibility branch to the thunk.

### Forbidden approaches

- no runtime descriptor `visibility` field for this feature;
- no access check in `ConstructData`;
- no access check in `LoadDataSingleton`;
- no optimizer guard based on visibility;
- no hidden metadata in every data value;
- no bytecode opcode change solely for visibility.

### Test changes required

CV-19, CV-20, and runtime half of CV-15.

### Tests to run now

Exact differential test first, then:

```sh
cargo test -p phalcom-core --test core compiler::product_optimization
```

If this existing module contains unrelated known failures, run the exact new test plus the directly affected subset and classify the rest.

### Acceptance

- Enabled and Disabled optimizer modes produce equivalent authorized behavior;
- nullary authorized path works;
- inaccessible source never reaches runtime as a legal compiled construction;
- no new runtime representation authority exists.

### Local STOP / CONSULT triggers

- unauthorized construction reaches executable lowering despite semantic error;
- optimizer can create a constructor operation without a semantic construction record;
- runtime must know source caller package/project to make the feature correct.

### Checkpoint update

Record “semantic authorization precedes lowering; runtime representation remains visibility-free.”

---

# G4 — Persistence and representation gate

Run:

```sh
cargo test -p phalcom-semantic --test semantic incremental::adts::data_incr_
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
cargo test -p phalcom-core --test core compiler::product_optimization
cargo check -p phalcom-core
```

Required evidence:

- visibility edit cold/incremental parity;
- constructor ID stability;
- re-export non-widening;
- nullary no bypass;
- optimizer differential parity.

Do not run workspace tests after G4 merely because it passed.

---

## T7 — Stabilize, synchronize authoritative docs/state, and close the checkpoint

### Purpose

Run the planned focused evidence once, reconcile the specification and repository records, and produce durable implementation artifacts.

### Risk

- Semantic: MEDIUM
- Implementation fanout: verification/documentation

### Preconditions

- G1–G4 PASS or any remaining failure classified under §17.

### Consumes

- complete implementation;
- focused test evidence.

### Produces

- updated `LANG006.C2-CHECKPOINT.md`;
- P1 walkthrough;
- P1 handoff;
- truthful verification status.

### Files

**Modify:**

- `docs/implementation/LANG006/LANG006.C2/LANG006.C2-CHECKPOINT.md`
- `docs/specs/objects/data-objects.md` if implementation discovered a wording correction

**Create:**

- `docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-walkthrough.md`
- `docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-handoff.md`

### Required stabilization review

Confirm each invariant has an owner and test/evidence.

Search production uses:

```sh
rg -n 'constructor_visibility|\.visibility' \
  phalcom-ast/src \
  phalcom-semantic/src
```

Inspect each data-constructor visibility use. Expected architecture:

```text
AST source fact
semantic constructor product
fingerprint
canonical access consumption
```

No parallel store.

Search direct data constructor target creation:

```sh
rg -n 'data_constructor\(|callable_target_for_data_info|InvocationTargetId::DataConstructor' \
  phalcom-semantic/src
```

Review every creation/acquisition path for authorization or pre-authorized capability semantics.

### Tests to run now

Proceed to G5.

### Acceptance

- no forgotten bypass;
- no duplicate authority;
- specs and code agree;
- checkpoint ledger is truthful;
- walkthrough and handoff exist.

### Local STOP / CONSULT triggers

- final search reveals an unplanned constructor-capability producer;
- spec wording would need to weaken an implemented invariant;
- focused tests disagree with the ratified access matrix.

### Checkpoint update

At completion set status according to actual evidence only.

---

# 19. Verification gates

## G1 — Surface and semantic product

Already defined after T2.

## G2 — Direct authorization

Already defined after T3.

## G3 — Capability acquisition closure

Already defined after T4.

## G4 — Incremental and representation independence

Already defined after T6.

## G5 — Focused checkpoint stabilization

Run once:

```sh
cargo test -p phalcom-ast --test data_syntax

cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo test -p phalcom-semantic --test semantic integration::data_constructor_visibility
cargo test -p phalcom-semantic --test semantic adts::associated::scenarios::data_associated_
cargo test -p phalcom-semantic --test semantic incremental::adts::data_incr_

cargo test -p phalcom-core --test core compiler::product_optimization

cargo check -p phalcom-semantic
cargo check -p phalcom-core

git diff --check
```

For each command record:

```text
command
result
coverage IDs proven
failures classified
```

Do not broaden automatically after G5 passes.

### Optional checkpoint-certification gate

Only if requested by the checkpoint/user:

```sh
cargo +stable fmt --all -- --check
cargo +stable check --workspace --all-targets
cargo +stable test --workspace --all-targets
cargo +stable clippy --workspace --all-targets -- -D warnings
```

These prove broad compatibility/release readiness, not the constructor-visibility semantics already established by focused tests.

Because predecessor records document baseline failures/format drift, classify any broad failure before editing.

---

## 20. Final focused acceptance

At plan completion, populate this table in the walkthrough:

| Coverage ID | Test/evidence | Result |
|---|---|---|
| CV-01 | AST constructor visibility syntax | PASS / FAIL |
| CV-02 | record visibility syntax | PASS / FAIL |
| CV-03 | invalid protected/duplicate/top-level forms | PASS / FAIL |
| CV-04 | semantic constructor product + fingerprint | PASS / FAIL |
| CV-05 | private validated factory | PASS / FAIL |
| CV-06 | same-module non-owner private hostile case | PASS / FAIL |
| CV-07 | module boundary | PASS / FAIL |
| CV-08 | package boundary | PASS / FAIL |
| CV-09 | project/internal boundary | PASS / FAIL |
| CV-10 | tuple/record parity | PASS / FAIL |
| CV-11 | exact associated acquisition | PASS / FAIL |
| CV-12 | family acquisition | PASS / FAIL |
| CV-13 | capability escape | PASS / FAIL |
| CV-14 | generic specialization | PASS / FAIL |
| CV-15 | nullary | PASS / FAIL |
| CV-16 | re-export | PASS / FAIL |
| CV-17 | visibility edit invalidation | PASS / FAIL |
| CV-18 | unaffected reuse | PASS / FAIL |
| CV-19 | optimizer differential | PASS / FAIL |
| CV-20 | representation unchanged | PASS / FAIL |
| CV-21 | inaccessible-vs-missing diagnostic | PASS / FAIL |
| CV-22 | no independent unchecked reflection acquisition | PASS / FAIL |

A required row may not be marked PASS from inspection alone when a behavioral test is specified.

---

## 21. Performance/resource evidence

This is not a performance checkpoint. Do not add benchmarks.

Resource/representation evidence is limited to proving the feature does **not** introduce:

- per-data-value visibility metadata;
- per-construction runtime provenance checks;
- optimizer disablement for restricted constructors;
- duplicate constructor descriptors keyed by visibility.

The existing optimizer differential is correctness evidence, not a benchmark.

---

## 22. Checkpoint bookkeeping

### At plan start

Update:

```text
docs/implementation/LANG006/LANG006.C2/LANG006.C2-CHECKPOINT.md
```

Record:

- actual start revision;
- working-tree relevance;
- active plan `LANG006.C2.P1`;
- initial invariant set;
- inherited C1 interfaces.

### During plan

Update only for:

- durable new interface/invariant;
- consultation/amendment;
- deferred/baseline failure;
- coherent G gate result.

Do not write a diary after every edit.

### At plan completion

Record:

- plan status;
- completion status;
- verification status;
- final invariants;
- evidence ledger;
- deferred failures;
- consultations/amendments;
- whether C2 is closed;
- next action.

Suggested truth:

```text
IMPLEMENTED + FOCUSED_TESTED
```

is valid if G1–G5 pass while unrelated broad baseline failures remain documented.

Do not claim `RELEASE_COMPLETE` without release-level evidence.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-walkthrough.md
```

It must contain:

- final result;
- final revision / working-tree state;
- grammar implemented;
- canonical semantic owner;
- canonical access helper;
- direct construction paths changed;
- associated acquisition paths changed;
- incremental/fingerprint changes;
- runtime/optimizer non-change or justified change;
- important code changes by symbol;
- deviations from this plan;
- consultations and decisions;
- tests added;
- tests actually run;
- tests deferred;
- coverage table from §20;
- baseline classifications;
- verification classification;
- residual risks.

Do not say “all tests pass” if only focused tests ran.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C2/LANG006.C2.P1-handoff.md
```

It must contain:

- current revision/state;
- stable interfaces:
  - `DataDef.constructor_visibility`;
  - `DataConstructorSignature.visibility`;
  - canonical access helper;
  - associated acquisition rule;
- visibility matrix;
- capability escape law;
- constructor identity law;
- incremental fingerprint law;
- runtime representation boundary;
- next objective;
- must-read files;
- deferred failures;
- first recommended commands;
- things not to redesign/re-explore.

If C2 is complete, explicitly state that no additional C2 implementation plan is required unless a later audit finds a specific defect.

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

Use repository metadata truthfully.

| State | Meaning |
|---|---|
| `NOT_STARTED / UNVERIFIED` | plan only |
| `PARTIAL / UNVERIFIED` | implementation incomplete or no coherent gate passed |
| `IMPLEMENTED / FOCUSED_TESTED` | planned feature implemented and G1–G5 focused evidence passed |
| `IMPLEMENTED / BASELINE_BLOCKED` | feature evidence is good but a required broader gate is blocked by classified baseline failures |
| `IMPLEMENTED / RELEASE_COMPLETE` | explicit release-level gates passed |

No checkpoint may be marked complete while a required C2 semantic gate is still an INCIDENT.

---

## 26. Plan self-review

### Architecture

- [x] visibility has one shared enum.
- [x] constructor visibility has one canonical semantic owner.
- [x] access policy reuses the C1 access relation.
- [x] package/project provenance is canonical identity.
- [x] constructor identity is independent from visibility.
- [x] direct and associated construction paths are both covered.
- [x] capability acquisition vs invocation semantics are explicit.
- [x] runtime/value representation is not made an access authority.
- [x] optimizer soundness boundary is explicit.
- [x] export visibility is orthogonal.

### Luna executability

- [x] exact repository revision recorded.
- [x] required reads bounded.
- [x] fixed/flexible/verify-first areas explicit.
- [x] production paths and symbols named.
- [x] parser/semantic code shapes supplied where evidence supports them.
- [x] objective STOP/CONSULT triggers exist.
- [x] debugging loops are bounded.
- [x] no routine task requires premium adviser involvement.

### Testing

- [x] positive and hostile private cases included.
- [x] module/package/project adjacent boundaries included.
- [x] tuple/record parity included.
- [x] exact/family capability bypasses included.
- [x] capability escape included.
- [x] generic/nullary/re-export included.
- [x] cold/incremental visibility edit included.
- [x] unaffected reuse included.
- [x] optimizer Enabled/Disabled differential included.
- [x] broad tests are deferred during BUILD.
- [x] every required gate states what it proves.

### Documentation lifecycle

- [x] authoritative `docs/specs` owner identified.
- [x] reserved visibility question is explicitly closed.
- [x] checkpoint record required.
- [x] walkthrough required.
- [x] handoff required.

### No fake precision

- [x] remote HEAD is exact.
- [x] local working tree is explicitly unknown.
- [x] runtime production edits are described as verification-only unless evidence demands them.
- [x] test-module creation is recommended where current tree lacks an exact C2 owner.
- [x] code snippets that may require local reconciliation are marked STRUCTURAL.

---

# Suggested staged commit groups

Commits are optional execution boundaries, not mandatory per-task rituals.

```text
1. feat(data): add primary-constructor visibility syntax and semantic metadata
   - T1–T2
   - G1

2. feat(semantic): enforce validated data construction access
   - T3
   - G2

3. fix(associated): enforce data constructor capability acquisition visibility
   - T4
   - G3

4. test(data): close topology incremental nullary and optimizer boundaries
   - T5–T6
   - G4

5. docs(lang006): certify validated data construction checkpoint
   - T7
   - G5
```

Do not split mechanical field-plumbing into separate commits unless it materially improves failure attribution.

---

# Final negative / architecture gates

Before closing C2:

1. Search every data-constructor capability producer:

   ```sh
   rg -n 'InvocationTargetId::DataConstructor|AssociatedMemberId::DataConstructor|data_constructor\(' \
     phalcom-semantic/src
   ```

   Review all production hits.

2. Confirm no data-specific access relation was introduced:

   ```sh
   rg -n 'Visibility::Private|Visibility::Module|Visibility::Package|Visibility::Internal' \
     phalcom-semantic/src/checker
   ```

   Any new data-specific `match` over those four values requires justification; the expected design calls `is_member_accessible`.

3. Confirm representation owners did not become visibility authorities:

   ```sh
   rg -n 'is_member_accessible|MemberInaccessible' \
     phalcom-core/src/data.rs \
     phalcom-core/src/vm/data.rs \
     phalcom-core/src/compiler/lib/product_opt.rs
   ```

4. Confirm `DataConstructorId` definition was not changed to include visibility.

5. Confirm `docs/specs/objects/data-objects.md` no longer says primary-constructor visibility is an unresolved/reserved question.

---

# Release-complete criteria

The feature implementation is checkpoint-complete only when:

- every required invariant remains true;
- G1–G5 required focused evidence passes;
- all hostile cases pass;
- no direct/associated/family bypass remains;
- visibility edit incremental evidence passes;
- optimizer differential passes;
- constructor identity remains stable;
- specs are synchronized;
- checkpoint/walkthrough/handoff are current;
- no unresolved C2 INCIDENT remains.

Release certification is a separate claim and requires the explicitly requested broader gates.

---

# First implementation action

After importing this plan into the repository:

```text
1. read T0 required files;
2. record the actual local revision in LANG006.C2-CHECKPOINT.md;
3. verify the five primary symbol groups with the T0 rg commands;
4. begin T1 in docs/specs/objects/data-objects.md + DataDef + parse_data;
5. do not run broad tests.
```
