// Phase 02 — failure identity is source-aware. Two assertions with the same
// error class but different source sites are not the same failure origin.

import Assert from hypothesis
import failure from "core/failure"

const FailureOrigin = failure.FailureOrigin

const first = FailureOrigin.new(
  errorType: Error,
  module: #arithmetic,
  selector: #law,
  line: 20,
  column: 5,
  label: None
)
const second = FailureOrigin.new(
  errorType: Error,
  module: #arithmetic,
  selector: #law,
  line: 27,
  column: 5,
  label: None
)
const labeled = FailureOrigin.new(
  errorType: Error,
  module: #arithmetic,
  selector: #law,
  line: 20,
  column: 5,
  label: Some.new(#roundTrip)
)

Assert.isTrue(first != second)
Assert.isTrue(first != labeled)
Assert.isTrue(first.sameSite(first))
Assert.isFalse(first.sameSite(second))

System.print("PASS core failure origin")
