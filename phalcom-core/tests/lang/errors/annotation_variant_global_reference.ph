// area: errors
// spec: annotations-data.md @variant / this implementation specification
// status: PASS

@data @sealed
class Ordering {
  @variant Less()
  @variant Greater()

  @class less { Less.new() }
  @class greater { Greater.new() }
}

System.print(Less)
System.print(Greater)
System.print(Ordering.less is Less)
System.print(Ordering.greater is Greater)
