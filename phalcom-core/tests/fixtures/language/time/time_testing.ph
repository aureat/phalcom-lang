// area: time
// status: PASS

from universe.time import Duration, Timestamp, clock, wallClock
from universe.time.testing.clocks import ManualClock, ManualWallClock

const monotonic = ManualClock.new(100)
const first = monotonic.now
monotonic.advance(Duration.nanoseconds(25))
System.print(monotonic.now.since(first).nanoseconds)

const wall = ManualWallClock.new(Timestamp.seconds(10))
wall.advance(Duration.seconds(2))
System.print(wall.now.seconds)
wall.set(Timestamp.seconds(3))
System.print(wall.now.seconds)
System.print(clock.is(ManualClock))
System.print(wallClock.is(ManualWallClock))
