from universe.time import (
  Instant,
  SystemClock
)

let mut raw = Instant.from(SystemClock.instance, 0)
let mut protectedValue = SystemClock.instance.instant(0)
let mut systemClock = System.clock
