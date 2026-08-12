# Merge gates

## Semantic correctness
- Literal/construct inference is deterministic.
- Aliases, shadowing, reassignment, and joins follow the specified semantics.
- Unions deduplicate and widen at the specified boundary.
- Callable summaries converge under recursion.
- Arbitrary class-side calls are not assumed to return receiver instances.
- Opaque native code without a semantic contract yields `Unknown`.

## Member resolution
- User members appear after `instance.`.
- Inherited visible members appear.
- Overrides do not duplicate selectors.
- Class-side/instance-side surfaces do not leak.
- `super` starts from lexical superclass and excludes child-only members.
- Chained receiver completion uses expression knowledge.
- Unknown receivers do not become an unbounded generated builtin dump.

## Modules / identity
- `import "./x" as X` exposes the correct module surface.
- `A.User` and `B.User` remain distinct class identities.
- Provider edits invalidate dependent results.
- Missing/cyclic imports degrade without panics.

## User-facing LSP
- initialize advertises the implemented providers.
- completion/inlay/hover consume the same semantic model.
- `Unknown` is not rendered as a useful fake type.
- existing diagnostics/navigation/references/symbol tests remain green.

## Live editing
Without restart:
- Cat -> Dog changes completion/hints;
- add/remove method changes completion;
- provider edit updates consumers;
- unsaved content beats disk;
- close restores disk-backed semantics.

## Syntax
- semantic token fixture follows current grammar;
- expectations are human-readable text/kind pairs;
- comment fallback remains explicit while lexer trivia is unavailable.

## VS Code
- actual extension-host E2E starts real `phalcom-lsp`;
- declared and inherited completion work;
- live edit updates completion;
- VSIX packaging succeeds and manual install smoke passes.

## Commands
```sh
cargo fmt --check
cargo test -p phalcom-lsp
scripts/test.sh workspace
scripts/editor.sh vsphalcom
scripts/test.sh full
```
