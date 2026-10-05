# LANG007.C3.P5 Full Implementation Audit

**Repository:** `aureat/phalcom-lang`  
**Audited repository revision:** `e4418ae7364f202f74116b30b58f66496381a860`  
**Last production-code revision in the audited P5 sequence:** `f7ddedf62840573fdfb4e9f19507cbcea2b573ee`  
**Audit date:** 2026-10-01  
**Checkpoint:** `LANG007.C3`  
**Plan:** `LANG007.C3.P5`  
**Audit type:** implementation correctness, semantic soundness, cross-layer completeness, runtime/type-system consistency, performance/resource review, verification/lifecycle audit  
**Audit disposition:** **NOT READY FOR P5 COMPLETION / NOT READY FOR G12 CERTIFICATION**

---

## 1. Executive summary

`LANG007.C3.P5` is a substantial implementation, not a superficial or mostly-stubbed patch. The repository contains real cross-layer work for exact/open structural typing, first-class structural member presence, anonymous Tuple/Record remainders, nested Dynamic generic slots, witness-result specialization, Iterator/Iterable source-contract cleanup, metadata schema v10, and rest-call environment forwarding. Several focused suites recorded by the implementation are meaningful, and important predecessor invariants were preserved.

However, the implementation is **not semantically complete or internally coherent enough to close P5**. This audit found multiple source-proven defects in the exact areas P5 was intended to make authoritative.

The most consequential findings are:

1. **Structural subtyping is unsound for presence:** an optional source field/member is accepted as satisfying a required target field/member in both Records and labeled Tuples.
2. **Structural remainder state is not closed under specialization/materialization:** generic Tuple remainders are silently dropped by two materialization paths, and Record anonymous typed remainders/captured tails are incompletely specialized or normalized.
3. **Dynamic applied-type relation is wrong:** a relation depending on `StaticArgument::DynamicType` is hard-refuted rather than returning the required `DynamicBoundary`.
4. **Semantic rest dispatch does not implement the normative exact-before-rest hierarchy rule:** a subclass rest method can be selected semantically before an inherited exact method that the VM would execute.
5. **Semantic rest acceptance does not enforce the required ordered labeled prefix.**
6. **T11's module-export forwarding path still erases conformance evidence:** the environment-aware helpers were added, but module-export exact and rest forwarding call environment-erasing wrappers.
7. **Runtime instantiation/reflection does not fully consume schema-v10 structural products:** `OpenTuple`/`AnonymousRecord` templates are not recursively specialized, and row-valued runtime arguments discard presence/remainder information.
8. **The focused evidence is materially incomplete relative to the plan:** Dynamic metadata durability has no direct test in the cited metadata suites; CV-26 incremental/tooling evidence is not demonstrated by the current walkthrough; the required module-export transport case is not covered by the green rest-dispatch test; and T12 still has a known 17/18 semantic generic-conformance result.

The current checkpoint's lifecycle state—`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`—is therefore correct. The problem is not that the repository prematurely marked P5 complete. The problem is that parts of the walkthrough/handoff describe individual tasks, especially T10/T11, more strongly than the actual implementation and evidence justify.

### Audit verdict

**P5 must remain `IN_PROGRESS / PARTIAL`.** Do not advance to T14 broad certification as a closure attempt until the source-proven G2/G3/G9/G10 defects are repaired and the T12 generic-conformance failure is resolved or causally dispositioned.

There is no evidence from this audit of memory unsafety or unsafe Rust corruption. The blocking issues are semantic soundness, semantic/runtime divergence, incomplete environment transport, durability/materialization loss, and verification completeness.

---

## 2. Audit basis and methodology

### 2.1 Authoritative inputs

The audit used the current repository state and explicitly reconciled the following P5 records rather than treating any one document as truth:

- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-requirements-analysis.md`
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-luna-patch-grade-implementation-plan.md`
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md`
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-walkthrough.md`
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-handoff.md`
- `docs/workflow/implementation-record-lifecycle-convention.md`
- `docs/specs/structural-products.md`
- `docs/specs/tuple-and-record-typing.md`
- relevant callable dispatch/argument specifications, especially:
  - `docs/specs/callables/dispatch.md`
  - `docs/specs/callables/arguments.md`

Per the repository lifecycle convention:

- the plan is intent;
- the walkthrough is retrospective evidence;
- the handoff is continuation context;
- the checkpoint is the durable current-state authority;
- live source and accepted normative specifications decide actual behavior.

### 2.2 P5 implementation boundary

The principal P5 implementation revisions audited were:

| Revision | Subject | Primary role |
|---|---|---|
| `7e6b9473` | `docs(lang007): plan structural typing closure` | plan/spec/lifecycle preparation |
| `cbe62b4c` | `feat(types): add structural remainders and presence semantics` | structural model, Dynamic slots, broad semantic integration |
| `887269af` | `fix(iteration): close Iterable and Tuple trait-default dispatch and guarded evidence failures` | Iterator/Iterable and evidence/lowering closure |
| `d6def47c` | `feat(metadata): persist structural presence and anonymous remainder state` | schema v10 durability |
| `42784e07` | `feat(vm): preserve conformance environment across specialized rest dispatch` | VM rest environment forwarding |
| `f7ddedf6` | `fix(semantic): resolve dispatch with fallback callable id` | semantic rest dispatch identity |
| `e4418ae7` | `docs(lang007): update C3 checkpoint, P5 handoff, and walkthrough records` | current lifecycle records |

Unrelated documentation-archive commits adjacent to this sequence were excluded from semantic findings.

### 2.3 Evidence classes used in this audit

**SOURCE-PROVEN** means the current source directly contradicts an accepted specification/invariant or visibly drops required state. No runtime execution is needed to establish the defect.

**VERIFICATION GAP** means the implementation may or may not be correct, but the current evidence does not prove the acceptance claim made by the plan/walkthrough.

**RESOURCE CONCERN** means source structure creates a plausible efficiency/memory issue, but no measured regression was available.

**LIFECYCLE DRIFT** means records are stale, ambiguous, or stronger than the evidence.

### 2.4 Execution limitation

This audit inspected the committed repository and its recorded test evidence through repository tooling. It did **not** have a checked-out build environment in which to independently execute Cargo. GitHub status/workflow status was also not available to this integration.

Accordingly:

- source-proven findings below are independent code-review findings;
- recorded pass counts are reported as **repository-recorded evidence**, not independently rerun results;
- no unexecuted command is represented as a pass.

This distinction is particularly important because P5 itself requires `INV-28`: a test that does not reach its discriminating assertion is not acceptance evidence.

---

## 3. Normative acceptance model used by the audit

The audit enforced P5-A through P5-J and `INV-01..INV-28`, with particular emphasis on these rules.

### 3.1 Structural semantics

- closed Tuple/Record structural types are exact;
- openness is explicit semantic state;
- optionality is presence, not value optionality;
- an optional source member **does not** satisfy a required target member;
- optional positional Tuple members do not exist;
- captured rows and anonymous remainders are distinct;
- Tuple positional/labeled lane openness is independent;
- typed/unconstrained remainders are first-class canonical shape state;
- only proper kind-`Type` terms participate in ordinary value subtyping after identity.

### 3.2 Dynamic applications

- bare Dynamic remains outside standalone `TypeId`;
- a proper applied type may contain `StaticArgument::DynamicType` in a `Type`-kind slot;
- `Dynamic`, `List<Dynamic>`, `List<Any>`, `List<Int>`, and unresolved `List<X>` remain distinct;
- use-site materialization is dependency-sensitive:
  - `T [T := Dynamic] -> Dynamic`;
  - `Box<T> [T := Dynamic] -> Box<Dynamic>`;
  - fixed result independent of `T` remains Known;
- relation/conformance must not manufacture static proof from Dynamic and must preserve a Dynamic boundary when the result depends on a Dynamic slot.

### 3.3 Callable/conformance ownership

- canonical inferred return remains provider-owned;
- witness specialization materializes the provider contract at use site;
- no second result authority or consumer-side re-inference;
- semantic analysis selects conformance evidence;
- compiler/VM project or transport evidence only.

### 3.4 Dispatch

The current callable dispatch spec is explicit:

> exact selector lookup completes through the entire eligible hierarchy before rest-capable lookup begins.

Thus an inherited exact Method beats a subclass rest Method.

Rest labeled matching is also explicit:

> fixed labels are an **ordered labeled prefix**, and rest captures only the suffix after that prefix.

### 3.5 Persistence

Every durable P5 distinction must survive metadata, import/runtime reflection where promised, fingerprinting, and incremental recomputation without collapse.

---

# 4. Findings summary

| ID | Severity | Class | Finding | Blocking gates |
|---|---|---|---|---|
| P5-AUD-001 | **HIGH** | SOURCE-PROVEN | Optional source member satisfies required target in Record and labeled-Tuple subtyping | G2, G11 |
| P5-AUD-002 | **HIGH** | SOURCE-PROVEN | Tuple remainder state is silently dropped by generic materialization paths | G1, G2, G4, G9, G11 |
| P5-AUD-003 | **HIGH** | SOURCE-PROVEN | Record anonymous typed/captured tail state is incompletely materialized/normalized | G1, G2, G4, G9 |
| P5-AUD-004 | **MEDIUM** | SOURCE-PROVEN | Canonical `TypeStore` can represent forbidden optional positional Tuple elements | G1, G2 |
| P5-AUD-005 | **HIGH** | SOURCE-PROVEN | Applied-type relation hard-refutes Dynamic slots instead of returning `DynamicBoundary` | G3, G5, G11 |
| P5-AUD-006 | **MEDIUM-HIGH** | SOURCE-PROVEN | Type-lambda application rejects `DynamicType` before ordinary kind/application handling | G3 |
| P5-AUD-007 | **HIGH** | SOURCE-PROVEN | Semantic dispatch violates exact-before-rest hierarchy precedence | G10, G11 |
| P5-AUD-008 | **HIGH** | SOURCE-PROVEN | Semantic rest matching ignores ordered fixed-label prefix | G10, G11 |
| P5-AUD-009 | **HIGH** | SOURCE-PROVEN | Module-export exact forwarding erases conformance environment | G10, G11 |
| P5-AUD-010 | **HIGH** | SOURCE-PROVEN | Module-export rest forwarding erases both specialized environment paths | G10, G11 |
| P5-AUD-011 | **HIGH** | SOURCE-PROVEN | Runtime template instantiation does not recursively specialize schema-v10 open structural forms | G9 |
| P5-AUD-012 | **MEDIUM** | SOURCE-PROVEN | Runtime row reflection drops presence and anonymous remainder state | G9 |
| P5-AUD-013 | **MEDIUM** | SOURCE-PROVEN | Runtime structural presentation is incomplete/noncanonical for v10 forms | G9 |
| P5-AUD-014 | **MEDIUM** | SOURCE-PROVEN / VALIDATION | Dynamic metadata marker is not fenced by the schema-v10 compatibility floor | G9 |
| P5-AUD-015 | **HIGH** | VERIFICATION GAP | CV-15 Dynamic metadata durability is claimed but has no direct Dynamic metadata test | G9, G11 |
| P5-AUD-016 | **HIGH** | VERIFICATION GAP | T10 is recorded complete without explicit CV-26 incremental/tooling acceptance evidence | G9, G11 |
| P5-AUD-017 | **HIGH** | VERIFICATION GAP / OPEN FAILURE | T12 generic-conformance matrix is still 17/18 with owner/method-generic evidence publication unresolved | G4, G11 |
| P5-AUD-018 | **HIGH** | VERIFICATION GAP | Green rest-dispatch test covers ordinary rest dispatch, not required module-export forwarding | G10, G11 |
| P5-AUD-019 | **LOW** | PLAN DEVIATION | Dead/misleading TypePack subtype arms remain after the proper-Type gate | G2 hygiene |
| P5-AUD-020 | **MEDIUM** | RESOURCE CONCERN | Structural relation performs repeated allocation and O(n×m) labeled/row scans despite canonical ordering | resource evidence |
| P5-AUD-021 | **MEDIUM** | VERIFICATION GAP | Required P5 incremental/resource observations are absent from current completion evidence | G9/resource |
| P5-AUD-022 | **LOW** | LIFECYCLE DRIFT | P5 handoff revision anchor is stale/ambiguous and some task evidence wording is stronger than proven | T15 |

No Critical severity is assigned because this audit did not identify memory corruption, unsafe Rust misuse, or an unconditional VM crash. Several HIGH findings are nevertheless checkpoint-blocking semantic correctness defects.

---

# 5. Detailed findings

## P5-AUD-001 — Optional source member satisfies required target

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** structural relation  
**Violates:** P5-B, INV-03, T5 relation contract, structural-products §3.2, tuple-and-record-typing §10.1

### Required behavior

For a required target member `k: T`, the source must **provably contain** `k`.

Therefore:

```text
{ k?: Int } </: { k: Int }

(k?: Int) </: (k: Int)
```

An optional source declaration does not establish presence.

### Current implementation

In both structural relation paths, the implementation first finds a same-name source member and then checks only its value type.

For Records, `phalcom-semantic/src/types/relation.rs::check_record_row_subtype` does:

```text
if matching source field exists:
    compare source_field.ty <: target_field.ty
else if target is required:
    refute
```

It never checks that the matching source field itself is `Required`.

The labeled-Tuple branch has the same shape: a same-label source element goes directly to value-type comparison regardless of source presence.

### Consequence

A value satisfying `{k?: Int}` may omit `k`, yet the type relation can claim it satisfies `{k: Int}`. That is a direct structural-presence soundness error.

Any consumer relying on the required target may subsequently access a member that the source contract did not guarantee.

### Why existing CV-04 did not catch it

The current direct Record test covers:

- required source -> optional target, compatible;
- required source -> optional target, incompatible value type.

It does not test:

- optional source -> required target.

The implementation's CV-04 coverage is therefore directional rather than complete.

### Required correction

For target required members:

```text
matching source member must exist
AND source presence must be Required
AND source type <: target type
```

Add the symmetric negative for both Record and labeled Tuple.

### Required regressions

```text
record_optional_source_does_not_satisfy_required_target
tuple_optional_labeled_source_does_not_satisfy_required_target
required_source_satisfies_optional_target
optional_source_satisfies_optional_target_when_value_type_compatible
optional_source_refutes_optional_target_when_value_type_incompatible
```

---

## P5-AUD-002 — Tuple anonymous remainder state is dropped by generic materialization

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** generic specialization / structural type representation  
**Violates:** INV-05, P5-REQ-014, T3/T5/T6 shape preservation

### Current implementation: checked instantiation

In `phalcom-semantic/src/types/instantiation.rs`, `materialize_type_inner` handles `TypeData::Tuple` by materializing the element types and rebuilding with:

```rust
store.tuple(...)
```

`TypeStore::tuple` is the closed-Tuple constructor:

```text
tuple(elements)
    -> open_tuple(elements, None, None)
```

Therefore both:

- `positional_remainder`
- `labeled_remainder`

are silently discarded.

### Current implementation: lazy `TypeView`

`phalcom-semantic/src/types/environment.rs::materialize_view` repeats the same mistake:

```text
TypeData::Tuple
    -> materialize elements
    -> store.tuple(...)
```

Again, both remainder lanes are closed.

### Contrast with the correct path

`phalcom-semantic/src/types/substitution.rs` correctly reconstructs a Tuple with `store.open_tuple(...)` and recursively substitutes typed remainder element types.

`phalcom-semantic/src/types/projection.rs` likewise explicitly carries both Tuple remainders through normalization.

The repository therefore has **three semantic reconstruction paths with different behavior**.

### Consequence

A valid generic contract such as:

```text
(T, *: T)
```

or a Tuple with independent labeled openness can become a closed Tuple after receiver/generic materialization.

This can:

- make a previously valid subtype/call fail;
- change canonical identity and fingerprint behavior;
- alter witness specialization;
- make cold/substitution behavior differ from TypeView specialization;
- cause metadata/incremental parity failures.

The defect is especially important because P5 witness specialization uses `TypeView::materialize`.

### Required correction

All canonical Tuple reconstruction paths must preserve:

```text
elements
positional_remainder
labeled_remainder
```

and recursively materialize any `Typed(TypeId)` remainder.

The existing substitution implementation is a good behavioral reference.

### Required regressions

Extend the existing law:

```text
materialize(view(t, ρ)) == substitute(t, ρ)
```

to:

- positional typed remainder;
- labeled typed remainder;
- both lanes open;
- unconstrained remainders;
- optional labeled members plus remainders.

---

## P5-AUD-003 — Record anonymous/captured tails are incompletely materialized and normalized

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** record rows, TypeView, checked instantiation, projection normalization  
**Violates:** INV-05, P5-H, P5-REQ-013, T3/T5/T6

### Lazy TypeView

`phalcom-semantic/src/types/environment.rs::materialize_view` materializes Record field types but passes the row tail through unchanged.

That is wrong for at least:

```text
Anonymous(Typed(T))
```

when `T` is bound in the current environment.

It also does not consume `TypeEnvironment::row_bindings` to materialize a captured `RecordRowTail::Parameter` into its bound row.

### Checked generic instantiation

`materialize_row_tail` in `types/instantiation.rs` only recursively resolves:

```text
RecordRowTail::Parameter
```

All other tails are returned unchanged.

Therefore an anonymous typed remainder whose element type contains a declaration parameter is not recursively materialized.

### Projection normalization

`types/projection.rs::normalize_type` normalizes Record fixed fields but rebuilds with the original `row.tail` unchanged. A typed anonymous remainder containing a normalizable projection remains stale.

### Consequence

P5 made anonymous remainders canonical shape data, but the shape is not closed under the compiler's own specialization/normalization operations.

This creates cold/use-site/incremental disagreement risk and can leave declaration-owned parameters or projections inside an otherwise-specialized Record contract.

### Required correction

Introduce one canonical helper for recursively transforming Record rows, including:

- each field type;
- `Anonymous(Typed(T))`;
- `Projection(T)`;
- captured row parameter expansion when the operation is materialization and a row binding exists;
- correlation/lacks behavior exactly as owned by the row solver.

Do not make anonymous tails into row variables.

### Required regressions

```text
type_view_materializes_anonymous_typed_record_remainder
checked_instantiation_materializes_anonymous_typed_record_remainder
projection_normalization_normalizes_anonymous_typed_record_remainder
type_view_materializes_captured_row_binding_without_losing_correlation
```

---

## P5-AUD-004 — Canonical TypeStore can represent forbidden optional positional Tuple elements

**Severity:** MEDIUM  
**Class:** SOURCE-PROVEN  
**Subsystem:** TypeStore invariants

### Correct parser behavior

The parser explicitly rejects optional positional Tuple type members.

This is correct.

### Canonical representation problem

`TupleTypeElement` exposes:

```rust
optional_unlabeled(...)
```

which creates:

```text
label = None
presence = Optional
```

`TypeStore::open_tuple` checks:

- proper element type;
- label ordering;
- duplicate labels;
- remainder kinds;

but does not reject optional unlabeled elements.

### Metadata validator contrast

The schema-v10 metadata validator **does** reject an `OpenTuple` with an optional element whose label is `None`.

Thus the semantic store can create a canonical state that its own persistence validator declares malformed.

### Consequence

No current production caller of `optional_unlabeled` was found, so this is not presently proven source-reachable. It is nevertheless a canonical invariant breach and future footgun.

The positional relation path also ignores presence, meaning such an internally-created state would have undefined/incorrect subtype semantics.

### Required correction

- remove or make `optional_unlabeled` impossible;
- reject optional unlabeled entries in `TypeStore::open_tuple`;
- retain metadata validation as defense in depth.

---

## P5-AUD-005 — `StaticArgument::DynamicType` is hard-refuted by applied-type relation

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** subtype/assignability relation  
**Violates:** INV-10, INV-13, T6 relation contract

### Required behavior

P5 explicitly requires:

> when a relation step depends on a Dynamic argument, return a Dynamic boundary outcome rather than Proven or an invented refutation.

For example, comparing same-head covariant forms such as:

```text
List<Dynamic>
List<Int>
```

depends on the Dynamic slot.

### Current implementation

In the `TypeData::Applied` same-origin relation branch:

1. each static argument is converted through `as_type_id()`;
2. `DynamicType` has no `TypeId`;
3. when either argument is non-term:
   - equal static arguments are simply accepted;
   - unequal static arguments return `TypeMismatch`.

Thus:

```text
DynamicType vs Term(Int)
    -> Refuted
```

instead of:

```text
DynamicBoundary
```

### Consequence

This is not merely a diagnostic classification issue. It changes:

- assignment compatibility;
- generic constraint discharge;
- call inference;
- conformance applicability;
- relation composition.

A program that should cross a runtime Dynamic boundary can instead be statically rejected.

### Required correction

The applied relation must explicitly classify static-argument pairs:

```text
Term / Term
RecordRow / RecordRow
DynamicType / anything Type-kind
```

and return a bounded Dynamic obligation when the declaration's variance-relevant comparison depends on a Dynamic slot.

Do not map Dynamic to Any and do not make it a synthetic `TypeId`.

### Required regressions

```text
covariant_dynamic_argument_yields_dynamic_boundary
contravariant_dynamic_argument_yields_dynamic_boundary
invariant_dynamic_argument_yields_dynamic_boundary_when_compared_with_exact_term
unrelated_nominal_heads_remain_nominally_refuted
```

---

## P5-AUD-006 — Type-lambda application rejects Dynamic slots before normal kind/application handling

**Severity:** MEDIUM-HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** higher-kinded type application  
**Violates/risks:** P5-E/G generality, T4 “equivalent source forms”

### Current implementation

`TypeStore::apply_static_form_with_arena` special-cases `TypeData::Lambda` before the general heterogeneous argument-kind path.

For a type lambda, every argument is required to match:

```rust
StaticArgument::Term(...)
```

Any `StaticArgument::DynamicType` immediately yields:

```text
TypeApplicationError::MalformedLambda
```

The later generic kind logic, which correctly treats `DynamicType` as kind `Type`, is never reached.

### Why this matters

P5's architecture describes the Dynamic marker as a generic/static argument category legal in `Type`-kind parameter positions. A type lambda is also a Type-kind constructor.

The current implementation therefore supports nominal generic heads such as `List<Dynamic>` but not the corresponding Type-lambda application path.

### Disposition

This is a confirmed limitation in the canonical application API. The audit did not establish a currently exposed surface spelling that necessarily exercises this exact case, so severity is below the directly user-visible relation defects.

P5 should either:

1. implement Dynamic-aware type-lambda application/materialization; or
2. explicitly narrow the normative rule and document why type-lambda application is excluded.

Silent `MalformedLambda` is not an adequate final architecture.

---

## P5-AUD-007 — Semantic dispatch violates exact-before-rest hierarchy precedence

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** semantic dispatch  
**Violates:** `DSP-EXACT-BEFORE-REST`, runtime/semantic parity, P5-J evidence identity

### Normative rule

Dispatch is two-pass:

```text
exact selector through complete eligible hierarchy
    ↓ complete miss
rest-compatible method through hierarchy
```

An inherited exact Method therefore beats a subclass rest fallback.

### Runtime behavior

The VM follows this model:

- exact `lookup_method` first;
- only after miss, `lookup_rest_method`.

### Semantic behavior

`MemberSurface::get_callable`:

1. checks an exact selector in the current surface;
2. otherwise scans that **same surface** for a rest-capable signature accepting the invocation selector.

`SurfaceDispatchResolver::resolve_dispatch_with_trace` then walks owners from derived to base and returns the first `get_callable` result.

Therefore:

```text
Child: compatible rest method
Parent: exact method
```

selects the Child rest method semantically before the resolver ever reaches Parent.

The VM executes Parent exact.

### Consequence

This is a direct semantic/runtime method-identity divergence.

Possible symptoms include:

- wrong inferred return type;
- wrong generic bindings/evidence recipe;
- access/provenance mismatch;
- VM specialized-call identity validation failure;
- behavior that typechecks against one callable and executes another.

### Required correction

Semantic resolution must mirror runtime/spec phases:

```text
pass 1: exact only across owners
pass 2: rest compatibility across owners
```

Do not make the per-owner exact lookup API transparently rest-aware during the first pass.

### Required regression

Construct:

```text
Parent.exact foo(_)
Child rest foo(*xs)
```

on a Child receiver and prove:

- semantic `CallableId` is Parent exact;
- lowering recipe references Parent exact;
- VM executes Parent exact.

---

## P5-AUD-008 — Semantic rest matching ignores ordered fixed-label prefix

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** selector acceptance / rest dispatch  
**Violates:** callable arguments specification

### Required rule

For labeled or split/complete rest:

```text
L starts with K
```

where `K` is the ordered fixed-label prefix.

### Current implementation

`CallableSignature::accepts_selector` collects fixed labels and incoming labels, then checks:

```text
every fixed label is contained somewhere in incoming labels
```

with count checks.

This is set-like containment, not ordered-prefix matching.

### Incorrectly accepted shapes

A declaration expecting:

```text
mode, **options
```

can semantically accept labels where:

- `mode` appears after another residual label;
- fixed labels are permuted;
- in non-rest-labeled cases, the same label set is reordered.

Runtime shape validation is stricter, creating another semantic/runtime divergence.

### Required correction

Compare the fixed label sequence directly against the incoming prefix:

```text
labels[..fixed_labels.len()] == fixed_labels
```

then apply rest/non-rest suffix rules.

Add duplicate-label defenses consistent with existing selector construction.

---

## P5-AUD-009 — Module-export exact forwarding erases conformance environment

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** VM module export forwarding  
**Violates:** INV-23/24, P5-J, T11

### Current path

`invoke_at` materializes:

- `type_environment`;
- `conformance_environment`.

Before ordinary dispatch it calls:

```text
try_module_export_send_with_environment(...)
```

The module-export helper receives:

- type environment;
- the call recipe;

but **not the already-materialized conformance environment ID**.

On an exact exported target it validates the specialized recipe and then calls:

```text
call_method_with_selector_as_and_environment(...)
```

The singular helper intentionally delegates to the plural helper with:

```text
RuntimeConformanceEnvironmentId::EMPTY
```

### Consequence

Any exported generic callable whose body requires the semantic conformance evidence from the caller loses it at module-export forwarding.

This is exactly the class of issue T11 was meant to close.

### Required correction

Thread the materialized conformance environment into the module-export helper and enter exact exported calls through:

```text
call_method_with_selector_as_and_environments(...)
```

with both environments.

---

## P5-AUD-010 — Module-export rest forwarding also erases specialized environments

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** VM rest/module forwarding  
**Violates:** INV-23/24, P5-J, T11

### Current implementation

The rest fallback inside `try_module_export_send_with_environment` calls:

```text
activate_rest_method(...)
```

The non-environment overload installs empty type and conformance environments.

The environment-aware helper added by `42784e07` exists:

```text
activate_rest_method_with_environment(...)
```

but is not used here.

### Consequence

The ordinary specialized rest-send path is repaired, but the module-export rest path still loses the environment.

This means T11 implemented only part of the required topology.

### Required correction

Module-export rest forwarding must use the environment-aware activation path with the exact same specialized type/conformance environments established at the semantic call site.

Do not reconstruct proof in the VM.

---

## P5-AUD-011 — Runtime template instantiation ignores schema-v10 open structural forms

**Severity:** HIGH  
**Class:** SOURCE-PROVEN  
**Subsystem:** runtime typing/materialization  
**Violates:** P5-F, INV-25, T6/T10 durability

### Current implementation

`phalcom-core/src/typing/environment.rs::instantiate_type_ref` recursively instantiates metadata forms such as:

- Parameter;
- Applied;
- Union;
- legacy closed Tuple;
- legacy closed Record;
- Callable.

But schema-v10 forms:

- `TypeNode::OpenTuple`;
- `TypeNode::AnonymousRecord`;

are not handled and fall through to:

```text
_ => Some(ty)
```

`OpenRecord` similarly does not receive recursive specialization in this path.

### Consequence

Suppose an exported generic runtime type contract contains:

```text
(T, *: T)
```

or:

```text
{value: T, **: T}
```

Reifying the contract under `T = Int` can retain the original metadata node rather than producing a runtime type representation specialized to `Int`.

Metadata storage is therefore richer than runtime template instantiation.

### Required correction

Add runtime overlay forms or an equivalent bounded representation capable of preserving:

- element/field presence;
- tuple lane remainders;
- anonymous record remainder;
- recursively instantiated child types.

Do not silently return the unspecialized template when an active runtime type environment exists.

---

## P5-AUD-012 — Runtime row reflection drops structural presence and anonymous remainder state

**Severity:** MEDIUM  
**Class:** SOURCE-PROVEN  
**Subsystem:** runtime typing overlay/reflection  
**Violates:** INV-25, T6 runtime/reflection contract

### Current runtime row representation

`RuntimeRecordField` contains:

```text
name
ty
```

but no presence.

`RuntimeRecordRow` contains:

```text
fields
tail
```

but no anonymous remainder.

### Metadata-to-runtime projection

When `inspect::static_arguments` converts a metadata `RecordRowRef` into a runtime row argument, it copies:

- names;
- types;
- captured tail;

but omits:

- `RecordFieldRef.presence`;
- `RecordRowRef.remainder`.

### Consequence

Durable forms such as:

```text
{payload?: Int, ...}
{payload: Int, **: String}
```

collapse to a less precise runtime row descriptor.

This is particularly inconsistent because metadata schema v10 explicitly preserves those distinctions.

### Required correction

Extend `RuntimeRecordField` and `RuntimeRecordRow` so runtime reflection retains the canonical structural contract.

---

## P5-AUD-013 — Runtime structural presentation is incomplete/noncanonical

**Severity:** MEDIUM  
**Class:** SOURCE-PROVEN  
**Subsystem:** reflection/presentation

Examples from `phalcom-core/src/typing/inspect.rs`:

1. Optional OpenTuple labeled element is rendered after the type, effectively like:

   ```text
   name: String?
   ```

   while source syntax is:

   ```text
   name?: String
   ```

2. Typed/unconstrained Tuple remainder rendering uses ad-hoc spellings such as:

   ```text
   ...T
   **...T
   ```

   rather than the accepted syntax:

   ```text
   *: T
   **: T
   ```

3. Captured `OpenRecord` field display does not include optional presence.

4. Row-valued applied arguments cannot display anonymous remainder state because the runtime row projection discarded it earlier.

### Consequence

This is not just aesthetic. P5 explicitly requires presentation not to collapse new semantic distinctions.

### Required correction

Presentation should be a faithful projection of canonical semantics and use one accepted spelling per semantic shape.

---

## P5-AUD-014 — Dynamic metadata marker is not fenced to the schema version that introduced it

**Severity:** MEDIUM  
**Class:** SOURCE-PROVEN / VALIDATION  
**Subsystem:** metadata compatibility

### Current compatibility floor

For metadata schema `< 10`, validation explicitly rejects:

- `OpenTuple`;
- `AnonymousRecord`;
- optional structural presence;
- anonymous structural row tails.

But the same pre-v10 floor does **not** reject:

```text
StaticArgumentRef::DynamicType
```

The validator's kind machinery fully recognizes the marker.

### Why this is a compatibility defect

`DynamicType` is a P5 metadata-form addition. A bundle labeled with an older schema can currently carry the new enum variant and be accepted by the current validator, even though an implementation that truly understands only that older schema would not know the variant.

Schema versioning should fence all representation additions whose wire shape is new.

### Required correction

Add the Dynamic static-argument marker to the appropriate schema floor and add a compatibility discriminator.

---

## P5-AUD-015 — CV-15 Dynamic metadata durability is claimed without direct Dynamic metadata coverage

**Severity:** HIGH  
**Class:** VERIFICATION GAP

The P5 walkthrough maps CV-15 to schema-v10 round-trip evidence.

The actual new metadata tests cover:

- structural member presence;
- Tuple remainders;
- anonymous Record remainders;
- structural fingerprint differences.

A search of both:

- `phalcom-type-meta/tests/schema_compat.rs`;
- `phalcom-semantic/tests/semantic/integration/metadata.rs`;

shows no direct `DynamicType` / `List<Dynamic>` metadata test.

### Consequence

The structural half of CV-15 has evidence. The Dynamic applied-type half does not.

### Required test

Export, encode, validate, decode, load/present:

```text
List<Dynamic>
```

and assert:

- the outer List head is retained;
- argument is exactly `StaticArgumentRef::DynamicType`;
- it differs from `List<Any>`;
- schema compatibility floor works;
- runtime reflection reports the Dynamic slot without treating it as a proper type argument.

---

## P5-AUD-016 — T10 completion claim lacks explicit CV-26 incremental/tooling evidence

**Severity:** HIGH  
**Class:** VERIFICATION GAP

### Plan contract

T10 is not “metadata schema v10.” It is:

> metadata, incremental, source-index, and LSP persistence for P5 types.

Its required coverage includes:

- CV-15 metadata;
- CV-26 cold/import/incremental equivalence;
- CV-27 predecessor lifecycle;
- direct LSP/source-index tests;
- fingerprint/invalidation checks;
- bounded incremental resource observations.

### Current walkthrough/handoff evidence

T10 is described as completed using:

- 17/17 schema compatibility tests;
- 31/31 semantic metadata tests.

No explicit P5 CV-26 cold/incremental discriminator is recorded in the current P5 walkthrough/handoff, and repository search does not identify a P5 structural-presence/Dynamic-slot incremental test.

No direct P5 LSP/source-index test is cited.

### Disposition

This does not prove the incremental implementation is broken. It proves the current evidence is insufficient to label the full T10 contract accepted.

T10 should remain partially verified until the missing gates are executed and recorded.

---

## P5-AUD-017 — T12 semantic generic-conformance matrix still has a known failure

**Severity:** HIGH  
**Class:** VERIFICATION GAP / OPEN FAILURE

The P5 handoff records:

```text
capabilities::generic_conformance
17/18 PASS
```

with the active case:

```text
receiver_owner_evidence_is_published_without_owner_inference
```

The test expects:

- `c.show()` to publish one owner conformance/binding;
- `c.combine(User.new())` to publish two:
  - receiver-owner `Container<T>` evidence;
  - method-generic `U` evidence;
- generic forwarding to retain forwarded owner evidence.

This is a high-value test because it exercises precisely the boundary between receiver specialization, callable specialization, and conformance-environment construction.

### Disposition

Until the failure is explained and corrected/dispositioned, G4/G11 remain open.

A 17/18 aggregate is not completion evidence.

---

## P5-AUD-018 — Current green rest test does not exercise module-export transport

**Severity:** HIGH  
**Class:** VERIFICATION GAP

The cited focused test:

```text
p4_specialized_rest_dispatch_preserves_conformance_environment
```

creates:

```text
Forwarder.show<T>(item, *rest) where T |= Printable
```

and calls it directly as an ordinary class-side rest send.

It proves the ordinary environment-aware rest activation path added by P5.

It does **not**:

- send through a Module export;
- exercise `try_module_export_send_with_environment`;
- exercise the exact exported-call environment erasure;
- exercise the exported rest activation wrapper.

Thus the green 11/11 `language::generic_conformance` result cannot establish the full T11 plan objective.

---

## P5-AUD-019 — Dead/misleading TypePack subtype arms remain

**Severity:** LOW  
**Class:** PLAN DEVIATION / MAINTAINABILITY

P5-REQ-022 and T5 explicitly required removing or quarantining ordinary subtype match arms for pack/static domains.

The relation correctly performs a kind-`Type` gate before the main match. Equal non-Type terms also return through identity first.

Therefore the later `TypePack` and `LabeledTypePack` equality arms are effectively unreachable for unequal terms and unnecessary for equal terms.

This is not a present soundness error because the kind gate protects the relation. It is nevertheless exactly the misleading code P5 planned to remove.

Delete or relocate it to a domain-specific equality helper.

---

## P5-AUD-020 — Structural relation has avoidable O(n×m) scans and repeated allocations

**Severity:** MEDIUM  
**Class:** RESOURCE CONCERN  
**Subsystem:** type relation

### Tuple relation

Every Tuple-vs-Tuple check allocates four temporary vectors:

```text
sub_pos
sup_pos
sub_lab
sup_lab
```

Labeled matching then repeatedly searches with `find`/`any`.

### Record relation

Record rows are already canonical and sorted by field name.

Despite that, relation code:

- scans all source fields for every target field;
- scans all target fields for every source field.

This is O(n×m) rather than a linear merge over two sorted sequences.

### Consequence

For small source structural types this is negligible. For generated/wide structural products, nested relation calls can amplify the cost, especially under union/generic relation recursion.

No measured regression is available, so this is not a correctness blocker by itself.

### Recommendation

After correctness repair, use:

- slices/indices rather than allocating lane vectors where practical;
- linear ordered merge for Record fields;
- a label-index or ordered merge strategy for Tuple labeled lanes if lane ordering permits.

Do not add a cache until measured evidence shows it is needed.

---

## P5-AUD-021 — Required P5 resource/incremental observations are not recorded

**Severity:** MEDIUM  
**Class:** VERIFICATION GAP

P5 §21 requires at least one representative incremental observation recording:

- number of callable bodies recomputed;
- number of conformance plans recomputed/refreshed;
- whether a structural/Dynamic edit invalidates only dependents;
- whether unrelated callable bodies are reused.

It also requires observing that equivalent `List<Dynamic>` forms are interned rather than duplicated per expression/call.

The current P5 walkthrough does not contain those observations.

This is a completion-evidence gap even if implementation performance is acceptable.

---

## P5-AUD-022 — Lifecycle records contain revision/evidence drift

**Severity:** LOW  
**Class:** LIFECYCLE DRIFT

The P5 handoff committed at repository HEAD `e4418ae7` still says:

```text
Remote HEAD: f7ddedf6
```

`f7ddedf6` is the latest production-code revision before the documentation-only lifecycle update, so the likely intended meaning is understandable. It is nevertheless ambiguous.

Use separate fields:

```text
record_revision:
production_revision:
```

or equivalent.

There are also places where T10/T11 wording says “completed” while the acceptance evidence establishes only narrower slices, as described in P5-AUD-015/016/018.

T15 must correct those records after the code/evidence issues are resolved.

---

# 6. Additional design and implementation observations

## 6.1 `TypeEnvironment` has mutually inconsistent binding stores if a parameter is rebound

`TypeEnvironment` maintains separate:

- exact type bindings;
- Dynamic bindings;
- row bindings.

`bind_param` does not visibly remove a Dynamic binding for the same parameter, and `bind_dynamic` does not visibly remove an exact binding.

Different materialization APIs test those stores in different order:

- bare exact materialization can prefer an exact binding;
- knowledge-bearing materialization can prefer Dynamic;
- nested applied materialization explicitly checks Dynamic.

No current source path was proven to create conflicting bindings, so this is not elevated to a blocking finding. It should nevertheless be hardened:

```text
one parameter has exactly one binding category in an environment
```

with debug assertions or mutually-exclusive insertion.

## 6.2 `materialize_checked` cannot represent Dynamic bindings

`TypeEnvironment::to_generic_instantiation()` carries exact type and row bindings, not Dynamic bindings.

`materialize_checked` delegates through that exact-only representation.

No external caller of `materialize_checked` was found, so this is currently a latent API inconsistency rather than a demonstrated failure. If the checked path becomes active, it must either support Dynamic or explicitly document that it is exact-only and reject environments containing Dynamic bindings.

---

# 7. Positive audit findings

The audit also found important areas where the P5 architecture is implemented coherently.

## 7.1 Structural authority is reconciled

`structural-products.md` is now explicitly primary semantic authority and `tuple-and-record-typing.md` defers to it while supplying synchronized source notation.

This resolves the original specification-ownership defect.

## 7.2 Parser enforces several key structural well-formedness rules

The parser includes explicit handling for:

- optional labeled structural members;
- anonymous Tuple/Record remainders;
- independent tuple lanes;
- rejection of optional positional members;
- duplicate/ordering constraints.

The source grammar is materially more complete than before P5.

## 7.3 Metadata v10 structural validation is substantive

The validator correctly checks, among other things:

- OpenTuple kind;
- child topology/kinds;
- optional positional element rejection;
- Tuple remainder type kinds;
- AnonymousRecord ordering/duplicates;
- anonymous remainder type kind;
- pre-v10 rejection of the new structural forms/presence.

P5-AUD-014 is a narrow Dynamic-marker schema-floor omission, not a dismissal of schema-v10 validation as a whole.

## 7.4 Structural fingerprints include new v10 distinctions

Metadata fingerprint code includes:

- field/member presence;
- anonymous remainder category/type;
- Tuple lane remainder state;
- `StaticArgumentRef::DynamicType`.

This is the correct architectural direction for INV-26.

## 7.5 Exact substitution path preserves new structural state

`types/substitution.rs` correctly:

- carries Dynamic static argument markers;
- maps Tuple typed remainders;
- rebuilds with `open_tuple`;
- maps anonymous Record typed remainders.

This gives a concrete in-repository reference for repairing the inconsistent materialization paths.

## 7.6 Dynamic nominal-head formation tests are meaningful

The P5 type-annotation tests directly establish:

- `List<Dynamic>` is a ready applied type;
- it retains List origin;
- its argument is `StaticArgument::DynamicType`;
- it differs from `List<Object>`/ordinary type arguments;
- Dynamic is rejected for a non-Type-kind parameter;
- bare/nested/fixed TypeView materialization distinguishes the required cases.

The failure is downstream completeness/relation, not the core nominal-head formation idea.

## 7.7 Proper-Type gate remains intact

`check_subtype_impl` continues to reject non-kind-`Type` terms after the identity fast path.

P5 did not solve its pack convenience problem by weakening `ProperTypeId` semantics.

## 7.8 Ordinary specialized rest transport was genuinely repaired

The VM now has environment-aware rest activation/call helpers, and ordinary specialized rest dispatch uses them.

The focused `Forwarder.show<T>(..., *rest)` test is a meaningful positive discriminator.

The problem is incomplete coverage of other forwarding topology, particularly Module exports.

## 7.9 Iterator source contracts are materially better formed

The canonical Universe source now declares explicit Iterator bounds and callable result contracts for many adapters.

Examples audited include:

- `MappedIterator`;
- `FilterIterator`;
- `FilterMapIterator`;
- `FlatMapIterator`;
- `FlattenIterator`;
- `ChainIterator`;
- `Skip*`;
- `Take*`;
- `MapWhileIterator`;
- `InspectIterator`;
- `StrictZipIterator`;
- `IndexedIterator`.

The accepted Chain equality rule is retained, `toMap` has one advance per loop iteration, and `TakeIterator`/`SkipIterator` validate non-negative integral bounds.

## 7.10 The checkpoint does not falsely claim P5 completion

This is important. The canonical C3 checkpoint currently says P5 is:

```text
IN_PROGRESS / PARTIAL / FOCUSED_TESTED
```

and identifies T12 as in progress.

That lifecycle truth is consistent with this audit's overall disposition.

---

# 8. CV coverage audit

| CV | Audit status | Notes |
|---|---|---|
| CV-01 closed structural negative | **SUPPORTED** | direct Record exactness test exists |
| CV-02 explicit open positive | **SUPPORTED** | direct Record test |
| CV-03 optional absence | **SUPPORTED** | direct Record test |
| CV-04 optional presence | **INCOMPLETE / FAILING LAW** | missing optional-source -> required-target negative exposes P5-AUD-001 |
| CV-05 captured vs anonymous | **PARTIALLY SUPPORTED** | representation distinction exists; full transformation closure affected by P5-AUD-003 |
| CV-06 typed remainder | **PARTIALLY SUPPORTED** | direct relation case exists; materialization/normalization defect remains |
| CV-07 Tuple lane independence | **PARTIALLY SUPPORTED** | source representation/relation present; transformation loss P5-AUD-002 blocks full claim |
| CV-08 canonicalization | **PARTIAL** | metadata/interner work present; impossible optional positional state remains |
| CV-09 proper-Type boundary | **SUPPORTED** | kind gate intact |
| CV-10 `List<Dynamic>` formation | **SUPPORTED** | direct formation test |
| CV-11 Dynamic kind safety | **SUPPORTED for nominal/static applications** | type-lambda hole remains |
| CV-12 dependent Dynamic result | **SUPPORTED in TypeView test** | downstream applied relation still wrong |
| CV-13 nested dependent result | **SUPPORTED in nominal-head TypeView test** | type-lambda path unsupported |
| CV-14 fixed result remains Known | **SUPPORTED** | direct TypeView test |
| CV-15 metadata round trip | **PARTIAL** | structural v10 covered; Dynamic slot metadata not directly covered |
| CV-16 witness specialization | **PARTIAL** | implementation present; T12 owner evidence failure prevents closure |
| CV-17 witness negative | **PARTIAL** | focused predecessor controls exist; G11 not closed |
| CV-18 projection legality | **SUPPORTED substantially** | Iterator source bounds improved |
| CV-19 mapper U inference | **SUPPORTED substantially** | typed callback contracts present |
| CV-20 adapter semantics | **SUPPORTED substantially** | canonical source contracts retained |
| CV-21 toMap single advance | **SUPPORTED by source and recorded iteration suite** | loop source advances once per logical entry |
| CV-22 natural Range chain | **RECORDED PASS, not independently rerun** | 24/24 iteration suite recorded |
| CV-23 bootstrap ledger closure | **RECORDED PASS, not independently rerun** | checkpoint claims prior 20+3 eliminated |
| CV-24 evidence recipe exists | **SUPPORTED for ordinary rest path** | module-export topology still not proven |
| CV-25 forwarding transport | **FAIL/PARTIAL** | ordinary rest passes; module-export exact/rest still erase env |
| CV-26 cold/import/incremental equivalence | **NOT ADEQUATELY EVIDENCED** | T10 records do not show required P5 discriminator |
| CV-27 P4.R1 lifecycle regression | **PARTIAL** | focused predecessor evidence exists; T12/G11 incomplete |
| CV-28 failure classification | **IN PROGRESS** | T13 not complete |

The current walkthrough's concise PASS matrix should therefore not be carried into T15 unchanged.

---

# 9. Gate disposition

## G1 — Structural authority and representation

**Status: BLOCKED**

Positive:

- normative owner reconciled;
- parser/AST support exists;
- presence/remainder types exist.

Blocking:

- canonical store permits optional positional Tuple state;
- materialization does not preserve structural shape completely.

## G2 — Structural relation laws

**Status: FAIL**

Blocking:

- optional source -> required target is unsound in both Record and labeled Tuple;
- transformation closure for remainders is incomplete;
- planned direct law matrix did not cover the failing direction.

## G3 — Nested Dynamic application/materialization

**Status: FAIL/PARTIAL**

Positive:

- nominal-head formation and TypeView dependency-sensitive materialization work.

Blocking:

- applied relation returns hard refutation instead of Dynamic boundary;
- TypeLambda application rejects Dynamic marker;
- Dynamic metadata durability is not fully tested.

## G4 — Generic witness specialization

**Status: OPEN**

Positive:

- inferred return is now materialized in `specialize_witness_signature`.

Blocking/evidence:

- T12 still has the owner + method-generic evidence publication failure;
- TypeView structural-shape loss can corrupt structural signatures during specialization.

## G5 — Iterator adapter source contracts

**Status: SUBSTANTIALLY SATISFIED, pending upstream correctness**

The Universe source improvements are real. However G5 cannot be treated as checkpoint-complete while G3/G4 semantic machinery remains defective.

## G6 — Universe bootstrap

**Status: RECORDED PASS, not independently rerun**

The checkpoint records the prior 20+3 diagnostics as eliminated.

## G7 — Natural Range chain

**Status: RECORDED PASS, not independently rerun**

The repository records 24/24 `language::iteration`.

## G8 — Collection regressions

**Status: RECORDED focused pass**

Source inspection confirms the intended `toMap` single-advance loop.

## G9 — Persistence, incremental, tooling

**Status: BLOCKED**

Blocking:

- runtime template instantiation ignores v10 open structural forms;
- runtime row reflection loses presence/remainder;
- Dynamic marker compatibility floor missing;
- Dynamic metadata roundtrip not directly tested;
- CV-26 incremental/tooling evidence absent from current P5 completion records;
- required resource observations absent.

## G10 — Rest/module-export evidence transport

**Status: FAIL**

Positive:

- ordinary specialized rest send passes environment.

Blocking:

- semantic exact/rest precedence wrong;
- semantic label prefix matching wrong;
- module-export exact forwarding empties conformance env;
- module-export rest forwarding empties type/conformance env;
- current focused test does not cover module-export path.

## G11 — P4.R1/P5 focused acceptance matrix

**Status: FAIL/INCOMPLETE**

T12 is explicitly still in progress and has a known 17/18 semantic generic-conformance result.

## G12 — Broad checkpoint certification

**Status: NOT REACHED**

It would be premature to use broad certification as a repair oracle while G2/G3/G9/G10/G11 are known red.

---

# 10. Type-system completeness assessment

## 10.1 Structural system

**Assessment: conceptually coherent, implementation not closed under transformations.**

The canonical data model is sufficient in broad shape:

- first-class presence;
- anonymous typed/unconstrained Record remainder;
- independent Tuple lane remainders;
- captured row identity retained separately.

The main failures are not “missing enum variants.” They are **operation closure** failures: relation/materialization/normalization paths do not all implement the same semantics.

That is a more serious category than a missing parser branch because it allows the same canonical type to mean different things depending on which compiler service touched it.

## 10.2 Dynamic generic system

**Assessment: formation works, relation/durability completeness does not.**

The `StaticArgument::DynamicType` architecture is viable and avoids turning Dynamic into a fake `TypeId`.

However the system is not yet uniformly knowledge-bearing:

- formation understands Dynamic;
- TypeView understands selected cases;
- ordinary exact substitution simply preserves existing Dynamic markers;
- applied relation does not model DynamicBoundary;
- type-lambda application rejects the marker;
- metadata/runtime persistence is incompletely verified.

P5 therefore has the representation, but not yet the complete algebra over the representation.

## 10.3 Generic witness specialization

**Assessment: correct architectural direction, still not acceptance-complete.**

Materializing `inferred_return` in `specialize_witness_signature` is consistent with P5-I and avoids a second result authority.

The remaining T12 evidence failure and TypeView structural-shape defects mean this path needs another focused closure pass before G4 acceptance.

---

# 11. Runtime completeness assessment

## 11.1 Rest dispatch

Environment-aware helpers are correctly designed:

```text
call_method_with_selector_as_and_environments
activate_rest_method_with_environment
call_rest_method_as_with_environments
```

They carry explicit type and conformance environment IDs into the callee frame.

The failure is topology coverage: not every forwarding path uses them.

## 11.2 Module export

Module export is currently the clearest environment-loss site and should be repaired before any additional runtime proof logic is considered.

No VM conformance solver is necessary or justified.

## 11.3 Runtime typing overlay

The runtime overlay remains based on the older closed structural representation:

- Tuple element has label/type but no presence;
- Record field has name/type but no presence;
- row argument has captured tail but no anonymous remainder.

P5 durability cannot be called complete until the runtime type representation is capable of retaining the v10 contract it is asked to expose.

---

# 12. Incremental and metadata assessment

## 12.1 What is good

Metadata fingerprints explicitly hash:

- presence;
- Record remainder;
- Tuple remainder lanes;
- Dynamic static argument marker.

That is the right identity model.

## 12.2 What remains unproven or incorrect

- no direct Dynamic metadata roundtrip discriminator was found;
- no direct P5 structural/Dynamic cold-vs-incremental test was found;
- no resource-count evidence was recorded;
- runtime instantiation is incomplete for v10 nodes.

The metadata **wire representation** is ahead of the runtime/incremental acceptance evidence.

---

# 13. Efficiency, memory and resource review

## 13.1 No memory-safety defect found

No new `unsafe` ownership issue, invalid pointer usage, or obvious GC root-loss defect was identified in the P5 implementation paths reviewed.

The new runtime environments are IDs into VM-owned registries rather than borrowed Rust references, which is a sound ownership pattern.

## 13.2 Environment interning

`RuntimeTypeEnvironmentRegistry` interns immutable sorted binding sets. This is appropriate and prevents per-call duplicate environment objects for identical exact environments.

No analogous P5-specific leak was proven.

## 13.3 Structural relation complexity

As recorded in P5-AUD-020, relation can perform O(n×m) label scans.

This is the principal performance concern directly attributable to the P5 structural relation implementation.

## 13.4 Rest-call allocation

Rest activation copies the raw argument window into a temporary `Vec<Value>` before rewriting the stack.

That behavior is not new to the P5 environment fix, but P5's expanded use of rest dispatch makes it worth retaining in performance awareness. This audit does not classify it as a P5 defect without measured allocation pressure.

## 13.5 Canonical interning

The TypeStore and runtime overlay use interning keyed by structural nodes. The architecture should deduplicate equivalent `List<Dynamic>` and equivalent structural shapes.

The required explicit observation demonstrating this for P5 was not recorded.

---

# 14. Test-quality audit

## 14.1 Structural tests are example-driven rather than law-complete

The direct Record test is useful but misses the presence direction that exposes P5-AUD-001.

P5 explicitly asked for representative property/law tests, including transitivity. The current evidence is still closer to a curated case matrix than a systematic small-shape law suite.

### Recommended small-state matrix

Generate canonical structures from a small domain:

```text
presence: Required | Optional | Absent
tail: Closed | Unconstrained | Typed(Int) | Typed(Number)
field type: Int | Number | String
```

and assert hand-specified relation laws for source/target combinations.

Do the same independently for Tuple positional and labeled lanes.

## 14.2 Dynamic tests stop too early

CV-10..14 validate formation/materialization but not the relation algebra.

Add:

```text
List<Dynamic> against List<Int>
List<Int> against List<Dynamic>
invariant/covariant/contravariant heads
Dynamic-containing conformance constraints
```

## 14.3 Metadata tests cover structural v10 but not Dynamic slot

Add a single vertical test that starts in semantic TypeStore and ends in runtime inspect/presentation.

## 14.4 Rest test topology is incomplete

The ordinary rest test is good.

Add separate exact and rest Module export cases. Do not consider one test representative of both pathways.

## 14.5 Dispatch precedence needs a cross-layer test

The semantic/runtime exact-before-rest mismatch is a classic “both layers individually reasonable” bug.

The regression must assert both selected semantic `CallableId` and runtime result.

---

# 15. Required remediation sequence

The safest correction order follows dependency ownership rather than commit history.

## R1 — Repair structural presence relation

Files:

- `phalcom-semantic/src/types/relation.rs`

Work:

- required-target presence check for Record;
- required-target presence check for labeled Tuple;
- direct negative regressions.

Then rerun G2 structural law selectors.

## R2 — Make structural reconstruction closed over P5 shape

Files:

- `types/environment.rs`
- `types/instantiation.rs`
- `types/projection.rs`
- possibly shared row/tuple transformation helper

Work:

- preserve Tuple lane remainders;
- recursively materialize/normalize typed remainders;
- consume row bindings where semantically required;
- add substitution-vs-TypeView equivalence tests for open structures.

Do this before re-evaluating witness specialization.

## R3 — Enforce canonical Tuple invariants

Files:

- `types/store.rs`

Work:

- reject/remove optional positional elements at canonical construction boundary.

## R4 — Repair Dynamic applied relation

Files:

- `types/relation.rs`

Work:

- explicit Dynamic static-argument relation outcome;
- variance-aware DynamicBoundary tests;
- no Dynamic->Any conversion.

## R5 — Decide/repair Dynamic TypeLambda application

Files:

- `types/store.rs`
- type-lambda materialization tests

Either implement the P5 general rule or explicitly narrow the specification.

## R6 — Restore semantic/runtime dispatch parity

Files:

- `phalcom-semantic/src/surface.rs`
- `phalcom-semantic/src/dispatch.rs`

Work:

- full-hierarchy exact pass;
- full-hierarchy rest pass;
- ordered fixed-label prefix check;
- inherited-exact/subclass-rest regression.

## R7 — Complete T11 module-export environment transport

Files:

- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/vm/dispatch.rs` if call signature changes

Work:

- pass conformance env into module export helper;
- exact export uses plural environment call helper;
- rest export uses environment-aware rest activation;
- direct exact/rest export evidence tests.

## R8 — Complete runtime structural durability

Files:

- `phalcom-core/src/typing/overlay.rs`
- `typing/environment.rs`
- `typing/inspect.rs`
- reflection tests

Work:

- runtime presence/remainder representation;
- instantiate OpenTuple/AnonymousRecord/OpenRecord templates;
- correct presentation syntax.

## R9 — Close metadata/incremental evidence

Work:

- Dynamic static-argument schema-floor test;
- `List<Dynamic>` metadata roundtrip;
- structural/Dynamic cold/import/incremental parity;
- dependent-only invalidation check;
- source-index/LSP checks required by T10;
- resource/work-count observations.

## R10 — Resolve T12 owner-evidence failure

Only after R2/R4/R6 because those fixes can affect specialization and evidence.

Then rerun the full CV-01..CV-27 focused matrix.

## R11 — T13/T14/T15

Only when focused gates are green:

1. classify residual failures;
2. broad semantic/core/corpus/workspace certification under repository policy;
3. update checkpoint/walkthrough/handoff with exact commands/results;
4. mark completion only according to actual evidence.

---

# 16. Suggested regression tests

The following tests would directly close the audit findings.

```text
structural::record_optional_source_does_not_satisfy_required_target
structural::tuple_optional_source_does_not_satisfy_required_target

foundations::generics_core::
  open_tuple_typeview_matches_exact_substitution
  anonymous_record_remainder_typeview_matches_exact_substitution

foundations::type_model::
  typestore_rejects_optional_positional_tuple_element

foundations::relation::
  dynamic_covariant_argument_returns_dynamic_boundary
  dynamic_contravariant_argument_returns_dynamic_boundary
  dynamic_invariant_argument_returns_dynamic_boundary

foundations::type_annotations::
  dynamic_type_lambda_application_has_defined_p5_semantics

dispatch::
  inherited_exact_beats_subclass_rest_semantically
  rest_fixed_labels_require_ordered_prefix

language::generic_conformance::
  module_export_exact_specialization_preserves_conformance_environment
  module_export_rest_specialization_preserves_conformance_environment

typing_integration::
  runtime_open_tuple_template_specializes_nested_parameter
  runtime_anonymous_record_template_specializes_remainder
  runtime_record_row_reflection_preserves_presence_and_remainder

metadata::
  dynamic_static_argument_roundtrips_in_schema_v10
  schema_v9_rejects_dynamic_static_argument
  dynamic_slot_edit_changes_interface_fingerprint
  equivalent_dynamic_slot_contract_preserves_fingerprint

incremental::
  p5_structural_presence_edit_invalidates_only_dependents
  p5_dynamic_slot_edit_matches_cold_analysis
```

---

# 17. Completion criteria after remediation

P5 should not be accepted until all of these statements are true.

### Structural

- optional source never satisfies required target;
- impossible optional positional Tuple state cannot enter TypeStore;
- every structural transformation preserves presence and lane/tail state;
- direct law tests include negative directions and representative transitivity.

### Dynamic

- nominal and other adopted generic Type-kind heads have coherent Dynamic-slot semantics;
- relation returns DynamicBoundary when a proof depends on Dynamic;
- metadata/runtime preserve `DynamicType`;
- no Dynamic->Any, Unknown->Dynamic, or synthetic Dynamic TypeId.

### Witness/conformance

- `receiver_owner_evidence_is_published_without_owner_inference` passes or is replaced by a source-proven architectural disposition;
- negative witness strictness remains intact.

### Dispatch/runtime

- semantic and VM exact/rest resolution choose the same callable;
- labeled rest prefix semantics match;
- ordinary, module-export exact, and module-export rest paths all transport established environments;
- no VM proof search.

### Persistence/incremental/tooling

- schema v10 compatibility is complete for both structural and Dynamic additions;
- runtime reflection preserves all declared durable distinctions;
- cold/import/incremental products agree;
- source-index/LSP acceptance is directly evidenced;
- resource observations required by P5 are recorded.

### Verification

- CV-01..CV-27 have real PASS evidence after the fixes;
- CV-28 residual classification complete;
- G12 broad certification is run only after focused gates;
- lifecycle records reflect the actual final revision and exact evidence.

---

# 18. Final disposition

`LANG007.C3.P5` has made substantial architectural progress. The implementation correctly moved Phalcom toward:

- explicit structural exactness/openness;
- first-class presence;
- canonical anonymous remainders;
- known-head Dynamic generic applications;
- provider-owned result specialization;
- source-declared Iterator contracts;
- explicit runtime evidence environments;
- durable schema-v10 structural metadata.

But the current code still contains several contradictions between those newly-ratified semantics and the operations that consume them.

The most important pattern is:

> **P5 added the right semantic distinctions, but not every relation, materializer, dispatch path, and runtime projection preserves those distinctions yet.**

That means the right corrective strategy is not another redesign. The model is mostly decided. The work now is to make every existing owner obey it consistently.

### Final audit classification

```yaml
P5:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
  audit_acceptance: REJECT_COMPLETION
  broad_certification_ready: false

blocking_areas:
  - structural presence soundness
  - structural shape materialization
  - Dynamic applied relation
  - semantic/runtime rest dispatch parity
  - module-export conformance transport
  - runtime structural durability
  - T10 incremental/tooling evidence
  - T12 generic-conformance closure
```

The canonical checkpoint should remain open. Repair the source-proven defects first, then rerun the focused matrix. Broad T14 certification becomes meaningful only after those gates are green.
