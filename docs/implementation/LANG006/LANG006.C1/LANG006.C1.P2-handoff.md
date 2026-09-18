---
id: LANG006.C1.P2
kind: implementation-plan-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
base_revision: 9f1d2c9fa98e94edc6088aee1834520361e08d3e
working_tree: uncommitted scoped implementation changes
---

# LANG006.C1.P2 — Handoff

## Current state

The P2 corrective vertical is implemented and focused-tested in the shared
worktree. Preserve all existing modifications; do not reset, clean, or blanket
format. No commit or push was requested in this turn.

## Stable interfaces

- `phalcom-semantic/src/access.rs` owns the shared declaration/caller access
  relation and canonical module/package/project provenance.
- Semantic checker dispatch authorizes the selected target after lookup and
  retains inaccessible-versus-missing distinction.
- Source compiler method constructors stamp module-derived runtime origin.
- `NativeMemberAnchor.module` is the source authority for native runtime origin;
  `refresh_registered_primitive_origins` reconciles installed methods after
  module materialization.
- `CallableSemanticSignature::native_contract_mismatch_with_store` is the
  canonical source/native contract comparison. Do not add a core-side type
  normalizer.
- `phalcom-common::modifier::ImplementationKind` is the language category;
  `phalcom_native_meta::ImplementationProvenance` is execution provenance.
- `Foreign` is retained semantically and fails with
  `foreign.binding_unimplemented`; no FFI ABI or loader exists.

## Invariants to preserve

Keep exact Module/Package/Project identity semantics, lookup-then-authorize
ordering, non-fallthrough on inaccessible members, source-derived native
authority, independent Universe provenance, and the single language
implementation-kind axis. Do not reintroduce selector prefixes, path-based
package inference, blanket native Universe authority, or a Foreign stub/ABI.

## Verification already obtained

Passing focused evidence includes semantic access control, semantic native
conformance and presentation, canonical signature mismatch unit coverage,
core native units, three reflection-conformance tests, explicit Foreign
diagnostic behavior, AST modifier regressions, LSP hover-filtered coverage,
native metadata/surface/decl tests, semantic/core compile checks, and the exact
Universe bootstrap measurement test. The full semantic suite ended with 1,420
passed, 22 failed, and 50 ignored; the full core suite ended with 537 passed,
74 failed, and 28 ignored. `git diff --check` was green after the
latest documentation-only edits.

## Remaining work

Run and classify the plan's remaining direct evidence before closing C1:

- dedicated runtime source access parity for cold/warm cache, dynamic,
  bound/reference, reflection, and transient compiler authority;
- standalone module/package/nested-package/cross-project ownership fixtures;
- any missing Foreign semantic-retention filter and LSP package-completion
  filter, adapting names to live tests;
- G5 affected-package suites and final `git diff --check`.

The compiler declaration-dispatch failures for iterator syntax, retired private
syntax, and bootstrap `Class.allocate()` access reproduce on the pushed P1
revision and are deferred baseline evidence. The semantic incremental identity
failure, VM walk/fold failure, and repository-wide formatting drift remain
carried baselines.

## Recommended next commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_declaration_dispatch -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
git diff --check
```

Do not claim checkpoint completion or release certification from the current
focused evidence alone. Broad verification is currently baseline-blocked;
update the shared checkpoint and these records only after the remaining P2
coverage is genuinely run and classified.
