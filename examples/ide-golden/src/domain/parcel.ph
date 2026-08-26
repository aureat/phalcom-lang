from geo.point import Point
from units.weight import Weight

/*@definition.parcel*/
class Parcel {
  _id: String
  _destination: Point
  _weight: Weight

  @constructor
  new(_ id: String, destination: Point, weight: Weight) {
    _id = id
    _destination = destination
    _weight = weight
  }

  id -> String { _id }

  destination -> Point { _destination }

  weight -> Weight { _weight }
}

export Parcel
