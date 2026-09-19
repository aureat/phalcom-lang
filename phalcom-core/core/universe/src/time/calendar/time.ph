data Time private(_ hour: Int, _ minute: Int, _ second: Int, _ nanosecond: Int)

impl Time {
  @class
  new(_ hour: Int, _ minute: Int, _ second: Int, _ nanosecond: Int) -> Time {
    if not (hour is Int) { ArgumentError.new("Time.new: hour must be an Int").raise() }
    if not (minute is Int) { ArgumentError.new("Time.new: minute must be an Int").raise() }
    if not (second is Int) { ArgumentError.new("Time.new: second must be an Int").raise() }
    if not (nanosecond is Int) { ArgumentError.new("Time.new: nanosecond must be an Int").raise() }
    if hour < 0 or hour > 23 { ArgumentError.new("Time.new: hour must be between 0 and 23").raise() }
    if minute < 0 or minute > 59 { ArgumentError.new("Time.new: minute must be between 0 and 59").raise() }
    if second < 0 or second > 59 { ArgumentError.new("Time.new: second must be between 0 and 59").raise() }
    if nanosecond < 0 or nanosecond > 999_999_999 {
      ArgumentError.new("Time.new: nanosecond must be between 0 and 999999999").raise()
    }
    Time(hour, minute, second, nanosecond)
  }

  @class
  midnight -> Time { Time(0, 0, 0, 0) }

  @class
  noon -> Time { Time(12, 0, 0, 0) }

  compare(_ other: Time) -> Ordering {
    if not (other is Time) { ArgumentError.new("Time.compare: other must be a Time").raise() }
    if self.hour != other.hour { return self.hour.compare(other.hour) }
    if self.minute != other.minute { return self.minute.compare(other.minute) }
    if self.second != other.second { return self.second.compare(other.second) }
    self.nanosecond.compare(other.nanosecond)
  }

  <(_ other: Time) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: Time) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Less) or (order === Ordering::Equal)
  }

  >(_ other: Time) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: Time) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Greater) or (order === Ordering::Equal)
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
