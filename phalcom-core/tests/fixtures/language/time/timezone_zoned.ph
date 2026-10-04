// area: time
// status: PASS

from universe.time import (
  Date,
  DateTime,
  Duration,
  Month,
  Period,
  Time,
  TimeZone,
  UtcOffset
)

let zone = TimeZone.named("America/New_York")
System.print(zone == TimeZone.named("US/Eastern"))
System.print(TimeZone.fixed(UtcOffset.seconds(3_600)) == TimeZone.fixed(UtcOffset.seconds(3_600)))
let start = DateTime.new(
  Date.new(2024, Month::March, 9),
  Time.new(12, 0, 0, 0)
).inZone(zone)

let shifted = start.shift(Duration.hours(24))

System.print(shifted.date.day)
System.print(shifted.time.hour)
System.print(start.sameInstant(start.withTimeZone(TimeZone.utc)))
System.print(start.compareInstant(start.shift(Duration.hours(1))) == Ordering::Less)
