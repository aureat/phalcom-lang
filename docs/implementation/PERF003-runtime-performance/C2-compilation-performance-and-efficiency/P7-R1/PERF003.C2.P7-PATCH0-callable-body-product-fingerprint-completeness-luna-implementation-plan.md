---
id: PERF003.C2.P7.PATCH0
category: PERF
program: PERF003
checkpoint: PERF003.C2
parent_plan: PERF003.C2.P7
kind: corrective-prerequisite-implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
blocks:
  - PERF003.C2.P7.PATCH1
target_branch: perf/universe-characterization
prepared_against_revision: 21a54705c33dae95ee9c77d86789b4911635e6b0
audit_baseline_revision: 2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
executor_profile: GPT-5.6 Luna High / Extra High
---

# PERF003.C2.P7.PATCH0 — Complete `callable_body_product_fingerprint` Semantic Identity

> **Execution role:** bounded Luna-oriented prerequisite patch.  
> **Parent objective:** `PERF003.C2.P7` persisted semantic compiler image.  
> **Immediate dependency:** PATCH1 must not proceed past its fingerprint-dependent implementation gates until PATCH0 is complete and focused-tested.  
> **Patch size intent:** very small production diff, narrow tests, no artifact schema or allocation redesign.

---

# 0. Executive directive

PATCH0 repairs one specific semantic correctness defect:

```text
phalcom-semantic/src/db/fingerprint.rs
    callable_body_product_fingerprint
```

The callable product fingerprint is currently **not an exhaustive identity of executable callable semantics**.

The live implementation at the prepared revision omits or truncates semantic facts that core lowering consumes:

```text
ExpressionAnalysis.binding_write
CallableAnalysis.family_applications
most AssociatedResolution variants
BoundFamily family_type
BoundFamily selector/spec
several BoundFamily member facts
nested CallableReferenceResolutionKind::Associated payload
```

PATCH1 intends to persist and reconstruct complete `CallableAnalysis` owners, then recompute this fingerprint after relocation.

Therefore PATCH1 cannot safely use the current algorithm as its product identity.

The rule is:

> **Fix semantic fingerprint completeness first. Then persist/import callable owners.**

Do not merge PATCH0 into PATCH1 implementation work.

---

# 1. Repository grounding

## 1.1 Live revision

Planning rechecked the remote branch directly.

```text
repository:
aureat/phalcom-lang

branch:
perf/universe-characterization

HEAD:
21a54705c33dae95ee9c77d86789b4911635e6b0

commit:
fix(perf): close callable analysis type roots
```

Its parent is the audit baseline:

```text
2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
feat(perf): extend artifact loading foundations
```

No later pushed PATCH0 implementation was present when this plan was prepared.

---

# 1.2 Current implementation defects verified from source

Current expression hashing ends with:

```rust
expression.binding_read_safety.hash(&mut hasher);
expression.binding_read.hash(&mut hasher);
```

but does not hash:

```rust
expression.binding_write
```

Current associated-resolution hashing:

```rust
let expansion_field_count = analysis
    .associated_resolutions
    .values()
    .filter(|resolution| {
        matches!(
            &resolution.kind,
            AssociatedResolutionKind::DataExpansionField { .. }
        )
    })
    .count();

expansion_field_count.hash(&mut hasher);

for (expression_id, resolution) in analysis.associated_resolutions.iter() {
    if let AssociatedResolutionKind::DataExpansionField {
        logical_index,
        value_type,
    } = &resolution.kind
    {
        ...
    }
}
```

Therefore every other `AssociatedResolutionKind` is semantically invisible.

Current callable-reference hashing:

```rust
match &resolution.kind {
    CallableReferenceResolutionKind::BoundFamily { members, .. } => {
        0u8.hash(&mut hasher);
        members.len().hash(&mut hasher);
        for member in members.iter() {
            member.operation.hash(&mut hasher);
            member.target.hash(&mut hasher);
            hash_conditional_selected_call_capability(
                member.conditional_capability.as_ref(),
                &mut hasher,
            );
        }
    }
    CallableReferenceResolutionKind::Associated(_) => {
        1u8.hash(&mut hasher)
    }
}
```

This drops:

```text
BoundFamily.family_type
BoundFamily.spec
BoundBehavioralMember.member_kind
BoundBehavioralMember.callable_type
BoundBehavioralMember.conditional
BoundBehavioralMember.trait_dispatch
nested AssociatedResolution contents
```

Current `callable_body_product_fingerprint` has no `family_applications` reference at all.

---

# 1.3 Native owner fields that define the repair

Current `ExpressionAnalysis` includes:

```rust
pub binding_read_safety: Option<BindingReadSafety>,
pub binding_read: Option<BindingId>,
pub binding_write: Option<BindingId>,
```

Current `CallableAnalysis` includes:

```rust
pub associated_resolutions: Arc<AssociatedResolutionIndex>,
pub callable_reference_resolutions: Arc<CallableReferenceResolutionIndex>,
pub family_applications: Arc<FamilyApplicationResolutionIndex>,
```

Current `AssociatedResolution`:

```rust
pub struct AssociatedResolution {
    pub owner_form: TypeId,
    pub lookup_owner: DeclarationId,
    pub family: Option<AssociatedFamilyId>,
    pub kind: AssociatedResolutionKind,
}
```

Current exhaustive variants:

```rust
ExactValue {
    member,
    value_type,
}

DataExpansionField {
    logical_index,
    value_type,
}

ExactCallable {
    member,
    target,
    callable_type,
}

Family {
    family_type,
    members,
}

StaticInvoke {
    member,
    target,
    result_type,
    argument_mapping,
}

StaticDomain {
    member,
    domain,
}

StaticTerm {
    domain,
    case,
    term,
}

DynamicInvoke {
    candidates,
    result_type,
}
```

Current callable-reference owner:

```rust
pub enum CallableReferenceResolutionKind {
    BoundFamily {
        family_type: TypeId,
        spec: BehavioralFamilySpec,
        members: Box<[BoundBehavioralMember]>,
    },

    Associated(Box<AssociatedResolution>),
}
```

Current `BoundBehavioralMember`:

```rust
pub struct BoundBehavioralMember {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub target: InvocationTargetId,
    pub callable_type: TypeId,
    pub conditional: Option<ConditionalDispatchSelection>,
    pub conditional_capability: Option<ConditionalSelectedCallCapability>,
    pub trait_dispatch: Option<TraitDispatchSelection>,
}
```

Current family application:

```rust
pub struct FamilyApplicationResolution {
    pub family_type: TypeId,
    pub kind: FamilyApplicationKind,
    pub selection: FamilyApplicationSelection,
}

pub enum FamilyApplicationKind {
    Associated,
    Behavioral,
}

pub enum FamilyApplicationSelection {
    Static {
        operation,
        target,
        callable_type,
        result_type,
    },

    Dynamic {
        candidates,
        result_type,
    },
}

pub struct FamilyApplicationCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<InvocationTargetId>,
    pub callable_type: TypeId,
}
```

These definitions, not the artifact DTO, determine PATCH0 semantic coverage.

---

# 1.4 Existing test seam

The repository already has the correct integration test home:

```text
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

It already defines:

```rust
fn callable_analysis() -> CallableAnalysis
```

with all relevant resolution maps initialized empty.

Existing callable fingerprint tests already establish important product semantics, including:

- binding state sensitivity;
- expression denotation/status sensitivity;
- resolved callable sensitivity;
- diagnostic cause renumbering insensitivity;
- flow/callable status sensitivity;
- flow evidence sensitivity;
- local binding read sensitivity;
- explanation presentation insensitivity;
- diagnostic detail insensitivity;
- source-range/type-provenance insensitivity;
- flow source-range insensitivity.

PATCH0 must extend this suite rather than introduce a new fingerprint test subsystem.

---

# 1.5 Baseline evidence to preserve exactly

The implementer has already recorded PATCH1 T0 baseline evidence:

```text
semantic callable_analysis filter:
19 / 19 passed
```

and:

```text
phalcom-compiler-artifact callable-analysis test:
9 selected
4 passed
5 failed
```

The five artifact failures are pre-existing failures in the current allocation-boundary and conditional-dispatch validation paths.

PATCH0 must:

```text
record them as inherited baseline
not "fix" them
not broaden into those failure paths
not change their expected classification
```

The artifact test currently contains exactly nine tests.

PATCH0 changes no artifact schema and therefore has no legitimate reason to repair those five failures.

---

# 2. Why PATCH0 blocks PATCH1

PATCH1 must eventually execute:

```text
source CallableAnalysis
    ↓
portable owner graph
    ↓
fresh-process relocation
    ↓
reconstructed CallableAnalysis
    ↓
callable_body_product_fingerprint(...)
```

If the fingerprint does not include executable selections, then two semantically different reconstructed owners can produce one product identity.

That creates at least three correctness hazards:

1. **incremental reuse hazard**

   A changed executable choice may be treated as the same semantic product.

2. **artifact validation hazard**

   A lossy import can accidentally pass fingerprint equivalence because the omitted field is also omitted by the hash.

3. **query-image hazard**

   PATCH4 can inherit a false product fingerprint and reconstruct reusable query evidence for the wrong callable semantics.

Therefore:

```text
PATCH0 incomplete
    => PATCH1 fingerprint-dependent gates BLOCKED
```

PATCH1 may perform unrelated preparatory work only if its own plan explicitly allows it, but it must not declare callable owner parity or fingerprint closure until PATCH0 passes.

---

# 3. Scope

# 3.1 PATCH0 owns

Only:

1. `ExpressionAnalysis.binding_write` fingerprinting;
2. exhaustive `AssociatedResolution` hashing;
3. exhaustive `CallableReferenceResolution` hashing;
4. complete `BoundFamily` family/spec/member hashing;
5. complete `family_applications` hashing;
6. narrowly related helper extraction needed to avoid duplicated semantic hashing;
7. discriminating integration tests;
8. lifecycle/checkpoint note that PATCH1 is blocked/unblocked by PATCH0;
9. preservation of current product-fingerprint invariants.

---

# 3.2 PATCH0 does not own

Do **not** modify:

```text
CallableAnalysis artifact DTOs
SemanticWorldV2 schema
artifact root registration
callable owner ordinals
full CallableAnalysis import
frozen imported map storage
AllocationBudget
allocation transactions
semantic UnaccountedBoundary guards
query-image transport
static checked-body transport
formal projection
core lowering
container format
```

Do not remove:

```text
validate_semantic_world_export_coverage guards
ensure_complete_callable_analysis_payload guard
semantic validation allocation UnaccountedBoundary
semantic import allocation UnaccountedBoundary
```

Do not fix the five inherited artifact failures unless a PATCH0 edit somehow causes a new direct regression in that code, which should be treated as a STOP condition because PATCH0 should not touch it.

---

# 4. Required reads

Read before editing:

```text
AGENTS.md
docs/implementation/README.md

docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md

current P7 plan/checkpoint/walkthrough/handoff

current PATCH1 plan/checkpoint/handoff

phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/types/family.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/trait_dispatch.rs

phalcom-semantic/tests/semantic/incremental/fingerprints.rs

phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs
```

Also read the audit finding set:

```text
S2-04
S2-05
S2-06
S2-07
```

Do not expand PATCH0 into S2-08+.

---

# 5. Normative semantic rule

A product fingerprint must change when the **published executable semantic meaning** of a callable changes.

It should not change for fields intentionally classified as presentation/local numbering noise by the existing fingerprint contract.

PATCH0 therefore follows two rules simultaneously.

## Rule A — add omitted executable dimensions

Hash:

```text
binding write selection
associated resolution selection
family application selection
callable-reference family identity/selector/member semantics
nested associated callable-reference semantics
```

## Rule B — preserve existing intentional insensitivities

Do not begin hashing, solely as part of PATCH0:

```text
source ranges already intentionally omitted
diagnostic prose/presentation
explanation presentation
local DiagnosticCauseId numbering
dependency_fingerprint itself
```

The patch is additive with respect to missing semantic identity.

It must not redefine unrelated fingerprint semantics.

---

# 6. Implementation architecture

# 6.1 Keep `callable_body_product_fingerprint` orchestration-level

Do not inline another large variant tree into the main function.

The repaired main function should conceptually read:

```rust
pub fn callable_body_product_fingerprint(
    analysis: &CallableAnalysis,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();

    // existing fields...

    for (...) in &analysis.expressions {
        ...
        expression.binding_read_safety.hash(&mut hasher);
        expression.binding_read.hash(&mut hasher);
        expression.binding_write.hash(&mut hasher);
    }

    hash_associated_resolutions(
        &analysis.associated_resolutions,
        &mut hasher,
    );

    // existing binding/flow/etc dimensions...

    hash_callable_reference_resolutions(
        &analysis.callable_reference_resolutions,
        &mut hasher,
    );

    hash_family_application_resolutions(
        &analysis.family_applications,
        &mut hasher,
    );

    // existing dimensions...

    finish_product(hasher)
}
```

Exact helper names are mechanically flexible.

The architecture is not.

---

# 6.2 Binding write

The production change is intentionally trivial:

```rust
expression.binding_read_safety.hash(&mut hasher);
expression.binding_read.hash(&mut hasher);
expression.binding_write.hash(&mut hasher);
```

Why direct `Hash` is appropriate:

```text
Option<BindingId>
```

already has deterministic in-process semantic meaning inside one callable product.

PATCH0 is not the portable-ordinal redesign.

PATCH1 remains responsible for relocating/importing the owner correctly before recomputation.

---

# 6.3 Exhaustive associated-resolution helper

Introduce one canonical helper:

```rust
fn hash_associated_resolution(
    resolution: &AssociatedResolution,
    hasher: &mut impl Hasher,
)
```

It must hash the common owner fields first:

```rust
resolution.owner_form.hash(hasher);
resolution.lookup_owner.hash(hasher);
resolution.family.hash(hasher);
```

Then hash one stable explicit discriminant and **every semantic field of the selected variant**.

Recommended structure:

```rust
fn hash_associated_resolution(
    resolution: &AssociatedResolution,
    hasher: &mut impl Hasher,
) {
    resolution.owner_form.hash(hasher);
    resolution.lookup_owner.hash(hasher);
    resolution.family.hash(hasher);

    match &resolution.kind {
        AssociatedResolutionKind::ExactValue {
            member,
            value_type,
        } => {
            0u8.hash(hasher);
            member.hash(hasher);
            value_type.hash(hasher);
        }

        AssociatedResolutionKind::DataExpansionField {
            logical_index,
            value_type,
        } => {
            1u8.hash(hasher);
            logical_index.hash(hasher);
            value_type.hash(hasher);
        }

        AssociatedResolutionKind::ExactCallable {
            member,
            target,
            callable_type,
        } => {
            2u8.hash(hasher);
            member.hash(hasher);
            target.hash(hasher);
            callable_type.hash(hasher);
        }

        AssociatedResolutionKind::Family {
            family_type,
            members,
        } => {
            3u8.hash(hasher);
            family_type.hash(hasher);
            members.len().hash(hasher);
            for member in members.iter() {
                hash_specialized_associated_member(member, hasher);
            }
        }

        AssociatedResolutionKind::StaticInvoke {
            member,
            target,
            result_type,
            argument_mapping,
        } => {
            4u8.hash(hasher);
            member.hash(hasher);
            target.hash(hasher);
            result_type.hash(hasher);
            argument_mapping.hash(hasher);
        }

        AssociatedResolutionKind::StaticDomain {
            member,
            domain,
        } => {
            5u8.hash(hasher);
            member.hash(hasher);
            domain.hash(hasher);
        }

        AssociatedResolutionKind::StaticTerm {
            domain,
            case,
            term,
        } => {
            6u8.hash(hasher);
            domain.hash(hasher);
            case.hash(hasher);
            term.hash(hasher);
        }

        AssociatedResolutionKind::DynamicInvoke {
            candidates,
            result_type,
        } => {
            7u8.hash(hasher);
            candidates.len().hash(hasher);
            for candidate in candidates.iter() {
                hash_specialized_associated_member(candidate, hasher);
            }
            result_type.hash(hasher);
        }
    }
}
```

The exact numeric tags are implementation-private.

Requirements:

```text
closed match
no wildcard arm
no Debug-string hashing
no std::mem::discriminant as a substitute for payload hashing
no special-case-only caller
```

A future new enum variant should produce a compiler error at this helper until fingerprint semantics are explicitly chosen.

---

# 6.4 Specialized associated member helper

Use one helper for both:

```text
AssociatedResolutionKind::Family.members
AssociatedResolutionKind::DynamicInvoke.candidates
```

Recommended:

```rust
fn hash_specialized_associated_member(
    member: &SpecializedAssociatedMember,
    hasher: &mut impl Hasher,
) {
    member.member.hash(hasher);
    member.operation.hash(hasher);
    member.value_type.hash(hasher);
    member.target.hash(hasher);
}
```

Do not hash only target/operation.

`member` identity and specialized value/callable type are semantic choices.

---

# 6.5 Associated-resolution index helper

Replace the old `DataExpansionField` count/special-case block entirely.

Recommended:

```rust
fn hash_associated_resolutions(
    resolutions: &AssociatedResolutionIndex,
    hasher: &mut impl Hasher,
) {
    resolutions.len().hash(hasher);

    for (expression, resolution) in resolutions {
        expression.hash(hasher);
        hash_associated_resolution(resolution, hasher);
    }
}
```

Why include total length:

```text
empty
one ExactValue
one ExactCallable
```

must not rely only on nested payload coincidence.

The BTreeMap already gives canonical owner-local iteration order inside the current native owner.

Do not sort a second temporary vector.

---

# 6.6 Family application helper

Introduce:

```rust
fn hash_family_application_resolution(
    resolution: &FamilyApplicationResolution,
    hasher: &mut impl Hasher,
)
```

Hash:

```text
family_type
FamilyApplicationKind
complete FamilyApplicationSelection
```

Recommended:

```rust
fn hash_family_application_resolution(
    resolution: &FamilyApplicationResolution,
    hasher: &mut impl Hasher,
) {
    resolution.family_type.hash(hasher);

    match resolution.kind {
        FamilyApplicationKind::Associated => 0u8.hash(hasher),
        FamilyApplicationKind::Behavioral => 1u8.hash(hasher),
    }

    hash_family_application_selection(
        &resolution.selection,
        hasher,
    );
}
```

`FamilyApplicationKind` currently does not derive `Hash`, so explicit tags keep this helper exhaustive.

---

# 6.7 Family application selection helper

Recommended:

```rust
fn hash_family_application_selection(
    selection: &FamilyApplicationSelection,
    hasher: &mut impl Hasher,
) {
    match selection {
        FamilyApplicationSelection::Static {
            operation,
            target,
            callable_type,
            result_type,
        } => {
            0u8.hash(hasher);
            operation.hash(hasher);
            target.hash(hasher);
            callable_type.hash(hasher);
            result_type.hash(hasher);
        }

        FamilyApplicationSelection::Dynamic {
            candidates,
            result_type,
        } => {
            1u8.hash(hasher);
            candidates.len().hash(hasher);

            for candidate in candidates.iter() {
                candidate.operation.hash(hasher);
                candidate.target.hash(hasher);
                candidate.callable_type.hash(hasher);
            }

            result_type.hash(hasher);
        }
    }
}
```

Important:

```text
Static.target = None vs Some(...)
must differ.

Dynamic candidate order is semantic because the native product retains a boxed ordered sequence.
Do not sort it unless producer semantics already guarantee order-insensitivity and that is separately proven.
```

PATCH0 does not redesign producer canonicalization.

---

# 6.8 Family application index helper

Recommended:

```rust
fn hash_family_application_resolutions(
    resolutions: &FamilyApplicationResolutionIndex,
    hasher: &mut impl Hasher,
) {
    resolutions.len().hash(hasher);

    for (expression, resolution) in resolutions {
        expression.hash(hasher);
        hash_family_application_resolution(
            resolution,
            hasher,
        );
    }
}
```

Invoke it in `callable_body_product_fingerprint`.

The precise location relative to callable references is not semantically important if stable, but place related associated/reference/family products together for auditability.

Do not change ordering on subsequent refactors casually; product fingerprint compatibility is an internal incremental concern even if not a portable file ABI.

---

# 6.9 Complete callable-reference hashing

Introduce one helper:

```rust
fn hash_callable_reference_resolution(
    resolution: &CallableReferenceResolution,
    hasher: &mut impl Hasher,
)
```

Recommended shape:

```rust
fn hash_callable_reference_resolution(
    resolution: &CallableReferenceResolution,
    hasher: &mut impl Hasher,
) {
    match &resolution.kind {
        CallableReferenceResolutionKind::BoundFamily {
            family_type,
            spec,
            members,
        } => {
            0u8.hash(hasher);
            family_type.hash(hasher);
            spec.hash(hasher);
            members.len().hash(hasher);

            for member in members.iter() {
                hash_bound_behavioral_member(member, hasher);
            }
        }

        CallableReferenceResolutionKind::Associated(
            associated
        ) => {
            1u8.hash(hasher);
            hash_associated_resolution(
                associated,
                hasher,
            );
        }
    }
}
```

This is a key PATCH0 requirement:

> The `Associated` branch must reuse the same full associated-resolution helper as the top-level associated-resolution index.

Do not create a second partial nested hash.

---

# 6.10 `BoundFamily.spec` means selector semantics

Current:

```rust
pub enum BehavioralFamilySpec {
    Exact(Selector),
    Pattern(SelectorPattern),
}
```

already derives `Hash`.

Therefore:

```rust
spec.hash(hasher);
```

is preferred.

Do not convert it to strings.

The discriminator test must change only the selector/spec while keeping family type and members unchanged.

---

# 6.11 Complete bound-member hashing

The current code hashes only:

```text
operation
target
conditional_capability
```

A bound member's semantic product actually contains:

```text
operation
member_kind
target
callable_type
conditional
conditional_capability
trait_dispatch
```

PATCH0 should make `members` mean the whole published member semantics.

Recommended:

```rust
fn hash_bound_behavioral_member(
    member: &BoundBehavioralMember,
    hasher: &mut impl Hasher,
) {
    member.operation.hash(hasher);
    member.member_kind.hash(hasher);
    member.target.hash(hasher);
    member.callable_type.hash(hasher);
    member.conditional.hash(hasher);

    hash_conditional_selected_call_capability(
        member.conditional_capability.as_ref(),
        hasher,
    );

    hash_trait_dispatch_selection(
        member.trait_dispatch.as_ref(),
        hasher,
    );
}
```

The `conditional` selection derives `Hash`.

For `trait_dispatch`, do not use `Debug`.

---

# 6.12 Extract reusable trait-dispatch selection hashing

Current `hash_trait_dispatch_site` already defines the product dimensions for an evidenced `TraitDispatchSelection`.

Extract those exact dimensions into a reusable helper rather than inventing a second definition.

Conceptually:

```rust
fn hash_trait_dispatch_selection(
    selection: Option<&TraitDispatchSelection>,
    hasher: &mut impl Hasher,
) {
    match selection {
        None => 0u8.hash(hasher),

        Some(selection) => {
            1u8.hash(hasher);
            hash_evidenced_trait_dispatch_selection(
                selection,
                hasher,
            );
        }
    }
}
```

and:

```rust
fn hash_evidenced_trait_dispatch_selection(
    selection: &TraitDispatchSelection,
    hasher: &mut impl Hasher,
) {
    selection.source_impl.hash(hasher);
    selection.exact_target.hash(hasher);
    selection.exact_trait_ref.hash(hasher);
    selection.requirement.hash(hasher);
    selection.evidence_fingerprint.raw().hash(hasher);

    hash_requirement_selection(
        &selection.selection,
        hasher,
    );

    selection.requirement_targets.len().hash(hasher);
    for (requirement, target) in &selection.requirement_targets {
        requirement.hash(hasher);
        hash_requirement_selection(target, hasher);
    }
}
```

Then update the existing:

```rust
TraitDispatchSite::Evidenced(selection)
```

branch to call the extracted helper.

Critical preservation rule:

> Do not silently add or remove trait-dispatch fingerprint dimensions while extracting.

PATCH0's purpose here is reuse.

The existing product definition intentionally uses:

```text
source_impl
exact_target
exact_trait_ref
requirement
evidence_fingerprint
selection
requirement_targets
```

Preserve that meaning.

Do not suddenly hash diagnostic/provenance-heavy internals merely because `TraitDispatchSelection` contains them.

---

# 6.13 Callable-reference index helper

Recommended:

```rust
fn hash_callable_reference_resolutions(
    resolutions: &CallableReferenceResolutionIndex,
    hasher: &mut impl Hasher,
) {
    resolutions.len().hash(hasher);

    for (expression, resolution) in resolutions {
        expression.hash(hasher);
        hash_callable_reference_resolution(
            resolution,
            hasher,
        );
    }
}
```

Replace the current inlined match.

---

# 6.14 Preserve all existing fingerprint dimensions

This requirement is explicit.

Before editing, capture the current main function and ensure no existing call is accidentally dropped.

At minimum preserve current hashing of:

```text
analysis.callable
body_source_owner

expression:
    expression_id
    knowledge
    callable
    call specialization
    conditional dispatch
    conditional capability
    trait dispatch
    trait dispatch candidates
    contextual realization
    denotation
    status
    causal invalidity
    binding read safety
    binding read

bindings
binding origins

match resolutions
pattern site resolutions
iteration lanes
positional expansions
known invalid expansions
sequence literals

callable references

flow graph
entry flow
exit facts

internal incident shape
dependencies
constructor super initializer
callable analysis status
return validation
```

PATCH0 adds dimensions.

It does not subtract them.

---

# 6.15 Do not hash `dependency_fingerprint`

Preserve the current comment/invariant:

```rust
// dependency_fingerprint is assigned from this fingerprint
// after computation. Hashing it would make the product
// definition recursive.
```

Do not touch this relationship.

---

# 7. Tests

# 7.1 Test location

Use:

```text
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

Reuse:

```rust
fn callable_analysis() -> CallableAnalysis
```

Add focused fixture helpers only when they materially simplify the discriminators.

Avoid a large generic builder framework.

---

# 7.2 Test design rule

Each required test must:

```text
start from semantically identical owners
change exactly one semantic choice
assert fingerprint !=
```

Do not write tests where several fields change at once unless the data type makes one change impossible without a coupled invariant.

When coupling is required, document it in the test name/comment.

---

# 7.3 CV-01 — binding write discriminator

Required shape:

```rust
#[test]
fn callable_body_product_includes_binding_write_target() {
    let expr_id = ...;
    let binding_a = BindingId(...);
    let binding_b = BindingId(...);

    let mut left = callable_analysis();
    left.expressions.insert(
        expr_id,
        ExpressionAnalysis::ready(...)
    );

    left.expressions
        .get_mut(&expr_id)
        .unwrap()
        .binding_write = Some(binding_a);

    let mut right = left.clone();
    right.expressions
        .get_mut(&expr_id)
        .unwrap()
        .binding_write = Some(binding_b);

    assert_ne!(
        callable_body_product_fingerprint(&left),
        callable_body_product_fingerprint(&right)
    );
}
```

Do not also change binding state.

This test specifically catches the old omission.

---

# 7.4 CV-02 — family application target discriminator

Construct one `FamilyApplicationResolution::Static`.

Hold constant:

```text
expression owner
family_type
kind
operation
callable_type
result_type
```

Change only:

```text
target A
target B
```

Assert fingerprint changes.

If `None -> Some(target)` is simpler and semantically valid, that is acceptable, but two concrete target identities are stronger.

---

# 7.5 CV-03 — family application selection/spec discriminator

The implementer request says “family target/spec.”

Use at least one second family application discriminator.

Preferred options:

### Option A

Change:

```text
FamilyApplicationKind::Associated
vs
FamilyApplicationKind::Behavioral
```

while keeping selection identical.

### Option B

Change static `operation` shape/selector slot pattern while holding target/types constant.

### Option C

Change:

```text
Static selection
vs
Dynamic selection
```

with otherwise equivalent target/type meaning where possible.

The strongest narrow suite contains:

```text
target changes => fingerprint changes
kind/selection changes => fingerprint changes
```

Do not rely on only one family test.

---

# 7.6 CV-04 — associated resolution variant discriminator

Construct two `AssociatedResolution`s with identical common ownership fields:

```text
owner_form
lookup_owner
family
```

Change the variant/selection.

Example:

```text
ExactValue
vs
ExactCallable
```

using valid minimal identities.

Assert fingerprint changes.

This test proves the helper hashes variant identity, not just common ownership.

---

# 7.7 CV-05 — associated resolution payload discriminator

Use one variant and change only one selected semantic field.

Recommended:

```text
ExactCallable.target A
vs
ExactCallable.target B
```

or:

```text
StaticInvoke.argument_mapping [0,1]
vs [1,0]
```

or:

```text
Family.family_type A
vs B
```

Assert fingerprint changes.

This catches a future “variant tag only” regression.

---

# 7.8 CV-06 — associated resolution owner/family discriminator

Recommended additional narrow test:

Hold `kind` payload constant.

Change only one of:

```text
owner_form
lookup_owner
family
```

Assert change.

At minimum, test `family` if construction is easy.

Reason:

The helper contract explicitly includes all common owner/family fields.

---

# 7.9 CV-07 — bound family type discriminator

Construct:

```rust
CallableReferenceResolutionKind::BoundFamily {
    family_type,
    spec,
    members,
}
```

Clone the analysis.

Change only:

```text
family_type = TypeId(A)
family_type = TypeId(B)
```

Assert product fingerprint differs.

This directly reproduces S2-06.

---

# 7.10 CV-08 — bound family selector/spec discriminator

Hold:

```text
family_type
members
```

constant.

Change:

```text
BehavioralFamilySpec::Exact(selector A)
vs
BehavioralFamilySpec::Exact(selector B)
```

or Exact vs Pattern when fixture construction is simpler.

Assert product fingerprint differs.

Do not alter member operations to encode the selector difference indirectly.

The whole purpose is to prove `spec` itself is hashed.

---

# 7.11 CV-09 — bound member semantic discriminator

Because PATCH0 makes `members` complete, add one low-cost test changing an omitted member fact.

Preferred:

```text
member.callable_type A vs B
```

Alternative:

```text
member.member_kind Value vs Callable
```

when semantically constructible.

This test is not a substitute for CV-07/CV-08.

It proves `members` means complete member semantics rather than current partial fields.

---

# 7.12 CV-10 — nested associated callable-reference discriminator

Construct:

```rust
CallableReferenceResolutionKind::Associated(
    Box::new(AssociatedResolution { ... })
)
```

Create `left`.

Clone to `right`.

Change only a nested associated semantic choice:

```text
nested ExactCallable.target A
vs
nested ExactCallable.target B
```

or another payload field.

Assert callable body product fingerprint differs.

This must call the production nested branch.

It directly catches the current:

```rust
Associated(_) => 1u8.hash(...)
```

bug.

---

# 7.13 CV-11 — shared helper parity

Recommended unit-level or integration-level discriminator:

Put the same `AssociatedResolution`:

```text
once in analysis.associated_resolutions
once nested under callable_reference_resolutions
```

Mutate the same semantic field in each scenario.

Both scenarios must cause fingerprint invalidation.

The test does not require the numeric deltas/hashes to match.

It proves both paths are semantically sensitive to the same payload.

---

# 7.14 CV-12 — existing invariance tests remain green

At minimum the existing tests proving these properties must still pass:

```text
diagnostic cause renumbering ignored
explanation presentation ignored
diagnostic presentation ignored
source ranges ignored by product fingerprint
flow source ranges ignored
```

PATCH0 must not turn product fingerprints into source-position hashes.

---

# 8. Test fixture guidance

The exact constructors depend on current public APIs.

Prefer repository-native identities.

Typical fixture imports may include:

```rust
use phalcom_semantic::checker::associated::{
    AssociatedResolution,
    AssociatedResolutionKind,
    BehavioralFamilySpec,
    BoundBehavioralMember,
    CallableReferenceResolution,
    CallableReferenceResolutionKind,
    FamilyApplicationKind,
    FamilyApplicationResolution,
    FamilyApplicationSelection,
    SpecializedAssociatedMember,
};

use phalcom_semantic::types::family::{
    FamilyMemberTypeKind,
    FamilyOperationShape,
};
```

Use existing helpers:

```text
declaration(...)
semantic_signature()
callable_analysis()
```

Add small local helpers such as:

```rust
fn associated_member(
    owner: &DeclarationId,
    index: u32,
) -> AssociatedMemberId
```

only if the live identity constructor requires verbose setup.

Do not make tests depend on parsing/semantic analysis when direct native product construction suffices.

These are product fingerprint tests, not parser tests.

---

# 9. Focused verification commands

Use the live toolchain/repository conventions.

## 9.1 PATCH0 primary test filter

Recommended first command:

```bash
cargo test -p phalcom-semantic \
  --test semantic \
  callable_body_product \
  -- --nocapture
```

Requirement:

```text
selected test count must be nonzero
all selected tests pass
```

If the repository's exact filter name differs after adding tests, use the narrowest equivalent and record the command.

---

# 9.2 Existing callable-analysis baseline

Re-run the exact PATCH1 T0 command that previously selected:

```text
19 tests
19 passed
```

Do not substitute a broader suite and call it equivalent.

Record:

```text
before: 19/19
after: expected all selected pass
```

The selected count may increase only if PATCH0 test naming causes that exact filter to intentionally select new tests; if so, record the count change rather than claiming 19.

---

# 9.3 Existing artifact baseline

Run:

```bash
cargo test -p phalcom-compiler-artifact \
  --test semantic_world_callable_analysis_test
```

Expected inherited baseline from T0:

```text
9 selected
4 passed
5 failed
```

PATCH0 does not require this suite to become green.

Acceptance rule:

```text
the same known failures remain classified as inherited
no new failure appears
no previous passing artifact test regresses
```

If failure identities/counts change unexpectedly, STOP and investigate.

Do not rewrite artifact tests to preserve a count artificially.

---

# 9.4 Optional direct crate compile

If needed after helper extraction:

```bash
cargo check -p phalcom-semantic
```

Do not run workspace-wide test/check as PATCH0 certification unless a narrow compile failure gives evidence that it is necessary.

---

# 10. Baseline handling

Record T0 exactly in PATCH0 lifecycle notes.

Example:

```markdown
## PATCH0 baseline

Revision:
`21a54705c33dae95ee9c77d86789b4911635e6b0`

Inherited semantic baseline:
- callable_analysis filter: 19 selected, 19 passed

Inherited artifact baseline:
- semantic_world_callable_analysis_test: 9 selected
- 4 passed
- 5 failed
- failures are pre-existing allocation-boundary /
  conditional-dispatch validation failures

PATCH0 policy:
- semantic fingerprint repair only
- artifact failures are not PATCH0 scope
- allocation fail-closed guards remain
```

Do not write:

```text
artifact suite failing because PATCH0 incomplete
```

That would misclassify existing failures.

## PATCH0 execution result — 2026-10-15

Starting revision: `21a54705c33dae95ee9c77d86789b4911635e6b0`.
Implementation commit: `8284893769d121dc2fe33ed0c866a402ff48bd35`.

The callable fingerprint module's focused filter passed 23/23, including the
new binding-write, associated-resolution, static/dynamic family-application,
bound-family, and nested-associated discriminators plus existing
product-invariance tests. The
exact callable-analysis library filter passed 19/19. The standard semantic
integration target did not reach test selection because compilation is blocked
by an inherited stale `local_len()` call at
`tests/semantic/integration/canonical_image.rs:203`; the same fingerprint
module was compiled and tested through a temporary target/harness with the
normal incremental support module, and those temporary files were removed.
The artifact baseline remained 4/9 with the same five inherited failures and
no new failure. `rustfmt --edition 2024 --check` for the two scoped Rust files
and `git diff --check` passed.

PATCH0 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED. PATCH1's
`PATCH0_callable_fingerprint` prerequisite is SATISFIED; PATCH1 itself remains
otherwise unstarted. P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED. See the
PATCH0 walkthrough and handoff for exact commands and failure identities.

---

# 11. Task sequence

# T0 — Takeover and exact baseline

## Read

Required files from §4.

## Verify

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

Confirm HEAD or record divergence.

If branch moved:

- inspect intervening commits;
- re-check the function;
- reuse already-correct concurrent work;
- do not overwrite it.

## Baseline

Reproduce/record:

```text
semantic callable_analysis: 19/19
artifact callable analysis: 4/9 with 5 known failures
```

If exact baseline cannot be reproduced because repository state changed, record actual result and compare failure identities.

## Lifecycle

Mark:

```text
PATCH0 IN_PROGRESS
PATCH1 BLOCKED_ON_PATCH0_FINGERPRINT
```

No production changes yet.

---

# T1 — Add `binding_write`

## Modify

```text
phalcom-semantic/src/db/fingerprint.rs
```

Add:

```rust
expression.binding_write.hash(&mut hasher);
```

adjacent to binding read fields.

## Test

Add CV-01 immediately.

Run only the binding/fingerprint-focused test.

## Acceptance

Changing only assignment target changes product fingerprint.

---

# T2 — Replace associated special case with exhaustive helper

## Modify

Delete the current `DataExpansionField`-only count and loop.

Add:

```text
hash_associated_resolutions
hash_associated_resolution
hash_specialized_associated_member
```

with exhaustive variant matching.

## Test

Add CV-04, CV-05, CV-06.

Run the narrow `callable_body_product` filter.

## Acceptance

Every live `AssociatedResolutionKind` is represented by an explicit match arm.

No wildcard.

No special-case-only top-level path remains.

---

# T3 — Add family application hashing

## Modify

Add:

```text
hash_family_application_resolutions
hash_family_application_resolution
hash_family_application_selection
```

Invoke from `callable_body_product_fingerprint`.

## Test

Add CV-02 and CV-03.

Prefer one static and one dynamic discriminator if fixture cost remains small.

## Acceptance

A family target/selection change changes product fingerprint.

`analysis.family_applications` is visibly referenced by the main product function.

---

# T4 — Make callable-reference hashing exhaustive

## Modify

Replace current inline callable-reference branch with:

```text
hash_callable_reference_resolutions
hash_callable_reference_resolution
hash_bound_behavioral_member
```

BoundFamily must hash:

```text
family_type
spec
member count
complete member semantics
```

Associated must call:

```rust
hash_associated_resolution(...)
```

## Trait helper

Extract/reuse existing evidenced trait-dispatch dimensions if needed to hash `BoundBehavioralMember.trait_dispatch`.

Do not alter their semantic definition.

## Tests

Add:

```text
CV-07 family type
CV-08 selector/spec
CV-09 member semantic field
CV-10 nested Associated
CV-11 shared sensitivity
```

## Acceptance

No branch hashes only a tag when payload affects executable lowering.

---

# T5 — Preservation review

Diff `callable_body_product_fingerprint` against baseline.

Explicitly verify every old dimension remains.

Run existing invariance tests.

Review helper variant coverage against the live enum definitions.

Search:

```bash
rg -n \
  "AssociatedResolutionKind|CallableReferenceResolutionKind|FamilyApplicationSelection|FamilyApplicationKind" \
  phalcom-semantic/src/db/fingerprint.rs \
  phalcom-semantic/src/checker/associated.rs
```

Acceptance:

```text
all live variants covered
old dimensions preserved
presentation/local-number invariants preserved
```

---

# T6 — Focused certification and PATCH1 unblock

Run:

1. narrow callable body product tests;
2. exact callable-analysis semantic baseline filter;
3. artifact callable-analysis baseline.

Record commands and counts.

PATCH0 may be marked:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

only when:

```text
all new fingerprint discriminators pass
all previously passing focused semantic tests remain green
no unexpected artifact regression appears
```

Then change PATCH1 dependency status from:

```text
BLOCKED_ON_PATCH0_FINGERPRINT
```

to:

```text
PATCH0_PREREQUISITE_SATISFIED
```

Do **not** mark PATCH1 complete or otherwise advance its implementation status automatically.

---

# 12. Acceptance gates

# G0 — Scope gate

Diff is limited to:

```text
fingerprint implementation
incremental fingerprint tests
lifecycle documentation
```

Any artifact/import/allocation production change requires justification and normally fails this gate.

---

# G1 — Binding-write gate

```text
same analysis
only binding_write differs
fingerprints differ
```

---

# G2 — Family-application gate

```text
same analysis
only family application target differs
fingerprints differ
```

and at least one of:

```text
kind differs
selection kind differs
operation/spec differs
dynamic candidate differs
```

also changes fingerprint.

---

# G3 — Associated-resolution gate

At least:

```text
variant change changes fingerprint
payload change changes fingerprint
```

Every live variant is exhaustively implemented.

---

# G4 — Bound-family gate

Independently prove:

```text
family_type change
selector/spec change
member semantic change
```

each invalidates the callable product.

---

# G5 — Nested associated gate

```text
CallableReferenceResolutionKind::Associated(...)
nested semantic selection changes
callable body product changes
```

---

# G6 — Preservation gate

Existing product invariance tests remain green.

No intentional old dimension disappears.

---

# G7 — Baseline gate

Semantic:

```text
existing callable_analysis baseline remains green
```

Artifact:

```text
no new regression beyond inherited 5 failures
```

The artifact allocation/conditional failures remain recorded as baseline, not PATCH0 defects.

---

# G8 — PATCH1 unblock gate

PATCH1 is unblocked only after G1–G7 pass.

No earlier.

---

# 13. Coverage matrix

| ID | Semantic dimension | Old status | PATCH0 proof |
|---|---|---:|---|
| FP-01 | binding read safety | hashed | preservation |
| FP-02 | binding read target | hashed | preservation |
| FP-03 | binding write target | **missing** | CV-01 |
| FP-04 | associated owner form | partial/special-case | CV-06 |
| FP-05 | associated lookup owner | partial/special-case | CV-06 |
| FP-06 | associated family owner | **missing for most** | CV-06 |
| FP-07 | ExactValue | **missing** | exhaustive helper |
| FP-08 | DataExpansionField | hashed | preservation |
| FP-09 | ExactCallable | **missing** | CV-04/05 |
| FP-10 | Family | **missing** | helper/test |
| FP-11 | StaticInvoke | **missing** | helper/test coverage |
| FP-12 | StaticDomain | **missing** | exhaustive helper |
| FP-13 | StaticTerm | **missing** | exhaustive helper |
| FP-14 | DynamicInvoke | **missing** | exhaustive helper |
| FP-15 | family application index | **entirely missing** | CV-02/03 |
| FP-16 | family application family type | **missing** | CV-02/03 |
| FP-17 | family application kind | **missing** | CV-03 |
| FP-18 | static selection op/target/types | **missing** | CV-02 |
| FP-19 | dynamic candidates/result | **missing** | helper/test |
| FP-20 | BoundFamily family type | **missing** | CV-07 |
| FP-21 | BoundFamily selector spec | **missing** | CV-08 |
| FP-22 | BoundFamily member kind | **missing** | CV-09/helper |
| FP-23 | BoundFamily callable type | **missing** | CV-09 |
| FP-24 | BoundFamily conditional selection | **missing** | helper |
| FP-25 | BoundFamily conditional capability | hashed | preservation |
| FP-26 | BoundFamily trait dispatch | **missing** | helper |
| FP-27 | callable ref nested Associated | **tag only** | CV-10 |
| FP-28 | flow graph/summary/exits | hashed | preservation |
| FP-29 | status/return validation | hashed | preservation |
| FP-30 | source range presentation | intentionally omitted | preservation |

---

# 14. Invariants

### INV-01

A change to `binding_write` changes the product fingerprint.

### INV-02

All `AssociatedResolutionKind` variants participate.

### INV-03

Every semantic field of an associated resolution variant participates.

### INV-04

Common associated ownership fields participate.

### INV-05

Nested associated callable references reuse the same associated helper.

### INV-06

The family application index participates.

### INV-07

Family application kind participates.

### INV-08

Static family selection target/type/operation participates.

### INV-09

Dynamic family candidates and result type participate.

### INV-10

BoundFamily `family_type` participates.

### INV-11

BoundFamily `spec` participates.

### INV-12

Complete bound member semantics participate.

### INV-13

No `Debug` formatting defines semantic hash identity.

### INV-14

No wildcard match hides a future associated/reference/application variant.

### INV-15

Existing intentional source-range insensitivity remains.

### INV-16

Diagnostic/explanation presentation insensitivity remains.

### INV-17

`dependency_fingerprint` remains excluded to prevent recursion.

### INV-18

PATCH0 does not alter artifact serialization.

### INV-19

PATCH0 does not alter allocation policy.

### INV-20

Allocation fail-closed guards remain in place.

### INV-21

PATCH1 remains blocked until focused PATCH0 tests pass.

### INV-22

The five inherited artifact failures remain baseline unless independently changed by upstream code.

---

# 15. Fixed decisions

These are not open to implementer reinterpretation.

1. PATCH0 is separate from PATCH1.
2. Production repair is in `callable_body_product_fingerprint` and private helpers.
3. `binding_write` is hashed.
4. `family_applications` is hashed completely.
5. one exhaustive `hash_associated_resolution` helper exists;
6. nested associated callable references reuse that helper;
7. BoundFamily hashes family type and selector/spec;
8. BoundFamily members are hashed as semantic products, not only operation/target;
9. no artifact schema change;
10. no allocation guard removal;
11. no broad suite requirement;
12. PATCH1's fingerprint-dependent gates remain blocked until PATCH0 passes.

---

# 16. Mechanically flexible decisions

The implementer may choose:

- exact helper names;
- helper ordering in `fingerprint.rs`;
- exact numeric variant tag values;
- exact fixture helper names;
- exact concrete associated member identity values;
- whether CV-04/CV-05 use ExactCallable, StaticInvoke, or another valid payload;
- whether trait-dispatch reuse uses `Option` helper plus inner helper or one helper taking `&TraitDispatchSelection`.

The semantic dimensions are not flexible.

---

# 17. Verify-first items

Before editing:

1. confirm branch HEAD;
2. confirm no concurrent PATCH0 change already landed;
3. confirm current `AssociatedResolutionKind` variant list;
4. confirm current `BoundBehavioralMember` fields;
5. confirm current `FamilyApplicationSelection` fields;
6. confirm PATCH1 baseline command that produced 19/19;
7. confirm the identities of the five inherited artifact failures before comparing post-PATCH0 results.

If any relevant enum gained a new variant after this plan was written, add it exhaustively and update the plan/walkthrough.

---

# 18. STOP / CONSULT conditions

Stop if:

1. branch moved and already contains overlapping fingerprint repair;
2. a relevant semantic field's meaning is unclear enough that hashing it may intentionally violate product semantics;
3. exhaustive associated hashing causes an existing incremental test to prove a deliberate omission contract;
4. BoundFamily trait dispatch cannot reuse existing semantic selection hashing without redefining product meaning;
5. a test requires artifact schema/import changes to construct a fingerprint product;
6. any PATCH0 code change touches AllocationBudget or artifact guards;
7. the five baseline artifact failures change identity unexpectedly after only semantic fingerprint edits;
8. existing range/presentation invariance tests fail because the new helper accidentally hashes presentation fields;
9. implementation starts solving cross-process type-ID fingerprint portability;
10. implementation starts solving PATCH1 DTO/root/import work.

Consultation packet:

```text
revision
minimal diff
failing discriminator
expected invariant
actual fingerprint behavior
relevant native type
two plausible semantic interpretations
decision required
```

---

# 19. Performance and determinism constraints

PATCH0 must remain allocation-light.

Do not add temporary vectors where native maps/slices already have deterministic iteration order.

Good:

```rust
for (expression, resolution) in resolutions {
    ...
}
```

Bad:

```rust
let mut rows = resolutions.iter().collect::<Vec<_>>();
rows.sort(...);
```

when the owner is already a `BTreeMap`.

Do not serialize, clone, or format large products merely to hash them.

No:

```rust
format!("{resolution:?}").hash(...)
```

Use field-wise hashing.

The fingerprint is a hot incremental semantic operation.

Its complexity should remain:

```text
O(total semantic product size)
```

with one pass through each participating family.

---

# 20. Product compatibility note

PATCH0 intentionally changes the numeric `ProductFingerprint` of callables containing newly hashed semantic families.

That is correct.

Do not attempt to preserve old numeric values for affected callables.

The compatibility goal is semantic:

```text
same published semantics
    => same fingerprint under the repaired algorithm

different published executable semantics
    => different fingerprint for the required discriminators
```

Unaffected callable products may coincidentally also change if helper ordering changes unnecessarily.

Avoid gratuitous reordering so the diff remains easy to review.

---

# 21. PATCH1 dependency contract after completion

PATCH0 delivers PATCH1 this guarantee:

```text
callable_body_product_fingerprint
covers the executable associated/reference/family
dimensions PATCH1 is about to transport.
```

PATCH1 may then use the repaired fingerprint to:

```text
reconstruct owner
validate closure
recompute product fingerprint
publish imported callable authority
```

PATCH0 does **not** guarantee:

```text
cross-process raw numeric fingerprint equality
portable native TypeId identity
allocation-exact owner reconstruction
wire completeness
query-image correctness
```

Those remain later PATCH1/PATCH3/PATCH4 concerns according to their plans.

---

# 22. Lifecycle updates

At PATCH0 start, update the durable P7/PATCH ledger with:

```yaml
PATCH0:
  status: IN_PROGRESS
  completion: NOT_STARTED
  verification: UNVERIFIED

PATCH1:
  dependency:
    PATCH0_callable_fingerprint: BLOCKED
```

At successful implementation before tests:

```yaml
PATCH0:
  status: IN_PROGRESS
  completion: IMPLEMENTED
  verification: UNVERIFIED
```

After focused gates:

```yaml
PATCH0:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

PATCH1:
  dependency:
    PATCH0_callable_fingerprint: SATISFIED
```

Do not mutate overall P7 to complete.

P7 remains partial until all P7 acceptance gates pass.

---

# 23. Walkthrough requirement

Create:

```text
PERF003.C2.P7.PATCH0-walkthrough.md
```

Required sections:

1. starting revision;
2. final revision;
3. exact production diff;
4. old omissions;
5. final helper graph;
6. exhaustive associated variant table;
7. family application coverage;
8. BoundFamily coverage;
9. trait-dispatch helper reuse;
10. tests added;
11. exact commands;
12. selected/pass/fail counts;
13. inherited 19/19 semantic baseline;
14. inherited artifact 4/9 + five failure identities;
15. confirmation guards unchanged;
16. PATCH1 unblock statement;
17. residual risks deferred to PATCH1.

---

# 24. Handoff requirement

Create:

```text
PERF003.C2.P7.PATCH0-handoff.md
```

Must state:

```text
HEAD
commit(s)
clean/dirty state

callable_body_product_fingerprint repaired:
  binding_write
  family_applications
  exhaustive associated resolution
  complete callable references

focused tests:
  command
  count
  result

artifact baseline:
  4 passed
  5 inherited failed
  identities unchanged

allocation guards:
  unchanged

PATCH1:
  fingerprint prerequisite satisfied
  all other PATCH1 work still outstanding
```

Include:

> Do not revert the exhaustive helper back to DataExpansionField-only hashing during PATCH1 schema work. Imported associated callable references must recompute through the same native semantic fingerprint path.

---

# 25. Suggested code organization

Recommended local order near the current callable product function:

```text
callable_body_product_fingerprint
hash_call_specialization
hash_trait_dispatch_site
hash_evidenced_trait_dispatch_selection   # extracted if used
hash_contextual_realization
hash_requirement_selection

hash_associated_resolutions
hash_associated_resolution
hash_specialized_associated_member

hash_callable_reference_resolutions
hash_callable_reference_resolution
hash_bound_behavioral_member

hash_family_application_resolutions
hash_family_application_resolution
hash_family_application_selection

existing match/pattern/flow helpers...
```

Exact ordering can follow local style.

The important property is that semantic families are discoverable and auditable.

---

# 26. Recommended implementation diff shape

A healthy PATCH0 diff should look approximately like:

```text
phalcom-semantic/src/db/fingerprint.rs
    + binding_write line
    - old DataExpansionField special-case block
    + exhaustive associated helpers
    - old inline callable-reference partial match
    + exhaustive callable-reference helpers
    + family application helpers
    ~ small trait dispatch helper extraction

phalcom-semantic/tests/semantic/incremental/fingerprints.rs
    + direct fixture imports
    + 8–12 narrow discriminating tests/helpers

docs/implementation/.../P7/
    + PATCH0 lifecycle/walkthrough/handoff records
    ~ checkpoint dependency state
```

A production diff sprawling across artifact crates is evidence of scope drift.

---

# 27. Anti-shortcut list

Forbidden:

```text
hash only enum discriminants
hash Debug output
hash only DataExpansionField
hash only family application count
hash only callable reference member count
hash BoundFamily members but omit family_type/spec
hash Associated callable reference only as tag
use nested helper different from top-level associated helper
ignore dynamic family candidates
ignore StaticInvoke argument mapping
ignore associated family owner
drop existing old hash dimensions while refactoring
include source ranges accidentally
include dependency_fingerprint recursively
fix artifact allocation failures
remove UnaccountedBoundary
merge PATCH0 and PATCH1 status
```

---

# 28. Minimal required test set

At absolute minimum PATCH0 is incomplete without all of:

```text
[ ] binding_write A/B
[ ] family application target A/B
[ ] family application kind/selection/spec A/B
[ ] associated resolution variant A/B
[ ] associated resolution selected payload A/B
[ ] BoundFamily family_type A/B
[ ] BoundFamily selector/spec A/B
[ ] BoundFamily member semantic A/B
[ ] nested Associated callable-reference payload A/B
[ ] existing source-range invariance
[ ] existing diagnostic/explanation invariance
```

The first nine are positive invalidation tests.

The last two categories prevent accidental broadening of product semantics.

---

# 29. Certification checklist

PATCH0 may close only if:

```text
[ ] HEAD provenance recorded
[ ] baseline 19/19 semantic evidence preserved/updated honestly
[ ] baseline artifact 4/9 and five inherited failures recorded

[ ] binding_write hashed

[ ] associated index hashes all entries
[ ] AssociatedResolution common owner fields hashed
[ ] ExactValue hashed
[ ] DataExpansionField hashed
[ ] ExactCallable hashed
[ ] Family hashed
[ ] StaticInvoke hashed
[ ] StaticDomain hashed
[ ] StaticTerm hashed
[ ] DynamicInvoke hashed
[ ] no wildcard associated arm

[ ] family_applications index hashed
[ ] family type hashed
[ ] family kind hashed
[ ] static operation/target/callable/result hashed
[ ] dynamic candidates/result hashed
[ ] no wildcard family selection arm

[ ] callable reference index hashed
[ ] BoundFamily family_type hashed
[ ] BoundFamily spec hashed
[ ] complete member semantics hashed
[ ] Associated branch hashes nested payload recursively
[ ] trait dispatch helper preserves prior semantic dimensions

[ ] all required discriminators pass
[ ] existing invariance tests pass
[ ] no old fingerprint dimension removed
[ ] dependency_fingerprint remains excluded

[ ] artifact schema unchanged
[ ] allocation code unchanged
[ ] fail-closed allocation guards unchanged
[ ] no unexpected artifact regression

[ ] PATCH0 walkthrough created
[ ] PATCH0 handoff created
[ ] PATCH1 dependency marked satisfied only after tests
```

---

# 30. Expected end state

At PATCH0 completion:

```text
CallableAnalysis semantic product
    │
    ├── expression read/write identity
    │
    ├── associated resolution semantics
    │
    ├── callable reference semantics
    │       ├── bound family type
    │       ├── selector/spec
    │       ├── full members
    │       └── nested associated resolution
    │
    ├── family application semantics
    │
    └── all previously fingerprinted semantics
            preserved
```

Then:

```text
PATCH0 COMPLETE
    ↓
PATCH1 fingerprint prerequisite SATISFIED
    ↓
PATCH1 may safely build portable CallableAnalysis
transport against a product identity that actually
sees the executable semantic decisions it transports.
```

P7 remains:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

until its later semantic, allocation, query, container, loaded-image, package, and performance acceptance gates close.

---

# Appendix A — Exact live defect map

| Native fact | Live owner | Current fingerprint | Required |
|---|---|---|---|
| assignment target | `ExpressionAnalysis.binding_write` | absent | direct hash |
| associated ExactValue | `AssociatedResolutionKind` | absent | exhaustive helper |
| DataExpansionField | same | present | preserve via helper |
| ExactCallable | same | absent | exhaustive helper |
| Family | same | absent | exhaustive helper |
| StaticInvoke | same | absent | exhaustive helper |
| StaticDomain | same | absent | exhaustive helper |
| StaticTerm | same | absent | exhaustive helper |
| DynamicInvoke | same | absent | exhaustive helper |
| family applications | `CallableAnalysis.family_applications` | absent | full index |
| BoundFamily type | callable reference | absent | hash |
| BoundFamily spec | callable reference | absent | hash |
| member operation | BoundBehavioralMember | present | preserve |
| member target | BoundBehavioralMember | present | preserve |
| member kind | BoundBehavioralMember | absent | hash |
| member callable type | BoundBehavioralMember | absent | hash |
| member conditional | BoundBehavioralMember | absent | hash |
| member capability | BoundBehavioralMember | present | preserve |
| member trait dispatch | BoundBehavioralMember | absent | existing semantic helper |
| nested Associated | callable reference | tag only | recursive full helper |

---

# Appendix B — Exhaustive associated pseudo-code

```rust
fn hash_associated_resolution(
    resolution: &AssociatedResolution,
    hasher: &mut impl Hasher,
) {
    resolution.owner_form.hash(hasher);
    resolution.lookup_owner.hash(hasher);
    resolution.family.hash(hasher);

    match &resolution.kind {
        ExactValue {
            member,
            value_type,
        } => {
            tag(0, hasher);
            member.hash(hasher);
            value_type.hash(hasher);
        }

        DataExpansionField {
            logical_index,
            value_type,
        } => {
            tag(1, hasher);
            logical_index.hash(hasher);
            value_type.hash(hasher);
        }

        ExactCallable {
            member,
            target,
            callable_type,
        } => {
            tag(2, hasher);
            member.hash(hasher);
            target.hash(hasher);
            callable_type.hash(hasher);
        }

        Family {
            family_type,
            members,
        } => {
            tag(3, hasher);
            family_type.hash(hasher);
            members.len().hash(hasher);
            for member in members {
                hash_specialized_associated_member(
                    member,
                    hasher,
                );
            }
        }

        StaticInvoke {
            member,
            target,
            result_type,
            argument_mapping,
        } => {
            tag(4, hasher);
            member.hash(hasher);
            target.hash(hasher);
            result_type.hash(hasher);
            argument_mapping.hash(hasher);
        }

        StaticDomain {
            member,
            domain,
        } => {
            tag(5, hasher);
            member.hash(hasher);
            domain.hash(hasher);
        }

        StaticTerm {
            domain,
            case,
            term,
        } => {
            tag(6, hasher);
            domain.hash(hasher);
            case.hash(hasher);
            term.hash(hasher);
        }

        DynamicInvoke {
            candidates,
            result_type,
        } => {
            tag(7, hasher);
            candidates.len().hash(hasher);
            for candidate in candidates {
                hash_specialized_associated_member(
                    candidate,
                    hasher,
                );
            }
            result_type.hash(hasher);
        }
    }
}
```

Use actual qualified enum paths/style preferred by the file.

---

# Appendix C — Family application pseudo-code

```rust
fn hash_family_application_resolution(
    resolution: &FamilyApplicationResolution,
    hasher: &mut impl Hasher,
) {
    resolution.family_type.hash(hasher);

    match resolution.kind {
        FamilyApplicationKind::Associated => {
            0u8.hash(hasher)
        }

        FamilyApplicationKind::Behavioral => {
            1u8.hash(hasher)
        }
    }

    match &resolution.selection {
        FamilyApplicationSelection::Static {
            operation,
            target,
            callable_type,
            result_type,
        } => {
            0u8.hash(hasher);
            operation.hash(hasher);
            target.hash(hasher);
            callable_type.hash(hasher);
            result_type.hash(hasher);
        }

        FamilyApplicationSelection::Dynamic {
            candidates,
            result_type,
        } => {
            1u8.hash(hasher);
            candidates.len().hash(hasher);

            for candidate in candidates {
                candidate.operation.hash(hasher);
                candidate.target.hash(hasher);
                candidate.callable_type.hash(hasher);
            }

            result_type.hash(hasher);
        }
    }
}
```

---

# Appendix D — Callable reference pseudo-code

```rust
fn hash_callable_reference_resolution(
    resolution: &CallableReferenceResolution,
    hasher: &mut impl Hasher,
) {
    match &resolution.kind {
        CallableReferenceResolutionKind::BoundFamily {
            family_type,
            spec,
            members,
        } => {
            0u8.hash(hasher);

            family_type.hash(hasher);
            spec.hash(hasher);

            members.len().hash(hasher);
            for member in members {
                member.operation.hash(hasher);
                member.member_kind.hash(hasher);
                member.target.hash(hasher);
                member.callable_type.hash(hasher);
                member.conditional.hash(hasher);

                hash_conditional_selected_call_capability(
                    member.conditional_capability.as_ref(),
                    hasher,
                );

                hash_trait_dispatch_selection(
                    member.trait_dispatch.as_ref(),
                    hasher,
                );
            }
        }

        CallableReferenceResolutionKind::Associated(
            associated
        ) => {
            1u8.hash(hasher);
            hash_associated_resolution(
                associated,
                hasher,
            );
        }
    }
}
```

---

# Appendix E — Test naming recommendations

Names should make omissions auditable:

```text
callable_body_product_includes_binding_write_target

callable_body_product_includes_family_application_target
callable_body_product_includes_family_application_selection

callable_body_product_includes_associated_resolution_variant
callable_body_product_includes_associated_resolution_selection
callable_body_product_includes_associated_family_owner

callable_body_product_includes_bound_family_type
callable_body_product_includes_bound_family_spec
callable_body_product_includes_bound_family_member_semantics

callable_body_product_includes_nested_associated_callable_reference
```

Avoid generic names like:

```text
fingerprint_changes
fingerprint_more_fields
artifact_fix
```

---

# Appendix F — Review questions

Before commit, reviewer should answer yes to all:

1. If assignment target changes, can old fingerprint survive? **No.**
2. Is there any `AssociatedResolutionKind` variant not in the helper? **No.**
3. Does the nested callable-reference associated branch call the same helper? **Yes.**
4. Is `family_applications` referenced from the product function? **Yes.**
5. Can BoundFamily family type change invisibly? **No.**
6. Can BoundFamily selector/spec change invisibly? **No.**
7. Are bound member type/conditional/trait semantics represented? **Yes.**
8. Are existing product invariance tests still green? **Yes.**
9. Did PATCH0 touch artifact DTO/import code? **No.**
10. Did PATCH0 remove allocation guards? **No.**
11. Are the five artifact failures still recorded as inherited? **Yes.**
12. Is PATCH1 marked unblocked only after focused tests? **Yes.**

---

# Appendix G — PATCH0/PATCH1 separation statement

PATCH0 answers:

```text
"What semantic facts define one callable analysis product?"
```

PATCH1 answers:

```text
"How do we serialize, validate, allocate, relocate,
reconstruct, and publish that complete callable owner?"
```

Those are different patches.

Keeping them separate is deliberate:

- fingerprint correctness can be reviewed independently;
- PATCH1 cannot hide lossy transport behind a lossy fingerprint;
- baseline allocation failures remain attributable;
- artifact guards remain fail-closed while owner transport is incomplete;
- regressions have a much smaller search surface.

Do not collapse the two patches for convenience.
