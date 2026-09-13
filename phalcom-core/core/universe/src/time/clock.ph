// Monotonic clock values and signed nanosecond durations.

class Duration {
  @private
  @constructor
  create(_ nanoseconds: Int) {
    _nanoseconds = nanoseconds
  }

  @class
  nanoseconds(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.nanoseconds: value must be an Int") }
    Duration.create(value)
  }

  @class
  microseconds(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.microseconds: value must be an Int") }
    Duration.create(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.milliseconds: value must be an Int") }
    Duration.create(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.seconds: value must be an Int") }
    Duration.create(value * 1_000_000_000)
  }

  @class
  minutes(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.minutes: value must be an Int") }
    Duration.seconds(value * 60)
  }

  @class
  hours(_ value: Int) -> Duration {
    if not value.is(Int) { throw ArgumentError.new("Duration.hours: value must be an Int") }
    Duration.minutes(value * 60)
  }

  nanoseconds -> Int { _nanoseconds }

  microseconds -> Number { _nanoseconds / 1_000 }

  milliseconds -> Number { _nanoseconds / 1_000_000 }

  seconds -> Number { _nanoseconds / 1_000_000_000 }

  +(_ other: Duration) -> Duration {
    Duration.nanoseconds(_nanoseconds + other.nanoseconds)
  }

  -(_ other: Duration) -> Duration {
    Duration.nanoseconds(_nanoseconds - other.nanoseconds)
  }

  *(_ factor: Int) -> Duration {
    Duration.nanoseconds(_nanoseconds * factor)
  }

  compare(_ other: Duration) -> Ordering {
    _nanoseconds.compare(other.nanoseconds)
  }

  <(_ other: Duration) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Duration) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Less) or (order === Ordering::Equal)
  }

  >(_ other: Duration) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Duration) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Greater) or (order === Ordering::Equal)
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Duration) {
      return false
    }
    _nanoseconds == other.nanoseconds
  }

  hash -> Int { _nanoseconds.hash }

  isZero -> Bool { _nanoseconds == 0 }

  isNegative -> Bool { _nanoseconds < 0 }

  toString -> String {
    "\(_nanoseconds)ns"
  }
}

class Instant {
  @private
  @constructor
  create(_ clock: Clock, _ ticks: Int) {
    _clock = clock
    _ticks = ticks
  }

  @class
  @internal
  _$from(_ clock: Clock, _ ticks: Int) -> Instant {
    Instant.create(clock, ticks)
  }

  @internal
  _$clock -> Clock { _clock }

  @internal
  _$ticks -> Int { _ticks }

  elapsed -> Duration {
    _clock.since(self)
  }

  since(_ earlier: Instant) -> Duration {
    if not earlier.is(Instant) { throw ArgumentError.new("Instant.since: earlier must be an Instant") }
    self.requireSameClock(earlier)
    Duration.nanoseconds(_ticks - earlier._$ticks)
  }

  plus(_ duration: Duration) -> Instant {
    if not duration.is(Duration) { throw ArgumentError.new("Instant.plus: duration must be a Duration") }
    Instant._$from(_clock, _ticks + duration.nanoseconds)
  }

  minus(_ duration: Duration) -> Instant {
    if not duration.is(Duration) { throw ArgumentError.new("Instant.minus: duration must be a Duration") }
    Instant._$from(_clock, _ticks - duration.nanoseconds)
  }

  compare(_ other: Instant) -> Ordering {
    if not other.is(Instant) { throw ArgumentError.new("Instant.compare: other must be an Instant") }
    self.requireSameClock(other)
    _ticks.compare(other._$ticks)
  }

  <(_ other: Instant) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Instant) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Less) or (order === Ordering::Equal)
  }

  >(_ other: Instant) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Instant) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Greater) or (order === Ordering::Equal)
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Instant) {
      return false
    }
    (_clock === other._$clock) and (_ticks == other._$ticks)
  }

  hash -> Int {
    (_clock.hash * 31) + _ticks.hash
  }

  @private
  requireSameClock(_ other: Instant) -> Unit {
    if (_clock === other._$clock) {
      return ()
    }
    ArgumentError.new("Instant values belong to different clocks").raise()
  }
}

class Clock {
  @class _system

  @class
  @internal
  _$system -> Clock {
    if (_system == None) {
      _system = Clock.create()
    }
    _system
  }

  @private
  @constructor
  create() { self }

  now -> Instant {
    self.instant(System._$monotonicNanoseconds)
  }

  since(_ instant: Instant) -> Duration {
    self.now.since(instant)
  }

  @protected
  instant(_ ticks: Int) -> Instant {
    Instant._$from(self, ticks)
  }
}

export Clock, Instant, Duration
