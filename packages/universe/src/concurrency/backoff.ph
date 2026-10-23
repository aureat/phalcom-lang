from universe.errors.argument import ArgumentError
from universe.time.duration import Duration

/// Delay calculation and suspension strategy for retries and backoff.
class Backoff {
  mut _kind
  mut _a
  mut _b

  /// Immediate retry without delay.
  @class
  none -> Backoff { Backoff.new("none", 0, 0) }

  /// Fixed constant delay between attempts.
  ///
  /// @param ms Delay in milliseconds.
  @class
  fixed(_ ms: Int) -> Backoff {
    if ms < 0 {
      throw ArgumentError.new("Backoff.fixed: delay must be non-negative")
    }
    Backoff.new("fixed", ms, ms)
  }

  @class
  fixed(ms: Int) -> Backoff {
    fixed(ms)
  }

  /// Exponential backoff with maximum saturation limit.
  ///
  /// @param base Initial delay base in milliseconds.
  /// @param max Maximum capped delay in milliseconds.
  @class
  exponential(_ base: Int, _ max: Int) -> Backoff {
    if base < 0 {
      throw ArgumentError.new("Backoff.exponential: base must be non-negative")
    }
    if max < base {
      throw ArgumentError.new("Backoff.exponential: max must be greater than or equal to base")
    }
    Backoff.new("exponential", base, max)
  }

  @class
  exponential(base: Int, max: Int) -> Backoff {
    exponential(base, max)
  }

  @constructor
  new(_ kind: String, _ a: Int, _ b: Int) {
    _kind = kind
    _a = a
    _b = b
  }

  /// Strategy kind (`"none"`, `"fixed"`, `"exponential"`).
  kind -> String { _kind }

  /// Computes the delay in milliseconds for a specific attempt index.
  ///
  /// @param attempt Zero-indexed attempt number.
  /// @returns Delay in milliseconds.
  delayFor(_ attempt: Int) -> Int {
    if attempt < 0 {
      throw ArgumentError.new("Backoff.delayFor: attempt must be non-negative")
    }
    if _kind == "none" {
      return 0
    }
    if _kind == "fixed" {
      return _a
    }
    if _kind == "exponential" {
      let base = _a
      let max = _b
      if base == 0 {
        return 0
      }
      let mut delay = base
      let mut i = 0
      while i < attempt and delay < max {
        delay = delay * 2
        if delay > max {
          delay = max
        }
        i = i + 1
      }
      return delay
    }
    0
  }

  /// Asynchronously suspends the current fiber for the calculated backoff delay.
  ///
  /// @param attempt Zero-indexed attempt number.
  waitBefore(_ attempt: Int) -> Unit {
    let delay = delayFor(attempt)
    if delay == 0 {
      return ()
    }
    System.sleep(Duration.milliseconds(delay)).await
    ()
  }
}

export Backoff
