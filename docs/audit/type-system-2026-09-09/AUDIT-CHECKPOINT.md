# Type-system audit checkpoint — 2026-09-20

## Active repair

**Generic override, captured-write, and canonical subtype-relation soundness**

- **Status:** IMPLEMENTED
- **Verification:** FOCUSED_TESTED
- **Scope:** `AUD-SEM-001` override certification, `AUD-SEM-002` captured mutable-local invalidation, and `AUD-SEM-004`/`AUD-SEM-005` canonical relation repairs.

## Root cause

Override validation correctly specialized declaration-owned generic parameters through the canonical receiver-to-owner projection, but compared callable-owned generic parameter IDs directly. Equivalent inherited and overriding method binders therefore differed by `TypeParameterOwner::Callable`, causing false `IncompatibleOverride` diagnostics for symbolic/HKT methods.

## Established rule

Override compatibility first projects both signatures through `specialize_receiver_to_owner`. It then alpha-aligns only corresponding candidate callable binders to inherited callable binders after matching arity, kinds, and variance metadata. Parameter contravariance and result covariance continue through the canonical relation engine. Generic constraints are normalized by the same substitution and must match; this conservative rule prevents strengthened caller obligations without introducing a duplicate constraint-implication engine.

Declaration-owned parameters, `Self`, selectors, labels, rest shape, and all non-callable free identities remain unchanged.

## Implementation

- `phalcom-semantic/src/checker/declaration.rs`
  - alpha-aligns callable generic binders after ancestor specialization;
  - checks generic kind/variance metadata and normalized constraints;
  - retains existing selector-shape and variance relation checks.
- `phalcom-semantic/tests/audit_probes/override_certification.rs`
  - adds a valid symbolic HKT override regression;
  - adds an incompatible symbolic HKT result regression.

## Evidence

Passed:

```text
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::alpha_equivalent_generic_hkt_override_is_accepted -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::generic_hkt_override_with_incompatible_result_is_rejected -- --nocapture
cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture
cargo test -p phalcom-core --test core \
  typing_integration::monads::bodies -- --nocapture
```

`typing_integration::monads::bodies` passed all three symbolic-body regressions, including `traverse_body_resolves_symbolic_inherited_capabilities`. The expected `Monad<F> -> Applicative<F> -> Functor<F>.map` resolution now reaches the asserted `Assumed` symbolic-evidence products without override diagnostics.

The full `audit_probes::override_certification` selector remains baseline-blocked by unrelated `AUD-SEM-002` captured-write coverage; 7 tests passed, 2 ignored retained audits, and only `captured_write_must_invalidate_previous_int_fact` failed. `cargo fmt --all -- --check` is also baseline-blocked by unrelated formatting in pre-existing modified files.

## Next action

`TEST002.C1.P1 / T8` resumed and reached its intended symbolic-evidence assertions. BL-19's generic-override prerequisite is resolved; no further work on this override gate or symbolic inherited-capability trace is currently required.

## AUD-SEM-002 — captured mutable write

### Root cause

Closure construction correctly restores the enclosing `FlowState`. Every call
then enters the shared opaque-call barrier, which invalidated mutable predicates
and field-current facts but left mutable local `BindingState.current` and its
version untouched. A captured `Int | String` local initialized with `Int` could
therefore retain `Established Int` after the invoked closure wrote `String`.

### Established rule

At an opaque call boundary, mutable binding current knowledge and denotation are
transient and must be invalidated; persistent `BindingContract` remains. The
canonical binding helper derives a conservative replacement: a source annotation
becomes assumed contract knowledge, while non-source contracts and unconstrained
bindings become `Unknown(NoTypeEvidence)`. The invalidated binding advances its
version and loses its stale explanation. Immutable locals remain precise, and
field fallback remains owned by the existing field-lifecycle path.

This intentionally invalidates uncaptured mutable locals too. It is the current
conservative opaque-call policy, documented by regression coverage; precise
captured-write summaries are future work and must not weaken this repair.

### Implementation and evidence

- `phalcom-semantic/src/checker/binding.rs` owns conservative current-knowledge
  derivation from a retained binding contract.
- `phalcom-semantic/src/checker/flow/state.rs` applies it to mutable locals in
  `invalidate_opaque_calls()` while retaining contracts and invalidating stale
  predicate/explanation/version state.
- `phalcom-semantic/tests/semantic/capabilities/deep_regressions.rs` covers
  annotated captured writes, aliases, nested helper invocation, same-type
  writes, immutable precision, and the intentional uncaptured-local boundary.

Passed:

```text
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::captured_write_must_invalidate_previous_int_fact -- --nocapture
cargo test -p phalcom-semantic --test semantic capabilities::deep_regressions -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  capabilities::flow_loops::captured_block_write_is_not_applied_until_execution_is_proven -- --nocapture
```

## AUD-SEM-004 — active union-relation lifetime

### Root cause and rule

`check_subtype_impl` used one active-pair set for recursion detection, but early
terminal returns from union and structural branches bypassed its final removal.
A completed relation could therefore remain active and make an independent
sibling obligation appear recursive. The active pair is now owned by the
invocation that inserted it: the full relation body runs in a result-producing
scope, then removes the pair unconditionally before returning that result.
Pairs detected as already active do not own removal.

This remains active-recursion tracking only. It does not add result memoization,
change terminal-outcome precedence, or change AUD-SEM-007 budget accounting.

### Evidence

The repeated union obligation and allocation-order regressions now pass. The
AUD-SEM-007 retained budget probe remains red when explicitly enabled: its
observed `pairs_checked = 1` for two nested obligations remains separately
classified and was not changed by this work.

## AUD-SEM-005 — applied receiver to nominal ancestor

### Root cause and rule

An applied receiver had a generic-template projection path only when the target
was also applied. `Applied -> Nominal` therefore rejected a valid nongeneric
ancestor, even though member specialization could traverse the same hierarchy.

`types::specialization::project_direct_supertype` now owns direct ancestor
materialization for both member specialization and subtype relation checking.
It preserves declaration-owner substitutions and kinded type forms. The subtype
relation projects exactly one direct ancestor and recursively invokes the
canonical relation against the original target; it does not erase arguments or
declare ancestry success directly.

### Evidence

Passed:

```text
cargo test -p phalcom-semantic --test semantic audit_probes::subtyping_bounds -- --nocapture
cargo test -p phalcom-semantic --test semantic audit_probes::override_certification -- --nocapture
cargo test -p phalcom-semantic --lib types::relation::tests -- --nocapture
cargo test -p phalcom-core --test core typing_integration::monads::inheritance -- --nocapture
```

The enabled source control now accepts `Child<Int>` where `Base` is required.
Additional probes cover transitive nongeneric ancestry, generic multi-hop
substitution, invariant argument mismatch, and unrelated nominal rejection.

## AUD-SEM-003 — applied type-form lowering

The canonical-Universe diagnostic baseline lock was removed from bootstrap
compilation. Canonical bootstrap now projects its reviewed source surface despite
semantic diagnostic drift; ordinary user/project compilation continues to reject
semantic errors through `compile_analyzed`.

With that unrelated compilation blocker removed, both existing applied
`Set`/`Map` construction and the new production `Box<Int>.new()` regression
compile and execute. Runtime type-form lowering consumes the semantic
`ClassObject` projection for a declaration-backed applied type form, so its value
is the nominal class object rather than `Nil`. The historical lowering defect is
therefore resolved for the supported nominal applied type-form surface.

## AUD-SEM-006 — Unit normalization

The canonical `TypeStore::tuple` constructor already normalizes an empty tuple
to `store.unit()`, so both `Unit` and `()` annotations resolve to the same
proper type as the `()` value literal. The retained source probe passes, and
new direct-store plus source regressions cover canonical identity in explicit
annotations, callable results, defaults, and a `Unit | Int` expected context.

No relation-level equivalence bridge is required: the canonical construction
boundary provides the single identity before assignability or diagnostics run.

The formerly ignored tuple-union source assignment is not a Unit issue. It
became green with AUD-SEM-004's active-relation-pair lifetime repair: repeated
component obligations no longer inherit stale cycle state. Homogeneous and
heterogeneous tuple components are now enabled regressions under the relation
audit surface.

## AUD-SEM-007 — relation budgets

Relation accounting now occurs at every `check_subtype_impl` invocation rather
than only in the public root wrapper. Each relation obligation charges one step
and one pair; recursive calls carry an explicit depth and report a distinct
`BudgetExceeded(TypeDepth)` when exceeding `max_type_depth`. The root wrapper
only initializes active-cycle state.

This preserves cancellation precedence, active-pair cycle semantics, and
terminal relation outcomes. It does not add memoization or separately charge
generic template materialization/substitution. Enabled probes cover pair limits
below and at the exact required obligation count, plus depth below and at the
deepest nested comparison.
