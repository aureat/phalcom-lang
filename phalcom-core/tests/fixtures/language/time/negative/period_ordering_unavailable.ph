// area: time
// spec: stdlib/time.md
// status: NEGATIVE

from universe.time.calendar.period import Period

Period.months(1) < Period.months(2)
