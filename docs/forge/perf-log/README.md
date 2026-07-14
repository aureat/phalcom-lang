# Performance log

Running, measured record of optimization cuts and profiling findings for the
Phalcom VM. One entry per landed cut; findings that reshape the plan live in
[`findings.md`](findings.md).

Governed by [`performance.md`](../../spec/v0.2/performance.md) +
[ADR-0051](../../adr/0051-performance-strategy-measure-first-tiered-optimization.md)
(measure-first, tiered, behavior-invariant). Every entry cites a **before/after
number from the U-BENCH harness** (`benchmarks/vm/`, criterion target
`phalcom-core/benches/vm_bench.rs`) — no oral numbers (law P1).

## Cuts

| # | Unit / tier | Cut | Measured | Golden diff |
|---|-------------|-----|----------|-------------|
| [001](001-prim-abi-inline-args.md) | U-PRIM-ABI / Tier 2 | On-stack arg buffer replaces per-send `Vec` in the primitive path | arith_send −41.5%, bare_send −33.8% | none |

## Findings

See [`findings.md`](findings.md) — measured baseline (Skynet ~19–20× Wren, not
the oral 29×), the malloc-is-#1 re-rank, the falsified `Option`-escape premise,
the memmove mechanism correction, and U-IC preconditions.
