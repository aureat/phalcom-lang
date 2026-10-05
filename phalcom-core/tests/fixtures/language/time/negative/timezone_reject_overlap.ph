// area: time
// status: NEGATIVE

from universe.time import (
  Date,
  DateTime,
  Month,
  Time,
  TimeZone
)

let zone = TimeZone.named("America/New_York")
let overlap = DateTime.new(
  Date.new(2024, Month::November, 3),
  Time.new(1, 30, 0, 0)
)
overlap.inZone(zone)
