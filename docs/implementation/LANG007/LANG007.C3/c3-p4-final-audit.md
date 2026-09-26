# LANG007.C3.P4 post-implementation audit

I inspected the pushed `main` state at:

```text
0d6a6e9bbf74c82e5c3587eb4153fde589cbedd5
feat(lang007-c3-p4): complete tuple iterator specialization and projection non-injectivity laws
```

with parent:

```text
c38625360ed0b78da22cdb30f01469b3b1f5baf1
```

I compared the implementation against the active P4 plan, the current checkpoint, the earlier architectural audit/handoff, the changed source, relevant historical verification records, and the failure report you supplied.

The two pre-renumbering P4 notes are explicitly historical and say they are neither the current handoff nor the completion walkthrough, so I did not treat them as completion authority. :chatgpt-content-reference{index="0"} :chatgpt-content-reference{index="1"} The current checkpoint remains the durable authority and still records C3/P4 as `IN_PROGRESS / PARTIAL / UNVERIFIED`. :chatgpt-content-reference{index="2"}

My overall conclusion is:

```yaml
TupleIterator_incident:
  functional_status: SUBSTANTIALLY_FIXED
  architectural_status: PARTIALLY_FIXED
  verification: FOCUSED_ONLY

projection_solver:
  status: UNSOUNDLY_INCOMPLETE

P4:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: UNVERIFIED

C3:
  closable: false
```

The implementer did make several architecturally good repairs, and the specialization-aware Tuple iteration path is now real. But the claim that the work unit, or especially P4, is “finished” does not withstand audit. The most important remaining defect is not a missing test: the solver still has no representation for an **unproved deferred projection constraint**, so it can conflate “I did not infer backwards” with “this equality has been satisfied.” That is potentially a type-system soundness defect.

There are also substantial runtime conformance regressions that have been incorrectly labeled baseline, semantic-suite failures that include previously green acceptance tests, unclosed P4 tasks and gates, incomplete lifecycle records, and a source/internal canonicalization split in the new complete-pack round-trip rule.

---

## 1. Claim-by-claim verdict

| Implementer claim | Audit result | Verdict |
|---|---|---|
| `Tuple<***Es> → TupleIterator<***Es> → Iterator::Item → Tuple<***Es>::Element` preserved | Current Universe source does preserve this chain | **Confirmed** |
| No `Dynamic` escape in Tuple iterator | Current `TupleIterator` source is fully specialized | **Confirmed** |
| Complete-pack round-trip implemented | Implemented for internal source-less lane projections | **Partially confirmed** |
| Complete-pack round-trip is canonical generally | Source-backed projections are excluded by the helper | **Incomplete** |
| Conformance head matching unified | Exact/overlap matching now share `match_conformance_head_with_mode` | **Confirmed improvement** |
| Type-member projection solver preserves symbolic children | Yes | **Confirmed** |
| Associated/type-member projection non-injectivity is soundly enforced | Reverse solving was removed, but unresolved equalities are silently treated as `Unchanged` | **Not soundly complete** |
| Deferred projection normalization is covered | Tests exercise normalization, not the solver's residual-obligation lifecycle | **Overstated** |
| Invariant `Option<projection>` comparison repaired | Relation layer now uses semantic bidirectional subtype equality instead of only raw `TypeId` identity | **Confirmed improvement** |
| Witness symbolic mismatch handling repaired | Broad “symbolic refutation → Unknown” workaround has been removed | **Confirmed improvement** |
| All 9 core failures are downstream/baseline | Multiple failed tests were historically verified green | **Incorrect classification** |
| 40 semantic failures are migration/baseline | Several exact failures were historically verified green; some are P4-owned migration/incremental work | **Incorrect/inadequate classification** |
| P4 required criteria finished | T4/T5 downstream obligations, T11/T12, T13, walkthrough, handoff, CV ledger, broad gates remain | **False** |
| Checkpoint/lifecycle finished | Current records contradict themselves; no current P4 walkthrough/handoff exists | **False** |

The current checkpoint itself only certifies the newly repaired Tuple path and its focused tests. :chatgpt-content-reference{index="3"} That is materially narrower than complete P4 acceptance.

---

# 2. What was implemented correctly

## 2.1 The specialization-aware Tuple iterator is now correct at the source surface

Current `phalcom-core/core/universe/src/collections/tuple.ph`:

```phalcom
class TupleIterator<***Es: CompleteTypePack> {
  _source: Tuple<***Es>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: Tuple<***Es>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<Tuple<***Es>::Element> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    value
  }

  iterator { self }
}

impl<***Es: CompleteTypePack> CanonicalIterator for TupleIterator<***Es> {
  type Item = Tuple<***Es>::Element
}

impl<***Es: CompleteTypePack> CanonicalIterable for Tuple<***Es> {
  type Item = Tuple<***Es>::Element
  type Iterator = TupleIterator<***Es>

  iterator -> TupleIterator<***Es> {
    TupleIterator.new(self)
  }
}
```

This is the intended architecture.

The following identity chain is now represented directly rather than reconstructed from `Dynamic`:

```text
Tuple<***Es>
     │
     ├─ iterator
     ▼
TupleIterator<***Es>
     │
     ├─ Iterator::Item
     ▼
Tuple<***Es>::Element
```

This part of the incident is genuinely fixed.

---

## 2.2 General `TypeMemberProjection` is now visible inside inference

The previous solver represented most type-member projections as opaque:

```rust
InferenceTerm::Canonical(type_id)
```

even when the canonical `TypeId` contained declaration parameters requiring substitution.

The new inference representation preserves:

```text
subject
selector
arguments
source_declaration
result_kind
```

recursively.

That is architecturally correct and necessary.

The newly added `term_substitute_variables` also recursively traverses projection inputs rather than freezing them.

I would keep this design.

---

## 2.3 The witness compatibility fix moved in the correct direction

The previous witness path could compare:

```text
Option<ProjectionA>
Option<ProjectionB>
```

as invariant generic applications and refute them solely because:

```text
TypeId(ProjectionA) != TypeId(ProjectionB)
```

even if normalization would make them equal.

`types/relation.rs` now does, approximately:

```rust
Variance::Invariant => {
    if a_sub == a_sup {
        RelationOutcome::proven(())
    } else {
        let forward =
            check_subtype_impl(store, hierarchy, a_sub, a_sup, ...);

        match forward {
            RelationOutcome::Proven { .. } =>
                check_subtype_impl(
                    store,
                    hierarchy,
                    a_sup,
                    a_sub,
                    ...
                ),

            other => other,
        }
    }
}
```

That is a meaningful improvement.

It makes raw `TypeId` equality the fast path rather than the entire notion of invariant semantic equality.

---

## 2.4 The earlier witness-level workaround was correctly removed

Previously the implementation had logic conceptually equivalent to:

```text
if this whole type contains some symbolic projection
and subtype checking returns Refuted
    convert Refuted → Unknown
```

That was too broad because:

```text
Option<P::Item>
```

and:

```text
List<P::Item>
```

are definitely different even if `P::Item` is unresolved.

The new implementation normalizes both types and lets the canonical relation engine return `Proven`, `Refuted`, `Blocked`, etc.

That is the correct ownership boundary.

---

## 2.5 Conformance head matching was genuinely consolidated

This is another strong improvement.

The code now routes the directional exact matcher and overlap/coherence matcher through a common:

```rust
match_conformance_head_with_mode(...)
```

instead of maintaining two unrelated structural algebras.

That addresses the architectural problem I called out earlier:

```text
exact lookup semantics
             \
              shared structural decomposition
             /
coherence semantics
```

rather than:

```text
exact matcher A
coherence matcher B
```

with subtly different `TypeData` coverage.

The handling for `TupleFromPacks`, `CompleteTypePack`, concrete Tuple/Unit forms, etc. is substantially better.

---

# 3. Critical blocker: projection constraints still have no “deferred proof” state

This is the biggest remaining problem.

The prior handoff explicitly required a distinction between:

```text
proved
```

and:

```text
not currently provable, but do not infer backwards
```

The implementer removed reverse inference but did **not** implement that distinction.

Current `phalcom-semantic/src/checker/inference.rs` still has only:

```rust
pub enum SolveEffect {
    Unchanged,
    Changed,
}
```

There is no:

```rust
Deferred
```

or residual-obligation set.

That becomes unsound when projections are non-injective.

---

## 3.1 Current associated projection behavior is not a proof

Current code around `inference.rs:4039`:

```rust
(
    InferenceTerm::AssociatedProjection { .. },
    InferenceTerm::AssociatedProjection { .. },
) => {
    if left == right {
        return Ok(SolveEffect::Unchanged);
    }

    // Associated projections are non-injective by default.
    // Do not unify subjects or arguments backwards.
    Ok(SolveEffect::Unchanged)
}
```

The first branch is fine:

```text
P == P
→ already satisfied
```

The second branch is not.

For two distinct unresolved projections:

```text
<X as A>::Item
<Y as B>::Output
```

the solver currently reports the same effect:

```text
Unchanged
```

as it does for a proven reflexive equality.

Those are not semantically equivalent.

The correct outcome is:

```text
Deferred
```

until normalization can establish equality or inequality.

---

# 4. The same bug exists for general `TypeMemberProjection`

Current code around `inference.rs:4272`:

```rust
// Type-member operations are not injective. Their subjects and
// arguments may become canonical after other constraints solve,
// but projection equality must not solve those children backwards.
(InferenceTerm::TypeMemberProjection { .. }, _)
| (_, InferenceTerm::TypeMemberProjection { .. })
    => Ok(SolveEffect::Unchanged),
```

The comment is correct.

The result is not.

Consider:

```text
Tuple<X>::Element == String
```

If the left side cannot yet normalize, the correct conclusion is:

```text
cannot prove or refute yet
```

not:

```text
successful unification with no changes
```

This distinction matters because `finish_frame` does not inspect unresolved constraint proofs.

---

# 5. `finish_frame` can therefore publish a false `Solved`

Current completion logic around `inference.rs:2473` is effectively:

```rust
pub fn finish_frame(&self, frame: InferenceFrameId)
    -> InferenceOutcome
{
    let mut unsolved = Vec::new();

    for var in &self.variables {
        if var.frame == frame
            && !self.substitutions.contains_key(&var.id)
        {
            unsolved.push(var.id);
        }
    }

    if !unsolved.is_empty() {
        Underconstrained(...)
    } else {
        Solved(...)
    }
}
```

Likewise `finish_root`.

Notice what is absent:

```text
Are all required equality/subtype constraints proven?
```

Only variable substitution completeness is tested.

That permits the following sequence:

```text
constraint:
    F<X> == Int

1. F<X> is unresolved.
2. Reverse inference is correctly forbidden.
3. unify_terms returns Unchanged.
4. Some unrelated constraint solves X.
5. F<X> still does not normalize to Int,
   or remains symbolic.
6. All InferVarIds now have substitutions.
7. finish_frame sees no unsolved variable.
8. outcome = Solved.
```

That is precisely the bug the earlier handoff warned against.

---

# 6. `extract_and_normalize_projections` does not close this gap

The implementer did add more normalization machinery.

Current `extract_and_normalize_projections` walks constraints and, when a side can materialize and normalization changes its `TypeId`, rewrites that side.

That is useful.

But if:

```text
normalization remains symbolic
```

or:

```text
normalization returns the same projection TypeId
```

then no residual obligation is recorded.

Once the fixed point produces no more substitutions:

```text
solve loop terminates
    ↓
finish_frame
    ↓
all variables substituted?
    ↓
Solved
```

So normalization support does not substitute for proof-state tracking.

---

# 7. The pack-projection change has the same problem

The new code correctly removed the unsound rule:

```text
Positionals(P) == Positionals(Q)
    ⇒ P == Q
```

That is an improvement.

Current behavior is now approximately:

```rust
if left_subject == right_subject {
    return Ok(Unchanged);
}

if lane == Complete {
    return self.unify_terms(left_subject, right_subject, store);
}

// non-injective Positionals/Labeled
Ok(Unchanged)
```

Again:

```text
do not infer P == Q
```

is correct.

But:

```text
therefore the constraint is satisfied
```

is not.

For `Positionals` and `Labeled`, distinct unresolved subjects must produce a deferred obligation.

### About the `Complete` lane

The `Complete` lane can plausibly be treated as an identity operation if its specification really is:

```text
CompleteProjection(P) ≡ P
```

In that case reducing it forward and then ordinary-unifying the result is valid.

I would still encode that as forward normalization:

```text
CompleteProjection(P)
→ P
```

rather than as an exceptional “projection is injective” rule. That keeps the family law conceptually clean.

---

# 8. Required solver architecture

The inference engine now needs a real third semantic outcome.

Conceptually:

```rust
enum ConstraintProgress {
    Proven,
    Changed,
    Deferred(DeferredReason),
}
```

or, less invasively:

```rust
InferenceSession {
    ...
    pending_obligations: Vec<PendingConstraint>,
}
```

The required projection comparison lifecycle should be:

```text
apply current substitutions
        ↓
can both sides normalize/materialize?
        │
        ├─ no
        │    ↓
        │  Deferred
        │
        └─ yes
             ↓
        canonical normalization
             ↓
       ┌─────┼───────────┐
       │     │           │
     equal unequal    symbolic
       │     │           │
    Proven Refuted    Deferred
```

`finish_frame` and `finish_root` must not return `Solved` while required deferred obligations remain.

This is a P0 type-system correction.

---

# 9. The new tests do not actually prove solver non-injectivity

This is a significant verification gap.

The test:

```text
projection_equality_is_non_injective_for_type_member_projections
```

sounds like a solver test.

It is not.

It essentially constructs two `CompleteTypePack` parameters, obtains their pack projections, checks the canonical IDs differ, and checks a mixed complete-pack round trip does not collapse.

It never creates an:

```rust
InferenceSession
```

and never adds an:

```rust
InferenceRelation::Equivalent(...)
```

between two type-member projections.

Therefore it proves:

```text
TypeStore does not accidentally identify different lane projections
```

not:

```text
inference does not reverse-solve them
```

and certainly not:

```text
an unresolved equality cannot falsely finish as Solved
```

---

## 9.1 The “deferred projection” test also does not test solver deferral

The test named along the lines of:

```text
deferred_projection_with_generic_substitution_normalizes_to_symbolic
```

directly exercises projection normalization.

That proves:

```text
normalizer preserves a valid symbolic projection
```

which is useful.

But it does **not** prove:

```text
InferenceSession retains that equality as an unresolved obligation.
```

These are separate laws.

---

## 9.2 Missing mandatory solver regressions

Before this work can be called sound, I would require direct tests for:

```text
TypeMemberProjection:
    F<X> == F<Y>
    must not solve X = Y

AssociatedProjection:
    <X as Trait>::Item == <Y as Trait>::Item
    must not solve X = Y

PackProjection:
    Positionals(P) == Positionals(Q)
    must not solve P = Q

Pending:
    unresolved projection equality
    must prevent InferenceOutcome::Solved

Positive delayed resolution:
    solve X and Y independently
    normalize both projections
    equal result → success

Negative delayed resolution:
    solve X and Y independently
    normalized projection results differ
    → conflict

Projection vs concrete:
    F<X> == Int
    cannot silently succeed while F<X> is stuck

Zero-variable case:
    two distinct symbolic projections with no InferVar left
    must not immediately produce Solved
```

These need to call the solver itself.

---

# 10. Complete-pack round-trip: good law, incomplete canonical identity

The new helper:

```rust
TypeStore::canonical_complete_pack_subject
```

implements the right mathematical law:

```text
Complete(
    Positionals(P),
    Labeled(P)
)
≡ P
```

But its implementation currently recognizes only internal source-less projections:

```rust
pos.source_declaration.is_none()
&& lab.source_declaration.is_none()
```

This is a significant asymmetry.

---

## 10.1 Why this matters

Internal compiler helpers:

```rust
store.project_positionals(P)
store.project_labeled(P)
```

construct:

```rust
source_declaration: None
```

so:

```text
Complete(internal Positionals(P),
         internal Labeled(P))
→ P
```

works.

But the P4 architecture says public source-authored intrinsic members retain source identity.

A source-written symbolic projection can therefore be:

```text
TypeLevelProjection {
    subject: P,
    selector: Positionals,
    source_declaration:
        Some(CompleteTypePack::Positionals),
}
```

and similarly for `Labeled`.

Then:

```text
Complete(
    source-backed Positionals(P),
    source-backed Labeled(P)
)
```

does **not** satisfy the helper's `is_none()` predicate.

So the same semantic operation has representation-sensitive canonicalization.

---

# 11. The correct rule should use canonical member identity, not absence of provenance

Do not “fix” this by simply ignoring `source_declaration`.

Instead recognize:

```text
is canonical CompleteTypePack::Positionals operation?
is canonical CompleteTypePack::Labeled operation?
same subject?
correct result kinds?
zero arguments?
```

regardless of whether the operation was formed through:

```text
internal projection helper
```

or:

```text
published source intrinsic declaration
```

A source declaration ID is valuable semantic/provenance information. It should not prevent the intrinsic evaluator from recognizing the canonical operation.

Required tests:

```text
internal Positionals(P) + internal Labeled(P)
    → P

source-backed Positionals(P)
+ source-backed Labeled(P)
    → P

source-backed Positionals(P)
+ internal Labeled(P)
    → same canonical result, if those are semantically the same operation

Positionals(P) + Labeled(Q)
    → no collapse

unrelated user member named "Positionals"
    → must not trigger intrinsic round trip
```

This is a high-priority canonicalization gap.

---

# 12. The 9 `phalcom-core` failures are not established baselines

This is one of the weakest parts of the implementer's report.

The report says all nine belong to:

> “Baseline / Downstream VM Conformance Milestone”

That classification is not supported by repository history.

Several exact tests are explicitly recorded as previously passing.

For example, prior LANG005 records show:

```text
owner_and_method_local_evidence_execute_together: PASS
```

and state that the generic-conformance runtime set was passing.

The C6 checkpoint also explicitly references:

```text
cv51_environment_reuse_repeated_invocations
```

as existing runtime environment reuse coverage.

`bounded_generic_function_dispatches_exact_witness_and_default` was also part of previously repaired/tested generic runtime conformance work.

`direct_field_delegation_executes_as_ordinary_accessors` belongs to previously accepted associated-trait behavior, not to an unimplemented future VM milestone.

Therefore, at least a material subset of these nine failures are **regressions**, not inherited baselines.

A baseline classification requires reproduction at a known predecessor.

The implementer did not supply that.

---

# 13. There is a concrete evidence-loss seam in the current VM

The runtime error reported by several failures is:

```text
internal error:
no active conformance target for requirement ...
```

That is consistent with current source.

`InvokeTraitRequirement` reads:

```rust
let conformance_environment = self
    .frames
    .last()
    .map(|frame| frame.conformance_environment)
    .unwrap_or(RuntimeConformanceEnvironmentId::EMPTY);

let slot_entry = self
    .runtime_conformance_environments
    .slot(
        conformance_environment,
        spec.requirement_slot
    )
    .ok_or_else(|| RuntimeError::Internal(
        "no active conformance target ..."
    ))?;
```

So any forwarding path that installs a frame with:

```text
conformance_environment = EMPTY
```

will produce exactly the observed panic.

And such forwarding paths exist.

---

# 14. `dispatch_conditional_family_entry` discards the conformance environment

Current `phalcom-core/src/vm/send.rs`:

```rust
fn dispatch_conditional_family_entry(...) -> PhResult<CallOutcome> {
    let method = self.select_conditional_method(...);

    self.stack[receiver_idx] = receiver;

    self.activate_captured_method_as(
        receiver,
        method,
        view,
        source_range,
    )
}
```

But:

```rust
activate_captured_method_as(...)
```

is just a convenience wrapper that invokes:

```rust
activate_captured_method_as_with_environment(
    ...,
    RuntimeTypeEnvironmentId::EMPTY,
    RuntimeConformanceEnvironmentId::EMPTY,
)
```

So this path explicitly throws away conformance evidence.

That is not a hypothetical future architecture issue. It is a concrete runtime transport hole.

---

# 15. `InvokeConditional` contains another environment-erasing path

In `vm/dispatch.rs`:

```rust
Bytecode::InvokeConditional { ... } => {
    ...

    let method_to_call = ...;

    self.call_method(
        &receiver,
        method_to_call,
        arity,
        ...
    )?;
}
```

`call_method` goes through the compatibility wrapper:

```rust
call_method_with_selector(...)
```

which ultimately supplies:

```rust
RuntimeTypeEnvironmentId::EMPTY
RuntimeConformanceEnvironmentId::EMPTY
```

unless a richer path is called explicitly.

So conditional runtime dispatch can also lose the active evidence environment here.

That maps directly onto the failing conditional/derived/generic conformance tests.

---

# 16. There are likely more environment-loss seams

The current VM has a good explicit function:

```rust
call_method_with_selector_as_and_environments(...)
```

and a good explicit activation function:

```rust
activate_captured_method_as_with_environment(...)
```

but many compatibility wrappers still default to empty environments.

That means every forwarding/gateway path must be audited:

```text
ordinary specialized call
module export forwarding
bound method
bound family
conditional family
derived conformance
rest-method dispatch
DNU forwarding where applicable
captured callable
trait-selected invocation
```

The architecture should be:

```text
environment-bearing caller
        ↓
every semantic forwarding seam
        ↓
same required environment
        ↓
callee frame
```

not:

```text
environment-bearing caller
        ↓
convenience wrapper
        ↓
EMPTY
```

---

# 17. `direct_field_delegation` is a separate runtime regression

The implementer reports this failure as:

```text
<Counter instance> does not understand 'doubled'
```

That does not look like the same missing conformance-slot error.

It points toward:

```text
trait default/member installation
associated witness lowering
impl compiler
method-table publication
```

rather than merely environment transport.

Since `c3862536` made substantial changes in:

```text
compiler/lib/class_decl.rs
compiler/lib/impl_decl.rs
method tables
VM dispatch/send
```

this should be bisected separately.

The correct classification is:

```text
REGRESSION UNTIL PROVEN BASELINE
```

not:

```text
future VM milestone
```

---

# 18. Required runtime regression procedure

For each of the nine failures, run the exact test at:

```text
0d6a6e9b
c3862536
cce11d22
```

and, if necessary, the historically recorded green LANG005 revision.

That will immediately separate:

```text
introduced in c386
introduced in 0d6
older actual baseline
```

The parent `c3862536` is especially important because it contains the large runtime-dispatch changes and is the direct predecessor of the current completion commit.

Until that reproduction is done, none of these failures should be marked baseline.

---

# 19. The 40 semantic failures are also not adequately classified

The report's four clusters are useful as hypotheses.

They are not sufficient baseline evidence.

Several exact failing tests have historical PASS evidence.

---

## 19.1 “Unexpanded pack syntax migration” is P4 migration work, not a baseline exception

For example:

```text
variadic_callable_alias_lowers_and_beta_reduces
```

is explicitly recorded as passing in the completed LANG007.C2 work.

If P4 intentionally ratified:

```text
pack generic contributions now require explicit *, **, ***
```

then old positive fixtures must be migrated.

That makes them:

```text
P4 source/test migration obligations
```

not:

```text
unrelated baseline failures
```

This matters because T11 explicitly exists to migrate the repository corpus to the completed structural model.

Leaving 14 positive tests on obsolete syntax means T11 is incomplete.

---

# 20. The generic identity failure cannot simply be blamed on new pack syntax

The report includes:

```text
generic_identity_solves_parameter_from_argument_and_specializes_return
```

among failing semantic tests.

That exact test is part of the established generic inference baseline and has previously passed.

If it now fails because a normal generic application is unexpectedly interpreted through new pack-expansion rules, that is likely an over-broad syntax or annotation-lowering regression.

It must be investigated individually.

A P4 syntax change must not accidentally require explicit pack expansion for ordinary:

```text
T: Type
```

generic parameters.

---

# 21. The flow failures also have historical green evidence

The report says about 12 tests should now use:

```text
const
```

or explicit type contracts because opaque calls invalidate mutable `let` knowledge.

That may be semantically correct for some tests.

But the existing semantic closure records show `flow_branches` and `flow_loops` suites previously passing under the established flow model.

Therefore there are only two legitimate outcomes:

### Outcome A — regression

The implementation accidentally broadened opaque-call invalidation.

Then fix it.

### Outcome B — intentional language change

P4 or another accepted design intentionally changed mutable-binding flow semantics.

Then the implementation needs:

```text
ratified semantic decision
normative spec update
test migration with explicit justification
cold/incremental consequences checked
```

You cannot retroactively classify green law-level tests as “stale fixtures” merely because current implementation invalidates them.

---

# 22. Changing iteration tests from `let` to `const` weakened coverage

The completion commit changed several iteration fixtures, for example:

```phalcom
let values = [1, 2, 3]
let first = values.iterator
```

to:

```phalcom
const values = [1, 2, 3]
const first = values.iterator
```

Similar changes were made to Set, Bytes, String view iterator tests.

This may make sense if those locals are conceptually immutable.

But it also avoids the exact flow-analysis surface that is currently producing semantic-suite failures.

So:

```text
15/15 iteration tests
```

does **not** establish that the original mutable-local iteration behavior remains sound.

I would add a separate test proving whatever the intended language law is for:

```phalcom
let values = ...
let iterator = values.iterator
```

rather than changing the only integration coverage to `const`.

---

# 23. Workspace/editor failures were historically green

Two examples in the implementer's “declaration publishing/workspace indexing” cluster:

```text
invalid_generic_kind_does_not_publish_ready_declaration_header
editor_visible_symbols_use_prelude_policy_and_preserve_local_shadowing
```

are present as passing tests in the repository's earlier semantic baseline evidence.

Those are therefore not established baselines either.

And P4 explicitly owns:

```text
source indexing
LSP/editor behavior
declaration publication
```

so even if the changed behavior were intentional, this surface is directly relevant to P4 acceptance.

---

# 24. Incremental failures are particularly serious for P4 closure

The report places tests such as:

```text
test_reexport_retargeting_invalidates_exact_consumer
p3_mode_only_pack_binder_edit_invalidates_dependents_to_cold_parity
```

in a “cache invalidation expectations” bucket.

But these are not incidental tests.

The second is an explicit LANG007.C2 acceptance regression recorded as passing in the C2 P3 walkthrough.

P4 invariant `INV-41` is:

```text
cold and incremental analysis converge
```

and CV-42 exists specifically for it.

Therefore an incremental test failure cannot be dismissed as:

```text
old query recomputation-count expectation
```

without proving that:

1. semantic final-state cold parity still holds, and
2. only an implementation-detail counter changed.

If final semantic results differ, it is a P4 blocker.

---

# 25. P4 T4 is still not closed

The authoritative checkpoint says the data-constructor shape adapter is present, but its required focused coverage and shape-materialization behavior remain to be verified. It also says exact T5 layouts and shared behavior-class registration remain downstream. :chatgpt-content-reference{index="4"}

That was not superseded by the TupleIterator commit.

The completion commit primarily touched:

```text
inference
conformance matching
relation
Tuple iteration
pattern reads
```

It did not provide a full T4 acceptance ledger.

T4 is:

> Build semantic product-shape templates and domain-aware specialization

That task is substantially broader than Tuple iterator typing.

---

# 26. P4 T5 is also still open

T5 is:

> Make expanded nominal data runtime-correct with shared behavior class and exact specialization layouts

The current checkpoint explicitly says:

```text
T5 exact layouts and shared behavior-class registration remain downstream
```

and no subsequent completion record demonstrates otherwise. :chatgpt-content-reference{index="5"}

This alone is enough to reject a whole-P4 completion claim.

---

# 27. T6 improved, but runtime conformance regressions prevent acceptance

T6 is:

> Complete declaration type-family/runtime projections and consolidate intrinsic application

The new shared conformance-head matcher is good.

But the current runtime evidence transport failures mean the resulting structural/conformance behavior is not broadly preserved.

Given nine language-runtime failures in associated/generic/conditional/derived conformance, I would classify T6 as:

```text
IMPLEMENTED IN PART
NOT ACCEPTED
```

until regression classification is complete.

---

# 28. T7's Tuple portion improved significantly, but solver soundness blocks full acceptance

T7 is:

> Complete source-visible structural domains, intrinsic associated types, and specialization-aware Tuple/Record APIs

The TupleIterator incident belongs here.

The source API is now much better.

But T7 cannot be considered complete while the generic solver can potentially publish false `Solved` outcomes for unresolved type-member projections.

This is not a cosmetic issue.

`Tuple::Element` is itself exactly such a projection.

---

# 29. T11 corpus migration is demonstrably incomplete

T11 is:

> Migrate the Universe/core source and repository corpus to the completed structural model

The implementer's own report says 14 semantic tests still use obsolete pack application syntax.

That is direct evidence that the corpus migration is incomplete.

Those fixtures must be classified:

```text
positive:
    migrate to explicit expansion

negative:
    retain old spelling intentionally
    assert migration-quality diagnostic

historical:
    exclude explicitly
```

Do not leave current positive tests failing and call them baseline.

---

# 30. T12 is not accepted while editor/incremental tests fail

T12 is:

> Persist the new semantic surface through metadata, incremental fingerprints, source indexing, LSP, and specifications

The reported failures include:

```text
incremental invalidation
workspace publication
editor visible symbols
```

which map directly onto T12.

Therefore T12 is not complete.

---

# 31. T13 has plainly not happened

T13 is:

> Stabilize, certify C3.P4, re-close LANG007.C3, and produce execution records

The P4 plan requires, before calling P4 implemented, a walkthrough containing the full CV-01 through CV-47 coverage table.

The plan explicitly says:

```text
A missing required test is not replaced by code inspection.
```

The required final mapping includes:

```text
CV-01       lifecycle
CV-02..06   structural kinds
CV-07..15   parser/product-shape generics
CV-16..22   runtime specialization
CV-23..29   Unit/EmptyRecord
CV-30..36   Record/Map/Set
CV-37..40   intrinsic members/API
CV-41..43   metadata/incremental/LSP
CV-44       runtime owner
CV-45       scope census
CV-46       compatibility
CV-47       broad closure
```

There is currently no:

```text
LANG007.C3.P4-walkthrough.md
```

and no:

```text
LANG007.C3.P4-handoff.md
```

on `main`.

Both paths return 404 through the repository API.

So T13 is unambiguously incomplete.

---

# 32. Broad closure gates have not passed

The P4 broad closure requires the equivalent of:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check --workspace --all-targets

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test --workspace

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo clippy --workspace --all-targets -- -D warnings

cargo fmt --all -- --check

git diff --check
```

The available evidence already shows:

```text
core language:
    9 failures

semantic integration:
    40 failures
```

Therefore G6 cannot be green.

Even if every failure eventually receives a legitimate clean-predecessor baseline classification, that classification still has to be recorded individually and accurately.

It has not been.

---

# 33. Lifecycle records currently contradict themselves

This needs cleanup before closure.

`LANG007.C3-CHECKPOINT.md` correctly says:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

and the attached authoritative checkpoint confirms that state. :chatgpt-content-reference{index="6"}

`STATUS.md` also initially says P4 remains active.

But later it contains stale prose claiming:

```text
LANG007.C1, C2, and C3 are complete at focused-tested level.
No successor checkpoint or plan is accepted.
```

Meanwhile `PROGRAM.md` has the contradictory combination:

```text
C3:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

and:

```text
P4:
    IN_PROGRESS / PARTIAL / UNVERIFIED
```

An active incomplete P4 reopening C3 and a complete C3 cannot both be current truth.

The checkpoint should win.

---

# 34. The checkpoint's own next action is stale

Current checkpoint still says:

```text
Close the TupleIterator projection/conformance regression,
then prove native re-entry authority ...
```

But later in the same checkpoint the TupleIterator path is recorded as resolved.

That `next_action` should now advance to the remaining real blockers instead of retaining the old incident.

This is small compared with the semantic issues, but it demonstrates that lifecycle closure was not performed.

---

# 35. Native re-entry authority: implementation improved, but the exact ruling is not fully proven

The earlier checkpoint had an explicit native re-entry ruling requiring activation identity robust to:

```text
non-local return
frame truncation
frame-vector slot reuse
nested native/bytecode transitions
```

The current VM is better than before.

It now has:

```rust
NativeMethodContext {
    ...
    activation_id,
}
```

and:

```rust
native_reentry_owners
```

and the current access authority checks the native activation ID rather than just comparing raw frame-vector length.

That is a real improvement.

However the core check is still:

```rust
fn bytecode_reentered_from(&self, activation_id: u64) -> bool {
    !self.frames.is_empty()
        && self.native_reentry_owners.last()
            == Some(&Some(activation_id))
}
```

This proves:

```text
this native activation initiated a nested bytecode drive
```

but does not prove:

```text
the exact bytecode activation created by that drive is still live
```

Those are subtly different after a non-local return.

If the re-entered bytecode frame disappears while some older/outer frame remains, then:

```text
frames.is_empty() == false
```

can still be true even though execution has logically returned to the native callback.

The earlier ruling specifically emphasized stable activation/frame identity rather than vector position.

The current tests improve coverage, but I would add an exact hostile case:

```text
outer bytecode frame A
    ↓
native callback N
    ↓
re-enter bytecode frame B
    ↓
non-local return removes B
but A remains
    ↓
native N resumes
```

At the resume point, access authority must be N's native authority, not A's lexical authority.

If the current code passes that, activation-ID ownership may be sufficient. If not, retain the specific re-entered frame generation/token.

The important audit finding is that `0d6a6e9b` removed the open checkpoint section rather than recording a complete disposition for this ruling.

---

# 36. `Message.args` remains exactly as previously audited

This did not get fixed in the completion work.

Current source:

```phalcom
native class Message is Object {
  native selector -> Symbol
  native name -> String
  native labels -> List

  /// Runtime message arguments retain their call-site-selected shape.
  native args -> Dynamic
}
```

Current Rust native descriptor:

```rust
#[phalcom_native_macros::primitive(
    Message,
    "args",
    params = [],
    returns = Object,
    types = "() -> Object",
    effects = pure
)]
pub fn message_args(...) { ... }
```

The comment above the native method also still says the result is a `List`.

And neither:

```rust
phalcom_type_syntax::TypeExpr
```

nor:

```rust
phalcom_native_meta::TypeExprSpec
```

currently has a dedicated `Dynamic` variant.

So the earlier diagnosis remains valid.

### Scope qualification

I would not make `Message.args` alone a formal P4 blocker because the P4 plan does not explicitly name `Message`. It is, however, an unresolved item from the architectural handoff you gave the implementer and an example of the native/source contract problem that remains in the repository.

If you want to keep P4 tightly bounded, record it as a separate accepted follow-up instead of pretending it was implemented.

---

# 37. The pattern `at → unwrapOr(nil)` fix needs one explicit contract test

The completion commit changed shared pattern element reading from:

```text
collection.at(index)
```

to:

```text
collection.at(index).unwrapOr(nil)
```

This makes sense because the public `at` contract is now `Option<Element>`.

The surrounding destructuring machinery performs tuple/list class and arity checks first, so the use of `nil` should not normally mask an out-of-bounds mismatch.

Still, because this helper is shared by several pattern paths, I would add direct regressions showing:

```text
exact Tuple pattern wrong arity
    → pattern/shape failure
    not accidental nil binding

List fixed prefix wrong arity
    → failure

valid element equal to nil
    → actual nil value remains distinguishable from missing element
```

This is not my primary concern, but the implementation changed runtime destructuring semantics and deserves explicit evidence.

---

# 38. P4 task status after audit

My current task-level classification is:

| Task | Audit status | Main reason |
|---|---|---|
| T1 — lifecycle reopening | **Incomplete closure** | records contradict each other; next action stale |
| T2 — structural kinds | **Implemented substantially; broad regression evidence incomplete** | semantic suite includes generic/header failures |
| T3 — product-shape syntax/AST | **Likely largely implemented** | not independently re-certified in completion |
| T4 — semantic product-shape specialization | **Partial / not accepted** | checkpoint explicitly says coverage/materialization verification remains |
| T5 — runtime specialization layouts/shared class | **Open** | checkpoint explicitly says downstream |
| T6 — declaration/runtime projections | **Improved, not fully accepted** | shared matcher good; runtime conformance regressions |
| T7 — structural domains / Tuple APIs | **Tuple incident substantially fixed; solver still unsoundly incomplete** | no deferred projection proof state |
| T8 — Record/Map/Set migration | **Not shown complete by this work** | not part of final focused evidence |
| T9 — Unit/EmptyRecord runtime | **Earlier implementation exists** | still needs final P4 certification ledger |
| T10 — runtime-owner aliases | **Earlier implementation exists** | final closure evidence still absent |
| T11 — corpus migration | **Incomplete** | 14 stale pack-syntax tests by implementer's own report |
| T12 — metadata/incremental/LSP | **Not accepted** | editor/workspace/incremental failures |
| T13 — certification and closeout | **Not started/completed** | no walkthrough/handoff/CV table; broad suites fail |

That status is fully consistent with the current checkpoint's own retained `IN_PROGRESS / PARTIAL / UNVERIFIED` classification. :chatgpt-content-reference{index="7"}

---

# 39. What I would require the implementer to do next

## Priority 0 — repair inference proof soundness

Do this before chasing broad tests.

Introduce residual/deferred projection obligations.

Then change projection unification so:

```text
exact identical projection
    → Proven

different unresolved projection
    → Deferred

materializable projections
    → normalize
    → compare results

never:
    projection-result equality
    → backwards subject inference
```

And make:

```text
finish_frame
finish_root
```

refuse `Solved` while required projection obligations remain unresolved.

Add direct solver tests, not TypeStore-only tests.

---

## Priority 1 — repair complete-pack canonical identity

Make:

```text
Complete(Positionals(P), Labeled(P))
```

collapse for the canonical intrinsic operations whether they were formed:

```text
internally/source-less
```

or:

```text
through their published source declaration
```

while retaining proper semantic member identity and rejecting unrelated same-name members.

---

## Priority 2 — investigate the 9 runtime regressions as regressions

Do not use the current “Category D baseline” classification.

Bisect/reproduce:

```text
0d6a6e9b
c3862536
cce11d22
historically green LANG005 point if necessary
```

Audit every runtime forwarding seam for:

```text
RuntimeTypeEnvironmentId
RuntimeConformanceEnvironmentId
```

especially:

```text
dispatch_conditional_family_entry
InvokeConditional
rest-method specialized calls
bound/conditional family forwarding
module export forwarding
```

Fix direct-field delegation separately if its root is default-method installation rather than environment propagation.

---

## Priority 3 — classify all 40 semantic failures against known-green history

For each failure:

```text
CURRENT REGRESSION
INTENTIONAL P4 MIGRATION
TRUE CLEAN-PREDECESSOR BASELINE
```

No fourth category.

For syntax migrations, update positive fixtures to the ratified explicit expansion spelling.

For flow tests, prove whether semantics intentionally changed before editing assertions.

For editor/workspace failures, restore canonical publication/indexing behavior unless an accepted decision says otherwise.

For incremental tests, prove cold final-state parity rather than dismissing them as cache-count drift.

---

## Priority 4 — return to T4/T5 rather than jumping to closeout

The checkpoint already tells us these remain unfinished. :chatgpt-content-reference{index="8"}

Finish:

```text
T4 product-shape specialization acceptance
T5 exact runtime specialization layouts
T5 shared behavior-class identity
```

with their plan-defined tests.

---

## Priority 5 — close the native re-entry ruling properly

Restore a durable checkpoint disposition.

Add the hostile:

```text
outer frame remains
re-entered frame non-locally disappears
native callback resumes
```

test.

If the current activation-ID scheme passes it, document why it is equivalent to the required stable activation identity.

If it fails, tie re-entry state to an actual frame generation/token.

---

## Priority 6 — finish P4 gates honestly

Run the plan's complete G1–G5 matrix.

Then only once:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Do not call G6 green if it is red with “baseline” annotations. Record the actual result and separately classify accepted inherited failures where the lifecycle convention permits that.

---

## Priority 7 — produce the required closeout records

Only after T1–T12 acceptance:

```text
LANG007.C3.P4-walkthrough.md
LANG007.C3.P4-handoff.md
```

The walkthrough needs the CV-01..CV-47 table.

Then reconcile:

```text
LANG007.C3-CHECKPOINT.md
PROGRAM.md
STATUS.md
```

to one state.

---

# 40. Final verdict

The implementer solved an important and difficult portion of the incident. I would retain the following changes:

```text
specialized TupleIterator
solver-local TypeMemberProjection representation
projection-variable substitution
witness pre-normalization
semantic invariant generic equality
removal of blanket symbolic→Unknown witness behavior
shared conformance-head matcher
same-subject complete-pack reconstruction concept
Option unwrapping for checked positional pattern reads
```

But I would reject these completion claims:

> “Sound non-injective projections are complete.”

Not yet. Reverse inference was removed, but residual projection equality is not represented as an unproved obligation. The solver can still conflate “did nothing” with “proved.”

> “The nine core failures are baseline.”

Not established, and demonstrably false for several tests with documented historical PASS evidence.

> “The forty semantic failures are prior migration/cache issues.”

Not established. Several exact tests were historically green; multiple failure families are directly owned by P4 T11/T12.

> “All required P4 criteria are finished.”

False. T4/T5 remain open in the authoritative checkpoint, T11/T12 are contradicted by the reported failures, and T13 has no walkthrough, handoff, CV ledger, or broad green certification.

The checkpoint should therefore remain:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C3.P4
```

The narrow TupleIterator regression can be recorded as **functionally resolved / focused-tested**, but the projection solver needs one more architectural correction before I would call even that semantic path fully sound. The current checkpoint's focused TupleIterator evidence is legitimate as far as it goes, but it is not P4 closure evidence. :chatgpt-content-reference{index="9"}