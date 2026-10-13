---
id: PERF003.C2.P2
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# PERF003.C2.P2 implementation walkthrough

Implemented on `perf/universe-characterization` in `.worktrees/perf-universe-characterization`,
over `820514801e0de0480ab8bd03cf676c1e0c137b5a`; source changes remain uncommitted.

The semantic crate now builds and caches a completed formal Universe publication,
then freezes a closed set of committed typed query roots. The module crate retains
provider-derived parsed sources/interfaces. Core compilation acquires this snapshot
and retains native-contract and executable match-coverage checks. The cache retries
failures and rejects same-thread recursion; custom providers retain distinct worlds.
Exact source provenance and complete type/static prefix checks protect typed reads.
All rejected query rows have explicit classifications; failed/blocked text is retained.

After the user's continuation request, an opt-in canonical `SemanticDb` parent was
added. Products/fingerprints/origin/currentness can read the immutable parent;
canonical keys never become synthetic local Ready rows. Local forward dependencies
can observe immutable fingerprints without installing reverse edges for those roots.
Publication rejects canonical identities; discard/invalidation/retirement and
transaction rollback preserve the parent. Ordinary semantic-session construction
has not yet been switched to this API.

A separate P3 increment replaces global explicit-witness signature searches with
exact lookup through the existing CallableId hash index. Effective inherent-surface
assembly enumerates the maintained module index rather than the global table.
Compatibility, body validation, evidence and requirement rules are unchanged.

## Verification evidence

- Concurrent image first-access integration: 1 passed; eight simultaneous callers,
  one successful image build/update. Ten warm readers: 32.958 microseconds in the
  recorded unoptimized sample. Cold initialization: 70.812812667 seconds.
- Image exporter/provenance unit tests: 8 passed before the database-parent addition.
- Canonical database parent/rollback regression: 1 passed (69.43 seconds cold).
- Exact conformance witness lookup/edit regression: 1 passed.
- Semantic test target compilation and core perf-observability compilation passed
  during implementation; final check result is recorded in the checkpoint.
- Scoped formatting and diff whitespace checks performed.

Census: 148 sources; 9,318 examined query rows, 2,763 retained, 6,555 rejected;
2,432 final types. Four ordinary internal incidents remain visible in the snapshot.
No incident-free semantic certification is claimed.

## Deferred work

At the user's request, broad suites, source/provider differential execution,
core/runtime parity, release certification and the G6 performance matrix are deferred.
P2 is partial/unverified at full-plan scope despite the listed focused passes.
Full external module ownership, ordinary-session image construction, one-shot source
seeding removal, return-sensitive/SCC settlement and metadata closure are pending.
