# Live reactor implementation-link patches

Apply after the CONC002.C3 plans are committed.

Do not rewrite historical wiki/raw snapshots.

## Canonical implementation plans

Phase 1:

```text
docs/implementation/CONC002-concurrency-control-and-failure-observability/
C3-reactor-and-external-completion/
CONC002.C3.P1-reactor-core-workers-timers-and-executor-liveness.md
```

Poller phase:

```text
docs/implementation/CONC002-concurrency-control-and-failure-observability/
C3-reactor-and-external-completion/
CONC002.C3.P2-poller-and-external-readiness.md
```

## Live files requiring link repair

At minimum:

```text
docs/spec/current/stdlib/net.md
docs/spec/current/stdlib/process.md
docs/spec/current/stdlib/cancellation.md
docs/pdr/0015-network-surface-tcp-dns-endpoints.md
docs/pdr/0016-poller-backend-is-mio.md
docs/pdr/0017-future-cancel-is-renunciation.md
```

### From `docs/spec/current/stdlib/*`

Replace old U-REACTOR implementation links with:

```markdown
../../../implementation/CONC002-concurrency-control-and-failure-observability/C3-reactor-and-external-completion/CONC002.C3.P1-reactor-core-workers-timers-and-executor-liveness.md
```

Use C3.P2 for poller-specific references.

### From `docs/pdr/*`

Use:

```markdown
../implementation/CONC002-concurrency-control-and-failure-observability/C3-reactor-and-external-completion/CONC002.C3.P1-reactor-core-workers-timers-and-executor-liveness.md
```

or the P2 path for poller-specific statements.

## Status integrity

Do not change these statuses while repairing links:

```text
PDR-0015 Proposed
PDR-0016 Proposed
PDR-0017 Proposed
```

PDR-0003/PDR-0004 remain Accepted.

## Negative searches

```bash
rg -n 'forge/units/U-REACTOR/implementation-spec\.md' docs/spec/current docs/pdr
rg -n 'reactor-worker-pool-and-timers-spec' docs/spec/current docs/pdr docs/implementation/CONC002
```

Expected: zero live-current matches after migration, excluding historical/raw snapshots.
