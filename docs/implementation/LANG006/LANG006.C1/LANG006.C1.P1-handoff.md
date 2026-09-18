---
id: LANG006.C1.P1
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: bf4e12ba
working_tree: uncommitted scoped implementation changes
---

# LANG006.C1.P1 — Handoff

## Current state

The plan is complete as an implementation vertical in the shared working tree.
No commit or push was requested or performed. Preserve all existing changes and
do not reset or clean the checkout.

## Stable takeover map

- Shared semantic categories: `phalcom-common/src/modifier.rs`.
- AST normalization and retired namespace rejection: `phalcom-ast/src/{token,lexer,parser,ast}.rs`.
- Semantic source/native merge: `phalcom-semantic/src/core_surface/` and
  declaration/source-index products.
- Compiler/native source authority: `phalcom-core/src/compiler/`,
  `phalcom-core/src/native/`, and `phalcom-core/src/native/install.rs`.
- Runtime authorization: `phalcom-core/src/vm/send.rs`; native methods receive
  `RuntimeAccessOrigin::universe()` where the declaration is a Universe native
  descriptor.
- Canonical source: `phalcom-core/core/universe/src/**/*.ph`; no legacy
  modifier attributes or authored `_$` selectors remain. `__name__` entries are
  intentional standardized dunders.
- Editor parity: `phalcom-lsp/src/semantic_tokens.rs`, `completion.rs`,
  `hover.rs`, and `backend.rs`.
- Current authority: `docs/specs/source-declarations.md` and
  `docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md`.

## Invariants not to redesign

- Do not reintroduce prefix-derived visibility or implementation origin.
- Do not use `is_universe` as ordinary `Internal` authorization.
- Do not turn native descriptors into blanket privileged callers.
- Keep `foreign` a bounded bodyless scaffold; do not invent an FFI loader/ABI.
- Keep `@primitive`/`@intrinsic` metadata-only and Universe-only.
- Preserve standardized dunder protocols exactly; distinguish them from retired
  general implementation fields.
- Preserve the single semantic access relation across direct, cached, dynamic,
  reflective, and native calls.

## Known residuals

- One semantic incremental identity assertion and one core VM walk test are
  baseline failures documented in the checkpoint and walkthrough.
- Full package/workspace test certification is deferred.
- Repository-wide format check is blocked by existing drift; use narrow
  formatting only if a later authorized work unit needs it.

## Recommended first checks

```sh
git status --short
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_universe::boot_01_bootstrap_measurement_separates_catalog_closure_and_execution -- --exact
```

Do not re-explore or redesign C1. If broader certification is desired, run the
package-level suites one at a time and classify the two known baseline failures
before changing any source.

## Next action

Proceed to checkpoint-level certification when broader verification is desired;
otherwise continue with the next explicitly scoped LANG006 checkpoint.
