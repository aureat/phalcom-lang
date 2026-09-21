// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time import SystemClock

SystemClock.instance.now.since(Duration.seconds(1))
