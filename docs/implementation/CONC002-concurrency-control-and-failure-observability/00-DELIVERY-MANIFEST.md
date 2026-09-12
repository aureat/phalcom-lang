---
kind: delivery-manifest
prepared: 2026-09-12
scope: CONC002 restructuring scopes 5-8
prepared_against_remote_head: 2db7e3780772e847a178918ede239d91e0bcc5fd
---

# CONC002 scopes 5–8 delivery manifest

This bundle implements the documentation/program restructuring agreed for scopes 5–8:

1. migrate reactor implementation ownership out of `CONC001`;
2. rebuild `CONC002.C3` as the complete reactor/external-completion checkpoint;
3. close the stale `CONC001.C2` reactor/scheduler ownership record without losing scheduler provenance;
4. rebuild `CONC002.C4` as the concurrency standard-library completion checkpoint.

The repository remains authoritative. These records were prepared against remote `main` at `2db7e3780772e847a178918ede239d91e0bcc5fd`. Before applying them, rebase names, links, prerequisites, and verification commands onto the actual revision produced by `CONC002.C2.P2`.

## Architectural result

```text
C2.P2
  coroutine consumer / executor separation
  VM-owned executor driving
        |
        v
C3.P1
  phase-1 reactor:
  registrations + worker completion transport
  + timers + executor liveness + System.sleep
        |
        v
C3.P2
  poller-backed external readiness
  + unified reactor wait
        |
        +--------------------+
        |                    |
        v                    v
C4.P1                  C4.P2
Future foundations     aggregation / timeout / Backoff
```

C5 owns cancellation and structured concurrency. C6 owns channels/select.

## Files in this bundle

### Scope 5 — reactor ownership migration

- `CONC002/C3-reactor-and-external-completion/REACTOR-MIGRATION-MAP.md`
- `LINKS-AND-SPECS/REACTOR-LINK-AND-SPEC-AMENDMENTS.md`

### Scope 6 — new C3 reactor checkpoint

- `CONC002/C3-reactor-and-external-completion/CHECKPOINT.md`
- `CONC002/C3-reactor-and-external-completion/CONC002.C3.P1-reactor-core-workers-timers-and-executor-liveness.md`
- `CONC002/C3-reactor-and-external-completion/CONC002.C3.P2-poller-and-external-readiness.md`

### Scope 7 — CONC001 cleanup

- `CONC001/PROGRAM.md`
- `CONC001/STATUS.md`
- `CONC001/C2-scheduler/CHECKPOINT.md`
- `CONC001/C2-scheduler/P1-ready-queue-and-root-drive-historical-closure.md`
- `CONC001/C2-scheduler/scheduler-scope-spec.md`
- `CONC001/C2-scheduler/REACTOR-REMOVAL-NOTE.md`

The old `CONC001-fiber-scheduling/C2-scheduler-and-reactor/reactor-worker-pool-and-timers-spec.md` should be deleted only after the migration map is reviewed and the new C3 plans are committed.

### Scope 8 — new C4 standard-library checkpoint

- `CONC002/C4-concurrency-standard-library/CHECKPOINT.md`
- `CONC002/C4-concurrency-standard-library/C4-MIGRATION-MAP.md`
- `CONC002/C4-concurrency-standard-library/CONC002.C4.P1-future-state-registrations-and-completion-source.md`
- `CONC002/C4-concurrency-standard-library/CONC002.C4.P2-future-composition-timeout-and-backoff.md`

### Program graph follow-up

- `CONC002/PROGRAM-C3-C4-AMENDMENT.md`
- `CONC002/STATUS-C3-C4-AMENDMENT.md`

These amendments replace the transitional “C3/C4 plans not yet published” wording from the scopes 1–3 delivery.

## Files to supersede or remove

Once this bundle is adopted:

- supersede the old `CONC002.C3` library-polish checkpoint and both old P1/P2 plans;
- remove reactor implementation ownership from `CONC001`;
- keep accepted PDR-0003/PDR-0004 and the normative `docs/spec/current/stdlib/reactor.md` as architectural authorities;
- keep PDR-0016 proposed until separately ratified;
- do not rewrite historical wiki/raw snapshots.

## Apply order

1. Commit the new C3 migration map, checkpoint, P1 and P2.
2. Commit the new C4 migration map, checkpoint, P1 and P2.
3. Update CONC002 program/status links.
4. Apply live normative link/spec amendments.
5. Convert CONC001.C2 to historical scheduler provenance and remove reactor ownership.
6. Delete the old CONC001 reactor implementation spec.
7. Supersede the old CONC002.C3 executable plans.
8. Run link/negative searches from the amendment manifest.

Do not delete old implementation records before their requirements are visibly accounted for in the migration ledgers.
