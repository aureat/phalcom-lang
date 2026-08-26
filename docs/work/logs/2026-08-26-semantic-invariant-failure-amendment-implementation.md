# Semantic Correctness Part 1: Invariant Failure Amendment

- Date: 2026-08-26
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Work type: Semantic analyzer implementation
- Status: Implemented; changes remain uncommitted
- Spec: [phalcom-semantic-correctness-part1-invariant-failure-amendment.md](/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/semantic-analyzer-failure-invariants/phalcom-semantic-correctness-part1-invariant-failure-amendment.md)

## Outcome

Implemented structured internal semantic invariant failures for compiler-owned formal flow analysis. Impossible analyzer states remain distinct from user type errors, honest incompleteness, suppression, cancellation, and budget exhaustion.

## Implementation

- Added `FlowInvariantFailure` for divergent persistent binding contracts and mutability metadata.
- Made formal flow joins and loop widening fallible. Normal current-value disagreement remains a valid flow join.
- Added poisoned/fail-closed `FlowState` handling after an invariant failure.
- Added structured `InternalSemanticIncident` records with module, callable, expression, range, category, and semantic details.
- Added `InternalFailurePolicy::{Contain, FailFast}`. Fail-fast records the incident before panicking.
- Migrated expression and `for` flow-join callers to record incidents and stop unsafe transfer.
- Added callable-level `InternalFailure` publication and stopped body analysis after terminal internal failure.
- Aggregated incidents separately on `CallableAnalysis` and `SemanticSnapshot`; they are not ordinary diagnostics.
- Published contained internal-failure callable products so unrelated semantic products remain available.
- Added semantic-shape incident fingerprinting that ignores local incident allocator IDs.
- Added default semantic fixture/workspace assertions rejecting unexpected internal incidents, with an explicit opt-in constructor for incident tests.

## Focused evidence

```text
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic
380 passed, 0 failed, 10 ignored

cargo test -p phalcom-semantic --lib checker::context::tests
3 passed, 0 failed

flow_graph focused tests
12 passed, 0 failed

callable_internal_failure_fingerprint_ignores_local_incident_id
1 passed, 0 failed

cargo check -p phalcom-semantic
cargo check -p phalcom-lsp
passed
```

Targeted `rustfmt --check` and `git diff --check` passed. `graphify update .` completed after the implementation.

## Known validation boundaries

- Workspace-wide `cargo fmt --all -- --check` still reports unrelated existing formatting differences in semantic capability/support tests.
- Strict clippy remains blocked by existing baseline lint debt, including generated `phalcom-native-surface` warnings and existing semantic-crate warnings. No generated files were changed.
- This log records the invariant-failure amendment slice; it does not claim completion of broader Part 1/2/3 release gates.

## Worktree boundary

Existing documentation moves, deletions, and new documentation trees were preserved. No plan files were reverted or reorganized by this implementation.
