# Retained evidence

This folder follows the previous runtime audit: exact temporary probes and completed output are retained separately from implementation. Findings are not fixes. Final rerun instructions and probe counts are being populated.

- [Original prompt](original-audit-prompt.md)
- [Semantic baseline](semantic-baseline.txt): 1,138 passed, 42 ignored.
- [Semantic unit tests](semantic-unit-tests.txt): 82 passed.
- [Core typing integration](core-typing-integration.txt): 101 passed.
- [Initial invariant probes](initial-invariant-probes.txt): 1 passed, 8 intentionally exposed invariant failures.
- [Override runtime](override-runtime.txt): prints `wrong`.
- [Captured write runtime](captured-write-runtime.txt): prints `changed`.
- [Applied constructor runtime](applied-constructor-runtime.txt): None does not understand new().


## Final checkpoint and reproduction

- [Exact temporary test module](semantic_audit_probe.rs)
- [Final invariant output](final-invariant-probes.txt): 13 selected, 3 passed, 10 failed. Nine failures expose invariants; the tenth is the synthetic Object control error described in the executive report. The matrix logger is not a passing semantic property test.
- [Acceptance and captured-state output](acceptance-and-capture-state.txt): both acceptance probes pass; captured x remains Established Int after invocation.

The temporary module was copied to `phalcom-semantic/tests/semantic/foundations/audit_2026_09_09.rs` and registered with `mod audit_2026_09_09;` in that directory's mod.rs. Both temporary changes were removed after preserving these exact artifacts. To rerun, use an unused module name and preserve other contributors' changes. This probe uses the crate's existing `Fixture` helper; it is not a standalone rustc program. Correct the synthetic Object control before interpreting its extended matrix.

Completed commands, run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core typing_integration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic audit_2026_09_09 -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- docs/audit/type-system-2026-09-09/override-return.ph
target/debug/phalcom docs/audit/type-system-2026-09-09/captured-write.ph
target/debug/phalcom docs/audit/type-system-2026-09-09/applied-constructor.ph
```

The CLI build was checked by Cargo immediately before the runtime observations. The first two runtime programs completed and printed Strings; applied construction failed at None.new(). No implementation was fixed and no workspace gate was run. The final audit was stopped at the user's request to conserve the usage window.
