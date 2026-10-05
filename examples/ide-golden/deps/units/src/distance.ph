/*@navigation.distance.definition*/
class Distance {
  mut _units: Int = 0

  @constructor
  new(_ units: Int) {
    _units = units
  }

  units -> Int { _units }
}

export Distance
