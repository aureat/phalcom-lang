// Absolute Unix/POSIX time represented as signed nanoseconds from the epoch.

from universe.time.duration import Duration

class WallClock {
  now -> Timestamp
}

class SystemWallClock is WallClock {
  @class _instance

  @class
  instance -> SystemWallClock {
    if (_instance == None) {
      _instance = SystemWallClock.create()
    }
    _instance
  }

  @constructor
  private create() { self }

  @class
  @primitive
  internal native unixNanoseconds -> Int

  now -> Timestamp {
    Timestamp.nanoseconds(SystemWallClock.unixNanoseconds)
  }
}

let wallClock = SystemWallClock.instance

data Timestamp private(_ unixNanoseconds: Int)

impl Timestamp {
  @class
  epoch -> Timestamp { Timestamp.nanoseconds(0) }

  @class
  nanoseconds(_ value: Int) -> Timestamp {
    Timestamp(value)
  }

  @class
  microseconds(_ value: Int) -> Timestamp {
    Timestamp(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Timestamp {
    Timestamp(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Timestamp {
    Timestamp(value * 1_000_000_000)
  }

  microseconds -> Float { self.unixNanoseconds / 1_000 }

  milliseconds -> Float { self.unixNanoseconds / 1_000_000 }

  seconds -> Float { self.unixNanoseconds / 1_000_000_000 }

  @class
  now -> Timestamp { wallClock.now }

  plus(_ duration: Duration) -> Timestamp {
    Timestamp(self.unixNanoseconds + duration.nanoseconds)
  }

  minus(_ duration: Duration) -> Timestamp {
    Timestamp(self.unixNanoseconds - duration.nanoseconds)
  }

  +(_ duration: Duration) -> Timestamp {
    self.plus(duration)
  }

  -(_ duration: Duration) -> Timestamp {
    self.minus(duration)
  }

  since(_ earlier: Timestamp) -> Duration {
    Duration.nanoseconds(self.unixNanoseconds - earlier.unixNanoseconds)
  }

  compare(_ other: Timestamp) -> Ordering {
    self.unixNanoseconds.compare(other.unixNanoseconds)
  }

  <(_ other: Timestamp) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Timestamp) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: Timestamp) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Timestamp) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Timestamp) {
      return false
    }
    self.unixNanoseconds == other.unixNanoseconds
  }

  hash -> Int { self.unixNanoseconds.hash }

  toString -> String {
    const billion = 1_000_000_000
    let seconds = self.unixNanoseconds ~/ billion
    let nanos = self.unixNanoseconds % billion
    if nanos < 0 {
      seconds = seconds - 1
      nanos = nanos + billion
    }
    const days = Timestamp.floorDiv(seconds, 86_400)
    const daySeconds = seconds - (days * 86_400)
    const civil = Timestamp.civilFromDays(days)
    const hour = daySeconds ~/ 3_600
    const minute = (daySeconds % 3_600) ~/ 60
    const second = daySeconds % 60
    const yearText = "\(civil.get(#year).unwrapOr(0))"
    const monthNumber = civil.get(#month).unwrapOr(0)
    const dayNumber = civil.get(#day).unwrapOr(0)
    const monthText = if monthNumber < 10 { "0\(monthNumber)" } else { "\(monthNumber)" }
    const dayText = if dayNumber < 10 { "0\(dayNumber)" } else { "\(dayNumber)" }
    const hourText = if hour < 10 { "0\(hour)" } else { "\(hour)" }
    const minuteText = if minute < 10 { "0\(minute)" } else { "\(minute)" }
    const secondText = if second < 10 { "0\(second)" } else { "\(second)" }
    const fraction = if nanos == 0 { "" } else { ".\(nanos)" }
    "\(yearText)-\(monthText)-\(dayText)T\(hourText):\(minuteText):\(secondText)\(fraction)Z"
  }

  @class
  internal floorDiv(_ value: Int, _ divisor: Int) -> Int {
    const quotient = value ~/ divisor
    const remainder = value % divisor
    if remainder < 0 { quotient - 1 } else { quotient }
  }

  @class
  internal civilFromDays(_ days: Int) -> Record {
    const adjustedDays = days + 719468
    const era = Timestamp.floorDiv(adjustedDays, 146097)
    const dayOfEra = adjustedDays - (era * 146097)
    const yearOfEra = (dayOfEra - (dayOfEra ~/ 1460) + (dayOfEra ~/ 36524) - (dayOfEra ~/ 146096)) ~/ 365
    const yearBase = yearOfEra + (era * 400)
    const dayOfYear = dayOfEra - ((365 * yearOfEra) + (yearOfEra ~/ 4) - (yearOfEra ~/ 100))
    const monthPrime = (5 * dayOfYear + 2) ~/ 153
    const day = dayOfYear - ((153 * monthPrime + 2) ~/ 5) + 1
    let month = monthPrime
    if monthPrime < 10 { month = month + 3 } else { month = month - 9 }
    let year = yearBase
    if month <= 2 { year = year + 1 }
    return { year: year, month: month, day: day }
  }
}

export Timestamp, WallClock, SystemWallClock, wallClock
