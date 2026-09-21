// area: time
// status: PASS

from universe.time import Date, DateTime, Duration, Month, Period, Time, TimeZone, Timestamp, UtcOffset, ZonedDateTime

System.print(Date.new(2024, Month::January, 5))
System.print(Time.new(3, 4, 5, 0))
System.print(Time.new(3, 4, 5, 123_000_000))
System.print(DateTime.new(Date.new(2024, Month::January, 5), Time.new(3, 4, 5, 0)))
System.print(UtcOffset.seconds(3_600))
System.print(UtcOffset.seconds(-21_036))
System.print(UtcOffset.parse("+01:00"))
System.print(UtcOffset.parse("-05:50:36"))
System.print(Duration.seconds(2))
System.print(Period.months(1) + Period.days(2))
System.print(Timestamp.seconds(0))
System.print(ZonedDateTime.from(Timestamp.seconds(0), TimeZone.utc))
System.print(ZonedDateTime.from(Timestamp.seconds(0), TimeZone.named("America/New_York")))
