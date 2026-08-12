# Repository baseline

Inspected: `aureat/phalcom-lang`

```text
5b6d67be93d6167558931a5c5dae3ae69959c9c4
Harden benchmark reporting and GC stress handling
```

Relevant facts on this baseline:
- `phalcom-lsp/Cargo.toml` has `autotests = false`;
- one explicit integration target: `tests/integration.rs`;
- Stage 3 already drives `Backend` through in-process JSON-RPC;
- `scripts/test.sh lsp` only runs the integration target;
- `tools/vsphalcom` already uses `vscode-languageclient` and `@vscode/test-electron`;
- its extension test is still the generated sample;
- semantic tokens are lexer-driven plus AST declaration refinement;
- lexer trivia currently keeps comments out of semantic tokens;
- superclass references store one raw identifier;
- imports use whole-module bindings.

Merge conceptually if the implementation branch has moved beyond this baseline.
