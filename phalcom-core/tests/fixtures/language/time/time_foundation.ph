// area: time
// spec: stdlib/time.md; system.md
// status: PASS

from universe.time.duration import Measurement
from universe.time import (
  SystemClock,
  SystemWallClock
)
from universe.time.calendar.month import Month
from universe.time.calendar.weekday import Weekday

let systemClock = SystemClock.instance
System.print(systemClock === SystemClock.instance)

let duration = Duration.seconds(2) + Duration.milliseconds(500)
System.print(duration.nanoseconds)
System.print(duration.seconds)
System.print((duration - Duration.seconds(3)).isNegative)
System.print(Duration.seconds(2) == Duration.milliseconds(2_000))
System.print(Timestamp.epoch.unixNanoseconds)
System.print(Timestamp.nanoseconds(-1).unixNanoseconds)
System.print(Timestamp.seconds(-2).unixNanoseconds)
System.print(Duration.milliseconds(500).is(Duration))
let shifted = Timestamp.seconds(2) + Duration.milliseconds(500)
System.print(shifted.since(Timestamp.seconds(1)).nanoseconds)
System.print(shifted.minus(Duration.seconds(2)) == Timestamp.milliseconds(500))
System.print(Timestamp.epoch < Timestamp.nanoseconds(1))
System.print(Timestamp.epoch.hash == Timestamp.nanoseconds(0).hash)
System.print(SystemWallClock.instance === SystemWallClock.instance)
let importedWallClock = SystemWallClock.instance
System.print(importedWallClock === SystemWallClock.instance)
System.print(Timestamp.now.is(Timestamp))
System.print(Month::February.days(inYear: 2024))
System.print(Month::February.days(inYear: 1900))
System.print(Month.fromNumber(12).number)
System.print(Weekday.fromNumber(7).number)
let measurement = Measurement(7, Duration.seconds(1))
System.print(measurement.value)
System.print(measurement.duration.nanoseconds)

let start = systemClock.now
let finish = systemClock.now
System.print(finish >= start)
System.print(start.plus(Duration.seconds(1)).minus(Duration.seconds(1)) == start)
let measured = systemClock.measure { 7 }
System.print(measured.value)
