# Typing Bootstrap Contract

The typing package contains mutually referential descriptors: `Type` mentions `AppliedType`, `AppliedType` mentions `TypeEnvironment`, and `TypeParameter` is itself a `Type`. A production bootstrap therefore performs declaration indexing before resolving typing annotations.

The required sequence is:

1. Allocate trusted shells for the typing classes and protocol descriptors.
2. Parse every file in `src/typing/` and register its top-level declarations.
3. Resolve type-expression references against the complete declaration index.
4. Compile ordinary Phalcom method bodies.
5. Replace each `@native` source anchor with its bootstrap-installed Rust binding.
6. Freeze trusted descriptor classes and the reserved `<...>` selector against replacement.

This is not a general namespace merge for user modules. It is the same kind of staged bootstrap already required for core `Class`, `Method`, and primitive bindings, exposed here so the standard-library source remains readable and authoritative.
