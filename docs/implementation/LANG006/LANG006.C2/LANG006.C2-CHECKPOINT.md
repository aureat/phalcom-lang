---
id: LANG006.C2
category: LANG
program: LANG006
checkpoint: LANG006.C2
kind: checkpoint-record
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
requires:
  - LANG006.C1
  - LANG005.C1
active_plan: null
latest_completed_plan: LANG006.C2.P1
next_plan: null
planning_baseline_revision: c3a53c3e364e588d46bd56ade246976e1996d744
prepared: 2026-09-19
repository: aureat/phalcom-lang
---

# Checkpoint Record — LANG006.C2 Validated Data Construction

## 1. Objective and ownership

LANG006.C2 adds visibility to compiler-generated primary data constructors
without changing data transparency, nominal identity, component observation,
runtime representation, or optimizer behavior. It owns the post-name data
constructor syntax, semantic constructor visibility, canonical access at direct
construction and capability acquisition, incremental relevance, and focused
specification/test closure.

The plan deliberately preserves both currently accepted `where` placements.
The canonical new fixture style is the trailing form:

```phalcom
data Box<T> private(_ value: T) where T <: Object
data Box<T> private() where T <: Object
```

## 2. Plan ledger

| Plan | Status | Completion | Verification | Durable state |
| --- | --- | --- | --- | --- |
| `LANG006.C2.P1` | COMPLETE | COMPLETE | FOCUSED_TESTED | Post-name primary-constructor visibility, canonical access, capability closure, incremental and representation independence |

## 3. Established inherited invariants

- `phalcom_common::modifier::Visibility` is the sole visibility category.
- Omitted visibility is `Public`; there is no `public` source keyword.
- C1 `is_member_accessible` owns identity-based access for owner, module,
  package, and project provenance.
- `DataConstructorId` identifies the owner and does not contain visibility.
- Data values remain transparent immutable products; constructor visibility is
  checked before lowering and is not runtime value metadata.
- Both existing `where` placements remain accepted; the trailing post-shape
  form is canonical for new source and fixtures.
- Direct tuple, record, and associated constructor invocation consume the same
  `CheckingContext` declaration-member access helper.
- Exact/family constructor capabilities are authorized at acquisition and are
  not re-authorized after an authorized escape.
- Visibility participates in data input/product fingerprints while
  `DataConstructorId` and nominal identity remain stable.

## 4. Active C2 interfaces

| Fact | Canonical owner | Consumers | Forbidden duplicate |
| --- | --- | --- | --- |
| constructor syntax | `phalcom_ast::ast::DataDef` / `parse_data` | semantic data builder | generic top-level modifier parser |
| constructor visibility | `DataConstructorSignature.visibility` | direct and associated acquisition | `DataInfo` or runtime copy |
| access law | `phalcom_semantic::access::is_member_accessible` | checker helper and call application | data-specific visibility match |
| constructor identity | `DataConstructorId` | semantic/lowering/runtime mapping | visibility-sensitive IDs |
| value representation | existing data descriptors/objects | compiler/VM/optimizer | per-value visibility metadata |

## 5. Verification ledger

T0 completed at revision `c3a53c3e364e588d46bd56ade246976e1996d744` after the
all-worktree C1 commit/push. Live symbol checks confirmed the planned AST,
parser, semantic data, access, associated-constructor, and fingerprint owners.
No tests were run during T0.

Focused C2 evidence:

| Gate | Evidence | Result |
| --- | --- | --- |
| G1 | AST visibility tests; semantic constructor product tests; semantic check | PASS |
| G2 | 19 data-declaration tests; 5 topology/access matrix tests | PASS |
| G3 | exact/family/associated-invocation denial and authorized capability escape tests | PASS |
| G4 | 4 incremental data tests; 17 product-optimization tests; semantic/core checks | PASS |
| G5 | focused AST, semantic, incremental, associated, core, and `git diff --check` evidence | PASS |

## 6. Deferred and baseline state

The full AST integration target remains baseline-blocked by three inherited C1
snapshot mismatches (`invocation_authority: Ordinary` and dunder field-kind
output). They are outside C2 and were not updated. `cargo fmt --all -- --check`
also reports inherited formatting drift across unrelated packages and files.
The unrelated untracked `docs/implementation/TEST002/` plan artifact was
preserved. No broad formatting rewrite or unrelated cleanup was made.
Workspace/release certification remains deferred.

## 7. Completion and takeover

`LANG006.C2.P1` is implemented and focused-tested. Semantic authorization is
complete before lowering; runtime data representation and product optimization
remain visibility-free. Required walkthrough and handoff records capture the
evidence and the inherited baseline boundaries. The next action is the
checkpoint's subsequent plan, if any, starting from this focused-tested state.
