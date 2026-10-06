# LANG007.C4.P4 — Remaining-Implementation Audit and Replacement-Plan Requirements

**Program:** `LANG007`
**Checkpoint:** `LANG007.C4` — Structural Static Upgrade
**Plan under audit:** `LANG007.C4.P4` — Generalized `FieldKey` and Unordered-Product Calculus
**Audit purpose:** define the requirements for a **new implementation plan** that starts from the current committed/pushed P4 state, repairs architectural mistakes and incomplete seams, and carries P4 to a truthful completion.
**Repository:** `aureat/phalcom-lang`
**Live branch audited:** `fix/LANG007-a1-match-equality-authority`
**Lifecycle anchor supplied by the current handoff/walkthrough:** `548c7a3629832b50c2d5bfd35529abc759de322e`
**Audit date:** 2026-10-06

---

# 0. Executive verdict

The core P4 direction remains correct, but the **active implementation plan should no longer be used as the execution plan for the remainder of P4**.

The first six implementation slices have established a substantial and mostly coherent substrate:

- one semantic `FieldKey = Symbol | String | Int | Bool`;
- language-defined canonical ordering;
- canonical `FieldKey` rows and key-domain-aware anonymous Record tails;
- existing-row-solver migration rather than replacement;
- `UnorderedFieldLookup`;
- `StaticValue::Record` with `FieldKey`;
- runtime Record shapes with canonical logical keys and a separate presentation permutation;
- heterogeneous runtime construction/access/equality/hash support;
- preservation of the Symbol-only Tuple/Pack/call-label boundary.

Those foundations should **not be rewritten** without a specific regression.

The remaining P4 work is nevertheless not a simple continuation of the current amended T7.0D worklist. The live implementation exposes a structural problem at the root of the prolonged T7.0 effort:

> **Static-constant formation now has two executable owner paths.**

The pre-existing session still contains the P2 static-constant declaration/graph/SCC/check/evaluate/publication pipeline, while the new `StaticConstantDemandDriver` independently discovers a constant, checks it, recursively forces dependencies, evaluates it, constructs `StaticConstantProduct`, creates recursive-cycle products, and calls the database publication query.

That is more than a scheduler. It is a second implementation of the same semantic owner.

This violates the intended “one owner / one publisher / one dependency authority” architecture in substance even though both paths eventually call the same database publishing function. It also explains why T7.0 has accumulated increasingly broad amendments for context prerequisites, type-member readiness, alias deferral, exact demand routing, active-stack cycles, and fresh-context retry: the implementation is compensating for an owner boundary that was not first made demand-safe.

A second concrete defect follows from the same split. Projection-aware `FieldSignature` and `CallableSignature` construction currently depends on `ProjectedRecordKeySet` prepared externally by the session. Generic semantic-dependency replay, however, invokes those signature queries with `record_key_projections: None`. Therefore the canonical replay path can bypass the very computed-key preparation required to reproduce the original product. This is incompatible with robust incremental revalidation and is precisely the class of failure that T7.0E is intended to catch.

The correct replacement plan should therefore begin by **converging owner execution**, not by adding more special cases to `StaticConstantDemandDriver`.

The required direction is:

1. keep the established P4 T1–T6 substrate;
2. replace the current specialized demand-driver architecture with a bounded **exact semantic-owner executor/coordinator** that schedules canonical owners but does not independently implement their semantics;
3. establish one executable formation path for `StaticConstant`, and route both eager and demand-driven formation through it;
4. make every projection-aware owner re-enterable through the same canonical owner entrypoint, including dependency replay;
5. then implement computed-key alias ownership and run the full T7.0E matrix;
6. only after T7.0E passes, finish heterogeneous Record patterns;
7. narrow the old T8 scope to what P4 actually owns, while preserving P8's durable-schema ownership;
8. fix the dynamic Record-builder quadratic duplicate check before claiming the performance gate;
9. close P4 with focused evidence and truthful lifecycle records.

This is a **repair-and-converge plan**, not a continuation plan.

---

# 1. Audit basis and confidence

## 1.1 Lifecycle material audited

The audit consumed the supplied:

- `LANG007.C4-CHECKPOINT.md`;
- `LANG007.C4.P4-luna-patch-grade-implementation-plan.md`;
- `LANG007.C4.P4-walkthrough.md`;
- `LANG007.C4.P4-handoff.md`.

The lifecycle records agree on the important status facts:

- P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`;
- T1–T6 are recorded as complete and focused-tested;
- T7.0A/B exist;
- selected T7.0C/D owner slices exist;
- computed-key alias ownership is not implemented;
- full imported forcing, transitive mixed-cycle handling, fresh-context/no-leak acceptance, and T7.0E remain open;
- T7.1 is still gated;
- T7.1–T7.5, T8, and T9 are not complete.

That status classification is credible and should be retained.

## 1.2 Live repository implementation audited

The live branch contains only a small number of commits beyond current `main`; the unresolved branch-local code is concentrated primarily in:

```text
phalcom-semantic/src/static_constant_demand.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_key.rs
```

This matters: the P4 problem is **not** that the entire FieldKey migration is unfinished. Most of the low-level vertical is already integrated. The current risk is concentrated in the new static-key/owner scheduling seam and the explicitly unstarted later slices.

The audit was repository/static-analysis based. It did not execute the repository test suite. Test results cited as passing below are lifecycle-recorded evidence, not newly executed evidence.

---

# 2. Corrected P4 objective

The old plan's broad goal is directionally correct but should be rewritten more narrowly around the actual dependency required by P5.

P4 should complete when Telos has **one coherent unordered Record-key calculus** with the following properties:

```text
FieldKey semantic scalar
    ↓
canonical Record row + key-domain-aware tails
    ↓
existing row solver / relation / unordered lookup
    ↓
static Record values
    ↓
checked static projection of computed structural keys
    ↓
runtime Record shape / lookup / construction
    ↓
resolved Record patterns
    ↓
stable incremental identity + explicit durability boundary
```

P4 does **not** need to solve all future static scheduling, durable metadata, or reflection problems.

The completion contract should be:

- all concrete Record keys use exactly `FieldKey`;
- all canonical Record rows use one existing row authority;
- computed Record keys in type and pattern positions use ordinary checker selection plus P2 static evaluation;
- a semantic owner that contains such a key can be recomputed cold or incrementally through its canonical owner entrypoint;
- exact dependencies are recorded;
- cycles terminate deterministically;
- runtime Records preserve all four key kinds;
- Record patterns preserve all four key kinds;
- ordered Tuple/Pack/call labels remain Symbol-only;
- P4 leaves durable heterogeneous metadata to P8 without lossy export;
- no P4-owned canonical path is accidentally quadratic at the required scale.

That is enough to unblock P5 structurally.

---

# 3. Current implementation truth

## 3.1 Implemented substrate that should be preserved

| Area | Live state | Audit ruling |
|---|---|---|
| `FieldKey` | `Symbol`, `String`, exact `Int`, `Bool`; content-owned; explicit canonical order and stable tagged bytes | **Keep** |
| row field identity | `RecordRowField.key: FieldKey` | **Keep** |
| anonymous Record tails | carry `key_type`, optionally `value_type` | **Keep** |
| row canonicalization | canonical FieldKey sort + duplicate rejection | **Keep** |
| row solving | FieldKey-aware; canonical two-pointer merge exists | **Keep** |
| key-domain membership | delegates to ordinary subtype/relation engine | **Keep** |
| unordered lookup | `Required / Optional / Absent / Unknown` | **Keep** |
| static Record | canonical `FieldKey` storage and duplicate rejection | **Keep** |
| static value → FieldKey | content/value conversion through `field_key_of` | **Keep** |
| runtime shape | `logical_keys + presentation_to_logical` | **Keep** |
| runtime lookup | canonical binary search; borrowed comparison helper exists | **Keep** |
| runtime construction | `GuardFieldKey`, Record-specific construction | **Keep with perf fix** |
| Record equality/hash | canonical key order and canonical value order | **Keep** |
| ordered label boundary | Tuple/Pack continue through Symbol-only path | **Keep** |
| metadata bridge | non-Symbol keys/non-Symbol key domains fail closed in legacy export | **Keep as boundary** |

This is the strongest reason **not** to reopen T1–T6 as a wholesale rewrite.

## 3.2 Partially implemented T7.0 substrate

The live code already provides valuable pieces:

- a source/linked `ModuleStaticBindingCatalog`;
- `CheckedStaticExpression`;
- `StaticExpressionCheckOutcome::{Ready, Awaiting, Invalid}`;
- owner-relative `RecordKeyUseId`;
- ephemeral `ProjectedRecordKeySet`;
- exact dependency collection from checked key expressions;
- `project_checked_static_key`;
- field/callable signature projection hooks;
- static-constant declared-type projection support;
- exact missing type-member demands through `TrackingTypeResolver`;
- session-side servicing of selected type-member demands;
- a projection-aware `StaticDefinition` publication helper;
- deferred-static-definition resolver hooks.

These pieces should be reused.

They do **not** yet form a coherent owner-execution architecture.

## 3.3 Clearly unimplemented or incomplete

The following remain materially open:

1. canonical computed-key alias scheduling and publication;
2. exact `StaticDefinition` demand execution;
3. general re-entry of projection-aware owners during dependency replay;
4. imported computed-key forcing acceptance;
5. transitive mixed-cycle acceptance;
6. fresh-context retry/no-partial-diagnostics/no-partial-product proof;
7. T7.0E;
8. heterogeneous Record pattern semantics;
9. heterogeneous pattern lowering and rest exclusion;
10. P4-owned runtime overlay consistency;
11. performance hardening of dynamic Record construction;
12. final incremental/performance/compatibility gates;
13. lifecycle closure.

---

# 4. Root-cause findings

## F-01 — CRITICAL: `StaticConstant` has duplicate executable formation paths

### Evidence

The new `StaticConstantDemandDriver` does all of the following:

```text
force(id)
  -> detect its own active-constant recursion
  -> compute(id)
      -> recover declaration from catalog/source
      -> check dependencies
      -> recursively force dependencies
      -> check/evaluate initializer
      -> construct StaticConstantProduct
      -> call query_static_constant(...)
      -> update StaticConstantTable

publish_recursive_cycle(...)
  -> synthesize recursive StaticConstantProduct
  -> call query_static_constant(...)
```

The session still contains the existing static-constant pipeline that:

```text
collects ConstantDeclInfo
builds constant_graph
tracks record-key dependency origin
checks initializers
handles cycles/SCC ordering
evaluates constants
publishes StaticConstant products
```

The second path was introduced to make constants available early to computed type keys, but it duplicated owner formation rather than making the original owner re-entrant/demandable.

### Why this is architecturally wrong

The canonical semantic fact is:

```text
QueryKey::StaticConstant(id) -> one current StaticConstantProduct
```

There may be multiple callers, but there must be one semantic formation algorithm.

Two builders can drift in:

- prerequisites;
- diagnostics;
- cycle classification;
- checked-context assembly;
- fingerprint inputs;
- imported binding behavior;
- exact dependency recording;
- LKG behavior;
- future static evaluator features.

Even if both call the same final DB publication function, they are still two semantic authorities in practice.

### Required correction

The replacement plan must make **one owner entrypoint** authoritative for StaticConstant formation.

Conceptually:

```rust
ensure_static_constant_current(id, owner_context) -> OwnerOutcome<Arc<StaticConstantProduct>>
```

or mechanically equivalent.

Both:

- the ordinary/eager constant stage; and
- computed-key demand forcing

must delegate to this same owner implementation.

The existing batch graph may remain temporarily as scheduling/optimization data, but it must not retain a separate product-formation implementation.

### Acceptance

A code audit must be able to answer:

> “Where is a `StaticConstantProduct` semantically formed?”

with one executable path, plus trivial constructors used only by that path.

---

## F-02 — CRITICAL: projection-aware signature revalidation can bypass projection

### Evidence

`SignatureQueryInputs` accepts:

```rust
record_key_projections: Option<&ProjectedRecordKeySet>
```

The session prepares projections before calling field/callable signature queries.

However, generic semantic-dependency replay calls `query_callable_signature_with_inputs` and `query_field_signature_with_inputs` with:

```rust
record_key_projections: None
```

### Consequence

The original product may have been formed from:

```text
annotation + reduced computed Record key
```

while replay/revalidation can attempt:

```text
same annotation + no projection
```

This is not a valid recomputation path.

It can lead to:

- false blocked results;
- stale-cache retention;
- incorrect diagnostics;
- unprojected type formation;
- failure to re-establish exact constant dependencies;
- cold/incremental divergence.

### Required correction

No subsystem may revalidate/recompute a projection-aware owner by directly calling a lower-level `query_*` function that assumes projections are already supplied.

All re-entry must pass through the canonical owner executor:

```text
ensure_owner_current(QueryKey::FieldSignature(...))
ensure_owner_current(QueryKey::CallableSignature(...))
```

The owner executor prepares any required computed-key facts and only then invokes publication.

This requirement is broader than “fix two `None` arguments”; otherwise the same bug will recur for aliases and future projection-aware owners.

---

## F-03 — CRITICAL: the current demand driver violates its intended scheduling-only boundary

The amended plan says the driver schedules demands and owners remain publishers.

The live `StaticConstantDemandDriver` is nevertheless also a semantic producer for constants.

The name is therefore misleading and the architecture is conflated.

### Required correction

Split responsibilities:

```text
OwnerDemandExecutor / SemanticOwnerExecutor
    owns:
      active exact QueryKey stack
      demand routing
      progress/cycle state
      current-product refresh
      retry orchestration

StaticConstant owner
FieldSignature owner
CallableSignature owner
StaticDefinition owner
TypeMember owner
...
    own:
      semantic checking
      product formation
      diagnostics
      exact dependency set
      publication
```

The scheduler may call owners. It must not contain a second implementation of them.

---

## F-04 — CRITICAL: alias support is scaffolding, not an implementation

### Live evidence

The early alias loop still resolves/publishes aliases through the ordinary path.

The later key-projecting field/callable code is currently passed an empty deferred-static-definition set.

Although:

- `TrackingTypeResolver` can demand a known deferred `StaticDefinition`; and
- `query_static_definition_with_record_key_projections` exists,

the actual session flow does not yet:

1. classify only computed-key aliases as deferred;
2. withhold those forms from `TypeAliasTable`, `StaticDefinitionTable`, and resolver;
3. route an exact `QueryKey::StaticDefinition` through an owner executor;
4. prepare current projection facts;
5. publish the alias;
6. refresh resolver/table state;
7. resume the consumer.

### Required correction

This is the first new semantic feature to implement **after owner convergence**.

Do not bolt it into the current early alias loop with another local recursive helper.

---

## F-05 — HIGH: the current “type-member demand service” is too specialized for the architectural problem

The current session can service selected `TypeMemberDeclaration` demands.

T7.0 now also needs at least:

- `StaticConstant`;
- `StaticDefinition`;
- type-member products;
- potentially projection-aware signatures when they are exact prerequisites.

Continuing to add one `service_X_demands` function per query kind will create a manual second dependency engine.

### Required correction

Use a bounded exact-owner dispatcher keyed by existing `QueryKey`.

It should support **only the owner kinds P4 actually needs**. It is not a new general query database.

Example conceptual dispatch:

```rust
match key {
    QueryKey::StaticConstant(id) =>
        ensure_static_constant_owner(...),

    QueryKey::StaticDefinition(id) =>
        ensure_static_definition_owner(...),

    QueryKey::TypeMemberDeclaration(id) =>
        ensure_type_member_owner(...),

    QueryKey::FieldSignature(id) =>
        ensure_field_signature_owner(...),

    QueryKey::CallableSignature(id) =>
        ensure_callable_signature_owner(...),

    _ =>
        ExistingCurrentOrBlockedPolicy
}
```

This preserves the real semantic DB as the cache/dependency authority.

---

## F-06 — HIGH: the old T7.0C “integrate every type-bearing owner” scope is too broad and insufficiently repository-grounded

The amended plan lists a long family:

```text
declaration shell/surface
trait header/surface
field signature
callable signature
type-member declaration
static-case signature
associated-static-domain declaration
constant declared type
...
```

This list became an implementation mandate before proving that each owner can actually contain a computed Record-key `TypeForm` in current source.

### Why this matters

Blanket integration:

- enlarges the owner scheduler;
- creates false prerequisites;
- increases borrow/context complexity;
- expands invalidation;
- makes cycles harder to classify;
- encourages phase-wide orchestration.

### Required correction

The new plan must begin with an **owner matrix audit**:

| Source construct | Can contain a Record type annotation with computed key? | Canonical `QueryKey` | Current formation function | Needs P4 projection? |
|---|---:|---|---|---:|

Only owners with an actual reachable source path belong in T7.0 completion.

Do not generalize owner scheduling because a theoretical semantic product is “type-bearing”.

---

## F-07 — HIGH: `Awaiting` semantics are correct in shape but not yet proven as a transaction boundary

`StaticExpressionCheckOutcome::Awaiting` is a good design.

The remaining invariant is stronger:

> An awaiting attempt is semantically nonexistent except for its exact demand set.

It must contribute no:

- diagnostics;
- product;
- dependency edges on the waiting owner;
- row;
- alias form;
- stale resolver insertion;
- partial checked-expression cache visible as semantic success.

### Required correction

Owner preparation must be transactional:

```text
attempt check
    ↓
Awaiting(demands)
    -> discard transient attempt
    -> service exact owners
    -> rebuild fresh context
    -> retry

Ready/Invalid
    -> finalize owner diagnostics/dependencies/product
```

Add an integration test that deliberately creates a first-attempt diagnostic which disappears after the demanded owner becomes current. The final snapshot must contain only final-attempt diagnostics.

---

## F-08 — HIGH: current cycle machinery has multiple overlapping authorities

Today cycle information can come from:

- the existing static-constant graph/SCC pipeline;
- `StaticConstantDemandDriver.active_constants`;
- semantic DB `Computing` state;
- the amended T7.0 active owner/demand stack;
- existing alias SCC handling;
- existing type signature/member cycle handling.

The plan says not to create a second persistent cycle graph, yet the live implementation still has overlapping mechanisms.

### Required correction

The new plan must assign precise responsibilities:

```text
syntax/static precomputed SCC
    = optional eager diagnostic/order aid only

SemanticDb QueryState::Computing
    = canonical "this exact query is active/current-revision" fact

P4 transient owner stack
    = cause/path reconstruction for deterministic diagnostics

existing alias/type cycle semantics
    = domain-specific diagnostic selection
```

Do not make `active_constants` a second independent recursion authority after introducing an exact owner stack.

For P4 mixed cycles, use one transient owner stack whose frames record the edge cause:

```rust
enum OwnerDemandCause {
    OrdinaryDependency,
    ComputedRecordKey(RecordKeyUseId),
}
```

A cycle containing both:

- a closed `StaticConstant`; and
- a `ComputedRecordKey` cause

gets the static-formation cycle diagnostic.

A pure syntax alias SCC retains the existing alias-cycle diagnostic.

---

## F-09 — HIGH: Record patterns are still Symbol-only end-to-end

This is not merely untested. The live implementation still hardcodes Symbol/name semantics.

Examples:

### Semantic pattern resolution

Current shape:

```text
entry.label()
  -> FieldKey::symbol(...)
```

Computed keys become unrepresented.

`ResolvedRecordFieldPattern` still stores:

```rust
label: Box<str>
```

### Compiler lowering

Record pattern code repeatedly does:

```text
entry.label().unwrap_or("")
Value::symbol(...)
```

Rest exclusion stores string labels.

### Required correction

T7.1–T7.4 remain a real implementation vertical:

```rust
ResolvedRecordFieldPattern {
    key: FieldKey,
    source/provenance: ...,
    child: ...
}
```

The compiler must consume resolved semantic keys, not source labels.

Rest exclusion must compare exact FieldKey identity:

```text
#x does not exclude "x"
1 does not exclude true
```

No source string reconstruction may remain on the canonical Record pattern path.

---

## F-10 — HIGH: dynamic Record literal duplicate checking is quadratic

### Evidence

The private `RecordLiteralBuilderObject` stores:

```rust
entries: Vec<(FieldKey, Value)>
```

and:

```rust
contains_key -> entries.iter().any(...)
```

The VM performs this check:

- for dynamic field keys; and
- for every field copied by Record expansion.

Repeated insertion therefore has a worst-case:

```text
1 + 2 + ... + n = O(n²)
```

duplicate-check path before final `RecordProductShape` formation.

This conflicts with the plan's large-Record hardening objective.

### Required correction

Keep presentation order in the entry vector, but add a private membership index:

```rust
struct RecordLiteralBuilderObject {
    entries: Vec<(FieldKey, Value)>,
    seen: HashSet<FieldKey>, // or BTreeSet<FieldKey>
}
```

This is **not** the forbidden permanent heterogeneous shape lookup cache. It is ephemeral construction state.

Required behavior:

```text
check key
    -> reject duplicate before associated value when language timing requires
append key/value
finalize
    -> canonical shape sort O(n log n)
```

A deterministic `BTreeSet` is acceptable if worst-case `O(log n)` membership is preferred; a `HashSet` is acceptable if the repository's normal hashing policy is suitable because iteration order is not semantic.

---

## F-11 — MEDIUM/HIGH: metadata scope in the old T8 plan is too ambitious for P4

The repository currently does the correct compatibility-floor behavior:

- legacy metadata fields are still string/Symbol-shaped;
- non-Symbol `FieldKey` export fails explicitly;
- non-Symbol anonymous key domains fail explicitly.

That is coherent with the checkpoint architecture, which assigns final durable static metadata/schema work to P8.

The old T8 plan nevertheless asks P4 to introduce a full lossless `FieldKeyRef` staging model in `phalcom-type-meta`.

### Audit ruling

Do **not** make a `phalcom-type-meta` schema migration a prerequisite for P4 completion unless a concrete P4 consumer requires it.

P4 must own:

- deterministic FieldKey encoding/fingerprint semantics;
- no lossy export;
- explicit fail-closed compatibility boundary;
- in-process/runtime overlay preservation where the semantic data already exists.

P8 should own:

- durable heterogeneous metadata schema;
- cross-module `FieldKeyRef` wire representation;
- schema versioning;
- durable static-value/row persistence.

This reduces scope and makes the P4/P8 boundary match the checkpoint architecture.

---

## F-12 — MEDIUM: runtime type inspection remains Symbol-projected when loading legacy metadata

Runtime overlay structures already carry semantic `FieldKey` in places, but base metadata inspection still maps:

```text
RecordFieldRef.name -> FieldKey::Symbol(name)
```

That is correct for **legacy metadata**, because legacy names semantically mean Symbol.

The problem would be pretending that this is heterogeneous metadata support.

### Required correction

P4 should explicitly test and document:

```text
live/in-process heterogeneous row
    -> runtime overlay keeps FieldKey

legacy metadata row
    -> name means FieldKey::Symbol only

heterogeneous durable row export
    -> fail closed until P8
```

Do not add a fake String/Int/Bool reconstruction from legacy text.

---

## F-13 — MEDIUM: `labelAt` is now a partial compatibility API

The source `Record` API has both:

```telos
labelAt(index) -> Symbol
keyAt(index) -> FieldKey
```

The native `rawLabelAt` traps for a valid non-Symbol key.

This is logically understandable as a legacy Symbol-label API, but the new heterogeneous Record semantics make the distinction observable.

### Required correction

P4 must make the contract explicit.

At minimum:

- `keyAt` is the canonical total heterogeneous key observation;
- `labelAt` is explicitly Symbol-only compatibility behavior.

If the public API promises totality, it must be changed. If P9 owns API retirement/renaming, P4 may retain it only with a precise documented partial/precondition contract and tests.

---

# 5. The “baseline blocked” issue must be reclassified

The lifecycle state says P4 is:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

not:

```text
COMPLETE / BASELINE_BLOCKED
```

That distinction matters.

There are known repository-wide baseline failures and at least one historical isolated library-target compilation obstacle. Those do **not** excuse a red P4-owned focused selector.

The replacement plan must use this policy:

```text
Focused P4 acceptance test fails
    -> P4 blocker unless predecessor reproduction proves unrelated

Broad package/workspace gate fails
    -> classify A/B/C/D
    -> baseline-blocked is allowed only after causal comparison

An unrelated target prevents compilation of an isolated unit probe
    -> use an integration/production-path discriminator if possible
    -> record the baseline limitation
    -> do not call the P4 behavior verified unless another test proves it
```

The goal is to stop “baseline blocked” from becoming a catch-all status for incomplete P4 behavior.

---

# 6. Replacement architecture

## 6.1 One exact semantic-owner execution boundary

Introduce a private, bounded owner coordinator.

Mechanical naming is flexible:

```text
SemanticOwnerExecutor
OwnerDemandExecutor
CurrentOwnerScheduler
```

Its responsibility is orchestration, not semantics.

Conceptual interface:

```rust
enum OwnerEnsureOutcome {
    Current,
    DiagnosedInvalid,
    Cycle(OwnerCycle),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(...),
    Failed(...),
}

fn ensure_owner_current(
    &mut self,
    key: QueryKey,
    cause: OwnerDemandCause,
) -> OwnerEnsureOutcome;
```

### It owns

- active exact `QueryKey` stack;
- edge/cause stack;
- current-revision readiness checks;
- dispatch to the existing canonical owner implementation;
- canonical table/resolver refresh after owner publication;
- retry/no-progress detection.

### It does not own

- type checking rules;
- static evaluation;
- Record-key conversion;
- row formation;
- product construction for another owner;
- a second dependency graph;
- persistent cache state.

---

## 6.2 Canonical owner functions

Every P4-relevant owner must have one callable “ensure current” entrypoint.

Example conceptual matrix:

```text
StaticConstant(K)
    ensure_static_constant_current(K)

StaticDefinition(alias)
    ensure_static_definition_current(alias)

TypeMemberDeclaration(M)
    ensure_type_member_current(M)

FieldSignature(F)
    ensure_field_signature_current(F)

CallableSignature(C)
    ensure_callable_signature_current(C)
```

The existing `query_*` database functions may remain lower-level publication/cache functions.

The important rule is:

> **No caller bypasses owner preparation and directly recomputes a projection-aware product.**

This fixes the current replay bug by construction.

---

## 6.3 Projection preparation is owner-relative and transient

Preserve:

```text
RecordKeyUseId = owner QueryKey + deterministic ordinal
ProjectedRecordKeySet
```

The current syntax-address map is acceptable as an ephemeral lookup index inside one AST borrow/attempt. It must not become durable identity.

The owner flow is:

```text
owner source/annotations
    ↓
scan deterministic computed-key sites
    ↓
check site with ordinary checker
    ↓
if Awaiting(exact query keys):
    discard attempt
    ensure exact owners current
    rebuild fresh context
    retry
    ↓
Ready checked expression
    ↓
ensure exact selected constants current
    ↓
P2 static evaluator
    ↓
StaticValueStore::field_key_of
    ↓
Concrete FieldKey
    ↓
canonical TypeStore row construction
```

Only after all required sites are concrete may the owner publish a concrete type/signature.

---

## 6.4 Current-context construction

Do not carry a mutable `CheckingContext` across suspension.

A retry must instantiate a fresh context from current canonical products.

The replacement plan must define a small context factory for static-key checking, e.g.:

```rust
build_static_key_checking_context(module, current semantic tables...)
```

This avoids duplicated context-attachment code currently repeated around the session.

The factory should include only the semantic tables required by the supported P2 static-expression surface.

If a new owner/table is discovered to be required, add an exact demand hook. Do not broad-preflight all signatures/members.

---

## 6.5 Alias deferral

Pre-scan aliases:

```text
ordinary alias, no computed Record key
    -> existing early path

alias TypeForm contains computed Record key
    -> deferred owner set
    -> no alias form published
    -> declaration identity remains known
```

When demanded:

```text
ensure_owner_current(StaticDefinition(alias))
    ↓
ensure alias dependencies/current context
    ↓
prepare Record-key projections
    ↓
resolve canonical StaticDefinition
    ↓
query_static_definition_with_record_key_projections
    ↓
insert TypeAliasTable + StaticDefinitionTable
    ↓
refresh resolver
```

After the main owner work reaches quiescence, all deferred aliases that must be part of the current snapshot must be forced or diagnosed.

No successful snapshot should silently omit an otherwise valid declaration merely because nobody happened to demand it.

---

## 6.6 Cycle semantics

Use the transient owner stack:

```text
Frame {
    key: QueryKey,
    entered_by: OwnerDemandCause,
}
```

On re-entry:

1. extract the exact cycle path;
2. inspect causes and owner kinds;
3. choose established diagnostic category;
4. unwind deterministically;
5. publish only canonical diagnosed-invalid products where the owning query contract requires one;
6. never satisfy a current-revision dependency from LKG.

Do not retain separate long-lived P4 cycle graphs.

---

# 7. Requirements for the new implementation plan

The new plan should be decomposed as follows.

---

## RP0 — Re-baseline and freeze actual owner matrix

### Purpose

Make the new plan repository-truthful before edits.

### Required work

1. record current `main` and active P4 branch revisions;
2. inspect whether the two branch-local T7.0 commits remain unmerged;
3. list every source construct whose type annotation can contain `RecordKeySyntax::Computed`;
4. map each to its canonical query owner;
5. locate every direct call that recomputes one of those owners;
6. locate every `record_key_projections: None` call into a projection-aware query;
7. locate all `StaticConstantProduct` constructors;
8. classify them as canonical formation, test fixture, or duplicate formation;
9. freeze exact focused selectors.

### Acceptance

The plan contains an explicit owner matrix and no speculative “all type-bearing owners” list.

---

## RP1 — Collapse duplicate `StaticConstant` formation

### Purpose

Remove the root architectural duplication before adding alias behavior.

### Required work

Refactor:

```text
session static-constant stage
StaticConstantDemandDriver::compute
StaticConstantDemandDriver::publish_recursive_cycle
```

into one canonical owner path.

### Strong preference

The old `StaticConstantDemandDriver` should either:

- disappear; or
- become a thin consumer of the exact owner executor.

It should not retain `compute()` that forms products.

### Preserve

- P2 evaluator;
- `StaticConstantId`;
- static binding catalog;
- exact static-constant dependencies;
- current fingerprints;
- cancellation/budget semantics;
- closed-const reduction requirement;
- existing static-constant diagnostics.

### Tests

- old P2 static constant focused suite;
- annotated constant with computed-key type;
- direct const dependency;
- same-module isolation;
- imported dependency;
- direct ordinary static-constant cycle;
- computed-key mixed cycle.

### Stop condition

If unifying owner formation requires replacing the semantic DB or static evaluator, stop and re-consult. It should not.

---

## RP2 — Introduce bounded exact-owner orchestration

### Purpose

Provide one re-entry mechanism for current semantic owners.

### Required work

1. create private exact `QueryKey` active stack;
2. add `OwnerDemandCause`;
3. dispatch only required owner kinds;
4. reconcile an already-current DB product into current table/resolver views;
5. invoke canonical owner function when not current;
6. refresh the corresponding canonical view after publication;
7. detect no-progress;
8. preserve query budget/cancellation;
9. eliminate ad hoc type-member-only retry loops where the generic owner executor now applies.

### Forbidden

- new query key for Record-key projection;
- new persistent cache;
- callback-based resolution inside `TypeResolver`;
- phase-wide “make everything current” preflight;
- source-name demand inference.

---

## RP3 — Make projection-aware owners re-entry safe

### Purpose

Fix cold/incremental replay.

### Required work

For every owner from RP0's matrix:

```text
ensure_owner_current(owner)
    must perform projection preparation when source requires it
```

Remove raw replay paths that call the lower-level query with missing projections.

Particularly audit:

```text
ensure_semantic_dependency_current
query_callable_signature_with_inputs(... record_key_projections: None ...)
query_field_signature_with_inputs(... record_key_projections: None ...)
StaticDefinition replay
```

### Acceptance

A changed constant key causes the exact dependent signature to be recomputed through the same owner route as cold formation.

A same-result source edit may recompute the constant but must not unnecessarily invalidate unrelated products when product fingerprint is unchanged.

---

## RP4 — Implement computed-key alias ownership

### Purpose

Close INC-5 correctly after RP1–RP3.

### Required work

1. classify/defer only aliases containing computed Record keys;
2. keep declaration identity visible, but canonical alias form absent;
3. mark them in the tracking resolver;
4. route exact `StaticDefinition` demand through owner executor;
5. project keys;
6. publish through projection-aware static-definition query;
7. update all canonical alias/static-definition/resolver views atomically after success;
8. force remaining snapshot-required deferred aliases before finalization;
9. preserve existing syntax alias SCC semantics;
10. classify mixed alias/constant computed-key cycles through active owner stack.

### Acceptance tests

At minimum:

```telos
const K = #x
type A = { [K]: Int }
```

and:

```telos
const K = SomeStaticComputation(...)
type A = { [K]: Int }
type B = A
```

plus imported variants and mixed cycles.

---

## RP5 — Complete T7.0 owner/cycle/import/incremental matrix

### Purpose

Pass a replacement for T7.0E.

### Required matrix

#### Key values

- Symbol;
- String;
- huge positive/negative Int;
- Bool;
- supported static operations;
- invalid static value kind;
- runtime `let` rejection.

#### Owners

Every RP0 owner that can actually contain the syntax.

At minimum the live proven owners:

- field signature;
- callable signature;
- static constant declared type;
- computed-key alias.

Add type member/static case/etc. only if the owner matrix proves source reachability.

#### Dependency identity

- local constant;
- imported constant with remote canonical `SymbolId`;
- two same-spelled constants in different modules;
- exact dependency edge;
- unrelated constant edit does not invalidate consumer.

#### Incremental

- `#a -> #b`;
- `#x -> "x"`;
- same-result source edit;
- key-dependent duplicate introduced after reduction;
- key-dependent duplicate removed;
- cold/incremental parity.

#### Suspension

- first attempt returns Awaiting;
- exact prerequisite owner executes;
- fresh checker context;
- final attempt succeeds;
- no first-attempt diagnostics;
- no first-attempt product/dependency publication.

#### Cycles

- direct constant/type-key cycle;
- transitive constant/type-key cycle;
- alias ↔ signature/member cycle;
- alias → constant → alias;
- imported transitive mixed cycle where supported;
- no hang;
- no stale LKG;
- deterministic diagnostic.

#### Symbolic/open case

If a genuinely open generic key is legal to defer to later C4 work, prove that:

- no `FieldKey` is fabricated;
- no `RecordRowField` is fabricated;
- no AST placeholder becomes semantic identity.

If P4's closed owner requires a concrete key, diagnose instead of silently residualizing.

### Gate

T7.1 remains blocked until this matrix passes.

---

## RP6 — Finish heterogeneous Record patterns

### Semantic requirements

Change canonical resolved pattern representation to FieldKey.

Use shared static-key projection under the `CallableBody` owner.

Record exact constant dependencies on the body, not on the callable signature merely because a pattern body changed.

Use `UnorderedFieldLookup` for expected field typing.

### Runtime/compiler requirements

Remove Symbol-only lowering from canonical Record pattern paths.

Materialize or compare the resolved FieldKey accurately.

Rest reconstruction excludes exact heterogeneous keys.

### Required tests

```text
#x
"x"
42
true
same-text Symbol/String
1 vs true
missing key
optional field
typed open tail
rest exclusion
nested pattern
invalid/non-static key
const-backed pattern key
incremental const-backed pattern key
```

Compiler test scaffolding should prove lowering consumes resolved semantic key facts rather than re-reading source labels.

---

## RP7 — P4 durability/performance hardening, with narrowed metadata scope

### 7.1 Metadata

P4 completion requires:

- legacy Symbol-only export remains exact;
- non-Symbol export fails closed;
- non-Symbol key-domain export fails closed;
- no stringification/coercion;
- stable FieldKey tagged fingerprinting exists;
- runtime/in-process overlays do not erase key kind.

Do not require a final heterogeneous wire-schema migration.

If a lossless internal `FieldKeyRef` is useful, it may be added in a lower internal layer, but P4 should not force P8's schema publication.

### 7.2 Incrementality

Prove:

```text
#x -> "x" changes identity
source order only does not change canonical row identity
value type change invalidates
tail key-domain change invalidates
unrelated edit does not invalidate row
Record key edit does not invalidate P3 Pack shape/product
```

### 7.3 Dynamic builder performance

Fix `RecordLiteralBuilderObject::contains_key` quadratic behavior.

Expected complexity:

```text
dynamic duplicate membership  O(1) average or O(log n)
final shape formation          O(n log n)
lookup                         O(log n)
equality/hash                  O(n)
row merge                      O(n + m)
```

### 7.4 Large cases

Use structural tests at 1k and 10k mixed keys.

No fragile wall-clock threshold is required.

An operation-count or architectural assertion is preferable.

### 7.5 Public runtime Record compatibility

Verify `keyAt`.

Explicitly classify/document `labelAt` as Symbol-only compatibility behavior or alter it if the source contract requires total heterogeneous observation.

---

## RP8 — Stabilization and lifecycle closure

### Required gates

#### G1 — semantic FieldKey/row regression

- FieldKey;
- rows;
- tails;
- solver;
- relation;
- unordered lookup;
- existing optionality laws.

#### G2 — static/runtime Record vertical

- static Record;
- runtime construction;
- computed keys;
- spreads;
- duplicate timing;
- lookup;
- equality/hash;
- empty Record.

#### G3 — static-key owner/replay gate

- RP5 full matrix;
- cold/incremental parity;
- exact dependencies;
- aliases;
- imported forcing;
- cycles;
- no-leak retry.

#### G4 — patterns/boundary

- heterogeneous Record patterns;
- rest;
- Symbol-only call/Pack/Tuple boundary;
- relevant P3 Pack selectors.

#### G5 — durability/performance

- fail-closed metadata;
- runtime overlays;
- incremental fingerprints;
- 1k/10k structural resource tests;
- affected-crate checks.

### Lifecycle requirements

Only after all P4-owned gates pass:

```text
P4.status       = COMPLETE
P4.completion   = IMPLEMENTED
P4.verification = FOCUSED_TESTED
```

or the repository's exact lifecycle equivalent.

Do not mark P4 `BASELINE_BLOCKED` unless only broader non-P4 gates remain and every P4-owned focused obligation is green.

Create final walkthrough and handoff from actual executed evidence.

---

# 8. Required deletions or deprecations from the current implementation

The new plan should be explicit about code that should probably disappear rather than merely be extended.

Candidates:

1. **`StaticConstantDemandDriver::compute`**
   - remove or reduce to delegation to canonical constant owner.

2. **`StaticConstantDemandDriver::publish_recursive_cycle`**
   - cycle publication should belong to the canonical constant owner/cycle path.

3. **specialized session loops that manually service only type-member demands**
   - replace with exact owner executor where applicable.

4. **direct projection-aware signature replay with `record_key_projections: None`**
   - forbidden after owner convergence.

5. **empty `no_deferred_static_definitions` production plumbing**
   - replace with actual computed-key deferred alias set or remove.

6. **Symbol/string canonical Record pattern fields**
   - migrate to `FieldKey`.

7. **linear Record-literal builder duplicate lookup**
   - replace with indexed membership.

The implementation plan should include a deletion audit, not only additive tasks.

---

# 9. Invariants the replacement plan must preserve

```text
R-P4-01  FieldKey remains exactly Symbol | String | Int | Bool.

R-P4-02  Symbol/String equality and order are content based and tag-distinct.

R-P4-03  Int keys remain exact arbitrary precision.

R-P4-04  Record rows remain owned by the existing row store/solver.

R-P4-05  Row canonicalization and duplicate rejection remain centralized.

R-P4-06  Record source/presentation order is not structural identity.

R-P4-07  Tuple/Pack/call labels remain Symbol-only.

R-P4-08  Non-Symbol Record keys are never coerced to ordered labels.

R-P4-09  P2 StaticEvaluator remains the static execution authority.

R-P4-10  Ordinary checker selection remains the name/binding/dispatch authority.

R-P4-11  A concrete computed key is produced only by checked static evaluation
          followed by StaticValueStore::field_key_of.

R-P4-12  No unresolved/symbolic expression becomes a canonical FieldKey.

R-P4-13  Each QueryKey semantic owner has one executable formation path.

R-P4-14  Owner orchestration does not become a second semantic database.

R-P4-15  Awaiting attempts publish no semantic facts or diagnostics.

R-P4-16  Every retry uses current-revision products and a fresh checker context.

R-P4-17  Exact dependencies are recorded on the owning semantic product.

R-P4-18  No current work consumes an invalidated previous/LKG product as truth.

R-P4-19  Active cross-owner cycles terminate deterministically.

R-P4-20  Pattern lowering consumes resolved FieldKey, not source spelling.

R-P4-21  Metadata export is lossless or fail-closed; never coerced.

R-P4-22  P8 retains ownership of final durable heterogeneous metadata schema.

R-P4-23  Runtime fixed Record lookup remains O(log n) or better.

R-P4-24  Dynamic Record construction does not have O(n²) duplicate membership.

R-P4-25  Record equality/hash remain O(n) after shape formation.

R-P4-26  Existing P3 Pack representation and fixed-call invariants remain intact.
```

---

# 10. Verification strategy

## 10.1 Build-mode rule

Run the narrowest discriminator for each architectural correction.

Do not use broad package failures to debug a local owner issue.

## 10.2 Architecture-first tests

Before adding new feature coverage, create regressions that fail specifically under the current bad architecture:

1. projection-aware `FieldSignature` dependency replay through the generic revalidation path;
2. projection-aware `CallableSignature` replay;
3. demanded constant formed through the same fingerprint/dependency path as eager constant formation;
4. deferred alias demanded by a signature;
5. first-attempt Awaiting diagnostics do not leak;
6. transitive mixed cycle does not hang;
7. imported constant demanded from a computed type key.

These tests will prove the owner refactor, rather than merely proving happy-path language semantics.

## 10.3 Pattern tests after T7.0

Do not combine pattern migration with owner-scheduler repair.

The old gate is correct: T7.0 first, patterns second.

## 10.4 Baseline classification

For every unexpected failure:

```text
A: definitely P4
B: probably P4
C: unclear
D: proven baseline/unrelated
```

A/B block.

C gets one bounded causal classification.

D is recorded and ignored for P4 completion.

---

# 11. Scope reductions relative to the old plan

A new plan should deliberately remove these old-plan tendencies.

## 11.1 Do not implement a universal semantic scheduler

Only dispatch exact query owners needed by P4 computed-key formation.

The semantic DB already is the general query system.

## 11.2 Do not integrate hypothetical type owners

Use the RP0 owner matrix.

## 11.3 Do not require final `phalcom-type-meta` heterogeneous schema

P8 owns durable schema.

P4 requires fail-closed safety and stable semantic fingerprints.

## 11.4 Do not reopen settled T1–T6 architecture

Re-run focused regression gates, but do not redesign:

- FieldKey;
- row store;
- row solver;
- runtime shape;
- P3 boundary.

## 11.5 Do not treat every baseline failure as active P4 work

P4 must finish its focused vertical first.

---

# 12. Scope additions the old plan understated

The replacement plan must explicitly add:

1. **single executable owner path** as a first-class invariant;
2. **revalidation/replay through owner preparation**, not raw query calls;
3. **deletion of duplicate constant formation**;
4. **dynamic builder membership complexity fix**;
5. **transactional Awaiting/no-leak semantics**;
6. **explicit final forcing of snapshot-required deferred aliases**;
7. **legacy `labelAt` compatibility contract audit**;
8. **owner matrix audit** to bound the scheduler.

These are more important than several of the old plan's broad generic owner tasks.

---

# 13. Proposed ordering for the brand-new implementation plan

The recommended dependency order is:

```text
RP0  Rebaseline + exact owner matrix
 │
 ▼
RP1  Unify StaticConstant owner formation
 │
 ▼
RP2  Exact owner executor / active owner stack
 │
 ▼
RP3  Projection-aware owner replay/revalidation
 │
 ▼
RP4  Deferred computed-key alias owner
 │
 ▼
RP5  Full T7.0E replacement gate
 │
 ▼
RP6  Heterogeneous Record patterns
 │
 ▼
RP7  Incremental + metadata boundary + performance hardening
 │
 ▼
RP8  P4 stabilization and lifecycle closure
```

This order is intentionally different from “continue T7.0D where left off.”

The owner-boundary repair comes first.

---

# 14. Completion definition

P4 is ready to complete only if all of the following are true.

## Semantic core

- one FieldKey model;
- one row authority;
- one unordered lookup semantic;
- all four key kinds behave correctly;
- typed tail key domains behave correctly.

## Static key formation

- ordinary checker owns binding/semantic selection;
- P2 evaluator owns execution;
- StaticValueStore owns reduced-value-to-FieldKey conversion;
- exact dependencies are recorded;
- owner recomputation is cold/incremental equivalent;
- no duplicate semantic owner formation path remains.

## Owners

- all genuinely reachable computed-key type owners are supported;
- computed-key aliases work;
- imported constants work;
- transitive cycles terminate;
- no Awaiting leakage.

## Runtime

- construction/access/spread/equality/hash work for all key kinds;
- ordered-label boundary remains Symbol-only;
- dynamic construction duplicate checks are not quadratic.

## Patterns

- resolved pattern products carry FieldKey;
- lowering consumes resolved FieldKey;
- rest excludes exact keys.

## Durability boundary

- legacy metadata remains exact for Symbol;
- heterogeneous metadata does not silently degrade;
- final schema remains explicitly deferred to P8 unless separately ratified.

## Verification/lifecycle

- full P4 focused gates have actual executed evidence;
- every P4-owned failure is resolved;
- unrelated baselines are separately classified;
- final walkthrough/handoff reflect reality;
- P5 is unblocked only according to the checkpoint's P3 + P4 dependency state.

---

# 15. Principal implementation guidance

The next implementation plan should be written as a **repair plan against the live repository**, not an amendment to the 3,800-line historical P4 plan.

Its central architectural statement should be:

> A computed Record key is not a new semantic subsystem. It is a forcing edge between existing semantic owners. Existing owners must therefore become safely re-enterable through one exact owner-execution boundary. The forcing mechanism schedules those owners; it does not reimplement them.

That principle resolves the main failure mode visible in the current work.

The current work should be treated as valuable substrate, not discarded wholesale. In particular:

```text
KEEP:
  FieldKey
  canonical rows/tails
  row solver/relation
  static Record key bridge
  runtime Record shape
  checked-static-expression representation
  owner-relative key IDs
  ProjectedRecordKeySet concept
  TrackingTypeResolver demand capture
  StaticExpressionCheckOutcome
  exact semantic dependency representation

REFACTOR:
  StaticConstantDemandDriver
  session demand routing
  owner replay/revalidation
  deferred alias flow
  cycle authority

FINISH:
  T7.0E
  Record patterns
  runtime builder performance
  incremental/performance gates
  lifecycle closure

DEFER:
  final heterogeneous durable metadata schema -> P8
  public Product/UnorderedProduct reflection -> P5
  broad ontology/tooling retirement -> P9
```

That is the architecture on which a new Luna patch-grade plan should be based.

---

# 16. Recommended immediate next action

Do **not** implement INC-5 directly on top of the current `StaticConstantDemandDriver`.

The next coding slice should first prove and implement:

```text
one StaticConstant owner formation path
+
one exact owner re-entry dispatcher
+
projection-aware signature replay through that dispatcher
```

Then add the alias owner.

That sequence addresses the root cause rather than extending the current workaround architecture.
