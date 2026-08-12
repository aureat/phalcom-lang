# Review notes before copying

## Files that are safe additions

These are new paths on the inspected baseline:

```text
phalcom-lsp/tests/fixtures/**
phalcom-lsp/tests/support/**
phalcom-lsp/tests/fixture_syntax.rs
phalcom-lsp/tests/semantic_completion.rs
phalcom-lsp/tests/semantic_consistency.rs
phalcom-lsp/tests/inlay_hints.rs
phalcom-lsp/tests/workspace_semantics.rs
phalcom-lsp/tests/semantic_tokens_current_syntax.rs
tools/vsphalcom/src/test/suite/lsp.e2e.test.ts
tools/vsphalcom/src/test/fixtures/**
scripts/editor.sh
```

If the implementation agent created any of them independently, merge rather than overwrite.

## Existing files that should be edited, not blindly replaced

```text
phalcom-lsp/tests/integration.rs
scripts/test.sh
tools/vsphalcom/package.json
tools/vsphalcom/package-lock.json
```

Use the patch/snippet files for these.

## Why package-lock.json is not supplied

Run:

```sh
cd tools/vsphalcom
npm install --save-dev @vscode/vsce
```

and let the project's installed npm version update the lockfile consistently.

## Why there is no cross-module inheritance fixture

The inspected AST models `SuperclassRef` as one raw identifier. Whole-module
imports bind a Module object (`import "./a" as A`) whose members are accessed
through ordinary sends (`A.User`). The kit therefore tests:

- inheritance inside one module;
- imported module/member completion;
- distinct module-qualified class identities.

Do not add `class Child is A.Parent` as an editor contract unless the Phalcom
grammar itself gains that surface.

## Expected adaptation points

The semantic implementation may choose slightly different inlay label
punctuation. Keep semantic assertions (`contains("Person")`) unless exact
punctuation becomes a product contract.

If the implementation changes completion output from an array to an LSP
`CompletionList`, the provided helper already accepts both.

If configuration names changed from the earlier design spec, add configuration
tests using the landed names rather than preserving obsolete draft names.
