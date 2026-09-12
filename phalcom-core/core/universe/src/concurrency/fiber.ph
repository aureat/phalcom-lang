@native
class System is Object {

  @class @native print(_ value: Object) -> Unit

  @class @native new() -> Never

  @class @native schedule(_ fiber: Object) -> Fiber

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

// `Future` (concurrency.md §2; ADR-0030 §1): a settle-once state machine over
// a fulfilled/rejected result. A **plain `InstanceObject`** (concurrency.md §2
// "Implementation" ¶1) — zero new floor, with scheduler-owned waiting routed
// through private ticketed Fiber park/wake seams.
//
// Matching then/map/catch callbacks always run on observed scheduler Fibers,
// independent of whether the receiver was already settled at registration.
// Await of an already-settled Future is an immediate state read.
//
// State lives in three private fields (plan §6.1): `_state` (one of the
// strings `"pending"`, `"fulfilled"`, `"rejected"`), `_value` (the settled
// value or the captured `Error`), and `_waiters` — a `List` holding
// ticketed `Tuple`s registered by `await` and `Closure`s registered by
// `then`/`map`/`catch`. The tuple owns both the Fiber and the exact park
// generation that is allowed to wake it.
class Future<T> {
  // Builds a pending future (U-FUTURE Slice B).
  @constructor
  new() {
    _state = "pending"
    _value = None
    _waiters = List.new()
  }

  // Builds an already-`fulfilled` future wrapping `v` (concurrency.md §2
  // `@constructor value(_)`). Goes through the pending→`settleValue` path
  // rather than setting `_state`/`_value` directly so construction and
  // post-construction settlement share one settle-once code path.
  @constructor
  value(_ v: T) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `@constructor error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  @constructor
  error(_ e: Error) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`.
  isReady -> Bool { _state != "pending" }

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(_ v: T) -> Future<T> {
    if not self.isReady {
      _state = "fulfilled"
      _value = v
      self.drain()
    }

    self
  }

  // Settles `self` as `rejected` with `e` (an `Error`), unless already
  // settled — the rejection sibling of `settleValue(_)`; see it for
  // the settle-once contract (C-FUT-3).
  settleError(_ e: Error) -> Future<T> {
    if not self.isReady {
      _state = "rejected"
      _value = e
      self.drain()
    }

    self
  }

  // Wakes all waiters once settled. A `Tuple` is `(Fiber, generation)` and
  // must go through the ticket-aware wake authority; a closure is a
  // continuation work item and still uses ordinary scheduler admission.
  drain() -> Unit {
    _waiters.each |w| {
      if w is Tuple {
        System._$wake(w.at(0), w.at(1))
      } else {
        System.schedule(w)
      }
    }
    _waiters = List.new()

    return ()
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`).
  value -> Option<T> {
    if _state == "fulfilled" {
      return Some(_value)
    } else {
      return None
    }
  }

  // Suspends the current scheduler-owned fiber until settled (U-FUTURE Slice
  // B). A non-root await registers an exact `(Fiber, generation)` ticket and
  // performs an authorized Future park. The root fiber drives the scheduler
  // because it cannot park itself.
  //
  // The branch is chosen by **asking** (`Fiber#isRoot`), not by attempting a
  // suspension and inspecting the failure. The old yield probe ran through
  // nested native re-entry and could never distinguish a supported park from
  // a forbidden switch.
  //
  // Park preparation checks the native boundary before registration, so a
  // refusal cannot leave an actionable stale waiter behind. Wake is only
  // permission to run again; the loop rechecks readiness after every wake.
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
        _waiters._$push(Tuple._$fromList([current, generation]))
        current._$park(generation)
      }
    }
    if (_state == "rejected") {
      return _value.raise()
    }
    return _value
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

  // Subscribe to source readiness; this continuation only admits user work.
  @private
  whenReady(_ continuation: () -> Unit) -> Unit {
    if self.isReady {
      continuation.call()
    } else {
      _waiters._$push(continuation)
    }
    ()
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
    self.whenReady(|| {
      if _state == "fulfilled" {
        Future.runToTerminal(
          || { callback.call(_value) },
          |value| { result.settleValue(value); () },
          |error| { result.settleError(error); () }
        )
      } else {
        result.settleError(_value)
      }
      ()
    })
    result
  }

  // Chaining explicitly adopts the callback's Future; there is no type-based
  // runtime guess between U-as-data and Future<U>-as-computation.
  then<U>(_ callback: (T) -> Future<U>) -> Future<U> {
    const result: Future<U> = Future.new()
    self.whenReady(|| {
      if _state == "fulfilled" {
        Future.runToTerminal(
          || {
            const next = callback.call(_value)
            if next == result { throw Error.new("Future callback cannot adopt its own result") }
            next.await
          },
          |value| { result.settleValue(value); () },
          |error| { result.settleError(error); () }
        )
      } else {
        result.settleError(_value)
      }
      ()
    })
    result
  }

  // A recovery value must inhabit the Future's payload type. Returning another
  // Future as data remains distinct from recovering with asynchronous work.
  catch(_ callback: (Error) -> T) -> Future<T> {
    const result: Future<T> = Future.new()
    self.whenReady(|| {
      if _state == "rejected" {
        Future.runToTerminal(
          || { callback.call(_value) },
          |value| { result.settleValue(value); () },
          |error| { result.settleError(error); () }
        )
      } else {
        result.settleValue(_value)
      }
      ()
    })
    result
  }

  recoverWith(_ callback: (Error) -> Future<T>) -> Future<T> {
    const result: Future<T> = Future.new()
    self.whenReady(|| {
      if _state == "rejected" {
        Future.runToTerminal(
          || {
            const next = callback.call(_value)
            if next == result { throw Error.new("Future callback cannot adopt its own result") }
            next.await
          },
          |value| { result.settleValue(value); () },
          |error| { result.settleError(error); () }
        )
      } else {
        result.settleValue(_value)
      }
      ()
    })
    result
  }

  @class
  flatten<U>(_ nested: Future<Future<U>>) -> Future<U> {
    nested.then(|inner| { inner })
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
// need a real suspending wait between attempts, which needs `System.sleep(_)`
// — explicitly **not landed** (system.md: "still open", gated on a
// timer-completion-source follow-on unit, itself gated on U-SCHED's ready-
// queue/timer split per open-questions.md §15). Rather than silently busy-
// waiting or lying about elapsed time, `.fixed`/`.exponential`'s
// `waitBefore` raises until that primitive exists — a real gap, not a stub
// pretending to work.
class Backoff {
  @class
  none -> Backoff { Backoff.new("none", 0, 0) }
  @class
  fixed(_ ms: Int) -> Backoff { Backoff.new("fixed", ms, 0) }
  @class
  exponential(base: Int, max: Int) -> Backoff { Backoff.new("exponential", base, max) }

  @constructor
  new(_ kind: String, _ a: Int, _ b: Int) { _kind = kind; _a = a; _b = b }

  waitBefore(_ attempt: Int) -> Option<Never> {
    if (_kind == "none") {
      return None
    } else {
      return Error.new("Backoff." + _kind + " needs System.sleep(_), not yet landed (system.md)").raise()
    }
  }
}

export Fiber, Future, Tracer, OffBehavior, Backoff
