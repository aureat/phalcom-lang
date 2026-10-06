import .failure as Failures
import ..choices.example as Examples

/// The typed outcome of evaluating one generated input.
data ExampleStatus<T> {
  /// The generated value satisfied the property.
  Passed(
    _ value: T,
    _ example: Examples.Example
  ),

  /// Reserved for a future rejection/assumption API; current evaluators do not
  /// produce discarded examples.
  Discarded(_ reason: String),

  /// The generated value falsified the property.
  Falsified(
    _ value: T,
    _ failure: Failures.Failure,
    _ example: Examples.Example
  ),

  /// Strategy/provider execution could not produce a valid evaluation.
  Errored(_ reason: String),
}

export ExampleStatus
