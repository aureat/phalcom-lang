# Hypothesis for Phalcom

This package is being rebuilt around a small, statically typed property-testing
kernel. The active implementation is under `src/`; executable consumer tests
are under `tests/runtime/` and run through `scripts/run_runtime_tests.py`.

## Current kernel

The active source currently provides:

- typed integer, Boolean, index, and byte choices;
- copied choice tapes with scripted and replay providers;
- a seeded provider factory;
- integer and Boolean strategies;
- bounded property search and deterministic tape replay;
- integer and index shrinking, followed by two verification replays;
- separate generated-example, shrink-evaluation, accepted-shrink, discard, and
  verification counters.

Provider, provider-factory, strategy, and property-specification types remain
concrete through the generic pipeline. A property specification carries an
identity symbol used to keep shrinking on the same property failure. The
`Settings` value owns only search limits; seeded randomness is configured by
`SystemRandomChoiceProviderFactory` and injected through its typed
`providerFactory` into `Property.check`.

The package does not yet produce discarded examples. `Discarded` is reserved
for a future assumption/rejection API. `Span` is present as data scaffolding,
but no strategy, example, or shrink pass currently uses it. Composite
strategies, automatic derivation, reporting, persistence, and stateful testing
are also future work.

## Source layout

- `src/` — active implementation and package exports.
- `tests/runtime/` — current executable fixtures.
- `scripts/run_runtime_tests.py` — current runtime test runner, with
  `--probe-blocked` for fixtures awaiting a fresh compiler/runtime check.
- `archive/legacy-0.1.0/` — prior monolithic implementation, docs, examples,
  tests, release verifiers, and historical checksum data.

The current API inventory and kernel behavior are documented in
[`docs/public-api.md`](docs/public-api.md),
[`docs/strategies.md`](docs/strategies.md), and
[`docs/shrinking.md`](docs/shrinking.md). The archived 0.1.0 surface is not the
active package API. Current verification status is in
[`TEST-RESULTS.md`](TEST-RESULTS.md); the generic-conformance blocker is
documented in [`INCIDENT-REPORT.md`](INCIDENT-REPORT.md).

## Runtime fixtures

Build Phalcom once, then run the active runtime fixtures from the repository
root:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --phalcom target/debug/phalcom
```

Fixtures marked `BLOCKED` are reported without execution by default. Pass
`--probe-blocked` to run them and compare current output with their recorded
blocker signatures. `--strict` makes unresolved blocked fixtures fail the run.
