@!documentation("Time, duration, calendar, and monotonic clock utilities.")

expose .clock
expose .duration
expose .timestamp
expose .calendar
from universe.time.clock import Clock, Instant, SystemClock
from universe.time.duration import Duration, Measurement
from universe.time.timestamp import Timestamp, WallClock, SystemWallClock, wallClock
from universe.time.calendar.month import Month
from universe.time.calendar.weekday import Weekday
from universe.time.calendar.period import Period

let clock = SystemClock.instance

export Clock, Instant, SystemClock, Duration, Measurement, Timestamp, WallClock, SystemWallClock, Month, Weekday, Period, clock, wallClock
