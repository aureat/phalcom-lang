# Collection library surface

> Sources: Map/Set, Bytes, and standard-library buildout specifications; STDL002
> Raw: [standard-library specification and program snapshot](../raw/standard-library/2026-09-08-standard-library-sources.md)
> Updated: 2026-09-08

Collection library surfaces layer protocols and combinators over native or runtime-owned storage. Map/Set use hash/equality contracts; Bytes is a fixed-length mutable octet buffer with fallible UTF-8 decoding; stream/path/network items remain host-capability seams.

## Boundary

A native primitive may provide an inexpressible or bulk operation, but library combinators remain language-level behavior. [Native host capabilities](../native/host-capabilities.md) owns host effects and lifecycle; [collection traversal](../collections/traversal.md) owns eager/lazy consumption. STDL002 is proposed and not started.
