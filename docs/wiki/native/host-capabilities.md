# Host capabilities

> Sources: UNIV001, NATV001–NATV002
> Raw: [native host contract snapshot](../raw/native/2026-09-08-native-host-sources.md)
> Updated: 2026-09-08

Host capabilities cover bytes, filesystem/path, resources, streams, file I/O, and network operations. They are runtime-facing contracts layered on the declarative native pipeline and consumed by [standard-library](../standard-library/overview.md) protocols.

## Boundary

A host operation states its type, effect, lifecycle, error, and cancellation behavior in the primitive contract. The reactor/scheduler may coordinate readiness, but it does not own the host handle's semantic type or cleanup rules. [Lifecycle and memory](../runtime/lifecycle-and-memory.md) owns resource reachability and cleanup.

NATV001 and NATV002 are proposed and unverified; this page is a decomposition map, not a completion claim.
