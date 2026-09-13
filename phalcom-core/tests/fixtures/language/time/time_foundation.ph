// area: time
// spec: stdlib/time.md; system.md
// status: PASS

const systemClock = System.clock
System.print(systemClock === System.clock)

const duration = Duration.seconds(2) + Duration.milliseconds(500)
System.print(duration.nanoseconds)
System.print(duration.seconds)
System.print((duration - Duration.seconds(3)).isNegative)
System.print(Duration.seconds(2) == Duration.milliseconds(2_000))

const start = systemClock.now
const finish = systemClock.now
System.print(finish >= start)
System.print(start.plus(Duration.seconds(1)).minus(Duration.seconds(1)) == start)

class ManualClock is Clock {
  @constructor
  new() {
    _ticks = 0
  }

  now -> Instant {
    self.instant(_ticks)
  }

  advance(_ duration: Duration) -> ManualClock {
    _ticks = _ticks + duration.nanoseconds
    self
  }
}

const manual = ManualClock.new()
const manualStart = manual.now
manual.advance(Duration.seconds(2))
const manualFinish = manual.now
System.print(manualFinish.since(manualStart).nanoseconds)
System.print(manualStart.elapsed.nanoseconds)
