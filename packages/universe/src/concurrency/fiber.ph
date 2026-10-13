//! Concurrency primitives, asynchronous computation (`Future`), fibers, and scheduling.
//!
//! Provides the core building blocks for concurrency in Phalcom:
//! - `System`: System-level operations, scheduling, time, and fiber dispatch.
//! - `Fiber`: Lightweight cooperative coroutines and execution contexts.
//! - `Future<T>`: Settle-once asynchronous computation handles.
//! - `FutureSubscription`: Lifecycle handle for readiness observers on futures.
//!
/// Low-level system interface for scheduling, I/O, garbage collection, and diagnostics.
native class System is Object {

  /// Prints the string representation of an object to the standard output.
  ///
  /// @param value Object to print.
  @class
  native print(_ value: Object) -> Unit

  /// Prevents direct instantiation of `System`.
  @class
  native new() -> Dynamic

  /// Enqueues a fiber for execution in the cooperative scheduler.
  ///
  /// @param fiber Target fiber to schedule.
  /// @returns The scheduled fiber.
  @class
  native primitive schedule(_ fiber: Object) -> Fiber

  /// Suspends the current execution context asynchronously for the given duration.
  ///
  /// @param duration Time duration to sleep.
  /// @returns A `Future` that fulfills with `Unit` when the duration elapses.
  @class
  sleep(_ duration: Duration) -> Future<Unit> {
    sleepRaw(duration.nanoseconds)
  }

  @class
  internal native primitive sleepRaw(_ nanoseconds: Int) -> Future<Unit>

  @class
  internal native primitive nextScheduled -> Option<Fiber>

  @class
  internal native schedulerFailureCursor -> Int

  @class
  internal native takeUnhandledScheduledFailures(_ cursor: Int) -> Option<String>

  @class
  internal native reportUnhandledScheduledFailures -> Unit

  @class
  internal native primitive wake(_ fiber: Fiber, _ generation: Int) -> Bool

  /// Triggers a garbage collection cycle.
  @class
  native primitive gc -> Unit

  @class
  internal native primitive rawWrite(_ value: String) -> Unit

  @class
  internal native leakReport -> List<String>

  @class
  internal native strictResources(_ enabled: Bool) -> Unit

  /// Writes the string representation of an object to the standard output without a newline.
  ///
  /// @param obj Object to write.
  /// @returns The input object.
  @class
  write(_ obj: Object) -> Object {
    writeObject(obj)
    obj
  }

  @class
  private writeObject(_ obj: Object) -> Object {
    let s = obj.toString
    if s is String {
      rawWrite(s)
    } else {
      rawWrite("invalid toString")
    }
    obj
  }

  /// Pumps all currently scheduled fibers to completion in the current thread.
  ///
  /// Iterates through queued fibers until the run queue is exhausted and reports
  /// any unhandled failures.
  @class
  runScheduled() -> Unit {
    let mut next = nextScheduled
    while next.isSome {
      let mut f = next.unwrapOr(None)
      f.resumeScheduled()
      next = nextScheduled
    }
    reportUnhandledScheduledFailures
    ()
  }
}

/// Lightweight cooperative execution context (coroutine / fiber).
///
/// Fibers encapsulate a stack, execution state, and completion lifecycle.
native primitive class Fiber is Object {

  /// Creates a new fiber wrapping the given callable entry point.
  ///
  /// @param body Function or closure to execute.
  @class
  native primitive new(_ body: Function) -> Fiber

  /// Resumes or starts this fiber without arguments.
  native primitive call() -> Dynamic

  /// Resumes or starts this fiber, passing a value to the yield point.
  native primitive call(_ value: Dynamic) -> Dynamic

  /// Resumes this fiber in protected mode, capturing runtime errors.
  native try() -> Dynamic

  /// Resumes this fiber with an argument in protected mode.
  native try(_ value: Dynamic) -> Dynamic

  internal native primitive resumeScheduled() -> Dynamic

  internal native primitive preparePark() -> Int

  internal native primitive park(_ generation: Int) -> Dynamic

  internal native onComplete(_ observer: () -> Unit) -> Fiber

  internal native terminalValue -> Dynamic

  /// Yields execution from the currently active fiber back to its caller or scheduler.
  @class
  native primitive yield() -> Dynamic

  /// Yields execution from the active fiber, producing a value for the caller.
  @class
  native primitive yield(_ value: Dynamic) -> Dynamic

  /// Returns the currently executing fiber context.
  @class
  native primitive current -> Fiber

  /// Aborts the active fiber immediately with the given error.
  @class
  native primitive abort(_ error: Error) -> Never

  /// Returns `true` if this fiber has terminated.
  native isDone -> Bool

  /// Returns `true` if this fiber is the root execution fiber.
  native isRoot -> Bool

  /// Returns the error that caused this fiber to abort, if any.
  native error -> Option<Error>
}

/// Registration handle representing a listener subscribed to future readiness.
class FutureSubscription {
  mut _id
  mut _kind
  mut _fiber
  mut _generation
  mut _callback
  mut _onDetach
  mut _state
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

  /// Subscription identifier.
  id -> Int { _id }

  /// Returns `true` if this subscription is actively awaiting delivery.
  isActive -> Bool { _state == "active" }

  /// Returns `true` if this subscription has been cancelled or detached.
  isDetached -> Bool { _state == "detached" }

  /// Detaches and cancels this subscription, releasing all references immediately.
  ///
  /// @returns `true` if the subscription was active and successfully detached.
  detach() -> Bool {
    if _state == "active" {
      _state = "detached"
      _fiber = None
      _callback = None
      if _onDetach != None {
        let hook = _onDetach
        _onDetach = None
        hook.call()
      }
      return true
    }
    return false
  }

  /// Delivers readiness notification by waking the parked fiber or executing the callback.
  deliver() -> Unit {
    if _state == "active" {
      _state = "delivered"
      if _kind == "fiber" {
        let f = _fiber
        let gen = _generation
        _fiber = None
        _onDetach = None
        System.wake(f, gen)
      } else {
        let cb = _callback
        _callback = None
        _onDetach = None
        if cb != None {
          cb.call()
        }
      }
    }
    ()
  }
}

/// Settle-once representation of an asynchronous computation outcome.
///
/// A `Future<T>` transitions from pending to either fulfilled (with `T`) or
/// rejected (with `Error`). It provides combinators for chaining, transformation,
/// recovery, and concurrency orchestration.
class Future<T> {
  mut _outcome
  mut _registrations
  mut _nextRegId
  mut _detachCount

  /// Creates a pending future.
  @constructor
  new() {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
  }

  /// Creates an already-fulfilled future containing `v`.
  ///
  /// @param v Value to fulfill with.
  @constructor
  value(_ v: T) {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
    settleValue(v)
  }

  /// Creates an already-rejected future containing `e`.
  ///
  /// @param e Error to reject with.
  @constructor
  error(_ e: Error) {
    _outcome = None
    _registrations = List.new()
    _nextRegId = 0
    _detachCount = 0
    settleError(e)
  }

  /// Returns `true` if this future has settled (fulfilled or rejected).
  isReady -> Bool { _outcome.isSome }

  /// Internal settlement transition.
  internal trySettle(_ outcome: Result<T, Error>) -> Bool {
    if isReady {
      return false
    }
    _outcome = Option::Some(outcome)
    let regs = _registrations
    _registrations = List.new()
    let mut i = 0
    let n = regs.size
    while i < n {
      let r = regs.at(i)
      r.deliver()
      i = i + 1
    }
    true
  }

  /// Settles this future as fulfilled with `v` if pending; otherwise no-op.
  ///
  /// @param v Success value.
  /// @returns `self` for chaining.
  settleValue(_ v: T) -> Future<T> {
    let res: Result<T, Error> = Result::Ok(v)
    trySettle(res)
    self
  }

  /// Settles this future as rejected with `e` if pending; otherwise no-op.
  ///
  /// @param e Error instance.
  /// @returns `self` for chaining.
  settleError(_ e: Error) -> Future<T> {
    let res: Result<T, Error> = Result::Error(e)
    trySettle(res)
    self
  }

  /// Returns the settled value as an `Option<T>`.
  ///
  /// Returns `Some(value)` when fulfilled, or `None` when pending or rejected.
  value -> Option<T> {
    match _outcome {
      Option::Some(out) => {
        match out {
          Result::Ok(v) => Some(v)
          _ => None
        }
      }
      _ => None
    }
  }

  /// Returns the complete settlement outcome as `Option<Result<T, Error>>`.
  outcome -> Option<Result<T, Error>> {
    _outcome
  }

  // Amortized compaction to discard detached registration tombstones.
  private compactRegistrations() -> Unit {
    let compacted = List.new()
    let mut i = 0
    let n = _registrations.size
    while i < n {
      let r = _registrations.at(i)
      if r.isActive {
        compacted.rawPush(r)
      }
      i = i + 1
    }
    _registrations = compacted
    ()
  }

  /// Internal subscription seam for callbacks.
  internal subscribeReady(_ callback: () -> Unit) -> FutureSubscription {
    let regId = _nextRegId
    _nextRegId = _nextRegId + 1

    let reg = FutureSubscription.callback(
      regId,
      callback
    ) {
      _detachCount = _detachCount + 1
      if _detachCount >= 16 {
        compactRegistrations()
        _detachCount = 0
      }
    }


    if isReady {
      reg.deliver()
      return reg
    }

    _registrations.rawPush(reg)
    if isReady {
      reg.deliver()
    }
    reg
  }

  /// Suspends the current fiber until this future settles, returning the value or re-raising the error.
  ///
  /// @returns Fulfilled value of type `T`.
  await -> T {
    while not isReady {
      let current = Fiber.current
      if current.isRoot {
        System.runScheduled()
      } else {
        let generation = current.preparePark()
        if isReady {
          break
        }
        let regId = _nextRegId
        _nextRegId = _nextRegId + 1
        let reg = FutureSubscription.fiber(
          regId,
          current,
          generation
        ) {
          _detachCount = _detachCount + 1
          if _detachCount >= 16 {
            compactRegistrations()
            _detachCount = 0
          }
        }

        _registrations.rawPush(reg)
        try {
          current.park(generation)
        } catch e {
          reg.detach()
          e.raise()
        }
      }
    }

    match _outcome {
      Option::Some(out) => {
        match out {
          Result::Ok(v) => v
          Result::Error(e) => e.raise()
          _ => Error.new("await: invalid outcome").raise()
        }
      }
      _ => {
        Error.new("await: future settled without outcome").raise()
      }
    }
  }

  // Runs `action` to terminal completion and invokes the matching callback.
  @class
  private runToTerminal<U>(_ action: () -> U, _ onSuccess: (U) -> Unit, _ onError: (Error) -> Unit) -> Fiber {
    let fiber = Fiber.new(action)
    fiber.onComplete(|| {
      if fiber.error.isSome {
        onError.call(fiber.error.unwrapOr(None))
      } else {
        onSuccess.call(fiber.terminalValue)
      }
    })
    System.schedule(fiber)
    fiber
  }

  /// Executes an asynchronous block on a new fiber, returning a `Future` for its result.
  ///
  /// @typeparam U Return type of the asynchronous action.
  /// @param action Block to execute asynchronously.
  /// @returns Future tracking execution of the action.
  @class
  async<U>(_ action: () -> U) -> Future<U> {
    let result: Future<U> = Future.new()
    Future.runToTerminal(
      action,
      |value| { result.settleValue(value); () },
      |error| { result.settleError(error); () }
    )
    result
  }

  /// Transforms the fulfilled value of this future with a synchronous function.
  ///
  /// @typeparam U Target transformed value type.
  /// @param callback Function mapping `T` to `U`.
  /// @returns A new future resolving to the transformed value.
  map<U>(_ callback: (T) -> U) -> Future<U> {
    let result: Future<U> = Future.new()
    subscribeReady(|| {
      match _outcome {
        Option::Some(out) => {
          match out {
            Result::Ok(v) => {
              Future.runToTerminal(
                || { callback.call(v) },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
              ()
            }
            Result::Error(e) => {
              result.settleError(e)
              ()
            }
            _ => ()
          }
        }
        _ => ()
      }
      ()
    })
    result
  }

  /// Chains an asynchronous computation after this future fulfills.
  ///
  /// @typeparam U Value type of the subsequent future.
  /// @param callback Function returning a new future from the fulfilled value.
  /// @returns A future tracking the chained computation.
  then<U>(_ callback: (T) -> Future<U>) -> Future<U> {
    let result: Future<U> = Future.new()
    subscribeReady(|| {
      match _outcome {
        Option::Some(out) => {
          match out {
            Result::Ok(v) => {
              Future.runToTerminal(
                || {
                  let next = callback.call(v)
                  if next == result { throw Error.new("Future callback cannot adopt its own result") }
                  next.await
                },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
              ()
            }
            Result::Error(e) => {
              result.settleError(e)
              ()
            }
            _ => ()
          }
        }
        _ => ()
      }
      ()
    })
    result
  }

  /// Recovers from a rejection by computing a fallback value of type `T`.
  ///
  /// @param callback Error recovery function returning `T`.
  /// @returns A future recovering to a valid value on error.
  catch(_ callback: (Error) -> T) -> Future<T> {
    let result: Future<T> = Future.new()
    subscribeReady(|| {
      match _outcome {
        Option::Some(out) => {
          match out {
            Result::Ok(v) => {
              result.settleValue(v)
              ()
            }
            Result::Error(e) => {
              Future.runToTerminal(
                || { callback.call(e) },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
              ()
            }
            _ => ()
          }
        }
        _ => ()
      }
      ()
    })
    result
  }

  /// Recovers from a rejection using an asynchronous fallback operation.
  ///
  /// @param callback Error recovery function returning `Future<T>`.
  /// @returns A future tracking the recovery operation.
  recoverWith(_ callback: (Error) -> Future<T>) -> Future<T> {
    let result: Future<T> = Future.new()
    subscribeReady(|| {
      match _outcome {
        Option::Some(out) => {
          match out {
            Result::Ok(v) => {
              result.settleValue(v)
              ()
            }
            Result::Error(e) => {
              Future.runToTerminal(
                || {
                  let next = callback.call(e)
                  if next == result { throw Error.new("Future callback cannot adopt its own result") }
                  next.await
                },
                |value| { result.settleValue(value); () },
                |error| { result.settleError(error); () }
              )
              ()
            }
            _ => ()
          }
        }
        _ => ()
      }
      ()
    })
    result
  }

  /// Flattens a nested future `Future<Future<U>>` into `Future<U>`.
  ///
  /// @typeparam U Value type of inner future.
  /// @param nested Outer future containing an inner future.
  /// @returns Flattened future.
  @class
  flatten<U>(_ nested: Future<Future<U>>) -> Future<U> {
    nested.then(|inner| inner)
  }

  /// Waits for all input futures to fulfill, or rejects with the first observed rejection.
  ///
  /// @typeparam U Element value type.
  /// @param inputs List of futures to await.
  /// @returns Future resolving to a list of fulfilled values.
  @class
  all<U>(_ inputs: List<Future<U>>) -> Future<List<U>> {
    let n = inputs.size
    if n == 0 {
      return Future.value(List.new())
    }
    let result: Future<List<U>> = Future.new()
    let mut remaining = n
    let values = List.new()
    let mut idx = 0
    while idx < n {
      values.rawPush(None)
      idx = idx + 1
    }
    let mut settled = false
    let subs = List.new()

    let mut i = 0
    while i < n {
      if settled {
        break
      }
      let index = i
      let input: Future<U> = inputs.at(i)
      let sub = input.subscribeReady(|| {
        if settled {
          return ()
        }
        match input.outcome {
          Option::Some(out) => {
            match out {
              Result::Ok(v) => {
                if not settled {
                  values.at(index, put: v)
                  remaining = remaining - 1
                  if remaining == 0 {
                    settled = true
                    result.settleValue(values)
                  }
                }
                ()
              }
              Result::Error(e) => {
                if not settled {
                  settled = true
                  let mut j = 0
                  while j < subs.size {
                    let s = subs.at(j)
                    s.detach()
                    j = j + 1
                  }
                  result.settleError(e)
                }
                ()
              }
              _ => ()
            }
          }
          _ => ()
        }
        ()
      })
      subs.rawPush(sub)
      if settled {
        sub.detach()
      }
      i = i + 1
    }

    result
  }

  /// Waits for all input futures to settle, preserving outcome order as `List<Result<U, Error>>`.
  ///
  /// @typeparam U Element value type.
  /// @param inputs List of futures to monitor.
  /// @returns Future resolving to a list of settlement results.
  @class
  allSettled<U>(_ inputs: List<Future<U>>) -> Future<List<Result<U, Error>>> {
    let n = inputs.size
    if n == 0 {
      return Future.value(List.new())
    }
    let result: Future<List<Result<U, Error>>> = Future.new()
    let mut remaining = n
    let outcomes = List.new()
    let mut idx = 0
    while idx < n {
      outcomes.rawPush(None)
      idx = idx + 1
    }
    let subs = List.new()

    let mut i = 0
    while i < n {
      let index = i
      let input: Future<U> = inputs.at(i)
      let sub = input.subscribeReady(|| {
        match input.outcome {
          Option::Some(out) => {
            outcomes.at(index, put: out)
            remaining = remaining - 1
            if remaining == 0 {
              result.settleValue(outcomes)
            }
            ()
          }
          _ => ()
        }
        ()
      })
      subs.rawPush(sub)
      i = i + 1
    }

    result
  }

  /// Races input futures against each other; first observed settlement wins.
  ///
  /// @typeparam U Element value type.
  /// @param inputs List of futures to race.
  /// @returns Future reflecting the winning outcome.
  @class
  race<U>(_ inputs: List<Future<U>>) -> Future<U> {
    let n = inputs.size
    if n == 0 {
      return Future.error(ArgumentError.new("Future.race: cannot race on an empty list"))
    }
    let result: Future<U> = Future.new()
    let mut settled = false
    let subs = List.new()

    let mut i = 0
    while i < n {
      if settled {
        break
      }
      let index = i
      let input: Future<U> = inputs.at(i)
      let sub = input.subscribeReady(|| {
        if settled {
          return ()
        }
        match input.outcome {
          Option::Some(out) => {
            settled = true
            let mut j = 0
            while j < subs.size {
              let s = subs.at(j)
              s.detach()
              j = j + 1
            }
            match out {
              Result::Ok(v) => {
                result.settleValue(v)
                ()
              }
              Result::Error(e) => {
                result.settleError(e)
                ()
              }
              _ => ()
            }
          }
          _ => ()
        }
        ()
      })
      subs.rawPush(sub)
      if settled {
        sub.detach()
      }
      i = i + 1
    }

    result
  }

  /// Returns a future that rejects with `TimeoutError` if this future does not settle within `duration`.
  ///
  /// @param duration Maximum allowed waiting duration.
  /// @returns A future bounded by the specified timeout.
  timeout(_ duration: Duration) -> Future<T> {
    if duration.isNegative {
      throw ArgumentError.new("Future#timeout: duration must be non-negative")
    }
    if isReady {
      return self
    }
    let result: Future<T> = Future.new()
    let mut settled = false
    let mut sourceSub = None
    let mut timerSub = None

    let timer = System.sleep(duration)

    sourceSub = subscribeReady(|| {
      if not settled {
        settled = true
        if timerSub != None {
          timerSub.detach()
        }
        match outcome {
          Option::Some(out) => {
            match out {
              Result::Ok(v) => {
                result.settleValue(v)
                ()
              }
              Result::Error(e) => {
                result.settleError(e)
                ()
              }
              _ => ()
            }
          }
          _ => ()
        }
      }
      ()
    })

    if not settled {
      timerSub = timer.subscribeReady(|| {
        if not settled {
          settled = true
          if sourceSub != None {
            sourceSub.detach()
          }
          result.settleError(TimeoutError.new("Future timed out after " + duration.toString))
        }
        ()
      })
      if settled {
        if timerSub != None {
          timerSub.detach()
        }
      }
    }

    result
  }
}

export Fiber, Future, FutureSubscription
