from universe.time.duration import Duration, Measurement

// Monotonic clock values. Duration is defined in duration.ph as transparent
// validated data and is imported by the package surface.

class Instant {
  mut _clock
  mut _ticks
  @constructor
  private create(_ clock: Clock, _ ticks: Int) {
    _clock = clock
    _ticks = ticks
  }

  @class
  internal from(_ clock: Clock, _ ticks: Int) -> Instant {
    Instant.create(clock, ticks)
  }

  internal clock -> Clock { _clock }

  internal ticks -> Int { _ticks }

  elapsed -> Duration {
    _clock.since(self)
  }

  since(_ earlier: Instant) -> Duration {
    self.requireSameClock(earlier)
    Duration.nanoseconds(_ticks - earlier.ticks)
  }

  plus(_ duration: Duration) -> Instant {
    Instant.from(_clock, _ticks + duration.nanoseconds)
  }

  minus(_ duration: Duration) -> Instant {
    Instant.from(_clock, _ticks - duration.nanoseconds)
  }

  compare(_ other: Instant) -> Ordering {
    self.requireSameClock(other)
    _ticks.compare(other.ticks)
  }

  <(_ other: Instant) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Instant) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: Instant) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Instant) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Instant) {
      return false
    }
    (_clock === other.clock) and (_ticks == other.ticks)
  }

  hash -> Int {
    (_clock.hash * 31) + _ticks.hash
  }

  private requireSameClock(_ other: Instant) -> Unit {
    if (_clock === other.clock) {
      return ()
    }
    ArgumentError.new("Instant values belong to different clocks").raise()
  }
}

class Clock {
  now -> Instant

  since(_ instant: Instant) -> Duration {
    self.now.since(instant)
  }

  measure<T>(_ action: () -> T) -> Measurement<T> {
    let started = self.now
    let value = action.call()
    let elapsed = self.now.since(started)
    Measurement(value, elapsed)
  }

  protected instant(_ ticks: Int) -> Instant {
    Instant.from(self, ticks)
  }
}

class SystemClock is Clock {
  @class mut _instance

  @class
  instance -> SystemClock {
    if (_instance == None) {
      _instance = SystemClock.create()
    }
    _instance
  }

  @constructor
  private create() { self }

  @class
  internal native primitive monotonicNanoseconds -> Int

  now -> Instant {
    self.instant(SystemClock.monotonicNanoseconds)
  }
}

export Clock, Instant, SystemClock
