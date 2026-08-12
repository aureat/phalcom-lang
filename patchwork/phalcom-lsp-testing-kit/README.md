# Phalcom LSP Live-Semantics Testing Kit

Baseline inspected: `5b6d67be93d6167558931a5c5dae3ae69959c9c4` (`main`, 2026-08-11).

This package is designed to be copied into `aureat/phalcom-lang` after the live semantic-intelligence implementation lands.

It contains:
- copy-ready Phalcom fixtures under `repo-files/phalcom-lsp/tests/fixtures/`;
- reusable in-process LSP JSON-RPC test infrastructure;
- Rust integration tests for completion, inlay hints, workspace semantics, semantic-token coverage, live edits, and consistency;
- a fixture-syntax regression test;
- a VS Code extension E2E test;
- a new `scripts/editor.sh` convenience command;
- patch snippets/instructions for `integration.rs`, `scripts/test.sh`, `tools/vsphalcom/package.json`, and VSIX packaging;
- a detailed test matrix and merge gates.

Markers such as `/*@completion*/` are removed by the fixture loader before source is sent to the server.

Start with `APPLY_AND_RUN.md`.
