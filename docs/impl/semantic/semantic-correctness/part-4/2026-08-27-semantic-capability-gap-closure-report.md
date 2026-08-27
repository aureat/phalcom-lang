# Phalcom Semantic Capability Gap Closure Report

Date: 2026-08-27

## Outcome

Tasks 0–10 are implemented and verified for the capability-gap scope. The
Technical 03 baseline remains authoritative. Formal semantic products, query
dependencies, source attachments, and advisory presentation remain separate.

## Implemented

- Trusted branch refinement requires the canonical core `Object.is` or
  `Object.is!` callable identity. Method spelling alone cannot refine. Abrupt
  `throw` exits remain separate from normal branch values.
- Structural callable `.call()` and direct lexical invocation share the
  canonical callable-application path.
- List element/rest pattern decomposition preserves `Known`, `Unknown`, and
  `Dynamic` epistemic categories and reasons for declaration and runtime
  patterns.
- Field contracts, lifecycle proofs, and path-local current field facts are
  separate. Constructors publish definite initialization; reads and writes
  consume/update current flow state without rewriting contracts.
- Callable input identity ignores absolute source ranges while retaining body
  source content, semantic syntax, namespace surfaces, linked availability,
  and field lifecycle inputs. Product identity remains semantic. Reused
  callable `Arc` values stay stable across trivia movement.
- Current source ranges are maintained in compiler-owned source indexes and
  formal projections, so reused semantic products do not publish stale
  positions.
- Incremental final dispositions are derived after inferred-return refresh.
  Removal and re-addition recompute unresolved callers instead of retaining
  stale products.
- Imported bindings retain lexical declaration metadata while reads resolve to
  canonical external declaration or module targets.
- Temporary debug diagnostics and the three plan-specific capability ignores
  were removed.

## Focused verification

All six closure scenarios passed, one test each:

```text
refined_branch_with_abrupt_else_publishes_only_normal_value
higher_order_block_call_propagates_captured_result
field_facts_survive_constructor_and_general_writes
collection_and_destructure_facts_preserve_element_shapes
dependency_edit_remove_readd_recomputes_affected_summary_deterministically
imported_binding_use_resolves_to_exported_declaration_not_local_import_site
```

Category results:

```text
Technical 03 generics: 19 passed
capabilities:          128 passed
incremental:            87 passed
phalcom-semantic lib:   38 passed
full semantic target:  468 passed, 12 intentional ignored
```

The full semantic target had zero failures and no unexpected internal
semantic incidents. Expected negative fixtures may print caught panic traces;
their tests still passed.

Additional gates passed:

```text
cargo check -p phalcom-semantic
cargo fmt --all -- --check
git diff --check
```

## Baseline failure

`cargo clippy -p phalcom-semantic --all-targets -- -D warnings` remains red on
the pre-existing generated native surface: 259 `clippy::deref_addrof`
diagnostics in `phalcom-native-surface/src/generated.rs`. This is outside the
semantic gap changes and was not absorbed into this closure.

## Ledgers and remaining gaps

`BASELINE_LEDGER.md` now marks all five capability-gap scenarios and imported
identity scenario `READY`. `COVERAGE_LEDGER.md` promotes source slots S06 and
A01; current count is 56 READY, 16 STAGED, 34 GATED.

The remaining 12 ignored tests are intentional golden tests outside this
plan: generic inheritance/Self specialization, nominal branch product joins,
iteration stabilization, Family activation, type-lambda parser/constraints,
linked workspace publication, variance recovery, closure contextual typing,
structural-record recovery, recursive generic fixed points, and row/effect
contracts. Their ignores remain unchanged.

The broader STAGED/GATED ledger slots remain future semantic work and are not
claimed by this report.

## Provenance

Technical 03 base: `f0bec822bf7e9cdeada55faa0f27a1e3cda208ca` (already present on
`origin/main` at handoff). Implementation commit: `7be08a22`. Documentation
commit is recorded in the delivery summary and Git history.
