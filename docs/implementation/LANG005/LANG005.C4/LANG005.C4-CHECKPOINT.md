---
id: LANG005.C4
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
requires:
  - LANG005.C3
entry_requirements:
  - C1_COMPLETE
  - C2_COMPLETE
  - C2_APPLICABILITY_PROOF_STATE_CLOSED
  - C3_COMPLETE
active_plan: LANG005.C4.P4
next_plan: LANG005.C5
planning_baseline_revision: 20ad3f31b39b0fe1b1fdf028df4ac9579fcfd9ad
planning_baseline_commit: "docs: pin lang005 checkpoint revision"
plan:
  - LANG005.C4.P1-explicit-conformance-and-coherence-plan.md
planned_followups:
  - LANG005.C4.P2-witness-resolution-defaults-and-conformance-evidence
  - LANG005.C4.P3-trait-evidenced-dispatch-lowering-and-integration
  - LANG005.C4.P4-completion-and-certification-plan.md
---

# Checkpoint Record — LANG005.C4 Explicit Trait Conformance, Witness Evidence, and Trait-Evidenced Dispatch

## 1. Objective and Ownership Boundary

`LANG005.C4` turns the abstract trait contracts established by C3 into **explicit, coherent, executable conformances**.

The checkpoint answers one question:

```text
Why does this exact target satisfy this exact trait reference,
and precisely which concrete behavior satisfies every behavioral requirement?
```

The full C4 acceptance path is:

```text
source:
    impl TraitRef for Target {
        ...
    }
        ↓
authorized conformance source identity
        ↓
coherence-safe conformance head
        ↓
exact target + exact TraitRef match
        ↓
TraitSurface requirements/defaults
        +
target effective behavior
        ↓
requirement-by-requirement witness/default selection
        ↓
structured ConformanceEvidence
        ↓
trait-evidenced ordinary member availability
        ↓
semantic call/reference selection
        ↓
compiler lowering
        ↓
correct execution
```

C4 owns:

```text
explicit `impl TraitRef for Target`
conformance source/provenance identity
canonical target + TraitRef head resolution
generic conformance heads
trait-or-target ownership
cross-module conformance publication
exact conformance candidate lookup
global coherence / overlap rejection
conformance-local witness bodies
reuse of inherent/data/inherited/conditional witnesses
requirement/witness compatibility
default selection
conformance completeness
structured ConformanceEvidence
trait-evidenced ordinary dispatch
ambiguity across multiple traits/defaults
bound callable references to trait-evidenced behavior
default → requirement → concrete witness lowering
compiler/runtime transport of already-proven evidence
incremental/source/LSP products for all of the above
complex executable conformance regression programs
```

C4 explicitly does **not** own:

```text
associated type declarations/bindings/projection normalization       -> C5
generic trait-bound syntax and generic `T: Trait` proof machinery    -> C6
conditional conformance depending on trait constraints               -> C6
trait objects / existential trait values / public vtable semantics   -> later
public reflection descriptors for conformance/witnesses              -> C7+
supertraits / trait inheritance                                      -> unratified
trait-conformance specialization / "most specific wins"              -> not defined
trait-qualified member syntax                                        -> reserved
metatype/class-side conformance                                      -> reserved
implicit/structural conformance                                      -> forbidden
derived/synthesized conformance                                      -> separate policy
```

C4 must preserve the C1–C3 semantic separations:

```text
representation              != conformance
inherent behavior           != trait-evidenced behavior
trait requirement           != witness
trait default declaration   != default selection
source conformance ImplId   != exact applied conformance
candidate head match        != complete conformance evidence
runtime implementation      != semantic authority
```

---

## 2. Current Lifecycle State

Planning state:

```yaml
checkpoint: LANG005.C4
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active_plan: LANG005.C4.P4
next_plan: LANG005.C5
```

Execution state on the live repository:

```text
P1/P2 complete; P3 runtime and terminal closure landed on the takeover
branch; P4 T0 complete; P4 editor/LSP projection, exact-case closure,
terminal-state propagation, detached conformance-owner runtime lookup,
anti-injection coverage, and temporary-probe cleanup are implemented.
Focused C4 evidence is green. Broad certification is BASELINE_BLOCKED by
pre-existing Universe, incremental, Iterable-stress, formatting, and Clippy
failures recorded in the P4 completion records.
```

The P4 takeover baseline is `df2272d179e04eb18d50fd4e4bbb1ed7c80630c6`
(`ci: run C4 P3 final closure probes`). T0 re-grounding confirmed the
generic-source semantic and runtime paths already pass at this revision. The
editor facade now projects exact proven trait selections through the existing
`TraitDispatchIndex`/`resolve_trait_evidenced_candidates` authority, including
data-component witnesses; the LSP completion adapter preserves exact receiver
types. Temporary C4 P3 source-rewriting scripts and branch-specific workflows
were removed after their coverage was represented by normal tests.

The planning baseline remains pre-C3, but T0 re-grounding verified the live
post-C3 predecessor state before production edits began. P1 and P2 remain
complete; the current completion plan is focused-tested but not
release-certified.

The live implementation base for this P2 continuation is `a7861a5b148715179ef81e2dfe82986a7d0fc499` (`lang005: complete explicit conformance coherence`), with the scoped working-tree changes recorded by the P2 walkthrough and handoff.

This continuation additionally established the canonical instantiated
requirement view, proof-state-aware callable/data compatibility entry points,
an inherent-only effective witness query, exact target/data specialization,
source-plan/exact-evidence semantic fingerprints, proof-aware terminal
applicability propagation, exact C2 applicability evidence retention, and
incremental fingerprint/invalidation parity. P2 is complete at
`IMPLEMENTED` + `FOCUSED_TESTED`; P3 runtime/terminal closure is landed, and
P4 owns the remaining certification state.

P3 T0 re-grounding captured the live post-P2 baseline at
`1fc1e8449e6dd77fc533455bcb056fe9b00290b9`. The semantic crate checks
cleanly, while the first core/workspace check exposed the expected F01
owner-boundary defect in `phalcom-core/src/modules/semantic_lowering.rs`:
inherent projection matched only declaration- and variant-owned callables.
T1 now quarantines `CallableOwnerId::Conformance` from that projection,
removes the owner `Deref<Target = DeclarationId>` coercion, and migrates the
resulting module/LSP accessors to explicit safe APIs. Workspace compilation
is clean after this closure. Conformance lowering and runtime projection
remain unimplemented P3 work.

T0 of P1 proved:

```text
EC-01  C1 complete
EC-02  C2 complete
EC-03  C2 applicability proof-state closure complete
EC-04  C3 complete
EC-05  no hidden partial C4 implementation contradicts this checkpoint
```

The pre-C4 proof-state closure is satisfied. C4 must be able to distinguish at least:

```text
proved
disproved
unknown / insufficient evidence
blocked analysis
dynamic boundary
```

A collapsed `Applicable | NotApplicable | Blocked` model is insufficient for witness/conformance proof construction.

---

## 3. Repository Planning Baseline

Planning baseline:

```text
repository: aureat/phalcom-lang
branch: main
revision: 20ad3f31b39b0fe1b1fdf028df4ac9579fcfd9ad
commit: docs: pin lang005 checkpoint revision
planning date: 2026-09-14
```

This revision includes additional LANG005 predecessor repair work after the original C2 completion revision, including generic call-entry reification. It remains a **planning baseline**, not the eventual C4 implementation baseline.

T0 must capture the actual post-C3 implementation baseline with:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -1 --oneline
```

and must preserve unrelated worktree changes.

### Planning-baseline facts that constrain C4

At the planning baseline:

1. `ImplId` is already the stable source/provenance identity for an `impl` declaration.
2. implementation-local generics already have canonical ownership through `TypeParameterOwner::Impl(ImplId)`.
3. C2 already has canonical generic target/domain matching machinery for inherent impl applicability; C4 should reuse/generalize matching primitives without reclassifying conformances as inherent impls.
4. `InherentImplTarget` currently separates declaration targets from exact enum-case `VariantId` targets.
5. `DeclarationSurface` and conditional inherent member products are target-owned inherent behavior and must remain untouched by conformance publication.
6. `CallableOwnerId` at the planning baseline is declaration/variant-only. C4.P2, not P1, owns the deliberate introduction of conformance-local callable ownership if the landed C3 architecture still requires it.
7. C3 is expected to provide `TraitRef`, `TraitRequirementId`, `TraitSurface`, trait-owned default `CallableId`, abstract `Self`, trait source/index identity, and semantic-only abstract call targets.
8. C4 must consume C3's exact landed APIs rather than reconstructing trait surfaces from syntax.
9. source/incremental products already distinguish structural/signature/body fingerprints; conformance head identity must receive the same treatment.
10. compiler/runtime lowering already transports semantic selections in several places; C4.P3 should project proven conformance/witness selection through those boundaries rather than performing runtime trait discovery.

---

## 4. Canonical Checkpoint Location

Use the LANG005 checkpoint-family directory style:

```text
docs/implementation/LANG005/
  LANG005.C4-explicit-conformance-and-evidence/
    LANG005.C4-CHECKPOINT.md
    LANG005.C4-GUIDANCE.md
    LANG005.C4.P1-explicit-conformance-and-coherence-plan.md
    LANG005.C4.P1-walkthrough.md
    LANG005.C4.P1-handoff.md
    LANG005.C4.P2-...
    LANG005.C4.P3-...
```

Do not reorganize C1–C3 while landing C4 records.

---

## 5. Plan Ledger

| Plan | Scope | Status | Completion | Verification | Exit product |
|---|---|---|---|---|---|
| `LANG005.C4.P1` | explicit conformance declaration, canonical head resolution, ownership, publication, exact query, coherence, incrementality/source projection | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | coherence-safe `ConformanceContribution` + exact `ConformanceHeadMatch`; **no satisfaction proof** |
| `LANG005.C4.P2` | witness sources, conformance-local callables, compatibility, defaults, completeness, structured evidence | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | source `ConformanceWitnessPlan` and exact `ConformanceEvidence` resolver with terminal applicability, C2 specialization evidence, and incremental parity |
| `LANG005.C4.P3` | trait-evidenced ordinary dispatch, ambiguity, callable refs, default/requirement lowering, compiler/runtime execution, integration certification | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | executable conformance semantics; runtime and terminal closure |
| `LANG005.C4.P4` | completion, exact generic/enum closure, editor/LSP projection, terminal propagation, probe cleanup, and certification | COMPLETE | IMPLEMENTED | BASELINE_BLOCKED | focused C4 closure with broad baseline blockers |

The planned checkpoint sequence is:

```text
P1
    source conformance graph
    ownership
    exact head matching
    global coherence
        ↓
P2
    requirement satisfaction
    witnesses/defaults
    complete proof object
        ↓
P3
    ordinary member availability
    call/reference selection
    lowering/execution
    full vertical acceptance
```

The phase boundary is architectural, not administrative.

A unique P1 candidate is **not** proof of conformance.

---

## 6. Adopted Architecture

### 6.1 Shared `impl` syntax, separate semantic products

The source forms:

```phalcom
impl Target {
  ...
}

impl TraitRef for Target {
  ...
}
```

share one implementation-declaration mechanism.

They do not share the same semantic output.

```text
inherent impl
    → target-owned behavior contribution

conformance impl
    → source declaration establishing a potential proof relationship
```

A conformance must never be merged into:

```text
DeclarationSurface
ConditionalInherentMemberSet
runtime class method tables as semantic authority
```

### 6.2 Source identity remains `ImplId`

`ImplId` identifies the source implementation declaration.

For:

```phalcom
impl<T> Tagged for Value<T> {}
```

one `ImplId` can apply to many exact target applications.

Therefore:

```text
ImplId
    !=
exact target + exact TraitRef
```

C4 must not allocate one source identity per generic instantiation.

### 6.3 Exact relationship identity

The exact semantic relation is conceptually:

```text
ExactConformanceKey {
    exact_target,
    exact_trait_ref,
}
```

This key is snapshot/type-store sensitive wherever `TypeId` participates and must obey the repository's existing snapshot-safety rules.

The checkpoint does not mandate a standalone opaque `ConformanceId`.

Add one only if a concrete post-C3 requirement demonstrates that the exact relation needs durable opaque identity beyond:

```text
source ImplId
+
exact key
+
structured evidence
```

### 6.4 Target identity

Conformance targets are nominal:

```text
class/data nominal declaration
enum root
exact enum case
generic application of a nominal target
```

Anonymous structural tuple/record/function types are not direct conformance targets.

Exact enum cases preserve `VariantId` identity.

For generic targets, exact generic application matters:

```text
Value<String> != Value<Int>
```

for conformance lookup even though both share the same declaration identity `Value`.

### 6.5 Exact trait reference identity

A conformance applies to an exact `TraitRef`, not merely a trait declaration.

```text
Converter<Int> != Converter<String>
```

A target may independently conform to both where coherence and member semantics permit.

### 6.6 Trait-or-target ownership

A conformance is authorized only when:

```text
conformance module == canonical trait owner module
OR
conformance module == canonical target owner module
```

For an exact enum case, target ownership comes from the owning enum declaration.

Re-export does not transfer ownership.

Authorization and coherence are separate:

```text
two conformances can each be legally located
but still conflict globally
```

### 6.7 Cross-module static publication

Because either side may own the conformance, target-local publication alone is insufficient.

The workspace must have a semantic index capable of finding authorized conformance candidates independent of:

```text
import order
source order
runtime module initialization
which side owns the source declaration
```

The index should use a coarse family bucket such as:

```text
nominal target family + trait declaration
```

then run exact/generic head matching inside the bounded bucket.

### 6.8 Generic head matching

A generic source conformance:

```phalcom
impl<T> Tagged for Value<T> {}
```

can match:

```text
Value<Int>    + Tagged   with T := Int
Value<String> + Tagged   with T := String
```

The match product must preserve the implementation-local substitution/environment.

Alpha-renaming generic parameters must not change semantics.

### 6.9 Coherence

For one exact target + exact TraitRef, at most one applicable conformance source may exist.

There is no:

```text
last definition wins
last import wins
source-order precedence
trait-owner precedence
target-owner precedence
"more specific" conformance selection
```

Therefore:

```phalcom
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Int> {}
```

is invalid because both apply to:

```text
Value<Int> + Tagged
```

C2 specialized inherent behavior does not imply trait-conformance specialization.

### 6.10 Joint target + trait overlap

Coherence must compare the full relationship domain, not only target domains.

Example:

```phalcom
impl<T> Converter<T> for Value<T> {}
impl Converter<Int> for Value<String> {}
```

The first source at `Value<String>` means `Converter<String>`, so it does not overlap the second relation `Value<String> + Converter<Int>`.

A target-only overlap checker would be incorrect.

### 6.11 Candidate match is not evidence

P1 may prove:

```text
exact target + exact TraitRef
    has one authorized, coherence-safe source head
```

It may not prove:

```text
every requirement is satisfied
```

The latter belongs to P2.

Use names such as:

```text
ConformanceContribution
ConformanceHeadMatch
ConformanceCandidate
```

not:

```text
ConformanceEvidence
Conforms
Satisfied
```

for P1 products.

### 6.12 Witness sources

P2 must support requirements satisfied by:

```text
conformance-defined witness body
existing effective inherent callable
inherited effective callable
C2 proven conditional/specialized inherent callable
data component for compatible property requirement
trait default
```

The same inherent callable may witness multiple distinct `TraitRequirementId`s.

Data components retain `DataComponentId`; no synthetic getter `CallableId` is required.

### 6.13 Conformance-local witness ownership

A witness body written inside:

```phalcom
impl Printable for User {
  toString -> String { ... }
}
```

is not an inherent `User` member.

If the post-C3 architecture still requires a callable identity extension, the preferred model is conceptually:

```text
CallableOwnerId::Conformance(ImplId)
```

or the exact repository-equivalent identity.

P2 owns this migration.

Conformance-local bodies may satisfy trait requirements; they may not become an arbitrary extension namespace for helper methods.

### 6.14 Default selection

For one requirement in one conformance:

```text
explicit conformance witness
    otherwise
compatible effective concrete witness
    otherwise
trait default
    otherwise
conformance incomplete
```

A selected trait default remains trait-owned `CallableId`.

It is never copied into the target's inherent surface.

### 6.15 Structured evidence

P2's successful semantic proof is conceptually:

```text
ConformanceEvidence {
    source_impl,
    exact_target,
    exact_trait_ref,
    impl_substitution,
    target/trait substitutions,
    requirement_selections,
}
```

C5 later extends this with associated-type bindings/projections.

C6 later extends it with nested trait-constraint evidence.

A boolean `conforms: true` is not the authoritative semantic result.

### 6.16 Trait-evidenced ordinary behavior

P3 makes behavior selected through conformance evidence available to ordinary instance syntax without reclassifying it as inherent.

```text
available through ordinary dispatch
    !=
owned by target declaration
```

When multiple traits contribute the same ordinary selector:

```text
same concrete selected witness
    → ordinary call can converge unambiguously

different competing defaults
    → ordinary unqualified call is ambiguous
```

No declaration/import order chooses a default.

### 6.17 Default → requirement lowering

C3 defaults are checked once against abstract `Self`.

Example:

```phalcom
trait Comparable<Rhs> {
  compare(_ other: Rhs) -> Ordering

  <(_ other: Rhs) -> Bool {
    self.compare(other) === Ordering::Less
  }
}
```

For a concrete conformance, P3 must execute:

```text
Comparable.< default
    ↓
abstract requirement call Comparable.compare
    ↓
current ConformanceEvidence
    ↓
selected concrete witness
```

The runtime must not rediscover the trait relationship by scanning receiver classes.

### 6.18 No implicit conformance propagation

Unless a later ratified rule changes this, C4 treats conformance as attached to the declared target/domain.

A class inheriting behavior from a parent may use inherited behavior as a **witness** for a conformance declared on the child.

A conformance declared on a parent does not automatically create a child conformance merely because the class hierarchy relates the two types.

This rule must remain explicit in C4 tests and handoffs.

---

## 7. Established C4 Invariants

P2 implementation state now establishes:

- conformance-local witness callables use `CallableOwnerId::Conformance(ImplId)` and are published as semantic products without entering target-owned inherent surfaces;
- witness bodies are checked with a target-relative `Self` override while retaining the conformance source module as their lexical context;
- each eligible source conformance publishes one `ConformanceWitnessPlan`, and exact evidence resolution consumes the plan after unique P1 head matching rather than reselecting witnesses;
- source selection precedence is explicit witness, effective inherent callable, readable data component, then trait default; invalid explicit members keep the plan incomplete and cannot fall through to defaults.
- effective conditional-member lookup preserves `Unknown`, `Blocked`, `Dynamic`, `Cancelled`, `BudgetExceeded`, and `InternalFailure` outcomes; a proven data/default fallback may still satisfy the requirement;
- conditional inherent selections retain canonical `InherentImplSpecialization` evidence and exact evidence materializes that product through the exact conformance environment;
- generic callable witness constraints are checked directionally: candidate obligations must be implied by requirement obligations after alpha-renaming.

1. Conformance is explicit; matching members alone do not establish it.
2. Trait identity is nominal and declaration-based.
3. Exact trait arguments participate in conformance identity.
4. Exact target generic arguments participate in conformance identity.
5. `ImplId` is source/provenance identity, not exact conformance identity.
6. One generic source conformance may match many exact applications.
7. Inherent impl and conformance impl share source machinery but produce separate semantic products.
8. Conformance does not modify semantic instance representation.
9. Conformance does not inject members into `DeclarationSurface`.
10. Conformance does not publish members into C2 conditional inherent products.
11. Conformance targets are nominal declarations or exact enum cases.
12. Exact enum case identity is preserved through `VariantId`.
13. Structural anonymous types are not direct conformance targets.
14. The conformance module must own the trait or target.
15. Re-export does not transfer ownership.
16. Authorization and coherence are separate analyses.
17. Conformance publication is static and independent of runtime loading/import order.
18. Coherence is global for exact target + exact TraitRef.
19. There is no trait-conformance specialization precedence.
20. Overlap analysis considers target and TraitRef jointly.
21. Alpha-renaming impl generics does not change the conformance domain.
22. Invalid or unauthorized declarations do not pollute exact lookup.
23. Addition/removal/replacement of conformance contributions is owner-complete.
24. Body-only witness edits do not change P1 conformance-head identity.
25. Header edits invalidate the correct candidate/coherence products.
26. A P1 unique head match is not a conformance proof.
27. P2 validates every applicable behavioral requirement.
28. Requirement identity remains `TraitRequirementId`.
29. Witness identity is distinct from requirement identity.
30. An inherent callable may witness multiple trait requirements.
31. Data components may satisfy readable property requirements without synthetic getters.
32. An immutable data component cannot alone satisfy a writable property requirement.
33. Inherited effective behavior may be used as a witness.
34. C2 conditional/specialized behavior may be used only when its applicability is proven by the canonical C2 proof machinery.
35. Unknown, blocked, or dynamic proof states do not silently count as successful witnesses.
36. A conformance-local witness body is not an independent inherent overload.
37. Conformance bodies may not define unrelated extension helper members.
38. Concrete witness selection takes precedence over a trait default.
39. Trait defaults remain trait-owned callables after selection.
40. Conformance completeness requires every C4 behavioral requirement to be satisfied.
41. Associated-type completeness is not implemented until C5.
42. Structured `ConformanceEvidence` is the semantic authority for successful satisfaction.
43. Runtime/class-table scanning is not semantic authority for conformance.
44. Trait-evidenced behavior may participate in ordinary syntax without becoming inherent.
45. Shared concrete witness convergence is unambiguous.
46. Competing applicable trait defaults are ambiguous without a concrete resolver.
47. Source/import order never resolves competing defaults.
48. Bound references retain the same semantic conformance/witness selection as the original lookup.
49. Default abstract calls resolve through current conformance evidence.
50. Class-side/metatype conformance is not implied by instance-side conformance.
51. Parent conformance does not implicitly create child conformance in C4.
52. Cold and incremental analysis produce equivalent conformance/coherence/evidence results.
53. LSP/source tooling projects semantic identities and relations; it does not reconstruct them from text.
54. P1, P2, and P3 preserve the semantic/runtime authority split.
55. C4 does not implement C5 associated types or C6 conditional trait constraints by accident.

---

## 8. Testing Doctrine

C4 testing has four layers:

```text
Layer A — semantic micro-tests
    one invariant / failure cause

Layer B — semantic interaction tests
    several mechanisms combined

Layer C — executable vertical tests
    parser → semantic → lowering → VM

Layer D — adversarial/incremental tests
    overlap, ownership, edits, deletion, stale evidence
```

C4 is not complete with only Layer A/B tests.

### 8.1 P1 testing focus

P1 must cover:

```text
syntax normalization
trait/target resolution
generic impl scope
trait ownership
target ownership
third-party rejection
re-export rejection
exact TraitRef identity
exact target generic identity
exact enum-case targets
generic source head specialization
joint overlap
generic+exact overlap rejection
cross-owner conflict
order independence
add/edit/delete lifecycle
cold/incremental parity
surface separation
```

The `Value<String>` / `Value<Int>` case is a primary regression:

```phalcom
trait Tagged {
  tag -> String
}

class Value<T> {}

impl Tagged for Value<String> {
  tag -> String { "string" }
}

impl Tagged for Value<Int> {
  tag -> String { "int" }
}
```

P1 must distinguish the two exact conformance heads even though the nominal declaration identity is shared.

P1 does not yet validate or execute the `tag` bodies.

### 8.2 P2 testing focus

P2 must test all witness origins:

```text
direct inherent callable
inherent-impl callable
inherited callable
C2 proven conditional/specialized callable
conformance-local witness body
data component
trait default
```

and all major compatibility dimensions:

```text
selector
member role
dispatch side
Self substitution
trait generic substitution
impl generic substitution
parameter types
result types
callable generics
visibility/access coverage
property mutability
```

### 8.3 P3 testing focus

P3 must test:

```text
ordinary trait-evidenced member call
bound callable reference
shared-witness convergence
competing-default ambiguity
selected default execution
conformance-local witness execution
default → abstract requirement → concrete witness
nested default → default → witness paths
compiler/runtime evidence transport
no runtime trait scanning
```

### 8.4 C4 generic `Iterable` vertical fixture

The canonical LANG005 support `Iterable` uses associated types and therefore becomes a C5 fixture in its final form.

For C4, use a generic-parameter equivalent:

```phalcom
trait Iterable<Item, Cursor> {
  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item

  each(_ f) { ... }
  count -> Int { ... }
  contains(_ expected: Item) -> Bool { ... }
  toList -> List<Item> { ... }
}
```

This deliberately tests C4 without smuggling C5 associated types into the checkpoint.

A strong mixed-witness conformer is:

```text
RangeView
    iterate        → existing inherent callable
    iteratorValue  → conformance-local witness
    each/count/... → trait defaults
```

Executing `toList` then proves:

```text
conformance
→ default toList
→ default each
→ abstract iterate requirement
→ inherent witness
→ abstract iteratorValue requirement
→ conformance-local witness
→ result
```

### 8.5 `Iterator<Item>` vertical fixture

A second fixture should use mutable receiver state:

```phalcom
trait Iterator<Item> {
  next -> Option<Item>

  nextOr(_ fallback: Item) -> Item { ... }
  countRemaining -> Int { ... }
  drain -> List<Item> { ... }
}
```

with a stateful `CountdownIterator`.

This catches:

```text
conformance-local receiver binding
field access through concrete Self
repeated default→witness invocation
state mutation across calls
multiple independent instances
frame/environment contamination
```

### 8.6 Required complex acceptance corpus

By P3 certification, maintain at least one substantial valid program containing:

```text
6–8 traits
8–12 target types
15–20 conformances
```

with combinations of:

```text
explicit witnesses
inherent witness reuse
data-component witnesses
inherited witnesses
C2 specialized/conditional witnesses
trait defaults
defaults calling requirements
defaults calling defaults
same callable witnessing multiple traits
distinct generic TraitRefs
specialized generic targets
exact enum-case conformance
bound references
```

Also maintain a compile-fail/adversarial corpus for:

```text
third-party ownership
duplicate exact conformance
generic overlap
generic+exact overlap
missing witness
incompatible witness
visibility mismatch
property mutability mismatch
extra conformance-local member
duplicate explicit witness
competing defaults
unsupported associated binding
unsupported conditional trait constraint
unsupported metatype conformance
```

Each negative case should assert the precise diagnostic family.

---

## 9. Stable Interface / Takeover Map

### C3 interfaces C4 expects to consume

Names may differ in the landed repository, but semantic roles must exist:

```text
is_trait(DeclarationId)
TraitRef
TraitSurface
TraitRequirementId
trait generic signature
trait default CallableId
requirement/default source provenance
abstract Self representation
trait source/index targets
semantic-only abstract callable application
```

### C2/C1 interfaces C4 expects to consume

```text
exact TypeId identity
exact enum-case VariantId identity
generic substitution/environment machinery
nominal target specialization
effective inherent surface
conditional/specialized inherent lookup
rich applicability proof result
class hierarchy/effective dispatch lookup
DataComponentId
incremental module/source replacement
source fingerprints
```

### C4.P1 interfaces to establish

Conceptually:

```text
ConformanceTarget
ResolvedConformanceHead
ConformanceContribution
ConformanceIndex
ConformanceFamilyKey
ConformanceHeadMatch
exact candidate query
ownership validator
coherence validator
incremental replacement/removal
source/LSP conformance relationship projection
```

### C4.P2 interfaces to establish

Conceptually:

```text
conformance-local callable identity/ownership
RequirementSelection
WitnessSource
WitnessCompatibility
ConformanceEvidence
complete conformance query
diagnostics for missing/incompatible witnesses
```

### C4.P3 interfaces to establish

Conceptually:

```text
trait-evidenced member candidate
ordinary lookup integration
ambiguity product
call/reference selection carrying ConformanceEvidence
default requirement-call lowering
compiler/runtime evidence projection
execution support
```

---

## 10. T0 Entry Lock — Post-C3 Re-grounding

T0 was executed against the live post-C3 tree on 2026-09-14.

```text
starting revision: 9f6c9b8df8c06671eee51551495aca3a3aa92bdc
starting working tree: clean
active plan: LANG005.C4.P1
```

The predecessor takeover map is now grounded in live symbols:

```text
C3 trait declaration/surface authority:
    phalcom-semantic/src/capabilities.rs
    phalcom-semantic/src/checker/context.rs
    phalcom-semantic/src/traits.rs
C2 inherent-impl applicability authority:
    phalcom-semantic/src/impls.rs
    InherentImplDomain
    check_impl_domain_applicability
    ImplApplicabilityResult
C3 source/incremental products:
    phalcom-semantic/src/source_index.rs
    phalcom-semantic/src/incremental.rs
```

The C2-F03 entry prerequisite is closed at the result boundary. Applicability
now preserves distinct `Applicable`, `NotApplicable`, `Unknown`, `Blocked`,
`Dynamic`, `Cancelled`, `BudgetExceeded`, and `InternalFailure` outcomes;
only the proven `Applicable` outcome constructs an
`InherentImplSpecialization`. Constraint evaluation delegates bounded subtype
relations to the existing semantic relation authority and retains explicit
uncertainty or analysis-boundary outcomes instead of treating them as
disproof.

Focused evidence for the entry lock:

```text
cargo test -p phalcom-ast --test integration impl_syntax
    12 passed, 0 failed
cargo test -p phalcom-semantic --test semantic impls
    50 passed, 0 failed
cargo check -p phalcom-semantic
    passed
cargo fmt --all
    passed
git diff --check
    passed
```

No hidden C4 implementation was found in the targeted semantic, source-index,
or core trait surfaces. The live C3 AST was declaration-only; P1 is the first
conformance source-graph implementation. T1 and the initial T2–T7 semantic
slices are now landed; cross-module ownership/publication and source/LSP
projection are included in the completed P1 evidence below.

## 11. Durable P1 Progress

```text
T1/G1  accepted: shared ImplDef now distinguishes Inherent and Conformance;
        trait reference, `for`, target, generic parameters, body, and ranges
        remain independently recoverable. Existing inherent syntax is green.
T2     accepted at the current semantic boundary: ResolvedConformanceHead
        retains ImplId, impl-owned generic signature, canonical TraitRef,
        neutral ConformanceTarget, exact target head, eligibility, and source.
T3     accepted: source module must canonically own the trait or target
        declaration/variant. Unauthorized heads remain diagnostic-only and
        are excluded from lookup.
T4/T5  accepted locally: ConformanceContribution and snapshot-owned
        ConformanceIndex publish exact head matches without inherent-surface
        or callable-definition pollution; one impl substitution specializes
        both target and TraitRef arguments.
T6     accepted locally: pairwise joint target+TraitRef unification rejects
        generic/specialized overlap without source-order precedence.
T7     accepted: changed-module removal/replacement preserves index
       add/edit/delete behavior; exact enum-case source projection retains
       VariantId; cold and incremental head publication agree.
T8     accepted: adversarial P1 matrix covers exact specialized targets,
       absent specialization, generic trait references, joint overlap, and
       the multi-module ownership/re-export topology without entering P2.
T9     accepted: focused G1–G5 evidence is green; walkthrough and P2 handoff
       are recorded below as separate durable plan records.
```

P1 is complete at `IMPLEMENTED` + `FOCUSED_TESTED`. The stable P2 inputs are
the snapshot-owned `ConformanceIndex`, exact `ConformanceHeadMatch`, source
`ImplId`, canonical target/TraitRef heads, and impl substitution environment.
P1 intentionally produces no witness, default-selection, completeness, or
`ConformanceEvidence` product.

## 12. Deferred / Cross-Checkpoint Ledger

### PRE-C4-01 — applicability proof-state closure

Closed at T0 result-boundary implementation and focused verification above.
The richer result is an applicability-analysis product only; it is not
conformance evidence and does not authorize witness selection.

### C5 — associated types

C4 may parse or preserve future syntax only if needed for forward compatibility, but it must not implement:

```text
AssociatedTypeId
associated binding validation
projection normalization
associated-type cycles
```

The C4 generic `Iterable<Item, Cursor>` fixture intentionally avoids this.

### C6 — generic trait constraints / conditional conformance

Do not implement:

```text
GenericConstraint::Conforms
nested generic conformance proof
`where T: Trait` conformance applicability
conditional conformance evidence
```

A C4 conformance head may be generic, but trait-conditioned applicability remains C6.

### Later — trait objects/vtables/reflection

C4 runtime support may use internal witness tables or specialization if useful, but that does not define public trait-object semantics or public reflection descriptors.

---

## 13. C4.P1 Task / Gate Sequence

The active P1 plan sequence is:

```text
T0  post-C3 re-grounding and entry lock
T1  normalize `impl TraitRef for Target` syntax
G1  entry and syntax gate

T2  resolve conformance heads and canonical target identity
T3  enforce trait-or-target ownership
G2  canonical conformance-head gate

T4  publish ConformanceContribution and workspace index
T5  exact candidate lookup and generic head specialization
G3  publication and exact-query gate

T6  global coherence and overlap validation
T7  incremental fingerprints, replacement/removal, source/LSP projection
G4  coherence and incrementality gate

T8  adversarial conformance corpus and separation tests
T9  stabilization, P1 closure, walkthrough, P2 handoff
G5  P1 certification gate
```

P1 closes only when P2 can consume stable identity/ownership/coherence APIs without reopening them.

---

## 14. Verification Expectations

### BUILD mode

Run only the smallest tests that distinguish the current implementation hypothesis.

Do not repeatedly run whole-workspace verification after small parser/semantic edits.

### Gate progression

At each gate, expand verification only enough to prove the newly durable architecture.

### Final P1 certification

P1 final verification must include:

```text
focused parser tests
focused semantic conformance tests
ownership tests
generic exact-target/TraitRef tests
coherence tests
incremental lifecycle tests
source/LSP tests where affected
existing inherent-impl regressions
cold/incremental parity
complex P1 semantic fixture
negative anti-injection assertions
```

P1 should require no VM/runtime semantic changes.

If P1 requires runtime trait registration or class-table mutation, STOP: the semantic layering has been violated.

---

## 15. Completion Criteria

### C4.P1 complete when

All of the following are true:

```text
`impl TraitRef for Target` is losslessly represented
trait and target resolve separately
generic conformance heads retain impl-owned generics
trait-or-target ownership is canonical
cross-module publication is complete
exact generic targets remain distinct
exact TraitRefs remain distinct
exact enum cases remain exact
candidate query produces structured head match
global overlap/coherence is deterministic
no conformance specialization exists
incremental add/edit/delete is owner-complete
cold/incremental results agree
conformance body members are not inherent
P1 does not claim satisfaction
walkthrough + P2 handoff exist
```

### C4.P2 complete when

All of the following are true:

```text
every behavioral requirement is examined
all supported witness origins are represented
conformance-local witness bodies have correct identity
Self/trait/impl substitutions are correct
visibility and member-role compatibility are enforced
defaults are selected with correct precedence
incomplete conformances are diagnosed
structured ConformanceEvidence is authoritative
P2 does not yet require ordinary trait-evidenced execution
```

### C4.P3 complete when

All of the following are true:

```text
trait-evidenced behavior participates in ordinary lookup
shared concrete witnesses converge
competing defaults diagnose ambiguity
bound references preserve selection
trait defaults execute
conformance-local witnesses execute
default abstract calls reach selected concrete witnesses
semantic evidence reaches lowering/runtime
runtime does not define conformance by scanning classes
complex Iterable/Iterator vertical programs pass
incremental + cold + compiler/runtime integration are certified
C4 walkthrough/handoff/checkpoint are final
```

---

## 16. Active Plan and Next Action

### Active plan

```text
LANG005.C4.P4 — completion and certification
```

### Next plan

```text
LANG005.C5 — associated types and projection
```

### Current next action

P4 completion evidence on 2026-09-15:

```text
focused semantic impls query:                         PASS (1)
focused semantic impls suite:                         PASS (83)
focused semantic editor suite:                        PASS (10)
focused semantic incremental trait suite:              PASS (3)
focused core C4 trait closure suite:                   PASS (3)
focused core trait runtime suite:                      PASS (17)
focused LSP completion suite:                          PASS (7)
focused AST impl syntax suite:                         PASS (12)
workspace check:                                       PASS
workspace test:                                        BASELINE_BLOCKED
workspace clippy -D warnings:                          BASELINE_BLOCKED
workspace format check:                                BASELINE_BLOCKED
```

The targeted implementation establishes exact generic-source specialization,
preserves terminal proof states through checker analysis, attaches conformance
semantics to formal top-level checking, projects proven trait members through
the editor/LSP facade, and keeps conformance-owned runtime methods detached.
Exact enum-case dispatch now retains the complete applied target. The focused
runtime witness test also asserts that a conformance-local member is absent
from the target class dictionary.

Known baseline blockers are: the existing Universe
`kernel_option_match_override_reroutes_every_combinator` internal generic
call-entry failure; the existing incremental A7 cold/incremental presentation
parity divergence; the existing two Universe Bool declaration-shell failures
in `capabilities::traits`; the existing Iterable/outgoing-pack generic
call-entry failures; pre-existing formatting drift in `phalcom-semantic/src/db/query.rs`
and `tests/semantic/associated/lookup.rs`; and pre-existing AST
`large_enum_variant` Clippy violations. These remain outside the focused C4
implementation boundary.

The requested completion plan is therefore `IMPLEMENTED` + `FOCUSED_TESTED`
for its landed semantic/runtime/editor closure, with checkpoint verification
`BASELINE_BLOCKED`; C4 is not marked release-complete.
