# PERF003 Universe Startup Characterization Handoff

**Disposition:** `BASELINE_BLOCKED`
**Required baseline:** `b00d75b3ac9f50d37cf9abfddfde00917e099031`
**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
**Branch:** `perf/universe-characterization`

## Decision for the next owner

Keep the characterization as blocked and do not infer an artifact payoff from it. The committed baseline has two hard failures before a successful Telos execution:

- A minimal fixture fails metadata export: `legacy metadata cannot represent a Record remainder outside the Symbol key domain`.
- Full VM construction panics in canonical Universe source indexing: `intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration`.

Do not repair or route around these issues in this investigation. The feature-gated harness compiles; its full VM modes cannot produce measurements until the canonical source-index path and valid fixture compilation are working on a benchmarkable committed revision.

## What is established

- The test-timing harness emits pre-failure semantic measurements. Across the failed first fixture in each of two corpus processes, semantic update summed to 2.866 s and session initialization to 0.414 s; there are no warm samples and all post-projection phases are unobserved.
- `VM::new_kernel()` and `VM::new_native()` completed once at 13.871 ms and 76.058 ms. `VM::new()` failed before any full-VM sample or counter snapshot.
- Python startup controls completed: five standard Python samples (median 26.592 ms) and five `-S` samples (median 18.999 ms).
- Instrumentation/example compilation checks pass with and without `test-timings`. Focused corpus execution did not pass because it hit the baseline metadata failure; runtime parity of the harness has not been certified.

## Rerun gate

After relevant type-system/metadata fixes are committed, record the new full SHA and rerun the same matrix in a clean worktree at that SHA. At minimum, first prove that the tiny Telos fixture compiles and a full `VM::new()` completes. Then rerun a corpus lane with multiple fixtures, VM tiers, fresh Telos processes, and the warm retained-session harness. Keep the old `b00d75b3...` evidence separate for comparison.

The active dirty `main` checkout was deliberately not read, copied from, or modified. The user expects its unfinished type-system corrections may materially alter semantic or metadata costs; characterize those only after they land cleanly.

## Files

- Main report: `PERF003-universe-startup-performance-characterization.md`
- Corpus timing CSV and partial summary: `PERF003-universe-startup-measurements.csv`, `corpus-partial-summary.md`
- Python and failed Telos process samples: `process-startup-measurements.csv`
- Commands and raw outputs: `raw/commands-and-results.md`, `raw/*.txt`
- Feature-gated harness: `../../../../../phalcom-core/examples/perf_universe_characterization.rs`
