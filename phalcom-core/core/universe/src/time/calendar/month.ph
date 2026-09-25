// One source-level authority for proleptic Gregorian arithmetic. The helper
// lives beside Month so importing the public month module does not require a
// second nested module-linking path.
class Gregorian {
  @class
  isLeapYear(_ year: Int) -> Bool {
    if (year % 4) != 0 { return false }
    if (year % 100) != 0 { return true }
    (year % 400) == 0
  }

  @class
  daysInMonth(_ year: Int, _ month: Int) -> Int {
    if month < 1 or month > 12 {
      ArgumentError.new("Gregorian.daysInMonth: month must be between 1 and 12").raise()
    }
    if month == 1 { return 31 }
    if month == 2 {
      if Gregorian.isLeapYear(year) { return 29 } else { return 28 }
    }
    if month == 3 { return 31 }
    if month == 4 { return 30 }
    if month == 5 { return 31 }
    if month == 6 { return 30 }
    if month == 7 { return 31 }
    if month == 8 { return 31 }
    if month == 9 { return 30 }
    if month == 10 { return 31 }
    if month == 11 { return 30 }
    31
  }

  @class
  daysFromCivil(_ year: Int, _ month: Int, _ day: Int) -> Int {
    const monthLength = Gregorian.daysInMonth(year, month)
    if day < 1 or day > monthLength {
      ArgumentError.new("Gregorian.daysFromCivil: day is outside the month").raise()
    }
    let adjustedYear = year
    if month <= 2 { adjustedYear = adjustedYear - 1 }
    const era = Gregorian.floorDiv(adjustedYear, 400)
    const yearOfEra = adjustedYear - (era * 400)
    let monthPrime = month
    if monthPrime > 2 { monthPrime = monthPrime - 3 } else { monthPrime = monthPrime + 9 }
    const dayOfYear = ((153 * monthPrime + 2) ~/ 5) + day - 1
    const dayOfEra = (yearOfEra * 365) + (yearOfEra ~/ 4) - (yearOfEra ~/ 100) + dayOfYear
    (era * 146097) + dayOfEra - 719468
  }

  @class
  civilFromDays(_ days: Int) -> Record {
    const adjustedDays = days + 719468
    const era = Gregorian.floorDiv(adjustedDays, 146097)
    const dayOfEra = adjustedDays - (era * 146097)
    const yearOfEra = (dayOfEra - (dayOfEra ~/ 1460) + (dayOfEra ~/ 36524) - (dayOfEra ~/ 146096)) ~/ 365
    const yearBase = yearOfEra + (era * 400)
    const dayOfYear = dayOfEra - ((365 * yearOfEra) + (yearOfEra ~/ 4) - (yearOfEra ~/ 100))
    const monthPrime = (5 * dayOfYear + 2) ~/ 153
    const day = dayOfYear - ((153 * monthPrime + 2) ~/ 5) + 1
    let month = monthPrime
    if monthPrime < 10 { month = month + 3 } else { month = month - 9 }
    let resultYear = yearBase
    if month <= 2 { resultYear = resultYear + 1 }
    return { year: resultYear, month: month, day: day }
  }

  @class
  addMonthsYear(_ year: Int, _ month: Int, _ months: Int) -> Int {
    if month < 1 or month > 12 {
      ArgumentError.new("Gregorian.addMonthsYear: month must be between 1 and 12").raise()
    }
    const totalMonths = (year * 12) + (month - 1) + months
    Gregorian.floorDiv(totalMonths, 12)
  }

  @class
  addMonthsNumber(_ year: Int, _ month: Int, _ months: Int) -> Int {
    if month < 1 or month > 12 {
      ArgumentError.new("Gregorian.addMonthsNumber: month must be between 1 and 12").raise()
    }
    const totalMonths = (year * 12) + (month - 1) + months
    const targetYear = Gregorian.floorDiv(totalMonths, 12)
    totalMonths - (targetYear * 12) + 1
  }

  @class
  private floorDiv(_ value: Int, _ divisor: Int) -> Int {
    const quotient = value ~/ divisor
    const remainder = value % divisor
    if remainder < 0 { quotient - 1 } else { quotient }
  }
}

data Month {
  January
  February
  March
  April
  May
  June
  July
  August
  September
  October
  November
  December
}

impl Month {
  @class
  fromNumber(_ number: Int) -> Month {
    if number is not Int { ArgumentError.new("Month.fromNumber: number must be an Int").raise() }
    if number == 1 { return Month::January }
    if number == 2 { return Month::February }
    if number == 3 { return Month::March }
    if number == 4 { return Month::April }
    if number == 5 { return Month::May }
    if number == 6 { return Month::June }
    if number == 7 { return Month::July }
    if number == 8 { return Month::August }
    if number == 9 { return Month::September }
    if number == 10 { return Month::October }
    if number == 11 { return Month::November }
    if number == 12 { return Month::December }
    ArgumentError.new("Month.fromNumber: number must be between 1 and 12").raise()
  }

  number -> Int {
    match self {
      January => 1
      February => 2
      March => 3
      April => 4
      May => 5
      June => 6
      July => 7
      August => 8
      September => 9
      October => 10
      November => 11
      December => 12
    }
  }

  days(inYear: Int) -> Int {
    Gregorian.daysInMonth(inYear, self.number)
  }

  toString -> String {
    match self {
      January => "January"
      February => "February"
      March => "March"
      April => "April"
      May => "May"
      June => "June"
      July => "July"
      August => "August"
      September => "September"
      October => "October"
      November => "November"
      December => "December"
    }
  }
}

export Month, Gregorian
