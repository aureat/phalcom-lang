---
id: STDL001.C9
category: STDL
program: STDL001
checkpoint: STDL001.C9
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
active_plan: STDL001.C9.P1
---

# STDL001.C9 — Foundational math boundary and APIs

## Objective

Keep `universe.math` sufficient for ordinary Float and exact integer
mathematics, while moving advanced special functions to an optional
`numerics.special` package and placing representation-sensitive operations on
`Float`.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| `STDL001.C9.P1` | Separate special functions; consolidate Float and stable reduction APIs | COMPLETE | IMPLEMENTED | UNVERIFIED | Source edits and Rust/native surface checks are complete; Phalcom package behavior remains unverified |

## Established boundaries

- `universe.numbers` owns numeric values and capabilities; `universe.math`
  owns Float mathematical operations, stable reductions, constants, and exact
  integer algorithms.
- Advanced special functions are in the optional `numerics.special` package.
  Its kernel consumes public `universe.math` functions and numerical error
  types; `universe.math` does not depend on `numerics`.
- `Float.fma`, `Float.copySign`, and `Float.scalePow2` are Float-owned
  operations. Complex scalar-pair code delegates sign copying and binary
  scaling to those methods.
- `math.logAddExp`, `math.log1mexp`, and one-pass `math.logSumExp` are stable
  Float APIs. `math.mean` is the only public mean reduction; optional
  statistics delegates through its iterable input.
- Exact combinatorial functions and the six existing Float constants retain
  their owners and types.

## Verification and deferred work

- `phalcom-native-surface-gen --root .` regenerated 427 primitive records.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-modules -p phalcom-native-surface` passed.
- Phalcom source compilation and runtime behavior for the moved package,
  Float methods, and stable reductions were not run. The existing special
  function test clients were migrated to an explicit `numerics` dependency
  but were not executed.
- `log1pExp`, extended GCD, modular exponentiation, additional IEEE
  decomposition methods, and broad numerical certification remain deferred
  as lower-priority work in the proposal.

## Next action

Run focused Phalcom source acceptance for `numerics.special`, the Float methods,
and the stable reductions. Promote this checkpoint only after those results are
recorded; broad crate/workspace certification is not implied by the Rust check.
