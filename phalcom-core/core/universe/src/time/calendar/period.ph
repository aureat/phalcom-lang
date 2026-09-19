data Period private(_ months: Int, _ days: Int)

impl Period {
  @class
  years(_ value: Int) -> Period {
    if not (value is Int) { ArgumentError.new("Period.years: value must be an Int").raise() }
    Period.months(value * 12)
  }

  @class
  months(_ value: Int) -> Period {
    if not (value is Int) { ArgumentError.new("Period.months: value must be an Int").raise() }
    Period(value, 0)
  }

  @class
  weeks(_ value: Int) -> Period {
    if not (value is Int) { ArgumentError.new("Period.weeks: value must be an Int").raise() }
    Period.days(value * 7)
  }

  @class
  days(_ value: Int) -> Period {
    if not (value is Int) { ArgumentError.new("Period.days: value must be an Int").raise() }
    Period(0, value)
  }

  @class
  zero -> Period { Period(0, 0) }

  +(_ other: Period) -> Period {
    if not (other is Period) { ArgumentError.new("Period.+: other must be a Period").raise() }
    Period(self.months + other.months, self.days + other.days)
  }

  -(_ other: Period) -> Period {
    if not (other is Period) { ArgumentError.new("Period.-: other must be a Period").raise() }
    Period(self.months - other.months, self.days - other.days)
  }

  *(_ factor: Int) -> Period {
    if not (factor is Int) { ArgumentError.new("Period.*: factor must be an Int").raise() }
    Period(self.months * factor, self.days * factor)
  }

  negated -> Period { Period(-self.months, -self.days) }

  isZero -> Bool {
    if self.months == 0 { self.days == 0 } else { false }
  }
}

export Period
