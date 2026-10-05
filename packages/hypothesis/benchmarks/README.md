# Phase 11 benchmark workloads

The benchmark modules expose deterministic `run(iterations:)` or `seed(size:)` workloads for primitive generation, nested lists and span freezing, replay normalization, integer shrinking, and stateful shrinking.

Run them through the Phalcom benchmark runner when the toolchain provides one. Record wall time, allocations, choices consumed, candidates proposed, unique candidates replayed, and final example complexity. Phase 11 optimizations are accepted only when observable examples, spans, failure origins, reporter events, and database records are unchanged.

No benchmark timing is simulated by the Python source verifier.
