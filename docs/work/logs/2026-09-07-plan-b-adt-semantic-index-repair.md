# Plan B ADT and Semantic-Index Repair

- Date: 2026-09-07
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Status: Implemented, verified, committed, and pushed
- Commit: `94b9f143` (`Repair ADT and Plan B semantic indexing tests`)

## Scope

This work completed the remaining Plan B semantic-index gaps identified in the
investigation report and repaired the stale native `Option`/`None` test model.
The implementation preserves the existing optimized runtime representation and
keeps semantic identity, physical values, and runtime behavior classes distinct.

## Implemented changes

- Batched source-index publication once per changed module instead of attaching
  formal callable analyses through repeated module-wide passes.
- Narrowed incremental source, linked-target, and formal-projection worklists to
  exact rebuild and retirement sets.
- Retained unchanged reference-target and workspace-symbol contributions by
  `Arc`, with tests asserting contribution identity and zero unnecessary target
  updates.
- Projected imports through exact `ImportSiteId`-keyed resolution products,
  including per-segment compound-import prefixes; an errored exact product no
  longer falls back to a legacy path lookup.
- Indexed type references in enum headers, generic binders, variant payloads and
  result annotations, where clauses, and enum/variant behaviors.
- Limited LSP diagnostic source/line-index construction to the diagnostic
  module and modules named by diagnostic labels.
- Repaired the `Option::None` tests to assert canonical variant identity,
  `Value::none()` representation, and hidden `none_class` dispatch behavior
  without requiring a semantic `universe.None` class.
- Strengthened Plan B and source-index regressions, including high-fanout
  retention, exact import-site projection, enum type references, and the
  constructor-side formal/source attachment seam.

## Verification

- Full semantic integration suite: **1,131 passed, 0 failed, 42 ignored**
- Plan B incremental suite: **13 passed**
- Source-index integration suite: **17 passed**
- Imported-resolution integration suite: **9 passed**
- A7 performance suite: **13 passed**
- `cargo check -p phalcom-semantic -p phalcom-lsp`: passed
- `git diff --check`: passed

The broader workspace is not claimed release-complete. Existing unrelated
formatting and language-corpus baseline issues remain outside this repair.

## Delivery boundary

Only the 16 implementation and test files for this repair were committed and
pushed to `origin/main`. The unrelated working-tree files
`docs/.obsidian/workspace.json` and
`docs/audit/runtime-2026-09-07/AUD-RUNTIME-S2-uninvestigated-leads-and-insights.md`
were left untouched.
