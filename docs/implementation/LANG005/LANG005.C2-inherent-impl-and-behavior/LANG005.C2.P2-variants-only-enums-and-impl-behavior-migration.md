# LANG005.C2.P2 — Variants-Only Enum Declarations and Closed Enum Behavior Migration to `impl`

```yaml
---
id: LANG005.C2.P2
category: LANG
program: LANG005
checkpoint: LANG005.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C2.P1
follows: LANG005.C2.P1
supersedes: null
---
```

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementer working under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, identity rules, ownership boundaries, and invariants below;
- must not improvise material architecture;
- must not begin P2 source implementation until LANG005.C2.P1 is actually implemented and its takeover interfaces are verified in the live tree;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep the shared `LANG005.C2` checkpoint record current;
- must create the required walkthrough and handoff artifacts.

Testing is evidence gathering, not ritual. Do not run broad test suites merely because an edit occurred.

### Hard predecessor gate

At the repository revision used to prepare this plan, LANG005.C2.P1 exists only as a plan artifact. The live source does **not** yet contain the planned P1 `InherentImplContribution` architecture. Therefore:

> **Do not implement P2 on top of the pre-P1 tree.**

Before P2 begins, verify the implemented P1 repository contains repository-equivalent forms of all of the following:

```text
first-class Statement::Impl / ImplDef syntax
BehaviorMember or one canonical behavior-only AST category
ImplId / stable impl provenance
TypeParameterOwner::Impl or repository-equivalent stable impl generic owner
same-module inherent ownership enforcement
covering generic impl-head normalization
DeclaredSurface + InherentImplContribution + effective DeclarationSurface
callable-definition provenance resolving CallableId -> actual source body
body checking through effective impl-origin definitions
semantic-to-compiler impl lowering
compiler installation without runtime class reopening
source-index/LSP identity for impl-origin callables
incremental fingerprints/dependencies for impl contributions
```

If one of these is absent because P1 is incomplete, stop P2 and complete P1 first. If P1 landed with mechanically different names but equivalent architecture, adapt locally. If P1 materially changed any identity, ownership, effective-surface, provenance, or lowering contract assumed here, **STOP AND CONSULT** before editing P2.

---

## 1. Goal

Migrate Phalcom enums to variants-only declarations and move all closed-enum root/default/exact-case behavior into first-class `impl`, while preserving the existing closed requirement model, exact-case identity, GADT specialization, case-only member availability, source identity, incrementality, and runtime dispatch semantics.

---

## 2. Checkpoint acceptance objective

LANG005.C2 establishes the complete inherent-behavior architecture in three plans:

```text
C2.P1  first-class inherent impl + effective declaration surfaces
C2.P2  variants-only enum declarations + closed enum behavior migration
C2.P3  constrained/specialized inherent impl applicability
```

This plan is the **migration and enum-semantic integration plan** of the checkpoint. It is not full C2 closure; C2.P3 remains afterward.

P2 is accepted when:

1. canonical enum source declarations contain structural variant declarations only;
2. enum-root instance behavior is authored in `impl Enum`;
3. declaration-only enum-root instance members remain closed-enum requirements;
4. bodyful enum-root instance members remain root/default implementations;
5. exact-case behavior is authored in `impl Enum::Variant(shape)` and preserves `CallableOwnerId::Variant(VariantId)` identity;
6. exact-case implementations satisfy requirements or override defaults only after case/GADT specialization;
7. case-only behavior remains unavailable on the root enum type;
8. the existing closed enum requirement/default semantic machinery remains the semantic authority, fed from normalized impl contributions rather than behavior-bearing enum syntax;
9. compiler/runtime installation consumes semantic lowering and does not scan impl fragments or reopen runtime classes;
10. legacy behavior-bearing enum syntax is removed from the canonical AST/parser before plan completion, after differential equivalence evidence has been captured;
11. P1's ordinary inherent surface architecture remains intact and P3 conditional applicability has not been implemented prematurely.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: d8b823e857b4054420cc070cda9d6b030f151391
prepared: 2026-09-13
relevant predecessor: LANG005.C2.P1-inherent-impl-effective-surfaces-plan.md
```

### Verified repository facts at planning time

1. `docs/implementation/LANG005/LANG005.C2/` contains the C2.P1 plan but no P2 plan and no live C2 implementation record visible in that directory.
2. `InherentImplContribution` appears in the C2.P1 plan but not in live source at this revision. P1 is therefore a planned predecessor, not an implemented takeover state at this baseline.
3. `phalcom-ast/src/ast.rs` currently represents enum bodies as:

   ```rust
   enum EnumMember {
       Variant(VariantDecl),
       Behavior(EnumBehaviorMember),
   }
   ```

4. `phalcom-ast/src/parser.rs` currently classifies enum members and calls `parse_enum_behavior_member` for non-variant body entries.
5. `phalcom-semantic/src/checker/enum_behavior.rs` already owns the canonical enum behavior product:

   ```rust
   EnumBehaviorProduct {
       owner,
       root_defaults,
       root_requirements,
       case_implementations,
       diagnostics,
   }
   ```

6. `phalcom-semantic/src/enum_requirements.rs` already owns stable `EnumRequirementId`, `EnumRequirement`, `CaseRequirementResult`, `CaseRequirementStatus`, and GADT-specialized requirement checking through `VariantInfo.case_environment`.
7. `phalcom-semantic/src/checker/enum_declaration.rs` already separates structural enum semantics from behavior in important ways: it processes variants, constructs `VariantId`, exact-case templates, variant-local generic signatures, payload fields, and `CaseTypeEnvironment`.
8. `phalcom-semantic/src/session.rs` currently wires `EnumBehaviorProduct` into callable signatures, case requirement checking, and body checking, but it still discovers source behavior through enum/variant AST nesting.
9. `phalcom-semantic/src/db/query.rs`, `semantic_shard.rs`, `core_surface/source.rs`, and source-index code contain direct `EnumMember::Behavior` scans that must disappear from final P2 architecture.
10. `phalcom-core/src/compiler/lib/enum_decl.rs` currently compiles enum-root and case-local behavior by walking enum AST. That direct AST authority must be removed for migrated behavior.
11. `phalcom-core/src/modules/semantic_lowering.rs` currently keeps `EnumLoweringSpec` structural: enum owner, runtime representation, and variants. P1 plans a separate semantic impl-lowering product; P2 should extend that impl target projection rather than turning `EnumLoweringSpec` into a second behavior source of truth.
12. `phalcom-core/src/vm/adt.rs` already allocates one enum root behavior class plus one hidden behavior class per exact variant, with each case class inheriting from the root class. This is exactly the runtime topology P2 needs: root defaults install once on the root class; case implementations install on the exact case class; ordinary inheritance supplies defaults.
13. The current normative `docs/spec/adts.md` still describes behavior-bearing enum bodies and explicit `@variant` classification. That source-layout rule is stale relative to the ratified LANG005.C2.P2 update and must be reconciled as part of this plan before the implementation is considered complete.
14. `phalcom-semantic/tests/semantic/adts/declarations.rs` already contains formal closed-requirement, exact-case, and GADT tests that should be migrated/extended rather than replaced by a parallel test harness.
15. `phalcom-semantic/tests/semantic/README.md` and `phalcom-core/tests/README.md` require formal identities/proofs to be tested in semantic tests and executable behavior in core/runtime tests, with focused commands during migration.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these in order after P1 has landed:

```text
1. AGENTS.md
2. docs/implementation/README.md
3. docs/spec/README.md
4. docs/spec/adts.md
5. docs/implementation/LANG005/LANG005.C2/CHECKPOINT.md
   - if the new workflow has created it under a different canonical path, use that path
6. LANG005.C2.P1 plan
7. LANG005.C2.P1 walkthrough
8. LANG005.C2.P1 handoff
9. the current LANG005 program/semantic update source used by the project
10. relevant accepted PDR/ADR records, especially TDR-0081 for LANG005 identity/representation boundaries
11. TYPE001 ADT/GADT declaration-identity and runtime-lowering specs as historical implementation authority where still consistent with current docs/spec
12. phalcom-semantic/tests/semantic/README.md
13. phalcom-core/tests/README.md
```

Then inspect the live versions of:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/selector.rs

phalcom-semantic/src/identity.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/checker/enum_behavior.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/types/case_environment.rs
phalcom-semantic/src/types/case_instantiation.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs

P1's landed impl/effective-surface/provenance modules

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/enum_decl.rs
P1's landed impl compiler/lowering module
phalcom-core/src/vm/adt.rs
phalcom-core/src/native/source.rs

phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
current P1 impl/effective-surface tests
phalcom-core/tests/core/language/algebraic_data.rs or live repository-equivalent
```

Avoid broad exploration unless targeted evidence reveals another owner.

---

## 5. Normative authority

### 5.1 Desired language behavior — FIXED

The ratified LANG005.C2.P2 behavior is:

```phalcom
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}

impl Expression {
  evaluate -> Int

  precedence -> Int {
    100
  }
}

impl Expression::Literal(_) {
  evaluate -> Int {
    value
  }
}

impl Expression::Add(_, _) {
  evaluate -> Int {
    left.evaluate + right.evaluate
  }

  precedence -> Int {
    10
  }
}
```

Interpretation:

```text
Expression.evaluate
    closed enum root requirement

Expression.precedence
    root/default implementation

Expression::Literal.evaluate
    exact-case implementation / requirement witness

Expression::Add.evaluate
    exact-case implementation / requirement witness

Expression::Add.precedence
    exact-case override of root default
```

### 5.2 Existing normative semantics to preserve

Current ADT semantics remain authoritative for:

- enum closedness;
- exact `VariantId` and selector-family identity;
- singleton variant versus zero-argument constructor distinction;
- associated constructor identity versus ordinary method identity;
- exact-case types;
- GADT result specialization;
- variant-local generics;
- payload-field semantic identity;
- construction visibility not affecting closed-case completeness;
- case-local behavior being instance-side;
- requirement compatibility after exact-case specialization;
- representation not being semantics.

### 5.3 Normative source-layout update required by P2

The current `docs/spec/adts.md` still places behavior and requirements inside enum bodies. P2 changes that canonical source model. Update the normative spec as part of this plan so the live specification and implementation do not disagree.

Canonical P2 syntax treats an enum body as a list of variants. The `@variant` marker must no longer be required to distinguish variants from behavior because behavior is no longer legal in the enum body. For migration safety, the parser may continue accepting legacy `@variant` as a compatibility attribute until LANG005.C8, but it must normalize to the same `VariantDecl` and must not remain the semantic discriminator for “this body member is a variant.” New normative examples use bare variants.

### 5.4 P1 architecture — required implementation authority

P1 owns:

- `impl` source fragments;
- `ImplId` provenance;
- same-module inherent ownership;
- impl-owned generic binders;
- covering generic-head normalization;
- behavior-only members;
- declared versus effective declaration surfaces;
- callable-definition provenance;
- semantic-to-compiler impl lowering;
- non-reopening runtime installation.

P2 extends these mechanisms to enum contracts and exact-case targets; it does not create a parallel impl system.

### 5.5 Future/non-normative work

The following are later and must not alter P2 design:

```text
C2.P3 constrained/specialized ordinary inherent impls
C3 trait declarations
C4 trait conformance/witnesses
C5 associated types
C6 generic trait evidence
C7 public reflection API
C8 final legacy/derived-behavior policy
C9 program-wide performance/soundness closure
```

Closed enum requirements are not traits. Exact enum-case impls are not ordinary P3 specializations.

---

## 6. Takeover state

### 6.1 Stable pre-P1 enum interfaces verified at planning baseline

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Structural enum product | `build_enum_semantics`, `EnumDeclarationProduct` — `checker/enum_declaration.rs` | semantic | Variant structure/GADT facts are independent of behavior source layout. |
| Exact case identity | `VariantId`, `TypeData::ExactCase` | semantic type system | Exact case identity is not runtime discriminant or constructor identity. |
| Case specialization | `VariantInfo.case_environment`, `CaseTypeEnvironment` | semantic type system | GADT declaration equations are canonical specialization evidence. |
| Variant-local generics | variant constructor `GenericSignature`, callable-owned params | semantic type system | Constructor-local generic identity remains stable and callable-owned. |
| Root requirement identity | `EnumRequirementId` | semantic | Requirement identity is separate from any witness callable. |
| Requirement result | `CaseRequirementResult` / `CaseRequirementStatus` | semantic | Requirement satisfaction is explicit semantic evidence. |
| Behavior aggregate | `EnumBehaviorProduct` | semantic checker | Owns root defaults, root requirements, exact-case implementations. |
| Runtime enum topology | root class + `VariantId -> ClassId` hidden case classes — `vm/adt.rs` | runtime | Case class inherits root; physical class identity is not semantic case identity. |
| Structural lowering | `EnumLoweringSpec` | semantic-to-core projection | Contains runtime representation/variant facts, not source-level behavioral authority. |

### 6.2 Required P1 takeover interfaces — VERIFY FIRST

The exact landed names may differ. Verify repository-equivalent forms exist.

| Concept | Planned P1 shape | Required P2 invariant |
|---|---|---|
| Impl provenance | `ImplId` | Source fragment identity is not callable identity. |
| Behavior syntax | `BehaviorMember` | One canonical behavior-only AST category. |
| Impl target resolution | nominal `DeclarationId` + covering map | Semantic target resolution precedes consumers. |
| Impl generic owner | `TypeParameterOwner::Impl(ImplId)` | Impl source binders have stable identities and normalize before publication. |
| Contribution | `InherentImplContribution` | Semantics, not source order, build effective behavior. |
| Effective root surface | final `DeclarationSurface` | Ordinary lookup consumes checked effective root behavior. |
| Definition provenance | `CallableDefinitionOrigin::InherentImpl` or equivalent | `CallableId` resolves to correct impl body/source. |
| Compiler projection | `InherentImplLoweringSpec` or equivalent | Compiler receives semantic target/member facts and does not perform target lookup itself. |
| Incremental owner | impl/surface query products | Signature edits and body edits have appropriately narrow dependencies. |

### 6.3 Takeover stop condition

If P1 did not establish callable-definition provenance independently of surface membership, or if compiler lowering still discovers an impl target by source name/runtime lookup, P2 must not compensate locally. **STOP AND CONSULT.** P2 depends on those being solved correctly at P1.

---

## 7. Architecture

### 7.1 Direction of authority

```text
source enum declaration
    ↓
structural variant syntax only
    ↓
EnumDeclarationProduct / VariantInfo / CaseTypeEnvironment

source impl fragments
    ↓
P1 ImplId + target resolution + member signatures/provenance
    ↓
normalized impl contributions
    ├───────────────┐
    │               │
    │ nominal root  │ exact enum case
    v               v
P1 effective       EnumBehaviorProduct
root surface       root_defaults
                   root_requirements
                   case_implementations
                         ↓
                   requirement/default/override validation
                         ↓
                   exact-case lookup/body facts

semantic facts
    ↓ projected as stable lowering facts
compiler
    ↓ installs behavior on already-resolved runtime targets
VM/runtime
```

The semantic layer remains authoritative. Neither compiler nor VM may reconstruct whether a member is a requirement, default, witness, override, or case-only member from syntax or runtime class shape.

### 7.2 Impl target model

P2 extends the P1 semantic target domain from nominal declarations to exact enum cases.

Conceptually:

```rust
pub enum InherentImplTarget {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}
```

Use the repository's actual P1 target product/name if different. Do not encode an exact-case target as:

- a fake `DeclarationId`;
- a synthetic subclass declaration;
- an associated callable reference;
- a runtime `ClassId`;
- an ordinary `TypeId` specialization selected by P3 applicability;
- a string such as `"Expression::Literal"`.

An exact-case impl is identified by canonical `VariantId`.

### 7.3 Exact-case source target syntax

Canonical shapes:

```phalcom
impl Option::None {
  ...
}

impl Example::Empty() {
  ...
}

impl<T> Option<T>::Some(_) {
  ...
}

impl Result::Error(reason: _) {
  ...
}
```

The case target uses the existing variant selector shape and therefore preserves:

```text
getter/singleton versus callable constructor
positional arity
labels
zero-argument constructor shape
base family identity
```

The `_` placeholders describe selector shape; they do **not** introduce lexical pattern bindings. Payload names available in the body come from the canonical `VariantFieldId`/field semantics of the targeted variant.

Associated callable-reference syntax remains separate:

```phalcom
&Option::Some(_)
```

That expression is not an impl target.

### 7.4 Covering generic exact-case heads

P2 remains unconditional. Exact-case impl heads may be generic only when they cover the entire targeted case domain.

For an exact case whose enclosing enum has declaration-owned parameters and whose variant constructor has variant-local callable-owned parameters, target-head resolution must build explicit normalization maps from impl-owned binders into both canonical domains.

Conceptually:

```text
impl-owned binders
    ↓ covering map
[enum declaration params] + [variant constructor-local params]
```

A legal generic exact-case impl must:

1. resolve one exact `VariantId`;
2. cover every generic parameter exposed by the targeted exact case exactly once;
3. use no concrete specialization in the head;
4. repeat no generic binder;
5. omit no required binder;
6. add no applicability `where` condition beyond constraints already intrinsic to the enum/variant declaration;
7. normalize published case-member signatures out of `ImplId`-owned parameter space into the canonical declaration/variant parameter spaces.

Illustrative form where the variant has its own `U` binder:

```phalcom
impl<T, U> Boxed<T>::Pack<U>(_) {
  ...
}
```

If the live grammar already spells generic variant targets differently, use that established spelling. If supporting variant-local generic exact-case targets would require inventing a new public generic-argument syntax not already ratified or clearly implied by existing generic variant syntax, **STOP AND CONSULT** rather than choosing a new language design.

Concrete/repeated/non-covering heads remain C2.P3 and must still reject:

```phalcom
impl Option<Int>::Some(_) { ... }
impl<T> Pair<T, T>::Left(_) { ... }
```

### 7.5 Enum behavior normalization

Refactor `EnumBehaviorProduct` so it no longer builds directly from behavior nested in `EnumDef`.

Target conceptual flow:

```text
InherentImplSet(enum root)
+ exact-case impl contributions for VariantIds owned by enum
+ EnumDeclarationProduct
        ↓
build_enum_behavior(...)
        ↓
EnumBehaviorProduct
```

The final product should continue to expose repository-equivalent forms of:

```rust
pub struct EnumBehaviorProduct {
    pub owner: DeclarationId,
    pub root_defaults: Box<[CallableSemanticSignature]>,
    pub root_requirements: Box<[EnumRequirement]>,
    pub case_implementations: BTreeMap<VariantId, Box<[CallableSemanticSignature]>>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

Additional provenance/selection fields are permitted where needed, but do not replace the stable identities above.

### 7.6 Root member classification

For `impl Enum`:

```text
instance-side + bodyless
    -> EnumRequirement

instance-side + bodyful
    -> root/default implementation
       and ordinary root effective member

class-side + bodyful
    -> ordinary P1 class-side inherent behavior only
       NOT a closed-case default

class-side + bodyless
    -> invalid; closed-enum class-side requirements remain unsupported
```

A requirement and default with the same selector declared in separate root impl fragments are not a “requirement plus provided implementation” pair. They are conflicting root declarations of the same semantic slot. P2 does not import trait-style requirement/default pairing.

### 7.7 Exact-case member classification

For `impl Enum::Case(shape)`:

```text
bodyless member
    -> invalid

@class / class-side member
    -> invalid in P2

bodyful instance member matching root requirement
    -> exact-case requirement witness

bodyful instance member matching root default
    -> exact-case override

bodyful instance member absent from root
    -> case-only behavior
```

The callable owner remains:

```rust
CallableOwnerId::Variant(variant_id)
```

not the enum declaration and not the impl block.

### 7.8 Requirement identity and witness evidence

Preserve:

```text
EnumRequirementId(Expression.evaluate)
    !=
CallableId(Expression::Literal.evaluate)
```

`CaseRequirementResult` or its repository-equivalent remains the witness relation.

Do not rewrite the requirement ID to point at the selected case callable. Do not copy the root requirement into case callable identity.

P2 does not need to expose a public reflection object yet, but the semantic snapshot must retain enough stable identity/provenance for C7 to expose:

```text
root requirement
exact-case witness
impl provenance of witness
```

without reconstructing them from source.

### 7.9 Root defaults and override provenance

A root default is one root callable. It is not cloned into one synthetic callable per case.

For a case with no override:

```text
exact case runtime class
    inherits root callable
```

For a case override:

```text
exact case CallableId
    shadows root callable through ordinary dispatch
```

Semantic lookup may publish explicit selection/provenance metadata if necessary for exact-case tooling/reflection, but runtime duplication is forbidden.

### 7.10 GADT and variant-local specialization

Requirement/default compatibility must be evaluated in this order:

```text
root signature
    ↓
resolve targeted VariantInfo
    ↓
apply variant.case_environment
    ↓
apply covering declaration/variant-generic normalization
    ↓
compare exact-case member signature
```

Use existing case specialization machinery. Do not create an impl-specific GADT solver.

Parameter shape, labels/rest semantics, declared parameter types, return compatibility, and existing visibility/contract rules remain the same rules used by the existing enum requirement/default checker.

### 7.11 Exact-case body environment

When checking an exact-case impl member body, establish:

```text
Self = exact case template/specialized exact case
receiver owner = targeted VariantId
GADT case equalities = VariantInfo.case_environment
payload fields = canonical VariantFieldId semantics
root enum members = ordinary inherited/effective root behavior
case-local members = targeted case surface
impl generic binders = source checking environment only, normalized for publication
```

Do not create local bindings from the `_` placeholders in the impl target. Payload access remains receiver-owned data.

### 7.12 Case-only availability

Member lookup must preserve:

```text
receiver: Expression
    -> explicit root surface only

receiver: ExactCase(Expression::Literal, ...)
    -> root behavior
       + selected root defaults
       + exact-case overrides/witnesses
       + Literal-only members
```

Even if every current case declares the same case-only selector, do **not** promote it to the root surface. Root availability is explicit source semantics.

### 7.13 Closed requirement completeness

Continue validating root requirements over every semantic variant, including:

- singleton variants;
- zero-argument constructors;
- private-construction variants;
- generic variants;
- GADT-specialized variants.

Selection rule:

```text
root requirement R
    each variant must have compatible exact-case implementation

root default D
    case override exists -> use override
    otherwise            -> inherit D
```

A root default does not create a requirement-table entry merely because cases inherit it.

### 7.14 Compiler/runtime architecture

Keep `EnumLoweringSpec` structural unless live P1 architecture provides a compelling already-accepted reason otherwise.

Extend P1 semantic impl lowering target conceptually to:

```rust
pub enum InherentImplLoweringTarget {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}
```

Compiler behavior installation:

```text
Declaration(enum) target
    -> resolved enum root behavior class

ExactEnumCase(variant) target
    -> resolved hidden runtime behavior class for VariantId
```

The compiler must receive that target as semantic lowering. It must not:

- parse the impl target name itself;
- perform global class-name lookup;
- infer a `VariantId` from runtime class names;
- scan all impl blocks while compiling an enum;
- synthesize runtime class reopen operations.

`phalcom-core/src/vm/adt.rs` already owns root/case class topology. Reuse it. P2 should not redesign enum allocation, discriminants, `ProductStorage`, NativeOption, GC, or case class inheritance.

### 7.15 Transitional migration architecture

Use a temporary differential bridge, then delete it:

```text
TRANSITION ONLY

legacy behavior-bearing enum syntax ──┐
                                      ├─> normalized EnumBehaviorProduct
new impl-based behavior syntax ───────┘
                                      ↓
                              differential oracle
```

After semantic/compiler/runtime equivalence is proven at G3, remove legacy behavior source syntax and all direct `EnumMember::Behavior`/variant-body behavior consumers.

Permanent dual semantic paths are forbidden.

### 7.16 Incremental architecture

Required dependency distinctions:

```text
variant structural edit
    -> enum declaration product
    -> closed requirement completeness

root requirement signature edit
    -> root behavior product
    -> all case requirement statuses

root default body-only edit
    -> root callable body/lowering
    -> NOT enum structural identity

exact-case witness signature edit
    -> targeted case behavior product
    -> matching requirement/default compatibility

exact-case body-only edit
    -> targeted CallableId body/lowering
    -> NOT unrelated case signatures

case-only member add/remove
    -> targeted exact-case member surface
    -> NOT root declaration surface
```

Cold and incremental analyses must agree on diagnostics, callable identities, requirement status, and lookup results.

---

## 8. Ownership boundaries

### 8.1 Owns

P2 owns:

- canonical variants-only enum source syntax;
- migration of root behavior/requirements into `impl`;
- migration of case-local behavior into exact-case `impl`;
- semantic exact-case impl targeting by `VariantId`;
- root requirement/default classification for enum-root impls;
- exact-case witness/override/case-only classification;
- integration with GADT case environments;
- compiler target projection for exact-case impl behavior;
- removal of legacy enum-local behavior source paths;
- relevant incremental/source-index/LSP migration;
- focused compatibility migration of current enum fixtures/core source.

### 8.2 Does not own

P2 does not own:

- conditional `where`-guarded inherent impl applicability;
- concrete/repeated/non-covering ordinary impl specialization;
- trait declarations or trait conformance;
- associated types;
- public reflection APIs;
- a new GADT solver;
- new variant representation;
- changes to NativeOption representation;
- C1 product optimizer behavior;
- stable FFI layout;
- derived data equality/hash/toString policy;
- open classes/extensions;
- metatype trait syntax.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Enum variant set | `EnumDeclarationProduct` / `EnumInfo` | requirements, matching, lowering, tooling | impl parser or runtime registry as language authority |
| Exact case identity | `VariantId` + exact-case type | impl target resolution, lookup, compiler projection | strings, `ClassId`, discriminant |
| GADT case equations | `VariantInfo.case_environment` | body checking, requirement/default compatibility | impl-local re-derivation |
| Variant-local generic identity | variant constructor `GenericSignature` | case impl normalization/checking | name-only binders or new variant-owner category |
| Impl fragment provenance | P1 `ImplId` | diagnostics, source index, lowering provenance | `CallableId` as fragment ID |
| Root callable identity | `CallableId` owner `Declaration` | surfaces, compiler, tooling | impl ID or runtime method address |
| Exact-case callable identity | `CallableId` owner `Variant` | case lookup, witness relation, compiler | root callable or fake declaration |
| Enum requirement identity | `EnumRequirementId` | requirement table, diagnostics, future reflection | selected witness `CallableId` |
| Root/default/requirement/case classification | `EnumBehaviorProduct` or one normalized semantic product | checker, snapshot, lowering | compiler syntax scan |
| Ordinary root availability | P1 effective `DeclarationSurface` | dispatch/member lookup | case-union promotion |
| Case-only availability | exact-case semantic behavior product | refined lookup, tooling | root surface |
| Runtime case class | VM ADT registry | execution only | static exact-case identity |
| Impl executable target | semantic lowering | compiler/VM | compiler name lookup |

---

## 9. Global invariants

`INV-01` — Enum declarations are structural: canonical enum bodies contain variants only.

`INV-02` — `ImplId` records source/provenance; it never becomes callable, requirement, dispatch, or runtime class identity.

`INV-03` — `impl Enum` bodyless instance behavior is a closed-enum requirement; bodyful instance behavior is root/default behavior.

`INV-04` — Bodyful class-side `impl Enum` behavior remains ordinary P1 class-side behavior, not a closed-case default; bodyless class-side root requirements remain invalid.

`INV-05` — Exact-case behavior uses `CallableOwnerId::Variant(VariantId)` and is never re-owned by the enum root or impl block.

`INV-06` — `EnumRequirementId` remains distinct from every concrete witness `CallableId`.

`INV-07` — Closed enum requirements are not traits and create no trait/conformance objects.

`INV-08` — Requirement/default override compatibility is checked after exact-case/GADT specialization.

`INV-09` — Variant-local generic identity remains canonical and stable; impl-owned binders normalize into existing declaration/constructor parameter identities before publication.

`INV-10` — Exact-case impls are bodyful instance behavior only in P2.

`INV-11` — Exact-case target placeholders describe selector shape only; they do not create lexical payload bindings.

`INV-12` — Payload access in exact-case impl bodies comes from canonical `VariantFieldId` receiver state.

`INV-13` — Case-only behavior is available only on exact/refined case receivers and never structurally promoted to the root.

`INV-14` — Singleton, zero-argument constructor, positional, and labeled variant shapes remain distinct exact targets.

`INV-15` — Same-module inherent ownership from P1 applies to enum-root and exact-case impls.

`INV-16` — Source ordering among same-module impl fragments does not change accepted semantics or dispatch.

`INV-17` — Compiler lowering consumes semantic target/member facts; it does not reconstruct impl target semantics from AST or runtime names.

`INV-18` — Runtime dispatch continues to use one root behavior class and exact case behavior classes; P2 does not introduce runtime impl scanning or reopen semantics.

`INV-19` — Root defaults are installed once and inherited; they are not cloned into synthetic per-case callables.

`INV-20` — Variant constructors/singletons retain associated-callable identity and base reservation semantics; ordinary behavior never hides a variant constructor.

`INV-21` — `EnumLoweringSpec` remains structural unless an approved P1/P2 amendment explicitly changes that ownership boundary.

`INV-22` — C1 representation, `ProductStorage`, NativeOption specialization, exact runtime type semantics, and optimizer behavior are unchanged.

`INV-23` — No P3 conditional/specialized ordinary impl applicability is introduced by exact-case targeting.

`INV-24` — Transitional legacy and new syntax may coexist only long enough to prove differential equivalence; final P2 has one canonical behavior path.

`INV-25` — Incremental results match cold analysis for identities, diagnostics, requirement status, and exact-case lookup.

`INV-26` — Canonical P2 enum syntax no longer requires `@variant` to classify body members. Temporary compatibility acceptance of `@variant` must normalize to the same structural variant and add no semantic distinction.

---

## 10. Non-goals

Do not expand P2 into:

- `impl Point<Int>` or other ordinary concrete specialization;
- `impl<T> Point<T> where ...` conditional applicability;
- overlap/coherence selection for constrained inherent impls;
- trait syntax or witness tables;
- “hidden trait” lowering for enum requirements;
- exact-case public type annotation syntax;
- class-side exact-case behavior;
- bodyless exact-case requirements;
- runtime reflection APIs for requirements/witnesses;
- changes to enum physical representation;
- `match` redesign;
- derived behavior policy;
- module/package ownership redesign;
- repository-wide documentation reorganization.

---

## 11. Expected impact map

### 11.1 Expected source areas

#### Syntax / AST

- `phalcom-ast/src/ast.rs` — make enum structure variants-only; extend/reuse P1 impl target syntax for exact cases.
- `phalcom-ast/src/parser.rs` — canonical bare variant parsing; exact-case impl target parsing; transitional legacy parse bridge; final removal of enum behavior parsing.
- `phalcom-ast/src/selector.rs` — preserve exact variant selector construction without `EnumMember::Behavior` branching.

#### Semantic core

- P1 landed impl target/contribution modules — add exact enum-case target support.
- `phalcom-semantic/src/checker/enum_behavior.rs` — change input from behavior-bearing `EnumDef` to normalized impl contributions + structural enum semantics.
- `phalcom-semantic/src/enum_requirements.rs` — preserve existing checker; extend only where override/default compatibility/provenance requires it.
- `phalcom-semantic/src/checker/enum_declaration.rs` — consume variants-only AST.
- `phalcom-semantic/src/checker/declaration_signature.rs` — ensure shared `BehaviorMember`/impl-origin syntax works for requirement/default/case signatures.
- `phalcom-semantic/src/session.rs` — publish enum behavior/contracts from impl products, not nested enum syntax.
- `phalcom-semantic/src/db/product.rs`, `db/query.rs`, `db/fingerprint.rs` — product ownership and incremental dependencies.
- `phalcom-semantic/src/semantic_shard.rs` — body/source ownership migration.
- `phalcom-semantic/src/core_surface/source.rs` — remove enum behavior extraction.
- `phalcom-semantic/src/source_index/builder.rs`, `source_index/occurrence.rs` — source identity through impl provenance.
- exact-case member lookup/dispatch files identified from live P1 tree — preserve root + case layering.

#### Compiler/runtime

- `phalcom-core/src/modules/semantic_lowering.rs` — exact-case impl executable target projection.
- P1 landed impl compiler/lowering file — install exact-case members on semantically resolved case classes.
- `phalcom-core/src/compiler/lib/enum_decl.rs` — delete behavior compilation from enum AST; enum compilation becomes structural.
- `phalcom-core/src/native/source.rs` — stop indexing enum-local behavior; consume P1/P2 impl/source identities where required.
- `phalcom-core/src/vm/adt.rs` — expected to need little or no semantic change; only expose/reuse safe target lookup if P1 compiler installation needs it.

#### Tooling

- `phalcom-lsp/` only where P1 source-index abstractions do not already make the migration automatic.
- Do not add LSP-specific semantic reconstruction.

#### Normative docs / implementation state

- `docs/spec/adts.md` — canonical variants-only + impl-based enum behavior semantics.
- P1-established general `impl` spec chapter if one exists — add exact-case target grammar only where necessary.
- `docs/implementation/LANG005/LANG005.C2/CHECKPOINT.md` or the live canonical checkpoint record.
- P2 walkthrough and P2 handoff.

### 11.2 Expected tests

- `phalcom-ast/tests/enum_syntax.rs`
- `phalcom-ast/tests/parser.rs`
- current P1 impl parser/AST tests
- `phalcom-semantic/tests/semantic/adts/declarations.rs`
- current P1 effective-surface/impl tests
- semantic incremental tests under `phalcom-semantic/tests/semantic/incremental/`
- source-index integration tests where live P1 placed impl navigation tests
- `phalcom-core/tests/core/language/algebraic_data.rs` or repository-equivalent
- targeted language-corpus fixtures only if existing shipping enum behavior fixtures live there

### 11.3 Unexpected-touch rule

Touch adjacent helpers/tests when mechanically required by a type rename, enum exhaustiveness, or shared interface change.

**STOP AND CONSULT** before entering a materially different subsystem if doing so changes architecture, identity, ownership, or semantic authority. Examples: module linker policy, GC representation, optimizer IR, trait solver, or runtime class model.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- variants-only canonical enum source model;
- enum requirements/defaults remaining enum-specific, not traits;
- exact-case semantic target = `VariantId`;
- exact-case callable owner = variant;
- requirement identity separate from witness identity;
- GADT specialization owner = existing case environment;
- same-module P1 ownership policy;
- no conditional ordinary impl applicability in P2;
- compiler consumes semantic lowering;
- runtime root/case class topology remains execution machinery, not semantic identity;
- case-only members do not become root members;
- transitional dual source paths are deleted before completion.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact Rust enum/struct names for impl targets;
- helper names and private decomposition;
- whether provenance is stored directly on `EnumBehaviorProduct` or in an adjacent product;
- exact query-key names;
- exact location of target-normalization helpers;
- equivalent P1 APIs caused by implementation drift;
- diagnostic wording consistent with existing style;
- test file subdivision within the owning existing test modules.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify after P1 | If false |
|---|---|---|
| P1 has one canonical `BehaviorMember` syntax category. | AST + P1 walkthrough | Adapt mechanically if equivalent; consult if behavior syntax is duplicated by owner. |
| P1 has stable `ImplId`. | semantic identity + P1 walkthrough | STOP if impl provenance is source-range/name-only. |
| P1 has callable-definition provenance. | surface/query products | STOP if body lookup still depends on scanning declaration syntax. |
| P1 compiler gets semantic impl lowering. | `semantic_lowering.rs` + compiler | STOP if compiler resolves impl targets itself. |
| P1 same-module ownership is enforced semantically. | target resolver tests | Fix P1 first if absent. |
| P1 effective root surface separates signatures from bodies. | semantic DB products | Adapt if equivalent; STOP if merge mutates one global surface by source walk. |
| Exact-case member lookup already understands variant-owned callables. | checker dispatch/lookup | Reuse it. If exact-case behavior cannot be represented without changing canonical type identity, consult. |
| Existing requirement checker still specializes through `case_environment`. | `enum_requirements.rs` | Restore existing authority; do not duplicate it in impl checker. |
| Runtime still maps `VariantId` to hidden case `ClassId`. | `vm/adt.rs` / ADT registry | Adapt target lookup. STOP if P1 removed/replaced this topology materially. |
| Generic variant target syntax has an established spelling. | parser/spec/tests | Reuse. If absent and variant-local exact-case impl needs a new public syntax choice, consult. |
| C2 checkpoint record exists after workflow migration. | docs/implementation | Create only the minimal canonical record if policy clearly permits; otherwise consult before restructuring docs. |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of these occurs:

1. P1 required takeover architecture is absent or materially different.
2. Exact-case targeting cannot be represented by canonical `VariantId` without introducing another semantic identity.
3. Exact-case impl support appears to require ordinary P3 specialization/applicability machinery.
4. Variant-local generic exact-case impls require a new public syntax decision not already established by repository/spec/user ruling.
5. Requirement/default compatibility cannot reuse current case specialization without a second solver.
6. Correct case-only lookup would require promoting case members into the root `DeclarationSurface`.
7. Compiler installation would require runtime-global name lookup, class reopening, load-order mutation, or scanning source impls.
8. Native/Universe bootstrapping appears to require a permanent second enum behavior extraction path.
9. Existing runtime hidden case classes cannot accept semantically lowered case members without changing ADT representation or class identity.
10. Root default override compatibility has two plausible semantics with materially different type-system consequences.
11. A root/bodyless/class-side or exact-case class-side behavior rule conflicts with the current accepted spec.
12. A core plan premise conflicts materially with the post-P1 live source or tests.
13. The same nontrivial semantic failure persists after one serious corrective attempt.
14. Tests can pass only by weakening a numbered invariant.
15. A new subsystem outside the expected impact map becomes necessary for architectural reasons.
16. A duplicate source of truth would be introduced for variants, case environments, callable identities, or impl targets.
17. Cold/incremental parity cannot be restored without changing the P1 DB ownership model.
18. Scope expands into traits, associated types, optimizer representation, or general specialization.

A triggered consultation cannot be self-waived.

### Consultation packet format

Collect only:

```text
Observed:
Expected invariant:
Relevant revision/diff:
Exact owners/symbols:
Smallest failing test:
Current semantic product(s):
Why the planned owner cannot express the needed fact:
Two plausible architectures, if there are two:
Decision requested:
```

Do not ask an adviser to rediscover the repository from scratch.

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

Examples: renamed P1 helper, enum exhaustiveness compile errors, import changes, test filter mismatch.

### Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure. If it persists, **STOP AND CONSULT**.

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant code/state changed.

---

## 15. Testing surface analysis

### Coverage obligations

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | canonical syntax | INV-01, INV-26 | Bare singleton/constructor/payload variants parse in variants-only enum. |
| CV-02 | root requirement | INV-03, INV-06 | Bodyless instance member in `impl Enum` publishes stable `EnumRequirementId`. |
| CV-03 | root default | INV-03, INV-19 | Bodyful root member is ordinary root behavior/default and no requirement entry. |
| CV-04 | exact witness | INV-05, INV-06 | Exact-case impl satisfies root requirement and status names variant-owned callable. |
| CV-05 | missing witness | INV-03 | Missing case emits existing missing-requirement diagnostic. |
| CV-06 | incompatible witness | INV-08 | Wrong parameter/return/shape emits incompatibility. |
| CV-07 | GADT specialization | INV-08 | Root requirement specializes through case environment before comparison. |
| CV-08 | variant-local generics | INV-09 | Covering exact-case impl preserves canonical constructor-local generic identity. |
| CV-09 | payload access | INV-11, INV-12 | Case impl body accesses canonical payload field names; `_` target placeholders bind nothing. |
| CV-10 | root default override | INV-08, INV-19 | Exact case compatible override shadows root default; non-overriding case inherits root. |
| CV-11 | case-only member | INV-13 | Available on exact/refined case; rejected/unavailable on root receiver. |
| CV-12 | exact selector identity | INV-14 | Singleton `Case`, constructor `Case()`, and `Case(_)` target distinct `VariantId`s. |
| CV-13 | labels | INV-14 | Labeled variant target resolves exact selector and mismatched label diagnoses. |
| CV-14 | duplicate case fragments | INV-05, INV-16 | Two impl fragments defining same exact-case selector conflict; no last-wins. |
| CV-15 | root duplicate fragments | INV-03, INV-16 | Requirement/default/concrete root selector collision is deterministic source error. |
| CV-16 | class-side boundary | INV-04, INV-10 | Root bodyful class-side stays ordinary inherent behavior; root bodyless and case class-side reject. |
| CV-17 | associated reservation | INV-20 | Root behavior cannot hide/collide illegally with variant associated base; constructor identity survives. |
| CV-18 | ownership | INV-15 | Foreign-module enum root/case impl rejects exactly like P1 inherent impl. |
| CV-19 | differential oracle | INV-24 | Legacy nested behavior and new impl source produce equivalent enum behavior/requirement products before legacy removal. |
| CV-20 | runtime root/default | INV-17, INV-18, INV-19 | Root default runs on all cases without per-case clone. |
| CV-21 | runtime case override | INV-18 | Exact-case override dispatches on targeted case only. |
| CV-22 | runtime case-only | INV-13, INV-18 | Case-only member executes on targeted runtime case and is absent elsewhere. |
| CV-23 | incremental new variant | INV-25 | Adding a variant recomputes closed requirement completeness and can create missing-witness diagnostic. |
| CV-24 | incremental body edit | INV-25 | One case body edit does not rebuild unrelated enum structure/case signatures. |
| CV-25 | source identity | INV-02, INV-05, INV-06 | Navigation/source index distinguishes impl provenance, root requirement, and case callable. |
| CV-26 | final negative syntax | INV-01, INV-24 | Root/case behavior nested inside enum declaration no longer parses as canonical behavior. |
| CV-27 | final negative search | INV-24 | Production code no longer contains semantic consumers of `EnumMember::Behavior`/legacy nested case behavior. |
| CV-28 | cold/incremental equivalence | INV-25 | Cold and edited snapshot agree on requirements, lookup, diagnostics. |
| CV-29 | native/Universe composition | INV-17, INV-20 | Shipping enum sources/bootstrap resolve migrated impl behavior without duplicate native indexing. |
| CV-30 | C1 preservation | INV-22 | Representative enum construction/match/product tests remain unchanged in behavior. |

Do not mechanically add one test per row if one well-designed test proves several rows. Every row must, however, have explicit evidence by final acceptance.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Sparse, discriminating feedback only. Most implementation time belongs here.

#### STABILIZE MODE

Focused regressions for AST, enum semantic products, exact-case behavior, incremental dependencies, and ADT runtime behavior.

#### CERTIFY MODE

P2 is not C2 checkpoint closure because P3 remains. Workspace/release certification is therefore not mandatory for normal P2 completion.

### Verification ladder

```text
T0 — exact new regression / exact parser or semantic reproducer
T1 — directly affected enum/impl tests
T2 — owning AST/semantic/core focused suite
T3 — adjacent source-index/LSP/runtime integration only when changed
T4 — broad crate verification only if evidence warrants
T5 — workspace/release verification deferred to C2 checkpoint/release certification
```

### Mandatory during BUILD

- exact parser tests for each newly enabled/disabled syntax shape;
- exact semantic tests for root requirement/default and exact-case witness/override;
- exact compiler/runtime regression when lowering changes become executable;
- no broad suite after each task.

### Mandatory at named gates

Use the commands under Section 19. If a test filter is uncertain after P1 drift, first use `-- --list` and confirm the filter selects nonzero tests.

### Mandatory during STABILIZE

At minimum:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test enum_syntax
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
```

Also run the exact P1 impl/effective-surface focused test group and the exact incremental/source-index filters modified by P2.

### Only if evidence demands

- full `phalcom-semantic --test semantic`;
- full `phalcom-core --test core`;
- full `phalcom-ast` crate tests;
- `phalcom-lsp` crate tests if generic source-index products changed in a way not covered by focused integration tests;
- language-corpus subsets containing migrated shipping enum fixtures.

### Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --all-targets
broad unrelated language corpus
full REPL/CLI suites
optimizer/performance suites unrelated to enum behavior
```

Reason: P2 is a semantic/source migration inside a larger incomplete C2 checkpoint. These commands have poor information value during iterative implementation and can consume time on unrelated baseline failures.

---

## 17. Baseline / unrelated failure policy

Classify failures:

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
- Never weaken assertions, suppress diagnostics, or skip tests to make a broad gate green.
- Record deferred failures in the C2 checkpoint ledger and P2 walkthrough.

---

# 18. Tasks

## T0 — Verify P1 takeover and establish P2 checkpoint state

### Purpose

Make P2 execution conditional on the actual landed P1 architecture and establish durable workflow state before semantic edits.

### Preconditions

- C2.P1 implementation claims complete.
- P1 walkthrough/handoff are available.
- Working tree status is known and unrelated edits are preserved.

### Consumes

- P1 plan, walkthrough, handoff;
- live P1 source;
- current `LANG005.C2` checkpoint record if present.

### Produces

- verified P1 interface map;
- P2 marked active in checkpoint ledger;
- explicit mechanical drift notes;
- no language code changes unless minimal checkpoint bookkeeping is required.

### Files and symbols

**Read:**

```text
AGENTS.md
P1 plan/walkthrough/handoff
C2 CHECKPOINT.md
P1 impl AST/identity/contribution/effective-surface/query/lowering code
P1 focused tests
```

**Modify:**

- only the canonical C2 checkpoint record to mark P2 active, if required by repository workflow.

### Required implementation shape

Verify the hard predecessor gate from Section 0. Record exact landed symbol/path equivalents in `CHECKPOINT.md` or the P2 execution notes.

Run targeted source searches, not tests, to prove ownership:

```text
ImplId exists and is stable
impl target resolution is semantic
callable definition provenance exists
final root surface is an effective merge product
compiler impl lowering exists
compiler does not implement runtime reopen
```

### Forbidden approaches

- do not “fill in” a missing P1 interface inside P2;
- do not start parser work while predecessor architecture is uncertain;
- do not reorganize the implementation-doc tree merely to satisfy an illustrative README hierarchy.

### Tests to run now

No broad tests. Run at most the smallest P1 acceptance filter needed to verify a disputed predecessor claim.

### Acceptance

P1 takeover table is verified against live source with no architectural mismatch.

### Local STOP / CONSULT triggers

Any missing hard predecessor interface; any material P1 deviation in identity, surface ownership, provenance, or compiler lowering.

### Checkpoint update

Record P2 start revision and verified P1 interface names/paths.

---

## T1 — Reconcile the normative enum/impl surface and add canonical variants-only syntax

### Purpose

Make the language specification and parser/AST express the new source model before semantic consumers are migrated.

### Preconditions

T0 PASS.

### Consumes

- P1 `impl` AST/target grammar;
- current `EnumDef`, `VariantDecl`, `EnumMember` grammar;
- ratified P2 source forms.

### Produces

- updated normative ADT enum-behavior syntax;
- canonical bare variant parsing;
- exact-case impl target syntax represented in AST;
- transitional ability, if mechanically useful, to parse legacy enum behavior long enough for differential G2/G3 evidence;
- no permanent semantic dual authority.

### Files and symbols

**Read/Modify:**

```text
docs/spec/adts.md
P1's normative impl spec chapter, if one exists
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/selector.rs
P1 impl target AST/parser files
```

**Tests:**

```text
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/parser.rs
P1 impl syntax tests
```

### Required implementation shape

1. Update `docs/spec/adts.md` so canonical examples and rules use variants-only enum declarations plus root/exact-case impls.
2. Preserve all ADT identity/GADT/visibility/representation semantics unrelated to behavior location.
3. Make every canonical enum body item parse as a variant declaration. `@variant` may remain accepted as compatibility but cannot be required for variant classification.
4. Extend P1 impl-target AST to represent exact variant selector shapes explicitly enough to resolve a `VariantId` semantically.
5. Do not put semantic IDs directly into AST.
6. Keep exact-case target syntax distinct from expression callable-reference syntax.
7. During the transition only, retain enough legacy parser support to construct an old-source differential oracle. Mark/remove it in T6.

### Forbidden approaches

- no fake exact-case type source syntax;
- no AST `VariantId`;
- no method-reference parsing for impl targets;
- no capital-letter heuristic as semantic authority if the grammar can directly parse a variant declaration;
- no new general specialization syntax;
- no deletion of legacy parser support before G2/G3 differential evidence exists.

### Test changes required

Cover CV-01, CV-12, CV-13, CV-16, CV-26 in staged form.

Required parser examples:

```phalcom
enum E { A B() C(_ x: Int) D(label x: Int) }
impl E::A { ... }
impl E::B() { ... }
impl E::C(_) { ... }
impl E::D(label: _) { ... }
```

Also prove `&E::C(_)` remains a callable-reference expression and is not confused with an impl target.

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test enum_syntax
```

Run a narrower parser filter first while iterating if it selects the intended tests.

### Tests explicitly deferred

Semantic/runtime tests; intermediate semantic consumers still use old behavior AST.

### Acceptance

Canonical new syntax is representable and tested; old semantic consumers still compile through the transitional compatibility shape; docs/spec states one effective target rule.

### Local STOP / CONSULT triggers

Parser requires a previously unratified variant-generic target syntax; exact singleton/zero-arg selector distinction cannot survive the target AST.

### Checkpoint update

Record the canonical exact-case target AST shape because later tasks depend on it.

---

## T2 — Extend P1 impl target resolution to exact enum cases and normalize enum behavior contributions

### Purpose

Make semantic impl resolution produce canonical root/case behavior facts from new source without changing requirement semantics.

### Preconditions

T1 PASS; P1 target/contribution architecture verified.

### Consumes

- P1 `ImplId`, generic binder, target resolver, `InherentImplContribution`, effective surfaces;
- `EnumDeclarationProduct`, `VariantInfo`, `VariantId`;
- shared behavior-member signature formation.

### Produces

- exact-case `InherentImplTarget` repository-equivalent;
- covering generic maps for exact-case impls;
- normalized root/case impl contributions;
- `EnumBehaviorProduct` built from impl contributions rather than only nested enum source;
- stable `CallableId` ownership and `ImplId` provenance.

### Files and symbols

**Read/Modify:**

```text
P1 impl target resolver/product modules
phalcom-semantic/src/identity.rs only if target enum belongs there
phalcom-semantic/src/checker/enum_behavior.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/enum_declaration.rs as needed for target lookup helpers
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/types/case_environment.rs
phalcom-semantic/src/types/case_instantiation.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
```

### Required implementation shape

1. Resolve root targets through P1 unchanged.
2. Resolve exact-case targets by:
   - resolving the enum declaration/type owner;
   - deriving exact selector shape from the target syntax;
   - selecting the canonical `VariantId` from `EnumInfo`/variant products;
   - validating same-module ownership;
   - building explicit covering substitutions for declaration and variant-local generic domains.
3. Build case callable IDs with `CallableOwnerId::Variant`.
4. Keep impl-owned generic parameters only in source-checking space; normalize published member signatures into canonical target parameter identities.
5. Refactor `build_enum_behavior` or repository-equivalent so new impl contributions feed:
   - root defaults;
   - root requirements;
   - case implementations.
6. Preserve `EnumRequirementId(owner, selector)` exactly.
7. Classify class-side/bodyless boundaries per Section 7.6/7.7.
8. Keep transitional legacy enum-source contribution adapter only for differential testing. Both sources must enter the same normalization function/product.

### Forbidden approaches

- no `VariantId` construction from raw strings outside canonical selector/enum lookup;
- no case target encoded as `DeclarationId`;
- no requirement identity derived from impl ID;
- no second signature builder for enum impls;
- no duplicated generic inference/case substitution;
- no P3 applicability table.

### Test changes required

Cover CV-02, CV-03, CV-04, CV-08, CV-12, CV-13, CV-14, CV-15, CV-16, CV-18.

At semantic-product level assert:

```text
new root bodyless impl -> root_requirements exactly once
new root bodyful impl  -> root_defaults exactly once
exact case impl         -> case_implementations[VariantId]
case callable owner     -> Variant(VariantId)
requirement ID           -> root enum + selector
impl provenance          -> source ImplId, separately queryable
```

### Tests to run now

Run exact new semantic filters only. Prefer:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations::<new_test_name>
```

If filter names drift, inspect `-- --list` first.

### Tests explicitly deferred

Full semantic suite, compiler/runtime dispatch, LSP.

### Acceptance

New impl syntax produces canonical enum behavior facts with the same stable identities expected by existing requirement machinery.

### Local STOP / CONSULT triggers

Exact-case generic covering requires new ownership model; P1 contributions cannot target a variant without flattening into root surface; semantic product needs runtime identity.

### Checkpoint update

Record exact implemented target/product types and any mechanical deviations from conceptual names.

---

## T3 — Preserve closed contracts, GADT specialization, exact-case lookup, and body checking

### Purpose

Connect normalized impl-origin behavior to the existing requirement/default/case semantics and exact-case checker environment.

### Preconditions

T2 canonical products exist.

### Consumes

- `EnumBehaviorProduct` from T2;
- `check_enum_requirements` and requirement table;
- `VariantInfo.case_environment`;
- P1 callable-definition provenance/body query;
- exact-case member lookup/dispatch infrastructure.

### Produces

- requirement completeness from impl-origin requirements;
- specialized witness/default override compatibility;
- exact-case body checking under correct `Self`, GADT, payload, and generic environments;
- layered root/exact-case member availability;
- stable diagnostics and source attribution;
- incremental dependencies for root/case contract edits.

### Files and symbols

**Read/Modify:**

```text
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/session.rs
P1 callable body/signature query code
exact-case behavioral lookup/dispatch modules from live tree
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/semantic_shard.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/adts/declarations.rs
P1 impl body/surface tests
phalcom-semantic/tests/semantic/incremental/* relevant modules
```

### Required implementation shape

1. Feed T2 `root_requirements` and `case_implementations` into the existing requirement checker.
2. Preserve case specialization through `VariantInfo.case_environment` before type comparison.
3. Reuse current compatibility rules. If default override validation already has a dedicated owner, feed it the same specialized case view; otherwise factor a shared root-vs-case compatibility helper rather than duplicating requirement logic.
4. Check exact-case impl bodies through P1's callable-definition provenance, not by walking `VariantDecl.body`.
5. Establish exact-case `Self` and payload-field availability from semantic variant products.
6. Ensure root effective behavior is inherited/visible to exact cases while case-only members do not enter root `DeclarationSurface`.
7. Preserve source diagnostics at impl member spans and target spans.
8. Record correct query dependencies:
   - requirement signatures depend on relevant case signature products;
   - body-only edits do not invalidate enum structure;
   - adding/removing a variant invalidates closed requirement completeness;
   - case-only changes do not invalidate root surface.

### Forbidden approaches

- no checking unspecialized raw root signature against GADT case signature;
- no copying payload fields into lexical locals;
- no widening exact case to root before member lookup;
- no “member exists on all cases -> root member” inference;
- no requirement satisfaction from runtime inheritance;
- no scan of all module syntax on every member lookup.

### Test changes required

Cover CV-04–CV-11, CV-16, CV-23, CV-24, CV-28.

Required hostile cases:

1. GADT root requirement specialized to `Expr<Int>` succeeds only for correct case implementation.
2. Wrong unspecialized-compatible but specialized-incompatible witness fails.
3. Variant-local generic case implementation retains canonical generic identities.
4. Case payload field is available in case impl body.
5. `_` target placeholder is not a binding.
6. Case-only method unavailable on root even when every variant defines it.
7. New variant causes missing requirement diagnostic on incremental reanalysis.
8. Body-only change in one case preserves unrelated variant IDs/signatures.
9. Private constructor variant still participates in requirement completeness.
10. Root bodyful default is not added to requirement table.

### Tests to run now

Targeted semantic filters only, then G2.

### Tests explicitly deferred

Runtime behavior until compiler lowering is migrated.

### Acceptance

All formal enum contract semantics work from impl-origin products; exact-case body checking and lookup retain GADT/payload precision; incremental ownership is explicit.

### Local STOP / CONSULT triggers

A second GADT solver seems necessary; exact-case lookup can only work by mutating root surface; default override rule is semantically ambiguous; one semantic failure survives the allowed correction.

### Checkpoint update

Record any new durable selection/provenance product introduced for default overrides or exact-case effective behavior.

---

## T4 — Project impl behavior through semantic lowering and delete AST-driven enum behavior compilation

### Purpose

Make executable behavior installation consume canonical semantic impl targets while preserving existing root/case runtime topology.

### Preconditions

G2 PASS for semantic products/contracts.

### Consumes

- P1 impl lowering architecture;
- T2 exact-case target;
- T3 accepted callable definitions;
- VM ADT registry `VariantId -> case ClassId` mapping.

### Produces

- exact-case impl lowering target;
- root/case behavior installation through semantic lowering;
- structural-only enum declaration compilation;
- no direct compiler dependence on enum-local behavior AST.

### Files and symbols

**Read/Modify:**

```text
phalcom-core/src/modules/semantic_lowering.rs
P1 landed impl compiler/lowering files
phalcom-core/src/compiler/lib/enum_decl.rs
phalcom-core/src/vm/adt.rs only if a narrow semantic-target lookup API is needed
phalcom-core/src/chunk.rs only if P1 stores executable semantic pools there
```

### Required implementation shape

1. Extend P1 executable impl target to carry exact `VariantId`.
2. Project only semantically accepted bodyful members. Requirements have no executable body and no runtime installation.
3. Resolve enum root target to existing root behavior class through semantic/runtime registry identity.
4. Resolve exact case target to existing hidden case class by `VariantId`.
5. Compile/install root default once on root class.
6. Compile/install exact-case witness/override/case-only callable on case class.
7. Delete root/case behavior loops from `compile_enum`; enum compilation handles structural enum registration/variant representation only.
8. Preserve payload getters installed by VM from variant structural metadata.
9. Fail closed if semantic impl lowering is absent or target cannot be resolved; do not fall back to name-based runtime lookup.

### Forbidden approaches

- no runtime class reopen opcode/path;
- no global target-name lookup;
- no compiler scan of all impl AST;
- no behavior added to `EnumLoweringSpec` merely because `compile_enum` used to own it;
- no cloned root defaults per case;
- no runtime requirement/witness table unless an independent accepted runtime requirement appears.

### Test changes required

Cover CV-20, CV-21, CV-22, CV-29, CV-30.

Required runtime program should prove in one small fixture:

```phalcom
root default
case A inherits default
case B overrides default
root requirement witnessed by both
case A has A-only member
case B cannot dispatch A-only member
payload case method reads payload
```

Keep static availability failures in semantic tests; runtime test should exercise only semantically valid sends.

### Tests to run now

Exact core regression first, then:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
```

Do not run full core crate yet.

### Acceptance

Runtime behavior matches existing enum semantics using semantically lowered impls; `compile_enum` has no behavior-authority scan.

### Local STOP / CONSULT triggers

Case target cannot resolve without runtime naming; VM requires class-model redesign; NativeOption case behavior requires special duplicated semantics rather than the canonical case class mapping.

### Checkpoint update

Record executable target type and confirm `EnumLoweringSpec` remained structural.

---

## T5 — Migrate source identity, native/source indexing, and incremental consumers

### Purpose

Move tooling and native/source consumers from nested enum behavior syntax to P1/P2 canonical impl provenance/products.

### Preconditions

T3 semantic identities stable; T4 lowering target stable.

### Consumes

- P1 source-index impl support;
- `ImplId`, callable definition provenance;
- `EnumRequirementId` and case callables;
- T2 exact-case target identity.

### Produces

- source index/navigation for enum root/case impls;
- no direct enum behavior scan in semantic source tooling;
- native/Universe source indexing compatible with migrated impl syntax;
- incremental fingerprints keyed to correct owners.

### Files and symbols

**Read/Modify as needed:**

```text
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/native/source.rs
phalcom-lsp/* only if P1 generic source index does not make this automatic
```

### Required implementation shape

1. Requirements retain their own semantic identity but source occurrence/navigation points to the bodyless member in its root impl.
2. Case callables point to their exact impl member source and retain `ImplId` provenance.
3. Impl target occurrences navigate to enum declaration/variant identity appropriately.
4. Remove source-index logic that crawls `EnumMember::Behavior` or `VariantDecl.body` for behavior.
5. Native/source indexing must consume structural enum declarations separately from impl behavior; do not convert behavior back into synthetic `ClassMember` entries.
6. Incremental fingerprints distinguish enum structural changes from impl signature/body changes using P1 query ownership.
7. LSP consumes semantic/source-index products and performs no enum-specific semantic reconstruction.

### Forbidden approaches

- no LSP-only enum impl resolver;
- no source-range-as-identity fallback;
- no native-only duplicate behavior parser;
- no whole-module invalidation solely because one exact-case body changed if P1 can represent narrower ownership.

### Test changes required

Cover CV-23, CV-24, CV-25, CV-28, CV-29.

Add focused navigation/incremental tests only where current architecture has an owning test module.

### Tests to run now

- exact source-index/incremental filters;
- `cargo check -p phalcom-lsp` only if LSP code was mechanically touched and no smaller compile target exists;
- exact native/Universe bootstrap regression if `native/source.rs` changed materially.

### Tests explicitly deferred

Full LSP/extension suite unless a focused failure indicates it is necessary.

### Acceptance

No consumer derives enum behavior directly from enum syntax; source identity and incremental results are stable through impl provenance.

### Local STOP / CONSULT triggers

Native bootstrap cannot see P1/P2 impl products without creating a permanent second semantic path; source-index schema cannot represent requirement identity separately from callable identity.

### Checkpoint update

Record durable source-index identity relationships and any deferred tooling baseline failures.

---

## T6 — Capture differential evidence, remove legacy behavior-bearing enum syntax, and migrate repository fixtures

### Purpose

Close the migration by proving equivalence and deleting the old source/AST/consumer path.

### Preconditions

T2–T5 new path works end-to-end while transitional old syntax still exists.

### Consumes

- legacy enum-source adapter;
- new impl-origin semantic path;
- focused shipping enum fixtures/Universe sources.

### Produces

- recorded old/new semantic equivalence evidence;
- migrated shipping source/tests;
- variants-only final AST/parser;
- no production `EnumMember::Behavior` semantic path;
- no variant-local behavior body path;
- final negative syntax diagnostics for nested behavior.

### Files and symbols

**Modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/selector.rs
all production consumers found by targeted search for:
  EnumMember::Behavior
  EnumBehaviorMember
  parse_enum_behavior_member
  variant.body behavior traversal
shipping .ph enum behavior sources identified by rg
relevant tests/fixtures
```

`EnumBehaviorMember` may disappear entirely if P1 `BehaviorMember` is canonical and no compatibility API requires the alias. If a temporary type alias remains for source compatibility, no production enum-specific semantic consumer may depend on it.

### Required implementation shape

#### Step A — differential evidence before deletion

For representative nongeneric, generic, GADT, default/override, requirement, payload, singleton, and case-only programs, compare old source against new source at the canonical semantic products:

```text
root default selectors/signatures
root requirement IDs/signatures
case CallableIds/signatures
case requirement statuses
runtime result for executable equivalents
```

Record the evidence in tests and later in the walkthrough.

#### Step B — remove legacy canonical source path

- remove `EnumMember::Behavior` from final AST or reduce `EnumDef` to a direct variant collection, whichever best matches live P1 AST conventions;
- remove enum-root behavior parser;
- remove nested variant behavior bodies from canonical variant syntax;
- migrate semantic/compiler/source-index matches to variants-only exhaustive shapes;
- migrate repository `.ph` sources/tests to root/exact-case impls;
- keep `@variant` compatibility only if intentionally deferred to C8; it must parse as a variant attribute and must not reintroduce behavior classification.

#### Step C — final negative gates

New enum-local behavior must fail at syntax/semantic boundary with a clear diagnostic rather than silently becoming an odd variant.

### Forbidden approaches

- no permanent test-only production legacy semantic path;
- no hidden conversion of old nested behavior into synthetic impls after plan completion;
- no deleting historical implementation/spec records solely because source syntax changed;
- no opportunistic migration of unrelated legacy `@data` behavior.

### Test changes required

Cover CV-19, CV-26, CV-27, CV-29, CV-30.

### Tests to run now

Run G3/G4 focused gates below after migration. Do not run workspace tests.

### Acceptance

The repository has one production enum behavior source path: impl. Enum declarations are variants-only. Differential evidence was captured before the old path was deleted.

### Local STOP / CONSULT triggers

A shipping enum source depends on semantics not expressible through the ratified impl model; removing the old path requires weakening an existing GADT/visibility/associated-lookup invariant.

### Checkpoint update

Record legacy path deletion, compatibility status of `@variant`, and differential evidence summary.

---

## T7 — Focused stabilization, negative searches, and delivery records

### Purpose

Prove the coherent P2 slice without escalating to full C2/release certification and leave durable takeover state for C2.P3.

### Preconditions

T6 complete.

### Consumes

- all P2 implementation and tests;
- C2 checkpoint ledger.

### Produces

- focused verification ledger;
- clean negative searches;
- P2 walkthrough;
- P2 handoff for C2.P3;
- truthful completion state.

### Required implementation shape

1. Run final focused gates from Section 19 serially.
2. Run targeted negative source searches.
3. Review scoped diff for duplicate semantic authority, stale legacy scans, accidental P3 work, and unrelated changes.
4. Update C2 checkpoint record.
5. Create walkthrough and handoff.
6. Mark P2 `FOCUSED_TESTED` only if all required focused evidence passes. Do not mark C2 `RELEASE_COMPLETE`; P3 remains.

### Negative searches

Use repository-equivalent targeted `rg` checks. Expected production results after migration:

```text
EnumMember::Behavior
    -> no production source consumers

parse_enum_behavior_member
    -> absent

variant body behavior traversal
    -> absent from production semantics/compiler/source index

runtime reopen / target-name lookup in impl compiler
    -> absent

compiler-side enum impl target parsing/resolution
    -> absent
```

`EnumBehaviorProduct`, `EnumRequirementId`, and enum-specific contract semantics should still exist; do not remove them merely because source moved to impl.

### Tests to run now

G4/G5 below.

### Acceptance

All required coverage obligations have evidence; no known P2-caused focused failure remains; documentation state is durable; C2.P3 can start without re-investigating P2 architecture.

### Local STOP / CONSULT triggers

Any final negative search reveals a second production semantic path; focused stabilization uncovers one persistent semantic mismatch.

---

# 19. Verification gates

## G0 — P1 takeover gate

**Purpose:** prove P2 has a valid predecessor.

Run targeted source inspection and the smallest P1 acceptance filter necessary. Do not proceed if P1 architecture is not actually present.

Expected:

```text
PASS: canonical impl provenance/effective-surface/lowering interfaces exist
```

On failure: stop P2. This is not a P2 bug.

---

## G1 — Canonical syntax and exact-case target gate

**Purpose:** prove new enum/impl syntax forms exact structural targets without semantic workarounds.

Run:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test enum_syntax
```

Plus the exact P1 impl parser test filter containing exact-case target tests.

Expected:

```text
bare variants parse
singleton/zero-arg/payload/labeled target shapes remain distinct
impl target and &callable-reference syntax remain distinct
legacy compatibility, if temporarily present, is explicitly staged
```

Do not broaden after PASS.

---

## G2 — Enum semantic contract gate

**Purpose:** prove impl-origin root/case behavior preserves closed-enum semantics and GADT specialization.

Run the exact new tests first, then:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
```

Also run the P1 focused effective-surface test filter if root enum behavior now flows through that surface.

Expected:

```text
root requirement/default classification correct
case witnesses use Variant-owned CallableId
missing/incompatible diagnostics preserved
GADT specialization applied before compatibility
payload fields available in case body
case-only member absent from root
class-side/bodyless boundaries enforced
```

Do not broaden after PASS.

---

## G3 — Differential migration and executable dispatch gate

**Purpose:** prove source relocation did not change semantic or runtime behavior before legacy source deletion.

Run:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
```

plus exact differential semantic tests added for old/new source forms.

Expected:

```text
old/new canonical enum behavior products equivalent for selected migration fixtures
root defaults execute on all cases
case override shadows root default
case-only behavior dispatches only on target case
payload behavior works
requirements have no runtime body installation
```

Record PASS before deleting transitional legacy support.

---

## G4 — Incremental/source identity gate

**Purpose:** prove the new source organization does not corrupt incremental ownership or editor identity.

Run exact affected filters under:

```text
phalcom-semantic --test semantic incremental::<live-module>
phalcom-semantic --test semantic integration::<source-index-live-module>
```

Use `-- --list` first if names drift.

Required scenarios:

```text
add variant -> requirement completeness invalidated
edit root requirement signature -> case statuses recomputed
edit one case body -> unrelated structural enum products preserved
case-only member add/remove -> root surface unchanged
cold vs incremental diagnostics and identities match
```

Do not run full LSP unless a focused integration failure requires it.

---

## G5 — Final focused stabilization gate

Run serially:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test enum_syntax
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
cargo fmt --all -- --check
```

Also run:

- exact P1 impl/effective-surface focused tests;
- exact P2 incremental/source-index tests from G4;
- exact native/Universe test only if that subsystem changed materially.

**Do not automatically run:** workspace test, workspace clippy, workspace build, full language corpus.

Expected final classification:

```text
P2 source: IMPLEMENTED
P2 verification: FOCUSED_TESTED
C2 checkpoint: still IN_PROGRESS because P3 remains
```

---

## 20. Final focused acceptance

At completion, the walkthrough must include a table with every coverage ID and concrete evidence:

| Coverage ID | Test/evidence | Result |
|---|---|---|
| CV-01 | exact parser test | PASS |
| CV-02 through CV-29 | each remaining coverage obligation mapped to concrete focused evidence | PASS |
| CV-30 | representative existing ADT regression | PASS |

No row may be silently omitted. Several rows may point to one high-value test.

### Release-complete criteria for P2

P2 is complete only when:

- canonical enum syntax is variants-only;
- root/case behavior is authored through impl;
- root requirements/defaults retain exact old semantic meaning;
- exact-case behavior has canonical variant ownership;
- GADT/variant-local generic specialization remains authoritative;
- case-only lookup stays case-only;
- compiler uses semantic impl lowering;
- runtime topology remains root + hidden case classes with no reopen semantics;
- legacy behavior-bearing enum AST/parser/consumer paths are gone;
- focused AST/semantic/runtime/incremental evidence passes;
- checkpoint/walkthrough/handoff are current;
- no unresolved A/B-classified P2 failures remain.

This does **not** mean C2 or LANG005 is release certified.

---

## 21. Performance/resource evidence

P2 is not primarily a performance plan. Do not add benchmark ceremony.

Still protect these architecture-level cost invariants:

```text
no per-send impl scan
no per-send enum requirement scan
no runtime source-fragment lookup
no per-value requirement/witness metadata
no cloned root-default callable per variant
no module-wide recomputation for a case-body-only edit where P1 query granularity can avoid it
```

If implementation introduces an obvious new O(number-of-impls) send-time or lookup-time path, treat that as an architectural failure and consult.

---

## 22. Checkpoint bookkeeping

### At plan start

- verify/create the canonical C2 checkpoint record according to current repository workflow;
- mark `LANG005.C2.P2` active;
- record starting revision;
- record exact landed P1 interface names/paths.

### During plan

Update the checkpoint only for:

- durable new target/product identity;
- semantic architecture deviation approved by consultation;
- coherent verification gate completion;
- deferred baseline failure;
- material compatibility decision such as temporary `@variant` acceptance.

Do not turn the checkpoint into an edit log.

### At plan completion

Record:

```text
P2 status/completion/verification
final target/product interfaces
legacy behavior path removal status
@variant compatibility status
focused gates and results
deferred failures
consultations/amendments
next action = LANG005.C2.P3 planning/implementation takeover
```

---

## 23. Walkthrough deliverable

Create:

```text
LANG005.C2.P2-walkthrough.md
```

under the canonical C2 implementation directory.

It must contain:

- final result;
- implemented source syntax;
- exact semantic target/product architecture;
- how `EnumBehaviorProduct` is now populated;
- requirement/default/witness identity relationships;
- GADT/variant-local generic handling;
- compiler lowering target and runtime installation path;
- important code changes by subsystem;
- legacy path removed;
- `@variant` compatibility status;
- implementation deviations from this plan;
- consultations and resulting decisions;
- tests added;
- tests actually run;
- tests intentionally deferred;
- coverage ID -> evidence table;
- verification classification;
- residual risks relevant to P3/C3/C7.

---

## 24. Handoff deliverable

Create:

```text
LANG005.C2.P2-handoff.md
```

It must contain:

- current revision and worktree state;
- stable P1+P2 interfaces;
- exact `InherentImplTarget`/repository-equivalent target model;
- enum root/case surface and contract ownership;
- durable invariants needed by P3;
- next objective: constrained/specialized inherent impl applicability;
- must-read files;
- focused test commands;
- deferred failures;
- any compatibility syntax still retained for C8;
- explicit “do not redesign” list:
  - closed enum requirements are not traits;
  - exact-case impls are not the P3 specialization algorithm;
  - `VariantId` is exact-case target identity;
  - runtime `ClassId` is execution identity only;
  - P1 effective root surface remains canonical ordinary lookup;
  - GADT case environment remains specialization authority.

---

## 25. Completion truth table

Do not conflate:

```text
source written
!=
tests added
!=
targeted tests passed
!=
affected focused suites passed
!=
P2 accepted
!=
C2 checkpoint accepted
!=
LANG005 release certified
```

Expected successful P2 metadata:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

C2 remains in progress until P3 and checkpoint-level acceptance are complete.

---

## 26. Plan self-review

### Architecture

- [x] One canonical owner exists for enum structure: enum declaration semantics.
- [x] One canonical owner exists for source impl provenance: P1 `ImplId`.
- [x] Exact-case target identity is explicit: `VariantId`.
- [x] Requirement identity remains separate from witness callable identity.
- [x] Existing GADT case environment remains specialization authority.
- [x] Compiler/runtime contracts are explicit and fail closed.
- [x] Runtime representation is not promoted into semantic identity.
- [x] Transitional legacy support has an explicit deletion task.

### Luna executability

- [x] P1 takeover gate prevents implementation on the wrong baseline.
- [x] Fixed/flexible/verify-first decisions are explicit.
- [x] Every major task has a coherent owner and acceptance condition.
- [x] Architectural ambiguities trigger consultation before speculative coding.
- [x] Mechanical and semantic debugging loops are bounded.

### Testing

- [x] Coverage spans syntax, identity, GADT specialization, defaults, requirements, case-only behavior, runtime dispatch, incrementality, and migration equivalence.
- [x] Coverage obligations map to numbered invariants.
- [x] BUILD testing is intentionally narrow.
- [x] Workspace test/clippy/build are explicitly deferred.
- [x] Baseline failures have a classification policy.

### Documentation lifecycle

- [x] Normative spec drift is repaired.
- [x] Checkpoint state is durable.
- [x] Walkthrough and handoff are mandatory.
- [x] T/G terminology is consistent.

### Premium-compute discipline

- [x] Likely architectural forks are pre-decided.
- [x] Consultation triggers request narrow decisions, not broad discovery.
- [x] P2 does not require premium review for routine Rust/compiler cleanup.

### Final implementation principle

The implementer should be able to summarize P2's architecture in one sentence:

> **Enums own their closed structural case universe; `impl` owns the source fragments of behavior; semantic enum products classify root requirements/defaults and exact-case implementations; the compiler receives those resolved facts and installs bodyful callables onto the already-existing root/case runtime behavior classes.**
