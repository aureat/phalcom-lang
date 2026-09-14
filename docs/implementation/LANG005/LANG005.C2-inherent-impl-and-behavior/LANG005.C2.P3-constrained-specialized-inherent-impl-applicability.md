# LANG005.C2.P3 — Receiver-Specialized and Constrained Inherent `impl` Applicability, Coherence, and Conditional Dispatch

```yaml
---
id: LANG005.C2.P3
category: LANG
program: LANG005
checkpoint: LANG005.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C2.P1
  - LANG005.C2.P2
follows: LANG005.C2.P2
supersedes: null
---
```

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementer operating under constrained architectural authority.

The implementer:

- may adapt mechanical names, file locations, and helper boundaries to the live repository;
- must preserve the semantic architecture, identity model, coherence law, runtime boundary, and invariants in this plan;
- must not improvise an alternative specialization or overload system;
- must not begin P3 implementation until LANG005.C2.P1 and LANG005.C2.P2 are actually implemented and their takeover interfaces are verified in the live tree;
- must follow the testing budget below rather than running broad suites reflexively;
- must classify unrelated failures instead of repairing them opportunistically;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep the canonical `LANG005.C2` checkpoint state current;
- must create the required P3 walkthrough and handoff artifacts;
- must leave C2 ready for checkpoint-level acceptance and C3 trait-declaration planning.

Testing is evidence gathering, not ritual. Prefer one test that proves a semantic invariant over several tests that only prove syntax plumbing.

### Hard predecessor gate

At the repository revision used to prepare this plan, `main` contains the C2.P1 plan but does not yet contain the planned C2.P1/P2 source architecture. C2.P2 exists as the immediately preceding patch-grade deliverable, not as a verified landed implementation at this planning baseline.

> **Do not implement P3 on top of the pre-P1/P2 tree.**

Before P3 begins, verify repository-equivalent forms of all of the following are present:

```text
P1
  first-class Statement::Impl / ImplDef
  BehaviorMember or one canonical behavior-only AST category
  stable ImplId provenance
  TypeParameterOwner::Impl or equivalent stable impl generic ownership
  same-module inherent ownership enforcement
  covering generic head normalization
  DeclaredSurface + InherentImplContribution + effective DeclarationSurface
  target-indexed InherentImplSet
  effective callable-definition provenance
  body checking for impl-origin members
  semantic-to-compiler inherent impl lowering
  compiler installation without runtime class reopen semantics
  source-index/LSP identity for impl-origin callables
  incremental fingerprints/dependencies for impl contributions

P2
  variants-only canonical enum declarations
  enum-root behavior authored through impl
  exact-case impl target resolution to canonical VariantId
  EnumBehaviorProduct populated from normalized impl behavior
  EnumRequirementId distinct from exact-case witness CallableId
  GADT/CaseTypeEnvironment specialization retained as authority
  case-only effective behavior
  semantic lowering of enum root/case impl behavior
  removal of compiler/source-index AST scans for enum-local behavior
```

If a predecessor is incomplete, stop P3 and complete that predecessor first. If interfaces landed under different mechanical names but preserve the same architecture, adapt locally. If a predecessor materially changed target identity, callable identity, contribution/effective-surface ownership, enum exact-case identity, or semantic lowering, **STOP AND CONSULT** before editing P3.

---

## 1. Goal

Extend first-class inherent `impl` from unconditional/covering behavior to receiver-specialized and constrained behavior without turning Phalcom into an overload-by-type language, without flattening conditional members into unconditional declaration surfaces, and without installing receiver-conditional methods unconditionally on shared runtime behavior classes.

Target source forms include:

```phalcom
impl Point<Int> {
  intOnly { ... }
}

impl<T> Pair<T, T> {
  diagonalOnly { ... }
}

impl<T> Pair<T, Int> {
  rightIntOnly { ... }
}

impl<T> Point<T> where T <: Number {
  magnitude { ... }
}
```

P3 closes LANG005.C2 by adding:

```text
canonical impl domains
+ structural receiver-head matching
+ existing generic constraint proof
+ receiver-effective conditional member lookup
+ strict inherent coherence
+ conditional semantic lowering
+ executable conditional dispatch
+ incremental/tooling integration
```

---

## 2. Checkpoint acceptance objective

LANG005.C2 is complete after P3 when the three-plan architecture is coherent:

```text
C2.P1
  first-class inherent impl
  declared -> contribution -> unconditional effective surface

C2.P2
  variants-only enum declarations
  closed enum root/default/exact-case behavior through impl

C2.P3
  receiver-specialized/constrained impl domains
  conditional member applicability
  coherence + executable conditional dispatch
```

P3 is accepted when:

1. specialized and constrained impl heads are canonical semantic products, not AST predicates;
2. target-head matching is structural/canonical rather than subtype-driven;
3. every impl-owned parameter used by applicability is bound from the receiver head; constraints restrict those bindings but do not invent them;
4. ordinary P1/P2 `DeclarationSurface` remains the unconditional fast/base surface;
5. conditional members live in a distinct target-indexed conditional contribution/index layer;
6. lookup computes a receiver-effective member view from the actual semantic receiver plus ambient generic evidence;
7. applicability has explicit `Applicable` / `NotApplicable` / blocked-or-unknown outcomes rather than optimistic Boolean matching;
8. same-owner selector identity remains unique across primary, unconditional, and conditional inherent behavior—P3 does not introduce type-overloaded methods or most-specific specialization;
9. inherited receiver specialization is performed by the canonical receiver-specialization machinery before impl-head matching;
10. body checking composes impl-owned generics, head equalities, `where` constraints, `Self`, and P2 exact-case/GADT environments;
11. union receivers require compatible availability across all statically reachable non-dynamic arms;
12. P2 exact-case targets can additionally carry P3 head/constraint applicability without becoming ordinary nominal specialization;
13. conditional enum root requirements/defaults, if supported by the bounded domain-proof machinery in this plan, preserve the closed-enum contract model rather than becoming traits;
14. compiler lowering receives the semantically selected conditional implementation and never re-solves impl applicability from AST;
15. receiver-conditional members are **not** installed unconditionally on shared generic runtime classes;
16. executable dispatch preserves ordinary subclass overriding while using the selected conditional implementation as the applicable fallback;
17. bound method/callable references and class-side conditional behavior use the same semantic selection authority;
18. body-only edits do not invalidate impl-domain applicability products;
19. target-head or `where` edits invalidate only the affected conditional lookup dependents at the finest practical repository granularity;
20. source/LSP completion/navigation derives conditional visibility from semantic receiver-effective lookup;
21. no trait conformance, trait constraints, orphan rules, or trait specialization are introduced;
22. C2 checkpoint state, walkthroughs, handoffs, and focused acceptance evidence are complete.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
planning revision: d8b823e857b4054420cc070cda9d6b030f151391
prepared: 2026-09-13
predecessor plans:
  LANG005.C2.P1 — First-Class Inherent impl and Effective Declaration Surfaces
  LANG005.C2.P2 — Variants-Only Enum Declarations and Closed Enum Behavior Migration to impl
```

### Verified planning-time facts

1. C2.P1 explicitly reserves for P3:

   ```text
   constrained target head
   receiver applicability
   where constraints
   specialized applied heads
   overlap/conflict policy
   ```

2. P1's planned `ResolvedInherentImplTarget` already contains a semantic seam for applicability:

   ```rust
   pub enum InherentImplApplicability {
       Unconditional,
       Covering(CoveringImplSubstitution),
   }
   ```

   P3 should extend that architecture rather than replacing it.

3. Current `DeclarationSurface` is declaration-level and receiver-independent. It maps one selector to one callable/signature per dispatch side and cannot correctly encode `Box<Int>`-only membership without making that member appear on every `Box<T>`.

4. Current semantic dispatch already has canonical receiver specialization via `specialize_receiver_to_owner`, including nominal, applied, and exact-case receivers plus transformed generic inheritance.

5. Current call application already distinguishes canonical invocation target, callable identity, and receiver specialization. P3 should attach conditional selection to that semantic path rather than add compiler/LSP-local applicability solvers.

6. Existing `GenericSignature`, `TypeParameterOwner`, `GenericConstraint`, `TypeEnvironment`, substitution, inference, and relation machinery already provide the generic/constraint substrate P3 should reuse.

7. The language's selector/callable identity remains shape-based, not parameter/return-type overloaded. P1 explicitly treats two would-be same-selector callables on one target as a conflict.

8. Ordinary runtime `.` sends use class-hierarchy method lookup. Current runtime generic values do not generally have a distinct behavior class per applied semantic type.

9. Existing type-system completion records identify executable applied-class context/per-applied-type class state as a separate missing runtime capability. P3 must not silently solve that larger problem by allocating runtime classes per generic application.

10. P2's required runtime topology for enums remains one root behavior class plus hidden case behavior classes. P3 exact-case constraints must preserve that topology.

11. P1's same-module ownership rule remains in force. P3 changes applicability, not orphan/open-class ownership.

> Re-run the drift protocol against the landed P1/P2 tree before editing. Mechanical drift is local. Architectural drift is a consultation event.

---

## 4. Required reads before implementation

After P1 and P2 have landed, read in this order:

```text
1. AGENTS.md
2. docs/implementation/README.md
3. docs/implementation/LANG005/LANG005.C2/CHECKPOINT.md
   or the repository's canonical equivalent
4. LANG005.C2.P1 plan
5. LANG005.C2.P1 walkthrough
6. LANG005.C2.P1 handoff
7. LANG005.C2.P2 plan
8. LANG005.C2.P2 walkthrough
9. LANG005.C2.P2 handoff
10. current LANG005 semantic update/program record
11. docs/spec/typing/03-type-parameters-and-generic-signatures.md
12. current generic application / receiver specialization specs still marked authoritative
13. docs/spec/adts.md after P2 migration
14. relevant PDR/ADR records governing semantic identity and runtime representation
15. phalcom-semantic/tests/semantic/README.md
16. phalcom-core/tests/README.md
```

Then inspect live repository equivalents of:

```text
P1/P2 landed impl modules
  identity / ImplId
  resolved impl target
  impl contribution products
  InherentImplSet
  EffectiveSurfaceProduct / callable provenance
  impl body checking
  impl source index

phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/*

P2 enum products
phalcom-semantic/src/checker/enum_behavior.rs or landed equivalent
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/types/case_environment.rs
phalcom-semantic/src/types/case_instantiation.rs

phalcom-core/src/modules/semantic_lowering.rs
P1/P2 impl lowering/compiler modules
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/adt.rs

P1/P2 impl tests
semantic receiver-specialization tests
semantic generic-constraint tests
semantic incremental tests
core dispatch tests
core ADT tests
```

Do not browse unrelated runtime or type-system modules unless a failing invariant points there.

---

## 5. Normative authority

### 5.1 Supported P3 source forms — FIXED

Support specialized concrete heads:

```phalcom
impl Point<Int> {
  intOnly { ... }
}
```

Support repeated-parameter heads:

```phalcom
impl<T> Pair<T, T> {
  diagonalOnly { ... }
}
```

Support mixed concrete/parameter heads:

```phalcom
impl<T> Pair<T, Int> {
  rightIntOnly { ... }
}
```

Support nested structural heads when the existing type grammar/type lowering can canonically form them:

```phalcom
impl<T> Wrapper<List<T>> {
  listPayloadOnly { ... }
}
```

Support `where`-conditioned heads using only generic constraint forms already canonical in the Phalcom type system:

```phalcom
impl<T> Point<T> where T <: Number {
  magnitude { ... }
}
```

Do **not** add trait-conformance constraints in P3. Trait constraints belong to LANG005.C6 after trait/conformance architecture exists.

### 5.2 Head matching — FIXED

Impl-head matching is structural over canonical semantic type forms.

This means:

```phalcom
impl Point<Number> { ... }
```

matches `Point<Number>` exactly. It does **not** automatically match `Point<Int>` merely because `Int <: Number`.

The intended subtype-conditioned form is:

```phalcom
impl<T> Point<T> where T <: Number { ... }
```

Declaration variance must not silently change inherent-member availability.

### 5.3 Constraint role — FIXED

Receiver-head matching binds impl-owned parameters.

`where` constraints restrict those bindings.

Constraints do not manufacture an otherwise-unbound impl parameter.

Therefore reject semantically underdetermined heads such as:

```phalcom
impl<T, U> Box<T> where U <: Number {
  impossibleToSelect { ... }
}
```

All impl-owned parameters that affect applicability must be bound by the target head.

### 5.4 Callable identity and coherence — FIXED

P3 does not introduce type-based selector overloading or Rust-style most-specific specialization.

For one canonical callable owner and dispatch side:

```text
selector identity remains unique
```

Therefore this is a conflict:

```phalcom
impl<T> Box<T> where T <: Number {
  describe { ... }
}

impl Box<Int> {
  describe { ... }
}
```

The two source fragments would define the same canonical selector on the same declaration owner. P3 does not choose one as "more specific."

Different conditional impl domains may coexist when they contribute different selectors:

```phalcom
impl<T> Box<T> where T <: Number {
  numericOperation { ... }
}

impl Box<Int> {
  intOperation { ... }
}
```

Different exact enum cases may define the same selector because their canonical callable owners are different `VariantId`s, as established by P2.

### 5.5 Same-module inherent ownership — FIXED

P3 does not weaken P1's ownership rule:

```text
impl module == target declaration module
```

No orphan/open-extension policy is introduced here.

### 5.6 Runtime visibility boundary — FIXED

A conditional member is available when semantic receiver evidence proves its impl domain.

If a value is erased to a type that does not prove the domain, the conditional member is not part of that static surface.

`Dynamic` does not grant runtime discovery of arbitrary P3 conditional impls. P3 must not reconstruct erased generic evidence by inspecting runtime value classes.

### 5.7 Ordinary overriding — FIXED

A conditional member selected for an owner must still respect an ordinary method declared on a more-derived runtime class.

Conceptually:

```text
actual runtime receiver
    ↓
search more-derived ordinary overrides
    ↓
found -> invoke override
missing -> invoke statically selected conditional implementation
```

The conditional implementation shadows same-selector behavior above its declaring owner when applicable, exactly as a member on that owner would. Same-owner duplicate definitions are prevented by coherence.

### 5.8 Enum semantics — FIXED

P2 exact-case semantics remain a distinct semantic category. P3 may add applicability constraints to an exact-case impl, but it must not re-express exact cases as ordinary nominal specializations.

Closed enum requirements remain closed enum contracts, not traits.

### 5.9 Reflection — DEFERRED

P3 preserves stable domain/provenance evidence required by future LANG005.C7 reflection, but does not implement the final public reflection API for conditional impl domains.

---

## 6. Takeover state

### 6.1 Required P1 takeover interfaces — VERIFY FIRST

Expect repository-equivalent forms of:

```text
ImplId
TypeParameterOwner::Impl
ResolvedInherentImplTarget
InherentImplApplicability
CoveringImplSubstitution
InherentImplContribution
InherentImplSet
DeclaredSurface
EffectiveSurfaceProduct / DeclarationSurface
CallableDefinitionOrigin::InherentImpl
semantic impl lowering product
```

### 6.2 Required P2 takeover interfaces — VERIFY FIRST

Expect repository-equivalent forms of:

```text
root impl target
exact-case impl target -> VariantId
EnumBehaviorProduct
EnumRequirementId
root default / requirement classification
case implementation / override / case-only classification
CaseTypeEnvironment integration
exact-case body environment
root + hidden case runtime target mapping
```

### 6.3 Takeover stop condition

If P1 or P2 landed by flattening all members into one unconditional surface, by using `ImplId` as callable identity, by using runtime `ClassId` as semantic target identity, or by compiler-side AST re-resolution, stop and consult. P3 depends on those boundaries being correct.

---

## 7. Architecture

### 7.1 Direction of authority

Final authority flow:

```text
source impl header
    ↓
ImplId + canonical generic signature
    ↓
Resolved target owner / exact-case target
    ↓
canonical InherentImplDomain
    ↓
conditional member contribution templates
    ↓
target-indexed conditional member index
    ↓
actual receiver + ambient evidence
    ↓
applicability proof + impl specialization
    ↓
receiver-effective member selection
    ↓
canonical callable/call application
    ↓
semantic lowering attachment
    ↓
compiler executable conditional dispatch
```

No later layer reparses the source impl header.

### 7.2 Canonical impl domain

Introduce a semantic product repository-equivalent to:

```rust
pub struct InherentImplDomain {
    pub impl_id: ImplId,
    pub target: InherentImplTargetId,
    pub declaration: DeclarationId,
    pub head: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub constraints: Box<[GenericConstraint]>,
}
```

`InherentImplTargetId` should reuse/extend the landed P2 target model and distinguish at least:

```text
Declaration(DeclarationId)
ExactCase(VariantId)
```

Do not use runtime class identity here.

The precise storage of `head` may be a canonical `TypeId`, canonical type term/template, or repository-native equivalent. It must preserve enough structure to bind impl-owned parameters without retaining AST.

### 7.3 Applicability classification

Extend P1 applicability conceptually to distinguish:

```rust
pub enum InherentImplApplicability {
    Unconditional,
    Covering(CoveringImplSubstitution),
    Conditional(InherentImplDomainId),
}
```

Exact naming is mechanically flexible.

Do not route P1 covering impls through the generalized matcher on every lookup. Preserve the existing unconditional effective surface as the fast path.

### 7.4 Structural head matching

Given actual receiver `R` and impl domain `D`:

```text
1. project R to D.declaration using canonical receiver specialization
2. obtain canonical applied owner form
3. structurally match D.head against that form
4. bind each impl-owned parameter consistently
5. enforce repeated-parameter equalities
6. reject structural mismatch
7. substitute the resulting bindings into D.constraints
8. prove constraints using existing semantic relation/constraint authority
9. return explicit applicability evidence
```

Do not use ordinary subtype relation for step 3.

Nested canonical applications are matched recursively.

Aliases must be handled according to existing canonical type normalization; do not add alias-spelling-sensitive behavior.

### 7.5 Applicability evidence

Produce a semantic result conceptually equivalent to:

```rust
pub struct InherentImplSpecialization {
    pub impl_id: ImplId,
    pub receiver: TypeId,
    pub owner_view: TypeId,
    pub bindings: TypeEnvironment,
    pub satisfied_constraints: Box<[GenericConstraintEvidence]>,
}

pub enum ImplApplicabilityResult {
    Applicable(InherentImplSpecialization),
    NotApplicable,
    Blocked(BlockReason),
    Dynamic(DynamicReason),
}
```

Reuse repository-native proof/evidence types. Do not invent a parallel generic solver.

`Blocked`/unknown does not mean applicable.

### 7.6 Conditional member products

Keep P1/P2 unconditional `DeclarationSurface` unchanged as the declaration-wide surface.

Introduce a separate product conceptually equivalent to:

```rust
pub struct ConditionalInherentMember {
    pub impl_id: ImplId,
    pub domain: InherentImplDomainId,
    pub callable: CallableId,
    pub signature_template: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source: SourceSiteId,
}

pub struct ConditionalInherentMemberSet {
    pub target: InherentImplTargetId,
    pub by_selector: ...,
}
```

For a declaration target, conditional templates are indexed by declaration owner + side + selector.

For an exact case, index by canonical `VariantId` + side + selector.

Do not insert these templates into the unconditional `DeclarationSurface`.

### 7.7 Receiver-effective member view

Add one canonical semantic lookup operation that combines:

```text
unconditional surface
+ applicable conditional member for actual receiver
+ inherited owner traversal
+ P2 exact-case behavior where relevant
```

Conceptually:

```rust
resolve_effective_dispatch(
    receiver: TypeId,
    selector: &Selector,
    side: DispatchSide,
    ambient_evidence: ...,
) -> ResolvedDispatchResult
```

The existing public API may be extended rather than replaced.

All expression typing, method references, associated behavior lookup, and LSP receiver-effective completion must consume the same semantic authority.

### 7.8 Coherence without type overloading

P3's overlap policy is deliberately strict and simple:

```text
impl-domain overlap itself is allowed

but for one canonical callable owner:
  (dispatch side, selector) has at most one inherent concrete definition
```

Check duplicate selector definitions across:

```text
primary declaration
P1 unconditional impls
P2 root/exact-case impls
P3 conditional impls
```

For exact cases, the owner is the P2 variant callable owner, so two different variants do not conflict.

This policy means P3 does **not** need a general pairwise domain-intersection solver merely to decide which method body wins.

### 7.9 Inheritance and owner-relative matching

If the selected conditional impl belongs to an ancestor declaration, first project the receiver to that ancestor using existing receiver specialization.

Example:

```phalcom
class Child<T> : Base<List<T>> { ... }

impl<U> Base<List<U>> {
  listBaseOnly { ... }
}
```

`Child<Int>` must bind `U = Int` through the canonical `Base<List<Int>>` owner view.

Do not infer substitutions independently inside P3 dispatch.

### 7.10 Conditional body environment

When checking a conditional impl member body, establish:

```text
impl-owned generic parameters
+ target-head equalities/bindings
+ impl where constraints as assumptions
+ Self = target head / exact-case receiver template
+ target declaration visibility/private context from P1
+ P2 exact-case payload and CaseTypeEnvironment where applicable
```

The body should be able to call another conditional member only when its domain is provably applicable under the current environment.

No impl-owned parameter may escape a resolved external member signature after receiver specialization.

### 7.11 Resolved signature specialization

Conditional member templates may legitimately contain `TypeParameterOwner::Impl` parameters.

At lookup:

```text
conditional signature template
    + InherentImplSpecialization.bindings
    ↓
resolved callable signature
```

The resolved call product must contain no unsolved impl-owned parameter unless the language's existing generic callable machinery intentionally retains a callable-owned parameter.

Callable-owned method generics remain independent of impl-owned generic parameters.

### 7.12 Generic/symbolic receiver evidence

For:

```phalcom
impl<U> Point<U> where U <: Number {
  magnitude { ... }
}
```

then:

```phalcom
fn f<T>(p: Point<T>) {
  p.magnitude       // unavailable unless current context proves T <: Number
}

fn g<T>(p: Point<T>) where T <: Number {
  p.magnitude       // available
}
```

P3 must consume existing ambient generic constraints/evidence. It must not solve caller-owned generic variables merely to force applicability.

### 7.13 Union receivers

Use the existing union-call arm architecture.

A normal statically typed send is accepted only when every reachable non-dynamic arm resolves the selector compatibly under that arm's applicability evidence.

Example:

```text
Point<Int> | Point<String>
```

If `intOnly` exists only for `Point<Int>`, `x.intOnly` is not generally available on the union.

Do not collapse partial conditional availability to `Dynamic` merely to accept the call.

### 7.14 Exact-case and GADT composition

P2 exact-case identity remains authoritative:

```text
VariantId
+ exact enum type
+ CaseTypeEnvironment
```

P3 may constrain an exact-case impl further:

```phalcom
impl<T> Expr<T>::Literal(_) where T <: Number {
  numericLiteralOnly { ... }
}
```

Applicability must compose:

```text
exact-case proof
+ enum/declaration bindings
+ variant-local/GADT case environment
+ P3 impl-head bindings
+ P3 where constraints
```

Do not rederive GADT equalities in the P3 matcher.

### 7.15 Conditional closed enum requirements/defaults

After P2, declaration-only root impl members are closed-enum requirements. P3 therefore must define constrained root behavior intentionally.

Preferred complete rule:

```phalcom
impl<T> Shape<T> where T <: Numeric {
  area -> Float
}
```

creates a root requirement whose domain is the enclosing impl domain.

For every exact case, requirement completeness is checked only over receiver instantiations for which that root domain applies.

A case implementation/default satisfies the requirement only if its own applicability covers the required domain.

The bounded proof operation is conceptually:

```text
requirement-domain assumptions
    ⊢ case implementation domain applicable
```

Reuse the same applicability engine under assumptions. Do not build a separate logical implication solver unless existing relation machinery cannot express the needed proof.

**Consultation threshold:** if conditional requirement coverage requires quantifier reasoning or a materially more powerful constraint solver than the existing bounded generic relation machinery, STOP AND CONSULT. Do not silently weaken completeness. A deliberate temporary rejection of bodyless members inside conditional enum-root impls is preferable to unsound coverage.

### 7.16 Compiler/lowering authority

The semantic layer must attach the selected conditional implementation to the executable use site.

Conceptually extend semantic lowering with evidence equivalent to:

```rust
pub struct ConditionalBehaviorSelection {
    pub callable: CallableId,
    pub impl_id: ImplId,
    pub declaring_target: InherentImplTargetId,
    pub specialization: InherentImplSpecialization,
}
```

The compiler consumes this product. It must not:

```text
scan impl blocks
re-match type heads
re-check where constraints
pick the most specific candidate
look up methods by source name alone
```

### 7.17 Runtime installation rule

Conditional members must **not** be installed unconditionally into the shared runtime behavior class for their nominal declaration.

Wrong:

```text
impl Box<Int> { intOnly ... }
    ↓
install intOnly on shared Box class
```

That would make the method physically visible on `Box<String>` and break semantic/runtime agreement.

Compile conditional method bodies into executable callables/closures associated with their semantic definition/provenance, but keep them out of the ordinary unconditional method map.

### 7.18 Executable conditional dispatch

A selected conditional send needs runtime behavior equivalent to:

```text
selected owner = semantic declaring owner
selected conditional implementation = semantic lowering target

at runtime:
  search receiver hierarchy for an ordinary override below selected owner
  if found:
      invoke override
  else:
      invoke selected conditional implementation
```

Do not continue ordinary lookup above the selected owner before invoking the conditional fallback; when applicable, the conditional member semantically occupies that owner's slot and shadows ancestors.

Implementation options are mechanically flexible:

- extend an existing resolved/selected invocation primitive if it preserves the required override semantics;
- introduce a narrow conditional-send bytecode/runtime helper;
- use a compiler-owned call target plus runtime override probe.

**STOP AND CONSULT** if the only apparent solution is per-applied-type runtime classes, per-instance generic method tables, or globally installing the method on the shared class.

### 7.19 Method references and callable values

`&object.method(_)`, `&object.method(...)`, getters, setters, index members, and behavioral family/reference operations must use the same conditional selection authority.

A bound reference must capture behavior equivalent to the conditional-send rule, not merely the raw fallback body if a more-derived runtime override can exist.

Do not restore obsolete callable reference syntax.

### 7.20 Class-side conditional behavior

P3 applies to P1's class-side inherent behavior as well.

A specialized class-side member is available only when the class/type receiver semantically proves the impl domain.

Do not create one runtime class object per applied type merely for P3. Statically selected class-side calls/references should carry the semantic conditional target through lowering using the same conditional-dispatch architecture where applicable.

If the landed runtime cannot preserve required class-side override behavior without a larger applied-class runtime redesign, STOP AND CONSULT and isolate the smallest explicit deferred edge rather than weakening semantics.

### 7.21 Dynamic boundary

If a conditional method is not present on the unconditional runtime class table and the receiver has been erased to `Dynamic`, an unproven dynamic send may fail through ordinary runtime missing-method behavior.

P3 must not fabricate generic applicability from runtime class identity.

This is consistent with the rule that conditional member availability is a static semantic capability requiring evidence.

### 7.22 Incremental architecture

Separate fingerprints/dependencies for:

```text
impl domain/header
member signature template
member body
conditional member index
receiver-effective lookup result
```

Desired invalidation:

```text
body-only edit
    -> body/lowering dependents
    -> NOT impl-domain applicability index

target-head edit
    -> domain + receiver-effective lookups for affected target

where-clause edit
    -> domain + receiver-effective lookups for affected target

selector/signature edit
    -> affected conditional member index/call sites
```

Avoid module-wide invalidation where target/selector indexing can keep the dependency local.

---

## 8. Ownership boundaries

### 8.1 P3 owns

```text
specialized applied inherent impl heads
repeated/mixed/nested structural impl heads
where-conditioned inherent impl applicability
canonical impl-domain product
structural receiver-head matcher
applicability proof/evidence
conditional member index
receiver-effective conditional lookup
conditional body environments
conditional semantic lowering
conditional executable dispatch
P2 exact-case + P3 constraint composition
conditional enum requirement coverage when bounded proof permits
conditional tooling/incremental integration
C2 checkpoint closure evidence
```

### 8.2 P3 does not own

```text
basic impl syntax or ImplId creation                      P1
unconditional effective surface architecture             P1
variants-only enum migration                              P2
exact-case identity / closed enum contract semantics      P2
trait declaration semantics                               C3
trait conformance / witnesses / orphan rules              C4
associated types                                          C5
trait constraints / conditional conformance               C6
full impl/reflection API                                  C7
legacy derived behavior policy                            C8
workspace-wide program closure                            C9
per-applied-type class storage                            separate type/runtime program unless later ratified
```

### 8.3 Source-of-truth table

| Concern | Source of truth | Forbidden competing authority |
|---|---|---|
| Impl provenance | `ImplId` | source range/string identity |
| Target owner | landed P1/P2 resolved impl target | compiler global-name lookup |
| Exact-case target | `VariantId` | runtime case class / discriminant |
| Impl generic binders | `TypeParameterOwner::Impl` | anonymous solver IDs |
| Impl domain | new canonical semantic domain product | AST predicates at call sites |
| Receiver owner projection | existing receiver specialization | P3-specific inheritance walker |
| Constraint meaning | existing `GenericConstraint` + relation/constraint checker | custom P3 solver |
| Unconditional members | P1/P2 effective `DeclarationSurface` | conditional index |
| Conditional members | P3 target-indexed conditional member product | unconditional surface mutation |
| Callable identity | canonical owner + selector + side | impl domain / type arguments |
| Conditional selection | semantic receiver-effective dispatch | compiler/LSP re-resolution |
| Enum requirement identity | P2 `EnumRequirementId` | selected witness callable |
| GADT facts | P2/current `CaseTypeEnvironment` | P3 head matcher |
| Runtime ordinary override | runtime class hierarchy | semantic type head alone |
| Runtime conditional fallback | lowering-selected implementation | shared class method insertion |

---

## 9. Global invariants

I-01. `ImplId` remains provenance identity, never callable identity.

I-02. Specialized type arguments never become part of `CallableId` or selector identity.

I-03. A conditional member is not inserted into unconditional `DeclarationSurface`.

I-04. P1/P2 unconditional lookup remains the fast/base path.

I-05. Structural head matching does not use subtype relation to match concrete head components.

I-06. Receiver inheritance projection reuses the canonical receiver-specialization system.

I-07. Every applicability-relevant impl parameter is bound from the target head.

I-08. Bounds/constraints restrict a bound parameter; they do not invent candidates.

I-09. Repeated impl parameters impose equality of matched receiver components.

I-10. Blocked/unknown applicability never silently becomes applicable.

I-11. Same-owner selector uniqueness remains global across primary/unconditional/conditional inherent definitions.

I-12. P3 introduces no most-specific or last-wins method selection.

I-13. Different exact cases remain distinct callable-owner domains.

I-14. Conditional signature templates may retain impl-owned parameters; resolved receiver signatures may not retain unsolved impl-owned applicability parameters.

I-15. Callable-owned method generics remain independent of impl-owned generics.

I-16. Conditional body checking has `Self` bound to the target head/exact-case template and has impl constraints available as assumptions.

I-17. P2 GADT/variant-local generic evidence remains authoritative for exact-case semantics.

I-18. A member present only on one arm of a static union is not generally available on the union.

I-19. A conditional member is not dynamically discoverable merely because a runtime value's class is compatible with the nominal owner.

I-20. Conditional methods are not globally installed on shared generic runtime behavior classes.

I-21. Runtime dispatch preserves ordinary more-derived overrides before using the selected conditional fallback.

I-22. When applicable, the conditional fallback shadows ordinary same-selector ancestors above its declaring owner.

I-23. Compiler lowering consumes semantic selection evidence and does not re-solve applicability.

I-24. Bound method/family references use the same conditional selection semantics as direct sends.

I-25. Class-side conditional behavior does not force per-applied-type class object allocation.

I-26. Same-module inherent ownership remains unchanged.

I-27. Closed enum requirements remain enum contracts, not traits.

I-28. Exact-case impl targeting remains P2 exact-case semantics, not ordinary P3 nominal specialization.

I-29. Body-only edits do not invalidate impl-domain fingerprints.

I-30. Target-head/constraint edits invalidate conditional applicability dependents.

I-31. LSP/source tooling consumes canonical semantic targets/evidence rather than parsing impl heads independently.

I-32. Runtime representation identity is not promoted into semantic impl-domain identity.

I-33. No per-send scan over all source impl fragments is introduced.

I-34. No per-value generic argument array or conditional method table is introduced by P3.

I-35. P3 does not introduce trait conformance semantics.

I-36. C2 completion does not imply LANG005 release completion.

---

## 10. Non-goals

Do not implement in this plan:

- trait declarations;
- `impl Trait for T`;
- trait constraints in impl `where` clauses;
- associated types or projection normalization;
- specialization ordering / most-specific same-selector implementations;
- negative impls;
- open-world extension methods;
- cross-module inherent impl ownership;
- package-level orphan/coherence policy;
- runtime reflection API for impl domains;
- per-applied-type runtime classes or class-side storage;
- dynamic runtime discovery of erased conditional impl members;
- new generic constraint syntax beyond currently canonical forms;
- method overloading by argument/return type;
- general theorem proving for arbitrary impl-domain implication;
- performance optimization beyond preventing obvious pathological lookup/runtime costs.

---

## 11. Expected impact map

### 11.1 Semantic core

Primary expected areas after P1/P2 land:

```text
P1/P2 impl target/contribution module(s)
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
```

P2 enum integration areas only where conditional domains require composition:

```text
phalcom-semantic/src/checker/enum_behavior.rs or landed equivalent
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/types/case_environment.rs
```

### 11.2 Compiler/runtime

Expected:

```text
phalcom-core/src/modules/semantic_lowering.rs
P1/P2 impl compiler module(s)
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/bytecode.rs if a new conditional-send primitive is necessary
phalcom-core/src/vm/dispatch.rs
possibly method/callable binding support for conditional references
```

Do not touch ADT/product physical representation unless evidence proves the conditional dispatch seam cannot use existing behavior-class identity.

### 11.3 Tooling/incremental

Expected:

```text
phalcom-semantic/src/source_index/*
phalcom-lsp consumers of member completion/definition if they directly assume DeclarationSurface
incremental query/fingerprint tests
```

### 11.4 Tests

Prefer extending landed P1/P2 test modules. Expected categories:

```text
semantic inherent impl/applicability
semantic receiver specialization
semantic generic constraints
semantic enum exact-case/GADT
semantic unions/call application
semantic incremental/source index
core class/inheritance dispatch
core data behavior
core ADT behavior
```

### 11.5 Unexpected-touch rule

If implementation requires edits in a major subsystem not listed above, first determine whether the need is a mechanical caller update or an architectural dependency. Architectural expansion triggers consultation.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer may not change without consultation:

- structural rather than subtype head matching;
- all applicability parameters bound from the receiver head;
- constraints restrict rather than infer missing impl parameters;
- unconditional and conditional surfaces remain distinct;
- same-owner selector uniqueness;
- no most-specific same-selector specialization;
- same-module ownership;
- exact-case identity remains `VariantId`-based;
- GADT case environment remains authoritative;
- compiler consumes semantic selection;
- conditional members are not globally installed on shared generic classes;
- subclass override preservation for executable conditional calls;
- no per-applied-type runtime class requirement introduced by stealth;
- no trait semantics.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact Rust product/type names;
- whether domain IDs are separate interned IDs or `ImplId`-keyed products;
- internal map/index structure;
- whether `DeclarationSurface` API gets a companion resolver or dispatch resolver grows a new conditional index;
- exact diagnostic names/wording consistent with repository conventions;
- exact query keys/fingerprint wrappers;
- exact compiler helper/bytecode name;
- exact test module placement;
- whether conditional requirement coverage is implemented in `enum_requirements.rs` or an impl-domain helper consumed from there.

### 12.3 VERIFY-FIRST

Inspect before choosing:

- landed P1/P2 target/product names;
- whether `CallableId` owner representation changed in P2;
- current receiver-specialization API and ambient constraint evidence path;
- current union dispatch architecture;
- current semantic lowering attachment model;
- whether an existing resolved invocation primitive can preserve required runtime override semantics;
- how bound method references are lowered;
- class-side runtime override lookup mechanics;
- current incremental query granularity;
- current LSP member-completion API.

---

## 13. Global STOP / CONSULT triggers

STOP AND CONSULT if any of these occurs:

1. P1/P2 landed with materially different identity/effective-surface architecture.
2. Exact-case impls do not expose a reusable canonical target identity.
3. Conditional applicability appears to require changing selector or `CallableId` identity.
4. Supporting a requested case would require a most-specific/priority specialization rule.
5. A proposed matcher uses subtype matching for concrete target-head components.
6. Applicability needs to solve caller-owned generic variables merely to make an impl fit.
7. Constraint failure/unknown is being treated as success.
8. Same-owner duplicate selectors are being retained as multiple runtime candidates.
9. The only apparent runtime design installs conditional methods on the shared generic class.
10. The only apparent runtime design requires per-value generic method tables.
11. The only apparent runtime design requires per-applied-type behavior classes/class objects.
12. Conditional dispatch cannot preserve ordinary subclass overriding without a materially wider runtime redesign.
13. Class-side conditional behavior requires executable applied-class state beyond the bounded selected-call mechanism.
14. Conditional enum requirement coverage needs a substantially stronger implication solver.
15. Compiler or LSP code starts re-parsing/re-solving impl applicability from AST.
16. P3 would weaken P1 same-module ownership.
17. Trait coherence/orphan/conformance rules begin leaking into P3.
18. Semantic results become source-order dependent.
19. Lookup or runtime dispatch becomes O(all impl fragments) per send.
20. A persistent semantic failure survives two focused hypotheses without narrowing the cause.

### Consultation packet format

Record only reviewable facts:

```text
Trigger:
Observed repository interface:
Expected invariant:
Minimal reproducer:
Focused test/command:
Competing implementation options:
Why plan authority is insufficient:
Recommended decision:
```

Do not include private chain-of-thought.

---

## 14. Debugging budget

### Mechanical failures

For compile errors, renamed symbols, moved files, or simple API drift:

1. inspect the local definition and direct callers;
2. adapt the mechanical edit;
3. rerun the exact failing focused command;
4. after two unsuccessful iterations, classify whether the problem is actually semantic/architectural.

### Semantic failures

For wrong applicability, wrong member visibility, generic mismatch, incorrect dispatch, or unexpected diagnostics:

1. reduce to one exact receiver + one impl domain + one selector;
2. inspect the semantic product at the earliest wrong boundary;
3. formulate one hypothesis;
4. change the owner layer, not a downstream symptom;
5. rerun the narrow reproducer;
6. after two failed hypotheses, STOP AND CONSULT.

### Architectural failures

Immediately consult on identity, coherence, runtime representation, selector-overload, or semantic-authority conflicts. Do not use the debugging budget to redesign the language.

---

## 15. Testing surface analysis

The plan must design broad coverage but execute only the smallest evidence needed at each stage.

### Coverage obligations

| ID | Invariant / scenario |
|---|---|
| CA-01 | `impl Point<Int>` applies to `Point<Int>` |
| CA-02 | concrete head does not subtype-match `Point<SubInt>` into `Point<Int>` |
| CA-03 | `impl<T> Pair<T,T>` accepts equal canonical arguments |
| CA-04 | repeated parameter rejects unequal arguments |
| CA-05 | `impl<T> Pair<T,Int>` binds T and checks concrete second slot |
| CA-06 | nested head such as `Wrapper<List<T>>` binds structurally |
| CA-07 | unused/unbound impl parameter is rejected |
| CA-08 | covering P1 impl behavior remains unconditional and unchanged |
| CA-09 | satisfied `where` constraint enables member |
| CA-10 | violated `where` constraint hides member |
| CA-11 | symbolic receiver with ambient proof enables member |
| CA-12 | symbolic receiver without proof does not enable member |
| CA-13 | blocked/unknown constraint does not become applicable |
| CA-14 | transformed inherited receiver specializes to owner before head matching |
| CA-15 | primary/unconditional same selector conflicts with conditional definition |
| CA-16 | two conditional same-owner same-selector definitions conflict regardless of domain |
| CA-17 | overlapping domains with different selectors coexist |
| CA-18 | same selector on distinct exact cases is allowed |
| CA-19 | resolved signature substitutes all impl-owned applicability parameters |
| CA-20 | callable-owned method generics remain callable-owned after impl specialization |
| CA-21 | body `Self` reflects conditional target head |
| CA-22 | body can use impl `where` assumptions |
| CA-23 | body can call same-domain conditional peer |
| CA-24 | body cannot call another conditional member whose domain is not proved |
| CA-25 | union with one unavailable arm rejects normal send |
| CA-26 | union with all compatible applicable arms succeeds |
| CA-27 | exact-case constrained impl composes with P2 `VariantId` identity |
| CA-28 | GADT case environment composes before P3 constraint check |
| CA-29 | case-only conditional member never leaks to enum root |
| CA-30 | conditional enum requirement/default coverage behaves according to §7.15 or explicit consulted defer |
| CA-31 | compiler lowering carries selected ImplId/callable evidence |
| CA-32 | compiler does not install conditional method on shared generic class |
| CA-33 | runtime applicable conditional call executes fallback body |
| CA-34 | runtime more-derived ordinary override beats conditional fallback |
| CA-35 | conditional fallback shadows same-selector ancestor above declaring owner |
| CA-36 | inapplicable receiver follows ordinary surface/runtime behavior |
| CA-37 | bound method reference uses conditional selection semantics |
| CA-38 | getter/setter/index conditional behavior uses same applicability engine |
| CA-39 | class-side conditional member is available only under proven applied receiver |
| CA-40 | Dynamic does not reconstruct erased conditional applicability |
| CA-41 | body-only edit preserves impl-domain/effective-lookup structural fingerprints |
| CA-42 | target-head edit invalidates affected receiver-effective lookups |
| CA-43 | where-clause edit invalidates affected receiver-effective lookups |
| CA-44 | unrelated target impl edit does not invalidate another declaration's conditional index |
| CA-45 | LSP/source definition points to canonical impl-origin callable/provenance |
| CA-46 | completion differs correctly for `Box<Int>` vs `Box<String>` |
| CA-47 | no runtime/source-order dependence |
| CA-48 | no per-send source impl scan |

Several obligations may be proven by one high-value test. Every obligation must be mapped to evidence in the walkthrough.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Run only the exact new/changed test plus the smallest directly affected subsystem filter.

#### STABILIZE MODE

Run coherent semantic/runtime/incremental lanes after the integrated architecture exists.

#### CERTIFY MODE

Run the final C2-focused gates and repository-required formatting/checking. Do not automatically escalate to full LANG005/workspace certification unless the repository workflow requires it.

### Verification ladder

```text
L0 exact unit/test name
L1 focused module/filter
L2 affected crate integration lane
L3 C2 coherent acceptance lanes
L4 workspace/release gates
```

During BUILD, stay at L0/L1 unless the edited API fanout demands L2.

### Mandatory during BUILD

- exact matcher/domain test after matcher edits;
- exact semantic dispatch test after lookup edits;
- exact runtime conditional-send test after runtime lowering edits;
- exact incremental test after fingerprint/dependency edits.

### Mandatory during STABILIZE

Run one coherent lane for each changed authority:

```text
semantic inherent impl/applicability
semantic receiver specialization / generic constraints
semantic enum exact-case/GADT if touched
core conditional dispatch
incremental/source-index
```

### Mandatory during CERTIFY

Run the final focused gates in Section 19 plus:

```sh
cargo fmt --all -- --check
```

Run clippy/check only when required by canonical repository workflow or when the changed Rust APIs create compiler-warning risk not exercised by tests.

### Explicitly do not run repeatedly

```text
full workspace tests
full workspace clippy
full language corpus
all ADT suites after every semantic edit
all compiler/runtime suites after every matcher edit
```

---

## 17. Baseline / unrelated failure policy

Classify failures:

```text
A — caused by current P3 edit
B — predecessor regression exposed by required P3 path
C — baseline/unrelated repository failure
D — environment/toolchain/transient failure
```

Rules:

- Fix A.
- For B, verify whether the predecessor invariant is mandatory for P3. If yes, stop and repair predecessor under explicit checkpoint bookkeeping; do not hide the work inside P3.
- Record C and continue if it does not invalidate P3 evidence.
- Retry D once when transient; otherwise record environment limitation.

Do not opportunistically clean unrelated warnings/tests.

---

## T0 — Verify P1/P2 takeover and establish P3 checkpoint state

### Purpose

Prove P3 is starting from the architecture it is designed to extend.

### Preconditions

P1 and P2 claim implementation completion.

### Consumes

P1/P2 plan, walkthrough, handoff, current C2 checkpoint state, live source.

### Produces

A short takeover record containing exact live interface names/paths and any mechanical drift.

### Required checks

- verify all predecessor interfaces listed in Section 0;
- verify unconditional `DeclarationSurface` does not already contain receiver predicates;
- verify exact-case target remains `VariantId`-based;
- verify compiler gets semantic impl lowering rather than walking impl AST;
- verify current focused P1/P2 acceptance commands still pass or classify failures;
- record active revision/worktree state.

### Tests to run now

Only the smallest P1 and P2 acceptance filters from their handoffs.

### Acceptance

P3 can name one semantic owner for each prerequisite fact and no material drift exists.

### STOP / CONSULT

Any material predecessor architecture mismatch.

---

## T1 — Canonicalize specialized impl heads and introduce first-class impl domains

### Purpose

Turn formerly rejected P3 heads into durable semantic domains without changing lookup yet.

### Owned areas

Primarily landed P1 impl target/generic modules plus type-parameter/fingerprint support.

### Required implementation shape

1. Extend resolved impl target/applicability representation with a conditional domain form.
2. Resolve specialized target heads to one canonical nominal declaration or P2 exact-case target.
3. Lower target arguments into canonical semantic type/template forms using impl-owned parameters.
4. Preserve the full head instead of forcing a covering bijection.
5. Retain existing `where` constraints in the impl-owned `GenericSignature`/domain product.
6. Require every impl-owned parameter to occur in the target head at least once.
7. Keep same-module ownership and non-nominal/alias/storage restrictions from P1.
8. Add structural fingerprints that exclude source spelling/ranges but include target shape, binder kinds, and constraints.
9. Keep P1 unconditional/covering representations valid and fast.

### Forbidden approaches

- encoding specialization in `CallableId`;
- using source strings as domain keys;
- accepting unused applicability parameters because a bound mentions them;
- creating one synthetic declaration per specialization;
- converting exact-case targets to declaration targets.

### Tests to add/run

Exact tests for CA-01, CA-03, CA-05, CA-06, CA-07 and fingerprint stability.

Then run only the landed inherent-impl target-resolution filter.

### Acceptance

Specialized/constrained heads publish canonical domains but are not yet visible to ordinary dispatch.

### Local STOP / CONSULT

Canonical type lowering cannot represent the head without AST retention; impl-owned binder ownership conflicts with existing metadata publication.

---

## T2 — Implement structural head matching, applicability proof, and strict coherence

### Purpose

Create the reusable applicability engine and close member-definition ambiguity before integrating call sites.

### Owned areas

New/reused impl-domain matcher plus existing receiver specialization, type environments, relations, diagnostics, conditional index construction.

### Required implementation shape

1. Project receiver to domain owner through canonical receiver specialization.
2. Structurally match the canonical head recursively.
3. Bind impl-owned parameters into a fresh `TypeEnvironment`.
4. Repeated occurrence of one impl parameter must match the same canonical type.
5. Concrete/nested components require canonical structural equivalence; do not use subtype matching.
6. Evaluate substituted `where` constraints using existing constraint/relation machinery.
7. Return explicit applicability evidence/result.
8. Build a target-indexed conditional member set keyed by canonical owner + side + selector.
9. Reject any same-owner selector already defined by primary, P1/P2 unconditional, or another P3 conditional inherent definition.
10. Allow overlapping impl domains with distinct selectors.
11. Preserve visibility and definition provenance.
12. Add negative diagnostics with source sites for both definitions where repository conventions permit.

### Tests to add/run

CA-02 through CA-18, especially:

```text
concrete head no subtype-match
repeated parameter match/mismatch
nested application
constraint pass/fail/blocked
transformed inheritance
same-selector conflict
non-conflicting overlapping domains
```

Run exact tests first, then the focused P1/P3 inherent-impl semantic module.

### Acceptance

One reusable semantic function can answer whether an impl domain applies to a receiver under current evidence, and callable definition coherence is source-order independent.

### Local STOP / CONSULT

Need for most-specific selection; matcher starts solving ambient generic variables; relation machinery cannot distinguish blocked from false.

---

## T3 — Integrate receiver-effective dispatch, signature specialization, and body checking

### Purpose

Make conditional members part of static semantics without contaminating unconditional surfaces.

### Required implementation shape

1. Extend canonical dispatch/member resolution to consult unconditional surface first and conditional member index at the appropriate owner.
2. For one selector, use the unique conditional definition only when its domain is proven applicable.
3. Preserve owner traversal/inheritance semantics.
4. Attach `InherentImplSpecialization` or repository-equivalent evidence to `ResolvedDispatch`/call target.
5. Specialize the member signature with impl bindings before canonical call application.
6. Ensure no unresolved impl-owned applicability binder escapes the resolved signature.
7. Keep callable-local method generics for later normal call inference.
8. In impl body checking, seed `Self`, head equalities, and `where` assumptions.
9. Conditional peer calls resolve through the same receiver-effective lookup.
10. Reuse union-arm dispatch; require compatible success across all non-dynamic arms.
11. Ensure P1 covering/unconditional behavior remains byte-for-byte/semantically equivalent in focused regressions.

### Tests to add/run

CA-19 through CA-26.

Include one hostile test where a caller generic parameter could satisfy the domain only by being solved; member must remain unavailable unless existing evidence proves it.

### Acceptance

Static dispatch and body analysis correctly expose receiver-dependent members with one semantic applicability authority.

### Local STOP / CONSULT

Any need to duplicate generic application logic in dispatch; union logic requires feature-specific weakening to Dynamic.

---

## T4 — Compose conditional applicability with P2 exact cases, GADTs, and closed enum contracts

### Purpose

Close the semantic interaction between P2 enum behavior and P3 conditional domains without turning enum contracts into traits.

### Required implementation shape

1. Permit P3 domains on P2 exact-case targets.
2. Resolve exact case first; then combine enum/declaration bindings, variant-local/GADT environment, impl-head bindings, and impl constraints.
3. Preserve case-only availability rules.
4. Keep root `EnumRequirementId` separate from variant witness `CallableId` and conditional impl provenance.
5. If a bodyful root default is conditional, make it available only on receivers satisfying its domain.
6. For a conditional root requirement, perform bounded domain coverage as described in §7.15.
7. An unconditional case implementation may cover a conditional root requirement if signature compatibility succeeds under requirement assumptions.
8. A conditional case implementation covers the requirement only when requirement assumptions prove the case implementation domain.
9. Do not manufacture multiple same-selector case bodies for disjoint domains; strict same-owner selector coherence still applies.
10. If bounded domain implication cannot be represented soundly, stop before accepting conditional bodyless root members and consult.

### Tests to add/run

CA-27 through CA-30, including at least one GADT-specialized requirement or default.

Run only the affected ADT declaration/requirement filter after exact tests.

### Acceptance

Exact-case conditional behavior composes with existing enum/GADT authority and closed requirement completeness remains sound.

### Local STOP / CONSULT

Requirement coverage needs a new general solver or P2 exact-case identity is insufficient for target indexing.

---

## T5 — Lower conditional selections and implement executable override-preserving conditional dispatch

### Purpose

Make accepted conditional behavior executable without polluting shared runtime class method tables.

### Required implementation shape

1. Extend semantic lowering/use-site attachments so a statically selected conditional behavior carries canonical callable, impl provenance/domain specialization as needed, and declaring target.
2. Compile the conditional member body once per source definition, not per receiver specialization, unless existing compiler architecture proves a bounded specialization artifact is already canonical.
3. Do not install conditional members into the unconditional target behavior class.
4. Identify/reuse the smallest existing runtime primitive that can:
   - inspect the actual receiver's runtime class hierarchy below the semantic declaring owner;
   - invoke a more-derived ordinary override if present;
   - otherwise invoke the selected conditional fallback.
5. If no current primitive has exactly those semantics, add one narrow selected/conditional send path. Keep selector and selected fallback explicit.
6. Ensure the conditional fallback shadows ancestors above its declaring owner.
7. Preserve visibility/access checks from semantic analysis; runtime path is execution, not semantic policy.
8. Keep inline-cache changes bounded. Do not create a global impl-domain runtime registry consulted on every ordinary send.
9. Verify rejected/inapplicable conditional definitions are never runtime-installed.
10. Preserve P1/P2 runtime behavior and C1 data/enum representation.

### Runtime hostile cases

```text
Base defines ordinary foo
conditional impl on Mid defines foo? -> forbidden same-owner only if Base != Mid, so allowed as owner shadow
actual Sub overrides foo -> Sub wins
actual Mid no override -> conditional fallback wins over Base foo
inapplicable receiver -> conditional fallback is never selected
```

### Tests to add/run

CA-31 through CA-36.

Run exact runtime tests first, then one focused core dispatch/inherent-impl lane. Run ADT core lane only if exact-case runtime path changed.

### Acceptance

Semantic and runtime behavior agree for applicable/inapplicable receivers and ordinary subclass overriding.

### Local STOP / CONSULT

Need for per-applied-type runtime classes, per-value generic metadata, or broad VM dispatch redesign.

---

## T6 — Integrate references, behavior families, class-side behavior, source tooling, and incrementality

### Purpose

Close all non-direct-call semantic consumers and ensure receiver-dependent availability is visible to tooling without parallel logic.

### Required implementation shape

1. Route bound method references through receiver-effective conditional selection.
2. Route getter/setter/index/member-family behavior through the same applicability engine.
3. Preserve current callable-reference syntax and selector-family identity.
4. Integrate class-side conditional lookup using the same domain evidence on applied class/type receivers.
5. Do not allocate per-applied runtime class objects for class-side P3 behavior.
6. If executable class-side override preservation cannot use the T5 mechanism, consult rather than silently direct-call.
7. Extend source index/definition provenance so a conditional member resolves to the source impl member and canonical callable.
8. Expose a semantic receiver-effective member/completion view for LSP consumers.
9. LSP must not re-run head/constraint matching independently.
10. Add/fix query keys/fingerprints so domain/header and body dependencies remain separate.
11. Add cold-vs-incremental equivalence tests for target-head, where-clause, selector/signature, and body-only edits.
12. Preserve target isolation: edits to one declaration's conditional impl set should not invalidate unrelated target lookup products.

### Tests to add/run

CA-37 through CA-46.

Run exact source-index/incremental tests plus the smallest LSP-facing test if the LSP crate itself changes.

### Acceptance

Every language surface that can name/use inherent behavior sees the same conditional semantics and incrementality remains fine-grained.

### Local STOP / CONSULT

Tooling needs its own solver; class-side execution requires a wider runtime type-object redesign.

---

## T7 — Hostile coherence/performance audit and legacy-defer cleanup

### Purpose

Attack the architecture rather than merely demonstrating happy paths.

### Required audit cases

1. Many conditional impl fragments on one target with distinct selectors: lookup must be selector-indexed, not scan all impls per send.
2. Many unrelated targets: one target edit must not invalidate all conditional member products.
3. Deep generic inheritance: matcher reuses bounded receiver specialization and respects cancellation/budget mechanisms.
4. Nested target head with repeated binders: no exponential recursive matching.
5. Blocked relation/unknown semantic state: no optimistic applicability.
6. Duplicate selector diagnostics are deterministic under source reordering.
7. Conditional member and inherited ancestor same selector: static/runtime shadowing agrees.
8. More-derived ordinary override: runtime and semantic model agree.
9. Dynamic erasure: conditional member is not recovered from runtime nominal class alone.
10. Exact-case/GADT constraint combinations remain bounded and do not recursively expand exhaustiveness/type spaces.
11. No conditional member definition is copied once per concrete applied receiver.
12. No new per-value generic metadata exists solely for P3 dispatch.

### Negative source searches

Use repository-equivalent searches to prove absence of:

```text
conditional member insertion into unconditional DeclarationSurface
compiler-side impl head parsing/matching
LSP-side impl head parsing/matching
runtime scan over ImplId/InherentImplDomain on ordinary Invoke
specialized CallableId construction using type arguments
most-specific/specialization priority machinery
per-applied-type class allocation introduced by P3
```

### Tests to run

Only exact hostile tests and the smallest affected performance/resource assertions. No benchmark suite unless a real regression is observed.

### Acceptance

No hidden second semantic authority or obvious asymptotic/runtime representation regression remains.

---

## T8 — C2-focused stabilization, checkpoint closure, walkthrough, and handoff

### Purpose

Prove the complete C2 architecture coherently and leave durable state for C3.

### Required actions

1. Run final focused gates in Section 19 serially.
2. Re-run targeted negative searches from T7.
3. Review scoped diff for accidental trait semantics, runtime applied-class work, selector overloading, or duplicate applicability logic.
4. Update the C2 checkpoint record.
5. Create P3 walkthrough.
6. Create P3 handoff that summarizes the entire established C2 impl substrate for C3/C4 planners.
7. Mark P3 `FOCUSED_TESTED` only if all mandatory focused evidence passes.
8. Mark C2 complete only if the repository's checkpoint workflow allows completion with P1/P2/P3 all focused-tested and no unresolved C2 semantic incident.
9. Do not mark LANG005 release complete.

### Acceptance

C3 can begin trait declarations without re-investigating inherent impl identity, target domains, conditional lookup, or enum behavior migration.

---

# 18. Task dependency map

```text
T0 takeover
 ↓
T1 impl domains
 ↓
T2 matcher + coherence
 ↓
T3 static lookup/body semantics
 ├──────────────┐
 ↓              ↓
T4 enum/GADT    T5 executable lowering/runtime
 └──────┬───────┘
        ↓
T6 references/tooling/incrementality
        ↓
T7 hostile audit
        ↓
T8 stabilization + C2 closure
```

T4 and T5 may be implemented in either order after T3 if file ownership is independent, but both must complete before T6 certification.

---

# 19. Verification gates

## G0 — P1/P2 takeover gate

**Purpose:** prove P3 has valid predecessors.

Run the smallest P1/P2 acceptance filters named in their handoffs plus source/interface inspection.

Expected:

```text
PASS: unconditional impl/effective surfaces exist
PASS: variants-only enum + exact-case impl migration exists
PASS: compiler uses semantic impl lowering
```

Failure is predecessor work, not P3 implementation work.

---

## G1 — Impl-domain and matcher gate

**Purpose:** prove conditional heads are canonical and structurally applicable.

Run exact new tests for CA-01 through CA-18, then the focused inherent-impl semantic module.

Expected:

```text
concrete/repeated/mixed/nested heads correct
constraint pass/fail/blocked correct
transformed receiver projection correct
strict same-selector coherence correct
```

Do not broaden after PASS.

---

## G2 — Receiver-effective static semantics gate

**Purpose:** prove conditional availability and specialization are correct at use sites.

Run exact tests for CA-19 through CA-26 and the focused call/receiver-specialization lane.

Expected:

```text
resolved signatures contain correct substitutions
ambient generic proof controls availability
union behavior is sound
body Self/constraints are correct
P1 unconditional behavior unchanged
```

---

## G3 — Enum/exact-case conditional gate

**Purpose:** prove P2 and P3 compose.

Run exact tests CA-27 through CA-30, then:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
```

or the landed P2 focused replacement.

Expected:

```text
VariantId ownership retained
GADT case environment retained
case-only conditional behavior correct
conditional requirement coverage sound or explicitly consulted/deferred
```

---

## G4 — Executable conditional dispatch gate

**Purpose:** prove runtime does not leak specialized members and preserves overrides.

Run exact tests CA-31 through CA-40, then the landed focused core inherent-impl/dispatch filter.

Expected:

```text
conditional method absent from shared class map
applicable fallback executes
more-derived ordinary override wins
ancestor above declaring owner is shadowed
inapplicable receiver does not see fallback
references/class-side paths use same selection semantics
Dynamic does not reconstruct erased applicability
```

Do not run the full core suite after PASS.

---

## G5 — Incremental/tooling gate

**Purpose:** prove dependency and editor semantics.

Run exact CA-41 through CA-46 tests plus the smallest affected incremental/source-index filters.

Expected:

```text
body-only edit does not rebuild domain index
header/where edits invalidate applicable lookups
target isolation preserved
cold/incremental results agree
completion/definition use semantic receiver-effective view
```

---

## G6 — Final C2 focused stabilization gate

Run serially, adapting exact module names to the landed tree:

```sh
# P1/P3 inherent impl semantics
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::impls

# receiver specialization / generic application affected lane
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::receiver_specialization

# P2/P3 enum integration
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations

# executable behavior
RUSTFLAGS='' cargo test -p phalcom-core --test core inherent_impl

# exact incremental/source-index P3 filters
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::inherent_impl

cargo fmt --all -- --check
```

These filter names are the intended P1/P3 test-module conventions. During T0, record the exact landed names; if P1/P2 used a mechanically different module name, substitute that exact recorded name once. Use `-- --list` once to discover a moved filter; do not broaden merely because a test module was renamed.

Only if repository workflow explicitly requires checkpoint-wide certification after these focused gates, run the designated C2 checkpoint command from `AGENTS.md`/checkpoint record.

Expected classification:

```text
P3 source: IMPLEMENTED
P3 verification: FOCUSED_TESTED
C2: COMPLETE / FOCUSED_TESTED when repository checkpoint policy permits
LANG005: still IN_PROGRESS
```

---

## 20. Final focused acceptance

The P3 walkthrough must contain a coverage ledger mapping every CA ID to concrete evidence.

No coverage row may be silently omitted. One high-value test may satisfy multiple rows.

### P3 release-complete criteria

P3 is complete only when:

- specialized/constrained impl heads publish canonical domains;
- structural matching and existing constraint proof determine applicability;
- no applicability parameter is invented from constraints;
- unconditional declaration surfaces remain receiver-independent;
- conditional members use a distinct semantic index;
- same-owner selector uniqueness prevents type-overload/specialization ambiguity;
- inherited receiver projection precedes head matching;
- static dispatch/body checking/union logic consume one conditional authority;
- exact-case/GADT semantics compose without identity collapse;
- conditional closed contracts are sound or an explicitly consulted scoped rejection is documented;
- compiler lowering carries selected behavior evidence;
- conditional behavior is not installed on shared generic runtime classes;
- runtime more-derived override semantics are preserved;
- references/getters/setters/index/class-side paths are closed;
- Dynamic does not reconstruct erased generic applicability;
- incremental/source tooling consumes semantic receiver-effective lookup;
- hostile negative searches are clean;
- walkthrough/handoff/checkpoint state is current;
- no unresolved A/B-classified C2 failure remains.

---

## 21. Performance/resource evidence

P3 adds receiver-dependent lookup, so cost discipline is part of correctness.

Protect these invariants:

```text
ordinary unconditional lookup does not scan conditional impl fragments unnecessarily
conditional lookup is target + selector indexed
head matching operates on canonical type structure with bounded recursion
receiver specialization reuses existing bounded/cancellable path
no per-send global impl registry scan
no per-value generic argument array introduced for P3
no per-value conditional method table
no callable/body clone per concrete receiver application
no per-applied-type runtime class allocation introduced for P3
body-only edits do not rebuild applicability indexes
```

No benchmark suite is mandatory unless focused evidence shows a regression. Add one micro/resource test only when it answers a concrete risk.

---

## 22. Checkpoint bookkeeping

### At P3 start

Record in canonical C2 checkpoint state:

```text
active plan = LANG005.C2.P3
starting revision/worktree
landed P1 interface names
landed P2 interface names
known baseline failures
```

### During P3

Record only durable facts:

- final impl-domain product name/shape;
- final applicability-result/evidence product;
- final conditional member index/query identity;
- final conditional runtime invocation mechanism;
- any approved consultation/amendment;
- conditional enum requirement support/defer decision;
- coherent verification gate completion.

Do not maintain an edit diary.

### At P3 completion

Record:

```text
P3 status/completion/verification
C2 checkpoint status
final conditional impl semantic interfaces
runtime dispatch boundary
selector coherence law
incremental/tooling interfaces
deferred failures
consultations/amendments
next action = LANG005.C3 trait declarations and abstract trait surfaces
```

---

## 23. Walkthrough deliverable

Create:

```text
LANG005.C2.P3-walkthrough.md
```

under the canonical C2 implementation directory.

It must contain:

- final result;
- supported specialized/constrained source examples;
- exact landed impl-domain product and applicability evidence;
- structural head-matching algorithm at an architectural level;
- coherence rule and why same-selector type specialization remains rejected;
- receiver-effective lookup architecture;
- body `Self`/generic/constraint environment;
- P2 exact-case/GADT composition;
- conditional enum requirement/default status;
- compiler semantic lowering product;
- runtime conditional-send mechanism and override semantics;
- proof that conditional methods are not installed on shared generic classes;
- method-reference/getter/setter/index/class-side integration;
- incremental/source-tooling architecture;
- implementation deviations from this plan;
- consultations and decisions;
- tests added;
- tests actually run;
- tests intentionally deferred;
- CA coverage ID -> evidence table;
- negative-search results;
- residual risks for C3/C4/C7/C9.

---

## 24. Handoff deliverable

Create:

```text
LANG005.C2.P3-handoff.md
```

It must be suitable for the next planning/implementation session without reopening C2 design.

Include:

```text
current revision/worktree
C2 completion state
stable P1/P2/P3 interfaces
ImplId vs CallableId identity law
unconditional vs conditional surface law
InherentImplDomain/product name and ownership
applicability proof/evidence API
strict same-owner selector coherence rule
exact-case + GADT composition rules
runtime conditional dispatch mechanism
incremental query/fingerprint ownership
source/LSP receiver-effective member API
focused verification commands
known deferred failures
next objective = C3 trait declarations
```

Explicit do-not-redesign list:

- inherent impl is not open-class mutation;
- conditional impl is not selector overloading;
- conditional impl is not trait conformance;
- exact-case impl is not ordinary nominal specialization;
- closed enum requirement is not a trait;
- `ImplId` is provenance, not callable identity;
- conditional members do not belong in unconditional `DeclarationSurface`;
- runtime `ClassId` is not semantic impl-domain identity;
- generic applicability is not reconstructed from runtime values;
- trait witness selection must later remain separate from inherent callable lookup.

---

## 25. Completion truth table

Do not conflate:

```text
source written
!=
exact tests pass
!=
focused semantic lanes pass
!=
runtime conditional dispatch proven
!=
incremental/tooling proven
!=
P3 accepted
!=
C2 checkpoint accepted
!=
LANG005 release certified
```

Expected successful P3 metadata:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Expected successful C2 metadata after all three plans satisfy checkpoint policy:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

LANG005 remains in progress; C3 follows.

---

## 26. Plan self-review

### Architecture

- [x] Conditional availability is not flattened into receiver-independent declaration surfaces.
- [x] Impl domain is a first-class semantic product.
- [x] Head matching is structurally defined and separate from subtype constraints.
- [x] Existing receiver specialization remains inheritance authority.
- [x] Existing generic constraint machinery remains proof authority.
- [x] Callable/selector identity remains stable and non-specialized.
- [x] Strict coherence avoids unratified most-specific dispatch.
- [x] P2 exact-case/GADT semantics remain distinct and reusable.
- [x] Compiler/runtime consume semantic selection rather than source fragments.
- [x] Conditional methods are not installed on shared generic classes.
- [x] Runtime subclass overriding is explicitly preserved.
- [x] Dynamic erasure boundary is explicit.

### Luna executability

- [x] Hard P1/P2 takeover gate prevents work on the wrong baseline.
- [x] Fixed/flexible/verify-first boundaries are explicit.
- [x] Tasks follow semantic authority boundaries rather than file categories.
- [x] Objective STOP/CONSULT conditions prevent accidental language redesign.
- [x] Conditional enum requirement complexity has a bounded consultation threshold.
- [x] Runtime mechanism is semantically fixed while mechanical bytecode/helper choice remains flexible.

### Testing

- [x] Coverage spans target heads, constraints, symbolic generics, inheritance, coherence, unions, bodies, enums/GADTs, runtime dispatch, references, class-side behavior, incrementality, and tooling.
- [x] BUILD testing remains narrow.
- [x] Final certification is C2-focused rather than workspace-wide by default.
- [x] Negative searches guard duplicate semantic authority and runtime leakage.
- [x] Baseline failure classification is explicit.

### Documentation lifecycle

- [x] P3 closes C2 rather than silently expanding into C3/C4.
- [x] C2 checkpoint bookkeeping is mandatory.
- [x] Walkthrough and handoff are mandatory.
- [x] Handoff explicitly preserves inherent-vs-trait boundaries for the next checkpoints.
