# Applying the testing kit

Assume:

```sh
REPO=/path/to/phalcom-lang
KIT=/path/to/phalcom-lsp-testing-kit
cd "$REPO"
```

## 1. Copy the files

```sh
rsync -a "$KIT/repo-files/" "$REPO/"
```

or:

```sh
cp -R "$KIT/repo-files/." "$REPO/"
```

Review conflicts if the implementation agent independently created files with the same names.

## 2. Wire the Rust integration target

`phalcom-lsp/Cargo.toml` has `autotests = false`, so add these modules to `phalcom-lsp/tests/integration.rs` while keeping the existing Stage 1–5 modules:

```rust
mod support;

mod fixture_syntax;
mod semantic_completion;
mod semantic_consistency;
mod inlay_hints;
mod workspace_semantics;
mod semantic_tokens_current_syntax;
```

## 3. Make the LSP lane run unit + integration tests

Change the `lsp)` arm in `scripts/test.sh` from:

```sh
cargo test -p phalcom-lsp --test integration "$@"
```

to:

```sh
cargo test -p phalcom-lsp "$@"
```

This matters because semantic-engine unit tests under `src/semantic/` must also run.

A standalone `repo-files/scripts/editor.sh` is included:

```sh
scripts/editor.sh lsp
scripts/editor.sh vsphalcom
scripts/editor.sh all
scripts/editor.sh vsix
```

## 4. Validate fixture syntax first

```sh
cargo test -p phalcom-lsp --test integration fixture_syntax -- --nocapture
```

Normal completion fixtures remain valid source by placing the cursor marker before an existing member:

```phalcom
person./*@completion*/greet()
```

The loader sends `person.greet()` to Phalcom while remembering the cursor immediately after `.`.

Only `tests/fixtures/incomplete/` is intentionally malformed.

## 5. Run semantic completion

```sh
cargo test -p phalcom-lsp --test integration semantic_completion -- --nocapture
```

Coverage includes declared methods/getters, inheritance, overrides, class-side versus instance-side surfaces, `super`, field inference, chained receivers, and incomplete trailing-dot recovery.

## 6. Run inlay hints

```sh
cargo test -p phalcom-lsp --test integration inlay_hints -- --nocapture
```

The tests require stable inferred classes such as `Int` and `Person`, while protecting the rule that `Unknown` is not a fake source-level type.

## 7. Run workspace/live-edit tests

```sh
cargo test -p phalcom-lsp --test integration workspace_semantics -- --nocapture
```

These prove:
- `A.User` and `B.User` stay distinct;
- module aliases expose the right live surface;
- editing an open provider module invalidates an importing consumer without restarting the server.

## 8. Run semantic-token current-syntax coverage

```sh
cargo test -p phalcom-lsp --test integration semantic_tokens_current_syntax -- --nocapture
```

The test decodes LSP delta tokens to human-readable `(text, kind)` pairs. Prefer this over huge raw integer-array assertions.

Whenever syntax changes:
1. update `fixtures/highlighting/current_syntax.ph`;
2. update expected semantic classifications;
3. update the fixture-syntax smoke test in the same commit.

Comments remain a TextMate fallback concern while lexer trivia is unavailable to semantic tokens.

## 9. VS Code extension E2E

Build the server:

```sh
cargo build -p phalcom-lsp
```

Then:

```sh
cd tools/vsphalcom
npm test
```

or:

```sh
scripts/editor.sh vsphalcom
```

The included E2E test checks the actual path:

```text
VS Code API
  -> vsphalcom
  -> vscode-languageclient
  -> phalcom-lsp over stdio
  -> completion response
  -> VS Code CompletionList
```

## 10. Add VSIX creation

From `tools/vsphalcom`:

```sh
npm install --save-dev @vscode/vsce
```

Add to `scripts`:

```json
"test:lsp:e2e": "cargo build --manifest-path ../../Cargo.toml -p phalcom-lsp && npm test",
"vsix": "npm run package && vsce package"
```

Then:

```sh
npm run test:lsp:e2e
npm run vsix
```

For the current manifest version, install locally with:

```sh
code --install-extension ./vsphalcom-0.0.1.vsix --force
```

Adjust the filename when the version changes.

## 11. Suggested central test lanes

Recommended `scripts/test.sh` surface:

```text
lsp          cargo test -p phalcom-lsp
vsphalcom    build phalcom-lsp + npm extension-host tests
editor       lsp + vsphalcom
```

The provided `scripts/editor.sh` already supplies these without requiring an immediate edit to the central script.

## 12. Full pre-merge sequence

```sh
cargo fmt --check
cargo test -p phalcom-lsp
scripts/test.sh workspace
scripts/editor.sh vsphalcom
scripts/test.sh full

cd tools/vsphalcom
npm run vsix
```

## 13. Expected failures while implementation is incomplete

Do not weaken tests for these:
- no `textDocument/inlayHint` -> inlay test failure;
- no chained-return inference -> chained completion failure;
- `super` treated as ordinary `self` -> super negative assertion failure;
- simple-name class identity across modules -> `A.User`/`B.User` failure;
- stale caches after `didChange` -> live-edit failure;
- unknown receiver global builtin dump -> unknown-receiver regression.

## 14. Testing rules

Do not:
- execute user programs to obtain inferred semantic facts;
- assert native return shapes without source/semantic contracts;
- depend on generated `core-table.json`;
- use sleeps as the primary indexing synchronization mechanism;
- assert completion ordering unless ordering becomes language/editor contract;
- pin private SemanticDb struct layout in integration tests.
