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

  @class
  parse(_ text: String) -> Period {
    if text.size < 5 or text.codePointAt(0) != 80 { ArgumentError.new("Period.parse: expected P<months>M<days>D").raise() }
    let monthEnd = text.indexOf("M")
    let dayEnd = text.indexOf("D")
    if monthEnd < 2 or dayEnd <= monthEnd { ArgumentError.new("Period.parse: invalid fields").raise() }
    let monthText = text.slice(1, monthEnd)
    let dayText = text.slice(monthEnd + 1, dayEnd)
    Period.newParsed(Period.parseInteger(monthText), Period.parseInteger(dayText))
  }

  @class
  private newParsed(_ months: Int, _ days: Int) -> Period { Period(months, days) }

  @class
  private parseInteger(_ text: String) -> Int {
    let mut sign = 1
    let mut index = 0
    if text.size > 0 and text.codePointAt(0) == 45 { sign = -1; index = 1 }
    if index >= text.size { ArgumentError.new("Period.parse: invalid integer").raise() }
    let mut value = 0
    while index < text.size {
      let cp = text.codePointAt(index)
      if cp < 48 or cp > 57 { ArgumentError.new("Period.parse: invalid digit").raise() }
      value = value * 10 + cp - 48
      index = index + 1
    }
    value * sign
  }

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

  toString -> String { "P\(self.months)M\(self.days)D" }
}

export Period
