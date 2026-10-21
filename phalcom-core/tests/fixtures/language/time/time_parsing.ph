// area: time
// status: PASS

from universe.time import Date, DateTime, Duration, Month, Period, Time

System.print(Duration.parse("PT2000000000NS"))
System.print(Duration.parse("PT-5NS"))
System.print(Period.parse("P1M2D"))
System.print(Period.parse("P-1M2D"))
System.print(Period.parse("P-1M-2D"))
System.print(Period.parse("P0M0D"))
System.print(Date.parse("2024-01-05"))
System.print(Time.parse("03:04:05"))
System.print(Time.parse("03:04:05.123"))
System.print(DateTime.parse("2024-01-05T03:04:05.123"))
