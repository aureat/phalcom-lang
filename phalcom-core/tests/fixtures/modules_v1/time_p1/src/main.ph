from universe.time import (
  Date,
  DateTime,
  Duration as ImportedDuration,
  Month,
  Period,
  Time,
  Timestamp as ImportedTimestamp,
  UtcOffset,
  clock
)

let mut date = Date.new(2024, Month::February, 29)
let mut time = Time.new(12, 34, 56, 0)
let mut dateTime = DateTime.new(date, time)
let mut period = Period.months(1) + Period.days(2)
let mut offset = UtcOffset.seconds(-21_036)
let mut instant = clock.now
let mut elapsed = instant.elapsed
let mut duration = Duration.seconds(5)
let mut timestamp = Timestamp.now

if date.day != 29 { ArgumentError.new("time_p1: invalid date").raise() }
if dateTime.date != date { ArgumentError.new("time_p1: invalid DateTime date").raise() }
if dateTime.time != time { ArgumentError.new("time_p1: invalid DateTime time").raise() }
if period.months != 1 or period.days != 2 { ArgumentError.new("time_p1: invalid Period").raise() }
if offset.seconds != -21_036 { ArgumentError.new("time_p1: invalid UtcOffset").raise() }
if not (elapsed is Duration) { ArgumentError.new("time_p1: clock.now must yield an Instant").raise() }
if not (duration is Duration) { ArgumentError.new("time_p1: Duration prelude missing").raise() }
if not (timestamp is Timestamp) { ArgumentError.new("time_p1: Timestamp prelude missing").raise() }
if not (ImportedDuration === Duration) { ArgumentError.new("time_p1: Duration identity drift").raise() }
if not (ImportedTimestamp === Timestamp) { ArgumentError.new("time_p1: Timestamp identity drift").raise() }

System.print("time_p1 ok")
