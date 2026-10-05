import .provider as Providers
import .buffer as Buffers
import .request as Requests
import .choice as Choices
import .example as Examples

/// Strategy-facing choice access and the mutable buffer for one example.
class DrawData {
  mut _provider: Providers.ChoiceProvider
  mut _buffer: Buffers.ChoiceBuffer

  @constructor
  new(provider: Providers.ChoiceProvider, buffer: Buffers.ChoiceBuffer) {
    _provider = provider
    _buffer = buffer
  }

  /// Requests and records one normalized primitive choice.
  draw(request: Requests.ChoiceRequest) -> Choices.Choice {
    let choice = _provider.choose(request)
    _buffer.append(choice)
    choice
  }

  /// Number of choices consumed from the provider.
  consumedChoices -> Int { _provider.consumedChoices }

  /// Draws an integer without exposing the heterogeneous choice sum.
  drawInt(min: Int, max: Int, shrinkTowards: Int) -> Int {
    match self.draw(Requests.ChoiceRequest::Integer(min, max, shrinkTowards)) {
      Choices.Choice::Integer(value, _, _, _) => value
      _ => {
        throw Error.new("integer request returned a non-integer choice")
      }
    }
  }

  /// Draws a boolean without exposing the heterogeneous choice sum.
  drawBool(shrinkTowards: Bool) -> Bool {
    match self.draw(Requests.ChoiceRequest::Boolean(shrinkTowards)) {
      Choices.Choice::Boolean(value, _) => value
      _ => {
        throw Error.new("boolean request returned a non-boolean choice")
      }
    }
  }

  /// Draws an index in `[0, size)`.
  drawIndex(size: Int, shrinkTowards: Int) -> Int {
    match self.draw(Requests.ChoiceRequest::Index(size, shrinkTowards)) {
      Choices.Choice::Index(value, _, _) => value
      _ => {
        throw Error.new("index request returned a non-index choice")
      }
    }
  }

  /// Draws a byte sequence and records a defensive copy in the example.
  drawBytes(minSize: Int, maxSize: Int, shrinkTowards: Bytes) -> Bytes {
    match self.draw(
      Requests.ChoiceRequest::Bytes(minSize, maxSize, shrinkTowards)
    ) {
      Choices.Choice::Bytes(value, _, _, _) => value
      _ => {
        throw Error.new("byte request returned a non-byte choice")
      }
    }
  }

  /// Returns an immutable snapshot of the choices consumed so far.
  example -> Examples.Example { _buffer.snapshot }
}

export DrawData
