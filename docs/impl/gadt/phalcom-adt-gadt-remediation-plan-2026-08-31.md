# Phalcom ADT / GADT / Associated Lookup — Completion & Remediation Implementation Plan

> **Repository:** `aureat/phalcom-lang`  
> **Verified baseline:** `main` @ `347ffedf94c570c18c5589ac1dbf98549f9224cb`  
> **Baseline commit:** `feat: implement Part 06 core integration, reflection, and tooling for ADTs`  
> **Includes landed work:** Parts 05.2 and 06  
> **Purpose:** Fix the remaining correctness defects found in the ADT/GADT review, reconcile the associated-lookup implementation with the authoritative `::` semantics, complete enum behavior/requirements, harden executable lowering/runtime boundaries, and close the remaining semantic gaps without regressing Part 05.2 or Part 06.

---

## 0. Executive Summary

The ADT/GADT implementation is now broad, but it still has several correctness holes at important integration boundaries. Part 05.2 has successfully introduced executable pattern projection and match lowering, and Part 06 has added native core ADT identity, representation seams, reflection metadata, and tooling targets. Those additions do **not** resolve the remaining semantic pipeline defects identified by the review.

The highest-priority remaining problems are:

1. **Closed-enum requirements are still a production no-op.** The checker exists and has direct tests, but `SemanticSession` still calls it with `&[]` requirements and an empty case-method map, then publishes empty requirement products.
2. **Enum behavior is not a canonical semantic product.** Root behavior is not published like class behavior; case behavior is not integrated into the semantic signature/body-query pipeline; codegen independently recompiles a partial subset of enum behavior.
3. **Associated members and ordinary `::` behavior are conflated.** Current associated surfaces mix variants with `@class` methods and the resolver probes class-side and instance-side dispatch surfaces. That contradicts the intended model.
4. **The canonical `::` model must be preserved.** `::` remains receiver-bound deferred behavioral dispatch on instances, objects, and class objects. Associated lookup is an additional, higher-priority declaration-owned namespace layered on top of that mechanism.
5. **Record/map patterns are still semantically treated as wildcard.** Part 05.2 can lower `Record` and `Map`, but Part 05.1 semantic resolution still falls through to wildcard, making exhaustiveness/usefulness unsound.
6. **Pattern-space intersection/subtraction is not specialization/proof-safe for GADTs.** Same `VariantId` is currently treated as sufficient compatibility and proof bindings can overwrite each other silently.
7. **Several fail-open lowering/runtime paths remain.** Missing variant metadata can become `Singleton`; missing binding identity can become binding index 0; several arity/slot conversions narrow with unchecked casts.
8. **GADT record equality is incomplete and substitution normalization has an arbitrary 64-pass ceiling.**
9. **Runtime declaration resolution can bind the wrong module by leaf class name.**
10. **Standalone enum lowering still reconstructs selectors incorrectly and case behavior attaches by base instead of exact `VariantId`.**

This plan fixes those issues in dependency order. It deliberately avoids turning the remediation into another redesign of the whole compiler.

---

# 1. Authoritative Semantic Invariants

These invariants are prerequisites for every task below. Code and tests should encode them directly.

## 1.1 `::` remains receiver-bound deferred behavioral dispatch

The existing behavior remains valid:

```phalcom
instance::bar
instance::bar::(...)

Foo::bar
Foo::bar::(_)
Foo::bar::()
```

Meaning:

- the receiver expression is evaluated and captured;
- exact selector or selector-family identity is statically known;
- because Phalcom does not permit monkey-patching declaration membership, the compiler can statically determine the relevant declared candidate surface;
- invocation remains bound to the captured receiver and follows ordinary behavioral dispatch semantics;
- a class object is still a receiver; its ordinary behavior is the class-object behavior declared with `@class` on the corresponding class declaration.

Therefore `Foo::bar` is **not** syntax for “search the declaration `Foo` for an associated item named `bar`” unless `Foo` actually exposes an associated base `bar`.

## 1.2 Associated members are not behavior

A class/type declaration may expose declaration-owned associated members. Enum variants are the first important associated-member category.

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

Option::Some::(_)   // exact shape from associated variant-constructor family
Option::None        // associated singleton variant value
```

Associated members:

- are not methods/getters/setters/indexers;
- do not participate in ordinary behavioral inheritance/dispatch;
- live in a declaration-owned associated namespace;
- are resolved before behavioral `::` fallback;
- reserve their **entire selector base** in the declaration.

## 1.3 Associated base reservation is total within the declaration

If a declaration exposes associated base `Some`, no behavior declared in that same declaration may use base `Some`, regardless of exact selector shape or dispatch side.

Illegal:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)

    @class
    Some(_ left: T, _ right: T) {
        ...
    }
}
```

The conflict is not resolved by overload shape. `Some` is already reserved as an associated base.

The same rule applies to ordinary instance behavior declared in the same declaration: associated-base reservation is a declaration namespace rule, not merely a class-side rule.

## 1.4 Associated lookup has precedence, then ordinary bound-family behavior

Conceptually:

```text
resolve receiver::base/spec
    |
    +-- can the receiver be statically associated with a declaration exposing base?
    |       |
    |       +-- yes -> resolve ONLY in that associated family/member
    |       |          (no behavioral overload merge)
    |       |
    |       +-- no  -> ordinary receiver-bound :: behavior resolution
    |
    +-- invocation preserves the semantics of the chosen category
```

An associated base shadows/preempts behavioral `::` fallback for that base because coexistence in the same declaration is forbidden and the associated namespace has precedence.

## 1.5 `@class` is the source spelling for class methods

All documentation, diagnostics, tests, examples, and implementation comments in this remediation must use `@class` terminology for class-side behavior. Do not introduce a `static` keyword into language examples.

Internal Rust compatibility fields such as `is_static` may remain until separately refactored; they are not normative language syntax.

## 1.6 Semantic ownership remains upstream

The architecture must stay:

```text
parse
  -> canonical semantic declaration products
  -> canonical enum behavior + requirements + associated namespace
  -> canonical type/match/GADT proofs
  -> lossless executable projection
  -> mechanical compiler/VM execution
```

Do not:

- reconstruct semantic selectors from source strings in `phalcom-core`;
- make `session.rs` a second independent AST-to-signature parser;
- make the VM probe class/instance sides to guess semantic intent;
- replace missing semantic types with arbitrary `Unit` or `Object` values;
- recover missing variant metadata by assuming singleton shape.

---

# 2. Current-Main Finding Matrix

| ID | Finding | Current status after 05.2/06 | Severity | Primary files |
|---|---|---|---|---|
| R-01 | Production closed-enum requirements wired with empty inputs | **Still present** | Critical | `phalcom-semantic/src/session.rs`, `enum_requirements.rs` |
| R-02 | Root/case enum behavior lacks canonical semantic publication | **Still present** | Critical | `checker/enum_declaration.rs`, `session.rs`, new `checker/enum_behavior.rs` |
| R-03 | Core enum compiler only implements partial case behavior and no root behavior | **Still present** | Critical | `phalcom-core/src/compiler/lib/enum_decl.rs`, `class_decl.rs` |
| R-04 | Case behavior attaches by selector base, not exact `VariantId` | **Still present** | Critical | `phalcom-core/src/compiler/lib/enum_decl.rs` |
| R-05 | Standalone enum fallback synthesizes wrong selector labels | **Still present** | High | `phalcom-core/src/compiler/lib/enum_decl.rs` |
| R-06 | Associated surface conflates variants and behavioral methods | **Still present** | Critical architecture | `phalcom-semantic/src/associated.rs`, `session.rs`, `checker/associated.rs` |
| R-07 | Associated conflict emits diagnostic but still publishes mixed invalid family | **Still present** | High | `phalcom-semantic/src/associated.rs` |
| R-08 | Associated resolver probes both class and instance dispatch sides | **Still present** | High | `phalcom-semantic/src/checker/associated.rs` |
| R-09 | Unknown associated constructor parameter becomes `Unit`; behavioral unknown can become `Object` | **Still present** | High | `checker/associated.rs` |
| R-10 | Record/map semantic patterns fall through to wildcard | **Still present** | Critical | `checker/pattern.rs` |
| R-11 | Pattern-space algebra has no Record/Map representation | **Still present** | Critical | `checker/pattern_space.rs`, `match_semantics.rs` |
| R-12 | Variant-space compatibility ignores exact specialization/proof conflicts | **Still present** | High | `checker/pattern_space.rs`, `checker/gadt_proof.rs` |
| R-13 | Closed-enum initial space silently drops missing variant metadata | **Still present** | High | `checker/exhaustiveness.rs` |
| R-14 | GADT equality does not unify record rows | **Still present** | High | `checker/gadt_proof.rs`, `types/row.rs` |
| R-15 | GADT substitution normalization uses 64-pass ceiling | **Still present** | Moderate/High | `checker/gadt_proof.rs` |
| R-16 | Lowering missing enum variant metadata defaults to singleton | **Still present** | High | `phalcom-core/src/modules/semantic_lowering.rs` |
| R-17 | Lowering missing pattern binding defaults to index 0 | **Still present after 05.2** | High | `semantic_lowering.rs` |
| R-18 | Rest metadata erased in executable behavioral target | **Still present** | High | `semantic_lowering.rs` |
| R-19 | Unchecked arity/slot/discriminant narrowing | **Still present** | Moderate/High | `semantic_lowering.rs`, `enum_decl.rs`, `vm/adt.rs` |
| R-20 | VM declaration resolution can fall back by global leaf class name | **Still present** | High | `phalcom-core/src/vm/associated.rs` |
| R-21 | VM “associated behavioral target” probes metaclass then class | **Still present; model is wrong** | High | `vm/associated.rs` |
| R-22 | Part 05.2 match lowering/executable patterns | **Landed — preserve** | Gate | `semantic_lowering.rs`, `compiler/lib/match_expr.rs`, `patterns.rs` |
| R-23 | Part 06 native core ADTs / reflection / tooling targets | **Landed — preserve** | Gate | core-surface/reflection/tooling files |
| R-24 | Union normalization `flat.contains` is O(n²) | **Still present, perf-only** | Low | `checker/pattern_space.rs` |

---

# 3. Dependency Graph and Implementation Order

The implementation order is intentional:

```text
Task 0  Freeze corrected semantics/spec
   |
   v
Task 1  Canonical enum-behavior source->semantic builder
   |
   +--> Task 2  Publish root/case behavior and real requirements
   |       |
   |       +--> Task 3  Validate defaults/overrides/case legality
   |       |
   |       +--> Task 4  Build clean associated namespace + base reservation
   |                    |
   |                    v
   |               Task 5  Layer associated-first :: over ordinary bound Family
   |                    |
   |                    v
   |               Task 6  Correct associated specialization/type knowledge
   |
   +--> Task 10  Compile full enum behavior from semantic identities

Task 7  Complete Record/Map semantic patterns
   |
   v
Task 8  Specialization/proof-safe pattern-space algebra
   |
   v
Task 9  Fail-closed exhaustiveness + GADT row/proof hardening
   |
   v
Task 11  Harden semantic->executable projection
   |
   v
Task 12  Runtime cleanup + exact declaration identity
   |
   v
Task 13  Part 05.2 / Part 06 integration-preservation suite
   |
   v
Task 14  Documentation, coverage ledger, full verification
```

Do not start by patching `phalcom-core`. The semantic products must be corrected first so core can consume them mechanically.

---

# Task 0 — Ratify the Correct `::` + Associated Namespace Contract

**Goal:** Remove contradictory specification language before code changes so implementers do not “fix” the wrong semantics again.

## Files

Modify:

```text
docs/spec/current/selectors.md
docs/spec/adts.md
```

Audit and amend only where they are treated as active design inputs:

```text
docs/impl/adt-gadt-associated-lookup/part-2/*
docs/impl/adt-gadt-associated-lookup/part-3/*
docs/impl/adt-gadt-associated-lookup/part-4/*
```

Do not rewrite historical plans wholesale. Add a clear “superseded semantic clarification” where necessary.

## Edits

### 0.1 Update `docs/spec/current/selectors.md`

Keep the existing bound-family law, but add associated precedence:

```text
receiver::selector-spec
    1. evaluate receiver once
    2. if the receiver denotes/is a declaration-backed class object whose declaration
       exposes an associated member at this selector base, resolve associated member
    3. otherwise construct ordinary bound behavioral Family/reference
```

Clarify that:

- class objects are ordinary receivers;
- `Foo::bar` on ordinary behavior refers to behavior callable on `Foo`, i.e. behavior declared with `@class` on `Foo`;
- associated lookup is not inherited behavioral dispatch;
- whole base reservation prevents behavioral/associated overload mixing;
- exact selector/family-pattern syntax remains unchanged.

### 0.2 Update `docs/spec/adts.md`

Replace wording such as:

```text
Variant construction and lookup live on the static associated `::` surface.
```

with language equivalent to:

```text
Enum variants are declaration-owned associated members exposed through `::`.
Associated lookup has precedence over ordinary receiver-bound `::` behavioral family
resolution at a reserved associated base. Outside an associated base, `::` retains its
ordinary receiver-bound deferred-dispatch semantics.
```

Add the reservation example with `@class`, not `static`.

### 0.3 Make namespace terminology explicit

Use three separate concepts consistently:

```text
Behavior surface
    instance-side or @class-side callable behavior

Associated surface
    declaration-owned associated members, currently variants

Bound Family
    receiver-bound deferred behavioral dispatch value
```

Never call an `@class` method an “associated behavioral member”.

## Regression/contract tests

No runtime code should change in this task, but add/adjust parser/spec fixtures if the test suite has documentation examples. The semantic tests introduced in Tasks 4–6 are the executable conformance tests for this contract.

## Exit gate

Search must find no normative example in the ADT implementation docs that uses a `static` source keyword for a class method.

Suggested check:

```bash
rg -n '\bstatic\b.*Some|Some.*\bstatic\b' docs/spec docs/impl/adt-gadt-associated-lookup
```

Review false positives referring to Rust/internal storage separately.

---

# Task 1 — Introduce One Canonical Enum-Behavior Semantic Builder

**Goal:** Make enum behavior use the same source-to-semantic signature machinery as classes and establish stable root/case callable identities before session integration.

## Files

Create:

```text
phalcom-semantic/src/checker/enum_behavior.rs
```

Modify:

```text
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/identity.rs              # only if helper constructors are missing
phalcom-semantic/src/signature.rs             # only if product grouping type belongs here
phalcom-semantic/src/enum_requirements.rs      # product consumption only; avoid duplicate parsing
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/behavior.rs
phalcom-semantic/tests/semantic/adts/requirements.rs
```

## Design

Add a canonical enum behavior product, conceptually:

```rust
pub struct EnumBehaviorProduct {
    pub owner: DeclarationId,
    pub root_defaults: Box<[CallableSemanticSignature]>,
    pub root_requirements: Box<[EnumRequirement]>,
    pub case_implementations: BTreeMap<VariantId, Box<[CallableSemanticSignature]>>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

Exact concrete container types may follow existing snapshot conventions (`Arc<[T]>`, `BTreeMap`, etc.). The important invariant is that this is a **semantic product**, not a core/compiler reconstruction.

### 1.1 Extract reusable callable syntax adapters from `declaration_signature.rs`

Current `declaration_signature.rs` is explicitly documented as the one source-to-semantic boundary for callable declarations. Preserve that role.

Refactor private helpers so both `ClassMember` and `EnumBehaviorMember` can reuse:

```text
selector construction from method/getter/setter/index syntax
parameter semantic construction
return annotation fact construction
method generic signature resolution
rest mode preservation
source-span publication
initial return-validation state
```

Preferred structure:

```rust
fn semantic_signature_from_parts(
    ctx: &mut CheckingContext<'_>,
    callable: CallableId,
    syntax: CallableSyntaxRef<'_>,
    declaration_owner: &DeclarationId,
) -> CallableSemanticSignature
```

or a small internal adapter enum:

```rust
enum CallableSyntaxRef<'a> {
    Method(&'a MethodDef),
    Getter(&'a GetterDef),
    Setter(&'a SetterDef),
    Index(&'a IndexMethodDef),
}
```

Do **not** duplicate parameter/return/generic resolution in `session.rs`.

### 1.2 Establish correct callable identities

Root enum behavior:

```text
CallableOwnerId::Declaration(enum DeclarationId)
DispatchSide::Instance for ordinary root behavior
DispatchSide::Class only when root behavior explicitly has @class
```

Case-local behavior:

```text
CallableOwnerId::Variant(exact VariantId)
DispatchSide::Instance only in v1
```

Do not key case behavior by variant base string.

### 1.3 Classify root bodyful vs signature-only behavior

For each `EnumMember::Behavior`:

```text
body exists       -> root default/shared behavior
body absent       -> closed-enum requirement
```

If a root signature-only member is marked `@class`, decide according to the existing Part-2 rule. The currently ratified ADT design treats closed-enum requirements as instance behavior contracts; reject unsupported class-side requirement forms with a precise diagnostic rather than silently treating them as ordinary behavior.

### 1.4 Classify case-local behavior

For each `VariantBody::members` item:

- reject `@class`;
- reject declaration-only/no-body form;
- otherwise publish exact variant-owned instance callable signature.

Use existing diagnostics if present:

```text
EnumCaseStaticBehaviorUnsupported
EnumCaseDeclarationOnlyBehavior
```

If names differ on current main, reuse the current diagnostic-code vocabulary rather than adding near-duplicates.

## Tests — write first

Add direct builder tests:

1. root bodyful getter becomes root default, not requirement;
2. root signature-only getter becomes requirement;
3. root bodyful method preserves labels/rest/generics/return annotation;
4. case method owner is exact `VariantId`, not enum owner;
5. two same-base variant shapes get distinct case callable owners;
6. case getter/method/setter/index all publish signatures;
7. case `@class` produces the intended diagnostic and no executable signature;
8. case declaration-only member produces diagnostic and no executable signature;
9. source ranges on signatures/diagnostics point to the enum member, not enum root;
10. no `Unit`/`Object` substitution is introduced for unresolved annotations.

## Exit gate

`EnumBehaviorProduct` can be built from source independently of `SemanticSession`, and every callable signature is produced through the canonical declaration-signature machinery.

---

# Task 2 — Wire Enum Behavior and Closed Requirements into `SemanticSession`

**Goal:** Replace the current empty requirement pipeline with actual source-derived products and make enum callables visible to semantic analysis before body checking.

## Files

Modify:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/product.rs             # if new DB product required
phalcom-semantic/src/db/query.rs               # if a dedicated query is appropriate
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/dispatch.rs               # only for projection helpers if needed
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/behavior.rs
phalcom-semantic/tests/semantic/adts/requirements.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
```

## 2.1 Build enum structure first, enum behavior second

Keep structural enum metadata construction in `checker/enum_declaration.rs` focused on:

```text
EnumInfo
VariantInfo
VariantId / VariantFieldId / VariantConstructorId
case environments
exact-case templates
visibility
```

After `EnumDeclarationProduct` exists and variants are inserted into `EnumSemanticTable`, call the new enum-behavior builder with the exact variant table available.

### 2.2 Publish root default signatures into the semantic signature/dispatch world

Root bodyful behavior must be visible like ordinary declaration-owned behavior:

- add `CallableSemanticSignature` to `callable_signatures`;
- project it into an enum root `DeclarationSurface` / dispatch surface;
- register enum root type -> declaration in dispatch if not already registered;
- ensure case runtime classes can inherit the root behavior later.

Do not implement inherited defaults by copying each root callable into every case semantic table. The root class is the shared default owner.

### 2.3 Publish case implementation signatures

Case behavior signatures must be retained under exact variant-owned `CallableId`s.

If `CallableSignatureTable` is currently indexed only by declaration-owned callables, extend it cleanly to accept `CallableOwnerId::Variant`. Do not flatten the identity back to the enum declaration.

### 2.4 Replace empty requirement invocation

Delete the current effective no-op inputs:

```rust
&[]
&HashMap::new()
Arc::from([])
```

Pass:

```text
root_requirements = EnumBehaviorProduct.root_requirements
case_methods      = EnumBehaviorProduct.case_implementations
```

Publish those same requirements to:

```text
EnumRequirementTable
EnumRequirementsProduct
SemanticSnapshot
```

The stored product and the checked inputs must be identical semantic objects/identities, not separately reconstructed copies.

### 2.5 Ensure body queries can analyze enum callables

Current workspace body-query loop in `session.rs` only iterates `Statement::Class` members. Extend body-query scheduling to enum behavior:

```text
root bodyful callable -> query body with enum declaration owner context
case bodyful callable -> query body with exact VariantId owner context and case proof/type environment
```

Case body checking should make payload fields and case-specialized types available as designed by the ADT semantic model.

Prefer introducing a generic “source callable body job” iterator rather than adding another large duplicate body-query loop.

## Regression tests — source/session level, not direct helper only

These tests are mandatory because the current bug survived direct checker tests.

1. source enum root requirement appears in `snapshot.enum_requirements.requirements`;
2. source enum case implementation appears in `case_statuses` as `Satisfied`;
3. missing source case implementation emits `EnumRequirementMissing` through normal session analysis;
4. incompatible return emits `EnumRequirementIncompatible`;
5. incompatible parameter label/shape/rest emits incompatibility;
6. GADT-specialized requirement (`Expr<T>`, case `Expr<Int>`) succeeds after case substitution;
7. root bodyful default is **not** added to requirement table;
8. root bodyful default is callable on every case through semantic dispatch;
9. root and case callable analyses appear in snapshot with stable `CallableId`s;
10. incremental re-analysis changes only affected enum callable products when a case method changes.

## Exit gate

A test must fail if `session.rs` ever goes back to passing empty requirements/case maps.

---

# Task 3 — Complete Enum Default/Override Semantics and Legality Checks

**Goal:** Make root defaults, case overrides, and closed requirements coherent and prevent invalid case behavior from reaching core.

## Files

Modify:

```text
phalcom-semantic/src/checker/enum_behavior.rs
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/surface.rs / dispatch.rs       # if override projection support required
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/behavior.rs
phalcom-semantic/tests/semantic/adts/requirements.rs
```

## 3.1 Root bodyful behavior is inherited default behavior

Publish root callable once on enum root instance surface.

A case with no override inherits it.

A case with an exact-selector override replaces normal dispatch for that case runtime class, exactly like subclass override behavior, but the case remains closed to external subclassing/variant extension.

### 3.2 Validate case overrides against the root default when an exact selector matches

At minimum validate:

```text
selector identity
rest mode
parameter types / labels
return compatibility
```

Reuse ordinary override-compatibility machinery if available. Do not create a weaker enum-specific notion.

### 3.3 Root requirement and root default cannot be duplicate exact declarations

If source grammar allows duplicate root members with identical selector, use existing duplicate-member diagnostics. A signature-only requirement and bodyful default with the exact same selector are competing declarations, not “requirement + implementation on root”.

### 3.4 Case-local `@class` rejection

Case behavior is instance-side only in v1. Reject:

```phalcom
@variant Some(_ value: T) {
    @class
    make(...) { ... }
}
```

The case body must not create a metaclass/class-side method table.

### 3.5 Case-local declaration-only rejection

Reject before body query/core lowering. No empty closure should be synthesized for a missing body.

## Regression tests

1. inherited root getter works for singleton and constructor cases;
2. inherited root method works for all variants;
3. exact case override wins over root default;
4. unrelated selector on case coexists with root behavior;
5. incompatible case override is diagnosed;
6. `@class` case method rejected;
7. declaration-only case getter/method rejected;
8. setter/index override shapes checked;
9. root requirement is satisfied only by case implementation, not merely a root default with another selector shape;
10. GADT-specialized override return compatibility is checked in the exact case environment.

---

# Task 4 — Separate Associated Namespace from Behavior and Enforce Base Reservation

**Goal:** Stop representing `@class` methods as associated members. Make the associated surface a declaration-owned namespace containing only genuine associated declarations.

## Files

Modify substantially:

```text
phalcom-semantic/src/associated.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/checker/associated.rs
```

Potential identity cleanup:

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/types/family.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/associated/declarations.rs
phalcom-semantic/tests/semantic/adts/associated/resolution.rs
phalcom-semantic/tests/semantic/adts/behavior.rs
```

Use the existing `associated/` test directory structure rather than adding a `part3_remediation.rs` file.

## 4.1 Replace mixed associated-member model

Current model:

```rust
AssociatedFamilyKind::{Behavioral, Variant}
AssociatedMemberId::{Behavioral(CallableId), Variant(VariantId)}
```

Target direction:

```rust
pub enum AssociatedMemberId {
    Variant(VariantId),
    // future genuine associated categories here
}
```

and family kind can either remain as an extensible associated-category enum or become variant-specific for now. The key rule is: **ordinary behavior is absent from `AssociatedSurface`.**

Do not add `@class` callables to `AssociatedSurface` merely because the receiver can be a class object.

### 4.2 Change `build_associated_surface`

Inputs should represent:

```text
explicit associated declarations
behavior bases declared in same declaration (for conflict validation only)
possibly inherited associated namespace according to future rules, but do not infer ordinary behavior
```

For enums today:

```text
associated members = variants
```

For ordinary classes without explicit associated declarations:

```text
associated members = empty
```

### 4.3 Enforce entire-base reservation

Build a set of behavior bases declared in the same declaration from **all** ordinary callable behavior:

```text
instance methods/getters/setters/indexers
@class methods/getters/setters/indexers
```

If `associated_bases ∩ behavior_bases` is non-empty:

- emit one clear declaration-site diagnostic per conflicting base;
- attach labels to both declarations when source spans exist;
- do not publish a mixed family;
- ideally poison/omit the associated family for that declaration so downstream code cannot accidentally consume an invalid semantic object.

Use an existing diagnostic if it accurately expresses the rule; otherwise replace `EnumFamilyCategoryConflict` with a more general associated-base reservation diagnostic and keep a compatibility alias only if required.

### 4.4 Remove class-callable associated surface population from `session.rs`

Delete the pass that currently collects class callables with `DispatchSide::Class` and feeds them to `build_associated_surface` as associated behavior.

Class methods remain in ordinary declaration dispatch surfaces and are available to `Foo::...` through receiver-bound behavioral family semantics.

### 4.5 No mixed invalid family publication

Delete the current behavior that diagnoses variant+behavior conflict and then creates `AssociatedFamilyInfo { kind: Variant, members: [Variant..., Behavioral...] }`.

Invalid namespace state must never be published as valid.

## Regression tests

1. `Option` associated surface contains `Some` and `None` variants only;
2. `@class format(...)` on an ordinary class does **not** appear in `AssociatedSurface`;
3. `Foo::format...` remains resolvable through ordinary bound behavioral family resolution (Task 5);
4. enum variant `Some` + `@class Some(_,_)` is rejected by base reservation;
5. enum variant `Some` + instance getter/method `Some` is also rejected;
6. conflict occurs even when selector shapes differ;
7. two variants with same base but distinct legal constructor shapes remain members of the **same associated variant family**;
8. invalid conflict publishes no mixed family;
9. unrelated `@class parse` and associated `Some` coexist;
10. reservation diagnostics are deterministic independent of source map iteration order.

---

# Task 5 — Implement Layered `::`: Associated First, Bound Behavioral Family Otherwise

**Goal:** Reconcile the Part-3 associated resolver with the pre-existing receiver-bound Family semantics without changing either category’s meaning.

## Files

Semantic:

```text
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/types/denotation.rs
phalcom-semantic/src/types/family.rs
phalcom-semantic/src/match_semantics.rs              # only if target identity touches tooling products
```

Core/lowering after semantic form is stable:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/vm/associated.rs
# existing Family runtime/compiler files used by ordinary :: behavior
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/associated/resolution.rs
phalcom-semantic/tests/semantic/adts/associated/families.rs
phalcom-core/tests/core/language/families.rs         # or existing family test module
phalcom-core/tests/core/language/adts.rs              # follow current module naming
```

## 5.1 Introduce an explicit resolution category

Do not overload “AssociatedResolution” to mean every `::` expression.

Preferred semantic product:

```rust
enum DoubleColonResolution {
    Associated(AssociatedResolution),
    BoundBehavioralFamily(BoundFamilyResolution),
}
```

or keep separate indices if the existing architecture already has an ordinary Family resolution product. The important point is that the category is explicit and downstream lowering never guesses from a `CallableId.side`.

### 5.2 Determine whether associated lookup applies without treating every receiver as a type form

Current `resolve_associated_owner` requires `SemanticDenotation::TypeForm`. Preserve a narrow declaration-backed associated resolver, but call it only when the receiver’s static denotation/type identifies a declaration that exposes an associated base.

Algorithm:

```text
1. synthesize/evaluate receiver semantic expression
2. determine requested selector base
3. ask: is there a statically known declaration-backed associated surface for this receiver/base?
4. yes -> associated resolution
5. no  -> ordinary bound Family/reference resolution using the original receiver
```

A class-object receiver such as `Option` can expose associated members because its semantic denotation identifies the declaration. A class-object receiver such as `Foo` with no associated `bar` falls through to normal class-object behavior and may resolve `@class bar`.

Do not search instance/class dispatch surfaces inside the associated resolver.

### 5.3 Preserve exact and structural family syntax

The following remain behavior-family forms when no associated base wins:

```phalcom
instance::bar
instance::bar::(...)
Foo::bar
Foo::bar::(_)
Foo::bar::()
```

Associated families use the same selector-spec grammar once the associated base is chosen:

```phalcom
Option::Some::(_)
```

but the selected members are variant constructors/values, not methods.

### 5.4 Preserve receiver capture for ordinary behavior

For the behavioral path:

- receiver is evaluated exactly once;
- Family stores/captures that receiver;
- invocation dispatches on the captured receiver according to the family’s exact/pattern selector contract;
- no conversion to “lookup owner class” occurs;
- no metaclass-then-class probing occurs.

### 5.5 Static candidate knowledge is not static invocation

Because declaration membership cannot be monkey-patched, semantic candidate shapes can be known statically. That does **not** mean core should invoke a frozen method object from the declaring class for ordinary bound-family dispatch.

Keep the distinction:

```text
candidate-set knowledge       compile-time fact
captured receiver             runtime value
ordinary method dispatch      invocation semantics
```

### 5.6 Associated exact resolution does not fall back once base is reserved

If associated base `Some` exists but exact shape `Some(_,_)` does not, report an associated-family shape error. Do not fall through to an ordinary behavioral method of the same base.

The declaration should already forbid such same-base behavior, but this fail-closed rule prevents recovery state from changing semantics.

## Regression tests

Behavior path:

1. `instance::bar` binds getter to instance;
2. `instance::bar::(...)` is a bound method-family pattern;
3. `Foo::bar` reaches `@class bar` behavior on class object `Foo` when no associated base exists;
4. `Foo::bar::(_)` and `Foo::bar::()` select the correct exact/pattern shapes;
5. receiver expression evaluates once;
6. subclass/ordinary dispatch behavior remains correct for captured receiver;
7. absence of associated base does not produce `AssociatedFamilyMissing`; it falls back to behavior.

Associated path:

8. `Option::None` returns singleton associated variant;
9. `Option::Some::(_)` identifies exact unary constructor shape;
10. overloaded variant family selects by exact selector shape;
11. associated base precedence prevents behavioral fallback;
12. missing associated shape reports associated-family error;
13. associated variant family is not represented as a behavioral Method Family at runtime.

Mixed/reservation:

14. source cannot declare behavior with reserved associated base;
15. invalid/recovered AST still cannot make resolver merge associated + behavior candidates.

---

# Task 6 — Make Associated Variant Specialization Preserve Type Knowledge

**Goal:** Remove fabricated `Unit`/`Object` types from associated member specialization and make underconstrained generic associated values fail/propagate intentionally.

## Files

Modify:

```text
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/types/family.rs
phalcom-semantic/src/declaration_type.rs
phalcom-semantic/src/types/evidence.rs             # only if a helper is needed
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/associated/generics.rs
phalcom-semantic/tests/semantic/adts/associated/families.rs
```

## 6.1 Constructor parameter specialization

Delete:

```rust
p.declared_type.canonical_type().unwrap_or_else(|| ctx.store.unit())
```

A missing declared type is not `Unit`.

Choose one explicit semantic state:

- preserve `DeclaredTypeFact` / `TypeKnowledge` in the specialized member representation; or
- if a concrete `CallableType` is mandatory at this stage, return a blocked/underconstrained associated resolution and retain the unknown reason in diagnostics/evidence.

Do not invent `Object` either unless an explicit dynamic/erasure boundary requires it.

### 6.2 Behavioral specialization moves out of associated resolver

Once Task 5 separates bound behavior from associated members, the associated specializer should no longer contain ordinary `AssociatedMemberId::Behavioral` specialization. Move any generally useful callable specialization helper into the ordinary family/dispatch semantic path.

### 6.3 GADT owner constraints use equality compatibility

The current code approximates equality by checking subtyping both directions. Prefer reusing the GADT equality/proof compatibility helper introduced in Tasks 8–9. A GADT case constraint is equality evidence, not nominal subtype coincidence.

## Regression tests

1. unannotated variant parameter remains unknown and never appears as `Unit`;
2. underconstrained generic constructor family emits/retains `AssociatedGenericUnderconstrained` as intended;
3. contextual expected callable type can solve an otherwise underconstrained variant constructor where supported;
4. GADT owner mismatch rejects constructor family member;
5. GADT exact owner match specializes parameter/result types;
6. no associated test expects ordinary `@class` behavior to appear as `AssociatedMemberId`.

---

# Task 7 — Complete Record and Map Pattern Semantics

**Goal:** Eliminate the wildcard fallback and make semantic pattern analysis match the executable/runtime behavior already landed in Part 05.2.

## Files

Modify:

```text
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/pattern_space.rs
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/exhaustiveness.rs
```

Part 05.2 projection should mostly remain mechanical:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/matching/record_patterns.rs
phalcom-semantic/tests/semantic/adts/matching/map_patterns.rs
phalcom-core/tests/core/language/matching.rs          # or current equivalent module
```

## 7.1 Make `resolve_pattern_with_mode` exhaustive

Replace the final wildcard catch-all:

```rust
_ => (PatternResolution::Wildcard, expected_space.clone())
```

with explicit `Pattern::Record` and `Pattern::Map` arms. Then let Rust exhaustiveness checking guard future AST additions.

Never use `_` for a semantic pattern enum when a new syntax form could accidentally become irrefutable.

### 7.2 Resolve record field child types

For `Pattern::Record`:

- inspect structural record type/row when available;
- resolve known field child pattern against its canonical field type;
- for dynamic/unknown field type, preserve unknown knowledge conservatively;
- reject impossible/missing required structural fields where the type proves absence;
- publish `PatternResolution::Record(Vec<ResolvedFieldPattern>)` using canonical field labels.

### 7.3 Resolve map entry child types

For `Pattern::Map`:

- preserve key identity exactly as supported by AST/runtime pattern emitter;
- obtain value type from map semantics when statically known, else conservative unknown/Object-like **knowledge**, not fabricated proof;
- publish `PatternResolution::Map` and child bindings.

### 7.4 Introduce sound refutable spaces

`Record` and `Map` patterns are refutable. Returning the whole `expected_space` is unsound.

Preferred representation:

```rust
PatternSpace::Record(RecordSpace)
PatternSpace::Map(MapSpace)
```

with structures containing required entries and child spaces.

If exact complement algebra for arbitrary dynamic maps is not representable, use a conservative residual form such as:

```rust
PatternSpace::OpaquePredicate {
    domain: TypeId,
    predicate: ...
}
```

with the crucial law:

```text
Opaque(domain) - refutable predicate != Empty
```

unless coverage is formally proven.

Do not claim exhaustive coverage from one record/map structural pattern merely because its child patterns are wildcards.

### 7.5 Keep Part 05.2 projection mechanical

`semantic_lowering.rs` already has:

```text
ExecutablePattern::Record
ExecutablePattern::Map
```

Once semantic resolution is correct, projection should simply lower the resolved entries. No AST reinterpretation belongs in core.

## Regression tests

Record:

1. record pattern is not emitted as `PatternResolution::Wildcard`;
2. record child binding receives field type;
3. failed record shape is refutable;
4. a lone record pattern does not falsely prove an opaque domain exhaustive;
5. record or-pattern binding joins remain coherent;
6. executable projection produces `ExecutablePattern::Record` with same entries.

Map:

7. map pattern is not wildcard;
8. child binding is published;
9. missing key leaves residual space;
10. map or-pattern alternatives work;
11. executable projection produces `ExecutablePattern::Map`;
12. Part 05.2 runtime staged binding leaves no leaked bindings after failed record/map alternative.

---

# Task 8 — Make Variant Pattern-Space Algebra Specialization and Proof Aware

**Goal:** Prevent false overlap, false redundancy, and false exhaustiveness for generic/GADT variants that share `VariantId` but have incompatible exact specializations or branch equalities.

## Files

Modify:

```text
phalcom-semantic/src/checker/pattern_space.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/match_semantics.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/adts/matching/gadt.rs
phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs
phalcom-semantic/tests/semantic/adts/exact_cases.rs
```

## 8.1 Extract a proof compatibility/merge API

Introduce a helper with semantics like:

```rust
pub(crate) enum ProofMerge {
    Compatible(BranchProofEnvironment),
    Contradictory,
}

pub(crate) fn merge_branch_proofs(
    store: &mut TypeStore,
    left: &BranchProofEnvironment,
    right: &BranchProofEnvironment,
) -> ProofMerge
```

Use equality solving; never merge maps by blind `insert` overwrite.

If the same parameter is constrained to incompatible canonical types, result is contradictory.

### 8.2 Exact-case compatibility is part of variant compatibility

For `VariantSpace(v1) ∩ VariantSpace(v2)` require:

```text
same VariantId
AND compatible ExactCase enum specializations
AND compatible branch proof environments
AND compatible field spaces
```

Examples:

```text
ExactCase(Some, Option<Int>)
∩ ExactCase(Some, Option<String>)
= Empty
```

unless type variables/equalities make them unifiable.

### 8.3 Subtraction uses the same relation

If the right variant specialization is disjoint, subtraction returns the left unchanged.

If right fully covers compatible specialization + fields, subtraction can remove it.

Do not use a weaker compatibility rule in subtraction than intersection.

### 8.4 Preserve proof identities in result

When compatible, merged proof must retain all non-duplicate equalities deterministically. Sort/deduplicate using canonical constraint identity rather than insertion order.

## Regression tests

1. same `VariantId`, `Option<Int>` vs `Option<String>` do not intersect;
2. a generic parameter proof that unifies to `Int` intersects `Option<Int>`;
3. contradictory proof bindings return Empty instead of overwriting;
4. subtraction of disjoint specialization leaves left unchanged;
5. subtraction of identical specialization removes fully covered singleton;
6. multi-field Cartesian subtraction still works after specialization check;
7. GADT impossible arm is `Impossible`, not merely `Redundant`;
8. exact-case union exhaustiveness does not collapse incompatible specializations.

---

# Task 9 — Fail Closed in Exhaustiveness and Complete GADT Equality

This task has two tightly related correctness goals: finite-domain construction must not lose cases, and equality solving must cover the canonical type forms it already traverses.

## 9A — Exhaustiveness metadata completeness

### Files

```text
phalcom-semantic/src/checker/exhaustiveness.rs
phalcom-semantic/src/checker/pattern_space.rs
```

### Edits

Current closed-enum expansion uses `filter_map` over declared `VariantId`s. Replace with an all-or-nothing collection:

```rust
let mut infos = Vec::with_capacity(enum_info.variants.len());
for variant in enum_info.variants.iter() {
    let Some(info) = table.variants.get(variant).cloned() else {
        return PatternSpace::Opaque(scrutinee_ty);
        // or a formal Blocked space/result if the architecture supports it
    };
    infos.push(info);
}
```

The exact recovery type can differ, but it must never drop a case and then prove exhaustiveness over the smaller set.

Also replace field-type fabrication such as:

```rust
field.declared_type.canonical_type().unwrap_or(scrutinee_ty)
```

with a conservative space based on actual type knowledge. Unknown field type means an opaque/unknown field domain, not the containing enum type by fiat.

### Tests

1. missing metadata for one declared variant prevents `ExhaustivenessResult::Proven`;
2. missing metadata does not disappear from witness generation as if impossible;
3. exact-case missing variant metadata stays conservative;
4. unknown payload type cannot make a refutable child pattern irrefutable.

## 9B — GADT record-row equality

### Files

```text
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/substitution.rs
```

### Edits

Add `TypeData::Record` handling to `unify_equality`.

Do not naïvely zip fields only. `RecordRowData` has canonical sorted fields and a tail:

```rust
RecordRowTail::Closed
RecordRowTail::Parameter(TypeParameterId) // kind RecordRow
```

Implement row equality with these rules:

- same closed field set: unify corresponding field types;
- closed rows with different field names/count: unequal;
- open row tail may absorb the unmatched suffix/field set according to row-kind substitution rules;
- tail parameter must be kind `RecordRow`;
- occurs-check applies to row substitutions;
- two row-tail parameters may unify through the row solver;
- result is equality, not width-subtyping.

If general row unification already exists elsewhere in the type system, reuse/extract it instead of embedding a second row solver in `gadt_proof.rs`.

### Tests

1. identical closed records unify;
2. field type parameter unifies through a record;
3. different closed field sets refute equality;
4. open-tail record unifies with compatible extra fields;
5. incompatible row tails refute;
6. cyclic row-tail equality is rejected;
7. GADT case whose result contains record type can refine callable generic parameter.

## 9C — Remove arbitrary 64-pass substitution ceiling

### Files

```text
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/types/substitution.rs
```

### Edits

Replace:

```rust
for _ in 0..64 { ... }
```

with deterministic graph normalization.

Recommended approach:

```rust
enum VisitState {
    Visiting,
    Resolved(TypeId),
}

fn normalize_substitution_term(
    store: &mut TypeStore,
    subst: &TypeSubstitution,
    ty: TypeId,
    states: &mut HashMap<TypeId, VisitState>,
) -> Result<TypeId, SubstitutionCycle>
```

Because binding already occurs-checks, a cycle is an internal invariant violation/recovery state. Surface it explicitly rather than returning a partially normalized type.

### Tests

1. substitution chain longer than 64 resolves fully;
2. ordinary short chain unchanged;
3. invalid cycle returns explicit failure and does not loop;
4. branch proof application is idempotent after normalization.

---

# Task 10 — Compile Complete Enum Behavior from Exact Semantic Identities

**Goal:** Remove the partial ad hoc enum behavior compiler and make executable behavior match semantic products exactly.

## Files

Modify:

```text
phalcom-core/src/compiler/lib/enum_decl.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/modules/semantic_lowering.rs
```

Potentially add a shared compiler helper module if `class_decl.rs` is too large:

```text
phalcom-core/src/compiler/lib/member_compiler.rs
```

Tests under current normalized architecture:

```text
phalcom-core/tests/core/compiler/adts.rs
phalcom-core/tests/core/execution/adts.rs
phalcom-core/tests/core/language/adts.rs
```

If those exact module files already exist, extend them rather than creating duplicates.

## 10.1 Remove base-only case attachment

Delete:

```rust
.find(|vs| vs.id.selector.base == SelectorBase::Named(v.name.clone()))
```

Compute canonical source variant selector using:

```rust
phalcom_ast::selector::selector_from_variant(v)
```

then construct exact `VariantId(owner, selector)` and find that exact lowering spec.

Prefer eliminating the search entirely by having semantic lowering publish a source-site -> exact variant target mapping if available.

### 10.2 Fix standalone selector synthesis

In the no-semantic-lowering fallback, stop turning every local payload name into a label.

Use:

```rust
let selector = phalcom_ast::selector::selector_from_variant(v);
```

This preserves:

- positional `_` slots;
- explicit labels;
- label order;
- singleton getter vs nullary constructor distinction.

### 10.3 Compile root behavior

Current `compile_enum` ignores `EnumMember::Behavior`. Add compilation of root bodyful behavior onto the enum root class behavior surface.

Do not compile root signature-only requirements; they are static semantic contracts with no body.

The compiler should receive a semantic/lowering indication of whether a root behavior has a body rather than rediscovering the requirement classification if possible.

### 10.4 Compile all case behavior forms

Replace the partial match:

```rust
Method
Getter
Setter(_) | Index(_) => {}
```

with complete support for:

```text
Method
Getter
Setter
Index get
Index set
rest parameters
labels
```

### 10.5 Reuse ordinary class member compilation

`enum_decl.rs` currently hand-builds signatures, closures, MethodObjects, constants, and `VariantMethod` bytecode for method/getter. That will drift from `class_decl.rs`.

Extract a shared helper that can compile a callable body to a `MethodObject` given:

```text
canonical Selector
source callable syntax/body
parameter shape
lexical owner/case context
```

Then installation differs only by target:

```text
class/root member -> existing class method installation bytecode/path
case member       -> VariantMethod { exact variant target, selector }
```

### 10.6 Illegal case behavior should be unreachable in core

Semantic analysis should reject case `@class` and declaration-only behavior. If malformed lowering reaches core anyway, return an internal `CompilerError` rather than synthesizing an empty body.

Never use:

```rust
body.statements().unwrap_or_default()
```

for a case member that semantically requires a body.

## Regression tests

1. root getter executes on every case;
2. root method executes on every case;
3. case override executes only on matching case;
4. same-base variant overloads attach behavior to the correct exact variant;
5. positional payload selector remains positional in standalone compile;
6. labeled variant selector preserves exact labels;
7. singleton vs `None()` remain distinct;
8. case setter executes;
9. case index getter executes;
10. case index setter executes;
11. rest/label method behavior matches ordinary class method behavior;
12. declaration-only case member cannot compile as empty method;
13. `@class` case member cannot reach executable installation;
14. root requirement has no runtime MethodObject unless separately implemented by a bodyful default.

---

# Task 11 — Harden Semantic-to-Executable Lowering

**Goal:** Make the lowering boundary fail closed and lossless. Part 05.2 strengthened this boundary; finish the job consistently.

## Files

Modify:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/error.rs
```

Tests:

```text
phalcom-core/tests/core/compiler/semantic_lowering.rs
phalcom-core/tests/core/compiler/adts.rs
```

## 11.1 Missing enum variant metadata is an error

Replace:

```rust
let shape = var_info.map(|v| v.shape).unwrap_or(VariantShape::Singleton);
```

with:

```rust
let vinfo = snapshot
    .enum_semantics
    .variant_info(variant_id)
    .ok_or_else(|| ProjectionError::MissingVariantMetadata(variant_id.clone()))?;
```

No metadata means no executable enum spec.

### 11.2 Missing pattern binding is an error

Replace:

```rust
.position(...).unwrap_or(0) as u32
```

with:

```rust
let index = bindings
    .iter()
    .position(...)
    .ok_or_else(|| ProjectionError::MissingPatternBinding(binding.clone()))?;
let binding_index = u32::try_from(index)
    .map_err(|_| ProjectionError::PatternBindingIndexOverflow(index))?;
```

Add precise error variants.

### 11.3 Checked slot/arity conversions

Introduce small helpers:

```rust
fn checked_u8_arity(len: usize, context: ...) -> Result<u8, ProjectionError>
fn checked_u16_slot(index: usize, context: ...) -> Result<u16, ProjectionError>
fn checked_u32_index(index: usize, context: ...) -> Result<u32, ProjectionError>
```

Use them for:

- variant constructor arity;
- behavioral selector arity where executable bytecode uses `u8`;
- family application arity;
- variant payload field slot;
- binding index;
- any semantic pool index narrowed in this file.

### 11.4 Preserve rest metadata

After Task 5 has separated associated variants from ordinary bound behavior, derive `ExecutableRestMode` from canonical `CallableSemanticSignature.parameters[].rest` / selector-family semantics.

Do not leave every behavioral target as:

```rust
rest_mode: ExecutableRestMode::None
```

Mapping:

```text
no rest                 -> None
*positionals            -> Positional
**labeled               -> Labeled
combined complete rest  -> Complete
```

Use the actual language `RestMode` model; do not infer rest merely from selector slots, because rest is intentionally not encoded as ordinary fixed slots.

### 11.5 Split associated variant lowering from ordinary Family lowering

The target lowering model should no longer need:

```text
AssociatedMemberId::Behavioral
ExecutableFamilyTarget::Behavioral inside an associated variant family descriptor
```

Associated lowering handles genuine associated members.

Ordinary bound behavioral families use their existing family-value lowering path and retain/capture the receiver.

### 11.6 Deterministic ordering

Before storing semantically unordered sets/maps into executable arrays, sort by canonical identity:

```text
DeclarationId
VariantId
Selector
VariantFieldId
FamilyOperationShape
```

Do not rely on `HashMap` iteration order for emitted descriptors or bytecode semantic pools.

## Regression tests

1. missing variant metadata returns `ProjectionError::MissingVariantMetadata`;
2. missing binding returns explicit projection error, not binding 0;
3. >u8 arity returns projection/compiler error;
4. >u16 payload slot count returns error;
5. rest modes survive lowering;
6. repeated lowering of same snapshot produces byte-identical/deterministically equal executable semantics;
7. non-proven match remains rejected (existing Part 05.2 invariant);
8. missing pattern field layout remains rejected (existing Part 05.2 invariant);
9. associated variant family contains no behavioral target entries after Task 5 split.

---

# Task 12 — Runtime Cleanup: Exact Declaration Identity, Receiver-Correct Families, Checked ADT Registration

**Goal:** Make VM execution consume semantic decisions rather than recover/guess them.

## Files

Modify:

```text
phalcom-core/src/vm/associated.rs
phalcom-core/src/vm/adt.rs
phalcom-core/src/adt.rs
# ordinary Family runtime module(s) used by existing :: implementation
```

Tests:

```text
phalcom-core/tests/core/execution/adts.rs
phalcom-core/tests/core/execution/families.rs
phalcom-core/tests/core/modules/declaration_identity.rs
phalcom-core/tests/native_adt_runtime.rs
```

## 12.1 Remove global leaf-name declaration fallback

Delete the final cross-module scan in `resolve_declaration_class`:

```rust
for (key, &class_id) in &self.classes {
    if self.interner.lookup(key.name) == decl.name.as_ref() {
        return Ok(class_id);
    }
}
```

Resolution order should use exact identities only:

```text
ADT registry by DeclarationId
known builtin declaration identity
exact ModuleId registry + class name
otherwise error
```

If builtin resolution currently uses only leaf name, constrain it to known core declaration IDs so a user module cannot masquerade as a builtin.

### 12.2 Retire “behavioral associated target” probing

Current `bind_behavioral_associated_target`:

- resolves lookup owner class;
- resolves defining class;
- checks defining metaclass;
- then falls back to defining instance-side class methods.

That logic is not the semantics of either category.

After Tasks 4–5:

- genuine associated variants do not need behavioral method binding;
- ordinary bound Families keep the actual captured receiver and call the ordinary dispatch path;
- class-object receiver naturally dispatches its `@class` behavior through the object model.

Delete or narrow `bind_behavioral_associated_target` accordingly. Do not leave metaclass/class fallback probing as hidden recovery.

### 12.3 Checked ADT registration conversions

Change `register_enum_from_spec` to return `Result<ClassId, RuntimeError>` (or an initialization error type) if needed so conversions can fail.

Replace:

```rust
CaseDiscriminant(idx as u32)
var_spec.payload_fields.len() as u16
slot as u16
```

with checked conversions.

Because semantic lowering should already reject oversized forms, runtime overflow is an internal invariant failure; still fail explicitly.

### 12.4 Preserve Part 06 representation seam

Keep:

```text
RuntimeAdtRepresentation::General
RuntimeAdtRepresentation::NativeOption
RuntimeAdtRepresentation::NativeResult
```

Do not make semantics depend on the representation enum.

If native recognition currently keys only on `spec.owner.name == "Option"` / `"Result"`, strengthen it to exact core `DeclarationId`s so user-defined `other.Option` cannot receive native representation accidentally.

### 12.5 Preserve case behavior inheritance

Hidden case behavior classes currently inherit enum root class. Keep that structural relation so root default behavior works once Task 10 installs it.

## Regression tests

1. two modules with class `Foo`: exact `DeclarationId` resolves correct class;
2. unknown module-qualified class never falls back to same-named class elsewhere;
3. user-defined `Option` outside core does not get `NativeOption` representation;
4. core Option still gets native representation;
5. bound instance family invokes on actual instance receiver;
6. bound class-object family invokes `@class` behavior on actual class object;
7. no metaclass-then-instance fallback is required for valid semantic target;
8. ADT discriminant/payload overflow fails explicitly;
9. hidden case class still inherits enum root behavior;
10. singleton and constructor runtime identity remain distinct.

---

# Task 13 — Part 05.2 and Part 06 Preservation / Vertical Integration Suite

**Goal:** Prove the remediation closes old gaps without regressing newly landed executable matching, native core ADTs, reflection, or tooling.

## Files

Extend existing semantic ADT test modules:

```text
phalcom-semantic/tests/semantic/adts/native_core.rs
phalcom-semantic/tests/semantic/adts/exact_cases.rs
phalcom-semantic/tests/semantic/adts/matching/*
phalcom-semantic/tests/semantic/adts/associated/*
phalcom-semantic/tests/semantic/adts/behavior.rs
phalcom-semantic/tests/semantic/adts/requirements.rs
```

Extend normalized core test tree:

```text
phalcom-core/tests/core/compiler/*
phalcom-core/tests/core/execution/*
phalcom-core/tests/core/language/*
phalcom-core/tests/core/reflection/*
phalcom-core/tests/native_adt_runtime.rs
```

Extend tooling/LSP tests only where Part 06 target identities are consumed:

```text
phalcom-lsp/... or current protocol-neutral tooling test modules
```

Do not create implementation-phase filenames such as `part6_remediation.rs`.

## 13.1 Native Option end-to-end

Test source equivalent to:

```phalcom
const x = Option::Some(42)
const y = match x {
    Some(v) => v
    None => 0
}
```

Verify:

- associated `Some` resolves as variant constructor, not behavior;
- exact `Some<Int>` case type retained;
- executable match uses exact variant identity;
- runtime native Option representation matches correctly;
- payload extraction returns 42;
- no boxing assumption leaks into semantic typing.

### 13.2 Native Result end-to-end

Equivalent coverage for `Result<T,E>` with both `Ok` and `Error` constructor families.

### 13.3 User ADT and native ADT semantic parity

For equivalent shapes, verify semantic products have the same categories:

```text
EnumInfo
VariantInfo
VariantFamilyId
VariantFieldId
ExactCase
match candidate identities
reflection descriptor identity
```

Native implementation provenance may differ; language semantics may not.

### 13.4 Reflection remains separate from `::`

Part 06 explicitly introduced dedicated reflection metaobjects. Add tests that:

- `Option::Some` is associated construction/member denotation, not reflection;
- `VariantReflection` identifies `VariantId` exactly;
- `ExactCaseTypeReflection` preserves specialization (`Some<Int>` vs `Some<String>`);
- `.class` returns runtime case behavior class, not the static `ExactCase` metaobject;
- variant family tooling/reflection identity remains distinct from bound behavioral Family values.

### 13.5 Tooling target preservation

Part 06 extended `SemanticTargetId` with `VariantFamily` and `VariantField`. Verify associated/behavior split does not erase these:

- go-to-definition on `Option::Some` -> variant/family declaration as intended;
- hover can show exact constructor family shape;
- pattern completion still derives enum variants from `EnumSemanticTable`, not behavior surface;
- match generation remains exhaustive over exact variant set;
- class-object `Foo::bar` behavior target remains a callable target, not `VariantFamily`.

### 13.6 Part 05.2 staged-binding regressions

Add runtime tests combining newly fixed semantic patterns with 05.2 lowering:

1. record pattern failure does not leak bindings;
2. map pattern failure does not leak bindings;
3. or-pattern with GADT variants commits shared binding once;
4. exhaustive match fallthrough still triggers only `MatchInvariantFailure` if semantic/runtime invariant is violated, never a user-level MatchError;
5. scrutinee evaluated once;
6. `while let` RHS evaluated once per iteration;
7. nested ADT + tuple/list/record/map patterns use one shared executable pattern engine.

---

# Task 14 — Update Coverage Ledger, Remove Stale Claims, Run Full Verification

## Files

Modify:

```text
phalcom-semantic/tests/semantic/adts/COVERAGE.md
docs/spec/current/selectors.md
docs/spec/adts.md
relevant docs/impl/adt-gadt-associated-lookup/* completion/checklist docs
```

### 14.1 Update ADT coverage ledger by language capability

Keep descriptive capability categories, not implementation parts:

```text
Declarations and identities
Variant construction
Associated namespace
Bound behavioral families
Enum defaults and requirements
Exact cases and generics
GADT proof/refinement
Pattern resolution
Exhaustiveness/usefulness
Executable lowering/runtime
Native core ADTs
Reflection/tooling
Failure/invariant behavior
```

For every bug fixed in this remediation, point to at least one source-level/session/runtime regression test.

### 14.2 Search for stale semantic claims

Run:

```bash
rg -n 'AssociatedMemberId::Behavioral|AssociatedFamilyKind::Behavioral' phalcom-semantic phalcom-core
rg -n 'associated behavioral|class-side associated method|static associated' docs phalcom-semantic phalcom-core
rg -n 'filter_map\(.*variants|unwrap_or\(VariantShape::Singleton\)|unwrap_or\(0\).*binding' phalcom-semantic phalcom-core
rg -n 'for _ in 0\.\.64' phalcom-semantic
rg -n 'EnumBehaviorMember::Setter\(_\) \| EnumBehaviorMember::Index\(_\) => \{\}' phalcom-core
rg -n 'selector\.base ==.*v\.name' phalcom-core
rg -n 'payload\.parameters.*Label\(p\.name' phalcom-core
```

Expected results after remediation:

- no ordinary behavior remains in associated-member representation;
- no production requirement invocation uses empty placeholder inputs;
- no wildcard fallback in exhaustive `Pattern` semantic match;
- no missing variant -> singleton recovery;
- no missing binding -> 0 recovery;
- no 64-pass substitution ceiling;
- no ignored enum setter/index case behavior;
- no base-only case attachment.

### 14.3 Full verification matrix

Run from repository root:

```bash
cargo fmt --all -- --check
cargo test -p phalcom-ast
cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then run repository-defined CI lanes relevant to runtime/unsafe/memory behavior. Inspect `.github/workflows` for the current authoritative commands; include Miri only where the repository currently supports it.

Do not claim a fixed test count. Report actual command results.

### 14.4 Focused pre-merge verification

Before the full workspace run, run focused suites after each task:

```bash
cargo test -p phalcom-semantic --test semantic adts
# adapt exact test filter to the current harness names

cargo test -p phalcom-core adt
cargo test -p phalcom-core match
cargo test -p phalcom-core family
cargo test -p phalcom-core reflection
```

Use the actual normalized test harness/module names on main.

---

# 4. Detailed File-by-File Change Map

This section is the implementation checklist by file.

## `docs/spec/current/selectors.md`

- Preserve receiver-bound Family semantics.
- Add associated-first precedence when receiver/declaration exposes a reserved associated base.
- Clarify class-object receiver behavior and `@class`.
- Clarify associated-member family vs behavioral Family value.
- Remove any wording implying every `Type::name` is intrinsically associated/static lookup.

## `docs/spec/adts.md`

- Replace “static associated `::` surface” shorthand with layered semantics.
- Add entire-base reservation rule.
- Add legal/illegal `@class` examples.
- Preserve Part 06 native/reflection section.

## `phalcom-semantic/src/checker/declaration_signature.rs`

- Extract syntax-independent callable signature construction helpers.
- Keep it the single source-to-semantic callable boundary.
- Reuse exact selector construction, generics, rest, annotations, source facts for enum behavior.
- Avoid unchecked parameter-index narrowing if encountered (`index as u32` -> checked/invariant helper where practical).

## `phalcom-semantic/src/checker/enum_behavior.rs` — new

- Convert `EnumBehaviorMember` root/case syntax into canonical semantic signatures.
- Classify root default vs root requirement.
- Produce exact variant-owned case callable identities.
- Reject case `@class`.
- Reject case declaration-only behavior.
- Emit diagnostics and product consumed by session.

## `phalcom-semantic/src/checker/enum_declaration.rs`

- Keep structural variant metadata focus.
- Harden `exact_case_type(...).unwrap_or_else(...)` if an explicit invariant/error path can be introduced without destabilizing APIs.
- Avoid default-result fallbacks that silently change malformed GADT semantics; diagnostics may recover, but semantic products should record blocked/invalid state where the type system supports it.

## `phalcom-semantic/src/session.rs`

- Build enum structural product.
- Build enum behavior product.
- Publish root default signatures and case signatures before body queries.
- Populate real `EnumRequirement` arrays and case implementation map.
- Remove empty placeholders in `check_enum_requirements` invocation/publication.
- Schedule enum callable body queries.
- Stop treating all `DispatchSide::Class` callables as associated members.
- Build associated surfaces from explicit associated declarations only.
- Prefer shared source-callable job iteration over duplicate class/enum loops.

## `phalcom-semantic/src/enum_requirements.rs`

- Keep requirement checking focused on already canonical signatures.
- Reuse common callable compatibility helpers if ordinary override checking exists.
- Ensure GADT specialization is equality/substitution-correct.
- Keep `Blocked` for incomplete semantic information rather than guessing types.

## `phalcom-semantic/src/associated.rs`

- Remove ordinary behavior from associated member/family representation.
- Associated surface contains genuine associated declarations only.
- Accept behavior-base set only for reservation validation.
- On conflict: diagnose and omit/poison, never publish mixed family.
- Preserve grouping of same-base variant constructor shapes as one variant family.

## `phalcom-semantic/src/checker/associated.rs`

- Narrow to genuine associated member resolution/specialization.
- Remove dispatch-surface scans over class + instance sides.
- Stop treating hierarchy behavioral methods as associated members.
- Remove Unit/Object fallback typing.
- Reuse equality solver for GADT associated compatibility.
- Let `::` expression resolver choose associated path first, ordinary bound-family path second.

## `phalcom-semantic/src/types/family.rs`

- Ensure behavioral Family type can express bound exact/pattern selector semantics independent of associated family.
- If associated variant families also use `TypeData::Family`, encode family category/member kind explicitly enough that lowering cannot confuse method dispatch with variant construction.
- Preserve rest/call-shape metadata.

## `phalcom-semantic/src/checker/pattern.rs`

- Add explicit Record and Map resolution.
- Remove wildcard catch-all.
- Preserve typed child bindings.
- Keep detached/shared binding machinery for or-patterns.

## `phalcom-semantic/src/checker/pattern_space.rs`

- Add sound Record/Map refutable spaces or conservative predicate space.
- Add exact-case + proof compatibility for Variant intersection/subtraction.
- Use proof merge helper; no overwrite.
- Extend `is_empty`, `normalize`, `intersect`, `subtract`, `summarize`.
- Optional: replace O(n²) union dedup only after correctness changes; use ordered/hash canonical key if straightforward.

## `phalcom-semantic/src/checker/exhaustiveness.rs`

- All-or-nothing enum variant metadata collection.
- Unknown payload types become conservative spaces, not enum-type fallback.
- Extend witness generation for Record/Map/predicate spaces.
- Preserve exact-case fail-closed behavior.

## `phalcom-semantic/src/checker/gadt_proof.rs`

- Add reusable branch-proof merge/compatibility API.
- Add row-aware `TypeData::Record` equality.
- Replace fixed 64-pass substitution loop with graph normalization/cycle detection.
- Keep equality semantics distinct from subtyping.

## `phalcom-semantic/src/types/row.rs`

- Expose/reuse canonical row equality/unification primitives if not already elsewhere.
- Respect sorted unique fields and `RecordRowTail::{Closed, Parameter}`.
- Enforce row-kind and occurs-check invariants.

## `phalcom-core/src/modules/semantic_lowering.rs`

- Fail on missing variant metadata.
- Fail on missing pattern binding.
- Use checked arity/slot/index conversions.
- Preserve rest metadata.
- Split genuine associated target lowering from ordinary bound behavioral Family lowering.
- Preserve Part 05.2 `MatchLoweringSpec` / `ExecutablePattern` architecture.
- Keep deterministic output ordering.

## `phalcom-core/src/compiler/lib/enum_decl.rs`

- Use `selector_from_variant` in standalone fallback.
- Attach case behavior by exact `VariantId`.
- Compile root bodyful behavior.
- Compile method/getter/setter/index for cases.
- Reuse class member compiler helper.
- Do not synthesize empty body for declaration-only member.
- Checked slot/arity conversions.

## `phalcom-core/src/compiler/lib/class_decl.rs`

- Extract/reuse method-object/closure compilation helper needed by enum root/case behavior.
- Keep installation target separate from compilation of callable body.
- No language semantic changes to ordinary class behavior.

## `phalcom-core/src/compiler/lib/associated.rs`

- Compile associated variant/member operations only for the associated path.
- Delegate behavioral `::` to ordinary Family compilation.
- Ensure receiver capture is preserved for behavior.

## `phalcom-core/src/vm/associated.rs`

- Remove module-unqualified leaf-name fallback.
- Remove metaclass/class probing used to simulate “associated behavior”.
- Keep only runtime operations needed for genuine associated targets.

## `phalcom-core/src/vm/adt.rs`

- Checked discriminant/payload conversions.
- Recognize native core ADTs by exact core declaration identity, not leaf name.
- Preserve hidden case class -> enum root superclass relationship.
- Preserve Part 05.2 variant test/payload primitives.

## `phalcom-core/src/adt.rs`

- Preserve Part 06 `RuntimeAdtRepresentation` seam.
- If registration APIs become fallible for checked bounds, propagate errors cleanly.

## Reflection/tooling files landed in Part 06

- Do not redesign.
- Adjust only if associated/behavior enum variants change a shared identity enum.
- Maintain `VariantFamily` and `VariantField` targets and exact-case reflection.

---

# 5. Required Regression Test Catalog by Finding

This table is the minimum regression commitment. Every row should map to at least one concrete test before the corresponding fix merges.

| Finding | Required regression |
|---|---|
| Empty requirement pipeline | Source enum requirement appears in session snapshot and missing case emits diagnostic |
| Root behavior absent | Root bodyful method/getter executes on multiple cases |
| Case behavior partial | Setter/index/method/getter all execute |
| Base-only variant attachment | Same-base variants with different exact shapes execute their own case behavior |
| Wrong standalone selector synthesis | Positional `_` payload stays positional; explicit labels stay labeled |
| Associated/behavior conflation | `@class foo` absent from AssociatedSurface but `Foo::foo` still works behaviorally |
| Base reservation | `@variant Some...` + `@class Some(_,_)` is illegal despite different shape |
| Mixed family publication | conflict diagnostic leaves no family containing both variant + callable |
| Resolver side probing | instance and class-object family tests resolve through ordinary bound behavior, not associated scan |
| Unknown -> Unit/Object | unresolved variant/callable type remains unknown/blocked |
| Record pattern wildcard | record pattern leaves residual and produces Record resolution |
| Map pattern wildcard | map pattern leaves residual and produces Map resolution |
| Variant proof overwrite | contradictory proof bindings make intersection Empty |
| Specialization overlap | `Some<Int>` and `Some<String>` spaces are disjoint |
| Dropped variant metadata | missing one variant metadata prevents proven exhaustiveness |
| Record GADT equality | record-containing GADT case refines generic type |
| 64-pass limit | >64 substitution chain resolves fully |
| Missing variant -> singleton | lowering returns explicit ProjectionError |
| Missing binding -> 0 | lowering returns explicit ProjectionError |
| Rest metadata erased | family rest lane survives semantic->executable projection |
| Unchecked narrowing | oversized arity/slot returns explicit error |
| Global leaf fallback | two modules with same class name resolve exactly |
| Native name collision | user `Option` does not receive core native representation |
| Part 05.2 preservation | record/map/GADT match executes via shared pattern engine without leaked bindings |
| Part 06 preservation | native Option/Result reflection/exact-case/tooling identities remain correct |

---

# 6. Suggested Commit / Review Slices

Do not land this as one giant change. Recommended slices:

### Slice A — Semantic contract + enum behavior products

Tasks 0–1.

Review focus:

- no duplicated signature parser;
- exact callable owners;
- `@class` terminology;
- root default vs requirement classification.

### Slice B — Session integration + requirements/defaults

Tasks 2–3.

Review focus:

- no empty placeholders;
- source-level integration tests;
- body queries include enum callables;
- exact case identity.

### Slice C — Associated namespace / `::` reconciliation

Tasks 4–6.

Review focus:

- ordinary behavior removed from associated surface;
- base reservation;
- associated precedence;
- behavioral receiver capture retained;
- no class/instance probing.

### Slice D — Pattern/GADT correctness

Tasks 7–9.

Review focus:

- no wildcard fallback;
- conservative complements;
- proof compatibility;
- fail-closed metadata;
- row equality/substitution normalization.

### Slice E — Core lowering/compiler/runtime hardening

Tasks 10–12.

Review focus:

- exact VariantId attachment;
- full behavior forms;
- checked conversions;
- receiver-correct family execution;
- exact module identity.

### Slice F — Integration verification and docs completion

Tasks 13–14.

Review focus:

- Part 05.2/06 preservation;
- end-to-end source behavior;
- coverage ledger reflects implementation.

---

# 7. Non-Goals / Explicitly Deferred Work

Do not expand this remediation into the following unless a correctness dependency forces it:

- jump-table or decision-DAG match optimization;
- generalized open-world extensible variants;
- case-local `@class` behavior support;
- new exact-case source syntax beyond what is already ratified;
- redesign of Part 06 reflection APIs;
- ABI/serialization guarantees;
- representation optimization beyond preserving current native Option/Result seam;
- performance rewrite of all pattern-space algebra;
- changing `::` syntax or selector grammar;
- monkey-patching support;
- converting class methods to a `static` keyword.

The O(n²) union dedup in `PatternSpace::normalize` may be improved opportunistically after correctness tests pass, but it should not delay the semantic fixes.

---

# 8. Definition of Done

ADT/GADT/associated-lookup work is considered complete for this architecture when all of the following are true:

- [ ] Source enum root bodyful behavior is a canonical semantic callable and inherited runtime default.
- [ ] Source enum root signature-only behavior is a real closed-enum requirement.
- [ ] Every variant is checked against real source-derived requirements.
- [ ] Case behavior has exact `VariantId` ownership and full method/getter/setter/index support.
- [ ] Case `@class` and declaration-only behavior are rejected before codegen.
- [ ] Associated namespace contains genuine associated members only.
- [ ] Associated bases reserve the entire base against behavior in the same declaration.
- [ ] `receiver::...` first resolves a matching associated base when exposed, otherwise preserves ordinary receiver-bound deferred behavior.
- [ ] `Foo::bar` ordinary behavior continues to bind the class object and resolve `@class bar` when no associated `bar` exists.
- [ ] Associated variants and behavioral Families are distinct semantic/executable categories.
- [ ] No associated specialization fabricates `Unit`/`Object` for missing semantic types.
- [ ] Record and Map patterns have real semantic resolutions and refutable spaces.
- [ ] Pattern-space variant algebra respects exact specialization and GADT proof compatibility.
- [ ] Exhaustiveness never proves a domain complete after silently losing variant metadata.
- [ ] GADT equality supports record rows and has no arbitrary normalization pass ceiling.
- [ ] Semantic lowering fails closed on missing metadata/bindings and uses checked conversions.
- [ ] Enum compiler uses canonical selectors and exact variant identities.
- [ ] Runtime declaration resolution is module-qualified and exact.
- [ ] Bound behavioral Family runtime dispatch uses the captured receiver; no class/instance guessing remains.
- [ ] Native Option/Result semantics, reflection, ExactCase identity, match lowering, and tooling from Parts 05.2/06 remain green.
- [ ] `cargo fmt`, focused tests, `cargo test --workspace`, and clippy pass on the implementation branch.

---

# 9. Planning Verification Notes

This plan was re-grounded against repository `main` at:

```text
347ffedf94c570c18c5589ac1dbf98549f9224cb
feat: implement Part 06 core integration, reflection, and tooling for ADTs
```

The source audit explicitly included the newly landed Part 05.2/06 state. Current-main verification found, among other things:

- `phalcom-semantic/src/session.rs` still calls `check_enum_requirements` with empty requirement and case-method inputs and publishes empty requirement arrays;
- `phalcom-semantic/src/checker/associated.rs` still scans both `DispatchSide::Class` and `DispatchSide::Instance` while constructing an “associated” behavioral family;
- `phalcom-semantic/src/associated.rs` still publishes a mixed invalid family after diagnosing variant/behavior category conflict;
- `phalcom-semantic/src/checker/pattern.rs` still has a wildcard catch-all after handling wildcard/name/variant/or/tuple/list patterns;
- `phalcom-core/src/modules/semantic_lowering.rs` now contains Part 05.2 `ExecutablePattern::{Record, Map}` and match projection, but still defaults missing enum variant metadata to singleton and missing pattern binding to index 0;
- `phalcom-semantic/src/checker/pattern_space.rs` still has no Record/Map space and merges branch proof bindings by overwrite;
- `phalcom-semantic/src/checker/exhaustiveness.rs` still uses `filter_map` when expanding a closed enum’s declared variants;
- `phalcom-semantic/src/checker/gadt_proof.rs` still lacks `TypeData::Record` equality and still uses a 64-pass substitution fixpoint;
- `phalcom-core/src/compiler/lib/enum_decl.rs` still attaches case behavior by selector base, reconstructs fallback variant selector labels from local parameter names, ignores setter/index case behavior, and does not compile root behavior;
- `phalcom-core/src/vm/associated.rs` still has a global leaf-name class fallback and metaclass/class probing for “associated behavior”;
- `phalcom-core/src/vm/adt.rs` preserves Part 06 native representation selection but currently identifies `Option`/`Result` by declaration leaf name and uses unchecked discriminant/payload narrowing.

Local execution of the verification matrix was **not** performed during planning because the available container could not clone GitHub (network/DNS access was unavailable). The file-level findings above were verified through the connected GitHub repository API against the stated commit. The implementation worker must run the verification matrix in Task 14 on a real checkout before claiming completion.

---

# 10. Handoff Instruction for the Implementing Agent

Execute this plan in dependency order. Use test-driven development for each behavioral task. Do not start a later core/runtime task by reconstructing missing semantics; if a required semantic product is absent, stop and implement the upstream semantic task first.

For each task:

1. write the listed failing regression first;
2. make the smallest semantic/API change that establishes the required invariant;
3. run the focused package/module tests;
4. inspect the diff for fallback/default recovery that could hide invalid state;
5. only then proceed to the dependent task.

Before completion, run the full verification matrix and report actual failures/results. Do not claim completion based only on direct unit tests of helper functions; source-to-session-to-lowering-to-runtime vertical tests are required for the major ADT features.
