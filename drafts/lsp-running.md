Use repo wrappers from `/Users/altunhasanli/dev/phalcom/phalcom`:

```sh
npm ci --prefix tools/vsphalcom

# Rust LSP unit + integration tests
scripts/editor.sh lsp

# Build phalcom-lsp, then run VS Code extension-host E2E tests
scripts/editor.sh vsphalcom

# Run both
scripts/editor.sh all

# Build installable VSIX
scripts/editor.sh vsix
```

Equivalent package commands:

```sh
npm --prefix tools/vsphalcom run compile   # development webpack build
npm --prefix tools/vsphalcom test          # compile tests, compile extension, lint, run VS Code tests
npm --prefix tools/vsphalcom run vsix      # production bundle + VSIX
```

VSIX appears at:

```text
tools/vsphalcom/vsphalcom-0.0.1.vsix
```

Install and smoke-test:

```sh
code --install-extension tools/vsphalcom/vsphalcom-0.0.1.vsix --force
```

Then set:

```json
{
  "phalcom.lsp.enabled": true,
  "phalcom.lsp.serverPath": "/Users/altunhasanli/dev/phalcom/phalcom/target/debug/phalcom-lsp"
}
```

Build that server first:

```sh
cargo build -p phalcom-lsp
```

Important: current VSIX scripts do not copy `phalcom-lsp` into `server/<platform>-<architecture>/`; installed VSIX therefore needs `phalcom.lsp.serverPath` or `phalcom-lsp` on `PATH`. E2E tests configure the built binary automatically. See [package scripts](/Users/altunhasanli/dev/phalcom/phalcom/tools/vsphalcom/package.json:127), [editor lanes](/Users/altunhasanli/dev/phalcom/phalcom/scripts/editor.sh:7), and [manual checklist](/Users/altunhasanli/dev/phalcom/phalcom/tools/vsphalcom/manual-test/CHECKLIST.md:8).

For manual Extension Development Host testing: open `tools/vsphalcom`, run `npm run compile`, then press F5. Current shell lacks the `code` CLI; use VS Code’s “Extensions: Install from VSIX…” if needed.