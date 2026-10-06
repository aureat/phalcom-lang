import hypothesis.core.failure as Failures
import hypothesis.core.phase as Phases
import hypothesis_tests.support.test as Testing

let first = Failures.Failure(
  exampleIndex: 3,
  phase: Phases.Phase.Generate,
  origin: Failures.FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #integerProperty
  ),
  message: "value is too large"
)
let sameOrigin = Failures.Failure(
  exampleIndex: 9,
  phase: Phases.Phase.Shrink,
  origin: Failures.FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #integerProperty
  ),
  message: "value is too large"
)
let differentOrigin = Failures.Failure(
  exampleIndex: 3,
  phase: Phases.Phase.Generate,
  origin: Failures.FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #stringProperty
  ),
  message: "value is too large"
)

Testing.Test.expect(
  first.sameOrigin(sameOrigin),
  "matching origins must remain stable across phases"
)
Testing.Test.expect(
  not first.sameOrigin(differentOrigin),
  "different origins must not compare equal"
)
Testing.Test.pass("core.failure_origin")
