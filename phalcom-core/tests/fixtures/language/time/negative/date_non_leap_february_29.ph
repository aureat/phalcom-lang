// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time.calendar.date import Date
from universe.time.calendar.month import Month

Date.new(1900, Month::February, 29)
