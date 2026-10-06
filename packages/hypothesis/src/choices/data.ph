import .provider as Providers
import .buffer as Buffers
import .request as Requests
import .choice as Choices
import .example as Examples
from .request import ChoiceRequest
from .choice import Choice

/// Strategy-facing choice access and the mutable buffer for one example.
class DrawData<P>
    where P |= Providers.ChoiceProvider
{
  mut _provider: P
  mut _buffer: Buffers.ChoiceBuffer

  @constructor
  new(provider: P, buffer: Buffers.ChoiceBuffer) {
    _provider = provider
    _buffer = buffer
  }

  /// Requests and records one normalized primitive choice.
  draw(request: Requests.ChoiceRequest) -> Choices.Choice {
    let choice = _provider.choose(request: request)
    _buffer.append(choice)
    choice
  }

  /// Number of choices consumed from the provider.
  consumedChoices -> Int { _provider.consumedChoices }

  /// Whether a finite provider tape was consumed in full.
  fullyConsumed -> Bool {
    match _provider.remainingChoices {
      None => true
      Some(remaining) => remaining == 0
    }
  }

  /// Draws an integer without exposing the heterogeneous choice sum.
  drawInt(min: Int, max: Int, shrinkTowards: Int) -> Int {
    match self.draw(request: ChoiceRequest::Integer(min, max, shrinkTowards)) {
      Choice::Integer(value, _, _, _) => value
      _ => {
        throw Error.new("integer request returned a non-integer choice")
      }
    }
  }

  /// Draws a boolean without exposing the heterogeneous choice sum.
  drawBool(shrinkTowards: Bool) -> Bool {
    match self.draw(request: ChoiceRequest::Boolean(shrinkTowards)) {
      Choice::Boolean(value, _) => value
      _ => {
        throw Error.new("boolean request returned a non-boolean choice")
      }
    }
  }

  /// Draws an index in `[0, size)`.
  drawIndex(size: Int, shrinkTowards: Int) -> Int {
    match self.draw(request: ChoiceRequest::Index(size, shrinkTowards)) {
      Choice::Index(value, _, _) => value
      _ => {
        throw Error.new("index request returned a non-index choice")
      }
    }
  }

  /// Draws a byte sequence and records a defensive copy in the example.
  drawBytes(minSize: Int, maxSize: Int, shrinkTowards: Bytes) -> Bytes {
    match self.draw(
      request: ChoiceRequest::Bytes(minSize, maxSize, shrinkTowards)
    ) {
      Choice::Bytes(value, _, _, _) => value
      _ => {
        throw Error.new("byte request returned a non-byte choice")
      }
    }
  }

  /// Returns an immutable snapshot of the choices consumed so far.
  example -> Examples.Example { _buffer.snapshot }
}

export DrawData
