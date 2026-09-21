// area: time
// status: PASS

from universe.time import (
  Date,
  DateTime,
  Month,
  Time,
  TimeZone,
  Timestamp,
  LocalTimeDisambiguation
)

const newYork = TimeZone.named("America/New_York")

const unique = DateTime.new(
  Date.new(2024, Month::January, 15),
  Time.new(12, 0, 0, 0)
)
System.print(unique.inZone(newYork).offset.seconds)

const overlap = DateTime.new(
  Date.new(2024, Month::November, 3),
  Time.new(1, 30, 0, 0)
)
System.print(overlap.inZone(newYork, LocalTimeDisambiguation::Earlier).offset.seconds)
System.print(overlap.inZone(newYork, LocalTimeDisambiguation::Later).offset.seconds)
System.print(overlap.inZone(newYork, LocalTimeDisambiguation::Compatible).offset.seconds)

const gap = DateTime.new(
  Date.new(2024, Month::March, 10),
  Time.new(2, 30, 0, 0)
)
System.print(gap.inZone(newYork, LocalTimeDisambiguation::Earlier).offset.seconds)
System.print(gap.inZone(newYork, LocalTimeDisambiguation::Later).offset.seconds)
System.print(gap.inZone(newYork, LocalTimeDisambiguation::Compatible).offset.seconds)
const transition = newYork.transitionAfter(Timestamp.seconds(0))
System.print(transition.is(Some))
