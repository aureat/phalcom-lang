# Phalcom VS Code

Phalcom language support through `phalcom-lsp`: diagnostics, semantic
completion, hover, definition/references, semantic tokens, and standard LSP
inlay hints.

## Language server

Server resolution order:

1. `phalcom.lsp.serverPath` when configured.
2. A bundled binary at `server/<platform>-<architecture>/phalcom-lsp`.
3. `phalcom-lsp` on `PATH` for development.

Commands:

- `Phalcom: Restart Language Server`
- `Phalcom: Show Language Server Output`

The client synchronizes the `phalcom` configuration section and watches
`**/*.ph`, so normal source and hint-policy changes reach the server without a
manual restart.
