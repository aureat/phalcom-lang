# PDR-0017 — `Future#cancel` is renunciation: settle `#cancelled` now, suppress unstarted work best-effort, interrupt nothing

- Status: Proposed
- Date: 2026-07-20
- Related: [`reactor.md`](../spec/v0.2/core/reactor.md) §7/§11 **Q-R4** (the deferred
  unit this discharges; its §7.1-.3 obligations are the fixed substrate),
  [PDR-0004](0004-io-is-future-shaped-reactor-owned.md) (Consequences: "Cancellation is
  now unavoidable … a leaked registration is an fd leak"),
  [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §5 (leak reporting —
  the composition this record must not break),
  [PDR-0015](0015-network-surface-tcp-dns-endpoints.md) (its Q-N5 timeout/Happy-Eyeballs
  deferral is blocked on this),
  [ADR-0030](../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  (`Future` is pure `.ph`; the C-FUT-3 settle-once contract this leans on),
  [`impl/reactor.md`](../spec/v0.2/impl/reactor.md) (the registry, tokens, and pump this
  plugs into).

## Context

Reactor.md §7 deferred the user-facing cancellation surface but shipped its substrate:
generation-tagged tokens, deregistration-as-generation-bump, stale-completions-dropped-
at-drain (law 7), and the rule that a future `cancel` "composes with the token mechanism
above rather than adding a second one." PDR-0004 said the deferral could not be
indefinite. It ends here, because two consumers now wait on it: PDR-0015's Q-N5 (connect
timeouts and Happy Eyeballs both require abandoning a losing attempt), and reactor.md
§7.2's leak condition (a fiber parked on a registration nothing can complete has no
remedy without cancel).

Two facts in the tree shape the whole design:

1. **`Future` is pure `.ph` and settle-once is already a silent no-op.** `settleValue`/
   `settleError` on a settled receiver return `self` unchanged — C-FUT-3, `core.ph`
   (`class Future`, the settle-once comment above `settleValue`). Not a raise. So a
   producer that settles a future the consumer has meanwhile cancelled hits an
   *already-shipped* no-op path; cancellation needs **no amendment** to the settlement
   contract.
2. **`await` on a rejected future raises the settled error** (`core.ph`, `await`'s
   `_state == "rejected"` arm). So a cancelled future's waiters observe cancellation as
   a catchable raise of the settled error — the existing rejection channel, no new one.

What cancellation *cannot* mean here: interrupting work. Workers run blocking syscalls
(PDR-0004 §3) and an uninterruptible syscall is exactly that; there is no thread to kill
(PDR-0003) and no finalizer to lean on (PDR-0005 §1).

## Decision

### 1. `cancel` is renunciation of the result, and that is all it promises

Three clauses, in decreasing strength:

- **Guaranteed:** the future settles rejected with a `#cancelled` error *at the cancel
  call*, its waiters are rescheduled through the ordinary drain, and its reactor
  registration (if any) is released — generation bumped, poller/timer interest dropped,
  no longer a GC root, no longer counted by the pump's exit condition.
- **Best-effort:** work that has **not started** is suppressed — a pooled job still in
  the queue is skipped at dequeue; a poller op that never got its syscall never gets it.
- **Never:** work that has **started** is not interrupted. A running `getaddrinfo`
  or `rename(2)` completes on its worker; its completion arrives stale and is dropped
  (law 7). The world may have changed — a cancelled `Fs.rename` may still have renamed.
  This is the fundamental worker-pool truth (libuv's `uv_cancel` fails on started work
  for the same reason), documented rather than fought.

### 2. The surface: two selectors on `Future`

```
Future#cancel      -> Bool     // true iff THIS call moved pending -> cancelled
Future#isCancelled -> Bool
```

`cancel` on an already-settled or already-cancelled future returns `false` and does
nothing — idempotent, never an error (Java's `Future.cancel` boolean shape; the caller
who needs to know whether they won the race can branch). `isCancelled` distinguishes
"rejected because cancelled" from "rejected because failed" without kind-sniffing the
error. Errors carry `kind: #cancelled` — a dedicated `CancelledError < Error` class
until the traceback plan's T3/T6 kind carrier lands (the U-RESOURCE §2.4 mechanism);
the symbol joins the §8.1 normative table with that plan's lane.

### 3. `cancel` is ordinary `.ph`; the one native is deregistration

`cancel` runs in `.ph` and settles via the existing `settleError` — the
natives-never-settle architecture ([`impl/reactor.md`](../spec/v0.2/impl/reactor.md) §1)
is untouched, because `cancel` is not a native. The single new floor primitive is

```
System.cancelRegistration_(_)   -> Bool    // future object; releases its registration
```

(+1, `NEW_CANCEL`), which owns everything Rust-side: reverse-lookup future → token,
generation bump, poller/timer/queue release, root-set removal, pending-count decrement.
Returns whether a live registration existed — futures with no registration (plain
`Future.new()`, `Future.async` results) cancel fine with nothing to release.

### 4. Late settlement is already benign — C-FUT-3 does the work

A producer that settles after cancel hits the settle-once no-op that shipped with
U-FUTURE Slice B. This closes the race that fill-time staleness checking alone cannot:
a completion that entered the pending buffer *before* cancel bumped the generation is
past the drain's staleness check, but its `.ph` settlement lands on a settled future
and drops. **Both defenses are mandatory** — the drain-side drop (law 7) keeps stale
completions from minting values needlessly; the settle-side no-op catches the
in-flight window. Neither alone is sound.

Corollary for `.ph`-produced futures: cancelling a `Future.async` result does not stop
the driver fiber — the work runs to completion and its settlement drops. Renunciation,
uniformly, for IO and non-IO futures alike. Cooperative early exit is what
`isCancelled` is for.

### 5. Composition with leak reporting, and with `#closed`

- **Cancelled ⇒ not a leak.** Deregistration removes the registration from the pump's
  pending set and the leak surface in the same motion; reactor.md §7.2's "fiber parked
  on a registration nothing can complete" cannot apply because cancel *settles* the
  future — the parked fiber resumes with the raise. Cancel is the §7.2 condition's
  remedy, not a new source of it.
- **`#cancelled` is not `#closed`.** Same mechanical spine (deregister + settle `Err`),
  different initiator and different meaning: `#closed` is the *resource* going away
  under an operation (PDR-0015 ruling 8); `#cancelled` is the *consumer* renouncing
  the result. Conflating them would send a handler to the wrong recovery — retrying a
  cancelled op is wrong, retrying on a fresh connection after `#closed` may be right.

### 6. Cancellation is shallow

`cancel` touches exactly the receiver. It does not propagate up `then`/`map`/`catch`
chains to the upstream future, does not cancel sibling waiters' interests, and does not
cancel fibers. Multiple awaiters of one future all observe the one cancellation (a
settlement is a settlement). Propagation — structured concurrency, linked tokens,
cancel-on-drop scopes — is real design space with real precedents (JS `AbortSignal`
chaining, Kotlin structured concurrency) and is **deferred with a name**
([`core/cancellation.md`](../spec/v0.2/core/cancellation.md) Q-C1/Q-C2), not implied.

## Consequences

- **Q-N5 unblocks upon ratification**: a connect timeout is `sleep(t)` racing the
  attempt with `cancel` on the loser; Happy Eyeballs is the same shape twice.
- **The cost, named plainly:** a cancelled effectful operation may still have effected —
  users must treat `cancel` on writes/renames as "stop waiting", never "undo"; the
  best-effort suppression clause is deliberately untestable-by-timing (a job can start
  between check and cancel), so its tests assert only the guaranteed clauses; and every
  registering seam now carries reverse-lookup bookkeeping on the hot path (one map
  insert/remove per IO op).
- **What this precludes:** interruptive cancellation can never be retrofitted onto this
  surface — `cancel -> Bool` returning "already settled" leaves no slot for
  "interrupt requested, pending"; a future propagation design must layer *above*
  shallow `cancel` (composing it), not redefine it.
- `Fiber` cancellation remains unruled and independent (Q-C2).

## Alternatives rejected

- **Drop/scope-based implicit cancellation** (Rust's model). Needs destructors or
  scope hooks; Phalcom has no finalizers by PDR-0005 §1 and withdrew `using` (§6).
  No substrate, and reversing PDR-0005 for this would trade a leak report for a
  finalizer hazard ADR-0050 banked against.
- **A separate token object** (`AbortController`/`CancellationToken`). Buys many-op
  linked cancellation — which is §6's *deferred* propagation problem — at the price of
  a second user-visible mechanism beside the future itself. When propagation is ruled,
  a token composes over `cancel`; starting with the token forecloses the simple form.
- **Interruptive cancellation.** Uninterruptible syscalls, single VM thread, no
  killable threads (PDR-0003). Java's `Thread.interrupt` is the cautionary precedent:
  cooperative in disguise, and every consumer must handle "interrupted anyway".
- **Cancel as un-settlement / third state.** A cancelled-but-pending future hangs its
  waiters (reactor.md law 3: settle once, one channel). JS promises' lack of built-in
  cancellation produced exactly the ecosystem churn (`bluebird`, `AbortSignal`) this
  avoids by settling immediately.
- **`cancel -> None`.** The caller racing a completion cannot learn who won; Java's
  boolean exists because that branch is real (report "timed out" vs "completed").
- **Deferring again.** PDR-0004 called the deferral finite; Q-N5 is now concretely
  blocked, and §7.2's leak condition has no remedy without this surface.
