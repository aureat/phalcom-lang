from universe.time.clock import Clock, Instant
from universe.time.duration import Duration
from universe.time.timestamp import Timestamp
from universe.time.timestamp import WallClock

class ManualClock is Clock {
  mut _ticks
  @constructor
  new(_ initialTicks: Int) {
    _ticks = initialTicks
  }

  now -> Instant { self.instant(_ticks) }

  advance(_ duration: Duration) {
    if duration.isNegative { ArgumentError.new("ManualClock.advance: duration must not be negative").raise() }
    _ticks = _ticks + duration.nanoseconds
    self
  }
}

class ManualWallClock is WallClock {
  mut _current
  @constructor
  new(_ initial: Timestamp) {
    _current = initial
  }

  now -> Timestamp { _current }

  advance(_ duration: Duration) {
    _current = _current + duration
    self
  }

  set(_ timestamp: Timestamp) {
    _current = timestamp
    self
  }
}

export ManualClock, ManualWallClock
