// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time.calendar.month import Month
from universe.time.calendar.partial import MonthDay

MonthDay.new(Month::April, 31)
