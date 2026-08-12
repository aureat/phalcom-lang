# CI integration

The existing Rust CI already runs:

```sh
cargo build --workspace --all-targets
cargo test --workspace --all-targets
```

That should exercise the explicit `phalcom-lsp` integration target after the new modules are wired into `tests/integration.rs`.

Keep VS Code extension-host testing in a separate job. Electron/browser-host failures have different dependencies and diagnostics from Rust failures.

Suggested GitHub Actions job:

```yaml
vsphalcom:
  name: VS Code extension E2E
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - uses: dtolnay/rust-toolchain@stable

    - uses: Swatinem/rust-cache@v2

    - uses: actions/setup-node@v4
      with:
        node-version: 20
        cache: npm
        cache-dependency-path: tools/vsphalcom/package-lock.json

    - name: Build language server
      run: cargo build -p phalcom-lsp

    - name: Install extension dependencies
      run: npm ci --prefix tools/vsphalcom

    - name: Install virtual display
      run: |
        sudo apt-get update
        sudo apt-get install -y xvfb

    - name: Run VS Code extension-host tests
      run: xvfb-run -a npm --prefix tools/vsphalcom test
```

Do not make VSIX publishing part of pull-request CI. Packaging smoke may be run without publishing:

```sh
npm --prefix tools/vsphalcom run vsix
```

Publishing needs a separate release policy, credentials, and versioning decision.
