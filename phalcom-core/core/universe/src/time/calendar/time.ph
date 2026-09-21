data Time private(_ hour: Int, _ minute: Int, _ second: Int, _ nanosecond: Int)

impl Time {
  @class
  new(_ hour: Int, _ minute: Int, _ second: Int, _ nanosecond: Int) -> Time {
    if hour < 0 or hour > 23 { ArgumentError.new("Time.new: hour must be between 0 and 23").raise() }
    if minute < 0 or minute > 59 { ArgumentError.new("Time.new: minute must be between 0 and 59").raise() }
    if second < 0 or second > 59 { ArgumentError.new("Time.new: second must be between 0 and 59").raise() }
    if nanosecond < 0 or nanosecond > 999_999_999 {
      ArgumentError.new("Time.new: nanosecond must be between 0 and 999999999").raise()
    }
    Time(hour, minute, second, nanosecond)
  }

  @class
  parse(_ text: String) -> Time {
    const parts = text.split(":")
    if parts.size != 3 { ArgumentError.new("Time.parse: expected HH:MM:SS[.fraction]").raise() }
    const secondParts = parts.at(2).split(".")
    if secondParts.size > 2 { ArgumentError.new("Time.parse: invalid fraction").raise() }
    let nanos = 0
    if secondParts.size == 2 {
      const fraction = secondParts.at(1)
      if fraction.size == 0 or fraction.size > 9 { ArgumentError.new("Time.parse: fraction must contain 1-9 digits").raise() }
      let digits = 0
      let index = 0
      while index < fraction.size {
        const cp = fraction.codePointAt(index)
        if cp < 48 or cp > 57 { ArgumentError.new("Time.parse: invalid fraction").raise() }
        digits = digits * 10 + cp - 48
        index = index + 1
      }
      let scale = fraction.size
      while scale < 9 { digits = digits * 10; scale = scale + 1 }
      nanos = digits
    }
    Time.new(Time.parseField(parts.at(0)), Time.parseField(parts.at(1)), Time.parseField(secondParts.at(0)), nanos)
  }

  @class
  private parseField(_ text: String) -> Int {
    if text.size != 2 { ArgumentError.new("Time.parse: fields must have two digits").raise() }
    const a = text.codePointAt(0)
    const b = text.codePointAt(1)
    if a < 48 or a > 57 or b < 48 or b > 57 { ArgumentError.new("Time.parse: invalid digit").raise() }
    ((a - 48) * 10) + b - 48
  }

  @class
  midnight -> Time { Time(0, 0, 0, 0) }

  @class
  noon -> Time { Time(12, 0, 0, 0) }

  compare(_ other: Time) -> Ordering {
    if self.hour != other.hour { return self.hour.compare(other.hour) }
    if self.minute != other.minute { return self.minute.compare(other.minute) }
    if self.second != other.second { return self.second.compare(other.second) }
    self.nanosecond.compare(other.nanosecond)
  }

  <(_ other: Time) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: Time) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: Time) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: Time) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  toString -> String {
    const hourText = if self.hour < 10 { "0\(self.hour)" } else { "\(self.hour)" }
    const minuteText = if self.minute < 10 { "0\(self.minute)" } else { "\(self.minute)" }
    const secondText = if self.second < 10 { "0\(self.second)" } else { "\(self.second)" }
    const fraction = if self.nanosecond == 0 { "" } else { ".\(self.nanosecond)" }
    "\(hourText):\(minuteText):\(secondText)\(fraction)"
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Time) { return false }
    (self.hour == other.hour) and (self.minute == other.minute) and (self.second == other.second) and (self.nanosecond == other.nanosecond)
  }

  hash -> Int {
    (((self.hour * 60) + self.minute) * 60 + self.second) * 1_000_000_000 + self.nanosecond
  }
}

export Time
