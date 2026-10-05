from .point import Point
from units.distance import Distance

class Route {
  mut _origin: Point
  mut _destination: Point

  @constructor
  new(_ origin: Point, destination: Point) {
    _origin = origin
    _destination = destination
  }

  origin -> Point { _origin }
  destination -> Point { _destination }

  distance -> Distance {
    let dx = _destination.x - _origin.x
    let dy = _destination.y - _origin.y
    Distance.new(dx + dy)
  }
}

export Route
