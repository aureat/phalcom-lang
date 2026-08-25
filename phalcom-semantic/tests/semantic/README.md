# Canonical semantic test tree

`tests/semantic.rs` is the crate's only integration-test binary. Test modules
under `tests/semantic/` are organized by semantic responsibility:

- `foundations/`: pure type, knowledge, inference, flow, diagnostic, and explanation laws;
- `capabilities/`: source programs proving language-semantic behavior;
- `integration/`: workspace, source-index, presentation, advisory, native, and compiler integration;
- `incremental/`: revisions, fingerprints, dependency tracking, query ownership, and reuse;
- `advanced/`: effects, termination, contracts, and prover integration;
- `golden/`: broad composition fixtures from Plan 3;
- `support/`: shared fixtures, locators, expectations, diagnostics, dependencies, and rendering.

Use shallow assertions for common laws and deep expectations when status, origin,
contracts, explanations, or dependencies are part of the law. New scenarios are
classified `READY`, `RED-CAPABILITY`, `STAGED`, or `GATED`; unsupported source
syntax is not a checker regression. VM bytecode and runtime performance remain a
separate test domain.

Typical commands:

```text
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic --test semantic capabilities::generics
cargo test -p phalcom-semantic --test semantic incremental::db
```
