---
id: LANG006.C1.P3
kind: implementation-plan-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
base_revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
working_tree: uncommitted scoped implementation changes plus preserved unrelated STDL002 checkpoint work
---

# LANG006.C1.P3 — Implementation Walkthrough

## Result

P3 is complete as an implementation plan. It removed the compiler-generated
constructor authority regression, verified the normalized source-visibility
publication path, migrated stale certification fixtures to current language
authority, corrected the bounded `Selector.slots` generic contract, and ran
the required one-time broad semantic/core classification pass.

The plan is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. Focused P3 and P2
predecessor gates pass. Broad suites still contain independent SEMA,
iteration/argument-expansion, typing, and LANG005 conformance-runtime defects;
those are recorded below and are not P3 implementation work.

## Starting baseline

T0 reproduced the two P3-owned false reds:

- generated constructors reached `Class.allocate()` through ordinary invocation
  and were denied by internal access;
- the source visibility surface test expected an unmodified `_field` to be
  Private even though the ratified default is Public.

The predecessor broad baselines were semantic `1,420 passed / 22 failed / 50
ignored` and core `537 passed / 74 failed / 28 ignored`.

## Implemented changes

### Compiler-synthetic authority

`InvocationAuthority` is now an explicit call-site fact on method-call AST
nodes. Parser and ordinary compiler-created calls remain Ordinary. Only the
synthetic `self.allocate()` emitted by `lower_constructors()` is marked
CompilerInternal. `emit_invoke` consumes that fact and emits
`InvokeCompilerInternal` only for that call.

The VM authority remains dispatch-scoped: the compiler-internal depth is
available while selecting/entering the privileged dispatch and is removed
before initializer or user code executes. Ordinary source cannot construct the
authority, and nested ordinary calls do not inherit it.

### Visibility publication

The live source path is already the canonical one:

```text
ClassMember::visibility()
    -> build_declared_class_surface()
    -> declaration surface/query products
    -> published semantic snapshot
```

No second semantic source of truth or legacy prefix/attribute inference was
added. The failing field assertion was corrected to the current default-public
rule. A narrow parser lookahead fix was added so `private [_ index]` stores the
modifier on a subscript member without disturbing reserved-word method-name
lookahead.

### Stale authority migrations

- Enum behavior fixtures now use accepted root and exact-case `impl` forms.
- Cursor-style `iteratorValue` fixtures now use explicit Iterator/Iterable
  evidence; the negative case proves fail-closed behavior without canonical
  iterable evidence.
- Associated projection metadata now tests export and stable identity rather
  than the retired rejection branch.
- Private subscript coverage uses the current modifier syntax.
- The source non-forgeability oracle expects ordinary missing-message behavior
  for the retired selector-prefix namespace.

### Universe generic contract

`reflection.selector` now declares `native slots -> List<Symbol>`, and the
native primitive descriptor uses the same applied result type. The generated
native surface was regenerated and the one obsolete unsaturated-constructor
semantic-diagnostic baseline entry was removed.

## Focused verification

| Gate | Evidence | Result |
| --- | --- | --- |
| G1 | transient compiler authority, source non-mint, generated-constructor bytecode/provenance, constructor/subscript regressions | PASS |
| G2 | compiler surface visibility; semantic access-control group | PASS; access-control 8 tests |
| G3 | enum, canonical iteration, associated projection, private subscript, module linking, native generator-current checks | PASS |
| G4 | P2 access-control, native conformance, reflection/Foreign, authority parity, semantic/core checks | PASS |
| T2 | exact source visibility plus `cargo check -p phalcom-semantic` | PASS |
| T3/T4 | migrated semantic/core filters, module-linking filters, native conformance, surface generator | PASS |

The focused compiler and semantic crates both compile. The generated native
surface reports 334 current primitive declarations.

## Broad reclassification

The required one-time broad runs produced:

- semantic: `1,428 passed / 14 failed / 50 ignored`;
- core: `607 passed / 5 failed / 28 ignored`.

No remaining broad failure reports constructor `Class.allocate()` access
denial. The former constructor cascade therefore collapsed as required.

## Deferred residuals and owners

All remaining failures are D/baseline work relative to P3:

| Owner | Failures |
| --- | --- |
| SEMA flow/evidence | `opaque_call_drops_refined_field_current_without_rewriting_contract`; `unknown_expansions_do_not_disappear` |
| SEMA canonical subscript application | `assumed_list_receiver_caps_structural_index_result`; `list_subscript_checks_index_contract`; `list_subscript_set_checks_index_and_value_and_returns_unit`; `map_subscript_checks_key_contract` |
| SEMA incremental/product stability | `adding_impl_member_does_not_invalidate_another_target_or_its_caller`; `dependency_edit_remove_readd_recomputes_affected_summary_deterministically`; `canonical_universe_reads_track_canonical_query_dependencies`; `legacy_universe_core_path_is_not_a_dependency_sentinel`; `signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies`; `superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies`; `a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products`; `unused_public_export_does_not_recompute_unrelated_consumer_body` |
| Argument expansion / canonical iteration | `compiler_declaration_dispatch::positional_spread_of_non_iterable_has_structured_boundary`; `observability_traceback::test_list_spread_traceback_includes_spread_site` |
| LANG005 conformance runtime | `conditional_conformance::cv46_runtime_conditional_default_executes`; `generic_conformance::bounded_generic_function_dispatches_exact_witness_and_default` |
| Typing integration | `typing_integration::monads::bodies::traverse_body_resolves_symbolic_inherited_capabilities` |

The two observed argument-expansion failures are iterator/DNU boundary-shape
failures, not access-control regressions. The conformance failures report no
active conformance target during default dispatch, outside the P3 authority
seam. The typing failure is a blocked inherited-capability analysis product.

## Consultations and deviations

No architectural consultation was required. The only parser change was a
mechanical lookahead correction required to store an already accepted private
subscript modifier. No ownership, identity, VM authority lifetime, semantic
authority, or accepted language rule was redesigned.

## Final classification

P3 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. The checkpoint is
`COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. Workspace, clippy, formatter, and
release certification were deliberately not run by the plan and remain open.
