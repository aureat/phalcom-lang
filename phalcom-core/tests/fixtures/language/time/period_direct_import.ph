// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.period import Period

let p = Period.months(1)
System.print(p.months)
