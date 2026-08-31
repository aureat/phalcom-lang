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
