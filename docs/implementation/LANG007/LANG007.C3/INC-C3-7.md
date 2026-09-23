# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T7 — incremental signature dependencies and shell publication
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `1c9985b6d449d8680667a91651ea64b2808c44ca`
- Current revision: `1c9985b6d449d8680667a91651ea64b2808c44ca` (working tree changes)

## 2. Escalation trigger

Mandatory trigger 15: exact incremental invalidation is nonlocal and not established
by the current implementation. Trigger 16 also applies because the live graph and
retained-product lifecycle did not yet satisfy the accepted T7 plan contract.

## 3. Required invariant

- `AssociatedSurface` remains the sole owner of associated-name selection and collision
  decisions.
- Alias, generic-header, and source-member checked products use one stable-identity
  dependency graph for scheduling, SCC withholding, and reverse invalidation.
- A changed declaration invalidates only its dependent closure; unrelated siblings
  remain reusable, and incremental results match cold analysis.
- Surface rebuilds include old and new semantic targets for every namespace-affecting
  source edit, including behavior-only inherent impl changes.

## 4. Expected architecture

Persist the same Alias/Header/Member prerequisite graph used for topological scheduling
and mixed-SCC classification. Before dropping retained products, traverse its previous
reverse edges from changed declaration nodes. Rebuild the affected consumer closure,
replace edges for recomputed declarations from current syntax/resolution, and apply
mixed-SCC withholding before publication. Existing alias/header dependency maps may
remain only as projections of that graph.

## 5. Observed behavior

The current working diff builds the Alias/Header/Member graph after earlier recompute
sets have already been chosen. The graph is used for scheduling and SCC classification,
but is not retained for reverse invalidation. Alias recomputation still closes only
alias-to-alias reverse edges; retained member products are filtered out of pending work.

The shell builder gathers behavior names from current eligible inherent impls, but it
seeds affected owners from an impl only when that impl declares associated `type data`.
A behavior-only inherent impl edit can therefore leave owner collision facts stale.

## 6. Minimal reproduction

No runtime reproduction was run for this consultation. The stale-consumer paths follow
directly from the current worklist construction and retained product tables; focused
cold/incremental tests are required after correction.

## 7. Relevant code path

```text
session alias/member/header worklists
  -> retained alias/type-member products selected before signature_graph exists
session signature_graph construction
  -> topological scheduling and SCC detection only
build_source_associated_surfaces
  -> behavior bases include current impls
  -> affected-owner seeds currently require associated_type_data
```

Relevant code is in `phalcom-semantic/src/session.rs`, with formal-query product
dependencies in `phalcom-semantic/src/db/query.rs` and selection dependencies in
`phalcom-semantic/src/checker/context.rs`.

## 8. Scoped implementation diff

The working diff adds the Header node, the early shell-built AssociatedSurface, mixed
SCC withholding, exact formal-query dependencies on selected type-member products, and
retained Alias/Header/Member graph plus reverse adjacency. Source-level declaration
fingerprints seed the previous graph closure before retained products are removed.
Inherent impl owner identities are retained across updates so behavior-only changes
rebuild old and new target surfaces. Fingerprint refresh is scoped to structurally
recomputed modules, preserving retained fingerprints elsewhere. Dependency-edge
collection and SCC classification still rebuild/traverse the full declaration graph;
that performance concern remains open.

## 9. Evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib` passed.
- `git diff --check` passed.
- The focused semantic test lane was not run against this working diff. A prior exact
  semantic test attempt was blocked before the selected test by the canonical Universe
  parser failure in `core/universe/src/concurrency/fiber.ph`.
- Read-only architectural review confirmed stale paths for member-to-alias, alias-to-
  member, and mixed-SCC edits, plus behavior-only impl surface invalidation. The
  correction has since been implemented at the shared graph and shell-refresh seams.

## 10. Implementer's causal hypothesis

The graph's responsibilities were split: it determines what can be scheduled, while
older dependency maps determine what gets recomputed. The two views diverge for
cross-kind dependencies. Surface recomputation similarly uses an incomplete set of
namespace-affecting edit seeds.

## 11. Alternative hypotheses

Formal-query dependency edges may invalidate some later consumers, but they do not
re-lower alias forms or rebuild retained type-member declaration products. They do not
cover the declaration-level scheduler's retained products.

## 12. Attempts already made

The graph and shell publication were implemented and compile-checked. Review exposed
the invalidation gap; the correction now persists the same graph, traverses reverse
edges from changed declaration fingerprints, and tracks old/current impl targets.
Fingerprint refresh is module-scoped, while graph-edge reconstruction remains
workspace-wide. Behavioral, incremental, and performance acceptance remains unverified.

## 13. Why local authority is insufficient

The correction spans prior/current graph ownership, retained products, import-driven
resolution changes, mixed-SCC component invalidation, and source-shard delta tracking.
The shared T7 architecture is fixed. The architectural review resolved the direction:
persist and reuse this same graph, traverse previous reverse edges before dropping
products, and seed both old and new impl target owners. Implementation now follows that
direction; focused cold/incremental evidence is still required.

## 14. Decision requested

No further architecture decision is pending. Verify exact dependent recomputation,
unrelated sibling reuse, behavior-only impl collision refresh, and cold/incremental
parity before treating the T7 invalidation slice as accepted.
