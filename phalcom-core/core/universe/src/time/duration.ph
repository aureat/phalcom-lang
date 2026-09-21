// Exact elapsed time represented as signed nanoseconds.
//
// Construction is private so factories remain the ordinary validation and
// construction boundary while the value itself stays transparent data.
data Duration private(_ nanoseconds: Int)

impl Duration {
  @class
  nanoseconds(_ value: Int) -> Duration {
    Duration(value)
  }

  @class
  microseconds(_ value: Int) -> Duration {
    Duration(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Duration {
    Duration(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Duration {
    Duration(value * 1_000_000_000)
  }

  @class
  minutes(_ value: Int) -> Duration {
    Duration.seconds(value * 60)
  }

  @class
  hours(_ value: Int) -> Duration {
    Duration.minutes(value * 60)
  }

  @class
  parse(_ text: String) -> Duration {
    if text.size < 5 or text.slice(0, 2) != "PT" or text.slice(text.size - 2, text.size) != "NS" {
      ArgumentError.new("Duration.parse: expected PT<nanoseconds>NS").raise()
    }
    const body = text.slice(2, text.size - 2)
    let sign = 1
    let index = 0
    if body.size > 0 and body.codePointAt(0) == 45 {
      sign = -1
      index = 1
    }
    if index >= body.size { ArgumentError.new("Duration.parse: missing nanoseconds").raise() }
    let value = 0
    while index < body.size {
      const cp = body.codePointAt(index)
      if cp < 48 or cp > 57 { ArgumentError.new("Duration.parse: invalid digit").raise() }
      value = value * 10 + cp - 48
      index = index + 1
    }
    Duration.nanoseconds(value * sign)
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
    "PT\(self.nanoseconds)NS"
  }
}

// A value observed over an elapsed interval. The generic argument remains
// part of the canonical data identity; no runtime measurement side channel is
// introduced.
data Measurement<T>(_ value: T, _ duration: Duration)

export Duration, Measurement
