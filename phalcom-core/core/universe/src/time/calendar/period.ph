data Period private(_ months: Int, _ days: Int)

impl Period {
  @class
  years(_ value: Int) -> Period {
    Period.months(value * 12)
  }

  @class
  months(_ value: Int) -> Period {
    Period(value, 0)
  }

  @class
  weeks(_ value: Int) -> Period {
    Period.days(value * 7)
  }

  @class
  days(_ value: Int) -> Period {
    Period(0, value)
  }

  @class
  zero -> Period { Period(0, 0) }

  +(_ other: Period) -> Period {
    Period(self.months + other.months, self.days + other.days)
  }

  -(_ other: Period) -> Period {
    Period(self.months - other.months, self.days - other.days)
  }

  *(_ factor: Int) -> Period {
    Period(self.months * factor, self.days * factor)
  }

  negated -> Period { Period(-self.months, -self.days) }

  isZero -> Bool {
    if self.months == 0 { self.days == 0 } else { false }
  }
}

export Period
