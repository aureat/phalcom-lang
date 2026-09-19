---
id: LANG006.C2.P1
kind: implementation-plan-handoff
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
base_revision: c3a53c3e364e588d46bd56ade246976e1996d744
working_tree: uncommitted scoped C2 implementation changes
---

# LANG006.C2.P1 — Handoff

## Current state

LANG006.C2.P1 is complete as an implementation vertical and focused-tested.
The current worktree contains the scoped C2 changes and required records plus
the preserved unrelated untracked `docs/implementation/TEST002/` plan
artifact; no commit or push was requested after the initial all-worktree
synchronization.

## Stable takeover map

- `phalcom_ast::ast::DataDef` owns normalized primary-constructor visibility;
  `parse_data` accepts both existing `where` placements and the canonical
  trailing form.
- `DataConstructorSignature.visibility` in
  `phalcom-semantic/src/data_semantics.rs` is the semantic source of truth.
- `CheckingContext::declaration_member_is_accessible` and
  `data_constructor_is_accessible` consume the shared
  `is_member_accessible` relation and canonical topology provenance.
- Direct construction uses `CallableApplicationTarget.access_denied` before
  `apply_resolved_callable`; inaccessible existing constructors remain distinct
  from missing constructors.
- `specialize_associated_member` authorizes exact/family constructor capability
  acquisition. Captured data-constructor denotations are stable on later use;
  do not add invocation-time lexical reauthorization.
- `data_declaration_input_fingerprint` and
  `data_declaration_product_fingerprint` include visibility. Do not add
  visibility to `DataConstructorId`, nominal forms, data values, descriptors,
  bytecode, VM dispatch, or optimizer decisions.

## Invariants to preserve

- Private means exact owning data declaration, not same module.
- Module/package/internal use exact canonical module, package, and project
  identities; no path-prefix or string inference.
- Re-export/import can expose the type name but cannot widen constructor access.
- Tuple and record construction, nullary singleton construction, and associated
  constructor paths all authorize before lowering.
- Data component transparency and representation semantics remain unchanged.

## Verification already obtained

Focused AST, semantic, topology, associated, incremental, optimizer, package
check, and diff-check evidence is recorded in the walkthrough and checkpoint.
The full AST integration target retains three inherited C1 snapshot failures;
workspace/release certification and broad formatter repair remain deferred.

## Recommended next action

Continue with the next explicitly scoped LANG006.C2 plan, or perform a separate
checkpoint certification request. Do not repair the inherited snapshot or
formatter baselines inside this plan.
