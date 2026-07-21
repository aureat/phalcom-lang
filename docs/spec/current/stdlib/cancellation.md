# Specification — Cancellation (`Future#cancel`, `Future#isCancelled`, `CancelledError`)

> **Status:** **Proposed — normative upon ratification of
> [PDR-0017](../../../pdr/0017-future-cancel-is-renunciation.md)** (rule 5: no
> unit builds this until it flips). Discharges [`reactor.md`](reactor.md) §11 **Q-R4**
> on the substrate §7 already binds (token generations, deregistration-as-bump,
> stale-drop at drain). Already-Accepted inputs:
> [PDR-0004](../../../pdr/0004-io-is-future-shaped-reactor-owned.md)
> (Consequences — cancellation named unavoidable),
> [PDR-0005](../../../pdr/0005-resources-are-disposable-handles-not-finalized.md)
> §5 (leak-report composition), ADR-0030 / C-FUT-3 (settle-once no-op, the load-bearing
> shipped fact — `core.ph` `class Future`, `settleValue`'s settle-once comment).
> **Floor delta: +1** (`System.cancelRegistration_(_)`, `NEW_CANCEL`) — the U-SCHED
> seam-precedent shape; census at impl time under
> [PDR-0012](../../../pdr/0012-numeric-tower-implementation-and-floor-amendment.md)
> ruling 21's rebase discipline.
> **Build order:** needs U-REACTOR (phase 1) only — `System.sleep` is a sufficient test
> substrate; independent of and parallel-safe with U-NET.
>
> **Owner:** unassigned.

## 1. What `cancel` means — the three-clause contract

PDR-0017 ruling 1, restated as the contract every selector below serves:

| Clause | Strength | Content |
|---|---|---|
| settle | **guaranteed** | the future settles rejected with `CancelledError` (`kind: #cancelled`) at the `cancel` call; waiters reschedule through the ordinary drain |
| release | **guaranteed** | any reactor registration is released: generation bumped, poller/timer interest dropped, GC root removed, pump exit-count decremented |
| suppress | best-effort | unstarted work is skipped (queued pool job dropped at dequeue; unfired poller op never syscalls) |
| interrupt | **never** | started work completes on its worker; its completion arrives stale and drops (reactor law 7). A cancelled effectful op may still have effected |

"Stop waiting", never "undo".

## 2. Surface

```
Future#cancel      -> Bool    // true iff this call moved pending -> cancelled
Future#isCancelled -> Bool
```

- `cancel` on a settled or already-cancelled future: `false`, no effect, no error —
  idempotent. The `Bool` is the race outcome (did *I* cancel it, or had it settled?),
  which timeout patterns branch on.
- `isCancelled` distinguishes rejected-by-cancel from rejected-by-failure without
  inspecting the error; producers use it for cooperative early exit (§4).
- Awaiting a cancelled future **raises** the `CancelledError` — the existing rejected
  arm of `await` (`core.ph`); `catch(_)`/`on` observe it like any error. No new
  channel, no third state.
- `CancelledError < Error`, pure `.ph`, no natives — the dedicated-class kind mechanism
  until traceback T3/T6 land the `kind` carrier.

## 3. The cancel algorithm — ordinary `.ph`, one native

```phalcom
cancel() {
  if (self.isReady) { return false }
  System.cancelRegistration_(self)      // release; Bool ignored here — no-registration futures cancel too
  self.settleError(CancelledError.new)  // ordinary .ph settlement; waiters drain normally
  _cancelled = true
  return true
}
```

Order is normative: **release before settle** (a waiter resumed by the settlement must
not observe a still-armed registration), settle before flagging (nothing between them
can run — single VM thread, no re-entry in this sequence). Natives never settle
anything: `cancelRegistration_` only releases; the settlement above is `.ph` — the
[`../../forge/units/U-REACTOR/implementation-spec.md`](../../forge/units/U-REACTOR/implementation-spec.md) §1 architecture holds with zero exceptions.

## 4. Late settlement, and non-IO futures

Two defenses, **both mandatory** (PDR-0017 ruling 4):

1. **Drain-side**: a completion whose token generation is stale drops at the safepoint
   fill (reactor law 7) — the normal path, no minting wasted.
2. **Settle-side**: a completion that entered the pending buffer *before* the
   generation bump is past defense 1; its `.ph` settlement lands on the settled future
   and hits C-FUT-3's shipped no-op. Benign by construction, not by review.

Corollary: cancelling a `Future.async` result (or any `.ph`-settled future) stops
nothing — the driver fiber runs to completion, its `settleValue` drops. Renunciation is
uniform across IO and non-IO futures. A producer that wants to stop early polls
`future.isCancelled` at its own checkpoints; that is the entire cooperative story, and
it is opt-in.

## 5. Composition laws

1. **Cancelled ⇒ not a leak.** Release removes the registration from the pump's
   pending set and the leak surface in one motion (PDR-0005 §5 posture; reactor.md
   §7.2's parked-fiber condition is *remedied* by cancel — the fiber resumes with the
   raise — never triggered by it).
2. **`#cancelled` ≠ `#closed`.** Same spine, different initiator: resource-went-away
   vs consumer-renounced (PDR-0015 ruling 8 / PDR-0017 ruling 5). Handlers may retry
   after `#closed`; retrying `#cancelled` re-does what the canceller renounced.
3. **Liveness updated, not weakened**: the pump's exit conjunction (reactor.md §4)
   counts a cancelled registration as gone immediately — a program whose last pending
   op is cancelled exits cleanly without a synthetic wake.
4. **Settle-once is untouched.** `cancel` adds no amendment to C-FUT-3; it is a client
   of it.
5. **Cancellation is shallow** (PDR-0017 ruling 6): receiver only; no chain
   propagation, no fiber cancellation, all awaiters observe the one settlement.

## 6. Conformance harness

Timer-only where possible (U-REACTOR phase 1 suffices); rows marked ⊕ need a pool job
(phase-1 `Job::Test`).

| Check | Asserts |
|---|---|
| cancel pending sleep | `sleep(10_000).await` in a fiber; cancel from another; awaiter catches `CancelledError` promptly; **program exits immediately** (release decrements the exit count — law 3) |
| cancel returns the transition | first `cancel` → `true`; second → `false`; `isCancelled` → `true` both times |
| cancel settled | settle, then `cancel` → `false`; `isCancelled` → `false`; value unchanged |
| await-after-cancel | awaiting an already-cancelled future raises `CancelledError` without parking |
| multiple awaiters | three fibers awaiting; one cancel; all three catch `#cancelled` |
| late settlement drops ⊕ | cancel a future whose `Job::Test` completion is already in flight; result value never observed; no error, no crash (defense 2) |
| stale drain drop ⊕ | cancel before the worker finishes; completion dropped at fill (defense 1); heap-side: nothing minted |
| async renunciation | `Future.async` with an observable side effect; cancel; side effect **still happens**; settled value stays `CancelledError` |
| cooperative exit | producer loop checking `isCancelled` stops early; documents §4's opt-in story |
| not a leak | cancel a registered op, run to exit under `strictResources(true)`: clean report (law 1) |
| GC ⊗ cancel | after cancel, the future is collectable once unreferenced (root removed); forced `System.gc` + liveness assertion via the phase-1 M-row pattern |
| no-registration cancel | `Future.new()` cancels: `true`, waiters raise, `cancelRegistration_` reported no registration and nothing broke |

## 7. Open questions

| # | Question | Notes |
|---|---|---|
| Q-C1 | Propagation / structured concurrency | linked cancellation across `then`-chains and op groups (JS `AbortSignal` chaining, Kotlin scopes). Layers *above* shallow `cancel`, composing it — PDR-0017's preclusion note is the boundary |
| Q-C2 | `Fiber` cancellation | killing/unwinding a fiber is an unwind-semantics question (ensure interaction, E004 territory), not a future-settlement question; deliberately not touched |
| Q-C3 | Timeout sugar | `future.timeout(ms)` = race + cancel; pure `.ph`, no new floor — worth adding only once Q-N5's consumers exist |

## 8. What this document does not cover

- **The registration substrate** — [`reactor.md`](reactor.md) §3/§7, built by U-REACTOR.
- **`#closed` settlement on resource close** — PDR-0015 ruling 8 / stdlib/net.md §7
  (close semantics, not cancellation).
- **Interrupting workers, cancelling fibers, undoing effects** — §1's "never" clause
  and Q-C2.
- **Implementation seams** — [`../../forge/units/U-CANCEL/implementation-spec.md`](../../forge/units/U-CANCEL/implementation-spec.md)
  (U-CANCEL).
