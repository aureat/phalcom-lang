# Runtime

> Sources: object-model/runtime specifications; RUNT001–RUNT002, COMP001–COMP002, MEMM001
> Raw: [runtime specification and program snapshot](../raw/runtime/2026-09-08-runtime-sources.md)
> Updated: 2026-09-08

The runtime domain owns object/value representation, compiler-to-VM execution, lifecycle, memory, and runtime error behavior. It neighbors [semantic](../semantic/overview.md) for authoritative products, [native](../native/overview.md) for host contracts, [collections](../collections/overview.md) for value families, [concurrency](../concurrency/overview.md) for fibers, and [performance](../performance/overview.md) for measured optimization.

## Reading order

- [Representation and object model](representation-and-object-model.md)
- [Compiler/VM boundary](compiler-vm-boundary.md)
- [Lifecycle and memory](lifecycle-and-memory.md)
- [Selector identity](selector-identity.md)

RUNT002 is an as-built evidence program with unverified verification metadata; proposed runtime/compiler programs remain proposals.
