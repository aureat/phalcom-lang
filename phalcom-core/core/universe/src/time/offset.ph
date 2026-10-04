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

  @class
  parse(_ text: String) -> UtcOffset {
    if text.size < 6 { ArgumentError.new("UtcOffset.parse: expected signed offset").raise() }
    let signCode = text.codePointAt(0)
    if signCode != 43 and signCode != 45 { ArgumentError.new("UtcOffset.parse: expected sign").raise() }
    let parts = text.slice(1, text.size).split(":")
    if parts.size != 2 and parts.size != 3 { ArgumentError.new("UtcOffset.parse: expected HH:MM[:SS]").raise() }
    let hours = UtcOffset.parseField(parts.at(0))
    let minutes = UtcOffset.parseField(parts.at(1))
    let seconds = if parts.size == 3 { UtcOffset.parseField(parts.at(2)) } else { 0 }
    if minutes > 59 or seconds > 59 { ArgumentError.new("UtcOffset.parse: component out of range").raise() }
    let total = (hours * 3_600) + (minutes * 60) + seconds
    UtcOffset.seconds(if signCode == 45 { -total } else { total })
  }

  @class
  private parseField(_ text: String) -> Int {
    if text.size != 2 { ArgumentError.new("UtcOffset.parse: fields must have two digits").raise() }
    let first = text.codePointAt(0)
    let second = text.codePointAt(1)
    if first < 48 or first > 57 or second < 48 or second > 57 { ArgumentError.new("UtcOffset.parse: invalid digit").raise() }
    ((first - 48) * 10) + second - 48
  }

  negated -> UtcOffset { UtcOffset.seconds(-self.seconds) }

  compare(_ other: UtcOffset) -> Ordering {
    self.seconds.compare(other.seconds)
  }

  <(_ other: UtcOffset) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: UtcOffset) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: UtcOffset) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: UtcOffset) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(UtcOffset) { return false }
    self.seconds == other.seconds
  }

  hash -> Int { self.seconds.hash }

  toString -> String {
    let sign = if self.seconds < 0 { "-" } else { "+" }
    let absolute = if self.seconds < 0 { -self.seconds } else { self.seconds }
    let hours = absolute ~/ 3_600
    let minuteTotal = (absolute % 3_600)
    let minutes = minuteTotal ~/ 60
    let seconds = absolute % 60
    let hourText = if hours < 10 { "0\(hours)" } else { "\(hours)" }
    let minuteText = if minutes < 10 { "0\(minutes)" } else { "\(minutes)" }
    if seconds == 0 {
      "\(sign)\(hourText):\(minuteText)"
    } else {
      let secondText = if seconds < 10 { "0\(seconds)" } else { "\(seconds)" }
      "\(sign)\(hourText):\(minuteText):\(secondText)"
    }
  }
}

export UtcOffset
