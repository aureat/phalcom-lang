---
id: LANG005.C3.P1.requirements
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: requirements-analysis
status: COMPLETE_FOR_PLANNING
prepared: 2026-09-14
repository: aureat/phalcom-lang
branch: main
repository_revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
predecessor: LANG005.C2
plan_target: LANG005.C3.P1
---

# LANG005.C3.P1 — Requirements Analysis

## First-Class Trait Declarations and Abstract Trait Surfaces

This document is the fresh requirements analysis for `LANG005.C3.P1`. It replaces the pre-final-C2 assumptions that shaped the earlier C3 planning pass. It is intentionally repository-grounded: it consumes the landed C1/C2 architecture at revision `3d4d1a855337738eb86a056721e8c52f1cf3c18b`, the final C1/C2 walkthroughs and handoffs, the revised C1+C2 implementation audit, the current C3 checkpoint and guidance, the existing C3.P1 plan, the live Luna planning schema, canonical Phalcom specification material, and the current source tree.

The requirements analysis is complete before the implementation plan is derived. It determines what C3.P1 actually has to implement now, which predecessor defects must be repaired before trait semantics may depend on them, which audit findings remain explicitly deferred, and which architectural decisions are fixed for the Luna implementer.

---

# 1. Executive Conclusion

`LANG005.C3.P1` can still be a single checkpoint-closing implementation package, but it must no longer begin by parsing `trait`. The final C1/C2 delivery established most of the intended architecture, yet the post-completion audit proved three predecessor correctness boundaries that matter enough to C3 to integrate before trait semantics are allowed to build on them:

1. **C1 exact anonymous-product runtime reification must be completed end-to-end.** `RuntimeTypeRecipe`, runtime type environments, and recipe instantiation exist, but statically known anonymous-product lowering does not carry a type recipe and materialization therefore does not attach the exact instantiated runtime type.
2. **C2 conditional exact-enum-case products must be published and invalidated by full target identity.** Consumers already query `InherentImplTarget::ExactEnumCase`, but canonical publication currently registers only declaration-target conditional sets, and removal is correspondingly incomplete.
3. **Dynamic Record construction must canonicalize logical shape in the same way as static construction.** The current dynamic path makes encounter order the logical order via `RecordProductShape::from_ordered_labels`, violating the intended separation between structural logical identity and presentation order.

These are not trait features. They are **takeover repairs** required so C3 is built on truthful identity, representation, and behavioral-surface invariants rather than adapters around known defects.

After those repairs, C3.P1 must implement the actual trait checkpoint:

```text
source `trait` declaration
    ↓
DeclarationId + DeclarationKind::Trait
    ↓
trait header / generic signature
    ↓
TraitRef formation
    ↓
TraitSurface
    ├─ TraitRequirementId
    ├─ trait-owned CallableId
    ├─ CallableSemanticSignature
    ├─ visibility/source
    └─ optional default availability
            ↓
complete surface publication
            ↓
default-body analysis once
under abstract owner-relative Self
with contract-relative member calls
and no executable runtime target
```

The checkpoint remains intentionally semantic-first. A trait is a reusable behavioral/conformance contract, not a class, not an inhabitable nominal type, not a runtime method container, and not conformance evidence. C3 creates no concrete conformance, witness selection, associated type projection, generic conformance constraint, trait object, vtable, or full reflection descriptor.

If the focused implementation and verification obligations in this requirements analysis are satisfied, `LANG005.C3.P1` should close `LANG005.C3`. C4 then begins with explicit conformance and witness construction. However, **C2-F03 (proof-state collapse) is a hard precondition before C4 begins**, because conformance machinery must preserve the distinction between proven, disproven, unknown, blocked, and dynamic evidence rather than reconstructing information discarded by C2.

---

# 2. Current Repository State

## 2.1 Planning baseline

The current remote repository state verified during this planning pass is:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   3d4d1a855337738eb86a056721e8c52f1cf3c18b
commit:     lang005: complete C2 specialized impl applicability
```

The previous C3 planning baseline `986568da050d1bbfa7f1d769c9e1f4d58eb81cf3` is obsolete. It was captured before final C2.P3 delivery.

A remote repository connector cannot establish the future implementer's local working-tree state. Therefore this planning pass does **not** claim a clean `git status`. The implementation plan must begin with the exact local takeover commands:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
```

and must preserve all unrelated user changes. This is a takeover requirement, not an unresolved architecture question.

## 2.2 LANG005 lifecycle state

The live records and the implementation audit disagree in a bounded but important way:

| Checkpoint | Recorded lifecycle state | Source/audit state relevant to C3 | Planning interpretation |
|---|---|---|---|
| `LANG005.C1` | P1–P3 complete / focused tested | Core architecture landed; C1.P3 reification attachment incomplete; dynamic Record logical canonicalization incomplete | Reopen only the required product identity/reification invariants inside C3.P1 takeover remediation |
| `LANG005.C2` | P1–P3 complete / focused tested | P1/P2 sound; P3 central architecture landed; exact-case conditional target publication/removal incomplete | Reopen only the exact-case conditional target lifecycle inside C3.P1 takeover remediation |
| `LANG005.C3` | stale `BLOCKED / NOT_STARTED` lifecycle state that still names `LANG005.C2.P3` as its blocker | C2.P3 is now implemented; C3 source work has not started | Remove the stale C2.P3 blocker; P1 may begin with an internal remediation gate before trait source work |

The C3 checkpoint and guidance still contain the pre-final-C2 baseline and takeover language. They remain useful architectural records, but their lifecycle metadata and C2.P3-specific assumptions must be updated by the implementer at T0.

## 2.3 Missing historical input

The C3 checkpoint, guidance, and previous P1 plan refer to `LANG005.C3.P1-requirements-analysis.md`, but no such file exists in the live C3 directory at the audited revision, and no separate historical requirements-analysis artifact was attached to this planning session. That absence is recorded rather than papered over. This document therefore reconstructs the requirements directly from live source, accepted C3 decisions, predecessor records, and audit evidence.

## 2.4 Live symbols and repository facts verified

The following facts were directly revalidated against the final C1/C2 tree:

| Concern | Live fact | Consequence for C3 |
|---|---|---|
| declaration kind | `phalcom-modules/src/declaration.rs::DeclarationKind` still includes `Protocol`, not `Trait` | `Protocol -> Trait` remains the expected declaration-category migration seam, subject to final compatibility verification |
| statement AST | `phalcom-ast/src/ast.rs::Statement` has `Class`, `Enum`, `Data`, etc., but no `Trait` | C3 must add a first-class trait statement/declaration |
| behavior AST | shared `phalcom_ast::BehaviorMember` remains the canonical behavior-only member category | trait methods/getters/setters/index members must reuse it |
| index bodies | `IndexMethodDef.body` remains `Vec<Statement>` | shared `MemberBody` normalization remains mandatory before bodyless index requirements are complete |
| callable identity | `CallableId { owner, selector, side }` remains canonical | trait member/default source identity reuses `CallableOwnerId::Declaration(trait_decl)` |
| type-parameter ownership | `TypeParameterOwner` supports `Declaration`, `Callable`, `Impl` | trait declaration/member generics require no new owner variant |
| owner-relative Self | `SelfTypeTerm` and `TypeFormationSite::member` remain canonical | trait defaults reuse existing owner-relative Self semantics |
| lexical type binders | `TypeLevelBinding = TypeForm(TypeId) | RecordRow(TypeParameterId)` | do not add `TypeLevelBinding::Trait`; use it only to bind actual generic parameters |
| nominal declaration metadata | `DeclarationTypeTable` and `NominalDeclarationHeader::from_signature` remain coupled to nominal/class-object type formation | trait generic metadata must live in a trait header product, not a fake nominal entry |
| callable body generics | body analysis still reads owner generics via `declarations.generic_signature(callable.declaration_owner())` | body-analysis context must be generalized narrowly so trait defaults can supply owner generics explicitly |
| call application | `CallableApplicationTarget` has `signature`, optional `callable`, optional `InvocationTargetId`, and `CallTargetAuthority` | abstract contract calls can reuse canonical application while carrying no runtime target |
| source target identity | `SemanticTargetId` already has declaration/callable identity | no trait-specific navigation identity is required |
| source declaration kind | `SourceDeclarationKind` currently has class/enum/data/type-alias, no trait | add trait presentation category while preserving canonical semantic target identity |
| C1 anonymous-product lowering | `AnonymousProductConstructionLoweringSpec` contains only `kind` and `layout` | C1-F01 is still reproduced; runtime type recipe projection is absent |
| dynamic Record shape | runtime dynamic construction calls `RecordProductShape::from_ordered_labels` | C1-F02 is still reproduced; presentation order becomes logical order |
| data component operand | `logical_index as u16` remains in compiler and synthesized data getter paths | C1-F03 is still reproduced, but is not a C3 dependency |
| C2 conditional publication | session publication calls `register_conditional_members(InherentImplTarget::Declaration(...))` | C2-F01 is still reproduced |
| C2 conditional invalidation | `SurfaceDispatchResolver::remove_surface` removes only the declaration-target key | C2-F02 is still reproduced |
| C2 proof result | `ImplApplicabilityResult = Applicable | NotApplicable | Blocked` | C2-F03 remains and must be closed before C4, not inside C3 unless implementation unexpectedly depends on it |

## 2.5 Important changes from the previous C3 plan

The previous plan was correct about many trait-specific decisions but wrong about takeover state. The material changes are:

```text
OLD
    C2.P3 not implemented
    C3 blocked on C2.P3
    conditional-surface names partly prospective

NOW
    C2.P3 architecture is implemented
    conditional selection/lowering/runtime boundary is real
    C3 can consume the landed interfaces
    BUT exact-case conditional target publication/removal remains incomplete

OLD
    C1 runtime reification described as completed predecessor infrastructure

NOW
    helper machinery exists but the anonymous-product attachment path is incomplete

OLD
    C3 started with G0 and then trait/index work

NOW
    C3.P1 begins with G0 plus explicit C1/C2 takeover remediation before trait source edits
```

The semantic decisions about trait identity, `TraitRequirementId`, `TraitRef`, `TraitSurface`, abstract `Self`, and the compiler/runtime boundary remain valid after revalidation.

---

# 3. Authority Model

C3 sits at the intersection of language semantics, historical protocol documentation, current implementation architecture, and future conformance design. Those sources have different authority and must not be collapsed.

## 3.1 Canonical language semantics

The governing language taxonomy for LANG005 is:

```text
anonymous (...) / #{...}
    structural transparent immutable products

data
    nominal transparent immutable products

enum
    nominal closed sums of products

class
    opaque nominal object abstractions

trait
    reusable behavioral/conformance contracts
    no owned instance representation
```

The governing identity principle is:

> Semantic category, exact type identity, behavior identity, runtime descriptor identity, and physical representation are distinct concepts and must not collapse.

That principle constrains both the predecessor remediation and the trait design.

## 3.2 Ratified LANG005 decisions

The C3 checkpoint/guidance records accepted design decisions that remain valid unless live source makes them impossible:

- trait is a distinct declaration category, not class sugar;
- declaration identity remains `DeclarationId` plus verified declaration kind;
- `TraitRequirementId` is new semantic contract identity;
- trait source/default callable identity reuses `CallableId`;
- `TraitRef` is a contract reference, not automatically a `TypeId`;
- `TraitSurface` is distinct from all inherent/enum behavior surfaces;
- bodyful trait member means requirement plus default;
- defaults are checked once under abstract `Self`;
- trait-default member calls are contract-relative;
- no owned storage/superclass/runtime class is introduced;
- no conformance/witness machinery is implemented in C3;
- shared index AST must support declaration-only bodies rather than creating trait-only index syntax.

## 3.3 Current implementation architecture

Live source owns mechanical reality: exact names, function signatures, query types, and module placement. Mechanical drift from the prior plan is expected and may be adapted locally.

Live source does **not** override an accepted language rule merely because an implementation shortcut would be easier. If implementing the accepted C3 semantics would require changing ownership, identity, representation, semantic authority, runtime typing, incremental database architecture, or compiler/runtime contracts, the implementer must stop and consult rather than silently reinterpret the design.

## 3.4 Audit findings

The C1+C2 audit is first-class planning authority about predecessor implementation truth. It does not supersede language semantics, but it can disprove checkpoint completion claims and expose an implementation foundation that C3 must repair before depending on it.

## 3.5 Historical protocol design

Protocol-era documents that define `@protocol class`, signature-only protocols, structural conformance, first-class protocol descriptors, or class-side protocol requirements are historical/conflicting authority. They must not silently govern C3. C3 includes a coherent specification migration so the effective specification has one trait model.

## 3.6 Future proposals

C4–C8 support examples and `docs/spec/next/` material are forward-compatibility constraints, not permission to implement future features early. In particular:

```text
C4    conformance + witnesses
C5    associated types/bindings
C6    trait-conformance generic constraints / conditional conformances
C7    reflection/reification
C8    derived behavior
```

The C3 surface should not foreclose them, but it must not implement them.

---

# 4. C1/C2 Audit Integration Matrix

No substantive audit finding is allowed to disappear. The table below assigns a C3.P1 disposition to every meaningful finding and every resolved predecessor concern identified by the revised audit.

## 4.1 Open findings

| Audit finding | Severity / confidence | Owner | C3 dependency | Disposition | Required C3.P1 action |
|---|---|---|---|---|---|
| **C1-F01 — runtime generic/reification closure not wired end-to-end** | HIGH / PROVEN | C1.P3 runtime typing + product lowering/materialization | **Blocking**: C3–C6 rely on trustworthy exact generic identity boundaries | **PRECONDITION / TAKEOVER REPAIR** | Before trait source work, carry semantic exact anonymous-product type as `RuntimeTypeRecipe`, construct/propagate the needed runtime type environment, instantiate at materialization, attach canonical `exact_type`, and preserve the recipe through optimizer rematerialization. Add end-to-end generic product reification tests. |
| **C2-F01 — conditional exact-enum-case consumers have no canonical publisher** | HIGH / PROVEN | C2.P3 conditional surface assembly/publication | **Blocking**: C3 must inherit a truthful behavioral-surface architecture and C4 will compose inherent behavior with trait witnesses/defaults | **PRECONDITION / TAKEOVER REPAIR** | Publish immutable `ConditionalInherentMemberSet` products by full `InherentImplTarget`, including `ExactEnumCase`, from the semantic owner. Do not downstream-special-case checker/compiler/LSP consumers. |
| **C2-F02 — exact-case conditional invalidation not owner-complete** | MEDIUM / PROVEN | C2.P3 dispatch snapshot lifecycle | **Blocking together with C2-F01**: publishing without invalidation creates immediate stale incremental state | **PRECONDITION / TAKEOVER REPAIR** | Land atomically with C2-F01. Track owner→target keys or equivalent bounded ownership so replacement/removal deletes all conditional target products owned by a declaration. Add cold/incremental add/edit/delete/rename constraint cases. |
| **C1-F02 — dynamic Record logical shape not canonicalized** | MEDIUM / PROVEN | C1.P3 representation convergence | **Foundational identity dependency**: structural identity/presentation separation should be correct before richer trait/reflection/associated-type layers build around products | **INTEGRATE INTO C3.P1** | Route dynamic Record formation through the same canonical logical-label/permutation primitive as static construction. Preserve presentation order separately. Test static/dynamic permutations for shared logical identity plus distinct presentation. |
| **C1-F03 — `u32` data component identity narrowed unchecked to `u16` bytecode operand** | MEDIUM / PROVEN | C1 data compiler/runtime projection | No trait dependency; correction could widen executable operand and touch bytecode/VM encoding outside C3 semantic foundation | **DEFER WITH EXPLICIT JUSTIFICATION** | Record in C3 checkpoint/handoff as predecessor correctness debt. Do not widen C3 scope. Require dedicated C1 corrective work with 65,535/65,536 boundary tests before release certification. |
| **C2-F03 — applicability proof outcomes collapse semantic states** | MEDIUM / PROVEN | C2.P3 applicability proof model | C3 trait declaration/default checking does not require inherent proof-state granularity; **C4 conformance absolutely will** | **DEFER WITH HARD PRE-C4 PRECONDITION** | Record in checkpoint and C4 handoff: C4 must not begin witness/conformance proof machinery until applicability/proof state distinguishes Proven, Disproven, Unknown, Blocked, Dynamic (or repository-equivalent) and only Proven can select behavior. |
| **C1-F04 — RecordView bypasses shape lookup cache** | MEDIUM / PROVEN | C1 runtime product access | None for C3 correctness | **DEFER WITH EXPLICIT JUSTIFICATION** | Performance backlog only; avoid per-instance cache workaround. Future fix must use shared shape metadata. |
| **C1-F05 — generic ProductLayout validation O(N²)** | MEDIUM / PROVEN | C1 physical layout validation | None for C3 | **DEFER WITH EXPLICIT JUSTIFICATION** | Performance backlog; future dense constructor or sorted interval validation. |
| **C2-F04 — conditional method lowering scans impl/member inventory by CallableId** | MEDIUM / PROVEN | C2 compiler lowering | C3 creates no concrete trait-conformance lowering, so no new pressure on this path yet | **DEFER WITH EXPLICIT JUSTIFICATION** | Preindex before conformance/behavior inventory grows materially; do not pull compile-time optimization into C3. |
| **C1-F06 — ProductStorage access does not verify same-sized layout identity** | LOW / HIGH CONFIDENCE | C1 storage hardening | No C3 dependency | **DEFER WITH EXPLICIT JUSTIFICATION** | Defensive hardening backlog; no evidence of current user-visible C3 defect. |
| **C2-F05 — impl-domain incremental fingerprinting coarser than semantic product** | LOW/MEDIUM / HIGH CONFIDENCE | C2.P3 incrementality | Correctness-safe over-invalidation; C3 has separate trait products | **DEFER WITH EXPLICIT JUSTIFICATION** | Preserve as future precision/performance work; do not redesign DB in C3. |

## 4.2 Previously raised findings now resolved

| Prior finding | Current state | C3 disposition | Takeover evidence required |
|---|---|---|---|
| **R-F01 — current-main compile regression** | resolved; all-target build succeeds in audited CI | **ALREADY FIXED — VERIFY** | T0 verifies current source builds the affected crates when first coherent gate runs; do not reopen absent regression evidence |
| **R-F02 — bound conditional fallback GC rooting** | resolved via closure/chunk constant-pool rooting | **ALREADY FIXED — VERIFY** | Negative source inspection / existing focused regression; no side-table raw `ObjRef` reintroduction |
| **R-F03 — editor re-solving conditional applicability** | resolved; tooling delegates to semantic receiver-effective query | **ALREADY FIXED — VERIFY** | Preserve semantic query as sole applicability authority when exact-case publication is fixed |
| **R-F04 — conditional selection absent from callable fingerprint** | resolved | **ALREADY FIXED — VERIFY** | Exact-case repair must continue to feed the existing conditional selection fingerprint path |
| **R-F05 — specialized behavior leaked into shared generic runtime class** | not observed; architecture prevents it | **NOT REPRODUCED / ARCHITECTURALLY PREVENTED** | Exact-case remediation must remain semantic target publication; no global method installation or per-applied-type runtime class |
| **R-F06 — direct vs bound conditional runtime selection diverged** | resolved; both use shared VM selection boundary | **ALREADY FIXED — VERIFY** | Do not create a second runtime selection path during remediation |

## 4.3 Repository/release baseline findings

The audit also found repository-wide non-semantic release-gate failures: Nextest inventory/orchestration, rustfmt, and workspace Clippy. These do not establish C1/C2 semantic unsoundness and are not C3 implementation work. They are baseline/release certification concerns. C3 must classify them under the repository A/B/C/D taxonomy if encountered; it must not change unrelated code or weaken assertions to make broad gates green.

## 4.4 Why the remediation boundary stops here

The integrated set is intentionally narrow. C3 repairs:

```text
exact runtime type attachment
structural Record logical identity
conditional exact-case semantic publication/lifecycle
```

because each is an identity/surface foundation that becomes more expensive to repair after traits and conformance evidence exist.

C3 deliberately does **not** absorb bytecode-width cleanup, cache optimization, layout complexity, compiler lookup indexing, storage hardening, or incremental precision improvements merely because C1/C2 are temporarily reopened. This preserves the handoff rule that C3 must not become a general cleanup project.

---

# 5. C3 Dependencies on Final C1/C2

C3 should reuse, not recreate, the semantic infrastructure that final C1/C2 established.

## 5.1 Stable interface map

| Concept | Current live owner / symbol | C3 use | Required invariant |
|---|---|---|---|
| declaration identity | `phalcom_modules::DeclarationId` / semantic re-export | trait declaration identity | one named declaration identity universe |
| declaration category | `phalcom-modules/src/declaration.rs::DeclarationKind` | distinguish trait from class/data/enum/alias | category is metadata on canonical declaration shell, not a second ID universe |
| behavior syntax | `phalcom_ast::BehaviorMember` | trait method/getter/setter/index grammar | one behavior member AST across class/impl/trait contexts |
| callable identity | `phalcom_semantic::CallableId` | trait source/default callable | behavior identity remains owner + selector + side |
| callable owner | `phalcom_semantic::CallableOwnerId::Declaration` | trait member owner | no trait-specific callable-owner variant required |
| impl provenance | `phalcom_semantic::ImplId` | future C4 distinction from trait requirements/witnesses | contribution/domain identity is not callable identity |
| generic owner | `TypeParameterOwner::{Declaration, Callable, Impl}` | trait and member-local binders | no `TypeParameterOwner::Trait` |
| generic signature | `GenericSignature` | trait header and member signatures | stable binder identity remains owner+index |
| lexical type resolver | `TypeResolver` / `TypeLevelBinding` | resolve actual trait generic parameters in signatures/defaults | lexical binder state is not declaration namespace |
| owner-relative Self | `SelfTypeTerm` | abstract trait receiver | no trait-specific Self type system |
| callable signature formation | `semantic_signature_for_syntax_with_resolver(...)` or live equivalent | form trait member signatures | no duplicate trait signature checker |
| call application | `CallableApplicationTarget`, `CallTargetAuthority`, canonical argument/generic application | check calls from defaults | abstract contract call has semantic callable identity but no runtime invocation target |
| ordinary inherent surface | `DeclarationSurface` | explicit non-owner / future C4 input | trait members never contaminate it |
| conditional inherent surface | `ConditionalInherentMemberSet`, `InherentImplTarget`, receiver-effective semantic query | future C4 input and separation boundary | exact-case publication/removal must be repaired first |
| receiver specialization | C2 semantic receiver specialization/applicability path | future conformance composition; not reimplemented in C3 | semantics, not runtime/LSP, owns applied receiver reasoning |
| enum requirement precedent | `EnumRequirementId` / enum behavior products | identity design precedent only | closed-enum requirement remains distinct from reusable trait requirement |
| semantic source target | `SemanticTargetId::{Declaration, Callable}` | definition/reference/navigation | no trait-local navigation identity |
| incremental products | semantic shard / DB query/fingerprint machinery | trait header/surface/body products | cold and incremental results agree; body edits do not poison structural products |
| module declaration shell/interface | `phalcom-modules` interface/declaration tables | trait named binding/import/export | named trait participates in module namespace without implying runtime class |

## 5.2 C1 product identity dependency

C3 itself does not type-check traits by inspecting runtime anonymous products. The reason C1-F01/F02 still block takeover is architectural: LANG005's defining rule is separation of semantic identity from physical representation. Building a new generic contract system on top of a predecessor checkpoint that claims exact generic reification while silently dropping exact product type would normalize the wrong invariant and make C4–C7 retrofits harder.

## 5.3 C2 behavior dependency

C3 does not yet conform types to traits, but the future C4 witness search explicitly composes:

```text
ordinary declared/inherent behavior
C2 receiver-conditional inherent behavior
trait defaults
conformance-local implementations
```

Therefore C3 must hand C4 a truthful C2 target-indexed behavioral-surface foundation. A missing `ExactEnumCase` publisher is not safe to ignore until C4 because it would cause witness candidates to depend on whether behavior happened to be published, not on canonical semantic target identity.

---

# 6. Trait Semantic Category

A Phalcom trait is a named, reusable **behavioral/conformance contract declaration**.

It owns:

```text
DeclarationId
DeclarationKind::Trait
trait generic signature/header
TraitRequirementId values
trait-owned source CallableId values
CallableSemanticSignature values
TraitSurface
optional default source/body analysis
source spans / visibility / diagnostics
incremental fingerprints and dependencies for those products
```

It does **not** own:

```text
instance fields or stored components
ProductLayout / ProductStorage
constructor-managed instance representation
superclass edge
ordinary runtime ClassId
ordinary class object
allocation path
concrete conformance
witness table or witness selection
trait object / existential representation
associated type projections
runtime vtable dispatch
runtime method injection into conformers
```

A property/index requirement is a behavioral capability, not hidden storage.

A trait's semantic category must remain distinct from `class`, `data`, `enum`, inherent `impl`, and the closed enum contract system. Reuse of common identities or syntax does not erase those categories.

---

# 7. Trait Declaration Identity

## 7.1 Canonical rule

The canonical trait declaration identity is:

```text
DeclarationId
+
verified DeclarationKind::Trait
```

No mandatory parallel `TraitId` is justified by the final C2 architecture. `DeclarationId` already owns named declaration identity across module boundaries, and `DeclarationKind` already carries category.

A category-safe wrapper is mechanically acceptable only if it makes invalid API states harder to express, for example:

```rust
struct TraitDeclaration(DeclarationId);
```

with validated construction. Such a wrapper must remain a one-to-one checked view of `DeclarationId`, not become a second canonical identity or require duplicate registries/caches.

## 7.2 `DeclarationKind::Protocol` migration

The live enum still contains `Protocol`. The default C3 action is to rename/replace it with `Trait` after a final compatibility search proves there is no persisted cache schema, serialization ABI, generated metadata contract, plugin API, or other external compatibility dependency.

Keeping both `Protocol` and `Trait` as separate semantic categories merely to avoid migration is forbidden absent a ratified language reason.

If `Protocol` is discovered to be externally serialized or ABI-significant, implementation must stop and consult. That is an architecture/compatibility decision, not a mechanical rename.

---

# 8. Trait Requirement, Default, and Future Witness Identity

Three semantic concepts must remain distinct.

## 8.1 Requirement identity

A trait behavioral member defines a contract requirement independent of whether it has a default body. C3 requires a new identity equivalent to:

```rust
pub struct TraitRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

Exact Rust spelling is mechanically flexible. The identity facts are fixed: owner trait declaration, exact selector signature, dispatch side.

## 8.2 Trait source/default callable identity

The source member and any default body reuse existing callable identity:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_declaration),
    selector,
    side,
}
```

This already owns:

```text
callable-local generics
CallableSemanticSignature
body analysis
source navigation
fingerprints
callable diagnostics
```

No mandatory `TraitMemberId` or `TraitDefaultId` is required.

## 8.3 Future concrete witness identity

C4 may select or synthesize a target-owned callable as the witness for a requirement. That target callable's `CallableId` is not the `TraitRequirementId`, and it is not the trait-owned source callable.

The invariant is:

```text
TraitRequirementId
    != trait-owned CallableId
    != future target witness CallableId
```

A bodyful trait member is therefore:

```text
same TraitRequirementId
+
trait-owned CallableId with default body available
```

not a second unrelated method.

Bodyless ↔ bodyful transition with unchanged selector/side preserves both requirement identity and source callable identity; only default availability/body product changes.

---

# 9. TraitRef

C3 needs a durable semantic reference to an instantiated trait contract:

```rust
pub struct TraitRef {
    pub declaration: DeclarationId,  // validated DeclarationKind::Trait
    pub arguments: Box<[TypeId]>,    // canonical semantic generic arguments
}
```

Equivalent internal storage is allowed.

## 9.1 Formation

Trait reference formation must use named declaration resolution, not lexical `TypeLevelBinding`:

```text
source type-like path
    ↓
canonical declaration lookup
    ↓
DeclarationId
    ↓
verify DeclarationKind::Trait
    ↓
read trait header GenericSignature
    ↓
resolve canonical argument TypeId values
    ↓
validate arity / kind / existing ordinary constraints
    ↓
TraitRef
```

## 9.2 Relationship to TypeId

`TraitRef` is not automatically an inhabitable `TypeId`. C3 must not add `TypeData::Trait` merely to reuse ordinary nominal type application.

A trait may syntactically appear in type-like positions that mean “contract reference” in future features, but the semantic resolver must preserve the category distinction.

## 9.3 Relationship to conformance evidence

`TraitRef` does not prove that any target conforms. It carries only contract declaration identity plus canonical generic arguments. C4 combines `Target + TraitRef + TraitSurface + target behavior` to prove conformance and choose witnesses.

## 9.4 Runtime relationship

`TraitRef` is not a runtime trait object, descriptor instance, witness table, boolean conformance result, or class object in C3.

---

# 10. Generic Trait Signature Ownership

## 10.1 Trait declaration generics

Trait declaration binders use existing ownership:

```text
TypeParameterOwner::Declaration(trait DeclarationId)
```

Their `GenericSignature` must live in a trait header semantic product (name mechanically flexible: `TraitInfo`, `TraitHeader`, or repository-equivalent) keyed by the canonical declaration.

The product must expose enough information for:

```text
TraitRef formation
trait member signature formation
trait default body generic environment
future C4 conformance matching
incremental header fingerprinting
source/editor presentation
```

## 10.2 No fake nominal type entry

`DeclarationTypeTable` still couples generic signature storage to nominal/class-object type forms and supertype metadata. A trait intentionally has neither an ordinary nominal value type nor an ordinary class-object runtime type in C3.

Therefore C3 must **not** call `NominalDeclarationHeader::from_signature` for a trait merely to obtain generic metadata.

## 10.3 Member-local generics

Trait member-local type parameters use:

```text
TypeParameterOwner::Callable(trait-owned CallableId)
```

and the existing callable signature builder/resolver machinery.

## 10.4 Lexical resolver bindings

`TypeLevelBinding` remains exactly what it is: lexical generic binder infrastructure. C3 should populate a scoped resolver with bindings for the trait's actual `TypeParameterId` values and member-local generics, then call the canonical signature builder.

Do not add `TypeLevelBinding::Trait` because a named trait declaration is not a lexical type parameter.

## 10.5 Body analyzer generalization

Current callable body analysis obtains declaration generics through `DeclarationTypeTable`. C3 must generalize this assumption narrowly so the caller can provide owner generic parameters/signature explicitly.

A valid shape is conceptually:

```text
CallableBodyRequest / BodyAnalysisContext
    owner_generic_signature or owner_type_parameters
```

Existing nominal declaration callers may continue deriving this from `DeclarationTypeTable`. Trait default callers provide the trait header signature directly. This is an ownership generalization, not permission to duplicate generic reasoning.

---

# 11. TraitSurface

`TraitSurface` is the central C3 semantic contract product. It must be separate from all runtime/inherent behavior products.

Conceptually:

```rust
pub struct TraitSurface {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub members: /* selector/side keyed contract entries */,
    pub diagnostics: /* stable semantic diagnostics */,
}
```

Each contract entry must expose repository-equivalent facts:

```text
TraitRequirementId
trait-owned CallableId
CallableSemanticSignature
visibility
source provenance/range
default-present flag
optional default source/body handle (not body analysis itself)
```

## 11.1 Surface separation

The following products are deliberately different:

```text
TraitSurface
    reusable abstract contract

DeclarationSurface
    concrete ordinary declared/effective behavior

ConditionalInherentMemberSet
    receiver-domain-conditional inherent behavior

EnumBehaviorProduct / EnumRequirementId
    closed-enum root/case contract semantics
```

C3 must never copy trait requirements/defaults into `DeclarationSurface`, conditional inherent member sets, enum behavior products, or runtime class method tables.

## 11.2 Publication order

Trait member signatures must be resolved before any default body is checked:

```text
parse all members
    ↓
allocate stable requirement/callable identities
    ↓
resolve all member signatures
    ↓
validate duplicate selector/side and member legality
    ↓
publish complete TraitSurface
    ↓
analyze bodyful defaults
```

This guarantees source-order independence. A default may call a requirement or default declared later in the trait.

## 11.3 Duplicate/conflict behavior

Duplicate exact selector + dispatch side is a trait contract error. Diagnostics belong to trait surface construction, not runtime/compiler lowering.

C3 does not define overload-by-pattern or future selector-family matching beyond the language's canonical selector identity model.

---

# 12. Default-Body Checking

Default analysis is the highest-risk trait-specific semantic work because it must reuse the ordinary checker without pretending an executable receiver type or conformance exists.

## 12.1 Analysis environment

A trait default is analyzed once with:

```text
current declaration
    trait DeclarationId

current callable
    trait-owned CallableId

current dispatch side
    instance side for C3-supported trait members

owner generics
    trait header GenericSignature

member generics
    callable GenericSignature

Self
    existing SelfTypeTerm(owner = trait declaration,
                          side = Instance,
                          role = InstanceType)

available receiver behavior
    completed TraitSurface

fields/storage
    none

superclass/super
    none

conformance evidence
    none
```

C3 remains instance-contract-first. Final class-object/metatype conformance semantics are deferred.

## 12.2 Abstract Self member lookup

Ordinary concrete receiver lookup is backed by `DeclarationSurface` and C2 dispatch products. Trait `Self` requires an abstract contract lookup path that reads `TraitSurface`.

This may be a small checker facade or resolver branch, but it must not register `TraitSurface` into `SurfaceDispatchResolver` just to reuse concrete lookup.

Canonical rule:

```text
Self inside trait default
    ↓
TraitSurface selector lookup
    ↓
TraitRequirementId + trait-owned CallableId + signature
```

## 12.3 Canonical callable application reuse

After abstract member selection, argument binding, generic application, expected-result checking, diagnostics, and callable-signature reasoning must reuse the existing callable application checker.

C3 must not implement a second trait-only argument binder or generic solver.

## 12.4 No executable InvocationTargetId

A call such as:

```phalcom
self.compare(other)
```

inside a trait default is a valid semantic contract call, but no concrete target witness exists in C3.

The semantic application therefore carries:

```text
signature = selected contract signature
callable  = Some(trait-owned CallableId)
target    = None
authority = TraitContract / AbstractContract (repository-equivalent)
```

It must not manufacture:

```text
InvocationTargetId::Behavioral(trait-owned callable)
```

or mark the call as ordinary `ExactDispatch`.

The compiler must not be asked to discover later that such a target was fake.

## 12.5 Default-to-default calls remain contract-relative

Even if the selected requirement has a default body, a call from one default to that member resolves to the requirement contract, not directly to the default implementation. C4 may later choose a concrete witness that overrides the default.

## 12.6 Illegal stateful behavior

Trait defaults may not access fields, stored components, `super`, constructor-only storage, or class invariants because the trait owns no instance representation or superclass.

Diagnostics for these violations belong to semantic default checking.

---

# 13. Index Requirements and Shared MemberBody Normalization

The live AST mismatch remains real:

```text
MethodDef.body  -> MemberBody
GetterDef.body  -> MemberBody
SetterDef.body  -> MemberBody
IndexMethodDef.body -> Vec<Statement>
```

LANG005 requires bodyless index getter/setter requirements. C3 therefore must normalize `IndexMethodDef.body` to the shared declaration-or-block representation (`MemberBody` or exact repository equivalent).

## 13.1 Required behavior

Trait context must be able to represent:

```phalcom
trait IntIndexable {
  [_ index: Int] -> Int
}

trait IntMutableIndexable {
  [_ index: Int] -> Int
  [_ index: Int]=(_ value: Int) -> ()
}
```

using current canonical index signature semantics.

## 13.2 Required consumer audit

This is not a parser-only change. The implementer must search and update every production consumer of `IndexMethodDef.body`, including at minimum:

```text
phalcom-ast parser / AST helpers
phalcom-semantic checker/declaration_signature
CallableSyntaxRef::has_body or equivalent
phalcom-semantic session body extraction
semantic_shard structural/body fingerprints
DB/query fingerprint projection
source-index builder / occurrence paths
phalcom-core compiler class/impl index compilation
semantic lowering projections
product optimizer/exhaustive AST walks
attribute/native/generated index member construction
focused tests
```

## 13.3 Context legality

Existing bodyful class and inherent-impl index accessors must remain unchanged. Declaration-only index bodies are accepted only in declaration contexts that semantically permit them. C3 must not accidentally make abstract class/impl index members legal unless a separate language rule already does so.

## 13.4 Setter semantics boundary

C3 owns declaration-only representation, not an index-setter return-value redesign. If live canonical setter semantics conflict with a support fixture, preserve the canonical semantics and scope any redesign separately.

---

# 14. Compiler / Runtime Boundary

C3 is a semantic declaration checkpoint.

## 14.1 Preferred compiler behavior

A `Statement::Trait` is a compile-time/type-level declaration for C3. Module/interface collection makes it a named importable/exportable declaration, but compilation should emit no ordinary runtime class construction.

No C3 trait declaration may cause:

```text
ClassId allocation solely for trait contract semantics
field-slot or ProductLayout allocation
superclass installation
FinalizeClass
ordinary method installation on conformers
VM trait scanning during Invoke
runtime conformance registry
witness/vtable construction
per-generic-application runtime trait class
```

## 14.2 Runtime reification is deferred

C7 owns full trait reflection/reification. C3 preserves stable semantic identities so later reflection can project them, but runtime descriptor objects cannot become semantic authority now.

If module export/linkage genuinely requires a runtime value or descriptor for a trait at C3, implementation must stop and consult. It may not solve the problem by synthesizing a fake class or prematurely designing the C7 descriptor surface.

## 14.3 Compiler semantic authority

The compiler may consume stable semantic trait/no-op lowering facts if necessary to skip runtime emission. It must not reconstruct trait category, member signatures, abstract-call meaning, or conformance semantics from raw AST.

---

# 15. Incremental Architecture

C3 should publish at least three separable semantic products:

```text
TraitHeader / TraitInfo
    declaration category + generic signature + structural header facts

TraitSurface
    complete member contract/signature/default-availability facts

CallableAnalysis for each bodyful default
    body-derived semantic analysis and body fingerprint
```

Exact query-key names are mechanically flexible.

## 15.1 Required invalidation laws

### Body-only default edit

May change:

```text
default CallableAnalysis
callable body fingerprint
diagnostics dependent on body content
```

Must not change:

```text
DeclarationId
TraitRequirementId
trait-owned CallableId
trait header identity
member signature identity
TraitRef identity
unrelated trait members
```

The `TraitSurface` structural fingerprint should not hash full statement contents. It may retain a default-present bit/source fact.

### Signature edit

Changes:

```text
CallableSemanticSignature
TraitSurface fingerprint
dependent TraitRef/member lookups where relevant
```

### Selector/side edit

Changes:

```text
trait-owned CallableId
TraitRequirementId
TraitSurface key
```

### Bodyless ↔ bodyful with same selector/side

Preserves:

```text
TraitRequirementId
trait-owned CallableId
signature identity
```

Changes:

```text
default availability
TraitSurface structural fact
CallableAnalysis existence/body fingerprint
```

### Trait header generic edit

Changes:

```text
trait header GenericSignature
TraitSurface signature formation
TraitRef validation/application
all dependent default analyses
```

## 15.2 C2 exact-case remediation invalidation

C2-F01 and C2-F02 must be fixed together before trait work. Publication/removal must be owner-complete so cold and incremental semantic snapshots contain the same exact-case conditional member sets after add/edit/delete/variant rename/constraint changes.

## 15.3 C1 reification and optimization equivalence

C1-F01 repair must ensure eager product construction and optimizer rematerialization attach the same exact runtime type. Optimized code may not silently drop a runtime type recipe that canonical lowering retains.

---

# 16. Source Index and LSP Boundary

C3 must extend presentation without creating a second semantic model.

## 16.1 Canonical source identities

Use:

```text
trait declaration
    SemanticTargetId::Declaration(trait DeclarationId)

trait member/default source
    SemanticTargetId::Callable(trait-owned CallableId)
```

`TraitRequirementId` is conformance-contract identity, not necessarily the navigation target.

## 16.2 Source declaration presentation

`SourceDeclarationKind` currently lacks Trait. Add a trait category and project it into the existing editor/workspace-symbol model. An Interface-like LSP symbol kind is acceptable if the protocol has no dedicated Trait kind.

Presentation choice must not create `SemanticTargetId::Trait` or a parallel LSP identity.

## 16.3 LSP authority

The LSP consumes semantic source/index products. It must not:

```text
infer trait requirements from AST
re-run generic trait reference validation
resolve abstract Self independently
reconstruct TraitSurface
solve conformance
```

C2's repaired tooling lesson is directly applicable: editor behavior should project canonical semantic authority rather than re-solving it.

---

# 17. Protocol-Era Specification Migration

C3 must leave the repository with one effective language model for reusable behavioral contracts.

## 17.1 Confirmed conflicting cluster

At minimum audit and reconcile:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
docs/spec/typing/03-type-parameters-and-generic-signatures.md
docs/spec/next/phalcom-meta-dispatch-and-type-extension-spec.md
```

The live tree still contains claims that:

```text
@protocol class Name { ... }
creates a first-class Protocol descriptor
protocols are signature-only
protocols provide no defaults
structural conformance is the model
class-side requirements are first-version behavior
```

These conflict with LANG005 C3.

## 17.2 Migration outcome

The effective specification must state the C3 trait model:

```text
trait Name<...> { ... }
bodyless member = requirement
bodyful member = requirement + default
trait owns no instance representation
C3 defines no conformance yet
C3 is instance-contract-first
TraitRef is contract reference, not ordinary value type
```

## 17.3 Document disposition policy

- README/STATUS/index files must point to the new effective rule.
- Protocol-era normative documents must be archived, explicitly marked superseded, or narrowed to historical context according to specification governance.
- Generic-signature examples that use `@protocol class` as active syntax must be migrated to `trait` where they are still normative.
- `docs/spec/next/` protocol claims must be marked superseded/deferred where they compete with accepted LANG005 semantics; next/proposal material must not remain an accidental stronger authority.
- Do **not** globally replace the ordinary English word “protocol” in cursor, API, network, call protocol, or procedural contexts.

## 17.4 Runtime descriptor language

Do not preserve “Protocol descriptor” runtime semantics as a C3 compatibility layer. If later C7 reflection wants a runtime trait descriptor, it will be designed against the final trait identity model.

---

# 18. Non-Goals

C3.P1 must not implement or silently decide:

```text
C4
    impl Trait for Target
    structural/nominal conformance proof
    witness selection
    default selection for a concrete conformer
    coherence / overlap
    conformance-local method identity

C5
    associated types
    associated bindings
    projection normalization

C6
    generic trait-conformance constraints
    T: Trait / conforms constraints
    conditional conformances

C7
    trait objects/existentials
    full runtime trait descriptors
    public reflection/conformance registry
    witness/vtable runtime dispatch

C8
    derived behavior policy
```

Also out of scope:

- final metatype/class-object conformance syntax;
- class-side trait requirement semantics unless a newer ratified decision explicitly supersedes this checkpoint;
- redesigning selector semantics;
- redesigning index setter result semantics;
- C1-F03 bytecode component-width correction;
- C1-F04/F05/F06 performance/hardening;
- C2-F04/F05 performance/incremental precision;
- broad workspace release certification.

---

# 19. Risk Analysis

The risk ranking below drives task order and test allocation.

| Rank | Risk | Failure mode | Required mitigation |
|---:|---|---|---|
| 1 | predecessor identity/surface defect carried forward | traits/conformance later depend on false generic or exact-case behavior foundation | close C1-F01/F02 and C2-F01/F02 before trait source work; G1 gate |
| 2 | trait represented as nominal class/type surrogate | storage/runtime class semantics leak into contract model; fake `TypeId`/ClassId becomes hard dependency | dedicated trait header/`TraitRef`; no `DeclarationTypeTable` entry; negative compiler/runtime checks |
| 3 | requirement/default/witness identity collapse | C4 cannot distinguish contract from implementation and defaults become hard-bound | first-class `TraitRequirementId`; reuse `CallableId` only for source/default callable; identity tests |
| 4 | abstract call becomes executable too early | compiler sees fake `InvocationTargetId`, default dispatch semantics become unsound before witnesses | `CallableApplicationTarget.target = None` + abstract contract authority; semantic-only tests |
| 5 | trait surface contaminates inherent dispatch | ordinary sends or erased runtime classes observe trait members without conformance | `TraitSurface` separate product; negative source/runtime checks |
| 6 | duplicate generic reasoning | trait-specific resolver diverges from canonical inference/application | explicit generic-owner context + canonical resolver/signature/call APIs |
| 7 | default source-order dependence | earlier defaults cannot see later requirements; incremental ordering artifacts | publish complete `TraitSurface` before any default body analysis |
| 8 | index AST normalization regression | existing class/impl index behavior breaks or bodyless indices leak into wrong contexts | shared `MemberBody` conversion + exhaustive consumer audit + adjacent regression |
| 9 | incremental stale state | body/signature/default-presence edits reuse wrong products | separate header/surface/body fingerprints + cold/incremental parity tests |
| 10 | LSP creates local trait semantics | editor disagrees with compiler/checker | source index projects canonical identities/surface only |
| 11 | compiler/runtime semantic duplication | compiler reconstructs abstract trait meaning or runtime scans traits | compile-time no-op boundary + negative code inspection |
| 12 | protocol migration leaves dual authority | future implementer follows conflicting signature-only structural protocol spec | explicit document disposition matrix and effective-spec index update |
| 13 | proof-state debt leaks into C4 | conformance solver cannot distinguish unknown from false | hard pre-C4 checkpoint/handoff condition for C2-F03 |

---

# 20. Testing Requirements

Testing design is broad; execution remains narrow. Every important invariant needs a discriminating case, but BUILD mode must not reflexively run workspace-wide suites.

## 20.1 Audit-remediation coverage

| Coverage ID | Dimension | Invariant | Required case |
|---|---|---|---|
| `AR-01` | generic exact runtime type | exact generic identity survives shared code/layout | generic `make<T>` creates anonymous product values for at least two concrete `T`; descriptors expose distinct correct exact runtime type |
| `AR-02` | nested recipe | recipe recursively instantiates applied/tuple/record/callable/union terms as supported by current runtime recipe | one nested generic product type produces correct canonical runtime type |
| `AR-03` | closure/lifetime | runtime type environment survives lexical capture | escaped block creates product using owner generic after parent returns; exact type remains correct under GC stress where practical |
| `AR-04` | optimizer equivalence | eager and rematerialized products preserve exact type | same source with product optimization enabled/disabled yields identical exact type observation |
| `AR-05` | Record logical canonicalization | presentation order != logical identity | static/dynamic Records with permuted source order share canonical logical shape/type/equality/hash while preserving each presentation order |
| `AR-06` | exact-case conditional publication | full target identity owns conditional set | specialized exact case is applicable for matching applied case and unavailable for wrong args/sibling/root |
| `AR-07` | direct/bound/runtime coherence | repaired publisher feeds existing canonical selection | exact-case specialized member behaves coherently for direct and bound/family access without shared-root leakage |
| `AR-08` | exact-case cold/incremental parity | target removal is owner-complete | add/edit/delete/constraint-change/variant rename or module reload produces same cold and incremental conditional target set |
| `AR-09` | tooling projection | editor consumes semantic exact-case product | completion/member lookup observes specialized exact-case member only when semantic applicability says so |

## 20.2 Trait syntax/identity coverage

| Coverage ID | Dimension | Required case |
|---|---|---|
| `TR-01` | canonical declaration | `trait T { ... }` parses with precise source ranges and creates module declaration shell `DeclarationKind::Trait` |
| `TR-02` | namespace/import/export | trait collides with other declaration names normally and can be imported/exported as named declaration |
| `TR-03` | generic trait header | `trait Comparable<Rhs>` creates declaration-owned stable generic parameter/signature |
| `TR-04` | TraitRef | valid arity/kind/ordinary constraint case forms canonical `TraitRef`; wrong declaration kind and wrong arity/kind diagnose |
| `TR-05` | no nominal trait type | trait use as ordinary inhabitable value type is rejected/deferred according to C3 rule; no nominal/class-object type entry exists |
| `TR-06` | requirement identity | bodyless method requirement has stable `TraitRequirementId` and trait-owned `CallableId`, and they are distinct identities |
| `TR-07` | default identity | bodyful member has same requirement concept plus default attached to same source `CallableId`, not a new `TraitDefaultId` |
| `TR-08` | behavior shapes | bodyless method/getter/setter/index getter/index setter requirements are representable and correctly signature-checked |
| `TR-09` | duplicate selector | duplicate selector+side produces stable trait-surface diagnostic |
| `TR-10` | visibility | member visibility/source metadata is retained in `TraitSurface` |

## 20.3 Default/Self coverage

| Coverage ID | Dimension | Required case |
|---|---|---|
| `DF-01` | `Self` signature | `Self` is valid in supported trait signatures through canonical owner-relative `SelfTypeTerm` |
| `DF-02` | generic default | default body resolves trait declaration generic and member-local generic without nominal trait table entry |
| `DF-03` | requirement call | default calls bodyless requirement; canonical argument/generic checker is used; application has no runtime target |
| `DF-04` | default call | default calls another bodyful member but remains requirement-relative, not hard-bound to its default body |
| `DF-05` | later declaration | default calls member declared later; result independent of source order |
| `DF-06` | return/type diagnostics | default return mismatch and argument mismatch use canonical callable diagnostics |
| `DF-07` | illegal storage | field/stored-component access is rejected because trait owns no storage |
| `DF-08` | illegal super | `super` use is rejected because trait has no superclass |
| `DF-09` | deferred class side | unsupported class-side trait member follows stable deferred/unsupported diagnostic rather than inventing metatype semantics |

## 20.4 Incremental/tooling/compiler coverage

| Coverage ID | Dimension | Required case |
|---|---|---|
| `IN-01` | body edit | changing only a default body preserves declaration/requirement/source-callable/signature identities and unrelated TraitSurface members |
| `IN-02` | signature edit | member signature edit changes TraitSurface and dependent default analysis but not unrelated trait declarations |
| `IN-03` | bodyless↔bodyful | same selector preserves requirement and source callable; default availability/body product changes |
| `IN-04` | header edit | trait generic edit invalidates TraitRef/surface/default dependents appropriately |
| `IN-05` | cold parity | cold and incremental analysis produce identical trait surface/default diagnostics after representative edits |
| `LS-01` | source target | trait definition resolves to `SemanticTargetId::Declaration`; member to `::Callable` |
| `LS-02` | workspace symbol | trait projects through source index with trait/interface-like presentation and no LSP-local identity |
| `CP-01` | compiler no-op | source containing trait declaration compiles/module-loads without ordinary runtime class emission |
| `CP-02` | runtime non-discovery | trait requirements/defaults are not discoverable as ordinary methods on unrelated/conforming-by-shape classes in C3 |
| `CP-03` | enum separation | enum root requirement identity/behavior remains unchanged and distinct from trait requirement machinery |
| `CP-04` | conditional separation | C2 conditional inherent member lookup remains separate from TraitSurface |
| `SP-01` | spec authority | active specification index/STATUS no longer presents `@protocol class` signature-only model as canonical |

## 20.5 Shared index regression coverage

The AST normalization must include focused cases proving:

```text
bodyful class index getter still parses/checks/executes
bodyful class/impl index setter still parses/checks/executes
bodyless trait index getter parses and enters TraitSurface
bodyless trait index setter parses and enters TraitSurface
bodyless index remains rejected in contexts where abstract members are not legal
fingerprints distinguish signature/body structure correctly
```

## 20.6 Test execution doctrine

Use the repository ladder:

```text
T0 — exact new regression / exact reproducer
T1 — directly affected feature tests
T2 — owning subsystem suite
T3 — adjacent integration only when changed code creates plausible risk
T4 — broad crate verification only when evidence warrants
T5 — workspace/release only when explicitly required
```

During BUILD mode do not repeatedly run workspace tests, workspace Clippy, the full language corpus, REPL tests, concurrency tests, or unrelated optimizer suites.

Every filter must be confirmed to select nonzero tests, usually with `-- --list` before relying on a filter as evidence.

---

# 21. STOP / CONSULT Conditions

The Luna implementer must not independently decide any of the following. A trigger cannot be self-waived.

Stop and build a focused consultation packet if:

1. C1-F01 cannot be closed by completing the existing recipe/environment design and instead appears to require a different runtime generic identity architecture.
2. C2-F01/F02 cannot be fixed by canonical full-target publication plus owner-complete invalidation and instead appears to require consumer-specific exact-case logic.
3. Dynamic Record canonicalization would change language-visible presentation/evaluation order rather than only fixing logical identity.
4. an audit finding appears factually wrong or conflicts with live semantics in a way that changes its disposition.
5. trait declarations can only be integrated by making them classes, ordinary nominal `TypeId` values, or runtime class objects.
6. trait generic signatures can only be made available by inserting fake entries in `DeclarationTypeTable`.
7. a new `TraitId`, `TraitMemberId`, `TraitDefaultId`, `TypeParameterOwner::Trait`, or `TypeLevelBinding::Trait` appears necessary for semantic correctness rather than convenience.
8. `TraitSurface` appears to require insertion into `DeclarationSurface`, conditional inherent surfaces, or enum behavior products.
9. default analysis requires a concrete conformer, witness, or conformance proof.
10. abstract contract calls cannot use canonical callable application without manufacturing an executable `InvocationTargetId`.
11. default checking would require compiler/runtime behavior before C4.
12. bodyless index support cannot be achieved by safe shared `MemberBody` normalization and would require trait-only index AST.
13. generic ownership/resolver rules must change beyond passing owner generic context explicitly.
14. the canonical identity model (`DeclarationId`, `CallableId`, requirement identity separation) must change.
15. the runtime representation/reflection boundary must change.
16. the semantic DB/query architecture must be redesigned rather than extended with ordinary trait products/dependencies.
17. module linkage/export genuinely requires a runtime trait object/descriptor in C3.
18. final class-side/metatype trait semantics must be chosen to make P1 work.
19. associated types or conformance constraints become necessary for the C3 core.
20. a newer ratified specification contradicts the adopted trait model.
21. two plausible repairs have materially different ownership/identity/runtime consequences.
22. an unexpected subsystem becomes architecturally necessary.
23. one serious hypothesis-driven correction fails to fix the same semantic defect.
24. passing tests would require weakening any fixed invariant in this document.
25. implementation scope becomes materially more cross-cutting than this requirements analysis predicts.

Mechanical drift—renamed helpers, moved modules, collection types, equivalent private APIs—does not require consultation if ownership and invariants remain unchanged.

---

# 22. Final Requirements Statement

`LANG005.C3.P1` is complete only when the repository satisfies all of the following as one coherent state:

```text
PREDECESSOR FOUNDATION
    C1 anonymous-product exact runtime type is carried end-to-end through
    semantic recipe -> lowering -> runtime environment -> materialized descriptor;
    optimized/rematerialized products preserve the same exact type;
    dynamic Record logical identity is canonical and presentation remains separate;
    C2 conditional exact-case behavior is published by full target identity;
    C2 conditional target replacement/removal is owner-complete and cold/incremental equivalent.

TRAIT CATEGORY
    `trait` is a first-class named declaration;
    declaration identity remains DeclarationId + DeclarationKind::Trait;
    trait has no storage, superclass, ordinary runtime class, or inhabitable nominal TypeId.

CONTRACT IDENTITY
    TraitRequirementId exists and is distinct from trait source/default CallableId
    and future concrete witness CallableId;
    trait member/default source identity reuses canonical CallableId;
    TraitRef is declaration + canonical generic arguments and is not conformance evidence.

GENERIC OWNERSHIP
    trait declaration generics use TypeParameterOwner::Declaration;
    member generics use TypeParameterOwner::Callable;
    trait generic metadata lives in a dedicated trait header product;
    DeclarationTypeTable is not polluted with fake nominal trait entries;
    canonical resolver/signature/application machinery is reused.

TRAIT SURFACE
    TraitSurface is a separate complete contract product;
    all signatures publish before any default body is analyzed;
    bodyless member = requirement;
    bodyful member = same requirement + optional default;
    method/getter/setter/index requirements are supported through shared behavior AST.

DEFAULT SEMANTICS
    defaults are checked once under owner-relative abstract Self;
    trait Self lookup reads TraitSurface;
    member calls reuse canonical callable application;
    abstract contract applications carry semantic callable identity but no executable runtime target;
    default-to-default calls remain requirement-relative;
    storage and super access are illegal.

INCREMENTAL / TOOLING
    header, TraitSurface, and default-body products have correct dependency/fingerprint separation;
    cold and incremental results agree;
    source navigation reuses SemanticTargetId::Declaration / ::Callable;
    LSP projects semantic products rather than reconstructing trait semantics.

COMPILER / RUNTIME
    trait declarations do not allocate ordinary runtime classes or inject methods;
    ordinary VM dispatch remains trait-unaware in C3.

SPECIFICATION
    active protocol-era language-feature claims are reconciled so `trait` is the one effective contract model.

LIFECYCLE
    focused coverage obligations pass or any unrelated failures are truthfully classified;
    C3 checkpoint state is truthful;
    C3.P1 walkthrough exists;
    C4 handoff exists and explicitly carries the C2-F03 pre-C4 proof-state prerequisite.
```

Successful C3.P1 completion closes **trait declarations and abstract trait surfaces**. It does not mean that any target conforms to a trait, that a witness exists, that associated types are implemented, or that LANG005 as a whole is release-certified.
