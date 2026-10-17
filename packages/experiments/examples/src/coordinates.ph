from experiments.scientific import Vector, Point, LinearMap, AffineMap
from experiments.scientific.quantities import QuantityVectors
from measure.standard.si.base import m
from measure.standard.kinds import Length
from measure.quantity.quantity import Quantity

class World {}
class Camera {}
class Coordinates {
  @class run() -> Unit {
    let rotation = LinearMap<Float, 2, 2, World, Camera>.tabulate(|row, column| {
      if row == column { 0.0 } else if row == 0 { -1.0 } else { 1.0 }
    })
    let world = Vector<Float, 2, World>.tabulate(|index| { if index == 0 { 3.0 } else { 4.0 } })
    System.print(rotation.apply(world).coordinates.values)
    let origin = Point<Float, 2, World>.from(world.coordinates)
    let shift = Vector<Float, 2, Camera>.tabulate(|_| { 1.0 })
    let transform = AffineMap<Float, 2, 2, World, Camera>(linear: rotation, translation: shift)
    System.print(transform.apply(origin).coordinates.values)
    // Dimension-safe displacement: lengths are handled by measure's scalar
    // capabilities, rather than asserting that Quantity has a new conformance.
    let displacement = Vector<Quantity<Length, Float>, 2, World>.tabulate(|index| { m.approx(Float.new(index + 1)) })
    System.print(QuantityVectors.scaled(displacement, by: 2.0).coordinates.values)
    ()
  }
}
export World, Camera, Coordinates
