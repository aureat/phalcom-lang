from universe.reflection.structural import Product
from ..schemas import ModelSchema, ModelError
from .vector import Vector

/// Optimizer coordinates are tagged with their model type. Numerical conversion
/// is explicit, and categorical columns remain governed by their codec contracts.
impl<P, N: Int> ModelSchema<P, N> where P |= Product {
  encodeVector(_ model: P) -> Result<Vector<Float, N, P>, ModelError> {
    self.encode(model).map(|coordinates| { Vector<Float, N, P>.from(coordinates) })
  }
  decodeVector(_ vector: Vector<Float, N, P>, into base: P) -> Result<P, ModelError> {
    self.decode(vector.coordinates, into: base)
  }
}
