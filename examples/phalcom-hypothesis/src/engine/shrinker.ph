// Greedy ordered structural shrinking over immutable candidate examples.
//
// Custom ShrinkPass implementations may only propose candidates. This
// authoritative shrinker deduplicates proposals, enforces strict complexity
// decrease, replays the complete target, and preserves failure origin.

import ExampleComplexity from "engine/complexity"
import ShrinkPass from "engine/shrink_pass"
import passes from "engine/shrink_pass"
import search from "engine/search"
import reportingEvent from "reporting/event"
import reportingReporter from "reporting/reporter"

const ReportEvent = reportingEvent.ReportEvent

class Shrinker {
  @constructor
  new(passes: List<ShrinkPass>) {
    _passes = _ShrinkCopies.passes(passes)
    _acceptedComplexities = List.new()
  }

  @class
  standard -> Shrinker {
    return Shrinker.new(
      passes: const [
        passes._DeleteDiscardableSpans.new(),
        passes._ShortenTrailingChoices.new(),
        passes._MinimizeBranchIndices.new(),
        passes._MinimizeIntegerChoices.new(),
        passes._MinimizeIntegerBlocks.new(),
        passes._SimplifyBytesAndText.new(),
        passes._MinimizeRecursiveStructures.new()
      ]
    )
  }

  acceptedComplexities -> List<ExampleComplexity> {
    const copied = List.new()
    for complexity in _acceptedComplexities {
      copied.add(complexity)
    }
    return copied
  }

  shrinkFailure(
    initial: Any,
    evaluator: Any,
    maxShrinks: Int,
    statistics: Any
  ) -> Any {
    return self.shrinkFailure(
      initial: initial,
      evaluator: evaluator,
      maxShrinks: maxShrinks,
      statistics: statistics,
      reporter: reportingReporter.NullReporter.new(),
      id: #unknown
    )
  }

  shrinkFailure(
    initial: Any,
    evaluator: Any,
    maxShrinks: Int,
    statistics: Any,
    reporter: Any,
    id: Any
  ) -> Any {
    _acceptedComplexities = List.new()
    let current = initial
    let accepted = 0
    let changed = true

    while changed and accepted < maxShrinks {
      changed = false
      const currentComplexity = ExampleComplexity.of(current.tape)
      const seenSignatures = Set.new()

      for pass in _passes {
        for proposal in pass.candidates(current.tape) {
          const signature = proposal.signature + "|" + proposal.spanSignature
          if not seenSignatures.includes(signature) {
            seenSignatures.add(signature)
            const proposalComplexity = ExampleComplexity.of(proposal)
            if proposalComplexity.lessThan(currentComplexity) {
              const replayed = evaluator.replay(proposal)
              let candidate = replayed
              if replayed.respondsTo(#status) {
                candidate = replayed.status
              }

              if not candidate.invalid and not candidate.overrun {
                if candidate.failed and self.failure(current).sameOrigin(self.failure(candidate)) {
                  const candidateComplexity = ExampleComplexity.of(candidate.tape)
                  if candidateComplexity.lessThan(currentComplexity) {
                    const before = current.tape
                    current = candidate
                    _acceptedComplexities.add(candidateComplexity)
                    accepted++
                    if statistics != None {
                      statistics.recordShrink()
                    }
                    reporter.handle(
                      ReportEvent.shrinkAccepted(
                        id: id,
                        before: before,
                        after: candidate.tape
                      )
                    )
                    changed = true
                    break
                  }
                }
              }
            }
          }
        }
        if changed {
          break
        }
      }
    }
    return current
  }

  shrinkFound(
    initial: search._SearchResult<Any>,
    evaluator: Any,
    maxShrinks: Int,
    statistics: Any
  ) -> search._SearchResult<Any> {
    _acceptedComplexities = List.new()
    let current = initial
    let accepted = 0
    let changed = true

    while changed and accepted < maxShrinks {
      changed = false
      const currentComplexity = ExampleComplexity.of(current.example)
      const seenSignatures = Set.new()

      for pass in _passes {
        for proposal in pass.candidates(current.example) {
          const signature = proposal.signature + "|" + proposal.spanSignature
          if not seenSignatures.includes(signature) {
            seenSignatures.add(signature)
            const proposalComplexity = ExampleComplexity.of(proposal)
            if proposalComplexity.lessThan(currentComplexity) {
              const candidate = evaluator.replay(proposal)
              if candidate.found {
                const candidateComplexity = ExampleComplexity.of(candidate.example)
                if candidateComplexity.lessThan(currentComplexity) {
                  current = candidate
                  _acceptedComplexities.add(candidateComplexity)
                  accepted++
                  if statistics != None {
                    statistics.recordShrink()
                  }
                  changed = true
                  break
                }
              }
            }
          }
        }
        if changed {
          break
        }
      }
    }
    return current
  }

  failure(status: Any) -> Any {
    return status.match(
      valid: { _ => throw Error.new("valid example has no failure") },
      invalid: { _ => throw Error.new("invalid example has no failure") },
      overrun: { _ => throw Error.new("overrun example has no failure") },
      interesting: { value => value.failure }
    )
  }
}

class _ShrinkCopies {
  @class
  passes(values: List<ShrinkPass>) -> List<ShrinkPass> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
