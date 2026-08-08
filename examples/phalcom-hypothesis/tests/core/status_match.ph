// Phase 02 — every status and result case participates in exhaustive sealed
// dispatch. No semantic state is represented by a free-form string tag.

import Assert from hypothesis
import PropertyResult from hypothesis
import Statistics from hypothesis
import status from "core/status"
import failure from "core/failure"

const ExampleStatus = status.ExampleStatus
const FailureOrigin = failure.FailureOrigin
const Failure = failure.Failure

const origin = FailureOrigin.new(
  errorType: Error,
  module: #statusMatch,
  selector: #property,
  line: 12,
  column: 3,
  label: None
)
const found = Failure.new(
  origin: origin,
  error: Error.new("boom"),
  example: None,
  arguments: const [],
  notes: const []
)
const statistics = Statistics.empty

const statuses = const [
  ExampleStatus.valid(example: None, arguments: const [], context: None),
  ExampleStatus.invalid(reason: #assumption, example: None, arguments: const [], context: None),
  ExampleStatus.overrun(reason: #choiceBudget, example: None, arguments: const [], context: None),
  ExampleStatus.interesting(failure: found, example: None, arguments: const [], context: None)
]

const statusLabels = List.new()
for item in statuses {
  statusLabels.add(
    item.match(
      valid: |_| { #valid },
      invalid: |_| { #invalid },
      overrun: |_| { #overrun },
      interesting: |_| { #interesting }
    )
  )
}
Assert.equal(const [#valid, #invalid, #overrun, #interesting], statusLabels)
Assert.isTrue(statuses.at(0).passed)
Assert.isTrue(statuses.at(1).invalid)
Assert.isTrue(statuses.at(2).overrun)
Assert.isTrue(statuses.at(3).failed)

const results = const [
  PropertyResult.passed(id: #p, statistics: statistics),
  PropertyResult.falsified(id: #p, failure: found, statistics: statistics),
  PropertyResult.inconclusive(id: #p, reason: #discardLimit, statistics: statistics),
  PropertyResult.errored(id: #p, error: Error.new("engine"), statistics: statistics)
]

const resultLabels = List.new()
for item in results {
  resultLabels.add(
    item.match(
      passed: |_| { #passed },
      falsified: |_| { #falsified },
      inconclusive: |_| { #inconclusive },
      errored: |_| { #errored }
    )
  )
}
Assert.equal(const [#passed, #falsified, #inconclusive, #errored], resultLabels)
Assert.isTrue(results.at(0).passed)
Assert.isTrue(results.at(1).failed)
Assert.isFalse(results.at(2).failed)
Assert.isTrue(results.at(3).failed)

System.print("PASS core status match")
