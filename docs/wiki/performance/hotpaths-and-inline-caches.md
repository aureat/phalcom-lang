# Hot paths and inline caches

> Sources: performance specification; PERF001–PERF002
> Raw: [performance specification and program snapshot](../raw/performance/2026-09-08-performance-sources.md)
> Updated: 2026-09-08

The optimization model treats message sends and per-send allocation as distinct cost classes. A specialized path is valid only when a guard proves equivalence with the generic send and deoptimization reconstructs interpreter state.

## Boundaries

Inline-cache population is an optimization seam keyed by class/selector identity; it cannot alter dispatch semantics or reopen locked representation decisions. PERF001 and PERF002 are proposed and unverified. [Selector identity](../runtime/selector-identity.md) and [runtime boundary](../runtime/compiler-vm-boundary.md) own the contract being optimized.
