# Reactor live-link and normative-spec amendment manifest

## 1. Purpose

After C3 plans are committed, repair live implementation links and update the normative machinery wording to the post-C2 executor model.

Do not rewrite historical wiki/raw snapshots.

## 2. Live documents to inspect/update

At minimum:

```text
docs/spec/current/stdlib/reactor.md
docs/spec/current/system.md
docs/spec/current/stdlib/net.md
docs/spec/current/stdlib/process.md
docs/spec/current/stdlib/cancellation.md
docs/pdr/0015-network-surface-tcp-dns-endpoints.md
docs/pdr/0016-poller-backend-is-mio.md
docs/pdr/0017-future-cancel-is-renunciation.md
```

Replace live implementation links to the missing:

```text
docs/forge/units/U-REACTOR/implementation-spec.md
```

with the appropriate C3 plan/migration path.

## 3. `stdlib/reactor.md` amendments

Preserve these binding laws:

- Future-shaped blocking operations;
- worker/poller split;
- worker plain-data boundary;
- VM-thread settlement;
- generation-tagged registrations;
- registration GC roots;
- stale completion drop;
- monotonic timers;
- liveness three-way conjunction;
- shutdown/leak rules.

Revise:

### Implementation owner

```text
Owner: CONC002.C3
Phase-1 implementation: CONC002.C3.P1
Poller phase: CONC002.C3.P2, gated by PDR-0016
```

### Completion pump wording

Old:

```text
System.nextCompletion_
System.parkForCompletion_
.ph runScheduled completion drain
```

New normative mechanism:

```text
cross-thread/poller event
-> VM reactor ingress
-> executor-owned completion delivery on VM thread
-> ordinary Future settlement
-> Future waiter wake
-> queued execution on later executor turn
```

Do not require guest `.ph` code to be the reactor driver.

### Floor

Recompute floor delta against the actual P1 implementation.

Expected minimum new internal seam:

```text
System._$registerSleep(_,_)
```

or repository-equivalent.

Do not retain `+3` merely because the old phase-1 plan proposed two guest pump seams.

### Sleep

Amend to:

```phalcom
System.sleep(_ milliseconds: Int) -> Future<Unit>
```

with completion value `()`.

If numeric conventions require `Number` instead of `Int`, resolve deliberately and keep the Future payload `Unit`.

### PDR-0016

Keep PDR-0016 status visible as Proposed/Blocked until ratified.

## 4. `system.md`

Update `sleep(_)` from “unbuilt U-REACTOR” to the C3 ownership/path.

Once C3.P1 lands, mark it landed and document `Future<Unit>`.

Root-drive wording should follow the final C2.P2 executor rather than “internal scheduler-resume mode”.

## 5. `net.md`, `process.md`, cancellation/PDR links

Only update implementation-owner links.

Do not silently ratify proposed network/cancellation PDRs while fixing paths.

## 6. Negative link searches

After applying:

```bash
rg -n 'forge/units/U-REACTOR/implementation-spec\.md' docs/spec/current docs/pdr
rg -n 'CONC001.*reactor-worker-pool-and-timers' docs/spec/current docs/pdr docs/implementation/CONC002
rg -n 'System\.nextCompletion_|System\.parkForCompletion_' docs/spec/current/stdlib/reactor.md
```

Expected:

- zero live missing U-REACTOR implementation links;
- zero current implementation ownership pointing to CONC001 reactor record;
- old pump seams absent from normative required surface unless the final post-C2 architecture proves a retained internal equivalent.

## 7. PDR integrity

Do not edit accepted PDR decisions merely to make the implementation plan easier.

PDR-0003/PDR-0004 remain accepted architecture.

PDR-0016 remains proposed until separately ratified.
