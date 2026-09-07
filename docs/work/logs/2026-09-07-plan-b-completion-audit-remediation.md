# Plan B Completion Audit Remediation

- Date: 2026-09-07
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `codex/plan-b-completion`
- Status: Implemented, verified, committed, and pushed
- Implementation commit: `e136ecb9` (`fix(plan-b): close completion audit gaps`)
- Documentation commit: `44f4f1d5` (`docs: record Plan B audit remediation`)

## Scope

This work addressed the independent audit findings for the Plan B completion
patch. It stayed within semantic source-index publication, formal projection
reuse, LSP request adaptation, regression evidence, and the implementation
ledger. Unrelated core and language-corpus baseline failures were not part of
this work.

## Remediation

- Removed retained-source removal rediscovery from the production delta-driven
  update path. Exact Plan-A removal worklists now retire modules; the direct
  full-workspace input path retains only its documented compatibility fallback.
- Made production source indexing fail closed when an authored import lacks its
  canonical `ImportResolutionProduct`; legacy path maps remain available only
  to compatibility-level source-index callers.
- Computed formal projection reuse arithmetically from the previous projection,
  rebuilt modules, and retired modules instead of scanning the new source index.
- Added formal projection module counts for that constant-time accounting.
- Isolated workspace-symbol location conversion from reference/definition
  performance counters.
- Strengthened PB-10 to compare the complete published editor projection,
  PB-11 to cover import retargeting and old-snapshot behavior, and PB-12 to
  exercise body, reference, presentation, and removal updates through the real
  module-delta path.
- Added a production fail-closed import regression and expanded LSP acceptance
  coverage for alias definitions, import-prefix navigation, local and upstream
  references, bounded location conversion, and workspace-symbol counter
  isolation.
- Updated the Plan B implementation ledger to distinguish focused-green,
  audit-remediated status from release-complete status.

## Verification

- `cargo test -p phalcom-semantic --test semantic plan_b_indexing -- --nocapture`:
  **15 passed**
- `cargo test -p phalcom-semantic --test semantic source_index -- --nocapture`:
  **20 passed**
- `cargo test -p phalcom-semantic`: **1,134 passed, 42 ignored**
- `cargo test -p phalcom-lsp --test plan_b_indexing -- --nocapture`:
  **2 passed**
- `cargo test -p phalcom-lsp`: all package targets passed; performance harness
  cases remained ignored as designed
- Scoped `rustfmt --check` and `git diff --check`: passed

## Delivery boundary

The two remediation commits were pushed to
`origin/codex/plan-b-completion`. The worktree is clean and synchronized with
the remote branch. Workspace-wide release certification remains outside this
Plan B checkpoint.
