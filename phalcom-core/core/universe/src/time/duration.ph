// Exact elapsed time represented as signed nanoseconds.
//
// Construction is private so factories remain the ordinary validation and
// construction boundary while the value itself stays transparent data.
data Duration private(_ nanoseconds: Int)

impl Duration {
  @class
  nanoseconds(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.nanoseconds: value must be an Int").raise() }
    Duration(value)
  }

  @class
  microseconds(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.microseconds: value must be an Int").raise() }
    Duration(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.milliseconds: value must be an Int").raise() }
    Duration(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.seconds: value must be an Int").raise() }
    Duration(value * 1_000_000_000)
  }

  @class
  minutes(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.minutes: value must be an Int").raise() }
    Duration.seconds(value * 60)
  }

  @class
  hours(_ value: Int) -> Duration {
    if not (value is Int) { ArgumentError.new("Duration.hours: value must be an Int").raise() }
    Duration.minutes(value * 60)
  }

  microseconds -> Float { self.nanoseconds / 1_000 }

  milliseconds -> Float { self.nanoseconds / 1_000_000 }

  seconds -> Float { self.nanoseconds / 1_000_000_000 }

  +(_ other: Duration) -> Duration {
    Duration(self.nanoseconds + other.nanoseconds)
  }

  -(_ other: Duration) -> Duration {
    Duration(self.nanoseconds - other.nanoseconds)
  }

  *(_ factor: Int) -> Duration {
    Duration(self.nanoseconds * factor)
  }

  negated -> Duration { Duration(-self.nanoseconds) }

  abs -> Duration {
    if self.isNegative {
      return self.negated
    }
    self
  }

  compare(_ other: Duration) -> Ordering {
    self.nanoseconds.compare(other.nanoseconds)
  }

  <(_ other: Duration) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Duration) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: Duration) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Duration) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  isZero -> Bool { self.nanoseconds == 0 }

  isNegative -> Bool { self.nanoseconds < 0 }

  toString -> String {
    "\(self.nanoseconds)ns"
  }
}

// A value observed over an elapsed interval. The generic argument remains
// part of the canonical data identity; no runtime measurement side channel is
// introduced.
data Measurement<T>(_ value: T, _ duration: Duration)

export Duration, Measurement
