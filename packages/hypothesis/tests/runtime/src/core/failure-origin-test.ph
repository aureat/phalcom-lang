from hypothesis.core.failure import Failure, FailureOrigin
from hypothesis.core.phase import Phase
import hypothesis_tests.support.test as Testing

let first = Failure(
  exampleIndex: 3,
  phase: Phase::Generate,
  origin: FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #integerProperty
  ),
  message: "value is too large"
)
let sameOrigin = Failure(
  exampleIndex: 9,
  phase: Phase::Shrink,
  origin: FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #integerProperty
  ),
  message: "value is too large"
)
let differentOrigin = Failure(
  exampleIndex: 3,
  phase: Phase::Generate,
  origin: FailureOrigin(
    kind: #propertyReturnedFalse,
    property: #stringProperty
  ),
  message: "value is too large"
)

Testing.Test.expect(
  first.sameOrigin(other: sameOrigin),
  "matching origins must remain stable across phases"
)
Testing.Test.expect(
  not first.sameOrigin(other: differentOrigin),
  "different origins must not compare equal"
)
Testing.Test.pass("core.failure_origin")
