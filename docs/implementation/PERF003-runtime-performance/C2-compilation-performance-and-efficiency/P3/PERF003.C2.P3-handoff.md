# PERF003.C2.P3 handoff — conformance indexing and settlement increments

Continue on `perf/universe-characterization` in `.worktrees/perf-universe-characterization`.
Code committed at `c34f9c3f`.
`CHECKPOINT.md` owns durable status and next action.

## Established in this increment

1. **Partitioned Conformance Index:**
   - `ConformanceIndex` partitions contributions by trait, nominal family, and open heads.
   - `insert_with_store` populates family classifications.
   - Exact parity with brute-force query and overlap checks is proven by `partitions_queries_and_coherence_match_brute_force_through_mutations`.

2. **Inferred-Result Worklist & Edge Tracking:**
   - Affected-result and SCC worklist propagation replaces global rescans in `session.rs`.
   - `callable_analysis_dependencies` re-records observed query dependencies on recomputed bodies.
   - Proven by `inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once`.

3. **Fine-Grained Module-Value Provenance:**
   - Lexical lookup provenance avoids whole-environment invalidations for unused/shadowed module imports.
   - Negative export dependencies revalidate correctly when exports are added.

4. **Witness Plan Separation Foundation:**
   - Pre-instantiated `requirement_views` are passed directly to `build_conformance_witness_plan`.
   - Universe conformances are skipped based on `source_module.project.is_universe()`, allowing local augmentations.

## Invariants and Constraints for Successor Work

- **Do NOT add competing witness caches:** Canonical `ConformanceWitnessPlan` in `phalcom-semantic` remains the sole authority.
- **Retained Structural Plans:** When persisting structural witness plans across revisions, do not conflate structural selection with return-sensitive completion. Implement separate structural vs result-sensitive fingerprints and explicit positive/negative eligibility query edges.
- **Generic Proof Memoization:** Generic conformance query memoization remains pending.
- **Verification Policy:** Maintain BUILD MODE. Defer broad workspace certification, full parity suites, and performance benchmarking until explicitly requested.
