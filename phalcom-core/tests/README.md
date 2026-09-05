# Core test architecture

The core crate exposes focused integration targets: `core`, `language-corpus`,
and `cli-smoke`.

`tests/core/` groups compiler, runtime, and semantic-responsibility tests.
The broad source-language golden corpus lives in `tests/language_corpus/` and
runs through the production compiler API with one fresh VM and output sink per
fixture. `cli-smoke` keeps process-boundary behavior—status codes, path errors,
and diagnostic rendering—small and explicit.

The path boundary is architectural, not a Cargo-package boundary:
`phalcom-semantic` asserts formal identities and match proofs; compiler tests
assert executable projection; VM tests assert runtime behavior and ownership.

Use focused commands during migration:

```text
cargo test -p phalcom-core --test core language::algebraic_data
cargo test -p phalcom-core --test core memory
cargo test -p phalcom-core --test language-corpus booleans
cargo test -p phalcom-core --test cli-smoke
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

`language-corpus` compiles fixtures before creating their VM. Successful cases
capture `System.print` and raw system writes in a VM-owned sink; negative cases
retain structured compile/runtime errors and assert their diagnostic text
without emulating CLI exit codes.
