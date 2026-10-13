# Shadow Universe modernization guidelines

## Scope and authority

This is a source-quality program for `packages/universe/src`, the shadow tree. The live production Universe remains on its legacy source provider and production native/prelude/bootstrap mappings. Do not infer cutover or canonical identity from shadow paths, manifests, parser success, or source hashes. Follow `docs/specs/README.md` and the effective normative specification for every behavior-sensitive edit; use `packages/universe/docs/identity-migration.md` for current-vs-shadow ownership and `migration-status.md` for phase status.

Keep source modernization separate from ownership moves, semantic-linking changes, data representation changes, and runtime integration. Preserve signatures, module ownership, identity, constructor invariants, exception behavior, branch evaluation, iterator consumption, closure capture, evaluation timing, and resource/lifecycle behavior. If a proposed edit may change any of those, stop that slice and request architectural guidance under the repository escalation protocol.

## Evidence and inventory conventions

- Treat `INVENTORY.md` hashes and branch/revision as the baseline snapshot; refresh them before a later pilot if source changes.
- Distinguish AST/parser/interface evidence from textual occurrence counts. Text matches identify review locations only; they do not prove a migration opportunity.
- Record ambiguous receiver categories, nested/generated declarations, documentation association, public API typing, dataflow, and opportunity counts as unresolved until AST-aware/manual review.
- Do not interpret `PENDING` or a source-only edit as verification. Track status using the full vocabulary in `STATUS.md`.
- Report test filters and completed evidence precisely. Do not claim shadow semantic or runtime verification from parse/interface checks.

## Modernization rules

- Prefer native `match`, `if let`, `while let`, and `for` only when the current parser and effective specification support the exact form. The packet explicitly prohibits unverified `for let` syntax and implicit filtering through `for` patterns.
- Replace generated `.match` calls only after confirming the receiver type, branch exhaustiveness, selected-branch evaluation, and error/exception behavior. Do not bulk-rewrite.
- Replace manual Option/iterator extraction only after preserving next-call count, termination behavior, mutation order, and suspension/error propagation.
- Use trailing or expression closures only when selector/argument binding, capture, and evaluation timing remain equivalent. Do not change memoization or laziness.
- Remove `mut` only after a dataflow check; name compact control-flow candidates rather than formatting whole modules.
- Add module/public docs where absent and useful. Avoid redundant comments and stylistic API renames.
- Treat native declarations and reflection, scheduler, resource, and identity paths as elevated-risk boundaries. Do not infer host-to-data support; verify constructor, heap/GC, reflection, cache-key, and conversion contracts before any data migration.

## Ordering and exclusions

Use the packet's priority order, but coordinate collection/iteration work with S4 and do not touch modules currently participating in that source reorganization. Avoid active P4 semantic-linking work, including the concurrent `session.rs` and `workspace.rs` changes and any shadow reflection/typing or linker-dependent slice implicated by its live handoff. The first pilot must be selected from unaffected source modules and should be small, behavior-preserving, and separately reviewable. Do not edit production Universe or remove legacy sources without separate cutover authority.
