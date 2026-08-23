@native
class System is Object {
  // U-STRING write funnel (ADR-0049 amendment): pure `.ph` control flow over
  // native `write_(_)` and the `toString` message. Additive-only: does not
  // touch the native `print(_)` pathway (pre-existing divergence between
  // `Value::to_string` and the `.toString` message is out of scope).
  @class
  write(_ obj) {
    System.writeObject(obj)
    return obj
  }

  @private
  @class
  writeObject(_ obj) {
    const s = obj.toString
    (s.is(String)).ifTrue(|| {
      System._$write(s)
    }, ifFalse: || {
      System._$write("invalid toString")
    })
    return obj
  }

  // U-SCHED: the `.ph`-callable counterpart to `VM::run`'s native
  // root-drive belt-and-suspenders pump (`vm/dispatch.rs`) — pumps
  // `System.nextScheduled` to exhaustion, `try()`-resuming each queued
  // fiber (capture-not-propagate, so one scheduled task's uncaught raise
  // cannot abort another) — including any fiber a running scheduled fiber
  // itself schedules mid-drain, since `nextScheduled` is re-read every
  // iteration. Deliberately does **not** unwrap via `.match(some:none:)`
  // (which runs its arm through `Closure#call`'s native re-entrant
  // `run_until`, forbidding a fiber switch underneath, ADR-0030 §4): `f.try()`
  // must run at this method's own top level, not nested inside a block a
  // native primitive is driving, so the receiver is unwrapped via
  // `unwrapOr(_)` into a plain local first, and `try()` sent as its own
  // statement.
  @class
  runScheduled() {
    let next = System.nextScheduled
    while (next.isSome) {
      let f = next.unwrapOr(None)
      f.try()
      next = System.nextScheduled
    }
  }
}

@native
class Fiber is Object {}

// `Future` (concurrency.md §2; ADR-0030 §1): a settle-once state machine over
// a fulfilled/rejected result. A **plain `InstanceObject`** (concurrency.md §2
// "Implementation" ¶1) — zero new floor, written entirely in Phalcom over the
// same two public seams user code has, `System.schedule(_)` and `Fiber.yield`.
//
// **Both slices are landed.** Slice A is the scheduler-free half:
// `value(_)`/`error(_)` construct an already-settled future, `isReady`/`value`
// read it, and `then`/`map`/`catch` fire synchronously on an already-settled
// receiver. Slice B added `async(_)`, `await`, and the pending→settle `drain`
// over the native ready queue (`06432bd`, 2026-07-14). Note that this comment
// previously still described Slice B as "deliberately NOT built" long after it
// shipped, eleven lines above its own implementation — see
// `docs/learn/concurrency/future-await.md`.
//
// State lives in three private fields (plan §6.1): `_state` (one of the
// strings `"pending"`, `"fulfilled"`, `"rejected"`), `_value` (the settled
// value or the captured `Error`), and `_waiters` — a `List` holding two kinds
// of thing, `Fiber`s registered by `await` and `Closure`s registered by
// `then`/`map`/`catch`, unified by `System.schedule(_)` accepting both.
class Future {
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
  value(_ v) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `@constructor error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  @constructor
  error(_ e) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`.
  isReady { _state != "pending" }

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(_ v) {
    if (self.isReady) {
      return self
    } else {
      _state = "fulfilled"
      _value = v
      self.drain()
      return self
    }
  }

  // Settles `self` as `rejected` with `e` (an `Error`), unless already
  // settled — the rejection sibling of `settleValue(_)`; see it for
  // the settle-once contract (C-FUT-3).
  settleError(_ e) {
    if (self.isReady) {
      return self
    } else {
      _state = "rejected"
      _value = e
      self.drain()
      return self
    }
  }

  // Reschedules all waiters once settled.
  //
  // A waiter is either a `Fiber` (registered by `await`) or a `Closure`
  // (registered by `then`/`map`/`catch`); `System.schedule(_)` accepts both,
  // enqueueing a fiber as-is and wrapping anything else. A fiber waiter can,
  // however, be *finished* by the time we settle — it may have failed after
  // registering (E004(c): a caller that `await`s from under a native frame
  // raises out of `await` with its registration still in the list). Resuming a
  // finished fiber aborts the whole run, taking every other waiter on this
  // future down with it, so skip those rather than scheduling a corpse.
  drain() {
    _waiters.each |w| {
      const dead = w.is(Fiber) and w.isDone
      if (not dead) {
        System.schedule(w)
      }
    }
    _waiters = List.new()
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`).
  value { (_state == "fulfilled").ifTrue(|| { Some(_value) }, ifFalse: || { None }) }

  // Suspends the current fiber until settled (U-FUTURE Slice B). On the root
  // fiber — which has no resumer and so cannot yield — degrades to driving the
  // scheduler here instead.
  //
  // The branch is chosen by **asking** (`Fiber#isRoot`), not by attempting a
  // yield and inspecting the failure. It used to do the latter, via
  // `{ Fiber.yield(None) }.attempt()`, and that could never work: `.attempt()`
  // is two nested native re-entrant frames (`block_on` + `block_call`), so the
  // probe tripped the restricted-yield guard (ADR-0030 §4) it was probing for,
  // unconditionally, for every fiber. `await` therefore never suspended anyone
  // — E004. Attempt-and-inspect cannot work when the attempt changes the answer.
  //
  // Consequently the `Fiber.yield` below is **bare**. Wrapping it in anything
  // that reaches `Closure#call` — `.attempt()`, `.on(_)`, `ensure` — puts a native
  // frame between the fiber floor and the switch and reinstates the bug. A
  // `CannotYieldAcrossNativeFrame` raised from here is now a real one: it means
  // the *caller* invoked `await` from inside a block a native primitive is
  // driving, which is genuinely unsupported and correctly propagates.
  await {
    if (not self.isReady) {
      if (Fiber.current.isRoot) {
        // Pump until someone settles us. If the ready queue drains while we are
        // still pending, nothing can settle us and looping again would spin
        // forever in silence (E004(b)) — report it instead. `try()`-resume, so
        // one scheduled task's uncaught raise cannot abort the others, and as
        // its own statement rather than inside a block, for the same reason
        // `System.runScheduled` is written that way.
        while (not self.isReady) {
          const next = System.nextScheduled
          if (next.isNone) {
            return Error.new("await: the future is still pending and the scheduler is empty; nothing can settle it").raise()
          }
          const f = next.unwrapOr(None)
          f.try()
        }
      } else {
        _waiters._$push(Fiber.current)
        Fiber.yield(None)
      }
    }
    if (_state == "rejected") {
      return _value.raise()
    }
    return _value
  }

  // Runs `action` on a fresh fiber and settles the returned future with its
  // result (or captured error if it fails).
  @class
  async(_ action) {
    const f = Future.new()
    const driver = Fiber.new || {
      const fib = Fiber.new(action)
      const res = fib.try()
      if (fib.error.isSome) {
        f.settleError(fib.error.unwrapOr(None))
      } else {
        f.settleValue(res)
      }
    }
    System.schedule(driver)
    return f
  }

  // Normalizes a continuation result into a single Future layer. A callback
  // returning a Future is adopted; a plain value becomes an already-fulfilled
  // Future. This is the Future assimilation rule used by then/map/catch.
  @class
  flatten(_ value) {
    if (value.is(Future)) {
      return value
    }
    return Future.value(value)
  }

  // Registers a continuation on the settled/fulfilled path (concurrency.md
  // §2 `then(_)`). If pending, registers a continuation that will settle
  // the returned future when this receiver settles.
  then(_ f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "fulfilled") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // `then(_)` restricted to the fulfilled path (concurrency.md §2 `map(_)`).
  map(_ f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "fulfilled") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // Registers an error handler on the rejected path (concurrency.md §2
  // `catch(_)`).
  catch(_ f) {
    if (self.isReady) {
      if (_state == "rejected") {
        return Future.flatten(f.call(_value))
      } else {
        return self
      }
    } else {
      const f_next = Future.new()
      _waiters._$push(|| {
        if (_state == "rejected") {
          const fib = Fiber.new(|| { f.call(_value) })
          const res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            const flattened = Future.flatten(res)
            flattened.then |value| { f_next.settleValue(value) }
            flattened.catch |error| { f_next.settleError(error) }
          }
        } else {
          f_next.settleValue(_value)
        }
      })
      return f_next
    }
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
  stdout { Tracer.new() }

  enter(_ name, _ args) { System.print("-> " + name.toString + " " + args.toString) }
  exit(_ name, _ result, _ elapsed) { System.print("<- " + name.toString + " = " + result.toString) }
  threw(_ name, _ err) { System.print("!! " + name.toString + " threw " + err.toString) }
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
  raise { OffBehavior.new("raise", None) }
  @class
  fallback(_ sel) { OffBehavior.new("fallback", Some(sel)) }
  @class
  skip(_ value) { OffBehavior.new("skip", Some(value)) }

  @constructor
  new(_ kind, _ payload) { _kind = kind; _payload = payload }

  kind { _kind }
  payload { _payload }
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
  none { Backoff.new("none", 0, 0) }
  @class
  fixed(_ ms) { Backoff.new("fixed", ms, 0) }
  @class
  exponential(base, max) { Backoff.new("exponential", base, max) }

  @constructor
  new(_ kind, _ a, _ b) { _kind = kind; _a = a; _b = b }

  waitBefore(_ attempt) {
    if (_kind == "none") {
      return None
    } else {
      return Error.new("Backoff." + _kind + " needs System.sleep(_), not yet landed (system.md)").raise()
    }
  }
}
