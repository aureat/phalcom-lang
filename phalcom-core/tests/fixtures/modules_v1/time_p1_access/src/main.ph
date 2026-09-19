from universe.time import (
  Instant,
  SystemClock
)

let raw = Instant.from(SystemClock.instance, 0)
let protectedValue = SystemClock.instance.instant(0)
let systemClock = System.clock
