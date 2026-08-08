// Authoritative search engine. Search semantics are independent of formatting;
// optional typed report events expose phase and example progress.

import Phase from "core/phase"
import PropertyResult from "core/status"
import statistics from "core/statistics"
import errors from "core/errors"
import specification from "engine/specification"
import evaluator from "engine/evaluator"
import shrinker from "engine/shrinker"
import reportingEvent from "reporting/event"
import reportingReporter from "reporting/reporter"

const ReportEvent = reportingEvent.ReportEvent

class SearchEngine {
  check(spec: Any) -> PropertyResult {
    return self.check(
      spec,
      reporter: reportingReporter.NullReporter.new()
    )
  }

  check(spec: Any, reporter: Any) -> PropertyResult {
    const stats = statistics._StatisticsCollector.new()
    const checkedReporter = reportingReporter._CheckedReporter.new(reporter)
    const outcome = {
      self.check(spec, reporter: checkedReporter, statistics: stats)
    }.attempt()
    if outcome.isOk {
      return outcome.unwrap
    }
    const error = outcome.unwrapErr
    if error.isA(errors.ReporterFailure) {
      return PropertyResult.errored(
        id: spec.id,
        error: error,
        statistics: stats.snapshot
      )
    }
    error.raise()
  }

  @private
  check(spec: Any, reporter: Any, statistics: Any) -> PropertyResult {
    const stats = statistics
    const worker = evaluator._Evaluator.new(spec)
    let exampleIndex = 0

    // 1. Explicit examples are mandatory, run first, and never shrink.
    if self.phaseEnabled(spec.settings, Phase.Explicit) {
      reporter.handle(ReportEvent.phaseStarted(id: spec.id, phase: Phase.Explicit))
      for arguments in spec.explicitExamples {
        const status = worker.explicit(arguments).status
        if status.passed {
          reporter.handle(
            ReportEvent.exampleAccepted(
              id: spec.id,
              index: exampleIndex,
              example: status.tape,
              arguments: status.args,
              context: status.context
            )
          )
        } else if status.failed {
          stats.recordFailure(status.context)
          reporter.handle(
            ReportEvent.failureFound(
              id: spec.id,
              failure: self.failure(status)
            )
          )
          return PropertyResult.falsified(
            id: spec.id,
            failure: self.failure(status),
            statistics: stats.snapshot
          )
        } else if status.invalid {
          stats.recordReject(status.context)
          reporter.handle(
            ReportEvent.exampleRejected(
              id: spec.id,
              index: exampleIndex,
              reason: status.error,
              example: status.tape,
              arguments: status.args,
              context: status.context
            )
          )
          return PropertyResult.inconclusive(
            id: spec.id,
            reason: errors._UnsatisfiedAssumptions.new(
              "an explicit example was rejected"
            ),
            statistics: stats.snapshot
          )
        } else if status.overrun {
          self.reportOverrun(id: spec.id, status: status, reporter: reporter)
          return PropertyResult.errored(
            id: spec.id,
            error: status.error,
            statistics: stats.snapshot
          )
        }
        exampleIndex++
      }
    }

    let interesting = None

    // 2. Reuse supplied immutable examples. Stale invalid or overrun examples
    // are cache misses and are not ordinary counterexamples.
    if self.phaseEnabled(spec.settings, Phase.Reuse) {
      reporter.handle(ReportEvent.phaseStarted(id: spec.id, phase: Phase.Reuse))
      for example in spec.reuseExamples {
        stats.recordReplay()
        const status = worker.replay(example).status
        if status.failed {
          stats.recordFailure(status.context)
          reporter.handle(
            ReportEvent.failureFound(
              id: spec.id,
              failure: self.failure(status)
            )
          )
          interesting = Some.new(status)
          break
        }
        exampleIndex++
      }
    }

    // 3. Generate until enough valid examples pass or a failure is found.
    if interesting.isNone and self.phaseEnabled(spec.settings, Phase.Generate) {
      reporter.handle(ReportEvent.phaseStarted(id: spec.id, phase: Phase.Generate))
      const factory = spec.settings.resolvedChoiceProviderFactory
      let valid = 0
      let discarded = 0
      let generatedIndex = 0

      while valid < spec.settings.maxExamples and interesting.isNone {
        const size = (valid * 100) ~/ spec.settings.maxExamples
        const created = {
          factory.create(exampleIndex: generatedIndex, generationSize: size)
        }.attempt()
        if created.isErr {
          return PropertyResult.errored(
            id: spec.id,
            error: created.unwrapErr,
            statistics: stats.snapshot
          )
        }
        const status = worker.generated(created.unwrap, size).status

        if status.passed {
          valid++
          stats.recordPass(status.context)
          reporter.handle(
            ReportEvent.exampleAccepted(
              id: spec.id,
              index: exampleIndex,
              example: status.tape,
              arguments: status.args,
              context: status.context
            )
          )
        } else if status.invalid {
          discarded++
          stats.recordReject(status.context)
          reporter.handle(
            ReportEvent.exampleRejected(
              id: spec.id,
              index: exampleIndex,
              reason: status.error,
              example: status.tape,
              arguments: status.args,
              context: status.context
            )
          )
          if discarded > spec.settings.maxDiscards {
            return PropertyResult.inconclusive(
              id: spec.id,
              reason: errors._UnsatisfiedAssumptions.new(
                "discard limit exceeded before enough valid examples were generated"
              ),
              statistics: stats.snapshot
            )
          }
        } else if status.overrun {
          self.reportOverrun(id: spec.id, status: status, reporter: reporter)
          return PropertyResult.errored(
            id: spec.id,
            error: status.error,
            statistics: stats.snapshot
          )
        } else {
          stats.recordFailure(status.context)
          reporter.handle(
            ReportEvent.failureFound(
              id: spec.id,
              failure: self.failure(status)
            )
          )
          interesting = Some.new(status)
        }
        generatedIndex++
        exampleIndex++
      }
    }

    if interesting.isNone {
      return PropertyResult.passed(
        id: spec.id,
        statistics: stats.snapshot
      )
    }

    let minimal = interesting.unwrap

    // 4. Structural shrinking preserves the original source-aware origin.
    if self.phaseEnabled(spec.settings, Phase.Shrink) and spec.settings.maxShrinks > 0 {
      reporter.handle(ReportEvent.phaseStarted(id: spec.id, phase: Phase.Shrink))
      minimal = shrinker.Shrinker.standard.shrinkFailure(
        initial: minimal,
        evaluator: worker,
        maxShrinks: spec.settings.maxShrinks,
        statistics: stats,
        reporter: reporter,
        id: spec.id
      )
    }

    // 5. Final replay verification rejects non-reproducible counterexamples.
    const verified = self.verifyFailure(
      minimal: minimal,
      evaluator: worker,
      statistics: stats
    )
    if verified.isNone {
      return PropertyResult.errored(
        id: spec.id,
        error: errors._FlakyFailure.new(
          "minimal example did not reproduce the same failure origin"
        ),
        statistics: stats.snapshot
      )
    }

    return PropertyResult.falsified(
      id: spec.id,
      failure: self.failure(verified.unwrap),
      statistics: stats.snapshot
    )
  }

  find<T>(
    strategy: Any,
    predicate: [T] -> Bool,
    settings: Any,
    reuseExamples: List<Any>
  ) -> Option<T> {
    const spec = specification._FindSpec.create(
      strategy: strategy,
      predicate: predicate,
      reuseExamples: reuseExamples,
      settings: settings
    )
    const stats = statistics._StatisticsCollector.new()
    const worker = evaluator._Evaluator.new(spec)
    let found = None

    if self.phaseEnabled(settings, Phase.Reuse) {
      for example in reuseExamples {
        stats.recordReplay()
        const result = worker.replay(example)
        if result.found {
          found = Some.new(result)
          break
        }
        const status = result.status
        if status.failed {
          self.failure(status).error.raise()
        }
      }
    }

    if found.isNone and self.phaseEnabled(settings, Phase.Generate) {
      const factory = settings.resolvedChoiceProviderFactory
      let attempts = 0
      let discarded = 0

      while attempts < settings.maxExamples and found.isNone {
        const size = (attempts * 100) ~/ settings.maxExamples
        const created = {
          factory.create(exampleIndex: attempts, generationSize: size)
        }.attempt()
        if created.isErr {
          created.unwrapErr.raise()
        }
        const result = worker.generated(created.unwrap, size)
        if result.found {
          found = Some.new(result)
        } else {
          const status = result.status
          if status.invalid {
            discarded++
            if discarded > settings.maxDiscards {
              return None
            }
          } else if status.overrun {
            status.error.raise()
          } else if status.failed {
            self.failure(status).error.raise()
          }
        }
        attempts++
      }
    }

    if found.isNone {
      return None
    }

    let minimal = found.unwrap
    if self.phaseEnabled(settings, Phase.Shrink) and settings.maxShrinks > 0 {
      minimal = shrinker.Shrinker.standard.shrinkFound(
        initial: minimal,
        evaluator: worker,
        maxShrinks: settings.maxShrinks,
        statistics: stats
      )
    }

    let verification = 0
    while verification < 2 {
      stats.recordReplay()
      const replay = worker.replay(minimal.example)
      if not replay.found {
        throw errors._FlakyFailure.new(
          "minimal satisfying example did not reproduce"
        )
      }
      minimal = replay
      verification++
    }

    return Some.new(minimal.value)
  }

  verifyFailure(
    minimal: Any,
    evaluator: Any,
    statistics: Any
  ) -> Option<Any> {
    let verified = minimal
    const expected = self.failure(minimal)
    let replay = 0
    while replay < 2 {
      statistics.recordReplay()
      const candidate = evaluator.replay(verified.tape).status
      if not candidate.failed {
        return None
      }
      if not expected.sameOrigin(self.failure(candidate)) {
        return None
      }
      verified = candidate
      replay++
    }
    return Some.new(verified)
  }

  reportOverrun(id: Any, status: Any, reporter: Any) -> None {
    let classified = status.error
    if status.error.respondsTo(#primaryError) {
      classified = status.error.primaryError
    }
    if classified.isA(errors._HealthCheckFailure) {
      reporter.handle(
        ReportEvent.healthCheckFailed(id: id, error: status.error)
      )
    }
  }

  phaseEnabled(settings: Any, phase: Any) -> Bool {
    return settings.phases.includes(phase)
  }

  failure(status: Any) -> Any {
    return status.match(
      valid: |_| { throw Error.new("valid example has no failure") },
      invalid: |_| { throw Error.new("invalid example has no failure") },
      overrun: |_| { throw Error.new("overrun example has no failure") },
      interesting: |value| { value.failure }
    )
  }
}
