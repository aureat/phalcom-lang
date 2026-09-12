# CONC001 reactor implementation removal note

Delete the old:

```text
docs/implementation/CONC001-fiber-scheduling/
  C2-scheduler-and-reactor/
  reactor-worker-pool-and-timers-spec.md
```

only after the following are committed:

- CONC002.C3 reactor migration map;
- C3.P1 phase-1 reactor plan;
- C3.P2 poller plan;
- live normative implementation links updated to CONC002.C3.

The removed file is not a normative architectural authority. Its useful requirements are preserved by the migration ledger.

Do not delete or rewrite:

- PDR-0003;
- PDR-0004;
- `docs/spec/current/stdlib/reactor.md`;
- historical wiki/raw snapshots.

If repository history prefers retaining superseded plans instead of deleting them, replace the old reactor file body with a short `SUPERSEDED -> CONC002.C3` record rather than leaving it `dispatch-ready`.
