# SC-3 Implementation State

## Baseline

- plan baseline: `abb2b5d80654e2525d68f4ea8ff9d32b810330b3`
- execution baseline: `1863dee7f11fe853bb30ea25348ec25b50e40b3a`
- branch: `main`
- working tree: unrelated pre-existing changes preserved

## Established invariants

- `RecordRow` remains separate from proper `Type`.
- Closed and open Records use canonical `TypeData::Record(RecordRowId)`.
- `RecordAccess` is removed from semantic production code and row solving.
- Record inference, checked formation, scoped lowering, publication, and generic-call integration are implemented.
- Current `InferenceTerm` already contains a Record form; row metavariables remain in the separate `RecordRowVarId` domain.

## Decisions

- Implement inline in current checkout; no delegation.
- Preserve unrelated dirty files; stage only explicit SC-3-owned files for delivery.
- Treat live source and focused tests as authority where plan baseline has drifted.

## Evidence ledger

| Task/checkpoint | Command | Result | Proves |
|---|---|---|---|
| Task 0 | `git rev-parse HEAD` | `1863dee7f11fe853bb30ea25348ec25b50e40b3a` | execution baseline |
| Task 0 | `graphify query ...` | canonical row/inference/call/metadata nodes found | affected ownership map |
| Tasks 1–8 | focused semantic suites | pass: row solver 12, annotations 39, composition 16, materialization 3, row inference 2, generic application 7 | checked row formation, structural relation, separate inference domains, and call integration |
| Tasks 9–10 | focused semantic suites | pass: record-row polymorphism 9, scoped/open-row coverage in annotations and type lambdas | prefix pattern semantics and capture-safe scoped tails |
| Tasks 11–12 | focused semantic suites | pass: metadata 11; row diagnostics included in integration coverage | stable diagnostics and tail-sensitive metadata |
| Task 13 | incremental record-row suite | pass: 6 | invalidation, cold/incremental equivalence, retained snapshots, and solver-state isolation |
| Focused certification | `RUSTFLAGS='' cargo check -p phalcom-semantic --tests` | pass | semantic test target compiles |
| Focused certification | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic` | pass: 1001 passed, 0 failed, 48 ignored | existing semantic binary remains green |
| Type metadata | `RUSTFLAGS='' cargo test -p phalcom-type-meta` | pass: 5 passed | schema validation remains green |
| Workspace check | `RUSTFLAGS='' cargo check --workspace` | pass | workspace compiles |
| Workspace tests | `RUSTFLAGS='' cargo test --workspace` | interrupted after unrelated core test failure (`core_collections::range_literals_drive_collection_slices`) | full workspace certification remains open |

## Negative gates

| Search | Result | Meaning |
|---|---|---|
| SC-3 deletion ledger | pending | run after implementation |

## Deferred gates

- Full workspace test certification was interrupted after an unrelated core test failure; rerun/classify against a clean baseline.
- Repository-wide format, clippy, deletion-ledger, and graph-refresh gates remain pending.

## Active incident

Plan baseline drift: live `main` already contained prototype Record inference and metadata open-node handling. SC-3 work reconciled each task against current source. Final certification is not yet complete because workspace tests exposed an unrelated core failure before the run was stopped.

## Next action

Task 14 — complete repository-wide certification and deletion-ledger review; do not mark SC-3 release-complete until every final gate passes.
