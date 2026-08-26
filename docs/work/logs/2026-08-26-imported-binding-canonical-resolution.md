# Imported-Binding Canonical Definition Resolution

- Date: 2026-08-26
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Status: Fix implemented locally and verified; no commit or merge performed
- Focused area: `tests/imported-binding-resolution`

## Result

The diagnosis was directionally correct but incomplete. Imported-name navigation lost canonical declaration identity after later occurrences resolved through local import bindings. Full workspace source-index construction also lacked direct canonical targets for selective imports because its import keying did not match source-index expectations. Go-to-definition consequently returned the import alias declaration alongside the real declaration, often listing the local file first.

## Confirmed root causes

- Later imported-name occurrences replaced canonical declaration identity with the local import `Binding`.
- Full workspace source-index construction lacked direct canonical targets for selective imports and keyed imports differently from source-index lookup.
- Definition navigation returned both the import alias declaration and the actual canonical declaration.

## Implemented fixes

- Publish linked import targets directly into [`SourceIndexContext`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/source_index/builder.rs:14), preserving re-export-safe declaration identity.
- Preserve canonical targets when [`OccurrenceBuilder`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/source_index/occurrence.rs:464) resolves through a lexical import binding.
- Populate linked import targets from module layout in [`session.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs:1025).
- Make definition navigation select the actual canonical declaration and exclude the import-alias declaration in [`backend.rs`](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/backend.rs:1103).
- Update stale module-navigation expectations and register a dedicated imported-binding LSP regression target.

## Branch-quality findings

- Semantic regression initially failed to compile because `collect()` was ambiguous.
- LSP regression file was not registered, so Cargo did not execute it until registration was corrected.
- LSP waiter accepted the first non-empty legacy response instead of waiting for the canonical target.
- Cross-module formal type inference was already correct: imported `shapes.point.Point` retained declaring-module class-object identity.

## Verification

- Semantic suite: **386 passed, 10 ignored, 0 failed**
- LSP suite: **315 passed, 2 ignored, 0 failed**
- Dedicated imported-binding LSP regression: **passed**
- Module-navigation suite: **3 passed**
- `git diff --check`: **passed**

## Boundary

Testing stopped as requested. No commit or merge was performed. Existing unrelated workspace changes remain untouched.
