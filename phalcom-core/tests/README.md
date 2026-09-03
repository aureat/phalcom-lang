# Core test architecture

The core crate exposes one integration binary: `core`.

`tests/core/` is the single package boundary. Its folders group tests by
semantic responsibility: source-language behavior and ADT execution live under
`language/`; compiler projection, collections, execution, memory, modules,
reflection, observability, REPL, and object-model contracts live beside it.

The path boundary is architectural, not a Cargo-package boundary:
`phalcom-semantic` asserts formal identities and match proofs; compiler tests
assert executable projection; VM tests assert runtime behavior and ownership.

Use focused commands during migration:

```text
cargo test -p phalcom-core --test core language::algebraic_data
cargo test -p phalcom-core --test core memory
```

Shared helpers should stay small and domain-neutral. ADT-specific helpers belong
under the language test tree; they must not become a second semantic authority.

## VM bootstrap tiers

Choose lowest tier whose runtime contract contains behavior under test:

| Helper | Provides | Use for |
|---|---|---|
| `kernel_vm()` | Heap, object model, no native or source Universe state | Product/heap-only invariants |
| `native_vm()` | Kernel plus registered native primitives, no source-authored Universe state | Direct native and floor-runtime contracts |
| `universe_vm()` | Full shipping `VM::new()` bootstrap | Source-installed methods, ADTs, generic/runtime language behavior |

Source-language helpers such as `run_inline` and `compile_inline` remain full
Universe helpers by default. Do not lower a test whose assertion depends on
source-installed methods or semantic roots.
