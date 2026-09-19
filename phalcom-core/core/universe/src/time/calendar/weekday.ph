enum Weekday {
  Monday
  Tuesday
  Wednesday
  Thursday
  Friday
  Saturday
  Sunday
}

impl Weekday {
  @class
  fromNumber(_ number: Int) -> Weekday {
    if not (number is Int) { ArgumentError.new("Weekday.fromNumber: number must be an Int").raise() }
    if number == 1 { return Weekday::Monday }
    if number == 2 { return Weekday::Tuesday }
    if number == 3 { return Weekday::Wednesday }
    if number == 4 { return Weekday::Thursday }
    if number == 5 { return Weekday::Friday }
    if number == 6 { return Weekday::Saturday }
    if number == 7 { return Weekday::Sunday }
    ArgumentError.new("Weekday.fromNumber: number must be between 1 and 7").raise()
  }

  number -> Int {
    match self {
      Monday => 1
      Tuesday => 2
      Wednesday => 3
      Thursday => 4
      Friday => 5
      Saturday => 6
      Sunday => 7
    }
  }

  toString -> String {
    match self {
      Monday => "Monday"
      Tuesday => "Tuesday"
      Wednesday => "Wednesday"
      Thursday => "Thursday"
      Friday => "Friday"
      Saturday => "Saturday"
      Sunday => "Sunday"
    }
  }
}

export Weekday
