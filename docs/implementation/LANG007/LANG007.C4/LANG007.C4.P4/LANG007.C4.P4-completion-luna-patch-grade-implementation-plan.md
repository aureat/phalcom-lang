---
id: LANG007.C4.P4
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
title: "Corrective Completion of Generalized FieldKey and Unordered-Product Calculus"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
repository: aureat/phalcom-lang
planning_branch: fix/LANG007-a1-match-equality-authority
prepared_against_revision: 4f11cb1a900d76c1d45db108aa8d856ee5940394
main_revision_observed: ee7e8ee725f90d71ba4a96f907f27339abd989a0
prepared: 2026-10-06
depends_on:
  - LANG007.C4.P3
blocks:
  - LANG007.C4.P5
---

# LANG007.C4.P4 — Corrective Completion Plan

## Generalized `FieldKey` and Unordered-Product Calculus

This document is a **replacement execution plan for the remaining scope of `LANG007.C4.P4`**. It does not redefine the accepted `FieldKey`, Record-row, static-evaluation, or ordered-product semantics. It corrects the remaining implementation direction after auditing the partially implemented T7 owner/demand work against the live repository.

> **Central correction:** do not continue by adding more demand-specific scheduler code on top of the current `StaticConstantDemandDriver`. First converge every relevant semantic owner to one executable formation path, introduce one bounded exact-owner orchestration seam, and make every projection-aware owner use that seam for cold formation and replay. Only then implement computed-key aliases and pass T7.0E.

The plan preserves the completed T1–T6 substrate and completes P4 through semantic-owner convergence, computed-key alias ownership, T7.0E, heterogeneous Record patterns, Record-construction performance hardening, compatibility/metadata closure, and final stabilization.

---

# 0. Executor contract

The implementer:

- **must** re-read the named live files before editing;
- **must** preserve the language-level semantics and ownership invariants below;
- **may** adapt private helper names, file decomposition, lifetimes, borrowing strategies, and mechanically equivalent APIs;
- **must not** preserve a private abstraction merely because it exists if it violates one-owner/one-formation-path semantics;
- **must not** create a second row solver, static evaluator, Record-key semantics, dependency database, or pattern-key evaluator;
- **must not** generalize Tuple, Pack, call, selector, or ordered-product labels from `Symbol` to `FieldKey`;
- **must not** use LKG values, provisional rows, source-order speculation, or symbolic placeholders as current truth for a closed computed-key Record type;
- **must** treat `Awaiting(exact demands)` as scheduler control, not a semantic result;
- **must** keep the C4 checkpoint truthful at durable gates;
- **must** preserve unrelated branch/worktree changes, including unrelated package work observed on the audited branch;
- **must** use sparse discriminating tests during BUILD, focused suites at named stabilization gates, and broader checks only at final certification;
- **must** STOP AND CONSULT if fixed ownership, canonicalization, evaluation, incremental, or cross-layer contracts would need to change.

Testing is evidence gathering. Do not run broad suites reflexively after each edit.

---

# 1. Goal

Complete `LANG007.C4.P4` by establishing a single coherent semantic-owner architecture for computed Record keys, proving cold/incremental equivalence and exact dependency ownership, then carrying canonical heterogeneous `FieldKey` values through Record patterns and runtime construction without regressing the established Record row/runtime model or the P3 Symbol-only ordered-product boundary.

The completed authority flow must be:

```text
source Record key syntax
    ↓
ordinary checker selects bindings/operations/dependencies
    ↓
P2 CheckedStaticExpression
    ↓
exact semantic-owner demand orchestration
    ↓
P2 StaticEvaluator
    ↓
StaticValueStore::field_key_of
    ↓
concrete FieldKey
    ↓
existing TypeStore / RecordRow canonicalization
    ↓
semantic owner product + exact dependency edges
    ↓
compiler lowering consumes semantic facts
    ↓
runtime Record shape/storage/access/pattern behavior
```

The same semantic fact must be observed in cold analysis, incremental replay, downstream body analysis, lowering, runtime behavior, and diagnostics.

---

# 2. Repository grounding and current truth

Planning was performed against:

```text
repository: aureat/phalcom-lang
branch:     fix/LANG007-a1-match-equality-authority
revision:   4f11cb1a900d76c1d45db108aa8d856ee5940394
main seen:  ee7e8ee725f90d71ba4a96f907f27339abd989a0
date:       2026-10-06
```

At audit time the branch was six commits ahead of `main` and zero behind. It contains P4 work plus unrelated changes. P4 execution must not stage, rewrite, or “clean up” unrelated work.

The live C4 checkpoint records:

```text
LANG007.C4.P4
status:       IN_PROGRESS
completion:   PARTIAL
verification: UNVERIFIED
```

T1–T6 are recorded complete/implemented/focused-tested. T7.0A–B and portions of T7.0C–D exist. Computed-key aliases, complete owner/replay convergence, T7.0E, patterns, T8 closure, and T9 stabilization remain incomplete.

The existing interim handoff correctly states that the projection-aware static-definition helper and deferred-definition resolver hook are **groundwork**, not completed alias scheduling.

> Re-read the live paths before editing. Mechanical drift is expected. Architectural drift is an escalation condition.

---

# 3. Audit findings that determine the implementation order

## 3.1 F-01 — duplicate executable `StaticConstant` formation

There are currently **two substantial executable paths** capable of forming and publishing `QueryKey::StaticConstant` products.

### Session-owned eager path

`phalcom-semantic/src/session.rs` contains:

```text
ConstantDeclInfo
constant_declarations
checked_constants
constant_graph
constant_record_key_dependencies
ConstantTarjan
visit_constant
cycle product construction
computed-key projection
initializer re-check
StaticEvaluator execution
StaticConstantProduct construction
query_static_constant(...)
StaticConstantTable insertion
```

### Demand-owned path

`phalcom-semantic/src/static_constant_demand.rs` contains:

```rust
StaticConstantDemandDriver::force
StaticConstantDemandDriver::compute
StaticConstantDemandDriver::compute_with_projected_keys
StaticConstantDemandDriver::check_evaluate_publish
StaticConstantDemandDriver::recursive_cycle_outcome
StaticConstantDemandDriver::publish_recursive_cycle
```

This path independently discovers/checks reads, recursively forces constants, projects annotation keys, resolves expected types, rechecks initializers, evaluates, constructs products, publishes, mutates the constant table, and tracks `active_constants`.

This is not two callers of one owner. It is two semantic-formation implementations.

**Required correction:** `QueryKey::StaticConstant(K)` gets one canonical formation function. Eager session work and demand-driven work both call it.

## 3.2 F-02 — projection-aware signature replay bypass

`phalcom-semantic/src/db/query.rs` defines:

```rust
pub struct SignatureQueryInputs<'a> {
    ...
    pub record_key_projections:
        Option<&'a crate::static_key::ProjectedRecordKeySet<'a>>,
    ...
}
```

Cold/session formation prepares projections for field and callable signatures.

But `ensure_formal_semantic_dependency_current(...)` re-enters:

```rust
query_callable_signature_with_inputs(...)
query_field_signature_with_inputs(...)
```

with:

```rust
record_key_projections: None
```

for formal replay. Other wrapper paths also default to `None`.

This allows:

```text
cold formation:    projected
incremental replay: potentially unprojected
```

for the same `QueryKey`.

**Required correction:** every projection-aware owner has one preparation/re-entry path used by cold formation and replay.

## 3.3 F-03 — owner-specific demand servicing is becoming a second scheduler architecture

`session.rs::service_type_member_demands(...)` already implements a mini scheduler for one owner kind: reconcile ready DB product, inspect prerequisites, publish through `publish_type_member_owner`, refresh resolver, detect no-progress/cycle.

Adding `service_static_definition_demands`, `service_signature_demands`, etc. would replicate this machinery.

**Required correction:** replace owner-specific demand loops with one **bounded exact-owner executor** keyed by the existing `QueryKey` variants P4 actually needs. `SemanticDb` remains the persistent currentness/dependency/cache authority.

## 3.4 F-04 — computed-key aliases are scaffolding only

The ordinary alias/static-definition phase still calls:

```text
resolve_static_definition
query_static_definition
resolver.insert_alias_form
type_aliases.insert
static_definitions_table.insert
```

before the later T7 checking environment exists.

Useful groundwork already exists:

```rust
TrackingTypeResolver::track_deferred_static_definition
TrackingTypeResolver::semantic_query_demands_snapshot
query_static_definition_with_record_key_projections
ProjectedRecordKeyResolver
```

but the production flow still supplies an empty deferred-definition set in relevant paths.

**Required correction:** defer only aliases whose TypeForm contains computed Record keys. Keep them absent from `TypeAliasTable`, `StaticDefinitionTable`, and canonical resolver state until their `StaticDefinition` owner forms and publishes.

## 3.5 F-05 — suspension semantics exist, but must become transactional

`checker/static_body.rs` already has:

```rust
enum StaticExpressionCheckOutcome {
    Ready(...),
    Awaiting(BTreeSet<QueryKey>),
    Invalid(...),
}
```

and `check_static_expression` returns `Awaiting` whenever exact semantic demands were collected.

This is the right seam, but the architecture must guarantee:

> An Awaiting owner attempt is semantically nonexistent except for its exact demand set and transient scheduler trace.

No diagnostics, owner product, dependency edges, resolver alias form, or owner-table insertion from that attempt may survive.

## 3.6 F-06 — cycle authority currently overlaps

Current/recent logic contains several potential authorities:

- eager static-constant SCC/Tarjan graph;
- `StaticConstantDemandDriver::active_constants`;
- `SemanticDb` query state;
- intended transient cross-owner stack;
- alias SCC logic;
- existing signature/type-member cycle logic.

These must not all decide semantic currentness independently.

**Required division:**

```text
SemanticDb             = persistent current-revision query state/dependencies
transient owner stack  = active path + causal explanation
existing domain logic  = diagnostic classification
syntax SCCs            = syntax/domain-specific diagnostics where already canonical
```

The unified owner stack replaces `active_constants` as cross-owner runtime cycle trace.

## 3.7 F-07 — Record-pattern syntax is generalized, semantics are still Symbol-only

AST support already exists:

```rust
enum RecordKeySyntax {
    Symbol { ... },
    Computed { expr, ... },
}

struct RecordPatternEntry {
    key: RecordKeySyntax,
    ...
}
```

But:

- `checker/pattern.rs::resolve_record_pattern` uses `entry.label()` + `FieldKey::symbol`;
- rest subtraction uses Symbol labels;
- `match_semantics.rs::ResolvedRecordFieldPattern` stores `label: Box<str>`;
- `compiler/lib/patterns.rs` emits `Value::symbol`;
- `emit_record_rest_temp` accepts `&[String]` and uses `rawLabelAt`.

T7.1–T7.5 are therefore real cross-layer implementation, not only tests.

## 3.8 F-08 — dynamic Record literal duplicate checking is O(n²)

`phalcom-core/src/heap/record_literal_builder.rs` currently stores:

```rust
entries: Vec<(FieldKey, Value)>
```

and:

```rust
contains_key -> entries.iter().any(...)
```

Repeated insertions/spreads can perform quadratic duplicate checks.

**Required correction:** add a transient `HashSet<FieldKey>` or `BTreeSet<FieldKey>` membership index while preserving `entries` for presentation order.

## 3.9 F-09 — old P4 metadata scope is stale

The current exporter correctly fails closed using:

```rust
MetadataExportError::NonExportableRecordKey(FieldKeyKind)
MetadataExportError::NonExportableRecordKeyDomain
```

and `legacy_record_field_name` only exports `FieldKey::Symbol`.

`typing/inspect.rs` correctly interprets legacy metadata names as Symbol keys.

The old requirement for P4 to publish the final durable heterogeneous metadata schema overlaps P8.

**Correct P4 boundary:**

- preserve stable tagged `FieldKey` fingerprint semantics;
- preserve lossless in-process behavior;
- legacy metadata remains Symbol-only;
- heterogeneous legacy export fails closed;
- no stringification/coercion/truncation.

P8 owns final durable heterogeneous schema/version unless a concrete current consumer proves otherwise.

---

# 4. Normative authority

## 4.1 `FieldKey`

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

Canonical cross-kind order:

```text
Symbol < String < Int < Bool
```

Within-kind order:

```text
Symbol  exact content
String  exact content
Int     exact mathematical integer order
Bool    false < true
```

Consequences:

```text
#x != "x"
1  != true
0  != false
```

No Unicode normalization, locale comparison, VM symbol numeric ID, source range, AST pointer, semantic arena address, or heap identity participates in key identity.

## 4.2 Record structural semantics

From the normative structural-product and VM-product specifications:

- Record structural identity is the canonical `FieldKey` set plus associated field types/values;
- source/presentation order does not define structural identity;
- canonical rows use the language-defined `FieldKey` order;
- duplicate keys are invalid, including collisions exposed after static reduction/specialization;
- optionality is member presence, not `Option<T>`;
- anonymous tails retain key/value-domain information;
- captured row tails preserve correlation;
- runtime shapes may retain presentation coordinates separately from canonical logical coordinates.

## 4.3 Ordered-product boundary

Tuple, Pack, call, selector, and ordered labeled lanes remain `Symbol`-only:

```text
unordered Record keys:
    Symbol | String | Int | Bool

ordered labeled lanes:
    Symbol
```

Non-Symbol Record keys must never silently cross that boundary.

## 4.4 Static computation authority

Computed keys use:

```text
ordinary checker
    ↓
CheckedStaticExpression
    ↓
existing P2 StaticEvaluator
    ↓
StaticValueRef
    ↓
StaticValueStore::field_key_of
    ↓
FieldKey
```

`TypeResolver`, row formation, pattern lowering, and compiler lowering must not independently evaluate or resolve the key expression.

## 4.5 Type-system authority

These remain canonical:

```text
TypeStore
RecordRowData
RecordRowSolver
type relation engine
UnorderedFieldLookup
```

P4 completes consumers around them; it does not replace them.

---

# 5. Established substrate to preserve

| Concept | Live symbol/path | Treatment |
|---|---|---|
| key domain/order | `field_key.rs::{FieldKey, FieldKeyKind, canonical_cmp, stable_fingerprint}` | preserve |
| row field key | `types/row.rs::RecordRowField` | preserve |
| canonical row | `types/row.rs::RecordRowData` | preserve |
| tail domains | `types/row.rs::{RecordRowTail, AnonymousRecordRemainder}` | preserve |
| row solver | `types/row_solver.rs::RecordRowSolver` | preserve as sole solver |
| key-domain membership | `types/relation.rs::check_field_key_domain_membership` | preserve |
| unordered lookup | `types/relation.rs::{UnorderedFieldLookup, lookup_unordered_field}` | reuse |
| static Record | `static_value/store.rs::{StaticRecordField, intern_record}` | preserve |
| static value → key | `StaticValueStore::field_key_of` | mandatory conversion authority |
| key use identity | `static_key.rs::RecordKeyUseId` | preserve |
| projection facts | `static_key.rs::ProjectedRecordKeySet` | preserve |
| projection | `static_key.rs::project_checked_static_key` | preserve |
| suspension result | `checker/static_body.rs::StaticExpressionCheckOutcome` | preserve/strengthen |
| runtime shape | `core/product/shape.rs::RecordProductShape` | preserve |
| runtime conversions | `core/product/mod.rs::{value_to_field_key, field_key_to_value, compare_field_key_to_value}` | preserve |
| Record guard | `compiler/lib/expr.rs::GuardFieldKey` | preserve |
| ordered label guard | `compiler/lib/expr.rs::GuardSymbol` | preserve |
| metadata fail-closed | `semantic/metadata/export.rs` errors + legacy helper | preserve |
| legacy metadata read | `core/typing/inspect.rs` name → `FieldKey::symbol` | preserve |
| total key runtime API | `record.ph::{rawKeyAt,keyAt,rawLookup}` | preserve |
| Symbol compatibility API | `record.ph::{rawLabelAt,labelAt}` | document/test |

---

# 6. Corrected architecture

## 6.1 One owner, one executable formation path

For each relevant stable query:

```text
StaticConstant(K)
StaticDefinition(A)
FieldSignature(F)
CallableSignature(C)
TypeMemberDeclaration(M)
```

there is exactly one executable semantic formation contract.

Different clients can request currentness; none may reconstruct a parallel owner implementation.

```text
caller A ─┐
caller B ─┼─> ensure_owner_current(QueryKey)
caller C ─┘          │
                     ▼
             canonical owner formation
                     │
                     ▼
             canonical owner publication
```

## 6.2 Bounded exact-owner executor

Introduce a private session-owned orchestration abstraction. Private naming is flexible:

```rust
SemanticOwnerExecutor
OwnerDemandExecutor
ExactOwnerScheduler
```

Conceptual model:

```rust
enum OwnerEnsureOutcome {
    Current,
    DiagnosedInvalid,
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(...),
    Failed(String),
}

enum OwnerDemandCause {
    Eager,
    OrdinaryDependency,
    StaticExpressionRead,
    ComputedRecordKey(RecordKeyUseId),
}

struct OwnerFrame {
    key: QueryKey,
    cause: OwnerDemandCause,
}
```

The executor owns:

- exact `QueryKey` currentness checks;
- transient active owner stack;
- causal edge/frame information;
- dispatch to canonical owner formation;
- ready-product reconciliation into session tables/resolver views;
- servicing exact `Awaiting` demands;
- fresh retry;
- budget/cancellation propagation;
- deterministic no-progress classification.

It does **not** own:

- name/type selection;
- static evaluation;
- row canonicalization;
- relation logic;
- FieldKey conversion;
- another owner's product construction;
- a second persistent cache/dependency graph/database.

`SemanticDb` remains the authority for current revision, ready/computing state, dependency fingerprints, invalidation, and cached products.

## 6.3 Transactional suspension

Conceptually:

```text
attempt owner
    ├─ complete -> commit product/diagnostics/dependencies
    └─ Awaiting(Qs)
         -> discard attempt-local semantic effects
         -> ensure Qs current
         -> rebuild fresh context
         -> retry from source/current world
```

Only the exact demand set and transient scheduler trace survive `Awaiting`.

No owner diagnostic, product, dependency edge, alias form, row publication, resolver insertion, or owner-table insertion from that attempt may survive.

Immutable `TypeStore` interning may leave unreachable canonical nodes; no abandoned `TypeId` may become current owner truth.

## 6.4 Cycle authority

Use:

```text
SemanticDb query state   = persistent currentness
owner active stack       = transient causal path
domain cycle logic       = diagnostic class
```

Do not retain `active_constants` as an independent semantic cycle authority after convergence.

Do not retain eager constant Tarjan/SCC as a second publication authority. If any eager graph remains, it may only be a non-authoritative ordering/diagnostic optimization that cannot disagree with exact owner execution.

## 6.5 Projection is owner preparation

Computed-key owner algorithm:

```text
1. stable owner QueryKey
2. deterministic computed-key site enumeration
3. ordinary-check each key expression
4. if Awaiting: suspend owner
5. ensure selected constants current
6. P2 evaluate
7. StaticValueStore::field_key_of
8. retain Concrete / DeferredToP7 / Invalid
9. ProjectedRecordKeyResolver
10. normal mutable TypeStore formation
11. canonical row duplicate detection
12. build owner product
13. publish owner + exact dependencies
```

No row exists before required concrete keys exist.

`DeferredToP7` is not a key and must not enter `FieldKey`, `RecordRowData`, lowering, or source AST placeholders.

## 6.6 Replay uses identical preparation

Forbidden:

```text
cold:   prepare projections
replay: call query with None
```

Required:

```text
dependency needs owner current
        ↓
exact owner executor
        ↓
same projection preparation
        ↓
same owner query/publication
```

## 6.7 Alias ownership

```text
ordinary alias
    -> existing eager StaticDefinition path

computed-key alias
    -> deferred owner registration
    -> absent from canonical alias/static-definition/resolver views
    -> exact QueryKey::StaticDefinition formation
    -> publish
    -> canonical table/resolver insertion
```

## 6.8 Pattern ownership

Pattern-key dependencies belong to `QueryKey::CallableBody(callable)`, not the signature:

```text
CallableBody
    ↓
ordinary checker sees Record pattern
    ↓
shared static-key checker/evaluator
    ↓
ResolvedRecordFieldPattern { key: FieldKey, ... }
    ↓
body records exact dependencies
    ↓
compiler consumes resolved FieldKey
```

A body pattern edit must not invalidate a callable signature unless the signature itself changed.

---

# 7. Global invariants

## FieldKey / row

```text
P4R-INV-01  FieldKey is exactly Symbol | String | Int | Bool.
P4R-INV-02  Key identity is content/value based, never pointer/range/VM-ID based.
P4R-INV-03  Kind order is Symbol < String < Int < Bool.
P4R-INV-04  Symbol("x") != String("x").
P4R-INV-05  Int(1) != Bool(true); Int(0) != Bool(false).
P4R-INV-06  Int keys retain arbitrary-precision identity/order.
P4R-INV-07  Existing TypeStore row construction owns canonicalization/duplicates.
P4R-INV-08  Presentation/source order is not Record structural identity.
P4R-INV-09  Existing required/optional/open/captured-row semantics are preserved.
P4R-INV-10  Existing RecordRowSolver/relation engine remain sole authorities.
```

## Ordered-product isolation

```text
P4R-INV-11  Tuple labels remain Symbol-only.
P4R-INV-12  Pack labels remain Symbol-only.
P4R-INV-13  Call labels remain Symbol-only.
P4R-INV-14  Non-Symbol Record keys are never coerced into ordered labels.
P4R-INV-15  P3 Pack identity/rest/spread behavior is not redesigned.
```

## Static projection

```text
P4R-INV-16  Ordinary checker binding selection is authoritative.
P4R-INV-17  P2 StaticEvaluator is the sole P4 static execution authority.
P4R-INV-18  StaticValueStore::field_key_of is the final key conversion authority.
P4R-INV-19  Computed canonical key requires checked expression + concrete reduction.
P4R-INV-20  DeferredToP7 never becomes FieldKey/RecordRow/AST placeholder.
P4R-INV-21  Publishing owner records every exact selected static-constant dependency.
P4R-INV-22  Imported constants preserve remote SymbolId -> StaticConstantId identity.
```

## Owner / incremental

```text
P4R-INV-23  Each relevant QueryKey has one executable formation path.
P4R-INV-24  Executor schedules/reconciles; it is not another semantic database.
P4R-INV-25  Awaiting publishes no product/diagnostic/dependency/resolver/table result.
P4R-INV-26  Resumption rebuilds a fresh context from current products.
P4R-INV-27  Cold and replay use the same projection-preparation contract.
P4R-INV-28  Current closed formation never consumes stale/LKG value as truth.
P4R-INV-29  Exact invalidation republishes affected owners and preserves unrelated reuse.
P4R-INV-30  Mixed cycles terminate diagnostically; no fixed point/provisional key/row.
```

## Pattern/runtime

```text
P4R-INV-31  Resolved Record pattern fields carry FieldKey, not string label identity.
P4R-INV-32  Pattern expected-field typing uses existing unordered lookup/relation.
P4R-INV-33  Compiler consumes resolved semantic FieldKey, not source re-resolution.
P4R-INV-34  Rest excludes exact FieldKey (#x != "x"; 1 != true).
P4R-INV-35  Fixed runtime Record lookup remains O(log n) or better.
P4R-INV-36  Dynamic Record construction duplicate tracking is not O(n²).
P4R-INV-37  Equality/hash remain canonical logical operations.
```

## Metadata/API

```text
P4R-INV-38  Heterogeneous keys are never serialized through lossy coercion.
P4R-INV-39  Legacy metadata field names denote Symbol keys only.
P4R-INV-40  Unsupported heterogeneous legacy export fails closed.
P4R-INV-41  keyAt/rawKeyAt are the total heterogeneous key observation path.
P4R-INV-42  labelAt/rawLabelAt remain Symbol-only compatibility behavior in P4.
```

---

# 8. Non-goals

P4 remaining work does not own:

- a new general semantic query framework;
- replacing `SemanticDb`;
- replacing the row solver;
- widening ordered labels to `FieldKey`;
- P5 structural capability derivation;
- P6 static structural matching;
- P7 residualization/fixed-point static computation;
- P8 final durable heterogeneous metadata schema/version;
- P9 public API retirement/tooling polish;
- mutable Record semantics;
- Map/Record semantic unification;
- arbitrary runtime expressions as fixed Record pattern keys;
- unrelated Hypothesis/P10/C4.1 cleanup.


# 9. Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| key domain/identity | semantic `FieldKey` | rows/static/compiler/runtime/metadata | VM/store-local semantic key enum |
| key ordering | `FieldKey::canonical_cmp` | row/runtime/fingerprint/reflection | per-layer rank logic |
| exact integer key | existing exact static integer scalar | static/key/runtime conversion | truncated `i64` identity |
| static key binding selection | ordinary checker | evaluator/projector | textual lookup in projector |
| static expression execution | P2 `StaticEvaluator` | type/pattern key projection | type-specific or pattern-specific evaluator |
| static value → key | `StaticValueStore::field_key_of` | projection | ad-hoc caller conversion |
| row construction | `TypeStore` + Record-row authority | signatures/aliases/pattern typing | owner-local row maps |
| row relation | existing solver/relation | checker/pattern/P5 | new unordered solver |
| owner currentness | `SemanticDb` + canonical owner executor | session/replay | phase-local currentness graph |
| static constant semantics | one canonical `StaticConstant` owner | eager + demanded clients | session pipeline + demand pipeline |
| alias/static definition | `QueryKey::StaticDefinition` owner | resolver/signatures | provisional early alias form |
| field/callable signature | canonical owner preparation + query | surfaces/bodies/replay | replay with different inputs |
| resolved pattern key | `CallableBody` semantic product | compiler/P6 later | compiler re-reading AST |
| Record rest exclusion | exact `FieldKey` | semantic remainder/compiler | string labels |
| dynamic duplicate membership | private transient builder index | VM builder | repeated O(n) scan |
| durable heterogeneous metadata | P8 | cross-module persistence | P4 string coercion |
| checkpoint truth | C4 `CHECKPOINT.md` | all plans | plan-local competing status |

---

# 10. Expected impact map

## 10.1 Primary semantic files

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/static_constant_demand.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/types/relation.rs
```

A new private file such as:

```text
phalcom-semantic/src/owner_executor.rs
phalcom-semantic/src/static_constant_owner.rs
```

is permitted if it clarifies ownership. Names are mechanically flexible.

## 10.2 Primary compiler/runtime files

```text
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/heap/record_literal_builder.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/primitive/record.rs
phalcom-core/core/universe/src/collections/record.ph
```

Do not churn already-correct runtime FieldKey behavior.

## 10.3 Metadata/inspection

```text
phalcom-semantic/src/metadata/export.rs
phalcom-core/src/typing/inspect.rs
```

`phalcom-type-meta/**` is **VERIFY-FIRST**, not a mandatory P4 schema migration.

## 10.4 Tests

Inspect and extend the actual live test modules under:

```text
phalcom-semantic/tests/semantic/foundations/
phalcom-semantic/tests/semantic/incremental/
phalcom-semantic/tests/semantic/integration/
phalcom-semantic/tests/semantic/**pattern**
phalcom-core/tests/**record**
phalcom-core/tests/**pattern**
```

Known existing focused areas include static constants, type members, static-definition bodies, and query ownership. Verify exact selectors with `-- --list` if necessary.

## 10.5 Lifecycle docs

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/
  LANG007.C4.P4-luna-patch-grade-implementation-plan.md
  LANG007.C4.P4-walkthrough.md
  LANG007.C4.P4-handoff.md
```

The repository should amend/replace the active P4 remaining plan direction rather than leave two competing active execution contracts.

---

# 11. Implementer decision authority

## 11.1 FIXED

The implementer must not change:

1. four-kind `FieldKey`;
2. canonical key ordering;
3. exact integer semantics;
4. Symbol-only ordered labels;
5. ordinary checker as binding/type selection authority;
6. P2 evaluator as static execution authority;
7. `StaticValueStore::field_key_of` as key conversion authority;
8. existing row solver/type relation ownership;
9. one executable formation path per relevant `QueryKey`;
10. transactional Awaiting semantics;
11. exact dependency ownership;
12. no LKG/provisional-row semantics for current closed formation;
13. `CallableBody` ownership of body pattern-key dependencies;
14. P8 ownership of final durable heterogeneous metadata schema;
15. cold/incremental semantic equivalence.

## 11.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper/file names;
- struct vs closure/helper decomposition for owner executor;
- lifetimes and borrowing strategy;
- local outcome enum;
- exact location of canonical static-constant formation;
- generic vs per-owner reconciliation helpers;
- private cycle-frame representation;
- attempt-local diagnostic buffering;
- test placement following live conventions;
- `HashSet` vs `BTreeSet` for transient builder membership.

## 11.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| only owners in the live matrix can currently contain computed Record-key type forms | AST + owner builders | update matrix; do not implement old speculative list |
| immutable TypeStore interning during abandoned attempt is harmless | store/query semantics | STOP if rollback of mutable owner truth is needed |
| signatures can be routed through canonical preparation without SemanticDb redesign | `db/query.rs`, session | adapt private seam; STOP before changing DB authority |
| deferred aliases can be detected from `TypeDefinitionDef::type_form_body()` | AST/static definition | adapt source extraction mechanically |
| compiler can consume resolved body pattern facts | semantic lowering/compiler | introduce stable lowering fact if needed; never re-evaluate AST |
| Record builder is private/transient | heap/VM | consult if externally observable |
| legacy metadata cannot encode heterogeneous keys | type-meta schema | fail closed; do not invent encoding |
| native `rawLabelAt` rejects non-Symbol valid keys | primitive/runtime | document actual behavior; do not coerce |

---

# 12. Required live owner matrix

Before architecture edits, freeze a repository-grounded matrix:

| Source construct | Can carry computed Record key now? | Canonical `QueryKey` | Current formation | P4 action |
|---|---:|---|---|---|
| field annotation | yes | `FieldSignature` | signature query | canonical replay/preparation |
| callable parameter/return | yes | `CallableSignature` | signature query | canonical replay/preparation |
| `const` declared type | yes | `StaticConstant` | duplicated | collapse to one owner |
| type alias TypeForm | yes | `StaticDefinition` | early ordinary alias path | deferred exact owner |
| type-member prerequisite | checker can demand it | `TypeMemberDeclaration` | `publish_type_member_owner` | service through executor |
| declaration shell/surface | verify | existing | live builder | include only if source reachability proves it |
| trait header/surface | verify | existing | live builder | include only if proven |
| static-case signature | verify | existing | live builder | include only if proven |
| associated static-domain declaration | verify | existing | live builder | include only if proven |

The old T7.0C plan text is not a mandate to integrate every theoretical type-bearing query. Integrate actual reachable owners only.

---

# 13. Debugging and escalation

## 13.1 Mechanical failures

Permit up to three coherent correction cycles for imports, borrow cleanup, private signature drift, renamed helpers, and test plumbing.

```text
inspect
→ identify concrete cause
→ make one coherent correction
→ rerun smallest discriminator
```

## 13.2 Semantic failure protocol

Before nontrivial semantic correction:

```text
Observed:
Causal hypothesis:
Evidence:
Predicted effect:
Discriminating test:
```

One serious correction attempt for the same underlying semantic failure. If it persists: STOP AND CONSULT.

## 13.3 Immediate consultation triggers

Stop if:

- one-owner/one-path appears impossible;
- `SemanticDb` authority would need replacement;
- a provisional row or LKG semantic input appears necessary;
- ordinary checker selection cannot be preserved;
- compiler cannot consume semantic pattern keys without re-resolving source and no local semantic lowering seam can be introduced;
- row solver replacement appears necessary;
- ordered labels would need `FieldKey`;
- cold and incremental semantics require different products;
- unsupported metadata would need lossy encoding;
- an invariant must be weakened;
- scope becomes materially broader than the frozen owner matrix.

---

# 14. Verification strategy

## 14.1 Modes

### BUILD MODE

Run the smallest discriminator for the current slice.

### STABILIZE MODE

At named gates, run directly affected semantic/core suites.

### CERTIFY MODE

At P4 closure, run affected crate/package checks and P4 acceptance sets. Do not use broad unrelated failures as a substitute for causal verification.

## 14.2 Verification ladder

```text
V0 exact new regression
V1 focused feature selector
V2 owning module/subsystem suite
V3 semantic → compiler → runtime integration
V4 affected crate checks/tests
V5 broader checkpoint/workspace only if explicitly required
```

## 14.3 Failure classification

```text
A definitely introduced by this patch
B probably introduced by this patch
C unclear after one bounded causal pass
D clearly unrelated/pre-existing
```

A/B are active responsibility. C gets one bounded classification pass. D is recorded/deferred.

A P4-owned focused acceptance failure is a P4 blocker unless predecessor/baseline reproduction proves otherwise. “Baseline blocked” is not a blanket status for incomplete behavior.

---

# 15. Testing surface matrix

## 15.1 Owner/projection

| ID | Dimension | Required case |
|---|---|---|
| CV-O1 | key kinds | Symbol, String, large +/- Int, Bool |
| CV-O2 | static computation | supported operation/application |
| CV-O3 | invalid reduced value | non-FieldKey rejected |
| CV-O4 | runtime binding | runtime `let` rejected as static key |
| CV-O5 | exact dependency | owner records exact selected constant |
| CV-O6 | imported identity | remote SymbolId/StaticConstantId preserved |
| CV-O7 | same spelling | different modules stay distinct |
| CV-O8 | changed key | dependent owner republishes |
| CV-O9 | same-result edit | semantic result/cold parity retained |
| CV-O10 | duplicate after reduction | canonical duplicate rejected |
| CV-O11 | cold/incremental | same row/product/diagnostics |
| CV-O12 | unrelated edit | unrelated owner reusable |
| CV-O13 | Awaiting retry | exact prerequisite then fresh retry |
| CV-O14 | no leak | first-attempt diagnostics/products/deps absent |
| CV-O15 | direct mixed cycle | deterministic diagnostic/no hang |
| CV-O16 | transitive mixed cycle | deterministic path |
| CV-O17 | alias mixed cycle | alias → const → alias |
| CV-O18 | symbolic | DeferredToP7 creates no key/row |

## 15.2 Patterns

| ID | Required case |
|---|---|
| CV-P1 | `{ name: x }` |
| CV-P2 | `{ ["x"]: x }` |
| CV-P3 | `{ [42]: x }` |
| CV-P4 | `{ [true]: x }` |
| CV-P5 | Symbol/String same-text distinction |
| CV-P6 | Int/Bool distinction |
| CV-P7 | missing closed key |
| CV-P8 | optional fixed field |
| CV-P9 | typed open-tail lookup |
| CV-P10 | exact rest subtraction |
| CV-P11 | nested Record pattern |
| CV-P12 | invalid/non-static key |
| CV-P13 | const-backed pattern key |
| CV-P14 | const edit invalidates body, not signature |
| CV-P15 | lowering consumes resolved key |

## 15.3 Performance / metadata / API

| ID | Required case |
|---|---|
| CV-R1 | 1k dynamic fields avoid repeated linear scan |
| CV-R2 | 10k construction same structural algorithm |
| CV-R3 | final canonical shape/lookup still valid |
| CV-M1 | legacy Symbol export works |
| CV-M2 | String/Int/Bool legacy export fails |
| CV-M3 | non-Symbol tail domain fails closed |
| CV-M4 | legacy metadata read yields Symbol |
| CV-M5 | no large-Int truncation/stringification |
| CV-A1 | `keyAt` covers all key kinds |
| CV-A2 | `labelAt` non-Symbol behavior matches compatibility contract |

---

# 16. Task graph

```text
T0  Re-baseline and freeze actual owner matrix
 │
 ▼
T1  Collapse StaticConstant to one canonical owner
 │
 ▼
T2  Add bounded exact-owner executor + transactional suspension
 │
 ▼
T3  Make field/callable replay projection-safe
 │
 ▼
T4  Implement deferred computed-key StaticDefinition aliases
 │
 ▼
G1  Owner-architecture convergence gate
 │
 ▼
T5  Complete T7.0E
 │
 ▼
G2  T7.0E hard PASS gate
 │
 ├─────────────┐
 ▼             ▼
T6 pattern     T8 builder performance
semantics      hardening
 │             │
 ▼             │
T7 compiler/   │
runtime        │
 └─────┬───────┘
       ▼
T9  Metadata/API/incremental hardening
       │
       ▼
T10 Stabilize/certify/lifecycle closure
```

T8 may proceed after G2 in parallel with T6/T7 if worktree coordination permits.

---

# 17. T0 — Re-baseline and freeze the actual owner matrix

## Purpose

Replace stale plan assumptions with the smallest live set of owners P4 must service.

## Required reads

```text
AGENTS.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/implementation-record-lifecycle-convention.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
current P4 plan
P4 handoff
P4 walkthrough

docs/specs/structural-products.md
docs/specs/type-level-domains.md
docs/specs/static-evaluation.md
docs/specs/vm/products.md

phalcom-semantic/src/session.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/static_constant_demand.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/types/annotation.rs
```

## Work

1. Confirm live revision and scoped P4 diff.
2. Preserve unrelated changes.
3. Enumerate all calls to:
   - `type_record_key_expressions`;
   - `project_annotation` / `project_annotations`;
   - `query_field_signature_with_inputs`;
   - `query_callable_signature_with_inputs`;
   - `query_static_definition` / `_with_record_key_projections`;
   - `query_static_constant`.
4. Fill the owner matrix in §12.
5. Mark each owner:
   - already canonical;
   - projection-aware but replay-unsafe;
   - duplicated;
   - deferred/unimplemented;
   - theoretical/out-of-scope.
6. Record exact focused test selectors.
7. Amend the repository P4 plan/checkpoint to make this corrective ordering authoritative.

## Acceptance

- owner matrix is explicit;
- one intended canonical formation entry exists on paper for every required owner;
- no production code change yet beyond lifecycle-plan amendment if desired.

---

# 18. T1 — Collapse `StaticConstant` to one canonical owner formation path

## Purpose

Remove the largest source of T7 semantic duplication before adding scheduling behavior.

## Current symbols

### Demand path

```text
phalcom-semantic/src/static_constant_demand.rs

StaticConstantDemandDriver::force
StaticConstantDemandDriver::compute
StaticConstantDemandDriver::compute_with_projected_keys
StaticConstantDemandDriver::check_evaluate_publish
StaticConstantDemandDriver::recursive_cycle_outcome
StaticConstantDemandDriver::publish_recursive_cycle
```

### Session path

```text
phalcom-semantic/src/session.rs

ConstantDeclInfo
constant_declarations
checked_constants
constant_graph
constant_record_key_dependencies
ConstantTarjan
visit_constant
cycle publication branch
non-cycle projection/check/evaluate/publication branch
```

### DB publication

```text
db/query.rs::query_static_constant
db/query.rs::cached_static_constant
```

## Target architecture

Create one canonical operation, conceptually:

```rust
fn ensure_static_constant_owner_current(
    ctx: &mut OwnerContext<'_>,
    id: &StaticConstantId,
) -> OwnerEnsureOutcome
```

or a mechanically equivalent form/publish split.

It owns:

1. catalog/source lookup;
2. current-product reconciliation;
3. initializer discovery/check;
4. exact prerequisite demands;
5. declared-type computed-key projection;
6. expected-type formation;
7. fresh initializer recheck against current products;
8. P2 evaluation;
9. cycle diagnostic/product formation;
10. `StaticConstantProduct`;
11. canonical DB publication;
12. reconciliation into `StaticConstantTable`.

## Required convergence/deletion

Final code must not retain two semantic implementations.

Preferred direction:

- extract canonical owner formation from the demand-capable path;
- have the eager session constant phase call the same owner executor;
- delete the session's duplicate check/evaluate/publish logic;
- reduce/retire `StaticConstantDemandDriver` as an owner-forming scheduler;
- remove `active_constants` once T2 owner stack exists;
- remove `constant_record_key_dependencies` as independent cycle authority;
- remove local `ConstantTarjan` from publication semantics.

If a graph remains, it must be demonstrably non-authoritative.

## Algorithmic intuition

A static constant is a semantic owner node, not a closed phase-only node:

```text
ensure K
  → check enough to discover exact prerequisites
  → ensure prerequisites
  → form K
```

This naturally supports mixed edges to aliases/type owners.

## LKG rule

The `lkg_value` field may remain for editor/diagnostic compatibility if used elsewhere, but computed-key formation must only use the current product's `value`.

```text
value absent/invalid/recursive
    -> no closed FieldKey from LKG
```

## Tests

Focused discriminators:

- ordinary const dependency;
- declared computed key depending on const;
- direct self-cycle;
- transitive const cycle;
- changed dependency invalidation;
- cold behavior unchanged from valid predecessor cases.

## Acceptance

- every session static constant publication goes through one owner path;
- no second path independently checks/evaluates/publishes;
- exact dependencies remain precise;
- focused static-constant tests pass.

---

# 19. T2 — Bounded exact-owner executor + transactional suspension

## Purpose

Replace owner-specific demand servicing with one exact orchestration seam.

## Primary files

```text
session.rs
checker/context.rs
checker/static_body.rs
static_constant_demand.rs
db/query.rs
```

A new private module is allowed.

## Useful existing seams

Preserve:

```rust
TrackingTypeResolver::track_deferred_static_definition
TrackingTypeResolver::semantic_query_demands_snapshot

StaticExpressionCheckOutcome::{
    Ready,
    Awaiting(BTreeSet<QueryKey>),
    Invalid,
}
```

The checker already emits exact demands for `StaticDefinition` and `TypeMemberDeclaration`; checked static expressions expose exact static-constant reads.

## Executor algorithm

```text
ensure_owner_current(key, cause):

  if db key is current:
      reconcile product into session view
      return Current

  if key already in active stack:
      classify cycle from stack slice + edge cause
      return diagnosed terminal result

  push OwnerFrame(key, cause)

  loop:
      attempt canonical owner formation

      if complete:
          publish/reconcile
          pop
          return

      if Awaiting(demands):
          discard attempt-local semantic outputs
          for demand in deterministic order:
              ensure_owner_current(demand, appropriate cause)
          rebuild current checker/resolver context
          retry from source

      if blocked/cancelled/budget/failed:
          pop
          propagate
```

## Minimum dispatch set

Based on T0, expected:

```text
StaticConstant
StaticDefinition
TypeMemberDeclaration
FieldSignature
CallableSignature
```

Do not make this a universal query framework.

## Ready-product reconciliation

The executor may reconcile:

```text
StaticConstant          -> StaticConstantTable
StaticDefinition        -> StaticDefinitionTable + alias table/resolver
TypeMemberDeclaration   -> member table + resolver
FieldSignature          -> FieldSignatureTable
CallableSignature       -> CallableSignatureTable
```

Reconciliation is not semantic formation.

## Replace `service_type_member_demands`

Delete it or make it a trivial adapter into the executor. Do not create sibling service loops.

## Attempt-local diagnostics

Buffer diagnostics until an owner attempt reaches a non-Awaiting terminal semantic result.

Never append suspended-attempt diagnostics to the session's durable diagnostic map.

## Attempt-local resolver/table state

No canonical insertion before successful publication.

## Cycle causes

Retain cause/provenance such as:

```rust
OrdinaryDependency
StaticExpressionRead
ComputedRecordKey(RecordKeyUseId)
```

so diagnostics can distinguish a static-formation cycle through a key.

## Required no-leak regression

Construct an owner attempt where incomplete context would produce a transient diagnostic before discovering a not-current exact prerequisite. After servicing and retry:

- final result succeeds;
- transient diagnostic absent;
- no first-attempt product;
- no first-attempt dependency edge;
- no stale resolver form.

## Acceptance

- one executor services exact owner demands;
- type-member mini-scheduler is gone as an independent architecture;
- Awaiting is transactional;
- unified stack is the active cross-owner cycle trace;
- `active_constants` is gone by end of task.

---

# 20. T3 — Make field/callable signature re-entry projection-safe

## Purpose

Close the cold-versus-replay semantic hole before alias work depends on this machinery.

## Primary symbols

```text
db/query.rs::SignatureQueryInputs
db/query.rs::query_field_signature_with_inputs
db/query.rs::query_callable_signature_with_inputs
db/query.rs::ensure_formal_semantic_dependency_current
```

## Required owner entries

Conceptually:

```rust
ensure_field_signature_current(...)
ensure_callable_signature_current(...)
```

Each:

1. identifies owner source;
2. detects computed Record-key sites;
3. uses ordinary path if none;
4. otherwise projects all annotations under the stable owner `QueryKey`;
5. services exact prerequisites through T2;
6. rebuilds/retries fresh after suspension;
7. calls the low-level query with owner-matching projections;
8. reconciles the result.

Cold/session callers and formal replay callers use the same entry.

## Low-level query precondition

It is acceptable for `query_*_signature_with_inputs` to remain low-level if its contract is explicit:

> An owner containing computed Record keys must be called with the matching prepared projection set.

Generic formal replay may not call it unprepared.

## Mechanically acceptable designs

- outer executor ensures dependencies before low-level publication;
- narrow preparation callback/context in formal query inputs;
- split dependency validation from product formation.

## Forbidden

- rebuilding key projections inside `SemanticDb` without session context;
- persisting `ProjectedRecordKeySet<'a>`;
- source/AST pointer identity in persistent state;
- treating `None` as “try to form anyway” for a known computed-key owner;
- adding `QueryKey::RecordKeyProjection`.

## Tests

- cold field signature;
- cold callable parameter + return;
- invalidate key constant and force replay through downstream dependency;
- replay sees new key;
- fresh cold run matches incremental;
- same-result edit remains structurally coherent;
- exact dependency edge retained.

## Acceptance

No executable path forms a projection-bearing field/callable signature without canonical projection preparation.

---

# 21. T4 — Deferred computed-key `StaticDefinition` / type-alias ownership

## Purpose

Implement INC-5 only after owner convergence.

## Primary symbols/files

```text
session.rs
static_definition.rs::resolve_static_definition
static_definition.rs::StaticDefinitionTable
db/query.rs::query_static_definition
db/query.rs::query_static_definition_with_record_key_projections
checker/context.rs::TrackingTypeResolver
types/annotation.rs::ProjectedRecordKeyResolver
static_key.rs
ast.rs::TypeDefinitionDef::type_form_body
```

## 21.1 Classification in ordinary alias phase

Conceptually:

```rust
let deferred =
    alias.type_form_body()
        .is_some_and(|f|
            !type_record_key_expressions(f, 0).is_empty());
```

Equivalent live extraction is acceptable.

### Ordinary alias

Keep existing eager path.

### Computed-key alias

Do not:

- resolve final alias form;
- publish `StaticDefinition`;
- insert into `TypeAliasTable`;
- insert resolver alias form;
- leave old revision form current.

Instead:

1. remove/invalidate stale prior forms as normal incremental logic requires;
2. record declaration in `deferred_static_definitions`;
3. expose this set to checker contexts using the existing tracking hook;
4. let use-sites demand exact `QueryKey::StaticDefinition(id)`.

## 21.2 Canonical alias owner algorithm

```text
ensure StaticDefinition(A)

1. current? reconcile and return

2. load source/declaration + existing alias dependency facts

3. ensure ordinary prerequisites needed for current context

4. collect computed key sites in type-form body

5. for each site:
     ordinary checker
     if Awaiting -> suspend/retry
     ensure selected constants current
     P2 evaluate
     field_key_of
     retain projection

6. ProjectedRecordKeyResolver(parent, projections)

7. resolve_static_definition(... projected resolver ...)

8. only after complete:
     query_static_definition_with_record_key_projections

9. reconcile:
     StaticDefinitionTable
     TypeAliasTable when alias_info exists
     resolver alias form
     resolver static-definition view

10. retry waiting owner from fresh context
```

## 21.3 Dependency and cycle rules

- preserve existing alias dependency graph/SCC semantics;
- exact key dependencies are added to the `StaticDefinition` owner;
- imported dependencies retain remote identity;
- alias → const → alias uses exact owner stack;
- no provisional alias form breaks a cycle.

## 21.4 Snapshot closure

Do not silently omit a deferred source alias because nobody demanded it.

Before snapshot finalization, force deferred aliases required by the source/snapshot semantic contract to a current or diagnosed-invalid result. If existing semantics deliberately permit unused aliases to remain lazily unformed, verify that contract explicitly before relying on it.

## Tests

- cold computed-key alias;
- alias consumed by field signature;
- alias consumed by callable signature;
- local const-backed alias;
- imported const-backed alias;
- imported alias;
- same spelling across modules;
- key change republishes alias + dependent signature;
- unrelated const does not;
- duplicate key after reduction;
- existing syntax alias cycle diagnostic preserved;
- alias → const → alias mixed cycle;
- Awaiting alias attempt leaks nothing;
- cold/incremental parity.

## Acceptance

- production deferred set is real, not empty scaffolding;
- computed-key alias absent until canonical owner publishes;
- exact `StaticDefinition` demand routing works;
- resolver/table insertion happens only after publication;
- no provisional alias form or phase-order workaround.

---

# 22. G1 — Owner architecture convergence gate

Do not proceed to full T7.0E until:

```text
[ ] one StaticConstant semantic formation path
[ ] bounded exact-owner executor
[ ] no independent active_constants cycle authority
[ ] Awaiting transactional
[ ] type-member demand routed through executor
[ ] field replay projection-safe
[ ] callable replay projection-safe
[ ] computed-key alias owner works
[ ] no provisional alias/row/LKG semantic input
```

Focused constants/signatures/type-members/static-definition tests must pass.


# 23. T5 — Complete T7.0E: computed-key type-owner acceptance

## Purpose

Prove the corrected owner architecture. T7.0 does not complete because the code compiles or because a few owner slices pass.

## 23.1 Key-domain matrix

Exercise:

```text
Symbol
String
Bool
large positive Int
large negative Int
```

Use integers beyond `i64` in at least one test so exact arbitrary-precision identity is established.

## 23.2 Static-computation matrix

Include:

- direct key literals;
- key constants;
- a supported static arithmetic/string/symbolic operation as appropriate to P2;
- a supported static function/application/construction path already implemented by P2;
- reduced non-FieldKey value;
- runtime-only binding rejection;
- symbolic/P7 deferral if currently reachable.

The point is to prove reuse of P2, not just literal fast paths.

## 23.3 Owner matrix

At minimum, based on current live reachability:

```text
FieldSignature
CallableSignature
StaticConstant declared type
StaticDefinition / type alias
```

Exercise `TypeMemberDeclaration` as a demanded prerequisite because current checker infrastructure already emits exact type-member readiness demands.

Add further owner kinds only if T0 proved real source reachability.

## 23.4 Exact dependency identity matrix

Prove:

- local exact `StaticConstantId`;
- imported remote exact `StaticConstantId`;
- same spelling in different modules remains distinct;
- owner dependency index contains the selected owner, not a module-wide substitute;
- unrelated constant edits do not republish the owner.

Identity chain to preserve:

```text
checker-selected binding
    -> SymbolId
    -> StaticConstantId
    -> SemanticDependency::StaticConstant(id)
    -> QueryKey::StaticConstant(id)
```

Do not re-resolve by source text during projection.

## 23.5 Incremental matrix

Run edit sequences against an already-analyzed session and compare with a fresh cold session after the edit.

### Key identity change

```text
#a -> #b
```

### Key kind change

```text
#x -> "x"
```

### Same semantic result after source edit

Change the expression/source spelling while it still reduces to the same `FieldKey`.

### Duplicate introduced

Change one projected key so two fields reduce to the same canonical key.

### Duplicate removed

Change the key back to a distinct value.

For each compare:

```text
incrementally updated session
vs
fresh cold session
```

on:

- concrete key;
- Record row/type;
- owner product;
- diagnostics;
- downstream signature/body visibility;
- exact dependency edges.

## 23.6 Suspension/no-leak matrix

At least one test must force:

```text
owner attempt
  -> computed key check
  -> exact prerequisite not current
  -> Awaiting
  -> prerequisite owner becomes current
  -> fresh owner attempt
  -> success
```

Assert the first attempt leaves:

```text
no canonical diagnostic
no published owner product
no dependency edge
no stale alias/resolver form
```

The test should be discriminating enough that a leaked transient diagnostic would fail it.

## 23.7 Cycle matrix

Required where current syntax permits:

```text
const K -> K's computed declared-type key -> K

const A -> key expression -> const B -> ... -> A

alias A -> key const K -> alias A

alias A -> alias B -> key const K -> A
```

Expected:

- deterministic source/domain diagnostic;
- bounded termination;
- no stale LKG key;
- no fabricated `FieldKey`;
- no partial canonical row;
- no owner permanently left in an unexplained computing state.

## 23.8 Imported matrix

Where module fixtures permit:

- local owner → imported key constant;
- imported alias with computed key;
- remote key change invalidates importing dependent;
- unrelated remote edit does not;
- imported identity remains remote, not re-bound locally.

## 23.9 Duplicate-after-reduction authority

Use a fixture such as:

```telos
const A = #x
const B = #x

type T = {
    [A]: Int,
    [B]: String,
}
```

Before projection there are two syntactic sites. After projection there is one canonical key twice.

The error must come from the normal canonical row formation/duplicate rule. Do not pre-deduplicate expressions/constants.

## 23.10 P7 seam

If projection returns `DeferredToP7`:

- no key is fabricated;
- no closed row is formed using a placeholder;
- no AST is rewritten with a fake key;
- behavior follows the current deferred/static-stage contract;
- P4 does not implement residualization.

## Acceptance

T7.0E passes only when:

```text
cold == incremental
exact dependencies are stable
replay == cold owner preparation
cycles terminate diagnostically
suspension is transactional
no symbolic placeholder reaches canonical rows
```

---

# 24. G2 — T7.0E hard gate

**Do not begin Record-pattern work until T5 passes.**

Patterns reuse the same checked-static-expression and owner-demand architecture. Adding a second consumer before the scheduler/owner architecture is stable increases the blast radius and obscures root causes.

Checkpoint consequence at G2:

```text
P4: still IN_PROGRESS / PARTIAL
T7.0: IMPLEMENTED / FOCUSED_TESTED
T7.1: unblocked
```

Do not mark P4 complete.

---

# 25. T6 — Resolve heterogeneous Record pattern keys in `CallableBody`

## Purpose

Carry the already-generalized pattern-key syntax into canonical semantic pattern facts.

## Primary files/symbols

```text
phalcom-ast/src/ast.rs
  RecordKeySyntax
  RecordPatternEntry

phalcom-semantic/src/checker/pattern.rs
  resolve_record_pattern

phalcom-semantic/src/match_semantics.rs
  ResolvedRecordFieldPattern

phalcom-semantic/src/static_key.rs

callable-body checking/analysis path discovered live
```

## 25.1 Current limitation

The live semantic checker currently has logic equivalent to:

```rust
let label_opt = entry.label();
let key_opt = label_opt.map(FieldKey::symbol);
```

and Record rest subtraction looks only at Symbol labels.

`ResolvedRecordFieldPattern` currently stores a text label.

These are P4 bugs once computed pattern syntax exists.

## 25.2 Target semantic product

Conceptually:

```rust
pub struct ResolvedRecordFieldPattern {
    pub key: FieldKey,
    pub key_range: SourceRange, // or existing provenance equivalent
    pub child: Box<PatternResolution>,
}
```

If canonical source provenance already exists elsewhere in the resolved pattern graph, reuse it instead of adding redundant range storage.

Identity is `key`; range/provenance is diagnostic metadata.

## 25.3 Pattern key-site collection

Do not force pattern ASTs through `type_record_key_expressions`.

Instead, either:

- add a pattern-specific deterministic site collector that feeds the shared checker/projector; or
- generalize the owner-local projection helper to accept pre-collected key sites.

The shared semantic pipeline must remain:

```text
ordinary checker
→ CheckedStaticExpression
→ exact prerequisite service
→ P2 StaticEvaluator
→ field_key_of
```

No pattern-only evaluator or resolver.

## 25.4 Owner-relative identity

Pattern key facts belong to:

```rust
QueryKey::CallableBody(callable_id)
```

Use deterministic owner traversal order for any local `RecordKeyUseId`-like identity.

A body pattern edit should invalidate/recompute the body. It should not invalidate `CallableSignature` unless signature inputs separately changed.

## 25.5 Exact dependency attachment

Merge exact checker/static dependencies from pattern key expressions into the callable body semantic dependency set.

Do not record them on the signature.

## 25.6 Expected-field typing

Replace direct fixed-field-only logic with the existing unordered lookup authority.

Conceptually:

```rust
match lookup_unordered_field(..., key) {
    UnorderedFieldLookup::Required(ty) => { ... }
    UnorderedFieldLookup::Optional(ty) => { ... }
    UnorderedFieldLookup::Absent => { ...diagnostic... }
    UnorderedFieldLookup::Unknown => { ...conservative existing policy... }
}
```

Requirements:

- Required stays Required.
- Optional stays refutable/optional under existing match rules.
- typed anonymous tail can contribute precision where membership/type-domain rules prove it;
- Absent on a closed known row uses existing mismatch diagnostics;
- Unknown is not proof of presence.

## 25.7 Rest semantic type

For:

```telos
{ [42]: x, **rest }
```

subtract exactly `FieldKey::Int(42)` from known fixed fields and preserve tail semantics.

Examples:

```text
matching #x leaves "x"
matching "x" leaves #x
matching 1 leaves true
matching true leaves 1
```

Re-intern the remainder through the existing checked row constructor. Do not mutate the original row.

## 25.8 Invalid key behavior

A fixed structural Record pattern key must reduce statically to a concrete `FieldKey`.

Runtime-only expressions or invalid key-domain values are rejected through ordinary checker/projector diagnostics. Do not fall back to Map semantics or Symbol stringification.

## Semantic tests

Implement CV-P1 through CV-P14 at the semantic layer, including:

- four key kinds;
- exact kind distinctions;
- closed missing field;
- optional/open-tail typing;
- rest type;
- const-backed key;
- invalid key;
- body dependency ownership;
- key-constant incremental edit invalidates body but not signature.

## Acceptance

- semantic Record patterns no longer use `entry.label()` as canonical identity;
- `ResolvedRecordFieldPattern` carries exact `FieldKey`;
- shared static projection is used;
- `UnorderedFieldLookup` owns field lookup precision;
- body owns exact static dependencies.

---

# 26. T7 — Lower exact resolved Record pattern keys

## Purpose

Remove remaining string/Symbol reconstruction from executable and destructuring pattern lowering.

## Primary file

```text
phalcom-core/src/compiler/lib/patterns.rs
```

Also inspect the semantic-body-to-compiler lowering bridge.

## 26.1 Current problematic patterns

The live compiler contains Record paths that use forms such as:

```rust
Value::symbol(...)
entry.label().unwrap_or("")
```

and:

```rust
emit_record_rest_temp(
    ...,
    matched_labels: &[String],
    ...
)
```

with `rawLabelAt`.

These paths must not remain authoritative for Record pattern keys.

## 26.2 Compiler input

Compiler lowering must receive/use the resolved semantic key:

```text
ResolvedRecordFieldPattern.key: FieldKey
```

Convert using the existing canonical runtime helper, conceptually:

```rust
field_key_to_value(&resolved.key, ...)
```

Do not switch on or evaluate the original source expression again.

## 26.3 Audit every Record pattern lowering path

At minimum:

- refutable match tests;
- destructuring/binding;
- nested patterns;
- rest reconstruction;
- any assignment/binding path that shares pattern compilation.

Every route must preserve exact heterogeneous key semantics.

## 26.4 Rest reconstruction

Change the logical exclusion input from:

```rust
&[String]
```

to exact keys:

```rust
&[FieldKey]
```

or a lowered equivalent derived directly from semantic keys.

Use the heterogeneous key observation path:

```text
rawKeyAt
```

not `rawLabelAt`.

Preserve all non-excluded fields exactly.

## 26.5 No Map conflation

`MapPatternKey` remains its own semantic category. Do not merge Map and Record pattern implementation merely because both may use non-Symbol scalar keys.

## 26.6 If a compiler path has only AST

If an executable path currently lacks access to resolved pattern facts, add/extend a stable body semantic lowering fact.

Forbidden fallback:

```text
compiler sees RecordKeySyntax::Computed
→ evaluates/resolves it itself
```

Semantic authority must remain in `phalcom-semantic`.

## Core/runtime tests

Required:

- destructure Symbol/String/Int/Bool;
- Symbol/String same-text distinction;
- Int/Bool distinction;
- nested Record pattern;
- exact key absence fails;
- rest excludes exact key only;
- lowering test proves semantic key, not AST label spelling, is consumed.

## Acceptance

A source audit finds no unconditional Symbol conversion or label-string rest exclusion in Record semantic lowering paths.

---

# 27. T8 — Remove quadratic dynamic Record construction

## Purpose

Meet P4's performance contract without changing permanent Record identity/shape architecture.

## Primary file

```text
phalcom-core/src/heap/record_literal_builder.rs
```

## Target representation

Preferred:

```rust
pub struct RecordLiteralBuilderObject {
    entries: Vec<(FieldKey, Value)>,
    seen: HashSet<FieldKey>,
}
```

A `BTreeSet<FieldKey>` is also valid.

`entries` remains necessary for source/presentation order. `seen` is only transient construction membership.

## Why this is not a competing Record index

The set is:

- private;
- transient;
- builder-local;
- discarded after final construction;
- not structural identity;
- not reflection order;
- not a persistent shape lookup cache.

Final `RecordProductShape` remains canonical sorted logical storage.

## Required operations

If the VM currently does:

```text
contains_key
then append
```

make `contains_key` O(1) expected or O(log n), or replace with a private atomic `try_append`.

Ensure all construction routes, including expansion/spread, use the indexed duplicate check.

If `take_entries` leaves the builder reusable, reset membership consistently:

```rust
let entries = std::mem::take(&mut self.entries);
self.seen.clear();
entries
```

or consume the builder.

## GC/lifetime verification

Verify `FieldKey` contains no GC-managed `Value` requiring heap tracing. Existing `entries` values remain the GC trace source.

Do not add duplicate rooting solely for `seen`.

## Complexity target

```text
dynamic duplicate tracking:
  HashSet -> expected O(n)
  BTreeSet -> O(n log n)

final canonical shape:
  O(n log n)

fixed lookup:
  O(log n)

equality/hash:
  O(n)
```

## Evidence

Avoid fragile wall-clock assertions.

Prefer:

- structural unit coverage for the indexed path;
- test-only comparison/counter if repository convention supports it;
- 1k and 10k construction smoke/scaling fixtures;
- existing benchmark harness only if one already exists.

## Acceptance

No production dynamic Record builder performs a linear scan for each inserted field.

---

# 28. T9 — Metadata, API, incrementality, and compatibility hardening

## Purpose

Close P4 durability boundaries without taking P8/P9 ownership.

## 28.1 Fail-closed metadata

Preserve:

```rust
MetadataExportError::NonExportableRecordKey(...)
MetadataExportError::NonExportableRecordKeyDomain
MetadataExporter::legacy_record_field_name
```

Required behavior:

```text
Symbol -> legacy export allowed
String -> error
Int    -> error
Bool   -> error
non-Symbol key-domain tail -> error
```

No `Display`, `toString`, JSON-ish encoding, Symbol coercion, or integer truncation may be used as a compatibility escape hatch.

## 28.2 Legacy metadata read

`phalcom-core/src/typing/inspect.rs` currently maps legacy field `name` to:

```rust
FieldKey::symbol(...)
```

This is correct because the legacy schema represents names, not the full `FieldKey` domain.

Test that legacy `"x"` means `Symbol("x")`, not `String("x")`.

## 28.3 P8 boundary

Do not introduce the final durable heterogeneous `FieldKeyRef` wire/schema solely because the old P4 plan mentioned it.

Only add an in-process staging representation if a concrete current P4 consumer cannot otherwise preserve lossless information.

If cross-module durable schema/version changes become necessary to pass P4 behavior: STOP AND CONSULT.

## 28.4 `keyAt` / `labelAt` contract

Universe Record currently exposes:

```telos
rawLabelAt(_ index: Int) -> Symbol
rawKeyAt(_ index: Int) -> FieldKey
labelAt(...)
keyAt(...)
rawLookup(_ key: FieldKey)
```

P4 should establish/document/test:

- `keyAt`/`rawKeyAt` are the total heterogeneous key observation path;
- `labelAt`/`rawLabelAt` remain Symbol-only compatibility behavior;
- non-Symbol behavior follows the current native error/precondition;
- no implicit conversion to Symbol.

API retirement/renaming belongs to later public-ontology work.

## 28.5 Incremental hardening after pattern work

Audit:

- pattern-key dependency invalidates `CallableBody`;
- signature remains reusable when only body pattern key changes;
- alias dependency invalidation is targeted;
- unaffected owner reuse remains;
- no retained key identity depends on source range/AST address.

## 28.6 Ordered-product isolation regression

Run focused Tuple/Pack/call label tests proving:

```text
GuardSymbol remains ordered-label authority
non-Symbol Record key cannot become call/Pack/Tuple label
P3 Pack behavior remains unchanged
```

## Acceptance

- metadata behavior is honest and fail-closed;
- runtime API distinction is explicit;
- no P8 schema expansion;
- incremental pattern/body behavior is targeted;
- P3 ordered-label boundary remains intact.

---

# 29. T10 — Stabilize, certify P4, and close lifecycle records

## Purpose

Establish honest completion evidence and leave a clean P5/P8/P9 boundary.

## 29.1 Static source audit before broad testing

Search for stale patterns such as:

```text
Record semantic paths using String/Box<str> as key identity
entry.label() in Record semantic/compiler paths
FieldKey::symbol fallback for computed Record patterns
rawLabelAt in heterogeneous Record rest lowering
record_key_projections: None on projection-bearing re-entry
active_constants as independent cycle authority
second StaticConstant check/evaluate/publish path
linear RecordLiteralBuilder contains_key scan
lossy metadata key formatting
```

Every hit must be classified:

```text
valid legacy/Symbol-only boundary
or
bug
```

## 29.2 STABILIZE gate

Run serially:

1. exact T7.0E semantic selectors;
2. Record-pattern semantic selectors;
3. Record-pattern core/runtime selectors;
4. builder complexity regressions;
5. metadata fail-closed tests;
6. ordered-label isolation regressions;
7. the already-completed T1–T6 focused selectors most plausibly affected by owner plumbing.

Confirm filters select actual tests.

## 29.3 Affected crate checks

At minimum:

```text
cargo check -p phalcom-semantic
cargo check -p phalcom-core
```

plus directly affected test targets.

Use repository-standard wrapper/RUSTFLAGS conventions from the active handoff.

## 29.4 CERTIFY gate

Only now run broader semantic/core suites required by checkpoint workflow or explicit project requirements.

Do not repair unrelated failures. Classify A/B/C/D and record them.

## 29.5 Lifecycle updates

### P4 plan state

Use:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only if required focused acceptance actually passed.

Use `BASELINE_BLOCKED` only if a required broader gate is genuinely prevented by proven unrelated baseline failure.

### C4 checkpoint

Update:

- P4 ledger row;
- established owner/projection invariants;
- verification evidence;
- residual/baseline issues;
- consultation/amendment ledger;
- final revision;
- next action.

### Walkthrough

Record:

- static-constant owner convergence;
- exact-owner executor;
- signature replay fix;
- deferred alias ownership;
- T7.0E evidence;
- pattern semantic/compiler migration;
- builder complexity fix;
- metadata scope boundary;
- deviations from plan;
- exact tests run/deferred.

### Handoff

Give the next plan:

- canonical `FieldKey`;
- canonical row/lookup APIs;
- stable owner/replay invariants;
- resolved pattern FieldKey facts;
- P8 metadata boundary;
- ordered-label isolation;
- performance facts;
- known residual issues.

## P4 completion definition

P4 is complete only when:

```text
[ ] T1-T6 substrate preserved
[ ] StaticConstant has one executable formation path
[ ] exact-owner executor exists and is bounded
[ ] Awaiting is transactional
[ ] field/callable cold and replay preparation are equivalent
[ ] computed-key aliases use canonical StaticDefinition owner
[ ] T7.0E matrix passes
[ ] Record patterns resolve and lower exact FieldKey
[ ] dynamic Record construction is not O(n²)
[ ] legacy metadata is lossless-or-fail-closed
[ ] ordered labels remain Symbol-only
[ ] focused P4 acceptance is green
[ ] lifecycle records match actual evidence
```

---

# 30. Algorithmic and conceptual references

## 30.1 Demand-driven semantic owner graph, not a closed phase graph

The old constant model is effectively:

```text
collect constants
→ infer constant-only graph
→ SCC
→ publish constants
```

That is appropriate only while the dependency domain is closed.

Computed type keys create edges among heterogeneous semantic owners:

```text
StaticConstant
StaticDefinition
FieldSignature
CallableSignature
TypeMemberDeclaration
...
```

The correct abstraction is a demand-driven graph whose vertices are the existing `QueryKey`s.

This does **not** prohibit deterministic eager iteration. Eager passes may request owners in source/stable order. It only means that the semantic dependency truth is owned by exact query edges, not by a parallel phase graph.

## 30.2 DFS active-stack cycle detection

For P4's bounded owner graph, active-stack detection is enough to identify an executing cycle.

Example active stack:

```text
A --ordinary--> B --computed-key--> C
```

If C requests B, the cycle slice is:

```text
B --computed-key--> C -> B
```

The transient stack preserves edge causes, which a plain SCC component does not. This is useful for choosing the domain diagnostic.

Persistent dependency/cached state remains in `SemanticDb`; the stack is not stored as semantic product identity.

## 30.3 Why no fixed point or LKG solution exists for closed key formation

Consider:

```telos
const K: { [K]: Int } = ...
```

A closed Record row needs a concrete canonical key set.

Using:

- old value of K;
- a symbolic placeholder;
- source expression identity;
- “first pass” provisional key;

would make row identity depend on non-current information and break:

- deterministic type identity;
- duplicate detection;
- incremental equivalence;
- exact dependency invalidation.

Therefore this is a static-formation cycle and must be diagnosed, not solved with a fixed point in P4.

## 30.4 Transactional restart as semantic discipline

An `Awaiting` checker pass behaves like an optimistic read transaction:

```text
read current semantic products
discover missing prerequisite
abort attempt
make prerequisite current
restart
```

Preserving partially checked mutable state across suspension creates order-sensitive diagnostics and dependency ownership. Fresh retry is simpler and semantically stronger.

## 30.5 Duplicate detection after projection

Example:

```telos
const A = #x
const B = #x

type T = {
    [A]: Int,
    [B]: String,
}
```

The syntax has two distinct expressions and two distinct constants. Canonical semantics has the same `FieldKey::Symbol("x")` twice.

Therefore duplicate detection belongs **after** projection and **inside the existing row canonicalization path**.

## 30.6 `RecordKeyUseId` is provenance, not a new query identity

The stable local identity:

```text
(owner QueryKey, deterministic traversal ordinal)
```

is appropriate because ranges move and pointers are ephemeral.

It is useful for:

- diagnostics;
- owner-local projection lookup;
- causal cycle information.

It must not become:

```text
QueryKey::RecordKeyProjection
```

or a persistent standalone product.

## 30.7 Pattern dependency ownership intuition

Signature keys affect the external callable contract; body pattern keys affect executable body/refinement semantics.

Thus:

```text
parameter/return computed key -> CallableSignature
Record pattern computed key   -> CallableBody
```

This gives correct targeted invalidation.

## 30.8 Builder scratch index versus semantic identity

A `HashSet<FieldKey>` inside `RecordLiteralBuilderObject` is an algorithmic scratch structure.

Permanent structural identity remains:

```text
canonical logical FieldKey sequence
```

The scratch set disappears after construction and therefore does not create a competing shape or semantic-key authority.

---

# 31. Required symbol disposition checklist

The final walkthrough must explicitly state what happened to each group.

## Preserve

```text
phalcom-semantic/src/field_key.rs
  FieldKey
  FieldKeyKind
  FieldKey::canonical_cmp
  FieldKey::stable_fingerprint

phalcom-semantic/src/types/row.rs
  RecordRowField
  RecordRowData
  RecordRowTail
  AnonymousRecordRemainder

phalcom-semantic/src/types/row_solver.rs
  RecordRowSolver

phalcom-semantic/src/types/relation.rs
  check_field_key_domain_membership
  UnorderedFieldLookup
  lookup_unordered_field

phalcom-semantic/src/static_value/store.rs
  StaticValueStore::intern_record
  StaticValueStore::field_key_of

phalcom-semantic/src/static_key.rs
  RecordKeyUseId
  ProjectedRecordKeySet
  StaticKeyProjection
  project_checked_static_key

phalcom-core/src/product/shape.rs
  RecordProductShape
  RecordProductShape::from_presentation_keys

phalcom-core/src/product/mod.rs
  value_to_field_key
  field_key_to_value
  compare_field_key_to_value
```

## Converge/refactor

```text
phalcom-semantic/src/static_constant_demand.rs
  StaticConstantDemandDriver
  force
  compute
  compute_with_projected_keys
  check_evaluate_publish
  recursive_cycle_outcome
  publish_recursive_cycle

phalcom-semantic/src/session.rs
  ConstantDeclInfo
  constant_graph
  constant_record_key_dependencies
  ConstantTarjan
  visit_constant
  service_type_member_demands
  ordinary alias/static-definition worklist

phalcom-semantic/src/db/query.rs
  SignatureQueryInputs
  ensure_formal_semantic_dependency_current
  query_field_signature_with_inputs
  query_callable_signature_with_inputs
  query_static_definition_with_record_key_projections
```

## Complete

```text
phalcom-ast/src/ast.rs
  RecordKeySyntax
  RecordPatternEntry

phalcom-semantic/src/checker/pattern.rs
  resolve_record_pattern

phalcom-semantic/src/match_semantics.rs
  ResolvedRecordFieldPattern

phalcom-core/src/compiler/lib/patterns.rs
  Record paths
  emit_record_rest_temp
```

## Harden

```text
phalcom-core/src/heap/record_literal_builder.rs
  RecordLiteralBuilderObject
  contains_key
  append
  take_entries

phalcom-semantic/src/metadata/export.rs
  NonExportableRecordKey
  NonExportableRecordKeyDomain
  legacy_record_field_name

phalcom-core/src/typing/inspect.rs
  legacy Record name -> FieldKey::symbol

phalcom-core/core/universe/src/collections/record.ph
  rawLabelAt
  rawKeyAt
  labelAt
  keyAt
  rawLookup
```

---

# 32. Forbidden implementation shortcuts

Do not:

```text
1. add another service_*_demands loop per owner;
2. retain both session and demand static-constant semantic pipelines;
3. let record_key_projections: None mean "form without the computed key";
4. persist ProjectedRecordKeySet using AST address identity;
5. add QueryKey::RecordKeyProjection;
6. evaluate key expressions in TypeResolver;
7. evaluate pattern keys in the compiler;
8. convert every Record pattern key to Value::symbol;
9. exclude rest fields by string spelling;
10. use LKG to break a current formation cycle;
11. place Symbol/string placeholders in canonical rows;
12. add a P4-specific unordered row solver;
13. widen Pack/Tuple/call labels to FieldKey;
14. stringify heterogeneous keys into legacy metadata;
15. call a P4 behavior "baseline blocked" without proving the failure is unrelated.
```

---

# 33. Recommended work-unit / commit decomposition

Suggested coherent units:

```text
1. p4: converge static constant semantic owner
2. p4: add exact owner orchestration and transactional retry
3. p4: canonicalize signature replay through owner preparation
4. p4: implement deferred computed-key static definitions
5. p4: close T7.0E acceptance and lifecycle gate
6. p4: resolve heterogeneous Record pattern keys
7. p4: lower exact Record pattern keys and rest
8. p4: harden dynamic Record builder complexity
9. p4: close metadata/API/incremental compatibility
10. p4: stabilize, document, and close lifecycle
```

Do not combine unrelated package/P10/C4.1 work into these commits.

---

# 34. Final architectural target

At completion, P4 should be explainable by one coherent model:

```text
                   ┌─────────────────────────┐
                   │       SemanticDb        │
                   │ currentness/deps/cache  │
                   └───────────┬─────────────┘
                               │
                   ensure exact QueryKey current
                               │
                   ┌───────────▼─────────────┐
                   │  Exact Owner Executor   │
                   │ stack/demand/reconcile  │
                   └───────┬────────┬────────┘
                           │        │
          ┌────────────────┘        └────────────────┐
          ▼                                          ▼
 canonical owner formation                 canonical owner formation
 StaticConstant / StaticDefinition         Field/Callable/Member
          │                                          │
          └─────────────────┬────────────────────────┘
                            ▼
                 ordinary checker facts
                            ▼
                    P2 StaticEvaluator
                            ▼
               StaticValueStore::field_key_of
                            ▼
                        FieldKey
                            ▼
                  existing TypeStore rows
                            ▼
              published owner + exact deps
                            ▼
           resolved body pattern semantic facts
                            ▼
                    compiler lowering
                            ▼
             heterogeneous Record runtime
```

There is:

```text
one FieldKey semantics
one row semantics
one static evaluator
one semantic owner per QueryKey
one persistent query/dependency authority
one exact owner orchestration seam
```

The completion criterion is this coherence across cold, incremental, pattern, and runtime paths—not merely acceptance of four literal key kinds.
