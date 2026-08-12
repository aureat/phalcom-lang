# `tools/vsphalcom/package.json`

Install the VS Code packager:

```sh
cd tools/vsphalcom
npm install --save-dev @vscode/vsce
```

Let npm update both `package.json` and `package-lock.json`.

Add to `scripts`:

```json
"test:lsp:e2e": "cargo build --manifest-path ../../Cargo.toml -p phalcom-lsp && npm test",
"vsix": "npm run package && vsce package"
```

Keep the existing:

```json
"vscode:prepublish": "npm run package"
```

Use:

```sh
npm run test:lsp:e2e
npm run vsix
```

For the current `0.0.1` manifest:

```sh
code --install-extension ./vsphalcom-0.0.1.vsix --force
```
