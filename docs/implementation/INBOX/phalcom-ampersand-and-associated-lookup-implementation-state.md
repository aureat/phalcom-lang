# Ampersand and Associated Lookup — Implementation State

Status: implementation complete through C0–C4; focused gates and workspace build are green, while workspace test/clippy remain baseline-blocked.

## Scope

- Plan: `phalcom-ampersand-and-associated-lookup-implementation-spec.md`
- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/ampersand-associated-lookup`
- Branch: `codex/ampersand-associated-lookup`
- Base: `288da3f5 chore(repo): fix formatting drift`
- `main` and the fibers/concurrency worktree were not modified.

## Implemented

- Added a dedicated `&` callable-reference AST and parser surface.
- Retained `::` for associated lookup and removed bound behavioral `::` fallback.
- Added semantic callable-reference resolution products and behavioral-family denotation.
- Added formal lowering for bound families, bound method references, associated families, and associated constructor thunks.
- Reused existing `MakeFamily` and associated-family runtime dispatch paths.
- Updated compiler, source-index, LSP, fixtures, and current specification text.
- Kept exact-getter, operator, and subscript reference spellings out of scope.

## Focused evidence

The following completed successfully:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration`
  - 202 passed, 1 ignored.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic 'associated::' -- --nocapture`
  - 29 passed, 5 ignored.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic 'families::' -- --nocapture`
  - 15 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic --no-run`
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core --no-run`
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic`
  - 1,139 passed, 42 ignored.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_family_runtime -- --nocapture`
  - 14 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core`
  - 461 passed, 24 ignored.
- Core language corpus target
  - 61 passed, 4 ignored.
- Associated lowering, associated reification, and associated runtime focused tests
  - 5, 2, and 1 passed respectively.
- Source-index callable-reference regression
  - 1 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration semantic_tokens_current_syntax::operator_symbols_and_named_family_references_follow_parser_syntax -- --nocapture`
  - 1 passed.
- LSP integration target
  - 59 passed, 2 ignored.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets`
- `cargo fmt --all -- --check`
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy -p phalcom-ast --all-targets -- -D warnings`
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy -p phalcom-semantic --lib --no-deps -- -D warnings`

The full workspace test gate completed all AST, core, LSP, modules, and related targets, then stopped at three unrelated `phalcom-repl` baseline failures: root `PackageInfo` export, module selector property access, and selective selector-class import. Changed files do not include `phalcom-repl` or `phalcom-modules`.

The full workspace clippy gate is baseline-blocked by five existing `phalcom-modules` warnings (`unnecessary_map_or`, `too_many_arguments`, `type_complexity`, and `collapsible_if`). Package-level core and LSP lint also surface only existing warnings outside the changed implementation paths.

The planned negative fixture search found no retired `::*`, `::(_)`, `::...`, or `::()` spellings. The only remaining old terminology match is the intentional historical-retirement sentence in `docs/spec/current/selectors.md`; `BoundBehavioralMember` is current semantic metadata, not a retired resolution kind.

Final review: `git diff --check` passed. The dedicated worktree remains uncommitted on `codex/ampersand-associated-lookup`; the main checkout still contains its pre-existing unrelated edits, including the fibers/concurrency change.

## Remaining next steps

1. If release certification is required, resolve or separately waive the unrelated `phalcom-repl`, `phalcom-modules`, and other pre-existing lint baselines, then rerun the affected workspace gates.
2. Otherwise, review the scoped diff and final worktree status, then make an explicitly authorized commit/push.

No commit or push has been requested.
