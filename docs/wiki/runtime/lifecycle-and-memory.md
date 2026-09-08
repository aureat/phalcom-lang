# Lifecycle and memory

> Sources: memory-management specification; MEMM001, RUNT001–RUNT002
> Raw: [runtime specification and program snapshot](../raw/runtime/2026-09-08-runtime-sources.md)
> Updated: 2026-09-08

Lifecycle owns allocation, rooting, frame/block lifetime, cleanup, and garbage collection. A runtime handle or arena rule must preserve object reachability across calls, fibers, native boundaries, and error unwinding.

## Cleanup

Protected execution and non-local return must either complete against a live frame token or produce a defined runtime error. Native resources require explicit ownership/cleanup contracts; [host capabilities](../native/host-capabilities.md) documents those boundaries.

MEMM001 is proposed. Runtime records can be as-built while verification remains unverified; these are deliberately separate status labels.
