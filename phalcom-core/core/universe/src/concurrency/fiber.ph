@native
class System is Object {

  @class @native print(_ value: Object) -> Unit

  @class @native new() -> Never

  @class @native schedule(_ fiber: Object) -> Fiber

  @class @native sleep(_ milliseconds: Int) -> Future<Unit>

  @class
  clock -> Clock {
    Clock._$system
  }

  @class @internal @native _$monotonicNanoseconds -> Int

  @class @internal @native _$nextScheduled -> Option<Fiber>

  @class @internal @native _$schedulerFailureCursor -> Int

  @class @internal @native _$takeUnhandledScheduledFailures(_ cursor: Int) -> Option<String>

  @class @internal @native _$reportUnhandledScheduledFailures -> Unit

  @class @internal @native _$wake(_ fiber: Fiber, _ generation: Int) -> Bool

  @class @native gc -> Unit

  @class @internal @native _$write(_ value: String) -> Unit

  @class @internal @native _$leakReport -> List<String>

  @class @internal @native _$strictResources(_ enabled: Bool) -> Unit

  // U-STRING write funnel (ADR-0049 amendment): pure `.ph` control flow over
  // native `write_(_)` and the `toString` message. Additive-only: does not
  // touch the native `print(_)` pathway (pre-existing divergence between
  // `Value::to_string` and the `.toString` message is out of scope).
  @class
  write(_ obj: Object) -> Object {
    System.writeObject(obj)
    obj
  }

  @private
  @class
  writeObject(_ obj: Object) -> Object {
    const s = obj.toString
    if s is String {
      System._$write(s)
    } else {
      System._$write("invalid toString")
    }
    obj
  }

  // U-SCHED: the `.ph`-callable counterpart to `VM::run`'s native
  // root-drive belt-and-suspenders pump (`vm/dispatch.rs`) — pumps
  // `System._$nextScheduled` to exhaustion, scheduler-resuming each queued
  // fiber (capture-not-propagate, so one scheduled task's uncaught raise
  // cannot abort another) — including any fiber a running scheduled fiber
  // itself schedules mid-drain, since `_$nextScheduled` is re-read every
  // iteration. Deliberately does **not** unwrap via `.match(some:none:)`
  // (which runs its arm through `Closure#call`'s native re-entrant
  // `run_until`, forbidding a fiber switch underneath, ADR-0030 §4):
  // `f._$resumeScheduled()`
  // must run at this method's own top level, not nested inside a block a
  // native primitive is driving, so the receiver is unwrapped via
  // `unwrapOr(_)` into a plain local first, and `_$resumeScheduled()` sent as its own
  // statement.
  @class
  runScheduled() -> Unit {
    let next = System._$nextScheduled
    while (next.isSome) {
      let f = next.unwrapOr(None)
      f._$resumeScheduled()
      next = System._$nextScheduled
    }
    System._$reportUnhandledScheduledFailures
    ()
  }
}

@native
class Fiber is Object {
  @class @native new(_ body: Function) -> Fiber

  @native call() -> Dynamic

  @native call(_ value: Dynamic) -> Dynamic

  @native try() -> Dynamic

  @native try(_ value: Dynamic) -> Dynamic

  @internal @native _$resumeScheduled() -> Dynamic

  @internal @native _$preparePark() -> Int

  @internal @native _$park(_ generation: Int) -> Dynamic

  @internal @native _$onComplete(_ observer: () -> Unit) -> Fiber

  @internal @native _$terminalValue -> Dynamic

  @class @native yield() -> Dynamic

  @class @native yield(_ value: Dynamic) -> Dynamic

  @class @native current -> Fiber

  @class @native abort(_ error: Error) -> Never

  @native isDone -> Bool

  @native isRoot -> Bool

  @native error -> Option<Error>
}

// Explicit readiness registration variant (`ParkedFiber` or `Callback`).
// Supports idempotent detachment, release of captures, and lifecycle tracking.
class FutureSubscription {
  @constructor
  fiber(_ id: Int, _ fiber: Fiber, _ generation: Int, _ onDetach: () -> Unit) {
    _id = id
    _kind = "fiber"
    _fiber = fiber
    _generation = generation
    _callback = None
    _onDetach = onDetach
    _state = "active"
  }

  @constructor
  callback(_ id: Int, _ callback: () -> Unit, _ onDetach: () -> Unit) {
    _id = id
    _kind = "callback"
    _fiber = None
    _generation = 0
    _callback = callback
    _onDetach = onDetach
    _state = "active"
  }

  id -> Int { _id }
  isActive -> Bool { _state == "active" }
  isDetached -> Bool { _state == "detached" }

  // Idempotently cancels this subscription. Releases references immediately.
  detach() -> Bool {
    if (_state == "active") {
      _state = "detached"
      _fiber = None
      _callback = None
      if (_onDetach != None) {
        const hook = _onDetach
        _onDetach = None
        hook.call()
      }
      return true
    }
    return false
  }

  // Delivers readiness notification: wakes parked fiber or runs callback.
  deliver() -> Unit {
    if (_state == "active") {
      _state = "delivered"
      if (_kind == "fiber") {
        const f = _fiber
        const gen = _generation
        _fiber = None
        _onDetach = None
        System._$wake(f, gen)
      } else {
        const cb = _callback
        _callback = None
        _onDetach = None
        if (cb != None) {
          cb.call()
        }
      }
    }
    ()
  }
}

// `Future` (concurrency.md §2; ADR-0030 §1): a settle-once state machine over
// a fulfilled/rejected result. A **plain `InstanceObject`** (concurrency.md §2
// "Implementation" ¶1) — zero new floor, with scheduler-owned waiting routed
// through private ticketed Fiber park/wake seams.
//
// Matching then/map/catch callbacks always run on observed scheduler Fibers,
// independent of whether the receiver was already settled at registration.
// Await of an already-settled Future is an immediate state read.
class Future<T> {
  // Builds a pending future (U-FUTURE Slice B).
  @constructor
  new() {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
  }

  // Builds an already-`fulfilled` future wrapping `v` (concurrency.md §2
  // `@constructor value(_)`). Goes through the pending→`settleValue` path
  // rather than setting `_outcome` directly so construction and
  // post-construction settlement share one settle-once code path.
  @constructor
  value(_ v: T) {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `@constructor error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  @constructor
  error(_ e: Error) {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`.
  isReady -> Bool { _outcome.isSome }

  // Internal single settlement transition. Returns `true` if this call settled the future,
  // or `false` if the future was already settled (settle-once).
  @internal
  _$trySettle(_ outcome: Result<T, Error>) -> Bool {
    if self.isReady {
      return false
    }
    _outcome = Some(outcome)
    const regs = _registrations
    _registrations = List.new()
    let i = 0
    const n = regs.size
    while (i < n) {
      const r = regs.at(i)
      r.deliver()
      i = i + 1
    }
    true
  }

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(_ v: T) -> Future<T> {
    const res: Result<T, Error> = Result::Ok(v)
    self._$trySettle(res)
    self
  }

  // Settles `self` as `rejected` with `e` (an `Error`), unless already
  // settled — the rejection sibling of `settleValue(_)`; see it for
  // the settle-once contract (C-FUT-3).
  settleError(_ e: Error) -> Future<T> {
    const res: Result<T, Error> = Result::Error(e)
    self._$trySettle(res)
    self
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`).
  value -> Option<T> {
    _outcome.match(
      some: |out| {
        out.match(
          ok: |v| { Some(v) },
          err: |_| { None }
        )
      },
      none: || { None }
    )
  }

  // The complete settlement outcome as an `Option<Result<T, Error>>`.
  outcome -> Option<Result<T, Error>> {
    _outcome
  }

  // Amortized compaction to discard detached registration tombstones.
  @private
  compactRegistrations() -> Unit {
    const compacted = List.new()
    let i = 0
    const n = _registrations.size
    while (i < n) {
      const r = _registrations.at(i)
      if r.isActive {
        compacted._$push(r)
      }
      i = i + 1
    }
    _registrations = compacted
    ()
  }

  // Subscribes a callback to readiness, returning a detachable `FutureSubscription`.
  subscribeReady(_ callback: () -> Unit) -> FutureSubscription {
    _nextRegId = _nextRegId + 1
    const reg = FutureSubscription.callback(
      _nextRegId,
      callback,
      || {
        _detachCount = _detachCount + 1
        if (_detachCount >= 16) {
          self.compactRegistrations()
          _detachCount = 0
        }
      }
    )
    if self.isReady {
      reg.deliver()
    } else {
      _registrations._$push(reg)
    }
    reg
  }

  // Backward compatibility helper for internal combinator wiring.
  @private
  whenReady(_ continuation: () -> Unit) -> FutureSubscription {
    self.subscribeReady(continuation)
  }

  // Suspends the current scheduler-owned fiber until settled (U-FUTURE Slice
  // B). A non-root await registers an exact `(Fiber, generation)` ticket and
  // performs an authorized Future park. The root fiber drives the scheduler
  // because it cannot park itself.
  await -> T {
    while (not self.isReady) {
      if (Fiber.current.isRoot) {
        // Pump until someone settles us. If the ready queue drains while we are
        // still pending, nothing can settle us and looping again would spin
        // forever in silence (E004(b)) — report it instead. Scheduler resume
        // isolates one scheduled task's uncaught raise from the others.
        const failure_cursor = System._$schedulerFailureCursor
        while (not self.isReady) {
          const next = System._$nextScheduled
          if (next.isNone) {
            if (self.isReady) {
              break
            }
            const failures = System._$takeUnhandledScheduledFailures(failure_cursor)
            if failures.isSome {
              const details = failures.unwrapOr("")
              return Error.new("await: the future is still pending and the scheduler is empty; nothing can currently settle it.\n" + details).raise()
            }
            return Error.new("await: the future is still pending and the scheduler is empty; nothing can settle it").raise()
          }
          const f = next.unwrapOr(None)
          f._$resumeScheduled()
        }
      } else {
        const current = Fiber.current
        const generation = current._$preparePark()
        _nextRegId = _nextRegId + 1
        const reg = FutureSubscription.fiber(
          _nextRegId,
          current,
          generation,
          || {
            _detachCount = _detachCount + 1
            if (_detachCount >= 16) {
              self.compactRegistrations()
              _detachCount = 0
            }
          }
        )
        _registrations._$push(reg)
        try {
          current._$park(generation)
        } catch e {
          reg.detach()
          e.raise()
        }
      }
    }
    _outcome.match(
      some: |out| {
        out.match(
          ok: |v| { v },
          err: |e| { e.raise() }
        )
      },
      none: || {
        Error.new("await: future settled without outcome").raise()
      }
    )
  }

  // Runs `action` to terminal completion and invokes the matching callback.
  // Nonterminal park/yield turns do not call either callback.
  @private
  @class
  runToTerminal<U>(_ action: () -> U, _ onSuccess: (U) -> Unit, _ onError: (Error) -> Unit) -> Fiber {
    const fiber = Fiber.new(action)
    fiber._$onComplete(|| {
      if fiber.error.isSome {
        onError.call(fiber.error.unwrapOr(None))
      } else {
        onSuccess.call(fiber._$terminalValue)
      }
    })
    System.schedule(fiber)
    fiber
  }

  @class
  async<U>(_ action: () -> U) -> Future<U> {
    const result: Future<U> = Future.new()
    Future.runToTerminal(
      action,
      |value| { result.settleValue(value); () },
      |error| { result.settleError(error); () }
    )
    result
  }

  // Value mapping preserves U exactly, including when U is itself a Future.
  map<U>(_ callback: (T) -> U) -> Future<U> {
    const result: Future<U> = Future.new()
    self.subscribeReady(|| {
      _outcome.match(
        some: |out| {
          out.match(
            ok: |v| {
              Future.runToTerminal(
                || { callback.call(v) },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
            },
            err: |e| {
              result.settleError(e)
              ()
            }
          )
        },
        none: || { () }
      )
    })
    result
  }

  // Chaining explicitly adopts the callback's Future; there is no type-based
  // runtime guess between U-as-data and Future<U>-as-computation.
  then<U>(_ callback: (T) -> Future<U>) -> Future<U> {
    const result: Future<U> = Future.new()
    self.subscribeReady(|| {
      _outcome.match(
        some: |out| {
          out.match(
            ok: |v| {
              Future.runToTerminal(
                || {
                  const next = callback.call(v)
                  if next == result { throw Error.new("Future callback cannot adopt its own result") }
                  next.await
                },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
            },
            err: |e| {
              result.settleError(e)
              ()
            }
          )
        },
        none: || { () }
      )
    })
    result
  }

  // A recovery value must inhabit the Future's payload type. Returning another
  // Future as data remains distinct from recovering with asynchronous work.
  catch(_ callback: (Error) -> T) -> Future<T> {
    const result: Future<T> = Future.new()
    self.subscribeReady(|| {
      _outcome.match(
        some: |out| {
          out.match(
            ok: |v| {
              result.settleValue(v)
              ()
            },
            err: |e| {
              Future.runToTerminal(
                || { callback.call(e) },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
            }
          )
        },
        none: || { () }
      )
    })
    result
  }

  recoverWith(_ callback: (Error) -> Future<T>) -> Future<T> {
    const result: Future<T> = Future.new()
    self.subscribeReady(|| {
      _outcome.match(
        some: |out| {
          out.match(
            ok: |v| {
              result.settleValue(v)
              ()
            },
            err: |e| {
              Future.runToTerminal(
                || {
                  const next = callback.call(e)
                  if next == result { throw Error.new("Future callback cannot adopt its own result") }
                  next.await
                },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
            }
          )
        },
        none: || { () }
      )
    })
    result
  }

  @class
  flatten<U>(_ nested: Future<Future<U>>) -> Future<U> {
    nested.then(|inner| { inner })
  }

  // Waits for all input futures to fulfill, or rejects with the first observed rejection.
  @class
  all<U>(_ inputs: List<Future<U>>) -> Future<List<U>> {
    const n = inputs.size
    if (n == 0) {
      return Future.value(List.new())
    }
    const result: Future<List<U>> = Future.new()
    let remaining = n
    const values = List.new()
    let idx = 0
    while (idx < n) {
      values._$push(None)
      idx = idx + 1
    }
    let settled = false
    const subs = List.new()

    let i = 0
    while (i < n) {
      if settled {
        break
      }
      const index = i
      const input = inputs.at(i)
      const sub = input.subscribeReady(|| {
        if settled {
          return ()
        }
        input.outcome.match(
          some: |out| {
            out.match(
              ok: |v| {
                if (not settled) {
                  values.at(index, put: v)
                  remaining = remaining - 1
                  if (remaining == 0) {
                    settled = true
                    result.settleValue(values)
                  }
                }
                ()
              },
              err: |e| {
                if (not settled) {
                  settled = true
                  let j = 0
                  while (j < subs.size) {
                    const s = subs.at(j)
                    s.detach()
                    j = j + 1
                  }
                  result.settleError(e)
                }
                ()
              }
            )
          },
          none: || { () }
        )
        ()
      })
      subs._$push(sub)
      if settled {
        sub.detach()
      }
      i = i + 1
    }

    result
  }

  // Waits for all input futures to settle, preserving order as List<Result<U, Error>>.
  @class
  allSettled<U>(_ inputs: List<Future<U>>) -> Future<List<Result<U, Error>>> {
    const n = inputs.size
    if (n == 0) {
      return Future.value(List.new())
    }
    const result: Future<List<Result<U, Error>>> = Future.new()
    let remaining = n
    const outcomes = List.new()
    let idx = 0
    while (idx < n) {
      outcomes._$push(None)
      idx = idx + 1
    }
    const subs = List.new()

    let i = 0
    while (i < n) {
      const index = i
      const input = inputs.at(i)
      const sub = input.subscribeReady(|| {
        input.outcome.match(
          some: |out| {
            outcomes.at(index, put: out)
            remaining = remaining - 1
            if (remaining == 0) {
              result.settleValue(outcomes)
            }
            ()
          },
          none: || { () }
        )
        ()
      })
      subs._$push(sub)
      i = i + 1
    }

    result
  }

  // Races input futures; first observed terminal outcome (fulfillment or rejection) wins.
  @class
  race<U>(_ inputs: List<Future<U>>) -> Future<U> {
    const n = inputs.size
    if (n == 0) {
      return Future.error(ArgumentError.new("Future.race: cannot race on an empty list"))
    }
    const result: Future<U> = Future.new()
    let settled = false
    const subs = List.new()

    let i = 0
    while (i < n) {
      if settled {
        break
      }
      const index = i
      const input = inputs.at(i)
      const sub = input.subscribeReady(|| {
        if settled {
          return ()
        }
        input.outcome.match(
          some: |out| {
            settled = true
            let j = 0
            while (j < subs.size) {
              const s = subs.at(j)
              s.detach()
              j = j + 1
            }
            out.match(
              ok: |v| {
                result.settleValue(v)
                ()
              },
              err: |e| {
                result.settleError(e)
                ()
              }
            )
          },
          none: || { () }
        )
        ()
      })
      subs._$push(sub)
      if settled {
        sub.detach()
      }
      i = i + 1
    }

    result
  }

  // Returns a future that rejects with TimeoutError if self does not settle within `milliseconds`.
  timeout(_ milliseconds: Int) -> Future<T> {
    if (milliseconds < 0) {
      throw ArgumentError.new("Future#timeout: milliseconds must be non-negative")
    }
    if self.isReady {
      return self
    }
    const result: Future<T> = Future.new()
    let settled = false
    let sourceSub = None
    let timerSub = None

    const timer = System.sleep(milliseconds)

    sourceSub = self.subscribeReady(|| {
      if (not settled) {
        settled = true
        if (timerSub != None) {
          timerSub.detach()
        }
        self.outcome.match(
          some: |out| {
            out.match(
              ok: |v| {
                result.settleValue(v)
                ()
              },
              err: |e| {
                result.settleError(e)
                ()
              }
            )
          },
          none: || { () }
        )
      }
      ()
    })

    if (not settled) {
      timerSub = timer.subscribeReady(|| {
        if (not settled) {
          settled = true
          if (sourceSub != None) {
            sourceSub.detach()
          }
          result.settleError(TimeoutError.new("Future timed out after " + milliseconds.toString + "ms"))
        }
        ()
      })
      if settled {
        if (timerSub != None) {
          timerSub.detach()
        }
      }
    }

    result
  }
}

// Producer capability wrapper providing isolated settlement authority over an internal `Future<T>`.
class CompletionSource<T> {
  @constructor
  new() {
    const f: Future<T> = Future.new()
    _future = f
  }

  future -> Future<T> {
    _future
  }

  tryResolve(_ value: T) -> Bool {
    const res: Result<T, Error> = Result::Ok(value)
    _future._$trySettle(res)
  }

  tryReject(_ error: Error) -> Bool {
    const res: Result<T, Error> = Result::Error(error)
    _future._$trySettle(res)
  }
}

// `Tracer` (decorators-dispatch-observability.md D-2, ratified 2026-07-13):
// the pluggable observability sink `@traced`'s `sink:` argument targets —
// duck-typed (`enter`/`exit`/`threw`), so any object answering this protocol
// drops in. `Tracer.stdout` is the shipped default, routing through
// `System.print` (Phalcom has no dedicated logging primitive, system.md).
// Ships standalone: `@traced` itself is Install/Dispatch/Runtime-tier
// decorator-mechanism work, not yet built (see PLAN-DECORATORS.md), so this
// class has no caller yet — the sink protocol is ready when it lands.
class Tracer {
  @class
  stdout -> Tracer { Tracer.new() }

  enter(_ name: Object, _ args: Object) -> Unit { System.print("-> " + name.toString + " " + args.toString) }
  exit(_ name: Object, _ result: Dynamic, _ elapsed: Object) -> Unit { System.print("<- " + name.toString + " = " + result.toString) }
  threw(_ name: Object, _ err: Error) -> Unit { System.print("!! " + name.toString + " threw " + err.toString) }
}

// `OffBehavior` (decorators-dispatch-observability.md D-3, ratified
// 2026-07-13): `@featureFlag`'s off-path — what a gated call does when its
// flag reads false. `applyTo(inv)` (invoked by the not-yet-built
// `@featureFlag` Runtime interceptor's `aroundSend` hook against a
// not-yet-defined `inv` envelope) is deliberately NOT implemented here —
// this class ships now as pure value semantics only; wiring it to a real
// interception envelope is Install/Dispatch/Runtime mechanism work.
class OffBehavior {
  @class
  raise -> OffBehavior { OffBehavior.new("raise", None) }
  @class
  fallback(_ sel: Symbol) -> OffBehavior { OffBehavior.new("fallback", Some(sel)) }
  @class
  skip(_ value: Dynamic) -> OffBehavior { OffBehavior.new("skip", Some(value)) }

  @constructor
  new(_ kind: String, _ payload: Dynamic) { _kind = kind; _payload = payload }

  kind -> String { _kind }
  payload -> Dynamic { _payload }
}

// `Backoff` (decorators-behavioral.md B-2, ratified 2026-07-13): `@retry`'s
// backoff strategy. `.none` is fully usable today — no suspension needed,
// matching `@retry`'s own default. `.fixed(ms)`/`.exponential(base:,max:)`
// compute pure delay policies with overflow-safe saturation. Suspending waits
// between attempts via `waitBefore` require `System.sleep(_)`.
class Backoff {
  @class
  none -> Backoff { Backoff.new("none", 0, 0) }

  @class
  fixed(_ ms: Int) -> Backoff {
    if (ms < 0) {
      throw ArgumentError.new("Backoff.fixed: delay must be non-negative")
    }
    Backoff.new("fixed", ms, ms)
  }

  @class
  fixed(ms: Int) -> Backoff {
    Backoff.fixed(ms)
  }

  @class
  exponential(_ base: Int, _ max: Int) -> Backoff {
    if (base < 0) {
      throw ArgumentError.new("Backoff.exponential: base must be non-negative")
    }
    if (max < base) {
      throw ArgumentError.new("Backoff.exponential: max must be greater than or equal to base")
    }
    Backoff.new("exponential", base, max)
  }

  @class
  exponential(base: Int, max: Int) -> Backoff {
    Backoff.exponential(base, max)
  }

  @constructor
  new(_ kind: String, _ a: Int, _ b: Int) {
    _kind = kind
    _a = a
    _b = b
  }

  kind -> String { _kind }

  // Pure delay calculation with overflow-safe saturation.
  delayFor(_ attempt: Int) -> Int {
    if (attempt < 0) {
      throw ArgumentError.new("Backoff.delayFor: attempt must be non-negative")
    }
    if (_kind == "none") {
      return 0
    }
    if (_kind == "fixed") {
      return _a
    }
    if (_kind == "exponential") {
      const base = _a
      const max = _b
      if (base == 0) {
        return 0
      }
      let delay = base
      let i = 0
      while (i < attempt and delay < max) {
        delay = delay * 2
        if (delay > max) {
          delay = max
        }
        i = i + 1
      }
      return delay
    }
    0
  }

  waitBefore(_ attempt: Int) -> Unit {
    const delay = self.delayFor(attempt)
    if (delay == 0) {
      return ()
    }
    System.sleep(delay).await
    ()
  }
}

export Fiber, Future, FutureSubscription, CompletionSource, Tracer, OffBehavior, Backoff
