# Implementation spec — the reactor, phase 1: worker pool + timers (U-REACTOR)

> **Status:** dispatch-ready. Governing records **Accepted**:
> [PDR-0004](../../../../pdr/0004-io-is-future-shaped-reactor-owned.md) (all),
> [PDR-0003](../../../../pdr/0003-no-user-visible-threads-fibers-and-isolates.md) §3;
> machinery contract [`../stdlib/reactor.md`](../../spec/current/stdlib/reactor.md).
> **Phase 1 scope ruling (settles the surface spec's Q-R3):** **std-only** — worker
> pool (`std::thread` + `std::sync::mpsc`) and timers; **no poller, no sockets, no new
> dependency**. `epoll`/`kqueue` arrives with a network unit; the completion pipeline
> built here is the part both mechanisms share, and timers need no thread at all (§5).
> **Floor delta: +3** — `System.sleep_(_)`, `System.nextCompletion_`,
> `System.parkForCompletion_(_)` (amends reactor.md's "+1"; the two extra are the pump
> seams, the exact shape U-SCHED's `schedule_`/`nextScheduled` pair was admitted in —
> `primitive/system.rs:56`/`:70`, census `NEW_SCHED`).
> Needs nothing shipped beyond HEAD; its integration tests use raw jobs, not `File`.
> Read [`bytes.md`](bytes.md) §7 first. Anchors as of `a86b8bc`.

## 1. The one architectural move: natives never call `.ph`

`Future` is pure `.ph` (`core.ph` `class Future`: `_state`/`_waiters`,
`settleValue`/`settleError`, settle-once). The reactor therefore **never settles a
future from Rust**. Completions surface through a native *pop seam* and settlement is
ordinary `.ph` code in the pump — the same shape the scheduler already has
(`System.runScheduled`, `core.ph`, draining `System.nextScheduled`,
`primitive/system.rs:70`). This realizes reactor.md law 2 ("settlement only at the
drain, on the VM thread") with zero native re-entry into the interpreter.

```
worker/timer ──Completion(plain data)──▶ mpsc ──▶ [VM: pending buffer, filled at safepoint]
                                                        │ System.nextCompletion_   (native pop)
                                                        ▼
                              .ph pump: future.settleValue(v) / settleError(e)
                                        → waiters requeue via existing scheduler
```

## 2. File-by-file

### 2.1 `phalcom-core/src/reactor.rs` — new

```rust
/// Work shipped to a pool thread. PLAIN DATA ONLY (PDR-0003 §3 / PDR-0004 §4):
/// no Value, no ObjRef, no heap access — the boundary is structural.
pub enum Job { Test(TestJob) }                     // File(..) etc. arrive with U-FS
/// What comes back. Same rule.
pub enum Completion { pub token: Token, pub outcome: Result<Payload, IoErrorData> }
/// Plain result payloads (bytes read, count written, …).
pub enum Payload { Unit, Count(u64), Buffer(Vec<u8>), Text(String) }
```

- **`Token`**: generation-tagged (`index`+`generation` `u32` pair) — the
  `FrameToken`/resource-table discipline (`frame.rs:19`), own registry here.
- **Registration registry** `token -> ObjRef /* the pending Future instance */`.
  Unlike U-RESOURCE's table this one **holds `ObjRef`s and is a GC root**
  (reactor.md law 5): a parked fiber is reachable only through its future's `.ph`
  `_waiters` list, so rooting the future roots the fiber. Wire the registry into the
  collector's root enumeration next to the existing VM roots; the gc suite's
  fiber-reachability tests (`../../../../../phalcom-core/tests/gc.rs`) are the pattern, and a new
  M-row test asserts a registered future + parked fiber survive `System.gc` with no
  other reference.
- **Pool**: lazily spawned, bounded at a `const WORKERS: usize = 4` with a doc comment
  citing Q-R2 (measure before changing; never user-visible in v0.2). Workers loop on a
  job channel, push to the completion channel. Shutdown: drop the job sender, join with
  a bounded wait (reactor.md §8).
- **Timers**: a `BinaryHeap<Reverse<(Instant, Token)>>` on the VM — **no timer
  thread**. The park call (§2.3) computes `min(next deadline, caller timeout)` and uses
  `recv_timeout` on the completion channel; on wake it converts expired deadlines into
  `Completion`s locally. Monotonic by construction (`Instant`), reactor.md law 6.
- **Plain-data boundary, enforced:** a unit test asserting `Job: Send + 'static` and —
  the structural half — the types simply cannot embed `Value`/`ObjRef` (no such
  fields). Add the `const _: ()` assert-Send idiom; reactor.md §10's compile-time row.

### 2.2 Safepoint fill

`service_gc_safepoint` (`vm/dispatch.rs:540`) is the single existing back-edge hook;
extend that site (same function or a sibling called next to it) to `try_recv`-drain the
mpsc into a VM-side `pending_completions: VecDeque<Completion>` — bounded batch, no
blocking, no interpreter re-entry, no allocation into the Phalcom heap. Stale tokens
(generation mismatch — cancelled/superseded) are dropped **here** (law 7); live ones
stay queued for the pop seam.

### 2.3 Native seams — `primitive/system.rs`

| Rust fn | Binding | Behavior |
|---|---|---|
| `system_sleep` | `System.sleep_(_)` | validated integral-ms `Number` (reuse `expect_index`'s shape); allocate token, push `(Instant::now() + ms, token)` on the heap, register the future the `.ph` wrapper passes… **no** — see below: returns a fresh token `Number`; the `.ph` layer owns the future |
| `system_next_completion` | `System.nextCompletion_` | pop one pending completion; returns `None` when empty, else a 3-slot `Tuple` `(futureRef, payloadValue, isError)` minted **here, on the VM thread** — the only place plain data becomes `Value`s (PDR-0004 §4) |
| `system_park_for_completion` | `System.parkForCompletion_(_)` | with runnables absent (caller's responsibility): if **no** registration and no timer pending → return `None` immediately (the pump's exit signal — reactor.md §4's three-way conjunction, third clause); else block in `recv_timeout(min(next timer, arg ms))`, fill the pending buffer, return `true` |

**Registration split, precisely:** a native that *starts* an operation (here only
`sleep_`; U-FS's `open_`/`read_`/… later) allocates the token, submits the job or
timer, and must associate `token -> Future`. The future is created in `.ph` (it is a
`.ph` instance). So the pattern every IO native follows:
`.ph` creates `Future.new()`, calls the native with it as an argument, native registers
`token -> that ObjRef` and submits. `sleep_(ms, future)` — two args, then. Rule:
**every reactor-registering native takes the pending future as its last argument.**
(Amend the floor spelling to `System.sleep_(_,_)`.)

### 2.4 The `.ph` pump — core.ph

Extend the existing drain (`System.runScheduled`, `core.ph`; root-drive at exit +
root-fiber `await` both already loop it — C-FUT-2):

```phalcom
static runScheduled() {
  while (true) {
    // 1. existing: drain ready fibers via nextScheduled
    // 2. new: drain completions
    let c = System.nextCompletion_
    while (not (c == None)) {
      // c: (future, payload, isError)
      ... isError.ifTrue({ future.settleError(payload) }, ifFalse: { future.settleValue(payload) })
      c = System.nextCompletion_
    }
    // 3. new: nothing runnable and nothing drained -> park or exit
    //    parkForCompletion_(None-or-cap) returns None => truly idle => return
  }
}
```

Settlement runs here as ordinary `.ph`; `Future#drain` requeues waiters through the
existing scheduler — no new resume mechanism. `System.sleep(ms)` wrapper: validate
(raise on negative/fractional), `const f = Future.new()`, `System.sleep_(ms, f)`,
return `f`.

**Liveness (reactor.md §4, law 4) lands entirely in this loop's exit condition**: leave
only when no runnable fiber AND `parkForCompletion_` reports idle. reactor.md names
this the sharpest failure mode; its test is written FIRST (§4 below).

### 2.5 Census + registration

`NEW_REACTOR: usize = 3` (`137→148` is history; base whatever the live census reads —
verify against `floor_census_matches_installed_bindings`, never this document). No new
kernel class, so bytes.md §7 obligation 1 does not bite; obligation 2 (match-site
sweep) does not either — no new heap arm. GC-root wiring (§2.1) is this unit's
equivalent hazard: an unrooted registered future is a use-after-collect at settle time.

## 3. Ordering

1. `reactor.rs`: token registry + channels + pool + timer heap, pure-Rust tests.
2. Safepoint fill (§2.2) — behavior-neutral while nothing registers; full suite green.
3. GC root wiring + the M-row gc test.
4. Seams (§2.3) + census. Boot green.
5. `.ph` pump extension + `System.sleep` (§2.4).
6. **Liveness test first**, then the rest of §4. Clean-worktree verify.

## 4. Test plan

reactor.md §10's table, phase-1-adapted (no sockets, no `File` — raw `Job::Test`
variants stand in; the socket-echo row moves to the network unit):

| Check | Asserts |
|---|---|
| liveness (FIRST) | a program whose only work is `System.sleep(50).await` completes; exits cleanly after |
| exit exactness | with a timer outstanding the pump does not exit early; with nothing pending `parkForCompletion_` returns idle and the program ends |
| worker round-trip | a `Job::Test` (e.g. echo-payload) settles its future `Ok` with the payload; fiber resumes via the ready queue, never inline |
| settle-once / stale token | double completion for one token settles once; a deregistered token's completion is dropped at the safepoint fill |
| timer ordering | two sleeps settle in deadline order regardless of registration order; `sleep(0)` settles on the next pump round |
| monotonic posture | documented-only (wall-clock fakes are not std-reachable); the heap keys on `Instant` — assert by construction in a unit test comparing tokens after an artificial `Instant` offset |
| GC ⊗ parked fiber | registered future + parked fiber with no `.ph` reference survive forced `System.gc`, then settle and resume correctly |
| plain-data boundary | the compile-time Send/no-Value assertions (§2.1) |
| negative | `System.sleep(-1)` and `System.sleep(1.5)` raise; `sleep` on… (root-fiber await of a pending future is the C-FUT-2 drive path, positive not negative) |

## 5. What must NOT happen

- No `Value`/`ObjRef` in `Job`/`Completion`/worker code — structural, not reviewed-for.
- No native code calling `settleValue`/`settleError` or otherwise re-entering the
  interpreter from the drain path (§1's whole architecture).
- No always-settled stub futures for operations that CAN block (PDR-0004 §2) —
  `sleep` genuinely parks.
- No unbounded park: `parkForCompletion_` always has a timeout ceiling so a lost
  completion degrades to a spin with progress checks, not a hang.
- No `mio`/`tokio`/any new dependency (phase-1 ruling above).

## 6. Not in this unit — file as DEFERRED on landing

Pollable descriptors + the poller backend (network unit; Q-R3's abstraction decision
real only then), cancellation surface (Q-R4 — the token generations built here are its
substrate), fairness tuning beyond the bounded batch (Q-R1), isolate-scoped reactors
(PDR-0003 §2).
