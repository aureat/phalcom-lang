class Weight {
  _units: Int = 0

  @constructor
  new(_ units: Int) {
    _units = units
  }

  units -> Int { _units }
}

const weight: Int = Weight.new(5).units

export Weight
