---
id: CONC002.C3.MIGRATION
program: CONC002
checkpoint: CONC002.C3
kind: migration-ledger
status: PROPOSED
prepared: 2026-09-12
---

# Reactor implementation ownership migration map

## 1. Purpose

This ledger moves **implementation ownership**, not architectural authority.

The following remain where they are:

- PDR-0003 — no user-visible shared-memory threads; worker plain-data boundary;
- PDR-0004 — IO is Future-shaped and reactor-owned; reactor before IO surfaces;
- `docs/spec/current/stdlib/reactor.md` — normative reactor machinery contract.

The historical CONC001 reactor implementation record is mined for requirements and then removed as an active implementation authority.

## 2. Migration law

No old reactor requirement may disappear merely because its implementation owner changes.

Every requirement is classified as:

- `C3.P1` — accepted-decision phase-1 reactor work;
- `C3.P2` — poller-backed readiness work, gated by PDR-0016;
- `C5` — public cancellation/structured policy;
- `HOST-SURFACE` — filesystem/network/process selector work;
- `HISTORICAL` — provenance, not current implementation authority.

## 3. Old CONC001 reactor requirement mapping

| Old requirement | New owner | Revision |
|---|---|---|
| generation-tagged reactor token | C3.P1 | retain; identity remains distinct from Fiber park/admission/control identities |
| token registry | C3.P1 | retain; registry roots the completion target |
| registration GC roots | C3.P1 | retain and strengthen release tests |
| bounded worker pool | C3.P1 | retain; plain-data-only jobs/completions |
| worker MPSC completion channel | C3.P1 | retain |
| `Job` / `Completion` plain-data boundary | C3.P1 | retain as structural compile-time law |
| timer heap | C3.P1 | retain; monotonic, no timer thread |
| safepoint ingress | C3.P1 | retain |
| stale-token drop | C3.P1 | retain |
| `System.sleep` | C3.P1 | revise result to `Future<Unit>` |
| `.ph` `System.runScheduled` completion pump | C3.P1 | **replace** with post-C2 VM-executor integration |
| `System.nextCompletion_` | C3.P1 | expected removal; only retain if post-C2 evidence proves a VM-internal equivalent needs source exposure |
| `System.parkForCompletion_(_)` | C3.P1 | expected removal; executor blocks in reactor directly |
| ready-queue-empty liveness rule | C3.P1 | retain as executor three-way exit/progress law |
| worker completion round-trip | C3.P1 | retain |
| timer ordering / zero timer | C3.P1 | retain |
| shutdown | C3.P1 | retain |
| leak reporting | C3.P1 | retain; public cancellation semantics deferred |
| poller backend | C3.P2 | governed by PDR-0016 gate |
| socket readiness | C3.P2 mechanism only | user network API remains host-surface program |
| deregistration/generation invalidation | C3.P1 | mechanism lands now; public cancellation belongs C5 |
| `Future.cancel` | C5 | not C3 |
| Task cancellation | C5 | not C3 |
| filesystem selectors/jobs | host-surface program | C3 supplies worker machinery only |
| network selectors | host-surface program | C3 supplies poller machinery only |

## 4. Normative reactor spec amendments required

The existing machinery spec remains normative, but post-C2 implementation architecture requires these amendments:

1. implementation owner becomes `CONC002.C3`;
2. replace broken `forge/units/U-REACTOR/implementation-spec.md` references;
3. completion delivery is integrated into the VM-owned executor rather than a guest `.ph` pump;
4. remove the requirement that two pump seams be user/source-visible floor operations;
5. define `System.sleep(Int) -> Future<Unit>`;
6. keep generation-tagged registrations, GC roots, stale drop, monotonic timer, liveness, worker plain-data and shutdown laws unchanged;
7. PDR-0016 remains a hard gate for the poller phase.

## 5. Identity discipline

Never reuse one identity for another:

```text
Fiber handle
Fiber park generation
scheduler admission identity
C2 control/frame identity
reactor registration token
future C5 cancellation generation
```

A reactor event may authorize completion of one reactor registration. Completion then settles a Future or another VM-owned target, whose own waiter wakes authorize Fiber scheduling independently.

## 6. Deletion gate for old CONC001 reactor spec

Delete `CONC001.../reactor-worker-pool-and-timers-spec.md` only after:

- C3.P1 contains every P1 row above;
- C3.P2 contains every poller row above;
- `docs/spec/current/stdlib/reactor.md` points to C3;
- live PDR/spec implementation links no longer point at the missing U-REACTOR file or old CONC001 implementation record;
- the old file is referenced only by historical snapshots, if any.

## 7. Non-migration list

Do not move into C3:

- Future state representation;
- detachable Future subscriptions;
- CompletionSource;
- `Future.all` / `race` / `timeout`;
- Backoff policy;
- TaskScope/cancellation;
- Channel/select;
- filesystem/network user APIs.
