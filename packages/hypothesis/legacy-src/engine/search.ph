// Search-result wrapper shared by ordinary properties and find-mode searches.
// A satisfying find result is a value variant, never a success exception.

import ExampleStatus from "core/status"
import Example from "choices/example"

@data
@immutable
@sealed
class _SearchResult<T> {
  @variant Evaluated(status:)
  @variant Found(value:, example:, arguments:, context:)

  @class
  evaluated(status: ExampleStatus) -> _SearchResult<T> {
    return Evaluated.new(status: status)
  }

  @class
  found(
    value: T,
    example: Example,
    arguments: List<Any>,
    context: Any
  ) -> _SearchResult<T> {
    return Found.new(
      value: value,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  found -> Bool {
    return self.match(
      evaluated: |_| { false },
      found: |_| { true }
    )
  }

  status -> ExampleStatus {
    return self.match(
      evaluated: |value| { value.status },
      found: |_| { throw Error.new("a found search result has no ExampleStatus") }
    )
  }

  example -> Example {
    return self.match(
      evaluated: |value| { value.status.tape },
      found: |value| { value.example }
    )
  }

  value -> T {
    return self.match(
      evaluated: |_| { throw Error.new("an evaluated search result has no found value") },
      found: |value| { value.value }
    )
  }

  arguments -> List<Any> {
    return self.match(
      evaluated: |value| { value.status.args },
      found: |value| { value.arguments }
    )
  }

  context -> Any {
    return self.match(
      evaluated: |value| { value.status.context },
      found: |value| { value.context }
    )
  }
}
