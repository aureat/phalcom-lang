# Suggested commit sequence

```sh
git add phalcom-lsp/tests/fixtures phalcom-lsp/tests/support
git commit -m "test(lsp): add semantic fixtures and JSON-RPC harness"

git add phalcom-lsp/tests/integration.rs \
        phalcom-lsp/tests/fixture_syntax.rs \
        phalcom-lsp/tests/semantic_completion.rs \
        phalcom-lsp/tests/semantic_consistency.rs \
        phalcom-lsp/tests/inlay_hints.rs \
        phalcom-lsp/tests/workspace_semantics.rs \
        phalcom-lsp/tests/semantic_tokens_current_syntax.rs
git commit -m "test(lsp): cover live semantic intelligence"

git add tools/vsphalcom/src/test \
        tools/vsphalcom/package.json \
        tools/vsphalcom/package-lock.json
git commit -m "test(vsphalcom): add language-server end-to-end coverage"

git add scripts/test.sh scripts/editor.sh
git commit -m "chore(test): add editor intelligence test lanes"
```

If acceptance tests are landed first on a feature branch, temporary red commits are fine; the merge/PR gate must be green.
