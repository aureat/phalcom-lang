# Implementation spec — `Future#cancel` (U-CANCEL)

> **Status:** **blocked on ratification** of
> [PDR-0017](../../../pdr/0017-future-cancel-is-renunciation.md) (Proposed —
> rule 5). Dispatch-ready otherwise. Surface contract
> [`../stdlib/cancellation.md`](../../spec/current/stdlib/cancellation.md); substrate contract
> [`../stdlib/reactor.md`](../../spec/current/stdlib/reactor.md) §3/§7.
> **Needs shipped: U-REACTOR** (phase 1 — registry, tokens, pump). Independent of and
> **parallel-safe with U-NET** (disjoint files except `reactor.rs`, where this unit adds
> one method and one map; coordinate if truly concurrent). Timers are the test
> substrate — no sockets needed.
> **Floor delta: +1** — `System.cancelRegistration_(_)` (`NEW_CANCEL`). Census against
> the live `floor_census_matches_installed_bindings` under PDR-0012 ruling 21.
> No new kernel class ([`bytes.md`](bytes.md) §7 obligation 1 does not bite:
> `CancelledError` is pure `.ph`); no new heap arm (obligation 2 idle).

## 1. Shape

One Rust method on the reactor (release), one native seam (the `.ph` bridge), a `.ph`
`cancel` that settles through the existing `settleError`, and a tombstone set so pool
workers skip cancelled-but-queued jobs. Settlement never moves to Rust; the
[`reactor.md`](reactor.md) §1 architecture gains zero exceptions.

## 2. File-by-file

### 2.1 `reactor.rs` — release + reverse map + tombstones

- **Reverse map** `future_tokens: HashMap<ObjRef, Token>` maintained at registration
  and every release path (drain-settle, close-with-pending, cancel). Keying on `ObjRef`
  identity is sound because the collector never moves objects (ADR-0050); the map holds
  the same `ObjRef` the registry roots, adding no root of its own.
- **`cancel_registration(&mut self, fut: ObjRef) -> bool`**: look up; on miss return
  `false`. On hit: bump the entry's generation, remove the registry entry (**the GC
  root and the pump's pending count shrink here** — reactor.md §4's exit conjunction
  must see the decrement immediately, or a cancelled-last-op program never exits),
  remove the reverse-map row, insert the token's bits into the tombstone set, and — if
  the U-NET poller exists — deregister interest (guard on the op's presence; phase-1
  timer entries need nothing: the heap entry outlives the registry row and drops at
  expiry by generation check, the existing law-7 path).
- **Tombstones**: `Arc<Mutex<HashSet<u64>>>` shared with pool workers (token bits are
  plain data — the §2 boundary is untouched). A worker checks-and-removes its job's
  token at **dequeue**: tombstoned ⇒ skip the job entirely, push no completion
  (nothing waits; the registration is already gone). A job already running when the
  tombstone lands completes normally and its completion drops at the fill (stale
  generation) — the best-effort/never split of the surface §1 table, mechanically.
  Bound the set: entries are removed on hit, and the drain sweeps tombstones whose
  generation is stale anyway (no unbounded growth from never-dequeued jobs at
  shutdown — §8's drain clears the queue).

### 2.2 `primitive/system.rs` — the one native

| Rust fn | Binding | Behavior |
|---|---|---|
| `system_cancel_registration` | `System.cancelRegistration_(_)` | any `Value` accepted; non-object or reverse-map miss ⇒ `false` (a plain `Future.new()` has no registration and that is not an error — surface §3); hit ⇒ full release per §2.1, `true`. Never settles, never re-enters the interpreter |

### 2.3 core.ph — `Future` additions + `CancelledError`

- `_cancelled = false` joins the three constructors' init (fourth field; `.ph` class,
  recompiled at boot — no frozen-offset hazard).
- `cancel()` exactly as [`../stdlib/cancellation.md`](../../spec/current/stdlib/cancellation.md) §3 —
  release **before** `settleError`, flag after; order is normative there, copy it.
- `isCancelled => _cancelled`.
- `class CancelledError is Error` — pure `.ph`, message names the wait it ended;
  gains `kind: #cancelled` when T3/T6 land, nothing migrates (the U-RESOURCE §2.4
  posture).
- Rustdoc/comment sweep: `settleValue`'s settle-once comment gains one line naming the
  new client ("late settles after `cancel` land here by design — PDR-0017 ruling 4");
  do **not** otherwise touch the settle path.

### 2.4 Census

`NEW_CANCEL: usize = 1`. No class rows (nothing bootstrapped). Verify live baseline
first — U-REACTOR's +3 precedes; U-NET/U-FS/PDR-0012 may or may not have landed.

## 3. Ordering

1. `reactor.rs`: reverse map + `cancel_registration` + tombstones; pure-Rust tests
   (release decrements pending count; tombstoned job skipped; running job's completion
   dropped stale).
2. Native + census. Boot green.
3. core.ph additions.
4. Golden lanes (§4). Clean-worktree verify at the SHA.

## 4. Test plan

[`../stdlib/cancellation.md`](../../spec/current/stdlib/cancellation.md) §6's harness row for row in
`cancel/` + `cancel/negative/` lanes (phase-1 substrate: `System.sleep` + `Job::Test`),
plus unit-owned rows:

| Check | Asserts |
|---|---|
| exit promptness (**first**) | the §6 "cancel pending sleep" row — its failure mode is the pump exit-count desync, this unit's sharpest hazard |
| reverse-map hygiene | after settle-by-completion (not cancel), the reverse row is gone too — `cancelRegistration_` on a normally-settled future returns `false`, and the map never grows monotonically (assert size in a Rust test across 100 settle cycles) |
| tombstone bound | cancel 100 queued `Job::Test`s: workers skip all, set drains to empty |
| release-before-settle order | a waiter resumed by the cancel settlement observes `cancelRegistration_(f) == false` (already released) — the §3 ordering, made observable |
| plain-data boundary | tombstone set carries `u64` only; joins the compile-time no-`Value` assertions |

## 5. What must NOT happen

- No settlement from Rust — `cancel_registration` releases only; `settleError` runs in
  `.ph` (`cancel`), full stop.
- No interruption machinery: no worker signaling, no thread kill, no syscall abort.
  The tombstone is checked at dequeue and nowhere later.
- No touch of `settleValue`/`settleError` bodies (C-FUT-3 is load-bearing as shipped;
  PDR-0017 ruling 4 depends on it *unchanged*).
- No propagation: `cancel` reaches exactly one future (surface §5 law 5); no chain
  walking, no fiber cancellation.
- No second bookkeeping mechanism: one reverse map, one tombstone set, both inside
  `reactor.rs`.

## 6. Not in this unit — file as DEFERRED on landing

Propagation / structured concurrency (Q-C1), `Fiber` cancellation (Q-C2), timeout
sugar (Q-C3 — pure `.ph`, wants Q-N5's consumers first), `#closed`-path reuse audit
(U-NET's close-with-pending shares the release spine; once both land, factor the
common release into one method if the diff is mechanical).
