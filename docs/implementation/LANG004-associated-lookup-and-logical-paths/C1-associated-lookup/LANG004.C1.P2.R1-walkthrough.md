---
id: LANG004.C1.P2.R1
category: LANG
program: LANG004
checkpoint: LANG004.C1
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
based_on_revision: 2ea4f3a1
---

# LANG004.C1.P2.R1 — Setter Value Lane Invocation ABI Remediation Walkthrough

## Outcome

Completed the runtime ABI correction in the supplied R1 plan. Setter values remain a
dedicated trailing invocation lane and are no longer accepted as positional arguments
when activating a captured Setter. Invocation layouts are checked against selector
structural arity and against the exact receiver-plus-arguments stack window before
dispatch. `ArgumentView` lane accessors safely return `None` if a malformed view points
outside the stack.

The patch did not alter selector identity, setter syntax, physical argument order, or
assignment result behavior. It did not touch semantic/AST sources.

## Changed files

- `phalcom-core/src/method/object.rs` — validates structural selector arity and stack
  window boundaries; makes lane reads bounds-safe.
- `phalcom-core/src/method/mod.rs` — extends invocation-layout invariant coverage.
- `phalcom-core/src/vm/send.rs` — validates shared dispatch windows and selector kinds,
  removes setter-as-positional captured-method fallback, and adds exact/pattern Family,
  BoundMethodFamily, and captured-setter regression tests.
- `docs/implementation/LANG004-associated-lookup-and-logical-paths/C1-associated-lookup/CHECKPOINT.md`
  and `selector-shapes-and-family-activation-implementation-state.md` — record R1 state
  and verification boundary.

## Verification

Focused PASS:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core`
- invocation layout unit test: 1 passed
- captured setter invariant unit test: 1 passed
- exact and pattern `Family.set(Tuple, value)` lane regression: passed
- BoundMethodFamily labeled setter lane regression: passed
- shape-aware static and dynamic-pack setter integration regression: 1 passed
- scoped rustfmt check on all changed Rust files
- `git diff --check`

Baseline-blocked:

- `execution_family_runtime`: 11 passed, 9 failed while compiling fixtures due to
  `EnumFamilyCategoryConflict` on `Tuple.[]` in the canonical Universe source. This
  failure was present before R1 edits.
- `cargo fmt --all -- --check`: reports formatting drift in untouched repository files;
  the changed Rust files pass scoped formatting.
- `cargo check --workspace --all-targets`: fails in unrelated semantic test code due to
  one stale `DeclarationTypeInfo` initializer and old `InferenceStaticArgument` test
  literals. The core crate check and focused core tests pass.

## Remaining boundary

Do not classify the full core family runtime suite or workspace check as green. Re-run
them after the unrelated Universe and semantic-test baselines are repaired. No R1 source
work remains identified by the negative searches; remaining `saturating_sub(1)` hits
are instruction-pointer location calculations, and the setter arity addition is used
only to report expected argument count.
