// Sealed outcomes for one evaluated example and one completed property.
// Variant identity replaces the prototype's free-form string tags.

import Failure from "core/failure"
import Statistics from "core/statistics"
import errors from "core/errors"

@data
@immutable
@sealed
class ExampleStatus {
  @variant Valid(example:, arguments:, context:)
  @variant Invalid(reason:, example:, arguments:, context:)
  @variant Overrun(reason:, example:, arguments:, context:)
  @variant Interesting(failure:, example:, arguments:, context:)

  @class
  valid(example: Any, arguments: List<Any>, context: Any) -> ExampleStatus {
    return Valid.new(example: example, arguments: arguments, context: context)
  }

  @class
  invalid(reason: Any, example: Any, arguments: List<Any>, context: Any) -> ExampleStatus {
    return Invalid.new(
      reason: reason,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  @class
  overrun(reason: Any, example: Any, arguments: List<Any>, context: Any) -> ExampleStatus {
    return Overrun.new(
      reason: reason,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  @class
  interesting(failure: Failure, example: Any, arguments: List<Any>, context: Any) -> ExampleStatus {
    return Interesting.new(
      failure: failure,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  // Compatibility construction names used by the temporary engine.
  @class
  passed(example: Any, arguments: List<Any>, context: Any) -> ExampleStatus {
    return self.valid(example: example, arguments: arguments, context: context)
  }

  @class
  rejected(example: Any, arguments: List<Any>, error: Error, context: Any) -> ExampleStatus {
    return self.invalid(
      reason: error,
      example: example,
      arguments: arguments,
      context: context
    )
  }


  @class
  failed(example: Any, arguments: List<Any>, error: Error, context: Any) -> ExampleStatus {
    return self.interesting(
      failure: Failure.from(error, example, arguments),
      example: example,
      arguments: arguments,
      context: context
    )
  }

  passed -> Bool {
    return self.match(
      valid: { _ => true },
      invalid: { _ => false },
      overrun: { _ => false },
      interesting: { _ => false }
    )
  }

  rejected -> Bool {
    return self.match(
      valid: { _ => false },
      invalid: { _ => true },
      overrun: { _ => false },
      interesting: { _ => false }
    )
  }

  invalid -> Bool => self.rejected

  overrun -> Bool {
    return self.match(
      valid: { _ => false },
      invalid: { _ => false },
      overrun: { _ => true },
      interesting: { _ => false }
    )
  }

  failed -> Bool {
    return self.match(
      valid: { _ => false },
      invalid: { _ => false },
      overrun: { _ => false },
      interesting: { _ => true }
    )
  }

  tape -> Any {
    return self.match(
      valid: { value => value.example },
      invalid: { value => value.example },
      overrun: { value => value.example },
      interesting: { value => value.example }
    )
  }

  args -> List<Any> {
    return self.match(
      valid: { value => value.arguments },
      invalid: { value => value.arguments },
      overrun: { value => value.arguments },
      interesting: { value => value.arguments }
    )
  }

  error -> Any {
    return self.match(
      valid: { _ => None },
      invalid: { value => value.reason },
      overrun: { value => value.reason },
      interesting: { value => value.failure.error }
    )
  }

  context -> Any {
    return self.match(
      valid: { value => value.context },
      invalid: { value => value.context },
      overrun: { value => value.context },
      interesting: { value => value.context }
    )
  }
}

@data
@immutable
@sealed
class PropertyResult {
  @variant Passed(id:, statistics:)
  @variant Falsified(id:, failure:, statistics:)
  @variant Inconclusive(id:, reason:, statistics:)
  @variant Errored(id:, error:, statistics:)

  @class
  passed(id: Any, statistics: Statistics) -> PropertyResult {
    return Passed.new(id: id, statistics: statistics)
  }

  @class
  falsified(id: Any, failure: Failure, statistics: Statistics) -> PropertyResult {
    return Falsified.new(id: id, failure: failure, statistics: statistics)
  }

  @class
  inconclusive(id: Any, reason: Any, statistics: Statistics) -> PropertyResult {
    return Inconclusive.new(id: id, reason: reason, statistics: statistics)
  }

  @class
  errored(id: Any, error: Error, statistics: Statistics) -> PropertyResult {
    return Errored.new(id: id, error: error, statistics: statistics)
  }

  // Compatibility factories used by the Phase 01 runner.
  @class
  pass(id: Any, statistics: Statistics) -> PropertyResult {
    return self.passed(id: id, statistics: statistics)
  }

  @class
  fail(
    id: Any,
    error: Error,
    arguments: List<Any>,
    example: Any,
    statistics: Statistics
  ) -> PropertyResult {
    return self.falsified(
      id: id,
      failure: Failure.from(error, example, arguments),
      statistics: statistics
    )
  }

  passed -> Bool {
    return self.match(
      passed: { _ => true },
      falsified: { _ => false },
      inconclusive: { _ => false },
      errored: { _ => false }
    )
  }

  failed -> Bool {
    return self.match(
      passed: { _ => false },
      falsified: { _ => true },
      inconclusive: { _ => false },
      errored: { _ => true }
    )
  }

  name -> Any {
    return self.match(
      passed: { value => value.id },
      falsified: { value => value.id },
      inconclusive: { value => value.id },
      errored: { value => value.id }
    )
  }

  stats -> Statistics {
    return self.match(
      passed: { value => value.statistics },
      falsified: { value => value.statistics },
      inconclusive: { value => value.statistics },
      errored: { value => value.statistics }
    )
  }

  error -> Any {
    return self.match(
      passed: { _ => None },
      falsified: { value => value.failure.error },
      inconclusive: { value =>
        if value.reason.isA(Error) {
          return value.reason
        }
        return errors._InconclusiveProperty.new(value.reason.toString)
      },
      errored: { value => value.error }
    )
  }

  args -> List<Any> {
    return self.match(
      passed: { _ => const [] },
      falsified: { value => value.failure.arguments },
      inconclusive: { _ => const [] },
      errored: { _ => const [] }
    )
  }

  tape -> Any {
    return self.match(
      passed: { _ => None },
      falsified: { value => Some.new(value.failure.example) },
      inconclusive: { _ => None },
      errored: { _ => None }
    )
  }
}
