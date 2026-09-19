// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time.calendar.date import Date, DateOverflow
from universe.time.calendar.month import Month
from universe.time.calendar.period import Period

Date.new(2024, Month::January, 31).advance(
  Period.months(1),
  onOverflow: DateOverflow::Reject
)
