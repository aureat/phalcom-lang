from universe.functional.chain import Chain
from universe.reflection.structural import Product
from ..buffer import Sized
from .field import Column, Parameter

data ModelError(_ field: String, _ message: String)

/// An ordered numerical view of a reflected product. Decoding updates a supplied
/// base value, preserving fields that the schema does not encode. Setter purity,
/// disjoint focuses, and round-trip laws remain caller obligations.
data ModelSchema<P, N: Int> private(
  _ columns: Sized<Column, N>,
  _ encodeModel: (P) -> Result<Sized<Float, N>, ModelError>,
  _ decodeModel: (Sized<Float, N>, P) -> Result<P, ModelError>
) where P |= Product

impl<P, N: Int> ModelSchema<P, N> where P |= Product {
  type Model = P
  type Shape = <P as Product>::Shape
  type Coordinates = Sized<Float, N>
  @class empty() -> ModelSchema<P, N> where N == 0 {
    ModelSchema<P, N>(Sized<Column, N>.from(Chain<Column>::Empty),
      |_| { Result::Ok(Sized<Float, N>.from(Chain<Float>::Empty)) },
      |_, base| { Result::Ok(base) })
  }
  columns -> Sized<Column, N> { self.columns }
  encode(_ model: P) -> Result<Sized<Float, N>, ModelError> { self.encodeModel.call(model) }
  decode(_ coordinates: Sized<Float, N>, into base: P) -> Result<P, ModelError> { self.decodeModel.call(coordinates, base) }

  /// Append in declaration order. M must be the successor extent; it is checked
  /// before composition, avoiding a claim of symbolic arithmetic normalization.
  append<T, M: Int>(_ parameter: Parameter<P, T>) -> ModelSchema<P, M> {
    if M != N + 1 { ArgumentError.new("ModelSchema.append: output extent must be input extent plus one").raise() }
    let column = parameter.column
    if not self.columns.values.filter(|existing| { existing.name == column.name }).isEmpty {
      ArgumentError.new("ModelSchema: duplicate parameter name").raise()
    }
    let columns = Sized<Column, M>.from(self.columns.values.append(Chain<Column>::Empty.prepend(column)))
    ModelSchema<P, M>(columns,
      |model| {
        match self.encode(model) {
          Result::Error(error) => Result::Error(error)
          Result::Ok(prefix) => {
            match parameter.encode(model) {
              Result::Error(error) => Result::Error(ModelError(column.name, error.message))
              Result::Ok(value) => Result::Ok(Sized<Float, M>.from(prefix.values.append(Chain<Float>::Empty.prepend(value))))
            }
          }
        }
      },
      |coordinates, base| {
        let prefix = Sized<Float, N>.from(coordinates.values.take(N))
        match self.decode(prefix, into: base) {
          Result::Error(error) => Result::Error(error)
          Result::Ok(updated) => {
            match parameter.decode(coordinates.at(N), into: updated) {
              Result::Error(error) => Result::Error(ModelError(column.name, error.message))
              Result::Ok(model) => Result::Ok(model)
            }
          }
        }
      })
  }
}
export ModelError, ModelSchema
