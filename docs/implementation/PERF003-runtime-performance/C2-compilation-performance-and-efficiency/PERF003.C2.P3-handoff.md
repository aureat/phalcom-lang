# PERF003.C2.P3 handoff — first increment

Continue on `perf/universe-characterization` in the existing worktree. Changes are
uncommitted. `CHECKPOINT.md` owns durable status and next action.

`CallableSignatureTable::get_conformance_witness` is exact indexed lookup;
`iter_module` uses the existing maintained module index. Do not add another witness
candidate cache for the same canonical identity. Requirement matching/compatibility
remains in `build_conformance_witness_plan` after lookup.

Remaining work includes dependency-complete instantiated requirement reuse,
structural/return-sensitive witness settlement, inferred-return SCC propagation,
and generic proof memoization. Preserve negative dependencies and existing formal
proof strength. Broader verification is explicitly deferred by the user.
