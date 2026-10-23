# Incident: semantic imports still assume Universe ownership

Date: 2026-10-23. Stage: architectural review during F2, for slice G.

## Evidence

`metadata/stable_identity.rs::from_stable_project` accepts only the exact
Universe builtin identity. Package identities return an error requiring an
exact `StableOwnerCatalog` binding.

`artifact/import.rs::import_stable_module_with_allocation_budget` calls that
context-free conversion. Its declaration/callable/impl helpers are used
throughout structural, generic, callable, and nested semantic imports.
`import_structural_primary_contracts_with_owner_catalog` receives the catalog
but still imports data owners through the context-free declaration helper.
The catalog currently covers type-graph and intrinsic owners, not this full
import surface.

This disproves EXP-C4's original claim that import already resolved stable
references through the catalog. It would block a real package semantic row
even after V5 external module loading was implemented. This is source
evidence; no hostile package load has been executed for this incident.

## Decision

PLAN NEEDS MATERIAL AMENDMENT. Add LD-C4 and a G prerequisite that threads the
existing catalog through every semantic owner import. Explicit context must
reach both budgeted helpers and context-free conversion sites. Preserve
exact receipts, allocation transactions, and Universe compatibility wrappers.
No ambient owner context or alternate identity mechanism is permitted.

F1 remains complete. F2 and F3 can continue because F3's load self-check is
already deferred until G. The implementer may resume those slices and must
complete LD-C4 before claiming package composition.

Shared type-store interning also must not be treated as external declaration
existence evidence: `type_import.rs::validate_and_intern_type` can intern a
nominal owner after symbolic relocation. LD-C3 is now explicitly VERIFY-FIRST;
LDH-7 must establish rejection through the final production path.

## Verification

Import package-owned structural and callable rows with an exact receipt;
reject missing or wrong receipts without publication. Then prove VER-H3 and
LDH-7 through composition. The full LDH matrix and Universe regression gate
remain required. This prose amendment runs no compilation or tests.
