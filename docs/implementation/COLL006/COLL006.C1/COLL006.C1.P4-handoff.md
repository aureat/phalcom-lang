---
id: COLL006.C1.P4
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P4 — Continuation after T10/G6

P4 remains open. T10 and G6 are complete; continue with T11, then G7 and T12.
The active worktree is `telos-collections-upgrade` on branch
`feat/telos-collection-upgrade`. Start from its current HEAD, which contains
the T10/G6 implementation and records.

## Stable T10 behavior

- A bare runtime bracket literal defaults to `Array<T>`.
- A bare static/`const` bracket literal defaults to exact-extent `Vector<T,N>`.
- Explicit heads and unique contextual family selection retain precedence over
  the phase default. Explicit List use remains supported.
- A static empty literal has known extent zero but does not invent element type
  evidence; underconstrained `T` is diagnosed and no value is published.
- The checker sets `EvaluationPhase::Static` while checking a `const`
  initializer, then restores the caller phase. This keeps ordinary module
  binding facts aligned with the canonical static-constant product. Runtime
  `let`/`let mut` inherits the enclosing phase.
- Headless and explicit Vector literals fail closed in the context-free
  compiler without semantic lowering. Syntax-explicit `<T>[...]` Array remains
  the supported compatibility path.
- Current Phalcom specs do not permit selector overloads distinguished by
  parameter/result types. CV-25 is covered by ambiguous trait dispatch; the
  Array phase default does not choose a candidate. Do not add an overload
  system as part of this cutover.

## G6 evidence

All required G6 commands passed:

| Command/filter | Result |
|---|---|
| `cargo test -p phalcom-semantic --test semantic coll006` | 49 passed |
| `cargo test -p phalcom-core --test core core_array` | 11 passed |
| `cargo test -p phalcom-core --test core core_vector` | 8 passed |
| `cargo test -p phalcom-core --test core static_const` | 23 passed |

The supplementary semantic `static_constants` filter had 21 passes and the
same three D4 baseline failures: `test_callable_signature_projects_all_parameter_and_return_record_keys`,
`test_field_signature_projects_a_cold_computed_record_key`, and
`test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly`.
Each reports `RecordKeyNotStatic` for computed Record keys. The bare static
Vector, empty underconstraint, static List, and arbitrary-constructor boundary
cases in that filter passed. Keep G2 `BASELINE_BLOCKED`; do not repair D4 under
P4.

## T11 work remaining

No T11 incremental matrix has been retained yet. An exploratory T11 test found
the `const` phase-owner gap described above; the correction is now part of T10
and has its own G6 regression. Continue with the plan's remaining cases:

1. Incremental static contents and exact extent changes; removing an Array
   annotation; `const` to `let` and `let` to `const`; Vector spread source
   losing exactness; invalid-to-valid and valid-to-invalid transitions.
   Assert cold/incremental parity, no stale current value/descriptor, changed
   fingerprints where inputs/results change, and unaffected product reuse
   where the live invalidation boundary promises it.
2. Selective, renamed, qualified, re-exported, and two-consumer module reads;
   verify canonical source owner identity.
3. GC before reads, after materialization, during nested construction where
   hooks permit, and with only the owner module cache rooting the value.
4. Reconfirm runtime alias mutation fails for frozen const Array while ordinary
   Array mutation remains available.
5. Retain source-order/exactly-once checks for bare Array and explicit Vector.
6. Check zero and supported concrete extents, host-count conversion, and
   controlled allocation failure only where existing hooks support them; add
   no artificial global limit.
7. Reconfirm static List fails closed and bracket support does not authorize
   arbitrary runtime constructors.

Existing relevant tests include semantic incremental publication parity,
`static_const` module import/re-export/GC and alias coverage, `core_array`,
`core_vector`, and the static-constant boundary tests. Inspect them before
adding coverage and avoid duplicating established cases. T11's focused command
set is in the P4 plan's T11 section.

## G7 and T12

G7 requires the plan's focused semantic, static-sequence, static-constant,
incremental, Array, Vector, and module/static-constant gates, plus
`cargo check --workspace --all-targets`, applicable native-surface checking,
and `git diff --check`. Do not run workspace tests, clippy, or release
certification; they are outside this plan's gate.

The G5 census records 495 sites (331 `.ph`, 164 Rust source/test strings),
with no ambiguous or unclassified sites. T10 added two regression-fixture
bracket literals after that snapshot; refresh and classify the census in T12.
Then update the checkpoint and program status as required, create the P4
walkthrough and P5 handoff, and mark P4 complete only if G7 passes and no
P4-owned A/B failures remain unresolved. Keep completion and verification
classifications separate.

## Records to read

- [P4 implementation plan](COLL006.C1.P4-static-homogeneous-sequences-and-atomic-bracket-cutover-luna-patch-grade-implementation-plan.md)
- [C1 checkpoint](COLL006.C1-CHECKPOINT.md)
- [G5 migration census](COLL006.C1.P4-bracket-migration-census.md)
