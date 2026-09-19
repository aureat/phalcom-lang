// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time.calendar.time import Time

Time.new(0, 0, 0, 1_000_000_000)
