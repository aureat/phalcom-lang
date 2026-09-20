// Fixed signed displacement from UTC, represented with whole-second precision.
// The public validation range is the approved P1/P2 backend-interoperability
// envelope and is intentionally symmetric so negation is total at both ends.
data UtcOffset private(_ seconds: Int)

impl UtcOffset {
  @class
  utc -> UtcOffset { UtcOffset(0) }

  @class
  seconds(_ value: Int) -> UtcOffset {
    if value < -93_599 or value > 93_599 {
      ArgumentError.new("UtcOffset.seconds: value must be between -93599 and 93599").raise()
    }
    UtcOffset(value)
  }

  @class
  hours(_ value: Int) -> UtcOffset {
    UtcOffset.seconds(value * 3_600)
  }

  negated -> UtcOffset { UtcOffset.seconds(-self.seconds) }

  compare(_ other: UtcOffset) -> Ordering {
    self.seconds.compare(other.seconds)
  }

  <(_ other: UtcOffset) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: UtcOffset) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: UtcOffset) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: UtcOffset) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(UtcOffset) { return false }
    self.seconds == other.seconds
  }

  hash -> Int { self.seconds.hash }
}

export UtcOffset
