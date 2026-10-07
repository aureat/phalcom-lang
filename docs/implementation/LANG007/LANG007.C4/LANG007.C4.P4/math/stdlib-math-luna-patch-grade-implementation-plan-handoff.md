---
kind: implementation-handoff
plan: stdlib-math-luna-patch-grade-implementation-plan.md
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
branch: stdlib-random-math
date: 2026-10-07
---

# `universe.math` continuation handoff

## Current state

Continue on `stdlib-random-math`. T9 through T11 are implemented and focused
tested; T12 focused gates passed. The whole plan is still partial: T7's
per-function Float IEEE/accuracy matrix and T8's arbitrary-size integer
property/boundary matrix have not been completed. Do not start `universe.random`
or claim M1 completion from this continuation.

The implementation continuation was delivered in commit
`2c248f93c7047080908453a6d46039eb3a768ccc`.

T9's semantic owner is the canonical module-value boundary. Linked exports
resolve to canonical symbol/binding facts; member-body analysis receives the
same checked values and records precise module static-binding dependencies.
Do not introduce a math-specific coercion, a second Family call checker, or
consumer-side type reconstruction.

## Last verification

- Semantic T9 focused tests, including provider mutation and cold parity: PASS.
- Core exact Family/type rejection and hidden-child tests: PASS.
- Exported static constant selected/module read and re-export/GC tests: PASS.
- `language-corpus math`: PASS (one selected test).
- Core/modules/native-meta/native-surface `cargo check`: PASS.
- Native-surface generator `--check`: PASS (383 declarations; no generated
  changes).
- `git diff --check` and targeted `rustfmt --check` on changed Rust files:
  PASS.
- Workspace `cargo fmt --all -- --check`: FAIL due to existing formatting
  drift outside this continuation's Rust changes. No broad formatting rewrite
  was applied.
- `RUSTFLAGS='' cargo test -p phalcom-core`: FAIL, 160 passed / 3 failed in
  unrelated Option-match override, setter-shape, and stack-walk tests. They
  were not modified or repaired here.

## Next work

1. Implement and test T7's six-constant bit-pattern oracle, IEEE edge cases,
   per-function accuracy expectations, and purity checks, preserving the
   documented Rust `f64` accuracy boundary.
2. Implement T8's `gcd`/`lcm`/`isqrt` property, boundary, negative-input, and
   resource-limit coverage.
3. Re-run only the relevant focused tests as those matrices are added. At the
   eventual final gate, reclassify the three core failures and formatter drift
   only if their causes or the changed surface have changed.

The C4 checkpoint and P4/R1 acceptance state are not changed by this adjacent
math work. Random, secure random, and distributions remain unimplemented.
