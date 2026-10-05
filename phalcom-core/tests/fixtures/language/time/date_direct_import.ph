// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.date import Date
from universe.time.calendar.month import Month

let epoch = Date.new(1970, Month::January, 1)
System.print(epoch.daysSince)
System.print(epoch.weekday.number)
