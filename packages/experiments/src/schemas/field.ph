from universe.functional.lens import Lens
from universe.reflection.structural import Product
from .encoding import Encoding, Codec, CodecError

/// Numerical column metadata; names and order are explicit schema choices.
data Column(_ name: String, _ encoding: Encoding)

/// Heterogeneous field typing is retained before a parameter becomes a column.
/// Product.Shape is canonical reflection, independent of the numerical encoding.
data Parameter<P, T> private(_ name: String, _ focus: Lens<P, T>, _ codec: Codec<T>)
  where P |= Product
impl<P, T> Parameter<P, T> where P |= Product {
  type Model = P
  type Value = T
  type ModelShape = <P as Product>::Shape
  @class new(_ name: String, focus: Lens<P, T>, codec: Codec<T>) -> Parameter<P, T> {
    if name == "" { ArgumentError.new("Parameter name must be nonempty").raise() }
    Parameter<P, T>(name, focus, codec)
  }
  column -> Column { Column(self.name, self.codec.encoding) }
  encode(_ model: P) -> Result<Float, CodecError> { self.codec.encode(self.focus.get(model)) }
  decode(_ coordinate: Float, into model: P) -> Result<P, CodecError> {
    self.codec.decode(coordinate).map(|value| { self.focus.set(model, to: value) })
  }
}
export Column, Parameter
