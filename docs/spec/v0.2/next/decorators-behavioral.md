# Behavioral decorators — `@memoize`/`@lazy`/`@synchronized`/`@retry`

- Status: **Accepted** (ratified 2026-07-13 under [ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md);
  B-1/B-2 open questions resolved the same day).
- Date: 2026-07-13
- Depends on:
  [attribute-classes.md](attribute-classes.md) (the `Attribute` root, `@On(target…, tier:)`
  declaration, the `wrap(_)`/`finalizeLayout(_)` hook protocol, the Install/Layout
  state-scope table — this doc fills in four named members of that library) ·
  [decorators.md](decorators.md) (the five-tier axis, the fixed phase order, the
  `runtime` flag) ·
  [concurrency.md](../concurrency.md) (cooperative single-threaded fibers — the model
  that decides what `@synchronized` can and cannot mean)
- Related:
  [ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
  (per-receiver decorator state is Layout-confined — the rule that places `@lazy`
  and per-receiver `@memoize`) ·
  [ADR-0053](../../../adr/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
  (Runtime interceptor guard) ·
  [ADR-0057](../../../adr/0057-decorator-granularity-vs-proxy-granularity-split.md)
  (the decorator-vs-proxy granularity split — why `@retry` the decorator and
  `Retry` the proxy coexist) ·
  [proxy.md](proxy.md) (`Lazy`/`Retry` proxies — the *object*-granularity siblings
  of `@lazy`/`@retry`, distinguished below) ·
  [decorators-stdlib.md](decorators-stdlib.md) (the earlier scattered library sketch
  this doc supersedes for these four)

## Context

Four behavioral decorators are named in [decorators.md](decorators.md)'s tier table
and sketched, inconsistently, across [attribute-classes.md](attribute-classes.md)'s
worked examples and [decorators-stdlib.md](decorators-stdlib.md)'s library. They
share one theme — **wrap a method with a runtime policy** (a cache, a compute-once
gate, a critical section, a retry loop) — and they compose heavily with one
another, so they are specified together here at ratification depth.

The four sketches carry three unresolved problems this doc closes:

1. **Surface drift.** [decorators-stdlib.md](decorators-stdlib.md) uses the
   pre-[A-1](attribute-classes.md) surface (`@install class memoize { wrap(method) { … } }`,
   lower-case class names, `{ recv, args => … }` blocks). The ratified surface
   ([attribute-classes.md A-1](attribute-classes.md)) is `@On(Method, tier: Install)`
   on a capitalized `Attribute` subclass whose `wrap(m)` returns a
   `Method.fromBlock`. This doc uses the ratified surface throughout; the stdlib
   subsections are marked superseded.

2. **The `@synchronized` OS-lock assumption is wrong for Phalcom.** Both existing
   sketches reach for `Lock.new()` / `Mutex.new()` and a `critical`/`hold` block —
   an OS-mutex mental model. Phalcom's concurrency is **cooperative and
   single-threaded with no preemption**; message send is atomic and runs to
   completion, so there is *nothing to lock against* inside a suspension-free
   method ([concurrency.md §1](../concurrency.md): "no synchronization primitives are
   needed"). `@synchronized` is re-specified below as a **cooperative monitor that
   guards across `yield`/`await` suspension points**, not an OS lock.

3. **`@retry`/`@lazy` collide by name with the `Retry`/`Lazy` proxies**
   ([proxy.md](proxy.md)). Resolved per
   [ADR-0057](../../../adr/0057-decorator-granularity-vs-proxy-granularity-split.md):
   the decorator is method-granularity and author-applied; the proxy is
   whole-object and third-party-applied. The two do *not* merge — see each
   decorator's "Relationship to the proxy" note.

## Decision

### `@memoize` — Install, class-wide cache keyed by `(receiver, args)`

`@memoize` caches a method's result so repeated calls with the same key return the
cached value instead of recomputing. It is a pure Install-tier user decorator: the
cache lives in the attribute instance (created once at class-definition time,
shared by every receiver), which is the **per-method / per-class** state row of
[attribute-classes.md](attribute-classes.md)'s scope table — no reserved receiver
slot, so no Layout, so user-authorable.

```phalcom
@On(Method, tier: Install)
class Memoize extends Attribute {
  var _cache
  var _max                                     // None = unbounded; Some(n) = LRU bound
  construct new(max: None) { _cache = Map.new(); _max = max }

  wrap(m) {
    return Method.fromBlock { args =>
      let key = Pair.of(self, args)            // (receiver identity, args) — see key strategy
      return _cache.at(key).ifNone {
        let v = m.invokeOn(self, args)
        _cache.at(key, put: v)
        _max.ifSome { n => (_cache.size > n).ifTrue { _cache.evictOldest } }
        Some.new(v)
      }.unwrap
    }
  }
}
```

**Cache-key strategy — `(receiver, args)`, never `args` alone.** Keying on `args`
only shares one cached result across every instance of the decorated class, which
is silently wrong for any method whose result depends on receiver state — correct
only by accident for a pure function of its arguments (`Fib.fib`). The key is the
`(receiver-identity, args)` pair; receiver identity is object identity
(`ObjRef`), not structural `==`, so two distinct-but-equal receivers get distinct
cache lines (conservative; never returns another object's cached answer).

**Eviction — none by default; opt-in LRU via `@memoize(max: n)`.** The default is
**unbounded**, matching memoize's contract ("same key → same result, forever").
The unbounded cache retains every `(receiver, args)` pair it has ever seen for the
life of the attribute instance (= the life of the program), holding each receiver
*strongly* — the documented retention cost [ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
priced (a `@memoize`d method on a long-lived, high-cardinality receiver set is a
leak by construction, not a bug to silently fix). `@memoize(max: n)` bounds the
cache to `n` entries under **LRU** eviction, capping retention for exactly that
case. LRU (not size-in-bytes, not TTL) is chosen because it needs only recency
order — no value-size introspection, no clock dependency — and matches the
"recently-computed keys recur" access pattern memoize targets.

**Mutable-receiver interaction — caller's contract.** `@memoize` assumes the
method is **referentially transparent given `(receiver identity, args)`**: same
receiver, same args ⇒ same result, for the receiver's whole lifetime. If the
receiver mutates such that the method's result *would* change, the cache is
silently stale. This is the caller's responsibility, exactly as retry-safety is
(below) — the decorator states the contract; it cannot enforce it (Phalcom has no
purity analysis, the same floor-not-proof limit as the truthiness ban,
[ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)). Memoize a method only
when its result is a pure function of receiver identity + args.

**Fiber-safety.** In the cooperative model, a memoized method that runs to
completion without suspending is atomic — no two fibers can interleave, so the
"check-miss-then-compute-then-store" sequence is race-free for free. A memoized
method that *yields/awaits mid-computation* can be entered by a second fiber that
also misses and also computes (double compute; last store wins). `@memoize` does
**not** guard across suspension — memoizing a suspending method is a misuse.
Memoize synchronous, pure methods.

### `@lazy` — Layout (builtin), per-receiver compute-once slot

`@lazy` on a getter computes its value on first access and caches it *on the
receiver* for every later access. The cache is per-receiver, so it must live in a
reserved slot on the object — the **per-receiver** row of the scope table — which
crosses from user Install into **builtin Layout**
([attribute-classes.md](attribute-classes.md) already reserves `@lazy` here;
[ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
confirms per-receiver state is Layout-only). `tier: Layout` is compiler-reserved
([attribute-classes.md A-3](attribute-classes.md)); a user attempt to author
`@lazy` hits `attr.compile_tier_reserved`.

```phalcom
// BUILTIN — Layout is compiler-owned; shown for semantics, not as user source.
@On(Getter, tier: Layout)
class Lazy extends Attribute {
  finalizeLayout(field) { field.reserveSlot(#__lazy) }   // one reserved slot per receiver

  wrap(getter) {
    return Method.fromBlock {
      return self.slotAt(#__lazy).ifNone {
        let v = getter.invokeOn(self)          // if this throws, slot stays empty — see below
        self.setSlotAt(#__lazy, v)
        Some.new(v)
      }.unwrap
    }
  }
}
```

**Initializer-throws policy — retry-next-access, not cached-throw.** If the
initializer raises, the reserved slot stays empty (`None`) and the exception
propagates to the caller; the *next* access re-runs the initializer. Rationale: a
first-force that failed transiently (a not-yet-available resource) should be
retryable, not permanently poison the slot. The alternative — cache the throw so
every access re-raises the same error deterministically — trades retryability for
determinism; Phalcom favors transient-retry here, consistent with `@retry`'s own
transient-failure model. This is the one real fork; it is settled to
**retry-next-access**, recorded as such (not left open) because caching the throw
is recoverable-from by the user (wrap the initializer to memoize its own failure)
whereas the reverse is not.

**Fiber-safety of first-force.** Same rule as `@memoize`: a synchronous
initializer is atomic (cooperative model); an initializer that yields/awaits can
be double-forced by a second fiber (last `setSlotAt` wins; the losing computation
is discarded). A `@lazy` initializer **should be synchronous**. If it must
suspend, compose `@synchronizedClassWide` (below) to serialize the force, or accept
idempotent double-compute.

**Relationship to the `Lazy` proxy ([proxy.md](proxy.md)).** Genuinely different
mechanisms, not a collision to merge
([ADR-0057](../../../adr/0057-decorator-granularity-vs-proxy-granularity-split.md)):

| | `@lazy` (this doc) | `Lazy` proxy ([proxy.md](proxy.md)) |
|---|---|---|
| Defers | one **method result** | building a whole **object** |
| Granularity | one getter on a class the author owns | any target, wrapped from outside |
| Storage | reserved slot on the receiver | the proxy's own `_built` field |
| Applied by | the class author, at the declaration | a third party, by wrapping |

`@lazy subtotal` caches *the number*; `Lazy.from(thunk: openLedger)` defers *the
ledger's construction*. Both keep their names — one is a sigil (`@lazy`), one is a
class (`Lazy`) — with this table as the disambiguation.

### `@synchronized` — Layout (builtin), per-receiver cooperative reentrant monitor

Phalcom has **no preemption and no shared-memory data race**
([concurrency.md §1](../concurrency.md)); a running fiber runs until it explicitly
`yield`s, `await`s, returns, or raises, and message send is atomic. Therefore a
synchronized method **whose body never suspends needs no lock at all** — the
cooperative scheduler already guarantees it runs to completion without
interleaving. `@synchronized` has teeth in exactly one case: a method that
**suspends mid-body** (`await`s a future, `yield`s), at which point another fiber
could enter the same critical section on the same receiver before the first fiber
resumes. `@synchronized` guards *that* window with a **cooperative monitor**, not
an OS mutex (there is no OS thread to exclude).

The monitor is **per-receiver** (Java-`synchronized(this)` semantics — each object
serializes its own synchronized methods), so its state (owning fiber + reentrancy
depth) lives in a reserved slot on the receiver → **Layout, builtin**, same
placement as `@lazy` and per-[ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md).

```phalcom
// BUILTIN — Layout. Per-receiver cooperative monitor.
@On(Method, tier: Layout)
class Synchronized extends Attribute {
  finalizeLayout(method) { method.owner.reserveSlot(#__monitor) }   // one monitor per receiver

  wrap(m) {
    return Method.fromBlock { args =>
      let mon = self.slotAt(#__monitor).ifNone {
        let fresh = Monitor.new(); self.setSlotAt(#__monitor, fresh); Some.new(fresh)
      }.unwrap
      return mon.enter {                         // reentrant on the same fiber; suspends a second fiber
        m.invokeOn(self, args)
      }
    }
  }
}
```

- `Monitor` is a cooperative primitive (a fiber-queue + owner cell), **not**
  `Mutex`/`Lock`: `enter` on an unowned monitor claims it for the current fiber and
  runs the block; on entry by a **different** fiber it appends the caller to the
  monitor's wait-queue and `Fiber.yield`s to the scheduler, resuming when the owner
  releases; release is wired through the unwind primitive (`ensure`,
  [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)) so a throw out of
  the body still releases.
- **Reentrancy — yes.** The monitor records the owning fiber and a depth counter; a
  synchronized method calling *another* synchronized method **on the same receiver,
  on the same fiber** re-enters (depth++), never deadlocks — the deadlock the
  question raises cannot occur, because the owner is the same fiber and cooperative
  scheduling never preempts it. A synchronized call to a synchronized method on a
  *different* receiver takes that receiver's own monitor (independent), so
  cross-object collaboration serializes per object, not globally.

**Class-wide variant (`@synchronizedClassWide`, Install, user-authorable).** If you
genuinely want *every* receiver of a class to serialize on one shared monitor (rare
— usually a design smell), that state is per-*class* (the attribute instance), so it
is Install-tier and user-writable, matching
[attribute-classes.md](attribute-classes.md)'s existing `@synchronized` worked
example (which is the class-wide form under its old name):

```phalcom
@On(Method, tier: Install)
class SynchronizedClassWide extends Attribute {
  var _mon
  construct new() { _mon = Monitor.new() }     // ONE monitor, shared by all receivers
  wrap(m) { return Method.fromBlock { args => _mon.enter { m.invokeOn(self, args) } } }
}
```

The default `@synchronized` is the **per-receiver monitor**; `@synchronizedClassWide`
is the class-wide form. They are two decorators because a class declares at most one
tier ([A-1](attribute-classes.md)).

### `@retry` — Install, per-method, configurable backoff and error filter

`@retry(times:, on:, backoff:)` re-invokes a method on failure, up to `times`
attempts, retrying only errors matching `on:`, waiting per `backoff:` between
attempts. The retry counter is per-call (frame-local); the configuration lives in
the attribute instance (per-method). Both fit **Install** — no receiver slot, so
user-authorable; the stateless row of the scope table.

```phalcom
@On(Method, tier: Install)
class Retry extends Attribute {
  var _times
  var _on                                        // retryable Error type; default Error (all)
  var _backoff                                   // Backoff strategy; default Backoff.none
  construct new(times:, on: Error, backoff: Backoff.none) {
    _times = times; _on = on; _backoff = backoff
  }

  wrap(m) {
    return Method.fromBlock { args =>
      var attempt = 0
      while (true) {
        { return m.invokeOn(self, args) }.on(_on) { e =>     // success returns from the wrapper
          attempt = attempt + 1
          (attempt >= _times).ifTrue { throw e }             // exhausted → rethrow the last error
          _backoff.waitBefore(attempt)                       // yield-wait per strategy, then loop
        }
        // a non-`_on` error is not caught by `.on(_on)`, so it propagates immediately (never retried)
      }
    }
  }
}
```

- **Backoff — configurable, default `Backoff.none` (immediate).** `Backoff` is a
  small strategy object: `Backoff.none` (retry immediately), `Backoff.fixed(ms)`
  (constant delay), `Backoff.exponential(base:, max:)` (doubling, capped). Default
  is `none` to match the existing sketch and keep the zero-config case
  side-effect-free; production callers opt into `exponential` for network work.
  `waitBefore(attempt)` on the cooperative model is a **fiber-yielding** wait (it
  suspends to the scheduler, it does not busy-block a thread) — consistent with
  [concurrency.md](../concurrency.md).
- **Which errors retry — `on:` filter, default `Error` (all).** `on: TimeoutError`
  retries only that type (and subtypes, via `.on(_on)` = `Block#on`, U-ERR); every
  other error propagates on the first occurrence, never retried. Defaulting to all
  `Error`s matches the sketch but is deliberately the *broad* default: narrow it in
  real code.
- **Retry-safety is the caller's problem — stated explicitly.** `@retry` re-invokes
  the method verbatim; if the method has side effects (charged a card, sent a
  message, wrote a row), those effects happen once *per attempt*. `@retry` cannot
  know whether a method is idempotent — the author must guarantee it before
  decorating. This is identical in spirit to the `Retry` proxy's own inline caveat
  ("scope to a transient type in real code") and is the reason `@retry` is
  per-method (the author asserts *this* method is retry-safe), not per-object.

**Relationship to the `Retry` proxy ([proxy.md](proxy.md)) — resolved.** The
`Retry` proxy wraps a whole object and retries **every** method sent through it —
unsafe unless *every* method is idempotent, a guarantee that rarely holds for a
whole object. `@retry` is the **primary, recommended** surface precisely because
retry-safety is a per-method property: the author opts in method by method. The
`Retry` proxy is retained only for the black-box case (retrying a third-party object
you cannot annotate), with an explicit whole-object-idempotence caveat added to
[proxy.md](proxy.md). Per
[ADR-0057](../../../adr/0057-decorator-granularity-vs-proxy-granularity-split.md):
decorator = method granularity (author-applied), proxy = object granularity
(third-party-applied).

## Composition

The [decorators.md](decorators.md) phase order is total —
`generate → weave → finalize (Layout) → install (Install) → dispatch → runtime` — so
these four never fight; each acts in its own phase on the artifact the previous
handed it. Concrete rules for the pairs that co-occur:

- **`@synchronized` (Layout) ⊗ `@retry` (Install).** Layout finalizes before
  Install, so `@retry` wraps the already-monitor-guarded method. Each retry attempt
  therefore **re-enters the synchronized body**: acquire monitor → run → (on
  failure) release monitor → back off → re-acquire on the next attempt. The lock is
  *not* held across the backoff wait (release happens on the throw out of the body,
  before `@retry`'s `waitBefore`) — correct: you do not want to hold a receiver's
  monitor while a fiber sleeps between attempts. Written order `@synchronized @retry`
  and `@retry @synchronized` produce the same nesting here, because the tiers, not
  source order, fix Layout-inside-Install.
- **`@memoize` (Install) ⊗ `@retry` (Install)** — same tier, so **source order,
  innermost-last** ([decorators.md](decorators.md)). `@memoize @retry method`:
  memoize outermost — a cached hit skips retry entirely (retry only runs on a
  miss); a miss retries, and only a *successful* result is cached (a thrown,
  exhausted retry is never cached — the `wrap` stores only on the success path).
  `@retry @memoize method`: retry outermost — retry re-invokes the memoized wrapper,
  but a memoized *success* is cached so a second attempt short-circuits; only useful
  if the first attempt threw before caching. Prefer `@memoize @retry`
  (cache-hit-skips-retry) — document the choice at the call site.
- **`@memoize` (Install) ⊗ `@synchronized` (Layout).** Monitor-guarded method,
  memoize-wrapped on top. A cache hit **does not** enter the monitor (the memoize
  wrapper returns before invoking the inner method), so read-mostly memoized state
  is contention-free; only a miss enters the critical section to compute. This is
  the desirable ordering and is what the fixed tier order gives you.
- **`@lazy` (Layout, Getter-target) ⊗ everything.** `@lazy` is Getter-target only;
  a `@lazy` getter that must force under mutual exclusion uses
  `@synchronizedClassWide` on the enclosing computation, not `@synchronized` on the
  getter (which is Method-target).
- **With already-ratified `@invariant` ([annotations-contracts.md](../experimental/annotations-contracts.md)).**
  `@invariant` weaves at the Compile `weave` phase (before Layout/Install), so its
  entry/exit checks are *inside* every wrapper here. A `@memoize`d method under
  `@invariant`: the invariant is checked on the real invocation (a miss), not on a
  cache hit (the hit never enters the method body) — acceptable, since a hit returns
  a value already validated when it was computed. A `@synchronized` `@invariant`
  method checks the invariant inside the monitor (consistent state observed under
  exclusion) — the correct interaction.

## Hazards

- **`@synchronized` on a suspension-free method is a silent no-op.** It reserves a
  monitor slot and enters/exits it, but since the body cannot be interleaved anyway,
  the guard never actually excludes anyone — pure overhead. This is *correct* (it
  costs a slot + an uncontended enter/exit) but authors may over-apply it expecting
  thread-style protection the cooperative model already provides. Lint opportunity:
  warn on `@synchronized` methods with no `await`/`yield` in the body (and no call
  to a method that suspends) — deferred, noted.
- **`@memoize` retention leak (documented, not fixed).** The default unbounded cache
  holds every receiver it has computed for, strongly, for the program's life
  ([ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)).
  `@memoize(max: n)` is the escape. A golden test asserts the leak shape is
  *documented* behavior (the cache grows monotonically without `max:`), not a
  regression to silently patch — the fix, if ever, is weak keys, which Phalcom's
  non-moving mark-sweep collector ([ADR-0050](../../../adr/0050-non-moving-mark-sweep-collector.md))
  does not yet provide (ADR-0052's revisit trigger).
- **Backoff waits are cooperative yields, not blocks.** `@retry(backoff:)` on a
  method that runs on the root fiber will `Fiber.yield` during the wait, which
  requires the wait to sit in a suspension-legal position
  ([concurrency.md §Execution model](../concurrency.md): no native frame between the
  fiber entry and the yield). A backoff wait invoked from inside a native combinator
  (`each`) raises `CannotYieldAcrossNativeFrame`. Real backoff belongs on methods
  called from ordinary send positions; noted as an interaction with the
  restricted-yield rule, not a `@retry` bug.
- **`@lazy` double-force under suspension.** See the fiber-safety note above; a
  suspending initializer can compute twice. Not guarded by default; compose the
  class-wide monitor or keep the initializer synchronous.

## Test strategy

Golden `.ph` cases (positive stdout-exact unless noted), one lane per decorator plus
a composition lane:

- **`@memoize`**: (1) same-args repeat returns cached value, method body runs once
  (assert via a side-effect counter); (2) two distinct receivers cache
  independently (`(recv,args)` key — receiver A's cache never answers receiver B);
  (3) `@memoize(max: 2)` evicts LRU after a third distinct key; (4) a thrown
  computation is not cached (next call re-runs); (5) *negative-lane* — a stateful
  method whose result changes after receiver mutation returns the stale cached value
  (documents the caller-contract, not a bug fixture).
- **`@lazy`**: (1) initializer runs on first access only, across many reads; (2)
  per-receiver — two instances force independently; (3) initializer throws → next
  access re-runs (retry-next-access), then eventually succeeds; (4) reserved-slot
  golden — the `@lazy` builtin never retains a receiver-keyed side table (the
  ADR-0052 snapshot assertion, extended to `@lazy`).
- **`@synchronized`**: (1) suspension-free method — behaviorally identical
  with/without the decorator (no-op guard); (2) two fibers entering a *suspending*
  synchronized method on the **same** receiver serialize (second fiber's body starts
  only after the first fully exits — assert via interleaved print order); (3) two
  fibers on **different** receivers run concurrently (independent monitors); (4)
  reentrancy — a synchronized method calling another synchronized method on `self`
  on the same fiber completes without deadlock; (5) release-on-throw — an exception
  out of a synchronized body releases the monitor (a later fiber can enter).
- **`@retry`**: (1) succeeds on attempt `k < times` → method runs `k` times, returns;
  (2) exhausts `times` attempts → rethrows the last error; (3) `on: TimeoutError` —
  a non-`Timeout` error propagates on attempt 1 (never retried); (4)
  `backoff: Backoff.exponential(...)` — waits grow between attempts (assert via a
  fake clock); (5) side-effecting method runs its effect once per attempt (documents
  the retry-safety contract).
- **Composition lane**: (1) `@memoize @retry` — cache hit skips retry; miss retries
  then caches success; (2) `@synchronized @retry` — monitor released across the
  backoff wait, re-acquired per attempt; (3) `@memoize @synchronized` — cache hit
  does not enter the monitor; (4) `@invariant @synchronized` — invariant checked
  inside the monitor.

## What this precludes

- **A user-authorable per-receiver `@memoize` or `@synchronized`.** Per-receiver
  state is Layout/builtin ([ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md));
  the user-writable forms are class-wide (memoize's `(recv,args)`-keyed shared cache;
  the `SynchronizedClassWide` shared monitor). A leak-free per-receiver-lifetime
  memoize is a future Layout builtin, not shippable as user code — noted, not built.
- **OS-thread `@synchronized`.** There is no OS thread in Phalcom's model to
  exclude; `@synchronized` is a cooperative monitor over fibers. If Phalcom ever
  gains real OS threads (not planned — [concurrency.md](../concurrency.md) is
  single-threaded by ratified design), `@synchronized`'s semantics would be revisited
  in a superseding ADR; until then, "synchronized" means "mutually excluded across
  suspension points on one receiver," nothing more.
- **Retry of non-idempotent methods made safe.** `@retry` does not and cannot make a
  side-effecting method safe to re-run; it forecloses only the *illusion* that it
  does, by stating the contract at the decorator rather than hiding it.

## Open questions — resolved

| # | Decision |
|---|----------|
| B-1 | **(a) — class-wide Install cache only, `max:` LRU as the only bound.** No Layout-tier `perReceiver:` builtin variant ships in v0.2. Revisit alongside ADR-0052's weak-key GC revisit in v0.3 (see [DEFERRED.md](../../../forge/DEFERRED.md)) — do not build a second builtin before that story is decided. |
| B-2 | **(a) — `Backoff` is a ratified core class**, as specified above (`Backoff.none`/`.fixed(ms)`/`.exponential(base:,max:)`, `waitBefore(attempt)`). Chosen for the fake-clock test seam a raw block can't offer. |
