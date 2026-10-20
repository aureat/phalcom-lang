---
id: PERF003.C2.P7.PATCH5-R.T0
kind: baseline-record
program: PERF003
checkpoint: PERF003.C2
plan: PERF003.C2.P7.PATCH5-R
status: COMPLETE
verification: FOCUSED_TESTED
branch: perf/universe-characterization
revision: 9cbba391a4482052dec476b327619c957a028b19
date: 2026-10-17
---

# PERF003.C2.P7.PATCH5-R baseline

## Result

R.G0 baseline capture is **PASS**. This records the live starting state and its
failures; it does not pass any implementation gate. R.G1-R.G7 remain
`UNVERIFIED`.

The branch was `perf/universe-characterization` at
`9cbba391a4482052dec476b327619c957a028b19`, equal to
`origin/perf/universe-characterization` after `git fetch origin`. Before the
checkpoint update, the only worktree item was the untracked PATCH5-R plan
supplied for this task. The cached and unstaged diffs were empty. The referenced
`AGENTS-old.md` did not exist in the worktree; the current `AGENTS.md` was used.

## First accounting discriminator

The strict whole-load test independently observed `528,785,777` allocator-
requested bytes. The production ledger reported `323,324,452`, leaving a
`205,461,325`-byte delta. The strict assertion failed before reaching the test's
`N-1` branch. The audit harness's separate `allocation-check` found
`N = 323,324,452`; it accepted any error at `N-1` and therefore does not prove
the required typed exhaustion or allocator equality.

R.T1-R.T3 own localization and closure of the allocation delta. No production
code changed during this baseline.

## Compiler and fixture baseline

- The four affected libraries pass `cargo check --lib` together.
- The type-meta allocation helper test passes 1/1.
- The modules topology and query targets pass 12/12.
- The semantic artifact allocation target passes 1/1.
- `cargo test --workspace --no-run` stops at
  `phalcom-core/examples/perf_workloads.rs:132` with E0601 because the example
  has no `main`; this is a pre-existing, unrelated workspace target failure.
- The source CLI rejects `packages/slotmap/src/main.ph:27` because its loop
  uses `for (entry in propositions)`.
- The parity harness uses `EntrySelection::Package` for `packages/slotmap`,
  which has `project.toml` rather than a root `package.ph`. Both modes report
  the same discovery error, and the harness exits successfully without
  compiling the project. This is not parity evidence.
- The runtime test project resolves its snake-case physical files
  `basic_test.ph` and `dense_test.ph` as non-canonical; the plan requires
  verifying the logical import mapping before renaming them.
- The artifact-mode runtime probe reaches VM bootstrap and reports the known
  `ProductKind` facade failure. Runtime bootstrap remains outside P7 and in P8.

## Source/artifact observations

One debug `print42` sample compiled successfully through each path. The source
sample reported 46,147.502 ms initialization and 37.128 ms user compilation;
the artifact sample reported 1,604.619 ms initialization and 72.188 ms user
compilation. These are single debug samples, not a performance comparison or
published speedup claim.

The generated artifact is 15,775,219 bytes. The generator's same-process
double-build comparison reports byte equality. `source-denial-check` verifies
an empty source list, complete snapshot, and successful synthetic user-package
compilation. It does not deny filesystem access. No cold/warm suite or
cross-process build-determinism claim was made.

## Evidence files

- `PERF003.C2.P7.PATCH5-R-command-log.md`: executed commands and outcomes.
- `PERF003.C2.P7.PATCH5-R-first-failure-ledger.md`: baseline failure owners,
  classifications, and follow-up tasks.
- `target/perf003-p7-audit/ARTIFACT.json`: generated artifact summary in the
  ignored build tree; it is not a checked-in baseline artifact.
