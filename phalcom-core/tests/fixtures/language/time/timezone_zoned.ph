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

const zone = TimeZone.named("America/New_York")
System.print(zone == TimeZone.named("US/Eastern"))
System.print(TimeZone.fixed(UtcOffset.seconds(3_600)) == TimeZone.fixed(UtcOffset.seconds(3_600)))
const start = DateTime.new(
  Date.new(2024, Month::March, 9),
  Time.new(12, 0, 0, 0)
).inZone(zone)

const shifted = start.shift(Duration.hours(24))

System.print(shifted.date.day)
System.print(shifted.time.hour)
System.print(start.sameInstant(start.withTimeZone(TimeZone.utc)))
System.print(start.compareInstant(start.shift(Duration.hours(1))) == Ordering::Less)
