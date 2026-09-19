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
    if not (value is Int) { ArgumentError.new("Timestamp.nanoseconds: value must be an Int").raise() }
    Timestamp(value)
  }

  @class
  microseconds(_ value: Int) -> Timestamp {
    if not (value is Int) { ArgumentError.new("Timestamp.microseconds: value must be an Int").raise() }
    Timestamp(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Timestamp {
    if not (value is Int) { ArgumentError.new("Timestamp.milliseconds: value must be an Int").raise() }
    Timestamp(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Timestamp {
    if not (value is Int) { ArgumentError.new("Timestamp.seconds: value must be an Int").raise() }
    Timestamp(value * 1_000_000_000)
  }

  microseconds -> Float { self.unixNanoseconds / 1_000 }

  milliseconds -> Float { self.unixNanoseconds / 1_000_000 }

  seconds -> Float { self.unixNanoseconds / 1_000_000_000 }

  @class
  now -> Timestamp { wallClock.now }

  plus(_ duration: Duration) -> Timestamp {
    if not (duration is Duration) { ArgumentError.new("Timestamp.plus: duration must be a Duration").raise() }
    Timestamp(self.unixNanoseconds + duration.nanoseconds)
  }

  minus(_ duration: Duration) -> Timestamp {
    if not (duration is Duration) { ArgumentError.new("Timestamp.minus: duration must be a Duration").raise() }
    Timestamp(self.unixNanoseconds - duration.nanoseconds)
  }

  +(_ duration: Duration) -> Timestamp {
    self.plus(duration)
  }

  -(_ duration: Duration) -> Timestamp {
    self.minus(duration)
  }

  since(_ earlier: Timestamp) -> Duration {
    if not (earlier is Timestamp) { ArgumentError.new("Timestamp.since: earlier must be a Timestamp").raise() }
    Duration.nanoseconds(self.unixNanoseconds - earlier.unixNanoseconds)
  }

  compare(_ other: Timestamp) -> Ordering {
    if not (other is Timestamp) { ArgumentError.new("Timestamp.compare: other must be a Timestamp").raise() }
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
    "\(self.unixNanoseconds)ns"
  }
}

export Timestamp, WallClock, SystemWallClock, wallClock
