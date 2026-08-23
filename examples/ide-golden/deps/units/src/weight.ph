class Weight {
  _units: Int = 0

  @constructor
  new(_ units: Int) {
    _units = units
  }

  units -> Int { _units }
}

export Weight
